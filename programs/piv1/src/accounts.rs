//! Read-only authentication of the fixed PIV1 state and custody topology.
//!
//! The program ID and Rent are explicit **trusted runtime inputs**: a future
//! handler must supply its executing program ID and `Rent::get()` (or separately
//! authenticated sysvar), never instruction data or unvalidated account fields.
//! This module does not authenticate the official pool/mint relationships,
//! guardian state, recipients, transfers, CPI, or signer/writable privileges.
//!
//! Outputs are owned point-in-time evidence, not receipts or authorization
//! capabilities. Reauthenticate after CPI or any account/state mutation. Pause
//! does not prevent reading balances. Future handlers enforce their own gates.

use anchor_lang::{
    prelude::{AccountInfo, Pubkey, Rent},
    solana_program::{program_pack::Pack, system_program},
    AnchorDeserialize,
};
use spl_token::state::{Account as TokenAccount, AccountState};

use crate::{
    errors::{Piv1Error, Piv1Result},
    state::{
        reconciliation::{
            assess_operational_surplus, economic_custody_obligations, economic_custody_surplus,
            EconomicCustodyObservation, OperationalSurplusAssessment, SolVaultBalance,
        },
        ActiveDistribution, PivConfig,
    },
};

/// Canonical Stake Program identity from the pinned solana-sdk-ids 2.2.1.
/// Anchor 0.32.1 does not reexport its stake module.
pub const STAKE_PROGRAM_ID: Pubkey =
    Pubkey::from_str_const("Stake11111111111111111111111111111111111111");
// Pinned solana-rent 2.2.1 ACCOUNT_STORAGE_OVERHEAD; not reexported by Anchor.
const RENT_STORAGE_OVERHEAD: u64 = 128;

/// Confirmed Phase 0 fixed seeds. These identify roles, not deployed addresses.
pub mod seeds {
    pub const CONFIG: &[u8] = b"config";
    pub const AUTHORITY: &[u8] = b"authority";
    pub const DISTRIBUTION: &[u8] = b"distribution";
    pub const PENDING_SOL: &[u8] = b"pending-sol";
    pub const PRINCIPAL_SOL: &[u8] = b"principal-sol";
    pub const OPERATIONAL_SOL: &[u8] = b"operational-sol";
    pub const DISTRIBUTION_ESCROW: &[u8] = b"distribution-escrow";
    pub const KIF_SOL: &[u8] = b"kif-sol";
    pub const PRINCIPAL_JITO: &[u8] = b"principal-jito-vault";
    pub const PENDING_JITO: &[u8] = b"pending-jito-vault";
}

/// First eight bytes of SHA-256(`account:PivConfig`), the default Anchor rule.
pub const CONFIG_DISCRIMINATOR: [u8; 8] = [98, 115, 11, 164, 170, 207, 163, 20];
/// First eight bytes of SHA-256(`account:ActiveDistribution`).
pub const DISTRIBUTION_DISCRIMINATOR: [u8; 8] = [104, 51, 125, 187, 226, 55, 209, 99];

/// Required actual accounts. PivAuthority is derived as an address only.
#[derive(Clone, Copy)]
pub struct FixedAccountInfos<'a, 'info> {
    pub config: &'a AccountInfo<'info>,
    pub active_distribution: &'a AccountInfo<'info>,
    pub pending_sol: &'a AccountInfo<'info>,
    pub principal_sol: &'a AccountInfo<'info>,
    pub operational_sol: &'a AccountInfo<'info>,
    pub distribution_escrow: &'a AccountInfo<'info>,
    pub kif_sol: &'a AccountInfo<'info>,
    pub principal_jito: &'a AccountInfo<'info>,
    pub pending_jito: &'a AccountInfo<'info>,
}

/// Token units and native account funding are separate dimensions.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct TokenVaultBalance {
    pub token_units: u64,
    /// Native balance and runtime-derived rent floor, never JitoSOL units.
    pub native: SolVaultBalance,
}

