use anchor_lang::prelude::Pubkey;

use piv1::{
    constants::{
        CONFIG_MIGRATION_RESERVE_BYTES, GUARDIAN_COUNT,
        INSUFFICIENT_RETRY_COOLDOWN_SECONDS, KIF_PERIOD_SECONDS,
        MINIMUM_DISTRIBUTION_INTERVAL_SECONDS, RECOVERY_FLAG_COOLDOWN_LOSS,
        RECOVERY_FLAG_RESIDUAL_HWM, STATE_LAYOUT_VERSION,
    },
    errors::Piv1Error,
    state::{
        finalize_withdrawal_leg, initiate_withdrawal_leg,
        integrate_pending_and_complete, open_distribution, settle_distribution,
        record_no_yield_evaluation, record_valid_insufficient_attempt,
        ActiveDistribution, DistributionFunding, DistributionLifecycle,
        GuardianRegistry, GuardianReward, LegFinalizationInput,
        LegFinalizationOutcome, LegInitiationInput, OpenDistributionInput,
        PendingIntegrationInput, PivConfig, PivConfigBumps, SettlementInput,
        SettlementOutcome, WithdrawalLeg,
    },
};

const PREPARED_AT: i64 = 1;
const INITIAL_HWM: u64 = 1_000;
const HISTORICAL_VALUE: u64 = 1_100;
const GROSS_YIELD: u64 = HISTORICAL_VALUE - INITIAL_HWM;
const OUTGOING_OBLIGATION: u64 = 80;
const WITHDRAWAL_TARGET: u64 = 100;
const LEG_FLOOR: u64 = 25;
const MAXIMUM_USEFUL_LEGS: u64 = 4;
const PROTECTED_VALUE_FLOOR: u64 = 2_000;

struct World {
    config: PivConfig,
    round: ActiveDistribution,
    registry: GuardianRegistry,
    rewards: [GuardianReward; GUARDIAN_COUNT],
}

fn key(tag: u8) -> Pubkey {
    Pubkey::new_from_array([tag; 32])
}

fn guardian_keys() -> [Pubkey; GUARDIAN_COUNT] {
    core::array::from_fn(|index| key(u8::try_from(index + 101).expect("guardian tag")))
}

fn config(pending_sol_lamports: u64, next_sequence: u64) -> PivConfig {
    PivConfig {
        version: STATE_LAYOUT_VERSION,
        is_initialized: true,
        paused: false,
        bumps: PivConfigBumps {
            config: 1,
            piv_authority: 2,
            active_distribution: 3,
            principal_jito_vault: 4,
            pending_jito_vault: 5,
            pending_sol_vault: 6,
            principal_sol_queue: 7,
            operational_sol_vault: 8,
            distribution_escrow: 9,
            kif_sol_vault: 10,
            guardian_registry: 11,
        },
        stake_pool_program: key(1),
        stake_pool: key(2),
        validator_list: key(3),
        reserve_stake: key(4),
        jitosol_mint: key(5),
        token_program: key(6),
        stake_program: key(7),
        system_program: key(8),
        manager_fee_account: key(9),
        referrer_token_account: key(10),
        piv_authority: key(11),
        active_distribution: key(12),
        principal_jito_vault: key(13),
        pending_jito_vault: key(14),
        pending_sol_vault: key(15),
        principal_sol_queue: key(16),
        operational_sol_vault: key(17),
        distribution_escrow: key(18),
        kif_sol_vault: key(19),
        htfp_recipient: key(20),
        team_owner_recipient: key(21),
        guardian_registry: key(22),
        basis_points_denominator: 10_000,
        htfp_reserve_bps: 5_900,
        permanent_compound_bps: 1_950,
        team_owner_pool_bps: 1_950,
        kif_bps: 200,
        configured_slippage_bps: 1,
        slippage_hard_cap_bps: 1,
        minimum_distribution_interval_seconds: MINIMUM_DISTRIBUTION_INTERVAL_SECONDS,
        insufficient_retry_cooldown_seconds: INSUFFICIENT_RETRY_COOLDOWN_SECONDS,
        last_successful_preparation_at: None,
        last_valid_insufficient_attempt_at: None,
        next_distribution_sequence: next_sequence,
        protected_principal_hwm_lamports: INITIAL_HWM,
        accounted_historical_jitosol_units: 1_000,
        accounted_historical_sol_lamports: 10,
        accounted_pending_jitosol_units: 0,
        accounted_pending_sol_lamports: pending_sol_lamports,
        next_cycle_yield_lamports: 0,
        kif_claim_liability_lamports: 0,
        collective_kif_carry_lamports: 0,
        cumulative_contribution_value_lamports: 0,
        cumulative_gross_yield_lamports: 0,
        cumulative_htfp_paid_lamports: 0,
        cumulative_team_owner_paid_lamports: 0,
        cumulative_kif_credited_lamports: 0,
        cumulative_kif_claimed_lamports: 0,
        cumulative_permanent_compound_lamports: 0,
        cumulative_retained_dust_lamports: 0,
        cumulative_zero_active_kif_compound_lamports: 0,
        cumulative_cooldown_yield_recorded_lamports: 0,
        kif_anchor_timestamp: 0,
        kif_period_seconds: KIF_PERIOD_SECONDS,
        guardian_registry_revision: 7,
        migration_reserve: [0; CONFIG_MIGRATION_RESERVE_BYTES],
    }
}

fn world(pending_sol_lamports: u64, next_sequence: u64) -> World {
    let config = config(pending_sol_lamports, next_sequence);
    config.validate_initialized().expect("valid config fixture");
    let registry = GuardianRegistry::new(12, 7, guardian_keys())
        .expect("valid registry fixture");
    let rewards = core::array::from_fn(|index| {
        let guardian_index = u8::try_from(index).expect("guardian index");
        GuardianReward::new(guardian_index, &registry, guardian_index)
            .expect("valid reward fixture")
    });
    World {
        config,
        round: ActiveDistribution::new_idle(13),
        registry,
        rewards,
    }
}

