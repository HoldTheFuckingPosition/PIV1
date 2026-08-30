//! Bounded distribution-header and temporary withdrawal-leg markers.
//!
//! One reusable `ActiveDistribution` header may represent only one active
//! round. It must never contain an unbounded leg vector. Each later
//! `WithdrawalLeg` account is bound to `(round_sequence, leg_index)` and rolls
//! checked cumulative values into the header before safe closure. Actual
//! layouts and transitions are deferred to Task 1.3.

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct ActiveDistribution;

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct WithdrawalLeg;
