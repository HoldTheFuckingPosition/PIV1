'use strict';
// Stored facts from supplied bytes only; no live state, full economics or actions.
const assert=require('node:assert/strict');
const fs=require('node:fs');
const path=require('node:path');
const {createHash}=require('node:crypto');
const initializer=require('./initializer-plan.cjs');
const {PublicKey}=initializer;
const SOURCE_PINS=Object.freeze({
  "programs/piv1/src/state/config.rs": "9d7a57c87b6781b943e0e9c13d08f73a21058d400b40540c485e6c596b042198",
  "programs/piv1/src/state/distribution.rs": "a291d3fb94921cd036161ee669190074fd5f159fe0d9f523a98ef3b0fa25d3a3",
  "programs/piv1/src/accounts.rs": "08e3d28971160910088a40d74215d15a9c2d07b11142e28c4fb5162d725f1c80",
  "programs/piv1/src/state_persistence.rs": "d26483f7e1b95a020623b85705ab0006cb884e85cff37a0666bafa83123401a5",
  "programs/piv1/src/constants.rs": "2498c79c42d732be84aae423cf31d82997574cc08cca02fb226fea9acaa24f94",
  "programs/piv1/src/guardian_clock_accounts.rs": "5cc36b79d429bb3e1818466d429259474c8a26d1c1288da16a71f22033f84946",
  "programs/piv1/src/state/reconciliation.rs": "0604726bc0470664d90da7040bccca34d2a062d3bfa02dc546d8b02214afc386",
  "validation/genesis-bank-runtime/tests/lifecycle_fixture.rs": "84e9f5a735fd33c4c5a92cf7833b15ade7a95d952d014cb95d95a2e93c3fddf4",
  "clients/cli/initializer-plan.cjs": "347d5ab436b3e8878fd337c53ef8636721ca64a6465bc86c06f2637921ac6f6f"
});
const BASELINE='454fc5988f523693c1cb401a37d794d3c80e7200';
const TEMPLATE={
  "schema": "piv1-state-inspection-v1",
  "cluster": "testnet",
  "programId": null,
  "context": {
    "sourceKind": null,
    "sourceLabel": null,
    "slot": null
  },
  "config": {
    "address": null,
    "owner": null,
    "executable": null,
    "lamports": null,
    "dataBase64": null
  },
  "distribution": {
    "address": null,
    "owner": null,
    "executable": null,
    "lamports": null,
    "dataBase64": null
  }
};
const REPO=path.resolve(__dirname,'../..'),IDS=initializer.IDS,U64=(1n<<64n)-1n;
const CONFIG=[
  ["version", "u8"],
  ["is_initialized", "bool"],
  ["paused", "bool"],
  ["bumps", "bumps"],
  ["stake_pool_program", "key"],
  ["stake_pool", "key"],
  ["validator_list", "key"],
  ["reserve_stake", "key"],
  ["jitosol_mint", "key"],
  ["token_program", "key"],
  ["stake_program", "key"],
  ["system_program", "key"],
  ["manager_fee_account", "key"],
  ["referrer_token_account", "key"],
  ["piv_authority", "key"],
  ["active_distribution", "key"],
  ["principal_jito_vault", "key"],
  ["pending_jito_vault", "key"],
  ["pending_sol_vault", "key"],
  ["principal_sol_queue", "key"],
  ["operational_sol_vault", "key"],
  ["distribution_escrow", "key"],
  ["kif_sol_vault", "key"],
  ["htfp_recipient", "key"],
  ["team_owner_recipient", "key"],
  ["guardian_registry", "key"],
  ["basis_points_denominator", "u16"],
  ["htfp_reserve_bps", "u16"],
  ["permanent_compound_bps", "u16"],
  ["team_owner_pool_bps", "u16"],
  ["kif_bps", "u16"],
  ["configured_slippage_bps", "u16"],
  ["slippage_hard_cap_bps", "u16"],
  ["minimum_distribution_interval_seconds", "i64"],
  ["insufficient_retry_cooldown_seconds", "i64"],
  ["last_successful_preparation_at", "option_i64"],
  ["last_valid_insufficient_attempt_at", "option_i64"],
  ["next_distribution_sequence", "u64"],
  ["protected_principal_hwm_lamports", "u64"],
  ["accounted_historical_jitosol_units", "u64"],
  ["accounted_historical_sol_lamports", "u64"],
  ["accounted_pending_jitosol_units", "u64"],
  ["accounted_pending_sol_lamports", "u64"],
  ["next_cycle_yield_lamports", "u64"],
  ["kif_claim_liability_lamports", "u64"],
  ["collective_kif_carry_lamports", "u64"],
  ["cumulative_contribution_value_lamports", "u64"],
  ["cumulative_gross_yield_lamports", "u64"],
  ["cumulative_htfp_paid_lamports", "u64"],
  ["cumulative_team_owner_paid_lamports", "u64"],
  ["cumulative_kif_credited_lamports", "u64"],
  ["cumulative_kif_claimed_lamports", "u64"],
  ["cumulative_permanent_compound_lamports", "u64"],
  ["cumulative_retained_dust_lamports", "u64"],
  ["cumulative_zero_active_kif_compound_lamports", "u64"],
  ["cumulative_cooldown_yield_recorded_lamports", "u64"],
  ["kif_anchor_timestamp", "i64"],
  ["kif_period_seconds", "i64"],
  ["guardian_registry_revision", "u64"],
  ["migration_reserve", "reserve"],
],ROUND=[
  ["version", "u8"],
  ["bump", "u8"],
  ["is_initialized", "bool"],
  ["lifecycle", "phase"],
  ["recovery_flags", "u8"],
  ["settlement_recorded", "bool"],
  ["active_sequence", "u64"],
  ["last_completed", "summary"],
  ["prepared_at", "i64"],
  ["prepared_slot", "u64"],
  ["prepared_epoch", "u64"],
  ["old_protected_principal_lamports", "u64"],
  ["historical_jitosol_units", "u64"],
  ["historical_sol_lamports", "u64"],
  ["historical_value_lamports", "u64"],
  ["snapshot_pool_total_lamports", "u64"],
  ["snapshot_pool_token_supply", "u64"],
  ["snapshot_withdrawal_fee_numerator", "u64"],
  ["snapshot_withdrawal_fee_denominator", "u64"],
  ["gross_yield_lamports", "u64"],
  ["prior_next_cycle_yield_lamports", "u64"],
  ["htfp_gross_obligation_lamports", "u64"],
  ["permanent_compound_lamports", "u64"],
  ["team_owner_gross_obligation_lamports", "u64"],
  ["kif_gross_obligation_lamports", "u64"],
  ["split_dust_lamports", "u64"],
  ["outgoing_gross_obligation_lamports", "u64"],
  ["pending_sol_snapshot_lamports", "u64"],
  ["pending_sol_used_lamports", "u64"],
  ["snapshot_conversion_dust_lamports", "u64"],
  ["fixed_jitosol_withdrawal_target_units", "u64"],
  ["snapshot_leg_input_floor_units", "u64"],
  ["maximum_useful_legs", "u64"],
  ["stored_round_minimum_native_lamports", "u64"],
  ["stored_residual_hwm_floor_lamports", "u64"],
  ["stored_slippage_bps", "u16"],
  ["cumulative_jitosol_assigned_units", "u64"],
  ["cumulative_withdrawal_fee_units", "u64"],
  ["cumulative_burned_units", "u64"],
  ["cumulative_expected_native_lamports", "u64"],
  ["cumulative_delegated_native_lamports", "u64"],
  ["cumulative_finalized_delegated_native_lamports", "u64"],
  ["cumulative_finalized_native_lamports", "u64"],
  ["cumulative_recovered_stake_rent_lamports", "u64"],
  ["cumulative_recovered_metadata_rent_lamports", "u64"],
  ["cumulative_cooldown_rewards_lamports", "u64"],
  ["cumulative_cooldown_losses_lamports", "u64"],
  ["next_leg_index", "u64"],
  ["successful_leg_count", "u64"],
  ["finalized_leg_count", "u64"],
  ["recorded_escrow_available_lamports", "u64"],
  ["outstanding_active_round_liability_lamports", "u64"],
  ["htfp_recipient", "key"],
  ["team_owner_recipient", "key"],
  ["guardian_registry", "key"],
  ["guardian_registry_revision", "u64"],
  ["guardian_keys", "keys6"],
  ["kif_eligibility_bitmap", "u8"],
  ["kif_active_guardian_count", "u8"],
  ["kif_period_id", "u64"],
  ["kif_carry_input_lamports", "u64"],
  ["proposed_hwm_delta_lamports", "u64"],
  ["proposed_hwm_after_settlement_lamports", "u64"],
  ["actual_net_available_lamports", "u64"],
  ["actual_htfp_lamports", "u64"],
  ["actual_team_owner_lamports", "u64"],
  ["actual_kif_allocation_lamports", "u64"],
  ["actual_net_allocation_dust_lamports", "u64"],
  ["actual_allocated_outgoing_lamports", "u64"],
  ["actual_escrow_remainder_lamports", "u64"],
  ["actual_retained_conservative_dust_lamports", "u64"],
  ["actual_kif_liability_lamports", "u64"],
  ["actual_kif_carry_next_lamports", "u64"],
  ["actual_zero_active_kif_compound_lamports", "u64"],
  ["actual_hwm_delta_lamports", "u64"],
  ["settled_protected_hwm_lamports", "u64"],
],SUMMARY=[
  ["sequence", "u64"],
  ["completed_at", "i64"],
  ["gross_yield_lamports", "u64"],
  ["actual_allocated_outgoing_lamports", "u64"],
  ["integrated_contribution_value_lamports", "u64"],
  ["final_protected_hwm_lamports", "u64"],
  ["fixed_jitosol_withdrawal_target_units", "u64"],
  ["successful_leg_count", "u64"],
  ["cumulative_cooldown_rewards_lamports", "u64"],
  ["actual_kif_liability_lamports", "u64"],
  ["actual_kif_carry_next_lamports", "u64"],
];
const BUMPS=[
  ["config", "u8"],
  ["piv_authority", "u8"],
  ["active_distribution", "u8"],
  ["principal_jito_vault", "u8"],
  ["pending_jito_vault", "u8"],
  ["pending_sol_vault", "u8"],
  ["principal_sol_queue", "u8"],
  ["operational_sol_vault", "u8"],
  ["distribution_escrow", "u8"],
  ["kif_sol_vault", "u8"],
  ["guardian_registry", "u8"],
];
const PHASES=['Idle','WithdrawalActive','EscrowFunded','Settled','RecoveryRequired'];
const PDA_SEEDS={config:'config',piv_authority:'authority',active_distribution:'distribution',principal_jito_vault:'principal-jito-vault',
  pending_jito_vault:'pending-jito-vault',pending_sol_vault:'pending-sol',principal_sol_queue:'principal-sol',operational_sol_vault:'operational-sol',
  distribution_escrow:'distribution-escrow',kif_sol_vault:'kif-sol',guardian_registry:'guardian-registry'};
