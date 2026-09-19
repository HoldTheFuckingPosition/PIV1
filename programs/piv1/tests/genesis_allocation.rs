#[allow(dead_code)]
#[path = "support/squads_invocation.rs"]
mod support;
#[path = "support/jito_identity_oracle.rs"]
mod oracle;

use anchor_lang::{
    prelude::{AccountInfo, Pubkey},
    solana_program::{bpf_loader_upgradeable, entrypoint::ProgramResult, instruction::Instruction,
        program_error::ProgramError, program_option::COption, program_pack::Pack, system_program},
};
use piv1::{
    accounts::STAKE_PROGRAM_ID,
    genesis_allocation::*,
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
use GenesisAllocationError as E;

// Reuses the established Task2.20 account/oracle construction without modifying
// its existing tests. Additional payer/System accounts are in the approved list.
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
struct World { f: Fixture, roles: GenesisPreflightRoles, parameters: GenesisModelParameters, payer: usize, system: usize }
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
        f.rebuild_message(); Self { f, roles: GenesisPreflightRoles { bootstrap: bootstrap(), protocol, targets }, parameters, payer, system }
    }
    fn allocation_roles(&self) -> GenesisAllocationRoles {
        GenesisAllocationRoles { preflight: self.roles, payer: self.payer, system_program: self.system }
    }
    fn raw(&mut self, behavior: Behavior) -> (GenesisAllocationResult<AllocatedGenesisAccounts>, usize) {
        let before = self.clone(); let program = self.f.program; let data = self.f.inner_data.clone();
        let roles = self.allocation_roles(); let ctx = context(&self.f); let calls = expected_calls(&before);
        let mut count = 0;
        let (result, observed) = self.f.with_infos(|infos| {
            let result = allocate_approved_genesis_accounts_with_host_invoker(&program, infos, &data, roles, ctx,
                |ix, actual, signers| {
                    let call = &calls[count]; let current = count; count += 1;
                    emulate(&before, call, behavior, current, ix, actual, signers, infos)
                });
            // The modeled allocator replaces data slices using owned host buffers;
            // never call SDK resize/realloc on non-runtime memory layouts.
            let observed = infos.iter().map(snapshot).collect::<Vec<_>>();
            (result, observed)
        });
        self.f.accounts = observed;
        (result, count)
    }
    /// Explicit clone/discard transaction MODEL, never evidence of SVM rollback.
    fn transaction(&mut self, behavior: Behavior) -> (GenesisAllocationResult<AllocatedGenesisAccounts>, usize) {
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
enum Op { Transfer(u64), Allocate(usize), Assign(Pubkey) }
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
    Good, ErrorBefore(usize), ErrorAfter(usize), NoOp(usize), ShortTransfer,
    ExtraTransfer, WrongSize, DirtyAllocation, WrongOwner, PreviousTarget, PayerCredit,
}
#[allow(clippy::too_many_arguments)]
fn emulate(
    w: &World, call: &Call, behavior: Behavior, index: usize, ix: &Instruction,
    infos: &[AccountInfo<'_>], signers: &[&[&[u8]]], all: &[AccountInfo<'_>],
) -> ProgramResult {
    assert_eq!(ix.program_id, system_program::ID);
    let target = w.f.accounts[w.roles.targets[call.slot]].key;
    let payer = w.f.accounts[w.payer].key;
    let (expected_bytes, expected_metas, expected_infos, expected_signers) = match call.op {
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
            let transferred = match behavior { Behavior::ShortTransfer=>value-1, Behavior::ExtraTransfer=>value+1, _=>value };
            **infos[0].try_borrow_mut_lamports()? -= transferred;
            **infos[1].try_borrow_mut_lamports()? += transferred;
            if behavior == Behavior::PayerCredit { **infos[0].try_borrow_mut_lamports()? += 1; }
        }
        Op::Allocate(size) => {
            let size = if behavior == Behavior::WrongSize { size - 1 } else { size };
            let value = if behavior == Behavior::DirtyAllocation { 7 } else { 0 };
            *infos[0].try_borrow_mut_data()? = Box::leak(vec![value;size].into_boxed_slice());
        }
        Op::Assign(owner) => infos[0].assign(&if behavior == Behavior::WrongOwner { key(225) } else { owner }),
    }
    if behavior == Behavior::PreviousTarget && index == 3 {
        **all[w.roles.targets[0]].try_borrow_mut_lamports()? += 1;
    }
    if behavior == Behavior::ErrorAfter(index) { return Err(ProgramError::Custom(22102)); }
    Ok(())
}

