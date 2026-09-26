#![forbid(unsafe_code)]
#![allow(unexpected_cfgs)]
//! Validation-only entrypoint for the unchanged full normalized initializer.
//! This binary is not PIV1's production artifact or initializer ABI.

use anchor_lang::{prelude::{AccountInfo, Pubkey},
    solana_program::{entrypoint::ProgramResult, program_error::ProgramError}};
use piv1::{genesis_preflight::{GenesisPreflightRoles, GenesisProtocolRoles},
    genesis_allocation::{GenesisAllocationError, GenesisAllocationRoles},
    genesis_initialization::{GenesisInitializationError, GenesisInitializationRoles, RecipientCheckedGenesisRoles},
    genesis_recipients::GenesisRecipientSelection, instructions::initialize::GenesisModelParameters,
    squads_execution::SquadsBootstrapRoles};

pub const HOST_UNAVAILABLE: u32 = 0x2310;
pub const INITIALIZATION_REJECTED: u32 = 0x2311;

/// Fixed complete 35/34-account topology; no caller-selected role indices.
pub fn profile(data: &[u8], account_count: usize) -> Result<RecipientCheckedGenesisRoles, ProgramError> {
    let model = GenesisModelParameters::decode(data).map_err(|_| ProgramError::InvalidInstructionData)?;
    let shared = model.protocol.manager_fee_account == model.protocol.referrer_token_account;
    let payer = if shared { 29 } else { 30 };
    let recipient = payer + 3;
    if account_count != recipient + 2 { return Err(ProgramError::InvalidArgument); }
    Ok(RecipientCheckedGenesisRoles {
        initialization: GenesisInitializationRoles {
            allocation: GenesisAllocationRoles {
                preflight: GenesisPreflightRoles {
                    bootstrap: SquadsBootstrapRoles { program: 0, program_data: 1, multisig: 2,
                        proposal: 3, transaction: 4, vault: 5, instructions: 6, config: 7 },
                    protocol: GenesisProtocolRoles { program: 23, pool: 24, validator_list: 25,
                        reserve: 26, mint: 27, manager_fee: 28, referrer: if shared { 28 } else { 29 } },
                    targets: core::array::from_fn(|index| 7 + index),
                },
                payer, system_program: payer + 1,
            },
            token_program: payer + 2,
        },
        recipients: GenesisRecipientSelection { htfp_recipient: recipient, team_owner_recipient: recipient + 1,
            htfp_vault_index: 0, team_owner_vault_index: 255 },
    })
}

/// Preserve actual CPI errors; validation errors share a probe-only category.
pub fn map_error(error: GenesisInitializationError) -> ProgramError {
    match error {
        GenesisInitializationError::Invocation(error)
        | GenesisInitializationError::Allocation(GenesisAllocationError::Invocation(error)) => error,
        _ => ProgramError::Custom(INITIALIZATION_REJECTED),
    }
}

pub fn process_instruction(program: &Pubkey, accounts: &[AccountInfo<'_>], data: &[u8]) -> ProgramResult {
    // No ordinary-host syscall stub may turn this probe into a successful call.
    #[cfg(not(target_os = "solana"))]
    { let _ = (program, accounts, data); Err(ProgramError::Custom(HOST_UNAVAILABLE)) }
    #[cfg(target_os = "solana")]
    {
        let roles = profile(data, accounts.len())?;
        let _ = piv1::genesis_initialization::initialize_approved_genesis_with_checked_recipients(program, accounts, data, roles)
            .map_err(map_error)?;
        Ok(())
    }
}

#[cfg(target_os = "solana")]
anchor_lang::solana_program::entrypoint!(process_instruction);
