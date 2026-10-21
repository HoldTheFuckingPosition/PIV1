'use strict';
// No key generation or signing. Positive signatures are immutable consumed D-047 evidence.
const test=require('node:test'),assert=require('node:assert/strict'),fs=require('node:fs'),path=require('node:path');
const {createHash}=require('node:crypto');
const REPO=process.env.PIV1_PUBLIC_CHECK_REPO||path.resolve(__dirname,'../..');
const c=require('./startup-public-check.cjs');
const {PublicKey,VersionedTransaction,AddressLookupTableAccount}=require(path.join(REPO,'spikes/task-0.4-jito/node_modules/@solana/web3.js'));
const sha=b=>createHash('sha256').update(b).digest('hex'),clone=x=>JSON.parse(JSON.stringify(x));
const HIST=readPinned(path.join(__dirname,'startup-public-check.fixture.json'),'0855cd85667a565f4b6184ef2b389c2e4ec4421582f2b52f58b2cf8897852c0d');
function readPinned(file,want){const raw=fs.readFileSync(file);assert.equal(sha(raw),want);return JSON.parse(raw);}
function reviewedHistorical(t){
  const wire=Buffer.from(t.signedWireBase64,'base64');assert.equal(wire[0],t.signatureCount);
  const message=wire.subarray(1+64*t.signatureCount),n=message[3];assert(n<128);
  const keys=Array.from({length:n},(_,i)=>new PublicKey(message.subarray(4+32*i,36+32*i)).toBase58());
  const accountRoles=keys.map((address,i)=>({address,signer:i<message[0],writable:i<message[0]?i<message[0]-message[1]:i<n-message[2]}));
  return {signedWireBase64:t.signedWireBase64,reviewed:{messageBase64:message.toString('base64'),messageSha256:t.messageSha256,feePayer:keys[0],requiredSigners:keys.slice(0,message[0]),accountRoles,lookupTables:[]}};
}
function fixture(){
  const t=HIST.transactions[0],packet=reviewedHistorical(t),d=c.decodeWire(t.signedWireBase64);
  const r=HIST.fundPoststate,accounts=clone(r.accounts);
  const frozen={packageSha256:'a'.repeat(64),payloadSha256:'b'.repeat(64),anchorUnixTimestamp:'0',transactionIndex:'1'};
  const p={id:'HISTORICAL_D047_FUND_NEVER_REPLAY',packet,lifetime:{recentBlockhash:d.recentBlockhash,currentBlockHeight:'18446744073709551500',lastValidBlockHeight:'18446744073709551615',minimumRemainingBlockHeights:'40'},feeQuoteLamports:t.feeLamports,feeCapLamports:t.feeLamports,expectedPoststate:accounts};
  const manifest={schema:c.SCHEMA,frozen,cumulativeFeeCapLamports:'25000',phases:[p],trace:[]};
  const observation={status:{signatureHex:d.firstSignatureHex,slot:t.slot,confirmationStatus:'finalized',err:null},transaction:{slot:t.slot,wireBase64:t.signedWireBase64,feeLamports:r.feeLamports,err:null},poststate:{contextSlot:r.contextSlot,accounts:clone(accounts)}};
  return {manifest,p,d,observation};
}
function append(m,state,observation=null,phase=m.phases[0].id){
  const p=m.phases.find(p=>p.id===phase),d=c.decodeWire(p.packet.signedWireBase64,p.packet.reviewed.lookupTables);
  m.trace.push({previousDigest:m.trace.length?c.entryDigest(m.trace.at(-1)):null,phase,state,packetSha256:d.wireSha256,signatureHex:d.firstSignatureHex,frozen:clone(m.frozen),observation});return m;
}
function finalized(){const f=fixture();append(f.manifest,'PREPARED');append(f.manifest,'SUBMISSION_START_RECORDED');append(f.manifest,'FINALIZED',f.observation);return f;}
function stop(m,reason){const r=c.check(m);assert.equal(r.status,'STOP');if(reason)assert.equal(r.reason,reason);assert.equal(r.liveReady,false);return r;}

