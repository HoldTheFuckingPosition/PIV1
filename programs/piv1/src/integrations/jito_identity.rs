//! Seven-account Jito/SPL identity evidence, not execution readiness.
//!
//! Fixed source identities come from the pinned Jito reference and SPL 2.0.3.
//! Runtime accounts cannot attest cluster genesis or deployed binary revision.
//! Pool epochs, supplies, fees, preferences and optional authorities remain raw
//! facts; no quote, fee conversion, funding, governance or CPI is performed.
//! Reauthenticate after mutation/CPI. No persistent capability is returned.

use anchor_lang::{prelude::{AccountInfo, Pubkey}, solana_program::{bpf_loader_upgradeable, program_pack::Pack}};
use spl_token::state::{Account as TokenAccount, AccountState, Mint};
use crate::accounts::STAKE_PROGRAM_ID;

/// Jito reference b553e90d39e1ff583011dab344a11b5d9bfd284c constants/index.ts.
pub const JITO_STAKE_POOL_PROGRAM: Pubkey = Pubkey::from_str_const("SPoo1Ku8WFXoNDMHPsrGSTSG1Y47rzgn41SLUNakuHy");
pub const JITO_STAKE_POOL: Pubkey = Pubkey::from_str_const("Jito4APyf642JPZPx3hGc6WWJ8zPKtRbRs4P815Awbb");
pub const JITOSOL_MINT: Pubkey = Pubkey::from_str_const("J1toso1uCk3RLmjorhTtrVwY9HJ7X8V9yYac6Y7kGCPn");

