mod support;

use anchor_lang::AnchorSerialize;
use piv1::{
    errors::Piv1Error,
    integrations::*,
    state::{*, reconciliation::*},
};
use support::{
    stake_pool_mock::MockFailurePoint,
    vault_custody_model::*,
};

fn principal(sol: u64, tokens: u64) -> World {
    let mut w = World::empty(3, 42);
    if sol > 0 { w.explicit_sol(sol, sol).unwrap(); }
    if tokens > 0 { w.explicit_tokens(tokens, tokens).unwrap(); }
    w.bootstrap().unwrap();
    w
}

fn rejected<T: core::fmt::Debug>(
    w: &mut World,
    action: impl FnOnce(&mut World) -> Result<T>,
) -> Error {
    let before = w.clone();
    let error = action(w).unwrap_err();
    assert_eq!(*w, before);
    error
}

fn observation(w: &World, amount: u64, minimum: u64) -> PrincipalSolDepositObservation {
    let mut pool = w.pool.clone();
    let pool_before = pool.pool_snapshot().unwrap();
    let request = SolDepositRequest {
        snapshot: pool_before.identity(), native_lamports: amount,
        caller_minimum_pool_tokens_out: minimum,
        slippage_bps: w.config.configured_slippage_bps,
    };
    let execution = pool.execute_protected_sol_deposit(request).unwrap();
    let custody_before = w.observation();
    let mut custody_after = custody_before;
    custody_after.principal_sol.lamports -= amount;
    custody_after.principal_jitosol_units += execution.actual_pool_tokens_out;
    PrincipalSolDepositObservation {
        request, execution, pool_before, pool_after: pool.pool_snapshot().unwrap(),
        custody_before, custody_after,
    }
}

fn pure_rejected(w: &mut World, o: PrincipalSolDepositObservation) -> Piv1Error {
    let before = w.clone();
    let error = record_protected_principal_deposit(&mut w.config, &w.round, o).unwrap_err();
    assert_eq!(*w, before);
    error
}

fn successful(w: &mut World, amount: u64, minimum: u64) -> PrincipalSolDepositRecord {
    let mut expected = w.clone();
    let pool_before = w.pool.raw_snapshot();
    // Independent test arithmetic, not the production boundary's returned value.
    let minted = if pool_before.pool_token_supply == 0 { amount } else {
        (u128::from(amount) * u128::from(pool_before.pool_token_supply)
            / u128::from(pool_before.total_pool_lamports)) as u64
    };
    let before_value = w.config.accounted_historical_sol_lamports
        + (u128::from(w.tokens[PRINCIPAL_TOKEN]) * u128::from(pool_before.total_pool_lamports)
            / u128::from(pool_before.pool_token_supply)) as u64;
    expected.pool.execute_protected_sol_deposit(SolDepositRequest {
        snapshot: pool_before.identity(), native_lamports: amount,
        caller_minimum_pool_tokens_out: minimum,
        slippage_bps: w.config.configured_slippage_bps,
    }).unwrap();
    expected.config.accounted_historical_sol_lamports -= amount;
    expected.config.accounted_historical_jitosol_units += minted;
    expected.sol[PRINCIPAL] -= amount;
    expected.tokens[PRINCIPAL_TOKEN] += minted;
    expected.audit.deposited_native += u128::from(amount);
    expected.audit.minted_user_tokens += u128::from(minted);
    let header = w.round.try_to_vec().unwrap();
    let record = w.deposit_principal_sol(amount, minimum).unwrap();
    let post = w.pool.raw_snapshot();
    let after_value = w.config.accounted_historical_sol_lamports
        + (u128::from(w.tokens[PRINCIPAL_TOKEN]) * u128::from(post.total_pool_lamports)
            / u128::from(post.pool_token_supply)) as u64;
    assert_eq!(record, PrincipalSolDepositRecord {
        deposited_sol_lamports: amount, minted_jitosol_units: minted,
        historical_value_before_lamports: before_value,
        historical_value_after_lamports: after_value,
    });
    assert!(after_value >= before_value);
    assert!(after_value >= w.config.protected_principal_hwm_lamports);
    assert_eq!(*w, expected);
    assert_eq!(w.round.try_to_vec().unwrap(), header);
    w.validate().unwrap();
    record
}

