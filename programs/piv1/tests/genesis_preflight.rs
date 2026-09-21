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
    instructions::initialize::*,
    integrations::{self, jito_identity::*},
    squads_execution::{ModeledSquadsInvocationContext, SquadsBootstrapRoles, SquadsExecutionError},
    state::{ActiveDistribution, GuardianRegistry, GuardianReward, PivConfig},
};
use solana_stake_interface::state::{Authorized, Lockup, Meta, StakeStateV2};
use spl_token::state::{Account as TokenAccount, AccountState, Mint};
use support::{key, BackingAccount, Fixture, PROGRAM, PROGRAM_ACCOUNT, PROGRAM_DATA,
    MULTISIG, PROPOSAL, TRANSACTION, VAULT, INSTRUCTIONS, CONFIG};
use GenesisPreflightError as E;

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
struct World { f: Fixture, roles: GenesisPreflightRoles, parameters: GenesisModelParameters }
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
        f.rebuild_message(); Self { f, roles: GenesisPreflightRoles { bootstrap: bootstrap(), protocol, targets }, parameters }
    }
    fn run(&mut self) -> GenesisPreflightResult<GenesisAccountPreflight> {
        let before = self.f.clone(); let program = self.f.program; let data = self.f.inner_data.clone();
        let roles = self.roles; let ctx = context(&self.f);
        let result = self.f.with_infos(|a| preflight_approved_genesis_accounts_with_host_context(&program, a, &data, roles, ctx));
        assert_preserved(&self.f, &before); result
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
fn expected_spaces() -> [usize;16] {
    [PivConfig::SPACE, ActiveDistribution::SPACE, GuardianRegistry::SPACE,
        GuardianReward::SPACE, GuardianReward::SPACE, GuardianReward::SPACE,
        GuardianReward::SPACE, GuardianReward::SPACE, GuardianReward::SPACE, 0,0,0,0,0,165,165]
}

#[test]
fn nested_genesis_observations_keep_bounded_inline_footprints() {
    use core::mem::{align_of, size_of};
    use piv1::{genesis_allocation::AllocatedGenesisAccounts,
        genesis_initialization::InitializedGenesisAccounts,
        genesis_model::ApprovedGenesisModel, genesis_recipients::GenesisRecipientPreflight};

    // These returned aggregates were copied through nested Results before the
    // private boxing correction. Bound the aggregates, not a guessed compiler
    // frame or total genesis heap consumption; the target compiler is the gate.
    assert!(size_of::<GenesisAccountPreflight>() <= 64);
    assert!(size_of::<GenesisPreflightResult<GenesisAccountPreflight>>() <= 128);
    assert!(size_of::<GenesisRecipientPreflight>() <= 512);
    assert!(size_of::<AllocatedGenesisAccounts>() <= 1024);
    assert!(size_of::<InitializedGenesisAccounts>() <= 1024);
    println!("genesis inline bytes: preflight={}, result={}, recipients={}, allocation={}, initialization={}",
        size_of::<GenesisAccountPreflight>(), size_of::<GenesisPreflightResult<GenesisAccountPreflight>>(),
        size_of::<GenesisRecipientPreflight>(), size_of::<AllocatedGenesisAccounts>(),
        size_of::<InitializedGenesisAccounts>());
    // Exactly one new Box per model and protocol on each successful preflight.
    // SBF's bump allocator does not reclaim either request during the call.
    println!("added preflight heap requests: model={} align={}, protocol={} align={}, total={}",
        size_of::<ApprovedGenesisModel>(), align_of::<ApprovedGenesisModel>(),
        size_of::<AuthenticatedJitoIdentity>(), align_of::<AuthenticatedJitoIdentity>(),
        size_of::<ApprovedGenesisModel>() + size_of::<AuthenticatedJitoIdentity>());
}

#[test]
fn both_runtime_ids_bind_all_targets_intended_owners_rent_and_zero_model() {
    for program in [PROGRAM, key(211)] {
        let mut w = World::new(program,false); let result = w.run().unwrap(); let mut expected_total = 0;
        for (slot, observation) in result.targets().iter().enumerate() {
            let expected_owner = if slot < 9 { program } else if slot < 14 { system_program::ID } else { spl_token::ID };
            assert_eq!((observation.target().owner(), observation.target().size()), (expected_owner, expected_spaces()[slot]));
            assert_eq!(observation.target().address(), w.f.accounts[w.roles.targets[slot]].key);
            let minimum = ((128 + expected_spaces()[slot]) as u64) * 3480 * 2;
            assert_eq!((observation.observed_lamports(), observation.rent_minimum(), observation.shortfall()), (0,minimum,minimum));
            expected_total += minimum;
        }
        assert_eq!(result.total_rent_shortfall(),expected_total);
        let c = result.model().proposed_config();
        assert_eq!((c.configured_slippage_bps,c.htfp_reserve_bps,c.permanent_compound_bps,c.team_owner_pool_bps,c.kif_bps), (1,5900,1950,1950,200));
        assert_eq!([c.protected_principal_hwm_lamports,c.accounted_historical_jitosol_units,c.accounted_historical_sol_lamports,
            c.accounted_pending_jitosol_units,c.accounted_pending_sol_lamports,c.next_cycle_yield_lamports,c.kif_claim_liability_lamports,
            c.collective_kif_carry_lamports,c.cumulative_contribution_value_lamports,c.cumulative_gross_yield_lamports,
            c.cumulative_htfp_paid_lamports,c.cumulative_team_owner_paid_lamports,c.cumulative_kif_credited_lamports,
            c.cumulative_kif_claimed_lamports,c.cumulative_permanent_compound_lamports,c.cumulative_retained_dust_lamports,
            c.cumulative_zero_active_kif_compound_lamports,c.cumulative_cooldown_yield_recorded_lamports], [0;18]);
        assert_eq!((c.last_successful_preparation_at,c.last_valid_insufficient_attempt_at), (None,None));
        assert_eq!(*result.model().proposed_distribution(),ActiveDistribution::new_idle(c.bumps.active_distribution));
        assert_eq!(result.model().proposed_registry().guardian_keys,[key(96),key(95),key(94),key(93),key(92),key(91)]);
        assert!(result.model().proposed_rewards().iter().all(|r|
            (r.last_active_period,r.claimable_lamports,r.cumulative_earned,r.cumulative_claimed)==(None,0,0,0)));
        assert_eq!(result.protocol().keys().pool,c.stake_pool);
        assert_ne!(result.protocol().pool().pool_token_supply(),result.protocol().mint().supply);
    }
}

#[test]
fn prefunding_is_observed_without_netting_classification_or_model_changes() {
    let mut w = World::new(PROGRAM,false); let initial = w.run().unwrap();
    for variant in 0..4 {
        for (slot,index) in w.roles.targets.iter().enumerate() {
            let floor = initial.targets()[slot].rent_minimum();
            w.f.accounts[*index].lamports = match variant {0=>floor-1,1=>floor,2=>floor+1,_=>u64::MAX};
        }
        let current = w.run().unwrap(); assert_eq!(current.model(),initial.model());
        assert_eq!(current.total_rent_shortfall(),if variant==0 {16}else{0});
        for (slot,observation) in current.targets().iter().enumerate() {
            assert_eq!(observation.observed_lamports(),w.f.accounts[w.roles.targets[slot]].lamports);
            assert_eq!(observation.shortfall(),if variant==0 {1}else{0});
        }
    }
    w.f.accounts[w.roles.targets[15]].lamports=0;
    assert_eq!(w.run().unwrap().total_rent_shortfall(),initial.targets()[15].rent_minimum());
}

#[test]
fn every_target_rejects_wrong_key_owner_executable_data_and_unapproved_or_approved_readonly() {
    for slot in 0..16 {
        for corruption in 0..6 {
            let mut w=World::new(PROGRAM,false);let i=w.roles.targets[slot];
            match corruption {0=>w.f.accounts[i].key=key(220),1=>w.f.accounts[i].owner=PROGRAM,
                2=>w.f.accounts[i].executable=true,3=>w.f.accounts[i].data.push(0),
                _=>w.f.accounts[i].writable=false}
            // Independent canonical target checks must reject even approved bad inputs.
            if corruption!=4 {w.f.rebuild_message();}
            assert!(w.run().is_err(),"target={slot},corruption={corruption}");
        }
    }
}

#[test]
fn every_target_data_or_lamport_borrow_failure_preserves_all_inputs() {
    for slot in 0..16 {for data_borrow in [false,true] {
        let mut w=World::new(PROGRAM,false);let before=w.f.clone();let ctx=context(&w.f);
        let program=w.f.program;let bytes=w.f.inner_data.clone();let roles=w.roles;let i=roles.targets[slot];
        w.f.with_infos(|a| {
            if data_borrow {let _guard=a[i].try_borrow_mut_data().unwrap();
                assert!(preflight_approved_genesis_accounts_with_host_context(&program,a,&bytes,roles,ctx).is_err());
            } else {let _guard=a[i].try_borrow_mut_lamports().unwrap();
                assert!(preflight_approved_genesis_accounts_with_host_context(&program,a,&bytes,roles,ctx).is_err());}
        });assert_preserved(&w.f,&before);
    }}
}

#[test]
fn role_groups_reject_invalid_duplicate_or_cross_group_indices_and_preserve_allowed_aliases() {
    let mut good=World::new(PROGRAM,true);good.run().unwrap();
    for corruption in 0..8 {
        let mut w=World::new(PROGRAM,false);
        match corruption {0=>w.roles.targets[0]=w.roles.targets[1],1=>w.roles.targets[3]=w.roles.targets[2],
            2=>w.roles.targets[15]=usize::MAX,3=>w.roles.protocol.mint=usize::MAX,
            4=>w.roles.protocol.program=PROGRAM_ACCOUNT,5=>w.roles.protocol.referrer=w.roles.protocol.mint,
            6=>w.roles.targets[9]=VAULT,_=>w.roles.protocol.referrer=w.roles.targets[9]}
        assert_eq!(w.run(),Err(E::InvalidRoles));
    }
}

#[test]
fn differently_keyed_accounts_cannot_share_target_backing() {
    for other_role in [0,1,2] {for data_alias in [false,true] {
        let mut w=World::new(PROGRAM,false);let before=w.f.clone();let program=w.f.program;
        let roles=w.roles;let bytes=w.f.inner_data.clone();let ctx=context(&w.f);
        let target=roles.targets[15];let other=match other_role {0=>roles.targets[14],1=>VAULT,_=>roles.protocol.pool};
        w.f.with_infos(|a|{let mut aliased=a.to_vec();
            if data_alias {aliased[target].data=aliased[other].data.clone();}
            else {aliased[target].lamports=aliased[other].lamports.clone();}
            assert_eq!(preflight_approved_genesis_accounts_with_host_context(&program,&aliased,&bytes,roles,ctx),Err(E::State(Piv1Error::AccountAlias)));
        });assert_preserved(&w.f,&before);
    }}
}

#[test]
fn approved_bytes_and_current_protocol_accounts_are_bound_and_rechecked() {
    let mut w=World::new(PROGRAM,false);w.run().unwrap();
    w.f.inner_data[11]^=1;assert!(w.run().is_err());
    let mut w=World::new(PROGRAM,false);w.parameters.protocol.stake_pool=key(221);w.approve_parameters();
    assert_eq!(w.run(),Err(E::Protocol(JitoIdentityError::InvalidIdentity)));
    let mut w=World::new(PROGRAM,false);w.run().unwrap();
    w.f.accounts[w.roles.protocol.mint].data[44]=8;
    assert_eq!(w.run(),Err(E::Protocol(JitoIdentityError::InvalidMint)));
    let mut w=World::new(PROGRAM,false);let i=w.roles.protocol.pool;
    w.f.accounts[i].key=key(222);w.f.rebuild_message();
    assert_eq!(w.run(),Err(E::Protocol(JitoIdentityError::InvalidIdentity)));
}

#[test]
fn fresh_approval_authority_clock_and_depth_are_required_after_prior_success() {
    for change in 0..5 {
        let mut w=World::new(PROGRAM,false);w.run().unwrap();
        match change {0=>{w.f.multisig.threshold=1;w.f.sync_multisig();},
            1=>{w.f.proposal.approved.pop();w.f.sync_proposal();},
            2=>{w.f.proposal.status=5;w.f.sync_proposal();},3=>w.f.clock.unix_timestamp=99,_=>w.f.stack_height=3}
        assert!(w.run().is_err(),"freshness case {change}");
    }
    let mut w=World::new(PROGRAM,false);w.parameters.kif_anchor_timestamp=101;w.approve_parameters();
    assert!(w.run().is_err());
    w.f.clock.unix_timestamp=101;
    assert_eq!(w.run().unwrap().model().modeled_period().id,0);
}

#[test]
fn rent_parameters_and_aggregate_shortfall_overflow_reject_without_prefund_aggregation() {
    for invalid in 0..5 {
        let mut w=World::new(PROGRAM,false);
        match invalid {0=>w.f.rent.exemption_threshold=f64::NAN,1=>w.f.rent.exemption_threshold=f64::INFINITY,
            2=>w.f.rent.exemption_threshold=-1.0,3=>w.f.rent.burn_percent=101,_=>w.f.rent.lamports_per_byte_year=u64::MAX}
        assert!(matches!(w.run(),Err(E::State(Piv1Error::InvalidRent|Piv1Error::ArithmeticOverflow))));
    }
    let mut w=World::new(PROGRAM,false);
    w.f.rent=Rent {lamports_per_byte_year:u64::MAX/2048,exemption_threshold:1.0,burn_percent:50};
    assert!(expected_spaces().iter().all(|size|size+128<=2048));
    let floors=expected_spaces().map(|size|w.f.rent.minimum_balance(size));
    assert!(floors.iter().all(|floor|*floor<u64::MAX));
    assert!(floors.iter().map(|floor|u128::from(*floor)).sum::<u128>()>u128::from(u64::MAX));
    assert_eq!(w.run(),Err(E::State(Piv1Error::ArithmeticOverflow)));
    for i in w.roles.targets {w.f.accounts[i].lamports=u64::MAX;}
    assert_eq!(w.run().unwrap().total_rent_shortfall(),0);
}

#[test]
fn ordinary_host_guard_and_existing_instruction_selectors_remain_separate() {
    let mut w=World::new(PROGRAM,false);let before=w.f.clone();let program=w.f.program;
    let data=w.f.inner_data.clone();let roles=w.roles;
    w.f.with_infos(|a|{let _guard=a[CONFIG].try_borrow_mut_data().unwrap();
        assert_eq!(preflight_approved_genesis_accounts(&program,a,&[],roles),
            Err(E::Model(GenesisModelError::Authorization(SquadsExecutionError::HostRuntimeUnavailable))));
        assert_eq!(piv1::process_instruction(&program,a,&data),Err(ProgramError::InvalidInstructionData));
        assert_eq!(piv1::instruction_boundary::process_instruction_with_host_callbacks(&program,a,&data,
            ||panic!("model tag must not read Rent"),|_,_,_|panic!("model tag must not invoke"),|_|panic!("model tag must not emit")),
            Err(ProgramError::InvalidInstructionData));
    });assert_preserved(&w.f,&before);
}
