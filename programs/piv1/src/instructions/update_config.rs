//! Governance-controlled configuration markers.
//!
//! These types do not define authorities, account inputs, live addresses, or
//! migration behavior. Real recipients and guardian keys must never be
//! invented by the program scaffold.

instruction_marker!(pub UpdateRecipients);
instruction_marker!(pub UpdateGuardianSet);
instruction_marker!(pub UpdateStrategyConfig);
