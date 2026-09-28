use anchor_lang::{prelude::{AccountInfo, Pubkey}, solana_program::program_error::ProgramError};
use piv1::{
    genesis_allocation::GenesisAllocationError as A,
    genesis_initialization::GenesisInitializationError as I,
    genesis_model::GenesisModelError as M,
    genesis_preflight::GenesisPreflightError as P,
    genesis_recipients::GenesisRecipientError as R,
    instruction_boundary::{process_instruction, process_instruction_with_host_callbacks},
    instruction_errors::{initialization_program_error as mapped, HOST_RUNTIME_UNAVAILABLE_CODE},
    instructions::initialize::*,
    integrations::jito_identity::JitoIdentityError as J,
    squads_execution::SquadsExecutionError as S,
};

fn parameters(shared: bool) -> InitializePiv1Parameters {
    let key = |byte| Pubkey::new_from_array([byte; 32]);
    InitializePiv1Parameters { model: GenesisModelParameters {
        vault_index: 7, initially_paused: true,
        protocol: DeclaredGenesisProtocol { stake_pool_program: key(11), stake_pool: key(12),
            validator_list: key(13), reserve_stake: key(14), jitosol_mint: key(15),
            manager_fee_account: key(16), referrer_token_account: key(if shared {16} else {17}) },
        htfp_recipient: key(18), team_owner_recipient: key(19),
        kif_anchor_timestamp: 0x0102_0304_0506_0708, guardian_slot_permutation: [5, 4, 3, 2, 1, 0],
    }, htfp_vault_index: 23, team_owner_vault_index: 172 }
}

#[test]
fn native_wire_has_independent_literal_fields_and_preserves_legacy_codec() {
    let value = parameters(false);
    let mut literal = b"PIV1IN01".to_vec();
    literal.extend([1, 7, 1]);
    for byte in 11..=19 { literal.extend([byte; 32]); }
    literal.extend([8, 7, 6, 5, 4, 3, 2, 1, 5, 4, 3, 2, 1, 0, 23, 172]);
    assert_eq!(literal.len(), 315);
    assert_eq!(INITIALIZE_PIV1_DATA_SIZE, 315);
    assert_eq!(value.encode().unwrap().as_slice(), literal);
    assert_eq!(InitializePiv1Parameters::decode(&literal), Ok(value));
    let mut legacy = literal[..313].to_vec(); legacy[..8].copy_from_slice(b"PIV1GM01");
    assert_eq!(value.model.encode().unwrap().as_slice(), legacy);
    assert_eq!(GenesisModelParameters::decode(&legacy), Ok(value.model));
    assert!(GenesisModelParameters::decode(&literal).is_err());
    assert!(InitializePiv1Parameters::decode(&legacy).is_err());
    for witness in 0..=255 {
        let value = InitializePiv1Parameters { htfp_vault_index: witness,
            team_owner_vault_index: 255 - witness, ..value };
        assert_eq!(InitializePiv1Parameters::decode(&value.encode().unwrap()), Ok(value));
    }
}

#[test]
fn native_decoder_rejects_noncanonical_lengths_domains_versions_flags_and_permutations() {
    let bytes = parameters(false).encode().unwrap();
    for length in 0..315 { assert!(InitializePiv1Parameters::decode(&bytes[..length]).is_err()); }
    let mut extra = bytes.to_vec(); extra.push(0);
    assert!(InitializePiv1Parameters::decode(&extra).is_err());
    for offset in [0, 7, 8, 10, 307, 312] {
        let mut invalid = bytes; invalid[offset] = 255;
        assert!(InitializePiv1Parameters::decode(&invalid).is_err());
        assert_eq!(process_instruction(&Pubkey::new_from_array([220; 32]), &[], &invalid), Err(ProgramError::InvalidInstructionData));
    }
    let mut duplicate = bytes; duplicate[308] = duplicate[307];
    assert_eq!(InitializePiv1Parameters::decode(&duplicate), Err(GenesisModelFormatError::InvalidSlotPermutation));
    let legacy = parameters(false).model.encode().unwrap();
    assert_eq!(process_instruction(&Pubkey::new_from_array([220; 32]), &[], &legacy), Err(ProgramError::InvalidInstructionData));
}

