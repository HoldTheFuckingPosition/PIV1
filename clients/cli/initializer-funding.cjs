'use strict';
// Conditional rent arithmetic over explicit public inputs. No RPC or wallet APIs.
const assert=require('node:assert/strict');
const fs=require('node:fs');
const path=require('node:path');
const {createHash}=require('node:crypto');
const initializer=require('./initializer-plan.cjs');
const SOURCE_PINS=Object.freeze({
  "clients/cli/initializer-plan.cjs": "347d5ab436b3e8878fd337c53ef8636721ca64a6465bc86c06f2637921ac6f6f",
  "programs/piv1/src/genesis_initialization.rs": "4fcc60c7592125da1bda2a549f94933f26eff5f960be1ea5fb98f51350ed879b"
});
const BASELINE='9418d9c9b7eb8e3da10dd8b7a07dff53e717bd36';
const REPO=path.resolve(__dirname,'../..'),U64=(1n<<64n)-1n,MAX_BYTES=65536;
const SIZES=[0,56,84,102,165,210,568,646,891,1014,1548,1559,1581,1592];
const TARGET_SIZES=[1014,891,210,84,84,84,84,84,84,0,0,0,0,0,165,165];
const TARGET_ROLES=['config','distribution','guardian_registry',...Array.from({length:6},(_,i)=>`guardian_reward_${i}`),
  'pending_sol','principal_sol','operational_sol','distribution_escrow','kif_sol','principal_token','pending_token'];
const TEMPLATE={status:'READ_ONLY_RENT_OBSERVATIONS',endpoint:'https://api.testnet.solana.com',
  genesisHash:'4uhcVJyU9pJkvQyS88uRDiswHXSCkY3zQawwpjk2NsNY',commitment:'finalized',observedAtUtc:null,
  slotBefore:null,slotAfter:null,rentLamportsByBytes:Object.fromEntries(SIZES.map(n=>[n,null])),
  logicalRequests:null,httpRequests:null,limitations:null};
const OPEN=Object.freeze({publicReady:false,deploymentAuthorized:false,actualFundingQuote:null,actualFees:null,
  priorityFees:null,actualShortfall:null,availableWalletBalances:null,peakLiquidity:null,actualRequiredWalletBalance:null,
  recipientFunding:null,governanceProvisioning:null,deploymentCosts:null,operationalLiquidity:null,
  signerControlVerified:false,accountSnapshotVerified:false,observationAuthenticated:false,m4Reservation:'OPEN_UNWAIVED'});
