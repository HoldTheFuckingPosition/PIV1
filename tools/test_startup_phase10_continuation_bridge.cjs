'use strict';
// Fake-only callback tests use retained public signatures, no native key work.
const test=require('node:test'),assert=require('node:assert/strict');
const fs=require('node:fs'),path=require('node:path'),crypto=require('node:crypto');
const root=path.resolve(__dirname,'..'),bridge=require('./startup_phase10_continuation_bridge.cjs');
const checker=require('../clients/cli/startup-public-check.cjs');
const sha=b=>crypto.createHash('sha256').update(b).digest('hex');
const encode=x=>Buffer.from(checker.canonical(x)+'\n');
const tx=JSON.parse(fs.readFileSync(path.join(root,'clients/cli/startup-public-check.fixture.json'))).transactions[1],signed=checker.decodeWire(tx.signedWireBase64,[]);
const names=['startup_release_operator.py','startup_manifest.py','startup_manifest.oracle.json','startup_signer_backend.cjs','collect_startup_public_evidence.py','startup_phase10_exception.py','startup_phase10_signer_bridge.cjs','startup_phase10_continuation.py','startup_phase10_continuation_bridge.cjs'];
function fixture(){
 const profile={schema:'piv1-disposable-testnet-signer-profile-v1',cluster:'testnet',genesisHash:'4uhcVJyU9pJkvQyS88uRDiswHXSCkY3zQawwpjk2NsNY',disposableTestnetOnly:true,signers:signed.requiredSigners.map((address,i)=>({address,secretKeyPath:'/not-opened-private/'+i+'.json'}))};
 const profileRaw=encode(profile),wire=Buffer.from(signed.wire);wire.fill(0,1,1+64*signed.signatures.length);
 const packet={id:'startup_0011',wireBase64:wire.toString('base64'),lifetime:{recentBlockhash:signed.recentBlockhash,currentBlockHeight:'100',lastValidBlockHeight:'1000',minimumRemainingBlockHeights:'40'},feeQuoteLamports:tx.feeLamports,reviewed:{messageBase64:signed.message.toString('base64'),messageSha256:signed.messageSha256,feePayer:signed.feePayer,requiredSigners:signed.requiredSigners,accountRoles:signed.roles,lookupTables:[]}};
 const pins=Object.fromEntries(names.map(n=>[n,sha(fs.readFileSync(path.join(__dirname,n)))]));
 const inner={schema:'piv1-launcher-release-authorization-v2',manifestSha256:'c'.repeat(64),baseParentTail:{sequence:'11',digest:'a'.repeat(64)},operatorSourceSha256:pins['startup_release_operator.py'],expiresUnixTimestamp:'18446744073709551615',scope:'EXACT_BOUNDED_TESTNET_RELEASE',feeBudgetNamespace:'piv1-testnet-startup-release-'+'b'.repeat(64),signerBackendSha256:pins['startup_signer_backend.cjs'],signerProfileSha256:sha(profileRaw)};
 const raw=encode(inner),body={schema:'piv1-startup-signing-request-v1',cluster:'testnet',genesisHash:profile.genesisHash,operatorTail:'d'.repeat(64),operatorSourceSha256:inner.operatorSourceSha256,authorizationSha256:sha(raw),baseParentPath:'/public-original-parent',baseParentTail:inner.baseParentTail,manifestSha256:inner.manifestSha256,sealSha256:null,packets:[{id:packet.id,preparedRecordDigest:'e'.repeat(64),preparedSha256:sha(encode(packet)),unsignedWireSha256:sha(wire),messageSha256:signed.messageSha256,requiredSigners:signed.requiredSigners}]};
 const input={authorizationRawBase64:raw.toString('base64'),authorizationSha256:sha(raw),request:{...body,requestId:sha(encode(body))},packets:[packet],signerProfilePath:'/not-opened-private/profile.json'};
 const outer={schema:'piv1-au-phase10-continuation-authorization-v1',scope:'EXACT_PHASE11_948_AFTER_NEW_PHASE10_FINALIZED',packageSha256:'b'.repeat(64),journalPath:'/public-suffix-journal',baseParentPath:body.baseParentPath,sourcePins:pins,phase10Tail:'f'.repeat(64),originalTail:'8e5cf7b6aa2be23bd6ed08a0fa652a4f58d4141310043b8db5f13e76e8e58a18',expiresUnixTimestamp:inner.expiresUnixTimestamp,transportMode:'FIXED_ORIGIN_TLS',backendAuthorizationRawBase64:raw.toString('base64'),backendAuthorizationSha256:sha(raw),manifestSha256:inner.manifestSha256,carriedPossibleFeeLamports:'5000',combinedFeeCapLamports:'4820000'};
 const reads=[],signs=[];let now=1000;
 const options={allowLiveSign:true,now:()=>now,readSecure(file){reads.push(file);return file===input.signerProfilePath?profileRaw:Buffer.from('PUBLIC PLACEHOLDER');},loadSigner(raw,address){assert.equal(raw.toString(),'PUBLIC PLACEHOLDER');return {address};},signMessage(message,key,address,id,index){assert.equal(id,'startup_0011');assert(message.equals(signed.message));signs.push(address);return Buffer.from(signed.signatures[index]);}};
 return {outer,input,reads,signs,options,setNow:x=>now=x,payload(){const raw=encode(outer);return {outerAuthorizationRawBase64:raw.toString('base64'),outerAuthorizationSha256:sha(raw),input};}};
}
function trapped(fn){
 const originalSign=crypto.sign,originalPrivate=crypto.createPrivateKey,open=fs.openSync;let forbidden=0;
 crypto.sign=crypto.createPrivateKey=()=>{forbidden++;throw Error('native signing/key forbidden');};
 fs.openSync=function(file,...args){if(typeof file==='string'&&file.startsWith('/not-opened-private/')){forbidden++;throw Error('private open forbidden');}return open.call(this,file,...args);};
 try{fn();assert.equal(forbidden,0);}finally{crypto.sign=originalSign;crypto.createPrivateKey=originalPrivate;fs.openSync=open;}
}
test('exact outer suffix contract delegates retained public signature through fakes',()=>trapped(()=>{
 const x=fixture(),r=bridge.run(x.payload(),x.options);assert.equal(r.packets[0].wireBase64,tx.signedWireBase64);assert.deepEqual(x.signs,signed.requiredSigners);
 // After the last private callback, the original backend can return a complete
 // public response even when the new extension changes. The parent must retain
 // that response before its own fresh grant/source barrier refuses dispatch.
 const y=fixture(),read=y.options.readSecure,readFile=fs.readFileSync;let profiles=0,drift=false;
 y.options.readSecure=(file,bound)=>{const raw=read(file,bound);if(file===y.input.signerProfilePath&&++profiles===2)drift=true;return raw;};
 fs.readFileSync=function(file,...args){const raw=readFile.call(this,file,...args);return drift&&file===path.join(__dirname,'startup_phase10_continuation.py')?Buffer.concat([raw,Buffer.from('\n')]):raw;};
 try{const retained=bridge.run(y.payload(),y.options);assert.equal(profiles,2);assert(drift);assert.equal(retained.packets[0].wireBase64,tx.signedWireBase64);}finally{fs.readFileSync=readFile;}

}));
test('phase10, fake history, wrong carry and source changes refuse before private read',()=>trapped(()=>{
 for(const change of [x=>x.input.packets[0].id='startup_0010',x=>x.outer.transportMode='OFFLINE_INJECTED_TEST',x=>x.outer.carriedPossibleFeeLamports='0',x=>x.outer.sourcePins['startup_phase10_continuation.py']='0'.repeat(64)]){
  const x=fixture();change(x);assert.throws(()=>bridge.run(x.payload(),x.options));assert.equal(x.reads.length,0);assert.equal(x.signs.length,0);
 }
}));
test('fresh outer expiry is checked again inside each key/sign callback',()=>trapped(()=>{
 const x=fixture(),read=x.options.readSecure;x.options.readSecure=(file,bound)=>{const result=read(file,bound);x.setNow(Number(18446744073709551615n)*1000);return result;};
 assert.throws(()=>bridge.run(x.payload(),x.options),/expired/);assert.equal(x.reads.length,1);assert.equal(x.signs.length,0);
}));
