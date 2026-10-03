//! Host System/CPI models and independent full-account state oracles; no Bank rollback claim.
mod support;
#[path = "support/pending_custody.rs"] pub mod pending_custody;
#[allow(dead_code)]
#[path = "support/economic_custody.rs"] mod custody;
#[path = "support/jito_identity_oracle.rs"] mod oracle;
#[allow(dead_code)]
#[path = "support/bootstrap_custody.rs"] mod bootstrap_custody;
#[path = "support/distribution_preparation_custody.rs"] mod preparation_custody;
#[path = "support/withdrawal_preparation_custody.rs"] mod withdrawal_custody;
use support::kif_claim_custody;
use anchor_lang::{prelude::{AccountInfo, Pubkey}, solana_program::{entrypoint::ProgramResult,
    instruction::{AccountMeta, Instruction}, program_error::ProgramError, system_program}};
use piv1::{distribution_preparation_execution::process_withdrawal_instruction_with_host_callbacks as execute,
    instruction_boundary, instruction_errors::{piv1_error_code, DISTRIBUTION_WITHDRAWAL_UNAVAILABLE_CODE, HOST_RUNTIME_UNAVAILABLE_CODE},
    instructions::prepare_distribution::*, events::DistributionPrepared, errors::Piv1Error,
    integrations, state::{ActiveDistribution, DistributionLifecycle}};
