//! Exact initializer probe ELFs through real Bank storage and loader accounts.
//! Local unsigned entry only: no deployment, key, signature or public transport.
#[path = "genesis_fixture.rs"]
mod genesis_fixture;

use {
    genesis_fixture::{key, program_data, World, JITO_PROGRAM, PROGRAM, SIZES, SQUADS, TOKEN},
    serde_json::{json, Value},
    sha2::{Digest, Sha256},
    solana_account::{AccountSharedData, ReadableAccount, WritableAccount},
    solana_compute_budget_interface::ComputeBudgetInstruction,
    solana_genesis_config::GenesisConfig,
    solana_instruction_error::InstructionError,
    solana_loader_v3_interface::state::UpgradeableLoaderState,
    solana_message::Message,
    solana_pubkey::Pubkey,
    solana_runtime::bank::{Bank, BankTestConfig, SlotLeader},
    solana_sdk_ids::{bpf_loader_upgradeable, system_program, sysvar},
    solana_svm::{transaction_commit_result::{CommittedTransaction, TransactionCommitResult},
        transaction_processor::ExecutionRecordingConfig},
    solana_transaction::Transaction,
    solana_transaction_error::TransactionError,
    std::{collections::BTreeMap, env, fs, num::NonZeroUsize, path::Path},
};

const LIMIT: u32 = 1_400_000;
const FEE: u64 = 15_000;
const INITIAL_FEE_BALANCE: u64 = 100_000_000;
type Accounts = BTreeMap<Pubkey, AccountSharedData>;
type Cpi = (Pubkey, Vec<u8>, Vec<Pubkey>, u8);

fn hex(bytes: &[u8]) -> String { bytes.iter().map(|byte| format!("{byte:02x}")).collect() }
fn emit(value: Value) { println!("PIV1_BANK_GENESIS_EVIDENCE {value}"); }

fn snapshot(bank: &Bank, world: &World, profile: &str, phase: &str) -> Accounts {
    assert_eq!(bank.slot(), 1);
    let mut scanned = BTreeMap::new();
    bank.scan_all_accounts(|item| {
        if let Some((pubkey, enumerated, slot)) = item {
            assert!(slot <= 1);
            assert!(scanned.insert(*pubkey, enumerated).is_none(), "unique scanned account");
        }
    }).expect("complete ancestry-visible account scan");
    // The upstream scan holds an index slot-list read lock inside its callback.
    // Finish that scan before any independent read can reacquire the same lock.
    let mut accounts = Accounts::new();
    for (pubkey, enumerated) in scanned {
        let reread = bank.get_account(&pubkey);
        if enumerated.lamports() == 0 {
            assert!(reread.is_none(), "zero balance means absent");
            continue;
        }
        let account = reread.expect("scan entry must be readable");
        assert_eq!(account, enumerated, "full ancestry scan and independent Bank read");
        assert!(accounts.insert(pubkey, account).is_none(), "unique visible account");
    }
    for (pubkey, account) in &accounts {
        emit(json!({"kind": "account", "profile": profile, "phase": phase,
            "pubkey": pubkey.to_string(), "pubkey_hex": hex(pubkey.as_ref()),
            "lamports": account.lamports(), "owner": account.owner().to_string(),
            "owner_hex": hex(account.owner().as_ref()), "executable": account.executable(),
            "rent_epoch": account.rent_epoch(), "data_len": account.data().len(), "data_hex": hex(account.data())}));
    }
    for slot in 0..16 {
        let address = world.roles[7 + slot].key;
        let account = bank.get_account(&address);
        assert_eq!(account.as_ref(), accounts.get(&address));
        emit(json!({"kind": "target_presence", "profile": profile, "phase": phase,
            "target_slot": slot, "pubkey": address.to_string(), "present": account.is_some()}));
    }
    assert!(!accounts.contains_key(&sysvar::instructions::id()), "Instructions is transaction-local");
    emit(json!({"kind": "snapshot", "profile": profile, "phase": phase,
        "account_count": accounts.len(), "slot": bank.slot(), "parent_slot": bank.parent_slot(),
        "transaction_count": bank.transaction_count(), "signature_count": bank.signature_count()}));
    accounts
}

