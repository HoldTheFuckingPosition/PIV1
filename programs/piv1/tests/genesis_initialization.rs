#[allow(dead_code)]
#[path = "support/squads_invocation.rs"]
mod support;
#[path = "support/jito_identity_oracle.rs"]
mod oracle;

use anchor_lang::{
    prelude::{AccountInfo, Pubkey},
    AnchorDeserialize, AnchorSerialize,
    solana_program::{bpf_loader_upgradeable, entrypoint::ProgramResult, instruction::Instruction,
        program_error::ProgramError, program_option::COption, program_pack::Pack, system_program},
};
use piv1::{
    accounts::STAKE_PROGRAM_ID,
    genesis_allocation::GenesisAllocationRoles,
    genesis_initialization::*,
    accounts::{authenticate_fixed_accounts, FixedAccountInfos},
    state::{ActiveDistribution, GuardianRegistry, GuardianReward, PivConfig},
    genesis_model::prepare_approved_genesis_model_with_host_context,
    genesis_preflight::*,
    instructions::initialize::*,
    integrations::{self, jito_identity::*},
    squads_execution::{ModeledSquadsInvocationContext, SquadsBootstrapRoles},
};
use solana_stake_interface::state::{Authorized, Lockup, Meta, StakeStateV2};
use spl_token::state::{Account as TokenAccount, AccountState, Mint};
use support::{key, BackingAccount, Fixture, PROGRAM, PROGRAM_ACCOUNT, PROGRAM_DATA,
    MULTISIG, PROPOSAL, TRANSACTION, VAULT, INSTRUCTIONS, CONFIG};
use GenesisInitializationError as E;

