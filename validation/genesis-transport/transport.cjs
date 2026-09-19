'use strict';

// Host-only packet evidence for the Task 2.22 template. No RPC, wallet, signing,
// installation or native initializer is provided. Public fixtures are synthetic.
const assert = require('node:assert/strict');
const { createHash } = require('node:crypto');
const path = require('node:path');
const fs = require('node:fs');
const REPO = path.resolve(__dirname, '../..');
const SDK = path.join(REPO, 'spikes/task-0.4-jito/node_modules/@solana/web3.js');
assert.equal(require(path.join(SDK, 'package.json')).version, '1.98.4');
const { PublicKey, TransactionInstruction, TransactionMessage, VersionedTransaction,
  AddressLookupTableAccount, AddressLookupTableProgram, ComputeBudgetProgram,
  SystemProgram, SYSVAR_INSTRUCTIONS_PUBKEY } = require(SDK);

const PIN = '64af7330413d5c85cbbccfd8c27a05d45b6e666f';
const BASE = '208b7fb4b7f597fe409429c3a7483b12f73a66af';
// This bound is for the pinned legacy/v0 wire formats, not a claim about v1.
const PACKET_LIMIT = 1232;
const BUFFER_LIMIT = 10128;
const SQUADS = new PublicKey('SQDS4ep65T869zMMBKyuUq6aD6EgTu8psMjkvj52pCf');
const TOKEN = new PublicKey('TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA');
const LOADER = new PublicKey('BPFLoaderUpgradeab1e11111111111111111111111');
const JITO_PROGRAM = new PublicKey('SPoo1Ku8WFXoNDMHPsrGSTSG1Y47rzgn41SLUNakuHy');
const JITO_POOL = new PublicKey('Jito4APyf642JPZPx3hGc6WWJ8zPKtRbRs4P815Awbb');
const JITO_MINT = new PublicKey('J1toso1uCk3RLmjorhTtrVwY9HJ7X8V9yYac6Y7kGCPn');
const key = tag => new PublicKey(Buffer.alloc(32, tag));
const hash = bytes => createHash('sha256').update(bytes).digest();
const disc = name => hash(Buffer.from(`global:${name}`)).subarray(0, 8);
const eq = (a, b) => a.equals(b);
const u16 = value => { const b = Buffer.alloc(2); b.writeUInt16LE(value); return b; };
const u32 = value => { const b = Buffer.alloc(4); b.writeUInt32LE(value); return b; };
const u64 = value => { const b = Buffer.alloc(8); b.writeBigUInt64LE(BigInt(value)); return b; };
const vec = bytes => Buffer.concat([u32(bytes.length), bytes]);
const meta = (pubkey, isSigner = false, isWritable = false) => ({ pubkey, isSigner, isWritable });
const pda = (program, seeds) => PublicKey.findProgramAddressSync(seeds, program)[0];
const text = value => Buffer.from(value);

