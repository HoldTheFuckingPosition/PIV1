//! Dedicated synthetic complete invocation. Builders serialize independently
//! through Anchor/Borsh and the pinned host Instructions-sysvar constructor.
#[path = "kif_claim_custody.rs"]
mod claim_support;

use anchor_lang::{
    prelude::{borsh, AccountInfo, Clock, Pubkey, Rent, SolanaSysvar},
    solana_program::{bpf_loader_upgradeable, instruction::{AccountMeta, Instruction}, sysvar},
    AnchorSerialize,
};
use piv1::{
    accounts::{seeds, CONFIG_DISCRIMINATOR},
    guardian_clock_accounts::{GUARDIAN_REGISTRY_DISCRIMINATOR, GUARDIAN_REGISTRY_SEED},
    kif_claim_accounts::GUARDIAN_REWARD_DISCRIMINATOR,
    squads_accounts::{SQUADS_MULTISIG_DISCRIMINATOR, SQUADS_V4_PROGRAM_ID},
    squads_execution::*,
    state::{GuardianRegistry, GuardianReward, PivConfig},
};
pub use claim_support::{key, BackingAccount, PROGRAM};
use claim_support::{envelope, Fixture as ClaimFixture};

pub const PROGRAM_ACCOUNT: usize = 0;
pub const PROGRAM_DATA: usize = 1;
pub const MULTISIG: usize = 2;
pub const PROPOSAL: usize = 3;
pub const TRANSACTION: usize = 4;
pub const VAULT: usize = 5;
pub const INSTRUCTIONS: usize = 6;
pub const CONFIG: usize = 7;
pub const REGISTRY: usize = 8;
pub const CLOCK: usize = 15;

