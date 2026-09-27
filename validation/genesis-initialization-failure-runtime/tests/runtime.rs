//! Actual SBF execution through a synthetic caller, not Squads governance.
#[path = "../../genesis-initialization-runtime/tests/fixture.rs"]
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
    // The unreferenced sentinel is retained in Mollusk's returned vector but
    // excluded from its compiled message; runtime Instructions takes its place.
    let mut keys: BTreeSet<_>=expected.iter().filter(|a|a.0!=bridge(key(243))).map(|a|a.0).collect();
    assert_eq!(keys.len()+1,expected.len());assert!(keys.insert(instructions_id()));
    assert_eq!(actual.len(),keys.len());assert_eq!(actual.iter().map(|a|a.0).collect::<BTreeSet<_>>(),keys);
    for (key,account) in actual {
        if *key==instructions_id() {
            assert_eq!(account,&Account {lamports:0,data:instructions_bytes(instructions,current),
                owner:bridge(anchor_lang::solana_program::sysvar::ID),executable:false,rent_epoch:0});
        } else {assert_eq!(account,&expected.iter().find(|a|a.0==*key).expect("unexpected implicit account").1,"complete account {key}");}
    }
    assert_eq!(actual.iter().filter(|a|a.0==instructions_id()).count(),1);
}