#[test]
fn genuine_sol_contribution_bootstrap_deposit_then_only_later_yield_distributes() {
    let mut w = World::empty(3, 42);
    w.explicit_sol(1_010_000, 1_010_000).unwrap();
    rejected(&mut w, |w| w.deposit_principal_sol(1_010_000, 0));
    w.bootstrap().unwrap();
    assert_eq!(successful(&mut w, 1_010_000, 0).minted_jitosol_units, 1_000_000);
    assert_eq!(w.config.protected_principal_hwm_lamports, 1_010_000);
    assert_eq!(w.config.cumulative_gross_yield_lamports, 0);
    assert_eq!(w.audit.external_sol, 1_010_000);
    assert_eq!(w.audit.external_tokens, 0);
    assert_eq!(w.audit.deposited_native, 1_010_000);
    assert_eq!(w.audit.minted_user_tokens, 1_000_000);
    rejected(&mut w, |w| w.open(900_000));
    w.pool.increase_exchange_rate(110_000).unwrap();
    w.open(900_000).unwrap();
    assert_eq!(w.round.gross_yield_lamports, 10_000);
    assert_eq!(w.round.pending_sol_used_lamports, 0);
    let leg = w.initiate(1).unwrap();
    w.advance_epoch().unwrap();
    w.finalize(leg).unwrap();
    assert_eq!(w.settle().unwrap(), SettlementOutcome::Settled);
    w.integrate(900_100).unwrap();
    assert_eq!(w.config.cumulative_contribution_value_lamports, 1_010_000);
    assert_eq!(w.config.cumulative_gross_yield_lamports, 10_000);
    assert!(w.sol[HTFP] > 0 && w.sol[TEAM] > 0);
    assert!(w.config.protected_principal_hwm_lamports >= 1_011_950);
    assert_eq!(w.audit.external_tokens, 0);
    assert!(w.audit.burned_tokens > 0 && w.audit.recovered_rent > 0);
    w.validate().unwrap();
}

#[test]
fn partial_and_repeated_exact_conversions_preserve_pending_and_full_world() {
    let mut w = principal(1_414, 100);
    w.explicit_sol(29, 29).unwrap();
    w.explicit_tokens(37, 37).unwrap();
    for (amount, minted) in [(101, 100), (202, 200), (404, 400), (707, 700)] {
        assert_eq!(successful(&mut w, amount, minted).minted_jitosol_units, minted);
    }
    assert_eq!(w.config.accounted_historical_sol_lamports, 0);
    assert_eq!(w.config.accounted_pending_sol_lamports, 29);
    assert_eq!(w.config.accounted_pending_jitosol_units, 37);
    assert_eq!(w.config.cumulative_contribution_value_lamports, 1_515);
    assert_eq!(w.audit.minted_user_tokens, 1_400);
    assert_eq!(w.audit.external_tokens, 137);
    rejected(&mut w, |w| w.deposit_principal_sol(1, 0));
}

#[test]
fn completed_history_kif_liabilities_and_operations_survive_later_deposit() {
    let mut w = World::new(10_000, 50, 111, 9, 3, FeeFraction::ZERO, 100_000);
    // Existing funded fixture behavior is unchanged; this boundary binds its bump.
    w.round.bump = w.config.bumps.active_distribution;
    w.open(900_000).unwrap();
    assert_eq!(w.settle().unwrap(), SettlementOutcome::Settled);
    w.integrate(900_100).unwrap();
    w.explicit_sol(23, 23).unwrap();
    w.explicit_tokens(11, 11).unwrap();
    w.direct_sol(OPERATIONS, 31).unwrap();
    assert!(w.round.last_completed.is_some());
    assert!(w.config.cumulative_gross_yield_lamports > 0);
    assert!(w.config.kif_claim_liability_lamports > 90);
    assert!(w.config.cumulative_contribution_value_lamports > 0);
    successful(&mut w, 101, 100);
    assert!(w.normalize().unwrap().is_no_change());
}

#[test]
fn zero_fee_one_lamport_loss_rejects_while_exact_707_succeeds() {
    let mut lossy = principal(700, 0);
    let o = observation(&lossy, 700, 0);
    assert_eq!(o.execution.actual_pool_tokens_out, 693);
    assert_eq!(o.execution.quote.derived_slippage_floor_pool_tokens, 692);
    assert_eq!(observed_token_book_value(693, o.pool_after).unwrap(), 699);
    assert_eq!(pure_rejected(&mut lossy, o), Piv1Error::PrincipalDepositHistoricalValueLoss);
    assert_eq!(rejected(&mut lossy, |w| w.deposit_principal_sol(700, 0)),
        Error::State(Piv1Error::PrincipalDepositHistoricalValueLoss));
    assert_eq!(lossy.config.accounted_historical_sol_lamports, 700);
    let mut exact = principal(707, 0);
    assert_eq!(successful(&mut exact, 707, 0).minted_jitosol_units, 700);
}

