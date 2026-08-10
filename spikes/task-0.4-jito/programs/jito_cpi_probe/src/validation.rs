use std::num::NonZeroU32;

use anchor_lang::{prelude::*, solana_program::program_pack::Pack};
use borsh1::BorshDeserialize;
use solana_stake_interface::state::StakeStateV2;
use spl_stake_pool::state::StakePool;
use spl_token::state::{Account as TokenAccount, Mint};

use crate::{
    constants::{JITOSOL_MINT, JITO_STAKE_POOL},
    errors::ProbeError,
};

pub struct PoolBindings {
    pub stake_pool: Pubkey,
    pub validator_list: Pubkey,
    pub reserve_stake: Pubkey,
    pub pool_mint: Pubkey,
    pub manager_fee_account: Pubkey,
    pub token_program: Pubkey,
    pub withdraw_authority: Pubkey,
}

pub fn decode_stake_pool(account: &AccountInfo<'_>) -> Result<StakePool> {
    require_keys_eq!(*account.owner, spl_stake_pool::id(), ProbeError::WrongAccountOwner);
    let data = account.try_borrow_data()?;
    let mut bytes: &[u8] = &data;
    StakePool::deserialize(&mut bytes).map_err(|_| error!(ProbeError::InvalidStakePoolState))
}

pub fn validate_pool_bindings(
    stake_pool_program: &AccountInfo<'_>,
    stake_pool_account: &AccountInfo<'_>,
    bindings: &PoolBindings,
) -> Result<StakePool> {
    require_keys_eq!(stake_pool_program.key(), spl_stake_pool::id(), ProbeError::WrongStakePoolProgram);
    require!(stake_pool_program.executable, ProbeError::WrongStakePoolProgram);
    require_keys_eq!(stake_pool_account.key(), JITO_STAKE_POOL, ProbeError::WrongStakePool);
    require_keys_eq!(bindings.stake_pool, JITO_STAKE_POOL, ProbeError::WrongStakePool);

    let pool = decode_stake_pool(stake_pool_account)?;
    require_keys_eq!(pool.validator_list, bindings.validator_list, ProbeError::StakePoolBindingMismatch);
    require_keys_eq!(pool.reserve_stake, bindings.reserve_stake, ProbeError::StakePoolBindingMismatch);
    require_keys_eq!(pool.pool_mint, JITOSOL_MINT, ProbeError::WrongPoolMint);
    require_keys_eq!(pool.pool_mint, bindings.pool_mint, ProbeError::WrongPoolMint);
    require_keys_eq!(pool.manager_fee_account, bindings.manager_fee_account, ProbeError::StakePoolBindingMismatch);
    require_keys_eq!(pool.token_program_id, spl_token::id(), ProbeError::WrongTokenProgram);
    require_keys_eq!(pool.token_program_id, bindings.token_program, ProbeError::WrongTokenProgram);

    let (expected_withdraw_authority, expected_bump) =
        spl_stake_pool::find_withdraw_authority_program_address(&spl_stake_pool::id(), &JITO_STAKE_POOL);
    require_keys_eq!(bindings.withdraw_authority, expected_withdraw_authority, ProbeError::StakePoolBindingMismatch);
    require_eq!(pool.stake_withdraw_bump_seed, expected_bump, ProbeError::StakePoolBindingMismatch);
    Ok(pool)
}

pub fn validate_token_account(
    account: &AccountInfo<'_>,
    expected_mint: &Pubkey,
    expected_authority: &Pubkey,
) -> Result<TokenAccount> {
    require_keys_eq!(*account.owner, spl_token::id(), ProbeError::WrongAccountOwner);
    let data = account.try_borrow_data()?;
    let token = TokenAccount::unpack(&data).map_err(|_| error!(ProbeError::InvalidTokenAccount))?;
    require_keys_eq!(token.mint, *expected_mint, ProbeError::TokenMintMismatch);
    require_keys_eq!(token.owner, *expected_authority, ProbeError::TokenAuthorityMismatch);
    Ok(token)
}

pub fn validate_mint(account: &AccountInfo<'_>, expected: &Pubkey) -> Result<Mint> {
    require_keys_eq!(account.key(), *expected, ProbeError::WrongPoolMint);
    require_keys_eq!(*account.owner, spl_token::id(), ProbeError::WrongAccountOwner);
    let data = account.try_borrow_data()?;
    Mint::unpack(&data).map_err(|_| error!(ProbeError::InvalidMint))
}

