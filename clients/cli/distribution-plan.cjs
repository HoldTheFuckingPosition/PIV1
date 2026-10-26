'use strict';
// Offline native distribution boundaries only; no action selection or economics.
const assert=require('node:assert/strict');
const fs=require('node:fs');
const path=require('node:path');
const {createHash}=require('node:crypto');
const initializer=require('./initializer-plan.cjs');
const intake=require('./intake-plan.cjs');
const t=require('../../validation/genesis-transport/transport.cjs');
const {PublicKey,TransactionInstruction,ComputeBudgetProgram}=require(t.SDK);
const SOURCE_PINS=Object.freeze({
  "programs/piv1/src/instructions/prepare_distribution.rs": "cd8560e7de24432223e2cf76bee331c3206c5c212d9e920c9747beb938fc6e91",
  "programs/piv1/src/instructions/settle_distribution.rs": "b4b6e04ec3c64097ba3d82528623af968874539eddde6994f85a0153dfd76dc2",
  "programs/piv1/src/instructions/integrate_pending.rs": "863eb5fa77e6874d3c27fdd0a2f224b76d038d19dbac2ba4a56c6f0d172e82be",
  "programs/piv1/src/distribution_preparation_execution.rs": "e1e1829384ebf600deaa504aaa897653c16e230a860d711f16d58a46a67cb2eb",
  "programs/piv1/src/settlement_execution.rs": "0ff6fd24c43e3689707d0e6f7574e02e3a878d93ec8dab8c54668b62d9bf1552",
  "programs/piv1/src/pending_integration_execution.rs": "b9ebebae64b4779e298f80d00517a4846698cd5b1a07a76b8b4834551cace0fb",
  "programs/piv1/src/guardian_clock_accounts.rs": "5cc36b79d429bb3e1818466d429259474c8a26d1c1288da16a71f22033f84946",
  "programs/piv1/src/kif_claim_accounts.rs": "671079e1fdd714f39986f8e6ff6e65515e65fde7a35f6d1ba8b6f1e158e53e2a",
  "programs/piv1/src/state/guardian.rs": "ce762257b8025789f8ec43351ce166f0a5f7ad0bb6f05b4c9515173d7f1cd619",
  "programs/piv1/src/state/distribution.rs": "a291d3fb94921cd036161ee669190074fd5f159fe0d9f523a98ef3b0fa25d3a3",
  "programs/piv1/src/state/transitions.rs": "a7319afe56f2bf5301e689f68832c23d05620d3e478830c651777125e9671cf2",
  "programs/piv1/src/state/timing.rs": "c9c1441bd3115ad60bae32c2564e2a6e1cdc29b104df9ff0905e3691e6111ac5",
  "programs/piv1/src/state/reconciliation.rs": "0604726bc0470664d90da7040bccca34d2a062d3bfa02dc546d8b02214afc386",
  "programs/piv1/src/state/config.rs": "9d7a57c87b6781b943e0e9c13d08f73a21058d400b40540c485e6c596b042198",
  "programs/piv1/src/accounts.rs": "08e3d28971160910088a40d74215d15a9c2d07b11142e28c4fb5162d725f1c80",
  "programs/piv1/src/integrations/jito_identity.rs": "0dba585d0454af35ce9c087cfaa260b384e9d3f3396413282641d9ca087354d3",
  "programs/piv1/src/integrations/jito_withdrawal_preparation.rs": "b057cf6716a353d90bcb4993466243a09f9f31d0c2291166380b80d5983c94fa",
  "programs/piv1/src/instruction_boundary.rs": "0e082adeebe7cabe5f5e14f69b244c2f1464c3c2f4eacd0077173a9aad91ab9f",
  "programs/piv1/src/instruction_errors.rs": "3ae5001e0c90440e3f4384b4d94da67d35a20240877a462421daed8df5e8d7b7",
  "programs/piv1/src/events.rs": "8860d27171af88b8752841146d0d02915ec9dbe59083749bfdd2e4f86c620c44",
  "clients/cli/initializer-plan.cjs": "92c13a4205f4557c74f55b03ee2dfea458592938228a7a959f5a2baf35bbeb83",
  "clients/cli/intake-plan.cjs": "ae4bb850ca4a9c8432830a6d0e60f5398fb895f3b137ff428120dec6568b253c",
  "clients/cli/initializer-plan.test.cjs": "16c642256aac6183a5e65b0cb99494bd8255b3f19b1b5f564b2124dfc2880e58",
  "programs/piv1/src/constants.rs": "2498c79c42d732be84aae423cf31d82997574cc08cca02fb226fea9acaa24f94"
});
const BASELINE='5a76a0646bd3e1c36018e639e388c7e3fc406e4c';
const TEMPLATE={
  "schema": "piv1-distribution-plan-v1",
  "cluster": "testnet",
  "programId": null,
  "feePayer": null,
  "action": null,
  "protocol": {
    "validatorList": null,
    "reserve": null,
    "managerFee": null,
    "referrer": null
  },
  "currentGuardians": {
    "registryRevision": null,
    "guardianKeys": [
      null,
      null,
      null,
      null,
      null,
      null
    ]
  },
  "withdrawal": {
    "validatorIndex": null,
    "sourceStake": null
  },
  "frozenSettlement": {
    "registryRevision": null,
    "guardianKeys": [
      null,
      null,
      null,
      null,
      null,
      null
    ],
    "htfpRecipient": null,
    "teamOwnerRecipient": null
  },
  "transport": {
    "recentBlockhash": null,
    "lastValidBlockHeight": null,
    "referenceSlot": null
  },
  "resources": {
    "computeUnits": null,
    "heapBytes": null,
    "loadedAccountBytes": null
  }
};
const IDS=Object.freeze({...initializer.IDS,clock:'SysvarC1ock11111111111111111111111111111111'}),RESOURCE_POLICY=intake.RESOURCE_POLICY;
const REPO=path.resolve(__dirname,'../..'),U64=(1n<<64n)-1n,U32=(1n<<32n)-1n;
const ACTIONS=Object.freeze({'prepare-liquid':'PIV1PD01','prepare-withdrawal':'PIV1PW01',settle:'PIV1SD01','integrate-pending':'PIV1IP01'});
const OPEN=Object.freeze({publicReady:false,deploymentAuthorized:false,signerControlVerified:false,chainSnapshotVerified:false,
  accountValidityVerified:false,resourceSufficiencyVerified:false,eligibility:null,fees:null,rent:null,funding:null,
  grossYield:null,withdrawalTarget:null,technicalMinimum:null,sourceCapacity:null,settlementAmounts:null,pendingIntegrationValue:null,m4Reservation:'OPEN_UNWAIVED'});
