//! Approved, deterministic genesis MODEL preparation without account creation.
//!
//! Exact approved bytes and fresh Squads authentication are composed in one
//! call. Proposed external keys remain unverified protocol/recipient declarations.
//! Returned state is neither an authenticated custody snapshot nor a capability
//! to create/persist accounts. Funding, actual target validity, vote activity,
//! first-payout readiness and real initialization remain separate dependencies.

use anchor_lang::{
    prelude::{AccountInfo, Clock, Pubkey, Rent, SolanaSysvar},
    solana_program::{instruction::get_stack_height, program_error::ProgramError, system_program},
};
use crate::{
    accounts::{seeds, STAKE_PROGRAM_ID},
    constants::*,
    errors::Piv1Error,
    guardian_clock_accounts::GUARDIAN_REGISTRY_SEED,
    instructions::initialize::{GenesisModelFormatError, GenesisModelParameters},
    kif_claim_accounts::GUARDIAN_REWARD_SEED,
    squads_execution::{dispatch_bootstrap, AuthenticatedSquadsBootstrapInvocation,
        SquadsBootstrapRoles, SquadsExecutionError},
    state::{derive_kif_period, ActiveDistribution, GuardianRegistry, GuardianReward,
        KifPeriod, PivConfig, PivConfigBumps},
};

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum GenesisModelError {
    Format(GenesisModelFormatError),
    Authorization(SquadsExecutionError),
    State(Piv1Error),
}
impl From<GenesisModelFormatError> for GenesisModelError {
    fn from(error: GenesisModelFormatError) -> Self { Self::Format(error) }
}
impl From<SquadsExecutionError> for GenesisModelError {
    fn from(error: SquadsExecutionError) -> Self { Self::Authorization(error) }
}
impl From<Piv1Error> for GenesisModelError {
    fn from(error: Piv1Error) -> Self { Self::State(error) }
}
pub type GenesisModelResult<T> = Result<T, GenesisModelError>;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum GenesisTargetRole {
    Config, ActiveDistribution, GuardianRegistry, GuardianReward(u8),
    PendingSol, PrincipalSol, OperationalSol, DistributionEscrow, KifSol,
    PrincipalJito, PendingJito,
}

/// Expected future topology only; no actual target owner, allocation, balance,
/// rent floor or creation readiness is authenticated by this descriptor.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct DerivedGenesisTarget {
    role: GenesisTargetRole,
    address: Pubkey,
    owner: Pubkey,
    size: usize,
    bump: u8,
}
impl DerivedGenesisTarget {
    pub fn role(&self) -> GenesisTargetRole { self.role }
    pub fn address(&self) -> Pubkey { self.address }
    pub fn owner(&self) -> Pubkey { self.owner }
    pub fn size(&self) -> usize { self.size }
    pub fn bump(&self) -> u8 { self.bump }
}

/// Private-field, non-Clone proposed state. Approval covers the declarations;
/// it does not prove official Jito identity, recipient control or KIF activity.
/// Re-run preparation after mutation/CPI; no persistent replay receipt exists.
#[derive(Debug, Eq, PartialEq)]
pub struct ApprovedGenesisModel {
    program: Pubkey,
    config: Box<PivConfig>,
    distribution: Box<ActiveDistribution>,
    registry: GuardianRegistry,
    rewards: [GuardianReward; 6],
    targets: Box<[DerivedGenesisTarget; 16]>,
    period: KifPeriod,
}
impl ApprovedGenesisModel {
    pub fn program(&self) -> Pubkey { self.program }
    pub fn proposed_config(&self) -> &PivConfig { &self.config }
    pub fn proposed_distribution(&self) -> &ActiveDistribution { &self.distribution }
    pub fn proposed_registry(&self) -> &GuardianRegistry { &self.registry }
    pub fn proposed_rewards(&self) -> &[GuardianReward; 6] { &self.rewards }
    pub fn targets(&self) -> &[DerivedGenesisTarget; 16] { &self.targets }
    /// Virtual authority address; there is no seventeenth created account here.
    pub fn piv_authority(&self) -> Pubkey { self.config.piv_authority }
    pub fn modeled_period(&self) -> KifPeriod { self.period }
}

