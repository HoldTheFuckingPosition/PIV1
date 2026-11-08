//! D107/D108 deterministic accounting for the restricted Jupiter preparation.
//!
//! Values must come from authenticated PIV/Jito accounts and measured CPI
//! effects. This module is not an oracle, route authenticator or instruction.
//! No caller-selected slippage parameter or administrative override exists.
use anchor_lang::prelude::borsh;
use crate::errors::{Piv1Error, Piv1Result};

pub const DELAYED_SHORTFALL_THRESHOLD_LAMPORTS: u64 = 1_500_000_000;
pub const MAX_TOTAL_SWAP_LOSS_BPS: u64 = 500;
pub const MIN_RETAINED_BOOK_VALUE_BPS: u64 = 9_500;
const BPS: u128 = 10_000;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum LiquidityRoute { Liquid, Jupiter, DelayedWithdrawal }

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct LiquidityPlan {
    pub pending_used_lamports: u64,
    pub carry_used_lamports: u64,
    pub shortfall_lamports: u64,
    pub route: LiquidityRoute,
}

/// Pending contributions first, then eligible carry. Protected native principal,
/// rent and unrelated physical balances must never be supplied as eligible SOL.
pub fn liquidity_plan(outgoing: u64, eligible_pending: u64, eligible_carry: u64) -> LiquidityPlan {
    let pending = eligible_pending.min(outgoing);
    let carry = eligible_carry.min(outgoing - pending);
    let shortfall = outgoing - pending - carry;
    LiquidityPlan {
        pending_used_lamports: pending, carry_used_lamports: carry,
        shortfall_lamports: shortfall,
        route: if shortfall == 0 { LiquidityRoute::Liquid }
            else if shortfall < DELAYED_SHORTFALL_THRESHOLD_LAMPORTS { LiquidityRoute::Jupiter }
            else { LiquidityRoute::DelayedWithdrawal },
    }
}

fn narrow(value: u128) -> Piv1Result<u64> {
    u64::try_from(value).map_err(|_| Piv1Error::ArithmeticOverflow)
}
fn ceil_div(value: u128, divisor: u128) -> Piv1Result<u128> {
    if divisor == 0 { return Err(Piv1Error::InvalidSwapInput); }
    (value / divisor).checked_add(u128::from(value % divisor != 0))
        .ok_or(Piv1Error::ArithmeticOverflow)
}
fn book(units: u64, total: u64, supply: u64) -> Piv1Result<u64> {
    if total == 0 || supply == 0 { return Err(Piv1Error::InvalidSwapInput); }
    narrow(u128::from(units).checked_mul(u128::from(total))
        .ok_or(Piv1Error::ArithmeticOverflow)? / u128::from(supply))
}

/// Exact decrease in the entire historical position at ONE unchanged ratio.
/// Independently flooring the spent units can understate this debit by one.
pub fn exact_book_debit(historical_units: u64, spent: u64, total: u64, supply: u64) -> Piv1Result<u64> {
    if spent == 0 { return Err(Piv1Error::InvalidSwapInput); }
    let remaining = historical_units.checked_sub(spent).ok_or(Piv1Error::InvalidSwapInput)?;
    book(historical_units, total, supply)?.checked_sub(book(remaining, total, supply)?)
        .ok_or(Piv1Error::ArithmeticOverflow)
}

/// Largest integer input whose exact historical debit fits the small shortfall.
/// Invert the residual book-value floor; no caller-selected micro-input budget.
pub fn largest_safe_input(historical_units: u64, total: u64, supply: u64, shortfall: u64) -> Piv1Result<u64> {
    if shortfall == 0 || shortfall >= DELAYED_SHORTFALL_THRESHOLD_LAMPORTS {
        return Err(Piv1Error::InvalidSwapInput);
    }
    let before = book(historical_units, total, supply)?;
    let minimum_residual_book = before.saturating_sub(shortfall);
    let minimum_residual_units = narrow(ceil_div(
        u128::from(minimum_residual_book).checked_mul(u128::from(supply))
            .ok_or(Piv1Error::ArithmeticOverflow)?, u128::from(total))?)?;
    let input = historical_units.checked_sub(minimum_residual_units)
        .ok_or(Piv1Error::ArithmeticOverflow)?;
    let debit = exact_book_debit(historical_units, input, total, supply)?;
    if debit == 0 || debit > shortfall { return Err(Piv1Error::InvalidSwapInput); }
    Ok(input)
}

/// A positive debit always requires positive actual native output, even at dust
/// sizes. The combined loss allowance is immutable and is applied only once.
pub fn minimum_native_output(debit: u64) -> Piv1Result<u64> {
    if debit == 0 { return Err(Piv1Error::InvalidSwapInput); }
    narrow(ceil_div(u128::from(debit).checked_mul(u128::from(MIN_RETAINED_BOOK_VALUE_BPS))
        .ok_or(Piv1Error::ArithmeticOverflow)?, BPS)?)
}

#[derive(anchor_lang::AnchorSerialize, anchor_lang::AnchorDeserialize, anchor_lang::InitSpace, Clone, Copy, Debug, PartialEq, Eq)]
pub struct SwapReceipt {
    pub spent_units: u64,
    pub official_book_debit_lamports: u64,
    pub actual_native_output_lamports: u64,
    pub minimum_native_output_lamports: u64,
}

/// Output is the authenticated swap-only native receipt after excluding rents,
/// refunds, prefunds and unrelated credits. A quote cannot supply this argument.
pub fn validate_receipt(historical_units: u64, actual_spent: u64, total: u64, supply: u64,
    shortfall: u64, actual_native_output: u64) -> Piv1Result<SwapReceipt>
{
    if actual_spent != largest_safe_input(historical_units, total, supply, shortfall)? {
        return Err(Piv1Error::InvalidSwapInput);
    }
    let debit = exact_book_debit(historical_units, actual_spent, total, supply)?;
    let minimum = minimum_native_output(debit)?;
    if actual_native_output < minimum { return Err(Piv1Error::SwapMinimumNotMet); }
    Ok(SwapReceipt { spent_units: actual_spent, official_book_debit_lamports: debit,
        actual_native_output_lamports: actual_native_output, minimum_native_output_lamports: minimum })
}
