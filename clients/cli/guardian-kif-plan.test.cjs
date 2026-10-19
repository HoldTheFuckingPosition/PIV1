'use strict';
const assert=require('node:assert/strict'),fs=require('node:fs'),path=require('node:path'),os=require('node:os');
const {spawnSync}=require('node:child_process'),{createHash}=require('node:crypto'),test=require('node:test');
const p=require('./guardian-kif-plan.cjs'),n=require('./initializer-plan.cjs'),{publicPoints}=require('./initializer-plan.test.cjs');
const t=require('../../validation/genesis-transport/transport.cjs'),{PublicKey,VersionedTransaction}=require(t.SDK);
const POINTS=publicPoints(),copy=v=>JSON.parse(JSON.stringify(v)),MAX=(1n<<64n)-1n;
const F={system:'11111111111111111111111111111111',clock:'SysvarC1ock11111111111111111111111111111111',loader:'BPFLoaderUpgradeab1e11111111111111111111111',squads:'SQDS4ep65T869zMMBKyuUq6aD6EgTu8psMjkvj52pCf',compute:'ComputeBudget111111111111111111111111111111'};
const ACTIONS=['guardian-heartbeat','claim-kif'],SIZES=[[601,697],[382,478]],sha=b=>createHash('sha256').update(b).digest('hex');
const little=(v,width=8)=>Buffer.from(Array.from({length:width},(_,i)=>Number(BigInt(v)>>BigInt(i*8)&255n)));
const derive=(program,seeds)=>{const [k,bump]=PublicKey.findProgramAddressSync(seeds,new PublicKey(program));return {address:k.toBase58(),bump};};
function fixture(action=ACTIONS[0],{shared=false,revision='9007199254740993',slot=4,vault=255,amount='9007199254740995',counter='9007199254740997'}={}){
  const m=copy(p.TEMPLATE);Object.assign(m,{programId:POINTS[18],action});let guardian;
  if(action===ACTIONS[0]){const createKey=POINTS[12],multisig=derive(F.squads,[Buffer.from('multisig'),Buffer.from('multisig'),new PublicKey(createKey).toBuffer()]);Object.assign(m.heartbeat,{registryRevision:revision,guardianKeys:[5,2,0,4,1,3].map(i=>POINTS[i]),guardianIndex:slot,vaultIndex:vault,createKey,multisigAddress:multisig.address});guardian=m.heartbeat.guardianKeys[slot];}
  else{Object.assign(m.claim,{guardian:POINTS[8],registryRevision:revision,guardianIndex:slot,amountLamports:amount,expectedCumulativeClaimed:counter});guardian=m.claim.guardian;}
  m.feePayer=shared?guardian:POINTS[7];Object.assign(m.transport,{recentBlockhash:POINTS[16],lastValidBlockHeight:'777777',referenceSlot:'9007199254740999'});Object.assign(m.resources,{computeUnits:1400000,heapBytes:262144,loadedAccountBytes:8388608});return m;
}
function literal(m){
  const r=(address,writable=false,signer=false)=>({address,writable,signer}),fixed=s=>derive(m.programId,[Buffer.from(s)]).address;
  const rw=(g,v,i)=>derive(m.programId,[Buffer.from('guardian-reward'),new PublicKey(g).toBuffer(),little(v),Buffer.from([i])]);
  if(m.action===ACTIONS[0]){const h=m.heartbeat,rewards=h.guardianKeys.map((g,i)=>rw(g,h.registryRevision,i));const rows=[r(fixed('config')),r(fixed('guardian-registry')),...rewards.map((x,i)=>r(x.address,i===h.guardianIndex)),r(F.clock),r(h.guardianKeys[h.guardianIndex],false,true),r(m.programId),r(derive(F.loader,[new PublicKey(m.programId).toBuffer()]).address),r(h.multisigAddress)];
    return {rows,bytes:Buffer.from([...Buffer.from('504956314842303101','hex'),h.guardianIndex,h.vaultIndex]),rewards};}
  const c=m.claim,reward=rw(c.guardian,c.registryRevision,c.guardianIndex);
  return {rows:[r(fixed('config'),true),r(reward.address,true),r(fixed('kif-sol'),true),r(c.guardian,true,true),r(F.system)],bytes:Buffer.concat([Buffer.from('fd97ac0bca4d76aa','hex'),little(c.amountLamports),little(c.expectedCumulativeClaimed)]),reward};
}
function rawPacket(raw){let at=0;const byte=()=>{assert(at<raw.length);return raw[at++];},take=n=>{assert(at+n<=raw.length);const b=raw.subarray(at,at+n);at+=n;return b;};const sv=()=>{let v=0,s=0,b;do{b=byte();v+=(b&127)*2**s;s+=7;assert(s<=21);}while(b&128);return v;};
  const signatures=Array.from({length:sv()},()=>take(64)),header=[byte(),byte(),byte()];assert(header[0]<128);const keys=Array.from({length:sv()},()=>new PublicKey(take(32)).toBase58()),blockhash=new PublicKey(take(32)).toBase58();
  const instructions=Array.from({length:sv()},()=>{const program=keys[byte()],indices=Array.from({length:sv()},byte),data=take(sv());return {program,data,indices,rows:indices.map(i=>({address:keys[i],signer:i<header[0],writable:i<header[0]?i<header[0]-header[1]:i<keys.length-header[2]}))};});assert.equal(at,raw.length);return {signatures,header,keys,blockhash,instructions};}
