#[allow(dead_code)]
#[path = "support/squads_invocation.rs"]
mod support;

use anchor_lang::{prelude::{Pubkey, SolanaSysvar}, AnchorDeserialize,
    solana_program::program_error::ProgramError};
use piv1::{
    constants::KIF_PERIOD_SECONDS,
    events::{GuardianHeartbeat, PauseChanged},
    guardian_operations::{self, GuardianOperationEvent as Event},
    instruction_boundary,
    instructions::{guardian_heartbeat::*, pause::*},
    squads_accounts::SQUADS_V4_PROGRAM_ID,
    squads_execution::ModeledSquadsInvocationContext,
    state::{open_distribution, ActiveDistribution, DistributionFunding, DistributionLifecycle,
        GuardianRegistry, GuardianReward, OpenDistributionInput, PivConfig},
};
use support::{key, BackingAccount, Fixture, PROGRAM, CONFIG, REGISTRY, CLOCK,
    PROGRAM_ACCOUNT, PROGRAM_DATA, MULTISIG, PROPOSAL, TRANSACTION, VAULT, INSTRUCTIONS};

fn envelope<T: anchor_lang::AnchorSerialize>(value: &T, discriminator: [u8; 8], size: usize) -> Vec<u8> {
    let mut bytes = discriminator.to_vec(); bytes.extend(value.try_to_vec().unwrap());
    assert!(bytes.len() <= size); bytes.resize(size, 0); bytes
}
fn decode<T: AnchorDeserialize>(data: &[u8]) -> T { T::deserialize(&mut &data[8..]).unwrap() }

#[derive(Clone, Debug, PartialEq)]
struct World { f: Fixture, indices: Vec<usize>, data: Vec<u8>, target: usize }

impl World {
    fn base() -> (Fixture, BackingAccount) {
        let mut f = Fixture::new();
        let mut config: PivConfig = decode(&f.accounts[CONFIG].data);
        config.kif_anchor_timestamp = 0;
        config.protected_principal_hwm_lamports = 1_000_000;
        config.accounted_historical_jitosol_units = 1000;
        config.accounted_historical_sol_lamports = 0;
        config.accounted_pending_sol_lamports = 8050;
        config.accounted_pending_jitosol_units = 73;
        config.next_cycle_yield_lamports = 0;
        config.collective_kif_carry_lamports = 19;
        config.kif_claim_liability_lamports = 1000;
        config.cumulative_kif_credited_lamports = 1300;
        config.cumulative_kif_claimed_lamports = 300;
        config.last_successful_preparation_at = None;
        config.last_valid_insufficient_attempt_at = None;
        let registry: GuardianRegistry = decode(&f.accounts[REGISTRY].data);
        let rewards: [GuardianReward; 6] = core::array::from_fn(|slot| {
            let mut reward: GuardianReward = decode(&f.accounts[9 + slot].data);
            reward.claimable_lamports = 10 + slot as u64;
            reward.cumulative_earned = 20 + slot as u64; reward.cumulative_claimed = 10;
            f.accounts[9 + slot].data = envelope(&reward, [169,109,89,17,75,171,105,39], 84); reward
        });
        assert_eq!(rewards.iter().map(|r| r.claimable_lamports).sum::<u64>(), 75);
        let mut round = ActiveDistribution::new_idle(config.bumps.active_distribution);
        let input = OpenDistributionInput { sequence: config.next_distribution_sequence,
            prepared_at: 100, prepared_slot: 10, prepared_epoch: 0,
            historical_jitosol_units: 1000, historical_sol_lamports: 0, historical_value_lamports: 1_010_000,
            snapshot_pool_total_lamports: 10_000_000, snapshot_pool_token_supply: 9_000_000,
            snapshot_withdrawal_fee_numerator: 1, snapshot_withdrawal_fee_denominator: 1000,
            gross_yield_lamports: 10_000, pending_sol_snapshot_lamports: 8050, pending_sol_used_lamports: 8050,
            snapshot_conversion_dust_lamports: 0, stored_residual_hwm_floor_lamports: 1_010_000,
            funding: DistributionFunding::Liquid { escrow_available_lamports: 8050 } };
        open_distribution(&mut config, &mut round, &registry, &rewards, input).unwrap();
        assert_eq!(round.lifecycle, DistributionLifecycle::EscrowFunded); round.validate().unwrap();
        let account = BackingAccount { key: config.active_distribution, owner: PROGRAM,
            executable: false, signer: false, writable: false, lamports: f.rent.minimum_balance(891),
            data: envelope(&round, [104,51,125,187,226,55,209,99], 891) };
        f.accounts[CONFIG].data = envelope(&config, [98,115,11,164,170,207,163,20], 1014);
        (f, account)
    }

