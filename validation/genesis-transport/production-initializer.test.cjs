'use strict';
const test=require('node:test');
const assert=require('node:assert/strict');
const n=require('./production-initializer.cjs');
const t=require('./transport.cjs');
const {PublicKey,TransactionInstruction,TransactionMessage}=require(t.SDK);
const key=i=>new PublicKey(Buffer.alloc(32,i));
const same=(a,b)=>a.equals(b);
const derive=(program,seeds)=>PublicKey.findProgramAddressSync(seeds,program)[0];
const bytes=s=>Buffer.from(s);
const squads=new PublicKey('SQDS4ep65T869zMMBKyuUq6aD6EgTu8psMjkvj52pCf');
const txIndex=Buffer.from([19,0,0,0,0,0,0,0]);
const copyIx=ix=>new TransactionInstruction({programId:ix.programId,data:Buffer.from(ix.data),keys:ix.keys.map(m=>({...m}))});

// Independent fixed layout: no production encoder or historical fixture used.
function literal({programTag=217,shared=false,paused=false,witnesses=[0,255],permutation=[5,4,3,2,1,0],anchor=0n}={}) {
  const program=key(programTag),multisig=derive(squads,[bytes('multisig'),bytes('multisig'),key(80).toBuffer()]);
  const vault=i=>derive(squads,[bytes('multisig'),multisig.toBuffer(),bytes('vault'),Buffer.from([i])]);
  const protocol=[new PublicKey('SPoo1Ku8WFXoNDMHPsrGSTSG1Y47rzgn41SLUNakuHy'),
    new PublicKey('Jito4APyf642JPZPx3hGc6WWJ8zPKtRbRs4P815Awbb'),key(201),key(202),
    new PublicKey('J1toso1uCk3RLmjorhTtrVwY9HJ7X8V9yYac6Y7kGCPn'),key(203),key(shared?203:204)];
  const recipient=witnesses.map(vault),data=Buffer.alloc(315);
  data.write('PIV1IN01');data[8]=1;data[9]=7;data[10]=Number(paused);
  [...protocol,...recipient].forEach((p,i)=>p.toBuffer().copy(data,11+32*i));
  data.writeBigInt64LE(anchor,299);Buffer.from(permutation).copy(data,307);data[313]=witnesses[0];data[314]=witnesses[1];
  const target=['config','distribution','guardian-registry'].map(s=>derive(program,[bytes(s)]));
  for(let slot=0;slot<6;slot++)target.push(derive(program,[bytes('guardian-reward'),key(91+permutation[slot]).toBuffer(),Buffer.alloc(8),Buffer.from([slot])]));
  target.push(...['pending-sol','principal-sol','operational-sol','distribution-escrow','kif-sol','principal-jito-vault','pending-jito-vault'].map(s=>derive(program,[bytes(s)])));
  const accounts=[program,derive(new PublicKey('BPFLoaderUpgradeab1e11111111111111111111111'),[program.toBuffer()]),multisig,
    derive(squads,[bytes('multisig'),multisig.toBuffer(),bytes('transaction'),txIndex,bytes('proposal')]),
    derive(squads,[bytes('multisig'),multisig.toBuffer(),bytes('transaction'),txIndex]),vault(7),
    new PublicKey('Sysvar1nstructions1111111111111111111111111'),...target,...protocol.slice(0,shared?6:7),key(222),
    new PublicKey('11111111111111111111111111111111'),new PublicKey('TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA'),...recipient];
  return {data,accounts,target};
}

test('source binding preserves current baseline and historical tool/test bytes',()=>{
  n.verifySources();assert.equal(n.BASELINE,'21cd68e5c02def1a9620320e126e6a3dcd7fa013');
  assert.equal(t.hash(Buffer.from(`${JSON.stringify(t.report(),null,2)}\n`)).toString('hex'),
    '5eb9f7cac5b1acd5b715e6dd12a9340406429e353bf39d1e9e97ab79ac87b771','historical default report unchanged');
  assert.equal(n.SOURCE_PINS['validation/genesis-transport/transport.cjs'],'a7694ff07577988e96546a53008bc8ab85bbc0ae7cb9353dcfc25e6c5a3c3724');
  assert.equal(n.SOURCE_PINS['validation/genesis-transport/transport.test.cjs'],'626294af1840e82e438242d7dcd56150a3f64f30862b6f9daeb56e9f82175856');
});

