'use strict';
const assert=require('node:assert/strict');
const fs=require('node:fs');
const path=require('node:path');
const os=require('node:os');
const {createHash}=require('node:crypto');
const {spawnSync}=require('node:child_process');
const test=require('node:test');
const s=require('./state-inspect.cjs');
const n=require('./initializer-plan.cjs');
const {PublicKey}=n;
const copy=v=>JSON.parse(JSON.stringify(v)),sha=b=>createHash('sha256').update(b).digest('hex');
const fixturePath=path.join(__dirname,'state-inspect.fixture.json');
const fixture=()=>JSON.parse(fs.readFileSync(fixturePath));
const MAX=(1n<<64n)-1n,MIN=-(1n<<63n),IMAX=(1n<<63n)-1n;
const bytes=(m,role)=>Buffer.from(m[role].dataBase64,'base64');
const put=(m,role,b)=>{m[role].dataBase64=b.toString('base64');return m;};
const number=(value,signed=false)=>{const b=Buffer.alloc(8);signed?b.writeBigInt64LE(BigInt(value)):b.writeBigUInt64LE(BigInt(value));return b;};
// Independent literal layout writer: byte offsets/counts are bound to Rust source,
// not to any exported implementation field tables or decoder-produced values.
function literalConfig(first=null,second=null,wide=false) {
  const original=bytes(fixture(),'config'),option=value=>value===null?Buffer.from([0]):Buffer.concat([Buffer.from([1]),number(value,true)]);
  const prefix=original.subarray(0,756),counters=Array.from({length:19},(_,i)=>number(wide?MAX:1001+i));
  const head=Buffer.concat([prefix,option(first),option(second),...counters,number(MIN,true),number(IMAX,true),number(MAX)]);
  const reserveAt=head.length,used=head.length+64,b=Buffer.alloc(1014);head.copy(b);
  return {b,reserveAt,used};
}
function literalRound(summary=false,phase=1) {
  const header=Buffer.from([104,51,125,187,226,55,209,99,1,bytes(fixture(),'distribution')[9],1,phase,255,1]);
  const completion=[];if(summary)for(let i=0;i<11;i++)completion.push(number(i===1?MIN:400+i,i===1));
  const snapshot=[];for(let i=0;i<27;i++)snapshot.push(number(i===0?MIN:100+i,i===0));
  const slip=Buffer.from([255,255]),cumulative=Array.from({length:16},(_,i)=>number(200+i));
  const keys=[17,18,19].map(i=>Buffer.alloc(32,i)),guardians=Array.from({length:6},(_,i)=>Buffer.alloc(32,21+i));
  const body=Buffer.concat([header,number(MAX),Buffer.from([Number(summary)]),...completion,...snapshot,slip,...cumulative,...keys,number(MAX),...guardians,
    Buffer.from([255,255]),number(12345),number(MAX),...Array.from({length:15},(_,i)=>number(300+i))]);
  const b=Buffer.alloc(891);body.copy(b);return {b,used:body.length};
}
function literalSnapshot(first=null,second=null,summary=false,phase=1) {
  const m=fixture();m.context={sourceKind:'synthetic-literal',sourceLabel:'Public literal boundary fixture, not economic validity',slot:MAX.toString()};
  put(m,'config',literalConfig(first,second).b);put(m,'distribution',literalRound(summary,phase).b);return m;
}

