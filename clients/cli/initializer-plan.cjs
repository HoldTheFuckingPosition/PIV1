'use strict';
// Public declarations and unsigned offline encodings only. No wallet or network API.
const assert = require('node:assert/strict');
const fs = require('node:fs');
const path = require('node:path');
const t = require('../../validation/genesis-transport/transport.cjs');
const native = require('../../validation/genesis-transport/production-initializer.cjs');
const {PublicKey,TransactionInstruction,TransactionMessage,VersionedTransaction,
  AddressLookupTableAccount,AddressLookupTableProgram,ComputeBudgetProgram,SystemProgram} = require(t.SDK);

const BASELINE = 'a0f785b6dc321c8aca776fb4cdf522aae3176f41';
const SOURCE_PINS = Object.freeze({
  "programs/piv1/src/instructions/initialize.rs": "718cb1cb92a01101ab25660a2d6155ab1051d207be06079e20d85940422aafe5",
  "programs/piv1/src/instruction_boundary.rs": "0e082adeebe7cabe5f5e14f69b244c2f1464c3c2f4eacd0077173a9aad91ab9f",
  "programs/piv1/src/accounts.rs": "08e3d28971160910088a40d74215d15a9c2d07b11142e28c4fb5162d725f1c80",
  "programs/piv1/src/constants.rs": "2498c79c42d732be84aae423cf31d82997574cc08cca02fb226fea9acaa24f94",
  "programs/piv1/src/genesis_model.rs": "fe1544ee506429ad9b2d02fee8fdea381cd27265d9ad220360076b7d25e6cc44",
  "programs/piv1/src/state/config.rs": "9d7a57c87b6781b943e0e9c13d08f73a21058d400b40540c485e6c596b042198",
  "programs/piv1/src/state/distribution.rs": "a291d3fb94921cd036161ee669190074fd5f159fe0d9f523a98ef3b0fa25d3a3",
  "programs/piv1/src/state/guardian.rs": "ce762257b8025789f8ec43351ce166f0a5f7ad0bb6f05b4c9515173d7f1cd619",
  "programs/piv1/src/guardian_clock_accounts.rs": "5cc36b79d429bb3e1818466d429259474c8a26d1c1288da16a71f22033f84946",
  "programs/piv1/src/kif_claim_accounts.rs": "671079e1fdd714f39986f8e6ff6e65515e65fde7a35f6d1ba8b6f1e158e53e2a",
  "programs/piv1/src/genesis_allocation.rs": "d7878a592ad670396a9158e7cd5c1edc31f390302af373ac16d8cf69b56e968e",
  "programs/piv1/src/genesis_recipients.rs": "2f543c0f60ce431efbcae202fe4098ab8ca918a018b7c831a5d86b9a405f9d17",
  "programs/piv1/src/genesis_preflight.rs": "478c8fc69f409b911478ff07592eb8ec95f31b197b132971134eab6df579da4c",
  "programs/piv1/src/squads_accounts.rs": "b5ccacf047d86666eadaf568019ac2df7318fb954d62ae87866ba284324e518b",
  "programs/piv1/src/squads_execution.rs": "589252b8357d76e9af61d4f4aaed7e7d54692db63bddd98ea807cee6357321fd",
  "validation/genesis-bank-runtime/tests/real_squads_initializer.rs": "181bf8a47dfa674df20563229e7e7b89cccce200a92ea97112b7391bb19787c9",
  "validation/genesis-bank-runtime/Cargo.lock": "7321dae7beee9ca1db69b332f75c3d1196104ab68fd5718a68f3bf5023cee903",
  "validation/genesis-transport/transport.cjs": "a7694ff07577988e96546a53008bc8ab85bbc0ae7cb9353dcfc25e6c5a3c3724",
  "validation/genesis-transport/production-initializer.cjs": "c9cf0e28fa21d13e8b23b697023ad807d40f609cf46473c5019e17c46f8383a7",
  "spikes/task-0.4-jito/package-lock.json": "a47e4310aa29ffb2ec0943626616bfefb0f0b8d7408090b360018b7e5e215e89",
  "spikes/task-0.4-jito/node_modules/@solana/web3.js/package.json": "33c76444048957077187a5ef01365a421f4680a1bde95aae2366de8105d4534f",
  "spikes/task-0.4-jito/node_modules/@solana/web3.js/lib/index.cjs.js": "796768bf5a689d808fb6fabfc5ce0db27eee064782cf87621331b807bab9f7e2"
});
const SCHEMA = 'piv1-initializer-plan-v2';
const IDS = Object.freeze({
  squads:'SQDS4ep65T869zMMBKyuUq6aD6EgTu8psMjkvj52pCf',
  poolProgram:'SPoo1Ku8WFXoNDMHPsrGSTSG1Y47rzgn41SLUNakuHy',
  pool:'Jito4APyf642JPZPx3hGc6WWJ8zPKtRbRs4P815Awbb',
  mint:'J1toso1uCk3RLmjorhTtrVwY9HJ7X8V9yYac6Y7kGCPn',
  token:'TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA',
  stake:'Stake11111111111111111111111111111111111111',
  system:'11111111111111111111111111111111',
  loader:'BPFLoaderUpgradeab1e11111111111111111111111',
  instructions:'Sysvar1nstructions1111111111111111111111111',
  altProgram:'AddressLookupTab1e1111111111111111111111111',
  computeBudget:'ComputeBudget111111111111111111111111111111',
});
const TEMPLATE = {
  schema:SCHEMA,cluster:'testnet',programId:null,feePayer:null,rentPayer:null,
  governance:{multisigAddress:null,createKey:null,vaultIndex:null,bufferIndex:null,
    transactionIndex:null,currentTransactionIndex:null,staleTransactionIndex:null,
    guardians:Array(6).fill(null),guardianSlotPermutation:Array(6).fill(null),
    creator:null,executor:null,approvers:Array(4).fill(null),timeLockSeconds:null},
  protocol:{validatorList:null,reserve:null,managerFee:null,referrer:null},
  recipients:{htfpRecipient:null,teamOwnerRecipient:null},
  initialization:{initiallyPaused:null,anchorUnixTimestamp:null},
  transport:{recentBlockhash:null,lastValidBlockHeight:null,referenceSlot:null,
    altAuthority:null,altRecentSlot:null},
};
const U64=(1n<<64n)-1n, I64_MIN=-(1n<<63n), I64_MAX=(1n<<63n)-1n;
const MAX_MANIFEST_BYTES=65536;
const text=s=>Buffer.from(s), same=(a,b)=>a.equals(b), pk=s=>new PublicKey(s);
const meta=(pubkey,isSigner=false,isWritable=false)=>({pubkey,isSigner,isWritable});
const u64=n=>{const b=Buffer.alloc(8);b.writeBigUInt64LE(BigInt(n));return b;};
const u32=n=>{const b=Buffer.alloc(4);b.writeUInt32LE(n);return b;};
const vec=b=>Buffer.concat([u32(b.length),b]);
const pda=(program,seeds)=>PublicKey.findProgramAddressSync(seeds,program)[0];
const fail=(code,field)=>{const e=new Error(code);e.code=code;e.field=field;throw e;};
const plain=v=>v!==null&&typeof v==='object'&&!Array.isArray(v)&&Object.getPrototypeOf(v)===Object.prototype;

