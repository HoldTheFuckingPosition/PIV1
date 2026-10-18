'use strict';
const assert=require('node:assert/strict');
const fs=require('node:fs');
const os=require('node:os');
const path=require('node:path');
const {createHash}=require('node:crypto');
const {spawnSync}=require('node:child_process');
const test=require('node:test');
const a=require('./intake-plan.cjs');
const n=require('./initializer-plan.cjs');
const {publicPoints}=require('./initializer-plan.test.cjs');
const t=require('../../validation/genesis-transport/transport.cjs');
const {PublicKey,TransactionMessage,VersionedTransaction}=require(t.SDK);
const POINTS=publicPoints(),copy=v=>JSON.parse(JSON.stringify(v));
const ACTIONS=['contribute-sol','contribute-jitosol','bootstrap-initial'];
function fixture(action,{sameFee=false,shared=false,amount='72623859790382856'}={}) {
  const m=copy(a.TEMPLATE);m.programId=POINTS[18];m.feePayer=POINTS[7];m.action=action;
  Object.assign(m.transport,{recentBlockhash:POINTS[16],lastValidBlockHeight:'777777',referenceSlot:'9007199254740997'});
  Object.assign(m.resources,{computeUnits:1400000,heapBytes:262144,loadedAccountBytes:8388608});
  if(action==='contribute-sol'){m.sol={donor:POINTS[0],amountLamports:amount};if(sameFee)m.feePayer=m.sol.donor;}
  else if(action==='contribute-jitosol'){m.jitosol={owner:POINTS[5],sourceToken:POINTS[9],amountUnits:amount};if(sameFee)m.feePayer=m.jitosol.owner;}
  else m.bootstrap={validatorList:POINTS[9],reserve:POINTS[10],managerFee:POINTS[11],referrer:shared?POINTS[11]:POINTS[13]};
  return m;
}
function literal(m) {
  const derive=seed=>PublicKey.findProgramAddressSync([Buffer.from(seed)],new PublicKey(m.programId))[0].toBase58();
  const row=(address,signer=false,writable=false)=>({address,signer,writable});
  let prefix,rows,amount;
  if(m.action==='contribute-sol') {
    prefix='504956314453303101';amount=BigInt(m.sol.amountLamports);
    rows=[row(derive('config'),false,true),row(derive('distribution')),row(derive('pending-sol'),false,true),row(derive('pending-jito-vault')),row(m.sol.donor,true,true),row(a.IDS.system)];
  } else if(m.action==='contribute-jitosol') {
    prefix='50495631444a303101';amount=BigInt(m.jitosol.amountUnits);
    rows=[row(derive('config'),false,true),row(derive('distribution')),row(derive('pending-sol')),row(derive('pending-jito-vault'),false,true),row(m.jitosol.sourceToken,false,true),row(m.jitosol.owner,true),row(a.IDS.mint),row(a.IDS.token)];
  } else {
    prefix='504956314942303101';
    rows=[row(derive('config'),false,true),row(derive('distribution')),row(derive('pending-sol'),false,true),row(derive('principal-sol'),false,true),
      row(derive('operational-sol')),row(derive('distribution-escrow')),row(derive('kif-sol')),row(derive('principal-jito-vault'),false,true),row(derive('pending-jito-vault'),false,true),
      row(derive('authority')),row(a.IDS.mint),row(a.IDS.system),row(a.IDS.token),row(a.IDS.poolProgram),row(a.IDS.pool),row(m.bootstrap.validatorList),row(m.bootstrap.reserve),row(m.bootstrap.managerFee)];
    if(m.bootstrap.managerFee!==m.bootstrap.referrer)rows.push(row(m.bootstrap.referrer));
  }
  const bytes=Buffer.concat([Buffer.from(prefix,'hex'),amount===undefined?Buffer.alloc(0):Buffer.from(Array.from({length:8},(_,i)=>Number(amount>>BigInt(8*i)&255n)))]);
  return {rows,bytes};
}
const body=p=>p.plan.packets[0];
const decode=p=>VersionedTransaction.deserialize(Buffer.from(p.wireBase64,'base64'));
const row=r=>({address:r.address,signer:r.signer,writable:r.writable});

