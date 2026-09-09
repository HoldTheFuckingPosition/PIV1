mod support;

use anchor_lang::{
    prelude::{Clock, Pubkey, Rent, SolanaSysvar},
    solana_program::{system_program, sysvar},
    AnchorDeserialize, AnchorSerialize,
};
use piv1::{
    accounts::{CONFIG_DISCRIMINATOR, STAKE_PROGRAM_ID},
    constants::{GUARDIAN_COUNT, KIF_PERIOD_SECONDS},
    errors::Piv1Error,
    guardian_clock_accounts::*,
    kif_claim_accounts::GUARDIAN_REWARD_DISCRIMINATOR,
    state::*,
};
use support::kif_claim_custody::{
    envelope, key, BackingAccount, Fixture as ClaimFixture, PROGRAM,
    CONFIG as CLAIM_CONFIG, REWARD as CLAIM_REWARD,
};

const CONFIG: usize = 0;
const REGISTRY: usize = 1;
const REWARD_START: usize = 2;
const CLOCK: usize = 8;

/// Nine-account read-only fixture. All writes below construct/corrupt host bytes;
/// no initialization, rotation or heartbeat handler is implemented or exercised.
#[derive(Clone, Debug, PartialEq)]
struct Fixture {
    program_id: Pubkey,
    rent: Rent,
    accounts: [BackingAccount; 9],
}

impl Fixture {
    fn new(mask: u8) -> Self {
        // Reuse only a valid Config value and envelope constructor. The claim
        // fixture's custody, audit, permissions and actions are not snapshot inputs.
        let base = ClaimFixture::new(0);
        let mut config = base.config();
        let (registry_key, bump) = Pubkey::find_program_address(&[b"guardian-registry"], &PROGRAM);
        config.guardian_registry = registry_key;
        config.bumps.guardian_registry = bump;
        config.kif_anchor_timestamp = 0;
        let registry = GuardianRegistry::new(bump, config.guardian_registry_revision,
            core::array::from_fn(|i| key(91 + i as u8))).unwrap();
        let rent = Rent::default();
        let state = |key, data: Vec<u8>| BackingAccount {
            key, owner: PROGRAM, executable: false, signer: false, writable: false,
            lamports: rent.minimum_balance(data.len()), data,
        };
        let rewards: [BackingAccount; GUARDIAN_COUNT] = core::array::from_fn(|i| {
            let mut reward = GuardianReward::new(0, &registry, i as u8).unwrap();
            reward.last_active_period = if mask & (1 << i) != 0 { Some(40) } else { None };
            let (address, bump) = reward_address(&reward);
            reward.bump = bump;
            state(address, envelope(&reward, GUARDIAN_REWARD_DISCRIMINATOR, GuardianReward::SPACE))
        });
        let [r0, r1, r2, r3, r4, r5] = rewards;
        let mut fixture = Self { program_id: PROGRAM, rent: rent.clone(), accounts: [
            state(base.accounts[CLAIM_CONFIG].key, envelope(&config, CONFIG_DISCRIMINATOR, PivConfig::SPACE)),
            state(registry_key, envelope(&registry, GUARDIAN_REGISTRY_DISCRIMINATOR, GuardianRegistry::SPACE)),
            r0, r1, r2, r3, r4, r5,
            BackingAccount { key: sysvar::clock::ID, owner: sysvar::ID, executable: false,
                signer: false, writable: false, lamports: 0, data: vec![0; 40] },
        ] };
        fixture.set_clock(Clock { slot: 123_456, epoch_start_timestamp: 101_000_000,
            epoch: 41, leader_schedule_epoch: 42, unix_timestamp: 40 * KIF_PERIOD_SECONDS + 17 });
        fixture
    }

