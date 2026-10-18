//! Real production ELF and real Token/pool/Stake execution through local Bank.
//! Unsigned entry, synthetic external genesis/governance; no live deployment.
mod lifecycle_fixture;
use lifecycle_fixture::*;
use serde_json::{json,Value};
use sha2::{Digest,Sha256};
use solana_account::{AccountSharedData,ReadableAccount,WritableAccount};
use solana_compute_budget_interface::ComputeBudgetInstruction;
use solana_genesis_config::GenesisConfig;
use solana_instruction::Instruction;
use solana_instruction_error::InstructionError;
use solana_loader_v3_interface::state::UpgradeableLoaderState;
use solana_message::Message;
use solana_pubkey::Pubkey;
use solana_runtime::bank::{Bank,BankTestConfig,SlotLeader};
use solana_sdk_ids::{bpf_loader_upgradeable,system_program,sysvar};
use solana_svm::{transaction_commit_result::TransactionCommitResult,transaction_processor::ExecutionRecordingConfig};
use solana_transaction::Transaction;
use solana_transaction_error::TransactionError;
use std::{collections::BTreeMap,env,fs,num::NonZeroUsize,path::Path};
type Accounts=BTreeMap<Pubkey,AccountSharedData>;
const LIMIT:u32=1_400_000;
const HEAP:u32=262_144;
// Pinned agave-feature-set 4.2.0 get_sysvar_syscall_enabled (SIMD-0127).
// Bind the public ID without adding a dependency just for its declaration.
const GET_SYSVAR_FEATURE:Pubkey=Pubkey::from_str_const("CLCoTADvV64PSrnR6QXty6Fwrt9Xc6EdxSJE4wLRePjq");
// Stake 5.1 uses the 9% rate from epoch zero and fixed-point stake_v2 math.
const REDUCED_COOLDOWN_FEATURE:Pubkey=Pubkey::from_str_const("GwtDQBghCTBgmX2cpEGNPxTEBUTQRaDMGTr5qychdGMj");
const STAKE_V5_1_FEATURE:Pubkey=Pubkey::from_str_const("s51VGwCAgebo2745DSUris72RavoLkXGUmVJosESCXr");
const REQUIRED_FEATURES:[Pubkey;3]=[GET_SYSVAR_FEATURE,REDUCED_COOLDOWN_FEATURE,STAKE_V5_1_FEATURE];
// Pinned integer cooldown recurrence for the asserted fixture below: epoch 13
// retains [75_545_467,54_523_224]; epoch 14 reaches [0,0]. This is only a bound;
// real full Stake withdrawal remains the sole runtime readiness decision.
const FINALIZATION_EPOCH_BOUND:u64=14;
fn hex(b:&[u8])->String{b.iter().map(|v|format!("{v:02x}")).collect()}
fn emit(v:Value){println!("PIV1_LIFECYCLE_EVIDENCE {v}");}
fn assert_runtime_capabilities(bank:&Bank,phase:&str,capability_accounts:&Accounts){
    assert_eq!(bank.feature_set.active().len(),REQUIRED_FEATURES.len(),"exact synthetic capability profile: {phase}");
    for feature in REQUIRED_FEATURES{assert_eq!(bank.feature_set.activated_slot(&feature),Some(0));}
    assert!(bank.feature_set.snapshot().get_sysvar_syscall_enabled);
    assert!(bank.feature_set.is_active(&REDUCED_COOLDOWN_FEATURE));
    assert!(bank.feature_set.snapshot().upgrade_bpf_stake_program_to_v5_1);
    assert_eq!(bank.new_warmup_cooldown_rate_epoch(),Some(0));
    // Complete canonical feature accounts and unchanged loaded Stake executable,
    // ProgramData bytes and authority must survive genesis and every transition.
    for(key,expected)in capability_accounts{assert_eq!(bank.get_account(key).as_ref(),Some(expected),"capability account {key}: {phase}");}
    emit(json!({"kind":"feature_profile","phase":phase,"slot":bank.slot(),"epoch":bank.epoch(),
        "synthetic":true,"active_features":REQUIRED_FEATURES.map(|id|json!({"id":id.to_string(),"activated_at":0})),
        "get_sysvar_syscall_enabled":true,"reduce_stake_warmup_cooldown":true,
        "upgrade_bpf_stake_program_to_v5_1":true,"new_warmup_cooldown_rate_epoch":0,
        "feature_accounts_match_canonical_genesis":true,"stake_program_accounts_match_loaded_genesis":true}));
}
fn snapshot(bank:&Bank,phase:&str)->Accounts{
    let mut scanned=Accounts::new();bank.scan_all_accounts(|v|{if let Some((key,a,_))=v{assert!(scanned.insert(*key,a).is_none());}}).unwrap();
    let mut result=Accounts::new();for(key,a)in scanned{let read=bank.get_account(&key);if a.lamports()==0{assert!(read.is_none());continue;}
        assert_eq!(read.as_ref(),Some(&a));emit(json!({"kind":"account","phase":phase,"slot":bank.slot(),"pubkey":key.to_string(),"key_hex":hex(key.as_ref()),
            "owner":a.owner().to_string(),"owner_hex":hex(a.owner().as_ref()),"lamports":a.lamports(),"rent_epoch":a.rent_epoch(),"executable":a.executable(),"data_hex":hex(a.data())}));result.insert(key,a);}
    emit(json!({"kind":"snapshot","phase":phase,"slot":bank.slot(),"epoch":bank.epoch(),"count":result.len()}));result
}
fn loader(genesis:&mut GenesisConfig,role:&str,program:Pubkey,authority:Option<Pubkey>){
    let path=env::var(format!("PIV_LIFECYCLE_{role}_PATH")).unwrap();let hash=env::var(format!("PIV_LIFECYCLE_{role}_SHA256")).unwrap();
    let bytes=env::var(format!("PIV_LIFECYCLE_{role}_BYTES")).unwrap().parse::<usize>().unwrap();let elf=fs::read(&path).unwrap();
    assert_eq!(elf.len(),bytes);assert_eq!(format!("{:x}",Sha256::digest(&elf)),hash);assert_eq!(&elf[..4],b"\x7fELF");
    let pd=program_data(program);let program_bytes=bincode::serialize(&UpgradeableLoaderState::Program{programdata_address:pd}).unwrap();
    let mut data=bincode::serialize(&UpgradeableLoaderState::ProgramData{slot:0,upgrade_authority_address:authority}).unwrap();data.resize(45,0);data.extend(&elf);
    for(k,d,executable)in[(program,program_bytes,true),(pd,data,false)]{let mut a=account(bpf_loader_upgradeable::id(),genesis.rent.minimum_balance(d.len()),d);a.set_executable(executable);genesis.accounts.insert(k,a.into());}
    emit(json!({"kind":"artifact","role":role,"program":program.to_string(),"sha256":hash,"bytes":bytes,"path":path}));
}
fn genesis(w:&World,extras:Vec<(Pubkey,AccountSharedData)>)->GenesisConfig{
    let mut g=GenesisConfig{creation_time:100,..GenesisConfig::default()};g.rent.lamports_per_byte=6960;
    // Stake 5.1 queries EpochRewards before dispatch. Use the runtime's real
    // inactive genesis sysvar; a missing cache entry aborts the syscall in Agave.
    solana_runtime::genesis_utils::add_genesis_epoch_rewards_account(&mut g);
    g.fee_rate_governor.lamports_per_signature=5000;g.fee_rate_governor.target_lamports_per_signature=5000;
    g.fee_rate_governor.target_signatures_per_slot=0;g.fee_rate_governor.min_lamports_per_signature=5000;g.fee_rate_governor.max_lamports_per_signature=5000;
    for r in &w.roles{if r.lamports==0||[sysvar::instructions::id(),system_program::id(),TOKEN,PROGRAM,program_data(PROGRAM),JITO_PROGRAM].contains(&r.key){continue;}g.accounts.insert(r.key,r.account().into());}
    for(k,a)in extras{g.accounts.insert(k,a.into());}
    g.accounts.insert(key(242),account(system_program::id(),1000*SOL,vec![]).into());
    for(role,k,a)in[("PIV",PROGRAM,Some(w.vault)),("CALLER",SQUADS,None),("TOKEN",TOKEN,None),("POOL",JITO_PROGRAM,None),("STAKE",STAKE,None)]{loader(&mut g,role,k,a);}
    // Align the Bank with the unchanged Stake 5.1 ELF: history syscall, 9% rate,
    // and fixed-point history arithmetic. Canonical Some(0) accounts are already
    // active, not pending: Bank's epoch-boundary upgrade-buffer path only runs
    // for newly activated pending features, so no program migration is requested.
    for feature in REQUIRED_FEATURES{
        assert!(!g.accounts.contains_key(&feature));
        solana_runtime::genesis_utils::activate_feature(&mut g,feature);
    }
    g
}
fn bounded()->BankTestConfig{let mut c=BankTestConfig::default();let d=&mut c.accounts_db_config;d.num_foreground_threads=NonZeroUsize::new(1);d.num_background_threads=NonZeroUsize::new(1);
    d.read_cache_limit_bytes=Some((2*1024*1024,4*1024*1024));d.read_cache_num_shards=Some(16);d.read_cache_evict_sample_size=Some(4);d.write_cache_limit_bytes=Some(8*1024*1024);
    let i=d.index.as_mut().unwrap();i.bins=Some(2);i.num_flush_threads=NonZeroUsize::new(1);i.num_initial_accounts=Some(128);c}
