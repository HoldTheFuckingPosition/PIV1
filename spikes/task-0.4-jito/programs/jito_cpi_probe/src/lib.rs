#![allow(deprecated, unexpected_cfgs)]

use anchor_lang::prelude::*;
use solana_stake_interface::state::StakeStateV2;

pub mod constants;
pub mod cpi;
pub mod errors;
pub mod state;
pub mod validation;

use constants::*;
use errors::ProbeError;
use state::*;
use validation::{PoolBindings, validate_mint, validate_pool_bindings, validate_round_id, validate_token_account};

declare_id!("BbHNk57mVmZmfH1HfiyPaapwJ5FVPe3c7MsidbGVewG6");

#[program]
pub mod jito_cpi_probe {
    use super::*;

    pub fn initialize(ctx: Context<Initialize>) -> Result<()> {
        let bindings = PoolBindings {
            stake_pool: ctx.accounts.stake_pool.key(),
            validator_list: ctx.accounts.validator_list.key(),
            reserve_stake: ctx.accounts.reserve_stake.key(),
            pool_mint: ctx.accounts.pool_mint.key(),
            manager_fee_account: ctx.accounts.manager_fee_account.key(),
            token_program: ctx.accounts.token_program.key(),
            withdraw_authority: ctx.accounts.stake_pool_withdraw_authority.key(),
        };
        validate_pool_bindings(
            &ctx.accounts.stake_pool_program.to_account_info(),
            &ctx.accounts.stake_pool.to_account_info(),
            &bindings,
        )?;
        require_keys_eq!(ctx.accounts.token_program.key(), spl_token::id(), ProbeError::WrongTokenProgram);
        require!(ctx.accounts.token_program.executable, ProbeError::WrongTokenProgram);
        validate_mint(&ctx.accounts.pool_mint.to_account_info(), &JITOSOL_MINT)?;
        validate_token_account(
            &ctx.accounts.token_vault.to_account_info(),
            &JITOSOL_MINT,
            &ctx.accounts.authority.key(),
        )?;

        let config = &mut ctx.accounts.config;
        config.version = 1;
        config.stake_pool_program = ctx.accounts.stake_pool_program.key();
        config.stake_pool = ctx.accounts.stake_pool.key();
        config.validator_list = ctx.accounts.validator_list.key();
        config.reserve_stake = ctx.accounts.reserve_stake.key();
        config.pool_mint = ctx.accounts.pool_mint.key();
        config.manager_fee_account = ctx.accounts.manager_fee_account.key();
        config.token_vault = ctx.accounts.token_vault.key();
        config.config_bump = ctx.bumps.config;
        config.authority_bump = ctx.bumps.authority;
        config.sol_vault_bump = ctx.bumps.sol_vault;
        config.sol_escrow_bump = ctx.bumps.sol_escrow;
        config.next_round = 0;

        emit!(ProbeInitialized {
            config: config.key(),
            authority: ctx.accounts.authority.key(),
            sol_vault: ctx.accounts.sol_vault.key(),
            token_vault: ctx.accounts.token_vault.key(),
            sol_escrow: ctx.accounts.sol_escrow.key(),
        });
        Ok(())
    }

    pub fn deposit_sol(
        ctx: Context<DepositSol>,
        lamports_in: u64,
        minimum_pool_tokens_out: u64,
    ) -> Result<()> {
        require!(lamports_in > 0, ProbeError::AmountZero);
        require_keys_eq!(*ctx.accounts.sol_vault.owner, System::id(), ProbeError::WrongAccountOwner);
        require!(ctx.accounts.sol_vault.data_is_empty(), ProbeError::WrongAccountOwner);
        let pool = checked_pool(ctx.accounts)?;
        require_eq!(pool.last_update_epoch, Clock::get()?.epoch, ProbeError::StalePool);
        let expected = validation::expected_deposit_tokens(&pool, lamports_in)?;
        require!(expected >= minimum_pool_tokens_out, ProbeError::SlippageExceeded);

        let before_sol = ctx.accounts.sol_vault.lamports();
        let before_tokens = validate_token_account(
            &ctx.accounts.token_vault.to_account_info(),
            &JITOSOL_MINT,
            &ctx.accounts.authority.key(),
        )?.amount;
        let vault_bump = [ctx.accounts.config.sol_vault_bump];
        let vault_seeds: &[&[u8]] = &[SOL_VAULT_SEED, &vault_bump];
        cpi::deposit_sol(
            ctx.accounts.stake_pool_program.to_account_info(),
            ctx.accounts.stake_pool.to_account_info(),
            ctx.accounts.stake_pool_withdraw_authority.to_account_info(),
            ctx.accounts.reserve_stake.to_account_info(),
            ctx.accounts.sol_vault.to_account_info(),
            ctx.accounts.token_vault.to_account_info(),
            ctx.accounts.manager_fee_account.to_account_info(),
            ctx.accounts.pool_mint.to_account_info(),
            ctx.accounts.system_program.to_account_info(),
            ctx.accounts.token_program.to_account_info(),
            lamports_in,
            minimum_pool_tokens_out,
            vault_seeds,
        )?;

        let after_sol = ctx.accounts.sol_vault.lamports();
        let after_tokens = validate_token_account(
            &ctx.accounts.token_vault.to_account_info(),
            &JITOSOL_MINT,
            &ctx.accounts.authority.key(),
        )?.amount;
        let sol_delta = before_sol.checked_sub(after_sol).ok_or_else(|| error!(ProbeError::BalanceDelta))?;
        let token_delta = after_tokens.checked_sub(before_tokens).ok_or_else(|| error!(ProbeError::BalanceDelta))?;
        require_eq!(sol_delta, lamports_in, ProbeError::BalanceDelta);
        require_eq!(token_delta, expected, ProbeError::BalanceDelta);
        emit!(SolDeposited {
            caller: ctx.accounts.caller.key(),
            lamports_in,
            pool_tokens_out: token_delta,
        });
        Ok(())
    }

