//! Factual claim/guardian/contribution events and retained lifecycle markers.
//! Other unit markers are not emit-ready. State is the accounting authority.

use anchor_lang::{prelude::{borsh, Pubkey}, AnchorDeserialize, AnchorSerialize, Discriminator};
#[cfg(feature = "idl-build")]
use anchor_lang::IdlBuild;

macro_rules! event_marker {
    ($($name:ident),+ $(,)?) => {
        $(
            #[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
            pub struct $name;
        )+
    };
}

event_marker!(
    PivInitialized,
    DelayedWithdrawalInitiated,
    WithdrawalReady,
    DistributionFinalized,
    PendingIntegrated,
    KifRewardsCredited,
    StrategyConfigUpdated,
);

/// Protected conversion of existing principal, emitted after exact CPI checks
/// and Config commit. Require transaction success; accounts remain authoritative.
#[anchor_lang::event]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PendingSolStaked {
    pub config: Pubkey,
    pub deposited_sol_lamports: u64,
    pub minted_jitosol_units: u64,
    pub historical_value_before_lamports: u64,
    pub historical_value_after_lamports: u64,
}

/// Initial holdings and their conservative current contribution value, after
/// both custody movements and Config commit. No yield or depositor right is
/// created. Require transaction success; account state remains authoritative.
#[anchor_lang::event]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct InitialContributionsBootstrapped {
    pub config: Pubkey,
    pub integrated_sol_lamports: u64,
    pub integrated_jitosol_units: u64,
    pub contribution_value_lamports: u64,
}

/// Proven economic surplus moved/recognized as pending after all CPI checks
/// and the Config commit. Token-native/operational funding is not classified.
/// Consumers must require transaction success; account state is authoritative.
#[anchor_lang::event]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct UntrackedBalanceReconciled {
    pub config: Pubkey,
    pub newly_accounted_sol_lamports: u64,
    pub newly_accounted_jitosol_units: u64,
    pub pending_sol_lamports_after: u64,
    pub pending_jitosol_units_after: u64,
}

/// Exact newly received SOL, after verified custody and Config commit. No claim
/// or depositor right is created. Consumers must require transaction success.
#[anchor_lang::event]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SolContribution {
    pub config: Pubkey,
    pub donor: Pubkey,
    pub amount_lamports: u64,
    pub pending_lamports_after: u64,
}

/// Exact newly received JitoSOL units; no conversion, yield or depositor right.
/// Consumers must require transaction success; account state remains authoritative.
#[anchor_lang::event]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct JitoSolContribution {
    pub config: Pubkey,
    pub source: Pubkey,
    pub owner: Pubkey,
    pub amount_units: u64,
    pub pending_units_after: u64,
}

/// Emitted once after successful claim execution and all postchecks. A later
/// transaction failure can still leave logs: consumers must require transaction
/// success. This event neither creates liability nor replaces authoritative state.
#[anchor_lang::event]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct KifClaimed {
    pub guardian_reward: Pubkey,
    pub guardian: Pubkey,
    pub amount_lamports: u64,
}

/// Current-period activity only: no award, payout or historical snapshot change.
/// Consumers must require transaction success; later failure can leave logs.
#[anchor_lang::event]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct GuardianHeartbeat {
    pub guardian_registry: Pubkey,
    pub guardian_reward: Pubkey,
    pub guardian: Pubkey,
    pub registry_revision: u64,
    pub guardian_index: u8,
    pub period_id: u64,
}

/// Factual result after explicit setting, including idempotent same-value calls.
/// Logs from a subsequently failed transaction must not be treated as committed.
#[anchor_lang::event]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PauseChanged {
    pub config: Pubkey,
    pub multisig: Pubkey,
    pub transaction_index: u64,
    pub previously_paused: bool,
    pub paused: bool,
}

/// Factual Config destination update, including idempotent same-value calls.
/// Prepared rounds retain their frozen keys; require whole-transaction success.
#[anchor_lang::event]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RecipientsUpdated {
    pub config: Pubkey,
    pub multisig: Pubkey,
    pub transaction_index: u64,
    pub previous_htfp_recipient: Pubkey,
    pub previous_team_owner_recipient: Pubkey,
    pub htfp_recipient: Pubkey,
    pub team_owner_recipient: Pubkey,
}

/// Fixed round, emitted only after custody and both state commits.
/// The proposed HWM is not credited until settlement; require transaction success.
#[anchor_lang::event]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DistributionPrepared {
    pub config: Pubkey,
    pub sequence: u64,
    pub gross_yield_lamports: u64,
    pub outgoing_lamports: u64,
    pub pending_sol_used_lamports: u64,
    pub prior_yield_used_lamports: u64,
    pub kif_eligibility_bitmap: u8,
}

/// Actual isolated swap receipt, emitted after both state commits. Require
/// successful transaction finality; logs alone never establish economic effects.
#[anchor_lang::event]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DistributionSwapPrepared {
    pub config: Pubkey,
    pub sequence: u64,
    pub spent_jitosol_units: u64,
    pub official_book_debit_lamports: u64,
    pub actual_native_output_lamports: u64,
    pub minimum_native_output_lamports: u64,
    pub normalized_prefund_lamports: u64,
    pub technical_rent_recovered_lamports: u64,
}

#[anchor_lang::event]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DistributionRoundMigrated {
    pub round: Pubkey,
    pub prior_version: u8,
    pub new_version: u8,
    pub rent_advanced_lamports: u64,
}

/// Authenticated technically insufficient withdrawal evaluation, emitted after
/// the sole Config retry-clock commit. No round, movement or reclassification.
#[anchor_lang::event]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DistributionPreparationInsufficient {
    pub config: Pubkey,
    pub attempted_at: i64,
    pub target_jitosol_units: u64,
    pub technical_minimum_jitosol_units: u64,
}

