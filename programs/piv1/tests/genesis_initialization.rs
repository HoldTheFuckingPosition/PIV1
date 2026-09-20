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
struct World { f: Fixture, roles: GenesisPreflightRoles, parameters: GenesisModelParameters, payer: usize, system: usize, token: usize, normalize: bool }
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
        f.rebuild_message(); Self { f, roles: GenesisPreflightRoles { bootstrap: bootstrap(), protocol, targets }, parameters, payer, system, token, normalize: false }
    }
    fn normalized(program: Pubkey, same_receiver: bool, paused: bool) -> Self {
        let mut w = Self::new(program, same_receiver); w.normalize = true;
        w.parameters.initially_paused = paused;
        w.f.inner_data = w.parameters.encode().unwrap().to_vec(); w.f.rebuild_message(); w
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
            let result = initialize_with_mode(before.normalize, &program, infos, &data, roles, ctx,
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

fn initialize_with_mode<'info>(normalize: bool, program: &Pubkey, accounts: &[AccountInfo<'info>],
    bytes: &[u8], roles: GenesisInitializationRoles, ctx: ModeledSquadsInvocationContext,
    invoke: impl FnMut(&Instruction, &[AccountInfo<'info>], &[&[&[u8]]]) -> ProgramResult,
) -> GenesisInitializationResult<InitializedGenesisAccounts> {
    if normalize {
        initialize_approved_genesis_accounts_normalizing_token_prefunds_with_host_invoker(program, accounts, bytes, roles, ctx, invoke)
    } else { initialize_approved_genesis_accounts_with_host_invoker(program, accounts, bytes, roles, ctx, invoke) }
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
enum Op { Transfer(u64), Sweep(u64), Allocate(usize), Assign(Pubkey), Token }
#[derive(Clone, Debug)]
struct Call { slot: usize, op: Op }
fn expected_calls(w: &World) -> Vec<Call> {
    let mut result = vec![];
    for (slot, index) in w.roles.targets.iter().copied().enumerate() {
        let amount = floor(slot).saturating_sub(w.f.accounts[index].lamports);
        if amount != 0 { result.push(Call { slot, op: Op::Transfer(amount) }); }
        if w.normalize && slot >= 14 {
            let excess = w.f.accounts[index].lamports.saturating_sub(floor(slot));
            if excess != 0 { result.push(Call { slot, op: Op::Sweep(excess) }); }
        }
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
    TokenField(u8), TokenNativeDelta, StateTamper, NativeTamper, PayerCredit, MintTamper, PriorToken, WriteBorrow(usize), SweepTamper(u8),
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
        Op::Sweep(value) => {
            let pending = w.f.accounts[w.roles.targets[9]].key;
            assert_eq!(*infos[0].owner, system_program::ID);
            assert!(infos[0].data_is_empty());
            assert_eq!(amount(infos.first().unwrap()), floor(call.slot) + value);
            assert!(amount(&infos[1]) >= w.f.accounts[w.roles.targets[9]].lamports.max(floor(9)),
                "original PendingSol rent is funded before contributions arrive");
            let mut bytes = 2_u32.to_le_bytes().to_vec(); bytes.extend(value.to_le_bytes());
            (bytes, vec![(target,true,true),(pending,false,true)], vec![target,pending,system_program::ID], vec![independent_seeds(w,call.slot)])
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
        Op::Transfer(value) | Op::Sweep(value) => {
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
    if matches!(call.op, Op::Sweep(_)) {
        if let Behavior::SweepTamper(field) = behavior {
            match field {
                0 => **infos[0].try_borrow_mut_lamports()? += 1,
                1 => **infos[1].try_borrow_mut_lamports()? -= 1,
                2 => **all[w.payer].try_borrow_mut_lamports()? += 1,
                3 => **all[w.roles.targets[11]].try_borrow_mut_lamports()? += 1,
                4 => **all[w.roles.targets[15]].try_borrow_mut_lamports()? += 1,
                5 => infos[0].assign(&spl_token::ID),
                _ => all[w.roles.targets[0]].try_borrow_mut_data()?[0] = 1,
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
    assert_eq!(initialize_approved_genesis_accounts_normalizing_token_prefunds(&PROGRAM,&[],&[],w.initialization_roles()),Err(E::HostRuntimeUnavailable));
    assert_eq!(piv1::instruction_boundary::process_instruction_with_host_callbacks(&PROGRAM,&[],&w.f.inner_data,
        ||panic!("no Rent for undispatched genesis"),|_,_,_|panic!("no CPI for undispatched genesis"),|_|panic!("no event")),
        Err(ProgramError::InvalidInstructionData));
}

fn fixed_observation(w: &mut World) -> Result<piv1::state::reconciliation::EconomicCustodyObservation, piv1::errors::Piv1Error> {
    let program=w.f.program; let rent=w.f.rent.clone(); let t=w.roles.targets;
    w.f.with_infos(|a| authenticate_fixed_accounts(&program,&rent,FixedAccountInfos {
        config:&a[t[0]],active_distribution:&a[t[1]],pending_sol:&a[t[9]],principal_sol:&a[t[10]],operational_sol:&a[t[11]],
        distribution_escrow:&a[t[12]],kif_sol:&a[t[13]],principal_jito:&a[t[14]],pending_jito:&a[t[15]],
    })?.economic_observation())
}

fn recognize_pending(w: &mut World) -> piv1::state::PendingReconciliationResult {
    let program=w.f.program; let rent=w.f.rent.clone(); let t=w.roles.targets;
    w.f.with_infos(|a| piv1::pending_reconciliation::execute_pending_reconciliation(&program,&rent,
        piv1::pending_accounts::PendingAccountInfos {
            config:&a[t[0]],active_distribution:&a[t[1]],pending_sol:&a[t[9]],pending_jito:&a[t[15]],
        })).unwrap()
}

#[test]
fn normalized_genesis_preserves_original_rent_and_recognizes_full_custody_once() {
    for program in [PROGRAM,key(211)] { for shared in [false,true] { for paused in [false,true] { for mode in 0..6 {
        let mut w=World::normalized(program,shared,paused); let t=w.roles.targets;
        for (slot,index) in t.iter().copied().enumerate() {
            w.f.accounts[index].lamports=match mode {
                0=>0, 1=>floor(slot)-1, 2=>floor(slot), _=>floor(slot)+55,
            };
        }
        if mode==4 {
            // A preexisting partial PendingSol rent deposit cannot consume either
            // Token contribution. Unrelated operational/state prefunds are untouched.
            w.f.accounts[t[9]].lamports=floor(9)-1;
            w.f.accounts[t[7]].lamports=u64::MAX; w.f.accounts[t[11]].lamports=u64::MAX;
            w.f.accounts[t[14]].lamports=floor(14)+1; w.f.accounts[t[15]].lamports=floor(15)+73;
        } else if mode==5 {
            w.f.accounts[t[9]].lamports=0;
            w.f.accounts[t[14]].lamports=u64::MAX; w.f.accounts[t[15]].lamports=floor(15);
        }
        let before=w.clone();
        let excess=(before.f.accounts[t[14]].lamports.saturating_sub(floor(14)) as u128)
            +(before.f.accounts[t[15]].lamports.saturating_sub(floor(15)) as u128);
        let pending=before.f.accounts[t[9]].lamports.max(floor(9)) as u128+excess;
        let pending=u64::try_from(pending).unwrap(); let excess=u64::try_from(excess).unwrap();
        let original_rent: u64=t.iter().enumerate().map(|(slot,index)|floor(slot).saturating_sub(before.f.accounts[*index].lamports)).sum();
        let expected_calls=expected_calls(&before).len(); let (result,calls)=w.raw(Behavior::Good); let result=result.unwrap();
        assert_eq!(calls,expected_calls); assert_eq!(result.allocation().funded_rent_lamports(),original_rent);
        assert_eq!(result.allocation().normalized_token_prefund_lamports(),excess);
        assert_eq!(w.f.accounts[w.payer].lamports,before.f.accounts[w.payer].lamports-original_rent);
        for (slot,index) in t.iter().copied().enumerate() {
            let expected=if slot==9 {pending}else if slot>=14 {floor(slot)}else{before.f.accounts[index].lamports.max(floor(slot))};
            assert_eq!(w.f.accounts[index].lamports,expected,"target {slot}");
            if slot>=14 {assert_eq!(w.f.accounts[index].data,independent_token(&w));}
        }
        for (index,account) in before.f.accounts.iter().enumerate() {
            if !t.contains(&index) && index!=w.payer {assert_eq!(&w.f.accounts[index],account);}
        }
        let initial: PivConfig=decoded(&w.f.accounts[t[0]].data);
        assert_eq!(&initial,result.model().proposed_config()); assert_eq!(initial.paused,paused);
        assert_eq!(initial.accounted_pending_sol_lamports,0,"custody normalization does not recognize a ledger");
        assert_eq!(fixed_observation(&mut w).unwrap().pending_sol.lamports,pending);
        let initialized=w.f.clone(); let recognized=recognize_pending(&mut w);
        assert_eq!(recognized.newly_accounted_sol_lamports,pending-floor(9));
        assert_eq!(recognized.newly_accounted_jitosol_units,0);
        let mut only_pending_changed=initial;
        only_pending_changed.accounted_pending_sol_lamports=pending-floor(9);
        let actual: PivConfig=decoded(&w.f.accounts[t[0]].data);
        assert_eq!(actual,only_pending_changed,"HWM, history, KIF, carry, pause and all other Config fields stay unchanged");
        for (index,account) in initialized.accounts.iter().enumerate() {
            if index!=t[0] {assert_eq!(&w.f.accounts[index],account,"only Config bytes may change");}
        }
        let once=w.f.clone(); let repeated=recognize_pending(&mut w);
        assert_eq!((repeated.newly_accounted_sol_lamports,repeated.newly_accounted_jitosol_units),(0,0));
        assert_fixture(&w.f,&once); assert!(fixed_observation(&mut w).is_ok());
    } } } }
}

#[test]
fn normalized_destination_capacity_and_excess_overflow_are_checked_before_any_effect() {
    for case in 0..3 {
        let mut w=World::normalized(PROGRAM,false,false); let t=w.roles.targets;
        w.f.accounts[t[14]].lamports=u64::MAX;
        match case {
            0=>w.f.accounts[t[15]].lamports=u64::MAX, // excess sum overflows
            1=>w.f.accounts[t[9]].lamports=floor(14)+1, // destination balance overflows
            _=>{w.f.accounts[t[9]].lamports=u64::MAX;w.f.accounts[t[14]].lamports=floor(14)+1;},
        }
        let before=w.f.clone(); let (result,calls)=w.raw(Behavior::Good);
        assert_eq!(result,Err(E::Allocation(piv1::genesis_allocation::GenesisAllocationError::State(piv1::errors::Piv1Error::ArithmeticOverflow))));
        assert_eq!(calls,0); assert_fixture(&w.f,&before);
    }
    // Exactly-full destination is valid; rejecting all large prefunds is not a fix.
    let mut w=World::normalized(PROGRAM,false,false); let t=w.roles.targets;
    w.f.accounts[t[9]].lamports=u64::MAX-1;w.f.accounts[t[14]].lamports=floor(14)+1;
    assert!(w.raw(Behavior::Good).0.is_ok());assert_eq!(w.f.accounts[t[9]].lamports,u64::MAX);
    assert_eq!(recognize_pending(&mut w).newly_accounted_sol_lamports,u64::MAX-floor(9));
}

fn both_prefunds() -> World {
    let mut w=World::normalized(PROGRAM,false,false); let t=w.roles.targets;
    w.f.accounts[t[14]].lamports=floor(14)+17;w.f.accounts[t[15]].lamports=floor(15)+29;w
}

#[test]
fn normalized_path_requires_fresh_approval_payer_context_and_exact_roles() {
    for mutation in 0..10 {
        let mut w=both_prefunds();
        match mutation {
            0=>w.f.inner_data[16]^=1,
            1=>{w.f.proposal.approved.truncate(3);w.f.sync_proposal();},
            2=>w.f.accounts[w.payer].signer=false,
            3=>{let payer=w.f.accounts[w.payer].key;for m in &mut w.f.outer[0].accounts {if m.pubkey==payer {m.is_signer=false;}}w.f.sync_sysvar();},
            4=>w.f.stack_height=3,
            5=>w.f.clock.unix_timestamp=-1,
            6=>w.token=usize::MAX,
            7=>w.f.accounts[w.token].executable=false,
            8=>w.roles.targets.swap(14,15),
            _=>w.f.accounts[w.payer].lamports=floor(9),
        }
        w.reject_before();
    }
    let base=both_prefunds();
    for slot in [9,14,15] {
        let mut w=base.clone();w.f.accounts[w.roles.targets[slot]].writable=false;w.f.rebuild_message();w.reject_before();
    }
    for index in [base.payer,base.roles.protocol.mint,base.roles.targets[9],base.roles.targets[14],base.roles.targets[15]] {
        for data in [false,true] {
            let mut w=base.clone();let ctx=context(&w.f);let bytes=w.f.inner_data.clone();let roles=w.initialization_roles();
            let before=w.f.clone();
            w.f.with_infos(|a| {
                let data_guard=if data {Some(a[index].try_borrow_data().unwrap())}else{None};
                let lamport_guard=if !data {Some(a[index].try_borrow_mut_lamports().unwrap())}else{None};
                let mut calls=0;
                let result=initialize_with_mode(true,&PROGRAM,a,&bytes,roles,ctx,|_,_,_|{calls+=1;Ok(())});
                assert!(result.is_err());assert_eq!(calls,0);drop((data_guard,lamport_guard));
            });assert_fixture(&w.f,&before);
        }
    }
    for data in [false,true] {
        let mut w=base.clone();let ctx=context(&w.f);let bytes=w.f.inner_data.clone();let roles=w.initialization_roles();
        let before=w.f.clone();w.f.with_infos(|a| {
            let mut aliased=a.to_vec();let t=roles.allocation.preflight.targets;
            if data {aliased[t[15]].data=aliased[t[14]].data.clone();}
            else {aliased[t[15]].lamports=aliased[t[14]].lamports.clone();}
            let result=initialize_with_mode(true,&PROGRAM,&aliased,&bytes,roles,ctx,|_,_,_|panic!("no aliased effect"));
            assert!(result.is_err());
        });assert_fixture(&w.f,&before);
    }
}

#[test]
fn normalized_failures_at_every_cpi_propagate_without_production_undo() {
    let base=both_prefunds();let calls=expected_calls(&base);
    assert_eq!(calls.iter().filter(|c|matches!(c.op,Op::Sweep(_))).count(),2);
    for boundary in 0..calls.len() {for after in [false,true] {
        let mut w=base.clone();let before=w.f.clone();
        let behavior=if after {Behavior::ErrorAfter(boundary)}else{Behavior::ErrorBefore(boundary)};
        let error=ProgramError::Custom(if after {22102}else{22101});
        let error=if calls[boundary].op==Op::Token {E::Invocation(error)}
            else{E::Allocation(piv1::genesis_allocation::GenesisAllocationError::Invocation(error))};
        let (result,count)=w.transaction(behavior);assert_eq!(result,Err(error.clone()));assert_eq!(count,boundary+1);assert_fixture(&w.f,&before);
        let (result,count)=w.raw(behavior);assert_eq!(result,Err(error));assert_eq!(count,boundary+1);
        if after||boundary>0 {assert_ne!(w.f.accounts,before.accounts,"only the explicit staged model discards effects");}
        for slot in 0..9 {assert!(w.f.accounts[w.roles.targets[slot]].data.iter().all(|b|*b==0));}
    }}
}

#[test]
fn normalized_sweeps_require_exact_full_batch_effects_and_token_completion() {
    let base=both_prefunds();let calls=expected_calls(&base);
    for boundary in calls.iter().enumerate().filter_map(|(i,c)|matches!(c.op,Op::Sweep(_)).then_some(i)) {
        let mut w=base.clone();let before=w.f.clone();let (result,count)=w.transaction(Behavior::NoOp(boundary));
        assert_eq!(result,Err(E::Allocation(piv1::genesis_allocation::GenesisAllocationError::ObservationMismatch)));
        assert_eq!(count,boundary+1);assert_fixture(&w.f,&before);
    }
    let first=calls.iter().position(|c|matches!(c.op,Op::Sweep(_))).unwrap();
    for field in 0..7 {
        let mut w=base.clone();let before=w.f.clone();let (result,count)=w.transaction(Behavior::SweepTamper(field));
        assert_eq!(result,Err(E::Allocation(piv1::genesis_allocation::GenesisAllocationError::ObservationMismatch)));
        assert_eq!(count,first+1);assert_fixture(&w.f,&before);
    }
    for behavior in [Behavior::TokenNativeDelta,Behavior::PayerCredit,Behavior::MintTamper,Behavior::PriorToken,Behavior::WriteBorrow(8)] {
        let mut w=base.clone();let before=w.f.clone();assert!(w.transaction(behavior).0.is_err());assert_fixture(&w.f,&before);
    }
}

#[test]
fn normalized_genesis_cannot_replay_or_repair_later_token_donations_even_when_paused() {
    for paused in [false,true] {
        let mut w=World::normalized(PROGRAM,false,paused);let t=w.roles.targets;
        w.f.accounts[t[14]].lamports=floor(14)+1;assert!(w.raw(Behavior::Good).0.is_ok());w.reject_before();
        // The initialized Config (including paused=true) is not virgin bootstrap.
        // Later legacy Token-owned native donations remain a separate limitation.
        w.f.accounts[t[15]].lamports+=1;
        assert_eq!(fixed_observation(&mut w),Err(piv1::errors::Piv1Error::UnsupportedTokenNativeExcess));w.reject_before();
    }
    let mut partial=both_prefunds();let token_boundary=expected_calls(&partial).iter().position(|c|c.op==Op::Token).unwrap();
    assert!(partial.raw(Behavior::ErrorBefore(token_boundary)).0.is_err());partial.reject_before();
}
