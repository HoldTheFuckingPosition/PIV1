//! Independent host custody oracles. Failed callback worlds are discarded
//! explicitly; these tests do not prove actual Bank/System/Token rollback.
mod support;
#[path = "support/pending_custody.rs"] pub mod pending_custody;
#[allow(dead_code)] #[path = "support/economic_custody.rs"] mod custody;
#[path = "support/jito_identity_oracle.rs"] mod oracle;
#[allow(dead_code)] #[path = "support/bootstrap_custody.rs"] mod bootstrap_custody;
#[allow(dead_code)] #[path = "support/distribution_preparation_custody.rs"] mod preparation_custody;
#[allow(dead_code)] #[path = "support/withdrawal_preparation_custody.rs"] mod withdrawal_custody;
#[allow(dead_code)] #[path = "support/withdrawal_leg_custody.rs"] mod leg_custody;
#[allow(dead_code)] #[path = "support/withdrawal_finalization_custody.rs"] mod final_custody;
#[allow(dead_code)] #[path = "support/settlement_custody.rs"] mod settlement_custody;
#[path = "support/pending_integration_custody.rs"] mod integration_custody;
use support::kif_claim_custody;
use piv1::integrations;
use anchor_lang::{solana_program::program_error::ProgramError};
use piv1::{errors::Piv1Error, instruction_boundary,
    instruction_errors::{piv1_error_code, HOST_RUNTIME_UNAVAILABLE_CODE},
    instructions::integrate_pending::*, events::PendingContributionsIntegrated,
    state::{DistributionLifecycle, reconciliation::{derive_pending_integration, EconomicCustodyObservation}},
    pending_integration_execution::process_instruction_with_host_callbacks as execute};
use custody::*;
use bootstrap_custody::{POOL, LIST};
use integration_custody::{Fixture, expected};
use leg_custody::{emulate, snapshot};
use support::kif_claim_custody::{key, PROGRAM};
fn error(e: Piv1Error) -> ProgramError { ProgramError::Custom(piv1_error_code(e)) }
#[derive(Clone, Copy, Debug)]
enum Behavior { Good, Before(usize), After(usize), Noop(usize), Data(usize, usize, usize),
    Lamports(usize, usize), Hold(usize) }
