'use strict';

const {IDS,PublicKey}=require('./initializer-plan.cjs');
const {validateSnapshot}=require('./state-inspect.cjs');

const SCHEMA='piv1-withdrawal-source-selection-v1';
const CLOCK='SysvarC1ock11111111111111111111111111111111';
const RENT='SysvarRent111111111111111111111111111111111';
const SYSVAR='Sysvar1111111111111111111111111111111111111';
const U64=(1n<<64n)-1n;
const poolKey=new PublicKey(IDS.pool),poolProgram=new PublicKey(IDS.poolProgram);
const [withdrawKey,withdrawBump]=PublicKey.findProgramAddressSync([poolKey.toBuffer(),Buffer.from('withdraw')],poolProgram);
const min=(a,b)=>a<b?a:b;
const max=(a,b)=>a>b?a:b;
const ceil=(n,d)=>(n+d-1n)/d;
const key=bytes=>new PublicKey(bytes).toBase58();
const u64=(bytes,offset)=>bytes.readBigUInt64LE(offset);

function requireFact(ok,code,field) {
  if(ok)return;
  const error=new Error(code);
  error.code=code;
  error.field=field;
  throw error;
}

function decimal(value,field) {
  requireFact(typeof value==='string'&&/^(?:0|[1-9]\d{0,19})$/.test(value),'invalid_decimal',field);
  const n=BigInt(value);
  requireFact(n<=U64,'integer_range',field);
  return n;
}

function account(raw,address,owner,field) {
  requireFact(raw?.address===address&&raw.owner===owner&&raw.executable===false,'account_binding',field);
  requireFact(typeof raw.dataBase64==='string','account_data',field);
  const data=Buffer.from(raw.dataBase64,'base64');
  requireFact(data.toString('base64')===raw.dataBase64,'account_data',field);
  return {data,balance:decimal(raw.lamports,field)};
}

// SPL Stake Pool 2.0.3 Borsh layout. Keep the variable-width option and fee
// fields on the same cursor, including fields after the current withdrawal fee.
function decodePool(data,config,epoch) {
  requireFact(data.length>=435&&data[0]===1,'invalid_pool','pool');
  const publicKeyAt=offset=>key(data.subarray(offset,offset+32));
  requireFact(data[97]===withdrawBump&&publicKeyAt(98)===config.validator_list&&
    publicKeyAt(130)===config.reserve_stake&&publicKeyAt(162)===IDS.mint&&
    publicKeyAt(194)===config.manager_fee_account&&publicKeyAt(226)===IDS.token&&
    u64(data,274)===epoch&&data.subarray(282,330).every(byte=>byte===0),'pool_state_mismatch','pool');
  const total=u64(data,258),supply=u64(data,266);
  requireFact(total>0n&&supply>0n&&total+supply<=U64,'pool_ratio','pool');
  let offset=330;
  const skip=length=>{
    const start=offset;
    offset+=length;
    requireFact(offset<=data.length,'truncated_pool','pool');
    return start;
  };
  const fee=()=>{
    const start=skip(16);
    const denominator=u64(data,start),numerator=u64(data,start+8);
    requireFact(numerator<=denominator,'invalid_pool_fee','pool');
    return {numerator,denominator};
  };
  const tag=(limit,code)=>{
    const value=data[skip(1)];
    requireFact(value<=limit,code,'pool');
    return value;
  };
  const optionKey=()=>tag(1,'invalid_pool_option')===1?publicKeyAt(skip(32)):null;
  const futureFee=()=>{if(tag(2,'invalid_pool_future_fee')!==0)fee();};
  fee();futureFee();optionKey();
  const preferred=optionKey();
  fee();
  const withdrawalFee=fee();
  futureFee();tag(100,'pool_percentage');optionKey();fee();
  tag(100,'pool_percentage');optionKey();fee();futureFee();skip(16);
  return {total,supply,preferred,...withdrawalFee};
}

function decodeList(data) {
  requireFact(data.length>=9&&data[0]===2,'invalid_validator_list','validatorList');
  const maximum=data.readUInt32LE(1),count=data.readUInt32LE(5);
  requireFact(maximum>0&&count<=maximum&&maximum===Math.floor((data.length-9)/73),
    'validator_list_geometry','validatorList');
  return Array.from({length:count},(_,index)=>{
    const start=9+73*index,vote=data.subarray(start+41,start+73);
    return {index,vote:key(vote),voteBytes:vote,balance:u64(data,start),epoch:u64(data,start+16),
      suffix:data.subarray(start+36,start+40),status:data[start+40]};
  });
}

