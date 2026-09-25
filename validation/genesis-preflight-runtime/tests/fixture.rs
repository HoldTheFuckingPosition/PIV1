//! Independently serialized synthetic accounts; never a host preflight result.
#![allow(dead_code)]
#[path = "../../../programs/piv1/tests/support/squads_invocation.rs"]
pub mod support;
#[path = "../../../programs/piv1/tests/support/jito_identity_oracle.rs"]
mod oracle;
use anchor_lang::{prelude::Pubkey, solana_program::{bpf_loader_upgradeable,
    program_option::COption, program_pack::Pack, system_program}};
use piv1::{accounts::STAKE_PROGRAM_ID, instructions::initialize::*, integrations::jito_identity::*};
use solana_stake_interface::state::{Authorized, Lockup, Meta, StakeStateV2};
use spl_token::state::{Account as TokenAccount, AccountState, Mint};
use support::{key, BackingAccount, Fixture, PROGRAM_ACCOUNT, PROGRAM_DATA, MULTISIG, CONFIG};
const SQUADS: Pubkey = Pubkey::from_str_const("SQDS4ep65T869zMMBKyuUq6aD6EgTu8psMjkvj52pCf");
fn recipient(multisig: Pubkey, index: u8) -> (Pubkey,u8) {
    Pubkey::find_program_address(&[b"multisig",multisig.as_ref(),b"vault",&[index]],&SQUADS)
}
fn native_floor() -> u64 { 128 * 3480 * 2 }
fn account(address: Pubkey, owner: Pubkey, data: Vec<u8>, executable: bool, writable: bool) -> BackingAccount {
    BackingAccount { key: address, owner, data, executable, writable, signer: false, lamports: 0 }
}
#[derive(Clone)]
pub struct World { pub f: Fixture, pub parameters: GenesisModelParameters }
impl World {
    pub fn new(same_receiver: bool) -> Self {
        let program = support::PROGRAM;
        let indices = [0,255];
        let paused = false;
        let mut f = Fixture::new(); f.accounts.truncate(8); f.program = program;
        f.accounts[PROGRAM_ACCOUNT].key = program;
        let pd = Pubkey::find_program_address(&[program.as_ref()], &bpf_loader_upgradeable::ID).0;
        f.accounts[PROGRAM_DATA].key = pd;
        f.accounts[PROGRAM_ACCOUNT].data[4..36].copy_from_slice(pd.as_ref());
        f.accounts[CONFIG] = account(Pubkey::find_program_address(&[b"config"], &program).0,
            system_program::ID, vec![], false, true);
        let parameters = GenesisModelParameters { vault_index: 7, initially_paused: paused,
            protocol: DeclaredGenesisProtocol { stake_pool_program: JITO_STAKE_POOL_PROGRAM, stake_pool: JITO_STAKE_POOL,
                validator_list: key(201), reserve_stake: key(202), jitosol_mint: JITOSOL_MINT,
                manager_fee_account: key(203), referrer_token_account: if same_receiver { key(203) } else { key(204) } },
            htfp_recipient: recipient(f.accounts[MULTISIG].key,indices[0]).0,
            team_owner_recipient: recipient(f.accounts[MULTISIG].key,indices[1]).0, kif_anchor_timestamp: 0,
            guardian_slot_permutation: [5,4,3,2,1,0] };
        f.inner_data = parameters.encode().unwrap().to_vec(); f.rebuild_message();
        // Independent literal target recipes, not a call to production model/preflight.
        let pda = |seeds: &[&[u8]]| Pubkey::find_program_address(seeds, &program).0;
        let mut target_keys = vec![pda(&[b"config"]), pda(&[b"distribution"]), pda(&[b"guardian-registry"])];
        for slot in 0..6_u8 {
            target_keys.push(pda(&[b"guardian-reward", key(96-slot).as_ref(), &0_u64.to_le_bytes(), &[slot]]));
        }
        for seed in [b"pending-sol".as_slice(), b"principal-sol", b"operational-sol", b"distribution-escrow", b"kif-sol", b"principal-jito-vault", b"pending-jito-vault"] {
            target_keys.push(pda(&[seed]));
        }
        assert_eq!(target_keys.len(), 16);
        assert_eq!(target_keys[0], f.accounts[CONFIG].key);
        for address in target_keys.into_iter().skip(1) {
            f.accounts.push(account(address, system_program::ID, vec![], false, true));
        }
        let p = parameters.protocol;
        let (withdraw, bump) = Pubkey::find_program_address(&[p.stake_pool.as_ref(), b"withdraw"], &p.stake_pool_program);
        let mut program_bytes = 2_u32.to_le_bytes().to_vec();
        program_bytes.extend(Pubkey::find_program_address(&[p.stake_pool_program.as_ref()], &bpf_loader_upgradeable::ID).0.to_bytes());
        let mut pool = oracle::distinct_pool(0,0); pool.validator_list = p.validator_list; pool.reserve_stake = p.reserve_stake;
        pool.pool_mint = p.jitosol_mint; pool.manager_fee_account = p.manager_fee_account; pool.token_program_id = spl_token::ID;
        pool.stake_withdraw_bump_seed = bump; pool.lockup = Lockup::default();
        let mut pool_bytes = borsh1::to_vec(&pool).unwrap(); pool_bytes.resize(611, 0xA5);
        let mut list = vec![2]; list.extend(1_u32.to_le_bytes()); list.extend(0_u32.to_le_bytes()); list.resize(82, 0xA5);
        let mut reserve = borsh1::to_vec(&StakeStateV2::Initialized(Meta { rent_exempt_reserve: 123,
            authorized: Authorized { staker: withdraw, withdrawer: withdraw }, lockup: Lockup::default() })).unwrap();
        reserve.resize(200, 0xA5);
        let mut mint = vec![0;82]; Mint::pack(Mint { mint_authority: COption::Some(withdraw), supply: 9999,
            decimals: 9, is_initialized: true, freeze_authority: COption::None }, &mut mint).unwrap();
        let mut receiver = vec![0;165]; TokenAccount::pack(TokenAccount { mint: p.jitosol_mint, owner: key(207),
            amount: 99, delegate: COption::Some(key(208)), state: AccountState::Initialized, is_native: COption::None,
            delegated_amount: 1, close_authority: COption::Some(key(209)) }, &mut receiver).unwrap();
        f.accounts.extend([
            account(p.stake_pool_program, bpf_loader_upgradeable::ID, program_bytes, true, false),
            account(p.stake_pool, p.stake_pool_program, pool_bytes, false, false),
            account(p.validator_list, p.stake_pool_program, list, false, false),
            account(p.reserve_stake, STAKE_PROGRAM_ID, reserve, false, false),
            account(p.jitosol_mint, spl_token::ID, mint, false, false),
            account(p.manager_fee_account, spl_token::ID, receiver.clone(), false, false),
        ]);
        if !same_receiver { f.accounts.push(account(p.referrer_token_account, spl_token::ID, receiver, false, false)); }
        for address in [parameters.htfp_recipient,parameters.team_owner_recipient] {
            let mut vault=account(address,system_program::ID,vec![],false,false);
            vault.lamports=native_floor();f.accounts.push(vault);
        }
        f.rebuild_message(); Self { f, parameters }
    }
}
