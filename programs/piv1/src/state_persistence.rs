//! Validated fixed state envelopes and atomic existing-account byte persistence.
//!
//! This is a structural library utility, not an instruction or authorization
//! capability. Future handlers must supply trusted executing program ID/Rent,
//! authenticate relevant topology, enforce governance/signers/pause/lifecycle,
//! and derive replacements through authorized checked transitions. Per-account
//! validation does not prove cross-account economics or a legal HWM change.
//! Reauthenticate actual accounts around interactions. No lamports are moved.

use std::{cell::RefMut, rc::Rc};
use anchor_lang::{
    prelude::{AccountInfo, Pubkey, Rent},
    solana_program::system_program,
    AnchorSerialize,
};
use crate::{
    accounts::{rent_floor, seeds, validate_pda, CONFIG_DISCRIMINATOR,
        DISTRIBUTION_DISCRIMINATOR, STAKE_PROGRAM_ID},
    errors::{Piv1Error, Piv1Result},
    guardian_clock_accounts::{GUARDIAN_REGISTRY_DISCRIMINATOR, GUARDIAN_REGISTRY_SEED},
    kif_claim_accounts::{GUARDIAN_REWARD_DISCRIMINATOR, GUARDIAN_REWARD_SEED},
    state::{ActiveDistribution, GuardianRegistry, GuardianReward, PivConfig},
};

/// Immutable means PDA identity, not immutability of the whole payload. A fixed
/// registry PDA does not freeze membership/revision or authorize their change.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum StateIdentity {
    Config(u8),
    Distribution(u8),
    Registry(u8),
    Reward { bump: u8, guardian: Pubkey, revision: u64, index: u8 },
}

impl StateIdentity {
    fn validate(self, program: &Pubkey, target: &Pubkey) -> Piv1Result<()> {
        match self {
            Self::Config(bump) => validate_pda(program, target, seeds::CONFIG, bump),
            Self::Distribution(bump) => validate_pda(program, target, seeds::DISTRIBUTION, bump),
            Self::Registry(bump) => validate_pda(program, target, GUARDIAN_REGISTRY_SEED, bump),
            Self::Reward { bump, guardian, revision, index } => {
                let (expected, canonical_bump) = Pubkey::try_find_program_address(
                    &[GUARDIAN_REWARD_SEED, guardian.as_ref(), &revision.to_le_bytes(), &[index]],
                    program,
                ).ok_or(Piv1Error::InvalidAccountPda)?;
                if *target != expected || bump != canonical_bump {
                    return Err(Piv1Error::InvalidAccountPda);
                }
                Ok(())
            }
        }
    }
}

/// Owned canonical envelope constructed only from one of the four validated
/// typed payloads. Callers cannot supply sizes, discriminators or raw bytes.
#[derive(Debug, Eq, PartialEq)]
pub struct StateEnvelope {
    identity: StateIdentity,
    bytes: Vec<u8>,
}

impl StateEnvelope {
    pub fn config(value: &PivConfig) -> Piv1Result<Self> {
        value.validate_initialized()?;
        Self::encode(value, StateIdentity::Config(value.bumps.config),
            PivConfig::SPACE, CONFIG_DISCRIMINATOR)
    }

    pub fn distribution(value: &ActiveDistribution) -> Piv1Result<Self> {
        value.validate()?;
        Self::encode(value, StateIdentity::Distribution(value.bump),
            ActiveDistribution::SPACE, DISTRIBUTION_DISCRIMINATOR)
    }

    pub fn registry(value: &GuardianRegistry) -> Piv1Result<Self> {
        value.validate()?;
        Self::encode(value, StateIdentity::Registry(value.bump),
            GuardianRegistry::SPACE, GUARDIAN_REGISTRY_DISCRIMINATOR)
    }

    pub fn reward(value: &GuardianReward) -> Piv1Result<Self> {
        value.validate()?;
        Self::encode(value, StateIdentity::Reward { bump: value.bump,
            guardian: value.guardian, revision: value.registry_revision,
            index: value.guardian_index }, GuardianReward::SPACE, GUARDIAN_REWARD_DISCRIMINATOR)
    }

    /// The complete discriminator-inclusive fixed allocation, including zeros.
    pub fn as_bytes(&self) -> &[u8] { &self.bytes }

    fn encode<T: AnchorSerialize>(value: &T, identity: StateIdentity,
        space: usize, discriminator: [u8; 8]) -> Piv1Result<Self>
    {
        let mut bytes = Vec::new();
        bytes.try_reserve_exact(space).map_err(|_| Piv1Error::StateEnvelopeEncodingFailed)?;
        bytes.resize(space, 0);
        let (prefix, mut payload) = bytes.split_at_mut(discriminator.len());
        prefix.copy_from_slice(&discriminator);
        // A fixed slice writer reports insufficient capacity instead of growing,
        // truncating a successful result, or leaving bytes from an older value.
        value.serialize(&mut payload).map_err(|_| Piv1Error::StateEnvelopeEncodingFailed)?;
        Ok(Self { identity, bytes })
    }
}

