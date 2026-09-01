//! Pure deterministic transitions over already validated numeric/account facts.
//!
//! Future instruction handlers remain responsible for Clock/sysvar decoding,
//! protocol accounts, stake inactivity, balance deltas, and CPI. Every
//! multi-object transition stages complete copies and commits only after all
//! checks pass, so an error leaves every supplied state object unchanged.

use core::cmp::{max, min};

use anchor_lang::prelude::Pubkey;

use crate::{
    constants::{
        GUARDIAN_COUNT, RECOVERY_FLAG_COOLDOWN_LOSS, RECOVERY_FLAG_RESIDUAL_HWM,
    },
    errors::{Piv1Error, Piv1Result},
    state::{
        derive_kif_period,
        distribution::{
            ActiveDistribution, CompletedDistributionSummary, DistributionLifecycle,
            WithdrawalLeg, WithdrawalLegStatus,
        },
        guardian::{GuardianRegistry, GuardianReward},
        timing::{validate_insufficient_retry, validate_preparation_interval},
        PivConfig,
    },
};

/// Tagged funding route prevents a liquid-only zero JitoSOL target from being
/// confused with an invalid zero-target delayed-withdrawal round.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum DistributionFunding {
    /// Pending native SOL fully covers the fixed outgoing gross obligation.
    Liquid {
        escrow_available_lamports: u64,
    },
    /// A nonzero JitoSOL target is required for the native shortfall.
    Withdrawal {
        fixed_jitosol_target_units: u64,
        snapshot_leg_input_floor_units: u64,
        maximum_useful_legs: u64,
        stored_round_minimum_native_lamports: u64,
        initial_escrow_available_lamports: u64,
    },
}

/// Immutable facts fixed when a positive-yield distribution opens.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct OpenDistributionInput {
    pub sequence: u64,
    pub prepared_at: i64,
    pub prepared_slot: u64,
    pub prepared_epoch: u64,
    pub historical_jitosol_units: u64,
    pub historical_sol_lamports: u64,
    pub historical_value_lamports: u64,
    pub snapshot_pool_total_lamports: u64,
    pub snapshot_pool_token_supply: u64,
    pub snapshot_withdrawal_fee_numerator: u64,
    pub snapshot_withdrawal_fee_denominator: u64,
    pub gross_yield_lamports: u64,
    pub pending_sol_snapshot_lamports: u64,
    pub pending_sol_used_lamports: u64,
    pub snapshot_conversion_dust_lamports: u64,
    pub stored_residual_hwm_floor_lamports: u64,
    pub funding: DistributionFunding,
}

/// Already protocol-validated facts recorded for one maximum-safe leg fill.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct LegInitiationInput {
    pub sequence: u64,
    pub leg_index: u64,
    pub validator_list_index: u32,
    pub validator_seed_suffix: u32,
    pub validator_vote: Pubkey,
    pub validator_stake_source: Pubkey,
    pub initiation_epoch: u64,
    pub pool_total_lamports: u64,
    pub pool_token_supply: u64,
    pub withdrawal_fee_numerator: u64,
    pub withdrawal_fee_denominator: u64,
    pub current_technical_floor_units: u64,
    pub maximum_safe_capacity_units: u64,
    pub jitosol_input_units: u64,
    pub withdrawal_fee_units: u64,
    pub burned_units: u64,
    pub expected_native_lamports: u64,
    pub observed_delegated_native_lamports: u64,
    pub minimum_native_lamports: u64,
    pub stake_rent_advanced_lamports: u64,
    pub metadata_rent_advanced_lamports: u64,
}

/// Already inactivity- and balance-validated facts for one leg finalization.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct LegFinalizationInput {
    pub sequence: u64,
    pub leg_index: u64,
    pub finalized_epoch: u64,
    pub finalized_native_lamports: u64,
    pub recovered_stake_rent_lamports: u64,
    pub recovered_metadata_rent_lamports: u64,
    pub cooldown_reward_lamports: u64,
    pub cooldown_loss_lamports: u64,
    pub validated_residual_historical_value_lamports: u64,
    pub escrow_available_after_lamports: u64,
}

/// State-level result of a successful leg finalization record.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum LegFinalizationOutcome {
    /// The leg finalized while more target assignment or legs remain.
    Recorded,
    /// Exact assignment and all successful-leg finalizations now reconcile.
    EscrowFunded,
    /// A recorded loss or residual-HWM failure blocks normal progression.
    RecoveryRequired,
}

/// Exact state-level escrow and protected-value facts for atomic settlement.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct SettlementInput {
    pub sequence: u64,
    pub escrow_available_lamports: u64,
    /// Future-handler-validated actual HTFP amount, bounded here by its fixed obligation.
    pub actual_htfp_lamports: u64,
    /// Future-handler-validated actual Team Owner amount, bounded here by its fixed obligation.
    pub actual_team_owner_lamports: u64,
    /// Future-handler-validated current KIF amount, before applying prior carry.
    pub actual_kif_allocation_lamports: u64,
    pub validated_post_settlement_protected_value_lamports: u64,
}

/// State-level result of evaluating a settlement boundary.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SettlementOutcome {
    /// Beneficiary/KIF/HWM accounting was atomically recorded.
    Settled,
    /// The supplied validated protected value requires governed recovery.
    RecoveryRequired,
}