    fn config(&self) -> PivConfig {
        PivConfig::deserialize(&mut &self.accounts[CONFIG].data[8..]).unwrap()
    }
    fn registry(&self) -> GuardianRegistry {
        GuardianRegistry::deserialize(&mut &self.accounts[REGISTRY].data[8..]).unwrap()
    }
    fn reward(&self, index: usize) -> GuardianReward {
        GuardianReward::deserialize(&mut &self.accounts[REWARD_START + index].data[8..]).unwrap()
    }
    fn update_config(&mut self, edit: impl FnOnce(&mut PivConfig)) {
        let mut c = self.config(); edit(&mut c);
        self.accounts[CONFIG].data = envelope(&c, CONFIG_DISCRIMINATOR, PivConfig::SPACE);
    }
    fn update_registry(&mut self, edit: impl FnOnce(&mut GuardianRegistry)) {
        let mut r = self.registry(); edit(&mut r);
        self.accounts[REGISTRY].data = envelope(&r, GUARDIAN_REGISTRY_DISCRIMINATOR, GuardianRegistry::SPACE);
    }
    fn update_reward(&mut self, index: usize, edit: impl FnOnce(&mut GuardianReward)) {
        let mut r = self.reward(index); edit(&mut r);
        self.accounts[REWARD_START + index].data = envelope(&r, GUARDIAN_REWARD_DISCRIMINATOR, GuardianReward::SPACE);
    }
    fn rebind_reward(&mut self, index: usize, edit: impl FnOnce(&mut GuardianReward)) {
        let mut r = self.reward(index); edit(&mut r);
        let (address, bump) = reward_address(&r); r.bump = bump;
        self.accounts[REWARD_START + index].key = address;
        self.accounts[REWARD_START + index].data = envelope(&r, GUARDIAN_REWARD_DISCRIMINATOR, GuardianReward::SPACE);
    }
    fn set_guardian(&mut self, index: usize, guardian: Pubkey) {
        self.update_registry(|r| r.guardian_keys[index] = guardian);
        self.rebind_reward(index, |r| r.guardian = guardian);
    }
    fn set_revision(&mut self, revision: u64) {
        self.update_config(|c| c.guardian_registry_revision = revision);
        self.update_registry(|r| r.revision = revision);
        for i in 0..6 { self.rebind_reward(i, |r| r.registry_revision = revision); }
    }
    fn set_clock(&mut self, clock: Clock) {
        let mut account = self.accounts[CLOCK].info();
        clock.to_account_info(&mut account).unwrap();
    }
    fn set_time(&mut self, timestamp: i64) {
        self.set_clock(Clock { unix_timestamp: timestamp, ..Clock::default() });
    }
    fn with_infos<T>(&mut self, action: impl FnOnce(GuardianClockAccountInfos<'_, '_>) -> T) -> T {
        let [c, r, r0, r1, r2, r3, r4, r5, k] = &mut self.accounts;
        let c = c.info(); let r = r.info(); let r0 = r0.info(); let r1 = r1.info();
        let r2 = r2.info(); let r3 = r3.info(); let r4 = r4.info(); let r5 = r5.info(); let k = k.info();
        action(GuardianClockAccountInfos { config: &c, guardian_registry: &r,
            rewards: [&r0, &r1, &r2, &r3, &r4, &r5], clock: &k })
    }
    fn authenticate(&mut self) -> Result<AuthenticatedGuardianClockSnapshot, Piv1Error> {
        let program = self.program_id; let rent = self.rent.clone();
        self.with_infos(|infos| authenticate_guardian_clock_snapshot(&program, &rent, infos))
    }
    fn success(&mut self) -> AuthenticatedGuardianClockSnapshot {
        let before = self.clone(); let result = self.authenticate().unwrap();
        assert_eq!(*self, before, "success must preserve every byte, flag and lamport");
        result
    }
    fn reject(&mut self, error: Piv1Error) {
        let before = self.clone(); assert_eq!(self.authenticate(), Err(error));
        assert_eq!(*self, before, "rejection must preserve every byte, flag and lamport");
    }
}

fn reward_address(reward: &GuardianReward) -> (Pubkey, u8) {
    Pubkey::find_program_address(&[b"guardian-reward", reward.guardian.as_ref(),
        &reward.registry_revision.to_le_bytes(), &[reward.guardian_index]], &PROGRAM)
}

