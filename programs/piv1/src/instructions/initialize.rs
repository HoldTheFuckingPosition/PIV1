//! Initialization marker for permanent, separated custody accounts.
//!
//! Later initialization must create distinct principal/pending legacy Token
//! accounts at PIV1-derived addresses, both controlled by the shared PIV
//! authority, plus the separately owned native-SOL vault roles. Neither token
//! vault may be an ATA. No creation or funding occurs here.

instruction_marker!(pub InitializePiv1);