fn withdrawal_open_input(config: &PivConfig) -> OpenDistributionInput {
    OpenDistributionInput {
        sequence: config.next_distribution_sequence,
        prepared_at: PREPARED_AT,
        prepared_slot: 2,
        prepared_epoch: 3,
        historical_jitosol_units: config.accounted_historical_jitosol_units,
        historical_sol_lamports: config.accounted_historical_sol_lamports,
        historical_value_lamports: HISTORICAL_VALUE,
        snapshot_pool_total_lamports: 10_000,
        snapshot_pool_token_supply: 10_000,
        snapshot_withdrawal_fee_numerator: 1,
        snapshot_withdrawal_fee_denominator: 100,
        gross_yield_lamports: GROSS_YIELD,
        pending_sol_snapshot_lamports: config.accounted_pending_sol_lamports,
        pending_sol_used_lamports: config
            .accounted_pending_sol_lamports
            .min(OUTGOING_OBLIGATION),
        snapshot_conversion_dust_lamports: 0,
        stored_residual_hwm_floor_lamports: PROTECTED_VALUE_FLOOR,
        funding: DistributionFunding::Withdrawal {
            fixed_jitosol_target_units: WITHDRAWAL_TARGET,
            snapshot_leg_input_floor_units: LEG_FLOOR,
            maximum_useful_legs: MAXIMUM_USEFUL_LEGS,
            stored_round_minimum_native_lamports: OUTGOING_OBLIGATION,
            initial_escrow_available_lamports: config
                .accounted_pending_sol_lamports
                .min(OUTGOING_OBLIGATION),
        },
    }
}

fn liquid_open_input(config: &PivConfig) -> OpenDistributionInput {
    let mut input = withdrawal_open_input(config);
    input.funding = DistributionFunding::Liquid {
        escrow_available_lamports: OUTGOING_OBLIGATION,
    };
    input
}

fn opened_withdrawal(next_sequence: u64) -> World {
    let mut world = world(20, next_sequence);
    let input = withdrawal_open_input(&world.config);
    open_distribution(
        &mut world.config,
        &mut world.round,
        &world.registry,
        &world.rewards,
        input,
    )
    .expect("open withdrawal fixture");
    world
}

fn opened_liquid(next_sequence: u64) -> World {
    let mut world = world(OUTGOING_OBLIGATION, next_sequence);
    let input = liquid_open_input(&world.config);
    open_distribution(
        &mut world.config,
        &mut world.round,
        &world.registry,
        &world.rewards,
        input,
    )
    .expect("open liquid fixture");
    world
}

fn leg_input(
    round: &ActiveDistribution,
    leg_index: u64,
    maximum_safe_capacity_units: u64,
    jitosol_input_units: u64,
) -> LegInitiationInput {
    let tag_offset = u8::try_from(leg_index).unwrap_or(0);
    LegInitiationInput {
        sequence: round.active_sequence,
        leg_index,
        validator_list_index: u32::try_from(leg_index).expect("validator index"),
        validator_seed_suffix: u32::try_from(leg_index + 10).expect("validator seed"),
        validator_vote: key(200_u8.saturating_add(tag_offset)),
        validator_stake_source: key(220_u8.saturating_add(tag_offset)),
        initiation_epoch: 4,
        pool_total_lamports: 10_000,
        pool_token_supply: 10_000,
        withdrawal_fee_numerator: 1,
        withdrawal_fee_denominator: 100,
        current_technical_floor_units: LEG_FLOOR,
        maximum_safe_capacity_units,
        jitosol_input_units,
        withdrawal_fee_units: 0,
        burned_units: jitosol_input_units,
        expected_native_lamports: jitosol_input_units,
        observed_delegated_native_lamports: jitosol_input_units,
        minimum_native_lamports: jitosol_input_units,
        stake_rent_advanced_lamports: 10,
        metadata_rent_advanced_lamports: 5,
    }
}

fn finalization_input(
    round: &ActiveDistribution,
    leg: &WithdrawalLeg,
) -> LegFinalizationInput {
    let finalized_native_lamports = leg
        .observed_delegated_native_lamports
        .checked_add(leg.stake_rent_advanced_lamports)
        .expect("fixture finalization value");
    let cumulative_finalized = round
        .cumulative_finalized_native_lamports
        .checked_add(finalized_native_lamports)
        .expect("fixture cumulative finalization");
    let cumulative_recovered_stake_rent = round
        .cumulative_recovered_stake_rent_lamports
        .checked_add(leg.stake_rent_advanced_lamports)
        .expect("fixture cumulative rent");
    let escrow_available_after_lamports = round
        .pending_sol_used_lamports
        .checked_add(cumulative_finalized)
        .and_then(|value| value.checked_sub(cumulative_recovered_stake_rent))
        .expect("fixture escrow value");

    LegFinalizationInput {
        sequence: round.active_sequence,
        leg_index: leg.leg_index,
        finalized_epoch: 5,
        finalized_native_lamports,
        recovered_stake_rent_lamports: leg.stake_rent_advanced_lamports,
        recovered_metadata_rent_lamports: leg.metadata_rent_advanced_lamports,
        cooldown_reward_lamports: 0,
        cooldown_loss_lamports: 0,
        validated_residual_historical_value_lamports: PROTECTED_VALUE_FLOOR,
        escrow_available_after_lamports,
    }
}

fn settlement_input(round: &ActiveDistribution) -> SettlementInput {
    SettlementInput {
        sequence: round.active_sequence,
        escrow_available_lamports: round.recorded_escrow_available_lamports,
        actual_htfp_lamports: round.htfp_gross_obligation_lamports,
        actual_team_owner_lamports: round.team_owner_gross_obligation_lamports,
        actual_kif_allocation_lamports: round.kif_gross_obligation_lamports,
        validated_post_settlement_protected_value_lamports: PROTECTED_VALUE_FLOOR,
    }
}