test('source pins and null template remain incomplete without action or identity substitutes',()=>{
  a.verifySources();assert.deepEqual(JSON.parse(fs.readFileSync(path.join(__dirname,'intake-plan.example.json'))),a.TEMPLATE);
  for(const m of [copy(a.TEMPLATE),{}]){const p=a.planManifest(m);assert.equal(p.status,'INCOMPLETE');assert.equal(p.plan,null);assert(p.missing.includes('action'));assert(!p.readiness.publicReady);}
  assert.deepEqual(a.ERROR_GUIDE.map(e=>[e.code,e.name]),[[6017,'AccountAlias'],[6022,'InvalidTokenCustody'],[6026,'InvalidBootstrapState'],[6034,'InvalidLifecycle'],[6035,'PausedOperation'],[6045,'ZeroContribution'],[6046,'InvalidCustodyObservation'],[6048,'ContributionObservationMismatch'],[6049,'PendingCustodyDeficit']]);
});
test('three exact native ABIs and6/8/18/19 roles match independent literal metas and PDAs',()=>{
  for(const action of ACTIONS)for(const shared of [false,true]) {
    const m=fixture(action,{shared}),p=a.planManifest(m),expected=literal(m);
    assert.equal(p.status,'OFFLINE_PLAN_ONLY');assert.equal(p.plan.native.hex,expected.bytes.toString('hex'));
    assert.deepEqual(p.plan.roles.map(row),expected.rows);assert.equal(p.plan.native.bytes,action==='bootstrap-initial'?9:17);
    assert.equal(p.plan.roles.length,action==='contribute-sol'?6:action==='contribute-jitosol'?8:shared?18:19);
    assert.equal(p.plan.packets.length,1);assert.equal(body(p).version,'legacy');assert.equal(p.plan.bootstrapAmounts,null);assert.equal(p.plan.bootstrapBookValueLamports,null);
    assert.equal(p.readiness.bootstrapEligible,null);assert.equal(p.readiness.fees,null);assert.equal(p.readiness.resourceSufficiencyVerified,false);
    if(action==='bootstrap-initial'){
      assert.equal(p.plan.amount,null);assert(p.plan.roles.every(r=>!r.signer));assert.deepEqual(p.plan.roles.map((r,i)=>r.writable?i:null).filter(i=>i!==null),[0,2,3,7,8]);
      assert.equal(body(p).signatureCount,1);assert.deepEqual(body(p).requiredSigners,[m.feePayer]);assert(!p.plan.roles[9].writable);
      assert(p.preconditions.some(s=>s.includes('floored book value0')));
    } else assert(p.preconditions.some(s=>s.includes('not Idle-only')));
  }
});
test('positive full-u64 amounts use exact little-endian bytes without changing account roles',()=>{
  const values=[1n,2n,127n,128n,255n,256n,65535n,65536n,(1n<<32n)-1n,1n<<32n,(1n<<53n)-1n,(1n<<53n)+1n,(1n<<64n)-2n,(1n<<64n)-1n];
  for(const action of ACTIONS.slice(0,2)) {
    let roles;
    for(const value of values){const p=a.planManifest(fixture(action,{amount:String(value)})),bytes=Buffer.from(p.plan.native.hex,'hex');
      assert.deepEqual(bytes,literal(fixture(action,{amount:String(value)})).bytes);assert.equal(bytes.readBigUInt64LE(9),value);
      if(roles)assert.deepEqual(p.plan.roles,roles);else roles=p.plan.roles;assert.equal(p.plan.amount.value,String(value));}
  }
  assert.equal(a.planManifest(fixture('contribute-sol')).plan.native.hex,'5049563144533031010807060504030201');
  assert.equal(a.planManifest(fixture('contribute-jitosol')).plan.native.hex,'50495631444a3031010807060504030201');
  for(const invalid of [0,1,'0','-1','+1','01','1.0','1e9','18446744073709551616'])for(const action of ACTIONS.slice(0,2))assert.throws(()=>a.planManifest(fixture(action,{amount:invalid})));
});
test('actual1/2 zero-signature packets preserve requested and outer privilege unions',()=>{
  const observed=new Set();
  for(const action of ACTIONS)for(const sameFee of [false,true]) {
    const m=fixture(action,{sameFee}),p=a.planManifest(m),packet=body(p),tx=decode(packet),recovered=TransactionMessage.decompile(tx.message).instructions;
    const expected=literal(m),expectedSigners=[...new Set([m.feePayer,...(action==='bootstrap-initial'?[]:[action==='contribute-sol'?m.sol.donor:m.jitosol.owner])])].sort();
    observed.add(packet.signatureCount);assert.deepEqual([...packet.requiredSigners].sort(),expectedSigners);assert.equal(packet.zeroSignatureBytes,64*expectedSigners.length);
    assert.equal(packet.wireBytes,Buffer.from(packet.wireBase64,'base64').length);assert(packet.wireBytes<=1232);assert(tx.signatures.every(s=>s.length===64&&s.every(b=>b===0)));
    assert.deepEqual(Buffer.from(tx.serialize()),Buffer.from(packet.wireBase64,'base64'));assert.equal(recovered.length,4);
    assert.deepEqual(recovered.slice(0,3).map(i=>i.data.toString('hex')),['02c05c1500','0100000400','0400008000']);assert(recovered.slice(0,3).every(i=>i.keys.length===0));
    assert.deepEqual(recovered[3].data,literal(m).bytes);assert.equal(recovered[3].programId.toBase58(),m.programId);
    recovered[3].keys.forEach((k,i)=>{const want=expected.rows[i];assert.equal(k.pubkey.toBase58(),want.address);assert.equal(k.isSigner,want.signer);assert.equal(k.isWritable,want.writable||want.address===m.feePayer);});
    if(action==='contribute-jitosol'&&sameFee){assert.equal(p.plan.roles[5].writable,false);assert.equal(packet.wireInstructions[3].accounts[5].writable,true);}
  }
  assert.deepEqual([...observed].sort(),[1,2]);
  const m=fixture('contribute-jitosol');m.jitosol.sourceToken=PublicKey.findProgramAddressSync([Buffer.from('public-off-curve-source')],new PublicKey(m.programId))[0].toBase58();
  assert.equal(PublicKey.isOnCurve(new PublicKey(m.jitosol.sourceToken).toBytes()),false);assert.equal(a.planManifest(m).status,'OFFLINE_PLAN_ONLY');
});
test('explicit resource domain boundaries encode once and unsupported requests reject',()=>{
  for(const units of [1,2,1399999,1400000])for(const loaded of [1,8388608]) {
    const m=fixture('bootstrap-initial');m.resources.computeUnits=units;m.resources.loadedAccountBytes=loaded;
    const data=body(a.planManifest(m)).requestedInstructions.slice(0,3).map(i=>Buffer.from(i.dataHex,'hex'));
    assert.deepEqual(data.map(b=>b[0]),[2,1,4]);assert.deepEqual(data.map(b=>b.readUInt32LE(1)),[units,262144,loaded]);
  }
  for(let heap=32768;heap<=262144;heap+=1024){const m=fixture('contribute-sol');m.resources.heapBytes=heap;assert.deepEqual(a.validateManifest(m).missing,[]);}
  for(const [field,values]of [['computeUnits',[0,1400001,1.5,'1400000']],['heapBytes',[32767,32769,262145,'32768']],['loadedAccountBytes',[0,8388609,-1,'8388608']]])
    for(const value of values){const m=copy(a.TEMPLATE);m.resources[field]=value;assert.throws(()=>a.planManifest(m));}
  const m=fixture('contribute-sol');m.resources={computeUnits:1,heapBytes:32768,loadedAccountBytes:1};
  assert.deepEqual(body(a.planManifest(m)).requestedInstructions.slice(0,3).map(i=>i.dataHex),['0201000000','0100800000','0401000000']);
});
test('strict action isolation, known invalid partials, aliases and missing fields never select substitutes',()=>{
  const changes=[m=>m.action='stake',m=>m.cluster='mainnet-beta',m=>m.schema='other',m=>m.programId=a.IDS.pool,
    m=>m.jitosol.owner=m.jitosol.sourceToken,m=>m.feePayer=m.jitosol.sourceToken,m=>m.jitosol.sourceToken=a.derived(m.programId).pending_token,
    m=>m.jitosol.owner=a.derived(m.programId).piv_authority,m=>m.feePayer=a.derived(m.programId).config,
    m=>m.sol.amountLamports='1',m=>m.bootstrap.managerFee=POINTS[11],m=>m.jitosol.extra='do-not-echo',m=>m.transport.referenceSlot='01'];
  for(const change of changes){const m=fixture('contribute-jitosol');change(m);assert.throws(()=>a.planManifest(m));m.transport.recentBlockhash=null;assert.throws(()=>a.planManifest(m));}
  for(const sameReceiver of [false,true]){
    const m=fixture('bootstrap-initial',{shared:sameReceiver});m.bootstrap.reserve=m.bootstrap.validatorList;assert.throws(()=>a.planManifest(m));}
  const bootstrap=fixture('bootstrap-initial');bootstrap.bootstrap.amount='1';assert.throws(()=>a.planManifest(bootstrap));
  const governance=fixture('bootstrap-initial');governance.governance={};assert.throws(()=>a.planManifest(governance));
  const offcurve=fixture('contribute-sol');offcurve.sol.donor=a.derived(offcurve.programId).config;assert.throws(()=>a.planManifest(offcurve));
  for(const action of ACTIONS) {
    const m=fixture(action),block=action==='contribute-sol'?'sol':action==='contribute-jitosol'?'jitosol':'bootstrap';
    for(const field of ['programId','feePayer','action']){const incomplete=copy(m);incomplete[field]=null;const p=a.planManifest(incomplete);assert.equal(p.status,'INCOMPLETE');assert.equal(p.plan,null);}
    for(const field of Object.keys(m[block])){const incomplete=copy(m);incomplete[block][field]=null;assert.equal(a.planManifest(incomplete).plan,null);}
  }
  for(const source of ['{"action":null,"action":null}','{"feePayer":null,"fee\\u0050ayer":null}','[]','{"unknown":"do-not-echo"}'])assert.throws(()=>a.planManifest(n.parseManifest(source)));
});
test('CLI retains all three complete public reports and supports only template/manifest',context=>{
  const script=path.join(__dirname,'intake-plan.cjs'),limits=process.execArgv.filter(s=>s.startsWith('--max-old-space-size'));
  const run=args=>spawnSync(process.execPath,[...limits,script,...args],{encoding:'utf8',maxBuffer:2*1024*1024,timeout:30000});
  const template=run(['--template']);assert.equal(template.status,0);assert.equal(template.stderr,'');assert.deepEqual(JSON.parse(template.stdout),a.TEMPLATE);
  const incomplete=run(['--manifest',path.join(__dirname,'intake-plan.example.json')]);assert.equal(incomplete.status,0);assert.equal(JSON.parse(incomplete.stdout).status,'INCOMPLETE');assert.equal(JSON.parse(incomplete.stdout).plan,null);
  const directory=fs.mkdtempSync(path.join(os.tmpdir(),'piv1-intake-public-test-'));
  for(const action of ACTIONS){const manifest=fixture(action,{sameFee:action==='contribute-jitosol',shared:true}),filename=path.join(directory,`${action}.manifest.json`);
    fs.writeFileSync(filename,JSON.stringify(manifest,null,2)+'\n',{flag:'wx'});const complete=run(['--manifest',filename]);
    fs.writeFileSync(path.join(directory,`${action}.report.json`),complete.stdout,{flag:'wx'});
    assert.equal(complete.status,0);assert.equal(complete.stderr,'');const parsed=JSON.parse(complete.stdout);assert.equal(parsed.status,'OFFLINE_PLAN_ONLY');
    assert.deepEqual(parsed.plan,a.planManifest(manifest).plan);assert.equal(parsed.plan.native.hex,literal(manifest).bytes.toString('hex'));
    assert.equal(parsed.plan.packets.length,1);assert(parsed.plan.packets[0].wireBytes<=1232);assert.equal(parsed.readiness.publicReady,false);assert.equal(parsed.readiness.funding,null);
  }
  const invalid=path.join(directory,'invalid.public.json');fs.writeFileSync(invalid,'{"unknown":"do-not-echo"}',{flag:'wx'});
  for(const args of [[],['--send'],['--template','--rpc'],['--manifest'],['--manifest',invalid]]){const bad=run(args);assert.notEqual(bad.status,0);assert.equal(bad.stdout,'');assert.equal(JSON.parse(bad.stderr).status,'INVALID_OR_UNSUPPORTED_INTAKE_PLAN');assert(!bad.stderr.includes('do-not-echo'));}
  context.diagnostic(`Three complete public-only CLI manifests/reports retained: ${directory}`);
});
