//! Distribution preparation/snapshot boundary marker.
//!
//! This boundary eventually fixes one active round's obligations, HWM proof,
//! KIF eligibility, shortfall, dynamic technical floor, and exact total target.
//! Task 1.1 performs none of those calculations or state changes.

instruction_marker!(pub PrepareDistribution);
