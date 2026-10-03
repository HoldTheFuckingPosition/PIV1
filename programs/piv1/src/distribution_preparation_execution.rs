//! Authenticated liquid preparation and bounded active-source withdrawal proofs.
//! No fabricated quote, revision or capacity. Actual withdrawal execution and
//! unsupported transient/reserve/removal/preferred fallback remain separate.
use std::rc::Rc;
use anchor_lang::{prelude::{AccountInfo, Clock, Pubkey, Rent, SolanaSysvar}, solana_program::{
    entrypoint::ProgramResult, instruction::Instruction, program::{invoke_signed, get_return_data},
    program_error::ProgramError, program_pack::Pack, system_instruction, system_program}};
use solana_sha256_hasher::hash;
use spl_token::state::Mint;
use crate::{accounts::{authenticate_fixed_accounts, rent_floor, seeds, FixedAccountInfos},
    errors::Piv1Error, events::{DistributionPrepared, DistributionPreparationInsufficient},
    guardian_clock_accounts::{authenticate_guardian_clock_snapshot, GuardianClockAccountInfos},
    instruction_errors::{piv1_error_code, protocol_program_error, HOST_RUNTIME_UNAVAILABLE_CODE,
        DISTRIBUTION_WITHDRAWAL_UNAVAILABLE_CODE},
    instructions::prepare_distribution::{decode_prepare_distribution, decode_prepare_withdrawal, PREPARE_WITHDRAWAL_SELECTOR},
    integrations::jito_identity::{authenticate_jito_identity, AuthenticatedJitoIdentity,
        DeclaredJitoKeys, JitoIdentityAccountInfos},
    state::{open_distribution, record_no_yield_evaluation, DistributionFunding, OpenDistributionInput, ValidInsufficientAttemptInput,
        PivConfig, reconciliation::economic_custody_obligations},
    state_persistence::{commit_state_writes, PreparedStateWrite, StateEnvelope}};
/// Factual successful outcomes; old liquid-only host seam still emits its
/// original DistributionPrepared type, while withdrawal adds insufficient data.
#[derive(Debug,Clone,Copy,PartialEq,Eq)]
pub enum PreparationEvent { Prepared(DistributionPrepared), Insufficient(DistributionPreparationInsufficient) }
const AUTHORITY: usize = 9;
const MINT: usize = 10;
const SYSTEM: usize = 11;
const TOKEN: usize = 12;

