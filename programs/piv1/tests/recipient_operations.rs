//! Explicit modeled authority and whole-account recipient replacement oracles.
//! These are host tests, not live Squads/proposal consumption or Bank evidence.
#[allow(dead_code)]
#[path = "support/squads_invocation.rs"]
mod support;
use anchor_lang::{prelude::Pubkey, AnchorDeserialize,
    solana_program::{program_error::ProgramError, system_program}};
use piv1::{events::RecipientsUpdated, instruction_boundary,
    instructions::update_config::*, recipient_operations,
    squads_accounts::SQUADS_V4_PROGRAM_ID, squads_execution::ModeledSquadsInvocationContext,
    state::{open_distribution, ActiveDistribution, DistributionFunding,
        GuardianRegistry, GuardianReward, OpenDistributionInput, PivConfig}};
use support::{key, BackingAccount, Fixture, PROGRAM, CONFIG, REGISTRY,
    PROGRAM_DATA, MULTISIG, PROPOSAL, TRANSACTION, VAULT, INSTRUCTIONS};
fn envelope<T: anchor_lang::AnchorSerialize>(value: &T, discriminator: [u8; 8], size: usize) -> Vec<u8> {
    let mut bytes = discriminator.to_vec(); bytes.extend(value.try_to_vec().unwrap());
    assert!(bytes.len() <= size); bytes.resize(size, 0); bytes
}
fn decode<T: AnchorDeserialize>(data: &[u8]) -> T { T::deserialize(&mut &data[8..]).unwrap() }
fn vault(multisig: &Pubkey, index: u8) -> Pubkey {
    Pubkey::find_program_address(&[b"multisig", multisig.as_ref(), b"vault", &[index]], &SQUADS_V4_PROGRAM_ID).0
}
#[derive(Clone, Debug, PartialEq)]
struct World { f: Fixture, data: Vec<u8> }
impl World {
    fn base(active: bool) -> (Fixture, BackingAccount) {
        let mut f = Fixture::new();
        let mut config: PivConfig = decode(&f.accounts[CONFIG].data);
        config.htfp_recipient = vault(&f.accounts[MULTISIG].key, 1);
        config.team_owner_recipient = vault(&f.accounts[MULTISIG].key, 2);
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
        if active { open_distribution(&mut config, &mut round, &registry, &rewards, input).unwrap(); }
        round.validate().unwrap();
        let account = BackingAccount { key: config.active_distribution, owner: PROGRAM,
            executable: false, signer: false, writable: false, lamports: f.rent.minimum_balance(891),
            data: envelope(&round, [104,51,125,187,226,55,209,99], 891) };
        f.accounts[CONFIG].data = envelope(&config, [98,115,11,164,170,207,163,20], 1014);
        (f, account)
    }