fn tx(bank:&Bank,ixs:Vec<Instruction>,nonce:u32)->Transaction{
    // A changing CU limit yields distinct unsigned messages without signatures.
    let mut all=vec![ComputeBudgetInstruction::set_compute_unit_limit(LIMIT-nonce),ComputeBudgetInstruction::request_heap_frame(HEAP)];all.extend(ixs);
    Transaction::new_unsigned(Message::new_with_blockhash(&all,Some(&key(242)),&bank.last_blockhash()))
}
// Read-only diagnosis of a Bank cache assertion. No cache flush, insertion,
// account write or environment replacement occurs. The failing panic is resumed.
fn cache_diagnostic(bank:&Bank,phase:&str,t:&Transaction){
    let processor=bank.get_transaction_processor();
    let cache_poisoned=processor.global_program_cache.is_poisoned();
    let cache=processor.global_program_cache.read().unwrap_or_else(|error|error.into_inner());
    let graph=cache.fork_graph.as_ref().unwrap().upgrade().unwrap();
    let graph=graph.read().unwrap_or_else(|error|error.into_inner());
    let root=graph.root();let highest=graph.highest_slot();let current=bank.slot();
    let registered=graph.get(current);let descendants=graph.descendants();
    // Literal read-only equivalent of pinned BankForks::relationship. Its trait
    // lives in an indirect dependency; emit its inputs and label this derivation.
    let relationship=|slot:u64|{
        if !(root..=highest).contains(&slot)||!(root..=highest).contains(&current){"Unknown"}
        else if slot==current{"Equal"}
        else if registered.as_ref().is_some_and(|b|b.proper_ancestors_set().contains(&slot)){"Ancestor"}
        else if descendants.get(&current).is_some_and(|slots|slots.contains(&slot)){"Descendant"}
        else{"Unrelated"}
    };
    let mut entries=Vec::new();
    for key in &t.message.account_keys{
        let account=bank.get_account(key);
        let metadata=account.as_ref().map(|a|json!({"owner":a.owner().to_string(),"executable":a.executable(),
            "data_bytes":a.data().len(),"prefix_hex":hex(&a.data()[..a.data().len().min(12)])}));
        let versions=cache.get_slot_versions_for_tests(key).iter().map(|entry|json!({
            "type":format!("{:?}",entry.program),"account_owner":format!("{:?}",entry.account_owner),
            "deployment_slot":entry.deployment_slot,"effective_slot":entry.effective_slot,
            "environment_matches_current":entry.program.get_environment().map(|env|env==&processor.program_runtime_environment),
            "derived_fork_relationship":relationship(entry.deployment_slot),
            "root_or_fork_visible":entry.deployment_slot<=cache.latest_root_slot||matches!(relationship(entry.deployment_slot),"Equal"|"Ancestor")
        })).collect::<Vec<_>>();
        entries.push(json!({"key":key.to_string(),"account":metadata,"versions":versions}));
    }
    emit(json!({"kind":"cache_diagnostic","phase":phase,"slot":current,"epoch":bank.epoch(),
        "cache_poisoned":cache_poisoned,"cache_root_slot":cache.latest_root_slot,"fork_root":root,"fork_highest":highest,
        "current_bank_registered":registered.is_some(),"current_bank_actual_ancestors":bank.proper_ancestors_set(),
        "relationship_source":"literal pinned BankForks::relationship from read-only inputs","entries":entries}));
}
// Regression for the exact ProgramData tombstone failure reproduced in K.
// These are observations only; canonical BankForks insertion is performed by
// the epoch loop. The original slot-one entry must remain visible and unchanged.
fn assert_registered_cache(bank:&Bank,phase:&str){
    let processor=bank.get_transaction_processor();
    let cache=processor.global_program_cache.read().unwrap();
    let graph=cache.fork_graph.as_ref().unwrap().upgrade().unwrap();let graph=graph.read().unwrap();
    let current=graph.get(bank.slot()).expect("current Bank is registered in cache fork graph");
    assert!(std::ptr::eq(current.as_ref(),bank));
    assert_eq!(graph.root(),0);assert_eq!(cache.latest_root_slot,0);
    assert!(graph.highest_slot()>=bank.slot());
    let ancestors=bank.proper_ancestors_set();
    assert_eq!(graph.ancestors().get(&bank.slot()),Some(&ancestors));
    for slot in &ancestors{assert!(graph.get(*slot).is_some(),"ancestor {slot} is registered");}
    let versions=cache.get_slot_versions_for_tests(&program_data(PROGRAM));
    assert_eq!(versions.len(),1,"original metadata tombstone is retained without replacement");
    let entry=&versions[0];assert_eq!(format!("{:?}",entry.program),"ProgramCacheEntryType::Closed");
    assert_eq!(entry.deployment_slot,1);assert_eq!(entry.effective_slot,1);
    assert!(entry.program.get_environment().is_none());
    assert!(ancestors.contains(&entry.deployment_slot),"original tombstone is on the registered ancestor branch");
    emit(json!({"kind":"cache_registration","phase":phase,"slot":bank.slot(),"epoch":bank.epoch(),
        "current_bank_registered":true,"all_ancestors_registered":true,"programdata_key":program_data(PROGRAM).to_string(),
        "versions":1,"type":format!("{:?}",entry.program),"deployment_slot":entry.deployment_slot,
        "effective_slot":entry.effective_slot,"ancestor_visible":true}));
}
fn assert_zero_floor_event(logs:&[String],case:&str,committed:bool){
    // Independent literal Anchor event encoding. Exactly 57 bytes need no base64
    // padding; compare the actual production log, not a model-produced value.
    let mut payload=Sha256::digest(b"event:DistributionSettlementRecovery")[..8].to_vec();
    payload.extend(target(0).as_ref());payload.extend(0_u64.to_le_bytes());payload.extend(0_u64.to_le_bytes());payload.push(2);
    assert_eq!(payload.len(),57);let alphabet=b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";let mut encoded=String::new();
    for bytes in payload.chunks_exact(3){let word=(u32::from(bytes[0])<<16)|(u32::from(bytes[1])<<8)|u32::from(bytes[2]);
        for shift in [18,12,6,0]{encoded.push(alphabet[((word>>shift)&63) as usize] as char);}}
    let expected=format!("Program data: {encoded}");assert_eq!(logs.iter().filter(|line|**line==expected).count(),1,"exact real recovery event");
    emit(json!({"kind":"settlement_recovery_event","case":case,"transaction_succeeded":committed,
        "event_data_hex":hex(&payload),"observed_protected_value":0,"recovery_flags":2}));
}
fn execute(bank:&Bank,name:&str,t:&Transaction)->TransactionCommitResult{
    assert!(t.signatures.iter().all(|s|s.as_ref().iter().all(|v|*v==0)));
    let fees=u64::from(t.message.header.num_required_signatures)*5000;
    emit(json!({"kind":"transaction","case":name,"slot":bank.slot(),"epoch":bank.epoch(),"message_hex":hex(&t.message.serialize()),"message_hash":t.message.hash().to_string(),
        "unsigned":true,"signatures":t.signatures.len(),"expected_fee":fees,"heap_bytes":HEAP,"packet_bytes":1+t.signatures.len()*64+t.message.serialize().len()}));
    let diagnose=name=="mature-finalization-pause";
    if diagnose{assert_registered_cache(bank,"before-mature-pause");cache_diagnostic(bank,"before-mature-pause",t);}
    let batch=bank.prepare_entry_batch(vec![t.clone().into()]).unwrap();
    let run=||bank.load_execute_and_commit_transactions(&batch,ExecutionRecordingConfig::new_single_setting(true),&mut Default::default(),Some(512*1024));
    let(mut results,_)=if diagnose{
        match std::panic::catch_unwind(std::panic::AssertUnwindSafe(run)){
            Ok(result)=>{assert_registered_cache(bank,"after-mature-pause");cache_diagnostic(bank,"after-mature-pause",t);result},
            Err(payload)=>{
                let _=std::panic::catch_unwind(std::panic::AssertUnwindSafe(||cache_diagnostic(bank,"panic-mature-pause",t)));
                std::panic::resume_unwind(payload);
            }
        }
    }else{run()};
    assert_eq!(results.len(),1);let result=results.remove(0);drop(batch);
    match &result{Ok(c)=>{assert_eq!(c.fee_details.total_fee(),fees);assert!(c.executed_units>0&&c.executed_units<=u64::from(LIMIT));
        emit(json!({"kind":"commit","case":name,"status":format!("{:?}",c.status),"fee":fees,"compute_units":c.executed_units,"logs":c.log_messages}));
        if matches!(name,"severe-settlement"|"severe-settlement-late-failure"){
            assert!(c.inner_instructions.as_ref().unwrap().iter().all(|inner|inner.is_empty()),"no CPI in payment-free recovery");
            assert_zero_floor_event(c.log_messages.as_ref().unwrap(),name,c.status.is_ok());
        }
        for(outer,inners)in c.inner_instructions.as_ref().unwrap().iter().enumerate(){for(ordinal,inner)in inners.iter().enumerate(){let ix=&inner.instruction;
            emit(json!({"kind":"cpi","case":name,"outer":outer,"ordinal":ordinal,"height":inner.stack_height,
                "program":t.message.account_keys[ix.program_id_index as usize].to_string(),"accounts":ix.accounts.iter().map(|i|t.message.account_keys[*i as usize].to_string()).collect::<Vec<_>>(),"data_hex":hex(&ix.data)}));}}
    },Err(e)=>emit(json!({"kind":"commit","case":name,"rejected":format!("{e:?}")}))}result
}
fn debit(a:&mut Accounts,k:Pubkey,n:u64){let v=a.get_mut(&k).unwrap();v.set_lamports(v.lamports().checked_sub(n).unwrap());}
fn credit(a:&mut Accounts,k:Pubkey,n:u64){let v=a.get_mut(&k).unwrap();v.set_lamports(v.lamports().checked_add(n).unwrap());}
fn bytes(a:&mut Accounts,k:Pubkey,b:Vec<u8>){a.get_mut(&k).unwrap().set_data_from_slice(&b);}
fn token_delta(a:&mut Accounts,k:Pubkey,n:i64){let v=units(&a[&k]);let mut b=a[&k].data().to_vec();put64(&mut b,64,u64::try_from(i128::from(v)+i128::from(n)).unwrap());bytes(a,k,b);}
fn config(a:&Accounts)->Config{Config::read(&a[&target(0)])}
fn round(a:&Accounts)->Round{Round::read(&a[&target(1)])}
fn write_config(a:&mut Accounts,c:Config){bytes(a,target(0),c.bytes());}
fn success(bank:&Bank,name:&str,ixs:Vec<Instruction>,nonce:&mut u32)->(Accounts,Accounts){
    let before=snapshot(bank,&format!("{name}-before"));let t=tx(bank,ixs,*nonce);*nonce+=1;let c=execute(bank,name,&t).expect("executed transaction");assert_eq!(c.status,Ok(()),"{name}");
    let after=snapshot(bank,&format!("{name}-after"));assert_eq!(before[&key(242)].lamports()-after[&key(242)].lamports(),c.fee_details.total_fee());(before,after)
}
fn rollback(bank:&Bank,name:&str,ixs:Vec<Instruction>,nonce:&mut u32,successful_program:Option<Pubkey>){
    let before=snapshot(bank,&format!("{name}-before"));let t=tx(bank,ixs,*nonce);*nonce+=1;let c=execute(bank,name,&t).expect("executed rejected transaction");assert!(c.status.is_err(),"expected {name} to reject");
    if let Some(p)=successful_program{assert_eq!(c.status,Err(TransactionError::InstructionError(3,InstructionError::InvalidInstructionData)));
        assert!(c.log_messages.as_ref().unwrap().contains(&format!("Program {p} success")),"real inner program completed before outer failure");}
    else if name=="active-finalization"{inactive_failure(&c,&t);}
    else{assert_eq!(c.status,Err(TransactionError::InstructionError(2,InstructionError::Custom(6034))));}
    let after=snapshot(bank,&format!("{name}-after"));let mut expected=before;debit(&mut expected,key(242),c.fee_details.total_fee());assert_eq!(after,expected,"all non-fee account rollback");
    assert!(matches!(execute(bank,&format!("{name}-replay"),&t),Err(TransactionError::AlreadyProcessed)));
    assert_eq!(snapshot(bank,&format!("{name}-replayed")),after);
}
// Deliberately synthetic external shock, never a production instruction or
// evidence of live slashing. The balanced pair preserves capitalization and all
// account bytes; only the existing Stake/donor lamport balances change.
fn external_cooldown_loss(bank:&Bank){
    let before=snapshot(bank,"external-cooldown-loss-before");let capitalization=bank.capitalization();
    let mut expected=before.clone();assert_eq!(before[&stake(0)].owner(),&STAKE);
    assert_eq!(before[&key(220)].owner(),&system_program::id());
    assert_eq!(before[&stake(0)].lamports(),5*SOL+STAKE_RENT);
    debit(&mut expected,stake(0),1);credit(&mut expected,key(220),1);
    assert!(expected[&stake(0)].lamports()>=STAKE_RENT);
    let stake_key=stake(0);let donor_key=key(220);
    bank.store_accounts((bank.slot(),&[(&stake_key,&expected[&stake_key]),(&donor_key,&expected[&donor_key])][..]),None);
    let after=snapshot(bank,"external-cooldown-loss-after");assert_eq!(after,expected);
    let total=|accounts:&Accounts|accounts.values().map(|a|u128::from(a.lamports())).sum::<u128>();
    assert_eq!(total(&before),total(&after));assert_eq!(bank.capitalization(),capitalization);
    emit(json!({"kind":"external_cooldown_loss","synthetic":true,"production_instruction":false,"live_slashing_proof":false,
        "stake":stake_key.to_string(),"recipient":donor_key.to_string(),"lamports":1,"slot":bank.slot(),"epoch":bank.epoch(),
        "capitalization_before":capitalization,"capitalization_after":bank.capitalization(),"all_data_unchanged":true,"all_piv_accounts_unchanged":true}));
}
// Synthetic catastrophic external-pool deficit, distinct from loss on PIV's
// withdrawn temporary Stake. Never mutate PIV state, pool accounting or any data
// bytes. The real SPL maintenance instructions alone publish the changed ratio.
fn external_pool_loss(bank:&Bank,nonce:&mut u32){
    assert_eq!(bank.epoch(),14);let before=snapshot(bank,"external-pool-loss-before");let capitalization=bank.capitalization();
    assert_eq!(pool_ratio(&before[&JITO_POOL]),(1_191_391_363_639,1_092_108_750_001,14));
    assert_eq!(units(&before[&target(14)]),111_108_750_001);
    let mut expected=before.clone();let mut lost=0_u64;
    for(key,balance)in[(key(202),1_187_997_717_120),(source(0),1_002_282_880),(source(1),2_393_646_519)]{
        assert_eq!(before[&key].owner(),&STAKE);assert_eq!(before[&key].lamports(),balance);
        assert_eq!(u64at(before[&key].data(),4),STAKE_RENT);
        let delta=balance-STAKE_RENT;debit(&mut expected,key,delta);lost=lost.checked_add(delta).unwrap();
    }
    assert_eq!(lost,1_191_386_797_879);assert_eq!(before[&key(220)].owner(),&system_program::id());credit(&mut expected,key(220),lost);
    let reserve=key(202);let first=source(0);let second=source(1);let recipient=key(220);
    bank.store_accounts((bank.slot(),&[(&reserve,&expected[&reserve]),(&first,&expected[&first]),
        (&second,&expected[&second]),(&recipient,&expected[&recipient])][..]),None);
    let after=snapshot(bank,"external-pool-loss-after");assert_eq!(after,expected);
    let total=|a:&Accounts|a.values().map(|v|u128::from(v.lamports())).sum::<u128>();
    assert_eq!(total(&before),total(&after));assert_eq!(bank.capitalization(),capitalization);
    emit(json!({"kind":"external_pool_loss","synthetic":true,"production_instruction":false,"live_slashing_proof":false,
        "lamports":lost,"epoch":bank.epoch(),"recipient":recipient.to_string(),"affected_stake_accounts":[reserve.to_string(),first.to_string(),second.to_string()],
        "capitalization_before":capitalization,"capitalization_after":bank.capitalization(),"all_data_unchanged":true,"all_piv_accounts_unchanged":true,
        "withdrawn_legs_unchanged":true,"retained_rent_per_source":STAKE_RENT}));
    let(before,after)=success(bank,"severe-pool-maintenance",vec![update_validators(),update_pool()],nonce);let mut expected=before.clone();
    let mut list=before[&key(201)].data().to_vec();for i in 0..2{put64(&mut list,9+i*73,STAKE_RENT);}bytes(&mut expected,key(201),list);
    let mut pool=before[&JITO_POOL].data().to_vec();put64(&mut pool,258,2*STAKE_RENT);bytes(&mut expected,JITO_POOL,pool);
    expect_fee(&mut expected,&before,&after);assert_eq!(after,expected);
    assert_eq!(pool_ratio(&after[&JITO_POOL]),(4_565_760,1_092_108_750_001,14));
    assert_eq!(value(units(&after[&target(14)]),4_565_760,1_092_108_750_001),464_510);
    // Reserve-only depletion is insufficient: the remaining validator balances
    // would still value these units above the 100,000,000 committed pending use.
    assert_eq!(value(111_108_750_001,3_395_929_399,1_092_108_750_001),345_494_412);
}
fn assert_early_pool_finalization(before:&Accounts,after:&Accounts){
    let native=before[&stake(1)].lamports()-STAKE_RENT;assert_eq!(native,3_608_636_361);
    let metadata=before[&leg(1)].lamports();assert_eq!(metadata,2_721_360);let prior=round(before);assert_eq!(prior.cumulative[13],0);
    let mut expected=before.clone();expected.remove(&stake(1));expected.remove(&leg(1));credit(&mut expected,target(12),native);credit(&mut expected,target(11),STAKE_RENT+metadata);
    let mut header=before[&target(1)].data().to_vec();header[11]=4;header[12]=2;
    for(j,delta)in[(5,native),(6,native+STAKE_RENT),(7,STAKE_RENT),(8,metadata),(13,1),(14,native)]{round_cumulative(&mut header,j,prior.cumulative[j]+delta);}
    bytes(&mut expected,target(1),header);expect_fee(&mut expected,before,after);assert_eq!(*after,expected);
    assert_eq!(after[&stake(0)],before[&stake(0)]);assert_eq!(after[&leg(0)],before[&leg(0)]);
}
fn recovery_rejection(bank:&Bank,name:&str,ix:Instruction,nonce:&mut u32){
    let before=snapshot(bank,&format!("{name}-before"));let t=tx(bank,vec![ix],*nonce);*nonce+=1;
    let c=execute(bank,name,&t).unwrap();
    assert_eq!(c.status,Err(TransactionError::InstructionError(2,InstructionError::Custom(6073))));
    assert!(c.inner_instructions.as_ref().unwrap().iter().all(|inner|inner.is_empty()));
    let after=snapshot(bank,&format!("{name}-after"));let mut expected=before;debit(&mut expected,key(242),c.fee_details.total_fee());assert_eq!(after,expected);
    assert!(matches!(execute(bank,&format!("{name}-replay"),&t),Err(TransactionError::AlreadyProcessed)));
    assert_eq!(snapshot(bank,&format!("{name}-replayed")),after);
}
fn assert_loss_finalization(before:&Accounts,after:&Accounts){
    let native=before[&stake(0)].lamports()-STAKE_RENT;assert_eq!(native,5*SOL-1);
    let metadata=before[&leg(0)].lamports();let prior=round(before);assert_eq!(prior.cumulative[13],1);
    let mut expected=before.clone();expected.remove(&stake(0));expected.remove(&leg(0));
    credit(&mut expected,target(12),native);credit(&mut expected,target(11),STAKE_RENT+metadata);
    let mut header=before[&target(1)].data().to_vec();header[11]=4;header[12]=1;
    for(j,delta)in[(5,native+1),(6,native+STAKE_RENT),(7,STAKE_RENT),(8,metadata),(10,1),(13,1),(14,native)]{
        round_cumulative(&mut header,j,prior.cumulative[j]+delta);
    }
    bytes(&mut expected,target(1),header);expect_fee(&mut expected,before,after);assert_eq!(*after,expected);
    assert_eq!(after[&target(0)],before[&target(0)],"HWM, pending and KIF remain byte-identical");
}
fn failed_outer()->Instruction{Instruction{program_id:PROGRAM,accounts:vec![],data:vec![0]}}
fn expect_fee(expected:&mut Accounts,before:&Accounts,after:&Accounts){let fee=before[&key(242)].lamports()-after[&key(242)].lamports();debit(expected,key(242),fee);}

fn inactive_failure(c:&solana_svm::transaction_commit_result::CommittedTransaction,t:&Transaction){
    assert_eq!(c.status,Err(TransactionError::InstructionError(2,InstructionError::InsufficientFunds)));
    assert!(c.log_messages.as_ref().unwrap().iter().any(|line|line.starts_with(&format!("Program {STAKE} failed:"))));
    assert!(c.inner_instructions.as_ref().unwrap()[2].iter().any(|inner|t.message.account_keys[inner.instruction.program_id_index as usize]==STAKE
        &&inner.instruction.data.starts_with(&4_u32.to_le_bytes())),"actual Stake full-withdrawal attempted");
}
fn assert_finalization(before:&Accounts,after:&Accounts,i:u64){
    let native=before[&stake(i)].lamports()-STAKE_RENT;let metadata=before[&leg(i)].lamports();let prior=round(before);
    let mut expected=before.clone();expected.remove(&stake(i));expected.remove(&leg(i));credit(&mut expected,target(12),native);credit(&mut expected,target(11),STAKE_RENT+metadata);
    let mut header=before[&target(1)].data().to_vec();for(j,delta)in[(5,native),(6,native+STAKE_RENT),(7,STAKE_RENT),(8,metadata),(13,1),(14,native)]{round_cumulative(&mut header,j,prior.cumulative[j]+delta);}
    if prior.cumulative[13]+1==2{header[11]=2;}bytes(&mut expected,target(1),header);expect_fee(&mut expected,before,after);assert_eq!(*after,expected);
}

// Pause rejection must be the actual PIV guard, before every nested CPI. A
// complete Bank map proves fee-only effects; duplicate processing charges no fee.
fn paused_rejection(bank:&Bank,name:&str,ix:Instruction,nonce:&mut u32){
    let before=snapshot(bank,&format!("{name}-before"));let t=tx(bank,vec![ix],*nonce);*nonce+=1;
    let c=execute(bank,name,&t).unwrap();
    assert_eq!(c.status,Err(TransactionError::InstructionError(2,InstructionError::Custom(6035))));
    assert!(c.inner_instructions.as_ref().unwrap().iter().all(|inner|inner.is_empty()));
    let after=snapshot(bank,&format!("{name}-after"));let mut expected=before;debit(&mut expected,key(242),c.fee_details.total_fee());assert_eq!(after,expected);
    assert!(matches!(execute(bank,&format!("{name}-replay"),&t),Err(TransactionError::AlreadyProcessed)));
    assert_eq!(snapshot(bank,&format!("{name}-replayed")),after);
}
fn set_pause(bank:&Bank,name:&str,ix:&Instruction,paused:bool,nonce:&mut u32){
    let(before,after)=success(bank,name,vec![ix.clone()],nonce);let mut expected=before.clone();
    let mut config=before[&target(0)].data().to_vec();assert_eq!(config[10],u8::from(!paused));config[10]=u8::from(paused);bytes(&mut expected,target(0),config);
    expect_fee(&mut expected,&before,&after);assert_eq!(after,expected);
}
fn pause_gate(bank:&Bank,name:&str,governance:&[Instruction],pair:usize,blocked:Instruction,nonce:&mut u32){
    set_pause(bank,&format!("{name}-pause"),&governance[pair*2],true,nonce);
    paused_rejection(bank,&format!("{name}-paused"),blocked,nonce);
    set_pause(bank,&format!("{name}-unpause"),&governance[pair*2+1],false,nonce);
}

#[derive(Clone,Copy,PartialEq,Eq)]
enum Profile{Positive,CooldownLoss,EarlyPoolLoss,SettlementPoolLoss}
#[test]
fn complete_delayed_production_lifecycle(){run_lifecycle(Profile::Positive);}
#[test]
fn cooldown_loss_recovery_production_lifecycle(){run_lifecycle(Profile::CooldownLoss);}
#[test]
fn early_pool_loss_recovery_production_lifecycle(){run_lifecycle(Profile::EarlyPoolLoss);}
#[test]
fn settlement_pool_loss_recovery_production_lifecycle(){run_lifecycle(Profile::SettlementPoolLoss);}

