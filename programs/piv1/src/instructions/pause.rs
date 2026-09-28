//! Strict explicit-set emergency pause ABI, authenticated by the entire original
//! Squads-approved instruction. This is not an unauthenticated toggle.

use anchor_lang::solana_program::program_error::ProgramError;

instruction_marker!(pub SetPause);

pub const SET_PAUSE_SELECTOR: [u8; 8] = *b"PIV1PS01";
pub const SET_PAUSE_SIZE: usize = 11;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct SetPauseRequest {
    pub vault_index: u8,
    pub paused: bool,
}

impl SetPauseRequest {
    pub fn decode(data: &[u8]) -> Result<Self, ProgramError> {
        let data: &[u8; SET_PAUSE_SIZE] = data.try_into().map_err(|_| ProgramError::InvalidInstructionData)?;
        if data[..8] != SET_PAUSE_SELECTOR || data[8] != 1 || data[10] > 1 {
            return Err(ProgramError::InvalidInstructionData);
        }
        Ok(Self { vault_index: data[9], paused: data[10] == 1 })
    }

    pub fn encode(self) -> [u8; SET_PAUSE_SIZE] {
        let mut data = [0; SET_PAUSE_SIZE]; data[..8].copy_from_slice(&SET_PAUSE_SELECTOR);
        data[8] = 1; data[9] = self.vault_index; data[10] = u8::from(self.paused); data
    }
}
