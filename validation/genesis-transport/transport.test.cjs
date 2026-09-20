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

test('explicit recipient profile preserves the exact historical CLI report and rejects unknown selection',()=>{
  assert.equal(t.hash(Buffer.from(`${JSON.stringify(t.report(),null,2)}\n`)).toString('hex'),
    '5eb9f7cac5b1acd5b715e6dd12a9340406429e353bf39d1e9e97ab79ac87b771');
  assert.equal(t.selectProfile([]),'historical');
  assert.equal(t.selectProfile(['--recipient-checked']),'recipient-checked');
  for(const args of [['unknown'],['--recipient-checked','extra'],['--profile','unknown'],['--recipient-checked','--recipient-checked']])
    assert.throws(()=>t.selectProfile(args),/unknown genesis profile/);
  assert.throws(()=>t.fixture({profile:'unknown'}),/unknown genesis profile/);
  assert.equal(t.RECIPIENT_BASE,'648998b4f5767eadf14c511d1dd0034ffff29ee0');
  assert.equal(Object.keys(t.RECIPIENT_SOURCE_PINS).length,4);
  for(const [file,expected] of Object.entries(t.RECIPIENT_SOURCE_PINS))
    assert.equal(t.hash(fs.readFileSync(path.join(t.REPO,file))).toString('hex'),expected,file);
});

test('recipient topology and payload match independent literal Task 2.26 seeds, order and byte offsets',()=>{
  const pub=tag=>new t.PublicKey(Buffer.alloc(32,tag)),b=word=>Buffer.from(word);
  const squads=new t.PublicKey('SQDS4ep65T869zMMBKyuUq6aD6EgTu8psMjkvj52pCf');
  const derive=(seeds,program=squads)=>t.PublicKey.findProgramAddressSync(seeds,program)[0];
  const multisig=derive([b('multisig'),b('multisig'),pub(80).toBuffer()]);
  const transactionSeed=Buffer.from([19,0,0,0,0,0,0,0]);
  const recipients=[0,255].map(index=>derive([b('multisig'),multisig.toBuffer(),b('vault'),Buffer.from([index])]));
  for(const programTag of [217,211])for(const sameReceiver of [false,true])for(const initiallyPaused of [false,true]) {
    const f=t.fixture({programTag,sameReceiver,initiallyPaused,profile:'recipient-checked'}),program=pub(programTag);
    const targets=['config','distribution','guardian-registry'].map(seed=>derive([b(seed)],program));
    for(let slot=0;slot<6;slot++)targets.push(derive([b('guardian-reward'),pub(96-slot).toBuffer(),Buffer.alloc(8),Buffer.from([slot])],program));
    targets.push(...['pending-sol','principal-sol','operational-sol','distribution-escrow','kif-sol',
      'principal-jito-vault','pending-jito-vault'].map(seed=>derive([b(seed)],program)));
    const protocol=[new t.PublicKey('SPoo1Ku8WFXoNDMHPsrGSTSG1Y47rzgn41SLUNakuHy'),
      new t.PublicKey('Jito4APyf642JPZPx3hGc6WWJ8zPKtRbRs4P815Awbb'),pub(201),pub(202),
      new t.PublicKey('J1toso1uCk3RLmjorhTtrVwY9HJ7X8V9yYac6Y7kGCPn'),pub(203),pub(sameReceiver?203:204)];
    const expected=[program,
      derive([program.toBuffer()],new t.PublicKey('BPFLoaderUpgradeab1e11111111111111111111111')),multisig,
      derive([b('multisig'),multisig.toBuffer(),b('transaction'),transactionSeed,b('proposal')]),
      derive([b('multisig'),multisig.toBuffer(),b('transaction'),transactionSeed]),
      derive([b('multisig'),multisig.toBuffer(),b('vault'),Buffer.from([7])]),
      new t.PublicKey('Sysvar1nstructions1111111111111111111111111'),...targets,
      ...protocol.slice(0,sameReceiver?6:7),pub(222),new t.PublicKey('11111111111111111111111111111111'),
      new t.PublicKey('TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA'),...recipients];
    assert.equal(expected.length,sameReceiver?34:35);
    assert.deepEqual(f.inner.keys.map(m=>m.pubkey.toBase58()),expected.map(k=>k.toBase58()));
    expected.forEach((address,index)=>{
      assert.equal(f.inner.keys[index].isSigner,index===5 || index===expected.length-5);
      assert.equal(f.inner.keys[index].isWritable,(index>=7 && index<23) || index===expected.length-5);
    });
    const payload=Buffer.alloc(313);payload.write('PIV1GM01');payload[8]=1;payload[9]=7;payload[10]=Number(initiallyPaused);
    protocol.forEach((address,index)=>address.toBuffer().copy(payload,11+32*index));
    recipients[0].toBuffer().copy(payload,235);recipients[1].toBuffer().copy(payload,267);
    Buffer.from([5,4,3,2,1,0]).copy(payload,307);
    assert.deepEqual(f.inner.data,payload);
    const stored=t.decodeCompact(t.compactMessage(f));
    assert.deepEqual(stored.indices.map(index=>stored.keys[index].pubkey.toBase58()),expected.map(k=>k.toBase58()));
    // A separate outer guardian executor remains absent from the inner list.
    assert(!expected.some(address=>same(address,pub(91))));
    const result=t.roundtrip(f,[t.executeInstruction(f,stored)],[t.lookup(f)]);
    assert.deepEqual(result.message.staticAccountKeys.slice(0,2).map(k=>k.toBase58()),[pub(222),pub(91)].map(k=>k.toBase58()));
    assert.equal(result.message.header.numRequiredSignatures,2);
    assert(new t.VersionedTransaction(result.message).signatures.every(sig=>sig.every(byte=>byte===0)));
    const outer=result.instructions[0];t.assertExecution(f,stored,outer);
    for(const address of recipients) {
      assert(result.message.staticAccountKeys.some(k=>same(k,address)));
      const recovered=outer.keys.filter(m=>same(m.pubkey,address));assert.equal(recovered.length,1);
      assert.equal(recovered[0].isSigner,false);assert.equal(recovered[0].isWritable,false);
    }
    assert.equal(outer.keys.find(m=>same(m.pubkey,expected[5])).isSigner,false);
    assert.equal(result.message.addressTableLookups[0].readonlyIndexes.length,0);
  }
});

