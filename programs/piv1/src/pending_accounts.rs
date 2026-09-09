//! Four-role pending custody authentication. Runtime Program ID and Rent are
//! trusted inputs; owned observations are not receipts or authority capabilities.
//! No unrelated vault backing, official pool authenticity or token-native
//! excess classification is inferred. Reauthenticate around external effects.

use std::rc::Rc;
use anchor_lang::{prelude::{AccountInfo, Pubkey, Rent}, solana_program::{program_pack::Pack, system_program}};
use crate::{
    accounts::{decode_state, native_balance, rent_floor, token_balance, validate_fixed_config,
        TokenVaultBalance, CONFIG_DISCRIMINATOR, DISTRIBUTION_DISCRIMINATOR, STAKE_PROGRAM_ID},
    errors::{Piv1Error, Piv1Result},
    state::{reconciliation::{validate_custody_state_binding, SolVaultBalance},
        ActiveDistribution, PendingCustodyObservation, PivConfig},
};

#[derive(Clone, Copy)]
pub struct PendingAccountInfos<'a, 'info> {
    pub config: &'a AccountInfo<'info>,
    pub active_distribution: &'a AccountInfo<'info>,
    pub pending_sol: &'a AccountInfo<'info>,
    pub pending_jito: &'a AccountInfo<'info>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AuthenticatedPendingAccounts {
    config: Box<PivConfig>,
    distribution: Box<ActiveDistribution>,
    sol: SolVaultBalance,
    token: TokenVaultBalance,
}
impl AuthenticatedPendingAccounts {
    pub fn config(&self) -> &PivConfig { &self.config }
    pub fn distribution(&self) -> &ActiveDistribution { &self.distribution }
    /// Native token-account funding remains visible and unclassified.
    pub fn pending_jito(&self) -> TokenVaultBalance { self.token }
    pub fn observation(&self) -> PendingCustodyObservation {
        PendingCustodyObservation { pending_sol_vault_lamports: self.sol.lamports,
            pending_sol_non_economic_floor_lamports: self.sol.non_economic_floor_lamports,
            pending_jitosol_token_units: self.token.token_units }
    }
}

#[inline(never)]
pub fn authenticate_pending_accounts(program: &Pubkey, rent: &Rent,
    accounts: PendingAccountInfos<'_, '_>) -> Piv1Result<AuthenticatedPendingAccounts> {
    if *program == system_program::ID || *program == spl_token::ID || *program == STAKE_PROGRAM_ID {
        return Err(Piv1Error::InvalidProgramIdentity);
    }
    let all = [accounts.config, accounts.active_distribution, accounts.pending_sol, accounts.pending_jito];
    for (i, left) in all.iter().enumerate() {
        for right in &all[i + 1..] {
            if left.key == right.key || Rc::ptr_eq(&left.data, &right.data) {
                return Err(Piv1Error::AccountAlias);
            }
        }
    }
    let config: Box<PivConfig> = decode_state(accounts.config, program, rent,
        PivConfig::SPACE, CONFIG_DISCRIMINATOR)?;
    validate_fixed_config(&config, program, accounts.config.key)?;
    if accounts.active_distribution.key != &config.active_distribution {
        return Err(Piv1Error::InvalidAccountPda);
    }
    let distribution: Box<ActiveDistribution> = decode_state(accounts.active_distribution,
        program, rent, ActiveDistribution::SPACE, DISTRIBUTION_DISCRIMINATOR)?;
    if distribution.bump != config.bumps.active_distribution {
        return Err(Piv1Error::InvalidAccountPda);
    }
    validate_custody_state_binding(&config, &distribution)?;
    let sol = native_balance(accounts.pending_sol, &config.pending_sol_vault, rent_floor(rent, 0)?)?;
    let token = token_balance(accounts.pending_jito, &config.pending_jito_vault, &config,
        rent_floor(rent, spl_token::state::Account::LEN)?)?;
    Ok(AuthenticatedPendingAccounts { config, distribution, sol, token })
}
