'use strict';
const test=require('node:test'),assert=require('node:assert/strict');
const h=require('./hybrid-instructions.cjs');
const {PublicKey}=require('./initializer-plan.cjs');
const address=n=>new PublicKey(Buffer.alloc(32,n)).toBase58(); // public test bytes, no keys
const fixture=()=>({cluster:'testnet',programId:h.PROGRAM,sequence:'0',market:address(239),
  protocol:{validatorList:address(231),reserve:address(232),managerFee:address(233),referrer:address(234)},
  currentGuardians:{registryRevision:'0',guardianKeys:[101,102,103,104,105,106].map(address)}});
test('source-bound helper builds only the exact unsigned same-Program instruction',()=>{
  h.verifySources();const result=h.prepareInstruction(fixture());
  assert.equal(result.instruction.programId.toBase58(),h.PROGRAM);assert.equal(result.instruction.data.toString('hex'),'504956314a53303101');
  assert.equal(result.roles.length,39);assert(result.instruction.keys.every(k=>!k.isSigner));
  assert.deepEqual(result.instruction.keys.map((k,i)=>k.isWritable?i:null).filter(i=>i!==null),[0,1,2,3,4,5,7,27,28,29,34,35,36,37,38]);
  assert.deepEqual(result.roles.slice(27).map(r=>r.role),['swap_authority','swap_input','swap_output','wsol_mint','jupiter_program','jupiter_event_authority','manifest_program','manifest_market','manifest_base_vault','manifest_quote_vault','manifest_quote_global','manifest_quote_global_vault']);
  assert.equal(result.roles[31].address,h.JUPITER);assert.equal(result.roles[33].address,h.MANIFEST);
});
test('shared fee receiver and sequence changes preserve canonical isolation',()=>{
  const m=fixture();m.protocol.referrer=m.protocol.managerFee;const a=h.prepareInstruction(m);assert.equal(a.roles.length,38);
  const old=a.roles.slice(26,29).map(r=>r.address);m.sequence='1';const next=h.prepareInstruction(m).roles.slice(26,29).map(r=>r.address);
  assert(next.every((key,i)=>key!==old[i]));assert.equal(new Set([...old,...next]).size,6);
});
test('economic overrides, arbitrary routing bytes, cluster and identity changes reject',()=>{
  for(const extra of [{slippageBps:1000},{minimumOutput:'0'},{quotedOutput:'100'},{routeInstructions:[]},{amount:'1'}])assert.throws(()=>h.prepareInstruction({...fixture(),...extra}));
  assert.throws(()=>h.prepareInstruction({...fixture(),cluster:'mainnet-beta'}));
  assert.throws(()=>h.prepareInstruction({...fixture(),programId:address(217)}));
  for(const sequence of ['-1','01','18446744073709551616'])assert.throws(()=>h.prepareInstruction({...fixture(),sequence}));
  const aliased=fixture();aliased.market=h.JUPITER;assert.throws(()=>h.prepareInstruction(aliased));
});
test('migration has only exact current fixed roles and no signer or economics',()=>{
  const ix=h.migrationInstruction({cluster:'testnet',programId:h.PROGRAM});assert.equal(ix.data.toString('hex'),'504956314d52303201');
  assert.equal(ix.keys.length,10);assert(ix.keys.every(k=>!k.isSigner));
  assert.deepEqual(ix.keys.map((k,i)=>k.isWritable?i:null).filter(i=>i!==null),[1,4]);
  assert.throws(()=>h.migrationInstruction({cluster:'testnet',programId:h.PROGRAM,amount:'1'}));
});
test('V1 and V2 decoders keep exact allocations and factual distinct swap receipt',()=>{
  const header=Buffer.from([104,51,125,187,226,55,209,99]);
  const old=Buffer.alloc(891);header.copy(old);old[8]=1;old[10]=1;
  const legacy=h.decodeDistribution(old);assert.equal(legacy.value.version,1);assert(!Object.hasOwn(legacy.value,'swap'));
  const current=Buffer.alloc(924);old.copy(current);current[8]=2;
  assert.equal(h.decodeDistribution(current).value.swap,null);
  current[11]=2;current[803]=1;[1n,20n,19n,19n].forEach((v,i)=>current.writeBigUInt64LE(v,804+8*i));
  assert.deepEqual(h.decodeDistribution(current).value.swap,{spent_units:'1',official_book_debit_lamports:'20',actual_native_output_lamports:'19',minimum_native_output_lamports:'19'});
  assert.throws(()=>h.decodeDistribution(current.subarray(0,891)));
  const extra=Buffer.from(current);extra[923]=1;assert.throws(()=>h.decodeDistribution(extra));
});
test('D111 V3 decoder preserves padded V2 prefix and separates completed recovery authority',()=>{
  const current=Buffer.alloc(1199);Buffer.from([104,51,125,187,226,55,209,99]).copy(current);
  current[8]=3;current[10]=1;current[11]=4;current[924]=1;
  current.writeBigUInt64LE(2n,925);current.writeBigUInt64LE(1n,933);current.writeBigUInt64LE(77n,941);
  Buffer.alloc(32,7).copy(current,949);Buffer.alloc(32,8).copy(current,981);
  current.writeBigUInt64LE(19n,1013);Buffer.alloc(32,9).copy(current,1021);current.writeBigInt64LE(1001n,1053);
  const decoded=h.decodeDistribution(current);assert.equal(decoded.value.version,3);
  assert.equal(decoded.value.recovery.generation,'2');assert.equal(decoded.value.recovery.acknowledged_cumulative_loss,'77');
  assert.equal(decoded.value.recovery.frozen_snapshot_hash,'07'.repeat(32));assert.equal(decoded.value.last_completed_recovery,null);
  const corrupt=Buffer.from(current);corrupt[900]=1;assert.throws(()=>h.decodeDistribution(corrupt));
  assert.throws(()=>h.decodeDistribution(current.subarray(0,924)));
  const idle=Buffer.alloc(1199);current.subarray(0,924).copy(idle);idle[11]=0;idle[924]=0;idle[925]=1;
  idle.writeBigUInt64LE(7n,926);idle.writeBigUInt64LE(77n,934);idle.writeBigUInt64LE(2n,942);idle[950]=3;
  Buffer.alloc(32,8).copy(idle,951);idle.writeBigUInt64LE(19n,983);Buffer.alloc(32,9).copy(idle,991);
  Buffer.alloc(32,7).copy(idle,1023);idle.writeBigInt64LE(1001n,1055);
  const history=h.decodeDistribution(idle).value;assert.equal(history.recovery,null);assert.equal(history.last_completed_recovery.cumulative_cooldown_loss,'77');
});
