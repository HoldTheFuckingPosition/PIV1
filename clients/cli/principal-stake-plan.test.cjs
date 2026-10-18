'use strict';
const assert=require('node:assert/strict');
const fs=require('node:fs');
const path=require('node:path');
const os=require('node:os');
const {spawnSync}=require('node:child_process');
const {createHash}=require('node:crypto');
const test=require('node:test');
const p=require('./principal-stake-plan.cjs');
const n=require('./initializer-plan.cjs');
const {publicPoints}=require('./initializer-plan.test.cjs');
const t=require('../../validation/genesis-transport/transport.cjs');
const {PublicKey,VersionedTransaction,TransactionMessage}=require(t.SDK);
const POINTS=publicPoints(),copy=v=>JSON.parse(JSON.stringify(v)),MAX=(1n<<64n)-1n;
const FIXED={system:'11111111111111111111111111111111',token:'TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA',
  mint:'J1toso1uCk3RLmjorhTtrVwY9HJ7X8V9yYac6Y7kGCPn',poolProgram:'SPoo1Ku8WFXoNDMHPsrGSTSG1Y47rzgn41SLUNakuHy',pool:'Jito4APyf642JPZPx3hGc6WWJ8zPKtRbRs4P815Awbb'};
const WITHDRAW='6iQKfEyhr3bZMotVkW6beNZz5CPAkiwvgV2CTje9pVSS';
function fixture({shared=false,fee=7,amount='72623859790382856',minimum='0'}={}){
  const m=copy(p.TEMPLATE);Object.assign(m,{programId:POINTS[18],feePayer:POINTS[fee],amountLamports:amount,callerMinimumPoolTokensOut:minimum});
  Object.assign(m.protocol,{validatorList:POINTS[9],reserve:POINTS[10],managerFee:POINTS[11],referrer:shared?POINTS[11]:POINTS[13]});
  Object.assign(m.transport,{recentBlockhash:POINTS[16],lastValidBlockHeight:'777777',referenceSlot:'9007199254740997'});
  Object.assign(m.resources,{computeUnits:1400000,heapBytes:262144,loadedAccountBytes:8388608});return m;
}
function literal(m){
  const derive=seed=>PublicKey.findProgramAddressSync([Buffer.from(seed)],new PublicKey(m.programId))[0].toBase58();
  const row=(address,writable=false)=>({address,signer:false,writable});
  // Literal account order/flags independent of implementation role lists/helper.
  const rows=[row(derive('config'),true),row(derive('distribution')),row(derive('pending-sol')),row(derive('principal-sol'),true),row(derive('operational-sol')),
    row(derive('distribution-escrow')),row(derive('kif-sol')),row(derive('principal-jito-vault'),true),row(derive('pending-jito-vault')),row(derive('authority')),
    row(FIXED.mint,true),row(FIXED.system),row(FIXED.token),row(FIXED.poolProgram),row(FIXED.pool,true),row(m.protocol.validatorList),row(m.protocol.reserve,true),row(m.protocol.managerFee,true)];
  if(m.protocol.managerFee!==m.protocol.referrer)rows.push(row(m.protocol.referrer,true));rows.push(row(WITHDRAW));
  const little=value=>Buffer.from(Array.from({length:8},(_,i)=>Number(BigInt(value)>>BigInt(8*i)&255n)));
  return {rows,bytes:Buffer.concat([Buffer.from('504956315350303101','hex'),little(m.amountLamports),little(m.callerMinimumPoolTokensOut)])};
}
const packet=plan=>plan.plan.packets[0],row=r=>({address:r.address,signer:r.signer,writable:r.writable});
const sha=b=>createHash('sha256').update(b).digest('hex');

