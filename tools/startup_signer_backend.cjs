'use strict';
// Future disposable-Testnet backend only. This module never discovers wallets,
// creates keys, sends transactions, or writes keys/paths/partial output.
const fs=require('node:fs'),path=require('node:path'),crypto=require('node:crypto');
const REPO=path.resolve(__dirname,'..');
const GENESIS='4uhcVJyU9pJkvQyS88uRDiswHXSCkY3zQawwpjk2NsNY';
const CHECKER_SHA='50080d7baa17098e4ee687dcf0abb5fd4cef27e88154edd0c32b47e4f033fce9';
const NODE_SHA='bc17c508ffeed0ec622934f9b7fa72f8e78da65350e63c3eceb56fa688aa5e12';
const ORACLE_SHA='a1c17f54f8a5ccd953e8e86c792a784c7ce91884570c7f169618f462f43327e6';
const MAX_INPUT=1048576,MAX_OUTPUT=262144,MAX_PROFILE=32768;
const sha=b=>crypto.createHash('sha256').update(b).digest('hex');
function need(ok,code){if(!ok){const e=new Error(code);e.code=code;throw e;}}
function fields(x,keys){need(x!==null&&typeof x==='object'&&!Array.isArray(x)&&Object.getPrototypeOf(x)===Object.prototype&&Object.keys(x).length===keys.length&&keys.every(k=>Object.hasOwn(x,k)),'fields');}
function same(a,b){return JSON.stringify(a)===JSON.stringify(b);}
function expired(receipt,now){need(BigInt(receipt.expiresUnixTimestamp)>BigInt(Math.floor(now()/1000)),'authorization_expired');}
function identity(st){return [st.dev,st.ino,st.uid,st.mode,st.nlink,st.size,st.mtimeNs,st.ctimeNs].map(String).join(':');}
function safePath(file){
  need(typeof file==='string'&&file.length<=4096&&path.isAbsolute(file)&&path.normalize(file)===file&&!file.includes('\0'),'private_file');
  const pieces=file.split('/').slice(1);need(pieces.length&&pieces.every(x=>x&&x!=='.'&&x!=='..'),'private_file');
  need(file!==REPO&&!file.startsWith(REPO+'/'),'private_file_in_repository');return pieces;
}
function secureRead(file,maximum){
  const pieces=safePath(file),directories=[];let fd;
  try{
    // Linux proc-fd paths keep each lookup anchored to its opened directory.
    // No lstat-then-open parent traversal or symlink component is accepted.
    const flags=fs.constants.O_RDONLY|fs.constants.O_NOFOLLOW|fs.constants.O_DIRECTORY;
    directories.push(fs.openSync('/',flags));
    for(const component of pieces.slice(0,-1)){
      const parent=directories.at(-1),st=fs.fstatSync(parent,{bigint:true});
      need(st.isDirectory()&&(Number(st.mode)&0o022)===0&&(st.uid===0n||st.uid===BigInt(process.getuid())),'private_directory');
      directories.push(fs.openSync('/proc/self/fd/'+parent+'/'+component,flags));
    }
    const parent=directories.at(-1),directory=fs.fstatSync(parent,{bigint:true});
    need(directory.isDirectory()&&(Number(directory.mode)&0o022)===0&&(directory.uid===0n||directory.uid===BigInt(process.getuid())),'private_directory');
    const leaf='/proc/self/fd/'+parent+'/'+pieces.at(-1);
    fd=fs.openSync(leaf,fs.constants.O_RDONLY|fs.constants.O_NOFOLLOW|fs.constants.O_NONBLOCK);
    const before=fs.fstatSync(fd,{bigint:true});
    need(before.isFile()&&before.uid===BigInt(process.getuid())&&before.nlink===1n&&(Number(before.mode)&0o7777)===0o600&&before.size>0n&&before.size<=BigInt(maximum),'private_file');
    const raw=Buffer.alloc(Number(before.size));let count=0;
    while(count<raw.length){const n=fs.readSync(fd,raw,count,raw.length-count,null);need(n>0,'private_file_changed');count+=n;}
    const extra=Buffer.alloc(1);need(fs.readSync(fd,extra,0,1,null)===0,'private_file_changed');
    need(identity(before)===identity(fs.fstatSync(fd,{bigint:true}))&&identity(before)===identity(fs.lstatSync(leaf,{bigint:true})),'private_file_changed');
    return raw;
  }finally{if(fd!==undefined)fs.closeSync(fd);for(const directory of directories.reverse())fs.closeSync(directory);}
}
function sources(receipt){
  need(process.version==='v24.19.0'&&sha(fs.readFileSync(process.execPath))===NODE_SHA,'runtime_pin');
  const checkerPath=path.join(REPO,'clients/cli/startup-public-check.cjs');
  need(!fs.lstatSync(checkerPath).isSymbolicLink()&&sha(fs.readFileSync(checkerPath))===CHECKER_SHA,'checker_pin');
  need(!fs.lstatSync(__filename).isSymbolicLink()&&sha(fs.readFileSync(__filename))===receipt.signerBackendSha256,'backend_pin');
  const operatorPath=path.join(REPO,'tools/startup_release_operator.py');
  need(!fs.lstatSync(operatorPath).isSymbolicLink()&&sha(fs.readFileSync(operatorPath))===receipt.operatorSourceSha256,'operator_pin');
  const checker=require(checkerPath);checker.verifySources();return checker;
}
function approvedSigners(){
  const file=path.join(REPO,'tools/startup_manifest.oracle.json'),raw=fs.readFileSync(file);
  need(!fs.lstatSync(file).isSymbolicLink()&&sha(raw)===ORACLE_SHA,'startup_oracle_pin');
  const roles=JSON.parse(raw.toString('utf8')).roles;
  return ['g0','guardian1','guardian2','guardian3','h2','pivProgram'].map(name=>roles[name]);
}
function loadSigner(raw,address){
  const checker=require(path.join(REPO,'clients/cli/startup-public-check.cjs'));
  let secret;
  try{
    const values=checker.parseInput(raw.toString('utf8'));
    need(Array.isArray(values)&&values.length===64&&values.every(x=>Number.isInteger(x)&&x>=0&&x<=255),'private_key_format');
    secret=Buffer.from(values);values.fill(0);
    const {Keypair}=require(path.join(REPO,'spikes/task-0.4-jito/node_modules/@solana/web3.js/lib/index.cjs.js'));
    const keypair=Keypair.fromSecretKey(secret);
    need(keypair.publicKey.toBase58()===address,'private_key_address');
    const key=crypto.createPrivateKey({key:Buffer.concat([Buffer.from('302e020100300506032b657004220420','hex'),secret.subarray(0,32)]),format:'der',type:'pkcs8'});
    keypair.secretKey.fill(0);return key;
  }finally{raw.fill(0);if(secret)secret.fill(0);}
}
function run(input,options={}){
  // Options are explicit test seams only when imported. The CLI supplies only
  // allowLiveSign; it cannot select a reader, signer, or validation bypass.
  need(options.allowLiveSign===true,'live_sign_not_enabled');
  fields(input,['authorizationRawBase64','authorizationSha256','request','packets','signerProfilePath']);
  need(typeof input.authorizationRawBase64==='string'&&input.authorizationRawBase64.length<=32768,'authorization_bound');
  const receiptRaw=Buffer.from(input.authorizationRawBase64,'base64');
  need(receiptRaw.toString('base64')===input.authorizationRawBase64&&sha(receiptRaw)===input.authorizationSha256,'authorization_hash');
  // Parse the public envelope without importing SDKs or touching private files.
  const receipt=JSON.parse(receiptRaw.toString('utf8'));
  fields(receipt,['schema','manifestSha256','baseParentTail','operatorSourceSha256','expiresUnixTimestamp','scope','feeBudgetNamespace','signerBackendSha256','signerProfileSha256']);
  need(receipt.schema==='piv1-launcher-release-authorization-v2'&&receipt.scope==='EXACT_BOUNDED_TESTNET_RELEASE','signer_authorization_v2_required');
  need(typeof receipt.expiresUnixTimestamp==='string'&&/^(0|[1-9][0-9]{0,19})$/.test(receipt.expiresUnixTimestamp)&&BigInt(receipt.expiresUnixTimestamp)<=(1n<<64n)-1n,'authorization_expiry');
  const now=options.now||Date.now;expired(receipt,now);
  const checker=sources(receipt),canonical=x=>Buffer.from(checker.canonical(x)+'\n');
  need(canonical(receipt).equals(receiptRaw),'authorization_canonical');
  const request=input.request;
  fields(request,['schema','cluster','genesisHash','operatorTail','operatorSourceSha256','authorizationSha256','baseParentPath','baseParentTail','manifestSha256','sealSha256','packets','requestId']);
  const body={...request};delete body.requestId;
  need(request.schema==='piv1-startup-signing-request-v1'&&request.cluster==='testnet'&&request.genesisHash===GENESIS&&sha(canonical(body))===request.requestId,'request_binding');
  need(request.authorizationSha256===input.authorizationSha256&&request.operatorSourceSha256===receipt.operatorSourceSha256&&request.baseParentTail===receipt.baseParentTail&&request.manifestSha256===receipt.manifestSha256,'authorization_request_binding');
  need(typeof receipt.feeBudgetNamespace==='string'&&/^piv1-testnet-startup-release-[a-f0-9]{64}$/.test(receipt.feeBudgetNamespace)&&/^[a-f0-9]{64}$/.test(receipt.signerProfileSha256),'profile_binding');
  need(Array.isArray(input.packets)&&input.packets.length>=1&&input.packets.length<=64&&Array.isArray(request.packets)&&request.packets.length===input.packets.length,'packet_batch');
  const needed=new Set(),ids=new Set();
  const decoded=input.packets.map((p,i)=>{
    fields(p,['id','wireBase64','lifetime','feeQuoteLamports','reviewed']);const binding=request.packets[i];
    fields(binding,['id','preparedRecordDigest','preparedSha256','unsignedWireSha256','messageSha256','requiredSigners']);
    need(p.id===binding.id&&!ids.has(p.id)&&sha(canonical(p))===binding.preparedSha256,'prepared_binding');ids.add(p.id);
    const d=checker.decodeWire(p.wireBase64,p.reviewed.lookupTables);
    need(d.fullyResolved&&d.signatures.every(sig=>sig.every(v=>v===0))&&d.wireSha256===binding.unsignedWireSha256&&d.messageSha256===binding.messageSha256,'unsigned_packet_binding');
    need(d.message.toString('base64')===p.reviewed.messageBase64&&d.messageSha256===p.reviewed.messageSha256&&d.feePayer===p.reviewed.feePayer&&same(d.requiredSigners,p.reviewed.requiredSigners)&&same(d.requiredSigners,binding.requiredSigners)&&checker.canonical(d.roles)===checker.canonical(p.reviewed.accountRoles),'reviewed_packet_binding');
    d.requiredSigners.forEach(a=>needed.add(a));return d;
  });
  // Every message/role/signature slot is checked before the first private read.
  expired(receipt,now);sources(receipt);
  const read=options.readSecure||secureRead,parseKey=options.loadSigner||loadSigner,sign=options.signMessage||((message,key)=>crypto.sign(null,message,key));
  const profileRaw=read(input.signerProfilePath,MAX_PROFILE);
  need(Buffer.isBuffer(profileRaw)&&sha(profileRaw)===receipt.signerProfileSha256,'profile_hash');
  const profile=checker.parseInput(profileRaw.toString('utf8'));
  fields(profile,['schema','cluster','genesisHash','disposableTestnetOnly','signers']);
  need(profile.schema==='piv1-disposable-testnet-signer-profile-v1'&&profile.cluster==='testnet'&&profile.genesisHash===GENESIS&&profile.disposableTestnetOnly===true,'testnet_profile');
  need(Array.isArray(profile.signers)&&profile.signers.length>=1&&profile.signers.length<=16,'profile_signers');
  const configured=new Map(),allowed=approvedSigners();
  for(const row of profile.signers){fields(row,['address','secretKeyPath']);need(!configured.has(row.address)&&allowed.includes(row.address),'profile_signer_subset');safePath(row.secretKeyPath);configured.set(row.address,row.secretKeyPath);}
  need([...needed].every(a=>configured.has(a))&&(configured.size===needed.size||configured.size===allowed.length&&allowed.every(a=>configured.has(a))),'profile_required_signer');
  expired(receipt,now);sources(receipt);
  const keys=new Map();
  for(const address of needed){expired(receipt,now);keys.set(address,parseKey(read(configured.get(address),4096),address));}
  const packets=input.packets.map((p,i)=>{
    const d=decoded[i],signed=Buffer.from(d.wire);
    d.requiredSigners.forEach((address,index)=>{
      expired(receipt,now);const sig=sign(d.message,keys.get(address),address,p.id,index);
      need(Buffer.isBuffer(sig)&&sig.length===64,'signature_output');sig.copy(signed,1+index*64);
    });
    const wireBase64=signed.toString('base64');checker.verifyPacket({signedWireBase64:wireBase64,reviewed:p.reviewed});
    return {id:p.id,wireBase64};
  });
  expired(receipt,now);sources(receipt);
  need(sha(read(input.signerProfilePath,MAX_PROFILE))===receipt.signerProfileSha256,'profile_changed');
  expired(receipt,now);
  const response={schema:'piv1-startup-signing-response-v1',requestId:request.requestId,packets};
  need(Buffer.byteLength(JSON.stringify(response))<=MAX_OUTPUT,'output_bound');return response;
}
function readInput(){
  const chunks=[];let size=0;
  while(true){const chunk=Buffer.alloc(65536),count=fs.readSync(0,chunk,0,chunk.length,null);if(!count)break;size+=count;need(size<=MAX_INPUT,'input_bound');chunks.push(chunk.subarray(0,count));}
  return JSON.parse(Buffer.concat(chunks).toString('utf8'));
}
if(require.main===module){
  try{need(process.argv.length===3&&process.argv[2]==='--allow-live-sign','live_sign_not_enabled');const result=run(readInput(),{allowLiveSign:true});process.stdout.write(JSON.stringify(result)+'\n');}
  catch{process.stderr.write('{"status":"STOP","reason":"signer_backend_refused"}\n');process.exitCode=1;}
}
module.exports={run,secureRead,MAX_INPUT,MAX_OUTPUT};
