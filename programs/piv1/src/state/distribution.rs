//! Bounded reusable distribution header and one-leg metadata layout.
//!
//! The header stores cumulative counters only and never a validator or leg
//! collection. Conceptual predicates such as prepared, assigning, target
//! assigned, awaiting inactivity, and partially finalized are derived from a
//! coarse lifecycle phase plus exact counters, allowing earlier legs to finish
//! while later legs are still being assigned.

use std::io::{Read as _, Write as _};
use anchor_lang::prelude::{borsh, Pubkey};
use anchor_lang::{AnchorDeserialize, AnchorSerialize, InitSpace, Space};

use crate::{
    constants::{
        GUARDIAN_BITMAP_MASK, GUARDIAN_COUNT, MAX_CONFIGURED_SLIPPAGE_BPS,
        PLANNED_ACCOUNT_DISCRIMINATOR_BYTES, RECOVERY_FLAG_COOLDOWN_LOSS,
        RECOVERY_FLAG_RESIDUAL_HWM, STATE_LAYOUT_VERSION,
    },
    errors::{Piv1Error, Piv1Result},
};

const ALLOWED_RECOVERY_FLAGS: u8 = RECOVERY_FLAG_COOLDOWN_LOSS | RECOVERY_FLAG_RESIDUAL_HWM;

/// Deterministic Phase-0 allocation of the actually available beneficiary net.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct NetBeneficiaryAllocation {
    pub htfp_lamports: u64,
    pub team_owner_lamports: u64,
    pub kif_lamports: u64,
    pub dust_lamports: u64,
    pub allocated_lamports: u64,
}

/// Applies prior-cycle yield before comparing the historical asset value with
/// the protected high-water mark.
pub(crate) fn derive_gross_yield_lamports(
    historical_asset_value_lamports: u64,
    prior_next_cycle_yield_lamports: u64,
    protected_hwm_lamports: u64,
) -> Piv1Result<u64> {
    let gross_yield_basis = checked_add(
        historical_asset_value_lamports,
        prior_next_cycle_yield_lamports,
    )?;
    Ok(piv1_math::calculate_gross_yield(
        gross_yield_basis,
        protected_hwm_lamports,
    ))
}

/// Re-derives the founder-accepted Phase-0 relative-weight allocation.
pub(crate) fn derive_net_beneficiary_allocation(
    beneficiary_net_total_lamports: u64,
    htfp_gross_obligation_lamports: u64,
    team_owner_gross_obligation_lamports: u64,
    kif_gross_obligation_lamports: u64,
) -> Piv1Result<NetBeneficiaryAllocation> {
    let outgoing_weight = checked_add(
        checked_add(
            piv1_math::HTFP_RESERVE_BPS,
            piv1_math::TEAM_OWNER_POOL_BPS,
        )?,
        piv1_math::KIF_BPS,
    )?;
    let htfp_lamports = core::cmp::min(
        htfp_gross_obligation_lamports,
        piv1_math::checked_mul_div_floor(
            beneficiary_net_total_lamports,
            piv1_math::HTFP_RESERVE_BPS,
            outgoing_weight,
        )?,
    );
    let team_owner_lamports = core::cmp::min(
        team_owner_gross_obligation_lamports,
        piv1_math::checked_mul_div_floor(
            beneficiary_net_total_lamports,
            piv1_math::TEAM_OWNER_POOL_BPS,
            outgoing_weight,
        )?,
    );
    let kif_lamports = core::cmp::min(
        kif_gross_obligation_lamports,
        piv1_math::checked_mul_div_floor(
            beneficiary_net_total_lamports,
            piv1_math::KIF_BPS,
            outgoing_weight,
        )?,
    );
    let allocated_lamports = checked_add(
        checked_add(htfp_lamports, team_owner_lamports)?,
        kif_lamports,
    )?;
    let dust_lamports = checked_sub(
        beneficiary_net_total_lamports,
        allocated_lamports,
    )?;

    Ok(NetBeneficiaryAllocation {
        htfp_lamports,
        team_owner_lamports,
        kif_lamports,
        dust_lamports,
        allocated_lamports,
    })
}

/// Stored coarse phase for the one reusable active-round header.
#[derive(
    AnchorSerialize, AnchorDeserialize, InitSpace, Clone, Copy, Debug, Eq, PartialEq,
)]
pub enum DistributionLifecycle {
    /// No active round; only an optional prior terminal summary remains.
    Idle,
    /// A nonzero fixed JitoSOL target is being assigned and/or finalized.
    WithdrawalActive,
    /// Fixed liquid or finalized withdrawal proceeds reconcile in escrow.
    EscrowFunded,
    /// Atomic settlement accounting has been recorded.
    Settled,
    /// Normal progress is blocked pending governed recovery.
    RecoveryRequired,
}

/// Persistent bounded summary retained when the reusable header returns Idle.
#[derive(
    AnchorSerialize, AnchorDeserialize, InitSpace, Clone, Copy, Debug, Eq, PartialEq,
)]
pub struct CompletedDistributionSummary {
    pub sequence: u64,
    pub completed_at: i64,
    pub gross_yield_lamports: u64,
    pub actual_allocated_outgoing_lamports: u64,
    pub integrated_contribution_value_lamports: u64,
    pub final_protected_hwm_lamports: u64,
    pub fixed_jitosol_withdrawal_target_units: u64,
    pub successful_leg_count: u64,
    pub cumulative_cooldown_rewards_lamports: u64,
    pub actual_kif_liability_lamports: u64,
    pub actual_kif_carry_next_lamports: u64,
}

/// One reusable bounded active distribution header.
#[derive(
    InitSpace, Clone, Copy, Debug, Eq, PartialEq,
)]
pub struct ActiveDistribution {
    pub version: u8,
    pub bump: u8,
    pub is_initialized: bool,
    pub lifecycle: DistributionLifecycle,
    pub recovery_flags: u8,
    pub settlement_recorded: bool,
    pub active_sequence: u64,
    pub last_completed: Option<CompletedDistributionSummary>,

    // Immutable preparation snapshot.
    pub prepared_at: i64,
    pub prepared_slot: u64,
    pub prepared_epoch: u64,
    pub old_protected_principal_lamports: u64,
    pub historical_jitosol_units: u64,
    pub historical_sol_lamports: u64,
    /// Historical SOL-equivalent asset value before prior-cycle yield carry.
    pub historical_value_lamports: u64,
    pub snapshot_pool_total_lamports: u64,
    pub snapshot_pool_token_supply: u64,
    pub snapshot_withdrawal_fee_numerator: u64,
    pub snapshot_withdrawal_fee_denominator: u64,

    // Immutable gross economics and liquid-input snapshot.
    pub gross_yield_lamports: u64,
    /// Previously recorded cooldown yield consumed into this cycle's gross split.
    pub prior_next_cycle_yield_lamports: u64,
    pub htfp_gross_obligation_lamports: u64,
    pub permanent_compound_lamports: u64,
    pub team_owner_gross_obligation_lamports: u64,
    pub kif_gross_obligation_lamports: u64,
    pub split_dust_lamports: u64,
    pub outgoing_gross_obligation_lamports: u64,
    pub pending_sol_snapshot_lamports: u64,
    pub pending_sol_used_lamports: u64,
    pub snapshot_conversion_dust_lamports: u64,

    // Immutable withdrawal bounds. A liquid-only round stores a zero target.
    pub fixed_jitosol_withdrawal_target_units: u64,
    pub snapshot_leg_input_floor_units: u64,
    pub maximum_useful_legs: u64,
    pub stored_round_minimum_native_lamports: u64,
    pub stored_residual_hwm_floor_lamports: u64,
    pub stored_slippage_bps: u16,

    // Checked cumulative per-leg accounting.
    pub cumulative_jitosol_assigned_units: u64,
    pub cumulative_withdrawal_fee_units: u64,
    pub cumulative_burned_units: u64,
    pub cumulative_expected_native_lamports: u64,
    pub cumulative_delegated_native_lamports: u64,
    pub cumulative_finalized_delegated_native_lamports: u64,
    pub cumulative_finalized_native_lamports: u64,
    pub cumulative_recovered_stake_rent_lamports: u64,
    pub cumulative_recovered_metadata_rent_lamports: u64,
    pub cumulative_cooldown_rewards_lamports: u64,
    pub cumulative_cooldown_losses_lamports: u64,
    pub next_leg_index: u64,
    pub successful_leg_count: u64,
    pub finalized_leg_count: u64,

    // Fixed escrow, recipient, and KIF snapshot bindings.
    pub recorded_escrow_available_lamports: u64,
    pub outstanding_active_round_liability_lamports: u64,
    pub htfp_recipient: Pubkey,
    pub team_owner_recipient: Pubkey,
    pub guardian_registry: Pubkey,
    pub guardian_registry_revision: u64,
    pub guardian_keys: [Pubkey; GUARDIAN_COUNT],
    pub kif_eligibility_bitmap: u8,
    pub kif_active_guardian_count: u8,
    pub kif_period_id: u64,
    pub kif_carry_input_lamports: u64,

    // Proposed and actual settlement/HWM accounting.
    pub proposed_hwm_delta_lamports: u64,
    pub proposed_hwm_after_settlement_lamports: u64,
    pub actual_net_available_lamports: u64,
    pub actual_htfp_lamports: u64,
    pub actual_team_owner_lamports: u64,
    pub actual_kif_allocation_lamports: u64,
    pub actual_net_allocation_dust_lamports: u64,
    pub actual_allocated_outgoing_lamports: u64,
    pub actual_escrow_remainder_lamports: u64,
    pub actual_retained_conservative_dust_lamports: u64,
    pub actual_kif_liability_lamports: u64,
    pub actual_kif_carry_next_lamports: u64,
    pub actual_zero_active_kif_compound_lamports: u64,
    pub actual_hwm_delta_lamports: u64,
    pub settled_protected_hwm_lamports: u64,
    /// V2 only: factual swap funding, never withdrawal burns or leg counters.
    pub swap: Option<crate::integrations::jupiter::SwapReceipt>,
    /// V3 only: factual incidents and exact one-generation acknowledgement.
    pub recovery: Option<super::recovery::RecoveryReceipt>,
    /// V3 only: retained completed loss/action history, never active authority.
    pub last_completed_recovery: Option<super::recovery::CompletedRecoverySummary>,
}

// Count streaming bytes without allocating a second serialized round. Needed
// for the fixed V2 prefix; the SBF bump allocator does not reclaim temporaries.
struct CountingWriter<'a,W>{inner:&'a mut W,bytes:usize}
impl<W:std::io::Write> std::io::Write for CountingWriter<'_,W>{
    fn write(&mut self,buf:&[u8])->std::io::Result<usize>{let n=self.inner.write(buf)?;self.bytes+=n;Ok(n)}
    fn flush(&mut self)->std::io::Result<()>{self.inner.flush()}
}
struct CountingReader<'a,R>{inner:&'a mut R,bytes:usize}
impl<R:std::io::Read> std::io::Read for CountingReader<'_,R>{
    fn read(&mut self,buf:&mut[u8])->std::io::Result<usize>{let n=self.inner.read(buf)?;self.bytes+=n;Ok(n)}
}

