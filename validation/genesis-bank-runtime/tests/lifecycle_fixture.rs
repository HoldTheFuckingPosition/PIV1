//! Literal ABI and synthetic external genesis only. No PIV1 host library/model.
//! Every PIV1 economic account is absent until the real initializer executes.
#[allow(dead_code)]
#[path = "genesis_fixture.rs"]
pub mod genesis;
pub use genesis::{key, authority, program_data, World, PROGRAM, SQUADS, TOKEN, JITO_PROGRAM, JITO_POOL, JITO_MINT, STAKE};
use solana_account::{AccountSharedData, ReadableAccount, WritableAccount};
use solana_instruction::{Instruction, AccountMeta};
use solana_pubkey::Pubkey;
use solana_sdk_ids::{system_program, sysvar};
pub const SOL: u64 = 1_000_000_000;
pub const STAKE_RENT: u64 = 2_282_880;
pub fn target(i: usize) -> Pubkey { World::derive(i).0 }
pub fn withdraw() -> Pubkey { Pubkey::find_program_address(&[JITO_POOL.as_ref(), b"withdraw"], &JITO_PROGRAM).0 }
pub fn source(i: u8) -> Pubkey { Pubkey::find_program_address(&[key(210+i).as_ref(), JITO_POOL.as_ref()], &JITO_PROGRAM).0 }
pub fn transient(i: u8) -> Pubkey { Pubkey::find_program_address(&[b"transient", key(210+i).as_ref(), JITO_POOL.as_ref(), &0_u64.to_le_bytes()], &JITO_PROGRAM).0 }
pub fn leg(i: u64) -> Pubkey { Pubkey::find_program_address(&[b"withdrawal-leg", &0_u64.to_le_bytes(), &i.to_le_bytes()], &PROGRAM).0 }
pub fn stake(i: u64) -> Pubkey { Pubkey::find_program_address(&[b"withdrawal-stake", &0_u64.to_le_bytes(), &i.to_le_bytes()], &PROGRAM).0 }
pub fn account(owner: Pubkey, lamports: u64, data: Vec<u8>) -> AccountSharedData {
    let mut a = AccountSharedData::new(lamports, data.len(), &owner); a.set_data_from_slice(&data); a.set_rent_epoch(u64::MAX); a
}
pub fn put64(b: &mut [u8], p: usize, v: u64) { b[p..p+8].copy_from_slice(&v.to_le_bytes()); }
pub fn u64at(b: &[u8], p: usize) -> u64 { u64::from_le_bytes(b[p..p+8].try_into().unwrap()) }
pub fn append(b: &mut Vec<u8>, v: u64) { b.extend(v.to_le_bytes()); }
pub fn token(owner: Pubkey, amount: u64) -> Vec<u8> { let mut b=vec![0;165]; b[..32].copy_from_slice(JITO_MINT.as_ref()); b[32..64].copy_from_slice(owner.as_ref()); put64(&mut b,64,amount); b[108]=1;b }
pub fn stake_state(vote: Option<Pubkey>, delegated: u64) -> Vec<u8> {
    let mut b=(if vote.is_some(){2_u32}else{1_u32}).to_le_bytes().to_vec(); append(&mut b,STAKE_RENT);
    b.extend(withdraw().as_ref()); b.extend(withdraw().as_ref()); b.extend([0;48]);
    if let Some(vote)=vote { b.extend(vote.as_ref()); append(&mut b,delegated); append(&mut b,u64::MAX);
        append(&mut b,u64::MAX); b.extend(0.25_f64.to_le_bytes()); append(&mut b,0); b.push(0); }
    b.resize(200,0); b
}
pub fn external_world() -> (World, Vec<(Pubkey,AccountSharedData)>) {
    let mut w=World::production(false,false);
    let mut p=vec![1]; for k in [key(1),key(2),Pubkey::find_program_address(&[JITO_POOL.as_ref(),b"deposit"],&JITO_PROGRAM).0] {p.extend(k.as_ref());}
    p.push(Pubkey::find_program_address(&[JITO_POOL.as_ref(),b"withdraw"],&JITO_PROGRAM).1);
    for k in [key(201),key(202),JITO_MINT,key(203),TOKEN] {p.extend(k.as_ref());}
    for n in [1000*SOL,1000*SOL,0] {append(&mut p,n);} p.extend([0;48]);
    let fee=|p:&mut Vec<u8>|{append(p,1);append(p,0);}; // denominator then numerator
    fee(&mut p);p.extend([0,0,0]);fee(&mut p);fee(&mut p);p.extend([0,0,0]);fee(&mut p);p.extend([0,0]);fee(&mut p);p.push(0);
    append(&mut p,1000*SOL);append(&mut p,1000*SOL);p.resize(611,0);
    let mut list=vec![2];list.extend(2_u32.to_le_bytes());list.extend(2_u32.to_le_bytes());
    for i in 0..2 {for n in [6*SOL+STAKE_RENT,0,0,0]{append(&mut list,n);}list.extend([0;8]);list.push(0);list.extend(key(210+i).as_ref());}
    assert_eq!(list.len(),155);
    let mut mint=vec![0;82];mint[..4].copy_from_slice(&1_u32.to_le_bytes());mint[4..36].copy_from_slice(withdraw().as_ref());put64(&mut mint,36,1000*SOL);mint[44]=9;mint[45]=1;
    for (key,data,balance) in [(JITO_POOL,p,0),(key(201),list,0),(key(202),stake_state(None,0),1000*SOL-2*(6*SOL+STAKE_RENT)+STAKE_RENT),
        (JITO_MINT,mint,0),(key(203),token(key(1),980*SOL),0),(key(204),token(key(1),0),0)] {
        let r=w.roles.iter_mut().find(|r|r.key==key).unwrap();r.data=data;r.lamports=if balance==0{(128+r.data.len() as u64)*6960}else{balance};
    }
    let mut extra=vec![(key(220),account(system_program::id(),1000*SOL,vec![])),
        (key(221),account(TOKEN,(128+165)*6960,token(key(220),20*SOL)))];
    for i in 0..2 {extra.push((source(i),account(STAKE,6*SOL+STAKE_RENT,stake_state(Some(key(210+i)),6*SOL))));}
    // Distinct guardian destinations exist; their public bytes are unsigned local fixtures.
    for i in 91..97 {extra.push((key(i),account(system_program::id(),SOL,vec![])));}
    (w,extra)
}
pub fn meta(k:Pubkey,w:bool,s:bool)->AccountMeta{AccountMeta{pubkey:k,is_writable:w,is_signer:s}}
pub fn data(selector:&[u8;8],args:&[u8])->Vec<u8>{let mut d=selector.to_vec();d.push(1);d.extend(args);d}
pub fn piv(selector:&[u8;8],args:&[u8],accounts:Vec<AccountMeta>)->Instruction{Instruction{program_id:PROGRAM,accounts,data:data(selector,args)}}
pub fn base(writable:&[usize])->Vec<AccountMeta>{
    let keys=[target(0),target(1),target(9),target(10),target(11),target(12),target(13),target(14),target(15),authority(),JITO_MINT,system_program::id(),TOKEN,JITO_PROGRAM,JITO_POOL,key(201),key(202),key(203),key(204)];
    keys.into_iter().enumerate().map(|(i,k)|meta(k,writable.contains(&i),false)).collect()
}
pub fn deposit_sol(amount:u64)->Instruction{piv(b"PIV1DS01",&amount.to_le_bytes(),vec![meta(target(0),true,false),meta(target(1),false,false),meta(target(9),true,false),meta(target(15),false,false),meta(key(220),true,true),meta(system_program::id(),false,false)])}
pub fn deposit_token(amount:u64)->Instruction{piv(b"PIV1DJ01",&amount.to_le_bytes(),vec![meta(target(0),true,false),meta(target(1),false,false),meta(target(9),false,false),meta(target(15),true,false),meta(key(221),true,false),meta(key(220),false,true),meta(JITO_MINT,false,false),meta(TOKEN,false,false)])}
pub fn bootstrap()->Instruction{piv(b"PIV1IB01",&[],base(&[0,2,3,7,8]))}
pub fn principal_deposit(amount:u64)->Instruction{let mut a=base(&[0,3,7,10,14,16,17,18]);a.push(meta(withdraw(),false,false));let mut args=amount.to_le_bytes().to_vec();args.extend(amount.to_le_bytes());piv(b"PIV1SP01",&args,a)}
pub fn prepare()->Instruction{let mut a=base(&[0,1,2,3,5]);a.push(meta(target(2),false,false));for i in 3..9{a.push(meta(target(i),false,false));}a.push(meta(sysvar::clock::id(),false,false));a.push(meta(STAKE,false,false));a.push(meta(source(0),false,false));piv(b"PIV1PW01",&0_u32.to_le_bytes(),a)}
pub fn initiate(i:u64)->Instruction{let mut a=base(&[0,1,2,4,7,10,14,15,17]);a.extend([meta(sysvar::clock::id(),false,false),meta(STAKE,false,false),meta(source(i as u8),true,false),meta(withdraw(),false,false),meta(leg(i),true,false),meta(stake(i),true,false)]);piv(b"PIV1IL01",&(i as u32).to_le_bytes(),a)}
pub fn finalize(i:u64)->Instruction{let mut a=base(&[0,1,2,4,5]);a.extend([meta(sysvar::clock::id(),false,false),meta(STAKE,false,false),meta(sysvar::stake_history::id(),false,false),meta(leg(i),true,false),meta(stake(i),true,false)]);piv(b"PIV1FL01",&i.to_le_bytes(),a)}
pub fn settle(w:&World)->Instruction{let mut a=base(&[0,1,3,5,6]);a.push(meta(sysvar::clock::id(),false,false));for k in w.recipients{a.push(meta(k,true,false));}for i in 3..9{a.push(meta(target(i),true,false));}piv(b"PIV1SD01",&[],a)}
pub fn integrate()->Instruction{let mut a=base(&[0,1,2,3,5,7,8]);a.push(meta(sysvar::clock::id(),false,false));piv(b"PIV1IP01",&[],a)}
pub fn heartbeat(w:&World)->Instruction{let mut a=vec![meta(target(0),false,false),meta(target(2),false,false)];for i in 3..9{a.push(meta(target(i),i==3,false));}a.extend([meta(sysvar::clock::id(),false,false),meta(key(96),false,true),meta(PROGRAM,false,false),meta(program_data(PROGRAM),false,false),meta(w.roles[2].key,false,false)]);piv(b"PIV1HB01",&[0,7],a)}
pub fn claim(amount:u64)->Instruction{let mut d=vec![253,151,172,11,202,77,118,170];append(&mut d,amount);append(&mut d,0);Instruction{program_id:PROGRAM,data:d,accounts:vec![meta(target(0),true,false),meta(target(3),true,false),meta(target(13),true,false),meta(key(96),true,true),meta(system_program::id(),false,false)]}}
pub fn update_pool()->Instruction{Instruction{program_id:JITO_PROGRAM,data:vec![7],accounts:vec![meta(JITO_POOL,true,false),meta(withdraw(),false,false),meta(key(201),true,false),meta(key(202),false,false),meta(key(203),true,false),meta(JITO_MINT,true,false),meta(TOKEN,false,false)]}}
pub fn update_validators()->Instruction{let mut a=vec![meta(JITO_POOL,false,false),meta(withdraw(),false,false),meta(key(201),true,false),meta(key(202),true,false),meta(sysvar::clock::id(),false,false),meta(sysvar::stake_history::id(),false,false),meta(STAKE,false,false)];for i in 0..2{a.extend([meta(source(i),true,false),meta(transient(i),true,false)]);}let mut d=vec![6];d.extend(0_u32.to_le_bytes());d.push(1);Instruction{program_id:JITO_PROGRAM,accounts:a,data:d}}

