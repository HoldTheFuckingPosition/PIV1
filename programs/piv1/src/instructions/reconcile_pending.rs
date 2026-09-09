//! Exact pending-recognition ABI; broad economic normalization stays separate.
use anchor_lang::solana_program::program_error::ProgramError;
instruction_marker!(pub ReconcilePendingContributions);
pub const RECONCILE_PENDING_DISCRIMINATOR: [u8; 8] = [225, 128, 29, 102, 157, 24, 172, 206];
pub const RECONCILE_PENDING_INSTRUCTION_SIZE: usize = 8;
pub const fn encode_reconcile_pending() -> [u8; 8] { RECONCILE_PENDING_DISCRIMINATOR }
pub fn decode_reconcile_pending(data: &[u8]) -> Result<(), ProgramError> {
    if data != RECONCILE_PENDING_DISCRIMINATOR { return Err(ProgramError::InvalidInstructionData); }
    Ok(())
}
