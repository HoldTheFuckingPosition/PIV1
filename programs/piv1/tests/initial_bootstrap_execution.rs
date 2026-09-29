//! Host current-account valuation and signed-custody oracles, not VM rollback.
mod support;
#[path = "support/pending_custody.rs"]
pub mod pending_custody;
#[path = "support/economic_custody.rs"]
mod custody;
#[path = "support/jito_identity_oracle.rs"]
mod oracle;
#[path = "support/bootstrap_custody.rs"]
mod bootstrap_custody;
use support::kif_claim_custody;
use anchor_lang::{prelude::{AccountInfo, Pubkey}, solana_program::{entrypoint::ProgramResult,
    instruction::{AccountMeta, Instruction}, program_error::ProgramError, system_program}};
use piv1::{accounts::authenticate_fixed_accounts, errors::Piv1Error,
    events::InitialContributionsBootstrapped, initial_bootstrap_execution::process_instruction_with_host_callbacks as execute,
    instruction_boundary::{process_instruction, process_instruction_with_host_callbacks as claim_seam},
    instruction_errors::{piv1_error_code, HOST_RUNTIME_UNAVAILABLE_CODE},
    instructions::bootstrap_initial_contributions::*, integrations::{self, FeeFraction, StakePoolAdapter},
    state::{bootstrap_initial_contributions, DistributionLifecycle}};
use bootstrap_custody::{Fixture, *};
use custody::{Fixture as Custody, *};
use support::{kif_claim_custody::{key, PROGRAM}, vault_custody_model::World};