    fn new(active: bool) -> Self {
        let (mut f, round) = Self::base(active);
        for index in [3, 4] {
            f.accounts.push(BackingAccount { key: vault(&f.accounts[MULTISIG].key, index),
                owner: system_program::ID, executable: false, signer: false, writable: false,
                lamports: f.rent.minimum_balance(0), data: vec![] });
        }
        let data = UpdateRecipientsRequest { vault_index: 7}.encode().to_vec();
        f.inner_data = data.clone(); f.rebuild_message(); f.accounts.push(round);
        Self { f, data }
    }
    fn context(&self) -> ModeledSquadsInvocationContext { ModeledSquadsInvocationContext {
        stack_height: self.f.stack_height, clock: self.f.clock.clone(), rent: self.f.rent.clone() } }
    fn approve(&mut self) {
        let round = self.f.accounts.pop().unwrap();
        self.f.inner_data = self.data.clone(); self.f.rebuild_message(); self.f.accounts.push(round);
    }
    fn run(&mut self) -> (Result<(), ProgramError>, Vec<RecipientsUpdated>) {
        let context = self.context(); let data = self.data.clone(); let program = self.f.program; let mut events = vec![];
        let result = self.f.with_infos(|accounts| recipient_operations::process_instruction_with_host_callbacks(
            &program, &accounts[..18], &data, || Ok(context), |event| events.push(event)));
        (result, events)
    }
    fn reject(&mut self, expected: Option<ProgramError>) {
        let before = self.clone(); let (result, events) = self.run();
        assert!(result.is_err()); if let Some(error) = expected { assert_eq!(result, Err(error)); }
        assert!(events.is_empty()); assert_eq!(*self, before);
    }
    fn success(&mut self) {
        let before = self.f.accounts.clone(); let mut expected = before.clone();
        let mut config: PivConfig = decode(&before[CONFIG].data);
        let event = RecipientsUpdated { config: before[CONFIG].key, multisig: before[MULTISIG].key,
            transaction_index: self.f.transaction.index, previous_htfp_recipient: config.htfp_recipient,
            previous_team_owner_recipient: config.team_owner_recipient,
            htfp_recipient: before[16].key, team_owner_recipient: before[17].key };
        config.htfp_recipient = before[16].key; config.team_owner_recipient = before[17].key;
        expected[CONFIG].data = envelope(&config, [98,115,11,164,170,207,163,20], 1014);
        assert_eq!(self.run(), (Ok(()), vec![event])); assert_eq!(self.f.accounts, expected);
    }
}

#[test]
fn only_future_config_destinations_change_with_active_round_and_historical_liabilities_preserved() {
    for active in [false, true] { for paused in [false, true] {
        let mut w = World::new(active);
        let mut config: PivConfig = decode(&w.f.accounts[CONFIG].data); config.paused = paused;
        w.f.accounts[CONFIG].data = envelope(&config, [98,115,11,164,170,207,163,20], 1014);
        let round_before = w.f.accounts[18].clone();
        w.success(); assert_eq!(w.f.accounts[18], round_before);
        // The host model retains Approved. A real Squads execution consumes the
        // proposal externally; this repeated no-op is not an effect-once proof.
        w.success(); assert_eq!(w.f.accounts[18], round_before);
        let updated: PivConfig = decode(&w.f.accounts[CONFIG].data);
        assert_eq!(updated.kif_claim_liability_lamports, 1000);
        if active {
            let frozen: ActiveDistribution = decode(&round_before.data);
            assert_eq!(frozen.htfp_recipient, config.htfp_recipient);
            assert_eq!(frozen.team_owner_recipient, config.team_owner_recipient);
            assert_ne!(frozen.htfp_recipient, updated.htfp_recipient);
        }
    }}
}

#[test]
fn a_new_preparation_after_an_idle_update_freezes_the_new_destinations() {
    let mut w = World::new(false); w.success();
    let mut config: PivConfig = decode(&w.f.accounts[CONFIG].data);
    let registry: GuardianRegistry = decode(&w.f.accounts[REGISTRY].data);
    let rewards: [GuardianReward; 6] = core::array::from_fn(|i| decode(&w.f.accounts[9+i].data));
    let mut round: ActiveDistribution = decode(&w.f.accounts[18].data);
    let input = OpenDistributionInput { sequence: config.next_distribution_sequence,
        prepared_at: 100, prepared_slot: 10, prepared_epoch: 0,
        historical_jitosol_units: 1000, historical_sol_lamports: 0, historical_value_lamports: 1_010_000,
        snapshot_pool_total_lamports: 10_000_000, snapshot_pool_token_supply: 9_000_000,
        snapshot_withdrawal_fee_numerator: 1, snapshot_withdrawal_fee_denominator: 1000,
        gross_yield_lamports: 10_000, pending_sol_snapshot_lamports: 8050, pending_sol_used_lamports: 8050,
        snapshot_conversion_dust_lamports: 0, stored_residual_hwm_floor_lamports: 1_010_000,
        funding: DistributionFunding::Liquid { escrow_available_lamports: 8050 } };
    open_distribution(&mut config, &mut round, &registry, &rewards, input).unwrap();
    assert_eq!(round.htfp_recipient, w.f.accounts[16].key);
    assert_eq!(round.team_owner_recipient, w.f.accounts[17].key);
}

