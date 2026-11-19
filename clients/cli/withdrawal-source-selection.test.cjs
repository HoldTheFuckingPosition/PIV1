'use strict';

const assert=require('node:assert/strict');
const fs=require('node:fs');
const os=require('node:os');
const path=require('node:path');
const {spawnSync}=require('node:child_process');
const test=require('node:test');
const {IDS,PublicKey}=require('./initializer-plan.cjs');
const {validateSnapshot}=require('./state-inspect.cjs');
const {SCHEMA,selectSource}=require('./withdrawal-source-selection.cjs');

const M=1_000_000_000n,R=2_282_880n,EPOCH=40n,U64=(1n<<64n)-1n;
const Q=200_000_000_000n;
const CLOCK='SysvarC1ock11111111111111111111111111111111';
const RENT='SysvarRent111111111111111111111111111111111';
const SYSVAR='Sysvar1111111111111111111111111111111111111';
const key=bytes=>new PublicKey(bytes);
const votes=[9,11,13].map(tag=>Buffer.alloc(32,tag));
const [authority,bump]=PublicKey.findProgramAddressSync([key(IDS.pool).toBuffer(),Buffer.from('withdraw')],key(IDS.poolProgram));
const raw=(address,owner,data,balance=1n)=>({address,owner,executable:false,lamports:String(balance),dataBase64:data.toString('base64')});
const clone=value=>structuredClone(value);
const replace=(account,change)=>{
  const bytes=Buffer.from(account.dataBase64,'base64');
  change(bytes);
  account.dataBase64=bytes.toString('base64');
};

function fixture({preferred=null,capacities=[M,M],target=2n*M,slot=2n,leg=0n,assigned=0n,takenIndex=0,
    receipts={[takenIndex]:assigned},ratios=[Q-assigned,Q-assigned],fee=[0n,0n],epoch=EPOCH}={}) {
  // Existing literal Config identity/topology, and an independent active-round layout.
  const state=clone(require('./state-inspect.fixture.json'));
  state.context={sourceKind:'synthetic-literal',sourceLabel:'Withdrawal source selection test',slot:String(slot)};
  const c=validateSnapshot(state).decoded.config.value;
  const round=Buffer.alloc(891);
  Buffer.from([104,51,125,187,226,55,209,99,1,Buffer.from(state.distribution.dataBase64,'base64')[9],1,1]).copy(round);
  const fields={23:1n,31:1n,39:epoch,79:Q,87:Q,167:target,199:target,207:M,215:target/M,
    223:target-(target/M-1n)-target/10000n,231:M,241:assigned,257:assigned,265:assigned,
    273:assigned,329:leg,337:leg,691:M};
  for(const [offset,value] of Object.entries(fields))round.writeBigUInt64LE(value,Number(offset));
  round.writeUInt16LE(c.configured_slippage_bps,239);
  state.distribution.dataBase64=round.toString('base64');
  const clock=Buffer.alloc(40);
  clock.writeBigUInt64LE(slot,0);
  clock.writeBigUInt64LE(epoch,16);
  clock.writeBigInt64LE(2n,32);
  const rent=Buffer.alloc(17);
  rent.writeBigUInt64LE(3480n);
  rent.writeDoubleLE(2,8);
  rent[16]=50;
  const pool=Buffer.alloc(435+(preferred===null?0:32));
  pool[0]=1;
  pool[97]=bump;
  for(const [offset,address] of [[98,c.validator_list],[130,c.reserve_stake],[162,IDS.mint],
    [194,c.manager_fee_account],[226,IDS.token]])key(address).toBuffer().copy(pool,offset);
  pool.writeBigUInt64LE(ratios[0],258);
  pool.writeBigUInt64LE(ratios[1],266);
  pool.writeBigUInt64LE(epoch,274);
  if(preferred!==null){pool[348]=1;votes[preferred].copy(pool,349);}
  pool.writeBigUInt64LE(fee[1],365+(preferred===null?0:32));
  pool.writeBigUInt64LE(fee[0],373+(preferred===null?0:32));
  const mint=Buffer.alloc(82);
  mint.writeUInt32LE(1);
  authority.toBuffer().copy(mint,4);
  mint.writeBigUInt64LE(ratios[1],36);
  mint[44]=9;
  mint[45]=1;
  // Large allocation with opaque unused bytes and just the populated count.
  const list=Buffer.alloc(730_009,0xaa);
  list[0]=2;
  list.writeUInt32LE(10_000,1);
  list.writeUInt32LE(capacities.length,5);
  const sourceAccounts=capacities.map((capacity,index)=>{
    const remainingCapacity=capacity-(receipts[index]??0n);
    const delegated=M+remainingCapacity,balance=R+delegated;
    const suffix=Buffer.alloc(4);
    suffix.writeUInt32LE(index===1?123_456:0);
    const seeds=[votes[index],key(IDS.pool).toBuffer(),...(index===1?[suffix]:[])];
    const source=PublicKey.findProgramAddressSync(seeds,key(IDS.poolProgram))[0].toBase58();
    const at=9+73*index;
    list.writeBigUInt64LE(balance,at);
    list.writeBigUInt64LE(0n,at+8);
    list.writeBigUInt64LE(epoch,at+16);
    suffix.copy(list,at+36);
    list[at+40]=0;
    votes[index].copy(list,at+41);
    const stake=Buffer.alloc(200);
    stake.writeUInt32LE(2);
    stake.writeBigUInt64LE(R,4);
    authority.toBuffer().copy(stake,12);
    authority.toBuffer().copy(stake,44);
    votes[index].copy(stake,124);
    stake.writeBigUInt64LE(delegated,156);
    stake.writeBigUInt64LE(epoch-1n,164);
    stake.writeBigUInt64LE(U64,172);
    return {validatorIndex:String(index),account:raw(source,IDS.stake,stake,balance)};
  });
  return {schema:SCHEMA,state,clock:raw(CLOCK,SYSVAR,clock),rent:raw(RENT,SYSVAR,rent),minimumDelegationLamports:String(M),
    minimumDelegationEpoch:String(epoch),pool:raw(IDS.pool,IDS.poolProgram,pool),mint:raw(IDS.mint,IDS.token,mint),
    validatorList:raw(c.validator_list,IDS.poolProgram,list),sourceAccounts};
}

