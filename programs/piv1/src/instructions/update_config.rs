//! Strict current-governance recipient replacement and one-seat guardian synchronization.
//! Strategy updates remain a marker; neither ABI changes economic custody powers.

use anchor_lang::solana_program::program_error::ProgramError;

instruction_marker!(pub UpdateRecipients);
instruction_marker!(pub UpdateGuardianSet);
instruction_marker!(pub UpdateStrategyConfig);

pub const UPDATE_RECIPIENTS_SELECTOR: [u8; 8] = *b"PIV1UR02";
pub const UPDATE_RECIPIENTS_SIZE: usize = 10;

/// Governance vault witness only. Both recipient keys are bound by the exact
/// authenticated instruction account metas, independently of destination type.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct UpdateRecipientsRequest {
    pub vault_index: u8,
}

impl UpdateRecipientsRequest {
    pub fn decode(data: &[u8]) -> Result<Self, ProgramError> {
        let data: &[u8; UPDATE_RECIPIENTS_SIZE] = data.try_into()
            .map_err(|_| ProgramError::InvalidInstructionData)?;
        if data[..8] != UPDATE_RECIPIENTS_SELECTOR || data[8] != 2 {
            return Err(ProgramError::InvalidInstructionData);
        }
        Ok(Self { vault_index: data[9] })
    }

    pub fn encode(self) -> [u8; UPDATE_RECIPIENTS_SIZE] {
        let mut data = [0; UPDATE_RECIPIENTS_SIZE];
        data[..8].copy_from_slice(&UPDATE_RECIPIENTS_SELECTOR); data[8] = 2;
        data[9] = self.vault_index; data
    }
}

/// One-seat synchronization after a separately authorized Squads membership change.
pub const UPDATE_GUARDIAN_SET_SELECTOR: [u8; 8] = *b"PIV1UG01";
pub const UPDATE_GUARDIAN_SET_SIZE: usize = 51;
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct UpdateGuardianSetRequest {
    pub vault_index: u8,
    pub guardian_index: u8,
    pub expected_revision: u64,
    pub new_guardian: anchor_lang::prelude::Pubkey,
}
impl UpdateGuardianSetRequest {
    pub fn decode(data: &[u8]) -> Result<Self, ProgramError> {
        let data: &[u8; UPDATE_GUARDIAN_SET_SIZE] = data.try_into()
            .map_err(|_| ProgramError::InvalidInstructionData)?;
        if data[..8] != UPDATE_GUARDIAN_SET_SELECTOR || data[8] != 1 || data[10] >= 6 {
            return Err(ProgramError::InvalidInstructionData);
        }
        Ok(Self { vault_index: data[9], guardian_index: data[10],
            expected_revision: u64::from_le_bytes(data[11..19].try_into().unwrap()),
            new_guardian: anchor_lang::prelude::Pubkey::new_from_array(data[19..51].try_into().unwrap()) })
    }
    pub fn encode(self) -> [u8; UPDATE_GUARDIAN_SET_SIZE] {
        let mut data = [0; UPDATE_GUARDIAN_SET_SIZE];
        data[..8].copy_from_slice(&UPDATE_GUARDIAN_SET_SELECTOR); data[8] = 1;
        data[9] = self.vault_index; data[10] = self.guardian_index;
        data[11..19].copy_from_slice(&self.expected_revision.to_le_bytes());
        data[19..].copy_from_slice(self.new_guardian.as_ref()); data
    }
}