    pub fn contribute_jitosol(ctx: Context<ContributeJitoSol>, amount: u64) -> Result<()> {
        require!(amount > 0, ProbeError::AmountZero);
        let mint = validate_mint(&ctx.accounts.pool_mint.to_account_info(), &JITOSOL_MINT)?;
        let source_before = validate_token_account(
            &ctx.accounts.source_token_account.to_account_info(),
            &JITOSOL_MINT,
            &ctx.accounts.caller.key(),
        )?.amount;
        let vault_before = validate_token_account(
            &ctx.accounts.token_vault.to_account_info(),
            &JITOSOL_MINT,
            &ctx.accounts.authority.key(),
        )?.amount;
        cpi::transfer_jitosol(
            ctx.accounts.source_token_account.to_account_info(),
            ctx.accounts.pool_mint.to_account_info(),
            ctx.accounts.token_vault.to_account_info(),
            ctx.accounts.caller.to_account_info(),
            ctx.accounts.token_program.to_account_info(),
            amount,
            mint.decimals,
        )?;
        let source_after = validate_token_account(
            &ctx.accounts.source_token_account.to_account_info(),
            &JITOSOL_MINT,
            &ctx.accounts.caller.key(),
        )?.amount;
        let vault_after = validate_token_account(
            &ctx.accounts.token_vault.to_account_info(),
            &JITOSOL_MINT,
            &ctx.accounts.authority.key(),
        )?.amount;
        require_eq!(source_before.checked_sub(source_after).ok_or_else(|| error!(ProbeError::BalanceDelta))?, amount, ProbeError::BalanceDelta);
        require_eq!(vault_after.checked_sub(vault_before).ok_or_else(|| error!(ProbeError::BalanceDelta))?, amount, ProbeError::BalanceDelta);
        emit!(JitoSolContributed { caller: ctx.accounts.caller.key(), amount });
        Ok(())
    }

