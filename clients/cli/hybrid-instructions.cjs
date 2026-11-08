'use strict';
// Source-bound unsigned instruction helpers only. Reuse the pinned SDK and
// existing role derivation; no wallet, API quote, network, signing or sending.
const assert=require('node:assert/strict');
const fs=require('node:fs');
const path=require('node:path');
const {createHash}=require('node:crypto');
const distribution=require('./distribution-plan.cjs');
const intake=require('./intake-plan.cjs');
const state=require('./state-inspect.cjs');
const {PublicKey,TransactionInstruction}=require(require('../../validation/genesis-transport/transport.cjs').SDK);
const PROGRAM='GAjHRaC3XM98SoRoFy5B8P2ypTS1ndf1UPhXmjWgtPux';
const JUPITER='JUP6LkbZbjS1jKKwapdHNy74zcZ3tLUZoi5QNyVTaV4';
const MANIFEST='MNFSTqtC93rEfYHB6hF82sKdZpUDFWkViLByLd1k1Ms';
const WSOL='So11111111111111111111111111111111111111112';
const SOURCE_PINS=Object.freeze({
  "programs/piv1/src/swap_preparation_execution.rs": "695d0241a1ff4f028ce345fa9b82311cf7ce1c6d60cc36dee46c027615ef4468",
  "programs/piv1/src/integrations/jupiter.rs": "f91626d9d5e0c9b02728172cf410658d56b88c5a1424aa5c1e197c7dbc1f3abd",
  "programs/piv1/src/integrations/jupiter_route.rs": "4dedd2a239cbf358691cfd18e136b0cdd47f1e3a38f1e1dbf61ba9cd84baa0f5",
  "programs/piv1/src/instruction_boundary.rs": "7ea700c36ef0fa587a734b66b093767eab28a6a5e24fcf004c26f98965240028",
  "programs/piv1/src/events.rs": "eddf2a8ac08872a6c46718d830eb46a9bf192c12c14a56018cabc03a27088911",
  "programs/piv1/src/state/distribution.rs": "2a2cd3fcbef03cfe1c29978f10317979a76b464a42b4e659a0a44b11cece4146",
  "programs/piv1/src/state/transitions.rs": "f62f87917b62d29c65ed5c16ee72cda487dc75b8799a21773d5b89fd9e220334",
  "programs/piv1/src/state/reconciliation.rs": "3a7378dcad50decd46606c5610cf0c00d971936ca89916f896d03df53e25bdc3",
  "programs/piv1/src/state_persistence.rs": "ea5a03c7ae154ee0a6c580b6fc7ac84cfccad34bfe3bee83a1b350ebce3f8cc1",
  "programs/piv1/src/accounts.rs": "567ffd3f3d0c58ce031aa45a555007358bfdb3a8d419a6a84cfc069376513416",
  "programs/piv1/src/pending_accounts.rs": "937385b05e278c8be731e03f6cc9d20ca0b06b44d1200f188f1d78ec08d0bbf8",
  "clients/cli/state-inspect.cjs": "09f803c327bc3954597482b83c068928c186e93df20d47cb4eeaae343ebfc1a8",
  "clients/cli/distribution-plan.cjs": "cc6994f8dac0e5eadd2c299a3b7ef8b6a02256b627aad37a0bcd83b7ee6fc9c6",
  "clients/cli/intake-plan.cjs": "ae4bb850ca4a9c8432830a6d0e60f5398fb895f3b137ff428120dec6568b253c",
  "clients/cli/initializer-plan.cjs": "92c13a4205f4557c74f55b03ee2dfea458592938228a7a959f5a2baf35bbeb83"
});
const key=value=>{assert.equal(typeof value,'string');const k=new PublicKey(value);assert.equal(k.toBase58(),value);return k;};
const derive=(program,seeds)=>PublicKey.findProgramAddressSync(seeds,key(program))[0].toBase58();
const u64=value=>{assert.equal(typeof value,'string');assert.match(value,/^(0|[1-9][0-9]*)$/);const n=BigInt(value);assert(n<=(1n<<64n)-1n);const b=Buffer.alloc(8);b.writeBigUInt64LE(n);return b;};
function base(input){
  assert.equal(input.cluster,'testnet');assert.equal(input.programId,PROGRAM);
  const m=structuredClone(distribution.TEMPLATE);
  Object.assign(m,{cluster:input.cluster,programId:input.programId,action:'prepare-liquid',protocol:input.protocol,currentGuardians:input.currentGuardians});
  const validated=distribution.validateManifest(m);
  assert(input.protocol&&Object.values(input.protocol).every(v=>v!==null&&v!==undefined));
  assert(input.currentGuardians&&input.currentGuardians.registryRevision!==null&&input.currentGuardians.guardianKeys.length===6);
  assert(input.currentGuardians.guardianKeys.every(v=>v!==null&&v!==undefined));
  return distribution.nativeRoles(validated.manifest,validated.derived);
}
function prepareInstruction(input){
  assert.deepEqual(Object.keys(input).sort(),['cluster','currentGuardians','market','programId','protocol','sequence'].sort());
  const roles=base(input),program=input.programId,seq=u64(input.sequence),market=key(input.market);
  for(const i of [4,7])roles[i].writable=true;
  const suffix=[
    ['swap_authority',derive(program,[Buffer.from('swap-authority'),seq]),true],
    ['swap_input',derive(program,[Buffer.from('swap-jito'),seq]),true],
    ['swap_output',derive(program,[Buffer.from('swap-wsol'),seq]),true],
    ['wsol_mint',WSOL,false],['jupiter_program',JUPITER,false],
    ['jupiter_event_authority',derive(JUPITER,[Buffer.from('__event_authority')]),false],
    ['manifest_program',MANIFEST,false],['manifest_market',market.toBase58(),true],
    ['manifest_base_vault',derive(MANIFEST,[Buffer.from('vault'),market.toBuffer(),key(distribution.IDS.mint).toBuffer()]),true],
    ['manifest_quote_vault',derive(MANIFEST,[Buffer.from('vault'),market.toBuffer(),key(WSOL).toBuffer()]),true],
    ['manifest_quote_global',derive(MANIFEST,[Buffer.from('global'),key(WSOL).toBuffer()]),true],
    ['manifest_quote_global_vault',derive(MANIFEST,[Buffer.from('global-vault'),key(WSOL).toBuffer()]),true],
  ];
  roles.push(...suffix.map(([role,address,writable])=>({role,address,writable,signer:false})));
  assert.equal(new Set(roles.map(r=>r.address)).size,roles.length,'account alias');
  return {instruction:new TransactionInstruction({programId:key(program),keys:roles.map(r=>({pubkey:key(r.address),isWritable:r.writable,isSigner:false})),data:Buffer.from('PIV1JS01\x01','binary')}),roles,
    expectedReceiptEvents:['DistributionSwapPrepared','DistributionPrepared'],status:'UNSIGNED_INSTRUCTION_ONLY'};
}
function migrationInstruction(input){
  assert.deepEqual(Object.keys(input).sort(),['cluster','programId']);assert.equal(input.cluster,'testnet');assert.equal(input.programId,PROGRAM);
  const d=intake.derived(PROGRAM),names=['config','active_distribution','pending_sol','principal_sol','operational_sol','distribution_escrow','kif_sol','principal_token','pending_token'];
  const keys=names.map((name,i)=>({pubkey:key(d[name]),isSigner:false,isWritable:i===1||i===4}));
  keys.push({pubkey:key(distribution.IDS.system),isSigner:false,isWritable:false});
  return new TransactionInstruction({programId:key(PROGRAM),keys,data:Buffer.from('PIV1MR02\x01','binary')});
}
function verifySources(){
  assert(Object.keys(SOURCE_PINS).length>0,'unbound source');
  for(const [name,expected]of Object.entries(SOURCE_PINS))assert.equal(createHash('sha256').update(fs.readFileSync(path.resolve(__dirname,'../..',name))).digest('hex'),expected,`source binding: ${name}`);
  return SOURCE_PINS;
}
module.exports={PROGRAM,JUPITER,MANIFEST,WSOL,SOURCE_PINS,prepareInstruction,migrationInstruction,decodeDistribution:state.decodeDistribution,verifySources};
