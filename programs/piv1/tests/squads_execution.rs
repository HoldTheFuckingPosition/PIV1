#[path = "support/squads_invocation.rs"]
mod support;

use anchor_lang::{
    prelude::{Clock, SolanaSysvar},
    solana_program::instruction::Instruction,
    AnchorDeserialize, AnchorSerialize,
};
use piv1::{errors::Piv1Error, squads_execution::*, state::PivConfig};
use support::*;
use SquadsExecutionError as E;

#[test]
fn direct_current_governance_message_matches_actual_inputs_and_preserves_every_account() {
    let mut f = Fixture::new();
    let evidence = f.success();
    assert_eq!(evidence.program(), PROGRAM);
    assert_eq!(evidence.multisig(), f.accounts[MULTISIG].key);
    assert_eq!(evidence.proposal(), f.accounts[PROPOSAL].key);
    assert_eq!(evidence.transaction(), f.accounts[TRANSACTION].key);
    assert_eq!(evidence.vault(), f.accounts[VAULT].key);
    assert_eq!(evidence.transaction_index(), 19);
    assert_eq!(evidence.top_level_index(), 0);
    assert_eq!(evidence.approved_guardian_bitmap(), 0b111100); // Reversed PIV1 slots.
    assert_eq!(evidence.guardian_registry_revision(), 99);
    assert_eq!(f.accounts[PROPOSAL].data[48], 3); // Serialized Approved during CPI.
    assert_eq!(&f.accounts[TRANSACTION].data[..8], &VAULT_TRANSACTION_DISCRIMINATOR);
}

#[test]
fn any_current_outer_executor_can_execute_identical_approved_inner_bytes() {
    let mut f = Fixture::new();
    let proposal = f.accounts[PROPOSAL].clone();
    let transaction = f.accounts[TRANSACTION].clone();
    let inner_data = f.inner_data.clone();
    for executor in [key(91), key(92), key(96)] {
        f.outer[0].accounts[3].pubkey = executor; f.sync_sysvar(); f.success();
        assert_eq!(f.accounts[PROPOSAL], proposal);
        assert_eq!(f.accounts[TRANSACTION], transaction);
        assert_eq!(f.inner_data, inner_data);
        assert!(!f.accounts.iter().any(|account| account.key == executor));
    }
    f.outer[0].accounts[3].is_signer = false; f.sync_sysvar(); f.reject(E::InvalidInvocation);
    let mut f = Fixture::new(); f.outer[0].accounts[3].pubkey = key(200); f.sync_sysvar();
    f.reject(E::InvalidInvocation);
    let mut f = Fixture::new(); f.multisig.members[0].permissions = 3; f.sync_multisig();
    f.reject(E::InvalidInvocation);
}

#[test]
fn ordinary_host_path_and_nondirect_stack_heights_cannot_authorize() {
    let mut f = Fixture::new(); let before = f.clone(); let roles = f.roles(); let data = f.inner_data.clone();
    f.with_infos(|accounts| assert_eq!(authenticate_squads_invocation(&PROGRAM, accounts, &data, roles, 7), Err(E::HostRuntimeUnavailable)));
    assert_eq!(f, before);
    for height in [0, 1, 3, 4, usize::MAX] {
        let mut f = Fixture::new(); f.stack_height = height; f.reject(E::InvalidInvocation);
    }
}

#[test]
fn current_authority_guardians_and_clock_are_refreshed_and_same_runtime_bound() {
    let mut f = Fixture::new(); f.success(); f.multisig.threshold = 1; f.sync_multisig();
    f.reject(E::State(Piv1Error::InvalidGuardianSet));
    let mut f = Fixture::new(); f.program = key(218); f.reject(E::State(Piv1Error::InvalidProgramIdentity));
    let mut f = Fixture::new(); f.accounts[PROGRAM_DATA].data[13..45].copy_from_slice(key(91).as_ref());
    f.reject(E::State(Piv1Error::InvalidProgramIdentity));
    let mut f = Fixture::new(); f.accounts[REGISTRY].data[18..50].copy_from_slice(key(108).as_ref());
    f.reject_any();
    let mut f = Fixture::new(); f.clock.slot += 1; f.reject(E::InvalidInvocation);
    let mut f = Fixture::new();
    Clock { slot: 1, unix_timestamp: 100, ..Clock::default() }.to_account_info(&mut f.accounts[CLOCK].info()).unwrap();
    f.reject(E::InvalidInvocation);
    // Existing-state inspection/authorization does not silently select a pause
    // policy for future handlers; a governed unpause needs authorization too.
    let mut f = Fixture::new();
    let mut config = PivConfig::deserialize(&mut &f.accounts[CONFIG].data[8..]).unwrap();
    config.paused = true;
    let serialized = config.try_to_vec().unwrap();
    f.accounts[CONFIG].data[8..].fill(0);
    f.accounts[CONFIG].data[8..8 + serialized.len()].copy_from_slice(&serialized);
    f.success();
}

