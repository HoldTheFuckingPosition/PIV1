//! Read-only current guardian/activity snapshots derived from authenticated Clock.
//!
//! Executing program ID and Rent are trusted runtime inputs. Owned results are
//! point-in-time evidence, not authority to execute a heartbeat or distribution.
//! Future execution must refresh account, membership, activity and Clock reads.
//! No wallet account, signer, writable privilege, pool, custody or round is needed.

use anchor_lang::{
    prelude::{AccountInfo, Clock, Pubkey, Rent, SolanaSysvar},
    solana_program::{system_program, sysvar},
};
use crate::{
    accounts::{decode_state, seeds, validate_pda, CONFIG_DISCRIMINATOR, STAKE_PROGRAM_ID},
    constants::GUARDIAN_COUNT,
    errors::{Piv1Error, Piv1Result},
    kif_claim_accounts::authenticate_reward_account,
    state::{derive_kif_period, GuardianRegistry, GuardianReward, KifPeriod, PivConfig},
};

/// Fixed registry role seed selected as the Task 2.8 technical derivation.
pub const GUARDIAN_REGISTRY_SEED: &[u8] = b"guardian-registry";
/// First eight SHA-256 bytes of `account:GuardianRegistry`.
pub const GUARDIAN_REGISTRY_DISCRIMINATOR: [u8; 8] = [72, 14, 254, 2, 76, 233, 97, 92];
/// Locked Clock serialization: five eight-byte integer fields, with no envelope.
pub const CLOCK_ACCOUNT_SIZE: usize = 40;

/// Exactly nine read-only accounts; rewards must be in current registry slot order.
#[derive(Clone, Copy)]
pub struct GuardianClockAccountInfos<'a, 'info> {
    pub config: &'a AccountInfo<'info>,
    pub guardian_registry: &'a AccountInfo<'info>,
    pub rewards: [&'a AccountInfo<'info>; GUARDIAN_COUNT],
    pub clock: &'a AccountInfo<'info>,
}

/// Owned validated state and the complete Clock underlying the derived period.
/// Current reward accounts do not enumerate every historical earned liability.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AuthenticatedGuardianClockSnapshot {
    config: Box<PivConfig>,
    registry: GuardianRegistry,
    rewards: [GuardianReward; GUARDIAN_COUNT],
    clock: Clock,
    period: KifPeriod,
    activity_bitmap: u8,
    active_count: u8,
}

impl AuthenticatedGuardianClockSnapshot {
    pub fn config(&self) -> &PivConfig { &self.config }
    pub fn registry(&self) -> &GuardianRegistry { &self.registry }
    pub fn rewards(&self) -> &[GuardianReward; GUARDIAN_COUNT] { &self.rewards }
    pub fn clock(&self) -> &Clock { &self.clock }
    pub fn period(&self) -> KifPeriod { self.period }
    pub fn activity_bitmap(&self) -> u8 { self.activity_bitmap }
    pub fn active_count(&self) -> u8 { self.active_count }
}

/// Authenticates current membership then uses the accepted timing/activity rules.
///
/// Pause does not block inspection. None/older/newer activity is inactive for the
/// queried period; querying an earlier period is not a new activity-write rule.
/// No six-current-record sum is equated to global historical KIF liability, and
/// no earning provenance or wallet signing control is proved by this read path.
pub fn authenticate_guardian_clock_snapshot(
    trusted_runtime_program_id: &Pubkey,
    trusted_runtime_rent: &Rent,
    accounts: GuardianClockAccountInfos<'_, '_>,
) -> Piv1Result<AuthenticatedGuardianClockSnapshot> {
    let program_id = trusted_runtime_program_id;
    if *program_id == system_program::ID || *program_id == spl_token::ID
        || *program_id == STAKE_PROGRAM_ID
    {
        return Err(Piv1Error::InvalidProgramIdentity);
    }
    let all = [accounts.config, accounts.guardian_registry, accounts.rewards[0],
        accounts.rewards[1], accounts.rewards[2], accounts.rewards[3], accounts.rewards[4],
        accounts.rewards[5], accounts.clock];
    for (index, left) in all.iter().enumerate() {
        for right in &all[index + 1..] {
            if left.key == right.key { return Err(Piv1Error::AccountAlias); }
        }
    }
    let config: Box<PivConfig> = decode_state(accounts.config, program_id, trusted_runtime_rent,
                                        PivConfig::SPACE, CONFIG_DISCRIMINATOR)?;
    validate_snapshot_config(&config, program_id, &accounts)?;
    let registry: GuardianRegistry = decode_state(accounts.guardian_registry, program_id,
        trusted_runtime_rent, GuardianRegistry::SPACE, GUARDIAN_REGISTRY_DISCRIMINATOR)?;
    validate_snapshot_registry(&registry, &config, &all[..8])?;
    let rewards = authenticate_rewards(program_id, trusted_runtime_rent, accounts.rewards)?;
    for (index, reward) in rewards.iter().enumerate() {
        registry.validate_reward_binding(
            u8::try_from(index).map_err(|_| Piv1Error::ArithmeticOverflow)?, reward)?;
    }
    let clock = authenticate_clock(accounts.clock)?;
    let period = derive_kif_period(config.kif_anchor_timestamp, clock.unix_timestamp)?;
    let activity_bitmap = registry.activity_bitmap(&rewards, period.id)?;
    let active_count = u8::try_from(activity_bitmap.count_ones())
        .map_err(|_| Piv1Error::ArithmeticOverflow)?;
    GuardianRegistry::validate_activity_snapshot(activity_bitmap, active_count)?;
    Ok(AuthenticatedGuardianClockSnapshot {
        config, registry, rewards, clock, period, activity_bitmap, active_count,
    })
}

