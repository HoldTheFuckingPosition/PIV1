#![forbid(unsafe_code)]
//! Execution requires the separately reviewed runner and exact Task 2.12 ELF.
//! Synthetic message signers are not signatures. Failure outputs are Mollusk's
//! context discard, not a Bank/AccountsDB transaction rollback proof.

mod support;
use support::*;
use solana_instruction_error::InstructionError;
use solana_transaction_error::TransactionError;
use piv1::errors::Piv1Error;

const ORDINARY_CU: u64 = 200_000;

#[test]
fn account_evidence_retains_complete_config_and_reward_tails() {
    for length in [piv1::state::PivConfig::SPACE, piv1::state::GuardianReward::SPACE] {
        let mut data = vec![0; length];
        data[0] = 0x12; data[63] = 0xab; data[64] = 0xcd;
        data[length - 2] = 0xef; data[length - 1] = 0x91;
        let key = new_key(91); let owner = new_key(92);
        let account = solana_account::Account { lamports: 123456, data, owner,
            executable: true, rent_epoch: 987 };
        let line = account_evidence("formatter", "raw-after-invocation", Some(2), 3, &key, &account);
        let prefix = format!("ACCOUNT case=formatter stage=raw-after-invocation invocation_ordinal=Some(2) index=3 key={key} lamports=123456 owner={owner} executable=true rent_epoch=987 data_len={length} data_hex=");
        let actual_hex = line.strip_prefix(prefix.as_str()).unwrap();
        let expected_hex = format!("12{}abcd{}ef91", "00".repeat(62), "00".repeat(length - 67));
        assert_eq!(actual_hex.len(), length * 2);
        assert_eq!(actual_hex, expected_hex);
    }
}

fn failure(error: Piv1Error) -> InstructionError {
    InstructionError::Custom(piv1::instruction_errors::piv1_error_code(error))
}

fn is_compute_exhaustion(result: &Result<(), TransactionError>, logs: &[String]) -> bool {
    match result {
        Err(TransactionError::InstructionError(0, InstructionError::ComputationalBudgetExceeded)) => true,
        Err(TransactionError::InstructionError(0, InstructionError::ProgramFailedToComplete)) =>
            logs.iter().any(|line| line.contains("exceeded CUs meter at BPF instruction")),
        _ => false,
    }
}

#[test]
fn compute_classifier_does_not_accept_call_depth_or_unrelated_failures() {
    let wrapped = Err(TransactionError::InstructionError(0, InstructionError::ProgramFailedToComplete));
    assert!(!is_compute_exhaustion(&wrapped, &["exceeded max BPF to BPF call depth".into()]));
    assert!(is_compute_exhaustion(&wrapped, &["exceeded CUs meter at BPF instruction".into()]));
    assert!(is_compute_exhaustion(&Err(TransactionError::InstructionError(0,
        InstructionError::ComputationalBudgetExceeded)), &[]));
    assert!(!is_compute_exhaustion(&Err(TransactionError::InstructionError(0,
        InstructionError::InvalidArgument)), &["exceeded CUs meter at BPF instruction".into()]));
}

fn assert_rejected(label: &str, mutate: impl FnOnce(&mut Fixture, &mut solana_instruction::Instruction), error: InstructionError) {
    let runtime = Runtime::new(ORDINARY_CU);
    let mut fixture = Fixture::new(&runtime.vm.sysvars.rent, 0, false);
    let original_audit = fixture.audit.clone();
    let mut instruction = fixture.instruction(100, 10);
    mutate(&mut fixture, &mut instruction);
    let before = fixture.accounts.clone();
    let result = runtime.execute(label, &fixture, &[instruction]);
    assert_eq!(result.raw_result, Err(TransactionError::InstructionError(0, error)), "{label}");
    assert_eq!(result.resulting_accounts, before);
    assert_eq!(fixture.accounts, before);
    assert_eq!(fixture.audit, original_audit);
    assert!(result.inner_instructions.iter().all(Vec::is_empty));
    let observations = runtime.observations.borrow();
    assert_eq!(observations.after.len(), 1);
    assert!(event_bytes(&observations.after[0].logs).is_empty());
    for (key, account) in &observations.after[0].accounts {
        assert_eq!(account, &before.iter().find(|(k, _)| key == k).unwrap().1);
    }
}