#[test]
fn detached_or_duplicate_role_selection_and_inner_keys_reject() {
    for bad in [usize::MAX, PROPOSAL] {
        let mut f = Fixture::new(); let before = f.clone(); let mut roles = f.roles(); roles.transaction = bad;
        let data = f.inner_data.clone(); let context = ModeledSquadsInvocationContext { stack_height: 2, clock: f.clock.clone(), rent: f.rent.clone() };
        f.with_infos(|accounts| assert_eq!(authenticate_squads_invocation_with_host_context(&PROGRAM, accounts, &data, roles, 7, context), Err(E::InvalidInvocation)));
        assert_eq!(f, before);
    }
    let mut f = Fixture::new(); f.accounts[VAULT].key = f.accounts[PROPOSAL].key;
    f.reject(E::State(Piv1Error::AccountAlias));
}

#[test]
fn proposal_owner_executable_discriminator_multisig_and_bump_are_authenticated() {
    for role in [PROPOSAL, TRANSACTION] {
        let mut f = Fixture::new(); f.accounts[role].owner = key(200); f.reject(E::State(Piv1Error::InvalidAccountOwner));
        let mut f = Fixture::new(); f.accounts[role].executable = true; f.reject(E::State(Piv1Error::ExecutableAccount));
    }
    let mut f = Fixture::new(); f.accounts[PROPOSAL].data[0] ^= 1; f.reject(E::InvalidProposal);
    let mut f = Fixture::new(); f.proposal.multisig = key(200); f.sync_proposal(); f.reject(E::InvalidProposal);
    let mut f = Fixture::new(); f.proposal.bump = f.proposal.bump.wrapping_sub(1); f.sync_proposal(); f.reject(E::InvalidProposal);
    let mut f = Fixture::new(); f.accounts[PROPOSAL].key = key(200); f.reject(E::InvalidProposal);
}

#[test]
fn only_nonzero_current_configuration_approved_proposals_are_accepted() {
    for index in [0, 1, 3, 20, u64::MAX] {
        let mut f = Fixture::new(); f.proposal.index = index; f.sync_proposal(); f.reject(E::InvalidProposal);
    }
    for status in [0, 1, 2, 4, 5, 6, 255] {
        let mut f = Fixture::new(); f.proposal.status = status; f.sync_proposal(); f.reject(E::InvalidProposal);
    }
    let mut f = Fixture::new(); f.multisig.stale_transaction_index = 19; f.sync_multisig();
    f.reject(E::InvalidProposal);
    // Restored threshold4 with an old threshold1 approval is not proof of 4 votes.
    let mut f = Fixture::new(); f.proposal.approved.truncate(1); f.sync_proposal(); f.reject(E::InvalidProposal);
}

#[test]
fn approvals_are_distinct_eligible_and_cancellation_is_collective() {
    for count in [4, 5, 6] {
        let mut f = Fixture::new(); f.proposal.approved = (91..91 + count).map(key).collect();
        f.proposal.cancelled = vec![key(91), key(92), key(93)]; f.sync_proposal(); f.success();
    }
    for approved in [vec![], vec![key(91), key(92), key(93)], vec![key(91); 4],
        vec![key(92), key(91), key(93), key(94)], vec![key(91), key(92), key(93), key(200)],
        (91..98).map(key).collect()] {
        let mut f = Fixture::new(); f.proposal.approved = approved; f.sync_proposal(); f.reject(E::InvalidProposal);
    }
    let mut f = Fixture::new(); f.proposal.rejected = vec![key(95), key(96)]; f.sync_proposal(); f.success();
    for rejected in [vec![key(91)], vec![key(94), key(95), key(96)], vec![key(95), key(95)], vec![key(96), key(95)]] {
        let mut f = Fixture::new(); f.proposal.rejected = rejected; f.sync_proposal(); f.reject(E::InvalidProposal);
    }
    for cancelled in [vec![key(91), key(92), key(93), key(94)], vec![key(91), key(91)], vec![key(200)], vec![key(92), key(91)]] {
        let mut f = Fixture::new(); f.proposal.cancelled = cancelled; f.sync_proposal(); f.reject(E::InvalidProposal);
    }
}