test('source pins and null manifest preserve missing fields and all OPEN limits',()=>{
  p.verifySources();assert.deepEqual(JSON.parse(fs.readFileSync(path.join(__dirname,'principal-stake-plan.example.json'))),p.TEMPLATE);
  for(const m of [copy(p.TEMPLATE),{}]){const out=p.planManifest(m);assert.equal(out.status,'INCOMPLETE');assert.equal(out.plan,null);assert(out.missing.includes('callerMinimumPoolTokensOut'));assert.equal(out.action,'stake-historical-principal-sol');}
  const out=p.planManifest(fixture());for(const field of ['publicReady','deploymentAuthorized','signerControlVerified','chainSnapshotVerified','accountValidityVerified','resourceSufficiencyVerified'])assert.equal(out.readiness[field],false);
  for(const field of ['fees','rent','funding','eligibility','quotedMinted','effectiveProtectedMinimum','bookValueBefore','bookValueAfter'])assert.equal(out.readiness[field],null);
  assert(out.preconditions.some(s=>s.includes('caller minimum0')&&s.includes('NOT zero effective protection')));
  assert.deepEqual(p.ERROR_GUIDE.map(e=>[e.code,e.name]),[[6027,'ZeroPrincipalDeposit'],[6028,'PrincipalDepositExceedsQueue'],[6029,'UnsupportedPrincipalDepositFee'],[6030,'InvalidPrincipalDepositPool'],[6031,'PrincipalDepositObservationMismatch'],[6032,'PrincipalDepositMinimumNotMet'],[6033,'PrincipalDepositHistoricalValueLoss'],[6034,'InvalidLifecycle'],[6035,'PausedOperation'],[6046,'InvalidCustodyObservation'],[6064,'HighWaterMarkDecrease'],[6069,'InvalidSlippage'],[6072,'ArithmeticOverflow']]);
});
test('independent25-byte payload and19/20 native role oracle keep historical source and final withdraw',()=>{
  const derivedWithdraw=PublicKey.findProgramAddressSync([new PublicKey(FIXED.pool).toBuffer(),Buffer.from('withdraw')],new PublicKey(FIXED.poolProgram))[0].toBase58();
  assert.equal(derivedWithdraw,WITHDRAW);assert.equal(p.POOL_WITHDRAW_AUTHORITY,WITHDRAW);
  for(const shared of [false,true]){
    const m=fixture({shared}),out=p.planManifest(m),expected=literal(m);assert.equal(out.status,'OFFLINE_PLAN_ONLY');assert.equal(out.plan.native.bytes,25);
    assert.equal(out.plan.native.hex,'50495631535030310108070605040302010000000000000000');assert.equal(out.plan.native.hex,expected.bytes.toString('hex'));
    assert.deepEqual(out.plan.roles.map(row),expected.rows);assert.equal(out.plan.roles.length,shared?19:20);assert(out.plan.roles.every(r=>!r.signer));
    assert.deepEqual(out.plan.roles.map((r,i)=>r.writable?i:null).filter(i=>i!==null),shared?[0,3,7,10,14,16,17]:[0,3,7,10,14,16,17,18]);
    assert.equal(out.plan.roles[2].role,'pending_sol');assert.equal(out.plan.roles[2].writable,false);assert.equal(out.plan.roles[3].role,'principal_sol');assert.equal(out.plan.roles[3].writable,true);
    assert.equal(out.plan.roles[15].writable,false);assert.equal(out.plan.roles.at(-1).address,WITHDRAW);assert.equal(out.plan.roles.at(-1).writable,false);
    assert.equal(out.plan.derived.pool_withdraw_authority,WITHDRAW);assert.equal(out.plan.requested.callerMinimumPoolTokensOut,'0');
    assert.equal(out.plan.quotedMinted,null);assert.equal(out.plan.effectiveProtectedMinimum,null);assert.equal(out.plan.bookValueAfter,null);
  }
});
test('full-u64 amount/minimum domains including zero and larger caller minimum preserve exact bytes/roles',()=>{
  const pairs=[[1n,0n],[1n,MAX],[MAX,0n],[MAX,MAX],[127n,128n],[255n,256n],[65535n,65536n],[(1n<<32n)-1n,1n<<32n],[(1n<<53n)+1n,(1n<<53n)+3n],[MAX-1n,MAX-2n]];
  for(const shared of [false,true]){
    const baseline=p.planManifest(fixture({shared})).plan.roles;
    for(const [amount,minimum]of pairs){const m=fixture({shared,amount:amount.toString(),minimum:minimum.toString()}),out=p.planManifest(m),b=Buffer.from(out.plan.native.hex,'hex');
      assert.deepEqual(b,literal(m).bytes);assert.equal(b.readBigUInt64LE(9),amount);assert.equal(b.readBigUInt64LE(17),minimum);assert.deepEqual(out.plan.roles,baseline);assert.equal(out.readiness.eligibility,null);}
  }
  for(const field of ['amountLamports','callerMinimumPoolTokensOut'])for(const value of [0,1,-1,'-1','+1','01','1.0','1e9','18446744073709551616']){const m=copy(p.TEMPLATE);m[field]=value;assert.throws(()=>p.planManifest(m));}
  const zeroAmount=fixture({amount:'0'});assert.throws(()=>p.planManifest(zeroAmount));assert.equal(p.planManifest(fixture({minimum:'0'})).status,'OFFLINE_PLAN_ONLY');
});
test('one actual zero signature and legacy877/910-byte wire roundtrip preserve budget/native privileges',()=>{
  for(const shared of [false,true])for(const fee of [0,7,8]){
    const m=fixture({shared,fee}),out=p.planManifest(m),wire=packet(out),raw=Buffer.from(wire.wireBase64,'base64'),tx=VersionedTransaction.deserialize(raw),expected=literal(m);
    assert.equal(wire.version,'legacy');assert.equal(wire.wireBytes,shared?877:910);assert(raw.length<=1232);assert.equal(wire.zeroSignatureBytes,64);assert.equal(wire.signatureCount,1);assert.deepEqual(wire.requiredSigners,[m.feePayer]);
    assert.equal(tx.signatures.length,1);assert(tx.signatures[0].every(b=>b===0));assert.deepEqual(Buffer.from(tx.serialize()),raw);assert.equal(wire.sha256,sha(raw));
    assert.deepEqual([tx.message.header.numRequiredSignatures,tx.message.header.numReadonlySignedAccounts,tx.message.header.numReadonlyUnsignedAccounts],[1,0,14]);
    const recovered=TransactionMessage.decompile(tx.message).instructions;assert.equal(recovered.length,4);assert.deepEqual(recovered.slice(0,3).map(i=>i.data.toString('hex')),['02c05c1500','0100000400','0400008000']);assert(recovered.slice(0,3).every(i=>i.keys.length===0));
    assert.equal(recovered[3].programId.toBase58(),m.programId);assert.deepEqual(recovered[3].data,expected.bytes);
    assert.deepEqual(recovered[3].keys.map(k=>({address:k.pubkey.toBase58(),signer:k.isSigner,writable:k.isWritable})),expected.rows);
    assert(!recovered[3].keys.some(k=>k.pubkey.toBase58()===m.feePayer));
  }
});
test('resource bounds and all native/protected fee/protocol aliases reject without blanket curve rules',()=>{
  const m=fixture();for(const native of p.planManifest(m).plan.roles){const bad=copy(m);bad.feePayer=native.address;assert.throws(()=>p.planManifest(bad));}
  for(const field of ['validatorList','reserve','managerFee','referrer']){const partial=copy(p.TEMPLATE);partial.protocol[field]=WITHDRAW;assert.throws(()=>p.planManifest(partial));}
  for(const field of ['programId','feePayer']){const partial=copy(p.TEMPLATE);partial[field]=WITHDRAW;assert.throws(()=>p.planManifest(partial));}
  const same=fixture();same.protocol.reserve=same.protocol.validatorList;assert.throws(()=>p.planManifest(same));
  const feeAlias=fixture();feeAlias.feePayer=feeAlias.protocol.managerFee;assert.throws(()=>p.planManifest(feeAlias));
  const programAlias=copy(p.TEMPLATE);programAlias.programId=POINTS[9];programAlias.protocol.validatorList=POINTS[9];assert.throws(()=>p.planManifest(programAlias));
  const unknownPdas=fixture();unknownPdas.programId=PublicKey.findProgramAddressSync([Buffer.from('public-program-fixture')],new PublicKey(FIXED.poolProgram))[0].toBase58();
  unknownPdas.protocol.validatorList=PublicKey.findProgramAddressSync([Buffer.from('public-list-fixture')],new PublicKey(FIXED.poolProgram))[0].toBase58();assert(!PublicKey.isOnCurve(new PublicKey(unknownPdas.programId).toBytes()));assert.equal(p.planManifest(unknownPdas).status,'OFFLINE_PLAN_ONLY');
  for(const [field,values]of [['computeUnits',[0,1400001,1.5,'1400000']],['heapBytes',[32767,32769,262145,'32768']],['loadedAccountBytes',[0,8388609,-1,'8388608']]])for(const value of values){const partial=copy(p.TEMPLATE);partial.resources[field]=value;assert.throws(()=>p.planManifest(partial));}
  const small=fixture();small.resources={computeUnits:1,heapBytes:32768,loadedAccountBytes:1};assert.deepEqual(packet(p.planManifest(small)).requestedInstructions.slice(0,3).map(i=>i.dataHex),['0201000000','0100800000','0401000000']);
});
test('strict known-field/alias/schema checks run before missing input reporting',()=>{
  const changes=[m=>m.schema='other',m=>m.cluster='mainnet-beta',m=>m.programId=FIXED.pool,m=>m.feePayer='1',m=>m.protocol.validatorList=FIXED.mint,
    m=>m.amountLamports='0',m=>m.callerMinimumPoolTokensOut='-0',m=>m.transport.referenceSlot='01',m=>m.protocol.reserve=m.protocol.managerFee,
    m=>m.amountLamports=1,m=>m.protocol.extra='do-not-echo',m=>m.governance={},m=>m.slippageBps=1,m=>m.action='pending-stake'];
  for(const change of changes){const bad=fixture();change(bad);assert.throws(()=>p.planManifest(bad));bad.transport.recentBlockhash=null;assert.throws(()=>p.planManifest(bad));}
  for(const field of ['programId','feePayer','amountLamports','callerMinimumPoolTokensOut','protocol','transport','resources']){const m=fixture();m[field]=null;const out=p.planManifest(m);assert.equal(out.status,'INCOMPLETE');assert.equal(out.plan,null);}
  for(const block of ['protocol','transport','resources'])for(const field of Object.keys(fixture()[block])){const m=fixture();m[block][field]=null;assert.equal(p.planManifest(m).plan,null);}
  for(const text of ['{"feePayer":null,"feePayer":null}','{"feePayer":null,"fee\\u0050ayer":null}','[]','{"unknown":"do-not-echo"}'])assert.throws(()=>p.planManifest(n.parseManifest(text)));
});
test('CLI retains shared/distinct actual reports with explicit zero/fullwidth minima and no readiness claim',context=>{
  const script=path.join(__dirname,'principal-stake-plan.cjs'),limits=process.execArgv.filter(v=>v.startsWith('--max-old-space-size'));
  const run=args=>spawnSync(process.execPath,[...limits,script,...args],{encoding:'utf8',maxBuffer:2*1024*1024,timeout:30000});
  const template=run(['--template']);assert.equal(template.status,0);assert.equal(template.stderr,'');assert.deepEqual(JSON.parse(template.stdout),p.TEMPLATE);
  const incomplete=run(['--manifest',path.join(__dirname,'principal-stake-plan.example.json')]);assert.equal(incomplete.status,0);assert.equal(JSON.parse(incomplete.stdout).status,'INCOMPLETE');assert.equal(JSON.parse(incomplete.stdout).plan,null);
  const directory=fs.mkdtempSync(path.join(os.tmpdir(),'piv1-principal-stake-public-test-'));
  for(const shared of [true,false]){const name=shared?'shared':'distinct',manifest=fixture({shared,amount:shared?'1000000000':'9007199254740993',minimum:shared?'0':'9007199254740995'}),filename=path.join(directory,`${name}.manifest.json`);
    fs.writeFileSync(filename,JSON.stringify(manifest,null,2)+'\n',{flag:'wx'});const complete=run(['--manifest',filename]);fs.writeFileSync(path.join(directory,`${name}.report.json`),complete.stdout,{flag:'wx'});
    assert.equal(complete.status,0);assert.equal(complete.stderr,'');const parsed=JSON.parse(complete.stdout);assert.equal(parsed.status,'OFFLINE_PLAN_ONLY');assert.deepEqual(parsed.plan,p.planManifest(manifest).plan);
    assert.equal(parsed.plan.native.hex,literal(manifest).bytes.toString('hex'));assert.equal(parsed.plan.packets.length,1);assert.equal(parsed.plan.packets[0].wireBytes,shared?877:910);
    assert.equal(parsed.plan.requested.callerMinimumPoolTokensOut,manifest.callerMinimumPoolTokensOut);assert.equal(parsed.plan.effectiveProtectedMinimum,null);assert.equal(parsed.readiness.publicReady,false);assert.equal(parsed.readiness.eligibility,null);
  }
  const invalid=path.join(directory,'invalid.public.json');fs.writeFileSync(invalid,'{"unknown":"do-not-echo"}',{flag:'wx'});
  for(const args of [[],['--send'],['--template','--rpc'],['--manifest'],['--manifest',invalid]]){const bad=run(args);assert.notEqual(bad.status,0);assert.equal(bad.stdout,'');assert.equal(JSON.parse(bad.stderr).status,'INVALID_OR_UNSUPPORTED_PRINCIPAL_STAKE_PLAN');assert(!bad.stderr.includes('do-not-echo'));}
  context.diagnostic(`Complete shared/distinct public principal-stake reports retained: ${directory}`);
});
