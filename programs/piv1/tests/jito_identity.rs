#[path = "support/jito_identity_oracle.rs"]
mod oracle;
use piv1::integrations; // Shared extracted oracle also compiles in crate unit tests.

mod jito_identity_regressions {
    use super::oracle::{self, key, StakePool};
    use anchor_lang::{prelude::{AccountInfo,Pubkey}, solana_program::{bpf_loader_upgradeable,program_option::COption,program_pack::Pack}};
    use piv1::{accounts::STAKE_PROGRAM_ID,integrations::jito_identity::*};
    use solana_stake_interface::state::{Authorized,Lockup,Meta,StakeStateV2};
    use spl_token::state::{Account as TokenAccount,AccountState,Mint};

    #[derive(Clone,Debug,PartialEq)]
    struct Account {key:Pubkey,owner:Pubkey,executable:bool,signer:bool,writable:bool,lamports:u64,data:Vec<u8>}
    impl Account {
        fn info(&mut self)->AccountInfo<'_>{AccountInfo::new(&self.key,self.signer,self.writable,&mut self.lamports,
            &mut self.data,&self.owner,self.executable,0)}
    }
    #[derive(Clone,Debug,PartialEq)]
    struct Fixture {keys:DeclaredJitoKeys,accounts:Vec<Account>,pool:StakePool,reserve:Meta,mint:Mint,receiver:TokenAccount}
    impl Fixture {
        fn new()->Self {
            let keys=DeclaredJitoKeys {program:JITO_STAKE_POOL_PROGRAM,pool:JITO_STAKE_POOL,validator_list:key(51),reserve:key(52),
                mint:JITOSOL_MINT,manager_fee:key(53),referrer:key(54)};
            let (withdraw,bump)=Pubkey::find_program_address(&[JITO_STAKE_POOL.as_ref(),b"withdraw"],&JITO_STAKE_POOL_PROGRAM);
            let mut pool=oracle::distinct_pool(0,0);pool.lockup=Lockup::default();pool.validator_list=keys.validator_list;
            pool.reserve_stake=keys.reserve;pool.pool_mint=keys.mint;pool.manager_fee_account=keys.manager_fee;
            pool.token_program_id=spl_token::ID;pool.stake_withdraw_bump_seed=bump;
            let reserve=Meta {rent_exempt_reserve:1234,authorized:Authorized {staker:withdraw,withdrawer:withdraw},lockup:Lockup::default()};
            let mint=Mint {mint_authority:COption::Some(withdraw),supply:9999,decimals:9,is_initialized:true,freeze_authority:COption::None};
            let receiver=TokenAccount {mint:keys.mint,owner:key(41),amount:50,delegate:COption::Some(key(42)),
                state:AccountState::Initialized,is_native:COption::None,delegated_amount:40,close_authority:COption::Some(key(43))};
            let mut program_data=2_u32.to_le_bytes().to_vec();
            program_data.extend(Pubkey::find_program_address(&[keys.program.as_ref()],&bpf_loader_upgradeable::ID).0.to_bytes());
            let account=|key,owner,executable,data|Account {key,owner,executable,signer:false,writable:false,lamports:1,data};
            let mut list=vec![2];list.extend(2_u32.to_le_bytes());list.extend(1_u32.to_le_bytes());list.resize(9+73*2+13,0xFF);
            let accounts=vec![account(keys.program,bpf_loader_upgradeable::ID,true,program_data),
                account(keys.pool,keys.program,false,vec![]),account(keys.validator_list,keys.program,false,list),
                account(keys.reserve,STAKE_PROGRAM_ID,false,vec![]),account(keys.mint,spl_token::ID,false,vec![]),
                account(keys.manager_fee,spl_token::ID,false,vec![]),account(keys.referrer,spl_token::ID,false,vec![])];
            let mut f=Self {keys,accounts,pool,reserve,mint,receiver};f.sync_pool();f.sync_reserve();f.sync_mint();f.sync_receivers();f
        }
        fn sync_pool(&mut self){self.accounts[1].data=borsh1::to_vec(&self.pool).unwrap();self.accounts[1].data.resize(611,0xA5);}
        fn sync_reserve(&mut self){self.accounts[3].data=borsh1::to_vec(&StakeStateV2::Initialized(self.reserve)).unwrap();
            assert_eq!(self.accounts[3].data.len(),124);self.accounts[3].data.resize(200,0xA5);}
        fn sync_mint(&mut self){self.accounts[4].data=vec![0;82];Mint::pack(self.mint,&mut self.accounts[4].data).unwrap();}
        fn sync_receivers(&mut self){for index in [5,6]{self.accounts[index].data=vec![0;165];TokenAccount::pack(self.receiver,&mut self.accounts[index].data).unwrap();}}
        fn infos<T>(&mut self,f:impl FnOnce(&[AccountInfo<'_>])->T)->T{let infos:Vec<_>=self.accounts.iter_mut().map(Account::info).collect();f(&infos)}
        fn run(&mut self)->JitoIdentityResult<AuthenticatedJitoIdentity>{let before=self.clone();let keys=self.keys;
            let result=self.infos(|a|authenticate_jito_identity(&keys,roles(a)));assert_eq!(*self,before,"all inputs preserved");result}
        fn reject(&mut self,error:JitoIdentityError){assert_eq!(self.run(),Err(error));}
    }
    fn roles<'a,'info>(a:&'a [AccountInfo<'info>])->JitoIdentityAccountInfos<'a,'info>{JitoIdentityAccountInfos {
        program:&a[0],pool:&a[1],validator_list:&a[2],reserve:&a[3],mint:&a[4],manager_fee:&a[5],referrer:&a[6]}}

    #[test]
    fn complete_identity_preserves_raw_facts_and_does_not_claim_epoch_supply_or_rent_readiness(){
        let mut f=Fixture::new();let evidence=f.run().unwrap();
        assert_eq!(evidence.keys(),&f.keys);oracle::assert_fields(evidence.pool(),&f.pool);
        assert_eq!(evidence.validator_list().maximum(),2);assert_eq!(evidence.validator_list().count(),1);
        assert_eq!(evidence.reserve().rent_exempt_reserve(),1234);assert_eq!(evidence.reserve().staker(),f.reserve.authorized.staker);
        assert_eq!(evidence.reserve().withdrawer(),f.reserve.authorized.withdrawer);assert_eq!(evidence.reserve().lockup(),RawLockup::default());
        assert_eq!(evidence.withdraw_authority(),f.reserve.authorized.staker);assert_eq!(evidence.mint(),&f.mint);
        assert_eq!(evidence.manager_fee(),&f.receiver);assert_eq!(evidence.referrer(),&f.receiver);
        assert_ne!(evidence.pool().pool_token_supply(),evidence.mint().supply);
        for value in [0,u64::MAX]{f.pool.total_lamports=value;f.pool.pool_token_supply=value;f.pool.last_update_epoch=value;f.sync_pool();f.run().unwrap();}
        for a in &mut f.accounts {a.signer=true;a.writable=true;a.lamports=0;}f.run().unwrap();
    }
    #[test]
    fn every_identity_declaration_actual_key_and_alias_is_checked(){
        for role in 0..7 {
            let mut f=Fixture::new();f.accounts[role].key=key(100);f.reject(JitoIdentityError::InvalidIdentity);
            let mut f=Fixture::new();match role{0=>f.keys.program=key(100),1=>f.keys.pool=key(100),2=>f.keys.validator_list=key(100),
                3=>f.keys.reserve=key(100),4=>f.keys.mint=key(100),5=>f.keys.manager_fee=key(100),_=>f.keys.referrer=key(100)}
            f.reject(JitoIdentityError::InvalidIdentity);
        }
        let mut f=Fixture::new();f.keys.referrer=f.keys.reserve;f.accounts[6].key=f.keys.reserve;f.reject(JitoIdentityError::AccountAlias);
        let mut f=Fixture::new();f.keys.referrer=Pubkey::default();f.accounts[6].key=Pubkey::default();f.reject(JitoIdentityError::InvalidIdentity);
        let mut f=Fixture::new();f.keys.referrer=f.keys.manager_fee;let keys=f.keys;
        let before=f.clone();f.infos(|a|{let mut accounts=roles(a);accounts.referrer=&a[5];
            assert_eq!(authenticate_jito_identity(&keys,accounts).unwrap().manager_fee(),&f_receiver());});assert_eq!(f,before);
    }
    fn f_receiver()->TokenAccount{Fixture::new().receiver}
    #[test]
    fn every_owner_executable_and_borrow_failure_is_fallible_and_read_only(){
        for role in 0..7 {
            let mut f=Fixture::new();f.accounts[role].owner=key(100);
            f.reject(if role==0{JitoIdentityError::UnsupportedProgram}else{JitoIdentityError::InvalidOwner});
            let mut f=Fixture::new();f.accounts[role].executable=!f.accounts[role].executable;f.reject(JitoIdentityError::InvalidExecutable);
            let mut f=Fixture::new();let before=f.clone();let keys=f.keys;f.infos(|a|{let _guard=a[role].try_borrow_mut_data().unwrap();
                assert_eq!(authenticate_jito_identity(&keys,roles(a)),Err(JitoIdentityError::BorrowFailed));});assert_eq!(f,before);
        }
        let mut f=Fixture::new();let before=f.clone();let keys=f.keys;f.infos(|a|{
            let _guards:Vec<_>=a.iter().map(|a|a.try_borrow_mut_lamports().unwrap()).collect();authenticate_jito_identity(&keys,roles(a)).unwrap();
        });assert_eq!(f,before);
    }
    #[test]
    fn supported_loader_program_tag_pointer_and_exact_size_are_enforced(){
        for length in [0,35,37]{let mut f=Fixture::new();f.accounts[0].data.resize(length,0);f.reject(JitoIdentityError::UnsupportedProgram);}
        let mut f=Fixture::new();f.accounts[0].data[0]=3;f.reject(JitoIdentityError::UnsupportedProgram);
        let mut f=Fixture::new();f.accounts[0].data[4]^=1;f.reject(JitoIdentityError::UnsupportedProgram);
    }
    #[test]
    fn pool_links_bump_and_default_lockup_are_authenticated(){
        let changes:[fn(&mut StakePool);9]=[|p|p.validator_list=key(100),|p|p.reserve_stake=key(100),|p|p.pool_mint=key(100),
            |p|p.manager_fee_account=key(100),|p|p.token_program_id=key(100),|p|p.stake_withdraw_bump_seed^=1,
            |p|p.lockup.unix_timestamp=1,|p|p.lockup.epoch=1,|p|p.lockup.custodian=key(100)];
        for change in changes{let mut f=Fixture::new();change(&mut f.pool);f.sync_pool();f.reject(JitoIdentityError::InvalidPool);}
        let mut f=Fixture::new();f.accounts[1].data.resize(50000,0xF1);f.run().unwrap();
        f.accounts[1].data.truncate(434);f.reject(JitoIdentityError::InvalidPool);
    }
    #[test]
    fn list_geometry_accepts_every_residual_slack_without_decoding_entries(){
        for residual in 0..73 {let mut f=Fixture::new();f.accounts[2].data.resize(9+2*73+residual,0xFF);f.run().unwrap();}
        for length in [0,8,9,9+73,9+3*73]{let mut f=Fixture::new();f.accounts[2].data.resize(length,0);f.reject(JitoIdentityError::InvalidList);}
        for (offset,value) in [(1,0_u32),(1,u32::MAX),(5,3),(5,u32::MAX)]{let mut f=Fixture::new();
            f.accounts[2].data[offset..offset+4].copy_from_slice(&value.to_le_bytes());f.reject(JitoIdentityError::InvalidList);}
        let mut f=Fixture::new();f.accounts[2].data[0]=1;f.reject(JitoIdentityError::InvalidList);
        for count in [0_u32,2]{let mut f=Fixture::new();f.accounts[2].data[5..9].copy_from_slice(&count.to_le_bytes());f.run().unwrap();}
    }
    #[test]
    fn actual_stake_serializer_validates_initialized_authorities_lockup_and_allocation(){
        for length in [0,124,199,201]{let mut f=Fixture::new();f.accounts[3].data.resize(length,0);f.reject(JitoIdentityError::InvalidReserve);}
        for state in [StakeStateV2::Uninitialized,StakeStateV2::RewardsPool]{let mut f=Fixture::new();
            f.accounts[3].data=borsh1::to_vec(&state).unwrap();f.accounts[3].data.resize(200,0);f.reject(JitoIdentityError::InvalidReserve);}
        let changes:[fn(&mut Meta);5]=[|m|m.authorized.staker=key(100),|m|m.authorized.withdrawer=key(100),
            |m|m.lockup.unix_timestamp=1,|m|m.lockup.epoch=1,|m|m.lockup.custodian=key(100)];
        for change in changes{let mut f=Fixture::new();change(&mut f.reserve);f.sync_reserve();f.reject(JitoIdentityError::InvalidReserve);}
    }
    #[test]
    fn legacy_mint_checks_initialized_decimals_and_both_authorities(){
        let changes:[fn(&mut Mint);5]=[|m|m.is_initialized=false,|m|m.decimals=8,|m|m.mint_authority=COption::None,
            |m|m.mint_authority=COption::Some(key(100)),|m|m.freeze_authority=COption::Some(key(100))];
        for change in changes{let mut f=Fixture::new();change(&mut f.mint);f.sync_mint();f.reject(JitoIdentityError::InvalidMint);}
        for length in [0,81,83]{let mut f=Fixture::new();f.accounts[4].data.resize(length,0);f.reject(JitoIdentityError::InvalidMint);}
        let mut f=Fixture::new();f.accounts[4].data[..4].copy_from_slice(&2_u32.to_le_bytes());f.reject(JitoIdentityError::InvalidMint);
    }
    #[test]
    fn external_receivers_allow_authorities_delegates_and_close_but_reject_native_frozen_or_wrong_mint(){
        let changes:[fn(&mut TokenAccount);4]=[|t|t.state=AccountState::Uninitialized,|t|t.state=AccountState::Frozen,
            |t|t.is_native=COption::Some(1),|t|t.mint=key(100)];
        for role in [5,6]{for change in changes{let mut f=Fixture::new();let mut token=f.receiver;change(&mut token);
            TokenAccount::pack(token,&mut f.accounts[role].data).unwrap();f.reject(JitoIdentityError::InvalidReceiver);}
            for length in [0,164,166]{let mut f=Fixture::new();f.accounts[role].data.resize(length,0);f.reject(JitoIdentityError::InvalidReceiver);}}
    }
}
