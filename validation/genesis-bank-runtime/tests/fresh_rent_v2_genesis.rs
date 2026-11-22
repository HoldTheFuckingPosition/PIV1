//! One fresh-instance qualification in a local Agave Bank. No recovery or live sender.
#![recursion_limit = "256"]
use base64::{engine::general_purpose::STANDARD, Engine};
use serde_json::{json, Value};
use sha2::{Digest, Sha256};
use solana_account::{AccountSharedData, ReadableAccount, WritableAccount};
use solana_compute_budget_interface::ComputeBudgetInstruction as Budget;
use solana_genesis_config::GenesisConfig;
use solana_instruction::{AccountMeta, Instruction};
use solana_keypair::Keypair;
use solana_loader_v3_interface::state::UpgradeableLoaderState;
use solana_message::{v0, AddressLookupTableAccount, Message, VersionedMessage};
use solana_pubkey::Pubkey;
use solana_runtime::bank::{Bank, BankTestConfig, SlotLeader};
use solana_sdk_ids::{address_lookup_table, bpf_loader_upgradeable as loader, system_program as system, sysvar};
use solana_signer::Signer;
use solana_svm::{transaction_commit_result::CommittedTransaction, transaction_processor::ExecutionRecordingConfig};
use solana_transaction::{versioned::VersionedTransaction, TransactionVerificationMode};
use std::{collections::BTreeMap, env, fs, num::NonZeroUsize, path::Path};

const PROGRAM: Pubkey = Pubkey::new_from_array([217; 32]);
const SQUADS: Pubkey = Pubkey::from_str_const("SQDS4ep65T869zMMBKyuUq6aD6EgTu8psMjkvj52pCf");
const TOKEN: Pubkey = Pubkey::from_str_const("TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA");
const JITO: Pubkey = Pubkey::from_str_const("SPoo1Ku8WFXoNDMHPsrGSTSG1Y47rzgn41SLUNakuHy");
const JITO_POOL: Pubkey = Pubkey::from_str_const("Jito4APyf642JPZPx3hGc6WWJ8zPKtRbRs4P815Awbb");
const JITO_MINT: Pubkey = Pubkey::from_str_const("J1toso1uCk3RLmjorhTtrVwY9HJ7X8V9yYac6Y7kGCPn");
const ELF_SHA: &str = "2ea454e9eac013bab063651c6691bf1cf84fad3e5d3e0b67375f2603e4138109";
const SIZES: [usize;16] = [1014,891,210,84,84,84,84,84,84,0,0,0,0,0,165,165];
const LIMIT: u32 = 1_400_000;
const HEAP: u32 = 262_144;
const LOADED: u32 = 8 * 1024 * 1024;
const FEATURE: Pubkey = Pubkey::from_str_const("CLCoTADvV64PSrnR6QXty6Fwrt9Xc6EdxSJE4wLRePjq");

