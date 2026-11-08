//! D111 production-handler tests with synthetic coherent accounts and stored
//! Squads messages. These model invocation/effects, never public signatures.
#![allow(dead_code)]
mod support;
#[path="support/pending_custody.rs"] pub mod pending_custody;
#[path="support/economic_custody.rs"] mod custody;
#[path="support/jito_identity_oracle.rs"] mod oracle;
#[path="support/bootstrap_custody.rs"] mod bootstrap_custody;
#[path="support/distribution_preparation_custody.rs"] mod preparation_custody;
#[path="support/withdrawal_preparation_custody.rs"] mod withdrawal_custody;
#[path="support/withdrawal_leg_custody.rs"] mod leg_custody;
#[path="support/withdrawal_finalization_custody.rs"] mod final_custody;
#[path="support/settlement_custody.rs"] mod settlement_custody;
#[path="support/squads_invocation.rs"] mod governance;
use anchor_lang::{prelude::{Pubkey,Clock,SolanaSysvar},AnchorDeserialize,AnchorSerialize,
    solana_program::{instruction::{Instruction,AccountMeta},program_error::ProgramError,sysvar}};
use piv1::{state::{ActiveDistribution,DistributionLifecycle,GuardianRegistry,GuardianReward},
    state::recovery::ResumeRecovery,accounts::DISTRIBUTION_DISCRIMINATOR};
use support::kif_claim_custody::{envelope,BackingAccount,PROGRAM,key};
use custody::*;
use support::kif_claim_custody;
use piv1::integrations;
fn round(c:&custody::Fixture)->ActiveDistribution{ActiveDistribution::deserialize(&mut &c.accounts[ROUND].data[8..]).unwrap()}
fn set_round(c:&mut custody::Fixture,r:&ActiveDistribution){
    c.accounts[ROUND].data=envelope(r,DISTRIBUTION_DISCRIMINATOR,r.allocated_space());
    c.accounts[ROUND].lamports=c.rent.minimum_balance(r.allocated_space());
}
fn convert(a:&governance::BackingAccount)->BackingAccount{BackingAccount{key:a.key,owner:a.owner,
    executable:a.executable,signer:a.signer,writable:a.writable,lamports:a.lamports,data:a.data.clone()}}
/// Rebuild stored action and actual Instructions sysvar from independent Borsh
/// test types. This proves exact handler/auth composition, not real voting.
fn bind(a:&mut[BackingAccount],data:&[u8],g:&governance::Fixture,roles:[usize;6]){
    let [program,ms,proposal,transaction,vault,instructions]=roles;
    let mut sorted=a.iter().collect::<Vec<_>>();sorted.sort_by_key(|v|match(v.signer,v.writable){(true,true)=>0,(true,false)=>1,(false,true)=>2,_=>3});
    let keys=sorted.iter().map(|v|v.key).collect::<Vec<_>>();
    let mut tx=g.transaction.clone();tx.message=governance::Message{num_signers:sorted.iter().filter(|v|v.signer).count()as u8,
        writable_signers:sorted.iter().filter(|v|v.signer&&v.writable).count()as u8,
        writable_non_signers:sorted.iter().filter(|v|!v.signer&&v.writable).count()as u8,
        instructions:vec![governance::Inner{program_id_index:keys.iter().position(|k|*k==a[program].key).unwrap()as u8,
            accounts:a.iter().map(|v|keys.iter().position(|k|*k==v.key).unwrap()as u8).collect(),data:data.to_vec()}],keys:keys.clone(),lookups:vec![]};
    a[transaction].data=[piv1::squads_execution::VAULT_TRANSACTION_DISCRIMINATOR.as_slice(),tx.try_to_vec().unwrap().as_slice()].concat();
    let mut metas=vec![AccountMeta::new_readonly(a[ms].key,false),AccountMeta::new(a[proposal].key,false),
        AccountMeta::new_readonly(a[transaction].key,false),AccountMeta::new_readonly(g.multisig.members[0].key,true)];
    metas.extend(keys.iter().map(|k|{let v=a.iter().find(|v|v.key==*k).unwrap();AccountMeta{pubkey:*k,
        is_signer:v.signer&&*k!=a[vault].key,is_writable:v.writable||*k==a[proposal].key}}));
    let ix=Instruction{program_id:piv1::squads_accounts::SQUADS_V4_PROGRAM_ID,accounts:metas,
        data:piv1::squads_execution::VAULT_TRANSACTION_EXECUTE_DISCRIMINATOR.to_vec()};
    let borrowed=sysvar::instructions::BorrowedInstruction{program_id:&ix.program_id,data:&ix.data,
        accounts:ix.accounts.iter().map(|m|sysvar::instructions::BorrowedAccountMeta{pubkey:&m.pubkey,is_signer:m.is_signer,is_writable:m.is_writable}).collect()};
    a[instructions].data=sysvar::instructions::construct_instructions_data(&[borrowed]);
}
fn attach_governance(c:&mut custody::Fixture,clock:&Clock,r:&ActiveDistribution)->(usize,governance::Fixture){
    let b=if c.config().manager_fee_account==c.config().referrer_token_account{18}else{19};c.accounts.truncate(b);
    let mut g=governance::Fixture::new();
    let mut members=r.guardian_keys.to_vec();members.sort();
    g.multisig.members=members.iter().map(|k|governance::Member{key:*k,permissions:7}).collect();g.sync_multisig();
    g.proposal.approved=members[..4].to_vec();g.proposal.timestamp=clock.unix_timestamp-11;g.sync_proposal();
    let cfg=c.config();
    let registry=GuardianRegistry::new(cfg.bumps.guardian_registry,cfg.guardian_registry_revision,r.guardian_keys).unwrap();
    let mut registry_acc=convert(&g.accounts[8]);registry_acc.key=cfg.guardian_registry;
    registry_acc.data=envelope(&registry,piv1::guardian_clock_accounts::GUARDIAN_REGISTRY_DISCRIMINATOR,GuardianRegistry::SPACE);
    let mut clk=convert(&g.accounts[15]);clock.to_account_info(&mut clk.info()).unwrap();
    c.accounts.push(clk);
    for i in 0..7{c.accounts.push(convert(&g.accounts[i]));}
    c.accounts.push(registry_acc);
    for i in 0..6{let mut reward=GuardianReward::new(0,&registry,i).unwrap();
        let(k,bump)=Pubkey::find_program_address(&[b"guardian-reward",reward.guardian.as_ref(),&reward.registry_revision.to_le_bytes(),&[i]],&PROGRAM);reward.bump=bump;
        let mut a=convert(&g.accounts[9+i as usize]);a.key=k;a.data=envelope(&reward,piv1::kif_claim_accounts::GUARDIAN_REWARD_DISCRIMINATOR,GuardianReward::SPACE);
        c.accounts.push(a);}
    for(i,a)in c.accounts.iter_mut().enumerate(){a.writable=i==ROUND;a.signer=i==b+6;}
    (b,g)
}
fn request(r:&ActiveDistribution)->ResumeRecovery{ResumeRecovery{vault_index:7,sequence:r.active_sequence,
    generation:r.recovery.unwrap().generation,frozen_snapshot_hash:r.frozen_recovery_hash().unwrap(),
    cumulative_loss:r.cumulative_cooldown_losses_lamports,finalized_count:r.finalized_leg_count,
    successful_count:r.successful_leg_count,assigned_units:r.cumulative_jitosol_assigned_units,
    escrow_lamports:r.recorded_escrow_available_lamports,destination:0,validator_list_index:0}}