#[test]
fn all_64_current_activity_masks_and_counts_are_exact_and_read_only() {
    for mask in 0_u8..64 {
        let mut f = Fixture::new(mask);
        let snapshot = f.success();
        assert_eq!(snapshot.activity_bitmap(), mask);
        assert_eq!(u32::from(snapshot.active_count()), mask.count_ones());
        assert_eq!(snapshot.period(), KifPeriod { id: 40,
            start_timestamp: 40 * KIF_PERIOD_SECONDS, end_timestamp: 41 * KIF_PERIOD_SECONDS });
        assert_eq!(snapshot.config(), &f.config());
        assert_eq!(snapshot.registry(), &f.registry());
        for i in 0..6 { assert_eq!(snapshot.rewards()[i], f.reward(i)); }
        assert_eq!(snapshot.clock().slot, 123_456);
        assert_eq!(snapshot.clock().epoch, 41);
        assert_eq!(snapshot.clock().leader_schedule_epoch, 42);
        assert_eq!(snapshot.clock().epoch_start_timestamp, 101_000_000);
        assert_eq!(snapshot.clock().unix_timestamp, 40 * KIF_PERIOD_SECONDS + 17);
    }
}

#[test]
fn none_older_current_and_newer_activity_preserve_exact_query_equality() {
    let mut f = Fixture::new(0);
    let activity = [None, Some(39), Some(40), Some(41), Some(u64::MAX), Some(0)];
    for (i, value) in activity.into_iter().enumerate() { f.update_reward(i, |r| r.last_active_period = value); }
    let snapshot = f.success();
    assert_eq!(snapshot.activity_bitmap(), 4);
    assert_eq!(snapshot.active_count(), 1);
    f.set_time(41 * KIF_PERIOD_SECONDS);
    assert_eq!(f.success().activity_bitmap(), 8);
}

#[test]
fn half_open_edges_and_negative_anchors_come_only_from_clock_timestamp() {
    for anchor in [-5_184_001_i64, -1, 0, 1_700_000_000, i64::MIN] {
        for period in [0_i64, 1, 40] {
            let start = anchor + period * KIF_PERIOD_SECONDS;
            for timestamp in [start, start + KIF_PERIOD_SECONDS - 1] {
                let mut f = Fixture::new(0); f.update_config(|c| c.kif_anchor_timestamp = anchor);
                f.set_time(timestamp);
                assert_eq!(f.success().period(), KifPeriod { id: period as u64,
                    start_timestamp: start, end_timestamp: start + KIF_PERIOD_SECONDS });
            }
            let mut f = Fixture::new(0); f.update_config(|c| c.kif_anchor_timestamp = anchor);
            f.set_time(start + KIF_PERIOD_SECONDS);
            assert_eq!(f.success().period().id, period as u64 + 1);
        }
    }
}

#[test]
fn clock_timestamp_regression_and_checked_period_overflows_fail_read_only() {
    for (anchor, timestamp, error) in [
        (1, 0, Piv1Error::TimestampRegression),
        (-1, -2, Piv1Error::TimestampRegression),
        (i64::MIN, i64::MAX, Piv1Error::ArithmeticOverflow),
        (i64::MAX - KIF_PERIOD_SECONDS + 1, i64::MAX - KIF_PERIOD_SECONDS + 1,
         Piv1Error::ArithmeticOverflow),
        (0, i64::MAX, Piv1Error::ArithmeticOverflow),
    ] {
        let mut f = Fixture::new(0); f.update_config(|c| c.kif_anchor_timestamp = anchor);
        f.set_time(timestamp); f.reject(error);
    }
}

#[test]
fn clock_uses_official_decoding_with_exact_independent_five_field_byte_layout() {
    assert_eq!(CLOCK_ACCOUNT_SIZE, 40);
    assert_eq!(Clock::size_of(), 40);
    assert_eq!(sysvar::clock::ID.to_string(), "SysvarC1ock11111111111111111111111111111111");
    assert_eq!(sysvar::ID.to_string(), "Sysvar1111111111111111111111111111111111111");
    let clock = Clock { slot: u64::MAX, epoch_start_timestamp: i64::MIN,
        epoch: 73, leader_schedule_epoch: 0, unix_timestamp: -1 };
    let mut f = Fixture::new(0); f.update_config(|c| c.kif_anchor_timestamp = -2_592_001);
    f.set_clock(clock.clone());
    let mut expected = Vec::new();
    expected.extend(clock.slot.to_le_bytes()); expected.extend(clock.epoch_start_timestamp.to_le_bytes());
    expected.extend(clock.epoch.to_le_bytes()); expected.extend(clock.leader_schedule_epoch.to_le_bytes());
    expected.extend(clock.unix_timestamp.to_le_bytes());
    assert_eq!(f.accounts[CLOCK].data, expected);
    assert_eq!(f.success().clock(), &clock);
    // Every 40-byte integer field pattern decodes; no extra slot/epoch policy is invented.
    f.accounts[CLOCK].data = vec![0; 40];
    assert_eq!(f.success().clock(), &Clock::default());
}

