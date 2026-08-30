//! Guardian KIF-claim marker.
//!
//! Later claims must stay backed by the separate KIF SOL vault and cannot
//! alter activity history or another guardian's liability.

instruction_marker!(pub ClaimKif);