#[derive(Clone,Copy,Debug)]
enum Boundary { Preflight, FirstToken, SecondToken, PrefundedSecondToken }
impl Boundary {
    fn limit(self, shared: bool) -> u64 {
        // Frozen before execution from Task 2.32 run-b success log entries.
        // The 1.4m ceiling minus the logged Token entry balance gives the
        // consumed prefix. One additional CU cannot reach Token serialization.
        match (self,shared) {
            (Self::Preflight,_)=>200_000,
            (Self::FirstToken,false)=>953_565, (Self::FirstToken,true)=>947_057,
            (Self::SecondToken,false)=>972_990, (Self::SecondToken,true)=>966_482,
            (Self::PrefundedSecondToken,false)=>893_968,
            (Self::PrefundedSecondToken,true)=>887_460,
        }
    }
    fn prefunded(self) -> bool { matches!(self,Self::PrefundedSecondToken) }
    fn completed_tokens(self) -> usize {
        usize::from(matches!(self,Self::SecondToken|Self::PrefundedSecondToken))
    }
}
fn prepared(shared: bool, boundary: Boundary) -> (World,Instruction,Accounts,Pubkey) {
    let mut w=World::new(shared,boundary.prefunded(),boundary.prefunded());
    for (index,id) in [(w.payer+1,bridge(fixture::system())),(w.payer+2,bridge(spl_token::ID))] {
        let loaded=if index==w.payer+1 {
            let (key,account)=program::keyed_account_for_system_program();assert_eq!(key,id);account
        } else {program::create_program_account_loader_v3(&id)};
        let target=&mut w.f.accounts[index];target.owner=anchor_lang::prelude::Pubkey::new_from_array(loaded.owner.to_bytes());
        target.data=loaded.data;target.lamports=loaded.lamports;target.executable=loaded.executable;
    }
    let old=&w.f.outer[0];let ix=Instruction {program_id:bridge(old.program_id),data:old.data.clone(),
        accounts:old.accounts.iter().map(|m|AccountMeta {pubkey:bridge(m.pubkey),is_signer:m.is_signer,is_writable:m.is_writable}).collect()};
    let mut accounts:Accounts=w.f.accounts.iter().filter(|a|bridge(a.key)!=instructions_id()).map(|a|(
        bridge(a.key),Account {lamports:a.lamports,data:a.data.clone(),owner:bridge(a.owner),executable:a.executable,rent_epoch:0})).collect();
    accounts.push((bridge(key(91)),Account {lamports:1_000_000,..Account::default()}));
    accounts.push((squads(),program::create_program_account_loader_v3(&squads())));
    let message_payer=bridge(key(242));accounts.push((message_payer,Account {lamports:1_000_000,..Account::default()}));
    accounts.push((bridge(key(243)),Account {lamports:73,data:vec![9,7,5],owner:bridge(key(244)),rent_epoch:17,..Account::default()}));
    assert_eq!(accounts.iter().map(|a|a.0).collect::<BTreeSet<_>>().len(),accounts.len());
    assert!(!accounts.iter().any(|a|a.0==instructions_id()));
    (w,ix,accounts,message_payer)
}
fn staged(w: &World, before: &Accounts, boundary: Boundary) -> Accounts {
    if matches!(boundary,Boundary::Preflight) { return before.clone(); }
    let mut after=expected_after(w,before);
    for slot in 0..9 {
        after.iter_mut().find(|a|a.0==bridge(w.f.accounts[7+slot].key)).unwrap().1.data=vec![0;SIZES[slot]];
    }
    for slot in 14+boundary.completed_tokens()..16 {
        after.iter_mut().find(|a|a.0==bridge(w.f.accounts[7+slot].key)).unwrap().1.data=vec![0;165];
    }
    assert_ne!(after,*before,"in-initializer failure must follow actual System effects");
    after
}
fn check_trace(w: &World, ix: &Instruction, observed: &Observation, completed: bool, boundary: Boundary) {
    let calls=expected_calls(w);
    let count=if completed {calls.len()} else if matches!(boundary,Boundary::Preflight) {0}
        else {calls.len()-2+boundary.completed_tokens()+1};
    assert_eq!(observed.trace.len(),2+count);
    let outer=&observed.trace[0];assert_eq!(outer.height,1);assert_eq!(outer.program,ix.program_id);assert_eq!(outer.data,ix.data);
    assert_eq!(outer.accounts,ix.accounts.iter().map(|m|(
        m.pubkey,ix.accounts.iter().any(|other|other.pubkey==m.pubkey&&other.is_signer),
        ix.accounts.iter().any(|other|other.pubkey==m.pubkey&&other.is_writable))).collect::<Vec<_>>());
    let inner=&observed.trace[1];assert_eq!(inner.program,bridge(PROGRAM));assert_eq!(inner.height,2);assert_eq!(inner.data,w.f.inner_data);
    assert_eq!(inner.accounts,w.f.accounts.iter().map(|a|(bridge(a.key),a.signer,a.writable)).collect::<Vec<_>>());
    assert!(!inner.accounts[PROPOSAL].2);assert!(inner.accounts[VAULT].1);assert!(inner.accounts[w.payer].1&&inner.accounts[w.payer].2);
    assert!(!outer.accounts.iter().find(|a|a.0==bridge(w.f.accounts[VAULT].key)).unwrap().1);
    assert!(outer.accounts.iter().find(|a|a.0==bridge(w.f.accounts[w.payer].key)).unwrap().1);
    assert!(outer.accounts.iter().find(|a|a.0==bridge(w.f.accounts[PROPOSAL].key)).unwrap().2);
    for (trace,expected) in observed.trace.iter().skip(2).zip(&calls[..count]) {
        assert_eq!(trace.height,3);assert_eq!(trace.program,bridge(expected.program));assert_eq!(trace.data,expected.data);
        assert_eq!(trace.accounts,expected.accounts.iter().map(|a|(bridge(a.0),a.1,a.2)).collect::<Vec<_>>());
    }
    let system=bridge(fixture::system());let token=bridge(spl_token::ID);
    let system_success=observed.logs.iter().filter(|line|**line==format!("Program {system} success")).count();
    let token_success=observed.logs.iter().filter(|line|**line==format!("Program {token} success")).count();
    assert_eq!(system_success,if completed || !matches!(boundary,Boundary::Preflight) {calls.len()-2} else {0});
    assert_eq!(token_success,if completed {2} else {boundary.completed_tokens()});
    if !completed && !matches!(boundary,Boundary::Preflight) {
        assert!(observed.logs.iter().any(|line|line==&format!("Program {token} consumed 1 of 1 compute units")));
    }
    if !completed {
        let failed=if matches!(boundary,Boundary::Preflight) {bridge(PROGRAM)} else {token};
        let cause=if matches!(boundary,Boundary::Preflight) {"Computational budget exceeded"}
            else {"exceeded CUs meter at BPF instruction"};
        assert!(observed.logs.iter().any(|line|line==&format!("Program {failed} failed: {cause}")),
            "VM failure must specifically originate from resource exhaustion, not panic or access violation");
    }
}
fn run_message(label: &str,w: &World,ix: &Instruction,accounts: &Accounts,payer: Pubkey,
    boundary: Boundary,success: bool) -> Accounts {
    let (mut vm,observation)=runtime();let limit=if success {LIMIT} else {boundary.limit(w.payer==29)};
    vm.compute_budget.compute_unit_limit=limit;
    assert_eq!(vm.sysvars.rent,solana_rent::Rent::default());
    for slot in 0..16 {assert_eq!(fixture::floor(slot),vm.sysvars.rent.minimum_balance(SIZES[slot]));}
    let before=accounts.clone();let clock=bincode::serialize(&vm.sysvars.clock).unwrap();let rent=bincode::serialize(&vm.sysvars.rent).unwrap();
    println!("SYSVARS case={label} clock={:?} rent={:?} budget={:?}",vm.sysvars.clock,vm.sysvars.rent,vm.compute_budget);
    println!("FUNDING case={label} payer={} shortfall={} sweep={}",w.f.accounts[w.payer].key,w.shortfall(),w.sweep());
    account_evidence(label,"supplied",accounts);
    let instructions=vec![ix.clone()];let result=vm.process_transaction_instructions(&instructions,accounts,Some(&payer));
    account_evidence(label,"returned",&result.resulting_accounts);let observed=observation.borrow();
    for (index,raw) in observed.before.iter().enumerate() {account_evidence(label,&format!("raw-before-{index}"),raw);}
    for (index,raw) in observed.after.iter().enumerate() {account_evidence(label,&format!("raw-after-{index}"),raw);}
    println!("CASE {label} result={:?} compute={} limit={limit} return={:?} TRACE={:?} LOGS={:?}",result.raw_result,result.compute_units_consumed,result.return_data,observed.trace,observed.logs);
    assert_eq!(accounts,&before);assert_eq!(bincode::serialize(&vm.sysvars.clock).unwrap(),clock);assert_eq!(bincode::serialize(&vm.sysvars.rent).unwrap(),rent);
    assert!(result.return_data.is_empty());
    // Checked syscall-meter exhaustion preserves its InstructionError; Token
    // VM instruction exhaustion maps to ProgramFailedToComplete instead.
    let failure=if matches!(boundary,Boundary::Preflight) {InstructionError::ComputationalBudgetExceeded}
        else {InstructionError::ProgramFailedToComplete};
    let expected=if success {Ok(())} else {Err(TransactionError::InstructionError(0,failure))};
    assert_eq!(result.raw_result,expected,"unexpected result must not be relabeled as a budget failure");
    if success {assert!(result.compute_units_consumed>200_000&&result.compute_units_consumed<LIMIT);}
    else {assert_eq!(result.compute_units_consumed,limit);}
    assert_eq!(observed.before.len(),1);assert_eq!(observed.after.len(),1);
    check_raw(&observed.before[0],&before,&instructions,0);
    let raw_expected=if success {expected_after(w,&before)} else {staged(w,&before,boundary)};
    check_raw(&observed.after[0],&raw_expected,&instructions,0);
    assert_eq!(result.resulting_accounts,if success {raw_expected} else {before});
    check_trace(w,ix,&observed,success,boundary);
    let inner:Vec<_>=result.inner_instructions.iter().flatten().collect();
    assert_eq!(inner.len(),observed.trace.len()-1);let keys=result.message.as_ref().unwrap().account_keys();
    for (inner,trace) in inner.iter().zip(observed.trace.iter().skip(1)) {
        assert_eq!(inner.stack_height,Some(trace.height as u32));assert_eq!(inner.instruction.data,trace.data);
        assert_eq!(keys[inner.instruction.program_id_index as usize],trace.program);
        assert_eq!(inner.instruction.accounts.iter().map(|i|keys[*i as usize]).collect::<Vec<_>>(),trace.accounts.iter().map(|a|a.0).collect::<Vec<_>>());
    }
    if !success {println!("DISCARD case={label} initialized_tokens={} staged_effects={} returned_original=true bank_rollback_claim=false",boundary.completed_tokens(),!matches!(boundary,Boundary::Preflight));}
    result.resulting_accounts
}
fn exercise(label: &str,shared: bool,boundary: Boundary) {
    let (w,ix,original,payer)=prepared(shared,boundary);
    let returned=run_message(&format!("{label}-failure"),&w,&ix,&original,payer,boundary,false);
    assert_eq!(returned,original,"retry input must be actual returned originals");
    let retry=run_message(&format!("{label}-retry"),&w,&ix,&returned,payer,boundary,true);
    assert_eq!(retry,expected_after(&w,&original));
    println!("RETRY case={label} original_input=true fresh_runtime=true limit={LIMIT} success=true");
}
#[test] fn distinct_preflight_resource_failure_and_retry() {exercise("distinct-preflight",false,Boundary::Preflight);}
#[test] fn shared_preflight_resource_failure_and_retry() {exercise("shared-preflight",true,Boundary::Preflight);}
#[test] fn distinct_first_token_cpi_failure_and_retry() {exercise("distinct-first-token",false,Boundary::FirstToken);}
#[test] fn shared_first_token_cpi_failure_and_retry() {exercise("shared-first-token",true,Boundary::FirstToken);}
#[test] fn distinct_second_token_cpi_failure_and_retry() {exercise("distinct-second-token",false,Boundary::SecondToken);}
#[test] fn shared_second_token_cpi_failure_and_retry() {exercise("shared-second-token",true,Boundary::SecondToken);}
#[test] fn distinct_prefunded_second_token_cpi_failure_and_retry() {exercise("distinct-prefund-second",false,Boundary::PrefundedSecondToken);}
#[test] fn shared_prefunded_second_token_cpi_failure_and_retry() {exercise("shared-prefund-second",true,Boundary::PrefundedSecondToken);}
