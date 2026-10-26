'use strict';
// Public declarations/supplied ALT bytes and unsigned encodings only. No network/wallet API.
const assert=require('node:assert/strict'),fs=require('node:fs'),path=require('node:path');
const base=require('./initializer-plan.cjs');
const {createHash}=require('node:crypto');
const {PublicKey}=base;
const SCHEMA='piv1-initializer-existing-alt-plan-v2';
const BASELINE='e85ee1b79778086742c6591e6fe8342d175155a1';
const SOURCE_PINS=Object.freeze({
  'clients/cli/initializer-plan.cjs':'92c13a4205f4557c74f55b03ee2dfea458592938228a7a959f5a2baf35bbeb83'
});
const ALT_LAYOUT_REFERENCE=Object.freeze({interface:'solana-address-lookup-table-interface3.1.0',stateSourceSha256:'44bc430a102228883c329b0af472c5adf54f188b3ee9465242265f83f7f3cd8e',metadataBytes:56,accountBytes:568});
const TEMPLATE={schema:SCHEMA,initializer:JSON.parse(JSON.stringify(base.TEMPLATE)),existingAlt:{address:null,owner:null,executable:null,lamports:null,dataBase64:null,contextSlot:null}};
const U64=(1n<<64n)-1n,MAX_BYTES=65536;
const sha=b=>createHash('sha256').update(b).digest('hex'),pk=s=>new PublicKey(s);
const plain=x=>x!==null&&typeof x==='object'&&!Array.isArray(x)&&Object.getPrototypeOf(x)===Object.prototype;
const fail=(code,field)=>{const e=new Error(code);e.code=code;e.field=field;throw e;};
const u64=n=>{const b=Buffer.alloc(8);b.writeBigUInt64LE(BigInt(n));return b;};
function object(value,shape,field){if(!plain(value))fail('object_required',field);if(Object.keys(value).some(k=>!Object.hasOwn(shape,k)))fail('unknown_field',field);}
function publicKey(value,field){if(typeof value!=='string')fail('public_key_string',field);let k;try{k=pk(value);}catch{fail('invalid_public_key',field);}if(k.toBase58()!==value)fail('noncanonical_public_key',field);return k;}
function decimal(value,field){if(typeof value!=='string'||!/^(?:0|[1-9]\d*)$/.test(value))fail('decimal_string_required',field);if(BigInt(value)>U64)fail('integer_range',field);}
function initializerValidation(value){try{return base.validateManifest(value);}catch(e){e.field=e.field==='manifest'?'initializer':`initializer.${e.field||'manifest'}`;throw e;}}
function decodeAltData(value){
  const field='existingAlt.dataBase64';
  // 568 bytes have exactly760 canonical padded base64 characters.
  if(typeof value!=='string'||value.length!==760||!(/^[A-Za-z0-9+/]{758}==$/.test(value)))fail('alt_base64',field);
  const b=Buffer.from(value,'base64');if(b.length!==568||b.toString('base64')!==value)fail('alt_base64',field);
  if(b.readUInt32LE(0)!==1)fail('alt_state_variant',field);
  if(b.readBigUInt64LE(4)!==U64)fail('alt_not_active',field);
  if(b[20]>16)fail('alt_start_index',field);
  if(b[21]!==0&&b[21]!==1)fail('alt_authority_option',field);
  if(b[21]===0){if(b.subarray(22,56).some(x=>x!==0))fail('alt_padding',field);fail('alt_authority_required',field);}
  if(b[54]!==0||b[55]!==0)fail('alt_padding',field);
  const authority=pk(b.subarray(22,54)).toBase58();if(authority===base.IDS.system)fail('alt_zero_authority',field);
  const addresses=Array.from({length:16},(_,i)=>pk(b.subarray(56+32*i,88+32*i)).toBase58());
  if(new Set(addresses).size!==16||addresses.includes(base.IDS.system))fail('alt_invalid_addresses',field);
  return {bytes:568,sha256:sha(b),variant:1,deactivationSlot:U64.toString(),lastExtendedSlot:b.readBigUInt64LE(12).toString(),lastExtendedSlotStartIndex:b[20],authority,addresses};
}
function knownTargets(m){
  const expected=Array(16).fill(null);if(m?.programId==null)return expected;
  const program=pk(m.programId),derive=seeds=>PublicKey.findProgramAddressSync(seeds,program)[0].toBase58();
  ['config','distribution','guardian-registry'].forEach((s,i)=>expected[i]=derive([Buffer.from(s)]));
  ['pending-sol','principal-sol','operational-sol','distribution-escrow','kif-sol','principal-jito-vault','pending-jito-vault'].forEach((s,i)=>expected[9+i]=derive([Buffer.from(s)]));
  const g=m.governance;
  if(g?.guardians?.every(v=>v!==null)&&g.guardians.length===6){const members=g.guardians.map(pk).sort((x,y)=>Buffer.compare(x.toBuffer(),y.toBuffer()));for(let i=0;i<6;i++)if(g.guardianSlotPermutation?.[i]!=null)expected[3+i]=derive([Buffer.from('guardian-reward'),members[g.guardianSlotPermutation[i]].toBuffer(),u64(0),Buffer.from([i])]);}
  return expected;
}
function checkRelations(m,a,state){
  const c=m?.transport,g=m?.governance,targets=knownTargets(m);
  if(a?.address!=null){
    const known=[...Object.values(base.IDS),m?.programId,m?.feePayer,m?.rentPayer,g?.multisigAddress,g?.creator,g?.executor,...(g?.guardians||[]),...(g?.approvers||[]),...Object.values(m?.protocol||{}),...Object.values(m?.recipients||{}),c?.altAuthority,state?.authority,...targets];
    if(known.some(v=>v!=null&&v===a.address))fail('alt_known_role_alias','existingAlt.address');
    if(m?.programId!=null){const program=pk(m.programId);for(const k of [PublicKey.findProgramAddressSync([program.toBuffer()],pk(base.IDS.loader))[0],PublicKey.findProgramAddressSync([Buffer.from('authority')],program)[0]])if(k.toBase58()===a.address)fail('alt_known_role_alias','existingAlt.address');}
    if(g?.multisigAddress!=null){
      const root=[Buffer.from('multisig'),pk(g.multisigAddress).toBuffer()],derive=seeds=>PublicKey.findProgramAddressSync(seeds,pk(base.IDS.squads))[0].toBase58(),derived=[];
      for(const index of [g.vaultIndex])if(index!=null)derived.push(derive([...root,Buffer.from('vault'),Buffer.from([index])]));
      if(g.transactionIndex!=null){const tx=[...root,Buffer.from('transaction'),u64(g.transactionIndex)];derived.push(derive(tx),derive([...tx,Buffer.from('proposal')]));}
      if(g.creator!=null&&g.bufferIndex!=null)derived.push(derive([...root,Buffer.from('transaction_buffer'),pk(g.creator).toBuffer(),Buffer.from([g.bufferIndex])]));
      if(derived.includes(a.address))fail('alt_known_role_alias','existingAlt.address');
    }
  }
  if(c?.referenceSlot!=null&&a?.contextSlot!=null&&c.referenceSlot!==a.contextSlot)fail('alt_context_mismatch','existingAlt.contextSlot');
  if(state&&a?.contextSlot!=null&&BigInt(a.contextSlot)<=BigInt(state.lastExtendedSlot))fail('alt_not_strictly_warmed','existingAlt.contextSlot');
  if(state&&c?.altAuthority!=null&&state.authority!==c.altAuthority)fail('alt_authority_mismatch','existingAlt.dataBase64');
  if(state&&c?.altRecentSlot!=null&&BigInt(c.altRecentSlot)>BigInt(state.lastExtendedSlot))fail('alt_derivation_after_extension','initializer.transport.altRecentSlot');
  const authority=c?.altAuthority??state?.authority;
  if(a?.address!=null&&authority!=null&&c?.altRecentSlot!=null){const derived=PublicKey.findProgramAddressSync([pk(authority).toBuffer(),u64(c.altRecentSlot)],pk(base.IDS.altProgram))[0].toBase58();if(derived!==a.address)fail('alt_seed_mismatch','existingAlt.address');}
  if(state)targets.forEach((v,i)=>{if(v!==null&&v!==state.addresses[i])fail('alt_target_order_mismatch',`existingAlt.dataBase64.addresses[${i}]`);});
}
function validateManifest(input){
  object(input,TEMPLATE,'manifest');const missing=[];
  let schema=input.schema??null;if(schema===null)missing.push('schema');else if(schema!==SCHEMA)fail('unsupported_schema','schema');
  let m=null;if(input.initializer===null||input.initializer===undefined)missing.push('initializer');else{const v=initializerValidation(input.initializer);m=v.manifest;missing.push(...v.missing.map(f=>`initializer.${f}`));}
  let a=null,state=null;
  if(input.existingAlt===null||input.existingAlt===undefined)missing.push('existingAlt');else{
    object(input.existingAlt,TEMPLATE.existingAlt,'existingAlt');a={};for(const f of Object.keys(TEMPLATE.existingAlt)){a[f]=input.existingAlt[f]??null;if(a[f]===null)missing.push(`existingAlt.${f}`);}
    for(const f of ['address','owner'])if(a[f]!==null)publicKey(a[f],`existingAlt.${f}`);
    if(a.address===base.IDS.system)fail('alt_zero_address','existingAlt.address');
    if(a.owner!==null&&a.owner!==base.IDS.altProgram)fail('alt_owner','existingAlt.owner');
    if(a.executable!==null&&(typeof a.executable!=='boolean'||a.executable))fail('alt_executable','existingAlt.executable');
    for(const f of ['lamports','contextSlot'])if(a[f]!==null)decimal(a[f],`existingAlt.${f}`);
    if(a.lamports!==null&&BigInt(a.lamports)===0n)fail('alt_zero_lamports','existingAlt.lamports');
    if(a.dataBase64!==null)state=decodeAltData(a.dataBase64);
  }
  checkRelations(m,a,state);return {manifest:{schema,initializer:m,existingAlt:a},state,missing};
}
function planManifest(input){
  const {manifest:wrapped,state,missing}=validateManifest(input);
  const readiness={publicReady:false,deploymentAuthorized:false,walletControlVerified:false,chainSnapshotVerified:false,existingAltSnapshotAuthenticated:false,existingAltSnapshotFresh:false,altWarmupVerified:false,actualFees:null,actualRent:null,actualFundingQuote:null,fullProjectBudget:null,publicConsumptionMeasured:false,m4Reservation:'OPEN_UNWAIVED'};
  const common={schema:SCHEMA,sourceBaseline:BASELINE,altLayoutReference:ALT_LAYOUT_REFERENCE,readiness,newAltSetupPackets:0,newAltSetupRentOutflow:'0'};
  if(missing.length)return {...common,status:'INCOMPLETE',missing,plan:null};
  let original;try{original=base.planManifest(wrapped.initializer);}catch(e){e.field=e.field==='manifest'?'initializer':`initializer.${e.field||'manifest'}`;throw e;}
  assert.equal(original.status,'OFFLINE_PLAN_ONLY');const plan=original.plan,a=wrapped.existingAlt;
  assert.equal(plan.alt.address,a.address);assert.deepEqual(plan.alt.addresses,state.addresses);
  const packets=plan.packets.filter(p=>p.name!=='alt_create'&&p.name!=='alt_extend');assert.equal(plan.packets.length-packets.length,2);
  assert(packets.every(p=>p.requestedInstructions.every(i=>i.programId!==base.IDS.altProgram)));
  return {...common,status:'OFFLINE_PLAN_ONLY',missing:[],governanceSchema:original.governanceSchema,
    declarationSha256:sha(Buffer.from(JSON.stringify(wrapped))),initializerDeclarationSha256:original.declarationSha256,
    plan:{...plan,packets,accountSizes:{...plan.accountSizes,altTreatment:'EXISTING_SUPPLIED_ACCOUNT_NO_NEW_ALLOCATION'},
      alt:{address:a.address,authority:state.authority,derivationRecentSlot:wrapped.initializer.transport.altRecentSlot,addresses:state.addresses,
        compilationState:'SUPPLIED_SNAPSHOT_ADDRESSES_ONLY_NOT_AUTHENTICATED_OR_FRESH',suppliedSnapshot:{...a,dataBytes:568,dataSha256:state.sha256},
        decodedState:{variant:state.variant,deactivationSlot:state.deactivationSlot,lastExtendedSlot:state.lastExtendedSlot,lastExtendedSlotStartIndex:state.lastExtendedSlotStartIndex,authority:state.authority},
        declaredStrictLaterSlotSatisfied:true,warmupVerified:false,authenticated:false,freshnessVerified:false,creationSlotHashesMembershipRequired:false,
        preconditions:['Active existing-table reuse does not require its historical derivation slot in current SlotHashes.',
          'Supplied context/reference must be strictly later than lastExtendedSlot; this is only an unauthenticated declaration check.',
          'Independently refresh exact owner/active state/authority/all16 ordered addresses before future signed execution. No ALT management or setup is emitted.']},
      referenceContext:{...wrapped.initializer.transport,verified:false,altContextSlot:a.contextSlot,
        usage:'SUPPLIED_SNAPSHOT_ENCODING_ONLY; independently refresh blockhash/lifetime and actual ALT/state before any future signed step. No automatic context/anchor replacement.'},
      preconditions:[...plan.preconditions,'The existing ALT raw public snapshot is not authenticated, current, rent-health, funding or eligibility proof. No creation/extension charge or historical rent refund is planned.']}};
}
function verifySources(){const original=base.verifySources();for(const [file,want]of Object.entries(SOURCE_PINS)){const p=path.isAbsolute(file)?file:path.resolve(__dirname,'../..',file);assert.equal(sha(fs.readFileSync(p)),want,`source binding: ${file}`);}return {node:original.node,web3:original.web3,originalPins:original.pins,pins:SOURCE_PINS};}
function main(args){const sourceCheck=verifySources();if(args.length===1&&args[0]==='--template'){process.stdout.write(JSON.stringify(TEMPLATE,null,2)+'\n');return;}if(args.length!==2||args[0]!=='--manifest')fail('unsupported_arguments','arguments');const file=path.resolve(args[1]),stat=fs.statSync(file);if(!stat.isFile())fail('public_manifest_file_required','manifest');if(stat.size>MAX_BYTES)fail('manifest_size','manifest');const report=planManifest(base.parseManifest(fs.readFileSync(file,'utf8')));report.sourceCheck=sourceCheck;process.stdout.write(JSON.stringify(report,null,2)+'\n');}
module.exports={SCHEMA,BASELINE,SOURCE_PINS,ALT_LAYOUT_REFERENCE,TEMPLATE,PublicKey,parseManifest:base.parseManifest,validateManifest,decodeAltData,planManifest,verifySources};
if(require.main===module){try{main(process.argv.slice(2));}catch(e){process.stderr.write(JSON.stringify({status:'INVALID_OR_UNSUPPORTED_PLAN',code:e.code||'plan_validation_failed',field:e.field||'manifest'})+'\n');process.exitCode=1;}}
