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
    PendingSolStaked,
    DistributionPrepared,
    DelayedWithdrawalInitiated,
    WithdrawalLegInitiated,
    WithdrawalLegFinalized,
    WithdrawalReady,
    DistributionFinalized,
    PendingIntegrated,
    KifRewardsCredited,
    RecipientsUpdated,
    GuardianSetUpdated,
    StrategyConfigUpdated,
);

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