fn run_lifecycle(profile:Profile){
    let cooldown_loss=profile==Profile::CooldownLoss;
    assert_eq!(env::var("RAYON_NUM_THREADS").as_deref(),Ok("1"));let path=env::var("PIV1_BANK_ACCOUNTS_DIR").unwrap();let path=Path::new(&path);
    assert!(path.is_absolute());assert_eq!(path.parent().unwrap().canonicalize().unwrap(),path.parent().unwrap());assert!(!path.exists());fs::create_dir(path).unwrap();
    let(mut w,mut extras)=external_world();let governance=pause_governance(&mut w,&mut extras);let g=genesis(&w,extras);let leader=SlotLeader{id:key(246),vote_address:key(247)};
    // Bank genesis stores these canonical account fields unchanged; retain the
    // helper's complete account (including rent_epoch) as the independent oracle.
    let mut capability_accounts=Accounts::new();
    for feature in REQUIRED_FEATURES{
        let account=AccountSharedData::from(g.accounts[&feature].clone());
        assert_eq!(account.owner(),&solana_sdk_ids::feature::id());assert!(!account.executable());
        assert_eq!(account.lamports(),g.rent.minimum_balance(account.data().len()).max(1));
        capability_accounts.insert(feature,account);
    }
    for key in [STAKE,program_data(STAKE)]{capability_accounts.insert(key,AccountSharedData::from(g.accounts[&key].clone()));}
    let root=Bank::new_with_paths_for_tests(&g,Some(bounded()),vec![path.to_path_buf()],Some(leader));
    assert_runtime_capabilities(&root,"root",&capability_accounts);
    let(root,forks)=root.wrap_with_bank_forks_for_tests();let mut bank=Bank::new_from_parent_with_bank_forks(&forks,root,leader,1);let mut nonce=0;
    assert_runtime_capabilities(&bank,"initial-child",&capability_accounts);
    emit(json!({"kind":"fixture","genesis_hash":g.hash().to_string(),"synthetic_external_pool":true,"synthetic_governance":true,"real_token":"8.0.0","real_pool":"2.0.3","real_stake":"5.1.0","heap_bytes":HEAP,"unsigned":true,"economic_piv_seeded":false}));
    let initial=snapshot(&bank,"initial");for i in 0..16{assert!(!initial.contains_key(&target(i)));}
    let epoch_rewards=&initial[&sysvar::epoch_rewards::id()];
    assert_eq!(epoch_rewards,&account(sysvar::id(),g.rent.minimum_balance(81),vec![0;81]));
    emit(json!({"kind":"epoch_rewards_genesis","source":"pinned runtime genesis helper","bytes":81,"active":false,"helper_activates_features":false}));
    let(before,after)=success(&bank,"initialize",vec![w.outer.clone()],&mut nonce);
    let mut expected=before.clone();let states=w.state_bytes();for i in 0..16{let data=if i<9{states[i].clone()}else if i<14{vec![]}else{w.token_bytes()};expected.insert(target(i),account(World::target_owner(i),w.target_balance(i),data));}
    debit(&mut expected,w.roles[w.payer].key,w.shortfall());expect_fee(&mut expected,&before,&after);assert_eq!(after,expected);
    let funding=solana_system_interface::instruction::transfer(&key(220),&target(11),50_000_000);
    let(before,after)=success(&bank,"fund-operations",vec![funding],&mut nonce);let mut expected=before.clone();debit(&mut expected,key(220),50_000_000);credit(&mut expected,target(11),50_000_000);expect_fee(&mut expected,&before,&after);assert_eq!(after,expected);
    rollback(&bank,"pause-late-failure",vec![governance[0].clone(),failed_outer()],&mut nonce,Some(PROGRAM));
    set_pause(&bank,"intake-pause",&governance[0],true,&mut nonce);
    paused_rejection(&bank,"explicit-sol-paused",deposit_sol(1),&mut nonce);
    paused_rejection(&bank,"explicit-jito-paused",deposit_token(1),&mut nonce);
    let(before,after)=success(&bank,"direct-intake-paused",vec![solana_system_interface::instruction::transfer(&key(220),&target(9),1),direct_token(1)],&mut nonce);
    let mut expected=before.clone();debit(&mut expected,key(220),1);credit(&mut expected,target(9),1);token_delta(&mut expected,key(221),-1);token_delta(&mut expected,target(15),1);expect_fee(&mut expected,&before,&after);assert_eq!(after,expected);
    let(before,after)=success(&bank,"reconcile-direct-paused",vec![reconcile_pending()],&mut nonce);let mut expected=before.clone();let mut c=config(&before);c.n[4]+=1;c.n[5]+=1;write_config(&mut expected,c);expect_fee(&mut expected,&before,&after);assert_eq!(after,expected);
    let(before,after)=success(&bank,"heartbeat-paused",vec![heartbeat(&w)],&mut nonce);let mut expected=before.clone();let mut reward=before[&target(3)].data().to_vec();reward[51]=1;put64(&mut reward,52,0);bytes(&mut expected,target(3),reward);expect_fee(&mut expected,&before,&after);assert_eq!(after,expected);
    paused_rejection(&bank,"bootstrap-paused",bootstrap(),&mut nonce);
    set_pause(&bank,"intake-unpause",&governance[1],false,&mut nonce);
    rollback(&bank,"intake-late-failure",vec![deposit_sol(100*SOL),failed_outer()],&mut nonce,Some(system_program::id()));
    let(before,after)=success(&bank,"intake-sol",vec![deposit_sol(100*SOL-1)],&mut nonce);let mut expected=before.clone();debit(&mut expected,key(220),100*SOL-1);credit(&mut expected,target(9),100*SOL-1);let mut c=config(&before);c.n[5]+=100*SOL-1;write_config(&mut expected,c);expect_fee(&mut expected,&before,&after);assert_eq!(after,expected);
    let(before,after)=success(&bank,"intake-jito",vec![deposit_token(19*SOL-1)],&mut nonce);let mut expected=before.clone();token_delta(&mut expected,key(221),-((19*SOL-1) as i64));token_delta(&mut expected,target(15),(19*SOL-1) as i64);let mut c=config(&before);c.n[4]+=19*SOL-1;write_config(&mut expected,c);expect_fee(&mut expected,&before,&after);assert_eq!(after,expected);
    let(before,after)=success(&bank,"bootstrap",vec![bootstrap()],&mut nonce);let mut expected=before.clone();debit(&mut expected,target(9),100*SOL);credit(&mut expected,target(10),100*SOL);token_delta(&mut expected,target(15),-(19*SOL as i64));token_delta(&mut expected,target(14),19*SOL as i64);let mut c=config(&before);c.n[1]=119*SOL;c.n[2]=19*SOL;c.n[3]=100*SOL;c.n[4]=0;c.n[5]=0;c.n[9]=119*SOL;write_config(&mut expected,c);expect_fee(&mut expected,&before,&after);assert_eq!(after,expected);
    pause_gate(&bank,"principal-stake",&governance,1,principal_deposit(100*SOL),&mut nonce);
    rollback(&bank,"pool-deposit-late-failure",vec![principal_deposit(100*SOL),failed_outer()],&mut nonce,Some(JITO_PROGRAM));
    let(before,after)=success(&bank,"principal-stake",vec![principal_deposit(100*SOL)],&mut nonce);let mut expected=before.clone();debit(&mut expected,target(10),100*SOL);credit(&mut expected,key(202),100*SOL);token_delta(&mut expected,target(14),100*SOL as i64);
    let mut p=before[&JITO_POOL].data().to_vec();put64(&mut p,258,1100*SOL);put64(&mut p,266,1100*SOL);bytes(&mut expected,JITO_POOL,p);let mut mint=before[&JITO_MINT].data().to_vec();put64(&mut mint,36,1100*SOL);bytes(&mut expected,JITO_MINT,mint);let mut c=config(&before);c.n[2]=119*SOL;c.n[3]=0;write_config(&mut expected,c);expect_fee(&mut expected,&before,&after);assert_eq!(after,expected);
    // An external reserve donation followed by the actual SPL update changes
    // official pool value. This is a local yield stimulus, not inflation proof.
    let(before,after)=success(&bank,"external-yield-stimulus",vec![solana_system_interface::instruction::transfer(&key(220),&key(202),100*SOL),update_pool()],&mut nonce);
    let mut expected=before.clone();debit(&mut expected,key(220),100*SOL);credit(&mut expected,key(202),100*SOL);
    let mut pool=before[&JITO_POOL].data().to_vec();put64(&mut pool,258,1200*SOL);bytes(&mut expected,JITO_POOL,pool);expect_fee(&mut expected,&before,&after);assert_eq!(after,expected);
    let(before,after)=success(&bank,"guardian-heartbeat",vec![heartbeat(&w)],&mut nonce);let mut expected=before.clone();let mut reward=before[&target(3)].data().to_vec();reward[51]=1;put64(&mut reward,52,0);bytes(&mut expected,target(3),reward);expect_fee(&mut expected,&before,&after);assert_eq!(after,expected);
    let(before,after)=success(&bank,"pending-before-snapshot",vec![deposit_sol(SOL/10)],&mut nonce);let mut expected=before.clone();debit(&mut expected,key(220),SOL/10);credit(&mut expected,target(9),SOL/10);let mut c=config(&before);c.n[5]+=SOL/10;write_config(&mut expected,c);expect_fee(&mut expected,&before,&after);assert_eq!(after,expected);
    pause_gate(&bank,"preparation",&governance,2,prepare(),&mut nonce);
    let(before,after)=success(&bank,"prepare-withdrawal",vec![prepare()],&mut nonce);let r=round(&after);assert_eq!(r.state,1);assert_eq!(r.seq,0);let gross=value(119*SOL,1200*SOL,1100*SOL)-119*SOL;
    assert_eq!(r.snapshot[11],gross);let htfp=value(gross,5900,10000);let team=value(gross,1950,10000);let kif=value(gross,200,10000);let compound=value(gross,1950,10000);let splitdust=gross-htfp-team-kif-compound;let outgoing=htfp+team+kif;
    assert_eq!(&r.snapshot[13..19],&[htfp,compound,team,kif,splitdust,outgoing]);assert_eq!(r.snapshot[20],SOL/10);
    let budget=outgoing-SOL/10;let target_q=(((u128::from(budget)+1)*u128::from(1100*SOL)-1)/u128::from(1200*SOL)) as u64;assert_eq!(r.snapshot[22],target_q);assert!(value(target_q,1200*SOL,1100*SOL)>5*SOL+SOL);
    let mut expected=before.clone();debit(&mut expected,target(9),SOL/10);credit(&mut expected,target(12),SOL/10);let mut c=config(&before);c.prepared=Some(bank.clock().unix_timestamp as u64);c.n[0]=1;write_config(&mut expected,c);bytes(&mut expected,target(1),prepared_bytes(&w,bank.clock().unix_timestamp as u64,bank.slot(),bank.epoch(),gross,target_q));expect_fee(&mut expected,&before,&after);assert_eq!(after,expected);
    let(before,after)=success(&bank,"pending-after-snapshot",vec![deposit_sol(SOL/5),deposit_token(SOL/2)],&mut nonce);let mut expected=before.clone();debit(&mut expected,key(220),SOL/5);credit(&mut expected,target(9),SOL/5);token_delta(&mut expected,key(221),-(SOL as i64/2));token_delta(&mut expected,target(15),SOL as i64/2);let mut c=config(&before);c.n[5]+=SOL/5;c.n[4]+=SOL/2;write_config(&mut expected,c);expect_fee(&mut expected,&before,&after);assert_eq!(after,expected);
    pause_gate(&bank,"leg-initiation",&governance,3,initiate(0),&mut nonce);
    for i in 0..2{
        rollback(&bank,&format!("leg-{i}-late-failure"),vec![initiate(i),failed_outer()],&mut nonce,Some(JITO_PROGRAM));
        let(before,after)=success(&bank,&format!("leg-{i}"),vec![initiate(i)],&mut nonce);let prior=round(&before);let r=round(&after);let(t,s,_)=pool_ratio(&before[&JITO_POOL]);
        let maximum=((((5*SOL) as u128+1)*u128::from(s)-1)/u128::from(t)) as u64;let q=(target_q-prior.cumulative[0]).min(maximum);let native=value(q,t,s);assert!(native>=SOL);
        assert_eq!(r.cumulative[0],prior.cumulative[0]+q);assert_eq!(r.cumulative[2],prior.cumulative[2]+q);assert_eq!(r.cumulative[4],prior.cumulative[4]+native);assert_eq!(r.cumulative[12],i+1);
        assert_eq!(units(&before[&target(14)])-units(&after[&target(14)]),q);assert_eq!(pool_ratio(&after[&JITO_POOL]),(t-native,s-q,bank.epoch()));
        assert_eq!(before[&source(i as u8)].lamports()-after[&source(i as u8)].lamports(),native);assert_eq!(after[&stake(i)].lamports(),native+STAKE_RENT);
        assert_eq!(u64at(after[&stake(i)].data(),172),bank.epoch());assert_eq!(before[&target(0)],after[&target(0)]);
        let mut expected=before.clone();debit(&mut expected,source(i as u8),native);
        let mut source_data=before[&source(i as u8)].data().to_vec();put64(&mut source_data,156,6*SOL-native);bytes(&mut expected,source(i as u8),source_data);
        let mut destination=stake_state(Some(key(210+i as u8)),native);destination[12..44].copy_from_slice(authority().as_ref());destination[44..76].copy_from_slice(authority().as_ref());put64(&mut destination,172,bank.epoch());
        expected.insert(stake(i),account(STAKE,native+STAKE_RENT,destination));let metadata_rent=g.rent.minimum_balance(LEG_SIZE);
        expected.insert(leg(i),account(PROGRAM,metadata_rent,leg_bytes(i,bank.epoch(),t,s,q,native,prior.snapshot[23])));debit(&mut expected,target(11),STAKE_RENT+metadata_rent);
        token_delta(&mut expected,target(14),-(q as i64));let mut mint=before[&JITO_MINT].data().to_vec();let next_supply=u64at(&mint,36)-q;put64(&mut mint,36,next_supply);bytes(&mut expected,JITO_MINT,mint);
        let mut pool=before[&JITO_POOL].data().to_vec();put64(&mut pool,258,t-native);put64(&mut pool,266,s-q);bytes(&mut expected,JITO_POOL,pool);
        let mut list=before[&key(201)].data().to_vec();put64(&mut list,9+i as usize*73,6*SOL+STAKE_RENT-native);bytes(&mut expected,key(201),list);
        let mut header=before[&target(1)].data().to_vec();for(j,delta)in[(0,q),(2,q),(3,native),(4,native),(11,1),(12,1)]{round_cumulative(&mut header,j,prior.cumulative[j]+delta);}bytes(&mut expected,target(1),header);
        expect_fee(&mut expected,&before,&after);assert_eq!(after,expected);
    }
    let assigned=snapshot(&bank,"all-legs-assigned");assert_eq!(round(&assigned).cumulative[0],target_q);assert_eq!(round(&assigned).cumulative[12],2);
    if cooldown_loss{external_cooldown_loss(&bank);}
    rollback(&bank,"active-finalization",vec![finalize(0)],&mut nonce,None);
    // Advance real Bank epochs/history. No PIV/external economic account writes.
    let operations_before=assigned[&target(11)].lamports();let mut finalized=0;
    // Bind the finite bound to this fixture's actual assigned delegation totals.
    assert_eq!([u64at(assigned[&stake(0)].data(),156),u64at(assigned[&stake(1)].data(),156)],[5_000_000_000,3_608_636_361]);
    assert_eq!(u64at(assigned[&source(0)].data(),156)+u64at(assigned[&source(1)].data(),156),3_391_363_639);
    for i in 0..2{assert_eq!(u64at(assigned[&stake(i)].data(),164),u64::MAX);assert_eq!(u64at(assigned[&stake(i)].data(),172),0);}
    for epoch in 1..=FINALIZATION_EPOCH_BOUND{
        let slot=bank.epoch_schedule().get_first_slot_in_epoch(epoch);
        // Construct before taking the fork write lock: Bank construction itself
        // may consult the program cache and its fork graph.
        let warped=Bank::warp_from_parent(bank,leader,slot);
        let warped=forks.write().unwrap().insert(warped).clone_without_scheduler();
        assert_runtime_capabilities(&warped,"epoch-warp",&capability_accounts);
        assert_registered_cache(&warped,"epoch-warp");
        bank=Bank::new_from_parent_with_bank_forks(&forks,warped,leader,slot+1);assert_eq!(bank.epoch(),epoch);
        assert_runtime_capabilities(&bank,"epoch-child",&capability_accounts);
        assert_registered_cache(&bank,"epoch-child");
        let(before,after)=success(&bank,&format!("epoch-{epoch}-pool-update"),vec![update_validators(),update_pool()],&mut nonce);
        let mut expected=before.clone();let mut pool=before[&JITO_POOL].data().to_vec();let(t,s,_)=pool_ratio(&before[&JITO_POOL]);put64(&mut pool,274,epoch);put64(&mut pool,419,s);put64(&mut pool,427,t);bytes(&mut expected,JITO_POOL,pool);
        let mut list=before[&key(201)].data().to_vec();for i in 0..2{put64(&mut list,25+i*73,epoch);}bytes(&mut expected,key(201),list);expect_fee(&mut expected,&before,&after);assert_eq!(after,expected);
        if profile==Profile::EarlyPoolLoss&&epoch==14{
            external_pool_loss(&bank,&mut nonce);
            rollback(&bank,"early-recovery-late-failure",vec![finalize(1),failed_outer()],&mut nonce,Some(STAKE));
            let(before,after)=success(&bank,"early-recovery-finalization",vec![finalize(1)],&mut nonce);assert_early_pool_finalization(&before,&after);
            let r=round(&after);let c=config(&after);assert_eq!(r.state,4);assert_eq!(r.cumulative[13],1);
            assert_eq!(r.cumulative[9],0);assert_eq!(r.cumulative[10],0);assert_eq!(after[&target(1)].data()[12],2);
            assert_eq!(after[&target(0)],assigned[&target(0)]);assert_eq!(c.n[1],119*SOL);assert_eq!(c.n[5],3*SOL/10);assert_eq!(c.n[4],SOL/2);
            assert_eq!(after[&stake(0)],assigned[&stake(0)]);assert_eq!(after[&leg(0)],assigned[&leg(0)]);
            assert_eq!(after[&target(11)].lamports(),operations_before+STAKE_RENT+2_721_360);
            for key in w.recipients.into_iter().chain([target(13)]).chain((3..9).map(target)){assert_eq!(after[&key],assigned[&key]);}
            let residual=value(c.n[2]-target_q,4_565_760,1_092_108_750_001)+c.n[3];assert_eq!(residual,464_510);assert!(residual<r.snapshot[26]);assert!(residual<r.snapshot[20]);
            for(name,ix)in[("early-recovery-settlement",settle(&w)),("early-recovery-integration",integrate()),("early-recovery-companion",finalize(0))]{recovery_rejection(&bank,name,ix,&mut nonce);}
            emit(json!({"kind":"early_pool_loss_complete","status":"PASS","transactions":nonce,"legs":2,"finalized_legs":1,"epoch":bank.epoch(),
                "pool_total":4_565_760,"pool_supply":1_092_108_750_001_u64,"residual_value":residual,"pending_used":100_000_000,
                "cooldown_loss_lamports":0,"recovery_state":4,"recovery_flags":2,"protected_hwm":c.n[1],"pending_sol":c.n[5],"pending_tokens":c.n[4],
                "recovered_stake_rent":r.cumulative[7],"recovered_metadata_rent":r.cumulative[8],"temporary_accounts_closed":2,"outstanding_leg":0,
                "outstanding_stake_lamports":after[&stake(0)].lamports(),"outstanding_metadata_lamports":after[&leg(0)].lamports(),"companion_unchanged":true,
                "beneficiary_paid":0,"kif_claimed":0,"synthetic_external_loss":true,"synthetic_loss_conservation":true,"zero_floor_executed":false,
                "live_ready":false,"signature_verified":false,"actual_programs":["PIV1","System","Token8.0.0","SPLpool2.0.3","Stake5.1.0"]}));return;
        }
        if finalized==0{
            let before=snapshot(&bank,&format!("epoch-{epoch}-finalization-before"));let t=tx(&bank,vec![finalize(1)],nonce);nonce+=1;let c=execute(&bank,&format!("epoch-{epoch}-finalize-second"),&t).unwrap();let after=snapshot(&bank,&format!("epoch-{epoch}-finalization-after"));
            if c.status.is_err(){inactive_failure(&c,&t);let mut expected=before;debit(&mut expected,key(242),c.fee_details.total_fee());assert_eq!(after,expected);continue;}
            assert_finalization(&before,&after,1);assert_eq!(round(&after).cumulative[13],1);
        }
        pause_gate(&bank,"mature-finalization",&governance,4,finalize(0),&mut nonce);
        if cooldown_loss{rollback(&bank,"loss-finalization-late-failure",vec![finalize(0),failed_outer()],&mut nonce,Some(STAKE));}
        let(before,after)=success(&bank,"finalize-first",vec![finalize(0)],&mut nonce);assert_eq!(round(&after).cumulative[13],2);
        if cooldown_loss{assert_loss_finalization(&before,&after);}else{assert_eq!(round(&after).state,2);assert_finalization(&before,&after,0);}
        finalized=2;break;
    }
    assert_eq!(finalized,2,"bounded real deactivation progression must complete");
    if cooldown_loss{
        let recovered=snapshot(&bank,"cooldown-loss-recovery");let r=round(&recovered);let c=config(&recovered);
        assert_eq!(bank.epoch(),14);assert_eq!(r.state,4);assert_eq!(recovered[&target(1)].data()[12],1);
        assert_eq!(r.cumulative[9],0);assert_eq!(r.cumulative[10],1);assert_eq!(r.cumulative[13],2);
        let metadata_rents=assigned[&leg(0)].lamports()+assigned[&leg(1)].lamports();
        assert_eq!(r.cumulative[7],2*STAKE_RENT);assert_eq!(r.cumulative[8],metadata_rents);
        assert_eq!(recovered[&target(11)].lamports(),operations_before+2*STAKE_RENT+metadata_rents);
        for key in [stake(0),stake(1),leg(0),leg(1)]{assert!(!recovered.contains_key(&key));}
        assert_eq!(recovered[&target(0)],assigned[&target(0)]);assert_eq!(c.n[1],119*SOL);
        assert_eq!(c.n[5],3*SOL/10);assert_eq!(c.n[4],SOL/2);
        for key in w.recipients.into_iter().chain([target(13)]).chain((3..9).map(target)){
            assert_eq!(recovered[&key],assigned[&key],"no beneficiary or KIF payout/credit");
        }
        for(name,ix)in[("recovery-settlement",settle(&w)),("recovery-integration",integrate()),("recovery-finalization",finalize(0))]{
            recovery_rejection(&bank,name,ix,&mut nonce);
        }
        emit(json!({"kind":"cooldown_loss_complete","status":"PASS","transactions":nonce,"legs":2,"finalized_legs":2,"epoch":bank.epoch(),
            "loss_lamports":1,"recovery_state":r.state,"recovery_flags":1,"protected_hwm":c.n[1],"pending_sol":c.n[5],"pending_tokens":c.n[4],
            "recovered_stake_rent":r.cumulative[7],"recovered_metadata_rent":r.cumulative[8],"temporary_accounts_closed":4,
            "beneficiary_paid":0,"kif_claimed":0,"synthetic_external_loss":true,"synthetic_loss_conservation":true,
            "live_ready":false,"signature_verified":false,"actual_programs":["PIV1","System","Token8.0.0","SPLpool2.0.3","Stake5.1.0"]}));
        return;
    }
    if profile==Profile::SettlementPoolLoss{
        external_pool_loss(&bank,&mut nonce);let projected=snapshot(&bank,"before-severe-settlement");let r=round(&projected);let c=config(&projected);
        assert_eq!(r.state,2);assert_eq!(r.cumulative[13],2);assert_eq!(r.cumulative[9],0);assert_eq!(r.cumulative[10],0);
        let eligible=r.snapshot[20]+r.cumulative[5];let net=eligible.min(outgoing);
        let paid=value(net,5900,8050).min(htfp)+value(net,1950,8050).min(team)+value(net,200,8050).min(kif);
        let floor=g.rent.minimum_balance(0);let remaining_escrow=projected[&target(12)].lamports()-floor-paid;assert_eq!(remaining_escrow,2);
        let retained=projected[&target(10)].lamports()-floor+value(units(&projected[&target(14)]),4_565_760,1_092_108_750_001)+remaining_escrow;
        assert_eq!(retained,464_512);assert_eq!(r.snapshot[20],100_000_000);assert!(retained<r.snapshot[20]);
        rollback(&bank,"severe-settlement-late-failure",vec![settle(&w),failed_outer()],&mut nonce,Some(PROGRAM));
        let(before,after)=success(&bank,"severe-settlement",vec![settle(&w)],&mut nonce);let mut expected=before.clone();
        let mut header=before[&target(1)].data().to_vec();header[11]=4;header[12]=2;bytes(&mut expected,target(1),header);expect_fee(&mut expected,&before,&after);assert_eq!(after,expected);
        assert_eq!(after[&target(0)],assigned[&target(0)]);assert_eq!(round(&after).state,4);
        let metadata_rents=assigned[&leg(0)].lamports()+assigned[&leg(1)].lamports();assert_eq!(r.cumulative[7],2*STAKE_RENT);assert_eq!(r.cumulative[8],metadata_rents);
        assert_eq!(after[&target(11)].lamports(),operations_before+2*STAKE_RENT+metadata_rents);
        for key in [stake(0),stake(1),leg(0),leg(1)]{assert!(!after.contains_key(&key));}
        for key in w.recipients.into_iter().chain([target(13)]).chain((3..9).map(target)){assert_eq!(after[&key],assigned[&key]);}
        for(name,ix)in[("severe-recovery-settlement",settle(&w)),("severe-recovery-integration",integrate()),("severe-recovery-finalization",finalize(0))]{recovery_rejection(&bank,name,ix,&mut nonce);}
        emit(json!({"kind":"settlement_pool_loss_complete","status":"PASS","transactions":nonce,"legs":2,"finalized_legs":2,"epoch":bank.epoch(),
            "pool_total":4_565_760,"pool_supply":1_092_108_750_001_u64,"token_value":464_510,"projected_retained":retained,"pending_used":r.snapshot[20],
            "observed_protected_value":0,"recovery_event_verified":true,"zero_floor_executed":true,"cooldown_loss_lamports":0,"recovery_state":4,"recovery_flags":2,
            "protected_hwm":c.n[1],"pending_sol":c.n[5],"pending_tokens":c.n[4],"recovered_stake_rent":r.cumulative[7],"recovered_metadata_rent":r.cumulative[8],
            "temporary_accounts_closed":4,"beneficiary_paid":0,"kif_claimed":0,"synthetic_external_loss":true,"synthetic_loss_conservation":true,
            "live_ready":false,"signature_verified":false,"actual_programs":["PIV1","System","Token8.0.0","SPLpool2.0.3","Stake5.1.0"]}));return;
    }
    // Preserved G correctly required recovery: projected protected value was one
    // lamport below the settled HWM. This fresh success fixture adds an explicit
    // external economic stimulus, never resetting or bypassing that recovery.
    // At this exact ratio, 3 reserve lamports are the minimum needed to increase
    // the held principal token floor by 1; frozen distribution amounts stay fixed.
    let(before,after)=success(&bank,"external-reserve-appreciation",vec![solana_system_interface::instruction::transfer(&key(220),&key(202),3),update_pool()],&mut nonce);
    let mut expected=before.clone();debit(&mut expected,key(220),3);credit(&mut expected,key(202),3);
    let(t,s,_)=pool_ratio(&before[&JITO_POOL]);let q=units(&before[&target(14)]);let prior_value=value(q,t,s);
    assert_eq!((q,t,s),(111_108_750_001,1_191_391_363_639,1_092_108_750_001));
    assert_eq!(value(q,t+2,s),prior_value);assert_eq!(value(q,t+3,s),prior_value+1);
    let mut pool=before[&JITO_POOL].data().to_vec();put64(&mut pool,258,t+3);bytes(&mut expected,JITO_POOL,pool);
    expect_fee(&mut expected,&before,&after);assert_eq!(after,expected);
    assert_eq!(after[&target(1)],before[&target(1)],"frozen round remains unchanged");
    assert_eq!(value(units(&after[&target(14)]),pool_ratio(&after[&JITO_POOL]).0,s),121_209_545_456);
    let before_settle=snapshot(&bank,"before-settlement");let r=round(&before_settle);assert_eq!(r.cumulative[9],0);assert_eq!(r.cumulative[10],0);
    let metadata_rents=assigned[&leg(0)].lamports()+assigned[&leg(1)].lamports();
    assert_eq!(r.cumulative[7],2*STAKE_RENT);assert_eq!(r.cumulative[8],metadata_rents);
    assert_eq!(before_settle[&target(11)].lamports(),operations_before+2*STAKE_RENT+metadata_rents);
    let eligible=r.snapshot[20]+r.cumulative[5];let net=eligible.min(outgoing);let paid_h=value(net,5900,8050).min(htfp);let paid_t=value(net,1950,8050).min(team);let paid_k=value(net,200,8050).min(kif);let paid=paid_h+paid_t+paid_k;let dust=net-paid;let conservative=eligible-net;
    pause_gate(&bank,"settlement",&governance,5,settle(&w),&mut nonce);
    rollback(&bank,"settlement-late-failure",vec![settle(&w),failed_outer()],&mut nonce,Some(PROGRAM));
    let(before,after)=success(&bank,"settlement",vec![settle(&w)],&mut nonce);let mut expected=before.clone();debit(&mut expected,target(12),paid);credit(&mut expected,w.recipients[0],paid_h);credit(&mut expected,w.recipients[1],paid_t);credit(&mut expected,target(13),paid_k);
    let mut c=config(&before);let hwm_delta=compound+splitdust+r.snapshot[21]+dust+conservative;c.n[1]+=hwm_delta;c.n[10]+=gross;c.n[11]+=paid_h;c.n[12]+=paid_t;c.n[13]+=paid_k;c.n[7]+=paid_k;c.n[15]+=compound;c.n[16]+=splitdust+r.snapshot[21]+dust+conservative;
    write_config(&mut expected,c.clone());let mut reward=before[&target(3)].data().to_vec();put64(&mut reward,60,paid_k);put64(&mut reward,68,paid_k);bytes(&mut expected,target(3),reward);
    let mut header=before[&target(1)].data().to_vec();header[11]=3;header[13]=1;round_cumulative(&mut header,15,0);
    for(j,n)in [r.settlement[0],r.settlement[1],net,paid_h,paid_t,paid_k,dust,paid,r.cumulative[14]-paid,conservative,paid_k,0,0,hwm_delta,c.n[1]].into_iter().enumerate(){round_settlement(&mut header,j,n);}bytes(&mut expected,target(1),header);expect_fee(&mut expected,&before,&after);assert_eq!(after,expected);
    let settled=round(&after);assert_eq!(settled.state,3);assert_eq!(&settled.settlement[2..8],&[net,paid_h,paid_t,paid_k,dust,paid]);assert_eq!(settled.settlement[14],c.n[1]);
    pause_gate(&bank,"pending-integration",&governance,6,integrate(),&mut nonce);
    rollback(&bank,"integration-late-failure",vec![integrate(),failed_outer()],&mut nonce,Some(TOKEN));
    let(before,after)=success(&bank,"pending-integration",vec![integrate()],&mut nonce);let mut expected=before.clone();let floor=g.rent.minimum_balance(0);let physical=before[&target(9)].lamports()-floor;let escrow=before[&target(12)].lamports()-floor;
    assert_eq!(physical,SOL/5);debit(&mut expected,target(9),physical);debit(&mut expected,target(12),escrow);credit(&mut expected,target(10),physical+escrow);token_delta(&mut expected,target(15),-(SOL as i64/2));token_delta(&mut expected,target(14),SOL as i64/2);
    let mut c=config(&before);let(t,s,_)=pool_ratio(&before[&JITO_POOL]);let contribution=c.n[5]+value(c.n[4],t,s);c.n[1]+=contribution;c.n[2]=units(&expected[&target(14)]);c.n[3]=expected[&target(10)].lamports()-floor-c.n[6];c.n[4]=0;c.n[5]=0;c.n[9]+=contribution;assert!(c.n[3]+value(c.n[2],t,s)>=c.n[1]);write_config(&mut expected,c.clone());
    let mut idle=w.state_bytes()[1].clone();idle[22]=1;let summary=[0,bank.clock().unix_timestamp as u64,gross,paid,contribution,c.n[1],target_q,2,0,paid_k,0];for(i,n)in summary.into_iter().enumerate(){put64(&mut idle,23+i*8,n);}bytes(&mut expected,target(1),idle);expect_fee(&mut expected,&before,&after);assert_eq!(after,expected);
    rollback(&bank,"integration-replay",vec![integrate()],&mut nonce,None);
    set_pause(&bank,"earned-kif-pause",&governance[14],true,&mut nonce);
    let(before,after)=success(&bank,"earned-kif-claim",vec![claim(paid_k)],&mut nonce);let mut expected=before.clone();debit(&mut expected,target(13),paid_k);credit(&mut expected,key(96),paid_k);let mut c=config(&before);c.n[7]-=paid_k;c.n[14]+=paid_k;write_config(&mut expected,c);let mut reward=before[&target(3)].data().to_vec();put64(&mut reward,60,0);put64(&mut reward,76,paid_k);bytes(&mut expected,target(3),reward);expect_fee(&mut expected,&before,&after);assert_eq!(after,expected);
    set_pause(&bank,"earned-kif-unpause",&governance[15],false,&mut nonce);
    emit(json!({"kind":"pause_profile_complete","status":"PASS","distinct_governance_actions":16,"synthetic_proposal_consumption":false,"pause_error":6035,"economic_blocks":9,"earned_kif_allowed":true,"direct_intake_reconciled":true}));
    emit(json!({"kind":"lifecycle_complete","status":"PASS","transactions":nonce,"legs":2,"epoch":bank.epoch(),"final_hwm":config(&after).n[1],"pending_sol":config(&after).n[5],"pending_tokens":config(&after).n[4],"kif_claimed":paid_k,"live_ready":false,"signature_verified":false,"actual_programs":["PIV1","System","Token8.0.0","SPLpool2.0.3","Stake5.1.0"]}));
}