test('selects one leg from an authenticated eligible source, ignores hints, and preserves maximum fill',()=>{
  const input=fixture({capacities:[M,3n*M],preferred:0});
  input.validatorIndex='1';
  const required=selectSource(input);
  assert.equal(required.status,'MULTI_LEG_CANDIDATE');
  assert.equal(required.candidate.validatorIndex,'0');
  assert.equal(required.candidate.maximumFillUnits,String(M));
  assert.equal(required.projectedNextLeg.validatorIndex,'1');
  assert.deepEqual(required.rejections,[{validatorIndex:'1',reason:'PREFERRED_SOURCE_REQUIRED'}]);
  const exhausted=fixture({capacities:[1n,3n*M],preferred:0});
  const fallback=selectSource(exhausted);
  assert.equal(fallback.status,'SINGLE_LEG_CANDIDATE');
  assert.equal(fallback.candidate.validatorIndex,'1');
  assert.equal(fallback.candidate.sourceStake,exhausted.sourceAccounts[1].account.address);
  assert.equal(fallback.candidate.maximumFillUnits,String(2n*M));
  assert.equal(fallback.chainSnapshotVerified,false);
});

test('skips a dust-stranding maximum fill and selects refreshed continuation after observed progress',()=>{
  const capacities=[1_600_000_000n,1_500_000_000n,1_000_000_000n],target=2_500_000_000n;
  const firstState=fixture({capacities,target});
  const first=selectSource(firstState);
  assert.equal(first.status,'MULTI_LEG_CANDIDATE');
  assert.equal(first.candidate.validatorIndex,'1');
  assert.equal(first.candidate.maximumFillUnits,'1500000000');
  assert.deepEqual(first.rejections,[{validatorIndex:'0',reason:'MAXIMUM_FILL_STRANDS_REMAINDER',
    capacityUnits:'1600000000',remainingAfterUnits:'900000000'}]);
  assert.throws(()=>selectSource({...firstState,after:first.refreshAfter}),{code:'refresh_required'});
  const fresh=fixture({capacities,target,slot:3n,assigned:1_500_000_000n,leg:1n,takenIndex:1});
  // The earlier projection favored source 0. A refresh shows it is no longer Active.
  replace(fresh.validatorList,list=>{list[49]=1;});
  const second=selectSource({...fresh,after:first.refreshAfter});
  assert.equal(second.status,'SINGLE_LEG_CANDIDATE');
  assert.equal(second.nextLegIndex,'1');
  assert.equal(second.candidate.validatorIndex,'2');
  assert.equal(second.candidate.maximumFillUnits,String(M));
});