function sourceFor(row,raw,epoch,stakeRent,runtimeMinimum) {
  if(row.status!==0||row.epoch!==epoch||row.vote===IDS.system)return null;
  const seeds=[row.voteBytes,poolKey.toBuffer()];
  if(row.suffix.readUInt32LE()!==0)seeds.push(row.suffix);
  const address=PublicKey.findProgramAddressSync(seeds,poolProgram)[0].toBase58();
  const {data,balance}=account(raw,address,IDS.stake,'sourceAccounts');
  const authority=withdrawKey.toBuffer();
  requireFact(data.length===200&&data.readUInt32LE()===2&&data.subarray(12,44).equals(authority)&&
    data.subarray(44,76).equals(authority)&&data.subarray(76,124).every(byte=>byte===0)&&
    data.subarray(124,156).equals(row.voteBytes)&&u64(data,172)===U64&&
    (u64(data,164)===U64||u64(data,164)<=epoch)&&data[196]===0,'invalid_stake_source','sourceAccounts');
  const delegated=u64(data,156);
  requireFact(balance===row.balance&&delegated>0n&&balance-delegated>=stakeRent,
    'stake_balance_mismatch','sourceAccounts');
  return {index:row.index,address,vote:row.vote,balance,delegated,
    required:min(U64,u64(data,4)+max(runtimeMinimum,1_000_000n))};
}

function ratio({total,supply,numerator,denominator}) {
  const book=units=>units*total/supply;
  const fee=units=>numerator===0n?0n:ceil(units*numerator,denominator);
  const redeem=units=>book(units-fee(units));
  const first=(lo,hi,predicate)=>{
    while(lo<hi) {
      const middle=(lo+hi)/2n;
      if(predicate(middle))hi=middle;
      else lo=middle+1n;
    }
    return lo;
  };
  return {book,fee,redeem,minimum:(native,mint)=>redeem(mint)<native?null:first(1n,mint,q=>redeem(q)>=native),
    capacity:(native,mint)=>first(0n,mint+1n,q=>redeem(q)>native)-1n};
}

function assess(source,state,facts) {
  const {total,supply,mint,preferredBalance,remaining,legCount,cumulative,pool}=state;
  const threshold=min(U64,source.required+ceil(total,supply));
  if(pool.preferred!==null&&pool.preferred!==source.vote&&
    (preferredBalance===null||preferredBalance>threshold))return {reason:'PREFERRED_SOURCE_REQUIRED'};
  if(source.balance<=threshold||source.delegated<facts.runtimeMinimum)return {reason:'SOURCE_BELOW_SPL_THRESHOLD'};
  const current=ratio({...pool,total,supply});
  const currentMinimum=current.minimum(facts.runtimeMinimum,mint);
  if(currentMinimum===null)return {reason:'NATIVE_MINIMUM_UNAVAILABLE'};
  const available=min(source.balance-source.required,source.delegated-facts.runtimeMinimum);
  const capacity=current.capacity(available,mint);
  const input=min(remaining,capacity),remainder=remaining-input;
  const minimum=max(facts.snapshotMinimum,currentMinimum);
  if(input<minimum)return {reason:'SOURCE_BELOW_LEG_MINIMUM',capacity};
  if(remainder>0n&&remainder<minimum)return {reason:'MAXIMUM_FILL_STRANDS_REMAINDER',capacity,remainder};
  const fee=current.fee(input),burn=input-fee,output=current.redeem(input);
  const after={...state,total:total-output,supply:supply-burn,mint:mint-burn,remaining:remainder,
    legCount:legCount+1n,cumulative:cumulative+output};
  if(pool.preferred===source.vote&&facts.preferredIndex===source.index)after.preferredBalance-=output;
  if(output>U64||after.total<=0n||after.supply<=0n||after.mint<0n||after.cumulative>U64||
    after.legCount>U64)return {reason:'POST_POOL_RANGE',capacity};
  const slots=facts.maxLegs-after.legCount;
  if(slots<0n||(remainder!==0n&&slots===0n))return {reason:'USEFUL_LEG_BOUND',capacity};
  const next=ratio({...pool,total:after.total,supply:after.supply});
  let future=0n;
  if(remainder!==0n) {
    const nextMinimum=next.minimum(facts.runtimeMinimum,after.mint);
    if(nextMinimum===null||remainder<max(nextMinimum,facts.snapshotMinimum))
      return {reason:'POST_RATIO_LEG_MINIMUM',capacity,remainder};
    const legs=min(slots,remainder/facts.snapshotMinimum),reserve=pool.numerator===0n?0n:legs-1n;
    const nextBurn=remainder-next.fee(remainder)-reserve;
    future=nextBurn<0n?-1n:next.book(nextBurn)-(legs-1n);
  }
  if(future<0n||after.cumulative+future>U64||after.cumulative+future<facts.roundMinimum)
    return {reason:'ROUND_MINIMUM',capacity};
  const residual=facts.residualNative+next.book(facts.residualUnits);
  if(residual>U64||residual<facts.residualFloor)return {reason:'RESIDUAL_HWM',capacity};
  const minimumOutput=max(facts.runtimeMinimum,output*(10_000n-facts.slippage)/10_000n);
  return {source,input,capacity,output,minimumOutput,remainder,after};
}

