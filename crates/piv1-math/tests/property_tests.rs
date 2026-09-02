use piv1_math::{
    allocate_kif, calculate_gross_yield, checked_increase_high_water_mark,
    checked_mul_div_ceil, checked_mul_div_floor, split_gross_yield,
    HighWaterMarkComponents, KifAllocation, MathError, BASIS_POINTS_DENOMINATOR,
    HTFP_RESERVE_BPS, KIF_BPS, MAX_ACTIVE_GUARDIANS, PERMANENT_COMPOUND_BPS,
    TEAM_OWNER_POOL_BPS,
};

const MUL_DIV_SEED: u64 = 0x5049_5631_4d55_4c44;
const ACCOUNTING_SEED: u64 = 0x5049_5631_4143_4354;
const RANDOM_CASES: usize = 25_000;

const BOUNDARY_VALUES: [u64; 17] = [
    0,
    1,
    2,
    3,
    5,
    6,
    49,
    50,
    9_999,
    10_000,
    10_001,
    u32::MAX as u64,
    (u32::MAX as u64) + 1,
    u64::MAX / 2,
    (u64::MAX / 2) + 1,
    u64::MAX - 1,
    u64::MAX,
];

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
}

fn expected_narrow(value: u128) -> Result<u64, MathError> {
    u64::try_from(value).map_err(|_| MathError::NarrowingConversion)
}

fn assert_mul_div_case(a: u64, b: u64, denominator: u64, context: &str) {
    let floor = checked_mul_div_floor(a, b, denominator);
    let ceil = checked_mul_div_ceil(a, b, denominator);

    if denominator == 0 {
        assert_eq!(
            floor,
            Err(MathError::DivisionByZero),
            "floor zero denominator: {context}, a={a}, b={b}"
        );
        assert_eq!(
            ceil,
            Err(MathError::DivisionByZero),
            "ceil zero denominator: {context}, a={a}, b={b}"
        );
        return;
    }

    let product = u128::from(a) * u128::from(b);
    let denominator_wide = u128::from(denominator);
    let floor_wide = product / denominator_wide;
    let remainder = product % denominator_wide;
    let ceil_wide = floor_wide + u128::from(remainder != 0);
    let expected_floor = expected_narrow(floor_wide);
    let expected_ceil = expected_narrow(ceil_wide);

    assert_eq!(
        floor, expected_floor,
        "independent floor oracle: {context}, a={a}, b={b}, denominator={denominator}"
    );
    assert_eq!(
        ceil, expected_ceil,
        "independent ceil oracle: {context}, a={a}, b={b}, denominator={denominator}"
    );

    if let Ok(floor_value) = floor {
        let lower = u128::from(floor_value) * denominator_wide;
        let upper = (u128::from(floor_value) + 1) * denominator_wide;
        assert!(
            lower <= product,
            "floor exceeded exact rational: {context}, a={a}, b={b}, denominator={denominator}"
        );
        assert!(
            product < upper,
            "floor was not greatest lower integer: {context}, a={a}, b={b}, denominator={denominator}"
        );
    }

    if let Ok(ceil_value) = ceil {
        assert!(
            u128::from(ceil_value) * denominator_wide >= product,
            "ceil fell below exact rational: {context}, a={a}, b={b}, denominator={denominator}"
        );
    }

    if let (Ok(floor_value), Ok(ceil_value)) = (floor, ceil) {
        assert!(
            ceil_value >= floor_value && ceil_value - floor_value <= 1,
            "floor/ceil distance: {context}, a={a}, b={b}, denominator={denominator}"
        );
        if remainder == 0 {
            assert_eq!(
                floor_value, ceil_value,
                "exact division differs: {context}, a={a}, b={b}, denominator={denominator}"
            );
        }
    }
}

#[test]
fn checked_multiply_divide_matches_independent_u128_reference() {
    for (a_index, a) in BOUNDARY_VALUES.into_iter().enumerate() {
        for (b_index, b) in BOUNDARY_VALUES.into_iter().enumerate() {
            for (denominator_index, denominator) in
                BOUNDARY_VALUES.into_iter().enumerate()
            {
                let context = format!(
                    "boundary indices {a_index}/{b_index}/{denominator_index}"
                );
                assert_mul_div_case(a, b, denominator, &context);
            }
        }
    }

    let mut rng = SplitMix64::new(MUL_DIV_SEED);
    for case in 0..RANDOM_CASES {
        let a = rng.next_u64();
        let b = rng.next_u64();
        let denominator = match case % 8 {
            0 => 0,
            1 => 1,
            2 => u64::MAX,
            3 => u64::MAX - 1,
            _ => rng.next_u64(),
        };
        let context = format!("seed={MUL_DIV_SEED:#018x}, case={case}");
        assert_mul_div_case(a, b, denominator, &context);
    }

    assert_eq!(
        checked_mul_div_floor(u64::MAX, u64::MAX, 1),
        Err(MathError::NarrowingConversion)
    );
    assert_eq!(
        checked_mul_div_ceil(u64::MAX - 1, u64::MAX - 1, u64::MAX - 2),
        Err(MathError::NarrowingConversion)
    );
}