/// Private complete before/after records. Preparing does not authenticate an
/// account or approve a transition. Commit compares the entire prior envelope.
#[derive(Debug, Eq, PartialEq)]
pub struct PreparedStateWrite {
    program: Pubkey,
    target: Pubkey,
    before: StateEnvelope,
    after: StateEnvelope,
}

impl PreparedStateWrite {
    pub fn new(
        trusted_runtime_program_id: &Pubkey,
        target: Pubkey,
        before: StateEnvelope,
        after: StateEnvelope,
    ) -> Piv1Result<Self> {
        validate_program(trusted_runtime_program_id)?;
        before.identity.validate(trusted_runtime_program_id, &target)?;
        after.identity.validate(trusted_runtime_program_id, &target)?;
        if before.identity != after.identity {
            return Err(Piv1Error::InvalidAccountPda);
        }
        Ok(Self { program: *trusted_runtime_program_id, target, before, after })
    }

    pub fn target(&self) -> Pubkey { self.target }
    pub fn expected_before(&self) -> &[u8] { self.before.as_bytes() }
    pub fn replacement(&self) -> &[u8] { self.after.as_bytes() }
}

/// Commits a statically sized batch to existing accounts. N is a Rust call-site
/// choice, never an instruction-provided unbounded length. Empty and no-op batches
/// are allowed; byte equality alone is not general replay protection.
///
/// Every mutable borrow is acquired and retained before copying any bytes. The
/// final loop performs only previously size-checked copies, with no allocation,
/// serialization, validation or additional borrowing. Failures preserve every
/// byte and lamport. Host evidence does not prove runtime locking/rollback or
/// SBF compute/heap limits, which require separate integration validation.
pub fn commit_state_writes<const N: usize>(
    trusted_runtime_program_id: &Pubkey,
    trusted_runtime_rent: &Rent,
    writes: [(&PreparedStateWrite, &AccountInfo<'_>); N],
) -> Piv1Result<()> {
    validate_program(trusted_runtime_program_id)?;
    for (i, (write, account)) in writes.iter().enumerate() {
        if write.program != *trusted_runtime_program_id {
            return Err(Piv1Error::InvalidProgramIdentity);
        }
        for (_, other) in &writes[i + 1..] {
            if account.key == other.key || Rc::ptr_eq(&account.data, &other.data) {
                return Err(Piv1Error::AccountAlias);
            }
        }
        if *account.key != write.target { return Err(Piv1Error::InvalidAccountPda); }
        write.before.identity.validate(trusted_runtime_program_id, account.key)?;
        write.after.identity.validate(trusted_runtime_program_id, account.key)?;
        if account.owner != trusted_runtime_program_id { return Err(Piv1Error::InvalidAccountOwner); }
        if account.executable { return Err(Piv1Error::ExecutableAccount); }
        if !account.is_writable { return Err(Piv1Error::AccountNotWritable); }
        let floor = rent_floor(trusted_runtime_rent, write.before.bytes.len())?;
        let lamports = account.try_borrow_lamports().map_err(|_| Piv1Error::AccountBorrowFailed)?;
        if **lamports < floor { return Err(Piv1Error::AccountRentDeficit); }
    }

    let mut data_borrows: [Option<RefMut<'_, &mut [u8]>>; N] = core::array::from_fn(|_| None);
    for ((write, account), slot) in writes.iter().zip(data_borrows.iter_mut()) {
        let data = account.try_borrow_mut_data().map_err(|_| Piv1Error::AccountBorrowFailed)?;
        if data.len() != write.before.bytes.len() || data.len() != write.after.bytes.len() {
            return Err(Piv1Error::InvalidAccountSize);
        }
        if **data != write.before.bytes { return Err(Piv1Error::StateEnvelopeChanged); }
        *slot = Some(data);
    }

    // Every slot is populated, all lengths match, and all borrows stay held
    // throughout this loop. Nothing fallible or allocating follows the first copy.
    for ((write, _), data) in writes.iter().zip(data_borrows.iter_mut().flatten()) {
        data.copy_from_slice(&write.after.bytes);
    }
    Ok(())
}

fn validate_program(program: &Pubkey) -> Piv1Result<()> {
    if *program == system_program::ID || *program == spl_token::ID || *program == STAKE_PROGRAM_ID {
        return Err(Piv1Error::InvalidProgramIdentity);
    }
    Ok(())
}