/// Owned snapshot of authenticated accounts and mutually bound stored state.
///
/// Base authentication tolerates economic deficits and token native excess so
/// unexpected transfers do not prevent inspection. `economic_observation`
/// requires covered obligations and rejects unsupported token native excess.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AuthenticatedFixedAccounts {
    config: PivConfig,
    distribution: ActiveDistribution,
    economic: EconomicCustodyObservation,
    operational: SolVaultBalance,
    principal_jito: TokenVaultBalance,
    pending_jito: TokenVaultBalance,
}

impl AuthenticatedFixedAccounts {
    pub fn config(&self) -> &PivConfig { &self.config }
    pub fn distribution(&self) -> &ActiveDistribution { &self.distribution }
    pub fn operational_sol(&self) -> SolVaultBalance { self.operational }
    pub fn principal_jito(&self) -> TokenVaultBalance { self.principal_jito }
    pub fn pending_jito(&self) -> TokenVaultBalance { self.pending_jito }

    /// Operational funding provenance remains unsupported, regardless of balance.
    pub fn operational_surplus(&self) -> Piv1Result<OperationalSurplusAssessment> {
        assess_operational_surplus(self.operational)
    }

    /// Derive the existing reconciliation observation only when every economic
    /// vault covers its own state-derived obligation. Excess at another vault
    /// cannot conceal a deficit. No transfers or ledger updates are performed.
    ///
    /// Native token-account excess remains visible through the token getters,
    /// but this accessor rejects it: no safe normalization path exists yet.
    /// A single unsolicited lamport can therefore block this economic accessor;
    /// resolving that liveness limitation is required before handler integration.
    pub fn economic_observation(&self) -> Piv1Result<EconomicCustodyObservation> {
        if self.principal_jito.native.economic_lamports()? != 0
            || self.pending_jito.native.economic_lamports()? != 0
        {
            return Err(Piv1Error::UnsupportedTokenNativeExcess);
        }
        economic_custody_surplus(&self.config, &self.distribution, self.economic)?;
        Ok(self.economic)
    }
}

