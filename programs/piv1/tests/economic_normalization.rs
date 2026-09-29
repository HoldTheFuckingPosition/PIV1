//! Host-only signed-CPI and full-account evidence; modeled discard is explicit.
mod support;
#[path = "support/pending_custody.rs"]
pub mod pending_custody;
#[path = "support/economic_custody.rs"]
mod custody;
use support::kif_claim_custody;
use anchor_lang::{prelude::{AccountInfo, Pubkey}, solana_program::{entrypoint::ProgramResult,
    instruction::{AccountMeta, Instruction}, program_error::ProgramError, system_program}};
use piv1::{accounts::authenticate_fixed_accounts, economic_normalization::process_instruction_with_host_callbacks as execute,
    errors::Piv1Error, events::UntrackedBalanceReconciled,
    instruction_boundary::{process_instruction, process_instruction_with_host_callbacks as claim_seam},
    instruction_errors::{piv1_error_code, HOST_RUNTIME_UNAVAILABLE_CODE},
    instructions::reconcile_untracked_balances::*, integrations::{FeeFraction, WithdrawalSourceId},
    state::DistributionLifecycle};
use custody::*;
use support::{kif_claim_custody::{key, PROGRAM}, vault_custody_model::World};

const MIXED: [u64; 6] = [2, 3, 5, 7, 11, 13];
fn world() -> World { World::new(1_000, 50, 123, 9, 0, FeeFraction::ZERO, 100_000) }
fn err(value: Piv1Error) -> ProgramError { ProgramError::Custom(piv1_error_code(value)) }
fn plans(d: [u64; 6]) -> Vec<(usize, u64)> {
    [(PRINCIPAL_SOL, d[1]), (ESCROW_SOL, d[2]), (KIF_SOL, d[3]), (PRINCIPAL_JITO, d[5])]
        .into_iter().filter(|(_, amount)| *amount != 0).collect()
}