    fn heartbeat(slot: u8, paused: bool) -> Self {
        let (mut f, round) = Self::base();
        let mut config: PivConfig = decode(&f.accounts[CONFIG].data); config.paused = paused;
        f.accounts[CONFIG].data = envelope(&config, [98,115,11,164,170,207,163,20], 1014);
        f.accounts[CONFIG].writable = false;
        let target = 9 + usize::from(slot); f.accounts[target].writable = true;
        let reward: GuardianReward = decode(&f.accounts[target].data);
        f.accounts.push(BackingAccount { key: reward.guardian, owner: key(231), executable: false,
            signer: true, writable: false, lamports: 17, data: vec![4, 8, 15] });
        let indices = vec![CONFIG, REGISTRY, 9,10,11,12,13,14,CLOCK,16,PROGRAM_ACCOUNT,PROGRAM_DATA,MULTISIG];
        let data = GuardianHeartbeatRequest { guardian_index: slot, vault_index: 7 }.encode().unwrap().to_vec();
        f.accounts.push(round); Self { f, indices, data, target }
    }

    fn pause(paused: bool) -> Self {
        let (mut f, round) = Self::base();
        let data = SetPauseRequest { vault_index: 7, paused }.encode().to_vec();
        f.inner_data = data.clone(); f.rebuild_message();
        f.accounts.push(round);
        Self { f, indices: (0..16).collect(), data, target: CONFIG }
    }

    fn context(&self) -> ModeledSquadsInvocationContext { ModeledSquadsInvocationContext {
        stack_height: self.f.stack_height, clock: self.f.clock.clone(), rent: self.f.rent.clone() } }
    fn set_time(&mut self, timestamp: i64) {
        self.f.clock.unix_timestamp = timestamp;
        self.f.clock.to_account_info(&mut self.f.accounts[CLOCK].info()).unwrap();
    }
    fn run(&mut self) -> (Result<(), ProgramError>, Vec<Event>) {
        let context = self.context(); let program = self.f.program;
        let indices = self.indices.clone(); let data = self.data.clone(); let mut events = vec![];
        let result = self.f.with_infos(|all| {
            let accounts = indices.iter().map(|i| all[*i].clone()).collect::<Vec<_>>();
            guardian_operations::process_instruction_with_host_callbacks(&program, &accounts, &data,
                || Ok(context), |event| events.push(event))
        });
        (result, events)
    }
    fn reject(&mut self, expected: Option<ProgramError>) {
        let before = self.clone(); let (result, events) = self.run();
        assert!(result.is_err()); if let Some(error) = expected { assert_eq!(result, Err(error)); }
        assert!(events.is_empty()); assert_eq!(*self, before);
    }
}

