mod support;

use anchor_lang::AnchorSerialize;
use piv1::{
    errors::Piv1Error,
    integrations::{FeeFraction, PoolSnapshot, StakePoolAdapter},
    state::{*, reconciliation::*},
};
use support::vault_custody_model::*;

fn pending(sol: u64, tokens: u64) -> World {
    let mut w = World::empty(3, 42);
    if sol != 0 { w.explicit_sol(sol, sol).unwrap(); }
    if tokens != 0 { w.explicit_tokens(tokens, tokens).unwrap(); }
    w
}

fn moved(w: &World) -> EconomicCustodyObservation {
    let mut after = w.observation();
    after.pending_sol.lamports = after.pending_sol.non_economic_floor_lamports;
    after.pending_jitosol_units = 0;
    after.principal_sol.lamports += w.config.accounted_pending_sol_lamports;
    after.principal_jitosol_units += w.config.accounted_pending_jitosol_units;
    after
}

fn rejected<T: core::fmt::Debug>(w: &mut World, f: impl FnOnce(&mut World) -> Result<T>) {
    let before = w.clone();
    assert!(f(w).is_err());
    assert_eq!(*w, before);
}

fn pure_rejected(w: &World, pool: PoolSnapshot, before: EconomicCustodyObservation,
                 after: EconomicCustodyObservation) -> Piv1Error {
    let mut config = w.config.clone();
    let result = bootstrap_initial_contributions(&mut config, &w.round, pool, before, after);
    assert_eq!(config, w.config);
    result.unwrap_err()
}

fn bootstrap_assert(w: &mut World, expected_value: u64) {
    let sol = w.config.accounted_pending_sol_lamports;
    let tokens = w.config.accounted_pending_jitosol_units;
    let header_bytes = w.round.try_to_vec().unwrap();
    let mut expected = w.clone();
    expected.config.accounted_pending_sol_lamports = 0;
    expected.config.accounted_pending_jitosol_units = 0;
    expected.config.accounted_historical_sol_lamports = sol;
    expected.config.accounted_historical_jitosol_units = tokens;
    expected.config.protected_principal_hwm_lamports = expected_value;
    expected.config.cumulative_contribution_value_lamports = expected_value;
    expected.sol[PENDING] -= sol;
    expected.sol[PRINCIPAL] += sol;
    expected.tokens[PENDING_TOKEN] -= tokens;
    expected.tokens[PRINCIPAL_TOKEN] += tokens;
    assert_eq!(w.bootstrap().unwrap(), InitialContributionBootstrap {
        integrated_sol_lamports: sol, integrated_jitosol_units: tokens,
        contribution_value_lamports: expected_value,
    });
    // Includes every ledger, clock, guardian, pool audit, reserve, token rent,
    // recipient, temporary account and failure control, not just balances.
    assert_eq!(*w, expected);
    assert_eq!(w.round.try_to_vec().unwrap(), header_bytes);
    w.validate().unwrap();
    assert!(w.normalize().unwrap().is_no_change());
    assert!(w.reconcile().unwrap().is_no_change());
    rejected(w, World::bootstrap);
}

#[test]
fn genuine_empty_fixture_has_no_piv_economic_assets_or_history() {
    let mut w = World::empty(6, 71);
    assert_eq!(w.observation().amounts().unwrap(), EconomicVaultAmounts::default());
    assert_eq!(w.config.protected_principal_hwm_lamports, 0);
    assert_eq!(w.config.cumulative_contribution_value_lamports, 0);
    assert_eq!(w.config.cumulative_kif_credited_lamports, 0);
    assert_eq!(w.audit.external_sol, 0);
    assert_eq!(w.audit.external_tokens, 0);
    assert_eq!(w.spendable(OPERATIONS).unwrap(), 100_000);
    assert_eq!(w.config.next_distribution_sequence, 71);
    assert!(w.rewards.iter().all(|r| r.claimable_lamports == 0 && r.cumulative_earned == 0));
    w.validate().unwrap();
    let pool = w.pool.pool_snapshot().unwrap();
    assert_eq!(pure_rejected(&w, pool, w.observation(), moved(&w)), Piv1Error::ZeroContribution);
    rejected(&mut w, World::bootstrap);
    // Genuine contributions cannot be used to invent an opening yield snapshot.
    w.explicit_sol(50, 50).unwrap();
    w.explicit_tokens(100, 100).unwrap();
    rejected(&mut w, |w| w.open(900_000));
    rejected(&mut w, |w| w.integrate(900_000));
    let unchanged = w.clone();
    record_no_yield_evaluation(&w.config, &w.round, 900_000, 0).unwrap();
    assert_eq!(w, unchanged);
    bootstrap_assert(&mut w, 151);
}

