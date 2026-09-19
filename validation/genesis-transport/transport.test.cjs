'use strict';

const assert = require('node:assert/strict');
const { test } = require('node:test');
const fs = require('node:fs');
const path = require('node:path');
const t = require('./transport.cjs');
const copyIx = ix => ({ programId:ix.programId, data:Buffer.from(ix.data),
  keys:ix.keys.map(m=>({...m})) });
const copyPlan = plan => ({ address:plan.address,create:copyIx(plan.create),
  extend:plan.extend.map(copyIx),finish:copyIx(plan.finish) });
const same = (a,b) => a.equals(b);

test('literal Anchor discriminators and persisted u32 layout remain distinct from compact upload bytes',()=>{
  const literal={vault_transaction_create:[48,250,78,168,208,226,218,211],
    vault_transaction_execute:[194,8,161,87,153,164,25,171],
    transaction_buffer_create:[245,201,113,108,37,63,29,89],
    transaction_buffer_extend:[230,157,67,56,5,238,245,146],
    vault_transaction_create_from_buffer:[222,54,149,68,87,246,48,231]};
  for(const [name,bytes] of Object.entries(literal))assert.deepEqual([...t.disc(name)],bytes);
  const f=t.fixture(),compact=t.compactMessage(f),stored=t.decodeCompact(compact);
  const word=n=>{const b=Buffer.alloc(4);b.writeUInt32LE(n);return b;};
  const persisted=Buffer.concat([compact.subarray(0,3),word(stored.keys.length),
    ...stored.keys.map(m=>m.pubkey.toBuffer()),word(1),Buffer.from([stored.programIndex]),
    word(stored.indices.length),Buffer.from(stored.indices),word(stored.data.length),stored.data,word(0)]);
  assert.equal(persisted.length,1426);assert.equal(compact.length,1412);
  assert.throws(()=>t.decodeCompact(persisted));
});

test('concrete genesis roles and fixed bytes match the retained library format',()=>{
  const source=fs.readFileSync(path.join(t.REPO,'programs/piv1/src/instructions/initialize.rs'),'utf8');
  assert.match(source,/GENESIS_MODEL_DATA_SIZE: usize = 313/);
  assert.match(source,/GENESIS_MODEL_SELECTOR: \[u8; 8\] = \*b"PIV1GM01"/);
  assert.match(source,/GENESIS_MODEL_VERSION: u8 = 1/);
  for(const programTag of [217,211])for(const sameReceiver of [false,true]) {
    const f=t.fixture({programTag,sameReceiver});
    assert.equal(f.roles.length,sameReceiver?32:33);assert.equal(f.targets.length,16);
    assert.equal(f.inner.keys.filter(m=>m.isSigner).length,2);
    assert.equal(f.inner.keys.filter(m=>m.isWritable).length,17);
    assert(f.inner.keys.find(m=>same(m.pubkey,f.vault)).isSigner);
    assert(!f.inner.keys.find(m=>same(m.pubkey,f.vault)).isWritable);
    assert(!f.roles.some(([,m])=>same(m.pubkey,f.creator)));
    assert.equal(f.inner.data.subarray(0,8).toString(),'PIV1GM01');
    assert.deepEqual([...f.inner.data.subarray(8,11)],[1,7,0]);
    assert.deepEqual(f.inner.data.subarray(235,267),f.creator.toBuffer());
    assert.equal(f.inner.data.readBigInt64LE(299),0n);
    assert.deepEqual([...f.inner.data.subarray(307)],[5,4,3,2,1,0]);
    const compact=t.compactMessage(f), decoded=t.decodeCompact(compact);
    assert.equal(compact.length,sameReceiver?1379:1412);assert.equal(compact[3],f.roles.length);
    // Creation SmallVec lengths: one byte each for keys/instructions/accounts,
    // two bytes for payload. Persisted-account u32 Vec encoding is different.
    assert.equal(compact.readUInt16LE(7+33*f.roles.length),313);
    assert.equal(compact.at(-1),0);t.assertCompactMatches(f,decoded);
    assert.deepEqual(decoded.keys.filter(m=>m.isSigner).map(m=>m.pubkey.toBase58()),
      [f.payer.toBase58(),f.vault.toBase58()]);
  }
});

