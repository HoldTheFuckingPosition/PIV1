'use strict';
// Exact unsigned direct-Squads one-seat guardian synchronization sequence. Public declarations only.
const assert=require('node:assert/strict'),fs=require('node:fs'),path=require('node:path');
const {createHash}=require('node:crypto'),initializer=require('./initializer-plan.cjs'),intake=require('./intake-plan.cjs');
const t=require('../../validation/genesis-transport/transport.cjs'),{PublicKey,TransactionInstruction,ComputeBudgetProgram}=require(t.SDK);
const BASELINE='32aae6225be6028eb6c419affa1c518077f47ed8';
const SOURCE_PINS=Object.freeze({
  "programs/piv1/src/squads_execution.rs": "589252b8357d76e9af61d4f4aaed7e7d54692db63bddd98ea807cee6357321fd",
  "programs/piv1/src/squads_accounts.rs": "b5ccacf047d86666eadaf568019ac2df7318fb954d62ae87866ba284324e518b",
  "programs/piv1/src/guardian_clock_accounts.rs": "5cc36b79d429bb3e1818466d429259474c8a26d1c1288da16a71f22033f84946",
  "programs/piv1/src/accounts.rs": "08e3d28971160910088a40d74215d15a9c2d07b11142e28c4fb5162d725f1c80",
  "programs/piv1/src/state/guardian.rs": "ce762257b8025789f8ec43351ce166f0a5f7ad0bb6f05b4c9515173d7f1cd619",
  "programs/piv1/src/state/config.rs": "9d7a57c87b6781b943e0e9c13d08f73a21058d400b40540c485e6c596b042198",
  "programs/piv1/src/state_persistence.rs": "d26483f7e1b95a020623b85705ab0006cb884e85cff37a0666bafa83123401a5",
  "programs/piv1/src/instruction_boundary.rs": "d631db4329c72720cf6acaa4259cb7f244d7b84fa8db1266ad9a15881a3e773c",
  "programs/piv1/src/instruction_errors.rs": "3ae5001e0c90440e3f4384b4d94da67d35a20240877a462421daed8df5e8d7b7",
  "programs/piv1/src/events.rs": "8860d27171af88b8752841146d0d02915ec9dbe59083749bfdd2e4f86c620c44",
  "clients/cli/initializer-plan.cjs": "347d5ab436b3e8878fd337c53ef8636721ca64a6465bc86c06f2637921ac6f6f",
  "clients/cli/intake-plan.cjs": "77125b06ce0d1975f87f8d0201f2e77cca58ee6d41a1136bd99d6722a41f1a89",
  "clients/cli/initializer-plan.test.cjs": "7616bb36648e7d4fc835f80bbe27b0b67cd4a96a2a9f2540238b04e5acbbc1cb",
  "validation/genesis-transport/transport.cjs": "a7694ff07577988e96546a53008bc8ab85bbc0ae7cb9353dcfc25e6c5a3c3724",
  "validation/genesis-transport/production-initializer.cjs": "cdf326d32f764d55dc1bc30590a252cdffca069ad1efd2989aae36744c20553b",
  "programs/piv1/src/instructions/update_config.rs": "965c993e5fb86bd38e6b0d00a91c54817d9d49d87bec25760e8fb6c1137d0815",
  "programs/piv1/src/guardian_set_operations.rs": "6540b3490ec0f38a96f99ed138b87c125b3f3acd90679e61ad62d3010ceb25a3",
  "programs/piv1/src/kif_claim_accounts.rs": "9259221886dd4183b469d79d4860b17c1de7e274563ae07b65a532ecf8b08493",
  "clients/cli/pause-plan.cjs": "b64267ff36e1497517043356e942af35256bab0388e0fee677ca159633cade61"
});
const RETAINED_INTERFACE={
  "commit": "64af7330413d5c85cbbccfd8c27a05d45b6e666f",
  "root": "/tmp/piv1-squads-readonly-20260914-ng1dg59p",
  "pins": {
    "programs/squads_multisig_program/src/instructions/vault_transaction_create.rs": "ba6a40bb55bd6589fe9fca3c6f8a0660a2dddae2fa7cd1dd4771e04064ab8426",
    "programs/squads_multisig_program/src/instructions/vault_transaction_execute.rs": "ac9161ec101bc96a13e32a329f69734823df478d308718918f4f8608a5376d97",
    "programs/squads_multisig_program/src/utils/executable_transaction_message.rs": "698b0fbb690d6abb6a2d3269f6ec6a54e23a04cd2b4eb1000afd7f0a81da650d",
    "programs/squads_multisig_program/src/state/seeds.rs": "f062ac588cce48f7474775db6ee9d92e2e29e9931dfc6c7444530a0d908116a1",
    "programs/squads_multisig_program/src/state/multisig.rs": "4ad0764e559a0376ad39c9e4d84a106dd0ec8fc040cdf0bf223c55763f07c9a0",
    "sdk/multisig/idl/squads_multisig_program.json": "cb9a0a29040ec3853a5105c547ffc608bb0e3175d78dacf30d463aff939f9b9b"
  },
  "meaning": "Source/IDL interface only; no deployed-source equality or runtime archive requirement."
};
RETAINED_INTERFACE.bufferReferences={root:'/tmp/piv1-t223-squads-sources',pins:{'transaction_buffer_create.rs':'9fa902a9892612c22afd11cd44546fe05e58c9fb647ef6bda7b1dcb27d80ab50','transaction_buffer_extend.rs':'6fbd1f2ae16dfd22b19128fd959ffd979409458963578bc598c3fd293c528ea5','vault_transaction_create_from_buffer.rs':'94f7127e60bb26c394dbcd5a61b2ec97ed4861e38f6c9d231f4f7675bce7a59d'}};
const IDS=Object.freeze({...initializer.IDS,clock:'SysvarC1ock11111111111111111111111111111111'}),RESOURCE_POLICY=intake.RESOURCE_POLICY;
const TEMPLATE={schema:'piv1-guardian-sync-plan-v1',cluster:'testnet',action:'synchronize-one-guardian',programId:null,feePayer:null,rentPayer:null,nativeRentPayer:null,
  replacement:{guardianIndex:null,newGuardian:null},
  governance:{registryRevision:null,guardianKeys:Array(6).fill(null),currentMemberKeys:Array(6).fill(null),bufferIndex:null,createKey:null,multisigAddress:null,vaultIndex:null,transactionIndex:null,currentTransactionIndex:null,staleTransactionIndex:null,creator:null,executor:null,approvers:Array(4).fill(null),timeLockSeconds:null},
  transport:{recentBlockhash:null,lastValidBlockHeight:null,referenceSlot:null},resources:{computeUnits:null,heapBytes:null,loadedAccountBytes:null}};