#[test]
fn explicit_sol_token_and_mixed_bootstrap_change_only_initial_holdings() {
    for (sol, tokens, value) in [(700, 0, 700), (0, 100, 101), (700, 100, 801),
                                (1, 1, 2), (0, 99, 99)] {
        let mut w = pending(sol, tokens);
        bootstrap_assert(&mut w, value);
        assert_eq!(w.spendable(PRINCIPAL).unwrap(), sol);
        assert_eq!(w.audit.external_sol, u128::from(sol));
        assert_eq!(w.audit.external_tokens, u128::from(tokens));
    }
}

#[test]
fn bootstrap_preserves_arbitrary_next_sequence_anchor_and_guardian_activity() {
    for sequence in [0, 1, 42, u64::MAX] {
        for active_count in [0, 3, 6] {
            let mut w = World::empty(active_count, sequence);
            w.config.kif_anchor_timestamp = -2_592_000;
            w.explicit_tokens(100, 100).unwrap();
            bootstrap_assert(&mut w, 101);
            assert_eq!(w.config.next_distribution_sequence, sequence);
            assert_eq!(w.config.kif_anchor_timestamp, -2_592_000);
            assert_eq!(w.round.last_completed, None);
            assert_eq!(w.config.last_successful_preparation_at, None);
            assert_eq!(w.config.last_valid_insufficient_attempt_at, None);
        }
    }
}

#[test]
fn direct_pending_receipts_require_recognition_before_bootstrap() {
    let mut w = World::empty(0, 9);
    w.direct_sol(PENDING, 401).unwrap();
    w.direct_tokens(PENDING_TOKEN, 100).unwrap();
    rejected(&mut w, World::bootstrap);
    let result = w.reconcile().unwrap();
    assert_eq!(result.newly_accounted_sol_lamports, 401);
    assert_eq!(result.newly_accounted_jitosol_units, 100);
    bootstrap_assert(&mut w, 502);
}

#[test]
fn economic_surplus_needs_a_separate_normalization_boundary() {
    let mut w = pending(13, 5);
    for (vault, value) in [(PENDING, 2), (PRINCIPAL, 3), (ESCROW, 5), (KIF, 7)] {
        w.direct_sol(vault, value).unwrap();
    }
    w.direct_tokens(PENDING_TOKEN, 11).unwrap();
    w.direct_tokens(PRINCIPAL_TOKEN, 13).unwrap();
    // No bootstrap may silently absorb surplus from any economic vault.
    rejected(&mut w, World::bootstrap);
    let result = w.normalize().unwrap();
    assert_eq!(result.newly_accounted_sol_lamports, 17);
    assert_eq!(result.newly_accounted_jitosol_units, 24);
    assert_eq!(w.config.cumulative_contribution_value_lamports, 0);
    assert_eq!(w.config.protected_principal_hwm_lamports, 0);
    bootstrap_assert(&mut w, 59);
}

#[test]
fn appreciation_while_pending_becomes_contribution_principal() {
    let mut w = pending(300, 1_000_000);
    w.pool.increase_exchange_rate(900_000).unwrap();
    w.validate().unwrap();
    bootstrap_assert(&mut w, 1_100_300);
    assert_eq!(w.config.cumulative_gross_yield_lamports, 0);
    assert_eq!(w.config.next_cycle_yield_lamports, 0);
    rejected(&mut w, |w| w.open(900_000));
}