#[test]
fn supported_existing_recipient_and_swap_are_allowed_without_extra_policy() {
    for indices in [[1, 2], [2, 1], [1, 4]] {
        let mut w = World::new(true);
        for (role, index) in [16,17].into_iter().zip(indices) {
            w.f.accounts[role].key = vault(&w.f.accounts[MULTISIG].key, index);
        }
        w.approve(); w.success();
    }
}

#[test]
fn strict_abi_account_count_and_host_routes_fail_before_any_context() {
    assert_eq!(UpdateRecipientsRequest {vault_index:7}.encode(),
        [b'P',b'I',b'V',b'1',b'U',b'R',b'0',b'2',2,7]);
    let old=[b'P',b'I',b'V',b'1',b'U',b'R',b'0',b'1',1,7,3,4];
    assert_eq!(UpdateRecipientsRequest::decode(&old),Err(ProgramError::InvalidInstructionData));
    assert_eq!(instruction_boundary::process_instruction(&PROGRAM,&[],&old),Err(ProgramError::InvalidInstructionData));
    let mut w = World::new(true); let before = w.clone(); let data = w.data.clone();
    w.f.with_infos(|all| {
        let a = &all[..18];
        assert_eq!(instruction_boundary::process_instruction(&PROGRAM, a, &data), Err(ProgramError::Custom(6999)));
        assert_eq!(instruction_boundary::process_instruction_with_host_callbacks(&PROGRAM, a, &data,
            || panic!("no claim context"), |_,_,_| panic!("no CPI"), |_| panic!("no event")), Err(ProgramError::Custom(6999)));
        let reject = |a: &[_], data: &[u8], error| assert_eq!(
            recipient_operations::process_instruction_with_host_callbacks(&PROGRAM, a, data,
                || panic!("ABI first"), |_| panic!("no event")), Err(error));
        for size in 0..10 { reject(a, &data[..size], ProgramError::InvalidInstructionData); }
        let mut extra = data.clone(); extra.push(0); reject(a, &extra, ProgramError::InvalidInstructionData);
        for offset in [0,7,8] { let mut bad = data.clone(); bad[offset] = 255;
            reject(a, &bad, ProgramError::InvalidInstructionData); }
        for size in 0..18 { reject(&a[..size], &data, ProgramError::NotEnoughAccountKeys); }
        reject(all, &data, ProgramError::InvalidArgument);
    }); assert_eq!(w, before);
}

#[test]
fn original_approved_witnesses_and_recipient_keys_are_required() {
    for offset in [9] {
        let mut w = World::new(true); w.data[offset] ^= 1; w.reject(None);
    }
    for role in [16,17] {
        let mut w = World::new(true); w.f.accounts[role].key = vault(&w.f.accounts[MULTISIG].key, 5);
        w.reject(Some(ProgramError::Custom(6105)));
    }
}

#[test]
fn unapproved_stale_executed_wrong_current_authority_and_untrusted_context_reject_exactly() {
    for mutation in 0..16 {
        let mut w = World::new(true);
        match mutation {
            0 => { w.f.proposal.approved.truncate(3); w.f.sync_proposal(); },
            1 => { w.f.proposal.status=5; w.f.sync_proposal(); },
            2 => { w.f.multisig.stale_transaction_index=w.f.transaction.index; w.f.sync_multisig(); },
            3 => { w.f.proposal.timestamp=w.f.clock.unix_timestamp; w.f.sync_proposal(); },
            4 => w.f.stack_height=1,
            5 => w.f.accounts[VAULT].signer=false,
            6 => w.f.accounts[PROPOSAL].owner=key(240),
            7 => { w.f.outer[0].program_id=key(240); w.f.sync_sysvar(); },
            8 => w.f.accounts[CONFIG].writable=false,
            9 => w.f.accounts[CONFIG].lamports=0,
            10 => w.f.accounts[INSTRUCTIONS].owner=key(240),
            11 => w.f.accounts[TRANSACTION].data[0]^=1,
            12 => w.f.accounts[PROGRAM_DATA].data[13]^=1,
            13 => { w.f.multisig.members[0].key=key(240); w.f.multisig.members.sort_by_key(|m|m.key); w.f.sync_multisig(); },
            14 => w.f.clock.unix_timestamp+=1,
            _ => w.f.program=key(240),
        }
        w.reject(None);
    }
}

