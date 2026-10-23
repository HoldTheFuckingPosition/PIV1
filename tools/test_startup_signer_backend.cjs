'use strict';
// Root-executed fake-only backend tests. Native signing, key derivation and
// private-file opens are trapped; only retained public signatures are returned.
const test=require('node:test'),assert=require('node:assert/strict');
const fs=require('node:fs'),path=require('node:path'),crypto=require('node:crypto');
const REPO=path.resolve(__dirname,'..');
const checker=require(path.join(REPO,'clients/cli/startup-public-check.cjs'));
const backend=require('./startup_signer_backend.cjs');
const sdk=require(path.join(REPO,'spikes/task-0.4-jito/node_modules/@solana/web3.js/lib/index.cjs.js'));
const sha=b=>crypto.createHash('sha256').update(b).digest('hex');
const canonical=x=>Buffer.from(checker.canonical(x)+'\n');
const fixtureRaw=fs.readFileSync(path.join(REPO,'clients/cli/startup-public-check.fixture.json'));
assert.equal(sha(fixtureRaw),'0855cd85667a565f4b6184ef2b389c2e4ec4421582f2b52f58b2cf8897852c0d');
const tx=JSON.parse(fixtureRaw).transactions[1],signed=checker.decodeWire(tx.signedWireBase64,[]);
const oracle=JSON.parse(fs.readFileSync(path.join(REPO,'tools/startup_manifest.oracle.json')));
const allowed=['g0','guardian1','guardian2','guardian3','h2','pivProgram'].map(name=>oracle.roles[name]);
const GENESIS='4uhcVJyU9pJkvQyS88uRDiswHXSCkY3zQawwpjk2NsNY';
const counters={schema:'piv1-offline-signer-backend-test-counters-v1',fakeGroups:0,positiveFakeReads:0,positiveFakeKeyLoads:0,positiveFakeSignatures:0,nativeSigningCalls:0,nativeKeyDerivations:0,nativeKeyGenerations:0,nativePrivateKeyImports:0,privateFileOpens:0};
function inputFor({full=false}={}){
  const addresses=full?allowed:signed.requiredSigners;
  const profile={schema:'piv1-disposable-testnet-signer-profile-v1',cluster:'testnet',genesisHash:GENESIS,disposableTestnetOnly:true,
    signers:addresses.map((address,i)=>({address,secretKeyPath:'/not-opened-private/signer-'+i+'.json'}))};
  const profileRaw=canonical(profile),unsigned=Buffer.from(signed.wire);unsigned.fill(0,1,1+64*signed.signatures.length);
  const reviewed={messageBase64:signed.message.toString('base64'),messageSha256:signed.messageSha256,feePayer:signed.feePayer,requiredSigners:signed.requiredSigners,accountRoles:signed.roles,lookupTables:[]};
  const packet={id:'historical_public_packet',wireBase64:unsigned.toString('base64'),lifetime:{recentBlockhash:signed.recentBlockhash,currentBlockHeight:'100',lastValidBlockHeight:'1000',minimumRemainingBlockHeights:'40'},feeQuoteLamports:tx.feeLamports,reviewed};
  const receipt={schema:'piv1-launcher-release-authorization-v2',manifestSha256:'c'.repeat(64),baseParentTail:'b'.repeat(64),operatorSourceSha256:sha(fs.readFileSync(path.join(REPO,'tools/startup_release_operator.py'))),expiresUnixTimestamp:'18446744073709551615',scope:'EXACT_BOUNDED_TESTNET_RELEASE',feeBudgetNamespace:'piv1-testnet-startup-release-'+'d'.repeat(64),signerBackendSha256:sha(fs.readFileSync(path.join(REPO,'tools/startup_signer_backend.cjs'))),signerProfileSha256:sha(profileRaw)};
  const authorizationRaw=canonical(receipt),authorizationSha256=sha(authorizationRaw);
  const body={schema:'piv1-startup-signing-request-v1',cluster:'testnet',genesisHash:GENESIS,operatorTail:'a'.repeat(64),operatorSourceSha256:receipt.operatorSourceSha256,authorizationSha256,baseParentPath:'/public-parent-only',baseParentTail:receipt.baseParentTail,manifestSha256:receipt.manifestSha256,sealSha256:null,
    packets:[{id:packet.id,preparedRecordDigest:'e'.repeat(64),preparedSha256:sha(canonical(packet)),unsignedWireSha256:sha(unsigned),messageSha256:signed.messageSha256,requiredSigners:signed.requiredSigners}]};
  const request={...body,requestId:sha(canonical(body))};
  return {input:{authorizationRawBase64:authorizationRaw.toString('base64'),authorizationSha256,request,packets:[packet],signerProfilePath:'/not-opened-private/profile.json'},profile,profileRaw,receipt};
}
function repin(x){
  x.profileRaw=canonical(x.profile);x.receipt.signerProfileSha256=sha(x.profileRaw);
  const raw=canonical(x.receipt);x.input.authorizationRawBase64=raw.toString('base64');x.input.authorizationSha256=sha(raw);
  x.input.request.authorizationSha256=sha(raw);x.input.request.operatorSourceSha256=x.receipt.operatorSourceSha256;
  const body={...x.input.request};delete body.requestId;x.input.request.requestId=sha(canonical(body));
}
function fakeOptions(x,overrides={}){
  const calls={reads:[],loads:[],signs:[],nativeSigns:0,keyDerivations:0,privateOpens:0};
  const options={allowLiveSign:true,now:()=>1000,
    readSecure(file){calls.reads.push(file);if(file===x.input.signerProfilePath)return Buffer.from(x.profileRaw);assert(x.profile.signers.some(r=>r.secretKeyPath===file));return Buffer.from('PUBLIC_FIXTURE_PLACEHOLDER');},
    loadSigner(raw,address){assert.equal(raw.toString(),'PUBLIC_FIXTURE_PLACEHOLDER');calls.loads.push(address);return {address};},
    signMessage(message,key,address,id,index){assert(message.equals(signed.message));assert.equal(key.address,address);assert.equal(id,'historical_public_packet');assert.equal(address,signed.requiredSigners[index]);calls.signs.push(address);return Buffer.from(signed.signatures[index]);},...overrides};
  return {calls,options};
}
function trapped(fn){
  const originalSign=crypto.sign,originalKey=sdk.Keypair.fromSecretKey,originalOpen=fs.openSync,originalPrivateKey=crypto.createPrivateKey;
  const generationNames=['generateKey','generateKeySync','generateKeyPair','generateKeyPairSync'],generationOriginals=generationNames.map(name=>crypto[name]),originalGenerate=sdk.Keypair.generate;
  let forbidden=0;
  crypto.sign=()=>{forbidden++;counters.nativeSigningCalls++;throw Error('native signing forbidden');};
  sdk.Keypair.fromSecretKey=()=>{forbidden++;counters.nativeKeyDerivations++;throw Error('private derivation forbidden');};
  crypto.createPrivateKey=()=>{forbidden++;counters.nativePrivateKeyImports++;throw Error('private import forbidden');};
  for(const name of generationNames)crypto[name]=()=>{forbidden++;counters.nativeKeyGenerations++;throw Error('key generation forbidden');};
  sdk.Keypair.generate=()=>{forbidden++;counters.nativeKeyGenerations++;throw Error('key generation forbidden');};
  fs.openSync=function(file,...args){if(typeof file==='string'&&(file==='/'||file.startsWith('/proc/self/fd/')||file.startsWith('/not-opened-private/'))){forbidden++;counters.privateFileOpens++;throw Error('private open forbidden');}return originalOpen.call(this,file,...args);};
  try{fn();assert.equal(forbidden,0);counters.fakeGroups++;}finally{crypto.sign=originalSign;sdk.Keypair.fromSecretKey=originalKey;fs.openSync=originalOpen;sdk.Keypair.generate=originalGenerate;crypto.createPrivateKey=originalPrivateKey;generationNames.forEach((name,i)=>crypto[name]=generationOriginals[i]);}
}
test('exact historical output through read/key/sign fakes, unused approved keys untouched',()=>trapped(()=>{
  const x=inputFor({full:true}),f=fakeOptions(x),response=backend.run(x.input,f.options);
  assert.equal(response.packets[0].wireBase64,tx.signedWireBase64);
  assert.equal(response.requestId,x.input.request.requestId);
  assert.deepEqual(f.calls.loads,signed.requiredSigners);assert.deepEqual(f.calls.signs,signed.requiredSigners);
  const readKeys=f.calls.reads.filter(file=>file!==x.input.signerProfilePath);
  assert.deepEqual(readKeys,signed.requiredSigners.map(a=>x.profile.signers.find(r=>r.address===a).secretKeyPath));
  counters.positiveFakeReads=f.calls.reads.length;counters.positiveFakeKeyLoads=f.calls.loads.length;counters.positiveFakeSignatures=f.calls.signs.length;
}));
test.after(()=>process.stdout.write('# '+JSON.stringify(counters)+'\n'));
test('activation, receipt, source, expiry and unsigned-message refusals precede private access',()=>trapped(()=>{
  const mutations=[x=>{},x=>{x.receipt.schema='piv1-launcher-release-authorization-v1';repin(x);},x=>{x.receipt.expiresUnixTimestamp='0';repin(x);},x=>{x.receipt.signerBackendSha256='0'.repeat(64);repin(x);},x=>{x.receipt.operatorSourceSha256='0'.repeat(64);repin(x);},x=>{x.input.request.requestId='0'.repeat(64);},x=>{x.input.packets[0].wireBase64=tx.signedWireBase64;repin(x);}];
  mutations.forEach((mutate,index)=>{const x=inputFor();mutate(x);const f=fakeOptions(x,index===0?{allowLiveSign:false}:{});assert.throws(()=>backend.run(x.input,f.options));assert.deepEqual(f.calls.reads,[]);assert.deepEqual(f.calls.signs,[]);});
}));
test('profile pin, Testnet marker and unrelated signers refuse before key access',()=>trapped(()=>{
  for(const change of [x=>{x.profileRaw=Buffer.from('{}');},x=>{x.profile.cluster='mainnet';repin(x);},x=>{x.profile.disposableTestnetOnly=false;repin(x);},x=>{x.profile.signers[0].address=oracle.roles.guardian4;repin(x);},x=>{x.profile.signers=[];repin(x);}]){
    const x=inputFor();change(x);const f=fakeOptions(x);assert.throws(()=>backend.run(x.input,f.options));assert.deepEqual(f.calls.loads,[]);assert.deepEqual(f.calls.signs,[]);
  }
  const x=inputFor({full:true});x.profile.signers.find(row=>!signed.requiredSigners.includes(row.address)).secretKeyPath=null;repin(x);
  const f=fakeOptions(x);assert.throws(()=>backend.run(x.input,f.options),/private_file/);assert.deepEqual(f.calls.loads,[]);
}));
test('every ordered signature is verified; expiry before key use and profile changes stop output',()=>trapped(()=>{
  const x=inputFor(),f=fakeOptions(x,{signMessage(message,key,address,id,index){const sig=Buffer.from(signed.signatures[index]);if(index===1)sig[0]^=1;return sig;}});
  assert.throws(()=>backend.run(x.input,f.options),/signature_invalid/);
  const y=inputFor(),g=fakeOptions(y);let profileReads=0;
  g.options.readSecure=file=>{g.calls.reads.push(file);if(file===y.input.signerProfilePath)return ++profileReads===1?Buffer.from(y.profileRaw):Buffer.from('{}');return Buffer.from('PUBLIC_FIXTURE_PLACEHOLDER');};
  assert.throws(()=>backend.run(y.input,g.options),/profile_changed/);
  const z=inputFor(),k=fakeOptions(z);let expired=false;
  k.options.now=()=>expired?Number(18446744073709551615n)*1000:1000;
  k.options.readSecure=file=>{k.calls.reads.push(file);assert.equal(file,z.input.signerProfilePath);expired=true;return Buffer.from(z.profileRaw);};
  assert.throws(()=>backend.run(z.input,k.options),/authorization_expired/);assert.deepEqual(k.calls.loads,[]);
}));
