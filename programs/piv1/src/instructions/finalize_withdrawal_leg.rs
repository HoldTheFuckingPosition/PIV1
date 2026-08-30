//! Inactive-stake finalization into fixed distribution escrow marker.
//!
//! Each future ready leg is finalized independently while rent, reward/loss,
//! and replay accounting roll into the bounded round header. No logic exists
//! in Task 1.1.

instruction_marker!(pub FinalizeWithdrawalLeg);
