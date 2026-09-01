//! Pure checked timing helpers over future trusted Solana Clock timestamps.

use crate::{
    constants::{
        INSUFFICIENT_RETRY_COOLDOWN_SECONDS, KIF_PERIOD_SECONDS,
        MINIMUM_DISTRIBUTION_INTERVAL_SECONDS,
    },
    errors::{Piv1Error, Piv1Result},
};

/// Checked half-open KIF period derived from a configured anchor.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct KifPeriod {
    /// Monotonic period identifier beginning at zero at the configured anchor.
    pub id: u64,
    /// Inclusive half-open period start timestamp.
    pub start_timestamp: i64,
    /// Exclusive half-open period end timestamp.
    pub end_timestamp: i64,
}

/// Validates the confirmed minimum interval after a successful preparation.
pub fn validate_preparation_interval(
    last_successful_preparation_at: Option<i64>,
    now: i64,
) -> Piv1Result<()> {
    validate_optional_elapsed(
        last_successful_preparation_at,
        now,
        MINIMUM_DISTRIBUTION_INTERVAL_SECONDS,
        Piv1Error::PreparationIntervalNotElapsed,
    )
}

/// Validates the retry interval after a valid technically insufficient result.
pub fn validate_insufficient_retry(
    last_valid_insufficient_attempt_at: Option<i64>,
    now: i64,
) -> Piv1Result<()> {
    validate_optional_elapsed(
        last_valid_insufficient_attempt_at,
        now,
        INSUFFICIENT_RETRY_COOLDOWN_SECONDS,
        Piv1Error::InsufficientAttemptCooldownActive,
    )
}

/// Derives the fixed half-open KIF period containing `timestamp`.
pub fn derive_kif_period(anchor_timestamp: i64, timestamp: i64) -> Piv1Result<KifPeriod> {
    if timestamp < anchor_timestamp {
        return Err(Piv1Error::TimestampRegression);
    }

    let elapsed = timestamp
        .checked_sub(anchor_timestamp)
        .ok_or(Piv1Error::ArithmeticOverflow)?;
    let period_index = elapsed
        .checked_div(KIF_PERIOD_SECONDS)
        .ok_or(Piv1Error::InvalidTimestamp)?;
    let period_id = u64::try_from(period_index).map_err(|_| Piv1Error::ArithmeticOverflow)?;
    let period_offset = period_index
        .checked_mul(KIF_PERIOD_SECONDS)
        .ok_or(Piv1Error::ArithmeticOverflow)?;
    let start_timestamp = anchor_timestamp
        .checked_add(period_offset)
        .ok_or(Piv1Error::ArithmeticOverflow)?;
    let end_timestamp = start_timestamp
        .checked_add(KIF_PERIOD_SECONDS)
        .ok_or(Piv1Error::ArithmeticOverflow)?;

    if timestamp < start_timestamp || timestamp >= end_timestamp {
        return Err(Piv1Error::InvalidTimestamp);
    }

    Ok(KifPeriod {
        id: period_id,
        start_timestamp,
        end_timestamp,
    })
}

