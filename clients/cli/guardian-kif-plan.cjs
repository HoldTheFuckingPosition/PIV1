'use strict';
// Direct public signer declarations and unsigned wire only; no activity/claim engine.
const assert=require('node:assert/strict'),fs=require('node:fs'),path=require('node:path');
const {createHash}=require('node:crypto');
const initializer=require('./initializer-plan.cjs'),intake=require('./intake-plan.cjs');
const t=require('../../validation/genesis-transport/transport.cjs');
const {PublicKey,TransactionInstruction,ComputeBudgetProgram}=require(t.SDK);
const SOURCE_PINS=Object.freeze({
  "programs/piv1/src/instructions/guardian_heartbeat.rs": "a539b2d426a84e087a8384d1b59485f8f6a9efee3cbf9b3ee1082dad64e89bea",
  "programs/piv1/src/instructions/claim_kif.rs": "39ce641ad53ec6cdcc98c7061af9fe9cdb8db4462899ef7e7c0819ecb769e30c",
  "programs/piv1/src/guardian_operations.rs": "a7ae2d0f25dba8db9d50dbaea6ab3b3d20b065b990f486f4a33e5152cd6a351b",
  "programs/piv1/src/guardian_clock_accounts.rs": "5cc36b79d429bb3e1818466d429259474c8a26d1c1288da16a71f22033f84946",
  "programs/piv1/src/squads_accounts.rs": "b5ccacf047d86666eadaf568019ac2df7318fb954d62ae87866ba284324e518b",
  "programs/piv1/src/kif_claim_accounts.rs": "671079e1fdd714f39986f8e6ff6e65515e65fde7a35f6d1ba8b6f1e158e53e2a",
  "programs/piv1/src/kif_claim_execution.rs": "1367067dcf266f4d7a2669671c6bbbd3c82cc44ceb34a9dcc989d9e8f3047718",
  "programs/piv1/src/state/kif_claim.rs": "3e58b3554233774f8f244e2771ffb492b98e2073b802208c8c08704659af7014",
  "programs/piv1/src/state/guardian.rs": "ce762257b8025789f8ec43351ce166f0a5f7ad0bb6f05b4c9515173d7f1cd619",
  "programs/piv1/src/state/timing.rs": "c9c1441bd3115ad60bae32c2564e2a6e1cdc29b104df9ff0905e3691e6111ac5",
  "programs/piv1/src/state/config.rs": "9d7a57c87b6781b943e0e9c13d08f73a21058d400b40540c485e6c596b042198",
  "programs/piv1/src/accounts.rs": "08e3d28971160910088a40d74215d15a9c2d07b11142e28c4fb5162d725f1c80",
  "programs/piv1/src/state_persistence.rs": "d26483f7e1b95a020623b85705ab0006cb884e85cff37a0666bafa83123401a5",
  "programs/piv1/src/instruction_boundary.rs": "0e082adeebe7cabe5f5e14f69b244c2f1464c3c2f4eacd0077173a9aad91ab9f",
  "programs/piv1/src/instruction_errors.rs": "3ae5001e0c90440e3f4384b4d94da67d35a20240877a462421daed8df5e8d7b7",
  "programs/piv1/src/errors.rs": "2aa1f8abda9e16465a23a426fded03799b8ce2fd027e04de2c9adec4be5bbb8e",
  "programs/piv1/src/events.rs": "8860d27171af88b8752841146d0d02915ec9dbe59083749bfdd2e4f86c620c44",
  "programs/piv1/src/constants.rs": "2498c79c42d732be84aae423cf31d82997574cc08cca02fb226fea9acaa24f94",
  "clients/cli/initializer-plan.cjs": "92c13a4205f4557c74f55b03ee2dfea458592938228a7a959f5a2baf35bbeb83",
  "clients/cli/intake-plan.cjs": "ae4bb850ca4a9c8432830a6d0e60f5398fb895f3b137ff428120dec6568b253c",
  "clients/cli/initializer-plan.test.cjs": "16c642256aac6183a5e65b0cb99494bd8255b3f19b1b5f564b2124dfc2880e58",
  "validation/genesis-transport/transport.cjs": "a7694ff07577988e96546a53008bc8ab85bbc0ae7cb9353dcfc25e6c5a3c3724",
  "validation/genesis-transport/production-initializer.cjs": "c9cf0e28fa21d13e8b23b697023ad807d40f609cf46473c5019e17c46f8383a7"
});
const BASELINE='ddd47234ffd4ee39e1264665d0ab5d48270967ca';
const TEMPLATE={schema:'piv1-guardian-kif-plan-v1',cluster:'testnet',programId:null,feePayer:null,action:null,
  heartbeat:{registryRevision:null,guardianKeys:Array(6).fill(null),guardianIndex:null,vaultIndex:null,multisigAddress:null,createKey:null},
  claim:{guardian:null,registryRevision:null,guardianIndex:null,amountLamports:null,expectedCumulativeClaimed:null},
  transport:{recentBlockhash:null,lastValidBlockHeight:null,referenceSlot:null},resources:{computeUnits:null,heapBytes:null,loadedAccountBytes:null}};
