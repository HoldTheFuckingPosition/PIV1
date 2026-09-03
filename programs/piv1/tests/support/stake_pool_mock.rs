#![allow(dead_code)]

use core::cmp::{max, min};

use piv1::integrations::{
    DelayedWithdrawal, DelayedWithdrawalStatus, FeeFraction,
    FinalizeWithdrawalRequest, PoolSnapshot, PoolSnapshotIdentity,
    SolDepositExecution, SolDepositQuote, SolDepositRequest,
    StakePoolAdapter, StakePoolError, StakePoolResult,
    StakeWithdrawalFinalization, StakeWithdrawalInitiation,
    StakeWithdrawalQuote, StakeWithdrawalRequest, WithdrawalId,
    WithdrawalSourceId, MAX_PROTECTED_SLIPPAGE_BPS,
    SLIPPAGE_BASIS_POINTS_DENOMINATOR,
};

pub const MAX_MOCK_WITHDRAWAL_SOURCES: usize = 8;
pub const MAX_MOCK_WITHDRAWALS: usize = 16;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct MockWithdrawalSource {
    pub is_configured: bool,
    pub id: WithdrawalSourceId,
    pub capacity_pool_tokens: u64,
    pub cooldown_epochs: u64,
    pub stake_rent_lamports: u64,
    pub metadata_rent_lamports: u64,
    pub cooldown_reward_lamports: u64,
    pub cooldown_loss_lamports: u64,
}

impl MockWithdrawalSource {
    pub const VACANT: Self = Self {
        is_configured: false,
        id: WithdrawalSourceId(0),
        capacity_pool_tokens: 0,
        cooldown_epochs: 0,
        stake_rent_lamports: 0,
        metadata_rent_lamports: 0,
        cooldown_reward_lamports: 0,
        cooldown_loss_lamports: 0,
    };

