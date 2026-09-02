use anchor_lang::{prelude::Pubkey, AnchorDeserialize, AnchorSerialize};

use piv1::{
    constants::{
        CONFIG_MIGRATION_RESERVE_BYTES, GUARDIAN_COUNT,
        INSUFFICIENT_RETRY_COOLDOWN_SECONDS, KIF_PERIOD_SECONDS,
        MINIMUM_DISTRIBUTION_INTERVAL_SECONDS, STATE_LAYOUT_VERSION,
    },
    errors::Piv1Error,
    instructions::ClaimKif,
    state::{
        finalize_withdrawal_leg, initiate_withdrawal_leg,
        integrate_pending_and_complete, open_distribution,
        record_no_yield_evaluation, record_valid_insufficient_attempt,
        settle_distribution, ActiveDistribution, DistributionFunding,
        DistributionLifecycle, GuardianRegistry, GuardianReward,
        LegFinalizationInput, LegFinalizationOutcome, LegInitiationInput,
        OpenDistributionInput, PendingIntegrationInput, PivConfig,
        PivConfigBumps, SettlementInput, SettlementOutcome,
        ValidInsufficientAttemptInput, WithdrawalLeg, WithdrawalLegStatus,
    },
};

const STATE_SEED: u64 = 0x5049_5631_5354_4154;
const SERIALIZATION_SEED: u64 = 0x5049_5631_5345_5244;
const ADVERSARIAL_SEED: u64 = 0x5049_5631_4144_5652;
const STATE_RANDOM_CASES: usize = 4_096;
const SERIALIZATION_CASES: usize = 1_024;
const OLD_HWM: u64 = 1_000_000;

#[derive(Clone, Copy)]
struct SplitMix64 {
    state: u64,
}

impl SplitMix64 {
    const fn new(seed: u64) -> Self {
        Self { state: seed }
    }

    fn next_u64(&mut self) -> u64 {
        self.state = self.state.wrapping_add(0x9e37_79b9_7f4a_7c15);
        let mut value = self.state;
        value = (value ^ (value >> 30)).wrapping_mul(0xbf58_476d_1ce4_e5b9);
        value = (value ^ (value >> 27)).wrapping_mul(0x94d0_49bb_1331_11eb);
        value ^ (value >> 31)
    }

    fn bounded(&mut self, exclusive_upper: u64) -> u64 {
        self.next_u64() % exclusive_upper
    }
}