test('source pins; nullable input remains incomplete and never live ready',()=>{
  assert.equal(c.verifySources().sdk,'1.98.4');assert.deepEqual(HIST.provenance.map(p=>p.sha256),['bee322a30105eb515a10338b6a054f176cc24a48d6e441abe147839f16748a9b','055420c666857b71f60f75cf657b6cc7682af6890f46f6aa19a86ed61f7be72c','3bf393b5036a8706db859438ea1f2edf1613954a8fe3424ec06362266655522b']);const r=c.check(c.TEMPLATE);assert.equal(r.status,'INCOMPLETE');assert.equal(r.liveReady,false);
  assert.throws(()=>c.parseInput('{"schema":1,"\\u0073chema":2}'),/duplicate_json_key/);
});

test('all three historical packets and all five existing public signatures verify',()=>{
  let count=0;for(const t of HIST.transactions){const d=c.verifyPacket(reviewedHistorical(t));assert.equal(d.wireSha256,t.signedWireSha256);assert.equal(d.messageSha256,t.messageSha256);assert.equal(d.signatures.length,t.signatureCount);count+=d.signatures.length;}assert.equal(count,5);
  const {manifest}=finalized(),r=c.check(manifest);assert.equal(r.status,'VERIFIED_PUBLIC_DECLARATIONS_ONLY');assert.deepEqual(r.completedPhases,[manifest.phases[0].id]);assert.equal(r.actualFeeLamports,'5000');assert.equal(r.authorized,false);
});

test('generated signature-bit corruptions cannot preserve actual signature validity',()=>{
  for(const t of HIST.transactions){for(let signer=0;signer<t.signatureCount;signer++)for(const byte of [0,1,15,31,32,47,62,63])for(const bit of [1,128]){
    const p=reviewedHistorical(t),b=Buffer.from(p.signedWireBase64,'base64');b[1+64*signer+byte]^=bit;p.signedWireBase64=b.toString('base64');assert.throws(()=>c.verifyPacket(p),/signature_invalid/);
  }}
  const p=reviewedHistorical(HIST.transactions[1]),b=Buffer.from(p.signedWireBase64,'base64');b.fill(0,1,65);p.signedWireBase64=b.toString('base64');assert.throws(()=>c.verifyPacket(p),/zero_signature/);
});

test('message, signer order, payer, roles, extra/missing signatures and noncanonical base64 fail',()=>{
  const t=HIST.transactions[1];for(const mutate of [
    p=>p.reviewed.requiredSigners.reverse(),p=>p.reviewed.feePayer=p.reviewed.requiredSigners[1],
    p=>p.reviewed.accountRoles[0].writable=false,p=>p.reviewed.messageSha256='0'.repeat(64),
    p=>p.reviewed.messageBase64=Buffer.from('different').toString('base64'),p=>p.signedWireBase64+='\n',
    p=>{const b=Buffer.from(p.signedWireBase64,'base64');b[0]=1;p.signedWireBase64=b.toString('base64');},
    p=>{const b=Buffer.from(p.signedWireBase64,'base64');b[0]=3;p.signedWireBase64=b.toString('base64');},
    p=>{const b=Buffer.from(p.signedWireBase64,'base64');b[b.length-1]^=1;p.signedWireBase64=b.toString('base64');}
  ]){const p=reviewedHistorical(t);mutate(p);assert.throws(()=>c.verifyPacket(p));}
});

test('exhaustive u16 shortvec roundtrip constrained by consumed cursor; overlong/overflow fail',()=>{
  // Wire prefix counts 1..127 must consume exactly one byte; replacing with an
  // equivalent overlong shortvec must always fail before cryptography.
  const raw=Buffer.from(HIST.transactions[0].signedWireBase64,'base64');
  for(let n=1;n<=127;n++){const b=Buffer.from(raw);b[0]=n;if(n===1)assert.equal(c.decodeWire(b.toString('base64')).signatures.length,1);else assert.throws(()=>c.decodeWire(b.toString('base64')));}
  for(let n=0;n<=65535;n++){const encoded=c.shortvec(n);let value=0;for(let i=0;i<encoded.length;i++)value+=(encoded[i]&127)*2**(7*i);assert.equal(value,n);assert.equal(encoded.length,n<128?1:n<16384?2:3);}
  for(const prefix of [[129,0],[129,128,0],[255,255,4],[128,128,128],[128]])assert.throws(()=>c.decodeWire(Buffer.concat([Buffer.from(prefix),raw.subarray(1)]).toString('base64')));
});

