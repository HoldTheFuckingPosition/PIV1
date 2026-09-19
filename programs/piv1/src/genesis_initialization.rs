//! Same-call genesis allocation, Token initialization and nine typed state writes.
//!
//! This library has no native selector. Recipient control, prefund normalization,
//! operational funding provenance, transport and runtime resource/rollback proof
//! remain prerequisites to exposing an initializer. Every error MUST propagate
//! to the outer transaction: this function cannot undo CPI effects. A successful
//! result confirms account bytes at this invocation, not economic readiness.

use std::{cell::RefMut, rc::Rc};
use anchor_lang::{
    prelude::{AccountInfo, Clock, Pubkey, Rent, SolanaSysvar},
    solana_program::{entrypoint::ProgramResult, instruction::{get_stack_height, Instruction},
        program::invoke_signed, program_error::ProgramError, program_option::COption,
        program_pack::Pack, system_program},
};
use spl_token::state::{Account as TokenAccount, AccountState, Mint};
use crate::{
    accounts::{authenticate_fixed_accounts, rent_floor, FixedAccountInfos},
    errors::Piv1Error,
    genesis_allocation::{self, AllocatedGenesisAccounts, GenesisAllocationError, GenesisAllocationRoles},
    genesis_model::ApprovedGenesisModel,
    state_persistence::StateEnvelope,
};

#[derive(Clone, Copy, Debug)]
pub struct GenesisInitializationRoles {
    pub allocation: GenesisAllocationRoles,
    /// Canonical executable legacy Token Program in the same approved message.
    pub token_program: usize,
}
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum GenesisInitializationError {
    Allocation(GenesisAllocationError),
    State(Piv1Error),
    Invocation(ProgramError),
    HostRuntimeUnavailable,
    InvalidRoles,
    ObservationMismatch,
}
impl From<GenesisAllocationError> for GenesisInitializationError {
    fn from(error: GenesisAllocationError) -> Self { Self::Allocation(error) }
}
impl From<Piv1Error> for GenesisInitializationError {
    fn from(error: Piv1Error) -> Self { Self::State(error) }
}
pub type GenesisInitializationResult<T> = Result<T, GenesisInitializationError>;

/// Private, non-Clone point-invocation completion facts, not a durable replay
/// receipt or permission to bypass recipient/economic/transport readiness gates.
#[must_use]
#[derive(Debug, PartialEq)]
pub struct InitializedGenesisAccounts { allocation: AllocatedGenesisAccounts }
impl InitializedGenesisAccounts {
    pub fn model(&self) -> &ApprovedGenesisModel { self.allocation.model() }
    /// Historical funding/pre-allocation observations, not a reusable capability.
    pub fn allocation(&self) -> &AllocatedGenesisAccounts { &self.allocation }
}