#[test]
fn approval_timestamp_uses_checked_trusted_clock_and_timelock_boundary() {
    let mut f = Fixture::new(); f.success();
    for timestamp in [91, 101, i64::MAX, i64::MIN] {
        let mut f = Fixture::new(); f.proposal.timestamp = timestamp; f.sync_proposal(); f.reject(E::TimelockNotReleased);
    }
    let mut f = Fixture::new(); f.multisig.time_lock = 11; f.sync_multisig(); f.reject(E::TimelockNotReleased);
    let mut f = Fixture::new(); f.multisig.time_lock = 0; f.proposal.timestamp = 100;
    f.sync_multisig(); f.sync_proposal(); f.success();
}

#[test]
fn proposal_borsh_counts_are_bounded_and_unused_allocation_tail_is_opaque() {
    let mut f = Fixture::new();
    let used = 8 + f.proposal.try_to_vec().unwrap().len();
    f.accounts[PROPOSAL].data[used..].fill(0xA5); f.accounts[PROPOSAL].data.resize(50_000, 0xF1); f.success();
    for offset in [58, 62 + 4 * 32, 66 + 4 * 32] {
        for count in [7_u32, u32::MAX] {
            let mut f = Fixture::new(); f.accounts[PROPOSAL].data[offset..offset + 4].copy_from_slice(&count.to_le_bytes());
            f.reject(E::InvalidProposal);
        }
    }
    for length in 0..used {
        let mut f = Fixture::new(); f.accounts[PROPOSAL].data.truncate(length); f.reject_any();
    }
}

#[test]
fn transaction_account_links_indices_bumps_and_vault_are_exact() {
    let mut f = Fixture::new(); f.accounts[TRANSACTION].data[0] ^= 1; f.reject(E::InvalidTransaction);
    let mut f = Fixture::new(); f.transaction.multisig = key(200); f.sync_transaction(); f.reject(E::InvalidTransaction);
    let mut f = Fixture::new(); f.transaction.index -= 1; f.sync_transaction(); f.reject(E::InvalidTransaction);
    let mut f = Fixture::new(); f.transaction.bump = f.transaction.bump.wrapping_sub(1); f.sync_transaction(); f.reject(E::InvalidTransaction);
    let mut f = Fixture::new(); f.transaction.vault_bump = f.transaction.vault_bump.wrapping_sub(1); f.sync_transaction(); f.reject(E::InvalidTransaction);
    let mut f = Fixture::new(); f.accounts[TRANSACTION].key = key(200); f.reject(E::InvalidTransaction);
    let mut f = Fixture::new(); f.accounts[TRANSACTION].data[81] = 8; f.reject(E::InvalidTransaction);
    let mut f = Fixture::new(); f.accounts[VAULT].signer = false; f.reject(E::InvalidInvocation);
    let mut f = Fixture::new(); f.accounts[VAULT].key = key(200); f.reject(E::InvalidInvocation);
}

#[test]
fn batches_multiple_inner_instructions_lookups_and_ephemeral_signers_are_unsupported() {
    let mut f = Fixture::new(); f.transaction.ephemeral.push(250); f.sync_transaction(); f.reject(E::UnsupportedMessage);
    let mut f = Fixture::new(); f.transaction.message.lookups.push(Lookup { account_key: key(200), writable: vec![], readonly: vec![] });
    f.sync_transaction(); f.reject(E::UnsupportedMessage);
    let mut f = Fixture::new(); f.transaction.message.instructions.clear(); f.sync_transaction(); f.reject(E::UnsupportedMessage);
    let mut f = Fixture::new(); f.transaction.message.instructions.push(f.transaction.message.instructions[0].clone());
    f.sync_transaction(); f.reject(E::UnsupportedMessage);
    let mut f = Fixture::new(); f.outer[0].data = vec![0xBB; 8]; f.sync_sysvar(); f.reject(E::InvalidInvocation);
}

