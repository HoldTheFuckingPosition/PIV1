// Host AccountInfo evidence only: no Squads execution, loader deployment, keys,
// signatures, or live identities are created by these synthetic fixtures.
#[path = "support/kif_claim_custody.rs"]
mod claim_support;

use anchor_lang::{
    prelude::{borsh, Clock, Pubkey, Rent, SolanaSysvar},
    solana_program::{bpf_loader_upgradeable, sysvar},
    AnchorSerialize,
};
use piv1::{
    accounts::{seeds, CONFIG_DISCRIMINATOR},
    constants::GUARDIAN_COUNT,
    errors::{Piv1Error, Piv1Result},
    guardian_clock_accounts::{
        authenticate_guardian_clock_snapshot, AuthenticatedGuardianClockSnapshot,
        GuardianClockAccountInfos, GUARDIAN_REGISTRY_DISCRIMINATOR, GUARDIAN_REGISTRY_SEED,
    },
    kif_claim_accounts::GUARDIAN_REWARD_DISCRIMINATOR,
    squads_accounts::*,
    state::{GuardianRegistry, GuardianReward, PivConfig},
};
use claim_support::{envelope, key, BackingAccount, Fixture as ClaimFixture, PROGRAM};

const PROGRAM_ACCOUNT: usize = 0;
const PROGRAM_DATA: usize = 1;
const MULTISIG: usize = 2;

// Independent test encoder uses Anchor/Borsh serialization, not the production
// byte reader. These fields describe the pinned external wire format only.
#[derive(Clone, AnchorSerialize)]
struct MemberWire { key: Pubkey, permissions: u8 }
#[derive(Clone, AnchorSerialize)]
struct MultisigWire {
    create_key: Pubkey,
    config_authority: Pubkey,
    threshold: u16,
    time_lock: u32,
    transaction_index: u64,
    stale_transaction_index: u64,
    rent_collector: Option<Pubkey>,
    bump: u8,
    members: Vec<MemberWire>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct Fixture {
    program_id: Pubkey,
    vault_index: u8,
    accounts: [BackingAccount; 3],
}

impl Fixture {
    fn new(collector: Option<Pubkey>) -> Self {
        let (multisig, bump) = Pubkey::find_program_address(
            &[b"multisig", b"multisig", key(80).as_ref()], &SQUADS_V4_PROGRAM_ID);
        let vault_index = 7;
        let (vault, _) = Pubkey::find_program_address(
            &[b"multisig", multisig.as_ref(), b"vault", &[vault_index]], &SQUADS_V4_PROGRAM_ID);
        let (program_data, _) = Pubkey::find_program_address(&[PROGRAM.as_ref()], &bpf_loader_upgradeable::ID);
        let mut program_bytes = 2_u32.to_le_bytes().to_vec();
        program_bytes.extend(program_data.to_bytes());
        let mut program_data_bytes = 3_u32.to_le_bytes().to_vec();
        program_data_bytes.extend(4242_u64.to_le_bytes());
        program_data_bytes.push(1);
        program_data_bytes.extend(vault.to_bytes());
        // Opaque arbitrary bytes are not evidence of a real executable artifact.
        program_data_bytes.extend([0xA5; 23]);
        let wire = MultisigWire {
            create_key: key(80), config_authority: Pubkey::default(), threshold: 4,
            time_lock: 0, transaction_index: 19, stale_transaction_index: 3,
            rent_collector: collector, bump,
            members: (91..97).map(|tag| MemberWire { key: key(tag), permissions: 7 }).collect(),
        };
        let account = |key, owner, executable, data| BackingAccount {
            key, owner, executable, signer: false, writable: false, lamports: 12345, data,
        };
        Self { program_id: PROGRAM, vault_index, accounts: [
            account(PROGRAM, bpf_loader_upgradeable::ID, true, program_bytes),
            account(program_data, bpf_loader_upgradeable::ID, false, program_data_bytes),
            account(multisig, SQUADS_V4_PROGRAM_ID, false,
                envelope(&wire, SQUADS_MULTISIG_DISCRIMINATOR, 330)),
        ] }
    }