#[test]
fn default_rent_binary_abi_and_ordinary_compute_allowance_are_explicit() {
    assert_eq!(solana_program_runtime::execution_budget::DEFAULT_INSTRUCTION_COMPUTE_UNIT_LIMIT, ORDINARY_CU as u32);
    let runtime = Runtime::new(ORDINARY_CU);
    let old = rent_bridge(&runtime.vm.sysvars.rent);
    assert_eq!(old.lamports_per_byte_year, 6960);
    assert_eq!(old.exemption_threshold, 1.0);
    assert_eq!(old.burn_percent, 50);
    assert_eq!(bincode::serialize(&runtime.vm.sysvars.rent).unwrap(), bincode::serialize(&old).unwrap());
}

#[test]
fn exact_backed_positive_partial_claim_at_ordinary_200000_cu() {
    let runtime = Runtime::new(ORDINARY_CU);
    let mut fixture = Fixture::new(&runtime.vm.sysvars.rent, 0, false);
    let before = fixture.clone();
    let result = runtime.execute("ordinary-exact-backed", &fixture, &[fixture.instruction(100, 10)]);
    assert_eq!(result.raw_result, Ok(()));
    assert!(result.compute_units_consumed > 0 && result.compute_units_consumed <= ORDINARY_CU);
    assert_one_transfer(&result, &fixture, 100);
    let observations = runtime.observations.borrow();
    assert_eq!(observations.after.len(), 1);
    assert_raw_matches(&observations.after[0], &fixture.expected_payment(100));
    assert_one_event(&observations.after[0].logs, &fixture, 100);
    fixture.accept_payment(result.resulting_accounts, 100, &runtime.vm.sysvars.rent);
    assert_eq!(fixture.audit.paid, 100);
    assert_eq!(before.accounts[SENTINEL], fixture.accounts[SENTINEL]);
}

#[test]
fn paused_historical_tuple_full_claim_preserves_carry_excess_and_unrelated_state() {
    let runtime = Runtime::new(ORDINARY_CU);
    let mut fixture = Fixture::new(&runtime.vm.sysvars.rent, 991, true);
    assert!(config(&fixture.accounts).paused);
    assert_ne!(config(&fixture.accounts).guardian_registry_revision, reward(&fixture.accounts).registry_revision);
    assert_eq!(reward(&fixture.accounts).last_active_period, None);
    let result = runtime.execute("paused-historical-full", &fixture, &[fixture.instruction(300, 10)]);
    assert_eq!(result.raw_result, Ok(()));
    assert_one_transfer(&result, &fixture, 300);
    assert_one_event(&runtime.observations.borrow().after[0].logs, &fixture, 300);
    assert_raw_matches(&runtime.observations.borrow().after[0], &fixture.expected_payment(300));
    fixture.accept_payment(result.resulting_accounts, 300, &runtime.vm.sysvars.rent);
    assert_eq!(reward(&fixture.accounts).claimable_lamports, 0);
}

