//! PIV1 SBF and native System payout against five real Bank rent profiles.
//! Local unsigned fixture state only. No authority, genesis instruction or delayed recovery is exercised.

#[allow(dead_code, unused_imports)]
mod lifecycle_fixture;

use {
    lifecycle_fixture::{account, claim, key, meta, program_data, put64, target, Config, World, PROGRAM},
    serde_json::json,
    sha2::{Digest, Sha256},
    solana_account::{AccountSharedData, ReadableAccount, WritableAccount},
    solana_compute_budget_interface::ComputeBudgetInstruction,
    solana_genesis_config::GenesisConfig,
    solana_instruction::Instruction,
    solana_instruction_error::InstructionError,
    solana_loader_v3_interface::state::UpgradeableLoaderState,
    solana_message::Message,
    solana_pubkey::Pubkey,
    solana_runtime::bank::{Bank, BankTestConfig, SlotLeader},
    solana_sdk_ids::{bpf_loader_upgradeable, system_program, sysvar},
    solana_svm::{transaction_commit_result::CommittedTransaction, transaction_processor::ExecutionRecordingConfig},
    solana_transaction::Transaction,
    solana_transaction_error::TransactionError,
    std::{collections::BTreeMap, env, fs, num::NonZeroUsize, path::PathBuf},
};

const STORAGE: u64 = 128 * 5_080;
const EARNED: u64 = 2_000_000;
const COLLECTIVE_CARRY: u64 = 100_000;
const PRINCIPAL_HWM: u64 = 15_000_045_712;
const FEE: u64 = 10_000;
const CONFIG_LIABILITY: usize = 7;
const CONFIG_PAID: usize = 14;
const REWARD_CLAIMABLE: usize = 52;
const REWARD_PAID: usize = 68;
const RUNTIME_RENT_DEFICIT: u32 = 6020;
const INVALID_RENT: u32 = 6021;
const STALE_CLAIM: u32 = 6004;

type Snapshot = BTreeMap<Pubkey, AccountSharedData>;

fn get_accounts(bank: &Bank, keys: impl Iterator<Item = Pubkey>) -> Snapshot {
    keys.map(|address| (address, bank.get_account(&address).expect("fixture account"))).collect()
}

fn transfer_record(accounts: &mut Snapshot, key: Pubkey, debit: u64, credit: u64) {
    let account = accounts.get_mut(&key).unwrap();
    account.set_lamports(account.lamports().checked_sub(debit).unwrap().checked_add(credit).unwrap());
}

fn owned_claim(amount: u64, prior_paid: u64) -> Instruction {
    let mut instruction = claim(amount);
    put64(&mut instruction.data, 16, prior_paid);
    instruction
}

fn top_up_kif(amount: u64) -> Instruction {
    let mut data = b"PIV1RT01".to_vec();
    data.push(4);
    data.extend(amount.to_le_bytes());
    Instruction { program_id: PROGRAM, data, accounts: vec![
        meta(target(0), true, false), meta(target(1), false, false), meta(target(13), true, false),
        meta(key(220), true, true), meta(system_program::id(), false, false),
    ] }
}

fn expected_claim(accounts: &mut Snapshot, amount: u64, claimed: &mut u64) {
    let config_address = target(0);
    let reward_address = target(3);
    transfer_record(accounts, target(13), amount, 0);
    transfer_record(accounts, key(96), 0, amount);
    *claimed += amount;
    let mut config = Config::read(&accounts[&config_address]);
    config.n[CONFIG_LIABILITY] = EARNED - *claimed;
    config.n[CONFIG_PAID] = *claimed;
    accounts.get_mut(&config_address).unwrap().set_data_from_slice(&config.bytes());
    let mut reward = accounts[&reward_address].data().to_vec();
    put64(&mut reward, REWARD_CLAIMABLE, EARNED - *claimed);
    put64(&mut reward, REWARD_PAID, *claimed);
    accounts.get_mut(&reward_address).unwrap().set_data_from_slice(&reward);
}

