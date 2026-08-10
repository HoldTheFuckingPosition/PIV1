use anchor_lang::prelude::*;

#[account]
#[derive(InitSpace)]
pub struct ProbeConfig {
    pub version: u8,
    pub stake_pool_program: Pubkey,
    pub stake_pool: Pubkey,
    pub validator_list: Pubkey,
    pub reserve_stake: Pubkey,
    pub pool_mint: Pubkey,
    pub manager_fee_account: Pubkey,
    pub token_vault: Pubkey,
    pub config_bump: u8,
    pub authority_bump: u8,
    pub sol_vault_bump: u8,
    pub sol_escrow_bump: u8,
    pub next_round: u64,
}

#[account]
#[derive(InitSpace)]
pub struct WithdrawalRound {
    pub version: u8,
    pub config: Pubkey,
    pub round_id: u64,
    pub withdrawal_stake: Pubkey,
    pub pool_tokens_in: u64,
    pub minimum_lamports_out: u64,
    pub expected_lamports_out: u64,
    pub status: RoundStatus,
    pub stake_bump: u8,
    pub round_bump: u8,
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Copy, Debug, InitSpace, PartialEq, Eq)]
pub enum RoundStatus {
    Withdrawn,
    Deactivating,
    Finalized,
}
