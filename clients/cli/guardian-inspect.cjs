'use strict';
// Stored guardian facts only: no live authority, activity evaluation or claim plan.
const assert=require('node:assert/strict'),fs=require('node:fs'),path=require('node:path');
const {createHash}=require('node:crypto');
const initializer=require('./initializer-plan.cjs'),state=require('./state-inspect.cjs');
const {PublicKey}=initializer;
const SOURCE_PINS=Object.freeze({
  "programs/piv1/src/state/guardian.rs": "ce762257b8025789f8ec43351ce166f0a5f7ad0bb6f05b4c9515173d7f1cd619",
  "programs/piv1/src/state/config.rs": "9d7a57c87b6781b943e0e9c13d08f73a21058d400b40540c485e6c596b042198",
  "programs/piv1/src/guardian_clock_accounts.rs": "5cc36b79d429bb3e1818466d429259474c8a26d1c1288da16a71f22033f84946",
  "programs/piv1/src/kif_claim_accounts.rs": "9259221886dd4183b469d79d4860b17c1de7e274563ae07b65a532ecf8b08493",
  "programs/piv1/src/state/kif_claim.rs": "3e58b3554233774f8f244e2771ffb492b98e2073b802208c8c08704659af7014",
  "programs/piv1/src/accounts.rs": "08e3d28971160910088a40d74215d15a9c2d07b11142e28c4fb5162d725f1c80",
  "programs/piv1/src/state_persistence.rs": "d26483f7e1b95a020623b85705ab0006cb884e85cff37a0666bafa83123401a5",
  "programs/piv1/src/constants.rs": "2498c79c42d732be84aae423cf31d82997574cc08cca02fb226fea9acaa24f94",
  "clients/cli/state-inspect.cjs": "4f43a861ad7cdcc7b2d73116989c09146d36649aff71a3d98204aa93af938240",
  "clients/cli/initializer-plan.cjs": "347d5ab436b3e8878fd337c53ef8636721ca64a6465bc86c06f2637921ac6f6f",
  "clients/cli/initializer-plan.test.cjs": "7616bb36648e7d4fc835f80bbe27b0b67cd4a96a2a9f2540238b04e5acbbc1cb",
  "validation/genesis-transport/transport.cjs": "a7694ff07577988e96546a53008bc8ab85bbc0ae7cb9353dcfc25e6c5a3c3724",
  "validation/genesis-transport/production-initializer.cjs": "cdf326d32f764d55dc1bc30590a252cdffca069ad1efd2989aae36744c20553b"
});
const BASELINE='bb21213e5d17bc047443633840f1be0be54f9c49';
const ACCOUNT={address:null,owner:null,executable:null,lamports:null,dataBase64:null};
const TEMPLATE={schema:'piv1-guardian-inspection-v1',cluster:'testnet',programId:null,mode:null,
  context:{sourceKind:null,sourceLabel:null,slot:null},current:{config:{...ACCOUNT},registry:{...ACCOUNT},rewards:Array.from({length:6},()=>({...ACCOUNT}))},earned:{reward:{...ACCOUNT}}};
const IDS=initializer.IDS,REPO=path.resolve(__dirname,'../..'),U64=(1n<<64n)-1n;
const sha=b=>createHash('sha256').update(b).digest('hex'),present=v=>v!==null&&v!==undefined;
const plain=v=>v!==null&&typeof v==='object'&&!Array.isArray(v)&&Object.getPrototypeOf(v)===Object.prototype;
const fail=(code,field)=>{const e=new Error(code);e.code=code;e.field=field;throw e;};
const le64=v=>{const b=Buffer.alloc(8);b.writeBigUInt64LE(BigInt(v));return b;};
const pda=(program,seeds)=>{const [a,bump]=PublicKey.findProgramAddressSync(seeds,new PublicKey(program));return {address:a.toBase58(),bump};};
const rewardPda=(program,r)=>pda(program,[Buffer.from('guardian-reward'),new PublicKey(r.guardian).toBuffer(),le64(r.registry_revision),Buffer.from([r.guardian_index])]);
const OPEN=Object.freeze({publicReady:false,deploymentAuthorized:false,fullRustStateValidation:false,provenanceAuthenticated:false,snapshotCoherenceVerified:false,
  liveFreshnessVerified:false,currentAuthorityVerified:false,rentHealthVerified:false,guardianControlVerified:false,custodyVerified:false,
  eligibility:null,claimability:null,liabilityBacking:null,availablePayment:null,resourceSufficiency:null,fees:null,funding:null,m4Reservation:'OPEN_UNWAIVED'});
