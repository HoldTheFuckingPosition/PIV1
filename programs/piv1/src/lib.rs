#![forbid(unsafe_code)]
#![allow(unexpected_cfgs)]

//! Modular, compile-only PIV1 Anchor scaffold.
//!
//! No Program ID or entrypoint is declared in Task 1.1. The public modules are
//! boundaries for later bounded tasks, not implemented on-chain interfaces.

pub mod constants;
pub mod errors;
pub mod events;
pub mod instructions;
pub mod integrations;
pub mod state;

/// Pure accounting stays in a host-testable crate and is empty until Task 1.2.
pub use piv1_math as math;