#[test]
fn actual_inner_account_order_instruction_data_and_privileges_must_match() {
    let mut f = Fixture::new(); f.inner_data[0] ^= 1; f.reject(E::MessageMismatch);
    let mut f = Fixture::new(); f.inner_data.push(0); f.reject(E::MessageMismatch);
    let mut f = Fixture::new(); f.transaction.message.instructions[0].accounts.swap(CONFIG, REGISTRY);
    f.sync_transaction(); f.reject(E::MessageMismatch);
    let mut f = Fixture::new(); f.accounts[CONFIG].writable = false; f.reject(E::MessageMismatch);
    let mut f = Fixture::new(); f.accounts[CONFIG].signer = true; f.reject(E::MessageMismatch);
    let mut f = Fixture::new(); f.transaction.message.instructions[0].program_id_index = 0; // vault key, not PIV1
    f.sync_transaction(); f.reject(E::MessageMismatch);
    for role in [MULTISIG, PROPOSAL, TRANSACTION] {
        let mut f = Fixture::new(); f.accounts[role].writable = true; f.rebuild_message(); f.reject(E::InvalidInvocation);
    }
}

#[test]
fn static_keys_indices_and_persisted_vector_lengths_are_bounded_and_canonical() {
    let mut f = Fixture::new(); f.transaction.message.keys[1] = f.transaction.message.keys[0]; f.sync_transaction(); f.reject(E::UnsupportedMessage);
    let mut f = Fixture::new(); f.transaction.message.instructions[0].accounts[1] = f.transaction.message.instructions[0].accounts[0];
    f.sync_transaction(); f.reject(E::UnsupportedMessage);
    let mut f = Fixture::new(); f.transaction.message.instructions[0].accounts[0] = 255; f.sync_transaction(); f.reject(E::InvalidTransaction);
    let mut f = Fixture::new(); f.transaction.message.instructions[0].accounts.pop(); f.sync_transaction(); f.reject(E::MessageMismatch);
    let mut f = Fixture::new(); f.transaction.message.keys.push(key(200)); f.sync_transaction(); f.reject(E::UnsupportedMessage);
    for offset in [87, 88, 89] {
        let mut f = Fixture::new(); f.accounts[TRANSACTION].data[offset] = 255; f.reject(E::UnsupportedMessage);
    }
    for (offset, count) in [(83, u32::MAX), (90, u32::MAX), (90, 0)] {
        let mut f = Fixture::new(); f.accounts[TRANSACTION].data[offset..offset + 4].copy_from_slice(&count.to_le_bytes());
        f.reject(E::UnsupportedMessage);
    }
    let mut f = Fixture::new(); f.accounts[TRANSACTION].data.push(0); f.reject(E::UnsupportedMessage);
    let length = Fixture::new().accounts[TRANSACTION].data.len();
    for end in 0..length {
        let mut f = Fixture::new(); f.accounts[TRANSACTION].data.truncate(end); f.reject_any();
    }
}

#[test]
fn exact_outer_instruction_fixed_roles_remaining_order_and_required_privileges_are_bound() {
    let mut f = Fixture::new(); f.outer[0].program_id = key(200); f.sync_sysvar(); f.reject(E::InvalidInvocation);
    for length in [0, 7, 9] {
        let mut f = Fixture::new(); f.outer[0].data.resize(length, 0); f.sync_sysvar(); f.reject(E::InvalidInvocation);
    }
    for role in 0..3 {
        let mut f = Fixture::new(); f.outer[0].accounts[role].pubkey = key(200); f.sync_sysvar(); f.reject(E::InvalidInvocation);
    }
    let mut f = Fixture::new(); f.outer[0].accounts[1].is_writable = false; f.sync_sysvar(); f.reject(E::InvalidInvocation);
    let mut f = Fixture::new(); f.outer[0].accounts.swap(4, 5); f.sync_sysvar(); f.reject(E::InvalidInvocation);
    let mut f = Fixture::new(); f.outer[0].accounts.pop(); f.sync_sysvar(); f.reject(E::InvalidInvocation);
    let mut f = Fixture::new();
    for meta in &mut f.outer[0].accounts { meta.is_writable = false; }
    f.outer[0].accounts[1].is_writable = true; f.sync_sysvar(); f.reject(E::InvalidInvocation);
    // Global OUTER unions need not equal the inner request's minimal privileges.
    let mut f = Fixture::new();
    for meta in &mut f.outer[0].accounts { meta.is_writable = true; meta.is_signer = true; }
    f.sync_sysvar(); f.success();
}

#[test]
fn real_sysvar_geometry_selects_current_outer_instruction_not_a_neighbor() {
    let mut f = Fixture::new();
    let execute = f.outer[0].clone();
    let unrelated = Instruction { program_id: key(200), accounts: vec![], data: vec![0xA1, 0xA2] };
    f.outer = vec![unrelated.clone(), execute, unrelated]; f.current = 1; f.sync_sysvar();
    assert_eq!(f.success().top_level_index(), 1);
    for current in [0, 2, 3, u16::MAX] { f.current = current; f.sync_sysvar(); f.reject_any(); }
}

