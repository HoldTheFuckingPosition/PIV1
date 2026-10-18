//! One-seat synchronization after an external Squads membership change.
//! The expected workflow uses prior Squads approval; this handler does not prove
//! historical old-set votes or persisted same-multisig identity continuity.
//! The current member set separately approves this exact PIV instruction. Old
//! reward records and prepared snapshots remain owned by their original keys.
//! Rent comes only from an external signing payer. CPI failures propagate for
//! transaction rollback; host callbacks do not supply a runtime or rollback.
use std::rc::Rc;
use anchor_lang::{prelude::{AccountInfo, Clock, Pubkey, Rent, SolanaSysvar},
    solana_program::{entrypoint::ProgramResult, instruction::{get_stack_height, Instruction},
        program::invoke_signed, program_error::ProgramError, system_instruction, system_program}};
use crate::{accounts::rent_floor, errors::Piv1Error, events::GuardianSetUpdated,
    instruction_errors::{piv1_error_code, squads_program_error, HOST_RUNTIME_UNAVAILABLE_CODE},
    instructions::update_config::UpdateGuardianSetRequest,
    kif_claim_accounts::GUARDIAN_REWARD_SEED,
    squads_execution::{self, SquadsExecutionRoles},
    state::GuardianReward,
    state_persistence::{commit_state_writes, PreparedStateWrite, StateEnvelope}};