#[test]
fn repeated_accepted_claims_keep_original_audit_and_stale_counter_rejects() {
    let runtime = Runtime::new(ORDINARY_CU);
    let mut fixture = Fixture::new(&runtime.vm.sysvars.rent, 57, false);
    for (ordinal, (amount, counter)) in [(70, 10), (130, 80), (100, 210)].into_iter().enumerate() {
        let runtime = Runtime::new(ORDINARY_CU);
        let result = runtime.execute(&format!("sequential-{ordinal}"), &fixture, &[fixture.instruction(amount, counter)]);
        assert_eq!(result.raw_result, Ok(()));
        assert_one_transfer(&result, &fixture, amount);
        assert_one_event(&runtime.observations.borrow().after[0].logs, &fixture, amount);
        fixture.accept_payment(result.resulting_accounts, amount, &runtime.vm.sysvars.rent);
    }
    assert_eq!(fixture.audit.paid, 300);
    let before = fixture.clone();
    let runtime = Runtime::new(ORDINARY_CU);
    let result = runtime.execute("stale-after-complete", &fixture, &[fixture.instruction(1, 10)]);
    assert_eq!(result.raw_result, Err(TransactionError::InstructionError(0, failure(Piv1Error::StaleKifClaim))));
    assert_eq!(result.resulting_accounts, fixture.accounts);
    assert_eq!(fixture, before);
    assert!(event_bytes(&runtime.observations.borrow().after[0].logs).is_empty());
    fixture.validate_audit(&runtime.vm.sysvars.rent);
}

#[test]
fn default_1400000_budget_is_recorded_separately_from_required_ordinary_case() {
    let runtime = Runtime::new(1_400_000);
    let fixture = Fixture::new(&runtime.vm.sysvars.rent, 0, false);
    let result = runtime.execute("mollusk-default-budget", &fixture, &[fixture.instruction(100, 10)]);
    assert_eq!(result.raw_result, Ok(()));
    assert_eq!(result.resulting_accounts, fixture.expected_payment(100));
    assert_one_transfer(&result, &fixture, 100);
}

#[test]
fn shared_context_success_stale_replay_and_unreachable_third_expose_discard() {
    let runtime = Runtime::new(1_400_000);
    let fixture = Fixture::new(&runtime.vm.sysvars.rent, 19, true);
    let original = fixture.clone();
    let first = fixture.instruction(100, 10);
    let third = fixture.instruction(200, 110);
    let result = runtime.execute("shared-success-stale-unreachable", &fixture, &[first.clone(), first, third]);
    assert_eq!(result.raw_result, Err(TransactionError::InstructionError(1, failure(Piv1Error::StaleKifClaim))));
    let observations = runtime.observations.borrow();
    assert_eq!(observations.after.len(), 2);
    assert_eq!(observations.privileges.len(), 2);
    let actual_first_effects = fixture.expected_payment(100);
    assert_raw_matches(&observations.after[0], &actual_first_effects);
    assert_raw_matches(&observations.after[1], &actual_first_effects);
    assert_ne!(observations.after[0].accounts.iter().find(|(key, _)| *key == fixture.accounts[KIF].0).unwrap().1,
        fixture.accounts[KIF].1);
    assert_one_event(&observations.after[0].logs, &fixture, 100);
    assert_one_event(&observations.after[1].logs, &fixture, 100);
    assert_one_transfer(&result, &fixture, 100);
    assert!(result.inner_instructions[1..].iter().all(Vec::is_empty));
    // These equalities prove Mollusk's output discard only. Raw first effects
    // and its event above still existed before that discard.
    assert_eq!(result.resulting_accounts, original.accounts);
    assert_eq!(fixture, original);
    fixture.validate_audit(&runtime.vm.sysvars.rent);
}

#[test]
fn zero_stale_and_excessive_requests_fail_before_cpi() {
    for (amount, counter, error) in [(0, 10, Piv1Error::ZeroKifClaim),
        (1, 9, Piv1Error::StaleKifClaim), (301, 10, Piv1Error::KifClaimExceeded)] {
        assert_rejected("request", |fixture, ix| *ix = fixture.instruction(amount, counter), failure(error));
    }
}

#[test]
fn full_global_backing_is_required_even_when_small_request_is_payable() {
    assert_rejected("global-backing-one-short", |f, _| f.accounts[KIF].1.lamports -= 1,
        failure(Piv1Error::KifClaimBackingDeficit));
    assert_rejected("source-rent-one-short", |f, _| f.accounts[KIF].1.lamports = 890_879,
        failure(Piv1Error::AccountRentDeficit));
}