/// Already custody-validated facts for post-settlement pending integration.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct PendingIntegrationInput {
    pub sequence: u64,
    pub completed_at: i64,
    pub integrated_pending_sol_lamports: u64,
    pub integrated_pending_jitosol_units: u64,
    pub contribution_value_lamports: u64,
    pub new_accounted_historical_jitosol_units: u64,
    pub new_accounted_historical_sol_lamports: u64,
    pub new_protected_hwm_lamports: u64,
}

/// Validates a no-yield result without modifying either timing clock.
pub fn record_no_yield_evaluation(
    config: &PivConfig,
    round: &ActiveDistribution,
    evaluated_at: i64,
    historical_value_lamports: u64,
) -> Piv1Result<()> {
    config.ensure_unpaused()?;
    round.validate()?;
    if round.lifecycle != DistributionLifecycle::Idle {
        return Err(Piv1Error::InvalidLifecycle);
    }
    validate_preparation_interval(config.last_successful_preparation_at, evaluated_at)?;
    let historical_yield = piv1_math::calculate_gross_yield(
        historical_value_lamports,
        config.protected_principal_hwm_lamports,
    );
    if checked_add(historical_yield, config.next_cycle_yield_lamports)? != 0 {
        return Err(Piv1Error::CumulativeReconciliationMismatch);
    }
    Ok(())
}

/// Records the only mutation allowed for a valid technically insufficient result.
pub fn record_valid_insufficient_attempt(
    config: &mut PivConfig,
    round: &ActiveDistribution,
    attempted_at: i64,
) -> Piv1Result<()> {
    config.ensure_unpaused()?;
    round.validate()?;
    if round.lifecycle != DistributionLifecycle::Idle {
        return Err(Piv1Error::InvalidLifecycle);
    }
    validate_preparation_interval(config.last_successful_preparation_at, attempted_at)?;
    validate_insufficient_retry(config.last_valid_insufficient_attempt_at, attempted_at)?;

    let mut next_config = config.clone();
    next_config.last_valid_insufficient_attempt_at = Some(attempted_at);
    next_config.validate_initialized()?;
    *config = next_config;
    Ok(())
}

