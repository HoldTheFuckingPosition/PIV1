'use strict';
// Dedicated public-only legacy-message leaf. All source loading precedes ORIGINAL.
const fs=require('node:fs'),path=require('node:path'),crypto=require('node:crypto');
const REPO=path.resolve(__dirname,'..');
const sha=b=>crypto.createHash('sha256').update(b).digest('hex');
const need=(v,r)=>{if(!v)throw Error(r);};
function load(file,want){
  const Module=require('node:module'),raw=fs.readFileSync(file);
  need(!fs.lstatSync(file).isSymbolicLink()&&sha(raw)===want,'loaded_source_pin');
  const m=new Module(file);m.filename=file;m.paths=Module._nodeModulePaths(path.dirname(file));require.cache[file]=m;
  m._compile(raw.toString(),file);m.loaded=true;return m.exports;
}
const sdk=load(path.join(REPO,'spikes/task-0.4-jito/node_modules/@solana/web3.js/lib/index.cjs.js'),'796768bf5a689d808fb6fabfc5ce0db27eee064782cf87621331b807bab9f7e2');
const checker=load(path.join(REPO,'clients/cli/startup-public-check.cjs'),'50080d7baa17098e4ee687dcf0abb5fd4cef27e88154edd0c32b47e4f033fce9');
class TwoOperationSdk {
  constructor(){this.template=null;this.bound=null;checker.verifySources();}
  run(v){
    if(v.action==='PREPARE'){
      need(this.template===null&&[0,1].includes(v.index),'prepare_once');
      const raw=Buffer.from(v.templateBase64,'base64'),m=sdk.Message.from(raw);
      need(m.version==='legacy'&&m.serialize().equals(raw),'template_exact');
      this.template=Buffer.from(raw);this.index=v.index;return {sdk:'1.98.4',index:v.index};
    }
    need(this.template!==null,'not_prepared');
    if(v.action==='BIND'){
      need(this.bound===null,'one_original');
      const m=sdk.Message.from(this.template);m.recentBlockhash=v.blockhash;
      const wire=sdk.Transaction.populate(m).serialize({requireAllSignatures:false,verifySignatures:false});
      const d=checker.decodeWire(wire.toString('base64'),[]);
      const reviewed={messageBase64:d.message.toString('base64'),messageSha256:d.messageSha256,
        feePayer:d.feePayer,requiredSigners:d.requiredSigners,accountRoles:d.roles,lookupTables:[]};
      this.bound={wireBase64:wire.toString('base64'),reviewed};return this.bound;
    }
    need(v.action==='VERIFY'&&this.bound!==null,'action');
    const d=checker.verifyPacket({signedWireBase64:v.wireBase64,reviewed:this.bound.reviewed});
    need(sdk.Transaction.from(d.wire).serialize().equals(d.wire),'signed_exact');
    return {wireSha256:d.wireSha256,messageSha256:d.messageSha256,signatureHex:d.firstSignatureHex};
  }
}
async function main(){
  const w=new TwoOperationSdk(),reader=require('node:readline').createInterface({input:process.stdin,crlfDelay:Infinity});
  for await(const line of reader){need(Buffer.byteLength(line)<=16384,'input_bound');
    const out=JSON.stringify({result:w.run(checker.parseInput(line))})+'\n';need(out.length<=16384,'output_bound');process.stdout.write(out);}
}
if(require.main===module)main().catch(()=>{process.stderr.write('two_operation_sdk_refused\n');process.exitCode=1;});
module.exports={main,TwoOperationSdk};
