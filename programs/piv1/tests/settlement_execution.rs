//! Whole-account independent host settlement oracles; callback effects and
//! failed-world discard do not establish real runtime signatures or rollback.
mod support;
#[path = "support/pending_custody.rs"] pub mod pending_custody;
#[allow(dead_code)] #[path = "support/economic_custody.rs"] mod custody;
#[path = "support/jito_identity_oracle.rs"] mod oracle;
#[allow(dead_code)] #[path = "support/bootstrap_custody.rs"] mod bootstrap_custody;
#[allow(dead_code)] #[path = "support/distribution_preparation_custody.rs"] mod preparation_custody;
#[allow(dead_code)] #[path = "support/withdrawal_preparation_custody.rs"] mod withdrawal_custody;
#[allow(dead_code)] #[path = "support/withdrawal_leg_custody.rs"] mod leg_custody;
#[allow(dead_code)] #[path = "support/withdrawal_finalization_custody.rs"] mod final_custody;
#[path = "support/settlement_custody.rs"] mod settlement_custody;
use support::kif_claim_custody;
use piv1::integrations;
use anchor_lang::{prelude::Pubkey, solana_program::{program_error::ProgramError, system_program}};
use piv1::{errors::Piv1Error, instruction_boundary,
    instruction_errors::{piv1_error_code, HOST_RUNTIME_UNAVAILABLE_CODE},
    instructions::settle_distribution::*, state::DistributionLifecycle,
    settlement_execution::{process_instruction_with_host_callbacks as execute, SettlementEvent}};
use custody::*;
use bootstrap_custody::{POOL, LIST};
use settlement_custody::{Fixture, expected, amounts};
use leg_custody::{emulate, snapshot};
use support::kif_claim_custody::{key, PROGRAM};
fn error(e: Piv1Error) -> ProgramError { ProgramError::Custom(piv1_error_code(e)) }
#[derive(Clone, Copy, Debug)]
enum Behavior { Good, Before(usize), After(usize), Noop(usize), Data(usize, usize, usize),
    Lamports(usize, usize), Hold(usize) }
