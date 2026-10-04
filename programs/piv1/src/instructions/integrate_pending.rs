//! Strict permissionless post-settlement integration; no caller-selected amounts.
use anchor_lang::solana_program::program_error::ProgramError;
pub const INTEGRATE_PENDING_SELECTOR: [u8; 8] = *b"PIV1IP01";
pub fn decode_integrate_pending(data: &[u8]) -> Result<(), ProgramError> {
    if data.len() != 9 || data[..8] != INTEGRATE_PENDING_SELECTOR || data[8] != 1 {
        return Err(ProgramError::InvalidInstructionData);
    }
    Ok(())
}
instruction_marker!(pub IntegratePending);
