//! Pinned SPL 2.0.3 active-source and Phase0 checked preparation proofs. This is
//! not a withdrawal execution adapter. Stake 5.1.0 split feasibility assumes the
//! future destination is a 200-byte uninitialized stake prefunded with current
//! Rent; preparation checks available operational funding, not a future receipt.
use anchor_lang::{prelude::{AccountInfo, Clock, Pubkey, Rent}, AnchorDeserialize};
use solana_stake_interface::{state::{StakeStateV2, Lockup}, stake_flags::StakeFlags};
use crate::{accounts::{STAKE_PROGRAM_ID, rent_floor}, errors::{Piv1Error, Piv1Result},
    integrations::jito_identity::{AuthenticatedJitoIdentity, RawFee}, state::WithdrawalLeg};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct WithdrawalProof {
    pub target: u64, pub minimum: u64, pub maximum_legs: u64,
    pub round_minimum: u64, pub conversion_dust: u64,
}
impl WithdrawalProof { pub fn insufficient(self) -> bool { self.target < self.minimum } }

#[derive(Clone, Copy)]
struct Ratio { total: u64, supply: u64, fee: RawFee }
impl Ratio {
    fn book(self, q: u64) -> Piv1Result<u64> { mul_floor(q,self.total,self.supply) }
    fn fee(self, q: u64) -> Piv1Result<u64> {
        if self.fee.numerator == 0 { return Ok(0); }
        let product=u128::from(q)*u128::from(self.fee.numerator);
        let d=u128::from(self.fee.denominator);
        narrow(product.checked_add(d-1).ok_or(Piv1Error::ArithmeticOverflow)?/d)
    }
    fn redeem(self,q:u64)->Piv1Result<u64>{self.book(q.checked_sub(self.fee(q)?).ok_or(Piv1Error::ArithmeticOverflow)?)}
    fn target(self,budget:u64,held:u64)->Piv1Result<u64>{
        let top=(u128::from(budget)+1).checked_mul(u128::from(self.supply))
            .and_then(|v|v.checked_sub(1)).ok_or(Piv1Error::ArithmeticOverflow)?;
        // Clamp in u128 before narrowing: a tiny pool ratio can exceed u64.
        narrow((top/u128::from(self.total)).min(u128::from(held)))
    }
    fn minimum(self,native:u64,limit:u64)->Piv1Result<u64>{
        if native==0||limit==0||self.redeem(limit)?<native{return Err(Piv1Error::TechnicalFloorNotMet);}
        let(mut lo,mut hi)=(1,limit);
        while lo<hi {let mid=lo+(hi-lo)/2;if self.redeem(mid)? >= native {hi=mid;}else{lo=mid+1;}}
        Ok(lo)
    }
    fn capacity(self,native:u64,limit:u64)->Piv1Result<u64>{
        let(mut lo,mut hi)=(0,limit);
        while lo<hi {let gap=hi-lo;let mid=lo+gap/2+gap%2;
            if self.redeem(mid)?<=native {lo=mid;}else{hi=mid-1;}}
        Ok(lo)
    }
    fn round_floor(self,target:u64,minimum:u64,slippage:u16)->Piv1Result<(u64,u64)>{
        if slippage>1{return Err(Piv1Error::InvalidSlippage);}
        let legs=target.checked_div(minimum).filter(|v|*v>0).ok_or(Piv1Error::TechnicalFloorNotMet)?;
        let reserve=if self.fee.numerator==0{0}else{legs-1};
        let burn=target.checked_sub(self.fee(target)?).and_then(|v|v.checked_sub(reserve)).ok_or(Piv1Error::ArithmeticOverflow)?;
        let lower=self.book(burn)?.checked_sub(legs-1).ok_or(Piv1Error::ArithmeticOverflow)?;
        let floor=mul_floor(lower,10000-u64::from(slippage),10000)?;
        if floor==0{return Err(Piv1Error::TechnicalFloorNotMet);} Ok((legs,floor))
    }
}
fn narrow(v:u128)->Piv1Result<u64>{u64::try_from(v).map_err(|_|Piv1Error::ArithmeticOverflow)}
fn mul_floor(a:u64,b:u64,d:u64)->Piv1Result<u64>{
    narrow((u128::from(a)*u128::from(b)).checked_div(u128::from(d)).ok_or(Piv1Error::ArithmeticOverflow)?)
}
fn add(a:u64,b:u64)->Piv1Result<u64>{a.checked_add(b).ok_or(Piv1Error::ArithmeticOverflow)}
fn bad()->Piv1Error{Piv1Error::InvalidCustodyObservation}
fn read_u64(a:&[u8])->Piv1Result<u64>{Ok(u64::from_le_bytes(a.try_into().map_err(|_|bad())?))}