#[test]
fn exact_pinned_cpis_cover_both_runtime_ids_every_seed_and_all_prefunding_levels() {
    for program in [PROGRAM, key(211)] { for same_receiver in [false,true] { for mode in 0..5 {
        let mut w = World::new(program,same_receiver);
        for (slot,index) in w.roles.targets.iter().copied().enumerate() {
            w.f.accounts[index].lamports = match mode { 0=>0,1=>floor(slot)-1,2=>floor(slot),3=>floor(slot)+55,_=>u64::MAX };
        }
        let before = w.clone(); let (outcome,calls) = w.raw(Behavior::Good); let outcome = outcome.unwrap();
        let funding: u64 = (0..16).map(|slot| floor(slot).saturating_sub(before.f.accounts[before.roles.targets[slot]].lamports)).sum();
        assert_eq!(calls, if mode<2 {38}else{22}); assert_eq!(outcome.funded_rent_lamports(),funding);
        assert_eq!(outcome.payer(),before.f.accounts[before.payer].key);
        assert_eq!((outcome.payer_before(),outcome.payer_after()), (1_000_000_000,1_000_000_000-funding));
        for (slot,index) in w.roles.targets.iter().copied().enumerate() {
            let a = &w.f.accounts[index]; let old = &before.f.accounts[index];
            assert_eq!(a.lamports,old.lamports.max(floor(slot)));
            assert_eq!(a.owner,if slot<9 {program}else if slot<14 {system_program::ID}else{spl_token::ID});
            assert_eq!(a.data,vec![0;sizes()[slot]]); // deliberately uninitialized
            assert_eq!(outcome.before().targets()[slot].observed_lamports(),old.lamports);
        }
        for (index,a) in before.f.accounts.iter().enumerate() {
            if index!=w.payer && !w.roles.targets.contains(&index) { assert_eq!(&w.f.accounts[index],a); }
        }
        assert_eq!(outcome.model().proposed_config().protected_principal_hwm_lamports,0);
        assert_eq!(outcome.model().proposed_config().accounted_pending_sol_lamports,0);
        assert_eq!(outcome.model().proposed_config().kif_claim_liability_lamports,0);
        assert_eq!(w.f.inner_data,before.f.inner_data);
    } } }
}

#[test]
fn payer_retains_exact_rent_floor_and_one_lamport_short_rejects_without_effects() {
    let mut w = World::new(PROGRAM,false); let rent_total: u64 = (0..16).map(floor).sum();
    w.f.accounts[w.payer].lamports = rent_total + floor(9) - 1; w.reject_before();
    w.f.accounts[w.payer].lamports += 1;
    let (result,_) = w.raw(Behavior::Good); assert_eq!(result.unwrap().payer_after(),floor(9));
    let mut w = World::new(PROGRAM,false);
    for (slot,index) in w.roles.targets.iter().copied().enumerate() { w.f.accounts[index].lamports = floor(slot); }
    w.f.accounts[w.payer].lamports = floor(9)-1; w.reject_before();
    w.f.accounts[w.payer].lamports += 1; let (result,_) = w.raw(Behavior::Good);
    assert_eq!(result.unwrap().funded_rent_lamports(),0);
}