fn assert_split_case(gross_yield: u64, context: &str) {
    let split = split_gross_yield(gross_yield)
        .unwrap_or_else(|error| panic!("split failed: {context}, error={error:?}"));
    let gross_wide = u128::from(gross_yield);
    let denominator = u128::from(BASIS_POINTS_DENOMINATOR);
    let expected_htfp = gross_wide * u128::from(HTFP_RESERVE_BPS) / denominator;
    let expected_compound =
        gross_wide * u128::from(PERMANENT_COMPOUND_BPS) / denominator;
    let expected_team =
        gross_wide * u128::from(TEAM_OWNER_POOL_BPS) / denominator;
    let expected_kif = gross_wide * u128::from(KIF_BPS) / denominator;

    assert_eq!(u128::from(split.htfp_reserve), expected_htfp, "{context}");
    assert_eq!(
        u128::from(split.permanent_compound),
        expected_compound,
        "{context}"
    );
    assert_eq!(
        u128::from(split.team_owner_pool),
        expected_team,
        "{context}"
    );
    assert_eq!(u128::from(split.kif), expected_kif, "{context}");

    let reconciled = u128::from(split.htfp_reserve)
        + u128::from(split.permanent_compound)
        + u128::from(split.team_owner_pool)
        + u128::from(split.kif)
        + u128::from(split.dust);
    let outgoing = u128::from(split.htfp_reserve)
        + u128::from(split.team_owner_pool)
        + u128::from(split.kif);
    assert_eq!(reconciled, gross_wide, "split conservation: {context}");
    assert!(outgoing <= gross_wide, "outgoing bound: {context}");
    for component in [
        split.htfp_reserve,
        split.permanent_compound,
        split.team_owner_pool,
        split.kif,
        split.dust,
    ] {
        assert!(component <= gross_yield, "component bound: {context}");
    }
}

#[test]
fn gross_yield_split_and_hwm_properties_hold_over_boundaries_and_random_values() {
    for (case, gross_yield) in BOUNDARY_VALUES.into_iter().enumerate() {
        assert_split_case(gross_yield, &format!("boundary case={case}"));
    }

    let mut rng = SplitMix64::new(ACCOUNTING_SEED);
    for case in 0..RANDOM_CASES {
        let historical_value = rng.next_u64();
        let high_water_mark = rng.next_u64();
        let expected_yield = if historical_value > high_water_mark {
            historical_value - high_water_mark
        } else {
            0
        };
        assert_eq!(
            calculate_gross_yield(historical_value, high_water_mark),
            expected_yield,
            "gross-yield seed={ACCOUNTING_SEED:#018x}, case={case}"
        );
        if historical_value <= high_water_mark {
            assert_eq!(expected_yield, 0, "loss recovery became yield, case={case}");
        }

        let gross_yield = rng.next_u64();
        assert_split_case(
            gross_yield,
            &format!("seed={ACCOUNTING_SEED:#018x}, case={case}"),
        );

        let old_hwm = rng.next_u64();
        let components = HighWaterMarkComponents {
            contribution_value: rng.next_u64(),
            normal_compound_allocation: rng.next_u64(),
            split_dust: rng.next_u64(),
            conversion_dust: rng.next_u64(),
            net_allocation_dust: rng.next_u64(),
            zero_active_kif_compound: rng.next_u64(),
        };
        let reference_increase = u128::from(components.contribution_value)
            + u128::from(components.normal_compound_allocation)
            + u128::from(components.split_dust)
            + u128::from(components.conversion_dust)
            + u128::from(components.net_allocation_dust)
            + u128::from(components.zero_active_kif_compound);
        let reference_new = u128::from(old_hwm) + reference_increase;
        let actual = checked_increase_high_water_mark(old_hwm, components);
        if reference_increase > u128::from(u64::MAX)
            || reference_new > u128::from(u64::MAX)
        {
            assert_eq!(
                actual,
                Err(MathError::AdditionOverflow),
                "HWM overflow seed={ACCOUNTING_SEED:#018x}, case={case}"
            );
        } else {
            let update = actual.unwrap_or_else(|error| {
                panic!(
                    "HWM valid reference rejected: seed={ACCOUNTING_SEED:#018x}, case={case}, error={error:?}"
                )
            });
            assert_eq!(
                u128::from(update.increase),
                reference_increase,
                "HWM increase seed={ACCOUNTING_SEED:#018x}, case={case}"
            );
            assert_eq!(
                u128::from(update.new_high_water_mark),
                reference_new,
                "HWM result seed={ACCOUNTING_SEED:#018x}, case={case}"
            );
            assert!(
                update.new_high_water_mark >= old_hwm,
                "HWM monotonicity seed={ACCOUNTING_SEED:#018x}, case={case}"
            );
        }
    }
}

