//! Host protected-CPI composition and independent account-byte oracles.
//! The real Token processor runs; stake-pool effects and rollback are modeled.
mod support;
#[path = "support/pending_custody.rs"] pub mod pending_custody;
#[path = "support/economic_custody.rs"] mod custody;
#[path = "support/jito_identity_oracle.rs"] mod oracle;
#[path = "support/bootstrap_custody.rs"] mod bootstrap_custody;
#[path = "support/principal_deposit_custody.rs"] mod profile;
use support::kif_claim_custody;
use anchor_lang::{prelude::AccountInfo, solana_program::{instruction::{AccountMeta, Instruction},
    entrypoint::ProgramResult, program_error::ProgramError, system_program}, prelude::Pubkey};
use piv1::{accounts::authenticate_fixed_accounts, errors::Piv1Error, events::PendingSolStaked,
    instruction_boundary::{process_instruction, process_instruction_with_host_callbacks as claim_seam},
    instruction_errors::{piv1_error_code, HOST_RUNTIME_UNAVAILABLE_CODE},
    instructions::stake_pending_sol::*, integrations::{self, FeeFraction, StakePoolAdapter, SolDepositRequest},
    principal_deposit_execution::process_instruction_with_host_callbacks as execute,
    state::{record_protected_principal_deposit, PrincipalSolDepositObservation}};
use bootstrap_custody::{Fixture, POOL, PROTOCOL, LIST, RESERVE, MANAGER, REFERRER};
use custody::{Fixture as Custody, *};
use profile::{principal, from_world, refresh, completed_world};
use support::{kif_claim_custody::{key, PROGRAM}, vault_custody_model::World};