#[test]
fn rounding_loss_cannot_consume_unallocated_historical_yield_above_hwm() {
    let mut w = principal(700, 1_000_000);
    w.pool.increase_exchange_rate(100_000).unwrap();
    let o = observation(&w, 700, 0);
    let before = 700 + observed_token_book_value(1_000_000, o.pool_before).unwrap();
    let after = observed_token_book_value(o.custody_after.principal_jitosol_units,
                                          o.pool_after).unwrap();
    assert!(after >= w.config.protected_principal_hwm_lamports);
    assert!(after < before);
    assert_eq!(pure_rejected(&mut w, o), Piv1Error::PrincipalDepositHistoricalValueLoss);
    rejected(&mut w, |w| w.deposit_principal_sol(700, 0));
}

#[test]
fn actual_deposit_fees_reject_even_with_a_large_existing_yield_buffer() {
    for buffer in [false, true] {
        let mut w = principal(1_010, if buffer { 1_000_000 } else { 0 });
        if buffer { w.pool.increase_exchange_rate(900_000).unwrap(); }
        w.pool.set_fees(FeeFraction { numerator: 1, denominator: 100 },
                        FeeFraction::ZERO).unwrap();
        let o = observation(&w, 1_010, 0);
        assert!(o.execution.actual_fee_pool_tokens > 0);
        if buffer {
            assert!(observed_token_book_value(o.custody_after.principal_jitosol_units,
                                              o.pool_after).unwrap()
                > w.config.protected_principal_hwm_lamports);
        } else {
            assert_eq!(o.execution.actual_pool_tokens_out, 990);
            assert_eq!(observed_token_book_value(990, o.pool_after).unwrap(), 999);
        }
        assert_eq!(pure_rejected(&mut w, o), Piv1Error::UnsupportedPrincipalDepositFee);
        assert_eq!(rejected(&mut w, |w| w.deposit_principal_sol(1_010, 0)),
            Error::State(Piv1Error::UnsupportedPrincipalDepositFee));
        w.validate().unwrap();
    }
}

#[test]
fn configured_zero_or_one_bps_floor_and_stronger_caller_minimum_are_enforced() {
    for bps in [0, 1] {
        for minimum in [0, 699, 700] {
            let mut w = principal(707, 0);
            w.config.configured_slippage_bps = bps;
            let o = observation(&w, 707, minimum);
            assert_eq!(o.execution.quote.derived_slippage_floor_pool_tokens,
                       if bps == 0 { 700 } else { 699 });
            successful(&mut w, 707, minimum);
        }
        let mut w = principal(707, 0);
        w.config.configured_slippage_bps = bps;
        let mut o = observation(&w, 707, 0);
        o.request.caller_minimum_pool_tokens_out = 701;
        o.execution.quote.minimum_pool_tokens_out = 701;
        assert_eq!(pure_rejected(&mut w, o), Piv1Error::PrincipalDepositMinimumNotMet);
        assert_eq!(rejected(&mut w, |w| w.deposit_principal_sol(707, 701)),
                   Error::Pool(StakePoolError::SlippageExceeded));
        let mut o = observation(&w, 707, 0);
        o.request.slippage_bps = 1 - bps;
        assert_eq!(pure_rejected(&mut w, o), Piv1Error::InvalidSlippage);
    }
}

#[test]
fn every_receipt_and_request_field_is_independently_bound() {
    let mut w = principal(707, 100);
    let base = observation(&w, 707, 0);
    let mutations: &[fn(&mut PrincipalSolDepositObservation)] = &[
        |o| o.request.snapshot.revision += 1,
        |o| o.request.snapshot.current_epoch += 1,
        |o| o.request.snapshot.last_update_epoch -= 1,
        |o| o.request.native_lamports -= 1,
        |o| o.execution.quote.snapshot.revision += 1,
        |o| o.execution.quote.snapshot.current_epoch += 1,
        |o| o.execution.quote.snapshot.last_update_epoch -= 1,
        |o| o.execution.quote.native_lamports -= 1,
        |o| o.execution.quote.gross_pool_tokens += 1,
        |o| o.execution.quote.deposit_fee_pool_tokens = 1,
        |o| o.execution.quote.quoted_pool_tokens_out += 1,
        |o| o.execution.quote.derived_slippage_floor_pool_tokens -= 1,
        |o| o.execution.quote.minimum_pool_tokens_out -= 1,
        |o| o.execution.actual_pool_tokens_out -= 1,
        |o| o.execution.actual_fee_pool_tokens = 1,
        // Matching a fabricated mint across receipt/custody/pool cannot defeat arithmetic.
        |o| {
            o.execution.actual_pool_tokens_out += 1;
            o.execution.quote.gross_pool_tokens += 1;
            o.execution.quote.quoted_pool_tokens_out += 1;
            o.custody_after.principal_jitosol_units += 1;
            o.pool_after.pool_token_supply += 1;
        },
    ];
    for mutate in mutations {
        let mut o = base;
        mutate(&mut o);
        pure_rejected(&mut w, o);
    }
}

