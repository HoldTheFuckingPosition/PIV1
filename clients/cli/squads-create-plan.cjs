'use strict';
// Public declarations -> one unsigned legacy packet. No wallet/RPC/sign/send path.
const assert=require('node:assert/strict'),fs=require('node:fs'),path=require('node:path');
const {createHash}=require('node:crypto');
const initializer=require('./initializer-plan.cjs'),t=require('../../validation/genesis-transport/transport.cjs');
const {PublicKey,TransactionInstruction}=require(t.SDK);
const BASELINE='0b1a19e0b9878b01949733e083137d9459de04e5';
const GENESIS='4uhcVJyU9pJkvQyS88uRDiswHXSCkY3zQawwpjk2NsNY',IDS=initializer.IDS;
const SOURCE_PINS=Object.freeze({
  'clients/cli/initializer-plan.cjs':'347d5ab436b3e8878fd337c53ef8636721ca64a6465bc86c06f2637921ac6f6f',
  'clients/cli/initializer-plan.test.cjs':'7616bb36648e7d4fc835f80bbe27b0b67cd4a96a2a9f2540238b04e5acbbc1cb',
  'validation/genesis-transport/transport.cjs':'a7694ff07577988e96546a53008bc8ab85bbc0ae7cb9353dcfc25e6c5a3c3724',
  'validation/genesis-transport/production-initializer.cjs':'cdf326d32f764d55dc1bc30590a252cdffca069ad1efd2989aae36744c20553b'
});
const RETAINED_SOURCE={commit:'64af7330413d5c85cbbccfd8c27a05d45b6e666f',root:'/tmp/piv1-squads-readonly-20260914-ng1dg59p',
  scope:'RETAINED_INTERFACE_REFERENCE_ONLY; not a deployed-source equivalence, build attestation or creation runtime proof; external files are not required by this CLI',
  pins:{'programs/squads_multisig_program/src/instructions/multisig_create.rs':'fc7d0f9eb2d5ec895717e5092f06c228533c174f776ad02d2ccb1861086c7197',
    'programs/squads_multisig_program/src/state/multisig.rs':'4ad0764e559a0376ad39c9e4d84a106dd0ec8fc040cdf0bf223c55763f07c9a0',
    'programs/squads_multisig_program/src/state/seeds.rs':'f062ac588cce48f7474775db6ee9d92e2e29e9931dfc6c7444530a0d908116a1',
    'sdk/multisig/idl/squads_multisig_program.json':'cb9a0a29040ec3853a5105c547ffc608bb0e3175d78dacf30d463aff939f9b9b'}};
const TEMPLATE={schema:'piv1-squads-create-plan-v1',cluster:'testnet',expectedGenesis:GENESIS,guardians:Array(6).fill(null),createKey:null,payer:null,
  profile:{threshold:null,memberPermissionsMask:null,configAuthority:null,timeLockSeconds:null,rentCollector:null,memo:null},
  observations:{programConfig:null,treasury:null,creationFeeLamports:null,rent330Lamports:null,networkFeeLamports:null,slot:null,observedAtUtc:null},
  transport:{recentBlockhash:null,lastValidBlockHeight:null,referenceSlot:null}};
const U64=(1n<<64n)-1n,REPO=path.resolve(__dirname,'../..'),pk=v=>new PublicKey(v),sha=b=>createHash('sha256').update(b).digest('hex');
const present=v=>v!==null&&v!==undefined,plain=v=>v!==null&&typeof v==='object'&&!Array.isArray(v)&&Object.getPrototypeOf(v)===Object.prototype;
const fail=(code,field)=>{const e=new Error(code);e.code=code;e.field=field;throw e;};
const pda=seeds=>{const [key,bump]=PublicKey.findProgramAddressSync(seeds,pk(IDS.squads));return {address:key.toBase58(),bump};};
const programConfig=()=>pda([Buffer.from('multisig'),Buffer.from('program_config')]);
const multisig=createKey=>pda([Buffer.from('multisig'),Buffer.from('multisig'),pk(createKey).toBuffer()]);
const READINESS={publicReady:false,deploymentAuthorized:false,squadsCreationAuthorized:false,profileAccepted:false,signerControlVerified:false,
  currentProgramVerified:false,currentProgramConfigVerified:false,treasuryValidityVerified:false,multisigAbsenceVerified:false,observationsFreshnessVerified:false,
  fundingSufficiencyVerified:false,resourceSufficiencyVerified:false,simulationVerified:false,networkFeeVerified:false,m4Reservation:'OPEN_UNWAIVED'};
