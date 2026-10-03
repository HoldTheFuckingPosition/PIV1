#![forbid(unsafe_code)]
#![allow(unexpected_cfgs)]

//! PIV1 accounting and authenticated custody, genesis, guardian and staking boundaries.
//!
//! The thin native entrypoint receives the actual runtime program ID, avoiding
//! an invented static identity. No deployed Program ID is selected. Anchor
//! serialization/events and pinned Solana APIs remain in use. Ordinary host
//! processing rejects execution; explicit host seams model payment and rollback.

pub mod accounts;
mod allocation_budget;
pub mod constants;
pub mod contribution_execution;
pub mod economic_normalization;
pub mod distribution_preparation_execution;
pub mod errors;
pub mod genesis_allocation;
pub mod genesis_initialization;
pub mod genesis_model;
pub mod genesis_preflight;
pub mod genesis_recipients;
pub mod guardian_clock_accounts;
pub mod guardian_operations;
pub mod events;
pub mod instructions;
pub mod integrations;
pub mod instruction_boundary;
pub mod instruction_errors;
pub mod initial_bootstrap_execution;
pub mod kif_claim_accounts;
pub mod kif_claim_execution;
pub mod pending_accounts;
pub mod pending_reconciliation;
pub mod principal_deposit_execution;
pub mod squads_accounts;
pub mod squads_execution;
pub mod state;
pub mod state_persistence;
pub mod withdrawal_leg_execution;
pub mod withdrawal_finalization_execution;

/// Founder-accepted pure accounting remains in its host-testable crate.
pub use piv1_math as math;

// D-003 native boundary: Anchor program codegen requires a static ID, while the
// dedicated live identity remains uncreated. Keep pinned safe macro wiring and
// preserve no-entrypoint/cpi feature semantics; no handwritten unsafe decoder.
#[cfg(not(feature = "no-entrypoint"))]
anchor_lang::solana_program::entrypoint!(process_instruction);

pub use instruction_boundary::process_instruction;
