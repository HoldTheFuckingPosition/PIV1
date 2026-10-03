//! Production finalization host effects; no actual Stake readiness/Bank rollback.
mod support;
#[path = "support/pending_custody.rs"] pub mod pending_custody;
#[allow(dead_code)] #[path = "support/economic_custody.rs"] mod custody;
#[path = "support/jito_identity_oracle.rs"] mod oracle;
#[allow(dead_code)] #[path = "support/bootstrap_custody.rs"] mod bootstrap_custody;
#[allow(dead_code)] #[path = "support/distribution_preparation_custody.rs"] mod preparation_custody;
#[allow(dead_code)] #[path = "support/withdrawal_preparation_custody.rs"] mod withdrawal_custody;
#[allow(dead_code)] #[path = "support/withdrawal_leg_custody.rs"] mod leg_custody;
#[path = "support/withdrawal_finalization_custody.rs"] mod final_custody;
use support::kif_claim_custody;
use piv1::integrations;
use anchor_lang::{prelude::Pubkey, solana_program::{program_error::ProgramError, system_program}};
use piv1::{errors::Piv1Error, events::WithdrawalLegFinalized, instruction_boundary,
    instruction_errors::{piv1_error_code, HOST_RUNTIME_UNAVAILABLE_CODE},
    instructions::finalize_withdrawal_leg::*,
    state::DistributionLifecycle,
    constants::{RECOVERY_FLAG_COOLDOWN_LOSS, RECOVERY_FLAG_RESIDUAL_HWM},
    withdrawal_finalization_execution::process_instruction_with_host_callbacks as execute};
use solana_stake_interface::state::StakeStateV2;
use custody::*;
use bootstrap_custody::{POOL, LIST};
use final_custody::{Fixture, expected};
use leg_custody::{emulate, snapshot};
use support::kif_claim_custody::{key, PROGRAM};
fn error(e: Piv1Error) -> ProgramError { ProgramError::Custom(piv1_error_code(e)) }
#[derive(Clone, Copy, Debug)]
enum Behavior { Good, Before(usize), After(usize), Noop(usize), Data(usize, usize, usize),
    Lamports(usize, usize), HoldData(usize), HoldLamports(usize) }
fn raw(f: &mut Fixture, behavior: Behavior) -> (Result<(), ProgramError>, usize, Vec<WithdrawalLegFinalized>) {
    let reference = expected(f); let initial = f.custody.accounts.clone();
    let data = f.data(); let clock = f.clock.clone(); let rent = f.custody.rent.clone();
    let mut count = 0; let mut events = vec![];
    let (result, accounts) = f.custody.with_infos(|all| {
        let mut held_data = None; let mut held_lamports = None;
        let result = execute(&PROGRAM, all, &data, || Ok(clock), || Ok(rent), |ix, infos, seeds| {
            let i = count; count += 1;
            if matches!(behavior, Behavior::Before(j) if j == i) { return Err(ProgramError::Custom(24801)); }
            if matches!(behavior, Behavior::Noop(j) if j == i) { return Ok(()); }
            let before = if i == 0 { &initial } else { &reference.calls[i - 1].after };
            emulate(before, &reference.calls[i], ix, infos, seeds, all)?;
            if matches!(behavior, Behavior::After(j) if j == i) { return Err(ProgramError::Custom(24802)); }
            if let Behavior::Data(j, account, offset) = behavior { if i == j { all[account].try_borrow_mut_data()?[offset] ^= 1; } }
            if let Behavior::Lamports(j, account) = behavior { if i == j { **all[account].try_borrow_mut_lamports()? += 1; } }
            if i == 1 {
                if let Behavior::HoldData(account) = behavior { held_data = Some(all[account].try_borrow_data()?); }
                if let Behavior::HoldLamports(account) = behavior { held_lamports = Some(all[account].try_borrow_lamports()?); }
            }
            Ok(())
        }, |e| events.push(e));
        drop(held_data); drop(held_lamports);
        (result, all.iter().map(snapshot).collect::<Vec<_>>())
    });
    f.custody.accounts = accounts;
    (result, count, events)
}
fn success(f: &mut Fixture) {
    let before = f.clone(); let expected = expected(f);
    let (result, calls, events) = raw(f, Behavior::Good);
    assert_eq!(result, Ok(())); assert_eq!(calls, 2); assert_eq!(events, vec![expected.event]);
    assert_eq!(*f, expected.after);
    let total = |f: &Fixture| f.custody.accounts.iter().map(|a| u128::from(a.lamports)).sum::<u128>();
    assert_eq!(total(&before), total(f));
    let old = before.custody.config(); let new = f.custody.config();
    assert_eq!(new.protected_principal_hwm_lamports, old.protected_principal_hwm_lamports);
    assert_eq!(new.accounted_historical_sol_lamports, old.accounted_historical_sol_lamports);
    assert_eq!(new.accounted_historical_jitosol_units, old.accounted_historical_jitosol_units);
    assert_eq!(new.next_cycle_yield_lamports, old.next_cycle_yield_lamports);
    assert_eq!(new.kif_claim_liability_lamports, old.kif_claim_liability_lamports);
    assert_eq!(new.collective_kif_carry_lamports, old.collective_kif_carry_lamports);
}
fn reject(mut f: Fixture, expected_error: Option<ProgramError>) {
    let before = f.clone(); let clock = f.clock.clone(); let rent = f.custody.rent.clone(); let data = f.data();
    let result = f.custody.with_infos(|a| execute(&PROGRAM, a, &data, || Ok(clock), || Ok(rent),
        |_, _, _| panic!("preflight rejection must precede CPI"), |_| panic!("no failure event")));
    if let Some(e) = expected_error { assert_eq!(result, Err(e)); } else { assert!(result.is_err()); }
    assert_eq!(f, before);
}