// JSON.parse validates grammar first; this lexical walk additionally rejects
// duplicate object keys, including differently escaped spellings of one key.
function parseManifest(source) {
  if(typeof source!=='string'||Buffer.byteLength(source)>MAX_MANIFEST_BYTES)fail('manifest_size','manifest');
  let parsed;try{parsed=JSON.parse(source);}catch{fail('invalid_json','manifest');}
  const tokens=source.match(/"(?:\\.|[^"\\])*"|[{}\[\],:]|true|false|null|-?(?:0|[1-9]\d*)(?:\.\d+)?(?:[eE][+-]?\d+)?/g)||[];
  const stack=[];
  for(const token of tokens) {
    if(token==='{')stack.push({object:true,key:true,seen:new Set()});
    else if(token==='[')stack.push({object:false});
    else if(token==='}'||token===']')stack.pop();
    else if(token===','){if(stack.at(-1)?.object)stack.at(-1).key=true;}
    else if(token.startsWith('"')&&stack.at(-1)?.object&&stack.at(-1).key) {
      const state=stack.at(-1),key=JSON.parse(token);
      if(state.seen.has(key))fail('duplicate_json_key','manifest');state.seen.add(key);state.key=false;
    }
  }
  return parsed;
}

function validateManifest(input) {
  const missing=[];
  function walk(value,shape,field) {
    if(value===undefined||value===null){missing.push(field);return null;}
    if(Array.isArray(shape)) {
      if(!Array.isArray(value)||value.length!==shape.length)fail('array_shape',field);
      return Array.from({length:value.length},(_,i)=>walk(value[i],null,`${field}[${i}]`));
    }
    if(plain(shape)) {
      if(!plain(value))fail('object_required',field);
      if(Object.keys(value).some(k=>!Object.hasOwn(shape,k)))fail('unknown_field',field);
      return Object.fromEntries(Object.entries(shape).map(([k,v])=>[k,walk(value[k],v,field?`${field}.${k}`:k)]));
    }
    return value;
  }
  if(!plain(input))fail('object_required','manifest');
  const m=walk(input,TEMPLATE,''),g=m.governance,p=m.protocol,r=m.recipients,i=m.initialization,c=m.transport;
  const check=(value,field,fn)=>{if(value!==null)fn(value,field);};
  const publicKey=(value,field)=>{
    if(typeof value!=='string')fail('public_key_string',field);
    let k;try{k=pk(value);}catch{fail('invalid_public_key',field);}
    if(k.toBase58()!==value||value===IDS.system)fail('noncanonical_or_zero_public_key',field);
  };
  const decimal=(value,field,signed=false)=>{
    if(typeof value!=='string'||!(/^(?:0|[1-9]\d*|-[1-9]\d*)$/).test(value))fail('decimal_string_required',field);
    const n=BigInt(value);if(n<(signed?I64_MIN:0n)||n>(signed?I64_MAX:U64))fail('integer_range',field);
  };
  const small=(value,field,max)=>{if(typeof value!=='number'||!Number.isInteger(value)||value<0||value>max)fail('integer_range',field);};
  check(m.schema,'schema',v=>{if(v!==SCHEMA)fail('unsupported_schema','schema');});
  check(m.cluster,'cluster',v=>{if(v!=='testnet')fail('unsupported_cluster','cluster');});
  for(const field of ['programId','feePayer','rentPayer'])check(m[field],field,publicKey);
  if(m.programId!==null&&Object.values(IDS).includes(m.programId))fail('program_identity_alias','programId');
  if(g) {
    for(const field of ['multisigAddress','createKey','creator','executor'])check(g[field],`governance.${field}`,publicKey);
    for(const field of ['vaultIndex','bufferIndex'])check(g[field],`governance.${field}`,(v,f)=>small(v,f,255));
    for(const field of ['transactionIndex','currentTransactionIndex','staleTransactionIndex'])check(g[field],`governance.${field}`,decimal);
    for(const field of ['guardians','approvers'])if(g[field])g[field].forEach((v,n)=>check(v,`governance.${field}[${n}]`,publicKey));
    if(g.guardianSlotPermutation)g.guardianSlotPermutation.forEach((v,n)=>check(v,`governance.guardianSlotPermutation[${n}]`,(v,f)=>small(v,f,5)));
    check(g.timeLockSeconds,'governance.timeLockSeconds',(v,f)=>small(v,f,7776000));
    for(const field of ['guardians','approvers','guardianSlotPermutation'])if(g[field]){
      const known=g[field].filter(v=>v!==null);if(new Set(known).size!==known.length)fail('duplicate_entry',`governance.${field}`);
    }
    if(g.transactionIndex!==null&&BigInt(g.transactionIndex)===0n)fail('zero_transaction_index','governance.transactionIndex');
    if(g.currentTransactionIndex!==null&&BigInt(g.currentTransactionIndex)===U64)fail('next_index_overflow','governance.currentTransactionIndex');
    if(g.transactionIndex!==null&&g.currentTransactionIndex!==null&&BigInt(g.transactionIndex)!==BigInt(g.currentTransactionIndex)+1n)fail('next_index_mismatch','governance.transactionIndex');
    if(g.staleTransactionIndex!==null&&g.currentTransactionIndex!==null&&BigInt(g.staleTransactionIndex)>BigInt(g.currentTransactionIndex))fail('stale_index_mismatch','governance.staleTransactionIndex');
    const guardians=g.guardians?.filter(v=>v!==null)||[];
    if(m.rentPayer!==null&&guardians.includes(m.rentPayer))fail('rent_payer_guardian_alias','rentPayer');
    if(guardians.length===6)for(const member of [g.creator,g.executor,...(g.approvers||[])])if(member!==null&&!guardians.includes(member))fail('declared_member_required','governance');
  }
  if(p)for(const field of Object.keys(TEMPLATE.protocol))check(p[field],`protocol.${field}`,publicKey);
  if(r)for(const field of Object.keys(TEMPLATE.recipients))check(r[field],`recipients.${field}`,publicKey);
  if(r&&r.htfpRecipient!==null&&r.teamOwnerRecipient!==null&&r.htfpRecipient===r.teamOwnerRecipient)fail('recipient_alias','recipients');
  if(i){check(i.initiallyPaused,'initialization.initiallyPaused',(v,f)=>{if(typeof v!=='boolean')fail('boolean_required',f);});check(i.anchorUnixTimestamp,'initialization.anchorUnixTimestamp',(v,f)=>decimal(v,f,true));}
  if(c) {
    for(const field of ['recentBlockhash','altAuthority'])check(c[field],`transport.${field}`,publicKey);
    for(const field of ['lastValidBlockHeight','referenceSlot','altRecentSlot'])check(c[field],`transport.${field}`,decimal);
    if(c.altRecentSlot!==null&&c.referenceSlot!==null&&BigInt(c.altRecentSlot)>BigInt(c.referenceSlot))fail('future_alt_recent_slot','transport.altRecentSlot');
  }
  if(g&&g.multisigAddress!==null&&g.createKey!==null&&!same(pk(g.multisigAddress),pda(pk(IDS.squads),[text('multisig'),text('multisig'),pk(g.createKey).toBuffer()])))fail('multisig_seed_mismatch','governance.multisigAddress');
  const knownRoles=[['program',m.programId],['multisig',g?.multisigAddress],['rent',m.rentPayer],...Object.entries(p||{}).map(([k,v])=>[k,v])].filter(([,v])=>v!==null&&v!==undefined);
  for(let a=0;a<knownRoles.length;a++)for(let b=0;b<a;b++)if(knownRoles[a][1]===knownRoles[b][1]&&![knownRoles[a][0],knownRoles[b][0]].every(k=>['managerFee','referrer'].includes(k)))fail('unsupported_known_role_alias','manifest');
  const signers=[['feePayer',m.feePayer],['rentPayer',m.rentPayer],['governance.creator',g?.creator],['governance.executor',g?.executor],['transport.altAuthority',c?.altAuthority],...(g?.approvers||[]).map((v,n)=>[`governance.approvers[${n}]`,v])];
  for(const [field,value]of signers)if(value!==null&&value!==undefined&&!PublicKey.isOnCurve(pk(value).toBytes()))fail('unsupported_off_curve_outer_signer',field);
  return {manifest:m,missing};
}