// Compact repeated-cycle profile. The earlier delayed/pause/loss profiles above
// remain separate and unchanged. Only real production instructions change PIV
// economic state; exact synthetic Clock inputs are explicitly declared below.
// This compact fixture never leaves Bank slot one. Its original Closed(1)
// metadata entry is on the current registered Bank (Equal), not an ancestor.
// Observe the exact same identity after the first heartbeat and after all cycles;
// no cache entry, fork, account or runtime environment is changed here.
fn assert_cycle_cache(bank:&Bank,phase:&str){
    assert_eq!(bank.slot(),1);assert_eq!(bank.epoch(),0);
    let processor=bank.get_transaction_processor();let cache=processor.global_program_cache.read().unwrap();
    let graph=cache.fork_graph.as_ref().unwrap().upgrade().unwrap();let graph=graph.read().unwrap();
    let current=graph.get(1).expect("compact current Bank is registered");assert!(std::ptr::eq(current.as_ref(),bank));
    assert_eq!(graph.root(),0);assert_eq!(graph.highest_slot(),1);assert_eq!(cache.latest_root_slot,0);
    let ancestors=bank.proper_ancestors_set();assert_eq!(ancestors.len(),1);assert!(ancestors.contains(&0));
    assert_eq!(graph.ancestors().get(&1),Some(&ancestors));assert_eq!(graph.get(0).unwrap().slot(),0);
    let versions=cache.get_slot_versions_for_tests(&program_data(PROGRAM));assert_eq!(versions.len(),1);
    let entry=&versions[0];assert_eq!(format!("{:?}",entry.program),"ProgramCacheEntryType::Closed");
    assert_eq!(entry.deployment_slot,1);assert_eq!(entry.effective_slot,1);assert!(entry.program.get_environment().is_none());
    assert_eq!(entry.deployment_slot,bank.slot());assert!(!ancestors.contains(&entry.deployment_slot));
    emit(json!({"kind":"cycle_cache_registration","phase":phase,"slot":1,"epoch":0,"fork_root":0,"fork_highest":1,"cache_root_slot":0,
        "current_bank_registered":true,"registered_ancestors":[0],"programdata_key":program_data(PROGRAM).to_string(),
        "versions":1,"type":format!("{:?}",entry.program),"deployment_slot":1,"effective_slot":1,"environment_is_none":true,
        "derived_fork_relationship":"Equal","current_slot_visible":true,"ancestor_visible":false,"read_only":true}));
}
fn cycle_prepare()->Instruction{
    let mut a=base(&[0,1,2,3,5]);a.push(meta(target(2),false,false));
    for i in 3..9{a.push(meta(target(i),false,false));}a.push(meta(sysvar::clock::id(),false,false));
    piv(b"PIV1PD01",&[],a)
}
fn cycle_heartbeat(w:&World,index:u8)->Instruction{
    let mut ix=heartbeat(w);ix.data[9]=index;
    for i in 0..6{ix.accounts[2+i].is_writable=i==usize::from(index);}
    ix.accounts[9].pubkey=key(96-index);ix
}
fn cycle_clock(bank:&Bank,name:&str,timestamp:i64){
    let before=snapshot(bank,&format!("{name}-before"));let prior=bank.clock();assert!(timestamp>=prior.unix_timestamp);
    let capitalization=bank.capitalization();let mut next=prior.clone();next.unix_timestamp=timestamp;
    // Canonical pinned Bank test helper updates the real Clock account and its
    // syscall cache. This is synthetic time input, not elapsed wall-clock time.
    bank.set_sysvar_for_tests(&next);assert_eq!(bank.clock(),next);
    let after=snapshot(bank,&format!("{name}-after"));let mut expected=before.clone();
    let mut encoded=before[&sysvar::clock::id()].data().to_vec();assert_eq!(encoded.len(),40);
    encoded[32..40].copy_from_slice(&timestamp.to_le_bytes());bytes(&mut expected,sysvar::clock::id(),encoded);
    assert_eq!(after,expected);assert_eq!(bank.capitalization(),capitalization);
    emit(json!({"kind":"cycle_clock","case":name,"synthetic":true,"real_walltime":false,"prior_timestamp":prior.unix_timestamp,
        "timestamp":timestamp,"slot":bank.slot(),"epoch":bank.epoch(),"only_clock_timestamp_changed":true,"economic_accounts_unchanged":true}));
}
fn cycle_reject(bank:&Bank,name:&str,ix:Instruction,code:u32,nonce:&mut u32){
    let before=snapshot(bank,&format!("{name}-before"));let t=tx(bank,vec![ix],*nonce);*nonce+=1;
    let c=execute(bank,name,&t).unwrap();assert_eq!(c.status,Err(TransactionError::InstructionError(2,InstructionError::Custom(code))));
    assert!(c.inner_instructions.as_ref().unwrap().iter().all(|inner|inner.is_empty()));
    let after=snapshot(bank,&format!("{name}-after"));let mut expected=before;debit(&mut expected,key(242),c.fee_details.total_fee());assert_eq!(after,expected);
    assert!(matches!(execute(bank,&format!("{name}-replay"),&t),Err(TransactionError::AlreadyProcessed)));
    assert_eq!(snapshot(bank,&format!("{name}-replayed")),after);
    emit(json!({"kind":"cycle_rejection","case":name,"error":code,"no_cpi":true,"fee_only":true,"identical_replay_no_fee":true}));
}
fn cycle_activity(bank:&Bank,w:&World,index:u8,period:u64,name:&str,nonce:&mut u32){
    let(before,after)=success(bank,name,vec![cycle_heartbeat(w,index)],nonce);let mut expected=before.clone();
    let reward=target(3+usize::from(index));let mut b=before[&reward].data().to_vec();b[51]=1;put64(&mut b,52,period);bytes(&mut expected,reward,b);
    expect_fee(&mut expected,&before,&after);assert_eq!(after,expected);
}
fn cycle_header(w:&World,before:&Accounts,time:u64,slot:u64,epoch:u64,period:u64,bitmap:u8,count:u8)->Vec<u8>{
    let c=config(before);let(t,s,_)=pool_ratio(&before[&JITO_POOL]);let prior=before[&target(1)].data();
    assert_eq!(prior[11],0);assert_eq!(c.n[4],0);assert_eq!(c.n[5],2*SOL);assert_eq!(c.n[6],0);
    let historical=c.n[3]+value(c.n[2],t,s);assert_eq!(historical-c.n[1],1_900_000_000);
    let proposed=370_500_000;let outgoing=1_529_500_000;
    let mut b=w.state_bytes()[1][..14].to_vec();b[11]=2;append(&mut b,c.n[0]);
    b.push(prior[22]);if prior[22]==1{b.extend(&prior[23..111]);}else{assert_eq!(prior[22],0);}
    for n in [time,slot,epoch,c.n[1],c.n[2],c.n[3],historical,t,s,0,1,
        1_900_000_000,0,1_121_000_000,370_500_000,370_500_000,38_000_000,0,outgoing,2*SOL,outgoing,0,0,0,0,0,c.n[1]+proposed]{append(&mut b,n);}
    b.extend(1_u16.to_le_bytes());for i in 0..16{append(&mut b,if i==14||i==15{outgoing}else{0});}
    for k in [w.recipients[0],w.recipients[1],target(2)]{b.extend(k.as_ref());}append(&mut b,0);
    for i in 0..6{b.extend(key(96-i).as_ref());}b.extend([bitmap,count]);append(&mut b,period);append(&mut b,c.n[8]);
    append(&mut b,proposed);append(&mut b,c.n[1]+proposed);b.extend([0;13*8]);
    assert_eq!(b.len(),803+usize::from(prior[22])*88);b.resize(891,0);b
}
fn cycle_settlement_bytes(before:&Accounts,zero:u64,carry:u64,liability:u64)->Vec<u8>{
    let r=round(before);let mut b=before[&target(1)].data().to_vec();let shift=usize::from(b[22])*88;
    b[11]=3;b[13]=1;put64(&mut b,241+shift+15*8,0);
    let delta=370_500_000+zero;
    for(i,n)in [370_500_000,r.snapshot[3]+370_500_000,1_529_500_000,1_121_000_000,370_500_000,38_000_000,0,1_529_500_000,0,0,liability,carry,zero,delta,r.snapshot[3]+delta].into_iter().enumerate(){put64(&mut b,683+shift+i*8,n);}
    b
}
#[test]
fn repeated_cycles_production_lifecycle(){
    assert_eq!(env::var("RAYON_NUM_THREADS").as_deref(),Ok("1"));let path=env::var("PIV1_BANK_ACCOUNTS_DIR").unwrap();let path=Path::new(&path);
    assert!(path.is_absolute());assert_eq!(path.parent().unwrap().canonicalize().unwrap(),path.parent().unwrap());assert!(!path.exists());fs::create_dir(path).unwrap();
    let(w,extras)=external_world();let g=genesis(&w,extras);let leader=SlotLeader{id:key(246),vote_address:key(247)};
    let root=Bank::new_with_paths_for_tests(&g,Some(bounded()),vec![path.to_path_buf()],Some(leader));
    let(root,forks)=root.wrap_with_bank_forks_for_tests();let bank=Bank::new_from_parent_with_bank_forks(&forks,root,leader,1);let mut nonce=0;
    let mut capability_accounts=Accounts::new();for feature in REQUIRED_FEATURES{capability_accounts.insert(feature,AccountSharedData::from(g.accounts[&feature].clone()));}
    for k in [STAKE,program_data(STAKE)]{capability_accounts.insert(k,AccountSharedData::from(g.accounts[&k].clone()));}
    assert_runtime_capabilities(&bank,"cycles-initial",&capability_accounts);
    emit(json!({"kind":"fixture","profile":"repeated-cycles","genesis_hash":g.hash().to_string(),"synthetic_external_pool":true,"synthetic_governance":true,
        "synthetic_clock":true,"heap_bytes":HEAP,"unsigned":true,"economic_piv_seeded":false}));
    let initial=snapshot(&bank,"cycles-initial");for i in 0..16{assert!(!initial.contains_key(&target(i)));}
    let(before,after)=success(&bank,"cycles-initialize",vec![w.outer.clone()],&mut nonce);let mut expected=before.clone();let states=w.state_bytes();
    for i in 0..16{let d=if i<9{states[i].clone()}else if i<14{vec![]}else{w.token_bytes()};expected.insert(target(i),account(World::target_owner(i),w.target_balance(i),d));}
    debit(&mut expected,w.roles[w.payer].key,w.shortfall());expect_fee(&mut expected,&before,&after);assert_eq!(after,expected);
    assert_eq!(u64at(&config(&after).tail,0),0,"fixed KIF anchor");
    let(before,after)=success(&bank,"cycles-intake",vec![deposit_sol(SOL),deposit_token(19*SOL)],&mut nonce);let mut expected=before.clone();
    debit(&mut expected,key(220),SOL);credit(&mut expected,target(9),SOL);token_delta(&mut expected,key(221),-(19*SOL as i64));token_delta(&mut expected,target(15),19*SOL as i64);
    let mut c=config(&before);c.n[4]=19*SOL;c.n[5]=SOL;write_config(&mut expected,c);expect_fee(&mut expected,&before,&after);assert_eq!(after,expected);
    let(before,after)=success(&bank,"cycles-bootstrap",vec![bootstrap()],&mut nonce);let mut expected=before.clone();
    debit(&mut expected,target(9),SOL);credit(&mut expected,target(10),SOL);token_delta(&mut expected,target(15),-(19*SOL as i64));token_delta(&mut expected,target(14),19*SOL as i64);
    let mut c=config(&before);c.n[1]=20*SOL;c.n[2]=19*SOL;c.n[3]=SOL;c.n[4]=0;c.n[5]=0;c.n[9]=20*SOL;
    write_config(&mut expected,c);expect_fee(&mut expected,&before,&after);assert_eq!(after,expected);
    let floor=g.rent.minimum_balance(0);let mut final_accounts=after;
    for cycle in 0..3_u64{
        let prefix=format!("cycle-{cycle}");let timestamp=[1_728_000,2_592_000,5_184_000][cycle as usize];
        let(before,after)=success(&bank,&format!("{prefix}-yield-and-pending"),vec![solana_system_interface::instruction::transfer(&key(220),&key(202),100*SOL),update_pool(),deposit_sol(2*SOL)],&mut nonce);
        let mut expected=before.clone();debit(&mut expected,key(220),102*SOL);credit(&mut expected,key(202),100*SOL);credit(&mut expected,target(9),2*SOL);
        let(t,s,e)=pool_ratio(&before[&JITO_POOL]);assert_eq!((s,e),(1000*SOL,0));let mut pool=before[&JITO_POOL].data().to_vec();put64(&mut pool,258,t+100*SOL);bytes(&mut expected,JITO_POOL,pool);
        let mut c=config(&before);c.n[5]+=2*SOL;write_config(&mut expected,c);expect_fee(&mut expected,&before,&after);assert_eq!(after,expected);
        if cycle==1{
            cycle_clock(&bank,"cycle-cadence-clock-before",timestamp-1);
            cycle_reject(&bank,"cycle-cadence-before",cycle_prepare(),6038,&mut nonce);
        }
        cycle_clock(&bank,&format!("{prefix}-time"),timestamp);
        if cycle==2{for index in 0..3{cycle_activity(&bank,&w,index,2,&format!("{prefix}-heartbeat-{index}"),&mut nonce);}}
        if cycle==0{rollback(&bank,"cycle-prepare-late-failure",vec![cycle_prepare(),failed_outer()],&mut nonce,Some(PROGRAM));}
        let(before,after)=success(&bank,&format!("{prefix}-prepare"),vec![cycle_prepare()],&mut nonce);let mut expected=before.clone();
        debit(&mut expected,target(9),1_529_500_000);credit(&mut expected,target(12),1_529_500_000);
        let mut c=config(&before);c.prepared=Some(timestamp as u64);c.n[0]+=1;write_config(&mut expected,c);
        bytes(&mut expected,target(1),cycle_header(&w,&before,timestamp as u64,bank.slot(),0,cycle,if cycle==2{7}else{0},if cycle==2{3}else{0}));
        expect_fee(&mut expected,&before,&after);assert_eq!(after,expected);assert_eq!(round(&after).seq,cycle);
        if cycle==0{
            cycle_reject(&bank,"cycle-already-active",cycle_prepare(),6034,&mut nonce);
            // Activity after the frozen zero-active snapshot cannot earn retroactively.
            cycle_activity(&bank,&w,0,0,"cycle-late-heartbeat",&mut nonce);
            assert_cycle_cache(&bank,"cycles-after-first-heartbeat");
            cycle_clock(&bank,"cycle-delayed-settlement",timestamp+86_400);
        }
        if cycle==2{cycle_activity(&bank,&w,3,2,"cycle-active-late-heartbeat",&mut nonce);}
        if cycle==1{rollback(&bank,"cycle-settlement-late-failure",vec![settle(&w),failed_outer()],&mut nonce,Some(PROGRAM));}
        let(before,after)=success(&bank,&format!("{prefix}-settle"),vec![settle(&w)],&mut nonce);let mut expected=before.clone();
        let zero=[19_000_000,28_500_000,0][cycle as usize];let carry=[19_000_000,28_500_000,2][cycle as usize];let liability=if cycle==2{66_499_998}else{0};
        debit(&mut expected,target(12),1_529_500_000);credit(&mut expected,w.recipients[0],1_121_000_000);credit(&mut expected,w.recipients[1],370_500_000);
        credit(&mut expected,target(13),38_000_000);debit(&mut expected,target(13),zero);credit(&mut expected,target(10),zero);
        let mut c=config(&before);c.n[1]+=370_500_000+zero;c.n[7]+=liability;c.n[8]=carry;c.n[10]+=1_900_000_000;c.n[11]+=1_121_000_000;c.n[12]+=370_500_000;c.n[13]+=liability;c.n[15]+=370_500_000;c.n[17]+=zero;
        write_config(&mut expected,c);bytes(&mut expected,target(1),cycle_settlement_bytes(&before,zero,carry,liability));
        if cycle==2{for index in 0..3{let k=target(3+index);let mut b=before[&k].data().to_vec();put64(&mut b,60,22_166_666);put64(&mut b,68,22_166_666);bytes(&mut expected,k,b);}}
        expect_fee(&mut expected,&before,&after);assert_eq!(after,expected);
        assert_eq!(config(&after).prepared,Some(timestamp as u64),"settlement never restarts cadence");
        if cycle==2{rollback(&bank,"cycle-integration-late-failure",vec![integrate(),failed_outer()],&mut nonce,Some(PROGRAM));}
        let(before,after)=success(&bank,&format!("{prefix}-integrate"),vec![integrate()],&mut nonce);let mut expected=before.clone();
        assert_eq!(before[&target(9)].lamports()-floor,470_500_000);assert_eq!(before[&target(12)].lamports(),floor);
        debit(&mut expected,target(9),470_500_000);credit(&mut expected,target(10),470_500_000);
        let mut c=config(&before);c.n[1]+=2*SOL;c.n[3]=expected[&target(10)].lamports()-floor;c.n[5]=0;c.n[9]+=2*SOL;write_config(&mut expected,c.clone());
        let mut idle=w.state_bytes()[1].clone();idle[22]=1;
        for(i,n)in [cycle,bank.clock().unix_timestamp as u64,1_900_000_000,1_529_500_000,2*SOL,c.n[1],0,0,0,liability,carry].into_iter().enumerate(){put64(&mut idle,23+i*8,n);}
        bytes(&mut expected,target(1),idle);expect_fee(&mut expected,&before,&after);assert_eq!(after,expected);
        assert_eq!(c.n[1],[22_389_500_000,24_788_500_000,27_159_000_000][cycle as usize]);
        assert_eq!(c.n[3]+value(c.n[2],(1100+100*cycle)*SOL,1000*SOL),c.n[1]);
        assert_eq!(after[&target(13)].lamports()-floor,c.n[7]+c.n[8]);
        for index in 0..6{assert_eq!(u64at(after[&target(3+index)].data(),60),if cycle==2&&index<3{22_166_666}else{0});}
        emit(json!({"kind":"cycle_complete","sequence":cycle,"period":cycle,"prepared_at":timestamp,"completed_at":bank.clock().unix_timestamp,
            "active_count":if cycle==2{3}else{0},"gross":1_900_000_000,"kif_current":38_000_000,"kif_carry_input":([0,19_000_000,28_500_000][cycle as usize]),
            "kif_zero_compound":zero,"kif_carry_next":carry,"new_liability":liability,"hwm":c.n[1],"pending_sol":c.n[5],"pending_tokens":c.n[4],"full_account_oracle":true}));
        final_accounts=after;
    }
    let c=config(&final_accounts);assert_eq!(c.n,[3,27_159_000_000,19*SOL,2_459_000_000,0,0,0,66_499_998,2,26*SOL,5_700_000_000,3_363_000_000,1_111_500_000,66_499_998,0,1_111_500_000,0,47_500_000,0]);
    assert_eq!(c.prepared,Some(5_184_000));assert_eq!(c.insufficient,None);assert_eq!(round(&final_accounts).state,0);
    assert_eq!(final_accounts[&target(11)],account(system_program::id(),floor,vec![]));
    assert_runtime_capabilities(&bank,"cycles-final",&capability_accounts);assert_cycle_cache(&bank,"cycles-final");
    emit(json!({"kind":"repeated_cycles_complete","status":"PASS","transactions":nonce,"rounds":3,"periods":[0,1,2],"active_counts":[0,0,3],"final_hwm":c.n[1],
        "pending_sol":0,"pending_tokens":0,"kif_carry":2,"kif_liability":66_499_998,"zero_active_compound":47_500_000,"cadence_seconds":864_000,
        "cadence_error":6038,"active_round_error":6034,"synthetic_clock":true,"real_walltime":false,"live_ready":false,"signature_verified":false,
        "actual_programs":["PIV1","System","Token8.0.0","SPLpool2.0.3","synthetic-caller"],
        "loaded_programs":["PIV1","synthetic-caller","Token8.0.0","SPLpool2.0.3","Stake5.1.0"],"stake_executed":false}));
}

