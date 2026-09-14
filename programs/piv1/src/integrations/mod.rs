//! External protocol values and bounded Jito account identity authentication.
//! No protocol CPI is implemented.

pub mod jito;
pub mod jito_identity;
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