fn emit(value: Value) { println!("GENESIS_EVIDENCE {value}"); }
fn hex(bytes: &[u8]) -> String { bytes.iter().map(|v| format!("{v:02x}")).collect() }
fn digest(bytes: &[u8]) -> String { hex(&Sha256::digest(bytes)) }
fn discriminator(name: &str) -> Vec<u8> { Sha256::digest(name.as_bytes())[..8].to_vec() }
fn sized(bytes: &mut Vec<u8>, payload: &[u8]) { bytes.extend((payload.len() as u32).to_le_bytes()); bytes.extend(payload); }
fn u64_at(bytes: &[u8], at: usize) -> u64 { u64::from_le_bytes(bytes[at..at+8].try_into().unwrap()) }
fn key(value: &Value) -> Pubkey { value.as_str().unwrap().parse().unwrap() }
fn pda(program: Pubkey, seeds: &[&[u8]]) -> (Pubkey, u8) { Pubkey::find_program_address(seeds, &program) }
fn program_data(program: Pubkey) -> Pubkey { pda(loader::id(), &[program.as_ref()]).0 }
fn meta(pubkey: Pubkey, is_writable: bool, is_signer: bool) -> AccountMeta {
    AccountMeta { pubkey, is_writable, is_signer }
}
fn account(owner: Pubkey, lamports: u64, bytes: &[u8], executable: bool) -> AccountSharedData {
    let mut a = AccountSharedData::new(lamports, bytes.len(), &owner);
    a.set_data_from_slice(bytes);
    a.set_executable(executable);
    a.set_rent_epoch(u64::MAX);
    a
}
fn observed(inputs: &Value, pubkey: Pubkey) -> AccountSharedData {
    let row = &inputs["accounts"][pubkey.to_string()];
    let bytes = fs::read(row["path"].as_str().unwrap()).unwrap();
    assert_eq!((bytes.len() as u64, digest(&bytes)), (row["bytes"].as_u64().unwrap(), row["sha256"].as_str().unwrap().into()));
    let mut a = account(key(&row["owner"]), row["lamports"].as_u64().unwrap(), &bytes, row["executable"].as_bool().unwrap());
    a.set_rent_epoch(row["rent_epoch"].as_u64().unwrap());
    a
}
fn insert(g: &mut GenesisConfig, k: Pubkey, a: AccountSharedData) {
    assert!(g.accounts.insert(k, a.into()).is_none(), "duplicate genesis account: {k}");
}
fn install(g: &mut GenesisConfig, program: Pubkey, authority: Option<Pubkey>, elf: &[u8]) {
    assert_eq!(&elf[..4], b"\x7fELF");
    let pd = program_data(program);
    let program_record = bincode::serialize(&UpgradeableLoaderState::Program { programdata_address: pd }).unwrap();
    let mut image = bincode::serialize(&UpgradeableLoaderState::ProgramData { slot: 0, upgrade_authority_address: authority }).unwrap();
    image.resize(45, 0); image.extend(elf);
    for (key, bytes, executable) in [(program, program_record, true), (pd, image, false)] {
        insert(g, key, account(loader::id(), g.rent.minimum_balance(bytes.len()), &bytes, executable));
    }
}
fn install_observed(g: &mut GenesisConfig, inputs: &Value, program: Pubkey, sha: &str, size: usize) {
    let pd = program_data(program);
    let a = observed(inputs, program);
    let b = observed(inputs, pd);
    assert_eq!((*a.owner(), *b.owner(), a.executable(), b.executable()), (loader::id(), loader::id(), true, false));
    assert_eq!(bincode::deserialize::<UpgradeableLoaderState>(a.data()).unwrap(), UpgradeableLoaderState::Program { programdata_address: pd });
    let UpgradeableLoaderState::ProgramData { slot, upgrade_authority_address } =
        bincode::deserialize(&b.data()[..45]).unwrap() else { panic!("expected Loader ProgramData"); };
    assert_eq!((b.data()[45..].len(), digest(&b.data()[45..])), (size, sha.into()));
    install(g, program, upgrade_authority_address, &b.data()[45..]);
    emit(json!({"kind":"artifact","program":program.to_string(),"programData":pd.to_string(),"sha256":sha,"bytes":size,
        "observedDeploymentSlot":slot,"localDeploymentSlot":0}));
}
fn ix(program_id: Pubkey, accounts: Vec<AccountMeta>, data: Vec<u8>) -> Instruction {
    Instruction { program_id, accounts, data }
}
fn squad_ix(name: &str, roles: Vec<AccountMeta>, args: &[u8]) -> Instruction {
    let mut bytes = discriminator(&format!("global:{name}")); bytes.extend(args);
    ix(SQUADS, roles, bytes)
}
fn app_ix(selector: &[u8; 8], version: u8, metas: Vec<AccountMeta>, args: &[u8]) -> Instruction {
    let mut bytes = selector.to_vec(); bytes.push(version); bytes.extend(args); ix(PROGRAM, metas, bytes)
}
fn instruction(name: &str, ix: &Instruction) {
    emit(json!({"kind":"instruction","name":name,"programId":ix.program_id.to_string(),"dataHex":hex(&ix.data),
        "accounts":ix.accounts.iter().map(|a|json!({"address":a.pubkey.to_string(),"signer":a.is_signer,"writable":a.is_writable})).collect::<Vec<_>>()}));
}

struct Actors {
    fee: Keypair,
    rent: Keypair,
    contributor: Keypair,
    guardians: Vec<Keypair>,
    create: Pubkey,
    multisig: Pubkey,
    multisig_bump: u8,
    vault: Pubkey,
    recipients: [Pubkey; 2],
}
impl Actors {
    fn new() -> Self {
        let create = Keypair::new().pubkey();
        let (multisig, multisig_bump) = pda(SQUADS, &[b"multisig", b"multisig", create.as_ref()]);
        let mut guardians: Vec<_> = (0..6).map(|_| Keypair::new()).collect();
        guardians.sort_by_key(|signer| signer.pubkey().to_bytes());
        Self { fee: Keypair::new(), rent: Keypair::new(), contributor: Keypair::new(),
            recipients: [Keypair::new().pubkey(), Keypair::new().pubkey()], guardians, create, multisig, multisig_bump,
            vault: pda(SQUADS, &[b"multisig", multisig.as_ref(), b"vault", &[0]]).0 }
    }
    fn multisig_data(&self) -> Vec<u8> {
        let mut bytes = discriminator("account:Multisig"); bytes.extend(self.create.as_ref()); bytes.extend([0;32]);
        bytes.extend(4_u16.to_le_bytes()); bytes.extend(0_u32.to_le_bytes()); bytes.extend([0;16]);
        bytes.extend([0, self.multisig_bump]); bytes.extend(6_u32.to_le_bytes());
        for guardian in &self.guardians { bytes.extend(guardian.pubkey().as_ref()); bytes.push(7); }
        bytes.resize(330, 0); bytes
    }
}
fn topology(actors: &Actors) -> [Pubkey;16] {
    let seeds: [&[u8];16] = [b"config",b"distribution",b"guardian-registry",b"",b"",b"",b"",b"",b"",
        b"pending-sol",b"principal-sol",b"operational-sol",b"distribution-escrow",b"kif-sol",
        b"principal-jito-vault",b"pending-jito-vault"];
    std::array::from_fn(|i| {
        if (3..9).contains(&i) {
            pda(PROGRAM, &[b"guardian-reward",actors.guardians[i-3].pubkey().as_ref(),&0u64.to_le_bytes(),&[(i-3) as u8]]).0
        } else { pda(PROGRAM,&[seeds[i]]).0 }
    })
}
fn governance_keys(actors: &Actors, index: u64) -> (Pubkey, Pubkey) {
    let number = index.to_le_bytes();
    let seeds = [b"multisig".as_slice(), actors.multisig.as_ref(), b"transaction", &number];
    (pda(SQUADS,&seeds).0, pda(SQUADS,&[seeds[0],seeds[1],seeds[2],seeds[3],b"proposal"]).0)
}
fn governance_metas(actors: &Actors, index: u64) -> Vec<AccountMeta> {
    let (tx, proposal) = governance_keys(actors, index);
    vec![meta(PROGRAM,false,false), meta(program_data(PROGRAM),false,false), meta(actors.multisig,false,false),
        meta(proposal,false,false), meta(tx,false,false), meta(actors.vault,false,true), meta(sysvar::instructions::id(),false,false)]
}
fn fixed(targets: &[Pubkey;16], writable: &[usize]) -> Vec<AccountMeta> {
    [0,1,9,10,11,12,13,14,15].into_iter().enumerate()
        .map(|(i,j)|meta(targets[j],writable.contains(&i),false)).collect()
}

