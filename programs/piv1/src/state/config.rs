//! Bounded global configuration and cumulative accounting state.
//!
//! This layout is Anchor/Borsh-compatible but deliberately does not use
//! `#[account]`: PIV1 has no production Program ID yet. [`PivConfig::SPACE`]
//! includes the planned eight-byte discriminator for later owner-bound account
//! initialization.

use anchor_lang::{
    prelude::{borsh, Pubkey},
    AnchorDeserialize, AnchorSerialize, InitSpace, Space,
};

use crate::{
    constants::{
        CONFIG_MIGRATION_RESERVE_BYTES, INSUFFICIENT_RETRY_COOLDOWN_SECONDS,
        KIF_PERIOD_SECONDS, MAX_CONFIGURED_SLIPPAGE_BPS,
        MINIMUM_DISTRIBUTION_INTERVAL_SECONDS, PLANNED_ACCOUNT_DISCRIMINATOR_BYTES,
        STATE_LAYOUT_VERSION,
    },
    errors::{Piv1Error, Piv1Result},
};

/// PDA bumps for the permanent PIV1 configuration and custody topology.
///
/// The corresponding addresses remain explicit fields in [`PivConfig`].
/// Future handlers must rederive every address from the eventual real PIV1
/// Program ID instead of trusting a serialized bump or address in isolation.
#[derive(Clone, Copy, Debug, Eq, PartialEq, AnchorSerialize, AnchorDeserialize, InitSpace)]
pub struct PivConfigBumps {
    pub config: u8,
    pub piv_authority: u8,
    pub active_distribution: u8,
    pub principal_jito_vault: u8,
    pub pending_jito_vault: u8,
    pub pending_sol_vault: u8,
    pub principal_sol_queue: u8,
    pub operational_sol_vault: u8,
    pub distribution_escrow: u8,
    pub kif_sol_vault: u8,
    pub guardian_registry: u8,
}

impl PivConfigBumps {
    /// Exact serialized size of the fixed bump bundle.
    pub const SERIALIZED_SIZE: usize = <Self as Space>::INIT_SPACE;
}

/// Version-one global configuration, custody bindings, and cumulative ledgers.
///
/// Every field is fixed-size. The externally owned custody accounts named here
/// remain System, legacy Token, or Stake Program accounts; serializing their
/// addresses does not turn them into PIV1-owned state accounts.
#[derive(Clone, Debug, Eq, PartialEq, AnchorSerialize, AnchorDeserialize, InitSpace)]
pub struct PivConfig {
    /// Serialized layout version.
    pub version: u8,
    /// Explicit initialization marker; zeroed bytes are never valid state.
    pub is_initialized: bool,
    /// Orthogonal emergency-pause flag.
    pub paused: bool,
    /// Fixed permanent-account bump bundle.
    pub bumps: PivConfigBumps,

    // Current official external-program and Jito stake-pool bindings.
    pub stake_pool_program: Pubkey,
    pub stake_pool: Pubkey,
    pub validator_list: Pubkey,
    pub reserve_stake: Pubkey,
    pub jitosol_mint: Pubkey,
    pub token_program: Pubkey,
    pub stake_program: Pubkey,
    pub system_program: Pubkey,
    pub manager_fee_account: Pubkey,
    pub referrer_token_account: Pubkey,

    // Fixed PIV1 authority, state-header, custody, recipient, and registry bindings.
    pub piv_authority: Pubkey,
    pub active_distribution: Pubkey,
    pub principal_jito_vault: Pubkey,
    pub pending_jito_vault: Pubkey,
    pub pending_sol_vault: Pubkey,
    pub principal_sol_queue: Pubkey,
    pub operational_sol_vault: Pubkey,
    pub distribution_escrow: Pubkey,
    pub kif_sol_vault: Pubkey,
    pub htfp_recipient: Pubkey,
    pub team_owner_recipient: Pubkey,
    pub guardian_registry: Pubkey,

    // Stored bindings to the immutable V1 economics and safety policy.
    pub basis_points_denominator: u16,
    pub htfp_reserve_bps: u16,
    pub permanent_compound_bps: u16,
    pub team_owner_pool_bps: u16,
    pub kif_bps: u16,
    pub configured_slippage_bps: u16,
    pub slippage_hard_cap_bps: u16,
    pub minimum_distribution_interval_seconds: i64,
    pub insufficient_retry_cooldown_seconds: i64,