pub fn expected_deposit_tokens(pool: &StakePool, lamports: u64) -> Result<u64> {
    require!(lamports > 0, ProbeError::AmountZero);
    let gross = pool
        .calc_pool_tokens_for_deposit(lamports)
        .ok_or_else(|| error!(ProbeError::Arithmetic))?;
    let fee = pool
        .calc_pool_tokens_sol_deposit_fee(gross)
        .ok_or_else(|| error!(ProbeError::Arithmetic))?;
    gross.checked_sub(fee).ok_or_else(|| error!(ProbeError::Arithmetic))
}

pub fn expected_withdraw_lamports(pool: &StakePool, pool_tokens_in: u64) -> Result<u64> {
    require!(pool_tokens_in > 0, ProbeError::AmountZero);
    let fee = pool
        .calc_pool_tokens_stake_withdrawal_fee(pool_tokens_in)
        .ok_or_else(|| error!(ProbeError::Arithmetic))?;
    let burnt = pool_tokens_in
        .checked_sub(fee)
        .ok_or_else(|| error!(ProbeError::Arithmetic))?;
    pool.calc_lamports_withdraw_amount(burnt)
        .ok_or_else(|| error!(ProbeError::Arithmetic))
}

pub fn minimum_pool_tokens_for_withdrawal(pool: &StakePool, required_lamports: u64) -> Result<u64> {
    require!(required_lamports > 0, ProbeError::AmountZero);
    let mut low = 1u64;
    let mut high = pool.pool_token_supply;
    while low < high {
        let mid = low + (high - low) / 2;
        if expected_withdraw_lamports(pool, mid)? >= required_lamports {
            high = mid;
        } else {
            low = mid.checked_add(1).ok_or_else(|| error!(ProbeError::Arithmetic))?;
        }
    }
    require!(expected_withdraw_lamports(pool, low)? >= required_lamports, ProbeError::WithdrawalBelowTechnicalMinimum);
    Ok(low)
}

pub fn decode_stake_state(account: &AccountInfo<'_>) -> Result<StakeStateV2> {
    require_keys_eq!(*account.owner, solana_stake_interface::program::id(), ProbeError::WrongAccountOwner);
    let data = account.try_borrow_data()?;
    bincode::deserialize(&data).map_err(|_| error!(ProbeError::InvalidStakeState))
}

pub fn validate_validator_stake_source(account: &AccountInfo<'_>, pool: &Pubkey) -> Result<Pubkey> {
    let state = decode_stake_state(account)?;
    let stake = state.stake().ok_or_else(|| error!(ProbeError::InvalidStakeState))?;
    let voter = stake.delegation.voter_pubkey;
    let (standard, _) = spl_stake_pool::find_stake_program_address(
        &spl_stake_pool::id(),
        &voter,
        pool,
        None::<NonZeroU32>,
    );
    require_keys_eq!(account.key(), standard, ProbeError::WrongValidatorStake);
    Ok(voter)
}

