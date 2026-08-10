use anchor_lang::prelude::*;

pub const CONFIG_SEED: &[u8] = b"config";
pub const AUTHORITY_SEED: &[u8] = b"authority";
pub const SOL_VAULT_SEED: &[u8] = b"sol-vault";
pub const SOL_ESCROW_SEED: &[u8] = b"sol-escrow";
pub const ROUND_SEED: &[u8] = b"round";
pub const WITHDRAWAL_STAKE_SEED: &[u8] = b"withdrawal-stake";

pub const JITO_STAKE_POOL: Pubkey = pubkey!("Jito4APyf642JPZPx3hGc6WWJ8zPKtRbRs4P815Awbb");
pub const JITOSOL_MINT: Pubkey = pubkey!("J1toso1uCk3RLmjorhTtrVwY9HJ7X8V9yYac6Y7kGCPn");
pub const STAKE_ACCOUNT_SPACE: usize = 200;