    // Distribution timing and monotonic identity.
    pub last_successful_preparation_at: Option<i64>,
    pub last_valid_insufficient_attempt_at: Option<i64>,
    pub next_distribution_sequence: u64,

    // Protected principal and separately accounted physical quantities.
    pub protected_principal_hwm_lamports: u64,
    pub accounted_historical_jitosol_units: u64,
    pub accounted_historical_sol_lamports: u64,
    pub accounted_pending_jitosol_units: u64,
    pub accounted_pending_sol_lamports: u64,

    // Yield and KIF balances remain distinct from principal and one another.
    pub next_cycle_yield_lamports: u64,
    pub kif_claim_liability_lamports: u64,
    pub collective_kif_carry_lamports: u64,

    // Monotonic audit totals. None is used as a substitute for live custody checks.
    pub cumulative_contribution_value_lamports: u64,
    pub cumulative_gross_yield_lamports: u64,
    pub cumulative_htfp_paid_lamports: u64,
    pub cumulative_team_owner_paid_lamports: u64,
    pub cumulative_kif_credited_lamports: u64,
    pub cumulative_kif_claimed_lamports: u64,
    pub cumulative_permanent_compound_lamports: u64,
    pub cumulative_retained_dust_lamports: u64,
    pub cumulative_zero_active_kif_compound_lamports: u64,
    pub cumulative_cooldown_yield_recorded_lamports: u64,

    // Fixed Clock-derived KIF timing and guardian-set synchronization binding.
    pub kif_anchor_timestamp: i64,
    pub kif_period_seconds: i64,
    pub guardian_registry_revision: u64,

    /// Explicit zeroed migration reserve; V1 assigns it no hidden meaning.
    pub migration_reserve: [u8; CONFIG_MIGRATION_RESERVE_BYTES],
}

impl PivConfig {
    /// Maximum serialized Borsh payload size, excluding a discriminator.
    pub const SERIALIZED_SIZE: usize = <Self as Space>::INIT_SPACE;
    /// Planned discriminator-inclusive Anchor account allocation.
    pub const SPACE: usize = PLANNED_ACCOUNT_DISCRIMINATOR_BYTES + Self::SERIALIZED_SIZE;

    /// Validates the complete initialized V1 configuration binding.
    pub fn validate_initialized(&self) -> Piv1Result<()> {
        if self.version != STATE_LAYOUT_VERSION {
            return Err(Piv1Error::InvalidVersion);
        }
        if !self.is_initialized {
            return Err(Piv1Error::InvalidInitialization);
        }

        self.validate_addresses()?;
        self.validate_fixed_economics()?;
        self.validate_slippage()?;
        self.validate_timing()?;
        self.validate_reserved_bytes()?;
        self.validate_kif_liability_reconciliation()?;

        Ok(())
    }

    /// Applies the confirmed pause gate after validating the stored config.
    pub fn ensure_unpaused(&self) -> Piv1Result<()> {
        self.validate_initialized()?;
        if self.paused {
            return Err(Piv1Error::PausedOperation);
        }
        Ok(())
    }

    /// Allocates the exact next round sequence and advances it once.
    ///
    /// Overflow is rejected before mutation, allowing callers to stage this
    /// update with the new active-round header atomically.
    pub fn allocate_next_distribution_sequence(&mut self) -> Piv1Result<u64> {
        let allocated = self.next_distribution_sequence;
        let next = allocated
            .checked_add(1)
            .ok_or(Piv1Error::ArithmeticOverflow)?;
        self.next_distribution_sequence = next;
        Ok(allocated)
    }

    /// Increases the protected HWM with checked arithmetic.
    ///
    /// Eligibility of the supplied increase belongs to the surrounding
    /// transition; this helper only prevents wrapping or a downward update.
    pub fn checked_increase_protected_principal_hwm(
        &mut self,
        increase_lamports: u64,
    ) -> Piv1Result<u64> {
        let updated = self
            .protected_principal_hwm_lamports
            .checked_add(increase_lamports)
            .ok_or(Piv1Error::ArithmeticOverflow)?;
        self.protected_principal_hwm_lamports = updated;
        Ok(updated)
    }

