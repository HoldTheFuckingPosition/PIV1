//! Pure contribution-intake and pending-custody reconciliation.
//!
//! The numeric inputs in this module are observations, not inherently trusted
//! caller claims. Future handlers must derive them from the fixed,
//! program-controlled pending vaults after validating address, owner, mint,
//! token authority, account data, and the applicable non-economic SOL floor.
//! This module performs no transfer, CPI, account decoding, event emission, or
//! pending-principal integration.

use crate::{
    errors::{Piv1Error, Piv1Result},
    state::{ActiveDistribution, PivConfig},
};

/// Before/after native custody facts for one explicit contribution.
///
/// The same handler-validated non-economic floor applies to both observations.
/// It may represent rent or another validated custody-only balance and is
/// excluded before the economic increase is derived. No amount is hard-coded.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct SolCustodyObservation {
    pub vault_lamports_before: u64,
    pub vault_lamports_after: u64,
    pub non_economic_floor_lamports: u64,
}

/// Before/after token-amount facts for one explicit JitoSOL contribution.
///
/// These fields are decoded token units only. Lamports funding the token
/// account itself are outside this observation and cannot become JitoSOL value.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct JitoSolCustodyObservation {
    pub token_units_before: u64,
    pub token_units_after: u64,
}

/// Current physical facts for idempotent reconciliation of both pending vaults.
///
/// Future handlers must derive all three values from the fixed pending vaults;
/// naming them as observations does not confer trust or account validation.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct PendingCustodyObservation {
    pub pending_sol_vault_lamports: u64,
    pub pending_sol_non_economic_floor_lamports: u64,
    pub pending_jitosol_token_units: u64,
}

/// Deterministic result after one explicit contribution is recorded.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ExplicitContributionRecord {
    pub observed_increase: u64,
    pub accounted_pending_after: u64,
}

/// Deterministic result of reconciling both pending custody categories.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct PendingReconciliationResult {
    pub newly_accounted_sol_lamports: u64,
    pub newly_accounted_jitosol_units: u64,
    pub accounted_pending_sol_lamports_after: u64,
    pub accounted_pending_jitosol_units_after: u64,
}

impl PendingReconciliationResult {
    /// True when both current physical economic balances were already tracked.
    pub const fn is_no_change(self) -> bool {
        self.newly_accounted_sol_lamports == 0
            && self.newly_accounted_jitosol_units == 0
    }
}

/// Records an explicit positive native-SOL contribution from exact custody
/// balance observations.
///
/// Pause is deliberately not a gate in this accounting layer. Incoming value
/// may already have reached custody and must remain identifiable as pending.
/// Whether a future explicit transfer handler itself is callable while paused
/// remains a separate provisional handler policy.
pub fn record_explicit_sol_contribution(
    config: &mut PivConfig,
    active_distribution: &ActiveDistribution,
    expected_contribution_lamports: u64,
    observation: SolCustodyObservation,
) -> Piv1Result<ExplicitContributionRecord> {
    config.validate_initialized()?;
    active_distribution.validate()?;
    if expected_contribution_lamports == 0 {
        return Err(Piv1Error::ZeroContribution);
    }

    let spendable_before = spendable_sol_lamports(
        observation.vault_lamports_before,
        observation.non_economic_floor_lamports,
    )?;
    let spendable_after = spendable_sol_lamports(
        observation.vault_lamports_after,
        observation.non_economic_floor_lamports,
    )?;
    let observed_increase = spendable_after
        .checked_sub(spendable_before)
        .ok_or(Piv1Error::CustodyBalanceDecreased)?;
    if observed_increase != expected_contribution_lamports {
        return Err(Piv1Error::ContributionObservationMismatch);
    }

    let accounted_pending_after = config
        .accounted_pending_sol_lamports
        .checked_add(observed_increase)
        .ok_or(Piv1Error::ArithmeticOverflow)?;
    if config.accounted_pending_sol_lamports > spendable_before
        || accounted_pending_after > spendable_after
    {
        return Err(Piv1Error::PendingCustodyDeficit);
    }
    let mut next_config = config.clone();
    next_config.accounted_pending_sol_lamports = accounted_pending_after;
    next_config.validate_initialized()?;

    *config = next_config;
    Ok(ExplicitContributionRecord {
        observed_increase,
        accounted_pending_after,
    })
}

