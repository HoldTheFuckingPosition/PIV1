//! Production-boundary host effects with independent complete-state oracles.
//! Failed-world discard is explicit; no SVM/Bank/nested-pool claim is made.
mod support;
#[path="support/pending_custody.rs"]pub mod pending_custody;
#[allow(dead_code)]#[path="support/economic_custody.rs"]mod custody;
#[path="support/jito_identity_oracle.rs"]mod oracle;
#[allow(dead_code)]#[path="support/bootstrap_custody.rs"]mod bootstrap_custody;
#[allow(dead_code)]#[path="support/distribution_preparation_custody.rs"]mod preparation_custody;
#[allow(dead_code)]#[path="support/withdrawal_preparation_custody.rs"]mod withdrawal_custody;
#[path="support/withdrawal_leg_custody.rs"]mod leg_custody;
use support::kif_claim_custody;
use piv1::integrations;
use anchor_lang::{prelude::Pubkey,AnchorDeserialize,solana_program::{program_error::ProgramError}};
use piv1::{accounts::STAKE_PROGRAM_ID,instruction_boundary,instruction_errors::{piv1_error_code,HOST_RUNTIME_UNAVAILABLE_CODE},
    errors::Piv1Error,events::WithdrawalLegInitiated,state::WithdrawalLeg,
    instructions::initiate_withdrawal_leg::*,withdrawal_leg_execution::process_instruction_with_host_callbacks as execute};
use solana_stake_interface::state::StakeStateV2;
use custody::*;use bootstrap_custody::{POOL,LIST,MANAGER};use leg_custody::*;
use support::kif_claim_custody::{key,PROGRAM};
fn error(e:Piv1Error)->ProgramError{ProgramError::Custom(piv1_error_code(e))}
#[derive(Clone,Copy,Debug)]
enum Behavior{Good,Before(usize),After(usize),Noop(usize),Data(usize,usize,usize),Lamports(usize,usize),HoldFinal}
fn raw(f:&mut leg_custody::Fixture,behavior:Behavior)->(Result<(),ProgramError>,usize,Vec<WithdrawalLegInitiated>){
    let reference=expected(f);let initial=f.custody.accounts.clone();let data=f.data();let clock=f.clock.clone();let rent=f.custody.rent.clone();
    let minimum=f.minimum;let base=f.base;let mut count=0;let mut events=vec![];
    let (result,accounts)=f.custody.with_infos(|all|{
        let mut held=None;
        let result=execute(&PROGRAM,all,&data,||Ok(clock),||Ok(rent),|ix,infos,seeds|{
            let i=count;count+=1;
            if matches!(behavior,Behavior::Before(j)if j==i){return Err(ProgramError::Custom(24101));}
            if matches!(behavior,Behavior::Noop(j)if j==i){return Ok(());}
            let before=if i==0{&initial}else{&reference.calls[i-1].after};
            emulate(before,&reference.calls[i],ix,infos,seeds,all)?;
            if matches!(behavior,Behavior::After(j)if j==i){return Err(ProgramError::Custom(24102));}
            if let Behavior::Data(j,account,offset)=behavior{if i==j{all[account].try_borrow_mut_data()?[offset]^=1;}}
            if let Behavior::Lamports(j,account)=behavior{if i==j{**all[account].try_borrow_mut_lamports()?+=1;}}
            if matches!(behavior,Behavior::HoldFinal)&&i+1==reference.calls.len(){held=Some(all[base+4].try_borrow_mut_data()?);}
            Ok(())
        },||Some((STAKE_PROGRAM_ID,minimum.to_le_bytes().to_vec())),|e|events.push(e));
        drop(held);(result,all.iter().map(snapshot).collect::<Vec<_>>())
    });f.custody.accounts=accounts;(result,count,events)
}
fn success(f:&mut leg_custody::Fixture){
    let expect=expected(f);let(result,calls,events)=raw(f,Behavior::Good);assert_eq!(result,Ok(()));
    assert_eq!(calls,expect.calls.len());assert_eq!(events,vec![expect.event]);assert_eq!(*f,expect.after);
    let observed=WithdrawalLeg::deserialize(&mut &f.custody.accounts[f.base+4].data[8..]).unwrap();assert_eq!(observed,expect.leg);
}
fn reject(mut f:leg_custody::Fixture,expected_error:Option<ProgramError>){
    let before=f.clone();let clock=f.clock.clone();let rent=f.custody.rent.clone();let data=f.data();let minimum=f.minimum;let mut calls=0;
    let result=f.custody.with_infos(|a|execute(&PROGRAM,a,&data,||Ok(clock),||Ok(rent),|ix,_,_|{
        calls+=1;assert_eq!(ix.program_id,STAKE_PROGRAM_ID);assert_eq!(ix.data,vec![13,0,0,0]);Ok(())
    },||Some((STAKE_PROGRAM_ID,minimum.to_le_bytes().to_vec())),|_|panic!("no failure event")));
    assert!(calls<=1);if let Some(error)=expected_error{assert_eq!(result,Err(error));}else{assert!(result.is_err());}assert_eq!(f,before);
}