test('every truncation and bounded trailing extension is rejected structurally',()=>{
  const raw=Buffer.from(HIST.transactions[1].signedWireBase64,'base64');
  for(let i=0;i<raw.length;i++)assert.throws(()=>c.decodeWire(raw.subarray(0,i).toString('base64')));
  for(let size=1;size<=32;size++)assert.throws(()=>c.decodeWire(Buffer.concat([raw,Buffer.alloc(size)]).toString('base64')),/trailing_bytes/);
});

test('retained unsigned v0 initializer resolves actual ALT roles; zero signatures cannot verify',()=>{
  const p=HIST.unsignedV0,alt=p.alt,tables=[{address:alt.address,addresses:alt.addresses}];
  const d=c.decodeWire(p.wireBase64,tables);assert.equal(d.version,0);assert.equal(d.fullyResolved,true);assert.deepEqual(d.requiredSigners,p.requiredSigners);assert.equal(d.roles.length,new Set(d.roles.map(v=>v.address)).size);
  // Independent pinned SDK is a structural oracle; it does not sign.
  const v=VersionedTransaction.deserialize(d.wire);assert.deepEqual(Buffer.from(v.serialize()),d.wire);
  const originalTables=[new AddressLookupTableAccount({key:new PublicKey(alt.address),state:{deactivationSlot:BigInt(alt.deactivationSlot),lastExtendedSlot:Number(alt.lastExtendedSlot),lastExtendedSlotStartIndex:alt.lastExtendedSlotStartIndex,authority:new PublicKey(alt.authority),addresses:alt.addresses.map(a=>new PublicKey(a))}})];
  const keys=v.message.getAccountKeys({addressLookupTableAccounts:originalTables});
  for(let i=0;i<d.roles.length;i++){assert.equal(keys.get(i).toBase58(),d.roles[i].address);assert.equal(v.message.isAccountSigner(i),d.roles[i].signer);assert.equal(v.message.isAccountWritable(i),d.roles[i].writable);}
  assert.equal(c.decodeWire(p.wireBase64).fullyResolved,false);
  const packet={signedWireBase64:p.wireBase64,reviewed:{messageBase64:d.message.toString('base64'),messageSha256:d.messageSha256,feePayer:d.feePayer,requiredSigners:d.requiredSigners,accountRoles:d.roles,lookupTables:tables}};
  assert.throws(()=>c.verifyPacket(packet),/zero_signature/);
  const bad=clone(tables);bad[0].addresses=[];assert.throws(()=>c.decodeWire(p.wireBase64,bad),/lookup_index/);
  // Actual real-signed v0 positive fixture is unavailable and is not claimed.
});

test('UNKNOWN, null status, expiry and dependent/replayed phases never advance',()=>{
  const f=fixture(),m=f.manifest;append(m,'PREPARED');append(m,'SUBMISSION_START_RECORDED');append(m,'UNKNOWN');stop(m,'PENDING_OR_UNKNOWN_NO_REPLACEMENT');
  const nullStatus=clone(m),o=clone(f.observation);o.status=null;append(nullStatus,'FINALIZED',o);stop(nullStatus,'status_unknown');
  const repeat=clone(m);append(repeat,'SUBMISSION_START_RECORDED');stop(repeat,'journal_transition');
  const expired=clone(m);expired.phases[0].lifetime.currentBlockHeight='18446744073709551615';stop(expired,'lifetime_margin');
  const dependent=clone(m),next=clone(dependent.phases[0]);next.id='DEPENDENT';next.packet=reviewedHistorical(HIST.transactions[1]);next.lifetime.recentBlockhash=c.decodeWire(next.packet.signedWireBase64).recentBlockhash;dependent.phases.push(next);append(dependent,'PREPARED',null,next.id);stop(dependent,'dependent_or_replayed_phase');
  const final=finalized();append(final.manifest,'PREPARED');stop(final.manifest,'dependent_or_replayed_phase');
  // A positive resolution of UNKNOWN is allowed only by the complete receipt.
  append(m,'FINALIZED',f.observation);assert.equal(c.check(m).status,'VERIFIED_PUBLIC_DECLARATIONS_ONLY');
});