const row=r=>({address:r.address,writable:r.writable,signer:r.signer});
function checkWire(m,out){const a=ACTIONS.indexOf(m.action),shared=m.feePayer===out.plan.selectedGuardian,packet=out.plan.packets[0],raw=Buffer.from(packet.wireBase64,'base64'),d=rawPacket(raw),expected=literal(m),union=expected.rows.map(r=>({...r,writable:r.writable||r.address===m.feePayer}));
  assert.equal(raw.length,SIZES[a][shared?0:1]);assert.equal(packet.wireBytes,raw.length);assert(raw.length<=1232);assert.equal(packet.version,'legacy');assert.equal(packet.sha256,sha(raw));assert.equal(d.signatures.length,shared?1:2);assert(d.signatures.every(s=>s.every(b=>b===0)));assert.equal(packet.zeroSignatureBytes,64*d.signatures.length);assert.equal(packet.signatureCount,d.signatures.length);
  assert.deepEqual(packet.requiredSigners,[...new Set([m.feePayer,out.plan.selectedGuardian])]);assert.equal(d.keys[0],m.feePayer);assert.equal(d.blockhash,m.transport.recentBlockhash);assert.equal(d.header[1],a===0&&!shared?1:0);assert.equal(d.keys.length,(a===0?14:7)+(shared?0:1));
  assert.equal(d.instructions.length,4);assert.deepEqual(d.instructions.slice(0,3).map(i=>({p:i.program,n:i.rows.length,hex:i.data.toString('hex')})),['02c05c1500','0100000400','0400008000'].map(hex=>({p:F.compute,n:0,hex})));
  assert.equal(d.instructions[3].program,m.programId);assert.deepEqual(d.instructions[3].data,expected.bytes);assert.deepEqual(d.instructions[3].rows,union);assert.deepEqual(packet.requestedInstructions[3].accounts,expected.rows);assert.deepEqual(packet.wireInstructions[3].accounts,union);assert.deepEqual(Buffer.from(VersionedTransaction.deserialize(raw).serialize()),raw);
  assert.equal(d.keys.filter(k=>k===m.programId).length,1);if(a===0)assert.equal(d.keys[d.instructions[3].indices[10]],m.programId);return d;
}
test('source binding and null manifests retain unresolved control/economics/readiness',()=>{
  p.verifySources();assert.deepEqual(JSON.parse(fs.readFileSync(path.join(__dirname,'guardian-kif-plan.example.json'))),p.TEMPLATE);assert.equal(sha(Buffer.from('global:claim_kif')).slice(0,16),'fd97ac0bca4d76aa');
  for(const m of [{},copy(p.TEMPLATE)]){const out=p.planManifest(m);assert.equal(out.status,'INCOMPLETE');assert.equal(out.plan,null);assert(out.missing.includes('action'));}
  for(const action of ACTIONS){const out=p.planManifest(fixture(action));for(const [f,v]of Object.entries(out.readiness))assert.equal(v,f==='m4Reservation'?'OPEN_UNWAIVED':f.endsWith('Verified')||['publicReady','deploymentAuthorized'].includes(f)?false:null);assert(out.preconditions.some(s=>s.includes('unsupported by this planner, not forbidden by production')));}
});
test('independent literal payload/roles/PDA/gov declarations preserve current versus earned tuples',()=>{
  for(const action of ACTIONS)for(const shared of [true,false]){const m=fixture(action,{shared}),out=p.planManifest(m),e=literal(m);assert.equal(out.status,'OFFLINE_PLAN_ONLY');assert.deepEqual(out.plan.roles.map(row),e.rows);assert.equal(out.plan.native.hex,e.bytes.toString('hex'));assert.equal(out.plan.native.bytes,action===ACTIONS[0]?11:24);assert.equal(out.plan.roles.length,action===ACTIONS[0]?13:5);
    if(action===ACTIONS[0]){assert.deepEqual(out.plan.derived.currentRewards,e.rewards);assert.equal(out.plan.guardianContext.kind,'CURRENT_REGISTRY');assert.equal(out.plan.derived.multisig.address,m.heartbeat.multisigAddress);assert.equal(out.plan.derived.program_data.address,e.rows[11].address);assert.deepEqual(out.plan.derived.vault,derive(F.squads,[Buffer.from('multisig'),new PublicKey(m.heartbeat.multisigAddress).toBuffer(),Buffer.from('vault'),Buffer.from([m.heartbeat.vaultIndex])]));assert.equal(out.plan.roles[9].writable,false);assert.equal(out.plan.roles.filter(r=>r.writable).length,1);assert.equal(out.plan.requestedClaim,null);}
    else{assert.deepEqual(out.plan.derived.guardian_reward,e.reward);assert.equal(out.plan.guardianContext.kind,'EARNED_REWARD');assert.deepEqual(out.plan.requestedClaim,{amountLamports:m.claim.amountLamports,expectedCumulativeClaimed:m.claim.expectedCumulativeClaimed});assert.equal(out.plan.roles.filter(r=>r.writable).length,4);assert(!out.plan.roles.some(r=>r.role==='clock'||r.role==='guardian_registry'||r.role==='multisig'));assert(out.preconditions.some(s=>s.includes('allowed during global pause')));}
  }
});
test('finite u64/revision/slot/vault boundaries retain exact source bytes and checked-policy unknowns',()=>{
  const values=['0','1','127','128','255','256','65535','65536','4294967295','4294967296','9007199254740993',MAX.toString()];
  for(const revision of values)for(const action of ACTIONS){const m=fixture(action,{revision}),out=p.planManifest(m);assert.equal(out.plan.native.hex,literal(m).bytes.toString('hex'));assert.deepEqual(out.plan.roles.map(row),literal(m).rows);const changed=copy(m);changed[action===ACTIONS[0]?'heartbeat':'claim'].registryRevision=(BigInt(revision)===MAX?0n:BigInt(revision)+1n).toString();assert.notDeepEqual(out.plan.roles,p.planManifest(changed).plan.roles);}
  for(const slot of [0,1,2,3,4,5])for(const vault of [0,1,127,128,255]){const m=fixture(ACTIONS[0],{slot,vault});assert.equal(p.planManifest(m).plan.native.hex,literal(m).bytes.toString('hex'));assert.deepEqual(p.planManifest(m).plan.roles.map(row),literal(m).rows);}
  for(const [amount,counter]of [['1','0'],[MAX.toString(),MAX.toString()],['9007199254740993','9007199254740995'],['255','256'],['4294967295','4294967296']]){const m=fixture(ACTIONS[1],{amount,counter}),out=p.planManifest(m);assert.equal(out.plan.native.hex,literal(m).bytes.toString('hex'));assert.equal(out.readiness.claimable,null);assert.equal(out.readiness.liabilityBacking,null);}
  for(const [block,f]of [['heartbeat','registryRevision'],['claim','registryRevision'],['claim','amountLamports'],['claim','expectedCumulativeClaimed']])for(const value of [0,1,-1,'-1','+1','01','1.0','1e3',(MAX+1n).toString()]){const m=copy(p.TEMPLATE);m[block][f]=value;assert.throws(()=>p.planManifest(m));}
  const zero=fixture(ACTIONS[1],{amount:'0'});assert.throws(()=>p.planManifest(zero));
  for(const [block,f,max]of [['heartbeat','guardianIndex',5],['claim','guardianIndex',5],['heartbeat','vaultIndex',255]])for(const value of [-1,max+1,1.5,'0','1']){const m=copy(p.TEMPLATE);m[block][f]=value;assert.throws(()=>p.planManifest(m));}
});
test('independent raw wire verifies full signatures/program unions and readonly guardian escalation',()=>{
  for(const action of ACTIONS)for(const shared of [true,false]){const m=fixture(action,{shared});checkWire(m,p.planManifest(m));}
  const unselected=fixture();unselected.feePayer=unselected.heartbeat.guardianKeys[0];checkWire(unselected,p.planManifest(unselected));
  const contextOverlap=fixture(ACTIONS[1]);contextOverlap.feePayer=contextOverlap.transport.recentBlockhash;checkWire(contextOverlap,p.planManifest(contextOverlap));
});
test('direct signer scope/known-partial aliases/gov bindings/resources reject without blanket guardian curve check',()=>{
  const offcurve=derive(F.squads,[Buffer.from('unselected-public-pda')]).address;
  const other=fixture();other.heartbeat.guardianKeys[0]=offcurve;assert.equal(p.planManifest(other).status,'OFFLINE_PLAN_ONLY');
  const unsupported=copy(p.TEMPLATE);unsupported.claim.guardian=offcurve;assert.throws(()=>p.planManifest(unsupported));const selected=fixture();selected.heartbeat.guardianKeys[selected.heartbeat.guardianIndex]=offcurve;assert.throws(()=>p.planManifest(selected));
  for(const action of ACTIONS){const m=fixture(action),out=p.planManifest(m);for(const r of out.plan.roles)if(r.address!==out.plan.selectedGuardian){const bad=copy(m);bad.feePayer=r.address;bad.transport.recentBlockhash=null;assert.throws(()=>p.planManifest(bad));}}
  const dup=fixture();dup.heartbeat.guardianKeys[0]=dup.heartbeat.guardianKeys[1];assert.throws(()=>p.planManifest(dup));const internal=fixture();internal.heartbeat.guardianKeys[0]=p.planManifest(internal).plan.derived.config;assert.throws(()=>p.planManifest(internal));
  const mismatch=fixture();mismatch.heartbeat.multisigAddress=POINTS[10];mismatch.transport.recentBlockhash=null;assert.throws(()=>p.planManifest(mismatch));
  const rawPda=fixture();rawPda.programId=derive(F.squads,[Buffer.from('unverified-program-pda')]).address;assert.equal(p.planManifest(rawPda).status,'OFFLINE_PLAN_ONLY');
  for(const [f,values]of [['computeUnits',[0,1400001,1.5,'1400000']],['heapBytes',[32767,32769,262145,'32768']],['loadedAccountBytes',[0,8388609,-1,'8388608']]])for(const value of values){const m=copy(p.TEMPLATE);m.resources[f]=value;assert.throws(()=>p.planManifest(m));}
  const small=fixture();small.resources={computeUnits:1,heapBytes:32768,loadedAccountBytes:1};assert.deepEqual(p.planManifest(small).plan.packets[0].requestedInstructions.slice(0,3).map(i=>i.dataHex),['0201000000','0100800000','0401000000']);
});
test('strict malformed/unknown/unused/duplicate action fields fail before missing report',()=>{
  const changes=[m=>m.schema='other',m=>m.cluster='mainnet-beta',m=>m.action='set-pause',m=>m.programId=F.squads,m=>m.feePayer='1',m=>m.heartbeat.guardianKeys=[POINTS[0]],m=>m.heartbeat.guardianKeys='bad',m=>m.heartbeat.extra=1,m=>m.claim.amountLamports='1',m=>m.timestamp='0',m=>m.periodId='0',m=>m.active=true,m=>m.destination=POINTS[9],m=>m.source=POINTS[10],m=>m.transport.referenceSlot='01'];
  for(const change of changes){const m=fixture();change(m);assert.throws(()=>p.planManifest(m));m.transport.recentBlockhash=null;assert.throws(()=>p.planManifest(m));}
  const currentInClaim=fixture(ACTIONS[1]);currentInClaim.heartbeat.registryRevision='0';assert.throws(()=>p.planManifest(currentInClaim));
  for(const action of ACTIONS){const m=fixture(action),block=action===ACTIONS[0]?'heartbeat':'claim';for(const f of ['schema','cluster','action','programId','feePayer','transport','resources',block]){const bad=copy(m);bad[f]=null;assert.equal(p.planManifest(bad).plan,null);}
    for(const b of ['transport','resources',block])for(const f of Object.keys(m[b])){const bad=copy(m);bad[b][f]=null;assert.equal(p.planManifest(bad).plan,null);}if(block==='heartbeat'){const bad=copy(m);bad.heartbeat.guardianKeys[3]=null;assert.equal(p.planManifest(bad).plan,null);}}
  for(const text of ['{"action":null,"action":null}','{"action":null,"act\\u0069on":null}','[]','{"unknown":"do-not-echo"}'])assert.throws(()=>p.planManifest(n.parseManifest(text)));
});
test('CLI retains four complete public action/shared-distinct payer reports and rejects unsupported inputs',context=>{
  const script=path.join(__dirname,'guardian-kif-plan.cjs'),limits=process.execArgv.filter(v=>v.startsWith('--max-old-space-size'));
  const run=args=>spawnSync(process.execPath,[...limits,script,...args],{encoding:'utf8',maxBuffer:2*1024*1024,timeout:30000});const template=run(['--template']);assert.equal(template.status,0);assert.equal(template.stderr,'');assert.deepEqual(JSON.parse(template.stdout),p.TEMPLATE);
  const incomplete=run(['--manifest',path.join(__dirname,'guardian-kif-plan.example.json')]);assert.equal(incomplete.status,0);assert.equal(JSON.parse(incomplete.stdout).status,'INCOMPLETE');assert.equal(JSON.parse(incomplete.stdout).plan,null);
  const directory=fs.mkdtempSync(path.join(os.tmpdir(),'piv1-guardian-kif-public-test-'));
  for(const action of ACTIONS)for(const shared of [true,false]){const name=`${action}-${shared?'shared':'distinct'}`,m=fixture(action,{shared,revision:shared?'9007199254740993':MAX.toString(),slot:shared?0:5}),filename=path.join(directory,`${name}.manifest.json`);fs.writeFileSync(filename,JSON.stringify(m,null,2)+'\n',{flag:'wx'});const result=run(['--manifest',filename]);fs.writeFileSync(path.join(directory,`${name}.report.json`),result.stdout,{flag:'wx'});assert.equal(result.status,0);assert.equal(result.stderr,'');const out=JSON.parse(result.stdout);assert.equal(out.status,'OFFLINE_PLAN_ONLY');assert.deepEqual(out.plan,p.planManifest(m).plan);checkWire(m,out);assert.equal(out.readiness.publicReady,false);assert.equal(out.readiness.eligibility,null);}
  const invalid=path.join(directory,'invalid.public.json');fs.writeFileSync(invalid,'{"unknown":"do-not-echo"}',{flag:'wx'});for(const args of [[],['--send'],['--rpc','https://invalid.example'],['--template','--rpc'],['--manifest'],['--manifest',invalid],['--keypair','do-not-echo']]){const bad=run(args);assert.notEqual(bad.status,0);assert.equal(bad.stdout,'');assert.equal(JSON.parse(bad.stderr).status,'INVALID_OR_UNSUPPORTED_GUARDIAN_KIF_PLAN');assert(!bad.stderr.includes('do-not-echo'));}
  context.diagnostic(`Four complete public guardian/KIF reports retained: ${directory}`);
});