const ERROR_GUIDE=[
  [6009,'InvalidClockAccount','Canonical Clock bytes and runtime Clock must agree.'],[6017,'AccountAlias','Distinct native roles must not alias.'],
  [6034,'InvalidLifecycle','Preparation requires Idle, settlement EscrowFunded, integration Settled.'],[6035,'PausedOperation','All four economic transitions require unpaused state.'],
  [6037,'TimestampRegression','Current time/epoch/slot must satisfy source timing checks.'],[6038,'PreparationIntervalNotElapsed','Ten days since successful preparation have not elapsed.'],
  [6039,'InsufficientAttemptCooldownActive','A new valid-insufficient result is within the 24-hour retry cooldown.'],[6040,'InvalidInsufficientAttempt','An insufficiency result lacks required authenticated facts.'],
  [6043,'ZeroTarget','A positive withdrawal/outgoing target is required.'],[6046,'InvalidCustodyObservation','Custody or current pool facts are invalid.'],
  [6053,'TechnicalFloorNotMet','Supported source/minimum/capacity constraints fail.'],[6059,'CumulativeReconciliationMismatch','Stored cumulative economics do not reconcile.'],
  [6063,'SettlementReplay','Settlement was already recorded.'],[6064,'HighWaterMarkDecrease','Protected value must not fall below the required HWM.'],
  [6067,'InvalidGuardianSet','Guardian key/slot/revision binding fails.'],[6073,'RecoveryRequired','Stop normal progression; no recovery operation is planned.'],
  [6145,'DistributionWithdrawalUnavailable','Liquid preparation has positive native shortfall; it does not record technical insufficiency.'],
].map(([code,name,meaning])=>({code,name,meaning}));
const PRECONDITIONS=[
  'All public inputs are declarations. Independently authenticate exact cluster/artifact/Config binding, ownership, data, canonical bumps, rent, custody and current official pool topology.',
  'Program must be unpaused. Whole economic custody, token-native quarantine, pending obligations, operational rent, current pool epoch/supply and protected value must reconcile under production guards.',
  'No quote, yield, target, technical minimum, source capacity, time, amount, recipient choice or action eligibility is inferred. Refresh blockhash/state and verify funding/resources before any future signed step.',
];
const DETAILS={
  'prepare-liquid':{effect:'May leave no-yield evaluation unchanged with no snapshot/event, or record a liquid-funded round and move only its authenticated pending SOL/prior-yield funding to escrow. Outcome is unknown.',
    guards:['Requires Idle and the ten-day minimum since successful preparation; the clock starts at successful snapshot, not settlement. No yield creates no snapshot and resets neither clock.',
      'Current Config/registry revision and six CURRENT slot rewards determine activity using authenticated Clock; they are not caller-selected eligibility. Positive native shortfall rejects6145; this is not valid insufficiency.']},
  'prepare-withdrawal':{effect:'Queries Stake GetMinimumDelegation with no account metas; may record only a valid-insufficient timestamp/event, or freeze a withdrawal-funded round and move its liquid funding. No withdrawal leg, stake creation, burn or deactivation occurs here.',
    guards:['Requires unpaused Idle, ten-day preparation cadence, positive native shortfall and current guardian tuple. A valid-insufficient result additionally requires the 24-hour retry cooldown; it does not reset the ten-day clock or create a round.',
      'Explicit u32 validator-list index and sourceStake must match the current active-source record/PDA, vote/suffix, current epoch/status, preferred validator, Stake ownership/state/authorities/lockup/rent and exact balance. Index alone does not derive a source.',
      'Production authenticates Stake return-data origin/length and derives the dynamic minimum, safe capacity, fee-aware target and residual HWM floor. Operational rent funding, first fill/remainder and leg bounds must hold. No transient/reserve/removal/preferred fallback is selected. No-yield withdrawal rejects ZeroTarget.']},
  settle:{effect:'May atomically pay the two FROZEN recipients, credit frozen guardian rewards/KIF carry, update protected accounting and mark Settled; alternatively record RecoveryRequired without payments. Exact outcome/amounts are unknown.',
    guards:['Requires EscrowFunded, exact target assignment/all successful legs finalized, cumulative escrow reconciliation and no replay. Current pool/Clock and protected-value checks precede settlement.',
      'Use the active round\'s FROZEN revision, six guardian slots and two recipients, even after current governance membership/recipients change. No current registry account is supplied.',
      'Withdrawal fees reduce net outgoing allocation under production rules; zero-active KIF compounding/carry, old liabilities and rounding stay guarded. Guardian-recipient overlap is allowed; native account roles must remain distinct.']},
  'integrate-pending':{effect:'After settlement, integrate all authenticated pending contributions and residual escrow at current pool book value, clear pending ledgers and complete the round to Idle. Amounts/value are unknown; no staking occurs.',
    guards:['Requires Settled, valid current Clock/pool/custody and no RecoveryRequired. Full contribution value enters HWM even where pending SOL already funded the round; physical remaining SOL/token and escrow move exactly once.',
      'Preserve rent, KIF liability/carry, token-native quarantine, cooldown-yield classification and HWM protection. Current/frozen guardians and recipients are not inputs to this boundary.']},
};
const sha=b=>createHash('sha256').update(b).digest('hex'),pk=s=>new PublicKey(s);
const fail=(code,field)=>{const e=new Error(code);e.code=code;e.field=field;throw e;};
const plain=v=>v!==null&&typeof v==='object'&&!Array.isArray(v)&&Object.getPrototypeOf(v)===Object.prototype;
const present=v=>v!==null&&v!==undefined;
function rewardAddresses(program,context){
  const revision=Buffer.alloc(8);revision.writeBigUInt64LE(BigInt(context.registryRevision));
  return context.guardianKeys.map((guardian,slot)=>guardian===null?null:PublicKey.findProgramAddressSync(
    [Buffer.from('guardian-reward'),pk(guardian).toBuffer(),revision,Buffer.from([slot])],pk(program))[0].toBase58());
}
function validateManifest(input){
  function walk(value,shape,field){
    if(!present(value))return null;
    if(Array.isArray(shape)){if(!Array.isArray(value)||value.length!==6)fail('six_slots_required',field);return value.map((v,i)=>walk(v,null,`${field}.${i}`));}
    if(plain(shape)){if(!plain(value))fail('object_required',field||'manifest');if(Object.keys(value).some(k=>!Object.hasOwn(shape,k)))fail('unknown_field',field||'manifest');
      return Object.fromEntries(Object.entries(shape).map(([k,v])=>[k,walk(value[k],v,field?`${field}.${k}`:k)]));}return value;
  }
  if(!plain(input))fail('object_required','manifest');const m=walk(input,TEMPLATE,''),check=(v,f,fn)=>{if(present(v))fn(v,f);};
  const key=(v,f)=>{if(typeof v!=='string')fail('public_key_string',f);let p;try{p=pk(v);}catch{fail('invalid_public_key',f);}if(p.toBase58()!==v||v===IDS.system)fail('noncanonical_or_zero_public_key',f);};
  const decimal=(v,f,max=U64)=>{if(typeof v!=='string'||!/^(?:0|[1-9]\d*)$/.test(v))fail('decimal_string_required',f);if(BigInt(v)>max)fail('integer_range',f);};
  check(m.schema,'schema',v=>{if(v!==TEMPLATE.schema)fail('unsupported_schema','schema');});check(m.cluster,'cluster',v=>{if(v!=='testnet')fail('unsupported_cluster','cluster');});
  check(m.action,'action',v=>{if(typeof v!=='string'||!Object.hasOwn(ACTIONS,v))fail('unsupported_action','action');});
  for(const field of ['programId','feePayer'])check(m[field],field,key);
  if(m.protocol)for(const [f,v]of Object.entries(m.protocol))check(v,`protocol.${f}`,key);
  for(const block of ['currentGuardians','frozenSettlement'])if(m[block]){
    check(m[block].registryRevision,`${block}.registryRevision`,decimal);
    m[block].guardianKeys?.forEach((v,i)=>check(v,`${block}.guardianKeys.${i}`,key));
    const keys=(m[block].guardianKeys||[]).filter(present);if(new Set(keys).size!==keys.length)fail('duplicate_guardian',block);
  }
  for(const f of ['htfpRecipient','teamOwnerRecipient'])check(m.frozenSettlement?.[f],`frozenSettlement.${f}`,key);
  check(m.withdrawal?.sourceStake,'withdrawal.sourceStake',key);check(m.withdrawal?.validatorIndex,'withdrawal.validatorIndex',(v,f)=>decimal(v,f,U32));
  if(m.transport){check(m.transport.recentBlockhash,'transport.recentBlockhash',key);for(const f of ['lastValidBlockHeight','referenceSlot'])check(m.transport[f],`transport.${f}`,decimal);}
  if(m.resources)for(const [f,limit]of Object.entries(RESOURCE_POLICY))if(f!=='meaning')check(m.resources[f],`resources.${f}`,(v,field)=>{
    if(!Number.isSafeInteger(v)||v<limit.min||v>limit.max||limit.multiple&&v%limit.multiple!==0)fail('unsupported_resource_request',field);
  });
  if(present(m.feePayer)&&!PublicKey.isOnCurve(pk(m.feePayer).toBytes()))fail('unsupported_off_curve_fee_signer','feePayer');
  if(present(m.programId)&&Object.values(IDS).includes(m.programId))fail('unsupported_program_identity','programId');
  const d=present(m.programId)?intake.derived(m.programId):null,internal=new Set(d?Object.values(d):[]),rewardKeys=[];
  for(const block of ['currentGuardians','frozenSettlement'])if(m[block]){
    const keys=m[block].guardianKeys||[];
    if(d&&present(m[block].registryRevision)&&keys.length)rewardKeys.push(...rewardAddresses(m.programId,m[block]).filter(present));
    if(keys.some(k=>present(k)&&(internal.has(k)||k===m.programId)))fail('guardian_internal_alias',block);
  }
  const protectedKeys=new Set([...Object.values(IDS),...(d?[m.programId,...internal]:[]),...rewardKeys]);
  const native=[...Object.entries(m.protocol||{}).map(([f,v])=>[`protocol.${f}`,v]),['withdrawal.sourceStake',m.withdrawal?.sourceStake],
    ...['htfpRecipient','teamOwnerRecipient'].map(f=>[`frozenSettlement.${f}`,m.frozenSettlement?.[f]])].filter(([,v])=>present(v));
  for(const [f,v]of [...native,['feePayer',m.feePayer]])if(present(v)&&protectedKeys.has(v))fail('unsupported_protected_alias',f);
  for(let a=0;a<native.length;a++)for(let b=0;b<a;b++)if(native[a][1]===native[b][1]&&![native[a][0],native[b][0]].every(f=>['protocol.managerFee','protocol.referrer'].includes(f)))fail('native_role_alias',native[a][0]);
  if(present(m.feePayer)&&native.some(([,v])=>v===m.feePayer))fail('fee_payer_native_alias','feePayer');
  for(const block of ['currentGuardians','frozenSettlement'])if(m[block]?.guardianKeys?.some(k=>present(k)&&rewardKeys.includes(k)))fail('guardian_reward_alias',block);
  const populated=value=>Array.isArray(value)?value.some(populated):plain(value)?Object.values(value).some(populated):present(value);
  const prepare=['prepare-liquid','prepare-withdrawal'].includes(m.action),used=new Set(prepare?['currentGuardians',...(m.action==='prepare-withdrawal'?['withdrawal']:[])]:m.action==='settle'?['frozenSettlement']:[]);
  if(present(m.action))for(const b of ['currentGuardians','withdrawal','frozenSettlement'])if(!used.has(b)&&populated(m[b]))fail('unused_action_input',b);
  const fields=['schema','cluster','programId','feePayer','action',...['protocol','transport','resources'].flatMap(b=>Object.keys(TEMPLATE[b]).map(f=>`${b}.${f}`))];
  for(const b of used)for(const f of Object.keys(TEMPLATE[b]))if(f==='guardianKeys')fields.push(...Array.from({length:6},(_,i)=>`${b}.${f}.${i}`));else fields.push(`${b}.${f}`);
  return {manifest:m,derived:d,missing:fields.filter(f=>f.split('.').reduce((v,k)=>v?.[k],m)==null)};
}
function budgets(resources){
  const loaded=Buffer.alloc(5);loaded[0]=4;loaded.writeUInt32LE(resources.loadedAccountBytes,1);
  const result=[ComputeBudgetProgram.setComputeUnitLimit({units:resources.computeUnits}),ComputeBudgetProgram.requestHeapFrame({bytes:resources.heapBytes}),new TransactionInstruction({programId:pk(IDS.computeBudget),keys:[],data:loaded})];
  assert.deepEqual(result.map(i=>i.data[0]),[2,1,4]);assert(result.every(i=>i.keys.length===0&&i.data.length===5));
  assert.deepEqual(result.map(i=>i.data.readUInt32LE(1)),[resources.computeUnits,resources.heapBytes,resources.loadedAccountBytes]);return result;
}
function nativeRoles(m,d){
  const roles=[],add=(role,address,writable=false)=>roles.push({role,address,signer:false,writable});
  const prepare=m.action.startsWith('prepare-'),settle=m.action==='settle';
  const writable=prepare?[0,1,2,3,5]:settle?[0,1,3,5,6]:[0,1,2,3,5,7,8];
  ['config','active_distribution','pending_sol','principal_sol','operational_sol','distribution_escrow','kif_sol','principal_token','pending_token'].forEach((role,i)=>add(role,d[role],writable.includes(i)));
  add('piv_authority',d.piv_authority);add('mint',IDS.mint);add('system_program',IDS.system);add('token_program',IDS.token);add('pool_program',IDS.poolProgram);add('pool',IDS.pool);
  add('validator_list',m.protocol.validatorList);add('reserve',m.protocol.reserve);add('manager_fee',m.protocol.managerFee);if(m.protocol.managerFee!==m.protocol.referrer)add('referrer',m.protocol.referrer);
  if(prepare){add('guardian_registry',d.guardian_registry);rewardAddresses(m.programId,m.currentGuardians).forEach((a,i)=>add(`current_guardian_reward_${i}`,a));add('clock',IDS.clock);
    if(m.action==='prepare-withdrawal'){add('stake_program',IDS.stake);add('source_stake',m.withdrawal.sourceStake);}}
  else {add('clock',IDS.clock);if(settle){add('frozen_htfp_recipient',m.frozenSettlement.htfpRecipient,true);add('frozen_team_owner_recipient',m.frozenSettlement.teamOwnerRecipient,true);
    rewardAddresses(m.programId,m.frozenSettlement).forEach((a,i)=>add(`frozen_guardian_reward_${i}`,a,true));}}
  if(new Set(roles.map(r=>r.address)).size!==roles.length)fail('native_role_alias','manifest');return roles;
}
function planManifest(input){
  const {manifest:m,derived:d,missing}=validateManifest(input),details=DETAILS[m.action],common={schema:TEMPLATE.schema,sourceBaseline:BASELINE,action:m.action,missing,readiness:{...OPEN},
    expectedEffect:details?.effect??null,preconditions:[...PRECONDITIONS,...(details?.guards||[])],resourcePolicy:RESOURCE_POLICY,
    errorGuide:{scope:'NONEXHAUSTIVE_SOURCE_GUIDANCE_ONLY; no live error observed or diagnosed',custom:ERROR_GUIDE,downstream:'System/Token/Stake/runtime errors may propagate; no universal decoder or guessed remedy.'}};
  if(missing.length)return {...common,status:'INCOMPLETE',plan:null};
  const roles=nativeRoles(m,d),data=Buffer.alloc(m.action==='prepare-withdrawal'?13:9);data.write(ACTIONS[m.action]);data[8]=1;
  if(m.action==='prepare-withdrawal')data.writeUInt32LE(Number(BigInt(m.withdrawal.validatorIndex)),9);
  const instruction=new TransactionInstruction({programId:pk(m.programId),keys:roles.map(r=>({pubkey:pk(r.address),isSigner:false,isWritable:r.writable})),data});
  const packet=initializer.packet({m},m.action,[...budgets(m.resources),instruction]);assert.equal(packet.signatureCount,1);assert.deepEqual(packet.requiredSigners,[m.feePayer]);
  const context=m.action.startsWith('prepare-')?{kind:'CURRENT_REGISTRY',...m.currentGuardians}:m.action==='settle'?{kind:'FROZEN_ROUND',...m.frozenSettlement}:null;
  return {...common,status:'OFFLINE_PLAN_ONLY',declarationSha256:sha(Buffer.from(JSON.stringify(m))),plan:{program:m.programId,feePayer:m.feePayer,derived:d,roles,
    native:{bytes:data.length,hex:data.toString('hex'),sha256:sha(data)},guardianContext:context,withdrawal:m.action==='prepare-withdrawal'?m.withdrawal:null,
    requestedResources:m.resources,referenceContext:{...m.transport,verified:false,usage:'ENCODING_INPUT_ONLY; refresh/revalidate before any future signed step'},packets:[packet]}};
}
function verifySources(){const existing=initializer.verifySources();for(const [file,expected]of Object.entries(SOURCE_PINS))assert.equal(sha(fs.readFileSync(path.join(REPO,file))),expected,`source binding: ${file}`);
  return {node:existing.node,web3:existing.web3,pins:SOURCE_PINS,initializerPins:existing.pins};}
