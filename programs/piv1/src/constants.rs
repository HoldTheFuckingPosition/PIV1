//! Fixed state-model constants confirmed for PIV1 V1.
//!
//! Cluster addresses and dynamic protocol values deliberately do not appear
//! here. They remain validated configuration or future integration inputs.

/// First bounded serialized state-layout version.
pub const STATE_LAYOUT_VERSION: u8 = 1;
/// Bytes reserved by Anchor account discriminators once owner-bound accounts exist.
pub const PLANNED_ACCOUNT_DISCRIMINATOR_BYTES: usize = 8;
/// Immutable maximum configured slippage tolerance, in basis points.
pub const MAX_CONFIGURED_SLIPPAGE_BPS: u16 = 1;
/// Confirmed minimum time between successful distribution preparations.
pub const MINIMUM_DISTRIBUTION_INTERVAL_SECONDS: i64 = 10 * 24 * 60 * 60;
/// Confirmed retry cooldown after a valid technically insufficient attempt.
pub const INSUFFICIENT_RETRY_COOLDOWN_SECONDS: i64 = 24 * 60 * 60;
/// Confirmed fixed KIF period duration.
pub const KIF_PERIOD_SECONDS: i64 = 2_592_000;
/// Confirmed fixed guardian-set size.
pub const GUARDIAN_COUNT: usize = 6;
/// Bitmap mask covering exactly the six configured guardian slots.
pub const GUARDIAN_BITMAP_MASK: u8 = 0b00_111111;
/// Fixed migration reserve in the version-one configuration payload.
pub const CONFIG_MIGRATION_RESERVE_BYTES: usize = 64;

/// Recovery was triggered by a finalized stake leg losing delegated value.
pub const RECOVERY_FLAG_COOLDOWN_LOSS: u8 = 1 << 0;
/// Recovery was triggered because the validated residual value missed its HWM floor.
pub const RECOVERY_FLAG_RESIDUAL_HWM: u8 = 1 << 1;
