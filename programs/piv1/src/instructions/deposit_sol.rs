//! Exact versioned explicit SOL intake. A contribution creates no ownership,
//! withdrawal or reward right. D-032 rejects intake during emergency pause.

use anchor_lang::solana_program::program_error::ProgramError;

pub const DEPOSIT_SOL_SELECTOR: [u8; 8] = *b"PIV1DS01";
pub const DEPOSIT_DATA_LEN: usize = 17;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct DepositSolRequest { pub amount_lamports: u64 }
impl DepositSolRequest {
    pub fn decode(data: &[u8]) -> Result<Self, ProgramError> {
        Ok(Self { amount_lamports: decode_amount(data, &DEPOSIT_SOL_SELECTOR)? })
    }
    pub fn encode(self) -> [u8; DEPOSIT_DATA_LEN] { encode_amount(&DEPOSIT_SOL_SELECTOR, self.amount_lamports) }
}

pub(crate) fn decode_amount(data: &[u8], selector: &[u8; 8]) -> Result<u64, ProgramError> {
    if data.len() != DEPOSIT_DATA_LEN || &data[..8] != selector || data[8] != 1 {
        return Err(ProgramError::InvalidInstructionData);
    }
    Ok(u64::from_le_bytes(data[9..17].try_into().map_err(|_| ProgramError::InvalidInstructionData)?))
}
pub(crate) fn encode_amount(selector: &[u8; 8], amount: u64) -> [u8; DEPOSIT_DATA_LEN] {
    let mut data = [0; DEPOSIT_DATA_LEN]; data[..8].copy_from_slice(selector);
    data[8] = 1; data[9..].copy_from_slice(&amount.to_le_bytes()); data
}

instruction_marker!(pub DepositSol);
