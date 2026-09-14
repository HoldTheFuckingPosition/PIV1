#[allow(dead_code)]
#[path = "support/squads_invocation.rs"]
mod support;

use anchor_lang::{
    prelude::{AccountInfo, Pubkey, SolanaSysvar},
    solana_program::{bpf_loader_upgradeable, program_error::ProgramError,
        program_option::COption, program_pack::Pack, system_program, sysvar},
};
use piv1::{
    accounts::{authenticate_fixed_accounts, FixedAccountInfos, STAKE_PROGRAM_ID},
    errors::Piv1Error,
    genesis_model::*,
    guardian_clock_accounts::{authenticate_guardian_clock_snapshot, GuardianClockAccountInfos},
    instructions::initialize::*,
    squads_execution::{ModeledSquadsInvocationContext, SquadsBootstrapRoles, SquadsExecutionError},
    state::{ActiveDistribution, GuardianRegistry, GuardianReward, PivConfig},
    state_persistence::StateEnvelope,
};
use spl_token::state::{Account as TokenAccount, AccountState};
use support::*;
use GenesisModelError as E;
use GenesisModelFormatError as F;
use GenesisTargetRole as R;
use SquadsExecutionError as A;

fn parameters() -> GenesisModelParameters {
    GenesisModelParameters {
        vault_index: 7, initially_paused: false,
        protocol: DeclaredGenesisProtocol { stake_pool_program: key(201), stake_pool: key(202),
            validator_list: key(203), reserve_stake: key(204), jitosol_mint: key(205),
            manager_fee_account: key(206), referrer_token_account: key(207) },
        htfp_recipient: key(208), team_owner_recipient: key(209), kif_anchor_timestamp: -100,
        guardian_slot_permutation: [5, 4, 3, 2, 1, 0],
    }
}
fn roles() -> SquadsBootstrapRoles {
    SquadsBootstrapRoles { program: PROGRAM_ACCOUNT, program_data: PROGRAM_DATA, multisig: MULTISIG,
        proposal: PROPOSAL, transaction: TRANSACTION, vault: VAULT, instructions: INSTRUCTIONS, config: CONFIG }
}
fn approve(f: &mut Fixture, p: GenesisModelParameters) {
    f.inner_data = p.encode().unwrap().to_vec(); f.rebuild_message();
}
fn fixture(program: Pubkey) -> Fixture {
    let mut f = Fixture::new(); f.accounts.truncate(8); f.program = program;
    f.accounts[PROGRAM_ACCOUNT].key = program;
    let pd = Pubkey::find_program_address(&[program.as_ref()], &bpf_loader_upgradeable::ID).0;
    f.accounts[PROGRAM_DATA].key = pd;
    f.accounts[PROGRAM_ACCOUNT].data[4..36].copy_from_slice(pd.as_ref());
    f.accounts[CONFIG].key = Pubkey::find_program_address(&[b"config"], &program).0;
    f.accounts[CONFIG].owner = system_program::ID; f.accounts[CONFIG].data.clear();
    f.accounts[CONFIG].lamports = 0;
    approve(&mut f, parameters()); f
}
fn run(f: &mut Fixture) -> GenesisModelResult<ApprovedGenesisModel> {
    let before = f.clone(); let program = f.program; let data = f.inner_data.clone();
    let context = ModeledSquadsInvocationContext { stack_height: f.stack_height, clock: f.clock.clone(), rent: f.rent.clone() };
    let result = f.with_infos(|accounts| prepare_approved_genesis_model_with_host_context(&program, accounts, &data, roles(), context));
    assert_eq!(*f, before, "all input metadata/lamports/bytes and fixture context preserved"); result
}
fn reject(f: &mut Fixture, error: E) { assert_eq!(run(f), Err(error)); }

#[test]
fn fixed_model_format_has_independent_offsets_and_strict_domain() {
    let p = parameters(); let bytes = p.encode().unwrap();
    assert_eq!(bytes.len(), 313); assert_eq!(&bytes[..8], b"PIV1GM01");
    assert_eq!(&bytes[8..11], &[1, 7, 0]);
    for (i, tag) in (201..210).enumerate() { assert_eq!(&bytes[11 + 32*i..43 + 32*i], key(tag).as_ref()); }
    assert_eq!(&bytes[299..307], &(-100_i64).to_le_bytes());
    assert_eq!(&bytes[307..], &[5, 4, 3, 2, 1, 0]);
    assert_eq!(GenesisModelParameters::decode(&bytes), Ok(p));
    for end in 0..313 { assert_eq!(GenesisModelParameters::decode(&bytes[..end]), Err(F::InvalidLength)); }
    let mut extra = bytes.to_vec(); extra.push(0); assert_eq!(GenesisModelParameters::decode(&extra), Err(F::InvalidLength));
    for (offset, value, error) in [(0, 0, F::InvalidSelector), (8, 0, F::UnsupportedVersion),
        (8, 255, F::UnsupportedVersion), (10, 2, F::InvalidBoolean), (10, 255, F::InvalidBoolean)]
    {
        let mut bad = bytes; bad[offset] = value; assert_eq!(GenesisModelParameters::decode(&bad), Err(error));
    }
}