#[test]
fn stale_malformed_changed_fee_epoch_and_inexact_pool_deltas_reject() {
    let mut w = principal(707, 100);
    let base = observation(&w, 707, 0);
    let mutations: &[fn(&mut PoolSnapshot)] = &[
        |p| p.last_update_epoch -= 1,
        |p| p.last_update_epoch += 1,
        |p| p.total_pool_lamports = 0,
        |p| p.pool_token_supply = 0,
        |p| p.minimum_delegation_lamports = 0,
        |p| p.available_withdrawal_lamports = p.total_pool_lamports + 1,
        |p| p.sol_deposit_fee = FeeFraction { numerator: 0, denominator: 2 },
        |p| p.sol_deposit_fee = FeeFraction { numerator: 2, denominator: 1 },
        |p| p.stake_withdrawal_fee = FeeFraction { numerator: 1, denominator: 0 },
    ];
    for post in [false, true] {
        for mutate in mutations {
            let mut o = base;
            mutate(if post { &mut o.pool_after } else { &mut o.pool_before });
            pure_rejected(&mut w, o);
        }
    }
    let mutations: &[fn(&mut PoolSnapshot)] = &[
        |p| { p.current_epoch += 1; p.last_update_epoch += 1; },
        |p| p.sol_deposit_fee = FeeFraction { numerator: 1, denominator: 100 },
        |p| p.stake_withdrawal_fee = FeeFraction { numerator: 1, denominator: 100 },
        |p| p.minimum_delegation_lamports += 1,
        |p| p.total_pool_lamports += 1,
        |p| p.total_pool_lamports -= 1,
        |p| p.pool_token_supply += 1,
        |p| p.pool_token_supply -= 1,
    ];
    for mutate in mutations {
        let mut o = base;
        mutate(&mut o.pool_after);
        assert_eq!(pure_rejected(&mut w, o), Piv1Error::InvalidPrincipalDepositPool);
    }
    w.pool.advance_epoch_to(41).unwrap();
    rejected(&mut w, |w| w.deposit_principal_sol(707, 0));
    w.pool.refresh_pool().unwrap();
    successful(&mut w, 707, 0);
}

#[test]
fn pure_boundary_does_not_invent_revision_capacity_or_liquidity_update_rules() {
    let w = principal(707, 100);
    for revision in [0, 9, u64::MAX] {
        let mut o = observation(&w, 707, 0);
        o.pool_before.maximum_deposit_lamports = 0;
        o.pool_after.maximum_deposit_lamports = u64::MAX;
        o.pool_after.available_withdrawal_lamports = 0;
        o.pool_after.revision = revision;
        let mut config = w.config.clone();
        record_protected_principal_deposit(&mut config, &w.round, o).unwrap();
        assert_eq!(config.accounted_historical_sol_lamports, 0);
    }
    // The unchanged host adapter still enforces its own artificial capacity.
    let mut w = w;
    w.pool.set_maximum_deposit_lamports(706).unwrap();
    assert_eq!(rejected(&mut w, |w| w.deposit_principal_sol(707, 0)),
        Error::Pool(StakePoolError::InsufficientPoolLiquidity));
    w.pool.set_maximum_deposit_lamports(707).unwrap();
    successful(&mut w, 707, 0);
}

