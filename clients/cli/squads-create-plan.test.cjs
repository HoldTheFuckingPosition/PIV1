'use strict';
const assert=require('node:assert/strict'),fs=require('node:fs'),path=require('node:path'),os=require('node:os');
const {createHash}=require('node:crypto'),{spawnSync}=require('node:child_process'),test=require('node:test');
const p=require('./squads-create-plan.cjs'),n=require('./initializer-plan.cjs'),{publicPoints}=require('./initializer-plan.test.cjs');
const t=require('../../validation/genesis-transport/transport.cjs'),{PublicKey,VersionedTransaction}=require(t.SDK);
const POINTS=publicPoints(),copy=v=>JSON.parse(JSON.stringify(v)),MAX=(1n<<64n)-1n,sha=b=>createHash('sha256').update(b).digest('hex');
const SQUADS='SQDS4ep65T869zMMBKyuUq6aD6EgTu8psMjkvj52pCf',SYSTEM='11111111111111111111111111111111';
const CONFIG='BSTq9w3kZwNwpBXJEvTZz2G9ZTNyKBvoSeXMvwb4cNZr',GENESIS='4uhcVJyU9pJkvQyS88uRDiswHXSCkY3zQawwpjk2NsNY';
const derive=seeds=>{const [k,bump]=PublicKey.findProgramAddressSync(seeds,new PublicKey(SQUADS));return {address:k.toBase58(),bump};};
function fixture({unknownFee=false,actual=false}={}){
  const m=copy(p.TEMPLATE);m.guardians=[5,2,0,4,1,3].map(i=>POINTS[i]);m.createKey=POINTS[0];m.payer=POINTS[7];
  if(actual){const inventory=JSON.parse(fs.readFileSync(path.join(__dirname,'../../docs/PIV1_TESTNET_IDENTITY_SETUP.json'),'utf8'));
    m.guardians=Array.from({length:6},(_,i)=>inventory.keys.find(k=>k.role===`guardian-${i}`).publicKey);m.createKey=m.guardians[0];m.payer=inventory.keys.find(k=>k.role==='payer').publicKey;}
  m.profile={threshold:4,memberPermissionsMask:7,configAuthority:'none',timeLockSeconds:0,rentCollector:'none',memo:'none'};
  Object.assign(m.observations,{programConfig:CONFIG,treasury:POINTS[9],creationFeeLamports:'0',rent330Lamports:'2326640',networkFeeLamports:unknownFee?null:'10000',slot:'9007199254740993',observedAtUtc:'2026-10-19T00:00:00.000Z'});
  Object.assign(m.transport,{recentBlockhash:POINTS[16],lastValidBlockHeight:'9007199254740995',referenceSlot:'9007199254740997'});return m;
}
const rows=m=>[
  {address:CONFIG,signer:false,writable:false},{address:m.observations.treasury,signer:false,writable:true},
  {address:derive([Buffer.from('multisig'),Buffer.from('multisig'),new PublicKey(m.createKey).toBuffer()]).address,signer:false,writable:true},
  {address:m.createKey,signer:true,writable:false},{address:m.payer,signer:true,writable:true},{address:SYSTEM,signer:false,writable:false}];
function decodePayload(bytes){let at=0;const take=n=>{assert(at+n<=bytes.length);const b=bytes.subarray(at,at+n);at+=n;return b;};
  assert.equal(take(8).toString('hex'),'32ddc75d28f58be9');assert.equal(take(1)[0],0);const threshold=take(2).readUInt16LE(),count=take(4).readUInt32LE();assert.equal(count,6);
  const members=Array.from({length:count},()=>({address:new PublicKey(take(32)).toBase58(),permissionsMask:take(1)[0]})),timeLockSeconds=take(4).readUInt32LE();
  assert.equal(take(1)[0],0);assert.equal(take(1)[0],0);assert.equal(at,219);assert.equal(at,bytes.length);return {threshold,members,timeLockSeconds};}