#[test]
fn exact_complete_state_oracle_covers_fee_roles_prefunds_rents_and_burn_lag(){
    for shared in [false,true]{for prefunds in [[0,0],[1,17],[9_000_000,7_000_000]]{for zero_fee in [false,true]{
        let mut f=leg_custody::Fixture::new(shared);let mut pool=f.pool();
        if zero_fee{pool.stake_withdrawal_fee=oracle::Fee{numerator:0,denominator:0};f.set_pool(&pool);}
        let b=f.base;f.custody.accounts[b+4].lamports=prefunds[0];f.custody.accounts[b+5].lamports=prefunds[1];
        let mint=read_u64(&f.custody.accounts[MINT].data,36)-29;f.custody.accounts[MINT].data[36..44].copy_from_slice(&mint.to_le_bytes());
        f.custody.accounts[PRINCIPAL_JITO].lamports+=21;f.custody.accounts[PENDING_JITO].lamports+=33;
        let original=f.clone();success(&mut f);
        assert_eq!(f.custody.config().accounted_pending_sol_lamports,original.custody.config().accounted_pending_sol_lamports+prefunds.iter().sum::<u64>());
        assert_eq!(f.round().pending_sol_used_lamports,321);assert_eq!(f.round().prior_next_cycle_yield_used_lamports().unwrap(),117);
        assert_eq!(f.custody.config().protected_principal_hwm_lamports,original.custody.config().protected_principal_hwm_lamports);
        assert_eq!(f.custody.accounts[POOL].data[274..],original.custody.accounts[POOL].data[274..]);
        assert_eq!(f.custody.accounts[b+2].data[197..],original.custody.accounts[b+2].data[197..]);
    }}}
    // Source meta reserve remains authentic old metadata; actual runtime Rent is
    // higher and paid in full for both new accounts, never confused with pseudo rent.
    let mut f=leg_custody::Fixture::new(false);let old=f.custody.rent.clone();f.custody.rent.lamports_per_byte_year+=1;
    for (i,a) in f.custody.accounts.iter_mut().enumerate(){if i==f.base||i==f.base+1||i==f.base+3||i>=f.base+4||i==AUTHORITY{continue;}
        let difference=f.custody.rent.minimum_balance(a.data.len())-old.minimum_balance(a.data.len());a.lamports+=difference;
    }
    let source=f.custody.accounts[f.base+2].lamports;f.custody.accounts[LIST].data[9..17].copy_from_slice(&source.to_le_bytes());
    f.custody.accounts[OPERATIONAL_SOL].lamports+=10_000;
    let expected_rent=f.custody.rent.minimum_balance(200);assert_ne!(expected_rent,2_282_880);success(&mut f);
    let leg=WithdrawalLeg::deserialize(&mut &f.custody.accounts[f.base+4].data[8..]).unwrap();assert_eq!(leg.stake_rent_advanced_lamports,expected_rent);
    let StakeStateV2::Stake(meta,_,_)=StakeStateV2::deserialize(&mut &f.custody.accounts[f.base+5].data[..]).unwrap()else{panic!()};
    assert_eq!(meta.rent_exempt_reserve,2_282_880);
}

