mod support;

use piv1::integrations::{
    DelayedWithdrawalStatus, FeeFraction, FinalizeWithdrawalRequest,
    PoolSnapshot, SolDepositRequest, StakePoolAdapter,
    StakePoolError, StakeWithdrawalRequest, WithdrawalId, WithdrawalSourceId,
};
use support::stake_pool_mock::{
    MockFailurePoint, MockStakePool, MockWithdrawalSource,
    MAX_MOCK_WITHDRAWAL_SOURCES,
};

const RANDOM_SEED: u64 = 0x5049_5631_4144_5054;
const RANDOM_CASES: usize = 1_024;

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

fn snapshot() -> PoolSnapshot {
    PoolSnapshot {
        current_epoch: 40,
        last_update_epoch: 40,
        total_pool_lamports: 1_000_000,
        pool_token_supply: 1_000_000,
        sol_deposit_fee: FeeFraction::ZERO,
        stake_withdrawal_fee: FeeFraction::ZERO,
        minimum_delegation_lamports: 100,
        maximum_deposit_lamports: 100_000,
        available_withdrawal_lamports: 10_000,
        revision: 7,
    }
}

fn sources() -> [MockWithdrawalSource; MAX_MOCK_WITHDRAWAL_SOURCES] {
    let mut sources = [MockWithdrawalSource::VACANT; MAX_MOCK_WITHDRAWAL_SOURCES];
    sources[0] = MockWithdrawalSource::new(1, 500);
    sources[1] = MockWithdrawalSource::new(2, 300);
    sources[2] = MockWithdrawalSource::new(3, 200);
    sources
}

fn mock_pool() -> MockStakePool {
    MockStakePool::new(snapshot(), sources(), 10_000).expect("valid mock pool")
}

fn deposit_request(pool: &MockStakePool, native_lamports: u64) -> SolDepositRequest {
    SolDepositRequest {
        snapshot: pool.raw_snapshot().identity(),
        native_lamports,
        caller_minimum_pool_tokens_out: 0,
        slippage_bps: 1,
    }
}

fn withdrawal_id(sequence: u64, leg_index: u64) -> WithdrawalId {
    WithdrawalId {
        sequence,
        leg_index,
    }
}

fn withdrawal_request(
    pool: &MockStakePool,
    id: WithdrawalId,
    source_id: u32,
    remaining_target: u64,
) -> StakeWithdrawalRequest {
    StakeWithdrawalRequest {
        snapshot: pool.raw_snapshot().identity(),
        withdrawal_id: id,
        source_id: WithdrawalSourceId(source_id),
        remaining_pool_token_target: remaining_target,
        caller_minimum_native_lamports_out: 0,
        slippage_bps: 1,
    }
}

fn assert_deposit_rejected_unchanged(
    pool: &mut MockStakePool,
    request: SolDepositRequest,
    expected: StakePoolError,
) {
    let before = pool.clone();
    assert_eq!(pool.execute_protected_sol_deposit(request), Err(expected));
    assert_eq!(*pool, before);
}

fn assert_withdrawal_rejected_unchanged(
    pool: &mut MockStakePool,
    request: StakeWithdrawalRequest,
    expected: StakePoolError,
) {
    let before = pool.clone();
    assert_eq!(
        pool.initiate_protected_stake_withdrawal(request),
        Err(expected)
    );
    assert_eq!(*pool, before);
}

fn assert_finalization_rejected_unchanged(
    pool: &mut MockStakePool,
    id: WithdrawalId,
    expected: StakePoolError,
) {
    let before = pool.clone();
    assert_eq!(
        pool.finalize_delayed_stake_withdrawal(FinalizeWithdrawalRequest {
            withdrawal_id: id,
        }),
        Err(expected)
    );
    assert_eq!(*pool, before);
}

#[test]
fn snapshot_accepts_fresh_state_and_rejects_stale_or_impossible_state() {
    let valid = snapshot();
    assert_eq!(valid.validate(), Ok(()));
    assert_eq!(mock_pool().pool_snapshot(), Ok(valid));

    let stale = PoolSnapshot {
        current_epoch: 41,
        ..valid
    };
    assert_eq!(stale.validate(), Err(StakePoolError::StalePool));

    let future_update = PoolSnapshot {
        last_update_epoch: 41,
        ..valid
    };
    assert_eq!(future_update.validate(), Err(StakePoolError::InvalidSnapshot));

    let missing_total = PoolSnapshot {
        total_pool_lamports: 0,
        ..valid
    };
    assert_eq!(missing_total.validate(), Err(StakePoolError::InvalidSnapshot));

    let excess_liquidity = PoolSnapshot {
        available_withdrawal_lamports: valid.total_pool_lamports + 1,
        ..valid
    };
    assert_eq!(
        excess_liquidity.validate(),
        Err(StakePoolError::InvalidSnapshot)
    );
}