const OPEN=Object.freeze({publicReady:false,deploymentAuthorized:false,fullRustStateValidation:false,provenanceAuthenticated:false,
  snapshotCoherenceVerified:false,liveFreshnessVerified:false,custodyVerified:false,rentHealthVerified:false,eligibility:null,valuation:null,
  fees:null,funding:null,resourceSufficiency:null,m4Reservation:'OPEN_UNWAIVED'});
const LIMITS=[
  'Only supplied metadata/bytes and explicitly listed layout/PDA references are checked. No RPC, signature, origin, coherent snapshot or live freshness is authenticated.',
  'Full Rust Config/distribution validation and custody-state-binding economics are NOT implemented: phase normalization, counters/sequence, HWM/carry/liability math and valuation remain unverified.',
  'Stored pending/history/KIF amounts are ledgers, not physical vault balances, solvency, available funds or claimability. No custody or current Rent/pool/Clock data is inspected.',
  'Frozen round recipients/guardian keys/revision are separate stored snapshots and may differ from current Config after updates. No recipient rotation or guardian authorization is inferred.',
  'Timestamps remain signed stored integers, not current deadlines or action eligibility. RecoveryRequired calls for stopping/reporting, not an invented recovery command.',
  'No transaction packets, account creation, staking, keys, wallet/config/secret reads, signing, sending or deployment are performed or authorized.',
];
const sha=b=>createHash('sha256').update(b).digest('hex');
const fail=(code,field)=>{const e=new Error(code);e.code=code;e.field=field;throw e;};
const plain=v=>v!==null&&typeof v==='object'&&!Array.isArray(v)&&Object.getPrototypeOf(v)===Object.prototype;
class Reader {
  constructor(bytes,role){this.bytes=bytes;this.p=8;this.role=role;}
  take(n){if(this.p+n>this.bytes.length)fail('truncated_state_field',this.role);const b=this.bytes.subarray(this.p,this.p+n);this.p+=n;return b;}
  option(read){const tag=this.read('u8');if(tag>1)fail('invalid_option_tag',this.role);return tag===0?null:read();}
  record(fields){return Object.fromEntries(fields.map(([name,type])=>[name,this.read(type)]));}
  read(type){
    if(type==='u8')return this.take(1)[0];
    if(type==='u16')return this.take(2).readUInt16LE();
    if(type==='u64')return this.take(8).readBigUInt64LE().toString();
    if(type==='i64')return this.take(8).readBigInt64LE().toString();
    if(type==='bool'){const n=this.read('u8');if(n>1)fail('invalid_bool_tag',this.role);return n===1;}
    if(type==='key')return new PublicKey(this.take(32)).toBase58();
    if(type==='option_i64')return this.option(()=>this.read('i64'));
    if(type==='summary')return this.option(()=>this.record(SUMMARY));
    if(type==='bumps')return this.record(BUMPS);
    if(type==='keys6')return Array.from({length:6},()=>this.read('key'));
    if(type==='phase'){const tag=this.read('u8');if(tag>=PHASES.length)fail('unsupported_lifecycle_tag',this.role);return PHASES[tag];}
    if(type==='reserve'){const b=this.take(64);if(b.some(n=>n!==0))fail('nonzero_migration_reserve',this.role);return [...b];}
    throw new Error('internal state layout type');
  }
}
function decode(bytes,role) {
  const config=role==='config',size=config?1014:891,discriminator=config?[98,115,11,164,170,207,163,20]:[104,51,125,187,226,55,209,99];
  if(!Buffer.isBuffer(bytes)||bytes.length!==size)fail('account_size',role);
  if(!bytes.subarray(0,8).equals(Buffer.from(discriminator)))fail('state_discriminator',role);
  const reader=new Reader(bytes,role),value=reader.record(config?CONFIG:ROUND);
  if(value.version!==1||!value.is_initialized)fail('unsupported_state_header',role);
  if(bytes.subarray(reader.p).some(n=>n!==0))fail('nonzero_trailing_padding',role);
  return {value,layout:{allocationBytes:size,serializedPayloadBytes:reader.p-8,trailingZeroPaddingBytes:size-reader.p},sha256:sha(bytes)};
}
const decodeConfig=b=>decode(b,'config'),decodeDistribution=b=>decode(b,'distribution');
function deriveTopology(programId) {
  const program=new PublicKey(programId);
  return Object.fromEntries(Object.entries(PDA_SEEDS).map(([role,seed])=>{const [address,bump]=PublicKey.findProgramAddressSync([Buffer.from(seed)],program);return [role,{address:address.toBase58(),bump}];}));
}
function validateSnapshot(input) {
  function walk(value,shape,field){
    if(value===null||value===undefined)return null;
    if(plain(shape)){
      if(!plain(value))fail('object_required',field||'snapshot');
      if(Object.keys(value).some(k=>!Object.hasOwn(shape,k)))fail('unknown_field',field||'snapshot');
      return Object.fromEntries(Object.entries(shape).map(([k,v])=>[k,walk(value[k],v,field?`${field}.${k}`:k)]));
    }return value;
  }
  if(!plain(input))fail('object_required','snapshot');const m=walk(input,TEMPLATE,''),decoded={};
  const check=(v,field,fn)=>{if(v!==null&&v!==undefined)fn(v,field);};
  const key=(value,field)=>{if(typeof value!=='string')fail('public_key_string',field);let k;try{k=new PublicKey(value);}catch{fail('invalid_public_key',field);}
    if(k.toBase58()!==value||value===IDS.system)fail('noncanonical_or_zero_public_key',field);};
  const decimal=(value,field)=>{if(typeof value!=='string'||!/^(?:0|[1-9]\d*)$/.test(value))fail('decimal_string_required',field);if(BigInt(value)>U64)fail('integer_range',field);};
  check(m.schema,'schema',v=>{if(v!==TEMPLATE.schema)fail('unsupported_schema','schema');});
  check(m.cluster,'cluster',v=>{if(v!=='testnet')fail('unsupported_cluster','cluster');});
  check(m.programId,'programId',key);if(m.programId!==null&&Object.values(IDS).includes(m.programId))fail('unsupported_program_identity','programId');
  if(m.context){
    check(m.context.sourceKind,'context.sourceKind',v=>{if(!['provided-public-snapshot','retained-local-bank','synthetic-literal'].includes(v))fail('unsupported_source_kind','context.sourceKind');});
    check(m.context.sourceLabel,'context.sourceLabel',v=>{if(typeof v!=='string'||v.length<1||v.length>512)fail('source_label_string','context.sourceLabel');});
    check(m.context.slot,'context.slot',decimal);
  }
  for(const role of ['config','distribution'])if(m[role]){
    const a=m[role];for(const field of ['address','owner'])check(a[field],`${role}.${field}`,key);
    check(a.executable,`${role}.executable`,v=>{if(v!==false)fail('executable_state_account',`${role}.executable`);});
    check(a.lamports,`${role}.lamports`,decimal);
    if(m.programId!==null&&a.owner!==null&&a.owner!==m.programId)fail('declared_owner_mismatch',`${role}.owner`);
    check(a.dataBase64,`${role}.dataBase64`,v=>{
      if(typeof v!=='string')fail('base64_string_required',`${role}.dataBase64`);
      const bytes=Buffer.from(v,'base64');if(bytes.toString('base64')!==v)fail('noncanonical_base64',`${role}.dataBase64`);decoded[role]=decode(bytes,role);
    });
  }
  if(m.config?.address!=null&&m.config.address===m.distribution?.address)fail('state_account_alias','distribution.address');
  const c=decoded.config?.value,round=decoded.distribution?.value;
  if(c)for(const [field,id]of [['system_program',IDS.system],['token_program',IDS.token],['stake_program',IDS.stake]])if(c[field]!==id)fail('fixed_program_binding',`config.${field}`);
  const topology=m.programId===null?null:deriveTopology(m.programId);
  if(topology){
    for(const role of ['config','distribution']){
      const expected=topology[role==='config'?'config':'active_distribution'];
      if(m[role]?.address!=null&&m[role].address!==expected.address)fail('canonical_state_pda',`${role}.address`);
      const bump=role==='config'?c?.bumps.config:round?.bump;if(bump!==undefined&&bump!==expected.bump)fail('canonical_state_bump',role);
    }
    if(c)for(const [role,expected]of Object.entries(topology))if(role!=='config'&&(c[role]!==expected.address||c.bumps[role]!==expected.bump))fail('config_topology_binding',`config.${role}`);
  }
  if(c&&m.distribution?.address!=null&&c.active_distribution!==m.distribution.address)fail('distribution_reference','config.active_distribution');
  if(c&&round&&c.bumps.active_distribution!==round.bump)fail('distribution_bump_reference','distribution.bump');
  const fields=['schema','cluster','programId',...Object.keys(TEMPLATE.context).map(k=>`context.${k}`),...['config','distribution'].flatMap(role=>Object.keys(TEMPLATE[role]).map(k=>`${role}.${k}`))];
  const missing=fields.filter(field=>field.split('.').reduce((v,k)=>v?.[k],m)==null);
  return {snapshot:m,missing,decoded,topology};
}
const pick=(v,fields)=>Object.fromEntries(fields.map(k=>[k,v[k]]));
function inspectSnapshot(input) {
  const v=validateSnapshot(input),common={schema:TEMPLATE.schema,sourceBaseline:BASELINE,missing:v.missing,readiness:{...OPEN},limitations:LIMITS};
  if(v.missing.length)return {...common,status:'INCOMPLETE',inspection:null};
  const c=v.decoded.config.value,d=v.decoded.distribution.value;
  return {...common,status:'OFFLINE_STATE_INSPECTION',inspection:{context:{...v.snapshot.context,authenticated:false},program:v.snapshot.programId,
    decoded:{config:c,distribution:d},accounts:Object.fromEntries(['config','distribution'].map(role=>{const {dataBase64,...metadata}=v.snapshot[role];return [role,{...metadata,layout:v.decoded[role].layout,sha256:v.decoded[role].sha256}];})),
    structuralChecksPassed:['Declared owner==declared program; nonexecutable state metadata','Exact1014/891 bytes, discriminators, V1/initialized and strict Borsh bool/Option/enum tags',
      'Config migration reserve and unused allocation tail are zero','Canonical declared Config/distribution addresses and bumps','Config current fixed PDA topology and canonical bumps',
      'Config System/Token/Stake ID constants; other external bindings decoded only','Config.active_distribution points to declared distribution with matching bump'],
    summary:{config:{paused:c.paused,hwmLamports:c.protected_principal_hwm_lamports,
      history:pick(c,['accounted_historical_sol_lamports','accounted_historical_jitosol_units']),pending:pick(c,['accounted_pending_sol_lamports','accounted_pending_jitosol_units']),
      kif:pick(c,['kif_claim_liability_lamports','collective_kif_carry_lamports','kif_anchor_timestamp','kif_period_seconds','guardian_registry_revision']),
      timing:pick(c,['minimum_distribution_interval_seconds','insufficient_retry_cooldown_seconds','last_successful_preparation_at','last_valid_insufficient_attempt_at','next_distribution_sequence']),
      nextCycleYieldLamports:c.next_cycle_yield_lamports,cumulativeTotals:Object.fromEntries(Object.entries(c).filter(([k])=>k.startsWith('cumulative_'))),
      currentRecipients:pick(c,['htfp_recipient','team_owner_recipient']),currentRegistry:c.guardian_registry},
      distribution:{phase:d.lifecycle,activeSequence:d.active_sequence,recoveryFlags:d.recovery_flags,settlementRecorded:d.settlement_recorded,
        preparation:pick(d,['prepared_at','prepared_slot','prepared_epoch']),legCounters:pick(d,['fixed_jitosol_withdrawal_target_units','cumulative_jitosol_assigned_units','next_leg_index','successful_leg_count','finalized_leg_count']),
        escrowAndKif:pick(d,['recorded_escrow_available_lamports','outstanding_active_round_liability_lamports','actual_kif_liability_lamports','actual_kif_carry_next_lamports']),lastCompletion:d.last_completed,
        frozenBindings:pick(d,['htfp_recipient','team_owner_recipient','guardian_registry','guardian_registry_revision','guardian_keys','kif_eligibility_bitmap','kif_active_guardian_count','kif_period_id'])}}}};
}
function verifySources(){const existing=initializer.verifySources();for(const [file,expected]of Object.entries(SOURCE_PINS))assert.equal(sha(fs.readFileSync(path.join(REPO,file))),expected,`source binding: ${file}`);
  return {node:existing.node,web3:existing.web3,pins:SOURCE_PINS,initializerPins:existing.pins};}
