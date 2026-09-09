//! Actual-account pending recognition. No transfer, CPI, event, integration or
//! external callback occurs. Canonical persistence is the final fallible action;
//! every allocation, observation and transition is prepared before its sole copy.

use anchor_lang::prelude::{Pubkey, Rent};
use crate::{
    errors::{Piv1Error, Piv1Result},
    pending_accounts::{authenticate_pending_accounts, PendingAccountInfos},
    state::{reconcile_pending_contributions, PendingReconciliationResult},
    state_persistence::{commit_state_writes, PreparedStateWrite, StateEnvelope},
};

#[inline(never)]
pub fn execute_pending_reconciliation(program: &Pubkey, rent: &Rent,
    accounts: PendingAccountInfos<'_, '_>) -> Piv1Result<PendingReconciliationResult> {
    if !accounts.config.is_writable { return Err(Piv1Error::AccountNotWritable); }
    let before = authenticate_pending_accounts(program, rent, accounts)?;
    let mut next = Box::new(before.config().clone());
    let result = reconcile_pending_contributions(&mut next, before.distribution(), before.observation())?;
    let write = PreparedStateWrite::new(program, *accounts.config.key,
        StateEnvelope::config(before.config())?, StateEnvelope::config(&next)?)?;
    // No external interaction can change observations during this synchronous
    // boundary. Shared data aliases were rejected. Only Config data is writable
    // here; all full pre-state bytes are checked again by the existing committer.
    commit_state_writes(program, rent, [(&write, accounts.config)])?;
    Ok(result)
}