#[test]
fn all_six_guardians_record_only_current_activity_preserving_liabilities_and_active_snapshot() {
    for slot in 0..6 { for paused in [false, true] {
        let mut w = World::heartbeat(slot, paused); let before = w.f.accounts.clone();
        let mut expected = before.clone(); let mut reward: GuardianReward = decode(&before[w.target].data);
        reward.last_active_period = Some(0);
        expected[w.target].data = envelope(&reward, [169,109,89,17,75,171,105,39], 84);
        let event = Event::Heartbeat(GuardianHeartbeat { guardian_registry: before[REGISTRY].key,
            guardian_reward: before[w.target].key, guardian: reward.guardian,
            registry_revision: reward.registry_revision, guardian_index: slot, period_id: 0 });
        assert_eq!(w.run(), (Ok(()), vec![event])); assert_eq!(w.f.accounts, expected);
        assert_eq!(w.run(), (Ok(()), vec![event]), "same-period activity is idempotent");
        assert_eq!(w.f.accounts, expected);
        let config: PivConfig = decode(&w.f.accounts[CONFIG].data);
        assert_eq!(config.kif_claim_liability_lamports, 1000, "historical global liability exceeds six current claims");
        let round: ActiveDistribution = decode(&w.f.accounts.last().unwrap().data);
        assert_eq!(round.kif_eligibility_bitmap, 0, "post-snapshot activity is not retroactive");
    } }
}

#[test]
fn heartbeat_uses_runtime_half_open_periods_and_rejects_regression_or_pre_anchor_time() {
    let mut w = World::heartbeat(0, false);
    for (time, period) in [(0,0), (KIF_PERIOD_SECONDS - 1,0), (KIF_PERIOD_SECONDS,1), (2*KIF_PERIOD_SECONDS,2)] {
        w.set_time(time); assert!(w.run().0.is_ok());
        let reward: GuardianReward = decode(&w.f.accounts[w.target].data);
        assert_eq!(reward.last_active_period, Some(period));
    }
    w.set_time(KIF_PERIOD_SECONDS - 1); w.reject(Some(ProgramError::Custom(6037)));
    let mut early = World::heartbeat(0, true); early.set_time(-1);
    early.reject(Some(ProgramError::Custom(6037)));
}

#[test]
fn pause_and_unpause_change_exactly_one_config_byte_after_full_current_approval() {
    let mut w = World::pause(true);
    for desired in [true, true, false, false] {
        // Prepare a new exact approved request without adding the untouched round
        // to the fixed sixteen-account invocation profile.
        let round = w.f.accounts.pop().unwrap();
        w.data = SetPauseRequest { vault_index: 7, paused: desired }.encode().to_vec();
        w.f.inner_data = w.data.clone(); w.f.rebuild_message(); w.f.accounts.push(round);
        let before = w.f.accounts.clone(); let mut expected = before.clone();
        let previously_paused = before[CONFIG].data[10] == 1;
        expected[CONFIG].data[10] = u8::from(desired);
        assert_eq!(w.run(), (Ok(()), vec![Event::Pause(PauseChanged { config: before[CONFIG].key,
            multisig: before[MULTISIG].key, transaction_index: w.f.transaction.index,
            previously_paused, paused: desired })]));
        assert_eq!(w.f.accounts, expected);
        let config: PivConfig = decode(&w.f.accounts[CONFIG].data);
        assert_eq!(config.ensure_unpaused().is_ok(), !desired);
    }
}

#[test]
fn strict_wire_counts_and_ordinary_and_claim_host_guards_run_before_context_callbacks() {
    assert_eq!(GuardianHeartbeatRequest {guardian_index:5,vault_index:201}.encode().unwrap(),
        [b'P',b'I',b'V',b'1',b'H',b'B',b'0',b'1',1,5,201]);
    assert_eq!(SetPauseRequest {vault_index:201,paused:true}.encode(),
        [b'P',b'I',b'V',b'1',b'P',b'S',b'0',b'1',1,201,1]);
    for mut w in [World::heartbeat(0, false), World::pause(true)] {
        let program = w.f.program; let data = w.data.clone(); let indices = w.indices.clone();
        w.f.with_infos(|all| {
            let accounts = indices.iter().map(|i| all[*i].clone()).collect::<Vec<_>>();
            assert_eq!(instruction_boundary::process_instruction(&program, &accounts, &data), Err(ProgramError::Custom(6999)));
            assert_eq!(instruction_boundary::process_instruction_with_host_callbacks(&program, &accounts, &data,
                || panic!("claim context cannot execute guardian paths"), |_,_,_| panic!("no CPI"), |_| panic!("no claim event")),
                Err(ProgramError::Custom(6999)));
            let rejected = |accounts: &[_], data: &[u8], error| {
                assert_eq!(guardian_operations::process_instruction_with_host_callbacks(&program, accounts, data,
                    || panic!("decode/count must precede context"), |_| panic!("no failure event")), Err(error));
            };
            for size in 0..11 { rejected(&accounts, &data[..size], ProgramError::InvalidInstructionData); }
            let mut extra = data.clone(); extra.push(0); rejected(&accounts, &extra, ProgramError::InvalidInstructionData);
            for offset in [0, 7, 8, if indices.len()==13 {9} else {10}] {
                let mut invalid = data.clone(); invalid[offset] = 255;
                rejected(&accounts, &invalid, ProgramError::InvalidInstructionData);
            }
            for count in 0..accounts.len() { rejected(&accounts[..count], &data, ProgramError::NotEnoughAccountKeys); }
            let mut extra = accounts.clone(); extra.push(accounts[0].clone());
            rejected(&extra, &data, ProgramError::InvalidArgument);
        });
    }
}

