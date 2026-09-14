//! Read-only PIV1 loader-v3 authority and Squads v4 configuration observations.
//!
//! The executing PIV1 program ID is a trusted runtime input. Squads identity is
//! fixed to the official source ID at revision 64af7330413d5c85cbbccfd8c27a05d45b6e666f.
//! Neither that ID nor an owner/discriminator proves a deployed artifact matches
//! that revision. Public-Testnet availability and executable verification remain
//! unestablished. No account is initialized or changed, and no signer is required.
//!
//! These owned snapshots are point-in-time evidence, never instruction approval
//! or authorization. Refresh both observations after any mutation or CPI. Squads
//! allows previously approved stale vault transactions to execute; a current
//! threshold of four cannot prove that an action received four guardian votes.

use anchor_lang::{
    prelude::{AccountInfo, Pubkey},
    solana_program::bpf_loader_upgradeable::{self, UpgradeableLoaderState},
};
use crate::{
    constants::GUARDIAN_COUNT,
    errors::{Piv1Error, Piv1Result},
    guardian_clock_accounts::AuthenticatedGuardianClockSnapshot,
};

/// Official non-testing identity in the pinned Squads v4 source; not a cluster claim.
pub const SQUADS_V4_PROGRAM_ID: Pubkey =
    Pubkey::from_str_const("SQDS4ep65T869zMMBKyuUq6aD6EgTu8psMjkvj52pCf");
/// Pinned Anchor discriminator, corroborated by the upstream generated account.
pub const SQUADS_MULTISIG_DISCRIMINATOR: [u8; 8] = [224, 116, 121, 186, 68, 161, 79, 236];
const MULTISIG_MINIMUM_SIZE: usize = 330;
const MAX_TIME_LOCK: u32 = 3 * 30 * 24 * 60 * 60;
const INITIATE: u8 = 1;
const VOTE: u8 = 2;
const EXECUTE: u8 = 4;

/// Three inspected accounts. The vault authority is derived as an address only;
/// it is not the Squads-owned multisig account and need not be supplied or sign.
#[derive(Clone, Copy)]
pub struct SquadsAuthorityAccountInfos<'a, 'info> {
    pub program: &'a AccountInfo<'info>,
    pub program_data: &'a AccountInfo<'info>,
    pub multisig: &'a AccountInfo<'info>,
}

/// Current six-voter configuration with no unilateral config authority.
/// Fields are observations, not proof of votes, history, or wallet key control.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SquadsMultisigConfiguration {
    create_key: Pubkey,
    bump: u8,
    time_lock: u32,
    transaction_index: u64,
    stale_transaction_index: u64,
    rent_collector: Option<Pubkey>,
    member_keys: [Pubkey; GUARDIAN_COUNT],
    member_permissions: [u8; GUARDIAN_COUNT],
}

impl SquadsMultisigConfiguration {
    pub fn create_key(&self) -> Pubkey { self.create_key }
    pub fn bump(&self) -> u8 { self.bump }
    pub fn time_lock(&self) -> u32 { self.time_lock }
    pub fn transaction_index(&self) -> u64 { self.transaction_index }
    pub fn stale_transaction_index(&self) -> u64 { self.stale_transaction_index }
    pub fn rent_collector(&self) -> Option<Pubkey> { self.rent_collector }
    pub fn member_keys(&self) -> &[Pubkey; GUARDIAN_COUNT] { &self.member_keys }
    pub fn member_permissions(&self) -> &[u8; GUARDIAN_COUNT] { &self.member_permissions }
}

/// Current ProgramData authority equals this multisig's derived vault address.
/// Threshold four is authenticated, but neither proposal approvals nor absence
/// of separate spending-limit accounts is established by this observation.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AuthenticatedSquadsAuthoritySnapshot {
    trusted_runtime_program_id: Pubkey,
    program_data: Pubkey,
    program_modified_slot: u64,
    multisig: Pubkey,
    vault: Pubkey,
    vault_index: u8,
    configuration: SquadsMultisigConfiguration,
}

