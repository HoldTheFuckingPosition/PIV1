//! Permissionless inactive-leg finalization; amounts and destinations are derived.
use anchor_lang::solana_program::program_error::ProgramError;

pub const FINALIZE_WITHDRAWAL_LEG_SELECTOR: [u8; 8] = *b"PIV1FL01";

pub fn decode_finalize_withdrawal_leg(data: &[u8]) -> Result<u64, ProgramError> {
    if data.len() != 17 || data[..8] != FINALIZE_WITHDRAWAL_LEG_SELECTOR || data[8] != 1 {
        return Err(ProgramError::InvalidInstructionData);
    }
    Ok(u64::from_le_bytes(data[9..17].try_into()
        .map_err(|_| ProgramError::InvalidInstructionData)?))
}

instruction_marker!(pub FinalizeWithdrawalLeg);