/// Opens exactly one liquid-only or delayed-withdrawal distribution round.
pub fn open_distribution(
    config: &mut PivConfig,
    round: &mut ActiveDistribution,
    registry: &GuardianRegistry,
    rewards: &[GuardianReward; GUARDIAN_COUNT],
    input: OpenDistributionInput,
) -> Piv1Result<()> {
    config.ensure_unpaused()?;
    round.validate()?;
    registry.validate()?;
    if round.lifecycle != DistributionLifecycle::Idle {
        return Err(Piv1Error::InvalidLifecycle);
    }
    validate_preparation_interval(config.last_successful_preparation_at, input.prepared_at)?;
    if input.sequence != config.next_distribution_sequence {
        return Err(Piv1Error::SequenceMismatch);
    }
    if round
        .last_completed
        .is_some_and(|summary| input.sequence <= summary.sequence)
    {
        return Err(Piv1Error::Replay);
    }
    if registry.revision != config.guardian_registry_revision {
        return Err(Piv1Error::InvalidGuardianSet);
    }
    if input.historical_jitosol_units != config.accounted_historical_jitosol_units
        || input.historical_sol_lamports != config.accounted_historical_sol_lamports
        || input.pending_sol_snapshot_lamports != config.accounted_pending_sol_lamports
    {
        return Err(Piv1Error::CumulativeReconciliationMismatch);
    }
    if input.snapshot_pool_total_lamports == 0 || input.snapshot_pool_token_supply == 0 {
        return Err(Piv1Error::ZeroTarget);
    }

    let historical_yield = piv1_math::calculate_gross_yield(
        input.historical_value_lamports,
        config.protected_principal_hwm_lamports,
    );
    let expected_yield = checked_add(historical_yield, config.next_cycle_yield_lamports)?;
    if input.gross_yield_lamports == 0 || input.gross_yield_lamports != expected_yield {
        return Err(Piv1Error::ZeroTarget);
    }
    let split = piv1_math::split_gross_yield(input.gross_yield_lamports)?;
    let outgoing = checked_add(
        checked_add(split.htfp_reserve, split.team_owner_pool)?,
        split.kif,
    )?;
    if outgoing == 0 {
        return Err(Piv1Error::ZeroTarget);
    }
    let required_pending_sol = min(input.pending_sol_snapshot_lamports, outgoing);
    if input.pending_sol_used_lamports != required_pending_sol {
        return Err(Piv1Error::CumulativeReconciliationMismatch);
    }
    let remaining_after_pending = checked_sub(outgoing, required_pending_sol)?;
    let prior_next_cycle_yield_used = min(
        config.next_cycle_yield_lamports,
        remaining_after_pending,
    );
    let initial_liquid_funding = checked_add(
        required_pending_sol,
        prior_next_cycle_yield_used,
    )?;

    let period = derive_kif_period(config.kif_anchor_timestamp, input.prepared_at)?;
    let bitmap = registry.activity_bitmap(rewards, period.id)?;
    let active_count = u8::try_from(bitmap.count_ones())
        .map_err(|_| Piv1Error::ArithmeticOverflow)?;
    let proposed_hwm_delta = checked_add(
        checked_add(split.permanent_compound, split.dust)?,
        input.snapshot_conversion_dust_lamports,
    )?;
    let proposed_hwm = checked_add(
        config.protected_principal_hwm_lamports,
        proposed_hwm_delta,
    )?;
    if input.stored_residual_hwm_floor_lamports < proposed_hwm {
        return Err(Piv1Error::HighWaterMarkDecrease);
    }

    let (
        lifecycle,
        fixed_target,
        snapshot_floor,
        maximum_useful_legs,
        stored_round_minimum,
        escrow_available,
    ) = match input.funding {
        DistributionFunding::Liquid {
            escrow_available_lamports,
        } => {
            if initial_liquid_funding != outgoing
                || escrow_available_lamports != initial_liquid_funding
            {
                return Err(Piv1Error::EscrowReconciliationMismatch);
            }
            (
                DistributionLifecycle::EscrowFunded,
                0,
                0,
                0,
                0,
                escrow_available_lamports,
            )
        }
        DistributionFunding::Withdrawal {
            fixed_jitosol_target_units,
            snapshot_leg_input_floor_units,
            maximum_useful_legs,
            stored_round_minimum_native_lamports,
            initial_escrow_available_lamports,
        } => {
            if fixed_jitosol_target_units == 0 {
                return Err(Piv1Error::ZeroTarget);
            }
            if initial_liquid_funding >= outgoing
                || initial_escrow_available_lamports != initial_liquid_funding
            {
                return Err(Piv1Error::EscrowReconciliationMismatch);
            }
            if snapshot_leg_input_floor_units == 0
                || fixed_jitosol_target_units < snapshot_leg_input_floor_units
                || stored_round_minimum_native_lamports == 0
            {
                return Err(Piv1Error::TechnicalFloorNotMet);
            }
            let expected_maximum =
                fixed_jitosol_target_units / snapshot_leg_input_floor_units;
            if expected_maximum == 0 || maximum_useful_legs != expected_maximum {
                return Err(Piv1Error::UsefulLegBoundExceeded);
            }
            (
                DistributionLifecycle::WithdrawalActive,
                fixed_jitosol_target_units,
                snapshot_leg_input_floor_units,
                maximum_useful_legs,
                stored_round_minimum_native_lamports,
                initial_escrow_available_lamports,
            )
        }
    };

    let mut next_config = config.clone();
    let allocated_sequence = next_config.allocate_next_distribution_sequence()?;
    if allocated_sequence != input.sequence {
        return Err(Piv1Error::SequenceMismatch);
    }
    next_config.last_successful_preparation_at = Some(input.prepared_at);
    next_config.next_cycle_yield_lamports = 0;

    let mut next_round = ActiveDistribution::new_idle(round.bump);
    next_round.last_completed = round.last_completed;
    next_round.lifecycle = lifecycle;
    next_round.active_sequence = input.sequence;
    next_round.prepared_at = input.prepared_at;
    next_round.prepared_slot = input.prepared_slot;
    next_round.prepared_epoch = input.prepared_epoch;
    next_round.old_protected_principal_lamports =
        config.protected_principal_hwm_lamports;
    next_round.historical_jitosol_units = input.historical_jitosol_units;
    next_round.historical_sol_lamports = input.historical_sol_lamports;
    next_round.historical_value_lamports = input.historical_value_lamports;
    next_round.snapshot_pool_total_lamports = input.snapshot_pool_total_lamports;
    next_round.snapshot_pool_token_supply = input.snapshot_pool_token_supply;
    next_round.snapshot_withdrawal_fee_numerator =
        input.snapshot_withdrawal_fee_numerator;
    next_round.snapshot_withdrawal_fee_denominator =
        input.snapshot_withdrawal_fee_denominator;
    next_round.gross_yield_lamports = input.gross_yield_lamports;
    next_round.prior_next_cycle_yield_lamports = config.next_cycle_yield_lamports;
    next_round.htfp_gross_obligation_lamports = split.htfp_reserve;
    next_round.permanent_compound_lamports = split.permanent_compound;
    next_round.team_owner_gross_obligation_lamports = split.team_owner_pool;
    next_round.kif_gross_obligation_lamports = split.kif;
    next_round.split_dust_lamports = split.dust;
    next_round.outgoing_gross_obligation_lamports = outgoing;
    next_round.pending_sol_snapshot_lamports = input.pending_sol_snapshot_lamports;
    next_round.pending_sol_used_lamports = input.pending_sol_used_lamports;
    next_round.snapshot_conversion_dust_lamports = input.snapshot_conversion_dust_lamports;
    next_round.fixed_jitosol_withdrawal_target_units = fixed_target;
    next_round.snapshot_leg_input_floor_units = snapshot_floor;
    next_round.maximum_useful_legs = maximum_useful_legs;
    next_round.stored_round_minimum_native_lamports = stored_round_minimum;
    next_round.stored_residual_hwm_floor_lamports =
        input.stored_residual_hwm_floor_lamports;
    next_round.stored_slippage_bps = config.configured_slippage_bps;
    next_round.recorded_escrow_available_lamports = escrow_available;
    next_round.outstanding_active_round_liability_lamports = outgoing;
    next_round.htfp_recipient = config.htfp_recipient;
    next_round.team_owner_recipient = config.team_owner_recipient;
    next_round.guardian_registry = config.guardian_registry;
    next_round.guardian_registry_revision = registry.revision;
    next_round.guardian_keys = registry.guardian_keys;
    next_round.kif_eligibility_bitmap = bitmap;
    next_round.kif_active_guardian_count = active_count;
    next_round.kif_period_id = period.id;
    next_round.kif_carry_input_lamports = config.collective_kif_carry_lamports;
    next_round.proposed_hwm_delta_lamports = proposed_hwm_delta;
    next_round.proposed_hwm_after_settlement_lamports = proposed_hwm;

    next_config.validate_initialized()?;
    next_round.validate()?;
    *config = next_config;
    *round = next_round;
    Ok(())
}

