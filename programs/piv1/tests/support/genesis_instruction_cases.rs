//! Native boundary regressions reuse the established independently checked CPI
//! fixture; modeled effects here are not evidence of runtime atomic rollback.
use super::*;
use piv1::instruction_boundary::process_initialize_with_host_invoker;

fn native_world(program: Pubkey, shared: bool, paused: bool) -> World {
    let mut w = World::recipient_checked(program, shared, paused);
    let mut recipients = w.recipients.unwrap();
    recipients.htfp_vault_index = 23; recipients.team_owner_vault_index = 172;
    let vault = |index| Pubkey::find_program_address(
        &[b"multisig", w.f.accounts[MULTISIG].key.as_ref(), b"vault", &[index]],
        &piv1::squads_accounts::SQUADS_V4_PROGRAM_ID).0;
    let htfp = vault(23); let team = vault(172);
    w.f.accounts[recipients.htfp_recipient].key = htfp;
    w.f.accounts[recipients.team_owner_recipient].key = team;
    w.parameters.htfp_recipient = htfp; w.parameters.team_owner_recipient = team;
    w.recipients = Some(recipients);
    w.f.inner_data = InitializePiv1Parameters { model: w.parameters,
        htfp_vault_index: 23, team_owner_vault_index: 172 }.encode().unwrap().to_vec();
    w.f.rebuild_message(); w
}

fn run(w: &mut World, behavior: Behavior) -> (ProgramResult, usize) {
    let before = w.clone(); let program = w.f.program; let data = w.f.inner_data.clone();
    let ctx = context(&w.f); let calls = expected_calls(&before); let mut count = 0;
    let (result, observed) = w.f.with_infos(|infos| {
        let result = process_initialize_with_host_invoker(&program, infos, &data, ctx,
            |instruction, actual, seeds| {
                let index = count; count += 1;
                emulate(&before, &calls[index], behavior, index, instruction, actual, seeds, infos)
            });
        (result, infos.iter().map(snapshot).collect::<Vec<_>>())
    });
    w.f.accounts = observed; (result, count)
}

fn reject_unchanged(w: &mut World, expected: ProgramError) {
    let before = w.f.clone();
    assert_eq!(run(w, Behavior::Good), (Err(expected), 0)); assert_fixture(&w.f, &before);
}

#[test]
fn native_initializer_preserves_full_state_rent_and_prefunds_for_both_topologies_and_runtime_ids() {
    for program in [PROGRAM, key(211)] { for shared in [false, true] { for prefunded in [false, true] {
        let mut w = native_world(program, shared, prefunded); let t = w.roles.targets;
        if prefunded {
            for (slot, index) in t.iter().copied().enumerate() {
                w.f.accounts[index].lamports = match slot {
                    0 | 9 => floor(slot) - 1, 14 => floor(slot) + 77, 15 => floor(slot) + 67,
                    _ => floor(slot) + 19,
                };
            }
        }
        let before = w.clone();
        let rent: u64 = t.iter().enumerate().map(|(slot, index)| floor(slot).saturating_sub(w.f.accounts[*index].lamports)).sum();
        // Legacy full checked-recipient execution must retain exactly the same
        // account semantics. Its 313-byte approval is independently rebuilt.
        let mut legacy = w.clone(); legacy.f.inner_data = legacy.parameters.encode().unwrap().to_vec();
        legacy.f.rebuild_message(); assert!(legacy.raw(Behavior::Good).0.is_ok());
        assert_eq!(run(&mut w, Behavior::Good), (Ok(()), expected_calls(&before).len()));
        let excess = if prefunded {144} else {0};
        assert_eq!(w.f.accounts[w.payer].lamports, before.f.accounts[w.payer].lamports - rent);
        for (slot, index) in t.iter().copied().enumerate() {
            assert_eq!(w.f.accounts[index], legacy.f.accounts[index]);
            let expected = if slot == 9 {before.f.accounts[index].lamports.max(floor(slot)) + excess}
                else if slot >= 14 {floor(slot)} else {before.f.accounts[index].lamports.max(floor(slot))};
            assert_eq!(w.f.accounts[index].lamports, expected);
            if slot >= 14 { assert_eq!(w.f.accounts[index].data, independent_token(&w)); }
        }
        for (index, account) in before.f.accounts.iter().enumerate() {
            if index != w.payer && !t.contains(&index) { assert_eq!(&w.f.accounts[index], account); }
        }
        let config: PivConfig = decoded(&w.f.accounts[t[0]].data);
        assert_eq!(config.paused, prefunded);
        assert_eq!((config.protected_principal_hwm_lamports, config.accounted_pending_sol_lamports,
            config.accounted_pending_jitosol_units, config.kif_claim_liability_lamports), (0, 0, 0, 0));
        assert_eq!((config.htfp_recipient, config.team_owner_recipient),
            (before.parameters.htfp_recipient, before.parameters.team_owner_recipient));
        assert!(fixed_observation(&mut w).is_ok());
    } } }
}