#[test]
fn positive_zero_rounded_tokens_bootstrap_once_and_remain_historical() {
    let mut w = pending(0, 1);
    w.pool.decrease_exchange_rate(10_099_999).unwrap();
    w.validate().unwrap();
    bootstrap_assert(&mut w, 0);
    assert_eq!(w.config.accounted_historical_jitosol_units, 1);
    assert_eq!(w.config.cumulative_contribution_value_lamports, 0);
    w.explicit_tokens(1, 1).unwrap();
    let pool = w.pool.pool_snapshot().unwrap();
    assert_eq!(pure_rejected(&w, pool, w.observation(), moved(&w)),
               Piv1Error::InvalidBootstrapState);
    rejected(&mut w, World::bootstrap);
}

#[test]
fn all_existing_transfer_and_late_commit_failures_roll_back_the_entire_world() {
    for (sol, tokens) in [(700, 0), (0, 100), (700, 100)] {
        for failure in [Failure::AfterDebit, Failure::MissingCredit,
                        Failure::WrongCredit, Failure::BeforeCommit] {
            let mut w = pending(sol, tokens);
            w.failure = Some(failure);
            rejected(&mut w, World::bootstrap);
            w.validate().unwrap();
            w.failure = None;
            bootstrap_assert(&mut w, sol + tokens * 101 / 100);
        }
    }
}

#[test]
fn every_prior_economic_ledger_blocks_initial_bootstrap() {
    let mutations: &[fn(&mut PivConfig)] = &[
        |c| c.protected_principal_hwm_lamports = 1,
        |c| c.accounted_historical_sol_lamports = 1,
        |c| c.accounted_historical_jitosol_units = 1,
        |c| c.next_cycle_yield_lamports = 1,
        |c| c.collective_kif_carry_lamports = 1,
        |c| c.cumulative_contribution_value_lamports = 1,
        |c| c.cumulative_gross_yield_lamports = 1,
        |c| c.cumulative_htfp_paid_lamports = 1,
        |c| c.cumulative_team_owner_paid_lamports = 1,
        |c| { c.kif_claim_liability_lamports = 1; c.cumulative_kif_credited_lamports = 1; },
        |c| { c.cumulative_kif_claimed_lamports = 1; c.cumulative_kif_credited_lamports = 1; },
        |c| c.cumulative_permanent_compound_lamports = 1,
        |c| c.cumulative_retained_dust_lamports = 1,
        |c| c.cumulative_zero_active_kif_compound_lamports = 1,
        |c| c.cumulative_cooldown_yield_recorded_lamports = 1,
        |c| c.last_successful_preparation_at = Some(0),
        |c| c.last_valid_insufficient_attempt_at = Some(0),
    ];
    for mutate in mutations {
        let mut w = pending(700, 100);
        mutate(&mut w.config);
        w.config.validate_initialized().unwrap();
        assert_eq!(w.round.bump, w.config.bumps.active_distribution);
        assert_eq!(pure_rejected(&w, w.pool.pool_snapshot().unwrap(), w.observation(), moved(&w)),
                   Piv1Error::InvalidBootstrapState);
    }
}

#[test]
fn pause_and_malformed_config_header_or_binding_reject_without_mutation() {
    let mut w = pending(700, 100);
    w.config.paused = true;
    assert_eq!(pure_rejected(&w, w.pool.pool_snapshot().unwrap(), w.observation(), moved(&w)),
               Piv1Error::PausedOperation);
    rejected(&mut w, World::bootstrap);
    let mutations: &[fn(&mut World)] = &[
        |w| w.config.is_initialized = false,
        |w| w.config.version = 0,
        |w| w.config.htfp_reserve_bps += 1,
        |w| w.config.configured_slippage_bps = 2,
        |w| w.config.migration_reserve[0] = 1,
        |w| w.config.pending_jito_vault = w.config.principal_jito_vault,
        |w| w.round.is_initialized = false,
        |w| w.round.version = 0,
        |w| w.round.bump ^= 1,
        |w| w.round.active_sequence = 1,
        |w| w.round.recovery_flags = 1,
    ];
    for mutate in mutations {
        let mut w = pending(700, 100);
        mutate(&mut w);
        pure_rejected(&w, w.pool.pool_snapshot().unwrap(), w.observation(), moved(&w));
        rejected(&mut w, World::bootstrap);
    }
}

