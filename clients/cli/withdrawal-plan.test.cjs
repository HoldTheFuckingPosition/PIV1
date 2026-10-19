'use strict';
const assert=require('node:assert/strict'),fs=require('node:fs'),path=require('node:path'),os=require('node:os');
const {spawnSync}=require('node:child_process'),{createHash}=require('node:crypto'),test=require('node:test');
const p=require('./withdrawal-plan.cjs'),n=require('./initializer-plan.cjs'),{publicPoints}=require('./initializer-plan.test.cjs');
const t=require('../../validation/genesis-transport/transport.cjs'),{PublicKey,VersionedTransaction}=require(t.SDK);
const POINTS=publicPoints(),copy=v=>JSON.parse(JSON.stringify(v)),MAX=(1n<<64n)-1n;
const FIXED={system:'11111111111111111111111111111111',token:'TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA',mint:'J1toso1uCk3RLmjorhTtrVwY9HJ7X8V9yYac6Y7kGCPn',poolProgram:'SPoo1Ku8WFXoNDMHPsrGSTSG1Y47rzgn41SLUNakuHy',pool:'Jito4APyf642JPZPx3hGc6WWJ8zPKtRbRs4P815Awbb',clock:'SysvarC1ock11111111111111111111111111111111',stake:'Stake11111111111111111111111111111111111111',history:'SysvarStakeHistory1111111111111111111111111',compute:'ComputeBudget111111111111111111111111111111'};
const ACTIONS=['initiate-withdrawal-leg','finalize-withdrawal-leg'],SIZES=[[1030,1063],[1001,1034]];
const little=(value,width)=>Buffer.from(Array.from({length:width},(_,i)=>Number(BigInt(value)>>BigInt(i*8)&255n)));
const sha=b=>createHash('sha256').update(b).digest('hex'),row=r=>({address:r.address,signer:r.signer,writable:r.writable});
function fixture(action=ACTIONS[0],{shared=false,sequence='9007199254740993',index='9007199254740995',validator='4294967295'}={}){
  const m=copy(p.TEMPLATE);Object.assign(m,{programId:POINTS[18],feePayer:POINTS[7],action,roundSequence:sequence});
  Object.assign(m.protocol,{validatorList:POINTS[9],reserve:POINTS[10],managerFee:POINTS[11],referrer:shared?POINTS[11]:POINTS[13]});
  if(action===ACTIONS[0])Object.assign(m.initiation,{nextLegIndex:index,validatorIndex:validator,sourceStake:POINTS[14]});else m.finalization.legIndex=index;
  Object.assign(m.transport,{recentBlockhash:POINTS[16],lastValidBlockHeight:'777777',referenceSlot:'9007199254740997'});
  Object.assign(m.resources,{computeUnits:1400000,heapBytes:262144,loadedAccountBytes:8388608});return m;
}
function literal(m){
  const program=new PublicKey(m.programId),initiate=m.action===ACTIONS[0],index=initiate?m.initiation.nextLegIndex:m.finalization.legIndex;
  const derive=seed=>PublicKey.findProgramAddressSync([Buffer.from(seed)],program)[0].toBase58(),r=(address,writable=false)=>({address,signer:false,writable});
  const writes=initiate?[0,1,2,4,7]:[0,1,2,4,5];
  const rows=['config','distribution','pending-sol','principal-sol','operational-sol','distribution-escrow','kif-sol','principal-jito-vault','pending-jito-vault'].map((s,i)=>r(derive(s),writes.includes(i)));
  rows.push(r(derive('authority')),r(FIXED.mint,initiate),r(FIXED.system),r(FIXED.token),r(FIXED.poolProgram),r(FIXED.pool,initiate),r(m.protocol.validatorList,initiate),r(m.protocol.reserve),r(m.protocol.managerFee,initiate));
  if(m.protocol.managerFee!==m.protocol.referrer)rows.push(r(m.protocol.referrer));rows.push(r(FIXED.clock),r(FIXED.stake));
  const withdraw=PublicKey.findProgramAddressSync([new PublicKey(FIXED.pool).toBuffer(),Buffer.from('withdraw')],new PublicKey(FIXED.poolProgram))[0].toBase58();
  if(initiate)rows.push(r(m.initiation.sourceStake,true),r(withdraw));else rows.push(r(FIXED.history));
  const temp={};for(const seed of ['withdrawal-leg','withdrawal-stake']){const [key,bump]=PublicKey.findProgramAddressSync([Buffer.from(seed),little(m.roundSequence,8),little(index,8)],program);temp[seed]={address:key.toBase58(),bump};rows.push(r(key.toBase58(),true));}
  const bytes=Buffer.concat([Buffer.from(initiate?'50495631494c303101':'50495631464c303101','hex'),little(initiate?m.initiation.validatorIndex:index,initiate?4:8)]);
  return {rows,bytes,temp,withdraw};
}
// Independent raw legacy shortvec/header/account/instruction cursor, no planner decoder.
function rawPacket(raw){
  let at=0;const byte=()=>{assert(at<raw.length);return raw[at++];},take=n=>{assert(at+n<=raw.length);const b=raw.subarray(at,at+n);at+=n;return b;};
  const sv=()=>{let v=0,shift=0,b;do{b=byte();v+=(b&127)*2**shift;shift+=7;assert(shift<=21);}while(b&128);return v;};
  const count=sv(),signatures=Array.from({length:count},()=>take(64)),header=[byte(),byte(),byte()];assert(header[0]<128);
  const keys=Array.from({length:sv()},()=>new PublicKey(take(32)).toBase58()),blockhash=new PublicKey(take(32)).toBase58();
  const instructions=Array.from({length:sv()},()=>{const program=keys[byte()],indices=Array.from({length:sv()},byte),data=take(sv());return {program,data,rows:indices.map(i=>({address:keys[i],signer:i<header[0],writable:i<header[0]?i<header[0]-header[1]:i<keys.length-header[2]}))};});
  assert.equal(at,raw.length);return {signatures,header,keys,blockhash,instructions};
}
test('source pins/null manifests keep no readiness or economic result',()=>{
  p.verifySources();assert.deepEqual(JSON.parse(fs.readFileSync(path.join(__dirname,'withdrawal-plan.example.json'))),p.TEMPLATE);
  for(const m of [{},copy(p.TEMPLATE)]){const out=p.planManifest(m);assert.equal(out.status,'INCOMPLETE');assert.equal(out.plan,null);assert(out.missing.includes('action'));}
  for(const action of ACTIONS){const out=p.planManifest(fixture(action));for(const [f,v]of Object.entries(out.readiness))assert.equal(v,f==='m4Reservation'?'OPEN_UNWAIVED':f.endsWith('Verified')||['publicReady','deploymentAuthorized'].includes(f)?false:null);assert(out.preconditions.some(s=>s.includes('OperationalSOL')));assert(out.preconditions.some(s=>s.includes('unpaused')));}
  assert(p.planManifest(fixture(ACTIONS[0])).preconditions.some(s=>s.includes('next_leg_index')));assert(p.planManifest(fixture(ACTIONS[1])).preconditions.some(s=>s.includes('Stake History')));
  assert(p.ERROR_GUIDE.some(e=>e.code===6073));
});
test('literal source-derived payload/order/privileges/PDA seeds for shared/distinct paths',()=>{
  ACTIONS.forEach((action,a)=>{for(const shared of [true,false]){const m=fixture(action,{shared}),out=p.planManifest(m),expected=literal(m),base=shared?18:19;
    assert.equal(out.status,'OFFLINE_PLAN_ONLY');assert.deepEqual(out.plan.roles.map(row),expected.rows);assert.equal(out.plan.roles.length,(a===0?24:23)+(shared?0:1));assert(out.plan.roles.every(r=>!r.signer));
    assert.equal(out.plan.native.hex,expected.bytes.toString('hex'));assert.equal(out.plan.native.bytes,a===0?13:17);assert.equal(out.plan.roles[base].address,FIXED.clock);assert.equal(out.plan.roles[base+1].address,FIXED.stake);
    for(const [role,seed]of [['withdrawal_leg','withdrawal-leg'],['withdrawal_stake','withdrawal-stake']])assert.deepEqual(out.plan.derived[role],expected.temp[seed]);
    assert.equal(out.plan.roles[16].writable,false);assert.equal(out.plan.roles.find(r=>r.role==='referrer')?.writable??false,false);
    if(a===0){assert.equal(out.plan.derived.pool_withdraw_authority,expected.withdraw);assert.equal(out.plan.roles[base+3].address,expected.withdraw);}else{assert.equal(out.plan.roles[base+2].address,FIXED.history);assert(!out.plan.roles.some(r=>r.role==='pool_withdraw_authority'));}
  }});
});
test('full-width round/leg seed declarations and validator-index boundaries encode exactly without eligibility inference',()=>{
  const values=['0','1','127','128','255','256','65535','65536','4294967295','4294967296','9007199254740993',MAX.toString()];
  for(const action of ACTIONS)for(const [sequence,index]of values.map((v,i)=>[v,values[(i+3)%values.length]])){
    const m=fixture(action,{sequence,index}),out=p.planManifest(m),expected=literal(m);assert.deepEqual(out.plan.roles.map(row),expected.rows);assert.equal(out.plan.native.hex,expected.bytes.toString('hex'));
    assert.equal(out.plan.roundDeclaration.roundSequence,sequence);assert.equal(out.plan.roundDeclaration.legIndex,index);assert.equal(out.readiness.eligibility,null);
    const changed=copy(m);changed.roundSequence=(BigInt(sequence)===MAX?0n:BigInt(sequence)+1n).toString();assert.notDeepEqual(out.plan.roles.slice(-2),p.planManifest(changed).plan.roles.slice(-2));
    if(action===ACTIONS[0]){changed.initiation.nextLegIndex=(BigInt(index)===MAX?0n:BigInt(index)+1n).toString();assert.equal(out.plan.native.hex,p.planManifest(changed).plan.native.hex);}
  }
  for(const validator of ['0','1','255','256','65535','65536','2147483648','4294967295']){const m=fixture(ACTIONS[0],{validator});assert.equal(p.planManifest(m).plan.native.hex,literal(m).bytes.toString('hex'));}
  for(const [block,field,max]of [[null,'roundSequence',MAX],['initiation','nextLegIndex',MAX],['finalization','legIndex',MAX],['initiation','validatorIndex',(1n<<32n)-1n]])for(const v of [0,1,-1,'-1','+1','01','1.0','1e3',(max+1n).toString()]){
    const m=copy(p.TEMPLATE);(block?m[block]:m)[field]=v;assert.throws(()=>p.planManifest(m));}
});
test('independent full raw legacy wire decode counts zero signature and exact budgets/privilege unions',()=>{
  ACTIONS.forEach((action,a)=>{for(const shared of [true,false]){const m=fixture(action,{shared}),out=p.planManifest(m),packet=out.plan.packets[0],raw=Buffer.from(packet.wireBase64,'base64'),decoded=rawPacket(raw),expected=literal(m);
    assert.equal(packet.wireBytes,SIZES[a][shared?0:1]);assert.equal(raw.length,packet.wireBytes);assert(raw.length<=1232);assert.equal(packet.version,'legacy');assert.equal(packet.signatureCount,1);assert.equal(packet.zeroSignatureBytes,64);assert.deepEqual(packet.requiredSigners,[m.feePayer]);
    assert.equal(decoded.signatures.length,1);assert(decoded.signatures[0].every(b=>b===0));assert.equal(decoded.header[0],1);assert.equal(decoded.keys[0],m.feePayer);assert.equal(decoded.blockhash,m.transport.recentBlockhash);assert.equal(packet.sha256,sha(raw));
    assert.deepEqual(decoded.instructions.slice(0,3).map(i=>({program:i.program,rows:i.rows,hex:i.data.toString('hex')})),['02c05c1500','0100000400','0400008000'].map(hex=>({program:FIXED.compute,rows:[],hex})));
    assert.equal(decoded.instructions.length,4);assert.equal(decoded.instructions[3].program,m.programId);assert.deepEqual(decoded.instructions[3].data,expected.bytes);assert.deepEqual(decoded.instructions[3].rows,expected.rows);
    assert.deepEqual(packet.requestedInstructions[3].accounts,expected.rows);assert.deepEqual(packet.wireInstructions[3].accounts,expected.rows);assert.deepEqual(Buffer.from(VersionedTransaction.deserialize(raw).serialize()),raw);
  }});
});
test('resources/partial aliases/curve rules reject while reference-blockhash payer overlap remains declarative',()=>{
  for(const action of ACTIONS){const m=fixture(action),out=p.planManifest(m);for(const role of out.plan.roles){const bad=copy(m);bad.feePayer=role.address;assert.throws(()=>p.planManifest(bad));}
    for(const f of Object.keys(m.protocol))for(const address of [m.programId,p.POOL_WITHDRAW_AUTHORITY,...out.plan.roles.filter(r=>r.role==='withdrawal_leg'||r.role==='withdrawal_stake').map(r=>r.address)]){const bad=copy(m);bad.protocol[f]=address;bad.transport.recentBlockhash=null;assert.throws(()=>p.planManifest(bad));}
    const overlap=copy(m);overlap.feePayer=overlap.transport.recentBlockhash;assert.equal(p.planManifest(overlap).plan.packets[0].signatureCount,1);assert.equal(p.planManifest(overlap).readiness.signerControlVerified,false);
    const offcurve=copy(m);offcurve.programId=PublicKey.findProgramAddressSync([Buffer.from('unverified-program')],new PublicKey(FIXED.poolProgram))[0].toBase58();offcurve.protocol.validatorList=PublicKey.findProgramAddressSync([Buffer.from('unverified-list')],new PublicKey(FIXED.poolProgram))[0].toBase58();assert.equal(p.planManifest(offcurve).status,'OFFLINE_PLAN_ONLY');
  }
  const alias=fixture();alias.protocol.reserve=alias.protocol.validatorList;assert.throws(()=>p.planManifest(alias));const source=fixture();source.initiation.sourceStake=source.protocol.managerFee;assert.throws(()=>p.planManifest(source));
  for(const [field,values]of [['computeUnits',[0,1400001,1.5,'1400000']],['heapBytes',[32767,32769,262145,'32768']],['loadedAccountBytes',[0,8388609,-1,'8388608']]])for(const value of values){const m=copy(p.TEMPLATE);m.resources[field]=value;assert.throws(()=>p.planManifest(m));}
  const small=fixture();small.resources={computeUnits:1,heapBytes:32768,loadedAccountBytes:1};assert.deepEqual(p.planManifest(small).plan.packets[0].requestedInstructions.slice(0,3).map(i=>i.dataHex),['0201000000','0100800000','0401000000']);
});
test('strict schemas/unused/malformed partials and missing fields never select an economic action',()=>{
  const changes=[m=>m.schema='other',m=>m.cluster='mainnet-beta',m=>m.action='claim',m=>m.programId=FIXED.pool,m=>m.feePayer='1',m=>m.protocol.reserve=FIXED.clock,m=>m.protocol.extra=1,m=>m.transport.referenceSlot='01',m=>m.finalization.legIndex='1',m=>m.amountLamports='1',m=>m.minimumNative='0',m=>m.eligibility=true,m=>m.timestamp='0'];
  for(const change of changes){const m=fixture();change(m);assert.throws(()=>p.planManifest(m));m.transport.recentBlockhash=null;assert.throws(()=>p.planManifest(m));}
  const unused=fixture(ACTIONS[1]);unused.initiation.validatorIndex='0';assert.throws(()=>p.planManifest(unused));
  for(const action of ACTIONS){const m=fixture(action),block=action===ACTIONS[0]?'initiation':'finalization';for(const field of ['schema','cluster','action','programId','feePayer','roundSequence','protocol','transport','resources',block]){const bad=copy(m);bad[field]=null;assert.equal(p.planManifest(bad).plan,null);}
    for(const b of ['protocol','transport','resources',block])for(const f of Object.keys(m[b])){const bad=copy(m);bad[b][f]=null;assert.equal(p.planManifest(bad).plan,null);}}
  for(const text of ['{"action":null,"action":null}','{"action":null,"act\\u0069on":null}','[]','{"unknown":"do-not-echo"}'])assert.throws(()=>p.planManifest(n.parseManifest(text)));
});
test('CLI retains four complete shared/distinct public manifests/reports and rejects unsupported inputs',context=>{
  const script=path.join(__dirname,'withdrawal-plan.cjs'),limits=process.execArgv.filter(v=>v.startsWith('--max-old-space-size'));
  const run=args=>spawnSync(process.execPath,[...limits,script,...args],{encoding:'utf8',maxBuffer:2*1024*1024,timeout:30000});
  const template=run(['--template']);assert.equal(template.status,0);assert.equal(template.stderr,'');assert.deepEqual(JSON.parse(template.stdout),p.TEMPLATE);
  const incomplete=run(['--manifest',path.join(__dirname,'withdrawal-plan.example.json')]);assert.equal(incomplete.status,0);assert.equal(JSON.parse(incomplete.stdout).plan,null);assert.equal(JSON.parse(incomplete.stdout).status,'INCOMPLETE');
  const directory=fs.mkdtempSync(path.join(os.tmpdir(),'piv1-withdrawal-public-test-'));
  ACTIONS.forEach((action,a)=>{for(const shared of [true,false]){const name=`${action}-${shared?'shared':'distinct'}`,m=fixture(action,{shared,sequence:shared?'9007199254740993':MAX.toString(),index:shared?'0':MAX.toString()}),filename=path.join(directory,`${name}.manifest.json`);
    fs.writeFileSync(filename,JSON.stringify(m,null,2)+'\n',{flag:'wx'});const result=run(['--manifest',filename]);fs.writeFileSync(path.join(directory,`${name}.report.json`),result.stdout,{flag:'wx'});assert.equal(result.status,0);assert.equal(result.stderr,'');
    const out=JSON.parse(result.stdout),packet=out.plan.packets[0],decoded=rawPacket(Buffer.from(packet.wireBase64,'base64'));assert.equal(out.status,'OFFLINE_PLAN_ONLY');assert.deepEqual(out.plan,p.planManifest(m).plan);assert.equal(packet.wireBytes,SIZES[a][shared?0:1]);assert.deepEqual(decoded.instructions[3].rows,literal(m).rows);assert.deepEqual(decoded.instructions[3].data,literal(m).bytes);assert.equal(out.readiness.publicReady,false);assert.equal(out.readiness.cooldownReady,null);
  }});
  const invalid=path.join(directory,'invalid.public.json');fs.writeFileSync(invalid,'{"unknown":"do-not-echo"}',{flag:'wx'});
  for(const args of [[],['--send'],['--rpc','https://invalid.example'],['--template','--rpc'],['--manifest'],['--manifest',invalid],['--keypair','do-not-echo']]){const bad=run(args);assert.notEqual(bad.status,0);assert.equal(bad.stdout,'');assert.equal(JSON.parse(bad.stderr).status,'INVALID_OR_UNSUPPORTED_WITHDRAWAL_PLAN');assert(!bad.stderr.includes('do-not-echo'));}
  context.diagnostic(`Four complete public withdrawal reports retained: ${directory}`);
});
