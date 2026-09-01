use anchor_lang::prelude::Pubkey;

use piv1::{
    constants::{
        CONFIG_MIGRATION_RESERVE_BYTES, GUARDIAN_COUNT,
        INSUFFICIENT_RETRY_COOLDOWN_SECONDS, KIF_PERIOD_SECONDS,
        MAX_CONFIGURED_SLIPPAGE_BPS, MINIMUM_DISTRIBUTION_INTERVAL_SECONDS,
        STATE_LAYOUT_VERSION,
    },
    state::{
        finalize_withdrawal_leg, initiate_withdrawal_leg,
        integrate_pending_and_complete, open_distribution,
        record_no_yield_evaluation, record_valid_insufficient_attempt,
        settle_distribution, ActiveDistribution, CompletedDistributionSummary,
        DistributionFunding, DistributionLifecycle, GuardianRegistry,
        GuardianReward, LegFinalizationInput, LegFinalizationOutcome,
        LegInitiationInput, OpenDistributionInput, PendingIntegrationInput,
        PivConfig, PivConfigBumps, SettlementInput, SettlementOutcome,
        WithdrawalLeg,
    },
};

const PREPARED_AT: i64 = 900_000;
const OLD_HWM: u64 = 1_000_000;
const GROSS_YIELD: u64 = 10_000;
const OUTGOING_GROSS: u64 = 8_050;
const PROPOSED_HWM: u64 = 1_001_950;

