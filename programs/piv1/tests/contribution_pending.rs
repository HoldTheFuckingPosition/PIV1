mod support;

use anchor_lang::{prelude::Pubkey, AnchorSerialize};

use piv1::{
    constants::{
        CONFIG_MIGRATION_RESERVE_BYTES, KIF_PERIOD_SECONDS,
        MAX_CONFIGURED_SLIPPAGE_BPS, MINIMUM_DISTRIBUTION_INTERVAL_SECONDS,
        INSUFFICIENT_RETRY_COOLDOWN_SECONDS, RECOVERY_FLAG_RESIDUAL_HWM,
        STATE_LAYOUT_VERSION,
    },
    errors::Piv1Error,
    state::{
        reconcile_pending_contributions, record_explicit_jitosol_contribution,
        record_explicit_sol_contribution, ActiveDistribution,
        DistributionLifecycle, JitoSolCustodyObservation,
        PendingCustodyObservation, PivConfig, PivConfigBumps,
        SolCustodyObservation,
    },
};

use support::contribution_custody_mock::{
    MockContributionCustody, MockContributionError,
    MockContributionFailurePoint,
};

const RANDOM_SEED: u64 = 0x5049_5631_434f_4e54;
const RANDOM_CASES: usize = 1_024;
const ACTIONS_PER_CASE: usize = 64;

fn key(tag: u8) -> Pubkey {
    Pubkey::new_from_array([tag; 32])
}

fn valid_config(
    pending_sol_lamports: u64,
    pending_jitosol_units: u64,
) -> PivConfig {
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
        minimum_distribution_interval_seconds:
            MINIMUM_DISTRIBUTION_INTERVAL_SECONDS,
        insufficient_retry_cooldown_seconds:
            INSUFFICIENT_RETRY_COOLDOWN_SECONDS,
        last_successful_preparation_at: None,
        last_valid_insufficient_attempt_at: None,
        next_distribution_sequence: 0,
        protected_principal_hwm_lamports: 1_000,
        accounted_historical_jitosol_units: 2_000,
        accounted_historical_sol_lamports: 3_000,
        accounted_pending_jitosol_units: pending_jitosol_units,
        accounted_pending_sol_lamports: pending_sol_lamports,
        next_cycle_yield_lamports: 4_000,
        kif_claim_liability_lamports: 10,
        collective_kif_carry_lamports: 11,
        cumulative_contribution_value_lamports: 12,
        cumulative_gross_yield_lamports: 13,
        cumulative_htfp_paid_lamports: 14,
        cumulative_team_owner_paid_lamports: 15,
        cumulative_kif_credited_lamports: 17,
        cumulative_kif_claimed_lamports: 7,
        cumulative_permanent_compound_lamports: 18,
        cumulative_retained_dust_lamports: 19,
        cumulative_zero_active_kif_compound_lamports: 20,
        cumulative_cooldown_yield_recorded_lamports: 21,
        kif_anchor_timestamp: 0,
        kif_period_seconds: KIF_PERIOD_SECONDS,
        guardian_registry_revision: 7,
        migration_reserve: [0; CONFIG_MIGRATION_RESERVE_BYTES],
    }
}

fn prepared_round() -> ActiveDistribution {
    let mut round = ActiveDistribution::new_idle(9);
    round.lifecycle = DistributionLifecycle::WithdrawalActive;
    round.active_sequence = 7;
    round.prepared_at = 1_000;
    round.prepared_slot = 2_000;
    round.prepared_epoch = 3;
    round.old_protected_principal_lamports = 1_000;
    round.historical_jitosol_units = 500;
    round.historical_sol_lamports = 100;
    round.historical_value_lamports = 1_100;
    round.snapshot_pool_total_lamports = 2_000;
    round.snapshot_pool_token_supply = 1_000;
    round.snapshot_withdrawal_fee_numerator = 1;
    round.snapshot_withdrawal_fee_denominator = 1_000;
    round.gross_yield_lamports = 100;
    round.prior_next_cycle_yield_lamports = 0;
    round.htfp_gross_obligation_lamports = 59;
    round.permanent_compound_lamports = 19;
    round.team_owner_gross_obligation_lamports = 19;
    round.kif_gross_obligation_lamports = 2;
    round.split_dust_lamports = 1;
    round.outgoing_gross_obligation_lamports = 80;
    round.pending_sol_snapshot_lamports = 10;
    round.pending_sol_used_lamports = 10;
    round.fixed_jitosol_withdrawal_target_units = 100;
    round.snapshot_leg_input_floor_units = 25;
    round.maximum_useful_legs = 4;
    round.stored_round_minimum_native_lamports = 80;
    round.stored_residual_hwm_floor_lamports = 1_020;
    round.stored_slippage_bps = 1;
    round.recorded_escrow_available_lamports = 10;
    round.outstanding_active_round_liability_lamports = 80;
    round.htfp_recipient = key(30);
    round.team_owner_recipient = key(31);
    round.guardian_registry = key(32);
    round.guardian_registry_revision = 4;
    round.guardian_keys = core::array::from_fn(|index| key(40 + index as u8));
    round.kif_eligibility_bitmap = 0b00_1011;
    round.kif_active_guardian_count = 3;
    round.kif_period_id = 8;
    round.kif_carry_input_lamports = 9;
    round.proposed_hwm_delta_lamports = 20;
    round.proposed_hwm_after_settlement_lamports = 1_020;
    round.validate().expect("valid prepared round");
    round
}