const IDS=Object.freeze({...initializer.IDS,clock:'SysvarC1ock11111111111111111111111111111111'});
const RESOURCE_POLICY=intake.RESOURCE_POLICY,REPO=path.resolve(__dirname,'../..'),U64=(1n<<64n)-1n;
const ACTIONS=Object.freeze({'guardian-heartbeat':'heartbeat','claim-kif':'claim'});
const pk=s=>new PublicKey(s),sha=b=>createHash('sha256').update(b).digest('hex'),present=v=>v!==null&&v!==undefined;
const plain=v=>v!==null&&typeof v==='object'&&!Array.isArray(v)&&Object.getPrototypeOf(v)===Object.prototype;
const fail=(code,field)=>{const e=new Error(code);e.code=code;e.field=field;throw e;};
const le64=v=>{const b=Buffer.alloc(8);b.writeBigUInt64LE(BigInt(v));return b;};
const pda=(program,seeds)=>{const [a,bump]=PublicKey.findProgramAddressSync(seeds,pk(program));return {address:a.toBase58(),bump};};
const reward=(program,guardian,revision,index)=>pda(program,[Buffer.from('guardian-reward'),pk(guardian).toBuffer(),le64(revision),Buffer.from([index])]);
const OPEN=Object.freeze({publicReady:false,deploymentAuthorized:false,signerControlVerified:false,chainSnapshotVerified:false,accountValidityVerified:false,resourceSufficiencyVerified:false,
  eligibility:null,fees:null,rent:null,funding:null,claimable:null,liabilityBacking:null,currentPeriod:null,activityOutcome:null,m4Reservation:'OPEN_UNWAIVED'});