    pub fn initiate_withdrawal(
        ctx: Context<InitiateWithdrawal>,
        round_id: u64,
        pool_tokens_in: u64,
        minimum_lamports_out: u64,
        deactivate_in_same_transaction: bool,
    ) -> Result<()> {
        require!(pool_tokens_in > 0, ProbeError::AmountZero);
        validate_round_id(ctx.accounts.config.next_round, round_id)?;
        require_keys_eq!(*ctx.accounts.sol_vault.owner, System::id(), ProbeError::WrongAccountOwner);
        require!(ctx.accounts.sol_vault.data_is_empty(), ProbeError::WrongAccountOwner);
        require_keys_eq!(ctx.accounts.stake_program.key(), solana_stake_interface::program::id(), ProbeError::WrongStakeProgram);
        require!(ctx.accounts.stake_program.executable, ProbeError::WrongStakeProgram);

        let pool = checked_pool(ctx.accounts)?;
        require_eq!(pool.last_update_epoch, ctx.accounts.clock.epoch, ProbeError::StalePool);
        validation::validate_validator_stake_source(
            &ctx.accounts.validator_stake.to_account_info(),
            &ctx.accounts.stake_pool.key(),
        )?;
        validate_token_account(
            &ctx.accounts.token_vault.to_account_info(),
            &JITOSOL_MINT,
            &ctx.accounts.authority.key(),
        )?;
        let expected_lamports = validation::expected_withdraw_lamports(&pool, pool_tokens_in)?;
        require!(expected_lamports >= minimum_lamports_out, ProbeError::SlippageExceeded);
        let minimum_delegation = solana_stake_interface::tools::get_minimum_delegation()?;
        require!(expected_lamports >= minimum_delegation, ProbeError::WithdrawalBelowTechnicalMinimum);

        let round_bytes = round_id.to_le_bytes();
        let stake_bump = [ctx.bumps.withdrawal_stake];
        let stake_seeds: &[&[u8]] = &[WITHDRAWAL_STAKE_SEED, &round_bytes, &stake_bump];
        let sol_vault_bump = [ctx.accounts.config.sol_vault_bump];
        let sol_vault_seeds: &[&[u8]] = &[SOL_VAULT_SEED, &sol_vault_bump];
        let rent_funded = cpi::create_stake_pda(
            ctx.accounts.sol_vault.to_account_info(),
            ctx.accounts.withdrawal_stake.to_account_info(),
            ctx.accounts.system_program.to_account_info(),
            sol_vault_seeds,
            stake_seeds,
            0,
        )?;

        let tokens_before = validate_token_account(
            &ctx.accounts.token_vault.to_account_info(),
            &JITOSOL_MINT,
            &ctx.accounts.authority.key(),
        )?.amount;
        let authority_bump = [ctx.accounts.config.authority_bump];
        let authority_seeds: &[&[u8]] = &[AUTHORITY_SEED, &authority_bump];
        cpi::withdraw_stake(
            ctx.accounts.stake_pool_program.to_account_info(),
            ctx.accounts.stake_pool.to_account_info(),
            ctx.accounts.validator_list.to_account_info(),
            ctx.accounts.stake_pool_withdraw_authority.to_account_info(),
            ctx.accounts.validator_stake.to_account_info(),
            ctx.accounts.withdrawal_stake.to_account_info(),
            ctx.accounts.authority.to_account_info(),
            ctx.accounts.token_vault.to_account_info(),
            ctx.accounts.manager_fee_account.to_account_info(),
            ctx.accounts.pool_mint.to_account_info(),
            ctx.accounts.clock.to_account_info(),
            ctx.accounts.token_program.to_account_info(),
            ctx.accounts.stake_program.to_account_info(),
            pool_tokens_in,
            minimum_lamports_out,
            authority_seeds,
        )?;
        let tokens_after = validate_token_account(
            &ctx.accounts.token_vault.to_account_info(),
            &JITOSOL_MINT,
            &ctx.accounts.authority.key(),
        )?.amount;
        require_eq!(tokens_before.checked_sub(tokens_after).ok_or_else(|| error!(ProbeError::BalanceDelta))?, pool_tokens_in, ProbeError::BalanceDelta);
        require_eq!(ctx.accounts.withdrawal_stake.lamports(), rent_funded.checked_add(expected_lamports).ok_or_else(|| error!(ProbeError::Arithmetic))?, ProbeError::BalanceDelta);
        validate_withdrawal_authorities(
            &ctx.accounts.withdrawal_stake.to_account_info(),
            &ctx.accounts.authority.key(),
        )?;

        let status = if deactivate_in_same_transaction {
            cpi::deactivate_stake(
                ctx.accounts.withdrawal_stake.to_account_info(),
                ctx.accounts.clock.to_account_info(),
                ctx.accounts.authority.to_account_info(),
                ctx.accounts.stake_program.to_account_info(),
                authority_seeds,
            )?;
            RoundStatus::Deactivating
        } else {
            RoundStatus::Withdrawn
        };

        let round = &mut ctx.accounts.round;
        round.version = 1;
        round.config = ctx.accounts.config.key();
        round.round_id = round_id;
        round.withdrawal_stake = ctx.accounts.withdrawal_stake.key();
        round.pool_tokens_in = pool_tokens_in;
        round.minimum_lamports_out = minimum_lamports_out;
        round.expected_lamports_out = expected_lamports;
        round.status = status;
        round.stake_bump = ctx.bumps.withdrawal_stake;
        round.round_bump = ctx.bumps.round;
        ctx.accounts.config.next_round = round_id.checked_add(1).ok_or_else(|| error!(ProbeError::Arithmetic))?;
        emit!(WithdrawalInitiated {
            caller: ctx.accounts.caller.key(),
            round_id,
            withdrawal_stake: ctx.accounts.withdrawal_stake.key(),
            pool_tokens_in,
            expected_lamports_out: expected_lamports,
            rent_funded,
            deactivated: deactivate_in_same_transaction,
        });
        Ok(())
    }

    pub fn deactivate_withdrawal_stake(
        ctx: Context<ManageWithdrawalStake>,
        _round_id: u64,
    ) -> Result<()> {
        require!(ctx.accounts.round.status == RoundStatus::Withdrawn, ProbeError::WrongRoundStatus);
        validate_withdrawal_authorities(
            &ctx.accounts.withdrawal_stake.to_account_info(),
            &ctx.accounts.authority.key(),
        )?;
        let authority_bump = [ctx.accounts.config.authority_bump];
        let authority_seeds: &[&[u8]] = &[AUTHORITY_SEED, &authority_bump];
        cpi::deactivate_stake(
            ctx.accounts.withdrawal_stake.to_account_info(),
            ctx.accounts.clock.to_account_info(),
            ctx.accounts.authority.to_account_info(),
            ctx.accounts.stake_program.to_account_info(),
            authority_seeds,
        )?;
        ctx.accounts.round.status = RoundStatus::Deactivating;
        emit!(StakeDeactivated {
            caller: ctx.accounts.caller.key(),
            round_id: ctx.accounts.round.round_id,
            withdrawal_stake: ctx.accounts.withdrawal_stake.key(),
        });
        Ok(())
    }