#[test]
fn snapshot_bootstrap_and_fee_fraction_rules_are_explicit() {
    let bootstrap = PoolSnapshot {
        total_pool_lamports: 0,
        pool_token_supply: 0,
        available_withdrawal_lamports: 0,
        ..snapshot()
    };
    assert!(bootstrap.is_bootstrap());
    assert_eq!(bootstrap.validate(), Ok(()));

    let impossible_supply = PoolSnapshot {
        pool_token_supply: 0,
        ..snapshot()
    };
    assert_eq!(
        impossible_supply.validate(),
        Err(StakePoolError::InvalidSnapshot)
    );

    assert_eq!(FeeFraction::ZERO.validate(), Ok(()));
    assert_eq!(
        FeeFraction {
            numerator: 0,
            denominator: 0,
        }
        .validate(),
        Err(StakePoolError::DivisionByZero)
    );
    assert_eq!(
        FeeFraction {
            numerator: 2,
            denominator: 1,
        }
        .validate(),
        Err(StakePoolError::InvalidFee)
    );
    assert_eq!(
        FeeFraction {
            numerator: 1,
            denominator: 1,
        }
        .validate(),
        Err(StakePoolError::InvalidFee)
    );
}

#[test]
fn bootstrap_deposit_is_one_to_one_and_establishes_nonzero_pool_state() {
    let bootstrap = PoolSnapshot {
        total_pool_lamports: 0,
        pool_token_supply: 0,
        available_withdrawal_lamports: 0,
        ..snapshot()
    };
    let mut pool = MockStakePool::new(bootstrap, sources(), 10_000)
        .expect("valid bootstrap mock");
    let request = deposit_request(&pool, 777);
    let execution = pool
        .execute_protected_sol_deposit(request)
        .expect("bootstrap deposit");
    assert_eq!(execution.quote.gross_pool_tokens, 777);
    assert_eq!(execution.actual_pool_tokens_out, 777);
    assert_eq!(pool.raw_snapshot().total_pool_lamports, 777);
    assert_eq!(pool.raw_snapshot().pool_token_supply, 777);
    assert_eq!(pool.validate_conservation(), Ok(()));
}

#[test]
fn exchange_rate_increase_and_decrease_move_quotes_in_opposite_directions() {
    let mut pool = mock_pool();
    let base = pool
        .quote_sol_deposit(deposit_request(&pool, 1_000))
        .expect("base quote");
    assert_eq!(base.quoted_pool_tokens_out, 1_000);

    pool.increase_exchange_rate(250_000)
        .expect("increase exchange rate");
    let increased = pool
        .quote_sol_deposit(deposit_request(&pool, 1_000))
        .expect("increased-rate quote");
    assert_eq!(increased.quoted_pool_tokens_out, 800);

    pool.decrease_exchange_rate(500_000)
        .expect("decrease exchange rate");
    let decreased = pool
        .quote_sol_deposit(deposit_request(&pool, 1_000))
        .expect("decreased-rate quote");
    assert_eq!(decreased.quoted_pool_tokens_out, 1_333);
    assert!(increased.quoted_pool_tokens_out < base.quoted_pool_tokens_out);
    assert!(decreased.quoted_pool_tokens_out > base.quoted_pool_tokens_out);
}

#[test]
fn snapshot_identity_rejects_revision_and_epoch_mismatch_atomically() {
    let mut pool = mock_pool();
    let old = pool.raw_snapshot().identity();
    pool.increase_exchange_rate(1).expect("change revision");
    let mut request = deposit_request(&pool, 100);
    request.snapshot = old;
    assert_deposit_rejected_unchanged(&mut pool, request, StakePoolError::StaleQuote);

    let current = pool.raw_snapshot().identity();
    pool.set_current_epoch(current.current_epoch + 1)
        .expect("advance current epoch");
    pool.set_last_update_epoch(current.current_epoch + 1)
        .expect("refresh epoch marker");
    let mut withdrawal = withdrawal_request(&pool, withdrawal_id(1, 0), 1, 100);
    withdrawal.snapshot = current;
    assert_withdrawal_rejected_unchanged(
        &mut pool,
        withdrawal,
        StakePoolError::StaleQuote,
    );
}

#[test]
fn stale_pool_rejects_quotes_and_snapshot_reads_without_mutation() {
    let mut pool = mock_pool();
    pool.set_current_epoch(41).expect("advance epoch");
    let before = pool.clone();
    assert_eq!(pool.pool_snapshot(), Err(StakePoolError::StalePool));
    let request = SolDepositRequest {
        snapshot: pool.raw_snapshot().identity(),
        native_lamports: 100,
        caller_minimum_pool_tokens_out: 0,
        slippage_bps: 1,
    };
    assert_eq!(
        pool.execute_protected_sol_deposit(request),
        Err(StakePoolError::StalePool)
    );
    assert_eq!(pool, before);
}