fn assert_kif_case(current: u64, carry: u64, active_count: u8, context: &str) {
    let actual = allocate_kif(current, carry, active_count);
    if active_count > MAX_ACTIVE_GUARDIANS {
        assert_eq!(
            actual,
            Err(MathError::InvalidActiveGuardianCount {
                active_guardians: active_count,
            }),
            "invalid guardian count precedence: {context}"
        );
        return;
    }

    let available_wide = u128::from(current) + u128::from(carry);
    if available_wide > u128::from(u64::MAX) {
        assert_eq!(
            actual,
            Err(MathError::AdditionOverflow),
            "KIF available overflow: {context}"
        );
        return;
    }
    let available = available_wide as u64;

    match actual.unwrap_or_else(|error| panic!("KIF failed: {context}, error={error:?}")) {
        KifAllocation::ActiveGuardians(allocation) => {
            assert!(active_count > 0, "wrong active variant: {context}");
            let per_guardian = available / u64::from(active_count);
            let credited = per_guardian * u64::from(active_count);
            assert_eq!(allocation.kif_available, available, "{context}");
            assert_eq!(allocation.active_guardians, active_count, "{context}");
            assert_eq!(allocation.per_guardian, per_guardian, "{context}");
            assert_eq!(allocation.credited_total, credited, "{context}");
            assert_eq!(allocation.carry_next, available - credited, "{context}");
            assert_eq!(
                allocation.credited_total + allocation.carry_next,
                available,
                "KIF active conservation: {context}"
            );
            assert!(allocation.per_guardian <= available, "{context}");
        }
        KifAllocation::ZeroActiveGuardians(allocation) => {
            assert_eq!(active_count, 0, "wrong zero-active variant: {context}");
            assert_eq!(allocation.kif_available, available, "{context}");
            assert_eq!(allocation.compound_from_kif, available / 2, "{context}");
            assert_eq!(
                allocation.carry_next,
                available - (available / 2),
                "{context}"
            );
            assert_eq!(
                allocation.compound_from_kif + allocation.carry_next,
                available,
                "KIF zero-active conservation: {context}"
            );
        }
    }
}

#[test]
fn kif_allocation_is_equal_conservative_reproducible_and_checked() {
    for active_count in 0..=MAX_ACTIVE_GUARDIANS {
        for (case, available) in BOUNDARY_VALUES.into_iter().enumerate() {
            assert_kif_case(
                available,
                0,
                active_count,
                &format!("boundary active={active_count}, case={case}"),
            );
        }
    }

    let mut rng = SplitMix64::new(ACCOUNTING_SEED ^ 0x4b49_4600_0000_0000);
    for case in 0..RANDOM_CASES {
        let active_count = match case % 12 {
            0..=6 => (case % 12) as u8,
            7 => 7,
            8 => u8::MAX,
            _ => (rng.next_u64() % 9) as u8,
        };
        assert_kif_case(
            rng.next_u64(),
            rng.next_u64(),
            active_count,
            &format!("seed={ACCOUNTING_SEED:#018x}, case={case}"),
        );
    }

    let mut carry = u64::MAX;
    let mut total_compounded = 0_u128;
    for period in 0..65 {
        let allocation = allocate_kif(0, carry, 0).expect("repeated zero-active allocation");
        let KifAllocation::ZeroActiveGuardians(zero) = allocation else {
            panic!("unexpected active allocation in zero-active period {period}");
        };
        assert_eq!(zero.compound_from_kif + zero.carry_next, carry);
        total_compounded += u128::from(zero.compound_from_kif);
        carry = zero.carry_next;
    }
    assert_eq!(total_compounded + u128::from(carry), u128::from(u64::MAX));
    assert_eq!(carry, 1);
}
