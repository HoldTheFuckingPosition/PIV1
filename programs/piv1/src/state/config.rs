//! Global configuration/accounting header marker.
//!
//! `PivConfig` will eventually bind the confirmed vault roles, official
//! strategy configuration, fixed recipients, pause state, timing, HWM, and
//! bounded guardian/distribution references. Task 1.1 defines no layout.

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct PivConfig;
