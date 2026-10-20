'use strict';
const test=require('node:test'),assert=require('node:assert/strict'),fs=require('node:fs'),path=require('node:path'),os=require('node:os'),{spawnSync}=require('node:child_process');
const n=require('./initializer-existing-alt-plan.cjs'),base=require('./initializer-plan.cjs');
// Only public fixture exports; importing this module does not register/run its suite.
const prior=require('./initializer-plan.test.cjs'),t=require('../../validation/genesis-transport/transport.cjs');
const {VersionedTransaction,TransactionMessage,AddressLookupTableAccount}=require(t.SDK);
const copy=v=>JSON.parse(JSON.stringify(v)),POINTS=prior.publicPoints(),U64=(1n<<64n)-1n;
function altBytes(m,{last=BigInt(m.transport.altRecentSlot)+1n,start=0}={}){
  const b=Buffer.alloc(568);b.writeUInt32LE(1);b.writeBigUInt64LE(U64,4);b.writeBigUInt64LE(last,12);b[20]=start;b[21]=1;new n.PublicKey(m.transport.altAuthority).toBuffer().copy(b,22);
  prior.literal(m).targets.forEach((k,i)=>k.toBuffer().copy(b,56+32*i));return b;
}
function fixture({shared=true,actors='shared',slot='9007199254740995',index='73'}={}){
  const m=prior.fixture({shared,fee:actors==='shared'?'creator':'separate',index});if(actors==='shared')m.governance.executor=m.governance.creator;
  m.transport.altRecentSlot=slot;m.transport.referenceSlot=(BigInt(slot)+2n).toString();
  const recent=Buffer.alloc(8);recent.writeBigUInt64LE(BigInt(slot));const address=n.PublicKey.findProgramAddressSync([new n.PublicKey(m.transport.altAuthority).toBuffer(),recent],new n.PublicKey(base.IDS.altProgram))[0].toBase58();
  return {schema:n.SCHEMA,initializer:m,existingAlt:{address,owner:base.IDS.altProgram,executable:false,lamports:'3535680',dataBase64:altBytes(m).toString('base64'),contextSlot:m.transport.referenceSlot}};
}
function mutateBytes(w,fn){const b=Buffer.from(w.existingAlt.dataBase64,'base64');fn(b);w.existingAlt.dataBase64=b.toString('base64');}
function compactCursor(hex){const b=Buffer.from(hex,'hex');let at=0;const byte=()=>b[at++],signers=byte(),writableSigners=byte(),writableNonSigners=byte(),count=byte();const keys=Array.from({length:count},()=>{const k=new n.PublicKey(b.subarray(at,at+32));at+=32;return k;});assert.equal(byte(),1);const program=keys[byte()],length=byte(),indices=Array.from({length},byte),size=b.readUInt16LE(at);at+=2;const data=b.subarray(at,at+size);at+=size;assert.equal(byte(),0);assert.equal(at,b.length);return {signers,writableSigners,writableNonSigners,keys,program,indices,data};}
function table(w){const raw=Buffer.from(w.existingAlt.dataBase64,'base64');return new AddressLookupTableAccount({key:new n.PublicKey(w.existingAlt.address),state:{deactivationSlot:U64,lastExtendedSlot:0,lastExtendedSlotStartIndex:0,authority:new n.PublicKey(w.initializer.transport.altAuthority),addresses:Array.from({length:16},(_,i)=>new n.PublicKey(raw.subarray(56+32*i,88+32*i)))}});}
function wire(p,w){const b=Buffer.from(p.wireBase64,'base64'),tx=VersionedTransaction.deserialize(b);assert.equal(b.length,p.wireBytes);assert(b.length<=1232);assert.equal(tx.signatures.length,p.signatureCount);assert(tx.signatures.every(s=>s.every(x=>x===0)));assert.equal(p.zeroSignatureBytes,tx.signatures.length*64);assert.deepEqual(tx.message.staticAccountKeys.slice(0,tx.signatures.length).map(k=>k.toBase58()),p.requiredSigners);return {tx,instructions:TransactionMessage.decompile(tx.message,tx.message.version===0?{addressLookupTableAccounts:[table(w)]}:undefined).instructions};}

