#![forbid(unsafe_code)]
#![allow(unexpected_cfgs)]
//! Validation-only canonical Token InitializeAccount3 entrypoint.
//! The accepted operation delegates to the unchanged pinned Token processor.
//! This is not a deployed Token artifact or a general-purpose Token replacement.

use anchor_lang::{prelude::{AccountInfo, Pubkey},
    solana_program::{entrypoint::ProgramResult, program_error::ProgramError}};

pub const HOST_UNAVAILABLE: u32 = 0x2313;

/// Exact wire boundary needed by the initializer: opcode18 + owner32, two roles.
pub fn validate_instruction(program: &Pubkey, account_count: usize, data: &[u8]) -> ProgramResult {
    if *program != spl_token::ID { return Err(ProgramError::IncorrectProgramId); }
    if data.len() != 33 || data.first() != Some(&18) { return Err(ProgramError::InvalidInstructionData); }
    if account_count != 2 { return Err(ProgramError::InvalidArgument); }
    Ok(())
}

pub fn process_instruction(program: &Pubkey, accounts: &[AccountInfo<'_>], data: &[u8]) -> ProgramResult {
    #[cfg(not(target_os = "solana"))]
    { let _ = (program, accounts, data); Err(ProgramError::Custom(HOST_UNAVAILABLE)) }
    #[cfg(target_os = "solana")]
    {
        validate_instruction(program, accounts.len(), data)?;
        spl_token::processor::Processor::process(program, accounts, data)
    }
}

#[cfg(target_os = "solana")]
anchor_lang::solana_program::entrypoint!(process_instruction);