fn raw(f: &mut Fixture, behavior: Behavior) -> (Result<(), ProgramError>, usize, Vec<PendingContributionsIntegrated>) {
    let reference = expected(f); let initial = f.custody.accounts.clone();
    let data = f.data(); let clock = f.clock.clone(); let rent = f.custody.rent.clone();
    let mut count = 0; let mut events = vec![];
    let (result, accounts) = f.custody.with_infos(|all| {
        let mut held = None;
        let result = execute(&PROGRAM, all, &data, || Ok(clock), || Ok(rent), |ix, infos, seeds| {
            let i = count; count += 1;
            if matches!(behavior, Behavior::Before(j) if j == i) { return Err(ProgramError::Custom(25001)); }
            if matches!(behavior, Behavior::Noop(j) if j == i) { return Ok(()); }
            let before = if i == 0 { &initial } else { &reference.calls[i - 1].after };
            emulate(before, &reference.calls[i], ix, infos, seeds, all)?;
            if matches!(behavior, Behavior::After(j) if j == i) { return Err(ProgramError::Custom(25002)); }
            if let Behavior::Data(j, account, offset) = behavior { if i == j { all[account].try_borrow_mut_data()?[offset] ^= 1; } }
            if let Behavior::Lamports(j, account) = behavior { if i == j { **all[account].try_borrow_mut_lamports()? += 1; } }
            if i + 1 == reference.calls.len() { if let Behavior::Hold(account) = behavior { held = Some(all[account].try_borrow_data()?); } }
            Ok(())
        }, |e| events.push(e));
        drop(held); (result, all.iter().map(snapshot).collect::<Vec<_>>())
    });
    f.custody.accounts = accounts; (result, count, events)
}
fn success(f: &mut Fixture) {
    let before = f.clone(); let reference = expected(f); let (result, calls, events) = raw(f, Behavior::Good);
    assert_eq!(result, Ok(())); assert_eq!(calls, reference.calls.len()); assert_eq!(events, vec![reference.event]);
    assert_eq!(*f, reference.after);
    let sum = |f: &Fixture| f.custody.accounts.iter().map(|a| u128::from(a.lamports)).sum::<u128>();
    assert_eq!(sum(&before), sum(f));
    for i in [OPERATIONAL_SOL, KIF_SOL, MINT, AUTHORITY, POOL, LIST] { assert_eq!(f.custody.accounts[i], before.custody.accounts[i]); }
    for i in [PRINCIPAL_JITO, PENDING_JITO] { assert_eq!(f.custody.accounts[i].lamports, before.custody.accounts[i].lamports); }
    assert_eq!(f.round().lifecycle, DistributionLifecycle::Idle);
    let c = f.custody.config(); let old = before.custody.config();
    assert_eq!((c.accounted_pending_sol_lamports, c.accounted_pending_jitosol_units), (0, 0));
    assert_eq!((c.kif_claim_liability_lamports, c.collective_kif_carry_lamports, c.next_cycle_yield_lamports),
        (old.kif_claim_liability_lamports, old.collective_kif_carry_lamports, old.next_cycle_yield_lamports));
    f.custody.with_infos(|a| {
        let fixed = piv1::accounts::authenticate_fixed_accounts(&PROGRAM, &before.custody.rent, roles(a)).unwrap();
        let after = fixed.pending_integration_observation().unwrap();
        assert_eq!(after.amounts().unwrap(), piv1::state::reconciliation::economic_custody_obligations(fixed.config(), fixed.distribution()).unwrap());
    });
}
fn reject(mut f: Fixture, expected_error: Option<ProgramError>) {
    let before = f.clone(); let clock = f.clock.clone(); let rent = f.custody.rent.clone(); let data = f.data();
    let result = f.custody.with_infos(|a| execute(&PROGRAM, a, &data, || Ok(clock), || Ok(rent),
        |_, _, _| panic!("invalid request reached CPI"), |_| panic!("no failure event")));
    if let Some(e) = expected_error { assert_eq!(result, Err(e)); } else { assert!(result.is_err()); }
    assert_eq!(f, before);
}

#[test]
fn liquid_withdrawn_and_guardian_matrix_matches_full_accounts_and_completed_summary() {
    let mut count = 0;
    for shared in [false, true] { for bitmap in [0, 1, 7, 0b101011, 63] { for withdrawal in [false, true] {
        let mut f = Fixture::new(shared, bitmap, withdrawal, if withdrawal { 97 } else { 0 });
        f.custody.accounts[PRINCIPAL_JITO].lamports += 11; f.custody.accounts[PENDING_JITO].lamports += 17;
        let mint_supply = u64::from_le_bytes(f.custody.accounts[MINT].data[36..44].try_into().unwrap());
        let reduced_supply = mint_supply - 500_000;
        assert!(f.custody.token_units(PRINCIPAL_JITO) + f.custody.token_units(PENDING_JITO) <= reduced_supply);
        let pending_value_numerator = u128::from(f.custody.token_units(PENDING_JITO)) * u128::from(f.pool().total_lamports);
        assert_ne!(pending_value_numerator / u128::from(f.pool().pool_token_supply), pending_value_numerator / u128::from(reduced_supply));
        f.custody.accounts[MINT].data[36..44].copy_from_slice(&reduced_supply.to_le_bytes());
        let before = f.clone(); let old = before.custody.config(); let r = before.round();
        assert!(r.pending_sol_used_lamports > 0);
        success(&mut f);
        let summary = f.round().last_completed.unwrap();
        assert_eq!(summary.sequence, r.active_sequence); assert_eq!(summary.successful_leg_count, r.successful_leg_count);
        assert_eq!(f.custody.config().protected_principal_hwm_lamports - old.protected_principal_hwm_lamports,
            old.accounted_pending_sol_lamports + (u128::from(old.accounted_pending_jitosol_units) * u128::from(before.pool().total_lamports)
                / u128::from(before.pool().pool_token_supply)) as u64);
        assert_eq!(f.custody.config().accounted_historical_sol_lamports + old.next_cycle_yield_lamports,
            f.custody.accounts[PRINCIPAL_SOL].lamports - f.custody.rent.minimum_balance(0));
        reject(f, Some(error(Piv1Error::InvalidLifecycle))); count += 1;
    }}}
    assert_eq!(count, 20);
}

