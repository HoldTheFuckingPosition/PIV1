//! Compile-only markers for the confirmed, separated custody roles.
//!
//! These are not serialized PIV1 accounts and define no owner, address, seed,
//! authority, space, rent, or initialization behavior. The native vaults and
//! withdrawal stake are eventually System/Stake Program-owned; the two
//! JitoSOL vault addresses are distinct PIV1 PDAs whose initialized accounts
//! are legacy Token Program-owned; PivAuthority is address-only. The markers
//! exist solely to keep those roles distinct while schemas remain provisional.

macro_rules! custody_role_marker {
    ($($name:ident),+ $(,)?) => {
        $(
            #[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
            pub struct $name;
        )+
    };
}

custody_role_marker!(
    PivAuthorityRole,
    PrincipalJitoVaultRole,
    PendingJitoVaultRole,
    PendingSolVaultRole,
    PrincipalSolQueueRole,
    OperationalSolVaultRole,
    DistributionEscrowRole,
    KifSolVaultRole,
    WithdrawalStakeRole,
);