    fn validate_addresses(&self) -> Piv1Result<()> {
        const MANAGER_FEE_INDEX: usize = 8;
        const REFERRER_INDEX: usize = 9;

        let addresses = self.bound_addresses();
        if addresses.iter().any(|address| *address == Pubkey::default()) {
            return Err(Piv1Error::InvalidAddress);
        }

        for left_index in 0..addresses.len() {
            for right_index in (left_index + 1)..addresses.len() {
                let manager_referrer_alias = left_index == MANAGER_FEE_INDEX
                    && right_index == REFERRER_INDEX;
                if addresses[left_index] == addresses[right_index] && !manager_referrer_alias {
                    return Err(Piv1Error::InvalidAddress);
                }
            }
        }

        Ok(())
    }

    fn bound_addresses(&self) -> [Pubkey; 22] {
        [
            self.stake_pool_program,
            self.stake_pool,
            self.validator_list,
            self.reserve_stake,
            self.jitosol_mint,
            self.token_program,
            self.stake_program,
            self.system_program,
            self.manager_fee_account,
            self.referrer_token_account,
            self.piv_authority,
            self.active_distribution,
            self.principal_jito_vault,
            self.pending_jito_vault,
            self.pending_sol_vault,
            self.principal_sol_queue,
            self.operational_sol_vault,
            self.distribution_escrow,
            self.kif_sol_vault,
            self.htfp_recipient,
            self.team_owner_recipient,
            self.guardian_registry,
        ]
    }

    fn validate_fixed_economics(&self) -> Piv1Result<()> {
        if u64::from(self.basis_points_denominator) != piv1_math::BASIS_POINTS_DENOMINATOR
            || u64::from(self.htfp_reserve_bps) != piv1_math::HTFP_RESERVE_BPS
            || u64::from(self.permanent_compound_bps) != piv1_math::PERMANENT_COMPOUND_BPS
            || u64::from(self.team_owner_pool_bps) != piv1_math::TEAM_OWNER_POOL_BPS
            || u64::from(self.kif_bps) != piv1_math::KIF_BPS
        {
            return Err(Piv1Error::InvalidSplit);
        }
        Ok(())
    }

    fn validate_slippage(&self) -> Piv1Result<()> {
        if self.slippage_hard_cap_bps != MAX_CONFIGURED_SLIPPAGE_BPS
            || self.configured_slippage_bps > MAX_CONFIGURED_SLIPPAGE_BPS
        {
            return Err(Piv1Error::InvalidSlippage);
        }
        Ok(())
    }

    fn validate_timing(&self) -> Piv1Result<()> {
        if self.minimum_distribution_interval_seconds
            != MINIMUM_DISTRIBUTION_INTERVAL_SECONDS
            || self.insufficient_retry_cooldown_seconds
                != INSUFFICIENT_RETRY_COOLDOWN_SECONDS
            || self.kif_period_seconds != KIF_PERIOD_SECONDS
        {
            return Err(Piv1Error::InvalidTimingConfiguration);
        }
        Ok(())
    }

    fn validate_reserved_bytes(&self) -> Piv1Result<()> {
        if self.migration_reserve.iter().any(|byte| *byte != 0) {
            return Err(Piv1Error::InvalidInitialization);
        }
        Ok(())
    }

