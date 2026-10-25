'use strict';
// Fake-only boundary tests. This expired PUBLIC AU phase10 wire is retained
// from original record52 (SHA b9eb2074e46dec81afadab3131d3fa3fbc04bdfb1bff8088e22dae203c643104).
// Fake callbacks return its existing signature; no key or new signature exists.
const test=require('node:test'),assert=require('node:assert/strict');
const fs=require('node:fs'),path=require('node:path'),crypto=require('node:crypto');
const bridge=require('./startup_phase10_signer_bridge.cjs'),backend=require('./startup_signer_backend.cjs');
const checker=require('../clients/cli/startup-public-check.cjs');
const sha=b=>crypto.createHash('sha256').update(b).digest('hex');
const encode=x=>Buffer.from(checker.canonical(x)+'\n');
const PUBLIC_WIRE='ATVmv4ehfOmuvhdzsICD0v48O0YkNaT+CxDhVzRr4rE8Fb+eHMI8d/Uu5ZABR6nTgMplVs5z65+T5IA0ZtXFRwYBAAIEm7teaa47rVI7AYeEvX7lvFwpMd9ZBA6BfJOGsv3BpmLxq76lknm+zbMUzUSP3W1uZ0O01Nh+Xsq0KjVdqIsVlAMGRm/lIRcy/+ytunLDm+e8jOW7xfcSayxDmzpAAAAAAqj2kU6IobDiEBU+92OuKwDCuT0WwSTSwFN6EASAAAAp7F7XRfKmSpZaul6OsE9+LD805DmGELMc8GcB+zU2ZQQCAAUCwFwVAAIABQEAAAQAAgAFBAAAAAQDAgEAzAcBAAAAnCEAALwDAAAAAAAAGgAAAAAAAAAAAACAeaKg/gAAAAAdksr/AAAAAHmhsP4AAAAAexYYAAAAAAB5oaj+AAAAAHsWEAAAAAAAeaGg/gAAAAB7FggAAAAAAHoGAAABAAAAlQAAAAAAAAC3BAAAAAAAAFUEz/8AAAAAvzIAAAAAAAAYBwAAAH4AAAAAAAADAAAAFQHO/wAAAABXAgAA+P///78nAAAAAAAAe2qY/gAAAAAYAQAABwAAAAAAAAADAAAALRfM/wAAAAC3AQAACAAAALcCAAAAAgAAGAMAAGCxCwAAAAAAAAAAAIUQAADhWgEAtwgAAAEAAAAFABAAAAAAAL9xAAAAAAAAD2EAAAAAAAC/qQAAAAAAAAcJAACA////v5IAAAAAAAC3AwAAgAAAAIUQAAALaQEABwgAAAEAAAB7iuD+AAAAAL+iAAAAAAAABwIAAGj///+/kQAAAAAAAIUQAADF+///BwYAAIAAAABxofr/AAAAABUBz/8CAAAAeaHQ/gAAAABdGO7/AAAAAL+hAAAAAAAABwEAAND+//+/ggAAAAAAALcDAAABAAAAtwQAAAgAAAC3BQAAgAAAAIUQAAADBQAAeafY/gAAAAAFAOX/AAAAAL8WAAAAAAAAGAkAABoAAAAAAAAAAAAAgHuaEP8AAAAAeSEQAAAAAAB7Gjj/AAAAAHkhCAAAAAAAexow/wAAAAB5IQAAAAAAAHsaKP8AAAAAv6EAAAAAAAAHAQAAEP///3saQP8AAAAAv6EAAAAAAAAHAQAAYP///7+iAAAAAAAABwIAACj///+FEAAAGfz//xgBAAABAAAAAAAAAAAAAIB5omD/AAAAAF0SCgAAAAAAtwcAAAgAAAC3CAAAAAAAALcBAAAAAAAAeaIQ/wAAAABdkjoAAAAAAHuGGAAAAAAAe3YQAAAAAAB7FggAAAAAAHoGAAAAAAAABQA8AAAAAACFEAAA6kcBABgBAAAAAAAAAAAAAAMAAAB5EQAAAAAAAL8TAAAAAAAABwMAAAD///+3AgAAAAAAALcEAAABAAAAvRM0AAAAAAAVBDUAAAAAABgHAAAAfwAAAAAAAAMAAABVATYAAAAAAHtqCP8AAAAAGAEAAAcAAAAAAAAAAwAAAL0XOAAAAAAAGAEAAAAAAAAAAAAAAwAAAHtxAAAAAAAAv6IAAAAAAAAHAgAAYP///7cGAABAAAAAv3EAAAAAAAC3AwAAQAAAAIUQAAC9aAEAe3pQ/wAAAAB6Clj/AQAAAHoKSP8EAAAAeaFA/wAAAAA=';
const signed=checker.decodeWire(PUBLIC_WIRE,[]);
assert.equal(sha(signed.wire),'9d77da3aeca37e219e968c90d0b89b874c6e56bc5926db0e5e95d2e668b931ab');
const names=['startup_release_operator.py','startup_manifest.py','startup_manifest.oracle.json','startup_signer_backend.cjs','startup_phase10_exception.py','startup_phase10_signer_bridge.cjs'];
function fixture(){
 const profilePath='/not-opened-private/profile.json',reads=[],signs=[];let now=0;
 const profile={schema:'piv1-disposable-testnet-signer-profile-v1',cluster:'testnet',genesisHash:'4uhcVJyU9pJkvQyS88uRDiswHXSCkY3zQawwpjk2NsNY',disposableTestnetOnly:true,
  signers:signed.requiredSigners.map(address=>({address,secretKeyPath:'/not-opened-private/g0.json'}))};
 const profileRaw=encode(profile),wire=Buffer.from(signed.wire);wire.fill(0,1,65);
 const packet={id:'startup_0010',wireBase64:wire.toString('base64'),feeQuoteLamports:'5000',
  lifetime:{recentBlockhash:signed.recentBlockhash,currentBlockHeight:'407927313',lastValidBlockHeight:'407927460',minimumRemainingBlockHeights:'40'},
  reviewed:{messageBase64:signed.message.toString('base64'),messageSha256:signed.messageSha256,feePayer:signed.feePayer,requiredSigners:signed.requiredSigners,accountRoles:signed.roles,lookupTables:[]}};
 const pins=Object.fromEntries(names.map(name=>[name,sha(fs.readFileSync(path.join(__dirname,name)))])),tail='8e5cf7b6aa2be23bd6ed08a0fa652a4f58d4141310043b8db5f13e76e8e58a18',packageSha='a'.repeat(64);
 const inner={schema:'piv1-launcher-release-authorization-v2',manifestSha256:packageSha,baseParentTail:{sequence:'54',digest:tail},operatorSourceSha256:pins['startup_release_operator.py'],expiresUnixTimestamp:'1000',
  scope:'EXACT_BOUNDED_TESTNET_RELEASE',feeBudgetNamespace:'piv1-testnet-startup-release-'+packageSha,signerBackendSha256:pins['startup_signer_backend.cjs'],signerProfileSha256:sha(profileRaw)},innerRaw=encode(inner);
 const outer={schema:'piv1-au-phase10-authorization-v1',scope:'EXACT_SINGLE_AU_PHASE10_EXCEPTION',journalPath:'/not-opened-private/au-phase10-'+tail+'.exception',packageSha256:packageSha,originalTail:tail,sourcePins:pins,
  expiresUnixTimestamp:'1000',signerProfileSha256:sha(profileRaw),feeBudgetNamespace:inner.feeBudgetNamespace,backendAuthorizationRawBase64:innerRaw.toString('base64'),backendAuthorizationSha256:sha(innerRaw),
  writeSliceSha256:'bfce07c106862fa4be454ce975c6b13c2cc6d70906740a0a8a0dbbf6b04092fc'};
 const options={allowLiveSign:true,now:()=>now,readSecure(file){reads.push(file);return file===profilePath?profileRaw:Buffer.from('PUBLIC PLACEHOLDER');},
  loadSigner(raw,address){assert.equal(raw.toString(),'PUBLIC PLACEHOLDER');return {address};},
  signMessage(message,key,address,id,index){assert.equal(id,'startup_0010');assert(message.equals(signed.message));signs.push(address);return Buffer.from(signed.signatures[index]);}};
 return {outer,packet,options,reads,signs,setNow:value=>now=value,input(){
  const raw=encode(outer),body={schema:'piv1-au-phase10-signing-request-v1',cluster:'testnet',genesisHash:profile.genesisHash,operatorTail:'0'.repeat(64),operatorSourceSha256:pins['startup_phase10_exception.py'],
   authorizationSha256:sha(raw),baseParentPath:outer.journalPath,baseParentTail:inner.baseParentTail,manifestSha256:packageSha,sealSha256:null,
   packets:[{id:packet.id,preparedRecordDigest:'0'.repeat(64),preparedSha256:sha(encode(packet)),unsignedWireSha256:sha(Buffer.from(packet.wireBase64,'base64')),messageSha256:packet.reviewed.messageSha256,requiredSigners:packet.reviewed.requiredSigners}]};
  return {authorizationRawBase64:raw.toString('base64'),authorizationSha256:sha(raw),request:{...body,requestId:sha(encode(body))},packets:[packet],signerProfilePath:profilePath};
 }};
}
function trapped(fn){
 const originalSign=crypto.sign,originalPrivate=crypto.createPrivateKey,originalGenerate=crypto.generateKeyPairSync,open=fs.openSync,lstat=fs.lstatSync;
 const net=require('node:net'),child=require('node:child_process'),connect=net.Socket.prototype.connect;
 const processes=Object.fromEntries(['spawn','spawnSync','exec','execSync','execFile','execFileSync','fork'].map(name=>[name,child[name]]));let forbidden=0;
 const refuse=()=>{forbidden++;throw Error('native key/sign/network/child forbidden');};
 crypto.sign=crypto.createPrivateKey=crypto.generateKeyPairSync=refuse;net.Socket.prototype.connect=refuse;
 for(const name of Object.keys(processes))child[name]=refuse;
 const forbiddenPath=file=>typeof file==='string'&&(file.startsWith('/not-opened-private/')||file==='/home/jerem/.piv1-rpc'||file.startsWith('/home/jerem/.piv1-rpc/'));
 fs.openSync=function(file,...args){if(forbiddenPath(file))return refuse();return open.call(this,file,...args);};
 fs.lstatSync=function(file,...args){if(forbiddenPath(file))return refuse();return lstat.call(this,file,...args);};
 try{fn();assert.equal(forbidden,0);}finally{
  crypto.sign=originalSign;crypto.createPrivateKey=originalPrivate;crypto.generateKeyPairSync=originalGenerate;fs.openSync=open;fs.lstatSync=lstat;net.Socket.prototype.connect=connect;
  for(const [name,value] of Object.entries(processes))child[name]=value;
 }
}
test('complete backend response survives late outer source or expiry after return',()=>trapped(()=>{
 const baseline=fixture();assert.equal(bridge.run(baseline.input(),baseline.options).backendResponse.packets[0].wireBase64,PUBLIC_WIRE);
 for(const boundary of ['source','expiry']){
  const x=fixture(),run=backend.run,read=fs.readFileSync;let completed=false,drift=false;
  backend.run=function(...args){const response=run.apply(this,args);completed=true;if(boundary==='source')drift=true;else x.setNow(2000000);return response;};
  fs.readFileSync=function(file,...args){const raw=read.call(this,file,...args);return drift&&file===path.join(__dirname,'startup_phase10_exception.py')?Buffer.concat([raw,Buffer.from('\n')]):raw;};
  try{const response=bridge.run(x.input(),x.options);assert(completed);assert.equal(response.backendResponse.packets[0].wireBase64,PUBLIC_WIRE);assert.deepEqual(x.signs,signed.requiredSigners);}
  finally{backend.run=run;fs.readFileSync=read;}
 }
}));
test('wrong Write, source pin and initial expiry refuse before private callbacks',()=>trapped(()=>{
 for(const change of [x=>{const wire=Buffer.from(x.packet.wireBase64,'base64');wire[wire.length-1]^=1;x.packet.wireBase64=wire.toString('base64');},
  x=>x.outer.sourcePins['startup_phase10_exception.py']='0'.repeat(64),x=>x.setNow(2000000)]){
  const x=fixture();change(x);assert.throws(()=>bridge.run(x.input(),x.options));assert.equal(x.reads.length,0);assert.equal(x.signs.length,0);
 }
}));
test('outer expiry is rechecked after a private callback before any signing',()=>trapped(()=>{
 const x=fixture(),read=x.options.readSecure;x.options.readSecure=(file,bound)=>{const raw=read(file,bound);x.setNow(2000000);return raw;};
 assert.throws(()=>bridge.run(x.input(),x.options),/expired/);assert.equal(x.reads.length,1);assert.equal(x.signs.length,0);
}));
test('seven-source route binds synthetic credential and rechecks after its I/O',()=>trapped(()=>{
 const directory=fs.mkdtempSync(path.join(require('node:os').tmpdir(),'piv1-phase10-rpc-'));
 fs.chmodSync(directory,0o700);
 const endpoint=path.join(directory,'synthetic-endpoint.url'),raw=Buffer.from('SYNTHETIC_OWNER_ONLY_ENDPOINT_NO_LIVE_AUTHORITY\n');
 function privateFixture(){
  fs.writeFileSync(endpoint,raw,{mode:0o600});
  const x=fixture();x.outer.schema='piv1-au-phase10-authorization-v2';
  const source='startup_phase10_rpc.py',digest=sha(fs.readFileSync(path.join(__dirname,source)));
  x.outer.sourcePins[source]=digest;
  x.outer.rpcTransport={schema:'piv1-phase10-private-rpc-binding-v1',mode:'QUICKNODE_TESTNET_SAME_SLOT_FIVE_PLUS_FIVE',sourceSha256:digest,endpointCredentialSha256:sha(raw)};
  x.options.readRpcCredential=()=>{
   assert.equal(fs.lstatSync(directory).mode&0o7777,0o700);
   const fd=fs.openSync(endpoint,fs.constants.O_RDONLY|fs.constants.O_NOFOLLOW);
   try{
    const before=fs.fstatSync(fd,{bigint:true});assert(before.isFile()&&before.uid===BigInt(process.getuid())&&before.nlink===1n&&(Number(before.mode)&0o7777)===0o600&&before.size<=4096n);
    const value=fs.readFileSync(fd);assert.equal(BigInt(value.length),before.size);
    assert.equal(fs.fstatSync(fd,{bigint:true}).mtimeNs,before.mtimeNs);return value;
   }finally{fs.closeSync(fd);}
  };return x;
 }
 try{
  const baseline=privateFixture();assert.equal(bridge.run(baseline.input(),baseline.options).backendResponse.packets[0].wireBase64,PUBLIC_WIRE);
  const legacy=privateFixture();legacy.outer.schema='piv1-au-phase10-authorization-v1';delete legacy.outer.sourcePins['startup_phase10_rpc.py'];
  assert.throws(()=>bridge.run(legacy.input(),legacy.options),/legacy_rpc_binding_forbidden/);assert.equal(legacy.reads.length,0);assert.equal(legacy.signs.length,0);
  const expired=privateFixture(),readEndpoint=expired.options.readRpcCredential;
  expired.options.readRpcCredential=()=>{const value=readEndpoint();expired.setNow(2000000);return value;};
  assert.throws(()=>bridge.run(expired.input(),expired.options),/expired/);assert.equal(expired.reads.length,0);assert.equal(expired.signs.length,0);
  const changed=privateFixture(),read=changed.options.readSecure;
  changed.options.readSecure=(file,bound)=>{const value=read(file,bound);fs.writeFileSync(endpoint,Buffer.from('CHANGED_SYNTHETIC_ENDPOINT\n'));return value;};
  assert.throws(()=>bridge.run(changed.input(),changed.options),/rpc_credential_changed/);assert.equal(changed.reads.length,1);assert.equal(changed.signs.length,0);
 }finally{fs.rmSync(directory,{recursive:true,force:true});}
}));