test('pinned sources, nullable public template and known malformed partial declarations',()=>{
  n.verifySources();const example=JSON.parse(fs.readFileSync(path.join(__dirname,'initializer-existing-alt-plan.example.json'),'utf8'));assert.deepEqual(example,n.TEMPLATE);
  for(const v of [example,{}, {schema:n.SCHEMA,initializer:null,existingAlt:null}]){const p=n.planManifest(v);assert.equal(p.status,'INCOMPLETE');assert.equal(p.plan,null);assert(p.missing.length>0);assert.equal(p.readiness.publicReady,false);}
  const mutations=[w=>w.existingAlt.owner=base.IDS.system,w=>w.existingAlt.address=base.IDS.system,w=>w.existingAlt.executable=true,w=>w.existingAlt.lamports='0',w=>w.existingAlt.dataBase64='not canonical'];
  for(const change of mutations){const w=copy(n.TEMPLATE);change(w);assert.throws(()=>n.planManifest(w));}
  const w=fixture();w.initializer.initialization.anchorUnixTimestamp=null;assert.equal(n.planManifest(w).plan,null);mutateBytes(w,b=>b[54]=1);assert.throws(()=>n.planManifest(w));
});
test('complete34/35-role native and compact literals preserve remaining signer unions and v0 wires',()=>{
  const seen=new Set();
  for(const shared of [true,false])for(const actors of ['shared','distinct']){
    const w=fixture({shared,actors}),m=w.initializer,p=n.planManifest(w),literal=prior.literal(m),count=shared?34:35;
    assert.equal(p.status,'OFFLINE_PLAN_ONLY');assert.equal(p.newAltSetupPackets,0);assert.equal(p.newAltSetupRentOutflow,'0');assert.equal(p.readiness.actualFees,null);assert.equal(p.readiness.fullProjectBudget,null);
    assert.equal(p.plan.packets.length,9);assert(!p.plan.packets.some(x=>['alt_create','alt_extend'].includes(x.name)));assert(p.plan.packets.every(x=>x.requestedInstructions.every(i=>i.programId!==base.IDS.altProgram)));
    assert.equal(p.plan.native.hex,literal.data.toString('hex'));assert.equal(p.plan.native.bytes,315);assert.equal(p.plan.native.roles.length,count);assert.deepEqual(p.plan.native.roles.map(r=>r.address),literal.keys.map(k=>k.toBase58()));
    p.plan.native.roles.forEach((r,i)=>{assert.equal(r.signer,i===5||i===count-5);assert.equal(r.writable,(i>=7&&i<23)||i===count-5);});
    const c=compactCursor(p.plan.compact.hex);assert.equal(p.plan.compact.bytes,shared?1447:1480);assert.deepEqual([c.signers,c.writableSigners,c.writableNonSigners],[2,1,16]);assert.equal(c.program.toBase58(),m.programId);assert.deepEqual(c.indices.map(i=>c.keys[i].toBase58()),literal.keys.map(k=>k.toBase58()));assert.deepEqual(c.data,literal.data);
    for(const packet of p.plan.packets){const decoded=wire(packet,w);seen.add(packet.signatureCount);assert(!packet.requiredSigners.includes(m.transport.altAuthority));if(packet.name==='execute'){
      assert.equal(decoded.tx.message.version,0);assert.equal(decoded.tx.message.addressTableLookups.length,1);const lookup=decoded.tx.message.addressTableLookups[0];assert.equal(lookup.accountKey.toBase58(),w.existingAlt.address);assert.deepEqual([...lookup.writableIndexes],Array.from({length:16},(_,i)=>i));assert.deepEqual([...lookup.readonlyIndexes],[]);
      assert.deepEqual(decoded.instructions.slice(0,3).map(i=>i.data.toString('hex')),['02c05c1500','0100000400','0400008000']);assert.equal(decoded.instructions[3].data.toString('hex'),'c208a15799a419ab');assert(decoded.instructions[3].keys[1].isWritable);assert(!decoded.instructions[3].keys.find(k=>k.pubkey.toBase58()===p.plan.vault).isSigner);
    }}
    assert.equal(p.plan.alt.compilationState,'SUPPLIED_SNAPSHOT_ADDRESSES_ONLY_NOT_AUTHENTICATED_OR_FRESH');assert.equal(p.plan.alt.authenticated,false);assert.equal(p.plan.alt.freshnessVerified,false);assert.equal(p.plan.alt.warmupVerified,false);assert.equal(p.plan.alt.declaredStrictLaterSlotSatisfied,true);assert.equal(p.plan.alt.creationSlotHashesMembershipRequired,false);
  }
  assert.deepEqual([...seen].sort(),[1,2,3]);
});
test('raw ALT envelope rejects bounded-base64, layout/tag/option/padding and size variations',()=>{
  const w=fixture();
  for(const value of [0,{},'',w.existingAlt.dataBase64+' ',w.existingAlt.dataBase64.slice(0,-1),w.existingAlt.dataBase64.replace(/==$/,''),'A'.repeat(760),Buffer.alloc(567).toString('base64'),Buffer.alloc(569).toString('base64')]){const v=copy(w);v.existingAlt.dataBase64=value;assert.throws(()=>n.planManifest(v));}
  const changes=[b=>b.writeUInt32LE(0,0),b=>b.writeUInt32LE(2,0),b=>b[21]=2,b=>{b[21]=0;b.fill(0,22,56);},b=>b[54]=1,b=>b[55]=255,b=>b[20]=17,b=>b.fill(0,22,54),b=>b.copy(b,88,56,88)];
  for(const change of changes){const v=copy(w);mutateBytes(v,change);assert.throws(()=>n.planManifest(v));}
  // Canonical padding bits: same decoded bytes with a noncanonical final sextet must fail.
  const bad=copy(w),alphabet='ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/',at=bad.existingAlt.dataBase64.length-3,old=alphabet.indexOf(bad.existingAlt.dataBase64[at]);bad.existingAlt.dataBase64=bad.existingAlt.dataBase64.slice(0,at)+alphabet[old^1]+'==';assert.throws(()=>n.planManifest(bad));
});
test('exact owner/address/authority/order/active/warmup and partial known aliases are checked',()=>{
  const w=fixture();const mutations=[v=>v.existingAlt.owner=base.IDS.token,v=>v.existingAlt.executable=true,v=>v.existingAlt.address=POINTS[17],v=>mutateBytes(v,b=>new n.PublicKey(POINTS[7]).toBuffer().copy(b,22)),v=>mutateBytes(v,b=>{const a=Buffer.from(b.subarray(56,88));b.copy(b,56,88,120);a.copy(b,88);}),v=>mutateBytes(v,b=>b.writeBigUInt64LE(0n,4)),v=>v.existingAlt.contextSlot=(BigInt(v.existingAlt.contextSlot)-1n).toString(),v=>{v.existingAlt.contextSlot=v.initializer.transport.referenceSlot=(BigInt(w.initializer.transport.altRecentSlot)+1n).toString();}];
  for(const change of mutations){const v=copy(w);change(v);assert.throws(()=>n.planManifest(v));}
  for(const start of [0,1,15,16]){const v=copy(w);mutateBytes(v,b=>b[20]=start);assert.equal(n.planManifest(v).status,'OFFLINE_PLAN_ONLY');}
  for(const key of ['programId','feePayer','rentPayer']){const v=copy(n.TEMPLATE);v.initializer[key]=POINTS[18];v.existingAlt.address=POINTS[18];assert.throws(()=>n.planManifest(v));}
  for(const value of Object.values(base.IDS)){const v=copy(n.TEMPLATE);v.existingAlt.address=value;assert.throws(()=>n.planManifest(v));}
  const partial=copy(w),rewardTarget=prior.literal(partial.initializer).targets[3].toBase58();partial.initializer.initialization.anchorUnixTimestamp=null;partial.initializer.transport.altRecentSlot=null;partial.existingAlt.address=rewardTarget;assert.throws(()=>n.planManifest(partial),e=>e.code==='alt_known_role_alias');
  const seedOnly=copy(n.TEMPLATE);seedOnly.initializer.governance.createKey=POINTS[18];seedOnly.existingAlt.address=POINTS[18];assert.equal(n.planManifest(seedOnly).status,'INCOMPLETE');
  const mismatch=copy(n.TEMPLATE);mismatch.initializer.programId=w.initializer.programId;mismatch.existingAlt.dataBase64=w.existingAlt.dataBase64;mutateBytes(mismatch,b=>new n.PublicKey(POINTS[19]).toBuffer().copy(b,56));assert.throws(()=>n.planManifest(mismatch));
});
test('finite full-u64/string domains preserve raw slots and exact governance index precision',()=>{
  for(const recent of [0n,1n,255n,9007199254740993n,U64-2n]){const w=fixture({slot:recent.toString()}),p=n.planManifest(w);assert.equal(p.plan.alt.decodedState.lastExtendedSlot,(recent+1n).toString());assert.equal(p.plan.alt.suppliedSnapshot.contextSlot,(recent+2n).toString());assert.equal(p.plan.referenceContext.referenceSlot,(recent+2n).toString());assert.equal(p.plan.alt.derivationRecentSlot,recent.toString());}
  for(const index of ['1','9007199254740993',U64.toString()]){const w=fixture({index}),p=n.planManifest(w),proposal=p.plan.packets.find(x=>x.name==='proposal_create');assert.equal(Buffer.from(proposal.requestedInstructions[0].dataHex,'hex').readBigUInt64LE(8),BigInt(index));}
  for(const value of [0,1,9007199254740992,'-1','+1','01','1e3','1.0','18446744073709551616'])for(const field of ['lamports','contextSlot']){const w=copy(n.TEMPLATE);w.existingAlt[field]=value;assert.throws(()=>n.planManifest(w));}
  for(const value of ['1',U64.toString()]){const w=fixture();w.existingAlt.lamports=value;assert.equal(n.planManifest(w).plan.alt.suppliedSnapshot.lamports,value);}
});
test('strict wrapper/duplicate/unknown fields and retained D047 raw shape bind without choosing a real anchor',()=>{
  for(const source of ['{"schema":null,"schema":null}','{"existingAlt":{"owner":null,"owne\\u0072":null}}','[]','{"extra":0}','{"initializer":{"transport":{"unexpected":true}}}'])assert.throws(()=>n.planManifest(n.parseManifest(source)));
  assert.throws(()=>n.parseManifest('{'));assert.throws(()=>n.parseManifest(' '.repeat(65537)));
  for(const change of [w=>w.schema='other',w=>w.existingAlt.extra=true,w=>w.initializer.protocol.extra=true,w=>w.existingAlt=[],w=>w.initializer=0]){const w=copy(n.TEMPLATE);change(w);assert.throws(()=>n.planManifest(w));}
  const setup=JSON.parse(fs.readFileSync(path.resolve(__dirname,'../../docs/PIV1_TESTNET_FUNDING_ALT_SETUP.json'),'utf8')),pkg=JSON.parse(fs.readFileSync(path.resolve(__dirname,'../../docs/PIV1_TESTNET_PACKAGE_DRAFT.json'),'utf8')),a=setup.actualAlt,w=fixture(),m=w.initializer;
  m.programId=pkg.founderPublicInputs.pivProgramId;m.feePayer=pkg.founderPublicInputs.actors.feePayer;m.rentPayer=pkg.founderPublicInputs.actors.rentPayer;Object.assign(m.governance,{multisigAddress:pkg.founderPublicInputs.squads.multisigAddress,createKey:pkg.founderPublicInputs.squads.createKey,vaultIndex:0,guardians:pkg.founderPublicInputs.guardianPublicKeys,guardianSlotPermutation:pkg.founderPublicInputs.initialization.guardianSlotPermutation,creator:pkg.founderPublicInputs.actors.creator,executor:pkg.founderPublicInputs.actors.executor,approvers:pkg.founderPublicInputs.actors.approvers});Object.assign(m.recipients,{htfpVaultIndex:1,teamOwnerVaultIndex:2});Object.assign(m.transport,{altAuthority:a.authority,altRecentSlot:a.derivationRecentSlot,referenceSlot:a.finalObservedSlot});
  w.existingAlt={address:a.address,owner:a.owner,executable:false,lamports:a.lamports,dataBase64:a.dataBase64,contextSlot:a.finalObservedSlot};const p=n.planManifest(w);assert.deepEqual(p.plan.alt.addresses,a.orderedAddresses);assert.equal(p.plan.alt.decodedState.lastExtendedSlot,'454845694');assert.equal(p.plan.alt.suppliedSnapshot.dataSha256,a.dataSha256);assert.equal(p.plan.alt.creationSlotHashesMembershipRequired,false);assert.equal(p.readiness.chainSnapshotVerified,false);assert.equal(pkg.founderPublicInputs.initialization.anchorUnixTimestamp,null);assert.equal(m.initialization.anchorUnixTimestamp,'-123456789');
});
test('standalone CLI retains two explicitly synthetic complete reports and exact template/null/error semantics',context=>{
  const script=path.join(__dirname,'initializer-existing-alt-plan.cjs'),limits=process.execArgv.filter(x=>x.startsWith('--max-old-space-size'));
  const run=args=>spawnSync(process.execPath,[...limits,script,...args],{encoding:'utf8',timeout:15000,maxBuffer:2*1024*1024});
  const template=run(['--template']);assert.equal(template.status,0);assert.equal(template.stderr,'');assert.deepEqual(JSON.parse(template.stdout),n.TEMPLATE);
  const incomplete=run(['--manifest',path.join(__dirname,'initializer-existing-alt-plan.example.json')]);assert.equal(incomplete.status,0);assert.equal(JSON.parse(incomplete.stdout).status,'INCOMPLETE');assert.equal(JSON.parse(incomplete.stdout).plan,null);
  const directory=fs.mkdtempSync(path.join(os.tmpdir(),'piv1-existing-alt-cli-'));
  for(const [name,options]of [['synthetic-shared',{shared:true,actors:'shared'}],['synthetic-distinct',{shared:false,actors:'distinct'}]]){
    const w=fixture(options),manifest=path.join(directory,name+'-manifest.json'),report=path.join(directory,name+'-report.json');fs.writeFileSync(manifest,JSON.stringify(w,null,2)+'\n',{flag:'wx'});const result=run(['--manifest',manifest]);assert.equal(result.status,0);assert.equal(result.stderr,'');const parsed=JSON.parse(result.stdout);assert.deepEqual(parsed.plan,n.planManifest(w).plan);assert.equal(parsed.status,'OFFLINE_PLAN_ONLY');assert.equal(parsed.newAltSetupPackets,0);fs.writeFileSync(report,result.stdout,{flag:'wx'});context.diagnostic(`SYNTHETIC_ONLY_NOT_LIVE: ${manifest} -> ${report}`);
  }
  const invalid=path.join(directory,'invalid-public.json');fs.writeFileSync(invalid,'{"existingAlt":{"dataBase64":"do-not-echo"}}',{flag:'wx'});const bad=run(['--manifest',invalid]);assert.equal(bad.status,1);assert.equal(bad.stdout,'');assert.equal(JSON.parse(bad.stderr).status,'INVALID_OR_UNSUPPORTED_PLAN');assert(!bad.stderr.includes('do-not-echo'));
  for(const args of [[],['--send'],['--template','--rpc']]){const x=run(args);assert.equal(x.status,1);assert.equal(x.stdout,'');assert.equal(JSON.parse(x.stderr).status,'INVALID_OR_UNSUPPORTED_PLAN');}
});