#[test]
fn heartbeat_rejects_bad_signature_membership_envelopes_clock_rent_and_roles_without_events() {
    for mutation in 0..14 {
        let mut w = World::heartbeat(2, true);
        match mutation {
            0 => w.f.accounts[16].signer = false,
            1 => w.f.accounts[16].key = key(241),
            2 => w.f.accounts[w.target].owner = key(241),
            3 => w.f.accounts[w.target].key = key(241),
            4 => w.f.accounts[w.target].lamports = 0,
            5 => w.f.accounts[w.target].writable = false,
            6 => w.f.accounts[CLOCK].key = key(241),
            7 => w.f.accounts[CLOCK].owner = key(241),
            8 => w.f.clock.unix_timestamp += 1, // Supplied Clock differs from trusted runtime.
            9 => { let mut registry: GuardianRegistry = decode(&w.f.accounts[REGISTRY].data); registry.revision += 1;
                w.f.accounts[REGISTRY].data = envelope(&registry, [72,14,254,2,76,233,97,92], 210); }
            10 => { w.f.multisig.members[0].key = key(240); w.f.multisig.members.sort_by_key(|m| m.key); w.f.sync_multisig(); }
            11 => w.f.accounts[PROGRAM_DATA].data[13] ^= 1,
            12 => w.indices.swap(2,3),
            _ => w.f.accounts[w.target].data.push(0),
        }
        w.reject(None);
    }
}

#[test]
fn off_curve_guardian_can_sign_without_wallet_owner_or_data_constraints() {
    let mut w = World::heartbeat(2, false);
    let guardian = Pubkey::find_program_address(&[b"guardian-heartbeat-test"], &key(234)).0;
    assert!(!guardian.is_on_curve());
    let old = w.f.accounts[16].key;
    let mut registry: GuardianRegistry = decode(&w.f.accounts[REGISTRY].data); registry.guardian_keys[2] = guardian;
    w.f.accounts[REGISTRY].data = envelope(&registry, [72,14,254,2,76,233,97,92], 210);
    let mut reward: GuardianReward = decode(&w.f.accounts[w.target].data); reward.guardian = guardian;
    let (address,bump) = Pubkey::find_program_address(&[b"guardian-reward",guardian.as_ref(),
        &reward.registry_revision.to_le_bytes(), &[2]], &PROGRAM);
    reward.bump=bump; w.f.accounts[w.target].key=address;
    w.f.accounts[w.target].data=envelope(&reward,[169,109,89,17,75,171,105,39],84);
    w.f.multisig.members.iter_mut().find(|member|member.key==old).unwrap().key=guardian;
    w.f.multisig.members.sort_by_key(|member|member.key); w.f.sync_multisig(); w.f.accounts[16].key=guardian;
    assert!(w.run().0.is_ok());
}