// Explicit versioned Borsh codec. V1 emits/reads its original exact field list;
// only migrated V2 rounds carry the appended bounded swap receipt. No Config or
// withdrawal-leg schema is reinterpreted. Fixed-account decoding checks padding.
impl AnchorSerialize for ActiveDistribution {
    fn serialize<W: std::io::Write>(&self, writer: &mut W) -> std::io::Result<()> {
        let mut counted = CountingWriter { inner: writer, bytes: 0 };
        let writer = &mut counted;
        self.version.serialize(writer)?;
        self.bump.serialize(writer)?;
        self.is_initialized.serialize(writer)?;
        self.lifecycle.serialize(writer)?;
        self.recovery_flags.serialize(writer)?;
        self.settlement_recorded.serialize(writer)?;
        self.active_sequence.serialize(writer)?;
        self.last_completed.serialize(writer)?;
        self.prepared_at.serialize(writer)?;
        self.prepared_slot.serialize(writer)?;
        self.prepared_epoch.serialize(writer)?;
        self.old_protected_principal_lamports.serialize(writer)?;
        self.historical_jitosol_units.serialize(writer)?;
        self.historical_sol_lamports.serialize(writer)?;
        self.historical_value_lamports.serialize(writer)?;
        self.snapshot_pool_total_lamports.serialize(writer)?;
        self.snapshot_pool_token_supply.serialize(writer)?;
        self.snapshot_withdrawal_fee_numerator.serialize(writer)?;
        self.snapshot_withdrawal_fee_denominator.serialize(writer)?;
        self.gross_yield_lamports.serialize(writer)?;
        self.prior_next_cycle_yield_lamports.serialize(writer)?;
        self.htfp_gross_obligation_lamports.serialize(writer)?;
        self.permanent_compound_lamports.serialize(writer)?;
        self.team_owner_gross_obligation_lamports.serialize(writer)?;
        self.kif_gross_obligation_lamports.serialize(writer)?;
        self.split_dust_lamports.serialize(writer)?;
        self.outgoing_gross_obligation_lamports.serialize(writer)?;
        self.pending_sol_snapshot_lamports.serialize(writer)?;
        self.pending_sol_used_lamports.serialize(writer)?;
        self.snapshot_conversion_dust_lamports.serialize(writer)?;
        self.fixed_jitosol_withdrawal_target_units.serialize(writer)?;
        self.snapshot_leg_input_floor_units.serialize(writer)?;
        self.maximum_useful_legs.serialize(writer)?;
        self.stored_round_minimum_native_lamports.serialize(writer)?;
        self.stored_residual_hwm_floor_lamports.serialize(writer)?;
        self.stored_slippage_bps.serialize(writer)?;
        self.cumulative_jitosol_assigned_units.serialize(writer)?;
        self.cumulative_withdrawal_fee_units.serialize(writer)?;
        self.cumulative_burned_units.serialize(writer)?;
        self.cumulative_expected_native_lamports.serialize(writer)?;
        self.cumulative_delegated_native_lamports.serialize(writer)?;
        self.cumulative_finalized_delegated_native_lamports.serialize(writer)?;
        self.cumulative_finalized_native_lamports.serialize(writer)?;
        self.cumulative_recovered_stake_rent_lamports.serialize(writer)?;
        self.cumulative_recovered_metadata_rent_lamports.serialize(writer)?;
        self.cumulative_cooldown_rewards_lamports.serialize(writer)?;
        self.cumulative_cooldown_losses_lamports.serialize(writer)?;
        self.next_leg_index.serialize(writer)?;
        self.successful_leg_count.serialize(writer)?;
        self.finalized_leg_count.serialize(writer)?;
        self.recorded_escrow_available_lamports.serialize(writer)?;
        self.outstanding_active_round_liability_lamports.serialize(writer)?;
        self.htfp_recipient.serialize(writer)?;
        self.team_owner_recipient.serialize(writer)?;
        self.guardian_registry.serialize(writer)?;
        self.guardian_registry_revision.serialize(writer)?;
        self.guardian_keys.serialize(writer)?;
        self.kif_eligibility_bitmap.serialize(writer)?;
        self.kif_active_guardian_count.serialize(writer)?;
        self.kif_period_id.serialize(writer)?;
        self.kif_carry_input_lamports.serialize(writer)?;
        self.proposed_hwm_delta_lamports.serialize(writer)?;
        self.proposed_hwm_after_settlement_lamports.serialize(writer)?;
        self.actual_net_available_lamports.serialize(writer)?;
        self.actual_htfp_lamports.serialize(writer)?;
        self.actual_team_owner_lamports.serialize(writer)?;
        self.actual_kif_allocation_lamports.serialize(writer)?;
        self.actual_net_allocation_dust_lamports.serialize(writer)?;
        self.actual_allocated_outgoing_lamports.serialize(writer)?;
        self.actual_escrow_remainder_lamports.serialize(writer)?;
        self.actual_retained_conservative_dust_lamports.serialize(writer)?;
        self.actual_kif_liability_lamports.serialize(writer)?;
        self.actual_kif_carry_next_lamports.serialize(writer)?;
        self.actual_zero_active_kif_compound_lamports.serialize(writer)?;
        self.actual_hwm_delta_lamports.serialize(writer)?;
        self.settled_protected_hwm_lamports.serialize(writer)?;
        if self.version >= Self::SWAP_VERSION { self.swap.serialize(writer)?; }
        else if self.swap.is_some() { return Err(std::io::Error::new(std::io::ErrorKind::InvalidData, "V1 swap receipt")); }
        if self.version == Self::RECOVERY_VERSION {
            let padding = (Self::SWAP_SPACE - 8).checked_sub(writer.bytes)
                .ok_or_else(|| std::io::Error::new(std::io::ErrorKind::InvalidData, "V2 prefix overflow"))?;
            // Only bounded Option payloads can be absent: summary88 + swap32.
            if padding > 120 { return Err(std::io::Error::new(std::io::ErrorKind::InvalidData, "V2 prefix underflow")); }
            writer.write_all(&[0; 120][..padding])?;
            self.recovery.serialize(writer)?;
            self.last_completed_recovery.serialize(writer)?;
        } else if self.recovery.is_some() || self.last_completed_recovery.is_some() {
            return Err(std::io::Error::new(std::io::ErrorKind::InvalidData, "legacy recovery receipt"));
        }
        Ok(())
    }
}
impl AnchorDeserialize for ActiveDistribution {
    fn deserialize_reader<R: std::io::Read>(reader: &mut R) -> std::io::Result<Self> {
        let mut counted = CountingReader { inner: reader, bytes: 0 };
        let reader = &mut counted;
        let version = u8::deserialize_reader(reader)?;
        let mut value = Self {
            version,
            bump: AnchorDeserialize::deserialize_reader(reader)?,
            is_initialized: AnchorDeserialize::deserialize_reader(reader)?,
            lifecycle: AnchorDeserialize::deserialize_reader(reader)?,
            recovery_flags: AnchorDeserialize::deserialize_reader(reader)?,
            settlement_recorded: AnchorDeserialize::deserialize_reader(reader)?,
            active_sequence: AnchorDeserialize::deserialize_reader(reader)?,
            last_completed: AnchorDeserialize::deserialize_reader(reader)?,
            prepared_at: AnchorDeserialize::deserialize_reader(reader)?,
            prepared_slot: AnchorDeserialize::deserialize_reader(reader)?,
            prepared_epoch: AnchorDeserialize::deserialize_reader(reader)?,
            old_protected_principal_lamports: AnchorDeserialize::deserialize_reader(reader)?,
            historical_jitosol_units: AnchorDeserialize::deserialize_reader(reader)?,
            historical_sol_lamports: AnchorDeserialize::deserialize_reader(reader)?,
            historical_value_lamports: AnchorDeserialize::deserialize_reader(reader)?,
            snapshot_pool_total_lamports: AnchorDeserialize::deserialize_reader(reader)?,
            snapshot_pool_token_supply: AnchorDeserialize::deserialize_reader(reader)?,
            snapshot_withdrawal_fee_numerator: AnchorDeserialize::deserialize_reader(reader)?,
            snapshot_withdrawal_fee_denominator: AnchorDeserialize::deserialize_reader(reader)?,
            gross_yield_lamports: AnchorDeserialize::deserialize_reader(reader)?,
            prior_next_cycle_yield_lamports: AnchorDeserialize::deserialize_reader(reader)?,
            htfp_gross_obligation_lamports: AnchorDeserialize::deserialize_reader(reader)?,
            permanent_compound_lamports: AnchorDeserialize::deserialize_reader(reader)?,
            team_owner_gross_obligation_lamports: AnchorDeserialize::deserialize_reader(reader)?,
            kif_gross_obligation_lamports: AnchorDeserialize::deserialize_reader(reader)?,
            split_dust_lamports: AnchorDeserialize::deserialize_reader(reader)?,
            outgoing_gross_obligation_lamports: AnchorDeserialize::deserialize_reader(reader)?,
            pending_sol_snapshot_lamports: AnchorDeserialize::deserialize_reader(reader)?,
            pending_sol_used_lamports: AnchorDeserialize::deserialize_reader(reader)?,
            snapshot_conversion_dust_lamports: AnchorDeserialize::deserialize_reader(reader)?,
            fixed_jitosol_withdrawal_target_units: AnchorDeserialize::deserialize_reader(reader)?,
            snapshot_leg_input_floor_units: AnchorDeserialize::deserialize_reader(reader)?,
            maximum_useful_legs: AnchorDeserialize::deserialize_reader(reader)?,
            stored_round_minimum_native_lamports: AnchorDeserialize::deserialize_reader(reader)?,
            stored_residual_hwm_floor_lamports: AnchorDeserialize::deserialize_reader(reader)?,
            stored_slippage_bps: AnchorDeserialize::deserialize_reader(reader)?,
            cumulative_jitosol_assigned_units: AnchorDeserialize::deserialize_reader(reader)?,
            cumulative_withdrawal_fee_units: AnchorDeserialize::deserialize_reader(reader)?,
            cumulative_burned_units: AnchorDeserialize::deserialize_reader(reader)?,
            cumulative_expected_native_lamports: AnchorDeserialize::deserialize_reader(reader)?,
            cumulative_delegated_native_lamports: AnchorDeserialize::deserialize_reader(reader)?,
            cumulative_finalized_delegated_native_lamports: AnchorDeserialize::deserialize_reader(reader)?,
            cumulative_finalized_native_lamports: AnchorDeserialize::deserialize_reader(reader)?,
            cumulative_recovered_stake_rent_lamports: AnchorDeserialize::deserialize_reader(reader)?,
            cumulative_recovered_metadata_rent_lamports: AnchorDeserialize::deserialize_reader(reader)?,
            cumulative_cooldown_rewards_lamports: AnchorDeserialize::deserialize_reader(reader)?,
            cumulative_cooldown_losses_lamports: AnchorDeserialize::deserialize_reader(reader)?,
            next_leg_index: AnchorDeserialize::deserialize_reader(reader)?,
            successful_leg_count: AnchorDeserialize::deserialize_reader(reader)?,
            finalized_leg_count: AnchorDeserialize::deserialize_reader(reader)?,
            recorded_escrow_available_lamports: AnchorDeserialize::deserialize_reader(reader)?,
            outstanding_active_round_liability_lamports: AnchorDeserialize::deserialize_reader(reader)?,
            htfp_recipient: AnchorDeserialize::deserialize_reader(reader)?,
            team_owner_recipient: AnchorDeserialize::deserialize_reader(reader)?,
            guardian_registry: AnchorDeserialize::deserialize_reader(reader)?,
            guardian_registry_revision: AnchorDeserialize::deserialize_reader(reader)?,
            guardian_keys: AnchorDeserialize::deserialize_reader(reader)?,
            kif_eligibility_bitmap: AnchorDeserialize::deserialize_reader(reader)?,
            kif_active_guardian_count: AnchorDeserialize::deserialize_reader(reader)?,
            kif_period_id: AnchorDeserialize::deserialize_reader(reader)?,
            kif_carry_input_lamports: AnchorDeserialize::deserialize_reader(reader)?,
            proposed_hwm_delta_lamports: AnchorDeserialize::deserialize_reader(reader)?,
            proposed_hwm_after_settlement_lamports: AnchorDeserialize::deserialize_reader(reader)?,
            actual_net_available_lamports: AnchorDeserialize::deserialize_reader(reader)?,
            actual_htfp_lamports: AnchorDeserialize::deserialize_reader(reader)?,
            actual_team_owner_lamports: AnchorDeserialize::deserialize_reader(reader)?,
            actual_kif_allocation_lamports: AnchorDeserialize::deserialize_reader(reader)?,
            actual_net_allocation_dust_lamports: AnchorDeserialize::deserialize_reader(reader)?,
            actual_allocated_outgoing_lamports: AnchorDeserialize::deserialize_reader(reader)?,
            actual_escrow_remainder_lamports: AnchorDeserialize::deserialize_reader(reader)?,
            actual_retained_conservative_dust_lamports: AnchorDeserialize::deserialize_reader(reader)?,
            actual_kif_liability_lamports: AnchorDeserialize::deserialize_reader(reader)?,
            actual_kif_carry_next_lamports: AnchorDeserialize::deserialize_reader(reader)?,
            actual_zero_active_kif_compound_lamports: AnchorDeserialize::deserialize_reader(reader)?,
            actual_hwm_delta_lamports: AnchorDeserialize::deserialize_reader(reader)?,
            settled_protected_hwm_lamports: AnchorDeserialize::deserialize_reader(reader)?,
            swap: None,
            recovery: None,
            last_completed_recovery: None,
        };
        if version >= Self::SWAP_VERSION { value.swap = AnchorDeserialize::deserialize_reader(reader)?; }
        if version == Self::RECOVERY_VERSION {
            let padding_len = (Self::SWAP_SPACE - 8).checked_sub(reader.bytes)
                .ok_or_else(|| std::io::Error::new(std::io::ErrorKind::InvalidData, "V2 prefix overflow"))?;
            if padding_len > 120 { return Err(std::io::Error::new(std::io::ErrorKind::InvalidData, "V2 prefix underflow")); }
            let mut padding = [0; 120];
            reader.read_exact(&mut padding[..padding_len])?;
            if padding[..padding_len].iter().any(|b| *b != 0) {
                return Err(std::io::Error::new(std::io::ErrorKind::InvalidData, "V2 prefix padding"));
            }
            value.recovery = AnchorDeserialize::deserialize_reader(reader)?;
            value.last_completed_recovery = AnchorDeserialize::deserialize_reader(reader)?;
        }
        Ok(value)
    }
}

