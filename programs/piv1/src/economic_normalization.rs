//! Normalize only supported economic surplus into pending custody. Every vault
//! first covers its own obligation. Operational funding and Token-native excess
//! stay untouched/unclassified; bootstrap and valuation are separate boundaries.

use std::rc::Rc;
use anchor_lang::{prelude::{AccountInfo, Pubkey, Rent, SolanaSysvar}, solana_program::{
    entrypoint::ProgramResult, instruction::Instruction, program::invoke_signed,
    program_error::ProgramError, program_pack::Pack, system_instruction, system_program,
}};
use solana_sha256_hasher::hash;
use spl_token::state::Mint;
use crate::{accounts::{authenticate_fixed_accounts, rent_floor, seeds, FixedAccountInfos},
    errors::Piv1Error, events::UntrackedBalanceReconciled,
    instruction_errors::{piv1_error_code, HOST_RUNTIME_UNAVAILABLE_CODE},
    instructions::reconcile_untracked_balances::decode_reconcile_untracked,
    state::reconciliation::{economic_custody_surplus, record_economic_normalization},
    state_persistence::{commit_state_writes, PreparedStateWrite, StateEnvelope},
};

const COUNT: usize = 13;
const AUTHORITY: usize = 9;
const MINT: usize = 10;
const SYSTEM: usize = 11;
const TOKEN: usize = 12;