#[test]
fn post_snapshot_contributions_and_fresh_ratio_use_full_recognized_value_once() {
    for (sol, tokens) in [(0, 0), (1, 1), (1999, 117), (100_000, 10_001)] {
        let mut f = Fixture::new(false, 0, true, 47); let r = f.round();
        f.custody.edit_config(|c| { c.accounted_pending_sol_lamports += sol; c.accounted_pending_jitosol_units += tokens; });
        f.custody.accounts[PENDING_SOL].lamports += sol;
        f.custody.set_token_units(PENDING_JITO, f.custody.token_units(PENDING_JITO) + tokens);
        let mut pool = f.pool(); pool.total_lamports += 100_007; pool.last_update_epoch += 1;
        f.clock.epoch = pool.last_update_epoch; f.clock.slot += 9; f.clock.unix_timestamp += 100;
        f.set_pool(&pool); f.sync_clock(); let full = f.custody.config().accounted_pending_sol_lamports;
        assert_eq!(f.custody.accounts[PENDING_SOL].lamports - f.custody.rent.minimum_balance(0), full - r.pending_sol_used_lamports);
        let expected_value = full + (u128::from(f.custody.token_units(PENDING_JITO)) * u128::from(pool.total_lamports) / u128::from(pool.pool_token_supply)) as u64;
        success(&mut f); assert_eq!(f.round().last_completed.unwrap().integrated_contribution_value_lamports, expected_value);
    }
}

#[test]
fn fully_substituted_pending_and_no_transfer_completion_keep_full_p_and_idle_offsets() {
    let settled = settlement_custody::Fixture::liquid_gross(false, 1, Some(10_000));
    let mut f = Fixture::from_settlement(settled); let mut r = f.round(); let floor = f.custody.rent.minimum_balance(0);
    assert_eq!(r.actual_escrow_remainder_lamports, 0); assert!(r.pending_sol_used_lamports > 0);
    r.pending_sol_snapshot_lamports = r.pending_sol_used_lamports; f.set_round(&r);
    f.custody.edit_config(|c| { c.accounted_pending_sol_lamports = r.pending_sol_used_lamports; c.accounted_pending_jitosol_units = 0; });
    f.custody.accounts[PENDING_SOL].lamports = floor; f.custody.set_token_units(PENDING_JITO, 0);
    for i in [PENDING_SOL, PRINCIPAL_SOL, ESCROW_SOL, PRINCIPAL_JITO, PENDING_JITO] { f.custody.accounts[i].writable = false; }
    assert!(expected(&f).calls.is_empty()); let old_hwm = f.custody.config().protected_principal_hwm_lamports;
    success(&mut f); assert_eq!(f.custody.config().protected_principal_hwm_lamports, old_hwm + r.pending_sol_used_lamports);
    assert_eq!(f.round().active_sequence, 0);
    reject(f, Some(error(Piv1Error::InvalidLifecycle)));
}

#[test]
fn protected_hwm_rejects_real_current_pool_loss_before_any_cpi() {
    let mut f = Fixture::new(false, 7, true, 100_000); let r = f.round(); let c = f.custody.config();
    assert_eq!(r.validate(), Ok(())); assert_eq!(c.validate_initialized(), Ok(()));
    let mut p = f.pool(); p.total_lamports = 1; f.set_pool(&p);
    // Valid normalized settled custody survives; only current value has fallen.
    let floor = f.custody.rent.minimum_balance(0);
    let protected = f.custody.accounts[PRINCIPAL_SOL].lamports - floor
        + f.custody.accounts[PENDING_SOL].lamports - floor + f.custody.accounts[ESCROW_SOL].lamports - floor
        - c.next_cycle_yield_lamports;
    assert!(protected < c.protected_principal_hwm_lamports + c.accounted_pending_sol_lamports);
    reject(f, Some(error(Piv1Error::HighWaterMarkDecrease)));
}