pub fn process_instruction(program: &Pubkey, accounts: &[AccountInfo<'_>], data: &[u8]) -> ProgramResult {
    dispatch(program, accounts, data, cfg!(target_os = "solana"), Clock::get, Rent::get,
        |ix, a, seeds| invoke_signed(ix, a, seeds), get_return_data, true, |event| match event {
            PreparationEvent::Prepared(e)=>anchor_lang::emit!(e),
            PreparationEvent::Insufficient(e)=>anchor_lang::emit!(e) })
}
/// Explicit host effects only. Failed partial CPI effects are not rolled back
/// here; tests separately discard staged worlds. This is not runtime evidence.
#[cfg(not(target_os = "solana"))]
pub fn process_instruction_with_host_callbacks<'info>(program: &Pubkey, accounts: &[AccountInfo<'info>], data: &[u8],
    get_clock: impl FnOnce() -> Result<Clock, ProgramError>, get_rent: impl FnOnce() -> Result<Rent, ProgramError>,
    invoke: impl FnMut(&Instruction, &[AccountInfo<'info>], &[&[&[u8]]]) -> ProgramResult,
    emit: impl FnOnce(DistributionPrepared)) -> ProgramResult {
    dispatch(program, accounts, data, true, get_clock, get_rent, invoke, || None, false,
        |event| if let PreparationEvent::Prepared(e)=event{emit(e)})
}
/// Explicit host-only query/CPI effects and return-data seam. Neither a caller
/// instruction argument nor a substitute for runtime return-data authentication.
#[cfg(not(target_os = "solana"))]
pub fn process_withdrawal_instruction_with_host_callbacks<'info>(program: &Pubkey, accounts: &[AccountInfo<'info>], data: &[u8],
    get_clock: impl FnOnce() -> Result<Clock, ProgramError>, get_rent: impl FnOnce() -> Result<Rent, ProgramError>,
    invoke: impl FnMut(&Instruction, &[AccountInfo<'info>], &[&[&[u8]]]) -> ProgramResult,
    returned: impl FnOnce() -> Option<(Pubkey,Vec<u8>)>, emit: impl FnOnce(PreparationEvent)) -> ProgramResult {
    dispatch(program, accounts, data, true, get_clock, get_rent, invoke, returned, true, emit)
}
fn error(e: Piv1Error) -> ProgramError { ProgramError::Custom(piv1_error_code(e)) }
fn overflow() -> ProgramError { error(Piv1Error::ArithmeticOverflow) }
fn add(a: u64, b: u64) -> Result<u64, ProgramError> { a.checked_add(b).ok_or(overflow()) }
#[allow(clippy::too_many_arguments)]
fn dispatch<'info>(program: &Pubkey, a: &[AccountInfo<'info>], data: &[u8], available: bool,
    get_clock: impl FnOnce() -> Result<Clock, ProgramError>, get_rent: impl FnOnce() -> Result<Rent, ProgramError>,
    mut invoke: impl FnMut(&Instruction, &[AccountInfo<'info>], &[&[&[u8]]]) -> ProgramResult,
    returned: impl FnOnce() -> Option<(Pubkey,Vec<u8>)>, allow_withdrawal: bool,
    emit: impl FnOnce(PreparationEvent)) -> ProgramResult {
    let index = if data.get(..8)==Some(PREPARE_WITHDRAWAL_SELECTOR.as_slice()) {
        if !allow_withdrawal {return Err(ProgramError::InvalidInstructionData);}
        Some(decode_prepare_withdrawal(data)?)
    } else {decode_prepare_distribution(data)?;None};
    let extra=if index.is_some(){2}else{0};
    if a.len() < 26+extra { return Err(ProgramError::NotEnoughAccountKeys); }
    if a.len() > 27+extra { return Err(ProgramError::InvalidArgument); }
    if !available { return Err(ProgramError::Custom(HOST_RUNTIME_UNAVAILABLE_CODE)); }
    let clock = get_clock()?; let rent = get_rent()?; distinct_accounts(a)?;
    let Some(mut prepared) = prepare(program, &clock, &rent, a, index, || {
        let stake=&a[a.len()-2];
        if stake.key!=&crate::accounts::STAKE_PROGRAM_ID||!stake.executable{return Err(error(Piv1Error::InvalidProgramIdentity));}
        let before=a.iter().map(AccountRecord::read).collect::<Result<Vec<_>,_>>()?;
        // Pinned StakeInstruction::GetMinimumDelegation bincode discriminant13;
        // no account metas, signer or custody effect. Check all supplied records.
        let ix=Instruction{program_id:crate::accounts::STAKE_PROGRAM_ID,accounts:vec![],data:13_u32.to_le_bytes().to_vec()};
        invoke(&ix,&[stake.clone()],&[])?;
        verify_records(a,&before)?;
        let (origin,bytes)=returned().ok_or(ProgramError::InvalidInstructionData)?;
        if origin!=crate::accounts::STAKE_PROGRAM_ID{return Err(ProgramError::IncorrectProgramId);}
        let exact:[u8;8]=bytes.try_into().map_err(|_|ProgramError::InvalidInstructionData)?;
        Ok(u64::from_le_bytes(exact))
    })? else { return Ok(()); };
    if prepared.round_write.is_none() {
        if !a[0].is_writable{return Err(error(Piv1Error::AccountNotWritable));}
        commit_state_writes(program,&rent,[(&prepared.config_write,&a[0])]).map_err(error)?;
        emit(prepared.event.ok_or(ProgramError::InvalidArgument)?);
        return Ok(());
    }
    preflight_borrows(a, &prepared.movements)?;
    for movement in &prepared.movements {
        let bump = [movement.bump]; let signer = [movement.seed, &bump];
        let infos = [a[movement.source].clone(), a[5].clone(), a[SYSTEM].clone()];
        invoke(&movement.instruction, &infos, &[&signer])?;
        prepared.records[movement.source] = movement.source_after.clone();
        prepared.records[5] = movement.destination_after.clone();
        verify_records(a, &prepared.records)?;
    }
    // Old Config+Idle obligations remain until both final writes. Only
    // metadata/rent is reauthenticated here; exact predicted new custody was
    // proved against the staged state before effects and checked after each CPI.
    let _ = authenticate_fixed_accounts(program, &rent, roles(a)).map_err(error)?;
    let round_write=prepared.round_write.as_ref().ok_or(ProgramError::InvalidArgument)?;
    commit_state_writes(program, &rent, [(&prepared.config_write, &a[0]),
        (round_write, &a[1])]).map_err(error)?;
    emit(prepared.event.ok_or(ProgramError::InvalidArgument)?); Ok(())
}
fn roles<'a, 'info>(a: &'a [AccountInfo<'info>]) -> FixedAccountInfos<'a, 'info> {
    FixedAccountInfos { config: &a[0], active_distribution: &a[1], pending_sol: &a[2], principal_sol: &a[3],
        operational_sol: &a[4], distribution_escrow: &a[5], kif_sol: &a[6], principal_jito: &a[7], pending_jito: &a[8] }
}
fn distinct_accounts(accounts: &[AccountInfo<'_>]) -> ProgramResult {
    for (index, account) in accounts.iter().enumerate() {
        if accounts[index + 1..].iter().any(|other| account.key == other.key
            || Rc::ptr_eq(&account.data, &other.data) || Rc::ptr_eq(&account.lamports, &other.lamports)) {
            return Err(error(Piv1Error::AccountAlias));
        }
    }
    Ok(())
}

// Hash borrowed buffers so a large validator list/program cannot grow the heap.
#[derive(Clone, Debug, Eq, PartialEq)]
struct AccountRecord {
    key: Pubkey, owner: Pubkey, signer: bool, writable: bool, executable: bool,
    rent_epoch: u64, lamports: u64, data_len: usize, data_hash: [u8; 32],
}
impl AccountRecord {
    fn read(account: &AccountInfo<'_>) -> Result<Self, ProgramError> {
        let data = account.try_borrow_data().map_err(|_| error(Piv1Error::AccountBorrowFailed))?;
        Ok(Self { key: *account.key, owner: *account.owner, signer: account.is_signer,
            writable: account.is_writable, executable: account.executable, rent_epoch: account.rent_epoch,
            lamports: **account.try_borrow_lamports().map_err(|_| error(Piv1Error::AccountBorrowFailed))?,
            data_len: data.len(), data_hash: hash(&data).to_bytes() })
    }
}
fn verify_records(accounts: &[AccountInfo<'_>], expected: &[AccountRecord]) -> ProgramResult {
    for (account, record) in accounts.iter().zip(expected) {
        if AccountRecord::read(account)? != *record { return Err(error(Piv1Error::ContributionObservationMismatch)); }
    }
    Ok(())
}
#[inline(never)]
fn protocol_identity(config: &PivConfig, accounts: &[AccountInfo<'_>], shared: bool)
    -> Result<Box<AuthenticatedJitoIdentity>, ProgramError>
{
    let declared = DeclaredJitoKeys { program: config.stake_pool_program, pool: config.stake_pool,
        validator_list: config.validator_list, reserve: config.reserve_stake, mint: config.jitosol_mint,
        manager_fee: config.manager_fee_account, referrer: config.referrer_token_account };
    authenticate_jito_identity(&declared, JitoIdentityAccountInfos { program: &accounts[13], pool: &accounts[14],
        validator_list: &accounts[15], reserve: &accounts[16], mint: &accounts[MINT],
        manager_fee: &accounts[17], referrer: &accounts[if shared { 17 } else { 18 }] })
        .map(Box::new).map_err(protocol_program_error)
}

struct Movement {
    instruction: Instruction, source: usize, seed: &'static [u8], bump: u8,
    source_after: AccountRecord, destination_after: AccountRecord,
}
struct PreparedDistribution {
    records: Vec<AccountRecord>, movements: Vec<Movement>,
    config_write: PreparedStateWrite, round_write: Option<PreparedStateWrite>, event: Option<PreparationEvent>,
}
#[inline(never)]
fn prepare(program: &Pubkey, clock: &Clock, rent: &Rent, a: &[AccountInfo<'_>], withdrawal_index: Option<u32>,
    query_minimum: impl FnOnce()->Result<u64,ProgramError>)
    -> Result<Option<Box<PreparedDistribution>>, ProgramError> {
    let authenticated = authenticate_fixed_accounts(program, rent, roles(a)).map_err(error)?;
    let c = authenticated.config(); let shared = c.manager_fee_account == c.referrer_token_account;
    let base = if shared { 18 } else { 19 };
    let expected=base+8+if withdrawal_index.is_some(){2}else{0};
    if a.len() < expected { return Err(ProgramError::NotEnoughAccountKeys); }
    if a.len() > expected { return Err(ProgramError::InvalidArgument); }
    let before = authenticated.distribution_preparation_observation().map_err(error)?;
    if before.amounts().map_err(error)? != economic_custody_obligations(c, authenticated.distribution()).map_err(error)? {
        return Err(error(Piv1Error::CumulativeReconciliationMismatch));
    }
    let protocol = protocol_identity(c, a, shared)?;
    let guardians = authenticate_guardian_clock_snapshot(program, rent, GuardianClockAccountInfos {
        config: &a[0], guardian_registry: &a[base],
        rewards: [&a[base+1], &a[base+2], &a[base+3], &a[base+4], &a[base+5], &a[base+6]],
        clock: &a[base+7] }).map_err(error)?;
    if guardians.clock() != clock { return Err(error(Piv1Error::InvalidClockAccount)); }
    for (index, id) in [(SYSTEM, system_program::ID), (TOKEN, spl_token::ID)] {
        if a[index].key != &id || !a[index].executable { return Err(error(Piv1Error::InvalidProgramIdentity)); }
    }
    if a[AUTHORITY].key != &c.piv_authority { return Err(error(Piv1Error::InvalidAccountPda)); }
    if a[AUTHORITY].owner != &system_program::ID { return Err(error(Piv1Error::InvalidAccountOwner)); }
    if a[AUTHORITY].executable { return Err(error(Piv1Error::ExecutableAccount)); }
    if !a[AUTHORITY].try_borrow_data().map_err(|_| error(Piv1Error::AccountBorrowFailed))?.is_empty() {
        return Err(error(Piv1Error::InvalidAccountData));
    }
    if **a[MINT].try_borrow_lamports().map_err(|_| error(Piv1Error::AccountBorrowFailed))?
        < rent_floor(rent, Mint::LEN).map_err(error)? { return Err(error(Piv1Error::AccountRentDeficit)); }
    let pool = protocol.pool(); let total = pool.total_lamports(); let supply = pool.pool_token_supply();
    if pool.last_update_epoch() != clock.epoch || (total == 0) != (supply == 0)
        || protocol.mint().supply > supply
        || add(before.principal_jitosol_units, before.pending_jitosol_units)? > protocol.mint().supply {
        return Err(error(Piv1Error::InvalidCustodyObservation));
    }
    let tokens_value = if supply == 0 { 0 } else {
        piv1_math::checked_mul_div_floor(c.accounted_historical_jitosol_units, total, supply)
            .map_err(Piv1Error::from).map_err(error)? };
    let historical = add(c.accounted_historical_sol_lamports, tokens_value)?;
    let gross = add(historical, c.next_cycle_yield_lamports)?.saturating_sub(c.protected_principal_hwm_lamports);
    if gross == 0 {
        if withdrawal_index.is_some(){return Err(error(Piv1Error::ZeroTarget));}
        record_no_yield_evaluation(c, authenticated.distribution(), clock.unix_timestamp, historical).map_err(error)?;
        return Ok(None);
    }
    // Preserve pause/lifecycle/cadence validation even when withdrawal is unavailable.
    c.ensure_unpaused().map_err(error)?;
    if authenticated.distribution().lifecycle != crate::state::DistributionLifecycle::Idle {
        return Err(error(Piv1Error::InvalidLifecycle));
    }
    crate::state::timing::validate_preparation_interval(c.last_successful_preparation_at, clock.unix_timestamp).map_err(error)?;
    let split = piv1_math::split_gross_yield(gross).map_err(Piv1Error::from).map_err(error)?;
    let outgoing = add(add(split.htfp_reserve, split.team_owner_pool)?, split.kif)?;
    if outgoing == 0 { return Err(error(Piv1Error::ZeroTarget)); }
    let pending = c.accounted_pending_sol_lamports.min(outgoing);
    let carry = c.next_cycle_yield_lamports.min(outgoing - pending);
    let budget=outgoing-pending-carry;
    let proof=if let Some(index)=withdrawal_index {
        if budget==0{return Err(error(Piv1Error::ZeroTarget));}
        let minimum=query_minimum()?;
        let proof=crate::integrations::jito_withdrawal_preparation::derive(&protocol,&a[15],&a[base+9],index,
            clock,rent,minimum,authenticated.operational_sol().economic_lamports().map_err(error)?,
            budget,c.accounted_historical_jitosol_units,c.configured_slippage_bps).map_err(error)?;
        if proof.insufficient(){
            let mut next=c.clone();
            crate::state::transitions::record_authenticated_insufficient_attempt(&mut next,authenticated.distribution(),
                ValidInsufficientAttemptInput{attempted_at:clock.unix_timestamp,historical_value_lamports:historical,
                    pending_sol_snapshot_lamports:c.accounted_pending_sol_lamports,computed_jitosol_target_units:proof.target,
                    validated_technical_minimum_units:proof.minimum}).map_err(error)?;
            let write=PreparedStateWrite::new(program,*a[0].key,StateEnvelope::config(c).map_err(error)?,
                StateEnvelope::config(&next).map_err(error)?).map_err(error)?;
            return Ok(Some(Box::new(PreparedDistribution{records:vec![],movements:vec![],config_write:write,round_write:None,
                event:Some(PreparationEvent::Insufficient(DistributionPreparationInsufficient{config:*a[0].key,
                    attempted_at:clock.unix_timestamp,target_jitosol_units:proof.target,technical_minimum_jitosol_units:proof.minimum}))})));
        }
        Some(proof)
    }else{
        if budget!=0{return Err(ProgramError::Custom(DISTRIBUTION_WITHDRAWAL_UNAVAILABLE_CODE));}None
    };
    let dust=proof.map_or(0,|p|p.conversion_dust);
    let proposed = add(c.protected_principal_hwm_lamports, add(add(split.permanent_compound, split.dust)?,dust)?)?;
    let residual_units=c.accounted_historical_jitosol_units.checked_sub(proof.map_or(0,|p|p.target)).ok_or(overflow())?;
    let residual_tokens=if supply==0{0}else{piv1_math::checked_mul_div_floor(residual_units,total,supply).map_err(Piv1Error::from).map_err(error)?};
    let residual = add(add(c.accounted_historical_sol_lamports,residual_tokens)?, c.next_cycle_yield_lamports - carry)?;
    if residual < proposed { return Err(error(Piv1Error::HighWaterMarkDecrease)); }
    let liquid=add(pending,carry)?;
    let funding=if let Some(p)=proof{DistributionFunding::Withdrawal{fixed_jitosol_target_units:p.target,
        snapshot_leg_input_floor_units:p.minimum,maximum_useful_legs:p.maximum_legs,
        stored_round_minimum_native_lamports:p.round_minimum,initial_escrow_available_lamports:liquid}}
        else{DistributionFunding::Liquid{escrow_available_lamports:liquid}};
    let mut next = Box::new(c.clone()); let mut round = Box::new(*authenticated.distribution());
    open_distribution(&mut next, &mut round, guardians.registry(), guardians.rewards(), OpenDistributionInput {
        sequence: c.next_distribution_sequence, prepared_at: clock.unix_timestamp, prepared_slot: clock.slot,
        prepared_epoch: clock.epoch, historical_jitosol_units: c.accounted_historical_jitosol_units,
        historical_sol_lamports: c.accounted_historical_sol_lamports, historical_value_lamports: historical,
        snapshot_pool_total_lamports: total, snapshot_pool_token_supply: supply,
        snapshot_withdrawal_fee_numerator: pool.stake_withdrawal_fee().numerator,
        snapshot_withdrawal_fee_denominator: pool.stake_withdrawal_fee().denominator,
        gross_yield_lamports: gross, pending_sol_snapshot_lamports: c.accounted_pending_sol_lamports,
        pending_sol_used_lamports: pending, snapshot_conversion_dust_lamports: dust,
        stored_residual_hwm_floor_lamports: proposed,
        funding }).map_err(error)?;
    let mut after = before;
    after.pending_sol.lamports = after.pending_sol.lamports.checked_sub(pending).ok_or(overflow())?;
    after.principal_sol.lamports = after.principal_sol.lamports.checked_sub(carry).ok_or(overflow())?;
    after.distribution_escrow.lamports = add(after.distribution_escrow.lamports, liquid)?;
    if after.amounts().map_err(error)? != economic_custody_obligations(&next, &round).map_err(error)? {
        return Err(error(Piv1Error::CumulativeReconciliationMismatch));
    }
    let config_write = PreparedStateWrite::new(program, *a[0].key,
        StateEnvelope::config(c).map_err(error)?, StateEnvelope::config(&next).map_err(error)?).map_err(error)?;
    let round_write = PreparedStateWrite::new(program, *a[1].key,
        StateEnvelope::distribution(authenticated.distribution()).map_err(error)?, StateEnvelope::distribution(&round).map_err(error)?).map_err(error)?;
    let records: Vec<_> = a.iter().map(AccountRecord::read).collect::<Result<_, _>>()?;
    let mut expected = records.clone(); let mut movements = Vec::with_capacity(2);
    for (source, amount, seed, bump) in [(2, pending, seeds::PENDING_SOL, c.bumps.pending_sol_vault),
        (3, carry, seeds::PRINCIPAL_SOL, c.bumps.principal_sol_queue)] {
        if amount == 0 { continue; }
        expected[source].lamports = expected[source].lamports.checked_sub(amount).ok_or(overflow())?;
        expected[5].lamports = add(expected[5].lamports, amount)?;
        movements.push(Movement { instruction: system_instruction::transfer(a[source].key, a[5].key, amount),
            source, seed, bump, source_after: expected[source].clone(), destination_after: expected[5].clone() });
    }
    Ok(Some(Box::new(PreparedDistribution { records, movements, config_write, round_write:Some(round_write),
        event: Some(PreparationEvent::Prepared(DistributionPrepared { config: *a[0].key, sequence: round.active_sequence, gross_yield_lamports: gross,
            outgoing_lamports: outgoing, pending_sol_used_lamports: pending, prior_yield_used_lamports: carry,
            kif_eligibility_bitmap: round.kif_eligibility_bitmap })) })))
}
fn preflight_borrows(a: &[AccountInfo<'_>], movements: &[Movement]) -> ProgramResult {
    let mut mutable = [false; 29]; mutable[0] = true; mutable[1] = true;
    for movement in movements { mutable[movement.source] = true; mutable[5] = true; }
    let mut data = Vec::with_capacity(5); let mut lamports = Vec::with_capacity(5);
    for (index, account) in a.iter().enumerate() {
        if mutable[index] {
            if !account.is_writable { return Err(error(Piv1Error::AccountNotWritable)); }
            data.push(account.try_borrow_mut_data().map_err(|_| error(Piv1Error::AccountBorrowFailed))?);
            lamports.push(account.try_borrow_mut_lamports().map_err(|_| error(Piv1Error::AccountBorrowFailed))?);
        }
    }
    Ok(())
}
