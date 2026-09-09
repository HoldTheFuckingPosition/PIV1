mod support;

use piv1::{
    constants::RECOVERY_FLAG_RESIDUAL_HWM,
    errors::Piv1Error,
    integrations::{FeeFraction, WithdrawalSourceId},
    state::{*, reconciliation::*},
};
use support::vault_custody_model::*;

const SEED: u64 = 0x5049_5631_5641_554c;
const CASES: usize = 128;

fn world(pending: u64) -> World {
    World::new(pending, 50, 0, 9, 3, FeeFraction::ZERO, 100_000)
}

fn rejected<T: core::fmt::Debug>(w: &mut World, f: impl FnOnce(&mut World) -> Result<T>) {
    let before = w.clone();
    assert!(f(w).is_err());
    assert_eq!(*w, before);
}

fn finish_legs(w: &mut World, reverse: bool) {
    for source in 1..=8 {
        if w.round.is_withdrawal_target_assigned()
            || w.round.lifecycle != DistributionLifecycle::WithdrawalActive { break; }
        w.initiate(source).unwrap();
    }
    if w.round.fixed_jitosol_withdrawal_target_units > 0 {
        assert!(w.round.is_withdrawal_target_assigned());
        w.advance_epoch().unwrap();
        let count = w.round.successful_leg_count as usize;
        for i in 0..count {
            let index = if reverse { count - 1 - i } else { i };
            if w.legs[index].status == WithdrawalLegStatus::Initiated {
                w.finalize(index).unwrap();
            }
        }
    }
    assert_eq!(w.round.lifecycle, DistributionLifecycle::EscrowFunded);
}

fn integrate_assert(w: &mut World, now: i64) -> u64 {
    let p = w.config.accounted_pending_sol_lamports;
    let q = w.config.accounted_pending_jitosol_units;
    let pool = w.pool.raw_snapshot();
    let contribution = p + (u128::from(q) * u128::from(pool.total_pool_lamports)
        / u128::from(pool.pool_token_supply)) as u64;
    let hwm = w.config.protected_principal_hwm_lamports;
    let old_cumulative = w.config.cumulative_contribution_value_lamports;
    let physical_pending = w.spendable(PENDING).unwrap();
    let principal_before = w.spendable(PRINCIPAL).unwrap();
    let escrow_before = w.spendable(ESCROW).unwrap();
    let tokens_before = w.tokens[PRINCIPAL_TOKEN];
    let summary = w.integrate(now).unwrap();
    assert_eq!(summary.integrated_contribution_value_lamports, contribution);
    assert_eq!(w.config.protected_principal_hwm_lamports, hwm + contribution);
    assert_eq!(w.config.cumulative_contribution_value_lamports, old_cumulative + contribution);
    assert_eq!(w.spendable(PRINCIPAL).unwrap(), principal_before + physical_pending + escrow_before);
    assert_eq!(w.tokens[PRINCIPAL_TOKEN], tokens_before + q);
    assert_eq!(w.spendable(PENDING).unwrap(), 0);
    assert_eq!(w.tokens[PENDING_TOKEN], 0);
    assert_eq!(w.spendable(ESCROW).unwrap(), 0);
    assert_eq!(w.round.lifecycle, DistributionLifecycle::Idle);
    assert!(w.normalize().unwrap().is_no_change());
    assert!(w.reconcile().unwrap().is_no_change());
    rejected(w, |w| w.integrate(now));
    contribution
}

#[test]
fn pending_sol_value_survives_physical_funding_settlement_integration_and_next_round() {
    for initial in [0, 4_000, 8_050, 10_000] {
        for explicit in [false, true] {
            let mut w = world(initial);
            // An arrival before preparation is eligible in the actual snapshot.
            if explicit { w.explicit_sol(100, 100).unwrap(); }
            else { w.direct_sol(PENDING, 100).unwrap(); w.reconcile().unwrap(); }
            let snapshot_pending = initial + 100;
            w.open(900_000).unwrap();
            let used = snapshot_pending.min(8_050);
            assert_eq!(w.round.pending_sol_used_lamports, used);
            assert_eq!(w.spendable(PENDING).unwrap(), snapshot_pending - used);
            assert_eq!(w.config.accounted_pending_sol_lamports, snapshot_pending);
            assert!(w.reconcile().unwrap().is_no_change());
            let frozen = w.round;
            if explicit { w.explicit_sol(1_000, 1_000).unwrap(); }
            else { w.direct_sol(PENDING, 1_000).unwrap(); w.reconcile().unwrap(); }
            w.explicit_tokens(7, 7).unwrap();
            w.direct_tokens(PENDING_TOKEN, 3).unwrap();
            w.reconcile().unwrap();
            assert_eq!(w.round, frozen);
            assert_eq!(w.config.accounted_pending_sol_lamports, snapshot_pending + 1_000);
            assert_eq!(w.spendable(PENDING).unwrap(), snapshot_pending + 1_000 - used);
            assert!(w.reconcile().unwrap().is_no_change());
            finish_legs(&mut w, true);
            let frozen = w.round;
            w.explicit_sol(11, 11).unwrap();
            w.direct_sol(PENDING, 13).unwrap();
            w.reconcile().unwrap();
            assert_eq!(w.round, frozen);
            assert_eq!(w.settle().unwrap(), SettlementOutcome::Settled);
            let round = w.round;
            let settlement_hwm = w.config.protected_principal_hwm_lamports;
            assert_eq!(settlement_hwm, 1_000_000 + round.actual_hwm_delta_lamports);
            assert_eq!(w.spendable(ESCROW).unwrap(), round.actual_escrow_remainder_lamports);
            assert!(w.reconcile().unwrap().is_no_change());
            w.explicit_sol(17, 17).unwrap();
            w.direct_sol(PENDING, 19).unwrap();
            w.explicit_tokens(5, 5).unwrap();
            w.direct_tokens(PENDING_TOKEN, 2).unwrap();
            w.reconcile().unwrap();
            assert_eq!(w.round, round);
            assert_eq!(w.config.accounted_pending_sol_lamports, snapshot_pending + 1_060);
            let contribution = integrate_assert(&mut w, 900_100);
            assert_eq!(w.config.protected_principal_hwm_lamports, settlement_hwm + contribution);
            let hwm = w.config.protected_principal_hwm_lamports;
            // Legitimate later pool rewards, not donated token units, create yield.
            w.pool.increase_exchange_rate(200_000).unwrap();
            w.explicit_sol(30_000, 30_000).unwrap();
            w.open(900_000 + 864_000).unwrap();
            assert_eq!(w.round.active_sequence, 1);
            assert_eq!(w.round.old_protected_principal_lamports, hwm);
            assert_eq!(w.round.pending_sol_snapshot_lamports, 30_000);
            finish_legs(&mut w, false);
            w.settle().unwrap();
            assert_eq!(integrate_assert(&mut w, 1_764_100), 30_000);
            w.validate().unwrap();
        }
    }
}

