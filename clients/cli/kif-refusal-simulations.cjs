'use strict';
// Four independent, unsigned public-Testnet negative tests; no signing or submission API.
const fs=require('node:fs'),path=require('node:path'),assert=require('node:assert/strict'),{createHash}=require('node:crypto');
const R=path.resolve(__dirname,'../..'),sdk=require(R+'/spikes/task-0.4-jito/node_modules/@solana/web3.js/lib/index.cjs.js');
const planner=require('./guardian-kif-plan.cjs'),state=require('./state-inspect.cjs'),guardian=require('./guardian-inspect.cjs'),hybrid=require('./hybrid-instructions.cjs');
const K=JSON.parse(fs.readFileSync('/home/jerem/piv1-pilot-checkpoints/m5-practical-functional-tests-20261107-a/known-accounts-v2.public.json'));
const SOURCE='4WKQg3Sm8bvHS8DBmxoiMMi4Fev4mJU6ZLwGSTg6jDna',G='BUuuNvicRKF4ZFEUH4iS8xGzhAKQca3ou4U1s4NaBYoT',PROGRAM='GAjHRaC3XM98SoRoFy5B8P2ypTS1ndf1UPhXmjWgtPux';
const ELF='047c819b717bb0b74602596297a02bf5d84deaec327241a9981926fe144696ac',REWARD='9rKTQVnstMQygjZhzenRX7pemRQaGMUN1nEWFxPTpUBL';
const pk=k=>new sdk.PublicKey(k),sha=b=>createHash('sha256').update(b).digest('hex');
const CASES=[
 {id:'A',condition:'stale expected counter; amount also exceeds exhausted entitlement, but counter is checked first',amount:'4688',counter:'0',name:'StaleKifClaim',code:6004,guard:'programs/piv1/src/state/kif_claim.rs:102'},
 {id:'B',condition:'positive overclaim against exhausted entitlement, not every partially funded scenario',amount:'1',counter:'4688',name:'KifClaimExceeded',code:6005,guard:'programs/piv1/src/state/kif_claim.rs:105'},
 {id:'C',condition:'wrong earned owner/destination; signer privilege retained; authentication precedes entitlement',amount:'1',counter:'4688',name:'InvalidGuardianSet',code:6067,guard:'programs/piv1/src/kif_claim_accounts.rs:104'},
 {id:'D',condition:'legitimate guardian lacks compiled signer privilege; separate funded public payer',amount:'1',counter:'4688',name:'MissingGuardianSignature',code:6002,guard:'programs/piv1/src/kif_claim_accounts.rs:76'},
];
function build(test,latest){
 const m=JSON.parse(fs.readFileSync('/home/jerem/piv1-pilot-checkpoints/post-d109-readiness-kif-20261108-a/claim-manifest.public.json'));
 m.transport={recentBlockhash:latest.value.blockhash,lastValidBlockHeight:String(latest.value.lastValidBlockHeight),referenceSlot:String(latest.context.slot)};
 m.claim.amountLamports=test.amount;m.claim.expectedCumulativeClaimed=test.counter;
 const planned=planner.planManifest(m);assert.equal(planned.status,'OFFLINE_PLAN_ONLY');
 const instructions=planned.plan.packets[0].requestedInstructions.map(x=>new sdk.TransactionInstruction({programId:pk(x.programId),data:Buffer.from(x.dataHex,'hex'),keys:x.accounts.map(a=>({pubkey:pk(a.address),isSigner:a.signer,isWritable:a.writable}))}));
 assert.equal(instructions.length,4);const claim=instructions[3];assert.equal(claim.programId.toBase58(),PROGRAM);assert.equal(claim.keys[1].pubkey.toBase58(),REWARD);
 if(test.id==='C')claim.keys[3].pubkey=pk(SOURCE);
 if(test.id==='D')claim.keys[3].isSigner=false;
 const payer=test.id==='D'?SOURCE:G,tx=new sdk.VersionedTransaction(new sdk.TransactionMessage({payerKey:pk(payer),recentBlockhash:latest.value.blockhash,instructions}).compileToLegacyMessage());
 const roles=claim.keys.map(k=>{const index=tx.message.accountKeys.findIndex(a=>a.equals(k.pubkey));return{address:k.pubkey.toBase58(),requestedSigner:k.isSigner,requestedWritable:k.isWritable,compiledSigner:tx.message.isAccountSigner(index),compiledWritable:tx.message.isAccountWritable(index)};});
 assert.equal(roles[3].compiledSigner,test.id!=='D');assert(roles.slice(0,4).every(x=>x.compiledWritable));assert.equal(new Set(roles.map(x=>x.address)).size,5);
 assert(tx.signatures.every(s=>s.every(b=>b===0)));assert.equal(tx.signatures.length,test.id==='C'?2:1);
 return{tx,public:{...test,payer,roles,instructions:instructions.map(i=>({programId:i.programId.toBase58(),dataHex:i.data.toString('hex'),accounts:i.keys.map(k=>({address:k.pubkey.toBase58(),signer:k.isSigner,writable:k.isWritable}))})),wireBase64:Buffer.from(tx.serialize()).toString('base64'),allSignatureSlotsZero:true,signatureSlots:tx.signatures.length}};
}
async function main(output){
 assert(output&&path.isAbsolute(output),'ABSOLUTE_NEW_EVIDENCE_DIRECTORY_REQUIRED');fs.mkdirSync(output,{mode:0o700});
 function save(name,value){fs.writeFileSync(path.join(output,name),JSON.stringify(value,null,2)+'\n',{flag:'wx',mode:0o600});}
 const directive=fs.readFileSync('/home/jerem/.codex/attachments/147a25f2-3c9f-4027-8c3c-dd5d949b8678/Texte collé.txt');
 fs.writeFileSync(path.join(output,'founder-directive.md'),directive,{flag:'wx',mode:0o600});save('authorization.public.json',{scope:'FOUR_UNSIGNED_KIF_REFUSAL_SIMULATIONS_ONLY',directiveSHA256:sha(directive),privateKeyReads:0,signatures:0,submissions:0,actualFees:0,D109AndD110RemainConsumed:true});
 const c=new sdk.Connection('https://api.testnet.solana.com',{commitment:'finalized',disableRetryOnRateLimit:true,fetch:(url,options)=>fetch(url,{...options,signal:AbortSignal.timeout(20000)})});
 assert.equal(await c.getGenesisHash(),'4uhcVJyU9pJkvQyS88uRDiswHXSCkY3zQawwpjk2NsNY');assert.equal(K.program,PROGRAM);assert.equal(K.g0,G);assert.equal(K.economicKeys[3],REWARD);
 const sourceFiles=['clients/cli/kif-refusal-simulations.cjs','clients/cli/guardian-kif-plan.cjs','clients/cli/guardian-inspect.cjs','clients/cli/state-inspect.cjs','clients/cli/hybrid-instructions.cjs','programs/piv1/src/instructions/claim_kif.rs','programs/piv1/src/kif_claim_accounts.rs','programs/piv1/src/kif_claim_execution.rs','programs/piv1/src/state/kif_claim.rs','programs/piv1/src/instruction_errors.rs','programs/piv1/src/instruction_boundary.rs'];
 save('source-bindings.public.json',Object.fromEntries(sourceFiles.map(p=>[p,sha(fs.readFileSync(path.join(R,p)))])));
 const mapping=fs.readFileSync(R+'/programs/piv1/src/instruction_errors.rs','utf8');for(const t of CASES)assert(mapping.includes('Piv1Error::'+t.name+' => '+t.code+','));
 save('expected-cases.public.json',CASES); // Source-derived expectations are durable before any simulation.
 const keys=[...new Set([...K.economicKeys,G,SOURCE,PROGRAM,K.programData,sdk.SystemProgram.programId.toBase58()])];
 const before=await c.getMultipleAccountsInfoAndContext(keys.map(pk),{commitment:'finalized'});
 const publicAccount=a=>a?{owner:a.owner.toBase58(),executable:a.executable,lamports:a.lamports,dataBase64:a.data.toString('base64')}:null;
 save('opening.public.json',{keys,context:before.context,accounts:before.value.map(publicAccount)});
 const accounts=Object.fromEntries(keys.map((k,i)=>[k,before.value[i]])),config=state.decodeConfig(accounts[K.economicKeys[0]].data).value,reward=guardian.decodeReward(accounts[REWARD].data).value,round=hybrid.decodeDistribution(accounts[K.economicKeys[1]].data).value;
 const pd=accounts[K.programData],program=accounts[PROGRAM],loader='BPFLoaderUpgradeab1e11111111111111111111111';
 assert(program.executable&&program.owner.toBase58()===loader&&program.data.readUInt32LE()===2&&pk(program.data.subarray(4,36)).toBase58()===K.programData);
 assert(pd.owner.toBase58()===loader&&pd.data.readUInt32LE()===3&&pd.data[12]===1&&pk(pd.data.subarray(13,45)).toBase58()===K.vaultAuthority&&sha(pd.data.subarray(45,45+882256))===ELF&&pd.data.subarray(45+882256).every(x=>x===0));
 assert.equal(reward.guardian,G);assert.equal(reward.cumulative_earned,'4688');assert.equal(reward.cumulative_claimed,'4688');assert.equal(reward.claimable_lamports,'0');assert.equal(config.kif_claim_liability_lamports,'0');assert.equal(config.cumulative_kif_claimed_lamports,'4688');
 assert(accounts[SOURCE]&&accounts[SOURCE].owner.equals(sdk.SystemProgram.programId)&&!accounts[SOURCE].executable&&accounts[SOURCE].data.length===0&&accounts[SOURCE].lamports>10000,'EXISTING_FUNDED_PUBLIC_SIMULATION_PAYER');
 save('starting-state.public.json',{context:before.context,config,reward,round,sourcePublicAccountOnly:true,g0Lamports:accounts[G].lamports,sourceLamports:accounts[SOURCE].lamports,kifVaultLamports:accounts[K.economicKeys[13]].lamports});
 const latest=await c.getLatestBlockhashAndContext({commitment:'finalized',minContextSlot:before.context.slot});save('simulation-reference.public.json',latest);
 const postKeys=[K.economicKeys[0],REWARD,K.economicKeys[13],G,SOURCE],results=[];let stopped=false;
 try{for(const test of CASES){
  const built=build(test,latest);save('case-'+test.id+'-request.public.json',built.public);
  const response=await c.simulateTransaction(built.tx,{sigVerify:false,replaceRecentBlockhash:true,commitment:'finalized',minContextSlot:before.context.slot,accounts:{encoding:'base64',addresses:postKeys},innerInstructions:true});
  save('case-'+test.id+'-response.public.json',response);
  const value=response.value,logs=value.logs||[],reached=logs.some(l=>l===`Program ${PROGRAM} invoke [1]`),expectedError={InstructionError:[3,{Custom:test.code}]};
  const match=JSON.stringify(value.err)===JSON.stringify(expectedError)&&reached&&logs.some(l=>l===`Program ${PROGRAM} failed: custom program error: 0x${test.code.toString(16)}`);
  const result={...built.public,context:response.context,actualError:value.err,actualErrorName:match?test.name:null,piv1Reached:reached,intendedGuardExercised:match,computeUnitsConsumed:value.unitsConsumed??null,status:match?'PASS_EXPECTED_PROGRAM_REFUSAL':value.err===null?'FAIL_UNEXPECTED_SIMULATED_SUCCESS':'NOT_EXERCISED_DIFFERENT_FAILURE',logs,simulatedPoststates:{keys:postKeys,accounts:value.accounts??null,limitation:'Failed-simulation poststates may be null; not a finalized receipt or proof of every rollback path. Signature verification is disabled; no Ed25519 authenticity is tested.'}};
  results.push(result);save('case-'+test.id+'-result.public.json',result);console.log(JSON.stringify({case:test.id,status:result.status,error:value.err,CU:result.computeUnitsConsumed}));
  if(!match){stopped=true;break;}
 }}finally{
  const finalKeys=keys.filter(k=>![PROGRAM,K.programData].includes(k)),lastSlot=Math.max(before.context.slot,...results.map(r=>r.context.slot));
  const after=await c.getMultipleAccountsInfoAndContext(finalKeys.map(pk),{commitment:'finalized',minContextSlot:lastSlot});save('final-readback.public.json',{keys:finalKeys,context:after.context,accounts:after.value.map(publicAccount)});
  const unchanged=finalKeys.every((k,i)=>JSON.stringify(publicAccount(accounts[k]))===JSON.stringify(publicAccount(after.value[i])));
  const report={schema:'piv1-kif-current-state-refusal-simulations-v1',evidenceLabel:'PUBLIC TESTNET RPC SIMULATION — EXPECTED PROGRAM REFUSAL',status:results.length===4&&results.every(r=>r.intendedGuardExercised)&&unchanged?'PASS_FOUR_INDEPENDENT_REFUSALS':stopped?'STOPPED_ON_UNEXPECTED_RESULT':'INCOMPLETE_OR_STATE_CHANGED',cluster:'testnet',genesis:K.genesisHash,program:PROGRAM,activeELFSHA256:ELF,openingContext:before.context,finalContext:after.context,startingState:{HWM:config.protected_principal_hwm_lamports,principalJitoSOL:config.accounted_historical_jitosol_units,principalSOL:config.accounted_historical_sol_lamports,earned:reward.cumulative_earned,claimed:reward.cumulative_claimed,claimable:reward.claimable_lamports,globalLiability:config.kif_claim_liability_lamports,kifVaultLamports:accounts[K.economicKeys[13]].lamports,roundVersion:round.version,roundLifecycle:round.lifecycle},results,actualReadbackUnchanged:unchanged,actualSignatures:0,submissions:0,privateKeyReads:0,actualFeesLamports:0,actualEconomicEffects:0,accounting:{missionConfirmedFees:17480000,missionFeeHeadroom:15000,oldHold:135000,separateReserve:370000,nonceRent:1056640},D109AndD110Consumed:true,limitations:['Unsigned RPC simulation, never finalized transaction evidence.','Only these exact source guards and exhausted-entitlement state are exercised.','Pause, guardian replacement, old-earned ownership and collective carry remain separate.']};
  save('report.public.json',report);assert(unchanged,'ACTUAL_FINAL_READBACK_CHANGED');
 }
 assert(!stopped&&results.length===4,'BATCH_STOPPED_WITHOUT_RETRY');
}
module.exports={CASES,build};
if(require.main===module){if(process.argv.length!==4||process.argv[2]!=='--output')throw Error('EXPECTED_OUTPUT_ARGUMENT');main(process.argv[3]).catch(e=>{console.error(JSON.stringify({status:'STOPPED_NO_SIGNING_OR_SUBMISSION',type:e.name,code:'READONLY_BATCH_FAILED'}));process.exitCode=1;});}