#[test]
fn exact_whole_state_reward_rent_and_metadata_donation_oracles() {
    let mut cases = 0;
    for shared in [false, true] { for reward in [0, 1, 101, 100_000] { for donation in [0, 1, 7_000_000] {
        let mut f = Fixture::new(shared); f.reward(reward);
        let leg = f.leg(); let before = f.custody.config(); let op = f.custody.accounts[OPERATIONAL_SOL].lamports;
        f.custody.accounts[f.base + 3].lamports += donation;
        f.custody.accounts[PRINCIPAL_JITO].lamports += 31; f.custody.accounts[PENDING_JITO].lamports += 41;
        success(&mut f);
        assert_eq!(f.round().lifecycle, DistributionLifecycle::EscrowFunded);
        assert_eq!(f.round().cumulative_cooldown_rewards_lamports, reward);
        assert_eq!(f.custody.config().accounted_pending_sol_lamports, before.accounted_pending_sol_lamports + donation);
        assert_eq!(f.round().pending_sol_used_lamports, 321);
        assert_eq!(f.custody.accounts[OPERATIONAL_SOL].lamports - op,
            leg.stake_rent_advanced_lamports + leg.metadata_rent_advanced_lamports);
        assert_eq!(f.custody.accounts[f.base + 3].lamports, 0);
        assert!(f.custody.accounts[f.base + 3].data.iter().all(|b| *b == 0));
        assert_eq!(f.custody.accounts[f.base + 4].lamports, 0);
        assert!(f.custody.accounts[f.base + 4].data.is_empty()); cases += 1;
    }}}
    assert_eq!(cases, 24);
}

#[test]
fn actual_native_losses_and_residual_hwm_failures_commit_recovery() {
    for loss in [0, 1, 77, 1000] { for hwm_loss in [false, true] {
        if loss == 0 && !hwm_loss { continue; }
        let mut f = Fixture::new(false); f.custody.accounts[f.base + 4].lamports -= loss;
        if hwm_loss { let mut p = f.pool(); p.total_lamports /= 2; f.set_pool(&p); }
        let r = f.round(); success(&mut f);
        assert_eq!(f.round().lifecycle, DistributionLifecycle::RecoveryRequired);
        assert_eq!(f.round().cumulative_cooldown_losses_lamports, loss);
        assert_eq!(f.round().cumulative_cooldown_rewards_lamports, 0);
        assert_eq!(f.round().recovery_flags,
            if loss > 0 { RECOVERY_FLAG_COOLDOWN_LOSS } else { 0 }
            | if hwm_loss { RECOVERY_FLAG_RESIDUAL_HWM } else { 0 });
        assert_eq!(f.round().proposed_hwm_after_settlement_lamports, r.proposed_hwm_after_settlement_lamports);
        reject(f, Some(error(Piv1Error::RecoveryRequired)));
    }}
}

