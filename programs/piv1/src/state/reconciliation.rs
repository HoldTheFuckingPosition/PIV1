//! Phase-dependent economic custody obligations and exact normalization facts.
//!
//! These are pure derivations over observations, not authenticated accounts or
//! transfer receipts. An active offset is valid only when the surrounding
//! operation committed the corresponding custody movements with the transition.
//! The host composition demonstrates that coupling; future handlers must enforce
//! it using fixed accounts and exact atomic transfers. No serialized state is added.

use crate::{
    errors::{Piv1Error, Piv1Result},
    integrations::PoolSnapshot,
    state::{
        reconcile_pending_contributions, ActiveDistribution, DistributionLifecycle,
        PendingCustodyObservation, PendingIntegrationInput, PendingReconciliationResult,
        PivConfig,
    },
};

/// Total native balance and the unchanged, externally validated rent floor.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct SolVaultBalance {
    pub lamports: u64,
    pub non_economic_floor_lamports: u64,
}

impl SolVaultBalance {
    pub fn economic_lamports(self) -> Piv1Result<u64> {
        self.lamports.checked_sub(self.non_economic_floor_lamports)
            .ok_or(Piv1Error::InvalidCustodyObservation)
    }
}

/// Fixed economic vault observations. Token-account lamports are not token units.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct EconomicCustodyObservation {
    pub pending_sol: SolVaultBalance,
    pub principal_sol: SolVaultBalance,
    pub distribution_escrow: SolVaultBalance,
    pub kif_sol: SolVaultBalance,
    pub pending_jitosol_units: u64,
    pub principal_jitosol_units: u64,
}

impl EconomicCustodyObservation {
    pub fn amounts(self) -> Piv1Result<EconomicVaultAmounts> {
        Ok(EconomicVaultAmounts {
            pending_sol_lamports: self.pending_sol.economic_lamports()?,
            principal_sol_lamports: self.principal_sol.economic_lamports()?,
            escrow_sol_lamports: self.distribution_escrow.economic_lamports()?,
            kif_sol_lamports: self.kif_sol.economic_lamports()?,
            pending_jitosol_units: self.pending_jitosol_units,
            principal_jitosol_units: self.principal_jitosol_units,
        })
    }

    pub fn pending(self) -> PendingCustodyObservation {
        PendingCustodyObservation {
            pending_sol_vault_lamports: self.pending_sol.lamports,
            pending_sol_non_economic_floor_lamports:
                self.pending_sol.non_economic_floor_lamports,
            pending_jitosol_token_units: self.pending_jitosol_units,
        }
    }

    pub(crate) fn same_floors(self, other: Self) -> bool {
        self.pending_sol.non_economic_floor_lamports
            == other.pending_sol.non_economic_floor_lamports
            && self.principal_sol.non_economic_floor_lamports
                == other.principal_sol.non_economic_floor_lamports
            && self.distribution_escrow.non_economic_floor_lamports
                == other.distribution_escrow.non_economic_floor_lamports
            && self.kif_sol.non_economic_floor_lamports
                == other.kif_sol.non_economic_floor_lamports
    }
}

/// Bounded amounts, with independent native and token dimensions.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct EconomicVaultAmounts {
    pub pending_sol_lamports: u64,
    pub principal_sol_lamports: u64,
    pub escrow_sol_lamports: u64,
    pub kif_sol_lamports: u64,
    pub pending_jitosol_units: u64,
    pub principal_jitosol_units: u64,
}

/// Missing evidence is not a zero surplus or authority to sweep reserve custody.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum OperationalSurplusAssessment {
    UnsupportedFundingBaseline,
}

/// Validate the observed floor, but do not infer reserve excess from its balance.
pub fn assess_operational_surplus(
    observation: SolVaultBalance,
) -> Piv1Result<OperationalSurplusAssessment> {
    observation.economic_lamports()?;
    Ok(OperationalSurplusAssessment::UnsupportedFundingBaseline)
}

