//! Explicit host effects only. Router effects are modeled, never reported as
//! authentic Jupiter execution or Solana runtime rollback/CU evidence.
mod support;
#[path="support/pending_custody.rs"] pub mod pending_custody;
#[allow(dead_code)] #[path="support/economic_custody.rs"] mod custody;
#[path="support/jito_identity_oracle.rs"] mod oracle;
#[allow(dead_code)] #[path="support/bootstrap_custody.rs"] mod bootstrap_custody;
#[allow(dead_code)] #[path="support/distribution_preparation_custody.rs"] mod preparation_custody;
use anchor_lang::{prelude::{AccountInfo,Pubkey}, AnchorDeserialize, solana_program::{
    instruction::Instruction, program_error::ProgramError, program_option::COption,
    program_pack::Pack, system_program, bpf_loader_upgradeable}};
use spl_token::{instruction::TokenInstruction,state::{Account,AccountState,Mint}};
use piv1::{errors::Piv1Error,
    integrations::{self,jupiter_route as route},
    state::{ActiveDistribution,DistributionLifecycle},
    state_persistence::StateEnvelope,swap_preparation_execution::{self as execution,SELECTOR}};
use support::kif_claim_custody::{BackingAccount,PROGRAM};
use support::kif_claim_custody;
use preparation_custody::Fixture;

fn fixture(pending:u64,carry:u64,prefund:u64)->Fixture{
    let mut f=Fixture::new(pending,carry,false,63);
    let rent=f.base.custody.rent.clone();
    let mut r=f.round();r.version=2;
    f.base.custody.accounts[1].data=StateEnvelope::distribution(&r).unwrap().as_bytes().to_vec();
    f.base.custody.accounts[1].lamports=rent.minimum_balance(ActiveDistribution::SWAP_SPACE);
    for i in [0,1,2,3,4,5,7]{f.base.custody.accounts[i].writable=true;}
    f.base.custody.accounts[4].lamports=rent.minimum_balance(0)+10_000_000;
    let seq=f.base.custody.config().next_distribution_sequence.to_le_bytes();
    let b=f.base.custody.accounts.len();assert_eq!(b,27);
    let account=|key,owner,data:Vec<u8>,writable,executable|BackingAccount{key,owner,lamports:if executable{1}else{rent.minimum_balance(data.len())},data,writable,executable,signer:false};
    for (i,seed)in [route::SWAP_AUTHORITY,route::SWAP_INPUT,route::SWAP_OUTPUT].into_iter().enumerate(){
        let mut a=account(Pubkey::find_program_address(&[seed,&seq],&PROGRAM).0,system_program::ID,vec![],true,false);
        a.lamports=if i==1{prefund}else{0};f.base.custody.accounts.push(a);
    }
    let mint=Mint{mint_authority:COption::None,supply:0,decimals:9,is_initialized:true,freeze_authority:COption::None};
    let mut bytes=vec![0;Mint::LEN];Mint::pack(mint,&mut bytes).unwrap();
    f.base.custody.accounts.push(account(route::WSOL,spl_token::ID,bytes,false,false));
    let mut code=2_u32.to_le_bytes().to_vec();code.extend(Pubkey::from_str_const("4Ec7ZxZS6Sbdg5UGSLHbAnM7GQHp2eFd4KYWRexAipQT").to_bytes());
    f.base.custody.accounts.push(account(route::JUPITER,bpf_loader_upgradeable::ID,code,false,true));
    let mut event=account(Pubkey::find_program_address(&[b"__event_authority"],&route::JUPITER).0,system_program::ID,vec![],false,false);event.lamports=0;
    f.base.custody.accounts.push(event);
    let mut code=2_u32.to_le_bytes().to_vec();code.extend(Pubkey::from_str_const("G92J4ZFggKZ9LbsQ2zBpajRvRto7ed7uRuJJqWsX4BhJ").to_bytes());
    f.base.custody.accounts.push(account(route::MANIFEST,bpf_loader_upgradeable::ID,code,false,true));
    let market=support::kif_claim_custody::key(239);let jito=f.base.custody.config().jitosol_mint;
    let (base_vault,base_bump)=Pubkey::find_program_address(&[b"vault",market.as_ref(),jito.as_ref()],&route::MANIFEST);
    let (quote_vault,quote_bump)=Pubkey::find_program_address(&[b"vault",market.as_ref(),route::WSOL.as_ref()],&route::MANIFEST);
    let global=Pubkey::find_program_address(&[b"global",route::WSOL.as_ref()],&route::MANIFEST).0;
    let global_vault=Pubkey::find_program_address(&[b"global-vault",route::WSOL.as_ref()],&route::MANIFEST).0;
    let mut m=vec![0;256];m[..8].copy_from_slice(&4859840929024028656_u64.to_le_bytes());
    m[9]=9;m[10]=9;m[11]=base_bump;m[12]=quote_bump;
    for (offset,key)in [(16,jito),(48,route::WSOL),(80,base_vault),(112,quote_vault)]{m[offset..offset+32].copy_from_slice(key.as_ref());}
    f.base.custody.accounts.push(account(market,route::MANIFEST,m,true,false));
    let token=|vault,mint,amount|{
        let v=Account{mint,owner:vault,amount,delegate:COption::None,state:AccountState::Initialized,
            is_native:if mint==route::WSOL{COption::Some(rent.minimum_balance(Account::LEN))}else{COption::None},delegated_amount:0,close_authority:COption::None};
        let mut bytes=vec![0;Account::LEN];Account::pack(v,&mut bytes).unwrap();let mut a=account(vault,spl_token::ID,bytes,true,false);
        if mint==route::WSOL{a.lamports+=amount;}a
    };
    f.base.custody.accounts.push(token(base_vault,jito,0));
    f.base.custody.accounts.push(token(quote_vault,route::WSOL,1_000_000));
    let mut g=vec![0;96];g[..8].copy_from_slice(&10787423733276977665_u64.to_le_bytes());g[8..40].copy_from_slice(route::WSOL.as_ref());g[40..72].copy_from_slice(global_vault.as_ref());
    f.base.custody.accounts.push(account(global,route::MANIFEST,g,true,false));
    f.base.custody.accounts.push(token(global_vault,route::WSOL,1_000_000));
    f
}