const sha=b=>createHash('sha256').update(b).digest('hex');
const fail=(code,field)=>{const e=new Error(code);e.code=code;e.field=field;throw e;};
const plain=v=>v!==null&&typeof v==='object'&&!Array.isArray(v)&&Object.getPrototypeOf(v)===Object.prototype;
const decimal=(value,field)=>{
  if(typeof value!=='string'||!/^(?:0|[1-9]\d*)$/.test(value))fail('decimal_string_required',field);
  if(BigInt(value)>U64)fail('integer_range',field);
};
function validateObservations(input) {
  if(!plain(input))fail('object_required','observations');
  const missing=[],out={};
  if(Object.keys(input).some(k=>!Object.hasOwn(TEMPLATE,k)))fail('unknown_field','observations');
  for(const field of Object.keys(TEMPLATE)) {
    const value=input[field];out[field]=value??null;
    if(value===null||value===undefined){missing.push(`observations.${field}`);continue;}
    if(['status','endpoint','genesisHash','commitment'].includes(field)) {
      if(value!==TEMPLATE[field])fail('unsupported_observation_context',`observations.${field}`);
    } else if(['slotBefore','slotAfter'].includes(field))decimal(value,`observations.${field}`);
    else if(field==='observedAtUtc') {
      if(typeof value!=='string'||!/^\d{4}-\d\d-\d\dT\d\d:\d\d:\d\d(?:\.\d{1,9})?(?:Z|\+00:00)$/.test(value)||
        !Number.isFinite(Date.parse(value))||new Date(value).toISOString().slice(0,19)!==value.slice(0,19))fail('utc_timestamp_required',`observations.${field}`);
    } else if(field==='logicalRequests'||field==='httpRequests') {
      if(!Number.isSafeInteger(value)||value<1||value>10000)fail('request_count_range',`observations.${field}`);
    } else if(field==='limitations') {
      if(typeof value!=='string'||value.length<1||value.length>4096)fail('limitations_string_required',`observations.${field}`);
    } else {
      if(!plain(value))fail('object_required','observations.rentLamportsByBytes');
      if(Object.keys(value).some(k=>!SIZES.some(n=>String(n)===k)))fail('unknown_rent_size','observations.rentLamportsByBytes');
      out[field]={};let previous=null;
      for(const size of SIZES) {
        const amount=value[size];out[field][size]=amount??null;
        if(amount===null||amount===undefined){missing.push(`observations.rentLamportsByBytes.${size}`);continue;}
        decimal(amount,`observations.rentLamportsByBytes.${size}`);
        const n=BigInt(amount);if(n===0n||previous!==null&&n<previous)fail('unsupported_rent_floor',`observations.rentLamportsByBytes.${size}`);previous=n;
      }
    }
  }
  if(out.slotBefore!==null&&out.slotAfter!==null&&BigInt(out.slotBefore)>BigInt(out.slotAfter))fail('slot_bracket_order','observations.slotAfter');
  if(out.httpRequests!==null&&out.logicalRequests!==null&&out.httpRequests>out.logicalRequests)fail('request_count_order','observations.httpRequests');
  return {observations:out,missing};
}
const assumptions=[
  'CONDITIONAL ZERO-PREFUND SCENARIO: all16 PIV targets are virgin valid System-owned empty accounts with zero lamports.',
  'ALT, buffer, stored transaction and proposal are new supported addresses, with no prior allocation, prefunds or donations.',
  'Actual target shortfall is sum(max(floor(size)-valid ORIGINAL observed prefund,0)); balances/owner/data/freshness remain OPEN.',
  'Token-target excess is transferred internally to PendingSol, not refunded to payer, and cannot replace PendingSol ORIGINAL shortfall.',
  'Both recipient vaults already meet supported identity/owner/empty-data/funding guards; these packets do not provision them.',
  'The creator is a declared guardian, distinct from rentPayer under existing planner guards. No creator-to-rentPayer transfer is planned.',
];
const limitations=[
  'Supplied endpoint/genesis/rent metadata and file digest are provenance declarations, not authenticated observations or an RPC performed by this command.',
  'Rent responses have no per-result contextual slot. Slot brackets do not prove a coherent account snapshot, current prices or freshness.',
  'Gross payer outflow, creator refund, retained account balances and required residual are distinct quantities, never a complete funding budget.',
  'Stored transaction, proposal, ALT and PIV balances remain held through this plan; they are not burned or automatically reclaimed at execute.',
  'Actual buffer close refunds its ENTIRE balance to creator, including any donations. The modeled refund is only its original rent.',
  'Fees/priority, current wallet balances, account validity, prefunds, provisioning, retries and operational liquidity remain OPEN.',
  'Production deployment/artifact/ProgramData costs are separate OPEN; no loader recipe, size-based complete quote or artifact selection is inferred.',
  'Existing blockhash refresh, current governance, permissions/timelock, ALT owner/content/eligibility/warm-up and signer control checks remain OPEN.',
];
function scenario(variant,prices,manifest,plan) {
  const shared=variant==='shared34',R=size=>BigInt(prices[size]),bufferSize=shared?1559:1592,storedSize=shared?1548:1581;
  const roleAddress={rentPayer:plan?manifest.rentPayer:null,creator:plan?manifest.governance.creator:null,feePayer:plan?manifest.feePayer:null,
    alt:plan?.alt.address??null,buffer:plan?.buffer??null,stored_transaction:plan?.transaction??null,proposal:plan?.proposal??null};
  TARGET_ROLES.forEach((role,i)=>roleAddress[role]=plan?.native.roles[i+7].address??null);
  const wallets=new Map(),accountBalances=new Map();let ordinal=0;
  const walletKey=role=>roleAddress[role]??`UNSELECTED:${role}`;
  for(const role of ['rentPayer','creator','feePayer']) {
    const key=walletKey(role),old=wallets.get(key);
    if(old)old.roles.push(role);else wallets.set(key,{address:roleAddress[role],roles:[role],debits:0n,credits:0n,net:0n});
  }
  const apply=(role,amount)=>{const w=wallets.get(walletKey(role));w.net+=amount;if(amount<0n)w.debits-=amount;else w.credits+=amount;};
  const names=plan?plan.packets.map(p=>p.name):['alt_create','alt_extend','buffer_create','buffer_extend_*','transaction_from_buffer','proposal_create','approve_1','approve_2','approve_3','approve_4','execute'];
  const phases=names.map(name=>{
    const events=[];
    const transfer=(from,to,amount,purpose)=>{
      events.push({ordinal:++ordinal,fromRole:from,fromAddress:roleAddress[from]??null,toRole:to,toAddress:roleAddress[to]??null,lamports:amount.toString(),purpose});
      if(['rentPayer','creator'].includes(from))apply(from,-amount);else accountBalances.set(from,(accountBalances.get(from)||0n)-amount);
      if(['rentPayer','creator'].includes(to))apply(to,amount);else accountBalances.set(to,(accountBalances.get(to)||0n)+amount);
    };
    const pay=(to,amount,purpose)=>transfer('rentPayer',to,amount,purpose);
    if(name==='alt_create')pay('alt',R(56),'Create56-byte ALT');
    else if(name==='alt_extend')pay('alt',R(568)-R(56),'Only ALT growth56->568 delta');
    else if(name==='buffer_create')pay('buffer',R(bufferSize),'Entire final buffer allocation');
    else if(name==='transaction_from_buffer') {
      pay('stored_transaction',R(102),'Create empty102-byte stored transaction BEFORE buffer close');
      pay('stored_transaction',R(storedSize)-R(102),'Only stored transaction growth102->final delta BEFORE buffer close');
      transfer('buffer','creator',R(bufferSize),'Buffer closes AFTER stored transaction funding; refund goes to creator');
    } else if(name==='proposal_create')pay('proposal',R(646),'Create proposal');
    else if(name==='execute')TARGET_ROLES.forEach((role,i)=>pay(role,R(TARGET_SIZES[i]),'Conditional zero-prefund PIV target allocation'));
    else if(!name.startsWith('buffer_extend_')&&!/^approve_[1-4]$/.test(name))fail('unsupported_planner_phase','plan.packets');
    const packet=plan?.packets.find(p=>p.name===name);
    return {name,signatureCount:packet?.signatureCount??null,requiredSigners:packet?.requiredSigners??null,actualFee:null,events,
      cumulativeRentOnlyWalletChanges:[...wallets.values()].map(w=>({address:w.address,roles:w.roles,lamports:w.net.toString()}))};
  });
  const held=[...accountBalances].filter(([,balance])=>balance!==0n).map(([role,balance])=>({role,address:roleAddress[role]??null,
    bytes:role==='alt'?568:role==='stored_transaction'?storedSize:role==='proposal'?646:TARGET_SIZES[TARGET_ROLES.indexOf(role)],lamports:balance.toString()}));
  const targetFloors=TARGET_SIZES.reduce((a,size)=>a+R(size),0n),gross=R(568)+R(bufferSize)+R(storedSize)+R(646)+targetFloors;
  const retained=held.reduce((a,x)=>a+BigInt(x.lamports),0n);
  assert.equal(retained,gross-R(bufferSize));assert.equal(accountBalances.get('buffer'),0n);
  assert.equal([...wallets.values()].reduce((a,w)=>a+w.net,0n)+retained,0n);
  return {variant,nativeRoleCount:shared?34:35,scenario:'ZERO_PREFUND_ASSUMPTION_ONLY',
    accountSizes:{pivTargets:TARGET_SIZES,buffer:bufferSize,storedInitial:102,storedFinal:storedSize,proposal:646,altInitial:56,altFinal:568},
    phases,wallets:[...wallets.values()].map(w=>({address:w.address,roles:w.roles,rentOnlyDebits:w.debits.toString(),rentOnlyCredits:w.credits.toString(),rentOnlyNetChange:w.net.toString(),actualFees:null})),
    totals:{pivTargetFloors:targetFloors.toString(),grossRentPayerOutflow:gross.toString(),creatorBufferRefund:R(bufferSize).toString(),finalHeldAccountIncrease:retained.toString(),
      requiredRentPayerResidual:R(0).toString(),actualFundingQuote:null,actualShortfall:null,peakLiquidity:null,actualRequiredWalletBalance:null},
    heldAccounts:held,temporaryBufferFinalBalance:'0',
    prerequisites:{rentPayer:{address:roleAddress.rentPayer,postInitializerResidual:R(0).toString(),spent:false},
      recipients:['htfp_recipient','team_owner_recipient'].map((role,i)=>({role,address:plan?.recipients[i]??null,
        minimumExistingBalance:R(0).toString(),actualBalance:null,actualShortfall:null,provisionedByThesePackets:false})),governanceProvisioning:null,deploymentCosts:null}};
}
function fundingReport(manifestInput,observationInput) {
  const declared=initializer.validateManifest(manifestInput),validated=validateObservations(observationInput);
  const planned=declared.missing.length?null:initializer.planManifest(manifestInput).plan;
  const common={schema:'piv1-initializer-funding-v1',sourceBaseline:BASELINE,manifestStatus:planned?'OFFLINE_PLAN_ONLY':'INCOMPLETE',
    missingManifest:declared.missing,missingObservations:validated.missing,readiness:{...OPEN},
    observationContext:validated.observations,observationInputSha256:null,observationAuthentication:'NOT_PERFORMED',
    assumptions,limitations,units:'LAMPORTS_DECIMAL_STRINGS',fees:'OPEN; no per-signature or prioritization estimate'};
  if(validated.missing.length)return {...common,status:'INCOMPLETE',scenarios:null};
  const variants=planned?[planned.native.roles.length===34?'shared34':'distinct35']:['shared34','distinct35'];
  return {...common,status:'CONDITIONAL_RENT_ONLY',scenarios:variants.map(v=>scenario(v,validated.observations.rentLamportsByBytes,declared.manifest,planned))};
}
function verifySources() {
  const previous=initializer.verifySources();
  for(const [file,expected]of Object.entries(SOURCE_PINS))assert.equal(sha(fs.readFileSync(path.join(REPO,file))),expected,`source binding: ${file}`);
  return {node:previous.node,web3:previous.web3,pins:SOURCE_PINS,initializerPins:previous.pins};
}
function readPublicFile(filename) {
  const absolute=path.resolve(filename),stat=fs.statSync(absolute);
  if(!stat.isFile()||stat.size>MAX_BYTES)fail('public_json_file_required','input');
  const bytes=fs.readFileSync(absolute);if(bytes.length>MAX_BYTES)fail('input_size','input');
  return {parsed:initializer.parseManifest(bytes.toString('utf8')),sha256:sha(bytes)};
}
function main(args) {
  const sourceCheck=verifySources();
  if(args.length===1&&args[0]==='--template'){process.stdout.write(JSON.stringify(TEMPLATE,null,2)+'\n');return;}
  if(args.length!==4||args[0]!=='--manifest'||args[2]!=='--observations')fail('unsupported_arguments','arguments');
  const manifest=readPublicFile(args[1]),observations=readPublicFile(args[3]),result=fundingReport(manifest.parsed,observations.parsed);
  result.observationInputSha256=observations.sha256;result.manifestInputSha256=manifest.sha256;result.sourceCheck=sourceCheck;
  process.stdout.write(JSON.stringify(result,null,2)+'\n');
}
module.exports={BASELINE,SOURCE_PINS,SIZES,TARGET_SIZES,TEMPLATE,validateObservations,fundingReport,verifySources};
if(require.main===module){try{main(process.argv.slice(2));}catch(e){process.stderr.write(JSON.stringify({status:'INVALID_OR_UNSUPPORTED_FUNDING_REPORT',code:e.code||'funding_validation_failed',field:e.field||'input'})+'\n');process.exitCode=1;}}