#[test]
fn deposits_cover_exact_rounding_zero_and_nonzero_fees() {
    let mut exact = mock_pool();
    let before = exact.raw_snapshot();
    let execution = exact
        .execute_protected_sol_deposit(deposit_request(&exact, 1_000))
        .expect("exact deposit");
    assert_eq!(execution.quote.gross_pool_tokens, 1_000);
    assert_eq!(execution.quote.deposit_fee_pool_tokens, 0);
    assert_eq!(execution.actual_pool_tokens_out, 1_000);
    assert_eq!(
        exact.raw_snapshot().total_pool_lamports,
        before.total_pool_lamports + 1_000
    );
    assert_eq!(
        exact.raw_snapshot().pool_token_supply,
        before.pool_token_supply + 1_000
    );

    let rounded_snapshot = PoolSnapshot {
        total_pool_lamports: 3,
        pool_token_supply: 2,
        available_withdrawal_lamports: 3,
        minimum_delegation_lamports: 1,
        ..snapshot()
    };
    let mut rounded = MockStakePool::new(rounded_snapshot, sources(), 10_000)
        .expect("rounding pool");
    let rounded_result = rounded
        .execute_protected_sol_deposit(deposit_request(&rounded, 2))
        .expect("rounding deposit");
    assert_eq!(rounded_result.quote.gross_pool_tokens, 1);
    assert_eq!(rounded_result.actual_pool_tokens_out, 1);

    let mut fee_pool = mock_pool();
    fee_pool
        .set_fees(
            FeeFraction {
                numerator: 1,
                denominator: 10,
            },
            FeeFraction::ZERO,
        )
        .expect("configure deposit fee");
    let fee_result = fee_pool
        .execute_protected_sol_deposit(deposit_request(&fee_pool, 1_001))
        .expect("fee deposit");
    assert_eq!(fee_result.quote.gross_pool_tokens, 1_001);
    assert_eq!(fee_result.quote.deposit_fee_pool_tokens, 101);
    assert_eq!(fee_result.actual_pool_tokens_out, 900);
    assert_eq!(fee_pool.audit().deposit_fee_pool_tokens, 101);
    assert_eq!(fee_pool.validate_conservation(), Ok(()));
}

#[test]
fn deposit_minimums_zero_input_liquidity_and_slippage_are_atomic() {
    let mut pool = mock_pool();
    let zero = deposit_request(&pool, 0);
    assert_deposit_rejected_unchanged(&mut pool, zero, StakePoolError::ZeroInput);

    let mut exact_minimum = deposit_request(&pool, 1_000);
    exact_minimum.caller_minimum_pool_tokens_out = 1_000;
    assert_eq!(
        pool.quote_sol_deposit(exact_minimum)
            .expect("exact minimum")
            .minimum_pool_tokens_out,
        1_000
    );

    let mut one_too_high = exact_minimum;
    one_too_high.caller_minimum_pool_tokens_out = 1_001;
    assert_deposit_rejected_unchanged(
        &mut pool,
        one_too_high,
        StakePoolError::SlippageExceeded,
    );

    let mut invalid_slippage = deposit_request(&pool, 1_000);
    invalid_slippage.slippage_bps = 2;
    assert_deposit_rejected_unchanged(
        &mut pool,
        invalid_slippage,
        StakePoolError::InvalidSlippage,
    );

    pool.set_maximum_deposit_lamports(999)
        .expect("restrict deposit capacity");
    let insufficient = deposit_request(&pool, 1_000);
    assert_deposit_rejected_unchanged(
        &mut pool,
        insufficient,
        StakePoolError::InsufficientPoolLiquidity,
    );
}

#[test]
fn deposit_arithmetic_and_narrowing_boundaries_are_atomic() {
    let overflow_snapshot = PoolSnapshot {
        total_pool_lamports: u64::MAX,
        pool_token_supply: u64::MAX,
        maximum_deposit_lamports: u64::MAX,
        available_withdrawal_lamports: u64::MAX,
        ..snapshot()
    };
    let mut overflow = MockStakePool::new(overflow_snapshot, sources(), 10_000)
        .expect("overflow fixture");
    let request = deposit_request(&overflow, 1);
    assert_deposit_rejected_unchanged(
        &mut overflow,
        request,
        StakePoolError::ArithmeticOverflow,
    );

    let narrowing_snapshot = PoolSnapshot {
        total_pool_lamports: 1,
        pool_token_supply: u64::MAX,
        minimum_delegation_lamports: 1,
        available_withdrawal_lamports: 1,
        maximum_deposit_lamports: u64::MAX,
        ..snapshot()
    };
    let narrowing = MockStakePool::new(narrowing_snapshot, sources(), 10_000)
        .expect("narrowing fixture");
    assert_eq!(
        narrowing.quote_sol_deposit(deposit_request(&narrowing, 2)),
        Err(StakePoolError::NarrowingConversion)
    );
}

#[test]
fn every_deposit_failure_injection_point_preserves_full_state() {
    for failure in [
        MockFailurePoint::DepositBeforeValidation,
        MockFailurePoint::DepositAfterQuote,
        MockFailurePoint::DepositBeforeCommit,
    ] {
        let mut pool = mock_pool();
        pool.set_failure(failure);
        let request = deposit_request(&pool, 1_000);
        assert_deposit_rejected_unchanged(
            &mut pool,
            request,
            StakePoolError::InjectedMockFailure,
        );
    }

    let mut pool = mock_pool();
    pool.set_failure(MockFailurePoint::SnapshotRead);
    let before = pool.clone();
    assert_eq!(
        pool.pool_snapshot(),
        Err(StakePoolError::InjectedMockFailure)
    );
    assert_eq!(pool, before);
}

