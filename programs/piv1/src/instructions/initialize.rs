//! Initialization marker for permanent, separated custody accounts.
//!
//! Later initialization must create distinct principal/pending legacy Token
//! accounts at PIV1-derived addresses, both controlled by the shared PIV
//! authority, plus the separately owned native-SOL vault roles. Neither token
//! vault may be an ATA. No creation or funding occurs here.

instruction_marker!(pub InitializePiv1);

use anchor_lang::prelude::Pubkey;

/// Distinct ASCII model domain, not an Anchor initialize instruction selector.
/// The native entrypoint does not dispatch this library preparation format.
pub const GENESIS_MODEL_SELECTOR: [u8; 8] = *b"PIV1GM01";
pub const GENESIS_MODEL_VERSION: u8 = 1;
pub const GENESIS_MODEL_DATA_SIZE: usize = 313;

/// Proposed external references only. Approval does not authenticate official
/// Jito deployment, account relationships, mint properties or fee destinations.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct DeclaredGenesisProtocol {
    pub stake_pool_program: Pubkey,
    pub stake_pool: Pubkey,
    pub validator_list: Pubkey,
    pub reserve_stake: Pubkey,
    pub jitosol_mint: Pubkey,
    pub manager_fee_account: Pubkey,
    pub referrer_token_account: Pubkey,
}

/// Fixed model inputs. Recipient control remains unverified; guardian slots are
/// an explicit bijection over the freshly authenticated sorted Squads members.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct GenesisModelParameters {
    pub vault_index: u8,
    pub initially_paused: bool,
    pub protocol: DeclaredGenesisProtocol,
    pub htfp_recipient: Pubkey,
    pub team_owner_recipient: Pubkey,
    pub kif_anchor_timestamp: i64,
    pub guardian_slot_permutation: [u8; 6],
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum GenesisModelFormatError {
    InvalidLength,
    InvalidSelector,
    UnsupportedVersion,
    InvalidBoolean,
    InvalidSlotPermutation,
}

fn validate_permutation(permutation: &[u8; 6]) -> Result<(), GenesisModelFormatError> {
    let mut seen = 0_u8;
    for index in permutation {
        if *index >= 6 || seen & (1 << index) != 0 {
            return Err(GenesisModelFormatError::InvalidSlotPermutation);
        }
        seen |= 1 << index;
    }
    Ok(())
}

impl GenesisModelParameters {
    /// Exactly 313 bytes: selector8/version1/vault1/pause1/protocol224/
    /// recipients64/anchor8/permutation6. No vectors or trailing data.
    pub fn decode(bytes: &[u8]) -> Result<Self, GenesisModelFormatError> {
        let bytes: &[u8; GENESIS_MODEL_DATA_SIZE] = bytes.try_into()
            .map_err(|_| GenesisModelFormatError::InvalidLength)?;
        if bytes[..8] != GENESIS_MODEL_SELECTOR { return Err(GenesisModelFormatError::InvalidSelector); }
        if bytes[8] != GENESIS_MODEL_VERSION { return Err(GenesisModelFormatError::UnsupportedVersion); }
        let initially_paused = match bytes[10] {
            0 => false, 1 => true, _ => return Err(GenesisModelFormatError::InvalidBoolean),
        };
        // Exact fixed length established above; each bounded array reads a
        // compile-time-sized field without an input-sized allocation.
        let key = |start: usize| Pubkey::new_from_array(core::array::from_fn(|i| bytes[start + i]));
        let guardian_slot_permutation = core::array::from_fn(|i| bytes[307 + i]);
        validate_permutation(&guardian_slot_permutation)?;
        Ok(Self {
            vault_index: bytes[9], initially_paused,
            protocol: DeclaredGenesisProtocol {
                stake_pool_program: key(11), stake_pool: key(43), validator_list: key(75),
                reserve_stake: key(107), jitosol_mint: key(139),
                manager_fee_account: key(171), referrer_token_account: key(203),
            },
            htfp_recipient: key(235), team_owner_recipient: key(267),
            kif_anchor_timestamp: i64::from_le_bytes(core::array::from_fn(|i| bytes[299 + i])),
            guardian_slot_permutation,
        })
    }

    pub fn encode(&self) -> Result<[u8; GENESIS_MODEL_DATA_SIZE], GenesisModelFormatError> {
        validate_permutation(&self.guardian_slot_permutation)?;
        let mut bytes = [0_u8; GENESIS_MODEL_DATA_SIZE];
        bytes[..8].copy_from_slice(&GENESIS_MODEL_SELECTOR);
        bytes[8] = GENESIS_MODEL_VERSION; bytes[9] = self.vault_index;
        bytes[10] = u8::from(self.initially_paused);
        for (index, key) in [self.protocol.stake_pool_program, self.protocol.stake_pool,
            self.protocol.validator_list, self.protocol.reserve_stake, self.protocol.jitosol_mint,
            self.protocol.manager_fee_account, self.protocol.referrer_token_account,
            self.htfp_recipient, self.team_owner_recipient].iter().enumerate()
        {
            let start = 11 + 32 * index;
            bytes[start..start + 32].copy_from_slice(key.as_ref());
        }
        bytes[299..307].copy_from_slice(&self.kif_anchor_timestamp.to_le_bytes());
        bytes[307..].copy_from_slice(&self.guardian_slot_permutation);
        Ok(bytes)
    }
}
