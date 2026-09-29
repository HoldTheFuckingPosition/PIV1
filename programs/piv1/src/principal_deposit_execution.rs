//! One protected SPL/Jito deposit of already recognized historical SOL.
//! Prevalidated zero-fee/book-value preservation is coupled to exact CPI effects;
//! pending assets, HWM, rent, carry and Token-native quarantine never fund loss.
use std::rc::Rc;
use anchor_lang::{prelude::{AccountInfo, Clock, Pubkey, Rent, SolanaSysvar}, solana_program::{
    entrypoint::ProgramResult, instruction::Instruction, program::invoke_signed,
    program_error::ProgramError, program_pack::Pack, system_program}};
use solana_sha256_hasher::{hash, hashv};
use spl_token::state::Multisig;
use crate::{accounts::{authenticate_fixed_accounts, rent_floor, seeds, FixedAccountInfos},
    errors::Piv1Error, events::PendingSolStaked,
    instruction_errors::{piv1_error_code, protocol_program_error, HOST_RUNTIME_UNAVAILABLE_CODE},
    instructions::stake_pending_sol::StakePendingSolParameters,
    integrations::{jito_deposit::{self, NativeDeposit}, jito_identity::{authenticate_jito_identity,
        AuthenticatedJitoIdentity, DeclaredJitoKeys, JitoIdentityAccountInfos}},
    state::{principal_deposit::record_with_profile, PivConfig},
    state_persistence::{commit_state_writes, PreparedStateWrite, StateEnvelope}};

const AUTHORITY: usize = 9;
const MINT: usize = 10;
const SYSTEM: usize = 11;
const TOKEN: usize = 12;
const PROTOCOL: usize = 13;
const POOL: usize = 14;
const LIST: usize = 15;
const RESERVE: usize = 16;
const MANAGER: usize = 17;