#[test]
fn permutation_decoder_enforces_a_bijection_without_authorization_repetition() {
    for slot in 0..6 {
        for value in 0..=6 {
            let mut p = parameters(); p.guardian_slot_permutation = [0, 1, 2, 3, 4, 5];
            p.guardian_slot_permutation[slot] = value;
            let mut bytes = parameters().encode().unwrap(); bytes[307..].copy_from_slice(&p.guardian_slot_permutation);
            if usize::from(value) == slot {
                assert_eq!(GenesisModelParameters::decode(&bytes), Ok(p));
            } else {
                assert_eq!(p.encode(), Err(F::InvalidSlotPermutation));
                assert_eq!(GenesisModelParameters::decode(&bytes), Err(F::InvalidSlotPermutation));
            }
        }
    }
    let mut bytes = parameters().encode().unwrap(); bytes[307] = 255;
    assert_eq!(GenesisModelParameters::decode(&bytes), Err(F::InvalidSlotPermutation));
}

#[test]
fn approved_permutation_and_pause_are_explicit_without_vote_activity_inference() {
    for permutation in [[0,1,2,3,4,5], [5,4,3,2,1,0], [2,5,0,4,1,3]] {
        for paused in [false, true] {
            let mut f = fixture(PROGRAM); let mut p = parameters(); p.guardian_slot_permutation = permutation;
            p.initially_paused = paused; approve(&mut f, p); let m = run(&mut f).unwrap();
            assert_eq!(m.proposed_config().paused, paused);
            assert_eq!(m.proposed_registry().guardian_keys, permutation.map(|slot| key(91 + slot)));
            assert!(m.proposed_rewards().iter().all(|r| r.last_active_period.is_none()));
            assert_eq!(m.proposed_registry().revision, 0);
        }
    }
}

#[test]
fn all_target_addresses_owners_allocations_and_bumps_follow_the_runtime_program() {
    for program in [PROGRAM, key(210)] {
        let mut f = fixture(program); let m = run(&mut f).unwrap(); assert_eq!(m.program(), program);
        let state = [(R::Config, b"config".as_slice(), PivConfig::SPACE),
            (R::ActiveDistribution, b"distribution".as_slice(), ActiveDistribution::SPACE),
            (R::GuardianRegistry, b"guardian-registry".as_slice(), GuardianRegistry::SPACE)];
        let native = [(R::PendingSol, b"pending-sol".as_slice()), (R::PrincipalSol, b"principal-sol".as_slice()),
            (R::OperationalSol, b"operational-sol".as_slice()), (R::DistributionEscrow, b"distribution-escrow".as_slice()),
            (R::KifSol, b"kif-sol".as_slice())];
        for target in m.targets() {
            let (seed, expected_owner, expected_size) = match target.role() {
                R::Config | R::ActiveDistribution | R::GuardianRegistry => {
                    let (_,seed,size) = state.iter().find(|x| x.0 == target.role()).unwrap(); (*seed, program, *size)
                },
                R::GuardianReward(slot) => {
                    let key = m.proposed_registry().guardian_keys[usize::from(slot)];
                    let (address,bump) = Pubkey::find_program_address(&[b"guardian-reward", key.as_ref(), &0_u64.to_le_bytes(), &[slot]], &program);
                    assert_eq!((target.address(),target.bump()), (address,bump));
                    assert_eq!((target.owner(),target.size()), (program,GuardianReward::SPACE)); continue;
                },
                R::PrincipalJito => (b"principal-jito-vault".as_slice(), spl_token::ID, 165),
                R::PendingJito => (b"pending-jito-vault".as_slice(), spl_token::ID, 165),
                _ => (native.iter().find(|x| x.0 == target.role()).unwrap().1, system_program::ID, 0),
            };
            let (address,bump) = Pubkey::find_program_address(&[seed], &program);
            assert_eq!((target.address(),target.bump(),target.owner(),target.size()), (address,bump,expected_owner,expected_size));
        }
        let mut addresses: Vec<_> = m.targets().iter().map(|t| t.address()).collect(); addresses.push(m.piv_authority());
        addresses.sort(); addresses.dedup(); assert_eq!(addresses.len(),17);
        assert_eq!(m.piv_authority(), Pubkey::find_program_address(&[b"authority"], &program).0);
        assert_eq!(m.proposed_config().bumps.piv_authority, Pubkey::find_program_address(&[b"authority"], &program).1);
    }
    for program in [system_program::ID, spl_token::ID, STAKE_PROGRAM_ID] {
        let mut f = fixture(program); reject(&mut f, E::State(Piv1Error::InvalidProgramIdentity));
    }
}