test('current315-byte native data, exact account order and inner privileges match independent literal oracle',()=>{
  for(const programTag of [217,211])for(const sameReceiver of [false,true])for(const initiallyPaused of [false,true]) {
    const f=n.fixture({programTag,sameReceiver,initiallyPaused});
    const oracle=literal({programTag,shared:sameReceiver,paused:initiallyPaused}),count=sameReceiver?34:35;
    assert.deepEqual(f.inner.data,oracle.data);assert.equal(f.inner.keys.length,count);
    assert.deepEqual(f.inner.keys.map(m=>m.pubkey.toBase58()),oracle.accounts.map(p=>p.toBase58()));
    f.inner.keys.forEach((m,i)=>{assert.equal(m.isSigner,i===5||i===count-5);assert.equal(m.isWritable,(i>=7&&i<=22)||i===count-5);});
    const stored=t.decodeCompact(n.compactMessage(f));n.assertStored(f,stored);
    assert.deepEqual(stored.data,oracle.data);assert.deepEqual(stored.indices.map(i=>stored.keys[i].pubkey.toBase58()),oracle.accounts.map(p=>p.toBase58()));
    assert.deepEqual(stored.keys.filter(m=>m.isSigner).map(m=>m.pubkey.toBase58()),[key(222),oracle.accounts[5]].map(p=>p.toBase58()));
    assert(!oracle.accounts.some(p=>same(p,key(91))),'outer member is absent from inner native accounts');
  }
});

function* permutations(left,prefix=[]) {if(!left.length){yield prefix;return;}for(const x of left)yield* permutations(left.filter(i=>i!==x),[...prefix,x]);}
test('deterministic whole permutation domain and signed timestamp boundaries preserve exact offsets',()=>{
  let count=0;
  for(const permutation of permutations([0,1,2,3,4,5])) {
    const wire=literal({permutation}).data,p=n.decodeNative(wire);
    assert.deepEqual(p.permutation,permutation);assert.equal(p.anchor,0n);assert.deepEqual(p.witnesses,[0,255]);count++;
  }
  assert.equal(count,720);
  for(const anchor of [-(1n<<63n),-1n,0n,1n,(1n<<63n)-1n])for(const permutation of [[0,1,2,3,4,5],[2,0,5,1,4,3],[5,4,3,2,1,0]]) {
    const f=n.fixture({anchor,permutation}),oracle=literal({anchor,permutation});
    assert.deepEqual(f.inner.data,oracle.data);assert.equal(n.decodeNative(f.inner.data).anchor,anchor);
    assert.deepEqual(f.targets.map(p=>p.toBase58()),oracle.target.map(p=>p.toBase58()));
  }
});

test('every u8 recipient witness decodes; selected boundary pairs derive exact PDAs and reject aliases',()=>{
  for(let value=0;value<256;value++)for(const offset of [313,314]) {
    const data=literal().data;data[offset]=value;assert.equal(n.decodeNative(data).witnesses[offset-313],value);
  }
  for(const witnesses of [[0,255],[255,0],[31,202],[1,254],[127,128],[6,8]]) {
    const f=n.fixture({witnesses});assert.deepEqual(f.inner.data,literal({witnesses}).data);n.assertStored(f,t.decodeCompact(n.compactMessage(f)));
  }
  for(const witnesses of [[7,255],[0,7],[0,0],[255,255],[-1,255],[0,256],[0,1.1]])assert.throws(()=>n.fixture({witnesses}));
});

test('strict native decoder rejects all truncations, extra data, old model domain and malformed fields',()=>{
  const wire=literal().data;
  for(let end=0;end<315;end++)assert.throws(()=>n.decodeNative(wire.subarray(0,end)));
  assert.throws(()=>n.decodeNative(Buffer.concat([wire,Buffer.from([0])])));
  for(const change of [b=>b.write('PIV1GM01'),b=>b[8]=0,b=>b[8]=2,b=>b[10]=2,b=>b[307]=6,b=>b[308]=b[307]]) {
    const bad=Buffer.from(wire);change(bad);assert.throws(()=>n.decodeNative(bad));
  }
  assert.throws(()=>n.fixture({programTag:212}));assert.throws(()=>n.fixture({initiallyPaused:1}));
  for(const permutation of [[0,1,2,3,4,4],[0,1,2,3,4,6],[0,1,2,3,4]])assert.throws(()=>n.fixture({permutation}));
});