#[derive(Clone,Debug,PartialEq,Eq)]
pub struct Config{pub prefix:Vec<u8>,pub prepared:Option<u64>,pub insufficient:Option<u64>,pub n:[u64;19],pub tail:Vec<u8>}
impl Config{
    pub fn read(a:&AccountSharedData)->Self{let b=a.data();assert_eq!(b.len(),1014);let mut p=756;let prepared=option(b,&mut p);let insufficient=option(b,&mut p);let n=core::array::from_fn(|_|{let v=u64at(b,p);p+=8;v});Self{prefix:b[..756].to_vec(),prepared,insufficient,n,tail:b[p..p+88].to_vec()}}
    pub fn bytes(&self)->Vec<u8>{let mut b=self.prefix.clone();for v in [self.prepared,self.insufficient]{b.push(u8::from(v.is_some()));if let Some(v)=v{append(&mut b,v);}}for n in self.n{append(&mut b,n);}b.extend(&self.tail);b.resize(1014,0);b}
}
fn option(b:&[u8],p:&mut usize)->Option<u64>{let tag=b[*p];*p+=1;assert!(tag<=1);if tag==0{None}else{let v=u64at(b,*p);*p+=8;Some(v)}}
#[derive(Clone,Debug)]
pub struct Round{pub state:u8,pub seq:u64,pub snapshot:[u64;27],pub cumulative:[u64;16],pub settlement:[u64;15]}
impl Round{pub fn read(a:&AccountSharedData)->Self{let b=a.data();assert_eq!(b.len(),891);let state=b[11];let seq=u64at(b,14);let mut p=23;if b[22]==1{p+=88;}let snapshot=core::array::from_fn(|_|{let n=u64at(b,p);p+=8;n});p+=2;let cumulative=core::array::from_fn(|_|{let n=u64at(b,p);p+=8;n});p+=96+8+192+2+16;let settlement=core::array::from_fn(|_|{let n=u64at(b,p);p+=8;n});Self{state,seq,snapshot,cumulative,settlement}}}
pub fn units(a:&AccountSharedData)->u64{u64at(a.data(),64)}
pub fn pool_ratio(a:&AccountSharedData)->(u64,u64,u64){(u64at(a.data(),258),u64at(a.data(),266),u64at(a.data(),274))}
pub fn value(q:u64,t:u64,s:u64)->u64{(u128::from(q)*u128::from(t)/u128::from(s)) as u64}