fn settled_liquid(next_sequence: u64) -> World {
    let mut world = opened_liquid(next_sequence);
    let input = settlement_input(&world.round);
    assert_eq!(
        settle_distribution(
            &mut world.config,
            &mut world.round,
            &mut world.rewards,
            input,
        ),
        Ok(SettlementOutcome::Settled)
    );
    world
}

fn integration_input(world: &World) -> PendingIntegrationInput {
    let contribution_value_lamports = world.config.accounted_pending_sol_lamports;
    PendingIntegrationInput {
        sequence: world.round.active_sequence,
        completed_at: PREPARED_AT + 1,
        integrated_pending_sol_lamports: world.config.accounted_pending_sol_lamports,
        integrated_pending_jitosol_units: 0,
        contribution_value_lamports,
        new_accounted_historical_jitosol_units: world
            .config
            .accounted_historical_jitosol_units,
        new_accounted_historical_sol_lamports: world
            .config
            .accounted_historical_sol_lamports
            .checked_add(contribution_value_lamports)
            .expect("fixture historical SOL"),
        new_protected_hwm_lamports: world
            .config
            .protected_principal_hwm_lamports
            .checked_add(contribution_value_lamports)
            .expect("fixture HWM"),
    }
}

fn assert_open_rejected_unchanged(
    world: &mut World,
    input: OpenDistributionInput,
    expected: Piv1Error,
) {
    let config_before = world.config.clone();
    let round_before = world.round;
    assert_eq!(
        open_distribution(
            &mut world.config,
            &mut world.round,
            &world.registry,
            &world.rewards,
            input,
        ),
        Err(expected)
    );
    assert_eq!(world.config, config_before);
    assert_eq!(world.round, round_before);
}

fn assert_initiation_rejected_unchanged(
    config: &PivConfig,
    round: &mut ActiveDistribution,
    leg: &mut WithdrawalLeg,
    input: LegInitiationInput,
    expected: Piv1Error,
) {
    let round_before = *round;
    let leg_before = *leg;
    assert_eq!(
        initiate_withdrawal_leg(config, round, leg, input),
        Err(expected)
    );
    assert_eq!(*round, round_before);
    assert_eq!(*leg, leg_before);
}

fn assert_finalization_rejected_unchanged(
    config: &PivConfig,
    round: &mut ActiveDistribution,
    leg: &mut WithdrawalLeg,
    input: LegFinalizationInput,
    expected: Piv1Error,
) {
    let round_before = *round;
    let leg_before = *leg;
    assert_eq!(
        finalize_withdrawal_leg(config, round, leg, input),
        Err(expected)
    );
    assert_eq!(*round, round_before);
    assert_eq!(*leg, leg_before);
}

fn assert_settlement_rejected_unchanged(
    config: &mut PivConfig,
    round: &mut ActiveDistribution,
    rewards: &mut [GuardianReward; GUARDIAN_COUNT],
    input: SettlementInput,
    expected: Piv1Error,
) {
    let config_before = config.clone();
    let round_before = *round;
    let rewards_before = *rewards;
    assert_eq!(
        settle_distribution(config, round, rewards, input),
        Err(expected)
    );
    assert_eq!(*config, config_before);
    assert_eq!(*round, round_before);
    assert_eq!(*rewards, rewards_before);
}

fn assert_integration_rejected_unchanged(
    config: &mut PivConfig,
    round: &mut ActiveDistribution,
    input: PendingIntegrationInput,
    expected: Piv1Error,
) {
    let config_before = config.clone();
    let round_before = *round;
    assert_eq!(
        integrate_pending_and_complete(config, round, input),
        Err(expected)
    );
    assert_eq!(*config, config_before);
    assert_eq!(*round, round_before);
}

#[test]
fn opening_rejections_preserve_config_and_round() {
    let mut active = opened_withdrawal(1);
    let input = withdrawal_open_input(&active.config);
    assert_open_rejected_unchanged(&mut active, input, Piv1Error::InvalidLifecycle);

    let mut paused = world(20, 1);
    paused.config.paused = true;
    let input = withdrawal_open_input(&paused.config);
    assert_open_rejected_unchanged(&mut paused, input, Piv1Error::PausedOperation);

    let mut zero_target = world(20, 1);
    let mut input = withdrawal_open_input(&zero_target.config);
    input.funding = DistributionFunding::Withdrawal {
        fixed_jitosol_target_units: 0,
        snapshot_leg_input_floor_units: LEG_FLOOR,
        maximum_useful_legs: 0,
        stored_round_minimum_native_lamports: OUTGOING_OBLIGATION,
        initial_escrow_available_lamports: 20,
    };
    assert_open_rejected_unchanged(&mut zero_target, input, Piv1Error::ZeroTarget);

    let mut wrong_sequence = world(20, 1);
    let mut input = withdrawal_open_input(&wrong_sequence.config);
    input.sequence = 2;
    assert_open_rejected_unchanged(
        &mut wrong_sequence,
        input,
        Piv1Error::SequenceMismatch,
    );
}

#[test]
fn leg_input_and_binding_rejections_preserve_round_and_leg() {
    let mut world = opened_withdrawal(1);
    let mut leg = WithdrawalLeg::vacant(30, 31);
    let mut input = leg_input(&world.round, 0, 60, 60);
    input.sequence = 2;
    assert_initiation_rejected_unchanged(
        &world.config,
        &mut world.round,
        &mut leg,
        input,
        Piv1Error::SequenceMismatch,
    );

    let mut world = opened_withdrawal(1);
    let mut leg = WithdrawalLeg::vacant(30, 31);
    let input = leg_input(&world.round, 1, 60, 60);
    assert_initiation_rejected_unchanged(
        &world.config,
        &mut world.round,
        &mut leg,
        input,
        Piv1Error::LegIndexMismatch,
    );

    let mut world = opened_withdrawal(1);
    let mut leg = WithdrawalLeg::vacant(30, 31);
    let input = leg_input(&world.round, 0, 60, 59);
    assert_initiation_rejected_unchanged(
        &world.config,
        &mut world.round,
        &mut leg,
        input,
        Piv1Error::NonMaximumSafeLegFill,
    );

    let mut world = opened_withdrawal(1);
    let mut leg = WithdrawalLeg::vacant(30, 31);
    let input = leg_input(&world.round, 0, 20, 20);
    assert_initiation_rejected_unchanged(
        &world.config,
        &mut world.round,
        &mut leg,
        input,
        Piv1Error::TechnicalFloorNotMet,
    );

    let mut world = opened_withdrawal(1);
    let mut leg = WithdrawalLeg::vacant(30, 31);
    let input = leg_input(&world.round, 0, 101, 101);
    assert_initiation_rejected_unchanged(
        &world.config,
        &mut world.round,
        &mut leg,
        input,
        Piv1Error::TargetExceeded,
    );

    let mut world = opened_withdrawal(1);
    let mut leg = WithdrawalLeg::vacant(30, 31);
    let input = leg_input(&world.round, 0, 80, 80);
    assert_initiation_rejected_unchanged(
        &world.config,
        &mut world.round,
        &mut leg,
        input,
        Piv1Error::TechnicalFloorNotMet,
    );
}