#[test]
fn config_has_fixed_economics_and_every_economic_or_audit_history_is_zero() {
    let mut f = fixture(PROGRAM); let m = run(&mut f).unwrap(); let c = m.proposed_config();
    assert_eq!((c.version,c.is_initialized,c.next_distribution_sequence,c.guardian_registry_revision), (1,true,0,0));
    assert_eq!((c.basis_points_denominator,c.htfp_reserve_bps,c.permanent_compound_bps,c.team_owner_pool_bps,c.kif_bps), (10000,5900,1950,1950,200));
    assert_eq!((c.configured_slippage_bps,c.slippage_hard_cap_bps), (1,1));
    assert_eq!((c.minimum_distribution_interval_seconds,c.insufficient_retry_cooldown_seconds,c.kif_period_seconds), (864000,86400,2592000));
    assert_eq!((c.last_successful_preparation_at,c.last_valid_insufficient_attempt_at), (None,None));
    assert_eq!(c.migration_reserve, [0;64]);
    assert_eq!([c.protected_principal_hwm_lamports,c.accounted_historical_jitosol_units,c.accounted_historical_sol_lamports,
        c.accounted_pending_jitosol_units,c.accounted_pending_sol_lamports,c.next_cycle_yield_lamports,c.kif_claim_liability_lamports,
        c.collective_kif_carry_lamports,c.cumulative_contribution_value_lamports,c.cumulative_gross_yield_lamports,
        c.cumulative_htfp_paid_lamports,c.cumulative_team_owner_paid_lamports,c.cumulative_kif_credited_lamports,
        c.cumulative_kif_claimed_lamports,c.cumulative_permanent_compound_lamports,c.cumulative_retained_dust_lamports,
        c.cumulative_zero_active_kif_compound_lamports,c.cumulative_cooldown_yield_recorded_lamports], [0;18]);
    assert_eq!(*m.proposed_distribution(), ActiveDistribution::new_idle(c.bumps.active_distribution));
    for r in m.proposed_rewards() {
        assert_eq!((r.last_active_period,r.claimable_lamports,r.cumulative_earned,r.cumulative_claimed), (None,0,0,0));
    }
}

#[test]
fn fresh_exact_approval_cannot_be_reused_for_substituted_parameters_or_accounts() {
    for offset in [10, 11, 43, 75, 107, 139, 171, 203, 235, 267, 299] {
        let mut f = fixture(PROGRAM); run(&mut f).unwrap(); f.inner_data[offset] ^= 1;
        reject(&mut f, E::Authorization(A::MessageMismatch));
    }
    let mut f = fixture(PROGRAM); f.inner_data.swap(307,308); reject(&mut f, E::Authorization(A::MessageMismatch));
    let mut f = fixture(PROGRAM); f.inner_data[9] = 8;
    reject(&mut f, E::Authorization(A::State(Piv1Error::InvalidProgramIdentity)));
    let mut f = fixture(PROGRAM); run(&mut f).unwrap(); f.multisig.threshold = 1; f.sync_multisig();
    reject(&mut f, E::Authorization(A::State(Piv1Error::InvalidGuardianSet)));
    let mut f = fixture(PROGRAM); run(&mut f).unwrap(); f.proposal.approved.truncate(1); f.sync_proposal();
    reject(&mut f, E::Authorization(A::InvalidProposal));
    let mut f = fixture(PROGRAM); f.accounts[CONFIG].key = key(218); f.rebuild_message();
    reject(&mut f, E::Authorization(A::State(Piv1Error::InvalidAccountPda)));
    let mut f = fixture(PROGRAM); f.clock.unix_timestamp = 99;
    reject(&mut f, E::Authorization(A::TimelockNotReleased));
    let mut f = fixture(PROGRAM); f.stack_height = 3; reject(&mut f, E::Authorization(A::InvalidInvocation));
}

