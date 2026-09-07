//! Direct/untracked balance-reconciliation marker.
//!
//! Positive unaccounted deltas remain pending contributions. Events can never
//! replace on-chain balance/accounting truth. Task 2.2 reconciliation remains
//! recordable during pause because it neither spends nor integrates custody.

instruction_marker!(pub ReconcileUntrackedBalances);