// Normal valid-insufficiency timing profile. Existing profiles above are an exact
// retained prefix. No economic PIV state is seeded or changed outside its ABI.
fn insuff_base64(payload:&[u8])->String{
    let alphabet=b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";let mut out=String::new();
    for chunk in payload.chunks(3){let mut word=u32::from(chunk[0])<<16;
        if chunk.len()>1{word|=u32::from(chunk[1])<<8;}if chunk.len()>2{word|=u32::from(chunk[2]);}
        for (index,shift) in [18,12,6,0].into_iter().enumerate(){out.push(if index>chunk.len(){'='}else{alphabet[((word>>shift)&63) as usize] as char});}}
    out
}
fn insuff_attempt(bank:&Bank,name:&str,ix:Instruction,error:Option<InstructionError>,query:bool,nonce:&mut u32){
    let before=snapshot(bank,&format!("{name}-before"));let t=tx(bank,vec![ix],*nonce);*nonce+=1;
    let c=execute(bank,name,&t).unwrap();
    assert_eq!(c.status,error.clone().map_or(Ok(()),|error|Err(TransactionError::InstructionError(2,error))));
    let calls=c.inner_instructions.as_ref().unwrap().iter().flat_map(|calls|calls.iter()).collect::<Vec<_>>();
    assert_eq!(calls.len(),usize::from(query));
    if query{let call=&calls[0].instruction;assert_eq!(t.message.account_keys[call.program_id_index as usize],STAKE);
        assert_eq!(call.data,13_u32.to_le_bytes());assert!(call.accounts.is_empty());
        let returned=format!("Program return: {STAKE} {}",insuff_base64(&SOL.to_le_bytes()));
        assert_eq!(c.log_messages.as_ref().unwrap().iter().filter(|line|**line==returned).count(),1);}
    let events=c.log_messages.as_ref().unwrap().iter().filter(|line|line.starts_with("Program data: ")).collect::<Vec<_>>();
    let after=snapshot(bank,&format!("{name}-after"));let mut expected=before.clone();
    if error.is_none(){
        assert!(query);assert_eq!(config(&before).prepared,Some(864_000));assert_eq!(round(&before).state,0);
        let time=bank.clock().unix_timestamp as u64;let mut next=config(&before);next.insufficient=Some(time);write_config(&mut expected,next);
        let mut event=Sha256::digest(b"event:DistributionPreparationInsufficient")[..8].to_vec();event.extend(target(0).as_ref());
        for n in [time,665_000_000,869_565_218]{append(&mut event,n);}assert_eq!(event.len(),64);
        assert_eq!(events.len(),1);assert_eq!(events[0],&format!("Program data: {}",insuff_base64(&event)));
        emit(json!({"kind":"valid_insufficiency_event","case":name,"attempted_at":time,"target":665_000_000,
            "minimum":869_565_218,"event_data_hex":hex(&event),"config_only":true,"prepared_at":864_000,"round_idle":true}));
    }else{assert!(events.is_empty());}
    debit(&mut expected,key(242),c.fee_details.total_fee());assert_eq!(after,expected);
    assert!(matches!(execute(bank,&format!("{name}-replay"),&t),Err(TransactionError::AlreadyProcessed)));
    assert_eq!(snapshot(bank,&format!("{name}-replayed")),after);
    emit(json!({"kind":"insufficiency_attempt","case":name,"status":error.map_or("Ok".to_owned(),|error|format!("{error:?}")),
        "timestamp":bank.clock().unix_timestamp,"minimum_query_cpis":usize::from(query),"economic_cpis":0,
        "full_account_oracle":true,"identical_replay_no_fee":true,"prepared_at":config(&after).prepared,
        "insufficient_at":config(&after).insufficient}));
}
fn insuff_yield(bank:&Bank,name:&str,reserve:u64,pending:u64,nonce:&mut u32){
    let mut ixs=vec![solana_system_interface::instruction::transfer(&key(220),&key(202),reserve),update_pool()];
    if pending!=0{ixs.push(deposit_sol(pending));}
    let(before,after)=success(bank,name,ixs,nonce);let mut expected=before.clone();debit(&mut expected,key(220),reserve+pending);
    credit(&mut expected,key(202),reserve);let(t,s,e)=pool_ratio(&before[&JITO_POOL]);assert_eq!((s,e),(1000*SOL,0));
    let mut pool=before[&JITO_POOL].data().to_vec();put64(&mut pool,258,t+reserve);bytes(&mut expected,JITO_POOL,pool);
    if pending!=0{credit(&mut expected,target(9),pending);let mut c=config(&before);c.n[5]+=pending;write_config(&mut expected,c);}
    expect_fee(&mut expected,&before,&after);assert_eq!(after,expected);
    emit(json!({"kind":"insufficiency_yield_input","case":name,"external_reserve_transfer":reserve,"explicit_pending_sol":pending,
        "pool_total":t+reserve,"pool_supply":s,"real_pool_update":true,"live_yield_proof":false,"full_account_oracle":true}));
}
fn insuff_ready_bytes(w:&World,before:&Accounts)->Vec<u8>{
    let prior=before[&target(1)].data();assert_eq!(prior[11],0);assert_eq!(prior[22],1);
    let mut b=w.state_bytes()[1][..14].to_vec();b[11]=1;append(&mut b,1);b.push(1);b.extend(&prior[23..111]);
    for n in [1_814_401,1,0,22_389_500_000,19*SOL,1_489_500_000,24_289_500_000,1200*SOL,1000*SOL,0,1,
        1_900_000_000,0,1_121_000_000,370_500_000,370_500_000,38_000_000,0,1_529_500_000,1,1,0,
        1_274_583_333,833_333_334,1,1_529_347_049,22_760_000_000]{append(&mut b,n);}
    b.extend(1_u16.to_le_bytes());for i in 0..16{append(&mut b,if i==14{1}else if i==15{1_529_500_000}else{0});}
    for k in [w.recipients[0],w.recipients[1],target(2)]{b.extend(k.as_ref());}append(&mut b,0);
    for i in 0..6{b.extend(key(96-i).as_ref());}b.extend([1,1]);append(&mut b,0);append(&mut b,19_000_000);
    append(&mut b,370_500_000);append(&mut b,22_760_000_000);b.extend([0;13*8]);assert_eq!(b.len(),891);b
}
#[test]
fn valid_insufficiency_production_lifecycle(){
    assert_eq!(env::var("RAYON_NUM_THREADS").as_deref(),Ok("1"));let path=env::var("PIV1_BANK_ACCOUNTS_DIR").unwrap();let path=Path::new(&path);
    assert!(path.is_absolute());assert_eq!(path.parent().unwrap().canonicalize().unwrap(),path.parent().unwrap());assert!(!path.exists());fs::create_dir(path).unwrap();
    let(w,extras)=external_world();let g=genesis(&w,extras);let leader=SlotLeader{id:key(246),vote_address:key(247)};
    let root=Bank::new_with_paths_for_tests(&g,Some(bounded()),vec![path.to_path_buf()],Some(leader));
    let(root,forks)=root.wrap_with_bank_forks_for_tests();let bank=Bank::new_from_parent_with_bank_forks(&forks,root,leader,1);let mut nonce=0;
    let mut capability_accounts=Accounts::new();for feature in REQUIRED_FEATURES{capability_accounts.insert(feature,AccountSharedData::from(g.accounts[&feature].clone()));}
    for k in [STAKE,program_data(STAKE)]{capability_accounts.insert(k,AccountSharedData::from(g.accounts[&k].clone()));}
    assert_runtime_capabilities(&bank,"insufficiency-initial",&capability_accounts);
    emit(json!({"kind":"fixture","profile":"valid-insufficiency","genesis_hash":g.hash().to_string(),"synthetic_external_pool":true,"synthetic_governance":true,
        "synthetic_clock":true,"heap_bytes":HEAP,"unsigned":true,"economic_piv_seeded":false}));
    let initial=snapshot(&bank,"insufficiency-initial");for i in 0..16{assert!(!initial.contains_key(&target(i)));}
    let(before,after)=success(&bank,"insufficiency-initialize",vec![w.outer.clone()],&mut nonce);let mut expected=before.clone();let states=w.state_bytes();
    for i in 0..16{let d=if i<9{states[i].clone()}else if i<14{vec![]}else{w.token_bytes()};expected.insert(target(i),account(World::target_owner(i),w.target_balance(i),d));}
    debit(&mut expected,w.roles[w.payer].key,w.shortfall());expect_fee(&mut expected,&before,&after);assert_eq!(after,expected);
    let(before,after)=success(&bank,"insufficiency-intake",vec![deposit_sol(SOL),deposit_token(19*SOL),
        solana_system_interface::instruction::transfer(&key(220),&target(11),50_000_000)],&mut nonce);let mut expected=before.clone();
    debit(&mut expected,key(220),SOL+50_000_000);credit(&mut expected,target(9),SOL);credit(&mut expected,target(11),50_000_000);
    token_delta(&mut expected,key(221),-(19*SOL as i64));token_delta(&mut expected,target(15),19*SOL as i64);
    let mut c=config(&before);c.n[4]=19*SOL;c.n[5]=SOL;write_config(&mut expected,c);expect_fee(&mut expected,&before,&after);assert_eq!(after,expected);
    let(before,after)=success(&bank,"insufficiency-bootstrap",vec![bootstrap()],&mut nonce);let mut expected=before.clone();
    debit(&mut expected,target(9),SOL);credit(&mut expected,target(10),SOL);token_delta(&mut expected,target(15),-(19*SOL as i64));token_delta(&mut expected,target(14),19*SOL as i64);
    let mut c=config(&before);c.n[1]=20*SOL;c.n[2]=19*SOL;c.n[3]=SOL;c.n[4]=0;c.n[5]=0;c.n[9]=20*SOL;
    write_config(&mut expected,c);expect_fee(&mut expected,&before,&after);assert_eq!(after,expected);
    insuff_yield(&bank,"insufficiency-primer-yield",100*SOL,2*SOL,&mut nonce);cycle_clock(&bank,"insufficiency-primer-time",864_000);
    let(before,after)=success(&bank,"insufficiency-primer-prepare",vec![cycle_prepare()],&mut nonce);let mut expected=before.clone();
    debit(&mut expected,target(9),1_529_500_000);credit(&mut expected,target(12),1_529_500_000);
    let mut c=config(&before);c.prepared=Some(864_000);c.n[0]=1;write_config(&mut expected,c);
    bytes(&mut expected,target(1),cycle_header(&w,&before,864_000,1,0,0,0,0));expect_fee(&mut expected,&before,&after);assert_eq!(after,expected);
    cycle_activity(&bank,&w,0,0,"insufficiency-late-heartbeat",&mut nonce);assert_cycle_cache(&bank,"insufficiency-after-heartbeat");
    let(before,after)=success(&bank,"insufficiency-primer-settle",vec![settle(&w)],&mut nonce);let mut expected=before.clone();
    debit(&mut expected,target(12),1_529_500_000);credit(&mut expected,w.recipients[0],1_121_000_000);credit(&mut expected,w.recipients[1],370_500_000);
    credit(&mut expected,target(13),19_000_000);credit(&mut expected,target(10),19_000_000);
    let mut c=config(&before);c.n[1]+=389_500_000;c.n[8]=19_000_000;c.n[10]+=1_900_000_000;c.n[11]+=1_121_000_000;c.n[12]+=370_500_000;c.n[15]+=370_500_000;c.n[17]+=19_000_000;
    write_config(&mut expected,c);bytes(&mut expected,target(1),cycle_settlement_bytes(&before,19_000_000,19_000_000,0));expect_fee(&mut expected,&before,&after);assert_eq!(after,expected);
    let(before,after)=success(&bank,"insufficiency-primer-integrate",vec![integrate()],&mut nonce);let mut expected=before.clone();
    debit(&mut expected,target(9),470_500_000);credit(&mut expected,target(10),470_500_000);
    let mut c=config(&before);c.n[1]=22_389_500_000;c.n[3]=1_489_500_000;c.n[5]=0;c.n[9]=22*SOL;write_config(&mut expected,c);
    let mut idle=w.state_bytes()[1].clone();idle[22]=1;
    for(i,n)in [0,864_000,1_900_000_000,1_529_500_000,2*SOL,22_389_500_000,0,0,0,0,19_000_000].into_iter().enumerate(){put64(&mut idle,23+i*8,n);}
    bytes(&mut expected,target(1),idle);expect_fee(&mut expected,&before,&after);assert_eq!(after,expected);
    insuff_yield(&bank,"insufficiency-small-yield",50*SOL,0,&mut nonce);
    cycle_clock(&bank,"insufficiency-cadence-before",1_727_999);
    insuff_attempt(&bank,"insufficiency-ten-day-reject",prepare(),Some(InstructionError::Custom(6038)),false,&mut nonce);
    cycle_clock(&bank,"insufficiency-first-time",1_728_000);
    let mut malformed=prepare();malformed.data[8]=2;
    insuff_attempt(&bank,"insufficiency-malformed-initial",malformed.clone(),Some(InstructionError::InvalidInstructionData),false,&mut nonce);
    insuff_attempt(&bank,"insufficiency-first",prepare(),None,true,&mut nonce);
    insuff_attempt(&bank,"insufficiency-immediate",prepare(),Some(InstructionError::Custom(6039)),true,&mut nonce);
    cycle_clock(&bank,"insufficiency-before-retry",1_814_399);
    insuff_attempt(&bank,"insufficiency-before-boundary",prepare(),Some(InstructionError::Custom(6039)),true,&mut nonce);
    cycle_clock(&bank,"insufficiency-at-retry",1_814_400);
    insuff_attempt(&bank,"insufficiency-malformed-boundary",malformed,Some(InstructionError::InvalidInstructionData),false,&mut nonce);
    insuff_attempt(&bank,"insufficiency-exact-retry",prepare(),None,true,&mut nonce);
    // One normal pending lamport makes the sufficient target's independently
    // rounded residual exactly cover HWM; never weaken the partition-floor guard.
    insuff_yield(&bank,"insufficiency-sufficient-yield",50*SOL,1,&mut nonce);
    cycle_clock(&bank,"insufficiency-sufficient-time",1_814_401);
    let(before,after)=success(&bank,"insufficiency-sufficient-prepare",vec![prepare()],&mut nonce);let mut expected=before.clone();
    debit(&mut expected,target(9),1);credit(&mut expected,target(12),1);let mut c=config(&before);c.prepared=Some(1_814_401);c.n[0]=2;
    write_config(&mut expected,c);bytes(&mut expected,target(1),insuff_ready_bytes(&w,&before));expect_fee(&mut expected,&before,&after);assert_eq!(after,expected);
    let c=config(&after);assert_eq!(c.n,[2,22_389_500_000,19*SOL,1_489_500_000,0,1,0,0,19_000_000,22*SOL,1_900_000_000,1_121_000_000,370_500_000,0,0,370_500_000,0,19_000_000,0]);
    assert_eq!(c.insufficient,Some(1_814_400));assert_eq!(round(&after).state,1);assert_eq!(round(&after).seq,1);
    assert_eq!(after[&target(11)].lamports(),g.rent.minimum_balance(0)+50_000_000);
    assert!(after.keys().all(|key|![leg(0),leg(1),stake(0),stake(1)].contains(key)));
    assert_runtime_capabilities(&bank,"insufficiency-final",&capability_accounts);assert_cycle_cache(&bank,"insufficiency-final");
    emit(json!({"kind":"valid_insufficiency_complete","status":"PASS","transactions":nonce,"completed_rounds":1,"final_round_state":1,
        "final_sequence":1,"next_sequence":2,"prepared_at":1_814_401,"insufficient_at":1_814_400,"prior_prepared_at":864_000,
        "retry_seconds":86_400,"cadence_seconds":864_000,"insufficiency_target":665_000_000,"insufficiency_minimum":869_565_218,
        "sufficient_target":1_274_583_333,"sufficient_minimum":833_333_334,"round_minimum":1_529_347_049,
        "final_hwm":22_389_500_000_u64,"proposed_hwm":22_760_000_000_u64,"pending_sol":1,"pending_tokens":0,"escrow":1,"kif_carry":19_000_000,
        "withdrawal_legs_created":0,"stake_minimum":SOL,"synthetic_clock":true,"real_walltime":false,"live_ready":false,"signature_verified":false,
        "actual_programs":["PIV1","System","Token8.0.0","SPLpool2.0.3","Stake5.1.0","synthetic-caller"]}));
}