#[test]
fn original_rent_is_recovered_under_current_rent_changes_and_ambiguous_stake_excess_rejects() {
    let mut f = Fixture::new(false); let old = f.custody.rent.clone(); let leg = f.leg();
    f.custody.rent.lamports_per_byte_year += 1;
    for i in [CONFIG, ROUND, PENDING_SOL, PRINCIPAL_SOL, OPERATIONAL_SOL, ESCROW_SOL, KIF_SOL,
        PRINCIPAL_JITO, PENDING_JITO, MINT] {
        let len = f.custody.accounts[i].data.len();
        f.custody.accounts[i].lamports += f.custody.rent.minimum_balance(len) - old.minimum_balance(len);
    }
    assert_ne!(leg.stake_rent_advanced_lamports, f.custody.rent.minimum_balance(200));
    success(&mut f); assert_eq!(f.round().cumulative_recovered_stake_rent_lamports, leg.stake_rent_advanced_lamports);
    let base = Fixture::new(false);
    for extra in [1, 1_000_000] { let mut f = base.clone(); f.custody.accounts[f.base + 4].lamports += extra;
        reject(f, Some(error(Piv1Error::InvalidCustodyObservation))); }
    // Same ambiguity can come from legitimate rent-adjusted delegation. It is
    // unsupported, never mislabeled as both a loss and a pending contribution.
    let mut f = base.clone(); let StakeStateV2::Stake(meta, mut stake, flags) = f.stake() else { panic!() };
    stake.delegation.stake -= 1; f.set_stake(&StakeStateV2::Stake(meta, stake, flags));
    reject(f, Some(error(Piv1Error::InvalidCustodyObservation)));
    let mut f = base.clone(); f.custody.accounts[f.base + 4].lamports = f.leg().stake_rent_advanced_lamports - 1;
    reject(f, Some(error(Piv1Error::InvalidCustodyObservation)));
    let mut f = base; f.custody.accounts[f.base + 3].lamports = f.leg().metadata_rent_advanced_lamports - 1;
    reject(f, Some(error(Piv1Error::AccountRentDeficit)));
}

#[test]
fn partial_target_finalization_does_not_claim_complete_and_zero_stake_is_supported() {
    let mut leg = leg_custody::Fixture::new(false); leg.source_capacity(4000);
    let mut f = Fixture::from_leg(leg); success(&mut f);
    assert_eq!(f.round().lifecycle, DistributionLifecycle::WithdrawalActive);
    assert_eq!(f.round().successful_leg_count, f.round().finalized_leg_count);
    assert!(f.round().cumulative_jitosol_assigned_units < f.round().fixed_jitosol_withdrawal_target_units);
    // Refund revival does not recreate an initiated metadata envelope.
    f.custody.accounts[f.base + 3].lamports = f.custody.rent.minimum_balance(piv1::state::WithdrawalLeg::SPACE);
    reject(f, None);
    let mut f = Fixture::new(false); let StakeStateV2::Stake(meta, mut stake, flags) = f.stake() else { panic!() };
    stake.delegation.stake = 0; f.set_stake(&StakeStateV2::Stake(meta, stake, flags));
    f.custody.accounts[f.base + 4].lamports = f.leg().stake_rent_advanced_lamports;
    success(&mut f); assert_eq!(f.round().lifecycle, DistributionLifecycle::RecoveryRequired);
    // Instant activation/deactivation is a valid Stake state. The callee, not
    // an epoch-wait heuristic, decides full withdrawal readiness.
    let mut f = Fixture::new(false); let epoch = f.leg().initiation_epoch; f.set_epoch(epoch);
    let StakeStateV2::Stake(meta, mut stake, flags) = f.stake() else { panic!() };
    stake.delegation.activation_epoch = epoch; f.set_stake(&StakeStateV2::Stake(meta, stake, flags)); success(&mut f);
}

