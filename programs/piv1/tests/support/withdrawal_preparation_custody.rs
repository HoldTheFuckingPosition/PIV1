//! Fixed active-source witness and modeled Stake minimum; no pool execution.
use anchor_lang::prelude::Pubkey;
use solana_stake_interface::{state::{StakeStateV2,Stake,Delegation,Meta,Authorized,Lockup},stake_flags::StakeFlags};
use piv1::{accounts::STAKE_PROGRAM_ID,integrations::jito_identity::{JITO_STAKE_POOL,JITO_STAKE_POOL_PROGRAM},state::WithdrawalLeg};
use super::{preparation_custody,oracle,custody::*,bootstrap_custody::LIST,
    support::kif_claim_custody::{BackingAccount,key}};
#[derive(Clone,Debug,PartialEq)]
pub struct Fixture {pub inner:preparation_custody::Fixture,pub stake:StakeStateV2,pub minimum:u64,pub index:u32}
impl Fixture {
    pub fn new(pending:u64,carry:u64,shared:bool)->Self {
        let mut inner=preparation_custody::Fixture::new(pending,carry,shared,0b101011);
        inner.base.pool.total_lamports=10_000_000;
        inner.base.pool.stake_withdrawal_fee=oracle::Fee{numerator:1,denominator:1000};
        inner.base.pool.preferred_withdraw_validator_vote_address=None;
        inner.base.custody.edit_config(|c|c.protected_principal_hwm_lamports=990_000);
        inner.base.sync();
        let rent=&inner.base.custody.rent;let vote=key(202);
        let withdraw=Pubkey::find_program_address(&[JITO_STAKE_POOL.as_ref(),b"withdraw"],&JITO_STAKE_POOL_PROGRAM).0;
        let source=Pubkey::find_program_address(&[vote.as_ref(),JITO_STAKE_POOL.as_ref()],&JITO_STAKE_POOL_PROGRAM).0;
        let stake=StakeStateV2::Stake(Meta{rent_exempt_reserve:rent.minimum_balance(200),
            authorized:Authorized{staker:withdraw,withdrawer:withdraw},lockup:Lockup::default()},
            Stake{delegation:Delegation{voter_pubkey:vote,stake:5_000_000,activation_epoch:40,..Delegation::default()},credits_observed:17},StakeFlags::empty());
        inner.base.custody.accounts[OPERATIONAL_SOL].lamports=rent.minimum_balance(0)+rent.minimum_balance(200)+rent.minimum_balance(WithdrawalLeg::SPACE)+99;
        inner.base.custody.accounts.extend([
            BackingAccount{key:STAKE_PROGRAM_ID,owner:key(211),lamports:1,data:vec![9;37],signer:false,writable:false,executable:true},
            BackingAccount{key:source,owner:STAKE_PROGRAM_ID,lamports:rent.minimum_balance(200)+5_000_000,
                data:vec![],signer:false,writable:false,executable:false}]);
        let mut f=Self{inner,stake,minimum:100,index:0};f.sync_source();f
    }
    pub fn source(&self)->usize{self.inner.base.custody.accounts.len()-1}
    pub fn sync_source(&mut self){
        let source=self.source();let mut data=borsh1::to_vec(&self.stake).unwrap();data.resize(200,0xa5);
        self.inner.base.custody.accounts[source].data=data;
        let StakeStateV2::Stake(_,stake,_)=self.stake else{return;};
        let mut record=vec![0;73];record[..8].copy_from_slice(&self.inner.base.custody.accounts[source].lamports.to_le_bytes());
        record[16..24].copy_from_slice(&self.inner.base.clock.epoch.to_le_bytes());record[41..73].copy_from_slice(stake.delegation.voter_pubkey.as_ref());
        let mut list=vec![2];list.extend(1_u32.to_le_bytes());list.extend(1_u32.to_le_bytes());list.extend(record);
        self.inner.base.custody.accounts[LIST].data=list;
    }
    pub fn data(&self)->Vec<u8>{let mut data=b"PIV1PW01".to_vec();data.push(1);data.extend(self.index.to_le_bytes());data}
}