#[test]
fn payer_and_system_metadata_roles_protected_keys_and_exact_outer_signature_reject() {
    for change in 0..12 {
        let mut w = World::new(PROGRAM,false); let p = &mut w.f.accounts[w.payer];
        match change {
            0=>p.signer=false, 1=>p.writable=false, 2=>p.owner=key(231), 3=>p.executable=true,
            4=>p.data=vec![0], 5=>p.key=w.parameters.team_owner_recipient,
            6=>p.key=key(92), 7=>p.key=Pubkey::find_program_address(&[b"authority"],&PROGRAM).0,
            8=>w.f.accounts[w.system].key=key(232), 9=>w.f.accounts[w.system].executable=false,
            10=>w.payer=w.system, _=>w.system=w.payer,
        }
        w.f.rebuild_message(); w.reject_before();
    }
    let w = World::new(PROGRAM,false);
    let protected = [PROGRAM_ACCOUNT,PROGRAM_DATA,MULTISIG,PROPOSAL,TRANSACTION,VAULT,INSTRUCTIONS,CONFIG,
        w.roles.protocol.program,w.roles.protocol.pool,w.roles.protocol.validator_list,w.roles.protocol.reserve,
        w.roles.protocol.mint,w.roles.protocol.manager_fee,w.roles.protocol.referrer];
    for index in protected.into_iter().chain(w.roles.targets).chain([usize::MAX]) {
        let mut payer = w.clone(); payer.payer=index; payer.reject_before();
        let mut system = w.clone(); system.system=index; system.reject_before();
    }
    let mut w = World::new(PROGRAM,false); let payer = w.f.accounts[w.payer].key;
    for meta in &mut w.f.outer[0].accounts { if meta.pubkey==payer { meta.is_signer=false; } }
    w.f.sync_sysvar(); w.reject_before();
}

#[test]
fn approval_identity_and_target_mutations_are_fresh_before_every_allocation() {
    for change in 0..7 {
        let mut w = World::new(PROGRAM,false);
        let ctx=context(&w.f); let bytes=w.f.inner_data.clone(); let roles=w.roles;
        w.f.with_infos(|a| preflight_approved_genesis_accounts_with_host_context(&PROGRAM,a,&bytes,roles,ctx)).unwrap();
        match change {
            0=>{w.f.proposal.approved.truncate(3);w.f.sync_proposal();},
            1=>w.f.inner_data[16]^=1,
            2=>w.f.accounts[w.roles.protocol.mint].owner=key(230),
            3=>w.f.accounts[w.roles.targets[15]].owner=spl_token::ID,
            4=>w.f.clock.unix_timestamp=-1,
            5=>w.f.stack_height=3,
            _=>w.f.accounts[PROGRAM_DATA].data[13..45].copy_from_slice(key(229).as_ref()),
        }
        w.reject_before();
    }
}

#[test]
fn every_target_and_payer_borrow_conflict_rejects_before_first_effect() {
    let w = World::new(PROGRAM,false);
    for index in w.roles.targets.into_iter().chain([w.payer]) { for data_borrow in [false,true] {
        let mut current=w.clone(); let ctx=context(&current.f); let roles=current.allocation_roles();
        let bytes=current.f.inner_data.clone(); let before=current.f.clone();
        current.f.with_infos(|a| {
            let mut calls=0;
            let result=if data_borrow {
                let _guard=a[index].try_borrow_data().unwrap();
                allocate_approved_genesis_accounts_with_host_invoker(&PROGRAM,a,&bytes,roles,ctx,|_,_,_|{calls+=1;Ok(())})
            } else {
                let _guard=a[index].try_borrow_lamports().unwrap();
                allocate_approved_genesis_accounts_with_host_invoker(&PROGRAM,a,&bytes,roles,ctx,|_,_,_|{calls+=1;Ok(())})
            };
            assert!(result.is_err()); assert_eq!(calls,0);
        });
        assert_fixture(&current.f,&before);
    } }
}

#[test]
fn payer_backing_aliases_with_targets_protocol_and_governance_reject() {
    let base=World::new(PROGRAM,false);
    for index in [CONFIG,VAULT,PROGRAM_DATA,base.roles.protocol.pool] { for data_alias in [false,true] {
        let mut w=World::new(PROGRAM,false); let ctx=context(&w.f); let roles=w.allocation_roles();
        let bytes=w.f.inner_data.clone(); let before=w.f.clone();
        w.f.with_infos(|actual| {
            let mut a=actual.to_vec();
            if data_alias {a[roles.payer].data=a[index].data.clone();}
            else {a[roles.payer].lamports=a[index].lamports.clone();}
            let mut calls=0;
            let result=allocate_approved_genesis_accounts_with_host_invoker(&PROGRAM,&a,&bytes,roles,ctx,|_,_,_|{calls+=1;Ok(())});
            assert!(result.is_err()); assert_eq!(calls,0);
        });
        assert_fixture(&w.f,&before);
    } }
}