/// Declarations to bind, never an alternative trusted program/pool/mint profile.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct DeclaredJitoKeys {
    pub program: Pubkey, pub pool: Pubkey, pub validator_list: Pubkey, pub reserve: Pubkey,
    pub mint: Pubkey, pub manager_fee: Pubkey, pub referrer: Pubkey,
}
#[derive(Clone, Copy)]
pub struct JitoIdentityAccountInfos<'a, 'info> {
    pub program: &'a AccountInfo<'info>, pub pool: &'a AccountInfo<'info>,
    pub validator_list: &'a AccountInfo<'info>, pub reserve: &'a AccountInfo<'info>,
    pub mint: &'a AccountInfo<'info>, pub manager_fee: &'a AccountInfo<'info>, pub referrer: &'a AccountInfo<'info>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum JitoIdentityError {
    InvalidIdentity, AccountAlias, InvalidOwner, InvalidExecutable, BorrowFailed,
    UnsupportedProgram, InvalidPool, InvalidFee, InvalidList, InvalidReserve, InvalidMint, InvalidReceiver,
}
pub type JitoIdentityResult<T> = Result<T, JitoIdentityError>;
use JitoIdentityError as E;

/// Raw upstream denominator/numerator encoding. Valid zero encodings are not
/// converted into the accounting mock's canonical FeeFraction representation.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct RawFee { pub denominator: u64, pub numerator: u64 }
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum FutureFee { None, One(RawFee), Two(RawFee) }
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct RawLockup { pub unix_timestamp: i64, pub epoch: u64, pub custodian: Pubkey }

macro_rules! pool_fields {
    ($($name:ident: $kind:ty),+ $(,)?) => {
        /// Decoded facts only; this type alone authenticates no account.
        #[derive(Clone, Debug, Eq, PartialEq)]
        pub struct PoolIdentityFields { $($name: $kind),+ }
        impl PoolIdentityFields { $(pub fn $name(&self) -> $kind { self.$name })+ }
    };
}
pool_fields! {
    manager: Pubkey, staker: Pubkey, stake_deposit_authority: Pubkey, stake_withdraw_bump_seed: u8,
    validator_list: Pubkey, reserve_stake: Pubkey, pool_mint: Pubkey, manager_fee_account: Pubkey, token_program_id: Pubkey,
    total_lamports: u64, pool_token_supply: u64, last_update_epoch: u64, lockup: RawLockup,
    epoch_fee: RawFee, next_epoch_fee: FutureFee, preferred_deposit_validator_vote_address: Option<Pubkey>,
    preferred_withdraw_validator_vote_address: Option<Pubkey>, stake_deposit_fee: RawFee, stake_withdrawal_fee: RawFee,
    next_stake_withdrawal_fee: FutureFee, stake_referral_fee: u8, sol_deposit_authority: Option<Pubkey>,
    sol_deposit_fee: RawFee, sol_referral_fee: u8, sol_withdraw_authority: Option<Pubkey>, sol_withdrawal_fee: RawFee,
    next_sol_withdrawal_fee: FutureFee, last_epoch_pool_token_supply: u64, last_epoch_total_lamports: u64,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ValidatorListGeometry { maximum: u32, count: u32 }
impl ValidatorListGeometry {
    pub fn maximum(&self) -> u32 { self.maximum }
    pub fn count(&self) -> u32 { self.count }
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ReserveIdentityFields { rent_exempt_reserve: u64, staker: Pubkey, withdrawer: Pubkey, lockup: RawLockup }
impl ReserveIdentityFields {
    /// Serialized metadata only, not a current Rent/funding observation.
    pub fn rent_exempt_reserve(&self) -> u64 { self.rent_exempt_reserve }
    pub fn staker(&self) -> Pubkey { self.staker }
    pub fn withdrawer(&self) -> Pubkey { self.withdrawer }
    pub fn lockup(&self) -> RawLockup { self.lockup }
}

/// Private-field, non-Clone point-in-time account identity evidence. A valid
/// declared referrer receiver has no inferred official referral designation.
#[derive(Debug, PartialEq)]
pub struct AuthenticatedJitoIdentity {
    keys: DeclaredJitoKeys, pool: PoolIdentityFields, withdraw_authority: Pubkey,
    list: ValidatorListGeometry, reserve: ReserveIdentityFields,
    mint: Mint, manager_fee: TokenAccount, referrer: TokenAccount,
}
impl AuthenticatedJitoIdentity {
    pub fn keys(&self) -> &DeclaredJitoKeys { &self.keys }
    pub fn pool(&self) -> &PoolIdentityFields { &self.pool }
    pub fn withdraw_authority(&self) -> Pubkey { self.withdraw_authority }
    pub fn validator_list(&self) -> ValidatorListGeometry { self.list }
    pub fn reserve(&self) -> ReserveIdentityFields { self.reserve }
    pub fn mint(&self) -> &Mint { &self.mint }
    pub fn manager_fee(&self) -> &TokenAccount { &self.manager_fee }
    pub fn referrer(&self) -> &TokenAccount { &self.referrer }
}

pub fn authenticate_jito_identity(
    declared: &DeclaredJitoKeys, accounts: JitoIdentityAccountInfos<'_, '_>,
) -> JitoIdentityResult<AuthenticatedJitoIdentity> {
    let all = [accounts.program, accounts.pool, accounts.validator_list, accounts.reserve,
        accounts.mint, accounts.manager_fee, accounts.referrer];
    let keys = [declared.program, declared.pool, declared.validator_list, declared.reserve,
        declared.mint, declared.manager_fee, declared.referrer];
    if declared.program != JITO_STAKE_POOL_PROGRAM || declared.pool != JITO_STAKE_POOL || declared.mint != JITOSOL_MINT {
        return Err(E::InvalidIdentity);
    }
    for (index, account) in all.iter().enumerate() {
        if *account.key != keys[index] || *account.key == Pubkey::default() { return Err(E::InvalidIdentity); }
        for previous in 0..index {
            if account.key == all[previous].key && !(index == 6 && previous == 5) { return Err(E::AccountAlias); }
        }
    }
    validate_program(accounts.program)?;
    validate_state_account(accounts.pool, &JITO_STAKE_POOL_PROGRAM)?;
    let pool = decode_pool(&accounts.pool.try_borrow_data().map_err(|_| E::BorrowFailed)?)?;
    let (withdraw_authority, bump) = Pubkey::find_program_address(&[JITO_STAKE_POOL.as_ref(), b"withdraw"], &JITO_STAKE_POOL_PROGRAM);
    if pool.validator_list != declared.validator_list || pool.reserve_stake != declared.reserve
        || pool.pool_mint != JITOSOL_MINT || pool.manager_fee_account != declared.manager_fee
        || pool.token_program_id != spl_token::ID || pool.stake_withdraw_bump_seed != bump
        || pool.lockup != RawLockup::default()
    { return Err(E::InvalidPool); }
    validate_state_account(accounts.validator_list, &JITO_STAKE_POOL_PROGRAM)?;
    let list = decode_list(&accounts.validator_list.try_borrow_data().map_err(|_| E::BorrowFailed)?)?;
    validate_state_account(accounts.reserve, &STAKE_PROGRAM_ID)?;
    let reserve = decode_reserve(&accounts.reserve.try_borrow_data().map_err(|_| E::BorrowFailed)?, &withdraw_authority)?;
    validate_state_account(accounts.mint, &spl_token::ID)?;
    let mint = Mint::unpack(&accounts.mint.try_borrow_data().map_err(|_| E::BorrowFailed)?).map_err(|_| E::InvalidMint)?;
    if mint.decimals != 9 || mint.mint_authority != Some(withdraw_authority).into() || mint.freeze_authority.is_some() {
        return Err(E::InvalidMint);
    }
    let receiver = |account: &AccountInfo<'_>| {
        validate_state_account(account, &spl_token::ID)?;
        let token = TokenAccount::unpack(&account.try_borrow_data().map_err(|_| E::BorrowFailed)?).map_err(|_| E::InvalidReceiver)?;
        if token.state != AccountState::Initialized || token.mint != JITOSOL_MINT || token.is_native.is_some() {
            return Err(E::InvalidReceiver);
        }
        Ok(token)
    };
    let manager_fee = receiver(accounts.manager_fee)?;
    let referrer = receiver(accounts.referrer)?;
    Ok(AuthenticatedJitoIdentity { keys: *declared, pool, withdraw_authority, list, reserve, mint, manager_fee, referrer })
}

fn validate_state_account(account: &AccountInfo<'_>, owner: &Pubkey) -> JitoIdentityResult<()> {
    if account.owner != owner { return Err(E::InvalidOwner); }
    if account.executable { return Err(E::InvalidExecutable); }
    Ok(())
}
fn validate_program(account: &AccountInfo<'_>) -> JitoIdentityResult<()> {
    if !account.executable { return Err(E::InvalidExecutable); }
    if account.owner != &bpf_loader_upgradeable::ID { return Err(E::UnsupportedProgram); }
    let bytes = account.try_borrow_data().map_err(|_| E::BorrowFailed)?;
    // First supported loader profile only: loader-v3 Program tag and exact
    // canonical ProgramData pointer. No ProgramData bytes/artifact are supplied.
    if bytes.len() != 36 { return Err(E::UnsupportedProgram); }
    let mut reader = Reader(&bytes);
    let expected = Pubkey::find_program_address(&[account.key.as_ref()], &bpf_loader_upgradeable::ID).0;
    if reader.u32()? != 2 || reader.key()? != expected { return Err(E::UnsupportedProgram); }
    Ok(())
}

fn decode_list(bytes: &[u8]) -> JitoIdentityResult<ValidatorListGeometry> {
    if bytes.len() < 9 { return Err(E::InvalidList); }
    let mut reader = Reader(bytes);
    if reader.byte()? != 2 { return Err(E::InvalidList); }
    let maximum = reader.u32()?; let count = reader.u32()?;
    if maximum == 0 || count > maximum || u64::try_from((bytes.len() - 9) / 73).map_err(|_| E::InvalidList)? != u64::from(maximum) {
        return Err(E::InvalidList);
    }
    // Upstream capacity is floor division. Unused entries and 0..72 residual
    // bytes are opaque; no validator status, balance or epoch is inspected.
    Ok(ValidatorListGeometry { maximum, count })
}
fn decode_reserve(bytes: &[u8], authority: &Pubkey) -> JitoIdentityResult<ReserveIdentityFields> {
    if bytes.len() != 200 { return Err(E::InvalidReserve); }
    let mut reader = Reader(bytes);
    if reader.u32()? != 1 { return Err(E::InvalidReserve); }
    let reserve = ReserveIdentityFields { rent_exempt_reserve: reader.u64()?, staker: reader.key()?,
        withdrawer: reader.key()?, lockup: reader.lockup()? };
    if reserve.staker != *authority || reserve.withdrawer != *authority || reserve.lockup != RawLockup::default() {
        return Err(E::InvalidReserve);
    }
    // StakeStateV2::Initialized occupies 124 bytes; allocation tail may be stale.
    Ok(reserve)
}

fn decode_pool(bytes: &[u8]) -> JitoIdentityResult<PoolIdentityFields> {
    let mut r = Reader(bytes);
    if r.byte()? != 1 { return Err(E::InvalidPool); }
    let result = PoolIdentityFields {
        manager: r.key()?, staker: r.key()?, stake_deposit_authority: r.key()?, stake_withdraw_bump_seed: r.byte()?,
        validator_list: r.key()?, reserve_stake: r.key()?, pool_mint: r.key()?, manager_fee_account: r.key()?, token_program_id: r.key()?,
        total_lamports: r.u64()?, pool_token_supply: r.u64()?, last_update_epoch: r.u64()?, lockup: r.lockup()?,
        epoch_fee: r.fee()?, next_epoch_fee: r.future_fee()?, preferred_deposit_validator_vote_address: r.optional_key()?,
        preferred_withdraw_validator_vote_address: r.optional_key()?, stake_deposit_fee: r.fee()?, stake_withdrawal_fee: r.fee()?,
        next_stake_withdrawal_fee: r.future_fee()?, stake_referral_fee: r.percentage()?, sol_deposit_authority: r.optional_key()?,
        sol_deposit_fee: r.fee()?, sol_referral_fee: r.percentage()?, sol_withdraw_authority: r.optional_key()?,
        sol_withdrawal_fee: r.fee()?, next_sol_withdrawal_fee: r.future_fee()?,
        last_epoch_pool_token_supply: r.u64()?, last_epoch_total_lamports: r.u64()?,
    };
    // Known complete logical prefix is 435..611 bytes. Extra allocation may
    // contain stale nonzero bytes; prefix compatibility cannot attest upgrades.
    Ok(result)
}

#[cfg(test)]
#[path = "../../tests/support/jito_identity_oracle.rs"]
mod oracle;

#[cfg(test)]
mod parser_tests {
    use super::*;
    #[test]
    fn extracted_upstream_types_cover_all_option_and_future_variants() {
        for mask in 0..16 {
            for tags in 0..27 {
                let expected = oracle::distinct_pool(mask, tags);
                let bytes = borsh1::to_vec(&expected).unwrap();
                assert!((435..=611).contains(&bytes.len()));
                let actual = decode_pool(&bytes).unwrap();
                oracle::assert_fields(&actual, &expected);
                let mut padded = bytes.clone(); padded.resize(2000, 0xA5);
                assert_eq!(decode_pool(&padded).unwrap(), actual);
            }
        }
    }
    #[test]
    fn every_prefix_truncation_and_unknown_option_future_tag_rejects() {
        let pool = oracle::distinct_pool(15, 26);
        let bytes = borsh1::to_vec(&pool).unwrap(); assert_eq!(bytes.len(),611);
        for end in 0..bytes.len() { assert!(decode_pool(&bytes[..end]).is_err(), "end={end}"); }
        // Offsets independently follow the maximum populated upstream encoding.
        for offset in [346,363,396,461,479,529,578] {
            for tag in [3,255] {
                let mut bad = bytes.clone(); bad[offset]=tag; assert_eq!(decode_pool(&bad),Err(E::InvalidPool));
            }
        }
        for offset in [363,396,479,529] {
            let mut bad=bytes.clone(); bad[offset]=2; assert_eq!(decode_pool(&bad),Err(E::InvalidPool));
        }
        for discriminator in [0,2,255] {
            let mut bad=bytes.clone(); bad[0]=discriminator; assert_eq!(decode_pool(&bad),Err(E::InvalidPool));
        }
    }
    #[test]
    fn raw_zero_and_full_fees_are_valid_without_mock_normalization() {
        for fee in [oracle::Fee {denominator:0,numerator:0}, oracle::Fee {denominator:77,numerator:0},
            oracle::Fee {denominator:u64::MAX,numerator:u64::MAX}] {
            let mut p=oracle::distinct_pool(0,0); p.epoch_fee=fee; p.next_epoch_fee=oracle::FutureEpoch::Two(fee);
            p.stake_referral_fee=100; p.sol_referral_fee=100;
            let actual=decode_pool(&borsh1::to_vec(&p).unwrap()).unwrap(); oracle::assert_fields(&actual,&p);
        }
        let mut p=oracle::distinct_pool(0,0); p.sol_deposit_fee=oracle::Fee {denominator:0,numerator:1};
        assert_eq!(decode_pool(&borsh1::to_vec(&p).unwrap()),Err(E::InvalidFee));
        let mut p=oracle::distinct_pool(0,0); p.next_sol_withdrawal_fee=oracle::FutureEpoch::One(oracle::Fee {denominator:9,numerator:10});
        assert_eq!(decode_pool(&borsh1::to_vec(&p).unwrap()),Err(E::InvalidFee));
        for stake in [false,true] {
            let mut p=oracle::distinct_pool(0,0); if stake {p.stake_referral_fee=101} else {p.sol_referral_fee=101};
            assert_eq!(decode_pool(&borsh1::to_vec(&p).unwrap()),Err(E::InvalidFee));
        }
    }
}
struct Reader<'a>(&'a [u8]);
impl Reader<'_> {
    fn array<const N: usize>(&mut self) -> JitoIdentityResult<[u8; N]> {
        let value = self.0.get(..N).ok_or(E::InvalidPool)?.try_into().map_err(|_| E::InvalidPool)?;
        self.0 = &self.0[N..]; Ok(value)
    }
    fn byte(&mut self) -> JitoIdentityResult<u8> { Ok(self.array::<1>()?[0]) }
    fn u32(&mut self) -> JitoIdentityResult<u32> { Ok(u32::from_le_bytes(self.array()?)) }
    fn u64(&mut self) -> JitoIdentityResult<u64> { Ok(u64::from_le_bytes(self.array()?)) }
    fn key(&mut self) -> JitoIdentityResult<Pubkey> { Ok(Pubkey::new_from_array(self.array()?)) }
    fn lockup(&mut self) -> JitoIdentityResult<RawLockup> {
        Ok(RawLockup { unix_timestamp: i64::from_le_bytes(self.array()?), epoch: self.u64()?, custodian: self.key()? })
    }
    fn fee(&mut self) -> JitoIdentityResult<RawFee> {
        let fee = RawFee { denominator: self.u64()?, numerator: self.u64()? };
        if fee.numerator > fee.denominator { return Err(E::InvalidFee); } Ok(fee)
    }
    fn future_fee(&mut self) -> JitoIdentityResult<FutureFee> {
        match self.byte()? { 0 => Ok(FutureFee::None), 1 => Ok(FutureFee::One(self.fee()?)),
            2 => Ok(FutureFee::Two(self.fee()?)), _ => Err(E::InvalidPool) }
    }
    fn optional_key(&mut self) -> JitoIdentityResult<Option<Pubkey>> {
        match self.byte()? { 0 => Ok(None), 1 => Ok(Some(self.key()?)), _ => Err(E::InvalidPool) }
    }
    fn percentage(&mut self) -> JitoIdentityResult<u8> {
        let value = self.byte()?; if value > 100 { return Err(E::InvalidFee); } Ok(value)
    }
}