/// Fresh exact approval and allocation followed by both Token CPIs and all state
/// writes in one call. No detached allocation object can enter this boundary.
pub fn initialize_approved_genesis_accounts(
    program: &Pubkey, accounts: &[AccountInfo<'_>], data: &[u8], roles: GenesisInitializationRoles,
) -> GenesisInitializationResult<InitializedGenesisAccounts> {
    if !cfg!(target_os = "solana") { return Err(GenesisInitializationError::HostRuntimeUnavailable); }
    execute(program, accounts, data, roles, get_stack_height, Clock::get, Rent::get,
        |instruction, infos, seeds| invoke_signed(instruction, infos, seeds))
}

/// Explicit modeled runtime/CPI seam, absent from Solana and native dispatch.
/// Effects are not automatically rolled back if this function returns an error.
#[cfg(not(target_os = "solana"))]
pub fn initialize_approved_genesis_accounts_with_host_invoker<'info>(
    program: &Pubkey, accounts: &[AccountInfo<'info>], data: &[u8], roles: GenesisInitializationRoles,
    context: crate::squads_execution::ModeledSquadsInvocationContext,
    invoke: impl FnMut(&Instruction, &[AccountInfo<'info>], &[&[&[u8]]]) -> ProgramResult,
) -> GenesisInitializationResult<InitializedGenesisAccounts> {
    execute(program, accounts, data, roles, || context.stack_height,
        || Ok(context.clock), || Ok(context.rent), invoke)
}

struct MintObservation { owner: Pubkey, lamports: u64, bytes: Vec<u8> }

#[inline(never)]
fn execute<'info>(
    program: &Pubkey, accounts: &[AccountInfo<'info>], data: &[u8], roles: GenesisInitializationRoles,
    height: impl FnOnce() -> usize, clock: impl FnOnce() -> Result<Clock, ProgramError>,
    rent: impl FnOnce() -> Result<Rent, ProgramError>,
    mut invoke: impl FnMut(&Instruction, &[AccountInfo<'info>], &[&[&[u8]]]) -> ProgramResult,
) -> GenesisInitializationResult<InitializedGenesisAccounts> {
    let mint_before = validate_completion_roles(accounts, roles)?;
    let mut trusted_rent = None;
    let allocation = genesis_allocation::execute(program, accounts, data, roles.allocation,
        height, clock, || { let value = rent()?; trusted_rent = Some(value.clone()); Ok(value) }, &mut invoke)?;
    let rent = trusted_rent.ok_or(Piv1Error::InvalidRent)?;
    let envelopes = envelopes(allocation.model())?;
    let config = allocation.model().proposed_config();
    let token_bytes = expected_token(config.jitosol_mint, config.piv_authority)?;
    verify_stage(accounts, roles, &allocation, &envelopes, &token_bytes, &mint_before, 0, false)?;
    let targets = roles.allocation.preflight.targets;
    let mint = &accounts[roles.allocation.preflight.protocol.mint];
    let token_program = &accounts[roles.token_program];
    for (index, slot) in [14,15].into_iter().enumerate() {
        let target = &accounts[targets[slot]];
        let instruction = spl_token::instruction::initialize_account3(&spl_token::ID,
            target.key, mint.key, &config.piv_authority).map_err(GenesisInitializationError::Invocation)?;
        invoke(&instruction, &[target.clone(), mint.clone(), token_program.clone()], &[])
            .map_err(GenesisInitializationError::Invocation)?;
        verify_stage(accounts, roles, &allocation, &envelopes, &token_bytes, &mint_before, index+1, false)?;
    }
    write_initial_envelopes(program, accounts, targets, &allocation, &rent, &envelopes)?;
    verify_stage(accounts, roles, &allocation, &envelopes, &token_bytes, &mint_before, 2, true)?;
    verify_fixed_accounts(program, accounts, targets, &rent, allocation.model())?;
    Ok(InitializedGenesisAccounts { allocation })
}

fn validate_completion_roles(accounts: &[AccountInfo<'_>], roles: GenesisInitializationRoles)
    -> GenesisInitializationResult<MintObservation>
{
    let a = roles.allocation; let b = a.preflight.bootstrap; let p = a.preflight.protocol;
    let protected = [a.payer,a.system_program,b.program,b.program_data,b.multisig,b.proposal,b.transaction,
        b.vault,b.instructions,b.config,p.program,p.pool,p.validator_list,p.reserve,p.mint,p.manager_fee,p.referrer];
    if roles.token_program >= accounts.len() || p.mint >= accounts.len()
        || protected.contains(&roles.token_program) || a.preflight.targets.contains(&roles.token_program)
    { return Err(GenesisInitializationError::InvalidRoles); }
    let token = &accounts[roles.token_program];
    if *token.key != spl_token::ID || !token.executable { return Err(Piv1Error::InvalidProgramIdentity.into()); }
    for (index, other) in accounts.iter().enumerate() {
        if index != roles.token_program && (Rc::ptr_eq(&token.data, &other.data) || Rc::ptr_eq(&token.lamports, &other.lamports)) {
            return Err(Piv1Error::AccountAlias.into());
        }
    }
    let mint = &accounts[p.mint];
    // Pinned Token initialization borrows mint data mutably even though it never
    // writes it. Check availability before allocation and retain no guard for CPI.
    let bytes = mint.try_borrow_mut_data().map_err(|_| Piv1Error::AccountBorrowFailed)?;
    if bytes.len() != Mint::LEN { return Err(Piv1Error::InvalidAccountSize.into()); }
    Ok(MintObservation { owner: *mint.owner, lamports: lamports(mint)?, bytes: bytes.to_vec() })
}

#[inline(never)]
fn envelopes(model: &ApprovedGenesisModel) -> Result<[StateEnvelope;9], Piv1Error> {
    let rewards = model.proposed_rewards();
    Ok([StateEnvelope::config(model.proposed_config())?, StateEnvelope::distribution(model.proposed_distribution())?,
        StateEnvelope::registry(model.proposed_registry())?, StateEnvelope::reward(&rewards[0])?,
        StateEnvelope::reward(&rewards[1])?, StateEnvelope::reward(&rewards[2])?, StateEnvelope::reward(&rewards[3])?,
        StateEnvelope::reward(&rewards[4])?, StateEnvelope::reward(&rewards[5])?])
}

fn expected_token(mint: Pubkey, authority: Pubkey) -> Result<[u8;165], Piv1Error> {
    let mut bytes = [0;165];
    TokenAccount::pack(TokenAccount { mint, owner: authority, amount: 0, delegate: COption::None,
        state: AccountState::Initialized, is_native: COption::None, delegated_amount: 0,
        close_authority: COption::None }, &mut bytes).map_err(|_| Piv1Error::InvalidTokenCustody)?;
    Ok(bytes)
}

#[inline(never)]
fn write_initial_envelopes(
    program: &Pubkey, accounts: &[AccountInfo<'_>], targets: [usize;16], allocation: &AllocatedGenesisAccounts,
    rent: &Rent, envelopes: &[StateEnvelope;9],
) -> GenesisInitializationResult<()> {
    let mut guards: [Option<RefMut<'_, &mut [u8]>>;9] = core::array::from_fn(|_| None);
    for slot in 0..9 {
        let account = &accounts[targets[slot]]; let target = allocation.model().targets()[slot];
        if *account.key != target.address() { return Err(Piv1Error::InvalidAccountPda.into()); }
        if account.owner != program { return Err(Piv1Error::InvalidAccountOwner.into()); }
        if !account.is_writable { return Err(Piv1Error::AccountNotWritable.into()); }
        if account.executable { return Err(Piv1Error::ExecutableAccount.into()); }
        if lamports(account)? < rent_floor(rent,target.size())? { return Err(Piv1Error::AccountRentDeficit.into()); }
        let data = account.try_borrow_mut_data().map_err(|_| Piv1Error::AccountBorrowFailed)?;
        if data.len() != target.size() || data.len() != envelopes[slot].as_bytes().len() {
            return Err(Piv1Error::InvalidAccountSize.into());
        }
        if data.iter().any(|byte| *byte != 0) { return Err(Piv1Error::StateEnvelopeChanged.into()); }
        guards[slot] = Some(data);
    }
    // Every guard, key/owner/rent/zero check and serialization succeeded. Nothing
    // fallible or allocating follows the first byte copy in this private batch.
    for (data,envelope) in guards.iter_mut().flatten().zip(envelopes) { data.copy_from_slice(envelope.as_bytes()); }
    Ok(())
}

#[allow(clippy::too_many_arguments)]
#[inline(never)]
fn verify_stage(
    accounts: &[AccountInfo<'_>], roles: GenesisInitializationRoles, allocation: &AllocatedGenesisAccounts,
    envelopes: &[StateEnvelope;9], token_bytes: &[u8;165], mint_before: &MintObservation,
    initialized_tokens: usize, states_written: bool,
) -> GenesisInitializationResult<()> {
    let payer = &accounts[roles.allocation.payer];
    if *payer.key != allocation.payer() || *payer.owner != system_program::ID || payer.executable
        || !payer.is_signer || !payer.is_writable || lamports(payer)? != allocation.payer_after()
        || !payer.try_borrow_data().map_err(|_| Piv1Error::AccountBorrowFailed)?.is_empty()
    { return Err(GenesisInitializationError::ObservationMismatch); }
    let token_program = &accounts[roles.token_program];
    if *token_program.key != spl_token::ID || !token_program.executable { return Err(Piv1Error::InvalidProgramIdentity.into()); }
    let mint = &accounts[roles.allocation.preflight.protocol.mint];
    if *mint.key != allocation.model().proposed_config().jitosol_mint || *mint.owner != mint_before.owner
        || mint.executable || lamports(mint)? != mint_before.lamports
        || *mint.try_borrow_data().map_err(|_| Piv1Error::AccountBorrowFailed)? != mint_before.bytes.as_slice()
    { return Err(GenesisInitializationError::ObservationMismatch); }
    for (slot,index) in roles.allocation.preflight.targets.iter().copied().enumerate() {
        let account = &accounts[index]; let observed = allocation.before().targets()[slot]; let target = observed.target();
        let balance = observed.observed_lamports().checked_add(observed.shortfall()).ok_or(Piv1Error::ArithmeticOverflow)?;
        let data = account.try_borrow_data().map_err(|_| Piv1Error::AccountBorrowFailed)?;
        if *account.key != target.address() || *account.owner != target.owner() || !account.is_writable
            || account.executable || lamports(account)? != balance || data.len() != target.size()
        { return Err(GenesisInitializationError::ObservationMismatch); }
        let matches = if slot < 9 && states_written { *data == envelopes[slot].as_bytes() }
            else if slot >= 14 && slot - 14 < initialized_tokens { *data == token_bytes.as_slice() }
            else { data.iter().all(|byte| *byte == 0) };
        if !matches { return Err(GenesisInitializationError::ObservationMismatch); }
    }
    Ok(())
}

#[inline(never)]
fn verify_fixed_accounts(
    program: &Pubkey, accounts: &[AccountInfo<'_>], targets: [usize;16], rent: &Rent, model: &ApprovedGenesisModel,
) -> GenesisInitializationResult<()> {
    let a = |slot| &accounts[targets[slot]];
    let after = authenticate_fixed_accounts(program,rent,FixedAccountInfos {
        config:a(0),active_distribution:a(1),pending_sol:a(9),principal_sol:a(10),operational_sol:a(11),
        distribution_escrow:a(12),kif_sol:a(13),principal_jito:a(14),pending_jito:a(15),
    })?;
    if after.config() != model.proposed_config() || after.distribution() != model.proposed_distribution()
        || after.principal_jito().token_units != 0 || after.pending_jito().token_units != 0
    { return Err(GenesisInitializationError::ObservationMismatch); }
    // Do not call economic_observation: raw prefunds remain unclassified and
    // native excess in Token accounts is still unsupported by that accessor.
    Ok(())
}

fn lamports(account: &AccountInfo<'_>) -> Result<u64, Piv1Error> {
    Ok(**account.try_borrow_lamports().map_err(|_| Piv1Error::AccountBorrowFailed)?)
}
