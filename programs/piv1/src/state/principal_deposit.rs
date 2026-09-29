//! Protected conversion of recognized historical SOL while the bound round is Idle.
//!
//! This pure boundary performs no transfer or CPI and authenticates no account.
//! Composing handlers must derive both pool observations, custody and execution
//! from the same atomic protected deposit using validated accounts and Clock.
//! The existing adapter identity remains provisional; no real revision counter
//! or mock capacity/liquidity update rule is introduced here.
//!
//! Only zero-fee conversions preserving historical book value and HWM coverage
//! are supported. Even one lamport of integer loss rejects and leaves SOL queued
//! in the atomic host composition. This is not general principal staking support.

use crate::{
    errors::{Piv1Error, Piv1Result},
    integrations::{
        FeeFraction, PoolSnapshot, SolDepositExecution, SolDepositQuote,
        SolDepositRequest, SLIPPAGE_BASIS_POINTS_DENOMINATOR,
    },
    state::{
        reconciliation::{
            economic_custody_surplus, observed_token_book_value,
            validate_custody_state_binding, EconomicCustodyObservation,
            EconomicVaultAmounts,
        },
        ActiveDistribution, DistributionLifecycle, PivConfig,
    },
};

/// Same-operation observations, not trusted transfer receipts or serialized state.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct PrincipalSolDepositObservation {
    pub request: SolDepositRequest,
    pub execution: SolDepositExecution,
    pub pool_before: PoolSnapshot,
    pub pool_after: PoolSnapshot,
    pub custody_before: EconomicCustodyObservation,
    pub custody_after: EconomicCustodyObservation,
}

/// Exact conversion amounts and independently derived historical values.
/// Separate next-cycle yield is excluded from both book values.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct PrincipalSolDepositRecord {
    pub deposited_sol_lamports: u64,
    pub minted_jitosol_units: u64,
    pub historical_value_before_lamports: u64,
    pub historical_value_after_lamports: u64,
}

/// Records an unpaused Idle-to-Idle conversion of already recognized principal.
///
/// Config supplies the 0–1 bps tolerance; the caller may only strengthen its
/// minimum. The entire quote and receipt are checked against independent integer
/// output arithmetic and exact custody/pool deltas. Only the two historical
/// asset-unit fields change. No principal/HWM exception, fee allocation, carry
/// subsidy, contribution netting rule or automatic amount optimizer is implied.
pub fn record_protected_principal_deposit(
    config: &mut PivConfig,
    round: &ActiveDistribution,
    observation: PrincipalSolDepositObservation,
) -> Piv1Result<PrincipalSolDepositRecord> {
    record_with_profile(config, round, observation.custody_before,
        observation.custody_after, &observation)
}

/// Internal phased evidence only. The legacy profile retains its complete
/// snapshot/receipt validation; the native profile uses authenticated protocol
/// facts without inventing mock revision, liquidity or capacity fields.
pub(crate) trait DepositProfile {
    fn amount(&self) -> u64;
    fn slippage(&self) -> u16;
    fn validate_pools(&self) -> Piv1Result<()>;
    fn validate_zero_fee(&self) -> Piv1Result<()>;
    fn validate_supply_before(&self, custody: EconomicCustodyObservation) -> Piv1Result<()>;
    fn minted_and_validate_execution(&self, config: &PivConfig) -> Piv1Result<u64>;
    fn validate_supply_after(&self, custody: EconomicCustodyObservation) -> Piv1Result<()>;
    fn book_value_before(&self, units: u64) -> Piv1Result<u64>;
    fn book_value_after(&self, units: u64) -> Piv1Result<u64>;
}