fn take_step(bank: &Bank, actors: &Actors, name: &str, mut ixs: Vec<Instruction>, signers: &[&Keypair],
    alt: Option<&AddressLookupTableAccount>, bounded: bool, expect: bool) -> CommittedTransaction
{
    if bounded {
        let mut limits = vec![Budget::set_compute_unit_limit(LIMIT), Budget::request_heap_frame(HEAP),
            Budget::set_loaded_accounts_data_size_limit(LOADED)];
        limits.append(&mut ixs); ixs = limits;
    }
    let blockhash = bank.last_blockhash();
    let payer = actors.fee.pubkey();
    let message = match alt {
        Some(table) => VersionedMessage::V0(v0::Message::try_compile(&payer,&ixs,&[table.clone()],blockhash).unwrap()),
        None => VersionedMessage::Legacy(Message::new_with_blockhash(&ixs,Some(&payer),&blockhash)),
    };
    let mut actual_signers = vec![&actors.fee]; actual_signers.extend(signers);
    let tx = VersionedTransaction::try_new(message.clone(), &actual_signers).unwrap();
    let unsigned = VersionedTransaction { signatures: vec![Default::default();tx.signatures.len()], message };
    let unsigned_wire = bincode::serialize(&unsigned).unwrap();
    assert!(unsigned_wire.len() <= 1232, "{name}: wire exceeds legacy/v0 packet envelope");
    let sanitized = bank.verify_transaction(tx, TransactionVerificationMode::FullVerification).expect("verified local signatures");
    let batch = bank.prepare_sanitized_batch(std::slice::from_ref(&sanitized));
    let (mut results, _) = bank.load_execute_and_commit_transactions(&batch,
        ExecutionRecordingConfig::new_single_setting(true), &mut Default::default(), Some(256*1024));
    drop(batch);
    let result = results.remove(0).expect("Bank must record an executed result");
    let keys: Vec<_> = unsigned.message.static_account_keys().iter().map(ToString::to_string).collect();
    emit(json!({"kind":"packet","case":name,"status":format!("{:?}",result.status),
        "wireBytes":unsigned_wire.len(),"messageBase64":STANDARD.encode(unsigned.message.serialize()),
        "unsignedBase64":STANDARD.encode(unsigned_wire),"staticAccountKeys":keys,"signerSlots":unsigned.signatures.len(),
        "blockhash":blockhash.to_string(),"lastValidBlockHeight":bank.get_blockhash_last_valid_block_height(&blockhash).unwrap().to_string(),
        "computeLimit":if bounded{Some(LIMIT)}else{None},"heapBytes":if bounded{HEAP}else{32768},
        "loadedLimit":if bounded{LOADED}else{64*1024*1024},"executedUnits":result.executed_units,
        "loadedAccountBytes":result.loaded_account_stats.loaded_accounts_data_size,
        "loadedAccounts":result.loaded_account_stats.loaded_accounts_count,
        "feeLamports":result.fee_details.total_fee(),"logs":result.log_messages}));
    assert_eq!(result.fee_details.total_fee(),5_000*unsigned.signatures.len() as u64, "{name}: exact local fee");
    assert_eq!(result.status.is_ok(),expect,"{name}: {:?}",result.status);
    result
}