test('native template rejects identities, privileges, witness/key disagreement and unsupported governance vault index',()=>{
  for(const sameReceiver of [false,true]) {
    const count=sameReceiver?34:35;
    for(const index of [0,1,2,3,4,5,6,7,15,22,23,24,25,26,27,28,count-5,count-4,count-3,count-2,count-1])for(const field of ['pubkey','isSigner','isWritable']) {
      const f=n.fixture({sameReceiver});f.inner.keys[index][field]=field==='pubkey'?key(230):!f.inner.keys[index][field];
      assert.throws(()=>n.compactMessage(f),`${index}/${field}`);
    }
    for(const offset of [9,11,43,75,107,139,171,203,235,266,267,298,313,314]) {
      const f=n.fixture({sameReceiver});f.inner.data[offset]^=1;assert.throws(()=>n.compactMessage(f),`byte${offset}`);
    }
    const f=n.fixture({sameReceiver});f.inner.keys.pop();assert.throws(()=>n.compactMessage(f));
    const old=t.fixture({sameReceiver,profile:'recipient-checked'});assert.throws(()=>n.assertStored(n.fixture({sameReceiver}),t.decodeCompact(t.compactMessage(old))));
    const selfConsistent=n.fixture({sameReceiver});key(230).toBuffer().copy(selfConsistent.inner.data,235);selfConsistent.inner.keys.at(-2).pubkey=key(230);
    assert.throws(()=>n.compactMessage(selfConsistent),/native recipient witness/);
    const guardianPayer=n.fixture({sameReceiver});guardianPayer.payer=key(91);guardianPayer.inner.keys[count-5].pubkey=key(91);assert.throws(()=>n.compactMessage(guardianPayer));
  }
});

test('native packets include both signatures, exact prefix overhead, buffer chunks and adjacent ALT threshold',()=>{
  const report=n.report();assert.equal(report.cases.length,16);
  for(const c of report.cases) {
    const shared=c.sharedReceiver,overhead=c.computePrefix?48:0;
    assert.equal(c.nativeBytes,315);assert.equal(c.innerAccounts,shared?34:35);assert.equal(c.compactBytes,shared?1447:1480);
    assert.equal(c.outerSignatures,2);assert.equal(c.signatureBytes,128);assert.equal(c.storedLookupCount,0);
    assert.equal(c.directLegacyCreate.calculatedBytes,shared?1829:1862);
    assert.deepEqual(c.buffer.chunks,[800,shared?647:680]);assert.equal(c.buffer.bufferAccountBytes,112+c.compactBytes);
    assert.deepEqual(c.bufferPackets.map(p=>p.bytes),[1215,shared?992:1025,421]);assert(c.bufferPackets.every(p=>p.within1232));
    assert.equal(c.legacyExecute.bytes,(shared?1367:1400)+overhead);assert(!c.legacyExecute.within1232);
    assert.equal(c.v0Execute.bytes,(shared?907:940)+overhead);assert(c.v0Execute.within1232);
    assert.equal(c.proposalCreate.bytes,383);assert.deepEqual(c.proposalApprovals.map(p=>p.bytes),[341,341,341,341]);
    const minimum=(shared?6:7)+(c.computePrefix?2:0),base=(shared?1403:1436)+overhead;
    assert.deepEqual(c.lookupThreshold.lastOversized,{loadedTargets:minimum-1,bytes:base-31*(minimum-1)});
    assert.deepEqual(c.lookupThreshold.firstWithin1232,{loadedTargets:minimum,bytes:base-31*minimum});
  }
});

