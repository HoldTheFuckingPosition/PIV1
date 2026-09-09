//! Additive Task 2.14 tests; the nineteen existing claim tests remain intact.
//! Initialized token units are synthetic. Only System donations execute transfers.
use super::support::{self, Accounts, Runtime, account_evidence, new_key, program_id};
pub use super::support::original_fixture as kif_claim_custody;
#[path = "../../../programs/piv1/tests/support/pending_custody.rs"]
pub mod pending_fixture;
use mollusk_svm::result::types::TransactionResult;
use solana_account::Account;
use solana_instruction::{AccountMeta, Instruction};
use solana_instruction_error::InstructionError;
use solana_transaction_error::TransactionError;
use piv1::{errors::Piv1Error, instructions::reconcile_pending::encode_reconcile_pending,
    state::ActiveDistribution};
const CONFIG: usize = 0;
const ROUND: usize = 1;
const SOL: usize = 2;
const TOKEN: usize = 3;
const SYSTEM: usize = 4;
const PROGRAM: usize = 5;
const PAYER: usize = 6;
const DONOR: usize = 7;

fn fixture(runtime: &Runtime, paused: bool, native_excess: u64) -> Accounts {
    let mut config = kif_claim_custody::Fixture::new(0).config();
    config.accounted_pending_sol_lamports = 0; config.accounted_pending_jitosol_units = 0;
    config.paused = paused;
    let mut f = pending_fixture::Fixture::from_state(config, ActiveDistribution::new_idle(0), 0, 20);
    // Old default Rent and the runtime encode different pricing parameters
    // (3480/2.0 versus 6960/1.0). Compare actual funding minima, not structs.
    let runtime_rent = support::rent_bridge(&runtime.vm.sysvars.rent);
    for length in [0, piv1::state::PivConfig::SPACE, ActiveDistribution::SPACE, 165] {
        assert_eq!(f.rent.minimum_balance(length), runtime_rent.minimum_balance(length));
        assert_eq!(f.rent.minimum_balance(length), runtime.vm.sysvars.rent.minimum_balance(length));
    }
    f.donate_native(pending_fixture::TOKEN, native_excess); f.validate_audit();
    let mut accounts: Accounts = f.accounts.iter().map(|a| (
        solana_pubkey::Pubkey::new_from_array(a.key.to_bytes()), Account {
            lamports: a.lamports, data: a.data.clone(),
            owner: solana_pubkey::Pubkey::new_from_array(a.owner.to_bytes()),
            executable: a.executable, rent_epoch: 0,
        })).collect();
    let common = support::Fixture::new(&runtime.vm.sysvars.rent, 0, false);
    accounts.push(common.accounts[support::SYSTEM].clone());
    accounts.push(common.accounts[6].clone());
    accounts.push(common.accounts[support::PAYER].clone());
    accounts.push((new_key(234), Account { lamports: runtime.vm.sysvars.rent.minimum_balance(0) + 1000,
        owner: accounts[SYSTEM].0, ..Account::default() }));
    accounts.push(common.accounts[support::SENTINEL].clone());
    assert_eq!(accounts[PROGRAM].0, program_id());
    // This vector is the original fresh runtime funding baseline. No host claim
    // or host reconciliation is used to manufacture an expected runtime effect.
    accounts
}
fn recognition(a: &Accounts) -> Instruction {
    Instruction { program_id: program_id(), accounts: vec![AccountMeta::new(a[CONFIG].0, false),
        AccountMeta::new_readonly(a[ROUND].0, false), AccountMeta::new_readonly(a[SOL].0, false),
        AccountMeta::new_readonly(a[TOKEN].0, false)], data: encode_reconcile_pending().to_vec() }
}
fn donation(a: &Accounts, amount: u64) -> Instruction {
    let mut data = 2_u32.to_le_bytes().to_vec(); data.extend(amount.to_le_bytes());
    Instruction { program_id: a[SYSTEM].0,
        accounts: vec![AccountMeta::new(a[DONOR].0, true), AccountMeta::new(a[SOL].0, false)], data }
}
fn log_accounts(label: &str, stage: &str, ordinal: Option<usize>, a: &Accounts) {
    for (i, (key, account)) in a.iter().enumerate() {
        println!("{}", account_evidence(label, stage, ordinal, i, key, account));
    }
}
fn execute(runtime: &Runtime, label: &str, a: &Accounts, instructions: &[Instruction]) -> TransactionResult {
    log_accounts(label, "supplied", None, a);
    let result = runtime.vm.process_transaction_instructions(instructions, a, Some(&a[PAYER].0));
    log_accounts(label, "returned", None, &result.resulting_accounts);
    let observations = runtime.observations.borrow();
    for (i, raw) in observations.after.iter().enumerate() {
        log_accounts(label, "raw-after-invocation", Some(i), &raw.accounts);
        assert!(support::event_bytes(&raw.logs).is_empty(), "pending recognition emits no claim event");
    }
    println!("CASE {label} RESULT {result:#?} RAW {observations:#?}");
    let message = result.message.as_ref().unwrap();
    assert_eq!(message.account_keys()[0], a[PAYER].0);
    assert!(message.is_signer(0) && message.is_writable(0));
    assert!(result.inner_instructions.iter().all(Vec::is_empty), "neither pending recognition nor top-level System donation invokes CPI");
    let total = |a: &Accounts| a.iter().map(|(_, a)| u128::from(a.lamports)).sum::<u128>();
    assert_eq!(total(a), total(&result.resulting_accounts));
    result
}
fn assert_raw(actual: &Accounts, expected: &Accounts) {
    for (key, account) in actual {
        assert_eq!(account, &expected.iter().find(|(k, _)| k == key).unwrap().1, "raw account {key}");
    }
    for i in [CONFIG, ROUND, SOL, TOKEN, PROGRAM, PAYER] {
        assert!(actual.iter().any(|(key, _)| *key == expected[i].0));
    }
}