fn err(value: Piv1Error) -> ProgramError { ProgramError::Custom(piv1_error_code(value)) }
fn plans(f: &Fixture) -> Vec<(usize, u64)> {
    let c = f.custody.config(); [(PENDING_SOL, c.accounted_pending_sol_lamports),
        (PENDING_JITO, c.accounted_pending_jitosol_units)].into_iter().filter(|(_, v)| *v != 0).collect()
}
fn transfer(source: usize, amount: u64, ix: &Instruction, a: &[AccountInfo<'_>], signers: &[&[&[u8]]]) -> ProgramResult {
    let seed: &[u8] = if source == PENDING_SOL { b"pending-sol" } else { b"authority" };
    let (signer, bump) = Pubkey::find_program_address(&[seed], &FIXTURE_PROGRAM);
    assert_eq!(signers, &[&[seed, &[bump]][..]]);
    assert_eq!(Pubkey::create_program_address(signers[0], &PROGRAM).unwrap(), signer);
    if source == PENDING_SOL {
        assert_eq!(a.len(), 3); assert_eq!(*a[0].key, signer); assert_eq!(*a[2].key, system_program::ID);
        let mut expected = vec![2, 0, 0, 0]; expected.extend_from_slice(&amount.to_le_bytes());
        assert_eq!(ix.data, expected); assert_eq!(ix.program_id, system_program::ID);
        assert_eq!(ix.accounts, vec![AccountMeta::new(signer, true), AccountMeta::new(*a[1].key, false)]);
        **a[0].try_borrow_mut_lamports()? -= amount; **a[1].try_borrow_mut_lamports()? += amount; Ok(())
    } else {
        assert_eq!(source, PENDING_JITO); assert_eq!(a.len(), 5); assert_eq!(*a[3].key, signer); assert_eq!(*a[4].key, spl_token::ID);
        let mut expected = vec![12]; expected.extend_from_slice(&amount.to_le_bytes()); expected.push(9);
        assert_eq!(ix.data, expected); assert_eq!(ix.program_id, spl_token::ID);
        assert_eq!(ix.accounts, vec![AccountMeta::new(*a[0].key, false), AccountMeta::new_readonly(*a[1].key, false),
            AccountMeta::new(*a[2].key, false), AccountMeta::new_readonly(signer, true)]);
        let mut inner = a[..4].to_vec(); inner[3].is_signer = true;
        spl_token::processor::Processor::process(&spl_token::ID, &inner, &ix.data)
    }
}
fn raw_prefix(base: &Custody, moves: &[(usize, u64)], count: usize) -> Custody {
    let mut expected = base.clone();
    for &(source, amount) in &moves[..count] {
        if source == PENDING_SOL {
            expected.accounts[PENDING_SOL].lamports -= amount; expected.accounts[PRINCIPAL_SOL].lamports += amount;
        } else {
            expected.set_token_units(PENDING_JITO, expected.token_units(PENDING_JITO) - amount);
            expected.set_token_units(PRINCIPAL_JITO, expected.token_units(PRINCIPAL_JITO) + amount);
        }
    }
    expected
}
fn success(f: &mut Fixture) -> u64 {
    let c = f.custody.config(); let moves = plans(f);
    let token_value = if f.pool.pool_token_supply == 0 { 0 } else {
        u64::try_from(u128::from(c.accounted_pending_jitosol_units) * u128::from(f.pool.total_lamports)
            / u128::from(f.pool.pool_token_supply)).unwrap()
    };
    let value = c.accounted_pending_sol_lamports.checked_add(token_value).unwrap();
    let mut expected = raw_prefix(&f.custody, &moves, moves.len());
    expected.edit_config(|after| {
        after.accounted_pending_sol_lamports = 0; after.accounted_pending_jitosol_units = 0;
        after.accounted_historical_sol_lamports = c.accounted_pending_sol_lamports;
        after.accounted_historical_jitosol_units = c.accounted_pending_jitosol_units;
        after.protected_principal_hwm_lamports = value; after.cumulative_contribution_value_lamports = value;
    });
    let event = InitialContributionsBootstrapped { config: f.custody.accounts[CONFIG].key,
        integrated_sol_lamports: c.accounted_pending_sol_lamports,
        integrated_jitosol_units: c.accounted_pending_jitosol_units, contribution_value_lamports: value };
    let rent = f.custody.rent.clone(); let clock = f.clock.clone(); let mut events = vec![]; let mut count = 0;
    f.custody.with_infos(|a| execute(&PROGRAM, a, &INITIAL_BOOTSTRAP_DATA, || Ok(clock), || Ok(rent), |ix, infos, seeds| {
        let (source, amount) = moves[count]; count += 1; transfer(source, amount, ix, infos, seeds)
    }, |e| events.push(e))).unwrap();
    assert_eq!(count, moves.len()); assert_eq!(events, vec![event]); assert_eq!(f.custody, expected); value
}
fn reject(mut f: Fixture, expected: Option<ProgramError>) {
    let before = f.clone(); let rent = f.custody.rent.clone(); let clock = f.clock.clone();
    let result = f.custody.with_infos(|a| execute(&PROGRAM, a, &INITIAL_BOOTSTRAP_DATA, || Ok(clock), || Ok(rent),
        |_, _, _| panic!("must reject before effects"), |_| panic!("failed bootstrap event")));
    if let Some(expected) = expected { assert_eq!(result, Err(expected)); } else { assert!(result.is_err()); }
    assert_eq!(f, before);
}

#[test]
fn both_topologies_sol_token_mixed_bootstrap_change_only_initial_principal_and_reject_replay() {
    for shared in [false, true] { for (sol, tokens) in [(700, 0), (0, 100), (700, 100)] {
        let mut f = Fixture::new(sol, tokens, shared); assert_eq!(f.custody.accounts.len(), if shared {18} else {19});
        f.custody.edit_config(|c| { c.next_distribution_sequence = u64::MAX; c.kif_anchor_timestamp = -2_592_000; });
        assert_eq!(success(&mut f), sol + tokens * 101 / 100);
        reject(f, Some(err(Piv1Error::InvalidBootstrapState)));
    } }
}

#[test]
fn stored_denominator_supports_burn_lag_appreciation_zero_floor_and_integer_limits() {
    for (total, recorded, minted, tokens, expected) in [
        (11, 10, 5, 3, 3), (11, 5, 5, 3, 6), (21, 10, 10, 3, 6), (1, 10, 10, 1, 0),
        (u64::MAX, u64::MAX, u64::MAX, u64::MAX, u64::MAX),
    ] {
        let mut f = Fixture::new(0, tokens, false); f.pool.total_lamports = total;
        f.pool.pool_token_supply = recorded; f.mint.supply = minted; f.sync();
        assert_eq!(success(&mut f), expected); reject(f, Some(err(Piv1Error::InvalidBootstrapState)));
    }
    let mut empty_pool = Fixture::new(7, 0, true); empty_pool.pool.total_lamports = 0;
    empty_pool.pool.pool_token_supply = 0; empty_pool.mint.supply = 0; empty_pool.sync();
    assert_eq!(success(&mut empty_pool), 7);
}

#[test]
fn current_epoch_supply_consistency_and_value_overflow_reject_before_movement() {
    for case in 0..7 {
        let mut f = Fixture::new(7, 3, false); f.pool.total_lamports = 11; f.pool.pool_token_supply = 10; f.mint.supply = 10;
        match case { 0 => f.pool.last_update_epoch -= 1, 1 => f.pool.last_update_epoch += 1,
            2 => f.mint.supply = 11, 3 => f.mint.supply = 2, 4 => f.pool.total_lamports = 0,
            5 => { f.pool.pool_token_supply = 0; f.mint.supply = 0; },
            _ => { f.pool.total_lamports = 0; f.pool.pool_token_supply = 0; f.mint.supply = 0; } }
        f.sync(); reject(f, Some(err(Piv1Error::InvalidCustodyObservation)));
    }
    let mut f = Fixture::new(1, 2, false); f.pool.total_lamports = u64::MAX;
    f.pool.pool_token_supply = 2; f.mint.supply = 2; f.sync(); reject(f, Some(err(Piv1Error::ArithmeticOverflow)));
    let mut f = Fixture::new(1, 0, false); f.custody.accounts[PRINCIPAL_SOL].lamports = u64::MAX;
    reject(f, Some(err(Piv1Error::ArithmeticOverflow)));
}

#[test]
fn only_zero_history_initial_idle_can_bootstrap_even_after_successful_normalization() {
    reject(Fixture::new(0, 0, false), Some(err(Piv1Error::ZeroContribution)));
    let mut f = Fixture::new(7, 3, false); f.custody.edit_config(|c| c.paused = true);
    reject(f, Some(err(Piv1Error::PausedOperation)));
    for field in 0..18 {
        let mut f = Fixture::new(7, 3, false); f.custody.edit_config(|c| match field {
            0 => c.last_successful_preparation_at = Some(1), 1 => c.last_valid_insufficient_attempt_at = Some(1),
            2 => c.protected_principal_hwm_lamports = 1, 3 => c.accounted_historical_sol_lamports = 1,
            4 => c.accounted_historical_jitosol_units = 1, 5 => c.next_cycle_yield_lamports = 1,
            6 => { c.kif_claim_liability_lamports = 1; c.cumulative_kif_credited_lamports = 1; },
            7 => c.collective_kif_carry_lamports = 1, 8 => c.cumulative_contribution_value_lamports = 1,
            9 => c.cumulative_gross_yield_lamports = 1, 10 => c.cumulative_htfp_paid_lamports = 1,
            11 => c.cumulative_team_owner_paid_lamports = 1,
            12 => { c.cumulative_kif_credited_lamports = 1; c.cumulative_kif_claimed_lamports = 1; },
            13 => c.cumulative_permanent_compound_lamports = 1, 14 => c.cumulative_retained_dust_lamports = 1,
            15 => c.cumulative_zero_active_kif_compound_lamports = 1, 16 => c.cumulative_cooldown_yield_recorded_lamports = 1,
            _ => c.cumulative_kif_claimed_lamports = 1 });
        // Back legitimate historical obligations where applicable, so custody
        // shortage is not the sole reason these noninitial states reject.
        match field { 3 | 5 => f.custody.accounts[PRINCIPAL_SOL].lamports += 1,
            4 => f.custody.set_token_units(PRINCIPAL_JITO, 1),
            6 | 7 => f.custody.accounts[KIF_SOL].lamports += 1, _ => {} }
        reject(f, None);
    }
    let mut w = World::new(10_000, 50, 0, 9, 3, FeeFraction::ZERO, 100_000);
    w.open(900_000).unwrap(); reject(Fixture::from_world(&w, false), None);
    w.settle().unwrap(); reject(Fixture::from_world(&w, false), Some(err(Piv1Error::InvalidLifecycle)));
    w.integrate(900_100).unwrap(); reject(Fixture::from_world(&w, false), Some(err(Piv1Error::InvalidBootstrapState)));
    let mut loss = World::new(1_000, 50, 0, 9, 3, FeeFraction::ZERO, 100_000);
    loss.open(900_000).unwrap(); let leg = loss.initiate(1).unwrap();
    loss.pool.decrease_exchange_rate(loss.pool.raw_snapshot().total_pool_lamports - 1).unwrap();
    loss.advance_epoch().unwrap(); loss.finalize(leg).unwrap();
    assert_eq!(loss.round.lifecycle, DistributionLifecycle::RecoveryRequired); reject(Fixture::from_world(&loss, true), None);
}

#[test]
fn economic_surplus_needs_prior_real_handler_normalization_and_native_quarantine_is_preserved() {
    for index in [PENDING_SOL, PRINCIPAL_SOL, ESCROW_SOL, KIF_SOL, PRINCIPAL_JITO, PENDING_JITO] {
        let mut f = Fixture::new(7, 3, false);
        if index == PRINCIPAL_JITO || index == PENDING_JITO { f.custody.set_token_units(index, f.custody.token_units(index) + 1); }
        else { f.custody.accounts[index].lamports += 1; }
        reject(f, Some(err(Piv1Error::InvalidCustodyObservation)));
    }
    for token in [false, true] {
        let mut f = Fixture::new(7, 3, false);
        if token { f.custody.set_token_units(PENDING_JITO, 2); } else { f.custody.accounts[PENDING_SOL].lamports -= 1; }
        reject(f, Some(err(Piv1Error::EconomicCustodyDeficit)));
    }
    let mut f = Fixture::new(7, 3, false); f.custody.donate([2, 3, 5, 7, 11, 13]);
    f.custody.accounts[PRINCIPAL_JITO].lamports += 1; f.custody.accounts[PENDING_JITO].lamports += 999;
    f.custody.accounts[OPERATIONAL_SOL].lamports += 123;
    let rent = f.custody.rent.clone();
    f.custody.with_infos(|a| {
        let observation = authenticate_fixed_accounts(&PROGRAM, &rent, custody::roles(a)).unwrap();
        assert_eq!(observation.economic_observation(), Err(Piv1Error::UnsupportedTokenNativeExcess));
        observation.initial_bootstrap_observation().unwrap();
    });
    let rent = f.custody.rent.clone(); let mut count = 0;
    f.custody.with_infos(|a| piv1::economic_normalization::process_instruction_with_host_callbacks(&PROGRAM, &a[..13],
        &piv1::instructions::reconcile_untracked_balances::RECONCILE_UNTRACKED_DATA, || Ok(rent), |ix, infos, signers| {
            let (seed, amount): (&[u8], u64) = [(b"principal-sol".as_slice(), 3), (b"distribution-escrow".as_slice(), 5),
                (b"kif-sol".as_slice(), 7), (b"authority".as_slice(), 13)][count]; count += 1;
            let (key, bump) = Pubkey::find_program_address(&[seed], &PROGRAM); assert_eq!(signers, &[&[seed, &[bump]][..]]);
            if ix.program_id == system_program::ID {
                assert_eq!(*infos[0].key, key); **infos[0].try_borrow_mut_lamports()? -= amount; **infos[1].try_borrow_mut_lamports()? += amount; Ok(())
            } else {
                assert_eq!(*infos[3].key, key); let mut inner = infos[..4].to_vec(); inner[3].is_signer = true;
                spl_token::processor::Processor::process(&spl_token::ID, &inner, &ix.data)
            }
        }, |_| {})).unwrap(); assert_eq!(count, 4);
    assert_eq!(f.custody.config().accounted_pending_sol_lamports, 24);
    assert_eq!(f.custody.config().accounted_pending_jitosol_units, 27);
    assert_eq!(success(&mut f), 51);
}

#[test]
fn protocol_accounts_and_config_selected_topology_cannot_be_substituted() {
    for shared in [false, true] {
        let base = Fixture::new(7, 3, shared);
        for index in [PROTOCOL, POOL, LIST, RESERVE, MINT, MANAGER] {
            let mut f = base.clone(); f.custody.accounts[index].key = key(240); reject(f, Some(ProgramError::Custom(6107)));
            let mut f = base.clone(); f.custody.accounts[index].owner = key(240); reject(f, Some(ProgramError::Custom(if index == PROTOCOL {6112} else {6109})));
        }
        let mut f = base.clone(); f.pool.validator_list = key(240); f.sync(); reject(f, Some(ProgramError::Custom(6113)));
        let mut f = base.clone(); f.mint.decimals = 8; f.sync(); reject(f, Some(ProgramError::Custom(6117)));
        let mut f = base.clone(); f.custody.accounts[PROTOCOL].executable = false; reject(f, Some(ProgramError::Custom(6110)));
        let mut f = base.clone(); f.custody.accounts[AUTHORITY].owner = spl_token::ID; reject(f, Some(err(Piv1Error::InvalidAccountOwner)));
        let mut f = base.clone(); f.custody.accounts[TOKEN].executable = false; reject(f, Some(err(Piv1Error::InvalidProgramIdentity)));
        let mut f = base.clone(); f.custody.accounts[SYSTEM].key = key(240); reject(f, Some(err(Piv1Error::InvalidProgramIdentity)));
        let mut f = base.clone(); f.custody.accounts[MINT].lamports = 1; reject(f, Some(err(Piv1Error::AccountRentDeficit)));
    }
    let mut short = Fixture::new(7, 3, false); short.custody.accounts.pop(); reject(short, Some(ProgramError::NotEnoughAccountKeys));
    let mut long = Fixture::new(7, 3, true); let mut extra = long.custody.accounts[MANAGER].clone(); extra.key = key(240);
    long.custody.accounts.push(extra); reject(long, Some(ProgramError::InvalidArgument));
    let mut wrong = Fixture::new(7, 3, false); wrong.custody.accounts[REFERRER].key = key(240);
    reject(wrong, Some(ProgramError::Custom(6107)));
}

#[test]
fn abi_context_native_guards_and_later_borrow_write_alias_checks_precede_cpi() {
    let mut f = Fixture::new(7, 3, false); let before = f.clone(); assert_eq!(&INITIAL_BOOTSTRAP_DATA, b"PIV1IB01\x01");
    for kind in 0..4 {
        let mut data = INITIAL_BOOTSTRAP_DATA.to_vec(); match kind { 0 => data[0] ^= 1, 1 => data[8] = 2,
            2 => { data.pop(); }, _ => data.push(0) }
        f.custody.with_infos(|a| assert_eq!(execute(&PROGRAM, a, &data, || panic!(), || panic!(), |_, _, _| panic!(), |_| panic!()), Err(ProgramError::InvalidInstructionData)));
    }
    f.custody.with_infos(|a| {
        assert_eq!(process_instruction(&PROGRAM, a, &INITIAL_BOOTSTRAP_DATA), Err(ProgramError::Custom(HOST_RUNTIME_UNAVAILABLE_CODE)));
        assert_eq!(claim_seam(&PROGRAM, a, &INITIAL_BOOTSTRAP_DATA, || panic!(), |_, _, _| panic!(), |_| panic!()), Err(ProgramError::Custom(HOST_RUNTIME_UNAVAILABLE_CODE)));
        assert_eq!(execute(&PROGRAM, a, &INITIAL_BOOTSTRAP_DATA, || Err(ProgramError::Custom(901)), || panic!(), |_, _, _| panic!(), |_| panic!()), Err(ProgramError::Custom(901)));
    }); assert_eq!(f, before);
    for index in [CONFIG, PENDING_SOL, PRINCIPAL_SOL, PENDING_JITO, PRINCIPAL_JITO] {
        let mut f = before.clone(); f.custody.accounts[index].writable = false; reject(f, Some(err(Piv1Error::AccountNotWritable)));
    }
    for index in [CONFIG, PENDING_SOL, PRINCIPAL_SOL, PENDING_JITO, PRINCIPAL_JITO, MINT] {
        let mut f = before.clone(); let rent = f.custody.rent.clone(); let clock = f.clock.clone();
        f.custody.with_infos(|a| { let _held = a[index].try_borrow_data().unwrap();
            assert_eq!(execute(&PROGRAM, a, &INITIAL_BOOTSTRAP_DATA, || Ok(clock), || Ok(rent), |_, _, _| panic!(), |_| panic!()), Err(err(Piv1Error::AccountBorrowFailed)));
        }); assert_eq!(f, before);
    }
    for kind in 0..3 {
        let mut f = before.clone(); let rent = f.custody.rent.clone(); let clock = f.clock.clone();
        f.custody.with_infos(|a| { let mut a = a.to_vec(); match kind { 0 => a[7].key = a[8].key,
            1 => a[7].data = a[8].data.clone(), _ => a[7].lamports = a[8].lamports.clone() }
            assert_eq!(execute(&PROGRAM, &a, &INITIAL_BOOTSTRAP_DATA, || Ok(clock), || Ok(rent), |_, _, _| panic!(), |_| panic!()), Err(err(Piv1Error::AccountAlias)));
        }); assert_eq!(f, before);
    }
}

#[test]
fn legacy_pool_snapshot_still_validates_unused_fields_at_original_error_position() {
    let mut w = World::empty(3, 71); w.explicit_tokens(3, 3).unwrap();
    let before = w.observation(); let mut after = before; after.pending_jitosol_units = 0; after.principal_jitosol_units = 3;
    for kind in 0..4 {
        let mut pool = w.pool.pool_snapshot().unwrap(); match kind { 0 => pool.minimum_delegation_lamports = 0,
            1 => pool.available_withdrawal_lamports = pool.total_pool_lamports + 1,
            2 => pool.sol_deposit_fee = FeeFraction { numerator: 0, denominator: 0 },
            _ => pool.stake_withdrawal_fee = FeeFraction { numerator: 1, denominator: 1 } }
        let mut c = w.config.clone(); let saved = c.clone();
        assert_eq!(bootstrap_initial_contributions(&mut c, &w.round, pool, before, after), Err(Piv1Error::InvalidCustodyObservation)); assert_eq!(c, saved);
        c.paused = true; let saved = c.clone();
        assert_eq!(bootstrap_initial_contributions(&mut c, &w.round, pool, before, after), Err(Piv1Error::PausedOperation)); assert_eq!(c, saved);
        c.paused = false; c.accounted_pending_jitosol_units = 0; let saved = c.clone();
        assert_eq!(bootstrap_initial_contributions(&mut c, &w.round, pool, before, after), Err(Piv1Error::ZeroContribution)); assert_eq!(c, saved);
    }
}

#[test]
fn each_cpi_error_preserves_exact_raw_prefix_without_config_or_event_then_modeled_retry() {
    for shared in [false, true] { let base = Fixture::new(7, 3, shared); let moves = plans(&base);
        for failed in 0..2 { for partial in [false, true] {
            let mut staged = base.clone(); let rent = staged.custody.rent.clone(); let clock = staged.clock.clone(); let mut count = 0;
            assert_eq!(staged.custody.with_infos(|a| execute(&PROGRAM, a, &INITIAL_BOOTSTRAP_DATA, || Ok(clock), || Ok(rent), |ix, infos, seeds| {
                let current = count; count += 1; let (source, amount) = moves[current];
                if current == failed && !partial { return Err(ProgramError::Custom(9123)); }
                transfer(source, amount, ix, infos, seeds)?;
                if current == failed { Err(ProgramError::Custom(9123)) } else { Ok(()) }
            }, |_| panic!())), Err(ProgramError::Custom(9123)));
            assert_eq!(count, failed + 1); assert_eq!(staged.custody, raw_prefix(&base.custody, &moves, failed + usize::from(partial)));
            let mut retry = base.clone(); success(&mut retry);
        } }
    }
}

#[test]
fn per_cpi_all_account_fingerprints_detect_only_the_intended_raw_tamper() {
    let base = Fixture::new(7, 3, false); let moves = plans(&base);
    for failed in 0..2 { for index in 0..19 {
        let mut staged = base.clone(); let rent = staged.custody.rent.clone(); let clock = staged.clock.clone(); let mut count = 0;
        assert_eq!(staged.custody.with_infos(|a| execute(&PROGRAM, a, &INITIAL_BOOTSTRAP_DATA, || Ok(clock), || Ok(rent), |ix, infos, seeds| {
            let current = count; count += 1; let (source, amount) = moves[current]; transfer(source, amount, ix, infos, seeds)?;
            if current == failed { **a[index].try_borrow_mut_lamports()? += 1; } Ok(())
        }, |_| panic!())), Err(err(Piv1Error::ContributionObservationMismatch)));
        assert_eq!(count, failed + 1); let mut expected = raw_prefix(&base.custody, &moves, failed + 1);
        expected.accounts[index].lamports += 1; assert_eq!(staged.custody, expected);
    } }
    for index in [CONFIG, ROUND, PRINCIPAL_JITO, PENDING_JITO, MINT, PROTOCOL, POOL, LIST, RESERVE, MANAGER, REFERRER] {
        let mut staged = base.clone(); let rent = staged.custody.rent.clone(); let clock = staged.clock.clone();
        assert_eq!(staged.custody.with_infos(|a| execute(&PROGRAM, a, &INITIAL_BOOTSTRAP_DATA, || Ok(clock), || Ok(rent), |ix, infos, seeds| {
            transfer(PENDING_SOL, 7, ix, infos, seeds)?; a[index].try_borrow_mut_data()?[0] ^= 1; Ok(())
        }, |_| panic!())), Err(err(Piv1Error::ContributionObservationMismatch)));
        let mut expected = raw_prefix(&base.custody, &moves, 1); expected.accounts[index].data[0] ^= 1; assert_eq!(staged.custody, expected);
    }
}
