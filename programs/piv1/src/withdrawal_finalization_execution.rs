//! Authoritative inactive Stake withdrawal, exact rent recovery and atomic
//! metadata closure. Actual pinned Stake success proves current inactivity;
//! host callbacks model effects, not runtime readiness or rollback.
use std::rc::Rc;
use anchor_lang::{
    prelude::{AccountInfo, Clock, Pubkey, Rent, SolanaSysvar},
    AnchorDeserialize,
    solana_program::{entrypoint::ProgramResult, instruction::{AccountMeta, Instruction},
        program::invoke_signed, program_error::ProgramError, system_instruction,
        system_program, sysvar},
};
use solana_sha256_hasher::hash;
use solana_stake_interface::state::{Authorized, StakeStateV2};
use crate::{
    accounts::{authenticate_fixed_accounts, seeds, FixedAccountInfos, STAKE_PROGRAM_ID},
    errors::Piv1Error,
    events::WithdrawalLegFinalized,
    instruction_errors::{piv1_error_code, protocol_program_error, HOST_RUNTIME_UNAVAILABLE_CODE},
    instructions::finalize_withdrawal_leg::decode_finalize_withdrawal_leg,
    integrations::jito_identity::{authenticate_jito_identity, AuthenticatedJitoIdentity,
        DeclaredJitoKeys, JitoIdentityAccountInfos},
    state::{finalize_withdrawal_leg, reconcile_pending_contributions, ActiveDistribution,
        DistributionLifecycle, LegFinalizationInput, PivConfig, WithdrawalLeg, WithdrawalLegStatus,
        reconciliation::economic_custody_obligations},
    state_persistence::{commit_state_writes, PreparedStateWrite, StateEnvelope},
};

const AUTHORITY: usize = 9;
const MINT: usize = 10;
const SYSTEM: usize = 11;
const TOKEN: usize = 12;
const PROTOCOL: usize = 13;
const POOL: usize = 14;
const LIST: usize = 15;
const RESERVE: usize = 16;
const MANAGER: usize = 17;

