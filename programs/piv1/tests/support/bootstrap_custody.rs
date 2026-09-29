//! Host-only fixed custody plus independently serialized pinned protocol state.
use anchor_lang::{prelude::{Clock, Pubkey}, solana_program::{bpf_loader_upgradeable,
    program_option::COption, program_pack::Pack}};
use piv1::{accounts::STAKE_PROGRAM_ID, integrations::jito_identity::*};
use solana_stake_interface::state::{Authorized, Lockup, Meta, StakeStateV2};
use spl_token::state::{Account as TokenAccount, AccountState, Mint};
use super::{custody::{Fixture as Custody, *}, oracle,
    support::{kif_claim_custody::{BackingAccount, key}, vault_custody_model::World}};

pub const PROTOCOL: usize = 13;
pub const POOL: usize = 14;
pub const LIST: usize = 15;
pub const RESERVE: usize = 16;
pub const MANAGER: usize = 17;
pub const REFERRER: usize = 18;

#[derive(Clone, Debug, PartialEq)]
pub struct Fixture { pub custody: Custody, pub pool: oracle::StakePool, pub mint: Mint, pub clock: Clock }
impl Fixture {
    pub fn new(sol: u64, tokens: u64, shared: bool) -> Self {
        let mut w = World::empty(3, 71);
        if sol != 0 { w.explicit_sol(sol, sol).unwrap(); }
        if tokens != 0 { w.explicit_tokens(tokens, tokens).unwrap(); }
        Self::from_world(&w, shared)
    }
    pub fn from_world(w: &World, shared: bool) -> Self {
        let mut custody = Custody::from_world(w);
        custody.edit_config(|c| {
            c.stake_pool_program = JITO_STAKE_POOL_PROGRAM; c.stake_pool = JITO_STAKE_POOL;
            c.validator_list = key(231); c.reserve_stake = key(232); c.jitosol_mint = JITOSOL_MINT;
            c.manager_fee_account = key(233); c.referrer_token_account = key(if shared { 233 } else { 234 });
        });
        for index in [PRINCIPAL_JITO, PENDING_JITO] {
            custody.accounts[index].data[..32].copy_from_slice(JITOSOL_MINT.as_ref());
        }
        let c = custody.config();
        if c.accounted_pending_sol_lamports != 0 {
            custody.accounts[PENDING_SOL].writable = true; custody.accounts[PRINCIPAL_SOL].writable = true;
        }
        if c.accounted_pending_jitosol_units != 0 {
            custody.accounts[PENDING_JITO].writable = true; custody.accounts[PRINCIPAL_JITO].writable = true;
        }
        let (withdraw, bump) = Pubkey::find_program_address(&[JITO_STAKE_POOL.as_ref(), b"withdraw"], &JITO_STAKE_POOL_PROGRAM);
        let mut pool = oracle::distinct_pool(0, 0); pool.validator_list = c.validator_list;
        pool.reserve_stake = c.reserve_stake; pool.pool_mint = JITOSOL_MINT; pool.manager_fee_account = c.manager_fee_account;
        pool.token_program_id = spl_token::ID; pool.stake_withdraw_bump_seed = bump; pool.lockup = Lockup::default();
        pool.total_lamports = 10_100_000; pool.pool_token_supply = 10_000_000; pool.last_update_epoch = 40;
        let mint = Mint { mint_authority: COption::Some(withdraw), supply: pool.pool_token_supply,
            decimals: 9, is_initialized: true, freeze_authority: COption::None };
        custody.accounts[MINT].key = JITOSOL_MINT;
        let mut program = 2_u32.to_le_bytes().to_vec();
        program.extend(Pubkey::find_program_address(&[JITO_STAKE_POOL_PROGRAM.as_ref()], &bpf_loader_upgradeable::ID).0.to_bytes());
        let mut list = vec![2]; list.extend(1_u32.to_le_bytes()); list.extend(0_u32.to_le_bytes()); list.resize(82, 0xa5);
        let mut reserve = borsh1::to_vec(&StakeStateV2::Initialized(Meta { rent_exempt_reserve: 123,
            authorized: Authorized { staker: withdraw, withdrawer: withdraw }, lockup: Lockup::default() })).unwrap();
        reserve.resize(200, 0xa5);
        let receiver = TokenAccount { mint: JITOSOL_MINT, owner: key(235), amount: 0,
            delegate: COption::Some(key(236)), state: AccountState::Initialized, is_native: COption::None,
            delegated_amount: 0, close_authority: COption::Some(key(237)) };
        let mut receiver_bytes = vec![0xa5; TokenAccount::LEN]; TokenAccount::pack(receiver, &mut receiver_bytes).unwrap();
        let account = |key, owner, data: Vec<u8>, executable| BackingAccount { key, owner,
            lamports: custody.rent.minimum_balance(data.len()), data, executable, signer: false, writable: false };
        let extra = vec![account(JITO_STAKE_POOL_PROGRAM, bpf_loader_upgradeable::ID, program, true),
            account(JITO_STAKE_POOL, JITO_STAKE_POOL_PROGRAM, vec![], false),
            account(c.validator_list, JITO_STAKE_POOL_PROGRAM, list, false),
            account(c.reserve_stake, STAKE_PROGRAM_ID, reserve, false),
            account(c.manager_fee_account, spl_token::ID, receiver_bytes.clone(), false)];
        let referrer = if shared { None } else { Some(account(c.referrer_token_account, spl_token::ID, receiver_bytes, false)) };
        custody.accounts.extend(extra); if let Some(referrer) = referrer { custody.accounts.push(referrer); }
        let mut result = Self { custody, pool, mint, clock: Clock { epoch: 40, unix_timestamp: 1_000_000, ..Clock::default() } };
        result.sync(); result
    }
    pub fn sync(&mut self) {
        let mut bytes = borsh1::to_vec(&self.pool).unwrap(); bytes.resize(611, 0xa5);
        self.custody.accounts[POOL].lamports = self.custody.rent.minimum_balance(bytes.len());
        self.custody.accounts[POOL].data = bytes;
        Mint::pack(self.mint, &mut self.custody.accounts[MINT].data).unwrap();
    }
}