#[test]
fn exact_zero_full_and_partial_pending_use_have_no_false_deficit() {
    for initial in [0, 8_050, 10_000] {
        let mut w = world(initial);
        w.open(900_000).unwrap();
        assert_eq!(w.round.pending_sol_used_lamports, initial.min(8_050));
        assert_eq!(expected_pending_sol_lamports(&w.config, &w.round).unwrap(),
                   initial - initial.min(8_050));
        assert!(w.reconcile().unwrap().is_no_change());
    }
}

#[test]
fn economic_surplus_moves_into_pending_once_at_every_supported_boundary() {
    for stage in 0..5 {
        let mut w = World::new(10_000, 50, 123, 9, 0, FeeFraction::ZERO, 100_000);
        if stage > 0 { w.open(900_000).unwrap(); }
        if stage > 1 { finish_legs(&mut w, false); }
        if stage > 2 { w.settle().unwrap(); }
        if stage > 3 { integrate_assert(&mut w, 900_100); }
        let frozen = w.round;
        let hwm = w.config.protected_principal_hwm_lamports;
        let historical = w.config.accounted_historical_jitosol_units;
        let claims = w.config.kif_claim_liability_lamports;
        let carry = w.config.collective_kif_carry_lamports;
        let before = w.observation().amounts().unwrap();
        for (vault, amount) in [(PENDING, 2), (PRINCIPAL, 3), (ESCROW, 5), (KIF, 7)] {
            w.direct_sol(vault, amount).unwrap();
        }
        w.direct_tokens(PENDING_TOKEN, 11).unwrap();
        w.direct_tokens(PRINCIPAL_TOKEN, 13).unwrap();
        let result = w.normalize().unwrap();
        assert_eq!(result.newly_accounted_sol_lamports, 17);
        assert_eq!(result.newly_accounted_jitosol_units, 24);
        let after = w.observation().amounts().unwrap();
        assert_eq!(after.pending_sol_lamports, before.pending_sol_lamports + 17);
        assert_eq!(after.pending_jitosol_units, before.pending_jitosol_units + 24);
        assert_eq!(after.principal_sol_lamports, before.principal_sol_lamports);
        assert_eq!(after.escrow_sol_lamports, before.escrow_sol_lamports);
        assert_eq!(after.kif_sol_lamports, before.kif_sol_lamports);
        assert_eq!(after.principal_jitosol_units, before.principal_jitosol_units);
        assert_eq!(w.config.protected_principal_hwm_lamports, hwm);
        assert_eq!(w.config.accounted_historical_jitosol_units, historical);
        assert_eq!(w.config.kif_claim_liability_lamports, claims);
        assert_eq!(w.config.collective_kif_carry_lamports, carry);
        assert_eq!(w.round, frozen);
        let normalized = w.clone();
        assert!(w.normalize().unwrap().is_no_change());
        assert!(w.reconcile().unwrap().is_no_change());
        assert_eq!(w, normalized);
    }
}

#[test]
fn full_fee_plus_burn_debit_multileg_and_out_of_order_finalization() {
    for reverse in [false, true] {
        let mut w = World::new(1_000, 999, 0, 9, 3,
            FeeFraction { numerator: 1, denominator: 1_000 }, 2_500);
        w.open(900_000).unwrap();
        let initial_historical = w.tokens[PRINCIPAL_TOKEN];
        let pending = w.tokens[PENDING_TOKEN];
        let first = w.initiate(1).unwrap();
        w.advance_epoch().unwrap();
        assert_eq!(w.finalize(first).unwrap(), LegFinalizationOutcome::Recorded);
        // Finalization before assignment completes must not free pending tokens.
        let mut source = 2;
        while !w.round.is_withdrawal_target_assigned() {
            w.initiate(source).unwrap();
            source += 1;
        }
        assert!(w.round.successful_leg_count >= 3);
        let count = w.round.successful_leg_count as usize;
        w.advance_epoch().unwrap();
        for i in 1..count {
            let index = if reverse { count - i } else { i };
            w.finalize(index).unwrap();
        }
        assert_eq!(initial_historical - w.tokens[PRINCIPAL_TOKEN],
                   w.round.cumulative_jitosol_assigned_units);
        assert_eq!(initial_historical - w.tokens[PRINCIPAL_TOKEN],
                   w.round.cumulative_withdrawal_fee_units + w.round.cumulative_burned_units);
        assert!(w.round.cumulative_withdrawal_fee_units > 0);
        assert_eq!(w.tokens[PENDING_TOKEN], pending);
        let consumed = w.round.cumulative_jitosol_assigned_units;
        w.settle().unwrap();
        integrate_assert(&mut w, 900_100);
        assert_eq!(w.tokens[PRINCIPAL_TOKEN], initial_historical - consumed + pending);
        assert_eq!(w.spendable(OPERATIONS).unwrap(), 100_000);
        assert_eq!(w.audit.advanced_rent, w.audit.recovered_rent);
    }
}