fn error(e: Piv1Error) -> ProgramError { ProgramError::Custom(piv1_error_code(e)) }
fn request(amount: u64, minimum: u64) -> [u8; 25] {
    StakePendingSolParameters { native_lamports: amount, caller_minimum_pool_tokens_out: minimum }.encode()
}
fn minted(f: &Fixture, amount: u64) -> u64 {
    if f.pool.total_lamports == 0 || f.pool.pool_token_supply == 0 { amount } else {
        u64::try_from(u128::from(amount) * u128::from(f.pool.pool_token_supply) / u128::from(f.pool.total_lamports)).unwrap()
    }
}
fn value(units: u64, total: u64, supply: u64) -> u64 {
    if supply == 0 { assert_eq!(units, 0); 0 } else { u64::try_from(u128::from(units) * u128::from(total) / u128::from(supply)).unwrap() }
}
fn raw_prefix(f: &Fixture, amount: u64, stage: usize) -> Custody {
    let mut expected = f.custody.clone(); let minted = minted(f, amount);
    if stage >= 1 { expected.accounts[PRINCIPAL_SOL].lamports -= amount; expected.accounts[RESERVE].lamports += amount; }
    if stage >= 2 {
        expected.set_token_units(PRINCIPAL_JITO, expected.token_units(PRINCIPAL_JITO) + minted);
        expected.accounts[MINT].data[36..44].copy_from_slice(&(f.mint.supply + minted).to_le_bytes());
    }
    if stage >= 3 {
        let mut pool = f.pool.clone(); pool.total_lamports += amount; pool.pool_token_supply += minted;
        let bytes = borsh1::to_vec(&pool).unwrap(); expected.accounts[POOL].data[..bytes.len()].copy_from_slice(&bytes);
    }
    expected
}
fn invoke(f: &Fixture, amount: u64, minimum: u64, stage: usize,
    ix: &Instruction, a: &[AccountInfo<'_>], seeds: &[&[&[u8]]]) -> ProgramResult
{
    assert_eq!(a.len(), 11); let referrer = if f.custody.accounts.len() == 19 { MANAGER } else { 18 };
    let roles = [POOL, f.custody.accounts.len()-1, RESERVE, PRINCIPAL_SOL, PRINCIPAL_JITO, MANAGER, referrer, MINT, SYSTEM, TOKEN, PROTOCOL];
    for (actual, index) in a.iter().zip(roles) { assert_eq!(*actual.key, f.custody.accounts[index].key); }
    let units = minted(f, amount); let c = f.custody.config();
    let floor = (u128::from(units) * u128::from(10_000 - c.configured_slippage_bps) / 10_000) as u64;
    let mut bytes = vec![25]; bytes.extend_from_slice(&amount.to_le_bytes()); bytes.extend_from_slice(&minimum.max(floor).to_le_bytes());
    assert_eq!(ix.data, bytes); assert_eq!(ix.program_id, f.custody.accounts[PROTOCOL].key);
    assert_eq!(ix.accounts, vec![AccountMeta::new(*a[0].key, false), AccountMeta::new_readonly(*a[1].key, false),
        AccountMeta::new(*a[2].key, false), AccountMeta::new(*a[3].key, true), AccountMeta::new(*a[4].key, false),
        AccountMeta::new(*a[5].key, false), AccountMeta::new(*a[6].key, false), AccountMeta::new(*a[7].key, false),
        AccountMeta::new_readonly(system_program::ID, false), AccountMeta::new_readonly(spl_token::ID, false)]);
    let (source, bump) = Pubkey::find_program_address(&[b"principal-sol"], &FIXTURE_PROGRAM);
    assert_eq!(seeds, &[&[b"principal-sol".as_slice(), &[bump]][..]]);
    assert_eq!(Pubkey::create_program_address(seeds[0], &PROGRAM).unwrap(), source); assert_eq!(*a[3].key, source);
    if stage >= 1 { **a[3].try_borrow_mut_lamports()? -= amount; **a[2].try_borrow_mut_lamports()? += amount; }
    if stage >= 2 {
        let (withdraw, withdraw_bump) = Pubkey::find_program_address(&[a[0].key.as_ref(), b"withdraw"], &ix.program_id);
        assert_eq!(*a[1].key, withdraw); assert_eq!(withdraw_bump, f.pool.stake_withdraw_bump_seed);
        let mut authority = a[1].clone(); authority.is_signer = true;
        let mut mint_to = vec![7]; mint_to.extend_from_slice(&units.to_le_bytes());
        spl_token::processor::Processor::process(&spl_token::ID, &[a[7].clone(), a[4].clone(), authority], &mint_to)?;
    }
    if stage >= 3 {
        let mut pool = f.pool.clone(); pool.total_lamports += amount; pool.pool_token_supply += units;
        let bytes = borsh1::to_vec(&pool).unwrap(); a[0].try_borrow_mut_data()?[..bytes.len()].copy_from_slice(&bytes);
    }
    Ok(())
}
fn success(f: &mut Fixture, amount: u64, minimum: u64) {
    let before = f.clone(); let c = before.custody.config(); let units = minted(&before, amount);
    let before_value = c.accounted_historical_sol_lamports + value(c.accounted_historical_jitosol_units, f.pool.total_lamports, f.pool.pool_token_supply);
    let after_value = c.accounted_historical_sol_lamports - amount
        + value(c.accounted_historical_jitosol_units + units, f.pool.total_lamports + amount, f.pool.pool_token_supply + units);
    let mut expected = raw_prefix(&before, amount, 3);
    expected.edit_config(|c| { c.accounted_historical_sol_lamports -= amount; c.accounted_historical_jitosol_units += units; });
    let mut events = vec![]; let mut count = 0; let rent = f.custody.rent.clone(); let clock = f.clock.clone();
    f.custody.with_infos(|a| execute(&PROGRAM, a, &request(amount, minimum), || Ok(clock), || Ok(rent), |ix, infos, seeds| {
        count += 1; invoke(&before, amount, minimum, 3, ix, infos, seeds)
    }, |event| events.push(event))).unwrap();
    assert_eq!(count, 1); assert_eq!(f.custody, expected);
    assert_eq!(events, vec![PendingSolStaked { config: c_key(&before), deposited_sol_lamports: amount,
        minted_jitosol_units: units, historical_value_before_lamports: before_value, historical_value_after_lamports: after_value }]);
    assert!(after_value >= before_value && after_value >= c.protected_principal_hwm_lamports);
    f.pool.total_lamports += amount; f.pool.pool_token_supply += units; f.mint.supply += units;
}
fn c_key(f: &Fixture) -> Pubkey { f.custody.accounts[CONFIG].key }
fn reject(mut f: Fixture, amount: u64, minimum: u64, expected: Option<ProgramError>) {
    let before = f.clone(); let clock = f.clock.clone(); let rent = f.custody.rent.clone();
    let result = f.custody.with_infos(|a| execute(&PROGRAM, a, &request(amount, minimum), || Ok(clock), || Ok(rent),
        |_, _, _| panic!("preflight must reject before CPI"), |_| panic!("failed event")));
    if let Some(expected) = expected { assert_eq!(result, Err(expected)); } else { assert!(result.is_err()); }
    assert_eq!(f, before);
}

#[test]
fn both_topologies_partial_repeated_and_full_conversions_preserve_every_account_byte() {
    for shared in [false, true] { for slippage in [0, 1] {
        let mut w = World::empty(3, 42); w.explicit_sol(1_414, 1_414).unwrap(); w.explicit_tokens(100, 100).unwrap(); w.bootstrap().unwrap();
        w.explicit_sol(29, 29).unwrap(); w.explicit_tokens(37, 37).unwrap();
        let mut f = from_world(&w, shared); f.custody.edit_config(|c| c.configured_slippage_bps = slippage);
        assert_eq!(f.custody.accounts.len(), if shared {19} else {20});
        for (amount, units) in [(101, 100), (202, 200), (404, 400), (707, 700)] { success(&mut f, amount, units); }
        assert_eq!(f.custody.config().accounted_historical_sol_lamports, 0);
        assert_eq!(f.custody.config().accounted_pending_sol_lamports, 29);
        assert_eq!(f.custody.config().accounted_pending_jitosol_units, 37);
        reject(f, 1, 0, Some(error(Piv1Error::PrincipalDepositExceedsQueue)));
    } }
}

#[test]
fn zero_fee_encodings_burn_lag_empty_pool_and_opaque_authority_are_supported() {
    for denominator in [0, 1, 99] {
        let mut f = principal(707, 100, false); f.pool.sol_deposit_fee.denominator = denominator;
        f.mint.supply = 1_000; refresh(&mut f);
        let last = f.custody.accounts.len()-1; f.custody.accounts[last].owner = key(244);
        f.custody.accounts[last].data = vec![0x74; 4096]; f.custody.accounts[last].lamports = 77;
        success(&mut f, 707, 700); assert_eq!(f.pool.pool_token_supply - f.mint.supply, 9_999_000);
    }
    let mut f = principal(707, 0, true); f.pool.total_lamports = 0; f.pool.pool_token_supply = 0; f.mint.supply = 0;
    refresh(&mut f); success(&mut f, 707, 707);
}

#[test]
fn rounding_fee_floor_and_hwm_shortfall_reject_without_using_other_categories() {
    reject(principal(700, 0, false), 700, 0, Some(error(Piv1Error::PrincipalDepositHistoricalValueLoss)));
    let mut exact = principal(707, 0, false); success(&mut exact, 707, 700);
    reject(principal(707, 0, false), 707, 701, Some(error(Piv1Error::PrincipalDepositMinimumNotMet)));
    reject(principal(1, 0, false), 1, 0, Some(error(Piv1Error::PrincipalDepositMinimumNotMet)));
    let mut f = principal(707, 100, false); f.pool.sol_deposit_fee = oracle::Fee { numerator: 1, denominator: 100 };
    f.pool.total_lamports += 100_000; refresh(&mut f);
    reject(f, 707, 0, Some(error(Piv1Error::UnsupportedPrincipalDepositFee)));
    let mut f = principal(707, 0, false); f.custody.edit_config(|c| c.protected_principal_hwm_lamports = 708);
    reject(f, 707, 0, Some(error(Piv1Error::HighWaterMarkDecrease)));
}

#[test]
fn completed_history_carry_liabilities_and_token_native_quarantine_remain_exact() {
    let mut f = from_world(&completed_world(), false);
    f.custody.edit_config(|c| c.next_cycle_yield_lamports += 31);
    f.custody.accounts[PRINCIPAL_SOL].lamports += 31;
    f.custody.accounts[PRINCIPAL_JITO].lamports += 123; f.custody.accounts[PENDING_JITO].lamports += 456;
    f.custody.accounts[OPERATIONAL_SOL].lamports += 789;
    let rent = f.custody.rent.clone(); f.custody.with_infos(|a| {
        let observed = authenticate_fixed_accounts(&PROGRAM, &rent, custody::roles(a)).unwrap();
        assert_eq!(observed.economic_observation(), Err(Piv1Error::UnsupportedTokenNativeExcess));
        observed.principal_deposit_observation().unwrap();
    }); success(&mut f, 101, 100);
    assert!(f.custody.config().cumulative_gross_yield_lamports > 0);
    assert!(f.custody.config().kif_claim_liability_lamports > 90);
}

#[test]
fn phase_pause_amount_and_individual_custody_guards_precede_effects() {
    let mut f = principal(707, 100, false); f.custody.edit_config(|c| c.paused = true);
    reject(f, 707, 0, Some(error(Piv1Error::PausedOperation)));
    reject(principal(707, 0, false), 0, 0, Some(error(Piv1Error::ZeroPrincipalDeposit)));
    reject(principal(707, 0, false), 708, 0, Some(error(Piv1Error::PrincipalDepositExceedsQueue)));
    let mut w = World::new(10_000, 50, 111, 9, 3, FeeFraction::ZERO, 100_000);
    w.open(900_000).unwrap(); reject(from_world(&w, false), 1, 0, Some(error(Piv1Error::InvalidLifecycle)));
    w.settle().unwrap(); reject(from_world(&w, false), 1, 0, Some(error(Piv1Error::InvalidLifecycle)));
    for index in [PENDING_SOL, PRINCIPAL_SOL, ESCROW_SOL, KIF_SOL, PRINCIPAL_JITO, PENDING_JITO] {
        let mut f = principal(707, 100, false);
        if index == PRINCIPAL_JITO || index == PENDING_JITO { f.custody.set_token_units(index, f.custody.token_units(index)+1); }
        else { f.custody.accounts[index].lamports += 1; }
        reject(f, 707, 0, Some(error(Piv1Error::InvalidCustodyObservation)));
    }
    let rich = from_world(&completed_world(), false);
    for index in [PENDING_SOL, PRINCIPAL_SOL, KIF_SOL, PRINCIPAL_JITO, PENDING_JITO] {
        let mut f = rich.clone();
        if index == PRINCIPAL_JITO || index == PENDING_JITO { f.custody.set_token_units(index, f.custody.token_units(index)-1); }
        else { f.custody.accounts[index].lamports -= 1; }
        reject(f, 101, 0, Some(error(Piv1Error::EconomicCustodyDeficit)));
    }
    let mut f = principal(707, 0, false); f.custody.donate([1, 2, 3, 4, 5, 6]);
    reject(f, 707, 0, Some(error(Piv1Error::InvalidCustodyObservation)));
}

#[test]
fn authenticated_ratio_authority_supply_and_checked_arithmetic_cannot_be_forged() {
    for case in 0..8 {
        let mut f = principal(707, 100, false);
        match case { 0 => f.pool.last_update_epoch -= 1, 1 => f.pool.last_update_epoch += 1,
            2 => f.mint.supply += 1, 3 => f.mint.supply = 99, 4 => f.pool.total_lamports = 0,
            5 => { f.pool.pool_token_supply = 0; f.mint.supply = 0; },
            6 => f.pool.sol_deposit_authority = Some(key(244)),
            _ => { f.mint.supply = 100; f.custody.edit_config(|c| c.accounted_pending_jitosol_units = 1); f.custody.set_token_units(PENDING_JITO, 1); } }
        refresh(&mut f); reject(f, 707, 0, Some(error(Piv1Error::InvalidPrincipalDepositPool)));
    }
    let mut f = principal(707, 0, false); f.pool.total_lamports = u64::MAX; f.pool.pool_token_supply = u64::MAX; f.mint.supply = u64::MAX;
    refresh(&mut f); reject(f, 707, 0, Some(error(Piv1Error::ArithmeticOverflow)));
    let mut f = principal(707, 0, false); f.custody.accounts[RESERVE].lamports = u64::MAX;
    reject(f, 707, 0, Some(error(Piv1Error::ArithmeticOverflow)));
    for index in [PROTOCOL, POOL, LIST, RESERVE, MINT, MANAGER] {
        let mut f = principal(707, 0, false); f.custody.accounts[index].key = key(244);
        reject(f, 707, 0, Some(ProgramError::Custom(6107)));
    }
    for index in [CONFIG, ROUND, PENDING_SOL, PRINCIPAL_SOL, OPERATIONAL_SOL, ESCROW_SOL, KIF_SOL, PRINCIPAL_JITO, PENDING_JITO, MINT, POOL, LIST, RESERVE, MANAGER, REFERRER] {
        let mut f = principal(707, 0, false); f.custody.accounts[index].lamports = 1;
        reject(f, 707, 0, Some(error(Piv1Error::AccountRentDeficit)));
    }
}

#[test]
fn strict_abi_context_topology_and_all_later_borrows_fail_closed() {
    let mut f = principal(707, 0, false); let before = f.clone(); let data = request(707, 700);
    assert_eq!(&data[..9], b"PIV1SP01\x01"); assert_eq!(&data[9..17], &707_u64.to_le_bytes()); assert_eq!(&data[17..25], &700_u64.to_le_bytes());
    for case in 0..4 {
        let mut bad = data.to_vec(); match case { 0 => bad[0] ^= 1, 1 => bad[8] = 2, 2 => { bad.pop(); }, _ => bad.push(0) }
        f.custody.with_infos(|a| assert_eq!(execute(&PROGRAM, a, &bad, || panic!(), || panic!(), |_, _, _| panic!(), |_| panic!()), Err(ProgramError::InvalidInstructionData)));
    }
    f.custody.with_infos(|a| {
        assert_eq!(process_instruction(&PROGRAM, a, &data), Err(ProgramError::Custom(HOST_RUNTIME_UNAVAILABLE_CODE)));
        assert_eq!(claim_seam(&PROGRAM, a, &data, || panic!(), |_, _, _| panic!(), |_| panic!()), Err(ProgramError::Custom(HOST_RUNTIME_UNAVAILABLE_CODE)));
        assert_eq!(execute(&PROGRAM, a, &data, || Err(ProgramError::Custom(811)), || panic!(), |_, _, _| panic!(), |_| panic!()), Err(ProgramError::Custom(811)));
    }); assert_eq!(f, before);
    for index in [CONFIG, PRINCIPAL_SOL, PRINCIPAL_JITO, MINT, POOL, RESERVE, MANAGER, 18] {
        let mut f = before.clone(); f.custody.accounts[index].writable = false;
        reject(f, 707, 0, Some(error(Piv1Error::AccountNotWritable)));
        for data_borrow in [false, true] {
            let mut f = before.clone(); let rent = f.custody.rent.clone(); let clock = f.clock.clone();
            f.custody.with_infos(|a| {
                let _data = data_borrow.then(|| a[index].try_borrow_data().unwrap());
                let _lamports = (!data_borrow).then(|| a[index].try_borrow_lamports().unwrap());
                assert_eq!(execute(&PROGRAM, a, &data, || Ok(clock), || Ok(rent), |_, _, _| panic!(), |_| panic!()), Err(error(Piv1Error::AccountBorrowFailed)));
            }); assert_eq!(f, before);
        }
    }
    let mut f = before.clone(); f.custody.accounts[19].owner = spl_token::ID; let saved = f.clone();
    let rent = f.custody.rent.clone(); let clock = f.clock.clone(); f.custody.with_infos(|a| {
        let _held = a[19].try_borrow_mut_data().unwrap();
        assert_eq!(execute(&PROGRAM, a, &data, || Ok(clock), || Ok(rent), |_, _, _| panic!(), |_| panic!()), Err(error(Piv1Error::AccountBorrowFailed)));
    }); assert_eq!(f, saved);
    f.custody.accounts[19].data = vec![0; 355]; reject(f, 707, 0, Some(error(Piv1Error::InvalidAccountData)));
    let mut short = before.clone(); short.custody.accounts.pop(); reject(short, 707, 0, Some(ProgramError::NotEnoughAccountKeys));
    // The earlier bootstrap topology has no withdraw-authority role.
    reject(Fixture::new(707, 0, false), 707, 0, Some(ProgramError::NotEnoughAccountKeys));
    let mut wrong = principal(707, 0, true); let mut extra = wrong.custody.accounts[MANAGER].clone(); extra.key = key(244);
    wrong.custody.accounts.push(extra); reject(wrong, 707, 0, Some(ProgramError::InvalidArgument));
    for kind in 0..3 { let mut f = before.clone(); let rent = f.custody.rent.clone(); let clock = f.clock.clone();
        f.custody.with_infos(|a| { let mut a = a.to_vec(); match kind { 0 => a[19].key = a[AUTHORITY].key,
            1 => a[19].data = a[AUTHORITY].data.clone(), _ => a[19].lamports = a[AUTHORITY].lamports.clone() }
            assert_eq!(execute(&PROGRAM, &a, &data, || Ok(clock), || Ok(rent), |_, _, _| panic!(), |_| panic!()), Err(error(Piv1Error::AccountAlias)));
        }); assert_eq!(f, before);
    }
}

#[test]
fn legacy_profile_unused_fields_receipt_and_error_precedence_remain_unchanged() {
    let mut w = World::empty(3, 42); w.explicit_sol(707, 707).unwrap(); w.bootstrap().unwrap();
    let before = w.pool.pool_snapshot().unwrap(); let mut pool = w.pool.clone();
    let request = SolDepositRequest { snapshot: before.identity(), native_lamports: 707,
        caller_minimum_pool_tokens_out: 0, slippage_bps: w.config.configured_slippage_bps };
    let execution = pool.execute_protected_sol_deposit(request).unwrap(); let custody_before = w.observation(); let mut custody_after = custody_before;
    custody_after.principal_sol.lamports -= 707; custody_after.principal_jitosol_units += 700;
    let base = PrincipalSolDepositObservation { request, execution, pool_before: before,
        pool_after: pool.pool_snapshot().unwrap(), custody_before, custody_after };
    for kind in 0..5 {
        let mut observation = base;
        match kind { 0 => observation.pool_before.minimum_delegation_lamports = 0,
            1 => observation.pool_after.available_withdrawal_lamports = observation.pool_after.total_pool_lamports+1,
            2 => observation.pool_before.sol_deposit_fee = FeeFraction { numerator: 0, denominator: 0 },
            3 => observation.pool_before.stake_withdrawal_fee = FeeFraction { numerator: 1, denominator: 1 },
            _ => observation.request.snapshot.revision += 1 }
        observation.execution.actual_fee_pool_tokens = 1;
        let mut c = w.config.clone(); let original = c.clone();
        assert_eq!(record_protected_principal_deposit(&mut c, &w.round, observation), Err(Piv1Error::InvalidPrincipalDepositPool)); assert_eq!(c, original);
        c.paused = true; let original = c.clone();
        assert_eq!(record_protected_principal_deposit(&mut c, &w.round, observation), Err(Piv1Error::PausedOperation)); assert_eq!(c, original);
        c.paused = false; observation.request.native_lamports = 0; let original = c.clone();
        assert_eq!(record_protected_principal_deposit(&mut c, &w.round, observation), Err(Piv1Error::ZeroPrincipalDeposit)); assert_eq!(c, original);
    }
    let mut bad = base; bad.execution.actual_pool_tokens_out += 1; bad.custody_after.pending_jitosol_units += 1;
    let mut c = w.config.clone(); assert_eq!(record_protected_principal_deposit(&mut c, &w.round, bad), Err(Piv1Error::PrincipalDepositObservationMismatch)); assert_eq!(c, w.config);
}

#[test]
fn each_internal_effect_prefix_is_exact_on_cpi_error_then_explicit_discard_retry() {
    for shared in [false, true] { let base = principal(707, 100, shared);
        for stage in 0..4 {
            let mut staged = base.clone(); let rent = staged.custody.rent.clone(); let clock = staged.clock.clone();
            assert_eq!(staged.custody.with_infos(|a| execute(&PROGRAM, a, &request(707, 0), || Ok(clock), || Ok(rent), |ix, infos, seeds| {
                invoke(&base, 707, 0, stage, ix, infos, seeds)?; Err(ProgramError::Custom(812))
            }, |_| panic!())), Err(ProgramError::Custom(812)));
            assert_eq!(staged.custody, raw_prefix(&base, 707, stage));
            let mut retry = base.clone(); success(&mut retry, 707, 0);
        }
    }
}

#[test]
fn every_account_and_mutated_protocol_field_is_checked_before_config_commit() {
    let base = principal(707, 100, false);
    for index in 0..20 {
        let mut staged = base.clone(); let rent = staged.custody.rent.clone(); let clock = staged.clock.clone();
        assert_eq!(staged.custody.with_infos(|a| execute(&PROGRAM, a, &request(707, 0), || Ok(clock), || Ok(rent), |ix, infos, seeds| {
            invoke(&base, 707, 0, 3, ix, infos, seeds)?; **a[index].try_borrow_mut_lamports()? += 1; Ok(())
        }, |_| panic!())), Err(error(Piv1Error::PrincipalDepositObservationMismatch)));
        let mut expected = raw_prefix(&base, 707, 3); expected.accounts[index].lamports += 1; assert_eq!(staged.custody, expected);
    }
    for (index, offset) in [(CONFIG, 20), (ROUND, 20), (PRINCIPAL_JITO, 64), (PRINCIPAL_JITO, 129),
        (PENDING_JITO, 64), (MINT, 36), (MINT, 45), (POOL, 258), (POOL, 266), (POOL, 274),
        (POOL, 346), (POOL, 2047), (RESERVE, 12), (MANAGER, 64), (18, 64), (LIST, 8), (PROTOCOL, 10)] {
        let mut staged = base.clone(); let rent = staged.custody.rent.clone(); let clock = staged.clock.clone();
        assert_eq!(staged.custody.with_infos(|a| execute(&PROGRAM, a, &request(707, 0), || Ok(clock), || Ok(rent), |ix, infos, seeds| {
            invoke(&base, 707, 0, 3, ix, infos, seeds)?; a[index].try_borrow_mut_data()?[offset] ^= 1; Ok(())
        }, |_| panic!())), Err(error(Piv1Error::PrincipalDepositObservationMismatch)));
        let mut expected = raw_prefix(&base, 707, 3); expected.accounts[index].data[offset] ^= 1; assert_eq!(staged.custody, expected);
    }
}
