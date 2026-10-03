//! Distinct liquid and active-source withdrawal preparation profiles.
//! Actual delayed withdrawal execution remains a separate boundary.
use anchor_lang::solana_program::program_error::ProgramError;
pub const PREPARE_DISTRIBUTION_SELECTOR: [u8; 8] = *b"PIV1PD01";
pub const PREPARE_DISTRIBUTION_DATA: [u8; 9] = [b'P', b'I', b'V', b'1', b'P', b'D', b'0', b'1', 1];
pub fn decode_prepare_distribution(data: &[u8]) -> Result<(), ProgramError> {
    if data != PREPARE_DISTRIBUTION_DATA { return Err(ProgramError::InvalidInstructionData); }
    Ok(())
}
instruction_marker!(pub PrepareDistribution);

/// Separate active-source withdrawal preparation; caller supplies only a checked
/// validator-list index, never a value, target, capacity or technical minimum.
pub const PREPARE_WITHDRAWAL_SELECTOR: [u8; 8] = *b"PIV1PW01";
pub fn decode_prepare_withdrawal(data: &[u8]) -> Result<u32, ProgramError> {
    if data.len() != 13 || data[..8] != PREPARE_WITHDRAWAL_SELECTOR || data[8] != 1 {
        return Err(ProgramError::InvalidInstructionData);
    }
    Ok(u32::from_le_bytes(data[9..13].try_into().map_err(|_| ProgramError::InvalidInstructionData)?))
}