function rawPacket(raw){let at=0;const byte=()=>{assert(at<raw.length);return raw[at++];},take=n=>{assert(at+n<=raw.length);const b=raw.subarray(at,at+n);at+=n;return b;};
  const sv=()=>{let value=0,shift=0,b;do{b=byte();value+=(b&127)*2**shift;shift+=7;assert(shift<=21);}while(b&128);return value;};
  const signatures=Array.from({length:sv()},()=>take(64)),message=raw.subarray(at),header=[byte(),byte(),byte()];assert(header[0]<128);
  const keys=Array.from({length:sv()},()=>new PublicKey(take(32)).toBase58()),blockhash=new PublicKey(take(32)).toBase58();
  const instructions=Array.from({length:sv()},()=>{const program=keys[byte()],indices=Array.from({length:sv()},byte),data=take(sv());return {program,indices,data,
    rows:indices.map(i=>({address:keys[i],signer:i<header[0],writable:i<header[0]?i<header[0]-header[1]:i<keys.length-header[2]}))};});assert.equal(at,raw.length);return {signatures,message,header,keys,blockhash,instructions};}
function checkWire(m,out){const packet=out.plan.packets[0],raw=Buffer.from(packet.wireBase64,'base64'),d=rawPacket(raw),expected=rows(m);
  assert.equal(raw.length,619);assert(raw.length<=1232);assert.equal(packet.wireBytes,raw.length);assert.equal(packet.sha256,sha(raw));assert.equal(packet.version,'legacy');assert.equal(packet.signatureCount,2);assert.equal(packet.zeroSignatureBytes,128);
  assert.equal(d.signatures.length,2);assert(d.signatures.every(s=>s.length===64&&s.every(v=>v===0)));assert.deepEqual(packet.requiredSigners,[m.payer,m.createKey]);assert.deepEqual(d.header,[2,1,3]);assert.equal(d.keys.length,7);assert.equal(d.keys[0],m.payer);assert.equal(d.keys[1],m.createKey);
  assert.equal(d.blockhash,m.transport.recentBlockhash);assert.equal(d.instructions.length,1);const ix=d.instructions[0];assert.equal(ix.program,SQUADS);assert.deepEqual(ix.rows,expected);assert.equal(new Set(d.keys).size,7);
  assert.deepEqual(packet.requestedInstructions,[{programId:SQUADS,accounts:expected,dataHex:ix.data.toString('hex')}]);assert.deepEqual(packet.wireInstructions,packet.requestedInstructions);
  assert.equal(ix.data.toString('hex'),out.plan.native.hex);assert.equal(ix.data.length,219);assert.equal(out.plan.native.sha256,sha(ix.data));decodePayload(ix.data);
  assert.equal(d.message.length,490);assert.equal(out.plan.messageBase64,d.message.toString('base64'));assert.equal(out.plan.messageSha256,sha(d.message));assert.deepEqual(Buffer.from(VersionedTransaction.deserialize(raw).serialize()),raw);return d;}