#[test]
fn clock_wrong_id_owner_executable_and_short_or_long_data_are_rejected() {
    let mut f = Fixture::new(0); f.accounts[CLOCK].key = key(222);
    f.reject(Piv1Error::InvalidClockAccount);
    for owner in [system_program::ID, PROGRAM, sysvar::clock::ID, key(222)] {
        let mut f = Fixture::new(0); f.accounts[CLOCK].owner = owner;
        f.reject(Piv1Error::InvalidAccountOwner);
    }
    let mut f = Fixture::new(0); f.accounts[CLOCK].executable = true;
    f.reject(Piv1Error::ExecutableAccount);
    for len in [0, 8, 32, 39, 41, 48, 80] {
        let mut f = Fixture::new(0); f.accounts[CLOCK].data.resize(len, 0);
        f.reject(Piv1Error::InvalidAccountSize);
    }
}

#[test]
fn clock_needs_no_rent_or_lamport_borrow_but_data_conflicts_are_checked() {
    for lamports in [0, 1, u64::MAX] {
        let mut f = Fixture::new(63); f.accounts[CLOCK].lamports = lamports;
        assert_eq!(f.success().active_count(), 6);
    }
    let mut f = Fixture::new(21); let program = f.program_id; let rent = f.rent.clone();
    let before = f.clone();
    f.with_infos(|infos| {
        let _lamports = infos.clock.try_borrow_mut_lamports().unwrap();
        assert_eq!(authenticate_guardian_clock_snapshot(&program, &rent, infos).unwrap().activity_bitmap(), 21);
    });
    assert_eq!(f, before);
    f.with_infos(|infos| {
        let _data = infos.clock.try_borrow_mut_data().unwrap();
        assert_eq!(authenticate_guardian_clock_snapshot(&program, &rent, infos),
                   Err(Piv1Error::AccountBorrowFailed));
    });
    assert_eq!(f, before);
    f.with_infos(|infos| {
        let _shared_data = infos.clock.try_borrow_data().unwrap();
        assert_eq!(authenticate_guardian_clock_snapshot(&program, &rent, infos).unwrap().activity_bitmap(), 21);
    });
    assert_eq!(f, before);
}

#[test]
fn every_piv_state_account_requires_owner_nonexecutable_exact_size_and_discriminator() {
    for index in 0..8 {
        let mut f = Fixture::new(0); f.accounts[index].owner = key(222);
        f.reject(Piv1Error::InvalidAccountOwner);
        let mut f = Fixture::new(0); f.accounts[index].executable = true;
        f.reject(Piv1Error::ExecutableAccount);
        let mut f = Fixture::new(0); f.accounts[index].data[0] ^= 1;
        f.reject(Piv1Error::InvalidAccountDiscriminator);
        for longer in [false, true] {
            let mut f = Fixture::new(0);
            if longer { f.accounts[index].data.push(0); } else { f.accounts[index].data.pop(); }
            f.reject(Piv1Error::InvalidAccountSize);
        }
        let mut f = Fixture::new(0); f.accounts[index].data[8] = 0;
        f.reject(Piv1Error::InvalidVersion);
    }
}