#[test]
fn exact_native_topology_and_ordinary_and_claim_host_guards_precede_account_access() {
    let key = Pubkey::new_from_array([220; 32]); let owner = Pubkey::new_from_array([221; 32]);
    let mut lamports = 0; let mut data = [];
    let account = AccountInfo::new(&key, false, false, &mut lamports, &mut data, &owner, false, 0);
    let mut accounts = vec![account; 36];
    for shared in [false, true] {
        let count = if shared {34} else {35}; let data = parameters(shared).encode().unwrap();
        for length in 0..=36 {
            let expected = if length < count { ProgramError::NotEnoughAccountKeys }
                else if length > count { ProgramError::InvalidArgument }
                else { ProgramError::Custom(HOST_RUNTIME_UNAVAILABLE_CODE) };
            assert_eq!(process_instruction(&key, &accounts[..length], &data), Err(expected));
        }
        assert_eq!(process_instruction_with_host_callbacks(&key, &accounts[..count], &data,
            || panic!("initializer must not use the claim Rent callback"),
            |_, _, _| panic!("claim callback must not execute initializer CPI"),
            |_| panic!("initializer must not emit a claim event")),
            Err(ProgramError::Custom(HOST_RUNTIME_UNAVAILABLE_CODE)));
    }
    // Data is decoded before count checks and before any account borrowing.
    accounts.clear();
    assert_eq!(process_instruction(&key, &accounts, b"PIV1IN01"), Err(ProgramError::InvalidInstructionData));
}

fn nested(error: S) -> I { I::Allocation(A::Preflight(P::Model(M::Authorization(error)))) }

#[test]
fn runtime_and_both_cpi_errors_survive_all_initializer_wrappers() {
    for error in [ProgramError::Custom(22101), ProgramError::InsufficientFunds,
        ProgramError::UnsupportedSysvar, ProgramError::AccountBorrowFailed] {
        assert_eq!(mapped(I::Invocation(error.clone())), error);
        assert_eq!(mapped(I::Allocation(A::Invocation(error.clone()))), error);
        assert_eq!(mapped(nested(S::Runtime(error.clone()))), error);
        let runtime = P::Model(M::Authorization(S::Runtime(error.clone())));
        assert_eq!(mapped(I::Recipient(R::Preflight(runtime))), error);
    }
    use piv1::errors::Piv1Error as E;
    for error in [I::State(E::InvalidAccountOwner), I::Allocation(A::State(E::InvalidAccountOwner)),
        I::Recipient(R::State(E::InvalidAccountOwner)), I::Allocation(A::Preflight(P::State(E::InvalidAccountOwner))),
        I::Allocation(A::Preflight(P::Model(M::State(E::InvalidAccountOwner)))), nested(S::State(E::InvalidAccountOwner))] {
        assert_eq!(mapped(error), ProgramError::Custom(6000));
    }
    for error in [I::HostRuntimeUnavailable, I::Allocation(A::HostRuntimeUnavailable),
        I::Recipient(R::HostRuntimeUnavailable), nested(S::HostRuntimeUnavailable)] {
        assert_eq!(mapped(error), ProgramError::Custom(6999));
    }
}

#[test]
fn new_validation_error_assignments_are_literal_and_do_not_reuse_state_codes() {
    for (error, code) in [(S::InvalidInvocation, 6100), (S::InvalidInstructionsSysvar, 6101),
        (S::InvalidProposal, 6102), (S::InvalidTransaction, 6103), (S::UnsupportedMessage, 6104),
        (S::MessageMismatch, 6105), (S::TimelockNotReleased, 6106)] {
        assert_eq!(mapped(nested(error)), ProgramError::Custom(code));
    }
    for (error, code) in [(J::InvalidIdentity, 6107), (J::AccountAlias, 6108), (J::InvalidOwner, 6109),
        (J::InvalidExecutable, 6110), (J::BorrowFailed, 6111), (J::UnsupportedProgram, 6112),
        (J::InvalidPool, 6113), (J::InvalidFee, 6114), (J::InvalidList, 6115),
        (J::InvalidReserve, 6116), (J::InvalidMint, 6117), (J::InvalidReceiver, 6118)] {
        assert_eq!(mapped(I::Allocation(A::Preflight(P::Protocol(error)))), ProgramError::Custom(code));
    }
    for (error, code) in [(I::Allocation(A::Preflight(P::InvalidRoles)), 6120),
        (I::Allocation(A::InvalidRoles), 6121), (I::Allocation(A::InvalidPayer), 6122),
        (I::Allocation(A::InsufficientPayerRent), 6123), (I::Allocation(A::ObservationMismatch), 6124),
        (I::Recipient(R::InvalidRoles), 6125), (I::Recipient(R::UnapprovedRecipient), 6126),
        (I::Recipient(R::InvalidRecipientVault), 6127), (I::Recipient(R::UnfundedRecipient), 6128),
        (I::Recipient(R::ObservationMismatch), 6129), (I::InvalidRoles, 6130), (I::ObservationMismatch, 6131)] {
        assert_eq!(mapped(error), ProgramError::Custom(code));
    }
    for error in [GenesisModelFormatError::InvalidLength, GenesisModelFormatError::InvalidSelector,
        GenesisModelFormatError::UnsupportedVersion, GenesisModelFormatError::InvalidBoolean,
        GenesisModelFormatError::InvalidSlotPermutation] {
        assert_eq!(mapped(I::Allocation(A::Preflight(P::Model(M::Format(error))))), ProgramError::InvalidInstructionData);
    }
}