#[test]
fn sysvar_owner_identity_borrows_short_headers_offsets_flags_and_counts_are_defended() {
    let mut f = Fixture::new(); f.accounts[INSTRUCTIONS].owner = key(200); f.reject(E::InvalidInstructionsSysvar);
    let mut f = Fixture::new(); f.accounts[INSTRUCTIONS].key = key(200); f.rebuild_message(); f.reject(E::InvalidInstructionsSysvar);
    let mut f = Fixture::new(); f.accounts[INSTRUCTIONS].executable = true; f.reject(E::InvalidInstructionsSysvar);
    for bytes in [vec![], vec![0], vec![0,0], vec![0,0,0], vec![255,255,0,0], vec![1,0,0,0,0,0]] {
        let mut f = Fixture::new(); f.accounts[INSTRUCTIONS].data = bytes; f.reject_any();
    }
    for offset in [0_u16, 2, 5, u16::MAX] {
        let mut f = Fixture::new(); f.accounts[INSTRUCTIONS].data[2..4].copy_from_slice(&offset.to_le_bytes()); f.reject_any();
    }
    let mut f = Fixture::new(); f.accounts[INSTRUCTIONS].data[4..6].copy_from_slice(&u16::MAX.to_le_bytes()); f.reject(E::InvalidInvocation);
    let mut f = Fixture::new(); f.accounts[INSTRUCTIONS].data[6] = 0x80; f.reject(E::InvalidInvocation);
    let length = Fixture::new().accounts[INSTRUCTIONS].data.len();
    for end in 0..length {
        let mut f = Fixture::new(); f.accounts[INSTRUCTIONS].data.truncate(end); f.reject_any();
    }
}

#[test]
fn every_inspected_role_data_borrow_conflict_is_read_only_and_fallible() {
    for role in (0..16).filter(|index| *index != VAULT) {
        let mut f = Fixture::new(); let before = f.clone(); let roles = f.roles(); let data = f.inner_data.clone();
        let context = ModeledSquadsInvocationContext { stack_height: 2, clock: f.clock.clone(), rent: f.rent.clone() };
        f.with_infos(|accounts| {
            let _borrow = accounts[role].try_borrow_mut_data().unwrap();
            assert_eq!(authenticate_squads_invocation_with_host_context(&PROGRAM, accounts, &data, roles, 7, context),
                Err(E::State(Piv1Error::AccountBorrowFailed)));
        });
        assert_eq!(f, before);
    }
}

#[test]
fn an_unrelated_valid_approved_proposal_cannot_substitute_for_the_executing_one() {
    let mut f = Fixture::new();
    // Create another internally consistent current-index Proposal and Transaction,
    // update the approved message to name those supplied account keys, but keep
    // the outer executor bound to the original proposal and transaction.
    let original_outer = f.outer.clone();
    let index = 18_u64;
    let multisig = f.accounts[MULTISIG].key;
    let (proposal, proposal_bump) = anchor_lang::prelude::Pubkey::find_program_address(
        &[b"multisig", multisig.as_ref(), b"transaction", &index.to_le_bytes(), b"proposal"], &piv1::squads_accounts::SQUADS_V4_PROGRAM_ID);
    let (transaction, transaction_bump) = anchor_lang::prelude::Pubkey::find_program_address(
        &[b"multisig", multisig.as_ref(), b"transaction", &index.to_le_bytes()], &piv1::squads_accounts::SQUADS_V4_PROGRAM_ID);
    f.accounts[PROPOSAL].key = proposal; f.proposal.index = index; f.proposal.bump = proposal_bump;
    f.accounts[TRANSACTION].key = transaction; f.transaction.index = index; f.transaction.bump = transaction_bump;
    f.sync_proposal(); f.rebuild_message(); f.success();
    f.outer = original_outer; f.sync_sysvar(); f.reject(E::InvalidInvocation);
}

#[test]
fn approval_state_is_inspected_not_consumed_and_no_host_replay_receipt_is_invented() {
    let mut f = Fixture::new(); f.success(); f.success();
    // A modeled outer-success status change is not actual Squads execution or
    // rollback evidence. It demonstrates that Executed cannot authorize a replay.
    f.proposal.status = 5; f.sync_proposal(); f.reject(E::InvalidProposal);
}