fn execute(bank: &Bank, instructions: &[Instruction]) -> CommittedTransaction {
    let fee_payer = key(242);
    let transaction = Transaction::new_unsigned(Message::new_with_blockhash(
        instructions, Some(&fee_payer), &bank.last_blockhash(),
    ));
    assert_eq!(transaction.message.header.num_required_signatures, 2);
    assert!(transaction.signatures.iter().all(|signature| signature.as_ref().iter().all(|byte| *byte == 0)));
    let batch = bank.prepare_entry_batch(vec![transaction.into()]).unwrap();
    let (mut results, _) = bank.load_execute_and_commit_transactions(
        &batch, ExecutionRecordingConfig::new_single_setting(true), &mut Default::default(), Some(64 * 1024),
    );
    assert_eq!(results.len(), 1);
    let receipt = results.remove(0).expect("executed Bank entry");
    assert_eq!(receipt.fee_details.total_fee(), FEE);
    assert!(receipt.executed_units > 0);
    receipt
}

fn fixture() -> Snapshot {
    let world = World::production(false, false);
    let genesis_data = world.state_bytes();
    let config_address = target(0);
    let reward_address = target(3);
    let mut config = Config::read(&account(PROGRAM, 1, genesis_data[0].clone()));
    config.prefix[8] = 2;
    config.n[1] = PRINCIPAL_HWM;
    config.n[3] = 21;
    config.n[5] = 80_000;
    config.n[CONFIG_LIABILITY] = EARNED;
    config.n[8] = COLLECTIVE_CARRY;
    config.n[9] = PRINCIPAL_HWM;
    config.n[13] = EARNED;
    // The literal independent Config parser leaves KIF timing and registry
    // fields in its 88-byte tail, ahead of the 64-byte migration area.
    for offset in [24, 32, 40, 48, 56] {
        put64(&mut config.tail, offset, STORAGE);
    }
    let mut reward = genesis_data[3].clone();
    assert_eq!(reward[51], 0, "unrestricted earned claim does not need an active-period marker");
    put64(&mut reward, REWARD_CLAIMABLE, EARNED);
    put64(&mut reward, REWARD_CLAIMABLE + 8, EARNED);
    let state_balance = |bytes: usize| (128 + bytes as u64) * 6_960;
    BTreeMap::from([
        (config_address, account(PROGRAM, state_balance(1014), config.bytes())),
        (target(1), account(PROGRAM, state_balance(891), genesis_data[1].clone())),
        (reward_address, account(PROGRAM, state_balance(84), reward)),
        (target(13), account(system_program::id(), STORAGE + EARNED + COLLECTIVE_CARRY, vec![])),
        (key(96), account(system_program::id(), 1_000_000_000, vec![])),
        (key(220), account(system_program::id(), 10_000_000, vec![])),
        (key(242), account(system_program::id(), 10_000_000, vec![])),
    ])
}

fn loader(genesis: &mut GenesisConfig, elf: &[u8]) {
    let address = program_data(PROGRAM);
    let program = bincode::serialize(&UpgradeableLoaderState::Program { programdata_address: address }).unwrap();
    let mut data = bincode::serialize(&UpgradeableLoaderState::ProgramData {
        slot: 0, upgrade_authority_address: None,
    }).unwrap();
    data.resize(45, 0);
    data.extend_from_slice(elf);
    for (key, bytes, executable) in [(PROGRAM, program, true), (address, data, false)] {
        let mut image = account(bpf_loader_upgradeable::id(), genesis.rent.minimum_balance(bytes.len()), bytes);
        image.set_executable(executable);
        genesis.accounts.insert(key, image.into());
    }
}

fn bank_configuration() -> BankTestConfig {
    let mut config = BankTestConfig::default();
    let database = &mut config.accounts_db_config;
    database.num_foreground_threads = NonZeroUsize::new(1);
    database.num_background_threads = NonZeroUsize::new(1);
    database.read_cache_limit_bytes = Some((2 * 1024 * 1024, 4 * 1024 * 1024));
    database.read_cache_num_shards = Some(16);
    database.read_cache_evict_sample_size = Some(4);
    let index = database.index.as_mut().unwrap();
    index.bins = Some(2);
    index.num_flush_threads = NonZeroUsize::new(1);
    index.num_initial_accounts = Some(128);
    config
}

