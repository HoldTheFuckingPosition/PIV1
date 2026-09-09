mod support;

use anchor_lang::{prelude::{Pubkey, Rent}, solana_program::system_program, AnchorSerialize};
use piv1::{
    accounts::STAKE_PROGRAM_ID,
    errors::{Piv1Error, Piv1Result},
    integrations::FeeFraction,
    kif_claim_accounts::*,
    state::{*, reconciliation::SolVaultBalance},
};
use support::kif_claim_custody::*;

fn auth_rejected(f: &mut Fixture, error: Piv1Error) {
    let before = f.clone();
    assert_eq!(f.authenticate(), Err(error));
    assert_eq!(*f, before);
}

fn rejected(f: &mut Fixture, amount: u64, counter: u64) -> Error {
    let before = f.clone();
    let error = f.claim(amount, counter).unwrap_err();
    assert_eq!(*f, before, "all bytes, lamports and the original audit must roll back");
    error
}

fn success(f: &mut Fixture, amount: u64) {
    let mut expected = f.clone();
    let counter = f.reward().cumulative_claimed;
    expected.update_config(|c| {
        c.kif_claim_liability_lamports -= amount;
        c.cumulative_kif_claimed_lamports += amount;
    });
    expected.update_reward(|r| {
        r.claimable_lamports -= amount;
        r.cumulative_claimed += amount;
    });
    expected.accounts[KIF].lamports -= amount;
    expected.accounts[GUARDIAN].lamports += amount;
    expected.audit.paid += u128::from(amount);
    assert_eq!(f.claim(amount, counter).unwrap(), KifClaimTransfer {
        source: expected.accounts[KIF].key, destination: expected.accounts[GUARDIAN].key,
        amount_lamports: amount,
    });
    assert_eq!(*f, expected, "only four accounting fields and the fixed native transfer may change");
    f.validate_audit().unwrap();
}

fn prepared(f: &mut Fixture, amount: u64) -> (PivConfig, GuardianReward, KifClaimCustodyObservation, PreparedKifClaim) {
    let state = f.authenticate().unwrap();
    let c = state.config().clone(); let r = *state.reward();
    let custody = state.custody();
    let plan = prepare_kif_claim(&c, &r, KifClaimRequest {
        amount_lamports: amount, expected_cumulative_claimed: r.cumulative_claimed,
    }, custody).unwrap();
    let after = KifClaimCustodyObservation {
        kif_sol: SolVaultBalance { lamports: custody.kif_sol.lamports - amount, ..custody.kif_sol },
        guardian_lamports: custody.guardian_lamports + amount,
    };
    (c, r, after, plan)
}

fn pure_rejected(c: &PivConfig, r: &GuardianReward, custody: KifClaimCustodyObservation,
                 amount: u64, counter: u64) -> Piv1Error {
    let before_c = c.clone(); let before_r = *r;
    let error = prepare_kif_claim(c, r, KifClaimRequest {
        amount_lamports: amount, expected_cumulative_claimed: counter,
    }, custody).unwrap_err();
    assert_eq!(*c, before_c); assert_eq!(*r, before_r);
    error
}

fn rebind_tuple(f: &mut Fixture, edit: impl FnOnce(&mut GuardianReward)) {
    let mut r = f.reward(); edit(&mut r);
    let (address, bump) = Pubkey::find_program_address(&[
        b"guardian-reward", r.guardian.as_ref(), &r.registry_revision.to_le_bytes(), &[r.guardian_index],
    ], &f.program_id);
    r.bump = bump;
    f.accounts[REWARD].key = address;
    f.accounts[REWARD].data = envelope(&r, GUARDIAN_REWARD_DISCRIMINATOR, GuardianReward::SPACE);
    f.accounts[GUARDIAN].key = r.guardian;
}

#[test]
fn isolated_four_account_authentication_is_read_only_and_claims_preserve_complete_state() {
    for paused in [false, true] {
        for excess in [0, 1, 17, 100_000] {
            let mut f = Fixture::new(excess);
            f.update_config(|c| c.paused = paused);
            let before = f.clone();
            let state = f.authenticate().unwrap();
            assert_eq!(f, before);
            assert_eq!(state.reward().last_active_period, None);
            assert_eq!(state.reward().registry_revision, 7);
            assert_eq!(state.config().guardian_registry_revision, 99);
            assert_eq!(state.custody().kif_sol.economic_lamports().unwrap(), 960 + 73 + excess);
            success(&mut f, 1);
            success(&mut f, 49);
            success(&mut f, 250);
            assert_eq!(f.reward().claimable_lamports, 0);
            assert_eq!(f.config().kif_claim_liability_lamports, 660);
            assert_eq!(f.config().collective_kif_carry_lamports, 73);
            assert_eq!(f.accounts[KIF].lamports, f.rent.minimum_balance(0) + 660 + 73 + excess);
        }
    }
}


