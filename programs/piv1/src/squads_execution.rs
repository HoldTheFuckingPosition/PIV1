//! Bounded current-governance and preinitialization Squads authorization.
//!
//! A trusted handler supplies its actual entrypoint inputs and role indices.
//! Only a direct top-level Squads VaultTransactionExecute containing one PIV1
//! instruction is supported: no batch, lookup tables, ephemeral signers, stale
//! configuration, initializer transport, or guardian-rotation choreography.
//! The separate bootstrap boundary requires a virgin Config PDA and returns
//! current Squads member order, without reading initialized PIV1 guardian state.
//! No handler dispatch or effects are implemented. The returned evidence is
//! point-invocation only, not a reusable capability or durable replay receipt.
//!
//! Pinned Squads v4: 64af7330413d5c85cbbccfd8c27a05d45b6e666f. During its CPI,
//! Proposal bytes remain Approved and VaultTransaction bytes remain intact:
//! .take() changes only Anchor's owned value. Executed is persisted after the
//! successful outer instruction. Source/host evidence does not prove deployed
//! artifact identity, signature verification, runtime rollback, or effect-once.

use anchor_lang::{
    prelude::{AccountInfo, Clock, Pubkey, Rent, SolanaSysvar},
    solana_program::{instruction::get_stack_height, program_error::ProgramError, system_program, sysvar},
};
use crate::{
    accounts::seeds,
    errors::Piv1Error,
    guardian_clock_accounts::{authenticate_guardian_clock_snapshot, GuardianClockAccountInfos},
    squads_accounts::{authenticate_squads_authority_snapshot, SquadsAuthorityAccountInfos,
        AuthenticatedSquadsAuthoritySnapshot, SquadsMultisigConfiguration, SQUADS_V4_PROGRAM_ID},
};

pub const VAULT_TRANSACTION_EXECUTE_DISCRIMINATOR: [u8; 8] = [194, 8, 161, 87, 153, 164, 25, 171];
pub const PROPOSAL_DISCRIMINATOR: [u8; 8] = [26, 94, 189, 187, 116, 136, 53, 33];
pub const VAULT_TRANSACTION_DISCRIMINATOR: [u8; 8] = [168, 250, 162, 100, 81, 14, 162, 207];

/// Library-only failures. No existing instruction error mapping is changed.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum SquadsExecutionError {
    HostRuntimeUnavailable,
    Runtime(ProgramError),
    State(Piv1Error),
    InvalidInvocation,
    InvalidInstructionsSysvar,
    InvalidProposal,
    InvalidTransaction,
    UnsupportedMessage,
    MessageMismatch,
    TimelockNotReleased,
}
impl From<Piv1Error> for SquadsExecutionError {
    fn from(error: Piv1Error) -> Self { Self::State(error) }
}
pub type SquadsExecutionResult<T> = Result<T, SquadsExecutionError>;

/// Trusted handler mapping into the actual entrypoint account slice. These are
/// library indices, not a serialized ABI or caller-selected instruction context.
#[derive(Clone, Copy, Debug)]
pub struct SquadsExecutionRoles {
    pub program: usize,
    pub program_data: usize,
    pub multisig: usize,
    pub proposal: usize,
    pub transaction: usize,
    pub vault: usize,
    pub instructions: usize,
    pub config: usize,
    pub guardian_registry: usize,
    pub rewards: [usize; 6],
    pub clock: usize,
}

/// Trusted handler roles for preinitialization. No registry or rewards exist at
/// this boundary; these indices must select actual entrypoint accounts.
#[derive(Clone, Copy, Debug)]
pub struct SquadsBootstrapRoles {
    pub program: usize,
    pub program_data: usize,
    pub multisig: usize,
    pub proposal: usize,
    pub transaction: usize,
    pub vault: usize,
    pub instructions: usize,
    pub config: usize,
}

/// Private shared roles have no initialized or bootstrap state interpretation.
#[derive(Clone, Copy)]
struct SquadsRoles {
    program: usize, program_data: usize, multisig: usize, proposal: usize,
    transaction: usize, vault: usize, instructions: usize,
}

