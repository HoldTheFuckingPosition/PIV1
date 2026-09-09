//! Deterministic state-model errors for Task 1.3.

use core::fmt;

/// Failures returned by bounded layout validation and pure state transitions.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Piv1Error {
    /// A supplied account does not have the required program owner.
    InvalidAccountOwner,
    /// A required account lacks writable privilege.
    AccountNotWritable,
    /// The entitled guardian did not authorize the isolated claim.
    MissingGuardianSignature,
    /// An already-earned KIF claim must have positive input.
    ZeroKifClaim,
    /// The expected cumulative-claimed counter no longer matches the ledger.
    StaleKifClaim,
    /// A KIF claim exceeds the selected earned or global liability.
    KifClaimExceeded,
    /// KIF custody cannot cover rent, every recorded liability and collective carry.
    KifClaimBackingDeficit,
    /// State changed after claim effects were prepared.
    KifClaimStateChanged,
    /// Exact isolated KIF source/destination/floor observations do not match.
    KifClaimObservationMismatch,
    /// The Clock input does not use the canonical sysvar account key.
    InvalidClockAccount,
    /// A supplied custody or state account is executable.
    ExecutableAccount,
    /// A supplied account has the wrong fixed allocation.
    InvalidAccountSize,
    /// A PIV1 account has the wrong type discriminator.
    InvalidAccountDiscriminator,
    /// Account bytes cannot be decoded canonically.
    InvalidAccountData,
    /// Checked fixed-envelope allocation or serialization failed.
    StateEnvelopeEncodingFailed,
    /// Complete account bytes differ from the prepared canonical prior state.
    StateEnvelopeChanged,
    /// A fixed address or stored bump is not its canonical PIV1 PDA.
    InvalidAccountPda,
    /// Separate fixed account roles alias one another.
    AccountAlias,
    /// A program binding differs from its canonical runtime identity.
    InvalidProgramIdentity,
    /// A requested account borrow conflicts with an existing borrow.
    AccountBorrowFailed,
    /// An account does not cover its runtime-derived rent-exempt minimum.
    AccountRentDeficit,
    /// Rent parameters are invalid or cannot yield a checked minimum.
    InvalidRent,
    /// Legacy token state does not satisfy the fixed custody restrictions.
    InvalidTokenCustody,
    /// Token-account native excess has no supported economic normalization path.
    UnsupportedTokenNativeExcess,
    /// A serialized layout uses an unsupported schema version.
    InvalidVersion,
    /// A required state object is not explicitly initialized or is malformed.
    InvalidInitialization,
    /// Initial contribution integration requires no prior economic history.
    InvalidBootstrapState,
    /// A protected principal SOL conversion has zero native input.
    ZeroPrincipalDeposit,
    /// A conversion attempts to spend more than recognized historical SOL.
    PrincipalDepositExceedsQueue,
    /// Principal conversion cannot allocate an actual deposit fee.
    UnsupportedPrincipalDepositFee,
    /// Pool facts are stale, malformed, unbound or have incorrect deposit deltas.
    InvalidPrincipalDepositPool,
    /// Protected deposit quote, receipt or custody differs from checked output.
    PrincipalDepositObservationMismatch,
    /// Principal conversion produces no tokens or cannot meet its protected minimum.
    PrincipalDepositMinimumNotMet,
    /// A conversion would consume even one lamport of historical book value.
    PrincipalDepositHistoricalValueLoss,
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
    /// Supplied facts do not prove a valid technically insufficient attempt.
    InvalidInsufficientAttempt,
    /// A round sequence does not match the active or next monotonic sequence.
    SequenceMismatch,
    /// A withdrawal-leg index is not the exact next or recorded index.
    LegIndexMismatch,
    /// A required distribution or withdrawal target is zero.
    ZeroTarget,
    /// A withdrawal leg has zero input.
    ZeroInput,
    /// An explicit SOL or JitoSOL contribution amount is zero.
    ZeroContribution,
    /// A custody observation cannot represent the required economic balance.
    InvalidCustodyObservation,
    /// An explicit before/after custody observation decreases.
    CustodyBalanceDecreased,
    /// An observed explicit custody increase differs from its expected amount.
    ContributionObservationMismatch,
    /// Accounted pending value exceeds the observed physical economic balance.
    PendingCustodyDeficit,
    /// An economic vault is below its own derived custody obligation.
    EconomicCustodyDeficit,
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
            Self::InvalidAccountOwner => "invalid account program owner",
            Self::AccountNotWritable => "required account is not writable",
            Self::MissingGuardianSignature => "missing entitled guardian signature",
            Self::ZeroKifClaim => "zero KIF claim",
            Self::StaleKifClaim => "stale KIF cumulative-claimed counter",
            Self::KifClaimExceeded => "KIF claim exceeds earned liability",
            Self::KifClaimBackingDeficit => "KIF custody does not cover complete backing",
            Self::KifClaimStateChanged => "state changed after KIF claim preparation",
            Self::KifClaimObservationMismatch => "KIF claim custody observations mismatch",
            Self::InvalidClockAccount => "invalid canonical Clock sysvar account",
            Self::ExecutableAccount => "custody or state account is executable",
            Self::InvalidAccountSize => "invalid fixed account allocation",
            Self::InvalidAccountDiscriminator => "invalid account discriminator",
            Self::InvalidAccountData => "invalid or noncanonical account data",
            Self::StateEnvelopeEncodingFailed => "state envelope encoding failed",
            Self::StateEnvelopeChanged => "state envelope changed after preparation",
            Self::InvalidAccountPda => "invalid fixed account PDA or bump",
            Self::AccountAlias => "fixed account roles alias",
            Self::InvalidProgramIdentity => "invalid canonical program identity",
            Self::AccountBorrowFailed => "account borrow failed",
            Self::AccountRentDeficit => "account rent-exempt minimum not covered",
            Self::InvalidRent => "invalid runtime rent parameters",
            Self::InvalidTokenCustody => "invalid legacy token custody state",
            Self::UnsupportedTokenNativeExcess => "token native excess normalization unsupported",
            Self::InvalidVersion => "invalid state-layout version",
            Self::InvalidInitialization => "invalid or missing state initialization",
            Self::InvalidBootstrapState => "initial bootstrap requires zero economic history",
            Self::ZeroPrincipalDeposit => "zero principal SOL deposit",
            Self::PrincipalDepositExceedsQueue => "deposit exceeds recognized historical SOL",
            Self::UnsupportedPrincipalDepositFee => "principal deposit fee allocation unsupported",
            Self::InvalidPrincipalDepositPool => "invalid principal deposit pool observations",
            Self::PrincipalDepositObservationMismatch => "principal deposit observations mismatch",
            Self::PrincipalDepositMinimumNotMet => "principal deposit minimum output not met",
            Self::PrincipalDepositHistoricalValueLoss => "principal deposit reduces historical value",
            Self::InvalidLifecycle => "invalid lifecycle transition",
            Self::PausedOperation => "operation blocked while paused",
            Self::InvalidTimestamp => "invalid timestamp",
            Self::TimestampRegression => "timestamp regression",
            Self::PreparationIntervalNotElapsed => "minimum preparation interval not elapsed",
            Self::InsufficientAttemptCooldownActive => {
                "valid-insufficient-attempt cooldown is active"
            }
            Self::InvalidInsufficientAttempt => {
                "attempt is not a valid technically insufficient result"
            }
            Self::SequenceMismatch => "distribution sequence mismatch",
            Self::LegIndexMismatch => "withdrawal-leg index mismatch",
            Self::ZeroTarget => "zero distribution or withdrawal target",
            Self::ZeroInput => "zero withdrawal-leg input",
            Self::ZeroContribution => "zero contribution",
            Self::InvalidCustodyObservation => "invalid custody observation",
            Self::CustodyBalanceDecreased => "custody balance decreased",
            Self::ContributionObservationMismatch => {
                "observed contribution does not match the expected amount"
            }
            Self::PendingCustodyDeficit => {
                "observed pending custody is below its accounted balance"
            }
            Self::EconomicCustodyDeficit => "economic custody is below its obligation",
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