#[test]
fn all_staged_debits_credits_and_late_state_errors_roll_back() {
    for failure in [Failure::AfterDebit, Failure::MissingCredit,
                    Failure::WrongCredit, Failure::BeforeCommit] {
        let mut opening = world(10_000);
        opening.failure = Some(failure);
        rejected(&mut opening, |w| w.open(900_000));

        let mut initiating = world(1_000);
        initiating.open(900_000).unwrap();
        initiating.failure = Some(failure);
        rejected(&mut initiating, |w| w.initiate(1));

        let mut finalizing = world(1_000);
        finalizing.open(900_000).unwrap();
        finalizing.initiate(1).unwrap();
        finalizing.advance_epoch().unwrap();
        finalizing.failure = Some(failure);
        rejected(&mut finalizing, |w| w.finalize(0));

        let mut settling = world(10_000);
        settling.open(900_000).unwrap();
        settling.failure = Some(failure);
        rejected(&mut settling, |w| w.settle());

        let mut integrating = world(10_000);
        integrating.open(900_000).unwrap();
        integrating.settle().unwrap();
        integrating.failure = Some(failure);
        rejected(&mut integrating, |w| w.integrate(900_100));

        let mut normalizing = world(10_000);
        normalizing.direct_sol(PRINCIPAL, 50).unwrap();
        normalizing.failure = Some(failure);
        rejected(&mut normalizing, |w| w.normalize());
        let mut token_normalizing = world(0);
        token_normalizing.direct_tokens(PRINCIPAL_TOKEN, 50).unwrap();
        token_normalizing.failure = Some(failure);
        rejected(&mut token_normalizing, |w| w.normalize());
    }

    let mut late = world(10_000);
    late.open(900_000).unwrap();
    late.config.cumulative_gross_yield_lamports = u64::MAX;
    let before_late = late.clone();
    assert_eq!(late.settle(), Err(Error::State(Piv1Error::ArithmeticOverflow)));
    assert_eq!(late, before_late);

    let mut integration = world(10_000);
    integration.open(900_000).unwrap();
    integration.settle().unwrap();
    // This audit counter is not custody. Its overflow is a genuinely late
    // integrate_pending_and_complete error after all movements were staged.
    integration.config.cumulative_contribution_value_lamports = u64::MAX;
    rejected(&mut integration, |w| w.integrate(900_100));
}

#[test]
fn forged_round_offsets_snapshots_and_overconsumption_are_rejected() {
    let mut valid = world(10_000);
    valid.open(900_000).unwrap();
    for mode in 0..8 {
        let mut w = valid.clone();
        match mode {
            0 => w.config.next_distribution_sequence += 1,
            1 => w.config.accounted_historical_jitosol_units += 1,
            2 => w.config.last_successful_preparation_at = Some(0),
            3 => w.round.pending_sol_used_lamports += 1,
            4 => w.config.accounted_pending_sol_lamports = 1,
            5 => w.config.next_cycle_yield_lamports = 1,
            6 => w.round.historical_sol_lamports += 1,
            _ => w.round.snapshot_pool_total_lamports += 100_000,
        }
        rejected(&mut w, |w| w.reconcile());
        rejected(&mut w, |w| w.normalize());
    }
    valid.settle().unwrap();
    let completed = valid.round;
    valid.integrate(900_100).unwrap();
    assert_eq!(expected_pending_sol_lamports(&valid.config, &valid.round).unwrap(), 0);
    valid.round = completed;
    rejected(&mut valid, |w| w.reconcile());

    let mut legs = world(0);
    legs.open(900_000).unwrap();
    legs.initiate(1).unwrap();
    rejected(&mut legs, |w| w.initiate(2));
    legs.advance_epoch().unwrap();
    legs.finalize(0).unwrap();
    rejected(&mut legs, |w| w.finalize(0));
    let mut wrong = legs.clone();
    wrong.legs[0].sequence += 1;
    rejected(&mut wrong, |w| w.finalize(0));

    let mut impossible = world(0);
    impossible.open(900_000).unwrap();
    impossible.config.accounted_historical_jitosol_units = 1;
    impossible.round.historical_jitosol_units = 1;
    assert!(economic_custody_obligations(&impossible.config, &impossible.round).is_err());
}

#[test]
fn exact_normalization_observations_reject_missing_mismatched_or_wrong_credit() {
    let mut w = world(10_000);
    w.open(900_000).unwrap();
    w.direct_sol(PRINCIPAL, 5).unwrap();
    let before = w.observation();
    let config = w.config.clone();
    for mode in 0..4 {
        let mut after = before;
        match mode {
            0 => {} // no debit or credit
            1 => after.principal_sol.lamports -= 5, // missing destination
            2 => { after.principal_sol.lamports -= 5; after.kif_sol.lamports += 5; }
            _ => {
                after.principal_sol.lamports -= 5;
                after.pending_sol.lamports += 5;
                after.pending_sol.non_economic_floor_lamports += 1;
            }
        }
        assert!(record_economic_normalization(&mut w.config, &w.round, before, after).is_err());
        assert_eq!(w.config, config);
    }
}

#[test]
fn known_deficits_remain_visible_despite_unrelated_surplus_or_new_explicit_credit() {
    for vault in [PENDING, PRINCIPAL, ESCROW, KIF] {
        let mut w = World::new(10_000, 50, 100, 9, 3, FeeFraction::ZERO, 100_000);
        w.open(900_000).unwrap();
        w.direct_sol(if vault == PRINCIPAL { KIF } else { PRINCIPAL }, 10_000).unwrap();
        w.sol[vault] -= 1;
        assert_eq!(economic_custody_surplus(&w.config, &w.round, w.observation()),
                   Err(Piv1Error::EconomicCustodyDeficit));
        rejected(&mut w, |w| w.normalize());
        rejected(&mut w, |w| w.explicit_sol(1_000, 1_000));
    }
    let mut w = world(10_000);
    w.open(900_000).unwrap();
    let before = w.config.clone();
    let floor = w.floors[PENDING];
    let expected = 10_000 - 8_050;
    assert_eq!(record_explicit_sol_contribution(&mut w.config, &w.round, 1_000,
        SolCustodyObservation { vault_lamports_before: floor + expected - 1,
            vault_lamports_after: floor + expected + 999, non_economic_floor_lamports: floor }),
        Err(Piv1Error::PendingCustodyDeficit));
    assert_eq!(w.config, before);

    for token in [PENDING_TOKEN, PRINCIPAL_TOKEN] {
        let mut w = world(10_000);
        w.direct_sol(PRINCIPAL, 100).unwrap();
        w.tokens[token] -= 1;
        rejected(&mut w, |w| w.normalize());
    }
}

