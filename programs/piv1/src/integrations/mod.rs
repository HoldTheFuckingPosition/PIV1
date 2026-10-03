//! External protocol values and bounded Jito account identity authentication.
//! The private protected-deposit component maps the pinned SPL wire format;
//! remaining production stake-pool adapter operations are not implemented.

pub mod jito;
pub mod jito_identity;
pub(crate) mod jito_deposit;
pub(crate) mod jito_withdrawal_preparation;
pub mod stake_pool;

pub use jito::JitoStrategy;
pub use stake_pool::{
    DelayedWithdrawal, DelayedWithdrawalStatus, FeeFraction,
    FinalizeWithdrawalRequest, PoolSnapshot, PoolSnapshotIdentity,
    SolDepositExecution, SolDepositQuote, SolDepositRequest,
    StakePoolAdapter, StakePoolError, StakePoolResult,
    StakeWithdrawalFinalization, StakeWithdrawalInitiation,
    StakeWithdrawalQuote, StakeWithdrawalRequest, WithdrawalId,
    WithdrawalSourceId, MAX_PROTECTED_SLIPPAGE_BPS,
    SLIPPAGE_BASIS_POINTS_DENOMINATOR,
};
