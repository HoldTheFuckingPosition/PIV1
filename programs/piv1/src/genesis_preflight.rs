//! Fresh approved genesis model, protocol identity and current target observations.
//!
//! This read-only prerequisite does not create accounts or authorize funding or
//! persistence. Empty System-owned targets are currently unallocated; this is
//! not evidence that they were never initialized. Raw prefunding remains
//! unclassified. Rent shortfalls exclude operational liquidity and transaction
//! costs. Recipient control, transport and actual initialization remain deferred.
//! Refresh the complete observation after relevant account mutation/CPI.

use std::rc::Rc;
use anchor_lang::{
    prelude::{AccountInfo, Clock, Pubkey, Rent, SolanaSysvar},
    solana_program::{instruction::get_stack_height, program_error::ProgramError, system_program},
};
use crate::{
    accounts::rent_floor,
    errors::Piv1Error,
    genesis_model::{self, ApprovedGenesisModel, DerivedGenesisTarget, GenesisModelError},
    integrations::jito_identity::{authenticate_jito_identity, AuthenticatedJitoIdentity,
        DeclaredJitoKeys, JitoIdentityAccountInfos, JitoIdentityError},
    squads_execution::{SquadsBootstrapRoles, SquadsExecutionError},
};

/// Trusted handler role mapping, not an instruction-selected authority profile.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct GenesisProtocolRoles {
    pub program: usize, pub pool: usize, pub validator_list: usize, pub reserve: usize,
    pub mint: usize, pub manager_fee: usize, pub referrer: usize,
}
impl GenesisProtocolRoles {
    fn indices(self) -> [usize; 7] {
        [self.program, self.pool, self.validator_list, self.reserve, self.mint, self.manager_fee, self.referrer]
    }
}
#[derive(Clone, Copy, Debug)]
pub struct GenesisPreflightRoles {
    pub bootstrap: SquadsBootstrapRoles,
    pub protocol: GenesisProtocolRoles,
    /// Same order as ApprovedGenesisModel::targets(); index zero reuses Config.
    pub targets: [usize; 16],
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum GenesisPreflightError {
    Model(GenesisModelError),
    Protocol(JitoIdentityError),
    State(Piv1Error),
    InvalidRoles,
}
impl From<GenesisModelError> for GenesisPreflightError {
    fn from(error: GenesisModelError) -> Self { Self::Model(error) }
}
impl From<JitoIdentityError> for GenesisPreflightError {
    fn from(error: JitoIdentityError) -> Self { Self::Protocol(error) }
}
impl From<Piv1Error> for GenesisPreflightError {
    fn from(error: Piv1Error) -> Self { Self::State(error) }
}
pub type GenesisPreflightResult<T> = Result<T, GenesisPreflightError>;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct GenesisTargetRentObservation {
    target: DerivedGenesisTarget,
    observed_lamports: u64,
    rent_minimum: u64,
    shortfall: u64,
}
impl GenesisTargetRentObservation {
    /// Intended future owner/size; current owner is System and current size zero.
    pub fn target(&self) -> DerivedGenesisTarget { self.target }
    pub fn observed_lamports(&self) -> u64 { self.observed_lamports }
    pub fn rent_minimum(&self) -> u64 { self.rent_minimum }
    pub fn shortfall(&self) -> u64 { self.shortfall }
}

/// Private-field, non-Clone point-invocation observation, not a creation or
/// persistence capability. No public constructor accepts detached prior proofs.
#[derive(Debug, PartialEq)]
pub struct GenesisAccountPreflight {
    model: ApprovedGenesisModel,
    protocol: AuthenticatedJitoIdentity,
    targets: Box<[GenesisTargetRentObservation; 16]>,
    total_rent_shortfall: u64,
}
impl GenesisAccountPreflight {
    pub fn model(&self) -> &ApprovedGenesisModel { &self.model }
    pub fn protocol(&self) -> &AuthenticatedJitoIdentity { &self.protocol }
    pub fn targets(&self) -> &[GenesisTargetRentObservation; 16] { &self.targets }
    /// Checked sum of rent shortfalls only; no prefunding or economic total.
    pub fn total_rent_shortfall(&self) -> u64 { self.total_rent_shortfall }
}

pub fn preflight_approved_genesis_accounts(
    program: &Pubkey, accounts: &[AccountInfo<'_>], instruction_data: &[u8], roles: GenesisPreflightRoles,
) -> GenesisPreflightResult<GenesisAccountPreflight> {
    dispatch(program, accounts, instruction_data, roles, cfg!(target_os = "solana"),
        get_stack_height, Clock::get, Rent::get)
}

/// Explicit modeled context, absent from the Solana build and native dispatch.
#[cfg(not(target_os = "solana"))]
pub fn preflight_approved_genesis_accounts_with_host_context(
    program: &Pubkey, accounts: &[AccountInfo<'_>], instruction_data: &[u8], roles: GenesisPreflightRoles,
    context: crate::squads_execution::ModeledSquadsInvocationContext,
) -> GenesisPreflightResult<GenesisAccountPreflight> {
    dispatch(program, accounts, instruction_data, roles, true,
        || context.stack_height, || Ok(context.clock), || Ok(context.rent))
}

fn dispatch(
    program: &Pubkey, accounts: &[AccountInfo<'_>], instruction_data: &[u8], roles: GenesisPreflightRoles,
    available: bool, stack_height: impl FnOnce() -> usize,
    clock: impl FnOnce() -> Result<Clock, ProgramError>, rent: impl FnOnce() -> Result<Rent, ProgramError>,
) -> GenesisPreflightResult<GenesisAccountPreflight> {
    if !available { return Err(GenesisModelError::Authorization(SquadsExecutionError::HostRuntimeUnavailable).into()); }
    // The existing model dispatcher acquires stack/Clock/Rent exactly once and
    // preserves its decode/authentication ordering. Retain that same Rent value
    // for target observations, without a second runtime read or public injection.
    let mut observed_rent = None;
    let model = genesis_model::dispatch(program, accounts, instruction_data, roles.bootstrap, true,
        stack_height, clock, || {
            let value = rent()?;
            observed_rent = Some(value.clone());
            Ok(value)
        })?;
    let rent = observed_rent.ok_or(Piv1Error::InvalidRent)?;
    validate_roles(accounts.len(), roles)?;
    let config = model.proposed_config();
    let declared = DeclaredJitoKeys {
        program: config.stake_pool_program, pool: config.stake_pool,
        validator_list: config.validator_list, reserve: config.reserve_stake,
        mint: config.jitosol_mint, manager_fee: config.manager_fee_account, referrer: config.referrer_token_account,
    };
    let p = roles.protocol;
    let protocol = authenticate_jito_identity(&declared, JitoIdentityAccountInfos {
        program: &accounts[p.program], pool: &accounts[p.pool], validator_list: &accounts[p.validator_list],
        reserve: &accounts[p.reserve], mint: &accounts[p.mint], manager_fee: &accounts[p.manager_fee], referrer: &accounts[p.referrer],
    })?;
    let (targets, total_rent_shortfall) = observe_targets(accounts, &model, roles.targets, &rent)?;
    Ok(GenesisAccountPreflight { model, protocol, targets, total_rent_shortfall })
}

fn validate_roles(account_count: usize, roles: GenesisPreflightRoles) -> GenesisPreflightResult<()> {
    let b = roles.bootstrap;
    let bootstrap = [b.program, b.program_data, b.multisig, b.proposal, b.transaction, b.vault, b.instructions, b.config];
    let protocol = roles.protocol.indices();
    if roles.targets[0] != b.config { return Err(GenesisPreflightError::InvalidRoles); }
    for (index, target) in roles.targets.iter().enumerate() {
        if *target >= account_count || roles.targets[..index].contains(target)
            || (index != 0 && bootstrap.contains(target))
        { return Err(GenesisPreflightError::InvalidRoles); }
    }
    for (index, role) in protocol.iter().enumerate() {
        if *role >= account_count || bootstrap.contains(role) || roles.targets.contains(role) {
            return Err(GenesisPreflightError::InvalidRoles);
        }
        for previous in 0..index {
            if protocol[previous] == *role && !(previous == 5 && index == 6) {
                return Err(GenesisPreflightError::InvalidRoles);
            }
        }
    }
    Ok(())
}

#[inline(never)]
fn observe_targets(
    accounts: &[AccountInfo<'_>], model: &ApprovedGenesisModel, roles: [usize; 16], rent: &Rent,
) -> GenesisPreflightResult<(Box<[GenesisTargetRentObservation; 16]>, u64)> {
    let empty = GenesisTargetRentObservation { target: model.targets()[0], observed_lamports: 0, rent_minimum: 0, shortfall: 0 };
    let mut result = Box::new([empty; 16]);
    let mut total = 0_u64;
    for (slot, index) in roles.iter().copied().enumerate() {
        let account = &accounts[index];
        let target = model.targets()[slot];
        if *account.key != target.address() { return Err(Piv1Error::InvalidAccountPda.into()); }
        // Differently keyed synthetic AccountInfos must not share backing. This
        // prevents counting one store twice; same-index semantic sharing remains valid.
        for (other_index, other) in accounts.iter().enumerate() {
            if other_index != index && (Rc::ptr_eq(&account.data, &other.data)
                || Rc::ptr_eq(&account.lamports, &other.lamports))
            { return Err(Piv1Error::AccountAlias.into()); }
        }
        if account.owner != &system_program::ID { return Err(Piv1Error::InvalidAccountOwner.into()); }
        if account.executable { return Err(Piv1Error::ExecutableAccount.into()); }
        if !account.is_writable { return Err(Piv1Error::AccountNotWritable.into()); }
        if !account.try_borrow_data().map_err(|_| Piv1Error::AccountBorrowFailed)?.is_empty() {
            return Err(Piv1Error::InvalidAccountSize.into());
        }
        let observed_lamports = **account.try_borrow_lamports().map_err(|_| Piv1Error::AccountBorrowFailed)?;
        let rent_minimum = rent_floor(rent, target.size())?;
        let shortfall = if observed_lamports < rent_minimum {
            rent_minimum.checked_sub(observed_lamports).ok_or(Piv1Error::ArithmeticOverflow)?
        } else { 0 };
        total = total.checked_add(shortfall).ok_or(Piv1Error::ArithmeticOverflow)?;
        result[slot] = GenesisTargetRentObservation { target, observed_lamports, rent_minimum, shortfall };
    }
    Ok((result, total))
}