test('sixteen recipient cases measure real packets, buffer uploads and neighboring ALT thresholds',()=>{
  const report=t.recipientCheckedReport();assert.equal(report.cases.length,16);
  assert.equal(report.profile,'recipient-checked');assert.equal(report.sourceBaseline,t.RECIPIENT_BASE);
  const identities=new Set();
  for(const c of report.cases) {
    identities.add([c.programTag,c.sharedReceiver,c.initiallyPaused,c.computePrefix].join('/'));
    const n=c.sharedReceiver?34:35,overhead=c.computePrefix?48:0;
    assert.equal(c.innerAccounts,n);assert.equal(c.innerBytes,313);assert.equal(c.compactBytes,c.sharedReceiver?1445:1478);
    assert.deepEqual(c.recipientAccountIndices,[n-2,n-1]);assert.deepEqual(c.recipientVaultIndexWitnesses,[0,255]);
    assert.equal(c.remainingAccountCount,n);assert.equal(c.fixedAccountCount,4);assert.equal(c.executeInstructionAccountCount,n+4);
    assert.equal(c.storedLookupCount,0);assert.equal(c.outerLookupCount,1);assert.equal(c.loadedTargetCount,16);assert.equal(c.signatures,2);
    assert.equal(c.directCreate.calculatedBytes,c.sharedReceiver?1827:1860);
    assert.equal(c.directCreate.sdkSerialization,'RangeError: oversized instruction buffer');
    assert.deepEqual(c.buffer.chunks,[800,c.sharedReceiver?645:678]);assert.equal(c.buffer.bufferAccountBytes,112+c.compactBytes);
    assert.deepEqual(c.bufferPackets.map(p=>p.bytes),[1215,c.sharedReceiver?990:1023,421]);
    assert(c.bufferPackets.every(p=>p.within1232));
    assert.equal(c.legacyExecute.bytes,(c.sharedReceiver?1367:1400)+overhead);assert(!c.legacyExecute.within1232);
    assert.equal(c.v0Execute.bytes,(c.sharedReceiver?907:940)+overhead);assert(c.v0Execute.within1232);
    const minimum=(c.sharedReceiver?6:7)+(c.computePrefix?2:0),base=(c.sharedReceiver?1403:1436)+overhead;
    assert.deepEqual(c.lookupThreshold.firstWithin1232,{loadedTargets:minimum,bytes:base-31*minimum});
    assert.deepEqual(c.lookupThreshold.lastOversized,{loadedTargets:minimum-1,bytes:base-31*(minimum-1)});
    assert(c.lookupThreshold.lastOversized.bytes>1232);assert(c.lookupThreshold.firstWithin1232.bytes<=1232);
    assert(c.lookupThreshold.sdkRefusedCounts.every(count=>count<minimum-1));
  }
  assert.equal(identities.size,16);
});