    fn with_infos<T>(&mut self, action: impl FnOnce(SquadsAuthorityAccountInfos<'_, '_>) -> T) -> T {
        let [program, program_data, multisig] = &mut self.accounts;
        let program = program.info(); let program_data = program_data.info(); let multisig = multisig.info();
        action(SquadsAuthorityAccountInfos { program: &program, program_data: &program_data, multisig: &multisig })
    }
    fn authenticate(&mut self) -> Piv1Result<AuthenticatedSquadsAuthoritySnapshot> {
        let program_id = self.program_id; let index = self.vault_index;
        self.with_infos(|accounts| authenticate_squads_authority_snapshot(&program_id, index, accounts))
    }
    fn success(&mut self) -> AuthenticatedSquadsAuthoritySnapshot {
        let before = self.clone(); let snapshot = self.authenticate().unwrap();
        assert_eq!(*self, before, "success preserves every byte, key, owner, flag and lamport");
        snapshot
    }
    fn reject(&mut self, error: Piv1Error) {
        let before = self.clone(); assert_eq!(self.authenticate(), Err(error));
        assert_eq!(*self, before, "failure preserves every byte, key, owner, flag and lamport");
    }
    fn write(&mut self, account: usize, offset: usize, bytes: &[u8]) {
        self.accounts[account].data[offset..offset + bytes.len()].copy_from_slice(bytes);
    }
    fn member_offset(&self) -> usize { if self.accounts[MULTISIG].data[94] == 0 { 100 } else { 132 } }
}

#[test]
fn exact_loader_and_both_multisig_wire_profiles_are_owned_read_only_observations() {
    for collector in [None, Some(key(77)), Some(Pubkey::default())] {
        let mut f = Fixture::new(collector);
        let snapshot = f.success();
        assert_eq!(snapshot.trusted_runtime_program_id(), PROGRAM);
        assert_eq!(snapshot.program_data(), f.accounts[PROGRAM_DATA].key);
        assert_eq!(snapshot.program_modified_slot(), 4242);
        assert_eq!(snapshot.multisig(), f.accounts[MULTISIG].key);
        assert_eq!(snapshot.vault_index(), 7);
        assert_eq!(snapshot.vault().to_bytes().as_slice(), &f.accounts[PROGRAM_DATA].data[13..45]);
        let configuration = snapshot.configuration();
        assert_eq!(configuration.create_key(), key(80));
        assert_eq!(configuration.bump(), f.accounts[MULTISIG].data[f.member_offset() - 5]);
        assert_eq!(configuration.member_keys(), &core::array::from_fn(|i| key(91 + i as u8)));
        assert_eq!(configuration.member_permissions(), &[7; 6]);
        assert_eq!(configuration.rent_collector(), collector);
        assert_eq!(configuration.time_lock(), 0);
        assert_eq!(configuration.transaction_index(), 19);
        assert_eq!(configuration.stale_transaction_index(), 3);
        assert_eq!(f.member_offset() + 6 * 33, if collector.is_some() { 330 } else { 298 });
        // Owned evidence does not silently refresh after a later input change.
        f.write(MULTISIG, 72, &1_u16.to_le_bytes());
        assert_eq!(snapshot.configuration(), configuration);
        f.reject(Piv1Error::InvalidGuardianSet);
    }
}

#[test]
fn larger_allocations_nonzero_stale_tail_and_opaque_program_bytes_are_valid() {
    for collector in [None, Some(key(77))] {
        let mut f = Fixture::new(collector);
        let expected = f.success();
        let end = f.member_offset() + 6 * 33;
        f.accounts[MULTISIG].data[end..].fill(0xE3);
        f.accounts[MULTISIG].data.resize(100_000, 0xB7);
        f.accounts[PROGRAM_DATA].data[45..].fill(0xF1);
        f.accounts[PROGRAM_DATA].data.resize(120_000, 0xC8);
        assert_eq!(f.success(), expected);
        // The exact metadata floor is accepted without claiming executable validity.
        f.accounts[PROGRAM_DATA].data.truncate(45);
        assert_eq!(f.success(), expected);
    }
}

#[test]
fn signer_writable_unions_and_irrelevant_lamport_borrows_do_not_add_authority() {
    for flags in 0_u8..64 {
        let mut f = Fixture::new(None);
        let expected = f.success();
        for (index, account) in f.accounts.iter_mut().enumerate() {
            account.signer = flags & (1 << (2 * index)) != 0;
            account.writable = flags & (1 << (2 * index + 1)) != 0;
            account.lamports = if index == 0 { 0 } else { u64::MAX };
        }
        assert_eq!(f.success(), expected);
        let before = f.clone(); let program = f.program_id; let index = f.vault_index;
        f.with_infos(|accounts| {
            let _lamports = accounts.program_data.try_borrow_mut_lamports().unwrap();
            assert_eq!(authenticate_squads_authority_snapshot(&program, index, accounts).unwrap(), expected);
        });
        assert_eq!(f, before);
    }
}

#[test]
fn runtime_program_owner_executable_and_alias_substitutions_reject() {
    let mut f = Fixture::new(None); f.program_id = key(208);
    f.reject(Piv1Error::InvalidProgramIdentity);
    let mut f = Fixture::new(None); f.accounts[PROGRAM_ACCOUNT].key = key(208);
    f.reject(Piv1Error::InvalidProgramIdentity);
    let mut f = Fixture::new(None); f.accounts[PROGRAM_ACCOUNT].executable = false;
    f.reject(Piv1Error::InvalidProgramIdentity);
    for role in 0..3 {
        let mut f = Fixture::new(None); f.accounts[role].owner = key(208);
        f.reject(Piv1Error::InvalidAccountOwner);
    }
    for role in [PROGRAM_DATA, MULTISIG] {
        let mut f = Fixture::new(None); f.accounts[role].executable = true;
        f.reject(Piv1Error::ExecutableAccount);
    }
    for (left, right) in [(0, 1), (0, 2), (1, 2)] {
        let mut f = Fixture::new(None); f.accounts[right].key = f.accounts[left].key;
        f.reject(Piv1Error::AccountAlias);
    }
}

#[test]
fn loader_program_requires_exact_size_tag_embedded_and_canonical_programdata_address() {
    for length in 0..36 {
        let mut f = Fixture::new(None); f.accounts[PROGRAM_ACCOUNT].data.truncate(length);
        f.reject(Piv1Error::InvalidAccountSize);
    }
    let mut f = Fixture::new(None); f.accounts[PROGRAM_ACCOUNT].data.push(0);
    f.reject(Piv1Error::InvalidAccountSize);
    for tag in [0_u32, 1, 3, 4, u32::MAX] {
        let mut f = Fixture::new(None); f.write(PROGRAM_ACCOUNT, 0, &tag.to_le_bytes());
        f.reject(Piv1Error::InvalidAccountData);
    }
    let mut f = Fixture::new(None); f.write(PROGRAM_ACCOUNT, 4, key(208).as_ref());
    f.reject(Piv1Error::InvalidAccountPda);
    let mut f = Fixture::new(None); f.accounts[PROGRAM_DATA].key = key(208);
    f.write(PROGRAM_ACCOUNT, 4, key(208).as_ref());
    f.reject(Piv1Error::InvalidAccountPda);
}

#[test]
fn loader_programdata_requires_metadata_tag_some_and_exact_derived_authority() {
    for length in 0..45 {
        let mut f = Fixture::new(None); f.accounts[PROGRAM_DATA].data.truncate(length);
        f.reject(Piv1Error::InvalidAccountSize);
    }
    for tag in [0_u32, 1, 2, 4, u32::MAX] {
        let mut f = Fixture::new(None); f.write(PROGRAM_DATA, 0, &tag.to_le_bytes());
        f.reject(Piv1Error::InvalidAccountData);
    }
    for tag in [0_u8, 2, 255] {
        let mut f = Fixture::new(None); f.accounts[PROGRAM_DATA].data[12] = tag;
        f.reject(Piv1Error::InvalidAccountData);
    }
    for authority in [Pubkey::default(), key(91), Fixture::new(None).accounts[MULTISIG].key] {
        let mut f = Fixture::new(None); f.write(PROGRAM_DATA, 13, authority.as_ref());
        f.reject(Piv1Error::InvalidProgramIdentity);
    }
    let mut f = Fixture::new(None); f.vault_index = 8;
    f.reject(Piv1Error::InvalidProgramIdentity);
    for index in [0_u8, 255] {
        let mut f = Fixture::new(None); f.vault_index = index;
        let (vault, _) = Pubkey::find_program_address(
            &[b"multisig", f.accounts[MULTISIG].key.as_ref(), b"vault", &[index]], &SQUADS_V4_PROGRAM_ID);
        f.write(PROGRAM_DATA, 13, vault.as_ref());
        assert_eq!(f.success().vault_index(), index);
    }
}

#[test]
fn multisig_rejects_every_truncation_invalid_discriminator_option_and_hostile_count() {
    for collector in [None, Some(key(77))] {
        for length in 0..330 {
            let mut f = Fixture::new(collector); f.accounts[MULTISIG].data.truncate(length);
            f.reject(Piv1Error::InvalidAccountSize);
        }
        let mut f = Fixture::new(collector); f.accounts[MULTISIG].data[0] ^= 1;
        f.reject(Piv1Error::InvalidAccountDiscriminator);
        for tag in [2, 255] {
            let mut f = Fixture::new(collector); f.accounts[MULTISIG].data[94] = tag;
            f.reject(Piv1Error::InvalidAccountData);
        }
        for count in [0_u32, 1, 5, 7, 65535, u32::MAX] {
            let mut f = Fixture::new(collector); let offset = f.member_offset() - 4;
            f.write(MULTISIG, offset, &count.to_le_bytes());
            f.reject(Piv1Error::InvalidGuardianCount);
        }
    }
}

#[test]
fn multisig_address_create_key_and_noncanonical_bump_cannot_be_substituted() {
    let mut f = Fixture::new(None); f.accounts[MULTISIG].key = key(208);
    f.reject(Piv1Error::InvalidAccountPda);
    let mut f = Fixture::new(None); f.write(MULTISIG, 8, key(208).as_ref());
    f.reject(Piv1Error::InvalidAccountPda);
    for collector in [None, Some(key(77))] {
        let mut f = Fixture::new(collector); let offset = f.member_offset() - 5;
        f.accounts[MULTISIG].data[offset] = f.accounts[MULTISIG].data[offset].wrapping_sub(1);
        f.reject(Piv1Error::InvalidAccountPda);
    }
}

#[test]
fn exactly_six_sorted_nonzero_voters_and_four_threshold_with_no_admin_are_required() {
    for threshold in [0_u16, 1, 3, 5, 6, u16::MAX] {
        let mut f = Fixture::new(None); f.write(MULTISIG, 72, &threshold.to_le_bytes());
        f.reject(Piv1Error::InvalidGuardianSet);
    }
    for admin in [key(208), key(91), Fixture::new(None).accounts[MULTISIG].key] {
        let mut f = Fixture::new(None); f.write(MULTISIG, 40, admin.as_ref());
        f.reject(Piv1Error::InvalidGuardianSet);
    }
    for index in 0..6 {
        for replacement in [Pubkey::default(), key(91), key(90)] {
            if index == 0 && replacement == key(91) { continue; }
            if index == 0 && replacement == key(90) { continue; }
            let mut f = Fixture::new(None); let offset = f.member_offset() + index * 33;
            f.write(MULTISIG, offset, replacement.as_ref());
            f.reject(Piv1Error::InvalidGuardianSet);
        }
        for permissions in [0_u8, 1, 4, 5, 8, 10, 128, 255] {
            let mut f = Fixture::new(None); let offset = f.member_offset() + index * 33 + 32;
            f.accounts[MULTISIG].data[offset] = permissions;
            f.reject(Piv1Error::InvalidGuardianSet);
        }
    }
    let mut f = Fixture::new(None); let start = f.member_offset();
    let first = f.accounts[MULTISIG].data[start..start + 33].to_vec();
    let second = f.accounts[MULTISIG].data[start + 33..start + 66].to_vec();
    f.write(MULTISIG, start, &second); f.write(MULTISIG, start + 33, &first);
    f.reject(Piv1Error::InvalidGuardianSet);
}

#[test]
fn permission_combinations_require_some_initiator_and_executor_not_six_almighty_members() {
    for permissions in [[2_u8; 6], [3; 6], [6; 6]] {
        let mut f = Fixture::new(None); let start = f.member_offset();
        for (index, mask) in permissions.into_iter().enumerate() { f.accounts[MULTISIG].data[start + index * 33 + 32] = mask; }
        f.reject(Piv1Error::InvalidGuardianSet);
    }
    for permissions in [[3_u8, 6, 2, 2, 2, 2], [2, 2, 2, 2, 2, 7]] {
        let mut f = Fixture::new(None); let start = f.member_offset();
        for (index, mask) in permissions.into_iter().enumerate() { f.accounts[MULTISIG].data[start + index * 33 + 32] = mask; }
        assert_eq!(f.success().configuration().member_permissions(), &permissions);
    }
}

#[test]
fn transaction_indices_and_timelock_obey_current_upstream_invariants() {
    for (transaction, stale) in [(0_u64, 0_u64), (19, 19), (u64::MAX, u64::MAX)] {
        let mut f = Fixture::new(None); f.write(MULTISIG, 78, &transaction.to_le_bytes());
        f.write(MULTISIG, 86, &stale.to_le_bytes());
        assert_eq!(f.success().configuration().stale_transaction_index(), stale);
    }
    for stale in [20_u64, u64::MAX] {
        let mut f = Fixture::new(None); f.write(MULTISIG, 86, &stale.to_le_bytes());
        f.reject(Piv1Error::InvalidAccountData);
    }
    for time in [0_u32, 1, 7_776_000] {
        let mut f = Fixture::new(None); f.write(MULTISIG, 74, &time.to_le_bytes());
        assert_eq!(f.success().configuration().time_lock(), time);
    }
    for time in [7_776_001_u32, u32::MAX] {
        let mut f = Fixture::new(None); f.write(MULTISIG, 74, &time.to_le_bytes());
        f.reject(Piv1Error::InvalidTimingConfiguration);
    }
}

#[test]
fn every_inspected_data_borrow_conflict_rejects_without_panicking_or_effects() {
    for role in 0..3 {
        let mut f = Fixture::new(None); let before = f.clone();
        let program = f.program_id; let index = f.vault_index;
        f.with_infos(|accounts| {
            let all = [accounts.program, accounts.program_data, accounts.multisig];
            let _borrow = all[role].try_borrow_mut_data().unwrap();
            assert_eq!(authenticate_squads_authority_snapshot(&program, index, accounts), Err(Piv1Error::AccountBorrowFailed));
        });
        assert_eq!(f, before);
    }
}

#[test]
fn threshold_one_approved_history_is_not_disproved_by_a_current_four_of_six_snapshot() {
    let mut f = Fixture::new(None);
    f.write(MULTISIG, 72, &1_u16.to_le_bytes());
    f.reject(Piv1Error::InvalidGuardianSet);
    // Synthetic current metadata after threshold restoration. No proposal votes
    // are supplied or executed. Upstream permits approved stale vault proposals;
    // a genuine future authorization gate must inspect and bind their approvals.
    f.write(MULTISIG, 72, &4_u16.to_le_bytes());
    f.write(MULTISIG, 86, &19_u64.to_le_bytes());
    let snapshot = f.success();
    assert_eq!(snapshot.configuration().transaction_index(), 19);
    assert_eq!(snapshot.configuration().stale_transaction_index(), 19);
}

// Construct full authenticated PIV1 registry/activity evidence to test same-ID
// provenance. The starting Config comes from unchanged host support only.
fn guardian_snapshot(program: Pubkey, keys: [Pubkey; GUARDIAN_COUNT]) -> AuthenticatedGuardianClockSnapshot {
    let mut config = ClaimFixture::new(0).config();
    let (config_key, bump) = Pubkey::find_program_address(&[seeds::CONFIG], &program);
    config.bumps.config = bump;
    macro_rules! bind {
        ($field:ident, $seed:expr) => {{
            let (address, bump) = Pubkey::find_program_address(&[$seed], &program);
            config.$field = address; config.bumps.$field = bump;
        }};
    }
    bind!(piv_authority, seeds::AUTHORITY);
    bind!(active_distribution, seeds::DISTRIBUTION);
    bind!(principal_jito_vault, seeds::PRINCIPAL_JITO);
    bind!(pending_jito_vault, seeds::PENDING_JITO);
    bind!(pending_sol_vault, seeds::PENDING_SOL);
    bind!(principal_sol_queue, seeds::PRINCIPAL_SOL);
    bind!(operational_sol_vault, seeds::OPERATIONAL_SOL);
    bind!(distribution_escrow, seeds::DISTRIBUTION_ESCROW);
    bind!(kif_sol_vault, seeds::KIF_SOL);
    bind!(guardian_registry, GUARDIAN_REGISTRY_SEED);
    let registry = GuardianRegistry::new(config.bumps.guardian_registry, config.guardian_registry_revision, keys).unwrap();
    let rent = Rent::default();
    let state = |key, data: Vec<u8>| BackingAccount {
        key, owner: program, executable: false, signer: false, writable: false,
        lamports: rent.minimum_balance(data.len()), data,
    };
    let rewards: [BackingAccount; 6] = core::array::from_fn(|index| {
        let mut reward = GuardianReward::new(0, &registry, index as u8).unwrap();
        let (address, bump) = Pubkey::find_program_address(&[
            b"guardian-reward", reward.guardian.as_ref(), &reward.registry_revision.to_le_bytes(), &[reward.guardian_index],
        ], &program);
        reward.bump = bump;
        state(address, envelope(&reward, GUARDIAN_REWARD_DISCRIMINATOR, GuardianReward::SPACE))
    });
    let [r0, r1, r2, r3, r4, r5] = rewards;
    let mut accounts = [
        state(config_key, envelope(&config, CONFIG_DISCRIMINATOR, PivConfig::SPACE)),
        state(config.guardian_registry, envelope(&registry, GUARDIAN_REGISTRY_DISCRIMINATOR, GuardianRegistry::SPACE)),
        r0, r1, r2, r3, r4, r5,
        BackingAccount { key: sysvar::clock::ID, owner: sysvar::ID, executable: false,
            signer: false, writable: false, lamports: 0, data: vec![0; 40] },
    ];
    Clock::default().to_account_info(&mut accounts[8].info()).unwrap();
    let before = accounts.clone();
    let snapshot = {
        let [c, r, r0, r1, r2, r3, r4, r5, clock] = &mut accounts;
        let c = c.info(); let r = r.info(); let r0 = r0.info(); let r1 = r1.info();
        let r2 = r2.info(); let r3 = r3.info(); let r4 = r4.info(); let r5 = r5.info(); let clock = clock.info();
        authenticate_guardian_clock_snapshot(&program, &rent, GuardianClockAccountInfos {
            config: &c, guardian_registry: &r, rewards: [&r0, &r1, &r2, &r3, &r4, &r5], clock: &clock,
        }).unwrap()
    };
    assert_eq!(accounts, before, "guardian authentication preserves all supplied accounts");
    assert_eq!(snapshot.trusted_runtime_program_id(), program);
    snapshot
}

#[test]
fn guardian_correspondence_preserves_slots_and_requires_the_same_authenticated_runtime_id() {
    let snapshot = Fixture::new(None).success();
    let mut keys = *snapshot.configuration().member_keys();
    for _ in 0..6 {
        keys.rotate_left(1);
        let guardians = guardian_snapshot(PROGRAM, keys);
        let before = guardians.clone();
        assert_eq!(snapshot.validate_guardian_correspondence(&guardians), Ok(()));
        assert_eq!(guardians.registry().guardian_keys, keys);
        assert_eq!(guardians, before);
    }
    let guardians = guardian_snapshot(key(218), keys);
    let before = guardians.clone();
    assert_eq!(snapshot.validate_guardian_correspondence(&guardians), Err(Piv1Error::InvalidProgramIdentity));
    assert_eq!(guardians, before);
    keys[0] = key(108);
    let guardians = guardian_snapshot(PROGRAM, keys);
    let before = guardians.clone();
    assert_eq!(snapshot.validate_guardian_correspondence(&guardians), Err(Piv1Error::InvalidGuardianSet));
    assert_eq!(guardians, before);
}