const LIMITATIONS=[
  'M5 preparation only. Accepted six-guardian/4-of-6 intent is unchanged; the exact autonomous/timeLock0/rentCollectorNone creation profile remains PROPOSED, NOT AUTHORIZED. D-041 key/funding authority is consumed and does not authorize this transaction, a refill or PIV deployment.',
  'Only public declarations are encoded. This tool does not acquire live snapshots, read wallets/private keys, create keys, sign, submit or confirm. Actual signer control, balances, fees, account state, resources and success remain unverified.',
  'Explicit createKey must be one of the six guardians and signs alongside the distinct external payer. Creator, rent payer and network fee payer are this same external payer. Only these direct on-curve signer roles are supported. Other member declarations have no blanket curve restriction; input order is not PIV slot assignment.',
  'Members are encoded in ascending raw 32-byte key order with mask7 (Initiate/Vote/Execute). The source also sorts stored members. This is six logical signers, not a claim of independent custody, existing membership or four approvals. No proposal/vault signer is used for creation.',
  'Refresh official Testnet genesis, deployed Program/ProgramData, ProgramConfig owner/discriminator/current treasury and creation fee, all actor ownership/control, candidate multisig absence and 330-byte rent before any separately authorized signed phase. A derived PDA proves no existence/absence, virginity or validity.',
  'Declared fee/rent/network fee are unverified observations, not live quotes or a wallet budget. Checked totals assume no prefund/excess and exclude later provisioning, ALT, PIV deployment, fees, operational rent, capital and reserves. Stop on changed identities, profile, artifact, fees, funding or authority envelope and obtain revised exact-package coverage.',
  'Treasury is an unchecked writable account whose key must match ProgramConfig. Retained source skips transfer only when creation fee is zero; zero fee or an observed absent treasury does not prove executable creation. Root must separately verify/simulate exact current composition before authorization.',
  'One bare legacy instruction uses no ALT and no compute-budget or priority-fee instruction. Default runtime limits apply; resource sufficiency is unknown. Both 64-byte all-zero signature slots are counted in the1232-byte cap. Blockhash/lastValidBlockHeight/referenceSlot are declarations only; refresh the validity window before signing.',
  'Autonomous configAuthority=None is encoded by explicit string none; timeLockSeconds=0 is a proposal. rentCollector=None disables rent reclamation, memo=None writes no memo. The tool supports only this explicit profile and does not choose or accept it for the founder. No PIV ID, PIV slot/vault/pause/KIF-anchor choice follows.',
];
function validateManifest(input){
  function walk(v,s,f){if(!present(v))return null;if(Array.isArray(s)){if(!Array.isArray(v)||v.length!==6)fail('six_guardians_required',f);return v.map((x,i)=>walk(x,null,`${f}.${i}`));}
    if(plain(s)){if(!plain(v))fail('object_required',f||'manifest');if(Object.keys(v).some(k=>!Object.hasOwn(s,k)))fail('unknown_field',f||'manifest');return Object.fromEntries(Object.entries(s).map(([k,shape])=>[k,walk(v[k],shape,f?`${f}.${k}`:k)]));}return v;}
  if(!plain(input))fail('object_required','manifest');const m=walk(input,TEMPLATE,''),check=(v,f,fn)=>{if(present(v))fn(v,f);};
  const key=(v,f,hash=false)=>{if(typeof v!=='string')fail('public_key_string',f);let k;try{k=pk(v);}catch{fail('invalid_public_key',f);}if(k.toBase58()!==v||!hash&&v===IDS.system)fail('noncanonical_or_zero_public_key',f);};
  const decimal=(v,f,positive=false)=>{if(typeof v!=='string'||v.length>20||!/^(?:0|[1-9]\d*)$/.test(v))fail('decimal_u64_string_required',f);if(BigInt(v)>U64||positive&&BigInt(v)===0n)fail('integer_range',f);};
  for(const [f,want]of [['schema',TEMPLATE.schema],['cluster','testnet'],['expectedGenesis',GENESIS]])check(m[f],f,v=>{if(v!==want)fail('unsupported_'+f,f);});
  m.guardians?.forEach((g,i)=>check(g,`guardians.${i}`,key));for(const f of ['createKey','payer'])check(m[f],f,key);
  const supported={threshold:4,memberPermissionsMask:7,configAuthority:'none',timeLockSeconds:0,rentCollector:'none',memo:'none'};
  if(m.profile)for(const [f,want]of Object.entries(supported))check(m.profile[f],`profile.${f}`,v=>{if(v!==want)fail('unsupported_proposed_profile',`profile.${f}`);});
  if(m.observations){const o=m.observations;for(const f of ['programConfig','treasury'])check(o[f],`observations.${f}`,key);
    for(const f of ['creationFeeLamports','rent330Lamports','networkFeeLamports','slot'])check(o[f],`observations.${f}`,(v,field)=>decimal(v,field,['rent330Lamports','networkFeeLamports'].includes(f)));
    check(o.observedAtUtc,'observations.observedAtUtc',(v,f)=>{if(typeof v!=='string'||!/^\d{4}-\d\d-\d\dT\d\d:\d\d:\d\d\.\d{3}Z$/.test(v)||!Number.isFinite(Date.parse(v))||new Date(v).toISOString()!==v)fail('canonical_utc_milliseconds_required',f);});}
  if(m.transport){check(m.transport.recentBlockhash,'transport.recentBlockhash',(v,f)=>key(v,f,true));for(const f of ['lastValidBlockHeight','referenceSlot'])check(m.transport[f],`transport.${f}`,decimal);}
  const guardians=(m.guardians||[]).filter(present),config=programConfig(),candidate=present(m.createKey)?multisig(m.createKey):null;
  if(new Set(guardians).size!==guardians.length)fail('duplicate_guardian','guardians');
  if(present(m.createKey)&&guardians.length===6&&!guardians.includes(m.createKey))fail('create_key_not_member','createKey');
  if(present(m.payer)&&(guardians.includes(m.payer)||m.createKey===m.payer))fail('payer_member_or_create_key_alias','payer');
  if(present(m.observations?.programConfig)&&m.observations.programConfig!==config.address)fail('noncanonical_program_config','observations.programConfig');
  const reserved=new Set([...Object.values(IDS),config.address,...(candidate?[candidate.address]:[])]);
  for(const [f,v]of [...(m.guardians||[]).map((v,i)=>[`guardians.${i}`,v]),['createKey',m.createKey],['payer',m.payer],['observations.treasury',m.observations?.treasury]])if(present(v)&&reserved.has(v))fail('reserved_role_alias',f);
  if(present(m.observations?.treasury)&&[...guardians,m.createKey,m.payer].includes(m.observations.treasury))fail('treasury_actor_alias','observations.treasury');
  for(const f of ['createKey','payer'])if(present(m[f])&&!PublicKey.isOnCurve(pk(m[f]).toBytes()))fail('unsupported_off_curve_direct_signer',f);
  const costFields=['creationFeeLamports','rent330Lamports','networkFeeLamports'],knownCosts=costFields.map(f=>m.observations?.[f]).filter(present);
  if(knownCosts.reduce((s,v)=>s+BigInt(v),0n)>U64)fail('declared_cost_total_overflow','observations');
  const fields=['schema','cluster','expectedGenesis','createKey','payer',...Array.from({length:6},(_,i)=>`guardians.${i}`),...['profile','observations','transport'].flatMap(b=>Object.keys(TEMPLATE[b]).filter(f=>f!=='networkFeeLamports').map(f=>`${b}.${f}`))];
  return {manifest:m,config,candidate,missing:fields.filter(f=>f.split('.').reduce((v,k)=>v?.[k],m)==null)};
}
function planManifest(input){
  const v=validateManifest(input),m=v.manifest,common={schema:TEMPLATE.schema,sourceBaseline:BASELINE,action:'multisig-create-v2',missing:v.missing,
    status:v.missing.length?'INCOMPLETE':'OFFLINE_PLAN_ONLY',readiness:{...READINESS},sourceProvenance:RETAINED_SOURCE,limitations:[...LIMITATIONS]};
  if(v.missing.length)return {...common,plan:null};
  const members=[...m.guardians].sort((a,b)=>Buffer.compare(pk(a).toBuffer(),pk(b).toBuffer())).map(address=>({address,permissionsMask:7}));
  // Anchor discriminator + Borsh Option(None),u16,Vec<Member>,u32,Option(None),Option(None).
  const data=Buffer.concat([Buffer.from('32ddc75d28f58be9','hex'),Buffer.from('00040006000000','hex'),...members.map(g=>Buffer.concat([pk(g.address).toBuffer(),Buffer.from([7])])),Buffer.from('000000000000','hex')]);
  assert.equal(data.length,219);const roles=[
    {role:'program_config',address:v.config.address,signer:false,writable:false},
    {role:'treasury',address:m.observations.treasury,signer:false,writable:true},
    {role:'multisig',address:v.candidate.address,signer:false,writable:true},
    {role:'create_key',address:m.createKey,signer:true,writable:false},
    {role:'creator',address:m.payer,signer:true,writable:true},
    {role:'system_program',address:IDS.system,signer:false,writable:false}];
  assert.equal(new Set(roles.map(r=>r.address)).size,6);
  const ix=new TransactionInstruction({programId:pk(IDS.squads),keys:roles.map(r=>({pubkey:pk(r.address),isSigner:r.signer,isWritable:r.writable})),data}),c={m:{feePayer:m.payer,transport:m.transport}};
  const packet=initializer.packet(c,'multisig-create-v2',[ix]),message=Buffer.from(initializer.compile(c,[ix]).serialize());
  assert.equal(packet.signatureCount,2);assert.equal(packet.wireBytes,619);assert.deepEqual(packet.requiredSigners,[m.payer,m.createKey]);
  const o=m.observations,rentAndCreation=BigInt(o.rent330Lamports)+BigInt(o.creationFeeLamports);
  return {...common,declarationSha256:sha(Buffer.from(JSON.stringify(m))),plan:{program:IDS.squads,feePayer:m.payer,creator:m.payer,createKey:m.createKey,
    signerProfile:'TWO_DISTINCT_DIRECT_EOA_UNSIGNED_SIGNERS',profile:{...m.profile,status:'PROPOSED_NOT_APPROVED'},inputGuardians:[...m.guardians],inputOrderMeaning:'PUBLIC_INVENTORY_ONLY_NOT_PIV_SLOT_ORDER',members,
    derived:{programConfig:v.config,multisig:v.candidate,multisigAllocationBytes:330,existenceVerified:false},roles,native:{bytes:data.length,hex:data.toString('hex'),sha256:sha(data)},
    declaredCosts:{creationFeeLamports:o.creationFeeLamports,rent330Lamports:o.rent330Lamports,networkFeeLamports:o.networkFeeLamports,
      conditionalCreatorRentAndCreationFeeLamports:rentAndCreation.toString(),conditionalTotalIncludingDeclaredNetworkFeeLamports:present(o.networkFeeLamports)?(rentAndCreation+BigInt(o.networkFeeLamports)).toString():null,
      missingQuoteInputs:present(o.networkFeeLamports)?[]:['networkFeeLamports'],
      verified:false,actualWalletBudgetLamports:null,meaning:'DECLARED_UNVERIFIED_ZERO_PREFUND_COSTS_ONLY_NOT_AVAILABLE_BALANCE_OR_SPENDING_AUTHORITY'},
    observations:{...o,verified:false},referenceContext:{...m.transport,verified:false,usage:'ENCODING_ONLY_REFRESH_BEFORE_FUTURE_AUTHORIZED_SIGNING'},
    resourceProfile:{computeBudgetInstructions:0,priorityFeeInstructions:0,alt:false,measuredComputeUnits:null,measuredLoadedAccountBytes:null,sufficiencyVerified:false},
    messageBase64:message.toString('base64'),messageSha256:sha(message),packets:[packet]}};
}
function verifySources(){const inherited=initializer.verifySources();for(const [file,expected]of Object.entries(SOURCE_PINS))assert.equal(sha(fs.readFileSync(path.join(REPO,file))),expected,`source binding: ${file}`);return {...inherited,plannerPins:SOURCE_PINS};}
function main(args){const sourceCheck=verifySources();if(args.length===1&&args[0]==='--template'){process.stdout.write(JSON.stringify(TEMPLATE,null,2)+'\n');return;}
  if(args.length!==2||args[0]!=='--manifest')fail('unsupported_arguments','arguments');const file=path.resolve(args[1]),s=fs.statSync(file);if(!s.isFile())fail('public_manifest_file_required','manifest');if(s.size>65536)fail('manifest_size','manifest');
  const report=planManifest(initializer.parseManifest(fs.readFileSync(file,'utf8')));report.sourceCheck=sourceCheck;process.stdout.write(JSON.stringify(report,null,2)+'\n');}
module.exports={BASELINE,GENESIS,IDS,SOURCE_PINS,RETAINED_SOURCE,TEMPLATE,validateManifest,planManifest,verifySources};
if(require.main===module){try{main(process.argv.slice(2));}catch(e){process.stderr.write(JSON.stringify({status:'INVALID_OR_UNSUPPORTED_SQUADS_CREATE_PLAN',code:e.code||'plan_validation_failed',field:e.field||'manifest'})+'\n');process.exitCode=1;}}