fn transfer(source: usize, amount: u64, ix: &Instruction, a: &[AccountInfo<'_>], signers: &[&[&[u8]]]) -> ProgramResult {
    let seed: &[u8] = match source { PRINCIPAL_SOL => b"principal-sol", ESCROW_SOL => b"distribution-escrow",
        KIF_SOL => b"kif-sol", PRINCIPAL_JITO => b"authority", _ => panic!("unexpected source") };
    let (signer, bump) = Pubkey::find_program_address(&[seed], &FIXTURE_PROGRAM);
    assert_eq!(signers, &[&[seed, &[bump]][..]]);
    assert_eq!(Pubkey::create_program_address(signers[0], &PROGRAM).unwrap(), signer);
    if source == PRINCIPAL_JITO {
        assert_eq!(*a[3].key, signer); assert_eq!(*a[4].key, spl_token::ID); assert_eq!(a.len(), 5);
        let mut expected = vec![12]; expected.extend_from_slice(&amount.to_le_bytes()); expected.push(9);
        assert_eq!(ix.data, expected); assert_eq!(ix.program_id, spl_token::ID);
        assert_eq!(ix.accounts, vec![AccountMeta::new(*a[0].key, false), AccountMeta::new_readonly(*a[1].key, false),
            AccountMeta::new(*a[2].key, false), AccountMeta::new_readonly(signer, true)]);
        // The host models runtime PDA privilege promotion only after independent
        // seed derivation. Original outer flags and bytes remain unchanged.
        let mut inner = a[..4].to_vec(); inner[3].is_signer = true;
        spl_token::processor::Processor::process(&spl_token::ID, &inner, &ix.data)
    } else {
        assert_eq!(*a[0].key, signer); assert_eq!(*a[2].key, system_program::ID); assert_eq!(a.len(), 3);
        let mut expected = vec![2, 0, 0, 0]; expected.extend_from_slice(&amount.to_le_bytes());
        assert_eq!(ix.data, expected); assert_eq!(ix.program_id, system_program::ID);
        assert_eq!(ix.accounts, vec![AccountMeta::new(signer, true), AccountMeta::new(*a[1].key, false)]);
        **a[0].try_borrow_mut_lamports()? -= amount; **a[1].try_borrow_mut_lamports()? += amount; Ok(())
    }
}
fn success(f: &mut Fixture, d: [u64; 6]) {
    let mut expected = f.clone(); let c = f.config(); let sol: u64 = d[..4].iter().sum(); let tokens = d[4] + d[5];
    for (index, amount) in [(PRINCIPAL_SOL, d[1]), (ESCROW_SOL, d[2]), (KIF_SOL, d[3])] {
        expected.accounts[index].lamports -= amount; expected.accounts[PENDING_SOL].lamports += amount;
    }
    expected.set_token_units(PRINCIPAL_JITO, expected.token_units(PRINCIPAL_JITO) - d[5]);
    expected.set_token_units(PENDING_JITO, expected.token_units(PENDING_JITO) + d[5]);
    expected.edit_config(|c| { c.accounted_pending_sol_lamports += sol; c.accounted_pending_jitosol_units += tokens; });
    let event = UntrackedBalanceReconciled { config: f.accounts[CONFIG].key,
        newly_accounted_sol_lamports: sol, newly_accounted_jitosol_units: tokens,
        pending_sol_lamports_after: c.accounted_pending_sol_lamports + sol,
        pending_jitosol_units_after: c.accounted_pending_jitosol_units + tokens };
    let expected_plans = plans(d); let mut count = 0; let mut events = vec![]; let rent = f.rent.clone();
    f.with_infos(|a| execute(&PROGRAM, a, &RECONCILE_UNTRACKED_DATA, || Ok(rent), |ix, infos, seeds| {
        let (source, amount) = expected_plans[count]; count += 1; transfer(source, amount, ix, infos, seeds)
    }, |e| events.push(e))).unwrap();
    assert_eq!(count, expected_plans.len()); assert_eq!(events, vec![event]); assert_eq!(*f, expected);
}
fn reject(mut f: Fixture, expected: ProgramError) {
    let before = f.clone(); let rent = f.rent.clone();
    assert_eq!(f.with_infos(|a| execute(&PROGRAM, a, &RECONCILE_UNTRACKED_DATA, || Ok(rent),
        |_, _, _| panic!("must reject before first CPI"), |_| panic!("failed event"))), Err(expected));
    assert_eq!(f, before);
}
fn raw_prefix(base: &Fixture, transfers: &[(usize, u64)], count: usize) -> Fixture {
    let mut expected = base.clone();
    for &(source, amount) in &transfers[..count] {
        if source == PRINCIPAL_JITO {
            expected.set_token_units(PRINCIPAL_JITO, expected.token_units(PRINCIPAL_JITO) - amount);
            expected.set_token_units(PENDING_JITO, expected.token_units(PENDING_JITO) + amount);
        } else {
            expected.accounts[source].lamports -= amount;
            expected.accounts[PENDING_SOL].lamports += amount;
        }
    }
    expected
}
fn recovery() -> World {
    let mut w = world(); w.open(900_000).unwrap(); let leg = w.initiate(1).unwrap();
    w.pool.decrease_exchange_rate(w.pool.raw_snapshot().total_pool_lamports - 1).unwrap();
    w.advance_epoch().unwrap(); w.finalize(leg).unwrap();
    assert_eq!(w.round.lifecycle, DistributionLifecycle::RecoveryRequired); w
}

#[test]
fn all_six_surpluses_preserve_real_lifecycle_offsets_rent_history_and_repeat_noop() {
    let mut w = world(); let mut phases = vec![w.clone()];
    w.pool.set_source_finalization_terms(WithdrawalSourceId(1), 1, 20, 10, 37, 0).unwrap();
    w.open(900_000).unwrap(); assert!(w.round.pending_sol_used_lamports > 0); phases.push(w.clone());
    let leg = w.initiate(1).unwrap(); assert!(w.round.cumulative_jitosol_assigned_units > 0); phases.push(w.clone());
    w.advance_epoch().unwrap(); w.finalize(leg).unwrap(); phases.push(w.clone());
    assert_eq!(w.round.cumulative_cooldown_rewards_lamports, 37);
    w.settle().unwrap(); assert!(w.round.actual_zero_active_kif_compound_lamports > 0); phases.push(w.clone());
    w.integrate(900_100).unwrap(); assert!(w.round.last_completed.is_some()); phases.push(w);
    for w in &phases {
        let mut f = Fixture::from_world(w); f.donate(MIXED);
        success(&mut f, MIXED);
        for account in &mut f.accounts[1..] { account.writable = false; }
        success(&mut f, [0; 6]);
    }
    // Positive-surplus subsets cannot require unrelated accounts writable.
    for index in 0..6 {
        let mut d = [0; 6]; d[index] = 1; let mut f = Fixture::from_world(&world()); f.donate(d);
        success(&mut f, d);
    }
}

#[test]
fn paused_and_recovery_allow_only_already_pending_recognition() {
    for mut w in [world(), recovery()] { for paused in [false, true] {
        w.config.paused = paused;
        if paused || w.round.lifecycle == DistributionLifecycle::RecoveryRequired {
            let mut f = Fixture::from_world(&w); let d = [2, 0, 0, 0, 11, 0]; f.donate(d);
            success(&mut f, d); success(&mut f, [0; 6]);
            for index in [1, 2, 3, 5] {
                let mut f = Fixture::from_world(&w); let mut d = [0; 6]; d[index] = 1; f.donate(d);
                reject(f, err(if paused { Piv1Error::PausedOperation } else { Piv1Error::RecoveryRequired }));
            }
        }
    } }
}

#[test]
fn token_native_and_operational_funding_are_quarantined_without_changing_old_accessor() {
    for excess in [1, 999_999] {
        let mut f = Fixture::from_world(&world()); f.accounts[PRINCIPAL_JITO].lamports += excess;
        f.accounts[PENDING_JITO].lamports += excess + 1; f.accounts[OPERATIONAL_SOL].lamports += excess + 2;
        let rent = f.rent.clone(); f.with_infos(|a| {
            let observed = authenticate_fixed_accounts(&PROGRAM, &rent, roles(a)).unwrap();
            assert_eq!(observed.economic_observation(), Err(Piv1Error::UnsupportedTokenNativeExcess));
            observed.economic_normalization_observation().unwrap();
        }); f.donate(MIXED); success(&mut f, MIXED);
    }
}

#[test]
fn each_own_deficit_cannot_be_covered_by_unrelated_surplus() {
    for index in [PENDING_SOL, PRINCIPAL_SOL, ESCROW_SOL, KIF_SOL, PRINCIPAL_JITO, PENDING_JITO] {
        let mut w = world();
        if index == ESCROW_SOL { w.open(900_000).unwrap(); }
        let mut f = Fixture::from_world(&w);
        if index == PRINCIPAL_JITO || index == PENDING_JITO { f.set_token_units(index, f.token_units(index) - 1); }
        else { f.accounts[index].lamports -= 1; }
        let unrelated = if index == KIF_SOL { PRINCIPAL_SOL } else { KIF_SOL };
        f.accounts[unrelated].lamports += 1_000_000;
        reject(f, err(Piv1Error::EconomicCustodyDeficit));
    }
}

#[test]
fn strict_wire_counts_context_and_all_native_host_guards_hold() {
    let mut f = Fixture::from_world(&world()); let before = f.clone();
    assert_eq!(&RECONCILE_UNTRACKED_DATA, b"PIV1RB01\x01");
    for kind in 0..4 {
        let mut data = RECONCILE_UNTRACKED_DATA.to_vec(); match kind { 0 => data[0] ^= 1,
            1 => data[8] = 2, 2 => { data.pop(); }, _ => data.push(0) }
        f.with_infos(|a| assert_eq!(execute(&PROGRAM, a, &data, || panic!(), |_, _, _| panic!(), |_| panic!()),
            Err(ProgramError::InvalidInstructionData)));
    }
    f.with_infos(|a| {
        let data = &RECONCILE_UNTRACKED_DATA;
        assert_eq!(execute(&PROGRAM, &a[..12], data, || panic!(), |_, _, _| panic!(), |_| panic!()), Err(ProgramError::NotEnoughAccountKeys));
        let mut extra = a.to_vec(); extra.push(a[0].clone());
        assert_eq!(execute(&PROGRAM, &extra, data, || panic!(), |_, _, _| panic!(), |_| panic!()), Err(ProgramError::InvalidArgument));
        assert_eq!(process_instruction(&PROGRAM, a, data), Err(ProgramError::Custom(HOST_RUNTIME_UNAVAILABLE_CODE)));
        assert_eq!(claim_seam(&PROGRAM, a, data, || panic!(), |_, _, _| panic!(), |_| panic!()), Err(ProgramError::Custom(HOST_RUNTIME_UNAVAILABLE_CODE)));
        assert_eq!(execute(&PROGRAM, a, data, || Err(ProgramError::Custom(918)), |_, _, _| panic!(), |_| panic!()), Err(ProgramError::Custom(918)));
    }); assert_eq!(f, before);
}

#[test]
fn program_mint_authority_and_rent_validation_also_precede_noop() {
    for moved in [false, true] {
        let mut base = Fixture::from_world(&world()); if moved { base.donate(MIXED); }
        for index in [SYSTEM, TOKEN] {
            let mut f = base.clone(); f.accounts[index].key = key(230); reject(f, err(Piv1Error::InvalidProgramIdentity));
            let mut f = base.clone(); f.accounts[index].executable = false; reject(f, err(Piv1Error::InvalidProgramIdentity));
        }
        let mut f = base.clone(); f.accounts[AUTHORITY].key = key(231); reject(f, err(Piv1Error::InvalidAccountPda));
        let mut f = base.clone(); f.accounts[AUTHORITY].owner = spl_token::ID; reject(f, err(Piv1Error::InvalidAccountOwner));
        let mut f = base.clone(); f.accounts[AUTHORITY].data.push(0); reject(f, err(Piv1Error::InvalidAccountData));
        let mut f = base.clone(); f.accounts[AUTHORITY].executable = true; reject(f, err(Piv1Error::ExecutableAccount));
        let mut f = base.clone(); f.accounts[MINT].key = key(231); reject(f, err(Piv1Error::InvalidTokenCustody));
        let mut f = base.clone(); f.accounts[MINT].owner = system_program::ID; reject(f, err(Piv1Error::InvalidAccountOwner));
        let mut f = base.clone(); f.accounts[MINT].data[45] = 0; reject(f, err(Piv1Error::InvalidTokenCustody));
        for index in [PRINCIPAL_SOL, PRINCIPAL_JITO] {
            let mut f = base.clone(); f.accounts[index].key = key(232); reject(f, err(Piv1Error::InvalidAccountPda));
            let mut f = base.clone(); f.accounts[index].owner = key(232); reject(f, err(Piv1Error::InvalidAccountOwner));
        }
        for offset in [0, 72, 108, 109] {
            let mut f = base.clone(); f.accounts[PRINCIPAL_JITO].data[offset] ^= 1;
            reject(f, err(Piv1Error::InvalidTokenCustody));
        }
        for index in [CONFIG, ROUND, PENDING_SOL, PRINCIPAL_SOL, OPERATIONAL_SOL, ESCROW_SOL, KIF_SOL, PRINCIPAL_JITO, PENDING_JITO, MINT] {
            let mut f = base.clone(); f.accounts[index].lamports = f.rent.minimum_balance(f.accounts[index].data.len()) - 1;
            reject(f, err(Piv1Error::AccountRentDeficit));
        }
        let mut f = base.clone(); f.rent.exemption_threshold = f64::INFINITY; reject(f, err(Piv1Error::InvalidRent));
    }
    let mut f = Fixture::from_world(&world()); f.accounts[CONFIG].writable = false;
    reject(f, err(Piv1Error::AccountNotWritable));
    // An unrelated readonly, unallocated authority need not invent a rent deposit.
    let mut f = Fixture::from_world(&world()); assert_eq!(f.accounts[AUTHORITY].lamports, 0); success(&mut f, [0; 6]);
}

#[test]
fn aliases_required_write_access_and_all_later_borrows_fail_before_first_cpi() {
    let mut base = Fixture::from_world(&world()); base.donate(MIXED);
    for index in [CONFIG, PENDING_SOL, PRINCIPAL_SOL, ESCROW_SOL, KIF_SOL, PRINCIPAL_JITO, PENDING_JITO] {
        let mut f = base.clone(); f.accounts[index].writable = false; reject(f, err(Piv1Error::AccountNotWritable));
        for native in [false, true] {
            let mut f = base.clone(); let before = f.clone(); let rent = f.rent.clone();
            f.with_infos(|a| {
                if native { let _guard = a[index].try_borrow_lamports().unwrap();
                    assert_eq!(execute(&PROGRAM, a, &RECONCILE_UNTRACKED_DATA, || Ok(rent), |_, _, _| panic!(), |_| panic!()), Err(err(Piv1Error::AccountBorrowFailed)));
                } else { let _guard = a[index].try_borrow_data().unwrap();
                    assert_eq!(execute(&PROGRAM, a, &RECONCILE_UNTRACKED_DATA, || Ok(rent), |_, _, _| panic!(), |_| panic!()), Err(err(Piv1Error::AccountBorrowFailed)));
                }
            }); assert_eq!(f, before);
        }
    }
    let mut f = base.clone(); let rent = f.rent.clone(); f.with_infos(|a| {
        let _mint = a[MINT].try_borrow_data().unwrap();
        assert_eq!(execute(&PROGRAM, a, &RECONCILE_UNTRACKED_DATA, || Ok(rent), |_, _, _| panic!(), |_| panic!()), Err(err(Piv1Error::AccountBorrowFailed)));
    }); assert_eq!(f, base);
    for kind in 0..3 {
        let mut f = base.clone(); let rent = f.rent.clone(); f.with_infos(|a| {
            let mut a = a.to_vec(); match kind { 0 => a[7].key = a[8].key,
                1 => a[7].data = a[8].data.clone(), _ => a[7].lamports = a[8].lamports.clone() }
            assert_eq!(execute(&PROGRAM, &a, &RECONCILE_UNTRACKED_DATA, || Ok(rent), |_, _, _| panic!(), |_| panic!()), Err(err(Piv1Error::AccountAlias)));
        }); assert_eq!(f, base);
    }
}

#[test]
fn destination_and_active_pending_ledger_overflows_precede_effects() {
    let mut f = Fixture::from_world(&world()); f.donate(MIXED); f.accounts[PENDING_SOL].lamports = u64::MAX;
    reject(f, err(Piv1Error::ArithmeticOverflow));
    let mut f = Fixture::from_world(&world()); f.donate(MIXED); f.set_token_units(PENDING_JITO, u64::MAX);
    reject(f, err(Piv1Error::ArithmeticOverflow));
    let mut w = world(); w.open(900_000).unwrap(); let offset = w.round.pending_sol_used_lamports;
    let mut f = Fixture::from_world(&w); f.rent.exemption_threshold = 0.0;
    f.edit_config(|c| c.accounted_pending_sol_lamports = u64::MAX);
    f.accounts[PENDING_SOL].lamports = u64::MAX - offset;
    // Strip only fixture rent from other native vaults under explicit zero Rent.
    let old_floor = anchor_lang::prelude::Rent::default().minimum_balance(0);
    for index in [PRINCIPAL_SOL, ESCROW_SOL, KIF_SOL] { f.accounts[index].lamports -= old_floor; }
    f.donate([0, 1, 0, 0, 0, 0]); reject(f, err(Piv1Error::ArithmeticOverflow));
}

#[test]
fn every_late_cpi_failure_preserves_config_until_explicit_staged_discard_and_retry() {
    let mut base = Fixture::from_world(&world()); base.donate(MIXED); let expected = plans(MIXED);
    for failed in 0..4 { for partial in [false, true] {
        let mut staged = base.clone(); let rent = staged.rent.clone(); let mut count = 0;
        assert_eq!(staged.with_infos(|a| execute(&PROGRAM, a, &RECONCILE_UNTRACKED_DATA, || Ok(rent), |ix, infos, seeds| {
            let current = count; count += 1; let (source, amount) = expected[current];
            if current == failed && !partial { return Err(ProgramError::Custom(9991)); }
            transfer(source, amount, ix, infos, seeds)?;
            if current == failed { Err(ProgramError::Custom(9991)) } else { Ok(()) }
        }, |_| panic!())), Err(ProgramError::Custom(9991)));
        assert_eq!(count, failed + 1);
        // Exact prefix conservation and every original account byte/metadata,
        // including the still-uncommitted Config/round, are independently pinned.
        assert_eq!(staged, raw_prefix(&base, &expected, failed + usize::from(partial)));
        let mut retry = base.clone(); success(&mut retry, MIXED);
    } }
}

#[test]
fn each_cpi_postcheck_detects_wrong_effects_and_any_unrelated_account_change() {
    let mut base = Fixture::from_world(&world()); base.donate(MIXED); let expected = plans(MIXED);
    for failed in 0..4 { for account in 0..13 {
        let mut staged = base.clone(); let rent = staged.rent.clone(); let mut count = 0;
        assert_eq!(staged.with_infos(|a| execute(&PROGRAM, a, &RECONCILE_UNTRACKED_DATA, || Ok(rent), |ix, infos, seeds| {
            let current = count; count += 1; let (source, amount) = expected[current];
            transfer(source, amount, ix, infos, seeds)?;
            if current == failed { **a[account].try_borrow_mut_lamports()? += 1; } Ok(())
        }, |_| panic!())), Err(err(Piv1Error::ContributionObservationMismatch)));
        assert_eq!(count, failed + 1);
        let mut raw_expected = raw_prefix(&base, &expected, failed + 1);
        raw_expected.accounts[account].lamports += 1; assert_eq!(staged, raw_expected);
    } }
    for account in [CONFIG, ROUND, PRINCIPAL_JITO, PENDING_JITO, MINT, SYSTEM, TOKEN] {
        let mut staged = base.clone(); let rent = staged.rent.clone();
        assert_eq!(staged.with_infos(|a| execute(&PROGRAM, a, &RECONCILE_UNTRACKED_DATA, || Ok(rent), |ix, infos, seeds| {
            transfer(PRINCIPAL_SOL, 3, ix, infos, seeds)?; a[account].try_borrow_mut_data()?[0] ^= 1; Ok(())
        }, |_| panic!())), Err(err(Piv1Error::ContributionObservationMismatch)));
        let mut raw_expected = raw_prefix(&base, &expected, 1);
        raw_expected.accounts[account].data[0] ^= 1; assert_eq!(staged, raw_expected);
    }
    let mut staged = base.clone(); let rent = staged.rent.clone();
    assert_eq!(staged.with_infos(|a| execute(&PROGRAM, a, &RECONCILE_UNTRACKED_DATA, || Ok(rent), |_, _, _| Ok(()), |_| panic!())),
        Err(err(Piv1Error::ContributionObservationMismatch))); assert_eq!(staged, base);
}