#[test]
fn actual_active_settled_recovery_and_completed_lifecycles_reject() {
    for phase in [DistributionLifecycle::WithdrawalActive, DistributionLifecycle::EscrowFunded,
                  DistributionLifecycle::Settled, DistributionLifecycle::RecoveryRequired,
                  DistributionLifecycle::Idle] {
        let pending_sol = if phase == DistributionLifecycle::WithdrawalActive { 0 } else { 10_000 };
        let mut w = World::new(pending_sol, 100, 0, 9, 3, FeeFraction::ZERO, 100_000);
        // Match bumps so the bootstrap history/lifecycle gate is exercised.
        w.round.bump = w.config.bumps.active_distribution;
        w.open(900_000).unwrap();
        if phase == DistributionLifecycle::RecoveryRequired {
            w.pool.decrease_exchange_rate(10_099_000).unwrap();
            assert_eq!(w.settle().unwrap(), SettlementOutcome::RecoveryRequired);
        } else if phase == DistributionLifecycle::Settled || phase == DistributionLifecycle::Idle {
            w.settle().unwrap();
            if phase == DistributionLifecycle::Idle { w.integrate(900_100).unwrap(); }
        }
        assert_eq!(w.round.lifecycle, phase);
        w.validate().unwrap();
        rejected(&mut w, World::bootstrap);
        if phase == DistributionLifecycle::Idle {
            // Isolate completed-summary rejection from every economic-history field.
            let mut empty = pending(1, 0);
            empty.round = w.round;
            empty.config.next_distribution_sequence = w.config.next_distribution_sequence;
            validate_custody_state_binding(&empty.config, &empty.round).unwrap();
            assert_eq!(pure_rejected(&empty, empty.pool.pool_snapshot().unwrap(),
                empty.observation(), moved(&empty)), Piv1Error::InvalidBootstrapState);
        }
    }
}

#[test]
fn each_vault_surplus_and_pending_deficit_are_rejected_independently() {
    let w = pending(700, 100);
    let pool = w.pool.pool_snapshot().unwrap();
    let mutations: &[fn(&mut EconomicCustodyObservation)] = &[
        |o| o.pending_sol.lamports += 1,
        |o| o.principal_sol.lamports += 1,
        |o| o.distribution_escrow.lamports += 1,
        |o| o.kif_sol.lamports += 1,
        |o| o.pending_jitosol_units += 1,
        |o| o.principal_jitosol_units += 1,
    ];
    for mutate in mutations {
        let mut before = w.observation();
        mutate(&mut before);
        assert_eq!(pure_rejected(&w, pool, before, moved(&w)), Piv1Error::InvalidCustodyObservation);
    }
    for tokens in [false, true] {
        let mut before = w.observation();
        if tokens { before.pending_jitosol_units -= 1; }
        else { before.pending_sol.lamports -= 1; }
        // Unrelated principal surplus must not mask a pending deficit.
        before.principal_sol.lamports += 1;
        assert_eq!(pure_rejected(&w, pool, before, moved(&w)), Piv1Error::EconomicCustodyDeficit);
    }
}

