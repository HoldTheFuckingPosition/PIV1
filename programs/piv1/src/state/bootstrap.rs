//! Initial pending-to-principal integration, before any economic history.
//!
//! This pure boundary creates no distribution and performs no transfer. Future
//! handlers must authenticate the Config/header, current pool and fixed custody
//! accounts, then couple these exact observations to atomic same-asset transfers.
//! SOL remains in PrincipalSolQueue; protected staking is a separate boundary.

use crate::{
    errors::{Piv1Error, Piv1Result},
    integrations::PoolSnapshot,
    state::{
        reconciliation::{
            economic_custody_surplus, observed_token_book_value,
            validate_custody_state_binding, EconomicCustodyObservation,
            EconomicVaultAmounts,
        },
        ActiveDistribution, DistributionLifecycle, PivConfig,
    },
};

/// Initial holdings and conservative contribution value actually integrated.
/// This is an ordinary result, not serialized state or a distribution snapshot.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct InitialContributionBootstrap {
    pub integrated_sol_lamports: u64,
    pub integrated_jitosol_units: u64,
    pub contribution_value_lamports: u64,
}

/// Establishes initial principal from all recognized pending assets exactly once.
///
/// All economic history must be zero, even if earlier token holdings would now
/// have zero book value. Positive token units with zero floored value are valid
/// initial principal; the resulting historical units still prevent replay.
/// The complete Idle header and configured next sequence are preserved. This
/// function does not apply to subsequent Idle, insufficient or no-yield attempts.
/// No caller-selected contribution value, historical amount or HWM is accepted.
pub fn bootstrap_initial_contributions(
    config: &mut PivConfig,
    round: &ActiveDistribution,
    pool: PoolSnapshot,
    before: EconomicCustodyObservation,
    after: EconomicCustodyObservation,
) -> Piv1Result<InitialContributionBootstrap> {
    config.ensure_unpaused()?;
    validate_custody_state_binding(config, round)?;
    if round.bump != config.bumps.active_distribution {
        return Err(Piv1Error::InvalidAccountPda);
    }
    if round.lifecycle != DistributionLifecycle::Idle {
        return Err(Piv1Error::InvalidLifecycle);
    }
    if round.last_completed.is_some()
        || config.last_successful_preparation_at.is_some()
        || config.last_valid_insufficient_attempt_at.is_some()
        || [
            config.protected_principal_hwm_lamports,
            config.accounted_historical_sol_lamports,
            config.accounted_historical_jitosol_units,
            config.next_cycle_yield_lamports,
            config.kif_claim_liability_lamports,
            config.collective_kif_carry_lamports,
            config.cumulative_contribution_value_lamports,
            config.cumulative_gross_yield_lamports,
            config.cumulative_htfp_paid_lamports,
            config.cumulative_team_owner_paid_lamports,
            config.cumulative_kif_credited_lamports,
            config.cumulative_kif_claimed_lamports,
            config.cumulative_permanent_compound_lamports,
            config.cumulative_retained_dust_lamports,
            config.cumulative_zero_active_kif_compound_lamports,
            config.cumulative_cooldown_yield_recorded_lamports,
        ].iter().any(|&amount| amount != 0)
    {
        return Err(Piv1Error::InvalidBootstrapState);
    }
    let sol = config.accounted_pending_sol_lamports;
    let tokens = config.accounted_pending_jitosol_units;
    if sol == 0 && tokens == 0 {
        return Err(Piv1Error::ZeroContribution);
    }
    if economic_custody_surplus(config, round, before)? != EconomicVaultAmounts::default() {
        return Err(Piv1Error::InvalidCustodyObservation);
    }
    let expected_after = EconomicVaultAmounts {
        principal_sol_lamports: sol,
        principal_jitosol_units: tokens,
        ..EconomicVaultAmounts::default()
    };
    if !before.same_floors(after) || after.amounts()? != expected_after {
        return Err(Piv1Error::ContributionObservationMismatch);
    }
    let contribution_value_lamports = sol
        .checked_add(observed_token_book_value(tokens, pool)?)
        .ok_or(Piv1Error::ArithmeticOverflow)?;
    let mut next = config.clone();
    next.accounted_pending_sol_lamports = 0;
    next.accounted_pending_jitosol_units = 0;
    next.accounted_historical_sol_lamports = sol;
    next.accounted_historical_jitosol_units = tokens;
    next.checked_increase_protected_principal_hwm(contribution_value_lamports)?;
    next.cumulative_contribution_value_lamports = next.cumulative_contribution_value_lamports
        .checked_add(contribution_value_lamports)
        .ok_or(Piv1Error::ArithmeticOverflow)?;
    if economic_custody_surplus(&next, round, after)? != EconomicVaultAmounts::default() {
        return Err(Piv1Error::CumulativeReconciliationMismatch);
    }
    *config = next;
    Ok(InitialContributionBootstrap {
        integrated_sol_lamports: sol,
        integrated_jitosol_units: tokens,
        contribution_value_lamports,
    })
}
