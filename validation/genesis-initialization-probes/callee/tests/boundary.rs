use anchor_lang::{prelude::Pubkey, solana_program::program_error::ProgramError};
use piv1::instructions::initialize::{DeclaredGenesisProtocol, GenesisModelParameters};
use piv1_genesis_initialization_probe::{map_error, profile, process_instruction, HOST_UNAVAILABLE, INITIALIZATION_REJECTED};
use piv1::{genesis_allocation::GenesisAllocationError, genesis_initialization::GenesisInitializationError};

fn model(shared: bool) -> [u8; 313] {
    let key = |tag| Pubkey::new_from_array([tag; 32]);
    GenesisModelParameters { vault_index: 7, initially_paused: false,
        protocol: DeclaredGenesisProtocol { stake_pool_program: key(1), stake_pool: key(2),
            validator_list: key(3), reserve_stake: key(4), jitosol_mint: key(5),
            manager_fee_account: key(6), referrer_token_account: key(if shared {6} else {7}) },
        htfp_recipient: key(8), team_owner_recipient: key(9), kif_anchor_timestamp: 0,
        guardian_slot_permutation: [5, 4, 3, 2, 1, 0] }.encode().unwrap()
}

#[test]
fn fixed_topology_matches_literal_full_initializer_account_contract() {
    for (shared, count, recipient, referrer) in [(false,35,33,29), (true,34,32,28)] {
        let roles = profile(&model(shared), count).unwrap();
        let a = roles.initialization.allocation; let b = a.preflight.bootstrap; let p = a.preflight.protocol;
        assert_eq!((a.payer,a.system_program,roles.initialization.token_program),(recipient-3,recipient-2,recipient-1));
        let recipients = roles.recipients;
        assert_eq!([b.program,b.program_data,b.multisig,b.proposal,b.transaction,b.vault,b.instructions,b.config],
            [0,1,2,3,4,5,6,7]);
        assert_eq!(a.preflight.targets, [7,8,9,10,11,12,13,14,15,16,17,18,19,20,21,22]);
        assert_eq!([p.program,p.pool,p.validator_list,p.reserve,p.mint,p.manager_fee,p.referrer],
            [23,24,25,26,27,28,referrer]);
        assert_eq!((recipients.htfp_recipient,recipients.team_owner_recipient,recipients.htfp_vault_index,recipients.team_owner_vault_index),
            (recipient,recipient+1,0,255));
    }
}

#[test]
fn unsupported_counts_malformed_model_and_old_preflight_topology_reject() {
    for shared in [false,true] {
        let bytes = model(shared);
        for count in [0,4,30,31,32,33,36,usize::MAX] {
            assert_eq!(profile(&bytes,count).unwrap_err(),ProgramError::InvalidArgument);
        }
        assert_eq!(profile(&bytes,if shared {35} else {34}).unwrap_err(),ProgramError::InvalidArgument);
        for length in 0..313 { assert_eq!(profile(&bytes[..length],if shared {34} else {35}).unwrap_err(),ProgramError::InvalidInstructionData); }
        for index in [0,8,10,307] {
            let mut invalid=bytes; invalid[index]=255;
            assert_eq!(profile(&invalid,if shared {34} else {35}).unwrap_err(),ProgramError::InvalidInstructionData);
        }
    }
}

#[test]
fn ordinary_host_entrypoint_always_rejects_without_a_runtime_seam() {
    for data in [Vec::new(),model(false).to_vec()] {
        assert_eq!(process_instruction(&Pubkey::new_from_array([217;32]),&[],&data),
            Err(ProgramError::Custom(HOST_UNAVAILABLE)));
    }
}

#[test]
fn system_and_token_invocation_errors_propagate_without_losing_the_original_error() {
    for error in [ProgramError::Custom(22101), ProgramError::InsufficientFunds, ProgramError::InvalidAccountData] {
        assert_eq!(map_error(GenesisInitializationError::Invocation(error.clone())),error);
        assert_eq!(map_error(GenesisInitializationError::Allocation(GenesisAllocationError::Invocation(error.clone()))),error);
    }
    for error in [GenesisInitializationError::InvalidRoles,GenesisInitializationError::ObservationMismatch,
        GenesisInitializationError::Allocation(GenesisAllocationError::InvalidPayer)] {
        assert_eq!(map_error(error),ProgramError::Custom(INITIALIZATION_REJECTED));
    }
}
