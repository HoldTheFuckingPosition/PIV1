//! Narrow deterministic boundary for a future SPL/Jito stake-pool adapter.
//!
//! This module defines values and behavior only. It performs no account
//! decoding, CPI, RPC, HTTP, allocation, clock access, or fund movement. A
//! future production implementation must derive its observations from
//! validated accounts and sysvars and must map the pinned SPL Stake Pool
//! instructions without trusting caller-supplied economic results.

use core::fmt;

/// Basis-point denominator used by protected adapter operations.
pub const SLIPPAGE_BASIS_POINTS_DENOMINATOR: u64 = 10_000;
/// Immutable V1 slippage ceiling accepted by the adapter boundary.
pub const MAX_PROTECTED_SLIPPAGE_BPS: u16 = 1;

/// A validated fee fraction used for one pool operation.
///
/// Zero fee is represented canonically as `0 / 1`. A zero denominator is
/// always rejected. Translating an external protocol's serialized zero-fee
/// representation into this canonical form is deferred to the Phase 3 adapter.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct FeeFraction {
    pub numerator: u64,
    pub denominator: u64,
}

impl FeeFraction {
    /// Canonical zero-fee value.
    pub const ZERO: Self = Self {
        numerator: 0,
        denominator: 1,
    };

    /// Rejects division by zero and fractions at or above one whole unit.
    pub fn validate(self) -> StakePoolResult<()> {
        if self.denominator == 0 {
            return Err(StakePoolError::DivisionByZero);
        }
        if self.numerator >= self.denominator {
            return Err(StakePoolError::InvalidFee);
        }
        Ok(())
    }
}

/// Bounded identity binding a quote to one exact pool observation.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct PoolSnapshotIdentity {
    pub current_epoch: u64,
    pub last_update_epoch: u64,
    pub revision: u64,
}

/// Account-independent pool facts needed by PIV1's narrow integration seam.
///
/// `maximum_deposit_lamports` is a bounded adapter capacity. The host mock uses
/// it to simulate unavailable deposit liquidity; its exact Phase 3 mapping is
/// provisional. `available_withdrawal_lamports` is the currently usable native
/// pool liquidity after adapter-specific safety reservations.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct PoolSnapshot {
    pub current_epoch: u64,
    pub last_update_epoch: u64,
    pub total_pool_lamports: u64,
    pub pool_token_supply: u64,
    pub sol_deposit_fee: FeeFraction,
    pub stake_withdrawal_fee: FeeFraction,
    pub minimum_delegation_lamports: u64,
    pub maximum_deposit_lamports: u64,
    pub available_withdrawal_lamports: u64,
    pub revision: u64,
}

impl PoolSnapshot {
    /// Returns the bounded identity that every quote and execution must bind.
    pub const fn identity(self) -> PoolSnapshotIdentity {
        PoolSnapshotIdentity {
            current_epoch: self.current_epoch,
            last_update_epoch: self.last_update_epoch,
            revision: self.revision,
        }
    }

    /// True only for the explicit empty-pool bootstrap representation.
    pub const fn is_bootstrap(self) -> bool {
        self.total_pool_lamports == 0 && self.pool_token_supply == 0
    }

    /// Validates intrinsic snapshot consistency and current-epoch freshness.
    pub fn validate(self) -> StakePoolResult<()> {
        if self.last_update_epoch > self.current_epoch {
            return Err(StakePoolError::InvalidSnapshot);
        }
        if self.last_update_epoch < self.current_epoch {
            return Err(StakePoolError::StalePool);
        }
        if (self.total_pool_lamports == 0) != (self.pool_token_supply == 0)
            || self.available_withdrawal_lamports > self.total_pool_lamports
            || self.minimum_delegation_lamports == 0
        {
            return Err(StakePoolError::InvalidSnapshot);
        }
        self.sol_deposit_fee.validate()?;
        self.stake_withdrawal_fee.validate()?;
        Ok(())
    }
}