#[test]
fn extra_too_many_and_replayed_leg_initiations_are_atomic() {
    let mut replay = opened_withdrawal(1);
    let mut replayed_leg = WithdrawalLeg::vacant(30, 31);
    let input = leg_input(&replay.round, 0, 60, 60);
    initiate_withdrawal_leg(
        &replay.config,
        &mut replay.round,
        &mut replayed_leg,
        input,
    )
    .expect("first initiation");
    assert_initiation_rejected_unchanged(
        &replay.config,
        &mut replay.round,
        &mut replayed_leg,
        input,
        Piv1Error::Replay,
    );

    let mut assigned = opened_withdrawal(1);
    let mut first_leg = WithdrawalLeg::vacant(30, 31);
    let input = leg_input(&assigned.round, 0, WITHDRAWAL_TARGET, WITHDRAWAL_TARGET);
    initiate_withdrawal_leg(
        &assigned.config,
        &mut assigned.round,
        &mut first_leg,
        input,
    )
    .expect("exact target assignment");
    let mut extra_leg = WithdrawalLeg::vacant(32, 33);
    let input = leg_input(&assigned.round, 1, 1, 1);
    assert_initiation_rejected_unchanged(
        &assigned.config,
        &mut assigned.round,
        &mut extra_leg,
        input,
        Piv1Error::TargetExceeded,
    );

    let mut bounded = opened_withdrawal(1);
    bounded.round.cumulative_jitosol_assigned_units = 60;
    bounded.round.cumulative_burned_units = 60;
    bounded.round.cumulative_expected_native_lamports = 60;
    bounded.round.cumulative_delegated_native_lamports = 60;
    bounded.round.next_leg_index = MAXIMUM_USEFUL_LEGS;
    bounded.round.successful_leg_count = MAXIMUM_USEFUL_LEGS;
    bounded.round.validate().expect("bounded defensive fixture");
    let mut excess_leg = WithdrawalLeg::vacant(34, 35);
    let input = leg_input(&bounded.round, MAXIMUM_USEFUL_LEGS, 40, 40);
    assert_initiation_rejected_unchanged(
        &bounded.config,
        &mut bounded.round,
        &mut excess_leg,
        input,
        Piv1Error::UsefulLegBoundExceeded,
    );
}

#[test]
fn finalization_rejections_preserve_round_and_leg() {
    let mut vacant = opened_withdrawal(0);
    let mut vacant_leg = WithdrawalLeg::vacant(30, 31);
    let input = LegFinalizationInput {
        sequence: 0,
        leg_index: 0,
        finalized_epoch: 5,
        finalized_native_lamports: 0,
        recovered_stake_rent_lamports: 0,
        recovered_metadata_rent_lamports: 0,
        cooldown_reward_lamports: 0,
        cooldown_loss_lamports: 0,
        validated_residual_historical_value_lamports: PROTECTED_VALUE_FLOOR,
        escrow_available_after_lamports: 20,
    };
    assert_finalization_rejected_unchanged(
        &vacant.config,
        &mut vacant.round,
        &mut vacant_leg,
        input,
        Piv1Error::InvalidLifecycle,
    );

    let mut bad_reconciliation = opened_withdrawal(1);
    let mut initiated_leg = WithdrawalLeg::vacant(30, 31);
    let initiation = leg_input(&bad_reconciliation.round, 0, 60, 60);
    initiate_withdrawal_leg(
        &bad_reconciliation.config,
        &mut bad_reconciliation.round,
        &mut initiated_leg,
        initiation,
    )
    .expect("initiate finalization fixture");
    let mut finalization = finalization_input(&bad_reconciliation.round, &initiated_leg);
    finalization.finalized_native_lamports = finalization
        .finalized_native_lamports
        .checked_add(1)
        .expect("test mismatch");
    assert_finalization_rejected_unchanged(
        &bad_reconciliation.config,
        &mut bad_reconciliation.round,
        &mut initiated_leg,
        finalization,
        Piv1Error::CumulativeReconciliationMismatch,
    );

    let mut replay = opened_withdrawal(1);
    let mut finalized_leg = WithdrawalLeg::vacant(30, 31);
    let initiation = leg_input(&replay.round, 0, 60, 60);
    initiate_withdrawal_leg(
        &replay.config,
        &mut replay.round,
        &mut finalized_leg,
        initiation,
    )
    .expect("initiate replay fixture");
    let finalization = finalization_input(&replay.round, &finalized_leg);
    assert_eq!(
        finalize_withdrawal_leg(
            &replay.config,
            &mut replay.round,
            &mut finalized_leg,
            finalization,
        ),
        Ok(LegFinalizationOutcome::Recorded)
    );
    assert_finalization_rejected_unchanged(
        &replay.config,
        &mut replay.round,
        &mut finalized_leg,
        finalization,
        Piv1Error::AlreadyFinalized,
    );
}

