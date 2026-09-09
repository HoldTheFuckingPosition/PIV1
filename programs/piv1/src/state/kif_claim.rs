//! Isolated payment of an already-earned KIF liability, including during pause.
//!
//! Pure supplied state and custody are not runtime authorization capabilities.
//! Future handlers must authenticate the fixed accounts and signer before the
//! transfer, reauthenticate after interactions, and commit state/custody atomically.
//! No activity, current registry, distribution, pool or Clock input is needed.

use anchor_lang::prelude::Pubkey;
use crate::{
    errors::{Piv1Error, Piv1Result},
    state::{reconciliation::SolVaultBalance, GuardianReward, PivConfig},
};

/// Positive requested payment and optimistic replay guard on the earned ledger.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct KifClaimRequest {
    pub amount_lamports: u64,
    pub expected_cumulative_claimed: u64,
}

/// Isolated source funding and fixed guardian destination balance.
/// The source floor must come from authenticated runtime Rent.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct KifClaimCustodyObservation {
    pub kif_sol: SolVaultBalance,
    pub guardian_lamports: u64,
}

/// Exact transfer derived from stored identities, never a caller-chosen payout.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct KifClaimTransfer {
    pub source: Pubkey,
    pub destination: Pubkey,
    pub amount_lamports: u64,
}

/// Private staged effects. This is an ordinary pure value, not serialized state
/// or an account authorization capability. A cloned plan cannot bypass the
/// complete pre-state equality and cumulative-claimed replay guard at commit.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PreparedKifClaim {
    before_config: Box<PivConfig>,
    before_reward: GuardianReward,
    next_config: Box<PivConfig>,
    next_reward: GuardianReward,
    expected_after: KifClaimCustodyObservation,
    excess_lamports: u64,
    transfer: KifClaimTransfer,
}

impl PreparedKifClaim {
    pub fn transfer(&self) -> KifClaimTransfer { self.transfer }

    /// Internal CEI staging only. Expected custody is a prediction, not a receipt.
    pub(crate) fn staged_execution_values(
        &self,
    ) -> (&PivConfig, &GuardianReward, KifClaimCustodyObservation) {
        (&self.next_config, &self.next_reward, self.expected_after)
    }

    /// Validates observed payment before replacing either supplied state object.
    /// All preexisting source excess and collective carry must remain untouched.
    #[inline(never)]
    pub fn commit(
        self,
        config: &mut PivConfig,
        reward: &mut GuardianReward,
        after: KifClaimCustodyObservation,
    ) -> Piv1Result<KifClaimTransfer> {
        if *config != *self.before_config || *reward != self.before_reward {
            return Err(Piv1Error::KifClaimStateChanged);
        }
        if after != self.expected_after {
            return Err(Piv1Error::KifClaimObservationMismatch);
        }
        validate_selected_liability(&self.next_config, &self.next_reward)?;
        if source_excess(&self.next_config, after.kif_sol)? != self.excess_lamports {
            return Err(Piv1Error::KifClaimObservationMismatch);
        }
        *config = *self.next_config;
        *reward = self.next_reward;
        Ok(self.transfer)
    }
}

/// Prepares exact payment and all four accounting changes before interaction.
///
/// This verifies individual/global accounting identities and component bounds;
/// it cannot recompute the sum of every current and historical reward account.
/// Authenticated initialization/credits/claims must maintain that global sum.
/// Already-earned ownership survives later registry rotation and inactivity.
#[inline(never)]
pub fn prepare_kif_claim(
    config: &PivConfig,
    reward: &GuardianReward,
    request: KifClaimRequest,
    before: KifClaimCustodyObservation,
) -> Piv1Result<PreparedKifClaim> {
    validate_selected_liability(config, reward)?;
    if config.kif_sol_vault == reward.guardian { return Err(Piv1Error::AccountAlias); }
    if request.amount_lamports == 0 { return Err(Piv1Error::ZeroKifClaim); }
    if request.expected_cumulative_claimed != reward.cumulative_claimed {
        return Err(Piv1Error::StaleKifClaim);
    }
    if request.amount_lamports > reward.claimable_lamports
        || request.amount_lamports > config.kif_claim_liability_lamports
    {
        return Err(Piv1Error::KifClaimExceeded);
    }
    let excess_lamports = source_excess(config, before.kif_sol)?;
    let amount = request.amount_lamports;
    let mut next_config = clone_config(config);
    let mut next_reward = *reward;
    next_config.kif_claim_liability_lamports = next_config.kif_claim_liability_lamports
        .checked_sub(amount).ok_or(Piv1Error::KifClaimExceeded)?;
    next_config.cumulative_kif_claimed_lamports =
        add(next_config.cumulative_kif_claimed_lamports, amount)?;
    next_reward.claimable_lamports = next_reward.claimable_lamports
        .checked_sub(amount).ok_or(Piv1Error::KifClaimExceeded)?;
    next_reward.cumulative_claimed = add(next_reward.cumulative_claimed, amount)?;
    let expected_after = KifClaimCustodyObservation {
        kif_sol: SolVaultBalance {
            lamports: before.kif_sol.lamports.checked_sub(amount)
                .ok_or(Piv1Error::KifClaimBackingDeficit)?,
            non_economic_floor_lamports: before.kif_sol.non_economic_floor_lamports,
        },
        guardian_lamports: add(before.guardian_lamports, amount)?,
    };
    validate_selected_liability(&next_config, &next_reward)?;
    if source_excess(&next_config, expected_after.kif_sol)? != excess_lamports {
        return Err(Piv1Error::KifClaimObservationMismatch);
    }
    Ok(PreparedKifClaim {
        before_config: clone_config(config), before_reward: *reward, next_config, next_reward,
        expected_after, excess_lamports,
        transfer: KifClaimTransfer { source: config.kif_sol_vault,
            destination: reward.guardian, amount_lamports: amount },
    })
}

// Keep the fixed-size clone and allocation inside this frame, not its caller.
#[inline(never)]
fn clone_config(config: &PivConfig) -> Box<PivConfig> {
    Box::new(config.clone())
}

fn validate_selected_liability(config: &PivConfig, reward: &GuardianReward) -> Piv1Result<()> {
    // Deliberately no ensure_unpaused or current-registry/activity equality check.
    config.validate_initialized()?;
    reward.validate()?;
    if reward.cumulative_earned > config.cumulative_kif_credited_lamports
        || reward.cumulative_claimed > config.cumulative_kif_claimed_lamports
        || reward.claimable_lamports > config.kif_claim_liability_lamports
    {
        return Err(Piv1Error::CumulativeReconciliationMismatch);
    }
    Ok(())
}

fn source_excess(config: &PivConfig, source: SolVaultBalance) -> Piv1Result<u64> {
    let backing = add(source.non_economic_floor_lamports,
        add(config.kif_claim_liability_lamports, config.collective_kif_carry_lamports)?)?;
    source.lamports.checked_sub(backing).ok_or(Piv1Error::KifClaimBackingDeficit)
}

fn add(a: u64, b: u64) -> Piv1Result<u64> {
    a.checked_add(b).ok_or(Piv1Error::ArithmeticOverflow)
}
