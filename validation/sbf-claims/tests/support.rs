//! Isolated synthetic runtime fixtures. No wallet, signature, earning handler or
//! Bank is instantiated. Only the hash-bound ELF performs claims.

#[path = "../../../programs/piv1/tests/support/kif_claim_custody.rs"]
pub mod original_fixture;

use std::{cell::RefCell, collections::BTreeSet, rc::Rc};
use anchor_lang::{AnchorDeserialize, Event};
use mollusk_svm::{InvocationInspectCallback, Mollusk, program::{self, ProgramCache}};
use piv1::{events::KifClaimed, instructions::claim_kif::encode_claim_kif,
    state::{GuardianReward, KifClaimRequest, PivConfig}};
use sha2::{Digest, Sha256};
use solana_account::{Account, ReadableAccount};
use solana_instruction::{AccountMeta, Instruction};
use solana_program_runtime::invoke_context::InvokeContext;
use solana_pubkey::Pubkey;
use solana_rent::Rent;
use solana_svm_log_collector::LogCollector;
use solana_transaction_context::instruction_accounts::InstructionAccount;

pub use original_fixture::{CONFIG, REWARD, KIF, GUARDIAN};
pub const SYSTEM: usize = 4;
pub const PAYER: usize = 5;
pub const SENTINEL: usize = 7;
// The reviewed runner supplies the complete pin-bound artifact identity.
// These are harness environment inputs, never Solana instruction arguments.
fn artifact_identity() -> (String, String, usize) {
    (std::env::var("PIV_VALIDATION_ELF_PATH").expect("reviewed artifact path"),
     std::env::var("PIV_VALIDATION_ELF_SHA256").expect("reviewed artifact hash"),
     std::env::var("PIV_VALIDATION_ELF_BYTES").expect("reviewed artifact size").parse().unwrap())
}
pub type Accounts = Vec<(Pubkey, Account)>;

pub fn new_key(tag: u8) -> Pubkey { Pubkey::new_from_array([tag; 32]) }
pub fn program_id() -> Pubkey { Pubkey::new_from_array(original_fixture::PROGRAM.to_bytes()) }

pub fn rent_bridge(rent: &Rent) -> anchor_lang::prelude::Rent {
    assert_eq!(std::mem::size_of::<Rent>(), 24);
    assert_eq!(std::mem::align_of::<Rent>(), 8);
    assert_eq!(std::mem::size_of::<anchor_lang::prelude::Rent>(), 24);
    assert_eq!(std::mem::align_of::<anchor_lang::prelude::Rent>(), 8);
    let bytes = bincode::serialize(rent).unwrap();
    assert_eq!(bytes.len(), 17);
    let old: anchor_lang::prelude::Rent = bincode::deserialize(&bytes).unwrap();
    assert_eq!(bincode::serialize(&old).unwrap(), bytes);
    for length in [0, GuardianReward::SPACE, PivConfig::SPACE] {
        assert_eq!(rent.minimum_balance(length), old.minimum_balance(length));
        assert_eq!(rent.minimum_balance(length), anchor_lang::prelude::Rent::default().minimum_balance(length));
    }
    old
}

pub fn config(accounts: &Accounts) -> PivConfig {
    PivConfig::deserialize(&mut &accounts[CONFIG].1.data[8..]).unwrap()
}
pub fn reward(accounts: &Accounts) -> GuardianReward {
    GuardianReward::deserialize(&mut &accounts[REWARD].1.data[8..]).unwrap()
}
pub fn edit_config(accounts: &mut Accounts, change: impl FnOnce(&mut PivConfig)) {
    let mut value = config(accounts); change(&mut value);
    accounts[CONFIG].1.data = original_fixture::envelope(&value,
        piv1::accounts::CONFIG_DISCRIMINATOR, PivConfig::SPACE);
}
pub fn edit_reward(accounts: &mut Accounts, change: impl FnOnce(&mut GuardianReward)) {
    let mut value = reward(accounts); change(&mut value);
    accounts[REWARD].1.data = original_fixture::envelope(&value,
        piv1::kif_claim_accounts::GUARDIAN_REWARD_DISCRIMINATOR, GuardianReward::SPACE);
}

