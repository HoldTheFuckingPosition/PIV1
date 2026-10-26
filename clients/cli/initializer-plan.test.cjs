'use strict';
const assert=require('node:assert/strict');
const fs=require('node:fs');
const path=require('node:path');
const os=require('node:os');
const {spawnSync}=require('node:child_process');
const n=require('./initializer-plan.cjs');
const t=require('../../validation/genesis-transport/transport.cjs');
const {TransactionInstruction,TransactionMessage,VersionedTransaction,AddressLookupTableAccount}=require(t.SDK);
const copy=v=>JSON.parse(JSON.stringify(v));

// Public-point arithmetic using the published Ed25519 base point. These are
// deliberately public fixtures, with no keypair, private seed, RNG or signature.
function publicPoints() {
  const p=(1n<<255n)-19n,mod=n=>(n%p+p)%p;
  function pow(a,n){let r=1n;for(a=mod(a);n;n>>=1n,a=mod(a*a))if(n&1n)r=mod(r*a);return r;}
  const inv=a=>pow(a,p-2n),d=mod(-121665n*inv(121666n));
  const g=[15112221349535400772501151409588531511454012693041857206046113283949847762202n,46316835694926478169428394003475163141307993866256225615783033603165251855960n];
  let q=[0n,1n];const result=[];
  for(let i=0;i<20;i++){
    const [x,y]=q,[a,b]=g,k=mod(d*x*a*y*b);q=[mod((x*b+y*a)*inv(1n+k)),mod((y*b+x*a)*inv(1n-k))];
    let encoded=q[1]|((q[0]&1n)<<255n);const bytes=Buffer.alloc(32);for(let j=0;j<32;j++,encoded>>=8n)bytes[j]=Number(encoded&255n);
    result.push(new n.PublicKey(bytes).toBase58());
  }
  return result;
}
const POINTS=publicPoints();
function fixture({shared=false,fee='rent',vault=23,index='73',permutation=[3,0,5,1,4,2]}={}) {
  const m=copy(n.TEMPLATE),pk=n.PublicKey,squads=new pk(n.IDS.squads),createKey=POINTS[12];
  m.programId=POINTS[18];m.rentPayer=POINTS[6];m.feePayer=fee==='rent'?m.rentPayer:fee==='creator'?POINTS[0]:fee==='executor'?POINTS[5]:POINTS[7];
  Object.assign(m.governance,{createKey,multisigAddress:pk.findProgramAddressSync([Buffer.from('multisig'),Buffer.from('multisig'),new pk(createKey).toBuffer()],squads)[0].toBase58(),
    vaultIndex:vault,bufferIndex:249,transactionIndex:index,currentTransactionIndex:(BigInt(index)-1n).toString(),staleTransactionIndex:'0',
    guardians:[POINTS[4],POINTS[1],POINTS[5],POINTS[0],POINTS[3],POINTS[2]],guardianSlotPermutation:permutation,
    creator:POINTS[0],executor:POINTS[5],approvers:[POINTS[0],POINTS[2],POINTS[3],POINTS[4]],timeLockSeconds:3600});
  Object.assign(m.protocol,{validatorList:POINTS[9],reserve:POINTS[10],managerFee:POINTS[11],referrer:shared?POINTS[11]:POINTS[13]});
  Object.assign(m.recipients,{htfpRecipient:POINTS[14],teamOwnerRecipient:POINTS[15]});
  Object.assign(m.initialization,{initiallyPaused:true,anchorUnixTimestamp:'-123456789'});
  Object.assign(m.transport,{recentBlockhash:POINTS[16],lastValidBlockHeight:'777777',referenceSlot:'9007199254740997',altAuthority:POINTS[8],altRecentSlot:'9007199254740995'});
  return m;
}
function* permutations(values,prefix=[]) {if(!values.length){yield prefix;return;}for(const v of values)yield* permutations(values.filter(n=>n!==v),[...prefix,v]);}
function literal(m) {
  const pk=n.PublicKey,derive=(program,seeds)=>pk.findProgramAddressSync(seeds,program)[0],squads=new pk(n.IDS.squads),program=new pk(m.programId),multisig=new pk(m.governance.multisigAddress);
  const number=value=>{const b=Buffer.alloc(8);b.writeBigUInt64LE(BigInt(value));return b;};
  const vault=index=>derive(squads,[Buffer.from('multisig'),multisig.toBuffer(),Buffer.from('vault'),Buffer.from([index])]);
  const protocol=[n.IDS.poolProgram,n.IDS.pool,m.protocol.validatorList,m.protocol.reserve,n.IDS.mint,m.protocol.managerFee,m.protocol.referrer].map(s=>new pk(s));
  const recipients=[m.recipients.htfpRecipient,m.recipients.teamOwnerRecipient].map(v=>new pk(v));
  // Literal offsets and flags; no production encoder, fixture or role builder.
  const data=Buffer.alloc(313);data.write('PIV1IN02');data.set([2,m.governance.vaultIndex,Number(m.initialization.initiallyPaused)],8);
  [...protocol,...recipients].forEach((p,i)=>data.set(p.toBytes(),11+i*32));data.writeBigInt64LE(BigInt(m.initialization.anchorUnixTimestamp),299);
  data.set(m.governance.guardianSlotPermutation,307);
  const guardians=m.governance.guardians.map(s=>new pk(s)).sort((a,b)=>Buffer.compare(a.toBuffer(),b.toBuffer()));
  const targets=['config','distribution','guardian-registry'].map(seed=>derive(program,[Buffer.from(seed)]));
  for(let slot=0;slot<6;slot++)targets.push(derive(program,[Buffer.from('guardian-reward'),guardians[m.governance.guardianSlotPermutation[slot]].toBuffer(),Buffer.alloc(8),Buffer.from([slot])]));
  for(const seed of ['pending-sol','principal-sol','operational-sol','distribution-escrow','kif-sol','principal-jito-vault','pending-jito-vault'])targets.push(derive(program,[Buffer.from(seed)]));
  const keys=[program,derive(new pk(n.IDS.loader),[program.toBuffer()]),multisig,
    derive(squads,[Buffer.from('multisig'),multisig.toBuffer(),Buffer.from('transaction'),number(m.governance.transactionIndex),Buffer.from('proposal')]),
    derive(squads,[Buffer.from('multisig'),multisig.toBuffer(),Buffer.from('transaction'),number(m.governance.transactionIndex)]),vault(m.governance.vaultIndex),
    new pk(n.IDS.instructions),...targets,...protocol.slice(0,m.protocol.managerFee===m.protocol.referrer?6:7),new pk(m.rentPayer),new pk(n.IDS.system),new pk(n.IDS.token),...recipients];
  return {data,keys,targets,guardians,recipients};
}
const packet=(p,name)=>p.plan.packets.find(p=>p.name===name);
const decode=p=>VersionedTransaction.deserialize(Buffer.from(p.wireBase64,'base64'));
const data=(p,index=0)=>Buffer.from(p.requestedInstructions[index].dataHex,'hex');