#[test]
fn native_approval_authenticates_each_witness_and_the_entire_original_envelope() {
    for shared in [false, true] {
        for offset in [10, 313, 314] {
            let mut w = native_world(PROGRAM, shared, false); w.f.inner_data[offset] ^= 1;
            reject_unchanged(&mut w, ProgramError::Custom(6105));
        }
        let mut w = native_world(PROGRAM, shared, false);
        w.f.transaction.message.instructions[0].data = w.parameters.encode().unwrap().to_vec();
        w.f.sync_transaction();
        reject_unchanged(&mut w, ProgramError::Custom(6105));
        for offset in [313, 314] {
            let mut w = native_world(PROGRAM, shared, false); w.f.inner_data[offset] ^= 1;
            w.f.rebuild_message(); // Approval alone cannot turn a wrong witness into the recipient PDA.
            reject_unchanged(&mut w, ProgramError::Custom(6127));
        }
    }
}

#[test]
fn native_authorization_recipient_and_reinitialization_fail_before_cpi() {
    let mut wrong_height = native_world(PROGRAM, false, false); wrong_height.f.stack_height = 3;
    reject_unchanged(&mut wrong_height, ProgramError::Custom(6100));
    let mut approvals = native_world(PROGRAM, false, false);
    approvals.f.proposal.approved.truncate(3); approvals.f.sync_proposal();
    reject_unchanged(&mut approvals, ProgramError::Custom(6102));
    let mut recipient = native_world(PROGRAM, false, false);
    recipient.f.accounts[recipient.recipients.unwrap().htfp_recipient].lamports = 0;
    reject_unchanged(&mut recipient, ProgramError::Custom(6128));
    for shared in [false, true] {
        let mut w = native_world(PROGRAM, shared, false); assert!(run(&mut w, Behavior::Good).0.is_ok());
        reject_unchanged(&mut w, ProgramError::Custom(6000));
    }
}

#[test]
fn native_system_and_token_errors_and_false_success_propagate_without_host_rollback() {
    let base = native_world(PROGRAM, false, false); let calls = expected_calls(&base);
    for boundary in [0, calls.len() - 2, calls.len() - 1] { for after in [false, true] {
        let mut w = base.clone();
        let behavior = if after {Behavior::ErrorAfter(boundary)} else {Behavior::ErrorBefore(boundary)};
        assert_eq!(run(&mut w, behavior),
            (Err(ProgramError::Custom(if after {22102} else {22101})), boundary + 1));
        if after || boundary != 0 { assert_ne!(w.f.accounts, base.f.accounts, "host seam does not roll back effects"); }
        for index in &w.roles.targets[..9] { assert!(w.f.accounts[*index].data.iter().all(|byte| *byte == 0)); }
    } }
    let mut system = base.clone();
    assert_eq!(run(&mut system, Behavior::NoOp(0)), (Err(ProgramError::Custom(6124)), 1));
    let mut token = base;
    assert_eq!(run(&mut token, Behavior::NoOp(calls.len() - 2)),
        (Err(ProgramError::Custom(6131)), calls.len() - 1));
}

#[test]
fn native_account_order_and_identity_remain_authenticated_before_any_effect() {
    for shared in [false, true] {
        let mut reordered = native_world(PROGRAM, shared, false);
        reordered.f.accounts.swap(24, 25);
        let before = reordered.f.clone(); let (result, calls) = run(&mut reordered, Behavior::Good);
        assert!(result.is_err()); assert_eq!(calls, 0); assert_fixture(&reordered.f, &before);
        let mut wrong_id = native_world(PROGRAM, shared, false); wrong_id.f.program = key(213);
        let before = wrong_id.f.clone(); let (result, calls) = run(&mut wrong_id, Behavior::Good);
        assert!(result.is_err()); assert_eq!(calls, 0); assert_fixture(&wrong_id.f, &before);
    }
}