/// Keep all accounting gates and their original order shared by both profiles.
pub(crate) fn record_with_profile(
    config: &mut PivConfig, round: &ActiveDistribution,
    custody_before: EconomicCustodyObservation, custody_after: EconomicCustodyObservation,
    profile: &impl DepositProfile,
) -> Piv1Result<PrincipalSolDepositRecord> {
    config.ensure_unpaused()?;
    validate_custody_state_binding(config, round)?;
    if round.bump != config.bumps.active_distribution {
        return Err(Piv1Error::InvalidAccountPda);
    }
    if round.lifecycle != DistributionLifecycle::Idle {
        return Err(Piv1Error::InvalidLifecycle);
    }
    let amount = profile.amount();
    if amount == 0 {
        return Err(Piv1Error::ZeroPrincipalDeposit);
    }
    if amount > config.accounted_historical_sol_lamports {
        return Err(Piv1Error::PrincipalDepositExceedsQueue);
    }
    if profile.slippage() != config.configured_slippage_bps {
        return Err(Piv1Error::InvalidSlippage);
    }
    if economic_custody_surplus(config, round, custody_before)?
        != EconomicVaultAmounts::default()
    {
        return Err(Piv1Error::InvalidCustodyObservation);
    }
    profile.validate_pools()?;
    profile.validate_zero_fee()?;
    profile.validate_supply_before(custody_before)?;
    let minted = profile.minted_and_validate_execution(config)?;
    let mut expected_after = custody_before.amounts()?;
    expected_after.principal_sol_lamports = expected_after.principal_sol_lamports
        .checked_sub(amount)
        .ok_or(Piv1Error::PrincipalDepositExceedsQueue)?;
    expected_after.principal_jitosol_units =
        add(expected_after.principal_jitosol_units, minted)?;
    if !custody_before.same_floors(custody_after)
        || custody_after.amounts()? != expected_after
    {
        return Err(Piv1Error::PrincipalDepositObservationMismatch);
    }
    profile.validate_supply_after(custody_after)?;
    let before_value = add(config.accounted_historical_sol_lamports,
        profile.book_value_before(config.accounted_historical_jitosol_units)?)?;
    let mut next = config.clone();
    next.accounted_historical_sol_lamports = next.accounted_historical_sol_lamports
        .checked_sub(amount)
        .ok_or(Piv1Error::PrincipalDepositExceedsQueue)?;
    next.accounted_historical_jitosol_units =
        add(next.accounted_historical_jitosol_units, minted)?;
    let after_value = add(next.accounted_historical_sol_lamports,
        profile.book_value_after(next.accounted_historical_jitosol_units)?)?;
    if after_value < before_value {
        return Err(Piv1Error::PrincipalDepositHistoricalValueLoss);
    }
    if after_value < config.protected_principal_hwm_lamports {
        return Err(Piv1Error::HighWaterMarkDecrease);
    }
    if economic_custody_surplus(&next, round, custody_after)?
        != EconomicVaultAmounts::default()
    {
        return Err(Piv1Error::CumulativeReconciliationMismatch);
    }
    *config = next;
    Ok(PrincipalSolDepositRecord {
        deposited_sol_lamports: amount,
        minted_jitosol_units: minted,
        historical_value_before_lamports: before_value,
        historical_value_after_lamports: after_value,
    })
}

