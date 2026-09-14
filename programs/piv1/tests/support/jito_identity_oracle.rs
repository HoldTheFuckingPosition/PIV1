#![allow(dead_code)]
//! Test-only extracted upstream type oracle; no SPL program behavior is executed.
//! Copyright Solana contributors. SPDX-License-Identifier: Apache-2.0.
//! Source: solana-program/stake-pool commit864ba3c1c564cc270ca62b6e6b558f57538ae092,
//! program/src/state.rs SHA25664e9fde6944c036678eba10ab5ddd20d0b2b287b97222d5698c94960c20e6502.
//! The type definitions/derives/default below are exact source excerpts. Imports
//! select already-locked Borsh1.8.0, Pubkey and actual stake-interface1.2.1 Lockup.
//! This verifies serialization compatibility, not full-SPL execution or deployment.
use borsh1::{BorshDeserialize, BorshSerialize, BorshSchema};
use anchor_lang::prelude::Pubkey;
use solana_stake_interface::state::Lockup;

/// Enum representing the account type managed by the program
#[derive(Clone, Debug, Default, PartialEq, BorshDeserialize, BorshSerialize, BorshSchema)]
pub enum AccountType {
    /// If the account has not been initialized, the enum will be 0
    #[default]
    Uninitialized,
    /// Stake pool
    StakePool,
    /// Validator stake list
    ValidatorList,
}

/// Initialized program details.
#[repr(C)]
#[derive(Clone, Debug, Default, PartialEq, BorshDeserialize, BorshSerialize, BorshSchema)]
pub struct StakePool {
    /// Account type, must be `StakePool` currently
    pub account_type: AccountType,

    /// Manager authority, allows for updating the staker, manager, and fee
    /// account
    pub manager: Pubkey,

    /// Staker authority, allows for adding and removing validators, and
    /// managing stake distribution
    pub staker: Pubkey,

    /// Stake deposit authority
    ///
    /// If a depositor pubkey is specified on initialization, then deposits must
    /// be signed by this authority. If no deposit authority is specified,
    /// then the stake pool will default to the result of:
    /// `Pubkey::find_program_address(
    ///     &[&stake_pool_address.as_ref(), b"deposit"],
    ///     program_id,
    /// )`
    pub stake_deposit_authority: Pubkey,

    /// Stake withdrawal authority bump seed
    /// for `create_program_address(&[state::StakePool account, "withdrawal"])`
    pub stake_withdraw_bump_seed: u8,

    /// Validator stake list storage account
    pub validator_list: Pubkey,

    /// Reserve stake account, holds deactivated stake
    pub reserve_stake: Pubkey,

    /// Pool Mint
    pub pool_mint: Pubkey,

    /// Manager fee account
    pub manager_fee_account: Pubkey,

    /// Pool token program id
    pub token_program_id: Pubkey,

    /// Total stake under management.
    /// Note that if `last_update_epoch` does not match the current epoch then
    /// this field may not be accurate
    pub total_lamports: u64,

    /// Total supply of pool tokens (should always match the supply in the Pool
    /// Mint)
    pub pool_token_supply: u64,

    /// Last epoch the `total_lamports` field was updated
    pub last_update_epoch: u64,

    /// Lockup that all stakes in the pool must have
    pub lockup: Lockup,

    /// Fee taken as a proportion of rewards each epoch
    pub epoch_fee: Fee,

    /// Fee for next epoch
    pub next_epoch_fee: FutureEpoch<Fee>,

    /// Preferred deposit validator vote account pubkey
    pub preferred_deposit_validator_vote_address: Option<Pubkey>,

    /// Preferred withdraw validator vote account pubkey
    pub preferred_withdraw_validator_vote_address: Option<Pubkey>,

    /// Fee assessed on stake deposits
    pub stake_deposit_fee: Fee,

    /// Fee assessed on withdrawals
    pub stake_withdrawal_fee: Fee,

    /// Future stake withdrawal fee, to be set for the following epoch
    pub next_stake_withdrawal_fee: FutureEpoch<Fee>,

    /// Fees paid out to referrers on referred stake deposits.
    /// Expressed as a percentage (0 - 100) of deposit fees.
    /// i.e. `stake_deposit_fee`% of stake deposited is collected as deposit
    /// fees for every deposit and `stake_referral_fee`% of the collected
    /// stake deposit fees is paid out to the referrer
    pub stake_referral_fee: u8,

    /// Toggles whether the `DepositSol` instruction requires a signature from
    /// this `sol_deposit_authority`
    pub sol_deposit_authority: Option<Pubkey>,

    /// Fee assessed on SOL deposits
    pub sol_deposit_fee: Fee,

    /// Fees paid out to referrers on referred SOL deposits.
    /// Expressed as a percentage (0 - 100) of SOL deposit fees.
    /// i.e. `sol_deposit_fee`% of SOL deposited is collected as deposit fees
    /// for every deposit and `sol_referral_fee`% of the collected SOL
    /// deposit fees is paid out to the referrer
    pub sol_referral_fee: u8,

    /// Toggles whether the `WithdrawSol` instruction requires a signature from
    /// the `deposit_authority`
    pub sol_withdraw_authority: Option<Pubkey>,

    /// Fee assessed on SOL withdrawals
    pub sol_withdrawal_fee: Fee,

    /// Future SOL withdrawal fee, to be set for the following epoch
    pub next_sol_withdrawal_fee: FutureEpoch<Fee>,