function fixture({ programTag = 217, sameReceiver = false } = {}) {
  const program = key(programTag), creator = key(91), payer = key(222);
  const guardians = Array.from({ length: 6 }, (_, i) => key(91 + i));
  const multisig = pda(SQUADS, [text('multisig'), text('multisig'), key(80).toBuffer()]);
  const transaction = pda(SQUADS, [text('multisig'), multisig.toBuffer(), text('transaction'), u64(19)]);
  const proposal = pda(SQUADS, [text('multisig'), multisig.toBuffer(), text('transaction'), u64(19), text('proposal')]);
  const vault = pda(SQUADS, [text('multisig'), multisig.toBuffer(), text('vault'), Buffer.from([7])]);
  const stateSeeds = ['config', 'distribution', 'guardian-registry'];
  const targets = stateSeeds.map(seed => pda(program, [text(seed)]));
  for (let slot = 0; slot < 6; slot++) targets.push(pda(program,
    [text('guardian-reward'), guardians[5-slot].toBuffer(), u64(0), Buffer.from([slot])]));
  for (const seed of ['pending-sol', 'principal-sol', 'operational-sol', 'distribution-escrow',
    'kif-sol', 'principal-jito-vault', 'pending-jito-vault']) targets.push(pda(program, [text(seed)]));
  const protocol = [JITO_PROGRAM, JITO_POOL, key(201), key(202), JITO_MINT, key(203), key(sameReceiver ? 203 : 204)];
  const roles = [
    ['program', meta(program)], ['program_data', meta(pda(LOADER, [program.toBuffer()]))],
    ['multisig', meta(multisig)], ['proposal', meta(proposal)], ['transaction', meta(transaction)],
    ['vault', meta(vault, true)], ['instructions', meta(SYSVAR_INSTRUCTIONS_PUBKEY)],
    ...targets.map((address, i) => [`target_${i}`, meta(address, false, true)]),
    ...protocol.slice(0, sameReceiver ? 6 : 7).map((address, i) => [`protocol_${i}`, meta(address)]),
    ['payer', meta(payer, true, true)], ['system', meta(SystemProgram.programId)], ['token', meta(TOKEN)],
  ];
  assert.equal(roles.length, sameReceiver ? 32 : 33);
  assert.equal(new Set(roles.map(([, m]) => m.pubkey.toBase58())).size, roles.length);
  assert(!guardians.some(g => eq(g, payer)));
  // Exact existing PIV1GM01 model format, still not a native instruction ABI.
  const data = Buffer.concat([text('PIV1GM01'), Buffer.from([1, 7, 0]),
    ...protocol.map(p => p.toBuffer()), creator.toBuffer(), key(206).toBuffer(),
    u64(0), Buffer.from([5, 4, 3, 2, 1, 0])]);
  assert.equal(data.length, 313);
  const inner = new TransactionInstruction({ programId: program, keys: roles.map(([, m]) => m), data });
  return { program, programTag, sameReceiver, creator, payer, guardians, multisig, proposal,
    transaction, vault, targets, roles, inner, blockhash: key(240).toBase58() };
}

// Squads' compact creation schema uses fixed u8/u16 SmallVec lengths, not
// Solana shortvec and not the u32 Vec lengths of its persisted account schema.
function compactMessage(f) {
  const rank = m => m.isSigner ? (m.isWritable ? 0 : 1) : (m.isWritable ? 2 : 3);
  const keys = [...f.inner.keys].sort((a, b) => rank(a)-rank(b));
  const index = address => keys.findIndex(m => eq(m.pubkey, address));
  const data = Buffer.concat([Buffer.from([
    keys.filter(m => m.isSigner).length, keys.filter(m => m.isSigner && m.isWritable).length,
    keys.filter(m => !m.isSigner && m.isWritable).length, keys.length]),
    ...keys.map(m => m.pubkey.toBuffer()), Buffer.from([1, index(f.program), f.inner.keys.length]),
    Buffer.from(f.inner.keys.map(m => index(m.pubkey))), u16(f.inner.data.length), f.inner.data,
    Buffer.from([0])]);
  assertCompactMatches(f, decodeCompact(data));
  return data;
}

class Reader {
  constructor(bytes) { this.bytes = Buffer.from(bytes); this.offset = 0; }
  take(size) { assert(Number.isSafeInteger(size) && size >= 0 && this.offset + size <= this.bytes.length, 'truncated bytes');
    const value = this.bytes.subarray(this.offset, this.offset+size); this.offset += size; return value; }
  u8() { return this.take(1)[0]; }
  u16() { return this.take(2).readUInt16LE(); }
  u32() { return this.take(4).readUInt32LE(); }
  key() { return new PublicKey(this.take(32)); }
  vec() { return this.take(this.u32()); }
  end() { assert.equal(this.offset, this.bytes.length, 'trailing bytes'); }
}