#[test]
fn stale_pool_supply_bounds_zero_pairs_and_empty_pool_are_explicit() {
    let base = Fixture::new(false, 7, false, 0);
    for mode in 0..5 {
        let mut f = base.clone(); let mut p = f.pool();
        match mode { 0 => p.last_update_epoch -= 1, 1 => p.total_lamports = 0,
            2 => p.pool_token_supply = 0, 3 => f.custody.accounts[MINT].data[36..44].copy_from_slice(&(p.pool_token_supply + 1).to_le_bytes()),
            _ => { let held = f.custody.token_units(PRINCIPAL_JITO) + f.custody.token_units(PENDING_JITO);
                f.custody.accounts[MINT].data[36..44].copy_from_slice(&(held - 1).to_le_bytes()); } }
        f.set_pool(&p); reject(f, Some(error(Piv1Error::InvalidCustodyObservation)));
    }
    // Valid normalized SOL-only settled state admits an actually empty pool.
    let mut f = base; let mut r = f.round(); let token_value = r.historical_value_lamports - r.historical_sol_lamports;
    r.historical_sol_lamports = r.historical_value_lamports; r.historical_jitosol_units = 0; f.set_round(&r);
    f.custody.edit_config(|c| { c.accounted_historical_sol_lamports += token_value; c.accounted_historical_jitosol_units = 0;
        c.accounted_pending_jitosol_units = 0; });
    f.custody.accounts[PRINCIPAL_SOL].lamports += token_value;
    f.custody.set_token_units(PRINCIPAL_JITO, 0); f.custody.set_token_units(PENDING_JITO, 0);
    let mut p = f.pool(); p.total_lamports = 0; p.pool_token_supply = 0; f.set_pool(&p);
    f.custody.accounts[MINT].data[36..44].copy_from_slice(&0_u64.to_le_bytes()); success(&mut f);
}

#[test]
fn pause_clock_sequence_lifecycle_and_legacy_derivation_precedence_remain_fail_closed() {
    let base = Fixture::new(false, 7, true, 17);
    let mut f = base.clone(); f.custody.edit_config(|c| c.paused = true); reject(f, Some(error(Piv1Error::PausedOperation)));
    let mut f = base.clone(); f.clock.unix_timestamp = f.round().prepared_at - 1; f.sync_clock(); reject(f, Some(error(Piv1Error::TimestampRegression)));
    let mut f = base.clone(); f.clock.epoch = f.round().prepared_epoch - 1; f.sync_clock(); reject(f, Some(error(Piv1Error::TimestampRegression)));
    let mut f = base.clone(); let mut r = f.round(); r.prepared_slot = f.clock.slot + 1; f.set_round(&r); reject(f, Some(error(Piv1Error::TimestampRegression)));
    let mut f = base.clone(); f.custody.accounts[f.base].data[0] ^= 1; reject(f, Some(error(Piv1Error::InvalidClockAccount)));
    let mut f = base.clone(); f.custody.edit_config(|c| c.next_distribution_sequence += 1); reject(f, Some(error(Piv1Error::SequenceMismatch)));
    let mut unsettled = settlement_custody::Fixture::new(false, 7, false, 0);
    let mut p = unsettled.pool(); p.total_lamports = 1; unsettled.set_pool(&p);
    let recovery = Fixture::from_settlement(unsettled); assert_eq!(recovery.round().lifecycle, DistributionLifecycle::RecoveryRequired);
    reject(recovery.clone(), Some(error(Piv1Error::RecoveryRequired)));
    let invalid_pool = integrations::PoolSnapshot { current_epoch: 1, last_update_epoch: 0, total_pool_lamports: 0,
        pool_token_supply: 1, sol_deposit_fee: integrations::FeeFraction::ZERO, stake_withdrawal_fee: integrations::FeeFraction::ZERO,
        minimum_delegation_lamports: 0, maximum_deposit_lamports: 0, available_withdrawal_lamports: 0, revision: 0 };
    let zero = EconomicCustodyObservation::default(); let mut c = recovery.custody.config();
    assert_eq!(derive_pending_integration(&c, &recovery.round(), 0, invalid_pool, zero, zero), Err(Piv1Error::InvalidLifecycle));
    c.paused = true;
    assert_eq!(derive_pending_integration(&c, &recovery.round(), 0, invalid_pool, zero, zero), Err(Piv1Error::PausedOperation));
    // Legacy valid-lifecycle custody error still precedes invalid pool validation.
    assert_eq!(derive_pending_integration(&base.custody.config(), &base.round(), 0, invalid_pool, zero, zero), Err(Piv1Error::EconomicCustodyDeficit));
}