#[test]
fn global_ledger_identity_and_selected_record_bounds_reject() {
    assert_rejected("global-identity", |f, _| edit_config(&mut f.accounts,
        |c| c.cumulative_kif_credited_lamports += 1), failure(Piv1Error::CumulativeReconciliationMismatch));
    assert_rejected("selected-exceeds-global", |f, _| edit_reward(&mut f.accounts,
        |r| { r.claimable_lamports = 961; r.cumulative_earned = 971; }),
        failure(Piv1Error::CumulativeReconciliationMismatch));
}

#[test]
fn destination_addition_overflow_rejects_before_any_cpi() {
    assert_rejected("destination-overflow", |f, _| f.accounts[GUARDIAN].1.lamports = u64::MAX,
        failure(Piv1Error::ArithmeticOverflow));
}

#[test]
fn exact_abi_and_account_count_errors_keep_precedence() {
    for data in [vec![], vec![0; 24], vec![0; 23], vec![0; 25]] {
        assert_rejected("malformed-abi", |_, ix| { ix.data = data; ix.accounts.clear(); }, InstructionError::InvalidInstructionData);
    }
    // Preserve the accepted program's numeric error ABI. The newer runtime's
    // MissingAccount is distinct from its decoded legacy insufficient-keys error.
    let insufficient_keys_raw = u64::from(
        anchor_lang::solana_program::program_error::ProgramError::NotEnoughAccountKeys);
    assert_eq!(insufficient_keys_raw, 11_u64 << 32);
    let insufficient_keys = InstructionError::from(insufficient_keys_raw);
    assert_ne!(insufficient_keys, InstructionError::MissingAccount);
    assert_rejected("four-accounts", |_, ix| { ix.accounts.pop(); }, insufficient_keys);
    assert_rejected("six-accounts", |f, ix| ix.accounts.push(solana_instruction::AccountMeta::new_readonly(f.accounts[SENTINEL].0, false)),
        InstructionError::InvalidArgument);
}

#[test]
fn compiled_message_does_not_repair_missing_guardian_signature_or_readonly_roles() {
    assert_rejected("missing-guardian-signer", |_, ix| ix.accounts[GUARDIAN].is_signer = false,
        failure(Piv1Error::MissingGuardianSignature));
    for index in [CONFIG, REWARD, KIF, GUARDIAN] {
        assert_rejected("readonly-role", |_, ix| ix.accounts[index].is_writable = false,
            failure(Piv1Error::AccountNotWritable));
    }
}

#[test]
fn state_owner_discriminator_length_version_padding_and_rent_failures() {
    for index in [CONFIG, REWARD] {
        assert_rejected("state-owner", |f, _| f.accounts[index].1.owner = new_key(244), failure(Piv1Error::InvalidAccountOwner));
        assert_rejected("state-discriminator", |f, _| f.accounts[index].1.data[0] ^= 1, failure(Piv1Error::InvalidAccountDiscriminator));
        assert_rejected("state-length", |f, _| { f.accounts[index].1.data.pop(); }, failure(Piv1Error::InvalidAccountSize));
        assert_rejected("state-version", |f, _| f.accounts[index].1.data[8] = 9, failure(Piv1Error::InvalidVersion));
        assert_rejected("state-rent", |f, _| f.accounts[index].1.lamports -= 1, failure(Piv1Error::AccountRentDeficit));
    }
    assert_rejected("reward-zero-tail", |f, _| *f.accounts[REWARD].1.data.last_mut().unwrap() = 1,
        failure(Piv1Error::InvalidAccountData));
}