#[test]
fn full_aggregate_claim_leaves_exact_rent_plus_collective_carry() {
    let base = Fixture::new(0); let mut config = base.config(); let reward = base.reward();
    config.cumulative_kif_credited_lamports = reward.cumulative_earned;
    config.cumulative_kif_claimed_lamports = reward.cumulative_claimed;
    config.kif_claim_liability_lamports = reward.claimable_lamports;
    for carry in [0, 1, 73] {
        config.collective_kif_carry_lamports = carry;
        let mut f = Fixture::from_earned(config.clone(), reward, 0);
        f.update_config(|c| c.paused = true);
        success(&mut f, 300);
        assert_eq!(f.config().kif_claim_liability_lamports, 0);
        assert_eq!(f.accounts[KIF].lamports, f.rent.minimum_balance(0) + carry);
        assert_eq!(rejected(&mut f, 1, 310), Error::State(Piv1Error::KifClaimExceeded));
    }
}

#[test]
fn unsolicited_lamport_excess_never_blocks_a_paused_claim_or_becomes_payment() {
    let mut f = Fixture::new(0);
    f.update_config(|c| c.paused = true);
    f.unsolicited_excess(1).unwrap();
    success(&mut f, 100);
    f.unsolicited_excess(31).unwrap();
    success(&mut f, 200);
    assert_eq!(f.audit.external_excess, 32);
    assert_eq!(f.accounts[KIF].lamports, f.rent.minimum_balance(0) + 660 + 73 + 32);
}

#[test]
fn zero_overclaim_full_claim_and_stale_partial_replay_are_atomic() {
    let mut f = Fixture::new(0);
    assert_eq!(rejected(&mut f, 0, 10), Error::State(Piv1Error::ZeroKifClaim));
    assert_eq!(rejected(&mut f, 301, 10), Error::State(Piv1Error::KifClaimExceeded));
    for stale in [0, 9, 11, u64::MAX] {
        assert_eq!(rejected(&mut f, 1, stale), Error::State(Piv1Error::StaleKifClaim));
    }
    success(&mut f, 50);
    assert_eq!(rejected(&mut f, 50, 10), Error::State(Piv1Error::StaleKifClaim));
    success(&mut f, 250);
    assert_eq!(rejected(&mut f, 1, 310), Error::State(Piv1Error::KifClaimExceeded));
    assert_eq!(rejected(&mut f, 300, 10), Error::State(Piv1Error::StaleKifClaim));
}

#[test]
fn later_modeled_credit_cannot_reenable_an_old_claim_even_if_claimable_is_restored() {
    let mut f = Fixture::new(3);
    success(&mut f, 50);
    f.credit_snapshot(50).unwrap();
    assert_eq!(f.reward().claimable_lamports, 300);
    assert_eq!(f.reward().cumulative_claimed, 60);
    assert_eq!(rejected(&mut f, 50, 10), Error::State(Piv1Error::StaleKifClaim));
    success(&mut f, 300);
    f.credit_snapshot(7).unwrap();
    assert_eq!(rejected(&mut f, 7, 60), Error::State(Piv1Error::StaleKifClaim));
    success(&mut f, 7);
    assert_eq!(f.audit.credited, 57);
    assert_eq!(f.audit.paid, 357);
}

#[test]
fn prepared_effects_reject_any_changed_pre_state_including_credit_and_other_claims() {
    let mutations: &[fn(&mut PivConfig, &mut GuardianReward)] = &[
        |c, _| c.paused = !c.paused,
        |c, _| c.protected_principal_hwm_lamports += 1,
        |c, _| c.guardian_registry_revision += 1,
        |_, r| r.last_active_period = Some(7),
        |_, r| r.registry_revision += 1,
        |c, r| {
            r.credit_snapshot(r.guardian, r.guardian_index, r.registry_revision, 5).unwrap();
            c.cumulative_kif_credited_lamports += 5;
            c.kif_claim_liability_lamports += 5;
        },
        // A different earned ledger's claim changes the captured global pair.
        |c, _| { c.kif_claim_liability_lamports -= 1; c.cumulative_kif_claimed_lamports += 1; },
    ];
    for mutate in mutations {
        let (mut c, mut r, after, plan) = prepared(&mut Fixture::new(0), 50);
        mutate(&mut c, &mut r);
        let before_c = c.clone(); let before_r = r;
        assert_eq!(plan.commit(&mut c, &mut r, after), Err(Piv1Error::KifClaimStateChanged));
        assert_eq!(c, before_c); assert_eq!(r, before_r);
    }
    let (mut c, mut r, after, plan) = prepared(&mut Fixture::new(0), 50);
    let replay = plan.clone();
    plan.commit(&mut c, &mut r, after).unwrap();
    let before_c = c.clone(); let before_r = r;
    assert_eq!(replay.commit(&mut c, &mut r, after), Err(Piv1Error::KifClaimStateChanged));
    assert_eq!(c, before_c); assert_eq!(r, before_r);
}