const LIMITS=[
  'Only supplied declared metadata and stored bytes are inspected. Provenance, coherent snapshot, live freshness, current authority/governance, rent, wallet control and resource sufficiency remain unverified. No RPC or chain observation occurs.',
  'Checks are explicitly bounded layout/PDA/tuple checks plus standalone reward ledger identity. Full Rust Config/registry/reward semantic validation and runtime authentication are not claimed.',
  'Claimable and cumulative amounts are stored ledgers, not available payment, backing, solvency or claim eligibility. Current six reward records do not enumerate historical liabilities and are never summed as a global liability.',
  'Stored last_active_period is displayed only. No Clock, period/time, activity bitmap, membership eligibility or activity outcome is derived. A current-set declaration does not prove current governance authority.',
  'Earned-record inspection is independent of current membership, registry, Clock, distribution and pool. Its expected-counter source is the stored cumulative_claimed field, not a claimability/replay-validity guarantee or generated command.',
  'No transaction, claim/heartbeat command, account creation, signing, sending, deployment, wallet/key access or economic choice is implemented. M4 criterion stays open/unwaived; M5 preparation only.',
];
function decode(bytes,role){
  const registry=role==='registry',size=registry?210:84,disc=registry?[72,14,254,2,76,233,97,92]:[169,109,89,17,75,171,105,39];
  if(!Buffer.isBuffer(bytes)||bytes.length!==size)fail('account_size',role);if(!bytes.subarray(0,8).equals(Buffer.from(disc)))fail('state_discriminator',role);
  let at=8;const take=n=>{if(at+n>bytes.length)fail('truncated_state_field',role);const b=bytes.subarray(at,at+n);at+=n;return b;},u8=()=>take(1)[0],u64=()=>take(8).readBigUInt64LE().toString(),key=()=>new PublicKey(take(32)).toBase58();
  const value={version:u8(),bump:u8()};if(value.version!==1)fail('unsupported_state_version',role);
  if(registry){value.revision=u64();value.guardian_keys=Array.from({length:6},key);if(value.guardian_keys.includes(IDS.system)||new Set(value.guardian_keys).size!==6)fail('invalid_guardian_set',role);}
  else{value.guardian_index=u8();value.registry_revision=u64();value.guardian=key();const tag=u8();if(tag>1)fail('invalid_option_tag',role);value.last_active_period=tag===0?null:u64();
    value.claimable_lamports=u64();value.cumulative_earned=u64();value.cumulative_claimed=u64();if(value.guardian_index>=6||value.guardian===IDS.system)fail('invalid_guardian_tuple',role);
    if(BigInt(value.cumulative_claimed)>BigInt(value.cumulative_earned)||BigInt(value.cumulative_earned)-BigInt(value.cumulative_claimed)!==BigInt(value.claimable_lamports))fail('reward_ledger_identity',role);}
  if(bytes.subarray(at).some(b=>b!==0))fail('nonzero_trailing_padding',role);
  return {value,layout:{allocationBytes:size,serializedPayloadBytes:at-8,trailingZeroPaddingBytes:size-at},sha256:sha(bytes)};
}
const decodeRegistry=b=>decode(b,'registry'),decodeReward=b=>decode(b,'reward');
function validateSnapshot(input){
  function walk(v,s,f){if(!present(v))return null;if(Array.isArray(s)){if(!Array.isArray(v)||v.length!==6)fail('six_slots_required',f);return v.map((x,i)=>walk(x,s[i],`${f}.${i}`));}
    if(plain(s)){if(!plain(v))fail('object_required',f||'snapshot');if(Object.keys(v).some(k=>!Object.hasOwn(s,k)))fail('unknown_field',f||'snapshot');return Object.fromEntries(Object.entries(s).map(([k,shape])=>[k,walk(v[k],shape,f?`${f}.${k}`:k)]));}return v;}
  if(!plain(input))fail('object_required','snapshot');const m=walk(input,TEMPLATE,''),check=(v,f,fn)=>{if(present(v))fn(v,f);},decoded={};
  const key=(v,f)=>{if(typeof v!=='string')fail('public_key_string',f);let k;try{k=new PublicKey(v);}catch{fail('invalid_public_key',f);}if(k.toBase58()!==v||v===IDS.system)fail('noncanonical_or_zero_public_key',f);};
  const decimal=(v,f)=>{if(typeof v!=='string'||!/^(?:0|[1-9]\d*)$/.test(v))fail('decimal_string_required',f);if(BigInt(v)>U64)fail('integer_range',f);};
  check(m.schema,'schema',v=>{if(v!==TEMPLATE.schema)fail('unsupported_schema','schema');});check(m.cluster,'cluster',v=>{if(v!=='testnet')fail('unsupported_cluster','cluster');});check(m.programId,'programId',key);
  if(present(m.programId)&&Object.values(IDS).includes(m.programId))fail('unsupported_program_identity','programId');check(m.mode,'mode',v=>{if(!['current-set','earned-record'].includes(v))fail('unsupported_mode','mode');});
  if(m.context){check(m.context.sourceKind,'context.sourceKind',v=>{if(!['provided-public-snapshot','retained-local-bank','synthetic-literal'].includes(v))fail('unsupported_source_kind','context.sourceKind');});check(m.context.sourceLabel,'context.sourceLabel',v=>{if(typeof v!=='string'||v.length<1||v.length>512)fail('source_label_string','context.sourceLabel');});check(m.context.slot,'context.slot',decimal);}
  const accounts=[['current.config',m.current?.config,'config'],['current.registry',m.current?.registry,'registry'],...(m.current?.rewards||[]).map((a,i)=>[`current.rewards.${i}`,a,'reward']),['earned.reward',m.earned?.reward,'reward']];
  for(const [field,a,role]of accounts)if(a){for(const f of ['address','owner'])check(a[f],`${field}.${f}`,key);check(a.executable,`${field}.executable`,v=>{if(v!==false)fail('executable_state_account',`${field}.executable`);});check(a.lamports,`${field}.lamports`,decimal);
    if(present(m.programId)&&present(a.owner)&&a.owner!==m.programId)fail('declared_owner_mismatch',`${field}.owner`);
    check(a.dataBase64,`${field}.dataBase64`,v=>{if(typeof v!=='string')fail('base64_string_required',field);const b=Buffer.from(v,'base64');if(b.toString('base64')!==v)fail('noncanonical_base64',field);decoded[field]=role==='config'?state.decodeConfig(b):decode(b,role);});}
  // Reuse the existing bounded Config identity/topology validator without requiring a distribution snapshot.
  const cv=state.validateSnapshot({...state.TEMPLATE,programId:m.programId,context:null,config:m.current?.config??null,distribution:null});
  const topology=cv.topology,c=decoded['current.config']?.value,g=decoded['current.registry']?.value;
  if(topology){const a=m.current?.registry;if(present(a?.address)&&a.address!==topology.guardian_registry.address)fail('canonical_registry_pda','current.registry.address');if(g&&g.bump!==topology.guardian_registry.bump)fail('canonical_registry_bump','current.registry');}
  if(c&&g&&(c.guardian_registry_revision!==g.revision||c.bumps.guardian_registry!==g.bump))fail('config_registry_tuple','current.registry');
  if(c&&present(m.current?.registry?.address)&&c.guardian_registry!==m.current.registry.address)fail('config_registry_reference','current.registry.address');
  for(const [field,a,role]of accounts)if(role==='reward'&&decoded[field]){const r=decoded[field].value;if(present(m.programId)){const expected=rewardPda(m.programId,r);if(present(a?.address)&&a.address!==expected.address)fail('canonical_reward_pda',`${field}.address`);if(r.bump!==expected.bump)fail('canonical_reward_bump',field);}}
  const currentAccounts=accounts.filter(([f])=>f.startsWith('current.'));
  const addresses=currentAccounts.map(([,a])=>a?.address).filter(present);if(new Set(addresses).size!==addresses.length)fail('current_account_alias','current');
  const piv=new Set(topology?Object.values(topology).map(v=>v.address):[]);
  if(g&&g.guardian_keys.some(k=>piv.has(k)||addresses.includes(k)))fail('current_guardian_internal_alias','current.registry');
  for(let i=0;i<6;i++){const r=decoded[`current.rewards.${i}`]?.value;if(r&&g&&(r.guardian_index!==i||r.registry_revision!==g.revision||r.guardian!==g.guardian_keys[i]))fail('current_reward_tuple',`current.rewards.${i}`);
    if(r&&piv.has(m.current?.rewards?.[i]?.address))fail('current_reward_internal_alias',`current.rewards.${i}`);}
  const populated=v=>Array.isArray(v)?v.some(populated):plain(v)?Object.values(v).some(populated):present(v);
  if(present(m.mode)&&populated(m[m.mode==='current-set'?'earned':'current']))fail('unused_mode_input',m.mode==='current-set'?'earned':'current');
  const fields=['schema','cluster','programId','mode',...Object.keys(TEMPLATE.context).map(f=>`context.${f}`)];
  const requireAccount=f=>fields.push(...Object.keys(ACCOUNT).map(k=>`${f}.${k}`));if(m.mode==='current-set'){requireAccount('current.config');requireAccount('current.registry');for(let i=0;i<6;i++)requireAccount(`current.rewards.${i}`);}else if(m.mode==='earned-record')requireAccount('earned.reward');
  return {snapshot:m,decoded,topology,missing:fields.filter(f=>f.split('.').reduce((v,k)=>v?.[k],m)==null)};
}
function inspectSnapshot(input){const v=validateSnapshot(input),m=v.snapshot,common={schema:TEMPLATE.schema,sourceBaseline:BASELINE,mode:m.mode,missing:v.missing,readiness:{...OPEN},limitations:LIMITS};if(v.missing.length)return {...common,status:'INCOMPLETE',inspection:null};
  const account=(field,a)=>{const {dataBase64,...metadata}=a,d=v.decoded[field];return {...metadata,layout:d.layout,sha256:d.sha256};};
  const rewardSummary=r=>({guardian:r.guardian,registryRevision:r.registry_revision,guardianIndex:r.guardian_index,lastActivePeriod:r.last_active_period,storedClaimableLamports:r.claimable_lamports,cumulativeEarned:r.cumulative_earned,cumulativeClaimed:r.cumulative_claimed,expectedCumulativeClaimedSource:{field:'cumulative_claimed',value:r.cumulative_claimed,meaning:'STORED_FIELD_ONLY; refresh exact earned record; no replay-validity or claimability proof'}});
  let decoded,accounts,summary;if(m.mode==='current-set'){const c=m.current,rewards=c.rewards.map((a,i)=>v.decoded[`current.rewards.${i}`].value);decoded={config:v.decoded['current.config'].value,registry:v.decoded['current.registry'].value,rewards};accounts={config:account('current.config',c.config),registry:account('current.registry',c.registry),rewards:c.rewards.map((a,i)=>account(`current.rewards.${i}`,a))};summary={kind:'DECLARED_CURRENT_SET',registryRevision:decoded.registry.revision,guardianKeys:decoded.registry.guardian_keys,rewards:rewards.map(rewardSummary)};}
  else{const r=v.decoded['earned.reward'].value;decoded={reward:r};accounts={reward:account('earned.reward',m.earned.reward)};summary={kind:'INDEPENDENT_EARNED_RECORD',...rewardSummary(r)};}
  return {...common,status:'OFFLINE_GUARDIAN_INSPECTION',inspection:{context:{...m.context,authenticated:false},program:m.programId,decoded,accounts,summary,structuralChecksPassed:['Declared program owner/nonexecutable metadata and canonical account address/bump',m.mode==='current-set'?'Exact Registry210 and six Reward84 allocations, discriminators/version1 and strict variable Option tags/zero tail':'Exact Reward84 allocation, discriminator/version1 and strict variable Option tag/zero tail','Standalone reward claimed<=earned and claimable==earned-claimed (bounded ledger identity only)',...(m.mode==='current-set'?['Existing bounded Config decoder/topology checks; registry address/revision/bump and six exact ordered current reward tuples']:['Earned self tuple only; no current membership or activity requirement'])]}};
}
function verifySources(){const existing=state.verifySources();for(const [file,expected]of Object.entries(SOURCE_PINS))assert.equal(sha(fs.readFileSync(path.join(REPO,file))),expected,`source binding: ${file}`);return {node:existing.node,web3:existing.web3,pins:SOURCE_PINS,statePins:existing.pins,initializerPins:existing.initializerPins};}
function main(args){const sourceCheck=verifySources();if(args.length===1&&args[0]==='--template'){process.stdout.write(JSON.stringify(TEMPLATE,null,2)+'\n');return;}
  if(args.length!==2||args[0]!=='--snapshot')fail('unsupported_arguments','arguments');const filename=path.resolve(args[1]),stat=fs.statSync(filename);if(!stat.isFile()||stat.size>65536)fail('public_snapshot_file_required','snapshot');const bytes=fs.readFileSync(filename);if(bytes.length>65536)fail('snapshot_size','snapshot');const out=inspectSnapshot(initializer.parseManifest(bytes.toString('utf8')));out.snapshotInputSha256=sha(bytes);out.sourceCheck=sourceCheck;process.stdout.write(JSON.stringify(out,null,2)+'\n');}
module.exports={BASELINE,SOURCE_PINS,TEMPLATE,decodeRegistry,decodeReward,validateSnapshot,inspectSnapshot,verifySources};
if(require.main===module){try{main(process.argv.slice(2));}catch(e){process.stderr.write(JSON.stringify({status:'INVALID_OR_UNSUPPORTED_GUARDIAN_SNAPSHOT',code:e.code||'inspection_failed',field:e.field||'snapshot'})+'\n');process.exitCode=1;}}