/// A fresh runtime baseline, created once after all initial fixture funding.
/// No later credit or successful claim resets these original balances/counters.
#[derive(Clone, Debug, PartialEq)]
pub struct Audit { initial: Accounts, pub paid: u128 }

#[derive(Clone, Debug, PartialEq)]
pub struct Fixture { pub accounts: Accounts, pub audit: Audit }

impl Fixture {
    pub fn new(rent: &Rent, excess: u64, paused: bool) -> Self {
        let trusted_old_rent = rent_bridge(rent);
        let imported = original_fixture::Fixture::new(excess);
        // The imported object's original host audit is never modified or reset.
        imported.validate_audit().unwrap();
        let mut accounts: Accounts = imported.accounts.iter().map(|a| (
            Pubkey::new_from_array(a.key.to_bytes()), Account {
                lamports: a.lamports, data: a.data.clone(),
                owner: Pubkey::new_from_array(a.owner.to_bytes()),
                executable: a.executable, rent_epoch: 0,
            })).collect();
        for a in &accounts[..3] {
            assert!(a.1.lamports >= trusted_old_rent.minimum_balance(a.1.data.len()));
        }
        // Explicit synthetic initial funding belongs to this new runtime baseline.
        accounts[GUARDIAN].1.lamports = rent.minimum_balance(0).checked_add(11).unwrap();
        edit_config(&mut accounts, |c| c.paused = paused);
        accounts.push(program::keyed_account_for_system_program());
        accounts.push((new_key(231), Account { lamports: rent.minimum_balance(0) + 1_000_000,
            owner: accounts[SYSTEM].0, ..Account::default() }));
        accounts.push((program_id(), program::create_program_account_loader_v3(&program_id())));
        accounts.push((new_key(232), Account { lamports: 73, data: vec![9, 7, 5],
            owner: new_key(233), executable: false, rent_epoch: 17 }));
        assert_eq!(accounts.iter().map(|(k, _)| *k).collect::<BTreeSet<_>>().len(), accounts.len());
        let value = Self { audit: Audit { initial: accounts.clone(), paid: 0 }, accounts };
        value.validate_audit(rent);
        value
    }

    pub fn instruction(&self, amount: u64, counter: u64) -> Instruction {
        Instruction { program_id: program_id(), accounts: vec![
            AccountMeta::new(self.accounts[CONFIG].0, false),
            AccountMeta::new(self.accounts[REWARD].0, false),
            AccountMeta::new(self.accounts[KIF].0, false),
            AccountMeta::new(self.accounts[GUARDIAN].0, true),
            AccountMeta::new_readonly(self.accounts[SYSTEM].0, false),
        ], data: encode_claim_kif(KifClaimRequest { amount_lamports: amount,
            expected_cumulative_claimed: counter }).to_vec() }
    }

    pub fn expected_payment(&self, amount: u64) -> Accounts {
        let mut next = self.accounts.clone();
        // Independent arithmetic, not a call to prepare/commit/host claim.
        edit_config(&mut next, |c| {
            c.kif_claim_liability_lamports = c.kif_claim_liability_lamports.checked_sub(amount).unwrap();
            c.cumulative_kif_claimed_lamports = c.cumulative_kif_claimed_lamports.checked_add(amount).unwrap();
        });
        edit_reward(&mut next, |r| {
            r.claimable_lamports = r.claimable_lamports.checked_sub(amount).unwrap();
            r.cumulative_claimed = r.cumulative_claimed.checked_add(amount).unwrap();
        });
        next[KIF].1.lamports = next[KIF].1.lamports.checked_sub(amount).unwrap();
        next[GUARDIAN].1.lamports = next[GUARDIAN].1.lamports.checked_add(amount).unwrap();
        next
    }

    pub fn accept_payment(&mut self, actual: Accounts, amount: u64, rent: &Rent) {
        assert_eq!(actual, self.expected_payment(amount));
        let observed_debit = self.accounts[KIF].1.lamports.checked_sub(actual[KIF].1.lamports).unwrap();
        let observed_credit = actual[GUARDIAN].1.lamports.checked_sub(self.accounts[GUARDIAN].1.lamports).unwrap();
        assert_eq!(observed_debit, observed_credit);
        assert_eq!(observed_debit, amount);
        self.audit.paid = self.audit.paid.checked_add(u128::from(observed_debit)).unwrap();
        self.accounts = actual;
        self.validate_audit(rent);
    }

