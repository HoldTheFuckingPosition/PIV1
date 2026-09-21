#[allow(dead_code)]
#[path = "../../../../programs/piv1/tests/support/squads_invocation.rs"]
mod fixture;
use anchor_lang::{prelude::{AccountInfo, Pubkey}, solana_program::{program_error::ProgramError, system_program}};
use fixture::{key, BackingAccount, Fixture, PROGRAM, PROPOSAL, TRANSACTION, VAULT};
use piv1::{instructions::initialize::{DeclaredGenesisProtocol,GenesisModelParameters},
    squads_accounts::SQUADS_V4_PROGRAM_ID, squads_execution::VAULT_TRANSACTION_EXECUTE_DISCRIMINATOR};
use piv1_genesis_preflight_caller::{prepare, process_instruction, PreparedCall, HOST_UNAVAILABLE};

fn world(shared: bool) -> Fixture {
    let mut f=Fixture::new(); f.accounts.truncate(8);
    for index in 8..if shared {31} else {32} {
        f.accounts.push(BackingAccount {key:key(120+index as u8),owner:system_program::ID,
            data:vec![],lamports:12345,executable:false,signer:false,writable:index<=22});
    }
    f.inner_data=GenesisModelParameters { vault_index:7,initially_paused:false,
        protocol:DeclaredGenesisProtocol {stake_pool_program:key(201),stake_pool:key(202),
            validator_list:key(203),reserve_stake:key(204),jitosol_mint:key(205),
            manager_fee_account:key(206),referrer_token_account:key(if shared {206} else {207})},
        htfp_recipient:key(208),team_owner_recipient:key(209),kif_anchor_timestamp:0,
        guardian_slot_permutation:[5,4,3,2,1,0]}.encode().unwrap().to_vec();
    f.rebuild_message(); f
}