#[test]
fn rent_rewards_old_carry_and_zero_active_kif_have_exact_physical_destinations() {
    for prior in [0, 2_000, 20_000, 100_000] {
        let mut w = World::new(0, 0, prior, 101, 0, FeeFraction::ZERO, 100_000);
        w.pool.set_source_finalization_terms(WithdrawalSourceId(1), 1, 20, 10, 37, 0).unwrap();
        let old_hwm = w.config.protected_principal_hwm_lamports;
        w.open(900_000).unwrap();
        let old_carry_used = w.round.prior_next_cycle_yield_used_lamports().unwrap();
        assert_eq!(w.config.next_cycle_yield_lamports, 0);
        assert_eq!(w.spendable(PRINCIPAL).unwrap(), prior - old_carry_used);
        finish_legs(&mut w, false);
        let reward = w.round.cumulative_cooldown_rewards_lamports;
        assert_eq!(w.spendable(OPERATIONS).unwrap(), 100_000);
        let kif_before = w.spendable(KIF).unwrap();
        assert_eq!(w.settle().unwrap(), SettlementOutcome::Settled);
        let zero = w.round.actual_zero_active_kif_compound_lamports;
        assert_eq!(zero, (101 + w.round.actual_kif_allocation_lamports) / 2);
        assert_eq!(w.spendable(PRINCIPAL).unwrap(), prior - old_carry_used + zero);
        assert_eq!(w.spendable(KIF).unwrap(),
                   kif_before + w.round.actual_kif_allocation_lamports - zero);
        assert_eq!(w.config.kif_claim_liability_lamports, 90);
        assert_eq!(w.config.next_cycle_yield_lamports, reward);
        assert_eq!(w.spendable(ESCROW).unwrap(), reward
            + w.round.actual_net_allocation_dust_lamports
            + w.round.actual_retained_conservative_dust_lamports);
        assert_eq!(w.config.protected_principal_hwm_lamports,
                   old_hwm + w.round.actual_hwm_delta_lamports);
        integrate_assert(&mut w, 900_100);
        assert_eq!(w.spendable(PRINCIPAL).unwrap(),
                   w.config.accounted_historical_sol_lamports + reward);
        let carry = w.config.collective_kif_carry_lamports;
        w.pool.increase_exchange_rate(100_000).unwrap();
        w.explicit_sol(30_000, 30_000).unwrap();
        w.open(1_764_000).unwrap();
        finish_legs(&mut w, true);
        w.settle().unwrap();
        assert_eq!(w.round.actual_zero_active_kif_compound_lamports,
                   (carry + w.round.actual_kif_allocation_lamports) / 2);
        integrate_assert(&mut w, 1_764_100);
    }
}

#[test]
fn successful_loss_finalization_commits_custody_and_recovery_but_errors_do_not() {
    let mut w = world(0);
    w.pool.set_source_finalization_terms(WithdrawalSourceId(1), 1, 20, 10, 0, 3).unwrap();
    w.open(900_000).unwrap();
    w.initiate(1).unwrap();
    w.advance_epoch().unwrap();
    let before = w.clone();
    w.failure = Some(Failure::BeforeCommit);
    rejected(&mut w, |w| w.finalize(0));
    w.failure = None;
    assert_eq!(w.finalize(0).unwrap(), LegFinalizationOutcome::RecoveryRequired);
    assert_eq!(w.round.lifecycle, DistributionLifecycle::RecoveryRequired);
    assert_eq!(w.config.protected_principal_hwm_lamports,
               before.config.protected_principal_hwm_lamports);
    assert_eq!(w.spendable(OPERATIONS).unwrap(), 100_000);
    assert_eq!(w.spendable(ESCROW).unwrap(),
               w.round.cumulative_finalized_delegated_native_lamports - 3);
    let frozen = w.round;
    w.explicit_sol(100, 100).unwrap();
    w.direct_sol(PENDING, 20).unwrap();
    w.explicit_tokens(7, 7).unwrap();
    w.direct_tokens(PENDING_TOKEN, 9).unwrap();
    w.reconcile().unwrap();
    assert_eq!(w.round, frozen);
    rejected(&mut w, |w| w.settle());
    rejected(&mut w, |w| w.integrate(900_100));
    w.direct_sol(KIF, 5).unwrap();
    rejected(&mut w, |w| w.normalize());
}

#[test]
fn pause_recognition_and_isolated_existing_kif_claim_effects_preserve_economics() {
    for stage in 0..4 {
        let mut w = world(10_000);
        if stage > 0 { w.open(900_000).unwrap(); }
        if stage > 1 { w.settle().unwrap(); }
        if stage > 2 { w.integrate(900_100).unwrap(); }
        w.config.paused = true;
        let frozen = w.round;
        let hwm = w.config.protected_principal_hwm_lamports;
        let non_kif = [w.sol[PRINCIPAL], w.sol[ESCROW], w.sol[OPERATIONS]];
        w.explicit_sol(3, 3).unwrap();
        w.direct_sol(PENDING, 5).unwrap();
        w.explicit_tokens(7, 7).unwrap();
        w.direct_tokens(PENDING_TOKEN, 11).unwrap();
        w.reconcile().unwrap();
        assert_eq!(w.round, frozen);
        let pending = w.sol[PENDING];
        w.claim_effect(0, 15).unwrap();
        assert_eq!(w.sol[PENDING], pending);
        assert_eq!([w.sol[PRINCIPAL], w.sol[ESCROW], w.sol[OPERATIONS]], non_kif);
        assert_eq!(w.round, frozen);
        assert_eq!(w.config.protected_principal_hwm_lamports, hwm);
        rejected(&mut w, |w| w.claim_effect(0, u64::MAX));
        rejected(&mut w, |w| w.open(1_764_000));
        rejected(&mut w, |w| w.settle());
        rejected(&mut w, |w| w.integrate(900_100));
        w.direct_sol(PRINCIPAL, 1).unwrap();
        rejected(&mut w, |w| w.normalize());
    }
}

