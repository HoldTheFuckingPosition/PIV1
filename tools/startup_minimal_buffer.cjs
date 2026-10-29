'use strict';
// Public-only, operation-owned SDK worker. No keys, signer, network or send API.
const fs=require('node:fs'),path=require('node:path'),crypto=require('node:crypto');
const REPO=path.resolve(__dirname,'..');
const sha=b=>crypto.createHash('sha256').update(b).digest('hex');
const need=(ok,code)=>{if(!ok)throw Error(code);};
const NODE_SHA='bc17c508ffeed0ec622934f9b7fa72f8e78da65350e63c3eceb56fa688aa5e12';
const CHECKER_SHA='50080d7baa17098e4ee687dcf0abb5fd4cef27e88154edd0c32b47e4f033fce9';
const ORACLE_SHA='e3ca98bbc901dce6da6d1894c25a34b2649cc0d9bd19c226a11cb1ed10c3b868';
const SDK_PACKAGE_SHA='33c76444048957077187a5ef01365a421f4680a1bde95aae2366de8105d4534f';
const CHECKER=path.join(REPO,'clients/cli/startup-public-check.cjs');
const SDK=path.join(REPO,'spikes/task-0.4-jito/node_modules/@solana/web3.js');
const MAX_INPUT=24*1024*1024,MAX_OUTPUT=512*1024;
function exact(a,b){need(JSON.stringify(a)===JSON.stringify(b),'sdk_binding');}
class BufferSdk {
  constructor(){
    this.own=fs.readFileSync(__filename);this.ready=false;this.templates=[];this.bound=new Map();
    this.guard();this.checker=require(CHECKER);this.checker.verifySources();
    this.sdk=require(SDK);need(require(path.join(SDK,'package.json')).version==='1.98.4','sdk_version');
    const raw=fs.readFileSync(path.join(REPO,'tools/startup_manifest.candidate-oracle.json'));
    need(sha(raw)===ORACLE_SHA,'sdk_oracle_pin');this.roles=JSON.parse(raw).roles;
  }
  guard(){
    for(const [file,want]of [[process.execPath,NODE_SHA],[CHECKER,CHECKER_SHA],
      [path.join(SDK,'package.json'),SDK_PACKAGE_SHA]]){
      need(!fs.lstatSync(file).isSymbolicLink()&&sha(fs.readFileSync(file))===want,'sdk_source_pin');
    }
    need(process.version==='v24.19.0'&&!fs.lstatSync(__filename).isSymbolicLink()
      &&fs.readFileSync(__filename).equals(this.own),'sdk_worker_changed');
    if(this.checker)this.checker.verifySources();
  }
  decoded(wire){return this.checker.decodeWire(wire,[]);}
  unsigned(message){return this.sdk.Transaction.populate(message).serialize({requireAllSignatures:false,verifySignatures:false});}
  describe(wire){const d=this.decoded(wire);return {messageBase64:d.message.toString('base64'),messageSha256:d.messageSha256,
    feePayer:d.feePayer,requiredSigners:d.requiredSigners,accountRoles:d.roles,lookupTables:[]};}
  run(input){
    this.guard();need(input&&typeof input==='object'&&!Array.isArray(input),'sdk_request');let result;
    if(input.action==='PREPARE'){
      need(!this.ready&&Object.keys(input).sort().join(',')==='action,reviews,workerSha256'
        &&sha(this.own)===input.workerSha256&&Array.isArray(input.reviews)&&input.reviews.length===2,'sdk_prepare_once');
      this.templates=input.reviews.map((review,index)=>{
        const raw=Buffer.from(review.messageBase64,'base64'),message=this.sdk.Message.from(raw);
        need(message.version==='legacy'&&message.serialize().equals(raw),'sdk_legacy_exact');
        const wire=this.unsigned(message).toString('base64'),d=this.decoded(wire);
        need(d.signatures.every(s=>s.every(b=>b===0))&&d.fullyResolved,'sdk_unsigned');
        exact(d.requiredSigners,index===0?[this.roles.g0,this.roles.h2]:[this.roles.g0]);
        exact(this.checker.canonical(this.describe(wire)),this.checker.canonical(review));
        return Buffer.from(raw);
      });
      this.ready=true;result={sdk:'1.98.4',preparedPhases:[0,1]};
    }else{
      need(this.ready,'sdk_not_prepared');
      if(input.action==='BIND'){
        need(Object.keys(input).sort().join(',')==='action,blockhash,index'&&Number.isInteger(input.index)
          &&input.index>=0&&input.index<=1&&!this.bound.has(input.index),'sdk_original_once');
        const message=this.sdk.Message.from(this.templates[input.index]);message.recentBlockhash=input.blockhash;
        const wireBase64=this.unsigned(message).toString('base64'),reviewed=this.describe(wireBase64);
        // The unchanged Python builder independently compares every message byte.
        this.bound.set(input.index,{wireBase64,reviewed});result={wireBase64,reviewed};
      }else if(input.action==='VERIFY'){
        need(Object.keys(input).sort().join(',')==='action,index,wireBase64'&&this.bound.has(input.index),'sdk_bound_original_required');
        const bound=this.bound.get(input.index),decoded=this.checker.verifyPacket({signedWireBase64:input.wireBase64,reviewed:bound.reviewed});
        need(this.sdk.Transaction.from(decoded.wire).serialize().equals(decoded.wire),'sdk_signed_exact');
        result={wireSha256:decoded.wireSha256,messageSha256:decoded.messageSha256,signatureHex:decoded.firstSignatureHex};
      }else if(input.action==='CHECK'){
        need(Object.keys(input).sort().join(',')==='action,inputs'&&Array.isArray(input.inputs)&&input.inputs.length<=64,'sdk_check_bound');
        result=input.inputs.map(value=>this.checker.check(value)); // Fresh checks; no result cache.
      }else throw Error('sdk_action');
    }
    this.guard();return result;
  }
}
async function main(){
  const readline=require('node:readline'),worker=new BufferSdk();
  const lines=readline.createInterface({input:process.stdin,crlfDelay:Infinity});
  for await(const line of lines){
    need(Buffer.byteLength(line)<=MAX_INPUT,'sdk_input_bound');
    const value=worker.checker.parseInput(line),result=worker.run(value),raw=JSON.stringify({result})+'\n';
    need(Buffer.byteLength(raw)<=MAX_OUTPUT,'sdk_output_bound');process.stdout.write(raw);
  }
}
if(require.main===module)main().catch(()=>{process.stderr.write('sdk_worker_refused\n');process.exitCode=1;});
module.exports={BufferSdk,main,MAX_INPUT,MAX_OUTPUT};