/// Cross-object facts required before using any committed active-round offset.
///
/// This validates derivable state relationships, not provenance. Callers must
/// establish custody movement/state coupling independently. Recipient/registry
/// rotation authentication belongs to future handlers and is not inferred here.
pub fn validate_custody_state_binding(
    config: &PivConfig,
    round: &ActiveDistribution,
) -> Piv1Result<()> {
    config.validate_initialized()?;
    round.validate()?;
    if round.lifecycle == DistributionLifecycle::Idle {
        if let Some(summary) = round.last_completed {
            if config.next_distribution_sequence != add(summary.sequence, 1)? {
                return Err(Piv1Error::SequenceMismatch);
            }
        }
        return Ok(());
    }
    if config.next_distribution_sequence != add(round.active_sequence, 1)?
        || round.last_completed.is_some_and(|s| round.active_sequence <= s.sequence)
    {
        return Err(Piv1Error::SequenceMismatch);
    }
    if config.last_successful_preparation_at != Some(round.prepared_at)
        || config.accounted_historical_jitosol_units != round.historical_jitosol_units
        || config.accounted_historical_sol_lamports != round.historical_sol_lamports
        || config.accounted_pending_sol_lamports < round.pending_sol_snapshot_lamports
        || round.fixed_jitosol_withdrawal_target_units > round.historical_jitosol_units
        || round.historical_jitosol_units > round.snapshot_pool_token_supply
    {
        return Err(Piv1Error::CumulativeReconciliationMismatch);
    }
    let snapshot_value = add(round.historical_sol_lamports,
        piv1_math::checked_mul_div_floor(round.historical_jitosol_units,
            round.snapshot_pool_total_lamports, round.snapshot_pool_token_supply)?)?;
    if snapshot_value != round.historical_value_lamports {
        return Err(Piv1Error::CumulativeReconciliationMismatch);
    }
    let settled = round.lifecycle == DistributionLifecycle::Settled;
    let expected_hwm = if settled {
        round.settled_protected_hwm_lamports
    } else {
        round.old_protected_principal_lamports
    };
    let expected_carry = if settled {
        round.cumulative_cooldown_rewards_lamports
    } else {
        0
    };
    let expected_kif_carry = if settled {
        round.actual_kif_carry_next_lamports
    } else {
        round.kif_carry_input_lamports
    };
    if config.protected_principal_hwm_lamports != expected_hwm
        || config.next_cycle_yield_lamports != expected_carry
        || config.collective_kif_carry_lamports != expected_kif_carry
    {
        return Err(Piv1Error::CumulativeReconciliationMismatch);
    }
    Ok(())
}

/// Canonical physical pending SOL obligation. The ledger retains full recognized
/// contribution value until integration, including SOL already used by this round.
pub fn expected_pending_sol_lamports(
    config: &PivConfig,
    round: &ActiveDistribution,
) -> Piv1Result<u64> {
    validate_custody_state_binding(config, round)?;
    let used = if round.lifecycle == DistributionLifecycle::Idle {
        0
    } else {
        round.pending_sol_used_lamports
    };
    sub(config.accounted_pending_sol_lamports, used)
}