// Isolated P-035 functional profile. Existing source above remains unchanged.
// Abstract reward observation: paired native/delegation growth before cooldown.
// This is not inflation execution, a donation, or a claimed vote-credit history.
fn external_cooldown_reward(bank:&Bank)->Accounts{
    let before=snapshot(bank,"external-cooldown-reward-before");let capitalization=bank.capitalization();
    assert_eq!(bank.epoch(),0);assert_eq!(u64at(before[&stake(0)].data(),156),5*SOL);
    assert_eq!(before[&stake(0)].lamports(),5*SOL+STAKE_RENT);assert_eq!(before[&stake(0)].owner(),&STAKE);
    let mut expected=before.clone();debit(&mut expected,key(220),10_000);credit(&mut expected,stake(0),10_000);
    let mut data=before[&stake(0)].data().to_vec();put64(&mut data,156,5*SOL+10_000);bytes(&mut expected,stake(0),data);
    let stake_key=stake(0);let donor=key(220);
    // Pinned Bank::store_accounts updates its StakesCache from these same bytes.
    bank.store_accounts((bank.slot(),&[(&stake_key,&expected[&stake_key]),(&donor,&expected[&donor])][..]),None);
    let after=snapshot(bank,"external-cooldown-reward-after");assert_eq!(after,expected);
    assert_eq!(bank.capitalization(),capitalization);
    assert_eq!(before.values().map(|a|u128::from(a.lamports())).sum::<u128>(),after.values().map(|a|u128::from(a.lamports())).sum::<u128>());
    emit(json!({"kind":"external_cooldown_reward","synthetic":true,"inflation_executed":false,"credits_observed_unchanged":true,
        "lamports":10_000,"delegation_increment":10_000,"donor_debit":10_000,"epoch":0,"before_cooldown":true,
        "original_delegated":5_000_000_000_u64,"updated_delegated":5_000_010_000_u64,"stake_rent":STAKE_RENT,
        "all_piv_accounts_unchanged":true,"capitalization_unchanged":true,"full_account_oracle":true}));after
}
fn assert_reward_finalization(before:&Accounts,after:&Accounts){
    let native=5_000_010_000_u64;assert_eq!(before[&stake(0)].lamports(),native+STAKE_RENT);
    assert_eq!(u64at(before[&stake(0)].data(),156),native);let metadata=before[&leg(0)].lamports();assert_eq!(metadata,2_721_360);
    let prior=round(before);let mut expected=before.clone();expected.remove(&stake(0));expected.remove(&leg(0));
    credit(&mut expected,target(12),native);credit(&mut expected,target(11),STAKE_RENT+metadata);
    let mut header=before[&target(1)].data().to_vec();header[11]=2;
    for(j,delta)in[(5,5*SOL),(6,native+STAKE_RENT),(7,STAKE_RENT),(8,metadata),(9,10_000),(13,1),(14,native)]{round_cumulative(&mut header,j,prior.cumulative[j]+delta);}
    bytes(&mut expected,target(1),header);expect_fee(&mut expected,before,after);assert_eq!(*after,expected);
    emit(json!({"kind":"cooldown_reward_finalized","epoch":14,"reward":10_000,"original_total":8_608_636_361_u64,
        "recovered_stake_rent":4_565_760,"recovered_metadata_rent":5_442_720,"fixed_snapshot_unchanged":true,"full_account_oracle":true}));
}
fn reward_next_header(w:&World,before:&Accounts,bank:&Bank)->Vec<u8>{
    let prior=before[&target(1)].data();assert_eq!(prior[11],0);assert_eq!(prior[22],1);
    let mut b=w.state_bytes()[1][..14].to_vec();b[11]=2;append(&mut b,1);b.push(1);b.extend(&prior[23..111]);
    for n in [2_592_000,bank.slot(),14,121_955_000_003,111_608_750_001,200_000_002,121_955_000_003,
        1_191_391_363_642,1_092_108_750_001,0,1,10_000,10_000,5900,1950,1950,200,0,8050,0,0,0,0,0,0,0,121_955_001_953]{append(&mut b,n);}
    b.extend(1_u16.to_le_bytes());for i in 0..16{append(&mut b,if i==14||i==15{8050}else{0});}
    for k in [w.recipients[0],w.recipients[1],target(2)]{b.extend(k.as_ref());}append(&mut b,0);
    for i in 0..6{b.extend(key(96-i).as_ref());}b.extend([1,1]);append(&mut b,1);append(&mut b,0);
    append(&mut b,1950);append(&mut b,121_955_001_953);b.extend([0;13*8]);assert_eq!(b.len(),891);b
}
#[test]
fn cooldown_reward_production_lifecycle(){
    assert_eq!(env::var("RAYON_NUM_THREADS").as_deref(),Ok("1"));let path=env::var("PIV1_BANK_ACCOUNTS_DIR").unwrap();let path=Path::new(&path);
    assert!(path.is_absolute());assert_eq!(path.parent().unwrap().canonicalize().unwrap(),path.parent().unwrap());assert!(!path.exists());fs::create_dir(path).unwrap();
    let(w,extras)=external_world();let g=genesis(&w,extras);let leader=SlotLeader{id:key(246),vote_address:key(247)};
    // Bank genesis stores these canonical account fields unchanged; retain the
    // helper's complete account (including rent_epoch) as the independent oracle.
    let mut capability_accounts=Accounts::new();
    for feature in REQUIRED_FEATURES{
        let account=AccountSharedData::from(g.accounts[&feature].clone());
        assert_eq!(account.owner(),&solana_sdk_ids::feature::id());assert!(!account.executable());
        assert_eq!(account.lamports(),g.rent.minimum_balance(account.data().len()).max(1));
        capability_accounts.insert(feature,account);
    }
    for key in [STAKE,program_data(STAKE)]{capability_accounts.insert(key,AccountSharedData::from(g.accounts[&key].clone()));}
    let root=Bank::new_with_paths_for_tests(&g,Some(bounded()),vec![path.to_path_buf()],Some(leader));
    assert_runtime_capabilities(&root,"root",&capability_accounts);
    let(root,forks)=root.wrap_with_bank_forks_for_tests();let mut bank=Bank::new_from_parent_with_bank_forks(&forks,root,leader,1);let mut nonce=0;
    assert_runtime_capabilities(&bank,"initial-child",&capability_accounts);
    emit(json!({"kind":"fixture","genesis_hash":g.hash().to_string(),"synthetic_external_pool":true,"synthetic_governance":true,"real_token":"8.0.0","real_pool":"2.0.3","real_stake":"5.1.0","heap_bytes":HEAP,"unsigned":true,"economic_piv_seeded":false}));
    let initial=snapshot(&bank,"initial");for i in 0..16{assert!(!initial.contains_key(&target(i)));}
    let epoch_rewards=&initial[&sysvar::epoch_rewards::id()];
    assert_eq!(epoch_rewards,&account(sysvar::id(),g.rent.minimum_balance(81),vec![0;81]));
    emit(json!({"kind":"epoch_rewards_genesis","source":"pinned runtime genesis helper","bytes":81,"active":false,"helper_activates_features":false}));
    let(before,after)=success(&bank,"initialize",vec![w.outer.clone()],&mut nonce);
    let mut expected=before.clone();let states=w.state_bytes();for i in 0..16{let data=if i<9{states[i].clone()}else if i<14{vec![]}else{w.token_bytes()};expected.insert(target(i),account(World::target_owner(i),w.target_balance(i),data));}
    debit(&mut expected,w.roles[w.payer].key,w.shortfall());expect_fee(&mut expected,&before,&after);assert_eq!(after,expected);
    let funding=solana_system_interface::instruction::transfer(&key(220),&target(11),50_000_000);
    let(before,after)=success(&bank,"fund-operations",vec![funding],&mut nonce);let mut expected=before.clone();debit(&mut expected,key(220),50_000_000);credit(&mut expected,target(11),50_000_000);expect_fee(&mut expected,&before,&after);assert_eq!(after,expected);
    let(before,after)=success(&bank,"intake-sol",vec![deposit_sol(100*SOL)],&mut nonce);let mut expected=before.clone();debit(&mut expected,key(220),100*SOL);credit(&mut expected,target(9),100*SOL);let mut c=config(&before);c.n[5]+=100*SOL;write_config(&mut expected,c);expect_fee(&mut expected,&before,&after);assert_eq!(after,expected);
    let(before,after)=success(&bank,"intake-jito",vec![deposit_token(19*SOL)],&mut nonce);let mut expected=before.clone();token_delta(&mut expected,key(221),-((19*SOL) as i64));token_delta(&mut expected,target(15),(19*SOL) as i64);let mut c=config(&before);c.n[4]+=19*SOL;write_config(&mut expected,c);expect_fee(&mut expected,&before,&after);assert_eq!(after,expected);
    let(before,after)=success(&bank,"bootstrap",vec![bootstrap()],&mut nonce);let mut expected=before.clone();debit(&mut expected,target(9),100*SOL);credit(&mut expected,target(10),100*SOL);token_delta(&mut expected,target(15),-(19*SOL as i64));token_delta(&mut expected,target(14),19*SOL as i64);let mut c=config(&before);c.n[1]=119*SOL;c.n[2]=19*SOL;c.n[3]=100*SOL;c.n[4]=0;c.n[5]=0;c.n[9]=119*SOL;write_config(&mut expected,c);expect_fee(&mut expected,&before,&after);assert_eq!(after,expected);
    let(before,after)=success(&bank,"principal-stake",vec![principal_deposit(100*SOL)],&mut nonce);let mut expected=before.clone();debit(&mut expected,target(10),100*SOL);credit(&mut expected,key(202),100*SOL);token_delta(&mut expected,target(14),100*SOL as i64);
    let mut p=before[&JITO_POOL].data().to_vec();put64(&mut p,258,1100*SOL);put64(&mut p,266,1100*SOL);bytes(&mut expected,JITO_POOL,p);let mut mint=before[&JITO_MINT].data().to_vec();put64(&mut mint,36,1100*SOL);bytes(&mut expected,JITO_MINT,mint);let mut c=config(&before);c.n[2]=119*SOL;c.n[3]=0;write_config(&mut expected,c);expect_fee(&mut expected,&before,&after);assert_eq!(after,expected);
    // An external reserve donation followed by the actual SPL update changes
    // official pool value. This is a local yield stimulus, not inflation proof.
    let(before,after)=success(&bank,"external-yield-stimulus",vec![solana_system_interface::instruction::transfer(&key(220),&key(202),100*SOL),update_pool()],&mut nonce);
    let mut expected=before.clone();debit(&mut expected,key(220),100*SOL);credit(&mut expected,key(202),100*SOL);
    let mut pool=before[&JITO_POOL].data().to_vec();put64(&mut pool,258,1200*SOL);bytes(&mut expected,JITO_POOL,pool);expect_fee(&mut expected,&before,&after);assert_eq!(after,expected);
    let(before,after)=success(&bank,"guardian-heartbeat",vec![heartbeat(&w)],&mut nonce);let mut expected=before.clone();let mut reward=before[&target(3)].data().to_vec();reward[51]=1;put64(&mut reward,52,0);bytes(&mut expected,target(3),reward);expect_fee(&mut expected,&before,&after);assert_eq!(after,expected);
    let(before,after)=success(&bank,"pending-before-snapshot",vec![deposit_sol(SOL/10)],&mut nonce);let mut expected=before.clone();debit(&mut expected,key(220),SOL/10);credit(&mut expected,target(9),SOL/10);let mut c=config(&before);c.n[5]+=SOL/10;write_config(&mut expected,c);expect_fee(&mut expected,&before,&after);assert_eq!(after,expected);
    let(before,after)=success(&bank,"prepare-withdrawal",vec![prepare()],&mut nonce);let r=round(&after);assert_eq!(r.state,1);assert_eq!(r.seq,0);let gross=value(119*SOL,1200*SOL,1100*SOL)-119*SOL;
    assert_eq!(r.snapshot[11],gross);let htfp=value(gross,5900,10000);let team=value(gross,1950,10000);let kif=value(gross,200,10000);let compound=value(gross,1950,10000);let splitdust=gross-htfp-team-kif-compound;let outgoing=htfp+team+kif;
    assert_eq!(&r.snapshot[13..19],&[htfp,compound,team,kif,splitdust,outgoing]);assert_eq!(r.snapshot[20],SOL/10);
    let budget=outgoing-SOL/10;let target_q=(((u128::from(budget)+1)*u128::from(1100*SOL)-1)/u128::from(1200*SOL)) as u64;assert_eq!(r.snapshot[22],target_q);assert!(value(target_q,1200*SOL,1100*SOL)>5*SOL+SOL);
    let mut expected=before.clone();debit(&mut expected,target(9),SOL/10);credit(&mut expected,target(12),SOL/10);let mut c=config(&before);c.prepared=Some(bank.clock().unix_timestamp as u64);c.n[0]=1;write_config(&mut expected,c);bytes(&mut expected,target(1),prepared_bytes(&w,bank.clock().unix_timestamp as u64,bank.slot(),bank.epoch(),gross,target_q));expect_fee(&mut expected,&before,&after);assert_eq!(after,expected);
    let(before,after)=success(&bank,"pending-after-snapshot",vec![deposit_sol(SOL/5),deposit_token(SOL/2)],&mut nonce);let mut expected=before.clone();debit(&mut expected,key(220),SOL/5);credit(&mut expected,target(9),SOL/5);token_delta(&mut expected,key(221),-(SOL as i64/2));token_delta(&mut expected,target(15),SOL as i64/2);let mut c=config(&before);c.n[5]+=SOL/5;c.n[4]+=SOL/2;write_config(&mut expected,c);expect_fee(&mut expected,&before,&after);assert_eq!(after,expected);
    for i in 0..2{
        let(before,after)=success(&bank,&format!("leg-{i}"),vec![initiate(i)],&mut nonce);let prior=round(&before);let r=round(&after);let(t,s,_)=pool_ratio(&before[&JITO_POOL]);
        let maximum=((((5*SOL) as u128+1)*u128::from(s)-1)/u128::from(t)) as u64;let q=(target_q-prior.cumulative[0]).min(maximum);let native=value(q,t,s);assert!(native>=SOL);
        assert_eq!(r.cumulative[0],prior.cumulative[0]+q);assert_eq!(r.cumulative[2],prior.cumulative[2]+q);assert_eq!(r.cumulative[4],prior.cumulative[4]+native);assert_eq!(r.cumulative[12],i+1);
        assert_eq!(units(&before[&target(14)])-units(&after[&target(14)]),q);assert_eq!(pool_ratio(&after[&JITO_POOL]),(t-native,s-q,bank.epoch()));
        assert_eq!(before[&source(i as u8)].lamports()-after[&source(i as u8)].lamports(),native);assert_eq!(after[&stake(i)].lamports(),native+STAKE_RENT);
        assert_eq!(u64at(after[&stake(i)].data(),172),bank.epoch());assert_eq!(before[&target(0)],after[&target(0)]);
        let mut expected=before.clone();debit(&mut expected,source(i as u8),native);
        let mut source_data=before[&source(i as u8)].data().to_vec();put64(&mut source_data,156,6*SOL-native);bytes(&mut expected,source(i as u8),source_data);
        let mut destination=stake_state(Some(key(210+i as u8)),native);destination[12..44].copy_from_slice(authority().as_ref());destination[44..76].copy_from_slice(authority().as_ref());put64(&mut destination,172,bank.epoch());
        expected.insert(stake(i),account(STAKE,native+STAKE_RENT,destination));let metadata_rent=g.rent.minimum_balance(LEG_SIZE);
        expected.insert(leg(i),account(PROGRAM,metadata_rent,leg_bytes(i,bank.epoch(),t,s,q,native,prior.snapshot[23])));debit(&mut expected,target(11),STAKE_RENT+metadata_rent);
        token_delta(&mut expected,target(14),-(q as i64));let mut mint=before[&JITO_MINT].data().to_vec();let next_supply=u64at(&mint,36)-q;put64(&mut mint,36,next_supply);bytes(&mut expected,JITO_MINT,mint);
        let mut pool=before[&JITO_POOL].data().to_vec();put64(&mut pool,258,t-native);put64(&mut pool,266,s-q);bytes(&mut expected,JITO_POOL,pool);
        let mut list=before[&key(201)].data().to_vec();put64(&mut list,9+i as usize*73,6*SOL+STAKE_RENT-native);bytes(&mut expected,key(201),list);
        let mut header=before[&target(1)].data().to_vec();for(j,delta)in[(0,q),(2,q),(3,native),(4,native),(11,1),(12,1)]{round_cumulative(&mut header,j,prior.cumulative[j]+delta);}bytes(&mut expected,target(1),header);
        expect_fee(&mut expected,&before,&after);assert_eq!(after,expected);
    }
    let assigned=snapshot(&bank,"all-legs-assigned");assert_eq!(round(&assigned).cumulative[0],target_q);assert_eq!(round(&assigned).cumulative[12],2);
    let rewarded=external_cooldown_reward(&bank);
    rollback(&bank,"active-finalization",vec![finalize(0)],&mut nonce,None);
    // Advance real Bank epochs/history. No PIV/external economic account writes.
    let operations_before=assigned[&target(11)].lamports();let mut finalized=0;
    // Independent fixed-point9% recurrence with rewarded delegation gives
    // epoch13 [75_547_234,54_524_390], epoch14 [0,0]. Actual Stake decides readiness.
    // Bind the finite bound to this fixture's actual assigned delegation totals.
    assert_eq!([u64at(rewarded[&stake(0)].data(),156),u64at(assigned[&stake(1)].data(),156)],[5_000_010_000,3_608_636_361]);
    assert_eq!(u64at(assigned[&source(0)].data(),156)+u64at(assigned[&source(1)].data(),156),3_391_363_639);
    for i in 0..2{assert_eq!(u64at(assigned[&stake(i)].data(),164),u64::MAX);assert_eq!(u64at(assigned[&stake(i)].data(),172),0);}
    for epoch in 1..=FINALIZATION_EPOCH_BOUND{
        let slot=bank.epoch_schedule().get_first_slot_in_epoch(epoch);
        // Construct before taking the fork write lock: Bank construction itself
        // may consult the program cache and its fork graph.
        let warped=Bank::warp_from_parent(bank,leader,slot);
        let warped=forks.write().unwrap().insert(warped).clone_without_scheduler();
        assert_runtime_capabilities(&warped,"epoch-warp",&capability_accounts);
        assert_registered_cache(&warped,"epoch-warp");
        bank=Bank::new_from_parent_with_bank_forks(&forks,warped,leader,slot+1);assert_eq!(bank.epoch(),epoch);
        assert_runtime_capabilities(&bank,"epoch-child",&capability_accounts);
        assert_registered_cache(&bank,"epoch-child");
        let(before,after)=success(&bank,&format!("epoch-{epoch}-pool-update"),vec![update_validators(),update_pool()],&mut nonce);
        let mut expected=before.clone();let mut pool=before[&JITO_POOL].data().to_vec();let(t,s,_)=pool_ratio(&before[&JITO_POOL]);put64(&mut pool,274,epoch);put64(&mut pool,419,s);put64(&mut pool,427,t);bytes(&mut expected,JITO_POOL,pool);
        let mut list=before[&key(201)].data().to_vec();for i in 0..2{put64(&mut list,25+i*73,epoch);}bytes(&mut expected,key(201),list);expect_fee(&mut expected,&before,&after);assert_eq!(after,expected);
        if finalized==0{
            let before=snapshot(&bank,&format!("epoch-{epoch}-finalization-before"));let t=tx(&bank,vec![finalize(1)],nonce);nonce+=1;let c=execute(&bank,&format!("epoch-{epoch}-finalize-second"),&t).unwrap();let after=snapshot(&bank,&format!("epoch-{epoch}-finalization-after"));
            if c.status.is_err(){inactive_failure(&c,&t);let mut expected=before;debit(&mut expected,key(242),c.fee_details.total_fee());assert_eq!(after,expected);continue;}
            assert_finalization(&before,&after,1);assert_eq!(round(&after).cumulative[13],1);
        }
        rollback(&bank,"reward-finalization-late-failure",vec![finalize(0),failed_outer()],&mut nonce,Some(STAKE));
        let(before,after)=success(&bank,"finalize-first",vec![finalize(0)],&mut nonce);assert_eq!(round(&after).cumulative[13],2);
        assert_eq!(round(&after).state,2);assert_reward_finalization(&before,&after);
        finalized=2;break;
    }
    assert_eq!(finalized,2,"bounded real deactivation progression must complete");
    // Preserved G correctly required recovery: projected protected value was one
    // lamport below the settled HWM. This fresh success fixture adds an explicit
    // external economic stimulus, never resetting or bypassing that recovery.
    // At this exact ratio, 3 reserve lamports are the minimum needed to increase
    // the held principal token floor by 1; frozen distribution amounts stay fixed.
    let(before,after)=success(&bank,"external-reserve-appreciation",vec![solana_system_interface::instruction::transfer(&key(220),&key(202),3),update_pool()],&mut nonce);
    let mut expected=before.clone();debit(&mut expected,key(220),3);credit(&mut expected,key(202),3);
    let(t,s,_)=pool_ratio(&before[&JITO_POOL]);let q=units(&before[&target(14)]);let prior_value=value(q,t,s);
    assert_eq!((q,t,s),(111_108_750_001,1_191_391_363_639,1_092_108_750_001));
    assert_eq!(value(q,t+2,s),prior_value);assert_eq!(value(q,t+3,s),prior_value+1);
    let mut pool=before[&JITO_POOL].data().to_vec();put64(&mut pool,258,t+3);bytes(&mut expected,JITO_POOL,pool);
    expect_fee(&mut expected,&before,&after);assert_eq!(after,expected);
    assert_eq!(after[&target(1)],before[&target(1)],"frozen round remains unchanged");
    assert_eq!(value(units(&after[&target(14)]),pool_ratio(&after[&JITO_POOL]).0,s),121_209_545_456);
    let before_settle=snapshot(&bank,"before-settlement");let r=round(&before_settle);assert_eq!(r.cumulative[9],10_000);assert_eq!(r.cumulative[10],0);
    let metadata_rents=assigned[&leg(0)].lamports()+assigned[&leg(1)].lamports();
    assert_eq!(r.cumulative[7],2*STAKE_RENT);assert_eq!(r.cumulative[8],metadata_rents);
    assert_eq!(before_settle[&target(11)].lamports(),operations_before+2*STAKE_RENT+metadata_rents);
    let eligible=r.snapshot[20]+r.cumulative[5];let net=eligible.min(outgoing);let paid_h=value(net,5900,8050).min(htfp);let paid_t=value(net,1950,8050).min(team);let paid_k=value(net,200,8050).min(kif);let paid=paid_h+paid_t+paid_k;let dust=net-paid;let conservative=eligible-net;
    rollback(&bank,"settlement-late-failure",vec![settle(&w),failed_outer()],&mut nonce,Some(PROGRAM));
    let(before,after)=success(&bank,"settlement",vec![settle(&w)],&mut nonce);let mut expected=before.clone();debit(&mut expected,target(12),paid);credit(&mut expected,w.recipients[0],paid_h);credit(&mut expected,w.recipients[1],paid_t);credit(&mut expected,target(13),paid_k);
    let mut c=config(&before);let hwm_delta=compound+splitdust+r.snapshot[21]+dust+conservative;c.n[1]+=hwm_delta;c.n[10]+=gross;c.n[11]+=paid_h;c.n[12]+=paid_t;c.n[13]+=paid_k;c.n[7]+=paid_k;c.n[15]+=compound;c.n[16]+=splitdust+r.snapshot[21]+dust+conservative;
    c.n[6]=10_000;c.n[18]+=10_000;write_config(&mut expected,c.clone());let mut reward=before[&target(3)].data().to_vec();put64(&mut reward,60,paid_k);put64(&mut reward,68,paid_k);bytes(&mut expected,target(3),reward);
    let mut header=before[&target(1)].data().to_vec();header[11]=3;header[13]=1;round_cumulative(&mut header,15,0);
    for(j,n)in [r.settlement[0],r.settlement[1],net,paid_h,paid_t,paid_k,dust,paid,r.cumulative[14]-paid,conservative,paid_k,0,0,hwm_delta,c.n[1]].into_iter().enumerate(){round_settlement(&mut header,j,n);}bytes(&mut expected,target(1),header);expect_fee(&mut expected,&before,&after);assert_eq!(after,expected);
    let settled=round(&after);assert_eq!(settled.state,3);assert_eq!(&settled.settlement[2..8],&[net,paid_h,paid_t,paid_k,dust,paid]);assert_eq!(settled.settlement[14],c.n[1]);
    rollback(&bank,"integration-late-failure",vec![integrate(),failed_outer()],&mut nonce,Some(TOKEN));
    let(before,after)=success(&bank,"pending-integration",vec![integrate()],&mut nonce);let mut expected=before.clone();let floor=g.rent.minimum_balance(0);let physical=before[&target(9)].lamports()-floor;let escrow=before[&target(12)].lamports()-floor;
    assert_eq!(physical,SOL/5);debit(&mut expected,target(9),physical);debit(&mut expected,target(12),escrow);credit(&mut expected,target(10),physical+escrow);token_delta(&mut expected,target(15),-(SOL as i64/2));token_delta(&mut expected,target(14),SOL as i64/2);
    let mut c=config(&before);let(t,s,_)=pool_ratio(&before[&JITO_POOL]);let contribution=c.n[5]+value(c.n[4],t,s);c.n[1]+=contribution;c.n[2]=units(&expected[&target(14)]);c.n[3]=expected[&target(10)].lamports()-floor-c.n[6];c.n[4]=0;c.n[5]=0;c.n[9]+=contribution;assert!(c.n[3]+value(c.n[2],t,s)>=c.n[1]);write_config(&mut expected,c.clone());
    let mut idle=w.state_bytes()[1].clone();idle[22]=1;let summary=[0,bank.clock().unix_timestamp as u64,gross,paid,contribution,c.n[1],target_q,2,10_000,paid_k,0];for(i,n)in summary.into_iter().enumerate(){put64(&mut idle,23+i*8,n);}bytes(&mut expected,target(1),idle);expect_fee(&mut expected,&before,&after);assert_eq!(after,expected);

    assert_eq!(bank.epoch(),14);let c=config(&after);let(t,s,_)=pool_ratio(&after[&JITO_POOL]);
    assert_eq!((c.n[1],c.n[3],c.n[6],c.n[18]),(121_955_000_003,200_000_002,10_000,10_000));
    assert_eq!(c.n[3]+value(c.n[2],t,s),c.n[1]);assert_eq!(after[&target(10)].lamports()-floor,200_010_002);
    emit(json!({"kind":"cooldown_reward_retained","next_yield":10_000,"cumulative_reward":10_000,"historical_value":121_955_000_003_u64,
        "hwm":121_955_000_003_u64,"historical_sol":200_000_002,"principal_sol":200_010_002,"paid_htfp":paid_h,"paid_team":paid_t,"paid_kif":paid_k,"full_account_oracle":true}));
    cycle_clock(&bank,"reward-next-cycle-time",2_592_000);assert_eq!(u64at(&c.tail,0),0);
    cycle_activity(&bank,&w,0,1,"reward-next-heartbeat",&mut nonce);
    rollback(&bank,"reward-next-prepare-late-failure",vec![cycle_prepare(),failed_outer()],&mut nonce,Some(PROGRAM));
    let(before,after)=success(&bank,"reward-next-prepare",vec![cycle_prepare()],&mut nonce);let mut expected=before.clone();
    debit(&mut expected,target(10),8050);credit(&mut expected,target(12),8050);let mut c=config(&before);c.prepared=Some(2_592_000);c.n[0]=2;c.n[6]=0;
    write_config(&mut expected,c);bytes(&mut expected,target(1),reward_next_header(&w,&before,&bank));
    expect_fee(&mut expected,&before,&after);assert_eq!(after,expected);
    emit(json!({"kind":"cooldown_reward_later_split","gross":10_000,"historical_surplus":0,"prior_next_yield":10_000,"prior_yield_used":8050,
        "htfp":5900,"compound":1950,"team":1950,"kif":200,"dust":0,"active_count":1,"period":1,"new_legs":0,"full_account_oracle":true}));
    let(before,after)=success(&bank,"reward-next-settlement",vec![settle(&w)],&mut nonce);let mut expected=before.clone();
    debit(&mut expected,target(12),8050);credit(&mut expected,w.recipients[0],5900);credit(&mut expected,w.recipients[1],1950);credit(&mut expected,target(13),200);
    let mut c=config(&before);c.n[1]+=1950;c.n[7]+=200;c.n[10]+=10_000;c.n[11]+=5900;c.n[12]+=1950;c.n[13]+=200;c.n[15]+=1950;
    write_config(&mut expected,c.clone());let mut reward=before[&target(3)].data().to_vec();put64(&mut reward,60,paid_k+200);put64(&mut reward,68,paid_k+200);bytes(&mut expected,target(3),reward);
    let mut header=before[&target(1)].data().to_vec();header[11]=3;header[13]=1;put64(&mut header,241+88+15*8,0);
    for(j,n)in [1950,121_955_001_953,8050,5900,1950,200,0,8050,0,0,200,0,0,1950,121_955_001_953].into_iter().enumerate(){put64(&mut header,683+88+j*8,n);}
    bytes(&mut expected,target(1),header);expect_fee(&mut expected,&before,&after);assert_eq!(after,expected);
    let(before,after)=success(&bank,"reward-next-integration",vec![integrate()],&mut nonce);let mut expected=before.clone();
    let mut c=config(&before);c.n[3]+=1950;write_config(&mut expected,c);
    let mut idle=w.state_bytes()[1].clone();idle[22]=1;for(i,n)in [1,2_592_000,10_000,8050,0,121_955_001_953,0,0,0,200,0].into_iter().enumerate(){put64(&mut idle,23+i*8,n);}
    bytes(&mut expected,target(1),idle);expect_fee(&mut expected,&before,&after);assert_eq!(after,expected);
    let c=config(&after);assert_eq!((c.n[1],c.n[3],c.n[6],c.n[18]),(121_955_001_953,200_001_952,0,10_000));
    assert_eq!(c.n[3]+value(c.n[2],t,s),c.n[1]);assert_eq!(round(&after).state,0);
    assert_eq!((c.n[4],c.n[5],c.n[7],c.n[8]),(0,0,216_363_836,0));
    assert_runtime_capabilities(&bank,"reward-final",&capability_accounts);assert_registered_cache(&bank,"reward-final");
    emit(json!({"kind":"cooldown_reward_complete","status":"PASS","transactions":nonce,"completed_rounds":2,"epoch":14,"legs":2,
        "synthetic_reward":10_000,"inflation_executed":false,"first_hwm":121_955_000_003_u64,"final_hwm":121_955_001_953_u64,
        "later_gross":10_000,"later_htfp":5900,"later_compound":1950,"later_team":1950,"later_kif":200,
        "next_yield":0,"cumulative_reward":10_000,"pending_sol":0,"pending_tokens":0,"kif_liability":216_363_836,"kif_carry":0,
        "recovered_stake_rent":4_565_760,"recovered_metadata_rent":5_442_720,"final_state":0,"synthetic_clock":true,
        "live_ready":false,"signature_verified":false,"actual_programs":["PIV1","System","Token8.0.0","SPLpool2.0.3","Stake5.1.0"]}));
}