#[test]
fn each_individual_vault_surplus_deficit_and_rent_floor_change_rejects() {
    let mut w = principal(1_414, 100);
    w.explicit_sol(13, 13).unwrap();
    w.explicit_tokens(11, 11).unwrap();
    let base = observation(&w, 707, 0);
    let surplus: &[fn(&mut EconomicCustodyObservation)] = &[
        |o| o.pending_sol.lamports += 1,
        |o| o.principal_sol.lamports += 1,
        |o| o.distribution_escrow.lamports += 1,
        |o| o.kif_sol.lamports += 1,
        |o| o.pending_jitosol_units += 1,
        |o| o.principal_jitosol_units += 1,
    ];
    for post in [false, true] {
        for mutate in surplus {
            let mut o = base;
            mutate(if post { &mut o.custody_after } else { &mut o.custody_before });
            pure_rejected(&mut w, o);
        }
    }
    let deficit: &[fn(&mut EconomicCustodyObservation)] = &[
        |o| o.pending_sol.lamports -= 1,
        |o| o.principal_sol.lamports -= 1,
        |o| o.pending_jitosol_units -= 1,
        |o| o.principal_jitosol_units -= 1,
    ];
    for mutate in deficit {
        let mut o = base;
        mutate(&mut o.custody_before);
        o.custody_before.kif_sol.lamports += 1;
        assert_eq!(pure_rejected(&mut w, o), Piv1Error::EconomicCustodyDeficit);
    }
    for post in [false, true] {
        for index in 0..4 {
            let mut o = base;
            let c = if post { &mut o.custody_after } else { &mut o.custody_before };
            let vault = match index { 0 => &mut c.pending_sol, 1 => &mut c.principal_sol,
                2 => &mut c.distribution_escrow, _ => &mut c.kif_sol };
            vault.lamports = vault.non_economic_floor_lamports - 1;
            pure_rejected(&mut w, o);
            let mut o = base;
            let vault = match index { 0 => &mut o.custody_after.pending_sol,
                1 => &mut o.custody_after.principal_sol,
                2 => &mut o.custody_after.distribution_escrow, _ => &mut o.custody_after.kif_sol };
            vault.lamports += 1;
            vault.non_economic_floor_lamports += 1;
            assert_eq!(pure_rejected(&mut w, o), Piv1Error::PrincipalDepositObservationMismatch);
        }
    }
    for vault in [PENDING, PRINCIPAL, ESCROW, KIF] {
        let mut extra = w.clone();
        extra.direct_sol(vault, 1).unwrap();
        rejected(&mut extra, |w| w.deposit_principal_sol(707, 0));
    }
    for vault in [PENDING_TOKEN, PRINCIPAL_TOKEN] {
        let mut extra = w.clone();
        extra.direct_tokens(vault, 1).unwrap();
        rejected(&mut extra, |w| w.deposit_principal_sol(707, 0));
    }
}

#[test]
fn missing_or_wrong_native_token_custody_deltas_never_match_a_valid_receipt() {
    let mut w = principal(1_414, 100);
    let base = observation(&w, 707, 0);
    let mutations: &[fn(&mut EconomicCustodyObservation)] = &[
        |c| c.principal_sol.lamports -= 1,
        |c| c.principal_jitosol_units -= 1,
        |c| { c.principal_sol.lamports += 707; c.pending_sol.lamports -= 1; },
        |c| { c.principal_sol.lamports -= 1; c.pending_sol.lamports += 1; },
        |c| { c.principal_jitosol_units -= 700; c.pending_jitosol_units += 700; },
    ];
    for mutate in mutations {
        let mut o = base;
        mutate(&mut o.custody_after);
        pure_rejected(&mut w, o);
    }
    let mut o = base;
    o.custody_after = o.custody_before;
    assert_eq!(pure_rejected(&mut w, o), Piv1Error::PrincipalDepositObservationMismatch);
}

#[test]
fn all_host_and_deposit_adapter_failure_points_roll_back_full_world_and_retry() {
    for failure in [Failure::AfterDebit, Failure::MissingCredit,
                    Failure::WrongCredit, Failure::BeforeCommit] {
        let mut w = principal(707, 100);
        w.failure = Some(failure);
        rejected(&mut w, |w| w.deposit_principal_sol(707, 0));
        w.validate().unwrap();
        w.failure = None;
        successful(&mut w, 707, 0);
    }
    for failure in [MockFailurePoint::SnapshotRead, MockFailurePoint::DepositBeforeValidation,
                    MockFailurePoint::DepositAfterQuote, MockFailurePoint::DepositBeforeCommit] {
        let mut w = principal(707, 100);
        w.pool.set_failure(failure);
        assert_eq!(rejected(&mut w, |w| w.deposit_principal_sol(707, 0)),
                   Error::Pool(StakePoolError::InjectedMockFailure));
        w.validate().unwrap();
        w.pool.clear_failure();
        successful(&mut w, 707, 0);
    }
}