use custody::*;
use withdrawal_custody::Fixture;
use piv1::accounts::STAKE_PROGRAM_ID;
use piv1::distribution_preparation_execution::PreparationEvent;
use solana_stake_interface::state::StakeStateV2;
use support::kif_claim_custody::{key, PROGRAM};
fn err(e: Piv1Error) -> ProgramError { ProgramError::Custom(piv1_error_code(e)) }
fn economics(f: &Fixture) -> (u64, [u64; 5], u64, u64, u64) {
    let c = f.inner.base.custody.config();
    let value = c.accounted_historical_sol_lamports + if f.inner.base.pool.pool_token_supply == 0 { 0 } else {
        (u128::from(c.accounted_historical_jitosol_units) * u128::from(f.inner.base.pool.total_lamports)
            / u128::from(f.inner.base.pool.pool_token_supply)) as u64 };
    let gross = (value + c.next_cycle_yield_lamports).saturating_sub(c.protected_principal_hwm_lamports);
    let htfp = gross * 5900 / 10000; let compound = gross * 1950 / 10000;
    let team = gross * 1950 / 10000; let kif = gross * 200 / 10000;
    let split = [htfp, compound, team, kif, gross - htfp - compound - team - kif];
    let out = htfp + team + kif; let pending = c.accounted_pending_sol_lamports.min(out);
    (value, split, gross, pending, c.next_cycle_yield_lamports.min(out-pending))
}
fn plans(f: &Fixture) -> Vec<(usize,u64)> {
    let (_, _, _, pending, carry) = economics(f);
    [(PENDING_SOL,pending),(PRINCIPAL_SOL,carry)].into_iter().filter(|(_,v)| *v != 0).collect()
}
fn transfer(source: usize, amount: u64, ix: &Instruction, a: &[AccountInfo<'_>], seeds: &[&[&[u8]]]) -> ProgramResult {
    let seed: &[u8] = if source == PENDING_SOL { b"pending-sol" } else { b"principal-sol" };
    let (signer,bump) = Pubkey::find_program_address(&[seed], &PROGRAM);
    assert_eq!(seeds, &[&[seed,&[bump]][..]]); assert_eq!(a.len(),3);
    assert_eq!(*a[0].key,signer); assert_eq!(*a[2].key,system_program::ID);
    assert_eq!(Pubkey::create_program_address(seeds[0], &PROGRAM).unwrap(),signer);
    let mut data = vec![2,0,0,0]; data.extend(amount.to_le_bytes());
    assert_eq!(ix.data,data); assert_eq!(ix.program_id,system_program::ID);
    assert_eq!(ix.accounts,vec![AccountMeta::new(signer,true),AccountMeta::new(*a[1].key,false)]);
    **a[0].try_borrow_mut_lamports()? -= amount; **a[1].try_borrow_mut_lamports()? += amount; Ok(())
}
fn raw_prefix(f: &Fixture, count: usize) -> Fixture {
    let mut out=f.clone(); for (source,amount) in plans(f).into_iter().take(count) {
        out.inner.base.custody.accounts[source].lamports -= amount;
        out.inner.base.custody.accounts[ESCROW_SOL].lamports += amount;
    } out
}
fn expected(f: &Fixture,known_minimum:Option<u64>) -> (Fixture,DistributionPrepared) {
    let c=f.inner.base.custody.config(); let (value,s,gross,pending,carry)=economics(f);
    let outgoing=s[0]+s[2]+s[3];let budget=outgoing-pending-carry;
    let pool=&f.inner.base.pool;let t=u128::from(pool.total_lamports);let supply=u128::from(pool.pool_token_supply);
    let target=(((u128::from(budget)+1)*supply-1)/t).min(u128::from(c.accounted_historical_jitosol_units)) as u64;
    let fee=|q:u64|->u64{if pool.stake_withdrawal_fee.numerator==0{0}else{
        ((u128::from(q)*u128::from(pool.stake_withdrawal_fee.numerator)+u128::from(pool.stake_withdrawal_fee.denominator)-1)
            /u128::from(pool.stake_withdrawal_fee.denominator)) as u64}};
    let redeem=|q:u64|->u64{(u128::from(q-fee(q))*t/supply)as u64};
    let minimum=known_minimum.unwrap_or_else(||(1..=pool.pool_token_supply).find(|q|redeem(*q)>=f.minimum).unwrap());
    let legs=target/minimum;let reserve=if pool.stake_withdrawal_fee.numerator==0{0}else{legs-1};
    let lower=(u128::from(target-fee(target)-reserve)*t/supply) as u64-(legs-1);
    let round_floor=(u128::from(lower)*u128::from(10000-c.configured_slippage_bps)/10000) as u64;
    let dust=budget-(u128::from(target)*t/supply)as u64;let delta=s[1]+s[4]+dust;
    let mut next=raw_prefix(f,plans(f).len());
    next.inner.base.custody.edit_config(|n| {n.next_distribution_sequence+=1;
        n.last_successful_preparation_at=Some(f.inner.base.clock.unix_timestamp);n.next_cycle_yield_lamports=0;});
    // Independent complete round oracle, without calling the production transition.
    let mut r=ActiveDistribution::new_idle(f.inner.round().bump);r.last_completed=f.inner.round().last_completed;
    r.lifecycle=DistributionLifecycle::WithdrawalActive;r.active_sequence=c.next_distribution_sequence;
    r.prepared_at=f.inner.base.clock.unix_timestamp;r.prepared_slot=f.inner.base.clock.slot;r.prepared_epoch=f.inner.base.clock.epoch;
    r.old_protected_principal_lamports=c.protected_principal_hwm_lamports;
    r.historical_jitosol_units=c.accounted_historical_jitosol_units;r.historical_sol_lamports=c.accounted_historical_sol_lamports;
    r.historical_value_lamports=value;r.snapshot_pool_total_lamports=f.inner.base.pool.total_lamports;
    r.snapshot_pool_token_supply=f.inner.base.pool.pool_token_supply;
    r.snapshot_withdrawal_fee_numerator=f.inner.base.pool.stake_withdrawal_fee.numerator;
    r.snapshot_withdrawal_fee_denominator=f.inner.base.pool.stake_withdrawal_fee.denominator;
    r.gross_yield_lamports=gross;r.prior_next_cycle_yield_lamports=c.next_cycle_yield_lamports;
    r.htfp_gross_obligation_lamports=s[0];r.permanent_compound_lamports=s[1];
    r.team_owner_gross_obligation_lamports=s[2];r.kif_gross_obligation_lamports=s[3];r.split_dust_lamports=s[4];
    r.outgoing_gross_obligation_lamports=outgoing;r.pending_sol_snapshot_lamports=c.accounted_pending_sol_lamports;
    r.pending_sol_used_lamports=pending;r.stored_residual_hwm_floor_lamports=c.protected_principal_hwm_lamports+delta;
    r.stored_slippage_bps=c.configured_slippage_bps;r.recorded_escrow_available_lamports=pending+carry;
    r.fixed_jitosol_withdrawal_target_units=target;r.snapshot_leg_input_floor_units=minimum;
    r.maximum_useful_legs=legs;r.stored_round_minimum_native_lamports=round_floor;r.snapshot_conversion_dust_lamports=dust;
    r.outstanding_active_round_liability_lamports=outgoing;r.htfp_recipient=c.htfp_recipient;r.team_owner_recipient=c.team_owner_recipient;
    r.guardian_registry=c.guardian_registry;r.guardian_registry_revision=f.inner.registry.revision;r.guardian_keys=f.inner.registry.guardian_keys;
    r.kif_period_id=((f.inner.base.clock.unix_timestamp-c.kif_anchor_timestamp)/2_592_000) as u64;
    r.kif_eligibility_bitmap=f.inner.rewards.iter().enumerate().fold(0,|mask,(i,v)| mask | if v.last_active_period==Some(r.kif_period_id) {1<<i}else{0});
    r.kif_active_guardian_count=r.kif_eligibility_bitmap.count_ones() as u8;
    r.kif_carry_input_lamports=c.collective_kif_carry_lamports;r.proposed_hwm_delta_lamports=delta;
    r.proposed_hwm_after_settlement_lamports=c.protected_principal_hwm_lamports+delta;next.inner.set_round(r);
    let event=DistributionPrepared {config:f.inner.base.custody.accounts[CONFIG].key,sequence:c.next_distribution_sequence,
        gross_yield_lamports:gross,outgoing_lamports:outgoing,pending_sol_used_lamports:pending,
        prior_yield_used_lamports:carry,kif_eligibility_bitmap:r.kif_eligibility_bitmap};(next,event)
}
fn query(ix:&Instruction,a:&[AccountInfo<'_>],seeds:&[&[&[u8]]]) {
    assert_eq!(ix.program_id,STAKE_PROGRAM_ID);assert_eq!(ix.data,vec![13,0,0,0]);
    assert!(ix.accounts.is_empty());assert!(seeds.is_empty());assert_eq!(a.len(),1);assert_eq!(*a[0].key,STAKE_PROGRAM_ID);
}
fn success(f: &mut Fixture) {success_min(f,None)}
fn success_min(f: &mut Fixture,known_minimum:Option<u64>) {
    let (expected,event)=expected(f,known_minimum);let moves=plans(f);let clock=f.inner.base.clock.clone();let rent=f.inner.base.custody.rent.clone();
    let data=f.data();let minimum=f.minimum;let mut count=0;let mut queries=0;let mut events=vec![];
    f.inner.base.custody.with_infos(|a|execute(&PROGRAM,a,&data,||Ok(clock),||Ok(rent),|ix,a,seeds|{
        if ix.program_id==STAKE_PROGRAM_ID{query(ix,a,seeds);queries+=1;return Ok(());}
        let(source,amount)=moves[count];count+=1;transfer(source,amount,ix,a,seeds)
    },||Some((STAKE_PROGRAM_ID,minimum.to_le_bytes().to_vec())),|e|events.push(e))).unwrap();
    assert_eq!(queries,1);assert_eq!(count,moves.len());assert_eq!(events,vec![PreparationEvent::Prepared(event)]);assert_eq!(*f,expected);
}
fn reject(mut f: Fixture, expected: Option<ProgramError>) {
    let before=f.clone();let clock=f.inner.base.clock.clone();let rent=f.inner.base.custody.rent.clone();let data=f.data();let minimum=f.minimum;
    let result=f.inner.base.custody.with_infos(|a|execute(&PROGRAM,a,&data,||Ok(clock),||Ok(rent),
        |ix,a,seeds|{query(ix,a,seeds);Ok(())},||Some((STAKE_PROGRAM_ID,minimum.to_le_bytes().to_vec())),|_|panic!("no failure event")));
    if let Some(expected)=expected {assert_eq!(result,Err(expected));}else{assert!(result.is_err());}assert_eq!(f,before);
}
fn insufficient(f:&mut Fixture){
    let mut expected=f.clone();expected.inner.base.custody.edit_config(|c|c.last_valid_insufficient_attempt_at=Some(f.inner.base.clock.unix_timestamp));
    let clock=f.inner.base.clock.clone();let rent=f.inner.base.custody.rent.clone();let data=f.data();let minimum=f.minimum;let mut queries=0;let mut events=vec![];
    f.inner.base.custody.with_infos(|a|execute(&PROGRAM,a,&data,||Ok(clock),||Ok(rent),|ix,a,seeds|{
        query(ix,a,seeds);queries+=1;Ok(())},||Some((STAKE_PROGRAM_ID,minimum.to_le_bytes().to_vec())),|e|events.push(e))).unwrap();
    assert_eq!(queries,1);assert_eq!(*f,expected);
    assert_eq!(events.len(),1);let PreparationEvent::Insufficient(event)=events[0]else{panic!("insufficient event")};
    assert_eq!(event.config,f.inner.base.custody.accounts[CONFIG].key);assert_eq!(event.attempted_at,f.inner.base.clock.unix_timestamp);
    let (_,split,_,pending,carry)=economics(f);let budget=split[0]+split[2]+split[3]-pending-carry;
    let pool=&f.inner.base.pool;
    let target=(((u128::from(budget)+1)*u128::from(pool.pool_token_supply)-1)/u128::from(pool.total_lamports)) as u64;
    assert_eq!(event.target_jitosol_units,target);assert!(event.technical_minimum_jitosol_units>target);
    let redemption=|q:u64|{let fee=(u128::from(q)*u128::from(pool.stake_withdrawal_fee.numerator)+u128::from(pool.stake_withdrawal_fee.denominator)-1)
        /u128::from(pool.stake_withdrawal_fee.denominator);(u128::from(q)-fee)*u128::from(pool.total_lamports)/u128::from(pool.pool_token_supply)};
    assert!(redemption(event.technical_minimum_jitosol_units)>=u128::from(minimum));
    assert!(redemption(event.technical_minimum_jitosol_units-1)<u128::from(minimum));
}

#[test]
fn both_profiles_funding_mixes_snapshot_complete_state_and_preserve_old_liquid_boundary() {
    for shared in [false,true] {for (pending,carry) in [(0,0),(300,0),(0,1000),(300,1000)] {
        let mut f=Fixture::new(pending,carry,shared);assert_eq!(f.inner.base.custody.accounts.len(),if shared{28}else{29});
        f.inner.base.custody.edit_config(|c|c.last_valid_insufficient_attempt_at=Some(999_999));
        success(&mut f);reject(f,Some(err(Piv1Error::InvalidLifecycle)));
    }}
    let mut large=Fixture::new(0,0,false);large.minimum=1_000_000_000;
    large.inner.base.pool.total_lamports=1_000_000_000_000;large.inner.base.pool.pool_token_supply=1_000_000_000_000;
    large.inner.base.mint.supply=1_000_000_000_000;
    large.inner.base.custody.set_token_units(PRINCIPAL_JITO,100_000_000_000);
    large.inner.base.custody.edit_config(|c|{c.accounted_historical_jitosol_units=100_000_000_000;c.protected_principal_hwm_lamports=90_000_000_000;});
    if let StakeStateV2::Stake(_,ref mut stake,_)=large.stake{stake.delegation.stake=20_000_000_000;}
    let source=large.source();large.inner.base.custody.accounts[source].lamports=large.inner.base.custody.rent.minimum_balance(200)+20_000_000_000;
    large.inner.base.sync();large.sync_source();
    // Literal exact1:1 boundary: q−ceil(q/1000) equalsD at1_001_001_002,
    // but equalsD−1 at the immediately preceding input. No billion-step scan.
    assert_eq!(1_001_001_002_u64-1_001_002,large.minimum);
    assert_eq!(1_001_001_001_u64-1_001_002,large.minimum-1);
    success_min(&mut large,Some(1_001_001_002));
    let mut f=Fixture::new(0,0,false);let c=f.inner.base.clock.clone();let r=f.inner.base.custody.rent.clone();
    f.inner.base.custody.with_infos(|all|{
        assert_eq!(piv1::distribution_preparation_execution::process_instruction_with_host_callbacks(&PROGRAM,&all[..27],
            &PREPARE_DISTRIBUTION_DATA,||Ok(c),||Ok(r),|_,_,_|panic!("legacy query"),|_|panic!("event")),
            Err(ProgramError::Custom(DISTRIBUTION_WITHDRAWAL_UNAVAILABLE_CODE)));
    });
}
#[test]
fn raw_fee_burn_lag_preferred_active_and_seed_suffix_supported() {
    for (n,d) in [(0,0),(0,19),(1,1000)] {let mut f=Fixture::new(300,1000,true);
        f.inner.base.pool.stake_withdrawal_fee=oracle::Fee{numerator:n,denominator:d};
        f.inner.base.pool.preferred_withdraw_validator_vote_address=Some(key(202));
        f.inner.base.mint.supply=2_000_000;f.inner.base.sync();success(&mut f);
    }
    for activation in [39,40,u64::MAX] {let mut f=Fixture::new(0,0,false);
        if let StakeStateV2::Stake(_,ref mut stake,_)=f.stake{stake.delegation.activation_epoch=activation;}
        f.sync_source();let source=f.source();let suffix=71_u32;
        f.inner.base.custody.accounts[bootstrap_custody::LIST].data[9+36..9+40].copy_from_slice(&suffix.to_le_bytes());
        f.inner.base.custody.accounts[source].key=Pubkey::find_program_address(&[key(202).as_ref(),
            integrations::jito_identity::JITO_STAKE_POOL.as_ref(),&suffix.to_le_bytes()],&integrations::jito_identity::JITO_STAKE_POOL_PROGRAM).0;
        success(&mut f);
    }
}
#[test]
fn dynamic_minimum_valid_insufficiency_zero_inverse_and_retry_clock_only() {
    let mut f=Fixture::new(300,0,false);f.minimum=15_000;insufficient(&mut f);
    reject(f.clone(),Some(err(Piv1Error::InsufficientAttemptCooldownActive)));
    let mut ready=f.clone();ready.minimum=100;success(&mut ready);
    f.inner.base.clock.unix_timestamp+=86_400;f.inner.sync_clock();insufficient(&mut f);
    let mut zero=Fixture::new(0,0,true);zero.inner.base.pool.total_lamports=20_000_000;
    zero.inner.base.custody.edit_config(|c|c.protected_principal_hwm_lamports=1_999_998);zero.inner.base.sync();
    let mut legacy=zero.inner.base.custody.config();let before=legacy.clone();
    assert_eq!(piv1::state::record_valid_insufficient_attempt(&mut legacy,&zero.inner.round(),piv1::state::ValidInsufficientAttemptInput{
        attempted_at:1_000_000,historical_value_lamports:2_000_000,pending_sol_snapshot_lamports:0,
        computed_jitosol_target_units:0,validated_technical_minimum_units:51}),Err(Piv1Error::ZeroTarget));assert_eq!(legacy,before);
    insufficient(&mut zero);
}
#[test]
fn real_source_state_record_capacity_and_operational_rent_are_required_before_cooldown() {
    // Default lockup timestamp0 is still in force at an otherwise-valid negative
    // Clock timestamp. SPL cannot authorize the future withdrawer without a
    // custodian; a negative KIF anchor must not turn this into a feasible source.
    let mut locked=Fixture::new(0,0,false);locked.inner.base.clock.unix_timestamp=-1;
    locked.inner.base.custody.edit_config(|c|c.kif_anchor_timestamp=-1000);locked.inner.sync_clock();
    reject(locked,Some(err(Piv1Error::InvalidCustodyObservation)));
    for case in 0..15 {let mut f=Fixture::new(0,0,false);f.minimum=15_000;let source=f.source();
        match case {
            0=>f.index=1,
            1=>f.inner.base.custody.accounts[bootstrap_custody::LIST].data[9+40]=1,
            2=>f.inner.base.custody.accounts[bootstrap_custody::LIST].data[9+16..9+24].copy_from_slice(&39_u64.to_le_bytes()),
            3=>f.inner.base.custody.accounts[source].owner=system_program::ID,
            4=>f.inner.base.custody.accounts[source].key=key(201),
            5=>f.inner.base.custody.accounts[source].lamports+=1,
            6=>{if let StakeStateV2::Stake(ref mut meta,_,_)=f.stake{meta.authorized.withdrawer=key(201);}f.sync_source();},
            7=>{if let StakeStateV2::Stake(_,ref mut stake,_)=f.stake{stake.delegation.deactivation_epoch=40;}f.sync_source();},
            8=>{if let StakeStateV2::Stake(_,ref mut stake,_)=f.stake{stake.delegation.activation_epoch=41;}f.sync_source();},
            9=>f.inner.base.custody.accounts[source].data[196]=1,
            10=>{if let StakeStateV2::Stake(_,ref mut stake,_)=f.stake{stake.delegation.stake=0;}f.sync_source();},
            11=>f.inner.base.custody.accounts[OPERATIONAL_SOL].lamports=f.inner.base.custody.rent.minimum_balance(0),
            12=>{f.inner.base.pool.preferred_withdraw_validator_vote_address=Some(key(201));f.inner.base.sync();},
            13=>{f.inner.base.pool.stake_withdrawal_fee=oracle::Fee{numerator:1,denominator:1};f.inner.base.sync();},
            _=>{f.minimum=6_000_000;},
        } reject(f,None);
    }
}
#[test]
fn first_maximum_fill_stranding_and_exact_one_lamport_hwm_partition_fail_closed() {
    for (available,valid) in [(7992,false),(7941,true)] {let mut f=Fixture::new(0,0,false);let source=f.source();
        if let StakeStateV2::Stake(_,ref mut stake,_)=f.stake{stake.delegation.stake=1_000_000+available;}
        f.inner.base.custody.accounts[source].lamports=f.inner.base.custody.rent.minimum_balance(200)+1_000_000+available;f.sync_source();
        if valid {success(&mut f);}else{reject(f,Some(err(Piv1Error::TechnicalFloorNotMet)));}
    }
    let mut partition=Fixture::new(0,0,false);partition.inner.base.pool.total_lamports=10_100_000;
    partition.inner.base.custody.edit_config(|c|c.protected_principal_hwm_lamports=1_000_000);partition.inner.base.sync();
    // q=7971 has book8050, but floor((1m−7971)*1.01)=1,001,949 < H+1950.
    reject(partition,Some(err(Piv1Error::HighWaterMarkDecrease)));
}
#[test]
fn query_return_authentication_failure_and_every_account_corruption_reject_without_state() {
    let base=Fixture::new(300,1000,false);
    for result in [None,Some((key(201),100_u64.to_le_bytes().to_vec())),Some((STAKE_PROGRAM_ID,vec![0;7])),
        Some((STAKE_PROGRAM_ID,vec![0;9])),Some((STAKE_PROGRAM_ID,0_u64.to_le_bytes().to_vec()))] {
        let mut f=base.clone();let data=f.data();let clock=f.inner.base.clock.clone();let rent=f.inner.base.custody.rent.clone();
        let status=f.inner.base.custody.with_infos(|a|execute(&PROGRAM,a,&data,||Ok(clock),||Ok(rent),|ix,a,seeds|{query(ix,a,seeds);Ok(())},||result,|_|panic!("event")));
        assert!(status.is_err());assert_eq!(f,base);
    }
    for target in 0..base.inner.base.custody.accounts.len(){let mut f=base.clone();let data=f.data();let clock=f.inner.base.clock.clone();let rent=f.inner.base.custody.rent.clone();
        let status=f.inner.base.custody.with_infos(|all|execute(&PROGRAM,all,&data,||Ok(clock),||Ok(rent),|ix,a,seeds|{
            query(ix,a,seeds);**all[target].try_borrow_mut_lamports()?+=1;Ok(())},||Some((STAKE_PROGRAM_ID,100_u64.to_le_bytes().to_vec())),|_|panic!("event")));
        assert_eq!(status,Err(err(Piv1Error::ContributionObservationMismatch)));
        let mut expected=base.clone();expected.inner.base.custody.accounts[target].lamports+=1;assert_eq!(f,expected);
    }
    let mut f=base.clone();let data=f.data();let clock=f.inner.base.clock.clone();let rent=f.inner.base.custody.rent.clone();
    assert_eq!(f.inner.base.custody.with_infos(|a|execute(&PROGRAM,a,&data,||Ok(clock),||Ok(rent),|_,_,_|Err(ProgramError::Custom(988)),||None,|_|panic!("event"))),Err(ProgramError::Custom(988)));assert_eq!(f,base);
}
#[test]
fn both_funding_failures_preserve_old_snapshot_and_model_discard_retries() {
    let base=Fixture::new(300,1000,false);assert_eq!(plans(&base).len(),2);
    for failed in 0..2 {for partial in [false,true] {let mut f=base.clone();let moves=plans(&f);let data=f.data();
        let clock=f.inner.base.clock.clone();let rent=f.inner.base.custody.rent.clone();let mut count=0;
        let result=f.inner.base.custody.with_infos(|a|execute(&PROGRAM,a,&data,||Ok(clock),||Ok(rent),|ix,a,seeds|{
            if ix.program_id==STAKE_PROGRAM_ID{query(ix,a,seeds);return Ok(());}
            let n=count;count+=1;let(source,amount)=moves[n];if n==failed&&!partial{return Err(ProgramError::Custom(987));}
            transfer(source,amount,ix,a,seeds)?;if n==failed{Err(ProgramError::Custom(987))}else{Ok(())}
        },||Some((STAKE_PROGRAM_ID,100_u64.to_le_bytes().to_vec())),|_|panic!("failed event")));
        assert_eq!(result,Err(ProgramError::Custom(987)));assert_eq!(f,raw_prefix(&base,failed+usize::from(partial)));
        f=base.clone();success(&mut f);
    }}
}
#[test]
fn every_post_transfer_account_and_protocol_data_tamper_blocks_both_state_writes() {
    let base=Fixture::new(300,1000,false);
    for leg in 0..2 {for target in 0..base.inner.base.custody.accounts.len(){let mut f=base.clone();let moves=plans(&f);let data=f.data();
        let clock=f.inner.base.clock.clone();let rent=f.inner.base.custody.rent.clone();let mut count=0;
        let status=f.inner.base.custody.with_infos(|all|execute(&PROGRAM,all,&data,||Ok(clock),||Ok(rent),|ix,a,seeds|{
            if ix.program_id==STAKE_PROGRAM_ID{query(ix,a,seeds);return Ok(());}
            let n=count;count+=1;let(source,amount)=moves[n];transfer(source,amount,ix,a,seeds)?;
            if n==leg{**all[target].try_borrow_mut_lamports()?+=1;}Ok(())
        },||Some((STAKE_PROGRAM_ID,100_u64.to_le_bytes().to_vec())),|_|panic!("event")));
        assert_eq!(status,Err(err(Piv1Error::ContributionObservationMismatch)));
        let mut expected=raw_prefix(&base,leg+1);expected.inner.base.custody.accounts[target].lamports+=1;assert_eq!(f,expected);
    }}
    for target in [CONFIG,ROUND,bootstrap_custody::POOL,bootstrap_custody::LIST,MINT,base.source()] {
        let mut f=base.clone();let moves=plans(&f);let data=f.data();let clock=f.inner.base.clock.clone();let rent=f.inner.base.custody.rent.clone();
        let status=f.inner.base.custody.with_infos(|all|execute(&PROGRAM,all,&data,||Ok(clock),||Ok(rent),|ix,a,seeds|{
            if ix.program_id==STAKE_PROGRAM_ID{query(ix,a,seeds);return Ok(());}
            let(source,amount)=moves[0];transfer(source,amount,ix,a,seeds)?;all[target].try_borrow_mut_data()?[8]^=1;Ok(())
        },||Some((STAKE_PROGRAM_ID,100_u64.to_le_bytes().to_vec())),|_|panic!("event")));
        assert_eq!(status,Err(err(Piv1Error::ContributionObservationMismatch)));
        let mut expected=raw_prefix(&base,1);expected.inner.base.custody.accounts[target].data[8]^=1;assert_eq!(f,expected);
    }
}
#[test]
fn abi_topology_runtime_guard_writable_alias_and_hostile_borrows() {
    let base=Fixture::new(300,1000,false);
    let mut wrong=base.data();wrong[8]=2;
    for (size,data,want) in [(29,wrong,ProgramError::InvalidInstructionData),(27,base.data(),ProgramError::NotEnoughAccountKeys),
        (30,base.data(),ProgramError::InvalidArgument)] {let mut f=base.clone();
        f.inner.base.custody.with_infos(|all|{let mut a=all.to_vec();a.resize(size,all[0].clone());
            assert_eq!(instruction_boundary::process_instruction(&PROGRAM,&a,&data),Err(want));});assert_eq!(f,base);
    }
    let mut f=base.clone();let data=f.data();f.inner.base.custody.with_infos(|a|assert_eq!(instruction_boundary::process_instruction(&PROGRAM,a,&data),Err(ProgramError::Custom(HOST_RUNTIME_UNAVAILABLE_CODE))));
    for index in [CONFIG,ROUND,PENDING_SOL,PRINCIPAL_SOL,ESCROW_SOL]{let mut f=base.clone();f.inner.base.custody.accounts[index].writable=false;reject(f,Some(err(Piv1Error::AccountNotWritable)));}
    for index in [CONFIG,ROUND,base.source(),bootstrap_custody::LIST]{let mut f=base.clone();let data=f.data();let clock=f.inner.base.clock.clone();let rent=f.inner.base.custody.rent.clone();
        f.inner.base.custody.with_infos(|a|{let hold=a[index].try_borrow_mut_data().unwrap();
            assert!(execute(&PROGRAM,a,&data,||Ok(clock),||Ok(rent),|ix,a,seeds|{query(ix,a,seeds);Ok(())},||Some((STAKE_PROGRAM_ID,100_u64.to_le_bytes().to_vec())),|_|panic!("event")).is_err());drop(hold);});assert_eq!(f,base);
    }
    let mut f=base.clone();let source=f.source();f.inner.base.custody.accounts[source].key=f.inner.base.custody.accounts[AUTHORITY].key;reject(f,Some(err(Piv1Error::AccountAlias)));
}
#[test]
fn cadence_pause_current_pool_and_guardian_failures_never_use_economic_custody() {
    for case in 0..7{let mut f=Fixture::new(300,1000,false);match case{
        0=>f.inner.base.custody.edit_config(|c|c.paused=true),
        1=>f.inner.base.custody.edit_config(|c|c.last_successful_preparation_at=Some(999_999)),
        2=>{f.inner.base.pool.last_update_epoch-=1;f.inner.base.sync();},
        3=>f.inner.base.clock.slot+=1,
        4=>f.inner.base.custody.edit_config(|c|c.guardian_registry_revision+=1),
        5=>f.inner.base.custody.accounts[PENDING_SOL].lamports-=1,
        _=>f.inner.base.custody.accounts[PRINCIPAL_SOL].lamports+=1,
    }reject(f,None);}
    let mut f=Fixture::new(300,1000,false);
    for index in [PRINCIPAL_JITO,PENDING_JITO,OPERATIONAL_SOL]{f.inner.base.custody.accounts[index].lamports+=123;}
    success(&mut f);
}
