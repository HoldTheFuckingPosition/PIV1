#[allow(dead_code)]
#[path = "support/squads_invocation.rs"]
mod support;
#[path = "support/jito_identity_oracle.rs"]
mod oracle;

use anchor_lang::{
    prelude::{Pubkey, Rent},
    solana_program::{bpf_loader_upgradeable, program_error::ProgramError,
        program_option::COption, program_pack::Pack, system_program},
};
use piv1::{
    accounts::STAKE_PROGRAM_ID,
    errors::Piv1Error,
    genesis_model::{prepare_approved_genesis_model_with_host_context, GenesisModelError},
    genesis_preflight::*,
    genesis_recipients::*,
    instructions::initialize::*,
    integrations::{self, jito_identity::*},
    squads_execution::{ModeledSquadsInvocationContext, SquadsBootstrapRoles, SquadsExecutionError},
    state::ActiveDistribution,
};
use solana_stake_interface::state::{Authorized, Lockup, Meta, StakeStateV2};
use spl_token::state::{Account as TokenAccount, AccountState, Mint};
use support::{key, BackingAccount, Fixture, PROGRAM, PROGRAM_ACCOUNT, PROGRAM_DATA,
    MULTISIG, PROPOSAL, TRANSACTION, VAULT, INSTRUCTIONS, CONFIG};
use GenesisRecipientError as E;

