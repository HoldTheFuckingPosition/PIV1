//! Host-modeled exact approval, allocation and immutable reward ownership.
//! Successful System callbacks and clone/discard are not runtime rollback proof.
#[allow(dead_code)] #[path = "support/squads_invocation.rs"] mod support;
use anchor_lang::{prelude::{AccountInfo, Pubkey}, AnchorDeserialize, AnchorSerialize,
    solana_program::{instruction::Instruction, program_error::ProgramError, system_program}};
use piv1::{events::GuardianSetUpdated, guardian_set_operations as handler, instruction_boundary,
    instructions::update_config::*, squads_execution::ModeledSquadsInvocationContext,
    state::{GuardianRegistry, GuardianReward, PivConfig}};
use support::{key, BackingAccount, Fixture, PROGRAM, CONFIG, REGISTRY};
fn env<T: AnchorSerialize>(value: &T, discriminator: [u8;8], size: usize) -> Vec<u8> {
    let mut b = discriminator.to_vec(); b.extend(value.try_to_vec().unwrap()); assert!(b.len() <= size); b.resize(size,0); b
}
fn decode<T: AnchorDeserialize>(b: &[u8]) -> T { T::deserialize(&mut &b[8..]).unwrap() }
fn amount(a: &AccountInfo<'_>) -> u64 { **a.try_borrow_lamports().unwrap() }
fn snapshot(a: &AccountInfo<'_>) -> BackingAccount { BackingAccount { key:*a.key,owner:*a.owner,
    signer:a.is_signer,writable:a.is_writable,executable:a.executable,lamports:amount(a),data:a.try_borrow_data().unwrap().to_vec() } }