/// Authenticate the fixed accounts using trusted execution context.
///
/// State allocations remain exactly `PivConfig::SPACE` and
/// `ActiveDistribution::SPACE`. The discriminator precedes the unchanged Borsh
/// payload. Options can shorten that payload; every unconsumed allocation byte
/// must be zero. This explicit envelope rule is stronger than generic Anchor
/// serialization: future writers must clear the unused tail when Options shrink.
/// No production program ID is selected or serialized as a root of trust.
pub fn authenticate_fixed_accounts(
    trusted_runtime_program_id: &Pubkey,
    trusted_runtime_rent: &Rent,
    accounts: FixedAccountInfos<'_, '_>,
) -> Piv1Result<AuthenticatedFixedAccounts> {
    let program_id = trusted_runtime_program_id;
    if *program_id == system_program::ID || *program_id == spl_token::ID
        || *program_id == STAKE_PROGRAM_ID
    {
        return Err(Piv1Error::InvalidProgramIdentity);
    }
    let all = [accounts.config, accounts.active_distribution, accounts.pending_sol,
        accounts.principal_sol, accounts.operational_sol, accounts.distribution_escrow,
        accounts.kif_sol, accounts.principal_jito, accounts.pending_jito];
    for (i, left) in all.iter().enumerate() {
        for right in &all[i + 1..] {
            if left.key == right.key {
                return Err(Piv1Error::AccountAlias);
            }
        }
    }
    let config: PivConfig = decode_state(accounts.config, program_id,
        trusted_runtime_rent, PivConfig::SPACE, CONFIG_DISCRIMINATOR)?;
    config.validate_initialized()?;
    if config.system_program != system_program::ID || config.token_program != spl_token::ID
        || config.stake_program != STAKE_PROGRAM_ID
    {
        return Err(Piv1Error::InvalidProgramIdentity);
    }
    validate_pda(program_id, accounts.config.key, seeds::CONFIG, config.bumps.config)?;
    for (key, seed, bump) in [
        (&config.piv_authority, seeds::AUTHORITY, config.bumps.piv_authority),
        (&config.active_distribution, seeds::DISTRIBUTION, config.bumps.active_distribution),
        (&config.pending_sol_vault, seeds::PENDING_SOL, config.bumps.pending_sol_vault),
        (&config.principal_sol_queue, seeds::PRINCIPAL_SOL, config.bumps.principal_sol_queue),
        (&config.operational_sol_vault, seeds::OPERATIONAL_SOL, config.bumps.operational_sol_vault),
        (&config.distribution_escrow, seeds::DISTRIBUTION_ESCROW, config.bumps.distribution_escrow),
        (&config.kif_sol_vault, seeds::KIF_SOL, config.bumps.kif_sol_vault),
        (&config.principal_jito_vault, seeds::PRINCIPAL_JITO, config.bumps.principal_jito_vault),
        (&config.pending_jito_vault, seeds::PENDING_JITO, config.bumps.pending_jito_vault),
    ] {
        validate_pda(program_id, key, seed, bump)?;
    }
    // Config itself is not included in the pure model's explicit address list.
    // It must not alias any remaining externally configured role either.
    if [config.stake_pool_program, config.stake_pool, config.validator_list,
        config.reserve_stake, config.jitosol_mint, config.token_program,
        config.stake_program, config.system_program, config.manager_fee_account,
        config.referrer_token_account, config.htfp_recipient,
        config.team_owner_recipient, config.guardian_registry, config.piv_authority]
        .contains(accounts.config.key)
    {
        return Err(Piv1Error::AccountAlias);
    }
    if accounts.active_distribution.key != &config.active_distribution {
        return Err(Piv1Error::InvalidAccountPda);
    }
    let distribution: ActiveDistribution = decode_state(accounts.active_distribution,
        program_id, trusted_runtime_rent, ActiveDistribution::SPACE,
        DISTRIBUTION_DISCRIMINATOR)?;
    if distribution.bump != config.bumps.active_distribution {
        return Err(Piv1Error::InvalidAccountPda);
    }
    // Includes complete state validation, active offsets, sequence and checked
    // accounting arithmetic. This does not invent recipient/registry rotation policy.
    economic_custody_obligations(&config, &distribution)?;
    let native_floor = rent_floor(trusted_runtime_rent, 0)?;
    let pending_sol = native_balance(accounts.pending_sol, &config.pending_sol_vault, native_floor)?;
    let principal_sol = native_balance(accounts.principal_sol, &config.principal_sol_queue, native_floor)?;
    let operational = native_balance(accounts.operational_sol, &config.operational_sol_vault, native_floor)?;
    let distribution_escrow = native_balance(accounts.distribution_escrow, &config.distribution_escrow, native_floor)?;
    let kif_sol = native_balance(accounts.kif_sol, &config.kif_sol_vault, native_floor)?;
    let token_floor = rent_floor(trusted_runtime_rent, TokenAccount::LEN)?;
    let principal_jito = token_balance(accounts.principal_jito, &config.principal_jito_vault,
        &config, token_floor)?;
    let pending_jito = token_balance(accounts.pending_jito, &config.pending_jito_vault,
        &config, token_floor)?;
    Ok(AuthenticatedFixedAccounts {
        config, distribution, operational, principal_jito, pending_jito,
        economic: EconomicCustodyObservation {
            pending_sol, principal_sol, distribution_escrow, kif_sol,
            principal_jitosol_units: principal_jito.token_units,
            pending_jitosol_units: pending_jito.token_units,
        },
    })
}

pub(crate) fn validate_pda(program_id: &Pubkey, key: &Pubkey, seed: &[u8], bump: u8) -> Piv1Result<()> {
    let (expected, canonical_bump) = Pubkey::try_find_program_address(&[seed], program_id)
        .ok_or(Piv1Error::InvalidAccountPda)?;
    if *key != expected || bump != canonical_bump {
        return Err(Piv1Error::InvalidAccountPda);
    }
    Ok(())
}

fn validate_header(account: &AccountInfo<'_>, owner: &Pubkey) -> Piv1Result<()> {
    if account.owner != owner { return Err(Piv1Error::InvalidAccountOwner); }
    if account.executable { return Err(Piv1Error::ExecutableAccount); }
    Ok(())
}

