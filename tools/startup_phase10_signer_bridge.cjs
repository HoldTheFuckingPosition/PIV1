'use strict';
// Fresh outer exception grant + fresh, explicitly nested compatibility receipt.
// The original backend and its source identity remain unchanged.
const fs=require('node:fs'),path=require('node:path'),crypto=require('node:crypto');
const REPO=path.resolve(__dirname,'..');
const sha=b=>crypto.createHash('sha256').update(b).digest('hex');
const canonical=x=>Buffer.from(JSON.stringify(x,(_,v)=>v)+'\n');
function need(ok,reason){if(!ok)throw Error(reason);}
function sorted(x){if(Array.isArray(x))return x.map(sorted);if(x&&typeof x==='object')return Object.fromEntries(Object.keys(x).sort().map(k=>[k,sorted(x[k])]));return x;}
const encode=x=>canonical(sorted(x));
function run(input,options={}){
  need(options.allowLiveSign===true,'live_sign_not_enabled');
  need(input&&Object.keys(input).sort().join(',')==='authorizationRawBase64,authorizationSha256,packets,request,signerProfilePath','bridge_fields');
  const raw=Buffer.from(input.authorizationRawBase64,'base64');
  need(raw.length<=32768&&raw.toString('base64')===input.authorizationRawBase64&&sha(raw)===input.authorizationSha256,'outer_authorization_hash');
  const outer=JSON.parse(raw.toString('utf8')),request=input.request;
  const check=()=>{
    need(encode(outer).equals(raw)&&outer.schema==='piv1-au-phase10-authorization-v1'&&outer.scope==='EXACT_SINGLE_AU_PHASE10_EXCEPTION','outer_authorization');
    need(/^(0|[1-9][0-9]{0,19})$/.test(outer.expiresUnixTimestamp)&&BigInt(outer.expiresUnixTimestamp)>BigInt(Math.floor((options.now||Date.now)()/1000)),'outer_authorization_expired');
    need(outer.originalTail==='8e5cf7b6aa2be23bd6ed08a0fa652a4f58d4141310043b8db5f13e76e8e58a18','original_tail');
    for(const [name,digest] of Object.entries(outer.sourcePins)){
      need(['startup_release_operator.py','startup_manifest.py','startup_manifest.oracle.json','startup_signer_backend.cjs','startup_phase10_exception.py','startup_phase10_signer_bridge.cjs'].includes(name),'source_name');
      const file=path.join(REPO,'tools',name);
      need(!fs.lstatSync(file).isSymbolicLink()&&sha(fs.readFileSync(file))===digest,'outer_source_pin');
    }
    need(Object.keys(outer.sourcePins).length===6&&outer.sourcePins['startup_phase10_signer_bridge.cjs']===sha(fs.readFileSync(__filename)),'outer_source_set');
  };
  check();
  const innerRaw=Buffer.from(outer.backendAuthorizationRawBase64,'base64'),inner=JSON.parse(innerRaw.toString('utf8'));
  need(sha(innerRaw)===outer.backendAuthorizationSha256&&encode(inner).equals(innerRaw),'nested_receipt_hash');
  need(inner.schema==='piv1-launcher-release-authorization-v2'&&inner.scope==='EXACT_BOUNDED_TESTNET_RELEASE'&&inner.operatorSourceSha256===outer.sourcePins['startup_release_operator.py']&&inner.signerBackendSha256===outer.sourcePins['startup_signer_backend.cjs'],'nested_original_source_contract');
  need(inner.manifestSha256===outer.packageSha256&&inner.expiresUnixTimestamp===outer.expiresUnixTimestamp&&inner.signerProfileSha256===outer.signerProfileSha256&&inner.feeBudgetNamespace===outer.feeBudgetNamespace&&encode(inner.baseParentTail).equals(encode({sequence:'54',digest:outer.originalTail})),'nested_outer_binding');
  const body={...request};delete body.requestId;
  need(request.schema==='piv1-au-phase10-signing-request-v1'&&sha(encode(body))===request.requestId&&request.authorizationSha256===input.authorizationSha256&&request.operatorSourceSha256===outer.sourcePins['startup_phase10_exception.py']&&request.baseParentPath===outer.journalPath&&request.manifestSha256===outer.packageSha256&&request.baseParentTail.digest===outer.originalTail&&request.baseParentTail.sequence==='54'&&request.sealSha256===null,'outer_request_binding');
  need(input.packets.length===1&&request.packets.length===1&&input.packets[0].id==='startup_0010'&&request.packets[0].id==='startup_0010'&&input.packets[0].feeQuoteLamports==='5000','single_phase10_packet');
  const oracle=JSON.parse(fs.readFileSync(path.join(REPO,'tools/startup_manifest.oracle.json'))),packet=input.packets[0],wire=Buffer.from(packet.wireBase64,'base64'),template=Buffer.from(oracle.writeTemplate.prefixBase64,'base64');
  const message=wire.subarray(65),data=message.subarray(template.length+2);
  need(wire.length===1232&&wire[0]===1&&wire.subarray(1,65).every(v=>v===0)&&message.subarray(0,132).equals(template.subarray(0,132))&&message.subarray(164,template.length).equals(template.subarray(164))&&message[template.length]===0xcc&&message[template.length+1]===7&&data.readUInt32LE(0)===1&&data.readUInt32LE(4)===8604&&data.readBigUInt64LE(8)===956n&&sha(data.subarray(16))===outer.writeSliceSha256&&outer.writeSliceSha256==='bfce07c106862fa4be454ce975c6b13c2cc6d70906740a0a8a0dbbf6b04092fc','exact_phase10_semantics');
  // This distinct request honestly names the immutable original backend/core.
  const compatibility={...request,schema:'piv1-startup-signing-request-v1',operatorSourceSha256:inner.operatorSourceSha256,authorizationSha256:outer.backendAuthorizationSha256};
  delete compatibility.requestId;compatibility.requestId=sha(encode(compatibility));
  const backend=require('./startup_signer_backend.cjs');
  const backendInput={...input,authorizationRawBase64:outer.backendAuthorizationRawBase64,authorizationSha256:outer.backendAuthorizationSha256,request:compatibility};
  const backendOptions={allowLiveSign:true,now:options.now,readSecure:(file,bound)=>{check();return (options.readSecure||backend.secureRead)(file,bound);},signMessage:(message,key)=>{check();return crypto.sign(null,message,key);}};
  // Test-only options remain programmatic; CLI cannot supply private I/O seams.
  if(options.loadSigner)backendOptions.loadSigner=options.loadSigner;
  if(options.signMessage)backendOptions.signMessage=(message,key,...rest)=>{check();return options.signMessage(message,key,...rest);};
  const response=backend.run(backendInput,backendOptions);check();
  return {schema:'piv1-au-phase10-signing-response-v1',requestId:request.requestId,backendRequest:compatibility,backendResponse:response};
}
if(require.main===module){
  try{
    need(process.argv.length===3&&process.argv[2]==='--allow-live-sign','live_sign_not_enabled');
    const chunks=[];let count=0;
    while(true){const chunk=Buffer.alloc(65536),n=fs.readSync(0,chunk,0,chunk.length,null);if(!n)break;count+=n;need(count<=1048576,'input_bound');chunks.push(chunk.subarray(0,n));}
    const raw=Buffer.concat(chunks);
    process.stdout.write(JSON.stringify(run(JSON.parse(raw),{allowLiveSign:true}))+'\n');
  }catch{process.stderr.write('{"status":"STOP","reason":"phase10_bridge_refused"}\n');process.exitCode=1;}
}
module.exports={run};
