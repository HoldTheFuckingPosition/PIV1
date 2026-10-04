//! Atomic post-settlement pending integration using current official book value.
//! Full contribution P enters HWM; physical pending P-U and escrow move once.
use std::rc::Rc;
use anchor_lang::{prelude::{AccountInfo, Clock, Pubkey, Rent, SolanaSysvar},
    solana_program::{entrypoint::ProgramResult, instruction::Instruction, program::invoke_signed,
        program_error::ProgramError, system_instruction, system_program, sysvar}};
use solana_sha256_hasher::hash;
use crate::{accounts::{authenticate_fixed_accounts, seeds, FixedAccountInfos},
    errors::{Piv1Error, Piv1Result}, events::PendingContributionsIntegrated,
    instruction_errors::{piv1_error_code, protocol_program_error, HOST_RUNTIME_UNAVAILABLE_CODE},
    instructions::integrate_pending::decode_integrate_pending,
    integrations::jito_identity::{authenticate_jito_identity, AuthenticatedJitoIdentity,
        DeclaredJitoKeys, JitoIdentityAccountInfos},
    state::{integrate_pending_and_complete, ActiveDistribution, DistributionLifecycle, PivConfig,
        reconciliation::{derive_pending_integration_with_valuation, economic_custody_obligations}},
    state_persistence::{commit_state_writes, PreparedStateWrite, StateEnvelope}};
const AUTHORITY: usize = 9;
const MINT: usize = 10;
const SYSTEM: usize = 11;
const TOKEN: usize = 12;