#[test]
fn two_current_active_sources_fill_exact_target_and_preserve_first_record(){
    let mut f=leg_custody::Fixture::new(false);f.source_capacity(4_000);
    let extra=f.custody.rent.minimum_balance(200)+f.custody.rent.minimum_balance(WithdrawalLeg::SPACE);
    f.custody.accounts[OPERATIONAL_SOL].lamports+=extra;
    let mut list=f.custody.accounts[LIST].data.clone();list[1..5].copy_from_slice(&2_u32.to_le_bytes());list[5..9].copy_from_slice(&2_u32.to_le_bytes());
    let mut second=list[9..82].to_vec();second[41..73].copy_from_slice(key(203).as_ref());list.extend(second);
    f.custody.accounts[LIST].data=list;f.custody.accounts[LIST].lamports=f.custody.rent.minimum_balance(155);
    let fixed=f.round().fixed_jitosol_withdrawal_target_units;success(&mut f);assert!(f.round().cumulative_jitosol_assigned_units<fixed);
    let first=f.custody.accounts[f.base+4..].to_vec();let first_list=f.custody.accounts[LIST].data[9..82].to_vec();
    let StakeStateV2::Stake(meta,mut stake,flags)=f.source()else{panic!()};stake.delegation.voter_pubkey=key(203);stake.delegation.stake=1_004_000;
    f.set_source(StakeStateV2::Stake(meta,stake,flags));f.custody.accounts[f.base+2].key=Pubkey::find_program_address(&[key(203).as_ref(),f.custody.config().stake_pool.as_ref()],&f.custody.config().stake_pool_program).0;
    f.custody.accounts[f.base+2].lamports=meta.rent_exempt_reserve+1_004_000;f.index=1;f.new_pair();
    // The second candidate safely fills the remaining fixed target.
    success(&mut f);assert_eq!(f.round().cumulative_jitosol_assigned_units,fixed);assert_eq!(f.round().successful_leg_count,2);
    assert_eq!(f.custody.accounts[LIST].data[9..82],first_list);assert_eq!(first[0].data[8+4],1); // Initiated status.
    assert!(f.round().cumulative_delegated_native_lamports>=f.round().stored_round_minimum_native_lamports);
    let before=f.clone();reject(f,Some(error(Piv1Error::TargetExceeded)));assert_eq!(before.round().next_leg_index,2);
}

