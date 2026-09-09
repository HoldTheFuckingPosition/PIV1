#![allow(dead_code)]
//! Host-only atomic custody composition. Starts at Idle, never at a fabricated
//! active movement receipt. Public fields exist solely for negative corruption tests.

use anchor_lang::prelude::Pubkey;
use piv1::{
    constants::*,
    errors::Piv1Error,
    integrations::*,
    state::{*, reconciliation::*},
};
use super::stake_pool_mock::{
    MockStakePool, MockWithdrawalSource, MAX_MOCK_WITHDRAWAL_SOURCES,
    MAX_MOCK_WITHDRAWALS,
};

pub const PENDING: usize = 0;
pub const PRINCIPAL: usize = 1;
pub const ESCROW: usize = 2;
pub const KIF: usize = 3;
pub const OPERATIONS: usize = 4;
pub const HTFP: usize = 5;
pub const TEAM: usize = 6;
pub const CLAIMS: usize = 7;
const STAKE_START: usize = 8;
const METADATA_START: usize = STAKE_START + MAX_MOCK_WITHDRAWALS;
const SOL_ACCOUNTS: usize = METADATA_START + MAX_MOCK_WITHDRAWALS;
pub const PENDING_TOKEN: usize = 0;
pub const PRINCIPAL_TOKEN: usize = 1;
const FEE_TOKEN: usize = 2;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Failure {
    AfterDebit,
    MissingCredit,
    WrongCredit,
    BeforeCommit,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Error {
    State(Piv1Error),
    Pool(StakePoolError),
    Injected,
    Conservation,
    Capacity,
}
impl From<Piv1Error> for Error { fn from(e: Piv1Error) -> Self { Self::State(e) } }
impl From<StakePoolError> for Error { fn from(e: StakePoolError) -> Self { Self::Pool(e) } }
pub type Result<T> = core::result::Result<T, Error>;

/// Independent test oracle: explicit initial totals and external flows only.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct Audit {
    initial_native: u128,
    initial_tokens: u128,
    initial_reserve: u64,
    pub external_sol: u128,
    pub external_tokens: u128,
    pub consumed_tokens: u128,
    pub fee_tokens: u128,
    pub burned_tokens: u128,
    pub advanced_rent: u128,
    pub recovered_rent: u128,
    pub cooldown_reward: u128,
    pub cooldown_loss: u128,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct World {
    pub config: PivConfig,
    pub round: ActiveDistribution,
    pub registry: GuardianRegistry,
    pub rewards: [GuardianReward; GUARDIAN_COUNT],
    pub pool: MockStakePool,
    pub sol: [u64; SOL_ACCOUNTS],
    pub floors: [u64; SOL_ACCOUNTS],
    pub tokens: [u64; 3],
    pub token_rent: [u64; 2],
    pub legs: [WithdrawalLeg; MAX_MOCK_WITHDRAWALS],
    pub audit: Audit,
    pub failure: Option<Failure>,
}

impl World {
    pub fn new(pending_sol: u64, pending_tokens: u64, carry: u64,
               kif_carry: u64, active_count: usize, fee: FeeFraction,
               source_capacity: u64) -> Self {
        let mut config = base_config(pending_sol, pending_tokens);
        config.accounted_historical_jitosol_units = 1_000_000;
        config.accounted_historical_sol_lamports = 0;
        config.protected_principal_hwm_lamports = 1_000_000;
        config.next_cycle_yield_lamports = carry;
        config.collective_kif_carry_lamports = kif_carry;
        // An explicit existing earned-liability fixture, not a claim implementation.
        config.kif_claim_liability_lamports = 90;
        config.cumulative_kif_credited_lamports = 90;
        config.cumulative_kif_claimed_lamports = 0;
        config.cumulative_contribution_value_lamports = 0;
        config.cumulative_gross_yield_lamports = 0;
        config.cumulative_htfp_paid_lamports = 0;
        config.cumulative_team_owner_paid_lamports = 0;
        config.cumulative_permanent_compound_lamports = 0;
        config.cumulative_retained_dust_lamports = 0;
        config.cumulative_zero_active_kif_compound_lamports = 0;
        config.cumulative_cooldown_yield_recorded_lamports = 0;
        let registry = GuardianRegistry::new(90, 7,
            core::array::from_fn(|i| key(100 + i as u8))).unwrap();
        let mut rewards = core::array::from_fn(|i|
            GuardianReward::new(i as u8, &registry, i as u8).unwrap());
        for (i, reward) in rewards.iter_mut().enumerate() {
            reward.claimable_lamports = 15;
            reward.cumulative_earned = 15;
            if i < active_count { reward.record_activity(&registry, i as u8, 0).unwrap(); }
        }
        let snapshot = PoolSnapshot {
            current_epoch: 40, last_update_epoch: 40,
            total_pool_lamports: 10_100_000, pool_token_supply: 10_000_000,
            sol_deposit_fee: FeeFraction::ZERO, stake_withdrawal_fee: fee,
            minimum_delegation_lamports: 10,
            maximum_deposit_lamports: 10_000_000,
            available_withdrawal_lamports: 1_000_000, revision: 0,
        };
        let mut sources = [MockWithdrawalSource::VACANT; MAX_MOCK_WITHDRAWAL_SOURCES];
        for (i, source) in sources.iter_mut().enumerate() {
            *source = MockWithdrawalSource::new(i as u32 + 1, source_capacity);
        }
        let pool = MockStakePool::new(snapshot, sources, 100_000).unwrap();
        let mut floors = [0; SOL_ACCOUNTS];
        floors[..5].copy_from_slice(&[17, 19, 23, 29, 31]);
        let mut sol = floors;
        sol[PENDING] += pending_sol;
        sol[PRINCIPAL] += carry;
        sol[KIF] += 90 + kif_carry;
        sol[OPERATIONS] += 100_000;
        let mut world = Self {
            config, round: ActiveDistribution::new_idle(80), registry, rewards, pool,
            sol, floors, tokens: [pending_tokens, 1_000_000, 0],
            token_rent: [37, 41],
            legs: [WithdrawalLeg::vacant(0, 0); MAX_MOCK_WITHDRAWALS],
            audit: Audit::default(), failure: None,
        };
        world.audit.initial_native = world.total_native();
        world.audit.initial_tokens = world.total_tokens();
        world.audit.initial_reserve = 100_000;
        world.validate().unwrap();
        world
    }

    pub fn observation(&self) -> EconomicCustodyObservation {
        let balance = |i| SolVaultBalance {
            lamports: self.sol[i], non_economic_floor_lamports: self.floors[i],
        };
        EconomicCustodyObservation {
            pending_sol: balance(PENDING), principal_sol: balance(PRINCIPAL),
            distribution_escrow: balance(ESCROW), kif_sol: balance(KIF),
            pending_jitosol_units: self.tokens[PENDING_TOKEN],
            principal_jitosol_units: self.tokens[PRINCIPAL_TOKEN],
        }
    }

    pub fn spendable(&self, vault: usize) -> Result<u64> {
        sub(self.sol[vault], self.floors[vault])
    }

    fn atomic<T>(&mut self, action: impl FnOnce(&mut Self) -> Result<T>) -> Result<T> {
        // Known deficits reject before an incoming deposit can obscure them.
        self.validate()?;
        let mut next = self.clone();
        let value = action(&mut next)?;
        next.validate()?;
        if next.failure == Some(Failure::BeforeCommit) { return Err(Error::Injected); }
        *self = next;
        Ok(value)
    }

    /// Exact same-asset transfer; failure controls never leave staged state.
    fn move_sol(&mut self, source: usize, destination: usize, amount: u64) -> Result<()> {
        if amount == 0 { return Ok(()); }
        if source == destination { return Err(Error::Conservation); }
        let before_source = self.sol[source];
        let before_destination = self.sol[destination];
        if self.spendable(source)? < amount { return Err(Error::Conservation); }
        self.sol[source] = sub(before_source, amount)?;
        if self.failure == Some(Failure::AfterDebit) { return Err(Error::Injected); }
        let actual_destination = if self.failure == Some(Failure::WrongCredit) {
            if destination == CLAIMS { TEAM } else { CLAIMS }
        } else { destination };
        if self.failure != Some(Failure::MissingCredit) {
            self.sol[actual_destination] = add(self.sol[actual_destination], amount)?;
        }
        if sub(before_source, self.sol[source])? != amount
            || sub(self.sol[destination], before_destination)? != amount {
            return Err(Error::Conservation);
        }
        Ok(())
    }

    fn move_tokens(&mut self, source: usize, destination: usize, amount: u64) -> Result<()> {
        if amount == 0 { return Ok(()); }
        let before_source = self.tokens[source];
        let before_destination = self.tokens[destination];
        self.tokens[source] = sub(before_source, amount)?;
        if self.failure == Some(Failure::AfterDebit) { return Err(Error::Injected); }
        let actual_destination = if self.failure == Some(Failure::WrongCredit) {
            FEE_TOKEN
        } else { destination };
        if self.failure != Some(Failure::MissingCredit) {
            self.tokens[actual_destination] = add(self.tokens[actual_destination], amount)?;
        }
        if sub(before_source, self.tokens[source])? != amount
            || sub(self.tokens[destination], before_destination)? != amount {
            return Err(Error::Conservation);
        }
        Ok(())
    }

    pub fn explicit_sol(&mut self, amount: u64, credited: u64) -> Result<()> {
        self.atomic(|w| {
            let before = w.sol[PENDING];
            w.sol[PENDING] = add(before, credited)?;
            record_explicit_sol_contribution(&mut w.config, &w.round, amount,
                SolCustodyObservation { vault_lamports_before: before,
                    vault_lamports_after: w.sol[PENDING],
                    non_economic_floor_lamports: w.floors[PENDING] })?;
            w.audit.external_sol += u128::from(credited);
            Ok(())
        })
    }

    pub fn explicit_tokens(&mut self, amount: u64, credited: u64) -> Result<()> {
        self.atomic(|w| {
            let before = w.tokens[PENDING_TOKEN];
            w.tokens[PENDING_TOKEN] = add(before, credited)?;
            record_explicit_jitosol_contribution(&mut w.config, &w.round, amount,
                JitoSolCustodyObservation { token_units_before: before,
                    token_units_after: w.tokens[PENDING_TOKEN] })?;
            w.audit.external_tokens += u128::from(credited);
            Ok(())
        })
    }

    pub fn direct_sol(&mut self, vault: usize, amount: u64) -> Result<()> {
        self.atomic(|w| {
            if vault > OPERATIONS { return Err(Error::Capacity); }
            w.sol[vault] = add(w.sol[vault], amount)?;
            w.audit.external_sol += u128::from(amount);
            Ok(())
        })
    }

    pub fn direct_tokens(&mut self, vault: usize, amount: u64) -> Result<()> {
        self.atomic(|w| {
            if vault > PRINCIPAL_TOKEN { return Err(Error::Capacity); }
            w.tokens[vault] = add(w.tokens[vault], amount)?;
            w.audit.external_tokens += u128::from(amount);
            Ok(())
        })
    }

    pub fn reconcile(&mut self) -> Result<PendingReconciliationResult> {
        self.atomic(|w| {
            let obs = w.observation().pending();
            Ok(reconcile_pending_contributions(&mut w.config, &w.round, obs)?)
        })
    }

    pub fn normalize(&mut self) -> Result<PendingReconciliationResult> {
        self.atomic(|w| {
            let before = w.observation();
            let s = economic_custody_surplus(&w.config, &w.round, before)?;
            w.move_sol(PRINCIPAL, PENDING, s.principal_sol_lamports)?;
            w.move_sol(ESCROW, PENDING, s.escrow_sol_lamports)?;
            w.move_sol(KIF, PENDING, s.kif_sol_lamports)?;
            w.move_tokens(PRINCIPAL_TOKEN, PENDING_TOKEN, s.principal_jitosol_units)?;
            let after = w.observation();
            Ok(record_economic_normalization(&mut w.config, &w.round, before, after)?)
        })
    }

    /// Derives the snapshot and funding, then stages observed debits/credits.
    /// Target floors are deterministic mock evidence, not live SPL derivation.
    pub fn open(&mut self, now: i64) -> Result<()> {
        self.atomic(|w| {
            w.require_normalized()?;
            let pool = w.pool.pool_snapshot()?;
            let historical_value = add(w.config.accounted_historical_sol_lamports,
                observed_token_book_value(w.tokens[PRINCIPAL_TOKEN], pool)?)?;
            let basis = add(historical_value, w.config.next_cycle_yield_lamports)?;
            let gross = basis.checked_sub(w.config.protected_principal_hwm_lamports)
                .ok_or(Piv1Error::ZeroTarget)?;
            let split = piv1_math::split_gross_yield(gross).map_err(Piv1Error::from)?;
            let outgoing = add(add(split.htfp_reserve, split.team_owner_pool)?, split.kif)?;
            let used = w.config.accounted_pending_sol_lamports.min(outgoing);
            let carry_used = w.config.next_cycle_yield_lamports.min(sub(outgoing, used)?);
            let shortfall = sub(sub(outgoing, used)?, carry_used)?;
            let target = if shortfall == 0 { 0 } else {
                // Floor gross input before protocol fee: fees reduce beneficiaries.
                piv1_math::checked_mul_div_floor(shortfall, pool.pool_token_supply,
                    pool.total_pool_lamports).map_err(Piv1Error::from)?
            };
            let proposed = add(w.config.protected_principal_hwm_lamports,
                add(split.permanent_compound, split.dust)?)?;
            let before_pending = w.sol[PENDING];
            let before_principal = w.sol[PRINCIPAL];
            w.move_sol(PENDING, ESCROW, used)?;
            w.move_sol(PRINCIPAL, ESCROW, carry_used)?;
            let observed_used = sub(before_pending, w.sol[PENDING])?;
            if sub(before_principal, w.sol[PRINCIPAL])? != carry_used {
                return Err(Error::Conservation);
            }
            let escrow = w.spendable(ESCROW)?;
            let funding = if shortfall == 0 {
                DistributionFunding::Liquid { escrow_available_lamports: escrow }
            } else {
                // Probe the minimum using the accepted mock's quote computation.
                let quote = w.pool.quote_stake_withdrawal(StakeWithdrawalRequest {
                    snapshot: pool.identity(),
                    withdrawal_id: WithdrawalId { sequence: w.config.next_distribution_sequence,
                                                   leg_index: 0 },
                    source_id: WithdrawalSourceId(1), remaining_pool_token_target: target,
                    caller_minimum_native_lamports_out: 0,
                    slippage_bps: w.config.configured_slippage_bps,
                })?;
                DistributionFunding::Withdrawal {
                    fixed_jitosol_target_units: target,
                    snapshot_leg_input_floor_units: quote.technical_minimum_pool_tokens,
                    maximum_useful_legs: target / quote.technical_minimum_pool_tokens,
                    // Each successful mock leg yields at least the dynamic delegation
                    // minimum; real round slippage/HWM mapping remains deferred.
                    stored_round_minimum_native_lamports: pool.minimum_delegation_lamports,
                    initial_escrow_available_lamports: escrow,
                }
            };
            let input = OpenDistributionInput {
                sequence: w.config.next_distribution_sequence, prepared_at: now,
                prepared_slot: 1, prepared_epoch: pool.current_epoch,
                historical_jitosol_units: w.config.accounted_historical_jitosol_units,
                historical_sol_lamports: w.config.accounted_historical_sol_lamports,
                historical_value_lamports: historical_value,
                snapshot_pool_total_lamports: pool.total_pool_lamports,
                snapshot_pool_token_supply: pool.pool_token_supply,
                snapshot_withdrawal_fee_numerator: pool.stake_withdrawal_fee.numerator,
                snapshot_withdrawal_fee_denominator: pool.stake_withdrawal_fee.denominator,
                gross_yield_lamports: gross,
                pending_sol_snapshot_lamports: w.config.accounted_pending_sol_lamports,
                pending_sol_used_lamports: observed_used,
                snapshot_conversion_dust_lamports: 0,
                stored_residual_hwm_floor_lamports: proposed, funding,
            };
            open_distribution(&mut w.config, &mut w.round, &w.registry, &w.rewards, input)?;
            Ok(())
        })
    }

    pub fn initiate(&mut self, source: u32) -> Result<usize> {
        self.atomic(|w| {
            w.require_normalized()?;
            let index = usize::try_from(w.round.next_leg_index).map_err(|_| Error::Capacity)?;
            if index >= MAX_MOCK_WITHDRAWALS { return Err(Error::Capacity); }
            let pool = w.pool.pool_snapshot()?;
            let request = StakeWithdrawalRequest {
                snapshot: pool.identity(),
                withdrawal_id: WithdrawalId { sequence: w.round.active_sequence,
                                               leg_index: w.round.next_leg_index },
                source_id: WithdrawalSourceId(source),
                remaining_pool_token_target: w.round.remaining_withdrawal_target_units()?,
                caller_minimum_native_lamports_out: 0,
                slippage_bps: w.round.stored_slippage_bps,
            };
            let before_tokens = w.tokens[PRINCIPAL_TOKEN];
            let before_pool = pool.total_pool_lamports;
            let before_supply = pool.pool_token_supply;
            let execution = w.pool.initiate_protected_stake_withdrawal(request)?;
            let q = execution.quote;
            w.move_tokens(PRINCIPAL_TOKEN, FEE_TOKEN, q.withdrawal_fee_pool_tokens)?;
            w.tokens[PRINCIPAL_TOKEN] = sub(w.tokens[PRINCIPAL_TOKEN], q.burned_pool_tokens)?;
            if w.failure == Some(Failure::AfterDebit) { return Err(Error::Injected); }
            let delegated = sub(before_pool, w.pool.raw_snapshot().total_pool_lamports)?;
            if sub(before_supply, w.pool.raw_snapshot().pool_token_supply)? != q.burned_pool_tokens
                || delegated != execution.actual_delegated_native_lamports
                || sub(before_tokens, w.tokens[PRINCIPAL_TOKEN])? != q.pool_tokens_in {
                return Err(Error::Conservation);
            }
            if w.failure != Some(Failure::MissingCredit) {
                let destination = if w.failure == Some(Failure::WrongCredit) { CLAIMS }
                    else { STAKE_START + index };
                w.sol[destination] = add(w.sol[destination], delegated)?;
            }
            w.move_sol(OPERATIONS, STAKE_START + index, execution.stake_rent_lamports)?;
            w.move_sol(OPERATIONS, METADATA_START + index, execution.metadata_rent_lamports)?;
            let observed_delegated = sub(w.sol[STAKE_START + index], execution.stake_rent_lamports)?;
            let input = LegInitiationInput {
                sequence: request.withdrawal_id.sequence, leg_index: request.withdrawal_id.leg_index,
                validator_list_index: source, validator_seed_suffix: source,
                validator_vote: key(150 + source as u8),
                validator_stake_source: key(170 + source as u8),
                initiation_epoch: execution.initiation_epoch,
                pool_total_lamports: pool.total_pool_lamports, pool_token_supply: pool.pool_token_supply,
                withdrawal_fee_numerator: pool.stake_withdrawal_fee.numerator,
                withdrawal_fee_denominator: pool.stake_withdrawal_fee.denominator,
                current_technical_floor_units: q.technical_minimum_pool_tokens,
                maximum_safe_capacity_units: q.source_capacity_pool_tokens,
                jitosol_input_units: sub(before_tokens, w.tokens[PRINCIPAL_TOKEN])?,
                withdrawal_fee_units: q.withdrawal_fee_pool_tokens,
                burned_units: q.burned_pool_tokens,
                expected_native_lamports: q.expected_delegated_native_lamports,
                observed_delegated_native_lamports: observed_delegated,
                minimum_native_lamports: q.minimum_native_lamports_out,
                stake_rent_advanced_lamports: execution.stake_rent_lamports,
                metadata_rent_advanced_lamports: execution.metadata_rent_lamports,
            };
            // Leg slots are reused only after prior round completion; mock pool
            // retains its own immutable identifier history for replay checks.
            w.legs[index] = WithdrawalLeg::vacant(0, 0);
            initiate_withdrawal_leg(&w.config, &mut w.round, &mut w.legs[index], input)?;
            w.audit.consumed_tokens += u128::from(q.pool_tokens_in);
            w.audit.fee_tokens += u128::from(q.withdrawal_fee_pool_tokens);
            w.audit.burned_tokens += u128::from(q.burned_pool_tokens);
            w.audit.advanced_rent += u128::from(execution.stake_rent_lamports)
                + u128::from(execution.metadata_rent_lamports);
            Ok(index)
        })
    }

    pub fn advance_epoch(&mut self) -> Result<()> {
        self.atomic(|w| {
            w.pool.advance_epoch_to(add(w.pool.raw_snapshot().current_epoch, 1)?)?;
            // Explicit mock maintenance is required before fresh valuation.
            w.pool.refresh_pool()?;
            Ok(())
        })
    }

    pub fn finalize(&mut self, index: usize) -> Result<LegFinalizationOutcome> {
        self.atomic(|w| {
            w.require_normalized()?;
            let leg = *w.legs.get(index).ok_or(Error::Capacity)?;
            let result = w.pool.finalize_delayed_stake_withdrawal(FinalizeWithdrawalRequest {
                withdrawal_id: WithdrawalId { sequence: leg.sequence, leg_index: leg.leg_index },
            })?;
            let stake = STAKE_START + index;
            w.sol[stake] = sub(add(w.sol[stake], result.cooldown_reward_lamports)?,
                               result.cooldown_loss_lamports)?;
            let final_native = w.sol[stake];
            if final_native != result.finalized_native_lamports { return Err(Error::Conservation); }
            w.move_sol(stake, ESCROW, final_native)?;
            w.move_sol(ESCROW, OPERATIONS, result.recovered_stake_rent_lamports)?;
            w.move_sol(METADATA_START + index, OPERATIONS, result.recovered_metadata_rent_lamports)?;
            w.audit.recovered_rent += u128::from(result.recovered_stake_rent_lamports)
                + u128::from(result.recovered_metadata_rent_lamports);
            w.audit.cooldown_reward += u128::from(result.cooldown_reward_lamports);
            w.audit.cooldown_loss += u128::from(result.cooldown_loss_lamports);
            let residual = historical_value_for_recovery(add(w.spendable(PRINCIPAL)?,
                observed_token_book_value(w.tokens[PRINCIPAL_TOKEN], w.pool.pool_snapshot()?)?)?,
                w.round.pending_sol_used_lamports)?;
            let input = LegFinalizationInput {
                sequence: leg.sequence, leg_index: leg.leg_index,
                finalized_epoch: result.finalized_epoch,
                finalized_native_lamports: final_native,
                recovered_stake_rent_lamports: result.recovered_stake_rent_lamports,
                recovered_metadata_rent_lamports: result.recovered_metadata_rent_lamports,
                cooldown_reward_lamports: result.cooldown_reward_lamports,
                cooldown_loss_lamports: result.cooldown_loss_lamports,
                validated_residual_historical_value_lamports: residual,
                escrow_available_after_lamports: w.spendable(ESCROW)?,
            };
            Ok(finalize_withdrawal_leg(&w.config, &mut w.round, &mut w.legs[index], input)?)
        })
    }

    pub fn settle(&mut self) -> Result<SettlementOutcome> {
        self.atomic(|w| {
            w.require_normalized()?;
            let original = w.clone();
            let r = w.round;
            // Independent host implementation of accepted relative-weight floors.
            let eligible = add(add(r.pending_sol_used_lamports,
                r.prior_next_cycle_yield_used_lamports()?)?,
                r.cumulative_finalized_delegated_native_lamports)?;
            let net = eligible.min(r.outgoing_gross_obligation_lamports);
            let htfp = ((u128::from(net) * 5900 / 8050) as u64).min(r.htfp_gross_obligation_lamports);
            let team = ((u128::from(net) * 1950 / 8050) as u64).min(r.team_owner_gross_obligation_lamports);
            let kif = ((u128::from(net) * 200 / 8050) as u64).min(r.kif_gross_obligation_lamports);
            let zero = if r.kif_active_guardian_count == 0 { add(kif, r.kif_carry_input_lamports)? / 2 }
                else { 0 };
            let escrow_before = w.spendable(ESCROW)?;
            w.move_sol(ESCROW, HTFP, htfp)?;
            w.move_sol(ESCROW, TEAM, team)?;
            w.move_sol(ESCROW, KIF, kif)?;
            w.move_sol(KIF, PRINCIPAL, zero)?;
            // Exclude all pending value and the new delayed yield. The already
            // spent contribution offset also remains pending economically.
            let residual = add(w.spendable(PRINCIPAL)?,
                observed_token_book_value(w.tokens[PRINCIPAL_TOKEN], w.pool.pool_snapshot()?)?)?;
            let protected = historical_value_for_recovery(
                add(residual, sub(w.spendable(ESCROW)?, r.cumulative_cooldown_rewards_lamports)?)?,
                r.pending_sol_used_lamports)?;
            let outcome = settle_distribution(&mut w.config, &mut w.round, &mut w.rewards,
                SettlementInput { sequence: r.active_sequence,
                    escrow_available_lamports: escrow_before,
                    validated_post_settlement_protected_value_lamports: protected })?;
            if outcome == SettlementOutcome::RecoveryRequired {
                let recovery_round = w.round;
                *w = original;
                w.round = recovery_round;
            } else if sub(escrow_before, w.spendable(ESCROW)?)?
                != w.round.actual_allocated_outgoing_lamports {
                return Err(Error::Conservation);
            }
            Ok(outcome)
        })
    }

    pub fn integrate(&mut self, now: i64) -> Result<CompletedDistributionSummary> {
        self.atomic(|w| {
            w.require_normalized()?;
            let before = w.observation();
            let remaining = expected_pending_sol_lamports(&w.config, &w.round)?;
            w.move_sol(PENDING, PRINCIPAL, remaining)?;
            w.move_tokens(PENDING_TOKEN, PRINCIPAL_TOKEN, w.tokens[PENDING_TOKEN])?;
            w.move_sol(ESCROW, PRINCIPAL, w.spendable(ESCROW)?)?;
            let after = w.observation();
            let input = derive_pending_integration(&w.config, &w.round, now,
                                                   w.pool.pool_snapshot()?, before, after)?;
            Ok(integrate_pending_and_complete(&mut w.config, &mut w.round, input)?)
        })
    }

    /// K-012 effect simulation only: no production claim or signer proof.
    pub fn claim_effect(&mut self, index: usize, amount: u64) -> Result<()> {
        self.atomic(|w| {
            let reward = w.rewards.get_mut(index).ok_or(Error::Capacity)?;
            reward.claimable_lamports = sub(reward.claimable_lamports, amount)?;
            reward.cumulative_claimed = add(reward.cumulative_claimed, amount)?;
            w.config.kif_claim_liability_lamports = sub(w.config.kif_claim_liability_lamports, amount)?;
            w.config.cumulative_kif_claimed_lamports = add(w.config.cumulative_kif_claimed_lamports, amount)?;
            w.move_sol(KIF, CLAIMS, amount)?;
            Ok(())
        })
    }

    fn require_normalized(&self) -> Result<()> {
        if economic_custody_surplus(&self.config, &self.round, self.observation())?
            != EconomicVaultAmounts::default() { return Err(Error::Conservation); }
        Ok(())
    }

    fn total_native(&self) -> u128 {
        self.sol.iter().chain(self.token_rent.iter()).map(|&v| u128::from(v)).sum::<u128>()
            + u128::from(self.pool.raw_snapshot().total_pool_lamports)
    }

    fn total_tokens(&self) -> u128 {
        // Pool supply is a liability denominator, not another token asset.
        self.tokens.iter().map(|&v| u128::from(v)).sum()
    }

    pub fn validate(&self) -> Result<()> {
        economic_custody_surplus(&self.config, &self.round, self.observation())?;
        self.pool.validate_conservation()?;
        let pool_audit = self.pool.audit();
        if self.audit.consumed_tokens != u128::from(pool_audit.withdrawal_input_pool_tokens)
            || self.audit.fee_tokens != u128::from(pool_audit.withdrawal_fee_pool_tokens)
            || self.audit.burned_tokens != u128::from(pool_audit.burned_pool_tokens)
            || self.audit.advanced_rent != u128::from(pool_audit.rent_advanced_lamports)
            || self.audit.recovered_rent != u128::from(pool_audit.recovered_stake_rent_lamports)
                + u128::from(pool_audit.recovered_metadata_rent_lamports)
            || self.audit.cooldown_reward != u128::from(pool_audit.cooldown_reward_lamports)
            || self.audit.cooldown_loss != u128::from(pool_audit.cooldown_loss_lamports) {
            return Err(Error::Conservation);
        }
        let native_left = self.audit.initial_native + self.audit.external_sol
            + u128::from(pool_audit.external_pool_rewards_lamports) + self.audit.cooldown_reward;
        let native_right = self.total_native() + u128::from(pool_audit.external_pool_losses_lamports)
            + self.audit.cooldown_loss;
        if native_left != native_right
            || self.audit.initial_tokens + self.audit.external_tokens
                != self.total_tokens() + self.audit.burned_tokens
            || self.audit.consumed_tokens != self.audit.fee_tokens + self.audit.burned_tokens
            || u128::from(self.tokens[FEE_TOKEN]) != self.audit.fee_tokens {
            return Err(Error::Conservation);
        }
        let remaining_rent = self.audit.advanced_rent.checked_sub(self.audit.recovered_rent)
            .ok_or(Error::Conservation)?;
        // Extra operational credits remain unresolved: this oracle only proves
        // the legitimate minimum, never exports its baseline to production.
        let reserve_expected = u128::from(self.audit.initial_reserve)
            .checked_sub(remaining_rent).ok_or(Error::Conservation)?;
        if u128::from(self.spendable(OPERATIONS)?) < reserve_expected
            || u128::from(self.pool.operational_rent_lamports()) != reserve_expected {
            return Err(Error::Conservation);
        }
        let mut claims = 0u128;
        for reward in &self.rewards {
            reward.validate()?;
            claims += u128::from(reward.claimable_lamports);
        }
        if claims != u128::from(self.config.kif_claim_liability_lamports)
            || self.sol[HTFP] != self.config.cumulative_htfp_paid_lamports
            || self.sol[TEAM] != self.config.cumulative_team_owner_paid_lamports
            || self.sol[CLAIMS] != self.config.cumulative_kif_claimed_lamports {
            return Err(Error::Conservation);
        }
        if self.round.lifecycle != DistributionLifecycle::Idle {
            let r = self.round;
            let mut sums = [0u128; 9];
            let mut count = 0u64;
            let mut finalized = 0u64;
            for (i, leg) in self.legs.iter().enumerate() {
                if leg.status == WithdrawalLegStatus::Vacant || leg.sequence != r.active_sequence { continue; }
                leg.validate()?;
                if leg.leg_index != i as u64 || leg.leg_index >= r.next_leg_index {
                    return Err(Error::Conservation);
                }
                count += 1;
                sums[0] += u128::from(leg.jitosol_input_units);
                sums[1] += u128::from(leg.withdrawal_fee_units);
                sums[2] += u128::from(leg.burned_units);
                sums[3] += u128::from(leg.observed_delegated_native_lamports);
                if leg.status == WithdrawalLegStatus::Initiated {
                    if self.sol[STAKE_START + i] != add(leg.observed_delegated_native_lamports,
                                                       leg.stake_rent_advanced_lamports)?
                        || self.sol[METADATA_START + i] != leg.metadata_rent_advanced_lamports {
                        return Err(Error::Conservation);
                    }
                } else {
                    finalized += 1;
                    sums[4] += u128::from(leg.finalized_native_lamports);
                    sums[5] += u128::from(leg.recovered_stake_rent_lamports);
                    sums[6] += u128::from(leg.recovered_metadata_rent_lamports);
                    sums[7] += u128::from(leg.cooldown_reward_lamports);
                    sums[8] += u128::from(leg.cooldown_loss_lamports);
                    if self.sol[STAKE_START + i] != 0 || self.sol[METADATA_START + i] != 0 {
                        return Err(Error::Conservation);
                    }
                }
            }
            let expected = [
                r.cumulative_jitosol_assigned_units, r.cumulative_withdrawal_fee_units,
                r.cumulative_burned_units, r.cumulative_delegated_native_lamports,
                r.cumulative_finalized_native_lamports, r.cumulative_recovered_stake_rent_lamports,
                r.cumulative_recovered_metadata_rent_lamports, r.cumulative_cooldown_rewards_lamports,
                r.cumulative_cooldown_losses_lamports,
            ].map(u128::from);
            if sums != expected || count != r.successful_leg_count || finalized != r.finalized_leg_count {
                return Err(Error::Conservation);
            }
        }
        Ok(())
    }
}

/// HWM comparison value only, after validating bound state and normalized custody.
/// If retained value cannot cover the committed pending contribution, no value
/// remains for historical protection. Zero reaches the existing recovery branch
/// against the positive protected floor of a prepared round. This is not an
/// account balance, contribution value, HWM update or general subtraction rule.
fn historical_value_for_recovery(retained_value: u64, pending_used: u64) -> Result<u64> {
    if retained_value < pending_used { Ok(0) }
    else { sub(retained_value, pending_used) }
}

fn add(a: u64, b: u64) -> Result<u64> {
    a.checked_add(b).ok_or(Error::State(Piv1Error::ArithmeticOverflow))
}
fn sub(a: u64, b: u64) -> Result<u64> {
    a.checked_sub(b).ok_or(Error::State(Piv1Error::CumulativeReconciliationMismatch))
}

fn key(tag: u8) -> Pubkey { Pubkey::new_from_array([tag; 32]) }

fn base_config(
    pending_sol_lamports: u64,
    pending_jitosol_units: u64,
) -> PivConfig {
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
        configured_slippage_bps: MAX_CONFIGURED_SLIPPAGE_BPS,
        slippage_hard_cap_bps: MAX_CONFIGURED_SLIPPAGE_BPS,
        minimum_distribution_interval_seconds:
            MINIMUM_DISTRIBUTION_INTERVAL_SECONDS,
        insufficient_retry_cooldown_seconds:
            INSUFFICIENT_RETRY_COOLDOWN_SECONDS,
        last_successful_preparation_at: None,
        last_valid_insufficient_attempt_at: None,
        next_distribution_sequence: 0,
        protected_principal_hwm_lamports: 1_000,
        accounted_historical_jitosol_units: 2_000,
        accounted_historical_sol_lamports: 3_000,
        accounted_pending_jitosol_units: pending_jitosol_units,
        accounted_pending_sol_lamports: pending_sol_lamports,
        next_cycle_yield_lamports: 4_000,
        kif_claim_liability_lamports: 10,
        collective_kif_carry_lamports: 11,
        cumulative_contribution_value_lamports: 12,
        cumulative_gross_yield_lamports: 13,
        cumulative_htfp_paid_lamports: 14,
        cumulative_team_owner_paid_lamports: 15,
        cumulative_kif_credited_lamports: 17,
        cumulative_kif_claimed_lamports: 7,
        cumulative_permanent_compound_lamports: 18,
        cumulative_retained_dust_lamports: 19,
        cumulative_zero_active_kif_compound_lamports: 20,
        cumulative_cooldown_yield_recorded_lamports: 21,
        kif_anchor_timestamp: 0,
        kif_period_seconds: KIF_PERIOD_SECONDS,
        guardian_registry_revision: 7,
        migration_reserve: [0; CONFIG_MIGRATION_RESERVE_BYTES],
    }
}