    pub const fn new(id: u32, capacity_pool_tokens: u64) -> Self {
        Self {
            is_configured: true,
            id: WithdrawalSourceId(id),
            capacity_pool_tokens,
            cooldown_epochs: 1,
            stake_rent_lamports: 20,
            metadata_rent_lamports: 10,
            cooldown_reward_lamports: 0,
            cooldown_loss_lamports: 0,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum MockFailurePoint {
    SnapshotRead,
    DepositBeforeValidation,
    DepositAfterQuote,
    DepositBeforeCommit,
    WithdrawalBeforeValidation,
    WithdrawalAfterQuote,
    WithdrawalAfterPoolDebit,
    WithdrawalBeforeCommit,
    StatusRead,
    FinalizationBeforeValidation,
    FinalizationAfterReadiness,
    FinalizationAfterAccounting,
    FinalizationBeforeCommit,
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct MockAudit {
    pub external_pool_rewards_lamports: u64,
    pub external_pool_losses_lamports: u64,
    pub deposited_native_lamports: u64,
    pub deposited_user_pool_tokens: u64,
    pub deposit_fee_pool_tokens: u64,
    pub withdrawal_input_pool_tokens: u64,
    pub withdrawal_fee_pool_tokens: u64,
    pub burned_pool_tokens: u64,
    pub delegated_native_lamports: u64,
    pub finalized_native_lamports: u64,
    pub rent_advanced_lamports: u64,
    pub recovered_stake_rent_lamports: u64,
    pub recovered_metadata_rent_lamports: u64,
    pub cooldown_reward_lamports: u64,
    pub cooldown_loss_lamports: u64,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct MockWithdrawalRecord {
    is_occupied: bool,
    initiation: StakeWithdrawalInitiation,
    configured_reward_lamports: u64,
    configured_loss_lamports: u64,
    status: DelayedWithdrawalStatus,
    finalization: Option<StakeWithdrawalFinalization>,
}

impl MockWithdrawalRecord {
    const VACANT: Self = Self {
        is_occupied: false,
        initiation: StakeWithdrawalInitiation {
            quote: StakeWithdrawalQuote {
                snapshot: PoolSnapshotIdentity {
                    current_epoch: 0,
                    last_update_epoch: 0,
                    revision: 0,
                },
                withdrawal_id: WithdrawalId {
                    sequence: 0,
                    leg_index: 0,
                },
                source_id: WithdrawalSourceId(0),
                remaining_pool_token_target: 0,
                source_capacity_pool_tokens: 0,
                technical_minimum_pool_tokens: 0,
                pool_tokens_in: 0,
                withdrawal_fee_pool_tokens: 0,
                burned_pool_tokens: 0,
                expected_delegated_native_lamports: 0,
                derived_slippage_floor_native_lamports: 0,
                minimum_native_lamports_out: 0,
            },
            actual_delegated_native_lamports: 0,
            initiation_epoch: 0,
            deactivation_epoch: 0,
            first_eligible_finalization_epoch: 0,
            status: DelayedWithdrawalStatus::Initiated,
            stake_rent_lamports: 0,
            metadata_rent_lamports: 0,
        },
        configured_reward_lamports: 0,
        configured_loss_lamports: 0,
        status: DelayedWithdrawalStatus::Initiated,
        finalization: None,
    };
}

/// Fixed-capacity, host-only stake-pool simulator used by Task 2.1 tests.
///
/// Every fallible operation stages a complete clone, validates it, and commits
/// once. The type lives under the integration-test tree and is never exported
/// by the production library.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MockStakePool {
    snapshot: PoolSnapshot,
    initial_total_pool_lamports: u64,
    initial_pool_token_supply: u64,
    initial_operational_rent_lamports: u64,
    operational_rent_lamports: u64,
    sources: [MockWithdrawalSource; MAX_MOCK_WITHDRAWAL_SOURCES],
    initial_source_capacities: [u64; MAX_MOCK_WITHDRAWAL_SOURCES],
    withdrawals: [MockWithdrawalRecord; MAX_MOCK_WITHDRAWALS],
    withdrawal_count: u8,
    audit: MockAudit,
    failure: Option<MockFailurePoint>,
}

impl MockStakePool {
    pub fn new(
        snapshot: PoolSnapshot,
        sources: [MockWithdrawalSource; MAX_MOCK_WITHDRAWAL_SOURCES],
        operational_rent_lamports: u64,
    ) -> StakePoolResult<Self> {
        snapshot.validate()?;
        validate_sources(&sources)?;
        let initial_source_capacities =
            core::array::from_fn(|index| sources[index].capacity_pool_tokens);
        let pool = Self {
            snapshot,
            initial_total_pool_lamports: snapshot.total_pool_lamports,
            initial_pool_token_supply: snapshot.pool_token_supply,
            initial_operational_rent_lamports: operational_rent_lamports,
            operational_rent_lamports,
            sources,
            initial_source_capacities,
            withdrawals: [MockWithdrawalRecord::VACANT; MAX_MOCK_WITHDRAWALS],
            withdrawal_count: 0,
            audit: MockAudit::default(),
            failure: None,
        };
        pool.validate_conservation()?;
        Ok(pool)
    }

    pub const fn raw_snapshot(&self) -> PoolSnapshot {
        self.snapshot
    }

    pub const fn audit(&self) -> MockAudit {
        self.audit
    }

    pub const fn operational_rent_lamports(&self) -> u64 {
        self.operational_rent_lamports
    }

    pub const fn withdrawal_count(&self) -> u8 {
        self.withdrawal_count
    }

    pub fn source_capacity(&self, source_id: WithdrawalSourceId) -> StakePoolResult<u64> {
        Ok(self.source(source_id)?.capacity_pool_tokens)
    }

    pub fn set_failure(&mut self, failure: MockFailurePoint) {
        self.failure = Some(failure);
    }

    pub fn clear_failure(&mut self) {
        self.failure = None;
    }

    pub fn set_last_update_epoch(&mut self, epoch: u64) -> StakePoolResult<()> {
        let mut next = self.clone();
        next.snapshot.last_update_epoch = epoch;
        next.bump_revision()?;
        *self = next;
        Ok(())
    }

    pub fn set_current_epoch(&mut self, epoch: u64) -> StakePoolResult<()> {
        let mut next = self.clone();
        next.snapshot.current_epoch = epoch;
        next.bump_revision()?;
        *self = next;
        Ok(())
    }

    pub fn advance_epoch_to(&mut self, epoch: u64) -> StakePoolResult<()> {
        if epoch < self.snapshot.current_epoch {
            return Err(StakePoolError::InvalidConfiguration);
        }
        let mut next = self.clone();
        next.snapshot.current_epoch = epoch;
        next.bump_revision()?;
        *self = next;
        Ok(())
    }

    pub fn refresh_pool(&mut self) -> StakePoolResult<()> {
        let mut next = self.clone();
        next.snapshot.last_update_epoch = next.snapshot.current_epoch;
        next.bump_revision()?;
        next.snapshot.validate()?;
        *self = next;
        Ok(())
    }

    pub fn set_fees(
        &mut self,
        sol_deposit_fee: FeeFraction,
        stake_withdrawal_fee: FeeFraction,
    ) -> StakePoolResult<()> {
        sol_deposit_fee.validate()?;
        stake_withdrawal_fee.validate()?;
        let mut next = self.clone();
        next.snapshot.sol_deposit_fee = sol_deposit_fee;
        next.snapshot.stake_withdrawal_fee = stake_withdrawal_fee;
        next.bump_revision()?;
        *self = next;
        Ok(())
    }

    pub fn set_available_withdrawal_lamports(
        &mut self,
        available_lamports: u64,
    ) -> StakePoolResult<()> {
        if available_lamports > self.snapshot.total_pool_lamports {
            return Err(StakePoolError::InvalidConfiguration);
        }
        let mut next = self.clone();
        next.snapshot.available_withdrawal_lamports = available_lamports;
        next.bump_revision()?;
        *self = next;
        Ok(())
    }

    pub fn set_maximum_deposit_lamports(&mut self, capacity: u64) -> StakePoolResult<()> {
        let mut next = self.clone();
        next.snapshot.maximum_deposit_lamports = capacity;
        next.bump_revision()?;
        *self = next;
        Ok(())
    }

    pub fn set_minimum_delegation_lamports(
        &mut self,
        minimum_lamports: u64,
    ) -> StakePoolResult<()> {
        if minimum_lamports == 0 {
            return Err(StakePoolError::InvalidConfiguration);
        }
        let mut next = self.clone();
        next.snapshot.minimum_delegation_lamports = minimum_lamports;
        next.bump_revision()?;
        next.snapshot.validate()?;
        *self = next;
        Ok(())
    }

    pub fn set_source_capacity(
        &mut self,
        source_id: WithdrawalSourceId,
        capacity_pool_tokens: u64,
    ) -> StakePoolResult<()> {
        let mut next = self.clone();
        let index = next.source_index(source_id)?;
        let consumed = next.initial_source_capacities[index]
            .checked_sub(next.sources[index].capacity_pool_tokens)
            .ok_or(StakePoolError::InvalidConfiguration)?;
        next.sources[index].capacity_pool_tokens = capacity_pool_tokens;
        next.initial_source_capacities[index] = capacity_pool_tokens
            .checked_add(consumed)
            .ok_or(StakePoolError::ArithmeticOverflow)?;
        next.bump_revision()?;
        next.validate_conservation()?;
        *self = next;
        Ok(())
    }

    pub fn set_source_finalization_terms(
        &mut self,
        source_id: WithdrawalSourceId,
        cooldown_epochs: u64,
        stake_rent_lamports: u64,
        metadata_rent_lamports: u64,
        cooldown_reward_lamports: u64,
        cooldown_loss_lamports: u64,
    ) -> StakePoolResult<()> {
        if cooldown_epochs == 0
            || (cooldown_reward_lamports > 0 && cooldown_loss_lamports > 0)
        {
            return Err(StakePoolError::InvalidConfiguration);
        }
        let mut next = self.clone();
        let source = next.source_mut(source_id)?;
        source.cooldown_epochs = cooldown_epochs;
        source.stake_rent_lamports = stake_rent_lamports;
        source.metadata_rent_lamports = metadata_rent_lamports;
        source.cooldown_reward_lamports = cooldown_reward_lamports;
        source.cooldown_loss_lamports = cooldown_loss_lamports;
        next.bump_revision()?;
        *self = next;
        Ok(())
    }

    pub fn increase_exchange_rate(&mut self, reward_lamports: u64) -> StakePoolResult<()> {
        let mut next = self.clone();
        next.snapshot.total_pool_lamports = checked_add(
            next.snapshot.total_pool_lamports,
            reward_lamports,
        )?;
        next.snapshot.available_withdrawal_lamports = checked_add(
            next.snapshot.available_withdrawal_lamports,
            reward_lamports,
        )?;
        next.audit.external_pool_rewards_lamports = checked_add(
            next.audit.external_pool_rewards_lamports,
            reward_lamports,
        )?;
        next.bump_revision()?;
        next.validate_conservation()?;
        *self = next;
        Ok(())
    }

    pub fn decrease_exchange_rate(&mut self, loss_lamports: u64) -> StakePoolResult<()> {
        let mut next = self.clone();
        next.snapshot.total_pool_lamports = next
            .snapshot
            .total_pool_lamports
            .checked_sub(loss_lamports)
            .ok_or(StakePoolError::InvalidConfiguration)?;
        next.snapshot.available_withdrawal_lamports = min(
            next.snapshot.available_withdrawal_lamports,
            next.snapshot.total_pool_lamports,
        );
        next.audit.external_pool_losses_lamports = checked_add(
            next.audit.external_pool_losses_lamports,
            loss_lamports,
        )?;
        next.bump_revision()?;
        next.validate_conservation()?;
        *self = next;
        Ok(())
    }

    pub fn force_withdrawal_status(
        &mut self,
        withdrawal_id: WithdrawalId,
        status: DelayedWithdrawalStatus,
    ) -> StakePoolResult<()> {
        let mut next = self.clone();
        let index = next.withdrawal_index(withdrawal_id)?;
        if next.withdrawals[index].status == DelayedWithdrawalStatus::Finalized
            || status == DelayedWithdrawalStatus::Finalized
        {
            return Err(StakePoolError::InvalidConfiguration);
        }
        next.withdrawals[index].status = status;
        *self = next;
        Ok(())
    }

    pub fn validate_conservation(&self) -> StakePoolResult<()> {
        let expected_total = checked_add_u128(
            checked_add_u128(
                u128::from(self.initial_total_pool_lamports),
                u128::from(self.audit.deposited_native_lamports),
            )?,
            u128::from(self.audit.external_pool_rewards_lamports),
        )?;
        let observed_total = checked_add_u128(
            checked_add_u128(
                u128::from(self.snapshot.total_pool_lamports),
                u128::from(self.audit.delegated_native_lamports),
            )?,
            u128::from(self.audit.external_pool_losses_lamports),
        )?;
        if expected_total != observed_total {
            return Err(StakePoolError::InvalidConfiguration);
        }

        let expected_supply = checked_add_u128(
            checked_add_u128(
                u128::from(self.initial_pool_token_supply),
                u128::from(self.audit.deposited_user_pool_tokens),
            )?,
            u128::from(self.audit.deposit_fee_pool_tokens),
        )?;
        let observed_supply = checked_add_u128(
            u128::from(self.snapshot.pool_token_supply),
            u128::from(self.audit.burned_pool_tokens),
        )?;
        if expected_supply != observed_supply
            || checked_add(
                self.audit.withdrawal_fee_pool_tokens,
                self.audit.burned_pool_tokens,
            )? != self.audit.withdrawal_input_pool_tokens
        {
            return Err(StakePoolError::InvalidConfiguration);
        }

        let expected_operational = checked_add_u128(
            u128::from(self.initial_operational_rent_lamports),
            checked_add_u128(
                u128::from(self.audit.recovered_stake_rent_lamports),
                u128::from(self.audit.recovered_metadata_rent_lamports),
            )?,
        )?;
        let observed_operational = checked_add_u128(
            u128::from(self.operational_rent_lamports),
            u128::from(self.audit.rent_advanced_lamports),
        )?;
        if expected_operational != observed_operational {
            return Err(StakePoolError::InvalidConfiguration);
        }

        for index in 0..MAX_MOCK_WITHDRAWAL_SOURCES {
            if !self.sources[index].is_configured {
                continue;
            }
            let consumed = self.withdrawals.iter().try_fold(0_u64, |total, record| {
                if record.is_occupied && record.initiation.quote.source_id == self.sources[index].id {
                    checked_add(total, record.initiation.quote.pool_tokens_in)
                } else {
                    Ok(total)
                }
            })?;
            if checked_add(self.sources[index].capacity_pool_tokens, consumed)?
                != self.initial_source_capacities[index]
            {
                return Err(StakePoolError::InvalidConfiguration);
            }
        }

        let mut recorded_withdrawal_input = 0_u64;
        let mut recorded_withdrawal_fees = 0_u64;
        let mut recorded_burned = 0_u64;
        let mut recorded_delegated = 0_u64;
        let mut recorded_rent_advanced = 0_u64;
        let mut recorded_finalized_native = 0_u64;
        let mut recorded_recovered_stake_rent = 0_u64;
        let mut recorded_recovered_metadata_rent = 0_u64;
        let mut recorded_cooldown_rewards = 0_u64;
        let mut recorded_cooldown_losses = 0_u64;
        let mut occupied_count = 0_usize;

        for record in self.withdrawals.iter().filter(|record| record.is_occupied) {
            occupied_count = occupied_count
                .checked_add(1)
                .ok_or(StakePoolError::ArithmeticOverflow)?;
            recorded_withdrawal_input = checked_add(
                recorded_withdrawal_input,
                record.initiation.quote.pool_tokens_in,
            )?;
            recorded_withdrawal_fees = checked_add(
                recorded_withdrawal_fees,
                record.initiation.quote.withdrawal_fee_pool_tokens,
            )?;
            recorded_burned = checked_add(
                recorded_burned,
                record.initiation.quote.burned_pool_tokens,
            )?;
            recorded_delegated = checked_add(
                recorded_delegated,
                record.initiation.actual_delegated_native_lamports,
            )?;
            recorded_rent_advanced = checked_add(
                recorded_rent_advanced,
                checked_add(
                    record.initiation.stake_rent_lamports,
                    record.initiation.metadata_rent_lamports,
                )?,
            )?;

            match record.finalization {
                Some(finalization) => {
                    let final_left = checked_add(
                        checked_add(
                            record.initiation.actual_delegated_native_lamports,
                            record.initiation.stake_rent_lamports,
                        )?,
                        record.configured_reward_lamports,
                    )?;
                    let final_right = checked_add(
                        finalization.finalized_native_lamports,
                        record.configured_loss_lamports,
                    )?;
                    if record.status != DelayedWithdrawalStatus::Finalized
                        || finalization.status != DelayedWithdrawalStatus::Finalized
                        || finalization.withdrawal_id
                            != record.initiation.quote.withdrawal_id
                        || finalization.delegated_native_lamports
                            != record.initiation.actual_delegated_native_lamports
                        || finalization.recovered_stake_rent_lamports
                            != record.initiation.stake_rent_lamports
                        || finalization.recovered_metadata_rent_lamports
                            != record.initiation.metadata_rent_lamports
                        || finalization.cooldown_reward_lamports
                            != record.configured_reward_lamports
                        || finalization.cooldown_loss_lamports
                            != record.configured_loss_lamports
                        || final_left != final_right
                    {
                        return Err(StakePoolError::InvalidConfiguration);
                    }
                    recorded_finalized_native = checked_add(
                        recorded_finalized_native,
                        finalization.finalized_native_lamports,
                    )?;
                    recorded_recovered_stake_rent = checked_add(
                        recorded_recovered_stake_rent,
                        finalization.recovered_stake_rent_lamports,
                    )?;
                    recorded_recovered_metadata_rent = checked_add(
                        recorded_recovered_metadata_rent,
                        finalization.recovered_metadata_rent_lamports,
                    )?;
                    recorded_cooldown_rewards = checked_add(
                        recorded_cooldown_rewards,
                        finalization.cooldown_reward_lamports,
                    )?;
                    recorded_cooldown_losses = checked_add(
                        recorded_cooldown_losses,
                        finalization.cooldown_loss_lamports,
                    )?;
                }
                None if record.status == DelayedWithdrawalStatus::Finalized => {
                    return Err(StakePoolError::InvalidConfiguration);
                }
                None => {}
            }
        }

        if usize::from(self.withdrawal_count) != occupied_count
            || recorded_withdrawal_input != self.audit.withdrawal_input_pool_tokens
            || recorded_withdrawal_fees != self.audit.withdrawal_fee_pool_tokens
            || recorded_burned != self.audit.burned_pool_tokens
            || recorded_delegated != self.audit.delegated_native_lamports
            || recorded_rent_advanced != self.audit.rent_advanced_lamports
            || recorded_finalized_native != self.audit.finalized_native_lamports
            || recorded_recovered_stake_rent
                != self.audit.recovered_stake_rent_lamports
            || recorded_recovered_metadata_rent
                != self.audit.recovered_metadata_rent_lamports
            || recorded_cooldown_rewards != self.audit.cooldown_reward_lamports
            || recorded_cooldown_losses != self.audit.cooldown_loss_lamports
        {
            return Err(StakePoolError::InvalidConfiguration);
        }
        Ok(())
    }

    fn quote_deposit_inner(&self, request: SolDepositRequest) -> StakePoolResult<SolDepositQuote> {
        if self.should_fail(MockFailurePoint::DepositBeforeValidation) {
            return Err(StakePoolError::InjectedMockFailure);
        }
        validate_slippage(request.slippage_bps)?;
        if request.native_lamports == 0 {
            return Err(StakePoolError::ZeroInput);
        }
        self.snapshot.validate()?;
        self.validate_identity(request.snapshot)?;
        if request.native_lamports > self.snapshot.maximum_deposit_lamports {
            return Err(StakePoolError::InsufficientPoolLiquidity);
        }

        let gross_pool_tokens = if self.snapshot.is_bootstrap() {
            request.native_lamports
        } else {
            checked_mul_div_floor(
                request.native_lamports,
                self.snapshot.pool_token_supply,
                self.snapshot.total_pool_lamports,
            )?
        };
        let deposit_fee_pool_tokens = fee_ceil(gross_pool_tokens, self.snapshot.sol_deposit_fee)?;
        let quoted_pool_tokens_out = gross_pool_tokens
            .checked_sub(deposit_fee_pool_tokens)
            .ok_or(StakePoolError::InvalidFee)?;
        if quoted_pool_tokens_out == 0 {
            return Err(StakePoolError::InsufficientPoolLiquidity);
        }
        let derived_slippage_floor_pool_tokens =
            slippage_floor(quoted_pool_tokens_out, request.slippage_bps)?;
        let minimum_pool_tokens_out = max(
            request.caller_minimum_pool_tokens_out,
            derived_slippage_floor_pool_tokens,
        );
        if quoted_pool_tokens_out < minimum_pool_tokens_out {
            return Err(StakePoolError::SlippageExceeded);
        }

        let quote = SolDepositQuote {
            snapshot: self.snapshot.identity(),
            native_lamports: request.native_lamports,
            gross_pool_tokens,
            deposit_fee_pool_tokens,
            quoted_pool_tokens_out,
            derived_slippage_floor_pool_tokens,
            minimum_pool_tokens_out,
        };
        if self.should_fail(MockFailurePoint::DepositAfterQuote) {
            return Err(StakePoolError::InjectedMockFailure);
        }
        Ok(quote)
    }

    fn quote_withdrawal_inner(
        &self,
        request: StakeWithdrawalRequest,
    ) -> StakePoolResult<StakeWithdrawalQuote> {
        if self.should_fail(MockFailurePoint::WithdrawalBeforeValidation) {
            return Err(StakePoolError::InjectedMockFailure);
        }
        validate_slippage(request.slippage_bps)?;
        if request.remaining_pool_token_target == 0 {
            return Err(StakePoolError::ZeroInput);
        }
        self.snapshot.validate()?;
        self.validate_identity(request.snapshot)?;
        if self.find_withdrawal(request.withdrawal_id).is_some() {
            return Err(StakePoolError::IdentifierReuse);
        }
        let source = *self.source(request.source_id)?;
        let technical_minimum_pool_tokens = self.technical_minimum_pool_tokens()?;
        if request.remaining_pool_token_target < technical_minimum_pool_tokens {
            return Err(StakePoolError::TechnicalMinimumNotMet);
        }
        if source.capacity_pool_tokens < technical_minimum_pool_tokens {
            return Err(StakePoolError::InsufficientSourceCapacity);
        }

        let pool_tokens_in = min(
            request.remaining_pool_token_target,
            source.capacity_pool_tokens,
        );
        let remaining_after = request
            .remaining_pool_token_target
            .checked_sub(pool_tokens_in)
            .ok_or(StakePoolError::ArithmeticOverflow)?;
        if remaining_after != 0 && remaining_after < technical_minimum_pool_tokens {
            return Err(StakePoolError::InsufficientSourceCapacity);
        }
        if pool_tokens_in > self.snapshot.pool_token_supply {
            return Err(StakePoolError::InsufficientPoolLiquidity);
        }

        let withdrawal_fee_pool_tokens =
            fee_ceil(pool_tokens_in, self.snapshot.stake_withdrawal_fee)?;
        let burned_pool_tokens = pool_tokens_in
            .checked_sub(withdrawal_fee_pool_tokens)
            .ok_or(StakePoolError::InvalidFee)?;
        let expected_delegated_native_lamports = checked_mul_div_floor(
            burned_pool_tokens,
            self.snapshot.total_pool_lamports,
            self.snapshot.pool_token_supply,
        )?;
        if expected_delegated_native_lamports < self.snapshot.minimum_delegation_lamports {
            return Err(StakePoolError::TechnicalMinimumNotMet);
        }
        if expected_delegated_native_lamports > self.snapshot.available_withdrawal_lamports
            || expected_delegated_native_lamports > self.snapshot.total_pool_lamports
        {
            return Err(StakePoolError::InsufficientPoolLiquidity);
        }

        let derived_slippage_floor_native_lamports =
            slippage_floor(expected_delegated_native_lamports, request.slippage_bps)?;
        let minimum_native_lamports_out = max(
            self.snapshot.minimum_delegation_lamports,
            max(
                request.caller_minimum_native_lamports_out,
                derived_slippage_floor_native_lamports,
            ),
        );
        if expected_delegated_native_lamports < minimum_native_lamports_out {
            return Err(StakePoolError::SlippageExceeded);
        }

        let quote = StakeWithdrawalQuote {
            snapshot: self.snapshot.identity(),
            withdrawal_id: request.withdrawal_id,
            source_id: request.source_id,
            remaining_pool_token_target: request.remaining_pool_token_target,
            source_capacity_pool_tokens: source.capacity_pool_tokens,
            technical_minimum_pool_tokens,
            pool_tokens_in,
            withdrawal_fee_pool_tokens,
            burned_pool_tokens,
            expected_delegated_native_lamports,
            derived_slippage_floor_native_lamports,
            minimum_native_lamports_out,
        };
        if self.should_fail(MockFailurePoint::WithdrawalAfterQuote) {
            return Err(StakePoolError::InjectedMockFailure);
        }
        Ok(quote)
    }

    fn technical_minimum_pool_tokens(&self) -> StakePoolResult<u64> {
        if self.snapshot.is_bootstrap()
            || self.snapshot.pool_token_supply == 0
            || self.snapshot.available_withdrawal_lamports
                < self.snapshot.minimum_delegation_lamports
        {
            return Err(StakePoolError::InsufficientPoolLiquidity);
        }

        let maximum_output = self.expected_withdrawal_native(self.snapshot.pool_token_supply)?;
        if maximum_output < self.snapshot.minimum_delegation_lamports {
            return Err(StakePoolError::InsufficientPoolLiquidity);
        }

        let mut low = 1_u64;
        let mut high = self.snapshot.pool_token_supply;
        while low < high {
            let distance = high
                .checked_sub(low)
                .ok_or(StakePoolError::ArithmeticOverflow)?;
            let midpoint = low
                .checked_add(distance / 2)
                .ok_or(StakePoolError::ArithmeticOverflow)?;
            if self.expected_withdrawal_native(midpoint)?
                >= self.snapshot.minimum_delegation_lamports
            {
                high = midpoint;
            } else {
                low = midpoint
                    .checked_add(1)
                    .ok_or(StakePoolError::ArithmeticOverflow)?;
            }
        }
        Ok(low)
    }

    fn expected_withdrawal_native(&self, pool_tokens_in: u64) -> StakePoolResult<u64> {
        let fee = fee_ceil(pool_tokens_in, self.snapshot.stake_withdrawal_fee)?;
        let burned = pool_tokens_in
            .checked_sub(fee)
            .ok_or(StakePoolError::InvalidFee)?;
        checked_mul_div_floor(
            burned,
            self.snapshot.total_pool_lamports,
            self.snapshot.pool_token_supply,
        )
    }

    fn delayed_from_record(&self, record: &MockWithdrawalRecord) -> DelayedWithdrawal {
        let status = if record.status == DelayedWithdrawalStatus::Deactivating
            && self.snapshot.current_epoch
                >= record.initiation.first_eligible_finalization_epoch
        {
            DelayedWithdrawalStatus::Inactive
        } else {
            record.status
        };
        DelayedWithdrawal {
            withdrawal_id: record.initiation.quote.withdrawal_id,
            status,
            initiation_epoch: record.initiation.initiation_epoch,
            deactivation_epoch: record.initiation.deactivation_epoch,
            first_eligible_finalization_epoch: record
                .initiation
                .first_eligible_finalization_epoch,
            delegated_native_lamports: record.initiation.actual_delegated_native_lamports,
            stake_rent_lamports: record.initiation.stake_rent_lamports,
            metadata_rent_lamports: record.initiation.metadata_rent_lamports,
        }
    }

    fn validate_identity(&self, expected: PoolSnapshotIdentity) -> StakePoolResult<()> {
        if expected != self.snapshot.identity() {
            return Err(StakePoolError::StaleQuote);
        }
        Ok(())
    }

    fn source(&self, source_id: WithdrawalSourceId) -> StakePoolResult<&MockWithdrawalSource> {
        let index = self.source_index(source_id)?;
        Ok(&self.sources[index])
    }

    fn source_mut(
        &mut self,
        source_id: WithdrawalSourceId,
    ) -> StakePoolResult<&mut MockWithdrawalSource> {
        let index = self.source_index(source_id)?;
        Ok(&mut self.sources[index])
    }

    fn source_index(&self, source_id: WithdrawalSourceId) -> StakePoolResult<usize> {
        self.sources
            .iter()
            .position(|source| source.is_configured && source.id == source_id)
            .ok_or(StakePoolError::UnknownWithdrawalSource)
    }

    fn find_withdrawal(&self, withdrawal_id: WithdrawalId) -> Option<&MockWithdrawalRecord> {
        self.withdrawals.iter().find(|record| {
            record.is_occupied && record.initiation.quote.withdrawal_id == withdrawal_id
        })
    }

    fn withdrawal_index(&self, withdrawal_id: WithdrawalId) -> StakePoolResult<usize> {
        self.withdrawals
            .iter()
            .position(|record| {
                record.is_occupied && record.initiation.quote.withdrawal_id == withdrawal_id
            })
            .ok_or(StakePoolError::UnknownWithdrawalIdentifier)
    }

    fn vacant_withdrawal_index(&self) -> StakePoolResult<usize> {
        self.withdrawals
            .iter()
            .position(|record| !record.is_occupied)
            .ok_or(StakePoolError::InvalidConfiguration)
    }

    fn should_fail(&self, point: MockFailurePoint) -> bool {
        self.failure == Some(point)
    }

    fn bump_revision(&mut self) -> StakePoolResult<()> {
        self.snapshot.revision = self
            .snapshot
            .revision
            .checked_add(1)
            .ok_or(StakePoolError::ArithmeticOverflow)?;
        Ok(())
    }
}

impl StakePoolAdapter for MockStakePool {
    fn pool_snapshot(&self) -> StakePoolResult<PoolSnapshot> {
        if self.should_fail(MockFailurePoint::SnapshotRead) {
            return Err(StakePoolError::InjectedMockFailure);
        }
        self.snapshot.validate()?;
        Ok(self.snapshot)
    }

    fn quote_sol_deposit(&self, request: SolDepositRequest) -> StakePoolResult<SolDepositQuote> {
        self.quote_deposit_inner(request)
    }

    fn execute_protected_sol_deposit(
        &mut self,
        request: SolDepositRequest,
    ) -> StakePoolResult<SolDepositExecution> {
        let quote = self.quote_deposit_inner(request)?;
        let mut next = self.clone();
        next.snapshot.total_pool_lamports = checked_add(
            next.snapshot.total_pool_lamports,
            request.native_lamports,
        )?;
        next.snapshot.pool_token_supply = checked_add(
            next.snapshot.pool_token_supply,
            quote.gross_pool_tokens,
        )?;
        next.snapshot.available_withdrawal_lamports = checked_add(
            next.snapshot.available_withdrawal_lamports,
            request.native_lamports,
        )?;
        next.snapshot.maximum_deposit_lamports = next
            .snapshot
            .maximum_deposit_lamports
            .checked_sub(request.native_lamports)
            .ok_or(StakePoolError::InsufficientPoolLiquidity)?;
        next.audit.deposited_native_lamports = checked_add(
            next.audit.deposited_native_lamports,
            request.native_lamports,
        )?;
        next.audit.deposited_user_pool_tokens = checked_add(
            next.audit.deposited_user_pool_tokens,
            quote.quoted_pool_tokens_out,
        )?;
        next.audit.deposit_fee_pool_tokens = checked_add(
            next.audit.deposit_fee_pool_tokens,
            quote.deposit_fee_pool_tokens,
        )?;
        next.bump_revision()?;
        next.snapshot.last_update_epoch = next.snapshot.current_epoch;
        next.validate_conservation()?;
        if next.should_fail(MockFailurePoint::DepositBeforeCommit) {
            return Err(StakePoolError::InjectedMockFailure);
        }

        let execution = SolDepositExecution {
            quote,
            actual_pool_tokens_out: quote.quoted_pool_tokens_out,
            actual_fee_pool_tokens: quote.deposit_fee_pool_tokens,
        };
        *self = next;
        Ok(execution)
    }

    fn quote_stake_withdrawal(
        &self,
        request: StakeWithdrawalRequest,
    ) -> StakePoolResult<StakeWithdrawalQuote> {
        self.quote_withdrawal_inner(request)
    }

    fn initiate_protected_stake_withdrawal(
        &mut self,
        request: StakeWithdrawalRequest,
    ) -> StakePoolResult<StakeWithdrawalInitiation> {
        let quote = self.quote_withdrawal_inner(request)?;
        let source = *self.source(request.source_id)?;
        let rent_required = checked_add(
            source.stake_rent_lamports,
            source.metadata_rent_lamports,
        )?;
        if rent_required > self.operational_rent_lamports {
            return Err(StakePoolError::InsufficientOperationalRent);
        }
        let first_eligible_finalization_epoch = self
            .snapshot
            .current_epoch
            .checked_add(source.cooldown_epochs)
            .ok_or(StakePoolError::ArithmeticOverflow)?;

        let initiation = StakeWithdrawalInitiation {
            quote,
            actual_delegated_native_lamports: quote.expected_delegated_native_lamports,
            initiation_epoch: self.snapshot.current_epoch,
            deactivation_epoch: self.snapshot.current_epoch,
            first_eligible_finalization_epoch,
            status: DelayedWithdrawalStatus::Deactivating,
            stake_rent_lamports: source.stake_rent_lamports,
            metadata_rent_lamports: source.metadata_rent_lamports,
        };

        let mut next = self.clone();
        next.snapshot.total_pool_lamports = next
            .snapshot
            .total_pool_lamports
            .checked_sub(quote.expected_delegated_native_lamports)
            .ok_or(StakePoolError::InsufficientPoolLiquidity)?;
        next.snapshot.pool_token_supply = next
            .snapshot
            .pool_token_supply
            .checked_sub(quote.burned_pool_tokens)
            .ok_or(StakePoolError::InsufficientPoolLiquidity)?;
        next.snapshot.available_withdrawal_lamports = next
            .snapshot
            .available_withdrawal_lamports
            .checked_sub(quote.expected_delegated_native_lamports)
            .ok_or(StakePoolError::InsufficientPoolLiquidity)?;
        next.source_mut(request.source_id)?.capacity_pool_tokens = source
            .capacity_pool_tokens
            .checked_sub(quote.pool_tokens_in)
            .ok_or(StakePoolError::InsufficientSourceCapacity)?;
        next.operational_rent_lamports = next
            .operational_rent_lamports
            .checked_sub(rent_required)
            .ok_or(StakePoolError::InsufficientOperationalRent)?;

        next.audit.withdrawal_input_pool_tokens = checked_add(
            next.audit.withdrawal_input_pool_tokens,
            quote.pool_tokens_in,
        )?;
        next.audit.withdrawal_fee_pool_tokens = checked_add(
            next.audit.withdrawal_fee_pool_tokens,
            quote.withdrawal_fee_pool_tokens,
        )?;
        next.audit.burned_pool_tokens = checked_add(
            next.audit.burned_pool_tokens,
            quote.burned_pool_tokens,
        )?;
        next.audit.delegated_native_lamports = checked_add(
            next.audit.delegated_native_lamports,
            quote.expected_delegated_native_lamports,
        )?;
        next.audit.rent_advanced_lamports = checked_add(
            next.audit.rent_advanced_lamports,
            rent_required,
        )?;
        if next.should_fail(MockFailurePoint::WithdrawalAfterPoolDebit) {
            return Err(StakePoolError::InjectedMockFailure);
        }

        let record_index = next.vacant_withdrawal_index()?;
        next.withdrawals[record_index] = MockWithdrawalRecord {
            is_occupied: true,
            initiation,
            configured_reward_lamports: source.cooldown_reward_lamports,
            configured_loss_lamports: source.cooldown_loss_lamports,
            status: DelayedWithdrawalStatus::Deactivating,
            finalization: None,
        };
        next.withdrawal_count = next
            .withdrawal_count
            .checked_add(1)
            .ok_or(StakePoolError::ArithmeticOverflow)?;
        next.bump_revision()?;
        next.snapshot.last_update_epoch = next.snapshot.current_epoch;
        next.validate_conservation()?;
        if next.should_fail(MockFailurePoint::WithdrawalBeforeCommit) {
            return Err(StakePoolError::InjectedMockFailure);
        }
        *self = next;
        Ok(initiation)
    }

    fn delayed_withdrawal(
        &self,
        withdrawal_id: WithdrawalId,
    ) -> StakePoolResult<DelayedWithdrawal> {
        if self.should_fail(MockFailurePoint::StatusRead) {
            return Err(StakePoolError::InjectedMockFailure);
        }
        let index = self.withdrawal_index(withdrawal_id)?;
        Ok(self.delayed_from_record(&self.withdrawals[index]))
    }

    fn finalize_delayed_stake_withdrawal(
        &mut self,
        request: FinalizeWithdrawalRequest,
    ) -> StakePoolResult<StakeWithdrawalFinalization> {
        if self.should_fail(MockFailurePoint::FinalizationBeforeValidation) {
            return Err(StakePoolError::InjectedMockFailure);
        }
        let index = self.withdrawal_index(request.withdrawal_id)?;
        let record = self.withdrawals[index];
        if record.status == DelayedWithdrawalStatus::Finalized {
            return Err(StakePoolError::AlreadyFinalized);
        }
        let delayed = self.delayed_from_record(&record);
        if delayed.status != DelayedWithdrawalStatus::Inactive {
            return Err(StakePoolError::WithdrawalNotInactive);
        }
        if self.should_fail(MockFailurePoint::FinalizationAfterReadiness) {
            return Err(StakePoolError::InjectedMockFailure);
        }

        let delegated_plus_rent = checked_add(
            record.initiation.actual_delegated_native_lamports,
            record.initiation.stake_rent_lamports,
        )?;
        let rewarded = checked_add(
            delegated_plus_rent,
            record.configured_reward_lamports,
        )?;
        let finalized_native_lamports = rewarded
            .checked_sub(record.configured_loss_lamports)
            .ok_or(StakePoolError::ArithmeticOverflow)?;
        let finalization = StakeWithdrawalFinalization {
            withdrawal_id: request.withdrawal_id,
            status: DelayedWithdrawalStatus::Finalized,
            initiation_epoch: record.initiation.initiation_epoch,
            deactivation_epoch: record.initiation.deactivation_epoch,
            first_eligible_finalization_epoch: record
                .initiation
                .first_eligible_finalization_epoch,
            finalized_epoch: self.snapshot.current_epoch,
            delegated_native_lamports: record.initiation.actual_delegated_native_lamports,
            cooldown_reward_lamports: record.configured_reward_lamports,
            cooldown_loss_lamports: record.configured_loss_lamports,
            recovered_stake_rent_lamports: record.initiation.stake_rent_lamports,
            recovered_metadata_rent_lamports: record.initiation.metadata_rent_lamports,
            finalized_native_lamports,
        };

        let mut next = self.clone();
        next.withdrawals[index].status = DelayedWithdrawalStatus::Finalized;
        next.withdrawals[index].finalization = Some(finalization);
        let recovered_rent = checked_add(
            finalization.recovered_stake_rent_lamports,
            finalization.recovered_metadata_rent_lamports,
        )?;
        next.operational_rent_lamports = checked_add(
            next.operational_rent_lamports,
            recovered_rent,
        )?;
        next.audit.finalized_native_lamports = checked_add(
            next.audit.finalized_native_lamports,
            finalization.finalized_native_lamports,
        )?;
        next.audit.recovered_stake_rent_lamports = checked_add(
            next.audit.recovered_stake_rent_lamports,
            finalization.recovered_stake_rent_lamports,
        )?;
        next.audit.recovered_metadata_rent_lamports = checked_add(
            next.audit.recovered_metadata_rent_lamports,
            finalization.recovered_metadata_rent_lamports,
        )?;
        next.audit.cooldown_reward_lamports = checked_add(
            next.audit.cooldown_reward_lamports,
            finalization.cooldown_reward_lamports,
        )?;
        next.audit.cooldown_loss_lamports = checked_add(
            next.audit.cooldown_loss_lamports,
            finalization.cooldown_loss_lamports,
        )?;
        if next.should_fail(MockFailurePoint::FinalizationAfterAccounting) {
            return Err(StakePoolError::InjectedMockFailure);
        }
        next.validate_conservation()?;
        if next.should_fail(MockFailurePoint::FinalizationBeforeCommit) {
            return Err(StakePoolError::InjectedMockFailure);
        }
        *self = next;
        Ok(finalization)
    }
}

fn validate_sources(
    sources: &[MockWithdrawalSource; MAX_MOCK_WITHDRAWAL_SOURCES],
) -> StakePoolResult<()> {
    for (index, source) in sources.iter().enumerate() {
        if !source.is_configured {
            continue;
        }
        if source.cooldown_epochs == 0
            || (source.cooldown_reward_lamports > 0
                && source.cooldown_loss_lamports > 0)
            || sources[..index]
                .iter()
                .any(|prior| prior.is_configured && prior.id == source.id)
        {
            return Err(StakePoolError::InvalidConfiguration);
        }
    }
    Ok(())
}

fn validate_slippage(slippage_bps: u16) -> StakePoolResult<()> {
    if slippage_bps > MAX_PROTECTED_SLIPPAGE_BPS {
        return Err(StakePoolError::InvalidSlippage);
    }
    Ok(())
}

fn fee_ceil(amount: u64, fee: FeeFraction) -> StakePoolResult<u64> {
    fee.validate()?;
    checked_mul_div_ceil(amount, fee.numerator, fee.denominator)
}

fn slippage_floor(amount: u64, slippage_bps: u16) -> StakePoolResult<u64> {
    validate_slippage(slippage_bps)?;
    let retained_bps = SLIPPAGE_BASIS_POINTS_DENOMINATOR
        .checked_sub(u64::from(slippage_bps))
        .ok_or(StakePoolError::InvalidSlippage)?;
    checked_mul_div_floor(
        amount,
        retained_bps,
        SLIPPAGE_BASIS_POINTS_DENOMINATOR,
    )
}

fn checked_mul_div_floor(left: u64, right: u64, denominator: u64) -> StakePoolResult<u64> {
    piv1_math::checked_mul_div_floor(left, right, denominator).map_err(map_math_error)
}

fn checked_mul_div_ceil(left: u64, right: u64, denominator: u64) -> StakePoolResult<u64> {
    piv1_math::checked_mul_div_ceil(left, right, denominator).map_err(map_math_error)
}

fn map_math_error(error: piv1_math::MathError) -> StakePoolError {
    match error {
        piv1_math::MathError::DivisionByZero => StakePoolError::DivisionByZero,
        piv1_math::MathError::NarrowingConversion => StakePoolError::NarrowingConversion,
        _ => StakePoolError::ArithmeticOverflow,
    }
}

fn checked_add(left: u64, right: u64) -> StakePoolResult<u64> {
    left.checked_add(right)
        .ok_or(StakePoolError::ArithmeticOverflow)
}

fn checked_add_u128(left: u128, right: u128) -> StakePoolResult<u128> {
    left.checked_add(right)
        .ok_or(StakePoolError::ArithmeticOverflow)
}
