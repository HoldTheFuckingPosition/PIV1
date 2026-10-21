'use strict';
// Public bytes and declarations only. This module has no wallet, RPC or signing API.
const assert=require('node:assert/strict'),fs=require('node:fs'),path=require('node:path');
const {createHash}=require('node:crypto');
const base=require('./initializer-plan.cjs'),t=require('../../validation/genesis-transport/transport.cjs');
const {PublicKey,TransactionInstruction,ComputeBudgetProgram}=require(t.SDK);
const SCHEMA='piv1-loader-upload-resume-plan-v1',BASELINE='ed3d3329105c53cdef72f93a23cbfc9feeadeb10';
const SOURCE_PINS=Object.freeze({
  'clients/cli/initializer-plan.cjs':'347d5ab436b3e8878fd337c53ef8636721ca64a6465bc86c06f2637921ac6f6f',
  'validation/genesis-transport/transport.cjs':'a7694ff07577988e96546a53008bc8ab85bbc0ae7cb9353dcfc25e6c5a3c3724',
  '/home/jerem/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/solana-loader-v3-interface-3.0.0/src/instruction.rs':'6c67d3da13418180fabbeaa3b98de4041218b2d9453f8dd9af9fb16b08847571',
  '/home/jerem/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/solana-loader-v3-interface-3.0.0/src/state.rs':'c27fd423293630454cf79de6d28e1fbadee092fd83c637104a6d8fa9142770a5'
});
const PROFILE=Object.freeze({cluster:'testnet',genesisHash:'4uhcVJyU9pJkvQyS88uRDiswHXSCkY3zQawwpjk2NsNY',
  programId:'GAjHRaC3XM98SoRoFy5B8P2ypTS1ndf1UPhXmjWgtPux',programDataAddress:'6HV5GQesbZpTyBVTiMu64UU4JgCX2D2s489sKt26xpd8',
  writer:'BUuuNvicRKF4ZFEUH4iS8xGzhAKQca3ou4U1s4NaBYoT',bufferAddress:'HGP9XhQmoHGCUbgoSATZ19zZHonxWB3uo1hKF3SK3M6K',
  bufferSeed:'piv1-testnet-e29735e1',loader:'BPFLoaderUpgradeab1e11111111111111111111111',
  artifactBytes:894552,artifactSha256:'e29735e1467da98ecddb1eea9fd25abaffa4a450bd899ddc137f7d8c0da35b99',bufferHeaderBytes:37,chunkBytes:956});
// Deliberate retained requests from the startup sizing evidence; not measured usage.
const RESOURCES=Object.freeze({computeUnits:1400000,heapBytes:262144,loadedAccountBytes:67108864,measured:false});
const TEMPLATE={schema:SCHEMA,cluster:'testnet',genesisHash:null,programId:null,programDataAddress:null,writer:null,feePayer:null,bufferAddress:null,bufferSeed:null,
  artifact:{bytes:null,sha256:null,dataBase64:null},
  snapshot:{contextSlot:null,buffer:{exists:null,address:null,owner:null,executable:null,lamports:null,rentExemptMinimumLamports:null,dataBase64:null},
    program:{address:null,exists:null,contextSlot:null},programData:{address:null,exists:null,contextSlot:null}},
  transport:{recentBlockhash:null,currentBlockHeight:null,lastValidBlockHeight:null,minimumRemainingBlockHeights:null},batch:{maxPackets:null}};