#[test]
fn strict_abi_identity_pause_custody_and_fresh_context_rejections_are_atomic() {
    let base = Fixture::new(false);
    for len in 0..21 { if len == 17 { continue; } let mut d = base.data(); d.resize(len, 0);
        assert_eq!(decode_finalize_withdrawal_leg(&d), Err(ProgramError::InvalidInstructionData)); }
    for byte in [0, 8] { let mut d = base.data(); d[byte] ^= 1;
        assert_eq!(decode_finalize_withdrawal_leg(&d), Err(ProgramError::InvalidInstructionData)); }
    let mut f = base.clone(); let data = f.data();
    f.custody.with_infos(|a| assert_eq!(instruction_boundary::process_instruction(&PROGRAM, a, &data),
        Err(ProgramError::Custom(HOST_RUNTIME_UNAVAILABLE_CODE))));
    for count in [22, 23, 25] { let mut f = base.clone(); if count == 25 { f.custody.accounts.push(f.custody.accounts[0].clone()); }
        else { f.custody.accounts.truncate(count); } reject(f, None); }
    let mut f = base.clone(); f.custody.edit_config(|c| c.paused = true); reject(f, Some(error(Piv1Error::PausedOperation)));
    for i in [CONFIG, ROUND, PENDING_SOL, OPERATIONAL_SOL, ESCROW_SOL, base.base + 3, base.base + 4] {
        let mut f = base.clone(); f.custody.accounts[i].writable = false; reject(f, Some(error(Piv1Error::AccountNotWritable))); }
    for i in [PENDING_SOL, PRINCIPAL_SOL, ESCROW_SOL, KIF_SOL] { let mut f = base.clone(); f.custody.accounts[i].lamports -= 1; reject(f, None); }
    let mut f = base.clone(); f.custody.accounts[PRINCIPAL_JITO].data[64..72].copy_from_slice(&1_u64.to_le_bytes()); reject(f, None);
    let mut f = base.clone(); let mut p = f.pool(); p.last_update_epoch -= 1; f.set_pool(&p); reject(f, None);
    for i in [AUTHORITY, SYSTEM, TOKEN, base.base + 1, base.base + 2, base.base + 3, base.base + 4] {
        let mut f = base.clone(); f.custody.accounts[i].key = key(244); reject(f, None); }
    let mut f = base.clone(); f.custody.accounts[base.base + 2].owner = system_program::ID; reject(f, None);
    let mut f = base.clone(); f.custody.accounts[base.base + 2].data[..8].copy_from_slice(&513_u64.to_le_bytes()); reject(f, None);
    let mut f = base.clone(); f.custody.accounts[base.base + 2].data[8..16].copy_from_slice(&f.clock.epoch.to_le_bytes()); reject(f, None);
    let mut f = base.clone(); f.custody.accounts[base.base].data[16] ^= 1; reject(f, None);
    let mut f = base.clone(); f.custody.accounts[base.base + 3].data[0] ^= 1; reject(f, None);
    let mut f = base.clone(); f.index = 1; reject(f, Some(error(Piv1Error::LegIndexMismatch)));
    let mut f = base.clone(); let mut l = f.leg(); l.sequence += 1; f.set_leg(&l); reject(f, None);
    let mut f = base; let mut l = f.leg(); l.stake_bump ^= 1; f.set_leg(&l); reject(f, None);
}

#[test]
fn stake_authority_voter_lockup_and_deactivation_authentication_precedes_cpi() {
    let base = Fixture::new(false);
    for case in 0..7 {
        let mut f = base.clone(); let StakeStateV2::Stake(mut meta, mut stake, flags) = f.stake() else { panic!() };
        match case {
            0 => meta.authorized.staker = key(244), 1 => meta.authorized.withdrawer = key(244),
            2 => stake.delegation.voter_pubkey = key(244), 3 => stake.delegation.deactivation_epoch = u64::MAX,
            4 => stake.delegation.deactivation_epoch += 1, 5 => meta.lockup.epoch = f.clock.epoch + 1,
            _ => meta.lockup.unix_timestamp = f.clock.unix_timestamp + 1,
        }
        f.set_stake(&StakeStateV2::Stake(meta, stake, flags)); reject(f, None);
    }
    let mut f = base.clone(); f.set_epoch(f.leg().initiation_epoch - 1); reject(f, None);
    let mut f = base; f.custody.accounts[f.base + 4].owner = system_program::ID; reject(f, None);
}

