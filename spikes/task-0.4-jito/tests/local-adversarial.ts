import fs from 'node:fs';
import path from 'node:path';

import { AnchorProvider, BN, Idl, Program, Wallet } from '@coral-xyz/anchor';
import { getAssociatedTokenAddressSync } from '@solana/spl-token';
import { getStakePoolAccount } from '@solana/spl-stake-pool';
import {
  Connection,
  Keypair,
  PublicKey,
  StakeProgram,
  SystemProgram,
  SYSVAR_CLOCK_PUBKEY,
  SYSVAR_STAKE_HISTORY_PUBKEY,
} from '@solana/web3.js';

import {
  AUTHORITY_SEED,
  CONFIG_SEED,
  JITOSOL_MINT,
  JITO_STAKE_POOL,
  LEGACY_TOKEN_PROGRAM,
  PROBE_PROGRAM,
  ROUND_SEED,
  SOL_ESCROW_SEED,
  SOL_VAULT_SEED,
  STAKE_POOL_PROGRAM,
  STAKE_PROGRAM,
  WITHDRAWAL_STAKE_SEED,
} from '../scripts/constants.js';

const STATE_DIR = '/home/jerem/.local/share/piv1/task-0.4-jito';
const FEE_PAYER_PATH = path.join(STATE_DIR, 'testnet-fee-payer.json');
const OUTPUT_PATH = path.join(STATE_DIR, 'local-adversarial.json');
const STALE_OUTPUT_PATH = path.join(STATE_DIR, 'local-stale-pool.json');
const IDL_PATH = path.resolve('target/idl/jito_cpi_probe.json');
const VALIDATOR_STAKE = new PublicKey('GTpapQCpq64AhLskXapChCApM7XiWAZ2GUjrTfbXRC4D');

type ResultRecord = { name: string; rejected: boolean; error: unknown; logs: string[] };

function readKeypair(filename: string): Keypair {
  return Keypair.fromSecretKey(Uint8Array.from(JSON.parse(fs.readFileSync(filename, 'utf8')) as number[]));
}

function u64(value: bigint): Buffer {
  const bytes = Buffer.alloc(8);
  bytes.writeBigUInt64LE(value);
  return bytes;
}

function pda(seed: Buffer, round?: bigint): PublicKey {
  return PublicKey.findProgramAddressSync(round === undefined ? [seed] : [seed, u64(round)], PROBE_PROGRAM)[0];
}