function registerTests() {
const test=require('node:test');
test('pinned source/SDK inputs and null example remain explicit and offline',()=>{
  n.verifySources();const example=JSON.parse(fs.readFileSync(path.join(__dirname,'initializer-plan.example.json')));
  assert.deepEqual(example,n.TEMPLATE);const p=n.planManifest(example);assert.equal(p.status,'INCOMPLETE');assert.equal(p.plan,null);assert(p.missing.length>20);
  assert.equal(p.readiness.publicReady,false);assert.equal(p.readiness.actualRent,null);
  assert.equal(n.planManifest({}).status,'INCOMPLETE');
});
test('arbitrary public declarations match independent313-byte and34/35-role literal oracle',()=>{
  for(const shared of [false,true])for(const fee of ['rent','creator','separate'])for(const vault of [0,23,255]) {
    const m=fixture({shared,fee,vault,index:'18446744073709551615'}),expected=literal(m),p=n.planManifest(m),count=shared?34:35;
    assert.equal(p.status,'OFFLINE_PLAN_ONLY');assert.equal(p.plan.native.bytes,313);assert.equal(p.plan.native.hex,expected.data.toString('hex'));
    assert.deepEqual(p.plan.native.roles.map(r=>r.address),expected.keys.map(k=>k.toBase58()));
    p.plan.native.roles.forEach((r,i)=>{assert.equal(r.signer,i===5||i===count-5);assert.equal(r.writable,(i>=7&&i<23)||i===count-5);});
    assert.deepEqual(p.plan.canonicalDeclaredGuardians,expected.guardians.map(k=>k.toBase58()));
    assert.deepEqual(p.plan.alt.addresses,expected.targets.map(k=>k.toBase58()));assert.equal(p.plan.compact.bytes,shared?1445:1478);
    assert.equal(p.plan.accountSizes.transaction,shared?1546:1579);
    assert.equal(p.plan.accountSizes.transaction,p.plan.compact.bytes+101);
    assert.deepEqual(p.plan.accountSizes.pivTargets,[1014,891,210,84,84,84,84,84,84,0,0,0,0,0,165,165]);
    assert.equal(p.readiness.walletControlVerified,false);assert.equal(p.readiness.actualFees,null);assert.equal(p.plan.alt.actualLastExtendedSlot,null);
  }
});
test('all720 explicit slot permutations encode and public member input order is immaterial',()=>{
  const m=fixture();let count=0;
  for(const permutation of permutations([0,1,2,3,4,5])) {
    const candidate=copy(m);candidate.governance.guardianSlotPermutation=permutation;
    const ctx=n.deriveContext(n.validateManifest(candidate).manifest),expected=literal(candidate);
    assert.deepEqual(n.encodeNative(ctx),expected.data);assert.deepEqual(ctx.targets.map(k=>k.toBase58()),expected.targets.map(k=>k.toBase58()));count++;
  }
  assert.equal(count,720);
  const shuffled=copy(m);shuffled.governance.guardians.reverse();assert.deepEqual(n.planManifest(shuffled).plan,n.planManifest(m).plan);
});
test('signed timestamp/full-u64/byte boundaries preserve literal fields without Number loss',()=>{
  for(const anchor of ['-9223372036854775808','-1','0','1','9223372036854775807'])for(const vault of [0,127,128,255]) {
    const m=fixture({vault,index:'9007199254740993'});m.initialization.anchorUnixTimestamp=anchor;
    const c=n.deriveContext(n.validateManifest(m).manifest),actual=n.encodeNative(c);assert.deepEqual(actual,literal(m).data);
    assert.equal(actual.readBigInt64LE(299),BigInt(anchor));assert.equal(c.transaction.toBase58(),literal(m).keys[4].toBase58());
  }
});
test('buffer/schema plans bind selected vault/index/creator, commitment and exact upload bytes',()=>{
  const m=fixture({fee:'separate',vault:128,index:'9007199254740999'}),p=n.planManifest(m),stored=Buffer.from(p.plan.compact.hex,'hex');
  const first=packet(p,'buffer_create'),b=data(first);assert.equal(first.signatureCount,3);assert(p.plan.compact.uploadChunks[0]<800);
  assert.deepEqual(b.subarray(0,8),Buffer.from([245,201,113,108,37,63,29,89]));
  assert.equal(b[8],m.governance.bufferIndex);assert.equal(b[9],128);assert.deepEqual(b.subarray(10,42),t.hash(stored));assert.equal(b.readUInt16LE(42),stored.length);
  const chunks=[b.subarray(48,48+b.readUInt32LE(44))];
  for(const x of p.plan.packets.filter(x=>x.name.startsWith('buffer_extend_'))){const d=data(x);assert.deepEqual(d.subarray(0,8),Buffer.from([230,157,67,56,5,238,245,146]));assert.equal(d.readUInt32LE(8),d.length-12);chunks.push(d.subarray(12));}
  assert.deepEqual(Buffer.concat(chunks),stored);assert.equal(chunks.reduce((s,b)=>s+b.length,0),stored.length);
  const finish=data(packet(p,'transaction_from_buffer'));assert.deepEqual(finish,Buffer.concat([Buffer.from([222,54,149,68,87,246,48,231]),Buffer.from([128,0,6,0,0,0,0,0,0,0,0,0,0])]));
  const proposal=data(packet(p,'proposal_create'));assert.deepEqual(proposal.subarray(0,8),Buffer.from([220,60,73,224,30,108,79,159]));assert.equal(proposal.readBigUInt64LE(8),9007199254740999n);assert.equal(proposal[16],0);
  for(let i=0;i<4;i++){const a=packet(p,`approve_${i+1}`);assert.equal(a.requestedInstructions[0].accounts[1].address,m.governance.approvers[i]);assert.deepEqual(data(a),Buffer.from([144,37,164,136,188,216,42,248,0]));}
});
test('ALT create/extend encode exactu64,16targets and hypothetical warm-up constraints',()=>{
  const m=fixture(),p=n.planManifest(m),create=packet(p,'alt_create'),extend=packet(p,'alt_extend'),a=data(create),b=data(extend);
  assert.equal(a.length,13);assert.equal(a.readUInt32LE(),0);assert.equal(a.readBigUInt64LE(4),9007199254740995n);
  const seed=Buffer.alloc(8);seed.writeBigUInt64LE(9007199254740995n);const [address,bump]=n.PublicKey.findProgramAddressSync([new n.PublicKey(m.transport.altAuthority).toBuffer(),seed],new n.PublicKey(n.IDS.altProgram));
  assert.equal(address.toBase58(),p.plan.alt.address);assert.equal(a[12],bump);assert.equal(create.requestedInstructions[0].accounts[1].address,m.transport.altAuthority);
  assert.equal(b.readUInt32LE(),2);assert.equal(b.readBigUInt64LE(4),16n);assert.equal(b.length,12+16*32);
  assert.deepEqual(Array.from({length:16},(_,i)=>new n.PublicKey(b.subarray(12+i*32,44+i*32)).toBase58()),p.plan.alt.addresses);
  assert.equal(p.plan.alt.compilationState,'HYPOTHETICAL_ONLY');assert.equal(p.plan.alt.warmupVerified,false);assert(p.plan.alt.preconditions.some(x=>x.includes('strictly greater')));
  assert.equal(p.plan.referenceContext.verified,false);assert(p.plan.referenceContext.usage.includes('refresh blockhash'));
});
test('actual packet signer unions include1/2/3zero slots and full-budget v0 reconstructs native roles',()=>{
  const observed=new Set();
  for(const fee of ['rent','creator','executor','separate']) {
    const m=fixture({fee});if(fee==='rent')m.transport.altAuthority=m.rentPayer;
    const p=n.planManifest(m),c=n.deriveContext(n.validateManifest(m).manifest);
    for(const packet of p.plan.packets) {
      const tx=decode(packet),count=tx.message.header.numRequiredSignatures;observed.add(count);assert.equal(packet.signatureCount,count);assert.equal(packet.zeroSignatureBytes,64*count);
      assert(tx.signatures.every(s=>s.length===64&&s.every(b=>b===0)));assert.equal(Buffer.from(packet.wireBase64,'base64').length,packet.wireBytes);assert(packet.wireBytes<=1232);
      assert.deepEqual(tx.message.staticAccountKeys.slice(0,count).map(k=>k.toBase58()),packet.requiredSigners);
    }
    const x=packet(p,'execute'),tx=decode(x),table=new AddressLookupTableAccount({key:new n.PublicKey(p.plan.alt.address),state:{deactivationSlot:(1n<<64n)-1n,lastExtendedSlot:0,lastExtendedSlotStartIndex:0,authority:new n.PublicKey(m.transport.altAuthority),addresses:c.targets}});
    const recovered=TransactionMessage.decompile(tx.message,{addressLookupTableAccounts:[table]}).instructions;
    assert.deepEqual(recovered.slice(0,3).map(i=>i.data.toString('hex')),['02c05c1500','0100000400','0400008000']);assert(recovered.slice(0,3).every(i=>i.keys.length===0));
    const execute=recovered[3],compact=t.decodeCompact(Buffer.from(p.plan.compact.hex,'hex'));
    assert.equal(execute.data.toString('hex'),'c208a15799a419ab');assert.equal(execute.keys[1].pubkey.toBase58(),p.plan.proposal);assert(execute.keys[1].isWritable);
    assert.equal(execute.keys[3].pubkey.toBase58(),m.governance.executor);assert(execute.keys[3].isSigner);
    assert.equal(execute.keys[3].isWritable,m.feePayer===m.governance.executor);
    assert(!execute.keys.find(k=>k.pubkey.toBase58()===p.plan.vault).isSigner);
    assert(!compact.keys.find(k=>k.pubkey.toBase58()===p.plan.proposal).isWritable);
    assert.deepEqual(compact.indices.map(i=>compact.keys[i].pubkey.toBase58()),p.plan.native.roles.map(r=>r.address));
    const wrong=new AddressLookupTableAccount({key:table.key,state:{...table.state,addresses:[...c.targets].reverse()}});
    const changed=TransactionMessage.decompile(tx.message,{addressLookupTableAccounts:[wrong]}).instructions[3];assert.notDeepEqual(changed.keys.map(k=>k.pubkey.toBase58()),execute.keys.map(k=>k.pubkey.toBase58()));
  }
  assert.deepEqual([...observed].sort(),[1,2,3]);
});
test('independent three-signature adjacent1232-byte bound is enforced before oversize serialization',()=>{
  const m=fixture({fee:'separate'}),c=n.deriveContext(n.validateManifest(m).manifest),keys=[{pubkey:new n.PublicKey(m.governance.creator),isSigner:true,isWritable:false},{pubkey:new n.PublicKey(m.rentPayer),isSigner:true,isWritable:true}];
  // 3signatures +4static keys +one2-account instruction gives364bytes overhead.
  const make=size=>new TransactionInstruction({programId:new n.PublicKey(n.IDS.squads),keys,data:Buffer.alloc(size)});
  assert.equal(n.packet(c,'boundary',[make(868)]).wireBytes,1232);assert.throws(()=>n.packet(c,'oversized',[make(869)]));
});
test('strict JSON/known-field validation rejects malformed partial inputs without raw-value echo',()=>{
  for(const source of ['{"feePayer":null,"feePayer":null}','{"feePayer":null,"fee\\u0050ayer":null}','[]','{"unexpected":"do-not-echo"}'])assert.throws(()=>n.planManifest(n.parseManifest(source)));
  assert.throws(()=>n.parseManifest('{'));assert.throws(()=>n.parseManifest(' '.repeat(65537)));
  for(const [field,value]of [['schema','other'],['cluster','mainnet-beta'],['feePayer','1'],['rentPayer',7]])assert.throws(()=>n.planManifest({...copy(n.TEMPLATE),[field]:value}));
  assert.throws(()=>n.planManifest({...copy(n.TEMPLATE),programId:n.IDS.pool}));
  for(const anchor of [0,'-0','+1','01','1e3','1.0','9223372036854775808','-9223372036854775809']){const m=copy(n.TEMPLATE);m.initialization.anchorUnixTimestamp=anchor;assert.throws(()=>n.planManifest(m));}
  for(const index of [1,'0','-1','01','18446744073709551616']){const m=copy(n.TEMPLATE);m.governance.transactionIndex=index;assert.throws(()=>n.planManifest(m));}
  const unknown=copy(n.TEMPLATE);unknown.protocol.secret='do-not-echo';assert.throws(()=>n.planManifest(unknown));
});
test('membership/permutation/index/alias/off-curve/context failures reject rather than choose substitutes',()=>{
  const changes=[m=>m.governance.approvers[1]=m.governance.approvers[0],m=>m.governance.approvers[0]=POINTS[14],m=>m.governance.creator=POINTS[14],m=>m.governance.executor=POINTS[14],
    m=>m.governance.guardians[1]=m.governance.guardians[0],m=>m.governance.guardianSlotPermutation=[0,1,2,3,4,4],m=>m.governance.guardianSlotPermutation[0]=6,
    m=>m.rentPayer=m.governance.executor,m=>m.recipients.htfpRecipient=m.governance.multisigAddress,m=>m.recipients.teamOwnerRecipient=m.recipients.htfpRecipient,
    m=>m.governance.multisigAddress=POINTS[14],m=>m.protocol.validatorList=m.protocol.reserve,m=>m.programId=n.IDS.pool,
    m=>m.governance.currentTransactionIndex='18446744073709551615',m=>m.governance.currentTransactionIndex='5',m=>m.governance.staleTransactionIndex='999',
    m=>m.transport.altRecentSlot='9007199254740998',m=>m.initialization.initiallyPaused=0,m=>m.governance.bufferIndex=256,m=>m.governance.vaultIndex=-1,
    m=>m.feePayer=m.governance.multisigAddress,m=>m.transport.altAuthority=m.governance.multisigAddress];
  for(const change of changes){const m=fixture();change(m);assert.throws(()=>n.planManifest(m));}
  const m=fixture();m.initialization.anchorUnixTimestamp=null;const p=n.planManifest(m);assert.equal(p.status,'INCOMPLETE');assert.equal(p.plan,null);assert(p.missing.includes('initialization.anchorUnixTimestamp'));
});
test('CLI template/incomplete/complete/invalid commands have explicit exit semantics',context=>{
  const script=path.join(__dirname,'initializer-plan.cjs'),limits=process.execArgv.filter(s=>s.startsWith('--max-old-space-size'));
  const run=args=>spawnSync(process.execPath,[...limits,script,...args],{encoding:'utf8',maxBuffer:2*1024*1024,timeout:30000});
  const template=run(['--template']);assert.equal(template.status,0);assert.deepEqual(JSON.parse(template.stdout),n.TEMPLATE);assert.equal(template.stderr,'');
  const incomplete=run(['--manifest',path.join(__dirname,'initializer-plan.example.json')]);assert.equal(incomplete.status,0);assert.equal(JSON.parse(incomplete.stdout).status,'INCOMPLETE');assert.equal(JSON.parse(incomplete.stdout).plan,null);
  const directory=fs.mkdtempSync(path.join(os.tmpdir(),'piv1-public-plan-test-')),filename=path.join(directory,'public-manifest.json'),manifest=fixture({shared:true,fee:'separate'});
  fs.writeFileSync(filename,JSON.stringify(manifest,null,2)+'\n',{flag:'wx'});
  const complete=run(['--manifest',filename]);assert.equal(complete.status,0);assert.equal(complete.stderr,'');const parsed=JSON.parse(complete.stdout);
  assert.equal(parsed.status,'OFFLINE_PLAN_ONLY');assert.deepEqual(parsed.plan,n.planManifest(manifest).plan);assert.equal(parsed.readiness.publicReady,false);
  assert(parsed.plan.packets.every(p=>p.wireBytes<=1232));assert.equal(parsed.plan.native.hex,literal(manifest).data.toString('hex'));
  context.diagnostic(`Complete public-only fixture retained: ${filename}`);
  for(const args of [[],['--send'],['--template','--rpc'],['--manifest']]){const bad=run(args);assert.notEqual(bad.status,0);assert.equal(bad.stdout,'');assert.equal(JSON.parse(bad.stderr).status,'INVALID_OR_UNSUPPORTED_PLAN');}
});
}
module.exports={fixture,publicPoints,literal};
if(require.main===module)registerTests();
