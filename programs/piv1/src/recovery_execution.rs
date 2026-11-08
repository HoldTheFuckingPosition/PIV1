//! D111 Testnet-only exact same-round recovery and governed V2 migration.
//! No signer-only shortcut: the stored Squads instruction and current four votes
//! are authenticated at the actual invocation, independently of the frozen KIF set.
use std::rc::Rc;
use anchor_lang::{prelude::{AccountInfo,Clock,Pubkey,Rent,SolanaSysvar},AnchorDeserialize,
    solana_program::{entrypoint::ProgramResult,program_error::ProgramError,system_program,
        system_instruction,instruction::{Instruction,get_stack_height},program::{invoke_signed,get_return_data}}};
use solana_sha256_hasher::hash;
use crate::{accounts::{authenticate_fixed_accounts,FixedAccountInfos,STAKE_PROGRAM_ID},
    errors::Piv1Error,instruction_errors::{piv1_error_code,HOST_RUNTIME_UNAVAILABLE_CODE},
    state::{ActiveDistribution,PivConfig,
        recovery::ResumeRecovery, reconciliation::economic_custody_obligations},
    state_persistence::{PreparedStateWrite,StateEnvelope,commit_state_writes},
    squads_execution::{self,SquadsExecutionRoles,AuthenticatedSquadsInvocation},
    integrations::jito_identity::AuthenticatedJitoIdentity};