    pub fn validate_audit(&self, rent: &Rent) {
        let a = &self.audit; let before_c = config(&a.initial); let c = config(&self.accounts);
        let before_r = reward(&a.initial); let r = reward(&self.accounts);
        let total = |accounts: &Accounts| accounts.iter().map(|a| u128::from(a.1.lamports)).sum::<u128>();
        assert_eq!(total(&a.initial), total(&self.accounts));
        assert_eq!(u128::from(a.initial[KIF].1.lamports), u128::from(self.accounts[KIF].1.lamports) + a.paid);
        assert_eq!(u128::from(a.initial[GUARDIAN].1.lamports) + a.paid, u128::from(self.accounts[GUARDIAN].1.lamports));
        assert_eq!(u128::from(before_c.kif_claim_liability_lamports), u128::from(c.kif_claim_liability_lamports) + a.paid);
        assert_eq!(u128::from(before_c.cumulative_kif_claimed_lamports) + a.paid, u128::from(c.cumulative_kif_claimed_lamports));
        assert_eq!(u128::from(before_r.claimable_lamports), u128::from(r.claimable_lamports) + a.paid);
        assert_eq!(u128::from(before_r.cumulative_claimed) + a.paid, u128::from(r.cumulative_claimed));
        assert_eq!(before_c.cumulative_kif_credited_lamports, c.cumulative_kif_credited_lamports);
        assert_eq!(before_r.cumulative_earned, r.cumulative_earned);
        assert_eq!(before_c.collective_kif_carry_lamports, c.collective_kif_carry_lamports);
        let excess = |accounts: &Accounts, c: &PivConfig| accounts[KIF].1.lamports
            .checked_sub(rent.minimum_balance(0)).unwrap()
            .checked_sub(c.kif_claim_liability_lamports).unwrap()
            .checked_sub(c.collective_kif_carry_lamports).unwrap();
        assert_eq!(excess(&a.initial, &before_c), excess(&self.accounts, &c));
        for i in [CONFIG, REWARD] { assert_eq!(a.initial[i].1.lamports, self.accounts[i].1.lamports); }
        for i in SYSTEM..self.accounts.len() { assert_eq!(a.initial[i], self.accounts[i]); }
    }
}

#[derive(Clone, Debug)]
pub struct Observation { pub accounts: Accounts, pub logs: Vec<String> }
#[derive(Default, Debug)]
pub struct Observations { pub privileges: Vec<Vec<(Pubkey, bool, bool)>>, pub after: Vec<Observation> }
struct Observer(Rc<RefCell<Observations>>);

impl InvocationInspectCallback for Observer {
    fn before_invocation(&self, _: &Mollusk, _: &Pubkey, _: &[u8], accounts: &[InstructionAccount],
        context: &mut InvokeContext, tracing: bool) {
        assert!(!tracing);
        self.0.borrow_mut().privileges.push(accounts.iter().map(|a| (
            *context.transaction_context.get_key_of_account_at_index(a.index_in_transaction).unwrap(),
            a.is_signer(), a.is_writable())).collect());
        // The API lends mutable context, but no context mutation is performed.
    }
    fn after_invocation(&self, mollusk: &Mollusk, context: &InvokeContext, tracing: bool) {
        assert!(!tracing);
        let tx = &context.transaction_context;
        let accounts = (0..tx.get_number_of_accounts()).map(|i| {
            let key = *tx.get_key_of_account_at_index(i).unwrap();
            let account = tx.accounts().try_borrow(i).unwrap();
            (key, Account { lamports: account.lamports(), data: account.data().to_vec(),
                owner: *account.owner(), executable: account.executable(), rent_epoch: account.rent_epoch() })
        }).collect();
        let logs = mollusk.logger.as_ref().unwrap().borrow().get_recorded_content().to_vec();
        self.0.borrow_mut().after.push(Observation { accounts, logs });
    }
}