/// Derives expected custody under the documented atomic composition convention.
/// Carry resides in principal SOL at Idle; new cooldown yield stays in escrow
/// through settlement; zero-active KIF compound moves into principal at settlement.
pub fn economic_custody_obligations(
    config: &PivConfig,
    round: &ActiveDistribution,
) -> Piv1Result<EconomicVaultAmounts> {
    let pending_sol_lamports = expected_pending_sol_lamports(config, round)?;
    let active = round.lifecycle != DistributionLifecycle::Idle;
    let assigned = if active { round.cumulative_jitosol_assigned_units } else { 0 };
    let principal_extra = if active {
        add(
            sub(round.prior_next_cycle_yield_lamports,
                round.prior_next_cycle_yield_used_lamports()?)?,
            round.actual_zero_active_kif_compound_lamports,
        )?
    } else {
        config.next_cycle_yield_lamports
    };
    let escrow_sol_lamports = if !active {
        0
    } else if round.lifecycle == DistributionLifecycle::Settled {
        // The recorded balance is pre-payment; this field is post-payment.
        round.actual_escrow_remainder_lamports
    } else {
        sub(
            add(add(round.pending_sol_used_lamports,
                    round.prior_next_cycle_yield_used_lamports()?)?,
                round.cumulative_finalized_native_lamports)?,
            round.cumulative_recovered_stake_rent_lamports,
        )?
    };
    Ok(EconomicVaultAmounts {
        pending_sol_lamports,
        pending_jitosol_units: config.accounted_pending_jitosol_units,
        principal_sol_lamports: add(config.accounted_historical_sol_lamports,
                                   principal_extra)?,
        principal_jitosol_units: sub(config.accounted_historical_jitosol_units,
                                    assigned)?,
        escrow_sol_lamports,
        kif_sol_lamports: add(config.kif_claim_liability_lamports,
                             config.collective_kif_carry_lamports)?,
    })
}

/// Proves positive excess only after every economic vault's own obligation is
/// covered. An unrelated surplus cannot cover a known deficit.
pub fn economic_custody_surplus(
    config: &PivConfig,
    round: &ActiveDistribution,
    observed: EconomicCustodyObservation,
) -> Piv1Result<EconomicVaultAmounts> {
    let expected = economic_custody_obligations(config, round)?;
    let actual = observed.amounts()?;
    let delta = |a: u64, e: u64| a.checked_sub(e)
        .ok_or(Piv1Error::EconomicCustodyDeficit);
    Ok(EconomicVaultAmounts {
        pending_sol_lamports: delta(actual.pending_sol_lamports, expected.pending_sol_lamports)?,
        principal_sol_lamports: delta(actual.principal_sol_lamports, expected.principal_sol_lamports)?,
        escrow_sol_lamports: delta(actual.escrow_sol_lamports, expected.escrow_sol_lamports)?,
        kif_sol_lamports: delta(actual.kif_sol_lamports, expected.kif_sol_lamports)?,
        pending_jitosol_units: delta(actual.pending_jitosol_units, expected.pending_jitosol_units)?,
        principal_jitosol_units: delta(actual.principal_jitosol_units, expected.principal_jitosol_units)?,
    })
}

/// Records normalization only after exact matched movements from all economic
/// source vaults to the dedicated pending vaults. Existing pending excess is
/// recognized in the same atomic record. No transfer is performed by this helper.
///
/// Movement-bearing normalization is gated by pause. Already-received pending
/// custody remains recordable via the Task 2.2 functions during pause/recovery.
pub fn record_economic_normalization(
    config: &mut PivConfig,
    round: &ActiveDistribution,
    before: EconomicCustodyObservation,
    after: EconomicCustodyObservation,
) -> Piv1Result<PendingReconciliationResult> {
    let surplus = economic_custody_surplus(config, round, before)?;
    let moved_sol = add(add(surplus.principal_sol_lamports, surplus.escrow_sol_lamports)?,
                        surplus.kif_sol_lamports)?;
    if moved_sol != 0 || surplus.principal_jitosol_units != 0 {
        config.ensure_unpaused()?;
        if round.lifecycle == DistributionLifecycle::RecoveryRequired {
            return Err(Piv1Error::RecoveryRequired);
        }
    }
    let mut expected_after = economic_custody_obligations(config, round)?;
    let actual_before = before.amounts()?;
    expected_after.pending_sol_lamports = add(actual_before.pending_sol_lamports, moved_sol)?;
    expected_after.pending_jitosol_units = add(actual_before.pending_jitosol_units,
                                               surplus.principal_jitosol_units)?;
    if !before.same_floors(after) || after.amounts()? != expected_after {
        return Err(Piv1Error::ContributionObservationMismatch);
    }
    let mut next = config.clone();
    let result = reconcile_pending_contributions(&mut next, round, after.pending())?;
    if economic_custody_surplus(&next, round, after)? != EconomicVaultAmounts::default() {
        return Err(Piv1Error::CumulativeReconciliationMismatch);
    }
    *config = next;
    Ok(result)
}