#[test]
fn operational_surplus_is_explicitly_unsupported_and_cannot_block_economic_normalization() {
    let mut w = world(0);
    let reserve = w.sol[OPERATIONS];
    w.direct_sol(OPERATIONS, 777).unwrap();
    w.direct_sol(KIF, 13).unwrap();
    assert_eq!(assess_operational_surplus(SolVaultBalance {
        lamports: w.sol[OPERATIONS], non_economic_floor_lamports: w.floors[OPERATIONS],
    }).unwrap(), OperationalSurplusAssessment::UnsupportedFundingBaseline);
    assert_eq!(w.normalize().unwrap().newly_accounted_sol_lamports, 13);
    assert_eq!(w.sol[OPERATIONS], reserve + 777);
    assert_eq!(w.config.accounted_pending_sol_lamports, 13);
}

#[test]
fn arithmetic_limits_floors_and_missing_integration_movement_reject_atomically() {
    let mut w = world(10_000);
    w.open(900_000).unwrap();
    w.settle().unwrap();
    let before = w.observation();
    assert!(derive_pending_integration(&w.config, &w.round, 900_100,
        w.pool.raw_snapshot(), before, before).is_err());
    assert_eq!(SolVaultBalance { lamports: 0, non_economic_floor_lamports: 1 }.economic_lamports(),
               Err(Piv1Error::InvalidCustodyObservation));
    assert_eq!(SolVaultBalance { lamports: u64::MAX, non_economic_floor_lamports: 0 }
        .economic_lamports(), Ok(u64::MAX));
    let mut active_max = w.config.clone();
    active_max.accounted_pending_sol_lamports = u64::MAX;
    assert_eq!(expected_pending_sol_lamports(&active_max, &w.round),
               Ok(u64::MAX - w.round.pending_sol_used_lamports));
    let before_max = active_max.clone();
    assert_eq!(reconcile_pending_contributions(&mut active_max, &w.round,
        PendingCustodyObservation { pending_sol_vault_lamports: u64::MAX,
            pending_sol_non_economic_floor_lamports: 0,
            pending_jitosol_token_units: w.config.accounted_pending_jitosol_units }),
        Err(Piv1Error::ArithmeticOverflow));
    assert_eq!(active_max, before_max);
    assert_eq!(observed_token_book_value(u64::MAX, w.pool.raw_snapshot()),
               Err(Piv1Error::InvalidCustodyObservation));
    let idle = ActiveDistribution::new_idle(0);
    let mut c = w.config.clone();
    c.accounted_pending_sol_lamports = u64::MAX;
    assert_eq!(expected_pending_sol_lamports(&c, &idle), Ok(u64::MAX));
    let original = c.clone();
    assert_eq!(record_explicit_sol_contribution(&mut c, &idle, 1,
        SolCustodyObservation { vault_lamports_before: u64::MAX - 1,
            vault_lamports_after: u64::MAX, non_economic_floor_lamports: 0 }),
        Err(Piv1Error::ArithmeticOverflow));
    assert_eq!(c, original);
    c.kif_claim_liability_lamports = u64::MAX;
    c.cumulative_kif_credited_lamports = u64::MAX;
    c.collective_kif_carry_lamports = 1;
    assert_eq!(economic_custody_obligations(&c, &idle), Err(Piv1Error::ArithmeticOverflow));
}

struct Rng(u64);
impl Rng {
    fn next(&mut self) -> u64 {
        self.0 = self.0.wrapping_add(0x9e37_79b9_7f4a_7c15);
        let mut v = self.0;
        v = (v ^ (v >> 30)).wrapping_mul(0xbf58_476d_1ce4_e5b9);
        v = (v ^ (v >> 27)).wrapping_mul(0x94d0_49bb_1331_11eb);
        v ^ (v >> 31)
    }
}

