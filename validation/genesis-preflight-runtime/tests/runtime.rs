//! Actual SBF execution through a synthetic caller, not Squads governance.
mod fixture;
// Preserve the unchanged upstream test oracle's crate-root module path.
pub use piv1::integrations;
use fixture::{World, support::{key, PROGRAM, PROPOSAL, VAULT, INSTRUCTIONS}};
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
const REJECTED: u32 = 0x2291;
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
    for (role,id) in [("CALLER",squads()),("CALLEE",bridge(PROGRAM))] {
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
    (vm,observation)
}
#[derive(Clone,Copy)]
enum Case { Success, Approval, Stale, Clock, RecipientOwner, RecipientFunds, RecipientPda, Mint, Instructions, Direct, Outer }
fn execute(label: &str, shared: bool, case: Case) {
    let mut w=World::new(shared);let (mut vm,observation)=runtime();
    let recipient=if shared {29}else{30};
    for index in [recipient,recipient+1] {
        assert_eq!(w.f.accounts[index].lamports,vm.sysvars.rent.minimum_balance(0));
    }
    match case {
        Case::Approval=>{w.f.proposal.approved.pop();w.f.sync_proposal();},
        Case::Stale=>{w.f.multisig.stale_transaction_index=w.f.transaction.index;w.f.sync_multisig();},
        Case::Clock=>vm.sysvars.clock.unix_timestamp=99,
        Case::RecipientOwner=>w.f.accounts[recipient].owner=PROGRAM,
        Case::RecipientFunds=>w.f.accounts[recipient].lamports-=1,
        Case::RecipientPda=>{
            w.parameters.htfp_recipient=key(240);w.f.accounts[recipient].key=key(240);
            w.f.inner_data=w.parameters.encode().unwrap().to_vec();w.f.rebuild_message();
        },
        Case::Mint=>w.f.accounts[27].data[44]=8,
        Case::Instructions=>{w.f.accounts[INSTRUCTIONS].key=key(241);w.f.rebuild_message();},
        Case::Outer=>w.f.outer[0].data[0]^=1,
        Case::Success|Case::Direct=>{},
    }
    assert_eq!(w.f.accounts.len(),if shared {31}else{32});
    let direct=matches!(case,Case::Direct);
    let ix=if direct {Instruction {program_id:bridge(PROGRAM),data:w.f.inner_data.clone(),
        accounts:w.f.accounts.iter().map(|a|AccountMeta {pubkey:bridge(a.key),is_signer:a.signer,is_writable:a.writable}).collect()}}
        else {let old=&w.f.outer[0];Instruction {program_id:bridge(old.program_id),data:old.data.clone(),
            accounts:old.accounts.iter().map(|m|AccountMeta {pubkey:bridge(m.pubkey),is_signer:m.is_signer,is_writable:m.is_writable}).collect()}};
    // Never provide the fixture's host-constructed Instructions bytes. Mollusk
    // synthesizes the actual transaction's account when its canonical key appears.
    let mut accounts:Accounts=w.f.accounts.iter().filter(|a|bridge(a.key)!=instructions_id()).map(|a|(
        bridge(a.key),Account {lamports:a.lamports,data:a.data.clone(),owner:bridge(a.owner),executable:a.executable,rent_epoch:0})).collect();
    accounts.push((bridge(key(91)),Account {lamports:1_000_000,..Account::default()}));
    accounts.push((squads(),program::create_program_account_loader_v3(&squads())));
    let payer=bridge(key(242));accounts.push((payer,Account {lamports:1_000_000,..Account::default()}));
    let sentinel=bridge(key(243));accounts.push((sentinel,Account {lamports:73,data:vec![9,7,5],owner:bridge(key(244)),rent_epoch:17,..Account::default()}));
    assert_eq!(accounts.iter().map(|a|a.0).collect::<BTreeSet<_>>().len(),accounts.len());
    assert!(!accounts.iter().any(|a|a.0==instructions_id()));
    let before=accounts.clone();let clock_before=bincode::serialize(&vm.sysvars.clock).unwrap();
    let rent_before=bincode::serialize(&vm.sysvars.rent).unwrap();
    println!("SYSVARS case={label} clock={:?} rent={:?} budget={:?}",vm.sysvars.clock,vm.sysvars.rent,vm.compute_budget);
    account_evidence(label,"supplied",&accounts);
    let result=vm.process_transaction_instructions(&[ix.clone()],&accounts,Some(&payer));
    account_evidence(label,"returned",&result.resulting_accounts);
    let observed=observation.borrow();
    assert_eq!(observed.before.len(),1);assert_eq!(observed.after.len(),1);
    account_evidence(label,"raw-before",&observed.before[0]);account_evidence(label,"raw-after",&observed.after[0]);
    println!("CASE {label} result={:?} compute={} return={:?} TRACE={:?} LOGS={:?}",result.raw_result,result.compute_units_consumed,result.return_data,observed.trace,observed.logs);
    assert_eq!(accounts,before,"host fixture mutated");
    assert_eq!(result.resulting_accounts,before,"returned complete accounts changed");
    assert_eq!(observed.after,observed.before,"raw complete account state changed");
    for (key,actual) in &observed.before[0] {
        if *key==instructions_id() {
            assert_eq!(actual.owner,bridge(anchor_lang::solana_program::sysvar::ID));
            assert!(!actual.executable);assert_eq!(actual.lamports,0);assert_eq!(actual.rent_epoch,0);assert_eq!(&actual.data[actual.data.len()-2..],&0u16.to_le_bytes());
            // Independent legacy SDK constructor serializes the exact actual outer metas.
            use anchor_lang::solana_program::sysvar::instructions::{BorrowedAccountMeta,BorrowedInstruction,construct_instructions_data};
            let old_keys:Vec<_>=ix.accounts.iter().map(|m|anchor_lang::prelude::Pubkey::new_from_array(m.pubkey.to_bytes())).collect();
            let old_program=anchor_lang::prelude::Pubkey::new_from_array(ix.program_id.to_bytes());
            let borrowed=BorrowedInstruction {program_id:&old_program,data:&ix.data,accounts:ix.accounts.iter().zip(&old_keys).map(|(m,k)|BorrowedAccountMeta {pubkey:k,is_signer:m.is_signer,is_writable:m.is_writable}).collect()};
            assert_eq!(actual.data,construct_instructions_data(&[borrowed]));
        } else {assert_eq!(actual,&before.iter().find(|a|a.0==*key).expect("unexpected implicit account").1);}
    }
    assert_eq!(observed.before[0].iter().filter(|a|a.0==instructions_id()).count(),usize::from(!matches!(case,Case::Instructions)));
    assert_eq!(bincode::serialize(&vm.sysvars.clock).unwrap(),clock_before);
    assert_eq!(bincode::serialize(&vm.sysvars.rent).unwrap(),rent_before);
    assert!(result.return_data.is_empty());assert!(result.compute_units_consumed>0&&result.compute_units_consumed<=LIMIT);
    let expected=match case {Case::Success=>Ok(()),Case::Outer=>Err(TransactionError::InstructionError(0,InstructionError::InvalidInstructionData)),_=>Err(TransactionError::InstructionError(0,InstructionError::Custom(REJECTED)))};
    assert_eq!(result.raw_result,expected);
    let has_cpi=!direct&&!matches!(case,Case::Outer);
    assert_eq!(observed.trace.len(),if has_cpi {2}else{1});
    assert_eq!(observed.trace[0].height,1);assert_eq!(observed.trace[0].program,ix.program_id);assert_eq!(observed.trace[0].data,ix.data);
    assert_eq!(observed.trace[0].accounts,ix.accounts.iter().map(|m|(
        m.pubkey, ix.accounts.iter().any(|other|other.pubkey==m.pubkey&&other.is_signer),
        ix.accounts.iter().any(|other|other.pubkey==m.pubkey&&other.is_writable))).collect::<Vec<_>>());
    let inner:Vec<_>=result.inner_instructions.iter().flatten().collect();assert_eq!(inner.len(),usize::from(has_cpi));
    if has_cpi {
        let trace=&observed.trace[1];assert_eq!(trace.program,bridge(PROGRAM));assert_eq!(trace.height,2);
        assert_eq!(trace.data,w.f.inner_data);
        assert_eq!(trace.accounts,w.f.accounts.iter().map(|a|(bridge(a.key),a.signer,a.writable)).collect::<Vec<_>>());
        assert!(!trace.accounts[PROPOSAL].2);assert!(trace.accounts[VAULT].1);
        let outer_vault=observed.trace[0].accounts.iter().find(|a|a.0==bridge(w.f.accounts[VAULT].key)).unwrap();assert!(!outer_vault.1);
        assert!(observed.trace[0].accounts.iter().find(|a|a.0==bridge(w.f.accounts[PROPOSAL].key)).unwrap().2);
        assert_eq!(inner[0].stack_height,Some(2));assert_eq!(inner[0].instruction.data,w.f.inner_data);
        let message=result.message.as_ref().unwrap();let keys=message.account_keys();
        assert_eq!(keys[inner[0].instruction.program_id_index as usize],bridge(PROGRAM));
        assert_eq!(inner[0].instruction.accounts.iter().map(|i|keys[*i as usize]).collect::<Vec<_>>(),w.f.accounts.iter().map(|a|bridge(a.key)).collect::<Vec<_>>());
    }
}
#[test] fn distinct_fee_receivers_preflight() {execute("distinct-success",false,Case::Success);}
#[test] fn shared_receiver_preflight() {execute("shared-success",true,Case::Success);}
#[test] fn insufficient_approval_rejected() {execute("approval",false,Case::Approval);}
#[test] fn stale_approval_rejected() {execute("stale",false,Case::Stale);}
#[test] fn runtime_clock_enforces_timelock() {execute("clock",false,Case::Clock);}
#[test] fn recipient_owner_rejected() {execute("recipient-owner",false,Case::RecipientOwner);}
#[test] fn recipient_rent_floor_rejected() {execute("recipient-funds",false,Case::RecipientFunds);}
#[test] fn approved_noncanonical_recipient_pda_rejected() {execute("recipient-pda",false,Case::RecipientPda);}
#[test] fn invalid_protocol_mint_rejected() {execute("mint",false,Case::Mint);}
#[test] fn wrong_instructions_identity_rejected() {execute("instructions",false,Case::Instructions);}
#[test] fn direct_height_one_rejected() {execute("direct",false,Case::Direct);}
#[test] fn caller_rejects_wrong_outer_discriminator() {execute("outer",false,Case::Outer);}