/// Initiates one exact maximum-safe withdrawal leg and consumes its index once.
pub fn initiate_withdrawal_leg(
    config: &PivConfig,
    round: &mut ActiveDistribution,
    leg: &mut WithdrawalLeg,
    input: LegInitiationInput,
) -> Piv1Result<()> {
    config.ensure_unpaused()?;
    round.validate()?;
    leg.validate()?;
    if round.lifecycle == DistributionLifecycle::RecoveryRequired {
        return Err(Piv1Error::RecoveryRequired);
    }
    if round.lifecycle != DistributionLifecycle::WithdrawalActive {
        return Err(Piv1Error::InvalidLifecycle);
    }
    if input.sequence != round.active_sequence {
        return Err(Piv1Error::SequenceMismatch);
    }
    validate_active_sequence_binding(config, round)?;
    if leg.status != WithdrawalLegStatus::Vacant {
        return Err(Piv1Error::Replay);
    }
    if input.leg_index != round.next_leg_index {
        return Err(Piv1Error::LegIndexMismatch);
    }
    if round.is_withdrawal_target_assigned() {
        return Err(Piv1Error::TargetExceeded);
    }
    if round.successful_leg_count >= round.maximum_useful_legs {
        return Err(Piv1Error::UsefulLegBoundExceeded);
    }
    if input.pool_total_lamports == 0
        || input.pool_token_supply == 0
        || input.validator_vote == Pubkey::default()
        || input.validator_stake_source == Pubkey::default()
    {
        return Err(Piv1Error::InvalidInitialization);
    }

    let remaining = round.remaining_withdrawal_target_units()?;
    if input.jitosol_input_units > remaining {
        return Err(Piv1Error::TargetExceeded);
    }
    let maximum_fill = min(remaining, input.maximum_safe_capacity_units);
    if input.jitosol_input_units == 0 || maximum_fill == 0 {
        return Err(Piv1Error::ZeroInput);
    }
    if input.jitosol_input_units != maximum_fill {
        return Err(Piv1Error::NonMaximumSafeLegFill);
    }
    let effective_floor = max(
        round.snapshot_leg_input_floor_units,
        input.current_technical_floor_units,
    );
    if effective_floor == 0 || input.jitosol_input_units < effective_floor {
        return Err(Piv1Error::TechnicalFloorNotMet);
    }
    let remaining_after = remaining
        .checked_sub(input.jitosol_input_units)
        .ok_or(Piv1Error::TargetExceeded)?;
    if remaining_after != 0 && remaining_after < effective_floor {
        return Err(Piv1Error::TechnicalFloorNotMet);
    }
    if checked_add(input.withdrawal_fee_units, input.burned_units)?
        != input.jitosol_input_units
        || input.minimum_native_lamports > input.expected_native_lamports
        || input.observed_delegated_native_lamports < input.minimum_native_lamports
    {
        return Err(Piv1Error::CumulativeReconciliationMismatch);
    }

    let mut next_round = *round;
    next_round.cumulative_jitosol_assigned_units = checked_add(
        next_round.cumulative_jitosol_assigned_units,
        input.jitosol_input_units,
    )?;
    if next_round.cumulative_jitosol_assigned_units
        > next_round.fixed_jitosol_withdrawal_target_units
    {
        return Err(Piv1Error::TargetExceeded);
    }
    next_round.cumulative_withdrawal_fee_units = checked_add(
        next_round.cumulative_withdrawal_fee_units,
        input.withdrawal_fee_units,
    )?;
    next_round.cumulative_burned_units =
        checked_add(next_round.cumulative_burned_units, input.burned_units)?;
    next_round.cumulative_expected_native_lamports = checked_add(
        next_round.cumulative_expected_native_lamports,
        input.expected_native_lamports,
    )?;
    next_round.cumulative_delegated_native_lamports = checked_add(
        next_round.cumulative_delegated_native_lamports,
        input.observed_delegated_native_lamports,
    )?;
    next_round.next_leg_index = next_round
        .next_leg_index
        .checked_add(1)
        .ok_or(Piv1Error::ArithmeticOverflow)?;
    next_round.successful_leg_count = next_round
        .successful_leg_count
        .checked_add(1)
        .ok_or(Piv1Error::ArithmeticOverflow)?;

    let next_leg = WithdrawalLeg {
        version: leg.version,
        metadata_bump: leg.metadata_bump,
        stake_bump: leg.stake_bump,
        is_initialized: true,
        status: WithdrawalLegStatus::Initiated,
        recovery_flags: 0,
        sequence: input.sequence,
        leg_index: input.leg_index,
        validator_list_index: input.validator_list_index,
        validator_seed_suffix: input.validator_seed_suffix,
        validator_vote: input.validator_vote,
        validator_stake_source: input.validator_stake_source,
        initiation_epoch: input.initiation_epoch,
        pool_total_lamports: input.pool_total_lamports,
        pool_token_supply: input.pool_token_supply,
        withdrawal_fee_numerator: input.withdrawal_fee_numerator,
        withdrawal_fee_denominator: input.withdrawal_fee_denominator,
        technical_floor_units: effective_floor,
        jitosol_input_units: input.jitosol_input_units,
        withdrawal_fee_units: input.withdrawal_fee_units,
        burned_units: input.burned_units,
        expected_native_lamports: input.expected_native_lamports,
        observed_delegated_native_lamports: input.observed_delegated_native_lamports,
        minimum_native_lamports: input.minimum_native_lamports,
        stake_rent_advanced_lamports: input.stake_rent_advanced_lamports,
        metadata_rent_advanced_lamports: input.metadata_rent_advanced_lamports,
        finalized_epoch: None,
        finalized_native_lamports: 0,
        recovered_stake_rent_lamports: 0,
        recovered_metadata_rent_lamports: 0,
        cooldown_reward_lamports: 0,
        cooldown_loss_lamports: 0,
    };

    next_leg.validate()?;
    next_round.validate()?;
    *round = next_round;
    *leg = next_leg;
    Ok(())
}