/// Conservative book value over a validated pool observation. This uses the
/// accepted integer ratio seam; exact live SPL/Jito account mapping is deferred.
pub fn observed_token_book_value(units: u64, pool: PoolSnapshot) -> Piv1Result<u64> {
    pool.validate().map_err(|_| Piv1Error::InvalidCustodyObservation)?;
    if units > pool.pool_token_supply {
        return Err(Piv1Error::InvalidCustodyObservation);
    }
    if pool.pool_token_supply == 0 {
        return if units == 0 { Ok(0) } else { Err(Piv1Error::InvalidCustodyObservation) };
    }
    piv1_math::checked_mul_div_floor(units, pool.total_pool_lamports,
                                    pool.pool_token_supply).map_err(Into::into)
}

/// Derives integration inputs from exact before/after physical normalization.
/// No chosen historical amount, contribution value, or HWM proof is accepted.
pub fn derive_pending_integration(
    config: &PivConfig,
    round: &ActiveDistribution,
    completed_at: i64,
    pool: PoolSnapshot,
    before: EconomicCustodyObservation,
    after: EconomicCustodyObservation,
) -> Piv1Result<PendingIntegrationInput> {
    config.ensure_unpaused()?;
    if round.lifecycle != DistributionLifecycle::Settled {
        return Err(Piv1Error::InvalidLifecycle);
    }
    if economic_custody_surplus(config, round, before)? != EconomicVaultAmounts::default() {
        return Err(Piv1Error::InvalidCustodyObservation);
    }
    let b = before.amounts()?;
    let mut expected_after = b;
    expected_after.pending_sol_lamports = 0;
    expected_after.pending_jitosol_units = 0;
    expected_after.escrow_sol_lamports = 0;
    expected_after.principal_sol_lamports =
        add(add(b.principal_sol_lamports, b.pending_sol_lamports)?, b.escrow_sol_lamports)?;
    expected_after.principal_jitosol_units =
        add(b.principal_jitosol_units, b.pending_jitosol_units)?;
    if !before.same_floors(after) || after.amounts()? != expected_after {
        return Err(Piv1Error::ContributionObservationMismatch);
    }
    let contribution_value = add(config.accounted_pending_sol_lamports,
        observed_token_book_value(config.accounted_pending_jitosol_units, pool)?)?;
    let new_historical_sol = sub(expected_after.principal_sol_lamports,
                                 config.next_cycle_yield_lamports)?;
    let hwm = add(config.protected_principal_hwm_lamports, contribution_value)?;
    let protected_value = add(new_historical_sol,
        observed_token_book_value(expected_after.principal_jitosol_units, pool)?)?;
    if protected_value < hwm {
        return Err(Piv1Error::HighWaterMarkDecrease);
    }
    Ok(PendingIntegrationInput {
        sequence: round.active_sequence,
        completed_at,
        integrated_pending_sol_lamports: config.accounted_pending_sol_lamports,
        integrated_pending_jitosol_units: config.accounted_pending_jitosol_units,
        contribution_value_lamports: contribution_value,
        new_accounted_historical_jitosol_units: expected_after.principal_jitosol_units,
        new_accounted_historical_sol_lamports: new_historical_sol,
        new_protected_hwm_lamports: hwm,
    })
}

fn add(a: u64, b: u64) -> Piv1Result<u64> {
    a.checked_add(b).ok_or(Piv1Error::ArithmeticOverflow)
}

fn sub(a: u64, b: u64) -> Piv1Result<u64> {
    a.checked_sub(b).ok_or(Piv1Error::CumulativeReconciliationMismatch)
}