struct Context { height: usize, clock: Clock, rent: Rent }
fn error(e: Piv1Error) -> ProgramError { ProgramError::Custom(piv1_error_code(e)) }
fn bad() -> ProgramError { error(Piv1Error::InvalidAccountData) }
fn amount(a: &AccountInfo<'_>) -> Result<u64, ProgramError> { Ok(**a.try_borrow_lamports()?) }

pub fn process_instruction(program: &Pubkey, accounts: &[AccountInfo<'_>], data: &[u8]) -> ProgramResult {
    dispatch(program, accounts, data, cfg!(target_os = "solana"), || Ok(Context {
        height: get_stack_height(), clock: Clock::get()?, rent: Rent::get()? }),
        |ix, a, seeds| invoke_signed(ix, a, seeds), |event| anchor_lang::emit!(event))
}
/// Explicit host model only, unreachable from native dispatch and absent on Solana.
#[cfg(not(target_os = "solana"))]
pub fn process_instruction_with_host_callbacks<'info>(
    program: &Pubkey, accounts: &[AccountInfo<'info>], data: &[u8],
    context: impl FnOnce() -> Result<squads_execution::ModeledSquadsInvocationContext, ProgramError>,
    invoke: impl FnMut(&Instruction, &[AccountInfo<'info>], &[&[&[u8]]]) -> ProgramResult,
    emit: impl FnOnce(GuardianSetUpdated),
) -> ProgramResult {
    dispatch(program, accounts, data, true, || context().map(|c| Context {
        height: c.stack_height, clock: c.clock, rent: c.rent }), invoke, emit)
}

struct Record { owner: Pubkey, lamports: u64, data: Option<Vec<u8>> }
impl Record {
    fn read(a: &AccountInfo<'_>, copy_data: bool) -> Result<Self, ProgramError> {
        Ok(Self { owner: *a.owner, lamports: amount(a)?,
            data: if copy_data { Some(a.try_borrow_data()?.to_vec()) } else { None } })
    }
    fn verify(&self, a: &AccountInfo<'_>) -> ProgramResult {
        if *a.owner != self.owner || amount(a)? != self.lamports {
            return Err(bad());
        }
        if let Some(data) = &self.data { if *a.try_borrow_data()? != data.as_slice() {
            return Err(bad());
        }} Ok(())
    }
}
fn verify(records: &[Record], accounts: &[AccountInfo<'_>]) -> ProgramResult {
    for (record, account) in records.iter().zip(accounts) { record.verify(account)?; } Ok(())
}

// Program, ProgramData, multisig, proposal, transaction, governance vault,
// Instructions, Config, old registry, six old rewards, Clock, six new rewards,
// distinct external payer, executable System. No economic vault/active round.
#[inline(never)]
fn dispatch<'info>(
    program: &Pubkey, a: &[AccountInfo<'info>], data: &[u8], available: bool,
    context: impl FnOnce() -> Result<Context, ProgramError>,
    mut invoke: impl FnMut(&Instruction, &[AccountInfo<'info>], &[&[&[u8]]]) -> ProgramResult,
    emit: impl FnOnce(GuardianSetUpdated),
) -> ProgramResult {
    let request = UpdateGuardianSetRequest::decode(data)?;
    if a.len() < 24 { return Err(ProgramError::NotEnoughAccountKeys); }
    if a.len() > 24 { return Err(ProgramError::InvalidArgument); }
    if !available { return Err(ProgramError::Custom(HOST_RUNTIME_UNAVAILABLE_CODE)); }
    let c = context()?;
    for (i, left) in a.iter().enumerate() {
        for right in &a[i + 1..] {
            if left.key == right.key || Rc::ptr_eq(&left.data, &right.data) || Rc::ptr_eq(&left.lamports, &right.lamports) {
                return Err(error(Piv1Error::AccountAlias));
            }
        }
    }
    // Only these nine roles can change. In particular the large ProgramData
    // executable bytes stay runtime read-only and are never copied onto the heap.
    for (i, account) in a.iter().enumerate() {
        if account.is_writable != [7,8,16,17,18,19,20,21,22].contains(&i) {
            return Err(error(Piv1Error::AccountNotWritable));
        }
    }
    let auth = squads_execution::authenticate_guardian_replacement(program, a, data, SquadsExecutionRoles {
        program: 0, program_data: 1, multisig: 2, proposal: 3, transaction: 4, vault: 5,
        instructions: 6, config: 7, guardian_registry: 8, rewards: [9,10,11,12,13,14], clock: 15,
    }, request, c.height, &c.clock, &c.rent).map_err(squads_program_error)?;
    let prepared = prepare_replacement(program, a, request, &c, &auth)?;
    let PreparedReplacement { rewards, shortfalls, writes, total } = prepared;
    let old = auth.guardians();
    let registry = auth.registry();
    let revision = registry.revision.to_le_bytes();
    // Check all affected borrows before any CPI, without retaining a guard over CPI.
    {
        let mut bytes = Vec::with_capacity(9); let mut balances = Vec::with_capacity(9);
        for i in [7,8,16,17,18,19,20,21,22] {
            bytes.push(a[i].try_borrow_mut_data()?); balances.push(a[i].try_borrow_mut_lamports()?);
        }
    }
    let mut records = a.iter().enumerate().map(|(i, account)| Record::read(account, (7..23).contains(&i)))
        .collect::<Result<Vec<_>, _>>()?;
    for i in 0..6 {
        let target = &a[16+i];
        if shortfalls[i] != 0 {
            let ix = system_instruction::transfer(a[22].key, target.key, shortfalls[i]);
            invoke(&ix, &[a[22].clone(), target.clone(), a[23].clone()], &[])?;
            records[22].lamports = records[22].lamports.checked_sub(shortfalls[i]).ok_or_else(bad)?;
            records[16+i].lamports = records[16+i].lamports.checked_add(shortfalls[i]).ok_or_else(bad)?;
            verify(&records, a)?;
        }
        let index = [i as u8]; let bump = [rewards[i].bump];
        let seeds: &[&[u8]] = &[GUARDIAN_REWARD_SEED, rewards[i].guardian.as_ref(), &revision, &index, &bump];
        invoke(&system_instruction::allocate(target.key, GuardianReward::SPACE as u64),
            &[target.clone(), a[23].clone()], &[seeds])?;
        records[16+i].data = Some(vec![0; GuardianReward::SPACE]); verify(&records, a)?;
        invoke(&system_instruction::assign(target.key, program), &[target.clone(), a[23].clone()], &[seeds])?;
        records[16+i].owner = *program; verify(&records, a)?;
    }
    commit_state_writes(program, &c.rent, [(&writes[0], &a[7]), (&writes[1], &a[8]),
        (&writes[2], &a[16]), (&writes[3], &a[17]), (&writes[4], &a[18]),
        (&writes[5], &a[19]), (&writes[6], &a[20]), (&writes[7], &a[21])]).map_err(error)?;
    emit(GuardianSetUpdated { config: *a[7].key, guardian_registry: *a[8].key,
        multisig: auth.multisig(), transaction_index: auth.transaction_index(),
        previous_revision: old.registry().revision, registry_revision: registry.revision,
        guardian_index: request.guardian_index, previous_guardian: old.registry().guardian_keys[usize::from(request.guardian_index)],
        new_guardian: request.new_guardian, rent_payer: *a[22].key, funded_rent_lamports: total });
    Ok(())
}

struct PreparedReplacement {
    rewards: Vec<GuardianReward>, shortfalls: [u64; 6],
    writes: Vec<PreparedStateWrite>, total: u64,
}
/// Preparation contains the fixed address table and replacement Config, keeping
/// them out of the execution frame without changing preflight/effect ordering.
#[inline(never)]
fn prepare_replacement(
    program: &Pubkey, a: &[AccountInfo<'_>], request: UpdateGuardianSetRequest,
    c: &Context, auth: &squads_execution::AuthenticatedGuardianReplacement,
) -> Result<PreparedReplacement, ProgramError> {
    let old = auth.guardians();
    let config = old.config();
    let bound = [config.piv_authority, config.active_distribution, config.principal_jito_vault,
        config.pending_jito_vault, config.pending_sol_vault, config.principal_sol_queue,
        config.operational_sol_vault, config.distribution_escrow, config.kif_sol_vault,
        config.guardian_registry, config.stake_pool_program, config.stake_pool,
        config.validator_list, config.reserve_stake, config.jitosol_mint, config.token_program,
        config.stake_program, config.system_program, config.manager_fee_account,
        config.referrer_token_account, config.htfp_recipient, config.team_owner_recipient];
    // Neither an approved signer flag nor a reward seed may convert protocol
    // custody into a rent payer or an entitled guardian destination.
    if bound.contains(a[22].key) || bound[..10].contains(&request.new_guardian)
        || a[7..22].iter().any(|account| *account.key == request.new_guardian) {
        return Err(error(Piv1Error::AccountAlias));
    }
    if a[23].key != &system_program::ID || !a[23].executable { return Err(error(Piv1Error::InvalidProgramIdentity)); }
    if !a[22].is_signer { return Err(ProgramError::MissingRequiredSignature); }
    if a[22].owner != &system_program::ID || a[22].executable || !a[22].try_borrow_data()?.is_empty() {
        return Err(error(Piv1Error::InvalidAccountOwner));
    }
    for i in [7,8,16,17,18,19,20,21,22] {
        if !a[i].is_writable { return Err(error(Piv1Error::AccountNotWritable)); }
    }
    let registry = auth.registry();
    let revision = registry.revision.to_le_bytes();
    let floor = rent_floor(&c.rent, GuardianReward::SPACE).map_err(error)?;
    let mut total = 0_u64;
    let mut rewards = Vec::with_capacity(6);
    let mut shortfalls = [0; 6];
    for i in 0..6 {
        let target = &a[16+i];
        let (key, bump) = Pubkey::try_find_program_address(
            &[GUARDIAN_REWARD_SEED, registry.guardian_keys[i].as_ref(), &revision, &[i as u8]], program)
            .ok_or_else(|| error(Piv1Error::InvalidAccountPda))?;
        if target.key != &key { return Err(error(Piv1Error::InvalidAccountPda)); }
        if bound.contains(target.key) || registry.guardian_keys.contains(target.key)
            || old.registry().guardian_keys.contains(target.key) { return Err(error(Piv1Error::AccountAlias)); }
        if target.owner != &system_program::ID { return Err(error(Piv1Error::InvalidAccountOwner)); }
        if target.executable { return Err(error(Piv1Error::ExecutableAccount)); }
        if !target.try_borrow_data()?.is_empty() { return Err(error(Piv1Error::InvalidAccountSize)); }
        let funded = amount(target)?;
        shortfalls[i] = if funded < floor { floor.checked_sub(funded).ok_or_else(bad)? } else { 0 };
        total = total.checked_add(shortfalls[i]).ok_or_else(|| error(Piv1Error::ArithmeticOverflow))?;
        let mut reward = GuardianReward::new(bump, registry, i as u8).map_err(error)?;
        if let Some(previous) = old.rewards().iter().find(|r| r.guardian == reward.guardian) {
            reward.last_active_period = previous.last_active_period;
        }
        rewards.push(reward);
    }
    let payer_after = amount(&a[22])?.checked_sub(total).ok_or_else(|| error(Piv1Error::AccountRentDeficit))?;
    if payer_after < rent_floor(&c.rent, 0).map_err(error)? { return Err(error(Piv1Error::AccountRentDeficit)); }
    let mut next_config = Box::new(config.clone()); next_config.guardian_registry_revision = registry.revision;
    let mut writes = Vec::with_capacity(8);
    writes.push(PreparedStateWrite::new(program, *a[7].key, StateEnvelope::config(config).map_err(error)?,
        StateEnvelope::config(&next_config).map_err(error)?).map_err(error)?);
    writes.push(PreparedStateWrite::new(program, *a[8].key, StateEnvelope::registry(old.registry()).map_err(error)?,
        StateEnvelope::registry(registry).map_err(error)?).map_err(error)?);
    for i in 0..6 { writes.push(PreparedStateWrite::initialize_reward(program, *a[16+i].key, &rewards[i]).map_err(error)?); }
    Ok(PreparedReplacement { rewards, shortfalls, writes, total })
}