const REPO=path.resolve(__dirname,'../..'),U64=(1n<<64n)-1n,pk=v=>new PublicKey(v),sha=b=>createHash('sha256').update(b).digest('hex');
const present=v=>v!==null&&v!==undefined,plain=v=>v!==null&&typeof v==='object'&&!Array.isArray(v)&&Object.getPrototypeOf(v)===Object.prototype;
const fail=(code,field)=>{const e=new Error(code);e.code=code;e.field=field;throw e;};
const le64=v=>{const b=Buffer.alloc(8);b.writeBigUInt64LE(BigInt(v));return b;},vec=b=>{const n=Buffer.alloc(4);n.writeUInt32LE(b.length);return Buffer.concat([n,b]);};
const pda=(program,seeds)=>{const [a,bump]=PublicKey.findProgramAddressSync(seeds,pk(program));return {address:a.toBase58(),bump};};
const reward=(program,g,revision,i)=>pda(program,[Buffer.from('guardian-reward'),pk(g).toBuffer(),le64(revision),Buffer.from([i])]);
const DISC=Object.freeze({vault_transaction_create:'30fa4ea8d0e2dad3',proposal_create:'dc3c49e01e6c4f9f',proposal_approve:'9025a488bcd82af8',vault_transaction_execute:'c208a15799a419ab',transaction_buffer_create:'f5c9716c253f1d59',transaction_buffer_extend:'e69d433805eef592',vault_transaction_create_from_buffer:'de36954457f630e7'});
const READINESS={publicReady:false,deploymentAuthorized:false,guardianSyncExecutionAuthorized:false,signerControlVerified:false,accountValidityVerified:false,currentAuthorityVerified:false,currentMembershipVerified:false,actualFourPublicApprovalsVerified:false,
  chainSnapshotVerified:false,resourceSufficiencyVerified:false,fundingSufficiencyVerified:false,fees:null,rent:null,funding:null,eligibility:null,timelockRelease:null,actionOutcome:null,m4Reservation:'OPEN_UNWAIVED',liveScope:'D043_CONSUMED_NO_GUARDIAN_SYNC_OR_NEW_SPENDING_AUTHORIZATION'};