#[test]
fn every_piv_state_account_requires_its_checked_rent_and_fallible_borrows() {
    for index in 0..8 {
        let mut f = Fixture::new(0); f.accounts[index].lamports -= 1;
        f.reject(Piv1Error::AccountRentDeficit);
        for data in [false, true] {
            let mut f = Fixture::new(0); let program = f.program_id; let rent = f.rent.clone();
            let before = f.clone();
            f.with_infos(|infos| {
                let account = [infos.config, infos.guardian_registry, infos.rewards[0], infos.rewards[1],
                    infos.rewards[2], infos.rewards[3], infos.rewards[4], infos.rewards[5]][index];
                let result = if data {
                    let _guard = account.try_borrow_mut_data().unwrap();
                    authenticate_guardian_clock_snapshot(&program, &rent, infos)
                } else {
                    let _guard = account.try_borrow_mut_lamports().unwrap();
                    authenticate_guardian_clock_snapshot(&program, &rent, infos)
                };
                assert_eq!(result, Err(Piv1Error::AccountBorrowFailed));
            });
            assert_eq!(f, before);
        }
    }
}

#[test]
fn exact_registry_and_reward_spaces_and_option_padding_are_unchanged() {
    assert_eq!(GUARDIAN_REGISTRY_SEED, b"guardian-registry");
    assert_eq!(GUARDIAN_REGISTRY_DISCRIMINATOR, [72, 14, 254, 2, 76, 233, 97, 92]);
    assert_eq!(GuardianRegistry::SPACE, 210); assert_eq!(GuardianReward::SPACE, 84);
    let f = Fixture::new(0);
    assert_eq!(8 + f.registry().try_to_vec().unwrap().len(), 210);
    for reward in 0..6 {
        for activity in [None, Some(40), Some(41)] {
            let mut f = Fixture::new(0); f.update_reward(reward, |r| r.last_active_period = activity);
            let used = 8 + f.reward(reward).try_to_vec().unwrap().len();
            assert_eq!(84 - used, if activity.is_none() { 8 } else { 0 });
            f.success();
            for padding in used..84 {
                let mut invalid = f.clone(); invalid.accounts[REWARD_START + reward].data[padding] = 1;
                invalid.reject(Piv1Error::InvalidAccountData);
            }
        }
        let mut f = Fixture::new(0); f.accounts[REWARD_START + reward].data[51] = 2;
        f.reject(Piv1Error::InvalidAccountData);
    }
    let mut f = Fixture::new(0); f.update_config(|c| {
        c.last_successful_preparation_at = None; c.last_valid_insufficient_attempt_at = None;
    });
    let used = 8 + f.config().try_to_vec().unwrap().len();
    for padding in used..PivConfig::SPACE {
        let mut invalid = f.clone(); invalid.accounts[CONFIG].data[padding] = 1;
        invalid.reject(Piv1Error::InvalidAccountData);
    }
}

#[test]
fn config_registry_and_reward_canonical_addresses_bumps_and_references_are_required() {
    for index in 0..8 {
        let mut f = Fixture::new(0); f.accounts[index].key = key(222);
        f.reject(Piv1Error::InvalidAccountPda);
    }
    let edits: &[fn(&mut PivConfig)] = &[
        |c| c.bumps.config ^= 1,
        |c| c.bumps.guardian_registry ^= 1,
        |c| c.guardian_registry = key(222),
    ];
    for edit in edits { let mut f = Fixture::new(0); f.update_config(edit); f.reject(Piv1Error::InvalidAccountPda); }
    let mut f = Fixture::new(0); f.update_registry(|r| r.bump ^= 1);
    f.reject(Piv1Error::InvalidAccountPda);
    for i in 0..6 {
        let mut f = Fixture::new(0); f.update_reward(i, |r| r.bump ^= 1);
        f.reject(Piv1Error::InvalidAccountPda);
    }
}

#[test]
fn current_registry_requires_exact_config_revision_and_six_distinct_nonzero_guardians() {
    for revision in [0, 98, 100, u64::MAX] {
        let mut f = Fixture::new(0); f.update_registry(|r| r.revision = revision);
        f.reject(Piv1Error::InvalidGuardianSet);
        let mut f = Fixture::new(0); f.update_config(|c| c.guardian_registry_revision = revision);
        f.reject(Piv1Error::InvalidGuardianSet);
    }
    for i in 0..6 {
        let mut f = Fixture::new(0); f.update_registry(|r| r.guardian_keys[i] = Pubkey::default());
        f.reject(Piv1Error::InvalidAddress);
        let mut f = Fixture::new(0);
        f.update_registry(|r| r.guardian_keys[i] = r.guardian_keys[(i + 1) % 6]);
        f.reject(Piv1Error::InvalidGuardianSet);
    }
    for revision in [0, 1, 99, u64::MAX] {
        let mut f = Fixture::new(63); f.set_revision(revision);
        assert_eq!(f.success().registry().revision, revision);
    }
}