#[test]
fn failed_mock_controls_also_preserve_complete_state() {
    let mut pool = mock_pool();
    let before = pool.clone();
    assert_eq!(
        pool.set_fees(
            FeeFraction {
                numerator: 1,
                denominator: 0,
            },
            FeeFraction::ZERO,
        ),
        Err(StakePoolError::DivisionByZero)
    );
    assert_eq!(pool, before);

    assert_eq!(
        pool.set_minimum_delegation_lamports(0),
        Err(StakePoolError::InvalidConfiguration)
    );
    assert_eq!(pool, before);

    assert_eq!(
        pool.decrease_exchange_rate(pool.raw_snapshot().total_pool_lamports + 1),
        Err(StakePoolError::InvalidConfiguration)
    );
    assert_eq!(pool, before);

    let maximum_revision_snapshot = PoolSnapshot {
        revision: u64::MAX,
        ..snapshot()
    };
    let mut maximum_revision =
        MockStakePool::new(maximum_revision_snapshot, sources(), 10_000)
            .expect("maximum-revision fixture");
    let maximum_before = maximum_revision.clone();
    assert_eq!(
        maximum_revision.set_maximum_deposit_lamports(1),
        Err(StakePoolError::ArithmeticOverflow)
    );
    assert_eq!(maximum_revision, maximum_before);
}

#[test]
fn withdrawal_exact_minimum_and_one_below_are_distinguished() {
    let mut pool = mock_pool();
    pool.set_minimum_delegation_lamports(150)
        .expect("configure technical minimum");
    let exact = pool
        .quote_stake_withdrawal(withdrawal_request(
            &pool,
            withdrawal_id(1, 0),
            1,
            150,
        ))
        .expect("exact technical minimum");
    assert_eq!(exact.technical_minimum_pool_tokens, 150);
    assert_eq!(exact.pool_tokens_in, 150);
    assert_eq!(exact.expected_delegated_native_lamports, 150);

    let request = withdrawal_request(&pool, withdrawal_id(1, 0), 1, 149);
    assert_withdrawal_rejected_unchanged(
        &mut pool,
        request,
        StakePoolError::TechnicalMinimumNotMet,
    );
}

#[test]
fn withdrawal_source_capacity_exact_partial_exhausted_and_stranded_cases() {
    let mut exact = mock_pool();
    exact
        .set_source_capacity(WithdrawalSourceId(1), 100)
        .expect("exact source capacity");
    let request = withdrawal_request(&exact, withdrawal_id(2, 0), 1, 100);
    let initiated = exact
        .initiate_protected_stake_withdrawal(request)
        .expect("exact-capacity withdrawal");
    assert_eq!(initiated.quote.pool_tokens_in, 100);
    assert_eq!(exact.source_capacity(WithdrawalSourceId(1)), Ok(0));

    let mut partial = mock_pool();
    partial
        .set_source_capacity(WithdrawalSourceId(1), 200)
        .expect("partial source capacity");
    let request = withdrawal_request(&partial, withdrawal_id(3, 0), 1, 300);
    let initiated = partial
        .initiate_protected_stake_withdrawal(request)
        .expect("partial source fill");
    assert_eq!(initiated.quote.pool_tokens_in, 200);
    assert_eq!(initiated.quote.remaining_pool_token_target, 300);

    let mut exhausted = mock_pool();
    exhausted
        .set_source_capacity(WithdrawalSourceId(1), 0)
        .expect("exhaust source");
    let request = withdrawal_request(&exhausted, withdrawal_id(4, 0), 1, 100);
    assert_withdrawal_rejected_unchanged(
        &mut exhausted,
        request,
        StakePoolError::InsufficientSourceCapacity,
    );

    let mut stranded = mock_pool();
    stranded
        .set_source_capacity(WithdrawalSourceId(1), 100)
        .expect("stranding source");
    let request = withdrawal_request(&stranded, withdrawal_id(5, 0), 1, 150);
    assert_withdrawal_rejected_unchanged(
        &mut stranded,
        request,
        StakePoolError::InsufficientSourceCapacity,
    );
}

