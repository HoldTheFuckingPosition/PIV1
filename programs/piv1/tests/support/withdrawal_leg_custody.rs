//! Independent source/state and full-account oracle for modeled initiation.
//! Production preparation supplies the starting active round, not the expected
//! initiation result. No actual Stake/SPL runtime or rollback is simulated.
use anchor_lang::{prelude::{AccountInfo,Clock,Pubkey},AnchorDeserialize,
    solana_program::{entrypoint::ProgramResult,instruction::{Instruction,AccountMeta},system_program}};
use piv1::{accounts::{DISTRIBUTION_DISCRIMINATOR,STAKE_PROGRAM_ID},state::{ActiveDistribution,WithdrawalLeg,WithdrawalLegStatus},
    events::WithdrawalLegInitiated,integrations::jito_identity::{JITO_STAKE_POOL,JITO_STAKE_POOL_PROGRAM},
    distribution_preparation_execution::process_withdrawal_instruction_with_host_callbacks as prepare};
use solana_stake_interface::state::{StakeStateV2,Authorized};
use super::{custody::{self,*},bootstrap_custody::{POOL,LIST,MANAGER},withdrawal_custody,oracle,
    support::kif_claim_custody::{BackingAccount,envelope,PROGRAM}};
#[derive(Clone,Debug,PartialEq)]
pub struct Fixture{pub custody:custody::Fixture,pub clock:Clock,pub minimum:u64,pub index:u32,pub base:usize}
impl Fixture{
    pub fn new(shared:bool)->Self{Self::from_preparation(withdrawal_custody::Fixture::new(321,117,shared))}
    pub fn from_preparation(mut w:withdrawal_custody::Fixture)->Self{
        let clock=w.inner.base.clock.clone();let rent=w.inner.base.custody.rent.clone();let data=w.data();let minimum=w.minimum;
        w.inner.base.custody.with_infos(|a|prepare(&PROGRAM,a,&data,||Ok(clock.clone()),||Ok(rent),|ix,infos,_|{
            if ix.program_id==STAKE_PROGRAM_ID{assert_eq!(ix.data,13_u32.to_le_bytes());return Ok(());}
            assert_eq!(ix.data[..4],2_u32.to_le_bytes());let amount=u64::from_le_bytes(ix.data[4..12].try_into().unwrap());
            **infos[0].try_borrow_mut_lamports()?-=amount;**infos[1].try_borrow_mut_lamports()?+=amount;Ok(())
        },||Some((STAKE_PROGRAM_ID,minimum.to_le_bytes().to_vec())),|_|{})).unwrap();
        let base=w.inner.guardian_start;let mut custody=w.inner.base.custody;
        custody.accounts.drain(base..base+7); // Preserve exact Clock, Stake program and source.
        let authority=Pubkey::find_program_address(&[JITO_STAKE_POOL.as_ref(),b"withdraw"],&JITO_STAKE_POOL_PROGRAM).0;
        custody.accounts.push(empty(authority));
        let mut out=Self{custody,clock,minimum,index:w.index,base};out.new_pair();
        for i in [0,1,2,4,7,MINT,POOL,LIST,MANAGER,base+2,base+4,base+5]{out.custody.accounts[i].writable=true;}
        out
    }
    pub fn round(&self)->ActiveDistribution{ActiveDistribution::deserialize(&mut &self.custody.accounts[ROUND].data[8..]).unwrap()}
    pub fn set_round(&mut self,r:ActiveDistribution){self.custody.accounts[ROUND].data=envelope(&r,DISTRIBUTION_DISCRIMINATOR,ActiveDistribution::SPACE);}
    pub fn pool(&self)->oracle::StakePool{borsh1::BorshDeserialize::deserialize(&mut &self.custody.accounts[POOL].data[..]).unwrap()}
    pub fn set_pool(&mut self,p:&oracle::StakePool){let bytes=borsh1::to_vec(p).unwrap();self.custody.accounts[POOL].data[..bytes.len()].copy_from_slice(&bytes);}
    pub fn source(&self)->StakeStateV2{StakeStateV2::deserialize(&mut &self.custody.accounts[self.base+2].data[..]).unwrap()}
    pub fn set_source(&mut self,state:StakeStateV2){let data=borsh1::to_vec(&state).unwrap();self.custody.accounts[self.base+2].data[..data.len()].copy_from_slice(&data);}
    pub fn source_capacity(&mut self,native:u64){
        let StakeStateV2::Stake(meta,mut stake,flags)=self.source()else{panic!()};
        stake.delegation.stake=1_000_000+native;let value=meta.rent_exempt_reserve+stake.delegation.stake;
        self.custody.accounts[self.base+2].lamports=value;self.set_source(StakeStateV2::Stake(meta,stake,flags));
        let pos=9+self.index as usize*73;self.custody.accounts[LIST].data[pos..pos+8].copy_from_slice(&value.to_le_bytes());
    }
    pub fn new_pair(&mut self){
        self.custody.accounts.truncate(self.base+4);
        let r=self.round();for seed in [b"withdrawal-leg".as_slice(),b"withdrawal-stake".as_slice()]{
            let address=Pubkey::find_program_address(&[seed,&r.active_sequence.to_le_bytes(),&r.next_leg_index.to_le_bytes()],&PROGRAM).0;
            let mut a=empty(address);a.writable=true;self.custody.accounts.push(a);
        }
    }
    pub fn data(&self)->Vec<u8>{let mut d=b"PIV1IL01".to_vec();d.push(1);d.extend(self.index.to_le_bytes());d}
}
fn empty(key:Pubkey)->BackingAccount{BackingAccount{key,owner:system_program::ID,lamports:0,data:vec![],executable:false,signer:false,writable:false}}
pub fn read_u64(data:&[u8],offset:usize)->u64{u64::from_le_bytes(data[offset..offset+8].try_into().unwrap())}
fn write_u64(data:&mut[u8],offset:usize,value:u64){data[offset..offset+8].copy_from_slice(&value.to_le_bytes());}
pub fn snapshot(a:&AccountInfo<'_>)->BackingAccount{BackingAccount{key:*a.key,owner:*a.owner,lamports:a.lamports(),
    data:a.data.borrow().to_vec(),signer:a.is_signer,writable:a.is_writable,executable:a.executable}}
