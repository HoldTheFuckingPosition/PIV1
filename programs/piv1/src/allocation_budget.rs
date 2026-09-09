//! Source inventory for the normal successful five-account claim only.
//!
//! The pinned default bump allocator never reclaims freed allocations. Count all
//! four Config boxes, including fresh authentication after CPI, cumulatively.
//! This inventory is not measured total heap usage and does not change the 32 KiB
//! heap. Caller clones, oversized entrypoint account lists, error/panic paths and
//! runtime-internal costs are excluded. Box exhaustion can abort; post-CPI
//! authentication and event allocation still require transaction rollback.

use core::{cell::RefCell, mem::{align_of, size_of}};
use anchor_lang::{prelude::AccountInfo, solana_program::instruction::AccountMeta};
use crate::state::{ActiveDistribution, GuardianReward, PivConfig};

// Evaluate on the actual compilation target, not just the inspected host DWARF.
const _: () = {
    assert!(size_of::<PivConfig>() == 1024 && align_of::<PivConfig>() == 8);
    assert!(size_of::<ActiveDistribution>() == 896 && align_of::<ActiveDistribution>() == 8);
    assert!(size_of::<AccountInfo<'static>>() == 48 && align_of::<AccountInfo<'static>>() == 8);
    assert!(size_of::<AccountMeta>() == 34 && align_of::<AccountMeta>() == 1);
    assert!(size_of::<usize>() == 8 && align_of::<usize>() == 8);
    assert!(size_of::<*const u8>() == 8 && align_of::<*const u8>() == 8);
    assert!(size_of::<Box<PivConfig>>() == 8 && align_of::<Box<PivConfig>>() == 8);
    assert!(size_of::<RefCell<&'static mut u64>>() == 16
        && align_of::<RefCell<&'static mut u64>>() == 8);
    assert!(size_of::<RefCell<&'static mut [u8]>>() == 24
        && align_of::<RefCell<&'static mut [u8]>>() == 8);
    assert!(PivConfig::SPACE == 1014 && GuardianReward::SPACE == 84);

    let config_boxes = 4 * size_of::<PivConfig>();
    let state_envelopes = 2 * PivConfig::SPACE + 2 * GuardianReward::SPACE;
    let entrypoint_accounts = 5 * size_of::<AccountInfo<'static>>();
    // Pinned target Rust RcInner is repr(C): two usize Cell headers then value.
    // These public RefCell layout assertions bind its native/slice payloads.
    let rc_header = 2 * size_of::<usize>();
    let account_refs = 5 * (rc_header + size_of::<RefCell<&'static mut u64>>())
        + 5 * (rc_header + size_of::<RefCell<&'static mut [u8]>>());
    // Pinned System transfer uses two metas and bincode u32 variant + u64 amount.
    let transfer_instruction = 2 * size_of::<AccountMeta>() + 12;
    let event_buffer = 256;
    let requested = config_boxes + state_envelopes + entrypoint_accounts
        + account_refs + transfer_instruction + event_buffer;
    assert!(requested == 7228);
    let allocation_count = 4 + 4 + 1 + 10 + 2 + 1;
    assert!(allocation_count == 22);
    // Every listed alignment is <= 8. Charge seven bytes to every allocation,
    // including byte-aligned buffers, plus the allocator's initial usize cursor.
    let conservative_inventory = requested + allocation_count * 7 + size_of::<usize>();
    assert!(conservative_inventory == 7390);
    assert!(conservative_inventory <= 8192);
};