/// Full independent first-round encoding; every field and padding byte is set.
pub fn prepared_bytes(w:&World,time:u64,slot:u64,epoch:u64,gross:u64,target_q:u64)->Vec<u8>{
    let h=value(gross,5900,10000);let t=value(gross,1950,10000);let k=value(gross,200,10000);let compound=value(gross,1950,10000);let dust=gross-h-t-k-compound;
    let outgoing=h+t+k;let converted=value(target_q,1200*SOL,1100*SOL);let conversion=outgoing-SOL/10-converted;
    let minimum=(u128::from(SOL)*u128::from(1100*SOL)).div_ceil(u128::from(1200*SOL)) as u64;let legs=target_q/minimum;
    let proposed=compound+dust+conversion;let mut b=w.state_bytes()[1][..14].to_vec();b[11]=1;append(&mut b,0);b.push(0);
    for n in [time,slot,epoch,119*SOL,119*SOL,0,119*SOL+gross,1200*SOL,1100*SOL,0,1,
        gross,0,h,compound,t,k,dust,outgoing,SOL/10,SOL/10,conversion,target_q,minimum,legs,value(converted-(legs-1),9999,10000),119*SOL+proposed]{append(&mut b,n);}
    b.extend(1_u16.to_le_bytes());for i in 0..16{append(&mut b,if i==14{SOL/10}else if i==15{outgoing}else{0});}
    for k in [w.recipients[0],w.recipients[1],target(2)]{b.extend(k.as_ref());}append(&mut b,0);for i in 0..6{b.extend(key(96-i).as_ref());}b.extend([1,1]);append(&mut b,0);append(&mut b,0);
    append(&mut b,proposed);append(&mut b,119*SOL+proposed);b.extend([0;13*8]);assert_eq!(b.len(),803);b.resize(891,0);b
}
pub fn round_cumulative(b:&mut[u8],index:usize,n:u64){put64(b,241+index*8,n);}
pub fn round_settlement(b:&mut[u8],index:usize,n:u64){put64(b,683+index*8,n);}
pub const LEG_SIZE:usize=263;
pub fn leg_bytes(i:u64,epoch:u64,total:u64,supply:u64,q:u64,native:u64,snapshot_min:u64)->Vec<u8>{
    use sha2::{Digest,Sha256};let mut b=Sha256::digest(b"account:WithdrawalLeg")[..8].to_vec();
    let meta_bump=Pubkey::find_program_address(&[b"withdrawal-leg",&0_u64.to_le_bytes(),&i.to_le_bytes()],&PROGRAM).1;
    let stake_bump=Pubkey::find_program_address(&[b"withdrawal-stake",&0_u64.to_le_bytes(),&i.to_le_bytes()],&PROGRAM).1;
    b.extend([1,meta_bump,stake_bump,1,1,0]);append(&mut b,0);append(&mut b,i);b.extend((i as u32).to_le_bytes());b.extend(0_u32.to_le_bytes());b.extend(key(210+i as u8).as_ref());b.extend(source(i as u8).as_ref());
    let current_min=(u128::from(SOL)*u128::from(supply)).div_ceil(u128::from(total)) as u64;
    for n in [epoch,total,supply,0,1,snapshot_min.max(current_min),q,0,q,native,native,value(native,9999,10000).max(SOL),STAKE_RENT,(128+LEG_SIZE as u64)*6960]{append(&mut b,n);}
    b.push(0);b.extend([0;40]);assert_eq!(b.len(),255);b.resize(LEG_SIZE,0);b
}
