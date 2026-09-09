mod support;

use anchor_lang::{
    prelude::{AccountInfo, Clock, Pubkey, Rent, SolanaSysvar},
    solana_program::{program_option::COption, program_pack::Pack, system_program, sysvar},
    AnchorDeserialize, AnchorSerialize,
};
use piv1::{
    accounts::{authenticate_fixed_accounts, FixedAccountInfos, STAKE_PROGRAM_ID},
    errors::{Piv1Error, Piv1Result},
    guardian_clock_accounts::{authenticate_guardian_clock_snapshot, GuardianClockAccountInfos},
    integrations::FeeFraction,
    state::*,
    state_persistence::{commit_state_writes, PreparedStateWrite, StateEnvelope},
};
use spl_token::state::{Account as TokenAccount, AccountState};
use support::{
    kif_claim_custody::{key, BackingAccount, Fixture as ClaimFixture, PROGRAM},
    vault_custody_model::World,
};

const CONFIG: usize = 0;
const ROUND: usize = 1;
const REGISTRY: usize = 2;
const REWARD: usize = 3;
const SIZES: [usize; 4] = [1014, 891, 210, 84];
// Independently checked SHA-256("account:<type>") prefixes.
const DISCRIMINATORS: [[u8; 8]; 4] = [
    [98, 115, 11, 164, 170, 207, 163, 20],
    [104, 51, 125, 187, 226, 55, 209, 99],
    [72, 14, 254, 2, 76, 233, 97, 92],
    [169, 109, 89, 17, 75, 171, 105, 39],
];

fn raw<T: AnchorSerialize>(value: &T, index: usize) -> Vec<u8> {
    let mut out = DISCRIMINATORS[index].to_vec();
    out.extend(value.try_to_vec().unwrap());
    assert!(out.len() <= SIZES[index]);
    out.resize(SIZES[index], 0);
    out
}
fn decode<T: AnchorDeserialize>(bytes: &[u8]) -> T {
    let mut remaining = &bytes[8..];
    let state = T::deserialize(&mut remaining).unwrap();
    assert!(remaining.iter().all(|b| *b == 0));
    state
}
fn reward_address(r: &GuardianReward) -> (Pubkey, u8) {
    Pubkey::find_program_address(&[b"guardian-reward", r.guardian.as_ref(),
        &r.registry_revision.to_le_bytes(), &[r.guardian_index]], &PROGRAM)
}
fn state_account(key: Pubkey, data: Vec<u8>) -> BackingAccount {
    BackingAccount { key, owner: PROGRAM, executable: false, signer: false, writable: true,
        lamports: Rent::default().minimum_balance(data.len()), data }
}