// A-004 later compounding profile: the complete earlier source remains unchanged.
// Copy S's bounded economic sequence to retain its exact transaction/map prefix.
fn compound_deposit(amount:u64,minimum:u64)->Instruction{
    let mut accounts=base(&[0,3,7,10,14,16,17,18]);accounts.push(meta(withdraw(),false,false));
    let mut args=amount.to_le_bytes().to_vec();args.extend(minimum.to_le_bytes());piv(b"PIV1SP01",&args,accounts)
}
fn compound_reject(bank:&Bank,name:&str,amount:u64,minimum:u64,code:u32,nonce:&mut u32){
    let before=snapshot(bank,&format!("{name}-before"));let t=tx(bank,vec![compound_deposit(amount,minimum)],*nonce);*nonce+=1;
    let c=execute(bank,name,&t).unwrap();assert_eq!(c.status,Err(TransactionError::InstructionError(2,InstructionError::Custom(code))));
    assert!(c.inner_instructions.as_ref().unwrap().iter().all(|inner|inner.is_empty()));
    assert!(!c.log_messages.as_ref().unwrap().iter().any(|line|line.starts_with("Program data: ")));
    let after=snapshot(bank,&format!("{name}-after"));let mut expected=before;debit(&mut expected,key(242),c.fee_details.total_fee());assert_eq!(after,expected);
    assert!(matches!(execute(bank,&format!("{name}-replay"),&t),Err(TransactionError::AlreadyProcessed)));
    assert_eq!(snapshot(bank,&format!("{name}-replayed")),after);
    emit(json!({"kind":"compound_rejection","case":name,"amount":amount,"minimum":minimum,"error":code,
        "no_cpi":true,"no_events":true,"fee_only":true,"identical_replay_no_fee":true}));
}
fn compound_attempt(bank:&Bank,name:&str,late_outer:bool,nonce:&mut u32)->Accounts{
    let before=snapshot(bank,&format!("{name}-before"));let mut ixs=vec![compound_deposit(2_458_999_998,1_891_538_460)];
    if late_outer{ixs.push(failed_outer());}let t=tx(bank,ixs,*nonce);*nonce+=1;let c=execute(bank,name,&t).unwrap();
    assert_eq!(c.status,if late_outer{Err(TransactionError::InstructionError(3,InstructionError::InvalidInstructionData))}else{Ok(())});
    let logs=c.log_messages.as_ref().unwrap();assert!(logs.contains(&format!("Program {PROGRAM} success")));
    let mut event=Sha256::digest(b"event:PendingSolStaked")[..8].to_vec();event.extend(target(0).as_ref());
    for n in [2_458_999_998,1_891_538_460,27_159_000_000,27_159_000_000]{append(&mut event,n);}assert_eq!(event.len(),72);
    let events=logs.iter().filter(|line|line.starts_with("Program data: ")).collect::<Vec<_>>();
    assert_eq!(events.len(),1);assert_eq!(events[0],&format!("Program data: {}",insuff_base64(&event)));
    let calls=c.inner_instructions.as_ref().unwrap().iter().flatten().filter(|call|
        t.message.account_keys[call.instruction.program_id_index as usize]==JITO_PROGRAM).collect::<Vec<_>>();
    assert_eq!(calls.len(),1);let ix=&calls[0].instruction;let mut data=vec![25];append(&mut data,2_458_999_998);append(&mut data,1_891_538_460);assert_eq!(ix.data,data);
    let accounts=ix.accounts.iter().map(|i|t.message.account_keys[*i as usize]).collect::<Vec<_>>();
    assert_eq!(accounts,vec![JITO_POOL,withdraw(),key(202),target(10),target(14),key(203),key(204),JITO_MINT,system_program::id(),TOKEN]);
    let after=snapshot(bank,&format!("{name}-after"));let mut expected=before.clone();
    if !late_outer{
        debit(&mut expected,target(10),2_458_999_998);credit(&mut expected,key(202),2_458_999_998);token_delta(&mut expected,target(14),1_891_538_460);
        let mut pool=before[&JITO_POOL].data().to_vec();put64(&mut pool,258,1_302_458_999_998);put64(&mut pool,266,1_001_891_538_460);bytes(&mut expected,JITO_POOL,pool);
        let mut mint=before[&JITO_MINT].data().to_vec();put64(&mut mint,36,1_001_891_538_460);bytes(&mut expected,JITO_MINT,mint);
        let mut next=config(&before);next.n[2]=20_891_538_460;next.n[3]=2;write_config(&mut expected,next);
    }
    debit(&mut expected,key(242),c.fee_details.total_fee());assert_eq!(after,expected);
    assert_eq!(after[&target(1)],before[&target(1)],"entire completed summary and header unchanged");
    if late_outer{
        assert!(matches!(execute(bank,&format!("{name}-replay"),&t),Err(TransactionError::AlreadyProcessed)));
        assert_eq!(snapshot(bank,&format!("{name}-replayed")),after);
    }
    emit(json!({"kind":"compound_attempt","case":name,"late_outer":late_outer,"committed":!late_outer,
        "event_data_hex":hex(&event),"event_verified":true,"protected_cpi_verified":true,"full_account_oracle":true,
        "prior_summary_unchanged":true,"identical_replay_no_fee":late_outer}));after
}
#[test]
fn post_cycle_compounding_production_lifecycle(){
    assert_eq!(env::var("RAYON_NUM_THREADS").as_deref(),Ok("1"));let path=env::var("PIV1_BANK_ACCOUNTS_DIR").unwrap();let path=Path::new(&path);
    assert!(path.is_absolute());assert_eq!(path.parent().unwrap().canonicalize().unwrap(),path.parent().unwrap());assert!(!path.exists());fs::create_dir(path).unwrap();
    let(w,extras)=external_world();let g=genesis(&w,extras);let leader=SlotLeader{id:key(246),vote_address:key(247)};
    let root=Bank::new_with_paths_for_tests(&g,Some(bounded()),vec![path.to_path_buf()],Some(leader));
    let(root,forks)=root.wrap_with_bank_forks_for_tests();let bank=Bank::new_from_parent_with_bank_forks(&forks,root,leader,1);let mut nonce=0;
    let mut capability_accounts=Accounts::new();for feature in REQUIRED_FEATURES{capability_accounts.insert(feature,AccountSharedData::from(g.accounts[&feature].clone()));}
    for k in [STAKE,program_data(STAKE)]{capability_accounts.insert(k,AccountSharedData::from(g.accounts[&k].clone()));}
    assert_runtime_capabilities(&bank,"cycles-initial",&capability_accounts);
    emit(json!({"kind":"fixture","profile":"post-cycle-compounding","genesis_hash":g.hash().to_string(),"synthetic_external_pool":true,"synthetic_governance":true,
        "synthetic_clock":true,"heap_bytes":HEAP,"unsigned":true,"economic_piv_seeded":false}));
    let initial=snapshot(&bank,"cycles-initial");for i in 0..16{assert!(!initial.contains_key(&target(i)));}
    let(before,after)=success(&bank,"cycles-initialize",vec![w.outer.clone()],&mut nonce);let mut expected=before.clone();let states=w.state_bytes();
    for i in 0..16{let d=if i<9{states[i].clone()}else if i<14{vec![]}else{w.token_bytes()};expected.insert(target(i),account(World::target_owner(i),w.target_balance(i),d));}
    debit(&mut expected,w.roles[w.payer].key,w.shortfall());expect_fee(&mut expected,&before,&after);assert_eq!(after,expected);
    assert_eq!(u64at(&config(&after).tail,0),0,"fixed KIF anchor");
    let(before,after)=success(&bank,"cycles-intake",vec![deposit_sol(SOL),deposit_token(19*SOL)],&mut nonce);let mut expected=before.clone();
    debit(&mut expected,key(220),SOL);credit(&mut expected,target(9),SOL);token_delta(&mut expected,key(221),-(19*SOL as i64));token_delta(&mut expected,target(15),19*SOL as i64);
    let mut c=config(&before);c.n[4]=19*SOL;c.n[5]=SOL;write_config(&mut expected,c);expect_fee(&mut expected,&before,&after);assert_eq!(after,expected);
    let(before,after)=success(&bank,"cycles-bootstrap",vec![bootstrap()],&mut nonce);let mut expected=before.clone();
    debit(&mut expected,target(9),SOL);credit(&mut expected,target(10),SOL);token_delta(&mut expected,target(15),-(19*SOL as i64));token_delta(&mut expected,target(14),19*SOL as i64);
    let mut c=config(&before);c.n[1]=20*SOL;c.n[2]=19*SOL;c.n[3]=SOL;c.n[4]=0;c.n[5]=0;c.n[9]=20*SOL;
    write_config(&mut expected,c);expect_fee(&mut expected,&before,&after);assert_eq!(after,expected);
    let floor=g.rent.minimum_balance(0);let mut final_accounts=after;
    for cycle in 0..3_u64{
        let prefix=format!("cycle-{cycle}");let timestamp=[1_728_000,2_592_000,5_184_000][cycle as usize];
        let(before,after)=success(&bank,&format!("{prefix}-yield-and-pending"),vec![solana_system_interface::instruction::transfer(&key(220),&key(202),100*SOL),update_pool(),deposit_sol(2*SOL)],&mut nonce);
        let mut expected=before.clone();debit(&mut expected,key(220),102*SOL);credit(&mut expected,key(202),100*SOL);credit(&mut expected,target(9),2*SOL);
        let(t,s,e)=pool_ratio(&before[&JITO_POOL]);assert_eq!((s,e),(1000*SOL,0));let mut pool=before[&JITO_POOL].data().to_vec();put64(&mut pool,258,t+100*SOL);bytes(&mut expected,JITO_POOL,pool);
        let mut c=config(&before);c.n[5]+=2*SOL;write_config(&mut expected,c);expect_fee(&mut expected,&before,&after);assert_eq!(after,expected);
        if cycle==1{
            cycle_clock(&bank,"cycle-cadence-clock-before",timestamp-1);
            cycle_reject(&bank,"cycle-cadence-before",cycle_prepare(),6038,&mut nonce);
        }
        cycle_clock(&bank,&format!("{prefix}-time"),timestamp);
        if cycle==2{for index in 0..3{cycle_activity(&bank,&w,index,2,&format!("{prefix}-heartbeat-{index}"),&mut nonce);}}
        if cycle==0{rollback(&bank,"cycle-prepare-late-failure",vec![cycle_prepare(),failed_outer()],&mut nonce,Some(PROGRAM));}
        let(before,after)=success(&bank,&format!("{prefix}-prepare"),vec![cycle_prepare()],&mut nonce);let mut expected=before.clone();
        debit(&mut expected,target(9),1_529_500_000);credit(&mut expected,target(12),1_529_500_000);
        let mut c=config(&before);c.prepared=Some(timestamp as u64);c.n[0]+=1;write_config(&mut expected,c);
        bytes(&mut expected,target(1),cycle_header(&w,&before,timestamp as u64,bank.slot(),0,cycle,if cycle==2{7}else{0},if cycle==2{3}else{0}));
        expect_fee(&mut expected,&before,&after);assert_eq!(after,expected);assert_eq!(round(&after).seq,cycle);
        if cycle==0{
            cycle_reject(&bank,"cycle-already-active",cycle_prepare(),6034,&mut nonce);
            // Activity after the frozen zero-active snapshot cannot earn retroactively.
            cycle_activity(&bank,&w,0,0,"cycle-late-heartbeat",&mut nonce);
            assert_cycle_cache(&bank,"cycles-after-first-heartbeat");
            cycle_clock(&bank,"cycle-delayed-settlement",timestamp+86_400);
        }
        if cycle==2{cycle_activity(&bank,&w,3,2,"cycle-active-late-heartbeat",&mut nonce);}
        if cycle==1{rollback(&bank,"cycle-settlement-late-failure",vec![settle(&w),failed_outer()],&mut nonce,Some(PROGRAM));}
        let(before,after)=success(&bank,&format!("{prefix}-settle"),vec![settle(&w)],&mut nonce);let mut expected=before.clone();
        let zero=[19_000_000,28_500_000,0][cycle as usize];let carry=[19_000_000,28_500_000,2][cycle as usize];let liability=if cycle==2{66_499_998}else{0};
        debit(&mut expected,target(12),1_529_500_000);credit(&mut expected,w.recipients[0],1_121_000_000);credit(&mut expected,w.recipients[1],370_500_000);
        credit(&mut expected,target(13),38_000_000);debit(&mut expected,target(13),zero);credit(&mut expected,target(10),zero);
        let mut c=config(&before);c.n[1]+=370_500_000+zero;c.n[7]+=liability;c.n[8]=carry;c.n[10]+=1_900_000_000;c.n[11]+=1_121_000_000;c.n[12]+=370_500_000;c.n[13]+=liability;c.n[15]+=370_500_000;c.n[17]+=zero;
        write_config(&mut expected,c);bytes(&mut expected,target(1),cycle_settlement_bytes(&before,zero,carry,liability));
        if cycle==2{for index in 0..3{let k=target(3+index);let mut b=before[&k].data().to_vec();put64(&mut b,60,22_166_666);put64(&mut b,68,22_166_666);bytes(&mut expected,k,b);}}
        expect_fee(&mut expected,&before,&after);assert_eq!(after,expected);
        assert_eq!(config(&after).prepared,Some(timestamp as u64),"settlement never restarts cadence");
        if cycle==2{rollback(&bank,"cycle-integration-late-failure",vec![integrate(),failed_outer()],&mut nonce,Some(PROGRAM));}
        let(before,after)=success(&bank,&format!("{prefix}-integrate"),vec![integrate()],&mut nonce);let mut expected=before.clone();
        assert_eq!(before[&target(9)].lamports()-floor,470_500_000);assert_eq!(before[&target(12)].lamports(),floor);
        debit(&mut expected,target(9),470_500_000);credit(&mut expected,target(10),470_500_000);
        let mut c=config(&before);c.n[1]+=2*SOL;c.n[3]=expected[&target(10)].lamports()-floor;c.n[5]=0;c.n[9]+=2*SOL;write_config(&mut expected,c.clone());
        let mut idle=w.state_bytes()[1].clone();idle[22]=1;
        for(i,n)in [cycle,bank.clock().unix_timestamp as u64,1_900_000_000,1_529_500_000,2*SOL,c.n[1],0,0,0,liability,carry].into_iter().enumerate(){put64(&mut idle,23+i*8,n);}
        bytes(&mut expected,target(1),idle);expect_fee(&mut expected,&before,&after);assert_eq!(after,expected);
        assert_eq!(c.n[1],[22_389_500_000,24_788_500_000,27_159_000_000][cycle as usize]);
        assert_eq!(c.n[3]+value(c.n[2],(1100+100*cycle)*SOL,1000*SOL),c.n[1]);
        assert_eq!(after[&target(13)].lamports()-floor,c.n[7]+c.n[8]);
        for index in 0..6{assert_eq!(u64at(after[&target(3+index)].data(),60),if cycle==2&&index<3{22_166_666}else{0});}
        emit(json!({"kind":"cycle_complete","sequence":cycle,"period":cycle,"prepared_at":timestamp,"completed_at":bank.clock().unix_timestamp,
            "active_count":if cycle==2{3}else{0},"gross":1_900_000_000,"kif_current":38_000_000,"kif_carry_input":([0,19_000_000,28_500_000][cycle as usize]),
            "kif_zero_compound":zero,"kif_carry_next":carry,"new_liability":liability,"hwm":c.n[1],"pending_sol":c.n[5],"pending_tokens":c.n[4],"full_account_oracle":true}));
        final_accounts=after;
    }
    let c=config(&final_accounts);assert_eq!(c.n,[3,27_159_000_000,19*SOL,2_459_000_000,0,0,0,66_499_998,2,26*SOL,5_700_000_000,3_363_000_000,1_111_500_000,66_499_998,0,1_111_500_000,0,47_500_000,0]);
    assert_eq!(c.prepared,Some(5_184_000));assert_eq!(c.insufficient,None);assert_eq!(round(&final_accounts).state,0);
    assert_eq!(final_accounts[&target(11)],account(system_program::id(),floor,vec![]));
    assert_runtime_capabilities(&bank,"cycles-final",&capability_accounts);assert_cycle_cache(&bank,"cycles-final");
    // Bind the actual completed terminal state before any later conversion.
    assert_eq!(pool_ratio(&final_accounts[&JITO_POOL]),(1_300_000_000_000,1_000_000_000_000,0));
    assert_eq!(u64at(final_accounts[&JITO_MINT].data(),36),1_000_000_000_000);
    assert_eq!(units(&final_accounts[&target(14)]),19_000_000_000);
    assert_eq!(final_accounts[&target(10)].lamports()-floor,2_459_000_000);
    let prior_summary=final_accounts[&target(1)].clone();assert_eq!(prior_summary.data()[22],1);assert_eq!(u64at(prior_summary.data(),23),2);
    assert_eq!(value(2_458_999_998,1_000_000_000_000,1_300_000_000_000),1_891_538_460);
    assert_eq!(value(20_891_538_460,1_302_458_999_998,1_001_891_538_460)+2,27_159_000_000);
    emit(json!({"kind":"compound_ready","completed_rounds":3,"prior_sequence":2,"next_sequence":3,
        "pool_total":1_300_000_000_000_u64,"pool_supply":1_000_000_000_000_u64,"historical_sol":2_459_000_000_u64,
        "historical_tokens":19_000_000_000_u64,"hwm":27_159_000_000_u64,"kif_liability":66_499_998,"kif_carry":2,"full_account_oracle":true}));
    compound_reject(&bank,"compound-minimum-reject",2_458_999_998,1_891_538_461,6032,&mut nonce);
    compound_reject(&bank,"compound-queue-reject",2_459_000_001,0,6028,&mut nonce);
    compound_attempt(&bank,"compound-late-failure",true,&mut nonce);
    let after=compound_attempt(&bank,"compound-success",false,&mut nonce);let c=config(&after);
    assert_eq!(c.n,[3,27_159_000_000,20_891_538_460,2,0,0,0,66_499_998,2,26*SOL,5_700_000_000,3_363_000_000,1_111_500_000,66_499_998,0,1_111_500_000,0,47_500_000,0]);
    assert_eq!(c.prepared,Some(5_184_000));assert_eq!(c.insufficient,None);assert_eq!(after[&target(1)],prior_summary);
    assert_eq!(after[&target(10)].lamports(),floor+2);assert_eq!(after[&target(13)].lamports(),floor+66_500_000);
    assert_eq!(c.n[3]+value(c.n[2],1_302_458_999_998,1_001_891_538_460),c.n[1]);
    assert_runtime_capabilities(&bank,"compound-final",&capability_accounts);assert_cycle_cache(&bank,"compound-final");
    emit(json!({"kind":"post_cycle_compounding_complete","status":"PASS","transactions":nonce,"completed_rounds":3,
        "prior_sequence":2,"next_sequence":3,"deposited_sol":2_458_999_998_u64,"minted_tokens":1_891_538_460,
        "historical_sol":c.n[3],"historical_tokens":c.n[2],"final_hwm":c.n[1],"historical_value":27_159_000_000_u64,
        "pool_total":1_302_458_999_998_u64,"pool_supply":1_001_891_538_460_u64,"kif_liability":c.n[7],"kif_carry":c.n[8],
        "pending_sol":0,"pending_tokens":0,"next_yield":0,"prior_summary_unchanged":true,"rent_unchanged":true,
        "final_state":0,"synthetic_clock":true,"real_walltime":false,"live_ready":false,"signature_verified":false,"stake_executed":false,
        "actual_programs":["PIV1","System","Token8.0.0","SPLpool2.0.3","synthetic-caller"],
        "loaded_programs":["PIV1","synthetic-caller","Token8.0.0","SPLpool2.0.3","Stake5.1.0"]}));
}