#[test]
fn pause_zero_excess_queue_and_malformed_config_header_are_atomic() {
    let mut w = principal(707, 100);
    let base = observation(&w, 707, 0);
    w.config.paused = true;
    assert_eq!(pure_rejected(&mut w, base), Piv1Error::PausedOperation);
    rejected(&mut w, |w| w.deposit_principal_sol(707, 0));
    w.config.paused = false;
    let mut zero = base;
    zero.request.native_lamports = 0;
    assert_eq!(pure_rejected(&mut w, zero), Piv1Error::ZeroPrincipalDeposit);
    rejected(&mut w, |w| w.deposit_principal_sol(0, 0));
    let mut excess = base;
    excess.request.native_lamports = 708;
    assert_eq!(pure_rejected(&mut w, excess), Piv1Error::PrincipalDepositExceedsQueue);
    rejected(&mut w, |w| w.deposit_principal_sol(708, 0));
    let mutations: &[fn(&mut World)] = &[
        |w| w.config.is_initialized = false,
        |w| w.config.version = 0,
        |w| w.config.configured_slippage_bps = 2,
        |w| w.config.htfp_reserve_bps += 1,
        |w| w.config.migration_reserve[0] = 1,
        |w| w.config.pending_jito_vault = w.config.principal_jito_vault,
        |w| w.round.is_initialized = false,
        |w| w.round.version = 0,
        |w| w.round.bump ^= 1,
        |w| w.round.active_sequence = 1,
        |w| w.round.recovery_flags = 1,
    ];
    for mutate in mutations {
        let mut invalid = w.clone();
        mutate(&mut invalid);
        pure_rejected(&mut invalid, base);
        rejected(&mut invalid, |w| w.deposit_principal_sol(707, 0));
    }
}

#[test]
fn actual_withdrawal_funded_settled_and_recovery_phases_reject() {
    for phase in [DistributionLifecycle::WithdrawalActive, DistributionLifecycle::EscrowFunded,
                  DistributionLifecycle::Settled, DistributionLifecycle::RecoveryRequired] {
        let mut w = World::new(if phase == DistributionLifecycle::WithdrawalActive { 0 } else { 10_000 },
                              100, 0, 9, 3, FeeFraction::ZERO, 100_000);
        w.round.bump = w.config.bumps.active_distribution;
        w.open(900_000).unwrap();
        if phase == DistributionLifecycle::RecoveryRequired {
            w.pool.decrease_exchange_rate(10_099_000).unwrap();
            assert_eq!(w.settle().unwrap(), SettlementOutcome::RecoveryRequired);
        } else if phase == DistributionLifecycle::Settled {
            w.settle().unwrap();
        }
        assert_eq!(w.round.lifecycle, phase);
        w.validate().unwrap();
        let o = observation(&principal(707, 100), 707, 0);
        assert_eq!(pure_rejected(&mut w, o), Piv1Error::InvalidLifecycle);
        rejected(&mut w, |w| w.deposit_principal_sol(1, 0));
    }
}

#[test]
fn combined_principal_and_pending_supply_rejects_globally_impossible_holdings() {
    let mut w = principal(707, 100);
    // Pure malformed observations: each holding is <= supply, their sum is not.
    let mut o = observation(&w, 707, 0);
    w.config.accounted_pending_jitosol_units = 10_000_000;
    o.custody_before.pending_jitosol_units = 10_000_000;
    o.custody_after.pending_jitosol_units = 10_000_000;
    assert_eq!(pure_rejected(&mut w, o), Piv1Error::InvalidPrincipalDepositPool);
    w.config.accounted_pending_jitosol_units = u64::MAX;
    o.custody_before.pending_jitosol_units = u64::MAX;
    o.custody_after.pending_jitosol_units = u64::MAX;
    assert_eq!(pure_rejected(&mut w, o), Piv1Error::ArithmeticOverflow);
}

#[test]
fn pure_carry_is_preserved_but_cannot_hide_historical_hwm_shortfall_or_be_spent() {
    // Pure supplied-state/observation evidence only; no host custody or audit reset.
    let mut w = principal(707, 100);
    let mut o = observation(&w, 707, 0);
    w.config.next_cycle_yield_lamports = 50;
    o.custody_before.principal_sol.lamports += 50;
    o.custody_after.principal_sol.lamports += 50;
    let mut config = w.config.clone();
    let before = config.clone();
    let record = record_protected_principal_deposit(&mut config, &w.round, o).unwrap();
    let mut expected = before;
    expected.accounted_historical_sol_lamports = 0;
    expected.accounted_historical_jitosol_units = 800;
    assert_eq!(config, expected);
    assert_eq!(record.historical_value_before_lamports, 808);
    assert_eq!(record.historical_value_after_lamports, 808);
    w.config.protected_principal_hwm_lamports = 809;
    assert_eq!(pure_rejected(&mut w, o), Piv1Error::HighWaterMarkDecrease);
    o.request.native_lamports = 708;
    assert_eq!(pure_rejected(&mut w, o), Piv1Error::PrincipalDepositExceedsQueue);
}