fn authorize(bank: &Bank, a:&Actors, name:&str, index:u64, inner:&Instruction) -> Instruction {
    let (tx, proposal) = governance_keys(a,index);
    let initiator = &a.guardians[0]; let proposer = initiator.pubkey();
    let buffer = pda(SQUADS,&[b"multisig",a.multisig.as_ref(),b"transaction_buffer",proposer.as_ref(),&[index as u8]]).0;
    let mut order = inner.accounts.clone();
    order.sort_by_key(|m|match(m.is_signer,m.is_writable){(true,true)=>0,(true,false)=>1,(false,true)=>2,_=>3});
    let positions:Vec<u8> = inner.accounts.iter().map(|meta|order.iter().position(|m|m.pubkey==meta.pubkey).unwrap() as u8).collect();
    let writable_signers = order.iter().filter(|m|m.is_signer && m.is_writable).count() as u8;
    let signers = order.iter().filter(|m|m.is_signer).count() as u8;
    let writable_non_signers = order.iter().filter(|m|!m.is_signer && m.is_writable).count() as u8;
    let mut message = vec![signers,writable_signers,writable_non_signers,order.len() as u8];
    for m in &order {message.extend(m.pubkey.as_ref());}
    message.extend([1,positions[0],positions.len() as u8]); message.extend(&positions);
    message.extend((inner.data.len() as u16).to_le_bytes()); message.extend(&inner.data); message.push(0);
    let chunk = 800.min(message.len());
    let mut args = vec![index as u8,0]; args.extend(Sha256::digest(&message));
    args.extend((message.len() as u16).to_le_bytes()); sized(&mut args,&message[..chunk]);
    let create = squad_ix("transaction_buffer_create",vec![meta(a.multisig,false,false),meta(buffer,true,false),
        meta(proposer,false,true),meta(a.fee.pubkey(),true,true),meta(system::id(),false,false)],&args);
    take_step(bank,a,&format!("{name}_buffer"),vec![create],&[initiator],None,false,true);
    let deposit = bank.get_balance(&buffer);
    if chunk < message.len() {
        let mut args=Vec::new(); sized(&mut args,&message[chunk..]);
        let extend=squad_ix("transaction_buffer_extend",vec![meta(a.multisig,false,false),meta(buffer,true,false),
            meta(proposer,false,true)],&args);
        take_step(bank,a,&format!("{name}_extend"),vec![extend],&[initiator],None,false,true);
    }
    let mut args=vec![0,0]; sized(&mut args,&[0;6]); args.push(0);
    let save = squad_ix("vault_transaction_create_from_buffer",vec![meta(a.multisig,true,false),meta(tx,true,false),
        meta(proposer,false,true),meta(a.fee.pubkey(),true,true),meta(system::id(),false,false),meta(buffer,true,false),
        meta(proposer,true,true)],&args);
    let refund_before=bank.get_balance(&proposer);
    take_step(bank,a,&format!("{name}_store"),vec![save],&[initiator],None,false,true);
    assert!(bank.get_account(&buffer).is_none()); assert_eq!(bank.get_balance(&proposer)-refund_before,deposit);
    let tx_account=bank.get_account(&tx).unwrap();
    assert_eq!(*tx_account.owner(),SQUADS); assert!(tx_account.data().windows(inner.data.len()).any(|w|w==inner.data));
    let mut args=index.to_le_bytes().to_vec(); args.push(0);
    let proposal_create=squad_ix("proposal_create",vec![meta(a.multisig,false,false),meta(proposal,true,false),
        meta(proposer,false,true),meta(a.fee.pubkey(),true,true),meta(system::id(),false,false)],&args);
    take_step(bank,a,&format!("{name}_proposal"),vec![proposal_create],&[initiator],None,false,true);
    for (i,guardian) in a.guardians[..4].iter().enumerate() {
        let approve=squad_ix("proposal_approve",vec![meta(a.multisig,false,false),meta(guardian.pubkey(),true,true),
            meta(proposal,true,false)],&[0]);
        take_step(bank,a,&format!("{name}_vote{}",i+1),vec![approve],&[guardian],None,false,true);
        let account=bank.get_account(&proposal).unwrap();
        assert_eq!(account.data()[48],if i==3{3}else{1});
        assert_eq!(u32::from_le_bytes(account.data()[58..62].try_into().unwrap()),(i+1) as u32);
        let voters:Vec<_>=account.data()[62..62+(i+1)*32].chunks_exact(32).collect();
        for signer in &a.guardians[..=i] { assert!(voters.contains(&signer.pubkey().as_ref())); }
    }
    emit(json!({"kind":"governance","name":name,"index":index.to_string(),"proposal":proposal.to_string(),"transaction":tx.to_string(),
        "votes":4,"membership":6,"bufferRentLamports":deposit,"bufferRefundToCreatorLamports":deposit,
        "proposalRentLamports":bank.get_balance(&proposal),"storedActionRentLamports":bank.get_balance(&tx),
        "storedActionHash":digest(tx_account.data()),"authorizedDataHash":digest(&inner.data)}));
    let mut outer=vec![meta(a.multisig,false,false),meta(proposal,true,false),meta(tx,false,false),meta(proposer,false,true)];
    outer.extend(order.into_iter().map(|mut m|{ if m.pubkey==a.vault{m.is_signer=false;} if m.pubkey==proposal{m.is_writable=true;} m }));
    squad_ix("vault_transaction_execute",outer,&[])
}