#[test]
fn every_cpi_failure_noop_corruption_and_final_borrow_keeps_state_and_closure_uncommitted() {
    let mut base = Fixture::new(false); base.custody.accounts[base.base + 3].lamports += 101;
    let mut behaviors = vec![];
    for step in 0..2 { behaviors.extend([Behavior::Before(step), Behavior::After(step), Behavior::Noop(step),
        Behavior::Lamports(step, KIF_SOL), Behavior::Data(step, POOL, 100), Behavior::Data(step, base.base + 3, 20)]); }
    for account in [CONFIG, ROUND, base.base + 3] { behaviors.push(Behavior::HoldData(account)); }
    for account in [OPERATIONAL_SOL, PENDING_SOL, base.base + 3] { behaviors.push(Behavior::HoldLamports(account)); }
    for behavior in behaviors {
        let mut failed = base.clone(); let (result, _, events) = raw(&mut failed, behavior);
        assert!(result.is_err(), "{behavior:?}"); assert!(events.is_empty());
        assert_eq!(failed.custody.accounts[CONFIG].data, base.custody.accounts[CONFIG].data);
        assert_eq!(failed.custody.accounts[ROUND].data, base.custody.accounts[ROUND].data);
        if !matches!(behavior, Behavior::Data(_, _, _)) {
            assert_eq!(failed.custody.accounts[base.base + 3].data, base.custody.accounts[base.base + 3].data);
        }
        assert_eq!(failed.custody.accounts[base.base + 3].lamports, base.custody.accounts[base.base + 3].lamports);
        // Caller/runtime transaction discard is explicit. The raw failed world
        // can contain already-drained Stake/credited escrow, so is not rollback.
        let mut retry = base.clone(); success(&mut retry);
    }
}

