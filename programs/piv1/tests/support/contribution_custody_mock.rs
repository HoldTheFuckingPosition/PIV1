#![allow(dead_code)]

use piv1::{
    errors::Piv1Error,
    state::{
        reconcile_pending_contributions, record_explicit_jitosol_contribution,
        record_explicit_sol_contribution, ActiveDistribution,
        ExplicitContributionRecord, JitoSolCustodyObservation,
        PendingCustodyObservation, PendingReconciliationResult, PivConfig,
        SolCustodyObservation,
    },
};

/// Persistent deterministic failures used to prove whole-mock atomicity.
///
/// Failure configuration is not consumed on rejection. Tests explicitly clear
/// it, so rejected operations preserve even the injection state.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum MockContributionFailurePoint {
    ExplicitSolAfterPhysicalCredit,
    ExplicitSolAfterAccounting,
    ExplicitJitoSolAfterPhysicalCredit,
    ExplicitJitoSolAfterAccounting,
    ReconciliationAfterAccounting,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum MockContributionError {
    Accounting(Piv1Error),
    InjectedFailure(MockContributionFailurePoint),
}

impl From<Piv1Error> for MockContributionError {
    fn from(error: Piv1Error) -> Self {
        Self::Accounting(error)
    }
}

pub type MockContributionResult<T> = Result<T, MockContributionError>;

/// Fixed audit categories for incoming pending custody.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct MockContributionAudit {
    pub explicit_sol_lamports: u64,
    pub explicit_jitosol_units: u64,
    pub direct_sol_lamports: u64,
    pub direct_jitosol_units: u64,
    pub reconciled_direct_sol_lamports: u64,
    pub reconciled_direct_jitosol_units: u64,
    pub outgoing_sol_lamports: u64,
    pub outgoing_jitosol_units: u64,
}

/// Deterministic host-only custody model for the two pending vaults.
///
/// This fixed-size model is integration-test support and is never exported by
/// the production library. Its scalar balances do not claim exact System or
/// Token Program behavior. In particular, `pending_sol_excluded_lamports`
/// stands in for a handler-validated non-economic floor, and the JitoSOL field
/// is token units only rather than token-account lamports.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MockContributionCustody {
    pub config: PivConfig,
    pub active_distribution: ActiveDistribution,
    pub pending_sol_vault_lamports: u64,
    pub pending_sol_excluded_lamports: u64,
    pub pending_jitosol_token_units: u64,
    pub audit: MockContributionAudit,
    pub failure_point: Option<MockContributionFailurePoint>,
    initial_accounted_pending_sol_lamports: u64,
    initial_accounted_pending_jitosol_units: u64,
}

impl MockContributionCustody {
    pub fn new(
        config: PivConfig,
        active_distribution: ActiveDistribution,
        pending_sol_excluded_lamports: u64,
    ) -> MockContributionResult<Self> {
        config.validate_initialized()?;
        active_distribution.validate()?;
        let pending_sol_vault_lamports = pending_sol_excluded_lamports
            .checked_add(config.accounted_pending_sol_lamports)
            .ok_or(Piv1Error::ArithmeticOverflow)?;
        let pending_jitosol_token_units = config.accounted_pending_jitosol_units;
        let initial_accounted_pending_sol_lamports =
            config.accounted_pending_sol_lamports;
        let initial_accounted_pending_jitosol_units =
            config.accounted_pending_jitosol_units;
        let custody = Self {
            config,
            active_distribution,
            pending_sol_vault_lamports,
            pending_sol_excluded_lamports,
            pending_jitosol_token_units,
            audit: MockContributionAudit::default(),
            failure_point: None,
            initial_accounted_pending_sol_lamports,
            initial_accounted_pending_jitosol_units,
        };
        custody.validate_conservation()?;
        Ok(custody)
    }

    pub fn spendable_sol_lamports(&self) -> MockContributionResult<u64> {
        self.pending_sol_vault_lamports
            .checked_sub(self.pending_sol_excluded_lamports)
            .ok_or(Piv1Error::InvalidCustodyObservation.into())
    }

    pub fn unexplained_sol_lamports(&self) -> MockContributionResult<u64> {
        self.spendable_sol_lamports()?
            .checked_sub(self.config.accounted_pending_sol_lamports)
            .ok_or(Piv1Error::PendingCustodyDeficit.into())
    }

