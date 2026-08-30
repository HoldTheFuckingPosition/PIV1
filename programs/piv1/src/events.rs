//! Compile-only event names for the confirmed logical boundaries.
//!
//! These unit structs are not annotated with `#[event]` and are not emit-ready.
//! Event fields and stable IDL discriminators remain provisional.

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
    KifClaimed,
    PauseChanged,
    RecipientsUpdated,
    GuardianSetUpdated,
    StrategyConfigUpdated,
);
