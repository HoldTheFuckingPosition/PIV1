//! Strict claim, pending, initializer, guardian and explicit-intake dispatch.
//! No remaining accounts, Rent input account or caller-selected backend is accepted.
//! All errors propagate; the eventual transaction boundary must roll back effects.

use anchor_lang::{
    prelude::{AccountInfo, Pubkey, Rent, SolanaSysvar},
    solana_program::{entrypoint::ProgramResult, program_error::ProgramError},
};
#[cfg(not(target_os = "solana"))]
use anchor_lang::solana_program::instruction::Instruction;
use crate::{
    events::KifClaimed,
    genesis_initialization::{initialize_approved_genesis_with_checked_recipients,
        GenesisInitializationResult, InitializedGenesisAccounts, RecipientCheckedGenesisRoles},
    instruction_errors::{execution_program_error, initialization_program_error, HOST_RUNTIME_UNAVAILABLE_CODE},
    instructions::claim_kif::decode_claim_kif,
    instructions::initialize::{InitializePiv1Parameters, INITIALIZE_PIV1_SELECTOR},
    kif_claim_accounts::KifClaimAccountInfos,
    kif_claim_execution::{execute_kif_claim, KifClaimExecutionAccounts, KifClaimExecutionResult},
    state::KifClaimRequest,
};

