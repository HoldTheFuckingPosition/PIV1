//! Permissionless supported economic-surplus normalization. No amount, role or
//! valuation is caller-selected. Movement retains the accepted pause gates.

use anchor_lang::solana_program::program_error::ProgramError;
pub const RECONCILE_UNTRACKED_SELECTOR: [u8; 8] = *b"PIV1RB01";
pub const RECONCILE_UNTRACKED_DATA: [u8; 9] = [b'P', b'I', b'V', b'1', b'R', b'B', b'0', b'1', 1];
pub fn decode_reconcile_untracked(data: &[u8]) -> Result<(), ProgramError> {
    if data != RECONCILE_UNTRACKED_DATA { return Err(ProgramError::InvalidInstructionData); }
    Ok(())
}

instruction_marker!(pub ReconcileUntrackedBalances);