    pub fn create_inactive_stake_proof(
        ctx: Context<CreateInactiveStakeProof>,
        round_id: u64,
        extra_lamports: u64,
    ) -> Result<()> {
        validate_round_id(ctx.accounts.config.next_round, round_id)?;
        let round_bytes = round_id.to_le_bytes();
        let stake_bump = [ctx.bumps.withdrawal_stake];
        let stake_seeds: &[&[u8]] = &[WITHDRAWAL_STAKE_SEED, &round_bytes, &stake_bump];
        let sol_vault_bump = [ctx.accounts.config.sol_vault_bump];
        let sol_vault_seeds: &[&[u8]] = &[SOL_VAULT_SEED, &sol_vault_bump];
        let total_lamports = cpi::create_stake_pda(
            ctx.accounts.sol_vault.to_account_info(),
            ctx.accounts.withdrawal_stake.to_account_info(),
            ctx.accounts.system_program.to_account_info(),
            sol_vault_seeds,
            stake_seeds,
            extra_lamports,
        )?;
        cpi::initialize_inactive_stake(
            ctx.accounts.withdrawal_stake.to_account_info(),
            ctx.accounts.rent.to_account_info(),
            ctx.accounts.stake_program.to_account_info(),
            &ctx.accounts.authority.key(),
        )?;
        validate_withdrawal_authorities(
            &ctx.accounts.withdrawal_stake.to_account_info(),
            &ctx.accounts.authority.key(),
        )?;
        let round = &mut ctx.accounts.round;
        round.version = 1;
        round.config = ctx.accounts.config.key();
        round.round_id = round_id;
        round.withdrawal_stake = ctx.accounts.withdrawal_stake.key();
        round.pool_tokens_in = 0;
        round.minimum_lamports_out = 0;
        round.expected_lamports_out = extra_lamports;
        round.status = RoundStatus::Deactivating;
        round.stake_bump = ctx.bumps.withdrawal_stake;
        round.round_bump = ctx.bumps.round;
        ctx.accounts.config.next_round = round_id.checked_add(1).ok_or_else(|| error!(ProbeError::Arithmetic))?;
        emit!(InactiveStakeProofCreated {
            caller: ctx.accounts.caller.key(),
            round_id,
            withdrawal_stake: ctx.accounts.withdrawal_stake.key(),
            total_lamports,
        });
        Ok(())
    }

    pub fn finalize_withdrawal(
        ctx: Context<FinalizeWithdrawal>,
        _round_id: u64,
    ) -> Result<()> {
        require!(
            matches!(ctx.accounts.round.status, RoundStatus::Deactivating | RoundStatus::Withdrawn),
            ProbeError::WrongRoundStatus
        );
        validate_withdrawal_ready(
            &ctx.accounts.withdrawal_stake.to_account_info(),
            &ctx.accounts.authority.key(),
            ctx.accounts.clock.epoch,
        )?;
        require_keys_eq!(*ctx.accounts.sol_escrow.owner, System::id(), ProbeError::WrongAccountOwner);
        require!(ctx.accounts.sol_escrow.data_is_empty(), ProbeError::WrongAccountOwner);
        let amount = ctx.accounts.withdrawal_stake.lamports();
        require!(amount > 0, ProbeError::AmountZero);
        let escrow_before = ctx.accounts.sol_escrow.lamports();
        let authority_bump = [ctx.accounts.config.authority_bump];
        let authority_seeds: &[&[u8]] = &[AUTHORITY_SEED, &authority_bump];
        cpi::withdraw_native_sol(
            ctx.accounts.withdrawal_stake.to_account_info(),
            ctx.accounts.sol_escrow.to_account_info(),
            ctx.accounts.clock.to_account_info(),
            ctx.accounts.stake_history.to_account_info(),
            ctx.accounts.authority.to_account_info(),
            ctx.accounts.stake_program.to_account_info(),
            amount,
            authority_seeds,
        )?;
        let received = ctx.accounts.sol_escrow.lamports()
            .checked_sub(escrow_before)
            .ok_or_else(|| error!(ProbeError::BalanceDelta))?;
        require_eq!(received, amount, ProbeError::BalanceDelta);
        require_eq!(ctx.accounts.withdrawal_stake.lamports(), 0, ProbeError::BalanceDelta);
        ctx.accounts.round.status = RoundStatus::Finalized;
        emit!(WithdrawalFinalized {
            caller: ctx.accounts.caller.key(),
            round_id: ctx.accounts.round.round_id,
            withdrawal_stake: ctx.accounts.withdrawal_stake.key(),
            sol_escrow: ctx.accounts.sol_escrow.key(),
            lamports_received: received,
        });
        Ok(())
    }
}