#[test]
fn stored_basis_preserves_earned_kif_in_real_bank_rent_profiles() {
    let artifact = env::var("PIV1_RENT_ELF").expect("exact candidate artifact path");
    let expected_sha = env::var("PIV1_RENT_ELF_SHA256").expect("exact candidate artifact digest");
    let elf = fs::read(&artifact).unwrap();
    assert_eq!(&elf[..4], b"\x7fELF");
    assert_eq!(format!("{:x}", Sha256::digest(&elf)), expected_sha);
    let base = PathBuf::from(env::var_os("PIV1_RENT_BANK_DIR").expect("fresh evidence directory"));
    fs::create_dir(&base).unwrap();
    let config_address = target(0);
    let vault = target(13);
    let fee_payer = key(242);
    let mut state = fixture();
    let mut claimed = 0;
    let mut external_credit = 0;
    for rate in [5_080, 2_575, 1_322, 5_080, 6_960] {
        // Transfer exact post-transaction account states into a fresh Bank
        // snapshot with the new profile. Its actual collector and sysvar agree.
        // This does not simulate an epoch's feature-activation transaction.
        let mut genesis = GenesisConfig { creation_time: 100, ..GenesisConfig::default() };
        genesis.rent.lamports_per_byte = rate;
        genesis.fee_rate_governor.lamports_per_signature = 5_000;
        genesis.fee_rate_governor.target_lamports_per_signature = 5_000;
        genesis.fee_rate_governor.target_signatures_per_slot = 0;
        genesis.fee_rate_governor.min_lamports_per_signature = 5_000;
        genesis.fee_rate_governor.max_lamports_per_signature = 5_000;
        if rate == 1_322 {
            external_credit = 34_321;
            transfer_record(&mut state, vault, 0, external_credit);
        }
        for (address, account) in &state {
            genesis.accounts.insert(*address, account.clone().into());
        }
        loader(&mut genesis, &elf);
        let path = base.join(format!("rent-{rate}-after-{claimed}"));
        fs::create_dir(&path).unwrap();
        let leader = SlotLeader { id: key(246), vote_address: key(247) };
        let root = Bank::new_with_paths_for_tests(&genesis, Some(bank_configuration()), vec![path], Some(leader));
        let (root, forks) = root.wrap_with_bank_forks_for_tests();
        let bank = Bank::new_from_parent_with_bank_forks(&forks, root, leader, 1);
        let live_floor = genesis.rent.minimum_balance(0);
        assert_eq!(live_floor, 128 * rate);
        assert_eq!(bank.rent_collector().rent.minimum_balance(0), live_floor);
        assert_eq!(bank.get_account(&sysvar::rent::id()).unwrap().data(), bincode::serialize(&genesis.rent).unwrap());
        assert_eq!(state, get_accounts(&bank, state.keys().copied()), "carry real account state unchanged");

        let amount = 100_000;
        let sole_claim = owned_claim(amount, claimed);
        let mut expected = state.clone();
        if rate == 2_575 {
            // The first exact claim reaches native System CPI. A later stale
            // request rejects normally and the Bank must roll back both changes.
            let stale = owned_claim(amount + 1, claimed);
            let receipt = execute(&bank, &[sole_claim.clone(), stale]);
            assert_eq!(receipt.status, Err(TransactionError::InstructionError(1, InstructionError::Custom(STALE_CLAIM))), "{:?}", receipt.log_messages);
            let inner = receipt.inner_instructions.as_ref().unwrap();
            assert_eq!(inner[0].len(), 1);
            assert_eq!(inner[0][0].stack_height, 2);
            assert_eq!(inner[0][0].instruction.data, [2_u32.to_le_bytes().as_slice(), &amount.to_le_bytes()].concat());
            assert!(inner[1].is_empty());
            transfer_record(&mut expected, fee_payer, FEE, 0);
            assert_eq!(get_accounts(&bank, expected.keys().copied()), expected, "rollback preserves every non-fee fixture account");
            println!("PIV1_RENT_EVIDENCE {}", json!({"rentFloor":live_floor,"case":"outer-rollback-on-stale-claim","fee":FEE,"code":STALE_CLAIM}));
        }
        let receipt = execute(&bank, &[sole_claim]);
        if live_floor > STORAGE {
            assert!(state[&vault].lamports() >= live_floor, "raw backing alone must not conceal the missing technical funding");
            assert_eq!(receipt.status, Err(TransactionError::InstructionError(0, InstructionError::Custom(RUNTIME_RENT_DEFICIT))), "{:?}", receipt.log_messages);
            assert!(receipt.inner_instructions.as_ref().unwrap()[0].is_empty());
        } else {
            assert_eq!(receipt.status, Ok(()), "{:?}", receipt.log_messages);
            assert_eq!(receipt.inner_instructions.as_ref().unwrap()[0].len(), 1);
            expected_claim(&mut expected, amount, &mut claimed);
        }
        transfer_record(&mut expected, fee_payer, FEE, 0);
        state = get_accounts(&bank, expected.keys().copied());
        assert_eq!(state, expected, "exact program state and System payout or refusal");
        assert_eq!(state[&vault].lamports(), STORAGE + EARNED - claimed + COLLECTIVE_CARRY + external_credit);
        println!("PIV1_RENT_EVIDENCE {}", json!({"elf":expected_sha,"rentRate":rate,"rentFloor":live_floor,
            "storedTechnicalReserve":STORAGE,"status":format!("{:?}",receipt.status),"claimed":claimed,
            "vault":state[&vault].lamports(),"externalCredit":external_credit,"fee":FEE,"computeUnits":receipt.executed_units}));

        if live_floor > STORAGE {
            let gap = live_floor - STORAGE;
            let top_up = top_up_kif(gap);
            let funding = execute(&bank, &[top_up.clone()]);
            assert_eq!(funding.status, Ok(()), "{:?}", funding.log_messages);
            let inner = funding.inner_instructions.as_ref().unwrap();
            assert_eq!(inner[0].len(), 1);
            assert_eq!(inner[0][0].instruction.data, [2_u32.to_le_bytes().as_slice(), &gap.to_le_bytes()].concat());
            transfer_record(&mut state, fee_payer, FEE, 0);
            transfer_record(&mut state, key(220), gap, 0);
            transfer_record(&mut state, vault, 0, gap);
            let mut config = Config::read(&state[&config_address]);
            put64(&mut config.tail, 56, live_floor);
            state.get_mut(&config_address).unwrap().set_data_from_slice(&config.bytes());
            assert_eq!(get_accounts(&bank, state.keys().copied()), state, "only storage basis, sponsor and vault changed");

            let distinct_entry = ComputeBudgetInstruction::set_compute_unit_limit(200_001);
            let replay = execute(&bank, &[distinct_entry, top_up]);
            assert_eq!(replay.status, Err(TransactionError::InstructionError(1, InstructionError::Custom(INVALID_RENT))));
            assert!(replay.inner_instructions.as_ref().unwrap()[1].is_empty());
            transfer_record(&mut state, fee_payer, FEE, 0);
            assert_eq!(get_accounts(&bank, state.keys().copied()), state, "stale rent quote has only a fee effect");

            let resumed = execute(&bank, &[ComputeBudgetInstruction::set_compute_unit_limit(200_002), owned_claim(amount, claimed)]);
            assert_eq!(resumed.status, Ok(()), "{:?}", resumed.log_messages);
            assert_eq!(resumed.inner_instructions.as_ref().unwrap()[1].len(), 1);
            expected_claim(&mut state, amount, &mut claimed);
            transfer_record(&mut state, fee_payer, FEE, 0);
            assert_eq!(get_accounts(&bank, state.keys().copied()), state);
            assert_eq!(state[&vault].lamports(), live_floor + EARNED - claimed + COLLECTIVE_CARRY + external_credit);
            println!("PIV1_RENT_EVIDENCE {}", json!({"rentFloor":live_floor,"case":"sponsor-top-up-and-resumed-claim",
                "fundedGap":gap,"staleRepeatError":INVALID_RENT,"claimed":claimed,"vault":state[&vault].lamports(),
                "externalCredit":external_credit,"fees":3 * FEE,"topUpCu":funding.executed_units,"claimCu":resumed.executed_units}));
        }
    }
}