fn invoke<'i>(ix:&Instruction,a:&[AccountInfo<'i>],seeds:&[&[&[u8]]],rent:u64,output_adjust:i64)->Result<(),ProgramError>{
    let signer=seeds.first().map(|v|Pubkey::create_program_address(v,&PROGRAM).unwrap());
    if ix.program_id==system_program::ID{
        assert_eq!(signer,Some(*a[0].key));
        match u32::from_le_bytes(ix.data[..4].try_into().unwrap()){
            2=>{let amount=u64::from_le_bytes(ix.data[4..12].try_into().unwrap());**a[0].try_borrow_mut_lamports()?-=amount;**a[1].try_borrow_mut_lamports()?+=amount;},
            8=>{assert_eq!(a[0].data_len(),0);*a[0].try_borrow_mut_data()?=Box::leak(vec![0;165].into_boxed_slice());},
            1=>{assert_eq!(&ix.data[4..],spl_token::ID.as_ref());a[0].assign(&spl_token::ID);},
            _=>panic!("unexpected System instruction"),
        }return Ok(());
    }
    if ix.program_id==spl_token::ID{
        if let TokenInstruction::InitializeAccount3{owner}=TokenInstruction::unpack(&ix.data)?{
            let value=Account{mint:*a[1].key,owner,amount:0,delegate:COption::None,state:AccountState::Initialized,
                is_native:if *a[1].key==route::WSOL{COption::Some(rent)}else{COption::None},delegated_amount:0,close_authority:COption::None};
            return Account::pack(value,&mut a[0].try_borrow_mut_data()?);
        }
        let views:Vec<_>=a.iter().map(|v|{let mut view=v.clone();if signer==Some(*v.key){view.is_signer=true;}view}).collect();
        return spl_token::processor::Processor::process(&spl_token::ID,&views,&ix.data);
    }
    assert_eq!(ix.program_id,route::JUPITER);assert_eq!(ix.data.len(),40);assert_eq!(ix.accounts.len(),25);
    assert_eq!(&ix.data[..8],&[187,100,250,204,49,196,175,20]);assert_eq!(&ix.data[24..],&[0,0,0,0,0,0,1,0,0,0,116,1,16,39,0,1]);
    assert_eq!(signer,Some(*a[0].key));assert_eq!(a[0].lamports(),0);
    assert!(!a.iter().any(|v|*v.key==Pubkey::find_program_address(&[b"authority"],&PROGRAM).0));
    assert!(!a.iter().any(|v|*v.key==Pubkey::find_program_address(&[b"operational-sol"],&PROGRAM).0));
    let amount=u64::from_le_bytes(ix.data[8..16].try_into().unwrap());
    let minimum=u64::from_le_bytes(ix.data[16..24].try_into().unwrap());
    let output=minimum.checked_add_signed(output_adjust).unwrap();
    for (index,delta)in [(1,-(amount as i64)),(2,output as i64),(17,amount as i64),(18,-(output as i64))]{
        let mut v=Account::unpack(&a[index].try_borrow_data()?)?;v.amount=v.amount.checked_add_signed(delta).unwrap();
        Account::pack(v,&mut a[index].try_borrow_mut_data()?)?;
    }
    **a[2].try_borrow_mut_lamports()?+=output;**a[18].try_borrow_mut_lamports()?-=output;
    Ok(())
}
fn run(f:&mut Fixture,output_adjust:i64,fail:Option<usize>,mut corrupt:impl FnMut(usize,&[AccountInfo<'_>]))->Result<usize,ProgramError>{
    let clock=f.base.clock.clone();let rent=f.base.custody.rent.clone();let token_rent=rent.minimum_balance(165);let mut calls=0;
    let mut data=SELECTOR.to_vec();data.push(1);
    f.base.custody.with_infos(|a|execution::process_with_host_callbacks(&PROGRAM,a,&data,clock,rent,|ix,infos,seeds|{
        calls+=1;if fail==Some(calls){return Err(ProgramError::Custom(9001));}
        invoke(ix,infos,seeds,token_rent,output_adjust)?;corrupt(calls,a);Ok(())
    }))?;Ok(calls)
}
fn round(f:&Fixture)->ActiveDistribution{ActiveDistribution::deserialize(&mut &f.base.custody.accounts[1].data[8..]).unwrap()}

#[test]
fn atomic_model_receipt_exact_floor_mixed_funding_prefunds_and_rent_neutrality(){
    for (pending,carry,prefund,adjust)in [(0,0,0,0),(321,117,13,0),(321,117,0,100)]{
        let mut f=fixture(pending,carry,prefund);let before=f.clone();let c=before.base.custody.config();let op=before.base.custody.accounts[4].lamports;
        assert!(run(&mut f,adjust,None,|_,_|{}).unwrap()>10);
        let r=round(&f);r.validate().unwrap();let receipt=r.swap.unwrap();
        assert_eq!(r.lifecycle,DistributionLifecycle::EscrowFunded);assert_eq!(r.fixed_jitosol_withdrawal_target_units,0);
        assert_eq!(receipt.actual_native_output_lamports,receipt.minimum_native_output_lamports+adjust as u64);
        assert_eq!(f.base.custody.token_units(7),c.accounted_historical_jitosol_units-receipt.spent_units);
        assert_eq!(f.base.custody.accounts[4].lamports,op);
        assert_eq!(f.base.custody.config().accounted_pending_sol_lamports,pending+prefund);
        assert_eq!(f.base.custody.config().protected_principal_hwm_lamports,c.protected_principal_hwm_lamports);
        assert_eq!(f.base.custody.accounts[5].lamports-before.base.custody.accounts[5].lamports,r.pending_sol_used_lamports+r.prior_next_cycle_yield_used_lamports().unwrap()+receipt.actual_native_output_lamports);
        for i in [3,6,8,9,10,13,14,15,16,17,18]{
            if i!=3||carry==0{assert_eq!(f.base.custody.accounts[i],before.base.custody.accounts[i],"unchanged role{i}");}
        }
        let committed=f.clone();assert!(run(&mut f,0,None,|_,_|{}).is_err());assert_eq!(f,committed);
    }
}

#[test]
fn independent_floor_refuses_one_below_even_when_router_model_accepts(){
    let mut f=fixture(0,0,0);let before=f.clone();
    assert_eq!(run(&mut f,-1,None,|_,_|{}),Err(ProgramError::Custom(piv1::instruction_errors::piv1_error_code(Piv1Error::SwapMinimumNotMet))));
    assert_eq!(f.base.custody.accounts[0],before.base.custody.accounts[0]);
    assert_eq!(f.base.custody.accounts[1],before.base.custody.accounts[1]);
}

#[test]
fn every_cpi_failure_or_protected_mutation_keeps_state_uncommitted(){
    let baseline=fixture(321,117,13);let mut successful=baseline.clone();let count=run(&mut successful,0,None,|_,_|{}).unwrap();
    for fail in 1..=count{
        let mut f=baseline.clone();assert_eq!(run(&mut f,0,Some(fail),|_,_|{}),Err(ProgramError::Custom(9001)));
        assert_eq!(f.base.custody.accounts[0],baseline.base.custody.accounts[0]);assert_eq!(f.base.custody.accounts[1],baseline.base.custody.accounts[1]);
        // The explicit host world is discarded. Runtime rollback is tested in Bank.
    }
    for at in [1,count/2,count]{for target in [0,1,3,6,8,10,14,15]{
        let mut f=baseline.clone();assert!(run(&mut f,0,None,|n,a|if n==at{**a[target].try_borrow_mut_lamports().unwrap()+=1;}).is_err());
        assert_eq!(f.base.custody.accounts[0].data,baseline.base.custody.accounts[0].data);assert_eq!(f.base.custody.accounts[1].data,baseline.base.custody.accounts[1].data);
    }}
}

#[test]
fn substitutions_and_native_credit_cannot_bypass_receipt(){
    let baseline=fixture(0,0,0);let b=27;
    for (index,field) in [(b+4,0),(b+6,0),(b+3,0),(b+7,1),(b+8,1),(b+10,1),(b+11,1),(b+1,2)]{
        let mut f=baseline.clone();if field==0{f.base.custody.accounts[index].key=Pubkey::new_unique();}
        else if field==1{f.base.custody.accounts[index].owner=system_program::ID;}
        else{f.base.custody.accounts[index].signer=true;}
        let before=f.clone();assert!(run(&mut f,0,None,|_,_|{}).is_err());assert_eq!(f,before);
    }
    let mut f=baseline.clone();assert!(run(&mut f,-1,None,|_,a|{
        if a[b+2].owner==&spl_token::ID&&Account::unpack(&a[b+2].try_borrow_data().unwrap()).is_ok_and(|v|v.amount>0){
            **a[b+2].try_borrow_mut_lamports().unwrap()+=1;
        }
    }).is_err());
    assert_eq!(f.base.custody.accounts[0],baseline.base.custody.accounts[0]);
}

#[test]
fn versioned_codec_preserves_v1_and_rejects_illegal_v2_size_and_receipts(){
    assert_eq!(ActiveDistribution::SPACE,891);assert_eq!(ActiveDistribution::SWAP_SPACE,924);
    let f=fixture(0,0,0);let v2=round(&f);assert_eq!(v2.version,2);
    let mut v1=v2;v1.version=1;let old=StateEnvelope::distribution(&v1).unwrap();assert_eq!(old.as_bytes().len(),891);
    assert_eq!(ActiveDistribution::deserialize(&mut &old.as_bytes()[8..]).unwrap(),v1);
    let mut f=f;run(&mut f,0,None,|_,_|{}).unwrap();let r=round(&f);
    let mut corrupted=r;corrupted.swap.as_mut().unwrap().minimum_native_output_lamports-=1;assert!(corrupted.validate().is_err());
    corrupted=r;corrupted.swap.as_mut().unwrap().actual_native_output_lamports-=1;assert!(corrupted.validate().is_err());
    corrupted=r;corrupted.version=1;assert!(corrupted.validate().is_err());
}

#[test]
fn v2_pending_authentication_accepts_idle_and_active_without_reinterpreting_config(){
    let mut f=fixture(321,117,0);let rent=f.base.custody.rent.clone();
    let check=|f:&mut Fixture|f.base.custody.with_infos(|a|
        piv1::pending_accounts::authenticate_pending_accounts(&PROGRAM,&rent,piv1::pending_accounts::PendingAccountInfos{
            config:&a[0],active_distribution:&a[1],pending_sol:&a[2],pending_jito:&a[8]})
        .map(|authenticated|authenticated.distribution().version));
    assert_eq!(check(&mut f),Ok(2));
    run(&mut f,0,None,|_,_|{}).unwrap();assert_eq!(check(&mut f),Ok(2));
    let config=f.base.custody.accounts[0].clone();
    f.base.custody.accounts[1].data.truncate(891);
    assert_eq!(check(&mut f),Err(Piv1Error::InvalidAccountSize));
    assert_eq!(f.base.custody.accounts[0],config);
}

#[test]
fn v2_zero_shortfall_preserves_liquid_funding_without_spending_tokens_or_rent() {
    let mut f=fixture(20_000,0,0);let before=f.clone();
    run(&mut f,0,None,|_,_|{}).unwrap();let r=round(&f);
    assert_eq!(r.version,2);assert!(r.swap.is_none());assert_eq!(r.lifecycle,DistributionLifecycle::EscrowFunded);
    assert_eq!(r.recorded_escrow_available_lamports,r.outgoing_gross_obligation_lamports);
    for i in [4,7,27,28,29]{assert_eq!(f.base.custody.accounts[i],before.base.custody.accounts[i]);}
}

#[test]
fn swap_dispatch_does_not_intercept_existing_governed_pause_selector() {
    use piv1::{instruction_boundary,instructions::pause::SET_PAUSE_SELECTOR};
    assert_eq!(SELECTOR,*b"PIV1JS01");assert_ne!(SELECTOR,SET_PAUSE_SELECTOR);
    for paused in [0,1]{let mut data=SET_PAUSE_SELECTOR.to_vec();data.extend([1,7,paused]);
        assert_eq!(instruction_boundary::process_instruction(&PROGRAM,&[],&data),Err(ProgramError::NotEnoughAccountKeys));
    }
}