test('exact retained Bank fixture has independently reviewed final stored facts and scope limits',()=>{
  s.verifySources();assert.equal(sha(fs.readFileSync(fixturePath)),'0bdafa7694994c6ed66e5d48ae45dd1cf996455a9becfe6db9754e42e8c2b8f6');
  const p=s.inspectSnapshot(fixture()),{config:c,distribution:d}=p.inspection.decoded;
  assert.equal(p.status,'OFFLINE_STATE_INSPECTION');assert.equal(p.inspection.context.sourceKind,'retained-local-bank');assert.equal(p.inspection.context.slot,'524257');
  const expected={last_successful_preparation_at:'100',last_valid_insufficient_attempt_at:null,next_distribution_sequence:'1',protected_principal_hwm_lamports:'121955000003',
    accounted_historical_sol_lamports:'200000002',accounted_historical_jitosol_units:'111608750001',accounted_pending_sol_lamports:'0',accounted_pending_jitosol_units:'0',next_cycle_yield_lamports:'0',
    kif_claim_liability_lamports:'216149757',collective_kif_carry_lamports:'0',cumulative_contribution_value_lamports:'119845454545',cumulative_gross_yield_lamports:'10818181818',
    cumulative_htfp_paid_lamports:'6376417834',cumulative_team_owner_paid_lamports:'2107460131',cumulative_kif_credited_lamports:'216149757',cumulative_kif_claimed_lamports:'0',
    cumulative_permanent_compound_lamports:'2109545454',cumulative_retained_dust_lamports:'4',cumulative_zero_active_kif_compound_lamports:'0',cumulative_cooldown_yield_recorded_lamports:'0'};
  for(const [field,value]of Object.entries(expected))assert.equal(c[field],value,field);
  assert.equal(c.paused,false);assert.equal(d.lifecycle,'Idle');assert.equal(d.active_sequence,'0');
  assert.deepEqual(d.last_completed,{sequence:'0',completed_at:'100',gross_yield_lamports:'10818181818',actual_allocated_outgoing_lamports:'8700027722',integrated_contribution_value_lamports:'845454545',
    final_protected_hwm_lamports:'121955000003',fixed_jitosol_withdrawal_target_units:'7891249999',successful_leg_count:'2',cumulative_cooldown_rewards_lamports:'0',actual_kif_liability_lamports:'216149757',actual_kif_carry_next_lamports:'0'});
  assert.deepEqual(p.inspection.accounts.config.layout,{allocationBytes:1014,serializedPayloadBytes:998,trailingZeroPaddingBytes:8});
  assert.deepEqual(p.inspection.accounts.distribution.layout,{allocationBytes:891,serializedPayloadBytes:883,trailingZeroPaddingBytes:0});
  assert.equal(p.readiness.fullRustStateValidation,false);assert.equal(p.readiness.custodyVerified,false);assert.equal(p.readiness.rentHealthVerified,false);assert.equal(p.readiness.eligibility,null);
  assert.equal(p.readiness.publicReady,false);assert.equal(p.readiness.valuation,null);assert.equal(p.inspection.context.authenticated,false);
});
test('all4 Config Option combinations x2 summary shapes retain cursor positions and signed values',()=>{
  for(const first of [null,MIN])for(const second of [null,IMAX])for(const summary of [false,true]){
    const p=s.inspectSnapshot(literalSnapshot(first,second,summary)),c=p.inspection.decoded.config,d=p.inspection.decoded.distribution;
    const optionBytes=8*Number(first!==null)+8*Number(second!==null);
    assert.equal(c.last_successful_preparation_at,first===null?null:first.toString());assert.equal(c.last_valid_insufficient_attempt_at,second===null?null:second.toString());
    assert.equal(c.next_distribution_sequence,'1001');assert.equal(c.protected_principal_hwm_lamports,'1002');assert.equal(c.accounted_pending_sol_lamports,'1006');assert.equal(c.cumulative_cooldown_yield_recorded_lamports,'1019');
    assert.equal(c.kif_anchor_timestamp,MIN.toString());assert.equal(c.kif_period_seconds,IMAX.toString());assert.equal(c.guardian_registry_revision,MAX.toString());assert.deepEqual(c.migration_reserve,Array(64).fill(0));
    assert.deepEqual(p.inspection.accounts.config.layout,{allocationBytes:1014,serializedPayloadBytes:990+optionBytes,trailingZeroPaddingBytes:16-optionBytes});
    assert.equal(d.prepared_at,MIN.toString());assert.equal(d.prepared_slot,'101');assert.equal(d.stored_residual_hwm_floor_lamports,'126');assert.equal(d.stored_slippage_bps,65535);
    assert.equal(d.cumulative_jitosol_assigned_units,'200');assert.equal(d.finalized_leg_count,'213');assert.equal(d.recorded_escrow_available_lamports,'214');assert.equal(d.outstanding_active_round_liability_lamports,'215');
    assert.equal(d.guardian_registry_revision,MAX.toString());assert.equal(d.guardian_keys.length,6);assert.equal(d.kif_period_id,'12345');assert.equal(d.kif_carry_input_lamports,MAX.toString());
    assert.equal(d.proposed_hwm_delta_lamports,'300');assert.equal(d.settled_protected_hwm_lamports,'314');assert.equal(d.last_completed?.completed_at??null,summary?MIN.toString():null);
    assert.equal(d.last_completed?.actual_kif_carry_next_lamports??null,summary?'410':null);
    assert.deepEqual(p.inspection.accounts.distribution.layout,{allocationBytes:891,serializedPayloadBytes:summary?883:795,trailingZeroPaddingBytes:summary?0:88});
  }
});
test('all coarse phases/boolean states and full-u64 values remain stored facts, not economic validity',()=>{
  for(const [tag,name]of ['Idle','WithdrawalActive','EscrowFunded','Settled','RecoveryRequired'].entries()){
    const m=literalSnapshot(null,null,false,tag),c=literalConfig(null,null,true).b;c[10]=Number(tag%2===0);put(m,'config',c);
    const roundBytes=literalRound(false,tag).b; // None-summary:23+216+2+128+96=465 registry revision
    roundBytes.writeBigUInt64LE(MAX-1n,465);put(m,'distribution',roundBytes);
    const p=s.inspectSnapshot(m),d=p.inspection.decoded.distribution;
    assert.equal(d.lifecycle,name);assert.equal(p.inspection.decoded.config.paused,tag%2===0);assert.equal(p.inspection.decoded.config.protected_principal_hwm_lamports,MAX.toString());
    assert.equal(d.active_sequence,MAX.toString());assert.equal(d.recovery_flags,255);assert.equal(d.kif_eligibility_bitmap,255);assert.equal(d.kif_active_guardian_count,255);
    assert.notEqual(d.htfp_recipient,p.inspection.decoded.config.htfp_recipient);assert.equal(d.guardian_registry_revision,(MAX-1n).toString());assert.equal(p.inspection.decoded.config.guardian_registry_revision,MAX.toString());
    assert.notEqual(d.guardian_registry_revision,p.inspection.decoded.config.guardian_registry_revision);
    assert.equal(p.readiness.fullRustStateValidation,false);assert.equal(p.readiness.eligibility,null);assert.equal(p.inspection.accounts.config.sha256,sha(c));
  }
});
test('invalid Borsh tags/discriminators/reserve/tail and exact allocations reject',()=>{
  const changes=[['config',8,2],['config',9,2],['config',9,0],['config',10,2],['config',756,2],['config',757,2],
    ['distribution',8,2],['distribution',10,2],['distribution',10,0],['distribution',11,5],['distribution',13,2],['distribution',22,2],['config',0,0],['distribution',0,0]];
  for(const [role,at,value]of changes){const m=literalSnapshot(),b=bytes(m,role);b[at]=value;put(m,role,b);assert.throws(()=>s.inspectSnapshot(m));m.context=null;assert.throws(()=>s.inspectSnapshot(m));}
  for(const [role,at]of [['config',literalConfig().reserveAt],['config',literalConfig().used],['distribution',literalRound().used]]){
    const m=literalSnapshot(),b=bytes(m,role);b[at]=1;put(m,role,b);assert.throws(()=>s.inspectSnapshot(m));
  }
  const present=literalSnapshot(MIN,null);const b=bytes(present,'config');b[765]=2;put(present,'config',b);assert.throws(()=>s.inspectSnapshot(present));
  for(const role of ['config','distribution'])for(const delta of [-1,1]){const m=fixture(),b=bytes(m,role);put(m,role,delta===-1?b.subarray(0,-1):Buffer.concat([b,Buffer.from([0])]));assert.throws(()=>s.inspectSnapshot(m));}
});
test('supplied metadata, canonical PDAs/bumps and Config reference are checked without rent claims',()=>{
  const other=new PublicKey(Buffer.alloc(32,17)).toBase58();
  const changes=[m=>m.config.owner=other,m=>m.distribution.executable=true,m=>m.config.executable='false',m=>m.config.address=m.distribution.address,
    m=>m.config.address=other,m=>m.programId=n.IDS.token,m=>m.context.sourceKind='authenticated',m=>m.context.slot='01',m=>m.context.sourceLabel='',
    m=>m.config.lamports=1,m=>m.config.lamports='18446744073709551616',m=>m.config.dataBase64+='=',m=>m.distribution.dataBase64+='\n',m=>m.unknown='do-not-echo'];
  for(const change of changes){const m=fixture();change(m);assert.throws(()=>s.inspectSnapshot(m));}
  for(const [role,at]of [['config',11],['distribution',9]]){const m=fixture(),b=bytes(m,role);b[at]^=1;put(m,role,b);assert.throws(()=>s.inspectSnapshot(m));}
  const linked=fixture(),data=bytes(linked,'config');data.fill(17,22+11*32,22+12*32);put(linked,'config',data);assert.throws(()=>s.inspectSnapshot(linked));
  const empty=fixture();empty.config.lamports='0';empty.distribution.lamports=MAX.toString();const p=s.inspectSnapshot(empty);assert.equal(p.status,'OFFLINE_STATE_INSPECTION');assert.equal(p.readiness.rentHealthVerified,false);
});
test('missing snapshots stay incomplete while malformed known bytes/schema still reject',()=>{
  assert.deepEqual(JSON.parse(fs.readFileSync(path.join(__dirname,'state-inspect.example.json'))),s.TEMPLATE);
  for(const m of [copy(s.TEMPLATE),{}, {...fixture(),config:null},{...fixture(),distribution:null},{...fixture(),programId:null},{...fixture(),context:null}]){
    const p=s.inspectSnapshot(m);assert.equal(p.status,'INCOMPLETE');assert.equal(p.inspection,null);assert(p.missing.length>0);
  }
  const known=fixture();known.distribution=null;known.config.dataBase64='bad';assert.throws(()=>s.inspectSnapshot(known));
  for(const source of ['{"config":null,"config":null}','{"programId":null,"program\\u0049d":null}','[]','{"unknown":"do-not-echo"}'])assert.throws(()=>s.inspectSnapshot(n.parseManifest(source)));
});
test('CLI template/null/retained/invalid input paths retain a complete public report',context=>{
  const script=path.join(__dirname,'state-inspect.cjs'),limits=process.execArgv.filter(v=>v.startsWith('--max-old-space-size'));
  const run=args=>spawnSync(process.execPath,[...limits,script,...args],{encoding:'utf8',maxBuffer:2*1024*1024,timeout:30000});
  const template=run(['--template']);assert.equal(template.status,0);assert.equal(template.stderr,'');assert.deepEqual(JSON.parse(template.stdout),s.TEMPLATE);
  const incomplete=run(['--snapshot',path.join(__dirname,'state-inspect.example.json')]);assert.equal(incomplete.status,0);assert.equal(JSON.parse(incomplete.stdout).inspection,null);
  const directory=fs.mkdtempSync(path.join(os.tmpdir(),'piv1-state-public-test-')),snapshot=path.join(directory,'retained-snapshot.json');fs.copyFileSync(fixturePath,snapshot,fs.constants.COPYFILE_EXCL);
  const complete=run(['--snapshot',snapshot]);fs.writeFileSync(path.join(directory,'retained-report.json'),complete.stdout,{flag:'wx'});assert.equal(complete.status,0);assert.equal(complete.stderr,'');
  const parsed=JSON.parse(complete.stdout);assert.deepEqual(parsed.inspection,s.inspectSnapshot(fixture()).inspection);assert.equal(parsed.status,'OFFLINE_STATE_INSPECTION');assert.equal(parsed.snapshotInputSha256,sha(fs.readFileSync(snapshot)));
  assert.equal(parsed.readiness.fullRustStateValidation,false);assert.equal(parsed.readiness.snapshotCoherenceVerified,false);
  const badFile=path.join(directory,'invalid-public.json');fs.writeFileSync(badFile,'{"unknown":"do-not-echo"}',{flag:'wx'});
  for(const args of [[],['--rpc'],['--template','--send'],['--snapshot'],['--snapshot',badFile]]){const bad=run(args);assert.notEqual(bad.status,0);assert.equal(bad.stdout,'');assert.equal(JSON.parse(bad.stderr).status,'INVALID_OR_UNSUPPORTED_STATE_INPUT');assert(!bad.stderr.includes('do-not-echo'));}
  context.diagnostic(`Public-only retained snapshot/report: ${directory}`);
});
