#![forbid(unsafe_code)]
#![allow(unexpected_cfgs)]

//! Non-deployable PIV1 library with bounded pure state and accounting models.
//!
//! No Program ID, `#[program]` entrypoint, instruction handler, or CPI is
//! declared. Fixed-account authentication is read-only under explicit trusted
//! runtime program-ID and Rent inputs. A separate validated persistence utility
//! atomically copies existing state bytes; neither path performs transfers or CPI.

pub mod accounts;
pub mod constants;
pub mod errors;
pub mod guardian_clock_accounts;
pub mod events;
pub mod instructions;
pub mod integrations;
pub mod kif_claim_accounts;
pub mod state;
pub mod state_persistence;

/// Founder-accepted pure accounting remains in its host-testable crate.
pub use piv1_math as math;