#[test]
fn fixed_seed_composed_lifecycles_cover_successful_and_rejected_actions() {
    let mut rng = Rng(SEED);
    let mut completed = 0usize;
    let mut legs = 0usize;
    let mut accepted = 0usize;
    let mut rejections = 0usize;
    let mut liquid = 0usize;
    let mut multileg = 0usize;
    for case in 0..CASES {
        let pending = if case % 3 == 0 { 15_000 } else { rng.next() % 4_000 };
        let fee = if case % 2 == 0 { FeeFraction::ZERO }
            else { FeeFraction { numerator: 1, denominator: 1_000 } };
        let initial_pending_tokens = rng.next() % 100;
        let mut w = World::new(pending, initial_pending_tokens, rng.next() % 500,
            rng.next() % 100, (rng.next() % 7) as usize, fee, 3_000);
        for action in 0..32 {
            let selector = rng.next() % 8;
            let result = match selector {
                0 => w.explicit_sol(3, 3),
                1 => w.direct_sol(PRINCIPAL, 7),
                2 => w.direct_tokens(PRINCIPAL_TOKEN, 11),
                3 => w.normalize().map(|_| ()),
                4 => w.reconcile().map(|_| ()),
                5 => w.explicit_tokens(5, 5),
                6 => {
                    let before = w.clone();
                    let r = w.explicit_sol(4, 5);
                    assert!(r.is_err(), "seed={SEED:#x} case={case} action={action}");
                    assert_eq!(w, before, "seed={SEED:#x} case={case} action={action}");
                    rejections += 1;
                    continue;
                }
                _ => {
                    w.failure = Some(Failure::BeforeCommit);
                    let before = w.clone();
                    assert!(w.explicit_sol(1, 1).is_err(),
                            "seed={SEED:#x} case={case} action={action}");
                    assert_eq!(w, before, "seed={SEED:#x} case={case} action={action}");
                    w.failure = None;
                    rejections += 1;
                    continue;
                }
            };
            result.unwrap_or_else(|e| panic!("seed={SEED:#x} case={case} action={action} selector={selector}: {e:?}"));
            accepted += 1;
            w.validate().unwrap();
        }
        w.normalize().unwrap();
        w.open(900_000).unwrap_or_else(|e| panic!("seed={SEED:#x} case={case} open: {e:?}"));
        if w.round.fixed_jitosol_withdrawal_target_units == 0 { liquid += 1; }
        let frozen = w.round;
        for action in 32..40 {
            w.direct_sol(PENDING, rng.next() % 100).unwrap();
            w.explicit_tokens(1, 1).unwrap();
            w.reconcile().unwrap_or_else(|e| panic!("seed={SEED:#x} case={case} action={action}: {e:?}"));
            assert_eq!(w.round, frozen, "seed={SEED:#x} case={case} action={action}");
            accepted += 3;
        }
        // Surplus normalization and rejection checks also run after snapshot.
        w.direct_sol(ESCROW, 5).unwrap();
        w.direct_tokens(PRINCIPAL_TOKEN, 7).unwrap();
        w.normalize().unwrap();
        assert_eq!(w.round, frozen, "seed={SEED:#x} case={case} action=40");
        if w.round.fixed_jitosol_withdrawal_target_units > 0 {
            for source in 1..=8 {
                if w.round.is_withdrawal_target_assigned() { break; }
                let index = w.initiate(source).unwrap_or_else(|e|
                    panic!("seed={SEED:#x} case={case} action=41 source={source}: {e:?}"));
                let before = w.clone();
                assert!(w.finalize(index).is_err(), "seed={SEED:#x} case={case} action=42 index={index}");
                assert_eq!(w, before, "seed={SEED:#x} case={case} action=42 index={index}");
                rejections += 1;
                accepted += 1;
            }
            w.advance_epoch().unwrap();
            let count = w.round.successful_leg_count as usize;
            for i in 0..count {
                let index = if case % 2 == 0 { count - 1 - i } else { i };
                w.finalize(index).unwrap_or_else(|e|
                    panic!("seed={SEED:#x} case={case} action=43 index={index}: {e:?}"));
                let before = w.clone();
                assert!(w.finalize(index).is_err(), "seed={SEED:#x} case={case} action=44 index={index}");
                assert_eq!(w, before, "seed={SEED:#x} case={case} action=44 index={index}");
                rejections += 1;
                accepted += 1;
            }
        }
        legs += w.round.successful_leg_count as usize;
        if w.round.successful_leg_count > 1 { multileg += 1; }
        assert_eq!(w.settle().unwrap_or_else(|e| panic!("seed={SEED:#x} case={case} settle: {e:?}")),
                   SettlementOutcome::Settled, "seed={SEED:#x} case={case}");
        w.direct_sol(KIF, 3).unwrap();
        w.explicit_tokens(2, 2).unwrap();
        w.normalize().unwrap();
        // Independent recognition oracle from initial pending custody plus
        // external input audit; no output of the normalizer supplies this value.
        assert_eq!(u128::from(w.config.accounted_pending_sol_lamports),
            u128::from(pending) + w.audit.external_sol,
            "seed={SEED:#x} case={case} action=45 SOL recognition");
        assert_eq!(u128::from(w.config.accounted_pending_jitosol_units),
            u128::from(initial_pending_tokens) + w.audit.external_tokens,
            "seed={SEED:#x} case={case} action=45 token recognition");
        let checked_integration = std::panic::catch_unwind(std::panic::AssertUnwindSafe(||
            integrate_assert(&mut w, 900_100)));
        assert!(checked_integration.is_ok(), "seed={SEED:#x} case={case} action=46 integration");
        accepted += 6;
        completed += 1;
    }
    assert_eq!(completed, CASES);
    assert!(accepted > 5_000 && rejections > 800 && liquid >= 40 && multileg >= 60 && legs >= 160);
    println!("seed={SEED:#018x} cases={CASES} completed={completed} accepted_actions={accepted} rejected_actions={rejections} liquid={liquid} multileg={multileg} legs={legs}");
}

#[test]
fn settlement_hwm_recovery_commits_only_recovery_header_and_no_payments() {
    let mut w = world(10_000);
    w.open(900_000).unwrap();
    w.pool.decrease_exchange_rate(100_000).unwrap();
    let before = w.clone();
    assert_eq!(w.settle().unwrap(), SettlementOutcome::RecoveryRequired);
    assert_eq!(w.round.lifecycle, DistributionLifecycle::RecoveryRequired);
    assert_eq!(w.config, before.config);
    assert_eq!(w.rewards, before.rewards);
    assert_eq!(w.sol, before.sol);
    assert_eq!(w.tokens, before.tokens);
    assert_eq!(w.pool, before.pool);
    assert_eq!(w.audit, before.audit);
    let frozen = w.round;
    w.config.paused = true;
    w.explicit_sol(10, 10).unwrap();
    w.direct_sol(PENDING, 5).unwrap();
    w.explicit_tokens(3, 3).unwrap();
    w.direct_tokens(PENDING_TOKEN, 7).unwrap();
    w.reconcile().unwrap();
    w.claim_effect(0, 15).unwrap();
    rejected(&mut w, |w| w.claim_effect(0, 15));
    assert_eq!(w.round, frozen);
    assert_eq!(w.config.protected_principal_hwm_lamports,
               before.config.protected_principal_hwm_lamports);
}

#[test]
fn residual_hwm_failure_during_leg_finalization_retains_recovered_funds() {
    let mut w = world(0);
    w.open(900_000).unwrap();
    w.initiate(1).unwrap();
    w.pool.decrease_exchange_rate(100_000).unwrap();
    w.advance_epoch().unwrap();
    assert_eq!(w.finalize(0).unwrap(), LegFinalizationOutcome::RecoveryRequired);
    assert_eq!(w.round.cumulative_cooldown_losses_lamports, 0);
    assert_eq!(w.spendable(OPERATIONS).unwrap(), 100_000);
    assert_eq!(w.spendable(ESCROW).unwrap(),
               w.round.cumulative_finalized_delegated_native_lamports);
    assert_eq!(w.config.protected_principal_hwm_lamports, 1_000_000);
    w.validate().unwrap();
}

