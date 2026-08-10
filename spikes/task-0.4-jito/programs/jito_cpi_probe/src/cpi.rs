use anchor_lang::prelude::*;
use anchor_lang::solana_program::{
    program::{invoke, invoke_signed},
    system_instruction,
};

use crate::{constants::STAKE_ACCOUNT_SPACE, errors::ProbeError};

pub fn create_stake_pda<'info>(
    payer: AccountInfo<'info>,
    stake: AccountInfo<'info>,
    system_program: AccountInfo<'info>,
    payer_seeds: &[&[u8]],
    stake_seeds: &[&[u8]],
    extra_lamports: u64,
) -> Result<u64> {
    require_eq!(stake.lamports(), 0, ProbeError::StakePdaReuse);
    require!(stake.data_is_empty(), ProbeError::StakePdaReuse);
    let rent = Rent::get()?.minimum_balance(STAKE_ACCOUNT_SPACE);
    let funding_lamports = rent
        .checked_add(extra_lamports)
        .ok_or_else(|| error!(ProbeError::Arithmetic))?;
    let ix = system_instruction::create_account(
        payer.key,
        stake.key,
        funding_lamports,
        STAKE_ACCOUNT_SPACE as u64,
        &solana_stake_interface::program::id(),
    );
    invoke_signed(
        &ix,
        &[payer, stake, system_program],
        &[payer_seeds, stake_seeds],
    )?;
    Ok(funding_lamports)
}

#[allow(clippy::too_many_arguments)]
pub fn deposit_sol<'info>(
    stake_pool_program: AccountInfo<'info>,
    stake_pool: AccountInfo<'info>,
    withdraw_authority: AccountInfo<'info>,
    reserve_stake: AccountInfo<'info>,
    sol_vault: AccountInfo<'info>,
    token_vault: AccountInfo<'info>,
    manager_fee: AccountInfo<'info>,
    pool_mint: AccountInfo<'info>,
    system_program: AccountInfo<'info>,
    token_program: AccountInfo<'info>,
    lamports_in: u64,
    minimum_pool_tokens_out: u64,
    sol_vault_seeds: &[&[u8]],
) -> Result<()> {
    let ix = spl_stake_pool::instruction::deposit_sol_with_slippage(
        &spl_stake_pool::id(),
        stake_pool.key,
        withdraw_authority.key,
        reserve_stake.key,
        sol_vault.key,
        token_vault.key,
        manager_fee.key,
        manager_fee.key,
        pool_mint.key,
        token_program.key,
        lamports_in,
        minimum_pool_tokens_out,
    );
    invoke_signed(
        &ix,
        &[
            stake_pool,
            withdraw_authority,
            reserve_stake,
            sol_vault,
            token_vault.clone(),
            manager_fee.clone(),
            manager_fee,
            pool_mint,
            system_program,
            token_program,
            stake_pool_program,
        ],
        &[sol_vault_seeds],
    )?;
    Ok(())
}

#[allow(clippy::too_many_arguments)]
pub fn withdraw_stake<'info>(
    stake_pool_program: AccountInfo<'info>,
    stake_pool: AccountInfo<'info>,
    validator_list: AccountInfo<'info>,
    withdraw_authority: AccountInfo<'info>,
    validator_stake: AccountInfo<'info>,
    withdrawal_stake: AccountInfo<'info>,
    authority: AccountInfo<'info>,
    token_vault: AccountInfo<'info>,
    manager_fee: AccountInfo<'info>,
    pool_mint: AccountInfo<'info>,
    clock: AccountInfo<'info>,
    token_program: AccountInfo<'info>,
    stake_program: AccountInfo<'info>,
    pool_tokens_in: u64,
    minimum_lamports_out: u64,
    authority_seeds: &[&[u8]],
) -> Result<()> {
    let ix = spl_stake_pool::instruction::withdraw_stake_with_slippage(
        &spl_stake_pool::id(),
        stake_pool.key,
        validator_list.key,
        withdraw_authority.key,
        validator_stake.key,
        withdrawal_stake.key,
        authority.key,
        authority.key,
        token_vault.key,
        manager_fee.key,
        pool_mint.key,
        token_program.key,
        pool_tokens_in,
        minimum_lamports_out,
    );
    invoke_signed(
        &ix,
        &[
            stake_pool,
            validator_list,
            withdraw_authority,
            validator_stake,
            withdrawal_stake,
            authority.clone(),
            authority,
            token_vault,
            manager_fee,
            pool_mint,
            clock,
            token_program,
            stake_program,
            stake_pool_program,
        ],
        &[authority_seeds],
    )?;
    Ok(())
}

pub fn transfer_jitosol<'info>(
    source: AccountInfo<'info>,
    mint: AccountInfo<'info>,
    destination: AccountInfo<'info>,
    owner: AccountInfo<'info>,
    token_program: AccountInfo<'info>,
    amount: u64,
    decimals: u8,
) -> Result<()> {
    let ix = spl_token::instruction::transfer_checked(
        token_program.key,
        source.key,
        mint.key,
        destination.key,
        owner.key,
        &[],
        amount,
        decimals,
    )?;
    invoke(&ix, &[source, mint, destination, owner, token_program])?;
    Ok(())
}

pub fn deactivate_stake<'info>(
    stake: AccountInfo<'info>,
    clock: AccountInfo<'info>,
    authority: AccountInfo<'info>,
    stake_program: AccountInfo<'info>,
    authority_seeds: &[&[u8]],
) -> Result<()> {
    let ix = solana_stake_interface::instruction::deactivate_stake(stake.key, authority.key);
    invoke_signed(
        &ix,
        &[stake, clock, authority, stake_program],
        &[authority_seeds],
    )?;
    Ok(())
}

pub fn initialize_inactive_stake<'info>(
    stake: AccountInfo<'info>,
    rent: AccountInfo<'info>,
    stake_program: AccountInfo<'info>,
    authority: &Pubkey,
) -> Result<()> {
    let authorized = solana_stake_interface::state::Authorized {
        staker: *authority,
        withdrawer: *authority,
    };
    let ix = solana_stake_interface::instruction::initialize(
        stake.key,
        &authorized,
        &solana_stake_interface::state::Lockup::default(),
    );
    invoke(&ix, &[stake, rent, stake_program])?;
    Ok(())
}

#[allow(clippy::too_many_arguments)]
pub fn withdraw_native_sol<'info>(
    stake: AccountInfo<'info>,
    escrow: AccountInfo<'info>,
    clock: AccountInfo<'info>,
    stake_history: AccountInfo<'info>,
    authority: AccountInfo<'info>,
    stake_program: AccountInfo<'info>,
    lamports: u64,
    authority_seeds: &[&[u8]],
) -> Result<()> {
    let ix = solana_stake_interface::instruction::withdraw(
        stake.key,
        authority.key,
        escrow.key,
        lamports,
        None,
    );
    invoke_signed(
        &ix,
        &[stake, escrow, clock, stake_history, authority, stake_program],
        &[authority_seeds],
    )?;
    Ok(())
}
