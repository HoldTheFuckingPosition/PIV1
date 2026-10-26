#![forbid(unsafe_code)]
#![allow(unexpected_cfgs)]
//! Synthetic validation caller, never the real Squads executable or governance.
//! Its bounded runtime profiles call the production initializer, explicit pause, recipient update or one-seat guardian synchronization.
//! Historical Task 2.31 caller behavior is retained, with strict native version 2 address-only 313-byte
//! decoding; the complete original stored bytes are forwarded unchanged.

use anchor_lang::{prelude::{AccountInfo, Pubkey}, solana_program::{
    entrypoint::ProgramResult, instruction::{AccountMeta, Instruction}, program_error::ProgramError}};
use piv1::{instructions::{initialize::InitializePiv1Parameters, pause::SetPauseRequest, update_config::{UpdateRecipientsRequest, UpdateGuardianSetRequest}},
    squads_accounts::{authenticate_squads_authority_snapshot, SquadsAuthorityAccountInfos, SQUADS_V4_PROGRAM_ID},
    squads_execution::{VAULT_TRANSACTION_DISCRIMINATOR, VAULT_TRANSACTION_EXECUTE_DISCRIMINATOR}};

pub const HOST_UNAVAILABLE: u32 = 0x2312;
const MAX_TRANSACTION_BYTES: usize = 1600;

/// Read-only preparation for a bounded production CPI, not governance authorization.
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

/// Decode only the exact stored single-action production profile. The
/// callee reauthenticates current approvals, Clock/Rent and all genesis facts.
pub fn prepare(program: &Pubkey, accounts: &[AccountInfo<'_>], data: &[u8]) -> Result<PreparedCall, ProgramError> {
    if *program != SQUADS_V4_PROGRAM_ID { return Err(ProgramError::IncorrectProgramId); }
    if data != VAULT_TRANSACTION_EXECUTE_DISCRIMINATOR { return Err(ProgramError::InvalidInstructionData); }
    if accounts.len() != 20 && accounts.len() != 22 && accounts.len() != 28 && !(38..=39).contains(&accounts.len()) { return Err(ProgramError::InvalidArgument); }
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
    if (count != 16 && count != 18 && count != 24 && !(34..=35).contains(&count)) || accounts.len() != count + 4
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
    // Small profiles admit only strict pause, recipient-update or guardian-sync
    // ABIs; this cannot become a generic proxy. Initializer decoding is unchanged.
    let pause = count == 16;
    let recipient_update = count == 18;
    let guardian_update = count == 24;
    let governance_update = pause || recipient_update || guardian_update;
    let (expected_count, witness) = if pause {
        let request = SetPauseRequest::decode(inner_data)?;
        (16, request.vault_index)
    } else if recipient_update {
        let request = UpdateRecipientsRequest::decode(inner_data)?;
        (18, request.vault_index)
    } else if guardian_update {
        let request = UpdateGuardianSetRequest::decode(inner_data)?;
        (24, request.vault_index)
    } else {
        let model = InitializePiv1Parameters::decode(inner_data).map_err(|_| ProgramError::InvalidInstructionData)?.model;
        (if model.protocol.manager_fee_account == model.protocol.referrer_token_account { 34 } else { 35 }, model.vault_index)
    };
    if count != expected_count || witness != vault_index || reader.u32()? != 0 || !reader.0.is_empty() {
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
    let payer = if guardian_update { 22 } else { count - 5 };
    for (role, index) in indices.iter().copied().enumerate() {
        let (signer, writable) = flags(usize::from(index));
        let account = at(role);
        if (signer && *account.key != vault && !account.is_signer) || (writable && !account.is_writable)
            || signer != (role == 5 || ((!governance_update || guardian_update) && role == payer))
            || writable != (if guardian_update { role == 7 || role == 8 || (16..=22).contains(&role) } else if governance_update { role == 7 } else { (7..=22).contains(&role) || role == payer })
        { return Err(ProgramError::InvalidArgument); }
        metas.push(AccountMeta { pubkey: *account.key, is_signer: signer, is_writable: writable });
    }
    Ok(PreparedCall { instruction: Instruction { program_id: callee, accounts: metas, data: inner_data.to_vec() },
        multisig: *accounts[0].key, vault_index, vault_bump })
}

// Dedicated synthetic local signer path. It can call only the unchanged PIV1
// heartbeat/claim ABIs, never a caller-selected program or arbitrary instruction.
#[cfg(target_os = "solana")]
fn guardian_call(program: &Pubkey, accounts: &[AccountInfo<'_>], data: &[u8]) -> ProgramResult {
    use anchor_lang::solana_program::program::invoke_signed;
    const PIV: Pubkey = Pubkey::new_from_array([217; 32]);
    if *program != SQUADS_V4_PROGRAM_ID || data.get(..8) != Some(b"PIV1GP01") {
        return Err(ProgramError::InvalidInstructionData);
    }
    let inner = &data[8..];
    let (count, guardian) = if inner.get(..8) == Some(b"PIV1HB01") {
        piv1::instructions::guardian_heartbeat::GuardianHeartbeatRequest::decode(inner)?;
        (13, 9)
    } else if inner.get(..8) == Some([253,151,172,11,202,77,118,170].as_slice()) && inner.len() == 24 {
        (5, 3)
    } else { return Err(ProgramError::InvalidInstructionData); };
    if accounts.len() != count + 1 || accounts[count].key != &PIV || !accounts[count].executable {
        return Err(ProgramError::InvalidArgument);
    }
    let (expected, bump) = Pubkey::find_program_address(&[b"validation-guardian"], program);
    if accounts[guardian].key != &expected { return Err(ProgramError::InvalidArgument); }
    let metas = accounts[..count].iter().enumerate().map(|(i, account)| AccountMeta {
        pubkey: *account.key, is_writable: account.is_writable, is_signer: i == guardian,
    }).collect();
    invoke_signed(&Instruction { program_id: PIV, accounts: metas, data: inner.to_vec() }, accounts,
        &[&[b"validation-guardian", &[bump]]])
}

pub fn process_instruction(program: &Pubkey, accounts: &[AccountInfo<'_>], data: &[u8]) -> ProgramResult {
    #[cfg(not(target_os = "solana"))]
    { let _ = (program, accounts, data); Err(ProgramError::Custom(HOST_UNAVAILABLE)) }
    #[cfg(target_os = "solana")]
    {
        if data.get(..8) == Some(b"PIV1GP01") { return guardian_call(program, accounts, data); }
        let prepared = prepare(program, accounts, data)?;
        anchor_lang::solana_program::program::invoke_signed(&prepared.instruction, accounts,
            &[&[b"multisig", prepared.multisig.as_ref(), b"vault", &[prepared.vault_index], &[prepared.vault_bump]]])
    }
}

#[cfg(target_os = "solana")]
anchor_lang::solana_program::entrypoint!(process_instruction);