fn row(bank:&Bank,k:Pubkey)->Value {
    let a=bank.get_account(&k).unwrap();
    json!({"address":k.to_string(),"owner":a.owner().to_string(),"lamports":a.lamports().to_string(),
        "executable":a.executable(),"dataBase64":STANDARD.encode(a.data())})
}
fn state(bank:&Bank,a:&Actors,targets:&[Pubkey;16],name:&str) {
    emit(json!({"kind":"state","name":name,"config":row(bank,targets[0]),"distribution":row(bank,targets[1]),
        "operationalLamports":bank.get_balance(&targets[11]).to_string(),
        "rentPayerLamports":bank.get_balance(&a.rent.pubkey()).to_string()}));
}
fn capture(bank:&Bank, targets:&[Pubkey;16], extra:&[Pubkey])->BTreeMap<Pubkey,AccountSharedData> {
    targets.iter().chain(extra).map(|k|(*k,bank.get_account(k).unwrap())).collect()
}
fn assert_technical(bank:&Bank, targets:&[Pubkey;16], amount:u64) {
    let config=bank.get_account(&targets[0]).unwrap(); let data=config.data();
    assert_eq!(&data[8..11],&[2,1,0]);
    assert_eq!(&data[756..758],&[0,0]);
    for i in 0..5 { assert_eq!(u64_at(data,934+8*i),amount); assert_eq!(bank.get_balance(&targets[9+i]),amount); }
    assert!(data[974..].iter().all(|x|*x==0));
}
fn token_bytes(owner:Pubkey,units:u64)->Vec<u8> {
    let mut bytes=vec![0;165]; bytes[..32].copy_from_slice(JITO_MINT.as_ref());
    bytes[32..64].copy_from_slice(owner.as_ref()); bytes[64..72].copy_from_slice(&units.to_le_bytes());
    bytes[108]=1; bytes
}