pub fn process_instruction(program: &Pubkey, a: &[AccountInfo<'_>], data: &[u8]) -> ProgramResult {
    dispatch(program, a, data, cfg!(target_os = "solana"), Clock::get, Rent::get,
        |ix, infos, seeds| invoke_signed(ix, infos, seeds), |e| anchor_lang::emit!(e))
}

/// Explicit host effects seam. A callback must model the pinned Stake instruction;
/// production executes it against actual runtime Clock/Stake History. Errors keep
/// partial modeled CPI effects until the test transaction discards that world.
#[cfg(not(target_os = "solana"))]
pub fn process_instruction_with_host_callbacks<'info>(
    program: &Pubkey, a: &[AccountInfo<'info>], data: &[u8],
    clock: impl FnOnce() -> Result<Clock, ProgramError>,
    rent: impl FnOnce() -> Result<Rent, ProgramError>,
    invoke: impl FnMut(&Instruction, &[AccountInfo<'info>], &[&[&[u8]]]) -> ProgramResult,
    emit: impl FnOnce(WithdrawalLegFinalized),
) -> ProgramResult {
    dispatch(program, a, data, true, clock, rent, invoke, emit)
}

fn error(e: Piv1Error) -> ProgramError { ProgramError::Custom(piv1_error_code(e)) }
fn bad() -> ProgramError { error(Piv1Error::CumulativeReconciliationMismatch) }
fn add(a: u64, b: u64) -> Result<u64, ProgramError> {
    a.checked_add(b).ok_or(error(Piv1Error::ArithmeticOverflow))
}
fn sub(a: u64, b: u64) -> Result<u64, ProgramError> {
    a.checked_sub(b).ok_or(error(Piv1Error::ArithmeticOverflow))
}

#[allow(clippy::too_many_arguments)]
fn dispatch<'info>(program: &Pubkey, a: &[AccountInfo<'info>], data: &[u8], available: bool,
    get_clock: impl FnOnce() -> Result<Clock, ProgramError>,
    get_rent: impl FnOnce() -> Result<Rent, ProgramError>,
    mut invoke: impl FnMut(&Instruction, &[AccountInfo<'info>], &[&[&[u8]]]) -> ProgramResult,
    emit: impl FnOnce(WithdrawalLegFinalized),
) -> ProgramResult {
    let index = decode_finalize_withdrawal_leg(data)?;
    if a.len() < 23 { return Err(ProgramError::NotEnoughAccountKeys); }
    if a.len() > 24 { return Err(ProgramError::InvalidArgument); }
    if !available { return Err(ProgramError::Custom(HOST_RUNTIME_UNAVAILABLE_CODE)); }
    let clock = get_clock()?;
    let rent = get_rent()?;
    distinct(a)?;
    let mut plan = prepare(program, a, index, &clock, &rent)?;
    for step in &plan.steps {
        let infos = step.accounts.iter().map(|i| a[*i].clone()).collect::<Vec<_>>();
        let group = step.signer.iter().map(Vec::as_slice).collect::<Vec<_>>();
        invoke(&step.instruction, &infos, &[&group])?;
        for (i, record) in &step.changes { plan.records[*i] = record.clone(); }
        verify(a, &plan.records)?;
    }
    // Reauthenticate all current protocol/custody accounts after both CPIs.
    // Metadata donation is the only remaining direct lamport movement; include
    // its exact prechecked credit when checking the staged economic obligations.
    let fixed = authenticate_fixed_accounts(program, &rent, roles(a)).map_err(error)?;
    let protocol = protocol_identity(fixed.config(), a, plan.base)?;
    validate_clock(&a[plan.base], &clock)?;
    let mut observed = fixed.withdrawal_leg_observation().map_err(error)?;
    observed.pending_sol.lamports = add(observed.pending_sol.lamports, plan.metadata_excess)?;
    if observed.amounts().map_err(error)? != economic_custody_obligations(&plan.config, &plan.round).map_err(error)? {
        return Err(bad());
    }
    validate_pool(&protocol, &clock, observed.principal_jitosol_units, observed.pending_jitosol_units)?;
    if residual(fixed.config(), fixed.distribution(), &protocol)? != plan.residual { return Err(bad()); }
    commit_and_close(program, a, &rent, &plan)?;
    emit(plan.event);
    Ok(())
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
fn validate_history(a: &AccountInfo<'_>, clock: &Clock) -> ProgramResult {
    if a.key != &sysvar::stake_history::ID || a.owner != &sysvar::ID || a.executable {
        return Err(ProgramError::InvalidArgument);
    }
    // Canonical bounded bincode vector, potentially padded to the sysvar's
    // maximum allocation. The pinned callee uses runtime history, not a caller
    // readiness flag or this program's older interface activation algorithm.
    let data = a.try_borrow_data()?;
    if data.len() < 8 || data.len() > 8 + 512 * 32 { return Err(ProgramError::InvalidAccountData); }
    let count = u64::from_le_bytes(data[..8].try_into().map_err(|_| bad())?);
    if count > 512 { return Err(ProgramError::InvalidAccountData); }
    let end = 8 + count as usize * 32;
    if end > data.len() || data[end..].iter().any(|b| *b != 0) { return Err(ProgramError::InvalidAccountData); }
    let mut previous = clock.epoch;
    for entry in data[8..end].chunks_exact(32) {
        let epoch = u64::from_le_bytes(entry[..8].try_into().map_err(|_| bad())?);
        if epoch >= previous { return Err(ProgramError::InvalidAccountData); }
        previous = epoch;
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
fn residual(c: &PivConfig, r: &ActiveDistribution, p: &AuthenticatedJitoIdentity) -> Result<u64, ProgramError> {
    let units = sub(c.accounted_historical_jitosol_units, r.fixed_jitosol_withdrawal_target_units)?;
    let value = piv1_math::checked_mul_div_floor(units, p.pool().total_lamports(), p.pool().pool_token_supply())
        .map_err(|e| error(e.into()))?;
    let carry = sub(r.prior_next_cycle_yield_lamports, r.prior_next_cycle_yield_used_lamports().map_err(error)?)?;
    add(add(c.accounted_historical_sol_lamports, value)?, carry)
}
struct Step { instruction: Instruction, accounts: Vec<usize>, signer: Vec<Vec<u8>>, changes: Vec<(usize, Record)> }
struct Plan {
    records: Vec<Record>, steps: Vec<Step>, writes: [PreparedStateWrite; 2],
    config: Box<PivConfig>, round: Box<ActiveDistribution>, base: usize,
    metadata_before: StateEnvelope, metadata_rent: u64, metadata_excess: u64,
    residual: u64, event: WithdrawalLegFinalized,
}
fn signer(seed: &[u8], bump: u8) -> Vec<Vec<u8>> { vec![seed.to_vec(), vec![bump]] }

#[inline(never)]
fn read_leg(a: &AccountInfo<'_>, program: &Pubkey, sequence: u64, index: u64)
    -> Result<Box<WithdrawalLeg>, ProgramError> {
    if a.owner != program || a.executable { return Err(error(Piv1Error::InvalidAccountOwner)); }
    let bytes = a.try_borrow_data()?;
    if bytes.len() != WithdrawalLeg::SPACE { return Err(error(Piv1Error::InvalidAccountSize)); }
    let leg = Box::new(WithdrawalLeg::deserialize(&mut &bytes[8..]).map_err(|_| bad())?);
    let envelope = StateEnvelope::withdrawal_leg(&leg).map_err(error)?;
    if envelope.as_bytes() != *bytes { return Err(error(Piv1Error::InvalidAccountDiscriminator)); }
    if leg.sequence != sequence { return Err(error(Piv1Error::SequenceMismatch)); }
    if leg.leg_index != index { return Err(error(Piv1Error::LegIndexMismatch)); }
    // Typed identity validates both canonical bumps and the metadata address.
    let _ = PreparedStateWrite::new(program, *a.key, envelope,
        StateEnvelope::withdrawal_leg(&leg).map_err(error)?).map_err(error)?;
    if leg.status != WithdrawalLegStatus::Initiated { return Err(error(Piv1Error::AlreadyFinalized)); }
    Ok(leg)
}

#[inline(never)]
fn prepare(program: &Pubkey, a: &[AccountInfo<'_>], index: u64, clock: &Clock, rent: &Rent)
    -> Result<Box<Plan>, ProgramError> {
    let fixed = authenticate_fixed_accounts(program, rent, roles(a)).map_err(error)?;
    let c = fixed.config();
    let r = fixed.distribution();
    c.ensure_unpaused().map_err(error)?;
    if r.lifecycle == DistributionLifecycle::RecoveryRequired { return Err(error(Piv1Error::RecoveryRequired)); }
    if r.lifecycle != DistributionLifecycle::WithdrawalActive { return Err(error(Piv1Error::InvalidLifecycle)); }
    let base = if c.manager_fee_account == c.referrer_token_account { 18 } else { 19 };
    if a.len() < base + 5 { return Err(ProgramError::NotEnoughAccountKeys); }
    if a.len() > base + 5 { return Err(ProgramError::InvalidArgument); }
    let before = fixed.withdrawal_leg_observation().map_err(error)?;
    if before.amounts().map_err(error)? != economic_custody_obligations(c, r).map_err(error)? { return Err(bad()); }
    let protocol = protocol_identity(c, a, base)?;
    validate_pool(&protocol, clock, before.principal_jitosol_units, before.pending_jitosol_units)?;
    validate_clock(&a[base], clock)?;
    validate_history(&a[base + 2], clock)?;
    for (i, id) in [(SYSTEM, system_program::ID), (TOKEN, spl_token::ID), (base + 1, STAKE_PROGRAM_ID)] {
        if a[i].key != &id || !a[i].executable { return Err(error(Piv1Error::InvalidProgramIdentity)); }
    }
    if a[AUTHORITY].key != &c.piv_authority || a[AUTHORITY].owner != &system_program::ID
        || a[AUTHORITY].executable || !a[AUTHORITY].try_borrow_data()?.is_empty() {
        return Err(error(Piv1Error::InvalidAccountPda));
    }
    let mut leg = read_leg(&a[base + 3], program, r.active_sequence, index)?;
    let metadata_before = StateEnvelope::withdrawal_leg(&leg).map_err(error)?;
    let (stake_key, bump) = Pubkey::try_find_program_address(&[seeds::WITHDRAWAL_STAKE,
        &leg.sequence.to_le_bytes(), &index.to_le_bytes()], program).ok_or(error(Piv1Error::InvalidAccountPda))?;
    if a[base + 4].key != &stake_key || leg.stake_bump != bump { return Err(error(Piv1Error::InvalidAccountPda)); }
    let stake_data = a[base + 4].try_borrow_data()?;
    if a[base + 4].owner != &STAKE_PROGRAM_ID || a[base + 4].executable || stake_data.len() != 200 { return Err(bad()); }
    let StakeStateV2::Stake(meta, stake, _) = StakeStateV2::deserialize(&mut &stake_data[..]).map_err(|_| bad())?
        else { return Err(bad()); };
    if meta.authorized != (Authorized { staker: c.piv_authority, withdrawer: c.piv_authority })
        || stake.delegation.voter_pubkey != leg.validator_vote
        || stake.delegation.deactivation_epoch == u64::MAX
        || stake.delegation.deactivation_epoch != leg.initiation_epoch
        || clock.epoch < leg.initiation_epoch || meta.lockup.is_in_force(clock, None) {
        return Err(bad());
    }
    drop(stake_data);
    let records = a.iter().map(Record::read).collect::<Result<Vec<_>, _>>()?;
    let native = records[base + 4].lamports;
    let stake_rent = leg.stake_rent_advanced_lamports;
    let metadata_rent = leg.metadata_rent_advanced_lamports;
    let net = native.checked_sub(stake_rent).ok_or(error(Piv1Error::InvalidCustodyObservation))?;
    // Do not invent the provenance of B > original rent + current delegation:
    // native donations and runtime rent-adjustment history can be ambiguous.
    // Supported deficits are actual observed value losses, never HWM resets.
    if net > stake.delegation.stake { return Err(error(Piv1Error::InvalidCustodyObservation)); }
    let metadata_excess = records[base + 3].lamports.checked_sub(metadata_rent)
        .ok_or(error(Piv1Error::AccountRentDeficit))?;
    let reward = net.saturating_sub(leg.observed_delegated_native_lamports);
    let loss = leg.observed_delegated_native_lamports.saturating_sub(net);
    let residual = residual(c, r, &protocol)?;
    let mut next_c = Box::new(c.clone());
    let mut next_r = Box::new(*r);
    let mut after = before;
    after.pending_sol.lamports = add(after.pending_sol.lamports, metadata_excess)?;
    after.distribution_escrow.lamports = add(after.distribution_escrow.lamports, net)?;
    reconcile_pending_contributions(&mut next_c, r, after.pending()).map_err(error)?;
    finalize_withdrawal_leg(&next_c, &mut next_r, &mut leg, LegFinalizationInput {
        sequence: r.active_sequence, leg_index: index, finalized_epoch: clock.epoch,
        finalized_native_lamports: native, recovered_stake_rent_lamports: stake_rent,
        recovered_metadata_rent_lamports: metadata_rent, cooldown_reward_lamports: reward,
        cooldown_loss_lamports: loss, validated_residual_historical_value_lamports: residual,
        escrow_available_after_lamports: after.distribution_escrow.economic_lamports().map_err(error)?,
    }).map_err(error)?;
    if after.amounts().map_err(error)? != economic_custody_obligations(&next_c, &next_r).map_err(error)? { return Err(bad()); }
    let writes = [PreparedStateWrite::new(program, *a[0].key, StateEnvelope::config(c).map_err(error)?,
        StateEnvelope::config(&next_c).map_err(error)?).map_err(error)?,
        PreparedStateWrite::new(program, *a[1].key, StateEnvelope::distribution(r).map_err(error)?,
        StateEnvelope::distribution(&next_r).map_err(error)?).map_err(error)?];
    // All overflow and borrow failures are checked before the first effect.
    let _ = add(add(records[4].lamports, stake_rent)?, metadata_rent)?;
    preflight_borrows(a, base)?;
    let mut expected = records.clone();
    expected[base + 4].lamports = 0;
    expected[base + 4].len = 0;
    expected[base + 4].hash = hash(&[]).to_bytes();
    expected[5].lamports = add(expected[5].lamports, native)?;
    // Pinned Stake Withdraw bincode variant 4. Legacy account positions remain
    // exact even though Stake 5.1 obtains Clock/History via runtime syscalls.
    let mut data = 4_u32.to_le_bytes().to_vec();
    data.extend(native.to_le_bytes());
    let first = Step { instruction: Instruction { program_id: STAKE_PROGRAM_ID, data, accounts: vec![
        AccountMeta::new(stake_key, false), AccountMeta::new(c.distribution_escrow, false),
        AccountMeta::new_readonly(*a[base].key, false), AccountMeta::new_readonly(*a[base + 2].key, false),
        AccountMeta::new_readonly(c.piv_authority, true)] },
        accounts: vec![base + 4, 5, base, base + 2, AUTHORITY, base + 1],
        signer: signer(seeds::AUTHORITY, c.bumps.piv_authority),
        changes: vec![(base + 4, expected[base + 4].clone()), (5, expected[5].clone())] };
    expected[5].lamports = sub(expected[5].lamports, stake_rent)?;
    expected[4].lamports = add(expected[4].lamports, stake_rent)?;
    let second = Step { instruction: system_instruction::transfer(a[5].key, a[4].key, stake_rent),
        accounts: vec![5, 4, SYSTEM], signer: signer(seeds::DISTRIBUTION_ESCROW, c.bumps.distribution_escrow),
        changes: vec![(5, expected[5].clone()), (4, expected[4].clone())] };
    Ok(Box::new(Plan { records, steps: vec![first, second], writes, config: next_c, round: next_r,
        base, metadata_before, metadata_rent, metadata_excess, residual,
        event: WithdrawalLegFinalized { config: *a[0].key, sequence: leg.sequence, leg_index: index,
            finalized_native_lamports: native, recovered_stake_rent_lamports: stake_rent,
            recovered_metadata_rent_lamports: metadata_rent, cooldown_reward_lamports: reward,
            cooldown_loss_lamports: loss, normalized_metadata_excess_lamports: metadata_excess,
            recovery_flags: leg.recovery_flags } }))
}

fn preflight_borrows(a: &[AccountInfo<'_>], base: usize) -> ProgramResult {
    let mut data = Vec::new();
    let mut lamports = Vec::new();
    for i in [0, 1, 2, 4, 5, base + 3, base + 4] {
        if !a[i].is_writable { return Err(error(Piv1Error::AccountNotWritable)); }
        data.push(a[i].try_borrow_mut_data().map_err(|_| error(Piv1Error::AccountBorrowFailed))?);
        lamports.push(a[i].try_borrow_mut_lamports().map_err(|_| error(Piv1Error::AccountBorrowFailed))?);
    }
    Ok(())
}

/// Existing typed persistence retains both state borrows before copying. Hold
/// every closure borrow concurrently: after state commit only infallible writes
/// remain. Zeroed metadata cannot be revived as an initiated leg by refunding it;
/// actual zero-lamport account purge occurs at the transaction boundary.
fn commit_and_close(program: &Pubkey, a: &[AccountInfo<'_>], rent: &Rent, plan: &Plan) -> ProgramResult {
    verify(a, &plan.records)?;
    let metadata = &a[plan.base + 3];
    if metadata.owner != program || !metadata.is_writable || metadata.executable { return Err(bad()); }
    let mut data = metadata.try_borrow_mut_data().map_err(|_| error(Piv1Error::AccountBorrowFailed))?;
    if *data != plan.metadata_before.as_bytes() { return Err(error(Piv1Error::StateEnvelopeChanged)); }
    let mut source = metadata.try_borrow_mut_lamports().map_err(|_| error(Piv1Error::AccountBorrowFailed))?;
    let mut operations = a[4].try_borrow_mut_lamports().map_err(|_| error(Piv1Error::AccountBorrowFailed))?;
    let mut pending = a[2].try_borrow_mut_lamports().map_err(|_| error(Piv1Error::AccountBorrowFailed))?;
    if **source != add(plan.metadata_rent, plan.metadata_excess)? { return Err(bad()); }
    let op_after = add(**operations, plan.metadata_rent)?;
    let pending_after = add(**pending, plan.metadata_excess)?;
    commit_state_writes(program, rent, [(&plan.writes[0], &a[0]), (&plan.writes[1], &a[1])]).map_err(error)?;
    data.fill(0);
    **source = 0;
    **operations = op_after;
    **pending = pending_after;
    Ok(())
}
