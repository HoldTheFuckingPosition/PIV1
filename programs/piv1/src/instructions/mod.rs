//! Compile-only instruction namespace.
//!
//! Marker types below are not Anchor `Accounts` contexts and have no handlers.
//! Account constraints, arguments, authorization, checked effects, events,
//! and transitions must be introduced only by later bounded tasks.

macro_rules! instruction_marker {
    ($visibility:vis $name:ident) => {
        #[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
        $visibility struct $name;
    };
}

pub mod claim_kif;
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
pub mod settle_distribution;
pub mod stake_pending_sol;
pub mod update_config;

pub use claim_kif::ClaimKif;
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