#[test]
fn exact_endpoint_deltas_and_unchanged_floor_are_required_even_when_total_is_preserved() {
    let mutations: &[fn(&mut KifClaimCustodyObservation)] = &[
        |o| o.kif_sol.lamports += 1,
        |o| o.kif_sol.lamports -= 1,
        |o| o.guardian_lamports += 1,
        |o| o.guardian_lamports -= 1,
        |o| { o.kif_sol.lamports += 1; o.guardian_lamports -= 1; },
        |o| { o.kif_sol.lamports -= 1; o.guardian_lamports += 1; },
        |o| o.kif_sol.non_economic_floor_lamports += 1,
        |o| { o.kif_sol.non_economic_floor_lamports += 1; o.kif_sol.lamports += 1; },
    ];
    for mutate in mutations {
        let (mut c, mut r, mut after, plan) = prepared(&mut Fixture::new(17), 50);
        mutate(&mut after);
        let before_c = c.clone(); let before_r = r;
        assert_eq!(plan.commit(&mut c, &mut r, after), Err(Piv1Error::KifClaimObservationMismatch));
        assert_eq!(c, before_c); assert_eq!(r, before_r);
    }
}

#[test]
fn every_transfer_state_envelope_and_late_commit_failure_rolls_back_and_retries() {
    for failure in [Failure::AfterPrepare, Failure::AfterDebit, Failure::MissingCredit,
                    Failure::WrongCredit, Failure::AfterCredit, Failure::StateChanged,
                    Failure::AfterState, Failure::InvalidPostEnvelope, Failure::BeforeCommit] {
        let mut f = Fixture::new(0);
        f.failure = Some(failure);
        rejected(&mut f, 50, 10);
        f.validate_audit().unwrap();
        f.failure = None;
        success(&mut f, 50);
    }
}

#[test]
fn global_backing_includes_other_guardians_carry_and_rent_before_any_payment() {
    for missing in [1, 73, 660, 960, 1_000] {
        let mut f = Fixture::new(0);
        f.accounts[KIF].lamports -= missing;
        f.accounts[GUARDIAN].lamports += missing;
        assert!(f.accounts[KIF].lamports > 1);
        assert_eq!(rejected(&mut f, 1, 10), Error::State(Piv1Error::KifClaimBackingDeficit));
    }
    let mut f = Fixture::new(0);
    f.accounts[KIF].lamports = f.rent.minimum_balance(0) - 1;
    assert_eq!(rejected(&mut f, 1, 10), Error::State(Piv1Error::AccountRentDeficit));
}

#[test]
fn malformed_individual_and_global_identities_and_component_bounds_reject() {
    let reward_changes: &[fn(&mut GuardianReward)] = &[
        |r| r.claimable_lamports -= 1,
        |r| r.cumulative_earned -= 1,
        |r| r.cumulative_claimed += 1,
        |r| r.cumulative_claimed = r.cumulative_earned + 1,
        |r| { r.cumulative_claimed = 41; r.cumulative_earned = 341; },
        |r| { r.claimable_lamports = 961; r.cumulative_earned = 971; },
        |r| { r.cumulative_claimed = 701; r.cumulative_earned = 1_001; },
    ];
    for edit in reward_changes {
        let mut f = Fixture::new(0); f.update_reward(edit);
        assert_eq!(rejected(&mut f, 1, 10), Error::State(Piv1Error::CumulativeReconciliationMismatch));
    }
    let global_changes: &[fn(&mut PivConfig)] = &[
        |c| c.kif_claim_liability_lamports -= 1,
        |c| c.cumulative_kif_credited_lamports -= 1,
        |c| c.cumulative_kif_claimed_lamports += 1,
        |c| c.cumulative_kif_claimed_lamports = c.cumulative_kif_credited_lamports + 1,
        |c| { c.kif_claim_liability_lamports = 299; c.cumulative_kif_credited_lamports = 339; },
        |c| { c.cumulative_kif_claimed_lamports = 9; c.cumulative_kif_credited_lamports = 969; },
    ];
    for edit in global_changes {
        let mut f = Fixture::new(0); f.update_config(edit);
        assert_eq!(rejected(&mut f, 1, 10), Error::State(Piv1Error::CumulativeReconciliationMismatch));
    }
}