#[test]
fn key_backing_aliases_and_preflight_borrows_reject_before_any_effect() {
    let base = Fixture::new(false);
    for pair in [(PENDING_SOL, OPERATIONAL_SOL), (base.base + 3, base.base + 4), (CONFIG, ROUND)] {
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
    for account in [CONFIG, ROUND, PENDING_SOL, OPERATIONAL_SOL, ESCROW_SOL, base.base + 3, base.base + 4] {
        let mut f = base.clone(); let clock = f.clock.clone(); let rent = f.custody.rent.clone(); let data = f.data();
        f.custody.with_infos(|a| { let _held = a[account].try_borrow_data().unwrap();
            assert!(execute(&PROGRAM, a, &data, || Ok(clock), || Ok(rent), |_, _, _| panic!(), |_| panic!()).is_err()); });
        assert_eq!(f, base);
    }
}

#[test]
fn out_of_order_two_leg_completion_reconciles_once_and_closed_metadata_cannot_replay() {
    for reverse in [false, true] {
        let mut f = leg_custody::Fixture::new(false); f.source_capacity(4000);
        f.custody.accounts[OPERATIONAL_SOL].lamports += f.custody.rent.minimum_balance(200)
            + f.custody.rent.minimum_balance(piv1::state::WithdrawalLeg::SPACE);
        let mut list = f.custody.accounts[LIST].data.clone();
        list[1..5].copy_from_slice(&2_u32.to_le_bytes()); list[5..9].copy_from_slice(&2_u32.to_le_bytes());
        let mut second = list[9..82].to_vec(); second[41..73].copy_from_slice(key(203).as_ref()); list.extend(second);
        f.custody.accounts[LIST].data = list; f.custody.accounts[LIST].lamports = f.custody.rent.minimum_balance(155);
        let mut f = leg_custody::expected(&f).after;
        let first_pair = f.custody.accounts[f.base + 4..].to_vec();
        let StakeStateV2::Stake(meta, mut stake, flags) = f.source() else { panic!() };
        stake.delegation.voter_pubkey = key(203); stake.delegation.stake = 1_004_000;
        f.set_source(StakeStateV2::Stake(meta, stake, flags));
        f.custody.accounts[f.base + 2].key = Pubkey::find_program_address(&[key(203).as_ref(),
            f.custody.config().stake_pool.as_ref()], &f.custody.config().stake_pool_program).0;
        f.custody.accounts[f.base + 2].lamports = meta.rent_exempt_reserve + 1_004_000;
        f.index = 1; f.new_pair();
        let mut f = Fixture::from_leg(f); let b = f.base;
        let second_pair = f.custody.accounts[b + 3..].to_vec();
        assert_eq!(f.index, 1); assert_eq!(f.round().successful_leg_count, 2);
        assert_eq!(f.round().cumulative_jitosol_assigned_units, f.round().fixed_jitosol_withdrawal_target_units);
        if !reverse { f.custody.accounts[b + 3..].clone_from_slice(&first_pair); f.index = 0; }
        let r = f.round(); success(&mut f);
        assert_eq!(f.round().lifecycle, DistributionLifecycle::WithdrawalActive);
        assert_eq!(f.round().finalized_leg_count, 1);
        let once = f.clone(); reject(once, None);
        let pending_pair = if reverse { &first_pair } else { &second_pair };
        f.custody.accounts[b + 3..].clone_from_slice(pending_pair); f.index = if reverse { 0 } else { 1 };
        success(&mut f);
        assert_eq!(f.round().lifecycle, DistributionLifecycle::EscrowFunded);
        assert_eq!(f.round().finalized_leg_count, 2);
        assert_eq!(f.round().cumulative_finalized_delegated_native_lamports, r.cumulative_delegated_native_lamports);
        assert_eq!(f.round().cumulative_cooldown_rewards_lamports, 0);
        assert_eq!(f.round().cumulative_cooldown_losses_lamports, 0);
        reject(f, Some(error(Piv1Error::InvalidLifecycle)));
    }
}

#[test]
fn bounded_history_padding_and_checked_balance_boundaries() {
    let base = Fixture::new(false);
    let mut f = base.clone(); f.custody.accounts[f.base + 2].data.resize(16_392, 0); success(&mut f);
    let mut f = base.clone(); f.custody.accounts[f.base + 2].data.resize(16_392, 0);
    f.custody.accounts[f.base + 2].data[16_391] = 1; reject(f, None);
    let mut f = base.clone(); let entry = f.custody.accounts[f.base + 2].data[8..].to_vec();
    f.custody.accounts[f.base + 2].data[..8].copy_from_slice(&2_u64.to_le_bytes());
    f.custody.accounts[f.base + 2].data.extend(entry); reject(f, None);
    let mut f = base.clone(); f.custody.accounts[f.base + 2].data.resize(16_393, 0); reject(f, None);
    let mut f = base.clone(); f.custody.accounts[OPERATIONAL_SOL].lamports = u64::MAX; reject(f, Some(error(Piv1Error::ArithmeticOverflow)));
    // MAX at the source alone is valid: the original rent debit leaves enough
    // room for the pending credit. Prove both sides of the actual sum boundary.
    let mut f = base.clone(); f.custody.accounts[f.base + 3].lamports = u64::MAX; success(&mut f);
    let mut f = base.clone(); let metadata_rent = f.leg().metadata_rent_advanced_lamports;
    f.custody.edit_config(|c| c.accounted_pending_sol_lamports += metadata_rent);
    f.custody.accounts[PENDING_SOL].lamports += metadata_rent;
    f.custody.accounts[f.base + 3].lamports = u64::MAX;
    assert!(u128::from(f.custody.accounts[PENDING_SOL].lamports)
        + u128::from(u64::MAX - metadata_rent) > u128::from(u64::MAX));
    reject(f, Some(error(Piv1Error::ArithmeticOverflow)));
    let mut f = base; let StakeStateV2::Stake(meta, mut stake, flags) = f.stake() else { panic!() };
    stake.delegation.stake = u64::MAX; f.set_stake(&StakeStateV2::Stake(meta, stake, flags));
    f.custody.accounts[f.base + 4].lamports = u64::MAX; reject(f, Some(error(Piv1Error::ArithmeticOverflow)));
}
