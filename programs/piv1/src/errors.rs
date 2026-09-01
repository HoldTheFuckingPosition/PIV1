//! Deterministic state-model errors for Task 1.3.

use core::fmt;

/// Failures returned by bounded layout validation and pure state transitions.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Piv1Error {
    /// A serialized layout uses an unsupported schema version.
    InvalidVersion,
    /// A required state object is not explicitly initialized or is malformed.
    InvalidInitialization,
    /// The requested transition is not legal from the stored lifecycle phase.
    InvalidLifecycle,
    /// The confirmed pause policy blocks the requested operation.
    PausedOperation,
    /// A supplied signed timestamp or configured duration is invalid.
    InvalidTimestamp,
    /// A timestamp moved backwards relative to already recorded state.
    TimestampRegression,
    /// The confirmed ten-day preparation interval has not elapsed.
    PreparationIntervalNotElapsed,
    /// The 24-hour retry cooldown for valid insufficient attempts is active.
    InsufficientAttemptCooldownActive,
    /// A round sequence does not match the active or next monotonic sequence.
    SequenceMismatch,
    /// A withdrawal-leg index is not the exact next or recorded index.
    LegIndexMismatch,
    /// A required distribution or withdrawal target is zero.
    ZeroTarget,
    /// A withdrawal leg has zero input.
    ZeroInput,
    /// Assigned withdrawal input would exceed the fixed round target.
    TargetExceeded,
    /// A supplied leg input is not the required maximum-safe fill.
    NonMaximumSafeLegFill,
    /// A leg is below the stored or current validated technical floor.
    TechnicalFloorNotMet,
    /// The mathematical maximum useful-leg count would be exceeded.
    UsefulLegBoundExceeded,
    /// A round or leg action has already been recorded.
    Replay,
    /// A recorded withdrawal leg has already been finalized.
    AlreadyFinalized,
    /// Exact withdrawal-target assignment has not completed.
    TargetNotAssigned,
    /// Successful/finalized leg counters do not reconcile.
    CountMismatch,
    /// Stored or supplied cumulative values do not reconcile exactly.
    CumulativeReconciliationMismatch,
    /// The supplied fixed escrow value does not match cumulative accounting.
    EscrowReconciliationMismatch,
    /// An actual beneficiary allocation exceeds its immutable gross obligation.
    ObligationExceeded,
    /// A round still has an unpaid active-round liability.
    OutstandingLiability,
    /// Atomic beneficiary settlement has already been recorded.
    SettlementReplay,
    /// A proposed normal update would lower protected principal.
    HighWaterMarkDecrease,
    /// A guardian bitmap has bits outside the fixed six slots or a wrong count.
    InvalidGuardianBitmap,
    /// A guardian count is outside the confirmed range.
    InvalidGuardianCount,
    /// Guardian keys, slots, revisions, or reward bindings are invalid.
    InvalidGuardianSet,
    /// A required initialized address is the default key or violates separation.
    InvalidAddress,
    /// Configured slippage exceeds the immutable one-basis-point cap.
    InvalidSlippage,
    /// Stored fixed split bindings differ from the confirmed economics.
    InvalidSplit,
    /// A stored timing binding differs from the confirmed fixed policy.
    InvalidTimingConfiguration,
    /// Checked integer arithmetic or narrowing failed.
    ArithmeticOverflow,
    /// Normal progression is blocked pending governed recovery.
    RecoveryRequired,
}

impl fmt::Display for Piv1Error {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(match self {
            Self::InvalidVersion => "invalid state-layout version",
            Self::InvalidInitialization => "invalid or missing state initialization",
            Self::InvalidLifecycle => "invalid lifecycle transition",
            Self::PausedOperation => "operation blocked while paused",
            Self::InvalidTimestamp => "invalid timestamp",
            Self::TimestampRegression => "timestamp regression",
            Self::PreparationIntervalNotElapsed => "minimum preparation interval not elapsed",
            Self::InsufficientAttemptCooldownActive => {
                "valid-insufficient-attempt cooldown is active"
            }
            Self::SequenceMismatch => "distribution sequence mismatch",
            Self::LegIndexMismatch => "withdrawal-leg index mismatch",
            Self::ZeroTarget => "zero distribution or withdrawal target",
            Self::ZeroInput => "zero withdrawal-leg input",
            Self::TargetExceeded => "fixed withdrawal target exceeded",
            Self::NonMaximumSafeLegFill => "leg is not the maximum-safe fill",
            Self::TechnicalFloorNotMet => "technical withdrawal floor not met",
            Self::UsefulLegBoundExceeded => "maximum useful-leg bound exceeded",
            Self::Replay => "state transition replay",
            Self::AlreadyFinalized => "withdrawal leg already finalized",
            Self::TargetNotAssigned => "withdrawal target not assigned",
            Self::CountMismatch => "withdrawal-leg count mismatch",
            Self::CumulativeReconciliationMismatch => "cumulative accounting mismatch",
            Self::EscrowReconciliationMismatch => "distribution escrow mismatch",
            Self::ObligationExceeded => "fixed beneficiary obligation exceeded",
            Self::OutstandingLiability => "outstanding active-round liability",
            Self::SettlementReplay => "distribution settlement replay",
            Self::HighWaterMarkDecrease => "protected high-water mark decrease",
            Self::InvalidGuardianBitmap => "invalid guardian bitmap or count",
            Self::InvalidGuardianCount => "invalid guardian count",
            Self::InvalidGuardianSet => "invalid guardian set or reward binding",
            Self::InvalidAddress => "invalid initialized address",
            Self::InvalidSlippage => "invalid configured slippage",
            Self::InvalidSplit => "invalid fixed split binding",
            Self::InvalidTimingConfiguration => "invalid timing configuration",
            Self::ArithmeticOverflow => "checked arithmetic failure",
            Self::RecoveryRequired => "governed recovery required",
        })
    }
}

impl From<piv1_math::MathError> for Piv1Error {
    fn from(error: piv1_math::MathError) -> Self {
        match error {
            piv1_math::MathError::InvalidActiveGuardianCount { .. } => {
                Self::InvalidGuardianCount
            }
            _ => Self::ArithmeticOverflow,
        }
    }
}

/// Result alias used by the pure state model.
pub type Piv1Result<T> = Result<T, Piv1Error>;