impl ActiveDistribution {
    /// Maximum serialized Borsh payload size, excluding a discriminator.
    pub const SERIALIZED_SIZE: usize = 883;
    /// Planned discriminator-inclusive Anchor account allocation.
    pub const SPACE: usize = PLANNED_ACCOUNT_DISCRIMINATOR_BYTES + Self::SERIALIZED_SIZE;

    pub const SWAP_VERSION: u8 = 2;
    pub const SWAP_SPACE: usize = Self::SPACE + 33;
    pub const RECOVERY_VERSION: u8 = 3;
    pub const RECOVERY_SPACE: usize = Self::SWAP_SPACE
        + 1 + <super::recovery::RecoveryReceipt as Space>::INIT_SPACE
        + 1 + <super::recovery::CompletedRecoverySummary as Space>::INIT_SPACE;
    pub fn allocated_space(&self) -> usize {
        if self.version == Self::RECOVERY_VERSION { Self::RECOVERY_SPACE }
        else if self.version == Self::SWAP_VERSION { Self::SWAP_SPACE } else { Self::SPACE }
    }

    /// Creates an explicitly initialized reusable header in the Idle phase.
    pub fn new_idle(bump: u8) -> Self {
        Self::idle_with_summary(bump, None)
    }

    /// Returns the unassigned portion of the immutable withdrawal target.
    pub fn remaining_withdrawal_target_units(&self) -> Piv1Result<u64> {
        self.fixed_jitosol_withdrawal_target_units
            .checked_sub(self.cumulative_jitosol_assigned_units)
            .ok_or(Piv1Error::TargetExceeded)
    }

    /// Prior-cycle yield used as liquid funding after pending SOL is used first.
    pub fn prior_next_cycle_yield_used_lamports(&self) -> Piv1Result<u64> {
        let remaining_after_pending = self
            .outgoing_gross_obligation_lamports
            .checked_sub(self.pending_sol_used_lamports)
            .ok_or(Piv1Error::CumulativeReconciliationMismatch)?;
        Ok(core::cmp::min(
            self.prior_next_cycle_yield_lamports,
            remaining_after_pending,
        ))
    }

    /// True only before any successful withdrawal leg has assigned the target.
    pub fn is_prepared_withdrawal(&self) -> bool {
        self.lifecycle == DistributionLifecycle::WithdrawalActive
            && self.cumulative_jitosol_assigned_units == 0
            && self.fixed_jitosol_withdrawal_target_units > 0
    }

    /// True while a nonzero target remains after at least one assignment.
    pub fn is_assigning_withdrawal_legs(&self) -> bool {
        self.lifecycle == DistributionLifecycle::WithdrawalActive
            && self.cumulative_jitosol_assigned_units > 0
            && self.cumulative_jitosol_assigned_units
                < self.fixed_jitosol_withdrawal_target_units
    }

    /// Exact target-assignment predicate; a liquid-only round is not assigned.
    pub fn is_withdrawal_target_assigned(&self) -> bool {
        self.fixed_jitosol_withdrawal_target_units > 0
            && self.cumulative_jitosol_assigned_units
                == self.fixed_jitosol_withdrawal_target_units
    }

    /// True when one or more successful stake legs remain unfinalized.
    pub fn is_awaiting_leg_inactivity(&self) -> bool {
        self.successful_leg_count > self.finalized_leg_count
    }

    /// True after at least one finalization until the whole withdrawal completes.
    pub fn is_partially_finalized(&self) -> bool {
        self.finalized_leg_count > 0 && !self.is_withdrawal_complete()
    }

    /// True when at least one successful leg exists and all such legs finalized.
    pub fn are_all_successful_legs_finalized(&self) -> bool {
        self.successful_leg_count > 0
            && self.successful_leg_count == self.finalized_leg_count
    }

    /// Complete withdrawal predicate used before escrow funding and settlement.
    pub fn is_withdrawal_complete(&self) -> bool {
        self.is_withdrawal_target_assigned() && self.are_all_successful_legs_finalized()
    }

    /// Validates the normalized layout and every internally derivable identity.
    pub fn validate(&self) -> Piv1Result<()> {
        if ![STATE_LAYOUT_VERSION, Self::SWAP_VERSION, Self::RECOVERY_VERSION].contains(&self.version) {
            return Err(Piv1Error::InvalidVersion);
        }
        if !self.is_initialized {
            return Err(Piv1Error::InvalidInitialization);
        }

        self.validate_recovery_extension()?;
        if self.lifecycle == DistributionLifecycle::Idle {
            let mut expected = Self::idle_with_summary(self.bump, self.last_completed);
            expected.version = self.version;
            expected.last_completed_recovery = self.last_completed_recovery;
            if *self != expected {
                return Err(Piv1Error::InvalidLifecycle);
            }
            return Ok(());
        }

        self.validate_swap_funding()?;
        self.validate_active_snapshot()?;
        self.validate_withdrawal_accounting()?;

        match self.lifecycle {
            DistributionLifecycle::Idle => Err(Piv1Error::InvalidLifecycle),
            DistributionLifecycle::WithdrawalActive => {
                if self.fixed_jitosol_withdrawal_target_units == 0
                    || self.is_withdrawal_complete()
                    || self.settlement_recorded
                    || !self.recovery_flags_resolved()
                {
                    return Err(Piv1Error::InvalidLifecycle);
                }
                self.require_actual_settlement_fields_zero()
            }
            DistributionLifecycle::EscrowFunded => {
                if self.fixed_jitosol_withdrawal_target_units > 0
                    && !self.is_withdrawal_complete()
                {
                    return Err(Piv1Error::TargetNotAssigned);
                }
                if self.settlement_recorded || !self.recovery_flags_resolved() {
                    return Err(Piv1Error::InvalidLifecycle);
                }
                self.require_actual_settlement_fields_zero()
            }
            DistributionLifecycle::Settled => {
                if self.outstanding_active_round_liability_lamports != 0 {
                    return Err(Piv1Error::OutstandingLiability);
                }
                if !self.settlement_recorded || !self.recovery_flags_resolved() {
                    return Err(Piv1Error::InvalidLifecycle);
                }
                if self.fixed_jitosol_withdrawal_target_units > 0
                    && !self.is_withdrawal_complete()
                {
                    return Err(Piv1Error::TargetNotAssigned);
                }
                self.validate_settlement_accounting()
            }
            DistributionLifecycle::RecoveryRequired => {
                if self.recovery_flags == 0
                    || self.recovery_flags & !ALLOWED_RECOVERY_FLAGS != 0
                    || self.settlement_recorded
                {
                    return Err(Piv1Error::InvalidLifecycle);
                }
                if self.cumulative_cooldown_losses_lamports > 0
                    && self.recovery_flags & RECOVERY_FLAG_COOLDOWN_LOSS == 0
                {
                    return Err(Piv1Error::CumulativeReconciliationMismatch);
                }
                self.require_actual_settlement_fields_zero()
            }
        }
    }

    pub(crate) fn idle_after_completion(
        bump: u8,
        summary: CompletedDistributionSummary,
    ) -> Self {
        Self::idle_with_summary(bump, Some(summary))
    }

    fn idle_with_summary(
        bump: u8,
        last_completed: Option<CompletedDistributionSummary>,
    ) -> Self {
        Self {
            version: STATE_LAYOUT_VERSION,
            bump,
            is_initialized: true,
            lifecycle: DistributionLifecycle::Idle,
            recovery_flags: 0,
            settlement_recorded: false,
            active_sequence: 0,
            last_completed,
            prepared_at: 0,
            prepared_slot: 0,
            prepared_epoch: 0,
            old_protected_principal_lamports: 0,
            historical_jitosol_units: 0,
            historical_sol_lamports: 0,
            historical_value_lamports: 0,
            snapshot_pool_total_lamports: 0,
            snapshot_pool_token_supply: 0,
            snapshot_withdrawal_fee_numerator: 0,
            snapshot_withdrawal_fee_denominator: 0,
            gross_yield_lamports: 0,
            prior_next_cycle_yield_lamports: 0,
            htfp_gross_obligation_lamports: 0,
            permanent_compound_lamports: 0,
            team_owner_gross_obligation_lamports: 0,
            kif_gross_obligation_lamports: 0,
            split_dust_lamports: 0,
            outgoing_gross_obligation_lamports: 0,
            pending_sol_snapshot_lamports: 0,
            pending_sol_used_lamports: 0,
            snapshot_conversion_dust_lamports: 0,
            fixed_jitosol_withdrawal_target_units: 0,
            snapshot_leg_input_floor_units: 0,
            maximum_useful_legs: 0,
            stored_round_minimum_native_lamports: 0,
            stored_residual_hwm_floor_lamports: 0,
            stored_slippage_bps: 0,
            cumulative_jitosol_assigned_units: 0,
            cumulative_withdrawal_fee_units: 0,
            cumulative_burned_units: 0,
            cumulative_expected_native_lamports: 0,
            cumulative_delegated_native_lamports: 0,
            cumulative_finalized_delegated_native_lamports: 0,
            cumulative_finalized_native_lamports: 0,
            cumulative_recovered_stake_rent_lamports: 0,
            cumulative_recovered_metadata_rent_lamports: 0,
            cumulative_cooldown_rewards_lamports: 0,
            cumulative_cooldown_losses_lamports: 0,
            next_leg_index: 0,
            successful_leg_count: 0,
            finalized_leg_count: 0,
            recorded_escrow_available_lamports: 0,
            outstanding_active_round_liability_lamports: 0,
            htfp_recipient: Pubkey::default(),
            team_owner_recipient: Pubkey::default(),
            guardian_registry: Pubkey::default(),
            guardian_registry_revision: 0,
            guardian_keys: [Pubkey::default(); GUARDIAN_COUNT],
            kif_eligibility_bitmap: 0,
            kif_active_guardian_count: 0,
            kif_period_id: 0,
            kif_carry_input_lamports: 0,
            proposed_hwm_delta_lamports: 0,
            proposed_hwm_after_settlement_lamports: 0,
            actual_net_available_lamports: 0,
            actual_htfp_lamports: 0,
            actual_team_owner_lamports: 0,
            actual_kif_allocation_lamports: 0,
            actual_net_allocation_dust_lamports: 0,
            actual_allocated_outgoing_lamports: 0,
            actual_escrow_remainder_lamports: 0,
            actual_retained_conservative_dust_lamports: 0,
            actual_kif_liability_lamports: 0,
            actual_kif_carry_next_lamports: 0,
            actual_zero_active_kif_compound_lamports: 0,
            actual_hwm_delta_lamports: 0,
            settled_protected_hwm_lamports: 0,
            swap: None,
            recovery: None,
            last_completed_recovery: None,
        }
    }