#[test]
fn settlement_rejections_preserve_every_mutable_account() {
    let mut premature = opened_withdrawal(1);
    let input = settlement_input(&premature.round);
    assert_settlement_rejected_unchanged(
        &mut premature.config,
        &mut premature.round,
        &mut premature.rewards,
        input,
        Piv1Error::InvalidLifecycle,
    );

    let mut count_mismatch = opened_withdrawal(1);
    count_mismatch.round.finalized_leg_count = 1;
    let input = settlement_input(&count_mismatch.round);
    assert_settlement_rejected_unchanged(
        &mut count_mismatch.config,
        &mut count_mismatch.round,
        &mut count_mismatch.rewards,
        input,
        Piv1Error::CountMismatch,
    );

    let mut escrow_mismatch = opened_liquid(1);
    let mut input = settlement_input(&escrow_mismatch.round);
    input.escrow_available_lamports = input
        .escrow_available_lamports
        .checked_sub(1)
        .expect("nonzero escrow");
    assert_settlement_rejected_unchanged(
        &mut escrow_mismatch.config,
        &mut escrow_mismatch.round,
        &mut escrow_mismatch.rewards,
        input,
        Piv1Error::EscrowReconciliationMismatch,
    );

    let mut replay = settled_liquid(1);
    let input = settlement_input(&replay.round);
    assert_settlement_rejected_unchanged(
        &mut replay.config,
        &mut replay.round,
        &mut replay.rewards,
        input,
        Piv1Error::SettlementReplay,
    );
}

#[test]
fn integration_rejections_preserve_config_and_round() {
    let mut premature = opened_liquid(1);
    let input = integration_input(&premature);
    assert_integration_rejected_unchanged(
        &mut premature.config,
        &mut premature.round,
        input,
        Piv1Error::InvalidLifecycle,
    );

    let mut outstanding = settled_liquid(1);
    outstanding.round.outstanding_active_round_liability_lamports = 1;
    let input = integration_input(&outstanding);
    assert_integration_rejected_unchanged(
        &mut outstanding.config,
        &mut outstanding.round,
        input,
        Piv1Error::OutstandingLiability,
    );

    let mut hwm_decrease = settled_liquid(1);
    let mut input = integration_input(&hwm_decrease);
    input.new_protected_hwm_lamports = hwm_decrease
        .config
        .protected_principal_hwm_lamports
        .checked_sub(1)
        .expect("positive HWM");
    assert_integration_rejected_unchanged(
        &mut hwm_decrease.config,
        &mut hwm_decrease.round,
        input,
        Piv1Error::HighWaterMarkDecrease,
    );
}

#[test]
fn recovery_required_blocks_all_normal_progress_without_mutation() {
    let mut recovery = opened_liquid(1);
    let mut input = settlement_input(&recovery.round);
    input.validated_post_settlement_protected_value_lamports = 0;
    let config_before = recovery.config.clone();
    let rewards_before = recovery.rewards;
    assert_eq!(
        settle_distribution(
            &mut recovery.config,
            &mut recovery.round,
            &mut recovery.rewards,
            input,
        ),
        Ok(SettlementOutcome::RecoveryRequired)
    );
    assert_eq!(recovery.config, config_before);
    assert_eq!(recovery.rewards, rewards_before);
    assert_eq!(
        recovery.round.lifecycle,
        DistributionLifecycle::RecoveryRequired
    );

    let mut vacant_leg = WithdrawalLeg::vacant(30, 31);
    let initiation = leg_input(&recovery.round, 0, 1, 1);
    assert_initiation_rejected_unchanged(
        &recovery.config,
        &mut recovery.round,
        &mut vacant_leg,
        initiation,
        Piv1Error::RecoveryRequired,
    );

    let finalization = LegFinalizationInput {
        sequence: recovery.round.active_sequence,
        leg_index: 0,
        finalized_epoch: 5,
        finalized_native_lamports: 0,
        recovered_stake_rent_lamports: 0,
        recovered_metadata_rent_lamports: 0,
        cooldown_reward_lamports: 0,
        cooldown_loss_lamports: 0,
        validated_residual_historical_value_lamports: 0,
        escrow_available_after_lamports: recovery.round.recorded_escrow_available_lamports,
    };
    assert_finalization_rejected_unchanged(
        &recovery.config,
        &mut recovery.round,
        &mut vacant_leg,
        finalization,
        Piv1Error::RecoveryRequired,
    );

    let settlement = settlement_input(&recovery.round);
    assert_settlement_rejected_unchanged(
        &mut recovery.config,
        &mut recovery.round,
        &mut recovery.rewards,
        settlement,
        Piv1Error::RecoveryRequired,
    );

    let integration = integration_input(&recovery);
    assert_integration_rejected_unchanged(
        &mut recovery.config,
        &mut recovery.round,
        integration,
        Piv1Error::RecoveryRequired,
    );
}

#[test]
fn arithmetic_overflow_is_rejected_after_staging_without_mutation() {
    let mut sequence_overflow = world(20, u64::MAX);
    let input = withdrawal_open_input(&sequence_overflow.config);
    assert_open_rejected_unchanged(
        &mut sequence_overflow,
        input,
        Piv1Error::ArithmeticOverflow,
    );

    let mut settlement_overflow = world(OUTGOING_OBLIGATION, 1);
    settlement_overflow.config.cumulative_gross_yield_lamports = u64::MAX;
    let input = liquid_open_input(&settlement_overflow.config);
    open_distribution(
        &mut settlement_overflow.config,
        &mut settlement_overflow.round,
        &settlement_overflow.registry,
        &settlement_overflow.rewards,
        input,
    )
    .expect("open late-overflow fixture");
    let input = settlement_input(&settlement_overflow.round);
    assert_settlement_rejected_unchanged(
        &mut settlement_overflow.config,
        &mut settlement_overflow.round,
        &mut settlement_overflow.rewards,
        input,
        Piv1Error::ArithmeticOverflow,
    );
}