#[test]
fn custody_rent_untracked_donations_identity_and_cumulative_overflow_reject() {
    let base = Fixture::new(false, 7, false, 0);
    for i in [PENDING_SOL, PRINCIPAL_SOL, ESCROW_SOL, KIF_SOL] { for increase in [false, true] {
        let mut f = base.clone(); if increase { f.custody.accounts[i].lamports += 1; } else { f.custody.accounts[i].lamports -= 1; }
        reject(f, None);
    }}
    for i in [PRINCIPAL_JITO, PENDING_JITO] { for increase in [false, true] {
        let mut f = base.clone(); let amount = f.custody.token_units(i);
        f.custody.set_token_units(i, if increase { amount + 1 } else { amount - 1 }); reject(f, None);
    }}
    for i in [PENDING_SOL, PRINCIPAL_JITO, CONFIG] {
        let mut f = base.clone(); f.custody.accounts[i].lamports = 0; reject(f, None);
    }
    for i in [SYSTEM, TOKEN, AUTHORITY, MINT, POOL] {
        let mut f = base.clone(); f.custody.accounts[i].key = key(240); reject(f, None);
    }
    let mut f = base.clone(); f.custody.edit_config(|c| c.cumulative_contribution_value_lamports = u64::MAX);
    reject(f, Some(error(Piv1Error::ArithmeticOverflow)));
    let mut f = base; let extra = u64::MAX - f.custody.config().accounted_pending_sol_lamports;
    // Recognized ledger overflow/value rejection before physical destinations can change.
    f.custody.edit_config(|c| c.accounted_pending_sol_lamports += extra);
    reject(f, None);
}

#[test]
fn each_cpi_failure_and_unexpected_effect_preserves_both_uncommitted_states_and_retry() {
    let mut base = Fixture::new(false, 0, true, 101);
    base.custody.edit_config(|c| c.accounted_pending_sol_lamports += 11); base.custody.accounts[PENDING_SOL].lamports += 11;
    let count = expected(&base).calls.len(); assert_eq!(count, 3);
    let mut behaviors = vec![];
    for step in 0..count { behaviors.extend([Behavior::Before(step), Behavior::After(step), Behavior::Noop(step),
        Behavior::Lamports(step, OPERATIONAL_SOL), Behavior::Lamports(step, PRINCIPAL_JITO),
        Behavior::Lamports(step, PENDING_JITO), Behavior::Data(step, POOL, 100), Behavior::Data(step, LIST, 40),
        Behavior::Data(step, MINT, 36)]); }
    behaviors.extend([Behavior::Hold(CONFIG), Behavior::Hold(ROUND)]);
    for behavior in behaviors {
        let mut failed = base.clone(); let (result, _, events) = raw(&mut failed, behavior);
        assert!(result.is_err(), "{behavior:?}"); assert!(events.is_empty());
        for i in [CONFIG, ROUND] { assert_eq!(failed.custody.accounts[i], base.custody.accounts[i]); }
        // Discard failed-world effects explicitly; production atomicity requires
        // later Bank evidence. A new staged attempt succeeds against original state.
        let mut retry = base.clone(); success(&mut retry);
    }
}