#[test]
fn every_claim_account_requires_its_owner_nonexecutable_shape_and_writable_privilege() {
    for index in 0..4 {
        let mut f = Fixture::new(0);
        f.accounts[index].owner = key(222);
        auth_rejected(&mut f, Piv1Error::InvalidAccountOwner);
        let mut f = Fixture::new(0);
        f.accounts[index].executable = true;
        auth_rejected(&mut f, Piv1Error::ExecutableAccount);
        let mut f = Fixture::new(0);
        f.accounts[index].writable = false;
        auth_rejected(&mut f, Piv1Error::AccountNotWritable);
    }
    let mut f = Fixture::new(0);
    f.accounts[GUARDIAN].signer = false;
    auth_rejected(&mut f, Piv1Error::MissingGuardianSignature);
    for index in [KIF, GUARDIAN] {
        let mut f = Fixture::new(0);
        f.accounts[index].data.push(0);
        auth_rejected(&mut f, Piv1Error::InvalidAccountSize);
    }
}

#[test]
fn fixed_config_reward_and_kif_addresses_bumps_and_immutable_tuple_are_authenticated() {
    for index in [CONFIG, REWARD, KIF] {
        let mut f = Fixture::new(0); f.accounts[index].key = key(222);
        auth_rejected(&mut f, Piv1Error::InvalidAccountPda);
    }
    for change in [0, 1, 2] {
        let mut f = Fixture::new(0);
        f.update_config(|c| match change { 0 => c.bumps.config ^= 1,
            1 => c.bumps.kif_sol_vault ^= 1, _ => c.kif_sol_vault = key(222) });
        auth_rejected(&mut f, Piv1Error::InvalidAccountPda);
    }
    let changes: &[fn(&mut GuardianReward)] = &[
        |r| r.bump ^= 1,
        |r| r.guardian = key(222),
        |r| r.guardian_index = 3,
        |r| r.registry_revision += 1,
    ];
    for change in changes {
        let mut f = Fixture::new(0); f.update_reward(change);
        auth_rejected(&mut f, Piv1Error::InvalidAccountPda);
    }
    let mut f = Fixture::new(0);
    f.accounts[GUARDIAN].key = key(222);
    auth_rejected(&mut f, Piv1Error::InvalidGuardianSet);
}

#[test]
fn all_four_actual_roles_are_distinct_and_guardian_cannot_target_piv_controlled_roles() {
    for left in 0..4 {
        for right in left + 1..4 {
            let mut f = Fixture::new(0);
            f.accounts[left].key = f.accounts[right].key;
            auth_rejected(&mut f, Piv1Error::AccountAlias);
        }
    }
    let f = Fixture::new(0); let c = f.config();
    for destination in [c.piv_authority, c.active_distribution, c.principal_jito_vault,
                        c.pending_jito_vault, c.pending_sol_vault, c.principal_sol_queue,
                        c.operational_sol_vault, c.distribution_escrow, c.kif_sol_vault,
                        c.guardian_registry, f.accounts[CONFIG].key] {
        let mut invalid = Fixture::new(0);
        rebind_tuple(&mut invalid, |r| r.guardian = destination);
        auth_rejected(&mut invalid, Piv1Error::AccountAlias);
    }
}

#[test]
fn config_own_address_aliases_reject_but_external_beneficiary_guardian_overlap_is_allowed() {
    let mutations: &[fn(&mut PivConfig, Pubkey)] = &[
        |c, k| c.htfp_recipient = k,
        |c, k| c.team_owner_recipient = k,
        |c, k| c.guardian_registry = k,
        |c, k| c.principal_sol_queue = k,
        |c, k| c.stake_pool = k,
        |c, k| c.piv_authority = k,
    ];
    for edit in mutations {
        let mut f = Fixture::new(0); let address = f.accounts[CONFIG].key;
        f.update_config(|c| edit(c, address));
        auth_rejected(&mut f, Piv1Error::AccountAlias);
    }
    for team in [false, true] {
        let f = Fixture::new(0); let c = f.config(); let mut r = f.reward();
        r.guardian = if team { c.team_owner_recipient } else { c.htfp_recipient };
        let mut allowed = Fixture::from_earned(c, r, 0);
        success(&mut allowed, 300);
    }
}