#[test]
fn exact_matched_movements_and_every_rent_floor_are_required() {
    let w = pending(700, 100);
    let before = w.observation();
    let pool = w.pool.pool_snapshot().unwrap();
    let mutations: &[fn(&mut EconomicCustodyObservation)] = &[
        |o| o.pending_sol.lamports += 1,
        |o| o.principal_sol.lamports += 1,
        |o| o.principal_sol.lamports -= 1,
        |o| o.distribution_escrow.lamports += 1,
        |o| o.kif_sol.lamports += 1,
        |o| o.pending_jitosol_units += 1,
        |o| o.principal_jitosol_units += 1,
        |o| o.principal_jitosol_units -= 1,
        |o| { o.principal_sol.lamports -= 1; o.principal_jitosol_units += 1; },
        |o| { o.principal_sol.lamports -= 1; o.distribution_escrow.lamports += 1; },
        |o| { o.pending_sol.lamports += 1; o.pending_sol.non_economic_floor_lamports += 1; },
        |o| { o.principal_sol.lamports += 1; o.principal_sol.non_economic_floor_lamports += 1; },
        |o| { o.distribution_escrow.lamports += 1; o.distribution_escrow.non_economic_floor_lamports += 1; },
        |o| { o.kif_sol.lamports += 1; o.kif_sol.non_economic_floor_lamports += 1; },
    ];
    for mutate in mutations {
        let mut after = moved(&w);
        mutate(&mut after);
        assert_eq!(pure_rejected(&w, pool, before, after), Piv1Error::ContributionObservationMismatch);
    }
    assert_eq!(pure_rejected(&w, pool, before, before), Piv1Error::ContributionObservationMismatch);
    for index in 0..4 {
        let mut malformed = before;
        let vault = match index { 0 => &mut malformed.pending_sol, 1 => &mut malformed.principal_sol,
            2 => &mut malformed.distribution_escrow, _ => &mut malformed.kif_sol };
        vault.lamports = vault.non_economic_floor_lamports - 1;
        assert_eq!(pure_rejected(&w, pool, malformed, moved(&w)), Piv1Error::InvalidCustodyObservation);
        let mut malformed_after = moved(&w);
        let vault = match index { 0 => &mut malformed_after.pending_sol,
            1 => &mut malformed_after.principal_sol,
            2 => &mut malformed_after.distribution_escrow, _ => &mut malformed_after.kif_sol };
        vault.lamports = vault.non_economic_floor_lamports - 1;
        assert_eq!(pure_rejected(&w, pool, before, malformed_after), Piv1Error::InvalidCustodyObservation);
    }
}

#[test]
fn malformed_stale_and_insufficient_supply_pool_snapshots_reject_even_for_sol_only() {
    let mutations: &[fn(&mut PoolSnapshot)] = &[
        |p| p.last_update_epoch -= 1,
        |p| p.last_update_epoch += 1,
        |p| p.total_pool_lamports = 0,
        |p| p.pool_token_supply = 0,
        |p| p.minimum_delegation_lamports = 0,
        |p| p.available_withdrawal_lamports = p.total_pool_lamports + 1,
        |p| p.sol_deposit_fee = FeeFraction { numerator: 2, denominator: 1 },
        |p| p.stake_withdrawal_fee = FeeFraction { numerator: 1, denominator: 0 },
    ];
    for tokens in [0, 100] {
        let w = pending(700, tokens);
        for mutate in mutations {
            let mut pool = w.pool.pool_snapshot().unwrap();
            mutate(&mut pool);
            assert_eq!(pure_rejected(&w, pool, w.observation(), moved(&w)),
                       Piv1Error::InvalidCustodyObservation);
        }
    }
    let mut w = pending(700, 10_000_001);
    assert_eq!(pure_rejected(&w, w.pool.pool_snapshot().unwrap(), w.observation(), moved(&w)),
               Piv1Error::InvalidCustodyObservation);
    rejected(&mut w, World::bootstrap);
    let mut stale = pending(700, 100);
    stale.pool.advance_epoch_to(41).unwrap();
    rejected(&mut stale, World::bootstrap);
}

