//! Current signed guardian activity and exact-approved emergency pause changes.
//! Neither operation performs CPI, creates liability, moves rent/funds or edits
//! an active distribution snapshot. One typed state envelope is committed last.

use std::rc::Rc;
use anchor_lang::{prelude::{AccountInfo, Clock, Pubkey, Rent, SolanaSysvar},
    solana_program::{entrypoint::ProgramResult, instruction::get_stack_height, program_error::ProgramError}};
use crate::{
    accounts::{decode_state, CONFIG_DISCRIMINATOR},
    errors::{Piv1Error, Piv1Result},
    events::{GuardianHeartbeat, PauseChanged},
    guardian_clock_accounts::{authenticate_guardian_clock_snapshot, GuardianClockAccountInfos},
    instruction_errors::{piv1_error_code, squads_program_error, HOST_RUNTIME_UNAVAILABLE_CODE},
    instructions::{guardian_heartbeat::{GuardianHeartbeatRequest, GUARDIAN_HEARTBEAT_SELECTOR},
        pause::{SetPauseRequest, SET_PAUSE_SELECTOR}},
    squads_accounts::{authenticate_squads_authority_snapshot, SquadsAuthorityAccountInfos},
    squads_execution::{self, SquadsExecutionRoles},
    state::PivConfig,
    state_persistence::{commit_state_writes, PreparedStateWrite, StateEnvelope},
};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum GuardianOperationEvent { Heartbeat(GuardianHeartbeat), Pause(PauseChanged) }

struct RuntimeContext { stack_height: usize, clock: Clock, rent: Rent }
enum Action { Heartbeat(GuardianHeartbeatRequest), Pause(SetPauseRequest) }