pub fn validate_round_id(next_round: u64, requested: u64) -> Result<()> {
    require_eq!(next_round, requested, ProbeError::RoundReuse);
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use spl_stake_pool::state::Fee;

    fn pool() -> StakePool {
        StakePool {
            total_lamports: 1_250_000_000_000,
            pool_token_supply: 1_000_000_000_000,
            sol_deposit_fee: Fee { numerator: 3, denominator: 1000 },
            stake_withdrawal_fee: Fee { numerator: 3, denominator: 1000 },
            ..StakePool::default()
        }
    }

    #[test]
    fn pda_derivations_are_stable_and_distinct() {
        let program = crate::ID;
        let (authority, _) = Pubkey::find_program_address(&[crate::constants::AUTHORITY_SEED], &program);
        let (vault, _) = Pubkey::find_program_address(&[crate::constants::SOL_VAULT_SEED], &program);
        let (escrow, _) = Pubkey::find_program_address(&[crate::constants::SOL_ESCROW_SEED], &program);
        let (stake0, _) = Pubkey::find_program_address(
            &[crate::constants::WITHDRAWAL_STAKE_SEED, &0u64.to_le_bytes()],
            &program,
        );
        let (stake1, _) = Pubkey::find_program_address(
            &[crate::constants::WITHDRAWAL_STAKE_SEED, &1u64.to_le_bytes()],
            &program,
        );
        assert_ne!(authority, vault);
        assert_ne!(vault, escrow);
        assert_ne!(stake0, stake1);
    }

    #[test]
    fn stake_pool_instructions_require_pda_signers() {
        let signer = Pubkey::new_unique();
        let ix = spl_stake_pool::instruction::deposit_sol_with_slippage(
            &spl_stake_pool::id(), &JITO_STAKE_POOL, &Pubkey::new_unique(), &Pubkey::new_unique(),
            &signer, &Pubkey::new_unique(), &Pubkey::new_unique(), &Pubkey::new_unique(),
            &JITOSOL_MINT, &spl_token::id(), 1, 1,
        );
        assert!(ix.accounts.iter().any(|meta| meta.pubkey == signer && meta.is_signer));

        let ix = spl_stake_pool::instruction::withdraw_stake_with_slippage(
            &spl_stake_pool::id(), &JITO_STAKE_POOL, &Pubkey::new_unique(), &Pubkey::new_unique(),
            &Pubkey::new_unique(), &Pubkey::new_unique(), &signer, &signer,
            &Pubkey::new_unique(), &Pubkey::new_unique(), &JITOSOL_MINT, &spl_token::id(), 1, 1,
        );
        assert!(ix.accounts.iter().any(|meta| meta.pubkey == signer && meta.is_signer));
    }

    #[test]
    fn slippage_variants_keep_accounts_and_encode_an_output_floor() {
        let keys: Vec<Pubkey> = (0..11).map(|_| Pubkey::new_unique()).collect();
        let basic_deposit = spl_stake_pool::instruction::deposit_sol(
            &spl_stake_pool::id(), &JITO_STAKE_POOL, &keys[0], &keys[1], &keys[2],
            &keys[3], &keys[4], &keys[4], &JITOSOL_MINT, &spl_token::id(), 10,
        );
        let protected_deposit = spl_stake_pool::instruction::deposit_sol_with_slippage(
            &spl_stake_pool::id(), &JITO_STAKE_POOL, &keys[0], &keys[1], &keys[2],
            &keys[3], &keys[4], &keys[4], &JITOSOL_MINT, &spl_token::id(), 10, 9,
        );
        assert_eq!(basic_deposit.accounts, protected_deposit.accounts);
        assert!(protected_deposit.data.len() > basic_deposit.data.len());

        let basic_withdraw = spl_stake_pool::instruction::withdraw_stake(
            &spl_stake_pool::id(), &JITO_STAKE_POOL, &keys[0], &keys[1], &keys[2],
            &keys[3], &keys[4], &keys[4], &keys[5], &keys[6], &JITOSOL_MINT,
            &spl_token::id(), 10,
        );
        let protected_withdraw = spl_stake_pool::instruction::withdraw_stake_with_slippage(
            &spl_stake_pool::id(), &JITO_STAKE_POOL, &keys[0], &keys[1], &keys[2],
            &keys[3], &keys[4], &keys[4], &keys[5], &keys[6], &JITOSOL_MINT,
            &spl_token::id(), 10, 9,
        );
        assert_eq!(basic_withdraw.accounts, protected_withdraw.accounts);
        assert!(protected_withdraw.data.len() > basic_withdraw.data.len());
    }

    #[test]
    fn checked_deposit_math_floors_exchange_and_ceils_fee() {
        assert_eq!(expected_deposit_tokens(&pool(), 1_000_000_000).unwrap(), 797_600_000);
    }

    #[test]
    fn checked_withdraw_math_ceils_fee_and_floors_exchange() {
        assert_eq!(expected_withdraw_lamports(&pool(), 1_000_000_000).unwrap(), 1_246_250_000);
    }

    #[test]
    fn zero_amounts_are_rejected() {
        assert!(expected_deposit_tokens(&pool(), 0).is_err());
        assert!(expected_withdraw_lamports(&pool(), 0).is_err());
    }

    #[test]
    fn technical_minimum_is_exact_at_boundary() {
        let required = 1_002_282_880;
        let minimum = minimum_pool_tokens_for_withdrawal(&pool(), required).unwrap();
        assert!(expected_withdraw_lamports(&pool(), minimum).unwrap() >= required);
        assert!(expected_withdraw_lamports(&pool(), minimum - 1).unwrap() < required);
    }

    #[test]
    fn round_reuse_is_rejected() {
        assert!(validate_round_id(4, 3).is_err());
        assert!(validate_round_id(4, 4).is_ok());
        assert!(validate_round_id(4, 5).is_err());
    }
}