function deriveContext(m) {
  const g=m.governance,program=pk(m.programId),multisig=pk(g.multisigAddress),squads=pk(IDS.squads),
    deriveVault=index=>pda(squads,[text('multisig'),multisig.toBuffer(),text('vault'),Buffer.from([index])]);
  if(!same(multisig,pda(squads,[text('multisig'),text('multisig'),pk(g.createKey).toBuffer()])))fail('multisig_seed_mismatch','governance.multisigAddress');
  if(Object.values(IDS).includes(m.programId))fail('program_identity_alias','programId');
  const guardians=g.guardians.map(pk).sort((a,b)=>Buffer.compare(a.toBuffer(),b.toBuffer()));
  const assigned=g.guardianSlotPermutation.map(n=>guardians[n]);
  const targets=['config','distribution','guardian-registry'].map(s=>pda(program,[text(s)]));
  for(let slot=0;slot<6;slot++)targets.push(pda(program,[text('guardian-reward'),assigned[slot].toBuffer(),u64(0),Buffer.from([slot])]));
  targets.push(...['pending-sol','principal-sol','operational-sol','distribution-escrow','kif-sol','principal-jito-vault','pending-jito-vault'].map(s=>pda(program,[text(s)])));
  const recipients=[m.recipients.htfpRecipient,m.recipients.teamOwnerRecipient].map(pk);
  const protocol=[pk(IDS.poolProgram),pk(IDS.pool),pk(m.protocol.validatorList),pk(m.protocol.reserve),pk(IDS.mint),pk(m.protocol.managerFee),pk(m.protocol.referrer)];
  const shared=same(protocol[5],protocol[6]),vault=deriveVault(g.vaultIndex),transaction=pda(squads,[text('multisig'),multisig.toBuffer(),text('transaction'),u64(g.transactionIndex)]),
    proposal=pda(squads,[text('multisig'),multisig.toBuffer(),text('transaction'),u64(g.transactionIndex),text('proposal')]),
    programData=pda(pk(IDS.loader),[program.toBuffer()]),authority=pda(program,[text('authority')]),rentPayer=pk(m.rentPayer);
  const addresses=[program,programData,multisig,proposal,transaction,vault,pk(IDS.instructions),...targets,...protocol.slice(0,shared?6:7),rentPayer,SystemProgram.programId,pk(IDS.token),...recipients];
  if(new Set(addresses.map(k=>k.toBase58())).size!==addresses.length)fail('unsupported_inner_role_alias','manifest');
  if(addresses.some(k=>same(k,authority))||guardians.some(k=>addresses.slice(0,-2).some(a=>same(a,k)))||recipients.some(k=>same(k,authority)))fail('guardian_authority_or_role_alias','governance');
  const protectedKeys=new Set([...addresses.filter(k=>!same(k,rentPayer)),authority,...Object.values(IDS).map(pk)].map(k=>k.toBase58()));
  for(const [field,value]of [['feePayer',m.feePayer],['transport.altAuthority',m.transport.altAuthority]])if(protectedKeys.has(value))fail('unsupported_external_signer_alias',field);
  const roles=addresses.map((address,index)=>meta(address,index===5||index===addresses.length-5,(index>=7&&index<23)||index===addresses.length-5));
  const roleNames=['program','program_data','multisig','proposal','transaction','governance_vault','instructions',
    'config','distribution','guardian_registry',...Array.from({length:6},(_,i)=>`guardian_reward_${i}`),
    'pending_sol','principal_sol','operational_sol','distribution_escrow','kif_sol','principal_token','pending_token',
    'pool_program','pool','validator_list','reserve','mint','manager_fee',...(shared?[]:['referrer']),
    'rent_payer','system_program','token_program','htfp_recipient','team_owner_recipient'];
  assert.equal(roleNames.length,roles.length);
  const buffer=pda(squads,[text('multisig'),multisig.toBuffer(),text('transaction_buffer'),pk(g.creator).toBuffer(),Buffer.from([g.bufferIndex])]);
  const [altCreate,altAddress]=AddressLookupTableProgram.createLookupTable({authority:pk(m.transport.altAuthority),payer:rentPayer,recentSlot:BigInt(m.transport.altRecentSlot)});
  if(protectedKeys.has(buffer.toBase58())||protectedKeys.has(altAddress.toBase58())||same(buffer,altAddress)||guardians.some(k=>same(k,buffer)||same(k,altAddress)||same(k,authority)))fail('derived_resource_alias','governance');
  return {m,program,programData,multisig,proposal,transaction,vault,authority,guardians,assigned,targets,recipients,protocol,shared,roles,roleNames,buffer,altCreate,altAddress};
}