#[derive(Clone)]
struct World {
    config: PivConfig,
    registry: GuardianRegistry,
    rewards: [GuardianReward; GUARDIAN_COUNT],
    round: ActiveDistribution,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct Snapshot {
    active_sequence: u64,
    last_completed_sequence: Option<u64>,
    prepared_at: i64,
    old_hwm: u64,
    historical_value: u64,
    gross_yield: u64,
    prior_yield: u64,
    htfp_gross: u64,
    compound: u64,
    team_gross: u64,
    kif_gross: u64,
    split_dust: u64,
    outgoing_gross: u64,
    pending_snapshot: u64,
    pending_used: u64,
    fixed_target: u64,
    floor: u64,
    maximum_legs: u64,
    minimum_native: u64,
    residual_floor: u64,
    htfp_recipient: Pubkey,
    team_recipient: Pubkey,
    registry: Pubkey,
    registry_revision: u64,
    guardian_keys: [Pubkey; GUARDIAN_COUNT],
    bitmap: u8,
    active_count: u8,
    period: u64,
    carry: u64,
    proposed_delta: u64,
    proposed_hwm: u64,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct ReferenceSplit {
    htfp: u64,
    compound: u64,
    team: u64,
    kif: u64,
    dust: u64,
    outgoing: u64,
}

fn key(tag: u8) -> Pubkey {
    Pubkey::new_from_array([tag; 32])
}

fn reference_split(gross_yield: u64) -> ReferenceSplit {
    let gross = u128::from(gross_yield);
    let htfp = (gross * 5_900 / 10_000) as u64;
    let compound = (gross * 1_950 / 10_000) as u64;
    let team = (gross * 1_950 / 10_000) as u64;
    let kif = (gross * 200 / 10_000) as u64;
    let allocated = htfp + compound + team + kif;
    ReferenceSplit {
        htfp,
        compound,
        team,
        kif,
        dust: gross_yield - allocated,
        outgoing: htfp + team + kif,
    }
}

fn valid_config(pending_sol: u64, hwm: u64) -> PivConfig {
    PivConfig {
        version: STATE_LAYOUT_VERSION,
        is_initialized: true,
        paused: false,
        bumps: PivConfigBumps {
            config: 1,
            piv_authority: 2,
            active_distribution: 3,
            principal_jito_vault: 4,
            pending_jito_vault: 5,
            pending_sol_vault: 6,
            principal_sol_queue: 7,
            operational_sol_vault: 8,
            distribution_escrow: 9,
            kif_sol_vault: 10,
            guardian_registry: 11,
        },
        stake_pool_program: key(1),
        stake_pool: key(2),
        validator_list: key(3),
        reserve_stake: key(4),
        jitosol_mint: key(5),
        token_program: key(6),
        stake_program: key(7),
        system_program: key(8),
        manager_fee_account: key(9),
        referrer_token_account: key(10),
        piv_authority: key(11),
        active_distribution: key(12),
        principal_jito_vault: key(13),
        pending_jito_vault: key(14),
        pending_sol_vault: key(15),
        principal_sol_queue: key(16),
        operational_sol_vault: key(17),
        distribution_escrow: key(18),
        kif_sol_vault: key(19),
        htfp_recipient: key(20),
        team_owner_recipient: key(21),
        guardian_registry: key(22),
        basis_points_denominator: 10_000,
        htfp_reserve_bps: 5_900,
        permanent_compound_bps: 1_950,
        team_owner_pool_bps: 1_950,
        kif_bps: 200,
        configured_slippage_bps: 1,
        slippage_hard_cap_bps: 1,
        minimum_distribution_interval_seconds: MINIMUM_DISTRIBUTION_INTERVAL_SECONDS,
        insufficient_retry_cooldown_seconds: INSUFFICIENT_RETRY_COOLDOWN_SECONDS,
        last_successful_preparation_at: None,
        last_valid_insufficient_attempt_at: None,
        next_distribution_sequence: 0,
        protected_principal_hwm_lamports: hwm,
        accounted_historical_jitosol_units: 10_000,
        accounted_historical_sol_lamports: 0,
        accounted_pending_jitosol_units: 0,
        accounted_pending_sol_lamports: pending_sol,
        next_cycle_yield_lamports: 0,
        kif_claim_liability_lamports: 0,
        collective_kif_carry_lamports: 0,
        cumulative_contribution_value_lamports: 0,
        cumulative_gross_yield_lamports: 0,
        cumulative_htfp_paid_lamports: 0,
        cumulative_team_owner_paid_lamports: 0,
        cumulative_kif_credited_lamports: 0,
        cumulative_kif_claimed_lamports: 0,
        cumulative_permanent_compound_lamports: 0,
        cumulative_retained_dust_lamports: 0,
        cumulative_zero_active_kif_compound_lamports: 0,
        cumulative_cooldown_yield_recorded_lamports: 0,
        kif_anchor_timestamp: 0,
        kif_period_seconds: KIF_PERIOD_SECONDS,
        guardian_registry_revision: 7,
        migration_reserve: [0; CONFIG_MIGRATION_RESERVE_BYTES],
    }
}

fn world(pending_sol: u64, active_bitmap: u8) -> World {
    let guardian_keys = core::array::from_fn(|index| key(100 + index as u8));
    let registry = GuardianRegistry::new(70, 7, guardian_keys)
        .expect("valid property registry");
    let mut rewards = core::array::from_fn(|index| {
        GuardianReward::new(index as u8, &registry, index as u8)
            .expect("valid property reward")
    });
    for (index, reward) in rewards.iter_mut().enumerate() {
        if active_bitmap & (1 << index) != 0 {
            reward
                .record_activity(&registry, index as u8, 0)
                .expect("record property activity");
        }
    }
    let config = valid_config(pending_sol, OLD_HWM);
    config.validate_initialized().expect("valid property config");
    World {
        config,
        registry,
        rewards,
        round: ActiveDistribution::new_idle(80),
    }
}

fn snapshot(round: &ActiveDistribution) -> Snapshot {
    Snapshot {
        active_sequence: round.active_sequence,
        last_completed_sequence: round.last_completed.map(|value| value.sequence),
        prepared_at: round.prepared_at,
        old_hwm: round.old_protected_principal_lamports,
        historical_value: round.historical_value_lamports,
        gross_yield: round.gross_yield_lamports,
        prior_yield: round.prior_next_cycle_yield_lamports,
        htfp_gross: round.htfp_gross_obligation_lamports,
        compound: round.permanent_compound_lamports,
        team_gross: round.team_owner_gross_obligation_lamports,
        kif_gross: round.kif_gross_obligation_lamports,
        split_dust: round.split_dust_lamports,
        outgoing_gross: round.outgoing_gross_obligation_lamports,
        pending_snapshot: round.pending_sol_snapshot_lamports,
        pending_used: round.pending_sol_used_lamports,
        fixed_target: round.fixed_jitosol_withdrawal_target_units,
        floor: round.snapshot_leg_input_floor_units,
        maximum_legs: round.maximum_useful_legs,
        minimum_native: round.stored_round_minimum_native_lamports,
        residual_floor: round.stored_residual_hwm_floor_lamports,
        htfp_recipient: round.htfp_recipient,
        team_recipient: round.team_owner_recipient,
        registry: round.guardian_registry,
        registry_revision: round.guardian_registry_revision,
        guardian_keys: round.guardian_keys,
        bitmap: round.kif_eligibility_bitmap,
        active_count: round.kif_active_guardian_count,
        period: round.kif_period_id,
        carry: round.kif_carry_input_lamports,
        proposed_delta: round.proposed_hwm_delta_lamports,
        proposed_hwm: round.proposed_hwm_after_settlement_lamports,
    }
}

fn open_input(
    config: &PivConfig,
    prepared_at: i64,
    gross_yield: u64,
    funding: DistributionFunding,
) -> OpenDistributionInput {
    let split = reference_split(gross_yield);
    let historical_value = config
        .protected_principal_hwm_lamports
        .checked_add(gross_yield)
        .and_then(|value| value.checked_sub(config.next_cycle_yield_lamports))
        .expect("bounded property gross basis");
    let pending_used = config
        .accounted_pending_sol_lamports
        .min(split.outgoing);
    let proposed_delta = split.compound + split.dust;
    OpenDistributionInput {
        sequence: config.next_distribution_sequence,
        prepared_at,
        prepared_slot: prepared_at as u64 + 100,
        prepared_epoch: prepared_at as u64 / 400_000,
        historical_jitosol_units: config.accounted_historical_jitosol_units,
        historical_sol_lamports: config.accounted_historical_sol_lamports,
        historical_value_lamports: historical_value,
        snapshot_pool_total_lamports: 1_000_000_000,
        snapshot_pool_token_supply: 900_000_000,
        snapshot_withdrawal_fee_numerator: 1,
        snapshot_withdrawal_fee_denominator: 1_000,
        gross_yield_lamports: gross_yield,
        pending_sol_snapshot_lamports: config.accounted_pending_sol_lamports,
        pending_sol_used_lamports: pending_used,
        snapshot_conversion_dust_lamports: 0,
        stored_residual_hwm_floor_lamports: config
            .protected_principal_hwm_lamports
            .checked_add(proposed_delta)
            .expect("bounded proposed HWM"),
        funding,
    }
}

fn liquid_input(config: &PivConfig, prepared_at: i64, gross_yield: u64) -> OpenDistributionInput {
    let split = reference_split(gross_yield);
    let pending_used = config
        .accounted_pending_sol_lamports
        .min(split.outgoing);
    let remaining = split.outgoing - pending_used;
    let prior_used = config.next_cycle_yield_lamports.min(remaining);
    open_input(
        config,
        prepared_at,
        gross_yield,
        DistributionFunding::Liquid {
            escrow_available_lamports: pending_used + prior_used,
        },
    )
}

fn withdrawal_input(
    config: &PivConfig,
    prepared_at: i64,
    gross_yield: u64,
    target: u64,
    floor: u64,
) -> OpenDistributionInput {
    let split = reference_split(gross_yield);
    let pending_used = config
        .accounted_pending_sol_lamports
        .min(split.outgoing);
    let prior_used = config
        .next_cycle_yield_lamports
        .min(split.outgoing - pending_used);
    open_input(
        config,
        prepared_at,
        gross_yield,
        DistributionFunding::Withdrawal {
            fixed_jitosol_target_units: target,
            snapshot_leg_input_floor_units: floor,
            maximum_useful_legs: target / floor,
            stored_round_minimum_native_lamports: target,
            initial_escrow_available_lamports: pending_used + prior_used,
        },
    )
}

fn initiation_input(
    round: &ActiveDistribution,
    capacity: u64,
    fee: u64,
) -> LegInitiationInput {
    let input = round
        .remaining_withdrawal_target_units()
        .expect("valid remaining target")
        .min(capacity);
    LegInitiationInput {
        sequence: round.active_sequence,
        leg_index: round.next_leg_index,
        validator_list_index: round.next_leg_index as u32,
        validator_seed_suffix: (round.next_leg_index as u32) + 10,
        validator_vote: key(150 + (round.next_leg_index as u8 * 2)),
        validator_stake_source: key(151 + (round.next_leg_index as u8 * 2)),
        initiation_epoch: 20,
        pool_total_lamports: 1_000_000,
        pool_token_supply: 900_000,
        withdrawal_fee_numerator: 1,
        withdrawal_fee_denominator: 1_000,
        current_technical_floor_units: round.snapshot_leg_input_floor_units,
        maximum_safe_capacity_units: capacity,
        jitosol_input_units: input,
        withdrawal_fee_units: fee,
        burned_units: input - fee,
        expected_native_lamports: input,
        observed_delegated_native_lamports: input,
        minimum_native_lamports: input,
        stake_rent_advanced_lamports: 10 + round.next_leg_index,
        metadata_rent_advanced_lamports: 5 + round.next_leg_index,
    }
}

fn finalization_input(
    round: &ActiveDistribution,
    leg: &WithdrawalLeg,
    reward: u64,
    loss: u64,
    residual_value: u64,
) -> LegFinalizationInput {
    let finalized_native = leg.observed_delegated_native_lamports
        + leg.stake_rent_advanced_lamports
        + reward
        - loss;
    let cumulative_native = round.cumulative_finalized_native_lamports
        + finalized_native;
    let cumulative_rent = round.cumulative_recovered_stake_rent_lamports
        + leg.stake_rent_advanced_lamports;
    let initial_liquid = round.pending_sol_used_lamports
        + round
            .prior_next_cycle_yield_used_lamports()
            .expect("valid prior-yield funding");
    LegFinalizationInput {
        sequence: round.active_sequence,
        leg_index: leg.leg_index,
        finalized_epoch: leg.initiation_epoch + 1,
        finalized_native_lamports: finalized_native,
        recovered_stake_rent_lamports: leg.stake_rent_advanced_lamports,
        recovered_metadata_rent_lamports: leg.metadata_rent_advanced_lamports,
        cooldown_reward_lamports: reward,
        cooldown_loss_lamports: loss,
        validated_residual_historical_value_lamports: residual_value,
        escrow_available_after_lamports: initial_liquid + cumulative_native - cumulative_rent,
    }
}

fn settle_input(round: &ActiveDistribution, protected_value: u64) -> SettlementInput {
    SettlementInput {
        sequence: round.active_sequence,
        escrow_available_lamports: round.recorded_escrow_available_lamports,
        validated_post_settlement_protected_value_lamports: protected_value,
    }
}

fn assert_valid_world(world: &World) {
    world.config.validate_initialized().expect("valid config state");
    world.registry.validate().expect("valid registry state");
    world.round.validate().expect("valid distribution state");
    for (index, reward) in world.rewards.iter().enumerate() {
        reward.validate().expect("valid reward state");
        if reward.registry_revision == world.registry.revision
            && reward.guardian == world.registry.guardian_keys[index]
        {
            world
                .registry
                .validate_reward_binding(index as u8, reward)
                .expect("valid live reward binding");
        }
    }
}

#[test]
fn prior_cycle_yield_repairs_loss_before_yield_and_is_consumed_once() {
    for (carry, expected) in [
        (99, Ok(())),
        (100, Ok(())),
        (101, Err(Piv1Error::CumulativeReconciliationMismatch)),
    ] {
        let mut boundary = world(0, 0);
        boundary.config.protected_principal_hwm_lamports = 1_000;
        boundary.config.next_cycle_yield_lamports = carry;
        assert_eq!(
            record_no_yield_evaluation(&boundary.config, &boundary.round, 0, 900),
            expected,
            "carry/HWM deficit boundary carry={carry}"
        );
    }

    let mut rng = SplitMix64::new(STATE_SEED);
    for case in 0..STATE_RANDOM_CASES {
        let hwm = 1_000_000 + rng.bounded(1_000_000_000);
        let deficit = rng.bounded(hwm.min(100_000) + 1);
        let excess = 2 + rng.bounded(100_000);
        let carry = deficit + excess;
        let historical_value = hwm - deficit;
        let gross_yield = excess;
        let split = reference_split(gross_yield);
        if split.outgoing == 0 {
            continue;
        }

        let mut world = world(split.outgoing, (rng.next_u64() & 0x3f) as u8);
        world.config.protected_principal_hwm_lamports = hwm;
        world.config.next_cycle_yield_lamports = carry;
        world.config.accounted_pending_sol_lamports = split.outgoing;

        assert_eq!(
            record_no_yield_evaluation(
                &world.config,
                &world.round,
                0,
                hwm.saturating_sub(carry),
            ),
            Ok(()),
            "carry-at-deficit seed={STATE_SEED:#018x}, case={case}"
        );

        let proposed_delta = split.compound + split.dust;
        let input = OpenDistributionInput {
            sequence: 0,
            prepared_at: 0,
            prepared_slot: 0,
            prepared_epoch: 0,
            historical_jitosol_units: world.config.accounted_historical_jitosol_units,
            historical_sol_lamports: world.config.accounted_historical_sol_lamports,
            historical_value_lamports: historical_value,
            snapshot_pool_total_lamports: 1_000_000,
            snapshot_pool_token_supply: 900_000,
            snapshot_withdrawal_fee_numerator: 1,
            snapshot_withdrawal_fee_denominator: 1_000,
            gross_yield_lamports: gross_yield,
            pending_sol_snapshot_lamports: split.outgoing,
            pending_sol_used_lamports: split.outgoing,
            snapshot_conversion_dust_lamports: 0,
            stored_residual_hwm_floor_lamports: hwm + proposed_delta,
            funding: DistributionFunding::Liquid {
                escrow_available_lamports: split.outgoing,
            },
        };
        open_distribution(
            &mut world.config,
            &mut world.round,
            &world.registry,
            &world.rewards,
            input,
        )
        .unwrap_or_else(|error| {
            panic!(
                "carry open seed={STATE_SEED:#018x}, case={case}, error={error:?}"
            )
        });

        assert_eq!(world.round.gross_yield_lamports, excess);
        assert_eq!(world.round.prior_next_cycle_yield_lamports, carry);
        assert_eq!(world.config.next_cycle_yield_lamports, 0);
        assert!(world.round.proposed_hwm_after_settlement_lamports >= hwm);
        assert_valid_world(&world);

        let config_before = world.config.clone();
        let round_before = world.round;
        let input = liquid_input(&world.config, 1, gross_yield);
        assert!(open_distribution(
            &mut world.config,
            &mut world.round,
            &world.registry,
            &world.rewards,
            input,
        )
        .is_err());
        assert_eq!(world.config, config_before);
        assert_eq!(world.round, round_before);
    }

    let mut overflow = world(100, 0);
    overflow.config.next_cycle_yield_lamports = 1;
    let config_before = overflow.config.clone();
    assert_eq!(
        record_no_yield_evaluation(
            &overflow.config,
            &overflow.round,
            0,
            u64::MAX,
        ),
        Err(Piv1Error::ArithmeticOverflow)
    );
    assert_eq!(overflow.config, config_before);
}

#[test]
fn state_level_kif_credits_only_snapshotted_active_guardians_and_conserves_value() {
    let mut rng = SplitMix64::new(STATE_SEED ^ 0x4b49_4600_0000_0000);
    for case in 0..STATE_RANDOM_CASES {
        let gross_yield = 100 + rng.bounded(1_000_000);
        let split = reference_split(gross_yield);
        let bitmap = (rng.next_u64() & 0x3f) as u8;
        let carry = rng.bounded(10_000);
        let mut world = world(split.outgoing, bitmap);
        world.config.collective_kif_carry_lamports = carry;
        let input = liquid_input(&world.config, 0, gross_yield);
        open_distribution(
            &mut world.config,
            &mut world.round,
            &world.registry,
            &world.rewards,
            input,
        )
        .unwrap_or_else(|error| {
            panic!(
                "KIF open seed={STATE_SEED:#018x}, case={case}, error={error:?}"
            )
        });
        let immutable = snapshot(&world.round);
        let old_hwm = world.config.protected_principal_hwm_lamports;

        let settlement = settle_input(&world.round, u64::MAX);
        assert_eq!(
            settle_distribution(
                &mut world.config,
                &mut world.round,
                &mut world.rewards,
                settlement,
            ),
            Ok(SettlementOutcome::Settled),
            "KIF settlement seed={STATE_SEED:#018x}, case={case}"
        );
        assert_eq!(snapshot(&world.round), immutable);

        let expected_net_kif = core::cmp::min(
            split.kif,
            (u128::from(split.outgoing) * 200 / 8_050) as u64,
        );
        assert_eq!(
            world.round.actual_kif_allocation_lamports,
            expected_net_kif,
            "net KIF oracle seed={STATE_SEED:#018x}, case={case}"
        );
        let available = expected_net_kif + carry;
        let active_count = bitmap.count_ones() as u8;
        if active_count == 0 {
            let compounded = available / 2;
            let carry_next = available - compounded;
            assert_eq!(world.round.actual_kif_liability_lamports, 0);
            assert_eq!(
                world.round.actual_zero_active_kif_compound_lamports,
                compounded
            );
            assert_eq!(world.round.actual_kif_carry_next_lamports, carry_next);
            assert!(world
                .rewards
                .iter()
                .all(|reward| reward.claimable_lamports == 0));
        } else {
            let per_guardian = available / u64::from(active_count);
            let credited = per_guardian * u64::from(active_count);
            let carry_next = available - credited;
            for (index, reward) in world.rewards.iter().enumerate() {
                let expected = if bitmap & (1 << index) != 0 {
                    per_guardian
                } else {
                    0
                };
                assert_eq!(
                    reward.claimable_lamports, expected,
                    "guardian credit seed={STATE_SEED:#018x}, case={case}, index={index}"
                );
            }
            assert_eq!(world.round.actual_kif_liability_lamports, credited);
            assert_eq!(world.round.actual_kif_carry_next_lamports, carry_next);
            assert_eq!(world.round.actual_zero_active_kif_compound_lamports, 0);
        }
        assert_eq!(
            world.round.actual_kif_liability_lamports
                + world.round.actual_kif_carry_next_lamports
                + world.round.actual_zero_active_kif_compound_lamports,
            available
        );
        assert!(world.config.protected_principal_hwm_lamports >= old_hwm);
        assert_valid_world(&world);
    }
}

#[test]
fn accepted_partial_settlement_is_deterministic_and_protects_dust_once() {
    let gross_yield = 10_000;
    let mut world = world(1_000, 0b00_0001);
    let input = withdrawal_input(&world.config, 0, gross_yield, 200, 100);
    open_distribution(
        &mut world.config,
        &mut world.round,
        &world.registry,
        &world.rewards,
        input,
    )
    .expect("open accepted partial-settlement fixture");
    let immutable = snapshot(&world.round);

    let mut leg = WithdrawalLeg::vacant(31, 41);
    let input = initiation_input(&world.round, 200, 1);
    let mut input = input;
    input.expected_native_lamports = 800;
    input.observed_delegated_native_lamports = 800;
    input.minimum_native_lamports = 800;
    initiate_withdrawal_leg(&world.config, &mut world.round, &mut leg, input)
        .expect("initiate accepted partial-settlement leg");
    let finalization = finalization_input(
        &world.round,
        &leg,
        0,
        0,
        world.round.stored_residual_hwm_floor_lamports,
    );
    assert_eq!(
        finalize_withdrawal_leg(
            &world.config,
            &mut world.round,
            &mut leg,
            finalization,
        ),
        Ok(LegFinalizationOutcome::EscrowFunded)
    );
    assert_eq!(world.round.recorded_escrow_available_lamports, 1_800);

    let old_hwm = world.config.protected_principal_hwm_lamports;
    let settlement = settle_input(&world.round, u64::MAX);
    assert_eq!(
        settle_distribution(
            &mut world.config,
            &mut world.round,
            &mut world.rewards,
            settlement,
        ),
        Ok(SettlementOutcome::Settled)
    );
    assert_eq!(world.round.actual_htfp_lamports, 1_319);
    assert_eq!(world.round.actual_team_owner_lamports, 436);
    assert_eq!(world.round.actual_kif_allocation_lamports, 44);
    assert_eq!(world.round.actual_net_allocation_dust_lamports, 1);
    assert_eq!(world.round.actual_allocated_outgoing_lamports, 1_799);
    assert_eq!(world.round.actual_hwm_delta_lamports, 1_951);
    assert_eq!(world.config.protected_principal_hwm_lamports, old_hwm + 1_951);
    assert_eq!(world.config.cumulative_retained_dust_lamports, 1);
    assert_eq!(snapshot(&world.round), immutable);
}

#[test]
fn randomized_valid_state_round_trips_preserve_layouts_and_all_fields() {
    assert_eq!(PivConfig::SERIALIZED_SIZE, 1_006);
    assert_eq!(PivConfig::SPACE, 1_014);
    assert_eq!(ActiveDistribution::SERIALIZED_SIZE, 883);
    assert_eq!(ActiveDistribution::SPACE, 891);
    assert_eq!(WithdrawalLeg::SERIALIZED_SIZE, 255);
    assert_eq!(WithdrawalLeg::SPACE, 263);
    assert_eq!(GuardianRegistry::SERIALIZED_SIZE, 202);
    assert_eq!(GuardianRegistry::SPACE, 210);
    assert_eq!(GuardianReward::SERIALIZED_SIZE, 76);
    assert_eq!(GuardianReward::SPACE, 84);

    let mut rng = SplitMix64::new(SERIALIZATION_SEED);
    for case in 0..SERIALIZATION_CASES {
        let context = format!("seed={SERIALIZATION_SEED:#018x}, case={case}");
        let mut config = valid_config(rng.next_u64(), rng.next_u64());
        config.paused = case % 2 == 0;
        config.last_successful_preparation_at = (case % 3 != 0).then(|| rng.next_u64() as i64);
        config.last_valid_insufficient_attempt_at =
            (case % 5 != 0).then(|| rng.next_u64() as i64);
        config.next_distribution_sequence = rng.next_u64();
        config.accounted_historical_jitosol_units = rng.next_u64();
        config.accounted_historical_sol_lamports = rng.next_u64();
        config.accounted_pending_jitosol_units = rng.next_u64();
        config.next_cycle_yield_lamports = rng.next_u64();
        config.collective_kif_carry_lamports = rng.next_u64();
        config.cumulative_contribution_value_lamports = rng.next_u64();
        config.cumulative_gross_yield_lamports = rng.next_u64();
        config.cumulative_htfp_paid_lamports = rng.next_u64();
        config.cumulative_team_owner_paid_lamports = rng.next_u64();
        let earned = rng.next_u64();
        let claimed = rng.bounded(earned.saturating_add(1));
        config.cumulative_kif_credited_lamports = earned;
        config.cumulative_kif_claimed_lamports = claimed;
        config.kif_claim_liability_lamports = earned - claimed;
        config.cumulative_permanent_compound_lamports = rng.next_u64();
        config.cumulative_retained_dust_lamports = rng.next_u64();
        config.cumulative_zero_active_kif_compound_lamports = rng.next_u64();
        config.cumulative_cooldown_yield_recorded_lamports = rng.next_u64();
        config.kif_anchor_timestamp = rng.next_u64() as i64;
        config.guardian_registry_revision = rng.next_u64();
        config
            .validate_initialized()
            .unwrap_or_else(|error| panic!("config seed={SERIALIZATION_SEED:#018x}, case={case}, error={error:?}"));
        let encoded = config.try_to_vec().unwrap_or_else(|error| {
            panic!("serialize randomized config: {context}, error={error:?}")
        });
        assert!(encoded.len() <= PivConfig::SERIALIZED_SIZE, "{context}");
        let decoded = PivConfig::try_from_slice(&encoded).unwrap_or_else(|error| {
            panic!("deserialize randomized config: {context}, error={error:?}")
        });
        assert_eq!(decoded, config, "{context}");

        let bitmap = (rng.next_u64() & 0x3f) as u8;
        let mut world = world(10_000, bitmap);
        let gross = 100 + rng.bounded(1_000_000);
        let split = reference_split(gross);
        world.config.accounted_pending_sol_lamports = split.outgoing;
        let input = liquid_input(&world.config, 0, gross);
        open_distribution(
            &mut world.config,
            &mut world.round,
            &world.registry,
            &world.rewards,
            input,
        )
        .unwrap_or_else(|error| {
            panic!("open randomized serialized round: {context}, error={error:?}")
        });

        let registry_bytes = world.registry.try_to_vec().unwrap_or_else(|error| {
            panic!("serialize registry: {context}, error={error:?}")
        });
        assert_eq!(registry_bytes.len(), GuardianRegistry::SERIALIZED_SIZE, "{context}");
        assert_eq!(
            GuardianRegistry::try_from_slice(&registry_bytes)
                .unwrap_or_else(|error| panic!("deserialize registry: {context}, error={error:?}")),
            world.registry,
            "{context}"
        );
        for (index, reward) in world.rewards.into_iter().enumerate() {
            let bytes = reward.try_to_vec().unwrap_or_else(|error| {
                panic!("serialize reward: {context}, index={index}, error={error:?}")
            });
            assert!(bytes.len() <= GuardianReward::SERIALIZED_SIZE, "{context}, index={index}");
            assert_eq!(
                GuardianReward::try_from_slice(&bytes).unwrap_or_else(|error| {
                    panic!("deserialize reward: {context}, index={index}, error={error:?}")
                }),
                reward,
                "{context}, index={index}"
            );
        }

        let round_bytes = world.round.try_to_vec().unwrap_or_else(|error| {
            panic!("serialize active round: {context}, error={error:?}")
        });
        assert!(round_bytes.len() <= ActiveDistribution::SERIALIZED_SIZE, "{context}");
        let decoded_round = ActiveDistribution::try_from_slice(&round_bytes)
            .unwrap_or_else(|error| panic!("deserialize active round: {context}, error={error:?}"));
        assert_eq!(decoded_round, world.round, "{context}");
        decoded_round.validate().unwrap_or_else(|error| {
            panic!("validate decoded active round: {context}, error={error:?}")
        });

        if case % 2 == 0 {
            let settlement = settle_input(&world.round, u64::MAX);
            settle_distribution(
                &mut world.config,
                &mut world.round,
                &mut world.rewards,
                settlement,
            )
            .unwrap_or_else(|error| {
                panic!("settle randomized serialized round: {context}, error={error:?}")
            });
            let settled_bytes = world.round.try_to_vec().unwrap_or_else(|error| {
                panic!("serialize settled round: {context}, error={error:?}")
            });
            assert_eq!(
                ActiveDistribution::try_from_slice(&settled_bytes)
                    .unwrap_or_else(|error| panic!("deserialize settled round: {context}, error={error:?}")),
                world.round,
                "{context}"
            );
        } else {
            let mut withdrawal_world = crate::world(1, bitmap);
            let input = withdrawal_input(&withdrawal_world.config, 0, 10_000, 200, 100);
            open_distribution(
                &mut withdrawal_world.config,
                &mut withdrawal_world.round,
                &withdrawal_world.registry,
                &withdrawal_world.rewards,
                input,
            )
            .unwrap_or_else(|error| {
                panic!("open serialized withdrawal round: {context}, error={error:?}")
            });
            let mut leg = WithdrawalLeg::vacant((case & 0xff) as u8, 200);
            let initiation = initiation_input(&withdrawal_world.round, 200, case as u64 % 2);
            initiate_withdrawal_leg(
                &withdrawal_world.config,
                &mut withdrawal_world.round,
                &mut leg,
                initiation,
            )
            .unwrap_or_else(|error| {
                panic!("initiate serialized leg: {context}, error={error:?}")
            });
            let leg_bytes = leg.try_to_vec().unwrap_or_else(|error| {
                panic!("serialize initiated leg: {context}, error={error:?}")
            });
            assert!(leg_bytes.len() <= WithdrawalLeg::SERIALIZED_SIZE, "{context}");
            let decoded_leg = WithdrawalLeg::try_from_slice(&leg_bytes)
                .unwrap_or_else(|error| panic!("deserialize initiated leg: {context}, error={error:?}"));
            assert_eq!(decoded_leg, leg, "{context}");
            decoded_leg.validate().unwrap_or_else(|error| {
                panic!("validate decoded leg: {context}, error={error:?}")
            });
        }
    }
}

fn assert_decoded_config_rejected(config: &PivConfig, context: &str) {
    let bytes = config.try_to_vec().expect("serialize malformed config");
    let decoded = PivConfig::try_from_slice(&bytes).expect("decode malformed config");
    assert!(decoded.validate_initialized().is_err(), "{context}");
}

fn assert_decoded_round_rejected(round: &ActiveDistribution, context: &str) {
    let bytes = round.try_to_vec().expect("serialize malformed round");
    let decoded = ActiveDistribution::try_from_slice(&bytes).expect("decode malformed round");
    assert!(decoded.validate().is_err(), "{context}");
}

fn assert_decoded_leg_rejected(leg: &WithdrawalLeg, context: &str) {
    let bytes = leg.try_to_vec().expect("serialize malformed leg");
    let decoded = WithdrawalLeg::try_from_slice(&bytes).expect("decode malformed leg");
    assert!(decoded.validate().is_err(), "{context}");
}

#[test]
fn randomized_single_field_mutations_of_decoded_state_are_rejected() {
    let mut base_world = world(8_050, 0b00_1011);
    let open = liquid_input(&base_world.config, 0, 10_000);
    open_distribution(
        &mut base_world.config,
        &mut base_world.round,
        &base_world.registry,
        &base_world.rewards,
        open,
    )
    .expect("open mutation fixture");

    let mut withdrawal_world = world(1, 0b00_1011);
    let open = withdrawal_input(&withdrawal_world.config, 0, 10_000, 200, 100);
    open_distribution(
        &mut withdrawal_world.config,
        &mut withdrawal_world.round,
        &withdrawal_world.registry,
        &withdrawal_world.rewards,
        open,
    )
    .expect("open leg mutation fixture");
    let mut base_leg = WithdrawalLeg::vacant(31, 41);
    let initiation = initiation_input(&withdrawal_world.round, 200, 1);
    initiate_withdrawal_leg(
        &withdrawal_world.config,
        &mut withdrawal_world.round,
        &mut base_leg,
        initiation,
    )
    .expect("initiate leg mutation fixture");

    let mut rng = SplitMix64::new(ADVERSARIAL_SEED);
    for case in 0..STATE_RANDOM_CASES {
        let context = format!("seed={ADVERSARIAL_SEED:#018x}, case={case}");

        let mut config = valid_config(10, OLD_HWM);
        match rng.bounded(11) {
            0 => config.version ^= 0xff,
            1 => config.is_initialized = false,
            2 => config.pending_jito_vault = config.principal_jito_vault,
            3 => config.kif_bps ^= 1,
            4 => config.configured_slippage_bps = 2,
            5 => config.slippage_hard_cap_bps = 0,
            6 => config.minimum_distribution_interval_seconds += 1,
            7 => config.kif_period_seconds += 1,
            8 => config.migration_reserve[case % CONFIG_MIGRATION_RESERVE_BYTES] = 1,
            9 => config.kif_claim_liability_lamports = 1,
            _ => config.htfp_recipient = Pubkey::default(),
        }
        assert_decoded_config_rejected(&config, &context);

        let mut round = base_world.round;
        match rng.bounded(17) {
            0 => round.version ^= 0xff,
            1 => round.is_initialized = false,
            2 => round.lifecycle = DistributionLifecycle::Idle,
            3 => round.recovery_flags = 0xff,
            4 => round.htfp_recipient = Pubkey::default(),
            5 => round.team_owner_recipient = round.htfp_recipient,
            6 => round.snapshot_pool_total_lamports = 0,
            7 => round.guardian_keys[5] = round.guardian_keys[0],
            8 => round.kif_eligibility_bitmap ^= 1 << 5,
            9 => round.gross_yield_lamports += 1,
            10 => round.htfp_gross_obligation_lamports += 1,
            11 => round.outgoing_gross_obligation_lamports += 1,
            12 => round.pending_sol_used_lamports -= 1,
            13 => round.proposed_hwm_delta_lamports += 1,
            14 => round.stored_residual_hwm_floor_lamports = 0,
            15 => round.actual_net_available_lamports = 1,
            _ => round.recorded_escrow_available_lamports += 1,
        }
        assert_decoded_round_rejected(&round, &context);

        let mut leg = base_leg;
        match rng.bounded(14) {
            0 => leg.version ^= 0xff,
            1 => leg.is_initialized = false,
            2 => leg.status = WithdrawalLegStatus::Vacant,
            3 => leg.validator_vote = Pubkey::default(),
            4 => leg.validator_stake_source = leg.validator_vote,
            5 => leg.pool_total_lamports = 0,
            6 => leg.jitosol_input_units = 0,
            7 => leg.technical_floor_units = leg.jitosol_input_units + 1,
            8 => leg.withdrawal_fee_units += 1,
            9 => leg.minimum_native_lamports = leg.expected_native_lamports + 1,
            10 => leg.observed_delegated_native_lamports = leg.minimum_native_lamports - 1,
            11 => leg.finalized_epoch = Some(leg.initiation_epoch),
            12 => leg.finalized_native_lamports = 1,
            _ => leg.recovery_flags = 1,
        }
        assert_decoded_leg_rejected(&leg, &context);

        let mut registry = base_world.registry;
        if case % 2 == 0 {
            registry.guardian_keys[5] = registry.guardian_keys[0];
        } else {
            registry.version ^= 0xff;
        }
        let bytes = registry.try_to_vec().expect("serialize malformed registry");
        let decoded = GuardianRegistry::try_from_slice(&bytes)
            .expect("decode malformed registry");
        assert!(decoded.validate().is_err(), "{context}");

        let mut reward = base_world.rewards[0];
        if case % 2 == 0 {
            reward.claimable_lamports = 1;
        } else {
            reward.guardian_index = GUARDIAN_COUNT as u8;
        }
        let bytes = reward.try_to_vec().expect("serialize malformed reward");
        let decoded = GuardianReward::try_from_slice(&bytes)
            .expect("decode malformed reward");
        assert!(decoded.validate().is_err(), "{context}");
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum ModelPhase {
    Idle,
    Withdrawal,
    Escrow,
    Settled,
    Recovery,
}

struct ReferenceModel {
    phase: ModelPhase,
    next_sequence: u64,
    active_sequence: Option<u64>,
    fixed_target: u64,
    assigned: u64,
    successful: u64,
    finalized: u64,
    leg_status: [u8; 4],
    fixed_snapshot: Option<Snapshot>,
    last_hwm: u64,
}

impl ReferenceModel {
    fn new(hwm: u64) -> Self {
        Self {
            phase: ModelPhase::Idle,
            next_sequence: 0,
            active_sequence: None,
            fixed_target: 0,
            assigned: 0,
            successful: 0,
            finalized: 0,
            leg_status: [0; 4],
            fixed_snapshot: None,
            last_hwm: hwm,
        }
    }

    fn assert_matches(&self, world: &World, legs: &[WithdrawalLeg; 4], context: &str) {
        let expected_lifecycle = match self.phase {
            ModelPhase::Idle => DistributionLifecycle::Idle,
            ModelPhase::Withdrawal => DistributionLifecycle::WithdrawalActive,
            ModelPhase::Escrow => DistributionLifecycle::EscrowFunded,
            ModelPhase::Settled => DistributionLifecycle::Settled,
            ModelPhase::Recovery => DistributionLifecycle::RecoveryRequired,
        };
        assert_eq!(world.round.lifecycle, expected_lifecycle, "{context}");
        assert_eq!(
            world.config.next_distribution_sequence,
            self.next_sequence,
            "sequence model: {context}"
        );
        assert!(
            world.config.protected_principal_hwm_lamports >= self.last_hwm,
            "HWM decreased: {context}"
        );
        if let Some(sequence) = self.active_sequence {
            assert_eq!(world.round.active_sequence, sequence, "{context}");
            assert_eq!(world.round.fixed_jitosol_withdrawal_target_units, self.fixed_target, "{context}");
            assert_eq!(world.round.cumulative_jitosol_assigned_units, self.assigned, "{context}");
            assert_eq!(world.round.successful_leg_count, self.successful, "{context}");
            assert_eq!(world.round.finalized_leg_count, self.finalized, "{context}");
            assert!(self.assigned <= self.fixed_target, "reference target bound: {context}");
            assert_eq!(
                world.round.cumulative_withdrawal_fee_units
                    + world.round.cumulative_burned_units,
                world.round.cumulative_jitosol_assigned_units,
                "fee-plus-burn reconciliation: {context}"
            );
            if let Some(fixed) = self.fixed_snapshot {
                assert_eq!(snapshot(&world.round), fixed, "snapshot mutation: {context}");
            }
        }
        for (index, leg) in legs.iter().enumerate() {
            leg.validate()
                .unwrap_or_else(|error| panic!("leg validation: {context}, index={index}, error={error:?}"));
            let expected_status = match self.leg_status[index] {
                0 => WithdrawalLegStatus::Vacant,
                1 => WithdrawalLegStatus::Initiated,
                2 => WithdrawalLegStatus::Finalized,
                value => panic!("invalid reference leg status {value}"),
            };
            assert_eq!(leg.status, expected_status, "leg status: {context}, index={index}");
        }
        assert_valid_world(world);
    }
}

fn assert_rejected_world_unchanged(
    before: &(PivConfig, ActiveDistribution, [GuardianReward; GUARDIAN_COUNT], [WithdrawalLeg; 4]),
    world: &World,
    legs: &[WithdrawalLeg; 4],
    context: &str,
) {
    assert_eq!(world.config, before.0, "config changed on rejection: {context}");
    assert_eq!(world.round, before.1, "round changed on rejection: {context}");
    assert_eq!(world.rewards, before.2, "rewards changed on rejection: {context}");
    assert_eq!(*legs, before.3, "legs changed on rejection: {context}");
}

#[test]
fn randomized_model_based_instruction_orderings_preserve_lifecycle_invariants() {
    const MACHINE_SEEDS: usize = 192;
    const ACTIONS_PER_SEED: usize = 128;
    const GROSS_YIELD: u64 = 10_000;
    const OUTGOING: u64 = 8_050;
    const FLOOR: u64 = 100;

    let mut successful_opens = 0_usize;
    let mut successful_legs = 0_usize;
    let mut successful_finalizations = 0_usize;
    let mut successful_settlements = 0_usize;
    let mut successful_completions = 0_usize;
    let mut rejected_actions = 0_usize;
    let mut recovery_entries = 0_usize;
    let mut out_of_order_finalizations = 0_usize;
    let mut paused_rejections = [0_usize; 7];

    for seed_index in 0..MACHINE_SEEDS {
        let seed = STATE_SEED ^ (seed_index as u64).wrapping_mul(0x9e37_79b9);
        let mut rng = SplitMix64::new(seed);
        let mut world = world(0, (rng.next_u64() & 0x3f) as u8);
        let mut legs: [WithdrawalLeg; 4] =
            core::array::from_fn(|index| WithdrawalLeg::vacant(30 + index as u8, 40 + index as u8));
        let mut model = ReferenceModel::new(world.config.protected_principal_hwm_lamports);

        for action_index in 0..ACTIONS_PER_SEED {
            let action = rng.bounded(12);
            let context = format!(
                "seed={seed:#018x}, seed_index={seed_index}, action={action_index}, selector={action}"
            );
            let before = (
                world.config.clone(),
                world.round,
                world.rewards,
                legs,
            );
            let paused = world.config.paused;

            match action {
                0 | 1 => {
                    let legal = model.phase == ModelPhase::Idle && !paused;
                    if legal {
                        world.config.accounted_pending_sol_lamports = if action == 0 {
                            OUTGOING
                        } else {
                            1_000
                        };
                        world.config.accounted_pending_jitosol_units = rng.bounded(100);
                        legs = core::array::from_fn(|index| {
                            WithdrawalLeg::vacant(30 + index as u8, 40 + index as u8)
                        });
                        model.leg_status = [0; 4];
                    }
                    let prepared_at = world
                        .config
                        .last_successful_preparation_at
                        .map_or(0, |last| last + MINIMUM_DISTRIBUTION_INTERVAL_SECONDS);
                    let input = if action == 0 {
                        liquid_input(&world.config, prepared_at, GROSS_YIELD)
                    } else {
                        let target = (2 + rng.bounded(3)) * FLOOR;
                        withdrawal_input(
                            &world.config,
                            prepared_at,
                            GROSS_YIELD,
                            target,
                            FLOOR,
                        )
                    };
                    let opened_target = match input.funding {
                        DistributionFunding::Liquid { .. } => 0,
                        DistributionFunding::Withdrawal {
                            fixed_jitosol_target_units,
                            ..
                        } => fixed_jitosol_target_units,
                    };
                    let result = open_distribution(
                        &mut world.config,
                        &mut world.round,
                        &world.registry,
                        &world.rewards,
                        input,
                    );
                    if legal {
                        assert_eq!(result, Ok(()), "{context}");
                        model.phase = if action == 0 {
                            ModelPhase::Escrow
                        } else {
                            ModelPhase::Withdrawal
                        };
                        model.active_sequence = Some(model.next_sequence);
                        model.next_sequence += 1;
                        model.fixed_target = opened_target;
                        model.assigned = 0;
                        model.successful = 0;
                        model.finalized = 0;
                        model.fixed_snapshot = Some(snapshot(&world.round));
                        successful_opens += 1;
                    } else {
                        assert!(result.is_err(), "illegal open succeeded: {context}");
                        assert_rejected_world_unchanged(&before, &world, &legs, &context);
                        if paused {
                            paused_rejections[0] += 1;
                        }
                        rejected_actions += 1;
                    }
                }
                2 => {
                    let legal = model.phase == ModelPhase::Withdrawal
                        && !paused
                        && model.assigned < model.fixed_target
                        && model.successful < 4;
                    let index = usize::try_from(world.round.next_leg_index.min(3))
                        .expect("bounded leg index");
                    let remaining = world
                        .round
                        .remaining_withdrawal_target_units()
                        .unwrap_or(0);
                    let capacity = if legal {
                        let chunks = 1 + rng.bounded(remaining / FLOOR);
                        chunks * FLOOR
                    } else {
                        FLOOR
                    };
                    let input = initiation_input(&world.round, capacity, 0);
                    let assigned = input.jitosol_input_units;
                    let result = initiate_withdrawal_leg(
                        &world.config,
                        &mut world.round,
                        &mut legs[index],
                        input,
                    );
                    if legal {
                        assert_eq!(result, Ok(()), "{context}");
                        model.assigned += assigned;
                        model.leg_status[index] = 1;
                        model.successful += 1;
                        successful_legs += 1;
                    } else {
                        assert!(result.is_err(), "illegal initiation succeeded: {context}");
                        assert_rejected_world_unchanged(&before, &world, &legs, &context);
                        if paused {
                            paused_rejections[1] += 1;
                        }
                        rejected_actions += 1;
                    }
                }
                3 => {
                    let index = rng.bounded(4) as usize;
                    let legal = model.phase == ModelPhase::Withdrawal
                        && !paused
                        && model.leg_status[index] == 1;
                    let residual = world.round.stored_residual_hwm_floor_lamports;
                    let input = finalization_input(&world.round, &legs[index], 0, 0, residual);
                    let result = finalize_withdrawal_leg(
                        &world.config,
                        &mut world.round,
                        &mut legs[index],
                        input,
                    );
                    if legal {
                        if model.leg_status[..index].contains(&1) {
                            out_of_order_finalizations += 1;
                        }
                        model.leg_status[index] = 2;
                        model.finalized += 1;
                        let complete = model.assigned == model.fixed_target
                            && model.finalized == model.successful;
                        let expected = if complete {
                            model.phase = ModelPhase::Escrow;
                            LegFinalizationOutcome::EscrowFunded
                        } else {
                            LegFinalizationOutcome::Recorded
                        };
                        assert_eq!(result, Ok(expected), "{context}");
                        successful_finalizations += 1;
                    } else {
                        assert!(result.is_err(), "illegal finalization succeeded: {context}");
                        assert_rejected_world_unchanged(&before, &world, &legs, &context);
                        if paused {
                            paused_rejections[2] += 1;
                        }
                        rejected_actions += 1;
                    }
                }
                4 => {
                    let legal = model.phase == ModelPhase::Escrow && !paused;
                    let input = settle_input(&world.round, u64::MAX);
                    let result = settle_distribution(
                        &mut world.config,
                        &mut world.round,
                        &mut world.rewards,
                        input,
                    );
                    if legal {
                        assert_eq!(result, Ok(SettlementOutcome::Settled), "{context}");
                        assert_eq!(
                            world.round.actual_allocated_outgoing_lamports
                                + world.round.actual_escrow_remainder_lamports,
                            world.round.recorded_escrow_available_lamports,
                            "settlement conservation: {context}"
                        );
                        model.phase = ModelPhase::Settled;
                        successful_settlements += 1;
                    } else {
                        assert!(result.is_err(), "illegal settlement succeeded: {context}");
                        assert_rejected_world_unchanged(&before, &world, &legs, &context);
                        if paused {
                            paused_rejections[3] += 1;
                        }
                        rejected_actions += 1;
                    }
                }
                5 => {
                    let legal = model.phase == ModelPhase::Settled && !paused;
                    let contribution = world.config.accounted_pending_sol_lamports
                        + world.config.accounted_pending_jitosol_units;
                    let input = PendingIntegrationInput {
                        sequence: world.round.active_sequence,
                        completed_at: world.round.prepared_at + 1,
                        integrated_pending_sol_lamports: world
                            .config
                            .accounted_pending_sol_lamports,
                        integrated_pending_jitosol_units: world
                            .config
                            .accounted_pending_jitosol_units,
                        contribution_value_lamports: contribution,
                        new_accounted_historical_jitosol_units: world
                            .config
                            .accounted_historical_jitosol_units
                            + world.config.accounted_pending_jitosol_units,
                        new_accounted_historical_sol_lamports: world
                            .config
                            .accounted_historical_sol_lamports
                            + world.config.accounted_pending_sol_lamports,
                        new_protected_hwm_lamports: world
                            .config
                            .protected_principal_hwm_lamports
                            + contribution,
                    };
                    let result = integrate_pending_and_complete(
                        &mut world.config,
                        &mut world.round,
                        input,
                    );
                    if legal {
                        let summary = result.expect("legal model completion");
                        assert_eq!(Some(summary.sequence), model.active_sequence, "{context}");
                        model.phase = ModelPhase::Idle;
                        model.active_sequence = None;
                        model.fixed_target = 0;
                        model.assigned = 0;
                        model.successful = 0;
                        model.finalized = 0;
                        model.leg_status = [0; 4];
                        legs = core::array::from_fn(|index| {
                            WithdrawalLeg::vacant(30 + index as u8, 40 + index as u8)
                        });
                        model.fixed_snapshot = None;
                        successful_completions += 1;
                    } else {
                        assert!(result.is_err(), "illegal completion succeeded: {context}");
                        assert_rejected_world_unchanged(&before, &world, &legs, &context);
                        if paused {
                            paused_rejections[4] += 1;
                        }
                        rejected_actions += 1;
                    }
                }
                6 => {
                    let mut input = settle_input(&world.round, u64::MAX);
                    input.sequence = input.sequence.wrapping_add(1);
                    let result = settle_distribution(
                        &mut world.config,
                        &mut world.round,
                        &mut world.rewards,
                        input,
                    );
                    assert!(result.is_err(), "wrong-sequence settlement succeeded: {context}");
                    assert_rejected_world_unchanged(&before, &world, &legs, &context);
                    rejected_actions += 1;
                }
                7 => {
                    world.config.paused = !world.config.paused;
                }
                8 => {
                    let legal = model.phase == ModelPhase::Escrow && !paused;
                    let input = settle_input(&world.round, 0);
                    let result = settle_distribution(
                        &mut world.config,
                        &mut world.round,
                        &mut world.rewards,
                        input,
                    );
                    if legal {
                        assert_eq!(
                            result,
                            Ok(SettlementOutcome::RecoveryRequired),
                            "{context}"
                        );
                        assert_eq!(world.config, before.0, "recovery changed config: {context}");
                        assert_eq!(world.rewards, before.2, "recovery changed rewards: {context}");
                        model.phase = ModelPhase::Recovery;
                        recovery_entries += 1;
                    } else {
                        assert!(result.is_err(), "illegal recovery settlement succeeded: {context}");
                        assert_rejected_world_unchanged(&before, &world, &legs, &context);
                        if paused {
                            paused_rejections[3] += 1;
                        }
                        rejected_actions += 1;
                    }
                }
                9 => {
                    let legal = model.phase == ModelPhase::Idle && !paused;
                    let attempted_at = world
                        .config
                        .last_successful_preparation_at
                        .map_or(0, |last| last + MINIMUM_DISTRIBUTION_INTERVAL_SECONDS)
                        .max(
                            world
                                .config
                                .last_valid_insufficient_attempt_at
                                .map_or(0, |last| {
                                    last + INSUFFICIENT_RETRY_COOLDOWN_SECONDS
                                }),
                        );
                    let historical_value = world
                        .config
                        .protected_principal_hwm_lamports
                        + GROSS_YIELD
                        - world.config.next_cycle_yield_lamports;
                    let input = ValidInsufficientAttemptInput {
                        attempted_at,
                        historical_value_lamports: historical_value,
                        pending_sol_snapshot_lamports: world
                            .config
                            .accounted_pending_sol_lamports,
                        computed_jitosol_target_units: 99,
                        validated_technical_minimum_units: 100,
                    };
                    let result = record_valid_insufficient_attempt(
                        &mut world.config,
                        &world.round,
                        input,
                    );
                    if legal && world.config.accounted_pending_sol_lamports < OUTGOING {
                        assert_eq!(result, Ok(()), "{context}");
                    } else {
                        assert!(result.is_err(), "illegal insufficient result succeeded: {context}");
                        assert_rejected_world_unchanged(&before, &world, &legs, &context);
                        if paused {
                            paused_rejections[5] += 1;
                        }
                        rejected_actions += 1;
                    }
                }
                10 => {
                    let legal = model.phase == ModelPhase::Idle && !paused;
                    let evaluated_at = world
                        .config
                        .last_successful_preparation_at
                        .map_or(0, |last| last + MINIMUM_DISTRIBUTION_INTERVAL_SECONDS);
                    let historical_value = world
                        .config
                        .protected_principal_hwm_lamports
                        .saturating_sub(world.config.next_cycle_yield_lamports);
                    let result = record_no_yield_evaluation(
                        &world.config,
                        &world.round,
                        evaluated_at,
                        historical_value,
                    );
                    if legal {
                        assert_eq!(result, Ok(()), "{context}");
                    } else {
                        assert!(result.is_err(), "illegal no-yield result succeeded: {context}");
                        assert_rejected_world_unchanged(&before, &world, &legs, &context);
                        if paused {
                            paused_rejections[6] += 1;
                        }
                        rejected_actions += 1;
                    }
                }
                _ => {
                    let index = model
                        .leg_status
                        .iter()
                        .position(|status| *status != 0)
                        .unwrap_or(0);
                    if model.leg_status[index] == 0 {
                        let mut input = initiation_input(&world.round, FLOOR, 0);
                        input.leg_index = input.leg_index.wrapping_add(1);
                        let result = initiate_withdrawal_leg(
                            &world.config,
                            &mut world.round,
                            &mut legs[index],
                            input,
                        );
                        assert!(result.is_err(), "bad-index initiation succeeded: {context}");
                    } else {
                        let input = finalization_input(
                            &world.round,
                            &legs[index],
                            0,
                            0,
                            world.round.stored_residual_hwm_floor_lamports,
                        );
                        let result = finalize_withdrawal_leg(
                            &world.config,
                            &mut world.round,
                            &mut legs[index],
                            input,
                        );
                        if model.leg_status[index] == 1
                            && model.phase == ModelPhase::Withdrawal
                            && !paused
                        {
                            model.leg_status[index] = 2;
                            model.finalized += 1;
                            let complete = model.assigned == model.fixed_target
                                && model.finalized == model.successful;
                            if complete {
                                model.phase = ModelPhase::Escrow;
                                assert_eq!(result, Ok(LegFinalizationOutcome::EscrowFunded), "{context}");
                            } else {
                                assert_eq!(result, Ok(LegFinalizationOutcome::Recorded), "{context}");
                            }
                            successful_finalizations += 1;
                            model.assert_matches(&world, &legs, &context);
                            model.last_hwm = world.config.protected_principal_hwm_lamports;
                            continue;
                        }
                        assert!(result.is_err(), "replayed action succeeded: {context}");
                    }
                    assert_rejected_world_unchanged(&before, &world, &legs, &context);
                    rejected_actions += 1;
                }
            }

            model.assert_matches(&world, &legs, &context);
            model.last_hwm = world.config.protected_principal_hwm_lamports;
        }
    }

    assert!(successful_opens >= 100, "insufficient open coverage: {successful_opens}");
    assert!(successful_legs >= 40, "insufficient leg coverage: {successful_legs}");
    assert!(
        successful_finalizations >= 30,
        "insufficient finalization coverage: {successful_finalizations}"
    );
    assert!(
        successful_settlements >= 20,
        "insufficient settlement coverage: {successful_settlements}"
    );
    assert!(
        successful_completions >= 5,
        "insufficient completion coverage: {successful_completions}"
    );
    assert!(rejected_actions >= 1_000, "insufficient rejection coverage: {rejected_actions}");
    assert!(recovery_entries >= 10, "insufficient recovery coverage: {recovery_entries}");
    assert!(
        out_of_order_finalizations >= 5,
        "insufficient out-of-order finalization coverage: {out_of_order_finalizations}"
    );
    assert!(
        paused_rejections.into_iter().all(|count| count > 0),
        "missing paused action category: {paused_rejections:?}"
    );
    eprintln!(
        "model coverage: opens={successful_opens}, legs={successful_legs}, \
         finalizations={successful_finalizations}, settlements={successful_settlements}, \
         completions={successful_completions}, rejected={rejected_actions}, \
         recovery_entries={recovery_entries}, \
         out_of_order_finalizations={out_of_order_finalizations}, \
         paused_categories={paused_rejections:?}"
    );
}

#[test]
fn pending_contributions_and_guardian_rotation_cannot_rewrite_active_snapshots() {
    let mut rng = SplitMix64::new(STATE_SEED ^ 0x5045_4e44_494e_4700);
    for case in 0..SERIALIZATION_CASES {
        let context = format!("seed={STATE_SEED:#018x}, pending_case={case}");
        let bitmap = (rng.next_u64() & 0x3f) as u8;
        let mut world = world(1_000, bitmap);
        let open = withdrawal_input(&world.config, 0, 10_000, 200, 100);
        open_distribution(
            &mut world.config,
            &mut world.round,
            &world.registry,
            &world.rewards,
            open,
        )
        .unwrap_or_else(|error| {
            panic!("open snapshot-immutability round: {context}, error={error:?}")
        });
        let immutable = snapshot(&world.round);

        let late_sol = rng.bounded(100_000);
        let late_jitosol = rng.bounded(100_000);
        world.config.accounted_pending_sol_lamports += late_sol;
        world.config.accounted_pending_jitosol_units += late_jitosol;
        let rotated_keys = core::array::from_fn(|index| key(200 + index as u8));
        world.registry = GuardianRegistry::new(71, 8, rotated_keys)
            .expect("valid rotated registry");
        world.config.guardian_registry_revision = 8;
        assert_eq!(snapshot(&world.round), immutable, "{context}");

        let mut leg = WithdrawalLeg::vacant(31, 41);
        let mut initiation = initiation_input(&world.round, 200, 1);
        initiation.expected_native_lamports = 800;
        initiation.observed_delegated_native_lamports = 800;
        initiation.minimum_native_lamports = 800;
        initiate_withdrawal_leg(&world.config, &mut world.round, &mut leg, initiation)
            .unwrap_or_else(|error| {
                panic!("initiate immutable-snapshot leg: {context}, error={error:?}")
            });
        let finalization = finalization_input(
            &world.round,
            &leg,
            0,
            0,
            world.round.stored_residual_hwm_floor_lamports,
        );
        finalize_withdrawal_leg(
            &world.config,
            &mut world.round,
            &mut leg,
            finalization,
        )
        .unwrap_or_else(|error| {
            panic!("finalize immutable-snapshot leg: {context}, error={error:?}")
        });
        assert_eq!(snapshot(&world.round), immutable, "{context}");

        let settlement = settle_input(&world.round, u64::MAX);
        settle_distribution(
            &mut world.config,
            &mut world.round,
            &mut world.rewards,
            settlement,
        )
        .unwrap_or_else(|error| {
            panic!("settle immutable-snapshot round: {context}, error={error:?}")
        });
        assert_eq!(snapshot(&world.round), immutable, "{context}");
        assert_eq!(world.round.guardian_registry_revision, 7, "{context}");
        assert_eq!(world.round.guardian_keys, immutable.guardian_keys, "{context}");

        let all_pending_sol = 1_000 + late_sol;
        let contribution_value = all_pending_sol + late_jitosol;
        let old_hwm = world.config.protected_principal_hwm_lamports;
        let input = PendingIntegrationInput {
            sequence: world.round.active_sequence,
            completed_at: 1,
            integrated_pending_sol_lamports: all_pending_sol,
            integrated_pending_jitosol_units: late_jitosol,
            contribution_value_lamports: contribution_value,
            new_accounted_historical_jitosol_units: 10_000 + late_jitosol,
            new_accounted_historical_sol_lamports: all_pending_sol,
            new_protected_hwm_lamports: old_hwm + contribution_value,
        };
        let summary = integrate_pending_and_complete(
            &mut world.config,
            &mut world.round,
            input,
        )
        .unwrap_or_else(|error| {
            panic!(
                "pending integration {context}, error={error:?}"
            )
        });
        assert_eq!(summary.integrated_contribution_value_lamports, contribution_value, "{context}");
        assert_eq!(world.config.accounted_pending_sol_lamports, 0, "{context}");
        assert_eq!(world.config.accounted_pending_jitosol_units, 0, "{context}");
        assert_eq!(world.config.protected_principal_hwm_lamports, old_hwm + contribution_value, "{context}");
        assert_eq!(world.round.lifecycle, DistributionLifecycle::Idle, "{context}");
    }
}

#[test]
fn valid_insufficient_attempts_update_only_the_permitted_timestamp() {
    let mut rng = SplitMix64::new(ADVERSARIAL_SEED ^ 0x494e_5355_4646_0000);
    let mut accepted = 0_usize;
    for case in 0..STATE_RANDOM_CASES {
        let mode = rng.bounded(12);
        let mut world = world(rng.bounded(80), 0);
        let base = 1_000_000_i64;
        let mut attempted_at = base;
        let mut input = ValidInsufficientAttemptInput {
            attempted_at,
            historical_value_lamports: OLD_HWM + 100,
            pending_sol_snapshot_lamports: world.config.accounted_pending_sol_lamports,
            computed_jitosol_target_units: 24,
            validated_technical_minimum_units: 25,
        };
        let should_succeed = match mode {
            0 => true,
            1 => {
                world.config.last_successful_preparation_at = Some(base);
                attempted_at = base + MINIMUM_DISTRIBUTION_INTERVAL_SECONDS - 1;
                false
            }
            2 => {
                world.config.last_successful_preparation_at = Some(base + 1);
                false
            }
            3 => {
                world.config.last_valid_insufficient_attempt_at = Some(base);
                attempted_at = base + INSUFFICIENT_RETRY_COOLDOWN_SECONDS - 1;
                false
            }
            4 => {
                world.config.last_valid_insufficient_attempt_at = Some(base + 1);
                false
            }
            5 => {
                input.pending_sol_snapshot_lamports =
                    input.pending_sol_snapshot_lamports.wrapping_add(1);
                false
            }
            6 => {
                input.computed_jitosol_target_units = 25;
                false
            }
            7 => {
                input.computed_jitosol_target_units = 0;
                false
            }
            8 => {
                input.historical_value_lamports = OLD_HWM;
                false
            }
            9 => {
                world.config.accounted_pending_sol_lamports = 80;
                input.pending_sol_snapshot_lamports = 80;
                false
            }
            _ => {
                if mode == 10 {
                    world.config.paused = true;
                    false
                } else {
                    attempted_at = base + MINIMUM_DISTRIBUTION_INTERVAL_SECONDS;
                    world.config.last_successful_preparation_at = Some(base);
                    world.config.last_valid_insufficient_attempt_at = Some(
                        attempted_at - INSUFFICIENT_RETRY_COOLDOWN_SECONDS,
                    );
                    true
                }
            }
        };
        input.attempted_at = attempted_at;
        let before = world.clone();
        let result = record_valid_insufficient_attempt(
            &mut world.config,
            &world.round,
            input,
        );
        let context = format!("seed={ADVERSARIAL_SEED:#018x}, case={case}, mode={mode}");
        if should_succeed {
            assert_eq!(result, Ok(()), "{context}");
            let mut expected = before.config;
            expected.last_valid_insufficient_attempt_at = Some(attempted_at);
            assert_eq!(world.config, expected, "{context}");
            assert_eq!(world.round, before.round, "{context}");
            assert_eq!(world.registry, before.registry, "{context}");
            assert_eq!(world.rewards, before.rewards, "{context}");
            accepted += 1;
        } else {
            assert!(result.is_err(), "invalid insufficiency proof succeeded: {context}");
            assert_eq!(world.config, before.config, "{context}");
            assert_eq!(world.round, before.round, "{context}");
            assert_eq!(world.registry, before.registry, "{context}");
            assert_eq!(world.rewards, before.rewards, "{context}");
        }
    }
    assert!(accepted >= 300, "insufficient valid-attempt coverage: {accepted}");
}

#[test]
fn randomized_adversarial_leg_inputs_and_replays_are_atomic() {
    let mut base = world(1_000, 0b00_0001);
    let open = withdrawal_input(&base.config, 0, 10_000, 200, 100);
    open_distribution(
        &mut base.config,
        &mut base.round,
        &base.registry,
        &base.rewards,
        open,
    )
    .expect("open adversarial initiation fixture");

    let mut rng = SplitMix64::new(ADVERSARIAL_SEED);
    for case in 0..STATE_RANDOM_CASES {
        let mode = rng.bounded(12);
        let mut world = base.clone();
        let mut leg = WithdrawalLeg::vacant(31, 41);
        let mut input = initiation_input(&world.round, 200, 1);
        match mode {
            0 => input.sequence = input.sequence.wrapping_add(1),
            1 => input.leg_index = 1,
            2 => {
                input.maximum_safe_capacity_units = 150;
                input.jitosol_input_units = 149;
                input.withdrawal_fee_units = 1;
                input.burned_units = 148;
            }
            3 => {
                input.maximum_safe_capacity_units = 0;
                input.jitosol_input_units = 0;
                input.withdrawal_fee_units = 0;
                input.burned_units = 0;
            }
            4 => {
                input.jitosol_input_units = 201;
                input.withdrawal_fee_units = 1;
                input.burned_units = 200;
            }
            5 => input.current_technical_floor_units = 201,
            6 => input.withdrawal_fee_units += 1,
            7 => input.minimum_native_lamports = input.expected_native_lamports + 1,
            8 => input.observed_delegated_native_lamports = input.minimum_native_lamports - 1,
            9 => input.pool_total_lamports = 0,
            10 => input.validator_vote = Pubkey::default(),
            _ => world.config.paused = true,
        }
        let config_before = world.config.clone();
        let round_before = world.round;
        let leg_before = leg;
        let result = initiate_withdrawal_leg(
            &world.config,
            &mut world.round,
            &mut leg,
            input,
        );
        let context = format!("seed={ADVERSARIAL_SEED:#018x}, case={case}, mode={mode}");
        assert!(result.is_err(), "adversarial initiation succeeded: {context}");
        assert_eq!(world.config, config_before, "{context}");
        assert_eq!(world.round, round_before, "{context}");
        assert_eq!(leg, leg_before, "{context}");
    }

    let mut initiated_world = base;
    let mut initiated_leg = WithdrawalLeg::vacant(31, 41);
    let initiation = initiation_input(&initiated_world.round, 200, 1);
    initiate_withdrawal_leg(
        &initiated_world.config,
        &mut initiated_world.round,
        &mut initiated_leg,
        initiation,
    )
    .expect("initiate adversarial finalization fixture");
    for case in 0..STATE_RANDOM_CASES {
        let mode = rng.bounded(11);
        let mut world = initiated_world.clone();
        let mut leg = initiated_leg;
        let residual = world.round.stored_residual_hwm_floor_lamports;
        let mut input = finalization_input(&world.round, &leg, 0, 0, residual);
        match mode {
            0 => input.sequence = input.sequence.wrapping_add(1),
            1 => input.leg_index = 1,
            2 => input.finalized_epoch = leg.initiation_epoch - 1,
            3 => input.recovered_stake_rent_lamports += 1,
            4 => input.recovered_metadata_rent_lamports += 1,
            5 => {
                input.cooldown_reward_lamports = 1;
                input.cooldown_loss_lamports = 1;
            }
            6 => input.finalized_native_lamports += 1,
            7 => input.escrow_available_after_lamports -= 1,
            8 => input.escrow_available_after_lamports += 1,
            9 => world.config.paused = true,
            _ => world.config.next_distribution_sequence += 1,
        }
        let config_before = world.config.clone();
        let round_before = world.round;
        let leg_before = leg;
        let result = finalize_withdrawal_leg(
            &world.config,
            &mut world.round,
            &mut leg,
            input,
        );
        let context = format!("seed={ADVERSARIAL_SEED:#018x}, finalization_case={case}, mode={mode}");
        assert!(result.is_err(), "adversarial finalization succeeded: {context}");
        assert_eq!(world.config, config_before, "{context}");
        assert_eq!(world.round, round_before, "{context}");
        assert_eq!(leg, leg_before, "{context}");
    }
}

#[test]
fn escrow_shortages_surpluses_pause_and_recovery_are_never_bypassed() {
    for delta in [-1_i64, 1] {
        let mut world = world(8_050, 0b00_0001);
        let open = liquid_input(&world.config, 0, 10_000);
        open_distribution(
            &mut world.config,
            &mut world.round,
            &world.registry,
            &world.rewards,
            open,
        )
        .expect("open escrow-adversarial round");
        let mut input = settle_input(&world.round, u64::MAX);
        input.escrow_available_lamports = if delta < 0 {
            input.escrow_available_lamports - 1
        } else {
            input.escrow_available_lamports + 1
        };
        let before = world.clone();
        assert_eq!(
            settle_distribution(
                &mut world.config,
                &mut world.round,
                &mut world.rewards,
                input,
            ),
            Err(Piv1Error::EscrowReconciliationMismatch)
        );
        assert_eq!(world.config, before.config);
        assert_eq!(world.round, before.round);
        assert_eq!(world.rewards, before.rewards);
    }

    let mut recovery = world(8_050, 0b00_0001);
    let open = liquid_input(&recovery.config, 0, 10_000);
    open_distribution(
        &mut recovery.config,
        &mut recovery.round,
        &recovery.registry,
        &recovery.rewards,
        open,
    )
    .expect("open recovery fixture");
    let input = settle_input(&recovery.round, 0);
    assert_eq!(
        settle_distribution(
            &mut recovery.config,
            &mut recovery.round,
            &mut recovery.rewards,
            input,
        ),
        Ok(SettlementOutcome::RecoveryRequired)
    );
    let before = recovery.clone();
    let input = settle_input(&recovery.round, u64::MAX);
    assert_eq!(
        settle_distribution(
            &mut recovery.config,
            &mut recovery.round,
            &mut recovery.rewards,
            input,
        ),
        Err(Piv1Error::RecoveryRequired)
    );
    assert_eq!(recovery.config, before.config);
    assert_eq!(recovery.round, before.round);
    assert_eq!(recovery.rewards, before.rewards);

    let mut paused = world(8_050, 0b00_0001);
    paused.config.paused = true;
    let before = paused.clone();
    let input = liquid_input(&paused.config, 0, 10_000);
    assert_eq!(
        open_distribution(
            &mut paused.config,
            &mut paused.round,
            &paused.registry,
            &paused.rewards,
            input,
        ),
        Err(Piv1Error::PausedOperation)
    );
    assert_eq!(paused.config, before.config);
    assert_eq!(paused.round, before.round);
    assert_eq!(paused.rewards, before.rewards);
}

#[test]
fn state_level_overflow_rejections_are_atomic() {
    let mut carry_overflow = world(8_050, 0b00_0001);
    carry_overflow.config.collective_kif_carry_lamports = u64::MAX;
    let open = liquid_input(&carry_overflow.config, 0, 10_000);
    open_distribution(
        &mut carry_overflow.config,
        &mut carry_overflow.round,
        &carry_overflow.registry,
        &carry_overflow.rewards,
        open,
    )
    .expect("open KIF-overflow fixture");
    let before = carry_overflow.clone();
    let input = settle_input(&carry_overflow.round, u64::MAX);
    assert_eq!(
        settle_distribution(
            &mut carry_overflow.config,
            &mut carry_overflow.round,
            &mut carry_overflow.rewards,
            input,
        ),
        Err(Piv1Error::ArithmeticOverflow)
    );
    assert_eq!(carry_overflow.config, before.config);
    assert_eq!(carry_overflow.round, before.round);
    assert_eq!(carry_overflow.rewards, before.rewards);

    let mut reward_overflow = world(8_050, 0b00_0001);
    reward_overflow.rewards[0].claimable_lamports = u64::MAX;
    reward_overflow.rewards[0].cumulative_earned = u64::MAX;
    reward_overflow.rewards[0]
        .validate()
        .expect("balanced maximum reward fixture");
    let open = liquid_input(&reward_overflow.config, 0, 10_000);
    open_distribution(
        &mut reward_overflow.config,
        &mut reward_overflow.round,
        &reward_overflow.registry,
        &reward_overflow.rewards,
        open,
    )
    .expect("open reward-overflow fixture");
    let before = reward_overflow.clone();
    let input = settle_input(&reward_overflow.round, u64::MAX);
    assert_eq!(
        settle_distribution(
            &mut reward_overflow.config,
            &mut reward_overflow.round,
            &mut reward_overflow.rewards,
            input,
        ),
        Err(Piv1Error::ArithmeticOverflow)
    );
    assert_eq!(reward_overflow.config, before.config);
    assert_eq!(reward_overflow.round, before.round);
    assert_eq!(reward_overflow.rewards, before.rewards);
}

#[test]
fn claim_kif_pause_treatment_remains_an_unimplemented_marker() {
    let marker = ClaimKif;
    assert_eq!(marker, ClaimKif::default());
    assert_eq!(core::mem::size_of::<ClaimKif>(), 0);
}
