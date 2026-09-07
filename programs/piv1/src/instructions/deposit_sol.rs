//! Explicit SOL-contribution marker.
//!
//! Valid contributions belong in the pending SOL custody category and create
//! no depositor withdrawal, ownership, or reward right. Task 2.2 adds only a
//! pure observation-based accounting seam; transfer and pause-handler policy
//! remain deferred, with explicit-deposit callability during pause PROVISIONAL.

instruction_marker!(pub DepositSol);