#[test]
fn maximum_exact_contribution_and_checked_one_lamport_overflow() {
    // Pure numeric boundary: zero rent floors allow the full u64 native range.
    // No host audit or custody is reset to fabricate these observations.
    let mut w = pending(0, 1);
    let mut pool = w.pool.pool_snapshot().unwrap();
    pool.total_pool_lamports = u64::MAX;
    pool.pool_token_supply = 1;
    let before = w.observation();
    let after = moved(&w);
    assert_eq!(bootstrap_initial_contributions(&mut w.config, &w.round, pool, before, after)
        .unwrap().contribution_value_lamports, u64::MAX);
    assert_eq!(w.config.protected_principal_hwm_lamports, u64::MAX);
    assert_eq!(w.config.cumulative_contribution_value_lamports, u64::MAX);

    let w = pending(1, 1);
    assert_eq!(pure_rejected(&w, pool, w.observation(), moved(&w)), Piv1Error::ArithmeticOverflow);
    let mut w = pending(1, 1);
    w.pool.increase_exchange_rate(u64::MAX - 10_100_000).unwrap();
    // Full supply represents MAX lamports; adding one SOL lamport overflows.
    w.explicit_tokens(9_999_999, 9_999_999).unwrap();
    rejected(&mut w, World::bootstrap);
    w.validate().unwrap();

    let mut w = World::empty(0, u64::MAX);
    w.config.accounted_pending_sol_lamports = u64::MAX;
    let before = EconomicCustodyObservation {
        pending_sol: SolVaultBalance { lamports: u64::MAX, non_economic_floor_lamports: 0 },
        ..EconomicCustodyObservation::default()
    };
    let after = EconomicCustodyObservation {
        principal_sol: before.pending_sol, ..EconomicCustodyObservation::default()
    };
    let pool = w.pool.pool_snapshot().unwrap();
    assert_eq!(bootstrap_initial_contributions(&mut w.config, &w.round, pool, before, after)
        .unwrap().contribution_value_lamports, u64::MAX);
    assert_eq!(w.config.next_distribution_sequence, u64::MAX);
}

#[test]
fn contributed_token_principal_earns_only_later_pool_yield_and_completes_a_distribution() {
    let mut w = pending(0, 1_000_000);
    bootstrap_assert(&mut w, 1_010_000);
    rejected(&mut w, |w| w.open(900_000));
    w.pool.increase_exchange_rate(100_000).unwrap();
    w.open(900_000).unwrap();
    assert_eq!(w.round.active_sequence, 42);
    assert_eq!(w.round.gross_yield_lamports, 10_000);
    assert_eq!(w.round.old_protected_principal_lamports, 1_010_000);
    assert_eq!(w.round.pending_sol_used_lamports, 0);
    let index = w.initiate(1).unwrap();
    assert!(w.round.is_withdrawal_target_assigned());
    w.advance_epoch().unwrap();
    w.finalize(index).unwrap();
    assert_eq!(w.settle().unwrap(), SettlementOutcome::Settled);
    let summary = w.integrate(900_100).unwrap();
    assert_eq!(summary.gross_yield_lamports, 10_000);
    assert_eq!(summary.integrated_contribution_value_lamports, 0);
    assert_eq!(w.config.cumulative_contribution_value_lamports, 1_010_000);
    assert_eq!(w.config.cumulative_gross_yield_lamports, 10_000);
    assert!(w.config.protected_principal_hwm_lamports >= 1_011_950);
    assert!(w.sol[HTFP] > 0 && w.sol[TEAM] > 0);
    w.validate().unwrap();
}

#[test]
fn destination_balance_overflow_discards_all_staged_custody_and_state() {
    let mut w = World::empty(0, 5);
    let amount = u64::MAX - w.floors[PENDING];
    w.explicit_sol(amount, amount).unwrap();
    assert_eq!(w.sol[PENDING], u64::MAX);
    assert!(w.floors[PRINCIPAL] > w.floors[PENDING]);
    rejected(&mut w, World::bootstrap);
    w.validate().unwrap();
}

#[test]
fn bootstrap_charges_no_pool_fee_and_sol_only_accepts_a_valid_empty_pool() {
    let mut w = pending(700, 100);
    w.pool.set_fees(FeeFraction { numerator: 1, denominator: 10 },
                    FeeFraction { numerator: 1, denominator: 20 }).unwrap();
    bootstrap_assert(&mut w, 801);
    // No deposit is executed; this valid pool representation needs no conversion
    // while the bootstrap holds native SOL in PrincipalSolQueue.
    let mut w = pending(700, 0);
    let mut pool = w.pool.pool_snapshot().unwrap();
    pool.total_pool_lamports = 0;
    pool.pool_token_supply = 0;
    pool.available_withdrawal_lamports = 0;
    let before = w.observation();
    let after = moved(&w);
    assert_eq!(bootstrap_initial_contributions(&mut w.config, &w.round, pool, before, after)
        .unwrap().contribution_value_lamports, 700);
    let w = pending(700, 1);
    assert_eq!(pure_rejected(&w, pool, w.observation(), moved(&w)), Piv1Error::InvalidCustodyObservation);
}