fn resume_world(loss:u64)->(custody::Fixture,Clock,usize,governance::Fixture,Vec<u8>){
    let mut f=settlement_custody::Fixture::new(false,0b111111,true,0);
    let mut r=f.round();r.version=2;r.lifecycle=DistributionLifecycle::RecoveryRequired;
    r.recovery_flags=if loss>0{1}else{2};r.cumulative_cooldown_losses_lamports=loss;
    r.cumulative_finalized_native_lamports-=loss;r.recorded_escrow_available_lamports-=loss;
    f.custody.accounts[ESCROW_SOL].lamports-=loss;r.validate().unwrap();r=r.migrate_v2_recovery().unwrap();set_round(&mut f.custody,&r);
    let (b,g)=attach_governance(&mut f.custody,&f.clock,&r);
    let mut data=b"PIV1RR01\x01".to_vec();data.extend(request(&r).try_to_vec().unwrap());
    bind(&mut f.custody.accounts,&data,&g,[b+1,b+3,b+4,b+5,b+6,b+7]);
    (f.custody,f.clock,b,g,data)
}
fn run(c:&mut custody::Fixture,clock:Clock,data:&[u8])->Result<(),ProgramError>{let rent=c.rent.clone();
    c.with_infos(|a|piv1::recovery_execution::process_with_host_callbacks(&PROGRAM,a,data,clock,rent,2,
        |_,_,_|panic!("EscrowFunded resume has no CPI"),||None))}
#[test]
fn d111_exact_stored_four_votes_resume_loss_without_any_fund_movement(){
    for loss in [0,1,77,1000]{let(mut c,clock,_,_,data)=resume_world(loss);let before=c.clone();
        run(&mut c,clock,&data).unwrap();let r=round(&c);assert_eq!(r.lifecycle,DistributionLifecycle::EscrowFunded);
        assert_eq!(r.recovery.unwrap().acknowledged_cumulative_loss,loss);assert_eq!(r.cumulative_cooldown_losses_lamports,loss);
        for(i,(after,before))in c.accounts.iter().zip(&before.accounts).enumerate(){if i!=ROUND{assert_eq!(after,before);}else{
            assert_eq!(after.lamports,before.lamports);assert_eq!(after.owner,before.owner);}}
        let stopped=c.clone();assert!(run(&mut c,Clock::default(),&data).is_err());assert_eq!(c,stopped);
    }
}
#[test]
fn d111_exact_message_authority_custody_and_bound_state_refusals_are_atomic(){
    for case in 0..11{let(mut c,clock,b,g,mut data)=resume_world(7);
        match case{0=>data[10]^=1,1=>c.accounts[b+4].data[0]^=1,2=>c.accounts[b+2].data[13]^=1,
            3=>c.accounts[PRINCIPAL_JITO].data[64]^=1,4=>c.accounts[ESCROW_SOL].lamports+=1,
            5=>c.accounts[ROUND].writable=false,6=>c.accounts[b+6].signer=false,
            7=>{let mut gg=g.clone();gg.proposal.approved.truncate(3);gg.sync_proposal();c.accounts[b+4]=convert(&gg.accounts[3]);},
            8=>{c.accounts[b+9].data[0]^=1;},9=>{c.edit_config(|c|c.paused=true);},
            _=>{let mut r=round(&c);r.recovery.as_mut().unwrap().generation+=1;set_round(&mut c,&r);}}
        let before=c.clone();assert!(run(&mut c,clock,&data).is_err(),"case {case}");assert_eq!(c,before,"case {case}");
    }
}

