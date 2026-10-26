//! Exact-approved recipient replacement for future distributions. Active rounds
//! retain their own frozen destinations. No custody, economics or KIF changes.
//! Beneficiaries are authorized addresses; their application state is opaque.

use std::rc::Rc;
use anchor_lang::{prelude::{AccountInfo, Clock, Pubkey, Rent, SolanaSysvar},
    solana_program::{entrypoint::ProgramResult, instruction::get_stack_height,
        program_error::ProgramError}};
use crate::{
    accounts::{decode_state, CONFIG_DISCRIMINATOR},
    errors::Piv1Error,
    events::RecipientsUpdated,
    instruction_errors::{piv1_error_code, squads_program_error, HOST_RUNTIME_UNAVAILABLE_CODE},
    instructions::update_config::UpdateRecipientsRequest,
    squads_execution::{self, SquadsExecutionRoles},
    state::PivConfig,
    state_persistence::{commit_state_writes, PreparedStateWrite, StateEnvelope},
};

struct RuntimeContext { stack_height: usize, clock: Clock, rent: Rent }

/// Obtains actual runtime context. Ordinary host stubs cannot authorize this path.
pub fn process_instruction(program: &Pubkey, accounts: &[AccountInfo<'_>], data: &[u8]) -> ProgramResult {
    dispatch(program, accounts, data, cfg!(target_os = "solana"), || Ok(RuntimeContext {
        stack_height: get_stack_height(), clock: Clock::get()?, rent: Rent::get()?,
    }), |event| anchor_lang::emit!(event))
}

/// Explicit host model only, absent from Solana and from native dispatch.
#[cfg(not(target_os = "solana"))]
pub fn process_instruction_with_host_callbacks(
    program: &Pubkey, accounts: &[AccountInfo<'_>], data: &[u8],
    get_context: impl FnOnce() -> Result<squads_execution::ModeledSquadsInvocationContext, ProgramError>,
    emit: impl FnOnce(RecipientsUpdated),
) -> ProgramResult {
    dispatch(program, accounts, data, true, || get_context().map(|context| RuntimeContext {
        stack_height: context.stack_height, clock: context.clock, rent: context.rent,
    }), emit)
}

fn error(error: Piv1Error) -> ProgramError { ProgramError::Custom(piv1_error_code(error)) }

// Fixed initialized Squads roles: Program, ProgramData, multisig, proposal,
// transaction, vault, Instructions, Config, registry, six rewards, Clock,
// new HTFP recipient, new Team Owner recipient. Only Config is written.
#[inline(never)]
fn dispatch(
    program: &Pubkey, accounts: &[AccountInfo<'_>], data: &[u8], available: bool,
    get_context: impl FnOnce() -> Result<RuntimeContext, ProgramError>,
    emit: impl FnOnce(RecipientsUpdated),
) -> ProgramResult {
    let request = UpdateRecipientsRequest::decode(data)?;
    if accounts.len() < 18 { return Err(ProgramError::NotEnoughAccountKeys); }
    if accounts.len() > 18 { return Err(ProgramError::InvalidArgument); }
    if !available { return Err(ProgramError::Custom(HOST_RUNTIME_UNAVAILABLE_CODE)); }
    let context = get_context()?;
    for (index, account) in accounts.iter().enumerate() {
        if accounts[index + 1..].iter().any(|other| account.key == other.key
            || Rc::ptr_eq(&account.data, &other.data) || Rc::ptr_eq(&account.lamports, &other.lamports)) {
            return Err(error(Piv1Error::AccountAlias));
        }
    }
    let authority = squads_execution::dispatch(program, accounts, data, SquadsExecutionRoles {
        program: 0, program_data: 1, multisig: 2, proposal: 3, transaction: 4, vault: 5,
        instructions: 6, config: 7, guardian_registry: 8, rewards: [9, 10, 11, 12, 13, 14], clock: 15,
    }, request.vault_index, true, || context.stack_height,
        || Ok(context.clock.clone()), || Ok(context.rent.clone())).map_err(squads_program_error)?;
    let mut config: Box<PivConfig> = decode_state(&accounts[7], program, &context.rent,
        PivConfig::SPACE, CONFIG_DISCRIMINATOR).map_err(error)?;
    for index in [16, 17] {
        if *accounts[index].key == config.piv_authority || accounts[index].owner == program {
            return Err(error(Piv1Error::AccountAlias));
        }
    }
    let before = StateEnvelope::config(&config).map_err(error)?;
    let event = RecipientsUpdated { config: *accounts[7].key, multisig: authority.multisig(),
        transaction_index: authority.transaction_index(), previous_htfp_recipient: config.htfp_recipient,
        previous_team_owner_recipient: config.team_owner_recipient,
        htfp_recipient: *accounts[16].key, team_owner_recipient: *accounts[17].key };
    config.htfp_recipient = event.htfp_recipient; config.team_owner_recipient = event.team_owner_recipient;
    let after = StateEnvelope::config(&config).map_err(error)?;
    let write = PreparedStateWrite::new(program, *accounts[7].key, before, after).map_err(error)?;
    commit_state_writes(program, &context.rent, [(&write, &accounts[7])]).map_err(error)?;
    emit(event); Ok(())
}
