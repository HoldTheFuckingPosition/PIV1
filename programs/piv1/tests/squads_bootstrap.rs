#[allow(dead_code)]
#[path = "support/squads_invocation.rs"]
mod support;

use anchor_lang::solana_program::system_program;
use piv1::{errors::Piv1Error, squads_execution::*};
use support::*;
use SquadsExecutionError as E;

fn roles() -> SquadsBootstrapRoles {
    SquadsBootstrapRoles { program: PROGRAM_ACCOUNT, program_data: PROGRAM_DATA,
        multisig: MULTISIG, proposal: PROPOSAL, transaction: TRANSACTION,
        vault: VAULT, instructions: INSTRUCTIONS, config: CONFIG }
}

fn virgin() -> Fixture {
    let mut f = Fixture::new();
    f.accounts.truncate(8); // No registry, rewards or Clock AccountInfo supplied.
    f.accounts[CONFIG].owner = system_program::ID;
    f.accounts[CONFIG].data.clear();
    f.accounts[CONFIG].lamports = 0;
    f.rebuild_message();
    f
}

fn run(f: &mut Fixture, roles: SquadsBootstrapRoles) -> SquadsExecutionResult<AuthenticatedSquadsBootstrapInvocation> {
    let before = f.clone();
    let program = f.program; let data = f.inner_data.clone();
    let context = ModeledSquadsInvocationContext {
        stack_height: f.stack_height, clock: f.clock.clone(), rent: f.rent.clone(),
    };
    let result = f.with_infos(|accounts| authenticate_squads_bootstrap_invocation_with_host_context(
        &program, accounts, &data, roles, 7, context));
    assert_eq!(*f, before, "all keys, owners, flags, lamports, bytes and fixture inputs preserved");
    result
}

fn success(f: &mut Fixture) -> AuthenticatedSquadsBootstrapInvocation { run(f, roles()).unwrap() }
fn reject(f: &mut Fixture, error: E) { assert_eq!(run(f, roles()), Err(error)); }

#[test]
fn virgin_and_prefunded_config_authorize_without_initializing_or_classifying_funds() {
    for lamports in [0, 1, 9_999_999, u64::MAX] {
        let mut f = virgin(); f.accounts[CONFIG].lamports = lamports;
        let proof = success(&mut f);
        assert_eq!(proof.program(), PROGRAM);
        assert_eq!(proof.config(), f.accounts[CONFIG].key);
        assert_eq!(proof.multisig(), f.accounts[MULTISIG].key);
        assert_eq!(proof.proposal(), f.accounts[PROPOSAL].key);
        assert_eq!(proof.transaction(), f.accounts[TRANSACTION].key);
        assert_eq!(proof.vault(), f.accounts[VAULT].key);
        assert_eq!(proof.transaction_index(), 19);
        assert_eq!(proof.top_level_index(), 0);
        assert_eq!(proof.current_members(), &(91..97).map(key).collect::<Vec<_>>()[..]);
        assert_eq!(proof.approved_member_bitmap(), 0b001111);
        assert_eq!(success(&mut f), proof); // No persistent replay receipt exists.
    }
}

#[test]
fn bootstrap_bitmap_uses_sorted_squads_members_without_assigning_piv_slots() {
    let mut f = virgin(); f.proposal.approved = [92, 94, 95, 96].map(key).to_vec();
    f.sync_proposal();
    assert_eq!(success(&mut f).approved_member_bitmap(), 0b111010);
    let mut initialized = Fixture::new();
    assert_eq!(initialized.success().approved_guardian_bitmap(), 0b111100); // Existing reversed slots.
    reject(&mut initialized, E::State(Piv1Error::InvalidAccountOwner));
    initialized.accounts[CONFIG].owner = system_program::ID;
    initialized.accounts[CONFIG].data.clear();
    initialized.reject(E::State(Piv1Error::InvalidAccountOwner)); // Old gate still requires initialized PIV state.
}

