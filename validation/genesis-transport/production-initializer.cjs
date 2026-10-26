'use strict';

// Offline current-native ABI and unsigned packet preparation. No RPC or signing.
const assert = require('node:assert/strict');
const fs = require('node:fs');
const path = require('node:path');
const t = require('./transport.cjs');
const { PublicKey, TransactionInstruction, ComputeBudgetProgram, SystemProgram } = require(t.SDK);
const BASELINE = '21cd68e5c02def1a9620320e126e6a3dcd7fa013';
const SOURCE_PINS = Object.freeze({
  'programs/piv1/src/instructions/initialize.rs': '718cb1cb92a01101ab25660a2d6155ab1051d207be06079e20d85940422aafe5',
  'programs/piv1/src/instruction_boundary.rs': '0e082adeebe7cabe5f5e14f69b244c2f1464c3c2f4eacd0077173a9aad91ab9f',
  'programs/piv1/src/genesis_allocation.rs': 'd7878a592ad670396a9158e7cd5c1edc31f390302af373ac16d8cf69b56e968e',
  'programs/piv1/src/genesis_recipients.rs': '2f543c0f60ce431efbcae202fe4098ab8ca918a018b7c831a5d86b9a405f9d17',
  'programs/piv1/src/squads_execution.rs': '589252b8357d76e9af61d4f4aaed7e7d54692db63bddd98ea807cee6357321fd',
  'validation/genesis-bank-runtime/tests/genesis_fixture.rs': '506116fc7b8065c0879281972457765960ade509cf61a0689e8339dce11e461a',
  'validation/genesis-transport/transport.cjs': 'a7694ff07577988e96546a53008bc8ab85bbc0ae7cb9353dcfc25e6c5a3c3724',
  'validation/genesis-transport/transport.test.cjs': '626294af1840e82e438242d7dcd56150a3f64f30862b6f9daeb56e9f82175856',
});
// Selected schema read independently from the retained IDL at this pinned revision.
// No dependency on /tmp at execution: the schema is literal and reviewed here.
const GOVERNANCE_SCHEMA = Object.freeze({ revision: t.PIN,
  idlSha256: 'cb9a0a29040ec3853a5105c547ffc608bb0e3175d78dacf30d463aff939f9b9b',
  path: 'sdk/multisig/idl/squads_multisig_program.json',
  proposalCreate: { accounts: [['multisig',false,false],['proposal',false,true],['creator',true,false],
    ['rentPayer',true,true],['systemProgram',false,false]], fields: ['transactionIndex:u64','draft:bool'] },
  proposalApprove: { accounts: [['multisig',false,false],['member',true,true],['proposal',false,true]],
    fields: ['memo:Option<String>'] },
});
const same = (a,b) => a.equals(b);
const key = tag => new PublicKey(Buffer.alloc(32,tag));
const meta = (pubkey,isSigner=false,isWritable=false) => ({pubkey,isSigner,isWritable});
const SQUADS = new PublicKey('SQDS4ep65T869zMMBKyuUq6aD6EgTu8psMjkvj52pCf');
const derive = (program,seeds) => PublicKey.findProgramAddressSync(seeds,program)[0];
const text = s => Buffer.from(s);
const u64 = n => {const b=Buffer.alloc(8);b.writeBigUInt64LE(BigInt(n));return b;};
const vault = (multisig,index) => derive(SQUADS,[text('multisig'),multisig.toBuffer(),text('vault'),Buffer.from([index])]);
const assertMeta = (a,b) => {
  assert(same(a.pubkey,b.pubkey),'account identity/order mismatch');
  assert.equal(a.isSigner,b.isSigner,'signer privilege mismatch');
  assert.equal(a.isWritable,b.isWritable,'writable privilege mismatch');
};

function decodeNative(bytes) {
  assert(Buffer.isBuffer(bytes) && bytes.length===313,'native length must be 313');
  assert.equal(bytes.subarray(0,8).toString(),'PIV1IN02','native selector');
  assert.equal(bytes[8],2,'native version');assert(bytes[10]===0 || bytes[10]===1,'native boolean');
  const permutation=[...bytes.subarray(307,313)];
  assert(permutation.every(i=>i<6) && new Set(permutation).size===6,'guardian permutation');
  return { vaultIndex:bytes[9],initiallyPaused:!!bytes[10],
    protocol:Array.from({length:7},(_,i)=>new PublicKey(bytes.subarray(11+32*i,43+32*i))),
    recipients:[new PublicKey(bytes.subarray(235,267)),new PublicKey(bytes.subarray(267,299))],
    anchor:bytes.readBigInt64LE(299),permutation };
}