pub const RESUME_SELECTOR:[u8;8]=*b"PIV1RR01";
pub const MIGRATE_SELECTOR:[u8;8]=*b"PIV1MR03";
fn error(e:Piv1Error)->ProgramError{ProgramError::Custom(piv1_error_code(e))}
fn bad()->ProgramError{error(Piv1Error::CumulativeReconciliationMismatch)}
fn add(a:u64,b:u64)->Result<u64,ProgramError>{a.checked_add(b).ok_or(error(Piv1Error::ArithmeticOverflow))}
fn sub(a:u64,b:u64)->Result<u64,ProgramError>{a.checked_sub(b).ok_or_else(bad)}
fn governance_error(e:squads_execution::SquadsExecutionError)->ProgramError{
    match e { squads_execution::SquadsExecutionError::Runtime(e)=>e,
        squads_execution::SquadsExecutionError::State(e)=>error(e),_=>ProgramError::InvalidArgument }
}
pub fn process_instruction(program:&Pubkey,a:&[AccountInfo<'_>],data:&[u8])->ProgramResult{
    if !cfg!(feature="testnet-bounded-recovery"){return Err(ProgramError::InvalidInstructionData);}
    if !cfg!(target_os="solana"){return Err(ProgramError::Custom(HOST_RUNTIME_UNAVAILABLE_CODE));}
    execute(program,a,data,Clock::get()?,Rent::get()?,get_stack_height(),
        |ix,infos,seeds|invoke_signed(ix,infos,seeds),get_return_data)
}
#[cfg(not(target_os="solana"))]
pub fn process_with_host_callbacks<'i>(program:&Pubkey,a:&[AccountInfo<'i>],data:&[u8],clock:Clock,rent:Rent,
    stack_height:usize,invoke:impl FnMut(&Instruction,&[AccountInfo<'i>],&[&[&[u8]]])->ProgramResult,
    returned:impl FnOnce()->Option<(Pubkey,Vec<u8>)>)->ProgramResult{
    if !cfg!(feature="testnet-bounded-recovery"){return Err(ProgramError::InvalidInstructionData);}
    execute(program,a,data,clock,rent,stack_height,invoke,returned)
}
fn distinct(a:&[AccountInfo<'_>])->ProgramResult{
    for(i,left)in a.iter().enumerate(){for right in &a[i+1..]{
        if left.key==right.key||Rc::ptr_eq(&left.data,&right.data)||Rc::ptr_eq(&left.lamports,&right.lamports){return Err(error(Piv1Error::AccountAlias));}
    }}Ok(())
}
fn fixed<'a,'i>(a:&'a[AccountInfo<'i>],slots:[usize;9])->FixedAccountInfos<'a,'i>{FixedAccountInfos{
    config:&a[slots[0]],active_distribution:&a[slots[1]],pending_sol:&a[slots[2]],principal_sol:&a[slots[3]],
    operational_sol:&a[slots[4]],distribution_escrow:&a[slots[5]],kif_sol:&a[slots[6]],
    principal_jito:&a[slots[7]],pending_jito:&a[slots[8]]}}
#[derive(Clone,Debug,PartialEq,Eq)]
struct Record{key:Pubkey,owner:Pubkey,signer:bool,writable:bool,executable:bool,rent_epoch:u64,lamports:u64,len:usize,hash:[u8;32]}
impl Record{
    fn read(a:&AccountInfo<'_>)->Result<Self,ProgramError>{Self::read_after(a,None)}
    fn read_after(a:&AccountInfo<'_>,initial:Option<&Self>)->Result<Self,ProgramError>{
        let d=a.try_borrow_data()?;
        // Runtime readonly privileges apply to the entire descendant CPI tree.
        // Always hash initially and on HOST; writable data is never cached.
        let readonly_hash=initial.filter(|r|cfg!(target_os="solana")&&!r.writable).map(|r|r.hash);
        Ok(Self{key:*a.key,owner:*a.owner,signer:a.is_signer,writable:a.is_writable,executable:a.executable,
            rent_epoch:a.rent_epoch,lamports:a.lamports(),len:d.len(),
            hash:readonly_hash.unwrap_or_else(||hash(&d).to_bytes())})
    }
}
fn verify(a:&[AccountInfo<'_>],expected:&[Record])->ProgramResult{
    if a.len()!=expected.len(){return Err(bad());}
    for(a,r)in a.iter().zip(expected){if Record::read_after(a,Some(r))?!=*r{return Err(bad());}}Ok(())
}
fn authenticate(program:&Pubkey,a:&[AccountInfo<'_>],data:&[u8],roles:SquadsExecutionRoles,
    vault:u8,clock:&Clock,rent:&Rent,stack:usize)->Result<AuthenticatedSquadsInvocation,ProgramError>{
    squads_execution::dispatch(program,a,data,roles,vault,true,||stack,||Ok(clock.clone()),||Ok(rent.clone())).map_err(governance_error)
}
fn resume_roles(b:usize)->SquadsExecutionRoles{SquadsExecutionRoles{program:b+1,program_data:b+2,
    multisig:b+3,proposal:b+4,transaction:b+5,vault:b+6,instructions:b+7,config:0,
    guardian_registry:b+8,rewards:[b+9,b+10,b+11,b+12,b+13,b+14],clock:b}}
fn migration_roles()->SquadsExecutionRoles{SquadsExecutionRoles{program:0,program_data:1,multisig:2,
    proposal:3,transaction:4,vault:5,instructions:6,config:7,guardian_registry:8,rewards:[9,10,11,12,13,14],clock:15}}
#[inline(never)]
fn execute<'i>(program:&Pubkey,a:&[AccountInfo<'i>],data:&[u8],clock:Clock,rent:Rent,stack:usize,
    mut invoke:impl FnMut(&Instruction,&[AccountInfo<'i>],&[&[&[u8]]])->ProgramResult,
    returned:impl FnOnce()->Option<(Pubkey,Vec<u8>)>)->ProgramResult{
    distinct(a)?;
    if data.get(..8)==Some(&MIGRATE_SELECTOR){return migrate(program,a,data,&clock,&rent,stack,&mut invoke);}
    if data.get(..8)!=Some(&RESUME_SELECTOR)||data.get(8)!=Some(&1){return Err(ProgramError::InvalidInstructionData);}
    let request=ResumeRecovery::try_from_slice(&data[9..]).map_err(|_|ProgramError::InvalidInstructionData)?;
    if a.len()<33||a.len()>36{return Err(ProgramError::NotEnoughAccountKeys);}
    let f=authenticate_fixed_accounts(program,&rent,fixed(a,[0,1,2,3,4,5,6,7,8])).map_err(error)?;
    let c=f.config();let r=f.distribution();c.ensure_unpaused().map_err(error)?;
    let b=if c.manager_fee_account==c.referrer_token_account{18}else{19};
    if a.len()!=b+if request.destination==1{17}else{15}{return Err(ProgramError::InvalidArgument);}
    let roles=resume_roles(b);
    let proof=authenticate(program,a,data,roles,request.vault_index,&clock,&rent,stack)?;
    // Resume changes only the round. All other privileges are exact and read-only,
    // except the actual Vault PDA signer required by Squads.
    for(i,ai)in a.iter().enumerate(){if ai.is_writable!=(i==1)||ai.is_signer!=(i==b+6){return Err(bad());}}
    if c.protected_principal_hwm_lamports!=r.old_protected_principal_lamports
        || c.collective_kif_carry_lamports!=r.kif_carry_input_lamports
        || c.next_distribution_sequence!=add(r.active_sequence,1)? {return Err(bad());}
    for(i,id)in [(11,system_program::ID),(12,spl_token::ID)]{
        if a[i].key!=&id||!a[i].executable{return Err(error(Piv1Error::InvalidProgramIdentity));}
    }
    if a[9].key!=&c.piv_authority||a[9].owner!=&system_program::ID||a[9].executable
        ||!a[9].try_borrow_data()?.is_empty(){return Err(error(Piv1Error::InvalidAccountPda));}
    let before=f.withdrawal_leg_observation().map_err(error)?;
    if before.amounts().map_err(error)?!=economic_custody_obligations(c,r).map_err(error)?{return Err(bad());}
    let protocol=crate::withdrawal_finalization_execution::protocol_identity(c,a,b)?;
    crate::withdrawal_finalization_execution::validate_pool(&protocol,&clock,before.principal_jitosol_units,before.pending_jitosol_units)?;
    if clock.unix_timestamp<r.prepared_at||clock.epoch<r.prepared_epoch||clock.slot<r.prepared_slot{return Err(error(Piv1Error::TimestampRegression));}
    let next=r.resume_recovery(request,proof.multisig(),proof.transaction_index(),hash(data).to_bytes(),clock.unix_timestamp).map_err(error)?;
    let expected=a.iter().map(Record::read).collect::<Result<Vec<_>,_>>()?;
    let (validated_value,required_value)=if request.destination==1{
        if a[b+15].key!=&STAKE_PROGRAM_ID||!a[b+15].executable{return Err(error(Piv1Error::InvalidProgramIdentity));}
        invoke(&Instruction{program_id:STAKE_PROGRAM_ID,accounts:vec![],data:13_u32.to_le_bytes().to_vec()},&[a[b+15].clone()],&[])?;
        let(origin,bytes)=returned().ok_or(ProgramError::InvalidInstructionData)?;
        if origin!=STAKE_PROGRAM_ID{return Err(error(Piv1Error::InvalidProgramIdentity));}
        let minimum=u64::from_le_bytes(bytes.try_into().map_err(|_|ProgramError::InvalidInstructionData)?);
        verify(a,&expected)?;
        // Authenticated source order, current Stake minimum/rent, maximum-safe
        // fill and complete-target feasibility, shared with actual initiation.
        crate::integrations::jito_withdrawal::quote(&protocol,&a[15],&a[b+16],request.validator_list_index,
            &clock,&rent,minimum,f.operational_sol().economic_lamports().map_err(error)?,c,r).map_err(error)?;
        let residual=crate::withdrawal_finalization_execution::residual(c,r,&protocol)?;
        let required=add(r.proposed_hwm_after_settlement_lamports,r.pending_sol_used_lamports)?;
        if residual<required{return Err(error(Piv1Error::HighWaterMarkDecrease));}
        (residual,required)
    }else{
        if request.validator_list_index!=0{return Err(ProgramError::InvalidInstructionData);}
        prospective_settlement(c,r,before,&protocol)?
    };
    verify(a,&expected)?;
    // The complete fresh Record sweep above revalidates EVERY dependency of
    // this invocation's private Squads proof byte-for-byte, including current
    // governance, Instructions and privileges. Clock/Rent/stack are fixed during
    // this transaction. No proof survives this handler or permits changed data.

    let write=PreparedStateWrite::new(program,*a[1].key,StateEnvelope::distribution(r).map_err(error)?,
        StateEnvelope::distribution(&next).map_err(error)?).map_err(error)?;
    commit_state_writes(program,&rent,[(&write,&a[1])]).map_err(error)?;
    anchor_lang::emit!(crate::events::RecoveryResumed{round:*a[1].key,sequence:request.sequence,
        generation:request.generation,cumulative_loss:request.cumulative_loss,destination:request.destination,
        multisig:proof.multisig(),transaction_index:proof.transaction_index(),action_hash:hash(data).to_bytes(),frozen_snapshot_hash:request.frozen_snapshot_hash,
        observed_at:clock.unix_timestamp,escrow_lamports:request.escrow_lamports,
        validated_protected_value:validated_value,required_protected_value:required_value});
    Ok(())
}
fn prospective_settlement(c:&PivConfig,r:&ActiveDistribution,mut observed:crate::state::reconciliation::EconomicCustodyObservation,
    p:&AuthenticatedJitoIdentity)->Result<(u64,u64),ProgramError>{
    let eligible=r.eligible_native_for_settlement().map_err(error)?;let net=eligible.min(r.outgoing_gross_obligation_lamports);
    let allocation=crate::state::distribution::derive_net_beneficiary_allocation(net,r.htfp_gross_obligation_lamports,
        r.team_owner_gross_obligation_lamports,r.kif_gross_obligation_lamports).map_err(error)?;
    let zero=match piv1_math::allocate_kif(allocation.kif_lamports,r.kif_carry_input_lamports,r.kif_active_guardian_count).map_err(|e|error(e.into()))?{
        piv1_math::KifAllocation::ZeroActiveGuardians(z)=>z.compound_from_kif,_=>0};
    let settled=add(add(r.proposed_hwm_after_settlement_lamports,allocation.dust_lamports)?,add(sub(eligible,net)?,zero)?)?;
    observed.distribution_escrow.lamports=sub(observed.distribution_escrow.lamports,allocation.allocated_lamports)?;
    observed.principal_sol.lamports=add(observed.principal_sol.lamports,zero)?;
    let value=crate::settlement_execution::protected_value(observed,r,p)?;
    if c.protected_principal_hwm_lamports!=r.old_protected_principal_lamports
        || value<settled{return Err(error(Piv1Error::HighWaterMarkDecrease));}
    Ok((value,settled))
}

/// Migration ABI: selector8, version1, vault1, exact old padded body hash32,
/// maximum external rent debit8. The stored action also binds every account.
#[inline(never)]
fn migrate<'i>(program:&Pubkey,a:&[AccountInfo<'i>],data:&[u8],clock:&Clock,rent:&Rent,stack:usize,
    invoke:&mut impl FnMut(&Instruction,&[AccountInfo<'i>],&[&[&[u8]]])->ProgramResult)->ProgramResult{
    if data.len()!=50||data[8]!=1||a.len()!=26{return Err(ProgramError::InvalidInstructionData);}
    let roles=migration_roles();let vault=data[9];
    authenticate(program,a,data,roles,vault,clock,rent,stack)?;
    let f=authenticate_fixed_accounts(program,rent,fixed(a,[7,16,17,18,19,20,21,22,23])).map_err(error)?;
    let c=f.config();let r=f.distribution();c.ensure_unpaused().map_err(error)?;
    if f.withdrawal_leg_observation().map_err(error)?.amounts().map_err(error)?!=economic_custody_obligations(c,r).map_err(error)?{return Err(bad());}
    let next=r.migrate_v2_recovery().map_err(error)?;
    let old=StateEnvelope::distribution(r).map_err(error)?;
    if hash(old.as_bytes()).as_ref()!=&data[10..42]{return Err(bad());}
    let cap=u64::from_le_bytes(data[42..50].try_into().map_err(|_|ProgramError::InvalidInstructionData)?);
    if a[24].key!=&system_program::ID||!a[24].executable||a[25].owner!=&system_program::ID
        ||a[25].executable||!a[25].try_borrow_data()?.is_empty(){return Err(error(Piv1Error::InvalidAccountOwner));}
    for(i,ai)in a.iter().enumerate(){if ai.is_writable!=matches!(i,16|25)||ai.is_signer!=matches!(i,5|25){return Err(bad());}}
    if [c.piv_authority,c.htfp_recipient,c.team_owner_recipient,r.htfp_recipient,r.team_owner_recipient,
        c.jitosol_mint,c.stake_pool,c.validator_list,c.reserve_stake,c.manager_fee_account,c.referrer_token_account].contains(a[25].key)
        {return Err(error(Piv1Error::AccountAlias));}
    // An external guardian wallet (including G0) may pay approved operational
    // rent. Its separate earned KIF record/vault remain readonly and untouched.

    let envelope=StateEnvelope::distribution(&next).map_err(error)?;
    let delta=rent.minimum_balance(ActiveDistribution::RECOVERY_SPACE).saturating_sub(a[16].lamports());
    if delta>cap||a[25].lamports()<add(delta,rent.minimum_balance(0))?{return Err(error(Piv1Error::AccountRentDeficit));}
    let mut expected=a.iter().map(Record::read).collect::<Result<Vec<_>,_>>()?;
    expected[25].lamports=sub(expected[25].lamports,delta)?;expected[16].lamports=add(expected[16].lamports,delta)?;
    if delta>0{invoke(&system_instruction::transfer(a[25].key,a[16].key,delta),&[a[25].clone(),a[16].clone(),a[24].clone()],&[])?;}
    // Full fresh bytes/identity/privilege comparison preserves every input of
    // the initial actual Squads authentication. Only the exact approved rent
    // delta on round/payer is allowed; neither delta affects that predicate.
    verify(a,&expected)?;
    #[allow(deprecated)]
    a[16].realloc(ActiveDistribution::RECOVERY_SPACE,false)?;
    a[16].try_borrow_mut_data()?.copy_from_slice(envelope.as_bytes());
    expected[16].len=envelope.as_bytes().len();expected[16].hash=hash(envelope.as_bytes()).to_bytes();
    verify(a,&expected)?;
    authenticate_fixed_accounts(program,rent,fixed(a,[7,16,17,18,19,20,21,22,23])).map_err(error)?;
    anchor_lang::emit!(crate::events::DistributionRoundMigrated{round:*a[16].key,prior_version:2,new_version:3,rent_advanced_lamports:delta});
    Ok(())
}