fn run_with(f:&mut Fixture, program:Pubkey, alter:impl FnOnce(&mut [AccountInfo<'_>],&mut Vec<u8>))
    -> Result<PreparedCall,ProgramError> {
    let before=f.clone(); let ix=f.outer[0].clone();
    let mut backing=f.accounts.clone();
    backing.push(BackingAccount {key:key(91),owner:system_program::ID,data:vec![],lamports:12345,
        executable:false,signer:true,writable:false});
    let backing_before=backing.clone();
    let result={
        // One backing vector gives every AccountInfo the same lexical lifetime;
        // duplicate outer metas share the original data/lamport references.
        let infos:Vec<_>=backing.iter_mut().map(BackingAccount::info).collect();
        let mut outer:Vec<_>=ix.accounts.iter().map(|meta| {
            let mut account=infos.iter().find(|a| *a.key==meta.pubkey).unwrap().clone();
            assert_eq!(*account.key,meta.pubkey);account.is_signer=meta.is_signer;account.is_writable=meta.is_writable;account
        }).collect();
        let mut data=ix.data;alter(&mut outer,&mut data);prepare(&program,&outer,&data)
    };
    assert_eq!(backing,backing_before,"all invoked backing accounts, including executor, must be preserved");
    assert_eq!(*f,before,"preparation must preserve all fixture account bytes and metadata");result
}
fn run(f:&mut Fixture)->Result<PreparedCall,ProgramError> {run_with(f,SQUADS_V4_PROGRAM_ID,|_,_|{})}

#[test]
fn approved_message_recovers_exact_inner_privileges_data_order_and_seeds() {
    for shared in [false,true] {
        let mut f=world(shared);let call=run(&mut f).unwrap();
        assert_eq!(call.instruction.program_id,PROGRAM);assert_eq!(call.instruction.data,f.inner_data);
        assert_eq!(call.instruction.accounts.len(),if shared {31} else {32});
        for (index,meta) in call.instruction.accounts.iter().enumerate() {
            assert_eq!(meta.pubkey,f.accounts[index].key);
            assert_eq!(meta.is_signer,index==5);
            assert_eq!(meta.is_writable,(7..=22).contains(&index));
        }
        assert!(!call.instruction.accounts[PROPOSAL].is_writable,"outer writable proposal must downgrade in CPI");
        assert!(call.instruction.accounts[VAULT].is_signer,"signer is requested from stored message, not outer union");
        assert_eq!((call.multisig,call.vault_index),(f.accounts[2].key,7));
        assert_eq!(Pubkey::create_program_address(&[b"multisig",call.multisig.as_ref(),b"vault",
            &[call.vault_index],&[call.vault_bump]],&SQUADS_V4_PROGRAM_ID).unwrap(),f.accounts[VAULT].key);
    }
}

#[test]
fn truncated_oversized_or_trailing_transactions_never_prepare() {
    let original=world(false);
    for length in 0..original.accounts[TRANSACTION].data.len() {
        let mut f=original.clone();f.accounts[TRANSACTION].data.truncate(length);assert!(run(&mut f).is_err(),"length={length}");
    }
    for extra in [1,100,usize::from(u16::MAX)] {
        let mut f=original.clone();f.accounts[TRANSACTION].data.extend(vec![0;extra]);assert!(run(&mut f).is_err());
    }
}

#[test]
fn stored_message_extensions_counts_and_indices_fail_closed() {
    for change in 0..13 {
        let mut f=world(false);
        match change {
            0=>f.transaction.ephemeral.push(1),
            1=>f.transaction.message.num_signers=255,
            2=>f.transaction.message.writable_signers=255,
            3=>f.transaction.message.writable_non_signers=255,
            4=>f.transaction.message.keys[1]=f.transaction.message.keys[0],
            5=>f.transaction.message.instructions[0].program_id_index=255,
            6=>f.transaction.message.instructions[0].accounts[0]=255,
            7=>f.transaction.message.instructions[0].accounts[1]=f.transaction.message.instructions[0].accounts[0],
            8=>{let ix=f.transaction.message.instructions[0].clone();f.transaction.message.instructions.push(ix);},
            9=>f.transaction.message.lookups.push(fixture::Lookup {account_key:key(250),writable:vec![],readonly:vec![]}),
            10=>f.transaction.message.instructions[0].data.push(0),
            11=>f.transaction.message.instructions[0].data[9]^=1,
            _=>f.transaction.message.instructions[0].data[0]^=1,
        }
        f.sync_transaction();assert!(run(&mut f).is_err(),"change={change}");
    }
}

#[test]
fn raw_u32_lengths_reject_before_unbounded_indexing_or_allocation() {
    for shared in [false,true] {
        let original=world(shared);let count=if shared {31} else {32};
        // Independent Anchor/Borsh fixture: fixed header83, empty ephemeral vec4,
        // message flags3, then keys vec; each following vec has a u32 length.
        let instruction_count=94+32*count;
        let account_count=instruction_count+5;
        let data_count=account_count+4+count;
        let lookup_count=original.accounts[TRANSACTION].data.len()-4;
        for offset in [83,90,instruction_count,account_count,data_count,lookup_count] {
            let mut f=original.clone();f.accounts[TRANSACTION].data[offset..offset+4].copy_from_slice(&u32::MAX.to_le_bytes());
            assert!(run(&mut f).is_err(),"overflow offset={offset}");
        }
        for offset in [90,instruction_count,account_count,data_count] {
            let mut f=original.clone();f.accounts[TRANSACTION].data[offset..offset+4].copy_from_slice(&0_u32.to_le_bytes());
            assert!(run(&mut f).is_err(),"zero offset={offset}");
        }
    }
}

#[test]
fn independently_rebuilt_inner_privilege_escalations_and_missing_vault_signer_reject() {
    for role in [2,3,4] {
        for signer in [false,true] {
            let mut f=world(false);
            if signer {f.accounts[role].signer=true;} else {f.accounts[role].writable=true;}
            f.rebuild_message();assert!(run(&mut f).is_err(),"role={role}, signer={signer}");
        }
    }
    let mut f=world(false);f.accounts[VAULT].signer=false;f.rebuild_message();assert!(run(&mut f).is_err());
    assert_eq!(prepare(&SQUADS_V4_PROGRAM_ID,&[],&VAULT_TRANSACTION_EXECUTE_DISCRIMINATOR).err(),
        Some(ProgramError::InvalidArgument));
}

#[test]
fn canonical_bindings_and_outer_privilege_sources_are_required() {
    for change in 0..7 {
        let mut f=world(false);
        match change {
            0=>f.transaction.bump^=1,
            1=>f.transaction.vault_bump^=1,
            2=>f.transaction.multisig=key(240),
            3=>f.transaction.index+=1,
            4=>f.multisig.threshold=3,
            5=>f.accounts[TRANSACTION].owner=system_program::ID,
            _=>f.accounts[TRANSACTION].executable=true,
        }
        f.sync_multisig();f.sync_transaction();assert!(run(&mut f).is_err(),"change={change}");
    }
    for change in 0..7 {
        let mut f=world(false);let vault=f.accounts[VAULT].key;let target=f.accounts[7].key;
        let result=run_with(&mut f,SQUADS_V4_PROGRAM_ID,|a,data|match change {
            0=>data.push(0),1=>data[0]^=1,2=>a[1].is_writable=false,3=>a[3].is_signer=false,
            4=>{a.iter_mut().filter(|a|*a.key==vault).for_each(|a|a.is_signer=true);},
            5=>{a.iter_mut().filter(|a|*a.key==target).for_each(|a|a.is_writable=false);},
            _=>a.swap(4,5),
        });assert!(result.is_err(),"outer change={change}");
    }
    assert!(run_with(&mut world(false),key(240),|_,_|{}).is_err());
}

#[test]
fn ordinary_host_entrypoint_cannot_invoke_even_with_well_formed_execute_data() {
    assert_eq!(process_instruction(&SQUADS_V4_PROGRAM_ID,&[],&VAULT_TRANSACTION_EXECUTE_DISCRIMINATOR),
        Err(ProgramError::Custom(HOST_UNAVAILABLE)));
}