function main(args){
  const sourceCheck=verifySources();if(args.length===1&&args[0]==='--template'){process.stdout.write(JSON.stringify(TEMPLATE,null,2)+'\n');return;}
  if(args.length!==2||args[0]!=='--snapshot')fail('unsupported_arguments','arguments');
  const filename=path.resolve(args[1]),stat=fs.statSync(filename);if(!stat.isFile()||stat.size>65536)fail('public_snapshot_file_required','snapshot');
  const bytes=fs.readFileSync(filename);if(bytes.length>65536)fail('snapshot_size','snapshot');
  const report=inspectSnapshot(initializer.parseManifest(bytes.toString('utf8')));report.snapshotInputSha256=sha(bytes);report.sourceCheck=sourceCheck;process.stdout.write(JSON.stringify(report,null,2)+'\n');
}
module.exports={BASELINE,SOURCE_PINS,TEMPLATE,decodeConfig,decodeDistribution,deriveTopology,validateSnapshot,inspectSnapshot,verifySources};
if(require.main===module){try{main(process.argv.slice(2));}catch(e){process.stderr.write(JSON.stringify({status:'INVALID_OR_UNSUPPORTED_STATE_INPUT',code:e.code||'inspection_failed',field:e.field||'snapshot'})+'\n');process.exitCode=1;}}