function decodeCompact(bytes) {
  const r = new Reader(bytes);
  const signers = r.u8(), writableSigners = r.u8(), writableNonSigners = r.u8(), count = r.u8();
  assert(count > 0 && signers <= count && writableSigners <= signers && writableNonSigners <= count-signers);
  const keys = Array.from({ length: count }, (_, i) => meta(r.key(), i < signers,
    i < writableSigners || (i >= signers && i-signers < writableNonSigners)));
  assert.equal(new Set(keys.map(m => m.pubkey.toBase58())).size, count);
  assert.equal(r.u8(), 1, 'single stored instruction');
  const programIndex = r.u8(); assert(programIndex < count);
  const indices = [...r.take(r.u8())]; assert(indices.every(i => i < count));
  assert.equal(new Set(indices).size, indices.length);
  const data = r.take(r.u16()); assert.equal(r.u8(), 0, 'stored message must not use lookup tables'); r.end();
  return { keys, programIndex, indices, data };
}

function assertCompactMatches(f, stored) {
  assert(eq(stored.keys[stored.programIndex].pubkey, f.program));
  assert.deepEqual(stored.data, f.inner.data);
  assert.equal(stored.indices.length, f.inner.keys.length);
  for (const [i, index] of stored.indices.entries()) assertMeta(stored.keys[index], f.inner.keys[i]);
}

function assertMeta(actual, expected) {
  assert(eq(actual.pubkey, expected.pubkey), 'account identity/order mismatch');
  assert.equal(actual.isSigner, expected.isSigner, 'signer privilege mismatch');
  assert.equal(actual.isWritable, expected.isWritable, 'writable privilege mismatch');
}

function executeInstruction(f, stored = decodeCompact(compactMessage(f))) {
  return new TransactionInstruction({ programId: SQUADS, data: disc('vault_transaction_execute'), keys: [
    meta(f.multisig), meta(f.proposal, false, true), meta(f.transaction), meta(f.creator, true),
    ...stored.keys.map(m => meta(m.pubkey, m.isSigner && !eq(m.pubkey, f.vault), m.isWritable)),
  ] });
}

function lookup(f, targetCount = 16) {
  assert(Number.isInteger(targetCount) && targetCount >= 0 && targetCount <= 16);
  // This is a synthetic, uncreated table. No account lookup/creation is executed.
  const [, address] = AddressLookupTableProgram.createLookupTable({ authority: f.creator, payer: f.payer, recentSlot: 42 });
  return new AddressLookupTableAccount({ key: address, state: { deactivationSlot: 0xffffffffffffffffn,
    lastExtendedSlot: 0, lastExtendedSlotStartIndex: 0, authority: f.creator, addresses: f.targets.slice(0, targetCount) } });
}

function compile(f, instructions, tables = null) {
  const builder = new TransactionMessage({ payerKey: f.payer, recentBlockhash: f.blockhash, instructions });
  return tables === null ? builder.compileToLegacyMessage() : builder.compileToV0Message(tables);
}

function unionPrivileges(f, instructions) {
  const values = new Map();
  const add = m => { const name = m.pubkey.toBase58(), old = values.get(name);
    values.set(name, meta(m.pubkey, m.isSigner || !!old?.isSigner, m.isWritable || !!old?.isWritable)); };
  add(meta(f.payer, true, true));
  for (const ix of instructions) { add(meta(ix.programId)); ix.keys.forEach(add); }
  return values;
}

