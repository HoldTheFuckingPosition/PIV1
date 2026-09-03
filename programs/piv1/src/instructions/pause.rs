//! Emergency pause/unpause marker.
//!
//! Governance authorization, blocked-operation coverage, active-round policy,
//! and handlers remain deferred. Confirmed K-012 permits only an already-earned
//! recorded KIF liability to be claimed from the isolated KifSolVault during a
//! pause under the confirmed guardian and destination constraints; that claim
//! handler is not implemented.

instruction_marker!(pub SetPause);
