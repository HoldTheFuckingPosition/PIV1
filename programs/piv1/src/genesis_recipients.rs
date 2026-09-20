//! Read-only fresh genesis preflight with two temporary recipient vault identities.
//!
//! A funded vault PDA of the authenticated governance multisig is not proof of
//! exclusive four-of-six spending: delegated spending limits, stale transactions
//! and the live Squads artifact remain separate checks. No funding, persistence,
//! instruction selector or complete initializer is supplied. These two additional
//! AccountInfos need new transport evidence; Task 2.23 covers its old template.

use std::rc::Rc;
use anchor_lang::{
    prelude::{AccountInfo, Clock, Pubkey, Rent, SolanaSysvar},
    solana_program::{instruction::get_stack_height, program_error::ProgramError, system_program},
};
use crate::{
    accounts::rent_floor,
    errors::Piv1Error,
    genesis_preflight::{self, GenesisAccountPreflight, GenesisPreflightError, GenesisPreflightRoles},
    squads_accounts::SQUADS_V4_PROGRAM_ID,
};

/// Trusted account roles plus derivation witnesses for the exact approved keys.
/// Vault indices are not new approved parameters and grant no independent power.
#[derive(Clone, Copy, Debug)]
pub struct GenesisRecipientRoles {
    pub preflight: GenesisPreflightRoles,
    pub htfp_recipient: usize,
    pub team_owner_recipient: usize,
    pub htfp_vault_index: u8,
    pub team_owner_vault_index: u8,
}