fn assert_insufficient_rejected_unchanged(
    world: &mut World,
    attempted_at: i64,
    expected: Piv1Error,
) {
    let config_before = world.config.clone();
    let round_before = world.round;
    let registry_before = world.registry;
    let rewards_before = world.rewards;
    assert_eq!(
        record_valid_insufficient_attempt(
            &mut world.config,
            &world.round,
            attempted_at,
        ),
        Err(expected)
    );
    assert_eq!(world.config, config_before);
    assert_eq!(world.round, round_before);
    assert_eq!(world.registry, registry_before);
    assert_eq!(world.rewards, rewards_before);
}

#[test]
fn no_yield_rejects_each_positive_yield_source_without_mutation() {
    let historical_yield = world(0, 1);
    let config_before = historical_yield.config.clone();
    let round_before = historical_yield.round;
    assert_eq!(
        record_no_yield_evaluation(
            &historical_yield.config,
            &historical_yield.round,
            PREPARED_AT,
            HISTORICAL_VALUE,
        ),
        Err(Piv1Error::CumulativeReconciliationMismatch)
    );
    assert_eq!(historical_yield.config, config_before);
    assert_eq!(historical_yield.round, round_before);

    let mut prior_cycle_yield = world(0, 1);
    prior_cycle_yield.config.next_cycle_yield_lamports = 1;
    let config_before = prior_cycle_yield.config.clone();
    let round_before = prior_cycle_yield.round;
    assert_eq!(
        record_no_yield_evaluation(
            &prior_cycle_yield.config,
            &prior_cycle_yield.round,
            PREPARED_AT,
            INITIAL_HWM,
        ),
        Err(Piv1Error::CumulativeReconciliationMismatch)
    );
    assert_eq!(prior_cycle_yield.config, config_before);
    assert_eq!(prior_cycle_yield.round, round_before);
}

#[test]
fn pause_blocks_evaluation_insufficient_initiation_and_finalization_atomically() {
    let mut no_yield = world(0, 1);
    no_yield.config.paused = true;
    let config_before = no_yield.config.clone();
    let round_before = no_yield.round;
    assert_eq!(
        record_no_yield_evaluation(
            &no_yield.config,
            &no_yield.round,
            PREPARED_AT,
            INITIAL_HWM,
        ),
        Err(Piv1Error::PausedOperation)
    );
    assert_eq!(no_yield.config, config_before);
    assert_eq!(no_yield.round, round_before);

    let mut insufficient = world(0, 1);
    insufficient.config.paused = true;
    assert_insufficient_rejected_unchanged(
        &mut insufficient,
        PREPARED_AT,
        Piv1Error::PausedOperation,
    );

    let mut initiation = opened_withdrawal(1);
    initiation.config.paused = true;
    let mut vacant_leg = WithdrawalLeg::vacant(30, 31);
    let input = leg_input(&initiation.round, 0, 60, 60);
    assert_initiation_rejected_unchanged(
        &initiation.config,
        &mut initiation.round,
        &mut vacant_leg,
        input,
        Piv1Error::PausedOperation,
    );

    let mut finalization = opened_withdrawal(1);
    let mut initiated_leg = WithdrawalLeg::vacant(30, 31);
    let initiation_input = leg_input(&finalization.round, 0, 60, 60);
    initiate_withdrawal_leg(
        &finalization.config,
        &mut finalization.round,
        &mut initiated_leg,
        initiation_input,
    )
    .expect("initiate paused-finalization fixture");
    finalization.config.paused = true;
    let input = finalization_input(&finalization.round, &initiated_leg);
    assert_finalization_rejected_unchanged(
        &finalization.config,
        &mut finalization.round,
        &mut initiated_leg,
        input,
        Piv1Error::PausedOperation,
    );
}

#[test]
fn valid_insufficient_rejections_preserve_the_complete_world() {
    let mut cooldown = world(0, 1);
    cooldown.config.last_valid_insufficient_attempt_at = Some(PREPARED_AT);
    assert_insufficient_rejected_unchanged(
        &mut cooldown,
        PREPARED_AT + INSUFFICIENT_RETRY_COOLDOWN_SECONDS - 1,
        Piv1Error::InsufficientAttemptCooldownActive,
    );

    let mut preparation_timing = world(0, 1);
    preparation_timing.config.last_successful_preparation_at = Some(PREPARED_AT);
    assert_insufficient_rejected_unchanged(
        &mut preparation_timing,
        PREPARED_AT + MINIMUM_DISTRIBUTION_INTERVAL_SECONDS - 1,
        Piv1Error::PreparationIntervalNotElapsed,
    );

    let mut regression = world(0, 1);
    regression.config.last_successful_preparation_at = Some(PREPARED_AT + 1);
    assert_insufficient_rejected_unchanged(
        &mut regression,
        PREPARED_AT,
        Piv1Error::TimestampRegression,
    );

    let mut non_idle = opened_withdrawal(1);
    assert_insufficient_rejected_unchanged(
        &mut non_idle,
        PREPARED_AT + MINIMUM_DISTRIBUTION_INTERVAL_SECONDS,
        Piv1Error::InvalidLifecycle,
    );
}

#[test]
fn zero_leg_input_is_rejected_without_consuming_the_leg_index() {
    let mut world = opened_withdrawal(1);
    let mut leg = WithdrawalLeg::vacant(30, 31);
    let input = leg_input(&world.round, 0, 0, 0);
    assert_initiation_rejected_unchanged(
        &world.config,
        &mut world.round,
        &mut leg,
        input,
        Piv1Error::ZeroInput,
    );
    assert_eq!(world.round.next_leg_index, 0);
    assert_eq!(world.round.successful_leg_count, 0);
}

