//! Strict five-account KIF claim instruction dispatch. No other instruction,
//! remaining account, Rent input account or caller-selected backend is accepted.
//! All errors propagate; the eventual transaction boundary must roll back effects.

use anchor_lang::{
    prelude::{AccountInfo, Pubkey, Rent, SolanaSysvar},
    solana_program::{entrypoint::ProgramResult, program_error::ProgramError},
};
#[cfg(not(target_os = "solana"))]
use anchor_lang::solana_program::instruction::Instruction;
use crate::{
    events::KifClaimed,
    instruction_errors::{execution_program_error, HOST_RUNTIME_UNAVAILABLE_CODE},
    instructions::claim_kif::decode_claim_kif,
    kif_claim_accounts::KifClaimAccountInfos,
    kif_claim_execution::{execute_kif_claim, KifClaimExecutionAccounts, KifClaimExecutionResult},
    state::KifClaimRequest,
};

/// Runtime processor: decode, exact account count, host guard, Rent::get,
/// authenticated execution, then one factual Anchor event. No static ID is used.
pub fn process_instruction(
    runtime_program_id: &Pubkey,
    accounts: &[AccountInfo<'_>],
    data: &[u8],
) -> ProgramResult {
    dispatch(runtime_program_id, accounts, data, cfg!(target_os = "solana"), Rent::get,
        execute_kif_claim, |event| anchor_lang::emit!(event))
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
        }, emit_event)
}

fn dispatch<'info>(
    program: &Pubkey,
    accounts: &[AccountInfo<'info>],
    data: &[u8],
    execution_available: bool,
    get_rent: impl FnOnce() -> Result<Rent, ProgramError>,
    execute: impl FnOnce(&Pubkey, &Rent, KifClaimExecutionAccounts<'_, 'info>, KifClaimRequest) -> KifClaimExecutionResult,
    emit_event: impl FnOnce(KifClaimed),
) -> ProgramResult {
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