test('frozen payload/Clock/index, digest, packet and signature tampering fail closed',()=>{
  for(const field of ['packageSha256','payloadSha256','anchorUnixTimestamp','transactionIndex']){const {manifest}=finalized();manifest.trace[1].frozen[field]=field.endsWith('Sha256')?'c'.repeat(64):'2';stop(manifest,'frozen_fields_changed');}
  for(const field of ['previousDigest','packetSha256','signatureHex']){const {manifest}=finalized();manifest.trace[1][field]='0'.repeat(64);stop(manifest);}
  for(const value of ['-1','01','18446744073709551616',1,1.5,null]){const {manifest}=fixture();manifest.phases[0].lifetime.lastValidBlockHeight=value;stop(manifest,'u64_string');}
  assert.equal(c.uint('18446744073709551615'),(1n<<64n)-1n);
  const overflow=fixture();overflow.manifest.frozen.anchorUnixTimestamp='9223372036854775808';stop(overflow.manifest,'anchor_i64_overflow');
  const upper=fixture();upper.manifest.frozen.anchorUnixTimestamp='9223372036854775807';assert.equal(c.check(upper.manifest).reason,'PENDING_OR_UNKNOWN_NO_REPLACEMENT');
  const preAnchor=finalized();preAnchor.manifest.frozen.anchorUnixTimestamp=null;for(const e of preAnchor.manifest.trace)e.frozen.anchorUnixTimestamp=null;for(let i=1;i<preAnchor.manifest.trace.length;i++)preAnchor.manifest.trace[i].previousDigest=c.entryDigest(preAnchor.manifest.trace[i-1]);assert.equal(c.check(preAnchor.manifest).status,'VERIFIED_PUBLIC_DECLARATIONS_ONLY');
  preAnchor.manifest.trace[2].frozen.anchorUnixTimestamp='1';stop(preAnchor.manifest,'frozen_fields_changed');
});

test('fees, complete raw poststate/slot, finalized errors and transaction wire bind advancement',()=>{
  for(const mutate of [
    m=>m.cumulativeFeeCapLamports='4999',m=>m.phases[0].feeCapLamports='4999',
    m=>m.trace[2].observation.transaction.feeLamports='5001',m=>m.trace[2].observation.transaction.feeLamports='-1',
    m=>m.trace[2].observation.transaction.wireBase64=HIST.transactions[1].signedWireBase64,
    m=>m.trace[2].observation.status.confirmationStatus='confirmed',m=>m.trace[2].observation.status.signatureHex='0'.repeat(128),
    m=>m.trace[2].observation.poststate.contextSlot='0',m=>m.trace[2].observation.transaction.slot='1',
    m=>delete m.trace[2].observation.poststate.accounts[Object.keys(m.phases[0].expectedPoststate)[0]],
    m=>m.trace[2].observation.poststate.accounts[Object.keys(m.phases[0].expectedPoststate)[0]].lamports='1',
    m=>m.phases[0].expectedPoststate={},m=>m.trace[2].observation.status.err={InstructionError:[0,'Custom']}
  ]){const {manifest}=finalized();mutate(manifest);stop(manifest);}
  const f=fixture();append(f.manifest,'PREPARED');append(f.manifest,'SUBMISSION_START_RECORDED');const o=clone(f.observation);o.status.err=o.transaction.err={InstructionError:[0,'Custom']};o.poststate=null;append(f.manifest,'FAILED',o);const r=stop(f.manifest,'FINALIZED_FAILURE');assert.equal(r.actualFeeLamports,'5000');assert.deepEqual(r.completedPhases,[]);
});