#[test]
fn one_clock_controls_authentication_and_checked_half_open_anchor_periods() {
    for (anchor,now,period) in [(-100,-100,0), (-100,-1,0), (-100,2591899,0), (-100,2591900,1), (100,100,0)] {
        let mut f = fixture(PROGRAM); f.clock.unix_timestamp=now; f.multisig.time_lock=0;
        f.proposal.timestamp=now; f.sync_multisig(); f.sync_proposal();
        let mut p=parameters(); p.kif_anchor_timestamp=anchor; approve(&mut f,p);
        let m=run(&mut f).unwrap(); assert_eq!(m.modeled_period().id,period);
        assert!(m.modeled_period().start_timestamp <= now && now < m.modeled_period().end_timestamp);
    }
    for (anchor,now,error) in [(101,100,Piv1Error::TimestampRegression),
        (i64::MIN,i64::MAX,Piv1Error::ArithmeticOverflow), (i64::MAX,i64::MAX,Piv1Error::ArithmeticOverflow)]
    {
        let mut f=fixture(PROGRAM); f.clock.unix_timestamp=now; f.multisig.time_lock=0; f.proposal.timestamp=now;
        f.sync_multisig(); f.sync_proposal(); let mut p=parameters(); p.kif_anchor_timestamp=anchor; approve(&mut f,p);
        reject(&mut f,E::State(error));
    }
}

#[test]
fn declared_addresses_reject_defaults_and_aliases_but_preserve_valid_overlaps() {
    for offset in (11..299).step_by(32) {
        let mut f=fixture(PROGRAM); f.inner_data[offset..offset+32].fill(0); f.rebuild_message();
        reject(&mut f,E::State(Piv1Error::InvalidAddress));
    }
    let mut f=fixture(PROGRAM); let mut p=parameters(); p.htfp_recipient=p.protocol.stake_pool; approve(&mut f,p);
    reject(&mut f,E::State(Piv1Error::InvalidAddress));
    let mut f=fixture(PROGRAM); let mut p=parameters(); p.protocol.referrer_token_account=p.protocol.manager_fee_account;
    p.htfp_recipient=key(91); p.team_owner_recipient=f.accounts[VAULT].key; approve(&mut f,p);
    let m=run(&mut f).unwrap(); assert_eq!(m.proposed_config().htfp_recipient,key(91));
    assert_eq!(m.proposed_config().team_owner_recipient,f.accounts[VAULT].key);
}

#[test]
fn config_and_reward_targets_are_checked_beyond_config_bound_addresses() {
    let mut baseline=fixture(PROGRAM); let model=run(&mut baseline).unwrap();
    for target in model.targets().iter().map(|t|t.address()).chain([model.piv_authority()]) {
        let mut f=fixture(PROGRAM); let mut p=parameters(); p.htfp_recipient=target; approve(&mut f,p);
        assert!(matches!(run(&mut f),Err(E::State(Piv1Error::InvalidAddress | Piv1Error::AccountAlias))));
    }
    // A current member may be an arbitrary key; do not infer an on-curve rule.
    // Bind one to another guardian's reward PDA while preserving that slot/key.
    for forbidden in [model.targets()[0].address(),model.piv_authority(),model.targets()[3].address()] {
        let mut f=fixture(PROGRAM); f.multisig.members[0].key=forbidden; f.multisig.members.sort_by_key(|m|m.key);
        f.proposal.approved=[key(92),key(93),key(94),key(95)].to_vec(); f.sync_multisig(); f.sync_proposal();
        let mut p=parameters();
        // Preserve key96 in slot0 (whose reward PDA is the third forbidden key).
        let wanted=[key(96),key(95),key(94),key(93),key(92),forbidden];
        p.guardian_slot_permutation=wanted.map(|k|f.multisig.members.iter().position(|m|m.key==k).unwrap() as u8);
        approve(&mut f,p); f.outer[0].accounts[3].pubkey=key(92); f.sync_sysvar();
        reject(&mut f,E::State(Piv1Error::AccountAlias));
    }
}

#[test]
fn model_prefunding_and_actual_extra_targets_do_not_create_economic_history_or_readiness() {
    let mut f=fixture(PROGRAM); let baseline=run(&mut f).unwrap();
    for lamports in [1,u64::MAX] {
        f.accounts[CONFIG].lamports=lamports; assert_eq!(run(&mut f).unwrap(),baseline);
    }
    // An approved extra account is message-bound, but model preparation makes
    // no actual target-validity claim: even this malformed round stays untouched.
    f.accounts.push(BackingAccount { key:baseline.targets()[1].address(), owner:key(219), executable:true,
        signer:false,writable:false,lamports:999,data:vec![0xA5;3] });
    f.rebuild_message(); assert_eq!(run(&mut f).unwrap(),baseline);
}

