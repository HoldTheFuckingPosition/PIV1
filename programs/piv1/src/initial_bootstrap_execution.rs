//! Initial pending-to-principal custody and current official pool book value.
//! No post-settlement integration, staking, withdrawal, pool quote, mock revision
//! or capacity mapping. Token-native funding stays untouched and unclassified.

use std::rc::Rc;
use anchor_lang::{prelude::{AccountInfo, Clock, Pubkey, Rent, SolanaSysvar}, solana_program::{
    entrypoint::ProgramResult, instruction::Instruction, program::invoke_signed,
    program_error::ProgramError, program_pack::Pack, system_instruction, system_program,
}};
use solana_sha256_hasher::hash;
use spl_token::state::Mint;
use crate::{accounts::{authenticate_fixed_accounts, rent_floor, seeds, FixedAccountInfos},
    errors::{Piv1Error, Piv1Result}, events::InitialContributionsBootstrapped,
    instruction_errors::{piv1_error_code, protocol_program_error, HOST_RUNTIME_UNAVAILABLE_CODE},
    instructions::bootstrap_initial_contributions::decode_initial_bootstrap,
    integrations::jito_identity::{authenticate_jito_identity, AuthenticatedJitoIdentity,
        DeclaredJitoKeys, JitoIdentityAccountInfos},
    state::{bootstrap::bootstrap_with_valuation, PivConfig},
    state_persistence::{commit_state_writes, PreparedStateWrite, StateEnvelope},
};

const AUTHORITY: usize = 9;
const MINT: usize = 10;
const SYSTEM: usize = 11;
const TOKEN: usize = 12;