#[test]
fn finalization_binding_and_epoch_rejections_are_atomic() {
    let mut wrong_sequence = opened_withdrawal(1);
    let mut sequence_leg = WithdrawalLeg::vacant(30, 31);
    let initiation = leg_input(&wrong_sequence.round, 0, 60, 60);
    initiate_withdrawal_leg(
        &wrong_sequence.config,
        &mut wrong_sequence.round,
        &mut sequence_leg,
        initiation,
    )
    .expect("initiate wrong-sequence fixture");
    let mut input = finalization_input(&wrong_sequence.round, &sequence_leg);
    input.sequence = input.sequence.checked_add(1).expect("test sequence");
    assert_finalization_rejected_unchanged(
        &wrong_sequence.config,
        &mut wrong_sequence.round,
        &mut sequence_leg,
        input,
        Piv1Error::SequenceMismatch,
    );

    let mut wrong_index = opened_withdrawal(1);
    let mut index_leg = WithdrawalLeg::vacant(30, 31);
    let initiation = leg_input(&wrong_index.round, 0, 60, 60);
    initiate_withdrawal_leg(
        &wrong_index.config,
        &mut wrong_index.round,
        &mut index_leg,
        initiation,
    )
    .expect("initiate wrong-index fixture");
    let mut input = finalization_input(&wrong_index.round, &index_leg);
    input.leg_index = 1;
    assert_finalization_rejected_unchanged(
        &wrong_index.config,
        &mut wrong_index.round,
        &mut index_leg,
        input,
        Piv1Error::LegIndexMismatch,
    );

    let mut fabricated = opened_withdrawal(1);
    let mut fabricated_leg = WithdrawalLeg::vacant(30, 31);
    let initiation = leg_input(&fabricated.round, 0, 60, 60);
    initiate_withdrawal_leg(
        &fabricated.config,
        &mut fabricated.round,
        &mut fabricated_leg,
        initiation,
    )
    .expect("initiate fabricated-index fixture");
    fabricated_leg.leg_index = fabricated.round.next_leg_index;
    let input = finalization_input(&fabricated.round, &fabricated_leg);
    assert_finalization_rejected_unchanged(
        &fabricated.config,
        &mut fabricated.round,
        &mut fabricated_leg,
        input,
        Piv1Error::LegIndexMismatch,
    );

    let mut epoch_regression = opened_withdrawal(1);
    let mut epoch_leg = WithdrawalLeg::vacant(30, 31);
    let initiation = leg_input(&epoch_regression.round, 0, 60, 60);
    initiate_withdrawal_leg(
        &epoch_regression.config,
        &mut epoch_regression.round,
        &mut epoch_leg,
        initiation,
    )
    .expect("initiate epoch-regression fixture");
    let mut input = finalization_input(&epoch_regression.round, &epoch_leg);
    input.finalized_epoch = epoch_leg.initiation_epoch - 1;
    assert_finalization_rejected_unchanged(
        &epoch_regression.config,
        &mut epoch_regression.round,
        &mut epoch_leg,
        input,
        Piv1Error::TimestampRegression,
    );
}

#[test]
fn cooldown_loss_and_residual_hwm_finalizations_commit_recovery() {
    let mut cooldown_loss = opened_withdrawal(1);
    let mut loss_leg = WithdrawalLeg::vacant(30, 31);
    let initiation = leg_input(&cooldown_loss.round, 0, 60, 60);
    initiate_withdrawal_leg(
        &cooldown_loss.config,
        &mut cooldown_loss.round,
        &mut loss_leg,
        initiation,
    )
    .expect("initiate cooldown-loss fixture");
    let mut input = finalization_input(&cooldown_loss.round, &loss_leg);
    input.finalized_native_lamports -= 1;
    input.cooldown_loss_lamports = 1;
    input.escrow_available_after_lamports -= 1;
    assert_eq!(
        finalize_withdrawal_leg(
            &cooldown_loss.config,
            &mut cooldown_loss.round,
            &mut loss_leg,
            input,
        ),
        Ok(LegFinalizationOutcome::RecoveryRequired)
    );
    assert_eq!(
        cooldown_loss.round.lifecycle,
        DistributionLifecycle::RecoveryRequired
    );
    assert_eq!(
        cooldown_loss.round.recovery_flags,
        RECOVERY_FLAG_COOLDOWN_LOSS
    );
    assert_eq!(loss_leg.recovery_flags, RECOVERY_FLAG_COOLDOWN_LOSS);
    assert_eq!(cooldown_loss.round.cumulative_cooldown_losses_lamports, 1);
    assert_eq!(loss_leg.cooldown_loss_lamports, 1);

    let mut residual_hwm = opened_withdrawal(1);
    let mut residual_leg = WithdrawalLeg::vacant(32, 33);
    let initiation = leg_input(&residual_hwm.round, 0, 60, 60);
    initiate_withdrawal_leg(
        &residual_hwm.config,
        &mut residual_hwm.round,
        &mut residual_leg,
        initiation,
    )
    .expect("initiate residual-HWM fixture");
    let mut input = finalization_input(&residual_hwm.round, &residual_leg);
    input.validated_residual_historical_value_lamports =
        PROTECTED_VALUE_FLOOR - 1;
    assert_eq!(
        finalize_withdrawal_leg(
            &residual_hwm.config,
            &mut residual_hwm.round,
            &mut residual_leg,
            input,
        ),
        Ok(LegFinalizationOutcome::RecoveryRequired)
    );
    assert_eq!(
        residual_hwm.round.lifecycle,
        DistributionLifecycle::RecoveryRequired
    );
    assert_eq!(
        residual_hwm.round.recovery_flags,
        RECOVERY_FLAG_RESIDUAL_HWM
    );
    assert_eq!(residual_leg.recovery_flags, RECOVERY_FLAG_RESIDUAL_HWM);
}