const PRECONDITIONS=[
  'Only public declarations are encoded. Refresh exact program/artifact, accounts and reference blockhash/context before any future signed step. Actual identity/control/account state/resources/fees/funding remain unverified; no RPC, wallet/key files, keypair creation, signing or sending is implemented.',
  'This planner supports direct EOA signer-wallet packets only: selected guardian and fee payer must be on curve. Production may accept runtime PDA signers through CPI, including program-owned nonempty PDA claim accounts; such CPI signing/composition is unsupported by this planner, not forbidden by production. Other guardian declarations have no blanket curve requirement.',
  'The fee payer may equal the selected guardian or be distinct. Full all-zero signature slots count both signer unions and global writability. Requested CU/heap/loaded caps are declarations, not measured sufficiency or a send-ready transaction. No rent or fee estimate is calculated.',
];
const DETAILS={
  'guardian-heartbeat':{effect:'Production authenticates current membership/governance and runtime Clock, then records activity only for the selected current reward slot. It creates no earned liability or transfer and does not alter an active distribution snapshot.',guards:[
    'CURRENT registry revision and six declared guardian keys must match actual Config/registry/reward records in their exact slot order. GuardianIndex selects one direct guardian signature and one writable reward; never sort or remap slots.',
    'Production validates loader-v3 Program/derived ProgramData headers, current Squads multisig canonical create-key PDA, six-voter set/permissions/threshold4 and derived vaultIndex authority correspondence. ProgramData/executable provenance, membership/control and current authority remain unverified. This heartbeat is not a Squads transaction, proposal, four approvals or vault signature; no Squads CPI occurs.',
    'Period/time come only from authenticated runtime Clock and Config anchor using fixed2592000-second half-open periods. Production enforces activity monotonicity; no caller timestamp, period, active flag or liability amount is accepted. Post-snapshot activity is not retroactive and does not establish distribution eligibility.',
    'Heartbeat has no pause check in this production path. Production requires authenticated initialized/rent-backed state and exact current governance/Clock correspondence; this tool does not infer validity, activity outcome or period.',
    'Runtime Program is also explicit native role10 and must union with the instruction program key once. Guardian is requested readonly signer; shared fee payer elevates it globally writable. Only selected reward is requested writable; Config/registry/other rewards/Clock/ProgramData/multisig stay readonly.',
  ]},
  'claim-kif':{effect:'Production pays the requested already-earned KIF amount from isolated KifSolVault to the exact earned guardian signer/destination, with replay-guarded individual/global accounting and fixed System transfer. Actual claimability/success remain unknown.',guards:[
    'EARNED guardian/revision/slot tuple derives the historical reward PDA independently of current membership, registry, activity, Clock, pool or distribution. The guardian is both signer and fixed payout destination; alternate payout or caller-selected source is unsupported by this production boundary.',
    'Claim remains allowed during global pause. Positive amount must fit stored claimable balance AND global KIF liability; expectedCumulativeClaimed must equal the current earned record counter. All individual/global identities, source backing and checked additions including destination balance/cumulative counters must hold; full-width syntactic encoding is not an overflow or claimability proof.',
    'Source must back runtime rent floor + entire KIF liability + collective carry. Source excess, carry, state rents, principal, pending contributions, operational rent, distribution custody and HWM remain isolated. Claim creates no reward and changes no activity or eligibility.',
    'Actual claim guardian must be the recorded earned-owner key, a writable runtime signer and the same payment destination; owner/data are opaque and preserved during transfer. Config/reward/source must pass canonical PDA/discriminator/bump/owner/size/rent checks. All four state/source/destination roles are writable; System Program is readonly and executable. No current registry or governance account is required.',
    'Production commits checked accounting before fixed signed System transfer and reauthenticates exact post-state/custody. Errors must propagate for transaction rollback; no partial success or runtime proof is supplied by this unsigned encoding.',
  ]},
};
const ERROR_GUIDE=[[6002,'MissingGuardianSignature'],[6003,'ZeroKifClaim'],[6004,'StaleKifClaim'],[6005,'KifClaimExceeded'],[6006,'KifClaimBackingDeficit'],[6007,'KifClaimStateChanged'],[6008,'KifClaimObservationMismatch'],[6009,'InvalidClockAccount'],[6016,'InvalidAccountPda'],[6017,'AccountAlias'],[6018,'InvalidProgramIdentity'],[6020,'AccountRentDeficit'],[6037,'TimestampRegression'],[6059,'CumulativeReconciliationMismatch'],[6067,'InvalidGuardianSet'],[6072,'ArithmeticOverflow']].map(([code,name])=>({code,name}));
function validateManifest(input){
  function walk(v,s,f){if(!present(v))return null;if(Array.isArray(s)){if(!Array.isArray(v)||v.length!==6)fail('six_slots_required',f);return v.map((k,i)=>walk(k,null,`${f}.${i}`));}
    if(plain(s)){if(!plain(v))fail('object_required',f||'manifest');if(Object.keys(v).some(k=>!Object.hasOwn(s,k)))fail('unknown_field',f||'manifest');return Object.fromEntries(Object.entries(s).map(([k,shape])=>[k,walk(v[k],shape,f?`${f}.${k}`:k)]));}return v;}
  if(!plain(input))fail('object_required','manifest');const m=walk(input,TEMPLATE,''),check=(v,f,fn)=>{if(present(v))fn(v,f);};
  const key=(v,f)=>{if(typeof v!=='string')fail('public_key_string',f);let k;try{k=pk(v);}catch{fail('invalid_public_key',f);}if(k.toBase58()!==v||v===IDS.system)fail('noncanonical_or_zero_public_key',f);};
  const decimal=(v,f,positive=false)=>{if(typeof v!=='string'||!/^(?:0|[1-9]\d*)$/.test(v))fail('decimal_string_required',f);if(BigInt(v)>U64||positive&&BigInt(v)===0n)fail('integer_range',f);};
  const small=(v,f,max)=>{if(!Number.isSafeInteger(v)||v<0||v>max)fail('integer_range',f);};
  check(m.schema,'schema',v=>{if(v!==TEMPLATE.schema)fail('unsupported_schema','schema');});check(m.cluster,'cluster',v=>{if(v!=='testnet')fail('unsupported_cluster','cluster');});
  check(m.action,'action',v=>{if(typeof v!=='string'||!Object.hasOwn(ACTIONS,v))fail('unsupported_action','action');});for(const f of ['programId','feePayer'])check(m[f],f,key);
  const h=m.heartbeat,c=m.claim;
  check(h?.registryRevision,'heartbeat.registryRevision',decimal);h?.guardianKeys?.forEach((v,i)=>check(v,`heartbeat.guardianKeys.${i}`,key));
  check(h?.guardianIndex,'heartbeat.guardianIndex',(v,f)=>small(v,f,5));check(h?.vaultIndex,'heartbeat.vaultIndex',(v,f)=>small(v,f,255));
  for(const f of ['multisigAddress','createKey'])check(h?.[f],`heartbeat.${f}`,key);
  check(c?.guardian,'claim.guardian',key);check(c?.registryRevision,'claim.registryRevision',decimal);check(c?.guardianIndex,'claim.guardianIndex',(v,f)=>small(v,f,5));
  check(c?.amountLamports,'claim.amountLamports',(v,f)=>decimal(v,f,true));check(c?.expectedCumulativeClaimed,'claim.expectedCumulativeClaimed',decimal);
  if(m.transport){check(m.transport.recentBlockhash,'transport.recentBlockhash',key);for(const f of ['lastValidBlockHeight','referenceSlot'])check(m.transport[f],`transport.${f}`,decimal);}
  if(m.resources)for(const [f,limit]of Object.entries(RESOURCE_POLICY))if(f!=='meaning')check(m.resources[f],`resources.${f}`,(v,field)=>{if(!Number.isSafeInteger(v)||v<limit.min||v>limit.max||limit.multiple&&v%limit.multiple!==0)fail('unsupported_resource_request',field);});
  const knownGuardians=(h?.guardianKeys||[]).filter(present);if(new Set(knownGuardians).size!==knownGuardians.length)fail('duplicate_guardian','heartbeat.guardianKeys');
  if(present(m.programId)&&Object.values(IDS).includes(m.programId))fail('unsupported_program_identity','programId');
  const d=present(m.programId)?intake.derived(m.programId):null,internals=new Set(d?[m.programId,...Object.values(d)]:[]),rewards=[];
  if(d&&present(h?.registryRevision))h?.guardianKeys?.forEach((g,i)=>{if(present(g))rewards.push(reward(m.programId,g,h.registryRevision,i).address);});
  if(d&&present(c?.guardian)&&present(c?.registryRevision)&&present(c?.guardianIndex))rewards.push(reward(m.programId,c.guardian,c.registryRevision,c.guardianIndex).address);
  // Current wallets need no curve/owner restriction; PIV state/authority/custody and reward aliases remain forbidden.
  for(const g of [...knownGuardians,...(present(c?.guardian)?[c.guardian]:[])])if(internals.has(g)||rewards.includes(g))fail('guardian_internal_alias','guardian');
  let governance=null;
  if(present(h?.createKey)){const multisig=pda(IDS.squads,[Buffer.from('multisig'),Buffer.from('multisig'),pk(h.createKey).toBuffer()]);
    if(present(h?.multisigAddress)&&h.multisigAddress!==multisig.address)fail('multisig_create_key_mismatch','heartbeat.multisigAddress');
    governance={multisig};if(present(h?.vaultIndex))governance.vault=pda(IDS.squads,[Buffer.from('multisig'),pk(multisig.address).toBuffer(),Buffer.from('vault'),Buffer.from([h.vaultIndex])]);}
  const programData=d?pda(IDS.loader,[pk(m.programId).toBuffer()]):null;
  if(present(h?.multisigAddress)&&[...Object.values(IDS),...internals,...rewards,...(programData?[programData.address]:[])].includes(h.multisigAddress))fail('governance_native_alias','heartbeat.multisigAddress');
  const protectedKeys=new Set([...Object.values(IDS),...internals,...rewards,...(programData?[programData.address]:[]),...(present(h?.multisigAddress)?[h.multisigAddress]:[])]);
  const selected=m.action==='guardian-heartbeat'&&present(h?.guardianIndex)?h?.guardianKeys?.[h.guardianIndex]:m.action==='claim-kif'?c?.guardian:null;
  const heartbeatSigner=present(h?.guardianIndex)?h?.guardianKeys?.[h.guardianIndex]:null;
  for(const [f,v]of [['feePayer',m.feePayer],['heartbeat.selectedGuardian',heartbeatSigner],['claim.guardian',c?.guardian]])if(present(v)){
    if(!PublicKey.isOnCurve(pk(v).toBytes()))fail('unsupported_off_curve_direct_signer',f);if(protectedKeys.has(v))fail('signer_protected_alias',f);}
  const populated=v=>Array.isArray(v)?v.some(populated):plain(v)?Object.values(v).some(populated):present(v);
  if(present(m.action))for(const b of ['heartbeat','claim'])if(b!==ACTIONS[m.action]&&populated(m[b]))fail('unused_action_input',b);
  const fields=['schema','cluster','programId','feePayer','action',...['transport','resources'].flatMap(b=>Object.keys(TEMPLATE[b]).map(f=>`${b}.${f}`))];
  if(present(m.action)){const b=ACTIONS[m.action];for(const f of Object.keys(TEMPLATE[b]))if(f==='guardianKeys')fields.push(...Array.from({length:6},(_,i)=>`${b}.${f}.${i}`));else fields.push(`${b}.${f}`);}
  return {manifest:m,derived:d,programData,governance,selected,missing:fields.filter(f=>f.split('.').reduce((v,k)=>v?.[k],m)==null)};
}
function budgets(resources){const loaded=Buffer.alloc(5);loaded[0]=4;loaded.writeUInt32LE(resources.loadedAccountBytes,1);return [ComputeBudgetProgram.setComputeUnitLimit({units:resources.computeUnits}),ComputeBudgetProgram.requestHeapFrame({bytes:resources.heapBytes}),new TransactionInstruction({programId:pk(IDS.computeBudget),keys:[],data:loaded})];}
function planManifest(input){
  const v=validateManifest(input),m=v.manifest,details=DETAILS[m.action],common={schema:TEMPLATE.schema,sourceBaseline:BASELINE,action:m.action,missing:v.missing,readiness:{...OPEN},expectedEffect:details?.effect??null,preconditions:[...PRECONDITIONS,...(details?.guards||[])],resourcePolicy:RESOURCE_POLICY,errorGuide:{scope:'NONEXHAUSTIVE_SOURCE_GUIDANCE_ONLY; no live error observed or diagnosed',custom:ERROR_GUIDE,downstream:'System/loader/Squads validation/runtime errors may propagate; no universal decoder or guessed remedy.'}};
  if(v.missing.length)return {...common,status:'INCOMPLETE',plan:null};
  const heartbeat=m.action==='guardian-heartbeat',roles=[],add=(role,address,writable=false,signer=false)=>roles.push({role,address,writable,signer});let data,tuple,derived;
  if(heartbeat){const h=m.heartbeat,rewards=h.guardianKeys.map((g,i)=>reward(m.programId,g,h.registryRevision,i));
    add('config',v.derived.config);add('guardian_registry',v.derived.guardian_registry);rewards.forEach((r,i)=>add(`current_guardian_reward_${i}`,r.address,i===h.guardianIndex));add('clock',IDS.clock);add('guardian',v.selected,false,true);add('runtime_program',m.programId);add('program_data',v.programData.address);add('multisig',h.multisigAddress);
    data=Buffer.from([...Buffer.from('PIV1HB01'),1,h.guardianIndex,h.vaultIndex]);tuple={kind:'CURRENT_REGISTRY',...h};derived={config:v.derived.config,guardian_registry:v.derived.guardian_registry,currentRewards:rewards,program_data:v.programData,...v.governance};
  }else{const c=m.claim,r=reward(m.programId,c.guardian,c.registryRevision,c.guardianIndex);add('config',v.derived.config,true);add('guardian_reward',r.address,true);add('kif_sol',v.derived.kif_sol,true);add('guardian',c.guardian,true,true);add('system_program',IDS.system);
    data=Buffer.concat([Buffer.from([253,151,172,11,202,77,118,170]),le64(c.amountLamports),le64(c.expectedCumulativeClaimed)]);tuple={kind:'EARNED_REWARD',guardian:c.guardian,registryRevision:c.registryRevision,guardianIndex:c.guardianIndex};derived={config:v.derived.config,guardian_reward:r,kif_sol:v.derived.kif_sol};}
  if(new Set(roles.map(r=>r.address)).size!==roles.length)fail('native_role_alias','manifest');if(m.feePayer!==v.selected&&roles.some(r=>r.address===m.feePayer))fail('fee_payer_native_alias','feePayer');
  const instruction=new TransactionInstruction({programId:pk(m.programId),keys:roles.map(r=>({pubkey:pk(r.address),isSigner:r.signer,isWritable:r.writable})),data});const packet=initializer.packet({m},m.action,[...budgets(m.resources),instruction]);
  const expectedSigners=[...new Set([m.feePayer,v.selected])];assert.equal(packet.signatureCount,expectedSigners.length);assert.deepEqual([...packet.requiredSigners].sort(),expectedSigners.sort());
  return {...common,status:'OFFLINE_PLAN_ONLY',declarationSha256:sha(Buffer.from(JSON.stringify(m))),plan:{program:m.programId,feePayer:m.feePayer,selectedGuardian:v.selected,signerProfile:'DIRECT_EOA_UNSIGNED_ONLY',derived,roles,native:{bytes:data.length,hex:data.toString('hex'),sha256:sha(data)},guardianContext:tuple,requestedClaim:heartbeat?null:{amountLamports:m.claim.amountLamports,expectedCumulativeClaimed:m.claim.expectedCumulativeClaimed},requestedResources:m.resources,referenceContext:{...m.transport,verified:false,usage:'ENCODING_INPUT_ONLY; refresh/revalidate before any future signed step'},packets:[packet]}};
}
function verifySources(){const existing=initializer.verifySources();for(const [file,expected]of Object.entries(SOURCE_PINS))assert.equal(sha(fs.readFileSync(path.join(REPO,file))),expected,`source binding: ${file}`);return {node:existing.node,web3:existing.web3,pins:SOURCE_PINS,initializerPins:existing.pins};}
function main(args){const sourceCheck=verifySources();if(args.length===1&&args[0]==='--template'){process.stdout.write(JSON.stringify(TEMPLATE,null,2)+'\n');return;}
  if(args.length!==2||args[0]!=='--manifest')fail('unsupported_arguments','arguments');const filename=path.resolve(args[1]),stat=fs.statSync(filename);if(!stat.isFile()||stat.size>65536)fail('public_manifest_file_required','manifest');const bytes=fs.readFileSync(filename);if(bytes.length>65536)fail('manifest_size','manifest');
  const report=planManifest(initializer.parseManifest(bytes.toString('utf8')));report.manifestInputSha256=sha(bytes);report.sourceCheck=sourceCheck;process.stdout.write(JSON.stringify(report,null,2)+'\n');}
module.exports={BASELINE,SOURCE_PINS,TEMPLATE,IDS,RESOURCE_POLICY,ERROR_GUIDE,validateManifest,planManifest,verifySources};
if(require.main===module){try{main(process.argv.slice(2));}catch(e){process.stderr.write(JSON.stringify({status:'INVALID_OR_UNSUPPORTED_GUARDIAN_KIF_PLAN',code:e.code||'plan_validation_failed',field:e.field||'manifest'})+'\n');process.exitCode=1;}}
