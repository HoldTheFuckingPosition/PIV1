//! Retained Testnet images, synthetic genesis multisig/ALT, unsigned local entry.
//! Success-only composition; no live operations, signature verification or M4 waiver.
#[allow(dead_code)]
#[path = "genesis_fixture.rs"]
mod fixture;

use fixture::{key, program_data, World, JITO_MINT, JITO_POOL, JITO_PROGRAM, PROGRAM, SIZES, SQUADS, TOKEN};
use serde_json::{json, Value};
use sha2::{Digest, Sha256};
use solana_account::{AccountSharedData, ReadableAccount, WritableAccount};
use solana_compute_budget_interface::ComputeBudgetInstruction;
use solana_genesis_config::GenesisConfig;
use solana_instruction::{AccountMeta, Instruction};
use solana_loader_v3_interface::state::UpgradeableLoaderState;
use solana_message::{v0, AddressLookupTableAccount, Message, VersionedMessage};
use solana_pubkey::Pubkey;
use solana_runtime::bank::{Bank, BankTestConfig, SlotLeader};
use solana_sdk_ids::{address_lookup_table, bpf_loader_upgradeable, system_program, sysvar};
use solana_svm::{transaction_commit_result::CommittedTransaction, transaction_processor::ExecutionRecordingConfig};
use solana_transaction::versioned::VersionedTransaction;
use std::{collections::BTreeMap, env, fs, num::NonZeroUsize, path::Path, str::FromStr};

const LIMIT: u32 = 1_400_000;
const HEAP: u32 = 262_144;
const LOADED_LIMIT: u32 = 8 * 1024 * 1024;
const FEE: u64 = 10_000;
// Retained Token imports sol_get_sysvar; pinned Bank 4.2.0 gates it on this ID.
const GET_SYSVAR_FEATURE: Pubkey = Pubkey::from_str_const("CLCoTADvV64PSrnR6QXty6Fwrt9Xc6EdxSJE4wLRePjq");
const PIV_HASH: &str = "e29735e1467da98ecddb1eea9fd25abaffa4a450bd899ddc137f7d8c0da35b99";
type Accounts = BTreeMap<Pubkey, AccountSharedData>;
type Cpi = (Pubkey, Vec<u8>, Vec<Pubkey>, u8);

