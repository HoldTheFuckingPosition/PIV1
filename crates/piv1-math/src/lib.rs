#![forbid(unsafe_code)]

//! Deterministic checked-integer accounting primitives for PIV1.
//!
//! Economic amounts use [`Amount`], an unsigned 64-bit integer matching
//! Solana lamports and SPL token base units. Multiplication is performed in a
//! checked `u128` intermediate, and every conversion back to [`Amount`] is
//! checked. This crate has no Solana, Anchor, Jito, SPL, clock, account, or
//! network dependencies.

use core::fmt;

/// A SOL-lamport or token-base-unit amount stored by PIV1.
pub type Amount = u64;

/// Basis-point denominator used by the fixed gross-yield split.
pub const BASIS_POINTS_DENOMINATOR: Amount = 10_000;
/// HTFP reserve share of gross yield, in basis points.
pub const HTFP_RESERVE_BPS: Amount = 5_900;
/// Permanent compound share of gross yield, in basis points.
pub const PERMANENT_COMPOUND_BPS: Amount = 1_950;
/// Team Owner Pool share of gross yield, in basis points.
pub const TEAM_OWNER_POOL_BPS: Amount = 1_950;
/// KIF share of gross yield, in basis points.
pub const KIF_BPS: Amount = 200;
/// Maximum valid number of active guardians in the confirmed guardian set.
pub const MAX_ACTIVE_GUARDIANS: u8 = 6;

/// Explicit failures produced by checked PIV1 arithmetic.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum MathError {
    /// A multiply/divide operation received a zero denominator.
    DivisionByZero,
    /// An addition exceeded the destination integer type.
    AdditionOverflow,
    /// A multiplication exceeded its checked intermediate or destination type.
    MultiplicationOverflow,
    /// A subtraction would have produced a negative unsigned value.
    SubtractionUnderflow,
    /// A checked wide result could not be represented as [`Amount`].
    NarrowingConversion,
    /// The active count exceeded the confirmed six-guardian set.
    InvalidActiveGuardianCount {
        /// Rejected caller-supplied active-guardian count.
        active_guardians: u8,
    },
}

impl fmt::Display for MathError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::DivisionByZero => formatter.write_str("division by zero"),
            Self::AdditionOverflow => formatter.write_str("addition overflow"),
            Self::MultiplicationOverflow => formatter.write_str("multiplication overflow"),
            Self::SubtractionUnderflow => formatter.write_str("subtraction underflow"),
            Self::NarrowingConversion => formatter.write_str("narrowing conversion failed"),
            Self::InvalidActiveGuardianCount { active_guardians } => write!(
                formatter,
                "active guardian count {active_guardians} exceeds {MAX_ACTIVE_GUARDIANS}"
            ),
        }
    }
}

/// Result of the confirmed fixed gross-yield split.
///
/// Each named allocation is independently floored. [`Self::dust`] is the
/// exact residual, so all five fields sum to the input gross yield. Dust stays
/// protected inside PIV1 and is not a beneficiary allocation.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct GrossYieldSplit {
    /// Floored 59% HTFP reserve allocation.
    pub htfp_reserve: Amount,
    /// Floored 19.5% permanent compound allocation.
    pub permanent_compound: Amount,
    /// Floored 19.5% Team Owner Pool allocation.
    pub team_owner_pool: Amount,
    /// Floored 2% KIF allocation.
    pub kif: Amount,
    /// Gross yield left after all four independent floors.
    pub dust: Amount,
}

/// Pure numeric components eligible for a checked high-water-mark increase.
///
/// This structure does not decide when a component is eligible. Later state
/// transition logic must supply already validated values.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct HighWaterMarkComponents {
    /// Conservatively reconciled SOL value of new contributions.
    pub contribution_value: Amount,
    /// Normal 19.5% compound allocation from the fixed split.
    pub normal_compound_allocation: Amount,
    /// Dust retained from the independent fixed gross-yield split.
    pub split_dust: Amount,
    /// Retained dust from a validated conversion.
    pub conversion_dust: Amount,
    /// Retained dust from a validated net allocation.
    pub net_allocation_dust: Amount,
    /// Confirmed KIF amount compounded for a zero-active-guardian period.
    pub zero_active_kif_compound: Amount,
}

