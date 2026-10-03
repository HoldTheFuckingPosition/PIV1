//! Atomic frozen-recipient settlement and snapshot KIF credit. Exact current
//! custody/valuation and protected-value recovery precede all System effects.
use std::rc::Rc;
use anchor_lang::{prelude::{AccountInfo, Clock, Pubkey, Rent, SolanaSysvar},
    solana_program::{entrypoint::ProgramResult, instruction::Instruction,
        program::invoke_signed, program_error::ProgramError, system_instruction,
        system_program, sysvar}};
use solana_sha256_hasher::hash;
use crate::{accounts::{authenticate_fixed_accounts, rent_floor, seeds, FixedAccountInfos},
    constants::GUARDIAN_COUNT, errors::Piv1Error,
    events::{DistributionSettled, DistributionSettlementRecovery},
    instruction_errors::{piv1_error_code, protocol_program_error, HOST_RUNTIME_UNAVAILABLE_CODE},
    instructions::settle_distribution::decode_settle_distribution,
    integrations::jito_identity::{authenticate_jito_identity, AuthenticatedJitoIdentity,
        DeclaredJitoKeys, JitoIdentityAccountInfos},
    kif_claim_accounts::authenticate_reward_account,
    state::{settle_distribution, ActiveDistribution, DistributionLifecycle, GuardianReward,
        PivConfig, SettlementInput, SettlementOutcome,
        distribution::derive_net_beneficiary_allocation, reconciliation::{economic_custody_obligations, EconomicCustodyObservation}},
    state_persistence::{commit_state_writes, PreparedStateWrite, StateEnvelope}};