// Reuses the established Task2.21 fixture/oracle construction without modifying
// its tests. The canonical Token Program is added to the exact approved list.
fn bootstrap() -> SquadsBootstrapRoles {
    SquadsBootstrapRoles { program: PROGRAM_ACCOUNT, program_data: PROGRAM_DATA,
        multisig: MULTISIG, proposal: PROPOSAL, transaction: TRANSACTION, vault: VAULT,
        instructions: INSTRUCTIONS, config: CONFIG }
}
fn context(f: &Fixture) -> ModeledSquadsInvocationContext {
    ModeledSquadsInvocationContext { stack_height: f.stack_height, clock: f.clock.clone(), rent: f.rent.clone() }
}
fn account(address: Pubkey, owner: Pubkey, data: Vec<u8>, executable: bool, writable: bool) -> BackingAccount {
    BackingAccount { key: address, owner, data, executable, writable, signer: false, lamports: 0 }
}
#[derive(Clone)]
struct World { f: Fixture, roles: GenesisPreflightRoles, parameters: GenesisModelParameters, payer: usize, system: usize, token: usize }
impl World {
    fn new(program: Pubkey, same_receiver: bool) -> Self {
        let mut f = Fixture::new(); f.accounts.truncate(8); f.program = program;
        f.accounts[PROGRAM_ACCOUNT].key = program;
        let pd = Pubkey::find_program_address(&[program.as_ref()], &bpf_loader_upgradeable::ID).0;
        f.accounts[PROGRAM_DATA].key = pd;
        f.accounts[PROGRAM_ACCOUNT].data[4..36].copy_from_slice(pd.as_ref());
        f.accounts[CONFIG] = account(Pubkey::find_program_address(&[b"config"], &program).0,
            system_program::ID, vec![], false, true);
        let parameters = GenesisModelParameters { vault_index: 7, initially_paused: false,
            protocol: DeclaredGenesisProtocol { stake_pool_program: JITO_STAKE_POOL_PROGRAM, stake_pool: JITO_STAKE_POOL,
                validator_list: key(201), reserve_stake: key(202), jitosol_mint: JITOSOL_MINT,
                manager_fee_account: key(203), referrer_token_account: if same_receiver { key(203) } else { key(204) } },
            htfp_recipient: key(91), team_owner_recipient: key(206), kif_anchor_timestamp: 0,
            guardian_slot_permutation: [5,4,3,2,1,0] };
        f.inner_data = parameters.encode().unwrap().to_vec(); f.rebuild_message();
        // Existing accepted model supplies fixture topology only. The new API
        // will reauthenticate the final complete approved message independently.
        let ctx = context(&f); let bytes = f.inner_data.clone();
        let model = f.with_infos(|a| prepare_approved_genesis_model_with_host_context(&program, a, &bytes, bootstrap(), ctx)).unwrap();
        let mut targets = [CONFIG;16];
        for (slot, target) in model.targets().iter().enumerate().skip(1) {
            targets[slot] = f.accounts.len();
            f.accounts.push(account(target.address(), system_program::ID, vec![], false, true));
        }
        let start = f.accounts.len(); let p = parameters.protocol;
        let (withdraw, bump) = Pubkey::find_program_address(&[p.stake_pool.as_ref(), b"withdraw"], &p.stake_pool_program);
        let mut program_bytes = 2_u32.to_le_bytes().to_vec();
        program_bytes.extend(Pubkey::find_program_address(&[p.stake_pool_program.as_ref()], &bpf_loader_upgradeable::ID).0.to_bytes());
        let mut pool = oracle::distinct_pool(0,0); pool.validator_list = p.validator_list; pool.reserve_stake = p.reserve_stake;
        pool.pool_mint = p.jitosol_mint; pool.manager_fee_account = p.manager_fee_account; pool.token_program_id = spl_token::ID;
        pool.stake_withdraw_bump_seed = bump; pool.lockup = Lockup::default();
        let mut pool_bytes = borsh1::to_vec(&pool).unwrap(); pool_bytes.resize(611, 0xA5);
        let mut list = vec![2]; list.extend(1_u32.to_le_bytes()); list.extend(0_u32.to_le_bytes()); list.resize(82, 0xA5);
        let mut reserve = borsh1::to_vec(&StakeStateV2::Initialized(Meta { rent_exempt_reserve: 123,
            authorized: Authorized { staker: withdraw, withdrawer: withdraw }, lockup: Lockup::default() })).unwrap();
        reserve.resize(200, 0xA5);
        let mut mint = vec![0;82]; Mint::pack(Mint { mint_authority: COption::Some(withdraw), supply: 9999,
            decimals: 9, is_initialized: true, freeze_authority: COption::None }, &mut mint).unwrap();
        let mut receiver = vec![0;165]; TokenAccount::pack(TokenAccount { mint: p.jitosol_mint, owner: key(207),
            amount: 99, delegate: COption::Some(key(208)), state: AccountState::Initialized, is_native: COption::None,
            delegated_amount: 1, close_authority: COption::Some(key(209)) }, &mut receiver).unwrap();
        f.accounts.extend([
            account(p.stake_pool_program, bpf_loader_upgradeable::ID, program_bytes, true, false),
            account(p.stake_pool, p.stake_pool_program, pool_bytes, false, false),
            account(p.validator_list, p.stake_pool_program, list, false, false),
            account(p.reserve_stake, STAKE_PROGRAM_ID, reserve, false, false),
            account(p.jitosol_mint, spl_token::ID, mint, false, false),
            account(p.manager_fee_account, spl_token::ID, receiver.clone(), false, false),
        ]);
        if !same_receiver { f.accounts.push(account(p.referrer_token_account, spl_token::ID, receiver, false, false)); }
        let protocol = GenesisProtocolRoles { program: start, pool: start+1, validator_list: start+2, reserve: start+3,
            mint: start+4, manager_fee: start+5, referrer: if same_receiver { start+5 } else { start+6 } };
        let payer = f.accounts.len();
        let mut funding = account(key(222), system_program::ID, vec![], false, true);
        funding.signer = true; funding.lamports = 1_000_000_000;
        f.accounts.push(funding);
        let system = f.accounts.len();
        f.accounts.push(account(system_program::ID, key(223), vec![], true, false));
        let token=f.accounts.len();
        f.accounts.push(account(spl_token::ID,key(224),vec![],true,false));
        f.rebuild_message(); Self { f, roles: GenesisPreflightRoles { bootstrap: bootstrap(), protocol, targets }, parameters, payer, system, token }
    }
    fn initialization_roles(&self) -> GenesisInitializationRoles {
        GenesisInitializationRoles { allocation: GenesisAllocationRoles { preflight: self.roles,
            payer: self.payer, system_program: self.system }, token_program: self.token }
    }
    fn raw(&mut self, behavior: Behavior) -> (GenesisInitializationResult<InitializedGenesisAccounts>, usize) {
        let before = self.clone(); let program = self.f.program; let data = self.f.inner_data.clone();
        let roles = self.initialization_roles(); let ctx = context(&self.f); let calls = expected_calls(&before);
        let mut count = 0;
        let (result, observed) = self.f.with_infos(|infos| {
            let mut held_write_borrow = None;
            let result = initialize_approved_genesis_accounts_with_host_invoker(&program, infos, &data, roles, ctx,
                |ix, actual, signers| {
                    let call = &calls[count]; let current = count; count += 1;
                    let result=emulate(&before, call, behavior, current, ix, actual, signers, infos);
                    if call.op==Op::Token && call.slot==15 {
                        if let Behavior::WriteBorrow(slot)=behavior {
                            held_write_borrow=Some(infos[roles.allocation.preflight.targets[slot]].try_borrow_data().unwrap());
                        }
                    }
                    result
                });
            // The modeled allocator replaces data slices using owned host buffers;
            // never call SDK resize/realloc on non-runtime memory layouts.
            drop(held_write_borrow);
            let observed = infos.iter().map(snapshot).collect::<Vec<_>>();
            (result, observed)
        });
        self.f.accounts = observed;
        (result, count)
    }
    /// Explicit clone/discard transaction MODEL, never evidence of SVM rollback.
    fn transaction(&mut self, behavior: Behavior) -> (GenesisInitializationResult<InitializedGenesisAccounts>, usize) {
        let mut staged = self.clone(); let outcome = staged.raw(behavior);
        if outcome.0.is_ok() { *self = staged; }
        outcome
    }
    fn reject_before(&mut self) {
        let before = self.f.clone(); let (result, calls) = self.raw(Behavior::Good);
        assert!(result.is_err(), "must reject malformed preparation");
        assert_eq!(calls, 0); assert_fixture(&self.f, &before);
    }
}