#[test]
fn current_reward_tuple_binding_rejects_old_future_wrong_slot_and_wrong_guardian_records() {
    for i in 0..6 {
        let edits: &[fn(&mut GuardianReward)] = &[
            |r| r.registry_revision = 98,
            |r| r.registry_revision = 100,
            |r| r.guardian_index = (r.guardian_index + 1) % 6,
            |r| r.guardian = key(222),
        ];
        for edit in edits {
            // Correct standalone tuple PDA still cannot substitute for the current slot.
            let mut f = Fixture::new(0); f.rebind_reward(i, edit);
            f.reject(Piv1Error::InvalidGuardianSet);
            // Changing a tuple without updating its address fails the shared PDA check.
            let mut f = Fixture::new(0); f.update_reward(i, edit);
            f.reject(Piv1Error::InvalidAccountPda);
        }
        let mut f = Fixture::new(0); f.rebind_reward(i, |r| r.guardian_index = 6);
        f.reject(Piv1Error::InvalidGuardianSet);
    }
}

#[test]
fn reordered_duplicate_and_pairwise_aliased_inputs_cannot_change_activity_slot_identity() {
    for left in 0..9 {
        for right in left + 1..9 {
            let mut f = Fixture::new(0); f.accounts[left].key = f.accounts[right].key;
            f.reject(Piv1Error::AccountAlias);
        }
    }
    for i in 0..5 {
        let mut f = Fixture::new(1 << i);
        f.accounts.swap(REWARD_START + i, REWARD_START + i + 1);
        f.reject(Piv1Error::InvalidGuardianSet);
    }
}

#[test]
fn static_piv_role_aliases_are_rejected_without_fetching_unrelated_custody() {
    let base = Fixture::new(0); let c = base.config();
    for guardian in [c.piv_authority, c.active_distribution, c.principal_jito_vault,
                     c.pending_jito_vault, c.pending_sol_vault, c.principal_sol_queue,
                     c.operational_sol_vault, c.distribution_escrow, c.kif_sol_vault,
                     c.guardian_registry, base.accounts[CONFIG].key,
                     base.accounts[REWARD_START + 1].key] {
        let mut f = Fixture::new(0); f.set_guardian(0, guardian);
        f.reject(Piv1Error::AccountAlias);
    }
    let mut f = Fixture::new(0); let reward_key = f.accounts[REWARD_START].key;
    f.update_config(|c| c.principal_sol_queue = reward_key);
    f.reject(Piv1Error::AccountAlias);
    let edits: &[fn(&mut PivConfig, Pubkey)] = &[
        |c, k| c.htfp_recipient = k,
        |c, k| c.team_owner_recipient = k,
        |c, k| c.guardian_registry = k,
        |c, k| c.principal_sol_queue = k,
        |c, k| c.piv_authority = k,
    ];
    for edit in edits {
        let mut f = Fixture::new(0); let own_key = f.accounts[CONFIG].key;
        f.update_config(|c| edit(c, own_key)); f.reject(Piv1Error::AccountAlias);
    }
}

#[test]
fn external_beneficiary_overlap_and_off_curve_guardian_keys_do_not_require_wallet_accounts() {
    for team in [false, true] {
        let mut f = Fixture::new(1); let c = f.config();
        f.set_guardian(0, if team { c.team_owner_recipient } else { c.htfp_recipient });
        assert_eq!(f.success().activity_bitmap(), 1);
    }
    let (guardian, _) = Pubkey::find_program_address(&[b"host-guardian"], &key(222));
    assert!(!guardian.is_on_curve());
    let mut f = Fixture::new(1); f.set_guardian(0, guardian);
    assert_eq!(f.success().activity_bitmap(), 1);
}

