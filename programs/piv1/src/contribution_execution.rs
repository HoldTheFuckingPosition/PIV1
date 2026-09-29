//! Fixed external SOL and owner-signed legacy JitoSOL intake. Both old pending
//! obligations are covered before transfer; only the requested increment is
//! recorded. CPI/postcheck errors require transaction rollback, not compensation.

use std::rc::Rc;
use anchor_lang::{prelude::{AccountInfo, Pubkey, Rent, SolanaSysvar}, solana_program::{
    entrypoint::ProgramResult, instruction::Instruction,
    program::invoke, program_error::ProgramError, program_option::COption,
    program_pack::Pack, system_instruction, system_program,
}};
use spl_token::state::{Account as TokenAccount, AccountState, Mint, Multisig};
use solana_sha256_hasher::hash;
use crate::{accounts::rent_floor, errors::Piv1Error,
    events::{JitoSolContribution, SolContribution},
    instruction_errors::{piv1_error_code, HOST_RUNTIME_UNAVAILABLE_CODE},
    instructions::{deposit_sol::{DepositSolRequest, DEPOSIT_SOL_SELECTOR},
        deposit_jitosol::{DepositJitoSolRequest, DEPOSIT_JITOSOL_SELECTOR}},
    pending_accounts::{authenticate_pending_accounts, PendingAccountInfos},
    state::{record_explicit_jitosol_contribution, record_explicit_sol_contribution,
        JitoSolCustodyObservation, SolCustodyObservation, PivConfig,
        reconciliation::expected_pending_sol_lamports},
    state_persistence::{commit_state_writes, PreparedStateWrite, StateEnvelope},
};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ContributionEvent { Sol(SolContribution), JitoSol(JitoSolContribution) }

