//! Fixed six-guardian registry and per-guardian reward state.
//!
//! These are bounded Anchor/Borsh payload layouts. Their `SPACE`
//! constants include the discriminator bytes planned for future owner-bound
//! accounts; Task 1.3 deliberately does not use `#[account]` or require a
//! Program ID.

use anchor_lang::prelude::{borsh, Pubkey};
use anchor_lang::{AnchorDeserialize, AnchorSerialize, InitSpace, Space};

use crate::constants::{
    GUARDIAN_BITMAP_MASK, GUARDIAN_COUNT, PLANNED_ACCOUNT_DISCRIMINATOR_BYTES,
    STATE_LAYOUT_VERSION,
};
use crate::errors::{Piv1Error, Piv1Result};

/// Fixed guardian membership used to snapshot KIF eligibility.
#[derive(AnchorSerialize, AnchorDeserialize, InitSpace, Debug, Clone, Copy, PartialEq, Eq)]
pub struct GuardianRegistry {
    /// Serialized layout version.
    pub version: u8,
    /// Future guardian-registry PDA bump.
    pub bump: u8,
    /// Monotonic guardian-set revision binding reward records to this set.
    pub revision: u64,
    /// Exactly six configured guardian public keys.
    pub guardian_keys: [Pubkey; GUARDIAN_COUNT],
}

impl GuardianRegistry {
    /// Maximum serialized payload bytes, excluding a future discriminator.
    pub const SERIALIZED_SIZE: usize = 1 + 1 + 8 + (32 * GUARDIAN_COUNT);
    /// Planned future Anchor account allocation, including its discriminator.
    pub const SPACE: usize =
        PLANNED_ACCOUNT_DISCRIMINATOR_BYTES + Self::SERIALIZED_SIZE;

    /// Constructs and validates one fixed guardian-set revision.
    pub fn new(
        bump: u8,
        revision: u64,
        guardian_keys: [Pubkey; GUARDIAN_COUNT],
    ) -> Piv1Result<Self> {
        let registry = Self {
            version: STATE_LAYOUT_VERSION,
            bump,
            revision,
            guardian_keys,
        };
        registry.validate()?;
        Ok(registry)
    }

    /// Validates the supported version and six distinct, initialized keys.
    pub fn validate(&self) -> Piv1Result<()> {
        if self.version != STATE_LAYOUT_VERSION {
            return Err(Piv1Error::InvalidVersion);
        }

        for (index, guardian) in self.guardian_keys.iter().enumerate() {
            if *guardian == Pubkey::default() {
                return Err(Piv1Error::InvalidAddress);
            }
            if self.guardian_keys[..index].contains(guardian) {
                return Err(Piv1Error::InvalidGuardianSet);
            }
        }

        Ok(())
    }

    /// Returns the configured key for a checked guardian slot.
    pub fn guardian_key(&self, guardian_index: u8) -> Piv1Result<Pubkey> {
        self.validate()?;
        self.guardian_keys
            .get(usize::from(guardian_index))
            .copied()
            .ok_or(Piv1Error::InvalidGuardianSet)
    }

    /// Validates a reward record against an exact registry slot and revision.
    pub fn validate_reward_binding(
        &self,
        expected_index: u8,
        reward: &GuardianReward,
    ) -> Piv1Result<()> {
        self.validate()?;
        reward.validate()?;

        if reward.guardian_index != expected_index
            || reward.registry_revision != self.revision
            || reward.guardian != self.guardian_key(expected_index)?
        {
            return Err(Piv1Error::InvalidGuardianSet);
        }

        Ok(())
    }

    /// Derives the immutable six-bit activity snapshot for one KIF period.
    pub fn activity_bitmap(
        &self,
        rewards: &[GuardianReward; GUARDIAN_COUNT],
        period_id: u64,
    ) -> Piv1Result<u8> {
        self.validate()?;

        let mut bitmap = 0_u8;
        for (index, reward) in rewards.iter().enumerate() {
            let guardian_index =
                u8::try_from(index).map_err(|_| Piv1Error::ArithmeticOverflow)?;
            self.validate_reward_binding(guardian_index, reward)?;
            if reward.is_active_in(period_id) {
                let bit = 1_u8
                    .checked_shl(u32::from(guardian_index))
                    .ok_or(Piv1Error::ArithmeticOverflow)?;
                bitmap = bitmap
                    .checked_add(bit)
                    .ok_or(Piv1Error::ArithmeticOverflow)?;
            }
        }

        let active_count =
            u8::try_from(bitmap.count_ones()).map_err(|_| Piv1Error::ArithmeticOverflow)?;
        Self::validate_activity_snapshot(bitmap, active_count)?;
        Ok(bitmap)
    }

