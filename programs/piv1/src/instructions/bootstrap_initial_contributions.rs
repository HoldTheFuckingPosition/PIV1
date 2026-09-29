//! Initial-only pending-to-principal boundary; no amounts or values in the ABI.
use anchor_lang::solana_program::program_error::ProgramError;
pub const INITIAL_BOOTSTRAP_SELECTOR: [u8; 8] = *b"PIV1IB01";
pub const INITIAL_BOOTSTRAP_DATA: [u8; 9] = [b'P', b'I', b'V', b'1', b'I', b'B', b'0', b'1', 1];
pub fn decode_initial_bootstrap(data: &[u8]) -> Result<(), ProgramError> {
    if data != INITIAL_BOOTSTRAP_DATA { return Err(ProgramError::InvalidInstructionData); }
    Ok(())
}
instruction_marker!(pub BootstrapInitialContributions);
