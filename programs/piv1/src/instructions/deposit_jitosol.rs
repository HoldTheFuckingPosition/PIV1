//! Exact versioned legacy Token owner-signed JitoSOL intake into pending custody.

use anchor_lang::solana_program::program_error::ProgramError;
use super::deposit_sol::{decode_amount, encode_amount, DEPOSIT_DATA_LEN};

pub const DEPOSIT_JITOSOL_SELECTOR: [u8; 8] = *b"PIV1DJ01";
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct DepositJitoSolRequest { pub amount_units: u64 }
impl DepositJitoSolRequest {
    pub fn decode(data: &[u8]) -> Result<Self, ProgramError> {
        Ok(Self { amount_units: decode_amount(data, &DEPOSIT_JITOSOL_SELECTOR)? })
    }
    pub fn encode(self) -> [u8; DEPOSIT_DATA_LEN] { encode_amount(&DEPOSIT_JITOSOL_SELECTOR, self.amount_units) }
}

instruction_marker!(pub DepositJitoSol);