trait PoolAccounts<'info> {
    fn stake_pool_program_info(&self) -> AccountInfo<'info>;
    fn stake_pool_info(&self) -> AccountInfo<'info>;
    fn stake_pool_key(&self) -> Pubkey;
    fn validator_list_key(&self) -> Pubkey;
    fn reserve_stake_key(&self) -> Pubkey;
    fn pool_mint_key(&self) -> Pubkey;
    fn manager_fee_key(&self) -> Pubkey;
    fn token_program_key(&self) -> Pubkey;
    fn withdraw_authority_key(&self) -> Pubkey;
}

fn checked_pool<'info, T: PoolAccounts<'info>>(accounts: &T) -> Result<spl_stake_pool::state::StakePool> {
    let bindings = PoolBindings {
        stake_pool: accounts.stake_pool_key(),
        validator_list: accounts.validator_list_key(),
        reserve_stake: accounts.reserve_stake_key(),
        pool_mint: accounts.pool_mint_key(),
        manager_fee_account: accounts.manager_fee_key(),
        token_program: accounts.token_program_key(),
        withdraw_authority: accounts.withdraw_authority_key(),
    };
    validate_pool_bindings(&accounts.stake_pool_program_info(), &accounts.stake_pool_info(), &bindings)
}

fn validate_withdrawal_authorities(stake: &AccountInfo<'_>, authority: &Pubkey) -> Result<()> {
    let state = validation::decode_stake_state(stake)?;
    let meta = state.meta().ok_or_else(|| error!(ProbeError::InvalidStakeState))?;
    require_keys_eq!(meta.authorized.staker, *authority, ProbeError::WrongStakeAuthority);
    require_keys_eq!(meta.authorized.withdrawer, *authority, ProbeError::WrongStakeAuthority);
    Ok(())
}

fn validate_withdrawal_ready(stake: &AccountInfo<'_>, authority: &Pubkey, current_epoch: u64) -> Result<()> {
    let state = validation::decode_stake_state(stake)?;
    let meta = state.meta().ok_or_else(|| error!(ProbeError::InvalidStakeState))?;
    require_keys_eq!(meta.authorized.staker, *authority, ProbeError::WrongStakeAuthority);
    require_keys_eq!(meta.authorized.withdrawer, *authority, ProbeError::WrongStakeAuthority);
    match state {
        StakeStateV2::Initialized(_) => Ok(()),
        StakeStateV2::Stake(_, stake_state, _) => {
            require!(
                stake_state.delegation.deactivation_epoch != u64::MAX
                    && stake_state.delegation.deactivation_epoch < current_epoch,
                ProbeError::StakeNotDeactivated
            );
            Ok(())
        }
        _ => err!(ProbeError::InvalidStakeState),
    }
}