/// Runtime processor: decode, exact account count, host guard, trusted sysvars,
/// authenticated execution, then factual claim/guardian/contribution events. Pending
/// recognition performs no CPI. Initialization obtains its own trusted context
/// and completes the checked-recipient normalized profile without emitting an
/// event. No static ID is used.
pub fn process_instruction(
    runtime_program_id: &Pubkey,
    accounts: &[AccountInfo<'_>],
    data: &[u8],
) -> ProgramResult {
    dispatch(runtime_program_id, accounts, data, cfg!(target_os = "solana"), Rent::get,
        execute_kif_claim, |event| anchor_lang::emit!(event), initialize_approved_genesis_with_checked_recipients)
}

/// Host-only shared-dispatch evidence. Injected Rent and invocation/event callbacks
/// are modeling inputs, not runtime authorities. The actual Task 2.10 execution
/// runs inside this seam. It is absent from Solana and has no instruction selector.
#[cfg(not(target_os = "solana"))]
pub fn process_instruction_with_host_callbacks<'info>(
    runtime_program_id: &Pubkey,
    accounts: &[AccountInfo<'info>],
    data: &[u8],
    get_rent: impl FnOnce() -> Result<Rent, ProgramError>,
    invoker: impl FnOnce(&Instruction, &[AccountInfo<'info>], &[&[&[u8]]]) -> ProgramResult,
    emit_event: impl FnOnce(KifClaimed),
) -> ProgramResult {
    dispatch(runtime_program_id, accounts, data, true, get_rent,
        |program, rent, accounts, request| {
            crate::kif_claim_execution::execute_kif_claim_with_host_invoker(
                program, rent, accounts, request, invoker)
        }, emit_event, initialize_approved_genesis_with_checked_recipients)
}

/// Explicit initializer-only host effects seam, absent from Solana. Context and
/// invocation are modeled; errors do not undo effects. No event is emitted.
#[cfg(not(target_os = "solana"))]
pub fn process_initialize_with_host_invoker<'info>(
    program: &Pubkey, accounts: &[AccountInfo<'info>], data: &[u8],
    context: crate::squads_execution::ModeledSquadsInvocationContext,
    invoke: impl FnMut(&Instruction, &[AccountInfo<'info>], &[&[&[u8]]]) -> ProgramResult,
) -> ProgramResult {
    if data.get(..8) != Some(INITIALIZE_PIV1_SELECTOR.as_slice()) {
        return Err(ProgramError::InvalidInstructionData);
    }
    dispatch(program, accounts, data, true, Rent::get, execute_kif_claim, |_| {},
        |program, accounts, data, roles| {
            crate::genesis_initialization::initialize_approved_genesis_with_checked_recipients_with_host_invoker(
                program, accounts, data, roles, context, invoke)
        })
}

#[allow(clippy::too_many_arguments)]
fn dispatch<'info>(
    program: &Pubkey,
    accounts: &[AccountInfo<'info>],
    data: &[u8],
    execution_available: bool,
    get_rent: impl FnOnce() -> Result<Rent, ProgramError>,
    execute: impl FnOnce(&Pubkey, &Rent, KifClaimExecutionAccounts<'_, 'info>, KifClaimRequest) -> KifClaimExecutionResult,
    emit_event: impl FnOnce(KifClaimed),
    initialize: impl FnOnce(&Pubkey, &[AccountInfo<'info>], &[u8], RecipientCheckedGenesisRoles)
        -> GenesisInitializationResult<InitializedGenesisAccounts>,
) -> ProgramResult {
    use crate::instructions::{deposit_sol::DEPOSIT_SOL_SELECTOR, deposit_jitosol::DEPOSIT_JITOSOL_SELECTOR};
    if data.get(..8).is_some_and(|selector| selector == DEPOSIT_SOL_SELECTOR || selector == DEPOSIT_JITOSOL_SELECTOR) {
        // Deposit execution owns its runtime guard, never the claim host seam.
        return crate::contribution_execution::process_instruction(program, accounts, data);
    }
    use crate::instructions::{guardian_heartbeat::GUARDIAN_HEARTBEAT_SELECTOR, pause::SET_PAUSE_SELECTOR};
    if data.get(..8).is_some_and(|selector| selector == GUARDIAN_HEARTBEAT_SELECTOR || selector == SET_PAUSE_SELECTOR) {
        // This route owns its actual runtime context and cannot borrow the
        // claim-only host callback's execution authority or event callback.
        return crate::guardian_operations::process_instruction(program, accounts, data);
    }
    if data.get(..8) == Some(INITIALIZE_PIV1_SELECTOR.as_slice()) {
        let roles = initialization_roles(data, accounts.len())?;
        if !execution_available { return Err(ProgramError::Custom(HOST_RUNTIME_UNAVAILABLE_CODE)); }
        let _ = initialize(program, accounts, data, roles).map_err(initialization_program_error)?;
        return Ok(());
    }
    use crate::instructions::reconcile_pending::{decode_reconcile_pending, RECONCILE_PENDING_DISCRIMINATOR};
    if data.get(..8) == Some(RECONCILE_PENDING_DISCRIMINATOR.as_slice()) {
        decode_reconcile_pending(data)?;
        if accounts.len() < 4 { return Err(ProgramError::NotEnoughAccountKeys); }
        if accounts.len() > 4 { return Err(ProgramError::InvalidArgument); }
        if !execution_available { return Err(ProgramError::Custom(HOST_RUNTIME_UNAVAILABLE_CODE)); }
        let rent = get_rent()?;
        crate::pending_reconciliation::execute_pending_reconciliation(program, &rent,
            crate::pending_accounts::PendingAccountInfos { config: &accounts[0],
                active_distribution: &accounts[1], pending_sol: &accounts[2], pending_jito: &accounts[3] })
            .map_err(|error| ProgramError::Custom(crate::instruction_errors::piv1_error_code(error)))?;
        return Ok(());
    }
    let request = decode_claim_kif(data)?;
    if accounts.len() < 5 { return Err(ProgramError::NotEnoughAccountKeys); }
    if accounts.len() > 5 { return Err(ProgramError::InvalidArgument); }
    if !execution_available { return Err(ProgramError::Custom(HOST_RUNTIME_UNAVAILABLE_CODE)); }
    let rent = get_rent()?;
    let claim = KifClaimAccountInfos { config: &accounts[0], guardian_reward: &accounts[1],
        kif_sol: &accounts[2], guardian: &accounts[3] };
    let transfer = execute(program, &rent, KifClaimExecutionAccounts {
        claim, system_program: &accounts[4] }, request).map_err(execution_program_error)?;
    emit_event(KifClaimed { guardian_reward: *claim.guardian_reward.key,
        guardian: transfer.destination, amount_lamports: transfer.amount_lamports });
    Ok(())
}

/// A single bounded topology; only manager/referrer equality chooses 35 or 34.
/// Role indices are never supplied by the message or inferred through PDA scans.
#[inline(never)]
fn initialization_roles(data: &[u8], count: usize) -> Result<RecipientCheckedGenesisRoles, ProgramError> {
    use crate::{genesis_allocation::GenesisAllocationRoles,
        genesis_initialization::GenesisInitializationRoles,
        genesis_preflight::{GenesisPreflightRoles, GenesisProtocolRoles},
        genesis_recipients::GenesisRecipientSelection, squads_execution::SquadsBootstrapRoles};
    let parameters = InitializePiv1Parameters::decode(data).map_err(|_| ProgramError::InvalidInstructionData)?;
    let shared = parameters.model.protocol.manager_fee_account == parameters.model.protocol.referrer_token_account;
    let payer = if shared { 29 } else { 30 };
    let recipient = payer + 3;
    if count < recipient + 2 { return Err(ProgramError::NotEnoughAccountKeys); }
    if count > recipient + 2 { return Err(ProgramError::InvalidArgument); }
    Ok(RecipientCheckedGenesisRoles {
        initialization: GenesisInitializationRoles {
            allocation: GenesisAllocationRoles {
                preflight: GenesisPreflightRoles {
                    bootstrap: SquadsBootstrapRoles { program: 0, program_data: 1, multisig: 2,
                        proposal: 3, transaction: 4, vault: 5, instructions: 6, config: 7 },
                    protocol: GenesisProtocolRoles { program: 23, pool: 24, validator_list: 25,
                        reserve: 26, mint: 27, manager_fee: 28, referrer: if shared { 28 } else { 29 } },
                    targets: core::array::from_fn(|index| 7 + index),
                }, payer, system_program: payer + 1,
            }, token_program: payer + 2,
        },
        recipients: GenesisRecipientSelection { htfp_recipient: recipient, team_owner_recipient: recipient + 1,
            htfp_vault_index: parameters.htfp_vault_index, team_owner_vault_index: parameters.team_owner_vault_index },
    })
}
