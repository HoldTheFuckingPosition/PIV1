#![forbid(unsafe_code)]
#![allow(unexpected_cfgs)]
//! Synthetic validation caller, never the real Squads executable or governance.
//! Its only runtime purpose is a bounded signed CPI to the read-only probe.

use anchor_lang::{prelude::{AccountInfo, Pubkey}, solana_program::{
    entrypoint::ProgramResult, instruction::{AccountMeta, Instruction}, program_error::ProgramError}};
use piv1::{instructions::initialize::GenesisModelParameters,
    squads_accounts::{authenticate_squads_authority_snapshot, SquadsAuthorityAccountInfos, SQUADS_V4_PROGRAM_ID},
    squads_execution::{VAULT_TRANSACTION_DISCRIMINATOR, VAULT_TRANSACTION_EXECUTE_DISCRIMINATOR}};

pub const HOST_UNAVAILABLE: u32 = 0x2292;
const MAX_TRANSACTION_BYTES: usize = 1600;

/// Read-only prepared test CPI. This type is not a governance authorization.
pub struct PreparedCall {
    pub instruction: Instruction,
    pub multisig: Pubkey,
    pub vault_index: u8,
    pub vault_bump: u8,
}

struct Reader<'a>(&'a [u8]);
impl<'a> Reader<'a> {
    fn take(&mut self, count: usize) -> Result<&'a [u8], ProgramError> {
        let bytes = self.0.get(..count).ok_or(ProgramError::InvalidInstructionData)?;
        self.0 = &self.0[count..]; Ok(bytes)
    }
    fn array<const N: usize>(&mut self) -> Result<[u8; N], ProgramError> {
        self.take(N)?.try_into().map_err(|_| ProgramError::InvalidInstructionData)
    }
    fn byte(&mut self) -> Result<u8, ProgramError> { Ok(self.array::<1>()?[0]) }
    fn u32(&mut self) -> Result<usize, ProgramError> {
        usize::try_from(u32::from_le_bytes(self.array()?)).map_err(|_| ProgramError::InvalidInstructionData)
    }
    fn key(&mut self) -> Result<Pubkey, ProgramError> { Ok(Pubkey::new_from_array(self.array()?)) }
}