test('literal pinned proposal create and four approval plans preserve complete metas and bytes',()=>{
  const f=n.fixture(),p=n.governancePlan(f);
  // Independent Anchor discriminators from pinned IDL instruction names.
  assert.deepEqual([...p.create.data],[220,60,73,224,30,108,79,159,19,0,0,0,0,0,0,0,0]);
  assert.deepEqual(p.create.keys.map(m=>[m.pubkey.toBase58(),m.isSigner,m.isWritable]),[
    [f.multisig.toBase58(),false,false],[f.proposal.toBase58(),false,true],[key(91).toBase58(),true,false],
    [key(222).toBase58(),true,true],['11111111111111111111111111111111',false,false]]);
  assert.deepEqual(p.approve.map(a=>a.member.toBase58()),[91,92,93,94].map(i=>key(i).toBase58()));
  for(const {member,instruction:ix} of p.approve) {
    assert.deepEqual([...ix.data],[144,37,164,136,188,216,42,248,0]);
    assert.deepEqual(ix.keys.map(m=>[m.pubkey.toBase58(),m.isSigner,m.isWritable]),[
      [f.multisig.toBase58(),false,false],[member.toBase58(),true,true],[f.proposal.toBase58(),false,true]]);
  }
});

test('actual v0 packets reconstruct stored native data and exact privilege union; wrong ALT/outer metas reject',()=>{
  const f=n.fixture(),stored=t.decodeCompact(n.compactMessage(f)),packet=t.roundtrip(f,[n.executeInstruction(f,stored)],[t.lookup(f)]);
  assert.deepEqual(packet.message.staticAccountKeys.slice(0,2).map(p=>p.toBase58()),[key(222),key(91)].map(p=>p.toBase58()));
  assert.equal(packet.message.header.numRequiredSignatures,2);assert(new t.VersionedTransaction(packet.message).signatures.every(s=>s.length===64&&s.every(b=>b===0)));
  const ix=packet.instructions[0];n.assertExecution(f,stored,ix);
  assert(!stored.keys.find(m=>same(m.pubkey,f.proposal)).isWritable);assert(ix.keys.filter(m=>same(m.pubkey,f.proposal)).every(m=>m.isWritable));
  assert(!ix.keys.find(m=>same(m.pubkey,f.vault)).isSigner);
  for(const address of [f.payer,f.creator,f.vault,f.targets[0],f.recipients[0]])for(const field of ['isSigner','isWritable']) {
    const bad=copyIx(ix),m=bad.keys.find(m=>same(m.pubkey,address));m[field]=!m[field];assert.throws(()=>n.assertExecution(f,stored,bad));
  }
  const wrong=t.lookup(f);wrong.state.addresses.reverse();
  assert.throws(()=>n.assertExecution(f,stored,TransactionMessage.decompile(packet.message,{addressLookupTableAccounts:[wrong]}).instructions[0]));
  const missing=t.lookup(f,15);assert.throws(()=>TransactionMessage.decompile(packet.message,{addressLookupTableAccounts:[missing]}));
  const renamed=t.lookup(f);renamed.key=key(230);assert.throws(()=>TransactionMessage.decompile(packet.message,{addressLookupTableAccounts:[renamed]}));
});

test('native buffer commitment cannot substitute model bytes or a self-consistent tampered recipient witness',()=>{
  for(const sameReceiver of [false,true]) {
    const f=n.fixture({sameReceiver}),compact=n.compactMessage(f),plan=n.bufferPlan(f);n.validateBufferPlan(f,plan);
    const altered=Buffer.from(compact),payload=9+33*f.inner.keys.length;altered[payload+313]^=1;
    assert.throws(()=>n.bufferPlan(f,altered));
    const tampered=n.bufferPlan(f);tampered.extend[0].data[12+payload+313-800]^=1;t.hash(altered).copy(tampered.create.data,10);
    assert.throws(()=>n.validateBufferPlan(f,tampered));
    assert.throws(()=>n.validateBufferPlan(f,t.bufferPlan(t.fixture({sameReceiver,profile:'recipient-checked'}))));
    assert.equal(t.roundtrip(f,[n.bufferPlan(f,compact,817).create]).bytes,1232);
    assert.equal(t.roundtrip(f,[n.bufferPlan(f,compact,818).create]).bytes,1233);
  }
});