#[test]
fn system_owned_pda_guardian_signer_is_supported_without_on_curve_assumption() {
    let f = Fixture::new(0); let c = f.config(); let mut r = f.reward();
    let wallet_program = key(222);
    let (wallet, _) = Pubkey::find_program_address(&[b"host-wallet"], &wallet_program);
    assert!(!wallet.is_on_curve());
    r.guardian = wallet;
    let mut supported = Fixture::from_earned(c, r, 0);
    success(&mut supported, 300);
    let mut invalid = supported.clone();
    invalid.accounts[GUARDIAN].owner = wallet_program;
    auth_rejected(&mut invalid, Piv1Error::InvalidAccountOwner);
}

#[test]
fn canonical_reward_discriminator_tuple_endianness_allocations_and_zero_padding_are_pinned() {
    assert_eq!(GUARDIAN_REWARD_SEED, b"guardian-reward");
    assert_eq!(GUARDIAN_REWARD_DISCRIMINATOR, [169, 109, 89, 17, 75, 171, 105, 39]);
    assert_eq!(GuardianReward::SPACE, 84);
    assert_eq!(GuardianRegistry::SPACE, 210);
    for activity in [None, Some(0), Some(u64::MAX)] {
        let mut f = Fixture::new(0); f.update_reward(|r| r.last_active_period = activity);
        let reward = *f.authenticate().unwrap().reward();
        assert_eq!(reward.last_active_period, activity);
        let used = 8 + reward.try_to_vec().unwrap().len();
        assert_eq!(GuardianReward::SPACE - used, if activity.is_none() { 8 } else { 0 });
        for index in used..GuardianReward::SPACE {
            let mut invalid = f.clone(); invalid.accounts[REWARD].data[index] = 1;
            auth_rejected(&mut invalid, Piv1Error::InvalidAccountData);
        }
        success(&mut f, 1);
    }
    let mut f = Fixture::new(0);
    rebind_tuple(&mut f, |r| r.registry_revision = 0x0102_0304_0506_0708);
    let r = f.reward();
    let (wrong, _) = Pubkey::find_program_address(&[b"guardian-reward", r.guardian.as_ref(),
        &r.registry_revision.to_be_bytes(), &[r.guardian_index]], &PROGRAM);
    assert_ne!(wrong, f.accounts[REWARD].key);
    f.accounts[REWARD].key = wrong;
    auth_rejected(&mut f, Piv1Error::InvalidAccountPda);
}

#[test]
fn malformed_envelopes_versions_initialization_and_config_fields_reject() {
    for index in [CONFIG, REWARD] {
        let mut f = Fixture::new(0); f.accounts[index].data[0] ^= 1;
        auth_rejected(&mut f, Piv1Error::InvalidAccountDiscriminator);
        for longer in [false, true] {
            let mut f = Fixture::new(0);
            if longer { f.accounts[index].data.push(0); } else { f.accounts[index].data.pop(); }
            auth_rejected(&mut f, Piv1Error::InvalidAccountSize);
        }
        let mut f = Fixture::new(0); f.accounts[index].data[8] = 0;
        auth_rejected(&mut f, Piv1Error::InvalidVersion);
        let mut f = Fixture::new(0); f.accounts[index].lamports -= 1;
        auth_rejected(&mut f, Piv1Error::AccountRentDeficit);
    }
    let mut malformed = Fixture::new(0);
    // Version/bump/slot + revision + guardian place Option activity at byte 51.
    malformed.accounts[REWARD].data[51] = 2;
    auth_rejected(&mut malformed, Piv1Error::InvalidAccountData);
    let mut f = Fixture::new(0); f.update_reward(|r| r.guardian_index = 6);
    auth_rejected(&mut f, Piv1Error::InvalidGuardianSet);
    let mut f = Fixture::new(0); f.update_reward(|r| r.guardian = Pubkey::default());
    auth_rejected(&mut f, Piv1Error::InvalidAddress);
    let mut f = Fixture::new(0); f.update_config(|c| c.is_initialized = false);
    auth_rejected(&mut f, Piv1Error::InvalidInitialization);
    let mut f = Fixture::new(0); f.update_config(|c| c.configured_slippage_bps = 2);
    auth_rejected(&mut f, Piv1Error::InvalidSlippage);
    let mut f = Fixture::new(0); f.update_config(|c| c.migration_reserve[0] = 1);
    auth_rejected(&mut f, Piv1Error::InvalidInitialization);
    let mut f = Fixture::new(0); f.update_config(|c| c.last_successful_preparation_at = None);
    let used = 8 + f.config().try_to_vec().unwrap().len();
    f.accounts[CONFIG].data[used] = 1;
    auth_rejected(&mut f, Piv1Error::InvalidAccountData);
}