function* permutations(values,prefix=[]){if(!values.length){yield prefix;return;}for(let i=0;i<values.length;i++)yield* permutations([...values.slice(0,i),...values.slice(i+1)],[...prefix,values[i]]);}
// Digit arithmetic oracle independent of planner BigInt additions.
function decimalSum(values){let carry=0,result='',width=Math.max(...values.map(s=>s.length));for(let i=0;i<width||carry;i++){const sum=carry+values.reduce((s,v)=>s+(i<v.length?Number(v[v.length-1-i]):0),0);result=String(sum%10)+result;carry=Math.floor(sum/10);}return result.replace(/^0+(?=\d)/,'');}
test('pinned host inputs, retained interface reference and nullable example preserve unverified status',()=>{
  p.verifySources();assert.deepEqual(JSON.parse(fs.readFileSync(path.join(__dirname,'squads-create-plan.example.json'))),p.TEMPLATE);assert.equal(p.GENESIS,GENESIS);assert.equal(sha(Buffer.from('global:multisig_create_v2')).slice(0,16),'32ddc75d28f58be9');
  assert.equal(p.RETAINED_SOURCE.commit,'64af7330413d5c85cbbccfd8c27a05d45b6e666f');assert.equal(p.RETAINED_SOURCE.pins['programs/squads_multisig_program/src/instructions/multisig_create.rs'],'fc7d0f9eb2d5ec895717e5092f06c228533c174f776ad02d2ccb1861086c7197');
  for(const m of [{},copy(p.TEMPLATE)]){const out=p.planManifest(m);assert.equal(out.status,'INCOMPLETE');assert.equal(out.plan,null);assert(out.missing.includes('createKey'));}
  const out=p.planManifest(fixture());for(const [f,v]of Object.entries(out.readiness))assert.equal(v,f==='m4Reservation'?'OPEN_UNWAIVED':false);assert.equal(out.plan.profile.status,'PROPOSED_NOT_APPROVED');assert.equal(out.plan.derived.existenceVerified,false);assert.equal(out.plan.declaredCosts.actualWalletBudgetLamports,null);
});
test('independent literal Borsh cursor and canonical PDA/role oracle match the exact creation boundary',()=>{
  const m=fixture(),out=p.planManifest(m),d=decodePayload(Buffer.from(out.plan.native.hex,'hex'));
  const sorted=[...m.guardians].sort((a,b)=>Buffer.compare(new PublicKey(a).toBuffer(),new PublicKey(b).toBuffer()));assert.deepEqual(d,{threshold:4,members:sorted.map(address=>({address,permissionsMask:7})),timeLockSeconds:0});assert.deepEqual(out.plan.members,d.members);assert.equal(out.plan.native.bytes,219);
  assert.deepEqual(out.plan.roles.map(({role,...r})=>r),rows(m));assert.deepEqual(out.plan.roles.map(r=>r.role),['program_config','treasury','multisig','create_key','creator','system_program']);assert.equal(out.plan.roles.filter(r=>r.writable).length,3);
  assert.deepEqual(out.plan.derived.programConfig,{address:CONFIG,bump:255});assert.deepEqual(out.plan.derived.multisig,derive([Buffer.from('multisig'),Buffer.from('multisig'),new PublicKey(m.createKey).toBuffer()]));assert.equal(out.plan.derived.multisigAllocationBytes,330);
  const actual=fixture({actual:true}),actualOut=p.planManifest(actual);assert.equal(actual.createKey,'BUuuNvicRKF4ZFEUH4iS8xGzhAKQca3ou4U1s4NaBYoT');assert.equal(actual.payer,'H2AaoG8HZKfNyZzMhDGMB8n9edviyHtJGKpPL3uYpxt4');assert.deepEqual(actualOut.plan.derived.multisig,{address:'9mwto7RNokYc8n8MPtzHpubwwnKcborY2r9RnfZAScVS',bump:255});assert.equal(actualOut.plan.inputOrderMeaning,'PUBLIC_INVENTORY_ONLY_NOT_PIV_SLOT_ORDER');
});
test('independent full legacy cursor counts both zero signatures and exact requested/global privileges',()=>{
  for(const actual of [false,true])for(const unknownFee of [false,true]){const m=fixture({actual,unknownFee});checkWire(m,p.planManifest(m));}
  const contextAlias=fixture();contextAlias.transport.recentBlockhash=contextAlias.payer;checkWire(contextAlias,p.planManifest(contextAlias));
});
test('all720 public-member permutations preserve canonical bytes/PDA with fixed explicit createKey',()=>{
  const m=fixture(),base=p.planManifest(m),packet=base.plan.packets[0];let count=0;
  for(const order of permutations(m.guardians)){const next={...m,guardians:order},out=p.planManifest(next);assert.deepEqual(out.plan.members,base.plan.members);assert.deepEqual(out.plan.derived,base.plan.derived);assert.equal(out.plan.native.hex,base.plan.native.hex);assert.equal(out.plan.packets[0].wireBase64,packet.wireBase64);assert.equal(out.plan.createKey,m.createKey);assert.deepEqual(out.plan.inputGuardians,order);count++;}assert.equal(count,720);
  const changed=fixture();changed.createKey=changed.guardians[0];assert.notEqual(p.planManifest(changed).plan.derived.multisig.address,base.plan.derived.multisig.address);
});
test('bounded generated full-u64 observation arithmetic uses an independent decimal oracle and never alters wire',()=>{
  const m=fixture(),wire=p.planManifest(m).plan.packets[0].wireBase64;let state=0x123456789abcdef0n;const next=()=>{state^=state<<13n;state^=state>>7n;state^=state<<17n;state&=MAX;return state;};
  const triples=[['0','1','1'],['9007199254740993','9007199254740995','9007199254740997'],[(MAX-2n).toString(),'1','1']];
  for(let i=0;i<64;i++)triples.push([(next()/4n).toString(),(next()/4n+1n).toString(),(next()/4n+1n).toString()]);
  for(const [creation,rent,network]of triples){const x=copy(m);Object.assign(x.observations,{creationFeeLamports:creation,rent330Lamports:rent,networkFeeLamports:network});const out=p.planManifest(x),cost=out.plan.declaredCosts;
    assert.equal(cost.creationFeeLamports,creation);assert.equal(cost.rent330Lamports,rent);assert.equal(cost.networkFeeLamports,network);assert.equal(cost.conditionalCreatorRentAndCreationFeeLamports,decimalSum([creation,rent]));assert.equal(cost.conditionalTotalIncludingDeclaredNetworkFeeLamports,decimalSum([creation,rent,network]));assert.deepEqual(cost.missingQuoteInputs,[]);assert.equal(out.plan.packets[0].wireBase64,wire);}
  const unknown=fixture({unknownFee:true}),unquoted=p.planManifest(unknown);assert.equal(unquoted.status,'OFFLINE_PLAN_ONLY');assert.equal(unquoted.plan.packets[0].wireBase64,wire);assert.equal(unquoted.plan.declaredCosts.networkFeeLamports,null);assert.equal(unquoted.plan.declaredCosts.conditionalTotalIncludingDeclaredNetworkFeeLamports,null);assert.deepEqual(unquoted.plan.declaredCosts.missingQuoteInputs,['networkFeeLamports']);
  for(const f of ['slot','creationFeeLamports','rent330Lamports','networkFeeLamports'])for(const value of [0,1,-1,'-1','+1','01','1.0','1e3',(MAX+1n).toString()]){const x=copy(p.TEMPLATE);x.observations[f]=value;assert.throws(()=>p.planManifest(x));}
  for(const f of ['rent330Lamports','networkFeeLamports']){const x=fixture();x.observations[f]='0';assert.throws(()=>p.planManifest(x));}
  for(const field of ['slot','lastValidBlockHeight','referenceSlot'])for(const value of ['0','1','4294967296','9007199254740993',MAX.toString()]){const x=fixture();(field==='slot'?x.observations:x.transport)[field]=value;assert.equal(p.planManifest(x).plan.packets[0].wireBase64,wire);}
  for(const networkFeeLamports of ['1',null]){const x=fixture();Object.assign(x.observations,{creationFeeLamports:MAX.toString(),rent330Lamports:'1',networkFeeLamports});assert.throws(()=>p.planManifest(x));}
});
test('strict schemas, malformed known partials, aliases, profile and missing network identity fail closed',()=>{
  const changes=[m=>m.schema='other',m=>m.cluster='devnet',m=>m.expectedGenesis=POINTS[12],m=>m.guardians=[POINTS[0]],m=>m.guardians='bad',m=>m.createKey=POINTS[10],m=>m.payer=m.createKey,m=>m.payer=m.guardians[2],m=>m.guardians[2]=m.guardians[1],m=>m.observations.programConfig=POINTS[11],m=>m.observations.treasury=m.payer,m=>m.observations.treasury=m.guardians[1],m=>m.observations.treasury=CONFIG,m=>m.payer=SQUADS,m=>m.createKey=SYSTEM,m=>m.guardians[4]=SQUADS,m=>m.profile.threshold=3,m=>m.profile.memberPermissionsMask=2,m=>m.profile.timeLockSeconds='0',m=>m.profile.timeLockSeconds=1,m=>m.profile.configAuthority=POINTS[8],m=>m.profile.rentCollector=POINTS[8],m=>m.profile.memo='',m=>m.observations.slot='01',m=>m.observations.observedAtUtc='2026-02-30T00:00:00.000Z',m=>m.transport.referenceSlot=1,m=>m.creator=POINTS[8],m=>m.recipient=POINTS[9],m=>m.rpcUrl='https://invalid.example',m=>m.resources={},m=>m.observations.extra=0];
  for(const change of changes){const m=fixture();change(m);assert.throws(()=>p.planManifest(m));m.transport.recentBlockhash=null;assert.throws(()=>p.planManifest(m));}
  const offcurve=derive([Buffer.from('public-only-other-member')]).address,m=fixture();m.guardians[4]=offcurve;assert.equal(p.planManifest(m).status,'OFFLINE_PLAN_ONLY');
  for(const field of ['payer','createKey']){const x=copy(p.TEMPLATE);x[field]=offcurve;assert.throws(()=>p.planManifest(x));}
  const partialIndex=copy(p.TEMPLATE);partialIndex.guardians[5]=SQUADS;assert.throws(()=>p.planManifest(partialIndex),e=>e.field==='guardians.5');
  const candidate=p.planManifest(fixture()).plan.derived.multisig.address;for(const field of ['payer','treasury','guardian']){const x=fixture();if(field==='guardian')x.guardians[4]=candidate;else if(field==='treasury')x.observations.treasury=candidate;else x.payer=candidate;x.transport.recentBlockhash=null;assert.throws(()=>p.planManifest(x));}
  const full=fixture();for(const f of Object.keys(full)){const x=copy(full);x[f]=null;assert.equal(p.planManifest(x).plan,null);}
  for(const block of ['profile','observations','transport'])for(const f of Object.keys(full[block])){if(f==='networkFeeLamports')continue;const x=copy(full);x[block][f]=null;assert.equal(p.planManifest(x).plan,null);}
  for(let i=0;i<6;i++){const x=copy(full);x.guardians[i]=null;assert.equal(p.planManifest(x).plan,null);}
  for(const json of ['{"payer":null,"payer":null}','{"payer":null,"pay\\u0065r":null}','[]','{"unknown":"do-not-echo"}'])assert.throws(()=>p.planManifest(n.parseManifest(json)));
});
test('CLI retains complete public known/unknown-fee and actual-inventory synthetic-context reports',context=>{
  const script=path.join(__dirname,'squads-create-plan.cjs'),limits=process.execArgv.filter(v=>v.startsWith('--max-old-space-size'));
  const run=args=>spawnSync(process.execPath,[...limits,script,...args],{encoding:'utf8',maxBuffer:2*1024*1024,timeout:30000});
  const template=run(['--template']);assert.equal(template.status,0);assert.equal(template.stderr,'');assert.deepEqual(JSON.parse(template.stdout),p.TEMPLATE);
  const incomplete=run(['--manifest',path.join(__dirname,'squads-create-plan.example.json')]);assert.equal(incomplete.status,0);assert.equal(JSON.parse(incomplete.stdout).status,'INCOMPLETE');assert.equal(JSON.parse(incomplete.stdout).plan,null);
  const directory=fs.mkdtempSync(path.join(os.tmpdir(),'piv1-squads-create-public-test-'));
  for(const [name,options]of [['synthetic-known-fee',{}],['synthetic-unknown-fee',{unknownFee:true}],['actual-public-inventory-SYNTHETIC-context',{actual:true}]]){const m=fixture(options),filename=path.join(directory,`${name}.manifest.json`);fs.writeFileSync(filename,JSON.stringify(m,null,2)+'\n',{flag:'wx'});const result=run(['--manifest',filename]);fs.writeFileSync(path.join(directory,`${name}.report.json`),result.stdout,{flag:'wx'});assert.equal(result.status,0);assert.equal(result.stderr,'');const out=JSON.parse(result.stdout);assert.equal(out.status,'OFFLINE_PLAN_ONLY');assert.deepEqual(out.plan,p.planManifest(m).plan);checkWire(m,out);assert.equal(out.readiness.squadsCreationAuthorized,false);}
  const invalid=path.join(directory,'invalid.public.json');fs.writeFileSync(invalid,'{"unknown":"do-not-echo"}',{flag:'wx'});
  for(const args of [[],['--send'],['--rpc','https://invalid.example'],['--template','--manifest'],['--manifest'],['--manifest',invalid],['--keypair','do-not-echo'],['--manifest',directory]]){const bad=run(args);assert.notEqual(bad.status,0);assert.equal(bad.stdout,'');assert.equal(JSON.parse(bad.stderr).status,'INVALID_OR_UNSUPPORTED_SQUADS_CREATE_PLAN');assert(!bad.stderr.includes('do-not-echo'));}
  context.diagnostic(`Three complete public Squads-creation reports retained (synthetic contexts/quotes only): ${directory}`);
});
