//! Host System/CPI models and independent full-account state oracles; no Bank rollback claim.
mod support;
#[path = "support/pending_custody.rs"] pub mod pending_custody;
#[allow(dead_code)]
#[path = "support/economic_custody.rs"] mod custody;
#[path = "support/jito_identity_oracle.rs"] mod oracle;
#[allow(dead_code)]
#[path = "support/bootstrap_custody.rs"] mod bootstrap_custody;
#[path = "support/distribution_preparation_custody.rs"] mod preparation_custody;
use support::kif_claim_custody;
use anchor_lang::{prelude::{AccountInfo, Pubkey}, solana_program::{entrypoint::ProgramResult,
    instruction::{AccountMeta, Instruction}, program_error::ProgramError, system_program}};
use piv1::{distribution_preparation_execution::process_instruction_with_host_callbacks as execute,
    instruction_boundary, instruction_errors::{piv1_error_code, DISTRIBUTION_WITHDRAWAL_UNAVAILABLE_CODE, HOST_RUNTIME_UNAVAILABLE_CODE},
    instructions::prepare_distribution::*, events::DistributionPrepared, errors::Piv1Error,
    integrations, state::{ActiveDistribution, DistributionLifecycle}};
use custody::*;
use preparation_custody::Fixture;
use support::kif_claim_custody::{key, PROGRAM};
fn err(e: Piv1Error) -> ProgramError { ProgramError::Custom(piv1_error_code(e)) }
fn economics(f: &Fixture) -> (u64, [u64; 5], u64, u64, u64) {
    let c = f.base.custody.config();
    let value = c.accounted_historical_sol_lamports + if f.base.pool.pool_token_supply == 0 { 0 } else {
        (u128::from(c.accounted_historical_jitosol_units) * u128::from(f.base.pool.total_lamports)
            / u128::from(f.base.pool.pool_token_supply)) as u64 };
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
        out.base.custody.accounts[source].lamports -= amount;
        out.base.custody.accounts[ESCROW_SOL].lamports += amount;
    } out
}
fn expected(f: &Fixture) -> (Fixture,DistributionPrepared) {
    let c=f.base.custody.config(); let (value,s,gross,pending,carry)=economics(f);
    let outgoing=s[0]+s[2]+s[3]; let delta=s[1]+s[4];
    let mut next=raw_prefix(f,plans(f).len());
    next.base.custody.edit_config(|n| {n.next_distribution_sequence+=1;
        n.last_successful_preparation_at=Some(f.base.clock.unix_timestamp);n.next_cycle_yield_lamports=0;});
    // Independent complete round oracle, without calling the production transition.
    let mut r=ActiveDistribution::new_idle(f.round().bump);r.last_completed=f.round().last_completed;
    r.lifecycle=DistributionLifecycle::EscrowFunded;r.active_sequence=c.next_distribution_sequence;
    r.prepared_at=f.base.clock.unix_timestamp;r.prepared_slot=f.base.clock.slot;r.prepared_epoch=f.base.clock.epoch;
    r.old_protected_principal_lamports=c.protected_principal_hwm_lamports;
    r.historical_jitosol_units=c.accounted_historical_jitosol_units;r.historical_sol_lamports=c.accounted_historical_sol_lamports;
    r.historical_value_lamports=value;r.snapshot_pool_total_lamports=f.base.pool.total_lamports;
    r.snapshot_pool_token_supply=f.base.pool.pool_token_supply;
    r.snapshot_withdrawal_fee_numerator=f.base.pool.stake_withdrawal_fee.numerator;
    r.snapshot_withdrawal_fee_denominator=f.base.pool.stake_withdrawal_fee.denominator;
    r.gross_yield_lamports=gross;r.prior_next_cycle_yield_lamports=c.next_cycle_yield_lamports;
    r.htfp_gross_obligation_lamports=s[0];r.permanent_compound_lamports=s[1];
    r.team_owner_gross_obligation_lamports=s[2];r.kif_gross_obligation_lamports=s[3];r.split_dust_lamports=s[4];
    r.outgoing_gross_obligation_lamports=outgoing;r.pending_sol_snapshot_lamports=c.accounted_pending_sol_lamports;
    r.pending_sol_used_lamports=pending;r.stored_residual_hwm_floor_lamports=c.protected_principal_hwm_lamports+delta;
    r.stored_slippage_bps=c.configured_slippage_bps;r.recorded_escrow_available_lamports=outgoing;
    r.outstanding_active_round_liability_lamports=outgoing;r.htfp_recipient=c.htfp_recipient;r.team_owner_recipient=c.team_owner_recipient;
    r.guardian_registry=c.guardian_registry;r.guardian_registry_revision=f.registry.revision;r.guardian_keys=f.registry.guardian_keys;
    r.kif_period_id=((f.base.clock.unix_timestamp-c.kif_anchor_timestamp)/2_592_000) as u64;
    r.kif_eligibility_bitmap=f.rewards.iter().enumerate().fold(0,|mask,(i,v)| mask | if v.last_active_period==Some(r.kif_period_id) {1<<i}else{0});
    r.kif_active_guardian_count=r.kif_eligibility_bitmap.count_ones() as u8;
    r.kif_carry_input_lamports=c.collective_kif_carry_lamports;r.proposed_hwm_delta_lamports=delta;
    r.proposed_hwm_after_settlement_lamports=c.protected_principal_hwm_lamports+delta;next.set_round(r);
    let event=DistributionPrepared {config:f.base.custody.accounts[CONFIG].key,sequence:c.next_distribution_sequence,
        gross_yield_lamports:gross,outgoing_lamports:outgoing,pending_sol_used_lamports:pending,
        prior_yield_used_lamports:carry,kif_eligibility_bitmap:r.kif_eligibility_bitmap};(next,event)
}
fn success(f: &mut Fixture) {
    let (expected,event)=expected(f);let moves=plans(f);let clock=f.base.clock.clone();let rent=f.base.custody.rent.clone();
    let mut count=0;let mut events=vec![];
    f.base.custody.with_infos(|a|execute(&PROGRAM,a,&PREPARE_DISTRIBUTION_DATA,||Ok(clock),||Ok(rent),|ix,a,seeds|{
        let(source,amount)=moves[count];count+=1;transfer(source,amount,ix,a,seeds)
    },|e|events.push(e))).unwrap();
    assert_eq!(count,moves.len());assert_eq!(events,vec![event]);assert_eq!(*f,expected);
}
fn reject(mut f: Fixture, expected: Option<ProgramError>) {
    let before=f.clone();let clock=f.base.clock.clone();let rent=f.base.custody.rent.clone();
    let result=f.base.custody.with_infos(|a|execute(&PROGRAM,a,&PREPARE_DISTRIBUTION_DATA,||Ok(clock),||Ok(rent),
        |_,_,_|panic!("pre-effect rejection"),|_|panic!("no failure event")));
    if let Some(expected)=expected {assert_eq!(result,Err(expected));}else{assert!(result.is_err());} assert_eq!(f,before);
}
fn no_yield(mut f: Fixture) {
    let before=f.clone();let clock=f.base.clock.clone();let rent=f.base.custody.rent.clone();
    f.base.custody.with_infos(|a|execute(&PROGRAM,a,&PREPARE_DISTRIBUTION_DATA,||Ok(clock),||Ok(rent),
        |_,_,_|panic!("no yield effects"),|_|panic!("no yield event"))).unwrap();assert_eq!(f,before);
}