// Only borrowed state crosses these boundaries; no additional heap is used.
#[inline(never)]
fn validate_snapshot_config(config: &PivConfig, program_id: &Pubkey,
    accounts: &GuardianClockAccountInfos<'_, '_>) -> Piv1Result<()>
{
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
        || accounts.rewards.iter().any(|reward| piv_roles.contains(reward.key))
    {
        return Err(Piv1Error::AccountAlias);
    }
    validate_pda(program_id, accounts.config.key, seeds::CONFIG, config.bumps.config)?;
    validate_pda(program_id, &config.guardian_registry, GUARDIAN_REGISTRY_SEED,
                 config.bumps.guardian_registry)?;
    if *accounts.guardian_registry.key != config.guardian_registry {
        return Err(Piv1Error::InvalidAccountPda);
    }
    Ok(())
}

#[inline(never)]
fn validate_snapshot_registry(registry: &GuardianRegistry, config: &PivConfig,
    all: &[&AccountInfo<'_>]) -> Piv1Result<()>
{
    let piv_roles = [config.piv_authority, config.active_distribution,
        config.principal_jito_vault, config.pending_jito_vault, config.pending_sol_vault,
        config.principal_sol_queue, config.operational_sol_vault, config.distribution_escrow,
        config.kif_sol_vault, config.guardian_registry];
    registry.validate()?;
    if registry.bump != config.bumps.guardian_registry { return Err(Piv1Error::InvalidAccountPda); }
    if registry.revision != config.guardian_registry_revision {
        return Err(Piv1Error::InvalidGuardianSet);
    }
    // Guardian keys cannot designate PIV state/authority/custody. No ownership or
    // on-curve requirement is inferred, and external beneficiary overlap is valid.
    if registry.guardian_keys.iter().any(|guardian| piv_roles.contains(guardian)
        || all.iter().any(|account| account.key == guardian))
    {
        return Err(Piv1Error::AccountAlias);
    }
    Ok(())
}

#[inline(never)]
fn authenticate_rewards(program_id: &Pubkey, rent: &Rent,
    accounts: [&AccountInfo<'_>; GUARDIAN_COUNT]) -> Piv1Result<[GuardianReward; GUARDIAN_COUNT]>
{
    let decoded = accounts.map(|account|
        authenticate_reward_account(program_id, rent, account));
    let [r0, r1, r2, r3, r4, r5] = decoded;
    let rewards = [r0?, r1?, r2?, r3?, r4?, r5?];
    Ok(rewards)
}

fn authenticate_clock(account: &AccountInfo<'_>) -> Piv1Result<Clock> {
    if *account.key != sysvar::clock::ID { return Err(Piv1Error::InvalidClockAccount); }
    if *account.owner != sysvar::ID { return Err(Piv1Error::InvalidAccountOwner); }
    if account.executable { return Err(Piv1Error::ExecutableAccount); }
    // The pinned official decoder uses data.borrow(). This fallible immutable
    // borrow both preflights conflicts and remains held through that nested read.
    // No mutation or external call can intervene; lamports and rent are irrelevant.
    let data = account.try_borrow_data().map_err(|_| Piv1Error::AccountBorrowFailed)?;
    if data.len() != CLOCK_ACCOUNT_SIZE { return Err(Piv1Error::InvalidAccountSize); }
    Clock::from_account_info(account).map_err(|_| Piv1Error::InvalidAccountData)
}