function roundtrip(f, instructions, tables = null) {
  const message = compile(f, instructions, tables);
  const raw = Buffer.from(new VersionedTransaction(message).serialize());
  const decoded = VersionedTransaction.deserialize(raw);
  assert(decoded.signatures.every(s => s.every(byte => byte === 0)), 'only zero signature placeholders are allowed');
  assert.equal(decoded.signatures.length, 2, 'external payer and distinct member must both sign');
  assert.deepEqual(Buffer.from(decoded.serialize()), raw);
  const recovered = TransactionMessage.decompile(decoded.message,
    tables === null ? undefined : { addressLookupTableAccounts: tables });
  assert(eq(recovered.payerKey, f.payer)); assert.equal(recovered.recentBlockhash, f.blockhash);
  assert.equal(recovered.instructions.length, instructions.length);
  const union = unionPrivileges(f, instructions);
  for (const [i, ix] of recovered.instructions.entries()) {
    const original = instructions[i]; assert(eq(ix.programId, original.programId)); assert.deepEqual(ix.data, original.data);
    assert.equal(ix.keys.length, original.keys.length);
    ix.keys.forEach((m, j) => { assert(eq(m.pubkey, original.keys[j].pubkey)); assertMeta(m, union.get(m.pubkey.toBase58())); });
  }
  return { raw, message: decoded.message, instructions: recovered.instructions,
    bytes: raw.length, sha256: hash(raw).toString('hex'), withinLimit: raw.length <= PACKET_LIMIT };
}

function assertExecution(f, stored, recovered) {
  assertCompactMatches(f, stored);
  assert(eq(recovered.programId, SQUADS)); assert.deepEqual(recovered.data, disc('vault_transaction_execute'));
  assert.equal(recovered.keys.length, 4+stored.keys.length);
  [meta(f.multisig),meta(f.proposal,false,true),meta(f.transaction),meta(f.creator,true)]
    .forEach((m,i)=>assertMeta(recovered.keys[i],m));
  for (const [i, requested] of stored.keys.entries()) {
    const outer = recovered.keys[4+i]; assert(eq(outer.pubkey, requested.pubkey));
    assert.equal(outer.isWritable,requested.isWritable || eq(requested.pubkey,f.proposal));
    if (eq(requested.pubkey,f.vault)) assert.equal(outer.isSigner,false,'Squads supplies the inner PDA signature');
    else assert.equal(outer.isSigner,requested.isSigner);
  }
  // CPI metas come from the approved stored message, not outer privilege union:
  // the outer writable proposal is deliberately readonly in the PIV1 invocation.
  assert.equal(stored.keys.find(m => eq(m.pubkey,f.proposal)).isWritable,false);
  const inner = stored.indices.map(i => stored.keys[i]); inner.forEach((m,i) => assertMeta(m,f.inner.keys[i]));
}

function createArgs(message) { return Buffer.concat([Buffer.from([7,0]),vec(message),Buffer.from([0])]); }
function createKeys(f) { return [meta(f.multisig,false,true),meta(f.transaction,false,true),
  meta(f.creator,true),meta(f.payer,true,true),meta(SystemProgram.programId)]; }
function directCreate(f, message) {
  return new TransactionInstruction({ programId:SQUADS,keys:createKeys(f),data:Buffer.concat([disc('vault_transaction_create'),createArgs(message)]) });
}

function bufferPlan(f, message = compactMessage(f), chunkSize = 800) {
  assert(Number.isInteger(chunkSize) && chunkSize > 0 && chunkSize <= PACKET_LIMIT);
  assert(message.length > 0 && message.length <= BUFFER_LIMIT);
  const bufferIndex = 1;
  const address = pda(SQUADS,[text('multisig'),f.multisig.toBuffer(),text('transaction_buffer'),f.creator.toBuffer(),Buffer.from([bufferIndex])]);
  const chunks = []; for(let offset=0;offset<message.length;offset+=chunkSize) chunks.push(message.subarray(offset,offset+chunkSize));
  const create = new TransactionInstruction({ programId:SQUADS,
    keys:[meta(f.multisig),meta(address,false,true),meta(f.creator,true),meta(f.payer,true,true),meta(SystemProgram.programId)],
    data:Buffer.concat([disc('transaction_buffer_create'),Buffer.from([bufferIndex,7]),hash(message),u16(message.length),vec(chunks[0])]) });
  const extend = chunks.slice(1).map(chunk => new TransactionInstruction({ programId:SQUADS,
    keys:[meta(f.multisig),meta(address,false,true),meta(f.creator,true)],data:Buffer.concat([disc('transaction_buffer_extend'),vec(chunk)]) }));
  const finish = new TransactionInstruction({ programId:SQUADS,
    keys:[...createKeys(f),meta(address,false,true),meta(f.creator,true,true)],
    data:Buffer.concat([disc('vault_transaction_create_from_buffer'),createArgs(Buffer.alloc(6))]) });
  return { address,create,extend,finish };
}

