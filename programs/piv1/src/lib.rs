#![forbid(unsafe_code)]
#![allow(unexpected_cfgs)]

//! Non-deployable PIV1 library with bounded Task 1.3 state validation.
//!
//! No Program ID, `#[program]` entrypoint, instruction handler, or CPI is
//! declared. Task 1.3 adds only Anchor-compatible layouts and pure transitions.

pub mod constants;
pub mod errors;
pub mod events;
pub mod instructions;
pub mod integrations;
pub mod state;

/// Founder-accepted pure accounting remains in its host-testable crate.
pub use piv1_math as math;