    /// Last epoch's total pool tokens, used only for APR estimation
    pub last_epoch_pool_token_supply: u64,

    /// Last epoch's total lamports, used only for APR estimation
    pub last_epoch_total_lamports: u64,
}

/// Wrapper type that "counts down" epochs, which is Borsh-compatible with the
/// native `Option`
#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, BorshSerialize, BorshDeserialize, BorshSchema)]
pub enum FutureEpoch<T> {
    /// Nothing is set
    None,
    /// Value is ready after the next epoch boundary
    One(T),
    /// Value is ready after two epoch boundaries
    Two(T),
}
impl<T> Default for FutureEpoch<T> {
    fn default() -> Self {
        Self::None
    }
}
impl<T> FutureEpoch<T> {
    /// Create a new value to be unlocked in a two epochs
    pub fn new(value: T) -> Self {
        Self::Two(value)
    }
}

/// Fee rate as a ratio, minted on `UpdateStakePoolBalance` as a proportion of
/// the rewards
/// If either the numerator or the denominator is 0, the fee is considered to be
/// 0
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq, BorshSerialize, BorshDeserialize, BorshSchema)]
pub struct Fee {
    /// denominator of the fee ratio
    pub denominator: u64,
    /// numerator of the fee ratio
    pub numerator: u64,
}


// Locally authored deterministic fixtures/assertions below the exact excerpts.
// Source ranges:30..159,865..887,921..933; full provenance in the task report.
pub fn key(tag: u8) -> Pubkey { Pubkey::new_from_array([tag;32]) }
pub fn distinct_pool(mask: u8, tags: u8) -> StakePool {
    let fee=|n|Fee {denominator:100+u64::from(n),numerator:u64::from(n)};
    let future=|tag,n|match tag {0=>FutureEpoch::None,1=>FutureEpoch::One(fee(n)),_=>FutureEpoch::Two(fee(n))};
    StakePool {
        account_type:AccountType::StakePool,manager:key(1),staker:key(2),stake_deposit_authority:key(3),stake_withdraw_bump_seed:4,
        validator_list:key(5),reserve_stake:key(6),pool_mint:key(7),manager_fee_account:key(8),token_program_id:key(9),
        total_lamports:1011,pool_token_supply:1012,last_update_epoch:1013,
        lockup:Lockup {unix_timestamp:-14,epoch:15,custodian:key(16)},
        epoch_fee:fee(1_u8),next_epoch_fee:future(tags%3,2),
        preferred_deposit_validator_vote_address:(mask&1!=0).then(||key(17)),
        preferred_withdraw_validator_vote_address:(mask&2!=0).then(||key(18)),
        stake_deposit_fee:fee(3),stake_withdrawal_fee:fee(4),next_stake_withdrawal_fee:future((tags/3)%3,5),
        stake_referral_fee:21,sol_deposit_authority:(mask&4!=0).then(||key(22)),
        sol_deposit_fee:fee(6),sol_referral_fee:23,sol_withdraw_authority:(mask&8!=0).then(||key(24)),
        sol_withdrawal_fee:fee(7),next_sol_withdrawal_fee:future(tags/9,8),
        last_epoch_pool_token_supply:1025,last_epoch_total_lamports:1026,
    }
}
pub fn assert_fields(actual:&crate::integrations::jito_identity::PoolIdentityFields, expected:&StakePool) {
    use crate::integrations::jito_identity::{FutureFee,RawFee};
    macro_rules! same {($($field:ident),+)=>{$(assert_eq!(actual.$field(),expected.$field,stringify!($field));)+};}
    same!(manager,staker,stake_deposit_authority,stake_withdraw_bump_seed,validator_list,reserve_stake,pool_mint,
        manager_fee_account,token_program_id,total_lamports,pool_token_supply,last_update_epoch,
        preferred_deposit_validator_vote_address,preferred_withdraw_validator_vote_address,stake_referral_fee,
        sol_deposit_authority,sol_referral_fee,sol_withdraw_authority,last_epoch_pool_token_supply,last_epoch_total_lamports);
    assert_eq!((actual.lockup().unix_timestamp,actual.lockup().epoch,actual.lockup().custodian),
        (expected.lockup.unix_timestamp,expected.lockup.epoch,expected.lockup.custodian));
    let raw=|fee:Fee|RawFee {denominator:fee.denominator,numerator:fee.numerator};
    let future=|fee:FutureEpoch<Fee>|match fee {FutureEpoch::None=>FutureFee::None,
        FutureEpoch::One(f)=>FutureFee::One(raw(f)),FutureEpoch::Two(f)=>FutureFee::Two(raw(f))};
    for (a,e) in [(actual.epoch_fee(),expected.epoch_fee),(actual.stake_deposit_fee(),expected.stake_deposit_fee),
        (actual.stake_withdrawal_fee(),expected.stake_withdrawal_fee),(actual.sol_deposit_fee(),expected.sol_deposit_fee),
        (actual.sol_withdrawal_fee(),expected.sol_withdrawal_fee)] {assert_eq!(a,raw(e));}
    assert_eq!(actual.next_epoch_fee(),future(expected.next_epoch_fee));
    assert_eq!(actual.next_stake_withdrawal_fee(),future(expected.next_stake_withdrawal_fee));
    assert_eq!(actual.next_sol_withdrawal_fee(),future(expected.next_sol_withdrawal_fee));
}