/// Records an explicit positive JitoSOL contribution from exact decoded token
/// amount observations.
///
/// The function changes only the pending JitoSOL ledger. Token-account rent is
/// absent from the observation and cannot be counted as token value.
pub fn record_explicit_jitosol_contribution(
    config: &mut PivConfig,
    active_distribution: &ActiveDistribution,
    expected_contribution_units: u64,
    observation: JitoSolCustodyObservation,
) -> Piv1Result<ExplicitContributionRecord> {
    config.validate_initialized()?;
    active_distribution.validate()?;
    if expected_contribution_units == 0 {
        return Err(Piv1Error::ZeroContribution);
    }

    let observed_increase = observation
        .token_units_after
        .checked_sub(observation.token_units_before)
        .ok_or(Piv1Error::CustodyBalanceDecreased)?;
    if observed_increase != expected_contribution_units {
        return Err(Piv1Error::ContributionObservationMismatch);
    }

    let accounted_pending_after = config
        .accounted_pending_jitosol_units
        .checked_add(observed_increase)
        .ok_or(Piv1Error::ArithmeticOverflow)?;
    if config.accounted_pending_jitosol_units > observation.token_units_before
        || accounted_pending_after > observation.token_units_after
    {
        return Err(Piv1Error::PendingCustodyDeficit);
    }
    let mut next_config = config.clone();
    next_config.accounted_pending_jitosol_units = accounted_pending_after;
    next_config.validate_initialized()?;

    *config = next_config;
    Ok(ExplicitContributionRecord {
        observed_increase,
        accounted_pending_after,
    })
}

/// Reconciles unexplained positive balances in both dedicated pending vaults.
///
/// Reconciliation is idempotent: each ledger advances to its observed physical
/// economic balance, and the same observation immediately returns no change.
/// A deficit in either asset rejects the combined operation before either
/// ledger commits. No historical value, HWM, cumulative contribution value,
/// active-round obligation, or KIF liability is changed.
pub fn reconcile_pending_contributions(
    config: &mut PivConfig,
    active_distribution: &ActiveDistribution,
    observation: PendingCustodyObservation,
) -> Piv1Result<PendingReconciliationResult> {
    config.validate_initialized()?;
    active_distribution.validate()?;

    let current_spendable_sol = spendable_sol_lamports(
        observation.pending_sol_vault_lamports,
        observation.pending_sol_non_economic_floor_lamports,
    )?;
    let newly_accounted_sol_lamports = current_spendable_sol
        .checked_sub(config.accounted_pending_sol_lamports)
        .ok_or(Piv1Error::PendingCustodyDeficit)?;
    let newly_accounted_jitosol_units = observation
        .pending_jitosol_token_units
        .checked_sub(config.accounted_pending_jitosol_units)
        .ok_or(Piv1Error::PendingCustodyDeficit)?;

    let accounted_pending_sol_lamports_after = config
        .accounted_pending_sol_lamports
        .checked_add(newly_accounted_sol_lamports)
        .ok_or(Piv1Error::ArithmeticOverflow)?;
    let accounted_pending_jitosol_units_after = config
        .accounted_pending_jitosol_units
        .checked_add(newly_accounted_jitosol_units)
        .ok_or(Piv1Error::ArithmeticOverflow)?;
    if accounted_pending_sol_lamports_after != current_spendable_sol
        || accounted_pending_jitosol_units_after
            != observation.pending_jitosol_token_units
    {
        return Err(Piv1Error::ArithmeticOverflow);
    }

    let mut next_config = config.clone();
    next_config.accounted_pending_sol_lamports =
        accounted_pending_sol_lamports_after;
    next_config.accounted_pending_jitosol_units =
        accounted_pending_jitosol_units_after;
    next_config.validate_initialized()?;

    *config = next_config;
    Ok(PendingReconciliationResult {
        newly_accounted_sol_lamports,
        newly_accounted_jitosol_units,
        accounted_pending_sol_lamports_after,
        accounted_pending_jitosol_units_after,
    })
}

fn spendable_sol_lamports(
    total_vault_lamports: u64,
    non_economic_floor_lamports: u64,
) -> Piv1Result<u64> {
    total_vault_lamports
        .checked_sub(non_economic_floor_lamports)
        .ok_or(Piv1Error::InvalidCustodyObservation)
}