#[test]
fn trusted_program_rent_and_borrow_boundaries_remain_explicit_and_fail_safely() {
    for id in [system_program::ID, spl_token::ID, STAKE_PROGRAM_ID] {
        let mut f = Fixture::new(0); f.program_id = id;
        auth_rejected(&mut f, Piv1Error::InvalidProgramIdentity);
    }
    let mut f = Fixture::new(0); f.program_id = key(222);
    auth_rejected(&mut f, Piv1Error::InvalidAccountOwner);
    for role in 0..3 {
        let mut f = Fixture::new(0);
        f.update_config(|c| match role { 0 => c.system_program = key(222),
            1 => c.token_program = key(222), _ => c.stake_program = key(222) });
        auth_rejected(&mut f, Piv1Error::InvalidProgramIdentity);
    }
    for rent in [Rent { exemption_threshold: f64::INFINITY, ..Rent::default() },
                 Rent { exemption_threshold: -1.0, ..Rent::default() },
                 Rent { burn_percent: 101, ..Rent::default() }] {
        let mut f = Fixture::new(0); f.rent = rent;
        auth_rejected(&mut f, Piv1Error::InvalidRent);
    }
    let mut f = Fixture::new(0); f.rent.lamports_per_byte_year = u64::MAX;
    auth_rejected(&mut f, Piv1Error::ArithmeticOverflow);
    for index in 0..4 {
        for data in [false, true] {
            let mut f = Fixture::new(0);
            let id = f.program_id; let rent = f.rent.clone(); let before = f.clone();
            f.with_infos(|infos| {
                let account = [infos.config, infos.guardian_reward, infos.kif_sol, infos.guardian][index];
                let result: Piv1Result<AuthenticatedKifClaimAccounts> = if data {
                    let _borrow = account.try_borrow_mut_data().unwrap();
                    authenticate_kif_claim_accounts(&id, &rent, infos)
                } else {
                    let _borrow = account.try_borrow_mut_lamports().unwrap();
                    authenticate_kif_claim_accounts(&id, &rent, infos)
                };
                assert_eq!(result, Err(Piv1Error::AccountBorrowFailed));
            });
            assert_eq!(f, before);
        }
    }
}

#[test]
fn old_and_numerically_newer_earned_revisions_claim_without_current_membership_or_activity() {
    let base = Fixture::new(0);
    for revision in [0, 7, 99, 100, u64::MAX] {
        for index in 0..6 {
            let mut r = base.reward(); r.registry_revision = revision; r.guardian_index = index;
            let mut f = Fixture::from_earned(base.config(), r, 0);
            f.update_config(|c| c.paused = true);
            success(&mut f, 300);
        }
    }
    // A replacement key and old key retain distinct tuple-addressed earned ledgers.
    let old = base.reward(); let mut replacement = old; replacement.guardian = key(92);
    let mut former = Fixture::from_earned(base.config(), old, 0);
    let mut current = Fixture::from_earned(base.config(), replacement, 0);
    assert_ne!(former.accounts[REWARD].key, current.accounts[REWARD].key);
    let original_current = current.clone();
    success(&mut former, 300);
    assert_eq!(current, original_current);
    success(&mut current, 300);
}

#[test]
fn independent_initial_audit_detects_payment_credit_or_custody_counter_tampering() {
    let mut f = Fixture::new(17); success(&mut f, 50);
    let edits: &[fn(&mut Fixture)] = &[
        |f| f.audit.paid += 1,
        |f| f.audit.credited += 1,
        |f| f.audit.external_excess += 1,
        |f| f.accounts[CONFIG].lamports += 1,
        |f| { f.accounts[KIF].lamports -= 1; f.accounts[GUARDIAN].lamports += 1; },
        |f| f.update_reward(|r| { r.cumulative_claimed += 1; r.claimable_lamports -= 1; }),
        |f| f.update_config(|c| { c.cumulative_kif_claimed_lamports += 1; c.kif_claim_liability_lamports -= 1; }),
    ];
    for edit in edits {
        let mut corrupted = f.clone(); edit(&mut corrupted);
        assert!(corrupted.validate_audit().is_err());
    }
}

