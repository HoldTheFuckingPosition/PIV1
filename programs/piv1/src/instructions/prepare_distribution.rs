//! Liquid-funded preparation only; delayed withdrawal fails closed before effects.
use anchor_lang::solana_program::program_error::ProgramError;
pub const PREPARE_DISTRIBUTION_SELECTOR: [u8; 8] = *b"PIV1PD01";
pub const PREPARE_DISTRIBUTION_DATA: [u8; 9] = [b'P', b'I', b'V', b'1', b'P', b'D', b'0', b'1', 1];
pub fn decode_prepare_distribution(data: &[u8]) -> Result<(), ProgramError> {
    if data != PREPARE_DISTRIBUTION_DATA { return Err(ProgramError::InvalidInstructionData); }
    Ok(())
}
instruction_marker!(pub PrepareDistribution);