#[test]
fn sequence_and_unfinalized_target_settlement_rejections_are_atomic() {
    let mut wrong_sequence = opened_liquid(1);
    let mut input = settlement_input(&wrong_sequence.round);
    input.sequence = input.sequence.checked_add(1).expect("test sequence");
    assert_settlement_rejected_unchanged(
        &mut wrong_sequence.config,
        &mut wrong_sequence.round,
        &mut wrong_sequence.rewards,
        input,
        Piv1Error::SequenceMismatch,
    );

    let mut awaiting_finalization = opened_withdrawal(1);
    let mut leg = WithdrawalLeg::vacant(30, 31);
    let initiation = leg_input(
        &awaiting_finalization.round,
        0,
        WITHDRAWAL_TARGET,
        WITHDRAWAL_TARGET,
    );
    initiate_withdrawal_leg(
        &awaiting_finalization.config,
        &mut awaiting_finalization.round,
        &mut leg,
        initiation,
    )
    .expect("assign full target without finalizing");
    assert!(awaiting_finalization.round.is_withdrawal_target_assigned());
    assert!(awaiting_finalization.round.is_awaiting_leg_inactivity());
    let input = settlement_input(&awaiting_finalization.round);
    assert_settlement_rejected_unchanged(
        &mut awaiting_finalization.config,
        &mut awaiting_finalization.round,
        &mut awaiting_finalization.rewards,
        input,
        Piv1Error::InvalidLifecycle,
    );

    let mut obligation_overrun = opened_liquid(1);
    let mut input = settlement_input(&obligation_overrun.round);
    input.actual_htfp_lamports = input
        .actual_htfp_lamports
        .checked_add(1)
        .expect("test obligation overrun");
    assert_settlement_rejected_unchanged(
        &mut obligation_overrun.config,
        &mut obligation_overrun.round,
        &mut obligation_overrun.rewards,
        input,
        Piv1Error::ObligationExceeded,
    );
}

#[test]
fn integration_sequence_and_non_idle_cancellation_attempts_are_atomic() {
    let mut wrong_sequence = settled_liquid(1);
    let mut input = integration_input(&wrong_sequence);
    input.sequence = input.sequence.checked_add(1).expect("test sequence");
    assert_integration_rejected_unchanged(
        &mut wrong_sequence.config,
        &mut wrong_sequence.round,
        input,
        Piv1Error::SequenceMismatch,
    );

    let mut cancellation = opened_withdrawal(1);
    let input = PendingIntegrationInput {
        sequence: cancellation.round.active_sequence,
        completed_at: PREPARED_AT + 1,
        integrated_pending_sol_lamports: 0,
        integrated_pending_jitosol_units: 0,
        contribution_value_lamports: 0,
        new_accounted_historical_jitosol_units: cancellation
            .config
            .accounted_historical_jitosol_units,
        new_accounted_historical_sol_lamports: cancellation
            .config
            .accounted_historical_sol_lamports,
        new_protected_hwm_lamports: cancellation
            .config
            .protected_principal_hwm_lamports,
    };
    assert_integration_rejected_unchanged(
        &mut cancellation.config,
        &mut cancellation.round,
        input,
        Piv1Error::InvalidLifecycle,
    );
    assert_ne!(cancellation.round.lifecycle, DistributionLifecycle::Idle);
}

#[test]
fn completed_sequence_cannot_be_reopened() {
    let mut completed = settled_liquid(1);
    let integration = integration_input(&completed);
    let summary = integrate_pending_and_complete(
        &mut completed.config,
        &mut completed.round,
        integration,
    )
    .expect("complete replay fixture");
    assert_eq!(completed.round.lifecycle, DistributionLifecycle::Idle);
    assert_eq!(summary.sequence, 1);

    completed.config.next_distribution_sequence = summary.sequence;
    let mut input = withdrawal_open_input(&completed.config);
    input.prepared_at = PREPARED_AT + MINIMUM_DISTRIBUTION_INTERVAL_SECONDS;
    assert_open_rejected_unchanged(&mut completed, input, Piv1Error::Replay);
}

#[test]
fn malformed_active_snapshot_invariants_block_settlement_atomically() {
    let mut split = opened_liquid(1);
    split.round.htfp_gross_obligation_lamports += 1;
    let input = settlement_input(&split.round);
    assert_settlement_rejected_unchanged(
        &mut split.config,
        &mut split.round,
        &mut split.rewards,
        input,
        Piv1Error::InvalidSplit,
    );

    let mut slippage = opened_liquid(1);
    slippage.round.stored_slippage_bps = 2;
    let input = settlement_input(&slippage.round);
    assert_settlement_rejected_unchanged(
        &mut slippage.config,
        &mut slippage.round,
        &mut slippage.rewards,
        input,
        Piv1Error::InvalidSlippage,
    );

    let mut kif_bitmap = opened_liquid(1);
    kif_bitmap.round.kif_eligibility_bitmap = 1;
    kif_bitmap.round.kif_active_guardian_count = 0;
    let input = settlement_input(&kif_bitmap.round);
    assert_settlement_rejected_unchanged(
        &mut kif_bitmap.config,
        &mut kif_bitmap.round,
        &mut kif_bitmap.rewards,
        input,
        Piv1Error::InvalidGuardianBitmap,
    );
}

#[test]
fn malformed_settlement_kif_and_hwm_invariants_block_integration_atomically() {
    let mut obligation = settled_liquid(1);
    obligation.round.actual_htfp_lamports = obligation
        .round
        .htfp_gross_obligation_lamports
        .checked_add(1)
        .expect("stored obligation overrun");
    let input = integration_input(&obligation);
    assert_integration_rejected_unchanged(
        &mut obligation.config,
        &mut obligation.round,
        input,
        Piv1Error::ObligationExceeded,
    );

    let mut kif = settled_liquid(1);
    kif.round.actual_kif_carry_next_lamports = kif
        .round
        .actual_kif_carry_next_lamports
        .checked_add(1)
        .expect("stored KIF mismatch");
    let input = integration_input(&kif);
    assert_integration_rejected_unchanged(
        &mut kif.config,
        &mut kif.round,
        input,
        Piv1Error::CumulativeReconciliationMismatch,
    );

    let mut hwm = settled_liquid(1);
    hwm.round.settled_protected_hwm_lamports = hwm
        .round
        .settled_protected_hwm_lamports
        .checked_add(1)
        .expect("stored HWM mismatch");
    let input = integration_input(&hwm);
    assert_integration_rejected_unchanged(
        &mut hwm.config,
        &mut hwm.round,
        input,
        Piv1Error::HighWaterMarkDecrease,
    );
}
