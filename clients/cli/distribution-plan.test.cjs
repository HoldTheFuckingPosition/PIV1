'use strict';
const assert=require('node:assert/strict'),fs=require('node:fs'),path=require('node:path'),os=require('node:os');
const {spawnSync}=require('node:child_process'),{createHash}=require('node:crypto'),test=require('node:test');
const p=require('./distribution-plan.cjs'),n=require('./initializer-plan.cjs'),{publicPoints}=require('./initializer-plan.test.cjs');
const t=require('../../validation/genesis-transport/transport.cjs'),{PublicKey,VersionedTransaction,TransactionMessage}=require(t.SDK);
const POINTS=publicPoints(),copy=v=>JSON.parse(JSON.stringify(v)),MAX=(1n<<64n)-1n;
const FIXED={system:'11111111111111111111111111111111',token:'TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA',
  mint:'J1toso1uCk3RLmjorhTtrVwY9HJ7X8V9yYac6Y7kGCPn',poolProgram:'SPoo1Ku8WFXoNDMHPsrGSTSG1Y47rzgn41SLUNakuHy',pool:'Jito4APyf642JPZPx3hGc6WWJ8zPKtRbRs4P815Awbb',
  clock:'SysvarC1ock11111111111111111111111111111111',stake:'Stake11111111111111111111111111111111111111'};
