//! Bounded Anchor-compatible state layouts and pure transition validation.
//!
//! The five PIV1-owned schemas use Anchor/Borsh serialization without
//! `#[account]`: no Program ID exists yet, so the exported `SPACE` constants
//! are discriminator-inclusive plans rather than owner-bound account claims.

pub mod config;
pub mod custody;
pub mod distribution;
pub mod guardian;
pub mod timing;
pub mod transitions;

pub use config::{PivConfig, PivConfigBumps};
pub use custody::{
    DistributionEscrowRole, KifSolVaultRole, OperationalSolVaultRole,
    PendingJitoVaultRole, PendingSolVaultRole, PivAuthorityRole,
    PrincipalJitoVaultRole, PrincipalSolQueueRole, WithdrawalStakeRole,
};
pub use distribution::{
    ActiveDistribution, CompletedDistributionSummary, DistributionLifecycle,
    WithdrawalLeg, WithdrawalLegStatus,
};
pub use guardian::{GuardianRegistry, GuardianReward};
pub use timing::{
    derive_kif_period, validate_insufficient_retry, validate_preparation_interval, KifPeriod,
};
pub use transitions::{
    finalize_withdrawal_leg, initiate_withdrawal_leg,
    integrate_pending_and_complete, open_distribution, record_no_yield_evaluation,
    record_valid_insufficient_attempt, settle_distribution, DistributionFunding,
    LegFinalizationInput, LegFinalizationOutcome, LegInitiationInput,
    OpenDistributionInput, PendingIntegrationInput, SettlementInput, SettlementOutcome,
};