    /// Validates that a bitmap and stored active count describe six slots.
    pub fn validate_activity_snapshot(bitmap: u8, active_count: u8) -> Piv1Result<()> {
        if bitmap & !GUARDIAN_BITMAP_MASK != 0
            || usize::from(active_count) > GUARDIAN_COUNT
            || bitmap.count_ones() != u32::from(active_count)
        {
            return Err(Piv1Error::InvalidGuardianBitmap);
        }
        Ok(())
    }
}

/// One bounded reward/activity record for an exact guardian registry slot.
#[derive(AnchorSerialize, AnchorDeserialize, InitSpace, Debug, Clone, Copy, PartialEq, Eq)]
pub struct GuardianReward {
    /// Serialized layout version.
    pub version: u8,
    /// Future guardian-reward PDA bump.
    pub bump: u8,
    /// Fixed index in the six-key registry.
    pub guardian_index: u8,
    /// Registry revision for which this binding is valid.
    pub registry_revision: u64,
    /// Guardian public key bound to the fixed index.
    pub guardian: Pubkey,
    /// Most recent active KIF period; `None` is explicit inactivity since init.
    pub last_active_period: Option<u64>,
    /// Unclaimed KIF liability in lamports.
    pub claimable_lamports: u64,
    /// Total KIF lamports ever credited.
    pub cumulative_earned: u64,
    /// Total KIF lamports already claimed.
    pub cumulative_claimed: u64,
}

impl GuardianReward {
    /// Maximum serialized payload bytes, excluding a future discriminator.
    pub const SERIALIZED_SIZE: usize = 1 + 1 + 1 + 8 + 32 + (1 + 8) + 8 + 8 + 8;
    /// Planned future Anchor account allocation, including its discriminator.
    pub const SPACE: usize =
        PLANNED_ACCOUNT_DISCRIMINATOR_BYTES + Self::SERIALIZED_SIZE;

    /// Constructs a zero-balance reward record bound to one registry slot.
    pub fn new(
        bump: u8,
        registry: &GuardianRegistry,
        guardian_index: u8,
    ) -> Piv1Result<Self> {
        let reward = Self {
            version: STATE_LAYOUT_VERSION,
            bump,
            guardian_index,
            registry_revision: registry.revision,
            guardian: registry.guardian_key(guardian_index)?,
            last_active_period: None,
            claimable_lamports: 0,
            cumulative_earned: 0,
            cumulative_claimed: 0,
        };
        registry.validate_reward_binding(guardian_index, &reward)?;
        Ok(reward)
    }

    /// Validates the standalone reward layout and its accounting identity.
    pub fn validate(&self) -> Piv1Result<()> {
        if self.version != STATE_LAYOUT_VERSION {
            return Err(Piv1Error::InvalidVersion);
        }
        if self.guardian == Pubkey::default() {
            return Err(Piv1Error::InvalidAddress);
        }
        if usize::from(self.guardian_index) >= GUARDIAN_COUNT {
            return Err(Piv1Error::InvalidGuardianSet);
        }

        let expected_claimable = self
            .cumulative_earned
            .checked_sub(self.cumulative_claimed)
            .ok_or(Piv1Error::CumulativeReconciliationMismatch)?;
        if expected_claimable != self.claimable_lamports {
            return Err(Piv1Error::CumulativeReconciliationMismatch);
        }

        Ok(())
    }

    /// Returns whether this guardian was active in the exact snapshotted period.
    pub fn is_active_in(&self, period_id: u64) -> bool {
        self.last_active_period == Some(period_id)
    }

    /// Records monotonic activity after validating the full registry binding.
    ///
    /// All validation and staging completes before `self` is replaced, so a
    /// rejected regression or binding mismatch cannot partially mutate state.
    pub fn record_activity(
        &mut self,
        registry: &GuardianRegistry,
        expected_index: u8,
        period_id: u64,
    ) -> Piv1Result<()> {
        registry.validate_reward_binding(expected_index, self)?;
        if self
            .last_active_period
            .is_some_and(|last_period| period_id < last_period)
        {
            return Err(Piv1Error::TimestampRegression);
        }

        let mut staged = *self;
        staged.last_active_period = Some(period_id);
        registry.validate_reward_binding(expected_index, &staged)?;
        *self = staged;
        Ok(())
    }