#[allow(clippy::too_many_arguments)]
pub(crate) fn derive(protocol:&AuthenticatedJitoIdentity,list:&AccountInfo<'_>,source:&AccountInfo<'_>,
    index:u32,clock:&Clock,rent:&Rent,minimum_delegation:u64,operational_spendable:u64,
    budget:u64,historical_units:u64,slippage:u16)->Piv1Result<WithdrawalProof>{
    let pool=protocol.pool();let ratio=Ratio{total:pool.total_lamports(),supply:pool.pool_token_supply(),fee:pool.stake_withdrawal_fee()};
    if ratio.total==0||ratio.supply==0||minimum_delegation==0||clock.epoch==u64::MAX{return Err(bad());}
    let stake_rent=rent_floor(rent,StakeStateV2::size_of())?;
    if operational_spendable<add(stake_rent,rent_floor(rent,WithdrawalLeg::SPACE)?)? {return Err(Piv1Error::AccountRentDeficit);}
    let (active,epoch,suffix,status,vote)={
        let data=list.try_borrow_data().map_err(|_|Piv1Error::AccountBorrowFailed)?;
        let count=u32::from_le_bytes(data.get(5..9).ok_or(bad())?.try_into().map_err(|_|bad())?);
        if index>=count{return Err(bad());}
        let start=usize::try_from(index).map_err(|_|bad())?.checked_mul(73).and_then(|v|v.checked_add(9)).ok_or(Piv1Error::ArithmeticOverflow)?;
        let r=data.get(start..start.checked_add(73).ok_or(Piv1Error::ArithmeticOverflow)?).ok_or(bad())?;
        (read_u64(&r[..8])?,read_u64(&r[16..24])?,u32::from_le_bytes(r[36..40].try_into().map_err(|_|bad())?),r[40],
            Pubkey::new_from_array(r[41..73].try_into().map_err(|_|bad())?))
    };
    if epoch!=clock.epoch||status!=0||vote==Pubkey::default(){return Err(bad());}
    if pool.preferred_withdraw_validator_vote_address().is_some_and(|preferred|preferred!=vote){return Err(bad());}
    let suffix_bytes=suffix.to_le_bytes();
    let seeds:Vec<&[u8]>=if suffix==0 {vec![vote.as_ref(),list_key_pool(protocol).as_ref()]}else{vec![vote.as_ref(),list_key_pool(protocol).as_ref(),&suffix_bytes]};
    let expected=Pubkey::try_find_program_address(&seeds,&crate::integrations::jito_identity::JITO_STAKE_POOL_PROGRAM).ok_or(bad())?.0;
    if source.key!=&expected||source.owner!=&STAKE_PROGRAM_ID||source.executable{return Err(bad());}
    let source_data=source.try_borrow_data().map_err(|_|Piv1Error::AccountBorrowFailed)?;
    if source_data.len()!=StakeStateV2::size_of(){return Err(bad());}
    let state=StakeStateV2::deserialize(&mut &source_data[..]).map_err(|_|bad())?;
    let StakeStateV2::Stake(meta,stake,flags)=state else{return Err(bad());};
    let withdraw=Pubkey::try_find_program_address(&[list_key_pool(protocol).as_ref(),b"withdraw"],&crate::integrations::jito_identity::JITO_STAKE_POOL_PROGRAM).ok_or(bad())?.0;
    let lockup=pool.lockup();
    if meta.authorized.staker!=withdraw||meta.authorized.withdrawer!=withdraw
        ||meta.lockup!=(Lockup{unix_timestamp:lockup.unix_timestamp,epoch:lockup.epoch,custodian:lockup.custodian})
        ||meta.lockup.is_in_force(clock,None)
        ||stake.delegation.voter_pubkey!=vote||stake.delegation.stake==0
        ||stake.delegation.deactivation_epoch!=u64::MAX||flags!=StakeFlags::empty()
        ||(stake.delegation.activation_epoch!=u64::MAX&&stake.delegation.activation_epoch>clock.epoch){return Err(bad());}
    let balance=**source.try_borrow_lamports().map_err(|_|Piv1Error::AccountBorrowFailed)?;
    if balance!=active||balance.checked_sub(stake.delegation.stake).ok_or(bad())?<stake_rent{return Err(bad());}
    // SPL 2.0.3 MINIMUM_ACTIVE_STAKE is a pinned protocol constant, never
    // a substituted cluster delegation minimum. D above came from runtime CPI.
    let required=add(meta.rent_exempt_reserve,minimum_delegation.max(1_000_000))?;
    let tolerance=narrow((u128::from(ratio.total)+u128::from(ratio.supply)-1)/u128::from(ratio.supply))?;
    // This actual current record witnesses that SPL must select Active; there
    // is no full list scan, transient/reserve/removal or preferred fallback.
    if active<=add(required,tolerance)?{return Err(bad());}
    let available=balance.checked_sub(required).ok_or(bad())?
        .min(stake.delegation.stake.checked_sub(minimum_delegation).ok_or(bad())?);
    let minimum=ratio.minimum(minimum_delegation,protocol.mint().supply)?;
    let capacity=ratio.capacity(available,protocol.mint().supply)?;
    if capacity<minimum{return Err(Piv1Error::TechnicalFloorNotMet);}
    let target=ratio.target(budget,historical_units)?;
    if target<minimum{return Ok(WithdrawalProof{target,minimum,maximum_legs:0,round_minimum:0,conversion_dust:0});}
    let first=target.min(capacity);let remaining=target-first;
    if first<minimum||(remaining!=0&&remaining<minimum){return Err(Piv1Error::TechnicalFloorNotMet);}
    let (maximum_legs,round_minimum)=ratio.round_floor(target,minimum,slippage)?;
    let conversion_dust=budget.checked_sub(ratio.book(target)?).ok_or(Piv1Error::ArithmeticOverflow)?;
    Ok(WithdrawalProof{target,minimum,maximum_legs,round_minimum,conversion_dust})
}
fn list_key_pool(_: &AuthenticatedJitoIdentity)->&'static Pubkey {&crate::integrations::jito_identity::JITO_STAKE_POOL}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn inverse_and_monotone_searches_match_exhaustive_small_domain() {
        for total in 1..13 {for supply in 1..13 {for fee in [RawFee{numerator:0,denominator:0},RawFee{numerator:1,denominator:7}] {
            let r=Ratio{total,supply,fee};
            for budget in 0..25 {let brute=(0..=30).filter(|q|r.book(*q).unwrap()<=budget).max().unwrap();
                assert_eq!(r.target(budget,30).unwrap(),brute);
                let capacity=(0..=30).filter(|q|r.redeem(*q).unwrap()<=budget).max().unwrap();
                assert_eq!(r.capacity(budget,30).unwrap(),capacity);
                if budget!=0 {let minimum=(1..=30).find(|q|r.redeem(*q).unwrap()>=budget);
                    match minimum {Some(q)=>assert_eq!(r.minimum(budget,30).unwrap(),q),None=>assert_eq!(r.minimum(budget,30),Err(Piv1Error::TechnicalFloorNotMet))}}
            }
        }}}
    }
    #[test]
    fn split_fee_and_floor_reserves_bound_actual_partitions_at_zero_slippage() {
        for fee in [RawFee{numerator:0,denominator:0},RawFee{numerator:1,denominator:11}] {
            let r=Ratio{total:17,supply:13,fee};
            for target in 12..65 {let minimum=5;
                if let Ok((legs,floor))=r.round_floor(target,minimum,0) {
                    assert_eq!(legs,target/minimum);
                    for first in minimum..=target-minimum {let actual=r.redeem(first).unwrap()+r.redeem(target-first).unwrap();assert!(floor<=actual);}
                    let mut remaining=target;let mut actual=0;
                    while remaining>=2*minimum {actual+=r.redeem(minimum).unwrap();remaining-=minimum;}
                    actual+=r.redeem(remaining).unwrap();assert!(floor<=actual);
                }
            }
        }
    }
    #[test]
    fn checked_boundaries_clamp_before_narrowing_and_reject_impossible_fee_or_floor() {
        let r=Ratio{total:1,supply:u64::MAX,fee:RawFee{numerator:0,denominator:0}};
        assert_eq!(r.target(u64::MAX,u64::MAX).unwrap(),u64::MAX);
        let r=Ratio{total:u64::MAX,supply:u64::MAX,fee:RawFee{numerator:0,denominator:7}};
        assert_eq!(r.minimum(u64::MAX,u64::MAX).unwrap(),u64::MAX);
        assert_eq!(r.capacity(u64::MAX,u64::MAX).unwrap(),u64::MAX);
        assert_eq!(r.round_floor(u64::MAX,u64::MAX,0).unwrap(),(1,u64::MAX));
        assert_eq!(r.round_floor(10,1,2),Err(Piv1Error::InvalidSlippage));
        let r=Ratio{fee:RawFee{numerator:1,denominator:1},..r};
        assert_eq!(r.minimum(1,u64::MAX),Err(Piv1Error::TechnicalFloorNotMet));
        assert_eq!(r.round_floor(10,1,0),Err(Piv1Error::ArithmeticOverflow));
    }
}