/// Finalizes one already validated inactive stake leg exactly once.
pub fn finalize_withdrawal_leg(
    config: &PivConfig,
    round: &mut ActiveDistribution,
    leg: &mut WithdrawalLeg,
    input: LegFinalizationInput,
) -> Piv1Result<LegFinalizationOutcome> {
    config.ensure_unpaused()?;
    round.validate()?;
    leg.validate()?;
    if round.lifecycle == DistributionLifecycle::RecoveryRequired {
        return Err(Piv1Error::RecoveryRequired);
    }
    if round.lifecycle != DistributionLifecycle::WithdrawalActive {
        return Err(Piv1Error::InvalidLifecycle);
    }
    if input.sequence != round.active_sequence || input.sequence != leg.sequence {
        return Err(Piv1Error::SequenceMismatch);
    }
    validate_active_sequence_binding(config, round)?;
    if input.leg_index != leg.leg_index {
        return Err(Piv1Error::LegIndexMismatch);
    }
    match leg.status {
        WithdrawalLegStatus::Vacant => return Err(Piv1Error::InvalidLifecycle),
        WithdrawalLegStatus::Finalized => return Err(Piv1Error::AlreadyFinalized),
        WithdrawalLegStatus::Initiated => {}
    }
    if leg.leg_index >= round.next_leg_index
        || leg.leg_index >= round.successful_leg_count
    {
        return Err(Piv1Error::LegIndexMismatch);
    }
    if input.finalized_epoch < leg.initiation_epoch {
        return Err(Piv1Error::TimestampRegression);
    }
    if input.recovered_stake_rent_lamports != leg.stake_rent_advanced_lamports
        || input.recovered_metadata_rent_lamports != leg.metadata_rent_advanced_lamports
        || (input.cooldown_reward_lamports > 0 && input.cooldown_loss_lamports > 0)
    {
        return Err(Piv1Error::CumulativeReconciliationMismatch);
    }
    let final_left = checked_add(
        checked_add(
            leg.observed_delegated_native_lamports,
            input.recovered_stake_rent_lamports,
        )?,
        input.cooldown_reward_lamports,
    )?;
    let final_right = checked_add(
        input.finalized_native_lamports,
        input.cooldown_loss_lamports,
    )?;
    if final_left != final_right {
        return Err(Piv1Error::CumulativeReconciliationMismatch);
    }

    let mut recovery_flags = 0_u8;
    if input.cooldown_loss_lamports > 0 {
        recovery_flags |= RECOVERY_FLAG_COOLDOWN_LOSS;
    }
    if input.validated_residual_historical_value_lamports
        < round.stored_residual_hwm_floor_lamports
    {
        recovery_flags |= RECOVERY_FLAG_RESIDUAL_HWM;
    }

    let mut next_leg = *leg;
    next_leg.status = WithdrawalLegStatus::Finalized;
    next_leg.recovery_flags = recovery_flags;
    next_leg.finalized_epoch = Some(input.finalized_epoch);
    next_leg.finalized_native_lamports = input.finalized_native_lamports;
    next_leg.recovered_stake_rent_lamports = input.recovered_stake_rent_lamports;
    next_leg.recovered_metadata_rent_lamports = input.recovered_metadata_rent_lamports;
    next_leg.cooldown_reward_lamports = input.cooldown_reward_lamports;
    next_leg.cooldown_loss_lamports = input.cooldown_loss_lamports;

    let mut next_round = *round;
    next_round.cumulative_finalized_delegated_native_lamports = checked_add(
        next_round.cumulative_finalized_delegated_native_lamports,
        leg.observed_delegated_native_lamports,
    )?;
    next_round.cumulative_finalized_native_lamports = checked_add(
        next_round.cumulative_finalized_native_lamports,
        input.finalized_native_lamports,
    )?;
    next_round.cumulative_recovered_stake_rent_lamports = checked_add(
        next_round.cumulative_recovered_stake_rent_lamports,
        input.recovered_stake_rent_lamports,
    )?;
    next_round.cumulative_recovered_metadata_rent_lamports = checked_add(
        next_round.cumulative_recovered_metadata_rent_lamports,
        input.recovered_metadata_rent_lamports,
    )?;
    next_round.cumulative_cooldown_rewards_lamports = checked_add(
        next_round.cumulative_cooldown_rewards_lamports,
        input.cooldown_reward_lamports,
    )?;
    next_round.cumulative_cooldown_losses_lamports = checked_add(
        next_round.cumulative_cooldown_losses_lamports,
        input.cooldown_loss_lamports,
    )?;
    next_round.finalized_leg_count = next_round
        .finalized_leg_count
        .checked_add(1)
        .ok_or(Piv1Error::ArithmeticOverflow)?;
    next_round.recorded_escrow_available_lamports = input.escrow_available_after_lamports;

    let initial_liquid_funding = checked_add(
        next_round.pending_sol_used_lamports,
        next_round.prior_next_cycle_yield_used_lamports()?,
    )?;
    let expected_escrow = checked_sub(
        checked_add(
            initial_liquid_funding,
            next_round.cumulative_finalized_native_lamports,
        )?,
        next_round.cumulative_recovered_stake_rent_lamports,
    )?;
    if expected_escrow != input.escrow_available_after_lamports {
        return Err(Piv1Error::EscrowReconciliationMismatch);
    }

    let outcome = if recovery_flags != 0 {
        next_round.lifecycle = DistributionLifecycle::RecoveryRequired;
        next_round.recovery_flags |= recovery_flags;
        LegFinalizationOutcome::RecoveryRequired
    } else if next_round.is_withdrawal_complete() {
        next_round.lifecycle = DistributionLifecycle::EscrowFunded;
        LegFinalizationOutcome::EscrowFunded
    } else {
        LegFinalizationOutcome::Recorded
    };

    next_leg.validate()?;
    next_round.validate()?;
    *leg = next_leg;
    *round = next_round;
    Ok(outcome)
}