    pub fn unexplained_jitosol_units(&self) -> MockContributionResult<u64> {
        self.pending_jitosol_token_units
            .checked_sub(self.config.accounted_pending_jitosol_units)
            .ok_or(Piv1Error::PendingCustodyDeficit.into())
    }

    /// Simulates one atomic explicit SOL action with independently selectable
    /// expected and actual credited amounts for adversarial mismatch tests.
    pub fn record_explicit_sol(
        &mut self,
        expected_lamports: u64,
        physical_credit_lamports: u64,
    ) -> MockContributionResult<ExplicitContributionRecord> {
        let mut next = self.clone();
        let before = next.pending_sol_vault_lamports;
        next.pending_sol_vault_lamports = before
            .checked_add(physical_credit_lamports)
            .ok_or(Piv1Error::ArithmeticOverflow)?;
        next.fail_if(MockContributionFailurePoint::ExplicitSolAfterPhysicalCredit)?;

        let record = record_explicit_sol_contribution(
            &mut next.config,
            &next.active_distribution,
            expected_lamports,
            SolCustodyObservation {
                vault_lamports_before: before,
                vault_lamports_after: next.pending_sol_vault_lamports,
                non_economic_floor_lamports: next.pending_sol_excluded_lamports,
            },
        )?;
        next.fail_if(MockContributionFailurePoint::ExplicitSolAfterAccounting)?;
        next.audit.explicit_sol_lamports = next
            .audit
            .explicit_sol_lamports
            .checked_add(record.observed_increase)
            .ok_or(Piv1Error::ArithmeticOverflow)?;
        next.validate_conservation()?;

        *self = next;
        Ok(record)
    }

    /// Simulates one atomic explicit JitoSOL token contribution.
    pub fn record_explicit_jitosol(
        &mut self,
        expected_units: u64,
        physical_credit_units: u64,
    ) -> MockContributionResult<ExplicitContributionRecord> {
        let mut next = self.clone();
        let before = next.pending_jitosol_token_units;
        next.pending_jitosol_token_units = before
            .checked_add(physical_credit_units)
            .ok_or(Piv1Error::ArithmeticOverflow)?;
        next.fail_if(
            MockContributionFailurePoint::ExplicitJitoSolAfterPhysicalCredit,
        )?;

        let record = record_explicit_jitosol_contribution(
            &mut next.config,
            &next.active_distribution,
            expected_units,
            JitoSolCustodyObservation {
                token_units_before: before,
                token_units_after: next.pending_jitosol_token_units,
            },
        )?;
        next.fail_if(MockContributionFailurePoint::ExplicitJitoSolAfterAccounting)?;
        next.audit.explicit_jitosol_units = next
            .audit
            .explicit_jitosol_units
            .checked_add(record.observed_increase)
            .ok_or(Piv1Error::ArithmeticOverflow)?;
        next.validate_conservation()?;

        *self = next;
        Ok(record)
    }

    /// Applies an untracked physical SOL credit without touching accounting.
    pub fn direct_credit_sol(
        &mut self,
        lamports: u64,
    ) -> MockContributionResult<()> {
        let mut next = self.clone();
        next.pending_sol_vault_lamports = next
            .pending_sol_vault_lamports
            .checked_add(lamports)
            .ok_or(Piv1Error::ArithmeticOverflow)?;
        next.audit.direct_sol_lamports = next
            .audit
            .direct_sol_lamports
            .checked_add(lamports)
            .ok_or(Piv1Error::ArithmeticOverflow)?;
        next.validate_conservation()?;
        *self = next;
        Ok(())
    }

    /// Applies an untracked physical JitoSOL credit without touching accounting.
    pub fn direct_credit_jitosol(
        &mut self,
        units: u64,
    ) -> MockContributionResult<()> {
        let mut next = self.clone();
        next.pending_jitosol_token_units = next
            .pending_jitosol_token_units
            .checked_add(units)
            .ok_or(Piv1Error::ArithmeticOverflow)?;
        next.audit.direct_jitosol_units = next
            .audit
            .direct_jitosol_units
            .checked_add(units)
            .ok_or(Piv1Error::ArithmeticOverflow)?;
        next.validate_conservation()?;
        *self = next;
        Ok(())
    }