    fn validate_swap_funding(&self) -> Piv1Result<()> {
        if let Some(receipt) = self.swap {
            use crate::integrations::jupiter::{liquidity_plan, validate_receipt, LiquidityRoute};
            if self.version < Self::SWAP_VERSION || self.fixed_jitosol_withdrawal_target_units != 0 {
                return Err(Piv1Error::InvalidSwapInput);
            }
            let plan = liquidity_plan(self.outgoing_gross_obligation_lamports,
                self.pending_sol_snapshot_lamports, self.prior_next_cycle_yield_lamports);
            if plan.route != LiquidityRoute::Jupiter { return Err(Piv1Error::InvalidSwapInput); }
            let verified = validate_receipt(self.historical_jitosol_units, receipt.spent_units,
                self.snapshot_pool_total_lamports, self.snapshot_pool_token_supply,
                plan.shortfall_lamports, receipt.actual_native_output_lamports)?;
            if verified != receipt || self.snapshot_conversion_dust_lamports !=
                plan.shortfall_lamports - receipt.official_book_debit_lamports {
                return Err(Piv1Error::CumulativeReconciliationMismatch);
            }
        }
        Ok(())
    }

    /// Actual distinct swap funding, never reported as finalized stake proceeds.
    pub fn swap_native_output(&self) -> u64 {
        self.swap.map_or(0, |r| r.actual_native_output_lamports)
    }
    pub fn swap_spent_units(&self) -> u64 { self.swap.map_or(0, |r| r.spent_units) }

    fn validate_active_snapshot(&self) -> Piv1Result<()> {
        if self.htfp_recipient == Pubkey::default()
            || self.team_owner_recipient == Pubkey::default()
            || self.guardian_registry == Pubkey::default()
        {
            return Err(Piv1Error::InvalidAddress);
        }
        if self.htfp_recipient == self.team_owner_recipient
            || self.htfp_recipient == self.guardian_registry
            || self.team_owner_recipient == self.guardian_registry
        {
            return Err(Piv1Error::InvalidAddress);
        }
        if self.stored_slippage_bps > MAX_CONFIGURED_SLIPPAGE_BPS {
            return Err(Piv1Error::InvalidSlippage);
        }
        if self.snapshot_pool_total_lamports == 0 || self.snapshot_pool_token_supply == 0 {
            return Err(Piv1Error::InvalidInitialization);
        }

        for (index, guardian) in self.guardian_keys.iter().enumerate() {
            if *guardian == Pubkey::default()
                || self.guardian_keys[..index].contains(guardian)
            {
                return Err(Piv1Error::InvalidGuardianSet);
            }
        }

        if self.kif_eligibility_bitmap & !GUARDIAN_BITMAP_MASK != 0
            || self.kif_eligibility_bitmap.count_ones()
                != u32::from(self.kif_active_guardian_count)
            || usize::from(self.kif_active_guardian_count) > GUARDIAN_COUNT
        {
            return Err(Piv1Error::InvalidGuardianBitmap);
        }

        let expected_gross_yield = derive_gross_yield_lamports(
            self.historical_value_lamports,
            self.prior_next_cycle_yield_lamports,
            self.old_protected_principal_lamports,
        )?;
        if expected_gross_yield != self.gross_yield_lamports
            || self.gross_yield_lamports == 0
        {
            return Err(Piv1Error::CumulativeReconciliationMismatch);
        }
        let split = piv1_math::split_gross_yield(self.gross_yield_lamports)?;
        if self.htfp_gross_obligation_lamports != split.htfp_reserve
            || self.permanent_compound_lamports != split.permanent_compound
            || self.team_owner_gross_obligation_lamports != split.team_owner_pool
            || self.kif_gross_obligation_lamports != split.kif
            || self.split_dust_lamports != split.dust
        {
            return Err(Piv1Error::InvalidSplit);
        }

        let outgoing = checked_add(
            checked_add(
                self.htfp_gross_obligation_lamports,
                self.team_owner_gross_obligation_lamports,
            )?,
            self.kif_gross_obligation_lamports,
        )?;
        if outgoing == 0 || outgoing != self.outgoing_gross_obligation_lamports {
            return Err(Piv1Error::ZeroTarget);
        }
        if self.pending_sol_used_lamports
            != core::cmp::min(
                self.pending_sol_snapshot_lamports,
                self.outgoing_gross_obligation_lamports,
            )
        {
            return Err(Piv1Error::CumulativeReconciliationMismatch);
        }

        let expected_proposed_hwm_delta = checked_add(
            checked_add(self.permanent_compound_lamports, self.split_dust_lamports)?,
            self.snapshot_conversion_dust_lamports,
        )?;
        let proposed_hwm = checked_add(
            self.old_protected_principal_lamports,
            self.proposed_hwm_delta_lamports,
        )?;
        if self.proposed_hwm_delta_lamports != expected_proposed_hwm_delta
            || proposed_hwm != self.proposed_hwm_after_settlement_lamports
            || self.stored_residual_hwm_floor_lamports
                < self.proposed_hwm_after_settlement_lamports
        {
            return Err(Piv1Error::HighWaterMarkDecrease);
        }

        Ok(())
    }

    fn validate_withdrawal_accounting(&self) -> Piv1Result<()> {
        if self.cumulative_jitosol_assigned_units
            > self.fixed_jitosol_withdrawal_target_units
        {
            return Err(Piv1Error::TargetExceeded);
        }
        if checked_add(
            self.cumulative_withdrawal_fee_units,
            self.cumulative_burned_units,
        )? != self.cumulative_jitosol_assigned_units
        {
            return Err(Piv1Error::CumulativeReconciliationMismatch);
        }
        if self.finalized_leg_count > self.successful_leg_count
            || self.next_leg_index != self.successful_leg_count
        {
            return Err(Piv1Error::CountMismatch);
        }
        if self.cumulative_finalized_delegated_native_lamports
            > self.cumulative_delegated_native_lamports
        {
            return Err(Piv1Error::CumulativeReconciliationMismatch);
        }
        if self.successful_leg_count == 0
            && (self.cumulative_jitosol_assigned_units != 0
                || self.cumulative_expected_native_lamports != 0
                || self.cumulative_delegated_native_lamports != 0)
        {
            return Err(Piv1Error::CumulativeReconciliationMismatch);
        }
        if self.finalized_leg_count == 0
            && (self.cumulative_finalized_delegated_native_lamports != 0
                || self.cumulative_finalized_native_lamports != 0
                || self.cumulative_recovered_stake_rent_lamports != 0
                || self.cumulative_recovered_metadata_rent_lamports != 0
                || self.cumulative_cooldown_rewards_lamports != 0
                || self.cumulative_cooldown_losses_lamports != 0)
        {
            return Err(Piv1Error::CumulativeReconciliationMismatch);
        }

        let has_cooldown_loss = self.cumulative_cooldown_losses_lamports > 0;
        let cooldown_loss_flagged = self.recovery_flags & RECOVERY_FLAG_COOLDOWN_LOSS != 0;
        if has_cooldown_loss != cooldown_loss_flagged {
            return Err(Piv1Error::CumulativeReconciliationMismatch);
        }

        if self.fixed_jitosol_withdrawal_target_units == 0 {
            if self.snapshot_leg_input_floor_units != 0
                || self.maximum_useful_legs != 0
                || self.stored_round_minimum_native_lamports != 0
                || self.cumulative_jitosol_assigned_units != 0
                || self.cumulative_withdrawal_fee_units != 0
                || self.cumulative_burned_units != 0
                || self.cumulative_expected_native_lamports != 0
                || self.cumulative_delegated_native_lamports != 0
                || self.cumulative_finalized_delegated_native_lamports != 0
                || self.cumulative_finalized_native_lamports != 0
                || self.cumulative_recovered_stake_rent_lamports != 0
                || self.cumulative_recovered_metadata_rent_lamports != 0
                || self.cumulative_cooldown_rewards_lamports != 0
                || self.cumulative_cooldown_losses_lamports != 0
                || self.next_leg_index != 0
                || self.successful_leg_count != 0
                || self.finalized_leg_count != 0
            {
                return Err(Piv1Error::CumulativeReconciliationMismatch);
            }
        } else {
            if self.snapshot_leg_input_floor_units == 0
                || self.stored_round_minimum_native_lamports == 0
            {
                return Err(Piv1Error::TechnicalFloorNotMet);
            }
            let expected_maximum = self.fixed_jitosol_withdrawal_target_units
                / self.snapshot_leg_input_floor_units;
            if expected_maximum == 0 || expected_maximum != self.maximum_useful_legs {
                return Err(Piv1Error::UsefulLegBoundExceeded);
            }
            if self.successful_leg_count > self.maximum_useful_legs {
                return Err(Piv1Error::UsefulLegBoundExceeded);
            }
        }

        let finalized_left = checked_add(
            checked_add(
                self.cumulative_finalized_delegated_native_lamports,
                self.cumulative_recovered_stake_rent_lamports,
            )?,
            self.cumulative_cooldown_rewards_lamports,
        )?;
        let finalized_right = checked_add(
            self.cumulative_finalized_native_lamports,
            self.cumulative_cooldown_losses_lamports,
        )?;
        if finalized_left != finalized_right {
            return Err(Piv1Error::CumulativeReconciliationMismatch);
        }

        let initial_liquid_funding = checked_add(
            self.pending_sol_used_lamports,
            self.prior_next_cycle_yield_used_lamports()?,
        )?;
        let expected_escrow = checked_sub(
            checked_add(
                initial_liquid_funding,
                checked_add(self.cumulative_finalized_native_lamports, self.swap_native_output())?,
            )?,
            self.cumulative_recovered_stake_rent_lamports,
        )?;
        if expected_escrow != self.recorded_escrow_available_lamports {
            return Err(Piv1Error::EscrowReconciliationMismatch);
        }

        if self.is_withdrawal_complete() {
            if self.cumulative_finalized_delegated_native_lamports
                != self.cumulative_delegated_native_lamports
                || self.cumulative_delegated_native_lamports
                    < self.stored_round_minimum_native_lamports
            {
                return Err(Piv1Error::CumulativeReconciliationMismatch);
            }
        }

        Ok(())
    }