#[test]
fn wrong_pdas_bumps_guardian_and_native_custody_failures() {
    // Every changed key remains unique, so runtime account compilation cannot
    // silently select another supplied entry.
    for index in [CONFIG, REWARD, KIF] {
        assert_rejected("wrong-role-key", |f, ix| { f.accounts[index].0 = new_key(245); ix.accounts[index].pubkey = new_key(245); },
            failure(Piv1Error::InvalidAccountPda));
    }
    assert_rejected("wrong-guardian", |f, ix| { f.accounts[GUARDIAN].0 = new_key(246); ix.accounts[GUARDIAN].pubkey = new_key(246); },
        failure(Piv1Error::InvalidGuardianSet));
    assert_rejected("wrong-config-bump", |f, _| edit_config(&mut f.accounts, |c| c.bumps.config ^= 1), failure(Piv1Error::InvalidAccountPda));
    assert_rejected("wrong-reward-bump", |f, _| edit_reward(&mut f.accounts, |r| r.bump ^= 1), failure(Piv1Error::InvalidAccountPda));
    for index in [KIF, GUARDIAN] {
        assert_rejected("native-owner", |f, _| f.accounts[index].1.owner = new_key(247), failure(Piv1Error::InvalidAccountOwner));
        assert_rejected("native-data", |f, _| f.accounts[index].1.data.push(1), failure(Piv1Error::InvalidAccountSize));
    }
}

#[test]
fn duplicate_instruction_role_and_static_piv_role_alias_reject() {
    assert_rejected("duplicate-role-meta", |_, ix| ix.accounts[GUARDIAN].pubkey = ix.accounts[CONFIG].pubkey,
        failure(Piv1Error::AccountAlias));
    assert_rejected("static-piv-destination", |f, _| {
        let guardian = original_fixture::key(91);
        edit_config(&mut f.accounts, |c| c.piv_authority = guardian);
    }, failure(Piv1Error::AccountAlias));
}

#[test]
fn system_program_identity_and_executable_economic_accounts_reject() {
    assert_rejected("wrong-system-key", |f, ix| {
        f.accounts[SYSTEM].0 = new_key(248); ix.accounts[SYSTEM].pubkey = new_key(248);
    }, failure(Piv1Error::InvalidProgramIdentity));
    assert_rejected("non-executable-system", |f, _| f.accounts[SYSTEM].1.executable = false,
        failure(Piv1Error::InvalidProgramIdentity));
    for index in [CONFIG, REWARD, KIF, GUARDIAN] {
        assert_rejected("executable-role", |f, _| f.accounts[index].1.executable = true,
            failure(Piv1Error::ExecutableAccount));
    }
}

#[test]
fn reduced_compute_errors_record_the_actual_observed_boundary() {
    // Fixed bounded probes, each with a freshly consistent budget/cache/ELF.
    // No desired late-failure boundary is assumed or manufactured.
    for budget in [1, 5_000, 10_000, 20_000] {
        let runtime = Runtime::new(budget);
        let fixture = Fixture::new(&runtime.vm.sysvars.rent, 0, false);
        let original = fixture.clone();
        let result = runtime.execute(&format!("compute-probe-{budget}"), &fixture, &[fixture.instruction(100, 10)]);
        if result.raw_result.is_ok() {
            assert_eq!(result.resulting_accounts, fixture.expected_payment(100));
            assert_one_transfer(&result, &fixture, 100);
            assert_one_event(&runtime.observations.borrow().after[0].logs, &fixture, 100);
        } else {
            // Meter exhaustion from a syscall preserves InstructionError;
            // VM instruction-meter exhaustion is wrapped as failed-to-complete.
            assert_eq!(result.resulting_accounts, original.accounts);
            let observations = runtime.observations.borrow();
            assert_eq!(observations.after.len(), 1);
            assert!(is_compute_exhaustion(&result.raw_result, &observations.after[0].logs));
            let raw = &observations.after[0].accounts;
            let source = &raw.iter().find(|(k, _)| *k == fixture.accounts[KIF].0).unwrap().1;
            let state = &raw.iter().find(|(k, _)| *k == fixture.accounts[CONFIG].0).unwrap().1;
            println!("COMPUTE_BOUNDARY budget={budget} observed_state_changed={} observed_source_debit={} inner_count={}",
                state.data != fixture.accounts[CONFIG].1.data,
                fixture.accounts[KIF].1.lamports.checked_sub(source.lamports).unwrap(),
                result.inner_instructions.iter().map(Vec::len).sum::<usize>());
        }
        assert_eq!(fixture, original);
    }
}
