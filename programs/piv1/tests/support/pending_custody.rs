//! Synthetic four-account custody with an original funding baseline. Imported
//! lifecycle state is supplemental phase evidence, not continuous World custody.
use anchor_lang::{prelude::{AccountInfo, Pubkey, Rent}, AnchorDeserialize,
    solana_program::{program_option::COption, program_pack::Pack, system_program}};
use piv1::{accounts::{CONFIG_DISCRIMINATOR, DISTRIBUTION_DISCRIMINATOR, STAKE_PROGRAM_ID},
    pending_accounts::PendingAccountInfos, state::{PivConfig, ActiveDistribution}};
use super::kif_claim_custody::{BackingAccount, envelope, PROGRAM};
use spl_token::state::{Account as TokenAccount, AccountState};
pub const CONFIG: usize = 0;
pub const ROUND: usize = 1;
pub const SOL: usize = 2;
pub const TOKEN: usize = 3;

#[derive(Clone, Debug, PartialEq)]
pub struct Fixture {
    pub rent: Rent,
    pub accounts: [BackingAccount; 4],
    pub initial_native: u128,
    pub added_native: u128,
    pub initial_tokens: u64,
    pub added_tokens: u128,
}
impl Fixture {
    pub fn from_state(mut config: PivConfig, mut round: ActiveDistribution, sol: u64, tokens: u64) -> Self {
        let rent = Rent::default();
        let (config_key, bump) = Pubkey::find_program_address(&[b"config"], &PROGRAM);
        config.bumps.config = bump;
        macro_rules! bind {
            ($field:ident, $seed:literal) => {{
                let (key, bump) = Pubkey::find_program_address(&[$seed], &PROGRAM);
                config.$field = key; config.bumps.$field = bump;
            }};
        }
        bind!(piv_authority, b"authority"); bind!(active_distribution, b"distribution");
        bind!(pending_sol_vault, b"pending-sol"); bind!(principal_sol_queue, b"principal-sol");
        bind!(operational_sol_vault, b"operational-sol"); bind!(distribution_escrow, b"distribution-escrow");
        bind!(kif_sol_vault, b"kif-sol"); bind!(principal_jito_vault, b"principal-jito-vault");
        bind!(pending_jito_vault, b"pending-jito-vault");
        round.bump = config.bumps.active_distribution;
        config.system_program = system_program::ID; config.token_program = spl_token::ID;
        config.stake_program = STAKE_PROGRAM_ID;
        let state = |key, data: Vec<u8>, writable| BackingAccount {
            key, owner: PROGRAM, signer: false, writable, executable: false,
            lamports: rent.minimum_balance(data.len()), data };
        let token = TokenAccount { mint: config.jitosol_mint, owner: config.piv_authority,
            amount: tokens, delegate: COption::None, state: AccountState::Initialized,
            is_native: COption::None, delegated_amount: 0, close_authority: COption::None };
        let mut token_data = vec![0; TokenAccount::LEN]; TokenAccount::pack(token, &mut token_data).unwrap();
        let accounts = [
            state(config_key, envelope(&config, CONFIG_DISCRIMINATOR, PivConfig::SPACE), true),
            state(config.active_distribution, envelope(&round, DISTRIBUTION_DISCRIMINATOR, ActiveDistribution::SPACE), false),
            BackingAccount { key: config.pending_sol_vault, owner: system_program::ID,
                signer: false, writable: false, executable: false, lamports: rent.minimum_balance(0) + sol, data: vec![] },
            BackingAccount { key: config.pending_jito_vault, owner: spl_token::ID,
                signer: false, writable: false, executable: false,
                lamports: rent.minimum_balance(TokenAccount::LEN), data: token_data },
        ];
        Self { initial_native: accounts.iter().map(|a| u128::from(a.lamports)).sum(),
            added_native: 0, initial_tokens: tokens, added_tokens: 0, accounts, rent }
    }
    pub fn config(&self) -> PivConfig { PivConfig::deserialize(&mut &self.accounts[CONFIG].data[8..]).unwrap() }
    pub fn edit_config(&mut self, action: impl FnOnce(&mut PivConfig)) {
        let mut c = self.config(); action(&mut c);
        self.accounts[CONFIG].data = envelope(&c, CONFIG_DISCRIMINATOR, PivConfig::SPACE);
    }
    pub fn donate_native(&mut self, index: usize, amount: u64) {
        self.accounts[index].lamports = self.accounts[index].lamports.checked_add(amount).unwrap();
        self.added_native += u128::from(amount);
    }
    pub fn donate_tokens(&mut self, amount: u64) {
        let mut token = TokenAccount::unpack(&self.accounts[TOKEN].data).unwrap();
        token.amount = token.amount.checked_add(amount).unwrap();
        TokenAccount::pack(token, &mut self.accounts[TOKEN].data).unwrap();
        self.added_tokens += u128::from(amount);
    }
    pub fn validate_audit(&self) {
        assert_eq!(self.accounts.iter().map(|a| u128::from(a.lamports)).sum::<u128>(), self.initial_native + self.added_native);
        assert_eq!(u128::from(TokenAccount::unpack(&self.accounts[TOKEN].data).unwrap().amount),
            u128::from(self.initial_tokens) + self.added_tokens);
    }
    pub fn with_infos<T>(&mut self, action: impl FnOnce(&[AccountInfo<'_>; 4]) -> T) -> T {
        let [c, r, s, t] = &mut self.accounts; action(&[c.info(), r.info(), s.info(), t.info()])
    }
}
pub fn roles<'a, 'info>(a: &'a [AccountInfo<'info>; 4]) -> PendingAccountInfos<'a, 'info> {
    PendingAccountInfos { config: &a[0], active_distribution: &a[1], pending_sol: &a[2], pending_jito: &a[3] }
}