fn snapshot(a: &AccountInfo<'_>) -> BackingAccount {
    BackingAccount { key: *a.key, owner: *a.owner, executable: a.executable, signer: a.is_signer,
        writable: a.is_writable, lamports: amount(a), data: a.try_borrow_data().unwrap().to_vec() }
}
fn amount(a: &AccountInfo<'_>) -> u64 { **a.try_borrow_lamports().unwrap() }
fn assert_fixture(actual: &Fixture, before: &Fixture) {
    assert_eq!(actual.rent.exemption_threshold.to_bits(), before.rent.exemption_threshold.to_bits());
    let mut a = actual.clone(); let mut b = before.clone();
    a.rent.exemption_threshold = 0.0; b.rent.exemption_threshold = 0.0; assert_eq!(a, b);
}
fn sizes() -> [usize;16] { [1014,891,210,84,84,84,84,84,84,0,0,0,0,0,165,165] }
fn floor(slot: usize) -> u64 { (128 + sizes()[slot] as u64) * 3480 * 2 }

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum Op { Transfer(u64), Allocate(usize), Assign(Pubkey), Token }
#[derive(Clone, Debug)]
struct Call { slot: usize, op: Op }
fn expected_calls(w: &World) -> Vec<Call> {
    let mut result = vec![];
    for (slot, index) in w.roles.targets.iter().copied().enumerate() {
        let amount = floor(slot).saturating_sub(w.f.accounts[index].lamports);
        if amount != 0 { result.push(Call { slot, op: Op::Transfer(amount) }); }
        if sizes()[slot] != 0 {
            result.push(Call { slot, op: Op::Allocate(sizes()[slot]) });
            result.push(Call { slot, op: Op::Assign(if slot < 9 { w.f.program } else { spl_token::ID }) });
        }
    }
    result.push(Call {slot:14,op:Op::Token}); result.push(Call {slot:15,op:Op::Token});
    result
}
fn independent_seeds(w: &World, slot: usize) -> Vec<Vec<u8>> {
    let mut result = if (3..9).contains(&slot) {
        let guardian = key(96 - (slot as u8 - 3));
        vec![b"guardian-reward".to_vec(), guardian.to_bytes().to_vec(), 0_u64.to_le_bytes().to_vec(), vec![(slot - 3) as u8]]
    } else {
        vec![match slot {
            0 => b"config".as_slice(), 1 => b"distribution", 2 => b"guardian-registry",
            9 => b"pending-sol", 10 => b"principal-sol", 11 => b"operational-sol",
            12 => b"distribution-escrow", 13 => b"kif-sol", 14 => b"principal-jito-vault", 15 => b"pending-jito-vault",
            _ => panic!("invalid test slot"),
        }.to_vec()]
    };
    let refs = result.iter().map(Vec::as_slice).collect::<Vec<_>>();
    let (address, bump) = Pubkey::find_program_address(&refs, &w.f.program);
    assert_eq!(address, w.f.accounts[w.roles.targets[slot]].key); result.push(vec![bump]); result
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum Behavior {
    Good, ErrorBefore(usize), ErrorAfter(usize), NoOp(usize),
    TokenField(u8), TokenNativeDelta, StateTamper, NativeTamper, PayerCredit, MintTamper, PriorToken, WriteBorrow(usize),
}
#[allow(clippy::too_many_arguments)]
fn emulate(
    w: &World, call: &Call, behavior: Behavior, index: usize, ix: &Instruction,
    infos: &[AccountInfo<'_>], signers: &[&[&[u8]]], all: &[AccountInfo<'_>],
) -> ProgramResult {
    assert_eq!(ix.program_id, if call.op==Op::Token {spl_token::ID}else{system_program::ID});
    let target = w.f.accounts[w.roles.targets[call.slot]].key;
    let payer = w.f.accounts[w.payer].key;
    let (expected_bytes, expected_metas, expected_infos, expected_signers) = match call.op {
        Op::Token => {
            let authority=Pubkey::find_program_address(&[b"authority"],&w.f.program).0;
            let mint=w.parameters.protocol.jitosol_mint;
            let mut bytes=vec![18];bytes.extend(authority.to_bytes());
            (bytes,vec![(target,false,true),(mint,false,false)],vec![target,mint,spl_token::ID],vec![])
        }
        Op::Transfer(value) => {
            let mut bytes = 2_u32.to_le_bytes().to_vec(); bytes.extend(value.to_le_bytes());
            (bytes, vec![(payer,true,true),(target,false,true)], vec![payer,target,system_program::ID], vec![])
        }
        Op::Allocate(size) => {
            let mut bytes = 8_u32.to_le_bytes().to_vec(); bytes.extend((size as u64).to_le_bytes());
            (bytes, vec![(target,true,true)], vec![target,system_program::ID], vec![independent_seeds(w,call.slot)])
        }
        Op::Assign(owner) => {
            let mut bytes = 1_u32.to_le_bytes().to_vec(); bytes.extend(owner.to_bytes());
            (bytes, vec![(target,true,true)], vec![target,system_program::ID], vec![independent_seeds(w,call.slot)])
        }
    };
    assert_eq!(ix.data, expected_bytes);
    assert_eq!(ix.accounts.iter().map(|m| (m.pubkey,m.is_signer,m.is_writable)).collect::<Vec<_>>(), expected_metas);
    assert_eq!(infos.iter().map(|a| *a.key).collect::<Vec<_>>(), expected_infos);
    assert_eq!(signers.iter().map(|g| g.iter().map(|s| s.to_vec()).collect::<Vec<_>>()).collect::<Vec<_>>(), expected_signers);
    for group in signers { assert_eq!(Pubkey::create_program_address(group, &w.f.program).unwrap(), target); }
    for account in infos {
        assert!(account.try_borrow_mut_data().is_ok(), "no retained data guard");
        assert!(account.try_borrow_mut_lamports().is_ok(), "no retained lamport guard");
    }
    if behavior == Behavior::ErrorBefore(index) { return Err(ProgramError::Custom(22101)); }
    if behavior == Behavior::NoOp(index) { return Ok(()); }
    match call.op {
        Op::Transfer(value) => {
            let transferred = value;
            **infos[0].try_borrow_mut_lamports()? -= transferred;
            **infos[1].try_borrow_mut_lamports()? += transferred;

        }
        Op::Allocate(size) => {
            let value = 0;
            *infos[0].try_borrow_mut_data()? = Box::leak(vec![value;size].into_boxed_slice());
        }
        Op::Assign(owner) => infos[0].assign(&owner),
        Op::Token => {
            let mut bytes=independent_token(w);
            if let Behavior::TokenField(field)=behavior {
                match field {
                    0=>bytes[0]^=1,1=>bytes[32]^=1,2=>bytes[64]=1,3=>{bytes[72]=1;bytes[76]=1;},
                    4=>bytes[108]=2,5=>bytes[109]=1,6=>bytes[121]=1,7=>{bytes[129]=1;bytes[133]=1;},
                    _=>bytes[100]=1, // dirty None-option payload must also reject
                }
            }
            infos[0].try_borrow_mut_data()?.copy_from_slice(&bytes);
            match behavior {
                Behavior::TokenNativeDelta=>**infos[0].try_borrow_mut_lamports()?+=1,
                Behavior::StateTamper=>all[w.roles.targets[8]].try_borrow_mut_data()?[0]=1,
                Behavior::NativeTamper=>**all[w.roles.targets[13]].try_borrow_mut_lamports()?+=1,
                Behavior::PayerCredit=>**all[w.payer].try_borrow_mut_lamports()?+=1,
                Behavior::MintTamper=>infos[1].try_borrow_mut_data()?[36]^=1,
                Behavior::PriorToken if call.slot==15=>all[w.roles.targets[14]].try_borrow_mut_data()?[64]=1,
                _=>{},
            }
        }
    }
    if behavior == Behavior::ErrorAfter(index) { return Err(ProgramError::Custom(22102)); }
    Ok(())
}


fn independent_token(w: &World) -> [u8;165] {
    // Legacy Token account layout: all integer/COption bytes start at zero;
    // only mint, authority and Initialized state are written at genesis.
    let mut bytes=[0;165]; bytes[..32].copy_from_slice(w.parameters.protocol.jitosol_mint.as_ref());
    let authority=Pubkey::find_program_address(&[b"authority"],&w.f.program).0;
    bytes[32..64].copy_from_slice(authority.as_ref()); bytes[108]=1; bytes
}
fn envelope<T: AnchorSerialize>(value: &T, discriminator: [u8;8], space: usize) -> Vec<u8> {
    let mut bytes=discriminator.to_vec();bytes.extend(value.try_to_vec().unwrap());assert!(bytes.len()<=space);bytes.resize(space,0);bytes
}
fn decoded<T: AnchorDeserialize>(bytes: &[u8]) -> T {
    let mut payload=&bytes[8..]; let value=T::deserialize(&mut payload).unwrap(); assert!(payload.iter().all(|b|*b==0)); value
}

#[test]
fn complete_bytes_match_independent_envelopes_token_layout_and_all_prefunds() {
    for program in [PROGRAM,key(211)] { for same_receiver in [false,true] { for mode in 0..5 {
        let mut w=World::new(program,same_receiver);
        for (slot,index) in w.roles.targets.iter().copied().enumerate() {
            w.f.accounts[index].lamports=match mode {0=>0,1=>floor(slot)-1,2=>floor(slot),3=>floor(slot)+55,_=>u64::MAX};
        }
        let before=w.clone(); let ctx=context(&w.f); let bytes=w.f.inner_data.clone();
        let model=w.f.with_infos(|a|prepare_approved_genesis_model_with_host_context(&program,a,&bytes,bootstrap(),ctx)).unwrap();
        let mut expected=vec![
            envelope(model.proposed_config(),[98,115,11,164,170,207,163,20],1014),
            envelope(model.proposed_distribution(),[104,51,125,187,226,55,209,99],891),
            envelope(model.proposed_registry(),[72,14,254,2,76,233,97,92],210),
        ];
        expected.extend(model.proposed_rewards().iter().map(|r|envelope(r,[169,109,89,17,75,171,105,39],84)));
        let (result,calls)=w.raw(Behavior::Good);let result=result.unwrap();
        assert_eq!(calls,if mode<2 {40}else{24});assert_eq!(result.model(),&model);
        for (slot,index) in w.roles.targets.iter().copied().enumerate() {
            let a=&w.f.accounts[index]; assert_eq!(a.lamports,before.f.accounts[index].lamports.max(floor(slot)));
            assert_eq!(a.owner,if slot<9 {program}else if slot<14 {system_program::ID}else{spl_token::ID});
            if slot<9 {assert_eq!(a.data,expected[slot]);}
            else if slot<14 {assert!(a.data.is_empty());}
            else {assert_eq!(a.data,independent_token(&w));}
        }
        let actual_config: PivConfig=decoded(&w.f.accounts[w.roles.targets[0]].data);
        let actual_distribution: ActiveDistribution=decoded(&w.f.accounts[w.roles.targets[1]].data);
        let actual_registry: GuardianRegistry=decoded(&w.f.accounts[w.roles.targets[2]].data);
        assert_eq!(&actual_config,model.proposed_config());assert_eq!(&actual_distribution,model.proposed_distribution());
        assert_eq!(&actual_registry,model.proposed_registry());
        for slot in 0..6 {
            let reward:GuardianReward=decoded(&w.f.accounts[w.roles.targets[slot+3]].data);
            assert_eq!(&reward,&model.proposed_rewards()[slot]);actual_registry.validate_reward_binding(slot as u8,&reward).unwrap();
            assert_eq!((reward.claimable_lamports,reward.last_active_period),(0,None));
        }
        for (index,a) in before.f.accounts.iter().enumerate() {
            if index!=w.payer && !w.roles.targets.contains(&index) {assert_eq!(&w.f.accounts[index],a);}
        }
        assert_eq!(w.f.accounts[w.payer].lamports,result.allocation().payer_after());
        let rent=w.f.rent.clone();let t=w.roles.targets;
        let fixed=w.f.with_infos(|a|authenticate_fixed_accounts(&program,&rent,FixedAccountInfos {
            config:&a[t[0]],active_distribution:&a[t[1]],pending_sol:&a[t[9]],principal_sol:&a[t[10]],operational_sol:&a[t[11]],
            distribution_escrow:&a[t[12]],kif_sol:&a[t[13]],principal_jito:&a[t[14]],pending_jito:&a[t[15]],
        })).unwrap();
        if mode>=3 {assert_eq!(fixed.economic_observation(),Err(piv1::errors::Piv1Error::UnsupportedTokenNativeExcess));}
        else {assert!(fixed.economic_observation().is_ok());}
    } } }
}

#[test]
fn token_program_roles_metadata_and_bounds_reject_before_system_effects() {
    let base=World::new(PROGRAM,false);
    for index in 0..base.f.accounts.len()-1 {
        let mut w=base.clone();w.token=index;w.reject_before();
    }
    let mut w=base.clone();w.token=usize::MAX;w.reject_before();
    let mut w=base.clone();w.roles.protocol.mint=usize::MAX;w.reject_before();
    for mutation in 0..3 {
        let mut w=base.clone();
        match mutation {0=>w.f.accounts[w.token].key=key(230),1=>w.f.accounts[w.token].executable=false,
            _=>w.f.accounts[w.roles.protocol.mint].data.push(0)}
        w.f.rebuild_message();w.reject_before();
    }
}

#[test]
fn token_program_backing_aliases_and_mint_shared_borrow_reject_before_allocation() {
    let base=World::new(PROGRAM,false);
    for index in [CONFIG,VAULT,base.payer,base.system,base.roles.protocol.pool] { for data_alias in [false,true] {
        let mut w=base.clone();let ctx=context(&w.f);let bytes=w.f.inner_data.clone();let roles=w.initialization_roles();
        let before=w.f.clone();
        w.f.with_infos(|actual| {
            let mut a=actual.to_vec();
            if data_alias {a[roles.token_program].data=a[index].data.clone();}
            else {a[roles.token_program].lamports=a[index].lamports.clone();}
            let mut calls=0;
            let result=initialize_approved_genesis_accounts_with_host_invoker(&PROGRAM,&a,&bytes,roles,ctx,|_,_,_|{calls+=1;Ok(())});
            assert!(result.is_err());assert_eq!(calls,0);
        });assert_fixture(&w.f,&before);
    } }
    let mut w=base;let ctx=context(&w.f);let bytes=w.f.inner_data.clone();let roles=w.initialization_roles();
    let before=w.f.clone();
    w.f.with_infos(|a| {
        let _borrow=a[roles.allocation.preflight.protocol.mint].try_borrow_data().unwrap();let mut calls=0;
        let result=initialize_approved_genesis_accounts_with_host_invoker(&PROGRAM,a,&bytes,roles,ctx,|_,_,_|{calls+=1;Ok(())});
        assert_eq!(result,Err(E::State(piv1::errors::Piv1Error::AccountBorrowFailed)));assert_eq!(calls,0);
    });assert_fixture(&w.f,&before);
}

#[test]
fn same_approved_bytes_payer_outer_signature_and_context_are_required_fresh() {
    for mutation in 0..6 {
        let mut w=World::new(PROGRAM,false);
        let ctx=context(&w.f);let bytes=w.f.inner_data.clone();let roles=w.roles;
        w.f.with_infos(|a|preflight_approved_genesis_accounts_with_host_context(&PROGRAM,a,&bytes,roles,ctx)).unwrap();
        match mutation {
            0=>w.f.inner_data[16]^=1,
            1=>{w.f.proposal.approved.truncate(3);w.f.sync_proposal();},
            2=>w.f.accounts[w.payer].signer=false,
            3=>{let payer=w.f.accounts[w.payer].key;for m in &mut w.f.outer[0].accounts {if m.pubkey==payer {m.is_signer=false;}}w.f.sync_sysvar();},
            4=>w.f.stack_height=3,
            _=>w.f.clock.unix_timestamp=-1,
        }
        w.reject_before();
    }
}

#[test]
fn every_system_and_token_failure_propagates_with_explicit_modeled_rollback_only() {
    for boundary in 0..40 { for after in [false,true] {
        let mut w=World::new(PROGRAM,false);let before=w.f.clone();
        let behavior=if after {Behavior::ErrorAfter(boundary)}else{Behavior::ErrorBefore(boundary)};
        let expected=ProgramError::Custom(if after {22102}else{22101});
        let expected=if boundary<38 {E::Allocation(piv1::genesis_allocation::GenesisAllocationError::Invocation(expected))}else{E::Invocation(expected)};
        let (result,calls)=w.transaction(behavior);assert_eq!(result,Err(expected.clone()));assert_eq!(calls,boundary+1);assert_fixture(&w.f,&before);
        let (result,calls)=w.raw(behavior);assert_eq!(result,Err(expected));assert_eq!(calls,boundary+1);
        if after || boundary>0 {assert_ne!(w.f.accounts,before.accounts,"no implicit rollback");}
        for slot in 0..9 {assert!(w.f.accounts[w.roles.targets[slot]].data.iter().all(|b|*b==0),"Token failures precede every state write");}
    } }
}

#[test]
fn token_false_success_and_all_canonical_token_fields_are_checked() {
    for boundary in [38,39] {
        let mut w=World::new(PROGRAM,false);let before=w.f.clone();
        let (result,calls)=w.transaction(Behavior::NoOp(boundary));
        assert_eq!(result,Err(E::ObservationMismatch));assert_eq!(calls,boundary+1);assert_fixture(&w.f,&before);
    }
    for field in 0..9 {
        let mut w=World::new(PROGRAM,false);let before=w.f.clone();
        let (result,calls)=w.transaction(Behavior::TokenField(field));
        assert_eq!(result,Err(E::ObservationMismatch));assert_eq!(calls,39);assert_fixture(&w.f,&before);
    }
}

#[test]
fn token_cpi_cannot_change_native_payer_mint_or_unwritten_state() {
    for behavior in [Behavior::TokenNativeDelta,Behavior::StateTamper,Behavior::NativeTamper,Behavior::PayerCredit,Behavior::MintTamper,Behavior::PriorToken] {
        let mut w=World::new(PROGRAM,false);let before=w.f.clone();
        let (result,calls)=w.transaction(behavior);assert_eq!(result,Err(E::ObservationMismatch));
        assert_eq!(calls,if behavior==Behavior::PriorToken {40}else{39});assert_fixture(&w.f,&before);
    }
}

#[test]
fn every_initial_state_borrow_failure_preserves_all_nine_unwritten_envelopes() {
    for slot in 0..9 {
        let mut w=World::new(PROGRAM,false);let before=w.f.clone();
        let (result,calls)=w.transaction(Behavior::WriteBorrow(slot));
        assert_eq!(result,Err(E::State(piv1::errors::Piv1Error::AccountBorrowFailed)));assert_eq!(calls,40);assert_fixture(&w.f,&before);
        let (result,calls)=w.raw(Behavior::WriteBorrow(slot));assert!(result.is_err());assert_eq!(calls,40);
        for state in 0..9 {assert_eq!(w.f.accounts[w.roles.targets[state]].data,vec![0;sizes()[state]]);}
        for token in [14,15] {assert_eq!(w.f.accounts[w.roles.targets[token]].data,independent_token(&w));}
        assert!(w.f.accounts[w.payer].lamports<before.accounts[w.payer].lamports);
    }
}

#[test]
fn completed_genesis_cannot_replay_or_accept_a_detached_allocated_world() {
    let mut w=World::new(PROGRAM,false);let (result,_) =w.raw(Behavior::Good);assert!(result.is_ok());w.reject_before();
    let mut partial=World::new(PROGRAM,false);let (result,calls)=partial.raw(Behavior::ErrorBefore(38));
    assert!(result.is_err());assert_eq!(calls,39);partial.reject_before();
}

#[test]
fn host_runtime_guard_and_native_dispatch_remain_closed() {
    let w=World::new(PROGRAM,false);
    assert_eq!(initialize_approved_genesis_accounts(&PROGRAM,&[],&[],w.initialization_roles()),Err(E::HostRuntimeUnavailable));
    assert_eq!(piv1::instruction_boundary::process_instruction_with_host_callbacks(&PROGRAM,&[],&w.f.inner_data,
        ||panic!("no Rent for undispatched genesis"),|_,_,_|panic!("no CPI for undispatched genesis"),|_|panic!("no event")),
        Err(ProgramError::InvalidInstructionData));
}