/// Checked result of increasing the protected-principal high-water mark.
///
/// The invariant is `new_high_water_mark = old_high_water_mark + increase`.
/// Both the component accumulation and the final addition are checked.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct HighWaterMarkUpdate {
    /// Exact checked sum of the six eligible components.
    pub increase: Amount,
    /// Old high-water mark plus [`Self::increase`].
    pub new_high_water_mark: Amount,
}

/// Active-guardian KIF allocation with an explicit collective carry.
///
/// The invariant is `credited_total + carry_next = kif_available`. Every
/// active guardian receives exactly `per_guardian`; no guardian receives the
/// division remainder preferentially.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ActiveGuardianKifAllocation {
    /// Current KIF allocation plus approved prior collective carry.
    pub kif_available: Amount,
    /// Already snapshotted active-guardian count in `1..=6`.
    pub active_guardians: u8,
    /// Equal, floored amount credited to each active guardian.
    pub per_guardian: Amount,
    /// Total credited across all active guardians.
    pub credited_total: Amount,
    /// Unassigned division remainder retained as collective KIF carry.
    pub carry_next: Amount,
}

/// Zero-active-guardian KIF allocation.
///
/// The invariant is `compound_from_kif + carry_next = kif_available`.
/// Division floors, so the extra unit of an odd amount remains in carry.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ZeroActiveGuardianKifAllocation {
    /// Current KIF allocation plus all approved prior collective carry.
    pub kif_available: Amount,
    /// Floored half permanently assigned to compound.
    pub compound_from_kif: Amount,
    /// Residual half, including any odd unit, retained as collective carry.
    pub carry_next: Amount,
}

/// KIF allocation selected by an already snapshotted count in `0..=6`.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum KifAllocation {
    /// Equal allocation for one or more active guardians.
    ActiveGuardians(ActiveGuardianKifAllocation),
    /// Fifty-percent compound and residual carry for zero active guardians.
    ZeroActiveGuardians(ZeroActiveGuardianKifAllocation),
}

/// Computes `floor(multiplicand * multiplier / denominator)` with checked math.
///
/// The product uses a checked `u128` intermediate. Division floors toward
/// zero, which is downward for unsigned amounts. The result is narrowed to
/// [`Amount`] with a checked conversion. A zero denominator is always an
/// explicit [`MathError::DivisionByZero`], including when a factor is zero.
///
/// # Examples
///
/// ```
/// use piv1_math::checked_mul_div_floor;
///
/// assert_eq!(checked_mul_div_floor(10, 10, 6), Ok(16));
/// ```
pub fn checked_mul_div_floor(
    multiplicand: Amount,
    multiplier: Amount,
    denominator: Amount,
) -> Result<Amount, MathError> {
    let quotient = checked_mul_div_floor_wide(
        u128::from(multiplicand),
        u128::from(multiplier),
        u128::from(denominator),
    )?;
    narrow_amount(quotient)
}

/// Computes `ceil(multiplicand * multiplier / denominator)` with checked math.
///
/// The product uses a checked `u128` intermediate. Ceiling is calculated from
/// quotient and remainder instead of adding `denominator - 1`, avoiding a
/// spurious intermediate overflow. The result is narrowed to [`Amount`] with
/// a checked conversion. A zero denominator is always explicit.
pub fn checked_mul_div_ceil(
    multiplicand: Amount,
    multiplier: Amount,
    denominator: Amount,
) -> Result<Amount, MathError> {
    let quotient = checked_mul_div_ceil_wide(
        u128::from(multiplicand),
        u128::from(multiplier),
        u128::from(denominator),
    )?;
    narrow_amount(quotient)
}

/// Calculates confirmed gross yield against an already validated historical value.
///
/// This implements `max(0, historical_value - high_water_mark)`. Recovery at
/// or below the high-water mark is not yield. The supplied high-water mark is
/// not modified and therefore cannot decrease through this calculation.
pub fn calculate_gross_yield(
    historical_value: Amount,
    high_water_mark: Amount,
) -> Amount {
    match historical_value.checked_sub(high_water_mark) {
        Some(gross_yield) => gross_yield,
        None => 0,
    }
}