#[test]
fn ordinary_host_and_native_entrypoint_cannot_execute_the_model_format() {
    let mut f=fixture(PROGRAM); let before=f.clone(); let data=f.inner_data.clone();
    f.with_infos(|accounts| {
        let _borrow=accounts[CONFIG].try_borrow_mut_data().unwrap();
        assert_eq!(prepare_approved_genesis_model(&PROGRAM,accounts,&data,roles()),Err(E::Authorization(A::HostRuntimeUnavailable)));
        assert_eq!(prepare_approved_genesis_model(&PROGRAM,&[],&[],roles()),Err(E::Authorization(A::HostRuntimeUnavailable)));
        assert_eq!(piv1::process_instruction(&PROGRAM,accounts,&data),Err(ProgramError::InvalidInstructionData));
        piv1::instruction_boundary::process_instruction_with_host_callbacks(&PROGRAM,accounts,&data,
            || panic!("model tag must not fetch Rent"),|_,_,_|panic!("model tag must not invoke"),
            |_|panic!("model tag must not emit")).unwrap_err();
    }); assert_eq!(f,before);
}

#[test]
fn modeled_state_envelopes_are_compatible_with_existing_fixed_and_guardian_authentication() {
    let mut f=fixture(PROGRAM); let m=run(&mut f).unwrap(); let rent=f.rent.clone();
    let mut accounts:Vec<_>=m.targets().iter().map(|t| {
        let data=match t.role() {
            R::Config=>StateEnvelope::config(m.proposed_config()).unwrap().as_bytes().to_vec(),
            R::ActiveDistribution=>StateEnvelope::distribution(m.proposed_distribution()).unwrap().as_bytes().to_vec(),
            R::GuardianRegistry=>StateEnvelope::registry(m.proposed_registry()).unwrap().as_bytes().to_vec(),
            R::GuardianReward(slot)=>StateEnvelope::reward(&m.proposed_rewards()[usize::from(slot)]).unwrap().as_bytes().to_vec(),
            R::PrincipalJito|R::PendingJito=>{
                let token=TokenAccount { mint:m.proposed_config().jitosol_mint,owner:m.piv_authority(),amount:0,
                    delegate:COption::None,state:AccountState::Initialized,is_native:COption::None,
                    delegated_amount:0,close_authority:COption::None };
                let mut bytes=vec![0;TokenAccount::LEN]; TokenAccount::pack(token,&mut bytes).unwrap(); bytes
            },
            _=>vec![],
        };
        assert_eq!(data.len(),t.size());
        BackingAccount { key:t.address(),owner:t.owner(),executable:false,signer:false,writable:false,
            lamports:rent.minimum_balance(data.len()),data }
    }).collect();
    let mut clock=BackingAccount { key:sysvar::clock::ID,owner:sysvar::ID,executable:false,
        signer:false,writable:false,lamports:1,data:vec![0;40] };
    f.clock.to_account_info(&mut clock.info()).unwrap(); accounts.push(clock);
    let before=accounts.clone();
    {
        let infos:Vec<AccountInfo<'_>>=accounts.iter_mut().map(BackingAccount::info).collect();
        let fixed=authenticate_fixed_accounts(&PROGRAM,&rent,FixedAccountInfos {
            config:&infos[0],active_distribution:&infos[1],pending_sol:&infos[9],principal_sol:&infos[10],
            operational_sol:&infos[11],distribution_escrow:&infos[12],kif_sol:&infos[13],principal_jito:&infos[14],pending_jito:&infos[15],
        }).unwrap();
        assert_eq!(fixed.config(),m.proposed_config()); assert!(fixed.economic_observation().is_ok());
        let guardians=authenticate_guardian_clock_snapshot(&PROGRAM,&rent,GuardianClockAccountInfos {
            config:&infos[0],guardian_registry:&infos[2],rewards:[&infos[3],&infos[4],&infos[5],&infos[6],&infos[7],&infos[8]],clock:&infos[16],
        }).unwrap();
        assert_eq!(guardians.registry(),m.proposed_registry()); assert_eq!(guardians.rewards(),m.proposed_rewards());
        assert_eq!((guardians.active_count(),guardians.activity_bitmap()),(0,0));
        assert_eq!(guardians.period(),m.modeled_period());
    }
    assert_eq!(accounts,before,"modeled initialized fixture remained unchanged");
}