function encodeNative(c) {
  const data=Buffer.alloc(313);data.write('PIV1IN02');data[8]=2;data[9]=c.m.governance.vaultIndex;data[10]=Number(c.m.initialization.initiallyPaused);
  [...c.protocol,...c.recipients].forEach((k,n)=>k.toBuffer().copy(data,11+32*n));
  data.writeBigInt64LE(BigInt(c.m.initialization.anchorUnixTimestamp),299);Buffer.from(c.m.governance.guardianSlotPermutation).copy(data,307);
  const decoded=native.decodeNative(data);assert.equal(decoded.vaultIndex,c.m.governance.vaultIndex);assert.deepEqual(decoded.permutation,c.m.governance.guardianSlotPermutation);return data;
}
function instruction(name,keys,args=Buffer.alloc(0)){return new TransactionInstruction({programId:pk(IDS.squads),keys,data:Buffer.concat([t.disc(name),args])});}
const instructionJSON=ix=>({programId:ix.programId.toBase58(),accounts:ix.keys.map(k=>({address:k.pubkey.toBase58(),signer:k.isSigner,writable:k.isWritable})),dataHex:ix.data.toString('hex')});
function compile(c,instructions,tables=null) {
  const message=new TransactionMessage({payerKey:pk(c.m.feePayer),recentBlockhash:c.m.transport.recentBlockhash,instructions});
  return tables===null?message.compileToLegacyMessage():message.compileToV0Message(tables);
}
function packet(c,name,instructions,tables=null) {
  const message=compile(c,instructions,tables),n=message.header.numRequiredSignatures;
  if(tables===null)assert(t.legacyWireSize(message)<=1232,'packet exceeds1232');
  const wire=Buffer.from(new VersionedTransaction(message).serialize());assert(wire.length<=1232);
  const restored=VersionedTransaction.deserialize(wire);assert.equal(restored.signatures.length,n);assert(restored.signatures.every(s=>s.length===64&&s.every(b=>b===0)));
  assert.deepEqual(Buffer.from(restored.serialize()),wire);
  const recovered=TransactionMessage.decompile(restored.message,tables===null?undefined:{addressLookupTableAccounts:tables});
  assert(same(recovered.payerKey,pk(c.m.feePayer)));assert.equal(recovered.recentBlockhash,c.m.transport.recentBlockhash);
  const union=new Map(),add=k=>{const s=k.pubkey.toBase58(),old=union.get(s);union.set(s,meta(k.pubkey,k.isSigner||!!old?.isSigner,k.isWritable||!!old?.isWritable));};
  add(meta(pk(c.m.feePayer),true,true));for(const ix of instructions){add(meta(ix.programId));ix.keys.forEach(add);}
  assert.equal(recovered.instructions.length,instructions.length);
  recovered.instructions.forEach((ix,i)=>{assert(same(ix.programId,instructions[i].programId));assert.deepEqual(ix.data,instructions[i].data);assert.equal(ix.keys.length,instructions[i].keys.length);
    ix.keys.forEach((k,j)=>{const original=instructions[i].keys[j],expected=union.get(original.pubkey.toBase58());assert(same(k.pubkey,original.pubkey));assert.equal(k.isSigner,expected.isSigner);assert.equal(k.isWritable,expected.isWritable);});});
  const expectedSigners=[...union.values()].filter(k=>k.isSigner).map(k=>k.pubkey.toBase58()).sort();
  const signers=message.staticAccountKeys.slice(0,n).map(k=>k.toBase58());assert.deepEqual([...signers].sort(),expectedSigners);
  return {name,version:message.version,requiredSigners:signers,signatureCount:n,zeroSignatureBytes:64*n,wireBytes:wire.length,
    wireBase64:wire.toString('base64'),sha256:t.hash(wire).toString('hex'),requestedInstructions:instructions.map(instructionJSON),
    wireInstructions:recovered.instructions.map(instructionJSON)};
}
function budgets() {
  // The pinned JS SDK predates selector4; pinned native interface3.0.0 encodes
  // SetLoadedAccountsDataSizeLimit as u8(4),u32. No SDK migration is needed.
  const result=[ComputeBudgetProgram.setComputeUnitLimit({units:1400000}),ComputeBudgetProgram.requestHeapFrame({bytes:262144}),
    new TransactionInstruction({programId:ComputeBudgetProgram.programId,keys:[],data:Buffer.concat([Buffer.from([4]),u32(8388608)])})];
  assert.deepEqual(result.map(ix=>ix.data.toString('hex')),['02c05c1500','0100000400','0400008000']);assert(result.every(ix=>ix.keys.length===0));return result;
}
function fittedChunk(c,remaining,make) {
  for(let length=Math.min(800,remaining.length);length>0;length--)if(t.legacyWireSize(compile(c,[make(remaining.subarray(0,length))]))<=1232)return remaining.subarray(0,length);
  fail('no_fitting_upload_packet','manifest');
}
function buildPackets(c,compact,stored) {
  const g=c.m.governance,rent=pk(c.m.rentPayer),creator=pk(g.creator),base=[meta(c.multisig,false,true),meta(c.transaction,false,true),meta(creator,true),meta(rent,true,true),meta(SystemProgram.programId)];
  const makeCreate=chunk=>instruction('transaction_buffer_create',[meta(c.multisig),meta(c.buffer,false,true),meta(creator,true),meta(rent,true,true),meta(SystemProgram.programId)],
    Buffer.concat([Buffer.from([g.bufferIndex,g.vaultIndex]),t.hash(compact),Buffer.from([compact.length&255,compact.length>>8]),vec(chunk)]));
  const first=fittedChunk(c,compact,makeCreate),chunks=[first],uploads=[packet(c,'buffer_create',[makeCreate(first)])];let offset=first.length;
  const makeExtend=chunk=>instruction('transaction_buffer_extend',[meta(c.multisig),meta(c.buffer,false,true),meta(creator,true)],vec(chunk));
  while(offset<compact.length){const chunk=fittedChunk(c,compact.subarray(offset),makeExtend);chunks.push(chunk);uploads.push(packet(c,`buffer_extend_${uploads.length}`,[makeExtend(chunk)]));offset+=chunk.length;}
  assert.deepEqual(Buffer.concat(chunks),compact);
  const finish=instruction('vault_transaction_create_from_buffer',[...base,meta(c.buffer,false,true),meta(creator,true,true)],Buffer.concat([Buffer.from([g.vaultIndex,0]),vec(Buffer.alloc(6)),Buffer.from([0])]));
  const proposal=instruction('proposal_create',[meta(c.multisig),meta(c.proposal,false,true),meta(creator,true),meta(rent,true,true),meta(SystemProgram.programId)],Buffer.concat([u64(g.transactionIndex),Buffer.from([0])]));
  const approvals=g.approvers.map((s,n)=>packet(c,`approve_${n+1}`,[instruction('proposal_approve',[meta(c.multisig),meta(pk(s),true,true),meta(c.proposal,false,true)],Buffer.from([0]))]));
  const execute=instruction('vault_transaction_execute',[meta(c.multisig),meta(c.proposal,false,true),meta(c.transaction),meta(pk(g.executor),true),
    ...stored.keys.map(k=>meta(k.pubkey,k.isSigner&&!same(k.pubkey,c.vault),k.isWritable))]);
  const extend=AddressLookupTableProgram.extendLookupTable({lookupTable:c.altAddress,authority:pk(c.m.transport.altAuthority),payer:rent,addresses:c.targets});
  // Compilation-only table. These placeholder metadata are NOT chain observations.
  const table=new AddressLookupTableAccount({key:c.altAddress,state:{deactivationSlot:U64,lastExtendedSlot:0,lastExtendedSlotStartIndex:0,authority:pk(c.m.transport.altAuthority),addresses:c.targets}});
  const full=[...budgets(),execute],compiled=compile(c,full,[table]);assert.equal(compiled.addressTableLookups.length,1);
  assert.deepEqual([...compiled.addressTableLookups[0].writableIndexes],Array.from({length:16},(_,i)=>i));assert.equal(compiled.addressTableLookups[0].readonlyIndexes.length,0);
  return {chunks:chunks.map(b=>b.length),packets:[packet(c,'alt_create',[c.altCreate]),packet(c,'alt_extend',[extend]),...uploads,
    packet(c,'transaction_from_buffer',[finish]),packet(c,'proposal_create',[proposal]),...approvals,packet(c,'execute',full,[table])]};
}
const OPEN = Object.freeze({publicReady:false,deploymentAuthorized:false,walletControlVerified:false,chainSnapshotVerified:false,
  altCreated:false,altWarmupVerified:false,actualFees:null,actualRent:null,actualFundingQuote:null,publicConsumptionMeasured:false,
  remainingOperatorHandlers:'OPEN',m4Reservation:'OPEN_UNWAIVED'});