#[test]
fn fresh_fee_minimum_source_and_round_floor_drift_fail_before_creation(){
    let base=leg_custody::Fixture::new(false);
    let mut f=base.clone();f.minimum=10_000;reject(f,Some(error(Piv1Error::TechnicalFloorNotMet)));
    let mut f=base.clone();let mut p=f.pool();p.stake_withdrawal_fee=oracle::Fee{numerator:1,denominator:2};f.set_pool(&p);reject(f,Some(error(Piv1Error::TechnicalFloorNotMet)));
    let mut f=base.clone();let mut p=f.pool();p.total_lamports-=100_000;f.set_pool(&p);reject(f,None);
    let mut f=base.clone();let q=f.round().fixed_jitosol_withdrawal_target_units-50;f.source_capacity(q-q.div_ceil(1000));reject(f,Some(error(Piv1Error::TechnicalFloorNotMet))); // Stranded input remainder.
    let mut f=base.clone();f.source_capacity(4_000);let mut r=f.round();r.stored_round_minimum_native_lamports=r.fixed_jitosol_withdrawal_target_units;f.set_round(r);reject(f,None);
    let mut f=base.clone();let mut p=f.pool();p.preferred_withdraw_validator_vote_address=Some(key(204));f.set_pool(&p);reject(f,None);
    let mut f=base.clone();f.custody.accounts[LIST].data[25..33].copy_from_slice(&39_u64.to_le_bytes());reject(f,None);
    let mut f=base.clone();f.custody.accounts[f.base+2].lamports+=1;reject(f,None);
    let mut f=base.clone();let mut p=f.pool();p.last_update_epoch-=1;f.set_pool(&p);reject(f,None);
    // A later q_protocol increase cannot shrink the future rounding reserve:
    // it can fall again, so the immutable snapshot minimum must remain binding.
    let mut f=base;assert_eq!(f.round().fixed_jitosol_withdrawal_target_units,7705);
    assert_eq!(f.round().snapshot_leg_input_floor_units,101);assert_eq!(f.round().maximum_useful_legs,76);
    assert_eq!(f.round().stored_round_minimum_native_lamports,7546);
    f.minimum=200;let mut p=f.pool();p.stake_withdrawal_fee=oracle::Fee{numerator:12,denominator:1000};f.set_pool(&p);f.source_capacity(4000);
    let remaining=7705_u64-4049;let future_fee=(remaining*12).div_ceil(1000);
    let snapshot_legs=remaining/101;let current_legs=remaining/203;
    assert_eq!(4000+remaining-future_fee-2*(snapshot_legs-1),7542);
    assert_eq!(4000+remaining-future_fee-2*(current_legs-1),7578);
    reject(f,Some(error(Piv1Error::TechnicalFloorNotMet)));
}

#[test]
fn strict_abi_roles_pause_replay_aliases_and_prior_custody_are_rejected(){
    let base=leg_custody::Fixture::new(false);
    for n in 0..16{if n==13{continue;}let mut d=base.data();d.resize(n,0);assert_eq!(decode_initiate_withdrawal_leg(&d),Err(ProgramError::InvalidInstructionData));}
    for change in [0,8]{let mut f=base.clone();let mut d=f.data();d[change]^=1;let rent=f.custody.rent.clone();let clock=f.clock.clone();
        f.custody.with_infos(|a|assert_eq!(execute(&PROGRAM,a,&d,||Ok(clock),||Ok(rent),|_,_,_|panic!(),||None,|_|panic!()),Err(ProgramError::InvalidInstructionData)));}
    for count in [23,24,26]{let mut f=base.clone();if count==26{f.custody.accounts.push(f.custody.accounts[0].clone());}else{f.custody.accounts.truncate(count);}reject(f,None);}
    let mut f=base.clone();let data=f.data();f.custody.with_infos(|a|assert_eq!(instruction_boundary::process_instruction(&PROGRAM,a,&data),Err(ProgramError::Custom(HOST_RUNTIME_UNAVAILABLE_CODE))));
    let mut f=base.clone();f.custody.edit_config(|c|c.paused=true);reject(f,Some(error(Piv1Error::PausedOperation)));
    let mut f=base.clone();f.custody.accounts[PENDING_SOL].lamports-=1;reject(f,None);
    let mut f=base.clone();f.custody.accounts[PRINCIPAL_JITO].data[64..72].copy_from_slice(&1_u64.to_le_bytes());reject(f,None);
    let mut f=base.clone();f.custody.accounts[f.base+5].key=f.custody.accounts[f.base+4].key;reject(f,Some(error(Piv1Error::AccountAlias)));
    let mut f=base.clone();f.custody.accounts[f.base+4].owner=PROGRAM;reject(f,None);
    let mut f=base.clone();f.custody.accounts[f.base+5].data=vec![0;200];reject(f,None);
    let mut f=base;f.source_capacity(4_000);success(&mut f);reject(f,Some(error(Piv1Error::InvalidAccountPda))); // Already-owned old pair.
}

