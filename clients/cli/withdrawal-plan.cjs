'use strict';
// Public declarations and offline withdrawal encodings; no quote or readiness engine.
const assert=require('node:assert/strict'),fs=require('node:fs'),path=require('node:path');
const {createHash}=require('node:crypto');
const initializer=require('./initializer-plan.cjs'),intake=require('./intake-plan.cjs');
const t=require('../../validation/genesis-transport/transport.cjs');
const {PublicKey,TransactionInstruction,ComputeBudgetProgram}=require(t.SDK);
const SOURCE_PINS=Object.freeze({
  "programs/piv1/src/instructions/initiate_withdrawal_leg.rs": "1d192bce73eb02891ce406fb8f8b11527a0a7cd9a8740e09667c4857a2d19551",
  "programs/piv1/src/instructions/finalize_withdrawal_leg.rs": "c3c8f09a8b6a28aa9d6de94004a0b126a1c099f0d754505653e6d713a24790a0",
  "programs/piv1/src/withdrawal_leg_execution.rs": "b50e1a3ddc5594b5c8ef11ad16771ac0850cb856b7ba8ca1ad9ec6c3ddf814c1",
  "programs/piv1/src/withdrawal_finalization_execution.rs": "42b6fc96f3f7e8bb1e4d5a233baf9c142fb94e6c8d9d286fa589f12e872267c9",
  "programs/piv1/src/integrations/jito_withdrawal.rs": "69987addaaa6ef9054329bee660296dedc0bf63078e0e89e46295fef06d4b3cc",
  "programs/piv1/src/integrations/jito_withdrawal_preparation.rs": "b057cf6716a353d90bcb4993466243a09f9f31d0c2291166380b80d5983c94fa",
  "programs/piv1/src/integrations/jito_identity.rs": "0dba585d0454af35ce9c087cfaa260b384e9d3f3396413282641d9ca087354d3",
  "programs/piv1/src/accounts.rs": "08e3d28971160910088a40d74215d15a9c2d07b11142e28c4fb5162d725f1c80",
  "programs/piv1/src/state/config.rs": "9d7a57c87b6781b943e0e9c13d08f73a21058d400b40540c485e6c596b042198",
  "programs/piv1/src/state/distribution.rs": "a291d3fb94921cd036161ee669190074fd5f159fe0d9f523a98ef3b0fa25d3a3",
  "programs/piv1/src/state/transitions.rs": "a7319afe56f2bf5301e689f68832c23d05620d3e478830c651777125e9671cf2",
  "programs/piv1/src/state/reconciliation.rs": "0604726bc0470664d90da7040bccca34d2a062d3bfa02dc546d8b02214afc386",
  "programs/piv1/src/state_persistence.rs": "d26483f7e1b95a020623b85705ab0006cb884e85cff37a0666bafa83123401a5",
  "programs/piv1/src/instruction_boundary.rs": "0e082adeebe7cabe5f5e14f69b244c2f1464c3c2f4eacd0077173a9aad91ab9f",
  "programs/piv1/src/instruction_errors.rs": "3ae5001e0c90440e3f4384b4d94da67d35a20240877a462421daed8df5e8d7b7",
  "programs/piv1/src/events.rs": "8860d27171af88b8752841146d0d02915ec9dbe59083749bfdd2e4f86c620c44",
  "clients/cli/initializer-plan.cjs": "92c13a4205f4557c74f55b03ee2dfea458592938228a7a959f5a2baf35bbeb83",
  "clients/cli/intake-plan.cjs": "ae4bb850ca4a9c8432830a6d0e60f5398fb895f3b137ff428120dec6568b253c",
  "clients/cli/initializer-plan.test.cjs": "16c642256aac6183a5e65b0cb99494bd8255b3f19b1b5f564b2124dfc2880e58",
  "validation/genesis-transport/transport.cjs": "a7694ff07577988e96546a53008bc8ab85bbc0ae7cb9353dcfc25e6c5a3c3724",
  "validation/genesis-transport/production-initializer.cjs": "c9cf0e28fa21d13e8b23b697023ad807d40f609cf46473c5019e17c46f8383a7"
});
const BASELINE='e8f33f311f427c02529108fdd39fb85691f99479';
const TEMPLATE={schema:'piv1-withdrawal-plan-v1',cluster:'testnet',programId:null,feePayer:null,action:null,
  protocol:{validatorList:null,reserve:null,managerFee:null,referrer:null},roundSequence:null,
  initiation:{nextLegIndex:null,validatorIndex:null,sourceStake:null},finalization:{legIndex:null},
  transport:{recentBlockhash:null,lastValidBlockHeight:null,referenceSlot:null},
  resources:{computeUnits:null,heapBytes:null,loadedAccountBytes:null}};
