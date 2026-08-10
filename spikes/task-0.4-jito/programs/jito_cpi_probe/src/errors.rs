use anchor_lang::prelude::*;

#[error_code]
pub enum ProbeError {
    #[msg("The amount must be greater than zero")]
    AmountZero,
    #[msg("The supplied account is not the official Jito stake pool")]
    WrongStakePool,
    #[msg("The supplied stake-pool program is not the official SPL stake-pool program")]
    WrongStakePoolProgram,
    #[msg("The supplied pool mint is not JitoSOL")]
    WrongPoolMint,
    #[msg("The supplied token program is not the legacy SPL Token program")]
    WrongTokenProgram,
    #[msg("The supplied stake program is invalid")]
    WrongStakeProgram,
    #[msg("The supplied system program is invalid")]
    WrongSystemProgram,
    #[msg("The supplied account owner is invalid")]
    WrongAccountOwner,
    #[msg("The stake-pool account could not be decoded")]
    InvalidStakePoolState,
    #[msg("The stake-pool account relationships do not match its decoded state")]
    StakePoolBindingMismatch,
    #[msg("The JitoSOL token account could not be decoded")]
    InvalidTokenAccount,
    #[msg("The token-account mint does not match JitoSOL")]
    TokenMintMismatch,
    #[msg("The token-account authority is invalid")]
    TokenAuthorityMismatch,
    #[msg("The mint account could not be decoded")]
    InvalidMint,
    #[msg("The pool state is stale for the current epoch")]
    StalePool,
    #[msg("Checked arithmetic failed")]
    Arithmetic,
    #[msg("The observed token or lamport balance moved in the wrong direction")]
    BalanceDelta,
    #[msg("The CPI output was below the caller's conservative minimum")]
    SlippageExceeded,
    #[msg("The requested round is not the next unused round")]
    RoundReuse,
    #[msg("The withdrawal is below the stake account's technical minimum")]
    WithdrawalBelowTechnicalMinimum,
    #[msg("The withdrawal stake PDA is already in use")]
    StakePdaReuse,
    #[msg("The supplied validator stake account is not a deterministic account for this pool")]
    WrongValidatorStake,
    #[msg("The source stake account could not be decoded")]
    InvalidStakeState,
    #[msg("The withdrawal stake account is not controlled by the probe authority")]
    WrongStakeAuthority,
    #[msg("The withdrawal stake account has not been deactivated")]
    StakeNotDeactivated,
    #[msg("The round does not bind the supplied withdrawal stake account")]
    RoundStakeMismatch,
    #[msg("The round is not in the required lifecycle state")]
    WrongRoundStatus,
    #[msg("The supplied native-SOL destination is not the fixed escrow PDA")]
    WrongEscrow,
}
