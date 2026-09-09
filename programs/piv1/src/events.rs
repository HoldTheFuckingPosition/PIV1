//! Factual KIF claim event and retained markers for other logical boundaries.
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
    SolContribution,
    JitoSolContribution,
    UntrackedBalanceReconciled,
    PendingSolStaked,
    DistributionPrepared,
    DelayedWithdrawalInitiated,
    WithdrawalLegInitiated,
    WithdrawalLegFinalized,
    WithdrawalReady,
    DistributionFinalized,
    PendingIntegrated,
    GuardianHeartbeat,
    KifRewardsCredited,
    PauseChanged,
    RecipientsUpdated,
    GuardianSetUpdated,
    StrategyConfigUpdated,
);

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