// Independent literal source identity and seed recipe; no production recipient helper.
const SQUADS: Pubkey = Pubkey::from_str_const("SQDS4ep65T869zMMBKyuUq6aD6EgTu8psMjkvj52pCf");
fn recipient(multisig: Pubkey, index: u8) -> (Pubkey,u8) {
    Pubkey::find_program_address(&[b"multisig",multisig.as_ref(),b"vault",&[index]],&SQUADS)
}
fn native_floor() -> u64 { 128 * 3480 * 2 }

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
struct World { f: Fixture, roles: GenesisRecipientRoles, parameters: GenesisModelParameters }
impl World {
    fn new(program: Pubkey, same_receiver: bool, indices: [u8;2], paused: bool) -> Self {
        let mut f = Fixture::new(); f.accounts.truncate(8); f.program = program;
        f.accounts[PROGRAM_ACCOUNT].key = program;
        let pd = Pubkey::find_program_address(&[program.as_ref()], &bpf_loader_upgradeable::ID).0;
        f.accounts[PROGRAM_DATA].key = pd;
        f.accounts[PROGRAM_ACCOUNT].data[4..36].copy_from_slice(pd.as_ref());
        f.accounts[CONFIG] = account(Pubkey::find_program_address(&[b"config"], &program).0,
            system_program::ID, vec![], false, true);
        let parameters = GenesisModelParameters { vault_index: 7, initially_paused: paused,
            protocol: DeclaredGenesisProtocol { stake_pool_program: JITO_STAKE_POOL_PROGRAM, stake_pool: JITO_STAKE_POOL,
                validator_list: key(201), reserve_stake: key(202), jitosol_mint: JITOSOL_MINT,
                manager_fee_account: key(203), referrer_token_account: if same_receiver { key(203) } else { key(204) } },
            htfp_recipient: recipient(f.accounts[MULTISIG].key,indices[0]).0,
            team_owner_recipient: recipient(f.accounts[MULTISIG].key,indices[1]).0, kif_anchor_timestamp: 0,
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
        let htfp_recipient=f.accounts.len();
        let team_owner_recipient=htfp_recipient+1;
        for address in [parameters.htfp_recipient,parameters.team_owner_recipient] {
            let mut vault=account(address,system_program::ID,vec![],false,false);vault.lamports=native_floor();f.accounts.push(vault);
        }
        f.rebuild_message(); Self { f, roles: GenesisRecipientRoles {
            preflight:GenesisPreflightRoles { bootstrap: bootstrap(), protocol, targets },
            htfp_recipient,team_owner_recipient,htfp_vault_index:indices[0],team_owner_vault_index:indices[1],
        }, parameters }
    }
    fn run(&mut self) -> GenesisRecipientResult<GenesisRecipientPreflight> {
        let before = self.f.clone(); let program = self.f.program; let data = self.f.inner_data.clone();
        let roles = self.roles; let ctx = context(&self.f);
        let result = self.f.with_infos(|a| preflight_approved_genesis_recipients_with_host_context(&program, a, &data, roles, ctx));
        assert_preserved(&self.f, &before); result
    }
    fn recipients(&self) -> [usize;2] { [self.roles.htfp_recipient,self.roles.team_owner_recipient] }
    fn set_approved_recipient(&mut self, slot: usize, address: Pubkey) {
        let index=self.recipients()[slot];self.f.accounts[index].key=address;
        if slot==0 {self.parameters.htfp_recipient=address;}else{self.parameters.team_owner_recipient=address;}
        self.approve_parameters();
    }
    fn approve_parameters(&mut self) { self.f.inner_data = self.parameters.encode().unwrap().to_vec(); self.f.rebuild_message(); }
}
fn assert_preserved(actual: &Fixture, before: &Fixture) {
    // Whole accounts, instruction bytes, metadata and context, including NaN Rent.
    assert_eq!(actual.rent.exemption_threshold.to_bits(), before.rent.exemption_threshold.to_bits());
    let mut comparable = actual.clone(); let mut expected = before.clone();
    comparable.rent.exemption_threshold = 0.0; expected.rent.exemption_threshold = 0.0;
    assert_eq!(comparable, expected);
}

fn base() -> World { World::new(PROGRAM,false,[0,255],false) }
fn authorization(error: SquadsExecutionError) -> E {
    E::Preflight(GenesisPreflightError::Model(GenesisModelError::Authorization(error)))
}

#[test]
fn fresh_recipient_identities_bind_approved_keys_indices_rent_and_zero_genesis_without_mutation() {
    for program in [PROGRAM,key(211)] {for shared in [false,true] {for paused in [false,true] {
        for indices in [[0,255],[255,0],[13,200]] {for mode in 0..3 {
            let mut w=World::new(program,shared,indices,paused);
            let expected_balance=match mode {0=>native_floor(),1=>native_floor()+1,_=>u64::MAX};
            for index in w.recipients() {w.f.accounts[index].lamports=expected_balance;}
            if mode==2 {for index in w.roles.preflight.targets {w.f.accounts[index].lamports=u64::MAX;}}
            let result=w.run().unwrap();let multisig=w.f.accounts[MULTISIG].key;
            for (slot,observed) in [result.htfp_recipient(),result.team_owner_recipient()].into_iter().enumerate() {
                let (address,bump)=recipient(multisig,indices[slot]);
                assert_eq!((observed.address(),observed.multisig(),observed.vault_index(),observed.bump()),(address,multisig,indices[slot],bump));
                assert_eq!(Pubkey::create_program_address(&[b"multisig",multisig.as_ref(),b"vault",&[indices[slot]],&[bump]],&SQUADS).unwrap(),address);
                assert_eq!((observed.observed_lamports(),observed.rent_minimum()),(expected_balance,native_floor()));
                assert!(!w.f.accounts[w.recipients()[slot]].signer,"no recipient signature is required");
            }
            let genesis=result.genesis();let c=genesis.model().proposed_config();
            assert_eq!((c.htfp_recipient,c.team_owner_recipient,c.paused),(w.parameters.htfp_recipient,w.parameters.team_owner_recipient,paused));
            assert_eq!([c.protected_principal_hwm_lamports,c.accounted_historical_jitosol_units,c.accounted_historical_sol_lamports,
                c.accounted_pending_jitosol_units,c.accounted_pending_sol_lamports,c.next_cycle_yield_lamports,c.kif_claim_liability_lamports,
                c.collective_kif_carry_lamports,c.cumulative_contribution_value_lamports,c.cumulative_gross_yield_lamports,
                c.cumulative_htfp_paid_lamports,c.cumulative_team_owner_paid_lamports,c.cumulative_kif_credited_lamports,
                c.cumulative_kif_claimed_lamports,c.cumulative_permanent_compound_lamports,c.cumulative_retained_dust_lamports,
                c.cumulative_zero_active_kif_compound_lamports,c.cumulative_cooldown_yield_recorded_lamports],[0;18]);
            assert_eq!(*genesis.model().proposed_distribution(),ActiveDistribution::new_idle(c.bumps.active_distribution));
            assert!(genesis.model().proposed_rewards().iter().all(|r| (r.claimable_lamports,r.cumulative_earned,r.cumulative_claimed,r.last_active_period)==(0,0,0,None)));
            let sizes=[1014_u64,891,210,84,84,84,84,84,84,0,0,0,0,0,165,165];
            let expected_rent=if mode==2 {0}else{sizes.iter().map(|size|(128+size)*3480*2).sum()};
            assert_eq!(genesis.total_rent_shortfall(),expected_rent,"recipient funding never joins or offsets target rent");
            for index in w.roles.preflight.targets {
                assert_eq!(w.f.accounts[index].owner,system_program::ID);assert!(w.f.accounts[index].data.is_empty());
            }
            assert_eq!(w.f.inner_data.len(),313);assert_eq!(&w.f.inner_data[..8],b"PIV1GM01");
            assert_eq!(&w.f.inner_data[8..11],&[1,7,u8::from(paused)]);
            assert_eq!(&w.f.inner_data[235..267],c.htfp_recipient.as_ref());
            assert_eq!(&w.f.inner_data[267..299],c.team_owner_recipient.as_ref());
        }}
    }}}
}

#[test]
fn neither_declared_keys_nor_index_witnesses_replace_canonical_same_multisig_identity() {
    for slot in 0..2 {for mutation in 0..5 {
        let mut w=base();let index=w.recipients()[slot];
        let expected=match mutation {
            0=>{w.f.accounts[index].key=key(230);w.f.rebuild_message();E::UnapprovedRecipient},
            1=>{w.set_approved_recipient(slot,key(230));E::InvalidRecipientVault},
            2=>{if slot==0 {w.roles.htfp_vault_index=17;}else{w.roles.team_owner_vault_index=17;}E::InvalidRecipientVault},
            3=>{let other=recipient(key(231),if slot==0 {0}else{255}).0;w.set_approved_recipient(slot,other);E::InvalidRecipientVault},
            _=>{let other=Pubkey::find_program_address(&[b"multisig",w.f.accounts[MULTISIG].key.as_ref(),b"vault",&[if slot==0 {0}else{255}]],&PROGRAM).0;
                w.set_approved_recipient(slot,other);E::InvalidRecipientVault},
        };
        assert_eq!(w.run(),Err(expected),"slot {slot} mutation {mutation}");
    }}
}

#[test]
fn each_recipient_must_be_funded_empty_nonexecutable_and_system_owned() {
    for slot in 0..2 {for mutation in 0..5 {
        let mut w=base();let index=w.recipients()[slot];let account=&mut w.f.accounts[index];
        let expected=match mutation {
            0=>{account.owner=PROGRAM;E::State(Piv1Error::InvalidAccountOwner)},
            1=>{account.data.push(0);E::State(Piv1Error::InvalidAccountSize)},
            2=>{account.executable=true;E::State(Piv1Error::ExecutableAccount)},
            3=>{account.lamports=native_floor()-1;E::State(Piv1Error::AccountRentDeficit)},
            _=>{account.lamports=0;E::UnfundedRecipient},
        };
        assert_eq!(w.run(),Err(expected));
    }}
    let mut w=base();w.f.rent=Rent {lamports_per_byte_year:0,exemption_threshold:2.0,burn_percent:0};
    for index in w.recipients() {w.f.accounts[index].lamports=1;}
    let result=w.run().unwrap();assert_eq!(result.htfp_recipient().rent_minimum(),0);assert_eq!(result.genesis().total_rent_shortfall(),0);
    for slot in 0..2 {let mut absent=w.clone();let index=absent.recipients()[slot];absent.f.accounts[index].lamports=0;
        assert_eq!(absent.run(),Err(E::UnfundedRecipient),"zero rent does not prove an absent vault exists");}
    let mut w=base();w.f.rent.lamports_per_byte_year=4_000;
    assert_eq!(w.run(),Err(E::State(Piv1Error::AccountRentDeficit)));
    for index in w.recipients() {w.f.accounts[index].lamports=128*4_000*2;}
    assert_eq!(w.run().unwrap().team_owner_recipient().rent_minimum(),128*4_000*2);
    for value in [f64::NAN,f64::INFINITY,-1.0] {
        let mut w=base();w.f.rent.exemption_threshold=value;
        assert_eq!(w.run(),Err(E::Preflight(GenesisPreflightError::State(Piv1Error::InvalidRent))));
    }
}

#[test]
fn recipient_indices_keys_and_backing_cannot_alias_any_other_account_or_guardian() {
    let original=base();
    for slot in 0..2 {
        for index in (0..original.roles.htfp_recipient).chain([usize::MAX,original.recipients()[1-slot]]) {
            let mut w=original.clone();if slot==0 {w.roles.htfp_recipient=index;}else{w.roles.team_owner_recipient=index;}
            assert_eq!(w.run(),Err(E::InvalidRoles));
        }
        for other in 0..original.f.accounts.len() {
            if other==original.recipients()[slot] {continue;}
            let mut w=original.clone();let key=w.f.accounts[other].key;w.set_approved_recipient(slot,key);
            assert_eq!(w.run(),Err(E::State(Piv1Error::AccountAlias)));
        }
        for guardian in 91..97 {let mut w=original.clone();w.set_approved_recipient(slot,key(guardian));
            assert_eq!(w.run(),Err(E::State(Piv1Error::AccountAlias)));}
        let mut w=original.clone();let authority=Pubkey::find_program_address(&[b"authority"],&PROGRAM).0;w.set_approved_recipient(slot,authority);
        // Existing Config address validation rejects this declaration before
        // the later role-specific alias checks.
        assert_eq!(w.run(),Err(E::Preflight(GenesisPreflightError::Model(GenesisModelError::State(Piv1Error::InvalidAddress)))));
        for other in [VAULT,CONFIG,original.roles.preflight.protocol.mint,original.recipients()[1-slot]] {for data in [false,true] {
            let mut w=original.clone();let before=w.f.clone();let roles=w.roles;let bytes=w.f.inner_data.clone();let ctx=context(&w.f);let index=w.recipients()[slot];
            w.f.with_infos(|a| {let mut aliased=a.to_vec();
                if data {aliased[index].data=aliased[other].data.clone();}else{aliased[index].lamports=aliased[other].lamports.clone();}
                assert_eq!(preflight_approved_genesis_recipients_with_host_context(&PROGRAM,&aliased,&bytes,roles,ctx),Err(E::State(Piv1Error::AccountAlias)));
            });assert_preserved(&w.f,&before);
        }}
    }
}

#[test]
fn read_only_recipient_borrows_are_shared_and_mutable_conflicts_fail_without_effects() {
    for slot in 0..2 {for data in [false,true] {
        let mut w=base();let before=w.f.clone();let roles=w.roles;let bytes=w.f.inner_data.clone();let ctx=context(&w.f);let index=w.recipients()[slot];
        w.f.with_infos(|a| {
            let data_guard=if data {Some(a[index].try_borrow_mut_data().unwrap())}else{None};
            let lamport_guard=if !data {Some(a[index].try_borrow_mut_lamports().unwrap())}else{None};
            assert_eq!(preflight_approved_genesis_recipients_with_host_context(&PROGRAM,a,&bytes,roles,ctx),Err(E::State(Piv1Error::AccountBorrowFailed)));
            drop((data_guard,lamport_guard));
        });assert_preserved(&w.f,&before);
    }}
    let mut w=base();let before=w.f.clone();let roles=w.roles;let bytes=w.f.inner_data.clone();let ctx=context(&w.f);
    w.f.with_infos(|a| {
        let _data=a[roles.htfp_recipient].try_borrow_data().unwrap();let _lamports=a[roles.team_owner_recipient].try_borrow_lamports().unwrap();
        assert!(preflight_approved_genesis_recipients_with_host_context(&PROGRAM,a,&bytes,roles,ctx).is_ok());
    });assert_preserved(&w.f,&before);
}

#[test]
fn fresh_approval_and_full_account_message_are_required_after_previous_observation() {
    for change in 0..9 {
        let mut w=base();let prior=w.run().unwrap();
        let expected=match change {
            0=>{w.f.inner_data[16]^=1;authorization(SquadsExecutionError::MessageMismatch)},
            1=>{w.f.proposal.approved.pop();w.f.sync_proposal();authorization(SquadsExecutionError::InvalidProposal)},
            2=>{w.f.multisig.stale_transaction_index=w.f.transaction.index;w.f.sync_multisig();authorization(SquadsExecutionError::InvalidProposal)},
            3=>{w.f.proposal.status=5;w.f.sync_proposal();authorization(SquadsExecutionError::InvalidProposal)},
            4=>{w.f.clock.unix_timestamp=99;authorization(SquadsExecutionError::TimelockNotReleased)},
            5=>{w.f.stack_height=3;authorization(SquadsExecutionError::InvalidInvocation)},
            6=>{w.f.multisig.threshold=1;w.f.sync_multisig();authorization(SquadsExecutionError::State(Piv1Error::InvalidGuardianSet))},
            7=>{w.f.accounts.push(account(key(240),system_program::ID,vec![],false,false));authorization(SquadsExecutionError::MessageMismatch)},
            _=>{w.f.accounts.swap(w.roles.htfp_recipient,w.roles.team_owner_recipient);authorization(SquadsExecutionError::MessageMismatch)},
        };
        assert_eq!(w.run(),Err(expected));assert_eq!(prior.htfp_recipient().observed_lamports(),native_floor());
    }
    let mut w=base();let roles=w.roles.preflight;let bytes=w.f.inner_data.clone();let ctx=context(&w.f);
    let old_genesis=w.f.with_infos(|a|preflight_approved_genesis_accounts_with_host_context(&PROGRAM,a,&bytes,roles,ctx)).unwrap();
    let index=w.roles.htfp_recipient;w.f.accounts[index].lamports=0;
    assert_eq!(w.run(),Err(E::UnfundedRecipient));assert_eq!(old_genesis.model().proposed_config().accounted_pending_sol_lamports,0);
    w.f.accounts[index].lamports=native_floor()+17;assert_eq!(w.run().unwrap().htfp_recipient().observed_lamports(),native_floor()+17);
}

#[test]
fn recipient_privilege_unions_never_replace_exact_approved_identity_or_other_preflight_checks() {
    for signer in [false,true] {for writable in [false,true] {
        let mut w=base();for index in w.recipients() {w.f.accounts[index].signer=signer;w.f.accounts[index].writable=writable;}
        w.f.rebuild_message();assert!(w.run().is_ok());w.set_approved_recipient(0,key(230));assert_eq!(w.run(),Err(E::InvalidRecipientVault));
    }}
    let mut w=base();w.f.accounts[w.roles.htfp_recipient].writable=true;
    assert_eq!(w.run(),Err(authorization(SquadsExecutionError::MessageMismatch)));
    let mut w=base();w.f.accounts[w.roles.preflight.protocol.mint].data[44]=8;
    assert_eq!(w.run(),Err(E::Preflight(GenesisPreflightError::Protocol(JitoIdentityError::InvalidMint))));
    let mut w=base();w.f.accounts[w.roles.preflight.targets[15]].owner=PROGRAM;
    assert_eq!(w.run(),Err(E::Preflight(GenesisPreflightError::State(Piv1Error::InvalidAccountOwner))));
    for paused in [false,true] {
        let mut w=World::new(PROGRAM,false,[0,255],paused);w.f.accounts[CONFIG].data.push(1);
        assert_eq!(w.run(),Err(authorization(SquadsExecutionError::State(Piv1Error::InvalidAccountSize))));
    }
}

#[test]
fn ordinary_host_guard_precedes_access_and_genesis_native_selector_stays_closed() {
    let mut w=base();let before=w.f.clone();let mut roles=w.roles;roles.htfp_recipient=usize::MAX;
    assert_eq!(preflight_approved_genesis_recipients(&PROGRAM,&[],&[],roles),Err(E::HostRuntimeUnavailable));
    let bytes=w.f.inner_data.clone();let roles=w.roles;
    w.f.with_infos(|a| {
        let _guard=a[roles.htfp_recipient].try_borrow_mut_data().unwrap();
        assert_eq!(preflight_approved_genesis_recipients(&PROGRAM,a,&[],roles),Err(E::HostRuntimeUnavailable));
        assert_eq!(piv1::instruction_boundary::process_instruction_with_host_callbacks(&PROGRAM,a,&bytes,
            ||panic!("no runtime read for undispatched genesis"),|_,_,_|panic!("no CPI"),|_|panic!("no event")),Err(ProgramError::InvalidInstructionData));
    });assert_preserved(&w.f,&before);
}