#[test]
fn canonical_pdas_rent_writability_and_hostile_borrows_fail_without_effects(){
    let base=leg_custody::Fixture::new(false);
    for i in [0,1,2,4,7,MINT,POOL,LIST,MANAGER,base.base+2,base.base+4,base.base+5]{let mut f=base.clone();f.custody.accounts[i].writable=false;reject(f,None);}
    let mut f=base.clone();f.custody.accounts[OPERATIONAL_SOL].lamports=f.custody.rent.minimum_balance(0);reject(f,Some(error(Piv1Error::AccountRentDeficit)));
    for i in [AUTHORITY,base.base+3,base.base+4,base.base+5]{let mut f=base.clone();f.custody.accounts[i].key=key(208);reject(f,None);}
    for i in [0,1,AUTHORITY,base.base+3,base.base+4]{for data_borrow in [false,true]{let mut f=base.clone();let before=f.clone();let data=f.data();let clock=f.clock.clone();let rent=f.custody.rent.clone();
        f.custody.with_infos(|a|{let data_guard=if data_borrow{Some(a[i].try_borrow_mut_data().unwrap())}else{None};
            let lamports_guard=if !data_borrow{Some(a[i].try_borrow_mut_lamports().unwrap())}else{None};
            assert!(execute(&PROGRAM,a,&data,||Ok(clock),||Ok(rent),|_,_,_|panic!(),||None,|_|panic!()).is_err());drop(data_guard);drop(lamports_guard);});assert_eq!(f,before);
    }}
}

#[test]
fn minimum_query_authenticates_origin_length_value_and_unchanged_world(){
    for response in [None,Some((key(44),100_u64.to_le_bytes().to_vec())),Some((STAKE_PROGRAM_ID,vec![0;7])),Some((STAKE_PROGRAM_ID,vec![0;9])),Some((STAKE_PROGRAM_ID,vec![0;8]))]{
        let mut f=leg_custody::Fixture::new(false);let before=f.clone();let data=f.data();let clock=f.clock.clone();let rent=f.custody.rent.clone();let mut calls=0;
        let result=f.custody.with_infos(|a|execute(&PROGRAM,a,&data,||Ok(clock),||Ok(rent),|ix,_,_|{calls+=1;assert_eq!(ix.data,vec![13,0,0,0]);Ok(())},||response,|_|panic!()));
        assert!(result.is_err());assert_eq!(calls,1);assert_eq!(f,before);
    }
    for i in [CONFIG,ROUND,POOL,MINT,LIST]{let mut f=leg_custody::Fixture::new(false);let before=f.clone();let(result,count,events)=raw(&mut f,Behavior::Data(0,i,0));
        assert!(result.is_err());assert_eq!(count,1);assert!(events.is_empty());assert_ne!(f,before);}
}

#[test]
fn every_cpi_error_preserves_exact_raw_prefix_and_explicit_discard_allows_retry(){
    let mut original=leg_custody::Fixture::new(false);original.custody.accounts[original.base+4].lamports=7;original.custody.accounts[original.base+5].lamports=11;
    let reference=expected(&original);
    for i in 0..reference.calls.len(){for after in [false,true]{let mut staged=original.clone();let behavior=if after{Behavior::After(i)}else{Behavior::Before(i)};
        let(result,count,events)=raw(&mut staged,behavior);assert_eq!(result,Err(ProgramError::Custom(if after{24102}else{24101})));assert_eq!(count,i+1);assert!(events.is_empty());
        let prefix=if after{&reference.calls[i].after}else if i==0{&original.custody.accounts}else{&reference.calls[i-1].after};
        assert_eq!(&staged.custody.accounts,prefix);assert_eq!(staged.custody.accounts[CONFIG],original.custody.accounts[CONFIG]);assert_eq!(staged.custody.accounts[ROUND],original.custody.accounts[ROUND]);
        let mut committed=original.clone(); // Explicitly discard the failed staged world.
        success(&mut committed);
    }}
}