/// Exact protected withdrawal and immediate deactivation, emitted only after
/// Config/round/leg commit. Consumers must also require transaction success.
#[anchor_lang::event]
#[derive(Debug,Clone,Copy,PartialEq,Eq)]
pub struct WithdrawalLegInitiated {
    pub config:Pubkey, pub sequence:u64, pub leg_index:u64,
    pub jitosol_input_units:u64, pub delegated_native_lamports:u64,
    pub stake_rent_advanced_lamports:u64, pub metadata_rent_advanced_lamports:u64,
    pub normalized_prefund_lamports:u64,
}

/// Full inactive Stake receipt, original rent recovery and metadata closure.
/// Rewards remain in escrow for the next cycle; require transaction success.
#[anchor_lang::event]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct WithdrawalLegFinalized {
    pub config: Pubkey,
    pub sequence: u64,
    pub leg_index: u64,
    pub finalized_native_lamports: u64,
    pub recovered_stake_rent_lamports: u64,
    pub recovered_metadata_rent_lamports: u64,
    pub cooldown_reward_lamports: u64,
    pub cooldown_loss_lamports: u64,
    pub normalized_metadata_excess_lamports: u64,
    pub recovery_flags: u8,
}

/// Exact atomic payments, KIF credit and protected HWM commit. Logs are factual
/// only when the containing transaction succeeds; pending integration follows.
#[anchor_lang::event]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DistributionSettled {
    pub config: Pubkey,
    pub sequence: u64,
    pub htfp_lamports: u64,
    pub team_owner_lamports: u64,
    pub kif_allocation_lamports: u64,
    pub kif_liability_lamports: u64,
    pub kif_carry_lamports: u64,
    pub zero_active_compound_lamports: u64,
    pub protected_hwm_lamports: u64,
}

/// A successful no-payment recovery classification. Only the round changes;
/// this is not a failed transfer or a settled distribution.
#[anchor_lang::event]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DistributionSettlementRecovery {
    pub config: Pubkey,
    pub sequence: u64,
    pub observed_protected_value_lamports: u64,
    pub recovery_flags: u8,
}

/// Full pending contribution value integrated once after exact custody transfers.
#[anchor_lang::event]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PendingContributionsIntegrated {
    pub config: Pubkey,
    pub sequence: u64,
    pub integrated_sol_lamports: u64,
    pub integrated_jitosol_units: u64,
    pub contribution_value_lamports: u64,
    pub protected_hwm_lamports: u64,
}

/// One current Squads seat mirrored into a new PIV reward revision. Historical
/// rewards and prepared eligibility remain immutable; require transaction success.
#[anchor_lang::event]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct GuardianSetUpdated {
    pub config: Pubkey,
    pub guardian_registry: Pubkey,
    pub multisig: Pubkey,
    pub transaction_index: u64,
    pub previous_revision: u64,
    pub registry_revision: u64,
    pub guardian_index: u8,
    pub previous_guardian: Pubkey,
    pub new_guardian: Pubkey,
    pub rent_payer: Pubkey,
    pub funded_rent_lamports: u64,
}

/// Exact governed same-round recovery acknowledgement; no transfer occurs.
#[anchor_lang::event]
pub struct RecoveryResumed {
    pub round: Pubkey, pub sequence: u64, pub generation: u64,
    pub cumulative_loss: u64, pub destination: u8, pub multisig: Pubkey,
    pub transaction_index: u64, pub action_hash: [u8; 32],
    pub frozen_snapshot_hash: [u8;32], pub observed_at: i64,
    pub escrow_lamports: u64, pub validated_protected_value: u64,
    pub required_protected_value: u64,
}

/// Durable aggregate incident/collection identity after a canonical leg closes.
#[anchor_lang::event]
pub struct RecoveryObserved {
    pub round: Pubkey, pub sequence: u64, pub generation: u64,
    pub cumulative_loss: u64, pub recovery_flags: u8,
    pub frozen_snapshot_hash: [u8; 32], pub collection: bool,
    pub old_lifecycle: u8, pub observed_at: i64,
    pub assigned_units: u64, pub successful_legs: u64, pub finalized_legs: u64,
    pub escrow_lamports: u64, pub cooldown_rewards: u64,
    pub recovered_stake_rent: u64, pub recovered_metadata_rent: u64,
    pub proposed_hwm: u64, pub pending_used: u64, pub observed_protected_value: u64,
}
pub(crate) fn emit_recovery_observed(round:Pubkey,r:&crate::state::ActiveDistribution,collection:bool,old_lifecycle:u8,observed_at:i64,value:u64){
    if let Some(receipt)=r.recovery {
        anchor_lang::emit!(RecoveryObserved{round,sequence:r.active_sequence,generation:receipt.generation,
            cumulative_loss:r.cumulative_cooldown_losses_lamports,recovery_flags:r.recovery_flags,
            frozen_snapshot_hash:receipt.frozen_snapshot_hash,collection,old_lifecycle,observed_at,
            assigned_units:r.cumulative_jitosol_assigned_units,successful_legs:r.successful_leg_count,finalized_legs:r.finalized_leg_count,
            escrow_lamports:r.recorded_escrow_available_lamports,cooldown_rewards:r.cumulative_cooldown_rewards_lamports,
            recovered_stake_rent:r.cumulative_recovered_stake_rent_lamports,recovered_metadata_rent:r.cumulative_recovered_metadata_rent_lamports,
            proposed_hwm:r.proposed_hwm_after_settlement_lamports,pending_used:r.pending_sol_used_lamports,observed_protected_value:value});
    }
}
