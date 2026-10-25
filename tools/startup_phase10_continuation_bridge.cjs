'use strict';
// Outer suffix authority names the new extension and the immutable core library.
// Native key reading/signing remain solely in the pinned original backend.
const fs=require('node:fs'),path=require('node:path'),crypto=require('node:crypto');
const REPO=path.resolve(__dirname,'..');
const sha=x=>crypto.createHash('sha256').update(x).digest('hex');
function need(ok,reason){if(!ok)throw Error(reason);}
function sorted(x){if(Array.isArray(x))return x.map(sorted);if(x&&typeof x==='object')return Object.fromEntries(Object.keys(x).sort().map(k=>[k,sorted(x[k])]));return x;}
const encode=x=>Buffer.from(JSON.stringify(sorted(x))+'\n');
const SOURCE_NAMES=['startup_release_operator.py','startup_manifest.py','startup_manifest.oracle.json','startup_signer_backend.cjs','collect_startup_public_evidence.py','startup_phase10_exception.py','startup_phase10_signer_bridge.cjs','startup_phase10_continuation.py','startup_phase10_continuation_bridge.cjs'].sort();
function run(payload,options={}){
  need(options.allowLiveSign===true,'live_sign_not_enabled');
  need(payload&&Object.keys(payload).sort().join(',')==='input,outerAuthorizationRawBase64,outerAuthorizationSha256','suffix_bridge_fields');
  const raw=Buffer.from(payload.outerAuthorizationRawBase64,'base64');
  need(raw.length<=32768&&raw.toString('base64')===payload.outerAuthorizationRawBase64&&sha(raw)===payload.outerAuthorizationSha256,'suffix_outer_hash');
  const a=JSON.parse(raw.toString('utf8')),input=payload.input;
  const check=()=>{
    need(encode(a).equals(raw)&&a.schema==='piv1-au-phase10-continuation-authorization-v1'&&a.scope==='EXACT_PHASE11_948_AFTER_NEW_PHASE10_FINALIZED','suffix_outer_authorization');
    need(/^(0|[1-9][0-9]{0,19})$/.test(a.expiresUnixTimestamp)&&BigInt(a.expiresUnixTimestamp)>BigInt(Math.floor((options.now||Date.now)()/1000)),'suffix_outer_expired');
    need(a.originalTail==='8e5cf7b6aa2be23bd6ed08a0fa652a4f58d4141310043b8db5f13e76e8e58a18'&&a.carriedPossibleFeeLamports==='5000'&&a.combinedFeeCapLamports==='4820000'&&a.transportMode==='FIXED_ORIGIN_TLS','suffix_old_unknown_scope');
    need(Object.keys(a.sourcePins).sort().join(',')===SOURCE_NAMES.join(','),'suffix_source_set');
    for(const [name,digest] of Object.entries(a.sourcePins)){
      const file=path.join(REPO,'tools',name);
      need(!fs.lstatSync(file).isSymbolicLink()&&sha(fs.readFileSync(file))===digest,'suffix_source_pin');
    }
    need(a.sourcePins['startup_phase10_continuation_bridge.cjs']===sha(fs.readFileSync(__filename)),'suffix_bridge_source');
  };
  check();
  const innerRaw=Buffer.from(input.authorizationRawBase64,'base64');
  need(innerRaw.toString('base64')===a.backendAuthorizationRawBase64&&sha(innerRaw)===a.backendAuthorizationSha256&&input.authorizationSha256===a.backendAuthorizationSha256,'suffix_nested_hash');
  const inner=JSON.parse(innerRaw.toString('utf8')),r=input.request;
  need(encode(inner).equals(innerRaw)&&inner.schema==='piv1-launcher-release-authorization-v2'&&inner.scope==='EXACT_BOUNDED_TESTNET_RELEASE'&&inner.expiresUnixTimestamp===a.expiresUnixTimestamp&&inner.manifestSha256===a.manifestSha256&&inner.operatorSourceSha256===a.sourcePins['startup_release_operator.py']&&inner.signerBackendSha256===a.sourcePins['startup_signer_backend.cjs']&&inner.feeBudgetNamespace==='piv1-testnet-startup-release-'+a.packageSha256,'suffix_nested_contract');
  const body={...r};delete body.requestId;
  need(r.schema==='piv1-startup-signing-request-v1'&&sha(encode(body))===r.requestId&&r.baseParentPath===a.baseParentPath&&r.manifestSha256===a.manifestSha256&&r.operatorSourceSha256===inner.operatorSourceSha256&&r.authorizationSha256===a.backendAuthorizationSha256&&encode(r.baseParentTail).equals(encode(inner.baseParentTail)),'suffix_request_binding');
  need(Array.isArray(input.packets)&&input.packets.length>=1&&input.packets.length<=64&&r.packets.length===input.packets.length,'suffix_packet_count');
  for(let i=0;i<input.packets.length;i++){
    const p=input.packets[i],n=Number(p.id&&p.id.slice(8));
    need(Number.isInteger(n)&&n>=11&&n<=948&&p.id==='startup_'+String(n).padStart(4,'0')&&r.packets[i].id===p.id,'suffix_phase_range');
  }
  const backend=require('./startup_signer_backend.cjs');
  const opt={allowLiveSign:true,now:options.now,readSecure:(file,bound)=>{check();return (options.readSecure||backend.secureRead)(file,bound);},signMessage:(message,key,...rest)=>{check();return options.signMessage?options.signMessage(message,key,...rest):crypto.sign(null,message,key);}};
  if(options.loadSigner)opt.loadSigner=options.loadSigner;
  // Return the complete public response for parent retention. A later source
  // change must refuse dispatch after that fsync, never discard signed output.
  return backend.run(input,opt);
}
if(require.main===module){
  try{
    need(process.argv.length===3&&process.argv[2]==='--allow-live-sign','live_sign_not_enabled');
    const chunks=[];let count=0;
    while(true){const b=Buffer.alloc(65536),n=fs.readSync(0,b,0,b.length,null);if(!n)break;count+=n;need(count<=1048576,'input_bound');chunks.push(b.subarray(0,n));}
    process.stdout.write(JSON.stringify(run(JSON.parse(Buffer.concat(chunks)),{allowLiveSign:true}))+'\n');
  }catch{process.stderr.write('{"status":"STOP","reason":"suffix_bridge_refused"}\n');process.exitCode=1;}
}
module.exports={run};