fn add_program(genesis: &mut GenesisConfig, role: &str, program: Pubkey, authority: Option<Pubkey>, native: bool) {
    let (length, sha256) = match role {
        "CALLER" => (61_080, "e7fc7bf5d75a9494fd3a4787733df53adf8c3b724653a113ae36a9574707ec97"),
        "CALLEE" => (376_720, "07934627a3fdab928ab1aca2abf424eb683ea9e680681389c9b53dc2391a66b7"),
        "TOKEN" => (126_424, "c0f42a30da4079601711bec29bd0ca780674654ea71790eb32c76b5cedad4499"),
        _ => unreachable!(),
    };
    // Native artifacts are pinned independently in the reviewed runner profile;
    // legacy artifacts retain their literal historical identities above.
    let supplied_hash = env::var(format!("PIV_GENESIS_{role}_SHA256")).unwrap();
    let supplied_length = env::var(format!("PIV_GENESIS_{role}_BYTES")).unwrap().parse::<usize>().unwrap();
    let (length, sha256) = if native { (supplied_length, supplied_hash.as_str()) } else { (length, sha256) };
    assert_eq!(supplied_hash, sha256);
    assert_eq!(env::var(format!("PIV_GENESIS_{role}_BYTES")).unwrap().parse::<usize>().unwrap(), length);
    let elf = fs::read(env::var_os(format!("PIV_GENESIS_{role}_PATH")).unwrap()).unwrap();
    assert_eq!(elf.len(), length); assert_eq!(format!("{:x}", Sha256::digest(&elf)), sha256);
    assert_eq!(&elf[..4], b"\x7fELF");
    let pd_key = program_data(program);
    let program_bytes = bincode::serialize(&UpgradeableLoaderState::Program { programdata_address: pd_key }).unwrap();
    assert_eq!(program_bytes.len(), 36);
    let mut pd_bytes = bincode::serialize(&UpgradeableLoaderState::ProgramData {
        slot: 0, upgrade_authority_address: authority }).unwrap();
    pd_bytes.resize(UpgradeableLoaderState::size_of_programdata_metadata(), 0);
    assert_eq!(pd_bytes.len(), 45); pd_bytes.extend(&elf);
    for (address, data, executable) in [(program, program_bytes, true), (pd_key, pd_bytes, false)] {
        let mut account = AccountSharedData::new(genesis.rent.minimum_balance(data.len()), data.len(), &bpf_loader_upgradeable::id());
        account.set_data_from_slice(&data); account.set_executable(executable); account.set_rent_epoch(u64::MAX);
        genesis.accounts.insert(address, account.into());
    }
    emit(json!({"kind": "artifact", "role": role, "program": program.to_string(),
        "program_data": pd_key.to_string(), "elf_sha256": sha256, "elf_bytes": length,
        "deployment_slot": 0, "effective_slot": 1, "loader": bpf_loader_upgradeable::id().to_string(),
        "upgrade_authority": authority.map(|key| key.to_string())}));
}

fn make_genesis(world: &World) -> GenesisConfig {
    let mut genesis = GenesisConfig { creation_time: 100, ..GenesisConfig::default() };
    genesis.fee_rate_governor.lamports_per_signature = 5_000;
    genesis.fee_rate_governor.target_lamports_per_signature = 5_000;
    genesis.fee_rate_governor.target_signatures_per_slot = 0;
    genesis.fee_rate_governor.min_lamports_per_signature = 5_000;
    genesis.fee_rate_governor.max_lamports_per_signature = 5_000;
    genesis.fee_rate_governor.burn_percent = 50;
    genesis.rent.lamports_per_byte = 6_960;
    for (index, role) in world.roles.iter().enumerate() {
        if [sysvar::instructions::id(), system_program::id(), TOKEN, PROGRAM, program_data(PROGRAM)].contains(&role.key)
            || role.lamports == 0 { continue; }
        if !(7..23).contains(&index) {
            assert!(role.lamports >= genesis.rent.minimum_balance(role.data.len()));
        }
        assert!(genesis.accounts.insert(role.key, role.account().into()).is_none());
    }
    for (address, lamports) in [(key(242), INITIAL_FEE_BALANCE), (key(91), 1_000_000), (key(243), 3_000_000)] {
        let data: Vec<u8> = if address == key(243) { (0..64).collect() } else { vec![] };
        let owner = if address == key(243) { key(244) } else { system_program::id() };
        let mut account = AccountSharedData::new(lamports, data.len(), &owner);
        account.set_data_from_slice(&data); account.set_rent_epoch(u64::MAX);
        assert!(genesis.accounts.insert(address, account.into()).is_none());
    }
    let native = world.inner_data.starts_with(b"PIV1IN01");
    add_program(&mut genesis, "CALLER", SQUADS, None, native);
    add_program(&mut genesis, "CALLEE", PROGRAM, Some(world.vault), native);
    add_program(&mut genesis, "TOKEN", TOKEN, None, native);
    genesis
}

