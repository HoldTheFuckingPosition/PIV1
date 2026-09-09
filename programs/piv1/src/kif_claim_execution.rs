//! Isolated claim execution with checks-effects-interactions ordering.
//!
//! This library is not an entrypoint. A future handler must supply trusted
//! executing program ID/Rent and propagate every error to the transaction boundary.
//! Bookkeeping is persisted before the fixed System CPI. CPI/postcheck errors
//! require transaction rollback; this function cannot undo arbitrary CPI effects.
//! Never catch an execution error and commit the partially executed transaction.

use core::fmt;
use anchor_lang::{
    prelude::{AccountInfo, Pubkey, Rent},
    solana_program::{entrypoint::ProgramResult, instruction::Instruction,
        program::invoke_signed, program_error::ProgramError, system_instruction, system_program},
};
use crate::{
    accounts::seeds,
    errors::Piv1Error,
    kif_claim_accounts::{authenticate_kif_claim_accounts, KifClaimAccountInfos},
    state::{prepare_kif_claim, KifClaimRequest, KifClaimTransfer},
    state_persistence::{commit_state_writes, PreparedStateWrite, StateEnvelope},
};

/// Only the four isolated claim accounts and canonical executable System Program.
#[derive(Clone, Copy)]
pub struct KifClaimExecutionAccounts<'a, 'info> {
    pub claim: KifClaimAccountInfos<'a, 'info>,
    pub system_program: &'a AccountInfo<'info>,
}

/// Preserve the original state-validation or System invocation error.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum KifClaimExecutionError {
    State(Piv1Error),
    Invocation(ProgramError),
    /// Pinned host invocation is not a Solana runtime and must never be called.
    HostRuntimeUnavailable,
}
impl From<Piv1Error> for KifClaimExecutionError {
    fn from(error: Piv1Error) -> Self { Self::State(error) }
}
impl fmt::Display for KifClaimExecutionError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::State(error) => write!(f, "{error}"),
            Self::Invocation(error) => write!(f, "KIF System invocation failed: {error}"),
            Self::HostRuntimeUnavailable => f.write_str("KIF runtime invocation is unavailable on host"),
        }
    }
}
pub type KifClaimExecutionResult = Result<KifClaimTransfer, KifClaimExecutionError>;

/// Executes only the canonical official signed System transfer on Solana.
/// Non-Solana callers fail before account access, bookkeeping or invocation.
/// The cfg! guard retains type-checking of the pinned invocation on host builds.
pub fn execute_kif_claim(
    trusted_runtime_program_id: &Pubkey,
    trusted_runtime_rent: &Rent,
    accounts: KifClaimExecutionAccounts<'_, '_>,
    request: KifClaimRequest,
) -> KifClaimExecutionResult {
    if !cfg!(target_os = "solana") {
        return Err(KifClaimExecutionError::HostRuntimeUnavailable);
    }
    execute_with_invoker(trusted_runtime_program_id, trusted_runtime_rent, accounts, request,
        |instruction, infos, signer_seeds| invoke_signed(instruction, infos, signer_seeds))
}

