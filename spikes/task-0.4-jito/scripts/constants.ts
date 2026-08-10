import { PublicKey, StakeProgram } from '@solana/web3.js';
import { TOKEN_PROGRAM_ID } from '@solana/spl-token';

export const JITO_STAKE_POOL = new PublicKey('Jito4APyf642JPZPx3hGc6WWJ8zPKtRbRs4P815Awbb');
export const JITOSOL_MINT = new PublicKey('J1toso1uCk3RLmjorhTtrVwY9HJ7X8V9yYac6Y7kGCPn');
export const STAKE_POOL_PROGRAM = new PublicKey('SPoo1Ku8WFXoNDMHPsrGSTSG1Y47rzgn41SLUNakuHy');
export const LEGACY_TOKEN_PROGRAM = TOKEN_PROGRAM_ID;
export const STAKE_PROGRAM = StakeProgram.programId;
export const PROBE_PROGRAM = new PublicKey('BbHNk57mVmZmfH1HfiyPaapwJ5FVPe3c7MsidbGVewG6');

export const CONFIG_SEED = Buffer.from('config');
export const AUTHORITY_SEED = Buffer.from('authority');
export const SOL_VAULT_SEED = Buffer.from('sol-vault');
export const SOL_ESCROW_SEED = Buffer.from('sol-escrow');
export const ROUND_SEED = Buffer.from('round');
export const WITHDRAWAL_STAKE_SEED = Buffer.from('withdrawal-stake');