const IDS=Object.freeze({...initializer.IDS,clock:'SysvarC1ock11111111111111111111111111111111',stakeHistory:'SysvarStakeHistory1111111111111111111111111'});
const RESOURCE_POLICY=intake.RESOURCE_POLICY,REPO=path.resolve(__dirname,'../..'),U64=(1n<<64n)-1n,U32=(1n<<32n)-1n;
const ACTIONS=Object.freeze({'initiate-withdrawal-leg':'initiation','finalize-withdrawal-leg':'finalization'});
const pk=s=>new PublicKey(s),sha=b=>createHash('sha256').update(b).digest('hex');
const POOL_WITHDRAW_AUTHORITY=PublicKey.findProgramAddressSync([pk(IDS.pool).toBuffer(),Buffer.from('withdraw')],pk(IDS.poolProgram))[0].toBase58();
const OPEN=Object.freeze({publicReady:false,deploymentAuthorized:false,signerControlVerified:false,chainSnapshotVerified:false,
  accountValidityVerified:false,resourceSufficiencyVerified:false,fees:null,rent:null,funding:null,eligibility:null,
  sourceCapacity:null,withdrawalInput:null,protectedMinimum:null,finalizedValue:null,cooldownReady:null,m4Reservation:'OPEN_UNWAIVED'});