#[test]
fn approved_addresses_accept_receiver_type_and_balance_changes_but_preserve_isolation() {
    for role in [16,17] { for mutation in 0..7 {
        let mut w = World::new(true);
        match mutation {
            0 => w.f.accounts[role].key=key(240),
            1 => w.f.accounts[role].key=vault(&key(240), 3),
            2 => w.f.accounts[role].owner=key(240),
            3 => w.f.accounts[role].data=vec![7,8,9],
            4 => w.f.accounts[role].lamports=0,
            5 => w.f.accounts[role].lamports=1,
            _ => {let config:PivConfig=decode(&w.f.accounts[CONFIG].data);w.f.accounts[role].key=config.piv_authority;},
        }
        w.approve(); if mutation == 6 {w.reject(None);} else {w.success();}
    }}
}

#[test]
fn alias_and_commit_or_recipient_borrow_conflicts_are_atomic_without_events() {
    for case in [0,1,2,3,6,7] {
        let mut w=World::new(true); let before=w.clone(); let context=w.context(); let data=w.data.clone();
        w.f.with_infos(|all| {
            let mut a=all[..18].to_vec();
            if case==0 {a[17]=a[16].clone();}
            if case==1 {a[17].data=a[16].data.clone();}
            if case==2 {a[17].lamports=a[16].lamports.clone();}
            let _config=if case==3 {Some(all[CONFIG].try_borrow_data().unwrap())}else{None};
            let _recipient=if case==4 {Some(all[16].try_borrow_mut_data().unwrap())}else{None};
            let _lamports=if case==5 {Some(all[17].try_borrow_mut_lamports().unwrap())}else{None};
            if case==6 {a[16].data=a[CONFIG].data.clone();}
            if case==7 {a[17].lamports=a[CONFIG].lamports.clone();}
            assert_eq!(recipient_operations::process_instruction_with_host_callbacks(&PROGRAM,&a,&data,
                ||Ok(context), |_|panic!("no event")), Err(ProgramError::Custom(if [0,1,2,6,7].contains(&case){6017}else{6019})));
        }); assert_eq!(w,before);
    }
}

#[test]
fn failed_runtime_context_is_propagated_without_effects() {
    let mut w=World::new(false); let before=w.clone(); let data=w.data.clone();
    w.f.with_infos(|all| assert_eq!(recipient_operations::process_instruction_with_host_callbacks(
        &PROGRAM,&all[..18],&data,||Err(ProgramError::UnsupportedSysvar), |_|panic!("no event")),Err(ProgramError::UnsupportedSysvar)));
    assert_eq!(w,before);
}

#[test]
fn approved_retired_reward_or_other_piv_owned_state_cannot_become_external_recipient() {
    for role in [16,17] {
        let mut w=World::new(true);w.f.accounts[role].key=Pubkey::find_program_address(&[b"guardian-reward",key(240).as_ref(),&9_u64.to_le_bytes(),&[0]],&PROGRAM).0;
        w.f.accounts[role].owner=PROGRAM;w.f.accounts[role].data=vec![1,2,3];w.approve();w.reject(Some(ProgramError::Custom(6017)));
    }
}