#[test]
fn withdrawal_fees_burn_expected_output_and_slippage_reconcile() {
    let mut pool = mock_pool();
    pool.set_fees(
        FeeFraction::ZERO,
        FeeFraction {
            numerator: 1,
            denominator: 100,
        },
    )
    .expect("configure withdrawal fee");
    let quote = pool
        .quote_stake_withdrawal(withdrawal_request(
            &pool,
            withdrawal_id(6, 0),
            1,
            202,
        ))
        .expect("fee quote");
    assert_eq!(quote.technical_minimum_pool_tokens, 102);
    assert_eq!(quote.withdrawal_fee_pool_tokens, 3);
    assert_eq!(quote.burned_pool_tokens, 199);
    assert_eq!(quote.expected_delegated_native_lamports, 199);
    assert_eq!(
        quote.withdrawal_fee_pool_tokens + quote.burned_pool_tokens,
        quote.pool_tokens_in
    );

    let mut exact = withdrawal_request(&pool, withdrawal_id(6, 0), 1, 202);
    exact.caller_minimum_native_lamports_out = 199;
    let result = pool
        .initiate_protected_stake_withdrawal(exact)
        .expect("exact slippage threshold");
    assert_eq!(result.actual_delegated_native_lamports, 199);
    assert_eq!(pool.audit().withdrawal_fee_pool_tokens, 3);
    assert_eq!(pool.audit().burned_pool_tokens, 199);
    assert_eq!(pool.validate_conservation(), Ok(()));

    let mut failure_pool = mock_pool();
    failure_pool
        .set_fees(
            FeeFraction::ZERO,
            FeeFraction {
                numerator: 1,
                denominator: 100,
            },
        )
        .expect("configure failure-pool withdrawal fee");
    let mut fail = withdrawal_request(&failure_pool, withdrawal_id(6, 1), 2, 202);
    fail.caller_minimum_native_lamports_out = 200;
    assert_withdrawal_rejected_unchanged(
        &mut failure_pool,
        fail,
        StakePoolError::SlippageExceeded,
    );
}

#[test]
fn withdrawal_epoch_arithmetic_overflow_is_atomic() {
    let maximum_epoch_snapshot = PoolSnapshot {
        current_epoch: u64::MAX,
        last_update_epoch: u64::MAX,
        ..snapshot()
    };
    let mut pool = MockStakePool::new(maximum_epoch_snapshot, sources(), 10_000)
        .expect("maximum-epoch fixture");
    let request = withdrawal_request(&pool, withdrawal_id(6, 9), 1, 100);
    assert_withdrawal_rejected_unchanged(
        &mut pool,
        request,
        StakePoolError::ArithmeticOverflow,
    );
}

#[test]
fn withdrawal_liquidity_unknown_source_and_operational_rent_fail_atomically() {
    let mut exact_liquidity = mock_pool();
    exact_liquidity
        .set_available_withdrawal_lamports(100)
        .expect("set exact liquidity");
    let request = withdrawal_request(
        &exact_liquidity,
        withdrawal_id(7, 9),
        1,
        100,
    );
    assert_eq!(
        exact_liquidity
            .initiate_protected_stake_withdrawal(request)
            .expect("exact-liquidity withdrawal")
            .actual_delegated_native_lamports,
        100
    );

    let mut liquidity = mock_pool();
    liquidity
        .set_available_withdrawal_lamports(99)
        .expect("restrict liquidity");
    let request = withdrawal_request(&liquidity, withdrawal_id(7, 0), 1, 100);
    assert_withdrawal_rejected_unchanged(
        &mut liquidity,
        request,
        StakePoolError::InsufficientPoolLiquidity,
    );

    let mut unknown = mock_pool();
    let request = withdrawal_request(&unknown, withdrawal_id(7, 1), 99, 100);
    assert_withdrawal_rejected_unchanged(
        &mut unknown,
        request,
        StakePoolError::UnknownWithdrawalSource,
    );

    let mut no_rent = MockStakePool::new(snapshot(), sources(), 29)
        .expect("limited operational-rent fixture");
    let request = withdrawal_request(&no_rent, withdrawal_id(7, 2), 1, 100);
    assert_withdrawal_rejected_unchanged(
        &mut no_rent,
        request,
        StakePoolError::InsufficientOperationalRent,
    );
}

#[test]
fn multiple_sources_use_maximum_fills_and_the_minimum_selected_leg_count() {
    let mut pool = mock_pool();
    pool.set_source_capacity(WithdrawalSourceId(1), 600)
        .expect("largest source");
    pool.set_source_capacity(WithdrawalSourceId(2), 400)
        .expect("second source");
    pool.set_source_capacity(WithdrawalSourceId(3), 200)
        .expect("third source");

    let first = pool
        .initiate_protected_stake_withdrawal(withdrawal_request(
            &pool,
            withdrawal_id(8, 0),
            1,
            1_000,
        ))
        .expect("largest maximum fill");
    assert_eq!(first.quote.pool_tokens_in, 600);

    let second = pool
        .initiate_protected_stake_withdrawal(withdrawal_request(
            &pool,
            withdrawal_id(8, 1),
            2,
            400,
        ))
        .expect("second maximum fill");
    assert_eq!(second.quote.pool_tokens_in, 400);
    assert_eq!(pool.withdrawal_count(), 2);
    assert_eq!(pool.source_capacity(WithdrawalSourceId(3)), Ok(200));
    assert_eq!(pool.audit().withdrawal_input_pool_tokens, 1_000);
    assert_eq!(pool.validate_conservation(), Ok(()));
}