fn bounded_config() -> BankTestConfig {
    let mut config = BankTestConfig::default();
    let db = &mut config.accounts_db_config;
    db.num_foreground_threads = NonZeroUsize::new(1); db.num_background_threads = NonZeroUsize::new(1);
    db.read_cache_limit_bytes = Some((2 * 1024 * 1024, 4 * 1024 * 1024));
    db.read_cache_num_shards = Some(16); db.read_cache_evict_sample_size = Some(4);
    db.write_cache_limit_bytes = Some(8 * 1024 * 1024);
    let index = db.index.as_mut().unwrap(); index.bins = Some(2);
    index.num_flush_threads = NonZeroUsize::new(1); index.num_initial_accounts = Some(128);
    config
}

fn execute(bank: &Bank, profile: &str, case: &str, tx: &Transaction) -> TransactionCommitResult {
    assert_eq!(tx.message.header.num_required_signatures, 3);
    assert_eq!(tx.signatures.len(), 3);
    assert!(tx.signatures.iter().all(|signature| signature.as_ref().iter().all(|byte| *byte == 0)));
    assert_eq!(tx.message.account_keys[0], key(242));
    assert_eq!(tx.message.recent_blockhash, bank.last_blockhash());
    // Legacy wire format: shortvec(3) is one byte, followed by three signatures.
    let message_bytes = tx.message.serialize();
    let wire_len = 1 + 3 * 64 + message_bytes.len();
    assert!(wire_len > 1232, "this fixture explicitly requires future packet transport work");
    emit(json!({"kind": "transaction", "profile": profile, "case": case,
        "message_hex": hex(&message_bytes), "message_hash": tx.message.hash().to_string(),
        "account_keys": tx.message.account_keys.iter().map(ToString::to_string).collect::<Vec<_>>(),
        "signature_hex": tx.signatures.iter().map(|signature| hex(signature.as_ref())).collect::<Vec<_>>(),
        "required_signatures": 3, "expected_processed_fee": FEE, "wire_bytes": wire_len,
        "packet_limit_bytes": 1232, "packet_compatible": false, "compute_limit": LIMIT, "heap_bytes": 32768}));
    let batch = bank.prepare_entry_batch(vec![tx.clone().into()]).expect("unsigned local entry sanitation");
    let (mut results, _) = bank.load_execute_and_commit_transactions(&batch,
        ExecutionRecordingConfig::new_single_setting(true), &mut Default::default(), Some(256 * 1024));
    assert_eq!(results.len(), 1); let result = results.remove(0); drop(batch);
    match &result {
        Ok(committed) => {
            emit(json!({"kind": "commit", "profile": profile, "case": case, "committed": true,
                "status": format!("{:?}", committed.status), "executed_units": committed.executed_units,
                "transaction_fee": committed.fee_details.transaction_fee(), "priority_fee": committed.fee_details.prioritization_fee(),
                "total_fee": committed.fee_details.total_fee(), "fee_payer_post_balance": committed.fee_payer_post_balance,
                "logs": committed.log_messages}));
            for (outer_index, instructions) in committed.inner_instructions.as_ref().unwrap().iter().enumerate() {
                for (ordinal, inner) in instructions.iter().enumerate() {
                    let instruction = &inner.instruction;
                    emit(json!({"kind": "cpi", "profile": profile, "case": case,
                        "outer_index": outer_index, "ordinal": ordinal, "stack_height": inner.stack_height,
                        "program": tx.message.account_keys[usize::from(instruction.program_id_index)].to_string(),
                        "accounts": instruction.accounts.iter().map(|index| tx.message.account_keys[usize::from(*index)].to_string()).collect::<Vec<_>>(),
                        "data_hex": hex(&instruction.data)}));
                }
            }
        }
        Err(error) => emit(json!({"kind": "commit", "profile": profile, "case": case,
            "committed": false, "error": format!("{error:?}")})),
    }
    result
}

