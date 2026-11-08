//! Atomic bounded small-shortfall preparation. Only the isolated staged token
//! budget and zero-native SwapAuthority enter Jupiter. A failure requires the
//! Solana transaction's rollback; host callbacks alone do not prove rollback.
use std::rc::Rc;
use anchor_lang::{prelude::{AccountInfo, Clock, Pubkey, Rent, SolanaSysvar}, solana_program::{
    entrypoint::ProgramResult, instruction::Instruction, program::invoke_signed,
    program_error::ProgramError, program_option::COption, program_pack::Pack,
    system_instruction, system_program}};
use solana_sha256_hasher::hash;
use spl_token::state::{Account, AccountState};
use crate::{accounts::{authenticate_fixed_accounts, seeds, FixedAccountInfos}, errors::Piv1Error,
    guardian_clock_accounts::{authenticate_guardian_clock_snapshot, GuardianClockAccountInfos},
    instruction_errors::{piv1_error_code, HOST_RUNTIME_UNAVAILABLE_CODE},
    integrations::{
        jupiter::{liquidity_plan, largest_safe_input, exact_book_debit, minimum_native_output, validate_receipt, LiquidityRoute},
        jupiter_route as route},
    state::{open_distribution, reconcile_pending_contributions, DistributionFunding, DistributionLifecycle,
        OpenDistributionInput, reconciliation::economic_custody_obligations},
    state_persistence::{commit_state_writes, PreparedStateWrite, StateEnvelope}};

