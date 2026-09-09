#![forbid(unsafe_code)]
#![allow(unexpected_cfgs)]

//! Non-deployable PIV1 library with bounded pure state and accounting models.
//!
//! No Program ID, `#[program]` entrypoint, instruction handler, or CPI is
//! declared. Fixed-account authentication is read-only under explicit trusted
//! runtime program-ID and Rent inputs; it performs no transfer or CPI.

pub mod accounts;
pub mod constants;
pub mod errors;
pub mod events;
pub mod instructions;
pub mod integrations;
pub mod state;

/// Founder-accepted pure accounting remains in its host-testable crate.
pub use piv1_math as math;