function fixture({programTag=217,sameReceiver=false,initiallyPaused=false,
  witnesses=[0,255],permutation=[5,4,3,2,1,0],anchor=0n}={}) {
  assert([217,211].includes(programTag),'only fixed synthetic program identities');
  assert.equal(typeof sameReceiver,'boolean');assert.equal(typeof initiallyPaused,'boolean');
  assert(Array.isArray(witnesses) && witnesses.length===2 && witnesses.every(i=>Number.isInteger(i)&&i>=0&&i<=255),'u8 witnesses');
  assert(Array.isArray(permutation) && permutation.length===6 && permutation.every(i=>Number.isInteger(i)&&i>=0&&i<6)
    && new Set(permutation).size===6,'guardian permutation');
  const f=t.fixture({programTag,sameReceiver,initiallyPaused,profile:'recipient-checked'});
  f.profile='production-initializer';f.witnesses=[...witnesses];f.permutation=[...permutation];
  f.recipients=witnesses.map(i=>vault(f.multisig,i));
  f.targets=f.targets.map((address,i)=>i>=3&&i<9 ? derive(f.program,[text('guardian-reward'),
    f.guardians[permutation[i-3]].toBuffer(),Buffer.alloc(8),Buffer.from([i-3])]) : address);
  f.roles=f.roles.map(([name,m],i)=>[name,{...m,pubkey:i>=7&&i<23?f.targets[i-7]:i>=f.roles.length-2?f.recipients[i-f.roles.length+2]:m.pubkey}]);
  const data=Buffer.from(f.inner.data);data.write('PIV1IN02');data[8]=2;
  f.recipients.forEach((p,i)=>p.toBuffer().copy(data,235+32*i));
  data.writeBigInt64LE(BigInt(anchor),299);Buffer.from(permutation).copy(data,307);
  f.inner=new TransactionInstruction({programId:f.program,keys:f.roles.map(([,m])=>m),data});
  assertNativeTemplate(f);return f;
}

function assertNativeTemplate(f) {
  const p=decodeNative(f.inner.data),n=p.protocol[5].equals(p.protocol[6])?34:35;
  assert.equal(p.vaultIndex,7,'bounded fixture uses vault 7');assert.equal(f.inner.keys.length,n,'native role count');
  assert.equal(f.sameReceiver,n===34,'receiver profile');assert.equal(f.initiallyPaused,p.initiallyPaused,'pause profile');
assert.deepEqual(f.permutation,p.permutation,'permutation profile');
  assert(same(f.inner.programId,f.program),'native program');
  assert(same(f.program,key(f.programTag)) && [217,211].includes(f.programTag),'synthetic program');
  assert(same(f.payer,key(222)) && same(f.creator,key(91)),'fixed distinct external payer/member');
  assert.deepEqual(f.guardians.map(k=>k.toBase58()),Array.from({length:6},(_,i)=>key(91+i).toBase58()));
  const multisig=derive(SQUADS,[text('multisig'),text('multisig'),key(80).toBuffer()]);
  const targets=['config','distribution','guardian-registry'].map(s=>derive(f.program,[text(s)]));
  for(let slot=0;slot<6;slot++)targets.push(derive(f.program,[text('guardian-reward'),key(91+p.permutation[slot]).toBuffer(),Buffer.alloc(8),Buffer.from([slot])]));
  targets.push(...['pending-sol','principal-sol','operational-sol','distribution-escrow','kif-sol','principal-jito-vault','pending-jito-vault'].map(s=>derive(f.program,[text(s)])));
  const protocol=[new PublicKey('SPoo1Ku8WFXoNDMHPsrGSTSG1Y47rzgn41SLUNakuHy'),
    new PublicKey('Jito4APyf642JPZPx3hGc6WWJ8zPKtRbRs4P815Awbb'),key(201),key(202),
    new PublicKey('J1toso1uCk3RLmjorhTtrVwY9HJ7X8V9yYac6Y7kGCPn'),key(203),key(n===34?203:204)];
  assert.deepEqual(p.protocol.map(k=>k.toBase58()),protocol.map(k=>k.toBase58()),'synthetic protocol');
  const recipients=f.recipients;
  assert.deepEqual(p.recipients.map(k=>k.toBase58()),recipients.map(k=>k.toBase58()),'native recipient witness');
  const addresses=[f.program,derive(new PublicKey('BPFLoaderUpgradeab1e11111111111111111111111'),[f.program.toBuffer()]),multisig,
    derive(SQUADS,[text('multisig'),multisig.toBuffer(),text('transaction'),u64(19),text('proposal')]),
    derive(SQUADS,[text('multisig'),multisig.toBuffer(),text('transaction'),u64(19)]),vault(multisig,7),
    new PublicKey('Sysvar1nstructions1111111111111111111111111'),...targets,...protocol.slice(0,n===34?6:7),
    key(222),SystemProgram.programId,new PublicKey('TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA'),...recipients];
  assert.equal(new Set(addresses.map(k=>k.toBase58())).size,n,'native role alias/collision');
  addresses.forEach((pubkey,i)=>assertMeta(f.inner.keys[i],meta(pubkey,i===5||i===n-5,(i>=7&&i<23)||i===n-5)));
  for(const [name,address] of [['multisig',multisig],['proposal',addresses[3]],['transaction',addresses[4]],['vault',addresses[5]]])assert(same(f[name],address),`${name} metadata`);
  assert.deepEqual(f.targets.map(k=>k.toBase58()),targets.map(k=>k.toBase58()),'lookup targets');
  return p;
}

