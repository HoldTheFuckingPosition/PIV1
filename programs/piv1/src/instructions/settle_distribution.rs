//! Strict permissionless settlement; all amounts and recipients are authenticated.
use anchor_lang::solana_program::program_error::ProgramError;
pub const SETTLE_DISTRIBUTION_SELECTOR: [u8; 8] = *b"PIV1SD01";
pub fn decode_settle_distribution(data: &[u8]) -> Result<(), ProgramError> {
    if data.len() != 9 || data[..8] != SETTLE_DISTRIBUTION_SELECTOR || data[8] != 1 {
        return Err(ProgramError::InvalidInstructionData);
    }
    Ok(())
}
instruction_marker!(pub SettleDistribution);