    /// Credits a checked KIF liability after validating its registry binding.
    ///
    /// Both additions are staged before commit, preserving the complete prior
    /// state if either value overflows or any invariant fails.
    pub fn credit(
        &mut self,
        registry: &GuardianRegistry,
        expected_index: u8,
        lamports: u64,
    ) -> Piv1Result<()> {
        registry.validate_reward_binding(expected_index, self)?;

        let mut staged = *self;
        staged.claimable_lamports = staged
            .claimable_lamports
            .checked_add(lamports)
            .ok_or(Piv1Error::ArithmeticOverflow)?;
        staged.cumulative_earned = staged
            .cumulative_earned
            .checked_add(lamports)
            .ok_or(Piv1Error::ArithmeticOverflow)?;
        registry.validate_reward_binding(expected_index, &staged)?;
        *self = staged;
        Ok(())
    }

    /// Credits a reward from an immutable active-round guardian snapshot.
    ///
    /// This remains valid even if the live registry rotates after preparation:
    /// the exact key, slot, and revision were fixed in the active header.
    pub fn credit_snapshot(
        &mut self,
        expected_guardian: Pubkey,
        expected_index: u8,
        expected_registry_revision: u64,
        lamports: u64,
    ) -> Piv1Result<()> {
        self.validate()?;
        if self.guardian != expected_guardian
            || self.guardian_index != expected_index
            || self.registry_revision != expected_registry_revision
        {
            return Err(Piv1Error::InvalidGuardianSet);
        }

        let mut staged = *self;
        staged.claimable_lamports = staged
            .claimable_lamports
            .checked_add(lamports)
            .ok_or(Piv1Error::ArithmeticOverflow)?;
        staged.cumulative_earned = staged
            .cumulative_earned
            .checked_add(lamports)
            .ok_or(Piv1Error::ArithmeticOverflow)?;
        staged.validate()?;
        *self = staged;
        Ok(())
    }
}

const _: [(); GuardianRegistry::SERIALIZED_SIZE] = [(); GuardianRegistry::INIT_SPACE];
const _: [(); GuardianReward::SERIALIZED_SIZE] = [(); GuardianReward::INIT_SPACE];

#[cfg(test)]
mod tests {
    use super::*;

    fn guardian(seed: u8) -> Pubkey {
        Pubkey::new_from_array([seed; 32])
    }

    fn keys() -> [Pubkey; GUARDIAN_COUNT] {
        core::array::from_fn(|index| guardian((index + 1) as u8))
    }

    fn registry(revision: u64) -> GuardianRegistry {
        GuardianRegistry::new(254, revision, keys()).expect("valid registry fixture")
    }

    fn rewards(registry: &GuardianRegistry) -> [GuardianReward; GUARDIAN_COUNT] {
        core::array::from_fn(|index| {
            GuardianReward::new(
                index as u8,
                registry,
                index as u8,
            )
            .expect("valid reward fixture")
        })
    }

    #[test]
    fn registry_serialization_is_exact_and_round_trips() {
        let registry = GuardianRegistry::new(u8::MAX, u64::MAX, keys())
            .expect("maximum fixed fields are valid");
        let encoded = registry.try_to_vec().expect("serialize registry");
        assert_eq!(GuardianRegistry::INIT_SPACE, GuardianRegistry::SERIALIZED_SIZE);
        assert_eq!(encoded.len(), GuardianRegistry::SERIALIZED_SIZE);
        assert_eq!(GuardianRegistry::SPACE, 210);

        let mut bytes = encoded.as_slice();
        let decoded = GuardianRegistry::deserialize(&mut bytes).expect("deserialize registry");
        assert_eq!(decoded, registry);
        assert!(bytes.is_empty());
    }