#[test]
fn pause_authenticates_original_boolean_and_vault_witness_not_reconstructed_bytes() {
    let mut boolean = World::pause(true); boolean.data[10] = 0;
    boolean.reject(Some(ProgramError::Custom(6105)));
    let mut witness = World::pause(true);
    let round=witness.f.accounts.pop().unwrap();
    let (vault,bump)=Pubkey::find_program_address(&[b"multisig",witness.f.accounts[MULTISIG].key.as_ref(),b"vault",&[8]],&SQUADS_V4_PROGRAM_ID);
    witness.f.accounts[VAULT].key=vault; witness.f.accounts[PROGRAM_DATA].data[13..45].copy_from_slice(vault.as_ref());
    witness.f.transaction.vault_index=8; witness.f.transaction.vault_bump=bump;
    witness.f.rebuild_message(); witness.f.accounts.push(round); witness.data[9]=8;
    witness.reject(Some(ProgramError::Custom(6105)));
}

#[test]
fn pause_rejects_unapproved_stale_executed_or_mismatched_governance_before_any_write() {
    for mutation in 0..12 {
        let mut w=World::pause(true);
        match mutation {
            0=>{w.f.proposal.approved.truncate(3);w.f.sync_proposal();},
            1=>{w.f.proposal.status=5;w.f.sync_proposal();},
            2=>{w.f.multisig.stale_transaction_index=w.f.transaction.index;w.f.sync_multisig();},
            3=>{w.f.proposal.timestamp=w.f.clock.unix_timestamp;w.f.sync_proposal();},
            4=>w.f.stack_height=1,
            5=>w.f.accounts[VAULT].signer=false,
            6=>w.f.accounts[PROPOSAL].owner=key(240),
            7=>w.f.outer[0].program_id=key(240),
            8=>w.f.accounts[CONFIG].writable=false,
            9=>w.f.accounts[CONFIG].lamports=0,
            10=>w.f.accounts[INSTRUCTIONS].owner=key(240),
            _=>w.f.accounts[TRANSACTION].data[0]^=1,
        }
        if mutation==7 {w.f.sync_sysvar();}
        w.reject(None);
    }
}

#[test]
fn context_errors_and_commit_borrow_conflicts_preserve_all_accounts_and_emit_nothing() {
    for mut w in [World::heartbeat(1,false),World::pause(true)] {
        let before=w.clone(); let program=w.f.program; let data=w.data.clone(); let indices=w.indices.clone();
        for error in [ProgramError::UnsupportedSysvar,ProgramError::Custom(24001)] {
            w.f.with_infos(|all| {
                let accounts=indices.iter().map(|i|all[*i].clone()).collect::<Vec<_>>();
                assert_eq!(guardian_operations::process_instruction_with_host_callbacks(&program,&accounts,&data,
                    ||Err(error.clone()),|_|panic!("no event after failed context")),Err(error));
            });
            assert_eq!(w,before);
        }
        let context=w.context();let target=w.target;
        w.f.with_infos(|all| {
            let accounts=indices.iter().map(|i|all[*i].clone()).collect::<Vec<_>>();
            let _held=all[target].try_borrow_data().unwrap();
            assert_eq!(guardian_operations::process_instruction_with_host_callbacks(&program,&accounts,&data,
                ||Ok(context),|_|panic!("no event before complete commit")),Err(ProgramError::Custom(6019)));
        });
        assert_eq!(w,before);
    }
}

#[test]
fn runtime_id_and_host_backing_aliases_are_rejected_without_mutation_or_event() {
    for mut w in [World::heartbeat(0,false),World::pause(true)] {
        let original=w.f.program; w.f.program=key(239); w.reject(None); w.f.program=original;
        for lamports in [false,true] {
            let before=w.clone();let context=w.context();let data=w.data.clone();let indices=w.indices.clone();
            w.f.with_infos(|all| {
                let mut accounts=indices.iter().map(|i|all[*i].clone()).collect::<Vec<_>>();
                if lamports {accounts[1].lamports=accounts[0].lamports.clone();}
                else {accounts[1].data=accounts[0].data.clone();}
                assert_eq!(guardian_operations::process_instruction_with_host_callbacks(&original,&accounts,&data,
                    ||Ok(context),|_|panic!("no alias event")),Err(ProgramError::Custom(6017)));
            });assert_eq!(w,before);
        }
    }
}