/// Inputs used to quote or execute a protected native-SOL deposit.
///
/// Implementations independently derive their configured slippage floor and
/// use the greater of that floor and `caller_minimum_pool_tokens_out`; a caller
/// therefore cannot weaken the protected floor.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct SolDepositRequest {
    pub snapshot: PoolSnapshotIdentity,
    pub native_lamports: u64,
    pub caller_minimum_pool_tokens_out: u64,
    pub slippage_bps: u16,
}

/// Deterministic protected-deposit quote derived by an adapter.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct SolDepositQuote {
    pub snapshot: PoolSnapshotIdentity,
    pub native_lamports: u64,
    pub gross_pool_tokens: u64,
    pub deposit_fee_pool_tokens: u64,
    pub quoted_pool_tokens_out: u64,
    pub derived_slippage_floor_pool_tokens: u64,
    pub minimum_pool_tokens_out: u64,
}

/// Observed result of a protected deposit execution.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct SolDepositExecution {
    pub quote: SolDepositQuote,
    pub actual_pool_tokens_out: u64,
    pub actual_fee_pool_tokens: u64,
}

/// Deterministic identifier for one `(distribution sequence, leg index)`.
#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct WithdrawalId {
    pub sequence: u64,
    pub leg_index: u64,
}

/// Bounded adapter-local identity for one validated withdrawal source.
#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct WithdrawalSourceId(pub u32);

/// Inputs used to quote or initiate one maximum-safe protected withdrawal leg.
///
/// `remaining_pool_token_target` is the fixed unassigned round target. The
/// adapter, not the caller, chooses the leg input as the smaller of that target
/// and the validated source capacity.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct StakeWithdrawalRequest {
    pub snapshot: PoolSnapshotIdentity,
    pub withdrawal_id: WithdrawalId,
    pub source_id: WithdrawalSourceId,
    pub remaining_pool_token_target: u64,
    pub caller_minimum_native_lamports_out: u64,
    pub slippage_bps: u16,
}

/// Deterministic quote for one protected maximum-safe withdrawal leg.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct StakeWithdrawalQuote {
    pub snapshot: PoolSnapshotIdentity,
    pub withdrawal_id: WithdrawalId,
    pub source_id: WithdrawalSourceId,
    pub remaining_pool_token_target: u64,
    pub source_capacity_pool_tokens: u64,
    pub technical_minimum_pool_tokens: u64,
    pub pool_tokens_in: u64,
    pub withdrawal_fee_pool_tokens: u64,
    pub burned_pool_tokens: u64,
    pub expected_delegated_native_lamports: u64,
    pub derived_slippage_floor_native_lamports: u64,
    pub minimum_native_lamports_out: u64,
}

/// Delayed lifecycle states represented at the adapter boundary.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum DelayedWithdrawalStatus {
    Initiated,
    Deactivating,
    Inactive,
    Finalized,
}

/// Result recorded after a protected withdrawal leg is initiated.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct StakeWithdrawalInitiation {
    pub quote: StakeWithdrawalQuote,
    pub actual_delegated_native_lamports: u64,
    pub initiation_epoch: u64,
    pub deactivation_epoch: u64,
    pub first_eligible_finalization_epoch: u64,
    pub status: DelayedWithdrawalStatus,
    pub stake_rent_lamports: u64,
    pub metadata_rent_lamports: u64,
}

/// Read-only delayed status for one known withdrawal identifier.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct DelayedWithdrawal {
    pub withdrawal_id: WithdrawalId,
    pub status: DelayedWithdrawalStatus,
    pub initiation_epoch: u64,
    pub deactivation_epoch: u64,
    pub first_eligible_finalization_epoch: u64,
    pub delegated_native_lamports: u64,
    pub stake_rent_lamports: u64,
    pub metadata_rent_lamports: u64,
}

/// Input for replay-protected finalization of one inactive withdrawal leg.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct FinalizeWithdrawalRequest {
    pub withdrawal_id: WithdrawalId,
}

/// Exact categorized result of successful delayed finalization.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct StakeWithdrawalFinalization {
    pub withdrawal_id: WithdrawalId,
    pub status: DelayedWithdrawalStatus,
    pub initiation_epoch: u64,
    pub deactivation_epoch: u64,
    pub first_eligible_finalization_epoch: u64,
    pub finalized_epoch: u64,
    pub delegated_native_lamports: u64,
    pub cooldown_reward_lamports: u64,
    pub cooldown_loss_lamports: u64,
    pub recovered_stake_rent_lamports: u64,
    pub recovered_metadata_rent_lamports: u64,
    pub finalized_native_lamports: u64,
}