#[test]
fn noops_bad_account_deltas_stake_authorities_and_final_borrow_never_commit(){
    let original=leg_custody::Fixture::new(false);let reference=expected(&original);let spl=reference.calls.len()-2;let last=spl+1;
    for i in 1..reference.calls.len(){let mut f=original.clone();let(result,count,events)=raw(&mut f,Behavior::Noop(i));assert!(result.is_err());assert_eq!(count,i+1);assert!(events.is_empty());assert_eq!(f.custody.accounts[CONFIG],original.custody.accounts[CONFIG]);assert_eq!(f.custody.accounts[ROUND],original.custody.accounts[ROUND]);}
    for (call,account,offset) in [(spl,original.base+5,12),(spl,original.base+5,44),(spl,original.base+2,156),
        (spl,MANAGER,64),(spl,LIST,81),(spl,POOL,274),(spl,MINT,36),(last,original.base+5,172),(last,CONFIG,0),(last,ROUND,0)]{
        let mut f=original.clone();let(result,count,events)=raw(&mut f,Behavior::Data(call,account,offset));assert!(result.is_err());assert_eq!(count,call+1);assert!(events.is_empty());
    }
    for call in 1..reference.calls.len(){let mut f=original.clone();let(result,count,_)=raw(&mut f,Behavior::Lamports(call,PENDING_SOL));assert!(result.is_err());assert_eq!(count,call+1);}
    let mut f=original.clone();let(result,_,events)=raw(&mut f,Behavior::HoldFinal);assert_eq!(result,Err(error(Piv1Error::AccountBorrowFailed)));assert!(events.is_empty());
    assert_eq!(f.custody.accounts[CONFIG],original.custody.accounts[CONFIG]);assert_eq!(f.custody.accounts[ROUND],original.custody.accounts[ROUND]);assert!(f.custody.accounts[f.base+4].data.iter().all(|b|*b==0));
}

#[test]
fn checked_credit_counter_and_prefund_overflow_reject_before_creation(){
    let base=leg_custody::Fixture::new(false);
    let mut f=base.clone();f.custody.accounts[f.base+4].lamports=u64::MAX;f.custody.accounts[f.base+5].lamports=1;reject(f,Some(error(Piv1Error::ArithmeticOverflow)));
    let mut f=base.clone();f.custody.accounts[MANAGER].data[64..72].copy_from_slice(&u64::MAX.to_le_bytes());reject(f,Some(error(Piv1Error::ArithmeticOverflow)));
    let mut f=base.clone();f.minimum=0;reject(f,Some(error(Piv1Error::TechnicalFloorNotMet)));
    let mut f=base;let mut r=f.round();r.next_leg_index=u64::MAX;f.set_round(r);f.new_pair();reject(f,None);
}

#[test]
fn deterministic_numeric_domain_matches_independent_maximum_fill_and_conservation(){
    let mut cases=0;
    for slippage in [0,1]{for capacity in [3500,4000,5000,20_000]{for numerator in [0,1,2]{
        let mut w=withdrawal_custody::Fixture::new(321,117,false);w.inner.base.custody.edit_config(|c|c.configured_slippage_bps=slippage);
        w.inner.base.pool.stake_withdrawal_fee=oracle::Fee{numerator,denominator:1000};w.inner.base.sync();
        let mut f=leg_custody::Fixture::from_preparation(w);f.source_capacity(capacity);let before=f.clone();let expected=expected(&f);
        let output=expected.leg.observed_delegated_native_lamports;let q=expected.leg.jitosol_input_units;success(&mut f);cases+=1;
        assert!(output<=capacity);if q<before.round().remaining_withdrawal_target_units().unwrap(){
            let p=before.pool();let next=q+1;let fee=if numerator==0{0}else{(next*numerator).div_ceil(1000)};assert!(next-fee>capacity);
            assert!(p.total_lamports>0);
        }
        assert_eq!(before.custody.token_units(PRINCIPAL_JITO)-f.custody.token_units(PRINCIPAL_JITO),q);
        assert_eq!(before.custody.accounts[before.base+2].lamports-f.custody.accounts[f.base+2].lamports,output);
        assert_eq!(f.custody.accounts[f.base+5].lamports,output+f.custody.rent.minimum_balance(200));
    }}}
    assert_eq!(cases,24); // No filtered/vacuously accepted generated cases.
}