pub fn process_instruction(program: &Pubkey, accounts: &[AccountInfo<'_>], data: &[u8]) -> ProgramResult {
    dispatch(program, accounts, data, cfg!(target_os = "solana"), Clock::get, Rent::get,
        |ix, a, seeds| invoke_signed(ix, a, seeds), |event| anchor_lang::emit!(event))
}
/// Explicit host effects seam. Partial failed CPI effects remain visible until
/// the test discards its staged world; this is not VM execution or rollback.
#[cfg(not(target_os = "solana"))]
pub fn process_instruction_with_host_callbacks<'info>(
    program: &Pubkey, accounts: &[AccountInfo<'info>], data: &[u8],
    get_clock: impl FnOnce() -> Result<Clock, ProgramError>, get_rent: impl FnOnce() -> Result<Rent, ProgramError>,
    invoke: impl FnOnce(&Instruction, &[AccountInfo<'info>], &[&[&[u8]]]) -> ProgramResult,
    emit: impl FnOnce(PendingSolStaked),
) -> ProgramResult { dispatch(program, accounts, data, true, get_clock, get_rent, invoke, emit) }
fn error(value: Piv1Error) -> ProgramError { ProgramError::Custom(piv1_error_code(value)) }
fn overflow() -> ProgramError { error(Piv1Error::ArithmeticOverflow) }

#[allow(clippy::too_many_arguments)]
fn dispatch<'info>(program: &Pubkey, a: &[AccountInfo<'info>], data: &[u8], available: bool,
    get_clock: impl FnOnce() -> Result<Clock, ProgramError>, get_rent: impl FnOnce() -> Result<Rent, ProgramError>,
    invoke: impl FnOnce(&Instruction, &[AccountInfo<'info>], &[&[&[u8]]]) -> ProgramResult,
    emit: impl FnOnce(PendingSolStaked),
) -> ProgramResult {
    let request = StakePendingSolParameters::decode(data)?;
    if a.len() < 19 { return Err(ProgramError::NotEnoughAccountKeys); }
    if a.len() > 20 { return Err(ProgramError::InvalidArgument); }
    if !available { return Err(ProgramError::Custom(HOST_RUNTIME_UNAVAILABLE_CODE)); }
    let clock = get_clock()?; let rent = get_rent()?; distinct(a)?;
    let prepared = prepare(program, &clock, &rent, a, request)?;
    preflight_borrows(a, prepared.referrer)?;
    let infos = [POOL, a.len() - 1, RESERVE, 3, 7, MANAGER, prepared.referrer, MINT, SYSTEM, TOKEN, PROTOCOL]
        .into_iter().map(|index| a[index].clone()).collect::<Vec<_>>();
    let bump = [prepared.bump]; let signer = [seeds::PRINCIPAL_SOL, &bump];
    invoke(&prepared.instruction, &infos, &[&signer])?;
    for (account, expected) in a.iter().zip(&prepared.expected) {
        if AccountRecord::read(account)? != *expected { return Err(error(Piv1Error::PrincipalDepositObservationMismatch)); }
    }
    // Old historical fields still describe pre-CPI custody until commit. Only
    // metadata/rent may be reauthenticated against that old Config here.
    let authenticated = authenticate_fixed_accounts(program, &rent, roles(a)).map_err(error)?;
    let _ = protocol_identity(authenticated.config(), a, prepared.referrer)?;
    commit_state_writes(program, &rent, [(&prepared.write, &a[0])]).map_err(error)?;
    emit(prepared.event); Ok(())
}
fn roles<'a, 'info>(a: &'a [AccountInfo<'info>]) -> FixedAccountInfos<'a, 'info> {
    FixedAccountInfos { config: &a[0], active_distribution: &a[1], pending_sol: &a[2], principal_sol: &a[3],
        operational_sol: &a[4], distribution_escrow: &a[5], kif_sol: &a[6], principal_jito: &a[7], pending_jito: &a[8] }
}
fn distinct(a: &[AccountInfo<'_>]) -> ProgramResult {
    for (index, account) in a.iter().enumerate() {
        if a[index + 1..].iter().any(|other| account.key == other.key
            || Rc::ptr_eq(&account.data, &other.data) || Rc::ptr_eq(&account.lamports, &other.lamports)) {
            return Err(error(Piv1Error::AccountAlias));
        }
    }
    Ok(())
}
#[derive(Clone, Debug, Eq, PartialEq)]
struct AccountRecord {
    key: Pubkey, owner: Pubkey, signer: bool, writable: bool, executable: bool,
    rent_epoch: u64, lamports: u64, data_len: usize, data_hash: [u8; 32],
}
impl AccountRecord {
    fn read(a: &AccountInfo<'_>) -> Result<Self, ProgramError> {
        let data = a.try_borrow_data().map_err(|_| error(Piv1Error::AccountBorrowFailed))?;
        Ok(Self { key: *a.key, owner: *a.owner, signer: a.is_signer, writable: a.is_writable,
            executable: a.executable, rent_epoch: a.rent_epoch,
            lamports: **a.try_borrow_lamports().map_err(|_| error(Piv1Error::AccountBorrowFailed))?,
            data_len: data.len(), data_hash: hash(&data).to_bytes() })
    }
}
/// Hash an exact patch without copying a caller-sized pool allocation or tail.
fn patched_hash(a: &AccountInfo<'_>, offset: usize, replacement: &[u8]) -> Result<[u8; 32], ProgramError> {
    let data = a.try_borrow_data().map_err(|_| error(Piv1Error::AccountBorrowFailed))?;
    let end = offset.checked_add(replacement.len()).ok_or(overflow())?;
    if end > data.len() { return Err(error(Piv1Error::InvalidAccountSize)); }
    Ok(hashv(&[&data[..offset], replacement, &data[end..]]).to_bytes())
}
struct PreparedDeposit {
    instruction: Instruction, expected: Vec<AccountRecord>, referrer: usize, bump: u8,
    write: PreparedStateWrite, event: PendingSolStaked,
}
#[inline(never)]
fn prepare(program: &Pubkey, clock: &Clock, rent: &Rent, a: &[AccountInfo<'_>], request: StakePendingSolParameters)
    -> Result<Box<PreparedDeposit>, ProgramError>
{
    let authenticated = authenticate_fixed_accounts(program, rent, roles(a)).map_err(error)?;
    let c = authenticated.config(); let shared = c.manager_fee_account == c.referrer_token_account;
    let count = if shared { 19 } else { 20 };
    if a.len() < count { return Err(ProgramError::NotEnoughAccountKeys); }
    if a.len() > count { return Err(ProgramError::InvalidArgument); }
    let referrer = if shared { MANAGER } else { 18 };
    let before = authenticated.principal_deposit_observation().map_err(error)?;
    let protocol = protocol_identity(c, a, referrer)?;
    validate_extra_accounts(c, &protocol, rent, a, referrer)?;
    let profile = NativeDeposit { protocol: &protocol, epoch: clock.epoch, amount: request.native_lamports,
        minimum: request.caller_minimum_pool_tokens_out, slippage: c.configured_slippage_bps };
    let minted = profile.minted().map_err(error)?;
    let mut after = before;
    after.principal_sol.lamports = after.principal_sol.lamports.checked_sub(request.native_lamports)
        .ok_or(error(Piv1Error::PrincipalDepositExceedsQueue))?;
    after.principal_jitosol_units = after.principal_jitosol_units.checked_add(minted).ok_or(overflow())?;
    let mut next = Box::new(c.clone());
    let result = record_with_profile(&mut next, authenticated.distribution(), before, after, &profile).map_err(error)?;
    let minimum = profile.protected_output(result.minted_jitosol_units).map_err(error)?;
    let (total_after, supply_after, mint_after) = profile.after().map_err(error)?;
    let mut expected: Vec<_> = a.iter().map(AccountRecord::read).collect::<Result<_, _>>()?;
    expected[3].lamports = after.principal_sol.lamports;
    expected[RESERVE].lamports = expected[RESERVE].lamports.checked_add(request.native_lamports).ok_or(overflow())?;
    expected[7].data_hash = patched_hash(&a[7], 64, &after.principal_jitosol_units.to_le_bytes())?;
    expected[MINT].data_hash = patched_hash(&a[MINT], 36, &mint_after.to_le_bytes())?;
    // The pinned Borsh prefix has eight public keys and one bump after its tag;
    // total_lamports/supply occupy258..274, before every variable-size Option.
    let mut balances = [0; 16]; balances[..8].copy_from_slice(&total_after.to_le_bytes());
    balances[8..].copy_from_slice(&supply_after.to_le_bytes());
    expected[POOL].data_hash = patched_hash(&a[POOL], 258, &balances)?;
    let instruction = jito_deposit::instruction(*a[PROTOCOL].key, *a[POOL].key, *a[a.len()-1].key,
        *a[RESERVE].key, *a[3].key, *a[7].key, *a[MANAGER].key, *a[referrer].key, *a[MINT].key,
        request.native_lamports, minimum);
    let write = PreparedStateWrite::new(program, *a[0].key, StateEnvelope::config(c).map_err(error)?,
        StateEnvelope::config(&next).map_err(error)?).map_err(error)?;
    Ok(Box::new(PreparedDeposit { instruction, expected, referrer, bump: c.bumps.principal_sol_queue,
        write, event: PendingSolStaked { config: *a[0].key, deposited_sol_lamports: result.deposited_sol_lamports,
            minted_jitosol_units: result.minted_jitosol_units, historical_value_before_lamports: result.historical_value_before_lamports,
            historical_value_after_lamports: result.historical_value_after_lamports } }))
}
#[inline(never)]
fn protocol_identity(c: &PivConfig, a: &[AccountInfo<'_>], referrer: usize) -> Result<Box<AuthenticatedJitoIdentity>, ProgramError> {
    authenticate_jito_identity(&DeclaredJitoKeys { program: c.stake_pool_program, pool: c.stake_pool,
        validator_list: c.validator_list, reserve: c.reserve_stake, mint: c.jitosol_mint,
        manager_fee: c.manager_fee_account, referrer: c.referrer_token_account },
        JitoIdentityAccountInfos { program: &a[PROTOCOL], pool: &a[POOL], validator_list: &a[LIST],
            reserve: &a[RESERVE], mint: &a[MINT], manager_fee: &a[MANAGER], referrer: &a[referrer] })
        .map(Box::new).map_err(protocol_program_error)
}
fn validate_extra_accounts(c: &PivConfig, protocol: &AuthenticatedJitoIdentity, rent: &Rent,
    a: &[AccountInfo<'_>], referrer: usize) -> ProgramResult
{
    for (index, id) in [(SYSTEM, system_program::ID), (TOKEN, spl_token::ID)] {
        if a[index].key != &id || !a[index].executable { return Err(error(Piv1Error::InvalidProgramIdentity)); }
    }
    if a[AUTHORITY].key != &c.piv_authority { return Err(error(Piv1Error::InvalidAccountPda)); }
    if a[AUTHORITY].owner != &system_program::ID { return Err(error(Piv1Error::InvalidAccountOwner)); }
    if a[AUTHORITY].executable { return Err(error(Piv1Error::ExecutableAccount)); }
    if !a[AUTHORITY].try_borrow_data().map_err(|_| error(Piv1Error::AccountBorrowFailed))?.is_empty() {
        return Err(error(Piv1Error::InvalidAccountData));
    }
    let withdraw = &a[a.len()-1];
    if withdraw.key != &protocol.withdraw_authority() { return Err(error(Piv1Error::InvalidAccountPda)); }
    if withdraw.executable { return Err(error(Piv1Error::ExecutableAccount)); }
    // Token chooses multisig mode before ordinary PDA signer validation.
    let withdraw_data_len = withdraw.try_borrow_data().map_err(|_| error(Piv1Error::AccountBorrowFailed))?.len();
    if withdraw.owner == &spl_token::ID && withdraw_data_len == Multisig::LEN {
        return Err(error(Piv1Error::InvalidAccountData));
    }
    for index in [MINT, POOL, LIST, RESERVE, MANAGER, referrer] {
        let floor = rent_floor(rent, a[index].data_len()).map_err(error)?;
        if **a[index].try_borrow_lamports().map_err(|_| error(Piv1Error::AccountBorrowFailed))? < floor {
            return Err(error(Piv1Error::AccountRentDeficit));
        }
    }
    Ok(())
}
fn preflight_borrows(a: &[AccountInfo<'_>], referrer: usize) -> ProgramResult {
    let mut required = [false; 20];
    for index in [0, 3, 7, MINT, POOL, RESERVE, MANAGER, referrer] { required[index] = true; }
    let mut data = Vec::with_capacity(8); let mut lamports = Vec::with_capacity(8);
    for (index, account) in a.iter().enumerate() { if required[index] {
        if !account.is_writable { return Err(error(Piv1Error::AccountNotWritable)); }
        data.push(account.try_borrow_mut_data().map_err(|_| error(Piv1Error::AccountBorrowFailed))?);
        lamports.push(account.try_borrow_mut_lamports().map_err(|_| error(Piv1Error::AccountBorrowFailed))?);
    } }
    Ok(())
}