#[test]
fn system_donation_then_two_recognitions_executes_at_ordinary_budget() {
    let runtime = Runtime::new(200_000); let original = fixture(&runtime, false, 0);
    let ix = recognition(&original);
    assert!(!ix.accounts[SOL].is_writable);
    let result = execute(&runtime, "pending-system-donation-recognize-repeat", &original,
        &[donation(&original, 100), ix.clone(), ix]);
    assert_eq!(result.raw_result, Ok(())); assert!(result.compute_units_consumed <= 200_000);
    let mut funded = original.clone(); funded[DONOR].1.lamports -= 100; funded[SOL].1.lamports += 100;
    let mut expected = funded.clone(); support::edit_config(&mut expected, |c| {
        c.accounted_pending_sol_lamports = 100; c.accounted_pending_jitosol_units = 20;
    });
    assert_eq!(result.resulting_accounts, expected);
    let obs = runtime.observations.borrow(); assert_eq!(obs.after.len(), 3);
    assert_raw(&obs.after[0].accounts, &funded);
    for raw in &obs.after[1..] { assert_raw(&raw.accounts, &expected); }
    for privileges in &obs.privileges[1..] {
        assert_eq!(privileges.len(), 4);
        assert!(privileges[SOL].2, "compiled transaction union includes prior System transfer destination");
        assert!(!privileges[SOL].1 && !privileges[TOKEN].1);
    }
}
#[test]
fn paused_pending_token_native_excess_is_retained_without_sol_credit() {
    for excess in [1, 1_000_000] {
        let runtime = Runtime::new(200_000); let original = fixture(&runtime, true, excess);
        let result = execute(&runtime, "pending-paused-token-native-excess", &original, &[recognition(&original)]);
        assert_eq!(result.raw_result, Ok(()));
        let mut expected = original.clone(); support::edit_config(&mut expected, |c| c.accounted_pending_jitosol_units = 20);
        assert_eq!(result.resulting_accounts, expected);
        assert_raw(&runtime.observations.borrow().after[0].accounts, &expected);
    }
}
#[test]
fn pending_joint_deficits_and_native_rent_fail_without_partial_recognition() {
    for which in 0..3 {
        let runtime = Runtime::new(200_000); let mut original = fixture(&runtime, false, 1);
        match which { 0 => support::edit_config(&mut original, |c| c.accounted_pending_sol_lamports = 1),
            1 => { original[SOL].1.lamports += 100;
                support::edit_config(&mut original, |c| c.accounted_pending_jitosol_units = 21); },
            _ => original[TOKEN].1.lamports = runtime.vm.sysvars.rent.minimum_balance(original[TOKEN].1.data.len()) - 1 }
        let result = execute(&runtime, "pending-deficit", &original, &[recognition(&original)]);
        let error = if which == 2 { Piv1Error::AccountRentDeficit } else { Piv1Error::PendingCustodyDeficit };
        assert_eq!(result.raw_result, Err(TransactionError::InstructionError(0, super::failure(error))));
        assert_eq!(result.resulting_accounts, original);
        assert_raw(&runtime.observations.borrow().after[0].accounts, &original);
    }
}
#[test]
fn pending_abi_alias_and_config_privilege_failures_keep_original_accounts() {
    for which in 0..3 {
        let runtime = Runtime::new(200_000); let original = fixture(&runtime, false, 0);
        let mut ix = recognition(&original);
        let error = match which { 0 => { ix.data.push(0); InstructionError::InvalidInstructionData },
            1 => { ix.accounts[CONFIG].is_writable = false; super::failure(Piv1Error::AccountNotWritable) },
            _ => { ix.accounts[TOKEN].pubkey = ix.accounts[SOL].pubkey; super::failure(Piv1Error::AccountAlias) } };
        let result = execute(&runtime, "pending-dispatch-reject", &original, &[ix]);
        assert_eq!(result.raw_result, Err(TransactionError::InstructionError(0, error)));
        assert_eq!(result.resulting_accounts, original);
        // A duplicate instruction role need not load the omitted token account.
        for (key, account) in &runtime.observations.borrow().after[0].accounts {
            assert_eq!(account, &original.iter().find(|(k, _)| k == key).unwrap().1);
        }
    }
}
#[test]
fn successful_recognition_then_bad_instruction_exposes_raw_effects_and_output_discard() {
    let runtime = Runtime::new(200_000); let original = fixture(&runtime, false, 1);
    let good = recognition(&original); let mut bad = good.clone(); bad.data.push(0);
    let result = execute(&runtime, "pending-success-then-fail", &original, &[good.clone(), bad, good]);
    assert_eq!(result.raw_result, Err(TransactionError::InstructionError(1, InstructionError::InvalidInstructionData)));
    assert_eq!(result.resulting_accounts, original);
    let mut expected = original.clone(); support::edit_config(&mut expected, |c| c.accounted_pending_jitosol_units = 20);
    let obs = runtime.observations.borrow(); assert_eq!(obs.after.len(), 2);
    for raw in &obs.after { assert_raw(&raw.accounts, &expected); }
    // Actual first-instruction bytes survive in this failed shared context.
    // Returned originals prove Mollusk discard, not Bank transaction rollback.
}