fn check_completed_initializer(world: &World, tx: &Transaction, committed: &CommittedTransaction, late: bool) {
    let inner = committed.inner_instructions.as_ref().expect("CPI recording enabled");
    assert_eq!(inner.len(), if late { 3 } else { 2 });
    assert!(inner[0].is_empty());
    if late { assert!(inner[2].is_empty()); }
    let observed: Vec<Cpi> = inner[1].iter().map(|inner| {
        let instruction = &inner.instruction;
        (tx.message.account_keys[usize::from(instruction.program_id_index)], instruction.data.clone(),
            instruction.accounts.iter().map(|index| tx.message.account_keys[usize::from(*index)]).collect(), inner.stack_height)
    }).collect();
    assert_eq!(observed, world.expected_cpis(), "complete ordered initializer/System/Token CPI stream");
    assert!(!observed.iter().any(|item| item.0 == JITO_PROGRAM));
    let logs = committed.log_messages.as_ref().expect("recorded SBF logs");
    let callee_success = format!("Program {PROGRAM} success");
    let caller_success = format!("Program {SQUADS} success");
    let callee_position = logs.iter().position(|line| line == &callee_success).expect("initializer completed");
    let caller_position = logs.iter().position(|line| line == &caller_success).expect("first caller completed");
    assert!(callee_position < caller_position);
    assert_eq!(logs.iter().filter(|line| *line == &format!("Program {TOKEN} success")).count(), 2);
    assert!(!logs.iter().any(|line| line.starts_with(&format!("Program {JITO_PROGRAM} invoke"))));
    if late {
        let failed = logs.iter().position(|line| line.starts_with(&format!("Program {SQUADS} failed:")))
            .expect("later malformed caller failed");
        assert!(caller_position < failed);
    } else { assert!(!logs.iter().any(|line| line.contains(" failed:"))); }
    assert_eq!(committed.fee_details.transaction_fee(), FEE);
    assert_eq!(committed.fee_details.prioritization_fee(), 0);
    assert_eq!(committed.fee_details.total_fee(), FEE);
    assert!(committed.executed_units > 0 && committed.executed_units <= u64::from(LIMIT));
}

fn debit(accounts: &mut Accounts, pubkey: Pubkey, amount: u64) {
    let account = accounts.get_mut(&pubkey).unwrap();
    account.set_lamports(account.lamports().checked_sub(amount).unwrap());
}

fn expected_success(world: &World, before: &Accounts) -> Accounts {
    let states = world.state_bytes(); let mut expected = before.clone();
    for (slot, size) in SIZES.into_iter().enumerate() {
        let data = if slot < 9 { states[slot].clone() } else if slot < 14 { vec![] } else { world.token_bytes() };
        assert_eq!(data.len(), size);
        let mut account = AccountSharedData::new(world.target_balance(slot), size, &World::target_owner(slot));
        account.set_data_from_slice(&data);
        // SVM account_loader creates absent accounts with RENT_EXEMPT_RENT_EPOCH.
        account.set_rent_epoch(u64::MAX);
        expected.insert(world.roles[7 + slot].key, account);
    }
    debit(&mut expected, world.roles[world.payer].key, world.shortfall());
    debit(&mut expected, key(242), FEE);
    expected
}