#[test]
fn identifier_reuse_zero_input_invalid_slippage_and_failures_are_atomic() {
    let mut pool = mock_pool();
    let id = withdrawal_id(9, 0);
    let request = withdrawal_request(&pool, id, 1, 100);
    pool.initiate_protected_stake_withdrawal(request)
        .expect("initial identifier use");
    let reused = withdrawal_request(&pool, id, 2, 100);
    assert_withdrawal_rejected_unchanged(
        &mut pool,
        reused,
        StakePoolError::IdentifierReuse,
    );

    let mut zero_pool = mock_pool();
    let zero = withdrawal_request(&zero_pool, withdrawal_id(9, 1), 2, 0);
    assert_withdrawal_rejected_unchanged(
        &mut zero_pool,
        zero,
        StakePoolError::ZeroInput,
    );

    let mut slippage_pool = mock_pool();
    let mut slippage = withdrawal_request(&slippage_pool, withdrawal_id(9, 2), 2, 100);
    slippage.slippage_bps = 2;
    assert_withdrawal_rejected_unchanged(
        &mut slippage_pool,
        slippage,
        StakePoolError::InvalidSlippage,
    );

    for failure in [
        MockFailurePoint::WithdrawalBeforeValidation,
        MockFailurePoint::WithdrawalAfterQuote,
        MockFailurePoint::WithdrawalAfterPoolDebit,
        MockFailurePoint::WithdrawalBeforeCommit,
    ] {
        let mut injected = mock_pool();
        injected.set_failure(failure);
        let request = withdrawal_request(&injected, withdrawal_id(10, 0), 2, 100);
        assert_withdrawal_rejected_unchanged(
            &mut injected,
            request,
            StakePoolError::InjectedMockFailure,
        );
    }
}

fn initiate_with_terms(
    cooldown_epochs: u64,
    reward: u64,
    loss: u64,
) -> (MockStakePool, WithdrawalId) {
    let mut pool = mock_pool();
    pool.set_source_finalization_terms(
        WithdrawalSourceId(1),
        cooldown_epochs,
        20,
        10,
        reward,
        loss,
    )
    .expect("set finalization terms");
    let id = withdrawal_id(11, 0);
    let request = withdrawal_request(&pool, id, 1, 200);
    pool.initiate_protected_stake_withdrawal(request)
        .expect("initiate delayed withdrawal");
    (pool, id)
}

#[test]
fn finalization_rejects_immediate_and_one_epoch_before_readiness() {
    let (mut pool, id) = initiate_with_terms(3, 0, 0);
    let initiation = pool.delayed_withdrawal(id).expect("known withdrawal");
    assert_eq!(initiation.status, DelayedWithdrawalStatus::Deactivating);
    assert_finalization_rejected_unchanged(
        &mut pool,
        id,
        StakePoolError::WithdrawalNotInactive,
    );

    pool.advance_epoch_to(initiation.first_eligible_finalization_epoch - 1)
        .expect("advance one before readiness");
    assert_eq!(
        pool.delayed_withdrawal(id).expect("known withdrawal").status,
        DelayedWithdrawalStatus::Deactivating
    );
    assert_finalization_rejected_unchanged(
        &mut pool,
        id,
        StakePoolError::WithdrawalNotInactive,
    );
}

#[test]
fn exact_and_later_eligible_epochs_finalize_and_replay_is_rejected() {
    for extra_epochs in [0, 5] {
        let (mut pool, id) = initiate_with_terms(2, 0, 0);
        let eligible = pool
            .delayed_withdrawal(id)
            .expect("known withdrawal")
            .first_eligible_finalization_epoch;
        pool.advance_epoch_to(eligible + extra_epochs)
            .expect("advance to eligible epoch");
        assert_eq!(
            pool.delayed_withdrawal(id).expect("inactive withdrawal").status,
            DelayedWithdrawalStatus::Inactive
        );
        let result = pool
            .finalize_delayed_stake_withdrawal(FinalizeWithdrawalRequest {
                withdrawal_id: id,
            })
            .expect("eligible finalization");
        assert_eq!(result.finalized_epoch, eligible + extra_epochs);
        assert_eq!(result.status, DelayedWithdrawalStatus::Finalized);
        assert_eq!(result.finalized_native_lamports, 220);
        assert_eq!(pool.operational_rent_lamports(), 10_000);
        assert_eq!(pool.validate_conservation(), Ok(()));
        assert_finalization_rejected_unchanged(
            &mut pool,
            id,
            StakePoolError::AlreadyFinalized,
        );
    }
}

