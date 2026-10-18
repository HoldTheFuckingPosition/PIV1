//! Strict current-governance recipient replacement. Guardian/strategy updates
//! remain markers; this ABI introduces no migration or new custody authority.

use anchor_lang::solana_program::program_error::ProgramError;

instruction_marker!(pub UpdateRecipients);
instruction_marker!(pub UpdateGuardianSet);
instruction_marker!(pub UpdateStrategyConfig);

pub const UPDATE_RECIPIENTS_SELECTOR: [u8; 8] = *b"PIV1UR01";
pub const UPDATE_RECIPIENTS_SIZE: usize = 12;

/// Derivation witnesses for the authenticated governance vault and two approved
/// recipient account keys. The original complete instruction is authenticated.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct UpdateRecipientsRequest {
    pub vault_index: u8,
    pub htfp_vault_index: u8,
    pub team_owner_vault_index: u8,
}

impl UpdateRecipientsRequest {
    pub fn decode(data: &[u8]) -> Result<Self, ProgramError> {
        let data: &[u8; UPDATE_RECIPIENTS_SIZE] = data.try_into()
            .map_err(|_| ProgramError::InvalidInstructionData)?;
        if data[..8] != UPDATE_RECIPIENTS_SELECTOR || data[8] != 1 {
            return Err(ProgramError::InvalidInstructionData);
        }
        Ok(Self { vault_index: data[9], htfp_vault_index: data[10], team_owner_vault_index: data[11] })
    }

    pub fn encode(self) -> [u8; UPDATE_RECIPIENTS_SIZE] {
        let mut data = [0; UPDATE_RECIPIENTS_SIZE];
        data[..8].copy_from_slice(&UPDATE_RECIPIENTS_SELECTOR); data[8] = 1;
        data[9] = self.vault_index; data[10] = self.htfp_vault_index;
        data[11] = self.team_owner_vault_index; data
    }
}