fn validate_optional_elapsed(
    previous: Option<i64>,
    now: i64,
    required_seconds: i64,
    not_elapsed_error: Piv1Error,
) -> Piv1Result<()> {
    if required_seconds <= 0 {
        return Err(Piv1Error::InvalidTimingConfiguration);
    }

    let Some(previous) = previous else {
        return Ok(());
    };

    if now < previous {
        return Err(Piv1Error::TimestampRegression);
    }

    let elapsed = now
        .checked_sub(previous)
        .ok_or(Piv1Error::ArithmeticOverflow)?;
    if elapsed < required_seconds {
        return Err(not_elapsed_error);
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn preparation_interval_accepts_first_use_and_exact_boundary() {
        let previous = 1_000_000;

        assert_eq!(MINIMUM_DISTRIBUTION_INTERVAL_SECONDS, 864_000);
        assert_eq!(validate_preparation_interval(None, previous), Ok(()));
        assert_eq!(
            validate_preparation_interval(
                Some(previous),
                previous + MINIMUM_DISTRIBUTION_INTERVAL_SECONDS,
            ),
            Ok(())
        );
    }

    #[test]
    fn preparation_interval_rejects_pre_boundary_and_regression() {
        let previous = 1_000_000;

        assert_eq!(
            validate_preparation_interval(
                Some(previous),
                previous + MINIMUM_DISTRIBUTION_INTERVAL_SECONDS - 1,
            ),
            Err(Piv1Error::PreparationIntervalNotElapsed)
        );
        assert_eq!(
            validate_preparation_interval(Some(previous), previous - 1),
            Err(Piv1Error::TimestampRegression)
        );
    }

    #[test]
    fn insufficient_retry_accepts_first_use_and_exact_boundary() {
        let previous = 2_000_000;

        assert_eq!(INSUFFICIENT_RETRY_COOLDOWN_SECONDS, 86_400);
        assert_eq!(validate_insufficient_retry(None, previous), Ok(()));
        assert_eq!(
            validate_insufficient_retry(
                Some(previous),
                previous + INSUFFICIENT_RETRY_COOLDOWN_SECONDS,
            ),
            Ok(())
        );
    }

    #[test]
    fn insufficient_retry_rejects_pre_boundary_and_regression() {
        let previous = 2_000_000;

        assert_eq!(
            validate_insufficient_retry(
                Some(previous),
                previous + INSUFFICIENT_RETRY_COOLDOWN_SECONDS - 1,
            ),
            Err(Piv1Error::InsufficientAttemptCooldownActive)
        );
        assert_eq!(
            validate_insufficient_retry(Some(previous), previous - 1),
            Err(Piv1Error::TimestampRegression)
        );
    }

    #[test]
    fn elapsed_checks_report_signed_overflow() {
        assert_eq!(
            validate_preparation_interval(Some(i64::MIN), i64::MAX),
            Err(Piv1Error::ArithmeticOverflow)
        );
        assert_eq!(
            validate_insufficient_retry(Some(i64::MIN), i64::MAX),
            Err(Piv1Error::ArithmeticOverflow)
        );
    }

    #[test]
    fn kif_period_boundaries_are_half_open() {
        let anchor = 1_700_000_000;
        let first_end = anchor + KIF_PERIOD_SECONDS;
        let second_end = first_end + KIF_PERIOD_SECONDS;

        assert_eq!(KIF_PERIOD_SECONDS, 2_592_000);
        assert_eq!(
            derive_kif_period(anchor, anchor),
            Ok(KifPeriod {
                id: 0,
                start_timestamp: anchor,
                end_timestamp: first_end,
            })
        );
        assert_eq!(
            derive_kif_period(anchor, first_end - 1),
            Ok(KifPeriod {
                id: 0,
                start_timestamp: anchor,
                end_timestamp: first_end,
            })
        );
        assert_eq!(
            derive_kif_period(anchor, first_end),
            Ok(KifPeriod {
                id: 1,
                start_timestamp: first_end,
                end_timestamp: second_end,
            })
        );
    }

    #[test]
    fn kif_period_rejects_pre_anchor_timestamp() {
        let anchor = 1_700_000_000;

        assert_eq!(
            derive_kif_period(anchor, anchor - 1),
            Err(Piv1Error::TimestampRegression)
        );
    }

    #[test]
    fn kif_period_reports_checked_overflow_near_i64_limits() {
        assert_eq!(
            derive_kif_period(i64::MIN, i64::MAX),
            Err(Piv1Error::ArithmeticOverflow)
        );

        let anchor_with_unrepresentable_end = i64::MAX - KIF_PERIOD_SECONDS + 1;
        assert_eq!(
            derive_kif_period(
                anchor_with_unrepresentable_end,
                anchor_with_unrepresentable_end,
            ),
            Err(Piv1Error::ArithmeticOverflow)
        );
    }
}