pub const SELECTOR:[u8;8]=*b"PIV1JS01";
pub const MIGRATE_SELECTOR:[u8;8]=*b"PIV1MR02";
fn error(e:Piv1Error)->ProgramError{ProgramError::Custom(piv1_error_code(e))}
fn bad()->ProgramError{error(Piv1Error::ContributionObservationMismatch)}
fn add(a:u64,b:u64)->Result<u64,ProgramError>{a.checked_add(b).ok_or(error(Piv1Error::ArithmeticOverflow))}
fn sub(a:u64,b:u64)->Result<u64,ProgramError>{a.checked_sub(b).ok_or_else(bad)}
pub fn process_instruction(program:&Pubkey,a:&[AccountInfo<'_>],data:&[u8])->ProgramResult{
    if !cfg!(target_os="solana"){return Err(ProgramError::Custom(HOST_RUNTIME_UNAVAILABLE_CODE));}
    execute(program,a,data,Clock::get()?,Rent::get()?,|ix,infos,seeds|invoke_signed(ix,infos,seeds))
}
/// Permissionless, Idle-only, exact V1 -> V2 round extension. Existing Config,
/// completed summary, PDA and authorities are unchanged. Operational SOL pays
/// only the fresh rent shortfall; actual live use needs the deployment scope.
pub fn process_migration(program:&Pubkey,a:&[AccountInfo<'_>],data:&[u8])->ProgramResult{
    if data.len()!=9||data[..8]!=MIGRATE_SELECTOR||data[8]!=1{return Err(ProgramError::InvalidInstructionData);}
    if !cfg!(target_os="solana"){return Err(ProgramError::Custom(HOST_RUNTIME_UNAVAILABLE_CODE));}
    if a.len()!=10{return Err(ProgramError::NotEnoughAccountKeys);}
    let rent=Rent::get()?;let fixed=authenticate_fixed_accounts(program,&rent,roles(a)).map_err(error)?;
    let c=fixed.config();let r=fixed.distribution();c.ensure_unpaused().map_err(error)?;
    if r.version!=1||r.lifecycle!=DistributionLifecycle::Idle{return Err(error(Piv1Error::InvalidLifecycle));}
    if a[9].key!=&system_program::ID||!a[9].executable||a[9].is_writable{return Err(bad());}
    for (i,account) in a.iter().enumerate(){
        if account.is_signer || account.is_writable!=matches!(i,1|4){return Err(bad());}
    }
    let delta=rent.minimum_balance(crate::state::ActiveDistribution::SWAP_SPACE).saturating_sub(a[1].lamports());
    if fixed.operational_sol().economic_lamports().map_err(error)?<delta{return Err(error(Piv1Error::AccountRentDeficit));}
    let mut upgraded=*r;upgraded.version=crate::state::ActiveDistribution::SWAP_VERSION;
    let envelope=StateEnvelope::distribution(&upgraded).map_err(error)?;
    let mut expected=a.iter().map(|v|Record::read(v,None)).collect::<Result<Vec<_>,_>>()?;
    expected[4].lamports=sub(expected[4].lamports,delta)?;expected[1].lamports=add(expected[1].lamports,delta)?;
    if delta>0{
        let bump=[c.bumps.operational_sol_vault];let signer=[seeds::OPERATIONAL_SOL,bump.as_slice()];
        invoke_signed(&system_instruction::transfer(a[4].key,a[1].key,delta),&[a[4].clone(),a[1].clone(),a[9].clone()],&[&signer])?;
    }
    for (account,record)in a.iter().zip(&expected){if Record::read(account,Some(record))?!=*record{return Err(bad());}}
    #[allow(deprecated)]
    a[1].realloc(crate::state::ActiveDistribution::SWAP_SPACE,false)?;
    a[1].try_borrow_mut_data()?.copy_from_slice(envelope.as_bytes());
    expected[1].len=envelope.as_bytes().len();expected[1].hash=hash(envelope.as_bytes()).to_bytes();
    for(account,record)in a.iter().zip(&expected){if Record::read(account,Some(record))?!=*record{return Err(bad());}}
    anchor_lang::emit!(crate::events::DistributionRoundMigrated {round:*a[1].key,
        prior_version:1,new_version:2,rent_advanced_lamports:delta});
    Ok(())
}
#[cfg(not(target_os="solana"))]
pub fn process_with_host_callbacks<'info>(program:&Pubkey,a:&[AccountInfo<'info>],data:&[u8],clock:Clock,rent:Rent,
    invoke:impl FnMut(&Instruction,&[AccountInfo<'info>],&[&[&[u8]]])->ProgramResult)->ProgramResult{
    execute(program,a,data,clock,rent,invoke)
}
fn roles<'a,'i>(a:&'a[AccountInfo<'i>])->FixedAccountInfos<'a,'i>{FixedAccountInfos{
    config:&a[0],active_distribution:&a[1],pending_sol:&a[2],principal_sol:&a[3],operational_sol:&a[4],
    distribution_escrow:&a[5],kif_sol:&a[6],principal_jito:&a[7],pending_jito:&a[8]}}

#[derive(Clone,Debug,PartialEq,Eq)]
struct Record{key:Pubkey,owner:Pubkey,signer:bool,writable:bool,executable:bool,rent_epoch:u64,lamports:u64,len:usize,hash:[u8;32]}
impl Record{
    fn read(a:&AccountInfo<'_>,known:Option<&Self>)->Result<Self,ProgramError>{
        let d=a.try_borrow_data()?;
        let readonly=known.filter(|r|cfg!(target_os="solana")&&!r.writable);
        Ok(Self{key:*a.key,owner:*a.owner,signer:a.is_signer,writable:a.is_writable,executable:a.executable,
            rent_epoch:a.rent_epoch,lamports:a.lamports(),len:d.len(),hash:readonly.map_or_else(||hash(&d).to_bytes(),|r|r.hash)})
    }
}
struct Effects<'a,'i,F>{a:&'a[AccountInfo<'i>],records:Vec<Record>,invoke:F}
impl<'a,'i,F:FnMut(&Instruction,&[AccountInfo<'i>],&[&[&[u8]]])->ProgramResult> Effects<'a,'i,F>{
    fn verify(&self)->ProgramResult{
        for(a,r)in self.a.iter().zip(&self.records){if Record::read(a,Some(r))?!=*r{return Err(bad());}}Ok(())
    }
    fn call(&mut self,ix:Instruction,slots:&[usize],signers:&[&[&[u8]]])->ProgramResult{
        if slots.is_empty()||slots.len()>4{return Err(ProgramError::InvalidArgument);}
        let infos:[AccountInfo<'i>;4]=core::array::from_fn(|i|self.a[slots[i.min(slots.len()-1)]].clone());
        (self.invoke)(&ix,&infos[..slots.len()],signers)?;self.verify()
    }
    fn native(&mut self,from:usize,to:usize,amount:u64,signer:&[&[u8]])->ProgramResult{
        if amount==0{return Ok(());}
        self.records[from].lamports=sub(self.records[from].lamports,amount)?;
        self.records[to].lamports=add(self.records[to].lamports,amount)?;
        self.call(system_instruction::transfer(self.a[from].key,self.a[to].key,amount),&[from,to,11],&[signer])
    }
    fn token_transfer(&mut self,from:usize,to:usize,authority:usize,amount:u64,signer:&[&[u8]])->ProgramResult{
        for (i,increase) in [(from,false),(to,true)]{
            let mut d=self.a[i].try_borrow_data()?.to_vec();
            let old=u64::from_le_bytes(d[64..72].try_into().map_err(|_|bad())?);
            d[64..72].copy_from_slice(&(if increase{add(old,amount)?}else{sub(old,amount)?}).to_le_bytes());
            self.records[i].hash=hash(&d).to_bytes();
        }
        let ix=spl_token::instruction::transfer(&spl_token::ID,self.a[from].key,self.a[to].key,self.a[authority].key,&[],amount)?;
        self.call(ix,&[from,to,authority,12],&[signer])
    }
    fn create_token(&mut self,index:usize,mint:usize,authority:usize,rent:u64,seed:&[&[u8]],op_seed:&[&[u8]])->ProgramResult{
        self.native(4,index,rent,op_seed)?;
        self.records[index].len=Account::LEN;self.records[index].hash=hash(&[0;Account::LEN]).to_bytes();
        self.call(system_instruction::allocate(self.a[index].key,Account::LEN as u64),&[index,11],&[seed])?;
        self.records[index].owner=spl_token::ID;
        self.call(system_instruction::assign(self.a[index].key,&spl_token::ID),&[index,11],&[seed])?;
        let expected=Account{mint:*self.a[mint].key,owner:*self.a[authority].key,amount:0,delegate:COption::None,
            state:AccountState::Initialized,is_native:if *self.a[mint].key==route::WSOL{COption::Some(rent)}else{COption::None},
            delegated_amount:0,close_authority:COption::None};
        let mut bytes=[0;Account::LEN];Account::pack(expected,&mut bytes)?;self.records[index].hash=hash(&bytes).to_bytes();
        self.call(spl_token::instruction::initialize_account3(&spl_token::ID,self.a[index].key,self.a[mint].key,self.a[authority].key)?,
            &[index,mint,12],&[])
    }
    fn close_token(&mut self,index:usize,to:usize,authority:usize,signer:&[&[u8]])->ProgramResult{
        self.records[to].lamports=add(self.records[to].lamports,self.records[index].lamports)?;
        self.records[index].lamports=0;self.records[index].owner=system_program::ID;
        // Pinned Token host processor zeroes rather than resizes its test slice.
        self.records[index].len=if cfg!(target_os="solana"){0}else{Account::LEN};
        self.records[index].hash=hash(if cfg!(target_os="solana"){&[]}else{&[0;Account::LEN]}).to_bytes();
        self.call(spl_token::instruction::close_account(&spl_token::ID,self.a[index].key,self.a[to].key,self.a[authority].key,&[])?,
            &[index,to,authority,12],&[signer])
    }
}

// Bound V3 copy temporaries independently from the already bounded route frame.
#[inline(never)]
fn staged_round(r:&crate::state::ActiveDistribution)->Box<crate::state::ActiveDistribution>{Box::new(*r)}
#[inline(never)]
fn staged_config(c:&crate::state::PivConfig)->Box<crate::state::PivConfig>{Box::new(c.clone())}
#[inline(never)]
fn execute<'i>(program:&Pubkey,a:&[AccountInfo<'i>],data:&[u8],clock:Clock,rent:Rent,
    invoke:impl FnMut(&Instruction,&[AccountInfo<'i>],&[&[&[u8]]])->ProgramResult)->ProgramResult{
    if data.len()!=9 || data[..8]!=SELECTOR || data[8]!=1{return Err(ProgramError::InvalidInstructionData);}
    if a.len()<38 || a.len()>39{return Err(ProgramError::NotEnoughAccountKeys);}
    for (i,left) in a.iter().enumerate(){for right in &a[i+1..]{
        if left.key==right.key||Rc::ptr_eq(&left.data,&right.data)||Rc::ptr_eq(&left.lamports,&right.lamports){return Err(error(Piv1Error::AccountAlias));}
    }}
    let fixed=authenticate_fixed_accounts(program,&rent,roles(a)).map_err(error)?;
    let c=fixed.config();let r=fixed.distribution();c.ensure_unpaused().map_err(error)?;
    if r.lifecycle!=DistributionLifecycle::Idle||r.version<crate::state::ActiveDistribution::SWAP_VERSION{return Err(error(Piv1Error::InvalidLifecycle));}
    crate::state::timing::validate_preparation_interval(c.last_successful_preparation_at,clock.unix_timestamp).map_err(error)?;
    let base=if c.manager_fee_account==c.referrer_token_account{18}else{19};let b=base+8;
    if a.len()!=b+route::ROUTE_ACCOUNTS{return Err(ProgramError::InvalidArgument);}
    let before=fixed.distribution_preparation_observation().map_err(error)?;
    if before.amounts().map_err(error)?!=economic_custody_obligations(c,r).map_err(error)?{return Err(bad());}
    let protocol=crate::distribution_preparation_execution::protocol_identity(c,a,base==18)?;
    let guardians=authenticate_guardian_clock_snapshot(program,&rent,GuardianClockAccountInfos{config:&a[0],guardian_registry:&a[base],
        rewards:[&a[base+1],&a[base+2],&a[base+3],&a[base+4],&a[base+5],&a[base+6]],clock:&a[base+7]}).map_err(error)?;
    if guardians.clock()!=&clock{return Err(error(Piv1Error::InvalidClockAccount));}
    for (i,key)in [(11,system_program::ID),(12,spl_token::ID)]{if *a[i].key!=key||!a[i].executable||a[i].is_writable{return Err(error(Piv1Error::InvalidProgramIdentity));}}
    if *a[9].key!=c.piv_authority||a[9].owner!=&system_program::ID||a[9].executable||!a[9].try_borrow_data()?.is_empty(){return Err(bad());}
    for i in [9,10,13,14,15,16,17]{if a[i].is_writable||a[i].is_signer{return Err(bad());}}
    if base==19&&(a[18].is_writable||a[18].is_signer){return Err(bad());}
    for i in [0,1,2,3,4,5,7]{if !a[i].is_writable||a[i].is_signer{return Err(error(Piv1Error::AccountNotWritable));}}
    let pool=protocol.pool();let total=pool.total_lamports();let supply=pool.pool_token_supply();
    if pool.last_update_epoch()!=clock.epoch||total==0||supply==0||protocol.mint().supply>supply
        ||add(before.principal_jitosol_units,before.pending_jitosol_units)?>protocol.mint().supply{return Err(bad());}
    let seq=c.next_distribution_sequence.to_le_bytes();let mut bumps=[0;3];
    for(slot,seed)in [route::SWAP_AUTHORITY,route::SWAP_INPUT,route::SWAP_OUTPUT].into_iter().enumerate(){
        let(key,bump)=Pubkey::find_program_address(&[seed,&seq],program);bumps[slot]=bump;route::unused(&a[b+slot],key)?;
    }
    route::authenticate(a,b,c.jitosol_mint,&rent)?;
    let prefund=add(add(a[b].lamports(),a[b+1].lamports())?,a[b+2].lamports())?;
    let mut next=staged_config(c);let mut normalized=before;
    normalized.pending_sol.lamports=add(normalized.pending_sol.lamports,prefund)?;
    reconcile_pending_contributions(&mut next,r,normalized.pending()).map_err(error)?;
    let historical=add(c.accounted_historical_sol_lamports,piv1_math::checked_mul_div_floor(c.accounted_historical_jitosol_units,total,supply).map_err(|e|error(e.into()))?)?;
    let gross=add(historical,c.next_cycle_yield_lamports)?.saturating_sub(c.protected_principal_hwm_lamports);
    if gross==0{return Err(error(Piv1Error::ZeroTarget));}
    let split=piv1_math::split_gross_yield(gross).map_err(|e|error(e.into()))?;
    let outgoing=add(add(split.htfp_reserve,split.team_owner_pool)?,split.kif)?;
    let plan=liquidity_plan(outgoing,next.accounted_pending_sol_lamports,c.next_cycle_yield_lamports);
    if plan.route==LiquidityRoute::DelayedWithdrawal{return Err(error(Piv1Error::InvalidSwapInput));}
    let input=if plan.route==LiquidityRoute::Jupiter{largest_safe_input(c.accounted_historical_jitosol_units,total,supply,plan.shortfall_lamports).map_err(error)?}else{0};
    let debit=if input>0{exact_book_debit(c.accounted_historical_jitosol_units,input,total,supply).map_err(error)?}else{0};
    let dust=sub(plan.shortfall_lamports,debit)?;
    let proposed=add(c.protected_principal_hwm_lamports,add(add(split.permanent_compound,split.dust)?,dust)?)?;
    let residual=add(add(c.accounted_historical_sol_lamports,piv1_math::checked_mul_div_floor(sub(c.accounted_historical_jitosol_units,input)?,total,supply).map_err(|e|error(e.into()))?)?,sub(c.next_cycle_yield_lamports,plan.carry_used_lamports)?)?;
    if residual<proposed{return Err(error(Piv1Error::HighWaterMarkDecrease));}
    let token_rent=rent.minimum_balance(Account::LEN);
    let operational_before=a[4].lamports();
    if input>0&&fixed.operational_sol().economic_lamports().map_err(error)?<add(token_rent,token_rent)?{return Err(error(Piv1Error::AccountRentDeficit));}
    // No retained borrows cross a CPI. Check all writable data/lamports up front.
    for account in a {if account.is_writable{drop(account.try_borrow_mut_data()?);drop(account.try_borrow_mut_lamports()?);}}
    // The pinned bump allocator never reclaims growing Vec buffers. Preserve
    // every record/error in order with one exact-capacity allocation.
    let mut records=Vec::with_capacity(a.len());
    for account in a {records.push(Record::read(account,None)?);}
    let mut effects=Effects{a,records,invoke};
    let bump0=[bumps[0]];let bump1=[bumps[1]];let bump2=[bumps[2]];
    let authority_seed=[route::SWAP_AUTHORITY,seq.as_slice(),bump0.as_slice()];
    let input_seed=[route::SWAP_INPUT,seq.as_slice(),bump1.as_slice()];
    let output_seed=[route::SWAP_OUTPUT,seq.as_slice(),bump2.as_slice()];
    let op_bump=[c.bumps.operational_sol_vault];let op_seed=[seeds::OPERATIONAL_SOL,op_bump.as_slice()];
    for (slot,seed)in [(b,authority_seed.as_slice()),(b+1,input_seed.as_slice()),(b+2,output_seed.as_slice())]{
        let amount=effects.records[slot].lamports;effects.native(slot,2,amount,seed)?;
    }
    let mut receipt=None;
    if input>0{
        effects.create_token(b+1,10,b,token_rent,&input_seed,&op_seed)?;
        effects.create_token(b+2,b+3,b,token_rent,&output_seed,&op_seed)?;
        let piv_bump=[c.bumps.piv_authority];let piv_seed=[seeds::AUTHORITY,piv_bump.as_slice()];
        effects.token_transfer(7,b+1,9,input,&piv_seed)?;
        let minimum=minimum_native_output(debit).map_err(error)?;
        let vault_before=route::token(&a[b+8],&c.jitosol_mint,a[b+8].key,false)?.amount;
        let quote_before=add(route::token(&a[b+9],&route::WSOL,a[b+9].key,true)?.amount,route::token(&a[b+11],&route::WSOL,a[b+11].key,true)?.amount)?;
        let mut prior_tokens=Vec::with_capacity(5);
        for i in [b+1,b+2,b+8,b+9,b+11] { prior_tokens.push((i,a[i].try_borrow_data()?.to_vec())); }
        let ix=route::instruction(a,b,input,minimum);
        let mut infos=Vec::with_capacity(ix.accounts.len());
        for meta in &ix.accounts {infos.push(a.iter().find(|v|*v.key==meta.pubkey).cloned().ok_or_else(bad)?);}
        (effects.invoke)(&ix,&infos,&[&authority_seed])?;
        route::authenticate(a,b,c.jitosol_mint,&rent)?;
        let source=route::token(&a[b+1],&c.jitosol_mint,a[b].key,false)?;
        let output=route::token(&a[b+2],&route::WSOL,a[b].key,true)?;
        if source.amount!=0||a[b].lamports()!=0||a[b+1].lamports()!=token_rent
            ||a[b+2].lamports()!=add(token_rent,output.amount)?
            ||route::token(&a[b+8],&c.jitosol_mint,a[b+8].key,false)?.amount!=add(vault_before,input)?
            ||add(route::token(&a[b+9],&route::WSOL,a[b+9].key,true)?.amount,route::token(&a[b+11],&route::WSOL,a[b+11].key,true)?.amount)?!=sub(quote_before,output.amount)?{return Err(bad());}
        receipt=Some(validate_receipt(c.accounted_historical_jitosol_units,input,total,supply,plan.shortfall_lamports,output.amount).map_err(error)?);
        for (i,mut predicted) in prior_tokens {
            // Preserve every token byte except the measured amount. For native
            // tokens the lamport delta must equal that exact token delta, so
            // unsynchronized native credits cannot masquerade as swap output.
            let old_amount=u64::from_le_bytes(predicted[64..72].try_into().map_err(|_|bad())?);
            let current=Account::unpack(&a[i].try_borrow_data()?)?;
            predicted[64..72].copy_from_slice(&current.amount.to_le_bytes());
            effects.records[i].hash=hash(&predicted).to_bytes();
            if current.is_native.is_some() {
                effects.records[i].lamports=if current.amount>=old_amount {
                    add(effects.records[i].lamports,current.amount-old_amount)?
                }else{sub(effects.records[i].lamports,old_amount-current.amount)?};
            }
        }
        for i in [b+7,b+10] {
            let observed=Record::read(&a[i],None)?;let old=&effects.records[i];
            if observed.key!=old.key||observed.owner!=old.owner||observed.signer!=old.signer||observed.writable!=old.writable
                ||observed.executable!=old.executable||observed.rent_epoch!=old.rent_epoch||observed.len!=old.len
                ||observed.lamports!=old.lamports{return Err(bad());}
            effects.records[i]=observed;
        }
        effects.verify()?;
        effects.close_token(b+1,4,b,&authority_seed)?;
        effects.close_token(b+2,5,b,&authority_seed)?;
        // The native escrow receives WSOL plus its technical rent. Return only
        // that exact advance; no rent/prefund can satisfy the output floor.
        let escrow_bump=[c.bumps.distribution_escrow];let escrow_seed=[seeds::DISTRIBUTION_ESCROW,escrow_bump.as_slice()];
        effects.native(5,4,token_rent,&escrow_seed)?;
    }
    let pending_bump=[c.bumps.pending_sol_vault];let pending_seed=[seeds::PENDING_SOL,pending_bump.as_slice()];
    let principal_bump=[c.bumps.principal_sol_queue];let principal_seed=[seeds::PRINCIPAL_SOL,principal_bump.as_slice()];
    effects.native(2,5,plan.pending_used_lamports,&pending_seed)?;
    effects.native(3,5,plan.carry_used_lamports,&principal_seed)?;
    let mut round=staged_round(r);
    let pending_snapshot=next.accounted_pending_sol_lamports;
    open_distribution(&mut next,&mut round,guardians.registry(),guardians.rewards(),OpenDistributionInput{
        sequence:c.next_distribution_sequence,prepared_at:clock.unix_timestamp,prepared_slot:clock.slot,prepared_epoch:clock.epoch,
        historical_jitosol_units:c.accounted_historical_jitosol_units,historical_sol_lamports:c.accounted_historical_sol_lamports,
        historical_value_lamports:historical,snapshot_pool_total_lamports:total,snapshot_pool_token_supply:supply,
        snapshot_withdrawal_fee_numerator:pool.stake_withdrawal_fee().numerator,snapshot_withdrawal_fee_denominator:pool.stake_withdrawal_fee().denominator,
        gross_yield_lamports:gross,pending_sol_snapshot_lamports:pending_snapshot,
        pending_sol_used_lamports:plan.pending_used_lamports,snapshot_conversion_dust_lamports:dust,stored_residual_hwm_floor_lamports:proposed,
        funding:receipt.map_or(DistributionFunding::Liquid{escrow_available_lamports:outgoing},|receipt|DistributionFunding::Swap{receipt})}).map_err(error)?;
    effects.verify()?;
    let after=authenticate_fixed_accounts(program,&rent,roles(a)).map_err(error)?;
    if after.pending_integration_staged_observation(&next,&round).map_err(error)?.amounts().map_err(error)?!=economic_custody_obligations(&next,&round).map_err(error)?{return Err(bad());}
    if a[4].lamports()!=operational_before||a[b].lamports()!=0{return Err(bad());}
    let writes=[PreparedStateWrite::new(program,*a[0].key,StateEnvelope::config(c).map_err(error)?,StateEnvelope::config(&next).map_err(error)?).map_err(error)?,
        PreparedStateWrite::new(program,*a[1].key,StateEnvelope::distribution(r).map_err(error)?,StateEnvelope::distribution(&round).map_err(error)?).map_err(error)?];
    commit_state_writes(program,&rent,[(&writes[0],&a[0]),(&writes[1],&a[1])]).map_err(error)?;
    if let Some(receipt)=receipt {
        anchor_lang::emit!(crate::events::DistributionSwapPrepared {config:*a[0].key,sequence:round.active_sequence,
            spent_jitosol_units:receipt.spent_units,official_book_debit_lamports:receipt.official_book_debit_lamports,
            actual_native_output_lamports:receipt.actual_native_output_lamports,minimum_native_output_lamports:receipt.minimum_native_output_lamports,
            normalized_prefund_lamports:prefund,technical_rent_recovered_lamports:add(token_rent,token_rent)?});
    } else {
        anchor_lang::emit!(crate::events::DistributionPrepared {config:*a[0].key,sequence:round.active_sequence,
            gross_yield_lamports:gross,outgoing_lamports:outgoing,pending_sol_used_lamports:plan.pending_used_lamports,
            prior_yield_used_lamports:plan.carry_used_lamports,kif_eligibility_bitmap:round.kif_eligibility_bitmap});
    }
    Ok(())
}
