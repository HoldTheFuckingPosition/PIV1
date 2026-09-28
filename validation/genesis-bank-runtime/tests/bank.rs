//! Unsigned local Bank entry and AccountsDB commit/rollback smoke only.
//! No PIV1 artifact, secret key, signature verification, RPC or ledger replay.

use {
    serde_json::{json, Value},
    solana_account::{AccountSharedData, ReadableAccount, WritableAccount},
    solana_genesis_config::GenesisConfig,
    solana_instruction_error::InstructionError,
    solana_message::Message,
    solana_pubkey::Pubkey,
    solana_runtime::bank::{Bank, BankTestConfig, SlotLeader},
    solana_svm::{
        transaction_commit_result::TransactionCommitResult,
        transaction_processor::ExecutionRecordingConfig,
    },
    solana_system_interface::{instruction::transfer, program as system_program},
    solana_transaction::Transaction,
    solana_transaction_error::TransactionError,
    std::{collections::BTreeMap, env, fs, num::NonZeroUsize, path::PathBuf},
};

const FEE_PER_SIGNATURE: u64 = 5_000;
const REQUIRED_SIGNATURES: u8 = 2;
const FEE: u64 = FEE_PER_SIGNATURE * REQUIRED_SIGNATURES as u64;
const INITIAL_FEE_PAYER: u64 = 100_000_000;
const INITIAL_SOURCE: u64 = 10_000_000;
const INITIAL_DESTINATION: u64 = 2_000_000;
const FIRST_SUCCESS: u64 = 10_000;
const FIRST_LEG: u64 = 70;
const IMPOSSIBLE_SECOND_LEG: u64 = 1_000_000_000;
const RETRY_SECOND_LEG: u64 = 9;
const CREATION_TIME: i64 = 1_800_000_000;

type Accounts = BTreeMap<Pubkey, AccountSharedData>;

fn key(byte: u8) -> Pubkey {
    Pubkey::new_from_array([byte; 32])
}

fn hex(bytes: &[u8]) -> String {
    bytes.iter().map(|byte| format!("{byte:02x}")).collect()
}

fn emit(value: Value) {
    println!("PIV1_BANK_EVIDENCE {value}");
}

// A slot-zero Bank has no parent: this enumerates every account in its state.
// Every enumerated account is independently re-read through Bank::get_account.
fn snapshot(bank: &Bank, phase: &str) -> Accounts {
    assert_eq!(bank.slot(), 0);
    assert!(bank.parent().is_none());
    let mut accounts = BTreeMap::new();
    for (pubkey, enumerated) in bank.get_all_accounts_modified_since_parent() {
        let reread = bank.get_account(&pubkey).expect("enumerated account exists");
        assert_eq!(enumerated, reread, "enumeration and independent Bank read");
        assert!(accounts.insert(pubkey, reread).is_none(), "unique account");
    }
    for (pubkey, account) in &accounts {
        emit(json!({
            "kind": "account", "phase": phase, "pubkey": pubkey.to_string(),
            "pubkey_hex": hex(pubkey.as_ref()), "lamports": account.lamports(),
            "owner": account.owner().to_string(), "owner_hex": hex(account.owner().as_ref()),
            "executable": account.executable(), "rent_epoch": account.rent_epoch(),
            "data_len": account.data().len(), "data_hex": hex(account.data()),
        }));
    }
    emit(json!({
        "kind": "snapshot", "phase": phase, "account_count": accounts.len(),
        "slot": bank.slot(), "transaction_count": bank.transaction_count(),
        "signature_count": bank.signature_count(),
    }));
    accounts
}

