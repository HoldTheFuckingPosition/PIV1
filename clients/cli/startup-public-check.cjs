'use strict';
// Public-input verifier only: no wallet, signer, network, send or journal-write API.
const fs=require('node:fs'),path=require('node:path');
const {createHash,createPublicKey,verify}=require('node:crypto');
const REPO=process.env.PIV1_PUBLIC_CHECK_REPO||path.resolve(__dirname,'../..');
const SDK=path.join(REPO,'spikes/task-0.4-jito/node_modules/@solana/web3.js');
const SOURCE_PINS=Object.freeze({
  'spikes/task-0.4-jito/package-lock.json':'a47e4310aa29ffb2ec0943626616bfefb0f0b8d7408090b360018b7e5e215e89',
  'spikes/task-0.4-jito/node_modules/@solana/web3.js/lib/index.cjs.js':'796768bf5a689d808fb6fabfc5ce0db27eee064782cf87621331b807bab9f7e2'
});
const SCHEMA='piv1-startup-public-check-v1',U64=(1n<<64n)-1n,MAX_INPUT=16*1024*1024;
const TEMPLATE={schema:SCHEMA,frozen:null,cumulativeFeeCapLamports:null,phases:null,trace:null};
const sha=b=>createHash('sha256').update(b).digest('hex');
function fail(code){const e=new Error(code);e.code=code;throw e;}
const plain=x=>x!==null&&typeof x==='object'&&!Array.isArray(x)&&Object.getPrototypeOf(x)===Object.prototype;
function object(x,keys){if(!plain(x)||Object.keys(x).some(k=>!keys.includes(k))||keys.some(k=>!Object.hasOwn(x,k)))fail('object_fields');}
function list(x,max=4096){if(!Array.isArray(x)||x.length>max)fail('array_bound');return x;}
function uint(x){if(typeof x!=='string'||x.length>20||!/^(0|[1-9]\d*)$/.test(x)||BigInt(x)>U64)fail('u64_string');return BigInt(x);}
function digest(x){if(typeof x!=='string'||!(/^[a-f0-9]{64}$/.test(x)))fail('sha256');return x;}
function canonical(x){
  if(x===null||typeof x==='string'||typeof x==='boolean')return JSON.stringify(x);
  if(Array.isArray(x))return '['+x.map(canonical).join(',')+']';
  if(plain(x))return '{'+Object.keys(x).sort().map(k=>JSON.stringify(k)+':'+canonical(x[k])).join(',')+'}';
  if(Number.isSafeInteger(x)&&x>=0)return String(x);
  fail('canonical_value');
}
function equal(a,b,code){if(canonical(a)!==canonical(b))fail(code);}
function base64(s,max,empty=false){
  if(typeof s!=='string'||s.length>4*Math.ceil(max/3)||(!empty&&!s.length)||s.length%4||!/^[A-Za-z0-9+/]*={0,2}$/.test(s))fail('base64');
  const b=Buffer.from(s,'base64');if(b.length>max||b.toString('base64')!==s)fail('base64');return b;
}
let PublicKey;
function sdk(){if(!PublicKey){if(require(path.join(SDK,'package.json')).version!=='1.98.4')fail('sdk_version');PublicKey=require(SDK).PublicKey;}return PublicKey;}
function key(x){try{if(typeof x!=='string'||new (sdk())(x).toBase58()!==x)fail('public_key');return x;}catch{fail('public_key');}}
const address=b=>new (sdk())(b).toBase58();
function verifySources(){for(const [file,want]of Object.entries(SOURCE_PINS))if(sha(fs.readFileSync(path.join(REPO,file)))!==want)fail('source_pin');sdk();return {sdk:'1.98.4',pinsMatched:Object.keys(SOURCE_PINS).length};}
function shortvec(n){if(!Number.isInteger(n)||n<0||n>65535)fail('shortvec_range');const a=[];do{const v=n&127;n>>>=7;a.push(v|(n?128:0));}while(n);return Buffer.from(a);}
function reader(b){let p=0;return {
  get offset(){return p;},take(n){if(!Number.isInteger(n)||n<0||p+n>b.length)fail('truncated');const v=b.subarray(p,p+n);p+=n;return v;},
  u8(){return this.take(1)[0];},vec(){const start=p;let v=0;for(let i=0;i<3;i++){const c=this.u8();if(i===2&&(c&252))fail('shortvec_overflow');v|=(c&127)<<(7*i);if(!(c&128)){if(!b.subarray(start,p).equals(shortvec(v)))fail('shortvec_noncanonical');return v;}}fail('shortvec_overflow');},
  end(){if(p!==b.length)fail('trailing_bytes');}
};}
function decodeWire(wireBase64,lookupTables=[]){
  const wire=base64(wireBase64,1232),r=reader(wire),count=r.vec();
  if(count<1||count>127)fail('signature_count');const signatures=Array.from({length:count},()=>r.take(64));
  const message=wire.subarray(r.offset),m=reader(message),first=m.u8();
  const version=(first&128)?first&127:'legacy';if(version!=='legacy'&&version!==0)fail('message_version');
  const required=version==='legacy'?first:m.u8(),readonlySigned=m.u8(),readonlyUnsigned=m.u8(),n=m.vec();
  if(required!==count||required>n||readonlySigned>=required||readonlyUnsigned>n-required||n>256)fail('message_header');
  const staticKeys=Array.from({length:n},()=>address(m.take(32))),recentBlockhash=address(m.take(32));
  if(new Set(staticKeys).size!==n)fail('duplicate_account');
  const instructionCount=m.vec();if(instructionCount>1232)fail('instruction_bound');
  const instructions=Array.from({length:instructionCount},()=>{
    const programIdIndex=m.u8(),accounts=[...m.take(m.vec())],dataBase64=m.take(m.vec()).toString('base64');
    if(programIdIndex===0||programIdIndex>=n)fail('program_index');return {programIdIndex,accounts,dataBase64};
  });
  let lookups=[];if(version===0){const size=m.vec();if(size>32)fail('lookup_bound');lookups=Array.from({length:size},()=>({address:address(m.take(32)),writable:[...m.take(m.vec())],readonly:[...m.take(m.vec())]}));}
  if(lookups.some(t=>t.writable.length+t.readonly.length===0))fail('empty_lookup');
  m.end();
  const tableMap=new Map();for(const t of list(lookupTables,32)){object(t,['address','addresses']);key(t.address);list(t.addresses,256).forEach(key);if(tableMap.has(t.address))fail('duplicate_lookup_table');tableMap.set(t.address,t.addresses);}
  let allResolved=true;const loadedWritable=[],loadedReadonly=[];
  for(const t of lookups){const table=tableMap.get(t.address);if(!table){allResolved=false;continue;}
    for(const [indices,out]of [[t.writable,loadedWritable],[t.readonly,loadedReadonly]])for(const i of indices){if(i>=table.length)fail('lookup_index');out.push(table[i]);}}
  if(tableMap.size!==lookups.length||lookups.some(t=>!tableMap.has(t.address)))allResolved=false;
  const total=n+lookups.reduce((v,t)=>v+t.writable.length+t.readonly.length,0);
  if(total>256||instructions.some(i=>i.accounts.some(a=>a>=total)))fail('account_index');
  const staticRoles=staticKeys.map((a,i)=>({address:a,signer:i<required,writable:i<required?i<required-readonlySigned:i<n-readonlyUnsigned}));
  const roles=allResolved?[...staticRoles,...loadedWritable.map(a=>({address:a,signer:false,writable:true})),...loadedReadonly.map(a=>({address:a,signer:false,writable:false}))]:null;
  if(roles&&new Set(roles.map(a=>a.address)).size!==roles.length)fail('duplicate_account');
  return {wire,message,signatures,version,requiredSigners:staticKeys.slice(0,required),feePayer:staticKeys[0],recentBlockhash,roles,lookups,instructions,fullyResolved:allResolved,
    wireSha256:sha(wire),messageSha256:sha(message),firstSignatureHex:signatures[0].toString('hex')};
}
function verifyPacket(p){
  object(p,['signedWireBase64','reviewed']);const e=p.reviewed;
  object(e,['messageBase64','messageSha256','feePayer','requiredSigners','accountRoles','lookupTables']);
  digest(e.messageSha256);key(e.feePayer);list(e.requiredSigners,127).forEach(key);
  const d=decodeWire(p.signedWireBase64,e.lookupTables),expected=base64(e.messageBase64,1232);
  if(!d.message.equals(expected)||d.messageSha256!==e.messageSha256)fail('reviewed_message_mismatch');
  if(d.feePayer!==e.feePayer)fail('fee_payer_mismatch');equal(d.requiredSigners,e.requiredSigners,'signer_order_mismatch');
  if(!d.fullyResolved||!Array.isArray(e.accountRoles))fail('unresolved_account_roles');equal(d.roles,e.accountRoles,'account_roles_mismatch');
  const prefix=Buffer.from('302a300506032b6570032100','hex');
  for(let i=0;i<d.signatures.length;i++){
    const sig=d.signatures[i];if(sig.every(v=>v===0))fail('zero_signature');
    const pub=createPublicKey({key:Buffer.concat([prefix,new (sdk())(d.requiredSigners[i]).toBuffer()]),format:'der',type:'spki'});
    if(!verify(null,d.message,pub,sig))fail('signature_invalid');
  }
  return d;
}
function frozen(x){object(x,['packageSha256','payloadSha256','anchorUnixTimestamp','transactionIndex']);digest(x.packageSha256);digest(x.payloadSha256);if(x.anchorUnixTimestamp!==null&&uint(x.anchorUnixTimestamp)>(1n<<63n)-1n)fail('anchor_i64_overflow');uint(x.transactionIndex);return x;}
function lifetime(x,d){
  object(x,['recentBlockhash','currentBlockHeight','lastValidBlockHeight','minimumRemainingBlockHeights']);
  key(x.recentBlockhash);if(x.recentBlockhash!==d.recentBlockhash)fail('blockhash_binding');
  const now=uint(x.currentBlockHeight),last=uint(x.lastValidBlockHeight),margin=uint(x.minimumRemainingBlockHeights);
  if(margin===0n||now>last||last-now<margin)fail('lifetime_margin');
}
function account(a){
  object(a,['exists','owner','executable','lamports','dataBase64']);
  if(typeof a.exists!=='boolean')fail('account_exists');
  if(!a.exists){if(a.owner!==null||a.executable!==null||a.lamports!==null||a.dataBase64!==null)fail('absent_account');}
  else{key(a.owner);if(typeof a.executable!=='boolean')fail('account_executable');uint(a.lamports);base64(a.dataBase64,1024*1024,true);}
}
function poststate(expected,provided,slot){
  if(!plain(expected)||Object.keys(expected).length===0||Object.keys(expected).length>256)fail('expected_poststate_required');
  object(provided,['contextSlot','accounts']);if(uint(provided.contextSlot)<slot||!plain(provided.accounts))fail('poststate_context');
  equal(Object.keys(expected).sort(),Object.keys(provided.accounts).sort(),'poststate_subset');
  for(const k of Object.keys(expected)){key(k);account(expected[k]);account(provided.accounts[k]);equal(expected[k],provided.accounts[k],'poststate_mismatch');}
}
function receipt(obs,p,d,success){
  object(obs,['status','transaction','poststate']);
  if(obs.status===null)fail('status_unknown');object(obs.status,['signatureHex','slot','confirmationStatus','err']);
  object(obs.transaction,['slot','wireBase64','feeLamports','err']);
  const s=obs.status,t=obs.transaction;
  if(s.signatureHex!==d.firstSignatureHex||s.confirmationStatus!=='finalized')fail('status_not_finalized');
  const slot=uint(s.slot);if(uint(t.slot)!==slot||!base64(t.wireBase64,1232).equals(d.wire))fail('transaction_binding');
  const fee=uint(t.feeLamports);if(fee!==uint(p.feeQuoteLamports)||fee>uint(p.feeCapLamports))fail('actual_fee_mismatch');
  equal(s.err,t.err,'status_error_mismatch');if(success?(s.err!==null):(s.err===null))fail('transaction_result');
  if(success)poststate(p.expectedPoststate,obs.poststate,slot);
  else if(obs.poststate!==null)fail('failed_poststate_scope');
  return {fee,slot};
}
function entryDigest(e){return sha(Buffer.from(canonical(e)));}
function check(input){
  const base={schema:SCHEMA,liveReady:false,authorized:false,publicInputsAuthenticated:false,journalPersisted:false,journalHistoryAuthenticated:false,founderAuthorizationVerified:false,poststateCoverage:'EXACT_DECLARED_SUBSET_ONLY',signingOrSendPerformed:false};
  try{
    object(input,Object.keys(TEMPLATE));if(input.schema!==SCHEMA)fail('schema');
    const missing=Object.keys(TEMPLATE).filter(k=>input[k]===null);if(missing.length)return {...base,status:'INCOMPLETE',missing,result:null};
    frozen(input.frozen);const cap=uint(input.cumulativeFeeCapLamports),phases=list(input.phases,1024),trace=list(input.trace,4096);
    if(!phases.length)fail('phases_required');const plans=new Map(),packetHashes=new Set(),packetSignatures=new Set();
    for(const p of phases){
      object(p,['id','packet','lifetime','feeQuoteLamports','feeCapLamports','expectedPoststate']);
      if(typeof p.id!=='string'||!(/^[A-Za-z0-9_-]{1,64}$/.test(p.id))||plans.has(p.id))fail('phase_id');
      const d=verifyPacket(p.packet);lifetime(p.lifetime,d);
      if(packetHashes.has(d.wireSha256)||packetSignatures.has(d.firstSignatureHex))fail('packet_replay');packetHashes.add(d.wireSha256);packetSignatures.add(d.firstSignatureHex);
      if(uint(p.feeQuoteLamports)>uint(p.feeCapLamports))fail('fee_over_cap');
      plans.set(p.id,{p,d,state:null,started:false});
    }
    let previousDigest=null,reserved=0n,actual=0n,current=0,lastFinalizedSlot=null;const complete=[];
    for(const e of trace){
      object(e,['previousDigest','phase','state','packetSha256','signatureHex','frozen','observation']);
      if(e.previousDigest!==previousDigest)fail('journal_previous_digest');equal(frozen(e.frozen),input.frozen,'frozen_fields_changed');
      const item=plans.get(e.phase);if(!item||phases[current]?.id!==e.phase)fail('dependent_or_replayed_phase');
      const {p,d}=item;if(e.packetSha256!==d.wireSha256||e.signatureHex!==d.firstSignatureHex)fail('journal_packet_binding');
      const allowed={NONE:['PREPARED'],PREPARED:['SUBMISSION_START_RECORDED'],SUBMISSION_START_RECORDED:['UNKNOWN','FINALIZED','FAILED'],UNKNOWN:['UNKNOWN','FINALIZED','FAILED']};
      if(!(allowed[item.state||'NONE']||[]).includes(e.state))fail('journal_transition');
      if(e.state==='SUBMISSION_START_RECORDED'){
        if(item.started)fail('duplicate_submission');item.started=true;reserved+=uint(p.feeQuoteLamports);if(reserved>U64||reserved>cap)fail('cumulative_fee_cap');
      }
      if(e.state==='FINALIZED'||e.state==='FAILED'){
        const result=receipt(e.observation,p,d,e.state==='FINALIZED');actual+=result.fee;if(actual>U64||actual>cap)fail('cumulative_fee_cap');
        if(e.state==='FINALIZED'){if(lastFinalizedSlot!==null&&result.slot<=lastFinalizedSlot)fail('phase_slot_not_strictly_later');lastFinalizedSlot=result.slot;complete.push(e.phase);current++;}
      }else if(e.observation!==null)fail('nonterminal_observation');
      item.state=e.state;previousDigest=entryDigest(e);
      if(e.state==='FAILED')return {...base,status:'STOP',reason:'FINALIZED_FAILURE',actualFeeLamports:actual.toString(),reservedFeeLamports:reserved.toString(),journalDigest:previousDigest,completedPhases:complete};
    }
    const all=current===phases.length;
    return {...base,status:all?'VERIFIED_PUBLIC_DECLARATIONS_ONLY':'STOP',reason:all?'ALL_DECLARED_PHASES_FINALIZED_WITH_EXPECTED_POSTSTATE':'PENDING_OR_UNKNOWN_NO_REPLACEMENT',completedPhases:complete,
      journalDigest:previousDigest,actualFeeLamports:actual.toString(),reservedFeeLamports:reserved.toString(),dependentPhaseAllowedByDeclarations:all};
  }catch(e){return {...base,status:'STOP',reason:e.code||'invalid_input',result:null};}
}
function parseInput(raw){
  if(typeof raw!=='string'||Buffer.byteLength(raw)>MAX_INPUT)fail('input_bound');
  // Reject duplicate JSON keys lexically, including differently escaped names.
  let out;try{out=JSON.parse(raw);}catch{fail('invalid_json');}const stack=[];
  for(let i=0;i<raw.length;i++){
    const c=raw[i];if(c==='{'||c==='['){stack.push({object:c==='{',key:true,seen:new Set()});if(stack.length>32)fail('json_depth');}
    else if(c==='}'||c===']')stack.pop();else if(c===','&&stack.at(-1)?.object)stack.at(-1).key=true;
    else if(c==='"'){const start=i++;while(i<raw.length&&raw[i]!=='"'){if(raw[i]==='\\')i++;i++;}
      if(stack.at(-1)?.object&&stack.at(-1).key){const s=stack.at(-1),k=JSON.parse(raw.slice(start,i+1));if(s.seen.has(k))fail('duplicate_json_key');s.seen.add(k);s.key=false;}}
  }return out;
}
function main(args){
  const sourceCheck=verifySources();if(args.length===1&&args[0]==='--template'){process.stdout.write(JSON.stringify(TEMPLATE,null,2)+'\n');return;}
  if(args.length!==1)fail('usage');const fd=fs.openSync(args[0],'r');let raw;
  try{const st=fs.fstatSync(fd);if(!st.isFile()||st.size>MAX_INPUT)fail('input_bound');raw=fs.readFileSync(fd,'utf8');}finally{fs.closeSync(fd);}
  const report=check(parseInput(raw));report.sourceCheck=sourceCheck;process.stdout.write(JSON.stringify(report,null,2)+'\n');if(report.status==='STOP')process.exitCode=1;
}
if(require.main===module){try{main(process.argv.slice(2));}catch(e){process.stderr.write(JSON.stringify({error:e.code||'invalid_input'})+'\n');process.exitCode=1;}}
module.exports={SCHEMA,TEMPLATE,SOURCE_PINS,verifySources,decodeWire,verifyPacket,check,parseInput,entryDigest,shortvec,uint,canonical};