/// Native path always obtains actual Clock/Rent. Host syscall stubs cannot make
/// this entry point succeed, and no instruction can supply a runtime context.
pub fn process_instruction(program: &Pubkey, accounts: &[AccountInfo<'_>], data: &[u8]) -> ProgramResult {
    dispatch(program, accounts, data, cfg!(target_os = "solana"), || Ok(RuntimeContext {
        stack_height: get_stack_height(), clock: Clock::get()?, rent: Rent::get()?,
    }), |event| match event {
        GuardianOperationEvent::Heartbeat(event) => anchor_lang::emit!(event),
        GuardianOperationEvent::Pause(event) => anchor_lang::emit!(event),
    })
}

/// Explicit host-only modeling seam; never linked as a Solana entrypoint.
/// Callbacks model trusted context/events, not live authority or runtime proof.
#[cfg(not(target_os = "solana"))]
pub fn process_instruction_with_host_callbacks(
    program: &Pubkey, accounts: &[AccountInfo<'_>], data: &[u8],
    get_context: impl FnOnce() -> Result<squads_execution::ModeledSquadsInvocationContext, ProgramError>,
    emit: impl FnOnce(GuardianOperationEvent),
) -> ProgramResult {
    dispatch(program, accounts, data, true, || get_context().map(|context| RuntimeContext {
        stack_height: context.stack_height, clock: context.clock, rent: context.rent,
    }), emit)
}

fn dispatch(
    program: &Pubkey, accounts: &[AccountInfo<'_>], data: &[u8], available: bool,
    get_context: impl FnOnce() -> Result<RuntimeContext, ProgramError>,
    emit: impl FnOnce(GuardianOperationEvent),
) -> ProgramResult {
    let (action, count) = if data.get(..8) == Some(GUARDIAN_HEARTBEAT_SELECTOR.as_slice()) {
        (Action::Heartbeat(GuardianHeartbeatRequest::decode(data)?), 13)
    } else if data.get(..8) == Some(SET_PAUSE_SELECTOR.as_slice()) {
        (Action::Pause(SetPauseRequest::decode(data)?), 16)
    } else { return Err(ProgramError::InvalidInstructionData); };
    if accounts.len() < count { return Err(ProgramError::NotEnoughAccountKeys); }
    if accounts.len() > count { return Err(ProgramError::InvalidArgument); }
    if !available { return Err(ProgramError::Custom(HOST_RUNTIME_UNAVAILABLE_CODE)); }
    let context = get_context()?;
    distinct_accounts(accounts).map_err(state_error)?;
    let event = match action {
        Action::Heartbeat(request) => heartbeat(program, accounts, request, &context).map_err(state_error)?,
        Action::Pause(request) => set_pause(program, accounts, data, request, &context)?,
    };
    emit(event); Ok(())
}

fn state_error(error: Piv1Error) -> ProgramError { ProgramError::Custom(piv1_error_code(error)) }

fn distinct_accounts(accounts: &[AccountInfo<'_>]) -> Piv1Result<()> {
    for (index, account) in accounts.iter().enumerate() {
        if accounts[index + 1..].iter().any(|other| account.key == other.key
            || Rc::ptr_eq(&account.data, &other.data) || Rc::ptr_eq(&account.lamports, &other.lamports)) {
            return Err(Piv1Error::AccountAlias);
        }
    }
    Ok(())
}

// Fixed heartbeat accounts: Config, registry, six slot-ordered rewards, Clock,
// guardian signer, runtime Program, ProgramData, current governance multisig.
#[inline(never)]
fn heartbeat(program: &Pubkey, accounts: &[AccountInfo<'_>], request: GuardianHeartbeatRequest,
    context: &RuntimeContext) -> Piv1Result<GuardianOperationEvent>
{
    let snapshot = authenticate_guardian_clock_snapshot(program, &context.rent, GuardianClockAccountInfos {
        config: &accounts[0], guardian_registry: &accounts[1],
        rewards: core::array::from_fn(|index| &accounts[2 + index]), clock: &accounts[8],
    })?;
    if snapshot.clock() != &context.clock { return Err(Piv1Error::InvalidClockAccount); }
    let authority = authenticate_squads_authority_snapshot(program, request.vault_index,
        SquadsAuthorityAccountInfos { program: &accounts[10], program_data: &accounts[11], multisig: &accounts[12] })?;
    authority.validate_guardian_correspondence(&snapshot)?;
    let index = usize::from(request.guardian_index);
    let before = snapshot.rewards()[index];
    if !accounts[9].is_signer { return Err(Piv1Error::MissingGuardianSignature); }
    if *accounts[9].key != before.guardian { return Err(Piv1Error::InvalidGuardianSet); }
    let mut after = before;
    after.record_activity(snapshot.registry(), request.guardian_index, snapshot.period().id)?;
    let target = &accounts[2 + index];
    let write = PreparedStateWrite::new(program, *target.key,
        StateEnvelope::reward(&before)?, StateEnvelope::reward(&after)?)?;
    let event = GuardianHeartbeat { guardian_registry: *accounts[1].key,
        guardian_reward: *target.key, guardian: before.guardian, registry_revision: before.registry_revision,
        guardian_index: request.guardian_index, period_id: snapshot.period().id };
    commit_state_writes(program, &context.rent, [(&write, target)])?;
    Ok(GuardianOperationEvent::Heartbeat(event))
}

// Fixed initialized Squads profile: Program, ProgramData, multisig, proposal,
// transaction, vault, Instructions, Config, registry, six rewards, Clock.
#[inline(never)]
fn set_pause(program: &Pubkey, accounts: &[AccountInfo<'_>], data: &[u8], request: SetPauseRequest,
    context: &RuntimeContext) -> Result<GuardianOperationEvent, ProgramError>
{
    let authority = squads_execution::dispatch(program, accounts, data, SquadsExecutionRoles {
        program: 0, program_data: 1, multisig: 2, proposal: 3, transaction: 4, vault: 5,
        instructions: 6, config: 7, guardian_registry: 8, rewards: [9, 10, 11, 12, 13, 14], clock: 15,
    }, request.vault_index, true, || context.stack_height,
        || Ok(context.clock.clone()), || Ok(context.rent.clone())).map_err(squads_program_error)?;
    let mut config: Box<PivConfig> = decode_state(&accounts[7], program, &context.rent,
        PivConfig::SPACE, CONFIG_DISCRIMINATOR).map_err(state_error)?;
    let before = StateEnvelope::config(&config).map_err(state_error)?;
    let previously_paused = config.paused; config.paused = request.paused;
    let after = StateEnvelope::config(&config).map_err(state_error)?;
    let write = PreparedStateWrite::new(program, *accounts[7].key, before, after).map_err(state_error)?;
    let event = PauseChanged { config: *accounts[7].key, multisig: authority.multisig(),
        transaction_index: authority.transaction_index(), previously_paused, paused: request.paused };
    commit_state_writes(program, &context.rent, [(&write, &accounts[7])]).map_err(state_error)?;
    Ok(GuardianOperationEvent::Pause(event))
}
