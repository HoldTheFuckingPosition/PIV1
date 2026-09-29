//! Strict claim, pending, normalization, genesis, bootstrap, guardian and intake ABIs.
//! Markers are not Anchor `Accounts` contexts. Other lifecycle markers remain
//! unimplemented; no unchecked or allocation-only initializer is dispatched.

macro_rules! instruction_marker {
    ($visibility:vis $name:ident) => {
        #[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
        $visibility struct $name;
    };
}

pub mod claim_kif;
pub mod bootstrap_initial_contributions;
pub mod deposit_jitosol;
pub mod deposit_sol;
pub mod finalize_withdrawal_leg;
pub mod guardian_heartbeat;
pub mod initialize;
pub mod initiate_withdrawal_leg;
pub mod integrate_pending;
pub mod pause;
pub mod prepare_distribution;
pub mod reconcile_untracked_balances;
pub mod reconcile_pending;
pub mod settle_distribution;
pub mod stake_pending_sol;
pub mod update_config;

pub use claim_kif::ClaimKif;
pub use bootstrap_initial_contributions::BootstrapInitialContributions;
pub use deposit_jitosol::DepositJitoSol;
pub use deposit_sol::DepositSol;
pub use finalize_withdrawal_leg::FinalizeWithdrawalLeg;
pub use guardian_heartbeat::GuardianHeartbeat;
pub use initialize::InitializePiv1;
pub use initiate_withdrawal_leg::InitiateWithdrawalLeg;
pub use integrate_pending::IntegratePending;
pub use pause::SetPause;
pub use prepare_distribution::PrepareDistribution;
pub use reconcile_untracked_balances::ReconcileUntrackedBalances;
pub use settle_distribution::SettleDistribution;
pub use stake_pending_sol::StakePendingSol;
pub use update_config::{UpdateGuardianSet, UpdateRecipients, UpdateStrategyConfig};

pub use reconcile_pending::ReconcilePendingContributions;