#[derive(Debug, Eq, PartialEq)]
struct SquadsAction {
    program: Pubkey, multisig: Pubkey, proposal: Pubkey, transaction: Pubkey,
    vault: Pubkey, transaction_index: u64, top_level_index: u16,
    current_members: [Pubkey; 6], approved_member_bitmap: u8,
}

/// Non-Clone point-invocation evidence while Config remains virgin. This is not
/// existing-governance evidence, a parameter-semantics check or a replay receipt.
/// Reauthenticate after any mutation/CPI. Repeated virgin checks may succeed.
#[derive(Debug, Eq, PartialEq)]
pub struct AuthenticatedSquadsBootstrapInvocation {
    action: SquadsAction,
    config: Pubkey,
}
impl AuthenticatedSquadsBootstrapInvocation {
    pub fn program(&self) -> Pubkey { self.action.program }
    pub fn multisig(&self) -> Pubkey { self.action.multisig }
    pub fn proposal(&self) -> Pubkey { self.action.proposal }
    pub fn transaction(&self) -> Pubkey { self.action.transaction }
    pub fn vault(&self) -> Pubkey { self.action.vault }
    pub fn config(&self) -> Pubkey { self.config }
    pub fn transaction_index(&self) -> u64 { self.action.transaction_index }
    pub fn top_level_index(&self) -> u16 { self.action.top_level_index }
    /// Canonical sorted current Squads members; no PIV1 slots are assigned.
    pub fn current_members(&self) -> &[Pubkey; 6] { &self.action.current_members }
    /// Bits index current_members(), not an initialized guardian registry.
    pub fn approved_member_bitmap(&self) -> u8 { self.action.approved_member_bitmap }
}

/// Non-Clone point-invocation evidence. Reauthenticate before a later action;
/// handlers still enforce their own state transitions and atomic effects.
#[derive(Debug, Eq, PartialEq)]
pub struct AuthenticatedSquadsInvocation {
    program: Pubkey,
    multisig: Pubkey,
    proposal: Pubkey,
    transaction: Pubkey,
    vault: Pubkey,
    transaction_index: u64,
    top_level_index: u16,
    approved_guardian_bitmap: u8,
    guardian_registry_revision: u64,
}
impl AuthenticatedSquadsInvocation {
    pub fn program(&self) -> Pubkey { self.program }
    pub fn multisig(&self) -> Pubkey { self.multisig }
    pub fn proposal(&self) -> Pubkey { self.proposal }
    pub fn transaction(&self) -> Pubkey { self.transaction }
    pub fn vault(&self) -> Pubkey { self.vault }
    pub fn transaction_index(&self) -> u64 { self.transaction_index }
    pub fn top_level_index(&self) -> u16 { self.top_level_index }
    /// Bitmap in the existing PIV1 registry's slot order, not Squads sort order.
    pub fn approved_guardian_bitmap(&self) -> u8 { self.approved_guardian_bitmap }
    pub fn guardian_registry_revision(&self) -> u64 { self.guardian_registry_revision }
}

