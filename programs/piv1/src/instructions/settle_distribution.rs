//! Atomic beneficiary settlement/accounting boundary marker.
//!
//! Settlement must remain impossible until exact target assignment, complete
//! leg finalization, and escrow reconciliation. HTFP, Team, KIF, compound,
//! principal, rent, and external fees stay distinct. No transfer exists here.

instruction_marker!(pub SettleDistribution);