/// Explicit host-only recording/emulation seam through the same execution core.
/// It supplies no SVM, signatures or rollback. A host transaction model must stage
/// all accounts/audit and commit only on success. The callback is unavailable on
/// Solana; no invocation backend can be selected by future instruction data.
#[cfg(not(target_os = "solana"))]
pub fn execute_kif_claim_with_host_invoker<'info>(
    trusted_runtime_program_id: &Pubkey,
    trusted_runtime_rent: &Rent,
    accounts: KifClaimExecutionAccounts<'_, 'info>,
    request: KifClaimRequest,
    invoker: impl FnOnce(&Instruction, &[AccountInfo<'info>], &[&[&[u8]]]) -> ProgramResult,
) -> KifClaimExecutionResult {
    execute_with_invoker(trusted_runtime_program_id, trusted_runtime_rent, accounts, request, invoker)
}

fn execute_with_invoker<'info>(
    program: &Pubkey,
    rent: &Rent,
    accounts: KifClaimExecutionAccounts<'_, 'info>,
    request: KifClaimRequest,
    invoker: impl FnOnce(&Instruction, &[AccountInfo<'info>], &[&[&[u8]]]) -> ProgramResult,
) -> KifClaimExecutionResult {
    let claim = accounts.claim;
    for account in [claim.config, claim.guardian_reward, claim.kif_sol, claim.guardian] {
        if account.key == accounts.system_program.key { return Err(Piv1Error::AccountAlias.into()); }
    }
    if *accounts.system_program.key != system_program::ID || !accounts.system_program.executable {
        return Err(Piv1Error::InvalidProgramIdentity.into());
    }
    let before = authenticate_kif_claim_accounts(program, rent, claim)?;
    let plan = prepare_kif_claim(before.config(), before.reward(), request, before.custody())?;
    let transfer = plan.transfer();
    let (next_config, next_reward, predicted_custody) = plan.staged_execution_values();
    let config_write = PreparedStateWrite::new(program, *claim.config.key,
        StateEnvelope::config(before.config())?, StateEnvelope::config(next_config)?)?;
    let reward_write = PreparedStateWrite::new(program, *claim.guardian_reward.key,
        StateEnvelope::reward(before.reward())?, StateEnvelope::reward(next_reward)?)?;
    let state_lamports = [lamports(claim.config)?, lamports(claim.guardian_reward)?];
    let instruction = system_instruction::transfer(&transfer.source, &transfer.destination,
        transfer.amount_lamports);
    let bump = [before.config().bumps.kif_sol_vault];
    let signer_group: &[&[u8]] = &[seeds::KIF_SOL, &bump];
    let invocation_accounts = [claim.kif_sol.clone(), claim.guardian.clone(), accounts.system_program.clone()];

    // Both mutable data and native balances must be available before CEI effects.
    // Hold all four simultaneously to reject shared backing as well as outstanding
    // shared/mutable borrows, then drop every guard before persistence or CPI.
    {
        let _source_data = claim.kif_sol.try_borrow_mut_data().map_err(|_| Piv1Error::AccountBorrowFailed)?;
        let _source_lamports = claim.kif_sol.try_borrow_mut_lamports().map_err(|_| Piv1Error::AccountBorrowFailed)?;
        let _destination_data = claim.guardian.try_borrow_mut_data().map_err(|_| Piv1Error::AccountBorrowFailed)?;
        let _destination_lamports = claim.guardian.try_borrow_mut_lamports().map_err(|_| Piv1Error::AccountBorrowFailed)?;
    }
    commit_state_writes(program, rent,
        [(&config_write, claim.config), (&reward_write, claim.guardian_reward)])?;
    invoker(&instruction, &invocation_accounts, &[signer_group])
        .map_err(KifClaimExecutionError::Invocation)?;

    // These are fresh actual account observations. Predictions above are never
    // passed to plan.commit as evidence of a completed transfer.
    let after = authenticate_kif_claim_accounts(program, rent, claim)?;
    if after.custody() != predicted_custody {
        return Err(Piv1Error::KifClaimObservationMismatch.into());
    }
    for (account, write) in [(claim.config, &config_write), (claim.guardian_reward, &reward_write)] {
        let data = account.try_borrow_data().map_err(|_| Piv1Error::AccountBorrowFailed)?;
        if **data != *write.replacement() { return Err(Piv1Error::KifClaimStateChanged.into()); }
    }
    if [lamports(claim.config)?, lamports(claim.guardian_reward)?] != state_lamports {
        return Err(Piv1Error::KifClaimObservationMismatch.into());
    }
    let mut original_config = before.config().clone();
    let mut original_reward = *before.reward();
    let result = plan.commit(&mut original_config, &mut original_reward, after.custody())?;
    if after.config() != &original_config || after.reward() != &original_reward {
        return Err(Piv1Error::KifClaimStateChanged.into());
    }
    Ok(result)
}

fn lamports(account: &AccountInfo<'_>) -> Result<u64, Piv1Error> {
    Ok(**account.try_borrow_lamports().map_err(|_| Piv1Error::AccountBorrowFailed)?)
}