#[test]
fn invalid_rent_and_checked_aggregate_overflow_never_invoke() {
    for value in [f64::NAN,f64::INFINITY,-1.0] {
        let mut w=World::new(PROGRAM,false); w.f.rent.exemption_threshold=value; w.reject_before();
    }
    let mut w=World::new(PROGRAM,false); w.f.rent.lamports_per_byte_year=u64::MAX/2048;
    w.f.rent.exemption_threshold=1.0; w.f.accounts[w.payer].lamports=u64::MAX;
    assert!((0..16).map(|s|u128::from((128+sizes()[s])as u64)*u128::from(w.f.rent.lamports_per_byte_year)).sum::<u128>()>u128::from(u64::MAX));
    w.reject_before();
}

#[test]
fn every_cpi_failure_propagates_and_only_explicit_host_transaction_model_rolls_back() {
    for boundary in 0..38 { for after in [false,true] {
        let behavior=if after {Behavior::ErrorAfter(boundary)}else{Behavior::ErrorBefore(boundary)};
        let mut w=World::new(PROGRAM,false); let before=w.f.clone();
        let (result,calls)=w.transaction(behavior);
        assert_eq!(result,Err(E::Invocation(ProgramError::Custom(if after {22102}else{22101}))));
        assert_eq!(calls,boundary+1); assert_fixture(&w.f,&before);
        let (result,calls)=w.raw(behavior); assert!(result.is_err()); assert_eq!(calls,boundary+1);
        if after || boundary>0 {assert_ne!(w.f.accounts,before.accounts,"direct execution cannot undo CPI");}
        else {assert_fixture(&w.f,&before);}
    } }
}

#[test]
fn false_success_at_every_cpi_is_rejected_and_modeled_transaction_preserved() {
    for boundary in 0..38 {
        let mut w=World::new(PROGRAM,false); let before=w.f.clone();
        let (result,calls)=w.transaction(Behavior::NoOp(boundary));
        assert_eq!(result,Err(E::ObservationMismatch)); assert_eq!(calls,boundary+1); assert_fixture(&w.f,&before);
    }
}

#[test]
fn wrong_transfer_allocation_owner_and_prior_target_postconditions_reject() {
    for behavior in [Behavior::ShortTransfer,Behavior::ExtraTransfer,Behavior::WrongSize,
        Behavior::DirtyAllocation,Behavior::WrongOwner,Behavior::PreviousTarget,Behavior::PayerCredit] {
        let mut w=World::new(PROGRAM,false); let before=w.f.clone();
        let (result,calls)=w.transaction(behavior);
        assert_eq!(result,Err(E::ObservationMismatch)); assert!(calls>0); assert_fixture(&w.f,&before);
    }
}

#[test]
fn successful_allocation_cannot_replay_and_never_produces_initialized_token_state() {
    let mut w=World::new(PROGRAM,false); let (result,calls)=w.raw(Behavior::Good); assert!(result.is_ok()); assert_eq!(calls,38);
    for slot in [14,15] { assert_eq!(TokenAccount::unpack(&w.f.accounts[w.roles.targets[slot]].data),Err(ProgramError::UninitializedAccount)); }
    w.reject_before();
}

#[test]
fn ordinary_host_runtime_rejects_before_accounts_or_instruction_decoding() {
    let w=World::new(PROGRAM,false);
    assert_eq!(allocate_approved_genesis_accounts(&PROGRAM,&[],&[],w.allocation_roles()),Err(E::HostRuntimeUnavailable));
    // The new module does not add a native selector: approved model bytes still
    // reject through the existing ordinary host dispatcher.
    assert!(piv1::process_instruction(&PROGRAM,&[],&w.f.inner_data).is_err());
}