#[test]
fn liquid_pending_carry_mixed_both_topologies_and_full_snapshot_oracle() {
    for shared in [false,true] {for (pending,carry) in [(20_000,0),(0,100_000),(300,100_000),(200_000,100_000)] {
        let mut f=Fixture::new(pending,carry,shared,0b101011);
        assert_eq!(f.base.custody.accounts.len(),if shared{26}else{27});
        f.base.custody.edit_config(|c|c.last_valid_insufficient_attempt_at=Some(999_999));
        success(&mut f);reject(f,Some(err(Piv1Error::InvalidLifecycle)));
    }}
}
#[test]
fn historical_loss_is_offset_by_carry_and_unused_carry_and_kif_remain_separate() {
    for bitmap in [0,63] {let mut f=Fixture::new(300,100_000,false,bitmap);
        f.base.pool.total_lamports=9_900_000;f.base.sync();success(&mut f);
        assert!(f.round().prior_next_cycle_yield_lamports>f.round().prior_next_cycle_yield_used_lamports().unwrap());
        assert_eq!(f.round().kif_carry_input_lamports,19);assert_eq!(f.round().actual_zero_active_kif_compound_lamports,0);
    }
}
#[test]
fn no_yield_is_byte_identical_and_preserves_both_clocks_even_with_empty_pool() {
    let mut f=Fixture::new(100,0,true,0);f.base.pool.total_lamports=9_900_000;f.base.sync();
    f.base.custody.edit_config(|c|c.last_valid_insufficient_attempt_at=Some(1_000_000));
    f.base.custody.accounts[CONFIG].writable=false;f.base.custody.accounts[ROUND].writable=false;no_yield(f.clone());
    let mut too_soon=f.clone();too_soon.base.custody.edit_config(|c|c.last_successful_preparation_at=Some(999_999));
    reject(too_soon,Some(err(Piv1Error::PreparationIntervalNotElapsed)));
    f.base.custody.edit_config(|c|c.paused=true);reject(f,Some(err(Piv1Error::PausedOperation)));
    let mut f=Fixture::new(0,0,false,0);f.base.custody.set_token_units(PRINCIPAL_JITO,0);f.base.custody.set_token_units(PENDING_JITO,0);
    f.base.custody.edit_config(|c|{c.accounted_historical_jitosol_units=0;c.accounted_pending_jitosol_units=0;});
    f.base.pool.total_lamports=0;f.base.pool.pool_token_supply=0;f.base.mint.supply=0;f.base.sync();no_yield(f);
}
#[test]
fn withdrawal_shortfall_never_mutates_cooldown_even_one_lamport_missing() {
    for pending in [0,8049] {let mut f=Fixture::new(pending,0,false,1);
        f.base.custody.edit_config(|c|c.last_valid_insufficient_attempt_at=Some(999_999));
        reject(f,Some(ProgramError::Custom(DISTRIBUTION_WITHDRAWAL_UNAVAILABLE_CODE)));}
    assert_eq!(DISTRIBUTION_WITHDRAWAL_UNAVAILABLE_CODE,6145);
    let mut f=Fixture::new(1,0,false,1);f.base.pool.total_lamports=10_000_010;f.base.sync();
    reject(f,Some(err(Piv1Error::ZeroTarget)));
}
#[test]
fn cadence_pause_sequence_current_clock_registry_and_activity_period_guards() {
    let mut f=Fixture::new(20_000,0,false,63);
    f.base.custody.edit_config(|c|c.last_successful_preparation_at=Some(136_001));
    reject(f.clone(),Some(err(Piv1Error::PreparationIntervalNotElapsed)));
    f.base.custody.edit_config(|c|c.last_successful_preparation_at=Some(136_000));success(&mut f);
    let mut completed=Fixture::new(20_000,0,false,63);let mut r=completed.round();
    r.last_completed=Some(piv1::state::CompletedDistributionSummary { sequence:4,completed_at:100,
        gross_yield_lamports:10,actual_allocated_outgoing_lamports:8,integrated_contribution_value_lamports:17,
        final_protected_hwm_lamports:1_000_000,fixed_jitosol_withdrawal_target_units:0,successful_leg_count:0,
        cumulative_cooldown_rewards_lamports:0,actual_kif_liability_lamports:0,actual_kif_carry_next_lamports:0 });
    completed.set_round(r);completed.base.custody.edit_config(|c|c.next_distribution_sequence=5);
    let mut mismatch=completed.clone();mismatch.base.custody.edit_config(|c|c.next_distribution_sequence=4);
    reject(mismatch,Some(err(Piv1Error::SequenceMismatch)));success(&mut completed);
    for case in 0..5 {let mut f=Fixture::new(20_000,0,false,63);match case {
        0=>f.base.custody.edit_config(|c|c.paused=true),1=>f.base.custody.edit_config(|c|c.next_distribution_sequence=u64::MAX),
        2=>f.base.custody.edit_config(|c|c.guardian_registry_revision+=1),
        3=>f.base.custody.accounts[f.guardian_start+1].key=key(201),
        _=>f.base.clock.slot+=1,}reject(f,None);}
    for now in [2_591_999,2_592_000] {let mut f=Fixture::new(20_000,0,true,63);f.base.clock.unix_timestamp=now;f.sync_clock();
        success(&mut f);assert_eq!(f.round().kif_eligibility_bitmap,if now<2_592_000{63}else{0});}
}
#[test]
fn current_stored_ratio_burn_lag_raw_fees_and_combined_supply_validation() {
    let mut f=Fixture::new(20_000,0,false,1);f.base.mint.supply=2_000_000;f.base.sync();success(&mut f);
    assert_eq!(f.round().historical_value_lamports,1_010_000);
    let mut nonzero=Fixture::new(20_000,0,false,1);
    nonzero.base.pool.stake_withdrawal_fee=oracle::Fee{numerator:1,denominator:1000};nonzero.base.sync();success(&mut nonzero);
    assert_eq!(nonzero.round().snapshot_withdrawal_fee_numerator,1);
    let mut overflow=Fixture::new(20_000,0,false,1);overflow.base.pool.total_lamports=u64::MAX;
    overflow.base.pool.pool_token_supply=1_000_073;overflow.base.mint.supply=1_000_073;
    overflow.base.custody.edit_config(|c|c.next_cycle_yield_lamports=2_000_000_000_000_000);
    overflow.base.custody.accounts[PRINCIPAL_SOL].lamports+=2_000_000_000_000_000;overflow.base.sync();
    reject(overflow,Some(err(Piv1Error::ArithmeticOverflow)));
    for denominator in [0,17] {let mut f=Fixture::new(20_000,0,true,1);
        f.base.pool.stake_withdrawal_fee=oracle::Fee{numerator:0,denominator};f.base.sync();success(&mut f);
        assert_eq!(f.round().snapshot_withdrawal_fee_denominator,denominator);}
    for case in 0..6 {let mut f=Fixture::new(20_000,0,false,1);match case {
        0=>f.base.pool.last_update_epoch-=1,1=>f.base.pool.last_update_epoch+=1,
        2=>f.base.mint.supply=f.base.pool.pool_token_supply+1,
        3=>f.base.mint.supply=1_000_050,4=>f.base.pool.total_lamports=0,
        _=>{f.base.pool.pool_token_supply=1;f.base.mint.supply=1;},}f.base.sync();reject(f,None);}
}
#[test]
fn every_custody_surplus_deficit_and_native_quarantine() {
    for index in [PENDING_SOL,PRINCIPAL_SOL,ESCROW_SOL,KIF_SOL,PRINCIPAL_JITO,PENDING_JITO] {
        for more in [false,true] {let mut f=Fixture::new(20_000,1000,false,1);
            if index==PRINCIPAL_JITO||index==PENDING_JITO {let n=f.base.custody.token_units(index);f.base.custody.set_token_units(index,if more{n+1}else{n-1});}
            else if more {f.base.custody.accounts[index].lamports+=1;}else{f.base.custody.accounts[index].lamports-=1;}
            reject(f,None);
        }
    }
    let mut f=Fixture::new(20_000,0,false,1);for i in [PRINCIPAL_JITO,PENDING_JITO,OPERATIONAL_SOL]{f.base.custody.accounts[i].lamports+=123;}
    success(&mut f);
}
#[test]
fn every_transfer_failure_preserves_old_state_and_discarded_world_retries() {
    let base=Fixture::new(300,100_000,false,1);assert_eq!(plans(&base).len(),2);
    for failed in 0..2 {for partial in [false,true] {let mut f=base.clone();let moves=plans(&f);let clock=f.base.clock.clone();let rent=f.base.custody.rent.clone();let mut count=0;
        let result=f.base.custody.with_infos(|a|execute(&PROGRAM,a,&PREPARE_DISTRIBUTION_DATA,||Ok(clock),||Ok(rent),|ix,a,seeds|{
            let n=count;count+=1;let(source,amount)=moves[n];if n==failed&&!partial{return Err(ProgramError::Custom(987));}
            transfer(source,amount,ix,a,seeds)?;if n==failed {Err(ProgramError::Custom(987))}else{Ok(())}
        },|_|panic!("failed event")));
        assert_eq!(result,Err(ProgramError::Custom(987)));assert_eq!(f,raw_prefix(&base,failed+usize::from(partial)));
        // Explicit model transaction discard, followed by a fresh retry.
        f=base.clone();success(&mut f);
    }}
}
#[test]
fn every_account_tamper_after_each_cpi_rejects_before_state_commit() {
    let base=Fixture::new(300,100_000,false,1);
    for leg in 0..2 {for target in 0..base.base.custody.accounts.len() {let mut f=base.clone();let moves=plans(&f);
        let clock=f.base.clock.clone();let rent=f.base.custody.rent.clone();let mut count=0;
        let result=f.base.custody.with_infos(|all|execute(&PROGRAM,all,&PREPARE_DISTRIBUTION_DATA,||Ok(clock),||Ok(rent),|ix,a,seeds|{
            let n=count;count+=1;let(source,amount)=moves[n];transfer(source,amount,ix,a,seeds)?;
            if n==leg {**all[target].try_borrow_mut_lamports()?+=1;}Ok(())
        },|_|panic!("tampered event")));
        assert_eq!(result,Err(err(Piv1Error::ContributionObservationMismatch)));
        let mut expected=raw_prefix(&base,leg+1);expected.base.custody.accounts[target].lamports+=1;assert_eq!(f,expected);
    }}
    for leg in 0..2 {for target in [CONFIG,ROUND,base.guardian_start+1,bootstrap_custody::POOL,MINT] {
        let mut f=base.clone();let moves=plans(&f);let clock=f.base.clock.clone();let rent=f.base.custody.rent.clone();let mut count=0;
        let result=f.base.custody.with_infos(|all|execute(&PROGRAM,all,&PREPARE_DISTRIBUTION_DATA,||Ok(clock),||Ok(rent),|ix,a,seeds|{
            let n=count;count+=1;let(source,amount)=moves[n];transfer(source,amount,ix,a,seeds)?;
            if n==leg {all[target].try_borrow_mut_data()?[8]^=1;}Ok(())
        },|_|panic!("tampered data event")));
        assert_eq!(result,Err(err(Piv1Error::ContributionObservationMismatch)));
        let mut expected=raw_prefix(&base,leg+1);expected.base.custody.accounts[target].data[8]^=1;assert_eq!(f,expected);
    }}
}
#[test]
fn abi_host_guard_writable_borrow_and_alias_rejections_are_pre_effect() {
    let base=Fixture::new(300,100_000,false,1);
    for data in [vec![],PREPARE_DISTRIBUTION_DATA[..8].to_vec(),[PREPARE_DISTRIBUTION_DATA.as_slice(),&[0]].concat()] {
        assert_eq!(decode_prepare_distribution(&data),Err(ProgramError::InvalidInstructionData));}
    let mut wrong_version=PREPARE_DISTRIBUTION_DATA;wrong_version[8]=2;
    for (length,data,want) in [(27,wrong_version.as_slice(),ProgramError::InvalidInstructionData),
        (25,PREPARE_DISTRIBUTION_DATA.as_slice(),ProgramError::NotEnoughAccountKeys),
        (28,PREPARE_DISTRIBUTION_DATA.as_slice(),ProgramError::InvalidArgument)] {
        let mut f=base.clone();let clock=f.base.clock.clone();let rent=f.base.custody.rent.clone();
        f.base.custody.with_infos(|all|{let mut a=all.to_vec();a.resize(length,all[0].clone());
            assert_eq!(execute(&PROGRAM,&a,data,||Ok(clock),||Ok(rent),|_,_,_|panic!("ABI effects"),|_|panic!("event")),Err(want.clone()));
            assert_eq!(instruction_boundary::process_instruction(&PROGRAM,&a,data),Err(want));});assert_eq!(f,base);
    }
    for index in [CONFIG,ROUND,PENDING_SOL,PRINCIPAL_SOL,ESCROW_SOL] {let mut f=base.clone();f.base.custody.accounts[index].writable=false;
        reject(f,Some(err(Piv1Error::AccountNotWritable)));}
    let mut f=base.clone();let clock=f.base.clock.clone();let rent=f.base.custody.rent.clone();
    f.base.custody.with_infos(|a|{
        assert_eq!(instruction_boundary::process_instruction(&PROGRAM,a,&PREPARE_DISTRIBUTION_DATA),Err(ProgramError::Custom(HOST_RUNTIME_UNAVAILABLE_CODE)));
        let held=a[ROUND].try_borrow_data().unwrap();
        assert_eq!(execute(&PROGRAM,a,&PREPARE_DISTRIBUTION_DATA,||Ok(clock),||Ok(rent),|_,_,_|panic!("borrow effects"),|_|panic!("event")),Err(err(Piv1Error::AccountBorrowFailed)));drop(held);
    });assert_eq!(f,base);
    for target in [AUTHORITY,bootstrap_custody::POOL,base.guardian_start+1] {
        let mut f=base.clone();let clock=f.base.clock.clone();let rent=f.base.custody.rent.clone();
        f.base.custody.with_infos(|a|{let held=a[target].try_borrow_mut_data().unwrap();
            assert!(execute(&PROGRAM,a,&PREPARE_DISTRIBUTION_DATA,||Ok(clock),||Ok(rent),|_,_,_|panic!("borrow effects"),|_|panic!("event")).is_err());drop(held);});
        assert_eq!(f,base);
    }
    for target in [bootstrap_custody::PROTOCOL,bootstrap_custody::POOL,bootstrap_custody::LIST,MINT] {
        let mut f=base.clone();f.base.custody.accounts[target].key=key(207);reject(f,None);
    }
    let mut f=base.clone();f.base.custody.accounts[AUTHORITY].key=f.base.custody.accounts[PENDING_SOL].key;reject(f,Some(err(Piv1Error::AccountAlias)));
}