/// Atomically records beneficiary, KIF, HWM, and cumulative settlement state.
pub fn settle_distribution(
    config: &mut PivConfig,
    round: &mut ActiveDistribution,
    rewards: &mut [GuardianReward; GUARDIAN_COUNT],
    input: SettlementInput,
) -> Piv1Result<SettlementOutcome> {
    config.validate_initialized()?;
    round.validate()?;
    if round.lifecycle == DistributionLifecycle::Settled || round.settlement_recorded {
        return Err(Piv1Error::SettlementReplay);
    }
    if round.lifecycle == DistributionLifecycle::RecoveryRequired {
        return Err(Piv1Error::RecoveryRequired);
    }
    if round.lifecycle != DistributionLifecycle::EscrowFunded {
        return Err(Piv1Error::InvalidLifecycle);
    }
    if input.sequence != round.active_sequence {
        return Err(Piv1Error::SequenceMismatch);
    }
    validate_active_sequence_binding(config, round)?;
    if input.escrow_available_lamports != round.recorded_escrow_available_lamports {
        return Err(Piv1Error::EscrowReconciliationMismatch);
    }
    if config.protected_principal_hwm_lamports
        != round.old_protected_principal_lamports
        || config.collective_kif_carry_lamports != round.kif_carry_input_lamports
    {
        return Err(Piv1Error::CumulativeReconciliationMismatch);
    }

    let eligible_native = checked_add(
        checked_add(
            round.pending_sol_used_lamports,
            round.prior_next_cycle_yield_used_lamports()?,
        )?,
        round.cumulative_finalized_delegated_native_lamports,
    )?;
    let actual_net_available = min(
        round.outgoing_gross_obligation_lamports,
        eligible_native,
    );
    let actual_htfp = input.actual_htfp_lamports;
    let actual_team = input.actual_team_owner_lamports;
    let actual_kif = input.actual_kif_allocation_lamports;
    if actual_htfp > round.htfp_gross_obligation_lamports
        || actual_team > round.team_owner_gross_obligation_lamports
        || actual_kif > round.kif_gross_obligation_lamports
    {
        return Err(Piv1Error::ObligationExceeded);
    }
    let actual_allocated = checked_add(checked_add(actual_htfp, actual_team)?, actual_kif)?;
    if actual_allocated > actual_net_available {
        return Err(Piv1Error::ObligationExceeded);
    }
    let net_allocation_dust = checked_sub(actual_net_available, actual_allocated)?;
    let retained_conservative_dust = checked_sub(eligible_native, actual_net_available)?;
    let escrow_remainder = checked_sub(
        round.recorded_escrow_available_lamports,
        actual_allocated,
    )?;
    let expected_escrow_remainder = checked_add(
        checked_add(net_allocation_dust, retained_conservative_dust)?,
        round.cumulative_cooldown_rewards_lamports,
    )?;
    if escrow_remainder != expected_escrow_remainder {
        return Err(Piv1Error::EscrowReconciliationMismatch);
    }

    let kif_allocation = piv1_math::allocate_kif(
        actual_kif,
        round.kif_carry_input_lamports,
        round.kif_active_guardian_count,
    )?;
    let (per_active_guardian, kif_liability, kif_carry_next, zero_active_compound) =
        match kif_allocation {
            piv1_math::KifAllocation::ActiveGuardians(active) => (
                active.per_guardian,
                active.credited_total,
                active.carry_next,
                0,
            ),
            piv1_math::KifAllocation::ZeroActiveGuardians(zero) => {
                (0, 0, zero.carry_next, zero.compound_from_kif)
            }
        };

    let mut next_rewards = *rewards;
    for (index, reward) in next_rewards.iter_mut().enumerate() {
        let guardian_index =
            u8::try_from(index).map_err(|_| Piv1Error::ArithmeticOverflow)?;
        let bit = 1_u8
            .checked_shl(u32::from(guardian_index))
            .ok_or(Piv1Error::ArithmeticOverflow)?;
        let credit = if round.kif_eligibility_bitmap & bit != 0 {
            per_active_guardian
        } else {
            0
        };
        reward.credit_snapshot(
            round.guardian_keys[index],
            guardian_index,
            round.guardian_registry_revision,
            credit,
        )?;
    }

    let actual_hwm_delta = checked_add(
        checked_add(
            checked_add(
                round.proposed_hwm_delta_lamports,
                net_allocation_dust,
            )?,
            retained_conservative_dust,
        )?,
        zero_active_compound,
    )?;
    let settled_hwm = checked_add(
        round.old_protected_principal_lamports,
        actual_hwm_delta,
    )?;

    if input.validated_post_settlement_protected_value_lamports < settled_hwm {
        let mut recovery_round = *round;
        recovery_round.lifecycle = DistributionLifecycle::RecoveryRequired;
        recovery_round.recovery_flags |= RECOVERY_FLAG_RESIDUAL_HWM;
        recovery_round.validate()?;
        *round = recovery_round;
        return Ok(SettlementOutcome::RecoveryRequired);
    }

    let mut next_round = *round;
    next_round.actual_net_available_lamports = actual_net_available;
    next_round.actual_htfp_lamports = actual_htfp;
    next_round.actual_team_owner_lamports = actual_team;
    next_round.actual_kif_allocation_lamports = actual_kif;
    next_round.actual_net_allocation_dust_lamports = net_allocation_dust;
    next_round.actual_allocated_outgoing_lamports = actual_allocated;
    next_round.actual_escrow_remainder_lamports = escrow_remainder;
    next_round.actual_retained_conservative_dust_lamports = retained_conservative_dust;
    next_round.actual_kif_liability_lamports = kif_liability;
    next_round.actual_kif_carry_next_lamports = kif_carry_next;
    next_round.actual_zero_active_kif_compound_lamports = zero_active_compound;
    next_round.actual_hwm_delta_lamports = actual_hwm_delta;
    next_round.settled_protected_hwm_lamports = settled_hwm;
    next_round.outstanding_active_round_liability_lamports = 0;
    next_round.settlement_recorded = true;
    next_round.lifecycle = DistributionLifecycle::Settled;

    let mut next_config = config.clone();
    if next_config.checked_increase_protected_principal_hwm(actual_hwm_delta)?
        != settled_hwm
    {
        return Err(Piv1Error::HighWaterMarkDecrease);
    }
    next_config.cumulative_gross_yield_lamports = checked_add(
        next_config.cumulative_gross_yield_lamports,
        round.gross_yield_lamports,
    )?;
    next_config.cumulative_htfp_paid_lamports = checked_add(
        next_config.cumulative_htfp_paid_lamports,
        actual_htfp,
    )?;
    next_config.cumulative_team_owner_paid_lamports = checked_add(
        next_config.cumulative_team_owner_paid_lamports,
        actual_team,
    )?;
    next_config.cumulative_kif_credited_lamports = checked_add(
        next_config.cumulative_kif_credited_lamports,
        kif_liability,
    )?;
    next_config.kif_claim_liability_lamports = checked_add(
        next_config.kif_claim_liability_lamports,
        kif_liability,
    )?;
    next_config.collective_kif_carry_lamports = kif_carry_next;
    next_config.cumulative_permanent_compound_lamports = checked_add(
        next_config.cumulative_permanent_compound_lamports,
        round.permanent_compound_lamports,
    )?;
    let retained_dust = checked_add(
        checked_add(
            checked_add(
                round.split_dust_lamports,
                round.snapshot_conversion_dust_lamports,
            )?,
            net_allocation_dust,
        )?,
        retained_conservative_dust,
    )?;
    next_config.cumulative_retained_dust_lamports = checked_add(
        next_config.cumulative_retained_dust_lamports,
        retained_dust,
    )?;
    next_config.cumulative_zero_active_kif_compound_lamports = checked_add(
        next_config.cumulative_zero_active_kif_compound_lamports,
        zero_active_compound,
    )?;
    next_config.cumulative_cooldown_yield_recorded_lamports = checked_add(
        next_config.cumulative_cooldown_yield_recorded_lamports,
        round.cumulative_cooldown_rewards_lamports,
    )?;
    next_config.next_cycle_yield_lamports = checked_add(
        next_config.next_cycle_yield_lamports,
        round.cumulative_cooldown_rewards_lamports,
    )?;

    next_round.validate()?;
    next_config.validate_initialized()?;
    *round = next_round;
    *config = next_config;
    *rewards = next_rewards;
    Ok(SettlementOutcome::Settled)
}

