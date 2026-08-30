//! Narrow SPL stake-pool adapter marker.
//!
//! Method signatures, account validation, protected instruction builders,
//! dynamic minimums, balance deltas, and CPI belong to later tasks.

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct StakePoolAdapter;
