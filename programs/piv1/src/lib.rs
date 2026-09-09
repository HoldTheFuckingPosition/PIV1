#![forbid(unsafe_code)]
#![allow(unexpected_cfgs)]

//! Non-deployable PIV1 library with bounded pure state and accounting models.
//!
//! No Program ID, `#[program]` entrypoint or instruction handler is declared.
//! Account authentication and persistence use trusted runtime program-ID/Rent
//! inputs. Isolated KIF execution wires one fixed System CPI on Solana; the
//! runtime-facing path rejects hosts, which have an explicit modeling seam.

pub mod accounts;
pub mod constants;
pub mod errors;
pub mod guardian_clock_accounts;
pub mod events;
pub mod instructions;
pub mod integrations;
pub mod kif_claim_accounts;
pub mod kif_claim_execution;
pub mod state;
pub mod state_persistence;

/// Founder-accepted pure accounting remains in its host-testable crate.
pub use piv1_math as math;