#[test]
fn independent_audit_detects_deposit_counter_tampering_and_false_contribution_classification() {
    let mut w = principal(707, 100);
    successful(&mut w, 707, 0);
    let mutations: &[fn(&mut World)] = &[
        |w| w.audit.deposited_native += 1,
        |w| w.audit.minted_user_tokens += 1,
        |w| { w.audit.minted_user_tokens -= 700; w.audit.external_tokens += 700; },
        |w| w.tokens[PRINCIPAL_TOKEN] += 1,
        |w| w.sol[PRINCIPAL] += 1,
        |w| w.audit.burned_tokens += 1,
        |w| w.audit.fee_tokens += 1,
        |w| w.audit.advanced_rent += 1,
        |w| w.audit.recovered_rent += 1,
        |w| w.token_rent[0] += 1,
    ];
    for mutate in mutations {
        let mut invalid = w.clone();
        mutate(&mut invalid);
        assert!(invalid.validate().is_err());
    }
}


#[test]
fn later_deposit_preserves_completed_withdrawal_fee_burn_rent_and_cooldown_carry() {
    let mut w = principal(1_010_000, 0);
    successful(&mut w, 1_010_000, 0);
    w.pool.set_fees(FeeFraction::ZERO, FeeFraction { numerator: 1, denominator: 100 }).unwrap();
    w.pool.set_source_finalization_terms(WithdrawalSourceId(1), 1, 20, 10, 13, 0).unwrap();
    w.pool.increase_exchange_rate(110_000).unwrap();
    w.open(900_000).unwrap();
    // Contributions arriving during the withdrawal remain pending until integration.
    w.explicit_sol(1_000, 1_000).unwrap();
    let leg = w.initiate(1).unwrap();
    w.advance_epoch().unwrap();
    w.finalize(leg).unwrap();
    assert_eq!(w.settle().unwrap(), SettlementOutcome::Settled);
    w.integrate(900_100).unwrap();
    assert_eq!(w.config.next_cycle_yield_lamports, 13);
    assert_eq!(w.audit.cooldown_reward, 13);
    assert_eq!(w.audit.advanced_rent, 30);
    assert_eq!(w.audit.recovered_rent, 30);
    assert!(w.audit.burned_tokens > 0 && w.audit.fee_tokens > 0);
    let previous_audit = w.audit.clone();
    let record = successful(&mut w, 101, 0);
    assert_eq!(record.minted_jitosol_units, 99);
    assert_eq!(w.config.next_cycle_yield_lamports, 13);
    assert_eq!(w.audit.external_sol, 1_011_000);
    assert_eq!(w.audit.external_tokens, 0);
    let mut expected_audit = previous_audit;
    expected_audit.deposited_native += 101;
    expected_audit.minted_user_tokens += 99;
    assert_eq!(w.audit, expected_audit);
    w.validate().unwrap();
}

#[test]
fn compensated_host_deficits_cannot_spend_an_unrelated_vault_or_operational_rent() {
    let mut w = World::new(10_000, 50, 0, 9, 3, FeeFraction::ZERO, 100_000);
    w.round.bump = w.config.bumps.active_distribution;
    w.open(900_000).unwrap();
    w.settle().unwrap();
    w.integrate(900_100).unwrap();
    w.explicit_sol(13, 13).unwrap();
    w.explicit_tokens(11, 11).unwrap();
    for vault in [PENDING, PRINCIPAL, KIF, OPERATIONS] {
        let mut invalid = w.clone();
        invalid.sol[vault] -= 1;
        invalid.sol[ESCROW] += 1;
        assert!(invalid.validate().is_err());
        rejected(&mut invalid, |w| w.deposit_principal_sol(101, 0));
        // Pure economic observations also reject each covered non-operational deficit.
        if vault != OPERATIONS {
            let mut o = observation(&w, 101, 0);
            o.custody_before = invalid.observation();
            assert_eq!(pure_rejected(&mut w, o), Piv1Error::EconomicCustodyDeficit);
        }
    }
    for vault in [PENDING_TOKEN, PRINCIPAL_TOKEN] {
        let mut invalid = w.clone();
        invalid.tokens[vault] -= 1;
        invalid.tokens[1 - vault] += 1;
        rejected(&mut invalid, |w| w.deposit_principal_sol(101, 0));
    }
    successful(&mut w, 101, 0);
}