    fn validate_settlement_accounting(&self) -> Piv1Result<()> {
        let eligible_native = self.eligible_native_for_settlement()?;
        let expected_actual_net = core::cmp::min(
            self.outgoing_gross_obligation_lamports,
            eligible_native,
        );
        if self.actual_net_available_lamports != expected_actual_net {
            return Err(Piv1Error::CumulativeReconciliationMismatch);
        }
        if self.actual_htfp_lamports > self.htfp_gross_obligation_lamports
            || self.actual_team_owner_lamports
                > self.team_owner_gross_obligation_lamports
            || self.actual_kif_allocation_lamports
                > self.kif_gross_obligation_lamports
        {
            return Err(Piv1Error::ObligationExceeded);
        }
        let expected_allocation = derive_net_beneficiary_allocation(
            expected_actual_net,
            self.htfp_gross_obligation_lamports,
            self.team_owner_gross_obligation_lamports,
            self.kif_gross_obligation_lamports,
        )?;
        if self.actual_htfp_lamports != expected_allocation.htfp_lamports
            || self.actual_team_owner_lamports
                != expected_allocation.team_owner_lamports
            || self.actual_kif_allocation_lamports != expected_allocation.kif_lamports
            || self.actual_net_allocation_dust_lamports
                != expected_allocation.dust_lamports
            || self.actual_allocated_outgoing_lamports
                != expected_allocation.allocated_lamports
        {
            return Err(Piv1Error::CumulativeReconciliationMismatch);
        }
        if checked_sub(
            self.recorded_escrow_available_lamports,
            self.actual_allocated_outgoing_lamports,
        )? != self.actual_escrow_remainder_lamports
        {
            return Err(Piv1Error::EscrowReconciliationMismatch);
        }

        let categorized_remainder = checked_add(
            checked_add(
                self.actual_net_allocation_dust_lamports,
                self.cumulative_cooldown_rewards_lamports,
            )?,
            self.actual_retained_conservative_dust_lamports,
        )?;
        if categorized_remainder != self.actual_escrow_remainder_lamports {
            return Err(Piv1Error::CumulativeReconciliationMismatch);
        }

        if self.actual_retained_conservative_dust_lamports
            != checked_sub(eligible_native, self.actual_net_available_lamports)?
        {
            return Err(Piv1Error::CumulativeReconciliationMismatch);
        }

        let expected_kif = piv1_math::allocate_kif(
            self.actual_kif_allocation_lamports,
            self.kif_carry_input_lamports,
            self.kif_active_guardian_count,
        )?;
        match expected_kif {
            piv1_math::KifAllocation::ActiveGuardians(active) => {
                if self.actual_kif_liability_lamports != active.credited_total
                    || self.actual_kif_carry_next_lamports != active.carry_next
                    || self.actual_zero_active_kif_compound_lamports != 0
                {
                    return Err(Piv1Error::CumulativeReconciliationMismatch);
                }
            }
            piv1_math::KifAllocation::ZeroActiveGuardians(zero) => {
                if self.actual_kif_liability_lamports != 0
                    || self.actual_kif_carry_next_lamports != zero.carry_next
                    || self.actual_zero_active_kif_compound_lamports
                        != zero.compound_from_kif
                {
                    return Err(Piv1Error::CumulativeReconciliationMismatch);
                }
            }
        }

        let actual_hwm_delta = checked_add(
            checked_add(
                checked_add(
                    self.proposed_hwm_delta_lamports,
                    self.actual_net_allocation_dust_lamports,
                )?,
                self.actual_retained_conservative_dust_lamports,
            )?,
            self.actual_zero_active_kif_compound_lamports,
        )?;
        if actual_hwm_delta != self.actual_hwm_delta_lamports
            || checked_add(
                self.old_protected_principal_lamports,
                self.actual_hwm_delta_lamports,
            )? != self.settled_protected_hwm_lamports
        {
            return Err(Piv1Error::HighWaterMarkDecrease);
        }

        Ok(())
    }

    fn require_actual_settlement_fields_zero(&self) -> Piv1Result<()> {
        if self.outstanding_active_round_liability_lamports
            != self.outgoing_gross_obligation_lamports
        {
            return Err(Piv1Error::OutstandingLiability);
        }
        if self.actual_net_available_lamports != 0
            || self.actual_htfp_lamports != 0
            || self.actual_team_owner_lamports != 0
            || self.actual_kif_allocation_lamports != 0
            || self.actual_net_allocation_dust_lamports != 0
            || self.actual_allocated_outgoing_lamports != 0
            || self.actual_escrow_remainder_lamports != 0
            || self.actual_retained_conservative_dust_lamports != 0
            || self.actual_kif_liability_lamports != 0
            || self.actual_kif_carry_next_lamports != 0
            || self.actual_zero_active_kif_compound_lamports != 0
            || self.actual_hwm_delta_lamports != 0
            || self.settled_protected_hwm_lamports != 0
        {
            return Err(Piv1Error::CumulativeReconciliationMismatch);
        }
        Ok(())
    }
}

/// Stored state of one deterministic `(sequence, leg_index)` metadata record.
#[derive(
    AnchorSerialize, AnchorDeserialize, InitSpace, Clone, Copy, Debug, Eq, PartialEq,
)]
pub enum WithdrawalLegStatus {
    /// Explicit host-side vacant model before future account initialization.
    Vacant,
    /// Protected withdrawal and immediate deactivation were recorded.
    Initiated,
    /// Full inactive-stake finalization and rent reconciliation were recorded.
    Finalized,
}

/// Bounded metadata for exactly one deterministic withdrawal stake leg.
#[derive(
    AnchorSerialize, AnchorDeserialize, InitSpace, Clone, Copy, Debug, Eq, PartialEq,
)]
pub struct WithdrawalLeg {
    pub version: u8,
    pub metadata_bump: u8,
    pub stake_bump: u8,
    pub is_initialized: bool,
    pub status: WithdrawalLegStatus,
    pub recovery_flags: u8,
    pub sequence: u64,
    pub leg_index: u64,
    pub validator_list_index: u32,
    pub validator_seed_suffix: u32,
    pub validator_vote: Pubkey,
    pub validator_stake_source: Pubkey,
    pub initiation_epoch: u64,
    pub pool_total_lamports: u64,
    pub pool_token_supply: u64,
    pub withdrawal_fee_numerator: u64,
    pub withdrawal_fee_denominator: u64,
    pub technical_floor_units: u64,
    pub jitosol_input_units: u64,
    pub withdrawal_fee_units: u64,
    pub burned_units: u64,
    pub expected_native_lamports: u64,
    pub observed_delegated_native_lamports: u64,
    pub minimum_native_lamports: u64,
    pub stake_rent_advanced_lamports: u64,
    pub metadata_rent_advanced_lamports: u64,
    pub finalized_epoch: Option<u64>,
    pub finalized_native_lamports: u64,
    pub recovered_stake_rent_lamports: u64,
    pub recovered_metadata_rent_lamports: u64,
    pub cooldown_reward_lamports: u64,
    pub cooldown_loss_lamports: u64,
}

impl WithdrawalLeg {
    /// Maximum serialized Borsh payload size, excluding a discriminator.
    pub const SERIALIZED_SIZE: usize = <Self as Space>::INIT_SPACE;
    /// Planned discriminator-inclusive Anchor account allocation.
    pub const SPACE: usize = PLANNED_ACCOUNT_DISCRIMINATOR_BYTES + Self::SERIALIZED_SIZE;

    /// Creates an explicit vacant model; it is not an initialized on-chain account.
    pub fn vacant(metadata_bump: u8, stake_bump: u8) -> Self {
        Self {
            version: STATE_LAYOUT_VERSION,
            metadata_bump,
            stake_bump,
            is_initialized: false,
            status: WithdrawalLegStatus::Vacant,
            recovery_flags: 0,
            sequence: 0,
            leg_index: 0,
            validator_list_index: 0,
            validator_seed_suffix: 0,
            validator_vote: Pubkey::default(),
            validator_stake_source: Pubkey::default(),
            initiation_epoch: 0,
            pool_total_lamports: 0,
            pool_token_supply: 0,
            withdrawal_fee_numerator: 0,
            withdrawal_fee_denominator: 0,
            technical_floor_units: 0,
            jitosol_input_units: 0,
            withdrawal_fee_units: 0,
            burned_units: 0,
            expected_native_lamports: 0,
            observed_delegated_native_lamports: 0,
            minimum_native_lamports: 0,
            stake_rent_advanced_lamports: 0,
            metadata_rent_advanced_lamports: 0,
            finalized_epoch: None,
            finalized_native_lamports: 0,
            recovered_stake_rent_lamports: 0,
            recovered_metadata_rent_lamports: 0,
            cooldown_reward_lamports: 0,
            cooldown_loss_lamports: 0,
        }
    }

    /// Validates one vacant, initiated, or finalized bounded metadata record.
    pub fn validate(&self) -> Piv1Result<()> {
        if self.version != STATE_LAYOUT_VERSION {
            return Err(Piv1Error::InvalidVersion);
        }

        if self.status == WithdrawalLegStatus::Vacant {
            let expected = Self::vacant(self.metadata_bump, self.stake_bump);
            if *self != expected {
                return Err(Piv1Error::InvalidInitialization);
            }
            return Ok(());
        }

        if !self.is_initialized
            || self.validator_vote == Pubkey::default()
            || self.validator_stake_source == Pubkey::default()
            || self.validator_vote == self.validator_stake_source
            || self.pool_total_lamports == 0
            || self.pool_token_supply == 0
        {
            return Err(Piv1Error::InvalidInitialization);
        }
        if self.jitosol_input_units == 0 {
            return Err(Piv1Error::ZeroInput);
        }
        if self.technical_floor_units == 0
            || self.jitosol_input_units < self.technical_floor_units
        {
            return Err(Piv1Error::TechnicalFloorNotMet);
        }
        if checked_add(self.withdrawal_fee_units, self.burned_units)?
            != self.jitosol_input_units
            || self.minimum_native_lamports > self.expected_native_lamports
            || self.observed_delegated_native_lamports < self.minimum_native_lamports
        {
            return Err(Piv1Error::CumulativeReconciliationMismatch);
        }

        match self.status {
            WithdrawalLegStatus::Vacant => Err(Piv1Error::InvalidInitialization),
            WithdrawalLegStatus::Initiated => {
                if self.finalized_epoch.is_some()
                    || self.recovery_flags != 0
                    || self.finalized_native_lamports != 0
                    || self.recovered_stake_rent_lamports != 0
                    || self.recovered_metadata_rent_lamports != 0
                    || self.cooldown_reward_lamports != 0
                    || self.cooldown_loss_lamports != 0
                {
                    return Err(Piv1Error::InvalidLifecycle);
                }
                Ok(())
            }
            WithdrawalLegStatus::Finalized => {
                if self
                    .finalized_epoch
                    .is_some_and(|epoch| epoch < self.initiation_epoch)
                {
                    return Err(Piv1Error::TimestampRegression);
                }
                if self.finalized_epoch.is_none()
                    || self.recovery_flags & !ALLOWED_RECOVERY_FLAGS != 0
                    || self.recovered_stake_rent_lamports
                        != self.stake_rent_advanced_lamports
                    || self.recovered_metadata_rent_lamports
                        != self.metadata_rent_advanced_lamports
                    || (self.cooldown_reward_lamports > 0
                        && self.cooldown_loss_lamports > 0)
                {
                    return Err(Piv1Error::CumulativeReconciliationMismatch);
                }

                let left = checked_add(
                    checked_add(
                        self.observed_delegated_native_lamports,
                        self.recovered_stake_rent_lamports,
                    )?,
                    self.cooldown_reward_lamports,
                )?;
                let right = checked_add(
                    self.finalized_native_lamports,
                    self.cooldown_loss_lamports,
                )?;
                let has_cooldown_loss = self.cooldown_loss_lamports > 0;
                let cooldown_loss_flagged =
                    self.recovery_flags & RECOVERY_FLAG_COOLDOWN_LOSS != 0;
                if left != right || has_cooldown_loss != cooldown_loss_flagged
                {
                    return Err(Piv1Error::CumulativeReconciliationMismatch);
                }
                Ok(())
            }
        }
    }
}

fn checked_add(left: u64, right: u64) -> Piv1Result<u64> {
    left.checked_add(right).ok_or(Piv1Error::ArithmeticOverflow)
}

fn checked_sub(left: u64, right: u64) -> Piv1Result<u64> {
    left.checked_sub(right)
        .ok_or(Piv1Error::CumulativeReconciliationMismatch)
}

#[cfg(test)]
mod tests {
    use super::*;

    const COMPLETED_SUMMARY_SERIALIZED_SIZE: usize = 11 * 8;
    const EXPECTED_ACTIVE_DISTRIBUTION_SERIALIZED_SIZE: usize =
        6 // six one-byte header fields, including the largest enum variant
        + 8 // active sequence
        + (1 + COMPLETED_SUMMARY_SERIALIZED_SIZE) // maximum Some summary
        + (11 * 8) // preparation snapshot
        + (11 * 8) // gross economics, prior yield, and pending-SOL snapshot
        + (5 * 8) + 2 // withdrawal bounds and slippage
        + (14 * 8) // cumulative leg accounting
        + (2 * 8) + (3 * 32) + 8 + (GUARDIAN_COUNT * 32) + 2 + (2 * 8)
        + (15 * 8); // proposed and actual settlement accounting
    const EXPECTED_WITHDRAWAL_LEG_SERIALIZED_SIZE: usize =
        6 // six one-byte header fields, including the largest enum variant
        + (2 * 8) // sequence and leg index
        + (2 * 4) // validator indices
        + (2 * 32) // validator identities
        + (14 * 8) // initiation and protocol accounting
        + (1 + 8) // maximum Some finalized epoch
        + (5 * 8); // finalized accounting