const PRECONDITIONS=[
  'Declarations are unverified public encoding inputs. Refresh exact program/artifact, Config, round, protocol/custody accounts and blockhash context before any future signed step. No RPC, key files, signing or sending is implemented.',
  'Both actions require initialized authenticated fixed custody and active round sequence, unpaused WithdrawalActive, current pool update epoch equal to Clock.epoch, correct supply/holdings, normalized custody and production rent/owner/authority checks. Pause blocks both actions; RecoveryRequired means stop/report.',
  'Canonical protocol account declarations must match actual Config, including manager/referrer shared or distinct shape. No destination, amount, quote, timestamp, slippage, readiness flag or economic choice is accepted.',
  'Outer fee payer declares network-fee responsibility only; actual fees/funding/control remain unknown. OperationalSOL separately advances temporary Stake and metadata rents, excluded from historical principal/yield/pending/KIF; no rent amount or payer wallet requirement is calculated here.',
  'All native roles are externally nonsigner. PIV custody/authority/temporary PDAs sign only inside production. Requested CU/heap/loaded caps are explicit declarations, not measured sufficiency or a send-ready packet.',
];
const DETAILS={
  'initiate-withdrawal-leg':{effect:'Production selects maximum safe remaining-target input, performs protected pool withdrawal variant24, immediately deactivates new Stake, and atomically records the leg. This does not finalize or pay beneficiaries.',guards:[
    'roundSequence and nextLegIndex declare the CURRENT active_sequence/next_leg_index for PDA derivation, not eligibility. Production reads both from the round; only validatorIndex is encoded. Refresh after any competing successful leg; failed attempts do not consume an index. Full u64 values can encode while counter overflow still rejects live execution.',
    'Only an Active validator source is supported. Actual list index, vote/suffix-derived canonical source PDA, preferred validator, SPL source priority, current epoch/list balance, Stake authorities/lockup/flags/delegation and residual capacity must pass production. There is no transient/reserve/removal/preferred fallback or client capacity optimizer.',
    'Production queries real Stake minimum delegation and computes maximum-safe fill=min(remaining target,current capacity), dynamic/snapshot technical floors, fee/burn, configured <=1bps protection, future-round feasibility and residual HWM guards. No amount/minimum/fee estimate or valid candidate is inferred from this draft.',
    'Both temporary PDAs must be unused System-owned empty nonexecutable accounts; any prefund is normalized to pending contributions before OperationalSOL pays CURRENT actual rents. Stake pseudo-rent is not the operational advance. Exact pool postconditions and immediate real Stake deactivation must succeed atomically.',
  ]},
  'finalize-withdrawal-leg':{effect:'Production withdraws the complete ready Stake balance into fixed DistributionEscrow, returns original advanced rents to OperationalSOL, normalizes metadata excess to pending, records reward/loss and closes metadata atomically. Settlement is separate.',guards:[
    'roundSequence declares the current active_sequence; legIndex identifies an already initiated exact round/leg. Production authenticates metadata discriminator/both bumps/status and Stake PDA, vote, PIV staker/withdrawer, initiation/deactivation epoch and lockup. Out-of-order ready legs are supported; replay/already finalized rejects.',
    'Elapsed epoch or a local flag is not readiness. Actual pinned Stake withdrawal must prove inactivity against runtime Clock/Stake History; legacy Clock and StakeHistory account positions remain exact. No cooldown date, readiness or success is calculated here.',
    'Original recorded stake/metadata rents, not current rent or pseudo-rent, return to OperationalSOL. Whole Stake balance first enters fixed escrow, then original stake rent is returned; metadata original rent returns and excess becomes pending. Unsupported excess above original rent plus current delegation has ambiguous provenance and rejects.',
    'Cooldown reward is next-cycle yield excluded from this fixed distribution. Observed cooldown loss or residual HWM failure records RecoveryRequired without lowering HWM; stop/report. Target assignment, successful/finalized counts, exact escrow and all finalization predicates govern progress to EscrowFunded; no partial beneficiary payout is made here.',
  ]},
};
const ERROR_GUIDE=[[6017,'AccountAlias'],[6020,'AccountRentDeficit'],[6034,'InvalidLifecycle'],[6035,'PausedOperation'],[6041,'SequenceMismatch'],[6042,'LegIndexMismatch'],[6046,'InvalidCustodyObservation'],[6051,'TargetExceeded'],[6052,'NonMaximumSafeLegFill'],[6053,'TechnicalFloorNotMet'],[6054,'UsefulLegBoundExceeded'],[6064,'HighWaterMarkDecrease'],[6069,'InvalidSlippage'],[6072,'ArithmeticOverflow'],[6073,'RecoveryRequired']].map(([code,name])=>({code,name}));
const fail=(code,field)=>{const e=new Error(code);e.code=code;e.field=field;throw e;};
const plain=v=>v!==null&&typeof v==='object'&&!Array.isArray(v)&&Object.getPrototypeOf(v)===Object.prototype;
const present=v=>v!==null&&v!==undefined;
const le64=v=>{const b=Buffer.alloc(8);b.writeBigUInt64LE(BigInt(v));return b;};
function tempAddresses(program,sequence,index){
  return Object.fromEntries([['withdrawal_leg','withdrawal-leg'],['withdrawal_stake','withdrawal-stake']].map(([role,seed])=>{
    const [address,bump]=PublicKey.findProgramAddressSync([Buffer.from(seed),le64(sequence),le64(index)],pk(program));return [role,{address:address.toBase58(),bump}];}));
}
function validateManifest(input){
  function walk(value,shape,field){
    if(!present(value))return null;
    if(plain(shape)){if(!plain(value))fail('object_required',field||'manifest');if(Object.keys(value).some(k=>!Object.hasOwn(shape,k)))fail('unknown_field',field||'manifest');
      return Object.fromEntries(Object.entries(shape).map(([k,v])=>[k,walk(value[k],v,field?`${field}.${k}`:k)]));}return value;
  }
  if(!plain(input))fail('object_required','manifest');const m=walk(input,TEMPLATE,''),check=(v,f,fn)=>{if(present(v))fn(v,f);};
  const key=(v,f)=>{if(typeof v!=='string')fail('public_key_string',f);let p;try{p=pk(v);}catch{fail('invalid_public_key',f);}if(p.toBase58()!==v||v===IDS.system)fail('noncanonical_or_zero_public_key',f);};
  const decimal=(v,f,max=U64)=>{if(typeof v!=='string'||!/^(?:0|[1-9]\d*)$/.test(v))fail('decimal_string_required',f);if(BigInt(v)>max)fail('integer_range',f);};
  check(m.schema,'schema',v=>{if(v!==TEMPLATE.schema)fail('unsupported_schema','schema');});check(m.cluster,'cluster',v=>{if(v!=='testnet')fail('unsupported_cluster','cluster');});
  check(m.action,'action',v=>{if(typeof v!=='string'||!Object.hasOwn(ACTIONS,v))fail('unsupported_action','action');});
  for(const f of ['programId','feePayer'])check(m[f],f,key);
  if(m.protocol)for(const [f,v]of Object.entries(m.protocol))check(v,`protocol.${f}`,key);
  check(m.roundSequence,'roundSequence',decimal);check(m.initiation?.nextLegIndex,'initiation.nextLegIndex',decimal);
  check(m.initiation?.validatorIndex,'initiation.validatorIndex',(v,f)=>decimal(v,f,U32));check(m.initiation?.sourceStake,'initiation.sourceStake',key);
  check(m.finalization?.legIndex,'finalization.legIndex',decimal);
  if(m.transport){check(m.transport.recentBlockhash,'transport.recentBlockhash',key);for(const f of ['lastValidBlockHeight','referenceSlot'])check(m.transport[f],`transport.${f}`,decimal);}
  if(m.resources)for(const [f,limits]of Object.entries(RESOURCE_POLICY))if(f!=='meaning')check(m.resources[f],`resources.${f}`,(v,field)=>{
    if(!Number.isSafeInteger(v)||v<limits.min||v>limits.max||limits.multiple&&v%limits.multiple!==0)fail('unsupported_resource_request',field);
  });
  if(present(m.feePayer)&&!PublicKey.isOnCurve(pk(m.feePayer).toBytes()))fail('unsupported_off_curve_fee_signer','feePayer');
  const reserved=new Set([...Object.values(IDS),POOL_WITHDRAW_AUTHORITY]);
  if(present(m.programId)&&reserved.has(m.programId))fail('unsupported_program_identity','programId');
  const d=present(m.programId)?intake.derived(m.programId):null,temporary=[];
  // Validate every derivable known partial, including the currently unused action block.
  if(d&&present(m.roundSequence))for(const index of [m.initiation?.nextLegIndex,m.finalization?.legIndex].filter(present))temporary.push(...Object.values(tempAddresses(m.programId,m.roundSequence,index)).map(v=>v.address));
  const protectedKeys=new Set([...reserved,...(d?[m.programId,...Object.values(d)]:[]),...temporary]);
  const native=[...Object.entries(m.protocol||{}).map(([f,v])=>[`protocol.${f}`,v]),['initiation.sourceStake',m.initiation?.sourceStake]].filter(([,v])=>present(v));
  for(const [f,v]of [...native,['feePayer',m.feePayer]])if(present(v)&&protectedKeys.has(v))fail('unsupported_protected_alias',f);
  for(let a=0;a<native.length;a++)for(let b=0;b<a;b++)if(native[a][1]===native[b][1]&&![native[a][0],native[b][0]].every(f=>['protocol.managerFee','protocol.referrer'].includes(f)))fail('native_role_alias',native[a][0]);
  if(present(m.feePayer)&&native.some(([,v])=>v===m.feePayer))fail('fee_payer_native_alias','feePayer');
  if(present(m.action))for(const block of ['initiation','finalization'])if(block!==ACTIONS[m.action]&&m[block]&&Object.values(m[block]).some(present))fail('unused_action_input',block);
  const fields=['schema','cluster','programId','feePayer','action','roundSequence',...['protocol','transport','resources'].flatMap(b=>Object.keys(TEMPLATE[b]).map(f=>`${b}.${f}`))];
  if(present(m.action))fields.push(...Object.keys(TEMPLATE[ACTIONS[m.action]]).map(f=>`${ACTIONS[m.action]}.${f}`));
  return {manifest:m,derived:d,missing:fields.filter(f=>f.split('.').reduce((v,k)=>v?.[k],m)==null)};
}
function budgets(resources){
  const loaded=Buffer.alloc(5);loaded[0]=4;loaded.writeUInt32LE(resources.loadedAccountBytes,1);
  const result=[ComputeBudgetProgram.setComputeUnitLimit({units:resources.computeUnits}),ComputeBudgetProgram.requestHeapFrame({bytes:resources.heapBytes}),new TransactionInstruction({programId:pk(IDS.computeBudget),keys:[],data:loaded})];
  assert.deepEqual(result.map(i=>i.data[0]),[2,1,4]);assert(result.every(i=>i.keys.length===0&&i.data.length===5));return result;
}
function nativeRoles(m,d,temp){
  const initiate=m.action==='initiate-withdrawal-leg',roles=[],add=(role,address,writable=false)=>roles.push({role,address,signer:false,writable});
  const writable=initiate?[0,1,2,4,7]:[0,1,2,4,5];
  ['config','active_distribution','pending_sol','principal_sol','operational_sol','distribution_escrow','kif_sol','principal_token','pending_token'].forEach((role,i)=>add(role,d[role],writable.includes(i)));
  add('piv_authority',d.piv_authority);add('mint',IDS.mint,initiate);add('system_program',IDS.system);add('token_program',IDS.token);add('pool_program',IDS.poolProgram);add('pool',IDS.pool,initiate);
  add('validator_list',m.protocol.validatorList,initiate);add('reserve',m.protocol.reserve);add('manager_fee',m.protocol.managerFee,initiate);if(m.protocol.managerFee!==m.protocol.referrer)add('referrer',m.protocol.referrer);
  add('clock',IDS.clock);add('stake_program',IDS.stake);
  if(initiate){add('source_stake',m.initiation.sourceStake,true);add('pool_withdraw_authority',POOL_WITHDRAW_AUTHORITY);}else add('stake_history',IDS.stakeHistory);
  add('withdrawal_leg',temp.withdrawal_leg.address,true);add('withdrawal_stake',temp.withdrawal_stake.address,true);
  if(new Set(roles.map(r=>r.address)).size!==roles.length)fail('native_role_alias','manifest');return roles;
}
function planManifest(input){
  const {manifest:m,derived:d,missing}=validateManifest(input),details=DETAILS[m.action],common={schema:TEMPLATE.schema,sourceBaseline:BASELINE,action:m.action,missing,readiness:{...OPEN},expectedEffect:details?.effect??null,
    preconditions:[...PRECONDITIONS,...(details?.guards||[])],resourcePolicy:RESOURCE_POLICY,errorGuide:{scope:'NONEXHAUSTIVE_SOURCE_GUIDANCE_ONLY; no live error observed or diagnosed',custom:ERROR_GUIDE,downstream:'System/Token/pool/Stake/runtime errors may propagate; no universal decoder or guessed remedy.'}};
  if(missing.length)return {...common,status:'INCOMPLETE',plan:null};
  const initiate=m.action==='initiate-withdrawal-leg',index=initiate?m.initiation.nextLegIndex:m.finalization.legIndex,temp=tempAddresses(m.programId,m.roundSequence,index),roles=nativeRoles(m,d,temp);
  const data=Buffer.alloc(initiate?13:17);data.write(initiate?'PIV1IL01':'PIV1FL01');data[8]=1;
  if(initiate)data.writeUInt32LE(Number(BigInt(m.initiation.validatorIndex)),9);else data.writeBigUInt64LE(BigInt(m.finalization.legIndex),9);
  const instruction=new TransactionInstruction({programId:pk(m.programId),keys:roles.map(r=>({pubkey:pk(r.address),isSigner:false,isWritable:r.writable})),data});
  const packet=initializer.packet({m},m.action,[...budgets(m.resources),instruction]);assert.equal(packet.signatureCount,1);assert.deepEqual(packet.requiredSigners,[m.feePayer]);
  return {...common,status:'OFFLINE_PLAN_ONLY',declarationSha256:sha(Buffer.from(JSON.stringify(m))),plan:{program:m.programId,feePayer:m.feePayer,derived:{...d,...temp,...(initiate?{pool_withdraw_authority:POOL_WITHDRAW_AUTHORITY}:{})},roles,
    native:{bytes:data.length,hex:data.toString('hex'),sha256:sha(data)},roundDeclaration:{roundSequence:m.roundSequence,legIndex:index,meaning:initiate?'DECLARED_CURRENT_NEXT_LEG_INDEX; not eligibility proof':'DECLARED_EXISTING_LEG_INDEX; not readiness proof'},
    initiation:initiate?m.initiation:null,finalization:initiate?null:m.finalization,requestedResources:m.resources,referenceContext:{...m.transport,verified:false,usage:'ENCODING_INPUT_ONLY; refresh/revalidate before any future signed step'},packets:[packet]}};
}
function verifySources(){const existing=initializer.verifySources();for(const [file,expected]of Object.entries(SOURCE_PINS))assert.equal(sha(fs.readFileSync(path.join(REPO,file))),expected,`source binding: ${file}`);
  return {node:existing.node,web3:existing.web3,pins:SOURCE_PINS,initializerPins:existing.pins};}