#[derive(Accounts)]
pub struct Initialize<'info> {
    #[account(
        init,
        payer = caller,
        space = 8 + ProbeConfig::INIT_SPACE,
        seeds = [CONFIG_SEED],
        bump
    )]
    pub config: Account<'info, ProbeConfig>,
    #[account(mut)]
    pub caller: Signer<'info>,
    /// CHECK: Address-only signer PDA; it deliberately owns no data.
    #[account(seeds = [AUTHORITY_SEED], bump)]
    pub authority: UncheckedAccount<'info>,
    /// CHECK: System-owned custody PDA; direct transfers create it when funded.
    #[account(mut, seeds = [SOL_VAULT_SEED], bump)]
    pub sol_vault: UncheckedAccount<'info>,
    /// CHECK: Fixed system-owned final native-SOL destination.
    #[account(mut, seeds = [SOL_ESCROW_SEED], bump)]
    pub sol_escrow: UncheckedAccount<'info>,
    /// CHECK: Decoded and bound to authority and JitoSOL in the handler.
    #[account(mut)]
    pub token_vault: UncheckedAccount<'info>,
    /// CHECK: Must equal and be owned by the official SPL stake-pool program.
    pub stake_pool: UncheckedAccount<'info>,
    /// CHECK: Bound through decoded stake-pool state.
    pub validator_list: UncheckedAccount<'info>,
    /// CHECK: Bound through decoded stake-pool state.
    pub reserve_stake: UncheckedAccount<'info>,
    /// CHECK: Bound through decoded stake-pool state.
    pub manager_fee_account: UncheckedAccount<'info>,
    /// CHECK: Decoded as the fixed JitoSOL mint.
    pub pool_mint: UncheckedAccount<'info>,
    /// CHECK: Derived from official stake-pool state.
    pub stake_pool_withdraw_authority: UncheckedAccount<'info>,
    /// CHECK: Fixed executable SPL stake-pool program.
    pub stake_pool_program: UncheckedAccount<'info>,
    /// CHECK: Fixed executable legacy SPL Token program.
    pub token_program: UncheckedAccount<'info>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct DepositSol<'info> {
    #[account(
        seeds = [CONFIG_SEED],
        bump = config.config_bump,
        has_one = stake_pool_program,
        has_one = stake_pool,
        has_one = validator_list,
        has_one = reserve_stake,
        has_one = pool_mint,
        has_one = manager_fee_account,
        has_one = token_vault
    )]
    pub config: Account<'info, ProbeConfig>,
    pub caller: Signer<'info>,
    /// CHECK: Fixed PDA signer.
    #[account(seeds = [AUTHORITY_SEED], bump = config.authority_bump)]
    pub authority: UncheckedAccount<'info>,
    /// CHECK: Validated as an empty system account in the handler.
    #[account(mut, seeds = [SOL_VAULT_SEED], bump = config.sol_vault_bump)]
    pub sol_vault: UncheckedAccount<'info>,
    /// CHECK: Validated through pool state.
    #[account(mut)]
    pub stake_pool: UncheckedAccount<'info>,
    /// CHECK: Validated through pool state.
    pub validator_list: UncheckedAccount<'info>,
    /// CHECK: Validated through pool state.
    #[account(mut)]
    pub reserve_stake: UncheckedAccount<'info>,
    /// CHECK: Fixed JitoSOL token vault controlled by authority PDA.
    #[account(mut)]
    pub token_vault: UncheckedAccount<'info>,
    /// CHECK: Validated through pool state.
    #[account(mut)]
    pub manager_fee_account: UncheckedAccount<'info>,
    /// CHECK: Fixed JitoSOL mint.
    #[account(mut)]
    pub pool_mint: UncheckedAccount<'info>,
    /// CHECK: Derived official pool withdraw authority.
    pub stake_pool_withdraw_authority: UncheckedAccount<'info>,
    /// CHECK: Fixed executable official program.
    pub stake_pool_program: UncheckedAccount<'info>,
    /// CHECK: Fixed executable legacy SPL Token program.
    pub token_program: UncheckedAccount<'info>,
    pub system_program: Program<'info, System>,
}

impl<'info> PoolAccounts<'info> for DepositSol<'info> {
    fn stake_pool_program_info(&self) -> AccountInfo<'info> { self.stake_pool_program.to_account_info() }
    fn stake_pool_info(&self) -> AccountInfo<'info> { self.stake_pool.to_account_info() }
    fn stake_pool_key(&self) -> Pubkey { self.stake_pool.key() }
    fn validator_list_key(&self) -> Pubkey { self.validator_list.key() }
    fn reserve_stake_key(&self) -> Pubkey { self.reserve_stake.key() }
    fn pool_mint_key(&self) -> Pubkey { self.pool_mint.key() }
    fn manager_fee_key(&self) -> Pubkey { self.manager_fee_account.key() }
    fn token_program_key(&self) -> Pubkey { self.token_program.key() }
    fn withdraw_authority_key(&self) -> Pubkey { self.stake_pool_withdraw_authority.key() }
}

#[derive(Accounts)]
pub struct ContributeJitoSol<'info> {
    #[account(seeds = [CONFIG_SEED], bump = config.config_bump, has_one = pool_mint, has_one = token_vault)]
    pub config: Account<'info, ProbeConfig>,
    pub caller: Signer<'info>,
    /// CHECK: Fixed PDA authority.
    #[account(seeds = [AUTHORITY_SEED], bump = config.authority_bump)]
    pub authority: UncheckedAccount<'info>,
    /// CHECK: Decoded as a caller-owned JitoSOL account.
    #[account(mut)]
    pub source_token_account: UncheckedAccount<'info>,
    /// CHECK: Decoded as the fixed PIV-controlled JitoSOL account.
    #[account(mut)]
    pub token_vault: UncheckedAccount<'info>,
    /// CHECK: Decoded as JitoSOL mint.
    pub pool_mint: UncheckedAccount<'info>,
    /// CHECK: Fixed executable legacy SPL Token program.
    #[account(address = spl_token::id() @ ProbeError::WrongTokenProgram)]
    pub token_program: UncheckedAccount<'info>,
}

