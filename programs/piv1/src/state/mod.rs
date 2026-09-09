//! Bounded Anchor-compatible state layouts and pure transition validation.
//!
//! The five PIV1-owned schemas use Anchor/Borsh serialization without
//! `#[account]`: no Program ID exists yet, so the exported `SPACE` constants
//! are discriminator-inclusive plans rather than owner-bound account claims.

pub mod bootstrap;
pub mod config;
pub mod contributions;
pub mod custody;
pub mod distribution;
pub mod guardian;
pub mod kif_claim;
pub mod principal_deposit;
pub mod reconciliation;
pub mod timing;
pub mod transitions;

pub use bootstrap::{bootstrap_initial_contributions, InitialContributionBootstrap};
pub use config::{PivConfig, PivConfigBumps};
pub use contributions::{
    reconcile_pending_contributions, record_explicit_jitosol_contribution,
    record_explicit_sol_contribution, ExplicitContributionRecord,
    JitoSolCustodyObservation, PendingCustodyObservation,
    PendingReconciliationResult, SolCustodyObservation,
};
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
pub use kif_claim::{
    prepare_kif_claim, KifClaimCustodyObservation, KifClaimRequest, KifClaimTransfer,
    PreparedKifClaim,
};
pub use principal_deposit::{
    record_protected_principal_deposit, PrincipalSolDepositObservation,
    PrincipalSolDepositRecord,
};
pub use timing::{
    derive_kif_period, validate_insufficient_retry, validate_preparation_interval, KifPeriod,
};
pub use transitions::{
    finalize_withdrawal_leg, initiate_withdrawal_leg,
    integrate_pending_and_complete, open_distribution, record_no_yield_evaluation,
    record_valid_insufficient_attempt, settle_distribution, DistributionFunding,
    LegFinalizationInput, LegFinalizationOutcome, LegInitiationInput,
    OpenDistributionInput, PendingIntegrationInput, SettlementInput, SettlementOutcome,
    ValidInsufficientAttemptInput,
};