    /// Atomically accounts both currently unexplained pending-vault surpluses.
    pub fn reconcile(&mut self) -> MockContributionResult<PendingReconciliationResult> {
        let mut next = self.clone();
        let result = reconcile_pending_contributions(
            &mut next.config,
            &next.active_distribution,
            PendingCustodyObservation {
                pending_sol_vault_lamports: next.pending_sol_vault_lamports,
                pending_sol_non_economic_floor_lamports:
                    next.pending_sol_excluded_lamports,
                pending_jitosol_token_units: next.pending_jitosol_token_units,
            },
        )?;
        next.fail_if(MockContributionFailurePoint::ReconciliationAfterAccounting)?;
        next.audit.reconciled_direct_sol_lamports = next
            .audit
            .reconciled_direct_sol_lamports
            .checked_add(result.newly_accounted_sol_lamports)
            .ok_or(Piv1Error::ArithmeticOverflow)?;
        next.audit.reconciled_direct_jitosol_units = next
            .audit
            .reconciled_direct_jitosol_units
            .checked_add(result.newly_accounted_jitosol_units)
            .ok_or(Piv1Error::ArithmeticOverflow)?;
        next.validate_conservation()?;

        *self = next;
        Ok(result)
    }

    pub fn set_failure_point(&mut self, point: MockContributionFailurePoint) {
        self.failure_point = Some(point);
    }

    pub fn clear_failure_point(&mut self) {
        self.failure_point = None;
    }

    pub fn validate_conservation(&self) -> MockContributionResult<()> {
        self.config.validate_initialized()?;
        self.active_distribution.validate()?;

        let spendable_sol = self.spendable_sol_lamports()?;
        let unexplained_sol = self.unexplained_sol_lamports()?;
        let unexplained_jitosol = self.unexplained_jitosol_units()?;
        let expected_physical_sol = checked_add(
            checked_add(
                self.initial_accounted_pending_sol_lamports,
                self.audit.explicit_sol_lamports,
            )?,
            self.audit.direct_sol_lamports,
        )?;
        let expected_physical_jitosol = checked_add(
            checked_add(
                self.initial_accounted_pending_jitosol_units,
                self.audit.explicit_jitosol_units,
            )?,
            self.audit.direct_jitosol_units,
        )?;
        let expected_accounted_sol = checked_add(
            checked_add(
                self.initial_accounted_pending_sol_lamports,
                self.audit.explicit_sol_lamports,
            )?,
            self.audit.reconciled_direct_sol_lamports,
        )?;
        let expected_accounted_jitosol = checked_add(
            checked_add(
                self.initial_accounted_pending_jitosol_units,
                self.audit.explicit_jitosol_units,
            )?,
            self.audit.reconciled_direct_jitosol_units,
        )?;
        let categorized_direct_sol = checked_add(
            self.audit.reconciled_direct_sol_lamports,
            unexplained_sol,
        )?;
        let categorized_direct_jitosol = checked_add(
            self.audit.reconciled_direct_jitosol_units,
            unexplained_jitosol,
        )?;

        if spendable_sol != expected_physical_sol
            || self.pending_jitosol_token_units != expected_physical_jitosol
            || self.config.accounted_pending_sol_lamports
                != expected_accounted_sol
            || self.config.accounted_pending_jitosol_units
                != expected_accounted_jitosol
            || categorized_direct_sol != self.audit.direct_sol_lamports
            || categorized_direct_jitosol != self.audit.direct_jitosol_units
            || self.audit.outgoing_sol_lamports != 0
            || self.audit.outgoing_jitosol_units != 0
        {
            return Err(Piv1Error::CumulativeReconciliationMismatch.into());
        }
        Ok(())
    }

    fn fail_if(
        &self,
        point: MockContributionFailurePoint,
    ) -> MockContributionResult<()> {
        if self.failure_point == Some(point) {
            return Err(MockContributionError::InjectedFailure(point));
        }
        Ok(())
    }
}

fn checked_add(left: u64, right: u64) -> MockContributionResult<u64> {
    left.checked_add(right)
        .ok_or(Piv1Error::ArithmeticOverflow.into())
}