function compactMessage(f) {assertNativeTemplate(f);return t.compactMessage(f);}
function assertStored(f,stored) {assertNativeTemplate(f);t.assertCompactMatches(f,stored);}
function bufferPlan(f,message=compactMessage(f),chunkSize=800) {assertStored(f,t.decodeCompact(message));return t.bufferPlan(f,message,chunkSize);}
function validateBufferPlan(f,plan,message=compactMessage(f)) {assertStored(f,t.decodeCompact(message));return t.validateBufferPlan(f,plan,message);}
function executeInstruction(f,stored=t.decodeCompact(compactMessage(f))) {assertStored(f,stored);return t.executeInstruction(f,stored);}
function assertExecution(f,stored,ix) {assertStored(f,stored);t.assertExecution(f,stored,ix);}

function governancePlan(f) {
  assertNativeTemplate(f);
  const create=new TransactionInstruction({programId:SQUADS,keys:[meta(f.multisig),meta(f.proposal,false,true),
    meta(f.creator,true),meta(f.payer,true,true),meta(SystemProgram.programId)],
    data:Buffer.concat([t.disc('proposal_create'),u64(19),Buffer.from([0])])});
  // Explicit Active proposal, then four distinct approval plans. This does not
  // execute status transitions, authenticate permissions, consume or replay it.
  const approve=f.guardians.slice(0,4).map(member=>({member,instruction:new TransactionInstruction({programId:SQUADS,
    keys:[meta(f.multisig),meta(member,true,true),meta(f.proposal,false,true)],
    data:Buffer.concat([t.disc('proposal_approve'),Buffer.from([0])])})}));
  return {create,approve};
}
const summary=p=>({bytes:p.bytes,sha256:p.sha256,within1232:p.withinLimit});
function measure(f,computePrefix=false) {
  assertNativeTemplate(f);const compact=compactMessage(f),stored=t.decodeCompact(compact),plan=bufferPlan(f,compact);
  const buffer=validateBufferPlan(f,plan,compact),packets=[plan.create,...plan.extend,plan.finish].map(ix=>t.roundtrip(f,[ix]));
  assert(packets.every(p=>p.withinLimit));
  const governance=governancePlan(f),proposalCreate=t.roundtrip(f,[governance.create]);
  const approvals=governance.approve.map(({member,instruction})=>t.roundtrip({...f,creator:member},[instruction]));
  assert(proposalCreate.withinLimit&&approvals.every(p=>p.withinLimit));
  const instructions=[...(computePrefix?[ComputeBudgetProgram.setComputeUnitLimit({units:1_400_000}),
    ComputeBudgetProgram.requestHeapFrame({bytes:262144})]:[]),executeInstruction(f,stored)];
  const legacy=t.roundtrip(f,instructions),v0=t.roundtrip(f,instructions,[t.lookup(f)]);
  assertExecution(f,stored,legacy.instructions.at(-1));assertExecution(f,stored,v0.instructions.at(-1));
  assert(!legacy.withinLimit&&v0.withinLimit);assert.equal(v0.message.addressTableLookups.length,1);
  assert.deepEqual([...v0.message.addressTableLookups[0].writableIndexes],Array.from({length:16},(_,i)=>i));
  assert.equal(v0.message.addressTableLookups[0].readonlyIndexes.length,0);
  const direct=t.compile(f,[t.directCreate(f,compact)]),directBytes=t.legacyWireSize(direct);
  assert(directBytes>t.PACKET_LIMIT);assert.throws(()=>new t.VersionedTransaction(direct).serialize(),RangeError);
  return {programTag:f.programTag,sharedReceiver:f.sameReceiver,initiallyPaused:f.initiallyPaused,computePrefix,
    nativeBytes:313,innerAccounts:f.inner.keys.length,compactBytes:compact.length,witnesses:[...f.witnesses],
    innerSigners:['external payer','Squads vault PDA'],outerSignatures:2,signatureBytes:128,storedLookupCount:0,
    directLegacyCreate:{calculatedBytes:directBytes,sdkSerialization:'RangeError: oversized instruction buffer'},
    buffer,bufferPackets:packets.map(summary),proposalCreate:summary(proposalCreate),proposalApprovals:approvals.map(summary),
    legacyExecute:summary(legacy),v0Execute:summary(v0),outerLoadedTargets:16,
    lookupThreshold:lookupThreshold(f,stored,instructions)};
}
function lookupThreshold(f,stored,instructions) {
  let lastOversized;const sdkRefusedCounts=[];
  for(let count=0;count<=16;count++) {
    let packet;try{packet=t.roundtrip(f,instructions,[t.lookup(f,count)]);}catch(e){if(!(e instanceof RangeError))throw e;sdkRefusedCounts.push(count);continue;}
    assertExecution(f,stored,packet.instructions.at(-1));const sample={loadedTargets:count,bytes:packet.bytes};
    if(packet.withinLimit){assert(lastOversized&&lastOversized.loadedTargets===count-1);return {sdkRefusedCounts,lastOversized,firstWithin1232:sample};}
    lastOversized=sample;
  }
  assert.fail('no fitting native v0 packet');
}
function verifySources() {
  assert.equal(process.version,'v24.19.0','retained Node version');
  for(const [file,pin] of Object.entries(SOURCE_PINS))assert.equal(t.hash(fs.readFileSync(path.join(t.REPO,file))).toString('hex'),pin,`source binding: ${file}`);
  const lock=JSON.parse(fs.readFileSync(path.join(t.REPO,'spikes/task-0.4-jito/package-lock.json')));
  assert.equal(lock.packages['node_modules/@solana/web3.js'].version,'1.98.4');
}
function report() {
  verifySources();const cases=[];
  for(const programTag of [217,211])for(const sameReceiver of [false,true])for(const initiallyPaused of [false,true])for(const prefix of [false,true])cases.push(measure(fixture({programTag,sameReceiver,initiallyPaused}),prefix));
  return {scope:'Offline native initializer/governance wire preparation only; public package NOT READY',
    sourceBaseline:BASELINE,sourcePins:SOURCE_PINS,governanceSchema:GOVERNANCE_SCHEMA,node:process.version,web3Version:'1.98.4',packetLimit:1232,
    limitations:['Synthetic identities/ALT and zero signatures only; no selected wallet, Program ID, RPC or Squads runtime',
      'Fee payer and inner rent payer are the same synthetic key222; executor is guardian91: two outer zero signatures. AB used separate fee payer242 and three signatures; this is a different transport composition',
      'External payer must be distinct from every guardian, recipient and PIV authority under current allocation guard',
      'Proposal active-create and four approvals are schema plans; status/timelock/permissions/consumption remain unproved',
      'Illustrative 1,400,000 CU / 262,144-byte heap prefix measures packet overhead, not whole-governance runtime resources',
      'ALT creation, authority, funding, warm-up and lookup lifecycle remain open; stored Squads message has no lookups',
      'M4 evidence reservation remains open; no deployment or live-operation clearance'],cases};
}
module.exports={BASELINE,SOURCE_PINS,GOVERNANCE_SCHEMA,fixture,decodeNative,assertNativeTemplate,compactMessage,assertStored,
  bufferPlan,validateBufferPlan,executeInstruction,assertExecution,governancePlan,measure,verifySources,report};
if(require.main===module){assert.equal(process.argv.length,2,'no operational or profile arguments accepted');process.stdout.write(`${JSON.stringify(report(),null,2)}\n`);}
