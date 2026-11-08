//! D111 bounded same-round recovery. No receipt is a reusable governance capability.
use anchor_lang::{prelude::{Pubkey, borsh}, AnchorSerialize, AnchorDeserialize, InitSpace};
use solana_sha256_hasher::hash;
use crate::{errors::{Piv1Error, Piv1Result}, constants::{RECOVERY_FLAG_COOLDOWN_LOSS, RECOVERY_FLAG_RESIDUAL_HWM}};
use super::{ActiveDistribution, DistributionLifecycle};

#[derive(AnchorSerialize, AnchorDeserialize, InitSpace, Clone, Copy, Debug, Eq, PartialEq)]
pub struct RecoveryReceipt {
    pub generation: u64,
    pub acknowledged_generation: u64,
    pub acknowledged_cumulative_loss: u64,
    pub frozen_snapshot_hash: [u8; 32],
    pub approving_multisig: Pubkey,
    pub approving_transaction_index: u64,
    pub approving_instruction_hash: [u8; 32],
    pub approved_at: i64,
}
#[derive(AnchorSerialize, AnchorDeserialize, InitSpace, Clone, Copy, Debug, Eq, PartialEq)]
pub struct CompletedRecoverySummary {
    pub sequence: u64,
    pub cumulative_cooldown_loss: u64,
    pub generation: u64,
    pub recovery_flags: u8,
    pub approving_multisig: Pubkey,
    pub approving_transaction_index: u64,
    pub approving_instruction_hash: [u8; 32],
    pub frozen_snapshot_hash: [u8; 32],
    pub approved_at: i64,
}
fn mismatch() -> Piv1Error { Piv1Error::CumulativeReconciliationMismatch }
fn add(a:u64,b:u64)->Piv1Result<u64>{a.checked_add(b).ok_or(Piv1Error::ArithmeticOverflow)}
fn sub(a:u64,b:u64)->Piv1Result<u64>{a.checked_sub(b).ok_or_else(mismatch)}
impl ActiveDistribution {
    /// Hash the immutable preparation fields explicitly. Mutable collection,
    /// acknowledgement, settlement and prior summary fields are not authority.
    pub fn frozen_recovery_hash(&self) -> Piv1Result<[u8;32]> {
        let mut storage = [0_u8; 768];
        let mut bytes = std::io::Cursor::new(storage.as_mut_slice());
        std::io::Write::write_all(&mut bytes,b"PIV1-D111-FROZEN-V1").map_err(|_| mismatch())?;
        macro_rules! fields { ($($f:ident),* $(,)?) => { $(self.$f.serialize(&mut bytes).map_err(|_| mismatch())?;)* }; }
        fields!(bump, active_sequence, prepared_at, prepared_slot, prepared_epoch,
            old_protected_principal_lamports, historical_jitosol_units, historical_sol_lamports,
            historical_value_lamports, snapshot_pool_total_lamports, snapshot_pool_token_supply,
            snapshot_withdrawal_fee_numerator, snapshot_withdrawal_fee_denominator,
            gross_yield_lamports, prior_next_cycle_yield_lamports, htfp_gross_obligation_lamports,
            permanent_compound_lamports, team_owner_gross_obligation_lamports,
            kif_gross_obligation_lamports, split_dust_lamports, outgoing_gross_obligation_lamports,
            pending_sol_snapshot_lamports, pending_sol_used_lamports, snapshot_conversion_dust_lamports,
            fixed_jitosol_withdrawal_target_units, snapshot_leg_input_floor_units,
            maximum_useful_legs, stored_round_minimum_native_lamports, stored_residual_hwm_floor_lamports,
            stored_slippage_bps, htfp_recipient, team_owner_recipient, guardian_registry,
            guardian_registry_revision, guardian_keys, kif_eligibility_bitmap, kif_active_guardian_count,
            kif_period_id, kif_carry_input_lamports, proposed_hwm_delta_lamports,
            proposed_hwm_after_settlement_lamports, swap);
        let len=usize::try_from(bytes.position()).map_err(|_| mismatch())?;
        Ok(hash(&storage[..len]).to_bytes())
    }
    pub(crate) fn recovery_flags_resolved(&self) -> bool {
        self.recovery_flags == 0 || (self.version == Self::RECOVERY_VERSION
            && self.recovery.is_some_and(|r| r.generation != 0
                && r.acknowledged_generation == r.generation
                && r.acknowledged_cumulative_loss == self.cumulative_cooldown_losses_lamports))
    }
    pub(crate) fn validate_recovery_extension(&self) -> Piv1Result<()> {
        if self.version != Self::RECOVERY_VERSION {
            if self.recovery.is_some() || self.last_completed_recovery.is_some() { return Err(mismatch()); }
            return Ok(());
        }
        if let Some(h) = self.last_completed_recovery {
            if self.last_completed.map(|s| s.sequence) != Some(h.sequence)
                || h.generation == 0 || h.recovery_flags == 0
                || h.recovery_flags & !(RECOVERY_FLAG_COOLDOWN_LOSS | RECOVERY_FLAG_RESIDUAL_HWM) != 0
                || (h.cumulative_cooldown_loss > 0) != (h.recovery_flags & RECOVERY_FLAG_COOLDOWN_LOSS != 0)
                || h.approving_multisig == Pubkey::default() || h.approving_instruction_hash == [0;32]
                || h.frozen_snapshot_hash == [0;32] { return Err(mismatch()); }
        }
        match self.recovery {
            None if self.recovery_flags == 0 => Ok(()),
            Some(r) if self.recovery_flags != 0 && r.generation > 0
                && r.acknowledged_generation <= r.generation
                && r.acknowledged_cumulative_loss <= self.cumulative_cooldown_losses_lamports
                && r.frozen_snapshot_hash == self.frozen_recovery_hash()? => {
                let ack = r.acknowledged_generation == r.generation;
                if ack != (self.lifecycle != DistributionLifecycle::RecoveryRequired)
                    || (ack && (r.acknowledged_cumulative_loss != self.cumulative_cooldown_losses_lamports
                        || r.approving_multisig == Pubkey::default() || r.approving_instruction_hash == [0;32]
                        || r.approved_at < self.prepared_at)) { return Err(mismatch()); }
                Ok(())
            },
            _ => Err(mismatch()),
        }
    }
    /// Called only for a newly committed authenticated incident, not for a retry.
    pub(crate) fn record_recovery_incident(&mut self, flags:u8) -> Piv1Result<()> {
        if flags == 0 || flags & !(RECOVERY_FLAG_COOLDOWN_LOSS | RECOVERY_FLAG_RESIDUAL_HWM) != 0 { return Err(mismatch()); }
        self.recovery_flags |= flags;
        self.lifecycle = DistributionLifecycle::RecoveryRequired;
        if self.version == Self::RECOVERY_VERSION {
            let mut r = self.recovery.unwrap_or(RecoveryReceipt { generation:0, acknowledged_generation:0,
                acknowledged_cumulative_loss:0, frozen_snapshot_hash:self.frozen_recovery_hash()?,
                approving_multisig:Pubkey::default(), approving_transaction_index:0,
                approving_instruction_hash:[0;32], approved_at:0 });
            r.generation = add(r.generation,1)?;
            self.recovery = Some(r);
        }
        Ok(())
    }
    /// Cumulative loss is deducted exactly once, never a charge per resume.
    pub(crate) fn eligible_native_for_settlement(&self)->Piv1Result<u64>{
        let gross=add(add(self.pending_sol_used_lamports,self.prior_next_cycle_yield_used_lamports()?)?,
            add(self.cumulative_finalized_delegated_native_lamports,self.swap_native_output())?)?;
        if self.version == Self::RECOVERY_VERSION {
            sub(gross,self.cumulative_cooldown_losses_lamports)
        } else { Ok(gross) }
    }
    pub(crate) fn completed_recovery(&self)->Option<CompletedRecoverySummary>{
        self.recovery.map(|r| CompletedRecoverySummary { sequence:self.active_sequence,
            cumulative_cooldown_loss:self.cumulative_cooldown_losses_lamports, generation:r.generation,
            recovery_flags:self.recovery_flags, approving_multisig:r.approving_multisig,
            approving_transaction_index:r.approving_transaction_index,
            approving_instruction_hash:r.approving_instruction_hash, frozen_snapshot_hash:r.frozen_snapshot_hash,
            approved_at:r.approved_at })
    }
    pub fn migrate_v2_recovery(&self)->Piv1Result<Self>{
        self.validate()?;
        if self.version != Self::SWAP_VERSION || !matches!(self.lifecycle,
            DistributionLifecycle::Idle | DistributionLifecycle::RecoveryRequired) { return Err(Piv1Error::InvalidLifecycle); }
        let mut next=*self; next.version=Self::RECOVERY_VERSION;
        if self.lifecycle == DistributionLifecycle::RecoveryRequired { next.record_recovery_incident(self.recovery_flags)?; }
        next.validate()?; Ok(next)
    }
}