test('duplicate packet under another phase and cumulative u64 overflow are rejected',()=>{
  const f=finalized(),p=clone(f.p);p.id='SECOND';f.manifest.phases.push(p);stop(f.manifest,'packet_replay');
  p.packet=reviewedHistorical(HIST.transactions[1]);p.lifetime.recentBlockhash=c.decodeWire(p.packet.signedWireBase64).recentBlockhash;
  p.feeQuoteLamports=p.feeCapLamports='18446744073709551615';f.manifest.cumulativeFeeCapLamports='18446744073709551615';
  append(f.manifest,'PREPARED',null,p.id);append(f.manifest,'SUBMISSION_START_RECORDED',null,p.id);stop(f.manifest,'cumulative_fee_cap');
});


test('program-index zero and empty v0 lookup descriptor violate structural sanitize',()=>{
  const payer=new PublicKey(HIST.transactions[1]&&reviewedHistorical(HIST.transactions[1]).reviewed.feePayer).toBuffer();
  const program=new PublicKey('11111111111111111111111111111111').toBuffer();
  const blockhash=new PublicKey(c.decodeWire(HIST.transactions[0].signedWireBase64).recentBlockhash).toBuffer();
  function minimal(version,index,emptyLookup=false){
    const header=Buffer.from([1,0,1,2]),instruction=Buffer.from([1,index,0,0]);
    const lookups=emptyLookup?Buffer.concat([Buffer.from([1]),new PublicKey(HIST.unsignedV0.alt.address).toBuffer(),Buffer.from([0,0])]):Buffer.from([0]);
    return Buffer.concat([Buffer.from([1]),Buffer.alloc(64),...(version===0?[Buffer.from([128])]:[]),header,payer,program,blockhash,instruction,...(version===0?[lookups]:[])]).toString('base64');
  }
  for(const version of ['legacy',0]){assert.equal(c.decodeWire(minimal(version,1)).version,version);assert.throws(()=>c.decodeWire(minimal(version,0)),/program_index/);}
  assert.throws(()=>c.decodeWire(minimal(0,1,true)),/empty_lookup/);
});

function twoPhases(){
  const f=finalized(),p=clone(f.p),t=HIST.transactions[1];p.id='LATER_HISTORICAL_PACKET';p.packet=reviewedHistorical(t);p.lifetime.recentBlockhash=c.decodeWire(p.packet.signedWireBase64).recentBlockhash;p.feeQuoteLamports=p.feeCapLamports=t.feeLamports;f.manifest.phases.push(p);
  const d=c.decodeWire(t.signedWireBase64),o=clone(f.observation);o.status.signatureHex=d.firstSignatureHex;o.status.slot=o.transaction.slot=t.slot;o.transaction.wireBase64=t.signedWireBase64;o.transaction.feeLamports=t.feeLamports;o.poststate.contextSlot=t.slot;
  // Chronology examples use a synthetic declared subset for phase2; no claim
  // that phase1's balances were phase2's actual historical balances.
  append(f.manifest,'PREPARED',null,p.id);append(f.manifest,'SUBMISSION_START_RECORDED',null,p.id);append(f.manifest,'FINALIZED',o,p.id);return f;
}

test('sequential finalized phase slots must strictly increase, including equal-slot STOP',()=>{
  const f=twoPhases();assert.equal(c.check(f.manifest).status,'VERIFIED_PUBLIC_DECLARATIONS_ONLY');
  for(const slot of [HIST.transactions[0].slot,(BigInt(HIST.transactions[0].slot)-1n).toString()]){const m=clone(f.manifest),o=m.trace.at(-1).observation;o.status.slot=o.transaction.slot=o.poststate.contextSlot=slot;stop(m,'phase_slot_not_strictly_later');}
});