const MAX_MANIFEST_BYTES=3*1024*1024,U64=(1n<<64n)-1n;
const sha=b=>createHash('sha256').update(b).digest('hex'),pk=s=>new PublicKey(s);
const plain=x=>x!==null&&typeof x==='object'&&!Array.isArray(x)&&Object.getPrototypeOf(x)===Object.prototype;
const fail=(code,field)=>{const e=new Error(code);e.code=code;e.field=field;throw e;};
function parseManifest(source){
  if(typeof source!=='string'||Buffer.byteLength(source)>MAX_MANIFEST_BYTES)fail('manifest_size','manifest');
  let result;try{result=JSON.parse(source);}catch{fail('invalid_json','manifest');}
  // Lexical duplicate-key rejection also catches differently escaped names. No recursion.
  const stack=[];
  for(let i=0;i<source.length;i++){
    const token=source[i];
    if(token==='{'||token==='['){stack.push({object:token==='{',key:true,seen:new Set()});if(stack.length>8)fail('manifest_depth','manifest');}
    else if(token==='}'||token===']')stack.pop();
    else if(token===','){if(stack.at(-1)?.object)stack.at(-1).key=true;}
    else if(token==='"'){
      const start=i++;while(i<source.length&&source[i]!=='"'){if(source[i]==='\\')i++;i++;}
      if(stack.at(-1)?.object&&stack.at(-1).key){const s=stack.at(-1),key=JSON.parse(source.slice(start,i+1));if(s.seen.has(key))fail('duplicate_json_key','manifest');s.seen.add(key);s.key=false;}
    }
  }
  return result;
}
function key(value,field){if(typeof value!=='string'||value.length>44)fail('public_key_string',field);let k;try{k=pk(value);}catch{fail('invalid_public_key',field);}if(k.toBase58()!==value)fail('noncanonical_public_key',field);return k;}
function decimal(value,field){if(typeof value!=='string'||value.length>20||!/^(?:0|[1-9]\d*)$/.test(value))fail('decimal_string_required',field);if(BigInt(value)>U64)fail('integer_range',field);}
function bytes(value,size,field){
  if(typeof value!=='string'||value.length!==4*Math.ceil(size/3)||!(/^[A-Za-z0-9+/]*={0,2}$/.test(value)))fail('canonical_base64_required',field);
  const b=Buffer.from(value,'base64');if(b.length!==size||b.toString('base64')!==value)fail('canonical_base64_required',field);return b;
}
function validateManifest(input){
  const missing=[];
  function walk(value,shape,field){
    if(value===null||value===undefined){missing.push(field);return null;}
    if(plain(shape)){if(!plain(value))fail('object_required',field||'manifest');if(Object.keys(value).some(k=>!Object.hasOwn(shape,k)))fail('unknown_field',field||'manifest');return Object.fromEntries(Object.entries(shape).map(([k,s])=>[k,walk(value[k],s,field?`${field}.${k}`:k)]));}
    return value;
  }
  if(!plain(input))fail('object_required','manifest');const m=walk(input,TEMPLATE,''),a=m.artifact,s=m.snapshot,c=m.transport;
  const check=(v,f,fn)=>{if(v!==null&&v!==undefined)fn(v,f);};
  check(m.schema,'schema',(v,f)=>{if(v!==SCHEMA)fail('unsupported_schema',f);});
  for(const f of ['cluster','genesisHash','programId','programDataAddress','writer','bufferAddress','bufferSeed'])check(m[f],f,(v,k)=>{if(v!==PROFILE[k])fail('pinned_profile_mismatch',k);});
  check(m.feePayer,'feePayer',(v,f)=>{if(v!==PROFILE.writer)fail('pinned_profile_mismatch',f);});
  if(a){check(a.bytes,'artifact.bytes',(v,f)=>{if(v!==PROFILE.artifactBytes)fail('artifact_size',f);});check(a.sha256,'artifact.sha256',(v,f)=>{if(v!==PROFILE.artifactSha256)fail('artifact_hash',f);});}
  const artifact=a?.dataBase64===null||!a?null:bytes(a.dataBase64,PROFILE.artifactBytes,'artifact.dataBase64');
  if(artifact&&sha(artifact)!==PROFILE.artifactSha256)fail('artifact_hash','artifact.dataBase64');
  let buffer=null;
  if(s){
    check(s.contextSlot,'snapshot.contextSlot',decimal);
    if(s.buffer){const b=s.buffer;
      check(b.exists,'snapshot.buffer.exists',(v,f)=>{if(v!==true)fail('buffer_absent_or_closed',f);});
      for(const [f,want]of [['address',PROFILE.bufferAddress],['owner',PROFILE.loader],['executable',false]])check(b[f],`snapshot.buffer.${f}`,(v,k)=>{if(v!==want)fail('buffer_precondition',k);});
      for(const f of ['lamports','rentExemptMinimumLamports'])check(b[f],`snapshot.buffer.${f}`,decimal);
      if(b.rentExemptMinimumLamports!==null&&BigInt(b.rentExemptMinimumLamports)===0n)fail('positive_declared_rent_required','snapshot.buffer.rentExemptMinimumLamports');
      if(b.lamports!==null&&(BigInt(b.lamports)===0n||(b.rentExemptMinimumLamports!==null&&BigInt(b.lamports)<BigInt(b.rentExemptMinimumLamports))))fail('declared_buffer_rent_shortfall','snapshot.buffer.lamports');
      if(b.dataBase64!==null){buffer=bytes(b.dataBase64,PROFILE.artifactBytes+37,'snapshot.buffer.dataBase64');
        if(buffer.readUInt32LE(0)!==1||buffer[4]!==1)fail('buffer_header','snapshot.buffer.dataBase64');
        if(!buffer.subarray(5,37).equals(pk(PROFILE.writer).toBuffer()))fail('buffer_authority','snapshot.buffer.dataBase64');}
    }
    for(const [role,address]of [['program',PROFILE.programId],['programData',PROFILE.programDataAddress]])if(s[role]){
      const v=s[role];check(v.address,`snapshot.${role}.address`,(x,f)=>{if(x!==address)fail('pinned_profile_mismatch',f);});
      check(v.exists,`snapshot.${role}.exists`,(x,f)=>{if(x!==false)fail('program_or_programdata_present',f);});
      check(v.contextSlot,`snapshot.${role}.contextSlot`,decimal);
      if(v.contextSlot!==null&&s.contextSlot!==null&&v.contextSlot!==s.contextSlot)fail('snapshot_context_mismatch',`snapshot.${role}.contextSlot`);
    }
  }
  if(c){check(c.recentBlockhash,'transport.recentBlockhash',key);for(const f of ['currentBlockHeight','lastValidBlockHeight','minimumRemainingBlockHeights'])check(c[f],`transport.${f}`,decimal);
    if(c.minimumRemainingBlockHeights!==null&&BigInt(c.minimumRemainingBlockHeights)===0n)fail('positive_margin_required','transport.minimumRemainingBlockHeights');
    if(c.currentBlockHeight!==null&&c.lastValidBlockHeight!==null){const remaining=BigInt(c.lastValidBlockHeight)-BigInt(c.currentBlockHeight);
      if(remaining<0n)fail('declared_blockhash_expired','transport.lastValidBlockHeight');
      if(c.minimumRemainingBlockHeights!==null&&remaining<BigInt(c.minimumRemainingBlockHeights))fail('insufficient_declared_margin','transport.minimumRemainingBlockHeights');}
  }
  if(m.batch)check(m.batch.maxPackets,'batch.maxPackets',(v,f)=>{if(!Number.isSafeInteger(v)||v<1||v>64)fail('batch_limit',f);});
  // All chunks are inspected before any message is assembled, including chunks beyond this batch.
  const chunks=[];
  if(artifact&&buffer)for(let index=0,offset=0;offset<artifact.length;index++,offset+=956){
    const target=artifact.subarray(offset,Math.min(offset+956,artifact.length)),supplied=buffer.subarray(37+offset,37+offset+target.length);
    const status=supplied.equals(target)?'MATCHED_BYTES':supplied.every(x=>x===0)?'ZERO_PENDING':'CONFLICT';
    if(status==='CONFLICT')fail('conflicting_buffer_chunk',`snapshot.buffer.chunk[${index}]`);
    chunks.push({index,offset,bytes:target.length,status,targetSha256:sha(target),suppliedSha256:sha(supplied)});
  }
  return {manifest:m,missing,artifact,buffer,chunks};
}
function writeInstructions(offset,payload){
  const header=Buffer.alloc(16);header.writeUInt32LE(1,0);header.writeUInt32LE(offset,4);header.writeBigUInt64LE(BigInt(payload.length),8);
  const loaded=Buffer.alloc(5);loaded[0]=4;loaded.writeUInt32LE(RESOURCES.loadedAccountBytes,1);
  return [ComputeBudgetProgram.setComputeUnitLimit({units:RESOURCES.computeUnits}),ComputeBudgetProgram.requestHeapFrame({bytes:RESOURCES.heapBytes}),
    new TransactionInstruction({programId:ComputeBudgetProgram.programId,keys:[],data:loaded}),
    new TransactionInstruction({programId:pk(PROFILE.loader),keys:[{pubkey:pk(PROFILE.bufferAddress),isSigner:false,isWritable:true},{pubkey:pk(PROFILE.writer),isSigner:true,isWritable:false}],data:Buffer.concat([header,payload])})];
}
function planManifest(input){
  const {manifest:m,missing,artifact,buffer,chunks}=validateManifest(input);
  const common={schema:SCHEMA,sourceBaseline:BASELINE,missing,profile:PROFILE,requestedResources:RESOURCES,
    readiness:{operational:false,currentStateVerified:false,snapshotAuthenticated:false,blockhashVerified:false,authorized:false,publicReady:false,
      artifactDeploymentSelected:false,bootstrapTrustSelected:false,signerControlVerified:false,fundingSufficient:false,throughputGuaranteed:false,m4Reservation:'OPEN_UNWAIVED'},
    limitations:['Supplied public snapshots and lifetime declarations are unauthenticated and are never refreshed here.',
      'MATCHED_BYTES means byte equality only, including never-written all-zero target chunks; no prior transaction or signature is proved.',
      'Remaining block heights are exact integer differences, not a batch throughput or whole upload lifetime guarantee.',
      'Replan from independently refreshed complete snapshots and a separately supplied lifetime before each future batch; never infer progress from prior planned packets.',
      'No create, deploy, authority, bootstrap, Clock anchor, fee quote, funding or readiness action is included.']};
  if(missing.length)return {...common,status:'INCOMPLETE',plan:null};
  const pending=chunks.filter(c=>c.status==='ZERO_PENDING'),selected=pending.slice(0,m.batch.maxPackets),selectedIndices=selected.map(c=>c.index),remaining=pending.slice(selected.length).map(c=>c.index);
  const context={m:{feePayer:m.feePayer,transport:m.transport}},packets=selected.map(chunk=>{
    const instructions=writeInstructions(chunk.offset,artifact.subarray(chunk.offset,chunk.offset+chunk.bytes)),p=base.packet(context,`loader_write_${chunk.index}`,instructions),message=base.compile(context,instructions),messageBytes=Buffer.from(message.serialize());
    assert.equal(p.wireBytes,chunk.bytes+276);assert.equal(p.signatureCount,1);assert.deepEqual(p.requiredSigners,[PROFILE.writer]);
    return {...p,chunkIndex:chunk.index,offset:chunk.offset,payloadBytes:chunk.bytes,messageBytes:messageBytes.length,messageBase64:messageBytes.toString('base64'),messageSha256:sha(messageBytes),
      signatureRoles:[{address:PROFILE.writer,roles:['fee_payer','buffer_writer'],signaturePresent:false}],globalRoles:message.accountKeys.map((k,i)=>({address:k.toBase58(),signer:message.isAccountSigner(i),writable:message.isAccountWritable(i)}))};
  });
  return {...common,status:'OFFLINE_PLAN_ONLY',declarationSha256:sha(Buffer.from(JSON.stringify(m))),plan:{snapshotContextSlot:m.snapshot.contextSlot,suppliedBufferSha256:sha(buffer),
    suppliedArtifactSha256:sha(artifact),transport:{...m.transport,remainingBlockHeights:(BigInt(m.transport.lastValidBlockHeight)-BigInt(m.transport.currentBlockHeight)).toString(),verified:false},
    chunkCount:chunks.length,chunks,progress:{matchedChunks:chunks.length-pending.length,pendingChunks:pending.length,matchedBytes:chunks.filter(c=>c.status==='MATCHED_BYTES').reduce((n,c)=>n+c.bytes,0),pendingBytes:pending.reduce((n,c)=>n+c.bytes,0),confirmedTransactions:null},
    selectedIndices,remainingIndices:remaining,pendingIndices:pending.map(c=>c.index),allBytesMatchSuppliedArtifact:pending.length===0,
    hypotheticalMatchedChunksAfterBatch:chunks.length-pending.length+selected.length,nextStateVerified:false,packets}};
}
function verifySources(){
  const inherited=base.verifySources();for(const [file,want]of Object.entries(SOURCE_PINS))assert.equal(sha(fs.readFileSync(path.isAbsolute(file)?file:path.join(t.REPO,file))),want,`source binding: ${file}`);
  const seeded=pk(createHash('sha256').update(Buffer.concat([pk(PROFILE.writer).toBuffer(),Buffer.from(PROFILE.bufferSeed),pk(PROFILE.loader).toBuffer()])).digest()).toBase58();assert.equal(seeded,PROFILE.bufferAddress);
  assert.equal(PublicKey.findProgramAddressSync([pk(PROFILE.programId).toBuffer()],pk(PROFILE.loader))[0].toBase58(),PROFILE.programDataAddress);
  return {node:inherited.node,web3:inherited.web3,inheritedPins:inherited.pins,pins:SOURCE_PINS,loaderInterface:'solana-loader-v3-interface3.0.0'};
}
function main(args){
  const sourceCheck=verifySources();if(args.length===1&&args[0]==='--template'){process.stdout.write(JSON.stringify(TEMPLATE,null,2)+'\n');return;}
  if(args.length!==2||args[0]!=='--manifest')fail('unsupported_arguments','arguments');
  const file=path.resolve(args[1]),stat=fs.statSync(file);if(!stat.isFile()||stat.size>MAX_MANIFEST_BYTES)fail('public_manifest_size','manifest');const raw=fs.readFileSync(file);if(raw.length>MAX_MANIFEST_BYTES)fail('manifest_size','manifest');
  const report=planManifest(parseManifest(raw.toString('utf8')));report.manifestInputSha256=sha(raw);report.sourceCheck=sourceCheck;process.stdout.write(JSON.stringify(report,null,2)+'\n');
}
module.exports={SCHEMA,BASELINE,SOURCE_PINS,PROFILE,RESOURCES,TEMPLATE,MAX_MANIFEST_BYTES,parseManifest,validateManifest,planManifest,writeInstructions,verifySources};
if(require.main===module){try{main(process.argv.slice(2));}catch(e){process.stderr.write(JSON.stringify({status:'INVALID_OR_UNSUPPORTED_UPLOAD_RESUME_PLAN',code:e.code||'plan_validation_failed',field:e.field||'manifest'})+'\n');process.exitCode=1;}}