/// Compose exact-byte decoding with fresh Task 2.17 authorization. No detached
/// authorization object or independent decoded parameters can enter this API.
pub fn prepare_approved_genesis_model(
    program: &Pubkey, accounts: &[AccountInfo<'_>], instruction_data: &[u8], roles: SquadsBootstrapRoles,
) -> GenesisModelResult<ApprovedGenesisModel> {
    dispatch(program, accounts, instruction_data, roles, cfg!(target_os = "solana"),
        get_stack_height, Clock::get, Rent::get)
}

/// Explicit synthetic host seam, absent from the Solana build/entrypoint.
#[cfg(not(target_os = "solana"))]
pub fn prepare_approved_genesis_model_with_host_context(
    program: &Pubkey, accounts: &[AccountInfo<'_>], instruction_data: &[u8], roles: SquadsBootstrapRoles,
    context: crate::squads_execution::ModeledSquadsInvocationContext,
) -> GenesisModelResult<ApprovedGenesisModel> {
    dispatch(program, accounts, instruction_data, roles, true,
        || context.stack_height, || Ok(context.clock), || Ok(context.rent))
}

/// Internal composition only: callers acquire their own trusted runtime context.
/// This is not a public production interface for injected approval/context.
pub(crate) fn dispatch(
    program: &Pubkey, accounts: &[AccountInfo<'_>], instruction_data: &[u8], roles: SquadsBootstrapRoles,
    available: bool, stack_height: impl FnOnce() -> usize,
    clock: impl FnOnce() -> Result<Clock, ProgramError>, rent: impl FnOnce() -> Result<Rent, ProgramError>,
) -> GenesisModelResult<ApprovedGenesisModel> {
    if !available { return Err(SquadsExecutionError::HostRuntimeUnavailable.into()); }
    let parameters = GenesisModelParameters::decode(instruction_data)?;
    let height = stack_height();
    if height != 2 { return Err(SquadsExecutionError::InvalidInvocation.into()); }
    let clock = clock().map_err(SquadsExecutionError::Runtime)?;
    let rent = rent().map_err(SquadsExecutionError::Runtime)?;
    let authority = dispatch_bootstrap(program, accounts, instruction_data, roles,
        parameters.vault_index, true, || height, || Ok(clock.clone()), || Ok(rent))?;
    let period = derive_kif_period(parameters.kif_anchor_timestamp, clock.unix_timestamp)?;
    build_model(program, accounts, roles, parameters, authority, period)
}

fn derive_target(
    program: &Pubkey, seeds: &[&[u8]], role: GenesisTargetRole, owner: Pubkey, size: usize,
) -> GenesisModelResult<DerivedGenesisTarget> {
    let (address, bump) = Pubkey::try_find_program_address(seeds, program).ok_or(Piv1Error::InvalidAccountPda)?;
    Ok(DerivedGenesisTarget { role, address, owner, size, bump })
}

#[inline(never)]
fn build_model(
    program: &Pubkey, accounts: &[AccountInfo<'_>], roles: SquadsBootstrapRoles,
    parameters: GenesisModelParameters, authority: AuthenticatedSquadsBootstrapInvocation, period: KifPeriod,
) -> GenesisModelResult<ApprovedGenesisModel> {
    if *program == system_program::ID || *program == spl_token::ID || *program == STAKE_PROGRAM_ID {
        return Err(Piv1Error::InvalidProgramIdentity.into());
    }
    // Decode established a bijection before indexing the fresh member set.
    let guardian_keys = parameters.guardian_slot_permutation.map(|index| authority.current_members()[usize::from(index)]);
    let (targets, piv_authority, authority_bump) = derive_topology(program, &guardian_keys)?;
    if authority.program() != *program || authority.config() != targets[0].address {
        return Err(Piv1Error::InvalidProgramIdentity.into());
    }
    let registry = GuardianRegistry::new(targets[2].bump, 0, guardian_keys)?;
    let reward = |slot: u8| GuardianReward::new(targets[3 + usize::from(slot)].bump, &registry, slot);
    let rewards = [reward(0)?, reward(1)?, reward(2)?, reward(3)?, reward(4)?, reward(5)?];
    let config = build_config(&parameters, &targets, piv_authority, authority_bump);
    config.validate_initialized()?;
    validate_aliases(&config, &targets, &guardian_keys, accounts, roles)?;
    let distribution = boxed_idle(targets[1].bump);
    distribution.validate()?;
    registry.validate()?;
    if config.guardian_registry_revision != registry.revision || config.bumps.guardian_registry != registry.bump {
        return Err(Piv1Error::InvalidGuardianSet.into());
    }
    for (slot, reward) in rewards.iter().enumerate() { registry.validate_reward_binding(slot as u8, reward)?; }
    Ok(ApprovedGenesisModel { program: *program, config, distribution, registry, rewards, targets, period })
}

// Bound large construction temporaries to separate frames. This is not an SBF
// stack/heap budget proof; no target build or runtime execution occurs here.
#[inline(never)]
fn boxed_idle(bump: u8) -> Box<ActiveDistribution> { Box::new(ActiveDistribution::new_idle(bump)) }

#[inline(never)]
fn derive_topology(program: &Pubkey, guardian_keys: &[Pubkey; 6])
    -> GenesisModelResult<(Box<[DerivedGenesisTarget; 16]>, Pubkey, u8)>
{
    use GenesisTargetRole as R;
    let fixed = |seed, role, owner, size| derive_target(program, &[seed], role, owner, size);
    let config_target = fixed(seeds::CONFIG, R::Config, *program, PivConfig::SPACE)?;
    let round = fixed(seeds::DISTRIBUTION, R::ActiveDistribution, *program, ActiveDistribution::SPACE)?;
    let registry_target = fixed(GUARDIAN_REGISTRY_SEED, R::GuardianRegistry, *program, GuardianRegistry::SPACE)?;
    let pending_sol = fixed(seeds::PENDING_SOL, R::PendingSol, system_program::ID, 0)?;
    let principal_sol = fixed(seeds::PRINCIPAL_SOL, R::PrincipalSol, system_program::ID, 0)?;
    let operational_sol = fixed(seeds::OPERATIONAL_SOL, R::OperationalSol, system_program::ID, 0)?;
    let escrow = fixed(seeds::DISTRIBUTION_ESCROW, R::DistributionEscrow, system_program::ID, 0)?;
    let kif_sol = fixed(seeds::KIF_SOL, R::KifSol, system_program::ID, 0)?;
    let principal_jito = fixed(seeds::PRINCIPAL_JITO, R::PrincipalJito, spl_token::ID, 165)?;
    let pending_jito = fixed(seeds::PENDING_JITO, R::PendingJito, spl_token::ID, 165)?;
    let (piv_authority, authority_bump) = Pubkey::try_find_program_address(&[seeds::AUTHORITY], program)
        .ok_or(Piv1Error::InvalidAccountPda)?;
    let reward_target = |slot: u8| derive_target(program,
        &[GUARDIAN_REWARD_SEED, guardian_keys[usize::from(slot)].as_ref(), &0_u64.to_le_bytes(), &[slot]],
        R::GuardianReward(slot), *program, GuardianReward::SPACE);
    let reward_targets = [reward_target(0)?, reward_target(1)?, reward_target(2)?,
        reward_target(3)?, reward_target(4)?, reward_target(5)?];
    let targets = [config_target, round, registry_target, reward_targets[0], reward_targets[1],
        reward_targets[2], reward_targets[3], reward_targets[4], reward_targets[5],
        pending_sol, principal_sol, operational_sol, escrow, kif_sol, principal_jito, pending_jito];
    Ok((Box::new(targets), piv_authority, authority_bump))
}

#[inline(never)]
fn build_config(parameters: &GenesisModelParameters, targets: &[DerivedGenesisTarget; 16],
    piv_authority: Pubkey, authority_bump: u8) -> Box<PivConfig>
{
    let config_target = &targets[0]; let round = &targets[1]; let registry_target = &targets[2];
    let pending_sol = &targets[9]; let principal_sol = &targets[10]; let operational_sol = &targets[11];
    let escrow = &targets[12]; let kif_sol = &targets[13];
    let principal_jito = &targets[14]; let pending_jito = &targets[15];
    let p = parameters.protocol;
    Box::new(PivConfig {
        version: STATE_LAYOUT_VERSION, is_initialized: true, paused: parameters.initially_paused,
        bumps: PivConfigBumps {
            config: config_target.bump, piv_authority: authority_bump, active_distribution: round.bump,
            principal_jito_vault: principal_jito.bump, pending_jito_vault: pending_jito.bump,
            pending_sol_vault: pending_sol.bump, principal_sol_queue: principal_sol.bump,
            operational_sol_vault: operational_sol.bump, distribution_escrow: escrow.bump,
            kif_sol_vault: kif_sol.bump, guardian_registry: registry_target.bump,
        },
        stake_pool_program: p.stake_pool_program, stake_pool: p.stake_pool,
        validator_list: p.validator_list, reserve_stake: p.reserve_stake, jitosol_mint: p.jitosol_mint,
        token_program: spl_token::ID, stake_program: STAKE_PROGRAM_ID, system_program: system_program::ID,
        manager_fee_account: p.manager_fee_account, referrer_token_account: p.referrer_token_account,
        piv_authority, active_distribution: round.address, principal_jito_vault: principal_jito.address,
        pending_jito_vault: pending_jito.address, pending_sol_vault: pending_sol.address,
        principal_sol_queue: principal_sol.address, operational_sol_vault: operational_sol.address,
        distribution_escrow: escrow.address, kif_sol_vault: kif_sol.address,
        htfp_recipient: parameters.htfp_recipient, team_owner_recipient: parameters.team_owner_recipient,
        guardian_registry: registry_target.address,
        basis_points_denominator: crate::math::BASIS_POINTS_DENOMINATOR as u16,
        htfp_reserve_bps: crate::math::HTFP_RESERVE_BPS as u16,
        permanent_compound_bps: crate::math::PERMANENT_COMPOUND_BPS as u16,
        team_owner_pool_bps: crate::math::TEAM_OWNER_POOL_BPS as u16,
        kif_bps: crate::math::KIF_BPS as u16,
        configured_slippage_bps: MAX_CONFIGURED_SLIPPAGE_BPS,
        slippage_hard_cap_bps: MAX_CONFIGURED_SLIPPAGE_BPS,
        minimum_distribution_interval_seconds: MINIMUM_DISTRIBUTION_INTERVAL_SECONDS,
        insufficient_retry_cooldown_seconds: INSUFFICIENT_RETRY_COOLDOWN_SECONDS,
        last_successful_preparation_at: None, last_valid_insufficient_attempt_at: None, next_distribution_sequence: 0,
        protected_principal_hwm_lamports: 0, accounted_historical_jitosol_units: 0,
        accounted_historical_sol_lamports: 0, accounted_pending_jitosol_units: 0, accounted_pending_sol_lamports: 0,
        next_cycle_yield_lamports: 0, kif_claim_liability_lamports: 0, collective_kif_carry_lamports: 0,
        cumulative_contribution_value_lamports: 0, cumulative_gross_yield_lamports: 0,
        cumulative_htfp_paid_lamports: 0, cumulative_team_owner_paid_lamports: 0,
        cumulative_kif_credited_lamports: 0, cumulative_kif_claimed_lamports: 0,
        cumulative_permanent_compound_lamports: 0, cumulative_retained_dust_lamports: 0,
        cumulative_zero_active_kif_compound_lamports: 0, cumulative_cooldown_yield_recorded_lamports: 0,
        kif_anchor_timestamp: parameters.kif_anchor_timestamp, kif_period_seconds: KIF_PERIOD_SECONDS,
        guardian_registry_revision: 0, migration_reserve: [0; CONFIG_MIGRATION_RESERVE_BYTES],
    })
}

fn validate_aliases(
    config: &PivConfig, targets: &[DerivedGenesisTarget; 16], guardians: &[Pubkey; 6],
    accounts: &[AccountInfo<'_>], roles: SquadsBootstrapRoles,
) -> GenesisModelResult<()> {
    let external = [config.stake_pool_program, config.stake_pool, config.validator_list,
        config.reserve_stake, config.jitosol_mint, config.token_program, config.stake_program,
        config.system_program, config.manager_fee_account, config.referrer_token_account,
        config.htfp_recipient, config.team_owner_recipient];
    let authentication = [roles.program, roles.program_data, roles.multisig, roles.proposal,
        roles.transaction, roles.vault, roles.instructions]; // Config is deliberately the first target.
    for (index, target) in targets.iter().enumerate() {
        if target.address == config.piv_authority || targets[..index].iter().any(|other| other.address == target.address)
            || external.contains(&target.address) || guardians.contains(&target.address)
            || authentication.iter().any(|role| *accounts[*role].key == target.address)
        {
            return Err(Piv1Error::AccountAlias.into());
        }
    }
    if guardians.contains(&config.piv_authority) || external.contains(&config.piv_authority)
        || authentication.iter().any(|role| *accounts[*role].key == config.piv_authority)
    {
        return Err(Piv1Error::AccountAlias.into());
    }
    Ok(())
}