#[test]
fn keys_backing_aliases_writability_and_preflight_borrows_reject_without_effects() {
    let mut base = Fixture::new(false, 7, true, 17);
    base.custody.edit_config(|c| c.accounted_pending_sol_lamports += 11); base.custody.accounts[PENDING_SOL].lamports += 11;
    for pair in [(CONFIG, ROUND), (PENDING_SOL, PRINCIPAL_SOL), (ESCROW_SOL, KIF_SOL), (PRINCIPAL_JITO, PENDING_JITO), (MINT, POOL)] {
        let mut f = base.clone(); f.custody.accounts[pair.1].key = f.custody.accounts[pair.0].key; reject(f, Some(error(Piv1Error::AccountAlias)));
        for lamports in [false, true] {
            let mut f = base.clone(); let clock = f.clock.clone(); let rent = f.custody.rent.clone(); let data = f.data();
            f.custody.with_infos(|a| { let mut aliased = a.to_vec();
                if lamports { aliased[pair.1].lamports = a[pair.0].lamports.clone(); } else { aliased[pair.1].data = a[pair.0].data.clone(); }
                assert_eq!(execute(&PROGRAM, &aliased, &data, || Ok(clock), || Ok(rent), |_, _, _| panic!(), |_| panic!()), Err(error(Piv1Error::AccountAlias)));
            });
        }
    }
    for i in [CONFIG, ROUND, PENDING_SOL, PRINCIPAL_SOL, ESCROW_SOL, PRINCIPAL_JITO, PENDING_JITO] {
        let mut f = base.clone(); f.custody.accounts[i].writable = false; reject(f, Some(error(Piv1Error::AccountNotWritable)));
        let mut f = base.clone(); let clock = f.clock.clone(); let rent = f.custody.rent.clone(); let data = f.data();
        f.custody.with_infos(|a| { let _held = a[i].try_borrow_data().unwrap();
            assert!(execute(&PROGRAM, a, &data, || Ok(clock), || Ok(rent), |_, _, _| panic!(), |_| panic!()).is_err()); });
        assert_eq!(f, base);
    }
}

#[test]
fn strict_selector_account_topology_and_production_host_guard_are_not_bypasses() {
    let mut f = Fixture::new(false, 1, false, 0); let data = f.data();
    assert_eq!(decode_integrate_pending(&data), Ok(()));
    for malformed in [&data[..8], b"PIV1IP01\x00".as_slice(), b"PIV1IP01\x01\x00".as_slice()] {
        assert_eq!(decode_integrate_pending(malformed), Err(ProgramError::InvalidInstructionData));
        assert_eq!(instruction_boundary::process_instruction(&PROGRAM, &[], malformed), Err(ProgramError::InvalidInstructionData));
    }
    f.custody.with_infos(|a| {
        assert_eq!(instruction_boundary::process_instruction(&PROGRAM, &a[..18], &data), Err(ProgramError::NotEnoughAccountKeys));
        assert_eq!(instruction_boundary::process_instruction(&PROGRAM, a, &data), Err(ProgramError::Custom(HOST_RUNTIME_UNAVAILABLE_CODE)));
        let mut extra = a.to_vec(); extra.push(a[0].clone());
        assert_eq!(instruction_boundary::process_instruction(&PROGRAM, &extra, &data), Err(ProgramError::InvalidArgument));
    });
    let mut missing = f.clone(); missing.custody.accounts.pop(); reject(missing, Some(ProgramError::NotEnoughAccountKeys));
    let mut shared = Fixture::new(true, 1, false, 0); let mut extra = shared.custody.accounts[shared.base].clone(); extra.key = key(244);
    shared.custody.accounts.push(extra); reject(shared, Some(ProgramError::InvalidArgument));
}
