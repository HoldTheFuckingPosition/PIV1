//! Read-only authentication for the four isolated KIF claim accounts.
//!
//! Executing program ID and Rent are trusted runtime inputs, never instruction
//! data. Outputs are point-in-time evidence, not authorization capabilities.
//! A future handler must reauthenticate around interactions and atomically couple
//! actual transfer/state writes. No current registry, round, pool, principal
//! custody or Clock is read, and pause does not block an already-earned claim.

use anchor_lang::{
    prelude::{AccountInfo, Pubkey, Rent},
    solana_program::system_program,
};
use crate::{
    accounts::{decode_state, native_balance, rent_floor, seeds, validate_pda,
               CONFIG_DISCRIMINATOR, STAKE_PROGRAM_ID},
    errors::{Piv1Error, Piv1Result},
    state::{GuardianReward, KifClaimCustodyObservation, PivConfig},
};

/// Immutable earned-owner tuple prefix selected for Task 2.7 under D-026.
pub const GUARDIAN_REWARD_SEED: &[u8] = b"guardian-reward";
/// First eight SHA-256 bytes of `account:GuardianReward`.
pub const GUARDIAN_REWARD_DISCRIMINATOR: [u8; 8] = [169, 109, 89, 17, 75, 171, 105, 39];

/// The entitled guardian account is both signer and fixed payment destination.
#[derive(Clone, Copy)]
pub struct KifClaimAccountInfos<'a, 'info> {
    pub config: &'a AccountInfo<'info>,
    pub guardian_reward: &'a AccountInfo<'info>,
    pub kif_sol: &'a AccountInfo<'info>,
    pub guardian: &'a AccountInfo<'info>,
}

/// Owned isolated snapshots; claim amount/backing validation is a separate step.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AuthenticatedKifClaimAccounts {
    config: Box<PivConfig>,
    reward: GuardianReward,
    custody: KifClaimCustodyObservation,
}

impl AuthenticatedKifClaimAccounts {
    pub fn config(&self) -> &PivConfig { &self.config }
    pub fn reward(&self) -> &GuardianReward { &self.reward }
    pub fn custody(&self) -> KifClaimCustodyObservation { self.custody }
}

/// Validates stored earned ownership without a live membership/freshness check.
/// Reward PDA seeds are prefix, guardian bytes, little-endian u64 revision, u8 slot.
/// The 84-byte reward allocation preserves the existing variable Borsh/zero-tail
/// envelope; `None` activity leaves exactly eight zero padding bytes.
///
/// The System-owned, empty, non-executable guardian must be writable and sign.
/// No on-curve constraint is imposed: a System-owned PDA wallet can sign by CPI.
/// Program-owned/nonempty smart wallets and alternate payouts are unsupported.
/// Destination rent exemption is not a spending-control requirement; only source
/// and state accounts require their own checked runtime rent backing.
#[inline(never)]
pub fn authenticate_kif_claim_accounts(
    trusted_runtime_program_id: &Pubkey,
    trusted_runtime_rent: &Rent,
    accounts: KifClaimAccountInfos<'_, '_>,
) -> Piv1Result<AuthenticatedKifClaimAccounts> {
    let program_id = trusted_runtime_program_id;
    if *program_id == system_program::ID || *program_id == spl_token::ID
        || *program_id == STAKE_PROGRAM_ID
    {
        return Err(Piv1Error::InvalidProgramIdentity);
    }
    let all = [accounts.config, accounts.guardian_reward, accounts.kif_sol, accounts.guardian];
    for (index, left) in all.iter().enumerate() {
        if !left.is_writable { return Err(Piv1Error::AccountNotWritable); }
        for right in &all[index + 1..] {
            if left.key == right.key { return Err(Piv1Error::AccountAlias); }
        }
    }
    if !accounts.guardian.is_signer { return Err(Piv1Error::MissingGuardianSignature); }
    let config: Box<PivConfig> = decode_state(accounts.config, program_id,
        trusted_runtime_rent, PivConfig::SPACE, CONFIG_DISCRIMINATOR)?;
    config.validate_initialized()?;
    if config.system_program != system_program::ID || config.token_program != spl_token::ID
        || config.stake_program != STAKE_PROGRAM_ID
    {
        return Err(Piv1Error::InvalidProgramIdentity);
    }
    let piv_roles = [config.piv_authority, config.active_distribution,
        config.principal_jito_vault, config.pending_jito_vault, config.pending_sol_vault,
        config.principal_sol_queue, config.operational_sol_vault, config.distribution_escrow,
        config.kif_sol_vault, config.guardian_registry];
    let external_roles = [config.stake_pool_program, config.stake_pool, config.validator_list,
        config.reserve_stake, config.jitosol_mint, config.token_program, config.stake_program,
        config.system_program, config.manager_fee_account, config.referrer_token_account,
        config.htfp_recipient, config.team_owner_recipient];
    if piv_roles.contains(accounts.config.key) || external_roles.contains(accounts.config.key)
        || piv_roles.contains(accounts.guardian.key)
    {
        return Err(Piv1Error::AccountAlias);
    }
    // Guardian overlap with external HTFP/Team beneficiary wallets is not forbidden.
    // Only PIV-controlled state/authority/custody destinations violate isolation.
    validate_pda(program_id, accounts.config.key, seeds::CONFIG, config.bumps.config)?;
    validate_pda(program_id, &config.kif_sol_vault, seeds::KIF_SOL, config.bumps.kif_sol_vault)?;
    let reward = authenticate_reward_account(program_id, trusted_runtime_rent,
                                             accounts.guardian_reward)?;
    if *accounts.guardian.key != reward.guardian { return Err(Piv1Error::InvalidGuardianSet); }
    let kif_sol = native_balance(accounts.kif_sol, &config.kif_sol_vault,
                                 rent_floor(trusted_runtime_rent, 0)?)?;
    let guardian = native_balance(accounts.guardian, &reward.guardian, 0)?;
    Ok(AuthenticatedKifClaimAccounts { config, reward,
        custody: KifClaimCustodyObservation { kif_sol, guardian_lamports: guardian.lamports } })
}

/// Standalone earned tuple authentication, shared without current-registry policy.
#[inline(never)]
pub(crate) fn authenticate_reward_account(
    program_id: &Pubkey,
    rent: &Rent,
    account: &AccountInfo<'_>,
) -> Piv1Result<GuardianReward> {
    let reward: GuardianReward = decode_state(account, program_id,
        rent, GuardianReward::SPACE, GUARDIAN_REWARD_DISCRIMINATOR)?;
    reward.validate()?;
    let revision = reward.registry_revision.to_le_bytes();
    let index = [reward.guardian_index];
    let (expected, bump) = Pubkey::try_find_program_address(
        &[GUARDIAN_REWARD_SEED, reward.guardian.as_ref(), &revision, &index], program_id,
    ).ok_or(Piv1Error::InvalidAccountPda)?;
    if *account.key != expected || reward.bump != bump {
        return Err(Piv1Error::InvalidAccountPda);
    }
    Ok(reward)
}
