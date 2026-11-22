//! Sponsor the exact current storage gap of one registered permanent SOL vault.
//! This instruction never changes any economic ledger or initializes legacy state.

use anchor_lang::{
    prelude::{AccountInfo, Pubkey, Rent, SolanaSysvar},
    solana_program::{entrypoint::ProgramResult, program::invoke, program_error::ProgramError,
        system_instruction, system_program},
};
use crate::{
    accounts::{decode_fixed_distribution, decode_state, permanent_sol_balance, rent_floor,
        validate_fixed_config, CONFIG_DISCRIMINATOR},
    errors::{Piv1Error, Piv1Result},
    instruction_errors::{piv1_error_code, HOST_RUNTIME_UNAVAILABLE_CODE},
    state::{config::StorageVault, reconciliation::{economic_custody_obligations, validate_custody_state_binding},
        PivConfig},
    state_persistence::{commit_state_writes, PreparedStateWrite, StateEnvelope},
};

pub(super) const SELECTOR: &[u8; 8] = b"PIV1RT01";

pub(super) fn process_instruction(program: &Pubkey, accounts: &[AccountInfo<'_>], data: &[u8]) -> ProgramResult {
    let [config, round, vault, payer, system] = accounts else { return Err(ProgramError::InvalidArgument) };
    let [role, amount @ ..] = data.get(8..).ok_or(ProgramError::InvalidInstructionData)? else {
        return Err(ProgramError::InvalidInstructionData);
    };
    let role = StorageVault::ALL.get(usize::from(*role)).copied().ok_or(ProgramError::InvalidInstructionData)?;
    let expected = u64::from_le_bytes(amount.try_into().map_err(|_| ProgramError::InvalidInstructionData)?);
    if !cfg!(target_os = "solana") { return Err(ProgramError::Custom(HOST_RUNTIME_UNAVAILABLE_CODE)); }
    if *system.key != system_program::ID || !system.executable {
        return Err(ProgramError::IncorrectProgramId);
    }
    if !payer.is_signer { return Err(ProgramError::MissingRequiredSignature); }
    if !config.is_writable || !vault.is_writable || !payer.is_writable
        || payer.owner != &system_program::ID || payer.executable || !payer.data_is_empty() {
        return Err(ProgramError::InvalidAccountData);
    }
    let rent = Rent::get()?;
    let prepared = prepare(program, &rent, config, round, vault, payer, role, expected).map_err(error)?;
    invoke(&system_instruction::transfer(payer.key, vault.key, expected),
        &[payer.clone(), vault.clone(), system.clone()])?;
    let post = permanent_sol_balance(vault, &prepared.next, role, prepared.live_floor).map_err(error)?;
    if post.lamports != prepared.vault_after || lamports(payer).map_err(error)? != prepared.payer_after {
        return Err(error(Piv1Error::InvalidCustodyObservation));
    }
    commit_state_writes(program, &rent, [(&prepared.write, config)]).map_err(error)
}

struct PreparedTopUp {
    next: Box<PivConfig>,
    live_floor: u64,
    vault_after: u64,
    payer_after: u64,
    write: PreparedStateWrite,
}

#[allow(clippy::too_many_arguments)]
#[inline(never)]
fn prepare(program: &Pubkey, rent: &Rent, config_account: &AccountInfo<'_>, round_account: &AccountInfo<'_>,
    vault: &AccountInfo<'_>, payer: &AccountInfo<'_>, role: StorageVault, expected: u64) -> Piv1Result<PreparedTopUp> {
    let mut config: Box<PivConfig> = decode_state(config_account, program, rent, PivConfig::SPACE, CONFIG_DISCRIMINATOR)?;
    validate_fixed_config(&config, program, config_account.key)?;
    let stored = config.storage_reserve(role)?;
    let live_floor = rent_floor(rent, 0)?;
    if expected == 0 || live_floor.checked_sub(stored) != Some(expected) {
        return Err(Piv1Error::InvalidRent);
    }
    if StorageVault::ALL.iter().any(|other| other.address(&config) == payer.key)
        || &config.piv_authority == payer.key {
        return Err(Piv1Error::AccountAlias);
    }
    let round = decode_fixed_distribution(round_account, &config, program, rent)?;
    validate_custody_state_binding(&config, &round)?;
    let obligation = economic_custody_obligations(&config, &round)?;
    let booked = match role {
        StorageVault::PendingSol => obligation.pending_sol_lamports,
        StorageVault::PrincipalSol => obligation.principal_sol_lamports,
        StorageVault::OperationalSol => 0,
        StorageVault::DistributionEscrow => obligation.escrow_sol_lamports,
        StorageVault::KifSol => obligation.kif_sol_lamports,
    };
    let before = permanent_sol_balance(vault, &config, role, stored)?;
    if before.economic_lamports()? < booked { return Err(Piv1Error::EconomicCustodyDeficit); }
    let payer_after = lamports(payer)?.checked_sub(expected).ok_or(Piv1Error::AccountRentDeficit)?;
    if payer_after < live_floor { return Err(Piv1Error::AccountRentDeficit); }
    let vault_after = before.lamports.checked_add(expected).ok_or(Piv1Error::ArithmeticOverflow)?;
    let initial = StateEnvelope::config(&config)?;
    config.increase_storage_reserve(role, live_floor)?;
    let write = PreparedStateWrite::new(program, *config_account.key, initial, StateEnvelope::config(&config)?)?;
    Ok(PreparedTopUp { next: config, live_floor, vault_after, payer_after, write })
}

fn lamports(account: &AccountInfo<'_>) -> Piv1Result<u64> {
    Ok(**account.try_borrow_lamports().map_err(|_| Piv1Error::AccountBorrowFailed)?)
}

fn error(error: Piv1Error) -> ProgramError { ProgramError::Custom(piv1_error_code(error)) }