#[test]
fn pure_checked_maximum_payment_and_overflow_edges_do_not_claim_host_funding() {
    let mut f = Fixture::new(0); let state = f.authenticate().unwrap();
    let mut c = state.config().clone(); let mut r = *state.reward();
    c.cumulative_kif_credited_lamports = u64::MAX;
    c.cumulative_kif_claimed_lamports = 0;
    c.kif_claim_liability_lamports = u64::MAX;
    c.collective_kif_carry_lamports = 0;
    r.cumulative_earned = u64::MAX; r.cumulative_claimed = 0; r.claimable_lamports = u64::MAX;
    let custody = KifClaimCustodyObservation { kif_sol: SolVaultBalance {
        lamports: u64::MAX, non_economic_floor_lamports: 0 }, guardian_lamports: 0 };
    let plan = prepare_kif_claim(&c, &r, KifClaimRequest {
        amount_lamports: u64::MAX, expected_cumulative_claimed: 0,
    }, custody).unwrap();
    let mut next_c = c.clone(); let mut next_r = r;
    plan.commit(&mut next_c, &mut next_r, KifClaimCustodyObservation {
        kif_sol: SolVaultBalance { lamports: 0, non_economic_floor_lamports: 0 },
        guardian_lamports: u64::MAX,
    }).unwrap();
    assert_eq!(next_c.cumulative_kif_claimed_lamports, u64::MAX);
    assert_eq!(next_r.cumulative_claimed, u64::MAX);
    assert_eq!(next_c.kif_claim_liability_lamports, 0);
    assert_eq!(next_r.claimable_lamports, 0);
    let mut overflow_c = c.clone(); overflow_c.collective_kif_carry_lamports = 1;
    assert_eq!(pure_rejected(&overflow_c, &r, custody, 1, 0), Piv1Error::ArithmeticOverflow);
    let mut floor_overflow = custody; floor_overflow.kif_sol.non_economic_floor_lamports = 1;
    assert_eq!(pure_rejected(&c, &r, floor_overflow, 1, 0), Piv1Error::ArithmeticOverflow);
    let mut destination_overflow = custody; destination_overflow.guardian_lamports = u64::MAX;
    assert_eq!(pure_rejected(&c, &r, destination_overflow, 1, 0), Piv1Error::ArithmeticOverflow);
    let mut identical_source = c.clone(); identical_source.kif_sol_vault = r.guardian;
    assert_eq!(pure_rejected(&identical_source, &r, custody, 1, 0), Piv1Error::AccountAlias);
}

#[test]
fn isolated_claims_import_genuine_active_settled_and_recovery_states_without_whole_world_health() {
    use support::vault_custody_model::{World, KIF as WORLD_KIF};
    for phase in [DistributionLifecycle::Idle, DistributionLifecycle::WithdrawalActive,
                  DistributionLifecycle::EscrowFunded, DistributionLifecycle::Settled,
                  DistributionLifecycle::RecoveryRequired] {
        let mut w = World::new(if phase == DistributionLifecycle::WithdrawalActive { 0 } else { 10_000 },
                              50, 0, 9, 3, FeeFraction::ZERO, 100_000);
        if phase != DistributionLifecycle::Idle { w.open(900_000).unwrap(); }
        if phase == DistributionLifecycle::Settled { w.settle().unwrap(); }
        if phase == DistributionLifecycle::RecoveryRequired {
            w.pool.decrease_exchange_rate(10_099_000).unwrap();
            assert_eq!(w.settle().unwrap(), SettlementOutcome::RecoveryRequired);
        }
        assert_eq!(w.round.lifecycle, phase);
        w.validate().unwrap();
        let economic_source = w.spendable(WORLD_KIF).unwrap();
        let whole_before = w.clone();
        // A new isolated funding baseline imports phase-compatible earned state;
        // this does not assert continuous custody across the two fixtures.
        let mut isolated = Fixture::from_earned(w.config.clone(), w.rewards[0], 0);
        assert_eq!(isolated.accounts[KIF].lamports - isolated.rent.minimum_balance(0), economic_source);
        isolated.update_config(|c| c.paused = true);
        let claimable = isolated.reward().claimable_lamports;
        success(&mut isolated, claimable);
        assert_eq!(w, whole_before);
        // Supplemental synthetic unrelated corruption: these inputs never enter
        // the dedicated fixture and cannot become a global-health claim gate.
        w.pool.advance_epoch_to(999).unwrap();
        w.sol[support::vault_custody_model::PRINCIPAL] = 0;
        assert!(w.validate().is_err());
        let mut isolated = Fixture::from_earned(w.config.clone(), w.rewards[0], 1);
        isolated.update_config(|c| c.paused = true);
        success(&mut isolated, 1);
    }
}