#[derive(Clone, Debug, PartialEq)] struct World { f: Fixture, request: UpdateGuardianSetRequest }
impl World {
    fn new() -> Self {
        let mut f=Fixture::new();
        let registry:GuardianRegistry=decode(&f.accounts[REGISTRY].data);
        let mut config:PivConfig=decode(&f.accounts[CONFIG].data);
        config.kif_anchor_timestamp=0; config.kif_claim_liability_lamports=1000;
        config.cumulative_kif_credited_lamports=1300;config.cumulative_kif_claimed_lamports=300;
        f.accounts[CONFIG].data=env(&config,[98,115,11,164,170,207,163,20],PivConfig::SPACE);
        f.accounts[REGISTRY].writable=true;
        for i in 0..6 {
            let mut r:GuardianReward=decode(&f.accounts[9+i].data);
            r.last_active_period=if i==0||i==2 {Some(0)} else {None};
            r.claimable_lamports=10+i as u64;r.cumulative_earned=20+i as u64;r.cumulative_claimed=10;
            f.accounts[9+i].data=env(&r,[169,109,89,17,75,171,105,39],84);
        }
        let request=UpdateGuardianSetRequest{vault_index:7,guardian_index:2,expected_revision:registry.revision,new_guardian:key(97)};
        let mut next=registry;next.revision+=1;next.guardian_keys[2]=request.new_guardian;
        f.multisig.members.iter_mut().find(|m|m.key==registry.guardian_keys[2]).unwrap().key=request.new_guardian;
        f.multisig.members.sort_by_key(|m|m.key);f.sync_multisig();
        f.proposal.approved=f.multisig.members[..4].iter().map(|m|m.key).collect();f.sync_proposal();
        for i in 0..6 {
            let k=Pubkey::find_program_address(&[b"guardian-reward",next.guardian_keys[i].as_ref(),&next.revision.to_le_bytes(),&[i as u8]],&PROGRAM).0;
            f.accounts.push(BackingAccount{key:k,owner:system_program::ID,signer:false,writable:true,executable:false,
                lamports:if i==0 {f.rent.minimum_balance(84)+71} else {i as u64},data:vec![]});
        }
        f.accounts.push(BackingAccount{key:key(230),owner:system_program::ID,signer:true,writable:true,executable:false,lamports:100_000_000,data:vec![]});
        f.accounts.push(BackingAccount{key:system_program::ID,owner:key(231),signer:false,writable:false,executable:true,lamports:1,data:vec![]});
        f.inner_data=request.encode().to_vec();f.rebuild_message();Self{f,request}
    }
    fn approve(&mut self){self.f.inner_data=self.request.encode().to_vec();self.f.rebuild_message();}
    fn context(&self)->ModeledSquadsInvocationContext{ModeledSquadsInvocationContext{stack_height:self.f.stack_height,clock:self.f.clock.clone(),rent:self.f.rent.clone()}}
    fn raw(&mut self)->(Result<(),ProgramError>,usize,Vec<GuardianSetUpdated>){
        let before=self.clone();let context=self.context();let data=self.request.encode();let mut calls=0;let mut events=vec![];
        let (result,after)=self.f.with_infos(|a|{
            let result=handler::process_instruction_with_host_callbacks(&PROGRAM,a,&data,||Ok(context),|ix,infos,seeds|{
                emulate(&before,ix,infos,seeds);calls+=1;Ok(())
            },|e|events.push(e));(result,a.iter().map(snapshot).collect())});
        self.f.accounts=after;(result,calls,events)
    }
    fn reject(&mut self){let before=self.clone();let(r,c,e)=self.raw();assert!(r.is_err());assert_eq!(c,0);assert!(e.is_empty());assert_eq!(*self,before);}
    fn success(&mut self){
        let before=self.clone();let mut expected=before.f.accounts.clone();
        let mut config:PivConfig=decode(&expected[CONFIG].data);
        let mut registry:GuardianRegistry=decode(&expected[REGISTRY].data);
        let prior=registry.guardian_keys[2];registry.revision+=1;registry.guardian_keys[2]=self.request.new_guardian;
        config.guardian_registry_revision=registry.revision;
        expected[CONFIG].data=env(&config,[98,115,11,164,170,207,163,20],1014);
        expected[REGISTRY].data=env(&registry,[71,237,191,99,80,62,198,163],210);
        // Registry discriminator is independently taken from its unchanged
        // original envelope; only its serialized membership/revision changes.
        expected[REGISTRY].data[..8].copy_from_slice(&before.f.accounts[REGISTRY].data[..8]);
        let mut total=0;let mut transfers=0;
        for i in 0..6{
            let floor=self.f.rent.minimum_balance(84);let old=expected[16+i].lamports;
            let short=if old<floor {floor-old}else{0};total+=short;if short>0{transfers+=1;}
            expected[16+i].lamports+=short;expected[16+i].owner=PROGRAM;
            let bump=Pubkey::find_program_address(&[b"guardian-reward",registry.guardian_keys[i].as_ref(),&registry.revision.to_le_bytes(),&[i as u8]],&PROGRAM).1;
            let old_reward:GuardianReward=decode(&before.f.accounts[9+i].data);
            let reward=GuardianReward{version:1,bump,guardian_index:i as u8,registry_revision:registry.revision,
                guardian:registry.guardian_keys[i],last_active_period:if i==2{None}else{old_reward.last_active_period},
                claimable_lamports:0,cumulative_earned:0,cumulative_claimed:0};
            expected[16+i].data=env(&reward,[169,109,89,17,75,171,105,39],84);
        }
        expected[22].lamports-=total;
        let(r,c,e)=self.raw();assert_eq!(r,Ok(()));assert_eq!(c,12+transfers);
        assert_eq!(e,vec![GuardianSetUpdated{config:expected[CONFIG].key,guardian_registry:expected[REGISTRY].key,
            multisig:expected[2].key,transaction_index:self.f.transaction.index,previous_revision:self.request.expected_revision,
            registry_revision:registry.revision,guardian_index:2,previous_guardian:prior,new_guardian:self.request.new_guardian,
            rent_payer:expected[22].key,funded_rent_lamports:total}]);assert_eq!(self.f.accounts,expected);
    }
}
fn emulate(w:&World,ix:&Instruction,a:&[AccountInfo<'_>],seeds:&[&[&[u8]]]){
    assert_eq!(ix.program_id,system_program::ID);assert_eq!(*a.last().unwrap().key,system_program::ID);
    let opcode=u32::from_le_bytes(ix.data[..4].try_into().unwrap());
    if opcode==2{
        assert_eq!(a.len(),3);assert!(seeds.is_empty());assert_eq!(*a[0].key,w.f.accounts[22].key);
        let value=u64::from_le_bytes(ix.data[4..].try_into().unwrap());
        assert_eq!(ix.accounts,vec![anchor_lang::solana_program::instruction::AccountMeta::new(*a[0].key,true),anchor_lang::solana_program::instruction::AccountMeta::new(*a[1].key,false)]);
        let short=w.f.rent.minimum_balance(84).checked_sub(amount(&a[1])).unwrap();assert_eq!(value,short);
        **a[0].try_borrow_mut_lamports().unwrap()-=value;**a[1].try_borrow_mut_lamports().unwrap()+=value;
    }else{
        assert_eq!(a.len(),2);let i=w.f.accounts[16..22].iter().position(|x|x.key==*a[0].key).unwrap();
        let old:GuardianRegistry=decode(&w.f.accounts[REGISTRY].data);let key=if i==2{w.request.new_guardian}else{old.guardian_keys[i]};
        let revision=(old.revision+1).to_le_bytes();let index=[i as u8];
        let bump=[Pubkey::find_program_address(&[b"guardian-reward",key.as_ref(),&revision,&index],&PROGRAM).1];
        assert_eq!(seeds, &[&[b"guardian-reward".as_slice(),key.as_ref(),&revision,&index,&bump][..]]);
        assert_eq!(Pubkey::create_program_address(seeds[0],&PROGRAM).unwrap(),*a[0].key);
        assert_eq!(ix.accounts,vec![anchor_lang::solana_program::instruction::AccountMeta::new(*a[0].key,true)]);
        if opcode==8{assert_eq!(u64::from_le_bytes(ix.data[4..].try_into().unwrap()),84);
            *a[0].try_borrow_mut_data().unwrap()=Box::leak(vec![0;84].into_boxed_slice());
        }else{assert_eq!(opcode,1);assert_eq!(&ix.data[4..],PROGRAM.as_ref());a[0].assign(&PROGRAM);}
    }
}

#[test]
fn one_seat_rotation_preserves_every_old_record_and_all_non_revision_config_fields(){
    for paused in [false,true]{let mut w=World::new();let mut c:PivConfig=decode(&w.f.accounts[CONFIG].data);c.paused=paused;
        w.f.accounts[CONFIG].data=env(&c,[98,115,11,164,170,207,163,20],1014);w.success();
        // Replaying retained synthetic Approved cannot advance again.
        w.reject();}
}
#[test]
fn strict_abi_and_account_counts_and_native_host_routes_are_closed(){
    let mut w=World::new();let data=w.request.encode();assert_eq!(&data[..8],b"PIV1UG01");
    w.f.with_infos(|a|{
        assert_eq!(instruction_boundary::process_instruction(&PROGRAM,a,&data),Err(ProgramError::Custom(6999)));
        for n in 0..51{assert!(UpdateGuardianSetRequest::decode(&data[..n]).is_err());}
        let mut extra=data.to_vec();extra.push(0);assert!(UpdateGuardianSetRequest::decode(&extra).is_err());
        for offset in [0,7,8,10]{let mut bad=data;bad[offset]=255;assert!(UpdateGuardianSetRequest::decode(&bad).is_err());}
        for n in 0..24{assert_eq!(handler::process_instruction_with_host_callbacks(&PROGRAM,&a[..n],&data,
            ||panic!(),|_,_,_|panic!(),|_|panic!()),Err(ProgramError::NotEnoughAccountKeys));}
        let mut extra=a.to_vec();extra.push(a[0].clone());assert_eq!(handler::process_instruction_with_host_callbacks(&PROGRAM,&extra,&data,
            ||panic!(),|_,_,_|panic!(),|_|panic!()),Err(ProgramError::InvalidArgument));
    });
}
#[test]
fn old_new_mismatch_is_allowed_only_for_one_current_approved_replacement(){
    for mutation in 0..15{let mut w=World::new();match mutation{
        0=>{w.request.expected_revision+=1;w.approve();},
        1=>{w.request.new_guardian=key(95);w.approve();},
        2=>{w.request.new_guardian=key(98);w.approve();},
        3=>{w.f.multisig.members[0].key=key(90);w.f.sync_multisig();},
        4=>{w.f.proposal.approved.truncate(3);w.f.sync_proposal();},
        5=>{w.f.proposal.status=4;w.f.sync_proposal();},
        6=>{w.f.multisig.stale_transaction_index=w.f.transaction.index;w.f.sync_multisig();},
        7=>{w.f.multisig.threshold=3;w.f.sync_multisig();},
        8=>{w.f.multisig.config_authority=key(211);w.f.sync_multisig();},
        9=>{w.f.stack_height=1;},
        10=>{w.f.proposal.timestamp=91;w.f.sync_proposal();},
        11=>{w.f.accounts[5].signer=false;w.approve();},
        12=>{w.request.guardian_index=1;w.approve();},
        13=>{w.f.proposal.approved[0]=key(94);w.f.proposal.approved.sort();w.f.sync_proposal();},
        _=>{w.request.new_guardian=key(98);},
    }w.reject();}
    let mut w=World::new();assert!(w.f.run().is_err(),"ordinary approval still rejects mismatched current membership");
}
#[test]
fn targets_payer_shapes_rents_and_approved_role_privileges_fail_before_cpi(){
    for mutation in 0..15{let mut w=World::new();match mutation{
        0=>w.f.accounts[16].key=key(220),1=>w.f.accounts[16].owner=PROGRAM,
        2=>w.f.accounts[16].data=vec![0],3=>w.f.accounts[16].executable=true,
        4=>w.f.accounts[22].signer=false,5=>w.f.accounts[22].owner=PROGRAM,
        6=>w.f.accounts[22].data=vec![0],7=>w.f.accounts[22].lamports=0,
        8=>w.f.accounts[23].executable=false,9=>w.f.accounts[23].key=key(219),
        10=>w.f.accounts[22].executable=true,
        11=>{let c:PivConfig=decode(&w.f.accounts[CONFIG].data);w.f.accounts[22].key=c.kif_sol_vault;},
        12=>w.f.accounts[8].writable=false,13=>w.f.accounts[17].writable=false,
        _=>w.f.accounts[22].writable=false,
    }w.approve();w.reject();}
}
#[test]
fn account_backing_aliases_and_mutable_borrow_conflicts_reject_before_cpi(){
    let mut w=World::new();let data=w.request.encode();let before=w.clone();
    for pair in [(7,8),(16,17),(16,22),(9,16)]{for lamports in [false,true]{
        let context=w.context();w.f.with_infos(|a|{let mut b=a.to_vec();
            if lamports{b[pair.1].lamports=a[pair.0].lamports.clone()}else{b[pair.1].data=a[pair.0].data.clone()}
            assert!(handler::process_instruction_with_host_callbacks(&PROGRAM,&b,&data,||Ok(context),|_,_,_|panic!(),|_|panic!()).is_err());});}}
    for i in [7,8,16,17,18,19,20,21,22]{let context=w.context();w.f.with_infos(|a|{let _held=a[i].try_borrow_data().unwrap();
        assert!(handler::process_instruction_with_host_callbacks(&PROGRAM,a,&data,||Ok(context),|_,_,_|panic!(),|_|panic!()).is_err());});}
    assert_eq!(w,before);
}

#[test]
fn next_activity_snapshot_keeps_only_unchanged_keys_and_retired_owner_can_claim(){
    use piv1::{guardian_clock_accounts::{authenticate_guardian_clock_snapshot,GuardianClockAccountInfos},
        kif_claim_accounts::KifClaimAccountInfos,kif_claim_execution::{execute_kif_claim_with_host_invoker,KifClaimExecutionAccounts},
        state::KifClaimRequest};
    let mut w=World::new();let old:GuardianRegistry=decode(&w.f.accounts[REGISTRY].data);
    w.success();
    let mut current=w.f.accounts.clone();let config:PivConfig=decode(&current[CONFIG].data);
    let rent=w.f.rent.clone();let old_reward:GuardianReward=decode(&current[11].data);
    let infos:Vec<_>=current.iter_mut().map(BackingAccount::info).collect();
    let snapshot=authenticate_guardian_clock_snapshot(&PROGRAM,&rent,GuardianClockAccountInfos{
        config:&infos[CONFIG],guardian_registry:&infos[REGISTRY],rewards:core::array::from_fn(|i|&infos[16+i]),clock:&infos[15]}).unwrap();
    assert_eq!(snapshot.activity_bitmap(),1);assert_eq!(snapshot.registry().revision,old.revision+1);
    assert_eq!(snapshot.rewards()[2].last_active_period,None);drop(infos);
    // Extend this same host account world with the isolated KIF source/destination.
    // Their balances are explicitly modeled inputs, not proof of live funding.
    let mut claim_accounts=vec![w.f.accounts[CONFIG].clone(),w.f.accounts[11].clone(),
        BackingAccount{key:config.kif_sol_vault,owner:system_program::ID,signer:false,writable:true,executable:false,
            lamports:rent.minimum_balance(0)+config.kif_claim_liability_lamports+config.collective_kif_carry_lamports,data:vec![]},
        BackingAccount{key:old_reward.guardian,owner:system_program::ID,signer:true,writable:true,executable:false,lamports:rent.minimum_balance(0),data:vec![]},
        w.f.accounts[23].clone()];claim_accounts[1].writable=true;
    let before=claim_accounts.clone();let infos:Vec<_>=claim_accounts.iter_mut().map(BackingAccount::info).collect();
    let mut calls=0;
    let result=execute_kif_claim_with_host_invoker(&PROGRAM,&rent,KifClaimExecutionAccounts{claim:KifClaimAccountInfos{
        config:&infos[0],guardian_reward:&infos[1],kif_sol:&infos[2],guardian:&infos[3]},system_program:&infos[4]},
        KifClaimRequest{amount_lamports:old_reward.claimable_lamports,expected_cumulative_claimed:old_reward.cumulative_claimed},
        |ix,a,_|{calls+=1;assert_eq!(ix.program_id,system_program::ID);assert_eq!(ix.accounts[1].pubkey,old_reward.guardian);
            let value=u64::from_le_bytes(ix.data[4..].try_into().unwrap());assert_eq!(value,12);
            **a[0].try_borrow_mut_lamports()?-=value;**a[1].try_borrow_mut_lamports()?+=value;Ok(())});
    assert!(result.is_ok());assert_eq!(calls,1);drop(infos);
    let claimed:GuardianReward=decode(&claim_accounts[1].data);assert_eq!(claimed.guardian,old.guardian_keys[2]);
    assert_eq!(claimed.claimable_lamports,0);assert_eq!(claimed.cumulative_claimed,22);
    let after:PivConfig=decode(&claim_accounts[0].data);assert_eq!(after.kif_claim_liability_lamports,988);
    assert_eq!(after.guardian_registry_revision,old.revision+1);
    assert_eq!(claim_accounts[3].lamports,before[3].lamports+12);
    assert_eq!(decode::<GuardianReward>(&w.f.accounts[18].data).claimable_lamports,0);
}

#[test]
fn payer_can_be_new_guardian_and_external_recipient_overlap_remains_supported(){
    for recipient in [false,true]{let mut w=World::new();
        if recipient{let mut c:PivConfig=decode(&w.f.accounts[CONFIG].data);c.htfp_recipient=w.request.new_guardian;
            w.f.accounts[CONFIG].data=env(&c,[98,115,11,164,170,207,163,20],1014);
        }else{w.f.accounts[22].key=w.request.new_guardian;w.approve();}w.success();}
}

#[test]
fn future_snapshot_and_prior_period_activity_and_empty_or_funded_targets_are_preserved(){
    for prefund in [0,1,10_000_000]{let mut w=World::new();for i in 16..22{w.f.accounts[i].lamports=prefund;}
        // Same-key prior activity is historical evidence, not current eligibility.
        w.f.clock.unix_timestamp=2_592_100;use anchor_lang::prelude::SolanaSysvar;
        w.f.clock.to_account_info(&mut w.f.accounts[15].info()).unwrap();w.success();
        let r:GuardianReward=decode(&w.f.accounts[16].data);assert_eq!(r.last_active_period,Some(0));
        assert!(!r.is_active_in(1));}
}

#[test]
fn unsupported_additional_membership_changes_or_reordered_witnesses_do_not_mutate(){
    let mut w=World::new();w.f.multisig.members[0].key=key(90);w.f.sync_multisig();w.reject();
    let mut w=World::new();w.request.new_guardian=key(95);w.approve();w.reject();
    let mut w=World::new();let c:PivConfig=decode(&w.f.accounts[CONFIG].data);
    w.request.new_guardian=c.piv_authority;
    w.f.multisig.members.iter_mut().find(|m|m.key==key(97)).unwrap().key=w.request.new_guardian;
    w.f.multisig.members.sort_by_key(|m|m.key);w.f.sync_multisig();
    w.f.proposal.approved=w.f.multisig.members[..4].iter().map(|m|m.key).collect();w.f.sync_proposal();w.approve();w.reject();
}

#[test]
fn maximum_revision_rejects_checked_increment_before_allocation(){
    let mut w=World::new();w.request.expected_revision=u64::MAX;
    let mut c:PivConfig=decode(&w.f.accounts[CONFIG].data);c.guardian_registry_revision=u64::MAX;
    w.f.accounts[CONFIG].data=env(&c,[98,115,11,164,170,207,163,20],1014);
    let mut registry:GuardianRegistry=decode(&w.f.accounts[REGISTRY].data);registry.revision=u64::MAX;
    let disc=w.f.accounts[REGISTRY].data[..8].try_into().unwrap();w.f.accounts[REGISTRY].data=env(&registry,disc,210);
    for i in 0..6{let mut reward:GuardianReward=decode(&w.f.accounts[9+i].data);reward.registry_revision=u64::MAX;
        let (address,bump)=Pubkey::find_program_address(&[b"guardian-reward",reward.guardian.as_ref(),&u64::MAX.to_le_bytes(),&[i as u8]],&PROGRAM);
        reward.bump=bump;w.f.accounts[9+i].key=address;w.f.accounts[9+i].data=env(&reward,[169,109,89,17,75,171,105,39],84);}
    w.approve();let before=w.clone();let(r,c,e)=w.raw();
    assert_eq!(r,Err(ProgramError::Custom(piv1::instruction_errors::piv1_error_code(piv1::errors::Piv1Error::ArithmeticOverflow))));
    assert_eq!(c,0);assert!(e.is_empty());assert_eq!(w,before);
}

#[test]
fn exact_approved_message_and_every_fixed_writable_role_are_enforced(){
    let mut w=World::new();w.f.transaction.message.instructions[0].data[18]^=1;w.f.sync_transaction();w.reject();
    for role in 0..24{let mut w=World::new();w.f.accounts[role].writable=!w.f.accounts[role].writable;w.approve();w.reject();}
}

#[test]
fn readonly_executable_and_governance_padding_is_not_copied_by_effect_tracking(){
    let mut w=World::new();
    for role in [1,2,3]{let size=w.f.accounts[role].data.len()+65_536;w.f.accounts[role].data.resize(size,0);}
    w.success();
}

#[test]
fn successful_noop_system_callback_is_rejected_by_fresh_effect_observation(){
    // The callback returns success without the promised effect. It does not
    // mutate then fail internally; no transaction-runtime proof is asserted.
    for skip in 0..3{let mut w=World::new();let before=w.clone();let context=w.context();let data=w.request.encode();let mut calls=0;
        w.f.with_infos(|a|{let r=handler::process_instruction_with_host_callbacks(&PROGRAM,a,&data,||Ok(context),|ix,infos,seeds|{
            let n=calls;calls+=1;if n!=skip{emulate(&before,ix,infos,seeds);}Ok(())
        },|_|panic!("no event"));assert!(r.is_err());
            for i in 7..15{assert_eq!(snapshot(&a[i]),before.f.accounts[i]);}
        });assert_eq!(calls,skip+1);
    }
}