function main(args){
  const sourceCheck=verifySources();if(args.length===1&&args[0]==='--template'){process.stdout.write(JSON.stringify(TEMPLATE,null,2)+'\n');return;}
  if(args.length!==2||args[0]!=='--manifest')fail('unsupported_arguments','arguments');const filename=path.resolve(args[1]),stat=fs.statSync(filename);
  if(!stat.isFile()||stat.size>65536)fail('public_manifest_file_required','manifest');const bytes=fs.readFileSync(filename);if(bytes.length>65536)fail('manifest_size','manifest');
  const report=planManifest(initializer.parseManifest(bytes.toString('utf8')));report.manifestInputSha256=sha(bytes);report.sourceCheck=sourceCheck;process.stdout.write(JSON.stringify(report,null,2)+'\n');
}
module.exports={BASELINE,SOURCE_PINS,TEMPLATE,IDS,RESOURCE_POLICY,POOL_WITHDRAW_AUTHORITY,ERROR_GUIDE,validateManifest,tempAddresses,nativeRoles,planManifest,verifySources};
if(require.main===module){try{main(process.argv.slice(2));}catch(e){process.stderr.write(JSON.stringify({status:'INVALID_OR_UNSUPPORTED_WITHDRAWAL_PLAN',code:e.code||'plan_validation_failed',field:e.field||'manifest'})+'\n');process.exitCode=1;}}