#[test]
fn genuine_empty_contribution_lifecycle_earns_then_claims_without_creating_reward_history() {
    use support::vault_custody_model::World;
    let mut w = World::empty(3, 42);
    w.explicit_tokens(1_000_000, 1_000_000).unwrap();
    w.bootstrap().unwrap();
    assert_eq!(w.rewards[0].cumulative_earned, 0);
    w.pool.increase_exchange_rate(100_000).unwrap();
    w.open(900_000).unwrap();
    let leg = w.initiate(1).unwrap();
    w.advance_epoch().unwrap(); w.finalize(leg).unwrap();
    assert_eq!(w.settle().unwrap(), SettlementOutcome::Settled);
    assert!(w.rewards[0].claimable_lamports > 0);
    w.validate().unwrap();
    let earned = w.rewards[0];
    let mut isolated = Fixture::from_earned(w.config.clone(), earned, 0);
    isolated.update_config(|c| { c.paused = true; c.guardian_registry_revision += 1; });
    success(&mut isolated, earned.claimable_lamports);
    assert_eq!(isolated.reward().cumulative_earned, earned.cumulative_earned);
    assert_eq!(isolated.reward().last_active_period, earned.last_active_period);
    assert_eq!(isolated.reward().registry_revision, earned.registry_revision);
}

#[test]
fn owned_authentication_snapshot_and_pure_plan_are_not_runtime_signer_capabilities() {
    let mut f = Fixture::new(0);
    let old = f.authenticate().unwrap();
    f.accounts[GUARDIAN].signer = false;
    // Owned point-in-time values cannot observe later account privilege changes.
    let plan = prepare_kif_claim(old.config(), old.reward(), KifClaimRequest {
        amount_lamports: 50, expected_cumulative_claimed: 10,
    }, old.custody()).unwrap();
    assert_eq!(plan.transfer().destination, old.reward().guardian);
    // The actual host composition reauthenticates current accounts before transfer.
    assert_eq!(rejected(&mut f, 50, 10), Error::State(Piv1Error::MissingGuardianSignature));
}

#[test]
fn cloned_owned_claim_values_survive_original_drop_and_preserve_serialized_results() {
    let mut f = Fixture::new(17);
    let original = f.authenticate().unwrap();
    let snapshot = original.clone();
    assert_eq!(snapshot, original);
    let original_bytes = original.config().try_to_vec().unwrap();
    drop(original);
    f.update_config(|c| c.paused = !c.paused);
    assert_eq!(snapshot.config().try_to_vec().unwrap(), original_bytes);
    assert_ne!(snapshot.config(), f.authenticate().unwrap().config());

    let mut config = snapshot.config().clone();
    let mut reward = *snapshot.reward();
    let custody = snapshot.custody();
    let plan = prepare_kif_claim(&config, &reward, KifClaimRequest {
        amount_lamports: 50, expected_cumulative_claimed: reward.cumulative_claimed,
    }, custody).unwrap();
    let cloned_plan = plan.clone();
    assert_eq!(cloned_plan, plan);
    drop(snapshot);
    let mut cloned_config = config.clone();
    let mut cloned_reward = reward;
    let mut expected_config = config.clone();
    expected_config.kif_claim_liability_lamports -= 50;
    expected_config.cumulative_kif_claimed_lamports += 50;
    let mut expected_reward = reward;
    expected_reward.claimable_lamports -= 50;
    expected_reward.cumulative_claimed += 50;
    // Supplied pure observations test value ownership, not an actual transfer.
    let after = KifClaimCustodyObservation {
        kif_sol: SolVaultBalance { lamports: custody.kif_sol.lamports - 50, ..custody.kif_sol },
        guardian_lamports: custody.guardian_lamports + 50,
    };
    let fixture_before = f.clone();
    let transfer = plan.commit(&mut config, &mut reward, after).unwrap();
    assert_eq!(cloned_plan.commit(&mut cloned_config, &mut cloned_reward, after), Ok(transfer));
    assert_eq!(config, expected_config);
    assert_eq!(reward, expected_reward);
    assert_eq!(cloned_config.try_to_vec().unwrap(), expected_config.try_to_vec().unwrap());
    assert_eq!(cloned_reward.try_to_vec().unwrap(), expected_reward.try_to_vec().unwrap());
    assert_eq!(f, fixture_before, "pure commits change no account bytes, lamports or audit baseline");
}
