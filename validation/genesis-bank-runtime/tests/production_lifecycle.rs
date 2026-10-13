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
use std::{collections::BTreeMap,env,fs,num::NonZeroUsize,path::Path,sync::Arc};
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
fn execute(bank:&Bank,name:&str,t:&Transaction)->TransactionCommitResult{
    assert!(t.signatures.iter().all(|s|s.as_ref().iter().all(|v|*v==0)));
    let fees=u64::from(t.message.header.num_required_signatures)*5000;
    emit(json!({"kind":"transaction","case":name,"slot":bank.slot(),"epoch":bank.epoch(),"message_hex":hex(&t.message.serialize()),"message_hash":t.message.hash().to_string(),
        "unsigned":true,"signatures":t.signatures.len(),"expected_fee":fees,"heap_bytes":HEAP,"packet_bytes":1+t.signatures.len()*64+t.message.serialize().len()}));
    let batch=bank.prepare_entry_batch(vec![t.clone().into()]).unwrap();let(mut results,_)=bank.load_execute_and_commit_transactions(&batch,ExecutionRecordingConfig::new_single_setting(true),&mut Default::default(),Some(512*1024));
    assert_eq!(results.len(),1);let result=results.remove(0);drop(batch);
    match &result{Ok(c)=>{assert_eq!(c.fee_details.total_fee(),fees);assert!(c.executed_units>0&&c.executed_units<=u64::from(LIMIT));
        emit(json!({"kind":"commit","case":name,"status":format!("{:?}",c.status),"fee":fees,"compute_units":c.executed_units,"logs":c.log_messages}));
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

#[test]
fn complete_delayed_production_lifecycle(){
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
    rollback(&bank,"active-finalization",vec![finalize(0)],&mut nonce,None);
    // Advance real Bank epochs/history. No PIV/external economic account writes.
    let operations_before=assigned[&target(11)].lamports();let mut finalized=0;
    // Bind the finite bound to this fixture's actual assigned delegation totals.
    assert_eq!([u64at(assigned[&stake(0)].data(),156),u64at(assigned[&stake(1)].data(),156)],[5_000_000_000,3_608_636_361]);
    assert_eq!(u64at(assigned[&source(0)].data(),156)+u64at(assigned[&source(1)].data(),156),3_391_363_639);
    for i in 0..2{assert_eq!(u64at(assigned[&stake(i)].data(),164),u64::MAX);assert_eq!(u64at(assigned[&stake(i)].data(),172),0);}
    for epoch in 1..=FINALIZATION_EPOCH_BOUND{
        let slot=bank.epoch_schedule().get_first_slot_in_epoch(epoch);let warped=Arc::new(Bank::warp_from_parent(bank,leader,slot));
        assert_runtime_capabilities(&warped,"epoch-warp",&capability_accounts);
        bank=Arc::new(Bank::new_from_parent(warped,leader,slot+1));assert_eq!(bank.epoch(),epoch);
        assert_runtime_capabilities(&bank,"epoch-child",&capability_accounts);
        let(before,after)=success(&bank,&format!("epoch-{epoch}-pool-update"),vec![update_validators(),update_pool()],&mut nonce);
        let mut expected=before.clone();let mut pool=before[&JITO_POOL].data().to_vec();let(t,s,_)=pool_ratio(&before[&JITO_POOL]);put64(&mut pool,274,epoch);put64(&mut pool,419,s);put64(&mut pool,427,t);bytes(&mut expected,JITO_POOL,pool);
        let mut list=before[&key(201)].data().to_vec();for i in 0..2{put64(&mut list,25+i*73,epoch);}bytes(&mut expected,key(201),list);expect_fee(&mut expected,&before,&after);assert_eq!(after,expected);
        if finalized==0{
            let before=snapshot(&bank,&format!("epoch-{epoch}-finalization-before"));let t=tx(&bank,vec![finalize(1)],nonce);nonce+=1;let c=execute(&bank,&format!("epoch-{epoch}-finalize-second"),&t).unwrap();let after=snapshot(&bank,&format!("epoch-{epoch}-finalization-after"));
            if c.status.is_err(){inactive_failure(&c,&t);let mut expected=before;debit(&mut expected,key(242),c.fee_details.total_fee());assert_eq!(after,expected);continue;}
            assert_finalization(&before,&after,1);assert_eq!(round(&after).cumulative[13],1);
        }
        pause_gate(&bank,"mature-finalization",&governance,4,finalize(0),&mut nonce);
        let(before,after)=success(&bank,"finalize-first",vec![finalize(0)],&mut nonce);assert_eq!(round(&after).cumulative[13],2);assert_eq!(round(&after).state,2);
        assert_finalization(&before,&after,0);
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