#[derive(Clone)]
pub struct Call{pub ix:Instruction,pub infos:Vec<usize>,pub seeds:Vec<Vec<Vec<u8>>>,pub after:Vec<BackingAccount>}
pub struct Expected{pub after:Fixture,pub event:WithdrawalLegInitiated,pub calls:Vec<Call>,pub leg:WithdrawalLeg}
fn bump(seed:&[u8])->Vec<Vec<u8>>{let b=Pubkey::find_program_address(&[seed],&PROGRAM).1;vec![seed.to_vec(),vec![b]]}
fn leg_seeds(seed:&[u8],r:&ActiveDistribution)->Vec<Vec<u8>>{
    let sequence=r.active_sequence.to_le_bytes();let index=r.next_leg_index.to_le_bytes();
    let b=Pubkey::find_program_address(&[seed,&sequence,&index],&PROGRAM).1;
    vec![seed.to_vec(),sequence.to_vec(),index.to_vec(),vec![b]]
}
fn fee(q:u64,p:&oracle::StakePool)->u64{
    if p.stake_withdrawal_fee.numerator==0{0}else{(u128::from(q)*u128::from(p.stake_withdrawal_fee.numerator)).div_ceil(u128::from(p.stake_withdrawal_fee.denominator)) as u64}
}
fn redeem(q:u64,p:&oracle::StakePool)->u64{(u128::from(q-fee(q,p))*u128::from(p.total_lamports)/u128::from(p.pool_token_supply)) as u64}
fn record(calls:&mut Vec<Call>,f:&Fixture,ix:Instruction,infos:Vec<usize>,seeds:Vec<Vec<Vec<u8>>>){
    calls.push(Call{ix,infos,seeds,after:f.custody.accounts.clone()});
}
pub fn expected(f:&Fixture)->Expected{
    let mut out=f.clone();let mut calls=vec![];let b=f.base;let r=f.round();let c=f.custody.config();let mut pool=f.pool();
    let StakeStateV2::Stake(meta,mut stake,flags)=f.source()else{panic!()};
    let available=(f.custody.accounts[b+2].lamports-meta.rent_exempt_reserve-f.minimum.max(1_000_000)).min(stake.delegation.stake-f.minimum);
    let remaining=r.fixed_jitosol_withdrawal_target_units-r.cumulative_jitosol_assigned_units;
    // Exhaustive independent small-domain oracle, not the production binary searches.
    let q=(1..=remaining).rev().find(|q|redeem(*q,&pool)<=available).unwrap();
    let technical=(1..=remaining).find(|q|redeem(*q,&pool)>=f.minimum).unwrap();
    let charged=fee(q,&pool);let burn=q-charged;let output=redeem(q,&pool);
    let minimum=f.minimum.max((u128::from(output)*u128::from(10000-c.configured_slippage_bps)/10000) as u64);
    let metadata_rent=f.custody.rent.minimum_balance(WithdrawalLeg::SPACE);let stake_rent=f.custody.rent.minimum_balance(200);
    let donation=f.custody.accounts[b+4].lamports+f.custody.accounts[b+5].lamports;
    record(&mut calls,&out,Instruction{program_id:STAKE_PROGRAM_ID,data:vec![13,0,0,0],accounts:vec![]},vec![b+1],vec![]);
    for(slot,seed)in [b"withdrawal-leg".as_slice(),b"withdrawal-stake".as_slice()].into_iter().enumerate(){
        let target=b+4+slot;let amount=out.custody.accounts[target].lamports;
        if amount>0{
            out.custody.accounts[target].lamports=0;out.custody.accounts[PENDING_SOL].lamports+=amount;
            let mut data=vec![2,0,0,0];data.extend(amount.to_le_bytes());
            record(&mut calls,&out,Instruction{program_id:system_program::ID,data,accounts:vec![
                AccountMeta::new(out.custody.accounts[target].key,true),AccountMeta::new(c.pending_sol_vault,false)]},
                vec![target,PENDING_SOL,SYSTEM],vec![leg_seeds(seed,&r)]);
        }
        let (amount,size,owner)=if slot==0{(metadata_rent,WithdrawalLeg::SPACE,PROGRAM)}else{(stake_rent,200,STAKE_PROGRAM_ID)};
        out.custody.accounts[OPERATIONAL_SOL].lamports-=amount;
        out.custody.accounts[target].lamports=amount;out.custody.accounts[target].owner=owner;out.custody.accounts[target].data=vec![0;size];
        let mut data=vec![0,0,0,0];data.extend(amount.to_le_bytes());data.extend((size as u64).to_le_bytes());data.extend(owner.to_bytes());
        record(&mut calls,&out,Instruction{program_id:system_program::ID,data,accounts:vec![
            AccountMeta::new(c.operational_sol_vault,true),AccountMeta::new(out.custody.accounts[target].key,true)]},
            vec![OPERATIONAL_SOL,target,SYSTEM],vec![bump(b"operational-sol"),leg_seeds(seed,&r)]);
    }
    pool.total_lamports-=output;pool.pool_token_supply-=burn;out.set_pool(&pool);
    for(i,offset,debit)in [(PRINCIPAL_JITO,64,q),(MINT,36,burn)]{
        let value=read_u64(&out.custody.accounts[i].data,offset)-debit;write_u64(&mut out.custody.accounts[i].data,offset,value);
    }
    let value=read_u64(&out.custody.accounts[MANAGER].data,64)+charged;write_u64(&mut out.custody.accounts[MANAGER].data,64,value);
    out.custody.accounts[b+2].lamports-=output;
    let source_balance=out.custody.accounts[b+2].lamports;
    write_u64(&mut out.custody.accounts[LIST].data,9+f.index as usize*73,source_balance);
    stake.delegation.stake-=output;out.set_source(StakeStateV2::Stake(meta,stake,flags));
    let mut target_meta=meta;target_meta.rent_exempt_reserve=2_282_880;target_meta.authorized=Authorized{staker:c.piv_authority,withdrawer:c.piv_authority};
    let mut target_stake=stake;target_stake.delegation.stake=output;
    let mut target=borsh1::to_vec(&StakeStateV2::Stake(target_meta,target_stake,flags)).unwrap();target.resize(200,0);
    out.custody.accounts[b+5].data=target;out.custody.accounts[b+5].lamports+=output;
    let mut data=vec![24];data.extend(q.to_le_bytes());data.extend(minimum.to_le_bytes());
    let metas=vec![AccountMeta::new(c.stake_pool,false),AccountMeta::new(c.validator_list,false),
        AccountMeta::new_readonly(out.custody.accounts[b+3].key,false),AccountMeta::new(out.custody.accounts[b+2].key,false),
        AccountMeta::new(out.custody.accounts[b+5].key,false),AccountMeta::new_readonly(c.piv_authority,false),
        AccountMeta::new_readonly(c.piv_authority,true),AccountMeta::new(c.principal_jito_vault,false),AccountMeta::new(c.manager_fee_account,false),
        AccountMeta::new(c.jitosol_mint,false),AccountMeta::new_readonly(out.custody.accounts[b].key,false),
        AccountMeta::new_readonly(spl_token::ID,false),AccountMeta::new_readonly(STAKE_PROGRAM_ID,false)];
    record(&mut calls,&out,Instruction{program_id:JITO_STAKE_POOL_PROGRAM,data,accounts:metas},
        vec![POOL,LIST,b+3,b+2,b+5,AUTHORITY,PRINCIPAL_JITO,MANAGER,MINT,b,TOKEN,b+1,13],vec![bump(b"authority")]);
    target_stake.delegation.deactivation_epoch=f.clock.epoch;
    let data=borsh1::to_vec(&StakeStateV2::Stake(target_meta,target_stake,flags)).unwrap();out.custody.accounts[b+5].data[..data.len()].copy_from_slice(&data);
    record(&mut calls,&out,Instruction{program_id:STAKE_PROGRAM_ID,data:vec![5,0,0,0],accounts:vec![
        AccountMeta::new(out.custody.accounts[b+5].key,false),AccountMeta::new_readonly(out.custody.accounts[b].key,false),
        AccountMeta::new_readonly(c.piv_authority,true)]},vec![b+5,b,AUTHORITY,b+1],vec![bump(b"authority")]);
    out.custody.edit_config(|c|c.accounted_pending_sol_lamports+=donation);
    let mut next=r;next.cumulative_jitosol_assigned_units+=q;next.cumulative_withdrawal_fee_units+=charged;next.cumulative_burned_units+=burn;
    next.cumulative_expected_native_lamports+=output;next.cumulative_delegated_native_lamports+=output;next.next_leg_index+=1;next.successful_leg_count+=1;
    out.set_round(next);
    let p=f.pool();let metadata_bump=leg_seeds(b"withdrawal-leg",&r)[3][0];let stake_bump=leg_seeds(b"withdrawal-stake",&r)[3][0];
    let mut leg=WithdrawalLeg::vacant(metadata_bump,stake_bump);
    leg.is_initialized=true;leg.status=WithdrawalLegStatus::Initiated;leg.sequence=r.active_sequence;leg.leg_index=r.next_leg_index;
    leg.validator_list_index=f.index;leg.validator_seed_suffix=u32::from_le_bytes(f.custody.accounts[LIST].data[9+f.index as usize*73+36..9+f.index as usize*73+40].try_into().unwrap());
    leg.validator_vote=stake.delegation.voter_pubkey;leg.validator_stake_source=f.custody.accounts[b+2].key;leg.initiation_epoch=f.clock.epoch;
    leg.pool_total_lamports=p.total_lamports;leg.pool_token_supply=p.pool_token_supply;leg.withdrawal_fee_numerator=p.stake_withdrawal_fee.numerator;
    leg.withdrawal_fee_denominator=p.stake_withdrawal_fee.denominator;leg.technical_floor_units=technical.max(r.snapshot_leg_input_floor_units);
    leg.jitosol_input_units=q;leg.withdrawal_fee_units=charged;leg.burned_units=burn;leg.expected_native_lamports=output;
    leg.observed_delegated_native_lamports=output;leg.minimum_native_lamports=minimum;leg.stake_rent_advanced_lamports=stake_rent;leg.metadata_rent_advanced_lamports=metadata_rent;
    let disc=solana_sha256_hasher::hash(b"account:WithdrawalLeg").to_bytes();out.custody.accounts[b+4].data=envelope(&leg,disc[..8].try_into().unwrap(),WithdrawalLeg::SPACE);
    Expected{after:out,event:WithdrawalLegInitiated{config:f.custody.accounts[CONFIG].key,sequence:r.active_sequence,leg_index:r.next_leg_index,
        jitosol_input_units:q,delegated_native_lamports:output,stake_rent_advanced_lamports:stake_rent,
        metadata_rent_advanced_lamports:metadata_rent,normalized_prefund_lamports:donation},calls,leg}
}
/// Apply only expected changed records, independently encoded above; this is a
/// callback model, not invoking any production quote/transition/persistence API.
pub fn emulate(before:&[BackingAccount],call:&Call,ix:&Instruction,infos:&[AccountInfo<'_>],seeds:&[&[&[u8]]],all:&[AccountInfo<'_>])->ProgramResult{
    assert_eq!(*ix,call.ix);assert_eq!(infos.iter().map(|a|*a.key).collect::<Vec<_>>(),call.infos.iter().map(|i|before[*i].key).collect::<Vec<_>>());
    assert_eq!(seeds.iter().map(|g|g.iter().map(|s|s.to_vec()).collect::<Vec<_>>()).collect::<Vec<_>>(),call.seeds);
    for group in seeds{let derived=Pubkey::create_program_address(group,&PROGRAM).unwrap();assert!(ix.accounts.iter().any(|m|m.pubkey==derived&&m.is_signer));}
    for(i,(old,new))in before.iter().zip(&call.after).enumerate(){if old!=new{
        assert!(ix.accounts.iter().any(|m|m.pubkey==new.key&&m.is_writable));
        **all[i].try_borrow_mut_lamports()?=new.lamports;
        if old.owner!=new.owner{all[i].assign(&new.owner);}
        if old.data.len()!=new.data.len(){*all[i].try_borrow_mut_data()?=Box::leak(new.data.clone().into_boxed_slice());}
        else{all[i].try_borrow_mut_data()?.copy_from_slice(&new.data);}
    }}Ok(())
}