fn execute(bank: &Bank, name: &str, transaction: &Transaction) -> TransactionCommitResult {
    assert_eq!(transaction.message.header.num_required_signatures, REQUIRED_SIGNATURES);
    assert_eq!(transaction.signatures.len(), usize::from(REQUIRED_SIGNATURES));
    assert!(transaction.signatures.iter().all(|signature| {
        signature.as_ref().iter().all(|byte| *byte == 0)
    }));
    assert_eq!(transaction.message.recent_blockhash, bank.last_blockhash());
    emit(json!({
        "kind": "transaction", "case": name,
        "message_hex": hex(&transaction.message.serialize()),
        "message_hash": transaction.message.hash().to_string(),
        "required_signatures": transaction.message.header.num_required_signatures,
        "signature_hex": transaction.signatures.iter().map(|s| hex(s.as_ref())).collect::<Vec<_>>(),
        "blockhash": transaction.message.recent_blockhash.to_string(),
        "fee_per_signature": FEE_PER_SIGNATURE, "expected_processed_fee": FEE,
    }));
    let batch = bank.prepare_entry_batch(vec![transaction.clone().into()])
        .expect("unsigned structurally valid local entry");
    let (mut results, _) = bank.load_execute_and_commit_transactions(
        &batch,
        ExecutionRecordingConfig::new_single_setting(true),
        &mut Default::default(),
        Some(64 * 1024),
    );
    assert_eq!(results.len(), 1);
    let result = results.remove(0);
    // Release entry locks before the next message; no Bank/cache/state reset.
    drop(batch);
    match &result {
        Ok(committed) => emit(json!({
            "kind": "commit", "case": name, "committed": true,
            "status": format!("{:?}", committed.status),
            "executed_units": committed.executed_units,
            "transaction_fee": committed.fee_details.transaction_fee(),
            "priority_fee": committed.fee_details.prioritization_fee(),
            "total_fee": committed.fee_details.total_fee(),
            "fee_payer_post_balance": committed.fee_payer_post_balance,
            "logs": committed.log_messages,
        })),
        Err(error) => emit(json!({
            "kind": "commit", "case": name, "committed": false,
            "error": format!("{error:?}"),
        })),
    }
    result
}

fn adjust(accounts: &mut Accounts, pubkey: Pubkey, debit: u64, credit: u64) {
    let account = accounts.get_mut(&pubkey).expect("known account");
    let lamports = account.lamports().checked_sub(debit).unwrap()
        .checked_add(credit).unwrap();
    account.set_lamports(lamports);
}

fn assert_fee(result: &TransactionCommitResult, expected_balance: u64) {
    let committed = result.as_ref().expect("processed transaction committed");
    assert_eq!(committed.fee_details.transaction_fee(), FEE);
    assert_eq!(committed.fee_details.prioritization_fee(), 0);
    assert_eq!(committed.fee_details.total_fee(), FEE);
    assert_eq!(committed.fee_payer_post_balance, expected_balance);
}