/// Splits gross yield using the confirmed `5900/1950/1950/200` basis points.
///
/// Every named allocation is independently floored by
/// [`checked_mul_div_floor`]. Dust is subtracted only after checked allocation
/// accumulation. On success, allocations plus dust equal `gross_yield`
/// exactly, and outgoing amounts never exceed the input.
pub fn split_gross_yield(gross_yield: Amount) -> Result<GrossYieldSplit, MathError> {
    let htfp_reserve = checked_mul_div_floor(
        gross_yield,
        HTFP_RESERVE_BPS,
        BASIS_POINTS_DENOMINATOR,
    )?;
    let permanent_compound = checked_mul_div_floor(
        gross_yield,
        PERMANENT_COMPOUND_BPS,
        BASIS_POINTS_DENOMINATOR,
    )?;
    let team_owner_pool = checked_mul_div_floor(
        gross_yield,
        TEAM_OWNER_POOL_BPS,
        BASIS_POINTS_DENOMINATOR,
    )?;
    let kif = checked_mul_div_floor(gross_yield, KIF_BPS, BASIS_POINTS_DENOMINATOR)?;

    let allocated = checked_add_amount(htfp_reserve, permanent_compound)?;
    let allocated = checked_add_amount(allocated, team_owner_pool)?;
    let allocated = checked_add_amount(allocated, kif)?;
    let dust = checked_sub_amount(gross_yield, allocated)?;

    Ok(GrossYieldSplit {
        htfp_reserve,
        permanent_compound,
        team_owner_pool,
        kif,
        dust,
    })
}

/// Adds all confirmed pure numeric high-water-mark components with checks.
///
/// No component is rounded by this function. Every component addition and the
/// final increase of `old_high_water_mark` is checked. The function never
/// lowers the high-water mark and returns an explicit overflow error rather
/// than wrapping.
pub fn checked_increase_high_water_mark(
    old_high_water_mark: Amount,
    components: HighWaterMarkComponents,
) -> Result<HighWaterMarkUpdate, MathError> {
    let increase = checked_add_amount(0, components.contribution_value)?;
    let increase = checked_add_amount(increase, components.normal_compound_allocation)?;
    let increase = checked_add_amount(increase, components.split_dust)?;
    let increase = checked_add_amount(increase, components.conversion_dust)?;
    let increase = checked_add_amount(increase, components.net_allocation_dust)?;
    let increase = checked_add_amount(increase, components.zero_active_kif_compound)?;
    let new_high_water_mark = checked_add_amount(old_high_water_mark, increase)?;

    Ok(HighWaterMarkUpdate {
        increase,
        new_high_water_mark,
    })
}

/// Allocates current KIF plus approved prior carry for `0..=6` active guardians.
///
/// The available amount is added with checked arithmetic. Counts `1..=6`
/// receive an equal floored per-guardian allocation and collective remainder.
/// Count zero compounds `floor(kif_available / 2)` and carries the rest, so an
/// odd extra unit remains in carry. Passing the previous carry again in a later
/// call applies the confirmed repeated zero-active-period rule.
pub fn allocate_kif(
    current_kif_allocation: Amount,
    approved_prior_carry: Amount,
    active_guardians: u8,
) -> Result<KifAllocation, MathError> {
    if active_guardians > MAX_ACTIVE_GUARDIANS {
        return Err(MathError::InvalidActiveGuardianCount { active_guardians });
    }

    let kif_available = checked_add_amount(current_kif_allocation, approved_prior_carry)?;

    if active_guardians == 0 {
        let compound_from_kif = checked_mul_div_floor(kif_available, 1, 2)?;
        let carry_next = checked_sub_amount(kif_available, compound_from_kif)?;

        return Ok(KifAllocation::ZeroActiveGuardians(
            ZeroActiveGuardianKifAllocation {
                kif_available,
                compound_from_kif,
                carry_next,
            },
        ));
    }

    let guardian_count = Amount::from(active_guardians);
    let per_guardian = checked_mul_div_floor(kif_available, 1, guardian_count)?;
    let credited_total = checked_mul_amount(per_guardian, guardian_count)?;
    let carry_next = checked_sub_amount(kif_available, credited_total)?;

    Ok(KifAllocation::ActiveGuardians(
        ActiveGuardianKifAllocation {
            kif_available,
            active_guardians,
            per_guardian,
            credited_total,
            carry_next,
        },
    ))
}

