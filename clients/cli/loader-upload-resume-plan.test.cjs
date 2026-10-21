'use strict';
const test=require('node:test'),assert=require('node:assert/strict'),fs=require('node:fs');
const {createHash}=require('node:crypto');
const p=require('./loader-upload-resume-plan.cjs'),base=require('./initializer-plan.cjs'),t=require('../../validation/genesis-transport/transport.cjs');
// Test-only public ELF path, supplied by root. The planner accepts bytes, never an artifact path.
const artifactPath=process.env.PIV1_UPLOAD_TEST_ARTIFACT;
assert(artifactPath,'root must supply PIV1_UPLOAD_TEST_ARTIFACT for the public retained ELF');
const artifact=fs.readFileSync(artifactPath),sha=b=>createHash('sha256').update(b).digest('hex');
assert.equal(artifact.length,894552);assert.equal(sha(artifact),'e29735e1467da98ecddb1eea9fd25abaffa4a450bd899ddc137f7d8c0da35b99');
const b58alphabet='123456789ABCDEFGHJKLMNPQRSTUVWXYZabcdefghijkmnopqrstuvwxyz';
function b58(bytes){let n=0n;for(const b of bytes)n=(n<<8n)+BigInt(b);let out='';while(n){out=b58alphabet[Number(n%58n)]+out;n/=58n;}for(const b of bytes){if(b!==0)break;out='1'+out;}return out;}
function payload(m){return Buffer.from(m.snapshot.buffer.dataBase64,'base64');}
function setPayload(m,b){m.snapshot.buffer.dataBase64=b.toString('base64');}
function fixture(){
  const q=p.PROFILE,b=Buffer.alloc(894589);b.writeUInt32LE(1);b[4]=1;new base.PublicKey(q.writer).toBuffer().copy(b,5);
  return {schema:p.SCHEMA,cluster:'testnet',genesisHash:q.genesisHash,programId:q.programId,programDataAddress:q.programDataAddress,writer:q.writer,feePayer:q.writer,bufferAddress:q.bufferAddress,bufferSeed:q.bufferSeed,
    artifact:{bytes:894552,sha256:q.artifactSha256,dataBase64:artifact.toString('base64')},snapshot:{contextSlot:'18446744073709551615',
      buffer:{exists:true,address:q.bufferAddress,owner:q.loader,executable:false,lamports:'4545162360',rentExemptMinimumLamports:'4545162360',dataBase64:b.toString('base64')},
      program:{address:q.programId,exists:false,contextSlot:'18446744073709551615'},programData:{address:q.programDataAddress,exists:false,contextSlot:'18446744073709551615'}},
    transport:{recentBlockhash:'11111111111111111111111111111111',currentBlockHeight:'18446744073709551465',lastValidBlockHeight:'18446744073709551615',minimumRemainingBlockHeights:'150'},batch:{maxPackets:64}};
}
function matchChunk(b,index){const begin=index*956,end=Math.min(begin+956,894552);artifact.copy(b,37+begin,begin,end);}
function rawPacket(packet){
  const wire=Buffer.from(packet.wireBase64,'base64');let cursor=0;
  const byte=()=>{assert(cursor<wire.length);return wire[cursor++];},take=n=>{assert(cursor+n<=wire.length);const out=wire.subarray(cursor,cursor+n);cursor+=n;return out;};
  const short=()=>{let n=0,shift=0,x;do{x=byte();n+=(x&127)*2**shift;shift+=7;assert(shift<=21);}while(x&128);return n;};
  assert.equal(short(),1);assert(take(64).every(x=>x===0));const messageStart=cursor;
  assert.deepEqual([...take(3)],[1,0,2]);assert.equal(short(),4);const keys=Array.from({length:4},()=>b58(take(32)));
  assert.deepEqual(keys,[p.PROFILE.writer,p.PROFILE.bufferAddress,'ComputeBudget111111111111111111111111111111',p.PROFILE.loader]);
  const blockhash=b58(take(32));assert.equal(short(),4);const ix=[];
  for(let i=0;i<4;i++){const program=byte(),indices=[...take(short())],data=take(short());ix.push({program,indices,data});}
  assert.equal(cursor,wire.length);assert.deepEqual(ix.slice(0,3).map(i=>({program:i.program,indices:i.indices,hex:i.data.toString('hex')})),[
    {program:2,indices:[],hex:'02c05c1500'},{program:2,indices:[],hex:'0100000400'},{program:2,indices:[],hex:'0400000004'}]);
  assert.equal(ix[3].program,3);assert.deepEqual(ix[3].indices,[1,0]);const data=ix[3].data;
  assert.equal(data.readUInt32LE(0),1);const offset=data.readUInt32LE(4),length=data.readBigUInt64LE(8);assert.equal(length,BigInt(data.length-16));
  assert.equal(offset,packet.offset);assert.equal(Number(length),packet.payloadBytes);assert.equal(wire.length,Number(length)+276);
  assert.equal(sha(wire),packet.sha256);assert.equal(sha(wire.subarray(messageStart)),packet.messageSha256);assert.equal(wire.subarray(messageStart).toString('base64'),packet.messageBase64);
  assert.deepEqual(packet.globalRoles,[{address:keys[0],signer:true,writable:true},{address:keys[1],signer:false,writable:true},{address:keys[2],signer:false,writable:false},{address:keys[3],signer:false,writable:false}]);
  return {offset,data:data.subarray(16),blockhash};
}
function rejected(m,code){assert.throws(()=>p.planManifest(m),e=>e.code===code,code);}
test('pinned source contract, nullable template and explicit absence distinction',()=>{
  const source=p.verifySources();assert.equal(source.node,'v24.19.0');assert.equal(source.web3,'1.98.4');
  assert.deepEqual(JSON.parse(fs.readFileSync(`${__dirname}/loader-upload-resume-plan.example.json`,'utf8')),p.TEMPLATE);
  const blank=p.planManifest(p.TEMPLATE);assert.equal(blank.status,'INCOMPLETE');assert.equal(blank.plan,null);assert(blank.missing.includes('snapshot.program.exists'));
  for(const role of ['program','programData']){const m=fixture();delete m.snapshot[role].exists;const r=p.planManifest(m);assert.equal(r.plan,null);assert(r.missing.includes(`snapshot.${role}.exists`));}
  assert.equal(p.planManifest(fixture()).status,'OFFLINE_PLAN_ONLY');
});
test('literal cursors bind full956 and tail692 writes, roles, hashes and1232/1233 boundary',()=>{
  const m=fixture(),r=p.planManifest(m);assert.equal(r.plan.chunkCount,936);assert.equal(r.plan.progress.pendingChunks,936);assert.deepEqual(r.plan.selectedIndices,Array.from({length:64},(_,i)=>i));
  for(const packet of r.plan.packets){const raw=rawPacket(packet);assert(raw.data.equals(artifact.subarray(raw.offset,raw.offset+956)));assert.equal(packet.wireBytes,1232);}
  const b=payload(m);for(let i=0;i<935;i++)matchChunk(b,i);setPayload(m,b);const tail=p.planManifest(m).plan.packets[0],decoded=rawPacket(tail);
  assert.equal(tail.chunkIndex,935);assert.equal(decoded.offset,893860);assert.equal(decoded.data.length,692);assert.equal(tail.wireBytes,968);assert(decoded.data.equals(artifact.subarray(893860)));
  const c={m:{feePayer:m.feePayer,transport:m.transport}},tooBig=p.writeInstructions(0,artifact.subarray(0,957));assert.equal(t.legacyWireSize(base.compile(c,tooBig)),1233);assert.throws(()=>base.packet(c,'oversize',tooBig),/1232/);
  assert.deepEqual(r.readiness,{operational:false,currentStateVerified:false,snapshotAuthenticated:false,blockhashVerified:false,authorized:false,publicReady:false,artifactDeploymentSelected:false,bootstrapTrustSelected:false,signerControlVerified:false,fundingSufficient:false,throughputGuaranteed:false,m4Reservation:'OPEN_UNWAIVED'});
});
test('finite generated mixed states preserve matched chunks and reconstruct exact artifact by bounded replay',()=>{
  // Eight deterministic domains include sparse, alternating, tail-only and maximum-size batches.
  for(let trial=0;trial<8;trial++){
    const m=fixture(),b=payload(m),pending=[];m.batch.maxPackets=[1,3,64,7,2,64,11,3][trial];
    for(let i=0;i<936;i++){const leave=(i*17+trial*13)%97<trial+1||i===935||i===0;if(leave)pending.push(i);else matchChunk(b,i);}setPayload(m,b);
    const original=Buffer.from(b);let expected=[...pending],rounds=0;
    while(expected.length){const r=p.planManifest(m),before=payload(m),indices=expected.slice(0,m.batch.maxPackets);assert.deepEqual(r.plan.pendingIndices,expected);assert.deepEqual(r.plan.selectedIndices,indices);assert.deepEqual(r.plan.remainingIndices,expected.slice(indices.length));
      for(const packet of r.plan.packets){const decoded=rawPacket(packet);decoded.data.copy(before,37+decoded.offset);assert(decoded.data.equals(artifact.subarray(decoded.offset,decoded.offset+decoded.data.length)));}
      for(let i=0;i<936;i++)if(!pending.includes(i)){const begin=37+i*956,end=Math.min(begin+956,894589);assert(before.subarray(begin,end).equals(original.subarray(begin,end)));}
      setPayload(m,before);expected=expected.slice(indices.length);assert(++rounds<100);
    }
    const done=p.planManifest(m);assert.equal(done.plan.packets.length,0);assert(done.plan.allBytesMatchSuppliedArtifact);assert.equal(done.plan.progress.matchedBytes,894552);assert.equal(done.plan.progress.confirmedTransactions,null);assert(payload(m).subarray(37).equals(artifact));
    assert(done.plan.chunks.every(c=>c.status==='MATCHED_BYTES'));assert.equal(done.readiness.currentStateVerified,false);
  }
});
test('full936 upload uses at most64 per batch and replay equality does not prove transaction history',()=>{
  const m=fixture();let batches=0,total=0;
  while(true){const r=p.planManifest(m);if(!r.plan.packets.length)break;const b=payload(m);assert(r.plan.packets.length<=64);for(const packet of r.plan.packets){const decoded=rawPacket(packet);decoded.data.copy(b,37+decoded.offset);}setPayload(m,b);total+=r.plan.packets.length;assert(++batches<=15);}
  assert.equal(total,936);assert.equal(batches,15);assert(payload(m).subarray(37).equals(artifact));
  const exact=p.planManifest(m);assert.equal(exact.plan.progress.confirmedTransactions,null);assert.equal(exact.plan.nextStateVerified,false);assert.equal(exact.plan.transport.remainingBlockHeights,'150');
});
test('all936 chunks checked before first packet: late conflicts and partial nonzero writes stop',()=>{
  for(const index of [0,63,64,511,935]){const m=fixture(),b=payload(m),at=37+index*956;b[at]=artifact[index*956]===255?254:255;setPayload(m,b);rejected(m,'conflicting_buffer_chunk');}
  const m=fixture(),b=payload(m);b[b.length-1]=artifact[artifact.length-1]===255?254:255;setPayload(m,b);rejected(m,'conflicting_buffer_chunk');
  const partial=fixture(),bytes=payload(partial);artifact.copy(bytes,37,0,100);setPayload(partial,bytes);rejected(partial,'conflicting_buffer_chunk');
});
test('exact u64 lifetime and positive margin enforce declared boundaries without throughput inference',()=>{
  const m=fixture();m.transport.currentBlockHeight='0';m.transport.lastValidBlockHeight='18446744073709551615';m.transport.minimumRemainingBlockHeights='18446744073709551615';assert.equal(p.planManifest(m).plan.transport.remainingBlockHeights,'18446744073709551615');
  for(const [current,last,margin,code]of [['150','150','1','insufficient_declared_margin'],['151','150','1','declared_blockhash_expired'],['149','150','2','insufficient_declared_margin'],['149','150','0','positive_margin_required']]){const x=fixture();Object.assign(x.transport,{currentBlockHeight:current,lastValidBlockHeight:last,minimumRemainingBlockHeights:margin});rejected(x,code);}
  const one=fixture();Object.assign(one.transport,{currentBlockHeight:'149',lastValidBlockHeight:'150',minimumRemainingBlockHeights:'1'});assert.equal(p.planManifest(one).plan.transport.remainingBlockHeights,'1');
  for(const bad of ['18446744073709551616','01','-1','1e2',1,Number.MAX_SAFE_INTEGER+1]){const x=fixture();x.transport.currentBlockHeight=bad;rejected(x,typeof bad==='string'&&bad==='18446744073709551616'?'integer_range':'decimal_string_required');}
  const changed=fixture();changed.transport.recentBlockhash=p.PROFILE.genesisHash;assert.equal(rawPacket(p.planManifest(changed).plan.packets[0]).blockhash,p.PROFILE.genesisHash);
});
test('identity, snapshot, owner, authority, rent, bytes and batch malformed domains fail closed',()=>{
  const cases=[
    [m=>m.cluster='mainnet-beta','pinned_profile_mismatch'],[m=>m.feePayer=m.programId,'pinned_profile_mismatch'],[m=>m.bufferSeed+='x','pinned_profile_mismatch'],
    [m=>m.artifact.bytes++,'artifact_size'],[m=>m.artifact.sha256=m.artifact.sha256.toUpperCase(),'artifact_hash'],[m=>m.artifact.path='/tmp/unread','unknown_field'],
    [m=>m.artifact.dataBase64='A'+m.artifact.dataBase64.slice(1),'artifact_hash'],[m=>m.artifact.dataBase64+='\n','canonical_base64_required'],
    [m=>m.snapshot.buffer.exists=false,'buffer_absent_or_closed'],[m=>m.snapshot.buffer.executable=true,'buffer_precondition'],[m=>m.snapshot.buffer.owner=m.programId,'buffer_precondition'],
    [m=>m.snapshot.buffer.lamports='0','declared_buffer_rent_shortfall'],[m=>m.snapshot.buffer.lamports='4545162359','declared_buffer_rent_shortfall'],[m=>m.snapshot.buffer.rentExemptMinimumLamports='0','positive_declared_rent_required'],
    [m=>m.snapshot.program.exists=true,'program_or_programdata_present'],[m=>m.snapshot.programData.exists=true,'program_or_programdata_present'],[m=>m.snapshot.program.exists=0,'program_or_programdata_present'],
    [m=>m.snapshot.program.contextSlot='1','snapshot_context_mismatch'],[m=>m.snapshot.programData.address=m.programId,'pinned_profile_mismatch'],
    [m=>m.transport.recentBlockhash='1'.repeat(33),'invalid_public_key'],[m=>m.transport.recentBlockhash='0'.repeat(32),'invalid_public_key']
  ];
  for(const [mutate,code]of cases){const m=fixture();mutate(m);rejected(m,code);}
  for(const bad of [0,65,1.5,'1',true]){const m=fixture();m.batch.maxPackets=bad;rejected(m,'batch_limit');}
  for(const [offset,value,code]of [[0,0,'buffer_header'],[4,0,'buffer_header'],[4,2,'buffer_header'],[5,0,'buffer_authority']]){const m=fixture(),b=payload(m);b[offset]=value;setPayload(m,b);rejected(m,code);}
  for(const tail of [-1,1]){const m=fixture(),b=payload(m);setPayload(m,tail<0?b.subarray(0,b.length-1):Buffer.concat([b,Buffer.alloc(1)]));rejected(m,'canonical_base64_required');}
  // Nonzero pad bits decode to the same bytes, but are not canonical base64.
  const m=fixture(),s=m.snapshot.buffer.dataBase64;m.snapshot.buffer.dataBase64=s.slice(0,-3)+'B==';rejected(m,'canonical_base64_required');
});
test('bounded duplicate-key parser and nullable input reject malformed or hidden declarations',()=>{
  for(const text of ['{"schema":null,"schema":null}','{"x":{"exists":false,"exi\\u0073ts":true}}'])assert.throws(()=>p.parseManifest(text),e=>e.code==='duplicate_json_key');
  assert.throws(()=>p.parseManifest('['.repeat(9)+'0'+']'.repeat(9)),e=>e.code==='manifest_depth');
  assert.throws(()=>p.parseManifest(' '.repeat(p.MAX_MANIFEST_BYTES+1)),e=>e.code==='manifest_size');
  assert.throws(()=>p.parseManifest('{'),e=>e.code==='invalid_json');assert.deepEqual(p.parseManifest(JSON.stringify(p.TEMPLATE)),p.TEMPLATE);
  const m=fixture();delete m.snapshot.buffer.dataBase64;assert.equal(p.planManifest(m).plan,null);assert.throws(()=>p.validateManifest([]),e=>e.code==='object_required');
});