/// Runtime-only authorization using actual entrypoint values and fresh syscalls.
/// Ordinary host calls reject before obtaining context or authenticating accounts.
pub fn authenticate_squads_invocation(
    program: &Pubkey,
    accounts: &[AccountInfo<'_>],
    instruction_data: &[u8],
    roles: SquadsExecutionRoles,
    vault_index: u8,
) -> SquadsExecutionResult<AuthenticatedSquadsInvocation> {
    dispatch(program, accounts, instruction_data, roles, vault_index,
        cfg!(target_os = "solana"), get_stack_height, Clock::get, Rent::get)
}

/// Runtime-only preinitialization authorization with actual entrypoint inputs.
/// Empty Config data must be System-owned, nonexecutable and writable at its
/// canonical PDA. Lamports are neither read nor classified as economic funding.
pub fn authenticate_squads_bootstrap_invocation(
    program: &Pubkey, accounts: &[AccountInfo<'_>], instruction_data: &[u8],
    roles: SquadsBootstrapRoles, vault_index: u8,
) -> SquadsExecutionResult<AuthenticatedSquadsBootstrapInvocation> {
    dispatch_bootstrap(program, accounts, instruction_data, roles, vault_index,
        cfg!(target_os = "solana"), get_stack_height, Clock::get, Rent::get)
}

/// Explicit host modeling input; absent from the Solana build and not exposed
/// through any entrypoint selector. These fields are not runtime evidence.
#[cfg(not(target_os = "solana"))]
pub struct ModeledSquadsInvocationContext {
    pub stack_height: usize,
    pub clock: Clock,
    pub rent: Rent,
}

#[cfg(not(target_os = "solana"))]
pub fn authenticate_squads_invocation_with_host_context(
    program: &Pubkey,
    accounts: &[AccountInfo<'_>],
    instruction_data: &[u8],
    roles: SquadsExecutionRoles,
    vault_index: u8,
    context: ModeledSquadsInvocationContext,
) -> SquadsExecutionResult<AuthenticatedSquadsInvocation> {
    dispatch(program, accounts, instruction_data, roles, vault_index, true,
        || context.stack_height, || Ok(context.clock), || Ok(context.rent))
}

/// Explicit synthetic host context, never an onchain instruction selector.
#[cfg(not(target_os = "solana"))]
pub fn authenticate_squads_bootstrap_invocation_with_host_context(
    program: &Pubkey, accounts: &[AccountInfo<'_>], instruction_data: &[u8],
    roles: SquadsBootstrapRoles, vault_index: u8, context: ModeledSquadsInvocationContext,
) -> SquadsExecutionResult<AuthenticatedSquadsBootstrapInvocation> {
    dispatch_bootstrap(program, accounts, instruction_data, roles, vault_index, true,
        || context.stack_height, || Ok(context.clock), || Ok(context.rent))
}

fn runtime_context(
    available: bool, stack_height: impl FnOnce() -> usize,
    clock: impl FnOnce() -> Result<Clock, ProgramError>,
    rent: impl FnOnce() -> Result<Rent, ProgramError>,
) -> SquadsExecutionResult<(Clock, Rent)> {
    if !available { return Err(SquadsExecutionError::HostRuntimeUnavailable); }
    if stack_height() != 2 { return Err(SquadsExecutionError::InvalidInvocation); }
    Ok((clock().map_err(SquadsExecutionError::Runtime)?, rent().map_err(SquadsExecutionError::Runtime)?))
}

fn dispatch(
    program: &Pubkey, accounts: &[AccountInfo<'_>], instruction_data: &[u8],
    roles: SquadsExecutionRoles, vault_index: u8, available: bool,
    stack_height: impl FnOnce() -> usize,
    clock: impl FnOnce() -> Result<Clock, ProgramError>,
    rent: impl FnOnce() -> Result<Rent, ProgramError>,
) -> SquadsExecutionResult<AuthenticatedSquadsInvocation> {
    let (clock, rent) = runtime_context(available, stack_height, clock, rent)?;
    validate_invocation(program, accounts, instruction_data, roles, vault_index, &clock, &rent)
}

/// Trusted internal composition seam. Callers obtain runtime context themselves;
/// this is not a public production API for injected invocation context.
pub(crate) fn dispatch_bootstrap(
    program: &Pubkey, accounts: &[AccountInfo<'_>], instruction_data: &[u8],
    roles: SquadsBootstrapRoles, vault_index: u8, available: bool,
    stack_height: impl FnOnce() -> usize,
    clock: impl FnOnce() -> Result<Clock, ProgramError>,
    rent: impl FnOnce() -> Result<Rent, ProgramError>,
) -> SquadsExecutionResult<AuthenticatedSquadsBootstrapInvocation> {
    let (clock, _rent) = runtime_context(available, stack_height, clock, rent)?;
    validate_accounts_and_roles(accounts, &[roles.program, roles.program_data, roles.multisig,
        roles.proposal, roles.transaction, roles.vault, roles.instructions, roles.config])?;
    let config = &accounts[roles.config];
    let (expected, _) = Pubkey::find_program_address(&[seeds::CONFIG], program);
    if *config.key != expected { return Err(Piv1Error::InvalidAccountPda.into()); }
    if config.owner != &system_program::ID { return Err(Piv1Error::InvalidAccountOwner.into()); }
    if config.executable { return Err(Piv1Error::ExecutableAccount.into()); }
    if !config.is_writable { return Err(Piv1Error::AccountNotWritable.into()); }
    if !config.try_borrow_data().map_err(|_| Piv1Error::AccountBorrowFailed)?.is_empty() {
        return Err(Piv1Error::InvalidAccountSize.into());
    }
    let shared = SquadsRoles { program: roles.program, program_data: roles.program_data,
        multisig: roles.multisig, proposal: roles.proposal, transaction: roles.transaction,
        vault: roles.vault, instructions: roles.instructions };
    let authority = authenticate_authority(program, accounts, shared, vault_index)?;
    let action = validate_squads_action(program, accounts, instruction_data, shared, vault_index, &clock, &authority)?;
    Ok(AuthenticatedSquadsBootstrapInvocation { action, config: *config.key })
}

fn validate_accounts_and_roles(accounts: &[AccountInfo<'_>], indices: &[usize]) -> SquadsExecutionResult<()> {
    if accounts.is_empty() || accounts.len() > 256 { return Err(SquadsExecutionError::InvalidInvocation); }
    for (index, account) in accounts.iter().enumerate() {
        if accounts[index + 1..].iter().any(|other| account.key == other.key) {
            return Err(Piv1Error::AccountAlias.into());
        }
    }
    for (index, role) in indices.iter().enumerate() {
        if *role >= accounts.len() || indices[index + 1..].contains(role) {
            return Err(SquadsExecutionError::InvalidInvocation);
        }
    }
    Ok(())
}

fn authenticate_authority(
    program: &Pubkey, accounts: &[AccountInfo<'_>], roles: SquadsRoles, vault_index: u8,
) -> SquadsExecutionResult<AuthenticatedSquadsAuthoritySnapshot> {
    // Private callers validate every role before indexing actual entrypoint inputs.
    Ok(authenticate_squads_authority_snapshot(program, vault_index, SquadsAuthorityAccountInfos {
        program: &accounts[roles.program], program_data: &accounts[roles.program_data],
        multisig: &accounts[roles.multisig] })?)
}

#[inline(never)]
fn validate_invocation(
    program: &Pubkey, accounts: &[AccountInfo<'_>], instruction_data: &[u8],
    roles: SquadsExecutionRoles, vault_index: u8, clock: &Clock, rent: &Rent,
) -> SquadsExecutionResult<AuthenticatedSquadsInvocation> {
    let indices = [roles.program, roles.program_data, roles.multisig, roles.proposal,
        roles.transaction, roles.vault, roles.instructions, roles.config,
        roles.guardian_registry, roles.rewards[0], roles.rewards[1], roles.rewards[2],
        roles.rewards[3], roles.rewards[4], roles.rewards[5], roles.clock];
    validate_accounts_and_roles(accounts, &indices)?;
    // Every index was bounded above; no detached AccountInfo can enter these reads.
    let at = |index: usize| &accounts[index];
    let shared = SquadsRoles { program: roles.program, program_data: roles.program_data,
        multisig: roles.multisig, proposal: roles.proposal, transaction: roles.transaction,
        vault: roles.vault, instructions: roles.instructions };
    let authority = authenticate_authority(program, accounts, shared, vault_index)?;
    let guardians = authenticate_guardian_clock_snapshot(program, rent, GuardianClockAccountInfos {
        config: at(roles.config), guardian_registry: at(roles.guardian_registry),
        rewards: roles.rewards.map(at), clock: at(roles.clock),
    })?;
    authority.validate_guardian_correspondence(&guardians)?;
    if guardians.clock() != clock { return Err(SquadsExecutionError::InvalidInvocation); }
    let action = validate_squads_action(program, accounts, instruction_data, shared, vault_index, clock, &authority)?;
    let mut approved_guardian_bitmap = 0_u8;
    for (slot, key) in guardians.registry().guardian_keys.iter().enumerate() {
        let sorted_index = action.current_members.iter().position(|member| member == key)
            .ok_or(SquadsExecutionError::InvalidProposal)?;
        if action.approved_member_bitmap & (1 << sorted_index) != 0 { approved_guardian_bitmap |= 1 << slot; }
    }
    Ok(AuthenticatedSquadsInvocation {
        program: action.program, multisig: action.multisig, proposal: action.proposal,
        transaction: action.transaction, vault: action.vault, transaction_index: action.transaction_index,
        top_level_index: action.top_level_index, approved_guardian_bitmap,
        guardian_registry_revision: guardians.registry().revision,
    })
}

#[inline(never)]
fn validate_squads_action(
    program: &Pubkey, accounts: &[AccountInfo<'_>], instruction_data: &[u8],
    roles: SquadsRoles, vault_index: u8, clock: &Clock, authority: &AuthenticatedSquadsAuthoritySnapshot,
) -> SquadsExecutionResult<SquadsAction> {
    let at = |index: usize| &accounts[index];
    let configuration = authority.configuration();
    if *at(roles.vault).key != authority.vault() || !at(roles.vault).is_signer
    {
        return Err(SquadsExecutionError::InvalidInvocation);
    }
    for account in [at(roles.proposal), at(roles.transaction)] {
        if account.owner != &SQUADS_V4_PROGRAM_ID { return Err(Piv1Error::InvalidAccountOwner.into()); }
        if account.executable { return Err(Piv1Error::ExecutableAccount.into()); }
    }
    let proposal_data = at(roles.proposal).try_borrow_data().map_err(|_| Piv1Error::AccountBorrowFailed)?;
    let (index, approvals) = validate_proposal(&proposal_data, at(roles.proposal).key,
        at(roles.multisig).key, configuration, clock.unix_timestamp)?;
    let transaction_data = at(roles.transaction).try_borrow_data().map_err(|_| Piv1Error::AccountBorrowFailed)?;
    let message = validate_transaction(&transaction_data, at(roles.transaction).key,
        at(roles.multisig).key, &authority.vault(), vault_index, index, program, accounts, instruction_data)?;
    let top_level_index = validate_outer(at(roles.instructions), &message,
        [at(roles.multisig).key, at(roles.proposal).key, at(roles.transaction).key],
        &authority.vault(), configuration)?;
    // The pinned caller protects its proposal against writable inner metas. Keep
    // all governance metadata readonly in this direct, single-instruction profile.
    if [roles.multisig, roles.proposal, roles.transaction].iter().any(|index| accounts[*index].is_writable) {
        return Err(SquadsExecutionError::InvalidInvocation);
    }
    Ok(SquadsAction {
        program: *program, multisig: authority.multisig(), proposal: *at(roles.proposal).key,
        transaction: *at(roles.transaction).key, vault: authority.vault(), transaction_index: index,
        top_level_index, current_members: *configuration.member_keys(), approved_member_bitmap: approvals,
    })
}

fn validate_proposal(
    bytes: &[u8], address: &Pubkey, multisig: &Pubkey,
    configuration: &SquadsMultisigConfiguration, now: i64,
) -> SquadsExecutionResult<(u64, u8)> {
    let mut reader = Reader(bytes);
    if reader.array::<8>()? != PROPOSAL_DISCRIMINATOR || reader.key()? != *multisig {
        return Err(SquadsExecutionError::InvalidProposal);
    }
    let index = reader.u64()?;
    if index == 0 || index <= configuration.stale_transaction_index() || index > configuration.transaction_index()
        || reader.byte()? != 3
    {
        return Err(SquadsExecutionError::InvalidProposal);
    }
    let timestamp = i64::from_le_bytes(reader.array()?);
    let bump = reader.byte()?;
    let (expected, expected_bump) = Pubkey::find_program_address(
        &[b"multisig", multisig.as_ref(), b"transaction", &index.to_le_bytes(), b"proposal"], &SQUADS_V4_PROGRAM_ID);
    if *address != expected || bump != expected_bump { return Err(SquadsExecutionError::InvalidProposal); }
    let approved = votes(&mut reader, configuration)?;
    let rejected = votes(&mut reader, configuration)?;
    let cancelled = votes(&mut reader, configuration)?;
    if approved.count_ones() < 4 || approved & rejected != 0 || rejected.count_ones() >= 3 || cancelled.count_ones() >= 4 {
        return Err(SquadsExecutionError::InvalidProposal);
    }
    let elapsed = now.checked_sub(timestamp).ok_or(SquadsExecutionError::TimelockNotReleased)?;
    if elapsed < i64::from(configuration.time_lock()) { return Err(SquadsExecutionError::TimelockNotReleased); }
    // Proposal accounts reserve all vote vectors; their unused tail may be stale.
    Ok((index, approved))
}

fn votes(reader: &mut Reader<'_>, configuration: &SquadsMultisigConfiguration) -> SquadsExecutionResult<u8> {
    let count = reader.u32()?;
    if count > 6 { return Err(SquadsExecutionError::InvalidProposal); }
    let mut bitmap = 0_u8;
    let mut previous = None;
    for _ in 0..count {
        let key = reader.key()?;
        if previous.is_some_and(|old| key <= old) { return Err(SquadsExecutionError::InvalidProposal); }
        let index = configuration.member_keys().iter().position(|member| *member == key)
            .ok_or(SquadsExecutionError::InvalidProposal)?;
        if configuration.member_permissions()[index] & 2 == 0 { return Err(SquadsExecutionError::InvalidProposal); }
        bitmap |= 1 << index;
        previous = Some(key);
    }
    Ok(bitmap)
}

struct Message<'a> {
    keys: &'a [u8],
    num_signers: usize,
    writable_signers: usize,
    writable_non_signers: usize,
}
impl Message<'_> {
    fn len(&self) -> usize { self.keys.len() / 32 }
    fn key(&self, index: usize) -> SquadsExecutionResult<Pubkey> {
        let start = index.checked_mul(32).ok_or(SquadsExecutionError::InvalidTransaction)?;
        let end = start.checked_add(32).ok_or(SquadsExecutionError::InvalidTransaction)?;
        Ok(Pubkey::new_from_array(self.keys.get(start..end).ok_or(SquadsExecutionError::InvalidTransaction)?
            .try_into().map_err(|_| SquadsExecutionError::InvalidTransaction)?))
    }
    fn flags(&self, index: usize) -> u8 {
        let signer = index < self.num_signers;
        let writable = index < self.writable_signers
            || (index >= self.num_signers && index - self.num_signers < self.writable_non_signers);
        u8::from(signer) | (u8::from(writable) << 1)
    }
}

#[inline(never)]
fn validate_transaction<'a>(
    bytes: &'a [u8], address: &Pubkey, multisig: &Pubkey, vault: &Pubkey, vault_index: u8,
    index: u64, program: &Pubkey, accounts: &[AccountInfo<'_>], data: &[u8],
) -> SquadsExecutionResult<Message<'a>> {
    let mut reader = Reader(bytes);
    if reader.array::<8>()? != VAULT_TRANSACTION_DISCRIMINATOR || reader.key()? != *multisig {
        return Err(SquadsExecutionError::InvalidTransaction);
    }
    let _creator = reader.key()?; // Historical creator is not current execution authority.
    if reader.u64()? != index { return Err(SquadsExecutionError::InvalidTransaction); }
    let bump = reader.byte()?;
    let stored_vault_index = reader.byte()?;
    let vault_bump = reader.byte()?;
    let (expected, expected_bump) = Pubkey::find_program_address(
        &[b"multisig", multisig.as_ref(), b"transaction", &index.to_le_bytes()], &SQUADS_V4_PROGRAM_ID);
    let (expected_vault, expected_vault_bump) = Pubkey::find_program_address(
        &[b"multisig", multisig.as_ref(), b"vault", &[vault_index]], &SQUADS_V4_PROGRAM_ID);
    if *address != expected || bump != expected_bump || stored_vault_index != vault_index
        || expected_vault != *vault || vault_bump != expected_vault_bump
    {
        return Err(SquadsExecutionError::InvalidTransaction);
    }
    if reader.u32()? != 0 { return Err(SquadsExecutionError::UnsupportedMessage); }
    let num_signers = usize::from(reader.byte()?);
    let writable_signers = usize::from(reader.byte()?);
    let writable_non_signers = usize::from(reader.byte()?);
    let key_count = usize::try_from(reader.u32()?).map_err(|_| SquadsExecutionError::UnsupportedMessage)?;
    if key_count == 0 || key_count > 256 || key_count > accounts.len() + 1
        || num_signers > key_count || writable_signers > num_signers
        || writable_non_signers > key_count - num_signers
    {
        return Err(SquadsExecutionError::UnsupportedMessage);
    }
    let message = Message { keys: reader.take(key_count * 32)?, num_signers, writable_signers, writable_non_signers };
    for i in 0..key_count {
        for j in 0..i {
            if message.key(i)? == message.key(j)? { return Err(SquadsExecutionError::UnsupportedMessage); }
        }
    }
    if reader.u32()? != 1 { return Err(SquadsExecutionError::UnsupportedMessage); }
    let program_index = usize::from(reader.byte()?);
    if message.key(program_index)? != *program { return Err(SquadsExecutionError::MessageMismatch); }
    let count = usize::try_from(reader.u32()?).map_err(|_| SquadsExecutionError::UnsupportedMessage)?;
    if count != accounts.len() { return Err(SquadsExecutionError::MessageMismatch); }
    let account_indices = reader.take(count)?;
    for (position, index) in account_indices.iter().enumerate() {
        if account_indices[..position].contains(index) { return Err(SquadsExecutionError::UnsupportedMessage); }
        let account = &accounts[position];
        let flags = u8::from(account.is_signer) | (u8::from(account.is_writable) << 1);
        if message.key(usize::from(*index))? != *account.key || message.flags(usize::from(*index)) != flags {
            return Err(SquadsExecutionError::MessageMismatch);
        }
    }
    for index in 0..key_count {
        if index != program_index && !account_indices.iter().any(|item| usize::from(*item) == index) {
            return Err(SquadsExecutionError::UnsupportedMessage);
        }
    }
    let data_len = usize::try_from(reader.u32()?).map_err(|_| SquadsExecutionError::InvalidTransaction)?;
    if data_len != data.len() || reader.take(data_len)? != data { return Err(SquadsExecutionError::MessageMismatch); }
    if reader.u32()? != 0 || !reader.0.is_empty() { return Err(SquadsExecutionError::UnsupportedMessage); }
    Ok(message)
}

fn validate_outer(
    account: &AccountInfo<'_>, message: &Message<'_>, fixed: [&Pubkey; 3], vault: &Pubkey,
    configuration: &SquadsMultisigConfiguration,
) -> SquadsExecutionResult<u16> {
    if account.key != &sysvar::instructions::ID || account.owner != &sysvar::ID || account.executable {
        return Err(SquadsExecutionError::InvalidInstructionsSysvar);
    }
    let bytes = account.try_borrow_data().map_err(|_| Piv1Error::AccountBorrowFailed)?;
    // The pinned checked current-index helper reads the final two bytes without
    // a length defense. Hold this immutable preflight borrow through its call.
    if bytes.len() < 4 { return Err(SquadsExecutionError::InvalidInstructionsSysvar); }
    let body_end = bytes.len() - 2;
    let mut table = Reader(&bytes[..body_end]);
    let instruction_count = usize::from(table.u16()?);
    let table_end = 2 + instruction_count * 2;
    if instruction_count == 0 || table_end > body_end { return Err(SquadsExecutionError::InvalidInstructionsSysvar); }
    let current = sysvar::instructions::load_current_index_checked(account).map_err(SquadsExecutionError::Runtime)?;
    if usize::from(current) >= instruction_count { return Err(SquadsExecutionError::InvalidInstructionsSysvar); }
    let mut start = 0;
    let mut end = body_end;
    let mut previous = None;
    for index in 0..instruction_count {
        let offset = usize::from(table.u16()?);
        if offset < table_end || offset >= body_end || previous.is_some_and(|old| offset <= old)
            || (index == 0 && offset != table_end)
        {
            return Err(SquadsExecutionError::InvalidInstructionsSysvar);
        }
        if index == usize::from(current) { start = offset; }
        if index == usize::from(current) + 1 { end = offset; }
        previous = Some(offset);
    }
    let mut reader = Reader(&bytes[start..end]);
    if usize::from(reader.u16()?) != 4 + message.len() { return Err(SquadsExecutionError::InvalidInvocation); }
    for (index, key) in fixed.iter().enumerate() {
        let flags = reader.byte()?;
        if flags & !3 != 0 || reader.key()? != **key
            || (index == 1 && flags & 2 == 0)
        {
            return Err(SquadsExecutionError::InvalidInvocation);
        }
    }
    // The fourth OUTER account is Squads' executor. It need not be named by the
    // approved inner action: any current Execute member may execute that action.
    let executor_flags = reader.byte()?;
    let executor = reader.key()?;
    let executor_index = configuration.member_keys().iter().position(|key| *key == executor)
        .ok_or(SquadsExecutionError::InvalidInvocation)?;
    if executor_flags & !3 != 0 || executor_flags & 1 == 0
        || configuration.member_permissions()[executor_index] & 4 == 0
    {
        return Err(SquadsExecutionError::InvalidInvocation);
    }
    for index in 0..message.len() {
        let flags = reader.byte()?;
        let key = reader.key()?;
        let requested = message.flags(index);
        if flags & !3 != 0 || key != message.key(index)? || (requested & 2 != 0 && flags & 2 == 0)
            || (requested & 1 != 0 && key != *vault && flags & 1 == 0)
        {
            return Err(SquadsExecutionError::InvalidInvocation);
        }
    }
    if reader.key()? != SQUADS_V4_PROGRAM_ID || usize::from(reader.u16()?) != 8
        || reader.array::<8>()? != VAULT_TRANSACTION_EXECUTE_DISCRIMINATOR || !reader.0.is_empty()
    {
        return Err(SquadsExecutionError::InvalidInvocation);
    }
    Ok(current)
}

struct Reader<'a>(&'a [u8]);
impl<'a> Reader<'a> {
    fn take(&mut self, count: usize) -> SquadsExecutionResult<&'a [u8]> {
        let head = self.0.get(..count).ok_or(SquadsExecutionError::InvalidTransaction)?;
        self.0 = self.0.get(count..).ok_or(SquadsExecutionError::InvalidTransaction)?;
        Ok(head)
    }
    fn array<const N: usize>(&mut self) -> SquadsExecutionResult<[u8; N]> {
        self.take(N)?.try_into().map_err(|_| SquadsExecutionError::InvalidTransaction)
    }
    fn byte(&mut self) -> SquadsExecutionResult<u8> { Ok(self.array::<1>()?[0]) }
    fn key(&mut self) -> SquadsExecutionResult<Pubkey> { Ok(Pubkey::new_from_array(self.array()?)) }
    fn u16(&mut self) -> SquadsExecutionResult<u16> { Ok(u16::from_le_bytes(self.array()?)) }
    fn u32(&mut self) -> SquadsExecutionResult<u32> { Ok(u32::from_le_bytes(self.array()?)) }
    fn u64(&mut self) -> SquadsExecutionResult<u64> { Ok(u64::from_le_bytes(self.array()?)) }
}