async function main() {
  const connection = new Connection('http://127.0.0.1:8899', 'confirmed');
  const payer = readKeypair(FEE_PAYER_PATH);
  const provider = new AnchorProvider(connection, new Wallet(payer), {
    commitment: 'confirmed',
    preflightCommitment: 'confirmed',
  });
  const program = new Program(JSON.parse(fs.readFileSync(IDL_PATH, 'utf8')) as Idl, provider);
  const pool = (await getStakePoolAccount(connection, JITO_STAKE_POOL)).account.data;
  const [withdrawAuthority] = PublicKey.findProgramAddressSync(
    [JITO_STAKE_POOL.toBuffer(), Buffer.from('withdraw')],
    STAKE_POOL_PROGRAM,
  );
  const config = pda(CONFIG_SEED);
  const authority = pda(AUTHORITY_SEED);
  const solVault = pda(SOL_VAULT_SEED);
  const solEscrow = pda(SOL_ESCROW_SEED);
  const tokenVault = getAssociatedTokenAddressSync(JITOSOL_MINT, authority, true);
  const round1 = pda(ROUND_SEED, 1n);
  const stake1 = pda(WITHDRAWAL_STAKE_SEED, 1n);
  const round2 = pda(ROUND_SEED, 2n);
  const stake2 = pda(WITHDRAWAL_STAKE_SEED, 2n);
  const results: ResultRecord[] = [];

  async function rejected(name: string, builder: { transaction(): Promise<any> }) {
    try {
      const transaction = await builder.transaction();
      transaction.feePayer = payer.publicKey;
      transaction.recentBlockhash = (await connection.getLatestBlockhash('confirmed')).blockhash;
      transaction.sign(payer);
      const simulation = await connection.simulateTransaction(transaction, undefined, true);
      results.push({
        name,
        rejected: simulation.value.err !== null,
        error: simulation.value.err,
        logs: simulation.value.logs ?? [],
      });
    } catch (error: unknown) {
      results.push({ name, rejected: true, error: String(error), logs: [] });
    }
  }

  const depositAccounts = {
    config,
    caller: payer.publicKey,
    authority,
    solVault,
    stakePool: JITO_STAKE_POOL,
    validatorList: pool.validatorList,
    reserveStake: pool.reserveStake,
    tokenVault,
    managerFeeAccount: pool.managerFeeAccount,
    poolMint: JITOSOL_MINT,
    stakePoolWithdrawAuthority: withdrawAuthority,
    stakePoolProgram: STAKE_POOL_PROGRAM,
    tokenProgram: LEGACY_TOKEN_PROGRAM,
    systemProgram: SystemProgram.programId,
  };
  const withdrawalAccounts = {
    config,
    round: round2,
    caller: payer.publicKey,
    authority,
    solVault,
    withdrawalStake: stake2,
    stakePool: JITO_STAKE_POOL,
    validatorList: pool.validatorList,
    reserveStake: pool.reserveStake,
    validatorStake: VALIDATOR_STAKE,
    tokenVault,
    managerFeeAccount: pool.managerFeeAccount,
    poolMint: JITOSOL_MINT,
    stakePoolWithdrawAuthority: withdrawAuthority,
    clock: SYSVAR_CLOCK_PUBKEY,
    stakePoolProgram: STAKE_POOL_PROGRAM,
    tokenProgram: LEGACY_TOKEN_PROGRAM,
    stakeProgram: STAKE_PROGRAM,
    systemProgram: SystemProgram.programId,
  };

  if (process.argv[2] === 'stale-only') {
    await rejected('stale-pool-state', program.methods.depositSol(new BN(1_000_000), new BN(0))
      .accountsStrict(depositAccounts));
    const epoch = (await connection.getEpochInfo('confirmed')).epoch;
    const output = {
      epoch,
      poolLastUpdateEpoch: Number(pool.lastUpdateEpoch.toString()),
      allRejected: results.every((result) => result.rejected),
      results,
    };
    fs.writeFileSync(STALE_OUTPUT_PATH, `${JSON.stringify(output, null, 2)}\n`, { mode: 0o600 });
    process.stdout.write(`${JSON.stringify(output, null, 2)}\n`);
    if (!output.allRejected || output.poolLastUpdateEpoch >= epoch) process.exitCode = 1;
    return;
  }

  await rejected('amount-zero', program.methods.depositSol(new BN(0), new BN(0)).accountsStrict(depositAccounts));
  await rejected('slippage-minimum-too-high', program.methods.depositSol(
    new BN(1_000_000),
    new BN('18446744073709551615'),
  ).accountsStrict(depositAccounts));
  await rejected('wrong-pool', program.methods.depositSol(new BN(1), new BN(0)).accountsStrict({
    ...depositAccounts,
    stakePool: pool.validatorList,
  }));
  await rejected('wrong-mint', program.methods.depositSol(new BN(1), new BN(0)).accountsStrict({
    ...depositAccounts,
    poolMint: pool.managerFeeAccount,
  }));
  await rejected('wrong-token-authority', program.methods.contributeJitosol(new BN(1)).accountsStrict({
    config,
    caller: payer.publicKey,
    authority,
    sourceTokenAccount: pool.managerFeeAccount,
    tokenVault,
    poolMint: JITOSOL_MINT,
    tokenProgram: LEGACY_TOKEN_PROGRAM,
  }));
  await rejected('below-technical-withdrawal-minimum', program.methods.initiateWithdrawal(
    new BN(2), new BN(1), new BN(0), true,
  ).accountsStrict(withdrawalAccounts));
  await rejected('wrong-validator-stake', program.methods.initiateWithdrawal(
    new BN(2), new BN(1), new BN(0), true,
  ).accountsStrict({ ...withdrawalAccounts, validatorStake: pool.reserveStake }));
  await rejected('wrong-withdrawal-stake-pda', program.methods.initiateWithdrawal(
    new BN(2), new BN(1), new BN(0), true,
  ).accountsStrict({ ...withdrawalAccounts, withdrawalStake: stake1 }));
  await rejected('round-and-stake-reuse', program.methods.initiateWithdrawal(
    new BN(1), new BN(1), new BN(0), true,
  ).accountsStrict({ ...withdrawalAccounts, round: round1, withdrawalStake: stake1 }));
  await rejected('arbitrary-final-sol-destination', program.methods.finalizeWithdrawal(new BN(1)).accountsStrict({
    config,
    round: round1,
    caller: payer.publicKey,
    authority,
    withdrawalStake: stake1,
    solEscrow: payer.publicKey,
    clock: SYSVAR_CLOCK_PUBKEY,
    stakeHistory: SYSVAR_STAKE_HISTORY_PUBKEY,
    stakeProgram: StakeProgram.programId,
  }));
  await rejected('finalize-before-inactive-epoch', program.methods.finalizeWithdrawal(new BN(1)).accountsStrict({
    config,
    round: round1,
    caller: payer.publicKey,
    authority,
    withdrawalStake: stake1,
    solEscrow,
    clock: SYSVAR_CLOCK_PUBKEY,
    stakeHistory: SYSVAR_STAKE_HISTORY_PUBKEY,
    stakeProgram: StakeProgram.programId,
  }));

  const failed = results.filter((result) => !result.rejected);
  const output = {
    epoch: (await connection.getEpochInfo('confirmed')).epoch,
    assertions: results.length,
    allRejected: failed.length === 0,
    results,
  };
  fs.writeFileSync(OUTPUT_PATH, `${JSON.stringify(output, null, 2)}\n`, { mode: 0o600 });
  process.stdout.write(`${JSON.stringify({
    epoch: output.epoch,
    assertions: output.assertions,
    allRejected: output.allRejected,
    summary: results.map(({ name, rejected: wasRejected, logs }) => ({
      name,
      rejected: wasRejected,
      finalLog: logs.at(-1) ?? null,
    })),
  }, null, 2)}\n`);
  if (failed.length !== 0) process.exitCode = 1;
}

main().catch((error: unknown) => {
  process.stderr.write(`${error instanceof Error ? error.stack : String(error)}\n`);
  process.exitCode = 1;
});
