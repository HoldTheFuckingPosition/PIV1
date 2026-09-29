//! Fixed custody imported from genuine model transitions. This is host account
//! evidence, not proof that the model's earlier lifecycle executed on-chain.
use anchor_lang::{prelude::{AccountInfo, Pubkey, Rent}, AnchorDeserialize,
    solana_program::{program_option::COption, program_pack::Pack, system_program}};
use piv1::{accounts::{FixedAccountInfos, CONFIG_DISCRIMINATOR}, state::PivConfig};
use spl_token::state::{Account as TokenAccount, AccountState, Mint};
use super::{pending_custody, support::{kif_claim_custody::{BackingAccount, envelope, key, PROGRAM},
    vault_custody_model::{World, PENDING, PRINCIPAL, OPERATIONS, ESCROW, KIF, PENDING_TOKEN, PRINCIPAL_TOKEN}}};

pub const CONFIG: usize = 0;
pub const ROUND: usize = 1;
pub const PENDING_SOL: usize = 2;
pub const PRINCIPAL_SOL: usize = 3;
pub const OPERATIONAL_SOL: usize = 4;
pub const ESCROW_SOL: usize = 5;
pub const KIF_SOL: usize = 6;
pub const PRINCIPAL_JITO: usize = 7;
pub const PENDING_JITO: usize = 8;
pub const AUTHORITY: usize = 9;
pub const MINT: usize = 10;
pub const SYSTEM: usize = 11;
pub const TOKEN: usize = 12;

#[derive(Clone, Debug, PartialEq)]
pub struct Fixture { pub accounts: Vec<BackingAccount>, pub rent: Rent }
impl Fixture {
    pub fn from_world(w: &World) -> Self {
        let f = pending_custody::Fixture::from_state(w.config.clone(), w.round,
            w.spendable(PENDING).unwrap(), w.tokens[PENDING_TOKEN]);
        let c = f.config(); let rent = f.rent;
        let [config, round, pending, pending_token] = f.accounts;
        let native = |key, index| BackingAccount { key, owner: system_program::ID, signer: false,
            writable: false, executable: false, data: vec![], lamports: rent.minimum_balance(0) + w.spendable(index).unwrap() };
        let token = TokenAccount { mint: c.jitosol_mint, owner: c.piv_authority,
            amount: w.tokens[PRINCIPAL_TOKEN], delegate: COption::None, state: AccountState::Initialized,
            is_native: COption::None, delegated_amount: 0, close_authority: COption::None };
        let mut bytes = vec![0xa5; TokenAccount::LEN]; TokenAccount::pack(token, &mut bytes).unwrap();
        let principal_token = BackingAccount { key: c.principal_jito_vault, owner: spl_token::ID,
            signer: false, writable: false, executable: false, lamports: rent.minimum_balance(bytes.len()), data: bytes };
        let mint = Mint { mint_authority: COption::Some(key(223)), supply: 10_000_000,
            decimals: 9, is_initialized: true, freeze_authority: COption::None };
        let mut bytes = vec![0xb6; Mint::LEN]; Mint::pack(mint, &mut bytes).unwrap();
        let mint = BackingAccount { key: c.jitosol_mint, owner: spl_token::ID, signer: false,
            writable: false, executable: false, lamports: rent.minimum_balance(bytes.len()), data: bytes };
        let program = |key| BackingAccount { key, owner: key_for_loader(), signer: false,
            writable: false, executable: true, lamports: 1, data: vec![7; 67] };
        let accounts = vec![config, round, pending, native(c.principal_sol_queue, PRINCIPAL),
            native(c.operational_sol_vault, OPERATIONS), native(c.distribution_escrow, ESCROW),
            native(c.kif_sol_vault, KIF), principal_token, pending_token,
            BackingAccount { key: c.piv_authority, owner: system_program::ID, signer: false,
                writable: false, executable: false, lamports: 0, data: vec![] }, mint,
            program(system_program::ID), program(spl_token::ID)];
        Self { accounts, rent }
    }
    pub fn config(&self) -> PivConfig { PivConfig::deserialize(&mut &self.accounts[CONFIG].data[8..]).unwrap() }
    pub fn edit_config(&mut self, edit: impl FnOnce(&mut PivConfig)) {
        let mut c = self.config(); edit(&mut c);
        self.accounts[CONFIG].data = envelope(&c, CONFIG_DISCRIMINATOR, PivConfig::SPACE);
    }
    pub fn token_units(&self, index: usize) -> u64 { TokenAccount::unpack(&self.accounts[index].data).unwrap().amount }
    pub fn set_token_units(&mut self, index: usize, units: u64) {
        self.accounts[index].data[64..72].copy_from_slice(&units.to_le_bytes());
    }
    /// Explicit outside donation vector, independent of handler observations:
    /// pending SOL, principal SOL, escrow SOL, KIF SOL, pending/principal tokens.
    pub fn donate(&mut self, additions: [u64; 6]) {
        for (index, amount) in [PENDING_SOL, PRINCIPAL_SOL, ESCROW_SOL, KIF_SOL].into_iter().zip(additions) {
            self.accounts[index].lamports += amount;
        }
        for (index, amount) in [(PENDING_JITO, additions[4]), (PRINCIPAL_JITO, additions[5])] {
            self.set_token_units(index, self.token_units(index) + amount);
        }
        for source in [PRINCIPAL_SOL, ESCROW_SOL, KIF_SOL] {
            let pos = match source { PRINCIPAL_SOL => 1, ESCROW_SOL => 2, _ => 3 };
            if additions[pos] > 0 { self.accounts[source].writable = true; self.accounts[PENDING_SOL].writable = true; }
        }
        if additions[5] > 0 { self.accounts[PRINCIPAL_JITO].writable = true; self.accounts[PENDING_JITO].writable = true; }
    }
    pub fn with_infos<T>(&mut self, action: impl FnOnce(&[AccountInfo<'_>]) -> T) -> T {
        let infos: Vec<_> = self.accounts.iter_mut().map(BackingAccount::info).collect(); action(&infos)
    }
}
fn key_for_loader() -> Pubkey { key(224) }
pub fn roles<'a, 'info>(a: &'a [AccountInfo<'info>]) -> FixedAccountInfos<'a, 'info> {
    FixedAccountInfos { config: &a[0], active_distribution: &a[1], pending_sol: &a[2], principal_sol: &a[3],
        operational_sol: &a[4], distribution_escrow: &a[5], kif_sol: &a[6], principal_jito: &a[7], pending_jito: &a[8] }
}
// Every PDA is derived under the same synthetic executing program as the parent
// pending fixture; this assertion is available to independent signer oracles.
pub const FIXTURE_PROGRAM: Pubkey = PROGRAM;