    fn validate_kif_liability_reconciliation(&self) -> Piv1Result<()> {
        let expected_liability = self
            .cumulative_kif_credited_lamports
            .checked_sub(self.cumulative_kif_claimed_lamports)
            .ok_or(Piv1Error::CumulativeReconciliationMismatch)?;
        if expected_liability != self.kif_claim_liability_lamports {
            return Err(Piv1Error::CumulativeReconciliationMismatch);
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const EXPECTED_BUMP_SERIALIZED_SIZE: usize = 11;
    const EXPECTED_CONFIG_SERIALIZED_SIZE: usize = 1_006;

    fn key(tag: u8) -> Pubkey {
        Pubkey::new_from_array([tag; 32])
    }

    fn valid_config() -> PivConfig {
        PivConfig {
            version: STATE_LAYOUT_VERSION,
            is_initialized: true,
            paused: false,
            bumps: PivConfigBumps {
                config: 1,
                piv_authority: 2,
                active_distribution: 3,
                principal_jito_vault: 4,
                pending_jito_vault: 5,
                pending_sol_vault: 6,
                principal_sol_queue: 7,
                operational_sol_vault: 8,
                distribution_escrow: 9,
                kif_sol_vault: 10,
                guardian_registry: 11,
            },
            stake_pool_program: key(1),
            stake_pool: key(2),
            validator_list: key(3),
            reserve_stake: key(4),
            jitosol_mint: key(5),
            token_program: key(6),
            stake_program: key(7),
            system_program: key(8),
            manager_fee_account: key(9),
            referrer_token_account: key(10),
            piv_authority: key(11),
            active_distribution: key(12),
            principal_jito_vault: key(13),
            pending_jito_vault: key(14),
            pending_sol_vault: key(15),
            principal_sol_queue: key(16),
            operational_sol_vault: key(17),
            distribution_escrow: key(18),
            kif_sol_vault: key(19),
            htfp_recipient: key(20),
            team_owner_recipient: key(21),
            guardian_registry: key(22),
            basis_points_denominator: 10_000,
            htfp_reserve_bps: 5_900,
            permanent_compound_bps: 1_950,
            team_owner_pool_bps: 1_950,
            kif_bps: 200,
            configured_slippage_bps: 1,
            slippage_hard_cap_bps: 1,
            minimum_distribution_interval_seconds: MINIMUM_DISTRIBUTION_INTERVAL_SECONDS,
            insufficient_retry_cooldown_seconds: INSUFFICIENT_RETRY_COOLDOWN_SECONDS,
            last_successful_preparation_at: Some(1_000),
            last_valid_insufficient_attempt_at: Some(2_000),
            next_distribution_sequence: 3,
            protected_principal_hwm_lamports: 4,
            accounted_historical_jitosol_units: 5,
            accounted_historical_sol_lamports: 6,
            accounted_pending_jitosol_units: 7,
            accounted_pending_sol_lamports: 8,
            next_cycle_yield_lamports: 9,
            kif_claim_liability_lamports: 10,
            collective_kif_carry_lamports: 11,
            cumulative_contribution_value_lamports: 12,
            cumulative_gross_yield_lamports: 13,
            cumulative_htfp_paid_lamports: 14,
            cumulative_team_owner_paid_lamports: 15,
            cumulative_kif_credited_lamports: 17,
            cumulative_kif_claimed_lamports: 7,
            cumulative_permanent_compound_lamports: 18,
            cumulative_retained_dust_lamports: 19,
            cumulative_zero_active_kif_compound_lamports: 20,
            cumulative_cooldown_yield_recorded_lamports: 21,
            kif_anchor_timestamp: -1_000,
            kif_period_seconds: KIF_PERIOD_SECONDS,
            guardian_registry_revision: 1,
            migration_reserve: [0; CONFIG_MIGRATION_RESERVE_BYTES],
        }
    }

    #[test]
    fn config_layout_has_exact_fixed_maximum_space() {
        assert_eq!(
            PivConfigBumps::SERIALIZED_SIZE,
            EXPECTED_BUMP_SERIALIZED_SIZE
        );
        assert_eq!(PivConfig::SERIALIZED_SIZE, EXPECTED_CONFIG_SERIALIZED_SIZE);
        assert_eq!(
            PivConfig::SPACE,
            PLANNED_ACCOUNT_DISCRIMINATOR_BYTES + EXPECTED_CONFIG_SERIALIZED_SIZE
        );

        let encoded = valid_config().try_to_vec().expect("test serialization");
        assert_eq!(encoded.len(), EXPECTED_CONFIG_SERIALIZED_SIZE);
    }

    #[test]
    fn config_maximum_values_serialize_within_the_declared_bound() {
        let mut config = valid_config();
        config.bumps = PivConfigBumps {
            config: u8::MAX,
            piv_authority: u8::MAX,
            active_distribution: u8::MAX,
            principal_jito_vault: u8::MAX,
            pending_jito_vault: u8::MAX,
            pending_sol_vault: u8::MAX,
            principal_sol_queue: u8::MAX,
            operational_sol_vault: u8::MAX,
            distribution_escrow: u8::MAX,
            kif_sol_vault: u8::MAX,
            guardian_registry: u8::MAX,
        };
        config.basis_points_denominator = u16::MAX;
        config.htfp_reserve_bps = u16::MAX;
        config.permanent_compound_bps = u16::MAX;
        config.team_owner_pool_bps = u16::MAX;
        config.kif_bps = u16::MAX;
        config.configured_slippage_bps = u16::MAX;
        config.slippage_hard_cap_bps = u16::MAX;
        config.minimum_distribution_interval_seconds = i64::MAX;
        config.insufficient_retry_cooldown_seconds = i64::MAX;
        config.last_successful_preparation_at = Some(i64::MAX);
        config.last_valid_insufficient_attempt_at = Some(i64::MIN);
        config.next_distribution_sequence = u64::MAX;
        config.protected_principal_hwm_lamports = u64::MAX;
        config.accounted_historical_jitosol_units = u64::MAX;
        config.accounted_historical_sol_lamports = u64::MAX;
        config.accounted_pending_jitosol_units = u64::MAX;
        config.accounted_pending_sol_lamports = u64::MAX;
        config.next_cycle_yield_lamports = u64::MAX;
        config.kif_claim_liability_lamports = u64::MAX;
        config.collective_kif_carry_lamports = u64::MAX;
        config.cumulative_contribution_value_lamports = u64::MAX;
        config.cumulative_gross_yield_lamports = u64::MAX;
        config.cumulative_htfp_paid_lamports = u64::MAX;
        config.cumulative_team_owner_paid_lamports = u64::MAX;
        config.cumulative_kif_credited_lamports = u64::MAX;
        config.cumulative_kif_claimed_lamports = u64::MAX;
        config.cumulative_permanent_compound_lamports = u64::MAX;
        config.cumulative_retained_dust_lamports = u64::MAX;
        config.cumulative_zero_active_kif_compound_lamports = u64::MAX;
        config.cumulative_cooldown_yield_recorded_lamports = u64::MAX;
        config.kif_anchor_timestamp = i64::MAX;
        config.kif_period_seconds = i64::MAX;
        config.guardian_registry_revision = u64::MAX;
        config.migration_reserve = [u8::MAX; CONFIG_MIGRATION_RESERVE_BYTES];

        let encoded = config.try_to_vec().expect("test serialization");
        assert_eq!(encoded.len(), PivConfig::SERIALIZED_SIZE);
        let decoded = PivConfig::try_from_slice(&encoded).expect("test deserialization");
        assert_eq!(decoded, config);
    }

    #[test]
    fn config_round_trip_preserves_all_fields() {
        let config = valid_config();
        let encoded = config.try_to_vec().expect("test serialization");
        let decoded = PivConfig::try_from_slice(&encoded).expect("test deserialization");
        assert_eq!(decoded, config);
        assert_eq!(decoded.validate_initialized(), Ok(()));
    }

    #[test]
    fn absent_optional_timestamps_have_the_accounted_borsh_size() {
        let mut config = valid_config();
        config.last_successful_preparation_at = None;
        config.last_valid_insufficient_attempt_at = None;

        let encoded = config.try_to_vec().expect("test serialization");
        assert_eq!(encoded.len(), PivConfig::SERIALIZED_SIZE - (2 * 8));
        let decoded = PivConfig::try_from_slice(&encoded).expect("test deserialization");
        assert_eq!(decoded, config);
    }

    #[test]
    fn initialization_and_pause_validation_are_explicit() {
        let config = valid_config();
        assert_eq!(config.validate_initialized(), Ok(()));
        assert_eq!(config.ensure_unpaused(), Ok(()));

        let mut wrong_version = config.clone();
        wrong_version.version = STATE_LAYOUT_VERSION.saturating_add(1);
        assert_eq!(wrong_version.validate_initialized(), Err(Piv1Error::InvalidVersion));

        let mut uninitialized = config.clone();
        uninitialized.is_initialized = false;
        assert_eq!(
            uninitialized.validate_initialized(),
            Err(Piv1Error::InvalidInitialization)
        );

        let mut paused = config;
        paused.paused = true;
        assert_eq!(paused.validate_initialized(), Ok(()));
        assert_eq!(paused.ensure_unpaused(), Err(Piv1Error::PausedOperation));
    }

    #[test]
    fn initialized_addresses_must_be_nondefault_and_separated() {
        let config = valid_config();

        let mut default_recipient = config.clone();
        default_recipient.htfp_recipient = Pubkey::default();
        assert_eq!(
            default_recipient.validate_initialized(),
            Err(Piv1Error::InvalidAddress)
        );

        let mut merged_vaults = config.clone();
        merged_vaults.pending_jito_vault = merged_vaults.principal_jito_vault;
        assert_eq!(
            merged_vaults.validate_initialized(),
            Err(Piv1Error::InvalidAddress)
        );

        let mut fee_referrer_alias = config;
        fee_referrer_alias.referrer_token_account = fee_referrer_alias.manager_fee_account;
        assert_eq!(fee_referrer_alias.validate_initialized(), Ok(()));
    }

    #[test]
    fn fixed_economic_slippage_timing_and_reserved_bindings_are_enforced() {
        let config = valid_config();

        let mut zero_slippage = config.clone();
        zero_slippage.configured_slippage_bps = 0;
        assert_eq!(zero_slippage.validate_initialized(), Ok(()));

        let mut split = config.clone();
        split.kif_bps = split.kif_bps.checked_add(1).expect("test value");
        assert_eq!(split.validate_initialized(), Err(Piv1Error::InvalidSplit));

        let mut slippage = config.clone();
        slippage.configured_slippage_bps = MAX_CONFIGURED_SLIPPAGE_BPS + 1;
        assert_eq!(
            slippage.validate_initialized(),
            Err(Piv1Error::InvalidSlippage)
        );

        let mut hard_cap = config.clone();
        hard_cap.slippage_hard_cap_bps = 0;
        assert_eq!(
            hard_cap.validate_initialized(),
            Err(Piv1Error::InvalidSlippage)
        );

        let mut timing = config.clone();
        timing.kif_period_seconds = timing
            .kif_period_seconds
            .checked_add(1)
            .expect("test value");
        assert_eq!(
            timing.validate_initialized(),
            Err(Piv1Error::InvalidTimingConfiguration)
        );

        let mut reserved = config;
        reserved.migration_reserve[0] = 1;
        assert_eq!(
            reserved.validate_initialized(),
            Err(Piv1Error::InvalidInitialization)
        );
    }

    #[test]
    fn kif_liability_must_reconcile_with_cumulative_credits_and_claims() {
        let config = valid_config();

        let mut wrong_liability = config.clone();
        wrong_liability.kif_claim_liability_lamports = 11;
        assert_eq!(
            wrong_liability.validate_initialized(),
            Err(Piv1Error::CumulativeReconciliationMismatch)
        );

        let mut claims_exceed_credits = config;
        claims_exceed_credits.cumulative_kif_claimed_lamports = 18;
        assert_eq!(
            claims_exceed_credits.validate_initialized(),
            Err(Piv1Error::CumulativeReconciliationMismatch)
        );
    }

    #[test]
    fn sequence_and_hwm_helpers_are_checked_and_mutation_safe() {
        let mut config = valid_config();
        assert_eq!(config.allocate_next_distribution_sequence(), Ok(3));
        assert_eq!(config.next_distribution_sequence, 4);
        assert_eq!(
            config.checked_increase_protected_principal_hwm(6),
            Ok(10)
        );
        assert_eq!(config.protected_principal_hwm_lamports, 10);

        config.next_distribution_sequence = u64::MAX;
        let sequence_before = config.clone();
        assert_eq!(
            config.allocate_next_distribution_sequence(),
            Err(Piv1Error::ArithmeticOverflow)
        );
        assert_eq!(config, sequence_before);

        config.protected_principal_hwm_lamports = u64::MAX;
        let hwm_before = config.clone();
        assert_eq!(
            config.checked_increase_protected_principal_hwm(1),
            Err(Piv1Error::ArithmeticOverflow)
        );
        assert_eq!(config, hwm_before);
    }
}
