//! Compile-only error namespace.
//!
//! Variants and stable numeric codes are deferred until the instructions and
//! transition checks that can actually return them are implemented.

/// Uninhabited marker preserving the planned public error boundary.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Piv1Error {}