// Account::Debug truncates data at 64 bytes. This separate evidence format
// records complete actual account bytes, including failed pre-discard contexts.
pub fn account_evidence(label: &str, stage: &str, ordinal: Option<usize>, index: usize,
    key: &Pubkey, account: &Account) -> String {
    use std::fmt::Write;
    let mut line = format!("ACCOUNT case={label} stage={stage} invocation_ordinal={ordinal:?} index={index} key={key} lamports={} owner={} executable={} rent_epoch={} data_len={} data_hex=",
        account.lamports, account.owner, account.executable, account.rent_epoch, account.data.len());
    for byte in &account.data { write!(&mut line, "{byte:02x}").unwrap(); }
    line
}

fn log_accounts(label: &str, stage: &str, ordinal: Option<usize>, accounts: &Accounts) {
    println!("ACCOUNTS case={label} stage={stage} invocation_ordinal={ordinal:?} count={}", accounts.len());
    for (index, (key, account)) in accounts.iter().enumerate() {
        println!("{}", account_evidence(label, stage, ordinal, index, key, account));
    }
}

pub struct Runtime { pub vm: Mollusk, pub observations: Rc<RefCell<Observations>> }
impl Runtime {
    pub fn new(limit: u64) -> Self {
        let mut vm = Mollusk::default();
        assert_eq!(vm.compute_budget, solana_compute_budget::compute_budget::ComputeBudget::new_with_defaults(true));
        assert_eq!(vm.compute_budget.compute_unit_limit, 1_400_000);
        assert_eq!(vm.compute_budget.heap_size, 32768);
        assert_eq!(vm.compute_budget.stack_frame_size, 4096);
        assert_eq!(vm.compute_budget.max_call_depth, 64);
        assert_eq!(vm.compute_budget.max_instruction_stack_depth, 9);
        assert_eq!(vm.compute_budget.max_instruction_trace_length, 64);
        assert!(vm.feature_set.disable_sbpf_v0_execution);
        assert!(vm.feature_set.reenable_sbpf_v0_execution);
        // SVMFeatureSet 4.2.0 has no Debug/PartialEq implementation. Record every
        // public field in its pinned source and verify the all_enabled defaults.
        macro_rules! feature_values {
            ($($field:ident),+ $(,)?) => {
                [$( (stringify!($field), vm.feature_set.$field), )+]
            };
        }
        let features = feature_values!(
            move_precompile_verification_to_svm,
            syscall_parameter_address_restrictions,
            virtual_address_space_adjustments,
            account_data_direct_mapping,
            enable_bpf_loader_set_authority_checked_ix,
            deplete_cu_meter_on_vm_failure,
            abort_on_invalid_curve,
            blake3_syscall_enabled,
            curve25519_syscall_enabled,
            disable_fees_sysvar,
            disable_sbpf_v0_execution,
            disable_sbpf_v0_v1_v2_deployment,
            enable_alt_bn128_compression_syscall,
            enable_alt_bn128_syscall,
            enable_big_mod_exp_syscall,
            enable_get_epoch_stake_syscall,
            enable_poseidon_syscall,
            enable_sbpf_v1_deployment_and_execution,
            enable_sbpf_v2_deployment_and_execution,
            enable_sbpf_v3_deployment_and_execution,
            get_sysvar_syscall_enabled,
            last_restart_slot_sysvar,
            reenable_sbpf_v0_execution,
            remaining_compute_units_syscall_enabled,
            remove_bpf_loader_incorrect_program_id,
            move_stake_and_move_lamports_ixs,
            deprecate_legacy_vote_ixs,
            simplify_alt_bn128_syscall_error_codes,
            fix_alt_bn128_multiplication_input_length,
            increase_tx_account_lock_limit,
            formalize_loaded_transaction_data_size,
            disable_zk_elgamal_proof_program,
            reenable_zk_elgamal_proof_program,
            delay_commission_updates,
            raise_cpi_nesting_limit_to_8,
            increase_cpi_account_info_limit,
            poseidon_enforce_padding,
            fix_alt_bn128_pairing_length_check,
            alt_bn128_little_endian,
            create_account_allow_prefund,
            bls_pubkey_management_in_vote_account,
            enable_alt_bn128_g2_syscalls,
            commission_rate_in_basis_points,
            custom_commission_collector,
            enable_bls12_381_syscall,
            block_revenue_sharing,
            vote_account_initialize_v2,
            direct_account_pointers_in_program_input,
            loader_v3_minimum_extend_program_size,
            enable_sha512_syscall,
            relax_post_exec_min_balance_check,
            define_ltds_fee_only_semantics,
        );
        assert_eq!(features.len(), 52);
        assert!(features.iter().all(|(_, enabled)| *enabled));
        vm.compute_budget.compute_unit_limit = limit;
        vm.program_cache = ProgramCache::new(&vm.feature_set, &vm.compute_budget, false);
        vm.logger = Some(LogCollector::new_ref_with_limit(None));
        let observations = Rc::new(RefCell::new(Observations::default()));
        vm.invocation_inspect_callback = Box::new(Observer(observations.clone()));
        let (elf_path, elf_sha256, elf_bytes) = artifact_identity();
        let elf = std::fs::read(&elf_path).unwrap();
        assert_eq!(elf.len(), elf_bytes);
        assert_eq!(format!("{:x}", Sha256::digest(&elf)), elf_sha256);
        vm.add_program_with_loader_and_elf(&program_id(), &program::loader_keys::LOADER_V3, &elf);
        assert_eq!(vm.program_cache.get_program_elf_bytes(&program_id()).unwrap(), elf);
        let entry = vm.program_cache.load_program(&program_id()).unwrap();
        let solana_program_runtime::program_cache_entry::ProgramCacheEntryType::Loaded(executable) = &entry.program
            else { panic!("PIV1 must be a loaded ELF, never a native builtin"); };
        let vm_config = executable.get_config();
        assert_eq!(format!("{:?}", executable.get_sbpf_version()), "V0");
        assert_eq!(format!("{:?}", vm_config.enabled_sbpf_versions), "V0..=V3");
        assert_eq!(vm_config.stack_frame_size, 4096);
        assert_eq!(vm_config.max_call_depth, 64);
        assert!(vm_config.enable_instruction_meter && vm_config.enable_address_translation);
        assert!(!vm_config.enable_register_tracing && !vm_config.reject_broken_elfs);
        println!("LOADED_VM_CONFIG {vm_config:?}");
        println!("CONFIG budget={:?} features={:?} rent={:?} ELF={elf_sha256} loader={}",
            vm.compute_budget, features, vm.sysvars.rent, program::loader_keys::LOADER_V3);
        Self { vm, observations }
    }