fn hex(b: &[u8]) -> String { b.iter().map(|n| format!("{n:02x}")).collect() }
fn hash(b: &[u8]) -> String { format!("{:x}", Sha256::digest(b)) }
fn emit(v: Value) { println!("PIV1_REAL_SQUADS_EVIDENCE {v}"); }
fn disc(name: &str) -> Vec<u8> { Sha256::digest(name.as_bytes())[..8].to_vec() }
fn number(b: &mut Vec<u8>, n: u64) { b.extend(n.to_le_bytes()); }
fn count(b: &mut Vec<u8>, n: usize) { b.extend(u32::try_from(n).unwrap().to_le_bytes()); }
fn bytes(b: &mut Vec<u8>, data: &[u8]) { count(b, data.len()); b.extend(data); }
fn meta(k: Pubkey, writable: bool, signer: bool) -> AccountMeta { AccountMeta { pubkey: k, is_writable: writable, is_signer: signer } }
fn ix(name: &str, accounts: Vec<AccountMeta>, args: Vec<u8>) -> Instruction {
    let mut data = disc(&format!("global:{name}")); data.extend(args); Instruction { program_id: SQUADS, accounts, data }
}
fn account(owner: Pubkey, lamports: u64, data: Vec<u8>, executable: bool) -> AccountSharedData {
    let mut a = AccountSharedData::new(lamports, data.len(), &owner);
    a.set_data_from_slice(&data); a.set_executable(executable); a.set_rent_epoch(u64::MAX); a
}
fn debit(map: &mut Accounts, k: Pubkey, amount: u64) {
    let a = map.get_mut(&k).unwrap(); a.set_lamports(a.lamports().checked_sub(amount).unwrap());
}
fn credit(map: &mut Accounts, k: Pubkey, amount: u64) {
    let a = map.get_mut(&k).unwrap(); a.set_lamports(a.lamports().checked_add(amount).unwrap());
}
fn replace(map: &mut Accounts, k: Pubkey, data: Vec<u8>) { map.get_mut(&k).unwrap().set_data_from_slice(&data); }
fn read_file(path: &Path, expected: &str, length: usize) -> Vec<u8> {
    assert!(path.is_absolute()); assert_eq!(path.canonicalize().unwrap(), path);
    assert!(path.symlink_metadata().unwrap().is_file());
    let b = fs::read(path).unwrap(); assert_eq!(b.len(), length); assert_eq!(hash(&b), expected); b
}
fn observed(inputs: &Value, k: Pubkey) -> AccountSharedData {
    let v = &inputs["accounts"][k.to_string()];
    let data = read_file(Path::new(v["path"].as_str().unwrap()), v["sha256"].as_str().unwrap(), v["bytes"].as_u64().unwrap() as usize);
    let mut a = account(Pubkey::from_str(v["owner"].as_str().unwrap()).unwrap(), v["lamports"].as_u64().unwrap(), data, v["executable"].as_bool().unwrap());
    a.set_rent_epoch(v["rent_epoch"].as_u64().unwrap()); a
}
fn load_external(g: &mut GenesisConfig, inputs: &Value, program: Pubkey, expected_hash: &str, expected_len: usize) {
    let a = &inputs["artifacts"][program.to_string()];
    assert_eq!(a["sha256"].as_str(), Some(expected_hash)); assert_eq!(a["bytes"].as_u64(), Some(expected_len as u64));
    let elf = read_file(Path::new(a["path"].as_str().unwrap()), expected_hash, expected_len);
    let program_key = program_data(program); assert_eq!(a["programdata"].as_str(), Some(program_key.to_string().as_str()));
    let old_program = observed(inputs, program); let old_pd = observed(inputs, program_key);
    assert_eq!(old_program.owner(), &bpf_loader_upgradeable::id()); assert!(old_program.executable());
    assert_eq!(old_program.data(), bincode::serialize(&UpgradeableLoaderState::Program { programdata_address: program_key }).unwrap());
    assert_eq!(old_pd.owner(), &bpf_loader_upgradeable::id()); assert!(!old_pd.executable()); assert_eq!(&old_pd.data()[45..], elf);
    assert_eq!(hash(old_pd.data()), a["programdata_sha256"].as_str().unwrap());
    let (old_slot, authority) = match bincode::deserialize::<UpgradeableLoaderState>(&old_pd.data()[..45]).unwrap() {
        UpgradeableLoaderState::ProgramData { slot, upgrade_authority_address } => (slot, upgrade_authority_address), _ => panic!("ProgramData variant"),
    };
    load(g, program, authority, &elf);
    emit(json!({"kind":"artifact", "program":program.to_string(), "elf_sha256":expected_hash, "elf_bytes":expected_len,
        "observed_deployment_slot":old_slot, "local_deployment_slot":0, "upgrade_authority":authority.map(|k|k.to_string()),
        "local_rent_epoch":u64::MAX, "local_program_and_programdata_rent":"synthetic Rent floors", "source_equivalence":false}));
}
fn load(g: &mut GenesisConfig, program: Pubkey, authority: Option<Pubkey>, elf: &[u8]) {
    assert_eq!(&elf[..4], b"\x7fELF"); assert_eq!(&elf[48..52], &[0;4]);
    let p = bincode::serialize(&UpgradeableLoaderState::Program { programdata_address: program_data(program) }).unwrap();
    let mut pd = bincode::serialize(&UpgradeableLoaderState::ProgramData { slot:0, upgrade_authority_address:authority }).unwrap();
    pd.resize(45,0); pd.extend(elf);
    for (k, b, executable) in [(program,p,true),(program_data(program),pd,false)] {
        assert!(g.accounts.insert(k, account(bpf_loader_upgradeable::id(),g.rent.minimum_balance(b.len()),b,executable).into()).is_none());
    }
}
fn bounded() -> BankTestConfig {
    let mut c = BankTestConfig::default(); let d = &mut c.accounts_db_config;
    d.num_foreground_threads=NonZeroUsize::new(1); d.num_background_threads=NonZeroUsize::new(1);
    d.read_cache_limit_bytes=Some((2*1024*1024,4*1024*1024)); d.read_cache_num_shards=Some(16); d.read_cache_evict_sample_size=Some(4);
    d.write_cache_limit_bytes=Some(8*1024*1024); let i=d.index.as_mut().unwrap(); i.bins=Some(2); i.num_flush_threads=NonZeroUsize::new(1); i.num_initial_accounts=Some(128); c
}
fn assert_feature(bank: &Bank) {
    assert_eq!(bank.feature_set.active().len(),1);
    assert_eq!(bank.feature_set.activated_slot(&GET_SYSVAR_FEATURE),Some(0));
    assert!(bank.feature_set.snapshot().get_sysvar_syscall_enabled);
}
fn snapshot(bank: &Bank, phase: &str) -> Accounts {
    let mut scanned = Accounts::new(); bank.scan_all_accounts(|v| { if let Some((k,a,slot))=v { assert!(slot<=1); assert!(scanned.insert(*k,a).is_none()); } }).unwrap();
    let mut result=Accounts::new();
    for (k,a) in scanned {
        let actual=bank.get_account(&k); if a.lamports()==0 { assert!(actual.is_none()); continue; }
        assert_eq!(actual.as_ref(),Some(&a));
        emit(json!({"kind":"account","phase":phase,"key":k.to_string(),"key_hex":hex(k.as_ref()),"owner":a.owner().to_string(),
            "lamports":a.lamports(),"executable":a.executable(),"rent_epoch":a.rent_epoch(),"data_hex":hex(a.data())})); result.insert(k,a);
    }
    assert!(!result.contains_key(&sysvar::instructions::id()));
    emit(json!({"kind":"snapshot","phase":phase,"count":result.len(),"slot":bank.slot(),"transaction_count":bank.transaction_count()})); result
}
fn compiled(w: &World) -> (Vec<u8>, Vec<u8>, Instruction) {
    let mut order:Vec<usize>=(0..w.roles.len()).collect();
    order.sort_by_key(|i|match(w.roles[*i].signer,w.roles[*i].writable){(true,true)=>0,(true,false)=>1,(false,true)=>2,_=>3});
    let indices:Vec<u8>=(0..w.roles.len()).map(|i|order.iter().position(|j|*j==i).unwrap() as u8).collect();
    let mut compact=vec![2,1,16,order.len() as u8]; for i in &order { compact.extend(w.roles[*i].key.as_ref()); }
    compact.extend([1,indices[0],indices.len() as u8]);compact.extend(&indices);compact.extend((w.inner_data.len() as u16).to_le_bytes());compact.extend(&w.inner_data);compact.push(0);
    let mut stored=disc("account:VaultTransaction");stored.extend(w.roles[2].key.as_ref());stored.extend(key(91).as_ref());number(&mut stored,19);
    stored.extend([Pubkey::find_program_address(&[b"multisig",w.roles[2].key.as_ref(),b"transaction",&19_u64.to_le_bytes()],&SQUADS).1,7,
        Pubkey::find_program_address(&[b"multisig",w.roles[2].key.as_ref(),b"vault",&[7]],&SQUADS).1]);
    count(&mut stored,0);stored.extend([2,1,16]);count(&mut stored,order.len());for i in &order {stored.extend(w.roles[*i].key.as_ref());}
    count(&mut stored,1);stored.push(indices[0]);bytes(&mut stored,&indices);bytes(&mut stored,&w.inner_data);count(&mut stored,0);
    let mut roles=vec![meta(w.roles[2].key,false,false),meta(w.roles[3].key,true,false),meta(w.roles[4].key,false,false),meta(key(91),false,true)];
    roles.extend(order.iter().map(|i|meta(w.roles[*i].key,w.roles[*i].writable||*i==3,w.roles[*i].signer&&*i!=5)));
    assert_eq!(compact.len(),1447);assert_eq!(stored.len(),1548);
    (compact,stored,ix("vault_transaction_execute",roles,vec![]))
}
fn proposal(w: &World, status: u8, approvals: usize, time: i64) -> Vec<u8> {
    let mut b=disc("account:Proposal");b.extend(w.roles[2].key.as_ref());number(&mut b,19);b.push(status);b.extend(time.to_le_bytes());
    b.push(Pubkey::find_program_address(&[b"multisig",w.roles[2].key.as_ref(),b"transaction",&19_u64.to_le_bytes(),b"proposal"],&SQUADS).1);
    count(&mut b,approvals);for i in 0..approvals { b.extend(key(91+i as u8).as_ref()); } count(&mut b,0);count(&mut b,0);b.resize(646,0);b
}
fn buffer(w: &World, message: &[u8], uploaded: usize) -> Vec<u8> {
    let mut b=disc("account:TransactionBuffer");b.extend(w.roles[2].key.as_ref());b.extend(key(91).as_ref());b.extend([1,7]);
    b.extend(Sha256::digest(message));b.extend((message.len() as u16).to_le_bytes());bytes(&mut b,&message[..uploaded]);b.resize(112+message.len(),0);b
}
fn create_cpi(k: Pubkey, size: usize, lamports: u64) -> Cpi {
    let mut b=0_u32.to_le_bytes().to_vec();number(&mut b,lamports);number(&mut b,size as u64);b.extend(SQUADS.as_ref());
    (system_program::id(),b,vec![key(222),k],2)
}
fn step(bank: &Bank, name: &str, instructions: Vec<Instruction>, alt: Option<&AddressLookupTableAccount>, expected: &Accounts,
    expected_cpis: &[Cpi], fees: usize) -> (Accounts, CommittedTransaction) {
    assert_feature(bank);
    let message=if let Some(table)=alt { VersionedMessage::V0(v0::Message::try_compile(&key(222),&instructions,&[table.clone()],bank.last_blockhash()).unwrap()) }
        else { VersionedMessage::Legacy(Message::new_with_blockhash(&instructions,Some(&key(222)),&bank.last_blockhash())) };
    assert_eq!(message.header().num_required_signatures,2);assert_eq!(message.static_account_keys()[0],key(222));
    let tx=VersionedTransaction{signatures:vec![Default::default();2],message};let wire=bincode::serialize(&tx).unwrap();assert!(wire.len()<=1232);
    assert!(tx.signatures.iter().all(|s|s.as_ref()==[0;64]));
    let mut keys=tx.message.static_account_keys().to_vec();
    if let VersionedMessage::V0(m)=&tx.message { let table=alt.unwrap();assert_eq!(m.address_table_lookups.len(),1);assert_eq!(m.address_table_lookups[0].account_key,table.key);
        for i in &m.address_table_lookups[0].writable_indexes {keys.push(table.addresses[*i as usize]);}for i in &m.address_table_lookups[0].readonly_indexes {keys.push(table.addresses[*i as usize]);}
        assert_eq!(m.address_table_lookups[0].writable_indexes.len(),16);assert!(m.address_table_lookups[0].readonly_indexes.is_empty()); }
    emit(json!({"kind":"transaction","case":name,"wire_hex":hex(&wire),"wire_bytes":wire.len(),"message_hex":hex(&tx.message.serialize()),
        "message_hash":tx.message.hash().to_string(),"account_keys":keys.iter().map(ToString::to_string).collect::<Vec<_>>(),"zero_signature_slots":2,
        "compute_limit":if alt.is_some(){LIMIT}else{200000},"heap_bytes":if alt.is_some(){HEAP}else{32768},
        "loaded_account_limit":if alt.is_some(){LOADED_LIMIT}else{64*1024*1024},"signature_verified":false}));
    let batch=bank.prepare_entry_batch(vec![tx]).expect("unsigned local entry");
    let(mut results,_)=bank.load_execute_and_commit_transactions(&batch,ExecutionRecordingConfig::new_single_setting(true),&mut Default::default(),Some(256*1024));drop(batch);
    assert_eq!(results.len(),1);let committed=results.remove(0).expect("success reaches commit");
    emit(json!({"kind":"commit","case":name,"status":format!("{:?}",committed.status),"executed_units":committed.executed_units,
        "loaded_accounts_data_size":committed.loaded_account_stats.loaded_accounts_data_size,"loaded_accounts_count":committed.loaded_account_stats.loaded_accounts_count,
        "fee":committed.fee_details.total_fee(),"logs":committed.log_messages}));
    assert_eq!(committed.status,Ok(()));assert_eq!(committed.fee_details.total_fee(),FEE);assert_eq!(committed.fee_details.transaction_fee(),FEE);
    assert_eq!(committed.fee_details.prioritization_fee(),0);assert!(committed.executed_units>0&&committed.executed_units<=u64::from(LIMIT));
    assert!(committed.loaded_account_stats.loaded_accounts_data_size<=if alt.is_some(){LOADED_LIMIT}else{64*1024*1024});
    let mut actual_cpis=vec![];
    for (outer,list) in committed.inner_instructions.as_ref().unwrap().iter().enumerate() {for (ordinal,inner) in list.iter().enumerate() {
        let i=&inner.instruction;let c=(keys[i.program_id_index as usize],i.data.clone(),i.accounts.iter().map(|n|keys[*n as usize]).collect::<Vec<_>>(),inner.stack_height);
        emit(json!({"kind":"cpi","case":name,"outer":outer,"ordinal":ordinal,"program":c.0.to_string(),"data_hex":hex(&c.1),"accounts":c.2.iter().map(ToString::to_string).collect::<Vec<_>>(),"stack_height":c.3}));actual_cpis.push(c);
    }}
    assert_eq!(actual_cpis,expected_cpis,"complete ordered CPI stream: {name}");
    let after=snapshot(bank,name);assert_eq!(&after,expected,"complete independent expected map: {name}");
    assert_eq!(after[&key(222)].lamports(),committed.fee_payer_post_balance);assert_eq!(bank.transaction_count(),fees as u64);
    (after,committed)
}