#[derive(Accounts)]
#[instruction(round_id: u64)]
pub struct InitiateWithdrawal<'info> {
    #[account(
        mut,
        seeds = [CONFIG_SEED],
        bump = config.config_bump,
        has_one = stake_pool_program,
        has_one = stake_pool,
        has_one = validator_list,
        has_one = reserve_stake,
        has_one = pool_mint,
        has_one = manager_fee_account,
        has_one = token_vault
    )]
    pub config: Account<'info, ProbeConfig>,
    #[account(
        init,
        payer = caller,
        space = 8 + WithdrawalRound::INIT_SPACE,
        seeds = [ROUND_SEED, &round_id.to_le_bytes()],
        bump
    )]
    pub round: Account<'info, WithdrawalRound>,
    #[account(mut)]
    pub caller: Signer<'info>,
    /// CHECK: Fixed PDA signs token burn/fee transfer and becomes both stake authorities.
    #[account(seeds = [AUTHORITY_SEED], bump = config.authority_bump)]
    pub authority: UncheckedAccount<'info>,
    /// CHECK: Empty system-owned PDA pays reusable operational stake rent.
    #[account(mut, seeds = [SOL_VAULT_SEED], bump = config.sol_vault_bump)]
    pub sol_vault: UncheckedAccount<'info>,
    /// CHECK: Created as a Stake Program-owned deterministic PDA in the handler.
    #[account(mut, seeds = [WITHDRAWAL_STAKE_SEED, &round_id.to_le_bytes()], bump)]
    pub withdrawal_stake: UncheckedAccount<'info>,
    /// CHECK: Validated through pool state.
    #[account(mut)]
    pub stake_pool: UncheckedAccount<'info>,
    /// CHECK: Validated through pool state and by stake-pool CPI.
    #[account(mut)]
    pub validator_list: UncheckedAccount<'info>,
    /// CHECK: Included to bind the entire configured topology.
    pub reserve_stake: UncheckedAccount<'info>,
    /// CHECK: Derived active validator stake source validated before CPI.
    #[account(mut)]
    pub validator_stake: UncheckedAccount<'info>,
    /// CHECK: Fixed PIV-controlled JitoSOL vault.
    #[account(mut)]
    pub token_vault: UncheckedAccount<'info>,
    /// CHECK: Validated through pool state.
    #[account(mut)]
    pub manager_fee_account: UncheckedAccount<'info>,
    /// CHECK: Fixed JitoSOL mint.
    #[account(mut)]
    pub pool_mint: UncheckedAccount<'info>,
    /// CHECK: Derived official pool withdraw authority.
    pub stake_pool_withdraw_authority: UncheckedAccount<'info>,
    pub clock: Sysvar<'info, Clock>,
    /// CHECK: Fixed executable SPL stake-pool program.
    pub stake_pool_program: UncheckedAccount<'info>,
    /// CHECK: Fixed executable legacy SPL Token program.
    #[account(address = spl_token::id() @ ProbeError::WrongTokenProgram)]
    pub token_program: UncheckedAccount<'info>,
    /// CHECK: Fixed executable Stake Program.
    #[account(address = solana_stake_interface::program::id() @ ProbeError::WrongStakeProgram)]
    pub stake_program: UncheckedAccount<'info>,
    pub system_program: Program<'info, System>,
}

impl<'info> PoolAccounts<'info> for InitiateWithdrawal<'info> {
    fn stake_pool_program_info(&self) -> AccountInfo<'info> { self.stake_pool_program.to_account_info() }
    fn stake_pool_info(&self) -> AccountInfo<'info> { self.stake_pool.to_account_info() }
    fn stake_pool_key(&self) -> Pubkey { self.stake_pool.key() }
    fn validator_list_key(&self) -> Pubkey { self.validator_list.key() }
    fn reserve_stake_key(&self) -> Pubkey { self.reserve_stake.key() }
    fn pool_mint_key(&self) -> Pubkey { self.pool_mint.key() }
    fn manager_fee_key(&self) -> Pubkey { self.manager_fee_account.key() }
    fn token_program_key(&self) -> Pubkey { self.token_program.key() }
    fn withdraw_authority_key(&self) -> Pubkey { self.stake_pool_withdraw_authority.key() }
}

#[derive(Accounts)]
#[instruction(round_id: u64)]
pub struct ManageWithdrawalStake<'info> {
    #[account(seeds = [CONFIG_SEED], bump = config.config_bump)]
    pub config: Account<'info, ProbeConfig>,
    #[account(
        mut,
        seeds = [ROUND_SEED, &round_id.to_le_bytes()],
        bump = round.round_bump,
        has_one = config,
        has_one = withdrawal_stake @ ProbeError::RoundStakeMismatch
    )]
    pub round: Account<'info, WithdrawalRound>,
    pub caller: Signer<'info>,
    /// CHECK: Fixed authority PDA signs Stake Program CPI.
    #[account(seeds = [AUTHORITY_SEED], bump = config.authority_bump)]
    pub authority: UncheckedAccount<'info>,
    /// CHECK: Bound by round and decoded as stake state.
    #[account(mut, seeds = [WITHDRAWAL_STAKE_SEED, &round_id.to_le_bytes()], bump = round.stake_bump)]
    pub withdrawal_stake: UncheckedAccount<'info>,
    pub clock: Sysvar<'info, Clock>,
    /// CHECK: Fixed executable Stake Program.
    #[account(address = solana_stake_interface::program::id() @ ProbeError::WrongStakeProgram)]
    pub stake_program: UncheckedAccount<'info>,
}

