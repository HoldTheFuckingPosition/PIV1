'use strict';
const assert=require('node:assert/strict');
const fs=require('node:fs');
const os=require('node:os');
const path=require('node:path');
const {createHash}=require('node:crypto');
const {spawnSync}=require('node:child_process');
const test=require('node:test');
const f=require('./initializer-funding.cjs');
const n=require('./initializer-plan.cjs');
const {fixture}=require('./initializer-plan.test.cjs');
const copy=v=>JSON.parse(JSON.stringify(v)),sum=values=>values.reduce((a,v)=>a+BigInt(v),0n);
// Deliberately irregular prices: independent of any Bank Rent formula/public quote.
const LITERAL=[11,17,23,29,31,37,41,43,47,53,59,61,67,71];
function observations(values=LITERAL) {
  return {...copy(f.TEMPLATE),observedAtUtc:'2026-10-19T00:00:00Z',slotBefore:'9007199254740993',slotAfter:'9007199254740995',
    rentLamportsByBytes:Object.fromEntries(f.SIZES.map((size,i)=>[size,String(values[i])])),logicalRequests:18,httpRequests:3,limitations:'Synthetic arithmetic fixture; no RPC or current account claim.'};
}
const report=(o=observations(),m=copy(n.TEMPLATE))=>f.fundingReport(m,o);
const phase=(s,name)=>s.phases.find(p=>p.name===name);
const wallet=(s,role)=>s.wallets.find(w=>w.roles.includes(role));
const sha=b=>createHash('sha256').update(b).digest('hex');