#[test]
fn standalone_reward_and_config_identities_are_checked_without_new_cross_ledger_guards() {
    for i in 0..6 {
        let mut f = Fixture::new(0); f.update_reward(i, |r| r.claimable_lamports = 1);
        f.reject(Piv1Error::CumulativeReconciliationMismatch);
        let mut f = Fixture::new(0); f.update_reward(i, |r| r.cumulative_claimed = 1);
        f.reject(Piv1Error::CumulativeReconciliationMismatch);
    }
    let mut f = Fixture::new(0); f.update_config(|c| c.kif_claim_liability_lamports -= 1);
    f.reject(Piv1Error::CumulativeReconciliationMismatch);
    // Structural inspection is not a sum/bound proof over economic ledgers.
    let mut f = Fixture::new(1); f.update_reward(0, |r| {
        r.cumulative_earned = u64::MAX; r.claimable_lamports = u64::MAX;
    });
    assert_eq!(f.success().rewards()[0].claimable_lamports, u64::MAX);
}

#[test]
fn historical_unpaid_liability_coexists_with_zero_current_rewards_and_old_claims_remain_valid() {
    let mut f = Fixture::new(0); let snapshot = f.success();
    assert_eq!(snapshot.config().kif_claim_liability_lamports, 960);
    assert!(snapshot.rewards().iter().all(|r| r.claimable_lamports == 0));
    let mut old_claim = ClaimFixture::new(0);
    let current_config = snapshot.config().clone();
    old_claim.update_config(|c| *c = current_config);
    let old_before = old_claim.clone();
    f.accounts[REWARD_START] = old_claim.accounts[CLAIM_REWARD].clone();
    f.reject(Piv1Error::InvalidGuardianSet);
    assert_eq!(old_claim, old_before);
    // Same earned record still authenticates and claims through unchanged Task 2.7.
    old_claim.claim(300, 10).unwrap();
    old_claim.validate_audit().unwrap();
}

#[test]
fn flags_pause_and_unrelated_custody_values_do_not_gate_read_only_inspection() {
    for paused in [false, true] {
        for privileges in [false, true] {
            let mut f = Fixture::new(21); f.update_config(|c| c.paused = paused);
            for account in &mut f.accounts { account.signer = privileges; account.writable = privileges; }
            assert_eq!(f.success().activity_bitmap(), 21);
        }
    }
    // No round or economic-custody accounts exist in this fixture. These Config
    // values therefore do not assert matching physical custody or lifecycle proof.
    let mut f = Fixture::new(21); f.update_config(|c| {
        c.paused = true; c.next_distribution_sequence = u64::MAX;
        c.protected_principal_hwm_lamports = u64::MAX;
        c.accounted_historical_jitosol_units = u64::MAX;
        c.accounted_pending_sol_lamports = u64::MAX;
    });
    assert_eq!(f.success().activity_bitmap(), 21);
}

#[test]
fn accepted_activity_updates_and_new_clock_reads_do_not_mutate_an_earlier_owned_snapshot() {
    let mut f = Fixture::new(1); let old = f.success(); let old_copy = old.clone();
    let registry = f.registry();
    f.update_reward(0, |r| r.record_activity(&registry, 0, 41).unwrap());
    // A period-40 query after period-41 activity is valid and inactive.
    let refreshed = f.success();
    assert_eq!(refreshed.period().id, 40); assert_eq!(refreshed.activity_bitmap(), 0);
    assert_eq!(old, old_copy); assert_eq!(old.activity_bitmap(), 1);
    f.set_time(41 * KIF_PERIOD_SECONDS);
    assert_eq!(f.success().activity_bitmap(), 1);
    assert_eq!(old.period().id, 40);
    f.accounts[CLOCK].owner = key(222);
    f.reject(Piv1Error::InvalidAccountOwner);
    assert_eq!(old, old_copy);
}

