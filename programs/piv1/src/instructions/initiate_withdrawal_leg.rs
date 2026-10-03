//! Permissionless maximum-safe active-source initiation; no caller amount.
use anchor_lang::solana_program::program_error::ProgramError;
pub const INITIATE_WITHDRAWAL_LEG_SELECTOR:[u8;8]=*b"PIV1IL01";
pub fn decode_initiate_withdrawal_leg(data:&[u8])->Result<u32,ProgramError>{
    if data.len()!=13||data[..8]!=INITIATE_WITHDRAWAL_LEG_SELECTOR||data[8]!=1{
        return Err(ProgramError::InvalidInstructionData);
    }
    Ok(u32::from_le_bytes(data[9..13].try_into().map_err(|_|ProgramError::InvalidInstructionData)?))
}
instruction_marker!(pub InitiateWithdrawalLeg);