test('source binding, nullable template and retained historical sample are distinct',()=>{
  f.verifySources();const incomplete=report(copy(f.TEMPLATE));assert.equal(incomplete.status,'INCOMPLETE');assert.equal(incomplete.scenarios,null);assert(incomplete.missingObservations.length>14);
  const sample=fs.readFileSync(path.join(__dirname,'initializer-funding.example.json'));
  assert.equal(sha(sample),'537e702858a6a8f5d5c6c88877ba8300b690d3f2c6d4bd078b56d8636d09a0b3');
  assert.deepEqual(f.validateObservations(JSON.parse(sample)).missing,[]);
  assert.equal(incomplete.readiness.actualFundingQuote,null);assert.equal(incomplete.readiness.publicReady,false);
  assert.equal(report({}).status,'INCOMPLETE');
});
test('both symbolic variants match literal rent ledger, delta funding and creator refund order',()=>{
  const result=report();assert.equal(result.status,'CONDITIONAL_RENT_ONLY');assert.equal(result.manifestStatus,'INCOMPLETE');assert.equal(result.scenarios.length,2);
  for(const s of result.scenarios) {
    const shared=s.variant==='shared34';
    assert.deepEqual(s.totals,{pivTargetFloors:'392',grossRentPayerOutflow:shared?'596':'614',creatorBufferRefund:shared?'61':'71',
      finalHeldAccountIncrease:shared?'535':'543',requiredRentPayerResidual:'11',actualFundingQuote:null,actualShortfall:null,peakLiquidity:null,actualRequiredWalletBalance:null});
    assert.deepEqual(phase(s,'alt_create').events.map(e=>e.lamports),['17']);assert.deepEqual(phase(s,'alt_extend').events.map(e=>e.lamports),['24']);
    const finish=phase(s,'transaction_from_buffer');assert.deepEqual(finish.events.map(e=>[e.fromRole,e.toRole,e.lamports]),[
      ['rentPayer','stored_transaction','29'],['rentPayer','stored_transaction',shared?'30':'38'],['buffer','creator',shared?'61':'71']]);
    assert(finish.events[0].ordinal<finish.events[1].ordinal&&finish.events[1].ordinal<finish.events[2].ordinal);
    assert.deepEqual(phase(s,'execute').events.map(e=>e.lamports),['53','47','37','23','23','23','23','23','23','11','11','11','11','11','31','31']);
    assert.equal(wallet(s,'rentPayer').rentOnlyNetChange,shared?'-596':'-614');assert.equal(wallet(s,'creator').rentOnlyNetChange,shared?'61':'71');
    assert.equal(wallet(s,'feePayer').rentOnlyNetChange,'0');assert.equal(s.temporaryBufferFinalBalance,'0');
    assert.equal(s.heldAccounts.length,19);assert(!s.heldAccounts.some(a=>a.role==='buffer'));
    for(const p of s.phases){assert.equal(p.signatureCount,null);assert.equal(p.requiredSigners,null);assert.equal(p.actualFee,null);}
    for(const name of ['buffer_extend_*','approve_1','approve_2','approve_3','approve_4'])assert.deepEqual(phase(s,name).events,[]);
    assert(s.prerequisites.recipients.every(r=>r.minimumExistingBalance==='11'&&r.actualShortfall===null&&!r.provisionedByThesePackets));
    assert.equal(s.prerequisites.rentPayer.spent,false);assert.equal(s.prerequisites.rentPayer.postInitializerResidual,'11');
    assert(s.wallets.every(w=>w.address===null));
  }
  const partial=copy(n.TEMPLATE);partial.rentPayer=fixture().rentPayer;partial.governance.creator=partial.rentPayer;partial.feePayer=partial.rentPayer;
  const symbolic=report(observations(),partial);
  assert(symbolic.scenarios.every(s=>s.wallets.every(w=>w.address===null)));
  assert.equal(wallet(symbolic.scenarios[0],'rentPayer').rentOnlyNetChange,'-596');
  assert.equal(wallet(symbolic.scenarios[0],'creator').rentOnlyNetChange,'61');
  assert.equal(symbolic.scenarios[0].wallets.length,3);
});
test('deterministic irregular price domains conserve wallets plus final held capital',()=>{
  for(let k=0;k<32;k++) {
    const values=f.SIZES.map((_,i)=>BigInt(k+1)*10000000000000007n+BigInt(i*i+7*i+1));
    for(const s of report(observations(values)).scenarios) {
      const debits=sum(s.wallets.map(w=>w.rentOnlyDebits)),credits=sum(s.wallets.map(w=>w.rentOnlyCredits)),held=sum(s.heldAccounts.map(a=>a.lamports));
      assert.equal(debits-credits,held);assert.equal(sum(s.wallets.map(w=>w.rentOnlyNetChange)),-held);
      assert.equal(debits,BigInt(s.totals.grossRentPayerOutflow));assert.equal(credits,BigInt(s.totals.creatorBufferRefund));
      assert.equal(BigInt(s.totals.finalHeldAccountIncrease),held);
      assert.equal(sum(phase(s,'alt_create').events.map(e=>e.lamports))+sum(phase(s,'alt_extend').events.map(e=>e.lamports)),values[6]);
      const finish=phase(s,'transaction_from_buffer').events;
      assert.equal(sum(finish.slice(0,2).map(e=>e.lamports)),values[s.variant==='shared34'?10:12]);
      assert.equal(phase(s,'execute').events.length,16);assert.equal(sum(phase(s,'execute').events.map(e=>e.lamports)),BigInt(s.totals.pivTargetFloors));
      assert.equal(s.totals.peakLiquidity,null);assert.equal(s.totals.actualRequiredWalletBalance,null);
      assert.equal(s.heldAccounts.filter(a=>a.bytes===0).length,5);
    }
  }
});
test('complete public manifests use exact planner phases and1/2/3 signer unions without fee assumptions',()=>{
  const counts=new Set();
  for(const shared of [false,true])for(const fee of ['rent','creator','executor','separate']) {
    const m=fixture({shared,fee});if(fee==='rent')m.transport.altAuthority=m.rentPayer;
    if(fee==='executor')m.feePayer=m.governance.executor;
    const expected=n.planManifest(m).plan,r=f.fundingReport(m,observations()),s=r.scenarios[0];
    assert.equal(r.manifestStatus,'OFFLINE_PLAN_ONLY');assert.equal(r.scenarios.length,1);assert.equal(s.variant,shared?'shared34':'distinct35');
    assert.deepEqual(s.phases.map(p=>p.name),expected.packets.map(p=>p.name));
    for(let i=0;i<s.phases.length;i++){
      const p=s.phases[i];counts.add(p.signatureCount);assert.equal(p.signatureCount,expected.packets[i].signatureCount);
      assert.deepEqual(p.requiredSigners,expected.packets[i].requiredSigners);assert.equal(p.actualFee,null);
    }
    assert.equal(s.totals.grossRentPayerOutflow,shared?'596':'614');assert.equal(wallet(s,'rentPayer').address,m.rentPayer);
    assert.equal(wallet(s,'creator').address,m.governance.creator);assert.notEqual(wallet(s,'rentPayer'),wallet(s,'creator'));
    if(fee==='rent')assert(wallet(s,'rentPayer').roles.includes('feePayer'));
    if(fee==='creator')assert(wallet(s,'creator').roles.includes('feePayer'));
    if(fee==='executor')assert.equal(wallet(s,'feePayer').address,m.governance.executor);
    assert(r.readiness.actualFees===null&&!r.readiness.publicReady&&!r.readiness.accountSnapshotVerified);
  }
  assert.deepEqual([...counts].sort(),[1,2,3]);
});
test('strict observation shape and known invalid fields reject even with unrelated nulls',()=>{
  const changes=[o=>o.status='READY',o=>o.endpoint='https://example.invalid',o=>o.genesisHash='wrong',o=>o.commitment='processed',
    o=>o.observedAtUtc='2026-02-30T00:00:00Z',o=>o.observedAtUtc='2026-10-19T00:00:00+01:00',o=>o.slotAfter='1',
    o=>o.logicalRequests='18',o=>o.httpRequests=19,o=>o.logicalRequests=0,o=>o.limitations='',o=>o.extra='do-not-echo',
    o=>o.rentLamportsByBytes=[],o=>o.rentLamportsByBytes['01']='5',o=>o.rentLamportsByBytes[56]='10'];
  for(const change of changes){const o=observations();change(o);assert.throws(()=>report(o));o.rentLamportsByBytes[1592]=null;assert.throws(()=>report(o));}
  for(const value of [0,-1,1.5,'-1','+1','01','1.0','1e3','18446744073709551616']) {
    const o=copy(f.TEMPLATE);o.rentLamportsByBytes[0]=value;assert.throws(()=>report(o));
  }
  for(const text of ['{"status":null,"status":null}','{"slotBefore":null,"slot\\u0042efore":null}','[]'])
    assert.throws(()=>report(n.parseManifest(text)));
  const bad=copy(n.TEMPLATE);bad.feePayer=1;assert.throws(()=>f.fundingReport(bad,copy(f.TEMPLATE)));
});
test('null/missing prices preserve incomplete totals and BigInt u64 sums stay exact',()=>{
  for(const size of f.SIZES) {
    for(const remove of [false,true]){
      const o=observations();if(remove)delete o.rentLamportsByBytes[size];else o.rentLamportsByBytes[size]=null;
      const r=report(o);assert.equal(r.status,'INCOMPLETE');assert.equal(r.scenarios,null);assert(r.missingObservations.includes(`observations.rentLamportsByBytes.${size}`));
    }
  }
  const max=(1n<<64n)-1n,r=report(observations(f.SIZES.map(()=>max)));
  for(const s of r.scenarios) {
    assert.equal(BigInt(s.totals.pivTargetFloors),16n*max);assert.equal(BigInt(s.totals.grossRentPayerOutflow),20n*max);
    assert.equal(BigInt(s.totals.finalHeldAccountIncrease),19n*max);assert.equal(BigInt(s.totals.creatorBufferRefund),max);
    assert.equal(phase(s,'alt_extend').events[0].lamports,'0');assert.equal(phase(s,'transaction_from_buffer').events[1].lamports,'0');
    assert.equal(s.totals.actualRequiredWalletBalance,null); // mathematical aggregate is not a feasible wallet balance claim
  }
});
test('CLI template/incomplete/conditional complete and invalid paths stay offline and explicit',context=>{
  const script=path.join(__dirname,'initializer-funding.cjs'),limits=process.execArgv.filter(s=>s.startsWith('--max-old-space-size'));
  const run=args=>spawnSync(process.execPath,[...limits,script,...args],{encoding:'utf8',maxBuffer:2*1024*1024,timeout:30000});
  const template=run(['--template']);assert.equal(template.status,0);assert.equal(template.stderr,'');assert.deepEqual(JSON.parse(template.stdout),f.TEMPLATE);
  const directory=fs.mkdtempSync(path.join(os.tmpdir(),'piv1-funding-public-test-'));
  const write=(name,value)=>{const filename=path.join(directory,name);fs.writeFileSync(filename,typeof value==='string'?value:JSON.stringify(value,null,2)+'\n',{flag:'wx'});return filename;};
  const incompleteFile=write('nullable-rents.json',f.TEMPLATE),syntheticFile=write('synthetic-rents.json',observations()),manifestFile=write('public-manifest.json',fixture({shared:true,fee:'separate'}));
  const args=(manifest,observed)=>['--manifest',manifest,'--observations',observed];
  const incomplete=run(args(path.join(__dirname,'initializer-plan.example.json'),incompleteFile));assert.equal(incomplete.status,0);assert.equal(JSON.parse(incomplete.stdout).status,'INCOMPLETE');assert.equal(JSON.parse(incomplete.stdout).scenarios,null);
  const complete=run(args(manifestFile,syntheticFile));assert.equal(complete.status,0);assert.equal(complete.stderr,'');const parsed=JSON.parse(complete.stdout);
  assert.equal(parsed.status,'CONDITIONAL_RENT_ONLY');assert.equal(parsed.manifestStatus,'OFFLINE_PLAN_ONLY');assert.deepEqual(parsed.scenarios,f.fundingReport(fixture({shared:true,fee:'separate'}),observations()).scenarios);
  assert.equal(parsed.observationInputSha256,sha(fs.readFileSync(syntheticFile)));assert.equal(parsed.readiness.actualFees,null);assert.equal(parsed.readiness.peakLiquidity,null);
  const symbolic=run(args(path.join(__dirname,'initializer-plan.example.json'),path.join(__dirname,'initializer-funding.example.json')));
  assert.equal(symbolic.status,0);assert.equal(JSON.parse(symbolic.stdout).scenarios.length,2);assert.equal(JSON.parse(symbolic.stdout).readiness.actualFundingQuote,null);
  const duplicate=write('duplicate.json','{"status":null,"status":null}');
  for(const command of [[],['--send'],['--template','--rpc'],['--manifest'],args(manifestFile,duplicate)]){
    const bad=run(command);assert.notEqual(bad.status,0);assert.equal(bad.stdout,'');assert.equal(JSON.parse(bad.stderr).status,'INVALID_OR_UNSUPPORTED_FUNDING_REPORT');
  }
  context.diagnostic(`Public-only test inputs retained: ${directory}`);
});
