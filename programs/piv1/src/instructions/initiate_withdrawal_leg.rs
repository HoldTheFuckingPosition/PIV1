//! Protected withdrawal-leg plus immediate-deactivation boundary marker.
//!
//! One later call may create only its deterministic metadata/stake pair and
//! assign the program-derived maximum-safe portion of the remaining fixed
//! target. This file contains no validator logic, CPI, or account creation.

instruction_marker!(pub InitiateWithdrawalLeg);
