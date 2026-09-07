#![forbid(unsafe_code)]
#![allow(unexpected_cfgs)]

//! Non-deployable PIV1 library with bounded pure state and accounting models.
//!
//! No Program ID, `#[program]` entrypoint, instruction handler, or CPI is
//! declared. Task 2.2 adds observation-based pending contribution accounting
//! without account decoding, transfer, integration, or serialized-layout work.

pub mod constants;
pub mod errors;
pub mod events;
pub mod instructions;
pub mod integrations;
pub mod state;

/// Founder-accepted pure accounting remains in its host-testable crate.
pub use piv1_math as math;