/// Native entry obtains actual Rent and uses the canonical System/Token CPI.
/// Ordinary host stubs cannot execute this path.
pub fn process_instruction(program: &Pubkey, accounts: &[AccountInfo<'_>], data: &[u8]) -> ProgramResult {
    dispatch(program, accounts, data, cfg!(target_os = "solana"), Rent::get,
        |instruction, infos| invoke(instruction, infos), |event| match event {
            ContributionEvent::Sol(event) => anchor_lang::emit!(event),
            ContributionEvent::JitoSol(event) => anchor_lang::emit!(event),
        })
}

/// Explicit host-only modeling seam; callbacks neither authenticate a real
/// signature nor supply runtime rollback. A failed staged execution must be
/// discarded by its host transaction model. No signer seeds are used.
#[cfg(not(target_os = "solana"))]
pub fn process_instruction_with_host_callbacks<'info>(
    program: &Pubkey, accounts: &[AccountInfo<'info>], data: &[u8],
    get_rent: impl FnOnce() -> Result<Rent, ProgramError>,
    invoker: impl FnOnce(&Instruction, &[AccountInfo<'info>]) -> ProgramResult,
    emit: impl FnOnce(ContributionEvent),
) -> ProgramResult { dispatch(program, accounts, data, true, get_rent, invoker, emit) }

#[derive(Clone, Copy)]
enum Asset { Sol, JitoSol }
impl Asset { fn destination(self) -> usize { match self { Self::Sol => 2, Self::JitoSol => 3 } } }

fn error(value: Piv1Error) -> ProgramError { ProgramError::Custom(piv1_error_code(value)) }
fn arithmetic() -> ProgramError { error(Piv1Error::ArithmeticOverflow) }

#[allow(clippy::too_many_arguments)]
fn dispatch<'info>(program: &Pubkey, accounts: &[AccountInfo<'info>], data: &[u8], available: bool,
    get_rent: impl FnOnce() -> Result<Rent, ProgramError>,
    invoker: impl FnOnce(&Instruction, &[AccountInfo<'info>]) -> ProgramResult,
    emit: impl FnOnce(ContributionEvent),
) -> ProgramResult {
    let (asset, amount, count) = if data.get(..8) == Some(DEPOSIT_SOL_SELECTOR.as_slice()) {
        (Asset::Sol, DepositSolRequest::decode(data)?.amount_lamports, 6)
    } else if data.get(..8) == Some(DEPOSIT_JITOSOL_SELECTOR.as_slice()) {
        (Asset::JitoSol, DepositJitoSolRequest::decode(data)?.amount_units, 8)
    } else { return Err(ProgramError::InvalidInstructionData); };
    if accounts.len() < count { return Err(ProgramError::NotEnoughAccountKeys); }
    if accounts.len() > count { return Err(ProgramError::InvalidArgument); }
    if !available { return Err(ProgramError::Custom(HOST_RUNTIME_UNAVAILABLE_CODE)); }
    let rent = get_rent()?;
    distinct_accounts(accounts)?;
    let prepared = prepare(program, &rent, accounts, asset, amount)?;
    // Acquire all buffers that the fixed CPI and final commit will need before
    // any effect. SPL TransferChecked borrows even its readonly Mint mutably.
    preflight_borrows(accounts, asset)?;
    let infos = match asset {
        Asset::Sol => vec![accounts[4].clone(), accounts[2].clone(), accounts[5].clone()],
        Asset::JitoSol => vec![accounts[4].clone(), accounts[6].clone(), accounts[3].clone(),
            accounts[5].clone(), accounts[7].clone()],
    };
    invoker(&prepared.instruction, &infos)?;
    for (account, expected) in accounts.iter().zip(&prepared.after_transfer) {
        if &AccountRecord::read(account)? != expected {
            return Err(error(Piv1Error::ContributionObservationMismatch));
        }
    }
    // Reauthenticate the fixed pending accounts after the external effect, with
    // the unchanged old Config still present. The staged increment is committed
    // only after all exact metadata/byte/balance checks have passed.
    authenticate_pending_accounts(program, &rent, roles(accounts)).map_err(error)?;
    commit_state_writes(program, &rent, [(&prepared.write, &accounts[0])]).map_err(error)?;
    emit(prepared.event); Ok(())
}

fn roles<'a, 'info>(accounts: &'a [AccountInfo<'info>]) -> PendingAccountInfos<'a, 'info> {
    PendingAccountInfos { config: &accounts[0], active_distribution: &accounts[1],
        pending_sol: &accounts[2], pending_jito: &accounts[3] }
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

// Metadata and SHA-256 cover every supplied byte, without copying unbounded
// executable/authority buffers onto the SBF heap. Only authenticated 165-byte
// token buffers are copied when predicting their exact transfer replacements.
#[derive(Debug, Eq, PartialEq)]
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
struct PreparedContribution {
    instruction: Instruction, after_transfer: Vec<AccountRecord>,
    write: PreparedStateWrite, event: ContributionEvent,
}

#[inline(never)]
fn prepare(program: &Pubkey, rent: &Rent, accounts: &[AccountInfo<'_>], asset: Asset, amount: u64)
    -> Result<Box<PreparedContribution>, ProgramError>
{
    let authenticated = authenticate_pending_accounts(program, rent, roles(accounts)).map_err(error)?;
    let config = authenticated.config(); let observation = authenticated.observation();
    if config.paused { return Err(error(Piv1Error::PausedOperation)); }
    if amount == 0 { return Err(error(Piv1Error::ZeroContribution)); }
    let spendable = observation.pending_sol_vault_lamports
        .checked_sub(observation.pending_sol_non_economic_floor_lamports).ok_or(arithmetic())?;
    if expected_pending_sol_lamports(config, authenticated.distribution()).map_err(error)? > spendable
        || config.accounted_pending_jitosol_units > observation.pending_jitosol_token_units {
        return Err(error(Piv1Error::PendingCustodyDeficit));
    }
    for index in [0, 4, asset.destination()] {
        if !accounts[index].is_writable { return Err(error(Piv1Error::AccountNotWritable)); }
    }
    let signer = match asset { Asset::Sol => &accounts[4], Asset::JitoSol => &accounts[5] };
    if !signer.is_signer { return Err(ProgramError::MissingRequiredSignature); }
    if accounts[4].owner == program || signer.owner == program
        || internal_key(config, program, accounts[0].key, accounts[4].key)
        || internal_key(config, program, accounts[0].key, signer.key) {
        return Err(error(Piv1Error::InvalidAccountOwner));
    }
    let mut records: Vec<_> = accounts.iter().map(AccountRecord::read).collect::<Result<_, _>>()?;
    let mut next = Box::new(config.clone());
    let before = StateEnvelope::config(config).map_err(error)?;
    let (instruction, event) = match asset {
        Asset::Sol => {
            canonical_program(&accounts[5], &system_program::ID)?;
            if accounts[4].owner != &system_program::ID { return Err(error(Piv1Error::InvalidAccountOwner)); }
            if accounts[4].executable { return Err(error(Piv1Error::ExecutableAccount)); }
            if records[4].data_len != 0 { return Err(error(Piv1Error::InvalidAccountData)); }
            records[4].lamports = records[4].lamports.checked_sub(amount).ok_or(ProgramError::InsufficientFunds)?;
            records[2].lamports = records[2].lamports.checked_add(amount).ok_or(arithmetic())?;
            let result = record_explicit_sol_contribution(&mut next, authenticated.distribution(), amount,
                SolCustodyObservation { vault_lamports_before: observation.pending_sol_vault_lamports,
                    vault_lamports_after: records[2].lamports,
                    non_economic_floor_lamports: observation.pending_sol_non_economic_floor_lamports }).map_err(error)?;
            (system_instruction::transfer(accounts[4].key, accounts[2].key, amount),
                ContributionEvent::Sol(SolContribution { config: *accounts[0].key, donor: *signer.key,
                    amount_lamports: amount, pending_lamports_after: result.accounted_pending_after }))
        },
        Asset::JitoSol => {
            canonical_program(&accounts[7], &spl_token::ID)?;
            let (decimals, source_hash, destination_hash) = token_transfer(rent, accounts, config, amount)?;
            records[4].data_hash = source_hash; records[3].data_hash = destination_hash;
            let after_units = observation.pending_jitosol_token_units.checked_add(amount).ok_or(arithmetic())?;
            let result = record_explicit_jitosol_contribution(&mut next, authenticated.distribution(), amount,
                JitoSolCustodyObservation { token_units_before: observation.pending_jitosol_token_units,
                    token_units_after: after_units }).map_err(error)?;
            (spl_token::instruction::transfer_checked(&spl_token::ID, accounts[4].key, accounts[6].key,
                accounts[3].key, signer.key, &[], amount, decimals)?,
                ContributionEvent::JitoSol(JitoSolContribution { config: *accounts[0].key,
                    source: *accounts[4].key, owner: *signer.key, amount_units: amount,
                    pending_units_after: result.accounted_pending_after }))
        },
    };
    let write = PreparedStateWrite::new(program, *accounts[0].key, before,
        StateEnvelope::config(&next).map_err(error)?).map_err(error)?;
    Ok(Box::new(PreparedContribution { instruction, after_transfer: records, write, event }))
}

fn canonical_program(account: &AccountInfo<'_>, expected: &Pubkey) -> ProgramResult {
    if account.key != expected || !account.executable { return Err(error(Piv1Error::InvalidProgramIdentity)); }
    Ok(())
}
fn internal_key(config: &PivConfig, program: &Pubkey, config_key: &Pubkey, key: &Pubkey) -> bool {
    [*program, *config_key, config.piv_authority, config.active_distribution, config.guardian_registry,
        config.principal_jito_vault, config.pending_jito_vault, config.pending_sol_vault,
        config.principal_sol_queue, config.operational_sol_vault, config.distribution_escrow,
        config.kif_sol_vault].contains(key)
}

#[inline(never)]
fn token_transfer(rent: &Rent, accounts: &[AccountInfo<'_>], config: &PivConfig, amount: u64)
    -> Result<(u8, [u8; 32], [u8; 32]), ProgramError>
{
    let source = &accounts[4]; let authority = &accounts[5]; let mint_account = &accounts[6];
    for (account, length) in [(source, TokenAccount::LEN), (mint_account, Mint::LEN)] {
        if account.owner != &spl_token::ID { return Err(error(Piv1Error::InvalidAccountOwner)); }
        if account.executable { return Err(error(Piv1Error::ExecutableAccount)); }
        if account.data_len() != length { return Err(error(Piv1Error::InvalidAccountSize)); }
        if **account.try_borrow_lamports().map_err(|_| error(Piv1Error::AccountBorrowFailed))? < rent_floor(rent, length).map_err(error)? {
            return Err(error(Piv1Error::AccountRentDeficit));
        }
    }
    if mint_account.key != &config.jitosol_mint { return Err(error(Piv1Error::InvalidTokenCustody)); }
    // A signer account of arbitrary owner is supported, except the exact SPL
    // multisig representation that selects a different authorization protocol.
    if authority.owner == &spl_token::ID && authority.data_len() == Multisig::LEN {
        return Err(error(Piv1Error::InvalidTokenCustody));
    }
    let mint_data = mint_account.try_borrow_data().map_err(|_| error(Piv1Error::AccountBorrowFailed))?;
    let mint = Mint::unpack(&mint_data).map_err(|_| error(Piv1Error::InvalidTokenCustody))?;
    let mut source_data = source.try_borrow_data().map_err(|_| error(Piv1Error::AccountBorrowFailed))?.to_vec();
    let token = TokenAccount::unpack(&source_data).map_err(|_| error(Piv1Error::InvalidTokenCustody))?;
    if token.mint != config.jitosol_mint || token.owner != *authority.key
        || token.state != AccountState::Initialized || token.is_native != COption::None {
        return Err(error(Piv1Error::InvalidTokenCustody));
    }
    let remaining = token.amount.checked_sub(amount).ok_or(ProgramError::InsufficientFunds)?;
    source_data[64..72].copy_from_slice(&remaining.to_le_bytes());
    // SPL 8 chooses the matching delegate branch before the owner branch, even
    // when delegate == owner. Preserve opaque None payloads as its packer does.
    if token.delegate == COption::Some(*authority.key) {
        let allowance = token.delegated_amount.checked_sub(amount).ok_or(ProgramError::InsufficientFunds)?;
        source_data[121..129].copy_from_slice(&allowance.to_le_bytes());
        if allowance == 0 { source_data[72..76].fill(0); }
    }
    let mut destination = accounts[3].try_borrow_data().map_err(|_| error(Piv1Error::AccountBorrowFailed))?.to_vec();
    let units = u64::from_le_bytes(destination[64..72].try_into().map_err(|_| error(Piv1Error::InvalidAccountSize))?);
    destination[64..72].copy_from_slice(&units.checked_add(amount).ok_or(arithmetic())?.to_le_bytes());
    Ok((mint.decimals, hash(&source_data).to_bytes(), hash(&destination).to_bytes()))
}

fn preflight_borrows(accounts: &[AccountInfo<'_>], asset: Asset) -> ProgramResult {
    let indices: &[usize] = match asset { Asset::Sol => &[0, 4, 2], Asset::JitoSol => &[0, 4, 3, 6] };
    let mut data = Vec::with_capacity(indices.len()); let mut lamports = Vec::with_capacity(indices.len());
    for &index in indices {
        data.push(accounts[index].try_borrow_mut_data().map_err(|_| error(Piv1Error::AccountBorrowFailed))?);
        lamports.push(accounts[index].try_borrow_mut_lamports().map_err(|_| error(Piv1Error::AccountBorrowFailed))?);
    }
    Ok(())
}