fn finalization_after_pool_loss(pool_total: u64) -> World {
    let mut w = World::new(4_000, 0, 0, 0, 3, FeeFraction::ZERO, 100_000);
    w.open(900_000).unwrap();
    w.initiate(1).unwrap();
    w.pool.decrease_exchange_rate(w.pool.raw_snapshot().total_pool_lamports - pool_total).unwrap();
    w.advance_epoch().unwrap();
    w
}

fn liquid_settlement_after_pool_loss(pool_total: u64, active: usize, kif_carry: u64) -> World {
    let mut w = World::new(10_000, 0, 0, kif_carry, active, FeeFraction::ZERO, 100_000);
    w.open(900_000).unwrap();
    w.pool.decrease_exchange_rate(w.pool.raw_snapshot().total_pool_lamports - pool_total).unwrap();
    w
}

#[test]
fn severe_pool_loss_finalization_recovers_custody_across_pending_use_boundary() {
    // The first case is T23-R1's exact reproduction. The remaining pool totals
    // put retained value immediately below, at, and above the 4,000 pending use.
    for (pool_total, retained_value) in [(1_000, 99), (40_135, 3_999),
                                        (40_145, 4_000), (40_155, 4_001)] {
        let mut w = finalization_after_pool_loss(pool_total);
        let snapshot = w.pool.raw_snapshot();
        assert_eq!(w.spendable(PRINCIPAL).unwrap(), 0);
        assert_eq!(u128::from(w.tokens[PRINCIPAL_TOKEN]) * u128::from(pool_total)
            / u128::from(snapshot.pool_token_supply), retained_value);
        assert_eq!(w.round.pending_sol_used_lamports, 4_000);
        assert_eq!(w.round.cumulative_cooldown_losses_lamports, 0);
        let before = w.clone();
        let leg = before.legs[0];
        assert_eq!(w.finalize(0).unwrap(), LegFinalizationOutcome::RecoveryRequired);
        assert_eq!(w.round.lifecycle, DistributionLifecycle::RecoveryRequired);
        assert_eq!(w.round.recovery_flags, RECOVERY_FLAG_RESIDUAL_HWM);
        assert_eq!(w.round.finalized_leg_count, 1);
        assert_eq!(w.legs[0].status, WithdrawalLegStatus::Finalized);
        assert_eq!(w.legs[0].finalized_native_lamports,
                   leg.observed_delegated_native_lamports + leg.stake_rent_advanced_lamports);
        assert_eq!(w.round.cumulative_recovered_stake_rent_lamports,
                   leg.stake_rent_advanced_lamports);
        assert_eq!(w.round.cumulative_recovered_metadata_rent_lamports,
                   leg.metadata_rent_advanced_lamports);
        assert_eq!(w.spendable(ESCROW).unwrap(),
                   before.spendable(ESCROW).unwrap() + leg.observed_delegated_native_lamports);
        assert_eq!(w.spendable(OPERATIONS).unwrap(), 100_000);
        assert_eq!(w.audit.recovered_rent, w.audit.advanced_rent);
        assert_eq!(w.audit.cooldown_loss, 0);
        assert_eq!(w.audit.cooldown_reward, 0);
        assert_eq!(w.config, before.config); // Includes HWM and all pending/KIF ledgers.
        assert_eq!(w.rewards, before.rewards);
        assert_eq!(w.tokens, before.tokens);
        assert_eq!(w.token_rent, before.token_rent);
        assert_eq!(w.floors, before.floors);
        for vault in [PENDING, PRINCIPAL, KIF, HTFP, TEAM, CLAIMS] {
            assert_eq!(w.sol[vault], before.sol[vault]);
        }
        assert_eq!(w.sol.iter().map(|&v| u128::from(v)).sum::<u128>(),
                   before.sol.iter().map(|&v| u128::from(v)).sum::<u128>());
        w.validate().unwrap(); // Also checks closed temporary custody and pool conservation.
        rejected(&mut w, |w| w.finalize(0));
        rejected(&mut w, |w| w.settle());
        rejected(&mut w, |w| w.integrate(900_100));
        assert!(w.reconcile().unwrap().is_no_change());
    }
}

#[test]
fn severe_pool_loss_settlement_preserves_everything_except_recovery_header() {
    // Pool supply stays 10,000,000 and principal owns 1,000,000 units. These
    // cases reproduce retained value 100 and bracket the exact pending use.
    for (pool_total, retained_value) in [(1_000, 100), (80_490, 8_049),
                                        (80_500, 8_050), (80_510, 8_051)] {
        let mut w = liquid_settlement_after_pool_loss(pool_total, 3, 0);
        assert_eq!(w.round.pending_sol_used_lamports, 8_050);
        assert_eq!(w.spendable(PENDING).unwrap(), 1_950);
        let snapshot = w.pool.raw_snapshot();
        assert_eq!(u128::from(w.tokens[PRINCIPAL_TOKEN]) * u128::from(pool_total)
            / u128::from(snapshot.pool_token_supply), retained_value);
        let mut expected = w.clone();
        expected.round.lifecycle = DistributionLifecycle::RecoveryRequired;
        expected.round.recovery_flags |= RECOVERY_FLAG_RESIDUAL_HWM;
        assert_eq!(w.settle().unwrap(), SettlementOutcome::RecoveryRequired);
        assert_eq!(w, expected); // Includes escrow, HWM, pending, KIF and every audit counter.
        w.validate().unwrap();
        rejected(&mut w, |w| w.settle());
        rejected(&mut w, |w| w.integrate(900_100));
        assert!(w.reconcile().unwrap().is_no_change());
    }
}