function validateBufferPlan(f, plan, expected = compactMessage(f)) {
  const check = (ix,name,keys) => { assert(eq(ix.programId,SQUADS)); assert.equal(ix.keys.length,keys.length);
    ix.keys.forEach((m,i)=>assertMeta(m,keys[i]));const r=new Reader(ix.data);assert.deepEqual(r.take(8),disc(name));return r; };
  assert(f.guardians.some(g=>eq(g,f.creator)), 'synthetic creator is an eligible current member');
  const r=check(plan.create,'transaction_buffer_create',[meta(f.multisig),meta(plan.address,false,true),meta(f.creator,true),meta(f.payer,true,true),meta(SystemProgram.programId)]);
  const index=r.u8();assert.equal(r.u8(),7);const expectedHash=r.take(32);const size=r.u16();assert(size>0 && size<=BUFFER_LIMIT);
  assert(eq(plan.address,pda(SQUADS,[text('multisig'),f.multisig.toBuffer(),text('transaction_buffer'),f.creator.toBuffer(),Buffer.from([index])])));
  const pieces=[r.vec()];r.end();let length=pieces[0].length;assert(length<=size);
  for(const ix of plan.extend) {const e=check(ix,'transaction_buffer_extend',[meta(f.multisig),meta(plan.address,false,true),meta(f.creator,true)]);
    const next=e.vec();e.end();assert(next.length>0 && next.length<=size-length);pieces.push(next);length+=next.length;}
  const end=check(plan.finish,'vault_transaction_create_from_buffer',[...createKeys(f),meta(plan.address,false,true),meta(f.creator,true,true)]);
  assert.equal(end.u8(),7);assert.equal(end.u8(),0);assert.deepEqual(end.vec(),Buffer.alloc(6));assert.equal(end.u8(),0);end.end();
  const assembled=Buffer.concat(pieces);assert.equal(length,size);assert.deepEqual(hash(assembled),expectedHash);
  assert.deepEqual(assembled,expected);assertCompactMatches(f,decodeCompact(assembled));
  return { bytes:size,sha256:hash(assembled).toString('hex'),bufferAccountBytes:112+size,
    refundDestination:'creator (pinned Squads close constraint)',chunks:pieces.map(p=>p.length) };
}

// Count a compiled legacy layout without inventing outer wire bytes when the
// pinned SDK refuses an oversized instruction buffer. Feasible packets below
// always use actual SDK serialization and deserialization.
const shortSize = value => { assert(Number.isInteger(value) && value>=0 && value<=0xffff);
  return value<128?1:value<16384?2:3; };
function legacyWireSize(message) {
  assert.equal(message.version,'legacy');
  const n=message.header.numRequiredSignatures, keys=message.staticAccountKeys, ix=message.compiledInstructions;
  return shortSize(n)+64*n+3+shortSize(keys.length)+32*keys.length+32+shortSize(ix.length)
    +ix.reduce((sum,i)=>sum+1+shortSize(i.accountKeyIndexes.length)+i.accountKeyIndexes.length+shortSize(i.data.length)+i.data.length,0);
}

