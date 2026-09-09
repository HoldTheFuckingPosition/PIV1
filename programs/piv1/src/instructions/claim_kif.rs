//! Exact claim ABI alongside the compatible historical unit marker.
//! Only the instruction boundary dispatches this ABI; policy stays in execution.

use anchor_lang::solana_program::program_error::ProgramError;
use crate::state::KifClaimRequest;

instruction_marker!(pub ClaimKif);

/// SHA-256("global:claim_kif")[..8], compatible with Anchor global instructions.
pub const CLAIM_KIF_DISCRIMINATOR: [u8; 8] = [253, 151, 172, 11, 202, 77, 118, 170];
pub const CLAIM_KIF_INSTRUCTION_SIZE: usize = 24;

/// Fixed discriminator, positive-policy-independent amount, then replay counter.
pub fn encode_claim_kif(request: KifClaimRequest) -> [u8; CLAIM_KIF_INSTRUCTION_SIZE] {
    let mut bytes = [0; CLAIM_KIF_INSTRUCTION_SIZE];
    bytes[..8].copy_from_slice(&CLAIM_KIF_DISCRIMINATOR);
    bytes[8..16].copy_from_slice(&request.amount_lamports.to_le_bytes());
    bytes[16..24].copy_from_slice(&request.expected_cumulative_claimed.to_le_bytes());
    bytes
}

/// Strict shape decoding only. Zero amounts and stale counters remain execution
/// errors; malformed data never reaches account access, Rent or execution.
pub fn decode_claim_kif(data: &[u8]) -> Result<KifClaimRequest, ProgramError> {
    let bytes: &[u8; CLAIM_KIF_INSTRUCTION_SIZE] = data.try_into()
        .map_err(|_| ProgramError::InvalidInstructionData)?;
    if bytes[..8] != CLAIM_KIF_DISCRIMINATOR { return Err(ProgramError::InvalidInstructionData); }
    let amount = bytes[8..16].try_into().map_err(|_| ProgramError::InvalidInstructionData)?;
    let counter = bytes[16..24].try_into().map_err(|_| ProgramError::InvalidInstructionData)?;
    Ok(KifClaimRequest { amount_lamports: u64::from_le_bytes(amount),
        expected_cumulative_claimed: u64::from_le_bytes(counter) })
}