const ACTIONS=['prepare-liquid','prepare-withdrawal','settle','integrate-pending'],SIZES=[[1092,1125],[1162,1195],[1125,1158],[861,894]];
function fixture(action='prepare-liquid',{shared=false,revision='9007199254740993',index='4294967295'}={}){
  const m=copy(p.TEMPLATE);Object.assign(m,{programId:POINTS[18],feePayer:POINTS[7],action});
  Object.assign(m.protocol,{validatorList:POINTS[9],reserve:POINTS[10],managerFee:POINTS[11],referrer:shared?POINTS[11]:POINTS[13]});
  if(action.startsWith('prepare-'))Object.assign(m.currentGuardians,{registryRevision:revision,guardianKeys:[5,2,0,4,1,3].map(i=>POINTS[i])});
  if(action==='prepare-withdrawal')Object.assign(m.withdrawal,{validatorIndex:index,sourceStake:POINTS[14]});
  if(action==='settle')Object.assign(m.frozenSettlement,{registryRevision:revision,guardianKeys:[1,5,4,0,3,2].map(i=>POINTS[i]),htfpRecipient:POINTS[6],teamOwnerRecipient:POINTS[8]});
  Object.assign(m.transport,{recentBlockhash:POINTS[16],lastValidBlockHeight:'777777',referenceSlot:'9007199254740997'});
  Object.assign(m.resources,{computeUnits:1400000,heapBytes:262144,loadedAccountBytes:8388608});return m;
}
function literal(m){
  const program=new PublicKey(m.programId),derive=seed=>PublicKey.findProgramAddressSync([Buffer.from(seed)],program)[0].toBase58();
  const row=(address,writable=false)=>({address,signer:false,writable}),prep=m.action.startsWith('prepare-'),settle=m.action==='settle';
  const w=prep?[0,1,2,3,5]:settle?[0,1,3,5,6]:[0,1,2,3,5,7,8];
  // Literal fixed seeds/order; no implementation role builder or derived helper.
  const rows=['config','distribution','pending-sol','principal-sol','operational-sol','distribution-escrow','kif-sol','principal-jito-vault','pending-jito-vault'].map((s,i)=>row(derive(s),w.includes(i)));
  rows.push(row(derive('authority')),row(FIXED.mint),row(FIXED.system),row(FIXED.token),row(FIXED.poolProgram),row(FIXED.pool),row(m.protocol.validatorList),row(m.protocol.reserve),row(m.protocol.managerFee));
  if(m.protocol.referrer!==m.protocol.managerFee)rows.push(row(m.protocol.referrer));
  const c=prep?m.currentGuardians:m.frozenSettlement,rewards=()=>c.guardianKeys.map((g,slot)=>{
    const little=Buffer.from(Array.from({length:8},(_,i)=>Number(BigInt(c.registryRevision)>>BigInt(i*8)&255n)));
    return PublicKey.findProgramAddressSync([Buffer.from('guardian-reward'),new PublicKey(g).toBuffer(),little,Buffer.from([slot])],program)[0].toBase58();});
  if(prep){rows.push(row(derive('guardian-registry')),...rewards().map(a=>row(a)),row(FIXED.clock));if(m.action==='prepare-withdrawal')rows.push(row(FIXED.stake),row(m.withdrawal.sourceStake));}
  else{rows.push(row(FIXED.clock));if(settle)rows.push(row(c.htfpRecipient,true),row(c.teamOwnerRecipient,true),...rewards().map(a=>row(a,true)));}
  const hex={'prepare-liquid':'504956315044303101','prepare-withdrawal':'504956315057303101',settle:'504956315344303101','integrate-pending':'504956314950303101'};
  const suffix=m.action==='prepare-withdrawal'?Buffer.from(Array.from({length:4},(_,i)=>Number(BigInt(m.withdrawal.validatorIndex)>>BigInt(i*8)&255n))):Buffer.alloc(0);
  return {rows,bytes:Buffer.concat([Buffer.from(hex[m.action],'hex'),suffix])};
}
const row=r=>({address:r.address,signer:r.signer,writable:r.writable}),sha=b=>createHash('sha256').update(b).digest('hex');
function* permutations(values,prefix=[]){if(!values.length){yield prefix;return;}for(const v of values)yield* permutations(values.filter(n=>n!==v),[...prefix,v]);}
test('source pins/null manifests preserve all unresolved values and source-only guidance',()=>{
  p.verifySources();assert.deepEqual(JSON.parse(fs.readFileSync(path.join(__dirname,'distribution-plan.example.json'))),p.TEMPLATE);
  for(const m of [{},copy(p.TEMPLATE)]){const out=p.planManifest(m);assert.equal(out.status,'INCOMPLETE');assert.equal(out.plan,null);assert(out.missing.includes('action'));}
  for(const action of ACTIONS){const out=p.planManifest(fixture(action));for(const [field,value]of Object.entries(out.readiness))assert.equal(value,field==='m4Reservation'?'OPEN_UNWAIVED':field.endsWith('Verified')||['publicReady','deploymentAuthorized'].includes(field)?false:null);}
  assert(p.planManifest(fixture()).preconditions.some(s=>s.includes('ten-day')));assert(p.planManifest(fixture('prepare-withdrawal')).preconditions.some(s=>s.includes('24-hour')));
  assert(p.planManifest(fixture('settle')).preconditions.some(s=>s.includes('FROZEN')));assert(p.ERROR_GUIDE.some(e=>e.code===6145));assert(p.ERROR_GUIDE.some(e=>e.code===6073));
});
test('independent literal native selectors/account order/flags and current versus frozen append roles',()=>{
  const counts=[[26,27],[28,29],[27,28],[19,20]];
  ACTIONS.forEach((action,a)=>{for(const shared of [true,false]){const m=fixture(action,{shared}),out=p.planManifest(m),expected=literal(m);
    assert.equal(out.status,'OFFLINE_PLAN_ONLY');assert.deepEqual(out.plan.roles.map(row),expected.rows);assert.equal(out.plan.roles.length,counts[a][shared?0:1]);assert(out.plan.roles.every(r=>!r.signer));
    assert.equal(out.plan.native.hex,expected.bytes.toString('hex'));assert.equal(out.plan.native.bytes,action==='prepare-withdrawal'?13:9);
    const base=shared?18:19;if(action.startsWith('prepare-')){assert.equal(out.plan.roles[base].role,'guardian_registry');assert(out.plan.roles.slice(base,base+8).every(r=>!r.writable));assert.equal(out.plan.guardianContext.kind,'CURRENT_REGISTRY');}
    if(action==='prepare-withdrawal'){assert.equal(out.plan.roles.at(-2).address,FIXED.stake);assert.equal(out.plan.roles.at(-1).address,m.withdrawal.sourceStake);}
    if(action==='settle'){assert.equal(out.plan.roles[base].address,FIXED.clock);assert(!out.plan.roles.some(r=>r.role==='guardian_registry'));assert.equal(out.plan.guardianContext.kind,'FROZEN_ROUND');assert.equal(out.plan.guardianContext.htfpRecipient,m.frozenSettlement.htfpRecipient);}
    if(action==='integrate-pending'){assert.equal(out.plan.guardianContext,null);assert.equal(out.plan.roles.at(-1).address,FIXED.clock);}
  }});
});
test('all720 slot orders and exact u64 revisions/u32 index boundaries retain declared tuple',()=>{
  let count=0;for(const order of permutations([0,1,2,3,4,5])){const m=fixture();m.currentGuardians.guardianKeys=order.map(i=>POINTS[i]);const validated=p.validateManifest(m);
    assert.deepEqual(p.nativeRoles(validated.manifest,validated.derived).map(row),literal(m).rows);assert.deepEqual(validated.manifest.currentGuardians.guardianKeys,m.currentGuardians.guardianKeys);count++;}assert.equal(count,720);
  for(const revision of ['0','1','255','256','4294967296','9007199254740993',MAX.toString()])for(const action of ['prepare-liquid','settle']){
    const m=fixture(action,{revision}),out=p.planManifest(m);assert.deepEqual(out.plan.roles.map(row),literal(m).rows);assert.equal(out.plan.guardianContext.registryRevision,revision);
    const c=action==='settle'?m.frozenSettlement:m.currentGuardians;const previous=copy(m);(action==='settle'?previous.frozenSettlement:previous.currentGuardians).registryRevision=(BigInt(revision)===MAX?0n:BigInt(revision)+1n).toString();
    assert.notDeepEqual(out.plan.roles.filter(r=>r.role.includes('guardian_reward')),p.planManifest(previous).plan.roles.filter(r=>r.role.includes('guardian_reward')));assert.equal(c.guardianKeys.length,6);
  }
  for(const index of ['0','1','255','256','65535','65536','2147483648','4294967295']){const m=fixture('prepare-withdrawal',{index}),b=Buffer.from(p.planManifest(m).plan.native.hex,'hex');assert.deepEqual(b,literal(m).bytes);assert.equal(b.readUInt32LE(9),Number(BigInt(index)));}
});
test('all eight legacy packets count one full zero signature and exact budgets/native privilege unions',()=>{
  ACTIONS.forEach((action,a)=>{for(const shared of [true,false]){const m=fixture(action,{shared}),out=p.planManifest(m),packet=out.plan.packets[0],raw=Buffer.from(packet.wireBase64,'base64'),tx=VersionedTransaction.deserialize(raw),expected=literal(m);
    assert.equal(packet.version,'legacy');assert.equal(packet.wireBytes,SIZES[a][shared?0:1]);assert(raw.length<=1232);assert.equal(packet.signatureCount,1);assert.equal(packet.zeroSignatureBytes,64);assert.deepEqual(packet.requiredSigners,[m.feePayer]);
    assert(tx.signatures[0].every(b=>b===0));assert.deepEqual(Buffer.from(tx.serialize()),raw);assert.equal(packet.sha256,sha(raw));
    const ix=TransactionMessage.decompile(tx.message).instructions;assert.equal(ix.length,4);assert.deepEqual(ix.slice(0,3).map(i=>i.data.toString('hex')),['02c05c1500','0100000400','0400008000']);assert(ix.slice(0,3).every(i=>i.keys.length===0));
    assert.equal(ix[3].programId.toBase58(),m.programId);assert.deepEqual(ix[3].data,expected.bytes);assert.deepEqual(ix[3].keys.map(k=>({address:k.pubkey.toBase58(),signer:k.isSigner,writable:k.isWritable})),expected.rows);
    assert(!ix[3].keys.some(k=>k.pubkey.toBase58()===m.feePayer));
  }});
});
test('native aliases/resources reject even in partial declarations; guardian identity overlaps remain valid',()=>{
  for(const action of ACTIONS){const m=fixture(action);for(const role of p.planManifest(m).plan.roles){const bad=copy(m);bad.feePayer=role.address;assert.throws(()=>p.planManifest(bad));}}
  for(const action of ['prepare-liquid','prepare-withdrawal','settle']){const m=fixture(action),block=action==='settle'?'frozenSettlement':'currentGuardians';m.feePayer=m[block].guardianKeys[0];assert.equal(p.planManifest(m).status,'OFFLINE_PLAN_ONLY');}
  const overlap=fixture('settle');overlap.frozenSettlement.htfpRecipient=overlap.frozenSettlement.guardianKeys[0];assert.equal(p.planManifest(overlap).status,'OFFLINE_PLAN_ONLY');
  const offcurve=fixture();offcurve.currentGuardians.guardianKeys[0]=PublicKey.findProgramAddressSync([Buffer.from('guardian-public-fixture')],new PublicKey(FIXED.poolProgram))[0].toBase58();assert.equal(p.planManifest(offcurve).status,'OFFLINE_PLAN_ONLY');
  for(const [field,values]of [['computeUnits',[0,1400001,1.5,'1']],['heapBytes',[32767,32769,262145,'32768']],['loadedAccountBytes',[0,8388609,-1,'1']]])for(const value of values){const m=copy(p.TEMPLATE);m.resources[field]=value;assert.throws(()=>p.planManifest(m));}
  const small=fixture('integrate-pending');small.resources={computeUnits:1,heapBytes:32768,loadedAccountBytes:1};assert.deepEqual(p.planManifest(small).plan.packets[0].requestedInstructions.slice(0,3).map(i=>i.dataHex),['0201000000','0100800000','0401000000']);
  const same=copy(p.TEMPLATE);same.protocol.reserve=POINTS[9];same.protocol.validatorList=POINTS[9];assert.throws(()=>p.planManifest(same));
  const duplicate=copy(p.TEMPLATE);duplicate.currentGuardians.guardianKeys[0]=POINTS[0];duplicate.currentGuardians.guardianKeys[5]=POINTS[0];assert.throws(()=>p.planManifest(duplicate));
  const protectedAlias=fixture();protectedAlias.currentGuardians.guardianKeys[2]=p.planManifest(protectedAlias).plan.derived.principal_sol;assert.throws(()=>p.planManifest(protectedAlias));
  const rewardAlias=fixture();rewardAlias.protocol.reserve=p.planManifest(rewardAlias).plan.roles.find(r=>r.role==='current_guardian_reward_0').address;rewardAlias.transport.recentBlockhash=null;assert.throws(()=>p.planManifest(rewardAlias));
});
test('strict malformed/unused/current-frozen inputs reject before incomplete output without amount or policy inputs',()=>{
  const changes=[m=>m.schema='other',m=>m.cluster='mainnet-beta',m=>m.action='claim',m=>m.programId=FIXED.pool,m=>m.feePayer='1',m=>m.protocol.reserve=FIXED.clock,
    m=>m.currentGuardians.guardianKeys=[POINTS[0]],m=>m.currentGuardians.guardianKeys='bad',m=>m.currentGuardians.registryRevision='18446744073709551616',m=>m.protocol.extra=1,
    m=>m.transport.referenceSlot='01',m=>m.frozenSettlement.htfpRecipient=POINTS[6],m=>m.amountLamports='1',m=>m.target='1',m=>m.eligibility=true,m=>m.timestamp='0'];
  for(const change of changes){const m=fixture();change(m);assert.throws(()=>p.planManifest(m));m.transport.recentBlockhash=null;assert.throws(()=>p.planManifest(m));}
  for(const value of [0,1,-1,'-1','+1','01','1.0','1e3','4294967296']){const m=copy(p.TEMPLATE);m.withdrawal.validatorIndex=value;assert.throws(()=>p.planManifest(m));}
  for(const value of [0,-1,'-1','01','18446744073709551616']){const m=copy(p.TEMPLATE);m.frozenSettlement.registryRevision=value;assert.throws(()=>p.planManifest(m));}
  for(const action of ACTIONS){const m=fixture(action);for(const field of p.validateManifest(m).missing)assert.fail(field);for(const block of ['protocol','transport','resources'])for(const field of Object.keys(m[block])){const bad=copy(m);bad[block][field]=null;assert.equal(p.planManifest(bad).plan,null);}}
  for(const action of ['prepare-liquid','settle']){const m=fixture(action),b=action==='settle'?'frozenSettlement':'currentGuardians';for(const f of ['registryRevision','guardianKeys']){const bad=copy(m);bad[b][f]=null;assert.equal(p.planManifest(bad).plan,null);}const bad=copy(m);bad[b].guardianKeys[3]=null;assert.equal(p.planManifest(bad).plan,null);}
  for(const action of ['settle','integrate-pending']){const m=fixture(action);m.currentGuardians.registryRevision='1';assert.throws(()=>p.planManifest(m));}
  for(const text of ['{"action":null,"action":null}','{"action":null,"act\\u0069on":null}','[]','{"unknown":"do-not-echo"}'])assert.throws(()=>p.planManifest(n.parseManifest(text)));
});
test('CLI retains all eight complete shared/distinct reports in fresh directory without readiness claims',context=>{
  const script=path.join(__dirname,'distribution-plan.cjs'),limits=process.execArgv.filter(v=>v.startsWith('--max-old-space-size'));
  const run=args=>spawnSync(process.execPath,[...limits,script,...args],{encoding:'utf8',maxBuffer:2*1024*1024,timeout:30000});
  const directory=fs.mkdtempSync(path.join(os.tmpdir(),'piv1-distribution-public-test-'));
  ACTIONS.forEach((action,a)=>{for(const shared of [true,false]){const name=`${action}-${shared?'shared':'distinct'}`,m=fixture(action,{shared,revision:shared?'9007199254740993':MAX.toString()}),filename=path.join(directory,`${name}.manifest.json`);
    fs.writeFileSync(filename,JSON.stringify(m,null,2)+'\n',{flag:'wx'});const complete=run(['--manifest',filename]);fs.writeFileSync(path.join(directory,`${name}.report.json`),complete.stdout,{flag:'wx'});
    assert.equal(complete.status,0);assert.equal(complete.stderr,'');const parsed=JSON.parse(complete.stdout);assert.equal(parsed.status,'OFFLINE_PLAN_ONLY');assert.deepEqual(parsed.plan,p.planManifest(m).plan);
    assert.equal(parsed.plan.native.hex,literal(m).bytes.toString('hex'));assert.equal(parsed.plan.packets[0].wireBytes,SIZES[a][shared?0:1]);assert.equal(parsed.readiness.publicReady,false);assert.equal(parsed.readiness.eligibility,null);
  }});
  const invalid=path.join(directory,'invalid.public.json');fs.writeFileSync(invalid,'{"unknown":"do-not-echo"}',{flag:'wx'});
  for(const args of [[],['--send'],['--template','--rpc'],['--manifest'],['--manifest',invalid]]){const bad=run(args);assert.notEqual(bad.status,0);assert.equal(bad.stdout,'');assert.equal(JSON.parse(bad.stderr).status,'INVALID_OR_UNSUPPORTED_DISTRIBUTION_PLAN');assert(!bad.stderr.includes('do-not-echo'));}
  context.diagnostic(`All eight complete public distribution reports retained: ${directory}`);
});
