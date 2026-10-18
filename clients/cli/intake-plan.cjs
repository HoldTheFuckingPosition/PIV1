'use strict';
// One unsigned public action draft. No account snapshots, wallet or network APIs.
const assert=require('node:assert/strict');
const fs=require('node:fs');
const path=require('node:path');
const {createHash}=require('node:crypto');
const initializer=require('./initializer-plan.cjs');
const t=require('../../validation/genesis-transport/transport.cjs');
const {PublicKey,TransactionInstruction,ComputeBudgetProgram}=require(t.SDK);
const SOURCE_PINS=Object.freeze({
  "programs/piv1/src/instructions/deposit_sol.rs": "48be72aed608653a05c6b5ca1143fdee01665fe9c07056307b7a1b5e9b05704e",
  "programs/piv1/src/instructions/deposit_jitosol.rs": "b5783598c6e95c321a7acc309bdd366e7c7b79d620fdb5f126c86622fc496a92",
  "programs/piv1/src/instructions/bootstrap_initial_contributions.rs": "17abc5f7e7f2c62ff4c58c22896e98d2e1931f24dbb09c55f90685741926fabd",
  "programs/piv1/src/contribution_execution.rs": "6522424311ff114b574df12239080c5dec790c3c69bf9e8da00c305c7f4b1228",
  "programs/piv1/src/initial_bootstrap_execution.rs": "6a355b50eb3ab28e7620177e3b19290b840556f264188b695ab424fd33397d29",
  "programs/piv1/src/pending_accounts.rs": "6d994ad5aa9c2beada61709d3a87835f54aa09897ee9d8ff8e487194fb38a3b3",
  "programs/piv1/src/accounts.rs": "08e3d28971160910088a40d74215d15a9c2d07b11142e28c4fb5162d725f1c80",
  "programs/piv1/src/state/contributions.rs": "6e5210c6f5b76f31fb747374f8edff6de7c1faf7a415261aee52c3e34695aa3c",
  "programs/piv1/src/state/bootstrap.rs": "fc7debd02ca0f964bc9e813280be64834b1bc4bd12573e2fd9f270cb8ae673a5",
  "programs/piv1/src/state/reconciliation.rs": "0604726bc0470664d90da7040bccca34d2a062d3bfa02dc546d8b02214afc386",
  "programs/piv1/src/instruction_boundary.rs": "d631db4329c72720cf6acaa4259cb7f244d7b84fa8db1266ad9a15881a3e773c",
  "programs/piv1/src/instruction_errors.rs": "3ae5001e0c90440e3f4384b4d94da67d35a20240877a462421daed8df5e8d7b7",
  "programs/piv1/src/integrations/jito_identity.rs": "0dba585d0454af35ce9c087cfaa260b384e9d3f3396413282641d9ca087354d3",
  "validation/genesis-bank-runtime/tests/lifecycle_fixture.rs": "84e9f5a735fd33c4c5a92cf7833b15ade7a95d952d014cb95d95a2e93c3fddf4",
  "validation/genesis-bank-runtime/tests/real_squads_initializer.rs": "181bf8a47dfa674df20563229e7e7b89cccce200a92ea97112b7391bb19787c9",
  "clients/cli/initializer-plan.cjs": "347d5ab436b3e8878fd337c53ef8636721ca64a6465bc86c06f2637921ac6f6f",
  "clients/cli/initializer-plan.test.cjs": "7616bb36648e7d4fc835f80bbe27b0b67cd4a96a2a9f2540238b04e5acbbc1cb"
});
const BASELINE='b4827e09dc5092469afbd7fceb9ee6cc08cd4b6c';
const TEMPLATE={
  "schema": "piv1-intake-plan-v1",
  "cluster": "testnet",
  "programId": null,
  "feePayer": null,
  "action": null,
  "sol": {
    "donor": null,
    "amountLamports": null
  },
  "jitosol": {
    "owner": null,
    "sourceToken": null,
    "amountUnits": null
  },
  "bootstrap": {
    "validatorList": null,
    "reserve": null,
    "managerFee": null,
    "referrer": null
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
const IDS=initializer.IDS,U64=(1n<<64n)-1n,REPO=path.resolve(__dirname,'../..');
const ACTIONS={'contribute-sol':'sol','contribute-jitosol':'jitosol','bootstrap-initial':'bootstrap'};
const SEEDS={config:'config',active_distribution:'distribution',pending_sol:'pending-sol',principal_sol:'principal-sol',
  operational_sol:'operational-sol',distribution_escrow:'distribution-escrow',kif_sol:'kif-sol',principal_token:'principal-jito-vault',
  pending_token:'pending-jito-vault',piv_authority:'authority',guardian_registry:'guardian-registry'};
const RESOURCE_POLICY=Object.freeze({computeUnits:{min:1,max:1400000},heapBytes:{min:32768,max:262144,multiple:1024},
  loadedAccountBytes:{min:1,max:8388608},meaning:'CLIENT_SUPPORTED_REQUEST_RANGES_ONLY; not measured public sufficiency or all runtime limits'});
const OPEN=Object.freeze({publicReady:false,deploymentAuthorized:false,signerControlVerified:false,chainSnapshotVerified:false,
  accountValidityVerified:false,resourceSufficiencyVerified:false,fees:null,rent:null,funding:null,bootstrapEligible:null,
  bootstrapSolLamports:null,bootstrapTokenUnits:null,bootstrapBookValueLamports:null,m4Reservation:'OPEN_UNWAIVED'});
const ERROR_GUIDE=[
  [6017,'AccountAlias','Separate supplied native roles must not alias.'],[6022,'InvalidTokenCustody','Legacy Token owner/mint/state/authorization constraints failed.'],
  [6026,'InvalidBootstrapState','Initial bootstrap rejects prior economic history.'],[6034,'InvalidLifecycle','Initial bootstrap requires the initial Idle state.'],
  [6035,'PausedOperation','Explicit intake and bootstrap reject while paused.'],[6045,'ZeroContribution','Contribution amount or both recognized bootstrap assets are zero.'],
  [6046,'InvalidCustodyObservation','Custody or current pool valuation facts are unsupported.'],[6048,'ContributionObservationMismatch','Exact transfer/post-state facts differ.'],
  [6049,'PendingCustodyDeficit','Old pending obligations are not covered.'],
].map(([code,name,meaning])=>({code,name,meaning}));
const COMMON_PRECONDITIONS=[
  'All identities/context are declarations, not authenticated current accounts or signatures. Independently verify exact selected program/artifact/cluster and Config binding.',
  'Initialized canonical Config/distribution and custody PDAs must have valid owner/data/rent/mint/authority and current state binding; program must be unpaused.',
  'Refresh the blockhash/lastValidBlockHeight for this future signed step and recheck state, balances, funding and resource requests; this draft is never send-ready.',
];
const DETAILS={
  'contribute-sol':{
    expectedEffect:'Transfer the exact requested lamports from external donor to pending SOL; increment only accounted pending SOL and emit contribution event. No ownership/withdrawal/reward right, principal integration, staking or HWM increase.',
    preconditions:['Intake is not Idle-only. BOTH old SOL pending obligations (including active-round offset) and old token pending obligations must already be covered.',
      'Donor must be a valid external System-owned nonexecutable empty account, not PIV-owned/internal; signer+writable, sufficient lamports and valid post-transaction balance.'],
  },
  'contribute-jitosol':{
    expectedEffect:'Legacy Token TransferChecked of exact requested units from explicit source to pending token custody; increment only accounted pending token units/event. No mint/burn, ownership/reward right or principal/HWM change.',
    preconditions:['Intake is not Idle-only. BOTH old SOL and token pending obligations must already be covered.',
      'Source is explicit ATA-or-nonATA; verify rent-backed165-byte legacy Token-owned initialized non-native account, canonical configured JitoSOL mint, token.owner==declared owner and enough units. Verify rent-backed82-byte legacy mint.',
      'Owner signer may have arbitrary chain account owner except PIV/internal restrictions and exact Token-owned355-byte SPL multisig. System ownership is not required. Delegated-only authority is unsupported.',
      'If source.delegate equals owner, Token can consume delegated allowance even though owner signs; verify sufficient allowance. No delegate override/revoke is planned.'],
  },
  'bootstrap-initial':{
    expectedEffect:'Production derives ALL recognized pending amounts from current Config, transfers same assets to principal via canonical PDA-signed System/Token CPIs, clears pending ledgers and updates initial history/HWM/contribution value from current gross pool book value. No caller amount, Squads approval, native signer, staking or pool/Stake CPI.',
    preconditions:['Initial Idle/zero economic history only: no completed/prepared/insufficient history or nonzero required counters; at least one recognized pending asset positive. No later integration/replay is supported.',
      'No unexplained economic custody surplus or deficit; exact expected movements and rent floors must hold. Both Token native balances remain preserved/quarantined, not token value.',
      'Declared pool/list/reserve/mint/receiver topology and18/19 manager/referrer shape must match actual Config and current official accounts.',
      'Pool last update epoch must equal Clock.epoch; total/supply consistency and Mint.supply<=recorded pool supply must hold. Value uses recorded pool supply, not a caller quote.',
      'Positive token units with floored book value0 remain valid; no minimum book value is invented. Bootstrap amounts, book value and eligibility remain unknown.'],
  },
};
const sha=b=>createHash('sha256').update(b).digest('hex'),pk=s=>new PublicKey(s);
const fail=(code,field)=>{const e=new Error(code);e.code=code;e.field=field;throw e;};
const plain=v=>v!==null&&typeof v==='object'&&!Array.isArray(v)&&Object.getPrototypeOf(v)===Object.prototype;
const derived=program=>Object.fromEntries(Object.entries(SEEDS).map(([role,seed])=>[role,
  PublicKey.findProgramAddressSync([Buffer.from(seed)],pk(program))[0].toBase58()]));
function validateManifest(input) {
  function walk(value,shape,field) {
    if(value===null||value===undefined)return null;
    if(plain(shape)) {
      if(!plain(value))fail('object_required',field||'manifest');
      if(Object.keys(value).some(k=>!Object.hasOwn(shape,k)))fail('unknown_field',field||'manifest');
      return Object.fromEntries(Object.entries(shape).map(([k,v])=>[k,walk(value[k],v,field?`${field}.${k}`:k)]));
    }
    return value;
  }
  if(!plain(input))fail('object_required','manifest');const m=walk(input,TEMPLATE,''),check=(value,field,fn)=>{if(value!==null)fn(value,field);};
  check(m.schema,'schema',v=>{if(v!==TEMPLATE.schema)fail('unsupported_schema','schema');});
  check(m.cluster,'cluster',v=>{if(v!=='testnet')fail('unsupported_cluster','cluster');});
  check(m.action,'action',v=>{if(typeof v!=='string'||!Object.hasOwn(ACTIONS,v))fail('unsupported_action','action');});
  const key=(value,field)=>{if(typeof value!=='string')fail('public_key_string',field);let p;try{p=pk(value);}catch{fail('invalid_public_key',field);}
    if(p.toBase58()!==value||value===IDS.system)fail('noncanonical_or_zero_public_key',field);};
  const decimal=(value,field,positive=false)=>{
    if(typeof value!=='string'||!/^(?:0|[1-9]\d*)$/.test(value))fail('decimal_string_required',field);
    const n=BigInt(value);if(n>U64||positive&&n===0n)fail('integer_range',field);
  };
  for(const field of ['programId','feePayer'])check(m[field],field,key);
  for(const [block,field]of [['sol','donor'],['jitosol','owner'],['jitosol','sourceToken'],...Object.keys(TEMPLATE.bootstrap).map(k=>['bootstrap',k])])
    if(m[block])check(m[block][field],`${block}.${field}`,key);
  for(const [block,field]of [['sol','amountLamports'],['jitosol','amountUnits']])if(m[block])check(m[block][field],`${block}.${field}`,(v,f)=>decimal(v,f,true));
  if(m.transport) {
    check(m.transport.recentBlockhash,'transport.recentBlockhash',key);
    for(const field of ['lastValidBlockHeight','referenceSlot'])check(m.transport[field],`transport.${field}`,decimal);
  }
  if(m.resources)for(const [field,limits]of Object.entries(RESOURCE_POLICY))if(field!=='meaning')check(m.resources[field],`resources.${field}`,(v,f)=>{
    if(!Number.isSafeInteger(v)||v<limits.min||v>limits.max||limits.multiple&&v%limits.multiple!==0)fail('unsupported_resource_request',f);
  });
  for(const [field,value]of [['feePayer',m.feePayer],['sol.donor',m.sol?.donor],['jitosol.owner',m.jitosol?.owner]])
    if(value!==null&&value!==undefined&&!PublicKey.isOnCurve(pk(value).toBytes()))fail('unsupported_off_curve_outer_signer',field);
  const protectedKeys=new Set([...Object.values(IDS),...(m.programId?[m.programId,...Object.values(derived(m.programId))]:[])]);
  if(m.programId!==null&&Object.values(IDS).includes(m.programId))fail('unsupported_program_identity','programId');
  for(const [field,value]of [['feePayer',m.feePayer],['sol.donor',m.sol?.donor],['jitosol.owner',m.jitosol?.owner],['jitosol.sourceToken',m.jitosol?.sourceToken],
    ...Object.entries(m.bootstrap||{}).map(([k,v])=>[`bootstrap.${k}`,v])])if(value!==null&&value!==undefined&&protectedKeys.has(value))fail('unsupported_protected_alias',field);
  if(m.jitosol?.sourceToken!==null&&m.jitosol?.sourceToken!==undefined&&[m.jitosol.owner,m.feePayer].includes(m.jitosol.sourceToken))fail('source_signer_alias','jitosol.sourceToken');
  const protocol=Object.entries(m.bootstrap||{}).filter(([,v])=>v!==null);
  for(let a=0;a<protocol.length;a++)for(let b=0;b<a;b++)if(protocol[a][1]===protocol[b][1]&&![protocol[a][0],protocol[b][0]].every(k=>['managerFee','referrer'].includes(k)))fail('protocol_alias','bootstrap');
  if(m.action==='bootstrap-initial'&&protocol.some(([,v])=>v===m.feePayer))fail('fee_payer_protocol_alias','feePayer');
  if(m.action!==null)for(const block of ['sol','jitosol','bootstrap'])if(block!==ACTIONS[m.action]&&m[block]&&Object.values(m[block]).some(v=>v!==null))fail('unused_action_input',block);
  const missing=[],required=['schema','cluster','programId','feePayer','action',...Object.keys(TEMPLATE.transport).map(k=>`transport.${k}`),...Object.keys(TEMPLATE.resources).map(k=>`resources.${k}`)];
  if(m.action!==null)required.push(...Object.keys(TEMPLATE[ACTIONS[m.action]]).map(k=>`${ACTIONS[m.action]}.${k}`));
  for(const field of required)if(field.split('.').reduce((v,k)=>v?.[k],m)==null)missing.push(field);
  return {manifest:m,missing};
}
function budgets(resources) {
  const loaded=Buffer.alloc(5);loaded[0]=4;loaded.writeUInt32LE(resources.loadedAccountBytes,1);
  const result=[ComputeBudgetProgram.setComputeUnitLimit({units:resources.computeUnits}),ComputeBudgetProgram.requestHeapFrame({bytes:resources.heapBytes}),
    new TransactionInstruction({programId:pk(IDS.computeBudget),keys:[],data:loaded})];
  assert.deepEqual(result.map(i=>i.data[0]),[2,1,4]);assert(result.every(i=>i.keys.length===0&&i.data.length===5));
  assert.deepEqual(result.map(i=>i.data.readUInt32LE(1)),[resources.computeUnits,resources.heapBytes,resources.loadedAccountBytes]);return result;
}
function actionInstruction(m,d) {
  const roles=[],add=(role,address,writable=false,signer=false)=>roles.push({role,address,signer,writable});let selector,amount=null;
  if(m.action==='bootstrap-initial') {
    selector='PIV1IB01';
    for(const role of ['config','active_distribution','pending_sol','principal_sol','operational_sol','distribution_escrow','kif_sol','principal_token','pending_token'])
      add(role,d[role],['config','pending_sol','principal_sol','principal_token','pending_token'].includes(role));
    add('piv_authority',d.piv_authority);add('mint',IDS.mint);add('system_program',IDS.system);add('token_program',IDS.token);
    add('pool_program',IDS.poolProgram);add('pool',IDS.pool);add('validator_list',m.bootstrap.validatorList);add('reserve',m.bootstrap.reserve);add('manager_fee',m.bootstrap.managerFee);
    if(m.bootstrap.managerFee!==m.bootstrap.referrer)add('referrer',m.bootstrap.referrer);
  } else {
    const sol=m.action==='contribute-sol';selector=sol?'PIV1DS01':'PIV1DJ01';amount=sol?m.sol.amountLamports:m.jitosol.amountUnits;
    add('config',d.config,true);add('active_distribution',d.active_distribution);add('pending_sol',d.pending_sol,sol);add('pending_token',d.pending_token,!sol);
    if(sol){add('donor',m.sol.donor,true,true);add('system_program',IDS.system);}
    else {add('source_token',m.jitosol.sourceToken,true);add('owner',m.jitosol.owner,false,true);add('mint',IDS.mint);add('token_program',IDS.token);}
  }
  assert.equal(new Set(roles.map(r=>r.address)).size,roles.length);
  const data=Buffer.alloc(amount===null?9:17);data.write(selector);data[8]=1;if(amount!==null)data.writeBigUInt64LE(BigInt(amount),9);
  const instruction=new TransactionInstruction({programId:pk(m.programId),keys:roles.map(r=>({pubkey:pk(r.address),isSigner:r.signer,isWritable:r.writable})),data});
  return {roles,data,instruction,amount};
}
function planManifest(input) {
  const {manifest:m,missing}=validateManifest(input),details=DETAILS[m.action];
  const common={schema:TEMPLATE.schema,sourceBaseline:BASELINE,action:m.action,missing,readiness:{...OPEN},
    preconditions:[...COMMON_PRECONDITIONS,...(details?.preconditions||[])],expectedEffect:details?.expectedEffect??null,
    resourcePolicy:RESOURCE_POLICY,errorGuide:{scope:'NONEXHAUSTIVE_SOURCE_GUIDANCE_ONLY; no live error observed/diagnosed',custom:ERROR_GUIDE,
      standard:['MissingRequiredSignature','InsufficientFunds','InvalidInstructionData','NotEnoughAccountKeys','InvalidArgument'],downstreamErrors:'System/Token/runtime errors can propagate; this is not a universal decoder.'}};
  if(missing.length)return {...common,status:'INCOMPLETE',plan:null};
  const d=derived(m.programId),action=actionInstruction(m,d),packet=initializer.packet({m},m.action,[...budgets(m.resources),action.instruction]);
  assert.equal(packet.signatureCount,m.action==='bootstrap-initial'?1:m.feePayer===(m.action==='contribute-sol'?m.sol.donor:m.jitosol.owner)?1:2);
  return {...common,status:'OFFLINE_PLAN_ONLY',declarationSha256:sha(Buffer.from(JSON.stringify(m))),
    plan:{program:m.programId,feePayer:m.feePayer,derived:d,roles:action.roles,native:{bytes:action.data.length,hex:action.data.toString('hex'),sha256:sha(action.data)},
      amount:action.amount===null?null:{value:action.amount,unit:m.action==='contribute-sol'?'lamports':'legacy JitoSOL base units'},
      bootstrapAmounts:null,bootstrapBookValueLamports:null,requestedResources:m.resources,
      referenceContext:{...m.transport,verified:false,usage:'ENCODING_INPUT_ONLY; independently refresh/revalidate before any future signed step'},packets:[packet]}};
}
function verifySources() {
  const existing=initializer.verifySources();
  for(const [file,expected]of Object.entries(SOURCE_PINS))assert.equal(sha(fs.readFileSync(path.join(REPO,file))),expected,`source binding: ${file}`);
  return {node:existing.node,web3:existing.web3,pins:SOURCE_PINS,initializerPins:existing.pins};
}
function main(args) {
  const sourceCheck=verifySources();
  if(args.length===1&&args[0]==='--template'){process.stdout.write(JSON.stringify(TEMPLATE,null,2)+'\n');return;}
  if(args.length!==2||args[0]!=='--manifest')fail('unsupported_arguments','arguments');
  const filename=path.resolve(args[1]),stat=fs.statSync(filename);if(!stat.isFile()||stat.size>65536)fail('public_manifest_file_required','manifest');
  const bytes=fs.readFileSync(filename);if(bytes.length>65536)fail('manifest_size','manifest');
  const report=planManifest(initializer.parseManifest(bytes.toString('utf8')));report.manifestInputSha256=sha(bytes);report.sourceCheck=sourceCheck;
  process.stdout.write(JSON.stringify(report,null,2)+'\n');
}
module.exports={BASELINE,SOURCE_PINS,TEMPLATE,IDS,RESOURCE_POLICY,ERROR_GUIDE,validateManifest,derived,planManifest,verifySources};
if(require.main===module){try{main(process.argv.slice(2));}catch(e){process.stderr.write(JSON.stringify({status:'INVALID_OR_UNSUPPORTED_INTAKE_PLAN',code:e.code||'plan_validation_failed',field:e.field||'manifest'})+'\n');process.exitCode=1;}}