/// Recipient witnesses without a second, potentially conflicting genesis mapping.
#[derive(Clone, Copy, Debug)]
pub struct GenesisRecipientSelection {
    pub htfp_recipient: usize,
    pub team_owner_recipient: usize,
    pub htfp_vault_index: u8,
    pub team_owner_vault_index: u8,
}
impl GenesisRecipientSelection {
    fn roles(self, preflight: GenesisPreflightRoles) -> GenesisRecipientRoles {
        GenesisRecipientRoles { preflight, htfp_recipient: self.htfp_recipient,
            team_owner_recipient: self.team_owner_recipient, htfp_vault_index: self.htfp_vault_index,
            team_owner_vault_index: self.team_owner_vault_index }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum GenesisRecipientError {
    Preflight(GenesisPreflightError),
    State(Piv1Error),
    HostRuntimeUnavailable,
    InvalidRoles,
    UnapprovedRecipient,
    InvalidRecipientVault,
    UnfundedRecipient,
    ObservationMismatch,
}
impl From<GenesisPreflightError> for GenesisRecipientError {
    fn from(error: GenesisPreflightError) -> Self { Self::Preflight(error) }
}
impl From<Piv1Error> for GenesisRecipientError {
    fn from(error: Piv1Error) -> Self { Self::State(error) }
}
pub type GenesisRecipientResult<T> = Result<T, GenesisRecipientError>;

/// Point-invocation identity and funding observation, not a control capability.
#[derive(Debug, Eq, PartialEq)]
pub struct GenesisRecipientObservation {
    address: Pubkey,
    multisig: Pubkey,
    vault_index: u8,
    bump: u8,
    observed_lamports: u64,
    rent_minimum: u64,
}
impl GenesisRecipientObservation {
    pub fn address(&self) -> Pubkey { self.address }
    pub fn multisig(&self) -> Pubkey { self.multisig }
    pub fn vault_index(&self) -> u8 { self.vault_index }
    pub fn bump(&self) -> u8 { self.bump }
    pub fn observed_lamports(&self) -> u64 { self.observed_lamports }
    pub fn rent_minimum(&self) -> u64 { self.rent_minimum }
}

/// Private, non-Clone and nonserialized observations. No public constructor
/// accepts detached preflight/recipient evidence or authorizes subsequent effects.
#[must_use]
#[derive(Debug, PartialEq)]
pub struct GenesisRecipientPreflight {
    genesis: GenesisAccountPreflight,
    recipients: [GenesisRecipientObservation; 2],
}
impl GenesisRecipientPreflight {
    pub fn genesis(&self) -> &GenesisAccountPreflight { &self.genesis }
    pub fn htfp_recipient(&self) -> &GenesisRecipientObservation { &self.recipients[0] }
    pub fn team_owner_recipient(&self) -> &GenesisRecipientObservation { &self.recipients[1] }
}

/// Refresh the complete exact-approved-message preflight and both identities.
/// Ordinary host calls reject before reading accounts; no native selector exists.
pub fn preflight_approved_genesis_recipients(
    program: &Pubkey, accounts: &[AccountInfo<'_>], data: &[u8], roles: GenesisRecipientRoles,
) -> GenesisRecipientResult<GenesisRecipientPreflight> {
    dispatch(program, accounts, data, roles, cfg!(target_os = "solana"), get_stack_height, Clock::get, Rent::get)
}

/// Explicit synthetic runtime context, absent from Solana and native dispatch.
#[cfg(not(target_os = "solana"))]
pub fn preflight_approved_genesis_recipients_with_host_context(
    program: &Pubkey, accounts: &[AccountInfo<'_>], data: &[u8], roles: GenesisRecipientRoles,
    context: crate::squads_execution::ModeledSquadsInvocationContext,
) -> GenesisRecipientResult<GenesisRecipientPreflight> {
    dispatch(program, accounts, data, roles, true, || context.stack_height,
        || Ok(context.clock), || Ok(context.rent))
}

#[allow(clippy::too_many_arguments)]
fn dispatch(
    program: &Pubkey, accounts: &[AccountInfo<'_>], data: &[u8], roles: GenesisRecipientRoles,
    available: bool, height: impl FnOnce() -> usize,
    clock: impl FnOnce() -> Result<Clock, ProgramError>, rent: impl FnOnce() -> Result<Rent, ProgramError>,
) -> GenesisRecipientResult<GenesisRecipientPreflight> {
    if !available { return Err(GenesisRecipientError::HostRuntimeUnavailable); }
    validate_roles_and_backing(accounts, roles)?;
    let mut trusted_rent = None;
    let genesis = genesis_preflight::dispatch(program, accounts, data, roles.preflight, true,
        height, clock, || { let value = rent()?; trusted_rent = Some(value.clone()); Ok(value) })?;
    let rent = trusted_rent.ok_or(Piv1Error::InvalidRent)?;
    let recipients = observe_after_preflight(accounts, &genesis, roles, &rent)?;
    Ok(GenesisRecipientPreflight { genesis, recipients })
}

fn observe_after_preflight(
    accounts: &[AccountInfo<'_>], genesis: &GenesisAccountPreflight, roles: GenesisRecipientRoles, rent: &Rent,
) -> GenesisRecipientResult<[GenesisRecipientObservation; 2]> {
    let floor = rent_floor(rent, 0)?;
    // The complete bootstrap preflight authenticated this multisig role freshly.
    let multisig = *accounts[roles.preflight.bootstrap.multisig].key;
    let model = genesis.model(); let config = model.proposed_config();
    let observe = |index, approved, vault_index| {
        let account: &AccountInfo<'_> = &accounts[index];
        if *account.key == model.piv_authority() || model.proposed_registry().guardian_keys.contains(account.key) {
            return Err(Piv1Error::AccountAlias.into());
        }
        if *account.key != approved { return Err(GenesisRecipientError::UnapprovedRecipient); }
        let (expected, bump) = Pubkey::find_program_address(
            &[b"multisig", multisig.as_ref(), b"vault", &[vault_index]], &SQUADS_V4_PROGRAM_ID);
        if *account.key != expected { return Err(GenesisRecipientError::InvalidRecipientVault); }
        if *account.owner != system_program::ID { return Err(Piv1Error::InvalidAccountOwner.into()); }
        if account.executable { return Err(Piv1Error::ExecutableAccount.into()); }
        if !account.try_borrow_data().map_err(|_| Piv1Error::AccountBorrowFailed)?.is_empty() {
            return Err(Piv1Error::InvalidAccountSize.into());
        }
        let lamports = **account.try_borrow_lamports().map_err(|_| Piv1Error::AccountBorrowFailed)?;
        if lamports == 0 { return Err(GenesisRecipientError::UnfundedRecipient); }
        if lamports < floor { return Err(Piv1Error::AccountRentDeficit.into()); }
        Ok(GenesisRecipientObservation { address: expected, multisig, vault_index, bump,
            observed_lamports: lamports, rent_minimum: floor })
    };
    Ok([observe(roles.htfp_recipient, config.htfp_recipient, roles.htfp_vault_index)?,
        observe(roles.team_owner_recipient, config.team_owner_recipient, roles.team_owner_vault_index)?])
}

#[derive(Debug, PartialEq)]
struct RecipientMetadata { owner: Pubkey, data_len: usize, executable: bool, signer: bool, writable: bool }

/// Internal same-call facts only. No public constructor or effects interface.
#[derive(Debug, PartialEq)]
pub(crate) struct RecipientExecutionObservation {
    indices: [usize; 2],
    identities: [GenesisRecipientObservation; 2],
    metadata: [RecipientMetadata; 2],
}
impl RecipientExecutionObservation {
    pub(crate) fn verify(&self, accounts: &[AccountInfo<'_>]) -> GenesisRecipientResult<()> {
        for slot in 0..2 {
            let account = accounts.get(self.indices[slot]).ok_or(GenesisRecipientError::InvalidRoles)?;
            let before = &self.metadata[slot];
            if *account.key != self.identities[slot].address || *account.owner != before.owner
                || account.executable != before.executable || account.is_signer != before.signer
                || account.is_writable != before.writable
            { return Err(GenesisRecipientError::ObservationMismatch); }
            if account.try_borrow_data().map_err(|_| Piv1Error::AccountBorrowFailed)?.len() != before.data_len
                || **account.try_borrow_lamports().map_err(|_| Piv1Error::AccountBorrowFailed)? != self.identities[slot].observed_lamports
            { return Err(GenesisRecipientError::ObservationMismatch); }
        }
        Ok(())
    }
}

/// Called only by allocation after its own fresh full genesis authentication.
/// The caller supplies its one authoritative role mapping and retained Rent.
pub(crate) fn observe_for_execution(
    accounts: &[AccountInfo<'_>], genesis: &GenesisAccountPreflight, preflight_roles: GenesisPreflightRoles,
    selection: GenesisRecipientSelection, additional_roles: [usize; 3], rent: &Rent,
) -> GenesisRecipientResult<RecipientExecutionObservation> {
    let roles = selection.roles(preflight_roles);
    let indices = [roles.htfp_recipient, roles.team_owner_recipient];
    if indices.iter().any(|index| additional_roles.contains(index)) { return Err(GenesisRecipientError::InvalidRoles); }
    validate_roles_and_backing(accounts, roles)?;
    let identities = observe_after_preflight(accounts, genesis, roles, rent)?;
    let metadata = indices.map(|index| {
        let account = &accounts[index];
        RecipientMetadata { owner: *account.owner, data_len: 0, executable: account.executable,
            signer: account.is_signer, writable: account.is_writable }
    });
    Ok(RecipientExecutionObservation { indices, identities, metadata })
}

fn validate_roles_and_backing(accounts: &[AccountInfo<'_>], roles: GenesisRecipientRoles)
    -> GenesisRecipientResult<()>
{
    let b = roles.preflight.bootstrap; let p = roles.preflight.protocol;
    let protected = [b.program,b.program_data,b.multisig,b.proposal,b.transaction,b.vault,b.instructions,b.config,
        p.program,p.pool,p.validator_list,p.reserve,p.mint,p.manager_fee,p.referrer];
    if roles.htfp_recipient == roles.team_owner_recipient { return Err(GenesisRecipientError::InvalidRoles); }
    for index in [roles.htfp_recipient, roles.team_owner_recipient] {
        if index >= accounts.len() || protected.contains(&index) || roles.preflight.targets.contains(&index) {
            return Err(GenesisRecipientError::InvalidRoles);
        }
        let account = &accounts[index];
        for (other_index, other) in accounts.iter().enumerate() {
            if index != other_index && (account.key == other.key || Rc::ptr_eq(&account.data, &other.data)
                || Rc::ptr_eq(&account.lamports, &other.lamports))
            { return Err(Piv1Error::AccountAlias.into()); }
        }
    }
    Ok(())
}