pub fn process_instruction(program: &Pubkey, a: &[AccountInfo<'_>], data: &[u8]) -> ProgramResult {
    dispatch(program, a, data, cfg!(target_os = "solana"), Clock::get, Rent::get,
        |ix, infos, seeds| invoke_signed(ix, infos, seeds), |e| anchor_lang::emit!(e))
}
/// Host effects and explicit failed-world discard only; no signature or real
/// transaction rollback proof, and no production callback/backend selector.
#[cfg(not(target_os = "solana"))]
pub fn process_instruction_with_host_callbacks<'info>(program: &Pubkey, a: &[AccountInfo<'info>], data: &[u8],
    clock: impl FnOnce() -> Result<Clock, ProgramError>, rent: impl FnOnce() -> Result<Rent, ProgramError>,
    invoke: impl FnMut(&Instruction, &[AccountInfo<'info>], &[&[&[u8]]]) -> ProgramResult,
    emit: impl FnOnce(PendingContributionsIntegrated)) -> ProgramResult {
    dispatch(program, a, data, true, clock, rent, invoke, emit)
}
fn error(e: Piv1Error) -> ProgramError { ProgramError::Custom(piv1_error_code(e)) }
fn bad() -> ProgramError { error(Piv1Error::ContributionObservationMismatch) }
fn add(a: u64, b: u64) -> Result<u64, ProgramError> { a.checked_add(b).ok_or(error(Piv1Error::ArithmeticOverflow)) }

#[allow(clippy::too_many_arguments)]
fn dispatch<'info>(program: &Pubkey, a: &[AccountInfo<'info>], data: &[u8], available: bool,
    get_clock: impl FnOnce() -> Result<Clock, ProgramError>, get_rent: impl FnOnce() -> Result<Rent, ProgramError>,
    mut invoke: impl FnMut(&Instruction, &[AccountInfo<'info>], &[&[&[u8]]]) -> ProgramResult,
    emit: impl FnOnce(PendingContributionsIntegrated)) -> ProgramResult {
    decode_integrate_pending(data)?;
    if a.len() < 19 { return Err(ProgramError::NotEnoughAccountKeys); }
    if a.len() > 20 { return Err(ProgramError::InvalidArgument); }
    if !available { return Err(ProgramError::Custom(HOST_RUNTIME_UNAVAILABLE_CODE)); }
    let clock = get_clock()?; let rent = get_rent()?;
    distinct(a)?;
    let mut plan = prepare(program, a, &clock, &rent)?;
    preflight(a, &plan.steps)?;
    for step in &plan.steps {
        let infos = step.accounts.iter().map(|i| a[*i].clone()).collect::<Vec<_>>();
        let bump = [step.bump]; let signer = [step.seed, &bump];
        invoke(&step.instruction, &infos, &[&signer])?;
        plan.records[step.source] = step.source_after.clone();
        plan.records[step.destination] = step.destination_after.clone();
        verify(a, &plan.records)?;
    }
    verify(a, &plan.records)?;
    let fixed = authenticate_fixed_accounts(program, &rent, roles(a)).map_err(error)?;
    let protocol = protocol_identity(fixed.config(), a, plan.base)?;
    validate_clock(&a[plan.base], &clock)?;
    let after = fixed.pending_integration_staged_observation(&plan.config, &plan.round).map_err(error)?;
    validate_pool(&protocol, &clock, after.principal_jitosol_units, after.pending_jitosol_units)?;
    if after.amounts().map_err(error)? != economic_custody_obligations(&plan.config, &plan.round).map_err(error)? {
        return Err(bad());
    }
    let protected = add(plan.config.accounted_historical_sol_lamports,
        current_book_value(&protocol, plan.config.accounted_historical_jitosol_units).map_err(error)?)?;
    if protected < plan.config.protected_principal_hwm_lamports { return Err(error(Piv1Error::HighWaterMarkDecrease)); }
    commit_state_writes(program, &rent, [(&plan.writes[0], &a[0]), (&plan.writes[1], &a[1])]).map_err(error)?;
    emit(plan.event); Ok(())
}
fn roles<'a, 'info>(a: &'a [AccountInfo<'info>]) -> FixedAccountInfos<'a, 'info> {
    FixedAccountInfos { config: &a[0], active_distribution: &a[1], pending_sol: &a[2], principal_sol: &a[3],
        operational_sol: &a[4], distribution_escrow: &a[5], kif_sol: &a[6], principal_jito: &a[7], pending_jito: &a[8] }
}
fn distinct(a: &[AccountInfo<'_>]) -> ProgramResult {
    for (i, left) in a.iter().enumerate() { for right in &a[i + 1..] {
        if left.key == right.key || Rc::ptr_eq(&left.data, &right.data) || Rc::ptr_eq(&left.lamports, &right.lamports) {
            return Err(error(Piv1Error::AccountAlias));
        }
    }} Ok(())
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
    for (a, r) in a.iter().zip(records) { if Record::read(a)? != *r { return Err(bad()); } } Ok(())
}
fn validate_clock(a: &AccountInfo<'_>, clock: &Clock) -> ProgramResult {
    if a.key != &sysvar::clock::ID || a.owner != &sysvar::ID || a.executable
        || a.try_borrow_data()?.len() != 40 || Clock::from_account_info(a)? != *clock {
        return Err(error(Piv1Error::InvalidClockAccount));
    } Ok(())
}
fn protocol_identity(c: &PivConfig, a: &[AccountInfo<'_>], base: usize)
    -> Result<Box<AuthenticatedJitoIdentity>, ProgramError> {
    authenticate_jito_identity(&DeclaredJitoKeys { program: c.stake_pool_program, pool: c.stake_pool,
        validator_list: c.validator_list, reserve: c.reserve_stake, mint: c.jitosol_mint,
        manager_fee: c.manager_fee_account, referrer: c.referrer_token_account },
        JitoIdentityAccountInfos { program: &a[13], pool: &a[14], validator_list: &a[15], reserve: &a[16],
            mint: &a[MINT], manager_fee: &a[17], referrer: &a[if base == 18 { 17 } else { 18 }] })
        .map(Box::new).map_err(protocol_program_error)
}
fn validate_pool(p: &AuthenticatedJitoIdentity, clock: &Clock, principal: u64, pending: u64) -> ProgramResult {
    let pool = p.pool();
    if pool.last_update_epoch() != clock.epoch || (pool.total_lamports() == 0) != (pool.pool_token_supply() == 0)
        || p.mint().supply > pool.pool_token_supply() || add(principal, pending)? > p.mint().supply {
        return Err(error(Piv1Error::InvalidCustodyObservation));
    } Ok(())
}
/// Validation of the aggregate holdings and pool pair precedes this function.
/// Official stored supply remains the denominator during legitimate Mint burn lag.
fn current_book_value(p: &AuthenticatedJitoIdentity, units: u64) -> Piv1Result<u64> {
    if units > p.mint().supply { return Err(Piv1Error::InvalidCustodyObservation); }
    if p.pool().pool_token_supply() == 0 { return Ok(0); }
    piv1_math::checked_mul_div_floor(units, p.pool().total_lamports(), p.pool().pool_token_supply()).map_err(Into::into)
}
struct Step {
    instruction: Instruction, accounts: Vec<usize>, source: usize, destination: usize,
    seed: &'static [u8], bump: u8, source_after: Record, destination_after: Record,
}
struct Plan {
    records: Vec<Record>, steps: Vec<Step>, writes: Vec<PreparedStateWrite>,
    config: Box<PivConfig>, round: Box<ActiveDistribution>, base: usize, event: PendingContributionsIntegrated,
}
fn preflight(a: &[AccountInfo<'_>], steps: &[Step]) -> ProgramResult {
    let mut mutable = [false; 20]; mutable[0] = true; mutable[1] = true;
    for step in steps { mutable[step.source] = true; mutable[step.destination] = true; }
    let mut data = Vec::new(); let mut lamports = Vec::new();
    for (i, account) in a.iter().enumerate() { if mutable[i] {
        if !account.is_writable { return Err(error(Piv1Error::AccountNotWritable)); }
        data.push(account.try_borrow_mut_data().map_err(|_| error(Piv1Error::AccountBorrowFailed))?);
        lamports.push(account.try_borrow_mut_lamports().map_err(|_| error(Piv1Error::AccountBorrowFailed))?);
    }}
    Ok(())
}
#[inline(never)]
fn prepare(program: &Pubkey, a: &[AccountInfo<'_>], clock: &Clock, rent: &Rent) -> Result<Box<Plan>, ProgramError> {
    let fixed = authenticate_fixed_accounts(program, rent, roles(a)).map_err(error)?;
    let c = fixed.config(); let r = fixed.distribution(); c.ensure_unpaused().map_err(error)?;
    if r.lifecycle == DistributionLifecycle::RecoveryRequired { return Err(error(Piv1Error::RecoveryRequired)); }
    if r.lifecycle != DistributionLifecycle::Settled { return Err(error(Piv1Error::InvalidLifecycle)); }
    let base = if c.manager_fee_account == c.referrer_token_account { 18 } else { 19 };
    if a.len() < base + 1 { return Err(ProgramError::NotEnoughAccountKeys); }
    if a.len() > base + 1 { return Err(ProgramError::InvalidArgument); }
    validate_clock(&a[base], clock)?;
    if clock.unix_timestamp < r.prepared_at || clock.epoch < r.prepared_epoch || clock.slot < r.prepared_slot {
        return Err(error(Piv1Error::TimestampRegression));
    }
    let before = fixed.pending_integration_observation().map_err(error)?;
    if before.amounts().map_err(error)? != economic_custody_obligations(c, r).map_err(error)? {
        return Err(error(Piv1Error::InvalidCustodyObservation));
    }
    let protocol = protocol_identity(c, a, base)?;
    validate_pool(&protocol, clock, before.principal_jitosol_units, before.pending_jitosol_units)?;
    for (i, id) in [(SYSTEM, system_program::ID), (TOKEN, spl_token::ID)] {
        if a[i].key != &id || !a[i].executable { return Err(error(Piv1Error::InvalidProgramIdentity)); }
    }
    if a[AUTHORITY].key != &c.piv_authority || a[AUTHORITY].owner != &system_program::ID
        || a[AUTHORITY].executable || !a[AUTHORITY].try_borrow_data()?.is_empty() { return Err(error(Piv1Error::InvalidAccountPda)); }
    let sol = before.pending_sol.economic_lamports().map_err(error)?;
    let escrow = before.distribution_escrow.economic_lamports().map_err(error)?;
    let tokens = before.pending_jitosol_units;
    let mut after = before;
    after.pending_sol.lamports -= sol; after.distribution_escrow.lamports -= escrow;
    after.principal_sol.lamports = add(add(after.principal_sol.lamports, sol)?, escrow)?;
    after.pending_jitosol_units = 0; after.principal_jitosol_units = add(after.principal_jitosol_units, tokens)?;
    let input = derive_pending_integration_with_valuation(c, r, clock.unix_timestamp, before, after,
        |units| current_book_value(&protocol, units)).map_err(error)?;
    let mut next_c = Box::new(c.clone()); let mut next_r = Box::new(*r);
    let summary = integrate_pending_and_complete(&mut next_c, &mut next_r, input).map_err(error)?;
    let writes = vec![PreparedStateWrite::new(program, *a[0].key, StateEnvelope::config(c).map_err(error)?,
        StateEnvelope::config(&next_c).map_err(error)?).map_err(error)?,
        PreparedStateWrite::new(program, *a[1].key, StateEnvelope::distribution(r).map_err(error)?,
        StateEnvelope::distribution(&next_r).map_err(error)?).map_err(error)?];
    let records = a.iter().map(Record::read).collect::<Result<Vec<_>, _>>()?;
    let mut expected = records.clone(); let mut steps = Vec::with_capacity(3);
    for (source, amount, seed, bump) in [(2, sol, seeds::PENDING_SOL, c.bumps.pending_sol_vault),
        (5, escrow, seeds::DISTRIBUTION_ESCROW, c.bumps.distribution_escrow)] {
        if amount == 0 { continue; }
        expected[source].lamports -= amount; expected[3].lamports = add(expected[3].lamports, amount)?;
        steps.push(Step { instruction: system_instruction::transfer(a[source].key, a[3].key, amount),
            accounts: vec![source, 3, SYSTEM], source, destination: 3, seed, bump,
            source_after: expected[source].clone(), destination_after: expected[3].clone() });
    }
    if tokens != 0 {
        for (i, units) in [(8, 0), (7, after.principal_jitosol_units)] {
            let mut bytes = a[i].try_borrow_data()?.to_vec(); bytes[64..72].copy_from_slice(&units.to_le_bytes());
            expected[i].hash = hash(&bytes).to_bytes();
        }
        steps.push(Step { instruction: spl_token::instruction::transfer_checked(&spl_token::ID,
            a[8].key, a[MINT].key, a[7].key, a[AUTHORITY].key, &[], tokens, protocol.mint().decimals)?,
            accounts: vec![8, MINT, 7, AUTHORITY, TOKEN], source: 8, destination: 7,
            seed: seeds::AUTHORITY, bump: c.bumps.piv_authority,
            source_after: expected[8].clone(), destination_after: expected[7].clone() });
    }
    Ok(Box::new(Plan { records, steps, writes, config: next_c, round: next_r, base,
        event: PendingContributionsIntegrated { config: *a[0].key, sequence: summary.sequence,
            integrated_sol_lamports: input.integrated_pending_sol_lamports,
            integrated_jitosol_units: input.integrated_pending_jitosol_units,
            contribution_value_lamports: summary.integrated_contribution_value_lamports,
            protected_hwm_lamports: summary.final_protected_hwm_lamports } }))
}
