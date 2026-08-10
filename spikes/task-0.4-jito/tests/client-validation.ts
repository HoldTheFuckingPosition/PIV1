import assert from 'node:assert/strict';
import { PublicKey } from '@solana/web3.js';

import {
  AUTHORITY_SEED,
  CONFIG_SEED,
  JITOSOL_MINT,
  JITO_STAKE_POOL,
  PROBE_PROGRAM,
  ROUND_SEED,
  SOL_ESCROW_SEED,
  SOL_VAULT_SEED,
  WITHDRAWAL_STAKE_SEED,
} from '../scripts/constants.js';

function u64(value: bigint) {
  const bytes = Buffer.alloc(8);
  bytes.writeBigUInt64LE(value);
  return bytes;
}

const [config] = PublicKey.findProgramAddressSync([CONFIG_SEED], PROBE_PROGRAM);
const [authority] = PublicKey.findProgramAddressSync([AUTHORITY_SEED], PROBE_PROGRAM);
const [solVault] = PublicKey.findProgramAddressSync([SOL_VAULT_SEED], PROBE_PROGRAM);
const [solEscrow] = PublicKey.findProgramAddressSync([SOL_ESCROW_SEED], PROBE_PROGRAM);
const [round0] = PublicKey.findProgramAddressSync([ROUND_SEED, u64(0n)], PROBE_PROGRAM);
const [round1] = PublicKey.findProgramAddressSync([ROUND_SEED, u64(1n)], PROBE_PROGRAM);
const [stake0] = PublicKey.findProgramAddressSync([WITHDRAWAL_STAKE_SEED, u64(0n)], PROBE_PROGRAM);
const [stake1] = PublicKey.findProgramAddressSync([WITHDRAWAL_STAKE_SEED, u64(1n)], PROBE_PROGRAM);

assert.equal(new Set([
  config.toBase58(), authority.toBase58(), solVault.toBase58(), solEscrow.toBase58(),
  round0.toBase58(), round1.toBase58(), stake0.toBase58(), stake1.toBase58(),
]).size, 8, 'Every custody/state PDA must be distinct');
assert.notEqual(round0.toBase58(), round1.toBase58(), 'Rounds must not reuse state');
assert.notEqual(stake0.toBase58(), stake1.toBase58(), 'Rounds must not reuse stake PDAs');
assert.notEqual(solEscrow.toBase58(), authority.toBase58(), 'Final SOL has one fixed escrow destination');

const wrongPool = PublicKey.unique();
const wrongMint = PublicKey.unique();
assert(!wrongPool.equals(JITO_STAKE_POOL), 'Wrong-pool fixture must differ');
assert(!wrongMint.equals(JITOSOL_MINT), 'Wrong-mint fixture must differ');

process.stdout.write(JSON.stringify({
  result: 'ok',
  assertions: 6,
  pda: {
    config: config.toBase58(),
    authority: authority.toBase58(),
    solVault: solVault.toBase58(),
    solEscrow: solEscrow.toBase58(),
    round0: round0.toBase58(),
    withdrawalStake0: stake0.toBase58(),
  },
}, null, 2));
process.stdout.write('\n');