fn raw(f: &mut Fixture, behavior: Behavior) -> (Result<(), ProgramError>, usize, Vec<SettlementEvent>) {
    let reference = expected(f); let initial = f.custody.accounts.clone();
    let data = f.data(); let clock = f.clock.clone(); let rent = f.custody.rent.clone();
    let mut count = 0; let mut events = vec![];
    let (result, accounts) = f.custody.with_infos(|all| {
        let mut held = None;
        let result = execute(&PROGRAM, all, &data, || Ok(clock), || Ok(rent), |ix, infos, seeds| {
            let i = count; count += 1;
            if matches!(behavior, Behavior::Before(j) if j == i) { return Err(ProgramError::Custom(24901)); }
            if matches!(behavior, Behavior::Noop(j) if j == i) { return Ok(()); }
            let before = if i == 0 { &initial } else { &reference.calls[i - 1].after };
            emulate(before, &reference.calls[i], ix, infos, seeds, all)?;
            if matches!(behavior, Behavior::After(j) if j == i) { return Err(ProgramError::Custom(24902)); }
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
    let before = f.clone(); let expected = expected(f); let (result, calls, events) = raw(f, Behavior::Good);
    assert_eq!(result, Ok(())); assert_eq!(calls, expected.calls.len()); assert_eq!(events, vec![expected.event]);
    assert_eq!(*f, expected.after);
    let total = |f: &Fixture| f.custody.accounts.iter().map(|a| u128::from(a.lamports)).sum::<u128>();
    assert_eq!(total(&before), total(f));
    for i in [PENDING_SOL, OPERATIONAL_SOL, PRINCIPAL_JITO, PENDING_JITO] { assert_eq!(f.custody.accounts[i], before.custody.accounts[i]); }
    let old = before.custody.config(); let new = f.custody.config();
    assert_eq!(old.accounted_historical_sol_lamports, new.accounted_historical_sol_lamports);
    assert_eq!(old.accounted_historical_jitosol_units, new.accounted_historical_jitosol_units);
    assert_eq!(old.accounted_pending_sol_lamports, new.accounted_pending_sol_lamports);
    assert_eq!(old.accounted_pending_jitosol_units, new.accounted_pending_jitosol_units);
    assert!(new.protected_principal_hwm_lamports >= old.protected_principal_hwm_lamports);
}
fn reject(mut f: Fixture, expected_error: Option<ProgramError>) {
    let before = f.clone(); let clock = f.clock.clone(); let rent = f.custody.rent.clone(); let data = f.data();
    let result = f.custody.with_infos(|a| execute(&PROGRAM, a, &data, || Ok(clock), || Ok(rent),
        |_, _, _| panic!("invalid request reached CPI"), |_| panic!("no failure event")));
    if let Some(e) = expected_error { assert_eq!(result, Err(e)); } else { assert!(result.is_err()); }
    assert_eq!(f, before);
}

#[test]
fn liquid_and_withdrawn_full_account_oracles_preserve_kif_pending_and_cooldown_categories() {
    let mut count = 0;
    for shared in [false, true] { for bitmap in [0, 1, 7, 0b101011, 63] { for withdrawal in [false, true] {
        let mut f = Fixture::new(shared, bitmap, withdrawal, if withdrawal { 97 } else { 0 });
        f.custody.accounts[PRINCIPAL_JITO].lamports += 11; f.custody.accounts[PENDING_JITO].lamports += 17;
        let mint_supply = u64::from_le_bytes(f.custody.accounts[MINT].data[36..44].try_into().unwrap());
        f.custody.accounts[MINT].data[36..44].copy_from_slice(&(mint_supply - 13).to_le_bytes());
        let r = f.round(); let old = f.custody.config(); let n = amounts(&f); assert!(!n.recovery);
        success(&mut f); assert_eq!(f.round().lifecycle, DistributionLifecycle::Settled);
        assert_eq!(f.custody.config().kif_claim_liability_lamports, old.kif_claim_liability_lamports + n.liability);
        assert_eq!(f.custody.config().next_cycle_yield_lamports, r.cumulative_cooldown_rewards_lamports);
        assert_eq!(f.round().pending_sol_used_lamports, r.pending_sol_used_lamports);
        reject(f, Some(error(Piv1Error::SettlementReplay))); count += 1;
    }}}
    assert_eq!(count, 20);
}

#[test]
fn accepted_1800_floor_case_zero_transfer_and_recipient_rent_boundaries() {
    let mut f = Fixture::liquid_gross(false, 1, Some(2238));
    let n = amounts(&f); assert_eq!((n.net, n.htfp, n.team, n.kif, n.dust), (1800, 1319, 436, 44, 1)); success(&mut f);
    let mut f = Fixture::liquid_gross(false, 1, Some(2)); let n = amounts(&f);
    assert_eq!((n.net, n.total, n.dust), (1, 0, 1));
    f.custody.accounts[f.base + 1].lamports = 0; f.custody.accounts[f.base + 2].lamports = 0;
    assert!(expected(&f).calls.is_empty()); success(&mut f);
    let base = Fixture::new(false, 1, false, 0); let n = amounts(&base); let floor = base.custody.rent.minimum_balance(0);
    let mut f = base.clone(); f.custody.accounts[f.base + 1].lamports = floor - n.htfp; success(&mut f);
    let mut f = base.clone(); f.custody.accounts[f.base + 1].lamports = floor - n.htfp - 1;
    reject(f, Some(error(Piv1Error::AccountRentDeficit)));
    let mut f = base; f.custody.accounts[f.base + 1].lamports = u64::MAX; reject(f, Some(error(Piv1Error::ArithmeticOverflow)));
}

#[test]
fn frozen_eligibility_recipients_and_old_revision_claims_survive_later_activity_and_claims() {
    let mut f = Fixture::new(false, 0b010101, true, 101); let r = f.round();
    f.custody.edit_config(|c| { c.guardian_registry_revision += 1; c.htfp_recipient = key(240); c.team_owner_recipient = key(241); });
    for i in 0..6 { let mut reward = f.reward(i); reward.last_active_period = Some(99); f.set_reward(i, &reward); }
    let mut reward = f.reward(0); reward.claimable_lamports -= 3; reward.cumulative_claimed += 3; f.set_reward(0, &reward);
    f.custody.edit_config(|c| { c.kif_claim_liability_lamports -= 3; c.cumulative_kif_claimed_lamports += 3; });
    f.custody.accounts[KIF_SOL].lamports -= 3;
    let sum: u64 = (0..6).map(|i| f.reward(i).claimable_lamports).sum();
    assert!(sum < f.custody.config().kif_claim_liability_lamports);
    let before = f.clone(); success(&mut f);
    assert_eq!(f.round().kif_eligibility_bitmap, r.kif_eligibility_bitmap);
    assert_eq!(f.round().htfp_recipient, r.htfp_recipient);
    for i in 0..6 { let expected_credit = if r.kif_eligibility_bitmap & (1 << i) != 0 { amounts(&before).per_guardian } else { 0 };
        assert_eq!(f.reward(i).claimable_lamports - before.reward(i).claimable_lamports, expected_credit);
        assert_eq!(f.reward(i).last_active_period, Some(99)); }
}

#[test]
fn zero_active_carry_reapplies_and_never_spends_prior_earned_liabilities() {
    for carry in [0, 1, 2, 19, 1001, 1_000_000] {
        let mut f = Fixture::new(false, 0, true, 17); let old_carry = f.round().kif_carry_input_lamports;
        let mut r = f.round(); r.kif_carry_input_lamports = carry; f.set_round(&r);
        f.custody.edit_config(|c| c.collective_kif_carry_lamports = carry);
        f.custody.accounts[KIF_SOL].lamports = f.custody.accounts[KIF_SOL].lamports - old_carry + carry;
        let n = amounts(&f); assert_eq!(n.zero, (n.kif + carry) / 2); assert_eq!(n.carry, n.kif + carry - n.zero);
        let old = f.custody.config().kif_claim_liability_lamports; let old_rewards = (0..6).map(|i| f.reward(i)).collect::<Vec<_>>();
        success(&mut f); assert_eq!(f.custody.config().kif_claim_liability_lamports, old);
        assert_eq!((0..6).map(|i| f.reward(i)).collect::<Vec<_>>(), old_rewards);
        // A successive valid zero-active round reapplies the same rule to carry.
        let mut next = Fixture::new(false, 0, false, 0); let previous = next.round().kif_carry_input_lamports;
        let mut r = next.round(); r.kif_carry_input_lamports = n.carry; next.set_round(&r);
        next.custody.edit_config(|c| c.collective_kif_carry_lamports = n.carry);
        next.custody.accounts[KIF_SOL].lamports = next.custody.accounts[KIF_SOL].lamports - previous + n.carry;
        let again = amounts(&next); assert_eq!(again.zero, (again.kif + n.carry) / 2); success(&mut next);
    }
}

#[test]
fn severe_pool_loss_commits_only_recovery_and_success_only_overflows_do_not_mask_it() {
    for bitmap in [0, 1, 63] {
        let mut f = Fixture::new(false, bitmap, false, 0); let mut p = f.pool(); p.total_lamports = 1; f.set_pool(&p);
        // Config paid counters and recipient balances would overflow on success,
        // but must not prevent an otherwise valid no-payment recovery outcome.
        f.custody.edit_config(|c| c.cumulative_htfp_paid_lamports = u64::MAX);
        f.custody.accounts[f.base + 1].lamports = u64::MAX;
        let n = amounts(&f); assert_eq!(n.protected, 0); assert!(n.recovery);
        let before = f.clone(); success(&mut f); assert_eq!(f.round().lifecycle, DistributionLifecycle::RecoveryRequired);
        for i in 0..f.custody.accounts.len() { if i != ROUND { assert_eq!(f.custody.accounts[i], before.custody.accounts[i]); } }
        reject(f, Some(error(Piv1Error::RecoveryRequired)));
    }
    // Accepted model validates snapshot reward credit arithmetic BEFORE recovery.
    let mut f = Fixture::new(false, 1, false, 0); let mut p = f.pool(); p.total_lamports = 1; f.set_pool(&p);
    let mut reward = f.reward(0); reward.cumulative_earned = u64::MAX;
    reward.cumulative_claimed = u64::MAX - reward.claimable_lamports; f.set_reward(0, &reward);
    reject(f, Some(error(Piv1Error::ArithmeticOverflow)));
}

#[test]
fn pending_use_and_cooldown_rewards_cannot_cover_historical_hwm_shortfall() {
    let mut f = Fixture::new(false, 1, true, 1_000_000); let mut p = f.pool(); p.total_lamports /= 2; f.set_pool(&p);
    let n = amounts(&f); assert!(n.recovery);
    assert!(n.protected + f.round().cumulative_cooldown_rewards_lamports >= n.hwm);
    success(&mut f); assert_eq!(f.round().lifecycle, DistributionLifecycle::RecoveryRequired);
    let mut f = Fixture::new(false, 1, false, 0); let mut p = f.pool();
    // Search exact input ratio with an independent integer oracle for a case
    // where forgetting U accepts, while correct historical protection fails.
    let original = p.total_lamports;
    let mut found = false;
    for total in (original - 100_000..=original).step_by(100) {
        p.total_lamports = total; f.set_pool(&p); let n = amounts(&f);
        if n.protected < n.hwm && n.protected + f.round().pending_sol_used_lamports >= n.hwm { found = true; break; }
    }
    assert!(found); success(&mut f); assert_eq!(f.round().lifecycle, DistributionLifecycle::RecoveryRequired);
}

#[test]
fn strict_abi_context_recipients_rewards_and_unfinished_rounds_reject_before_effects() {
    let base = Fixture::new(false, 7, true, 0);
    for len in 0..14 { if len == 9 { continue; } let mut d = base.data(); d.resize(len, 0);
        assert_eq!(decode_settle_distribution(&d), Err(ProgramError::InvalidInstructionData)); }
    for byte in [0, 8] { let mut d = base.data(); d[byte] ^= 1;
        assert_eq!(decode_settle_distribution(&d), Err(ProgramError::InvalidInstructionData)); }
    let mut f = base.clone(); let d = f.data();
    f.custody.with_infos(|a| assert_eq!(instruction_boundary::process_instruction(&PROGRAM, a, &d), Err(ProgramError::Custom(HOST_RUNTIME_UNAVAILABLE_CODE))));
    for count in [26, 27, 29] { let mut f = base.clone(); if count == 29 { f.custody.accounts.push(f.custody.accounts[0].clone()); }
        else { f.custody.accounts.truncate(count); } reject(f, None); }
    let mut f = base.clone(); f.custody.edit_config(|c| c.paused = true); reject(f, Some(error(Piv1Error::PausedOperation)));
    for i in [CONFIG, ROUND, PRINCIPAL_SOL, ESCROW_SOL, KIF_SOL, base.base + 1, base.base + 2, base.base + 3, base.base + 8] {
        let mut f = base.clone(); f.custody.accounts[i].writable = false; reject(f, Some(error(Piv1Error::AccountNotWritable))); }
    for i in [AUTHORITY, SYSTEM, TOKEN, base.base + 1, base.base + 2, base.base + 3] {
        let mut f = base.clone(); f.custody.accounts[i].key = key(245); reject(f, None); }
    let mut f = base.clone(); f.custody.accounts[f.base + 1].owner = PROGRAM; reject(f, None);
    let mut f = base.clone(); f.custody.accounts[f.base + 2].data = vec![0]; reject(f, None);
    let mut f = base.clone(); f.custody.accounts[f.base + 1].executable = true; reject(f, None);
    let mut f = base.clone(); f.custody.accounts[f.base + 3].owner = system_program::ID; reject(f, None);
    let mut f = base.clone(); let mut reward = f.reward(0); reward.registry_revision += 1;
    let (key, bump) = Pubkey::find_program_address(&[b"guardian-reward", reward.guardian.as_ref(),
        &reward.registry_revision.to_le_bytes(), &[0]], &PROGRAM);
    reward.bump = bump; f.set_reward(0, &reward); f.custody.accounts[f.base + 3].key = key;
    reject(f, Some(error(Piv1Error::InvalidGuardianSet)));
    let mut f = base.clone(); f.custody.accounts.swap(f.base + 3, f.base + 4); reject(f, Some(error(Piv1Error::InvalidGuardianSet)));
    let mut f = base.clone(); let mut p = f.pool(); p.last_update_epoch -= 1; f.set_pool(&p); reject(f, None);
    let mut f = base.clone(); f.custody.accounts[f.base].data[16] ^= 1; reject(f, None);
    let mut f = base.clone(); f.clock.unix_timestamp = f.round().prepared_at - 1;
    f.custody.accounts[f.base].data[32..40].copy_from_slice(&f.clock.unix_timestamp.to_le_bytes()); reject(f, Some(error(Piv1Error::TimestampRegression)));
    let mut f = base.clone(); f.clock.epoch = f.round().prepared_epoch - 1;
    f.custody.accounts[f.base].data[16..24].copy_from_slice(&f.clock.epoch.to_le_bytes());
    let mut p = f.pool(); p.last_update_epoch = f.clock.epoch; f.set_pool(&p);
    reject(f, Some(error(Piv1Error::TimestampRegression)));
    let mut f = base.clone(); let mut r = f.round(); r.prepared_slot = f.clock.slot + 1;
    assert_eq!(r.validate(), Ok(())); f.set_round(&r); reject(f, Some(error(Piv1Error::TimestampRegression)));
    // A valid initiated-but-unfinalized header has exact pending/carry escrow
    // and zero finalized sums. Settlement must reject lifecycle, not malformed
    // counters or custody. Closed metadata is not an input to this boundary.
    let mut f = base.clone(); let mut r = f.round(); r.lifecycle = DistributionLifecycle::WithdrawalActive;
    r.finalized_leg_count = 0;
    r.cumulative_finalized_delegated_native_lamports = 0;
    r.cumulative_finalized_native_lamports = 0;
    r.cumulative_recovered_stake_rent_lamports = 0;
    r.cumulative_recovered_metadata_rent_lamports = 0;
    r.cumulative_cooldown_rewards_lamports = 0;
    r.cumulative_cooldown_losses_lamports = 0;
    r.recorded_escrow_available_lamports = r.pending_sol_used_lamports + r.prior_next_cycle_yield_used_lamports().unwrap();
    f.custody.accounts[ESCROW_SOL].lamports = f.custody.rent.minimum_balance(0) + r.recorded_escrow_available_lamports;
    assert_eq!(r.validate(), Ok(())); f.set_round(&r);
    reject(f, Some(error(Piv1Error::InvalidLifecycle)));
    let mut f = base.clone(); let mut r = f.round(); r.lifecycle = DistributionLifecycle::WithdrawalActive;
    r.finalized_leg_count -= 1; f.set_round(&r); reject(f, None);
    let mut f = base; let mut r = f.round(); r.cumulative_jitosol_assigned_units -= 1; f.set_round(&r); reject(f, None);
}

#[test]
fn each_custody_obligation_untracked_donation_and_selected_claim_sum_is_enforced() {
    let base = Fixture::new(false, 7, false, 0);
    for i in [PENDING_SOL, PRINCIPAL_SOL, ESCROW_SOL, KIF_SOL] {
        for increase in [false, true] { let mut f = base.clone();
            if increase { f.custody.accounts[i].lamports += 1; } else { f.custody.accounts[i].lamports -= 1; }
            reject(f, None);
        }
    }
    for i in [PRINCIPAL_JITO, PENDING_JITO] { let mut f = base.clone();
        let units = f.custody.token_units(i); f.custody.set_token_units(i, units + 1); reject(f, None); }
    let mut f = base.clone(); let mut reward = f.reward(0); reward.claimable_lamports += 100;
    reward.cumulative_earned += 100; f.set_reward(0, &reward); reject(f, Some(error(Piv1Error::CumulativeReconciliationMismatch)));
    let mut f = base.clone(); let mut reward = f.reward(0); reward.cumulative_earned += 1; f.set_reward(0, &reward); reject(f, None);
    let mut f = base; f.custody.edit_config(|c| c.cumulative_htfp_paid_lamports = u64::MAX);
    reject(f, Some(error(Piv1Error::ArithmeticOverflow)));
}

#[test]
fn every_payment_failure_corruption_and_late_state_borrow_keeps_all_eight_states_uncommitted() {
    for bitmap in [0, 7] {
        let base = Fixture::new(false, bitmap, true, 101); let count = expected(&base).calls.len();
        assert_eq!(count, if bitmap == 0 { 4 } else { 3 }); let mut behaviors = vec![];
        for step in 0..count { behaviors.extend([Behavior::Before(step), Behavior::After(step), Behavior::Noop(step),
            Behavior::Lamports(step, base.base + 1), Behavior::Lamports(step, OPERATIONAL_SOL),
            Behavior::Data(step, POOL, 100), Behavior::Data(step, LIST, 40)]); }
        for i in [CONFIG, ROUND, base.base + 3, base.base + 4, base.base + 5, base.base + 6, base.base + 7, base.base + 8] {
            behaviors.push(Behavior::Hold(i)); }
        for behavior in behaviors {
            let mut failed = base.clone(); let (result, _, events) = raw(&mut failed, behavior);
            assert!(result.is_err(), "{behavior:?}"); assert!(events.is_empty());
            for i in [CONFIG, ROUND, base.base + 3, base.base + 4, base.base + 5, base.base + 6, base.base + 7, base.base + 8] {
                assert_eq!(failed.custody.accounts[i], base.custody.accounts[i]);
            }
            // Raw effects can include earlier payments. Only explicit transaction
            // discard returns to the original world; this is not a Bank proof.
            let mut retry = base.clone(); success(&mut retry);
        }
    }
}

#[test]
fn key_and_backing_aliases_and_all_preflight_state_borrows_reject_without_cpi() {
    let base = Fixture::new(false, 7, false, 0);
    for pair in [(ESCROW_SOL, base.base + 1), (KIF_SOL, PRINCIPAL_SOL), (base.base + 1, base.base + 2),
        (base.base + 3, base.base + 4), (CONFIG, ROUND)] {
        let mut f = base.clone(); f.custody.accounts[pair.1].key = f.custody.accounts[pair.0].key;
        reject(f, Some(error(Piv1Error::AccountAlias)));
        for lamports in [false, true] {
            let mut f = base.clone(); let clock = f.clock.clone(); let rent = f.custody.rent.clone(); let data = f.data();
            f.custody.with_infos(|a| { let mut aliased = a.to_vec();
                if lamports { aliased[pair.1].lamports = a[pair.0].lamports.clone(); }
                else { aliased[pair.1].data = a[pair.0].data.clone(); }
                assert_eq!(execute(&PROGRAM, &aliased, &data, || Ok(clock), || Ok(rent), |_, _, _| panic!(), |_| panic!()),
                    Err(error(Piv1Error::AccountAlias))); });
        }
    }
    for account in [CONFIG, ROUND, PRINCIPAL_SOL, ESCROW_SOL, KIF_SOL, base.base + 1, base.base + 2,
        base.base + 3, base.base + 4, base.base + 5, base.base + 6, base.base + 7, base.base + 8] {
        let mut f = base.clone(); let clock = f.clock.clone(); let rent = f.custody.rent.clone(); let data = f.data();
        f.custody.with_infos(|a| { let _held = a[account].try_borrow_data().unwrap();
            assert!(execute(&PROGRAM, a, &data, || Ok(clock), || Ok(rent), |_, _, _| panic!(), |_| panic!()).is_err()); });
        assert_eq!(f, base);
    }
}