/// Integrates already validated pending contributions, records completion, and
/// returns the reusable active header to Idle without reusing its sequence.
pub fn integrate_pending_and_complete(
    config: &mut PivConfig,
    round: &mut ActiveDistribution,
    input: PendingIntegrationInput,
) -> Piv1Result<CompletedDistributionSummary> {
    config.validate_initialized()?;
    round.validate()?;
    if round.lifecycle == DistributionLifecycle::RecoveryRequired {
        return Err(Piv1Error::RecoveryRequired);
    }
    if round.lifecycle != DistributionLifecycle::Settled {
        return Err(Piv1Error::InvalidLifecycle);
    }
    if input.sequence != round.active_sequence {
        return Err(Piv1Error::SequenceMismatch);
    }
    validate_active_sequence_binding(config, round)?;
    if round.outstanding_active_round_liability_lamports != 0 {
        return Err(Piv1Error::OutstandingLiability);
    }
    if input.completed_at < round.prepared_at {
        return Err(Piv1Error::TimestampRegression);
    }
    if input.integrated_pending_sol_lamports != config.accounted_pending_sol_lamports
        || input.integrated_pending_jitosol_units
            != config.accounted_pending_jitosol_units
        || input.contribution_value_lamports < input.integrated_pending_sol_lamports
    {
        return Err(Piv1Error::CumulativeReconciliationMismatch);
    }
    if config.protected_principal_hwm_lamports != round.settled_protected_hwm_lamports {
        return Err(Piv1Error::CumulativeReconciliationMismatch);
    }
    let expected_hwm = checked_add(
        config.protected_principal_hwm_lamports,
        input.contribution_value_lamports,
    )?;
    if input.new_protected_hwm_lamports != expected_hwm
        || input.new_protected_hwm_lamports < config.protected_principal_hwm_lamports
    {
        return Err(Piv1Error::HighWaterMarkDecrease);
    }

    let mut next_config = config.clone();
    next_config.accounted_pending_sol_lamports = next_config
        .accounted_pending_sol_lamports
        .checked_sub(input.integrated_pending_sol_lamports)
        .ok_or(Piv1Error::CumulativeReconciliationMismatch)?;
    next_config.accounted_pending_jitosol_units = next_config
        .accounted_pending_jitosol_units
        .checked_sub(input.integrated_pending_jitosol_units)
        .ok_or(Piv1Error::CumulativeReconciliationMismatch)?;
    next_config.accounted_historical_jitosol_units =
        input.new_accounted_historical_jitosol_units;
    next_config.accounted_historical_sol_lamports =
        input.new_accounted_historical_sol_lamports;
    next_config.protected_principal_hwm_lamports = input.new_protected_hwm_lamports;
    next_config.cumulative_contribution_value_lamports = checked_add(
        next_config.cumulative_contribution_value_lamports,
        input.contribution_value_lamports,
    )?;

    let summary = CompletedDistributionSummary {
        sequence: round.active_sequence,
        completed_at: input.completed_at,
        gross_yield_lamports: round.gross_yield_lamports,
        actual_allocated_outgoing_lamports: round.actual_allocated_outgoing_lamports,
        integrated_contribution_value_lamports: input.contribution_value_lamports,
        final_protected_hwm_lamports: input.new_protected_hwm_lamports,
        fixed_jitosol_withdrawal_target_units: round.fixed_jitosol_withdrawal_target_units,
        successful_leg_count: round.successful_leg_count,
        cumulative_cooldown_rewards_lamports: round.cumulative_cooldown_rewards_lamports,
        actual_kif_liability_lamports: round.actual_kif_liability_lamports,
        actual_kif_carry_next_lamports: round.actual_kif_carry_next_lamports,
    };
    let next_round = ActiveDistribution::idle_after_completion(round.bump, summary);

    next_config.validate_initialized()?;
    next_round.validate()?;
    *config = next_config;
    *round = next_round;
    Ok(summary)
}

fn checked_add(left: u64, right: u64) -> Piv1Result<u64> {
    left.checked_add(right).ok_or(Piv1Error::ArithmeticOverflow)
}

fn validate_active_sequence_binding(
    config: &PivConfig,
    round: &ActiveDistribution,
) -> Piv1Result<()> {
    let expected_next = round
        .active_sequence
        .checked_add(1)
        .ok_or(Piv1Error::ArithmeticOverflow)?;
    if config.next_distribution_sequence != expected_next {
        return Err(Piv1Error::SequenceMismatch);
    }
    Ok(())
}

fn checked_sub(left: u64, right: u64) -> Piv1Result<u64> {
    left.checked_sub(right)
        .ok_or(Piv1Error::CumulativeReconciliationMismatch)
}