/// Explicit bounded failures for the production seam and host mock.
///
/// Anchor/program error-number mapping is intentionally deferred to the future
/// handler task.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum StakePoolError {
    InvalidSnapshot,
    InvalidConfiguration,
    StalePool,
    StaleQuote,
    ZeroInput,
    DivisionByZero,
    ArithmeticOverflow,
    NarrowingConversion,
    InvalidFee,
    InvalidSlippage,
    SlippageExceeded,
    TechnicalMinimumNotMet,
    InsufficientPoolLiquidity,
    UnknownWithdrawalSource,
    InsufficientSourceCapacity,
    InsufficientOperationalRent,
    UnknownWithdrawalIdentifier,
    IdentifierReuse,
    WithdrawalNotInactive,
    AlreadyFinalized,
    InjectedMockFailure,
}

impl fmt::Display for StakePoolError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(match self {
            Self::InvalidSnapshot => "invalid stake-pool snapshot",
            Self::InvalidConfiguration => "invalid adapter configuration",
            Self::StalePool => "stake-pool state is stale",
            Self::StaleQuote => "quote snapshot no longer matches the pool",
            Self::ZeroInput => "operation input is zero",
            Self::DivisionByZero => "division by zero",
            Self::ArithmeticOverflow => "checked arithmetic overflow",
            Self::NarrowingConversion => "wide result cannot be represented as u64",
            Self::InvalidFee => "invalid fee fraction",
            Self::InvalidSlippage => "slippage exceeds the immutable adapter bound",
            Self::SlippageExceeded => "protected minimum output is not satisfied",
            Self::TechnicalMinimumNotMet => "technical withdrawal minimum not met",
            Self::InsufficientPoolLiquidity => "insufficient pool liquidity",
            Self::UnknownWithdrawalSource => "unknown withdrawal source",
            Self::InsufficientSourceCapacity => "insufficient withdrawal-source capacity",
            Self::InsufficientOperationalRent => "insufficient operational rent capacity",
            Self::UnknownWithdrawalIdentifier => "unknown withdrawal identifier",
            Self::IdentifierReuse => "withdrawal identifier already exists",
            Self::WithdrawalNotInactive => "withdrawal is not inactive",
            Self::AlreadyFinalized => "withdrawal was already finalized",
            Self::InjectedMockFailure => "host mock failure injected",
        })
    }
}

/// Result alias for the bounded stake-pool seam.
pub type StakePoolResult<T> = Result<T, StakePoolError>;

/// Statically dispatched, synchronous contract used by future PIV1 handlers.
///
/// Quote outputs are informational. Execution methods accept the original
/// bounded request, revalidate its snapshot identity, and independently derive
/// all outputs; they never accept a caller-provided object labeled as
/// "validated." Implementations must commit no partial mutation on failure.
pub trait StakePoolAdapter: Sized {
    fn pool_snapshot(&self) -> StakePoolResult<PoolSnapshot>;

    fn quote_sol_deposit(
        &self,
        request: SolDepositRequest,
    ) -> StakePoolResult<SolDepositQuote>;

    fn execute_protected_sol_deposit(
        &mut self,
        request: SolDepositRequest,
    ) -> StakePoolResult<SolDepositExecution>;

    fn quote_stake_withdrawal(
        &self,
        request: StakeWithdrawalRequest,
    ) -> StakePoolResult<StakeWithdrawalQuote>;

    fn initiate_protected_stake_withdrawal(
        &mut self,
        request: StakeWithdrawalRequest,
    ) -> StakePoolResult<StakeWithdrawalInitiation>;

    fn delayed_withdrawal(
        &self,
        withdrawal_id: WithdrawalId,
    ) -> StakePoolResult<DelayedWithdrawal>;

    fn finalize_delayed_stake_withdrawal(
        &mut self,
        request: FinalizeWithdrawalRequest,
    ) -> StakePoolResult<StakeWithdrawalFinalization>;
}