impl DepositProfile for PrincipalSolDepositObservation {
    fn amount(&self) -> u64 { self.request.native_lamports }
    fn slippage(&self) -> u16 { self.request.slippage_bps }
    fn validate_pools(&self) -> Piv1Result<()> {
        let Self { request, pool_before, pool_after, .. } = *self;
        pool_before.validate().map_err(|_| Piv1Error::InvalidPrincipalDepositPool)?;
        pool_after.validate().map_err(|_| Piv1Error::InvalidPrincipalDepositPool)?;
        if request.snapshot != pool_before.identity()
            || pool_after.current_epoch != pool_before.current_epoch
            || pool_after.last_update_epoch != pool_before.last_update_epoch
            || pool_after.sol_deposit_fee != pool_before.sol_deposit_fee
            || pool_after.stake_withdrawal_fee != pool_before.stake_withdrawal_fee
            || pool_after.minimum_delegation_lamports != pool_before.minimum_delegation_lamports
        { return Err(Piv1Error::InvalidPrincipalDepositPool); }
        Ok(())
    }
    fn validate_zero_fee(&self) -> Piv1Result<()> {
        // Positive output with a nonzero fee incurs a ceiled token fee, even
        // where preexisting yield could mask that economic cost.
        if self.pool_before.sol_deposit_fee != FeeFraction::ZERO
            || self.execution.actual_fee_pool_tokens != 0
            || self.execution.quote.deposit_fee_pool_tokens != 0
        { return Err(Piv1Error::UnsupportedPrincipalDepositFee); }
        Ok(())
    }
    fn validate_supply_before(&self, custody: EconomicCustodyObservation) -> Piv1Result<()> {
        validate_combined_supply(custody, self.pool_before)
    }
    fn minted_and_validate_execution(&self, config: &PivConfig) -> Piv1Result<u64> {
        let Self { request, execution, pool_before, pool_after, .. } = *self;
        let minted = if pool_before.is_bootstrap() { request.native_lamports } else {
            piv1_math::checked_mul_div_floor(request.native_lamports,
                pool_before.pool_token_supply, pool_before.total_pool_lamports)?
        };
        let (floor, minimum) = protected_minimum(minted, config.configured_slippage_bps,
            request.caller_minimum_pool_tokens_out)?;
        let expected_quote = SolDepositQuote {
            snapshot: pool_before.identity(), native_lamports: request.native_lamports,
            gross_pool_tokens: minted, deposit_fee_pool_tokens: 0, quoted_pool_tokens_out: minted,
            derived_slippage_floor_pool_tokens: floor, minimum_pool_tokens_out: minimum,
        };
        if execution.quote != expected_quote || execution.actual_pool_tokens_out != minted {
            return Err(Piv1Error::PrincipalDepositObservationMismatch);
        }
        if pool_after.total_pool_lamports != add(pool_before.total_pool_lamports, request.native_lamports)?
            || pool_after.pool_token_supply != add(pool_before.pool_token_supply, minted)?
        { return Err(Piv1Error::InvalidPrincipalDepositPool); }
        Ok(minted)
    }
    fn validate_supply_after(&self, custody: EconomicCustodyObservation) -> Piv1Result<()> {
        validate_combined_supply(custody, self.pool_after)
    }
    fn book_value_before(&self, units: u64) -> Piv1Result<u64> { observed_token_book_value(units, self.pool_before) }
    fn book_value_after(&self, units: u64) -> Piv1Result<u64> { observed_token_book_value(units, self.pool_after) }
}

pub(crate) fn protected_minimum(minted: u64, slippage: u16, caller: u64) -> Piv1Result<(u64, u64)> {
    let floor = piv1_math::checked_mul_div_floor(minted,
        SLIPPAGE_BASIS_POINTS_DENOMINATOR.checked_sub(u64::from(slippage))
            .ok_or(Piv1Error::InvalidSlippage)?, SLIPPAGE_BASIS_POINTS_DENOMINATOR)?;
    let minimum = floor.max(caller);
    if minted == 0 || minted < minimum { return Err(Piv1Error::PrincipalDepositMinimumNotMet); }
    Ok((floor, minimum))
}

fn validate_combined_supply(
    custody: EconomicCustodyObservation,
    pool: PoolSnapshot,
) -> Piv1Result<()> {
    if add(custody.principal_jitosol_units, custody.pending_jitosol_units)?
        > pool.pool_token_supply
    {
        return Err(Piv1Error::InvalidPrincipalDepositPool);
    }
    Ok(())
}

fn add(a: u64, b: u64) -> Piv1Result<u64> {
    a.checked_add(b).ok_or(Piv1Error::ArithmeticOverflow)
}
