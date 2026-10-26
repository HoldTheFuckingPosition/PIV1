'use strict';
// Offline conversion of recognized HISTORICAL principal SOL, never pending intake.
const assert=require('node:assert/strict');
const fs=require('node:fs');
const path=require('node:path');
const {createHash}=require('node:crypto');
const initializer=require('./initializer-plan.cjs');
const intake=require('./intake-plan.cjs');
const t=require('../../validation/genesis-transport/transport.cjs');
const {PublicKey,TransactionInstruction,ComputeBudgetProgram}=require(t.SDK);
const SOURCE_PINS=Object.freeze({
  "programs/piv1/src/instructions/stake_pending_sol.rs": "fd4fedcd8f70de2600fae68bc162095e9bf81b363d32015eb99be3aa774ed1d6",
  "programs/piv1/src/principal_deposit_execution.rs": "90469cae015c29ce909b8972836f4eb0c171d8be37f7e66aba618bdc989a88b2",
  "programs/piv1/src/state/principal_deposit.rs": "4b8a7daa930a3cd856ac3aa30c33cf4bf24c996719ea4575a3eaa42cbaa0c4d3",
  "programs/piv1/src/integrations/jito_deposit.rs": "3732c29d4fb493c0d91aa4827a31cf229706ddf73d18c0c49ed2a23edc0e6801",
  "programs/piv1/src/integrations/jito_identity.rs": "0dba585d0454af35ce9c087cfaa260b384e9d3f3396413282641d9ca087354d3",
  "programs/piv1/src/accounts.rs": "08e3d28971160910088a40d74215d15a9c2d07b11142e28c4fb5162d725f1c80",
  "programs/piv1/src/state/reconciliation.rs": "0604726bc0470664d90da7040bccca34d2a062d3bfa02dc546d8b02214afc386",
  "programs/piv1/src/state/config.rs": "9d7a57c87b6781b943e0e9c13d08f73a21058d400b40540c485e6c596b042198",
  "programs/piv1/src/instruction_boundary.rs": "0e082adeebe7cabe5f5e14f69b244c2f1464c3c2f4eacd0077173a9aad91ab9f",
  "programs/piv1/src/instruction_errors.rs": "3ae5001e0c90440e3f4384b4d94da67d35a20240877a462421daed8df5e8d7b7",
  "programs/piv1/src/events.rs": "8860d27171af88b8752841146d0d02915ec9dbe59083749bfdd2e4f86c620c44",
  "validation/genesis-bank-runtime/tests/lifecycle_fixture.rs": "84e9f5a735fd33c4c5a92cf7833b15ade7a95d952d014cb95d95a2e93c3fddf4",
  "clients/cli/initializer-plan.cjs": "92c13a4205f4557c74f55b03ee2dfea458592938228a7a959f5a2baf35bbeb83",
  "clients/cli/intake-plan.cjs": "ae4bb850ca4a9c8432830a6d0e60f5398fb895f3b137ff428120dec6568b253c",
  "clients/cli/initializer-plan.test.cjs": "16c642256aac6183a5e65b0cb99494bd8255b3f19b1b5f564b2124dfc2880e58"
});
const BASELINE='7216a6b58428bb8e17836e9f9acfcfe0a324c220';
const TEMPLATE={
  "schema": "piv1-principal-stake-plan-v1",
  "cluster": "testnet",
  "programId": null,
  "feePayer": null,
  "amountLamports": null,
  "callerMinimumPoolTokensOut": null,
  "protocol": {
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
const IDS=initializer.IDS,RESOURCE_POLICY=intake.RESOURCE_POLICY,REPO=path.resolve(__dirname,'../..'),U64=(1n<<64n)-1n;
const ACTION='stake-historical-principal-sol',pk=s=>new PublicKey(s);
const POOL_WITHDRAW_AUTHORITY=PublicKey.findProgramAddressSync([pk(IDS.pool).toBuffer(),Buffer.from('withdraw')],pk(IDS.poolProgram))[0].toBase58();
const OPEN=Object.freeze({publicReady:false,deploymentAuthorized:false,signerControlVerified:false,chainSnapshotVerified:false,
  accountValidityVerified:false,resourceSufficiencyVerified:false,fees:null,rent:null,funding:null,eligibility:null,
  quotedMinted:null,effectiveProtectedMinimum:null,bookValueBefore:null,bookValueAfter:null,m4Reservation:'OPEN_UNWAIVED'});
const PRECONDITIONS=[
  'This is recognized HISTORICAL-principal SOL conversion, not pending contribution staking. Amount must fit backed accounted_historical_sol_lamports in current valid principal SOL custody.',
  'Current initialized Config/fixed custody/round binding must pass production validation; unpaused and Idle. Whole economic custody must be normalized without unexplained surplus/deficit.',
  'Canonical official pool/list/reserve/mint/manager/referrer must match actual Config, including shared/distinct receiver shape and all owner/data/rent/authority checks.',
  'Pool update epoch must equal Clock.epoch; total/supply bootstrap consistency, Mint.supply<=recorded pool supply and combined PIV token holdings<=mint supply must hold.',
  'Only permissionless SOL deposits with no sol_deposit_authority and zero SOL deposit fee numerator are supported (valid0/0 or0/N); current facts remain unverified.',
  'Configured slippage is read from Config, not this manifest. Production computes max(config-derived floor, caller minimum); explicit caller minimum0 adds no stronger caller floor, NOT zero effective protection.',
  'Minted units must be positive and meet the protected minimum. Rounding, oversized minima, stale state or overflow can reject a syntactically valid draft; no quote/amount optimizer is provided.',
  'Protected poolCPI25 and exact postchecks must preserve historical book value without even1lamport loss and keep value>=HWM. Only two historical unit ledgers change; HWM, pending assets, rent, KIF/carry and full Token-native quarantine remain preserved.',
  'Principal SOL PDA signs internally using principal-sol+bump. PIV authority is externally nonsigner, System-owned/nonexecutable/empty. Final pool-withdraw PDA is canonical/nonexecutable and not exact Token-owned355-byte multisig; it need not be System-owned.',
  'Independently verify signer control/valid fee payer/funding and exact program/artifact/account state. Refresh blockhash/context before any future signed step. Resource requests are not measured sufficiency; this packet is never send-ready.',
];
const EXPECTED_EFFECT='Production converts requested historical principal SOL through protected pool DepositSolWithSlippage variant25 into principal JitoSOL. Pending contributions are untouched. Source principalSOL signs only inside PIV; this draft requests no governance or external custody signer. Actual minted output/effective minimum/book values remain unknown.';
const ERROR_GUIDE=[
  [6027,'ZeroPrincipalDeposit'],[6028,'PrincipalDepositExceedsQueue'],[6029,'UnsupportedPrincipalDepositFee'],[6030,'InvalidPrincipalDepositPool'],
  [6031,'PrincipalDepositObservationMismatch'],[6032,'PrincipalDepositMinimumNotMet'],[6033,'PrincipalDepositHistoricalValueLoss'],[6034,'InvalidLifecycle'],
  [6035,'PausedOperation'],[6046,'InvalidCustodyObservation'],[6064,'HighWaterMarkDecrease'],[6069,'InvalidSlippage'],[6072,'ArithmeticOverflow'],
].map(([code,name])=>({code,name}));
const sha=b=>createHash('sha256').update(b).digest('hex');
const fail=(code,field)=>{const e=new Error(code);e.code=code;e.field=field;throw e;};
const plain=v=>v!==null&&typeof v==='object'&&!Array.isArray(v)&&Object.getPrototypeOf(v)===Object.prototype;
function validateManifest(input){
  function walk(value,shape,field){
    if(value===null||value===undefined)return null;
    if(plain(shape)){
      if(!plain(value))fail('object_required',field||'manifest');
      if(Object.keys(value).some(k=>!Object.hasOwn(shape,k)))fail('unknown_field',field||'manifest');
      return Object.fromEntries(Object.entries(shape).map(([k,v])=>[k,walk(value[k],v,field?`${field}.${k}`:k)]));
    }return value;
  }
  if(!plain(input))fail('object_required','manifest');const m=walk(input,TEMPLATE,''),check=(v,f,fn)=>{if(v!==null&&v!==undefined)fn(v,f);};
  const key=(value,field)=>{if(typeof value!=='string')fail('public_key_string',field);let p;try{p=pk(value);}catch{fail('invalid_public_key',field);}
    if(p.toBase58()!==value||value===IDS.system)fail('noncanonical_or_zero_public_key',field);};
  const decimal=(value,field,positive=false)=>{if(typeof value!=='string'||!/^(?:0|[1-9]\d*)$/.test(value))fail('decimal_string_required',field);
    const n=BigInt(value);if(n>U64||positive&&n===0n)fail('integer_range',field);};
  check(m.schema,'schema',v=>{if(v!==TEMPLATE.schema)fail('unsupported_schema','schema');});
  check(m.cluster,'cluster',v=>{if(v!=='testnet')fail('unsupported_cluster','cluster');});
  for(const field of ['programId','feePayer'])check(m[field],field,key);
  check(m.amountLamports,'amountLamports',(v,f)=>decimal(v,f,true));check(m.callerMinimumPoolTokensOut,'callerMinimumPoolTokensOut',decimal);
  if(m.protocol)for(const [field,value]of Object.entries(m.protocol))check(value,`protocol.${field}`,key);
  if(m.transport){check(m.transport.recentBlockhash,'transport.recentBlockhash',key);for(const field of ['lastValidBlockHeight','referenceSlot'])check(m.transport[field],`transport.${field}`,decimal);}
  if(m.resources)for(const [field,limits]of Object.entries(RESOURCE_POLICY))if(field!=='meaning')check(m.resources[field],`resources.${field}`,(v,f)=>{
    if(!Number.isSafeInteger(v)||v<limits.min||v>limits.max||limits.multiple&&v%limits.multiple!==0)fail('unsupported_resource_request',f);
  });
  if(m.feePayer!==null&&!PublicKey.isOnCurve(pk(m.feePayer).toBytes()))fail('unsupported_off_curve_fee_signer','feePayer');
  if(m.programId!==null&&(Object.values(IDS).includes(m.programId)||m.programId===POOL_WITHDRAW_AUTHORITY))fail('unsupported_program_identity','programId');
  const derived=m.programId===null?null:intake.derived(m.programId);
  const protectedKeys=new Set([...Object.values(IDS),POOL_WITHDRAW_AUTHORITY,...(m.programId===null?[]:[m.programId,...Object.values(derived)])]);
  for(const [field,value]of [['feePayer',m.feePayer],...Object.entries(m.protocol||{}).map(([k,v])=>[`protocol.${k}`,v])])
    if(value!==null&&protectedKeys.has(value))fail('unsupported_protected_alias',field);
  const protocol=Object.entries(m.protocol||{}).filter(([,v])=>v!==null);
  for(let a=0;a<protocol.length;a++)for(let b=0;b<a;b++)if(protocol[a][1]===protocol[b][1]&&![protocol[a][0],protocol[b][0]].every(k=>['managerFee','referrer'].includes(k)))fail('protocol_alias','protocol');
  if(m.feePayer!==null&&protocol.some(([,v])=>v===m.feePayer))fail('fee_payer_protocol_alias','feePayer');
  const fields=['schema','cluster','programId','feePayer','amountLamports','callerMinimumPoolTokensOut',...['protocol','transport','resources'].flatMap(block=>Object.keys(TEMPLATE[block]).map(k=>`${block}.${k}`))];
  return {manifest:m,derived,missing:fields.filter(f=>f.split('.').reduce((v,k)=>v?.[k],m)==null)};
}
function budgets(resources){
  const loaded=Buffer.alloc(5);loaded[0]=4;loaded.writeUInt32LE(resources.loadedAccountBytes,1);
  const result=[ComputeBudgetProgram.setComputeUnitLimit({units:resources.computeUnits}),ComputeBudgetProgram.requestHeapFrame({bytes:resources.heapBytes}),new TransactionInstruction({programId:pk(IDS.computeBudget),keys:[],data:loaded})];
  assert.deepEqual(result.map(i=>i.data[0]),[2,1,4]);assert(result.every(i=>i.keys.length===0&&i.data.length===5));
  assert.deepEqual(result.map(i=>i.data.readUInt32LE(1)),[resources.computeUnits,resources.heapBytes,resources.loadedAccountBytes]);return result;
}
function nativeRoles(m,d){
  const roles=[],add=(role,address,writable=false)=>roles.push({role,address,signer:false,writable});
  for(const role of ['config','active_distribution','pending_sol','principal_sol','operational_sol','distribution_escrow','kif_sol','principal_token','pending_token'])add(role,d[role],['config','principal_sol','principal_token'].includes(role));
  add('piv_authority',d.piv_authority);add('mint',IDS.mint,true);add('system_program',IDS.system);add('token_program',IDS.token);add('pool_program',IDS.poolProgram);add('pool',IDS.pool,true);
  add('validator_list',m.protocol.validatorList);add('reserve',m.protocol.reserve,true);add('manager_fee',m.protocol.managerFee,true);
  if(m.protocol.managerFee!==m.protocol.referrer)add('referrer',m.protocol.referrer,true);
  add('pool_withdraw_authority',POOL_WITHDRAW_AUTHORITY);if(new Set(roles.map(r=>r.address)).size!==roles.length)fail('native_role_alias','manifest');return roles;
}
function planManifest(input){
  const {manifest:m,derived:d,missing}=validateManifest(input),common={schema:TEMPLATE.schema,sourceBaseline:BASELINE,action:ACTION,missing,readiness:{...OPEN},expectedEffect:EXPECTED_EFFECT,
    preconditions:PRECONDITIONS,resourcePolicy:RESOURCE_POLICY,errorGuide:{scope:'NONEXHAUSTIVE_SOURCE_GUIDANCE_ONLY; no live error observed or diagnosed',custom:ERROR_GUIDE,downstream:'System/Token/pool/runtime errors may propagate; no universal decoder or guessed remedy.'}};
  if(missing.length)return {...common,status:'INCOMPLETE',plan:null};
  const roles=nativeRoles(m,d),data=Buffer.alloc(25);data.write('PIV1SP01');data[8]=1;data.writeBigUInt64LE(BigInt(m.amountLamports),9);data.writeBigUInt64LE(BigInt(m.callerMinimumPoolTokensOut),17);
  const instruction=new TransactionInstruction({programId:pk(m.programId),keys:roles.map(r=>({pubkey:pk(r.address),isSigner:false,isWritable:r.writable})),data});
  const packet=initializer.packet({m},ACTION,[...budgets(m.resources),instruction]);assert.equal(packet.signatureCount,1);assert.deepEqual(packet.requiredSigners,[m.feePayer]);
  return {...common,status:'OFFLINE_PLAN_ONLY',declarationSha256:sha(Buffer.from(JSON.stringify(m))),plan:{program:m.programId,feePayer:m.feePayer,
    derived:{...d,pool_withdraw_authority:POOL_WITHDRAW_AUTHORITY},roles,native:{bytes:25,hex:data.toString('hex'),sha256:sha(data)},
    requested:{amountLamports:m.amountLamports,callerMinimumPoolTokensOut:m.callerMinimumPoolTokensOut},quotedMinted:null,effectiveProtectedMinimum:null,bookValueBefore:null,bookValueAfter:null,
    requestedResources:m.resources,referenceContext:{...m.transport,verified:false,usage:'ENCODING_INPUT_ONLY; refresh/revalidate before any future signed step'},packets:[packet]}};
}
function verifySources(){const existing=initializer.verifySources();for(const [file,expected]of Object.entries(SOURCE_PINS))assert.equal(sha(fs.readFileSync(path.join(REPO,file))),expected,`source binding: ${file}`);
  return {node:existing.node,web3:existing.web3,pins:SOURCE_PINS,initializerPins:existing.pins};}
function main(args){
  const sourceCheck=verifySources();if(args.length===1&&args[0]==='--template'){process.stdout.write(JSON.stringify(TEMPLATE,null,2)+'\n');return;}
  if(args.length!==2||args[0]!=='--manifest')fail('unsupported_arguments','arguments');
  const filename=path.resolve(args[1]),stat=fs.statSync(filename);if(!stat.isFile()||stat.size>65536)fail('public_manifest_file_required','manifest');
  const bytes=fs.readFileSync(filename);if(bytes.length>65536)fail('manifest_size','manifest');
  const report=planManifest(initializer.parseManifest(bytes.toString('utf8')));report.manifestInputSha256=sha(bytes);report.sourceCheck=sourceCheck;process.stdout.write(JSON.stringify(report,null,2)+'\n');
}
module.exports={BASELINE,SOURCE_PINS,TEMPLATE,IDS,RESOURCE_POLICY,POOL_WITHDRAW_AUTHORITY,ERROR_GUIDE,validateManifest,planManifest,verifySources};
if(require.main===module){try{main(process.argv.slice(2));}catch(e){process.stderr.write(JSON.stringify({status:'INVALID_OR_UNSUPPORTED_PRINCIPAL_STAKE_PLAN',code:e.code||'plan_validation_failed',field:e.field||'manifest'})+'\n');process.exitCode=1;}}