impl AuthenticatedSquadsAuthoritySnapshot {
    pub fn trusted_runtime_program_id(&self) -> Pubkey { self.trusted_runtime_program_id }
    pub fn program_data(&self) -> Pubkey { self.program_data }
    pub fn program_modified_slot(&self) -> u64 { self.program_modified_slot }
    pub fn multisig(&self) -> Pubkey { self.multisig }
    pub fn vault(&self) -> Pubkey { self.vault }
    pub fn vault_index(&self) -> u8 { self.vault_index }
    pub fn configuration(&self) -> &SquadsMultisigConfiguration { &self.configuration }

    /// Compare already-authenticated observations from the SAME runtime PIV1 ID.
    /// Squads sorts members, whereas PIV1 slots bind reward records. Set equality
    /// preserves PIV1's slot order. This neither rotates nor synchronizes state;
    /// the caller must refresh both snapshots after mutations or CPI.
    pub fn validate_guardian_correspondence(
        &self,
        guardians: &AuthenticatedGuardianClockSnapshot,
    ) -> Piv1Result<()> {
        if self.trusted_runtime_program_id != guardians.trusted_runtime_program_id() {
            return Err(Piv1Error::InvalidProgramIdentity);
        }
        if !guardians.registry().guardian_keys.iter()
            .all(|key| self.configuration.member_keys.contains(key))
        {
            return Err(Piv1Error::InvalidGuardianSet);
        }
        Ok(())
    }
}

/// Authenticate metadata and present configuration without executing governance.
/// The explicit vault index is bound to the actual ProgramData authority; it
/// cannot select an unrelated authority. Signer/writable unions and lamports do
/// not affect these read-only checks. ProgramData's executable bytes are opaque.
pub fn authenticate_squads_authority_snapshot(
    trusted_runtime_program_id: &Pubkey,
    vault_index: u8,
    accounts: SquadsAuthorityAccountInfos<'_, '_>,
) -> Piv1Result<AuthenticatedSquadsAuthoritySnapshot> {
    let all = [accounts.program, accounts.program_data, accounts.multisig];
    for (index, account) in all.iter().enumerate() {
        if all[index + 1..].iter().any(|other| account.key == other.key) {
            return Err(Piv1Error::AccountAlias);
        }
    }
    if accounts.program.key != trusted_runtime_program_id || !accounts.program.executable {
        return Err(Piv1Error::InvalidProgramIdentity);
    }
    if accounts.program.owner != &bpf_loader_upgradeable::ID
        || accounts.program_data.owner != &bpf_loader_upgradeable::ID
        || accounts.multisig.owner != &SQUADS_V4_PROGRAM_ID
    {
        return Err(Piv1Error::InvalidAccountOwner);
    }
    if accounts.program_data.executable || accounts.multisig.executable {
        return Err(Piv1Error::ExecutableAccount);
    }
    let (program_data, _) = Pubkey::find_program_address(
        &[trusted_runtime_program_id.as_ref()], &bpf_loader_upgradeable::ID);
    if *accounts.program_data.key != program_data { return Err(Piv1Error::InvalidAccountPda); }
    {
        let bytes = accounts.program.try_borrow_data().map_err(|_| Piv1Error::AccountBorrowFailed)?;
        if bytes.len() != UpgradeableLoaderState::size_of_program() {
            return Err(Piv1Error::InvalidAccountSize);
        }
        let mut reader = Reader(&bytes);
        if u32::from_le_bytes(reader.read()?) != 2 { return Err(Piv1Error::InvalidAccountData); }
        if Pubkey::new_from_array(reader.read()?) != program_data {
            return Err(Piv1Error::InvalidAccountPda);
        }
    }
    let (program_modified_slot, authority) = {
        let bytes = accounts.program_data.try_borrow_data()
            .map_err(|_| Piv1Error::AccountBorrowFailed)?;
        if bytes.len() < UpgradeableLoaderState::size_of_programdata_metadata() {
            return Err(Piv1Error::InvalidAccountSize);
        }
        let mut reader = Reader(&bytes);
        if u32::from_le_bytes(reader.read()?) != 3 { return Err(Piv1Error::InvalidAccountData); }
        let slot = u64::from_le_bytes(reader.read()?);
        // A valid None authority is immutable, outside this upgradeable profile.
        if reader.read::<1>()? != [1] { return Err(Piv1Error::InvalidAccountData); }
        (slot, Pubkey::new_from_array(reader.read()?))
    };
    let configuration = {
        let bytes = accounts.multisig.try_borrow_data()
            .map_err(|_| Piv1Error::AccountBorrowFailed)?;
        decode_multisig(&bytes)?
    };
    let (multisig, bump) = Pubkey::find_program_address(
        &[b"multisig", b"multisig", configuration.create_key.as_ref()], &SQUADS_V4_PROGRAM_ID);
    if *accounts.multisig.key != multisig || configuration.bump != bump {
        return Err(Piv1Error::InvalidAccountPda);
    }
    let (vault, _) = Pubkey::find_program_address(
        &[b"multisig", multisig.as_ref(), b"vault", &[vault_index]], &SQUADS_V4_PROGRAM_ID);
    if authority != vault { return Err(Piv1Error::InvalidProgramIdentity); }
    Ok(AuthenticatedSquadsAuthoritySnapshot {
        trusted_runtime_program_id: *trusted_runtime_program_id,
        program_data, program_modified_slot, multisig, vault, vault_index, configuration,
    })
}

