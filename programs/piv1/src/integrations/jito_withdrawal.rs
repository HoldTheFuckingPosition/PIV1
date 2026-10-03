//! Pinned SPL 2.0.3 protected active withdrawal, with Stake 5.1.0 split facts.
//! This is the bounded initiation component, not finalization or a mock adapter.
use anchor_lang::{prelude::{AccountInfo,Clock,Pubkey,Rent},solana_program::instruction::{AccountMeta,Instruction}};
use crate::{errors::{Piv1Error,Piv1Result},state::{PivConfig,ActiveDistribution},
    integrations::{jito_identity::AuthenticatedJitoIdentity,jito_withdrawal_preparation::{Ratio,ActiveSource,active_source}}};

/// Pinned Stake 5.1.0 metadata constant; NEVER the operational rent advance.
/// Upstream processor split writes this even when actual current Rent differs.
pub(crate) const STAKE_PSEUDO_RENT:u64=2_282_880;
#[derive(Clone,Copy,Debug,PartialEq)]
pub(crate) struct LegQuote {
    pub source:ActiveSource,pub input:u64,pub fee:u64,pub burn:u64,pub output:u64,
    pub minimum_output:u64,pub total_after:u64,pub supply_after:u64,pub mint_after:u64,
}
fn add(a:u64,b:u64)->Piv1Result<u64>{a.checked_add(b).ok_or(Piv1Error::ArithmeticOverflow)}
fn sub(a:u64,b:u64)->Piv1Result<u64>{a.checked_sub(b).ok_or(Piv1Error::ArithmeticOverflow)}
fn floor(a:u64,b:u64,d:u64)->Piv1Result<u64>{piv1_math::checked_mul_div_floor(a,b,d).map_err(Into::into)}
#[allow(clippy::too_many_arguments)]
pub(crate) fn quote(protocol:&AuthenticatedJitoIdentity,list:&AccountInfo<'_>,source:&AccountInfo<'_>,index:u32,
    clock:&Clock,rent:&Rent,minimum:u64,operational:u64,c:&PivConfig,r:&ActiveDistribution)->Piv1Result<LegQuote>{
    let source=active_source(protocol,list,source,index,clock,rent,minimum,operational)?;
    let pool=protocol.pool();
    let ratio=Ratio{total:pool.total_lamports(),supply:pool.pool_token_supply(),fee:pool.stake_withdrawal_fee()};
    if pool.last_update_epoch()!=clock.epoch||protocol.mint().supply>ratio.supply{return Err(Piv1Error::InvalidCustodyObservation);}
    let remaining=r.remaining_withdrawal_target_units()?;let input=remaining.min(source.capacity);
    let effective=r.snapshot_leg_input_floor_units.max(source.minimum);
    if input==0||input<effective||(remaining!=input&&remaining-input<effective){return Err(Piv1Error::TechnicalFloorNotMet);}
    let fee=ratio.fee(input)?;let burn=sub(input,fee)?;let output=ratio.redeem(input)?;
    if c.configured_slippage_bps>1||r.stored_slippage_bps!=c.configured_slippage_bps{return Err(Piv1Error::InvalidSlippage);}
    let minimum_output=minimum.max(floor(output,10000-u64::from(c.configured_slippage_bps),10000)?);
    if output<minimum_output{return Err(Piv1Error::TechnicalFloorNotMet);}
    let result=LegQuote{source,input,fee,burn,output,minimum_output,
        total_after:sub(ratio.total,output)?,supply_after:sub(ratio.supply,burn)?,mint_after:sub(protocol.mint().supply,burn)?};
    validate_outlook(c,r,input,output,result.total_after,result.supply_after,result.mint_after,ratio.fee,minimum)?;
    Ok(result)
}

/// Current continuation feasibility, not future liquidity/fee guarantees. Use
/// the immutable snapshot input floor for the maximum future rounding count:
/// later q_protocol may fall. Separately require today's post-ratio minimum.
#[allow(clippy::too_many_arguments)]
pub(crate) fn validate_outlook(c:&PivConfig,r:&ActiveDistribution,input:u64,output:u64,
    total:u64,supply:u64,mint:u64,fee:crate::integrations::jito_identity::RawFee,minimum:u64)->Piv1Result<()> {
    if total==0||supply==0||mint>supply{return Err(Piv1Error::InvalidCustodyObservation);}
    let ratio=Ratio{total,supply,fee};
    let remaining=sub(r.remaining_withdrawal_target_units()?,input)?;
    let count=add(r.successful_leg_count,1)?;
    let slots=r.maximum_useful_legs.checked_sub(count).ok_or(Piv1Error::UsefulLegBoundExceeded)?;
    let cumulative=add(r.cumulative_delegated_native_lamports,output)?;
    let future=if remaining==0{0}else{
        let current=ratio.minimum(minimum,mint)?;
        if remaining<current.max(r.snapshot_leg_input_floor_units){return Err(Piv1Error::TechnicalFloorNotMet);}
        let legs=slots.min(remaining.checked_div(r.snapshot_leg_input_floor_units).ok_or(Piv1Error::TechnicalFloorNotMet)?);
        if legs==0{return Err(Piv1Error::UsefulLegBoundExceeded);}
        let reserve=if fee.numerator==0{0}else{legs-1};
        let burn=sub(sub(remaining,ratio.fee(remaining)?)?,reserve)?;
        sub(ratio.book(burn)?,legs-1)?
    };
    if add(cumulative,future)?<r.stored_round_minimum_native_lamports{return Err(Piv1Error::TechnicalFloorNotMet);}
    // Protect the residual after assignment of the WHOLE fixed target, not
    // transient still-unassigned outgoing units held in the principal vault.
    let units=sub(c.accounted_historical_jitosol_units,r.fixed_jitosol_withdrawal_target_units)?;
    let carry=sub(r.prior_next_cycle_yield_lamports,r.prior_next_cycle_yield_used_lamports()?)?;
    let residual=add(add(c.accounted_historical_sol_lamports,ratio.book(units)?)?,carry)?;
    if residual<r.stored_residual_hwm_floor_lamports||residual<r.proposed_hwm_after_settlement_lamports{
        return Err(Piv1Error::HighWaterMarkDecrease);
    }
    Ok(())
}

/// Exact pinned Borsh variant 24 (protected), with both authority metas bound
/// to the same authenticated PIV PDA. The executable is pinned by the handler.
#[allow(clippy::too_many_arguments)]
pub(crate) fn instruction(pool:Pubkey,list:Pubkey,withdraw:Pubkey,source:Pubkey,destination:Pubkey,
    authority:Pubkey,principal:Pubkey,manager:Pubkey,mint:Pubkey,input:u64,minimum:u64)->Instruction{
    let mut data=vec![24];data.extend(input.to_le_bytes());data.extend(minimum.to_le_bytes());
    Instruction{program_id:crate::integrations::jito_identity::JITO_STAKE_POOL_PROGRAM,data,accounts:vec![
        AccountMeta::new(pool,false),AccountMeta::new(list,false),AccountMeta::new_readonly(withdraw,false),
        AccountMeta::new(source,false),AccountMeta::new(destination,false),AccountMeta::new_readonly(authority,false),
        AccountMeta::new_readonly(authority,true),AccountMeta::new(principal,false),AccountMeta::new(manager,false),
        AccountMeta::new(mint,false),AccountMeta::new_readonly(anchor_lang::solana_program::sysvar::clock::ID,false),
        AccountMeta::new_readonly(spl_token::ID,false),AccountMeta::new_readonly(crate::accounts::STAKE_PROGRAM_ID,false)]}
}
