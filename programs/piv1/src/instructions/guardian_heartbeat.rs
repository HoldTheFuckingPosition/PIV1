//! Fixed current-guardian heartbeat ABI. Period and timestamp are runtime inputs,
//! never instruction fields; activity recording does not create KIF liabilities.

use anchor_lang::solana_program::program_error::ProgramError;

instruction_marker!(pub GuardianHeartbeat);

pub const GUARDIAN_HEARTBEAT_SELECTOR: [u8; 8] = *b"PIV1HB01";
pub const GUARDIAN_HEARTBEAT_SIZE: usize = 11;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct GuardianHeartbeatRequest {
    pub guardian_index: u8,
    pub vault_index: u8,
}

impl GuardianHeartbeatRequest {
    pub fn decode(data: &[u8]) -> Result<Self, ProgramError> {
        let data: &[u8; GUARDIAN_HEARTBEAT_SIZE] = data.try_into().map_err(|_| ProgramError::InvalidInstructionData)?;
        if data[..8] != GUARDIAN_HEARTBEAT_SELECTOR || data[8] != 1 || data[9] >= 6 {
            return Err(ProgramError::InvalidInstructionData);
        }
        Ok(Self { guardian_index: data[9], vault_index: data[10] })
    }

    pub fn encode(self) -> Result<[u8; GUARDIAN_HEARTBEAT_SIZE], ProgramError> {
        if self.guardian_index >= 6 { return Err(ProgramError::InvalidInstructionData); }
        let mut data = [0; GUARDIAN_HEARTBEAT_SIZE];
        data[..8].copy_from_slice(&GUARDIAN_HEARTBEAT_SELECTOR);
        data[8] = 1; data[9] = self.guardian_index; data[10] = self.vault_index;
        Ok(data)
    }
}