fn native_funding(account: &AccountInfo<'_>, floor: u64) -> Piv1Result<SolVaultBalance> {
    let lamports = **account.try_borrow_lamports().map_err(|_| Piv1Error::AccountBorrowFailed)?;
    if lamports < floor { return Err(Piv1Error::AccountRentDeficit); }
    Ok(SolVaultBalance { lamports, non_economic_floor_lamports: floor })
}

pub(crate) fn decode_state<T: AnchorDeserialize>(account: &AccountInfo<'_>, program_id: &Pubkey,
    rent: &Rent, space: usize, discriminator: [u8; 8]) -> Piv1Result<T>
{
    validate_header(account, program_id)?;
    let data = account.try_borrow_data().map_err(|_| Piv1Error::AccountBorrowFailed)?;
    if data.len() != space { return Err(Piv1Error::InvalidAccountSize); }
    if data.get(..8) != Some(discriminator.as_slice()) {
        return Err(Piv1Error::InvalidAccountDiscriminator);
    }
    let mut remaining = &data[8..];
    let decoded = T::deserialize(&mut remaining).map_err(|_| Piv1Error::InvalidAccountData)?;
    if remaining.iter().any(|byte| *byte != 0) { return Err(Piv1Error::InvalidAccountData); }
    native_funding(account, rent_floor(rent, space)?)?;
    Ok(decoded)
}

pub(crate) fn native_balance(account: &AccountInfo<'_>, key: &Pubkey, floor: u64) -> Piv1Result<SolVaultBalance> {
    if account.key != key { return Err(Piv1Error::InvalidAccountPda); }
    validate_header(account, &system_program::ID)?;
    let data = account.try_borrow_data().map_err(|_| Piv1Error::AccountBorrowFailed)?;
    if !data.is_empty() { return Err(Piv1Error::InvalidAccountSize); }
    native_funding(account, floor)
}

fn token_balance(account: &AccountInfo<'_>, key: &Pubkey, config: &PivConfig,
    floor: u64) -> Piv1Result<TokenVaultBalance>
{
    if account.key != key { return Err(Piv1Error::InvalidAccountPda); }
    validate_header(account, &spl_token::ID)?;
    let data = account.try_borrow_data().map_err(|_| Piv1Error::AccountBorrowFailed)?;
    if data.len() != TokenAccount::LEN { return Err(Piv1Error::InvalidAccountSize); }
    let token = TokenAccount::unpack(&data).map_err(|_| Piv1Error::InvalidTokenCustody)?;
    if token.mint != config.jitosol_mint || token.owner != config.piv_authority
        || token.state != AccountState::Initialized || token.is_native.is_some()
        || token.delegate.is_some() || token.delegated_amount != 0
        || token.close_authority.is_some()
    {
        return Err(Piv1Error::InvalidTokenCustody);
    }
    Ok(TokenVaultBalance { token_units: token.amount, native: native_funding(account, floor)? })
}

/// Delegate rent pricing to the pinned runtime primitive after checking its
/// integer preconditions. No PIV1 economic calculation uses floating point.
pub(crate) fn rent_floor(rent: &Rent, data_len: usize) -> Piv1Result<u64> {
    if !rent.exemption_threshold.is_finite() || rent.exemption_threshold < 0.0
        || rent.burn_percent > 100
    {
        return Err(Piv1Error::InvalidRent);
    }
    let bytes = u64::try_from(data_len).map_err(|_| Piv1Error::ArithmeticOverflow)?;
    RENT_STORAGE_OVERHEAD.checked_add(bytes)
        .and_then(|n| n.checked_mul(rent.lamports_per_byte_year))
        .ok_or(Piv1Error::ArithmeticOverflow)?;
    let minimum = rent.minimum_balance(data_len);
    // The runtime saturates an out-of-range float conversion. Do not accept
    // that artificial floor as authenticated evidence.
    if minimum == u64::MAX {
        return Err(Piv1Error::ArithmeticOverflow);
    }
    Ok(minimum)
}