/// Decode only the exact stored single-action profile used by the probe. The
/// callee reauthenticates current approvals, Clock/Rent and all genesis facts.
pub fn prepare(program: &Pubkey, accounts: &[AccountInfo<'_>], data: &[u8]) -> Result<PreparedCall, ProgramError> {
    if *program != SQUADS_V4_PROGRAM_ID { return Err(ProgramError::IncorrectProgramId); }
    if data != VAULT_TRANSACTION_EXECUTE_DISCRIMINATOR { return Err(ProgramError::InvalidInstructionData); }
    if !(35..=36).contains(&accounts.len()) { return Err(ProgramError::InvalidArgument); }
    if !accounts[1].is_writable || !accounts[3].is_signer
        || accounts[2].owner != &SQUADS_V4_PROGRAM_ID || accounts[2].executable
    { return Err(ProgramError::InvalidAccountData); }
    let transaction = accounts[2].try_borrow_data()?;
    if transaction.len() > MAX_TRANSACTION_BYTES { return Err(ProgramError::InvalidInstructionData); }
    let mut reader = Reader(&transaction);
    if reader.array::<8>()? != VAULT_TRANSACTION_DISCRIMINATOR
        || reader.key()? != *accounts[0].key
    { return Err(ProgramError::InvalidInstructionData); }
    let _historical_creator = reader.key()?;
    let index = reader.array::<8>()?;
    let transaction_bump = reader.byte()?;
    let vault_index = reader.byte()?;
    let vault_bump = reader.byte()?;
    let (expected_transaction, expected_bump) = Pubkey::find_program_address(
        &[b"multisig", accounts[0].key.as_ref(), b"transaction", &index], &SQUADS_V4_PROGRAM_ID);
    if *accounts[2].key != expected_transaction || transaction_bump != expected_bump || reader.u32()? != 0 {
        return Err(ProgramError::InvalidInstructionData);
    }
    let signers = usize::from(reader.byte()?);
    let writable_signers = usize::from(reader.byte()?);
    let writable_unsigned = usize::from(reader.byte()?);
    let count = reader.u32()?;
    if !(31..=32).contains(&count) || accounts.len() != count + 4
        || signers > count || writable_signers > signers || writable_unsigned > count - signers
    { return Err(ProgramError::InvalidInstructionData); }
    let keys = reader.take(count * 32)?;
    let key = |index: usize| -> Result<Pubkey, ProgramError> {
        let start = index.checked_mul(32).ok_or(ProgramError::InvalidInstructionData)?;
        let bytes = keys.get(start..start + 32).ok_or(ProgramError::InvalidInstructionData)?;
        Ok(Pubkey::new_from_array(bytes.try_into().map_err(|_| ProgramError::InvalidInstructionData)?))
    };
    let flags = |index: usize| (index < signers,
        if index < signers { index < writable_signers } else { index - signers < writable_unsigned });
    for index in 0..count {
        if key(index)? != *accounts[4 + index].key { return Err(ProgramError::InvalidArgument); }
        for previous in 0..index {
            if key(index)? == key(previous)? { return Err(ProgramError::InvalidArgument); }
        }
    }
    if reader.u32()? != 1 { return Err(ProgramError::InvalidInstructionData); }
    let program_index = usize::from(reader.byte()?);
    let callee = key(program_index)?;
    if reader.u32()? != count { return Err(ProgramError::InvalidInstructionData); }
    let indices = reader.take(count)?;
    for (position, index) in indices.iter().enumerate() {
        if usize::from(*index) >= count || indices[..position].contains(index) {
            return Err(ProgramError::InvalidInstructionData);
        }
    }
    let model_length = reader.u32()?;
    let inner_data = reader.take(model_length)?;
    let model = GenesisModelParameters::decode(inner_data).map_err(|_| ProgramError::InvalidInstructionData)?;
    let expected_count = if model.protocol.manager_fee_account == model.protocol.referrer_token_account { 31 } else { 32 };
    if count != expected_count || model.vault_index != vault_index || reader.u32()? != 0 || !reader.0.is_empty() {
        return Err(ProgramError::InvalidInstructionData);
    }
    let at = |role: usize| &accounts[4 + usize::from(indices[role])];
    if *at(0).key != callee || at(2).key != accounts[0].key
        || at(3).key != accounts[1].key || at(4).key != accounts[2].key
    { return Err(ProgramError::InvalidArgument); }
    let authority = authenticate_squads_authority_snapshot(&callee, vault_index,
        SquadsAuthorityAccountInfos { program: at(0), program_data: at(1), multisig: at(2) })
        .map_err(|_| ProgramError::InvalidAccountData)?;
    let (vault, expected_vault_bump) = Pubkey::find_program_address(
        &[b"multisig", accounts[0].key.as_ref(), b"vault", &[vault_index]], &SQUADS_V4_PROGRAM_ID);
    if authority.vault() != vault || *at(5).key != vault || vault_bump != expected_vault_bump || at(5).is_signer {
        return Err(ProgramError::InvalidArgument);
    }
    let mut metas = Vec::with_capacity(count);
    for (role, index) in indices.iter().copied().enumerate() {
        let (signer, writable) = flags(usize::from(index));
        let account = at(role);
        if (signer && *account.key != vault && !account.is_signer) || (writable && !account.is_writable)
            || ((2..=4).contains(&role) && (signer || writable)) || (role == 5 && !signer)
        { return Err(ProgramError::InvalidArgument); }
        metas.push(AccountMeta { pubkey: *account.key, is_signer: signer, is_writable: writable });
    }
    Ok(PreparedCall { instruction: Instruction { program_id: callee, accounts: metas, data: inner_data.to_vec() },
        multisig: *accounts[0].key, vault_index, vault_bump })
}

pub fn process_instruction(program: &Pubkey, accounts: &[AccountInfo<'_>], data: &[u8]) -> ProgramResult {
    #[cfg(not(target_os = "solana"))]
    { let _ = (program, accounts, data); Err(ProgramError::Custom(HOST_UNAVAILABLE)) }
    #[cfg(target_os = "solana")]
    {
        let prepared = prepare(program, accounts, data)?;
        anchor_lang::solana_program::program::invoke_signed(&prepared.instruction, accounts,
            &[&[b"multisig", prepared.multisig.as_ref(), b"vault", &[prepared.vault_index], &[prepared.vault_bump]]])
    }
}

#[cfg(target_os = "solana")]
anchor_lang::solana_program::entrypoint!(process_instruction);