test('all eight SDK packet cases preserve exact stored/outer semantics and fit only the v0 route',()=>{
  const report=t.report();assert.equal(report.cases.length,8);
  for(const c of report.cases) {
    assert.equal(c.storedLookupCount,0);assert.equal(c.outerLookupCount,1);assert.equal(c.loadedTargetCount,16);
    assert.equal(c.signatures,2);assert.equal(c.innerBytes,313);assert.equal(c.buffer.chunks[0],800);
    assert(c.bufferPackets.every(p=>p.within1232));assert(c.v0Execute.within1232);assert(!c.legacyExecute.within1232);
    assert.equal(c.legacyExecute.bytes,(c.sharedReceiver?1301:1334)+(c.computePrefix?48:0));
    assert.equal(c.v0Execute.bytes,(c.sharedReceiver?841:874)+(c.computePrefix?48:0));
    assert.equal(c.directCreate.calculatedBytes,c.sharedReceiver?1761:1794);
    const fromBufferBytes=129+3+1+224+32+1+1+1+7+1+21;
    assert.equal(fromBufferBytes,421);
    assert.deepEqual(c.bufferPackets.map(p=>p.bytes),[1215,c.sharedReceiver?924:957,fromBufferBytes]);
  }
});

test('v0 needs enough lookup compression; signatures remain static and all zero',()=>{
  const f=t.fixture(),stored=t.decodeCompact(t.compactMessage(f)),ix=t.executeInstruction(f,stored);
  for(const count of [0,4,5,16]) {
    const result=t.roundtrip(f,[ix],[t.lookup(f,count)]);
    assert.equal(result.withinLimit,count>=5);
    if(count>0)assert.equal(result.bytes,1370-31*count);
    assert.equal(result.message.header.numRequiredSignatures,2);
    const staticSigners=result.message.staticAccountKeys.slice(0,2).map(k=>k.toBase58());
    assert.deepEqual(staticSigners,[f.payer.toBase58(),f.creator.toBase58()]);
    assert(!staticSigners.includes(f.vault.toBase58()));t.assertExecution(f,stored,result.instructions[0]);
  }
});

test('loaded table identity/index contents must reconstruct the exact approved account list',()=>{
  const f=t.fixture(),stored=t.decodeCompact(t.compactMessage(f)),table=t.lookup(f);
  const result=t.roundtrip(f,[t.executeInstruction(f,stored)],[table]);
  const wrong=t.lookup(f);[wrong.state.addresses[0],wrong.state.addresses[1]]=[wrong.state.addresses[1],wrong.state.addresses[0]];
  const recovered=t.TransactionMessage.decompile(result.message,{addressLookupTableAccounts:[wrong]});
  assert.throws(()=>t.assertExecution(f,stored,recovered.instructions[0]));
  const missing=t.lookup(f,15);assert.throws(()=>t.TransactionMessage.decompile(result.message,{addressLookupTableAccounts:[missing]}));
  const substituted=t.lookup(f);substituted.key=new t.PublicKey(Buffer.alloc(32,231));
  assert.throws(()=>t.TransactionMessage.decompile(result.message,{addressLookupTableAccounts:[substituted]}));
});

test('outer signers/privilege union cannot substitute for the approved inner privileges',()=>{
  const f=t.fixture(),stored=t.decodeCompact(t.compactMessage(f));
  const good=t.roundtrip(f,[t.executeInstruction(f,stored)],[t.lookup(f)]).instructions[0];
  const remainingProposal=4+stored.keys.findIndex(m=>same(m.pubkey,f.proposal));
  assert(good.keys[remainingProposal].isWritable);
  assert(!stored.keys[remainingProposal-4].isWritable);
  for(const mutation of ['payer','vault','creator','target','proposal','program','data']) {
    const ix=copyIx(good);
    if(mutation==='payer')ix.keys.find(m=>same(m.pubkey,f.payer)).isSigner=false;
    if(mutation==='vault')ix.keys.find(m=>same(m.pubkey,f.vault)).isSigner=true;
    if(mutation==='creator')ix.keys[3].isSigner=false;
    if(mutation==='target')ix.keys.find(m=>same(m.pubkey,f.targets[0])).isWritable=false;
    if(mutation==='proposal')ix.keys[remainingProposal].isWritable=false;
    if(mutation==='program')ix.keys.find(m=>same(m.pubkey,f.program)).isWritable=true;
    if(mutation==='data')ix.data[0]^=1;
    assert.throws(()=>t.assertExecution(f,stored,ix),mutation);
  }
  const altered=t.decodeCompact(t.compactMessage(f));altered.data=Buffer.from(altered.data);altered.data[11]^=1;
  assert.throws(()=>t.assertExecution(f,altered,good));
});

