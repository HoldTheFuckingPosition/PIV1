//! Protected historical-principal SOL conversion; pending contributions stay put.
use anchor_lang::solana_program::program_error::ProgramError;
pub const STAKE_PENDING_SOL_SELECTOR: [u8; 8] = *b"PIV1SP01";
pub const STAKE_PENDING_SOL_DATA_LEN: usize = 25;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct StakePendingSolParameters {
    pub native_lamports: u64,
    pub caller_minimum_pool_tokens_out: u64,
}
impl StakePendingSolParameters {
    pub fn decode(data: &[u8]) -> Result<Self, ProgramError> {
        if data.len() != STAKE_PENDING_SOL_DATA_LEN || data[..8] != STAKE_PENDING_SOL_SELECTOR || data[8] != 1 {
            return Err(ProgramError::InvalidInstructionData);
        }
        Ok(Self { native_lamports: u64::from_le_bytes(data[9..17].try_into().unwrap()),
            caller_minimum_pool_tokens_out: u64::from_le_bytes(data[17..25].try_into().unwrap()) })
    }
    pub fn encode(self) -> [u8; STAKE_PENDING_SOL_DATA_LEN] {
        let mut data = [0; STAKE_PENDING_SOL_DATA_LEN]; data[..8].copy_from_slice(&STAKE_PENDING_SOL_SELECTOR);
        data[8] = 1; data[9..17].copy_from_slice(&self.native_lamports.to_le_bytes());
        data[17..25].copy_from_slice(&self.caller_minimum_pool_tokens_out.to_le_bytes()); data
    }
}

instruction_marker!(pub StakePendingSol);