test('uses the selected source Meta rent for the preferred threshold and rounds the pool fee upward',()=>{
  const input=fixture({capacities:[M,3n*M],preferred:0,fee:[1n,7n]});
  const alternate=input.sourceAccounts[1].account;
  alternate.lamports=String(BigInt(alternate.lamports)+M);
  replace(alternate,b=>b.writeBigUInt64LE(R+M,4));
  replace(input.validatorList,b=>b.writeBigUInt64LE(BigInt(alternate.lamports),82));
  replace(input.state.distribution,b=>b.writeBigUInt64LE(1_714_285_712n,223));
  const selection=selectSource(input);
  assert.equal(selection.status,'SINGLE_LEG_CANDIDATE');
  assert.equal(selection.candidate.validatorIndex,'1');
  assert.equal(selection.candidate.expectedNativeLamports,'1714285714');
  assert.equal(selection.candidate.protectedMinimumNativeLamports,'1714114285');
});

test('refreshes each assigned leg of a three-leg target, including advancement within one slot',()=>{
  const target=3n*M,capacities=[M,M,M];
  const first=selectSource(fixture({target,capacities}));
  assert.equal(first.status,'MULTI_LEG_CANDIDATE');
  assert.equal(first.candidate.validatorIndex,'0');
  const second=selectSource({...fixture({target,capacities,leg:1n,assigned:M}),after:first.refreshAfter});
  assert.equal(second.status,'MULTI_LEG_CANDIDATE');
  assert.equal(second.candidate.validatorIndex,'1');
  const third=selectSource({...fixture({target,capacities,slot:3n,leg:2n,assigned:2n*M,receipts:{0:M,1:M}}),
    after:second.refreshAfter});
  assert.equal(third.status,'SINGLE_LEG_CANDIDATE');
  assert.equal(third.candidate.validatorIndex,'2');
  assert.equal(third.candidate.remainingAfterUnits,'0');
});

test('mandatory preferred dust conflict requires exact SPL exhaustion before fallback',()=>{
  const target=2n*M+1n,capacities=[M+2n,target],preferred=0;
  const beforePreference=selectSource(fixture({target,capacities}));
  assert.equal(beforePreference.status,'SINGLE_LEG_CANDIDATE');
  assert.equal(beforePreference.candidate.validatorIndex,'1');
  const initial=fixture({target,capacities,preferred});
  const blocked=selectSource(initial);
  assert.equal(blocked.status,'NO_ADMISSIBLE_SOURCE');
  assert.deepEqual(blocked.rejections,[
    {validatorIndex:'0',reason:'MAXIMUM_FILL_STRANDS_REMAINDER',capacityUnits:String(M+2n),remainingAfterUnits:String(M-1n)},
    {validatorIndex:'1',reason:'PREFERRED_SOURCE_REQUIRED'},
  ]);
  // Compare the post-state of taking only M with the stronger exhaustion condition.
  const stranded=selectSource(fixture({target,capacities,preferred,leg:1n,assigned:M}));
  assert.equal(stranded.status,'NO_ADMISSIBLE_SOURCE');
  assert(stranded.rejections.some(r=>r.reason==='PREFERRED_SOURCE_REQUIRED'));
  const exhausted=selectSource(fixture({target,capacities,preferred,leg:1n,assigned:M+1n}));
  assert.equal(exhausted.status,'SINGLE_LEG_CANDIDATE');
  assert.equal(exhausted.candidate.validatorIndex,'1');
  assert.equal(exhausted.candidate.maximumFillUnits,String(M));
});

