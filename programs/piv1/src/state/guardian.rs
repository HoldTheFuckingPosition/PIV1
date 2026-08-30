//! Fixed six-guardian registry and per-guardian reward-account markers.
//!
//! Activity periods, claims, collective carry, fields, sizes, and governance
//! synchronization are intentionally not implemented in Task 1.1.

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct GuardianRegistry;

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct GuardianReward;