#[test]
fn canonical_config_identity_ownership_allocation_and_privileges_are_required() {
    let mut f = virgin(); f.accounts[CONFIG].key = key(210); f.rebuild_message();
    reject(&mut f, E::State(Piv1Error::InvalidAccountPda));
    for owner in [PROGRAM, key(210)] {
        let mut f = virgin(); f.accounts[CONFIG].owner = owner;
        reject(&mut f, E::State(Piv1Error::InvalidAccountOwner));
    }
    let mut f = virgin(); f.accounts[CONFIG].executable = true;
    reject(&mut f, E::State(Piv1Error::ExecutableAccount));
    for data in [vec![0], vec![0; 1014], vec![0xA5; 1014]] {
        let mut f = virgin(); f.accounts[CONFIG].data = data;
        reject(&mut f, E::State(Piv1Error::InvalidAccountSize));
    }
    let mut f = virgin(); f.accounts[CONFIG].writable = false; f.rebuild_message();
    reject(&mut f, E::State(Piv1Error::AccountNotWritable));
    let mut f = virgin(); f.program = key(210);
    reject(&mut f, E::State(Piv1Error::InvalidAccountPda));
}

#[test]
fn host_runtime_rejects_before_account_reads_and_direct_stack_is_required() {
    let mut f = virgin(); let before = f.clone(); let data = f.inner_data.clone();
    f.with_infos(|accounts| {
        let _borrow = accounts[CONFIG].try_borrow_mut_data().unwrap();
        assert_eq!(authenticate_squads_bootstrap_invocation(&PROGRAM, accounts, &data, roles(), 7),
            Err(E::HostRuntimeUnavailable));
        assert_eq!(authenticate_squads_bootstrap_invocation(&PROGRAM, &[], &[], roles(), 7),
            Err(E::HostRuntimeUnavailable));
    });
    assert_eq!(f, before);
    for height in [0, 1, 3, 4, usize::MAX] {
        let mut f = virgin(); f.stack_height = height; reject(&mut f, E::InvalidInvocation);
    }
}

#[test]
fn config_and_shared_account_borrow_failures_preserve_inputs_but_lamports_are_not_read() {
    for role in [CONFIG, PROGRAM_ACCOUNT, PROGRAM_DATA, MULTISIG, PROPOSAL, TRANSACTION, INSTRUCTIONS] {
        let mut f = virgin(); let before = f.clone(); let data = f.inner_data.clone();
        let context = ModeledSquadsInvocationContext { stack_height: 2, clock: f.clock.clone(), rent: f.rent.clone() };
        f.with_infos(|accounts| {
            let _borrow = accounts[role].try_borrow_mut_data().unwrap();
            assert_eq!(authenticate_squads_bootstrap_invocation_with_host_context(&PROGRAM, accounts,
                &data, roles(), 7, context), Err(E::State(Piv1Error::AccountBorrowFailed)));
        });
        assert_eq!(f, before);
    }
    let mut f = virgin(); let before = f.clone(); let data = f.inner_data.clone();
    let context = ModeledSquadsInvocationContext { stack_height: 2, clock: f.clock.clone(), rent: f.rent.clone() };
    f.with_infos(|accounts| {
        let _borrow = accounts[CONFIG].try_borrow_mut_lamports().unwrap();
        authenticate_squads_bootstrap_invocation_with_host_context(&PROGRAM, accounts, &data, roles(), 7, context).unwrap();
    });
    assert_eq!(f, before);
}

#[test]
fn all_roles_are_bounded_distinct_and_can_select_actual_permuted_accounts() {
    let setters: [fn(&mut SquadsBootstrapRoles, usize); 8] = [
        |r,v| r.program=v, |r,v| r.program_data=v, |r,v| r.multisig=v,
        |r,v| r.proposal=v, |r,v| r.transaction=v, |r,v| r.vault=v,
        |r,v| r.instructions=v, |r,v| r.config=v,
    ];
    for (index, set) in setters.iter().enumerate() {
        for invalid in [usize::MAX, (index + 1) % 8] {
            let mut f = virgin(); let mut selected = roles(); set(&mut selected, invalid);
            assert_eq!(run(&mut f, selected), Err(E::InvalidInvocation));
        }
    }
    let mut f = virgin(); f.accounts[CONFIG].key = f.accounts[MULTISIG].key;
    reject(&mut f, E::State(Piv1Error::AccountAlias));
    let mut f = virgin(); f.accounts.swap(PROGRAM_ACCOUNT, CONFIG); f.rebuild_message();
    let mut selected = roles(); selected.program = CONFIG; selected.config = PROGRAM_ACCOUNT;
    assert_eq!(run(&mut f, selected).unwrap().program(), PROGRAM);
}