fn checked_mul_div_floor_wide(
    multiplicand: u128,
    multiplier: u128,
    denominator: u128,
) -> Result<u128, MathError> {
    if denominator == 0 {
        return Err(MathError::DivisionByZero);
    }

    let product = multiplicand
        .checked_mul(multiplier)
        .ok_or(MathError::MultiplicationOverflow)?;
    Ok(product / denominator)
}

fn checked_mul_div_ceil_wide(
    multiplicand: u128,
    multiplier: u128,
    denominator: u128,
) -> Result<u128, MathError> {
    if denominator == 0 {
        return Err(MathError::DivisionByZero);
    }

    let product = multiplicand
        .checked_mul(multiplier)
        .ok_or(MathError::MultiplicationOverflow)?;
    let quotient = product / denominator;

    if product % denominator == 0 {
        Ok(quotient)
    } else {
        quotient
            .checked_add(1)
            .ok_or(MathError::AdditionOverflow)
    }
}

fn narrow_amount(value: u128) -> Result<Amount, MathError> {
    Amount::try_from(value).map_err(|_| MathError::NarrowingConversion)
}

fn checked_add_amount(left: Amount, right: Amount) -> Result<Amount, MathError> {
    left.checked_add(right).ok_or(MathError::AdditionOverflow)
}

fn checked_mul_amount(left: Amount, right: Amount) -> Result<Amount, MathError> {
    left.checked_mul(right)
        .ok_or(MathError::MultiplicationOverflow)
}