#[test]
fn bank_commit_rollback_fee_replay_and_retry() {
    assert_eq!(env::var("RAYON_NUM_THREADS").as_deref(), Ok("1"));
    let accounts_path = PathBuf::from(env::var_os("PIV1_BANK_ACCOUNTS_DIR")
        .expect("root must provide a fresh retained AccountsDB directory"));
    assert!(accounts_path.is_absolute());
    assert_eq!(accounts_path.parent().unwrap().canonicalize().unwrap(),
        accounts_path.parent().unwrap(), "parent must be canonical");
    // create_dir refuses an existing directory, including a symlink. No cleanup.
    fs::create_dir(&accounts_path).expect("fresh AccountsDB directory");

    let payer = key(41);
    let source = key(42);
    let destination = key(43);
    let second_destination = key(44);
    let sentinel = key(45);
    let mut genesis = GenesisConfig {
        creation_time: CREATION_TIME,
        ..GenesisConfig::default()
    };
    genesis.fee_rate_governor.lamports_per_signature = FEE_PER_SIGNATURE;
    genesis.fee_rate_governor.target_lamports_per_signature = FEE_PER_SIGNATURE;
    genesis.fee_rate_governor.target_signatures_per_slot = 0;
    genesis.fee_rate_governor.min_lamports_per_signature = FEE_PER_SIGNATURE;
    genesis.fee_rate_governor.max_lamports_per_signature = FEE_PER_SIGNATURE;
    genesis.fee_rate_governor.burn_percent = 50;
    genesis.rent.lamports_per_byte = 6_960;
    assert_eq!(genesis.rent.minimum_balance(0), 890_880);
    for (pubkey, lamports) in [
        (payer, INITIAL_FEE_PAYER), (source, INITIAL_SOURCE),
        (destination, INITIAL_DESTINATION), (second_destination, INITIAL_DESTINATION),
    ] {
        let mut account = AccountSharedData::new(lamports, 0, &system_program::id());
        account.set_rent_epoch(u64::MAX);
        genesis.accounts.insert(pubkey, account.into());
    }
    let mut sentinel_account = AccountSharedData::new(3_000_000, 64, &key(77));
    sentinel_account.set_data_from_slice(&(0..64).collect::<Vec<u8>>());
    sentinel_account.set_rent_epoch(u64::MAX);
    genesis.accounts.insert(sentinel, sentinel_account.into());

    let mut config = BankTestConfig::default();
    let db = &mut config.accounts_db_config;
    db.num_foreground_threads = NonZeroUsize::new(1);
    db.num_background_threads = NonZeroUsize::new(1);
    db.read_cache_limit_bytes = Some((2 * 1024 * 1024, 4 * 1024 * 1024));
    db.read_cache_num_shards = Some(16);
    db.read_cache_evict_sample_size = Some(4);
    db.write_cache_limit_bytes = Some(8 * 1024 * 1024);
    let index = db.index.as_mut().expect("test constructor has an in-memory index");
    index.bins = Some(2);
    index.num_flush_threads = NonZeroUsize::new(1);
    index.num_initial_accounts = Some(128);
    let leader = SlotLeader { id: key(46), vote_address: key(47) };
    let bank = Bank::new_with_paths_for_tests(
        &genesis, Some(config), vec![accounts_path.clone()], Some(leader),
    );
    assert!(bank.feature_set.active().is_empty(), "no activated genesis features");
    let (bank, _bank_forks) = bank.wrap_with_bank_forks_for_tests();
    emit(json!({
        "kind": "fixture", "scope": "native-system-bank-smoke",
        "accounts_path": accounts_path, "creation_time": CREATION_TIME,
        "genesis_hash": genesis.hash().to_string(), "slot": bank.slot(),
        "active_features": 0, "inactive_features": bank.feature_set.inactive().len(),
        "rent_lamports_per_byte": genesis.rent.lamports_per_byte,
        "rent_minimum_zero_data": genesis.rent.minimum_balance(0),
        "fee_per_signature": FEE_PER_SIGNATURE, "fee_burn_percent": 50,
        "payer": payer.to_string(), "source": source.to_string(),
        "destination": destination.to_string(), "second_destination": second_destination.to_string(),
        "sentinel": sentinel.to_string(), "foreground_threads": 1,
        "background_threads": 1, "index_flush_threads": 1,
        "index_bins": 2, "index_initial_accounts": 128,
        "read_cache_low_bytes": 2 * 1024 * 1024, "read_cache_high_bytes": 4 * 1024 * 1024,
        "read_cache_shards": 16, "write_cache_bytes": 8 * 1024 * 1024,
        "upstream_async_hash_pool_threads": 4, "rayon_threads": 1,
    }));
    let initial = snapshot(&bank, "initial");
    assert_eq!(initial[&payer].lamports(), INITIAL_FEE_PAYER);
    assert_eq!(initial[&source].lamports(), INITIAL_SOURCE);
    assert_eq!(initial[&destination].lamports(), INITIAL_DESTINATION);
    assert_eq!(initial[&second_destination].lamports(), INITIAL_DESTINATION);
    assert_eq!(initial[&sentinel].data(), &(0..64).collect::<Vec<u8>>());

    let success = Transaction::new_unsigned(Message::new_with_blockhash(
        &[transfer(&source, &destination, FIRST_SUCCESS)], Some(&payer), &bank.last_blockhash(),
    ));
    let success_result = execute(&bank, "success", &success);
    let after_success = snapshot(&bank, "after_success");
    assert_eq!(success_result.as_ref().unwrap().status, Ok(()));
    assert_fee(&success_result, INITIAL_FEE_PAYER - FEE);
    let mut expected = initial.clone();
    adjust(&mut expected, payer, FEE, 0);
    adjust(&mut expected, source, FIRST_SUCCESS, 0);
    adjust(&mut expected, destination, 0, FIRST_SUCCESS);
    assert_eq!(after_success, expected, "exact successful persisted state");

    let failure = Transaction::new_unsigned(Message::new_with_blockhash(
        &[transfer(&source, &destination, FIRST_LEG),
          transfer(&source, &second_destination, IMPOSSIBLE_SECOND_LEG)],
        Some(&payer), &bank.last_blockhash(),
    ));
    let failure_result = execute(&bank, "late_failure", &failure);
    let after_failure = snapshot(&bank, "after_late_failure");
    let failed = failure_result.as_ref().expect("executed failure is committed");
    assert_eq!(failed.status, Err(TransactionError::InstructionError(1, InstructionError::Custom(1))));
    assert_fee(&failure_result, INITIAL_FEE_PAYER - 2 * FEE);
    let logs = failed.log_messages.as_ref().expect("recorded native logs");
    let success_log = format!("Program {} success", system_program::id());
    let insufficient_log = format!(
        "Transfer: insufficient lamports {}, need {}",
        INITIAL_SOURCE - FIRST_SUCCESS - FIRST_LEG, IMPOSSIBLE_SECOND_LEG,
    );
    let first_success_position = logs.iter().position(|line| line == &success_log)
        .expect("first instruction succeeded before failure");
    let insufficient_position = logs.iter().position(|line| line == &insufficient_log)
        .expect("second instruction observed source after first transfer");
    assert!(first_success_position < insufficient_position);
    adjust(&mut expected, payer, FEE, 0);
    assert_eq!(after_failure, expected, "all non-fee accounts exactly rolled back");

    let replay = failure.clone();
    assert_eq!(replay, failure, "exact unsigned message and signature replay");
    let replay_result = execute(&bank, "exact_replay", &replay);
    let after_replay = snapshot(&bank, "after_exact_replay");
    assert!(matches!(replay_result, Err(TransactionError::AlreadyProcessed)));
    assert_eq!(after_replay, after_failure, "replay charges no second fee");

    let retry = Transaction::new_unsigned(Message::new_with_blockhash(
        &[transfer(&source, &destination, FIRST_LEG),
          transfer(&source, &second_destination, RETRY_SECOND_LEG)],
        Some(&payer), &bank.last_blockhash(),
    ));
    assert_ne!(retry.message.serialize(), failure.message.serialize());
    assert_ne!(retry.message.hash(), failure.message.hash());
    let retry_result = execute(&bank, "distinct_retry", &retry);
    let after_retry = snapshot(&bank, "after_distinct_retry");
    assert_eq!(retry_result.as_ref().unwrap().status, Ok(()));
    assert_fee(&retry_result, INITIAL_FEE_PAYER - 3 * FEE);
    adjust(&mut expected, payer, FEE, 0);
    adjust(&mut expected, source, FIRST_LEG + RETRY_SECOND_LEG, 0);
    adjust(&mut expected, destination, 0, FIRST_LEG);
    adjust(&mut expected, second_destination, 0, RETRY_SECOND_LEG);
    assert_eq!(after_retry, expected, "retry persisted from actual failed/replayed Bank state");
    assert_eq!(after_retry.len(), initial.len());
    emit(json!({
        "kind": "complete", "status": "PASS", "message_cases": 4,
        "snapshots": 5, "complete_account_records": initial.len() * 5,
        "fee_debits": [FEE, FEE, 0, FEE], "same_bank_slot": bank.slot(),
        "account_directory_retained": true,
    }));
}