// These synthetic boundaries exercise checked pure arithmetic without claiming
// physically possible host custody, pool history or funded rent at u64 maxima.
fn numeric(
    historical_sol: u64,
    historical_tokens: u64,
    total: u64,
    supply: u64,
    amount: u64,
    minted: u64,
    total_after: u64,
    supply_after: u64,
) -> (World, PrincipalSolDepositObservation) {
    let mut w = principal(707, 0);
    w.config.accounted_historical_sol_lamports = historical_sol;
    w.config.accounted_historical_jitosol_units = historical_tokens;
    w.config.protected_principal_hwm_lamports = 0;
    let mut pool_before = w.pool.raw_snapshot();
    pool_before.total_pool_lamports = total;
    pool_before.pool_token_supply = supply;
    pool_before.available_withdrawal_lamports = 0;
    let pool_after = PoolSnapshot { total_pool_lamports: total_after,
        pool_token_supply: supply_after, ..pool_before };
    let custody_before = EconomicCustodyObservation {
        principal_sol: SolVaultBalance { lamports: historical_sol, non_economic_floor_lamports: 0 },
        principal_jitosol_units: historical_tokens,
        ..EconomicCustodyObservation::default()
    };
    let custody_after = EconomicCustodyObservation {
        principal_sol: SolVaultBalance { lamports: historical_sol.saturating_sub(amount),
                                        non_economic_floor_lamports: 0 },
        principal_jitosol_units: historical_tokens.saturating_add(minted),
        ..custody_before
    };
    let floor = (u128::from(minted) * 9_999 / 10_000) as u64;
    let request = SolDepositRequest { snapshot: pool_before.identity(), native_lamports: amount,
        caller_minimum_pool_tokens_out: 0, slippage_bps: 1 };
    let execution = SolDepositExecution {
        quote: SolDepositQuote { snapshot: request.snapshot, native_lamports: amount,
            gross_pool_tokens: minted, deposit_fee_pool_tokens: 0, quoted_pool_tokens_out: minted,
            derived_slippage_floor_pool_tokens: floor, minimum_pool_tokens_out: floor },
        actual_pool_tokens_out: minted, actual_fee_pool_tokens: 0,
    };
    (w, PrincipalSolDepositObservation {
        request, execution, pool_before, pool_after, custody_before, custody_after,
    })
}

#[test]
fn pure_empty_pool_exact_maximum_and_checked_overflows() {
    let (mut w, o) = numeric(u64::MAX, 0, 0, 0, u64::MAX, u64::MAX, u64::MAX, u64::MAX);
    let record = record_protected_principal_deposit(&mut w.config, &w.round, o).unwrap();
    assert_eq!(record.minted_jitosol_units, u64::MAX);
    assert_eq!(record.historical_value_before_lamports, u64::MAX);
    assert_eq!(record.historical_value_after_lamports, u64::MAX);
    let cases = [
        numeric(1, 0, u64::MAX, u64::MAX, 1, 1, u64::MAX, u64::MAX),
        numeric(1, 0, 1, u64::MAX, 1, u64::MAX, 2, u64::MAX),
        numeric(2, 0, 1, u64::MAX, 2, u64::MAX, 3, u64::MAX),
        numeric(u64::MAX / 2 + 2, 2, u64::MAX / 2, 2,
                u64::MAX / 2, 2, u64::MAX - 1, 4),
    ];
    for (mut w, o) in cases {
        assert_eq!(pure_rejected(&mut w, o), Piv1Error::ArithmeticOverflow);
    }
    let (mut w, o) = numeric(1, 0, 10_000_000, 1, 1, 0, 10_000_001, 1);
    assert_eq!(pure_rejected(&mut w, o), Piv1Error::PrincipalDepositMinimumNotMet);
    let (mut w, o) = numeric(707, 1, 0, 0, 707, 707, 707, 707);
    assert_eq!(pure_rejected(&mut w, o), Piv1Error::InvalidPrincipalDepositPool);
}

#[test]
fn real_host_pool_overflow_discards_staged_native_debit() {
    let mut w = principal(10_000_000_000_000, 0);
    w.pool.set_maximum_deposit_lamports(10_000_000_000_000).unwrap();
    w.pool.increase_exchange_rate(u64::MAX - 10_100_000).unwrap();
    w.validate().unwrap();
    assert_eq!(rejected(&mut w, |w| w.deposit_principal_sol(10_000_000_000_000, 0)),
               Error::Pool(StakePoolError::ArithmeticOverflow));
    w.validate().unwrap();
}