fn assigning_round() -> ActiveDistribution {
    let mut round = prepared_round();
    round.cumulative_jitosol_assigned_units = 50;
    round.cumulative_withdrawal_fee_units = 2;
    round.cumulative_burned_units = 48;
    round.cumulative_expected_native_lamports = 60;
    round.cumulative_delegated_native_lamports = 60;
    round.next_leg_index = 2;
    round.successful_leg_count = 2;
    round.validate().expect("valid assigning round");
    round
}

fn partially_finalized_round() -> ActiveDistribution {
    let mut round = assigning_round();
    round.cumulative_finalized_delegated_native_lamports = 30;
    round.cumulative_finalized_native_lamports = 30;
    round.finalized_leg_count = 1;
    round.recorded_escrow_available_lamports = 40;
    round.validate().expect("valid partially finalized round");
    round
}

fn target_assigned_round() -> ActiveDistribution {
    let mut round = prepared_round();
    round.cumulative_jitosol_assigned_units = 100;
    round.cumulative_withdrawal_fee_units = 2;
    round.cumulative_burned_units = 98;
    round.cumulative_expected_native_lamports = 100;
    round.cumulative_delegated_native_lamports = 100;
    round.next_leg_index = 2;
    round.successful_leg_count = 2;
    round.validate().expect("valid target-assigned round");
    round
}

fn escrow_funded_round() -> ActiveDistribution {
    let mut round = target_assigned_round();
    round.lifecycle = DistributionLifecycle::EscrowFunded;
    round.cumulative_finalized_delegated_native_lamports = 100;
    round.cumulative_finalized_native_lamports = 100;
    round.finalized_leg_count = 2;
    round.recorded_escrow_available_lamports = 110;
    round.validate().expect("valid escrow-funded round");
    round
}

fn settled_round() -> ActiveDistribution {
    let mut round = escrow_funded_round();
    round.lifecycle = DistributionLifecycle::Settled;
    round.settlement_recorded = true;
    round.outstanding_active_round_liability_lamports = 0;
    round.actual_net_available_lamports = 80;
    round.actual_htfp_lamports = 58;
    round.actual_team_owner_lamports = 19;
    round.actual_kif_allocation_lamports = 1;
    round.actual_net_allocation_dust_lamports = 2;
    round.actual_allocated_outgoing_lamports = 78;
    round.actual_escrow_remainder_lamports = 32;
    round.actual_retained_conservative_dust_lamports = 30;
    round.actual_kif_liability_lamports = 9;
    round.actual_kif_carry_next_lamports = 1;
    round.actual_hwm_delta_lamports = 52;
    round.settled_protected_hwm_lamports = 1_052;
    round.validate().expect("valid settled round");
    round
}

fn recovery_round() -> ActiveDistribution {
    let mut round = prepared_round();
    round.lifecycle = DistributionLifecycle::RecoveryRequired;
    round.recovery_flags = RECOVERY_FLAG_RESIDUAL_HWM;
    round.validate().expect("valid recovery round");
    round
}