#[test]
fn independently_delayed_legs_finalize_out_of_order() {
    let mut pool = mock_pool();
    pool.set_source_finalization_terms(WithdrawalSourceId(1), 3, 20, 10, 0, 0)
        .expect("slow source");
    pool.set_source_finalization_terms(WithdrawalSourceId(2), 1, 20, 10, 0, 0)
        .expect("fast source");
    let slow_id = withdrawal_id(12, 0);
    let fast_id = withdrawal_id(12, 1);
    pool.initiate_protected_stake_withdrawal(withdrawal_request(
        &pool, slow_id, 1, 200,
    ))
    .expect("initiate slow leg");
    pool.initiate_protected_stake_withdrawal(withdrawal_request(
        &pool, fast_id, 2, 200,
    ))
    .expect("initiate fast leg");

    pool.advance_epoch_to(41).expect("advance fast leg");
    assert_eq!(
        pool.delayed_withdrawal(fast_id).expect("fast status").status,
        DelayedWithdrawalStatus::Inactive
    );
    assert_eq!(
        pool.delayed_withdrawal(slow_id).expect("slow status").status,
        DelayedWithdrawalStatus::Deactivating
    );
    pool.finalize_delayed_stake_withdrawal(FinalizeWithdrawalRequest {
        withdrawal_id: fast_id,
    })
    .expect("finalize later-index leg first");

    pool.advance_epoch_to(43).expect("advance slow leg");
    pool.finalize_delayed_stake_withdrawal(FinalizeWithdrawalRequest {
        withdrawal_id: slow_id,
    })
    .expect("finalize earlier-index leg second");
    assert_eq!(pool.validate_conservation(), Ok(()));
}

#[test]
fn finalization_categories_reward_loss_and_both_rents_exactly() {
    let (mut reward_pool, reward_id) = initiate_with_terms(1, 7, 0);
    reward_pool.advance_epoch_to(41).expect("reward readiness");
    let reward = reward_pool
        .finalize_delayed_stake_withdrawal(FinalizeWithdrawalRequest {
            withdrawal_id: reward_id,
        })
        .expect("reward finalization");
    assert_eq!(reward.delegated_native_lamports, 200);
    assert_eq!(reward.cooldown_reward_lamports, 7);
    assert_eq!(reward.cooldown_loss_lamports, 0);
    assert_eq!(reward.recovered_stake_rent_lamports, 20);
    assert_eq!(reward.recovered_metadata_rent_lamports, 10);
    assert_eq!(reward.finalized_native_lamports, 227);
    assert_eq!(reward_pool.audit().cooldown_reward_lamports, 7);

    let (mut loss_pool, loss_id) = initiate_with_terms(1, 0, 9);
    loss_pool.advance_epoch_to(41).expect("loss readiness");
    let loss = loss_pool
        .finalize_delayed_stake_withdrawal(FinalizeWithdrawalRequest {
            withdrawal_id: loss_id,
        })
        .expect("loss finalization");
    assert_eq!(loss.cooldown_reward_lamports, 0);
    assert_eq!(loss.cooldown_loss_lamports, 9);
    assert_eq!(loss.finalized_native_lamports, 211);
    assert_eq!(loss_pool.audit().cooldown_loss_lamports, 9);
    assert_eq!(loss_pool.operational_rent_lamports(), 10_000);
    assert_eq!(loss_pool.validate_conservation(), Ok(()));
}

#[test]
fn finalization_unknown_status_injection_and_overlarge_loss_are_atomic() {
    let mut unknown = mock_pool();
    assert_finalization_rejected_unchanged(
        &mut unknown,
        withdrawal_id(99, 99),
        StakePoolError::UnknownWithdrawalIdentifier,
    );

    let (mut initiated_status, id) = initiate_with_terms(1, 0, 0);
    initiated_status
        .force_withdrawal_status(id, DelayedWithdrawalStatus::Initiated)
        .expect("force initiated status");
    initiated_status.advance_epoch_to(41).expect("advance epoch");
    assert_finalization_rejected_unchanged(
        &mut initiated_status,
        id,
        StakePoolError::WithdrawalNotInactive,
    );

    let (mut loss_pool, loss_id) = initiate_with_terms(1, 0, 1_000);
    loss_pool.advance_epoch_to(41).expect("advance loss pool");
    assert_finalization_rejected_unchanged(
        &mut loss_pool,
        loss_id,
        StakePoolError::ArithmeticOverflow,
    );

    for failure in [
        MockFailurePoint::FinalizationBeforeValidation,
        MockFailurePoint::FinalizationAfterReadiness,
        MockFailurePoint::FinalizationAfterAccounting,
        MockFailurePoint::FinalizationBeforeCommit,
    ] {
        let (mut injected, injected_id) = initiate_with_terms(1, 0, 0);
        injected.advance_epoch_to(41).expect("advance injected pool");
        injected.set_failure(failure);
        assert_finalization_rejected_unchanged(
            &mut injected,
            injected_id,
            StakePoolError::InjectedMockFailure,
        );
    }

    let (mut status_failure, status_id) = initiate_with_terms(1, 0, 0);
    status_failure.set_failure(MockFailurePoint::StatusRead);
    let before = status_failure.clone();
    assert_eq!(
        status_failure.delayed_withdrawal(status_id),
        Err(StakePoolError::InjectedMockFailure)
    );
    assert_eq!(status_failure, before);
}