const AUTHORITY: usize = 9;
const MINT: usize = 10;
const SYSTEM: usize = 11;
const TOKEN: usize = 12;
const PROTOCOL: usize = 13;
const POOL: usize = 14;
const LIST: usize = 15;
const RESERVE: usize = 16;
const MANAGER: usize = 17;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SettlementEvent { Settled(DistributionSettled), Recovery(DistributionSettlementRecovery) }
pub fn process_instruction(program: &Pubkey, a: &[AccountInfo<'_>], data: &[u8]) -> ProgramResult {
    dispatch(program, a, data, cfg!(target_os = "solana"), Clock::get, Rent::get,
        |ix, infos, seeds| invoke_signed(ix, infos, seeds), |event| match event {
            SettlementEvent::Settled(e) => anchor_lang::emit!(e),
            SettlementEvent::Recovery(e) => anchor_lang::emit!(e),
        })
}
/// Host callbacks model effects and explicit failed-world discard. They are not
/// transaction signatures, runtime rollback, or a production bypass selector.
#[cfg(not(target_os = "solana"))]
pub fn process_instruction_with_host_callbacks<'info>(program: &Pubkey, a: &[AccountInfo<'info>], data: &[u8],
    clock: impl FnOnce() -> Result<Clock, ProgramError>, rent: impl FnOnce() -> Result<Rent, ProgramError>,
    invoke: impl FnMut(&Instruction, &[AccountInfo<'info>], &[&[&[u8]]]) -> ProgramResult,
    emit: impl FnOnce(SettlementEvent)) -> ProgramResult {
    dispatch(program, a, data, true, clock, rent, invoke, emit)
}
fn error(e: Piv1Error) -> ProgramError { ProgramError::Custom(piv1_error_code(e)) }
fn bad() -> ProgramError { error(Piv1Error::CumulativeReconciliationMismatch) }
fn add(a: u64, b: u64) -> Result<u64, ProgramError> { a.checked_add(b).ok_or(error(Piv1Error::ArithmeticOverflow)) }
fn sub(a: u64, b: u64) -> Result<u64, ProgramError> { a.checked_sub(b).ok_or(error(Piv1Error::ArithmeticOverflow)) }

#[allow(clippy::too_many_arguments)]
fn dispatch<'info>(program: &Pubkey, a: &[AccountInfo<'info>], data: &[u8], available: bool,
    get_clock: impl FnOnce() -> Result<Clock, ProgramError>, get_rent: impl FnOnce() -> Result<Rent, ProgramError>,
    mut invoke: impl FnMut(&Instruction, &[AccountInfo<'info>], &[&[&[u8]]]) -> ProgramResult,
    emit: impl FnOnce(SettlementEvent)) -> ProgramResult {
    decode_settle_distribution(data)?;
    if a.len() < 27 { return Err(ProgramError::NotEnoughAccountKeys); }
    if a.len() > 28 { return Err(ProgramError::InvalidArgument); }
    if !available { return Err(ProgramError::Custom(HOST_RUNTIME_UNAVAILABLE_CODE)); }
    let clock = get_clock()?; let rent = get_rent()?;
    distinct(a)?;
    let mut plan = prepare(program, a, &clock, &rent)?;
    if let SettlementEvent::Recovery(_) = plan.event {
        verify(a, &plan.records)?;
        commit_state_writes(program, &rent, [(&plan.writes[1], &a[1])]).map_err(error)?;
        emit(plan.event); return Ok(());
    }
    for step in &plan.steps {
        let infos = step.accounts.iter().map(|i| a[*i].clone()).collect::<Vec<_>>();
        let group = step.signer.iter().map(Vec::as_slice).collect::<Vec<_>>();
        invoke(&step.instruction, &infos, &[&group])?;
        plan.records[step.source].lamports = step.source_after;
        plan.records[step.destination].lamports = step.destination_after;
        verify(a, &plan.records)?;
    }
    let fixed = authenticate_fixed_accounts(program, &rent, roles(a)).map_err(error)?;
    let protocol = protocol_identity(fixed.config(), a, plan.base)?;
    validate_clock(&a[plan.base], &clock)?;
    let observed = fixed.withdrawal_leg_staged_observation(&plan.config, &plan.round).map_err(error)?;
    validate_pool(&protocol, &clock, observed.principal_jitosol_units, observed.pending_jitosol_units)?;
    if observed.amounts().map_err(error)? != economic_custody_obligations(&plan.config, &plan.round).map_err(error)?
        || protected_value(observed, fixed.distribution(), &protocol)? != plan.protected_value {
        return Err(bad());
    }
    commit_state_writes(program, &rent, core::array::from_fn::<_, 8, _>(|i| {
        (&plan.writes[i], &a[if i < 2 { i } else { plan.base + 1 + i }])
    })).map_err(error)?;
    emit(plan.event); Ok(())
}

fn roles<'a, 'info>(a: &'a [AccountInfo<'info>]) -> FixedAccountInfos<'a, 'info> {
    FixedAccountInfos { config: &a[0], active_distribution: &a[1], pending_sol: &a[2],
        principal_sol: &a[3], operational_sol: &a[4], distribution_escrow: &a[5],
        kif_sol: &a[6], principal_jito: &a[7], pending_jito: &a[8] }
}
fn distinct(a: &[AccountInfo<'_>]) -> ProgramResult {
    for (i, left) in a.iter().enumerate() {
        for right in &a[i + 1..] {
            if left.key == right.key || Rc::ptr_eq(&left.data, &right.data)
                || Rc::ptr_eq(&left.lamports, &right.lamports) { return Err(error(Piv1Error::AccountAlias)); }
        }
    }
    Ok(())
}
#[derive(Clone, Debug, PartialEq, Eq)]
struct Record {
    key: Pubkey, owner: Pubkey, signer: bool, writable: bool, executable: bool,
    rent_epoch: u64, lamports: u64, len: usize, hash: [u8; 32],
}
impl Record {
    fn read(a: &AccountInfo<'_>) -> Result<Self, ProgramError> {
        let data = a.try_borrow_data().map_err(|_| error(Piv1Error::AccountBorrowFailed))?;
        Ok(Self { key: *a.key, owner: *a.owner, signer: a.is_signer, writable: a.is_writable,
            executable: a.executable, rent_epoch: a.rent_epoch,
            lamports: **a.try_borrow_lamports().map_err(|_| error(Piv1Error::AccountBorrowFailed))?,
            len: data.len(), hash: hash(&data).to_bytes() })
    }
}
fn verify(a: &[AccountInfo<'_>], records: &[Record]) -> ProgramResult {
    for (a, r) in a.iter().zip(records) { if Record::read(a)? != *r { return Err(bad()); } }
    Ok(())
}
fn validate_clock(a: &AccountInfo<'_>, clock: &Clock) -> ProgramResult {
    if a.key != &sysvar::clock::ID || a.owner != &sysvar::ID || a.executable
        || a.try_borrow_data()?.len() != 40 || Clock::from_account_info(a)? != *clock {
        return Err(error(Piv1Error::InvalidClockAccount));
    }
    Ok(())
}
fn protocol_identity(c: &PivConfig, a: &[AccountInfo<'_>], base: usize)
    -> Result<Box<AuthenticatedJitoIdentity>, ProgramError> {
    authenticate_jito_identity(&DeclaredJitoKeys { program: c.stake_pool_program, pool: c.stake_pool,
        validator_list: c.validator_list, reserve: c.reserve_stake, mint: c.jitosol_mint,
        manager_fee: c.manager_fee_account, referrer: c.referrer_token_account },
        JitoIdentityAccountInfos { program: &a[PROTOCOL], pool: &a[POOL], validator_list: &a[LIST],
            reserve: &a[RESERVE], mint: &a[MINT], manager_fee: &a[MANAGER],
            referrer: &a[if base == 18 { MANAGER } else { 18 }] })
        .map(Box::new).map_err(protocol_program_error)
}
fn validate_pool(p: &AuthenticatedJitoIdentity, clock: &Clock, principal: u64, pending: u64) -> ProgramResult {
    if p.pool().last_update_epoch() != clock.epoch || p.mint().supply > p.pool().pool_token_supply()
        || add(principal, pending)? > p.mint().supply { return Err(bad()); }
    Ok(())
}

/// Only the final pending-use subtraction floors at zero, solely to classify
/// accepted T23-R1 recovery. No custody, liability, transfer or HWM is saturated.
fn protected_value(observed: EconomicCustodyObservation, r: &ActiveDistribution,
    p: &AuthenticatedJitoIdentity) -> Result<u64, ProgramError> {
    let principal = observed.principal_sol.economic_lamports().map_err(error)?;
    let token_value = piv1_math::checked_mul_div_floor(observed.principal_jitosol_units,
        p.pool().total_lamports(), p.pool().pool_token_supply()).map_err(|e| error(e.into()))?;
    let escrow = sub(observed.distribution_escrow.economic_lamports().map_err(error)?, r.cumulative_cooldown_rewards_lamports)?;
    let retained = add(add(principal, token_value)?, escrow)?;
    Ok(if retained < r.pending_sol_used_lamports { 0 } else { retained - r.pending_sol_used_lamports })
}
struct Step {
    instruction: Instruction, accounts: Vec<usize>, signer: Vec<Vec<u8>>,
    source: usize, destination: usize, source_after: u64, destination_after: u64,
}
struct Plan {
    records: Vec<Record>, steps: Vec<Step>, writes: Vec<PreparedStateWrite>,
    config: Box<PivConfig>, round: Box<ActiveDistribution>, base: usize,
    protected_value: u64, event: SettlementEvent,
}
#[inline(never)]
fn rewards(program: &Pubkey, rent: &Rent, a: &[AccountInfo<'_>], start: usize, r: &ActiveDistribution,
    global: u64) -> Result<Box<[GuardianReward; GUARDIAN_COUNT]>, ProgramError> {
    let decoded = core::array::from_fn::<_, GUARDIAN_COUNT, _>(|i|
        authenticate_reward_account(program, rent, &a[start + i]));
    let [a0, a1, a2, a3, a4, a5] = decoded;
    let result = Box::new([a0.map_err(error)?, a1.map_err(error)?, a2.map_err(error)?,
        a3.map_err(error)?, a4.map_err(error)?, a5.map_err(error)?]);
    let mut sum = 0;
    for (i, reward) in result.iter().enumerate() {
        if reward.guardian != r.guardian_keys[i] || reward.guardian_index != i as u8
            || reward.registry_revision != r.guardian_registry_revision { return Err(error(Piv1Error::InvalidGuardianSet)); }
        sum = add(sum, reward.claimable_lamports)?;
    }
    // Current selected records need not enumerate liabilities from other revisions.
    if sum > global { return Err(bad()); }
    Ok(result)
}
fn recipient(a: &AccountInfo<'_>, key: &Pubkey) -> ProgramResult {
    if a.key != key || a.owner != &system_program::ID || a.executable || !a.try_borrow_data()?.is_empty() {
        return Err(error(Piv1Error::InvalidAccountOwner));
    }
    Ok(())
}
fn preflight(a: &[AccountInfo<'_>], base: usize) -> ProgramResult {
    let mut data = Vec::new(); let mut lamports = Vec::new();
    for i in [0, 1, 3, 5, 6, base + 1, base + 2, base + 3, base + 4, base + 5, base + 6, base + 7, base + 8] {
        if !a[i].is_writable { return Err(error(Piv1Error::AccountNotWritable)); }
        data.push(a[i].try_borrow_mut_data().map_err(|_| error(Piv1Error::AccountBorrowFailed))?);
        lamports.push(a[i].try_borrow_mut_lamports().map_err(|_| error(Piv1Error::AccountBorrowFailed))?);
    }
    Ok(())
}
#[inline(never)]
fn prepare(program: &Pubkey, a: &[AccountInfo<'_>], clock: &Clock, rent: &Rent) -> Result<Box<Plan>, ProgramError> {
    let fixed = authenticate_fixed_accounts(program, rent, roles(a)).map_err(error)?;
    let c = fixed.config(); let r = fixed.distribution();
    c.ensure_unpaused().map_err(error)?;
    if r.lifecycle == DistributionLifecycle::Settled || r.settlement_recorded { return Err(error(Piv1Error::SettlementReplay)); }
    if r.lifecycle == DistributionLifecycle::RecoveryRequired { return Err(error(Piv1Error::RecoveryRequired)); }
    if r.lifecycle != DistributionLifecycle::EscrowFunded { return Err(error(Piv1Error::InvalidLifecycle)); }
    let base = if c.manager_fee_account == c.referrer_token_account { 18 } else { 19 };
    if a.len() < base + 9 { return Err(ProgramError::NotEnoughAccountKeys); }
    if a.len() > base + 9 { return Err(ProgramError::InvalidArgument); }
    validate_clock(&a[base], clock)?;
    if clock.unix_timestamp < r.prepared_at || clock.epoch < r.prepared_epoch || clock.slot < r.prepared_slot { return Err(error(Piv1Error::TimestampRegression)); }
    let before = fixed.withdrawal_leg_observation().map_err(error)?;
    if before.amounts().map_err(error)? != economic_custody_obligations(c, r).map_err(error)? { return Err(bad()); }
    let protocol = protocol_identity(c, a, base)?;
    validate_pool(&protocol, clock, before.principal_jitosol_units, before.pending_jitosol_units)?;
    for (i, id) in [(SYSTEM, system_program::ID), (TOKEN, spl_token::ID)] {
        if a[i].key != &id || !a[i].executable { return Err(error(Piv1Error::InvalidProgramIdentity)); }
    }
    if a[AUTHORITY].key != &c.piv_authority || a[AUTHORITY].owner != &system_program::ID
        || a[AUTHORITY].executable || !a[AUTHORITY].try_borrow_data()?.is_empty() { return Err(error(Piv1Error::InvalidAccountPda)); }
    recipient(&a[base + 1], &r.htfp_recipient)?;
    recipient(&a[base + 2], &r.team_owner_recipient)?;
    let old_rewards = rewards(program, rent, a, base + 3, r, c.kif_claim_liability_lamports)?;
    let eligible = add(add(r.pending_sol_used_lamports, r.prior_next_cycle_yield_used_lamports().map_err(error)?)?,
        r.cumulative_finalized_delegated_native_lamports)?;
    let allocation = derive_net_beneficiary_allocation(eligible.min(r.outgoing_gross_obligation_lamports),
        r.htfp_gross_obligation_lamports, r.team_owner_gross_obligation_lamports, r.kif_gross_obligation_lamports).map_err(error)?;
    let zero = match piv1_math::allocate_kif(allocation.kif_lamports, r.kif_carry_input_lamports, r.kif_active_guardian_count)
        .map_err(|e| error(e.into()))? {
        piv1_math::KifAllocation::ActiveGuardians(_) => 0,
        piv1_math::KifAllocation::ZeroActiveGuardians(zero) => zero.compound_from_kif,
    };
    let mut after = before;
    after.distribution_escrow.lamports = sub(after.distribution_escrow.lamports, allocation.allocated_lamports)?;
    after.kif_sol.lamports = sub(add(after.kif_sol.lamports, allocation.kif_lamports)?, zero)?;
    after.principal_sol.lamports = add(after.principal_sol.lamports, zero)?;
    let protected_value = protected_value(after, r, &protocol)?;
    let mut next_c = Box::new(c.clone()); let mut next_r = Box::new(*r); let mut next_rewards = old_rewards.clone();
    // Exactly one model call with actual projected value. A MAX probe would run
    // success-only cumulative checks before an otherwise valid recovery branch.
    let outcome = settle_distribution(&mut next_c, &mut next_r, &mut next_rewards, SettlementInput {
        sequence: r.active_sequence, escrow_available_lamports: before.distribution_escrow.economic_lamports().map_err(error)?,
        validated_post_settlement_protected_value_lamports: protected_value,
    }).map_err(error)?;
    let mut writes = Vec::with_capacity(8);
    writes.push(PreparedStateWrite::new(program, *a[0].key, StateEnvelope::config(c).map_err(error)?,
        StateEnvelope::config(&next_c).map_err(error)?).map_err(error)?);
    writes.push(PreparedStateWrite::new(program, *a[1].key, StateEnvelope::distribution(r).map_err(error)?,
        StateEnvelope::distribution(&next_r).map_err(error)?).map_err(error)?);
    for i in 0..GUARDIAN_COUNT {
        writes.push(PreparedStateWrite::new(program, *a[base + 3 + i].key, StateEnvelope::reward(&old_rewards[i]).map_err(error)?,
            StateEnvelope::reward(&next_rewards[i]).map_err(error)?).map_err(error)?);
    }
    preflight(a, base)?;
    let records = a.iter().map(Record::read).collect::<Result<Vec<_>, _>>()?;
    if outcome == SettlementOutcome::RecoveryRequired {
        return Ok(Box::new(Plan { records, steps: vec![], writes, config: next_c, round: next_r,
            base, protected_value, event: SettlementEvent::Recovery(DistributionSettlementRecovery {
                config: *a[0].key, sequence: r.active_sequence, observed_protected_value_lamports: protected_value,
                recovery_flags: crate::constants::RECOVERY_FLAG_RESIDUAL_HWM }) }));
    }
    if next_r.actual_htfp_lamports != allocation.htfp_lamports || next_r.actual_team_owner_lamports != allocation.team_owner_lamports
        || next_r.actual_kif_allocation_lamports != allocation.kif_lamports || next_r.actual_zero_active_kif_compound_lamports != zero
        || after.amounts().map_err(error)? != economic_custody_obligations(&next_c, &next_r).map_err(error)? { return Err(bad()); }
    let mut expected = records.clone(); let mut steps = Vec::with_capacity(4);
    for (source, destination, amount, seed, bump) in [
        (5, base + 1, allocation.htfp_lamports, seeds::DISTRIBUTION_ESCROW, c.bumps.distribution_escrow),
        (5, base + 2, allocation.team_owner_lamports, seeds::DISTRIBUTION_ESCROW, c.bumps.distribution_escrow),
        (5, 6, allocation.kif_lamports, seeds::DISTRIBUTION_ESCROW, c.bumps.distribution_escrow),
        (6, 3, zero, seeds::KIF_SOL, c.bumps.kif_sol_vault),
    ] {
        if amount == 0 { continue; }
        expected[source].lamports = sub(expected[source].lamports, amount)?;
        expected[destination].lamports = add(expected[destination].lamports, amount)?;
        let floor = rent_floor(rent, 0).map_err(error)?;
        if expected[source].lamports < floor || expected[destination].lamports < floor { return Err(error(Piv1Error::AccountRentDeficit)); }
        steps.push(Step { instruction: system_instruction::transfer(a[source].key, a[destination].key, amount),
            accounts: vec![source, destination, SYSTEM], signer: vec![seed.to_vec(), vec![bump]],
            source, destination, source_after: expected[source].lamports, destination_after: expected[destination].lamports });
    }
    let event = SettlementEvent::Settled(DistributionSettled { config: *a[0].key, sequence: r.active_sequence,
        htfp_lamports: next_r.actual_htfp_lamports, team_owner_lamports: next_r.actual_team_owner_lamports,
        kif_allocation_lamports: next_r.actual_kif_allocation_lamports, kif_liability_lamports: next_r.actual_kif_liability_lamports,
        kif_carry_lamports: next_r.actual_kif_carry_next_lamports, zero_active_compound_lamports: zero,
        protected_hwm_lamports: next_r.settled_protected_hwm_lamports });
    Ok(Box::new(Plan { records, steps, writes, config: next_c, round: next_r, base, protected_value, event }))
}