const PRECONDITIONS=[
  'M5 preparation only/public NOT READY/PIV deployment NOT AUTHORIZED. D043 covered one completed external-Squads creation+return only; it is consumed and authorizes no guardian synchronization, external membership change or additional spending. H2 was drained. No actual identity, slot, index, funding source, activity or KIF anchor is chosen.',
  'Only public declarations and all-zero unsigned packets are emitted. No RPC/snapshot acquisition, private wallet/key access, key generation, signing, submission, confirmation, external membership rotation or live error diagnosis is implemented. Existing9mw multisig does not authenticate supplied declarations.',
  'This is only the typed PIV one-seat SYNCHRONIZATION AFTER a separately authorized and executed external Squads membership change. governance.guardianKeys and registryRevision describe the OLD CURRENT PIV slot order/revision; governance.currentMemberKeys describes the CURRENT post-change Squads member set, independent of array order. The current new set separately approves this exact PIV action. This boundary proves no historical old-set vote or persisted original-multisig continuity.',
  'PIV1UG01/version1 binds governance vault index, exact old PIV guardian slot, expected old revision and new guardian. Old revision must permit checked u64+1. The replacement key must not be in the old six. Replace only that slot and require set equality with the six declared current Squads members. Do not sort/remap old PIV slots to Squads order.',
  'The24 distinct native roles are Program,ProgramData,multisig,proposal,transaction,governance vault,Instructions,Config,old registry,six old rewards,Clock,six new rewards,native external rent payer,System. Writable exactly7,8,16..22; signers5 and22. The vault is readonly via Squads CPI; nativeRentPayer is writable and must sign at the top level. Old reward records remain readonly. Stored governance metadata remain readonly while outer proposal is globally writable.',
  'Current ProgramData loader authority dynamically determines the current governance vault/multisig. Runtime must authenticate exact Program/ProgramData headers, current six-member autonomous threshold4 Squads configuration/permissions, old Config/registry/reward tuple and the same actual Clock, then the typed exactly-one-seat mismatch. This establishes no old/current historical authority continuity. No pause, distribution-phase or activity gate is invented.',
  'The six new reward PDAs derive every slot at old revision+1 with only the selected key replaced. Each must be a distinct empty-data nonexecutable System account, possibly prefunded; none can alias custody, old/new guardian keys, current protocol bindings or any native role. Runtime allocates84 bytes and assigns each to PIV. Only six positive shortfalls max(current84-byte rent floor minus actual target balance,0) are paid, using checked sum. The external System-owned empty-data nativeRentPayer must retain the current zero-data rent floor. No supplied balance/quote/prefund or funding sufficiency is claimed.',
  'Success writes Config.guardian_registry_revision, registry revision/keys and six fresh reward records only. Existing guardians retain their prior recorded last_active_period; the newcomer has no invented activity. New records start with no old earned liability/counters; old earned records and their original guardian entitlement remain unchanged. Prepared round snapshots and frozen recipients/eligibility, debt and all economic ledgers remain unchanged. Native payer rent movement is separate from economic funds and from parent Squads state rent/network fees.',
  'Fresh current/stale declarations must bind direct create or create-from-buffer to actual current+1 without u64 overflow; creation increments the multisig index. Stop if another creation changes that next index before creation. Later current index may advance; require this action index<=actual current and>actual stale with still-current approved bytes, membership and permissions. New transaction/proposal/buffer targets and actual rent funding remain unverified.',
  'Use direct vault_transaction_create only when the full legacy packet fits1232. Otherwise canonical transaction_buffer_create/extend bind the full compact SHA256/size and exact chunks, followed by vault_transaction_create_from_buffer with six zero placeholder bytes. Buffer index must be explicitly supplied when needed, never chosen. Buffer closure refunds its rent to creator, not automatically to rentPayer. Parent buffer/transaction/proposal creation/reallocation costs and actual refund remain unknown; repeated steps or unrelated transaction creation require fresh state review.',
  'After exact transaction creation use proposal_create(isDraft=false), four separately signed distinct current-member approvals, then direct top-level vault_transaction_execute. Actual Approved status, valid current Vote permissions, rejection/cancellation state, nonstale index, runtime stack height2, Instructions sysvar, exact single child/no lookups/no ephemeral signer and released Clock/timelock must authenticate. Declarations are not votes or execution.',
  'Selected outer fee/rent/nativeRentPayer/creator/executor/approver direct signers must be on curve; no blanket on-curve rule is imposed on unselected old/current members or the new guardian. Permitted outer actors may overlap; every native role stays distinct and custody/native/reserved aliases are rejected. No top-level governance-vault signature is generated.',
  'Each standalone packet includes three explicit requested CU/heap/loaded-byte limits and no priority fee. Count every signature slot and require full legacy bytes<=1232 with no truncation. Requested limits prove no runtime sufficiency. No ALT or live operator transport is supplied. The shared reference blockhash is encoding context only: refresh independently after account creation/upload/votes/review/timelock waits. Future signed work requires separate explicit scope.'
];
function validateManifest(input){
  function walk(v,s,f){if(!present(v))return null;if(Array.isArray(s)){if(!Array.isArray(v)||v.length!==s.length)fail('array_shape',f);return Array.from({length:s.length},(_,i)=>walk(v[i],null,`${f}.${i}`));}
    if(plain(s)){if(!plain(v))fail('object_required',f||'manifest');if(Object.keys(v).some(k=>!Object.hasOwn(s,k)))fail('unknown_field',f||'manifest');return Object.fromEntries(Object.entries(s).map(([k,shape])=>[k,walk(v[k],shape,f?`${f}.${k}`:k)]));}return v;}
  if(!plain(input))fail('object_required','manifest');const m=walk(input,TEMPLATE,''),g=m.governance,r=m.replacement,check=(v,f,fn)=>{if(present(v))fn(v,f);};
  const key=(v,f,hash=false)=>{if(typeof v!=='string')fail('public_key_string',f);let a;try{a=pk(v);}catch{fail('invalid_public_key',f);}if(a.toBase58()!==v||!hash&&v===IDS.system)fail('noncanonical_or_zero_public_key',f);};
  const decimal=(v,f)=>{if(typeof v!=='string'||v.length>20||!/^(?:0|[1-9]\d*)$/.test(v))fail('decimal_u64_string_required',f);if(BigInt(v)>U64)fail('integer_range',f);};
  const small=(v,f,max)=>{if(!Number.isSafeInteger(v)||v<0||v>max)fail('integer_range',f);};
  for(const [f,want]of [['schema',TEMPLATE.schema],['cluster','testnet'],['action','synchronize-one-guardian']])check(m[f],f,v=>{if(v!==want)fail('unsupported_'+f,f);});
  for(const f of ['programId','feePayer','rentPayer','nativeRentPayer'])check(m[f],f,key);
  if(r){check(r.guardianIndex,'replacement.guardianIndex',(v,f)=>small(v,f,5));check(r.newGuardian,'replacement.newGuardian',key);}
  if(present(m.programId)&&Object.values(IDS).includes(m.programId))fail('program_identity_alias','programId');
  if(g){for(const f of ['createKey','multisigAddress','creator','executor'])check(g[f],`governance.${f}`,key);for(const f of ['guardianKeys','currentMemberKeys','approvers'])g[f]?.forEach((v,i)=>check(v,`governance.${f}.${i}`,key));
    for(const f of ['registryRevision','transactionIndex','currentTransactionIndex','staleTransactionIndex'])check(g[f],`governance.${f}`,decimal);
    check(g.bufferIndex,'governance.bufferIndex',(v,f)=>small(v,f,255));if(present(g.registryRevision)&&BigInt(g.registryRevision)===U64)fail('next_revision_overflow','governance.registryRevision');
    check(g.vaultIndex,'governance.vaultIndex',(v,f)=>small(v,f,255));check(g.timeLockSeconds,'governance.timeLockSeconds',(v,f)=>small(v,f,7776000));
    for(const f of ['guardianKeys','currentMemberKeys','approvers']){const values=(g[f]||[]).filter(present);if(new Set(values).size!==values.length)fail('duplicate_entry',`governance.${f}`);}
    if(present(g.transactionIndex)&&g.transactionIndex==='0')fail('zero_transaction_index','governance.transactionIndex');if(present(g.currentTransactionIndex)&&BigInt(g.currentTransactionIndex)===U64)fail('next_index_overflow','governance.currentTransactionIndex');
    if(present(g.transactionIndex)&&present(g.currentTransactionIndex)&&BigInt(g.transactionIndex)!==BigInt(g.currentTransactionIndex)+1n)fail('next_index_mismatch','governance.transactionIndex');
    if(present(g.staleTransactionIndex)&&present(g.currentTransactionIndex)&&BigInt(g.staleTransactionIndex)>BigInt(g.currentTransactionIndex))fail('stale_index_mismatch','governance.staleTransactionIndex');
    if(present(g.staleTransactionIndex)&&present(g.transactionIndex)&&BigInt(g.staleTransactionIndex)>=BigInt(g.transactionIndex))fail('stale_next_index','governance.staleTransactionIndex');
    const members=(g.currentMemberKeys||[]).filter(present);if(members.length===6)for(const [f,v]of [['creator',g.creator],['executor',g.executor],...(g.approvers||[]).map((v,i)=>[`approvers.${i}`,v])])if(present(v)&&!members.includes(v))fail('declared_current_member_required',`governance.${f}`);}
  const oldKeys=g?.guardianKeys||[],currentKeys=g?.currentMemberKeys||[];
  if(present(r?.newGuardian)&&oldKeys.includes(r.newGuardian))fail('replacement_already_old_member','replacement.newGuardian');
  const newKeys=oldKeys.map((value,i)=>present(r?.guardianIndex)?i===r.guardianIndex?r.newGuardian:value:null);
  if(currentKeys.length===6&&currentKeys.every(present)&&present(r?.newGuardian)&&!currentKeys.includes(r.newGuardian))fail('new_guardian_not_current','replacement.newGuardian');
  if(present(r?.guardianIndex)){
    const removed=oldKeys[r.guardianIndex];if(present(removed)&&currentKeys.includes(removed))fail('removed_guardian_still_current','governance.currentMemberKeys');
    if(newKeys.length===6&&newKeys.every(present)){
      for(const value of currentKeys.filter(present))if(!newKeys.includes(value))fail('current_set_not_one_seat_replacement','governance.currentMemberKeys');
      const selected=[g?.creator,g?.executor,...(g?.approvers||[])].filter(present);if(selected.some(value=>!newKeys.includes(value)))fail('declared_replacement_member_required','governance');
    }
    if(currentKeys.length===6&&currentKeys.every(present))for(const value of newKeys.filter(present))if(!currentKeys.includes(value))fail('current_set_not_one_seat_replacement','governance.currentMemberKeys');
  }
  if(oldKeys.length===6&&oldKeys.every(present)&&currentKeys.length===6&&currentKeys.every(present)){
    const added=currentKeys.filter(value=>!oldKeys.includes(value)),removed=oldKeys.filter(value=>!currentKeys.includes(value));
    if(added.length!==1||removed.length!==1||present(r?.newGuardian)&&added[0]!==r.newGuardian||present(r?.guardianIndex)&&removed[0]!==oldKeys[r.guardianIndex])fail('current_set_not_one_seat_replacement','governance.currentMemberKeys');
  }
  if(m.transport){check(m.transport.recentBlockhash,'transport.recentBlockhash',(v,f)=>key(v,f,true));for(const f of ['lastValidBlockHeight','referenceSlot'])check(m.transport[f],`transport.${f}`,decimal);}
  if(m.resources)for(const [f,limits]of Object.entries(RESOURCE_POLICY))if(f!=='meaning')check(m.resources[f],`resources.${f}`,(v,field)=>{if(!Number.isSafeInteger(v)||v<limits.min||v>limits.max||limits.multiple&&v%limits.multiple!==0)fail('unsupported_resource_request',field);});
  const fixed=present(m.programId)?intake.derived(m.programId):null,programData=present(m.programId)?pda(IDS.loader,[pk(m.programId).toBuffer()]):null;
  const ms=present(g?.createKey)?pda(IDS.squads,[Buffer.from('multisig'),Buffer.from('multisig'),pk(g.createKey).toBuffer()]):null;
  if(ms&&present(g?.multisigAddress)&&g.multisigAddress!==ms.address)fail('multisig_seed_mismatch','governance.multisigAddress');
  const declaredMs=ms?.address??g?.multisigAddress,vault=present(declaredMs)&&present(g?.vaultIndex)?pda(IDS.squads,[Buffer.from('multisig'),pk(declaredMs).toBuffer(),Buffer.from('vault'),Buffer.from([g.vaultIndex])]):null;
  const txSeeds=present(declaredMs)&&present(g?.transactionIndex)?[Buffer.from('multisig'),pk(declaredMs).toBuffer(),Buffer.from('transaction'),le64(g.transactionIndex)]:null;
  const transaction=txSeeds?pda(IDS.squads,txSeeds):null,proposal=txSeeds?pda(IDS.squads,[...txSeeds,Buffer.from('proposal')]):null;
  const rewards=fixed&&present(g?.registryRevision)?(g.guardianKeys||[]).map((v,i)=>present(v)?reward(m.programId,v,g.registryRevision,i):null):[];
  const newRevision=present(g?.registryRevision)?(BigInt(g.registryRevision)+1n).toString():null,newRewards=fixed&&present(newRevision)?newKeys.map((value,i)=>present(value)?reward(m.programId,value,newRevision,i):null):[];
  const buffer=present(declaredMs)&&present(g?.creator)&&present(g?.bufferIndex)?pda(IDS.squads,[Buffer.from('multisig'),pk(declaredMs).toBuffer(),Buffer.from('transaction_buffer'),pk(g.creator).toBuffer(),Buffer.from([g.bufferIndex])]):null;
  const knownNative=[m.programId,programData?.address,declaredMs,proposal?.address,transaction?.address,vault?.address,IDS.instructions,fixed?.config,fixed?.guardian_registry,...rewards.map(r=>r?.address),IDS.clock,...newRewards.map(r=>r?.address),m.nativeRentPayer,IDS.system].filter(present);
  if(new Set(knownNative).size!==knownNative.length)fail('native_role_alias','manifest');
  const protectedKeys=new Set([...Object.values(IDS),...(fixed?Object.values(fixed):[]),...([m.programId,declaredMs,programData?.address,vault?.address,transaction?.address,proposal?.address].filter(present)),...rewards.filter(present).map(v=>v.address),...newRewards.filter(present).map(v=>v.address),...(buffer?[buffer.address]:[])]);
  if(buffer&&knownNative.includes(buffer.address))fail('buffer_native_alias','governance.bufferIndex');
  if(present(g?.multisigAddress)&&[...Object.values(IDS),m.programId,...(fixed?Object.values(fixed):[]),programData?.address].includes(g.multisigAddress))fail('governance_native_alias','governance.multisigAddress');
  const actors=[['feePayer',m.feePayer],['rentPayer',m.rentPayer],['nativeRentPayer',m.nativeRentPayer],['replacement.newGuardian',r?.newGuardian],['governance.createKey',g?.createKey],...(g?.guardianKeys||[]).map((v,i)=>[`governance.guardianKeys.${i}`,v]),...(g?.currentMemberKeys||[]).map((v,i)=>[`governance.currentMemberKeys.${i}`,v]),['governance.creator',g?.creator],['governance.executor',g?.executor],...(g?.approvers||[]).map((v,i)=>[`governance.approvers.${i}`,v])];
  for(const [f,v]of actors)if(present(v)&&protectedKeys.has(v))fail('actor_protected_alias',f);
  for(const [f,v]of actors.filter(([f])=>!f.startsWith('governance.guardianKeys.')&&!f.startsWith('governance.currentMemberKeys.')&&f!=='governance.createKey'&&f!=='replacement.newGuardian'))if(present(v)&&!PublicKey.isOnCurve(pk(v).toBytes()))fail('unsupported_off_curve_direct_signer',f);
  const leaves=(shape,prefix='')=>Object.entries(shape).flatMap(([k,v])=>{const f=prefix?`${prefix}.${k}`:k;return Array.isArray(v)?v.map((_,i)=>`${f}.${i}`):plain(v)?leaves(v,f):[f];});
  return {manifest:m,fixed,programData,multisig:ms,vault,transaction,proposal,rewards,newKeys,newRevision,newRewards,buffer,missing:leaves(TEMPLATE).filter(f=>f!=='governance.bufferIndex').filter(f=>f.split('.').reduce((v,k)=>v?.[k],m)==null)};
}
function budgets(r){const loaded=Buffer.alloc(5);loaded[0]=4;loaded.writeUInt32LE(r.loadedAccountBytes,1);return [ComputeBudgetProgram.setComputeUnitLimit({units:r.computeUnits}),ComputeBudgetProgram.requestHeapFrame({bytes:r.heapBytes}),new TransactionInstruction({programId:pk(IDS.computeBudget),keys:[],data:loaded})];}
const meta=(address,signer=false,writable=false)=>({pubkey:pk(address),isSigner:signer,isWritable:writable});
const instruction=(name,keys,args=Buffer.alloc(0))=>new TransactionInstruction({programId:pk(IDS.squads),keys,data:Buffer.concat([Buffer.from(DISC[name],'hex'),args])});
function creationPlan(v,compact,base,direct){
  const m=v.manifest,g=m.governance,full=ix=>[...budgets(m.resources),ix],size=ix=>t.legacyWireSize(initializer.compile({m},full(ix))),directWireBytes=size(direct);
  if(directWireBytes<=1232)return {mode:'DIRECT',directWireBytes,chunks:[],phases:[{name:'transaction_create',ix:direct}]};
  if(!present(g.bufferIndex))return {missing:true};
  const makeCreate=chunk=>instruction('transaction_buffer_create',[meta(v.multisig.address),meta(v.buffer.address,false,true),meta(g.creator,true),meta(m.rentPayer,true,true),meta(IDS.system)],Buffer.concat([Buffer.from([g.bufferIndex,g.vaultIndex]),Buffer.from(sha(compact),'hex'),Buffer.from([compact.length&255,compact.length>>8]),vec(chunk)]));
  const makeExtend=chunk=>instruction('transaction_buffer_extend',[meta(v.multisig.address),meta(v.buffer.address,false,true),meta(g.creator,true)],vec(chunk));
  const fit=(remaining,make)=>{for(let length=Math.min(800,remaining.length);length>0;length--){const chunk=remaining.subarray(0,length);if(size(make(chunk))<=1232)return chunk;}fail('no_fitting_buffer_chunk','governance.bufferIndex');};
  const first=fit(compact,makeCreate),chunks=[first],phases=[{name:'buffer_create',ix:makeCreate(first)}];let offset=first.length;
  while(offset<compact.length){const chunk=fit(compact.subarray(offset),makeExtend);phases.push({name:`buffer_extend_${chunks.length}`,ix:makeExtend(chunk)});chunks.push(chunk);offset+=chunk.length;}
  assert.deepEqual(Buffer.concat(chunks),compact);
  phases.push({name:'transaction_from_buffer',ix:instruction('vault_transaction_create_from_buffer',[...base,meta(v.buffer.address,false,true),meta(g.creator,true,true)],Buffer.concat([Buffer.from([g.vaultIndex,0]),vec(Buffer.alloc(6)),Buffer.from([0])]))});
  return {mode:'BUFFER',directWireBytes,chunks:chunks.map(c=>c.length),phases};
}
function planManifest(input){
  const v=validateManifest(input),m=v.manifest,g=m.governance,common={schema:TEMPLATE.schema,sourceBaseline:BASELINE,action:m.action,targetReplacement:m.replacement,missing:v.missing,readiness:{...READINESS},resourcePolicy:RESOURCE_POLICY,retainedInterface:RETAINED_INTERFACE,preconditions:[...PRECONDITIONS]};
  if(v.missing.length)return {...common,status:'INCOMPLETE',plan:null};
  const addresses=[m.programId,v.programData.address,v.multisig.address,v.proposal.address,v.transaction.address,v.vault.address,IDS.instructions,v.fixed.config,v.fixed.guardian_registry,...v.rewards.map(r=>r.address),IDS.clock,...v.newRewards.map(r=>r.address),m.nativeRentPayer,IDS.system];
  const names=['program','program_data','multisig','proposal','transaction','governance_vault','instructions','config','guardian_registry',...Array.from({length:6},(_,i)=>`old_guardian_reward_${i}`),'clock',...Array.from({length:6},(_,i)=>`new_guardian_reward_${i}`),'native_rent_payer','system'];
  if(new Set(addresses).size!==24)fail('native_role_alias','manifest');const roles=addresses.map((address,i)=>({role:names[i],address,signer:i===5||i===22,writable:[7,8,16,17,18,19,20,21,22].includes(i)}));
  const data=Buffer.concat([Buffer.from('PIV1UG01'),Buffer.from([1,g.vaultIndex,m.replacement.guardianIndex]),le64(g.registryRevision),pk(m.replacement.newGuardian).toBuffer()]),inner=new TransactionInstruction({programId:pk(m.programId),keys:roles.map(r=>meta(r.address,r.signer,r.writable)),data});
  const compact=t.compactMessage({profile:'public-guardian-sync-plan',program:pk(m.programId),inner}),stored=t.decodeCompact(compact);assert.equal(compact.length,853);assert.equal(stored.keys.length,24);assert.deepEqual([...compact.subarray(0,4)],[2,1,8,24]);
  const base=[meta(v.multisig.address,false,true),meta(v.transaction.address,false,true),meta(g.creator,true),meta(m.rentPayer,true,true),meta(IDS.system)];
  const create=instruction('vault_transaction_create',base,Buffer.concat([Buffer.from([g.vaultIndex,0]),vec(compact),Buffer.from([0])]));assert.equal(create.data.length,868);
  const proposalCreate=instruction('proposal_create',[meta(v.multisig.address),meta(v.proposal.address,false,true),meta(g.creator,true),meta(m.rentPayer,true,true),meta(IDS.system)],Buffer.concat([le64(g.transactionIndex),Buffer.from([0])]));
  const approvals=g.approvers.map((a,i)=>({name:`approve_${i+1}`,ix:instruction('proposal_approve',[meta(v.multisig.address),meta(a,true,true),meta(v.proposal.address,false,true)],Buffer.from([0]))}));
  const execute=instruction('vault_transaction_execute',[meta(v.multisig.address),meta(v.proposal.address,false,true),meta(v.transaction.address),meta(g.executor,true),...stored.keys.map(k=>meta(k.pubkey.toBase58(),k.isSigner&&!k.pubkey.equals(pk(v.vault.address)),k.isWritable))]);
  const creation=creationPlan(v,compact,base,create);if(creation.missing)return {...common,status:'INCOMPLETE',missing:[...common.missing,'governance.bufferIndex'],plan:null};
  const phases=[...creation.phases,{name:'proposal_create',ix:proposalCreate},...approvals,{name:'execute',ix:execute}],packets=phases.map(({name,ix})=>initializer.packet({m},name,[...budgets(m.resources),ix]));
  assert(packets.every(p=>p.wireBytes<=1232));assert(packets.every(p=>!p.requiredSigners.includes(v.vault.address)));
  return {...common,status:'OFFLINE_PLAN_ONLY',declarationSha256:sha(Buffer.from(JSON.stringify(m))),plan:{program:m.programId,feePayer:m.feePayer,rentPayer:m.rentPayer,nativeRentPayer:m.nativeRentPayer,desiredReplacement:{...m.replacement},creation:{mode:creation.mode,directWireBytes:creation.directWireBytes,buffer:creation.mode==='BUFFER'?v.buffer:null,chunks:creation.chunks,bufferRefundRecipient:creation.mode==='BUFFER'?g.creator:null},roles,native:{bytes:data.length,hex:data.toString('hex'),sha256:sha(data)},
    derived:{programData:v.programData,multisig:v.multisig,vault:v.vault,transaction:v.transaction,proposal:v.proposal,config:v.fixed.config,guardianRegistry:v.fixed.guardian_registry,oldRewards:v.rewards,newRewards:v.newRewards,newRevision:v.newRevision,newGuardianKeys:v.newKeys},
    governanceDeclaration:{...g,currentSetAuthenticated:false,inputKeysMeaning:'OLD_PIV_SLOT_ORDER_AND_SEPARATE_CURRENT_SQUADS_MEMBER_SET',declaredFourApproversAreNotActualVotes:true},
    compact:{bytes:compact.length,hex:compact.toString('hex'),sha256:sha(compact),storedLookupCount:0,ephemeralSignerCount:0,programIndex:stored.programIndex,accountIndices:stored.indices,keys:stored.keys.map(k=>({address:k.pubkey.toBase58(),signer:k.isSigner,writable:k.isWritable}))},
    sequence:phases.map((p,i)=>({step:i+1,packet:p.name,status:'UNSIGNED_NOT_EXECUTED',guard:p.name==='execute'?'ACTUAL_CURRENT_NEW_SET_APPROVED_NONSTALE_RELEASED_TIMELOCK_DIRECT_SQUADS':p.name.startsWith('approve_')?'CURRENT_NEW_SET_DISTINCT_VOTER_ACTIVE_PROPOSAL':p.name==='proposal_create'?'CREATED_EXACT_TRANSACTION_ACTIVE_PROPOSAL':'FRESH_TARGETS_EXACT_BYTES_NEXT_INDEX_AT_TRANSACTION_CREATION'})),
    intendedEvent:{name:'GuardianSetUpdated',observed:false,config:v.fixed.config,guardianRegistry:v.fixed.guardian_registry,multisig:v.multisig.address,transactionIndex:g.transactionIndex,previousRevision:g.registryRevision,registryRevision:v.newRevision,guardianIndex:m.replacement.guardianIndex,previousGuardian:g.guardianKeys[m.replacement.guardianIndex],newGuardian:m.replacement.newGuardian,rentPayer:m.nativeRentPayer,fundedRentLamports:null},requestedResources:m.resources,referenceContext:{...m.transport,verified:false,usage:'ENCODING_ONLY_REFRESH_INDEPENDENTLY_EACH_PHASE'},packets}};
}
function verifySources(){const inherited=initializer.verifySources();for(const [file,expected]of Object.entries(SOURCE_PINS))assert.equal(sha(fs.readFileSync(path.join(REPO,file))),expected,`source binding: ${file}`);return {node:inherited.node,web3:inherited.web3,pins:SOURCE_PINS,initializerPins:inherited.pins};}
function main(args){const sourceCheck=verifySources();if(args.length===1&&args[0]==='--template'){process.stdout.write(JSON.stringify(TEMPLATE,null,2)+'\n');return;}if(args.length!==2||args[0]!=='--manifest')fail('unsupported_arguments','arguments');
  const filename=path.resolve(args[1]),stat=fs.statSync(filename);if(!stat.isFile()||stat.size>65536)fail('public_manifest_file_required','manifest');const bytes=fs.readFileSync(filename);if(bytes.length>65536)fail('manifest_size','manifest');const report=planManifest(initializer.parseManifest(bytes.toString('utf8')));report.manifestInputSha256=sha(bytes);report.sourceCheck=sourceCheck;process.stdout.write(JSON.stringify(report,null,2)+'\n');}
module.exports={BASELINE,SOURCE_PINS,RETAINED_INTERFACE,TEMPLATE,IDS,RESOURCE_POLICY,validateManifest,planManifest,verifySources};
if(require.main===module){try{main(process.argv.slice(2));}catch(e){process.stderr.write(JSON.stringify({status:'INVALID_OR_UNSUPPORTED_GUARDIAN_SYNC_PLAN',code:e.code||'plan_validation_failed',field:e.field||'manifest'})+'\n');process.exitCode=1;}}
