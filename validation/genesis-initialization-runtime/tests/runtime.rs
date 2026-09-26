//! Actual SBF execution through a synthetic caller, not Squads governance.
mod fixture;
// Preserve the unchanged upstream test oracle's crate-root module path.
pub use piv1::integrations;
use fixture::{World, SIZES, expected_calls, support::{key, PROGRAM, PROPOSAL, VAULT}};
use std::{cell::RefCell, collections::BTreeSet, rc::Rc};
use mollusk_svm::{InvocationInspectCallback, Mollusk, program};
use solana_account::{Account, ReadableAccount};
use solana_instruction::{AccountMeta, Instruction};
use solana_pubkey::Pubkey;
use solana_program_runtime::{invoke_context::InvokeContext, program_cache_entry::ProgramCacheEntryType};
use solana_transaction_context::instruction_accounts::InstructionAccount;
use solana_instruction_error::InstructionError;
use solana_transaction_error::TransactionError;
use solana_svm_log_collector::LogCollector;
use sha2::{Digest, Sha256};
const LIMIT: u64 = 1_400_000;
const REJECTED: u32 = 0x2311;
type Accounts = Vec<(Pubkey, Account)>;
fn bridge(key: anchor_lang::prelude::Pubkey) -> Pubkey { Pubkey::new_from_array(key.to_bytes()) }
fn squads() -> Pubkey { bridge(piv1::squads_accounts::SQUADS_V4_PROGRAM_ID) }
fn instructions_id() -> Pubkey { bridge(anchor_lang::solana_program::sysvar::instructions::ID) }
fn snapshot(context: &InvokeContext) -> Accounts {
    let tx = &context.transaction_context;
    (0..tx.get_number_of_accounts()).map(|i| {
        let a = tx.accounts().try_borrow(i).unwrap();
        (*tx.get_key_of_account_at_index(i).unwrap(), Account { lamports: a.lamports(), data: a.data().to_vec(),
            owner: *a.owner(), executable: a.executable(), rent_epoch: a.rent_epoch() })
    }).collect()
}
#[derive(Debug)]
struct Trace { program: Pubkey, height: usize, data: Vec<u8>, accounts: Vec<(Pubkey,bool,bool)> }
#[derive(Default)]
struct Observation { before: Vec<Accounts>, after: Vec<Accounts>, trace: Vec<Trace>, logs: Vec<String> }
struct Observer(Rc<RefCell<Observation>>);
impl InvocationInspectCallback for Observer {
    fn before_invocation(&self, _: &Mollusk, _: &Pubkey, _: &[u8], _: &[InstructionAccount],
        context: &mut InvokeContext, tracing: bool) {
        assert!(!tracing); self.0.borrow_mut().before.push(snapshot(context));
    }
    fn after_invocation(&self, vm: &Mollusk, context: &InvokeContext, tracing: bool) {
        assert!(!tracing);
        let mut observation = self.0.borrow_mut(); observation.after.push(snapshot(context));
        let tx = &context.transaction_context;
        observation.trace = (0..tx.get_instruction_trace_length()).map(|i| {
            let ix = tx.get_instruction_context_at_index_in_trace(i).unwrap();
            Trace { program: *tx.get_key_of_account_at_index(ix.get_index_of_program_account_in_transaction().unwrap()).unwrap(),
                height: ix.get_stack_height(), data: ix.get_instruction_data().to_vec(),
                accounts: (0..ix.get_number_of_instruction_accounts()).map(|j| (
                    *tx.get_key_of_account_at_index(ix.get_index_of_instruction_account_in_transaction(j).unwrap()).unwrap(),
                    ix.is_instruction_account_signer(j).unwrap(), ix.is_instruction_account_writable(j).unwrap())).collect() }
        }).collect();
        observation.logs = vm.logger.as_ref().unwrap().borrow().get_recorded_content().to_vec();
    }
}
fn account_evidence(label: &str, stage: &str, accounts: &Accounts) {
    use std::fmt::Write;
    println!("ACCOUNTS case={label} stage={stage} count={}",accounts.len());
    for (index,(key,a)) in accounts.iter().enumerate() {
        let mut bytes=String::new(); for byte in &a.data {write!(&mut bytes,"{byte:02x}").unwrap();}
        println!("ACCOUNT case={label} stage={stage} index={index} key={key} lamports={} owner={} executable={} rent_epoch={} data_len={} data_hex={bytes}",a.lamports,a.owner,a.executable,a.rent_epoch,a.data.len());
    }
}
fn runtime() -> (Mollusk,Rc<RefCell<Observation>>) {
    let mut vm=Mollusk::default();
    assert_eq!(vm.compute_budget,solana_compute_budget::compute_budget::ComputeBudget::new_with_defaults(true));
    assert_eq!(vm.compute_budget.compute_unit_limit,LIMIT);
    assert_eq!(vm.compute_budget.heap_size,32768);assert_eq!(vm.compute_budget.stack_frame_size,4096);
    vm.sysvars.clock.unix_timestamp=100;
    let observation=Rc::new(RefCell::new(Observation::default()));
    vm.invocation_inspect_callback=Box::new(Observer(observation.clone()));
    vm.logger=Some(LogCollector::new_ref_with_limit(None));
    for (role,id) in [("CALLER",squads()),("CALLEE",bridge(PROGRAM)),("TOKEN",bridge(spl_token::ID))] {
        let path=std::env::var(format!("PIV_GENESIS_{role}_PATH")).unwrap();
        let expected=std::env::var(format!("PIV_GENESIS_{role}_SHA256")).unwrap();
        let length=std::env::var(format!("PIV_GENESIS_{role}_BYTES")).unwrap().parse::<usize>().unwrap();
        let bytes=std::fs::read(&path).unwrap(); assert_eq!(bytes.len(),length);
        assert_eq!(format!("{:x}",Sha256::digest(&bytes)),expected);
        vm.add_program_with_loader_and_elf(&id,&program::loader_keys::LOADER_V3,&bytes);
        assert_eq!(vm.program_cache.get_program_elf_bytes(&id).unwrap(),bytes);
        let entry=vm.program_cache.load_program(&id).unwrap();
        let ProgramCacheEntryType::Loaded(executable)=&entry.program else {panic!("probe must be a loaded ELF")};
        assert_eq!(format!("{:?}",executable.get_sbpf_version()),"V0");
        assert_eq!(executable.get_config().stack_frame_size,4096);
        println!("ARTIFACT role={role} key={id} sha256={expected} bytes={length} config={:?}",executable.get_config());
    }
    let system_entry=vm.program_cache.load_program(&bridge(fixture::system())).unwrap();
    assert!(matches!(system_entry.program,ProgramCacheEntryType::Builtin(_)),"actual System builtin required");
    (vm,observation)
}
#[derive(Clone,Copy)]
enum Case { Success, Prefund, LateFailure, Approval, Clock, Recipient, Mint, PayerFunds, PayerSigner, Outer, Direct }
fn instructions_bytes(instructions: &[Instruction], current: u16) -> Vec<u8> {
    use anchor_lang::solana_program::sysvar::instructions::{BorrowedAccountMeta,BorrowedInstruction,construct_instructions_data};
    let old:Vec<_>=instructions.iter().map(|ix| anchor_lang::solana_program::instruction::Instruction {
        program_id:anchor_lang::prelude::Pubkey::new_from_array(ix.program_id.to_bytes()),data:ix.data.clone(),
        accounts:ix.accounts.iter().map(|m|anchor_lang::solana_program::instruction::AccountMeta {
            pubkey:anchor_lang::prelude::Pubkey::new_from_array(m.pubkey.to_bytes()),is_signer:m.is_signer,is_writable:m.is_writable}).collect()}).collect();
    let borrowed:Vec<_>=old.iter().map(|ix|BorrowedInstruction {program_id:&ix.program_id,data:&ix.data,
        accounts:ix.accounts.iter().map(|m|BorrowedAccountMeta {pubkey:&m.pubkey,is_signer:m.is_signer,is_writable:m.is_writable}).collect()}).collect();
    let mut bytes=construct_instructions_data(&borrowed);let end=bytes.len();bytes[end-2..].copy_from_slice(&current.to_le_bytes());bytes
}
fn expected_after(w: &World, before: &Accounts) -> Accounts {
    let states=w.state_bytes();let token=w.token_bytes();let mut after=before.clone();
    for (slot,size) in SIZES.into_iter().enumerate() {
        let account=&mut after.iter_mut().find(|a|a.0==bridge(w.f.accounts[7+slot].key)).unwrap().1;
        account.lamports=w.target_balance(slot);account.owner=bridge(w.target_owner(slot));
        account.data=if slot<9 {states[slot].clone()} else if slot<14 {vec![]} else {token.clone()};
        assert_eq!(account.data.len(),size);
    }
    after.iter_mut().find(|a|a.0==bridge(w.f.accounts[w.payer].key)).unwrap().1.lamports-=w.shortfall();
    assert_eq!(before.iter().map(|a|u128::from(a.1.lamports)).sum::<u128>(),after.iter().map(|a|u128::from(a.1.lamports)).sum::<u128>());
    after
}
fn check_raw(actual: &Accounts, expected: &Accounts, instructions: &[Instruction], current: u16) {
    for (key,account) in actual {
        if *key==instructions_id() {
            assert_eq!(account,&Account {lamports:0,data:instructions_bytes(instructions,current),
                owner:bridge(anchor_lang::solana_program::sysvar::ID),executable:false,rent_epoch:0});
        } else {assert_eq!(account,&expected.iter().find(|a|a.0==*key).expect("unexpected implicit account").1,"complete account {key}");}
    }
    assert_eq!(actual.iter().filter(|a|a.0==instructions_id()).count(),1);
}
fn execute(label: &str, shared: bool, case: Case) {
    let prefunded=matches!(case,Case::Prefund|Case::LateFailure);
    let mut w=World::new(shared,prefunded,prefunded);let (mut vm,observation)=runtime();
    assert_eq!(vm.sysvars.rent,solana_rent::Rent::default());
    for slot in 0..16 {assert_eq!(fixture::floor(slot),vm.sysvars.rent.minimum_balance(SIZES[slot]));}
    // Exact runtime program metadata, not arbitrary host program placeholders.
    for (index,id) in [(w.payer+1,bridge(fixture::system())),(w.payer+2,bridge(spl_token::ID))] {
        let loaded=if index==w.payer+1 {
            let (key,account)=program::keyed_account_for_system_program();assert_eq!(key,id);account
        } else {program::create_program_account_loader_v3(&id)};
        let target=&mut w.f.accounts[index];target.owner=anchor_lang::prelude::Pubkey::new_from_array(loaded.owner.to_bytes());
        target.data=loaded.data;target.lamports=loaded.lamports;target.executable=loaded.executable;
    }
    match case {
        Case::Approval=>{w.f.proposal.approved.pop();w.f.sync_proposal();},
        Case::Clock=>vm.sysvars.clock.unix_timestamp=99,
        Case::Recipient=>w.f.accounts[w.payer+3].lamports-=1,
        Case::Mint=>w.f.accounts[27].data[44]=8,
        Case::PayerFunds=>w.f.accounts[w.payer].lamports=w.shortfall()-1,
        Case::PayerSigner=>{for meta in &mut w.f.outer[0].accounts {if meta.pubkey==w.f.accounts[w.payer].key {meta.is_signer=false;}}},
        Case::Outer=>w.f.outer[0].data[0]^=1,
        _=>{},
    }
    let direct=matches!(case,Case::Direct);
    let ix=if direct {Instruction {program_id:bridge(PROGRAM),data:w.f.inner_data.clone(),
        accounts:w.f.accounts.iter().map(|a|AccountMeta {pubkey:bridge(a.key),is_signer:a.signer,is_writable:a.writable}).collect()}}
        else {let old=&w.f.outer[0];Instruction {program_id:bridge(old.program_id),data:old.data.clone(),
            accounts:old.accounts.iter().map(|m|AccountMeta {pubkey:bridge(m.pubkey),is_signer:m.is_signer,is_writable:m.is_writable}).collect()}};
    let late=matches!(case,Case::LateFailure);
    let mut instructions=vec![ix.clone()];if late {let mut bad=ix.clone();bad.data[0]^=1;instructions.push(bad);}
    let mut accounts:Accounts=w.f.accounts.iter().filter(|a|bridge(a.key)!=instructions_id()).map(|a|(
        bridge(a.key),Account {lamports:a.lamports,data:a.data.clone(),owner:bridge(a.owner),executable:a.executable,rent_epoch:0})).collect();
    accounts.push((bridge(key(91)),Account {lamports:1_000_000,..Account::default()}));
    accounts.push((squads(),program::create_program_account_loader_v3(&squads())));
    let message_payer=bridge(key(242));accounts.push((message_payer,Account {lamports:1_000_000,..Account::default()}));
    accounts.push((bridge(key(243)),Account {lamports:73,data:vec![9,7,5],owner:bridge(key(244)),rent_epoch:17,..Account::default()}));
    assert_eq!(accounts.iter().map(|a|a.0).collect::<BTreeSet<_>>().len(),accounts.len());
    assert!(!accounts.iter().any(|a|a.0==instructions_id()));
    let before=accounts.clone();let clock_before=bincode::serialize(&vm.sysvars.clock).unwrap();
    let rent_before=bincode::serialize(&vm.sysvars.rent).unwrap();
    println!("SYSVARS case={label} clock={:?} rent={:?} budget={:?}",vm.sysvars.clock,vm.sysvars.rent,vm.compute_budget);
    println!("FUNDING case={label} payer={} shortfall={} sweep={}",w.f.accounts[w.payer].key,w.shortfall(),w.sweep());
    account_evidence(label,"supplied",&accounts);
    let result=vm.process_transaction_instructions(&instructions,&accounts,Some(&message_payer));
    account_evidence(label,"returned",&result.resulting_accounts);
    let observed=observation.borrow();
    for (index,raw) in observed.before.iter().enumerate() {account_evidence(label,&format!("raw-before-{index}"),raw);}
    for (index,raw) in observed.after.iter().enumerate() {account_evidence(label,&format!("raw-after-{index}"),raw);}
    println!("CASE {label} result={:?} compute={} return={:?} TRACE={:?} LOGS={:?}",result.raw_result,result.compute_units_consumed,result.return_data,observed.trace,observed.logs);
    assert_eq!(accounts,before,"host fixture mutated");
    assert_eq!(bincode::serialize(&vm.sysvars.clock).unwrap(),clock_before);
    assert_eq!(bincode::serialize(&vm.sysvars.rent).unwrap(),rent_before);
    assert!(result.return_data.is_empty());assert!(result.compute_units_consumed>0&&result.compute_units_consumed<=LIMIT);
    let success=matches!(case,Case::Success|Case::Prefund);
    let expected=match case {
        Case::Success|Case::Prefund=>Ok(()),
        Case::Outer=>Err(TransactionError::InstructionError(0,InstructionError::InvalidInstructionData)),
        Case::PayerSigner=>Err(TransactionError::InstructionError(0,InstructionError::InvalidArgument)),
        Case::LateFailure=>Err(TransactionError::InstructionError(1,InstructionError::InvalidInstructionData)),
        _=>Err(TransactionError::InstructionError(0,InstructionError::Custom(REJECTED))),
    };
    assert_eq!(result.raw_result,expected,"initializer resource or validation failure must not be relabeled success");
    assert_eq!(observed.before.len(),if late {2}else{1});assert_eq!(observed.after.len(),observed.before.len());
    check_raw(&observed.before[0],&before,&instructions,0);
    let completes=success||late;
    let after=if completes {expected_after(&w,&before)} else {before.clone()};
    check_raw(&observed.after[0],&after,&instructions,0);
    if late {
        assert_ne!(after,before,"late failure requires real completed initialization effects");
        check_raw(&observed.before[1],&after,&instructions,0);
        check_raw(&observed.after[1],&after,&instructions,1);
        println!("DISCARD case={label} raw_initialized=true returned_original=true bank_rollback_claim=false");
    }
    assert_eq!(result.resulting_accounts,if success {after}else{before});
    let has_cpi=!direct&&!matches!(case,Case::Outer|Case::PayerSigner);
    let calls=if completes {expected_calls(&w)} else {vec![]};
    // Pinned Agave reserves all top-level trace slots first; CPIs are appended
    // after that prefix, even when they execute before the second outer action.
    let first_inner=instructions.len();
    assert_eq!(observed.trace.len(),first_inner+usize::from(has_cpi)+calls.len());
    for (trace,instruction) in observed.trace[..first_inner].iter().zip(&instructions) {
        assert_eq!(trace.height,1);assert_eq!(trace.program,instruction.program_id);assert_eq!(trace.data,instruction.data);
        assert_eq!(trace.accounts,instruction.accounts.iter().map(|m|(
            m.pubkey,instructions.iter().flat_map(|ix|&ix.accounts).any(|other|other.pubkey==m.pubkey&&other.is_signer),
            instructions.iter().flat_map(|ix|&ix.accounts).any(|other|other.pubkey==m.pubkey&&other.is_writable))).collect::<Vec<_>>());
    }
    let outer=&observed.trace[0];
    if has_cpi {
        let trace=&observed.trace[first_inner];assert_eq!(trace.program,bridge(PROGRAM));assert_eq!(trace.height,2);assert_eq!(trace.data,w.f.inner_data);
        assert_eq!(trace.accounts,w.f.accounts.iter().map(|a|(bridge(a.key),a.signer,a.writable)).collect::<Vec<_>>());
        assert!(!trace.accounts[PROPOSAL].2);assert!(trace.accounts[VAULT].1);assert!(trace.accounts[w.payer].1&&trace.accounts[w.payer].2);
        assert!(!outer.accounts.iter().find(|a|a.0==bridge(w.f.accounts[VAULT].key)).unwrap().1);
        assert!(outer.accounts.iter().find(|a|a.0==bridge(w.f.accounts[w.payer].key)).unwrap().1);
        assert!(outer.accounts.iter().find(|a|a.0==bridge(w.f.accounts[PROPOSAL].key)).unwrap().2);
    }
    for (trace,expected) in observed.trace.iter().skip(first_inner+1).zip(&calls) {
        assert_eq!(trace.height,3);assert_eq!(trace.program,bridge(expected.program));assert_eq!(trace.data,expected.data);
        assert_eq!(trace.accounts,expected.accounts.iter().map(|a|(bridge(a.0),a.1,a.2)).collect::<Vec<_>>());
    }
    if late {let rejected=&observed.trace[1];assert_eq!(rejected.height,1);assert_eq!(rejected.program,squads());assert_eq!(rejected.data,instructions[1].data);}
    let inner:Vec<_>=result.inner_instructions.iter().flatten().collect();
    let trace_inner:Vec<_>=observed.trace.iter().filter(|t|t.height>1).collect();assert_eq!(inner.len(),trace_inner.len());
    let message=result.message.as_ref().unwrap();let keys=message.account_keys();
    for (inner,trace) in inner.iter().zip(trace_inner) {
        assert_eq!(inner.stack_height,Some(trace.height as u32));assert_eq!(inner.instruction.data,trace.data);
        assert_eq!(keys[inner.instruction.program_id_index as usize],trace.program);
        assert_eq!(inner.instruction.accounts.iter().map(|i|keys[*i as usize]).collect::<Vec<_>>(),trace.accounts.iter().map(|a|a.0).collect::<Vec<_>>());
    }
}
#[test] fn distinct_fee_receivers_initialize() {execute("distinct-success",false,Case::Success);}
#[test] fn shared_receiver_initializes() {execute("shared-success",true,Case::Success);}
#[test] fn paused_prefunded_distinct_normalizes_without_recognition() {execute("distinct-prefund",false,Case::Prefund);}
#[test] fn paused_prefunded_shared_normalizes_without_recognition() {execute("shared-prefund",true,Case::Prefund);}
#[test] fn late_error_discards_returned_accounts_after_real_initialization() {execute("late-failure",false,Case::LateFailure);}
#[test] fn insufficient_approval_rejected() {execute("approval",false,Case::Approval);}
#[test] fn runtime_clock_enforces_timelock() {execute("clock",false,Case::Clock);}
#[test] fn recipient_rent_floor_rejected() {execute("recipient",false,Case::Recipient);}
#[test] fn invalid_protocol_mint_rejected() {execute("mint",false,Case::Mint);}
#[test] fn original_payer_rent_shortfall_enforced() {execute("payer-funds",false,Case::PayerFunds);}
#[test] fn actual_outer_payer_signer_required() {execute("payer-signer",false,Case::PayerSigner);}
#[test] fn direct_height_one_rejected() {execute("direct",false,Case::Direct);}
#[test] fn caller_rejects_wrong_outer_discriminator() {execute("outer",false,Case::Outer);}
