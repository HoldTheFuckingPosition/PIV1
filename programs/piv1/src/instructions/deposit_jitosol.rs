//! Explicit JitoSOL-contribution marker.
//!
//! Contributions must enter the distinct pending JitoSOL token vault and
//! cannot become historical yield before integration. Task 2.2 adds only a
//! pure decoded-token-delta accounting seam; no token transfer exists here.

instruction_marker!(pub DepositJitoSol);