#[derive(Accounts)]
#[instruction(round_id: u64)]
pub struct CreateInactiveStakeProof<'info> {
    #[account(mut, seeds = [CONFIG_SEED], bump = config.config_bump)]
    pub config: Account<'info, ProbeConfig>,
    #[account(
        init,
        payer = caller,
        space = 8 + WithdrawalRound::INIT_SPACE,
        seeds = [ROUND_SEED, &round_id.to_le_bytes()],
        bump
    )]
    pub round: Account<'info, WithdrawalRound>,
    #[account(mut)]
    pub caller: Signer<'info>,
    /// CHECK: Fixed authority PDA becomes both stake authorities.
    #[account(seeds = [AUTHORITY_SEED], bump = config.authority_bump)]
    pub authority: UncheckedAccount<'info>,
    /// CHECK: Empty system-owned PDA pays reusable operational stake rent.
    #[account(mut, seeds = [SOL_VAULT_SEED], bump = config.sol_vault_bump)]
    pub sol_vault: UncheckedAccount<'info>,
    /// CHECK: Created and initialized as a deterministic inactive stake PDA.
    #[account(mut, seeds = [WITHDRAWAL_STAKE_SEED, &round_id.to_le_bytes()], bump)]
    pub withdrawal_stake: UncheckedAccount<'info>,
    pub rent: Sysvar<'info, Rent>,
    /// CHECK: Fixed executable Stake Program.
    #[account(address = solana_stake_interface::program::id() @ ProbeError::WrongStakeProgram)]
    pub stake_program: UncheckedAccount<'info>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
#[instruction(round_id: u64)]
pub struct FinalizeWithdrawal<'info> {
    #[account(seeds = [CONFIG_SEED], bump = config.config_bump)]
    pub config: Account<'info, ProbeConfig>,
    #[account(
        mut,
        seeds = [ROUND_SEED, &round_id.to_le_bytes()],
        bump = round.round_bump,
        has_one = config,
        has_one = withdrawal_stake @ ProbeError::RoundStakeMismatch
    )]
    pub round: Account<'info, WithdrawalRound>,
    pub caller: Signer<'info>,
    /// CHECK: Fixed authority PDA signs Stake Program CPI.
    #[account(seeds = [AUTHORITY_SEED], bump = config.authority_bump)]
    pub authority: UncheckedAccount<'info>,
    /// CHECK: Bound by round and decoded as stake state.
    #[account(mut, seeds = [WITHDRAWAL_STAKE_SEED, &round_id.to_le_bytes()], bump = round.stake_bump)]
    pub withdrawal_stake: UncheckedAccount<'info>,
    /// CHECK: Fixed system-owned destination; no caller-provided address is accepted.
    #[account(mut, seeds = [SOL_ESCROW_SEED], bump = config.sol_escrow_bump)]
    pub sol_escrow: UncheckedAccount<'info>,
    pub clock: Sysvar<'info, Clock>,
    /// CHECK: Fixed Stake History sysvar required by Stake Program.
    #[account(address = solana_stake_interface::stake_history::id())]
    pub stake_history: UncheckedAccount<'info>,
    /// CHECK: Fixed executable Stake Program.
    #[account(address = solana_stake_interface::program::id() @ ProbeError::WrongStakeProgram)]
    pub stake_program: UncheckedAccount<'info>,
}

#[event]
pub struct ProbeInitialized {
    pub config: Pubkey,
    pub authority: Pubkey,
    pub sol_vault: Pubkey,
    pub token_vault: Pubkey,
    pub sol_escrow: Pubkey,
}

#[event]
pub struct SolDeposited {
    pub caller: Pubkey,
    pub lamports_in: u64,
    pub pool_tokens_out: u64,
}

#[event]
pub struct JitoSolContributed {
    pub caller: Pubkey,
    pub amount: u64,
}

#[event]
pub struct WithdrawalInitiated {
    pub caller: Pubkey,
    pub round_id: u64,
    pub withdrawal_stake: Pubkey,
    pub pool_tokens_in: u64,
    pub expected_lamports_out: u64,
    pub rent_funded: u64,
    pub deactivated: bool,
}

#[event]
pub struct StakeDeactivated {
    pub caller: Pubkey,
    pub round_id: u64,
    pub withdrawal_stake: Pubkey,
}

#[event]
pub struct InactiveStakeProofCreated {
    pub caller: Pubkey,
    pub round_id: u64,
    pub withdrawal_stake: Pubkey,
    pub total_lamports: u64,
}

#[event]
pub struct WithdrawalFinalized {
    pub caller: Pubkey,
    pub round_id: u64,
    pub withdrawal_stake: Pubkey,
    pub sol_escrow: Pubkey,
    pub lamports_received: u64,
}
