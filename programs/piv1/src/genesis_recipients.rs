//! Exact approved beneficiary addresses, independent of destination account type.

use std::rc::Rc;
use anchor_lang::{
    prelude::{AccountInfo, Clock, Pubkey, Rent, SolanaSysvar},
    solana_program::{instruction::get_stack_height, program_error::ProgramError},
};
use crate::{
    errors::Piv1Error,
    genesis_preflight::{self, GenesisAccountPreflight, GenesisPreflightError, GenesisPreflightRoles},
};

/// Trusted account roles for the exact approved beneficiary addresses.
/// Vault indices are not new approved parameters and grant no independent power.
#[derive(Clone, Copy, Debug)]
pub struct GenesisRecipientRoles {
    pub preflight: GenesisPreflightRoles,
    pub htfp_recipient: usize,
    pub team_owner_recipient: usize,
}

/// Recipient roles without a second, potentially conflicting genesis mapping.
#[derive(Clone, Copy, Debug)]
pub struct GenesisRecipientSelection {
    pub htfp_recipient: usize,
    pub team_owner_recipient: usize,
}
impl GenesisRecipientSelection {
    fn roles(self, preflight: GenesisPreflightRoles) -> GenesisRecipientRoles {
        GenesisRecipientRoles { preflight, htfp_recipient: self.htfp_recipient,
            team_owner_recipient: self.team_owner_recipient }
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
    observed_lamports: u64,
}
impl GenesisRecipientObservation {
    pub fn address(&self) -> Pubkey { self.address }
    pub fn observed_lamports(&self) -> u64 { self.observed_lamports }
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
    accounts: &[AccountInfo<'_>], genesis: &GenesisAccountPreflight, roles: GenesisRecipientRoles, _rent: &Rent,
) -> GenesisRecipientResult<[GenesisRecipientObservation; 2]> {
    let model = genesis.model(); let config = model.proposed_config();
    let observe = |index, approved| {
        let account: &AccountInfo<'_> = &accounts[index];
        if *account.key == model.piv_authority() || *account.owner == model.program() { return Err(Piv1Error::AccountAlias.into()); }
        if *account.key != approved { return Err(GenesisRecipientError::UnapprovedRecipient); }
        let lamports = **account.try_borrow_lamports().map_err(|_| Piv1Error::AccountBorrowFailed)?;
        Ok(GenesisRecipientObservation { address: approved, observed_lamports: lamports })
    };
    Ok([observe(roles.htfp_recipient, config.htfp_recipient)?,
        observe(roles.team_owner_recipient, config.team_owner_recipient)?])
}

#[derive(Debug, PartialEq)]
struct RecipientMetadata { owner: Pubkey, data_len: usize, data_hash: [u8; 32], rent_epoch: u64, executable: bool, signer: bool, writable: bool }

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
                || account.is_writable != before.writable || account.rent_epoch != before.rent_epoch
            { return Err(GenesisRecipientError::ObservationMismatch); }
            let data = account.try_borrow_data().map_err(|_| Piv1Error::AccountBorrowFailed)?;
            if data.len() != before.data_len || solana_sha256_hasher::hash(&data).to_bytes() != before.data_hash
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
    let snapshot = |index: usize| -> GenesisRecipientResult<RecipientMetadata> {
        let account = &accounts[index];
        let data = account.try_borrow_data().map_err(|_| Piv1Error::AccountBorrowFailed)?;
        Ok(RecipientMetadata { owner: *account.owner, data_len: data.len(), data_hash: solana_sha256_hasher::hash(&data).to_bytes(),
            rent_epoch: account.rent_epoch, executable: account.executable, signer: account.is_signer, writable: account.is_writable })
    };
    let metadata = [snapshot(indices[0])?, snapshot(indices[1])?];
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
