//! Bounded SPL Stake Pool 2.0.3 protected SOL deposit, not the complete adapter.
//! Source: instruction.rs DepositSolWithSlippage/deposit_sol_internal (variant
//! 25); state.rs164..176/222..236; processor.rs2561..2729. Only permissionless,
//! zero-fee deposits are supported. No mock revision/capacity is manufactured.
use anchor_lang::{prelude::Pubkey, solana_program::{instruction::{AccountMeta, Instruction}, system_program}};
use crate::{errors::{Piv1Error, Piv1Result}, integrations::jito_identity::AuthenticatedJitoIdentity,
    state::{PivConfig, principal_deposit::{DepositProfile, protected_minimum},
        reconciliation::EconomicCustodyObservation}};

pub(crate) struct NativeDeposit<'a> {
    pub protocol: &'a AuthenticatedJitoIdentity,
    pub epoch: u64,
    pub amount: u64,
    pub minimum: u64,
    pub slippage: u16,
}
impl NativeDeposit<'_> {
    pub fn minted(&self) -> Piv1Result<u64> {
        let p = self.protocol.pool();
        if p.total_lamports() == 0 || p.pool_token_supply() == 0 { return Ok(self.amount); }
        piv1_math::checked_mul_div_floor(self.amount, p.pool_token_supply(), p.total_lamports()).map_err(Into::into)
    }
    pub fn after(&self) -> Piv1Result<(u64, u64, u64)> {
        let minted = self.minted()?; let p = self.protocol.pool();
        Ok((add(p.total_lamports(), self.amount)?, add(p.pool_token_supply(), minted)?,
            add(self.protocol.mint().supply, minted)?))
    }
    pub fn protected_output(&self, minted: u64) -> Piv1Result<u64> {
        Ok(protected_minimum(minted, self.slippage, self.minimum)?.1)
    }
}
impl DepositProfile for NativeDeposit<'_> {
    fn amount(&self) -> u64 { self.amount }
    fn slippage(&self) -> u16 { self.slippage }
    fn validate_pools(&self) -> Piv1Result<()> {
        let p = self.protocol.pool();
        if p.last_update_epoch() != self.epoch || (p.total_lamports() == 0) != (p.pool_token_supply() == 0)
            || self.protocol.mint().supply > p.pool_token_supply() || p.sol_deposit_authority().is_some() {
            return Err(Piv1Error::InvalidPrincipalDepositPool);
        }
        Ok(())
    }
    fn validate_zero_fee(&self) -> Piv1Result<()> {
        // Identity parsing already validated numerator <= denominator. Pinned
        // external 0/0 and 0/N both apply zero fee; legacy internal 0/1 is unchanged.
        if self.protocol.pool().sol_deposit_fee().numerator != 0 {
            return Err(Piv1Error::UnsupportedPrincipalDepositFee);
        }
        Ok(())
    }
    fn validate_supply_before(&self, custody: EconomicCustodyObservation) -> Piv1Result<()> {
        combined_supply(custody, self.protocol.mint().supply)
    }
    fn minted_and_validate_execution(&self, _: &PivConfig) -> Piv1Result<u64> {
        let minted = self.minted()?; self.protected_output(minted)?; self.after()?; Ok(minted)
    }
    fn validate_supply_after(&self, custody: EconomicCustodyObservation) -> Piv1Result<()> {
        combined_supply(custody, self.after()?.2)
    }
    fn book_value_before(&self, units: u64) -> Piv1Result<u64> {
        value(units, self.protocol.pool().total_lamports(), self.protocol.pool().pool_token_supply())
    }
    fn book_value_after(&self, units: u64) -> Piv1Result<u64> {
        let (total, supply, _) = self.after()?; value(units, total, supply)
    }
}
fn value(units: u64, total: u64, supply: u64) -> Piv1Result<u64> {
    if units > supply { return Err(Piv1Error::InvalidPrincipalDepositPool); }
    if supply == 0 { return Ok(0); }
    piv1_math::checked_mul_div_floor(units, total, supply).map_err(Into::into)
}
fn combined_supply(c: EconomicCustodyObservation, minted_supply: u64) -> Piv1Result<()> {
    if add(c.principal_jitosol_units, c.pending_jitosol_units)? > minted_supply {
        return Err(Piv1Error::InvalidPrincipalDepositPool);
    }
    Ok(())
}
fn add(a: u64, b: u64) -> Piv1Result<u64> { a.checked_add(b).ok_or(Piv1Error::ArithmeticOverflow) }

/// Exact pinned Borsh variant and account order; never unprotected DepositSol.
#[allow(clippy::too_many_arguments)]
pub(crate) fn instruction(program: Pubkey, pool: Pubkey, withdraw: Pubkey, reserve: Pubkey,
    source: Pubkey, destination: Pubkey, manager: Pubkey, referrer: Pubkey, mint: Pubkey,
    amount: u64, minimum: u64) -> Instruction
{
    let mut data = vec![25]; data.extend_from_slice(&amount.to_le_bytes()); data.extend_from_slice(&minimum.to_le_bytes());
    Instruction { program_id: program, data, accounts: vec![
        AccountMeta::new(pool, false), AccountMeta::new_readonly(withdraw, false),
        AccountMeta::new(reserve, false), AccountMeta::new(source, true),
        AccountMeta::new(destination, false), AccountMeta::new(manager, false),
        AccountMeta::new(referrer, false), AccountMeta::new(mint, false),
        AccountMeta::new_readonly(system_program::ID, false), AccountMeta::new_readonly(spl_token::ID, false),
    ] }
}