test('missing, swapped, substituted or privilege-altered recipient metas and approved keys are rejected',()=>{
  for(const sameReceiver of [false,true]) {
    const f=t.fixture({sameReceiver,profile:'recipient-checked'}),bytes=t.compactMessage(f),stored=t.decodeCompact(bytes);
    const good=t.roundtrip(f,[t.executeInstruction(f,stored)],[t.lookup(f)]).instructions[0];
    const indices=stored.indices.slice(-2),outerIndices=indices.map(index=>4+index);
    for(const slot of [0,1])for(const mutation of ['missing','substituted','signer','writable']) {
      const s=t.decodeCompact(bytes),ix=copyIx(good),index=indices[slot],outer=outerIndices[slot];
      if(mutation==='missing'){s.indices.splice(s.indices.length-2+slot,1);ix.keys.splice(outer,1);}
      if(mutation==='substituted'){s.keys[index].pubkey=f.payer;ix.keys[outer].pubkey=f.payer;}
      if(mutation==='signer'){s.keys[index].isSigner=true;ix.keys[outer].isSigner=true;}
      if(mutation==='writable'){s.keys[index].isWritable=true;ix.keys[outer].isWritable=true;}
      assert.throws(()=>t.assertCompactMatches(f,s),`${slot}/${mutation}/stored`);
      assert.throws(()=>t.assertExecution(f,stored,ix),`${slot}/${mutation}/outer`);
    }
    const swapped=t.decodeCompact(bytes);[swapped.indices[swapped.indices.length-2],swapped.indices[swapped.indices.length-1]]=swapped.indices.slice(-2).reverse();
    assert.throws(()=>t.assertCompactMatches(f,swapped));
    const outer=copyIx(good);[outer.keys[outerIndices[0]],outer.keys[outerIndices[1]]]=[outer.keys[outerIndices[1]],outer.keys[outerIndices[0]]];
    assert.throws(()=>t.assertExecution(f,stored,outer));
    for(let offset=235;offset<299;offset++) {
      const changed=t.decodeCompact(bytes);changed.data=Buffer.from(changed.data);changed.data[offset]^=1;
      assert.throws(()=>t.assertCompactMatches(f,changed));
    }
    // Self-consistent replacement still fails the fixed canonical recipient witness.
    const replaced=t.fixture({sameReceiver,profile:'recipient-checked'});
    const unrelated=new t.PublicKey(Buffer.alloc(32,231));
    assert(!replaced.inner.keys.some(m=>same(m.pubkey,unrelated)));
    replaced.inner.keys.at(-2).pubkey=unrelated;unrelated.toBuffer().copy(replaced.inner.data,235);
    assert.throws(()=>t.compactMessage(replaced),/account identity\/order mismatch/);
    const old=t.fixture({sameReceiver});
    assert.throws(()=>t.assertCompactMatches(f,t.decodeCompact(t.compactMessage(old))));
    assert.throws(()=>t.assertExecution(f,stored,t.roundtrip(old,[t.executeInstruction(old)],[t.lookup(old)]).instructions[0]));
  }
});

test('recipient profile rejects altered outer lookup identities and target reconstruction',()=>{
  const f=t.fixture({profile:'recipient-checked'}),stored=t.decodeCompact(t.compactMessage(f));
  const result=t.roundtrip(f,[t.executeInstruction(f,stored)],[t.lookup(f)]);
  for(const slot of [0,1]) {
    const wrong=t.lookup(f);wrong.state.addresses[slot]=f.recipients[slot];
    const ix=t.TransactionMessage.decompile(result.message,{addressLookupTableAccounts:[wrong]}).instructions[0];
    assert.throws(()=>t.assertExecution(f,stored,ix));
  }
  const reordered=t.lookup(f);reordered.state.addresses.reverse();
  assert.throws(()=>t.assertExecution(f,stored,t.TransactionMessage.decompile(result.message,{addressLookupTableAccounts:[reordered]}).instructions[0]));
  assert.throws(()=>t.TransactionMessage.decompile(result.message,{addressLookupTableAccounts:[t.lookup(f,15)]}));
  const renamed=t.lookup(f);renamed.key=f.recipients[0];
  assert.throws(()=>t.TransactionMessage.decompile(result.message,{addressLookupTableAccounts:[renamed]}));
});

test('recipient payload and topology stay bound through buffer hashing, assembly and compact parsing',()=>{
  for(const sameReceiver of [false,true]) {
    const f=t.fixture({sameReceiver,profile:'recipient-checked'}),compact=t.compactMessage(f),plan=t.bufferPlan(f);
    const payloadStart=9+33*f.inner.keys.length;
    for(const offset of [235,266,267,298]) {
      const altered=Buffer.from(compact);altered[payloadStart+offset]^=1;
      const changed=copyPlan(plan);changed.extend[0].data[12+payloadStart+offset-800]^=1;
      assert.throws(()=>t.validateBufferPlan(f,changed));
      t.hash(altered).copy(changed.create.data,10);
      // A matching attacker-supplied hash cannot replace the approved message.
      assert.throws(()=>t.validateBufferPlan(f,changed));
      assert.throws(()=>t.validateBufferPlan(f,t.bufferPlan(f,altered),altered));
    }
    assert.throws(()=>t.validateBufferPlan(f,t.bufferPlan(t.fixture({sameReceiver}))));
    for(const end of [4,4+32*f.roles.length,payloadStart+235,payloadStart+299,compact.length-1])
      assert.throws(()=>t.decodeCompact(compact.subarray(0,end)));
    const chunkBoundary=t.bufferPlan(f,compact,817),tooLarge=t.bufferPlan(f,compact,818);
    assert.equal(t.roundtrip(f,[chunkBoundary.create]).bytes,1232);
    assert.equal(t.roundtrip(f,[tooLarge.create]).bytes,1233);
  }
});