/// Exact frozen intent. Current market/custody proofs are authenticated by the
/// runtime handler; numeric facts here are never a production authorization.
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Copy, Debug, Eq, PartialEq)]
pub struct ResumeRecovery {
    pub vault_index:u8,
    pub sequence:u64,
    pub generation:u64,
    pub frozen_snapshot_hash:[u8;32],
    pub cumulative_loss:u64,
    pub finalized_count:u64,
    pub successful_count:u64,
    pub assigned_units:u64,
    pub escrow_lamports:u64,
    /// 0=EscrowFunded, 1=WithdrawalActive; no other destination exists.
    pub destination:u8,
    pub validator_list_index:u32,
}
impl ActiveDistribution {
    pub(crate) fn resume_recovery(&self, request:ResumeRecovery, multisig:Pubkey,
        transaction_index:u64, action_hash:[u8;32], now:i64)->Piv1Result<Self>{
        self.validate()?;
        let r=self.recovery.ok_or(Piv1Error::RecoveryRequired)?;
        if self.version!=Self::RECOVERY_VERSION || self.lifecycle!=DistributionLifecycle::RecoveryRequired
            || request.sequence!=self.active_sequence || request.generation!=r.generation
            || request.frozen_snapshot_hash!=r.frozen_snapshot_hash
            || request.cumulative_loss!=self.cumulative_cooldown_losses_lamports
            || request.finalized_count!=self.finalized_leg_count
            || request.successful_count!=self.successful_leg_count
            || request.assigned_units!=self.cumulative_jitosol_assigned_units
            || request.escrow_lamports!=self.recorded_escrow_available_lamports
            || self.finalized_leg_count!=self.successful_leg_count
            || self.cumulative_cooldown_losses_lamports>self.outgoing_gross_obligation_lamports
            || multisig==Pubkey::default() || action_hash==[0;32] || now<self.prepared_at {return Err(mismatch());}
        let next_phase=match request.destination {
            0 if self.fixed_jitosol_withdrawal_target_units==0 || self.is_withdrawal_complete()=>DistributionLifecycle::EscrowFunded,
            1 if self.remaining_withdrawal_target_units()?>0=>DistributionLifecycle::WithdrawalActive,
            _=>return Err(Piv1Error::InvalidLifecycle),
        };
        let eligible=self.eligible_native_for_settlement()?;
        let allocation=super::distribution::derive_net_beneficiary_allocation(
            eligible.min(self.outgoing_gross_obligation_lamports),self.htfp_gross_obligation_lamports,
            self.team_owner_gross_obligation_lamports,self.kif_gross_obligation_lamports)?;
        if allocation.allocated_lamports==0{return Err(Piv1Error::ZeroTarget);}
        let mut next=*self; let mut receipt=r;
        receipt.acknowledged_generation=r.generation;receipt.acknowledged_cumulative_loss=request.cumulative_loss;
        receipt.approving_multisig=multisig;receipt.approving_transaction_index=transaction_index;
        receipt.approving_instruction_hash=action_hash;receipt.approved_at=now;
        next.recovery=Some(receipt);next.lifecycle=next_phase;next.validate()?;Ok(next)
    }
}