    fn key(tag: u8) -> Pubkey {
        Pubkey::new_from_array([tag; 32])
    }

    pub(super) fn completed_summary_maximum() -> CompletedDistributionSummary {
        CompletedDistributionSummary {
            sequence: u64::MAX,
            completed_at: i64::MAX,
            gross_yield_lamports: u64::MAX,
            actual_allocated_outgoing_lamports: u64::MAX,
            integrated_contribution_value_lamports: u64::MAX,
            final_protected_hwm_lamports: u64::MAX,
            fixed_jitosol_withdrawal_target_units: u64::MAX,
            successful_leg_count: u64::MAX,
            cumulative_cooldown_rewards_lamports: u64::MAX,
            actual_kif_liability_lamports: u64::MAX,
            actual_kif_carry_next_lamports: u64::MAX,
        }
    }

    pub(super) fn maximum_active_distribution() -> ActiveDistribution {
        let maximum_key = Pubkey::new_from_array([u8::MAX; 32]);
        ActiveDistribution {
            version: STATE_LAYOUT_VERSION,
            bump: u8::MAX,
            is_initialized: true,
            lifecycle: DistributionLifecycle::RecoveryRequired,
            recovery_flags: u8::MAX,
            settlement_recorded: true,
            active_sequence: u64::MAX,
            last_completed: Some(completed_summary_maximum()),
            prepared_at: i64::MAX,
            prepared_slot: u64::MAX,
            prepared_epoch: u64::MAX,
            old_protected_principal_lamports: u64::MAX,
            historical_jitosol_units: u64::MAX,
            historical_sol_lamports: u64::MAX,
            historical_value_lamports: u64::MAX,
            snapshot_pool_total_lamports: u64::MAX,
            snapshot_pool_token_supply: u64::MAX,
            snapshot_withdrawal_fee_numerator: u64::MAX,
            snapshot_withdrawal_fee_denominator: u64::MAX,
            gross_yield_lamports: u64::MAX,
            prior_next_cycle_yield_lamports: u64::MAX,
            htfp_gross_obligation_lamports: u64::MAX,
            permanent_compound_lamports: u64::MAX,
            team_owner_gross_obligation_lamports: u64::MAX,
            kif_gross_obligation_lamports: u64::MAX,
            split_dust_lamports: u64::MAX,
            outgoing_gross_obligation_lamports: u64::MAX,
            pending_sol_snapshot_lamports: u64::MAX,
            pending_sol_used_lamports: u64::MAX,
            snapshot_conversion_dust_lamports: u64::MAX,
            fixed_jitosol_withdrawal_target_units: u64::MAX,
            snapshot_leg_input_floor_units: u64::MAX,
            maximum_useful_legs: u64::MAX,
            stored_round_minimum_native_lamports: u64::MAX,
            stored_residual_hwm_floor_lamports: u64::MAX,
            stored_slippage_bps: u16::MAX,
            cumulative_jitosol_assigned_units: u64::MAX,
            cumulative_withdrawal_fee_units: u64::MAX,
            cumulative_burned_units: u64::MAX,
            cumulative_expected_native_lamports: u64::MAX,
            cumulative_delegated_native_lamports: u64::MAX,
            cumulative_finalized_delegated_native_lamports: u64::MAX,
            cumulative_finalized_native_lamports: u64::MAX,
            cumulative_recovered_stake_rent_lamports: u64::MAX,
            cumulative_recovered_metadata_rent_lamports: u64::MAX,
            cumulative_cooldown_rewards_lamports: u64::MAX,
            cumulative_cooldown_losses_lamports: u64::MAX,
            next_leg_index: u64::MAX,
            successful_leg_count: u64::MAX,
            finalized_leg_count: u64::MAX,
            recorded_escrow_available_lamports: u64::MAX,
            outstanding_active_round_liability_lamports: u64::MAX,
            htfp_recipient: maximum_key,
            team_owner_recipient: maximum_key,
            guardian_registry: maximum_key,
            guardian_registry_revision: u64::MAX,
            guardian_keys: [maximum_key; GUARDIAN_COUNT],
            kif_eligibility_bitmap: u8::MAX,
            kif_active_guardian_count: u8::MAX,
            kif_period_id: u64::MAX,
            kif_carry_input_lamports: u64::MAX,
            proposed_hwm_delta_lamports: u64::MAX,
            proposed_hwm_after_settlement_lamports: u64::MAX,
            actual_net_available_lamports: u64::MAX,
            actual_htfp_lamports: u64::MAX,
            actual_team_owner_lamports: u64::MAX,
            actual_kif_allocation_lamports: u64::MAX,
            actual_net_allocation_dust_lamports: u64::MAX,
            actual_allocated_outgoing_lamports: u64::MAX,
            actual_escrow_remainder_lamports: u64::MAX,
            actual_retained_conservative_dust_lamports: u64::MAX,
            actual_kif_liability_lamports: u64::MAX,
            actual_kif_carry_next_lamports: u64::MAX,
            actual_zero_active_kif_compound_lamports: u64::MAX,
            actual_hwm_delta_lamports: u64::MAX,
            settled_protected_hwm_lamports: u64::MAX,
            swap: None,
            recovery: None,
            last_completed_recovery: None,
        }
    }

    fn maximum_withdrawal_leg() -> WithdrawalLeg {
        let maximum_key = Pubkey::new_from_array([u8::MAX; 32]);
        WithdrawalLeg {
            version: u8::MAX,
            metadata_bump: u8::MAX,
            stake_bump: u8::MAX,
            is_initialized: true,
            status: WithdrawalLegStatus::Finalized,
            recovery_flags: u8::MAX,
            sequence: u64::MAX,
            leg_index: u64::MAX,
            validator_list_index: u32::MAX,
            validator_seed_suffix: u32::MAX,
            validator_vote: maximum_key,
            validator_stake_source: maximum_key,
            initiation_epoch: u64::MAX,
            pool_total_lamports: u64::MAX,
            pool_token_supply: u64::MAX,
            withdrawal_fee_numerator: u64::MAX,
            withdrawal_fee_denominator: u64::MAX,
            technical_floor_units: u64::MAX,
            jitosol_input_units: u64::MAX,
            withdrawal_fee_units: u64::MAX,
            burned_units: u64::MAX,
            expected_native_lamports: u64::MAX,
            observed_delegated_native_lamports: u64::MAX,
            minimum_native_lamports: u64::MAX,
            stake_rent_advanced_lamports: u64::MAX,
            metadata_rent_advanced_lamports: u64::MAX,
            finalized_epoch: Some(u64::MAX),
            finalized_native_lamports: u64::MAX,
            recovered_stake_rent_lamports: u64::MAX,
            recovered_metadata_rent_lamports: u64::MAX,
            cooldown_reward_lamports: u64::MAX,
            cooldown_loss_lamports: u64::MAX,
        }
    }

    pub(super) fn valid_active_round() -> ActiveDistribution {
        let mut round = ActiveDistribution::new_idle(9);
        round.lifecycle = DistributionLifecycle::WithdrawalActive;
        round.active_sequence = 7;
        round.prepared_at = 1_000;
        round.prepared_slot = 2_000;
        round.prepared_epoch = 3;
        round.old_protected_principal_lamports = 1_000;
        round.historical_jitosol_units = 500;
        round.historical_sol_lamports = 100;
        round.historical_value_lamports = 1_100;
        round.snapshot_pool_total_lamports = 2_000;
        round.snapshot_pool_token_supply = 1_000;
        round.snapshot_withdrawal_fee_numerator = 1;
        round.snapshot_withdrawal_fee_denominator = 1_000;
        round.gross_yield_lamports = 100;
        round.prior_next_cycle_yield_lamports = 0;
        round.htfp_gross_obligation_lamports = 59;
        round.permanent_compound_lamports = 19;
        round.team_owner_gross_obligation_lamports = 19;
        round.kif_gross_obligation_lamports = 2;
        round.split_dust_lamports = 1;
        round.outgoing_gross_obligation_lamports = 80;
        round.pending_sol_snapshot_lamports = 10;
        round.pending_sol_used_lamports = 10;
        round.fixed_jitosol_withdrawal_target_units = 100;
        round.snapshot_leg_input_floor_units = 25;
        round.maximum_useful_legs = 4;
        round.stored_round_minimum_native_lamports = 80;
        round.stored_residual_hwm_floor_lamports = 1_020;
        round.stored_slippage_bps = 1;
        round.recorded_escrow_available_lamports = 10;
        round.outstanding_active_round_liability_lamports = 80;
        round.htfp_recipient = key(1);
        round.team_owner_recipient = key(2);
        round.guardian_registry = key(3);
        round.guardian_registry_revision = 4;
        round.guardian_keys = core::array::from_fn(|index| key((index + 10) as u8));
        round.kif_eligibility_bitmap = 0b00_1011;
        round.kif_active_guardian_count = 3;
        round.kif_period_id = 8;
        round.kif_carry_input_lamports = 9;
        round.proposed_hwm_delta_lamports = 20;
        round.proposed_hwm_after_settlement_lamports = 1_020;
        round
    }

    fn valid_initiated_leg() -> WithdrawalLeg {
        WithdrawalLeg {
            version: STATE_LAYOUT_VERSION,
            metadata_bump: 1,
            stake_bump: 2,
            is_initialized: true,
            status: WithdrawalLegStatus::Initiated,
            recovery_flags: 0,
            sequence: 7,
            leg_index: 0,
            validator_list_index: 3,
            validator_seed_suffix: 4,
            validator_vote: key(20),
            validator_stake_source: key(21),
            initiation_epoch: 5,
            pool_total_lamports: 2_000,
            pool_token_supply: 1_000,
            withdrawal_fee_numerator: 1,
            withdrawal_fee_denominator: 1_000,
            technical_floor_units: 25,
            jitosol_input_units: 100,
            withdrawal_fee_units: 2,
            burned_units: 98,
            expected_native_lamports: 1_000,
            observed_delegated_native_lamports: 990,
            minimum_native_lamports: 980,
            stake_rent_advanced_lamports: 20,
            metadata_rent_advanced_lamports: 10,
            finalized_epoch: None,
            finalized_native_lamports: 0,
            recovered_stake_rent_lamports: 0,
            recovered_metadata_rent_lamports: 0,
            cooldown_reward_lamports: 0,
            cooldown_loss_lamports: 0,
        }
    }

    pub(super) fn valid_completed_withdrawal_round() -> ActiveDistribution {
        let mut round = valid_active_round();
        round.lifecycle = DistributionLifecycle::EscrowFunded;
        round.cumulative_jitosol_assigned_units = 100;
        round.cumulative_withdrawal_fee_units = 2;
        round.cumulative_burned_units = 98;
        round.cumulative_expected_native_lamports = 100;
        round.cumulative_delegated_native_lamports = 100;
        round.cumulative_finalized_delegated_native_lamports = 100;
        round.cumulative_finalized_native_lamports = 100;
        round.next_leg_index = 2;
        round.successful_leg_count = 2;
        round.finalized_leg_count = 2;
        round.recorded_escrow_available_lamports = 110;
        round
    }

    #[test]
    fn gross_yield_basis_applies_carry_before_the_high_water_mark() {
        assert_eq!(derive_gross_yield_lamports(900, 50, 1_000), Ok(0));
        assert_eq!(derive_gross_yield_lamports(900, 100, 1_000), Ok(0));
        assert_eq!(derive_gross_yield_lamports(900, 125, 1_000), Ok(25));
        assert_eq!(derive_gross_yield_lamports(1_100, 25, 1_000), Ok(125));
        assert_eq!(derive_gross_yield_lamports(1_100, 0, 1_000), Ok(100));
        assert_eq!(
            derive_gross_yield_lamports(u64::MAX, 1, 0),
            Err(Piv1Error::ArithmeticOverflow)
        );
    }

