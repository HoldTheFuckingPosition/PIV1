//! D107/D108 host accounting tests, not router CPI or public-chain evidence.
use piv1::{errors::Piv1Error, integrations::jupiter::{
    exact_book_debit, largest_safe_input, liquidity_plan, minimum_native_output,
    validate_receipt, LiquidityRoute, MAX_TOTAL_SWAP_LOSS_BPS, MIN_RETAINED_BOOK_VALUE_BPS,
}};

#[test]
fn selector_uses_actual_missing_liquidity_including_equality() {
    use LiquidityRoute::*;
    for (outgoing, pending, carry, shortfall, route) in [
        (0,0,0,0,Liquid), (1,0,0,1,Jupiter),
        (10_000_000,0,0,10_000_000,Jupiter),
        (110_200_000_000,110_000_000_000,0,200_000_000,Jupiter),
        (1_499_999_999,0,0,1_499_999_999,Jupiter),
        (1_500_000_000,0,0,1_500_000_000,DelayedWithdrawal),
        (1_500_000_001,0,0,1_500_000_001,DelayedWithdrawal),
        (15_000_000_000,10_000_000_000,0,5_000_000_000,DelayedWithdrawal),
        (15_000_000_000,15_000_000_000,0,0,Liquid),
        (800_000_000,500_000_000,300_000_000,0,Liquid),
        (800_000_000,500_000_000,100_000_000,200_000_000,Jupiter),
    ] {
        let p=liquidity_plan(outgoing,pending,carry);
        assert_eq!((p.shortfall_lamports,p.route),(shortfall,route));
        assert_eq!(p.pending_used_lamports, pending.min(outgoing));
        assert_eq!(p.pending_used_lamports+p.carry_used_lamports+shortfall,outgoing);
    }
}

#[test]
fn exact_minimum_passes_and_one_lamport_less_fails() {
    assert_eq!(MAX_TOTAL_SWAP_LOSS_BPS,500);
    assert_eq!(MIN_RETAINED_BOOK_VALUE_BPS,9500);
    for debit in [1,2,3,19,20,21,99,100,101,805_000_000,1_499_999_999] {
        let min=minimum_native_output(debit).unwrap();
        let result=validate_receipt(10_000_000_000,debit,10_000_000_000,10_000_000_000,debit,min).unwrap();
        assert_eq!(result.minimum_native_output_lamports,min);
        assert_eq!(result.actual_native_output_lamports,min);
        assert_eq!(validate_receipt(10_000_000_000,debit,10_000_000_000,10_000_000_000,debit,min-1),Err(Piv1Error::SwapMinimumNotMet));
    }
}

#[test]
fn dust_cannot_round_minimum_to_zero_and_extreme_values_are_checked() {
    for debit in 1..20 { assert_eq!(minimum_native_output(debit).unwrap(),debit); }
    let max=minimum_native_output(u64::MAX).unwrap();
    assert!(u128::from(max)*10_000>=u128::from(u64::MAX)*9500);
    assert!(u128::from(max-1)*10_000<u128::from(u64::MAX)*9500);
    assert!(minimum_native_output(0).is_err());
    assert!(exact_book_debit(u64::MAX,1,u64::MAX,1).is_err());
    assert!(exact_book_debit(1,2,1,1).is_err());
    assert!(exact_book_debit(1,1,0,1).is_err());
    assert!(exact_book_debit(1,1,1,0).is_err());
}

#[test]
fn combined_loss_is_applied_once_and_better_execution_is_not_discounted() {
    // 2% fee + 2% impact + 2% slippage exceeds the single combined 5% limit.
    assert_eq!(validate_receipt(10_000,1000,10_000,10_000,1000,940),Err(Piv1Error::SwapMinimumNotMet));
    assert!(validate_receipt(10_000,1000,10_000,10_000,1000,950).is_ok());
    let better=validate_receipt(10_000,1000,10_000,10_000,1000,1031).unwrap();
    assert_eq!(better.actual_native_output_lamports,1031);
    assert_eq!(better.official_book_debit_lamports,1000);
}

#[test]
fn exact_position_debit_includes_floor_carry_and_rejects_micro_inputs() {
    assert_eq!(exact_book_debit(4,1,5,3).unwrap(),1);
    // book(2)=3, book(1)=1; independently flooring one spent unit gives only1.
    assert_eq!(exact_book_debit(2,1,5,3).unwrap(),2);
    assert_eq!(largest_safe_input(2,5,3,1),Err(Piv1Error::InvalidSwapInput));
    assert_eq!(largest_safe_input(2,5,3,2).unwrap(),1);
    assert_eq!(validate_receipt(10_000,1,10_000,10_000,1000,1000),Err(Piv1Error::InvalidSwapInput));
    assert!(largest_safe_input(10,10,10,0).is_err());
    assert!(largest_safe_input(10,10,10,1_500_000_000).is_err());
}

#[test]
fn seeded_domain_proves_ceiling_minimality_and_maximal_safe_input() {
    // Reuse the project's dependency-free deterministic property-test approach.
    // Assert inequalities/maximality, not a copy of the inverse implementation.
    let mut state=0x4431_3038_5357_4150_u64;
    for _ in 0..4096 {
        let mut random=|| { state=state.wrapping_mul(6364136223846793005).wrapping_add(1); state };
        let debit=random()|1;
        let min=minimum_native_output(debit).unwrap();
        assert!(u128::from(min)*10_000>=u128::from(debit)*9500);
        assert!(u128::from(min-1)*10_000<u128::from(debit)*9500);
        let supply=1+random()%200; let total=1+random()%400;
        let units=1+random()%supply; let budget=1+random()%400;
        let old=units*total/supply;
        let feasible=(1..=units).filter(|q| { let d=old-(units-q)*total/supply; d>0 && d<=budget }).max();
        match (feasible,largest_safe_input(units,total,supply,budget)) {
            (Some(expected),Ok(q))=>assert_eq!(q,expected),
            (None,Err(Piv1Error::InvalidSwapInput))=>(),
            pair=>panic!("inverse disagrees with exhaustive feasible set: {pair:?}"),
        }
    }
}