#[test]
fn real_squads_native_initializer_success_only() {
    assert_eq!(env::var("RAYON_NUM_THREADS").as_deref(),Ok("1"));
    let input_path=env::var("PIV_REAL_SQUADS_INPUTS").unwrap();let input_path=Path::new(&input_path);
    let input_bytes=fs::read(input_path).unwrap();assert_eq!(hash(&input_bytes),env::var("PIV_REAL_SQUADS_INPUTS_SHA256").unwrap());
    let inputs:Value=serde_json::from_slice(&input_bytes).unwrap();
    let mut w=World::production(true,false);assert_eq!(w.roles.len(),34);assert_eq!(w.payer,29);assert_eq!(w.inner_data.len(),315);
    let list=Pubkey::from_str("G5N6K3qW86GSkNEpywcbJk42LjEZoshzECFg1LNVjSLa").unwrap();
    let reserve=Pubkey::from_str("CzKqc9cs4XpyG6y4peQgk3vBjPyqhktmfqaMuMBCXCqm").unwrap();
    let manager=Pubkey::from_str("8yoigZfzZ1nNaadumY9uPVD118225UYHTDpmjpr2nrSa").unwrap();
    for (role,k) in [(24,JITO_POOL),(25,list),(26,reserve),(27,JITO_MINT),(28,manager)] {
        let a=observed(&inputs,k);assert!(!a.executable());let r=&mut w.roles[role];r.key=k;r.owner=*a.owner();r.data=a.data().to_vec();r.lamports=a.lamports();
    }
    assert_eq!(w.roles[25].data.len(),730009);
    for (offset,k) in [(75,list),(107,reserve),(171,manager),(203,manager)] { w.inner_data[offset..offset+32].copy_from_slice(k.as_ref()); }
    assert_eq!(&w.inner_data[..11],b"PIV1IN01\x01\x07\x00");assert_eq!(&w.inner_data[313..],&[31,202]);
    w.roles[2].data[74..78].copy_from_slice(&0_u32.to_le_bytes());w.roles[2].data[78..86].copy_from_slice(&18_u64.to_le_bytes());
    assert_eq!(&w.roles[2].data[86..94],&3_u64.to_le_bytes());
    let(message,stored,execute)=compiled(&w);
    let buffer_key=Pubkey::find_program_address(&[b"multisig",w.roles[2].key.as_ref(),b"transaction_buffer",key(91).as_ref(),&[1]],&SQUADS).0;
    let alt_key=Pubkey::find_program_address(&[key(91).as_ref(),&42_u64.to_le_bytes()],&address_lookup_table::id()).0;
    let alt=AddressLookupTableAccount{key:alt_key,addresses:(0..16).map(|i|World::derive(i).0).collect()};
    let mut alt_data=1_u32.to_le_bytes().to_vec();number(&mut alt_data,u64::MAX);number(&mut alt_data,0);alt_data.extend([0,1]);alt_data.extend(key(91).as_ref());alt_data.extend([0;2]);assert_eq!(alt_data.len(),56);
    for k in &alt.addresses {alt_data.extend(k.as_ref());}
    let mut g=GenesisConfig{creation_time:100,..GenesisConfig::default()};g.rent.lamports_per_byte=6960;
    g.fee_rate_governor.lamports_per_signature=5000;g.fee_rate_governor.target_lamports_per_signature=5000;g.fee_rate_governor.target_signatures_per_slot=0;
    g.fee_rate_governor.min_lamports_per_signature=5000;g.fee_rate_governor.max_lamports_per_signature=5000;g.fee_rate_governor.burn_percent=50;
    for (i,r) in w.roles.iter().enumerate() {
        if [0,1,3,4,6,23,30,31].contains(&i)||(7..23).contains(&i){continue;}
        assert!(r.lamports>=g.rent.minimum_balance(r.data.len()));assert!(g.accounts.insert(r.key,r.account().into()).is_none());
    }
    for i in 91..97 {assert!(g.accounts.insert(key(i),account(system_program::id(),1_000_000_000,vec![],false).into()).is_none());}
    g.accounts.insert(alt.key,account(address_lookup_table::id(),g.rent.minimum_balance(alt_data.len()),alt_data,false).into());
    load_external(&mut g,&inputs,SQUADS,"95d0811d25f9df7d21f230e34ce93fcb37d22a9b4fe1365f07cd339e9ad165f1",980096);
    load_external(&mut g,&inputs,TOKEN,"8190d3f7ceb6cb7a7a8d8924bff89f9f611e15ce1f806f2b6237f3311a98f697",108600);
    load_external(&mut g,&inputs,JITO_PROGRAM,"6c0cb7ef5da80a23c3ad85d7fec566f3cc17b815d428daec84bb916441a2a143",1106080);
    let piv_path=env::var("PIV_REAL_SQUADS_PIV_PATH").unwrap();let piv=read_file(Path::new(&piv_path),PIV_HASH,894552);load(&mut g,PROGRAM,Some(w.vault),&piv);
    // Exact synthetic capability selection, using the unchanged cached helper.
    // It constructs Feature-owned Some(0), not a pending feature or image change.
    assert!(!g.accounts.contains_key(&GET_SYSVAR_FEATURE));
    solana_runtime::genesis_utils::activate_feature(&mut g,GET_SYSVAR_FEATURE);
    let feature_account=AccountSharedData::from(g.accounts[&GET_SYSVAR_FEATURE].clone());
    assert_eq!(feature_account.owner(),&solana_sdk_ids::feature::id());assert!(!feature_account.executable());
    assert_eq!(feature_account.data(),&[1,0,0,0,0,0,0,0,0]);assert_eq!(feature_account.lamports(),g.rent.minimum_balance(9));
    emit(json!({"kind":"fixture","genesis_hash":g.hash().to_string(),"inputs_sha256":hash(&input_bytes),"piv_sha256":PIV_HASH,
        "synthetic_multisig":true,"timelock":0,"threshold":4,"members":6,"transaction_index":18,"stale_index":3,"synthetic_piv_authority":w.vault.to_string(),
        "retained_mixed_slot_external_accounts":true,"validator_list_bytes":730009,"synthetic_seeded_alt":alt.key.to_string(),"alt_last_extended_slot":0,
        "synthetic_feature_profile":[{"id":GET_SYSVAR_FEATURE.to_string(),"activated_at":0}],"get_sysvar_syscall_enabled":true,
        "signature_verified":false,"public_ready":false,"m4_criterion_waived":false}));
    let directory=env::var("PIV1_BANK_ACCOUNTS_DIR").unwrap();let directory=Path::new(&directory);assert!(directory.is_absolute());
    assert_eq!(directory.parent().unwrap().canonicalize().unwrap(),directory.parent().unwrap());assert!(!directory.exists());fs::create_dir(directory).unwrap();
    let leader=SlotLeader{id:key(246),vote_address:key(247)};let root=Bank::new_with_paths_for_tests(&g,Some(bounded()),vec![directory.to_path_buf()],Some(leader));assert_feature(&root);
    assert_eq!(root.get_account(&GET_SYSVAR_FEATURE).as_ref(),Some(&feature_account));
    let(root,forks)=root.wrap_with_bank_forks_for_tests();let bank=Bank::new_from_parent_with_bank_forks(&forks,root,leader,1);
    assert_feature(&bank);assert_eq!(bank.clock().unix_timestamp,100);assert_eq!(bank.slot(),1);
    assert_eq!(bank.get_account(&GET_SYSVAR_FEATURE).as_ref(),Some(&feature_account));
    let mut before=snapshot(&bank,"initial");for (k,a) in &g.accounts {assert_eq!(before.get(k),Some(&AccountSharedData::from(a.clone())));}
    for k in (0..16).map(|i|World::derive(i).0).chain([buffer_key,w.roles[3].key,w.roles[4].key]) {assert!(!before.contains_key(&k));}
    for i in 0..16 {assert_eq!(fixture::floor(i),g.rent.minimum_balance(SIZES[i]));}
    let mut args=vec![1,7];args.extend(Sha256::digest(&message));args.extend((message.len() as u16).to_le_bytes());bytes(&mut args,&message[..800]);
    let create=ix("transaction_buffer_create",vec![meta(w.roles[2].key,false,false),meta(buffer_key,true,false),meta(key(91),false,true),meta(key(222),true,true),meta(system_program::id(),false,false)],args);
    let buffer_rent=g.rent.minimum_balance(112+message.len());assert_eq!(buffer_rent,11_741_520);
    let mut expected=before.clone();debit(&mut expected,key(222),FEE+buffer_rent);expected.insert(buffer_key,account(SQUADS,buffer_rent,buffer(&w,&message,800),false));
    (before,_)=step(&bank,"buffer_create",vec![create],None,&expected,&[create_cpi(buffer_key,1559,buffer_rent)],1);
    let mut args=vec![];bytes(&mut args,&message[800..]);let extend=ix("transaction_buffer_extend",vec![meta(w.roles[2].key,false,false),meta(buffer_key,true,false),meta(key(91),false,true)],args);
    expected=before.clone();debit(&mut expected,key(222),FEE);replace(&mut expected,buffer_key,buffer(&w,&message,message.len()));
    (before,_)=step(&bank,"buffer_extend",vec![extend],None,&expected,&[],2);
    let mut args=vec![7,0];bytes(&mut args,&[0;6]);args.push(0);
    let finish=ix("vault_transaction_create_from_buffer",vec![meta(w.roles[2].key,true,false),meta(w.roles[4].key,true,false),meta(key(91),false,true),meta(key(222),true,true),meta(system_program::id(),false,false),meta(buffer_key,true,false),meta(key(91),true,true)],args);
    let tx_rent=g.rent.minimum_balance(stored.len());assert_eq!(tx_rent,11_664_960);
    expected=before.clone();debit(&mut expected,key(222),FEE+tx_rent);credit(&mut expected,key(91),buffer_rent);expected.remove(&buffer_key);
    expected.insert(w.roles[4].key,account(SQUADS,tx_rent,stored.clone(),false));let mut multisig=w.roles[2].data.clone();multisig[78..86].copy_from_slice(&19_u64.to_le_bytes());replace(&mut expected,w.roles[2].key,multisig);
    let initial_tx_rent=g.rent.minimum_balance(102);let mut top_up=2_u32.to_le_bytes().to_vec();number(&mut top_up,tx_rent-initial_tx_rent);
    (before,_)=step(&bank,"transaction_from_buffer",vec![finish],None,&expected,&[create_cpi(w.roles[4].key,102,initial_tx_rent),(system_program::id(),top_up,vec![key(222),w.roles[4].key],2)],3);
    let mut args=19_u64.to_le_bytes().to_vec();args.push(0);let create=ix("proposal_create",vec![meta(w.roles[2].key,false,false),meta(w.roles[3].key,true,false),meta(key(91),false,true),meta(key(222),true,true),meta(system_program::id(),false,false)],args);
    let proposal_rent=g.rent.minimum_balance(646);assert_eq!(proposal_rent,5_387_040);
    expected=before.clone();debit(&mut expected,key(222),FEE+proposal_rent);expected.insert(w.roles[3].key,account(SQUADS,proposal_rent,proposal(&w,1,0,100),false));
    (before,_)=step(&bank,"proposal_create",vec![create],None,&expected,&[create_cpi(w.roles[3].key,646,proposal_rent)],4);
    for i in 0..4 {let approve=ix("proposal_approve",vec![meta(w.roles[2].key,false,false),meta(key(91+i),true,true),meta(w.roles[3].key,true,false)],vec![0]);
        expected=before.clone();debit(&mut expected,key(222),FEE);replace(&mut expected,w.roles[3].key,proposal(&w,if i==3{3}else{1},i as usize+1,100));
        (before,_)=step(&bank,&format!("approve_{}",i+1),vec![approve],None,&expected,&[],5+i as usize);
        for n in 0..16 {assert!(!before.contains_key(&World::derive(n).0));}
    }
    expected=before.clone();debit(&mut expected,key(222),FEE+w.shortfall());replace(&mut expected,w.roles[3].key,proposal(&w,5,4,100));
    let mut states=w.state_bytes();for (n,k) in [(2,list),(3,reserve),(8,manager),(9,manager)] {let start=22+32*n;states[0][start..start+32].copy_from_slice(k.as_ref());}
    for i in 0..16 {let data=if i<9{states[i].clone()}else if i<14{vec![]}else{w.token_bytes()};expected.insert(World::derive(i).0,account(World::target_owner(i),w.target_balance(i),data,false));}
    let(final_map,result)=step(&bank,"execute",vec![ComputeBudgetInstruction::set_compute_unit_limit(LIMIT),ComputeBudgetInstruction::request_heap_frame(HEAP),ComputeBudgetInstruction::set_loaded_accounts_data_size_limit(LOADED_LIMIT),execute],Some(&alt),&expected,&w.expected_cpis(),9);
    assert!(result.loaded_account_stats.loaded_accounts_data_size>=730009);assert_eq!(final_map[&w.roles[3].key].data()[48],5);assert_eq!(final_map[&w.roles[4].key].data(),stored);
    let logs=result.log_messages.as_ref().unwrap();let piv_done=logs.iter().position(|l|l==&format!("Program {PROGRAM} success")).unwrap();let squads_done=logs.iter().position(|l|l==&format!("Program {SQUADS} success")).unwrap();assert!(piv_done<squads_done);
    assert_eq!(logs.iter().filter(|l|*l==&format!("Program {TOKEN} success")).count(),2);
    for p in [JITO_PROGRAM,fixture::STAKE] {assert!(!logs.iter().any(|l|l.starts_with(&format!("Program {p} invoke"))));}
    emit(json!({"kind":"complete","status":"PASS","successful_transactions":9,"complete_maps":10,"total_fees":9*FEE,
        "piv_rent":w.shortfall(),"buffer_refund_to_creator":buffer_rent,"transaction_rent":tx_rent,"proposal_rent":proposal_rent,
        "proposal_executed_directly_read":true,"approved_during_cpi":"inference from unchanged PIV guard success and pinned source; no mid-CPI account dump",
        "whole_execute_units":result.executed_units,"pool_executed":false,"stake_executed":false,"signature_verified":false,
        "multisig_creation_executed":false,"alt_creation_executed":false,"replay_tested":false,"public_ready":false,"m4_criterion_waived":false}));
}