    #[test]
    fn net_beneficiary_allocation_uses_exact_relative_weights_caps_and_dust() {
        let full = derive_net_beneficiary_allocation(8_050, 5_900, 1_950, 200)
            .expect("full allocation");
        assert_eq!(full.htfp_lamports, 5_900);
        assert_eq!(full.team_owner_lamports, 1_950);
        assert_eq!(full.kif_lamports, 200);
        assert_eq!(full.dust_lamports, 0);
        assert_eq!(full.allocated_lamports, 8_050);

        let partial = derive_net_beneficiary_allocation(1_800, 5_900, 1_950, 200)
            .expect("partial allocation");
        assert_eq!(partial.htfp_lamports, 1_319);
        assert_eq!(partial.team_owner_lamports, 436);
        assert_eq!(partial.kif_lamports, 44);
        assert_eq!(partial.dust_lamports, 1);
        assert_eq!(partial.allocated_lamports, 1_799);

        let zero = derive_net_beneficiary_allocation(0, 5_900, 1_950, 200)
            .expect("zero allocation");
        assert_eq!(zero.allocated_lamports, 0);
        assert_eq!(zero.dust_lamports, 0);

        let one = derive_net_beneficiary_allocation(1, 5_900, 1_950, 200)
            .expect("one-lamport allocation");
        assert_eq!(one.allocated_lamports, 0);
        assert_eq!(one.dust_lamports, 1);

        let capped = derive_net_beneficiary_allocation(10, 1, 1, 1)
            .expect("small obligation caps");
        assert_eq!(capped.htfp_lamports, 1);
        assert_eq!(capped.team_owner_lamports, 1);
        assert_eq!(capped.kif_lamports, 0);
        assert_eq!(capped.dust_lamports, 8);

        let maximum = derive_net_beneficiary_allocation(
            u64::MAX,
            u64::MAX,
            u64::MAX,
            u64::MAX,
        )
        .expect("u64 boundary allocation");
        assert_eq!(
            maximum
                .allocated_lamports
                .checked_add(maximum.dust_lamports),
            Some(u64::MAX)
        );
    }

    #[test]
    fn layouts_match_manual_fixed_maximum_sizes_and_planned_space() {
        assert_eq!(
            <CompletedDistributionSummary as Space>::INIT_SPACE,
            COMPLETED_SUMMARY_SERIALIZED_SIZE
        );
        assert_eq!(
            ActiveDistribution::SERIALIZED_SIZE,
            EXPECTED_ACTIVE_DISTRIBUTION_SERIALIZED_SIZE
        );
        assert_eq!(
            WithdrawalLeg::SERIALIZED_SIZE,
            EXPECTED_WITHDRAWAL_LEG_SERIALIZED_SIZE
        );
        assert_eq!(
            ActiveDistribution::SPACE,
            PLANNED_ACCOUNT_DISCRIMINATOR_BYTES
                + EXPECTED_ACTIVE_DISTRIBUTION_SERIALIZED_SIZE
        );
        assert_eq!(
            WithdrawalLeg::SPACE,
            PLANNED_ACCOUNT_DISCRIMINATOR_BYTES + EXPECTED_WITHDRAWAL_LEG_SERIALIZED_SIZE
        );
        assert_eq!(ActiveDistribution::SERIALIZED_SIZE, 883);
        assert_eq!(ActiveDistribution::SPACE, 891);
        assert_eq!(WithdrawalLeg::SERIALIZED_SIZE, 255);
        assert_eq!(WithdrawalLeg::SPACE, 263);
    }

    #[test]
    fn maximum_options_enums_and_values_serialize_exactly_and_round_trip() {
        let round = maximum_active_distribution();
        let encoded_round = round.try_to_vec().expect("serialize maximum round");
        assert_eq!(encoded_round.len(), ActiveDistribution::SERIALIZED_SIZE);
        assert_eq!(
            ActiveDistribution::try_from_slice(&encoded_round)
                .expect("deserialize maximum round"),
            round
        );

        let leg = maximum_withdrawal_leg();
        let encoded_leg = leg.try_to_vec().expect("serialize maximum leg");
        assert_eq!(encoded_leg.len(), WithdrawalLeg::SERIALIZED_SIZE);
        assert_eq!(
            WithdrawalLeg::try_from_slice(&encoded_leg).expect("deserialize maximum leg"),
            leg
        );

        assert_eq!(
            DistributionLifecycle::RecoveryRequired
                .try_to_vec()
                .expect("serialize maximum lifecycle variant"),
            vec![4]
        );
        assert_eq!(
            WithdrawalLegStatus::Finalized
                .try_to_vec()
                .expect("serialize maximum leg-status variant"),
            vec![2]
        );
    }

    #[test]
    fn fixed_construction_has_only_exact_bounded_option_deltas() {
        let idle_without_summary = ActiveDistribution::new_idle(1)
            .try_to_vec()
            .expect("serialize idle round");
        let idle_with_summary = ActiveDistribution::idle_after_completion(
            1,
            completed_summary_maximum(),
        )
        .try_to_vec()
        .expect("serialize completed summary");
        assert_eq!(
            idle_without_summary.len(),
            ActiveDistribution::SERIALIZED_SIZE - COMPLETED_SUMMARY_SERIALIZED_SIZE
        );
        assert_eq!(idle_with_summary.len(), ActiveDistribution::SERIALIZED_SIZE);

        let vacant_without_epoch = WithdrawalLeg::vacant(1, 2)
            .try_to_vec()
            .expect("serialize vacant leg");
        assert_eq!(
            vacant_without_epoch.len(),
            WithdrawalLeg::SERIALIZED_SIZE - 8
        );

        // Every other field in both layouts is scalar or a fixed-size array;
        // the two bounded Options above are the only value-dependent deltas.
        assert_eq!(
            maximum_withdrawal_leg()
                .try_to_vec()
                .expect("serialize Some epoch")
                .len(),
            WithdrawalLeg::SERIALIZED_SIZE
        );
    }

    #[test]
    fn derived_predicates_allow_only_consistent_coexisting_phases() {
        let prepared = valid_active_round();
        assert_eq!(prepared.validate(), Ok(()));
        assert!(prepared.is_prepared_withdrawal());
        assert!(!prepared.is_assigning_withdrawal_legs());
        assert!(!prepared.is_withdrawal_target_assigned());
        assert!(!prepared.is_awaiting_leg_inactivity());
        assert!(!prepared.is_partially_finalized());

        let mut partially_finalized = valid_active_round();
        partially_finalized.cumulative_jitosol_assigned_units = 50;
        partially_finalized.cumulative_withdrawal_fee_units = 2;
        partially_finalized.cumulative_burned_units = 48;
        partially_finalized.cumulative_expected_native_lamports = 60;
        partially_finalized.cumulative_delegated_native_lamports = 60;
        partially_finalized.cumulative_finalized_delegated_native_lamports = 30;
        partially_finalized.cumulative_finalized_native_lamports = 30;
        partially_finalized.next_leg_index = 2;
        partially_finalized.successful_leg_count = 2;
        partially_finalized.finalized_leg_count = 1;
        partially_finalized.recorded_escrow_available_lamports = 40;
        assert_eq!(partially_finalized.validate(), Ok(()));
        assert!(partially_finalized.is_assigning_withdrawal_legs());
        assert!(partially_finalized.is_awaiting_leg_inactivity());
        assert!(partially_finalized.is_partially_finalized());
        assert!(!partially_finalized.is_withdrawal_target_assigned());

        let mut assigned_before_finalization = valid_active_round();
        assigned_before_finalization.cumulative_jitosol_assigned_units = 100;
        assigned_before_finalization.cumulative_withdrawal_fee_units = 2;
        assigned_before_finalization.cumulative_burned_units = 98;
        assigned_before_finalization.cumulative_expected_native_lamports = 100;
        assigned_before_finalization.cumulative_delegated_native_lamports = 100;
        assigned_before_finalization.next_leg_index = 2;
        assigned_before_finalization.successful_leg_count = 2;
        assert_eq!(assigned_before_finalization.validate(), Ok(()));
        assert!(assigned_before_finalization.is_withdrawal_target_assigned());
        assert!(assigned_before_finalization.is_awaiting_leg_inactivity());
        assert!(!assigned_before_finalization.is_withdrawal_complete());

        let completed = valid_completed_withdrawal_round();
        assert_eq!(completed.validate(), Ok(()));
        assert!(completed.is_withdrawal_target_assigned());
        assert!(completed.are_all_successful_legs_finalized());
        assert!(completed.is_withdrawal_complete());
        assert!(!completed.is_awaiting_leg_inactivity());
        assert!(!completed.is_partially_finalized());
    }

    #[test]
    fn active_round_rejects_count_cumulative_bitmap_and_status_inconsistency() {
        let mut bad_count = valid_active_round();
        bad_count.next_leg_index = 1;
        assert_eq!(bad_count.validate(), Err(Piv1Error::CountMismatch));

        let mut bad_cumulative = valid_active_round();
        bad_cumulative.cumulative_jitosol_assigned_units = 1;
        assert_eq!(
            bad_cumulative.validate(),
            Err(Piv1Error::CumulativeReconciliationMismatch)
        );

        let mut bad_bitmap_count = valid_active_round();
        bad_bitmap_count.kif_active_guardian_count = 2;
        assert_eq!(
            bad_bitmap_count.validate(),
            Err(Piv1Error::InvalidGuardianBitmap)
        );

        let mut bad_bitmap_bits = valid_active_round();
        bad_bitmap_bits.kif_eligibility_bitmap = 0b1000_0000;
        bad_bitmap_bits.kif_active_guardian_count = 1;
        assert_eq!(
            bad_bitmap_bits.validate(),
            Err(Piv1Error::InvalidGuardianBitmap)
        );

        let mut completed_but_still_active = valid_completed_withdrawal_round();
        completed_but_still_active.lifecycle = DistributionLifecycle::WithdrawalActive;
        assert_eq!(
            completed_but_still_active.validate(),
            Err(Piv1Error::InvalidLifecycle)
        );

        let mut settled_without_record = valid_active_round();
        settled_without_record.lifecycle = DistributionLifecycle::Settled;
        settled_without_record.outstanding_active_round_liability_lamports = 0;
        assert_eq!(
            settled_without_record.validate(),
            Err(Piv1Error::InvalidLifecycle)
        );
    }

    #[test]
    fn withdrawal_leg_rejects_cumulative_and_status_inconsistency() {
        let initiated = valid_initiated_leg();
        assert_eq!(initiated.validate(), Ok(()));

        let mut bad_cumulative = initiated;
        bad_cumulative.burned_units = 97;
        assert_eq!(
            bad_cumulative.validate(),
            Err(Piv1Error::CumulativeReconciliationMismatch)
        );

        let mut initiated_with_finalization = initiated;
        initiated_with_finalization.finalized_epoch = Some(6);
        assert_eq!(
            initiated_with_finalization.validate(),
            Err(Piv1Error::InvalidLifecycle)
        );

        let mut finalized_without_epoch = initiated;
        finalized_without_epoch.status = WithdrawalLegStatus::Finalized;
        assert_eq!(
            finalized_without_epoch.validate(),
            Err(Piv1Error::CumulativeReconciliationMismatch)
        );

        let mut valid_finalized = initiated;
        valid_finalized.status = WithdrawalLegStatus::Finalized;
        valid_finalized.finalized_epoch = Some(6);
        valid_finalized.finalized_native_lamports = 1_015;
        valid_finalized.recovered_stake_rent_lamports = 20;
        valid_finalized.recovered_metadata_rent_lamports = 10;
        valid_finalized.cooldown_reward_lamports = 5;
        assert_eq!(valid_finalized.validate(), Ok(()));

        let mut mutually_exclusive_reward_and_loss = valid_finalized;
        mutually_exclusive_reward_and_loss.cooldown_loss_lamports = 1;
        assert_eq!(
            mutually_exclusive_reward_and_loss.validate(),
            Err(Piv1Error::CumulativeReconciliationMismatch)
        );

        let mut nonvacant_vacant = WithdrawalLeg::vacant(1, 2);
        nonvacant_vacant.is_initialized = true;
        assert_eq!(
            nonvacant_vacant.validate(),
            Err(Piv1Error::InvalidInitialization)
        );
    }