    pub fn execute(&self, label: &str, fixture: &Fixture, instructions: &[Instruction])
        -> mollusk_svm::result::types::TransactionResult {
        assert!(self.observations.borrow().after.is_empty());
        assert_eq!(fixture.accounts.iter().map(|(key, _)| *key).collect::<BTreeSet<_>>().len(), fixture.accounts.len());
        log_accounts(label, "supplied", None, &fixture.accounts);
        let result = self.vm.process_transaction_instructions(instructions, &fixture.accounts,
            Some(&fixture.accounts[PAYER].0));
        log_accounts(label, "returned", None, &result.resulting_accounts);
        for (ordinal, observation) in self.observations.borrow().after.iter().enumerate() {
            log_accounts(label, "raw-after-invocation", Some(ordinal), &observation.accounts);
        }
        // Durable raw evidence includes failed context before Mollusk discards it.
        println!("CASE {label} RESULT {result:#?} RAW {:#?}", self.observations.borrow());
        let message = result.message.as_ref().unwrap();
        assert_eq!(message.account_keys()[0], fixture.accounts[PAYER].0);
        assert!(message.is_signer(0));
        assert!(message.is_writable(0));
        let observations = self.observations.borrow();
        for (ordinal, actual) in observations.privileges.iter().enumerate() {
            let instruction = &instructions[ordinal];
            assert_eq!(actual.len(), instruction.accounts.len());
            for (meta, (key, signer, writable)) in instruction.accounts.iter().zip(actual) {
                assert_eq!(*key, meta.pubkey);
                let union_signer = instructions.iter().flat_map(|ix| &ix.accounts)
                    .any(|m| m.pubkey == *key && m.is_signer);
                let union_writable = instructions.iter().flat_map(|ix| &ix.accounts)
                    .any(|m| m.pubkey == *key && m.is_writable);
                assert_eq!(*signer, union_signer);
                assert_eq!(*writable, union_writable);
                assert_ne!(*key, fixture.accounts[PAYER].0);
            }
        }
        result
    }
}

