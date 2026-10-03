//! Stable custom instruction error ABI. Existing numbers must never be changed
//! or reused. 6000..=6073 maps Piv1Error; 6100..=6118 and 6120..=6131 map initializer
//! validation failures; 6145 is unsupported withdrawal preparation. Other
//! unassigned codes through 6998 remain reserved; 6999
//! belongs only to host runtime unavailability. New variants require new literal
//! match arms. Enum ordering/discriminants do not define these wire numbers.

use anchor_lang::solana_program::program_error::ProgramError;
use crate::{errors::Piv1Error, kif_claim_execution::KifClaimExecutionError};

/// Positive native shortfall requires the still-unimplemented real withdrawal
/// preparation profile. This rejection never records a technical insufficiency.
pub const DISTRIBUTION_WITHDRAWAL_UNAVAILABLE_CODE: u32 = 6145;

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

/// Initializer validation mapping is exhaustive at every nested boundary.
/// Actual System/Token CPI and trusted sysvar errors propagate unchanged.
pub fn initialization_program_error(error: crate::genesis_initialization::GenesisInitializationError) -> ProgramError {
    use crate::genesis_initialization::GenesisInitializationError as E;
    match error {
        E::Allocation(error) => allocation_program_error(error),
        E::State(error) => ProgramError::Custom(piv1_error_code(error)),
        E::Invocation(error) => error,
        E::HostRuntimeUnavailable => ProgramError::Custom(HOST_RUNTIME_UNAVAILABLE_CODE),
        E::InvalidRoles => ProgramError::Custom(6130),
        E::ObservationMismatch => ProgramError::Custom(6131),
        E::Recipient(error) => recipient_program_error(error),
    }
}

fn allocation_program_error(error: crate::genesis_allocation::GenesisAllocationError) -> ProgramError {
    use crate::genesis_allocation::GenesisAllocationError as E;
    match error {
        E::Preflight(error) => preflight_program_error(error),
        E::State(error) => ProgramError::Custom(piv1_error_code(error)),
        E::Invocation(error) => error,
        E::HostRuntimeUnavailable => ProgramError::Custom(HOST_RUNTIME_UNAVAILABLE_CODE),
        E::InvalidRoles => ProgramError::Custom(6121),
        E::InvalidPayer => ProgramError::Custom(6122),
        E::InsufficientPayerRent => ProgramError::Custom(6123),
        E::ObservationMismatch => ProgramError::Custom(6124),
        E::Recipient(error) => recipient_program_error(error),
    }
}

fn recipient_program_error(error: crate::genesis_recipients::GenesisRecipientError) -> ProgramError {
    use crate::genesis_recipients::GenesisRecipientError as E;
    match error {
        E::Preflight(error) => preflight_program_error(error),
        E::State(error) => ProgramError::Custom(piv1_error_code(error)),
        E::HostRuntimeUnavailable => ProgramError::Custom(HOST_RUNTIME_UNAVAILABLE_CODE),
        E::InvalidRoles => ProgramError::Custom(6125),
        E::UnapprovedRecipient => ProgramError::Custom(6126),
        E::InvalidRecipientVault => ProgramError::Custom(6127),
        E::UnfundedRecipient => ProgramError::Custom(6128),
        E::ObservationMismatch => ProgramError::Custom(6129),
    }
}

fn preflight_program_error(error: crate::genesis_preflight::GenesisPreflightError) -> ProgramError {
    use crate::{genesis_model::GenesisModelError as M, genesis_preflight::GenesisPreflightError as E,
        instructions::initialize::GenesisModelFormatError as F};
    match error {
        E::Model(M::Format(F::InvalidLength | F::InvalidSelector | F::UnsupportedVersion
            | F::InvalidBoolean | F::InvalidSlotPermutation)) => ProgramError::InvalidInstructionData,
        E::Model(M::Authorization(error)) => squads_program_error(error),
        E::Model(M::State(error)) | E::State(error) => ProgramError::Custom(piv1_error_code(error)),
        E::Protocol(error) => protocol_program_error(error),
        E::InvalidRoles => ProgramError::Custom(6120),
    }
}

pub(crate) fn squads_program_error(error: crate::squads_execution::SquadsExecutionError) -> ProgramError {
    use crate::squads_execution::SquadsExecutionError as E;
    match error {
        E::HostRuntimeUnavailable => ProgramError::Custom(HOST_RUNTIME_UNAVAILABLE_CODE),
        E::Runtime(error) => error,
        E::State(error) => ProgramError::Custom(piv1_error_code(error)),
        E::InvalidInvocation => ProgramError::Custom(6100),
        E::InvalidInstructionsSysvar => ProgramError::Custom(6101),
        E::InvalidProposal => ProgramError::Custom(6102),
        E::InvalidTransaction => ProgramError::Custom(6103),
        E::UnsupportedMessage => ProgramError::Custom(6104),
        E::MessageMismatch => ProgramError::Custom(6105),
        E::TimelockNotReleased => ProgramError::Custom(6106),
    }
}

pub(crate) fn protocol_program_error(error: crate::integrations::jito_identity::JitoIdentityError) -> ProgramError {
    use crate::integrations::jito_identity::JitoIdentityError as E;
    match error {
        E::InvalidIdentity => ProgramError::Custom(6107),
        E::AccountAlias => ProgramError::Custom(6108),
        E::InvalidOwner => ProgramError::Custom(6109),
        E::InvalidExecutable => ProgramError::Custom(6110),
        E::BorrowFailed => ProgramError::Custom(6111),
        E::UnsupportedProgram => ProgramError::Custom(6112),
        E::InvalidPool => ProgramError::Custom(6113),
        E::InvalidFee => ProgramError::Custom(6114),
        E::InvalidList => ProgramError::Custom(6115),
        E::InvalidReserve => ProgramError::Custom(6116),
        E::InvalidMint => ProgramError::Custom(6117),
        E::InvalidReceiver => ProgramError::Custom(6118),
    }
}
