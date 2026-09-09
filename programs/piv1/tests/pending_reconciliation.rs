mod support;
#[path = "support/pending_custody.rs"]
pub mod pending_custody;
use support::kif_claim_custody;
use anchor_lang::{prelude::AccountInfo, solana_program::{program_error::ProgramError, program_option::COption, program_pack::Pack}};
use piv1::{errors::Piv1Error, pending_accounts::authenticate_pending_accounts,
    pending_reconciliation::execute_pending_reconciliation,
    instruction_boundary::{process_instruction, process_instruction_with_host_callbacks},
    instruction_errors::{piv1_error_code, HOST_RUNTIME_UNAVAILABLE_CODE},
    instructions::reconcile_pending::*, integrations::FeeFraction,
    state::{DistributionLifecycle, PendingReconciliationResult}};
use pending_custody::*;
use support::{kif_claim_custody::{PROGRAM, key},
    vault_custody_model::{World, PENDING, PENDING_TOKEN}};
use spl_token::state::{Account as TokenAccount, AccountState};

fn world_fixture(w: &World) -> Fixture {
    Fixture::from_state(w.config.clone(), w.round.clone(), w.spendable(PENDING).unwrap(), w.tokens[PENDING_TOKEN])
}
fn fixture() -> Fixture { world_fixture(&World::new(50, 20, 0, 7, 3, FeeFraction::ZERO, 100_000)) }
fn execute(f: &mut Fixture) -> Result<PendingReconciliationResult, Piv1Error> {
    let rent = f.rent.clone(); f.with_infos(|a| execute_pending_reconciliation(&PROGRAM, &rent, roles(a)))
}
fn assert_success(f: &mut Fixture, sol: u64, tokens: u64) {
    let before = f.clone(); let old = f.config();
    let result = execute(f).unwrap();
    assert_eq!(result.newly_accounted_sol_lamports, sol);
    assert_eq!(result.newly_accounted_jitosol_units, tokens);
    let mut expected = before;
    expected.edit_config(|c| { c.accounted_pending_sol_lamports = old.accounted_pending_sol_lamports + sol;
        c.accounted_pending_jitosol_units = old.accounted_pending_jitosol_units + tokens; });
    assert_eq!(*f, expected); f.validate_audit();
}
fn rejects(mut f: Fixture, expected: Piv1Error) {
    let before = f.clone(); assert_eq!(execute(&mut f), Err(expected)); assert_eq!(f, before);
}
fn dispatch(a: &[AccountInfo<'_>], data: &[u8], rent: anchor_lang::prelude::Rent) -> Result<(), ProgramError> {
    process_instruction_with_host_callbacks(&PROGRAM, a, data, || Ok(rent),
        |_,_,_| panic!("pending recognition must never invoke"), |_| panic!("no claim event"))
}

#[test]
fn joint_sol_only_token_only_and_sequential_noops_preserve_original_audit() {
    for (sol, tokens) in [(0, 0), (17, 0), (0, 13), (17, 13)] {
        let mut f = fixture(); let baseline = (f.initial_native, f.initial_tokens);
        f.donate_native(SOL, sol); f.donate_tokens(tokens); assert_success(&mut f, sol, tokens);
        assert_success(&mut f, 0, 0); assert_success(&mut f, 0, 0);
        f.donate_native(SOL, 3); f.donate_tokens(2); assert_success(&mut f, 3, 2);
        assert_eq!((f.initial_native, f.initial_tokens), baseline);
    }
}
#[test]
fn token_native_excess_is_visible_unclassified_and_never_blocks_recognition() {
    for excess in [1, 1_000_000] {
        let mut f = fixture(); f.donate_native(TOKEN, excess); f.donate_native(SOL, 5); f.donate_tokens(9);
        let rent = f.rent.clone(); let before = f.clone();
        let auth = f.with_infos(|a| authenticate_pending_accounts(&PROGRAM, &rent, roles(a))).unwrap();
        assert_eq!(auth.pending_jito().native.lamports, before.accounts[TOKEN].lamports);
        assert_eq!(auth.pending_jito().native.economic_lamports().unwrap(), excess);
        assert_eq!(f, before); assert_success(&mut f, 5, 9);
    }
}
#[test]
fn paused_idle_and_genuine_prepared_settled_recovery_offsets_are_preserved() {
    let mut w = World::new(100_000, 20, 0, 7, 3, FeeFraction::ZERO, 100_000);
    let mut idle = world_fixture(&w); idle.edit_config(|c| c.paused = true);
    idle.donate_native(SOL, 6); assert_success(&mut idle, 6, 0);
    w.open(1_000_000).unwrap(); assert!(w.round.pending_sol_used_lamports > 0);
    for paused in [false, true] {
        let mut f = world_fixture(&w); f.edit_config(|c| c.paused = paused);
        f.donate_native(SOL, 8); f.donate_tokens(4); assert_success(&mut f, 8, 4);
    }
    // A genuine liquid-funded round has no withdrawal leg and can settle.
    w.settle().unwrap(); assert_eq!(w.round.lifecycle, DistributionLifecycle::Settled);
    let mut settled = world_fixture(&w); settled.donate_native(SOL, 10); assert_success(&mut settled, 10, 0);
    let mut loss = World::new(4_000, 20, 0, 7, 3, FeeFraction::ZERO, 100_000);
    loss.open(1_000_000).unwrap(); let leg = loss.initiate(1).unwrap();
    loss.pool.decrease_exchange_rate(loss.pool.raw_snapshot().total_pool_lamports - 1).unwrap();
    loss.advance_epoch().unwrap(); loss.finalize(leg).unwrap();
    assert_eq!(loss.round.lifecycle, DistributionLifecycle::RecoveryRequired);
    let mut recovery = world_fixture(&loss); recovery.edit_config(|c| c.paused = true);
    recovery.donate_native(SOL, 11); recovery.donate_tokens(2); assert_success(&mut recovery, 11, 2);
}
#[test]
fn either_asset_deficit_rejects_both_updates_and_rent_floors_hold() {
    let mut sol = fixture(); sol.accounts[SOL].lamports -= 1; sol.donate_tokens(10); rejects(sol, Piv1Error::PendingCustodyDeficit);
    let mut tok = fixture(); tok.edit_config(|c| c.accounted_pending_jitosol_units += 1); tok.donate_native(SOL, 10); rejects(tok, Piv1Error::PendingCustodyDeficit);
    for index in 0..4 { let mut f = fixture(); f.accounts[index].lamports = f.rent.minimum_balance(f.accounts[index].data.len()) - 1;
        rejects(f, Piv1Error::AccountRentDeficit); }
}
#[test]
fn checked_offset_overflow_rejects_before_either_ledger_changes() {
    let mut w = World::new(100_000, 20, 0, 7, 3, FeeFraction::ZERO, 100_000); w.open(1_000_000).unwrap();
    let mut f = world_fixture(&w); f.edit_config(|c| c.accounted_pending_sol_lamports = u64::MAX);
    // Explicit synthetic trusted-Rent arithmetic boundary. With zero native
    // floor, physical u64::MAX plus the committed offset cannot fit the ledger.
    f.rent.exemption_threshold = 0.0;
    f.accounts[SOL].lamports = u64::MAX; f.donate_tokens(1);
    rejects(f, Piv1Error::ArithmeticOverflow);
}
#[test]
fn keys_owners_bumps_envelopes_and_readonly_config_reject_atomically() {
    let mut f = fixture(); f.accounts[CONFIG].writable = false; rejects(f, Piv1Error::AccountNotWritable);
    for index in 0..4 {
        let mut f = fixture(); f.accounts[index].key = key(201); rejects(f, Piv1Error::InvalidAccountPda);
        let mut f = fixture(); f.accounts[index].owner = key(202); rejects(f, Piv1Error::InvalidAccountOwner);
        let mut f = fixture(); f.accounts[index].executable = true; rejects(f, Piv1Error::ExecutableAccount);
    }
    for index in [CONFIG, ROUND] {
        let mut f = fixture(); f.accounts[index].data[0] ^= 1; rejects(f, Piv1Error::InvalidAccountDiscriminator);
        let mut f = fixture(); f.accounts[index].data[8] = 0; rejects(f, Piv1Error::InvalidVersion);
        let mut f = fixture(); f.accounts[index].data.push(0); rejects(f, Piv1Error::InvalidAccountSize);
        let mut f = fixture(); *f.accounts[index].data.last_mut().unwrap() = 1; rejects(f, Piv1Error::InvalidAccountData);
    }
    let mut f = fixture(); f.edit_config(|c| c.bumps.pending_sol_vault ^= 1); rejects(f, Piv1Error::InvalidAccountPda);
    let mut f = fixture(); f.accounts[ROUND].data[9] ^= 1; rejects(f, Piv1Error::InvalidAccountPda);
}
#[test]
fn token_custody_checks_are_reused_without_weakening() {
    for edit in 0..7 {
        let mut f = fixture(); let mut token = TokenAccount::unpack(&f.accounts[TOKEN].data).unwrap();
        match edit { 0 => token.mint = key(205), 1 => token.owner = key(205),
            2 => token.state = AccountState::Frozen, 3 => token.delegate = COption::Some(key(205)),
            4 => token.delegated_amount = 1, 5 => token.close_authority = COption::Some(key(205)),
            _ => token.is_native = COption::Some(1) }
        TokenAccount::pack(token, &mut f.accounts[TOKEN].data).unwrap(); rejects(f, Piv1Error::InvalidTokenCustody);
    }
}
#[test]
fn alias_and_borrow_failures_precede_every_write() {
    for left in 0..4 { for right in left + 1..4 {
        let mut f = fixture(); f.accounts[right].key = f.accounts[left].key; rejects(f, Piv1Error::AccountAlias);
    }}
    for index in 0..4 {
        let mut f = fixture(); f.donate_native(SOL, 5); let before = f.clone(); let rent = f.rent.clone();
        f.with_infos(|a| { let _held = a[index].try_borrow_mut_data().unwrap();
            assert_eq!(dispatch(a, &encode_reconcile_pending(), rent), Err(ProgramError::Custom(piv1_error_code(Piv1Error::AccountBorrowFailed)))); });
        assert_eq!(f, before);
        let rent = f.rent.clone(); f.with_infos(|a| { let _held = a[index].try_borrow_mut_lamports().unwrap();
            assert_eq!(dispatch(a, &encode_reconcile_pending(), rent), Err(ProgramError::Custom(piv1_error_code(Piv1Error::AccountBorrowFailed)))); });
        assert_eq!(f, before);
    }
    let mut f = fixture(); f.donate_native(SOL, 5); let before = f.clone(); let rent = f.rent.clone();
    f.with_infos(|a| { let mut alias = a.clone(); alias[ROUND].data = alias[CONFIG].data.clone();
        assert_eq!(execute_pending_reconciliation(&PROGRAM, &rent, roles(&alias)), Err(Piv1Error::AccountAlias)); }); assert_eq!(f, before);
    let rent = f.rent.clone(); f.with_infos(|a| { let _read = a[CONFIG].try_borrow_data().unwrap();
        assert_eq!(execute_pending_reconciliation(&PROGRAM, &rent, roles(a)), Err(Piv1Error::AccountBorrowFailed)); }); assert_eq!(f, before);
}
#[test]
fn exact_discriminator_counts_host_boundary_and_privilege_union() {
    assert_eq!(encode_reconcile_pending(), [225,128,29,102,157,24,172,206]);
    assert_eq!(RECONCILE_PENDING_INSTRUCTION_SIZE, 8);
    let mut f = fixture(); let before = f.clone(); let rent = f.rent.clone();
    f.with_infos(|a| {
        for length in 0..=25 { if length == 8 { continue; }
            let mut data = encode_reconcile_pending().to_vec(); data.resize(length, 0);
            assert_eq!(dispatch(a, &data, rent.clone()), Err(ProgramError::InvalidInstructionData));
        }
        for index in 0..8 { let mut data = encode_reconcile_pending(); data[index] ^= 1;
            assert_eq!(dispatch(a, &data, rent.clone()), Err(ProgramError::InvalidInstructionData)); }
        for count in 0..=6 { if count == 4 { continue; }
            let list: Vec<_> = (0..count).map(|i| a[i % 4].clone()).collect();
            assert_eq!(dispatch(&list, &encode_reconcile_pending(), rent.clone()), Err(if count < 4 { ProgramError::NotEnoughAccountKeys } else { ProgramError::InvalidArgument }));
        }
        assert_eq!(process_instruction(&PROGRAM, a, &encode_reconcile_pending()), Err(ProgramError::Custom(HOST_RUNTIME_UNAVAILABLE_CODE)));
    }); assert_eq!(f, before);
    for a in &mut f.accounts { a.writable = true; }
    f.donate_native(SOL, 9); assert_success(&mut f, 9, 0);
}

#[test]
fn active_cross_state_sequence_hwm_and_pending_snapshot_mismatches_reject() {
    let mut w = World::new(100_000, 20, 0, 7, 3, FeeFraction::ZERO, 100_000);
    w.open(1_000_000).unwrap(); assert!(w.round.pending_sol_used_lamports > 0);
    for which in 0..3 {
        let mut f = world_fixture(&w); f.donate_native(SOL, 10); f.donate_tokens(2);
        f.edit_config(|c| match which {
            0 => c.next_distribution_sequence += 1,
            1 => c.protected_principal_hwm_lamports += 1,
            _ => c.accounted_pending_sol_lamports = 0,
        });
        rejects(f, if which == 0 { Piv1Error::SequenceMismatch } else { Piv1Error::CumulativeReconciliationMismatch });
    }
}