fn checked_sub_amount(left: Amount, right: Amount) -> Result<Amount, MathError> {
    left.checked_sub(right)
        .ok_or(MathError::SubtractionUnderflow)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn floor_handles_zero_numerator_and_zero_multiplicand() {
        assert_eq!(checked_mul_div_floor(0, Amount::MAX, 7), Ok(0));
        assert_eq!(checked_mul_div_floor(Amount::MAX, 0, 7), Ok(0));
    }

    #[test]
    fn ceil_handles_zero_numerator_and_zero_multiplicand() {
        assert_eq!(checked_mul_div_ceil(0, Amount::MAX, 7), Ok(0));
        assert_eq!(checked_mul_div_ceil(Amount::MAX, 0, 7), Ok(0));
    }

    #[test]
    fn multiply_divide_handles_denominator_one_and_exact_division() {
        assert_eq!(checked_mul_div_floor(123, 456, 1), Ok(56_088));
        assert_eq!(checked_mul_div_ceil(123, 456, 1), Ok(56_088));
        assert_eq!(checked_mul_div_floor(21, 10, 7), Ok(30));
        assert_eq!(checked_mul_div_ceil(21, 10, 7), Ok(30));
    }

    #[test]
    fn multiply_divide_rounds_non_exact_results_in_the_required_direction() {
        assert_eq!(checked_mul_div_floor(10, 10, 6), Ok(16));
        assert_eq!(checked_mul_div_ceil(10, 10, 6), Ok(17));
        assert_eq!(checked_mul_div_floor(1, 1, Amount::MAX), Ok(0));
        assert_eq!(checked_mul_div_ceil(1, 1, Amount::MAX), Ok(1));
    }

    #[test]
    fn multiply_divide_rejects_zero_denominator_even_for_zero_input() {
        assert_eq!(checked_mul_div_floor(1, 1, 0), Err(MathError::DivisionByZero));
        assert_eq!(checked_mul_div_ceil(1, 1, 0), Err(MathError::DivisionByZero));
        assert_eq!(checked_mul_div_floor(0, 1, 0), Err(MathError::DivisionByZero));
        assert_eq!(checked_mul_div_ceil(0, 1, 0), Err(MathError::DivisionByZero));
    }

    #[test]
    fn multiply_divide_supports_the_exact_amount_boundary() {
        assert_eq!(
            checked_mul_div_floor(Amount::MAX, 2, 2),
            Ok(Amount::MAX)
        );
        assert_eq!(
            checked_mul_div_ceil(Amount::MAX, 2, 2),
            Ok(Amount::MAX)
        );
        assert_eq!(
            checked_mul_div_floor(Amount::MAX, Amount::MAX, Amount::MAX),
            Ok(Amount::MAX)
        );
        assert_eq!(
            checked_mul_div_ceil(Amount::MAX, Amount::MAX, Amount::MAX),
            Ok(Amount::MAX)
        );
    }

    #[test]
    fn multiply_divide_rejects_failed_narrowing() {
        assert_eq!(
            checked_mul_div_floor(Amount::MAX, Amount::MAX, 1),
            Err(MathError::NarrowingConversion)
        );
        assert_eq!(
            checked_mul_div_ceil(Amount::MAX, Amount::MAX, 1),
            Err(MathError::NarrowingConversion)
        );
        assert_eq!(
            checked_mul_div_floor(Amount::MAX - 1, Amount::MAX - 1, Amount::MAX - 2),
            Ok(Amount::MAX)
        );
        assert_eq!(
            checked_mul_div_ceil(Amount::MAX - 1, Amount::MAX - 1, Amount::MAX - 2),
            Err(MathError::NarrowingConversion)
        );
    }

    #[test]
    fn wide_multiply_divide_rejects_intermediate_overflow() {
        assert_eq!(
            checked_mul_div_floor_wide(u128::MAX, 2, 1),
            Err(MathError::MultiplicationOverflow)
        );
        assert_eq!(
            checked_mul_div_ceil_wide(u128::MAX, 2, 1),
            Err(MathError::MultiplicationOverflow)
        );
    }

    #[test]
    fn checked_amount_operations_report_each_failure_class() {
        assert_eq!(
            checked_add_amount(Amount::MAX, 1),
            Err(MathError::AdditionOverflow)
        );
        assert_eq!(
            checked_mul_amount(Amount::MAX, 2),
            Err(MathError::MultiplicationOverflow)
        );
        assert_eq!(
            checked_sub_amount(0, 1),
            Err(MathError::SubtractionUnderflow)
        );
        assert_eq!(
            narrow_amount(u128::from(Amount::MAX) + 1),
            Err(MathError::NarrowingConversion)
        );
    }

    #[test]
    fn fixed_split_handles_zero_and_one_lamport() {
        assert_eq!(split_gross_yield(0), Ok(GrossYieldSplit::default()));
        assert_eq!(
            split_gross_yield(1),
            Ok(GrossYieldSplit {
                htfp_reserve: 0,
                permanent_compound: 0,
                team_owner_pool: 0,
                kif: 0,
                dust: 1,
            })
        );
    }

    #[test]
    fn fixed_split_handles_values_below_allocation_thresholds() {
        assert_eq!(
            split_gross_yield(5),
            Ok(GrossYieldSplit {
                htfp_reserve: 2,
                permanent_compound: 0,
                team_owner_pool: 0,
                kif: 0,
                dust: 3,
            })
        );
        assert_eq!(
            split_gross_yield(49),
            Ok(GrossYieldSplit {
                htfp_reserve: 28,
                permanent_compound: 9,
                team_owner_pool: 9,
                kif: 0,
                dust: 3,
            })
        );
        assert_eq!(
            split_gross_yield(50),
            Ok(GrossYieldSplit {
                htfp_reserve: 29,
                permanent_compound: 9,
                team_owner_pool: 9,
                kif: 1,
                dust: 2,
            })
        );
    }

    #[test]
    fn fixed_split_matches_exact_ten_thousand_lamport_weights() {
        assert_eq!(
            split_gross_yield(10_000),
            Ok(GrossYieldSplit {
                htfp_reserve: 5_900,
                permanent_compound: 1_950,
                team_owner_pool: 1_950,
                kif: 200,
                dust: 0,
            })
        );
    }

    #[test]
    fn fixed_split_retains_non_divisible_dust() {
        assert_eq!(
            split_gross_yield(12_345),
            Ok(GrossYieldSplit {
                htfp_reserve: 7_283,
                permanent_compound: 2_407,
                team_owner_pool: 2_407,
                kif: 246,
                dust: 2,
            })
        );
    }

    #[test]
    fn fixed_split_reconciles_large_and_boundary_values() -> Result<(), MathError> {
        for gross_yield in [
            0,
            1,
            2,
            5,
            6,
            49,
            50,
            9_999,
            10_000,
            10_001,
            Amount::MAX - 1,
            Amount::MAX,
        ] {
            let split = split_gross_yield(gross_yield)?;
            let total = u128::from(split.htfp_reserve)
                + u128::from(split.permanent_compound)
                + u128::from(split.team_owner_pool)
                + u128::from(split.kif)
                + u128::from(split.dust);
            let outgoing = u128::from(split.htfp_reserve)
                + u128::from(split.team_owner_pool)
                + u128::from(split.kif);

            assert_eq!(total, u128::from(gross_yield));
            assert!(outgoing <= u128::from(gross_yield));
            assert!(split.htfp_reserve <= gross_yield);
            assert!(split.permanent_compound <= gross_yield);
            assert!(split.team_owner_pool <= gross_yield);
            assert!(split.kif <= gross_yield);
            assert!(split.dust <= gross_yield);
        }

        Ok(())
    }

    #[test]
    fn fixed_split_matches_the_maximum_amount_boundary() {
        assert_eq!(
            split_gross_yield(Amount::MAX),
            Ok(GrossYieldSplit {
                htfp_reserve: 10_883_579_003_488_635_452,
                permanent_compound: 3_597_115_094_373_362_564,
                team_owner_pool: 3_597_115_094_373_362_564,
                kif: 368_934_881_474_191_032,
                dust: 3,
            })
        );
    }

    #[test]
    fn gross_yield_is_zero_below_or_at_the_high_water_mark() {
        assert_eq!(calculate_gross_yield(99, 100), 0);
        assert_eq!(calculate_gross_yield(100, 100), 0);
    }

    #[test]
    fn gross_yield_handles_one_lamport_and_normal_positive_yield() {
        assert_eq!(calculate_gross_yield(101, 100), 1);
        assert_eq!(calculate_gross_yield(1_500_000, 1_000_000), 500_000);
    }

    #[test]
    fn gross_yield_handles_amount_boundaries_without_lowering_the_hwm() {
        assert_eq!(calculate_gross_yield(Amount::MAX, 0), Amount::MAX);
        assert_eq!(calculate_gross_yield(Amount::MAX, Amount::MAX - 1), 1);
        assert_eq!(calculate_gross_yield(0, Amount::MAX), 0);
        assert_eq!(calculate_gross_yield(Amount::MAX - 1, Amount::MAX), 0);
    }

    #[test]
    fn hwm_increase_handles_all_zero_components() {
        assert_eq!(
            checked_increase_high_water_mark(42, HighWaterMarkComponents::default()),
            Ok(HighWaterMarkUpdate {
                increase: 0,
                new_high_water_mark: 42,
            })
        );
    }

    #[test]
    fn hwm_increase_accepts_each_component_independently() {
        let component_cases = [
            HighWaterMarkComponents {
                contribution_value: 7,
                ..HighWaterMarkComponents::default()
            },
            HighWaterMarkComponents {
                normal_compound_allocation: 7,
                ..HighWaterMarkComponents::default()
            },
            HighWaterMarkComponents {
                split_dust: 7,
                ..HighWaterMarkComponents::default()
            },
            HighWaterMarkComponents {
                conversion_dust: 7,
                ..HighWaterMarkComponents::default()
            },
            HighWaterMarkComponents {
                net_allocation_dust: 7,
                ..HighWaterMarkComponents::default()
            },
            HighWaterMarkComponents {
                zero_active_kif_compound: 7,
                ..HighWaterMarkComponents::default()
            },
        ];

        for components in component_cases {
            assert_eq!(
                checked_increase_high_water_mark(10, components),
                Ok(HighWaterMarkUpdate {
                    increase: 7,
                    new_high_water_mark: 17,
                })
            );
        }
    }

    #[test]
    fn hwm_increase_reconciles_combined_components() {
        let components = HighWaterMarkComponents {
            contribution_value: 1,
            normal_compound_allocation: 2,
            split_dust: 3,
            conversion_dust: 4,
            net_allocation_dust: 5,
            zero_active_kif_compound: 6,
        };

        assert_eq!(
            checked_increase_high_water_mark(100, components),
            Ok(HighWaterMarkUpdate {
                increase: 21,
                new_high_water_mark: 121,
            })
        );
    }

    #[test]
    fn hwm_increase_accepts_maximum_exact_accumulation() {
        let components = HighWaterMarkComponents {
            contribution_value: 1,
            normal_compound_allocation: 2,
            split_dust: 3,
            conversion_dust: 4,
            net_allocation_dust: 5,
            zero_active_kif_compound: 6,
        };

        assert_eq!(
            checked_increase_high_water_mark(Amount::MAX - 21, components),
            Ok(HighWaterMarkUpdate {
                increase: 21,
                new_high_water_mark: Amount::MAX,
            })
        );
    }

    #[test]
    fn hwm_increase_rejects_component_and_final_overflow() {
        assert_eq!(
            checked_increase_high_water_mark(
                0,
                HighWaterMarkComponents {
                    contribution_value: Amount::MAX,
                    normal_compound_allocation: 1,
                    ..HighWaterMarkComponents::default()
                },
            ),
            Err(MathError::AdditionOverflow)
        );
        assert_eq!(
            checked_increase_high_water_mark(
                Amount::MAX,
                HighWaterMarkComponents {
                    contribution_value: 1,
                    ..HighWaterMarkComponents::default()
                },
            ),
            Err(MathError::AdditionOverflow)
        );
    }

    #[test]
    fn kif_zero_available_reconciles_for_zero_and_active_counts() {
        assert_eq!(
            allocate_kif(0, 0, 0),
            Ok(KifAllocation::ZeroActiveGuardians(
                ZeroActiveGuardianKifAllocation {
                    kif_available: 0,
                    compound_from_kif: 0,
                    carry_next: 0,
                }
            ))
        );
        assert_eq!(
            allocate_kif(0, 0, 6),
            Ok(KifAllocation::ActiveGuardians(
                ActiveGuardianKifAllocation {
                    kif_available: 0,
                    active_guardians: 6,
                    per_guardian: 0,
                    credited_total: 0,
                    carry_next: 0,
                }
            ))
        );
    }

    #[test]
    fn kif_zero_active_splits_even_and_odd_available_amounts() {
        assert_eq!(
            allocate_kif(8, 2, 0),
            Ok(KifAllocation::ZeroActiveGuardians(
                ZeroActiveGuardianKifAllocation {
                    kif_available: 10,
                    compound_from_kif: 5,
                    carry_next: 5,
                }
            ))
        );
        assert_eq!(
            allocate_kif(8, 3, 0),
            Ok(KifAllocation::ZeroActiveGuardians(
                ZeroActiveGuardianKifAllocation {
                    kif_available: 11,
                    compound_from_kif: 5,
                    carry_next: 6,
                }
            ))
        );
    }

    #[test]
    fn kif_zero_active_reapplies_the_full_carry_each_period() -> Result<(), MathError> {
        let first = allocate_kif(5, 0, 0)?;
        assert_eq!(
            first,
            KifAllocation::ZeroActiveGuardians(ZeroActiveGuardianKifAllocation {
                kif_available: 5,
                compound_from_kif: 2,
                carry_next: 3,
            })
        );

        let second = allocate_kif(0, 3, 0)?;
        assert_eq!(
            second,
            KifAllocation::ZeroActiveGuardians(ZeroActiveGuardianKifAllocation {
                kif_available: 3,
                compound_from_kif: 1,
                carry_next: 2,
            })
        );

        let third = allocate_kif(0, 2, 0)?;
        assert_eq!(
            third,
            KifAllocation::ZeroActiveGuardians(ZeroActiveGuardianKifAllocation {
                kif_available: 2,
                compound_from_kif: 1,
                carry_next: 1,
            })
        );

        let fourth = allocate_kif(0, 1, 0)?;
        assert_eq!(
            fourth,
            KifAllocation::ZeroActiveGuardians(ZeroActiveGuardianKifAllocation {
                kif_available: 1,
                compound_from_kif: 0,
                carry_next: 1,
            })
        );

        Ok(())
    }

    #[test]
    fn kif_active_counts_one_through_six_allocate_equally() -> Result<(), MathError> {
        let expected_per_guardian = [59, 29, 19, 14, 11, 9];

        for (active_guardians, per_guardian) in
            (1..=MAX_ACTIVE_GUARDIANS).zip(expected_per_guardian)
        {
            let allocation = allocate_kif(31, 28, active_guardians)?;
            let expected_credited = per_guardian * Amount::from(active_guardians);

            assert_eq!(
                allocation,
                KifAllocation::ActiveGuardians(ActiveGuardianKifAllocation {
                    kif_available: 59,
                    active_guardians,
                    per_guardian,
                    credited_total: expected_credited,
                    carry_next: 59 - expected_credited,
                })
            );
        }

        Ok(())
    }

    #[test]
    fn kif_active_handles_exact_and_non_divisible_allocations() {
        assert_eq!(
            allocate_kif(12, 0, 3),
            Ok(KifAllocation::ActiveGuardians(
                ActiveGuardianKifAllocation {
                    kif_available: 12,
                    active_guardians: 3,
                    per_guardian: 4,
                    credited_total: 12,
                    carry_next: 0,
                }
            ))
        );
        assert_eq!(
            allocate_kif(10, 1, 4),
            Ok(KifAllocation::ActiveGuardians(
                ActiveGuardianKifAllocation {
                    kif_available: 11,
                    active_guardians: 4,
                    per_guardian: 2,
                    credited_total: 8,
                    carry_next: 3,
                }
            ))
        );
    }

    #[test]
    fn kif_rejects_counts_above_six() {
        assert_eq!(
            allocate_kif(10, 20, 7),
            Err(MathError::InvalidActiveGuardianCount {
                active_guardians: 7,
            })
        );
        assert_eq!(
            allocate_kif(10, 20, u8::MAX),
            Err(MathError::InvalidActiveGuardianCount {
                active_guardians: u8::MAX,
            })
        );
    }

    #[test]
    fn kif_rejects_available_amount_overflow() {
        assert_eq!(
            allocate_kif(Amount::MAX, 1, 0),
            Err(MathError::AdditionOverflow)
        );
        assert_eq!(
            allocate_kif(1, Amount::MAX, 6),
            Err(MathError::AdditionOverflow)
        );
    }

    #[test]
    fn kif_zero_active_handles_the_maximum_odd_amount() {
        assert_eq!(
            allocate_kif(Amount::MAX, 0, 0),
            Ok(KifAllocation::ZeroActiveGuardians(
                ZeroActiveGuardianKifAllocation {
                    kif_available: Amount::MAX,
                    compound_from_kif: 9_223_372_036_854_775_807,
                    carry_next: 9_223_372_036_854_775_808,
                }
            ))
        );
    }

    #[test]
    fn kif_allocations_reconcile_at_amount_boundaries() -> Result<(), MathError> {
        for active_guardians in 0..=MAX_ACTIVE_GUARDIANS {
            let allocation = allocate_kif(Amount::MAX - 1, 1, active_guardians)?;

            match allocation {
                KifAllocation::ActiveGuardians(active) => {
                    assert_eq!(
                        u128::from(active.credited_total) + u128::from(active.carry_next),
                        u128::from(active.kif_available)
                    );
                    assert_eq!(
                        active.credited_total,
                        active.per_guardian * Amount::from(active.active_guardians)
                    );
                }
                KifAllocation::ZeroActiveGuardians(zero_active) => {
                    assert_eq!(
                        u128::from(zero_active.compound_from_kif)
                            + u128::from(zero_active.carry_next),
                        u128::from(zero_active.kif_available)
                    );
                }
            }
        }

        Ok(())
    }
}