test('compact schema rejects every truncation, trailing bytes, lookup payload and malformed indices',()=>{
  const f=t.fixture(),bytes=t.compactMessage(f),n=f.roles.length;
  for(let end=0;end<bytes.length;end++)assert.throws(()=>t.decodeCompact(bytes.subarray(0,end)));
  assert.throws(()=>t.decodeCompact(Buffer.concat([bytes,Buffer.from([0])])));
  for(const mutate of [b=>b[0]=255,b=>b[1]=255,b=>b[2]=255,b=>b[3]=0,
    b=>b[4+32*n]=2,b=>b[5+32*n]=255,b=>b[7+32*n]=255,
    b=>b[8+32*n]=b[7+32*n],b=>b[b.length-1]=1,
    b=>b.copy(b,36,4,36),b=>b.writeUInt16LE(0xffff,7+33*n)]) {
    const b=Buffer.from(bytes);mutate(b);assert.throws(()=>t.decodeCompact(b));
  }
});

test('buffer create/extend/finalize authority, hash, size, chunks and placeholder remain bound',()=>{
  const f=t.fixture(),compact=t.compactMessage(f),plan=t.bufferPlan(f);
  const summary=t.validateBufferPlan(f,plan);assert.equal(summary.bufferAccountBytes,1524);
  for(const mutate of [
    p=>p.create.data[10]^=1,p=>p.create.data.writeUInt16LE(10129,42),
    p=>p.create.data.writeUInt16LE(compact.length-1,42),p=>p.extend[0].data[12]^=1,
    p=>p.extend=[],p=>p.extend.push(copyIx(p.extend[0])),p=>p.finish.data[14]=1,
    p=>p.finish.data[9]=1,p=>p.finish.data[8]=8,p=>p.finish.keys[6].isSigner=false,
    p=>p.extend[0].keys[2].pubkey=f.payer,p=>p.create.keys[3].isSigner=false,
    p=>p.address=new t.PublicKey(Buffer.alloc(32,230)),
  ]) {const p=copyPlan(plan);mutate(p);assert.throws(()=>t.validateBufferPlan(f,p));}
  const many=t.bufferPlan(f,compact,300);[many.extend[0],many.extend[1]]=[many.extend[1],many.extend[0]];
  assert.throws(()=>t.validateBufferPlan(f,many));
  const r=t.roundtrip(f,[plan.finish]);assert(r.instructions[0].keys[2].isWritable);
  assert(r.instructions[0].keys[6].isWritable);assert(same(r.instructions[0].keys[2].pubkey,r.instructions[0].keys[6].pubkey));
});

test('packet and bounded-buffer limits reject oversized candidates without signing',()=>{
  const f=t.fixture(),compact=t.compactMessage(f);
  assert.throws(()=>t.bufferPlan(f,Buffer.alloc(t.BUFFER_LIMIT+1)));
  assert.throws(()=>t.bufferPlan(f,compact,0));assert.throws(()=>t.bufferPlan(f,compact,1233));
  const oversized=t.bufferPlan(f,compact,818);
  assert.equal(t.roundtrip(f,[oversized.create]).bytes,1233);
  const limit=t.bufferPlan(f,compact,817);assert.equal(t.roundtrip(f,[limit.create]).bytes,1232);
  const direct=t.compile(f,[t.directCreate(f,compact)]);assert.equal(t.legacyWireSize(direct),1794);
  assert.throws(()=>new t.VersionedTransaction(direct).serialize(),RangeError);
});