test('proposes the exact preferred cut only with an eligible alternate and both rent advances',()=>{
  const target=2n*M+1n,capacities=[M+2n,target],preferred=0;
  const initial=fixture({target,capacities,preferred});
  const c=validateSnapshot(initial.state).decoded.config.value;
  const rentFloor=128n*6960n,legRent=(328n+391n)*6960n;
  const funded=raw(c.operational_sol_vault,IDS.system,Buffer.alloc(0),rentFloor+2n*legRent);
  const first=selectSource({...initial,operationalSol:funded});
  assert.equal(first.status,'PREFERRED_PARTITION_CANDIDATE');
  assert.equal(first.candidate.validatorIndex,'0');
  assert.equal(first.candidate.alternateStake,initial.sourceAccounts[1].account.address);
  assert.equal(first.candidate.maximumFillUnits,String(M+2n));
  assert.equal(first.candidate.inputUnits,String(M+1n));
  assert.equal(first.candidate.remainingAfterUnits,String(M));
  assert.equal(first.projectedNextLeg.validatorIndex,'1');
  assert.equal(first.projectedNextLeg.maximumFillUnits,String(M));
  assert.equal(first.projectedNextLeg.remainingAfterUnits,'0');
  assert.throws(()=>selectSource({...initial,operationalSol:funded,after:first.refreshAfter}),{code:'refresh_required'});
  const next=fixture({target,capacities,preferred,leg:1n,assigned:M+1n});
  const second=selectSource({...next,after:first.refreshAfter});
  assert.equal(second.status,'SINGLE_LEG_CANDIDATE');
  assert.equal(second.candidate.validatorIndex,'1');
  assert.equal(second.candidate.alternateStake,undefined);
  assert.equal(selectSource({...initial,operationalSol:{...funded,lamports:String(rentFloor+2n*legRent-1n)}}).status,
    'NO_ADMISSIBLE_SOURCE');
  for(const different of [[M+3n,target],[M+2n,M-1n]]) {
    assert.equal(selectSource({...fixture({target,capacities:different,preferred}),operationalSol:funded}).status,
      'NO_ADMISSIBLE_SOURCE');
  }
});

test('binds pool/list/mint/Stake data and rejects stale context and unconfirmed continuations',()=>{
  const input=fixture({capacities:[M,3n*M]});
  const mutations=[
    [m=>{m.validatorList.owner=IDS.stake;},'account_binding'],
    [m=>{m.pool.address=IDS.stake;},'account_binding'],
    [m=>{m.sourceAccounts[1].account.address=m.sourceAccounts[0].account.address;},'account_binding'],
    [m=>replace(m.sourceAccounts[1].account,b=>{b[44]^=1;}),'invalid_stake_source'],
    [m=>replace(m.mint,b=>{b[4]^=1;}),'invalid_mint'],
    [m=>replace(m.validatorList,b=>{b.writeUInt32LE(10_001,5);}),'validator_list_geometry'],
    [m=>{m.sourceAccounts[0].account.lamports='1';},'stake_balance_mismatch'],
    [m=>{m.state.context.slot='3';},'clock_context_mismatch'],
    [m=>{m.minimumDelegationEpoch='39';},'minimum_delegation_epoch_mismatch'],
  ];
  for(const [mutate,code] of mutations) {
    const changed=clone(input);
    mutate(changed);
    assert.throws(()=>selectSource(changed),{code});
  }
  const only=[input.sourceAccounts[0]];
  assert.equal(selectSource({...input,sourceAccounts:only}).status,'NO_OBSERVED_CONTINUATION');
});

test('CLI source mode accepts a large offline snapshot and emits the existing initiation fields',context=>{
  const directory=fs.mkdtempSync(path.join(os.tmpdir(),'piv1-source-selection-'));
  context.after(()=>fs.rmSync(directory,{recursive:true,force:true}));
  const filename=path.join(directory,'snapshot.json');
  fs.writeFileSync(filename,JSON.stringify(fixture({capacities:[M,3n*M]})));
  const result=spawnSync(process.execPath,[path.join(__dirname,'withdrawal-plan.cjs'),'--select-source',filename],{encoding:'utf8'});
  assert.equal(result.status,0,result.stderr);
  const report=JSON.parse(result.stdout);
  assert.equal(report.status,'SINGLE_LEG_CANDIDATE');
  assert.equal(report.roundSequence,'0');
  assert.equal(report.nextLegIndex,'0');
  assert.equal(report.candidate.validatorIndex,'1');
  assert(report.candidate.sourceStake);
});