function measure(f, computePrefix = false) {
  const compact=compactMessage(f),stored=decodeCompact(compact),plan=bufferPlan(f,compact);
  const buffer=validateBufferPlan(f,plan,compact);
  const packets=[plan.create,...plan.extend,plan.finish].map(ix=>roundtrip(f,[ix]));
  assert(packets.every(p=>p.withinLimit));
  const instructions=[...(computePrefix ? [ComputeBudgetProgram.setComputeUnitLimit({units:1_400_000}),
    ComputeBudgetProgram.requestHeapFrame({bytes:256*1024})] : []),executeInstruction(f,stored)];
  const legacy=roundtrip(f,instructions);assert.equal(legacy.bytes,legacyWireSize(compile(f,instructions)));assert(!legacy.withinLimit);
  assertExecution(f,stored,legacy.instructions.at(-1));
  const table=lookup(f),v0=roundtrip(f,instructions,[table]);assert(v0.withinLimit);
  assertExecution(f,stored,v0.instructions.at(-1));assert.equal(v0.message.addressTableLookups.length,1);
  assert.equal(v0.message.addressTableLookups[0].writableIndexes.length,16);
  assert.equal(v0.message.addressTableLookups[0].readonlyIndexes.length,0);
  assert(v0.message.staticAccountKeys.slice(0,2).some(k=>eq(k,f.payer)));
  assert(v0.message.staticAccountKeys.slice(0,2).some(k=>eq(k,f.creator)));
  const creation=compile(f,[directCreate(f,compact)]),directBytes=legacyWireSize(creation);assert(directBytes>PACKET_LIMIT);
  assert.throws(()=>new VersionedTransaction(creation).serialize(),RangeError);
  const summary=p=>({bytes:p.bytes,sha256:p.sha256,within1232:p.withinLimit});
  return {programTag:f.programTag,sharedReceiver:f.sameReceiver,innerAccounts:f.roles.length,innerBytes:f.inner.data.length,
    compactBytes:compact.length,storedLookupCount:0,signatures:2,computePrefix,
    directCreate:{calculatedBytes:directBytes,sdkSerialization:'RangeError: oversized instruction buffer'},buffer,
    bufferPackets:packets.map(summary),legacyExecute:summary(legacy),v0Execute:summary(v0),
    outerLookupCount:1,loadedTargetCount:16,remainingAccountCount:stored.keys.length,fixedAccountCount:4,
    executeInstructionAccountCount:4+stored.keys.length};
}

function report() {
  const cases=[];for(const programTag of [217,211])for(const sameReceiver of [false,true])for(const computePrefix of [false,true])
    cases.push(measure(fixture({programTag,sameReceiver}),computePrefix));
  return {scope:'Unsigned host-only legacy/v0 packet feasibility; no runtime or operational readiness',
    sourceBaseline:BASE,squadsRevision:PIN,web3Version:'1.98.4',node:process.version,packetLimit:PACKET_LIMIT,
    computePrefix:'Illustrative upper sizing values only, not measured or approved compute/heap budgets',
    assumptions:['Synthetic ALT contains all 16 target PDAs; existence/authority/funding/warm-up unresolved',
      'External payer and current guardian are distinct; both signatures are zero placeholders',
      'Proposal creation, approvals, live recipients, cluster/artifact and runtime remain unproven'],cases};
}

module.exports={fixture,compactMessage,decodeCompact,assertCompactMatches,executeInstruction,lookup,compile,
  roundtrip,assertExecution,bufferPlan,validateBufferPlan,directCreate,legacyWireSize,measure,report,
  PACKET_LIMIT,BUFFER_LIMIT,PIN,BASE,SDK,REPO,PublicKey,TransactionMessage,VersionedTransaction,hash,disc};
if(require.main===module) {
  const locked=JSON.parse(fs.readFileSync(path.join(REPO,'spikes/task-0.4-jito/package-lock.json')));
  assert.equal(locked.packages['node_modules/@solana/web3.js'].version,'1.98.4');
  process.stdout.write(`${JSON.stringify(report(),null,2)}\n`);
}
