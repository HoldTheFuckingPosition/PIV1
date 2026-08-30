//! Direct/untracked balance-reconciliation marker.
//!
//! Positive unaccounted deltas remain pending contributions. Events can never
//! replace on-chain balance/accounting truth.

instruction_marker!(pub ReconcileUntrackedBalances);