    #[test]
    fn initialized_round_and_leg_reject_wrong_versions_and_default_bindings() {
        let round = valid_active_round();
        assert_eq!(round.validate(), Ok(()));

        let mut wrong_round_version = round;
        wrong_round_version.version = ActiveDistribution::RECOVERY_VERSION.saturating_add(1);
        assert_eq!(wrong_round_version.validate(), Err(Piv1Error::InvalidVersion));

        let mut uninitialized_round = round;
        uninitialized_round.is_initialized = false;
        assert_eq!(
            uninitialized_round.validate(),
            Err(Piv1Error::InvalidInitialization)
        );

        let mut default_recipient = round;
        default_recipient.htfp_recipient = Pubkey::default();
        assert_eq!(default_recipient.validate(), Err(Piv1Error::InvalidAddress));

        let leg = valid_initiated_leg();
        assert_eq!(leg.validate(), Ok(()));

        let mut wrong_leg_version = leg;
        wrong_leg_version.version = STATE_LAYOUT_VERSION.saturating_add(1);
        assert_eq!(wrong_leg_version.validate(), Err(Piv1Error::InvalidVersion));

        let mut default_validator = leg;
        default_validator.validator_vote = Pubkey::default();
        assert_eq!(
            default_validator.validate(),
            Err(Piv1Error::InvalidInitialization)
        );
    }
}

#[cfg(test)]
mod recovery_tests {
    use super::*;
    use super::tests::valid_completed_withdrawal_round;
    use crate::state::recovery::ResumeRecovery;
    fn incident(loss:u64)->ActiveDistribution{
        let mut r=valid_completed_withdrawal_round();r.version=3;
        r.cumulative_cooldown_losses_lamports=loss;r.cumulative_finalized_native_lamports-=loss;
        r.recorded_escrow_available_lamports-=loss;
        r.record_recovery_incident(if loss>0{RECOVERY_FLAG_COOLDOWN_LOSS}else{RECOVERY_FLAG_RESIDUAL_HWM}).unwrap();
        r.validate().unwrap();r
    }
    fn request(r:&ActiveDistribution)->ResumeRecovery{ResumeRecovery{vault_index:0,sequence:r.active_sequence,
        generation:r.recovery.unwrap().generation,frozen_snapshot_hash:r.frozen_recovery_hash().unwrap(),
        cumulative_loss:r.cumulative_cooldown_losses_lamports,finalized_count:r.finalized_leg_count,
        successful_count:r.successful_leg_count,assigned_units:r.cumulative_jitosol_assigned_units,
        escrow_lamports:r.recorded_escrow_available_lamports,destination:0,validator_list_index:0}}
    #[test]
    fn d111_bounded_domain_loss_once_codec_and_net_conservation(){
        // Generated finite domain, independent u128 oracle: all integer losses
        // through the entire conditional obligation, including >5% incidents.
        for loss in 0..=80{
            let r=incident(loss);let next=r.resume_recovery(request(&r),Pubkey::new_from_array([70;32]),19,[7;32],1001).unwrap();
            assert_eq!(next.eligible_native_for_settlement().unwrap(),110-loss);
            let net=(110-loss).min(80);let a=derive_net_beneficiary_allocation(net,59,19,2).unwrap();
            let h=((u128::from(net)*5900/8050)as u64).min(59);
            let t=((u128::from(net)*1950/8050)as u64).min(19);
            let k=((u128::from(net)*200/8050)as u64).min(2);
            assert_eq!((a.htfp_lamports,a.team_owner_lamports,a.kif_lamports),(h,t,k));
            assert_eq!(a.allocated_lamports+a.dust_lamports+(110-loss-net),110-loss);
            let bytes=next.try_to_vec().unwrap();let decoded=ActiveDistribution::try_from_slice(&bytes).unwrap();
            assert_eq!(decoded,next);assert!(bytes.len()+8<=ActiveDistribution::RECOVERY_SPACE);
            assert_eq!(next.cumulative_cooldown_losses_lamports,loss);
            assert_eq!(next.proposed_hwm_after_settlement_lamports,r.proposed_hwm_after_settlement_lamports);
            assert_eq!(next.frozen_recovery_hash(),r.frozen_recovery_hash());
        }
        assert_eq!(ActiveDistribution::SPACE,891);assert_eq!(ActiveDistribution::SWAP_SPACE,924);
        assert_eq!(ActiveDistribution::RECOVERY_SPACE,1199);
        for version in [1,2]{let mut r=ActiveDistribution::new_idle(9);r.version=version;
            let bytes=r.try_to_vec().unwrap();assert_eq!(ActiveDistribution::try_from_slice(&bytes).unwrap(),r);
            assert_eq!(bytes.len(),if version==1{795}else{796});}
    }
    #[test]
    fn d111_new_incident_even_same_loss_invalidates_every_old_intent(){
        let r=incident(7);let old=request(&r);let mut r=r.resume_recovery(old,Pubkey::new_from_array([70;32]),19,[7;32],1001).unwrap();
        r.record_recovery_incident(RECOVERY_FLAG_RESIDUAL_HWM).unwrap();r.validate().unwrap();
        assert_eq!(r.recovery.unwrap().generation,2);assert_eq!(r.cumulative_cooldown_losses_lamports,7);
        assert!(r.resume_recovery(old,Pubkey::new_from_array([70;32]),20,[8;32],1002).is_err());
        for mutate in 0..9{let mut q=request(&r);match mutate{
            0=>q.sequence+=1,1=>q.generation-=1,2=>q.frozen_snapshot_hash[0]^=1,3=>q.cumulative_loss+=1,
            4=>q.finalized_count+=1,5=>q.successful_count+=1,6=>q.assigned_units+=1,7=>q.escrow_lamports+=1,_=>q.destination=2};
            assert!(r.resume_recovery(q,Pubkey::new_from_array([70;32]),20,[8;32],1002).is_err());}
        let mut forged=r;forged.htfp_recipient=Pubkey::new_from_array([99;32]);assert!(forged.validate().is_err());
        let mut exhausted=incident(81);assert!(exhausted.resume_recovery(request(&exhausted),Pubkey::new_from_array([70;32]),20,[8;32],1002).is_err());
        exhausted.recovery.as_mut().unwrap().generation=u64::MAX;
        assert_eq!(exhausted.record_recovery_incident(RECOVERY_FLAG_RESIDUAL_HWM),Err(Piv1Error::ArithmeticOverflow));
    }
    #[test]
    fn d111_migration_retains_legacy_body_and_completed_history_never_active_authority(){
        let mut old=incident(9);old.recovery=None;old.version=2;old.validate().unwrap();
        let before=old.try_to_vec().unwrap();let migrated=old.migrate_v2_recovery().unwrap();let bytes=migrated.try_to_vec().unwrap();
        assert_eq!(&before[1..],&bytes[1..before.len()]);assert_eq!(migrated.lifecycle,DistributionLifecycle::RecoveryRequired);
        assert_eq!(migrated.recovery.unwrap().acknowledged_generation,0);assert!(migrated.migrate_v2_recovery().is_err());
        let resumed=migrated.resume_recovery(request(&migrated),Pubkey::new_from_array([70;32]),19,[7;32],1001).unwrap();
        let mut idle=ActiveDistribution::new_idle(9);idle.version=3;
        let mut summary=super::tests::completed_summary_maximum();summary.sequence=resumed.active_sequence;
        idle.last_completed=Some(summary);idle.last_completed_recovery=resumed.completed_recovery();idle.validate().unwrap();
        assert!(idle.recovery.is_none());let bytes=idle.try_to_vec().unwrap();assert_eq!(ActiveDistribution::try_from_slice(&bytes).unwrap(),idle);
        let mut bad=idle;bad.last_completed.as_mut().unwrap().sequence+=1;assert!(bad.validate().is_err());
    }
}

#[cfg(test)]
mod recovery_refusal_tests {
    use super::*;
    use crate::state::recovery::ResumeRecovery;
    #[test]
    fn d111_zero_cash_all_floor_zero_and_idle_migration(){
        for cash in [0,1]{let mut r=super::tests::valid_active_round();r.version=3;
            r.pending_sol_snapshot_lamports=cash;r.pending_sol_used_lamports=cash;r.recorded_escrow_available_lamports=cash;
            r.record_recovery_incident(RECOVERY_FLAG_RESIDUAL_HWM).unwrap();r.validate().unwrap();
            let receipt=r.recovery.unwrap();let q=ResumeRecovery{vault_index:0,sequence:r.active_sequence,generation:receipt.generation,
                frozen_snapshot_hash:receipt.frozen_snapshot_hash,cumulative_loss:0,finalized_count:0,successful_count:0,
                assigned_units:0,escrow_lamports:cash,destination:1,validator_list_index:0};
            assert_eq!(r.resume_recovery(q,Pubkey::new_from_array([7;32]),1,[8;32],1001),Err(Piv1Error::ZeroTarget));}
        let mut old=ActiveDistribution::new_idle(9);old.version=2;old.last_completed=Some(super::tests::completed_summary_maximum());
        let before=crate::state_persistence::StateEnvelope::distribution(&old).unwrap();let migrated=old.migrate_v2_recovery().unwrap();
        let after=crate::state_persistence::StateEnvelope::distribution(&migrated).unwrap();
        assert_eq!(after.as_bytes().len(),1199);assert_eq!(&before.as_bytes()[9..],&after.as_bytes()[9..924]);
        assert!(migrated.recovery.is_none());assert!(migrated.last_completed_recovery.is_none());
        let mut bad=after.as_bytes().to_vec();bad[900]=1;
        assert!(ActiveDistribution::deserialize(&mut &bad[8..]).is_err());
    }
}

#[cfg(test)]
mod recovery_maximum_codec_test {
    use super::*;
    #[test]
    fn d111_maximum_v2_v3_payloads_and_padding_are_exact(){
        use crate::state::recovery::{RecoveryReceipt,CompletedRecoverySummary};
        let mut r=super::tests::maximum_active_distribution();r.version=2;
        r.swap=Some(crate::integrations::jupiter::SwapReceipt{spent_units:u64::MAX,official_book_debit_lamports:u64::MAX,
            actual_native_output_lamports:u64::MAX,minimum_native_output_lamports:u64::MAX});
        let old=r.try_to_vec().unwrap();assert_eq!(old.len()+8,924);
        r.version=3;r.recovery=Some(RecoveryReceipt{generation:u64::MAX,acknowledged_generation:u64::MAX,
            acknowledged_cumulative_loss:u64::MAX,frozen_snapshot_hash:[7;32],approving_multisig:Pubkey::new_from_array([8;32]),
            approving_transaction_index:u64::MAX,approving_instruction_hash:[9;32],approved_at:i64::MAX});
        r.last_completed_recovery=Some(CompletedRecoverySummary{sequence:u64::MAX,cumulative_cooldown_loss:u64::MAX,generation:u64::MAX,
            recovery_flags:3,approving_multisig:Pubkey::new_from_array([8;32]),approving_transaction_index:u64::MAX,
            approving_instruction_hash:[9;32],frozen_snapshot_hash:[7;32],approved_at:i64::MAX});
        let bytes=r.try_to_vec().unwrap();assert_eq!(bytes.len()+8,1199);assert_eq!(&old[1..],&bytes[1..old.len()]);
        assert_eq!(ActiveDistribution::try_from_slice(&bytes).unwrap(),r);
    }
}