function main(args){
  const sourceCheck=verifySources();if(args.length===1&&args[0]==='--template'){process.stdout.write(JSON.stringify(TEMPLATE,null,2)+'\n');return;}
  if(args.length!==2||args[0]!=='--manifest')fail('unsupported_arguments','arguments');const filename=path.resolve(args[1]),stat=fs.statSync(filename);
  if(!stat.isFile()||stat.size>65536)fail('public_manifest_file_required','manifest');const bytes=fs.readFileSync(filename);if(bytes.length>65536)fail('manifest_size','manifest');
  const report=planManifest(initializer.parseManifest(bytes.toString('utf8')));report.manifestInputSha256=sha(bytes);report.sourceCheck=sourceCheck;process.stdout.write(JSON.stringify(report,null,2)+'\n');
}
module.exports={BASELINE,SOURCE_PINS,TEMPLATE,IDS,ACTIONS,RESOURCE_POLICY,ERROR_GUIDE,rewardAddresses,validateManifest,nativeRoles,planManifest,verifySources};
if(require.main===module){try{main(process.argv.slice(2));}catch(e){process.stderr.write(JSON.stringify({status:'INVALID_OR_UNSUPPORTED_DISTRIBUTION_PLAN',code:e.code||'plan_validation_failed',field:e.field||'manifest'})+'\n');process.exitCode=1;}}
