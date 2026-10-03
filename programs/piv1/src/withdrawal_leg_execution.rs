//! Protected active-source withdrawal, immediate deactivation and atomic leg
//! recording. Temp-PDA donations become pending contributions; actual rent is
//! advanced exclusively by OperationalSOL. This is not finalization/settlement.
use std::rc::Rc;
use anchor_lang::{prelude::{AccountInfo,Clock,Pubkey,Rent,SolanaSysvar},AnchorDeserialize,AnchorSerialize,
    solana_program::{entrypoint::ProgramResult,instruction::Instruction,program::{invoke_signed,get_return_data},
        program_error::ProgramError,system_instruction,system_program,sysvar}};
use solana_sha256_hasher::{hash,hashv};
use solana_stake_interface::state::{StakeStateV2,Authorized};
use crate::{accounts::{authenticate_fixed_accounts,rent_floor,seeds,FixedAccountInfos,STAKE_PROGRAM_ID},
    errors::Piv1Error,events::WithdrawalLegInitiated,
    instruction_errors::{piv1_error_code,protocol_program_error,HOST_RUNTIME_UNAVAILABLE_CODE},
    instructions::initiate_withdrawal_leg::decode_initiate_withdrawal_leg,
    integrations::{jito_withdrawal::{self,LegQuote},jito_identity::{authenticate_jito_identity,
        AuthenticatedJitoIdentity,DeclaredJitoKeys,JitoIdentityAccountInfos}},
    state::{PivConfig,ActiveDistribution,WithdrawalLeg,DistributionLifecycle,LegInitiationInput,
        initiate_withdrawal_leg,reconcile_pending_contributions,reconciliation::economic_custody_obligations},
    state_persistence::{StateEnvelope,PreparedStateWrite,commit_state_writes}};