fn phase_rounds() -> [(&'static str, ActiveDistribution); 8] {
    [
        ("idle", ActiveDistribution::new_idle(9)),
        ("prepared", prepared_round()),
        ("assigning", assigning_round()),
        ("partially-finalized", partially_finalized_round()),
        ("target-assigned", target_assigned_round()),
        ("escrow-funded", escrow_funded_round()),
        ("settled", settled_round()),
        ("recovery-required", recovery_round()),
    ]
}

fn config_for_round(
    round: &ActiveDistribution,
    pending_sol_lamports: u64,
    pending_jitosol_units: u64,
) -> PivConfig {
    let mut config = valid_config(pending_sol_lamports, pending_jitosol_units);
    if round.lifecycle != DistributionLifecycle::Idle {
        config.next_distribution_sequence = round.active_sequence + 1;
        config.last_successful_preparation_at = Some(round.prepared_at);
        config.accounted_historical_jitosol_units = round.historical_jitosol_units;
        config.accounted_historical_sol_lamports = round.historical_sol_lamports;
        config.protected_principal_hwm_lamports = round.old_protected_principal_lamports;
        config.next_cycle_yield_lamports = 0;
        config.collective_kif_carry_lamports = round.kif_carry_input_lamports;
    }
    if round.lifecycle == DistributionLifecycle::Settled {
        config.protected_principal_hwm_lamports =
            round.settled_protected_hwm_lamports;
        config.collective_kif_carry_lamports = round.actual_kif_carry_next_lamports;
        config.next_cycle_yield_lamports = round.cumulative_cooldown_rewards_lamports;
        config.kif_claim_liability_lamports = 9;
        config.cumulative_kif_credited_lamports = 17;
        config.cumulative_kif_claimed_lamports = 8;
    }
    config.validate_initialized().expect("valid phase config");
    config
}

fn active_bytes(round: &ActiveDistribution) -> Vec<u8> {
    round.try_to_vec().expect("serialize active distribution")
}

#[test]
fn zero_explicit_contributions_reject_and_preserve_full_mock_state() {
    let config = valid_config(10, 20);
    let mut custody = MockContributionCustody::new(
        config,
        ActiveDistribution::new_idle(9),
        890,
    )
    .expect("valid mock");

    let before_sol = custody.clone();
    assert_eq!(
        custody.record_explicit_sol(0, 0),
        Err(MockContributionError::Accounting(
            Piv1Error::ZeroContribution
        ))
    );
    assert_eq!(custody, before_sol);

    let before_jitosol = custody.clone();
    assert_eq!(
        custody.record_explicit_jitosol(0, 0),
        Err(MockContributionError::Accounting(
            Piv1Error::ZeroContribution
        ))
    );
    assert_eq!(custody, before_jitosol);
}

#[test]
fn exact_explicit_sol_and_jitosol_change_only_their_pending_ledgers() {
    let round = prepared_round();
    let config = config_for_round(&round, 10, 20);
    let initial_config = config.clone();
    let initial_round_bytes = active_bytes(&round);
    let mut custody =
        MockContributionCustody::new(config, round, 890).expect("valid mock");

    let sol = custody
        .record_explicit_sol(25, 25)
        .expect("exact SOL contribution");
    assert_eq!(sol.observed_increase, 25);
    assert_eq!(sol.accounted_pending_after, 35);
    let mut expected_after_sol = initial_config.clone();
    expected_after_sol.accounted_pending_sol_lamports = 35;
    assert_eq!(custody.config, expected_after_sol);
    assert_eq!(custody.pending_sol_vault_lamports, 915);
    assert_eq!(custody.pending_sol_excluded_lamports, 890);

    let jitosol = custody
        .record_explicit_jitosol(30, 30)
        .expect("exact JitoSOL contribution");
    assert_eq!(jitosol.observed_increase, 30);
    assert_eq!(jitosol.accounted_pending_after, 50);
    let mut expected_final = expected_after_sol;
    expected_final.accounted_pending_jitosol_units = 50;
    assert_eq!(custody.config, expected_final);
    assert_eq!(custody.pending_jitosol_token_units, 50);
    assert_eq!(active_bytes(&custody.active_distribution), initial_round_bytes);
    assert_eq!(custody.audit.explicit_sol_lamports, 25);
    assert_eq!(custody.audit.explicit_jitosol_units, 30);
    assert_eq!(custody.audit.outgoing_sol_lamports, 0);
    assert_eq!(custody.audit.outgoing_jitosol_units, 0);
    assert_eq!(custody.validate_conservation(), Ok(()));
}

#[test]
fn malformed_mismatched_and_decreasing_observations_are_atomic() {
    let round = ActiveDistribution::new_idle(9);
    let mut config = valid_config(10, 20);

    let cases = [
        (
            SolCustodyObservation {
                vault_lamports_before: 150,
                vault_lamports_after: 171,
                non_economic_floor_lamports: 100,
            },
            20,
            Piv1Error::ContributionObservationMismatch,
        ),
        (
            SolCustodyObservation {
                vault_lamports_before: 150,
                vault_lamports_after: 149,
                non_economic_floor_lamports: 100,
            },
            1,
            Piv1Error::CustodyBalanceDecreased,
        ),
        (
            SolCustodyObservation {
                vault_lamports_before: 99,
                vault_lamports_after: 101,
                non_economic_floor_lamports: 100,
            },
            1,
            Piv1Error::InvalidCustodyObservation,
        ),
        (
            SolCustodyObservation {
                vault_lamports_before: 105,
                vault_lamports_after: 106,
                non_economic_floor_lamports: 100,
            },
            1,
            Piv1Error::PendingCustodyDeficit,
        ),
    ];
    for (observation, expected, error) in cases {
        let before = config.clone();
        assert_eq!(
            record_explicit_sol_contribution(
                &mut config,
                &round,
                expected,
                observation,
            ),
            Err(error)
        );
        assert_eq!(config, before);
    }

    let before_mismatch = config.clone();
    assert_eq!(
        record_explicit_jitosol_contribution(
            &mut config,
            &round,
            6,
            JitoSolCustodyObservation {
                token_units_before: 100,
                token_units_after: 105,
            },
        ),
        Err(Piv1Error::ContributionObservationMismatch)
    );
    assert_eq!(config, before_mismatch);

    let before_decrease = config.clone();
    assert_eq!(
        record_explicit_jitosol_contribution(
            &mut config,
            &round,
            1,
            JitoSolCustodyObservation {
                token_units_before: 100,
                token_units_after: 99,
            },
        ),
        Err(Piv1Error::CustodyBalanceDecreased)
    );
    assert_eq!(config, before_decrease);

    let before_deficit = config.clone();
    assert_eq!(
        record_explicit_jitosol_contribution(
            &mut config,
            &round,
            1,
            JitoSolCustodyObservation {
                token_units_before: 19,
                token_units_after: 20,
            },
        ),
        Err(Piv1Error::PendingCustodyDeficit)
    );
    assert_eq!(config, before_deficit);
}

#[test]
fn checked_boundaries_reach_u64_max_and_reject_overflow_atomically() {
    let round = ActiveDistribution::new_idle(9);
    let mut sol_config = valid_config(u64::MAX - 1, 0);
    assert_eq!(
        record_explicit_sol_contribution(
            &mut sol_config,
            &round,
            1,
            SolCustodyObservation {
                vault_lamports_before: u64::MAX - 1,
                vault_lamports_after: u64::MAX,
                non_economic_floor_lamports: 0,
            },
        )
        .expect("SOL maximum boundary")
        .accounted_pending_after,
        u64::MAX
    );
    let sol_before_overflow = sol_config.clone();
    assert_eq!(
        record_explicit_sol_contribution(
            &mut sol_config,
            &round,
            1,
            SolCustodyObservation {
                vault_lamports_before: 0,
                vault_lamports_after: 1,
                non_economic_floor_lamports: 0,
            },
        ),
        Err(Piv1Error::ArithmeticOverflow)
    );
    assert_eq!(sol_config, sol_before_overflow);

    let mut jitosol_config = valid_config(0, u64::MAX - 1);
    assert_eq!(
        record_explicit_jitosol_contribution(
            &mut jitosol_config,
            &round,
            1,
            JitoSolCustodyObservation {
                token_units_before: u64::MAX - 1,
                token_units_after: u64::MAX,
            },
        )
        .expect("JitoSOL maximum boundary")
        .accounted_pending_after,
        u64::MAX
    );
    let jitosol_before_overflow = jitosol_config.clone();
    assert_eq!(
        record_explicit_jitosol_contribution(
            &mut jitosol_config,
            &round,
            1,
            JitoSolCustodyObservation {
                token_units_before: 0,
                token_units_after: 1,
            },
        ),
        Err(Piv1Error::ArithmeticOverflow)
    );
    assert_eq!(jitosol_config, jitosol_before_overflow);

    let mut reconcile_config = valid_config(u64::MAX - 1, u64::MAX - 1);
    let reconciled = reconcile_pending_contributions(
        &mut reconcile_config,
        &round,
        PendingCustodyObservation {
            pending_sol_vault_lamports: u64::MAX,
            pending_sol_non_economic_floor_lamports: 0,
            pending_jitosol_token_units: u64::MAX,
        },
    )
    .expect("reconciliation maximum boundary");
    assert_eq!(reconciled.newly_accounted_sol_lamports, 1);
    assert_eq!(reconciled.newly_accounted_jitosol_units, 1);
    assert_eq!(reconcile_config.accounted_pending_sol_lamports, u64::MAX);
    assert_eq!(reconcile_config.accounted_pending_jitosol_units, u64::MAX);
}

#[test]
fn direct_transfers_reconcile_both_assets_once_and_never_as_yield() {
    let round = target_assigned_round();
    let config = config_for_round(&round, 10, 20);
    let before_config = config.clone();
    let round_bytes = active_bytes(&round);
    let mut custody =
        MockContributionCustody::new(config, round, 890).expect("valid mock");

    custody.direct_credit_sol(31).expect("direct SOL credit");
    custody
        .direct_credit_jitosol(47)
        .expect("direct JitoSOL credit");
    assert_eq!(custody.config, before_config);
    assert_eq!(custody.unexplained_sol_lamports(), Ok(31));
    assert_eq!(custody.unexplained_jitosol_units(), Ok(47));

    let first = custody.reconcile().expect("first reconciliation");
    assert_eq!(first.newly_accounted_sol_lamports, 31);
    assert_eq!(first.newly_accounted_jitosol_units, 47);
    let mut expected_config = before_config;
    expected_config.accounted_pending_sol_lamports = 41;
    expected_config.accounted_pending_jitosol_units = 67;
    assert_eq!(custody.config, expected_config);
    assert_eq!(custody.unexplained_sol_lamports(), Ok(0));
    assert_eq!(custody.unexplained_jitosol_units(), Ok(0));

    let state_before_repeat = custody.clone();
    let repeated = custody.reconcile().expect("idempotent reconciliation");
    assert!(repeated.is_no_change());
    assert_eq!(custody, state_before_repeat);
    assert_eq!(active_bytes(&custody.active_distribution), round_bytes);
    assert_eq!(custody.config.next_cycle_yield_lamports, 0);
    assert_eq!(custody.config.protected_principal_hwm_lamports, 1_000);
    assert_eq!(custody.config.cumulative_contribution_value_lamports, 12);
    assert_eq!(custody.validate_conservation(), Ok(()));
}

#[test]
fn reconciliation_deficits_and_malformed_floors_reject_both_assets_atomically() {
    let round = prepared_round();
    let mut config = config_for_round(&round, 20, 20);

    let before_sol_deficit = config.clone();
    assert_eq!(
        reconcile_pending_contributions(
            &mut config,
            &round,
            PendingCustodyObservation {
                pending_sol_vault_lamports: 899,
                pending_sol_non_economic_floor_lamports: 890,
                pending_jitosol_token_units: 100,
            },
        ),
        Err(Piv1Error::PendingCustodyDeficit)
    );
    assert_eq!(config, before_sol_deficit);

    let before_jitosol_deficit = config.clone();
    assert_eq!(
        reconcile_pending_contributions(
            &mut config,
            &round,
            PendingCustodyObservation {
                pending_sol_vault_lamports: 1_000,
                pending_sol_non_economic_floor_lamports: 890,
                pending_jitosol_token_units: 19,
            },
        ),
        Err(Piv1Error::PendingCustodyDeficit)
    );
    assert_eq!(config, before_jitosol_deficit);

    let before_bad_floor = config.clone();
    assert_eq!(
        reconcile_pending_contributions(
            &mut config,
            &round,
            PendingCustodyObservation {
                pending_sol_vault_lamports: 889,
                pending_sol_non_economic_floor_lamports: 890,
                pending_jitosol_token_units: 20,
            },
        ),
        Err(Piv1Error::InvalidCustodyObservation)
    );
    assert_eq!(config, before_bad_floor);
}

#[test]
fn mismatch_overflow_and_every_injected_failure_preserve_the_full_mock() {
    let mut mismatch = MockContributionCustody::new(
        config_for_round(&prepared_round(), 10, 20),
        prepared_round(),
        890,
    )
    .expect("valid mock");
    let mismatch_before = mismatch.clone();
    assert_eq!(
        mismatch.record_explicit_sol(4, 5),
        Err(MockContributionError::Accounting(
            Piv1Error::ContributionObservationMismatch
        ))
    );
    assert_eq!(mismatch, mismatch_before);

    let mut overflow = MockContributionCustody::new(
        valid_config(u64::MAX, 0),
        ActiveDistribution::new_idle(9),
        0,
    )
    .expect("maximum mock");
    let overflow_before = overflow.clone();
    assert_eq!(
        overflow.direct_credit_sol(1),
        Err(MockContributionError::Accounting(
            Piv1Error::ArithmeticOverflow
        ))
    );
    assert_eq!(overflow, overflow_before);

    let points = [
        MockContributionFailurePoint::ExplicitSolAfterPhysicalCredit,
        MockContributionFailurePoint::ExplicitSolAfterAccounting,
        MockContributionFailurePoint::ExplicitJitoSolAfterPhysicalCredit,
        MockContributionFailurePoint::ExplicitJitoSolAfterAccounting,
        MockContributionFailurePoint::ReconciliationAfterAccounting,
    ];
    for point in points {
        let mut custody = MockContributionCustody::new(
            config_for_round(&prepared_round(), 10, 20),
            prepared_round(),
            890,
        )
        .expect("valid mock");
        if point == MockContributionFailurePoint::ReconciliationAfterAccounting {
            custody.direct_credit_sol(3).expect("direct SOL credit");
            custody
                .direct_credit_jitosol(4)
                .expect("direct JitoSOL credit");
        }
        custody.set_failure_point(point);
        let before = custody.clone();
        let result = match point {
            MockContributionFailurePoint::ExplicitSolAfterPhysicalCredit
            | MockContributionFailurePoint::ExplicitSolAfterAccounting => {
                custody.record_explicit_sol(5, 5).map(|_| ())
            }
            MockContributionFailurePoint::ExplicitJitoSolAfterPhysicalCredit
            | MockContributionFailurePoint::ExplicitJitoSolAfterAccounting => {
                custody.record_explicit_jitosol(5, 5).map(|_| ())
            }
            MockContributionFailurePoint::ReconciliationAfterAccounting => {
                custody.reconcile().map(|_| ())
            }
        };
        assert_eq!(
            result,
            Err(MockContributionError::InjectedFailure(point))
        );
        assert_eq!(custody, before);
    }
}

#[test]
fn intake_and_reconciliation_preserve_every_distribution_phase_even_paused() {
    for (phase, round) in phase_rounds() {
        for paused in [false, true] {
            let mut config = config_for_round(&round, 10, 20);
            config.paused = paused;
            let initial_config = config.clone();
            let initial_round = round;
            let initial_round_bytes = active_bytes(&round);
            let mut custody = MockContributionCustody::new(config, round, 890)
                .unwrap_or_else(|error| {
                    panic!("phase={phase} paused={paused} setup={error:?}")
                });

            custody.record_explicit_sol(3, 3).unwrap_or_else(|error| {
                panic!("phase={phase} paused={paused} explicit-sol={error:?}")
            });
            custody
                .record_explicit_jitosol(4, 4)
                .unwrap_or_else(|error| {
                    panic!(
                        "phase={phase} paused={paused} explicit-jitosol={error:?}"
                    )
                });
            custody.direct_credit_sol(5).unwrap_or_else(|error| {
                panic!("phase={phase} paused={paused} direct-sol={error:?}")
            });
            custody
                .direct_credit_jitosol(6)
                .unwrap_or_else(|error| {
                    panic!("phase={phase} paused={paused} direct-jito={error:?}")
                });
            let reconciled = custody.reconcile().unwrap_or_else(|error| {
                panic!("phase={phase} paused={paused} reconcile={error:?}")
            });
            assert_eq!(reconciled.newly_accounted_sol_lamports, 5);
            assert_eq!(reconciled.newly_accounted_jitosol_units, 6);

            let mut expected_config = initial_config;
            expected_config.accounted_pending_sol_lamports = 18;
            expected_config.accounted_pending_jitosol_units = 30;
            assert_eq!(custody.config, expected_config, "phase={phase} paused={paused}");
            assert_eq!(
                custody.active_distribution,
                initial_round,
                "phase={phase} paused={paused}"
            );
            assert_eq!(
                active_bytes(&custody.active_distribution),
                initial_round_bytes,
                "phase={phase} paused={paused}"
            );
            assert_eq!(custody.audit.outgoing_sol_lamports, 0);
            assert_eq!(custody.audit.outgoing_jitosol_units, 0);
            assert_eq!(custody.validate_conservation(), Ok(()));
        }
    }
}

#[test]
fn randomized_pending_custody_model_is_reproducible_and_conservative() {
    let mut rng = SplitMix64::new(RANDOM_SEED);
    for case_index in 0..RANDOM_CASES {
        let phases = phase_rounds();
        let phase_index = (rng.next_u64() as usize) % phases.len();
        let (phase, round) = phases[phase_index];
        let initial_sol = rng.bounded(101) + round.pending_sol_snapshot_lamports;
        let initial_jitosol = rng.bounded(101);
        let floor = rng.bounded(1_001);
        let mut config = config_for_round(&round, initial_sol, initial_jitosol);
        config.paused = rng.next_u64() & 1 == 1;
        let immutable_round_bytes = active_bytes(&round);
        let mut custody = MockContributionCustody::new(config, round, floor)
            .unwrap_or_else(|error| {
                panic!(
                    "seed={RANDOM_SEED:#018x} case={case_index} setup phase={phase} error={error:?}"
                )
            });
        let mut physical_sol = initial_sol - round.pending_sol_used_lamports;
        let mut physical_jitosol = initial_jitosol;
        let mut accounted_sol = initial_sol;
        let mut accounted_jitosol = initial_jitosol;

        for action_index in 0..ACTIONS_PER_CASE {
            let selector = rng.next_u64() % 14;
            match selector {
                0 => {
                    let amount = rng.bounded(1_000) + 1;
                    custody.record_explicit_sol(amount, amount).unwrap_or_else(
                        |error| {
                            panic!(
                                "seed={RANDOM_SEED:#018x} case={case_index} action={action_index} selector={selector} phase={phase} error={error:?}"
                            )
                        },
                    );
                    physical_sol += amount;
                    accounted_sol += amount;
                }
                1 => {
                    let amount = rng.bounded(1_000) + 1;
                    custody
                        .record_explicit_jitosol(amount, amount)
                        .unwrap_or_else(|error| {
                            panic!(
                                "seed={RANDOM_SEED:#018x} case={case_index} action={action_index} selector={selector} phase={phase} error={error:?}"
                            )
                        });
                    physical_jitosol += amount;
                    accounted_jitosol += amount;
                }
                2 => {
                    let amount = rng.bounded(1_000);
                    custody.direct_credit_sol(amount).unwrap_or_else(|error| {
                        panic!(
                            "seed={RANDOM_SEED:#018x} case={case_index} action={action_index} selector={selector} phase={phase} error={error:?}"
                        )
                    });
                    physical_sol += amount;
                }
                3 => {
                    let amount = rng.bounded(1_000);
                    custody
                        .direct_credit_jitosol(amount)
                        .unwrap_or_else(|error| {
                            panic!(
                                "seed={RANDOM_SEED:#018x} case={case_index} action={action_index} selector={selector} phase={phase} error={error:?}"
                            )
                        });
                    physical_jitosol += amount;
                }
                4 => {
                    custody.reconcile().unwrap_or_else(|error| {
                        panic!(
                            "seed={RANDOM_SEED:#018x} case={case_index} action={action_index} selector={selector} phase={phase} error={error:?}"
                        )
                    });
                    accounted_sol = physical_sol + round.pending_sol_used_lamports;
                    accounted_jitosol = physical_jitosol;
                }
                5 => {
                    custody.reconcile().unwrap_or_else(|error| {
                        panic!(
                            "seed={RANDOM_SEED:#018x} case={case_index} action={action_index} selector={selector} phase={phase} first={error:?}"
                        )
                    });
                    accounted_sol = physical_sol + round.pending_sol_used_lamports;
                    accounted_jitosol = physical_jitosol;
                    let before_repeat = custody.clone();
                    let repeated = custody.reconcile().unwrap_or_else(|error| {
                        panic!(
                            "seed={RANDOM_SEED:#018x} case={case_index} action={action_index} selector={selector} phase={phase} repeated={error:?}"
                        )
                    });
                    assert!(
                        repeated.is_no_change(),
                        "seed={RANDOM_SEED:#018x} case={case_index} action={action_index} selector={selector} phase={phase}"
                    );
                    assert_eq!(
                        custody, before_repeat,
                        "seed={RANDOM_SEED:#018x} case={case_index} action={action_index} selector={selector} phase={phase}"
                    );
                }
                6 | 7 => {
                    let amount = rng.bounded(1_000) + 1;
                    let before = custody.clone();
                    let result = if selector == 6 {
                        custody.record_explicit_sol(amount + 1, amount)
                    } else {
                        custody.record_explicit_jitosol(amount + 1, amount)
                    };
                    assert_eq!(
                        result,
                        Err(MockContributionError::Accounting(
                            Piv1Error::ContributionObservationMismatch
                        )),
                        "seed={RANDOM_SEED:#018x} case={case_index} action={action_index} selector={selector} phase={phase}"
                    );
                    assert_eq!(
                        custody, before,
                        "seed={RANDOM_SEED:#018x} case={case_index} action={action_index} selector={selector} phase={phase}"
                    );
                }
                8 | 9 => {
                    let point = if selector == 8 {
                        MockContributionFailurePoint::ExplicitSolAfterAccounting
                    } else {
                        MockContributionFailurePoint::ExplicitJitoSolAfterAccounting
                    };
                    custody.set_failure_point(point);
                    let before = custody.clone();
                    let result = if selector == 8 {
                        custody.record_explicit_sol(1, 1)
                    } else {
                        custody.record_explicit_jitosol(1, 1)
                    };
                    assert_eq!(
                        result,
                        Err(MockContributionError::InjectedFailure(point)),
                        "seed={RANDOM_SEED:#018x} case={case_index} action={action_index} selector={selector} phase={phase}"
                    );
                    assert_eq!(
                        custody, before,
                        "seed={RANDOM_SEED:#018x} case={case_index} action={action_index} selector={selector} phase={phase}"
                    );
                    custody.clear_failure_point();
                }
                10 => {
                    let before = custody.clone();
                    assert_eq!(
                        custody.record_explicit_sol(0, 0),
                        Err(MockContributionError::Accounting(
                            Piv1Error::ZeroContribution
                        )),
                        "seed={RANDOM_SEED:#018x} case={case_index} action={action_index} selector={selector} phase={phase}"
                    );
                    assert_eq!(
                        custody, before,
                        "seed={RANDOM_SEED:#018x} case={case_index} action={action_index} selector={selector} phase={phase}"
                    );
                }
                11 => {
                    custody.config.paused = !custody.config.paused;
                }
                12 => {
                    let sol = rng.bounded(100);
                    let jitosol = rng.bounded(100);
                    custody.direct_credit_sol(sol).unwrap_or_else(|error| {
                        panic!(
                            "seed={RANDOM_SEED:#018x} case={case_index} action={action_index} selector={selector} phase={phase} direct-sol={error:?}"
                        )
                    });
                    custody
                        .direct_credit_jitosol(jitosol)
                        .unwrap_or_else(|error| {
                            panic!(
                                "seed={RANDOM_SEED:#018x} case={case_index} action={action_index} selector={selector} phase={phase} direct-jito={error:?}"
                            )
                        });
                    physical_sol += sol;
                    physical_jitosol += jitosol;
                    custody.reconcile().unwrap_or_else(|error| {
                        panic!(
                            "seed={RANDOM_SEED:#018x} case={case_index} action={action_index} selector={selector} phase={phase} reconcile={error:?}"
                        )
                    });
                    accounted_sol = physical_sol + round.pending_sol_used_lamports;
                    accounted_jitosol = physical_jitosol;
                }
                13 => {
                    custody.validate_conservation().unwrap_or_else(|error| {
                        panic!(
                            "seed={RANDOM_SEED:#018x} case={case_index} action={action_index} selector={selector} phase={phase} conservation={error:?}"
                        )
                    });
                }
                _ => unreachable!("bounded selector"),
            }

            assert_eq!(
                custody.spendable_sol_lamports(),
                Ok(physical_sol),
                "seed={RANDOM_SEED:#018x} case={case_index} action={action_index} selector={selector} phase={phase}"
            );
            assert_eq!(
                custody.pending_jitosol_token_units,
                physical_jitosol,
                "seed={RANDOM_SEED:#018x} case={case_index} action={action_index} selector={selector} phase={phase}"
            );
            assert_eq!(
                custody.config.accounted_pending_sol_lamports,
                accounted_sol,
                "seed={RANDOM_SEED:#018x} case={case_index} action={action_index} selector={selector} phase={phase}"
            );
            assert_eq!(
                custody.config.accounted_pending_jitosol_units,
                accounted_jitosol,
                "seed={RANDOM_SEED:#018x} case={case_index} action={action_index} selector={selector} phase={phase}"
            );
            assert_eq!(
                active_bytes(&custody.active_distribution),
                immutable_round_bytes,
                "seed={RANDOM_SEED:#018x} case={case_index} action={action_index} selector={selector} phase={phase}"
            );
            assert_eq!(
                custody.validate_conservation(),
                Ok(()),
                "seed={RANDOM_SEED:#018x} case={case_index} action={action_index} selector={selector} phase={phase}"
            );
        }
    }
}

struct SplitMix64 {
    state: u64,
}

impl SplitMix64 {
    const fn new(seed: u64) -> Self {
        Self { state: seed }
    }

    fn next_u64(&mut self) -> u64 {
        self.state = self.state.wrapping_add(0x9e37_79b9_7f4a_7c15);
        let mut value = self.state;
        value = (value ^ (value >> 30)).wrapping_mul(0xbf58_476d_1ce4_e5b9);
        value = (value ^ (value >> 27)).wrapping_mul(0x94d0_49bb_1331_11eb);
        value ^ (value >> 31)
    }

    fn bounded(&mut self, upper_exclusive: u64) -> u64 {
        self.next_u64() % upper_exclusive
    }
}