function planManifest(input) {
  const {manifest:m,missing}=validateManifest(input),common={schema:SCHEMA,sourceBaseline:BASELINE,
    governanceSchema:{revision:t.PIN,idlSha256:native.GOVERNANCE_SCHEMA.idlSha256},readiness:{...OPEN}};
  if(missing.length)return {...common,status:'INCOMPLETE',missing,plan:null};
  const c=deriveContext(m),data=encodeNative(c),inner=new TransactionInstruction({programId:c.program,keys:c.roles,data}),
    compact=t.compactMessage({profile:'public-offline-plan',program:c.program,inner}),stored=t.decodeCompact(compact);
  assert.equal(compact.length,c.shared?1445:1478);assert.equal(c.roles.length,c.shared?34:35);
  const made=buildPackets(c,compact,stored);
  return {...common,status:'OFFLINE_PLAN_ONLY',missing:[],declarationSha256:t.hash(Buffer.from(JSON.stringify(m))).toString('hex'),
    plan:{program:c.program.toBase58(),programData:c.programData.toBase58(),pivAuthority:c.authority.toBase58(),multisig:c.multisig.toBase58(),vault:c.vault.toBase58(),
      transaction:c.transaction.toBase58(),proposal:c.proposal.toBase58(),buffer:c.buffer.toBase58(),
      canonicalDeclaredGuardians:c.guardians.map(k=>k.toBase58()),guardianSlots:c.assigned.map(k=>k.toBase58()),recipients:c.recipients.map(k=>k.toBase58()),
      governanceDeclarations:{transactionIndex:m.governance.transactionIndex,currentTransactionIndex:m.governance.currentTransactionIndex,staleTransactionIndex:m.governance.staleTransactionIndex,
        timeLockSeconds:m.governance.timeLockSeconds,creator:m.governance.creator,executor:m.governance.executor,approvers:m.governance.approvers,authenticated:false},
      native:{bytes:313,hex:data.toString('hex'),sha256:t.hash(data).toString('hex'),
        parameters:{vaultIndex:m.governance.vaultIndex,initiallyPaused:m.initialization.initiallyPaused,anchorUnixTimestamp:m.initialization.anchorUnixTimestamp,
          guardianSlotPermutation:m.governance.guardianSlotPermutation,recipientAddresses:[m.recipients.htfpRecipient,m.recipients.teamOwnerRecipient],protocol:m.protocol},
        roles:c.roles.map((k,n)=>({index:n,role:c.roleNames[n],address:k.pubkey.toBase58(),signer:k.isSigner,writable:k.isWritable}))},
      compact:{bytes:compact.length,hex:compact.toString('hex'),sha256:t.hash(compact).toString('hex'),storedLookupCount:0,uploadChunks:made.chunks},
      accountSizes:{pivTargets:[1014,891,210,84,84,84,84,84,84,0,0,0,0,0,165,165],buffer:112+compact.length,transaction:c.shared?1546:1579,proposal:646,alt:568,
        dynamicExternalAccounts:null,programDataArtifact:null},
      requestedExecuteBudget:{computeUnits:1400000,heapBytes:262144,loadedAccountBytes:8388608},
      alt:{address:c.altAddress.toBase58(),authority:m.transport.altAuthority,recentSlot:m.transport.altRecentSlot,addresses:c.targets.map(k=>k.toBase58()),
        compilationState:'HYPOTHETICAL_ONLY',actualOwner:null,actualLastExtendedSlot:null,warmupVerified:false,
        preconditions:['Create must succeed using an eligible recent slot in SlotHashes.','Extend must succeed with these exact16 addresses in this order.','Independently verify actual ALT owner, authority, active state and complete content.','Execution currentSlot must be strictly greater than actual final lastExtendedSlot.']},
      referenceContext:{...m.transport,verified:false,usage:'ENCODING_INPUT_ONLY; refresh blockhash/lastValidBlockHeight independently for every future signed step, including after human approvals/timelock.'},
      preconditions:['Declarations are not authenticated current multisig membership/4-of-6 threshold/permissions/timelock or an account snapshot.',
        'Current multisig transaction index must still equal declared currentTransactionIndex before creation; refresh all current fields and stop on mismatch.',
        'Verify PIV ProgramData authority, virgin targets, recipient funding/owners, protocol ownership/links/prefunds and exact selected artifacts independently.',
        'Verify real signer control, funding, fees, rent and all phase validity windows. This bundle is never send-ready or deployment-ready.'],packets:made.packets}};
}
function verifySources() {
  assert.equal(process.version,'v24.19.0');assert.equal(require(path.join(t.SDK,'package.json')).version,'1.98.4');
  for(const [file,expected]of Object.entries(SOURCE_PINS))assert.equal(t.hash(fs.readFileSync(path.join(t.REPO,file))).toString('hex'),expected,`source binding: ${file}`);
  return {node:process.version,web3:'1.98.4',pins:SOURCE_PINS};
}
function main(args) {
  const sourceCheck=verifySources();
  if(args.length===1&&args[0]==='--template'){process.stdout.write(JSON.stringify(TEMPLATE,null,2)+'\n');return;}
  if(args.length!==2||args[0]!=='--manifest')fail('unsupported_arguments','arguments');
  const filename=path.resolve(args[1]),stat=fs.statSync(filename);if(!stat.isFile())fail('public_manifest_file_required','manifest');if(stat.size>MAX_MANIFEST_BYTES)fail('manifest_size','manifest');
  const report=planManifest(parseManifest(fs.readFileSync(filename,'utf8')));report.sourceCheck=sourceCheck;
  process.stdout.write(JSON.stringify(report,null,2)+'\n');
}
module.exports={BASELINE,SOURCE_PINS,SCHEMA,IDS,TEMPLATE,PublicKey,parseManifest,validateManifest,deriveContext,encodeNative,compile,packet,budgets,planManifest,verifySources};
if(require.main===module){try{main(process.argv.slice(2));}catch(e){process.stderr.write(JSON.stringify({status:'INVALID_OR_UNSUPPORTED_PLAN',code:e.code||'plan_validation_failed',field:e.field||'manifest'})+'\n');process.exitCode=1;}}
