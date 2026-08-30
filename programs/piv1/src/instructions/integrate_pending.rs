//! Post-settlement pending-contribution integration boundary marker.
//!
//! Pending SOL and pending JitoSOL remain isolated from the fixed active round
//! until this later logical boundary. No reconciliation is implemented here.

instruction_marker!(pub IntegratePending);