#[test]
fn current_authority_and_approved_votes_are_refreshed_for_each_bootstrap_check() {
    let mut f = virgin(); success(&mut f); f.multisig.threshold = 1; f.sync_multisig();
    reject(&mut f, E::State(Piv1Error::InvalidGuardianSet));
    let mut f = virgin(); success(&mut f); f.multisig.config_authority = key(91); f.sync_multisig();
    reject(&mut f, E::State(Piv1Error::InvalidGuardianSet));
    let mut f = virgin(); success(&mut f); f.accounts[PROGRAM_DATA].data[13..45].copy_from_slice(key(91).as_ref());
    reject(&mut f, E::State(Piv1Error::InvalidProgramIdentity));
    let mut f = virgin(); f.proposal.approved.truncate(1); f.sync_proposal(); // Threshold1 history with restored4.
    reject(&mut f, E::InvalidProposal);
    let mut f = virgin(); f.proposal.approved[1] = f.proposal.approved[0]; f.sync_proposal();
    reject(&mut f, E::InvalidProposal);
    let mut f = virgin(); f.proposal.approved[3] = key(210); f.sync_proposal();
    reject(&mut f, E::InvalidProposal);
    let mut f = virgin(); f.multisig.stale_transaction_index = 19; f.sync_multisig();
    reject(&mut f, E::InvalidProposal);
    for status in [0, 1, 2, 4, 5, 6, 255] {
        let mut f = virgin(); f.proposal.status = status; f.sync_proposal(); reject(&mut f, E::InvalidProposal);
    }
    let mut f = virgin(); f.proposal.cancelled = (91..95).map(key).collect(); f.sync_proposal();
    reject(&mut f, E::InvalidProposal);
    let mut f = virgin(); f.clock.unix_timestamp = 99; reject(&mut f, E::TimelockNotReleased);
}

#[test]
fn bootstrap_message_accounts_bytes_and_inner_privileges_match_exact_approval() {
    let mut f = virgin(); f.inner_data[0] ^= 1; reject(&mut f, E::MessageMismatch);
    let mut f = virgin(); f.transaction.message.instructions[0].accounts.swap(CONFIG, PROGRAM_DATA);
    f.sync_transaction(); reject(&mut f, E::MessageMismatch);
    let mut f = virgin(); f.accounts[CONFIG].signer = true; reject(&mut f, E::MessageMismatch);
    let mut f = virgin(); f.accounts[VAULT].signer = false; reject(&mut f, E::InvalidInvocation);
    for role in [MULTISIG, PROPOSAL, TRANSACTION] {
        let mut f = virgin(); f.accounts[role].writable = true; f.rebuild_message();
        reject(&mut f, E::InvalidInvocation);
    }
    let mut f = virgin(); f.proposal.index -= 1; f.sync_proposal(); reject(&mut f, E::InvalidProposal);
    let mut f = virgin(); f.transaction.index -= 1; f.sync_transaction(); reject(&mut f, E::InvalidTransaction);
    // Parameter semantics are intentionally opaque at this prerequisite.
    let mut f = virgin(); f.inner_data = vec![0xA5; 17]; f.rebuild_message(); success(&mut f);
}

#[test]
fn bootstrap_uses_the_same_direct_outer_executor_and_bounded_message_profile() {
    let mut f = virgin(); let transaction = f.accounts[TRANSACTION].clone();
    for executor in [key(91), key(96)] {
        f.outer[0].accounts[3].pubkey = executor; f.sync_sysvar(); success(&mut f);
        assert_eq!(f.accounts[TRANSACTION], transaction);
    }
    f.outer[0].accounts[3].is_signer = false; f.sync_sysvar(); reject(&mut f, E::InvalidInvocation);
    let mut f = virgin(); f.outer[0].program_id = key(210); f.sync_sysvar(); reject(&mut f, E::InvalidInvocation);
    let mut f = virgin(); f.outer[0].accounts.swap(4, 5); f.sync_sysvar(); reject(&mut f, E::InvalidInvocation);
    let mut f = virgin(); f.transaction.ephemeral.push(1); f.sync_transaction(); reject(&mut f, E::UnsupportedMessage);
    let mut f = virgin(); f.transaction.message.instructions.push(f.transaction.message.instructions[0].clone());
    f.sync_transaction(); reject(&mut f, E::UnsupportedMessage);
    let mut f = virgin(); f.transaction.message.lookups.push(Lookup { account_key: key(210), writable: vec![], readonly: vec![] });
    f.sync_transaction(); reject(&mut f, E::UnsupportedMessage);
    let mut f = virgin(); f.accounts[INSTRUCTIONS].data = vec![0, 0];
    reject(&mut f, E::InvalidInstructionsSysvar);
}