struct Fixture {
    config: PivConfig,
    registry: GuardianRegistry,
    rewards: [GuardianReward; GUARDIAN_COUNT],
    round: ActiveDistribution,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct FixedSnapshot {
    active_sequence: u64,
    last_completed: Option<CompletedDistributionSummary>,
    prepared_at: i64,
    prepared_slot: u64,
    prepared_epoch: u64,
    old_protected_principal_lamports: u64,
    historical_jitosol_units: u64,
    historical_sol_lamports: u64,
    historical_value_lamports: u64,
    snapshot_pool_total_lamports: u64,
    snapshot_pool_token_supply: u64,
    snapshot_withdrawal_fee_numerator: u64,
    snapshot_withdrawal_fee_denominator: u64,
    gross_yield_lamports: u64,
    prior_next_cycle_yield_lamports: u64,
    htfp_gross_obligation_lamports: u64,
    permanent_compound_lamports: u64,
    team_owner_gross_obligation_lamports: u64,
    kif_gross_obligation_lamports: u64,
    split_dust_lamports: u64,
    outgoing_gross_obligation_lamports: u64,
    pending_sol_snapshot_lamports: u64,
    pending_sol_used_lamports: u64,
    snapshot_conversion_dust_lamports: u64,
    fixed_jitosol_withdrawal_target_units: u64,
    snapshot_leg_input_floor_units: u64,
    maximum_useful_legs: u64,
    stored_round_minimum_native_lamports: u64,
    stored_residual_hwm_floor_lamports: u64,
    stored_slippage_bps: u16,
    htfp_recipient: Pubkey,
    team_owner_recipient: Pubkey,
    guardian_registry: Pubkey,
    guardian_registry_revision: u64,
    guardian_keys: [Pubkey; GUARDIAN_COUNT],
    kif_eligibility_bitmap: u8,
    kif_active_guardian_count: u8,
    kif_period_id: u64,
    kif_carry_input_lamports: u64,
    proposed_hwm_delta_lamports: u64,
    proposed_hwm_after_settlement_lamports: u64,
}

fn key(tag: u8) -> Pubkey {
    Pubkey::new_from_array([tag; 32])
}

fn valid_config(pending_sol_lamports: u64) -> PivConfig {
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
        configured_slippage_bps: MAX_CONFIGURED_SLIPPAGE_BPS,
        slippage_hard_cap_bps: MAX_CONFIGURED_SLIPPAGE_BPS,
        minimum_distribution_interval_seconds: MINIMUM_DISTRIBUTION_INTERVAL_SECONDS,
        insufficient_retry_cooldown_seconds: INSUFFICIENT_RETRY_COOLDOWN_SECONDS,
        last_successful_preparation_at: None,
        last_valid_insufficient_attempt_at: None,
        next_distribution_sequence: 0,
        protected_principal_hwm_lamports: OLD_HWM,
        accounted_historical_jitosol_units: 1_000,
        accounted_historical_sol_lamports: 0,
        accounted_pending_jitosol_units: 50,
        accounted_pending_sol_lamports: pending_sol_lamports,
        next_cycle_yield_lamports: 0,
        kif_claim_liability_lamports: 0,
        collective_kif_carry_lamports: 1,
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

fn fixture(pending_sol_lamports: u64, active_indices: &[usize]) -> Fixture {
    let guardian_keys = core::array::from_fn(|index| key(100 + index as u8));
    let registry = GuardianRegistry::new(90, 7, guardian_keys)
        .expect("valid fixed guardian registry");
    let mut rewards = core::array::from_fn(|index| {
        GuardianReward::new(index as u8, &registry, index as u8)
            .expect("valid guardian reward")
    });
    for index in active_indices {
        rewards[*index]
            .record_activity(&registry, *index as u8, 0)
            .expect("record deterministic activity");
    }

    let config = valid_config(pending_sol_lamports);
    config.validate_initialized().expect("valid config");
    Fixture {
        config,
        registry,
        rewards,
        round: ActiveDistribution::new_idle(80),
    }
}

fn liquid_open_input(config: &PivConfig, prepared_at: i64) -> OpenDistributionInput {
    OpenDistributionInput {
        sequence: config.next_distribution_sequence,
        prepared_at,
        prepared_slot: 12_345,
        prepared_epoch: 42,
        historical_jitosol_units: config.accounted_historical_jitosol_units,
        historical_sol_lamports: config.accounted_historical_sol_lamports,
        historical_value_lamports: config.protected_principal_hwm_lamports + GROSS_YIELD,
        snapshot_pool_total_lamports: 10_000_000,
        snapshot_pool_token_supply: 9_000_000,
        snapshot_withdrawal_fee_numerator: 1,
        snapshot_withdrawal_fee_denominator: 1_000,
        gross_yield_lamports: GROSS_YIELD,
        pending_sol_snapshot_lamports: config.accounted_pending_sol_lamports,
        pending_sol_used_lamports: OUTGOING_GROSS,
        snapshot_conversion_dust_lamports: 0,
        stored_residual_hwm_floor_lamports:
            config.protected_principal_hwm_lamports + 10_000,
        funding: DistributionFunding::Liquid {
            escrow_available_lamports: OUTGOING_GROSS,
        },
    }
}

fn withdrawal_open_input(
    config: &PivConfig,
    target_units: u64,
    stored_round_minimum_native_lamports: u64,
) -> OpenDistributionInput {
    let floor_units = 100;
    OpenDistributionInput {
        sequence: config.next_distribution_sequence,
        prepared_at: PREPARED_AT,
        prepared_slot: 12_345,
        prepared_epoch: 42,
        historical_jitosol_units: config.accounted_historical_jitosol_units,
        historical_sol_lamports: config.accounted_historical_sol_lamports,
        historical_value_lamports: config.protected_principal_hwm_lamports + GROSS_YIELD,
        snapshot_pool_total_lamports: 10_000_000,
        snapshot_pool_token_supply: 9_000_000,
        snapshot_withdrawal_fee_numerator: 1,
        snapshot_withdrawal_fee_denominator: 1_000,
        gross_yield_lamports: GROSS_YIELD,
        pending_sol_snapshot_lamports: config.accounted_pending_sol_lamports,
        pending_sol_used_lamports: config.accounted_pending_sol_lamports,
        snapshot_conversion_dust_lamports: 0,
        stored_residual_hwm_floor_lamports: PROPOSED_HWM,
        funding: DistributionFunding::Withdrawal {
            fixed_jitosol_target_units: target_units,
            snapshot_leg_input_floor_units: floor_units,
            maximum_useful_legs: target_units / floor_units,
            stored_round_minimum_native_lamports,
            initial_escrow_available_lamports: config.accounted_pending_sol_lamports,
        },
    }
}

fn initiate_leg(
    config: &PivConfig,
    round: &mut ActiveDistribution,
    leg: &mut WithdrawalLeg,
    maximum_safe_capacity_units: u64,
    delegated_native_lamports: u64,
) {
    let leg_index = round.next_leg_index;
    let input_units = round
        .remaining_withdrawal_target_units()
        .expect("valid remaining target")
        .min(maximum_safe_capacity_units);
    let fee_units = 1;
    let input = LegInitiationInput {
        sequence: round.active_sequence,
        leg_index,
        validator_list_index: leg_index as u32,
        validator_seed_suffix: 1_000 + leg_index as u32,
        validator_vote: key(150 + (leg_index as u8 * 2)),
        validator_stake_source: key(151 + (leg_index as u8 * 2)),
        initiation_epoch: 43,
        pool_total_lamports: 10_000_000,
        pool_token_supply: 9_000_000,
        withdrawal_fee_numerator: 1,
        withdrawal_fee_denominator: 1_000,
        current_technical_floor_units: 100,
        maximum_safe_capacity_units,
        jitosol_input_units: input_units,
        withdrawal_fee_units: fee_units,
        burned_units: input_units - fee_units,
        expected_native_lamports: delegated_native_lamports,
        observed_delegated_native_lamports: delegated_native_lamports,
        minimum_native_lamports: delegated_native_lamports,
        stake_rent_advanced_lamports: 10 + leg_index,
        metadata_rent_advanced_lamports: 5 + leg_index,
    };
    initiate_withdrawal_leg(config, round, leg, input).expect("legal leg initiation");
}

fn finalize_leg(
    config: &PivConfig,
    round: &mut ActiveDistribution,
    leg: &mut WithdrawalLeg,
) -> LegFinalizationOutcome {
    let finalized_native_lamports = leg.observed_delegated_native_lamports
        + leg.stake_rent_advanced_lamports;
    let cumulative_finalized_native = round.cumulative_finalized_native_lamports
        + finalized_native_lamports;
    let cumulative_recovered_stake_rent = round
        .cumulative_recovered_stake_rent_lamports
        + leg.stake_rent_advanced_lamports;
    let escrow_available_after = round.pending_sol_used_lamports
        + cumulative_finalized_native
        - cumulative_recovered_stake_rent;

    finalize_withdrawal_leg(
        config,
        round,
        leg,
        LegFinalizationInput {
            sequence: round.active_sequence,
            leg_index: leg.leg_index,
            finalized_epoch: 44,
            finalized_native_lamports,
            recovered_stake_rent_lamports: leg.stake_rent_advanced_lamports,
            recovered_metadata_rent_lamports: leg.metadata_rent_advanced_lamports,
            cooldown_reward_lamports: 0,
            cooldown_loss_lamports: 0,
            validated_residual_historical_value_lamports:
                round.stored_residual_hwm_floor_lamports,
            escrow_available_after_lamports: escrow_available_after,
        },
    )
    .expect("legal leg finalization")
}

fn settle(fixture: &mut Fixture) -> SettlementOutcome {
    let sequence = fixture.round.active_sequence;
    let escrow_available_lamports = fixture.round.recorded_escrow_available_lamports;
    let eligible_native = fixture.round.pending_sol_used_lamports
        + fixture
            .round
            .prior_next_cycle_yield_used_lamports()
            .expect("valid prior-yield funding")
        + fixture.round.cumulative_finalized_delegated_native_lamports;
    let actual_net = eligible_native.min(fixture.round.outgoing_gross_obligation_lamports);
    // Task 1.3 intentionally does not choose a production shortfall-allocation
    // policy. These values are one deterministic, cap-respecting fixture that
    // stands in for facts a future handler must validate under founder policy.
    let actual_htfp_lamports = actual_net.min(fixture.round.htfp_gross_obligation_lamports);
    let after_htfp = actual_net - actual_htfp_lamports;
    let actual_team_owner_lamports =
        after_htfp.min(fixture.round.team_owner_gross_obligation_lamports);
    let after_team = after_htfp - actual_team_owner_lamports;
    let actual_kif_allocation_lamports =
        after_team.min(fixture.round.kif_gross_obligation_lamports);
    settle_distribution(
        &mut fixture.config,
        &mut fixture.round,
        &mut fixture.rewards,
        SettlementInput {
            sequence,
            escrow_available_lamports,
            actual_htfp_lamports,
            actual_team_owner_lamports,
            actual_kif_allocation_lamports,
            validated_post_settlement_protected_value_lamports: u64::MAX,
        },
    )
    .expect("legal settlement")
}

fn fixed_snapshot(round: &ActiveDistribution) -> FixedSnapshot {
    FixedSnapshot {
        active_sequence: round.active_sequence,
        last_completed: round.last_completed,
        prepared_at: round.prepared_at,
        prepared_slot: round.prepared_slot,
        prepared_epoch: round.prepared_epoch,
        old_protected_principal_lamports: round.old_protected_principal_lamports,
        historical_jitosol_units: round.historical_jitosol_units,
        historical_sol_lamports: round.historical_sol_lamports,
        historical_value_lamports: round.historical_value_lamports,
        snapshot_pool_total_lamports: round.snapshot_pool_total_lamports,
        snapshot_pool_token_supply: round.snapshot_pool_token_supply,
        snapshot_withdrawal_fee_numerator: round.snapshot_withdrawal_fee_numerator,
        snapshot_withdrawal_fee_denominator: round.snapshot_withdrawal_fee_denominator,
        gross_yield_lamports: round.gross_yield_lamports,
        prior_next_cycle_yield_lamports: round.prior_next_cycle_yield_lamports,
        htfp_gross_obligation_lamports: round.htfp_gross_obligation_lamports,
        permanent_compound_lamports: round.permanent_compound_lamports,
        team_owner_gross_obligation_lamports: round.team_owner_gross_obligation_lamports,
        kif_gross_obligation_lamports: round.kif_gross_obligation_lamports,
        split_dust_lamports: round.split_dust_lamports,
        outgoing_gross_obligation_lamports: round.outgoing_gross_obligation_lamports,
        pending_sol_snapshot_lamports: round.pending_sol_snapshot_lamports,
        pending_sol_used_lamports: round.pending_sol_used_lamports,
        snapshot_conversion_dust_lamports: round.snapshot_conversion_dust_lamports,
        fixed_jitosol_withdrawal_target_units: round.fixed_jitosol_withdrawal_target_units,
        snapshot_leg_input_floor_units: round.snapshot_leg_input_floor_units,
        maximum_useful_legs: round.maximum_useful_legs,
        stored_round_minimum_native_lamports: round.stored_round_minimum_native_lamports,
        stored_residual_hwm_floor_lamports: round.stored_residual_hwm_floor_lamports,
        stored_slippage_bps: round.stored_slippage_bps,
        htfp_recipient: round.htfp_recipient,
        team_owner_recipient: round.team_owner_recipient,
        guardian_registry: round.guardian_registry,
        guardian_registry_revision: round.guardian_registry_revision,
        guardian_keys: round.guardian_keys,
        kif_eligibility_bitmap: round.kif_eligibility_bitmap,
        kif_active_guardian_count: round.kif_active_guardian_count,
        kif_period_id: round.kif_period_id,
        kif_carry_input_lamports: round.kif_carry_input_lamports,
        proposed_hwm_delta_lamports: round.proposed_hwm_delta_lamports,
        proposed_hwm_after_settlement_lamports:
            round.proposed_hwm_after_settlement_lamports,
    }
}

#[test]
fn no_yield_and_valid_insufficient_results_preserve_their_distinct_clocks() {
    let mut fixture = fixture(1_000, &[]);
    fixture.config.last_successful_preparation_at = Some(100);
    let evaluated_at = 100 + MINIMUM_DISTRIBUTION_INTERVAL_SECONDS;
    let config_before = fixture.config.clone();
    let round_before = fixture.round;

    record_no_yield_evaluation(
        &fixture.config,
        &fixture.round,
        evaluated_at,
        fixture.config.protected_principal_hwm_lamports,
    )
    .expect("legal no-yield evaluation");
    assert_eq!(fixture.config, config_before);
    assert_eq!(fixture.round, round_before);

    record_valid_insufficient_attempt(
        &mut fixture.config,
        &fixture.round,
        evaluated_at,
    )
    .expect("legal valid-insufficient result");
    let mut expected_config = config_before;
    expected_config.last_valid_insufficient_attempt_at = Some(evaluated_at);
    assert_eq!(fixture.config, expected_config);
    assert_eq!(fixture.config.last_successful_preparation_at, Some(100));
    assert_eq!(fixture.round, round_before);
}

#[test]
fn liquid_round_settles_integrates_and_preserves_monotonic_sequence() {
    let mut fixture = fixture(10_000, &[0, 1]);
    let input = liquid_open_input(&fixture.config, PREPARED_AT);
    open_distribution(
        &mut fixture.config,
        &mut fixture.round,
        &fixture.registry,
        &fixture.rewards,
        input,
    )
    .expect("open liquid round");

    assert_eq!(fixture.round.lifecycle, DistributionLifecycle::EscrowFunded);
    assert_eq!(fixture.round.kif_eligibility_bitmap, 0b00_0011);
    assert_eq!(fixture.config.next_distribution_sequence, 1);
    let immutable = fixed_snapshot(&fixture.round);

    assert_eq!(settle(&mut fixture), SettlementOutcome::Settled);
    assert_eq!(fixed_snapshot(&fixture.round), immutable);
    assert_eq!(fixture.round.actual_kif_liability_lamports, 200);
    assert_eq!(fixture.round.actual_kif_carry_next_lamports, 1);
    assert_eq!(fixture.rewards[0].claimable_lamports, 100);
    assert_eq!(fixture.rewards[1].claimable_lamports, 100);
    assert!(fixture.rewards[2..]
        .iter()
        .all(|reward| reward.claimable_lamports == 0));

    let contribution_value = 10_500;
    let expected_hwm = fixture.config.protected_principal_hwm_lamports
        + contribution_value;
    let summary = integrate_pending_and_complete(
        &mut fixture.config,
        &mut fixture.round,
        PendingIntegrationInput {
            sequence: 0,
            completed_at: PREPARED_AT + 1,
            integrated_pending_sol_lamports: 10_000,
            integrated_pending_jitosol_units: 50,
            contribution_value_lamports: contribution_value,
            new_accounted_historical_jitosol_units: 1_050,
            new_accounted_historical_sol_lamports: 0,
            new_protected_hwm_lamports: expected_hwm,
        },
    )
    .expect("integrate pending contribution and complete");

    assert_eq!(summary.sequence, 0);
    assert_eq!(summary.final_protected_hwm_lamports, expected_hwm);
    assert_eq!(fixture.round.lifecycle, DistributionLifecycle::Idle);
    assert_eq!(fixture.round.last_completed, Some(summary));
    assert_eq!(fixture.config.next_distribution_sequence, 1);
    assert_eq!(fixture.config.accounted_pending_sol_lamports, 0);
    assert_eq!(fixture.config.accounted_pending_jitosol_units, 0);

    fixture.config.accounted_pending_sol_lamports = OUTGOING_GROSS;
    let second_preparation = PREPARED_AT + MINIMUM_DISTRIBUTION_INTERVAL_SECONDS;
    let second_input = liquid_open_input(&fixture.config, second_preparation);
    open_distribution(
        &mut fixture.config,
        &mut fixture.round,
        &fixture.registry,
        &fixture.rewards,
        second_input,
    )
    .expect("open next monotonic sequence at exact timing boundary");
    assert_eq!(fixture.round.active_sequence, 1);
    assert_eq!(fixture.round.last_completed, Some(summary));
    assert_eq!(fixture.config.next_distribution_sequence, 2);
}

#[test]
fn one_leg_withdrawal_assigns_finalizes_and_settles() {
    let mut fixture = fixture(1_000, &[0]);
    let open_input = withdrawal_open_input(&fixture.config, 200, 500);
    open_distribution(
        &mut fixture.config,
        &mut fixture.round,
        &fixture.registry,
        &fixture.rewards,
        open_input,
    )
    .expect("open one-leg withdrawal round");
    assert!(fixture.round.is_prepared_withdrawal());
    let immutable = fixed_snapshot(&fixture.round);

    let mut leg = WithdrawalLeg::vacant(31, 41);
    initiate_leg(
        &fixture.config,
        &mut fixture.round,
        &mut leg,
        500,
        800,
    );
    assert!(fixture.round.is_withdrawal_target_assigned());
    assert_eq!(fixed_snapshot(&fixture.round), immutable);

    assert_eq!(
        finalize_leg(&fixture.config, &mut fixture.round, &mut leg),
        LegFinalizationOutcome::EscrowFunded
    );
    assert_eq!(fixture.round.lifecycle, DistributionLifecycle::EscrowFunded);
    assert_eq!(fixture.round.recorded_escrow_available_lamports, 1_800);
    assert_eq!(fixed_snapshot(&fixture.round), immutable);

    assert_eq!(settle(&mut fixture), SettlementOutcome::Settled);
    assert_eq!(fixture.round.actual_net_available_lamports, 1_800);
    assert_eq!(fixture.round.actual_net_allocation_dust_lamports, 0);
    assert_eq!(fixed_snapshot(&fixture.round), immutable);
}

fn run_two_leg_finalization_order(order: [usize; 2]) {
    let mut fixture = fixture(1_000, &[0, 2]);
    let open_input = withdrawal_open_input(&fixture.config, 300, 800);
    open_distribution(
        &mut fixture.config,
        &mut fixture.round,
        &fixture.registry,
        &fixture.rewards,
        open_input,
    )
    .expect("open multi-leg round");
    let immutable = fixed_snapshot(&fixture.round);

    let mut legs = [WithdrawalLeg::vacant(31, 41), WithdrawalLeg::vacant(32, 42)];
    initiate_leg(
        &fixture.config,
        &mut fixture.round,
        &mut legs[0],
        200,
        500,
    );
    initiate_leg(
        &fixture.config,
        &mut fixture.round,
        &mut legs[1],
        500,
        300,
    );
    assert!(fixture.round.is_withdrawal_target_assigned());
    assert_eq!(fixture.round.finalized_leg_count, 0);
    assert_eq!(fixed_snapshot(&fixture.round), immutable);

    assert_eq!(
        finalize_leg(
            &fixture.config,
            &mut fixture.round,
            &mut legs[order[0]],
        ),
        LegFinalizationOutcome::Recorded
    );
    assert!(fixture.round.is_partially_finalized());
    assert_eq!(fixed_snapshot(&fixture.round), immutable);

    assert_eq!(
        finalize_leg(
            &fixture.config,
            &mut fixture.round,
            &mut legs[order[1]],
        ),
        LegFinalizationOutcome::EscrowFunded
    );
    assert!(fixture.round.is_withdrawal_complete());
    assert_eq!(fixture.round.recorded_escrow_available_lamports, 1_800);
    assert_eq!(fixed_snapshot(&fixture.round), immutable);
}

#[test]
fn multi_leg_target_assignment_precedes_both_finalization_orders() {
    run_two_leg_finalization_order([0, 1]);
    run_two_leg_finalization_order([1, 0]);
}

#[test]
fn a_leg_can_finalize_while_more_target_assignment_remains() {
    let mut fixture = fixture(1_000, &[0]);
    let open_input = withdrawal_open_input(&fixture.config, 300, 800);
    open_distribution(
        &mut fixture.config,
        &mut fixture.round,
        &fixture.registry,
        &fixture.rewards,
        open_input,
    )
    .expect("open partial-finalization round");
    let immutable = fixed_snapshot(&fixture.round);

    let mut first_leg = WithdrawalLeg::vacant(31, 41);
    initiate_leg(
        &fixture.config,
        &mut fixture.round,
        &mut first_leg,
        200,
        500,
    );
    assert!(fixture.round.is_assigning_withdrawal_legs());
    assert!(!fixture.round.is_withdrawal_target_assigned());
    assert_eq!(
        finalize_leg(&fixture.config, &mut fixture.round, &mut first_leg),
        LegFinalizationOutcome::Recorded
    );
    assert!(fixture.round.is_assigning_withdrawal_legs());
    assert!(fixture.round.is_partially_finalized());

    let mut second_leg = WithdrawalLeg::vacant(32, 42);
    initiate_leg(
        &fixture.config,
        &mut fixture.round,
        &mut second_leg,
        500,
        300,
    );
    assert!(fixture.round.is_withdrawal_target_assigned());
    assert!(fixture.round.is_partially_finalized());
    assert_eq!(
        finalize_leg(&fixture.config, &mut fixture.round, &mut second_leg),
        LegFinalizationOutcome::EscrowFunded
    );
    assert_eq!(fixed_snapshot(&fixture.round), immutable);
}

#[test]
fn zero_active_kif_compounds_half_and_reapplies_collective_carry() {
    let mut fixture = fixture(10_000, &[]);
    let open_input = liquid_open_input(&fixture.config, PREPARED_AT);
    open_distribution(
        &mut fixture.config,
        &mut fixture.round,
        &fixture.registry,
        &fixture.rewards,
        open_input,
    )
    .expect("open zero-active KIF round");
    assert_eq!(fixture.round.kif_eligibility_bitmap, 0);

    assert_eq!(settle(&mut fixture), SettlementOutcome::Settled);
    assert_eq!(fixture.round.actual_kif_allocation_lamports, 200);
    assert_eq!(fixture.round.actual_kif_liability_lamports, 0);
    assert_eq!(fixture.round.actual_zero_active_kif_compound_lamports, 100);
    assert_eq!(fixture.round.actual_kif_carry_next_lamports, 101);
    assert_eq!(fixture.config.collective_kif_carry_lamports, 101);
    assert_eq!(
        fixture.config.cumulative_zero_active_kif_compound_lamports,
        100
    );
    assert!(fixture
        .rewards
        .iter()
        .all(|reward| reward.claimable_lamports == 0));
}

#[test]
fn prior_cycle_yield_is_snapshotted_cleared_and_split_exactly_once() {
    const PRIOR_CYCLE_YIELD: u64 = 1_000;
    const CURRENT_HISTORICAL_YIELD: u64 = GROSS_YIELD - PRIOR_CYCLE_YIELD;
    const PENDING_FUNDING: u64 = OUTGOING_GROSS - PRIOR_CYCLE_YIELD;

    let mut fixture = fixture(PENDING_FUNDING, &[0]);
    fixture.config.next_cycle_yield_lamports = PRIOR_CYCLE_YIELD;

    let mut input = liquid_open_input(&fixture.config, PREPARED_AT);
    input.historical_value_lamports =
        fixture.config.protected_principal_hwm_lamports + CURRENT_HISTORICAL_YIELD;
    input.pending_sol_used_lamports = PENDING_FUNDING;

    let config_before = fixture.config.clone();
    let round_before = fixture.round;
    let mut rejected = input;
    rejected.gross_yield_lamports = GROSS_YIELD - 1;
    assert!(open_distribution(
        &mut fixture.config,
        &mut fixture.round,
        &fixture.registry,
        &fixture.rewards,
        rejected,
    )
    .is_err());
    assert_eq!(fixture.config, config_before);
    assert_eq!(fixture.round, round_before);

    open_distribution(
        &mut fixture.config,
        &mut fixture.round,
        &fixture.registry,
        &fixture.rewards,
        input,
    )
    .expect("open with prior-cycle yield as explicit liquid funding");

    assert_eq!(fixture.round.gross_yield_lamports, GROSS_YIELD);
    assert_eq!(
        fixture.round.prior_next_cycle_yield_lamports,
        PRIOR_CYCLE_YIELD
    );
    assert_eq!(
        fixture
            .round
            .prior_next_cycle_yield_used_lamports()
            .expect("valid prior-yield funding"),
        PRIOR_CYCLE_YIELD
    );
    assert_eq!(fixture.round.pending_sol_used_lamports, PENDING_FUNDING);
    assert_eq!(
        fixture.round.recorded_escrow_available_lamports,
        OUTGOING_GROSS
    );
    assert_eq!(fixture.config.next_cycle_yield_lamports, 0);

    assert_eq!(settle(&mut fixture), SettlementOutcome::Settled);
    assert_eq!(fixture.config.cumulative_gross_yield_lamports, GROSS_YIELD);
    assert_eq!(fixture.config.next_cycle_yield_lamports, 0);
}

#[test]
fn cooldown_reward_becomes_next_cycle_yield_and_is_split_on_the_next_open() {
    const COOLDOWN_REWARD: u64 = 25;
    const NEXT_HISTORICAL_YIELD: u64 = GROSS_YIELD - COOLDOWN_REWARD;
    const NEXT_PENDING_FUNDING: u64 = OUTGOING_GROSS - COOLDOWN_REWARD;

    let mut fixture = fixture(1_000, &[0]);
    let open_input = withdrawal_open_input(&fixture.config, 200, 500);
    open_distribution(
        &mut fixture.config,
        &mut fixture.round,
        &fixture.registry,
        &fixture.rewards,
        open_input,
    )
    .expect("open cooldown-reward withdrawal round");

    let mut leg = WithdrawalLeg::vacant(31, 41);
    initiate_leg(
        &fixture.config,
        &mut fixture.round,
        &mut leg,
        500,
        700,
    );
    let finalized_native_lamports = leg.observed_delegated_native_lamports
        + leg.stake_rent_advanced_lamports
        + COOLDOWN_REWARD;
    let escrow_available_after_lamports = fixture.round.pending_sol_used_lamports
        + finalized_native_lamports
        - leg.stake_rent_advanced_lamports;
    let sequence = fixture.round.active_sequence;
    let leg_index = leg.leg_index;
    let stake_rent = leg.stake_rent_advanced_lamports;
    let metadata_rent = leg.metadata_rent_advanced_lamports;
    let residual_hwm_floor = fixture.round.stored_residual_hwm_floor_lamports;
    assert_eq!(
        finalize_withdrawal_leg(
            &fixture.config,
            &mut fixture.round,
            &mut leg,
            LegFinalizationInput {
                sequence,
                leg_index,
                finalized_epoch: 44,
                finalized_native_lamports,
                recovered_stake_rent_lamports: stake_rent,
                recovered_metadata_rent_lamports: metadata_rent,
                cooldown_reward_lamports: COOLDOWN_REWARD,
                cooldown_loss_lamports: 0,
                validated_residual_historical_value_lamports: residual_hwm_floor,
                escrow_available_after_lamports,
            },
        )
        .expect("finalize with a positive cooldown reward"),
        LegFinalizationOutcome::EscrowFunded
    );

    assert_eq!(settle(&mut fixture), SettlementOutcome::Settled);
    assert_eq!(
        fixture.config.next_cycle_yield_lamports,
        COOLDOWN_REWARD
    );
    assert_eq!(
        fixture.config.cumulative_cooldown_yield_recorded_lamports,
        COOLDOWN_REWARD
    );
    assert_eq!(
        fixture.round.actual_escrow_remainder_lamports,
        COOLDOWN_REWARD
    );

    let settled_hwm = fixture.config.protected_principal_hwm_lamports;
    integrate_pending_and_complete(
        &mut fixture.config,
        &mut fixture.round,
        PendingIntegrationInput {
            sequence: 0,
            completed_at: PREPARED_AT + 1,
            integrated_pending_sol_lamports: 1_000,
            integrated_pending_jitosol_units: 50,
            contribution_value_lamports: 1_000,
            new_accounted_historical_jitosol_units: 1_050,
            new_accounted_historical_sol_lamports: 0,
            new_protected_hwm_lamports: settled_hwm + 1_000,
        },
    )
    .expect("complete cooldown-reward source round");

    fixture.config.accounted_pending_sol_lamports = NEXT_PENDING_FUNDING;
    let next_prepared_at = PREPARED_AT + MINIMUM_DISTRIBUTION_INTERVAL_SECONDS;
    let mut next_input = liquid_open_input(&fixture.config, next_prepared_at);
    next_input.historical_value_lamports =
        fixture.config.protected_principal_hwm_lamports + NEXT_HISTORICAL_YIELD;
    next_input.pending_sol_used_lamports = NEXT_PENDING_FUNDING;
    open_distribution(
        &mut fixture.config,
        &mut fixture.round,
        &fixture.registry,
        &fixture.rewards,
        next_input,
    )
    .expect("split cooldown reward as part of the next cycle's gross yield");

    assert_eq!(fixture.round.active_sequence, 1);
    assert_eq!(fixture.round.gross_yield_lamports, GROSS_YIELD);
    assert_eq!(
        fixture.round.prior_next_cycle_yield_lamports,
        COOLDOWN_REWARD
    );
    assert_eq!(fixture.round.htfp_gross_obligation_lamports, 5_900);
    assert_eq!(fixture.round.permanent_compound_lamports, 1_950);
    assert_eq!(fixture.round.team_owner_gross_obligation_lamports, 1_950);
    assert_eq!(fixture.round.kif_gross_obligation_lamports, 200);
    assert_eq!(fixture.config.next_cycle_yield_lamports, 0);
}

#[test]
fn post_snapshot_pending_contributions_do_not_change_obligations_and_fully_integrate() {
    const LATE_PENDING_SOL: u64 = 2_000;
    const LATE_PENDING_JITOSOL: u64 = 10;

    let mut fixture = fixture(10_000, &[0]);
    let input = liquid_open_input(&fixture.config, PREPARED_AT);
    open_distribution(
        &mut fixture.config,
        &mut fixture.round,
        &fixture.registry,
        &fixture.rewards,
        input,
    )
    .expect("open before later contributions arrive");
    let immutable = fixed_snapshot(&fixture.round);

    fixture.config.accounted_pending_sol_lamports += LATE_PENDING_SOL;
    fixture.config.accounted_pending_jitosol_units += LATE_PENDING_JITOSOL;
    fixture
        .config
        .validate_initialized()
        .expect("post-snapshot pending accounting remains valid");
    assert_eq!(fixed_snapshot(&fixture.round), immutable);
    assert_eq!(fixture.round.pending_sol_snapshot_lamports, 10_000);
    assert_eq!(fixture.round.outgoing_gross_obligation_lamports, OUTGOING_GROSS);

    assert_eq!(settle(&mut fixture), SettlementOutcome::Settled);
    assert_eq!(fixed_snapshot(&fixture.round), immutable);

    let all_pending_sol = 10_000 + LATE_PENDING_SOL;
    let all_pending_jitosol = 50 + LATE_PENDING_JITOSOL;
    let contribution_value = all_pending_sol + 600;
    let expected_hwm = fixture.config.protected_principal_hwm_lamports
        + contribution_value;
    let summary = integrate_pending_and_complete(
        &mut fixture.config,
        &mut fixture.round,
        PendingIntegrationInput {
            sequence: 0,
            completed_at: PREPARED_AT + 1,
            integrated_pending_sol_lamports: all_pending_sol,
            integrated_pending_jitosol_units: all_pending_jitosol,
            contribution_value_lamports: contribution_value,
            new_accounted_historical_jitosol_units: 1_000 + all_pending_jitosol,
            new_accounted_historical_sol_lamports: 0,
            new_protected_hwm_lamports: expected_hwm,
        },
    )
    .expect("integrate both snapshotted and later pending contributions");

    assert_eq!(summary.gross_yield_lamports, GROSS_YIELD);
    assert_eq!(
        summary.integrated_contribution_value_lamports,
        contribution_value
    );
    assert_eq!(fixture.config.accounted_pending_sol_lamports, 0);
    assert_eq!(fixture.config.accounted_pending_jitosol_units, 0);
    assert_eq!(
        fixture.config.accounted_historical_jitosol_units,
        1_000 + all_pending_jitosol
    );
    assert_eq!(fixture.config.protected_principal_hwm_lamports, expected_hwm);
}

#[test]
fn settlement_credits_the_immutable_guardian_snapshot_after_live_rotation() {
    let mut fixture = fixture(10_000, &[0, 1]);
    let input = liquid_open_input(&fixture.config, PREPARED_AT);
    open_distribution(
        &mut fixture.config,
        &mut fixture.round,
        &fixture.registry,
        &fixture.rewards,
        input,
    )
    .expect("open with guardian revision seven");
    let snapshotted_keys = fixture.round.guardian_keys;
    assert_eq!(fixture.round.guardian_registry_revision, 7);

    let rotated_keys = core::array::from_fn(|index| key(200 + index as u8));
    let rotated_registry = GuardianRegistry::new(91, 8, rotated_keys)
        .expect("valid rotated guardian registry");
    fixture.registry = rotated_registry;
    fixture.config.guardian_registry_revision = rotated_registry.revision;
    fixture
        .config
        .validate_initialized()
        .expect("live config accepts the newer registry revision");
    assert_ne!(fixture.registry.guardian_keys, snapshotted_keys);

    assert_eq!(settle(&mut fixture), SettlementOutcome::Settled);
    assert_eq!(fixture.round.guardian_registry_revision, 7);
    assert_eq!(fixture.round.guardian_keys, snapshotted_keys);
    assert_eq!(fixture.rewards[0].guardian, snapshotted_keys[0]);
    assert_eq!(fixture.rewards[1].guardian, snapshotted_keys[1]);
    assert_eq!(fixture.rewards[0].registry_revision, 7);
    assert_eq!(fixture.rewards[1].registry_revision, 7);
    assert_eq!(fixture.rewards[0].claimable_lamports, 100);
    assert_eq!(fixture.rewards[1].claimable_lamports, 100);
}