/// Runtime Rent and fixed signed System/Token invocations only. Ordinary host
/// syscall stubs cannot execute this instruction.
pub fn process_instruction(program: &Pubkey, accounts: &[AccountInfo<'_>], data: &[u8]) -> ProgramResult {
    dispatch(program, accounts, data, cfg!(target_os = "solana"), Rent::get,
        |instruction, infos, signers| invoke_signed(instruction, infos, signers),
        |event| anchor_lang::emit!(event))
}

/// Explicit host-only invocation/event model. Signer flags and Rent are modeled;
/// errors do not undo partial effects. Discard failed staged host transactions.
#[cfg(not(target_os = "solana"))]
pub fn process_instruction_with_host_callbacks<'info>(
    program: &Pubkey, accounts: &[AccountInfo<'info>], data: &[u8],
    get_rent: impl FnOnce() -> Result<Rent, ProgramError>,
    invoker: impl FnMut(&Instruction, &[AccountInfo<'info>], &[&[&[u8]]]) -> ProgramResult,
    emit: impl FnOnce(UntrackedBalanceReconciled),
) -> ProgramResult { dispatch(program, accounts, data, true, get_rent, invoker, emit) }

fn error(value: Piv1Error) -> ProgramError { ProgramError::Custom(piv1_error_code(value)) }
fn overflow() -> ProgramError { error(Piv1Error::ArithmeticOverflow) }

fn dispatch<'info>(program: &Pubkey, accounts: &[AccountInfo<'info>], data: &[u8], available: bool,
    get_rent: impl FnOnce() -> Result<Rent, ProgramError>,
    mut invoker: impl FnMut(&Instruction, &[AccountInfo<'info>], &[&[&[u8]]]) -> ProgramResult,
    emit: impl FnOnce(UntrackedBalanceReconciled),
) -> ProgramResult {
    decode_reconcile_untracked(data)?;
    if accounts.len() < COUNT { return Err(ProgramError::NotEnoughAccountKeys); }
    if accounts.len() > COUNT { return Err(ProgramError::InvalidArgument); }
    if !available { return Err(ProgramError::Custom(HOST_RUNTIME_UNAVAILABLE_CODE)); }
    let rent = get_rent()?; distinct_accounts(accounts)?;
    let mut prepared = prepare(program, &rent, accounts)?;
    let prepared = prepared.as_mut();
    preflight_borrows(accounts, &prepared.movements)?;
    for movement in &prepared.movements {
        let bump = [movement.bump]; let signer = [movement.seed, &bump];
        let infos = if movement.source == 7 {
            vec![accounts[7].clone(), accounts[MINT].clone(), accounts[8].clone(),
                accounts[AUTHORITY].clone(), accounts[TOKEN].clone()]
        } else { vec![accounts[movement.source].clone(), accounts[2].clone(), accounts[SYSTEM].clone()] };
        invoker(&movement.instruction, &infos, &[&signer])?;
        prepared.records[movement.source] = movement.source_after.clone();
        prepared.records[movement.destination] = movement.destination_after.clone();
        verify_records(accounts, &prepared.records)?;
    }
    // Also covers the no-CPI profile. Old Config still exists at this point;
    // all obligations remain covered while pending surplus awaits its commit.
    verify_records(accounts, &prepared.records)?;
    authenticate_fixed_accounts(program, &rent, roles(accounts)).map_err(error)?
        .economic_normalization_observation().map_err(error)?;
    commit_state_writes(program, &rent, [(&prepared.write, &accounts[0])]).map_err(error)?;
    emit(prepared.event); Ok(())
}

fn roles<'a, 'info>(a: &'a [AccountInfo<'info>]) -> FixedAccountInfos<'a, 'info> {
    FixedAccountInfos { config: &a[0], active_distribution: &a[1], pending_sol: &a[2],
        principal_sol: &a[3], operational_sol: &a[4], distribution_escrow: &a[5],
        kif_sol: &a[6], principal_jito: &a[7], pending_jito: &a[8] }
}
fn distinct_accounts(accounts: &[AccountInfo<'_>]) -> ProgramResult {
    for (index, account) in accounts.iter().enumerate() {
        if accounts[index + 1..].iter().any(|other| account.key == other.key
            || Rc::ptr_eq(&account.data, &other.data) || Rc::ptr_eq(&account.lamports, &other.lamports)) {
            return Err(error(Piv1Error::AccountAlias));
        }
    }
    Ok(())
}

// Arbitrary external program buffers are borrowed and hashed, never copied.
#[derive(Clone, Debug, Eq, PartialEq)]
struct AccountRecord {
    key: Pubkey, owner: Pubkey, signer: bool, writable: bool, executable: bool,
    rent_epoch: u64, lamports: u64, data_len: usize, data_hash: [u8; 32],
}
impl AccountRecord {
    fn read(account: &AccountInfo<'_>) -> Result<Self, ProgramError> {
        let data = account.try_borrow_data().map_err(|_| error(Piv1Error::AccountBorrowFailed))?;
        Ok(Self { key: *account.key, owner: *account.owner, signer: account.is_signer,
            writable: account.is_writable, executable: account.executable, rent_epoch: account.rent_epoch,
            lamports: **account.try_borrow_lamports().map_err(|_| error(Piv1Error::AccountBorrowFailed))?,
            data_len: data.len(), data_hash: hash(&data).to_bytes() })
    }
}
fn verify_records(accounts: &[AccountInfo<'_>], expected: &[AccountRecord]) -> ProgramResult {
    for (account, record) in accounts.iter().zip(expected) {
        if AccountRecord::read(account)? != *record { return Err(error(Piv1Error::ContributionObservationMismatch)); }
    }
    Ok(())
}
struct Movement {
    instruction: Instruction, source: usize, destination: usize, seed: &'static [u8], bump: u8,
    source_after: AccountRecord, destination_after: AccountRecord,
}
struct PreparedNormalization {
    records: Vec<AccountRecord>, movements: Vec<Movement>, write: PreparedStateWrite,
    event: UntrackedBalanceReconciled,
}

#[inline(never)]
fn prepare(program: &Pubkey, rent: &Rent, accounts: &[AccountInfo<'_>])
    -> Result<Box<PreparedNormalization>, ProgramError>
{
    let authenticated = authenticate_fixed_accounts(program, rent, roles(accounts)).map_err(error)?;
    let config = authenticated.config(); let round = authenticated.distribution();
    let before = authenticated.economic_normalization_observation().map_err(error)?;
    let surplus = economic_custody_surplus(config, round, before).map_err(error)?;
    for (index, id) in [(SYSTEM, system_program::ID), (TOKEN, spl_token::ID)] {
        if accounts[index].key != &id || !accounts[index].executable {
            return Err(error(Piv1Error::InvalidProgramIdentity));
        }
    }
    // The shared canonical PDA remains unallocated, readonly System data. This
    // excludes SPL multisig semantics without a general wallet-owner policy.
    if accounts[AUTHORITY].key != &config.piv_authority { return Err(error(Piv1Error::InvalidAccountPda)); }
    if accounts[AUTHORITY].owner != &system_program::ID { return Err(error(Piv1Error::InvalidAccountOwner)); }
    if accounts[AUTHORITY].executable { return Err(error(Piv1Error::ExecutableAccount)); }
    if !accounts[AUTHORITY].try_borrow_data().map_err(|_| error(Piv1Error::AccountBorrowFailed))?.is_empty() {
        return Err(error(Piv1Error::InvalidAccountData));
    }
    let decimals = authenticate_mint(rent, &accounts[MINT], &config.jitosol_mint)?;
    if !accounts[0].is_writable { return Err(error(Piv1Error::AccountNotWritable)); }
    let mut after = before;
    let moved_sol = surplus.principal_sol_lamports.checked_add(surplus.escrow_sol_lamports)
        .and_then(|v| v.checked_add(surplus.kif_sol_lamports)).ok_or(overflow())?;
    after.pending_sol.lamports = after.pending_sol.lamports.checked_add(moved_sol).ok_or(overflow())?;
    after.principal_sol.lamports = after.principal_sol.lamports.checked_sub(surplus.principal_sol_lamports).ok_or(overflow())?;
    after.distribution_escrow.lamports = after.distribution_escrow.lamports.checked_sub(surplus.escrow_sol_lamports).ok_or(overflow())?;
    after.kif_sol.lamports = after.kif_sol.lamports.checked_sub(surplus.kif_sol_lamports).ok_or(overflow())?;
    after.pending_jitosol_units = after.pending_jitosol_units.checked_add(surplus.principal_jitosol_units).ok_or(overflow())?;
    after.principal_jitosol_units = after.principal_jitosol_units.checked_sub(surplus.principal_jitosol_units).ok_or(overflow())?;
    let mut next = Box::new(config.clone());
    let result = record_economic_normalization(&mut next, round, before, after).map_err(error)?;
    let write = PreparedStateWrite::new(program, *accounts[0].key,
        StateEnvelope::config(config).map_err(error)?, StateEnvelope::config(&next).map_err(error)?).map_err(error)?;
    let records: Vec<_> = accounts.iter().map(AccountRecord::read).collect::<Result<_, _>>()?;
    let mut expected = records.clone(); let mut movements = Vec::with_capacity(4);
    for (source, amount, seed, bump) in [
        (3, surplus.principal_sol_lamports, seeds::PRINCIPAL_SOL, config.bumps.principal_sol_queue),
        (5, surplus.escrow_sol_lamports, seeds::DISTRIBUTION_ESCROW, config.bumps.distribution_escrow),
        (6, surplus.kif_sol_lamports, seeds::KIF_SOL, config.bumps.kif_sol_vault),
    ] {
        if amount == 0 { continue; }
        expected[source].lamports = expected[source].lamports.checked_sub(amount).ok_or(overflow())?;
        expected[2].lamports = expected[2].lamports.checked_add(amount).ok_or(overflow())?;
        movements.push(Movement { instruction: system_instruction::transfer(accounts[source].key, accounts[2].key, amount),
            source, destination: 2, seed, bump, source_after: expected[source].clone(), destination_after: expected[2].clone() });
    }
    if surplus.principal_jitosol_units != 0 {
        // Fixed authenticated legacy Token accounts have no delegate/native/
        // close-authority mode. TransferChecked changes only amount bytes.
        for (index, units) in [(7, after.principal_jitosol_units), (8, after.pending_jitosol_units)] {
            let mut bytes = accounts[index].try_borrow_data().map_err(|_| error(Piv1Error::AccountBorrowFailed))?.to_vec();
            bytes[64..72].copy_from_slice(&units.to_le_bytes()); expected[index].data_hash = hash(&bytes).to_bytes();
        }
        movements.push(Movement { instruction: spl_token::instruction::transfer_checked(&spl_token::ID,
            accounts[7].key, accounts[MINT].key, accounts[8].key, accounts[AUTHORITY].key, &[],
            surplus.principal_jitosol_units, decimals)?, source: 7, destination: 8,
            seed: seeds::AUTHORITY, bump: config.bumps.piv_authority,
            source_after: expected[7].clone(), destination_after: expected[8].clone() });
    }
    Ok(Box::new(PreparedNormalization { records, movements, write,
        event: UntrackedBalanceReconciled { config: *accounts[0].key,
            newly_accounted_sol_lamports: result.newly_accounted_sol_lamports,
            newly_accounted_jitosol_units: result.newly_accounted_jitosol_units,
            pending_sol_lamports_after: result.accounted_pending_sol_lamports_after,
            pending_jitosol_units_after: result.accounted_pending_jitosol_units_after } }))
}

fn authenticate_mint(rent: &Rent, account: &AccountInfo<'_>, mint: &Pubkey) -> Result<u8, ProgramError> {
    if account.key != mint { return Err(error(Piv1Error::InvalidTokenCustody)); }
    if account.owner != &spl_token::ID { return Err(error(Piv1Error::InvalidAccountOwner)); }
    if account.executable { return Err(error(Piv1Error::ExecutableAccount)); }
    let bytes = account.try_borrow_data().map_err(|_| error(Piv1Error::AccountBorrowFailed))?;
    if bytes.len() != Mint::LEN { return Err(error(Piv1Error::InvalidAccountSize)); }
    if **account.try_borrow_lamports().map_err(|_| error(Piv1Error::AccountBorrowFailed))? < rent_floor(rent, Mint::LEN).map_err(error)? {
        return Err(error(Piv1Error::AccountRentDeficit));
    }
    Ok(Mint::unpack(&bytes).map_err(|_| error(Piv1Error::InvalidTokenCustody))?.decimals)
}

fn preflight_borrows(accounts: &[AccountInfo<'_>], movements: &[Movement]) -> ProgramResult {
    let mut mutable = [false; COUNT]; mutable[0] = true;
    for movement in movements {
        mutable[movement.source] = true; mutable[movement.destination] = true;
    }
    let mut data = Vec::with_capacity(COUNT); let mut lamports = Vec::with_capacity(COUNT);
    for (index, account) in accounts.iter().enumerate() {
        if mutable[index] {
            if !account.is_writable { return Err(error(Piv1Error::AccountNotWritable)); }
            data.push(account.try_borrow_mut_data().map_err(|_| error(Piv1Error::AccountBorrowFailed))?);
            lamports.push(account.try_borrow_mut_lamports().map_err(|_| error(Piv1Error::AccountBorrowFailed))?);
        }
    }
    // Pinned SPL TransferChecked borrows the readonly Mint data mutably.
    if movements.iter().any(|m| m.source == 7) {
        data.push(accounts[MINT].try_borrow_mut_data().map_err(|_| error(Piv1Error::AccountBorrowFailed))?);
    }
    Ok(())
}