#[test]
fn runtime_program_rent_and_initialized_config_constraints_are_preserved() {
    for program in [system_program::ID, spl_token::ID, STAKE_PROGRAM_ID] {
        let mut f = Fixture::new(0); f.program_id = program; f.reject(Piv1Error::InvalidProgramIdentity);
    }
    let mut f = Fixture::new(0); f.program_id = key(222); f.reject(Piv1Error::InvalidAccountOwner);
    for role in 0..3 {
        let mut f = Fixture::new(0); f.update_config(|c| match role {
            0 => c.system_program = key(222), 1 => c.token_program = key(222), _ => c.stake_program = key(222),
        }); f.reject(Piv1Error::InvalidProgramIdentity);
    }
    let mut f = Fixture::new(0); f.update_config(|c| c.is_initialized = false);
    f.reject(Piv1Error::InvalidInitialization);
    let mut f = Fixture::new(0); f.update_config(|c| c.configured_slippage_bps = 2);
    f.reject(Piv1Error::InvalidSlippage);
    let mut f = Fixture::new(0); f.update_config(|c| c.kif_period_seconds += 1);
    f.reject(Piv1Error::InvalidTimingConfiguration);
    let mut f = Fixture::new(0); f.update_config(|c| c.migration_reserve[0] = 1);
    f.reject(Piv1Error::InvalidInitialization);
    for rent in [Rent { exemption_threshold: f64::INFINITY, ..Rent::default() },
                 Rent { exemption_threshold: -1.0, ..Rent::default() },
                 Rent { burn_percent: 101, ..Rent::default() }] {
        let mut f = Fixture::new(0); f.rent = rent; f.reject(Piv1Error::InvalidRent);
    }
    let mut f = Fixture::new(0); f.rent.lamports_per_byte_year = u64::MAX;
    f.reject(Piv1Error::ArithmeticOverflow);
}

#[test]
fn valid_off_curve_alternate_bumps_cannot_replace_canonical_registry_or_reward_pdas() {
    let alternate = |seeds: &[&[u8]]| {
        let canonical = Pubkey::find_program_address(seeds, &PROGRAM);
        for bump in 0..=u8::MAX {
            let bump_seed = [bump];
            let mut with_bump = seeds.to_vec();
            with_bump.push(&bump_seed);
            if let Ok(address) = Pubkey::create_program_address(&with_bump, &PROGRAM) {
                if address != canonical.0 { return (address, bump); }
            }
        }
        panic!("fixture must contain a valid noncanonical bump");
    };
    let mut f = Fixture::new(0);
    let (address, bump) = alternate(&[b"guardian-registry"]);
    f.accounts[REGISTRY].key = address;
    f.update_registry(|r| r.bump = bump);
    f.update_config(|c| { c.guardian_registry = address; c.bumps.guardian_registry = bump; });
    f.reject(Piv1Error::InvalidAccountPda);
    for index in 0..6 {
        let mut f = Fixture::new(0); let r = f.reward(index);
        let (address, bump) = alternate(&[b"guardian-reward", r.guardian.as_ref(),
            &r.registry_revision.to_le_bytes(), &[r.guardian_index]]);
        f.accounts[REWARD_START + index].key = address;
        f.update_reward(index, |r| r.bump = bump);
        f.reject(Piv1Error::InvalidAccountPda);
    }
}

#[test]
fn later_reward_envelope_error_precedes_earlier_current_membership_error() {
    for later in 1..GUARDIAN_COUNT {
        let mut f = Fixture::new(0);
        // A valid historical tuple/PDA is structurally authenticated but does not
        // belong to this current registry. Envelope authentication must finish first.
        f.rebind_reward(0, |reward| reward.registry_revision += 1);
        f.clone().reject(Piv1Error::InvalidGuardianSet);
        f.accounts[REWARD_START + later].data[0] ^= 1;
        f.reject(Piv1Error::InvalidAccountDiscriminator);
    }
}

#[test]
fn multiple_reward_envelope_faults_propagate_in_slot_order() {
    for earlier in 0..GUARDIAN_COUNT - 1 {
        let mut f = Fixture::new(0);
        f.accounts[REWARD_START + earlier].data[8] = 2;
        f.accounts[REWARD_START + earlier + 1].data[0] ^= 1;
        f.reject(Piv1Error::InvalidVersion);
    }
}
