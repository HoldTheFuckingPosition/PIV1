mod support;

use anchor_lang::{
    prelude::{AccountInfo, Pubkey, Rent},
    solana_program::{entrypoint::ProgramResult, instruction::Instruction, program_error::ProgramError, system_program},
    AnchorDeserialize,
};
use piv1::{
    errors::Piv1Error,
    kif_claim_accounts::KifClaimAccountInfos,
    kif_claim_execution::*,
    state::{GuardianReward, KifClaimRequest},
};
use support::kif_claim_custody::{
    envelope, key, BackingAccount, Fixture as ClaimFixture, PROGRAM,
    CONFIG, REWARD, KIF, GUARDIAN,
};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum Behavior {
    Transfer, ErrorBefore, ErrorAfterDebit, ErrorAfterTransfer, FalseSuccess,
    TooLittle, TooMuch, WrongSource, WrongDestination, ConfigTamper, RewardTamper,
    ConfigDiscriminator, RewardDiscriminator, RewardOption, StateRentChanged,
    MalformedAndWrongPayment, WrongPaymentAndStateTamper, StateBytesAndLamports,
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct Invocation {
    program: Pubkey,
    data: Vec<u8>,
    metas: Vec<(Pubkey, bool, bool)>,
    accounts: Vec<(Pubkey, bool, bool)>,
    seeds: Vec<Vec<Vec<u8>>>,
    config_at_call: Vec<u8>,
    reward_at_call: Vec<u8>,
    source_at_call: u64,
    destination_at_call: u64,
}

#[derive(Clone, Debug, PartialEq)]
struct Fixture { claim: ClaimFixture, system: BackingAccount }
impl Fixture {
    fn new(excess: u64) -> Self { Self::from_claim(ClaimFixture::new(excess)) }
    fn from_claim(claim: ClaimFixture) -> Self {
        Self { claim, system: BackingAccount { key: system_program::ID,
            // Synthetic executable host account, not a loaded runtime program.
            owner: key(219), executable: true, signer: false, writable: false,
            lamports: 1, data: vec![1] } }
    }
    fn with_infos<T>(&mut self, f: impl FnOnce(KifClaimExecutionAccounts<'_, '_>) -> T) -> T {
        let [c, r, k, g] = &mut self.claim.accounts;
        let c = c.info(); let r = r.info(); let k = k.info(); let g = g.info(); let s = self.system.info();
        f(KifClaimExecutionAccounts { claim: KifClaimAccountInfos {
            config: &c, guardian_reward: &r, kif_sol: &k, guardian: &g }, system_program: &s })
    }
    fn request(&self, amount: u64) -> KifClaimRequest {
        KifClaimRequest { amount_lamports: amount,
            expected_cumulative_claimed: self.claim.reward().cumulative_claimed }
    }
    fn raw(&mut self, request: KifClaimRequest, behavior: Behavior) -> (KifClaimExecutionResult, Vec<Invocation>) {
        let mut calls = Vec::new();
        let before = self.clone(); let program = self.claim.program_id; let rent = self.claim.rent.clone();
        let result = self.with_infos(|a| execute_kif_claim_with_host_invoker(&program, &rent, a, request,
            |instruction, infos, seeds| emulate(&before, a.claim, request, behavior,
                instruction, infos, seeds, &mut calls)));
        (result, calls)
    }
    /// Explicit host transaction model. The baseline is cloned unchanged, never
    /// recomputed. Only actual modeled payment flow advances the existing audit.
    /// Discarding the staged clone on error is not evidence of SVM rollback.
    fn transaction(&mut self, request: KifClaimRequest, behavior: Behavior) -> (KifClaimExecutionResult, Vec<Invocation>) {
        let mut staged = self.clone();
        let (result, calls) = staged.raw(request, behavior);
        if result.is_ok() {
            let paid = self.claim.accounts[KIF].lamports.checked_sub(staged.claim.accounts[KIF].lamports).unwrap();
            let received = staged.claim.accounts[GUARDIAN].lamports.checked_sub(self.claim.accounts[GUARDIAN].lamports).unwrap();
            assert_eq!(paid, received);
            staged.claim.audit.paid = staged.claim.audit.paid.checked_add(u128::from(paid)).unwrap();
            staged.claim.validate_audit().unwrap();
            *self = staged;
        }
        (result, calls)
    }
    fn reject_before(&mut self, request: KifClaimRequest, error: Piv1Error) {
        let before = self.clone();
        let (result, calls) = self.raw(request, Behavior::Transfer);
        assert_eq!(result, Err(KifClaimExecutionError::State(error)));
        assert!(calls.is_empty()); assert_eq!(*self, before);
    }
}

fn amount(account: &AccountInfo<'_>) -> u64 { **account.try_borrow_lamports().unwrap() }
fn move_native(source: &AccountInfo<'_>, destination: &AccountInfo<'_>, value: u64) {
    let left = amount(source).checked_sub(value).unwrap();
    let right = amount(destination).checked_add(value).unwrap();
    **source.try_borrow_mut_lamports().unwrap() = left;
    **destination.try_borrow_mut_lamports().unwrap() = right;
}

fn emulate(
    before: &Fixture, claim: KifClaimAccountInfos<'_, '_>, request: KifClaimRequest,
    behavior: Behavior, instruction: &Instruction, infos: &[AccountInfo<'_>],
    seeds: &[&[&[u8]]], calls: &mut Vec<Invocation>,
) -> ProgramResult {
    let mut expected_c = before.claim.config(); let mut expected_r = before.claim.reward();
    expected_c.kif_claim_liability_lamports -= request.amount_lamports;
    expected_c.cumulative_kif_claimed_lamports += request.amount_lamports;
    expected_r.claimable_lamports -= request.amount_lamports;
    expected_r.cumulative_claimed += request.amount_lamports;
    // Capture state using the host closure only. The invocation's actual account
    // list contains exactly source, destination and System, never these states.
    let config_at_call = claim.config.try_borrow_data().unwrap().to_vec();
    let reward_at_call = claim.guardian_reward.try_borrow_data().unwrap().to_vec();
    assert_eq!(config_at_call, envelope(&expected_c, [98,115,11,164,170,207,163,20], 1014));
    assert_eq!(reward_at_call, envelope(&expected_r, [169,109,89,17,75,171,105,39], 84));
    for account in [claim.config, claim.guardian_reward, claim.kif_sol, claim.guardian] {
        assert!(account.try_borrow_mut_data().is_ok(), "no retained execution data guard");
        assert!(account.try_borrow_mut_lamports().is_ok(), "no retained execution native guard");
    }
    assert_eq!(infos.len(), 3);
    assert_eq!(infos[0].key, claim.kif_sol.key); assert_eq!(infos[1].key, claim.guardian.key);
    assert_eq!(*infos[2].key, system_program::ID);
    assert_eq!(seeds.len(), 1); assert_eq!(seeds[0].len(), 2); assert_eq!(seeds[0][0], b"kif-sol");
    let (source, bump) = Pubkey::find_program_address(&[b"kif-sol"], &before.claim.program_id);
    assert_eq!(seeds[0][1], [bump]);
    assert_eq!(Pubkey::create_program_address(seeds[0], &before.claim.program_id).unwrap(), source);
    assert_eq!(source, *claim.kif_sol.key); assert_ne!(source, before.claim.config().piv_authority);
    let mut expected_data = 2_u32.to_le_bytes().to_vec(); // official SystemInstruction::Transfer
    expected_data.extend(request.amount_lamports.to_le_bytes());
    assert_eq!(instruction.program_id, system_program::ID); assert_eq!(instruction.data, expected_data);
    let metas: Vec<_> = instruction.accounts.iter().map(|m| (m.pubkey, m.is_signer, m.is_writable)).collect();
    assert_eq!(metas, vec![(source, true, true), (*claim.guardian.key, false, true)]);
    assert_eq!(amount(claim.kif_sol), before.claim.accounts[KIF].lamports);
    assert_eq!(amount(claim.guardian), before.claim.accounts[GUARDIAN].lamports);
    calls.push(Invocation { program: instruction.program_id, data: instruction.data.clone(), metas,
        accounts: infos.iter().map(|a| (*a.key, a.is_signer, a.is_writable)).collect(),
        seeds: seeds.iter().map(|g| g.iter().map(|s| s.to_vec()).collect()).collect(),
        config_at_call, reward_at_call, source_at_call: amount(claim.kif_sol),
        destination_at_call: amount(claim.guardian) });
    if behavior == Behavior::ErrorBefore { return Err(ProgramError::Custom(772)); }
    if behavior == Behavior::FalseSuccess { return Ok(()); }
    if behavior == Behavior::ErrorAfterDebit {
        **claim.kif_sol.try_borrow_mut_lamports().unwrap() -= request.amount_lamports;
        return Err(ProgramError::InsufficientFunds);
    }
    match behavior {
        Behavior::TooLittle | Behavior::MalformedAndWrongPayment
            | Behavior::WrongPaymentAndStateTamper => move_native(claim.kif_sol, claim.guardian, request.amount_lamports - 1),
        Behavior::TooMuch => move_native(claim.kif_sol, claim.guardian, request.amount_lamports + 1),
        Behavior::WrongSource => move_native(claim.config, claim.guardian, request.amount_lamports),
        Behavior::WrongDestination => move_native(claim.kif_sol, claim.config, request.amount_lamports),
        _ => move_native(claim.kif_sol, claim.guardian, request.amount_lamports),
    }
    match behavior {
        Behavior::ErrorAfterTransfer => return Err(ProgramError::Custom(991)),
        Behavior::ConfigTamper => { claim.config.try_borrow_mut_data().unwrap()[10] ^= 1; }
        Behavior::RewardTamper => {
            let mut g = GuardianReward::deserialize(&mut &claim.guardian_reward.data.borrow()[8..]).unwrap();
            g.last_active_period = Some(999);
            claim.guardian_reward.try_borrow_mut_data().unwrap().copy_from_slice(
                &envelope(&g, [169,109,89,17,75,171,105,39], 84));
        }
        Behavior::ConfigDiscriminator | Behavior::MalformedAndWrongPayment => { claim.config.try_borrow_mut_data().unwrap()[0] ^= 1; }
        Behavior::RewardDiscriminator => { claim.guardian_reward.try_borrow_mut_data().unwrap()[0] ^= 1; }
        Behavior::RewardOption => { claim.guardian_reward.try_borrow_mut_data().unwrap()[51] = 2; }
        Behavior::StateRentChanged => { **claim.config.try_borrow_mut_lamports().unwrap() += 1; }
        Behavior::WrongPaymentAndStateTamper | Behavior::StateBytesAndLamports => {
            claim.config.try_borrow_mut_data().unwrap()[10] ^= 1;
            **claim.config.try_borrow_mut_lamports().unwrap() += 1;
        }
        _ => {}
    }
    Ok(())
}

#[test]
fn official_instruction_endpoints_metas_seed_group_and_cei_state_are_exact() {
    let mut f = Fixture::new(7); let before = f.clone(); let request = f.request(100);
    let (result, calls) = f.transaction(request, Behavior::Transfer); let paid = result.unwrap();
    assert_eq!(paid.amount_lamports, 100); assert_eq!(calls.len(), 1);
    let call = &calls[0];
    assert_eq!(call.data, [2,0,0,0,100,0,0,0,0,0,0,0]);
    assert_eq!(call.accounts, vec![(paid.source, false, true), (paid.destination, true, true),
        (system_program::ID, false, false)]);
    assert_eq!(call.source_at_call, before.claim.accounts[KIF].lamports);
    assert_eq!(call.destination_at_call, before.claim.accounts[GUARDIAN].lamports);
    let mut expected = before;
    expected.claim.update_config(|c| { c.kif_claim_liability_lamports -= 100; c.cumulative_kif_claimed_lamports += 100; });
    expected.claim.update_reward(|r| { r.claimable_lamports -= 100; r.cumulative_claimed += 100; });
    expected.claim.accounts[KIF].lamports -= 100; expected.claim.accounts[GUARDIAN].lamports += 100;
    expected.claim.audit.paid += 100;
    assert_eq!(f, expected); f.claim.validate_audit().unwrap();
}

#[test]
fn partial_then_full_historical_inactive_claims_preserve_pause_carry_rent_and_excess() {
    for paused in [false, true] {
        let mut base = ClaimFixture::new(0); base.update_config(|c| c.paused = paused);
        let mut f = Fixture::from_claim(base);
        assert_ne!(f.claim.config().guardian_registry_revision, f.claim.reward().registry_revision);
        assert_eq!(f.claim.reward().last_active_period, None);
        let initial = f.clone();
        f.claim.unsolicited_excess(29).unwrap();
        let first = f.request(123); assert!(f.transaction(first, Behavior::Transfer).0.is_ok());
        f.reject_before(first, Piv1Error::StaleKifClaim);
        let second = f.request(177); assert!(f.transaction(second, Behavior::Transfer).0.is_ok());
        assert_eq!(f.claim.reward().claimable_lamports, 0);
        assert_eq!(f.claim.audit.paid, 300); assert_eq!(f.claim.audit.external_excess, 29);
        assert_eq!(f.claim.config().collective_kif_carry_lamports, 73);
        assert_eq!(f.claim.accounts[KIF].lamports,
            f.claim.rent.minimum_balance(0) + 660 + 73 + 29);
        assert_eq!(f.claim.accounts[CONFIG].lamports, initial.claim.accounts[CONFIG].lamports);
        assert_eq!(f.claim.accounts[REWARD].lamports, initial.claim.accounts[REWARD].lamports);
        assert_eq!(f.system, initial.system); f.claim.validate_audit().unwrap();
        f.reject_before(f.request(1), Piv1Error::KifClaimExceeded);
    }
}

#[test]
fn full_global_claim_leaves_exact_rent_and_collective_carry_without_excess() {
    let base = ClaimFixture::new(0); let c = base.config(); let mut r = base.reward();
    r.claimable_lamports = c.kif_claim_liability_lamports;
    r.cumulative_earned = r.claimable_lamports + r.cumulative_claimed;
    let mut f = Fixture::from_claim(ClaimFixture::from_earned(c, r, 0));
    let request = f.request(960); f.transaction(request, Behavior::Transfer).0.unwrap();
    assert_eq!(f.claim.config().kif_claim_liability_lamports, 0);
    assert_eq!(f.claim.accounts[KIF].lamports, f.claim.rent.minimum_balance(0) + 73);
    f.claim.validate_audit().unwrap();
}

#[test]
fn later_modeled_credit_keeps_original_baseline_and_cannot_reenable_old_request() {
    let mut f = Fixture::new(0); let request = f.request(100);
    f.transaction(request, Behavior::Transfer).0.unwrap();
    f.claim.credit_snapshot(100).unwrap();
    assert_eq!(f.claim.reward().claimable_lamports, 300);
    f.reject_before(request, Piv1Error::StaleKifClaim);
    let request = f.request(300); f.transaction(request, Behavior::Transfer).0.unwrap();
    assert_eq!(f.claim.audit.paid, 400); assert_eq!(f.claim.audit.credited, 100);
    f.claim.validate_audit().unwrap();
}

#[test]
fn runtime_facing_function_rejects_hosts_without_access_invocation_or_mutation() {
    let mut f = Fixture::new(0); let before = f.clone();
    f.with_infos(|a| {
        let _held_config = a.claim.config.try_borrow_mut_data().unwrap();
        let _held_source = a.claim.kif_sol.try_borrow_mut_lamports().unwrap();
        assert_eq!(execute_kif_claim(&PROGRAM, &Rent::default(), a,
            KifClaimRequest { amount_lamports: 100, expected_cumulative_claimed: 10 }),
            Err(KifClaimExecutionError::HostRuntimeUnavailable));
    });
    assert_eq!(f, before);
}

#[test]
fn native_shared_and_mutable_data_and_lamport_conflicts_reject_before_bookkeeping() {
    for destination in [false, true] {
        for kind in 0..4 {
            let mut f = Fixture::new(0); let before = f.clone(); let request = f.request(100); let mut calls = 0;
            f.with_infos(|a| {
                let account = if destination { a.claim.guardian } else { a.claim.kif_sol };
                let data_read = (kind == 0).then(|| account.try_borrow_data().unwrap());
                let data_write = (kind == 1).then(|| account.try_borrow_mut_data().unwrap());
                let native_read = (kind == 2).then(|| account.try_borrow_lamports().unwrap());
                let native_write = (kind == 3).then(|| account.try_borrow_mut_lamports().unwrap());
                assert_eq!(execute_kif_claim_with_host_invoker(&PROGRAM, &Rent::default(), a, request,
                    |_, _, _| { calls += 1; Ok(()) }), Err(KifClaimExecutionError::State(Piv1Error::AccountBorrowFailed)));
                drop((data_read, data_write, native_read, native_write));
                for account in [a.claim.config, a.claim.guardian_reward, a.claim.kif_sol, a.claim.guardian] {
                    assert!(account.try_borrow_mut_data().is_ok()); assert!(account.try_borrow_mut_lamports().is_ok());
                }
            });
            assert_eq!(calls, 0); assert_eq!(f, before);
        }
    }
}

#[test]
fn second_state_borrow_failure_is_atomic_and_never_invokes() {
    for index in [CONFIG, REWARD] {
        for kind in 0..3 {
            let mut f = Fixture::new(0); let before = f.clone(); let request = f.request(100); let mut calls = 0;
            f.with_infos(|a| {
                let state = if index == CONFIG { a.claim.config } else { a.claim.guardian_reward };
                let read = (kind == 0).then(|| state.try_borrow_data().unwrap());
                let write = (kind == 1).then(|| state.try_borrow_mut_data().unwrap());
                let native = (kind == 2).then(|| state.try_borrow_mut_lamports().unwrap());
                assert_eq!(execute_kif_claim_with_host_invoker(&PROGRAM, &Rent::default(), a, request,
                    |_, _, _| { calls += 1; Ok(()) }), Err(KifClaimExecutionError::State(Piv1Error::AccountBorrowFailed)));
                drop((read, write, native));
            });
            assert_eq!(calls, 0); assert_eq!(f, before);
        }
    }
}

#[test]
fn wrong_nonexecutable_and_role_aliased_system_program_reject_before_effects() {
    for bad in 0..6 {
        let mut f = Fixture::new(0); let request = f.request(100);
        let error = match bad {
            0 => { f.system.key = key(218); Piv1Error::InvalidProgramIdentity }
            1 => { f.system.executable = false; Piv1Error::InvalidProgramIdentity }
            i => { f.system.key = f.claim.accounts[i - 2].key; Piv1Error::AccountAlias }
        };
        f.reject_before(request, error);
    }
}

#[test]
fn requests_and_checked_arithmetic_fail_with_zero_invocations() {
    let mut f = Fixture::new(0); f.reject_before(f.request(0), Piv1Error::ZeroKifClaim);
    f.reject_before(f.request(301), Piv1Error::KifClaimExceeded);
    f.reject_before(KifClaimRequest { amount_lamports: 1, expected_cumulative_claimed: 9 }, Piv1Error::StaleKifClaim);
    let mut f = Fixture::new(0); f.claim.accounts[GUARDIAN].lamports = u64::MAX;
    f.reject_before(f.request(1), Piv1Error::ArithmeticOverflow);
    let mut f = Fixture::new(0);
    f.claim.update_config(|c| { c.kif_claim_liability_lamports = u64::MAX;
        c.cumulative_kif_credited_lamports = u64::MAX; c.cumulative_kif_claimed_lamports = 0; });
    // Selected cumulative claim component also needs to fit the supplied global.
    f.claim.update_reward(|r| { r.cumulative_earned = r.claimable_lamports; r.cumulative_claimed = 0; });
    f.reject_before(f.request(1), Piv1Error::ArithmeticOverflow);
}

#[test]
fn full_source_backing_is_required_even_with_destination_surplus() {
    for deficit in [1, 73, 300] {
        let mut f = Fixture::new(0); f.claim.accounts[KIF].lamports -= deficit;
        f.claim.accounts[GUARDIAN].lamports = 1_000_000;
        assert!(f.claim.accounts[KIF].lamports > 100);
        f.reject_before(f.request(100), Piv1Error::KifClaimBackingDeficit);
    }
    let mut f = Fixture::new(0); f.claim.accounts[KIF].lamports = f.claim.rent.minimum_balance(0) - 1;
    f.reject_before(f.request(1), Piv1Error::AccountRentDeficit);
}

#[test]
fn composed_account_privilege_owner_identity_and_shape_errors_never_invoke() {
    for index in 0..4 {
        for kind in 0..5 {
            let mut f = Fixture::new(0); let request = f.request(100);
            let error = match kind {
                0 => { f.claim.accounts[index].owner = key(202); Piv1Error::InvalidAccountOwner }
                1 => { f.claim.accounts[index].writable = false; Piv1Error::AccountNotWritable }
                2 => { f.claim.accounts[index].executable = true; Piv1Error::ExecutableAccount }
                3 => { f.claim.accounts[index].data.push(0); Piv1Error::InvalidAccountSize }
                _ => { f.claim.accounts[index].key = key(203);
                    if index == GUARDIAN { Piv1Error::InvalidGuardianSet } else { Piv1Error::InvalidAccountPda } }
            };
            f.reject_before(request, error);
        }
    }
    let mut f = Fixture::new(0); f.claim.accounts[GUARDIAN].signer = false;
    f.reject_before(f.request(100), Piv1Error::MissingGuardianSignature);
}

#[test]
fn malformed_envelopes_and_invalid_state_reject_before_encoding_or_persistence() {
    for index in [CONFIG, REWARD] {
        for kind in 0..3 {
            let mut f = Fixture::new(0); let request = f.request(100);
            let offset = match kind { 0 => 0, 1 => 8, _ => if index == CONFIG { 756 } else { 51 } };
            f.claim.accounts[index].data[offset] = 2;
            f.reject_before(request, match kind { 0 => Piv1Error::InvalidAccountDiscriminator,
                1 => Piv1Error::InvalidVersion, _ => Piv1Error::InvalidAccountData });
        }
    }
    let mut f = Fixture::new(0); *f.claim.accounts[REWARD].data.last_mut().unwrap() = 1;
    f.reject_before(f.request(100), Piv1Error::InvalidAccountData);
    let mut f = Fixture::new(0); f.claim.update_config(|c| c.kif_claim_liability_lamports += 1);
    f.reject_before(f.request(100), Piv1Error::CumulativeReconciliationMismatch);
    let mut f = Fixture::new(0); f.claim.update_reward(|r| r.claimable_lamports += 1);
    f.reject_before(f.request(100), Piv1Error::CumulativeReconciliationMismatch);
}

#[test]
fn every_cpi_and_postcheck_failure_rolls_back_complete_host_transaction_and_retries_once() {
    let cases = [
        (Behavior::ErrorBefore, KifClaimExecutionError::Invocation(ProgramError::Custom(772))),
        (Behavior::ErrorAfterDebit, KifClaimExecutionError::Invocation(ProgramError::InsufficientFunds)),
        (Behavior::ErrorAfterTransfer, KifClaimExecutionError::Invocation(ProgramError::Custom(991))),
        (Behavior::FalseSuccess, KifClaimExecutionError::State(Piv1Error::KifClaimObservationMismatch)),
        (Behavior::TooLittle, KifClaimExecutionError::State(Piv1Error::KifClaimObservationMismatch)),
        (Behavior::TooMuch, KifClaimExecutionError::State(Piv1Error::KifClaimObservationMismatch)),
        (Behavior::WrongSource, KifClaimExecutionError::State(Piv1Error::AccountRentDeficit)),
        (Behavior::WrongDestination, KifClaimExecutionError::State(Piv1Error::KifClaimObservationMismatch)),
        (Behavior::ConfigTamper, KifClaimExecutionError::State(Piv1Error::KifClaimStateChanged)),
        (Behavior::RewardTamper, KifClaimExecutionError::State(Piv1Error::KifClaimStateChanged)),
        (Behavior::ConfigDiscriminator, KifClaimExecutionError::State(Piv1Error::InvalidAccountDiscriminator)),
        (Behavior::RewardDiscriminator, KifClaimExecutionError::State(Piv1Error::InvalidAccountDiscriminator)),
        (Behavior::RewardOption, KifClaimExecutionError::State(Piv1Error::InvalidAccountData)),
        (Behavior::StateRentChanged, KifClaimExecutionError::State(Piv1Error::KifClaimObservationMismatch)),
    ];
    for (behavior, error) in cases {
        let mut f = Fixture::new(19); let before = f.clone(); let request = f.request(100);
        let (result, calls) = f.transaction(request, behavior);
        assert_eq!(result, Err(error), "{behavior:?}"); assert_eq!(calls.len(), 1);
        assert_eq!(f, before, "modeled transaction rollback: {behavior:?}");
        f.claim.validate_audit().unwrap();
        f.transaction(request, Behavior::Transfer).0.unwrap();
        assert_eq!(f.claim.audit.paid, 100);
        f.reject_before(request, Piv1Error::StaleKifClaim); f.claim.validate_audit().unwrap();
    }
}

#[test]
fn raw_execution_errors_expose_transaction_rollback_requirement() {
    for behavior in [Behavior::ErrorBefore, Behavior::FalseSuccess, Behavior::ErrorAfterTransfer] {
        let mut f = Fixture::new(0); let before = f.clone(); let request = f.request(100);
        let (result, calls) = f.raw(request, behavior); assert!(result.is_err()); assert_eq!(calls.len(), 1);
        assert_eq!(f.claim.config().kif_claim_liability_lamports, before.claim.config().kif_claim_liability_lamports - 100);
        assert_eq!(f.claim.reward().cumulative_claimed, before.claim.reward().cumulative_claimed + 100);
        assert_eq!(f.claim.audit, before.claim.audit); assert!(f.claim.validate_audit().is_err());
        if behavior == Behavior::ErrorAfterTransfer {
            assert_eq!(f.claim.accounts[KIF].lamports, before.claim.accounts[KIF].lamports - 100);
        } else { assert_eq!(f.claim.accounts[KIF].lamports, before.claim.accounts[KIF].lamports); }
        // Catching this error and retaining raw mutated accounts would be wrong.
        // Production relies on the eventual transaction boundary to roll back.
    }
}

#[test]
fn allowed_external_wallet_overlap_and_system_owned_pda_guardian_remain_supported() {
    for choice in 0..3 {
        let base = ClaimFixture::new(0); let c = base.config(); let mut r = base.reward();
        r.guardian = match choice { 0 => c.htfp_recipient, 1 => c.team_owner_recipient,
            _ => Pubkey::find_program_address(&[b"synthetic-guardian"], &key(211)).0 };
        if choice == 2 { assert!(!r.guardian.is_on_curve()); }
        let mut f = Fixture::from_claim(ClaimFixture::from_earned(c, r, 0));
        let request = f.request(100); f.transaction(request, Behavior::Transfer).0.unwrap();
        f.claim.validate_audit().unwrap();
    }
}

#[test]
fn unrelated_synthetic_config_history_and_custody_fields_remain_byte_exact() {
    let base = ClaimFixture::new(0); let mut c = base.config(); let r = base.reward();
    c.paused = true; c.next_distribution_sequence = u64::MAX;
    c.last_successful_preparation_at = Some(i64::MAX);
    c.last_valid_insufficient_attempt_at = Some(i64::MIN);
    c.accounted_pending_sol_lamports = u64::MAX; c.accounted_historical_sol_lamports = u64::MAX;
    // Synthetic unrelated values demonstrate isolation, not a physically healthy
    // distribution phase. No current registry, Clock, pool or round is provided.
    let mut f = Fixture::from_claim(ClaimFixture::from_earned(c, r, 0));
    let request = f.request(300); let before = f.clone();
    f.transaction(request, Behavior::Transfer).0.unwrap();
    let mut expected = before.claim.config(); expected.kif_claim_liability_lamports -= 300;
    expected.cumulative_kif_claimed_lamports += 300;
    assert_eq!(f.claim.config(), expected); f.claim.validate_audit().unwrap();
}

#[test]
fn trusted_program_rent_and_state_rent_fail_before_invocation() {
    for (program, error) in [(system_program::ID, Piv1Error::InvalidProgramIdentity),
        (spl_token::ID, Piv1Error::InvalidProgramIdentity),
        (piv1::accounts::STAKE_PROGRAM_ID, Piv1Error::InvalidProgramIdentity),
        (key(214), Piv1Error::InvalidAccountOwner)]
    {
        let mut f = Fixture::new(0); let request = f.request(100); f.claim.program_id = program;
        f.reject_before(request, error);
    }
    for (rent, error) in [
        (Rent { exemption_threshold: -1.0, ..Rent::default() }, Piv1Error::InvalidRent),
        (Rent { burn_percent: 101, ..Rent::default() }, Piv1Error::InvalidRent),
        (Rent { lamports_per_byte_year: u64::MAX, ..Rent::default() }, Piv1Error::ArithmeticOverflow),
    ] {
        let mut f = Fixture::new(0); let request = f.request(100); f.claim.rent = rent;
        f.reject_before(request, error);
    }
    for index in [CONFIG, REWARD] {
        let mut f = Fixture::new(0); let request = f.request(100); f.claim.accounts[index].lamports -= 1;
        f.reject_before(request, Piv1Error::AccountRentDeficit);
    }
}

#[test]
fn all_claim_role_key_aliases_and_static_piv_destinations_reject_before_effects() {
    for left in 0..4 {
        for right in left + 1..4 {
            let mut f = Fixture::new(0); let request = f.request(100);
            f.claim.accounts[right].key = f.claim.accounts[left].key;
            f.reject_before(request, Piv1Error::AccountAlias);
        }
    }
    let initial = Fixture::new(0); let c = initial.claim.config();
    for destination in [c.piv_authority, c.active_distribution, c.principal_jito_vault,
        c.pending_jito_vault, c.pending_sol_vault, c.principal_sol_queue, c.operational_sol_vault,
        c.distribution_escrow, c.kif_sol_vault, c.guardian_registry]
    {
        let mut f = initial.clone(); let request = f.request(100);
        f.claim.accounts[GUARDIAN].key = destination;
        f.reject_before(request, Piv1Error::AccountAlias);
    }
}

#[test]
fn differently_keyed_native_shared_backing_is_rejected_by_simultaneous_preflight() {
    for shared_data in [false, true] {
        let mut f = Fixture::new(0); let before = f.clone(); let request = f.request(100); let mut calls = 0;
        f.with_infos(|a| {
            let mut guardian = a.claim.guardian.clone();
            if shared_data { guardian.data = a.claim.kif_sol.data.clone(); }
            else { guardian.lamports = a.claim.kif_sol.lamports.clone(); }
            assert_ne!(guardian.key, a.claim.kif_sol.key);
            let a = KifClaimExecutionAccounts { claim: KifClaimAccountInfos { guardian: &guardian, ..a.claim }, ..a };
            assert_eq!(execute_kif_claim_with_host_invoker(&PROGRAM, &Rent::default(), a, request,
                |_, _, _| { calls += 1; Ok(()) }), Err(KifClaimExecutionError::State(Piv1Error::AccountBorrowFailed)));
        });
        assert_eq!(calls, 0); assert_eq!(f, before);
    }
}

#[test]
fn canonical_stored_bumps_and_validated_typed_before_states_remain_required() {
    for kind in 0..5 {
        let mut f = Fixture::new(0); let request = f.request(100);
        let error = match kind {
            0 => { f.claim.update_config(|c| c.bumps.config ^= 1); Piv1Error::InvalidAccountPda }
            1 => { f.claim.update_config(|c| c.bumps.kif_sol_vault ^= 1); Piv1Error::InvalidAccountPda }
            2 => { f.claim.update_reward(|r| r.bump ^= 1); Piv1Error::InvalidAccountPda }
            3 => { f.claim.update_config(|c| c.is_initialized = false); Piv1Error::InvalidInitialization }
            _ => { f.claim.update_config(|c| c.kif_bps += 1); Piv1Error::InvalidSplit }
        };
        f.reject_before(request, error);
    }
}

#[test]
fn postcheck_mixed_faults_preserve_auth_custody_bytes_and_lamport_precedence() {
    for (behavior, error) in [
        (Behavior::MalformedAndWrongPayment, Piv1Error::InvalidAccountDiscriminator),
        (Behavior::WrongPaymentAndStateTamper, Piv1Error::KifClaimObservationMismatch),
        (Behavior::StateBytesAndLamports, Piv1Error::KifClaimStateChanged),
    ] {
        let mut raw = Fixture::new(7); let before = raw.clone(); let request = raw.request(100);
        let (result, calls) = raw.raw(request, behavior);
        assert_eq!(result, Err(KifClaimExecutionError::State(error)));
        assert_eq!(calls.len(), 1);
        assert_ne!(raw, before, "raw failure retains effects until transaction rollback");
        assert_eq!(raw.claim.audit, before.claim.audit, "the original audit is never rebased");
        let mut transaction = before.clone();
        let (result, calls) = transaction.transaction(request, behavior);
        assert_eq!(result, Err(KifClaimExecutionError::State(error)));
        assert_eq!(calls.len(), 1);
        assert_eq!(transaction, before, "modeled rollback preserves every byte, lamport and audit field");
    }
}