    #[test]
    fn reward_maximum_serialization_is_exact_and_round_trips() {
        let registry = registry(u64::MAX);
        let reward = GuardianReward {
            version: STATE_LAYOUT_VERSION,
            bump: u8::MAX,
            guardian_index: (GUARDIAN_COUNT - 1) as u8,
            registry_revision: u64::MAX,
            guardian: registry.guardian_keys[GUARDIAN_COUNT - 1],
            last_active_period: Some(u64::MAX),
            claimable_lamports: u64::MAX,
            cumulative_earned: u64::MAX,
            cumulative_claimed: 0,
        };
        registry
            .validate_reward_binding((GUARDIAN_COUNT - 1) as u8, &reward)
            .expect("maximum reward is valid");

        let encoded = reward.try_to_vec().expect("serialize reward");
        assert_eq!(GuardianReward::INIT_SPACE, GuardianReward::SERIALIZED_SIZE);
        assert_eq!(encoded.len(), GuardianReward::SERIALIZED_SIZE);
        assert_eq!(GuardianReward::SPACE, 84);

        let mut bytes = encoded.as_slice();
        let decoded = GuardianReward::deserialize(&mut bytes).expect("deserialize reward");
        assert_eq!(decoded, reward);
        assert!(bytes.is_empty());
    }

    #[test]
    fn reward_option_encoding_accounts_for_none_and_some_payloads() {
        let registry = registry(12);
        let inactive = GuardianReward::new(3, &registry, 3).expect("valid reward");
        let inactive_encoded = inactive.try_to_vec().expect("serialize inactive reward");
        assert_eq!(
            inactive_encoded.len(),
            GuardianReward::SERIALIZED_SIZE - core::mem::size_of::<u64>()
        );

        let mut active = inactive;
        active.last_active_period = Some(u64::MAX);
        let active_encoded = active.try_to_vec().expect("serialize active reward");
        assert_eq!(active_encoded.len(), GuardianReward::SERIALIZED_SIZE);
        assert_eq!(active_encoded.len() - inactive_encoded.len(), 8);

        let inactive_decoded = GuardianReward::try_from_slice(&inactive_encoded)
            .expect("deserialize inactive reward");
        let active_decoded = GuardianReward::try_from_slice(&active_encoded)
            .expect("deserialize active reward");
        assert_eq!(inactive_decoded, inactive);
        assert_eq!(active_decoded, active);
    }

    #[test]
    fn registry_requires_exactly_six_distinct_nondefault_keys() {
        let valid = registry(0);
        assert_eq!(valid.guardian_keys.len(), GUARDIAN_COUNT);

        let mut with_default = keys();
        with_default[3] = Pubkey::default();
        assert_eq!(
            GuardianRegistry::new(0, 0, with_default),
            Err(Piv1Error::InvalidAddress)
        );

        let mut with_duplicate = keys();
        with_duplicate[5] = with_duplicate[1];
        assert_eq!(
            GuardianRegistry::new(0, 0, with_duplicate),
            Err(Piv1Error::InvalidGuardianSet)
        );
    }

    #[test]
    fn activity_is_monotonic_and_produces_a_checked_six_bit_snapshot() {
        let registry = registry(9);
        let mut rewards = rewards(&registry);
        for index in [0_u8, 2, 5] {
            rewards[usize::from(index)]
                .record_activity(&registry, index, 41)
                .expect("record activity");
        }

        let bitmap = registry
            .activity_bitmap(&rewards, 41)
            .expect("derive activity bitmap");
        assert_eq!(bitmap, 0b10_0101);
        GuardianRegistry::validate_activity_snapshot(bitmap, 3)
            .expect("valid activity snapshot");
        assert_eq!(
            GuardianRegistry::validate_activity_snapshot(bitmap, 2),
            Err(Piv1Error::InvalidGuardianBitmap)
        );
        assert_eq!(
            GuardianRegistry::validate_activity_snapshot(1 << GUARDIAN_COUNT, 1),
            Err(Piv1Error::InvalidGuardianBitmap)
        );
        assert_eq!(
            registry
                .activity_bitmap(&rewards, 40)
                .expect("older period has no exact activity"),
            0
        );

        let before = rewards[2];
        assert_eq!(
            rewards[2].record_activity(&registry, 2, 40),
            Err(Piv1Error::TimestampRegression)
        );
        assert_eq!(rewards[2], before);
    }