#[test]
fn deterministic_randomized_actions_match_independent_reference_math() {
    let mut rng = SplitMix64::new(RANDOM_SEED);
    for case in 0..RANDOM_CASES {
        let total = 10_000 + rng.bounded(990_001);
        let supply = 10_000 + rng.bounded(990_001);
        let deposit_input = 1_000 + rng.bounded(5_000);
        let deposit_fee = if case % 2 == 0 {
            FeeFraction::ZERO
        } else {
            FeeFraction {
                numerator: 1 + rng.bounded(9),
                denominator: 1_000,
            }
        };
        let withdrawal_fee = if case % 3 == 0 {
            FeeFraction::ZERO
        } else {
            FeeFraction {
                numerator: 1 + rng.bounded(9),
                denominator: 1_000,
            }
        };
        let random_snapshot = PoolSnapshot {
            total_pool_lamports: total,
            pool_token_supply: supply,
            sol_deposit_fee: deposit_fee,
            stake_withdrawal_fee: withdrawal_fee,
            minimum_delegation_lamports: 1,
            maximum_deposit_lamports: u64::MAX,
            available_withdrawal_lamports: total,
            ..snapshot()
        };
        let withdrawal_target = 1_000 + rng.bounded(4_000);
        let mut random_sources = [MockWithdrawalSource::VACANT; MAX_MOCK_WITHDRAWAL_SOURCES];
        random_sources[0] = MockWithdrawalSource::new(1, withdrawal_target);
        let mut pool = MockStakePool::new(random_snapshot, random_sources, 100)
            .unwrap_or_else(|error| {
                panic!("seed={RANDOM_SEED:#018x}, case={case}, construct={error:?}")
            });

        let deposit = pool
            .execute_protected_sol_deposit(deposit_request(&pool, deposit_input))
            .unwrap_or_else(|error| {
                panic!("seed={RANDOM_SEED:#018x}, case={case}, deposit={error:?}")
            });
        let reference_gross =
            (u128::from(deposit_input) * u128::from(supply) / u128::from(total)) as u64;
        let reference_deposit_fee = reference_fee_ceil(reference_gross, deposit_fee);
        assert_eq!(
            deposit.quote.gross_pool_tokens, reference_gross,
            "seed={RANDOM_SEED:#018x}, case={case}, deposit gross"
        );
        assert_eq!(
            deposit.quote.deposit_fee_pool_tokens, reference_deposit_fee,
            "seed={RANDOM_SEED:#018x}, case={case}, deposit fee"
        );
        assert_eq!(
            deposit.actual_pool_tokens_out,
            reference_gross - reference_deposit_fee,
            "seed={RANDOM_SEED:#018x}, case={case}, deposit output"
        );

        let current = pool.raw_snapshot();
        let request = withdrawal_request(
            &pool,
            withdrawal_id(case as u64, 0),
            1,
            withdrawal_target,
        );
        let initiation = pool
            .initiate_protected_stake_withdrawal(request)
            .unwrap_or_else(|error| {
                panic!("seed={RANDOM_SEED:#018x}, case={case}, withdrawal={error:?}")
            });
        let reference_withdrawal_fee = reference_fee_ceil(withdrawal_target, withdrawal_fee);
        let reference_burn = withdrawal_target - reference_withdrawal_fee;
        let reference_native = (u128::from(reference_burn)
            * u128::from(current.total_pool_lamports)
            / u128::from(current.pool_token_supply)) as u64;
        assert_eq!(
            initiation.quote.withdrawal_fee_pool_tokens,
            reference_withdrawal_fee,
            "seed={RANDOM_SEED:#018x}, case={case}, withdrawal fee"
        );
        assert_eq!(
            initiation.quote.burned_pool_tokens, reference_burn,
            "seed={RANDOM_SEED:#018x}, case={case}, withdrawal burn"
        );
        assert_eq!(
            initiation.actual_delegated_native_lamports, reference_native,
            "seed={RANDOM_SEED:#018x}, case={case}, delegated output"
        );

        pool.advance_epoch_to(41).unwrap_or_else(|error| {
            panic!("seed={RANDOM_SEED:#018x}, case={case}, epoch={error:?}")
        });
        let finalization = pool
            .finalize_delayed_stake_withdrawal(FinalizeWithdrawalRequest {
                withdrawal_id: withdrawal_id(case as u64, 0),
            })
            .unwrap_or_else(|error| {
                panic!("seed={RANDOM_SEED:#018x}, case={case}, finalization={error:?}")
            });
        assert_eq!(
            finalization.finalized_native_lamports,
            reference_native + 20,
            "seed={RANDOM_SEED:#018x}, case={case}, finalized value"
        );
        pool.validate_conservation().unwrap_or_else(|error| {
            panic!("seed={RANDOM_SEED:#018x}, case={case}, conservation={error:?}")
        });
    }
}

fn reference_fee_ceil(amount: u64, fee: FeeFraction) -> u64 {
    if fee.numerator == 0 {
        return 0;
    }
    let product = u128::from(amount) * u128::from(fee.numerator);
    let denominator = u128::from(fee.denominator);
    let quotient = product / denominator;
    let rounded = quotient + u128::from(product % denominator != 0);
    rounded as u64
}
