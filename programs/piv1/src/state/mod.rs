//! Compile-only state namespace.
//!
//! All marker types are deliberately fieldless. Versioned layouts, `SPACE`
//! calculations, discriminators, bumps, integer bounds, and transitions are
//! provisional work for later authorized tasks.

pub mod config;
pub mod custody;
pub mod distribution;
pub mod guardian;

pub use config::PivConfig;
pub use custody::{
    DistributionEscrowRole, KifSolVaultRole, OperationalSolVaultRole,
    PendingJitoVaultRole, PendingSolVaultRole, PivAuthorityRole,
    PrincipalJitoVaultRole, PrincipalSolQueueRole, WithdrawalStakeRole,
};
pub use distribution::{ActiveDistribution, WithdrawalLeg};
pub use guardian::{GuardianRegistry, GuardianReward};
