//! External protocol boundary. Task 2.1 defines values and behavior only; no
//! account decoding or CPI is implemented.

pub mod jito;
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