/// Native entry: actual runtime ID, Clock and Rent, with fixed signed CPIs.
pub fn process_instruction(program: &Pubkey, accounts: &[AccountInfo<'_>], data: &[u8]) -> ProgramResult {
    dispatch(program, accounts, data, cfg!(target_os = "solana"), Clock::get, Rent::get,
        |ix, infos, seeds| invoke_signed(ix, infos, seeds), |event| anchor_lang::emit!(event))
}

/// Explicit host model only. This does not authenticate signatures or undo
/// partial CPI effects; failed staged transactions must be discarded by tests.
#[cfg(not(target_os = "solana"))]
pub fn process_instruction_with_host_callbacks<'info>(
    program: &Pubkey, accounts: &[AccountInfo<'info>], data: &[u8],
    get_clock: impl FnOnce() -> Result<Clock, ProgramError>,
    get_rent: impl FnOnce() -> Result<Rent, ProgramError>,
    invoker: impl FnMut(&Instruction, &[AccountInfo<'info>], &[&[&[u8]]]) -> ProgramResult,
    emit: impl FnOnce(InitialContributionsBootstrapped),
) -> ProgramResult { dispatch(program, accounts, data, true, get_clock, get_rent, invoker, emit) }

fn error(value: Piv1Error) -> ProgramError { ProgramError::Custom(piv1_error_code(value)) }
fn overflow() -> ProgramError { error(Piv1Error::ArithmeticOverflow) }

#[allow(clippy::too_many_arguments)]
fn dispatch<'info>(program: &Pubkey, accounts: &[AccountInfo<'info>], data: &[u8], available: bool,
    get_clock: impl FnOnce() -> Result<Clock, ProgramError>, get_rent: impl FnOnce() -> Result<Rent, ProgramError>,
    mut invoker: impl FnMut(&Instruction, &[AccountInfo<'info>], &[&[&[u8]]]) -> ProgramResult,
    emit: impl FnOnce(InitialContributionsBootstrapped),
) -> ProgramResult {
    decode_initial_bootstrap(data)?;
    if accounts.len() < 18 { return Err(ProgramError::NotEnoughAccountKeys); }
    if accounts.len() > 19 { return Err(ProgramError::InvalidArgument); }
    if !available { return Err(ProgramError::Custom(HOST_RUNTIME_UNAVAILABLE_CODE)); }
    let clock = get_clock()?; let rent = get_rent()?;
    distinct_accounts(accounts)?;
    let mut prepared = prepare(program, &clock, &rent, accounts)?;
    let prepared = prepared.as_mut();
    preflight_borrows(accounts, &prepared.movements)?;
    for movement in &prepared.movements {
        let bump = [movement.bump]; let signer = [movement.seed, &bump];
        let infos = if movement.source == 8 {
            vec![accounts[8].clone(), accounts[MINT].clone(), accounts[7].clone(),
                accounts[AUTHORITY].clone(), accounts[TOKEN].clone()]
        } else { vec![accounts[2].clone(), accounts[3].clone(), accounts[SYSTEM].clone()] };
        invoker(&movement.instruction, &infos, &[&signer])?;
        prepared.records[movement.source] = movement.source_after.clone();
        prepared.records[movement.destination] = movement.destination_after.clone();
        verify_records(accounts, &prepared.records)?;
    }
    // Metadata/rent reauthentication only: old Config still names pending
    // obligations until the staged transition is committed. Exact transfer
    // records and the prevalidated new Config prove the resulting obligations.
    let _ = authenticate_fixed_accounts(program, &rent, roles(accounts)).map_err(error)?;
    commit_state_writes(program, &rent, [(&prepared.write, &accounts[0])]).map_err(error)?;
    emit(prepared.event); Ok(())
}

fn roles<'a, 'info>(a: &'a [AccountInfo<'info>]) -> FixedAccountInfos<'a, 'info> {
    FixedAccountInfos { config: &a[0], active_distribution: &a[1], pending_sol: &a[2], principal_sol: &a[3],
        operational_sol: &a[4], distribution_escrow: &a[5], kif_sol: &a[6], principal_jito: &a[7], pending_jito: &a[8] }
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

// Hash borrowed buffers so a large validator list/program cannot grow the heap.
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
struct PreparedBootstrap {
    records: Vec<AccountRecord>, movements: Vec<Movement>, write: PreparedStateWrite,
    event: InitialContributionsBootstrapped,
}

#[inline(never)]
fn prepare(program: &Pubkey, clock: &Clock, rent: &Rent, accounts: &[AccountInfo<'_>])
    -> Result<Box<PreparedBootstrap>, ProgramError>
{
    let authenticated = authenticate_fixed_accounts(program, rent, roles(accounts)).map_err(error)?;
    let config = authenticated.config();
    let shared_receiver = config.manager_fee_account == config.referrer_token_account;
    let expected_count = if shared_receiver { 18 } else { 19 };
    if accounts.len() < expected_count { return Err(ProgramError::NotEnoughAccountKeys); }
    if accounts.len() > expected_count { return Err(ProgramError::InvalidArgument); }
    let before = authenticated.initial_bootstrap_observation().map_err(error)?;
    let protocol = protocol_identity(config, accounts, shared_receiver)?;
    for (index, id) in [(SYSTEM, system_program::ID), (TOKEN, spl_token::ID)] {
        if accounts[index].key != &id || !accounts[index].executable {
            return Err(error(Piv1Error::InvalidProgramIdentity));
        }
    }
    if accounts[AUTHORITY].key != &config.piv_authority { return Err(error(Piv1Error::InvalidAccountPda)); }
    if accounts[AUTHORITY].owner != &system_program::ID { return Err(error(Piv1Error::InvalidAccountOwner)); }
    if accounts[AUTHORITY].executable { return Err(error(Piv1Error::ExecutableAccount)); }
    if !accounts[AUTHORITY].try_borrow_data().map_err(|_| error(Piv1Error::AccountBorrowFailed))?.is_empty() {
        return Err(error(Piv1Error::InvalidAccountData));
    }
    if **accounts[MINT].try_borrow_lamports().map_err(|_| error(Piv1Error::AccountBorrowFailed))?
        < rent_floor(rent, Mint::LEN).map_err(error)? { return Err(error(Piv1Error::AccountRentDeficit)); }
    if !accounts[0].is_writable { return Err(error(Piv1Error::AccountNotWritable)); }
    let sol = config.accounted_pending_sol_lamports; let tokens = config.accounted_pending_jitosol_units;
    let mut after = before;
    after.pending_sol.lamports = after.pending_sol.lamports.checked_sub(sol).ok_or(overflow())?;
    after.principal_sol.lamports = after.principal_sol.lamports.checked_add(sol).ok_or(overflow())?;
    after.pending_jitosol_units = after.pending_jitosol_units.checked_sub(tokens).ok_or(overflow())?;
    after.principal_jitosol_units = after.principal_jitosol_units.checked_add(tokens).ok_or(overflow())?;
    let mut next = Box::new(config.clone());
    let result = bootstrap_with_valuation(&mut next, authenticated.distribution(), before, after,
        |units| current_book_value(&protocol, clock, units)).map_err(error)?;
    let write = PreparedStateWrite::new(program, *accounts[0].key,
        StateEnvelope::config(config).map_err(error)?, StateEnvelope::config(&next).map_err(error)?).map_err(error)?;
    let records: Vec<_> = accounts.iter().map(AccountRecord::read).collect::<Result<_, _>>()?;
    let mut expected = records.clone(); let mut movements = Vec::with_capacity(2);
    if sol != 0 {
        expected[2].lamports = after.pending_sol.lamports; expected[3].lamports = after.principal_sol.lamports;
        movements.push(Movement { instruction: system_instruction::transfer(accounts[2].key, accounts[3].key, sol),
            source: 2, destination: 3, seed: seeds::PENDING_SOL, bump: config.bumps.pending_sol_vault,
            source_after: expected[2].clone(), destination_after: expected[3].clone() });
    }
    if tokens != 0 {
        for (index, units) in [(8, after.pending_jitosol_units), (7, after.principal_jitosol_units)] {
            let mut bytes = accounts[index].try_borrow_data().map_err(|_| error(Piv1Error::AccountBorrowFailed))?.to_vec();
            bytes[64..72].copy_from_slice(&units.to_le_bytes()); expected[index].data_hash = hash(&bytes).to_bytes();
        }
        movements.push(Movement { instruction: spl_token::instruction::transfer_checked(&spl_token::ID,
            accounts[8].key, accounts[MINT].key, accounts[7].key, accounts[AUTHORITY].key, &[], tokens,
            protocol.mint().decimals)?, source: 8, destination: 7, seed: seeds::AUTHORITY,
            bump: config.bumps.piv_authority, source_after: expected[8].clone(), destination_after: expected[7].clone() });
    }
    Ok(Box::new(PreparedBootstrap { records, movements, write, event: InitialContributionsBootstrapped {
        config: *accounts[0].key, integrated_sol_lamports: result.integrated_sol_lamports,
        integrated_jitosol_units: result.integrated_jitosol_units,
        contribution_value_lamports: result.contribution_value_lamports } }))
}

#[inline(never)]
fn protocol_identity(config: &PivConfig, accounts: &[AccountInfo<'_>], shared: bool)
    -> Result<Box<AuthenticatedJitoIdentity>, ProgramError>
{
    let declared = DeclaredJitoKeys { program: config.stake_pool_program, pool: config.stake_pool,
        validator_list: config.validator_list, reserve: config.reserve_stake, mint: config.jitosol_mint,
        manager_fee: config.manager_fee_account, referrer: config.referrer_token_account };
    authenticate_jito_identity(&declared, JitoIdentityAccountInfos { program: &accounts[13], pool: &accounts[14],
        validator_list: &accounts[15], reserve: &accounts[16], mint: &accounts[MINT],
        manager_fee: &accounts[17], referrer: &accounts[if shared { 17 } else { 18 }] })
        .map(Box::new).map_err(protocol_program_error)
}

/// Pinned SPL 2.0.3 state.rs calc_lamports_withdraw_amount uses recorded pool
/// supply. A direct Token burn can reduce Mint.supply before pool maintenance
/// copies it back (processor.rs update_stake_pool_balance). Such burn lag must
/// not change the denominator early. These are gross book-value facts, not a
/// fee/withdrawal quote, adapter revision, capacity or executable pool operation.
fn current_book_value(protocol: &AuthenticatedJitoIdentity, clock: &Clock, units: u64) -> Piv1Result<u64> {
    let pool = protocol.pool(); let total = pool.total_lamports(); let supply = pool.pool_token_supply();
    if pool.last_update_epoch() != clock.epoch || (total == 0) != (supply == 0)
        || protocol.mint().supply > supply || units > protocol.mint().supply {
        return Err(Piv1Error::InvalidCustodyObservation);
    }
    if supply == 0 { return Ok(0); } // units <= Mint.supply <= supply already proved.
    piv1_math::checked_mul_div_floor(units, total, supply).map_err(Into::into)
}

fn preflight_borrows(accounts: &[AccountInfo<'_>], movements: &[Movement]) -> ProgramResult {
    let mut mutable = [false; 19]; mutable[0] = true;
    for movement in movements { mutable[movement.source] = true; mutable[movement.destination] = true; }
    let mut data = Vec::with_capacity(6); let mut lamports = Vec::with_capacity(5);
    for (index, account) in accounts.iter().enumerate() {
        if mutable[index] {
            if !account.is_writable { return Err(error(Piv1Error::AccountNotWritable)); }
            data.push(account.try_borrow_mut_data().map_err(|_| error(Piv1Error::AccountBorrowFailed))?);
            lamports.push(account.try_borrow_mut_lamports().map_err(|_| error(Piv1Error::AccountBorrowFailed))?);
        }
    }
    if movements.iter().any(|m| m.source == 8) {
        data.push(accounts[MINT].try_borrow_mut_data().map_err(|_| error(Piv1Error::AccountBorrowFailed))?);
    }
    Ok(())
}