const AUTHORITY:usize=9;const MINT:usize=10;const SYSTEM:usize=11;const TOKEN:usize=12;
const PROTOCOL:usize=13;const POOL:usize=14;const LIST:usize=15;const RESERVE:usize=16;const MANAGER:usize=17;
pub fn process_instruction(program:&Pubkey,a:&[AccountInfo<'_>],data:&[u8])->ProgramResult{
    dispatch(program,a,data,cfg!(target_os="solana"),Clock::get,Rent::get,
        |ix,infos,seeds|invoke_signed(ix,infos,seeds),get_return_data,|e|anchor_lang::emit!(e))
}
/// Host effects seam only. Errors retain partial modeled CPI effects; the test
/// transaction explicitly discards them. No VM/Bank rollback is claimed.
#[cfg(not(target_os="solana"))]
#[allow(clippy::too_many_arguments)]
pub fn process_instruction_with_host_callbacks<'info>(program:&Pubkey,a:&[AccountInfo<'info>],data:&[u8],
    clock:impl FnOnce()->Result<Clock,ProgramError>,rent:impl FnOnce()->Result<Rent,ProgramError>,
    invoke:impl FnMut(&Instruction,&[AccountInfo<'info>],&[&[&[u8]]])->ProgramResult,
    returned:impl FnOnce()->Option<(Pubkey,Vec<u8>)>,emit:impl FnOnce(WithdrawalLegInitiated))->ProgramResult{
    dispatch(program,a,data,true,clock,rent,invoke,returned,emit)
}
fn error(e:Piv1Error)->ProgramError{ProgramError::Custom(piv1_error_code(e))}
fn overflow()->ProgramError{error(Piv1Error::ArithmeticOverflow)}
fn bad()->ProgramError{error(Piv1Error::CumulativeReconciliationMismatch)}
fn add(a:u64,b:u64)->Result<u64,ProgramError>{a.checked_add(b).ok_or(overflow())}
fn sub(a:u64,b:u64)->Result<u64,ProgramError>{a.checked_sub(b).ok_or(overflow())}
#[allow(clippy::too_many_arguments)]
fn dispatch<'info>(program:&Pubkey,a:&[AccountInfo<'info>],data:&[u8],available:bool,
    get_clock:impl FnOnce()->Result<Clock,ProgramError>,get_rent:impl FnOnce()->Result<Rent,ProgramError>,
    mut invoke:impl FnMut(&Instruction,&[AccountInfo<'info>],&[&[&[u8]]])->ProgramResult,
    returned:impl FnOnce()->Option<(Pubkey,Vec<u8>)>,emit:impl FnOnce(WithdrawalLegInitiated))->ProgramResult{
    let index=decode_initiate_withdrawal_leg(data)?;
    if a.len()<24{return Err(ProgramError::NotEnoughAccountKeys);}
    if a.len()>25{return Err(ProgramError::InvalidArgument);}
    if !available{return Err(ProgramError::Custom(HOST_RUNTIME_UNAVAILABLE_CODE));}
    let clock=get_clock()?;let rent=get_rent()?;distinct(a)?;
    let mut plan=prepare(program,a,index,&clock,&rent,||{
        let stake=&a[a.len()-5];
        if stake.key!=&STAKE_PROGRAM_ID||!stake.executable{return Err(error(Piv1Error::InvalidProgramIdentity));}
        let records=a.iter().map(Record::read).collect::<Result<Vec<_>,_>>()?;
        invoke(&Instruction{program_id:STAKE_PROGRAM_ID,accounts:vec![],data:13_u32.to_le_bytes().to_vec()},&[stake.clone()],&[])?;
        verify(a,&records)?;
        let (origin,bytes)=returned().ok_or(ProgramError::InvalidInstructionData)?;
        if origin!=STAKE_PROGRAM_ID{return Err(ProgramError::IncorrectProgramId);}
        let bytes:[u8;8]=bytes.try_into().map_err(|_|ProgramError::InvalidInstructionData)?;
        let minimum=u64::from_le_bytes(bytes);if minimum==0{return Err(error(Piv1Error::TechnicalFloorNotMet));}Ok(minimum)
    })?;
    for step in &plan.steps{
        let infos=step.accounts.iter().map(|i|a[*i].clone()).collect::<Vec<_>>();
        let groups=step.signers.iter().map(|g|g.iter().map(Vec::as_slice).collect::<Vec<_>>()).collect::<Vec<_>>();
        let signers=groups.iter().map(Vec::as_slice).collect::<Vec<_>>();
        invoke(&step.instruction,&infos,&signers)?;
        for (index,record) in &step.changes{plan.records[*index]=record.clone();}
        verify(a,&plan.records)?;
    }
    // Fresh account authentication and actual post-pool proof, never stale
    // deserialized snapshots. Old state is still unchanged until this commit.
    let fixed=authenticate_fixed_accounts(program,&rent,roles(a)).map_err(error)?;
    let post=protocol_identity(fixed.config(),a,plan.base)?;
    validate_clock(&a[plan.base],&clock)?;
    let observed=fixed.withdrawal_leg_staged_observation(&plan.config,&plan.round).map_err(error)?;
    if observed.amounts().map_err(error)?!=economic_custody_obligations(&plan.config,&plan.round).map_err(error)?
        ||add(observed.principal_jitosol_units,observed.pending_jitosol_units)?>post.mint().supply{return Err(bad());}
    let p=post.pool();
    jito_withdrawal::validate_outlook(fixed.config(),fixed.distribution(),plan.quote.input,plan.quote.output,
        p.total_lamports(),p.pool_token_supply(),post.mint().supply,p.stake_withdrawal_fee(),plan.minimum).map_err(error)?;
    let destination=read_stake(&a[plan.base+5])?;
    let StakeStateV2::Stake(meta,stake,_) = destination else{return Err(bad());};
    if meta.authorized!=(Authorized{staker:plan.config.piv_authority,withdrawer:plan.config.piv_authority})
        ||stake.delegation.deactivation_epoch!=clock.epoch||stake.delegation.stake!=plan.quote.output{return Err(bad());}
    commit_state_writes(program,&rent,[(&plan.writes[0],&a[0]),(&plan.writes[1],&a[1]),
        (&plan.writes[2],&a[plan.base+4])]).map_err(error)?;
    emit(plan.event);Ok(())
}
fn roles<'a,'info>(a:&'a[AccountInfo<'info>])->FixedAccountInfos<'a,'info>{
    FixedAccountInfos{config:&a[0],active_distribution:&a[1],pending_sol:&a[2],principal_sol:&a[3],operational_sol:&a[4],
        distribution_escrow:&a[5],kif_sol:&a[6],principal_jito:&a[7],pending_jito:&a[8]}
}
fn distinct(a:&[AccountInfo<'_>])->ProgramResult{
    for(i,left)in a.iter().enumerate(){for right in &a[i+1..]{
        if left.key==right.key||Rc::ptr_eq(&left.data,&right.data)||Rc::ptr_eq(&left.lamports,&right.lamports){return Err(error(Piv1Error::AccountAlias));}
    }}Ok(())
}
#[derive(Clone,Debug,PartialEq,Eq)]
struct Record{key:Pubkey,owner:Pubkey,signer:bool,writable:bool,executable:bool,rent_epoch:u64,lamports:u64,len:usize,hash:[u8;32]}
impl Record{fn read(a:&AccountInfo<'_>)->Result<Self,ProgramError>{
    let data=a.try_borrow_data().map_err(|_|error(Piv1Error::AccountBorrowFailed))?;
    Ok(Self{key:*a.key,owner:*a.owner,signer:a.is_signer,writable:a.is_writable,executable:a.executable,rent_epoch:a.rent_epoch,
        lamports:**a.try_borrow_lamports().map_err(|_|error(Piv1Error::AccountBorrowFailed))?,len:data.len(),hash:hash(&data).to_bytes()})
}}
fn verify(a:&[AccountInfo<'_>],records:&[Record])->ProgramResult{
    for(a,r)in a.iter().zip(records){if Record::read(a)?!=*r{return Err(bad());}}Ok(())
}
fn patched(a:&AccountInfo<'_>,offset:usize,bytes:&[u8])->Result<[u8;32],ProgramError>{
    let data=a.try_borrow_data().map_err(|_|error(Piv1Error::AccountBorrowFailed))?;
    let end=offset.checked_add(bytes.len()).ok_or(overflow())?;if end>data.len(){return Err(bad());}
    Ok(hashv(&[&data[..offset],bytes,&data[end..]]).to_bytes())
}
fn read_stake(a:&AccountInfo<'_>)->Result<StakeStateV2,ProgramError>{
    let data=a.try_borrow_data().map_err(|_|error(Piv1Error::AccountBorrowFailed))?;
    if a.owner!=&STAKE_PROGRAM_ID||a.executable||data.len()!=200{return Err(bad());}
    StakeStateV2::deserialize(&mut &data[..]).map_err(|_|bad())
}
fn validate_clock(a:&AccountInfo<'_>,clock:&Clock)->ProgramResult{
    if a.key!=&sysvar::clock::ID||a.owner!=&sysvar::ID||a.executable{return Err(error(Piv1Error::InvalidClockAccount));}
    let data=a.try_borrow_data().map_err(|_|error(Piv1Error::AccountBorrowFailed))?;
    if data.len()!=40||Clock::from_account_info(a)?!=*clock{return Err(error(Piv1Error::InvalidClockAccount));}Ok(())
}
fn protocol_identity(c:&PivConfig,a:&[AccountInfo<'_>],base:usize)->Result<Box<AuthenticatedJitoIdentity>,ProgramError>{
    authenticate_jito_identity(&DeclaredJitoKeys{program:c.stake_pool_program,pool:c.stake_pool,validator_list:c.validator_list,
        reserve:c.reserve_stake,mint:c.jitosol_mint,manager_fee:c.manager_fee_account,referrer:c.referrer_token_account},
        JitoIdentityAccountInfos{program:&a[PROTOCOL],pool:&a[POOL],validator_list:&a[LIST],reserve:&a[RESERVE],mint:&a[MINT],
            manager_fee:&a[MANAGER],referrer:&a[if base==18{MANAGER}else{18}]}).map(Box::new).map_err(protocol_program_error)
}
struct Step{instruction:Instruction,accounts:Vec<usize>,signers:Vec<Vec<Vec<u8>>>,changes:Vec<(usize,Record)>}
struct Plan{records:Vec<Record>,steps:Vec<Step>,writes:[PreparedStateWrite;3],config:Box<PivConfig>,round:Box<ActiveDistribution>,
    base:usize,quote:LegQuote,minimum:u64,event:WithdrawalLegInitiated}
fn signer(seed:&[u8],bump:u8)->Vec<Vec<u8>>{vec![seed.to_vec(),vec![bump]]}
fn temp_signer(seed:&[u8],sequence:u64,index:u64,bump:u8)->Vec<Vec<u8>>{
    vec![seed.to_vec(),sequence.to_le_bytes().to_vec(),index.to_le_bytes().to_vec(),vec![bump]]
}
// Isolate construction from prepare's bounded SBF frame. Only this temporary
// staged value moves to the heap; encoding, checks and commit order stay exact.
#[inline(never)]
fn staged_leg(metadata_bump:u8,stake_bump:u8)->Box<WithdrawalLeg>{
    Box::new(WithdrawalLeg::vacant(metadata_bump,stake_bump))
}
#[inline(never)]
fn prepare(program:&Pubkey,a:&[AccountInfo<'_>],index:u32,clock:&Clock,rent:&Rent,
    query:impl FnOnce()->Result<u64,ProgramError>)->Result<Box<Plan>,ProgramError>{
    let fixed=authenticate_fixed_accounts(program,rent,roles(a)).map_err(error)?;
    let c=fixed.config();let r=fixed.distribution();c.ensure_unpaused().map_err(error)?;
    if r.lifecycle!=DistributionLifecycle::WithdrawalActive{return Err(error(Piv1Error::InvalidLifecycle));}
    let base=if c.manager_fee_account==c.referrer_token_account{18}else{19};
    if a.len()<base+6{return Err(ProgramError::NotEnoughAccountKeys);}if a.len()>base+6{return Err(ProgramError::InvalidArgument);}
    if r.is_withdrawal_target_assigned(){return Err(error(Piv1Error::TargetExceeded));}
    let before=fixed.withdrawal_leg_observation().map_err(error)?;
    if before.amounts().map_err(error)?!=economic_custody_obligations(c,r).map_err(error)?{return Err(bad());}
    let protocol=protocol_identity(c,a,base)?;validate_clock(&a[base],clock)?;
    let pool=protocol.pool();
    if pool.last_update_epoch()!=clock.epoch||protocol.mint().supply>pool.pool_token_supply()
        ||add(before.principal_jitosol_units,before.pending_jitosol_units)?>protocol.mint().supply{return Err(bad());}
    for(i,id)in [(SYSTEM,system_program::ID),(TOKEN,spl_token::ID),(base+1,STAKE_PROGRAM_ID)]{
        if a[i].key!=&id||!a[i].executable{return Err(error(Piv1Error::InvalidProgramIdentity));}
    }
    if a[AUTHORITY].key!=&c.piv_authority||a[AUTHORITY].owner!=&system_program::ID||a[AUTHORITY].executable
        ||!a[AUTHORITY].try_borrow_data().map_err(|_|error(Piv1Error::AccountBorrowFailed))?.is_empty(){return Err(error(Piv1Error::InvalidAccountPda));}
    if a[base+3].key!=&protocol.withdraw_authority()||a[base+3].executable{return Err(error(Piv1Error::InvalidAccountPda));}
    // Reject Token multisig interpretation of the pinned pool authority.
    if a[base+3].owner==&spl_token::ID&&a[base+3].try_borrow_data().map_err(|_|error(Piv1Error::AccountBorrowFailed))?.len()==355{return Err(bad());}
    for i in [MINT,POOL,LIST,RESERVE,MANAGER,if base==18{MANAGER}else{18}]{
        let len=a[i].try_borrow_data().map_err(|_|error(Piv1Error::AccountBorrowFailed))?.len();
        if **a[i].try_borrow_lamports().map_err(|_|error(Piv1Error::AccountBorrowFailed))?<rent_floor(rent,len).map_err(error)?{
            return Err(error(Piv1Error::AccountRentDeficit));
        }
    }
    let sequence=r.active_sequence;let leg_index=r.next_leg_index;
    let mut bumps=[0;2];
    for(slot,seed)in [seeds::WITHDRAWAL_LEG,seeds::WITHDRAWAL_STAKE].into_iter().enumerate(){
        let(expected,bump)=Pubkey::try_find_program_address(&[seed,&sequence.to_le_bytes(),&leg_index.to_le_bytes()],program)
            .ok_or(error(Piv1Error::InvalidAccountPda))?;bumps[slot]=bump;
        let target=&a[base+4+slot];
        if target.key!=&expected||target.owner!=&system_program::ID||target.executable
            ||!target.try_borrow_data().map_err(|_|error(Piv1Error::AccountBorrowFailed))?.is_empty(){return Err(error(Piv1Error::InvalidAccountPda));}
    }
    preflight_borrows(a,base)?;
    let minimum=query()?;
    let quote=jito_withdrawal::quote(&protocol,&a[LIST],&a[base+2],index,clock,rent,minimum,
        fixed.operational_sol().economic_lamports().map_err(error)?,c,r).map_err(error)?;
    let stake_rent=rent_floor(rent,200).map_err(error)?;let metadata_rent=rent_floor(rent,WithdrawalLeg::SPACE).map_err(error)?;
    let records=a.iter().map(Record::read).collect::<Result<Vec<_>,_>>()?;
    let mut expected=records.clone();let mut steps=Vec::with_capacity(6);
    let prefund=add(records[base+4].lamports,records[base+5].lamports)?;
    let mut next_c=Box::new(c.clone());let mut next_r=Box::new(*r);
    let mut after=before;after.pending_sol.lamports=add(after.pending_sol.lamports,prefund)?;
    reconcile_pending_contributions(&mut next_c,r,after.pending()).map_err(error)?;
    let mut leg=staged_leg(bumps[0],bumps[1]);
    initiate_withdrawal_leg(&next_c,&mut next_r,&mut leg,LegInitiationInput{sequence,leg_index,validator_list_index:index,
        validator_seed_suffix:quote.source.suffix,validator_vote:quote.source.vote,validator_stake_source:*a[base+2].key,
        initiation_epoch:clock.epoch,pool_total_lamports:pool.total_lamports(),pool_token_supply:pool.pool_token_supply(),
        withdrawal_fee_numerator:pool.stake_withdrawal_fee().numerator,withdrawal_fee_denominator:pool.stake_withdrawal_fee().denominator,
        current_technical_floor_units:quote.source.minimum,maximum_safe_capacity_units:quote.source.capacity,jitosol_input_units:quote.input,
        withdrawal_fee_units:quote.fee,burned_units:quote.burn,expected_native_lamports:quote.output,observed_delegated_native_lamports:quote.output,
        minimum_native_lamports:quote.minimum_output,stake_rent_advanced_lamports:stake_rent,metadata_rent_advanced_lamports:metadata_rent}).map_err(error)?;
    after.principal_jitosol_units=sub(after.principal_jitosol_units,quote.input)?;
    if after.amounts().map_err(error)?!=economic_custody_obligations(&next_c,&next_r).map_err(error)?
        ||add(after.principal_jitosol_units,after.pending_jitosol_units)?>quote.mint_after{return Err(bad());}
    let writes=[PreparedStateWrite::new(program,*a[0].key,StateEnvelope::config(c).map_err(error)?,StateEnvelope::config(&next_c).map_err(error)?).map_err(error)?,
        PreparedStateWrite::new(program,*a[1].key,StateEnvelope::distribution(r).map_err(error)?,StateEnvelope::distribution(&next_r).map_err(error)?).map_err(error)?,
        PreparedStateWrite::initialize_leg(program,*a[base+4].key,&leg).map_err(error)?];
    drop(leg);
    for(slot,seed)in [seeds::WITHDRAWAL_LEG,seeds::WITHDRAWAL_STAKE].into_iter().enumerate(){
        let target=base+4+slot;let amount=expected[target].lamports;
        if amount!=0{
            expected[target].lamports=0;expected[2].lamports=add(expected[2].lamports,amount)?;
            steps.push(Step{instruction:system_instruction::transfer(a[target].key,a[2].key,amount),accounts:vec![target,2,SYSTEM],
                signers:vec![temp_signer(seed,sequence,leg_index,bumps[slot])],changes:vec![(target,expected[target].clone()),(2,expected[2].clone())]});
        }
        let(size,owner,amount)=if slot==0{(WithdrawalLeg::SPACE,*program,metadata_rent)}else{(200,STAKE_PROGRAM_ID,stake_rent)};
        expected[4].lamports=sub(expected[4].lamports,amount)?;
        expected[target].lamports=amount;expected[target].owner=owner;expected[target].len=size;expected[target].hash=hash(&vec![0;size]).to_bytes();
        steps.push(Step{instruction:system_instruction::create_account(a[4].key,a[target].key,amount,size as u64,&owner),
            accounts:vec![4,target,SYSTEM],signers:vec![signer(seeds::OPERATIONAL_SOL,c.bumps.operational_sol_vault),temp_signer(seed,sequence,leg_index,bumps[slot])],
            changes:vec![(4,expected[4].clone()),(target,expected[target].clone())]});
    }
    let mut pool_balances=[0;16];pool_balances[..8].copy_from_slice(&quote.total_after.to_le_bytes());pool_balances[8..].copy_from_slice(&quote.supply_after.to_le_bytes());
    expected[POOL].hash=patched(&a[POOL],258,&pool_balances)?;
    expected[MINT].hash=patched(&a[MINT],36,&quote.mint_after.to_le_bytes())?;
    expected[7].hash=patched(&a[7],64,&after.principal_jitosol_units.to_le_bytes())?;
    let manager=protocol.manager_fee().amount;
    expected[MANAGER].hash=patched(&a[MANAGER],64,&add(manager,quote.fee)?.to_le_bytes())?;
    let list_offset=usize::try_from(index).map_err(|_|overflow())?.checked_mul(73).and_then(|n|n.checked_add(9)).ok_or(overflow())?;
    expected[base+2].lamports=sub(expected[base+2].lamports,quote.output)?;
    expected[LIST].hash=patched(&a[LIST],list_offset,&expected[base+2].lamports.to_le_bytes())?;
    let StakeStateV2::Stake(meta,mut source_stake,flags)=quote.source.state else{return Err(bad());};
    source_stake.delegation.stake=sub(source_stake.delegation.stake,quote.output)?;
    let source_state=StakeStateV2::Stake(meta,source_stake,flags);
    expected[base+2].hash=patched(&a[base+2],0,&source_state.try_to_vec().map_err(|_|bad())?)?;
    let mut destination_meta=meta;destination_meta.rent_exempt_reserve=jito_withdrawal::STAKE_PSEUDO_RENT;
    destination_meta.authorized=Authorized{staker:c.piv_authority,withdrawer:c.piv_authority};
    let mut destination_stake=source_stake;destination_stake.delegation.stake=quote.output;
    let destination=StakeStateV2::Stake(destination_meta,destination_stake,flags);
    let mut destination_bytes=destination.try_to_vec().map_err(|_|bad())?;destination_bytes.resize(200,0);
    expected[base+5].hash=hash(&destination_bytes).to_bytes();expected[base+5].lamports=add(stake_rent,quote.output)?;
    let changed=[POOL,LIST,MINT,7,MANAGER,base+2,base+5].into_iter().map(|i|(i,expected[i].clone())).collect();
    steps.push(Step{instruction:jito_withdrawal::instruction(*a[POOL].key,*a[LIST].key,*a[base+3].key,*a[base+2].key,*a[base+5].key,
        c.piv_authority,*a[7].key,*a[MANAGER].key,*a[MINT].key,quote.input,quote.minimum_output),
        accounts:vec![POOL,LIST,base+3,base+2,base+5,AUTHORITY,7,MANAGER,MINT,base,TOKEN,base+1,PROTOCOL],
        signers:vec![signer(seeds::AUTHORITY,c.bumps.piv_authority)],changes:changed});
    destination_stake.delegation.deactivation_epoch=clock.epoch;
    let destination=StakeStateV2::Stake(destination_meta,destination_stake,flags);
    let mut destination_bytes=destination.try_to_vec().map_err(|_|bad())?;destination_bytes.resize(200,0);
    expected[base+5].hash=hash(&destination_bytes).to_bytes();
    // Pinned StakeInstruction::Deactivate is bincode discriminant5.
    steps.push(Step{instruction:Instruction{program_id:STAKE_PROGRAM_ID,data:5_u32.to_le_bytes().to_vec(),accounts:vec![
        anchor_lang::solana_program::instruction::AccountMeta::new(*a[base+5].key,false),
        anchor_lang::solana_program::instruction::AccountMeta::new_readonly(*a[base].key,false),
        anchor_lang::solana_program::instruction::AccountMeta::new_readonly(c.piv_authority,true)]},
        accounts:vec![base+5,base,AUTHORITY,base+1],signers:vec![signer(seeds::AUTHORITY,c.bumps.piv_authority)],changes:vec![(base+5,expected[base+5].clone())]});
    Ok(Box::new(Plan{records,steps,writes,config:next_c,round:next_r,base,quote,minimum,event:WithdrawalLegInitiated{
        config:*a[0].key,sequence,leg_index,jitosol_input_units:quote.input,delegated_native_lamports:quote.output,
        stake_rent_advanced_lamports:stake_rent,metadata_rent_advanced_lamports:metadata_rent,normalized_prefund_lamports:prefund}}))
}
fn preflight_borrows(a:&[AccountInfo<'_>],base:usize)->ProgramResult{
    let mut required=[false;25];for i in [0,1,2,4,7,MINT,POOL,LIST,MANAGER,base+2,base+4,base+5]{required[i]=true;}
    let mut data=Vec::new();let mut lamports=Vec::new();
    for(i,a)in a.iter().enumerate(){if required[i]{
        if !a.is_writable{return Err(error(Piv1Error::AccountNotWritable));}
        data.push(a.try_borrow_mut_data().map_err(|_|error(Piv1Error::AccountBorrowFailed))?);
        lamports.push(a.try_borrow_mut_lamports().map_err(|_|error(Piv1Error::AccountBorrowFailed))?);
    }else{let _=Record::read(a)?;}}Ok(())
}
