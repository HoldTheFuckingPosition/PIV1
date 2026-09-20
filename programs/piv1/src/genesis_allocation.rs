//! Prefunding-safe genesis allocation, deliberately not an initializer.
//!
//! Fresh approval and preflight precede a rent-only batch paid by a distinct
//! signing System account. The public allocation API preserves all prefunds;
//! same-call initialization can privately normalize the two future Token targets.
//! No Token initialization, state serialization
//! or native instruction dispatch is supplied. A future caller MUST complete
//! those operations in the SAME atomic transaction and propagate every error.
//! Even successful allocation is an intermediate, never initialized PIV1 state.
//! Committing bare Token-owned zero data permits initialization takeover; never
//! expose this batch as a stand-alone handler or durable resumable checkpoint.
//! CPI/postcheck errors may leave partial host effects; rollback belongs to the
//! Solana transaction boundary, not this library function.

use std::rc::Rc;
use anchor_lang::{
    prelude::{AccountInfo, Clock, Pubkey, Rent, SolanaSysvar},
    solana_program::{entrypoint::ProgramResult, instruction::{get_stack_height, Instruction},
        program::invoke_signed, program_error::ProgramError, system_instruction, system_program},
};
use crate::{
    accounts::{rent_floor, seeds},
    errors::Piv1Error,
    genesis_model::{ApprovedGenesisModel, GenesisTargetRole},
    genesis_preflight::{self, GenesisAccountPreflight, GenesisPreflightError, GenesisPreflightRoles},
    genesis_recipients::{self, GenesisRecipientError, GenesisRecipientResult, GenesisRecipientSelection, RecipientExecutionObservation},
    guardian_clock_accounts::GUARDIAN_REGISTRY_SEED,
    kif_claim_accounts::GUARDIAN_REWARD_SEED,
};