const describe=quote=>({validatorIndex:String(quote.source.index),sourceStake:quote.source.address,
  capacityUnits:String(quote.capacity),maximumFillUnits:String(quote.input),expectedNativeLamports:String(quote.output),
  protectedMinimumNativeLamports:String(quote.minimumOutput),remainingAfterUnits:String(quote.remainder)});

function selectSource(input) {
  requireFact(input?.schema===SCHEMA,'unsupported_schema','schema');
  const p=validateSnapshot(input.state);
  requireFact(p.missing.length===0,'incomplete_state','state');
  const {snapshot,decoded}=p,c=decoded.config.value,r=decoded.distribution.value;
  const slot=BigInt(snapshot.context.slot);
  const clock=account(input.clock,CLOCK,SYSVAR,'clock').data;
  requireFact(clock.length===40&&u64(clock,0)===slot&&clock.readBigInt64LE(32)>=0n,'clock_context_mismatch','clock');
  const epoch=u64(clock,16),now=clock.readBigInt64LE(32);
  const rent=account(input.rent,RENT,SYSVAR,'rent').data;
  requireFact(rent.length===17,'invalid_rent','rent');
  const threshold=rent.readDoubleLE(8),rentBase=u64(rent,0)*328n;
  const rentAmount=Math.trunc(Number(rentBase)*threshold);
  requireFact(rentBase<=U64&&Number.isFinite(threshold)&&threshold>=0&&Number.isFinite(rentAmount)&&
    rentAmount>=0&&rent[16]<=100&&BigInt(rentAmount)<U64,
    'invalid_rent','rent');
  const minimum=decimal(input.minimumDelegationLamports,'minimumDelegationLamports');
  requireFact(minimum>0n&&decimal(input.minimumDelegationEpoch,'minimumDelegationEpoch')===epoch&&epoch<U64,
    'minimum_delegation_epoch_mismatch','minimumDelegationEpoch');
  requireFact(c.stake_pool_program===IDS.poolProgram&&c.stake_pool===IDS.pool&&c.jitosol_mint===IDS.mint,
    'config_pool_binding','state.config');
  const resolved=r.recovery_flags===0||(r.recovery&&r.recovery.generation!=='0'&&
    r.recovery.acknowledged_generation===r.recovery.generation&&
    r.recovery.acknowledged_cumulative_loss===r.cumulative_cooldown_losses_lamports);
  requireFact(!c.paused&&r.lifecycle==='WithdrawalActive'&&!r.settlement_recorded&&resolved&&
    BigInt(r.prepared_slot)<=slot&&BigInt(r.prepared_epoch)<=epoch&&BigInt(r.prepared_at)<=now,
    'round_not_active','state.distribution');
  requireFact(c.configured_slippage_bps<=1&&r.stored_slippage_bps===c.configured_slippage_bps,
    'round_slippage_mismatch','state.distribution');
  const sequence=r.active_sequence,nextLegIndex=BigInt(r.next_leg_index),assigned=BigInt(r.cumulative_jitosol_assigned_units);
  const target=BigInt(r.fixed_jitosol_withdrawal_target_units),remaining=target-assigned;
  requireFact(nextLegIndex===BigInt(r.successful_leg_count)&&remaining>=0n&&
    BigInt(c.next_distribution_sequence)===BigInt(sequence)+1n,'round_counters','state.distribution');
  if(input.after!==null&&input.after!==undefined) {
    const after=input.after;
    requireFact(after.programId===snapshot.programId&&after.roundSequence===sequence&&after.fixedTargetUnits===String(target)&&
      slot>=decimal(after.slot,'after.slot')&&nextLegIndex>decimal(after.nextLegIndex,'after.nextLegIndex')&&
      assigned>decimal(after.assignedUnits,'after.assignedUnits'),'refresh_required','after');
  }
  const report={schema:SCHEMA,programId:snapshot.programId,slot:String(slot),epoch:String(epoch),roundSequence:sequence,
    nextLegIndex:String(nextLegIndex),remainingUnits:String(remaining),chainSnapshotVerified:false,
    candidate:null,projectedNextLeg:null,refreshAfter:null,rejections:[]};
  if(remaining===0n)return {...report,status:'TARGET_ASSIGNED'};
  const pool=decodePool(account(input.pool,IDS.pool,IDS.poolProgram,'pool').data,c,epoch);
  const mintData=account(input.mint,IDS.mint,IDS.token,'mint').data;
  requireFact(mintData.length===82&&mintData.readUInt32LE()===1&&mintData.subarray(4,36).equals(withdrawKey.toBuffer())&&
    mintData[44]===9&&mintData[45]===1&&mintData.readUInt32LE(46)===0,'invalid_mint','mint');
  const mint=u64(mintData,36);
  const held=BigInt(c.accounted_historical_jitosol_units)-assigned+BigInt(c.accounted_pending_jitosol_units);
  requireFact(mint<=pool.supply&&held>=0n&&held<=mint,'mint_supply','mint');
  const library=decodeList(account(input.validatorList,c.validator_list,IDS.poolProgram,'validatorList').data);
  const preferred=library.find(row=>row.vote===pool.preferred);
  requireFact(pool.preferred===null||preferred!==undefined,'preferred_missing_from_list','validatorList');
  const snapshotMinimum=BigInt(r.snapshot_leg_input_floor_units),maxLegs=BigInt(r.maximum_useful_legs);
  const priorYield=BigInt(r.prior_next_cycle_yield_lamports),pendingUsed=BigInt(r.pending_sol_used_lamports);
  const unpaid=BigInt(r.outgoing_gross_obligation_lamports)-pendingUsed;
  const residualUnits=BigInt(c.accounted_historical_jitosol_units)-target;
  requireFact(snapshotMinimum>0n&&maxLegs>nextLegIndex&&unpaid>=0n&&residualUnits>=0n,
    'round_continuation','state.distribution');
  const facts={runtimeMinimum:minimum,snapshotMinimum,maxLegs,residualUnits,preferredIndex:preferred?.index,
    slippage:BigInt(r.stored_slippage_bps),roundMinimum:BigInt(r.stored_round_minimum_native_lamports),
    residualNative:BigInt(c.accounted_historical_sol_lamports)+priorYield-min(priorYield,unpaid),
    residualFloor:max(BigInt(r.stored_residual_hwm_floor_lamports),
      BigInt(r.proposed_hwm_after_settlement_lamports)+(r.recovery?pendingUsed:0n))};
  const state={pool,total:pool.total,supply:pool.supply,mint,preferredBalance:preferred?.balance??null,remaining,
    legCount:nextLegIndex,cumulative:BigInt(r.cumulative_delegated_native_lamports)};
  requireFact(Array.isArray(input.sourceAccounts)&&input.sourceAccounts.length<=library.length,'source_accounts','sourceAccounts');
  const seen=new Set(),sources=[];
  for(const entry of input.sourceAccounts) {
    const index=decimal(entry?.validatorIndex,'sourceAccounts.validatorIndex');
    requireFact(index<BigInt(library.length)&&!seen.has(index),'validator_index','sourceAccounts.validatorIndex');
    seen.add(index);
    const source=sourceFor(library[Number(index)],entry.account,epoch,BigInt(rentAmount),minimum);
    if(source)sources.push(source);
    else report.rejections.push({validatorIndex:String(index),reason:'ROW_NOT_CURRENT_ACTIVE'});
  }
  sources.sort((a,b)=>a.index-b.index);
  const available=[];
  for(const source of sources) {
    const quote=assess(source,state,facts);
    if(!quote.reason)available.push(quote);
    else report.rejections.push({validatorIndex:String(source.index),reason:quote.reason,
      ...(quote.capacity===undefined?{}:{capacityUnits:String(quote.capacity)}),
      ...(quote.remainder===undefined?{}:{remainingAfterUnits:String(quote.remainder)})});
  }
  available.sort((a,b)=>a.input===b.input?a.source.index-b.source.index:a.input>b.input?-1:1);
  for(const quote of available) {
    if(quote.remainder===0n)return {...report,status:'SINGLE_LEG_CANDIDATE',candidate:describe(quote)};
    let continuation;
    for(const source of sources) {
      const updated=source.index===quote.source.index
        ?{...source,balance:source.balance-quote.output,delegated:source.delegated-quote.output}:source;
      const option=assess(updated,quote.after,facts);
      if(option.reason)continue;
      if(!continuation||option.input>continuation.input)continuation=option;
      if(option.remainder===0n)break;
    }
    if(continuation)return {...report,status:'MULTI_LEG_CANDIDATE',candidate:describe(quote),projectedNextLeg:describe(continuation),
      refreshAfter:{programId:snapshot.programId,slot:String(slot),roundSequence:sequence,fixedTargetUnits:String(target),
        nextLegIndex:String(nextLegIndex),assignedUnits:String(assigned)}};
  }
  return {...report,status:available.length?'NO_OBSERVED_CONTINUATION':'NO_ADMISSIBLE_SOURCE'};
}

module.exports={SCHEMA,selectSource};