fn transfer(ix:&Instruction,infos:&[anchor_lang::prelude::AccountInfo<'_>],_:&[&[&[u8]]])->Result<(),ProgramError>{
    if ix.program_id==spl_token::ID {
        assert_eq!(ix.data[0],12);let amount=u64::from_le_bytes(ix.data[1..9].try_into().unwrap());
        assert_eq!(ix.data[9],9);
        for(i,increase)in [(0,false),(2,true)] {let mut d=infos[i].try_borrow_mut_data()?;
            let old=u64::from_le_bytes(d[64..72].try_into().unwrap());
            d[64..72].copy_from_slice(&(if increase{old+amount}else{old-amount}).to_le_bytes());}
        return Ok(());
    }
    assert_eq!(ix.program_id,anchor_lang::solana_program::system_program::ID);
    assert_eq!(&ix.data[..4],&2_u32.to_le_bytes());let amount=u64::from_le_bytes(ix.data[4..].try_into().unwrap());
    **infos[0].try_borrow_mut_lamports()?-=amount;**infos[1].try_borrow_mut_lamports()?+=amount;Ok(())
}
#[test]
fn d111_recovered_production_settlement_integration_retains_loss_once_and_full_contribution(){
    for loss in [1,77,1000]{let(mut c,clock,b,_,data)=resume_world(loss);run(&mut c,clock.clone(),&data).unwrap();
        let original=settlement_custody::Fixture::new(false,0b111111,true,0);
        c.accounts.truncate(b);c.accounts.extend(original.custody.accounts[b..].iter().cloned());
        for(i,a)in c.accounts.iter_mut().enumerate(){a.writable=matches!(i,0|1|3|5|6)||i>b;a.signer=false;}
        let before=c.clone();let r=round(&c);let eligible=r.pending_sol_used_lamports+r.prior_next_cycle_yield_used_lamports().unwrap()
            +r.cumulative_finalized_delegated_native_lamports-loss;
        let net=eligible.min(r.outgoing_gross_obligation_lamports);let portion=|w:u128,cap:u64|((u128::from(net)*w/8050)as u64).min(cap);
        let h=portion(5900,r.htfp_gross_obligation_lamports);let t=portion(1950,r.team_owner_gross_obligation_lamports);let k=portion(200,r.kif_gross_obligation_lamports);
        let rent=c.rent.clone();c.with_infos(|a|piv1::settlement_execution::process_instruction_with_host_callbacks(&PROGRAM,a,b"PIV1SD01\x01",
            ||Ok(clock.clone()),||Ok(rent),transfer,|_|{})).unwrap();
        let settled=round(&c);assert_eq!(settled.lifecycle,DistributionLifecycle::Settled);
        assert_eq!((settled.actual_htfp_lamports,settled.actual_team_owner_lamports,settled.actual_kif_allocation_lamports),(h,t,k));
        assert_eq!(c.accounts[b+1].lamports-before.accounts[b+1].lamports,h);assert_eq!(c.accounts[b+2].lamports-before.accounts[b+2].lamports,t);
        let old=c.config();let full=old.accounted_pending_sol_lamports;
        let pool=original.pool();let incoming=old.accounted_pending_jitosol_units;
        let contribution=full+(u128::from(incoming)*u128::from(pool.total_lamports)/u128::from(pool.pool_token_supply))as u64;let used=r.pending_sol_used_lamports;
        let escrow=c.accounts[ESCROW_SOL].lamports-c.rent.minimum_balance(0);let principal=c.accounts[PRINCIPAL_SOL].lamports;
        let tokens=c.token_units(PRINCIPAL_JITO);c.accounts.truncate(b+1);
        for(i,a)in c.accounts.iter_mut().enumerate(){a.writable=matches!(i,0|1|2|3|5|7|8);a.signer=false;}
        let rent=c.rent.clone();c.with_infos(|a|piv1::pending_integration_execution::process_instruction_with_host_callbacks(&PROGRAM,a,b"PIV1IP01\x01",
            ||Ok(clock.clone()),||Ok(rent),transfer,|_|{})).unwrap();
        let idle=round(&c);assert_eq!(idle.lifecycle,DistributionLifecycle::Idle);assert_eq!(idle.version,3);
        assert_eq!(idle.last_completed_recovery.unwrap().cumulative_cooldown_loss,loss);
        assert_eq!(idle.last_completed.unwrap().integrated_contribution_value_lamports,contribution);
        assert_eq!(c.config().protected_principal_hwm_lamports,old.protected_principal_hwm_lamports+contribution);
        assert_eq!(c.accounts[PRINCIPAL_SOL].lamports-principal,full-used+escrow);assert_eq!(c.token_units(PRINCIPAL_JITO),tokens+incoming);
        let before=c.clone();let rent=c.rent.clone();assert!(c.with_infos(|a|piv1::pending_integration_execution::process_instruction_with_host_callbacks(&PROGRAM,a,b"PIV1IP01\x01",
            ||Ok(clock.clone()),||Ok(rent),transfer,|_|{})).is_err());assert_eq!(c,before);
    }
}

#[test]
fn d111_collect_existing_inactive_leg_only_stays_recovery_and_recovers_exact_rent(){
    let mut f=final_custody::Fixture::new(false);let mut r=f.round();r.version=2;
    // An already authenticated residual incident with this still-initiated leg.
    r.lifecycle=DistributionLifecycle::RecoveryRequired;r.recovery_flags=2;
    r=r.migrate_v2_recovery().unwrap();set_round(&mut f.custody,&r);
    let old=f.custody.clone();let leg=f.leg();let b=f.base;let index=f.index;
    let native=f.custody.accounts[b+4].lamports;let rent=f.custody.rent.clone();let clock=f.clock.clone();
    let mut data=b"PIV1RC01\x01".to_vec();data.extend(index.to_le_bytes());
    let mut calls=0;f.custody.with_infos(|a|piv1::withdrawal_finalization_execution::process_instruction_with_host_callbacks(&PROGRAM,a,&data,
        ||Ok(clock),||Ok(rent),|ix,infos,seeds|{calls+=1;if ix.program_id==piv1::accounts::STAKE_PROGRAM_ID{
            assert_eq!(&ix.data[..4],&4_u32.to_le_bytes());assert_eq!(infos[0].key,&old.accounts[b+4].key);
            assert_eq!(infos[1].key,&old.accounts[ESCROW_SOL].key);assert_eq!(seeds[0][0],b"authority");
            let amount=u64::from_le_bytes(ix.data[4..].try_into().unwrap());assert_eq!(amount,native);
            **infos[0].try_borrow_mut_lamports()?-=amount;**infos[1].try_borrow_mut_lamports()?+=amount;
            *infos[0].try_borrow_mut_data()?=Box::leak(Vec::new().into_boxed_slice());Ok(())
        }else{transfer(ix,infos,seeds)}},|_|{})).unwrap();
    assert_eq!(calls,2);let next=round(&f.custody);assert_eq!(next.lifecycle,DistributionLifecycle::RecoveryRequired);
    assert_eq!(next.finalized_leg_count,r.finalized_leg_count+1);assert_eq!(next.recovery.unwrap().generation,1);
    assert_eq!(f.custody.accounts[OPERATIONAL_SOL].lamports-old.accounts[OPERATIONAL_SOL].lamports,
        leg.stake_rent_advanced_lamports+leg.metadata_rent_advanced_lamports);
    assert_eq!(f.custody.accounts[ESCROW_SOL].lamports-old.accounts[ESCROW_SOL].lamports,native-leg.stake_rent_advanced_lamports);
    assert_eq!(f.custody.accounts[b+3].lamports,0);assert_eq!(f.custody.config().protected_principal_hwm_lamports,old.config().protected_principal_hwm_lamports);
    let before=f.custody.clone();let rent=f.custody.rent.clone();let clock=f.clock.clone();
    assert!(f.custody.with_infos(|a|piv1::withdrawal_finalization_execution::process_instruction_with_host_callbacks(&PROGRAM,a,&data,
        ||Ok(clock),||Ok(rent),|_,_,_|panic!("closed leg cannot replay"),|_|{})).is_err());assert_eq!(f.custody,before);
}

fn outer(a:&[BackingAccount],g:&governance::Fixture,roles:[usize;6])->Instruction{
    let [_,ms,proposal,transaction,vault,_]=roles;
    let mut sorted=a.iter().collect::<Vec<_>>();sorted.sort_by_key(|v|match(v.signer,v.writable){(true,true)=>0,(true,false)=>1,(false,true)=>2,_=>3});
    let mut accounts=vec![AccountMeta::new_readonly(a[ms].key,false),AccountMeta::new(a[proposal].key,false),
        AccountMeta::new_readonly(a[transaction].key,false),AccountMeta::new_readonly(g.multisig.members[0].key,true)];
    accounts.extend(sorted.iter().map(|v|AccountMeta{pubkey:v.key,is_signer:v.signer&&v.key!=a[vault].key,
        is_writable:v.writable||v.key==a[proposal].key}));
    Instruction{program_id:piv1::squads_accounts::SQUADS_V4_PROGRAM_ID,accounts,
        data:piv1::squads_execution::VAULT_TRANSACTION_EXECUTE_DISCRIMINATOR.to_vec()}
}
fn plain(a:&[BackingAccount],data:Vec<u8>)->Instruction{Instruction{program_id:PROGRAM,data,
    accounts:a.iter().map(|v|AccountMeta{pubkey:v.key,is_writable:v.writable,is_signer:v.signer}).collect()}}
#[test]
fn d111_local_bank_fixture_is_bounded_same_round_with_real_instruction_shapes(){
    use solana_stake_interface::state::StakeStateV2;
    let mut f=final_custody::Fixture::new(false);let b=f.base;
    // Explicit synthetic prior incident and fully inactive canonical Stake leg.
    // Current delegated/net deficit77 is real in this local fixture, not yield.
    let StakeStateV2::Stake(meta,mut stake,flags)=f.stake()else{panic!()};
    stake.delegation.activation_epoch=stake.delegation.deactivation_epoch;f.set_stake(&StakeStateV2::Stake(meta,stake,flags));
    f.custody.accounts[b+4].lamports-=77;
    let mut r=f.round();r.lifecycle=DistributionLifecycle::RecoveryRequired;r.recovery_flags=2;f.set_round(&r);
    let predicted=final_custody::expected(&f).after;let mut predicted_r=predicted.round();predicted_r.version=2;
    predicted_r=predicted_r.migrate_v2_recovery().unwrap();predicted_r.recovery.as_mut().unwrap().generation=2;
    r.version=2;set_round(&mut f.custody,&r);let initial=f.custody.clone();
    let mut resumed=predicted.custody;set_round(&mut resumed,&predicted_r);
    let (b,g)=attach_governance(&mut resumed,&f.clock,&predicted_r);
    let mut resume_data=b"PIV1RR01\x01".to_vec();resume_data.extend(request(&predicted_r).try_to_vec().unwrap());
    let rr=[b+1,b+3,b+4,b+5,b+6,b+7];bind(&mut resumed.accounts,&resume_data,&g,rr);
    let resume_ix=outer(&resumed.accounts,&g,rr);
    let mut mg=g.clone();let index=18_u64;mg.transaction.index=index;mg.proposal.index=index;
    for(proposal,slot)in [(false,4),(true,3)]{
        let mut seeds=vec![b"multisig".as_slice(),mg.accounts[2].key.as_ref(),b"transaction".as_slice()];
        let index_bytes=index.to_le_bytes();seeds.push(&index_bytes);if proposal{seeds.push(b"proposal");}
        let(k,bump)=Pubkey::find_program_address(&seeds,&piv1::squads_accounts::SQUADS_V4_PROGRAM_ID);
        mg.accounts[slot].key=k;if proposal{mg.proposal.bump=bump}else{mg.transaction.bump=bump};}
    mg.sync_proposal();
    let mut migration=mg.accounts[..7].iter().map(convert).collect::<Vec<_>>();
    migration.push(initial.accounts[0].clone());migration.extend(resumed.accounts[b+8..b+15].iter().cloned());
    migration.push(initial.accounts[b].clone());migration.extend(initial.accounts[1..9].iter().cloned());
    migration.push(initial.accounts[11].clone());let mut payer=initial.accounts[9].clone();payer.key=g.multisig.members[0].key;payer.lamports=10_000_000; migration.push(payer);
    for(i,a)in migration.iter_mut().enumerate(){a.writable=matches!(i,16|25);a.signer=matches!(i,5|25);}
    let mut migrate_data=b"PIV1MR03\x01\x07".to_vec();migrate_data.extend(solana_sha256_hasher::hash(&initial.accounts[1].data).to_bytes());migrate_data.extend(10_000_000_u64.to_le_bytes());
    assert_eq!(migrate_data.len(),50);bind(&mut migration,&migrate_data,&mg,[0,2,3,4,5,6]);let migrate_ix=outer(&migration,&mg,[0,2,3,4,5,6]);
    let mut collect_data=b"PIV1RC01\x01".to_vec();collect_data.extend(f.index.to_le_bytes());let mut collect_accounts=initial.accounts.clone();
    for(i,a)in collect_accounts.iter_mut().enumerate(){a.signer=false;a.writable=matches!(i,0|1|2|4|5)||i==b+3||i==b+4;}
    let collect_ix=plain(&collect_accounts,collect_data);
    let mut settlement=settlement_custody::Fixture::new(false,0b111111,true,0);
    // Keep this round's actual frozen reward records; no inferred ownership.
    let mut settlement_accounts=resumed.accounts[..b].to_vec();settlement_accounts.extend(settlement.custody.accounts[b..].iter().cloned());
    for(i,a)in settlement_accounts.iter_mut().enumerate(){a.writable=matches!(i,0|1|3|5|6)||i>b;a.signer=false;}
    let settle_ix=plain(&settlement_accounts,b"PIV1SD01\x01".to_vec());
    let mut integration=settlement_accounts[..b+1].to_vec();for(i,a)in integration.iter_mut().enumerate(){a.writable=matches!(i,0|1|2|3|5|7|8);}
    let integrate_ix=plain(&integration,b"PIV1IP01\x01".to_vec());
    // Build one compact account fixture; later-stage copies contribute ONLY new
    // external identities (governance/recipients), never overwrite initial state.
    let mut all=std::collections::BTreeMap::new();
    for a in initial.accounts.iter().chain(migration.iter()).chain(resumed.accounts.iter()).chain(settlement_accounts.iter()){
        all.entry(a.key).or_insert_with(||a.clone());}
    // Governance tx18/19 and current registry/reward identities from exact proofs.
    let instructions=[migrate_ix,collect_ix,resume_ix,settle_ix,integrate_ix];
    let mut bytes=b"PIVD1111".to_vec();bytes.extend((all.len()as u32).to_le_bytes());
    for a in all.values(){bytes.extend(a.key.to_bytes());bytes.extend(a.owner.to_bytes());bytes.extend(a.lamports.to_le_bytes());
        bytes.push(u8::from(a.executable));bytes.extend((a.data.len()as u32).to_le_bytes());bytes.extend(&a.data);}
    bytes.extend((instructions.len()as u32).to_le_bytes());
    for ix in instructions{bytes.extend(ix.program_id.to_bytes());bytes.extend((ix.accounts.len()as u32).to_le_bytes());
        for m in ix.accounts{bytes.extend(m.pubkey.to_bytes());bytes.push(u8::from(m.is_signer)|(u8::from(m.is_writable)<<1));}
        bytes.extend((ix.data.len()as u32).to_le_bytes());bytes.extend(ix.data);}
    assert!(bytes.len()<256*1024);
    if let Ok(path)=std::env::var("PIV1_RECOVERY_LOCAL_FIXTURE"){
        assert!(path.starts_with("/home/jerem/piv1-pilot-checkpoints/recovery-implementation-20261108-a/"));
        std::fs::write(path,bytes).unwrap();
    }
    // Fixture declaration, not a runtime PASS; the local Bank consumes this file.
    assert_eq!(r.version,2);assert_eq!(predicted_r.cumulative_cooldown_losses_lamports,77);
    assert_eq!(predicted_r.recovery.unwrap().generation,2);
    settlement.clock=f.clock;
}

#[test]
fn d111_ordinary_v3_tiny_no_incident_keeps_existing_dust_settlement(){
    for version in [1,3]{let mut f=settlement_custody::Fixture::liquid_gross(false,1,Some(2));
        let mut r=f.round();r.version=version;set_round(&mut f.custody,&r);
        let rent=f.custody.rent.clone();let clock=f.clock.clone();
        f.custody.with_infos(|a|piv1::settlement_execution::process_instruction_with_host_callbacks(&PROGRAM,a,b"PIV1SD01\x01",
            ||Ok(clock),||Ok(rent),transfer,|_|{})).unwrap();
        let r=round(&f.custody);assert_eq!(r.lifecycle,DistributionLifecycle::Settled);
        assert_eq!(r.actual_allocated_outgoing_lamports,0);assert_eq!(r.actual_net_allocation_dust_lamports,1);assert!(r.recovery.is_none());}
}
#[test]
fn d111_partial_resume_requires_collected_cash_and_authenticated_current_source(){
    use solana_stake_interface::state::StakeStateV2;
    let mut lf=leg_custody::Fixture::new(false);lf.source_capacity(4000);let minimum=lf.minimum;
    let StakeStateV2::Stake(meta,mut stake,flags)=lf.source()else{panic!()};stake.delegation.voter_pubkey=key(203);
    let source_key=Pubkey::find_program_address(&[key(203).as_ref(),lf.custody.config().stake_pool.as_ref()],&lf.custody.config().stake_pool_program).0;
    let mut candidate=lf.custody.accounts[lf.base+2].clone();candidate.key=source_key;
    candidate.data=borsh1::to_vec(&StakeStateV2::Stake(meta,stake,flags)).unwrap();candidate.data.resize(200,0);
    let mut f=final_custody::Fixture::from_leg(lf);let b=f.base;
    let mut r=f.round();assert!(r.remaining_withdrawal_target_units().unwrap()>0);r.version=2;r.lifecycle=DistributionLifecycle::RecoveryRequired;r.recovery_flags=2;
    r=r.migrate_v2_recovery().unwrap();set_round(&mut f.custody,&r);
    let rent=f.custody.rent.clone();let clock=f.clock.clone();let mut data=b"PIV1RC01\x01".to_vec();data.extend(f.index.to_le_bytes());
    f.custody.with_infos(|a|piv1::withdrawal_finalization_execution::process_instruction_with_host_callbacks(&PROGRAM,a,&data,
        ||Ok(clock),||Ok(rent),|ix,infos,seeds|{if ix.program_id==piv1::accounts::STAKE_PROGRAM_ID{
            let amount=u64::from_le_bytes(ix.data[4..].try_into().unwrap());**infos[0].try_borrow_mut_lamports()?-=amount;**infos[1].try_borrow_mut_lamports()?+=amount;
            *infos[0].try_borrow_mut_data()?=Box::leak(Vec::new().into_boxed_slice());Ok(())}else{transfer(ix,infos,seeds)}},|_|{})).unwrap();
    let r=round(&f.custody);let stake_program=f.custody.accounts[b+1].clone();
    let (b,g)=attach_governance(&mut f.custody,&f.clock,&r);f.custody.accounts.push(stake_program);candidate.writable=false;f.custody.accounts.push(candidate);
    let list=&mut f.custody.accounts[15];list.data[1..5].copy_from_slice(&2_u32.to_le_bytes());list.data[5..9].copy_from_slice(&2_u32.to_le_bytes());
    let mut entry=list.data[9..82].to_vec();entry[0..8].copy_from_slice(&f.custody.accounts[b+16].lamports.to_le_bytes());
    entry[16..24].copy_from_slice(&f.clock.epoch.to_le_bytes());entry[41..73].copy_from_slice(key(203).as_ref());
    f.custody.accounts[15].data.extend(entry);f.custody.accounts[15].lamports=f.custody.rent.minimum_balance(155);
    let mut q=request(&r);q.destination=1;q.validator_list_index=1;let mut data=b"PIV1RR01\x01".to_vec();data.extend(q.try_to_vec().unwrap());
    let rr=[b+1,b+3,b+4,b+5,b+6,b+7];bind(&mut f.custody.accounts,&data,&g,rr);
    for case in 0..3{let malformed=case==0;let mut c=f.custody.clone();if malformed{c.accounts[b+16].data[124]^=1;}
        let before=c.clone();let rent=c.rent.clone();let clock=f.clock.clone();let mut calls=0;
        let result=c.with_infos(|a|piv1::recovery_execution::process_with_host_callbacks(&PROGRAM,a,&data,clock,rent,2,
            |ix,_,_|{calls+=1;assert_eq!(ix.program_id,piv1::accounts::STAKE_PROGRAM_ID);assert_eq!(ix.data,13_u32.to_le_bytes());
                if case==2{a[b+4].try_borrow_mut_data()?[0]^=1;}Ok(())},
            ||Some((piv1::accounts::STAKE_PROGRAM_ID,minimum.to_le_bytes().to_vec()))));
        assert_eq!(calls,1);
        if case==2{assert!(result.is_err());assert_eq!(c.accounts[ROUND],before.accounts[ROUND]);
            assert_ne!(c.accounts[b+4].data,before.accounts[b+4].data); // injected failed host world; discard
        }else if malformed{assert!(result.is_err());assert_eq!(c,before);}else{result.unwrap();assert_eq!(round(&c).lifecycle,DistributionLifecycle::WithdrawalActive);
            for(i,(a,before))in c.accounts.iter().zip(&before.accounts).enumerate(){if i!=1{assert_eq!(a,before);}}}
    }
}

#[test]
fn d111_three_approved_accounting_examples_preserve_old_kif_and_reject_real_deficit(){
    for (loss,principal_value,extra_pending,expected,allowed) in [
        (0,1_002_950,0,(5892,1947,199,1_001_952),true),
        (100,1_002_950,0,(5819,1923,197,1_001_951),true),
        (100,990_000,5000,(5819,1923,197,1_001_951),false)]{
        let mut f=settlement_custody::Fixture::new(false,3,true,0);let mut r=f.round();let b=f.base;
        r.version=2;r.recovery=None;r.recovery_flags=if loss>0{1}else{2};r.lifecycle=DistributionLifecycle::RecoveryRequired;
        r.old_protected_principal_lamports=1_000_000;r.historical_jitosol_units=1_010_000;r.historical_sol_lamports=0;r.historical_value_lamports=1_010_000;
        r.snapshot_pool_total_lamports=10_000_000;r.snapshot_pool_token_supply=10_000_000;
        r.snapshot_withdrawal_fee_numerator=0;r.snapshot_withdrawal_fee_denominator=1;
        r.gross_yield_lamports=10_000;r.prior_next_cycle_yield_lamports=0;
        r.htfp_gross_obligation_lamports=5900;r.permanent_compound_lamports=1950;r.team_owner_gross_obligation_lamports=1950;r.kif_gross_obligation_lamports=200;
        r.split_dust_lamports=0;r.outgoing_gross_obligation_lamports=8050;r.pending_sol_snapshot_lamports=1000;r.pending_sol_used_lamports=1000;r.snapshot_conversion_dust_lamports=0;
        r.fixed_jitosol_withdrawal_target_units=7050;r.snapshot_leg_input_floor_units=25;r.maximum_useful_legs=282;r.stored_round_minimum_native_lamports=7040;
        r.stored_residual_hwm_floor_lamports=1_001_950;r.cumulative_jitosol_assigned_units=7050;r.cumulative_withdrawal_fee_units=10;r.cumulative_burned_units=7040;
        r.cumulative_expected_native_lamports=7040;r.cumulative_delegated_native_lamports=7040;r.cumulative_finalized_delegated_native_lamports=7040;
        r.cumulative_finalized_native_lamports=7040-loss;r.cumulative_recovered_stake_rent_lamports=0;r.cumulative_recovered_metadata_rent_lamports=0;
        r.cumulative_cooldown_rewards_lamports=0;r.cumulative_cooldown_losses_lamports=loss;r.next_leg_index=2;r.successful_leg_count=2;r.finalized_leg_count=2;
        r.recorded_escrow_available_lamports=8040-loss;r.outstanding_active_round_liability_lamports=8050;r.kif_carry_input_lamports=0;
        r.proposed_hwm_delta_lamports=1950;r.proposed_hwm_after_settlement_lamports=1_001_950;r.validate().unwrap();
        r=r.migrate_v2_recovery().unwrap();set_round(&mut f.custody,&r);
        f.custody.edit_config(|c|{c.protected_principal_hwm_lamports=1_000_000;c.accounted_historical_jitosol_units=1_010_000;c.accounted_historical_sol_lamports=0;
            c.accounted_pending_sol_lamports=1000+extra_pending;c.accounted_pending_jitosol_units=0;c.kif_claim_liability_lamports=600;c.cumulative_kif_credited_lamports=600;c.cumulative_kif_claimed_lamports=0;c.collective_kif_carry_lamports=0;});
        let floor=f.custody.rent.minimum_balance(0);f.custody.accounts[PRINCIPAL_SOL].lamports=floor;
        f.custody.accounts[PENDING_SOL].lamports=floor+extra_pending;f.custody.accounts[ESCROW_SOL].lamports=floor+8040-loss;f.custody.accounts[KIF_SOL].lamports=floor+600;
        f.custody.set_token_units(PRINCIPAL_JITO,1_002_950);f.custody.set_token_units(PENDING_JITO,0);
        let mut p=f.pool();p.total_lamports=principal_value;p.pool_token_supply=1_002_950;f.set_pool(&p);
        f.custody.accounts[MINT].data[36..44].copy_from_slice(&1_002_950_u64.to_le_bytes());
        for i in 0..6{let mut reward=f.reward(i);reward.cumulative_earned=if i==0{600}else{0};reward.cumulative_claimed=0;reward.claimable_lamports=if i==0{600}else{0};f.set_reward(i,&reward);}
        let frozen_rewards=f.custody.accounts[b+3..b+9].to_vec();let recipients=f.custody.accounts[b+1..b+3].to_vec();let clk=f.custody.accounts[b].clone();
        let (b,g)=attach_governance(&mut f.custody,&f.clock,&r);let mut data=b"PIV1RR01\x01".to_vec();data.extend(request(&r).try_to_vec().unwrap());
        bind(&mut f.custody.accounts,&data,&g,[b+1,b+3,b+4,b+5,b+6,b+7]);let before=f.custody.clone();
        let result=run(&mut f.custody,f.clock.clone(),&data);
        if !allowed{assert!(result.is_err());assert_eq!(f.custody,before);continue;}result.unwrap();
        f.custody.accounts.truncate(b);f.custody.accounts.push(clk);f.custody.accounts.extend(recipients);f.custody.accounts.extend(frozen_rewards);
        for(i,a)in f.custody.accounts.iter_mut().enumerate(){a.writable=matches!(i,0|1|3|5|6)||i>b;a.signer=false;}
        let rent=f.custody.rent.clone();let clock=f.clock.clone();f.custody.with_infos(|a|piv1::settlement_execution::process_instruction_with_host_callbacks(&PROGRAM,a,b"PIV1SD01\x01",
            ||Ok(clock),||Ok(rent),transfer,|_|{})).unwrap();let settled=round(&f.custody);
        assert_eq!((settled.actual_htfp_lamports,settled.actual_team_owner_lamports,settled.actual_kif_allocation_lamports,settled.settled_protected_hwm_lamports),expected);
        assert_eq!(f.reward(0).cumulative_earned,600+settled.actual_kif_liability_lamports/2);
        assert_eq!(f.custody.config().kif_claim_liability_lamports,600+settled.actual_kif_liability_lamports);
    }
}

fn finalize_host(f:&mut final_custody::Fixture,recovery:bool,immature:bool)->Result<(),ProgramError>{
    let mut data=if recovery{b"PIV1RC01\x01".to_vec()}else{b"PIV1FL01\x01".to_vec()};data.extend(f.index.to_le_bytes());
    let clock=f.clock.clone();let rent=f.custody.rent.clone();
    f.custody.with_infos(|a|piv1::withdrawal_finalization_execution::process_instruction_with_host_callbacks(&PROGRAM,a,&data,
        ||Ok(clock),||Ok(rent),|ix,infos,seeds|{if ix.program_id==piv1::accounts::STAKE_PROGRAM_ID{
            if immature{return Err(ProgramError::Custom(24811));}let amount=u64::from_le_bytes(ix.data[4..].try_into().unwrap());
            **infos[0].try_borrow_mut_lamports()?-=amount;**infos[1].try_borrow_mut_lamports()?+=amount;
            *infos[0].try_borrow_mut_data()?=Box::leak(Vec::new().into_boxed_slice());Ok(())
        }else{transfer(ix,infos,seeds)}},|_|{}))
}
#[test]
fn d111_multiple_legs_loss_entry_immature_pause_and_closed_metadata_refusals(){
    use solana_stake_interface::state::StakeStateV2;
    let mut lf=leg_custody::Fixture::new(false);lf.source_capacity(4000);
    lf.custody.accounts[OPERATIONAL_SOL].lamports+=lf.custody.rent.minimum_balance(200)+lf.custody.rent.minimum_balance(piv1::state::WithdrawalLeg::SPACE);
    let list=&mut lf.custody.accounts[15];list.data[1..5].copy_from_slice(&2_u32.to_le_bytes());list.data[5..9].copy_from_slice(&2_u32.to_le_bytes());
    let mut entry=list.data[9..82].to_vec();entry[41..73].copy_from_slice(key(203).as_ref());list.data.extend(entry);list.lamports=lf.custody.rent.minimum_balance(155);
    let mut lf=leg_custody::expected(&lf).after;let first=lf.custody.accounts[lf.base+4..].to_vec();
    let StakeStateV2::Stake(meta,mut stake,flags)=lf.source()else{panic!()};stake.delegation.voter_pubkey=key(203);stake.delegation.stake=1_004_000;
    lf.set_source(StakeStateV2::Stake(meta,stake,flags));lf.custody.accounts[lf.base+2].key=Pubkey::find_program_address(&[key(203).as_ref(),lf.custody.config().stake_pool.as_ref()],&lf.custody.config().stake_pool_program).0;
    lf.custody.accounts[lf.base+2].lamports=meta.rent_exempt_reserve+1_004_000;lf.index=1;lf.new_pair();
    let mut f=final_custody::Fixture::from_leg(lf);let b=f.base;let second=f.custody.accounts[b+3..].to_vec();
    f.custody.accounts[b+3..].clone_from_slice(&first);f.index=0;f.custody.accounts[b+4].lamports-=1;
    let mut r=f.round();r.version=3;set_round(&mut f.custody,&r);finalize_host(&mut f,false,false).unwrap();
    let r=round(&f.custody);assert_eq!(r.lifecycle,DistributionLifecycle::RecoveryRequired);assert_eq!(r.cumulative_cooldown_losses_lamports,1);
    assert_eq!(r.recovery.unwrap().generation,1);assert_eq!(r.finalized_leg_count,1);
    f.custody.accounts[b+3..].clone_from_slice(&second);f.index=1;
    let before=f.clone();assert!(finalize_host(&mut f,false,false).is_err());assert_eq!(f,before);
    assert_eq!(finalize_host(&mut f,true,true),Err(ProgramError::Custom(24811)));assert_eq!(f,before);
    f.custody.edit_config(|c|c.paused=true);let paused=f.clone();assert!(finalize_host(&mut f,true,false).is_err());assert_eq!(f,paused);f.custody.edit_config(|c|c.paused=false);
    finalize_host(&mut f,true,false).unwrap();let r=round(&f.custody);assert_eq!(r.finalized_leg_count,2);
    assert_eq!(r.cumulative_cooldown_losses_lamports,1);assert_eq!(r.recovery.unwrap().generation,1);assert_eq!(r.lifecycle,DistributionLifecycle::RecoveryRequired);
    let before=f.clone();assert!(finalize_host(&mut f,true,false).is_err());assert_eq!(f,before);
}
#[test]
fn d111_new_committed_settlement_incident_invalidates_ack_even_without_new_loss(){
    let(mut c,clock,b,_,data)=resume_world(77);run(&mut c,clock.clone(),&data).unwrap();
    let original=settlement_custody::Fixture::new(false,0b111111,true,0);
    c.accounts.truncate(b);c.accounts.extend(original.custody.accounts[b..].iter().cloned());
    for(i,a)in c.accounts.iter_mut().enumerate(){a.writable=matches!(i,0|1|3|5|6)||i>b;a.signer=false;}
    let mut pool=original.pool();pool.total_lamports/=2;let bytes=borsh1::to_vec(&pool).unwrap();c.accounts[14].data[..bytes.len()].copy_from_slice(&bytes);
    let before=c.clone();let rent=c.rent.clone();c.with_infos(|a|piv1::settlement_execution::process_instruction_with_host_callbacks(&PROGRAM,a,b"PIV1SD01\x01",
        ||Ok(clock),||Ok(rent),|_,_,_|panic!("deficit must not pay"),|_|{})).unwrap();
    let r=round(&c);assert_eq!(r.lifecycle,DistributionLifecycle::RecoveryRequired);assert_eq!(r.recovery.unwrap().generation,2);
    assert_eq!(r.recovery.unwrap().acknowledged_generation,1);assert_eq!(r.cumulative_cooldown_losses_lamports,77);
    for(i,(a,before))in c.accounts.iter().zip(&before.accounts).enumerate(){if i!=ROUND{assert_eq!(a,before);}}
}

#[cfg(not(feature="testnet-bounded-recovery"))]
#[test]
fn d111_default_build_cannot_dispatch_testnet_recovery(){
    for data in [b"PIV1MR03\x01\x07".as_slice(),b"PIV1RR01\x01".as_slice(),b"PIV1RC01\x01".as_slice()]{
        assert_eq!(piv1::instruction_boundary::process_instruction(&PROGRAM,&[],data),Err(ProgramError::InvalidInstructionData));
    }
}
