'use strict';
// Exact unsigned direct-Squads recipient-update sequence. Public declarations only.
const assert=require('node:assert/strict'),fs=require('node:fs'),path=require('node:path');
const {createHash}=require('node:crypto'),initializer=require('./initializer-plan.cjs'),intake=require('./intake-plan.cjs');
const t=require('../../validation/genesis-transport/transport.cjs'),{PublicKey,TransactionInstruction,ComputeBudgetProgram}=require(t.SDK);
const BASELINE='824342de0bfd156dd3c6b41a6eed9744a3a2c1e7';
const SOURCE_PINS=Object.freeze({
  "programs/piv1/src/instructions/update_config.rs": "e456a3601b3c3c2656a8ae8c67c059711171cd31d14c647487a96c2429f7f3b7",
  "programs/piv1/src/recipient_operations.rs": "74ed0abadfb03b7a3c3f04a8fbc8f14b63e2a528219fc1314593fa6934fd8f43",
  "programs/piv1/src/squads_execution.rs": "589252b8357d76e9af61d4f4aaed7e7d54692db63bddd98ea807cee6357321fd",
  "programs/piv1/src/squads_accounts.rs": "b5ccacf047d86666eadaf568019ac2df7318fb954d62ae87866ba284324e518b",
  "programs/piv1/src/guardian_clock_accounts.rs": "5cc36b79d429bb3e1818466d429259474c8a26d1c1288da16a71f22033f84946",
  "programs/piv1/src/accounts.rs": "08e3d28971160910088a40d74215d15a9c2d07b11142e28c4fb5162d725f1c80",
  "programs/piv1/src/state/guardian.rs": "ce762257b8025789f8ec43351ce166f0a5f7ad0bb6f05b4c9515173d7f1cd619",
  "programs/piv1/src/state/config.rs": "9d7a57c87b6781b943e0e9c13d08f73a21058d400b40540c485e6c596b042198",
  "programs/piv1/src/state_persistence.rs": "d26483f7e1b95a020623b85705ab0006cb884e85cff37a0666bafa83123401a5",
  "programs/piv1/src/instruction_boundary.rs": "0e082adeebe7cabe5f5e14f69b244c2f1464c3c2f4eacd0077173a9aad91ab9f",
  "programs/piv1/src/instruction_errors.rs": "3ae5001e0c90440e3f4384b4d94da67d35a20240877a462421daed8df5e8d7b7",
  "programs/piv1/src/events.rs": "8860d27171af88b8752841146d0d02915ec9dbe59083749bfdd2e4f86c620c44",
  "clients/cli/initializer-plan.cjs": "92c13a4205f4557c74f55b03ee2dfea458592938228a7a959f5a2baf35bbeb83",
  "clients/cli/intake-plan.cjs": "ae4bb850ca4a9c8432830a6d0e60f5398fb895f3b137ff428120dec6568b253c",
  "clients/cli/initializer-plan.test.cjs": "16c642256aac6183a5e65b0cb99494bd8255b3f19b1b5f564b2124dfc2880e58",
  "validation/genesis-transport/transport.cjs": "a7694ff07577988e96546a53008bc8ab85bbc0ae7cb9353dcfc25e6c5a3c3724",
  "validation/genesis-transport/production-initializer.cjs": "c9cf0e28fa21d13e8b23b697023ad807d40f609cf46473c5019e17c46f8383a7",
  "clients/cli/pause-plan.cjs": "4c07b2c92d99d1c81aa97b26780e07d3107449d6f71dfa023669aea514b5a1a8"
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
const IDS=Object.freeze({...initializer.IDS,clock:'SysvarC1ock11111111111111111111111111111111'}),RESOURCE_POLICY=intake.RESOURCE_POLICY;
const TEMPLATE={schema:'piv1-recipient-plan-v2',cluster:'testnet',action:'update-recipients',programId:null,feePayer:null,rentPayer:null,
  recipients:{htfpRecipient:null,teamOwnerRecipient:null},
  governance:{registryRevision:null,guardianKeys:Array(6).fill(null),createKey:null,multisigAddress:null,vaultIndex:null,transactionIndex:null,currentTransactionIndex:null,staleTransactionIndex:null,creator:null,executor:null,approvers:Array(4).fill(null),timeLockSeconds:null},
  transport:{recentBlockhash:null,lastValidBlockHeight:null,referenceSlot:null},resources:{computeUnits:null,heapBytes:null,loadedAccountBytes:null}};
const REPO=path.resolve(__dirname,'../..'),U64=(1n<<64n)-1n,pk=v=>new PublicKey(v),sha=b=>createHash('sha256').update(b).digest('hex');
const present=v=>v!==null&&v!==undefined,plain=v=>v!==null&&typeof v==='object'&&!Array.isArray(v)&&Object.getPrototypeOf(v)===Object.prototype;
const fail=(code,field)=>{const e=new Error(code);e.code=code;e.field=field;throw e;};
const le64=v=>{const b=Buffer.alloc(8);b.writeBigUInt64LE(BigInt(v));return b;},vec=b=>{const n=Buffer.alloc(4);n.writeUInt32LE(b.length);return Buffer.concat([n,b]);};
const pda=(program,seeds)=>{const [a,bump]=PublicKey.findProgramAddressSync(seeds,pk(program));return {address:a.toBase58(),bump};};
const reward=(program,g,revision,i)=>pda(program,[Buffer.from('guardian-reward'),pk(g).toBuffer(),le64(revision),Buffer.from([i])]);
const DISC=Object.freeze({vault_transaction_create:'30fa4ea8d0e2dad3',proposal_create:'dc3c49e01e6c4f9f',proposal_approve:'9025a488bcd82af8',vault_transaction_execute:'c208a15799a419ab'});
const READINESS={publicReady:false,deploymentAuthorized:false,recipientUpdateExecutionAuthorized:false,signerControlVerified:false,accountValidityVerified:false,currentAuthorityVerified:false,currentMembershipVerified:false,actualFourPublicApprovalsVerified:false,
  chainSnapshotVerified:false,resourceSufficiencyVerified:false,fundingSufficiencyVerified:false,fees:null,rent:null,funding:null,eligibility:null,timelockRelease:null,actionOutcome:null,m4Reservation:'OPEN_UNWAIVED',liveScope:'D043_CONSUMED_NO_RECIPIENT_UPDATE_OR_NEW_SPENDING_AUTHORIZATION'};
const PRECONDITIONS=[
  'M5 preparation only/public NOT READY/PIV deployment NOT AUTHORIZED. D043 covered one completed external-Squads creation+return only; it is consumed and authorizes no recipient update or additional spending. The dedicated H2 payer was drained; this tool neither selects it nor supplies new funding authority.',
  'Only public declarations and all-zero unsigned packets are emitted. No RPC/snapshot acquisition, private wallet/key access, key generation, signing, submission, confirmation or live error diagnosis is implemented. Existing9mw multisig does not authenticate any supplied current manifest.',
  'Exact PIV1UR02/version2/governance vault index replaces only current Config.htfp_recipient and Config.team_owner_recipient. An active distribution keeps its separately frozen recipients. Native update performs no CPI, economic/rent movement, account creation, activity/period/cadence/pause/HWM/KIF change. Separate Squads transaction/proposal creation allocates its state accounts from the declared rent payer; rents and network fees remain unknown.',
  'Native roles are exact18: Program,ProgramData,multisig,proposal,transaction,vault,Instructions,Config,registry,six slot-ordered current rewards,Clock,new HTFP recipient,new Team Owner recipient. All18 must be distinct. Only Config is writable; governance vault is the only readonly native signer, supplied by Squads CPI. Governance metadata and recipients remain readonly in stored/native metas even though the outer proposal is globally writable.',
  'Each recipient is an independently authorized public address. Owner/data/curve/funding are not admission profiles; exact configured/frozen keys, protected-role isolation and runtime writable/rent rules remain. Native payment performs only a System transfer; downstream accounting belongs to the receiver.',
  'The canonical Program/ProgramData loader headers dynamically determine the CURRENT upgrade-authority vault and multisig. Authenticate them afresh with initialized/rent-backed Config/registry and six revision/guardian/slot reward tuples at the same runtime Clock. This is no immutable original-multisig or historical-continuity guarantee. Match the current six-member Squads set without sorting/remapping PIV slots. All six need Vote permission; creator needs Initiate, executor Execute, and four distinct approvers current Vote membership. No activity/heartbeat approval condition is invented.',
  'Declarations of current index/stale index must be fresh: direct create uses actual current+1, declared next index must fit u64 and be greater than stale. Create increments multisig index. Before creation stop if another creation invalidates declared current+1. Later global current index may advance legitimately: re-read and require this action index<=actual current and>actual stale, with still-current membership/authority and unchanged approved action. Transaction/proposal targets must be valid new accounts for creation; this planner proves no absence or rent sufficiency.',
  'Sequence is direct vault_transaction_create, proposal_create with isDraft=false (already Active, no separate activation packet), four separately signed approvals, then direct top-level vault_transaction_execute. Threshold4/configAuthorityNone and current permissions must hold. Active votes must actually reach Approved with four unique current approvals and acceptable rejection/cancellation state. The packets are declarations, not approvals or an executed proposal.',
  'Execution requires source-authenticated Approved status/index>current stale and index<=current multisig index, exact single-instruction stored message, no ephemeral signer/lookups, runtime stack height2 and Instructions sysvar proving direct top-level Squads execute. Upstream stale execution permissiveness does not waive stricter PIV nonstale checks.',
  'Actual runtime Clock must release configured timelock from Approved timestamp; no caller timestamp/eligibility time is accepted or calculated. The same reference blockhash is encoding context for each standalone packet, not a shared durable validity window: refresh independently after account creation, voting, human review and timelock waits.',
  'Outer fee/rent payers may overlap each other or declared guardian actors. Only selected direct outer signer roles must be on curve; runtime PDA/CPI signer-wallet compositions are unsupported by this planner, not forbidden by all production paths. Reserved program/native/PIV PDA/recipient aliases are unsupported. No vault top-level signature or wrapper is generated.',
  'Every phase carries three explicit requested CU/heap/loaded-byte bounds, no priority-fee instruction. Actual resources/fees/rent/prefunds/wallet budget/control remain unknown. Full legacy packets count every zero signature slot and must fit1232 bytes. No ALT/buffer/provisioning/live operator transport is supplied.',
  'RecipientsUpdated is an intended event only: previous current recipients and actual outcome are unknown. No prior recipient choice is inferred. Current Config replacement does not retarget frozen distributions, authorize treasury sweeps, fund recipient accounts or waive any economic handler checks. A future exact signed package needs separate explicit scope coverage.'
];
function validateManifest(input){
  function walk(v,s,f){if(!present(v))return null;if(Array.isArray(s)){if(!Array.isArray(v)||v.length!==s.length)fail('array_shape',f);return Array.from({length:s.length},(_,i)=>walk(v[i],null,`${f}.${i}`));}
    if(plain(s)){if(!plain(v))fail('object_required',f||'manifest');if(Object.keys(v).some(k=>!Object.hasOwn(s,k)))fail('unknown_field',f||'manifest');return Object.fromEntries(Object.entries(s).map(([k,shape])=>[k,walk(v[k],shape,f?`${f}.${k}`:k)]));}return v;}
  if(!plain(input))fail('object_required','manifest');const m=walk(input,TEMPLATE,''),g=m.governance,r=m.recipients,check=(v,f,fn)=>{if(present(v))fn(v,f);};
  const key=(v,f,hash=false)=>{if(typeof v!=='string')fail('public_key_string',f);let a;try{a=pk(v);}catch{fail('invalid_public_key',f);}if(a.toBase58()!==v||!hash&&v===IDS.system)fail('noncanonical_or_zero_public_key',f);};
  const decimal=(v,f)=>{if(typeof v!=='string'||v.length>20||!/^(?:0|[1-9]\d*)$/.test(v))fail('decimal_u64_string_required',f);if(BigInt(v)>U64)fail('integer_range',f);};
  const small=(v,f,max)=>{if(!Number.isSafeInteger(v)||v<0||v>max)fail('integer_range',f);};
  for(const [f,want]of [['schema',TEMPLATE.schema],['cluster','testnet'],['action','update-recipients']])check(m[f],f,v=>{if(v!==want)fail('unsupported_'+f,f);});
  for(const f of ['programId','feePayer','rentPayer'])check(m[f],f,key);
  if(r)for(const f of ['htfpRecipient','teamOwnerRecipient'])check(r[f],`recipients.${f}`,key);
  if(present(m.programId)&&Object.values(IDS).includes(m.programId))fail('program_identity_alias','programId');
  if(g){for(const f of ['createKey','multisigAddress','creator','executor'])check(g[f],`governance.${f}`,key);for(const f of ['guardianKeys','approvers'])g[f]?.forEach((v,i)=>check(v,`governance.${f}.${i}`,key));
    for(const f of ['registryRevision','transactionIndex','currentTransactionIndex','staleTransactionIndex'])check(g[f],`governance.${f}`,decimal);
    check(g.vaultIndex,'governance.vaultIndex',(v,f)=>small(v,f,255));check(g.timeLockSeconds,'governance.timeLockSeconds',(v,f)=>small(v,f,7776000));
    for(const f of ['guardianKeys','approvers']){const values=(g[f]||[]).filter(present);if(new Set(values).size!==values.length)fail('duplicate_entry',`governance.${f}`);}
    if(present(g.transactionIndex)&&g.transactionIndex==='0')fail('zero_transaction_index','governance.transactionIndex');if(present(g.currentTransactionIndex)&&BigInt(g.currentTransactionIndex)===U64)fail('next_index_overflow','governance.currentTransactionIndex');
    if(present(g.transactionIndex)&&present(g.currentTransactionIndex)&&BigInt(g.transactionIndex)!==BigInt(g.currentTransactionIndex)+1n)fail('next_index_mismatch','governance.transactionIndex');
    if(present(g.staleTransactionIndex)&&present(g.currentTransactionIndex)&&BigInt(g.staleTransactionIndex)>BigInt(g.currentTransactionIndex))fail('stale_index_mismatch','governance.staleTransactionIndex');
    if(present(g.staleTransactionIndex)&&present(g.transactionIndex)&&BigInt(g.staleTransactionIndex)>=BigInt(g.transactionIndex))fail('stale_next_index','governance.staleTransactionIndex');
    const members=(g.guardianKeys||[]).filter(present);if(members.length===6)for(const [f,v]of [['creator',g.creator],['executor',g.executor],...(g.approvers||[]).map((v,i)=>[`approvers.${i}`,v])])if(present(v)&&!members.includes(v))fail('declared_current_member_required',`governance.${f}`);}
  if(m.transport){check(m.transport.recentBlockhash,'transport.recentBlockhash',(v,f)=>key(v,f,true));for(const f of ['lastValidBlockHeight','referenceSlot'])check(m.transport[f],`transport.${f}`,decimal);}
  if(m.resources)for(const [f,limits]of Object.entries(RESOURCE_POLICY))if(f!=='meaning')check(m.resources[f],`resources.${f}`,(v,field)=>{if(!Number.isSafeInteger(v)||v<limits.min||v>limits.max||limits.multiple&&v%limits.multiple!==0)fail('unsupported_resource_request',field);});
  const fixed=present(m.programId)?intake.derived(m.programId):null,programData=present(m.programId)?pda(IDS.loader,[pk(m.programId).toBuffer()]):null;
  const ms=present(g?.createKey)?pda(IDS.squads,[Buffer.from('multisig'),Buffer.from('multisig'),pk(g.createKey).toBuffer()]):null;
  if(ms&&present(g?.multisigAddress)&&g.multisigAddress!==ms.address)fail('multisig_seed_mismatch','governance.multisigAddress');
  const declaredMs=ms?.address??g?.multisigAddress,vault=present(declaredMs)&&present(g?.vaultIndex)?pda(IDS.squads,[Buffer.from('multisig'),pk(declaredMs).toBuffer(),Buffer.from('vault'),Buffer.from([g.vaultIndex])]):null;
  const recipientAddresses={htfp:present(r?.htfpRecipient)?{address:r.htfpRecipient}:null,teamOwner:present(r?.teamOwnerRecipient)?{address:r.teamOwnerRecipient}:null};
  const knownRecipients=[r?.htfpRecipient,r?.teamOwnerRecipient].filter(present);
  const txSeeds=present(declaredMs)&&present(g?.transactionIndex)?[Buffer.from('multisig'),pk(declaredMs).toBuffer(),Buffer.from('transaction'),le64(g.transactionIndex)]:null;
  const transaction=txSeeds?pda(IDS.squads,txSeeds):null,proposal=txSeeds?pda(IDS.squads,[...txSeeds,Buffer.from('proposal')]):null;
  const rewards=fixed&&present(g?.registryRevision)?(g.guardianKeys||[]).map((v,i)=>present(v)?reward(m.programId,v,g.registryRevision,i):null):[];
  const knownNative=[m.programId,programData?.address,declaredMs,proposal?.address,transaction?.address,vault?.address,IDS.instructions,fixed?.config,fixed?.guardian_registry,...rewards.map(r=>r?.address),IDS.clock,...knownRecipients].filter(present);
  if(new Set(knownNative).size!==knownNative.length)fail('native_role_alias','manifest');
  const protectedKeys=new Set([...Object.values(IDS),...(fixed?Object.values(fixed):[]),...([m.programId,declaredMs,programData?.address,vault?.address,transaction?.address,proposal?.address].filter(present)),...rewards.filter(present).map(v=>v.address),...knownRecipients]);
  for(const [i,value]of knownRecipients.entries())if([...Object.values(IDS),...(fixed?Object.values(fixed):[]),].includes(value))fail('recipient_reserved_alias',`recipients.${i}`);
  if(present(g?.multisigAddress)&&[...Object.values(IDS),m.programId,...(fixed?Object.values(fixed):[]),programData?.address].includes(g.multisigAddress))fail('governance_native_alias','governance.multisigAddress');
  const actors=[['feePayer',m.feePayer],['rentPayer',m.rentPayer],['governance.createKey',g?.createKey],...(g?.guardianKeys||[]).map((v,i)=>[`governance.guardianKeys.${i}`,v]),['governance.creator',g?.creator],['governance.executor',g?.executor],...(g?.approvers||[]).map((v,i)=>[`governance.approvers.${i}`,v])];
  for(const [f,v]of actors)if(present(v)&&protectedKeys.has(v)&&!knownRecipients.includes(v))fail('actor_protected_alias',f);
  for(const [f,v]of actors.filter(([f])=>!f.startsWith('governance.guardianKeys.')&&f!=='governance.createKey'))if(present(v)&&!PublicKey.isOnCurve(pk(v).toBytes()))fail('unsupported_off_curve_direct_signer',f);
  const leaves=(shape,prefix='')=>Object.entries(shape).flatMap(([k,v])=>{const f=prefix?`${prefix}.${k}`:k;return Array.isArray(v)?v.map((_,i)=>`${f}.${i}`):plain(v)?leaves(v,f):[f];});
  return {manifest:m,fixed,programData,multisig:ms,vault,recipientAddresses,transaction,proposal,rewards,missing:leaves(TEMPLATE).filter(f=>f.split('.').reduce((v,k)=>v?.[k],m)==null)};
}
function budgets(r){const loaded=Buffer.alloc(5);loaded[0]=4;loaded.writeUInt32LE(r.loadedAccountBytes,1);return [ComputeBudgetProgram.setComputeUnitLimit({units:r.computeUnits}),ComputeBudgetProgram.requestHeapFrame({bytes:r.heapBytes}),new TransactionInstruction({programId:pk(IDS.computeBudget),keys:[],data:loaded})];}
const meta=(address,signer=false,writable=false)=>({pubkey:pk(address),isSigner:signer,isWritable:writable});
const instruction=(name,keys,args=Buffer.alloc(0))=>new TransactionInstruction({programId:pk(IDS.squads),keys,data:Buffer.concat([Buffer.from(DISC[name],'hex'),args])});
function planManifest(input){
  const v=validateManifest(input),m=v.manifest,g=m.governance,common={schema:TEMPLATE.schema,sourceBaseline:BASELINE,action:m.action,targetRecipients:m.recipients,missing:v.missing,readiness:{...READINESS},resourcePolicy:RESOURCE_POLICY,retainedInterface:RETAINED_INTERFACE,preconditions:[...PRECONDITIONS]};
  if(v.missing.length)return {...common,status:'INCOMPLETE',plan:null};
  const addresses=[m.programId,v.programData.address,v.multisig.address,v.proposal.address,v.transaction.address,v.vault.address,IDS.instructions,v.fixed.config,v.fixed.guardian_registry,...v.rewards.map(r=>r.address),IDS.clock,v.recipientAddresses.htfp.address,v.recipientAddresses.teamOwner.address];
  const names=['program','program_data','multisig','proposal','transaction','governance_vault','instructions','config','guardian_registry',...Array.from({length:6},(_,i)=>`current_guardian_reward_${i}`),'clock','new_htfp_recipient','new_team_owner_recipient'];
  if(new Set(addresses).size!==18)fail('native_role_alias','manifest');const roles=addresses.map((address,i)=>({role:names[i],address,signer:i===5,writable:i===7}));
  const data=Buffer.from([...Buffer.from('PIV1UR02'),2,g.vaultIndex]),inner=new TransactionInstruction({programId:pk(m.programId),keys:roles.map(r=>meta(r.address,r.signer,r.writable)),data});
  const compact=t.compactMessage({profile:'public-recipient-plan',program:pk(m.programId),inner}),stored=t.decodeCompact(compact);assert.equal(compact.length,614);assert.equal(stored.keys.length,18);assert.deepEqual([...compact.subarray(0,4)],[1,0,1,18]);
  const base=[meta(v.multisig.address,false,true),meta(v.transaction.address,false,true),meta(g.creator,true),meta(m.rentPayer,true,true),meta(IDS.system)];
  const create=instruction('vault_transaction_create',base,Buffer.concat([Buffer.from([g.vaultIndex,0]),vec(compact),Buffer.from([0])]));assert.equal(create.data.length,629);
  const proposalCreate=instruction('proposal_create',[meta(v.multisig.address),meta(v.proposal.address,false,true),meta(g.creator,true),meta(m.rentPayer,true,true),meta(IDS.system)],Buffer.concat([le64(g.transactionIndex),Buffer.from([0])]));
  const approvals=g.approvers.map((a,i)=>({name:`approve_${i+1}`,ix:instruction('proposal_approve',[meta(v.multisig.address),meta(a,true,true),meta(v.proposal.address,false,true)],Buffer.from([0]))}));
  const execute=instruction('vault_transaction_execute',[meta(v.multisig.address),meta(v.proposal.address,false,true),meta(v.transaction.address),meta(g.executor,true),...stored.keys.map(k=>meta(k.pubkey.toBase58(),k.isSigner&&!k.pubkey.equals(pk(v.vault.address)),k.isWritable))]);
  const phases=[{name:'transaction_create',ix:create},{name:'proposal_create',ix:proposalCreate},...approvals,{name:'execute',ix:execute}],packets=phases.map(({name,ix})=>initializer.packet({m},name,[...budgets(m.resources),ix]));
  assert(packets.every(p=>p.wireBytes<=1232));assert(packets.every(p=>!p.requiredSigners.includes(v.vault.address)));
  return {...common,status:'OFFLINE_PLAN_ONLY',declarationSha256:sha(Buffer.from(JSON.stringify(m))),plan:{program:m.programId,feePayer:m.feePayer,rentPayer:m.rentPayer,desiredRecipients:{...m.recipients},roles,native:{bytes:data.length,hex:data.toString('hex'),sha256:sha(data)},
    derived:{programData:v.programData,multisig:v.multisig,vault:v.vault,transaction:v.transaction,proposal:v.proposal,config:v.fixed.config,guardianRegistry:v.fixed.guardian_registry,currentRewards:v.rewards,recipientAddresses:v.recipientAddresses},
    governanceDeclaration:{...g,currentSetAuthenticated:false,inputKeysMeaning:'CURRENT_PIV_SLOT_ORDER_NOT_SORTED_SQUADS_MEMBER_ORDER',declaredFourApproversAreNotActualVotes:true},
    compact:{bytes:compact.length,hex:compact.toString('hex'),sha256:sha(compact),storedLookupCount:0,ephemeralSignerCount:0,programIndex:stored.programIndex,accountIndices:stored.indices,keys:stored.keys.map(k=>({address:k.pubkey.toBase58(),signer:k.isSigner,writable:k.isWritable}))},
    sequence:phases.map((p,i)=>({step:i+1,packet:p.name,status:'UNSIGNED_NOT_EXECUTED',guard:i===0?'CURRENT_INDEX_PLUS_ONE_FRESH_TARGETS':i===1?'CREATED_EXACT_TRANSACTION_ACTIVE_PROPOSAL':i<6?'CURRENT_DISTINCT_VOTER_ACTIVE_PROPOSAL':'ACTUAL_APPROVED_NONSTALE_RELEASED_TIMELOCK_DIRECT_SQUADS'})),
    intendedEvent:{name:'RecipientsUpdated',observed:false,config:v.fixed.config,multisig:v.multisig.address,transactionIndex:g.transactionIndex,previousHtfpRecipient:null,previousTeamOwnerRecipient:null,htfpRecipient:v.recipientAddresses.htfp.address,teamOwnerRecipient:v.recipientAddresses.teamOwner.address},requestedResources:m.resources,referenceContext:{...m.transport,verified:false,usage:'ENCODING_ONLY_REFRESH_INDEPENDENTLY_EACH_PHASE'},packets}};
}
function verifySources(){const inherited=initializer.verifySources();for(const [file,expected]of Object.entries(SOURCE_PINS))assert.equal(sha(fs.readFileSync(path.join(REPO,file))),expected,`source binding: ${file}`);return {node:inherited.node,web3:inherited.web3,pins:SOURCE_PINS,initializerPins:inherited.pins};}
function main(args){const sourceCheck=verifySources();if(args.length===1&&args[0]==='--template'){process.stdout.write(JSON.stringify(TEMPLATE,null,2)+'\n');return;}if(args.length!==2||args[0]!=='--manifest')fail('unsupported_arguments','arguments');
  const filename=path.resolve(args[1]),stat=fs.statSync(filename);if(!stat.isFile()||stat.size>65536)fail('public_manifest_file_required','manifest');const bytes=fs.readFileSync(filename);if(bytes.length>65536)fail('manifest_size','manifest');const report=planManifest(initializer.parseManifest(bytes.toString('utf8')));report.manifestInputSha256=sha(bytes);report.sourceCheck=sourceCheck;process.stdout.write(JSON.stringify(report,null,2)+'\n');}
module.exports={BASELINE,SOURCE_PINS,RETAINED_INTERFACE,TEMPLATE,IDS,RESOURCE_POLICY,validateManifest,planManifest,verifySources};
if(require.main===module){try{main(process.argv.slice(2));}catch(e){process.stderr.write(JSON.stringify({status:'INVALID_OR_UNSUPPORTED_RECIPIENT_PLAN',code:e.code||'plan_validation_failed',field:e.field||'manifest'})+'\n');process.exitCode=1;}}