/// Exactly six members are decoded without Vec or input-sized allocation. The
/// upstream Borsh Option is variable-length, while its allocation reserves Some.
/// Larger allocations and nonzero stale tails are valid and are not copied.
fn decode_multisig(bytes: &[u8]) -> Piv1Result<SquadsMultisigConfiguration> {
    if bytes.len() < MULTISIG_MINIMUM_SIZE { return Err(Piv1Error::InvalidAccountSize); }
    let mut reader = Reader(bytes);
    if reader.read::<8>()? != SQUADS_MULTISIG_DISCRIMINATOR {
        return Err(Piv1Error::InvalidAccountDiscriminator);
    }
    let create_key = Pubkey::new_from_array(reader.read()?);
    let config_authority = Pubkey::new_from_array(reader.read()?);
    let threshold = u16::from_le_bytes(reader.read()?);
    let time_lock = u32::from_le_bytes(reader.read()?);
    let transaction_index = u64::from_le_bytes(reader.read()?);
    let stale_transaction_index = u64::from_le_bytes(reader.read()?);
    let rent_collector = match reader.read::<1>()? {
        [0] => None,
        [1] => Some(Pubkey::new_from_array(reader.read()?)),
        _ => return Err(Piv1Error::InvalidAccountData),
    };
    let [bump] = reader.read()?;
    if u32::from_le_bytes(reader.read()?) != GUARDIAN_COUNT as u32 {
        return Err(Piv1Error::InvalidGuardianCount);
    }
    let mut member_keys = [Pubkey::default(); GUARDIAN_COUNT];
    let mut member_permissions = [0_u8; GUARDIAN_COUNT];
    for index in 0..GUARDIAN_COUNT {
        let key = Pubkey::new_from_array(reader.read()?);
        let [permissions] = reader.read()?;
        if key == Pubkey::default() || (index > 0 && key <= member_keys[index - 1])
            || permissions & !7 != 0 || permissions & VOTE == 0
        {
            return Err(Piv1Error::InvalidGuardianSet);
        }
        member_keys[index] = key;
        member_permissions[index] = permissions;
    }
    if config_authority != Pubkey::default() || threshold != 4
        || !member_permissions.iter().any(|mask| mask & INITIATE != 0)
        || !member_permissions.iter().any(|mask| mask & EXECUTE != 0)
    {
        return Err(Piv1Error::InvalidGuardianSet);
    }
    if stale_transaction_index > transaction_index { return Err(Piv1Error::InvalidAccountData); }
    if time_lock > MAX_TIME_LOCK { return Err(Piv1Error::InvalidTimingConfiguration); }
    Ok(SquadsMultisigConfiguration {
        create_key, bump, time_lock, transaction_index, stale_transaction_index,
        rent_collector, member_keys, member_permissions,
    })
}

struct Reader<'a>(&'a [u8]);
impl Reader<'_> {
    fn read<const N: usize>(&mut self) -> Piv1Result<[u8; N]> {
        let bytes = self.0.get(..N).ok_or(Piv1Error::InvalidAccountSize)?;
        let value = bytes.try_into().map_err(|_| Piv1Error::InvalidAccountSize)?;
        self.0 = self.0.get(N..).ok_or(Piv1Error::InvalidAccountSize)?;
        Ok(value)
    }
}