#[test]
fn fresh_piv1_v2_genesis() {
    assert_eq!(env::var("RAYON_NUM_THREADS").as_deref(),Ok("1"));
    let manifest=fs::read(env::var("PIV_GENESIS_INPUTS").unwrap()).unwrap();
    assert_eq!(digest(&manifest),env::var("PIV_GENESIS_INPUTS_SHA256").unwrap());
    let inputs:Value=serde_json::from_slice(&manifest).unwrap();
    let list=key(&inputs["config"]["list"]); let reserve=key(&inputs["config"]["reserve"]); let manager=key(&inputs["config"]["manager"]);
    assert_eq!(key(&inputs["config"]["pool"]),JITO_POOL); assert_eq!(key(&inputs["config"]["mint"]),JITO_MINT);
    let pool=observed(&inputs,JITO_POOL);
    let (total,supply,epoch)=(u64_at(pool.data(),258),u64_at(pool.data(),266),u64_at(pool.data(),274));
    assert_eq!(inputs["config"]["epoch"].as_u64(),Some(epoch));
    let units=(1_500_000_000u128*u128::from(supply)/u128::from(total)) as u64;
    let book=(u128::from(units)*u128::from(total)/u128::from(supply)) as u64;
    let a=Actors::new(); let targets=topology(&a);
    let source=Pubkey::new_unique();
    let authority=pda(PROGRAM,&[b"authority"]).0;
    let alt=AddressLookupTableAccount{key:pda(address_lookup_table::id(),&[a.create.as_ref(),&0u64.to_le_bytes()]).0,addresses:targets.to_vec()};
    let mut alt_data=1u32.to_le_bytes().to_vec(); alt_data.extend(u64::MAX.to_le_bytes()); alt_data.extend(0u64.to_le_bytes());
    alt_data.extend([0,1]);alt_data.extend(a.create.as_ref());alt_data.extend([0;2]);
    for k in &alt.addresses {alt_data.extend(k.as_ref());}
    let mut g=GenesisConfig{creation_time:inputs["config"]["unixTimestamp"].as_i64().unwrap(),..Default::default()};
    g.rent.lamports_per_byte=5080; let rent=g.rent.clone();
    assert_eq!(rent.minimum_balance(0),650240);
    g.fee_rate_governor.lamports_per_signature=5000;g.fee_rate_governor.target_lamports_per_signature=5000;
    g.fee_rate_governor.target_signatures_per_slot=0;g.fee_rate_governor.min_lamports_per_signature=5000;
    g.fee_rate_governor.max_lamports_per_signature=5000;g.fee_rate_governor.burn_percent=50;
    for who in [&a.fee,&a.rent,&a.contributor].into_iter().chain(a.guardians.iter()) {
        insert(&mut g,who.pubkey(),account(system::id(),1_000_000_000,&[],false));
    }
    let multisig=a.multisig_data();
    insert(&mut g,a.multisig,account(SQUADS,rent.minimum_balance(multisig.len()),&multisig,false));
    insert(&mut g,alt.key,account(address_lookup_table::id(),rent.minimum_balance(alt_data.len()),&alt_data,false));
    for k in [JITO_POOL,list,reserve,JITO_MINT,manager] { insert(&mut g,k,observed(&inputs,k)); }
    let original_holder=token_bytes(a.contributor.pubkey(),units);
    insert(&mut g,source,account(TOKEN,rent.minimum_balance(165),&original_holder,false));
    for (p,sha,size) in [
        (SQUADS,"95d0811d25f9df7d21f230e34ce93fcb37d22a9b4fe1365f07cd339e9ad165f1",980096),
        (TOKEN,"8190d3f7ceb6cb7a7a8d8924bff89f9f611e15ce1f806f2b6237f3311a98f697",108600),
        (JITO,"6c0cb7ef5da80a23c3ad85d7fec566f3cc17b815d428daec84bb916441a2a143",1106080),
    ] { install_observed(&mut g,&inputs,p,sha,size); }
    let piv=fs::read(env::var("PIV_GENESIS_ELF").unwrap()).unwrap();
    assert_eq!((piv.len(),digest(&piv)),(967696,ELF_SHA.into())); install(&mut g,PROGRAM,Some(a.vault),&piv);
    solana_runtime::genesis_utils::activate_feature(&mut g,FEATURE);
    let genesis=g.hash();
    let directory=env::var("PIV_GENESIS_BANK_DIR").unwrap(); let directory=Path::new(&directory);
    fs::create_dir(directory).unwrap();
    let mut settings=BankTestConfig::default(); let db=&mut settings.accounts_db_config;
    db.num_foreground_threads=NonZeroUsize::new(1);db.num_background_threads=NonZeroUsize::new(1);
    db.read_cache_limit_bytes=Some((2*1024*1024,4*1024*1024));db.read_cache_num_shards=Some(16);db.read_cache_evict_sample_size=Some(4);
    db.write_cache_limit_bytes=Some(8*1024*1024);
    let index=db.index.as_mut().unwrap(); index.bins=Some(2);index.num_flush_threads=NonZeroUsize::new(1);index.num_initial_accounts=Some(128);
    let leader=SlotLeader{id:Pubkey::new_unique(),vote_address:Pubkey::new_unique()};
    let root=Bank::new_with_paths_for_tests(&g,Some(settings),vec![directory.to_path_buf()],Some(leader));
    let (root,forks)=root.wrap_with_bank_forks_for_tests();
    let bank=Bank::new_from_parent_with_bank_forks(&forks,root,leader,1);
    let mut clock=bank.clock();clock.epoch=epoch;clock.leader_schedule_epoch=epoch;bank.set_sysvar_for_tests(&clock);
    assert_eq!(bank.clock().epoch,epoch);assert_eq!(bank.feature_set.active().len(),1);
    assert!(targets.iter().all(|k|bank.get_account(k).is_none()),"fresh targets");
    let floor=rent.minimum_balance(0);
    let init_rent=SIZES.into_iter().map(|size|rent.minimum_balance(size)).sum::<u64>();
    let mr2_rent=rent.minimum_balance(924)-rent.minimum_balance(891);
    let mr3_rent=rent.minimum_balance(1199)-rent.minimum_balance(924);
    let rent_by_size=[0,84,165,210,891,924,1014,1199].into_iter()
        .map(|n|(n.to_string(),rent.minimum_balance(n).to_string())).collect::<BTreeMap<_,_>>();
    let approvers=a.guardians[..4].iter().map(|s|s.pubkey().to_string()).collect::<Vec<_>>();
    emit(json!({"kind":"fixture","piv1ElfSha256":ELF_SHA,"genesis":genesis.to_string(),"program":PROGRAM.to_string(),
        "createKey":a.create.to_string(),"multisig":a.multisig.to_string(),"vault":a.vault.to_string(),
        "guardians":a.guardians.iter().map(|s|s.pubkey().to_string()).collect::<Vec<_>>(),
        "creator":approvers[0],"executor":approvers[0],"approvers":approvers,"bufferIndex":1,"timeLockSeconds":0,"staleTransactionIndex":"0",
        "initialTargetPrefundLamports":"0","operationalBeforeMR02Lamports":(floor+mr2_rent).to_string(),
        "migration3PayerBeforeLamports":(bank.get_balance(&a.rent.pubkey())-init_rent).to_string(),"anchorUnixTimestamp":"0",
        "recipients":a.recipients.map(|s|s.to_string()),"feePayer":a.fee.pubkey().to_string(),"rentPayer":a.rent.pubkey().to_string(),
        "contributor":a.contributor.pubkey().to_string(),"tokenSource":source.to_string(),"pool":JITO_POOL.to_string(),
        "mint":JITO_MINT.to_string(),"list":list.to_string(),"reserve":reserve.to_string(),"manager":manager.to_string(),
        "tokenProgram":TOKEN.to_string(),"jitoProgram":JITO.to_string(),"alt":alt.key.to_string(),"altAuthority":a.create.to_string(),
        "altRecentSlot":"0","altAddresses":targets.map(|s|s.to_string()),"slot":bank.slot().to_string(),
        "clock":{"epoch":epoch.to_string(),"unixTimestamp":clock.unix_timestamp.to_string()},
        "blockhash":bank.last_blockhash().to_string(),"lastValidBlockHeight":bank.get_blockhash_last_valid_block_height(&bank.last_blockhash()).unwrap().to_string(),
        "rentBySize":rent_by_size,
        "initRent":init_rent.to_string(),"migration2Rent":mr2_rent.to_string(),"migration3Rent":mr3_rent.to_string(),
        "migration3RentCap":mr3_rent.to_string(),"rentPayerLamports":bank.get_balance(&a.rent.pubkey()).to_string(),
        "initTransactionIndex":"1","migration3TransactionIndex":"2","initiallyPaused":false,"kifAnchorTimestamp":"0",
        "initialOperationalLamports":"0","poolTotalLamports":total.to_string(),"poolTokenSupply":supply.to_string(),
        "contributionUnits":units.to_string(),"recognizedBookValue":book.to_string(),
        "seededMultisigAndAlt":true,"localContributedTokenBalance":true}));
    let initializer=|idx| {
        let mut keys=governance_metas(&a,idx);
        keys.extend(targets.map(|k|meta(k,true,false)));
        for k in [JITO,JITO_POOL,list,reserve,JITO_MINT,manager] { keys.push(meta(k,false,false)); }
        keys.extend([meta(a.rent.pubkey(),true,true),meta(system::id(),false,false),meta(TOKEN,false,false)]);
        keys.extend(a.recipients.map(|k|meta(k,false,false)));
        let mut args=vec![0,0];
        for k in [JITO,JITO_POOL,list,reserve,JITO_MINT,manager,manager,a.recipients[0],a.recipients[1]] { args.extend(k.as_ref()); }
        args.extend(0i64.to_le_bytes());args.extend([0,1,2,3,4,5]);
        app_ix(b"PIV1IN02",2,keys,&args)
    };
    let before_rent=bank.get_balance(&a.rent.pubkey());
    let init=initializer(1);instruction("IN02",&init);
    let execution=authorize(&bank,&a,"init",1,&init);
    take_step(&bank,&a,"init_execute",vec![execution],&[&a.guardians[0],&a.rent],Some(&alt),true,true);
    assert_eq!(bank.get_account(&governance_keys(&a,1).1).unwrap().data()[48],5);
    assert_eq!(before_rent-bank.get_balance(&a.rent.pubkey()),init_rent);
    assert_eq!(bank.get_account(&targets[1]).unwrap().data()[8],1);
    assert_technical(&bank,&targets,floor);
    for (i,k) in targets.iter().enumerate() {
        let row=bank.get_account(k).unwrap();
        let owner=if i<9{PROGRAM}else if i<14{system::id()}else{TOKEN};
        assert_eq!((row.data().len(),row.lamports(),*row.owner()),(SIZES[i],rent.minimum_balance(SIZES[i]),owner));
    }
    state(&bank,&a,&targets,"initialized");
    let mut keys=fixed(&targets,&[1,4]);keys.push(meta(system::id(),false,false));
    let mr2=app_ix(b"PIV1MR02",1,keys,&[]);instruction("MR02",&mr2);
    let before=capture(&bank,&targets,&[]);
    take_step(&bank,&a,"round_v2_unfunded",vec![mr2.clone()],&[],None,false,false);
    assert_eq!(capture(&bank,&targets,&[]),before);
    let fund=solana_system_interface::instruction::transfer(&a.fee.pubkey(),&targets[11],mr2_rent);
    take_step(&bank,&a,"fund_operational",vec![fund],&[],None,false,true);
    // Distinguish the final MR02 from the original exact message's status-cache entry.
    bank.register_unique_recent_blockhash_for_test();
    take_step(&bank,&a,"round_v2_migrate",vec![mr2],&[],None,false,true);
    let round2=bank.get_account(&targets[1]).unwrap();assert_eq!((round2.data()[8],round2.data().len()),(2,924));
    assert_eq!(round2.lamports()-before[&targets[1]].lamports(),mr2_rent);assert_technical(&bank,&targets,floor);
    state(&bank,&a,&targets,"round_v2");
    let mut keys=governance_metas(&a,2);
    keys.extend([meta(targets[0],false,false),meta(targets[2],false,false)]);
    keys.extend(targets[3..9].iter().map(|k|meta(*k,false,false)));
    keys.push(meta(sysvar::clock::id(),false,false));
    keys.extend(fixed(&targets,&[1]).into_iter().skip(1));
    keys.extend([meta(system::id(),false,false),meta(a.rent.pubkey(),true,true)]);
    let mut args=vec![0];args.extend(Sha256::digest(round2.data()));args.extend(mr3_rent.to_le_bytes());
    let mr3=app_ix(b"PIV1MR03",1,keys,&args);instruction("MR03",&mr3);
    let execution=authorize(&bank,&a,"migration3",2,&mr3);
    let before_rent=bank.get_balance(&a.rent.pubkey());
    take_step(&bank,&a,"migration3_execute",vec![execution],&[&a.guardians[0],&a.rent],Some(&alt),true,true);
    assert_eq!(bank.get_account(&governance_keys(&a,2).1).unwrap().data()[48],5);
    assert_eq!(before_rent-bank.get_balance(&a.rent.pubkey()),mr3_rent);
    let round3=bank.get_account(&targets[1]).unwrap();assert_eq!((round3.data()[8],round3.data().len()),(3,1199));
    assert_eq!(round3.lamports()-round2.lamports(),mr3_rent);assert_technical(&bank,&targets,floor);
    state(&bank,&a,&targets,"round_v3");
    let duplicate=authorize(&bank,&a,"duplicate_init",3,&initializer(3));
    let before=capture(&bank,&targets,&[a.rent.pubkey()]);
    let rejected=take_step(&bank,&a,"duplicate_init_execute",vec![duplicate],&[&a.guardians[0],&a.rent],Some(&alt),true,false);
    assert!(rejected.log_messages.unwrap().iter().any(|l|l==&format!("Program {PROGRAM} invoke [2]")));
    assert_eq!(capture(&bank,&targets,&[a.rent.pubkey()]),before);

    // Change the local Rent sysvar to the retained lower rate. All accounts remain
    // exempt even under the Bank's original rent profile. Storage is not revenue.
    let mut lower=rent.clone();lower.lamports_per_byte=2575;bank.set_sysvar_for_tests(&lower);
    let dj=app_ix(b"PIV1DJ01",1,vec![meta(targets[0],true,false),meta(targets[1],false,false),
        meta(targets[9],false,false),meta(targets[15],true,false),meta(source,true,false),
        meta(a.contributor.pubkey(),false,true),meta(JITO_MINT,false,false),meta(TOKEN,false,false)],&units.to_le_bytes());
    let mut keys=fixed(&targets,&[0,2,3,7,8]);
    keys.extend([authority,JITO_MINT,system::id(),TOKEN,JITO,JITO_POOL,list,reserve,manager].map(|k|meta(k,false,false)));
    let ib=app_ix(b"PIV1IB01",1,keys,&[]);instruction("DJ01",&dj);instruction("IB01",&ib);
    let protected=[source,JITO_POOL,JITO_MINT,list,reserve,manager,a.contributor.pubkey(),a.rent.pubkey()];
    let before=capture(&bank,&targets,&protected);
    let mut wrong=ib.clone();wrong.accounts[17].pubkey=a.rent.pubkey();
    let rejected=take_step(&bank,&a,"reject_wrong_receiver_and_rollback",vec![dj.clone(),wrong],&[&a.contributor],None,true,false);
    assert_eq!(rejected.log_messages.unwrap().iter().filter(|l|*l==&format!("Program {TOKEN} success")).count(),1);
    assert_eq!(capture(&bank,&targets,&protected),before);
    let success=take_step(&bank,&a,"fund_and_integrate",vec![dj,ib.clone()],&[&a.contributor],None,true,true);
    assert_eq!(success.log_messages.unwrap().iter().filter(|l|*l==&format!("Program {TOKEN} success")).count(),2);
    let mut after=before.clone();
    after.get_mut(&source).unwrap().set_data_from_slice(&token_bytes(a.contributor.pubkey(),0));
    after.get_mut(&targets[14]).unwrap().set_data_from_slice(&token_bytes(authority,units));
    let mut config=after[&targets[0]].data().to_vec();
    assert!(config[758..910].iter().all(|x|*x==0),"pristine opening ledger");
    for (field,value) in [(1,book),(2,units),(9,book)] { let i=758+8*field;config[i..i+8].copy_from_slice(&value.to_le_bytes()); }
    after.get_mut(&targets[0]).unwrap().set_data_from_slice(&config);
    assert_eq!(capture(&bank,&targets,&protected),after,"exact protected opening state");
    assert_technical(&bank,&targets,floor);
    state(&bank,&a,&targets,"integrated");
    take_step(&bank,&a,"duplicate_bootstrap",vec![ib],&[],None,true,false);
    assert_eq!(capture(&bank,&targets,&protected),after);
    emit(json!({"kind":"complete","status":"PASS","squads":true,"verifiedLocalSignatures":true,
        "roundVersions":[1,2,3],"openingHwmLamports":book,"openingYieldLamports":0,"kifLiabilityLamports":0,
        "principalTokenUnits":units,"recognizedPendingSolLamports":0,"recognizedPendingTokenUnits":0,
        "operationalEconomicLamports":0,"originalTechnicalBasisPerVault":floor,"lowerRuntimeRent":lower.minimum_balance(0),
        "staleBootstrapRejected":true,"wrongReceiverRollback":true,"unfundedMigrationRejected":true,
        "programCodeChanged":false,"publicTransactions":0}));
}