    #[test]
    fn revision_key_and_index_mismatches_are_rejected_without_mutation() {
        let registry = registry(7);
        let base = GuardianReward::new(1, &registry, 1).expect("valid reward");

        let mut wrong_revision = base;
        wrong_revision.registry_revision = 8;
        let before = wrong_revision;
        assert_eq!(
            wrong_revision.credit(&registry, 1, 5),
            Err(Piv1Error::InvalidGuardianSet)
        );
        assert_eq!(wrong_revision, before);

        let mut wrong_key = base;
        wrong_key.guardian = registry.guardian_keys[2];
        let before = wrong_key;
        assert_eq!(
            wrong_key.record_activity(&registry, 1, 4),
            Err(Piv1Error::InvalidGuardianSet)
        );
        assert_eq!(wrong_key, before);

        let mut wrong_index = base;
        let before = wrong_index;
        assert_eq!(
            wrong_index.credit(&registry, 2, 5),
            Err(Piv1Error::InvalidGuardianSet)
        );
        assert_eq!(wrong_index, before);
    }

    #[test]
    fn unsupported_versions_and_default_reward_bindings_are_rejected() {
        let registry = registry(7);

        let mut wrong_registry_version = registry;
        wrong_registry_version.version = STATE_LAYOUT_VERSION.saturating_add(1);
        assert_eq!(wrong_registry_version.validate(), Err(Piv1Error::InvalidVersion));

        let mut reward = GuardianReward::new(1, &registry, 1).expect("valid reward");
        reward.version = STATE_LAYOUT_VERSION.saturating_add(1);
        assert_eq!(reward.validate(), Err(Piv1Error::InvalidVersion));

        reward.version = STATE_LAYOUT_VERSION;
        reward.guardian = Pubkey::default();
        assert_eq!(reward.validate(), Err(Piv1Error::InvalidAddress));
        assert_eq!(
            registry.validate_reward_binding(1, &reward),
            Err(Piv1Error::InvalidAddress)
        );
    }

    #[test]
    fn snapshot_credit_remains_bound_to_the_pre_rotation_guardian_set() {
        let original_registry = registry(21);
        let mut reward =
            GuardianReward::new(4, &original_registry, 4).expect("valid reward");
        let snapshotted_guardian = original_registry.guardian_keys[4];

        let rotated_registry = registry(22);
        assert_eq!(
            rotated_registry.validate_reward_binding(4, &reward),
            Err(Piv1Error::InvalidGuardianSet)
        );

        reward
            .credit_snapshot(snapshotted_guardian, 4, 21, 17)
            .expect("immutable pre-rotation snapshot remains creditable");
        assert_eq!(reward.claimable_lamports, 17);
        assert_eq!(reward.cumulative_earned, 17);

        for (guardian_key, index, revision) in [
            (guardian(99), 4, 21),
            (snapshotted_guardian, 3, 21),
            (snapshotted_guardian, 4, 22),
        ] {
            let before = reward;
            assert_eq!(
                reward.credit_snapshot(guardian_key, index, revision, 1),
                Err(Piv1Error::InvalidGuardianSet)
            );
            assert_eq!(reward, before);
        }
    }

    #[test]
    fn checked_credit_preserves_state_on_each_overflow_path() {
        let registry = registry(3);
        let mut claimable_overflow =
            GuardianReward::new(0, &registry, 0).expect("valid reward");
        claimable_overflow.claimable_lamports = u64::MAX;
        claimable_overflow.cumulative_earned = u64::MAX;
        let before = claimable_overflow;
        assert_eq!(
            claimable_overflow.credit(&registry, 0, 1),
            Err(Piv1Error::ArithmeticOverflow)
        );
        assert_eq!(claimable_overflow, before);

        let mut earned_overflow =
            GuardianReward::new(0, &registry, 0).expect("valid reward");
        earned_overflow.claimable_lamports = 1;
        earned_overflow.cumulative_earned = u64::MAX;
        earned_overflow.cumulative_claimed = u64::MAX - 1;
        earned_overflow.validate().expect("balanced overflow fixture");
        let before = earned_overflow;
        assert_eq!(
            earned_overflow.credit(&registry, 0, 1),
            Err(Piv1Error::ArithmeticOverflow)
        );
        assert_eq!(earned_overflow, before);

        let mut credited = GuardianReward::new(0, &registry, 0).expect("valid reward");
        credited
            .credit(&registry, 0, 11)
            .expect("checked credit");
        assert_eq!(credited.claimable_lamports, 11);
        assert_eq!(credited.cumulative_earned, 11);
        assert_eq!(credited.cumulative_claimed, 0);
    }
}