#[derive(Clone, Debug, Eq, PartialEq, AnchorSerialize)]
pub struct Member { pub key: Pubkey, pub permissions: u8 }
#[derive(Clone, Debug, Eq, PartialEq, AnchorSerialize)]
pub struct Multisig {
    pub create_key: Pubkey,
    pub config_authority: Pubkey,
    pub threshold: u16,
    pub time_lock: u32,
    pub transaction_index: u64,
    pub stale_transaction_index: u64,
    pub rent_collector: Option<Pubkey>,
    pub bump: u8,
    pub members: Vec<Member>,
}
#[derive(Clone, Debug, Eq, PartialEq, AnchorSerialize)]
pub struct Proposal {
    pub multisig: Pubkey, pub index: u64, pub status: u8, pub timestamp: i64,
    pub bump: u8, pub approved: Vec<Pubkey>, pub rejected: Vec<Pubkey>, pub cancelled: Vec<Pubkey>,
}
#[derive(Clone, Debug, Eq, PartialEq, AnchorSerialize)]
pub struct Inner { pub program_id_index: u8, pub accounts: Vec<u8>, pub data: Vec<u8> }
#[derive(Clone, Debug, Eq, PartialEq, AnchorSerialize)]
pub struct Lookup { pub account_key: Pubkey, pub writable: Vec<u8>, pub readonly: Vec<u8> }
#[derive(Clone, Debug, Eq, PartialEq, AnchorSerialize)]
pub struct Message {
    pub num_signers: u8, pub writable_signers: u8, pub writable_non_signers: u8,
    pub keys: Vec<Pubkey>, pub instructions: Vec<Inner>, pub lookups: Vec<Lookup>,
}
#[derive(Clone, Debug, Eq, PartialEq, AnchorSerialize)]
pub struct Transaction {
    pub multisig: Pubkey, pub creator: Pubkey, pub index: u64, pub bump: u8,
    pub vault_index: u8, pub vault_bump: u8, pub ephemeral: Vec<u8>, pub message: Message,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Fixture {
    pub program: Pubkey,
    pub accounts: Vec<BackingAccount>,
    pub inner_data: Vec<u8>,
    pub outer: Vec<Instruction>,
    pub current: u16,
    pub stack_height: usize,
    pub clock: Clock,
    pub rent: Rent,
    pub multisig: Multisig,
    pub proposal: Proposal,
    pub transaction: Transaction,
}

impl Fixture {
    pub fn new() -> Self {
        let (multisig_key, multisig_bump) = Pubkey::find_program_address(
            &[b"multisig", b"multisig", key(80).as_ref()], &SQUADS_V4_PROGRAM_ID);
        let index = 19_u64;
        let (transaction_key, transaction_bump) = Pubkey::find_program_address(
            &[b"multisig", multisig_key.as_ref(), b"transaction", &index.to_le_bytes()], &SQUADS_V4_PROGRAM_ID);
        let (proposal_key, proposal_bump) = Pubkey::find_program_address(
            &[b"multisig", multisig_key.as_ref(), b"transaction", &index.to_le_bytes(), b"proposal"], &SQUADS_V4_PROGRAM_ID);
        let (vault, vault_bump) = Pubkey::find_program_address(
            &[b"multisig", multisig_key.as_ref(), b"vault", &[7]], &SQUADS_V4_PROGRAM_ID);
        let (program_data, _) = Pubkey::find_program_address(&[PROGRAM.as_ref()], &bpf_loader_upgradeable::ID);
        let multisig = Multisig { create_key: key(80), config_authority: Pubkey::default(), threshold: 4,
            time_lock: 10, transaction_index: index, stale_transaction_index: 3, rent_collector: None,
            bump: multisig_bump, members: (91..97).map(|tag| Member { key: key(tag), permissions: 7 }).collect() };
        let proposal = Proposal { multisig: multisig_key, index, status: 3, timestamp: 90,
            bump: proposal_bump, approved: (91..95).map(key).collect(), rejected: vec![], cancelled: vec![] };
        let transaction = Transaction { multisig: multisig_key, creator: key(91), index, bump: transaction_bump,
            vault_index: 7, vault_bump, ephemeral: vec![], message: Message { num_signers: 0,
                writable_signers: 0, writable_non_signers: 0, keys: vec![], instructions: vec![], lookups: vec![] } };
        let account = |key, owner, executable, data| BackingAccount { key, owner, executable, signer: false,
            writable: false, lamports: 12345, data };
        let mut program_bytes = 2_u32.to_le_bytes().to_vec(); program_bytes.extend(program_data.to_bytes());
        let mut pd_bytes = 3_u32.to_le_bytes().to_vec(); pd_bytes.extend(4242_u64.to_le_bytes());
        pd_bytes.push(1); pd_bytes.extend(vault.to_bytes()); pd_bytes.extend([0xA5; 23]);
        let mut accounts = vec![
            account(PROGRAM, bpf_loader_upgradeable::ID, true, program_bytes),
            account(program_data, bpf_loader_upgradeable::ID, false, pd_bytes),
            account(multisig_key, SQUADS_V4_PROGRAM_ID, false, vec![]),
            account(proposal_key, SQUADS_V4_PROGRAM_ID, false, vec![]),
            account(transaction_key, SQUADS_V4_PROGRAM_ID, false, vec![]),
            account(vault, anchor_lang::solana_program::system_program::ID, false, vec![]),
            account(sysvar::instructions::ID, sysvar::ID, false, vec![]),
        ];
        accounts[VAULT].signer = true;
        let rent = Rent::default();
        let mut config = ClaimFixture::new(0).config();
        let (config_key, bump) = Pubkey::find_program_address(&[seeds::CONFIG], &PROGRAM);
        config.bumps.config = bump;
        macro_rules! bind {
            ($field:ident, $seed:expr) => {{
                let (address, bump) = Pubkey::find_program_address(&[$seed], &PROGRAM);
                config.$field = address; config.bumps.$field = bump;
            }};
        }
        bind!(piv_authority, seeds::AUTHORITY); bind!(active_distribution, seeds::DISTRIBUTION);
        bind!(principal_jito_vault, seeds::PRINCIPAL_JITO); bind!(pending_jito_vault, seeds::PENDING_JITO);
        bind!(pending_sol_vault, seeds::PENDING_SOL); bind!(principal_sol_queue, seeds::PRINCIPAL_SOL);
        bind!(operational_sol_vault, seeds::OPERATIONAL_SOL); bind!(distribution_escrow, seeds::DISTRIBUTION_ESCROW);
        bind!(kif_sol_vault, seeds::KIF_SOL); bind!(guardian_registry, GUARDIAN_REGISTRY_SEED);
        // Deliberately reversed PIV1 slots: authorization must preserve their meaning.
        let registry = GuardianRegistry::new(config.bumps.guardian_registry, config.guardian_registry_revision,
            core::array::from_fn(|i| key(96 - i as u8))).unwrap();
        let state = |key, data: Vec<u8>| BackingAccount { key, owner: PROGRAM, executable: false,
            signer: false, writable: false, lamports: rent.minimum_balance(data.len()), data };
        accounts.push(state(config_key, envelope(&config, CONFIG_DISCRIMINATOR, PivConfig::SPACE)));
        accounts[CONFIG].writable = true;
        accounts.push(state(config.guardian_registry, envelope(&registry, GUARDIAN_REGISTRY_DISCRIMINATOR, GuardianRegistry::SPACE)));
        for slot in 0..6 {
            let mut reward = GuardianReward::new(0, &registry, slot).unwrap();
            let (address, bump) = Pubkey::find_program_address(&[b"guardian-reward", reward.guardian.as_ref(),
                &reward.registry_revision.to_le_bytes(), &[slot]], &PROGRAM);
            reward.bump = bump;
            accounts.push(state(address, envelope(&reward, GUARDIAN_REWARD_DISCRIMINATOR, GuardianReward::SPACE)));
        }
        accounts.push(account(sysvar::clock::ID, sysvar::ID, false, vec![0; 40]));
        let clock = Clock { unix_timestamp: 100, ..Clock::default() };
        clock.to_account_info(&mut accounts[CLOCK].info()).unwrap();
        let mut fixture = Self { program: PROGRAM, accounts, inner_data: vec![23, 7, 4, 6], outer: vec![],
            current: 0, stack_height: 2, clock, rent, multisig, proposal, transaction };
        fixture.sync_multisig(); fixture.sync_proposal(); fixture.rebuild_message();
        fixture
    }
    pub fn roles(&self) -> SquadsExecutionRoles {
        SquadsExecutionRoles { program: PROGRAM_ACCOUNT, program_data: PROGRAM_DATA, multisig: MULTISIG,
            proposal: PROPOSAL, transaction: TRANSACTION, vault: VAULT,
            instructions: INSTRUCTIONS, config: CONFIG, guardian_registry: REGISTRY, rewards: [9, 10, 11, 12, 13, 14], clock: CLOCK }
    }
    pub fn sync_multisig(&mut self) { self.accounts[MULTISIG].data = envelope(&self.multisig, SQUADS_MULTISIG_DISCRIMINATOR, 330); }
    pub fn sync_proposal(&mut self) { self.accounts[PROPOSAL].data = envelope(&self.proposal, PROPOSAL_DISCRIMINATOR, 646); }
    pub fn sync_transaction(&mut self) {
        let encoded = self.transaction.try_to_vec().unwrap();
        self.accounts[TRANSACTION].data = [VAULT_TRANSACTION_DISCRIMINATOR.as_slice(), encoded.as_slice()].concat();
    }
    pub fn rebuild_message(&mut self) {
        let mut sorted: Vec<_> = self.accounts.iter().collect();
        sorted.sort_by_key(|account| match (account.signer, account.writable) { (true,true) => 0, (true,false) => 1, (false,true) => 2, _ => 3 });
        let keys: Vec<_> = sorted.iter().map(|account| account.key).collect();
        let indices = self.accounts.iter().map(|account| keys.iter().position(|key| *key == account.key).unwrap() as u8).collect();
        self.transaction.message = Message {
            num_signers: sorted.iter().filter(|account| account.signer).count() as u8,
            writable_signers: sorted.iter().filter(|account| account.signer && account.writable).count() as u8,
            writable_non_signers: sorted.iter().filter(|account| !account.signer && account.writable).count() as u8,
            instructions: vec![Inner { program_id_index: keys.iter().position(|key| *key == self.program).unwrap() as u8,
                accounts: indices, data: self.inner_data.clone() }], keys, lookups: vec![],
        };
        self.sync_transaction(); self.rebuild_outer();
    }
    pub fn rebuild_outer(&mut self) {
        let mut metas = vec![AccountMeta::new_readonly(self.accounts[MULTISIG].key, false),
            AccountMeta::new(self.accounts[PROPOSAL].key, false),
            AccountMeta::new_readonly(self.accounts[TRANSACTION].key, false),
            AccountMeta::new_readonly(key(91), true)];
        metas.extend(self.transaction.message.keys.iter().map(|key| {
            let account = self.accounts.iter().find(|account| account.key == *key).unwrap();
            // Outer message privilege union includes the fixed writable proposal.
            AccountMeta { pubkey: *key, is_signer: account.signer && *key != self.accounts[VAULT].key,
                is_writable: account.writable || *key == self.accounts[PROPOSAL].key }
        }));
        self.outer = vec![Instruction { program_id: SQUADS_V4_PROGRAM_ID, accounts: metas,
            data: VAULT_TRANSACTION_EXECUTE_DISCRIMINATOR.to_vec() }];
        self.current = 0; self.sync_sysvar();
    }
    pub fn sync_sysvar(&mut self) {
        use sysvar::instructions::{BorrowedAccountMeta, BorrowedInstruction, construct_instructions_data};
        let borrowed: Vec<_> = self.outer.iter().map(|ix| BorrowedInstruction { program_id: &ix.program_id,
            accounts: ix.accounts.iter().map(|meta| BorrowedAccountMeta { pubkey: &meta.pubkey,
                is_signer: meta.is_signer, is_writable: meta.is_writable }).collect(), data: &ix.data }).collect();
        let mut bytes = construct_instructions_data(&borrowed);
        let end = bytes.len(); bytes[end - 2..].copy_from_slice(&self.current.to_le_bytes());
        self.accounts[INSTRUCTIONS].data = bytes;
    }
    pub fn with_infos<T>(&mut self, action: impl FnOnce(&[AccountInfo<'_>]) -> T) -> T {
        let infos: Vec<_> = self.accounts.iter_mut().map(BackingAccount::info).collect(); action(&infos)
    }
    pub fn run(&mut self) -> SquadsExecutionResult<AuthenticatedSquadsInvocation> {
        let program = self.program; let data = self.inner_data.clone(); let roles = self.roles();
        let vault_index = self.transaction.vault_index;
        let context = ModeledSquadsInvocationContext { stack_height: self.stack_height, clock: self.clock.clone(), rent: self.rent.clone() };
        self.with_infos(|accounts| authenticate_squads_invocation_with_host_context(&program, accounts, &data, roles, vault_index, context))
    }
    pub fn success(&mut self) -> AuthenticatedSquadsInvocation {
        let before = self.clone(); let value = self.run().unwrap(); assert_eq!(*self, before, "complete input immutability"); value
    }
    pub fn reject(&mut self, expected: SquadsExecutionError) {
        let before = self.clone(); assert_eq!(self.run(), Err(expected)); assert_eq!(*self, before, "failure preserves every input");
    }
    pub fn reject_any(&mut self) {
        let before = self.clone(); assert!(self.run().is_err()); assert_eq!(*self, before, "malformed input is preserved");
    }
}