/// Newly funded AccountInfo envelope fixture; importing a valid World payload
/// below is codec compatibility evidence, not a continuous custody audit.
#[derive(Clone, Debug, Eq, PartialEq)]
struct Fixture { accounts: [BackingAccount; 4] }
impl Fixture {
    fn new(active: bool) -> Self {
        let mut world = World::new(10_000, 50, 123, 9, 3, FeeFraction::ZERO, 100_000);
        if active { world.open(1_000_000).unwrap(); }
        let mut c = world.config;
        let mut d = world.round;
        let (config_key, bump) = Pubkey::find_program_address(&[b"config"], &PROGRAM);
        c.bumps.config = bump;
        macro_rules! bind {
            ($field:ident, $seed:literal) => {{
                let (address, bump) = Pubkey::find_program_address(&[$seed], &PROGRAM);
                c.$field = address; c.bumps.$field = bump;
            }};
        }
        bind!(piv_authority, b"authority");
        bind!(active_distribution, b"distribution");
        bind!(pending_sol_vault, b"pending-sol");
        bind!(principal_sol_queue, b"principal-sol");
        bind!(operational_sol_vault, b"operational-sol");
        bind!(distribution_escrow, b"distribution-escrow");
        bind!(kif_sol_vault, b"kif-sol");
        bind!(principal_jito_vault, b"principal-jito-vault");
        bind!(pending_jito_vault, b"pending-jito-vault");
        bind!(guardian_registry, b"guardian-registry");
        c.system_program = system_program::ID;
        c.token_program = spl_token::ID;
        c.stake_program = STAKE_PROGRAM_ID;
        d.bump = c.bumps.active_distribution;
        // Round immutable snapshots intentionally retain the imported fixture
        // identities; per-account decoding does not authorize changing them.
        let registry = GuardianRegistry::new(c.bumps.guardian_registry,
            c.guardian_registry_revision, world.registry.guardian_keys).unwrap();
        let mut reward = world.rewards[0];
        let (reward_key, bump) = reward_address(&reward); reward.bump = bump;
        Self { accounts: [
            state_account(config_key, raw(&c, CONFIG)),
            state_account(c.active_distribution, raw(&d, ROUND)),
            state_account(c.guardian_registry, raw(&registry, REGISTRY)),
            state_account(reward_key, raw(&reward, REWARD)),
        ] }
    }
    fn config(&self) -> PivConfig { decode(&self.accounts[CONFIG].data) }
    fn round(&self) -> ActiveDistribution { decode(&self.accounts[ROUND].data) }
    fn registry(&self) -> GuardianRegistry { decode(&self.accounts[REGISTRY].data) }
    fn reward(&self) -> GuardianReward { decode(&self.accounts[REWARD].data) }
    fn envelopes(&self) -> [StateEnvelope; 4] {
        [StateEnvelope::config(&self.config()).unwrap(),
         StateEnvelope::distribution(&self.round()).unwrap(),
         StateEnvelope::registry(&self.registry()).unwrap(),
         StateEnvelope::reward(&self.reward()).unwrap()]
    }
    fn plans(&self, after: [StateEnvelope; 4]) -> [PreparedStateWrite; 4] {
        let mut before = self.envelopes().into_iter();
        let mut after = after.into_iter();
        core::array::from_fn(|i| PreparedStateWrite::new(&PROGRAM, self.accounts[i].key,
            before.next().unwrap(), after.next().unwrap()).unwrap())
    }
    fn changed(&self) -> [StateEnvelope; 4] {
        let mut c = self.config(); c.paused = !c.paused;
        let mut d = self.round(); d.last_completed = Some(summary());
        let mut r = self.registry(); r.revision += 1;
        let mut g = self.reward(); g.last_active_period = Some(99);
        // These structurally valid changes exercise four different envelopes;
        // they are not authorized governance, history or activity transitions.
        [StateEnvelope::config(&c).unwrap(), StateEnvelope::distribution(&d).unwrap(),
         StateEnvelope::registry(&r).unwrap(), StateEnvelope::reward(&g).unwrap()]
    }
    fn with_infos<T>(&mut self, f: impl FnOnce([&AccountInfo<'_>; 4]) -> T) -> T {
        let [c, d, r, g] = &mut self.accounts;
        let c = c.info(); let d = d.info(); let r = r.info(); let g = g.info();
        f([&c, &d, &r, &g])
    }
    fn commit(&mut self, plans: &[PreparedStateWrite; 4]) -> Piv1Result<()> {
        self.with_infos(|accounts| commit_state_writes(&PROGRAM, &Rent::default(),
            core::array::from_fn::<_, 4, _>(|i| (&plans[i], accounts[i]))))
    }
    fn reject(&mut self, plans: &[PreparedStateWrite; 4], error: Piv1Error) {
        let before = self.clone();
        assert_eq!(self.commit(plans), Err(error));
        assert_eq!(*self, before, "all bytes, lamports and metadata must remain unchanged");
    }

    /// Exercise all four envelopes through existing production account readers.
    /// Auxiliary custody is synthetic and not asserted as economically balanced.
    fn authenticate_all(&mut self) {
        let prior = self.clone();
        let c = self.config(); let registry = self.registry();
        let native = |key| BackingAccount { key, owner: system_program::ID, executable: false,
            signer: false, writable: false, lamports: Rent::default().minimum_balance(0), data: vec![] };
        let mut natives = [native(c.pending_sol_vault), native(c.principal_sol_queue),
            native(c.operational_sol_vault), native(c.distribution_escrow), native(c.kif_sol_vault)];
        let token = |key| {
            let mut data = vec![0; TokenAccount::LEN];
            TokenAccount::pack(TokenAccount { mint: c.jitosol_mint, owner: c.piv_authority,
                amount: 0, delegate: COption::None, state: AccountState::Initialized,
                is_native: COption::None, delegated_amount: 0, close_authority: COption::None },
                &mut data).unwrap();
            let mut account = state_account(key, data); account.owner = spl_token::ID; account
        };
        let mut tokens = [token(c.principal_jito_vault), token(c.pending_jito_vault)];
        let mut other_rewards: [BackingAccount; 5] = core::array::from_fn(|i| {
            let mut reward = GuardianReward::new(0, &registry, (i + 1) as u8).unwrap();
            let (key, bump) = reward_address(&reward); reward.bump = bump;
            state_account(key, raw(&reward, REWARD))
        });
        let mut clock = BackingAccount { key: sysvar::clock::ID, owner: sysvar::ID,
            executable: false, signer: false, writable: false, lamports: 0, data: vec![0; 40] };
        Clock { unix_timestamp: c.kif_anchor_timestamp, ..Clock::default() }
            .to_account_info(&mut clock.info()).unwrap();
        {
            let [c, d, r, g] = &mut self.accounts;
            let c = c.info(); let d = d.info(); let r = r.info(); let g = g.info();
            let config = &c; let round = &d; let registry_info = &r; let reward = &g;
            let [p, h, o, e, k] = &mut natives;
            let p = p.info(); let h = h.info(); let o = o.info(); let e = e.info(); let k = k.info();
            let [pt, pn] = &mut tokens; let pt = pt.info(); let pn = pn.info();
            let fixed = authenticate_fixed_accounts(&PROGRAM, &Rent::default(), FixedAccountInfos {
                config, active_distribution: round, pending_sol: &p, principal_sol: &h,
                operational_sol: &o, distribution_escrow: &e, kif_sol: &k,
                principal_jito: &pt, pending_jito: &pn,
            }).unwrap();
            assert_eq!(fixed.config(), &prior.config());
            assert_eq!(*fixed.distribution(), decode::<ActiveDistribution>(&round.data.borrow()));
            let [r1, r2, r3, r4, r5] = &mut other_rewards;
            let r1 = r1.info(); let r2 = r2.info(); let r3 = r3.info(); let r4 = r4.info(); let r5 = r5.info();
            let clock = clock.info();
            let snapshot = authenticate_guardian_clock_snapshot(&PROGRAM, &Rent::default(),
                GuardianClockAccountInfos { config, guardian_registry: registry_info,
                    rewards: [reward, &r1, &r2, &r3, &r4, &r5], clock: &clock }).unwrap();
            assert_eq!(snapshot.registry(), &registry);
            assert_eq!(snapshot.rewards()[0], decode::<GuardianReward>(&reward.data.borrow()));
        }
        assert_eq!(*self, prior);
    }
}

fn summary() -> CompletedDistributionSummary {
    CompletedDistributionSummary { sequence: 1, completed_at: i64::MAX,
        gross_yield_lamports: 2, actual_allocated_outgoing_lamports: 3,
        integrated_contribution_value_lamports: 4, final_protected_hwm_lamports: 5,
        fixed_jitosol_withdrawal_target_units: 6, successful_leg_count: 7,
        cumulative_cooldown_rewards_lamports: 8, actual_kif_liability_lamports: 9,
        actual_kif_carry_next_lamports: 10 }
}

#[test]
fn four_exact_envelopes_round_trip_through_existing_decoders_including_active_round() {
    for active in [false, true] {
        let mut f = Fixture::new(active);
        if active { assert_ne!(f.round().lifecycle, DistributionLifecycle::Idle); }
        for (i, encoded) in f.envelopes().iter().enumerate() {
            assert_eq!(encoded.as_bytes().len(), SIZES[i]);
            assert_eq!(&encoded.as_bytes()[..8], DISCRIMINATORS[i]);
            assert_eq!(encoded.as_bytes(), f.accounts[i].data);
        }
        let plans = f.plans(f.envelopes());
        let before = f.clone(); f.commit(&plans).unwrap(); assert_eq!(f, before);
        f.authenticate_all();
    }
}

#[test]
fn independent_reward_and_registry_field_offsets_match_little_endian_borsh() {
    let f = Fixture::new(false);
    let mut g = f.reward();
    g.registry_revision = 0x0123_4567_89ab_cdef;
    g.last_active_period = Some(0xfedc_ba98_7654_3210);
    g.claimable_lamports = 0x1122_3344_5566_7788;
    g.cumulative_earned = g.claimable_lamports + 5; g.cumulative_claimed = 5;
    let mut expected = DISCRIMINATORS[REWARD].to_vec();
    expected.extend([g.version, g.bump, g.guardian_index]);
    expected.extend(g.registry_revision.to_le_bytes());
    expected.extend(g.guardian.to_bytes()); expected.push(1);
    expected.extend(0xfedc_ba98_7654_3210_u64.to_le_bytes());
    expected.extend(g.claimable_lamports.to_le_bytes());
    expected.extend(g.cumulative_earned.to_le_bytes()); expected.extend(5_u64.to_le_bytes());
    assert_eq!(StateEnvelope::reward(&g).unwrap().as_bytes(), expected);
    let r = f.registry();
    let mut expected = DISCRIMINATORS[REGISTRY].to_vec();
    expected.extend([r.version, r.bump]); expected.extend(r.revision.to_le_bytes());
    for key in r.guardian_keys { expected.extend(key.to_bytes()); }
    assert_eq!(StateEnvelope::registry(&r).unwrap().as_bytes(), expected);
}

#[test]
fn repeated_option_growth_and_shrink_clear_all_config_reward_and_history_tails() {
    let mut f = Fixture::new(false);
    for present in [true, false, true, true, false] {
        let mut c = f.config();
        c.last_successful_preparation_at = present.then_some(i64::MAX);
        c.last_valid_insufficient_attempt_at = present.then_some(i64::MIN);
        c.next_distribution_sequence = 2;
        let mut d = f.round(); d.last_completed = present.then_some(summary());
        let mut g = f.reward(); g.last_active_period = present.then_some(u64::MAX);
        // Shrinking these fields is a codec stress case, not an authorized
        // deletion of history/activity or a valid timing transition.
        let after = [StateEnvelope::config(&c).unwrap(), StateEnvelope::distribution(&d).unwrap(),
            StateEnvelope::registry(&f.registry()).unwrap(), StateEnvelope::reward(&g).unwrap()];
        let payload_lengths = [c.try_to_vec().unwrap().len(), d.try_to_vec().unwrap().len(),
            f.registry().try_to_vec().unwrap().len(), g.try_to_vec().unwrap().len()];
        for (i, envelope) in after.iter().enumerate() {
            let expected_tail = if present { 0 } else { [16, 88, 0, 8][i] };
            assert_eq!(SIZES[i] - 8 - payload_lengths[i], expected_tail);
            assert!(envelope.as_bytes()[8 + payload_lengths[i]..].iter().all(|b| *b == 0));
        }
        let plans = f.plans(after); f.commit(&plans).unwrap();
        assert_eq!(f.config(), c); assert_eq!(f.round(), d); assert_eq!(f.reward(), g);
        f.authenticate_all();
    }
}

#[test]
fn invalid_typed_states_cannot_be_encoded_as_before_or_replacement() {
    let f = Fixture::new(false);
    let mut c = f.config(); c.is_initialized = false;
    assert_eq!(StateEnvelope::config(&c), Err(Piv1Error::InvalidInitialization));
    let mut c = f.config(); c.kif_bps += 1;
    assert_eq!(StateEnvelope::config(&c), Err(Piv1Error::InvalidSplit));
    let mut c = f.config(); c.kif_claim_liability_lamports += 1;
    assert_eq!(StateEnvelope::config(&c), Err(Piv1Error::CumulativeReconciliationMismatch));
    let mut d = f.round(); d.is_initialized = false;
    assert_eq!(StateEnvelope::distribution(&d), Err(Piv1Error::InvalidInitialization));
    let mut d = f.round(); d.active_sequence = 1;
    assert_eq!(StateEnvelope::distribution(&d), Err(Piv1Error::InvalidLifecycle));
    let mut r = f.registry(); r.guardian_keys[5] = r.guardian_keys[0];
    assert_eq!(StateEnvelope::registry(&r), Err(Piv1Error::InvalidGuardianSet));
    let mut g = f.reward(); g.cumulative_claimed = u64::MAX;
    assert_eq!(StateEnvelope::reward(&g), Err(Piv1Error::CumulativeReconciliationMismatch));
    let mut c = f.config(); c.version += 1;
    let mut d = f.round(); d.version += 1;
    let mut r = f.registry(); r.version += 1;
    let mut g = f.reward(); g.version += 1;
    assert_eq!(StateEnvelope::config(&c), Err(Piv1Error::InvalidVersion));
    assert_eq!(StateEnvelope::distribution(&d), Err(Piv1Error::InvalidVersion));
    assert_eq!(StateEnvelope::registry(&r), Err(Piv1Error::InvalidVersion));
    assert_eq!(StateEnvelope::reward(&g), Err(Piv1Error::InvalidVersion));
}

#[test]
fn successful_four_account_batch_changes_exactly_requested_bytes() {
    let mut f = Fixture::new(false); let before = f.clone();
    let plans = f.plans(f.changed()); f.commit(&plans).unwrap();
    for i in 0..4 {
        let mut expected = before.accounts[i].clone(); expected.data = plans[i].replacement().to_vec();
        assert_eq!(f.accounts[i], expected);
        assert_eq!(plans[i].target(), f.accounts[i].key);
        assert_eq!(plans[i].expected_before(), before.accounts[i].data);
    }
}

#[test]
fn stale_entire_before_envelope_and_noop_replay_limits_are_explicit() {
    let mut f = Fixture::new(false);
    let noops = f.plans(f.envelopes()); let before = f.clone();
    f.commit(&noops).unwrap(); f.commit(&noops).unwrap(); assert_eq!(f, before);
    let plans = f.plans(f.changed()); f.commit(&plans).unwrap();
    f.reject(&plans, Piv1Error::StateEnvelopeChanged);
    f.reject(&noops, Piv1Error::StateEnvelopeChanged);
    // Byte persistence has no monotonic revision oracle: a no-op may repeat.
    // Authorized claim counters/lifecycle rules remain the caller's concern.
    assert_eq!(commit_state_writes::<0>(&PROGRAM, &Rent::default(), []), Ok(()));
}

#[test]
fn malformed_discriminators_versions_options_and_nonzero_padding_reject_unchanged() {
    for i in 0..4 {
        for offset in [0, 8] {
            let mut f = Fixture::new(false); let plans = f.plans(f.changed());
            f.accounts[i].data[offset] ^= 0xff;
            f.reject(&plans, Piv1Error::StateEnvelopeChanged);
        }
    }
    for (i, option_offset) in [(CONFIG, 756), (ROUND, 22), (REWARD, 51)] {
        let mut f = Fixture::new(false); let plans = f.plans(f.changed());
        assert!(f.accounts[i].data[option_offset] <= 1);
        // Independent offsets: Config 8 + 3 + 11 + 22*32 + 7*2 + 2*8;
        // round 8 + 6 + 8; reward 8 + 3 + 8 + 32.
        f.accounts[i].data[option_offset] = 2;
        let bytes = &f.accounts[i].data[8..];
        match i {
            CONFIG => assert!(PivConfig::deserialize(&mut &*bytes).is_err()),
            ROUND => assert!(ActiveDistribution::deserialize(&mut &*bytes).is_err()),
            _ => assert!(GuardianReward::deserialize(&mut &*bytes).is_err()),
        }
        f.reject(&plans, Piv1Error::StateEnvelopeChanged);
    }
    for i in [CONFIG, ROUND, REWARD] {
        let mut f = Fixture::new(false);
        if i == CONFIG {
            let mut c = f.config(); c.last_successful_preparation_at = None;
            c.last_valid_insufficient_attempt_at = None; f.accounts[i].data = raw(&c, i);
        }
        if i == REWARD { let mut g = f.reward(); g.last_active_period = None; f.accounts[i].data = raw(&g, i); }
        let plans = f.plans(f.changed());
        assert_eq!(*f.accounts[i].data.last().unwrap(), 0);
        *f.accounts[i].data.last_mut().unwrap() = 1;
        f.reject(&plans, Piv1Error::StateEnvelopeChanged);
    }
}

#[test]
fn each_metadata_or_rent_failure_including_last_account_prevents_all_writes() {
    for index in 0..4 {
        for case in 0..6 {
            let mut f = Fixture::new(false); let plans = f.plans(f.changed());
            assert_ne!(plans[0].expected_before(), plans[0].replacement());
            let error = match case {
                0 => { f.accounts[index].key = key(201); Piv1Error::InvalidAccountPda }
                1 => { f.accounts[index].owner = key(202); Piv1Error::InvalidAccountOwner }
                2 => { f.accounts[index].executable = true; Piv1Error::ExecutableAccount }
                3 => { f.accounts[index].writable = false; Piv1Error::AccountNotWritable }
                4 => { f.accounts[index].lamports -= 1; Piv1Error::AccountRentDeficit }
                _ => { f.accounts[index].data[8] ^= 1; Piv1Error::StateEnvelopeChanged }
            };
            f.reject(&plans, error);
        }
    }
}

#[test]
fn each_allocation_must_match_exactly_before_any_copy() {
    for index in 0..4 {
        for longer in [false, true] {
            let mut f = Fixture::new(false); let plans = f.plans(f.changed());
            if longer { f.accounts[index].data.push(0); } else { f.accounts[index].data.pop(); }
            f.reject(&plans, Piv1Error::InvalidAccountSize);
        }
    }
}

#[test]
fn every_data_and_lamport_borrow_conflict_rolls_back_a_nonnoop_batch() {
    for index in 0..4 {
        for kind in 0..3 {
            let mut f = Fixture::new(false); let plans = f.plans(f.changed()); let before = f.clone();
            assert_ne!(plans[0].expected_before(), plans[0].replacement());
            f.with_infos(|accounts| {
                let data_read = (kind == 0).then(|| accounts[index].try_borrow_data().unwrap());
                let data_write = (kind == 1).then(|| accounts[index].try_borrow_mut_data().unwrap());
                let lamport_write = (kind == 2).then(|| accounts[index].try_borrow_mut_lamports().unwrap());
                assert_eq!(commit_state_writes(&PROGRAM, &Rent::default(),
                    core::array::from_fn::<_, 4, _>(|i| (&plans[i], accounts[i]))),
                    Err(Piv1Error::AccountBorrowFailed));
                drop((data_read, data_write, lamport_write));
                // All earlier temporary mutable borrows must also be released.
                for account in accounts { assert!(account.try_borrow_mut_data().is_ok()); }
            });
            assert_eq!(f, before);
        }
    }
}

#[test]
fn shared_lamport_reads_and_excess_do_not_block_state_only_writes() {
    let mut f = Fixture::new(false);
    for (i, account) in f.accounts.iter_mut().enumerate() { account.lamports += 100 + i as u64; }
    let plans = f.plans(f.changed()); let before = f.clone();
    f.with_infos(|accounts| {
        let held_reads: [_; 4] = core::array::from_fn(|i| accounts[i].try_borrow_lamports().unwrap());
        commit_state_writes(&PROGRAM, &Rent::default(),
            core::array::from_fn::<_, 4, _>(|i| (&plans[i], accounts[i]))).unwrap();
        for i in 0..4 { assert_eq!(**held_reads[i], before.accounts[i].lamports); }
    });
    for i in 0..4 { assert_eq!(f.accounts[i].lamports, before.accounts[i].lamports); }
}

#[test]
fn invalid_runtime_rent_rejects_before_any_mutation() {
    let mut rents = vec![
        Rent { exemption_threshold: f64::NAN, ..Rent::default() },
        Rent { exemption_threshold: f64::INFINITY, ..Rent::default() },
        Rent { exemption_threshold: -1.0, ..Rent::default() },
        Rent { burn_percent: 101, ..Rent::default() },
    ];
    for rent in rents.drain(..) {
        let mut f = Fixture::new(false); let plans = f.plans(f.changed()); let before = f.clone();
        f.with_infos(|accounts| assert_eq!(commit_state_writes(&PROGRAM, &rent,
            core::array::from_fn::<_, 4, _>(|i| (&plans[i], accounts[i]))), Err(Piv1Error::InvalidRent)));
        assert_eq!(f, before);
    }
    for rent in [Rent { lamports_per_byte_year: u64::MAX, ..Rent::default() },
        Rent { exemption_threshold: f64::MAX, ..Rent::default() }]
    {
        let mut f = Fixture::new(false); let plans = f.plans(f.changed()); let before = f.clone();
        f.with_infos(|accounts| assert_eq!(commit_state_writes(&PROGRAM, &rent,
            core::array::from_fn::<_, 4, _>(|i| (&plans[i], accounts[i]))), Err(Piv1Error::ArithmeticOverflow)));
        assert_eq!(f, before);
    }
}

#[test]
fn duplicate_keys_and_differently_keyed_shared_data_are_both_rejected() {
    for left in 0..4 {
        for right in left + 1..4 {
            for shared_data in [false, true] {
                let mut f = Fixture::new(false); let plans = f.plans(f.changed()); let before = f.clone();
                f.with_infos(|accounts| {
                    let mut alias = if shared_data { accounts[left].clone() } else { accounts[right].clone() };
                    alias.key = if shared_data { accounts[right].key } else { accounts[left].key };
                    if shared_data {
                        assert_ne!(accounts[left].key, alias.key);
                        assert!(std::rc::Rc::ptr_eq(&accounts[left].data, &alias.data));
                    } else {
                        assert_eq!(accounts[left].key, alias.key);
                        assert!(!std::rc::Rc::ptr_eq(&accounts[left].data, &alias.data));
                    }
                    let batch = core::array::from_fn::<_, 4, _>(|i|
                        (&plans[i], if i == right { &alias } else { accounts[i] }));
                    assert_eq!(commit_state_writes(&PROGRAM, &Rent::default(), batch), Err(Piv1Error::AccountAlias));
                });
                assert_eq!(f, before);
            }
        }
    }
}

#[test]
fn duplicate_plan_and_account_cannot_be_applied_twice() {
    let mut f = Fixture::new(false); let plans = f.plans(f.changed()); let before = f.clone();
    f.with_infos(|a| assert_eq!(commit_state_writes(&PROGRAM, &Rent::default(),
        [(&plans[0], a[0]), (&plans[0], a[0])]), Err(Piv1Error::AccountAlias)));
    assert_eq!(f, before);
}

#[test]
fn trusted_program_is_rechecked_for_every_prepared_record() {
    for wrong in [system_program::ID, spl_token::ID, STAKE_PROGRAM_ID, key(211)] {
        let mut f = Fixture::new(false); let plans = f.plans(f.changed()); let before = f.clone();
        f.with_infos(|a| assert_eq!(commit_state_writes(&wrong, &Rent::default(),
            core::array::from_fn::<_, 4, _>(|i| (&plans[i], a[i]))), Err(Piv1Error::InvalidProgramIdentity)));
        assert_eq!(f, before);
    }
    let mut f = Fixture::new(false); let plans = f.plans(f.changed()); let before = f.clone();
    let other_program = key(212);
    let (other_key, bump) = Pubkey::find_program_address(&[b"guardian-registry"], &other_program);
    let mut registry = f.registry(); registry.bump = bump;
    let mut replacement = registry; replacement.revision += 1;
    let other_plan = PreparedStateWrite::new(&other_program, other_key,
        StateEnvelope::registry(&registry).unwrap(), StateEnvelope::registry(&replacement).unwrap()).unwrap();
    let mut other = state_account(other_key, raw(&registry, REGISTRY)); other.owner = other_program;
    let other_before = other.clone();
    {
        let [c, d, _, g] = &mut f.accounts;
        let c = c.info(); let d = d.info(); let g = g.info(); let other = other.info();
        assert_eq!(commit_state_writes(&PROGRAM, &Rent::default(),
            [(&plans[0], &c), (&plans[1], &d), (&plans[3], &g), (&other_plan, &other)]),
            Err(Piv1Error::InvalidProgramIdentity));
    }
    assert_eq!(f, before); assert_eq!(other, other_before);
}

#[test]
fn preparation_rejects_wrong_program_key_and_canonical_bump_for_both_states() {
    let f = Fixture::new(false);
    for program in [system_program::ID, spl_token::ID, STAKE_PROGRAM_ID] {
        assert_eq!(PreparedStateWrite::new(&program, f.accounts[CONFIG].key,
            StateEnvelope::config(&f.config()).unwrap(), StateEnvelope::config(&f.config()).unwrap()),
            Err(Piv1Error::InvalidProgramIdentity));
    }
    for i in 0..4 {
        for before_bad in [false, true] {
            let valid = f.envelopes().into_iter().nth(i).unwrap();
            let bad = match i {
                CONFIG => { let mut c = f.config(); c.bumps.config ^= 1; StateEnvelope::config(&c).unwrap() }
                ROUND => { let mut d = f.round(); d.bump ^= 1; StateEnvelope::distribution(&d).unwrap() }
                REGISTRY => { let mut r = f.registry(); r.bump ^= 1; StateEnvelope::registry(&r).unwrap() }
                _ => { let mut g = f.reward(); g.bump ^= 1; StateEnvelope::reward(&g).unwrap() }
            };
            let (before, after) = if before_bad { (bad, valid) } else { (valid, bad) };
            assert_eq!(PreparedStateWrite::new(&PROGRAM, f.accounts[i].key, before, after),
                Err(Piv1Error::InvalidAccountPda));
        }
        let a = f.envelopes().into_iter().nth(i).unwrap();
        let b = f.envelopes().into_iter().nth(i).unwrap();
        assert_eq!(PreparedStateWrite::new(&PROGRAM, key(200), a, b), Err(Piv1Error::InvalidAccountPda));
    }
    assert_eq!(PreparedStateWrite::new(&PROGRAM, f.accounts[CONFIG].key,
        StateEnvelope::config(&f.config()).unwrap(), StateEnvelope::distribution(&f.round()).unwrap()),
        Err(Piv1Error::InvalidAccountPda));
}

#[test]
fn valid_noncanonical_program_addresses_cannot_replace_canonical_bumps() {
    let f = Fixture::new(false);
    let reward = f.reward(); let revision = reward.registry_revision.to_le_bytes();
    let index = [reward.guardian_index];
    for i in 0..4 {
        let seeds: Vec<&[u8]> = match i {
            CONFIG => vec![b"config"], ROUND => vec![b"distribution"], REGISTRY => vec![b"guardian-registry"],
            _ => vec![b"guardian-reward", reward.guardian.as_ref(), &revision, &index],
        };
        let (_, canonical) = Pubkey::find_program_address(&seeds, &PROGRAM);
        let (address, bump) = (0..canonical).rev().find_map(|bump| {
            let b = [bump]; let mut s = seeds.clone(); s.push(&b);
            Pubkey::create_program_address(&s, &PROGRAM).ok().map(|address| (address, bump))
        }).unwrap();
        assert_ne!(address, f.accounts[i].key); assert!(!address.is_on_curve());
        let make = || match i {
            CONFIG => { let mut c = f.config(); c.bumps.config = bump; StateEnvelope::config(&c).unwrap() }
            ROUND => { let mut d = f.round(); d.bump = bump; StateEnvelope::distribution(&d).unwrap() }
            REGISTRY => { let mut r = f.registry(); r.bump = bump; StateEnvelope::registry(&r).unwrap() }
            _ => { let mut g = f.reward(); g.bump = bump; StateEnvelope::reward(&g).unwrap() }
        };
        assert_eq!(PreparedStateWrite::new(&PROGRAM, address, make(), make()), Err(Piv1Error::InvalidAccountPda));
    }
}

#[test]
fn historical_reward_identity_cannot_be_rebound_but_fixed_registry_content_can_change() {
    let f = Fixture::new(false);
    for component in 0..3 {
        let mut g = f.reward();
        match component { 0 => g.guardian = key(201), 1 => g.registry_revision += 1, _ => g.guardian_index = 5 }
        g.bump = reward_address(&g).1;
        assert_eq!(PreparedStateWrite::new(&PROGRAM, f.accounts[REWARD].key,
            StateEnvelope::reward(&f.reward()).unwrap(), StateEnvelope::reward(&g).unwrap()),
            Err(Piv1Error::InvalidAccountPda));
    }
    let mut f = f;
    let mut registry = f.registry(); registry.revision += 100; registry.guardian_keys[0] = key(202);
    let plan = PreparedStateWrite::new(&PROGRAM, f.accounts[REGISTRY].key,
        StateEnvelope::registry(&f.registry()).unwrap(), StateEnvelope::registry(&registry).unwrap()).unwrap();
    let before = f.clone();
    f.with_infos(|a| commit_state_writes(&PROGRAM, &Rent::default(), [(&plan, a[REGISTRY])])).unwrap();
    assert_eq!(f.registry(), registry);
    for i in [CONFIG, ROUND, REWARD] { assert_eq!(f.accounts[i], before.accounts[i]); }
    assert_eq!(f.accounts[REGISTRY].lamports, before.accounts[REGISTRY].lamports);
    // This succeeds structurally; rotation authorization and Config synchronization
    // are deliberately external, and current snapshot authentication would fail.
}

#[test]
fn claim_derived_two_account_persistence_changes_bytes_without_a_native_transfer() {
    let mut f = ClaimFixture::new(7);
    let authenticated = f.authenticate().unwrap();
    let before_config = authenticated.config().clone(); let before_reward = *authenticated.reward();
    assert_ne!(before_config.guardian_registry_revision, before_reward.registry_revision);
    assert_eq!(before_reward.last_active_period, None);
    let mut next_config = before_config.clone(); let mut next_reward = before_reward;
    let request = KifClaimRequest { amount_lamports: 100,
        expected_cumulative_claimed: before_reward.cumulative_claimed };
    let claim = prepare_kif_claim(&next_config, &next_reward, request, authenticated.custody()).unwrap();
    // Synthetic pure after-observation solely derives accepted bookkeeping.
    // It is NOT an observed transfer, runtime receipt or continuous audit proof.
    let mut modeled_after = authenticated.custody();
    modeled_after.kif_sol.lamports -= 100; modeled_after.guardian_lamports += 100;
    claim.commit(&mut next_config, &mut next_reward, modeled_after).unwrap();
    let config_write = PreparedStateWrite::new(&PROGRAM, f.accounts[0].key,
        StateEnvelope::config(&before_config).unwrap(), StateEnvelope::config(&next_config).unwrap()).unwrap();
    let reward_write = PreparedStateWrite::new(&PROGRAM, f.accounts[1].key,
        StateEnvelope::reward(&before_reward).unwrap(), StateEnvelope::reward(&next_reward).unwrap()).unwrap();
    let before = f.clone();
    f.with_infos(|a| commit_state_writes(&PROGRAM, &Rent::default(),
        [(&config_write, a.config), (&reward_write, a.guardian_reward)])).unwrap();
    let after = f.authenticate().unwrap();
    assert_eq!(after.config(), &next_config); assert_eq!(after.reward(), &next_reward);
    for i in 0..4 {
        let mut expected = before.accounts[i].clone();
        if i == 0 { expected.data = config_write.replacement().to_vec(); }
        if i == 1 { expected.data = reward_write.replacement().to_vec(); }
        assert_eq!(f.accounts[i], expected);
        assert_eq!(f.accounts[i].lamports, before.accounts[i].lamports);
    }
    assert_eq!(after.custody(), authenticated.custody());
    assert_eq!(f.audit, before.audit);
    // The untouched original audit correctly detects bookkeeping without payment.
    assert!(f.validate_audit().is_err());
    assert_eq!(prepare_kif_claim(after.config(), after.reward(), request, after.custody()),
        Err(Piv1Error::StaleKifClaim));
    let before_replay = f.clone();
    f.with_infos(|a| assert_eq!(commit_state_writes(&PROGRAM, &Rent::default(),
        [(&config_write, a.config), (&reward_write, a.guardian_reward)]), Err(Piv1Error::StateEnvelopeChanged)));
    assert_eq!(f, before_replay);
}

#[test]
fn independent_config_and_round_prefixes_pin_optional_field_boundaries() {
    let f = Fixture::new(false);
    let mut c = f.config();
    c.last_successful_preparation_at = Some(i64::MIN);
    c.last_valid_insufficient_attempt_at = Some(i64::MAX);
    c.next_distribution_sequence = 0x0123_4567_89ab_cdef;
    let encoded = StateEnvelope::config(&c).unwrap(); let bytes = encoded.as_bytes();
    assert_eq!(&bytes[..8], DISCRIMINATORS[CONFIG]);
    assert_eq!(&bytes[8..11], [c.version, 1, u8::from(c.paused)]);
    assert_eq!(&bytes[11..22], [c.bumps.config, c.bumps.piv_authority, c.bumps.active_distribution,
        c.bumps.principal_jito_vault, c.bumps.pending_jito_vault, c.bumps.pending_sol_vault,
        c.bumps.principal_sol_queue, c.bumps.operational_sol_vault, c.bumps.distribution_escrow,
        c.bumps.kif_sol_vault, c.bumps.guardian_registry]);
    assert_eq!(&bytes[22..54], c.stake_pool_program.as_ref());
    assert_eq!(&bytes[694..726], c.guardian_registry.as_ref());
    assert_eq!(&bytes[726..740], [0x10, 0x27, 0x0c, 0x17, 0x9e, 0x07,
        0x9e, 0x07, 0xc8, 0, 1, 0, 1, 0]);
    assert_eq!(bytes[756], 1); assert_eq!(&bytes[757..765], i64::MIN.to_le_bytes());
    assert_eq!(bytes[765], 1); assert_eq!(&bytes[766..774], i64::MAX.to_le_bytes());
    assert_eq!(&bytes[774..782], 0x0123_4567_89ab_cdef_u64.to_le_bytes());
    let mut d = f.round(); d.last_completed = Some(summary());
    let encoded = StateEnvelope::distribution(&d).unwrap(); let bytes = encoded.as_bytes();
    assert_eq!(&bytes[..8], DISCRIMINATORS[ROUND]);
    assert_eq!(&bytes[8..14], [d.version, d.bump, 1, 0, 0, 0]);
    assert_eq!(&bytes[14..22], 0_u64.to_le_bytes()); assert_eq!(bytes[22], 1);
    let mut expected_summary = 1_u64.to_le_bytes().to_vec();
    expected_summary.extend(i64::MAX.to_le_bytes());
    for value in 2_u64..=10 { expected_summary.extend(value.to_le_bytes()); }
    assert_eq!(&bytes[23..111], expected_summary);
    assert_eq!(bytes.len(), 891);
}

#[test]
fn structurally_valid_stale_fields_in_each_record_reject_the_full_batch() {
    for index in 0..4 {
        let mut f = Fixture::new(false); let plans = f.plans(f.changed());
        let changed = match index {
            CONFIG => { let mut c = f.config(); c.protected_principal_hwm_lamports += 1;
                StateEnvelope::config(&c).unwrap() }
            ROUND => { let mut d = f.round(); d.last_completed = Some(summary());
                StateEnvelope::distribution(&d).unwrap() }
            REGISTRY => { let mut r = f.registry(); r.guardian_keys.swap(0, 1);
                StateEnvelope::registry(&r).unwrap() }
            _ => { let mut g = f.reward(); g.cumulative_earned += 1; g.cumulative_claimed += 1;
                StateEnvelope::reward(&g).unwrap() }
        };
        assert_ne!(changed.as_bytes(), plans[index].expected_before());
        f.accounts[index].data = changed.as_bytes().to_vec();
        f.reject(&plans, Piv1Error::StateEnvelopeChanged);
    }
}