pub fn assert_raw_matches(observation: &Observation, expected: &Accounts) {
    for (key, actual) in &observation.accounts {
        let expected = &expected.iter().find(|(k, _)| k == key).expect("unexpected implicit account").1;
        assert_eq!(actual, expected, "raw account {key}");
    }
    for i in [CONFIG, REWARD, KIF, GUARDIAN, SYSTEM, PAYER, 6] {
        assert!(observation.accounts.iter().any(|(key, _)| *key == expected[i].0));
    }
    // An unreferenced supplied sentinel is not a loaded transaction account;
    // it is checked in the complete returned vector, not invented in raw data.
}

fn base64_decode(text: &str) -> Vec<u8> {
    const ALPHABET: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    assert_eq!(text.len() % 4, 0);
    let mut out = Vec::new();
    for group in text.as_bytes().chunks_exact(4) {
        let value = |b| if b == b'=' { 0 } else { ALPHABET.iter().position(|x| *x == b).unwrap() as u32 };
        let n = (value(group[0]) << 18) | (value(group[1]) << 12) | (value(group[2]) << 6) | value(group[3]);
        out.push((n >> 16) as u8);
        if group[2] != b'=' { out.push((n >> 8) as u8); }
        if group[3] != b'=' { out.push(n as u8); }
    }
    out
}

pub fn event_bytes(logs: &[String]) -> Vec<Vec<u8>> {
    logs.iter().filter_map(|log| log.strip_prefix("Program data: ")).map(base64_decode).collect()
}
pub fn assert_one_event(logs: &[String], fixture: &Fixture, amount: u64) {
    let bytes = event_bytes(logs);
    let expected = KifClaimed { guardian_reward: anchor_lang::prelude::Pubkey::new_from_array(fixture.accounts[REWARD].0.to_bytes()),
        guardian: anchor_lang::prelude::Pubkey::new_from_array(fixture.accounts[GUARDIAN].0.to_bytes()),
        amount_lamports: amount };
    assert_eq!(bytes, vec![expected.data()]);
    assert_eq!(bytes[0].len(), 80);
    assert_eq!(&bytes[0][..8], &[0x04, 0xc9, 0xab, 0xfb, 0xd7, 0x77, 0x11, 0xd0]);
    assert_eq!(&bytes[0][8..40], fixture.accounts[REWARD].0.as_ref());
    assert_eq!(&bytes[0][40..72], fixture.accounts[GUARDIAN].0.as_ref());
    assert_eq!(u64::from_le_bytes(bytes[0][72..80].try_into().unwrap()), amount);
}

pub fn assert_one_transfer(result: &mollusk_svm::result::types::TransactionResult, fixture: &Fixture, amount: u64) {
    let all: Vec<_> = result.inner_instructions.iter().flatten().collect();
    assert_eq!(all.len(), 1);
    let inner = all[0]; let message = result.message.as_ref().unwrap();
    let keys = message.account_keys();
    assert_eq!(keys[inner.instruction.program_id_index as usize], fixture.accounts[SYSTEM].0);
    assert_eq!(inner.stack_height, Some(2));
    assert_eq!(inner.instruction.accounts.iter().map(|i| keys[*i as usize]).collect::<Vec<_>>(),
        vec![fixture.accounts[KIF].0, fixture.accounts[GUARDIAN].0]);
    let mut expected = 2u32.to_le_bytes().to_vec(); expected.extend(amount.to_le_bytes());
    assert_eq!(inner.instruction.data, expected);
    let source_index = keys.iter().position(|k| *k == fixture.accounts[KIF].0).unwrap();
    assert!(!message.is_signer(source_index));
}