/// Trusted handler roles; all actual accounts are in the exact approved message.
#[derive(Clone, Copy, Debug)]
pub struct GenesisAllocationRoles {
    pub preflight: GenesisPreflightRoles,
    /// Distinct external signing System account, never an existing PIV1 category.
    pub payer: usize,
    pub system_program: usize,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum GenesisAllocationError {
    Preflight(GenesisPreflightError),
    State(Piv1Error),
    Invocation(ProgramError),
    HostRuntimeUnavailable,
    InvalidRoles,
    InvalidPayer,
    InsufficientPayerRent,
    ObservationMismatch,
    Recipient(GenesisRecipientError),
}
impl From<GenesisPreflightError> for GenesisAllocationError {
    fn from(error: GenesisPreflightError) -> Self { Self::Preflight(error) }
}
impl From<Piv1Error> for GenesisAllocationError {
    fn from(error: Piv1Error) -> Self { Self::State(error) }
}
pub type GenesisAllocationResult<T> = Result<T, GenesisAllocationError>;

#[derive(Clone, Copy, Eq, PartialEq)]
pub(crate) enum TokenPrefundMode { Preserve, Normalize }

pub(crate) struct RecipientAllocationProfile {
    pub selection: GenesisRecipientSelection,
    pub token_program: usize,
}

/// Allocation-only intermediate. It has no public constructor or Clone and is
/// NOT a persisted initialization receipt, economic snapshot or replay permit.
#[must_use = "Allocation must be followed by Token initialization and all state writes in the same transaction"]
#[derive(Debug, PartialEq)]
pub struct AllocatedGenesisAccounts {
    preflight: GenesisAccountPreflight,
    payer: Pubkey,
    payer_before: u64,
    payer_after: u64,
    final_balances: [u64; 16],
    normalized_token_prefunds: u64,
    recipients: Option<RecipientExecutionObservation>,
}
impl AllocatedGenesisAccounts {
    /// Proposed zero-history state, still NOT serialized into the accounts.
    pub fn model(&self) -> &ApprovedGenesisModel { self.preflight.model() }
    /// Historical observations from immediately before this batch, not current state.
    pub fn before(&self) -> &GenesisAccountPreflight { &self.preflight }
    pub fn payer(&self) -> Pubkey { self.payer }
    pub fn payer_before(&self) -> u64 { self.payer_before }
    pub fn payer_after(&self) -> u64 { self.payer_after }
    pub fn funded_rent_lamports(&self) -> u64 { self.preflight.total_rent_shortfall() }
    /// Native custody moved to PendingSol; ledger recognition remains separate.
    pub fn normalized_token_prefund_lamports(&self) -> u64 { self.normalized_token_prefunds }
    pub(crate) fn final_balances(&self) -> &[u64; 16] { &self.final_balances }
    pub(crate) fn verify_recipients(&self, accounts: &[AccountInfo<'_>]) -> GenesisRecipientResult<()> {
        if let Some(observed) = &self.recipients { observed.verify(accounts)?; }
        Ok(())
    }
}

/// Actual System CPI composition on Solana; ordinary host calls reject before
/// account access. There is intentionally no reachable native selector.
pub fn allocate_approved_genesis_accounts(
    program: &Pubkey, accounts: &[AccountInfo<'_>], data: &[u8], roles: GenesisAllocationRoles,
) -> GenesisAllocationResult<AllocatedGenesisAccounts> {
    if !cfg!(target_os = "solana") { return Err(GenesisAllocationError::HostRuntimeUnavailable); }
    execute(program, accounts, data, roles, TokenPrefundMode::Preserve, None, get_stack_height, Clock::get, Rent::get,
        |instruction, infos, signers| invoke_signed(instruction, infos, signers))
}

/// Explicit synthetic context/CPI seam, absent from Solana and native dispatch.
/// The invoker models effects only; this API does not simulate rollback.
#[cfg(not(target_os = "solana"))]
pub fn allocate_approved_genesis_accounts_with_host_invoker<'info>(
    program: &Pubkey, accounts: &[AccountInfo<'info>], data: &[u8], roles: GenesisAllocationRoles,
    context: crate::squads_execution::ModeledSquadsInvocationContext,
    invoker: impl FnMut(&Instruction, &[AccountInfo<'info>], &[&[&[u8]]]) -> ProgramResult,
) -> GenesisAllocationResult<AllocatedGenesisAccounts> {
    execute(program, accounts, data, roles, TokenPrefundMode::Preserve, None, || context.stack_height,
        || Ok(context.clock), || Ok(context.rent), invoker)
}

struct TargetPlan {
    seeds: Vec<Vec<u8>>,
    transfer: Option<Instruction>,
    normalization: Option<(Instruction, u64)>,
    allocation: Option<(Instruction, Instruction)>,
}
struct AllocationPlan {
    targets: Vec<TargetPlan>,
    final_balances: [u64; 16],
    normalized_token_prefunds: u64,
}
#[derive(Clone, Copy)]
struct ExpectedTarget { balance: u64, owner: Pubkey, size: usize }

#[inline(never)]
#[allow(clippy::too_many_arguments)]
// Internal same-call completion shares the original trusted runtime reads.
pub(crate) fn execute<'info>(
    program: &Pubkey, accounts: &[AccountInfo<'info>], data: &[u8], roles: GenesisAllocationRoles,
    mode: TokenPrefundMode,
    recipient_profile: Option<RecipientAllocationProfile>,
    height: impl FnOnce() -> usize, clock: impl FnOnce() -> Result<Clock, ProgramError>,
    rent: impl FnOnce() -> Result<Rent, ProgramError>,
    mut invoke: impl FnMut(&Instruction, &[AccountInfo<'info>], &[&[&[u8]]]) -> ProgramResult,
) -> GenesisAllocationResult<AllocatedGenesisAccounts> {
    let mut runtime_rent = None;
    let preflight = genesis_preflight::dispatch(program, accounts, data, roles.preflight,
        true, height, clock, || { let r = rent()?; runtime_rent = Some(r.clone()); Ok(r) })?;
    let rent = runtime_rent.ok_or(Piv1Error::InvalidRent)?;
    let payer_before = validate_payer(accounts, &preflight, roles, &rent)?;
    let payer_after = payer_before.checked_sub(preflight.total_rent_shortfall())
        .ok_or(GenesisAllocationError::InsufficientPayerRent)?;
    if payer_after < rent_floor(&rent, 0)? { return Err(GenesisAllocationError::InsufficientPayerRent); }
    let recipients = recipient_profile.map(|profile| genesis_recipients::observe_for_execution(
        accounts, &preflight, roles.preflight, profile.selection,
        [roles.payer, roles.system_program, profile.token_program], &rent))
        .transpose().map_err(GenesisAllocationError::Recipient)?;
    let verify_recipients = || -> GenesisAllocationResult<()> {
        if let Some(observed) = &recipients { observed.verify(accounts).map_err(GenesisAllocationError::Recipient)?; }
        Ok(())
    };
    let payer = &accounts[roles.payer];
    let system = &accounts[roles.system_program];
    let plan = build_plans(program, &preflight, payer.key, mode)?;
    // All mutable borrows are checked together before the first effect. No
    // guard is retained across CPI. Target aliases were rejected by preflight.
    {
        let mut data_guards = Vec::with_capacity(17);
        let mut balance_guards = Vec::with_capacity(17);
        for index in roles.preflight.targets.into_iter().chain([roles.payer]) {
            data_guards.push(accounts[index].try_borrow_mut_data().map_err(|_| Piv1Error::AccountBorrowFailed)?);
            balance_guards.push(accounts[index].try_borrow_mut_lamports().map_err(|_| Piv1Error::AccountBorrowFailed)?);
        }
    }
    let mut expected = preflight.targets().map(|t| ExpectedTarget {
        balance: t.observed_lamports(), owner: system_program::ID, size: 0,
    });
    let mut expected_payer = payer_before;
    for (slot, target_plan) in plan.targets.iter().enumerate() {
        let target = &accounts[roles.preflight.targets[slot]];
        let observation = preflight.targets()[slot];
        if let Some(instruction) = &target_plan.transfer {
            invoke(instruction, &[payer.clone(), target.clone(), system.clone()], &[])
                .map_err(GenesisAllocationError::Invocation)?;
            verify_recipients()?;
            expected_payer = expected_payer.checked_sub(observation.shortfall()).ok_or(Piv1Error::ArithmeticOverflow)?;
            expected[slot].balance = expected[slot].balance.checked_add(observation.shortfall()).ok_or(Piv1Error::ArithmeticOverflow)?;
            verify_effects(accounts, roles, &preflight, &expected, expected_payer)?;
        }
        if let Some((instruction, amount)) = &target_plan.normalization {
            // PendingSol's ORIGINAL shortfall was funded at slot 9. A received
            // contribution cannot substitute for any of that external rent.
            // Check the fresh source is still empty/System-owned before CPI.
            verify_effects(accounts, roles, &preflight, &expected, expected_payer)?;
            if *target.owner != system_program::ID || !target.try_borrow_data()
                .map_err(|_| Piv1Error::AccountBorrowFailed)?.is_empty()
            { return Err(GenesisAllocationError::ObservationMismatch); }
            let seed_refs: Vec<&[u8]> = target_plan.seeds.iter().map(Vec::as_slice).collect();
            let pending = &accounts[roles.preflight.targets[9]];
            invoke(instruction, &[target.clone(), pending.clone(), system.clone()], &[&seed_refs])
                .map_err(GenesisAllocationError::Invocation)?;
            verify_recipients()?;
            expected[slot].balance = expected[slot].balance.checked_sub(*amount).ok_or(Piv1Error::ArithmeticOverflow)?;
            expected[9].balance = expected[9].balance.checked_add(*amount).ok_or(Piv1Error::ArithmeticOverflow)?;
            verify_effects(accounts, roles, &preflight, &expected, expected_payer)?;
        }
        if let Some((allocate, assign)) = &target_plan.allocation {
            let seed_refs: Vec<&[u8]> = target_plan.seeds.iter().map(Vec::as_slice).collect();
            let infos = [target.clone(), system.clone()];
            invoke(allocate, &infos, &[&seed_refs]).map_err(GenesisAllocationError::Invocation)?;
            verify_recipients()?;
            expected[slot].size = observation.target().size();
            verify_effects(accounts, roles, &preflight, &expected, expected_payer)?;
            invoke(assign, &infos, &[&seed_refs]).map_err(GenesisAllocationError::Invocation)?;
            verify_recipients()?;
            expected[slot].owner = observation.target().owner();
            verify_effects(accounts, roles, &preflight, &expected, expected_payer)?;
        }
    }
    if expected_payer != payer_after || expected.map(|t| t.balance) != plan.final_balances {
        return Err(GenesisAllocationError::ObservationMismatch);
    }
    verify_effects(accounts, roles, &preflight, &expected, payer_after)?;
    verify_recipients()?;
    Ok(AllocatedGenesisAccounts { preflight, payer: *payer.key, payer_before, payer_after,
        final_balances: plan.final_balances, normalized_token_prefunds: plan.normalized_token_prefunds, recipients })
}

fn validate_payer(
    accounts: &[AccountInfo<'_>], preflight: &GenesisAccountPreflight,
    roles: GenesisAllocationRoles, rent: &Rent,
) -> GenesisAllocationResult<u64> {
    let b = roles.preflight.bootstrap; let p = roles.preflight.protocol;
    let protected = [b.program, b.program_data, b.multisig, b.proposal, b.transaction, b.vault,
        b.instructions, b.config, p.program, p.pool, p.validator_list, p.reserve, p.mint, p.manager_fee, p.referrer];
    for index in [roles.payer, roles.system_program] {
        if index >= accounts.len() || protected.contains(&index) || roles.preflight.targets.contains(&index) {
            return Err(GenesisAllocationError::InvalidRoles);
        }
    }
    if roles.payer == roles.system_program { return Err(GenesisAllocationError::InvalidRoles); }
    let payer = &accounts[roles.payer]; let system = &accounts[roles.system_program];
    if *system.key != system_program::ID || !system.executable { return Err(Piv1Error::InvalidProgramIdentity.into()); }
    let model = preflight.model(); let config = model.proposed_config();
    if *payer.key == model.piv_authority() || *payer.key == config.htfp_recipient
        || *payer.key == config.team_owner_recipient || model.proposed_registry().guardian_keys.contains(payer.key)
        || !payer.is_signer || !payer.is_writable || payer.executable || *payer.owner != system_program::ID
        || !payer.try_borrow_data().map_err(|_| Piv1Error::AccountBorrowFailed)?.is_empty()
    { return Err(GenesisAllocationError::InvalidPayer); }
    for (index, other) in accounts.iter().enumerate() {
        if index != roles.payer && (Rc::ptr_eq(&payer.data, &other.data) || Rc::ptr_eq(&payer.lamports, &other.lamports)) {
            return Err(Piv1Error::AccountAlias.into());
        }
    }
    let balance = lamports(payer)?;
    if balance < rent_floor(rent, 0)? { return Err(GenesisAllocationError::InsufficientPayerRent); }
    Ok(balance)
}

#[inline(never)]
fn build_plans(program: &Pubkey, preflight: &GenesisAccountPreflight, payer: &Pubkey, mode: TokenPrefundMode)
    -> GenesisAllocationResult<AllocationPlan>
{
    use GenesisTargetRole as R;
    let mut plans = Vec::with_capacity(16);
    let mut final_balances = [0; 16];
    let mut normalized_token_prefunds = 0_u64;
    let pending = preflight.model().targets()[9].address();
    for (slot, observation) in preflight.targets().iter().enumerate() {
        let target = observation.target();
        let mut seed_bytes = match target.role() {
            R::GuardianReward(slot) => vec![GUARDIAN_REWARD_SEED.to_vec(),
                preflight.model().proposed_registry().guardian_keys[usize::from(slot)].to_bytes().to_vec(),
                0_u64.to_le_bytes().to_vec(), vec![slot]],
            role => vec![match role {
                R::Config => seeds::CONFIG, R::ActiveDistribution => seeds::DISTRIBUTION,
                R::GuardianRegistry => GUARDIAN_REGISTRY_SEED, R::PendingSol => seeds::PENDING_SOL,
                R::PrincipalSol => seeds::PRINCIPAL_SOL, R::OperationalSol => seeds::OPERATIONAL_SOL,
                R::DistributionEscrow => seeds::DISTRIBUTION_ESCROW, R::KifSol => seeds::KIF_SOL,
                R::PrincipalJito => seeds::PRINCIPAL_JITO, R::PendingJito => seeds::PENDING_JITO,
                R::GuardianReward(_) => unreachable!(),
            }.to_vec()],
        };
        seed_bytes.push(vec![target.bump()]);
        let refs: Vec<&[u8]> = seed_bytes.iter().map(Vec::as_slice).collect();
        if Pubkey::create_program_address(&refs, program).map_err(|_| Piv1Error::InvalidAccountPda)? != target.address() {
            return Err(Piv1Error::InvalidAccountPda.into());
        }
        // All outgoing values and bounded allocations are prepared before CPI.
        let funded = observation.observed_lamports().checked_add(observation.shortfall()).ok_or(Piv1Error::ArithmeticOverflow)?;
        final_balances[slot] = funded;
        let normalization = if mode == TokenPrefundMode::Normalize && matches!(target.role(), R::PrincipalJito | R::PendingJito)
            && observation.observed_lamports() > observation.rent_minimum()
        {
            let excess = observation.observed_lamports().checked_sub(observation.rent_minimum()).ok_or(Piv1Error::ArithmeticOverflow)?;
            normalized_token_prefunds = normalized_token_prefunds.checked_add(excess).ok_or(Piv1Error::ArithmeticOverflow)?;
            final_balances[slot] = funded.checked_sub(excess).ok_or(Piv1Error::ArithmeticOverflow)?;
            Some((system_instruction::transfer(&target.address(), &pending, excess), excess))
        } else { None };
        let transfer = (observation.shortfall() != 0).then(|| system_instruction::transfer(payer, &target.address(), observation.shortfall()));
        let allocation = if target.size() == 0 { None } else {
            let size = u64::try_from(target.size()).map_err(|_| Piv1Error::ArithmeticOverflow)?;
            Some((system_instruction::allocate(&target.address(), size), system_instruction::assign(&target.address(), &target.owner())))
        };
        plans.push(TargetPlan { seeds: seed_bytes, transfer, normalization, allocation });
    }
    // Validate the complete destination balance before ANY funding or sweep;
    // include PendingSol's original rent shortfall, never recompute it afterwards.
    final_balances[9] = final_balances[9].checked_add(normalized_token_prefunds).ok_or(Piv1Error::ArithmeticOverflow)?;
    Ok(AllocationPlan { targets: plans, final_balances, normalized_token_prefunds })
}

#[inline(never)]
fn verify_effects(
    accounts: &[AccountInfo<'_>], roles: GenesisAllocationRoles, before: &GenesisAccountPreflight,
    expected: &[ExpectedTarget; 16], payer_balance: u64,
) -> GenesisAllocationResult<()> {
    let payer = &accounts[roles.payer];
    if *payer.owner != system_program::ID || payer.executable || !payer.is_signer || !payer.is_writable
        || lamports(payer)? != payer_balance || !payer.try_borrow_data().map_err(|_| Piv1Error::AccountBorrowFailed)?.is_empty()
    { return Err(GenesisAllocationError::ObservationMismatch); }
    for (slot, index) in roles.preflight.targets.iter().copied().enumerate() {
        let account = &accounts[index]; let expected = expected[slot];
        let data = account.try_borrow_data().map_err(|_| Piv1Error::AccountBorrowFailed)?;
        if *account.key != before.targets()[slot].target().address() || !account.is_writable || account.executable
            || *account.owner != expected.owner || lamports(account)? != expected.balance
            || data.len() != expected.size || data.iter().any(|byte| *byte != 0)
        { return Err(GenesisAllocationError::ObservationMismatch); }
    }
    Ok(())
}

fn lamports(account: &AccountInfo<'_>) -> Result<u64, Piv1Error> {
    Ok(**account.try_borrow_lamports().map_err(|_| Piv1Error::AccountBorrowFailed)?)
}
