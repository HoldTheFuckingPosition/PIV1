//! Stable custom instruction error ABI. Existing numbers must never be changed
//! or reused. 6000..=6998 is reserved for explicit Piv1Error assignments; 6999
//! belongs only to host runtime unavailability. New variants require new literal
//! match arms. Enum ordering/discriminants do not define these wire numbers.

use anchor_lang::solana_program::program_error::ProgramError;
use crate::{errors::Piv1Error, kif_claim_execution::KifClaimExecutionError};

pub const HOST_RUNTIME_UNAVAILABLE_CODE: u32 = 6999;

/// Exhaustive literal mapping; extending the state error enum requires review.
pub const fn piv1_error_code(error: Piv1Error) -> u32 {
    match error {
        Piv1Error::InvalidAccountOwner => 6000,
        Piv1Error::AccountNotWritable => 6001,
        Piv1Error::MissingGuardianSignature => 6002,
        Piv1Error::ZeroKifClaim => 6003,
        Piv1Error::StaleKifClaim => 6004,
        Piv1Error::KifClaimExceeded => 6005,
        Piv1Error::KifClaimBackingDeficit => 6006,
        Piv1Error::KifClaimStateChanged => 6007,
        Piv1Error::KifClaimObservationMismatch => 6008,
        Piv1Error::InvalidClockAccount => 6009,
        Piv1Error::ExecutableAccount => 6010,
        Piv1Error::InvalidAccountSize => 6011,
        Piv1Error::InvalidAccountDiscriminator => 6012,
        Piv1Error::InvalidAccountData => 6013,
        Piv1Error::StateEnvelopeEncodingFailed => 6014,
        Piv1Error::StateEnvelopeChanged => 6015,
        Piv1Error::InvalidAccountPda => 6016,
        Piv1Error::AccountAlias => 6017,
        Piv1Error::InvalidProgramIdentity => 6018,
        Piv1Error::AccountBorrowFailed => 6019,
        Piv1Error::AccountRentDeficit => 6020,
        Piv1Error::InvalidRent => 6021,
        Piv1Error::InvalidTokenCustody => 6022,
        Piv1Error::UnsupportedTokenNativeExcess => 6023,
        Piv1Error::InvalidVersion => 6024,
        Piv1Error::InvalidInitialization => 6025,
        Piv1Error::InvalidBootstrapState => 6026,
        Piv1Error::ZeroPrincipalDeposit => 6027,
        Piv1Error::PrincipalDepositExceedsQueue => 6028,
        Piv1Error::UnsupportedPrincipalDepositFee => 6029,
        Piv1Error::InvalidPrincipalDepositPool => 6030,
        Piv1Error::PrincipalDepositObservationMismatch => 6031,
        Piv1Error::PrincipalDepositMinimumNotMet => 6032,
        Piv1Error::PrincipalDepositHistoricalValueLoss => 6033,
        Piv1Error::InvalidLifecycle => 6034,
        Piv1Error::PausedOperation => 6035,
        Piv1Error::InvalidTimestamp => 6036,
        Piv1Error::TimestampRegression => 6037,
        Piv1Error::PreparationIntervalNotElapsed => 6038,
        Piv1Error::InsufficientAttemptCooldownActive => 6039,
        Piv1Error::InvalidInsufficientAttempt => 6040,
        Piv1Error::SequenceMismatch => 6041,
        Piv1Error::LegIndexMismatch => 6042,
        Piv1Error::ZeroTarget => 6043,
        Piv1Error::ZeroInput => 6044,
        Piv1Error::ZeroContribution => 6045,
        Piv1Error::InvalidCustodyObservation => 6046,
        Piv1Error::CustodyBalanceDecreased => 6047,
        Piv1Error::ContributionObservationMismatch => 6048,
        Piv1Error::PendingCustodyDeficit => 6049,
        Piv1Error::EconomicCustodyDeficit => 6050,
        Piv1Error::TargetExceeded => 6051,
        Piv1Error::NonMaximumSafeLegFill => 6052,
        Piv1Error::TechnicalFloorNotMet => 6053,
        Piv1Error::UsefulLegBoundExceeded => 6054,
        Piv1Error::Replay => 6055,
        Piv1Error::AlreadyFinalized => 6056,
        Piv1Error::TargetNotAssigned => 6057,
        Piv1Error::CountMismatch => 6058,
        Piv1Error::CumulativeReconciliationMismatch => 6059,
        Piv1Error::EscrowReconciliationMismatch => 6060,
        Piv1Error::ObligationExceeded => 6061,
        Piv1Error::OutstandingLiability => 6062,
        Piv1Error::SettlementReplay => 6063,
        Piv1Error::HighWaterMarkDecrease => 6064,
        Piv1Error::InvalidGuardianBitmap => 6065,
        Piv1Error::InvalidGuardianCount => 6066,
        Piv1Error::InvalidGuardianSet => 6067,
        Piv1Error::InvalidAddress => 6068,
        Piv1Error::InvalidSlippage => 6069,
        Piv1Error::InvalidSplit => 6070,
        Piv1Error::InvalidTimingConfiguration => 6071,
        Piv1Error::ArithmeticOverflow => 6072,
        Piv1Error::RecoveryRequired => 6073,
    }
}

/// Preserve System CPI ProgramError exactly; only library-state errors receive
/// PIV1 custom codes. Runtime Rent errors bypass this mapping unchanged.
pub fn execution_program_error(error: KifClaimExecutionError) -> ProgramError {
    match error {
        KifClaimExecutionError::State(error) => ProgramError::Custom(piv1_error_code(error)),
        KifClaimExecutionError::Invocation(error) => error,
        KifClaimExecutionError::HostRuntimeUnavailable => ProgramError::Custom(HOST_RUNTIME_UNAVAILABLE_CODE),
    }
}
