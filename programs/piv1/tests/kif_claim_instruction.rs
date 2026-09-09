mod support;

use std::{cell::RefCell, collections::BTreeSet};
use anchor_lang::{
    prelude::{AccountInfo, Pubkey, Rent},
    solana_program::{entrypoint::ProgramResult, program_error::ProgramError, system_program},
    AnchorDeserialize, Discriminator, Event,
};
use piv1::{
    errors::Piv1Error,
    events::KifClaimed,
    instruction_boundary::{process_instruction, process_instruction_with_host_callbacks},
    instruction_errors::{execution_program_error, piv1_error_code, HOST_RUNTIME_UNAVAILABLE_CODE},
    instructions::claim_kif::{decode_claim_kif, encode_claim_kif, CLAIM_KIF_DISCRIMINATOR, CLAIM_KIF_INSTRUCTION_SIZE},
    kif_claim_execution::KifClaimExecutionError,
    state::{GuardianReward, KifClaimRequest, PivConfig},
};
use support::kif_claim_custody::{
    key, BackingAccount, Fixture as ClaimFixture, PROGRAM, CONFIG, REWARD, KIF, GUARDIAN,
};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum Mode { Transfer, InvocationError, NoPayment, WrongAmount, StateTamper, MalformedPost }
#[derive(Clone, Debug, Default, Eq, PartialEq)]
struct Trace { order: Vec<&'static str>, events: Vec<KifClaimed> }

#[derive(Clone, Debug, PartialEq)]
struct Fixture { claim: ClaimFixture, system: BackingAccount }
impl Fixture {
    fn new(paused: bool, excess: u64) -> Self {
        let mut claim = ClaimFixture::new(excess); claim.update_config(|c| c.paused = paused);
        Self { claim, system: BackingAccount { key: system_program::ID, owner: key(219),
            executable: true, signer: false, writable: false, lamports: 1, data: vec![1] } }
    }
    fn with_accounts<T>(&mut self, action: impl FnOnce(&[AccountInfo<'_>; 5]) -> T) -> T {
        let [c, r, k, g] = &mut self.claim.accounts;
        let accounts = [c.info(), r.info(), k.info(), g.info(), self.system.info()];
        action(&accounts)
    }
    fn data(&self, amount: u64) -> [u8; 24] {
        encode_claim_kif(KifClaimRequest { amount_lamports: amount,
            expected_cumulative_claimed: self.claim.reward().cumulative_claimed })
    }
    fn raw(&mut self, data: &[u8], mode: Mode) -> (ProgramResult, Trace) {
        let trace = RefCell::new(Trace::default());
        let program = self.claim.program_id; let rent = self.claim.rent.clone(); let before = self.clone();
        let result = self.with_accounts(|accounts| process_instruction_with_host_callbacks(&program, accounts, data,
            || { trace.borrow_mut().order.push("rent"); Ok(rent) },
            |instruction, infos, seeds| {
                trace.borrow_mut().order.push("invoke");
                let request = decode_claim_kif(data).unwrap();
                assert_eq!(infos.len(), 3);
                assert_eq!(*infos[0].key, before.claim.accounts[KIF].key);
                assert_eq!(*infos[1].key, before.claim.accounts[GUARDIAN].key);
                assert_eq!(*infos[2].key, system_program::ID);
                assert_eq!(instruction.program_id, system_program::ID);
                let mut expected_bytes = 2_u32.to_le_bytes().to_vec();
                expected_bytes.extend(request.amount_lamports.to_le_bytes());
                assert_eq!(instruction.data, expected_bytes);
                assert_eq!(seeds.len(), 1); assert_eq!(seeds[0].len(), 2); assert_eq!(seeds[0][0], b"kif-sol");
                assert_eq!(Pubkey::create_program_address(seeds[0], &program).unwrap(), *infos[0].key);
                let c = PivConfig::deserialize(&mut &accounts[CONFIG].data.borrow()[8..]).unwrap();
                let r = GuardianReward::deserialize(&mut &accounts[REWARD].data.borrow()[8..]).unwrap();
                let mut expected_c = before.claim.config(); let mut expected_r = before.claim.reward();
                expected_c.kif_claim_liability_lamports -= request.amount_lamports;
                expected_c.cumulative_kif_claimed_lamports += request.amount_lamports;
                expected_r.claimable_lamports -= request.amount_lamports;
                expected_r.cumulative_claimed += request.amount_lamports;
                assert_eq!(c, expected_c); assert_eq!(r, expected_r);
                assert!(trace.borrow().events.is_empty(), "no event before successful postchecks");
                for account in &accounts[..4] {
                    assert!(account.try_borrow_mut_data().is_ok()); assert!(account.try_borrow_mut_lamports().is_ok());
                }
                if mode == Mode::InvocationError { return Err(ProgramError::Custom(987_654)); }
                if mode == Mode::NoPayment { return Ok(()); }
                let amount = request.amount_lamports + u64::from(mode == Mode::WrongAmount);
                **infos[0].try_borrow_mut_lamports().unwrap() -= amount;
                **infos[1].try_borrow_mut_lamports().unwrap() += amount;
                if mode == Mode::StateTamper { accounts[CONFIG].try_borrow_mut_data().unwrap()[10] ^= 1; }
                if mode == Mode::MalformedPost { accounts[REWARD].try_borrow_mut_data().unwrap()[0] ^= 1; }
                Ok(())
            },
            |event| {
                // Actual account balances already reflect the completed payment.
                assert_eq!(**accounts[KIF].lamports.borrow(), before.claim.accounts[KIF].lamports - event.amount_lamports);
                assert_eq!(**accounts[GUARDIAN].lamports.borrow(), before.claim.accounts[GUARDIAN].lamports + event.amount_lamports);
                let mut trace = trace.borrow_mut(); trace.order.push("event"); trace.events.push(event);
            }));
        (result, trace.into_inner())
    }
    /// Explicit host transaction clone. No SVM rollback is supplied by dispatch.
    /// The original audit baseline is retained; only measured payment flow grows.
    fn transaction(&mut self, data: &[u8], mode: Mode) -> (ProgramResult, Trace) {
        let mut staged = self.clone(); let (result, trace) = staged.raw(data, mode);
        if result.is_ok() {
            let paid = self.claim.accounts[KIF].lamports.checked_sub(staged.claim.accounts[KIF].lamports).unwrap();
            assert_eq!(staged.claim.accounts[GUARDIAN].lamports - self.claim.accounts[GUARDIAN].lamports, paid);
            staged.claim.audit.paid += u128::from(paid); staged.claim.validate_audit().unwrap();
            *self = staged;
        }
        (result, trace)
    }
}

#[test]
fn claim_wire_is_exactly_24_bytes_with_independent_literal_discriminator_and_endianness() {
    assert_eq!(CLAIM_KIF_DISCRIMINATOR, [0xfd,0x97,0xac,0x0b,0xca,0x4d,0x76,0xaa]);
    assert_eq!(CLAIM_KIF_INSTRUCTION_SIZE, 24);
    let request = KifClaimRequest { amount_lamports: 0x0102_0304_0506_0708,
        expected_cumulative_claimed: 0x1112_1314_1516_1718 };
    let expected = [0xfd,0x97,0xac,0x0b,0xca,0x4d,0x76,0xaa,
        8,7,6,5,4,3,2,1,0x18,0x17,0x16,0x15,0x14,0x13,0x12,0x11];
    assert_eq!(encode_claim_kif(request), expected); assert_eq!(decode_claim_kif(&expected), Ok(request));
    for amount in [0, 1, u64::MAX - 1, u64::MAX] {
        for counter in [0, 1, u64::MAX - 1, u64::MAX] {
            let request = KifClaimRequest { amount_lamports: amount, expected_cumulative_claimed: counter };
            assert_eq!(decode_claim_kif(&encode_claim_kif(request)), Ok(request));
        }
    }
}

#[test]
fn all_short_lengths_extra_lengths_unknown_idl_and_event_cpi_tags_are_rejected() {
    let valid = encode_claim_kif(KifClaimRequest { amount_lamports: 1, expected_cumulative_claimed: 2 });
    for length in 0..24 { assert_eq!(decode_claim_kif(&valid[..length]), Err(ProgramError::InvalidInstructionData)); }
    for length in 25..=1024 {
        let mut extra = valid.to_vec(); extra.resize(length, 0);
        assert_eq!(decode_claim_kif(&extra), Err(ProgramError::InvalidInstructionData));
    }
    for i in 0..8 {
        for byte in 0..=255 {
            if byte == valid[i] { continue; }
            let mut bad = valid; bad[i] = byte;
            assert_eq!(decode_claim_kif(&bad), Err(ProgramError::InvalidInstructionData));
        }
    }
    for tag in [[0; 8], anchor_lang::idl::IDL_IX_TAG.to_le_bytes(), anchor_lang::event::EVENT_IX_TAG_LE.try_into().unwrap()] {
        let mut bad = valid; bad[..8].copy_from_slice(&tag);
        assert_eq!(decode_claim_kif(&bad), Err(ProgramError::InvalidInstructionData));
    }
}

#[test]
fn event_bytes_are_exactly_discriminator_two_public_keys_and_paid_lamports() {
    assert_eq!(KifClaimed::DISCRIMINATOR, [4,201,171,251,215,119,17,208]);
    for amount in [0, 1, u64::MAX] {
        let event = KifClaimed { guardian_reward: key(0x11), guardian: key(0x22), amount_lamports: amount };
        let mut expected = vec![4,201,171,251,215,119,17,208];
        expected.extend([0x11; 32]); expected.extend([0x22; 32]); expected.extend(amount.to_le_bytes());
        assert_eq!(event.data().len(), 80); assert_eq!(event.data(), expected);
        assert_eq!(KifClaimed::try_from_slice(&expected[8..]).unwrap(), event);
    }
    // These pure byte boundaries do not claim that a zero payment can execute.
}

#[test]
fn malformed_decode_and_exact_count_precede_every_callback_or_account_access() {
    for count in 0..=7 {
        let mut f = Fixture::new(false, 0); let before = f.clone(); let valid = f.data(100);
        f.with_accounts(|five| {
            let accounts: Vec<_> = (0..count).map(|i| five[i % 5].clone()).collect();
            let held: Vec<_> = five.iter().map(|a| a.try_borrow_mut_data().unwrap()).collect();
            assert_eq!(process_instruction(&PROGRAM, &accounts, &valid[..23]), Err(ProgramError::InvalidInstructionData));
            assert_eq!(process_instruction_with_host_callbacks(&PROGRAM, &accounts, &valid[..23],
                || panic!("Rent accessed"), |_,_,_| panic!("invoked"), |_| panic!("event")), Err(ProgramError::InvalidInstructionData));
            if count != 5 {
                let error = if count < 5 { ProgramError::NotEnoughAccountKeys } else { ProgramError::InvalidArgument };
                assert_eq!(process_instruction(&PROGRAM, &accounts, &valid), Err(error.clone()));
                assert_eq!(process_instruction_with_host_callbacks(&PROGRAM, &accounts, &valid,
                    || panic!("Rent accessed"), |_,_,_| panic!("invoked"), |_| panic!("event")), Err(error));
            } else {
                assert_eq!(process_instruction(&PROGRAM, &accounts, &valid), Err(ProgramError::Custom(6999)));
            }
            drop(held);
        });
        assert_eq!(f, before);
    }
}

#[test]
fn ordinary_host_rejection_precedes_runtime_rent_and_even_invalid_execution_inputs() {
    let mut f = Fixture::new(true, 0); f.claim.accounts[GUARDIAN].signer = false;
    let before = f.clone(); let data = f.data(0);
    f.with_accounts(|a| {
        let _source = a[KIF].try_borrow_mut_lamports().unwrap();
        assert_eq!(process_instruction(&system_program::ID, a, &data), Err(ProgramError::Custom(6999)));
    });
    assert_eq!(f, before);
}

#[test]
fn rent_failure_propagates_unchanged_before_execution_or_account_access() {
    for error in [ProgramError::UnsupportedSysvar, ProgramError::Custom(6007), ProgramError::InvalidAccountData] {
        let mut f = Fixture::new(false, 0); let before = f.clone(); let data = f.data(100); let calls = RefCell::new(0);
        f.with_accounts(|a| {
            let held: Vec<_> = a.iter().map(|x| x.try_borrow_mut_data().unwrap()).collect();
            assert_eq!(process_instruction_with_host_callbacks(&PROGRAM, a, &data,
                || { *calls.borrow_mut() += 1; Err(error.clone()) }, |_,_,_| panic!("invoked"), |_| panic!("event")), Err(error.clone()));
            drop(held);
        });
        assert_eq!(*calls.borrow(), 1); assert_eq!(f, before);
    }
}

#[test]
fn one_successful_dispatch_invokes_once_then_emits_exactly_one_factual_event() {
    let mut f = Fixture::new(true, 17); let before = f.clone(); let data = f.data(100);
    assert_ne!(f.claim.config().guardian_registry_revision, f.claim.reward().registry_revision);
    assert_eq!(f.claim.reward().last_active_period, None);
    let (result, trace) = f.transaction(&data, Mode::Transfer); assert_eq!(result, Ok(()));
    assert_eq!(trace.order, ["rent", "invoke", "event"]);
    assert_eq!(trace.events, [KifClaimed { guardian_reward: before.claim.accounts[REWARD].key,
        guardian: before.claim.accounts[GUARDIAN].key, amount_lamports: 100 }]);
    let mut expected = before;
    expected.claim.update_config(|c| { c.kif_claim_liability_lamports -= 100; c.cumulative_kif_claimed_lamports += 100; });
    expected.claim.update_reward(|r| { r.claimable_lamports -= 100; r.cumulative_claimed += 100; });
    expected.claim.accounts[KIF].lamports -= 100; expected.claim.accounts[GUARDIAN].lamports += 100;
    expected.claim.audit.paid += 100; assert_eq!(f, expected); f.claim.validate_audit().unwrap();
}

#[test]
fn partial_full_replay_and_later_credit_preserve_original_audit_and_event_counts() {
    let mut f = Fixture::new(true, 0); let first = f.data(100);
    f.transaction(&first, Mode::Transfer).0.unwrap();
    f.claim.credit_snapshot(100).unwrap();
    let before = f.clone(); let (result, trace) = f.transaction(&first, Mode::Transfer);
    assert_eq!(result, Err(ProgramError::Custom(6004))); assert_eq!(trace.order, ["rent"]);
    assert!(trace.events.is_empty()); assert_eq!(f, before);
    let full = f.data(300); let (result, trace) = f.transaction(&full, Mode::Transfer); result.unwrap();
    assert_eq!(trace.events.len(), 1); assert_eq!(trace.events[0].amount_lamports, 300);
    assert_eq!(f.claim.reward().claimable_lamports, 0); assert_eq!(f.claim.audit.paid, 400);
    assert_eq!(f.claim.audit.credited, 100); f.claim.validate_audit().unwrap();
}

#[test]
fn execution_precondition_errors_emit_nothing_and_preserve_all_accounts() {
    for kind in 0..5 {
        let mut f = Fixture::new(false, 0); let mut data = f.data(100);
        let expected = match kind {
            0 => { data = f.data(0); 6003 }
            1 => { data = f.data(301); 6005 }
            2 => { f.claim.accounts[GUARDIAN].signer = false; 6002 }
            3 => { f.claim.accounts[KIF].lamports -= 1; 6006 }
            _ => { f.claim.accounts[GUARDIAN].lamports = u64::MAX; 6072 }
        };
        let before = f.clone(); let (result, trace) = f.raw(&data, Mode::Transfer);
        assert_eq!(result, Err(ProgramError::Custom(expected))); assert_eq!(trace.order, ["rent"]);
        assert!(trace.events.is_empty()); assert_eq!(f, before);
    }
}

#[test]
fn all_account_reorderings_except_identity_fail_authentication_without_invocation_or_event() {
    fn permutations(prefix: &mut Vec<usize>, remaining: &mut Vec<usize>, out: &mut Vec<Vec<usize>>) {
        if remaining.is_empty() { out.push(prefix.clone()); return; }
        for i in 0..remaining.len() {
            let value = remaining.remove(i); prefix.push(value); permutations(prefix, remaining, out);
            prefix.pop(); remaining.insert(i, value);
        }
    }
    let mut orders = Vec::new(); permutations(&mut Vec::new(), &mut vec![0,1,2,3,4], &mut orders);
    assert_eq!(orders.len(), 120);
    for order in orders.into_iter().filter(|order| order != &[0,1,2,3,4]) {
        let mut f = Fixture::new(false, 0); let before = f.clone(); let data = f.data(100);
        f.with_accounts(|a| {
            let reordered: Vec<_> = order.iter().map(|i| a[*i].clone()).collect();
            assert!(process_instruction_with_host_callbacks(&PROGRAM, &reordered, &data, || Ok(Rent::default()),
                |_,_,_| panic!("invoked reordered accounts"), |_| panic!("event")).is_err());
        });
        assert_eq!(f, before);
    }
}

#[test]
fn supplied_runtime_id_is_used_instead_of_a_static_or_instruction_selected_identity() {
    for (program, code) in [(system_program::ID, 6018), (key(216), 6000)] {
        let mut f = Fixture::new(false, 0); f.claim.program_id = program;
        let before = f.clone(); let data = f.data(100);
        let (result, trace) = f.raw(&data, Mode::Transfer);
        assert_eq!(result, Err(ProgramError::Custom(code))); assert_eq!(trace.order, ["rent"]);
        assert!(trace.events.is_empty()); assert_eq!(f, before);
    }
}

#[test]
fn cpi_and_postcheck_errors_propagate_without_success_events_and_roll_back_model() {
    for (mode, expected) in [
        (Mode::InvocationError, ProgramError::Custom(987_654)),
        (Mode::NoPayment, ProgramError::Custom(6008)),
        (Mode::WrongAmount, ProgramError::Custom(6008)),
        (Mode::StateTamper, ProgramError::Custom(6007)),
        (Mode::MalformedPost, ProgramError::Custom(6012)),
    ] {
        let mut f = Fixture::new(true, 17); let before = f.clone(); let data = f.data(100);
        let (result, trace) = f.transaction(&data, mode); assert_eq!(result, Err(expected));
        assert_eq!(trace.order, ["rent", "invoke"]); assert!(trace.events.is_empty()); assert_eq!(f, before);
        f.claim.validate_audit().unwrap();
        let (result, trace) = f.transaction(&data, Mode::Transfer); result.unwrap();
        assert_eq!(trace.events.len(), 1); assert_eq!(f.claim.audit.paid, 100);
    }
}

#[test]
fn dispatch_does_not_catch_failed_execution_or_synthesize_rollback() {
    let mut f = Fixture::new(false, 0); let before = f.clone(); let data = f.data(100);
    let (result, trace) = f.raw(&data, Mode::NoPayment);
    assert_eq!(result, Err(ProgramError::Custom(6008))); assert!(trace.events.is_empty());
    assert_ne!(f.claim.accounts[CONFIG].data, before.claim.accounts[CONFIG].data);
    assert_eq!(f.claim.accounts[KIF].lamports, before.claim.accounts[KIF].lamports);
    assert_eq!(f.claim.audit, before.claim.audit); assert!(f.claim.validate_audit().is_err());
    // The eventual transaction boundary must discard these partial raw effects.
}

#[test]
fn emitted_instruction_success_is_not_a_receipt_for_a_later_failed_transaction() {
    let mut original = Fixture::new(false, 0); let before = original.clone(); let data = original.data(100);
    let mut transaction_clone = original.clone();
    let (result, trace) = transaction_clone.raw(&data, Mode::Transfer); result.unwrap();
    assert_eq!(trace.events.len(), 1);
    // Model a later unrelated instruction failure: retain the observable trace,
    // discard staged effects, and require transaction success before indexing it.
    drop(transaction_clone);
    assert_eq!(original, before);
    let (result, retry_trace) = original.transaction(&data, Mode::Transfer); result.unwrap();
    assert_eq!(retry_trace.events.len(), 1); assert_eq!(original.claim.audit.paid, 100);
}

#[test]
fn all_74_state_errors_have_unique_stable_literal_codes_independent_of_enum_layout() {
    use Piv1Error::*;
    // Published ABI golden values. Changes to these numbers require an explicit
    // ABI revision; appending/reordering a Rust enum must not renumber them.
    let expected = [
        (InvalidAccountOwner, 6000), (AccountNotWritable, 6001),
        (MissingGuardianSignature, 6002), (ZeroKifClaim, 6003),
        (StaleKifClaim, 6004), (KifClaimExceeded, 6005),
        (KifClaimBackingDeficit, 6006), (KifClaimStateChanged, 6007),
        (KifClaimObservationMismatch, 6008), (InvalidClockAccount, 6009),
        (ExecutableAccount, 6010), (InvalidAccountSize, 6011),
        (InvalidAccountDiscriminator, 6012), (InvalidAccountData, 6013),
        (StateEnvelopeEncodingFailed, 6014), (StateEnvelopeChanged, 6015),
        (InvalidAccountPda, 6016), (AccountAlias, 6017),
        (InvalidProgramIdentity, 6018), (AccountBorrowFailed, 6019),
        (AccountRentDeficit, 6020), (InvalidRent, 6021),
        (InvalidTokenCustody, 6022), (UnsupportedTokenNativeExcess, 6023),
        (InvalidVersion, 6024), (InvalidInitialization, 6025),
        (InvalidBootstrapState, 6026), (ZeroPrincipalDeposit, 6027),
        (PrincipalDepositExceedsQueue, 6028), (UnsupportedPrincipalDepositFee, 6029),
        (InvalidPrincipalDepositPool, 6030), (PrincipalDepositObservationMismatch, 6031),
        (PrincipalDepositMinimumNotMet, 6032), (PrincipalDepositHistoricalValueLoss, 6033),
        (InvalidLifecycle, 6034), (PausedOperation, 6035),
        (InvalidTimestamp, 6036), (TimestampRegression, 6037),
        (PreparationIntervalNotElapsed, 6038), (InsufficientAttemptCooldownActive, 6039),
        (InvalidInsufficientAttempt, 6040), (SequenceMismatch, 6041),
        (LegIndexMismatch, 6042), (ZeroTarget, 6043), (ZeroInput, 6044),
        (ZeroContribution, 6045), (InvalidCustodyObservation, 6046),
        (CustodyBalanceDecreased, 6047), (ContributionObservationMismatch, 6048),
        (PendingCustodyDeficit, 6049), (EconomicCustodyDeficit, 6050),
        (TargetExceeded, 6051), (NonMaximumSafeLegFill, 6052),
        (TechnicalFloorNotMet, 6053), (UsefulLegBoundExceeded, 6054),
        (Replay, 6055), (AlreadyFinalized, 6056), (TargetNotAssigned, 6057),
        (CountMismatch, 6058), (CumulativeReconciliationMismatch, 6059),
        (EscrowReconciliationMismatch, 6060), (ObligationExceeded, 6061),
        (OutstandingLiability, 6062), (SettlementReplay, 6063),
        (HighWaterMarkDecrease, 6064), (InvalidGuardianBitmap, 6065),
        (InvalidGuardianCount, 6066), (InvalidGuardianSet, 6067),
        (InvalidAddress, 6068), (InvalidSlippage, 6069), (InvalidSplit, 6070),
        (InvalidTimingConfiguration, 6071), (ArithmeticOverflow, 6072),
        (RecoveryRequired, 6073),
    ];
    assert_eq!(expected.len(), 74);
    let mut unique = BTreeSet::new();
    for (error, code) in expected {
        assert_eq!(piv1_error_code(error), code);
        assert_eq!(execution_program_error(KifClaimExecutionError::State(error)), ProgramError::Custom(code));
        assert!(unique.insert(code));
    }
    assert_eq!(HOST_RUNTIME_UNAVAILABLE_CODE, 6999); assert!(unique.insert(HOST_RUNTIME_UNAVAILABLE_CODE));
    assert_eq!(execution_program_error(KifClaimExecutionError::HostRuntimeUnavailable), ProgramError::Custom(6999));
}

#[test]
fn invocation_program_errors_are_preserved_even_when_custom_numbers_overlap_local_range() {
    for error in [ProgramError::InsufficientFunds, ProgramError::AccountBorrowFailed,
        ProgramError::Custom(6000), ProgramError::Custom(6999), ProgramError::Custom(u32::MAX)]
    {
        assert_eq!(execution_program_error(KifClaimExecutionError::Invocation(error.clone())), error);
    }
}

#[test]
fn dispatch_preserves_native_borrow_preflight_and_maps_failure_without_events() {
    for index in [KIF, GUARDIAN] {
        for kind in 0..4 {
            let mut f = Fixture::new(false, 0); let before = f.clone(); let data = f.data(100);
            f.with_accounts(|a| {
                let data_read = (kind == 0).then(|| a[index].try_borrow_data().unwrap());
                let data_write = (kind == 1).then(|| a[index].try_borrow_mut_data().unwrap());
                let lamports_read = (kind == 2).then(|| a[index].try_borrow_lamports().unwrap());
                let lamports_write = (kind == 3).then(|| a[index].try_borrow_mut_lamports().unwrap());
                assert_eq!(process_instruction_with_host_callbacks(&PROGRAM, a, &data, || Ok(Rent::default()),
                    |_,_,_| panic!("invoked"), |_| panic!("event")), Err(ProgramError::Custom(6019)));
                drop((data_read, data_write, lamports_read, lamports_write));
            });
            assert_eq!(f, before);
        }
    }
}