#[test]
fn severe_pool_loss_settlement_reverts_zero_active_kif_compound_and_carry() {
    let mut w = liquid_settlement_after_pool_loss(1_000, 0, 101);
    assert_eq!(w.round.kif_active_guardian_count, 0);
    assert_eq!(w.round.kif_gross_obligation_lamports, 200);
    assert_eq!(w.round.kif_carry_input_lamports, 101);
    // Speculative settlement would move 150 from KIF into principal and keep
    // 151 collective carry. Neither change is committed when HWM is unprotected.
    let mut expected = w.clone();
    expected.round.lifecycle = DistributionLifecycle::RecoveryRequired;
    expected.round.recovery_flags |= RECOVERY_FLAG_RESIDUAL_HWM;
    assert_eq!(w.settle().unwrap(), SettlementOutcome::RecoveryRequired);
    assert_eq!(w, expected);
    assert_eq!(w.config.collective_kif_carry_lamports, 101);
    assert_eq!(w.config.kif_claim_liability_lamports, 90);
    assert_eq!(w.spendable(PRINCIPAL).unwrap(), 0);
    w.validate().unwrap();
}

#[test]
fn severe_pool_loss_does_not_mask_transfer_failures_malformed_state_or_overflow() {
    for finalizing in [true, false] {
        let ready = if finalizing { finalization_after_pool_loss(1_000) }
            else { liquid_settlement_after_pool_loss(1_000, 3, 0) };
        let execute = |w: &mut World| {
            if finalizing { w.finalize(0).map(|_| ()) } else { w.settle().map(|_| ()) }
        };
        for failure in [Failure::AfterDebit, Failure::MissingCredit,
                        Failure::WrongCredit, Failure::BeforeCommit] {
            let mut w = ready.clone();
            w.failure = Some(failure);
            rejected(&mut w, execute);
        }
        for corruption in 0..3 {
            let mut w = ready.clone();
            match corruption {
                0 => w.sol[KIF] -= 1,
                1 => w.round.pending_sol_used_lamports += 1,
                _ => w.config.paused = true,
            }
            rejected(&mut w, execute);
        }
    }
    let mut overflow = liquid_settlement_after_pool_loss(1_000, 3, 0);
    // Preserve the standalone reward identity, then force a checked overflow
    // while staging new KIF credit before the accepted recovery comparison.
    overflow.rewards[0].cumulative_earned = u64::MAX;
    overflow.rewards[0].cumulative_claimed = u64::MAX - overflow.rewards[0].claimable_lamports;
    overflow.validate().unwrap();
    let before = overflow.clone();
    assert_eq!(overflow.settle(), Err(Error::State(Piv1Error::ArithmeticOverflow)));
    assert_eq!(overflow, before);
}

#[test]
fn direct_tokens_during_withdrawal_and_pool_appreciation_stay_contribution_value() {
    let mut w = World::new(0, 50, 0, 0, 6,
        FeeFraction { numerator: 1, denominator: 1_000 }, 3_000);
    w.open(900_000).unwrap();
    w.initiate(1).unwrap();
    let frozen = w.round;
    let historical = w.tokens[PRINCIPAL_TOKEN];
    w.direct_tokens(PRINCIPAL_TOKEN, 100).unwrap();
    w.direct_tokens(PENDING_TOKEN, 25).unwrap();
    w.explicit_tokens(25, 25).unwrap();
    assert_eq!(w.normalize().unwrap().newly_accounted_jitosol_units, 125);
    assert_eq!(w.tokens[PRINCIPAL_TOKEN], historical);
    assert_eq!(w.tokens[PENDING_TOKEN], 200);
    assert_eq!(w.round, frozen);
    assert!(w.reconcile().unwrap().is_no_change());
    w.initiate(2).unwrap();
    w.initiate(3).unwrap();
    w.advance_epoch().unwrap();
    for i in [2, 0, 1] { w.finalize(i).unwrap(); }
    w.settle().unwrap();
    w.pool.increase_exchange_rate(100_000).unwrap();
    let pool = w.pool.raw_snapshot();
    let expected_value = (200u128 * u128::from(pool.total_pool_lamports)
        / u128::from(pool.pool_token_supply)) as u64;
    assert_eq!(integrate_assert(&mut w, 900_100), expected_value);
}

#[test]
fn scaled_100_60_10_regression_protects_110_once_for_explicit_and_direct_credits() {
    for explicit in [false, true] {
        let mut w = World::new(100_000, 0, 0, 0, 6, FeeFraction::ZERO, 100_000);
        // Real mock appreciation yields 74,536: the accepted floored outgoing
        // split is exactly 60,000, so the illustrative ratio is executable.
        w.pool.increase_exchange_rate(645_360).unwrap();
        w.open(900_000).unwrap();
        assert_eq!(w.round.gross_yield_lamports, 74_536);
        assert_eq!(w.round.outgoing_gross_obligation_lamports, 60_000);
        assert_eq!(w.round.pending_sol_used_lamports, 60_000);
        assert_eq!(w.config.accounted_pending_sol_lamports, 100_000);
        assert_eq!(w.spendable(PENDING).unwrap(), 40_000);
        assert!(w.reconcile().unwrap().is_no_change());
        if explicit { w.explicit_sol(10_000, 10_000).unwrap(); }
        else { w.direct_sol(PENDING, 10_000).unwrap(); w.reconcile().unwrap(); }
        assert_eq!(w.spendable(PENDING).unwrap(), 50_000);
        assert_eq!(w.config.accounted_pending_sol_lamports, 110_000);
        assert!(w.reconcile().unwrap().is_no_change());
        assert_eq!(w.settle().unwrap(), SettlementOutcome::Settled);
        let settlement_component = w.round.actual_hwm_delta_lamports;
        assert_eq!(settlement_component, 14_537);
        assert_eq!(integrate_assert(&mut w, 900_100), 110_000);
        assert_eq!(w.config.protected_principal_hwm_lamports, 1_124_537);
        assert_eq!(w.config.accounted_historical_sol_lamports, 50_001);
        assert_eq!(w.config.cumulative_contribution_value_lamports, 110_000);
    }
}