fn run_profile(base: &Path, shared: bool, prefunded: bool, native: bool) {
    let profile = format!("{}-{}", if shared { "shared34" } else { "distinct35" },
        if prefunded { "prefunded-paused" } else { "fresh-unpaused" });
    let accounts_path = base.join(&profile); fs::create_dir(&accounts_path).expect("fresh retained profile storage");
    let world = if native { World::production(shared, prefunded) } else { World::new(shared, prefunded) };
    let genesis = make_genesis(&world);
    let leader = SlotLeader { id: key(246), vote_address: key(247) };
    let root = Bank::new_with_paths_for_tests(&genesis, Some(bounded_config()), vec![accounts_path.clone()], Some(leader));
    assert!(root.feature_set.active().is_empty());
    let (root, forks) = root.wrap_with_bank_forks_for_tests();
    // This freezes slot zero normally and activates slot-zero deployments at one.
    let bank = Bank::new_from_parent_with_bank_forks(&forks, root, leader, 1);
    assert!(bank.feature_set.active().is_empty()); assert_eq!(bank.clock().unix_timestamp, 100);
    assert_eq!(bank.clock().slot, 1);
    assert_eq!(bank.get_account(&sysvar::clock::id()).unwrap().data(), bincode::serialize(&bank.clock()).unwrap());
    assert_eq!(bank.get_account(&sysvar::rent::id()).unwrap().data(), bincode::serialize(&genesis.rent).unwrap());
    for slot in 0..16 { assert_eq!(genesis_fixture::floor(slot), genesis.rent.minimum_balance(SIZES[slot])); }
    emit(json!({"kind": "fixture", "profile": profile, "role_count": world.roles.len(),
        "shared": shared, "initially_paused": prefunded, "accounts_path": accounts_path,
        "genesis_hash": genesis.hash().to_string(), "deployment_slot": 0, "bank_slot": 1,
        "clock": format!("{:?}", bank.clock()), "active_features": 0,
        "rent_lamports_per_byte": genesis.rent.lamports_per_byte,
        "rent_payer": world.roles[world.payer].key.to_string(), "fee_payer": key(242).to_string(),
        "executor": key(91).to_string(), "rent_shortfall": world.shortfall(), "native_sweep": world.sweep(),
        "compute_limit": LIMIT, "heap_bytes": 32768, "foreground_threads": 1, "background_threads": 1,
        "index_flush_threads": 1, "upstream_hash_pool_threads": 4}));
    let initial = snapshot(&bank, &world, &profile, "initial");
    for slot in 0..16 {
        let target = initial.get(&world.roles[7 + slot].key);
        assert_eq!(target.is_some(), world.roles[7 + slot].lamports > 0);
        if let Some(target) = target { assert_eq!(target, &world.roles[7 + slot].account()); }
    }
    assert_eq!(initial[&key(242)].lamports(), INITIAL_FEE_BALANCE);
    assert_eq!(initial[&key(222)].lamports(), 1_000_000_000);
    let mut malformed = world.outer.clone(); malformed.data[0] ^= 1;
    let budget = ComputeBudgetInstruction::set_compute_unit_limit(LIMIT);
    let failure = Transaction::new_unsigned(Message::new_with_blockhash(
        &[budget.clone(), world.outer.clone(), malformed], Some(&key(242)), &bank.last_blockhash()));
    let result = execute(&bank, &profile, "late_failure", &failure);
    let after_failure = snapshot(&bank, &world, &profile, "after_late_failure");
    let committed = result.as_ref().expect("executed failure reaches Bank commit");
    assert_eq!(committed.status, Err(TransactionError::InstructionError(2, InstructionError::InvalidInstructionData)));
    check_completed_initializer(&world, &failure, committed, true);
    assert_eq!(committed.fee_payer_post_balance, INITIAL_FEE_BALANCE - FEE);
    let mut expected = initial.clone(); debit(&mut expected, key(242), FEE);
    assert_eq!(after_failure, expected, "all non-fee accounts, including inherited/rent/absent targets, rolled back");
    let replay = failure.clone(); assert_eq!(replay, failure);
    let replay_result = execute(&bank, &profile, "exact_replay", &replay);
    let after_replay = snapshot(&bank, &world, &profile, "after_exact_replay");
    assert!(matches!(replay_result, Err(TransactionError::AlreadyProcessed)));
    assert_eq!(after_replay, after_failure, "replay retains every stored account and charges no fee");
    let retry = Transaction::new_unsigned(Message::new_with_blockhash(
        &[budget, world.outer.clone()], Some(&key(242)), &bank.last_blockhash()));
    assert_ne!(retry.message.serialize(), failure.message.serialize());
    assert_ne!(retry.message.hash(), failure.message.hash());
    let retry_result = execute(&bank, &profile, "distinct_retry", &retry);
    let after_retry = snapshot(&bank, &world, &profile, "after_distinct_retry");
    let committed = retry_result.as_ref().unwrap(); assert_eq!(committed.status, Ok(()));
    check_completed_initializer(&world, &retry, committed, false);
    assert_eq!(committed.fee_payer_post_balance, INITIAL_FEE_BALANCE - 2 * FEE);
    assert_eq!(after_retry, expected_success(&world, &after_replay), "independent literal complete initialized state");
    emit(json!({"kind": "profile_complete", "profile": profile, "status": "PASS", "message_cases": 3,
        "snapshots": 4, "complete_account_records": initial.len() + after_failure.len() + after_replay.len() + after_retry.len(),
        "fee_debits": [FEE, 0, FEE], "rent_debit": world.shortfall(), "native_sweep": world.sweep()}));
}

#[test]
fn bank_initializer_failure_replay_retry_four_profiles() { run_profiles(false); }

pub(crate) fn run_profiles(native: bool) {
    assert_eq!(env::var("RAYON_NUM_THREADS").as_deref(), Ok("1"));
    let path = env::var_os("PIV1_BANK_ACCOUNTS_DIR").expect("fresh retained storage base");
    let base = Path::new(&path); assert!(base.is_absolute());
    assert_eq!(base.parent().unwrap().canonicalize().unwrap(), base.parent().unwrap());
    fs::create_dir(base).expect("exclusive new AccountsDB base");
    for (shared, prefunded) in [(false, false), (true, false), (false, true), (true, true)] {
        run_profile(base, shared, prefunded, native);
    }
    emit(json!({"kind": "complete", "status": "PASS", "profiles": 4, "message_cases": 12,
        "successful_initializations": 4, "snapshots": 16, "no_packet_transport_claim": true}));
}
