import fs from 'node:fs';
import { inspect } from 'node:util';
import path from 'node:path';
import process from 'node:process';

import { AnchorProvider, BN, Idl, Program, Wallet } from '@coral-xyz/anchor';
import {
  createAssociatedTokenAccountIdempotentInstruction,
  getAccount,
  getAssociatedTokenAddressSync,
} from '@solana/spl-token';
import { depositSol as buildClientDeposit, getStakePoolAccount } from '@solana/spl-stake-pool';
import {
  clusterApiUrl,
  ComputeBudgetProgram,
  Connection,
  Keypair,
  LAMPORTS_PER_SOL,
  PublicKey,
  sendAndConfirmTransaction,
  StakeProgram,
  SystemProgram,
  SYSVAR_CLOCK_PUBKEY,
  SYSVAR_RENT_PUBKEY,
  SYSVAR_STAKE_HISTORY_PUBKEY,
  Transaction,
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
} from './constants.js';

const STATE_DIR = '/home/jerem/.local/share/piv1/task-0.4-jito';
const FEE_PAYER_PATH = path.join(STATE_DIR, 'testnet-fee-payer.json');
const LOCAL_STATE_PATH = path.join(STATE_DIR, 'local-lifecycle.json');
const TESTNET_STATE_PATH = path.join(STATE_DIR, 'testnet-lifecycle.json');
const IDL_PATH = path.resolve('target/idl/jito_cpi_probe.json');
const TOP_VALIDATOR_STAKE = new PublicKey('GTpapQCpq64AhLskXapChCApM7XiWAZ2GUjrTfbXRC4D');

type NetworkMode = 'local' | 'testnet';
type JsonRecord = Record<string, unknown>;

function readKeypair(filename: string): Keypair {
  const secret = JSON.parse(fs.readFileSync(filename, 'utf8')) as number[];
  return Keypair.fromSecretKey(Uint8Array.from(secret));
}

function u64(value: bigint): Buffer {
  const bytes = Buffer.alloc(8);
  bytes.writeBigUInt64LE(value);
  return bytes;
}

function pda(seed: Buffer, round?: bigint): PublicKey {
  const seeds = round === undefined ? [seed] : [seed, u64(round)];
  return PublicKey.findProgramAddressSync(seeds, PROBE_PROGRAM)[0];
}

function loadState(filename: string): JsonRecord {
  if (!fs.existsSync(filename)) return { transactions: {} };
  return JSON.parse(fs.readFileSync(filename, 'utf8')) as JsonRecord;
}

function saveState(filename: string, state: JsonRecord) {
  fs.mkdirSync(path.dirname(filename), { recursive: true, mode: 0o700 });
  fs.writeFileSync(filename, `${JSON.stringify(state, null, 2)}\n`, { mode: 0o600 });
}

function feeCeil(amount: bigint, numerator: bigint, denominator: bigint): bigint {
  if (denominator === 0n || numerator === 0n) return 0n;
  return (amount * numerator + denominator - 1n) / denominator;
}

function expectedDeposit(pool: Awaited<ReturnType<typeof getStakePoolAccount>>['account']['data'], amount: bigint) {
  const total = BigInt(pool.totalLamports.toString());
  const supply = BigInt(pool.poolTokenSupply.toString());
  const gross = amount * supply / total;
  const fee = feeCeil(
    gross,
    BigInt(pool.solDepositFee.numerator.toString()),
    BigInt(pool.solDepositFee.denominator.toString()),
  );
  return gross - fee;
}

function expectedWithdrawal(pool: Awaited<ReturnType<typeof getStakePoolAccount>>['account']['data'], amount: bigint) {
  const fee = feeCeil(
    amount,
    BigInt(pool.stakeWithdrawalFee.numerator.toString()),
    BigInt(pool.stakeWithdrawalFee.denominator.toString()),
  );
  return (amount - fee) * BigInt(pool.totalLamports.toString()) / BigInt(pool.poolTokenSupply.toString());
}

function minimumWithdrawalInput(
  pool: Awaited<ReturnType<typeof getStakePoolAccount>>['account']['data'],
  targetLamports: bigint,
) {
  let low = 1n;
  let high = BigInt(pool.poolTokenSupply.toString());
  while (low < high) {
    const mid = low + (high - low) / 2n;
    if (expectedWithdrawal(pool, mid) >= targetLamports) high = mid;
    else low = mid + 1n;
  }
  return low;
}

async function transactionEvidence(connection: Connection, signature: string) {
  for (let attempt = 0; attempt < 10; attempt += 1) {
    const transaction = await connection.getTransaction(signature, {
      commitment: 'confirmed',
      maxSupportedTransactionVersion: 0,
    });
    if (transaction) {
      return {
        signature,
        slot: transaction.slot,
        feeLamports: transaction.meta?.fee ?? null,
        computeUnitsConsumed: transaction.meta?.computeUnitsConsumed ?? null,
        error: transaction.meta?.err ?? null,
        logMessages: transaction.meta?.logMessages ?? [],
      };
    }
    await new Promise((resolve) => setTimeout(resolve, 500));
  }
  return { signature, transactionDetailsUnavailable: true };
}

async function record(
  statePath: string,
  state: JsonRecord,
  connection: Connection,
  stage: string,
  signature: string,
  accountChanges?: JsonRecord,
) {
  const transactions = (state.transactions ?? {}) as JsonRecord;
  transactions[stage] = {
    ...(await transactionEvidence(connection, signature)),
    accountChanges,
  };
  state.transactions = transactions;
  saveState(statePath, state);
}

function programFor(connection: Connection, payer: Keypair) {
  const idl = JSON.parse(fs.readFileSync(IDL_PATH, 'utf8')) as Idl;
  const provider = new AnchorProvider(connection, new Wallet(payer), {
    commitment: 'confirmed',
    preflightCommitment: 'confirmed',
  });
  return { provider, program: new Program(idl, provider) };
}

async function ensureLocalFunding(connection: Connection, payer: Keypair) {
  const target = 20 * LAMPORTS_PER_SOL;
  if (await connection.getBalance(payer.publicKey, 'confirmed') >= target / 2) return;
  const signature = await connection.requestAirdrop(payer.publicKey, target);
  const latest = await connection.getLatestBlockhash('confirmed');
  await connection.confirmTransaction({ signature, ...latest }, 'confirmed');
}

async function sendInstructions(
  connection: Connection,
  payer: Keypair,
  instructions: Transaction['instructions'],
  extraSigners: Keypair[] = [],
) {
  const transaction = new Transaction().add(...instructions);
  return sendAndConfirmTransaction(connection, transaction, [payer, ...extraSigners], {
    commitment: 'confirmed',
  });
}

async function localInitialize() {
  const connection = new Connection('http://127.0.0.1:8899', 'confirmed');
  const payer = readKeypair(FEE_PAYER_PATH);
  await ensureLocalFunding(connection, payer);
  const { program } = programFor(connection, payer);
  const state = loadState(LOCAL_STATE_PATH);
  const statePath = LOCAL_STATE_PATH;

  const config = pda(CONFIG_SEED);
  const authority = pda(AUTHORITY_SEED);
  const solVault = pda(SOL_VAULT_SEED);
  const solEscrow = pda(SOL_ESCROW_SEED);
  const tokenVault = getAssociatedTokenAddressSync(JITOSOL_MINT, authority, true);
  const payerToken = getAssociatedTokenAddressSync(JITOSOL_MINT, payer.publicKey);
  const poolAccount = await getStakePoolAccount(connection, JITO_STAKE_POOL);
  const pool = poolAccount.account.data;
  const [withdrawAuthority] = PublicKey.findProgramAddressSync(
    [JITO_STAKE_POOL.toBuffer(), Buffer.from('withdraw')],
    STAKE_POOL_PROGRAM,
  );

  state.network = 'local-clone-of-testnet';
  state.rpcEndpoint = 'http://127.0.0.1:8899';
  state.programId = PROBE_PROGRAM.toBase58();
  state.feePayer = payer.publicKey.toBase58();
  state.publicAccounts = {
    config: config.toBase58(),
    authority: authority.toBase58(),
    solVault: solVault.toBase58(),
    solEscrow: solEscrow.toBase58(),
    tokenVault: tokenVault.toBase58(),
    payerToken: payerToken.toBase58(),
    validatorStake: TOP_VALIDATOR_STAKE.toBase58(),
  };

  const setupBefore = {
    payerLamports: await connection.getBalance(payer.publicKey, 'confirmed'),
    solVaultLamports: await connection.getBalance(solVault, 'confirmed'),
    solEscrowLamports: await connection.getBalance(solEscrow, 'confirmed'),
  };
  const systemAccountRent = await connection.getMinimumBalanceForRentExemption(0, 'confirmed');
  const setupSignature = await sendInstructions(connection, payer, [
    createAssociatedTokenAccountIdempotentInstruction(payer.publicKey, tokenVault, authority, JITOSOL_MINT),
    createAssociatedTokenAccountIdempotentInstruction(payer.publicKey, payerToken, payer.publicKey, JITOSOL_MINT),
    SystemProgram.transfer({ fromPubkey: payer.publicKey, toPubkey: solVault, lamports: 2_500_000_000 }),
    SystemProgram.transfer({ fromPubkey: payer.publicKey, toPubkey: solEscrow, lamports: systemAccountRent }),
  ]);
  await record(statePath, state, connection, 'fund_piv_sol_vault', setupSignature, {
    before: setupBefore,
    systemAccountRent,
    after: {
      payerLamports: await connection.getBalance(payer.publicKey, 'confirmed'),
      solVaultLamports: await connection.getBalance(solVault, 'confirmed'),
      solEscrowLamports: await connection.getBalance(solEscrow, 'confirmed'),
    },
  });

  if (!(await connection.getAccountInfo(config, 'confirmed'))) {
    const signature = await program.methods.initialize().accountsStrict({
      config,
      caller: payer.publicKey,
      authority,
      solVault,
      solEscrow,
      tokenVault,
      stakePool: JITO_STAKE_POOL,
      validatorList: pool.validatorList,
      reserveStake: pool.reserveStake,
      managerFeeAccount: pool.managerFeeAccount,
      poolMint: JITOSOL_MINT,
      stakePoolWithdrawAuthority: withdrawAuthority,
      stakePoolProgram: STAKE_POOL_PROGRAM,
      tokenProgram: LEGACY_TOKEN_PROGRAM,
      systemProgram: SystemProgram.programId,
    }).rpc();
    await record(statePath, state, connection, 'initialize_probe', signature);
  }

  const clientAmount = 1_050_000_000;
  const payerTokenBefore = (await getAccount(connection, payerToken, 'confirmed')).amount;
  const clientDeposit = await buildClientDeposit(
    connection,
    JITO_STAKE_POOL,
    payer.publicKey,
    clientAmount,
    payerToken,
    pool.managerFeeAccount,
  );
  const clientSignature = await sendInstructions(
    connection,
    payer,
    [ComputeBudgetProgram.setComputeUnitLimit({ units: 300_000 }), ...clientDeposit.instructions],
    clientDeposit.signers as Keypair[],
  );
  const payerTokenAfter = (await getAccount(connection, payerToken, 'confirmed')).amount;
  await record(statePath, state, connection, 'baseline_direct_client_deposit', clientSignature, {
    lamportsIn: clientAmount,
    payerJitoSolBefore: payerTokenBefore.toString(),
    payerJitoSolAfter: payerTokenAfter.toString(),
    poolTokensOut: (payerTokenAfter - payerTokenBefore).toString(),
    helperGeneratedSignerCount: clientDeposit.signers.length,
  });

  const contributionAmount = 10_000_000n;
  const vaultBeforeContribution = (await getAccount(connection, tokenVault, 'confirmed')).amount;
  const contributionSignature = await program.methods.contributeJitosol(new BN(contributionAmount.toString()))
    .accountsStrict({
      config,
      caller: payer.publicKey,
      authority,
      sourceTokenAccount: payerToken,
      tokenVault,
      poolMint: JITOSOL_MINT,
      tokenProgram: LEGACY_TOKEN_PROGRAM,
    }).rpc();
  const vaultAfterContribution = (await getAccount(connection, tokenVault, 'confirmed')).amount;
  await record(statePath, state, connection, 'direct_jitosol_contribution', contributionSignature, {
    amount: contributionAmount.toString(),
    vaultBefore: vaultBeforeContribution.toString(),
    vaultAfter: vaultAfterContribution.toString(),
    vaultAuthority: (await getAccount(connection, tokenVault, 'confirmed')).owner.toBase58(),
  });

  const keeper = Keypair.generate();
  const keeperAirdrop = await connection.requestAirdrop(keeper.publicKey, 2 * LAMPORTS_PER_SOL);
  await connection.confirmTransaction(keeperAirdrop, 'confirmed');
  const keeperProgram = programFor(connection, keeper).program;
  const depositAmount = 1_200_000_000n;
  const poolBeforeDeposit = (await getStakePoolAccount(connection, JITO_STAKE_POOL)).account.data;
  const minimumPoolTokensOut = expectedDeposit(poolBeforeDeposit, depositAmount);
  const depositBefore = {
    solVaultLamports: await connection.getBalance(solVault, 'confirmed'),
    reserveLamports: await connection.getBalance(pool.reserveStake, 'confirmed'),
    tokenVaultAmount: (await getAccount(connection, tokenVault, 'confirmed')).amount.toString(),
  };
  const depositSignature = await keeperProgram.methods.depositSol(
    new BN(depositAmount.toString()),
    new BN(minimumPoolTokensOut.toString()),
  ).accountsStrict({
    config,
    caller: keeper.publicKey,
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
  }).rpc();
  await record(statePath, state, connection, 'cpi_direct_sol_deposit', depositSignature, {
    caller: keeper.publicKey.toBase58(),
    lamportsIn: depositAmount.toString(),
    expectedPoolTokensOut: minimumPoolTokensOut.toString(),
    before: depositBefore,
    after: {
      solVaultLamports: await connection.getBalance(solVault, 'confirmed'),
      reserveLamports: await connection.getBalance(pool.reserveStake, 'confirmed'),
      tokenVaultAmount: (await getAccount(connection, tokenVault, 'confirmed')).amount.toString(),
    },
  });

  const inactiveRound = 0n;
  const inactiveRoundPda = pda(ROUND_SEED, inactiveRound);
  const inactiveStake = pda(WITHDRAWAL_STAKE_SEED, inactiveRound);
  const inactiveCreateSignature = await keeperProgram.methods.createInactiveStakeProof(
    new BN(inactiveRound.toString()),
    new BN('1000000'),
  ).accountsStrict({
    config,
    round: inactiveRoundPda,
    caller: keeper.publicKey,
    authority,
    solVault,
    withdrawalStake: inactiveStake,
    rent: SYSVAR_RENT_PUBKEY,
    stakeProgram: STAKE_PROGRAM,
    systemProgram: SystemProgram.programId,
  }).rpc();
  await record(statePath, state, connection, 'create_inactive_stake_finalization_proof', inactiveCreateSignature, {
    round: inactiveRound.toString(),
    withdrawalStake: inactiveStake.toBase58(),
    stakeLamports: await connection.getBalance(inactiveStake, 'confirmed'),
  });
  const escrowBeforeProof = await connection.getBalance(solEscrow, 'confirmed');
  const inactiveFinalizeSignature = await keeperProgram.methods.finalizeWithdrawal(new BN(inactiveRound.toString()))
    .accountsStrict({
      config,
      round: inactiveRoundPda,
      caller: keeper.publicKey,
      authority,
      withdrawalStake: inactiveStake,
      solEscrow,
      clock: SYSVAR_CLOCK_PUBKEY,
      stakeHistory: SYSVAR_STAKE_HISTORY_PUBKEY,
      stakeProgram: STAKE_PROGRAM,
    }).rpc();
  await record(statePath, state, connection, 'finalize_inactive_stake_proof', inactiveFinalizeSignature, {
    escrowBefore: escrowBeforeProof,
    escrowAfter: await connection.getBalance(solEscrow, 'confirmed'),
    withdrawalStakeExistsAfter: (await connection.getAccountInfo(inactiveStake, 'confirmed')) !== null,
  });

  const withdrawalRound = 1n;
  const roundPda = pda(ROUND_SEED, withdrawalRound);
  const withdrawalStake = pda(WITHDRAWAL_STAKE_SEED, withdrawalRound);
  const poolBeforeWithdrawal = (await getStakePoolAccount(connection, JITO_STAKE_POOL)).account.data;
  const poolTokensIn = minimumWithdrawalInput(poolBeforeWithdrawal, 1_000_000_000n) + 1_000_000n;
  const minimumLamportsOut = expectedWithdrawal(poolBeforeWithdrawal, poolTokensIn);
  const withdrawalBefore = {
    tokenVaultAmount: (await getAccount(connection, tokenVault, 'confirmed')).amount.toString(),
    validatorStakeLamports: await connection.getBalance(TOP_VALIDATOR_STAKE, 'confirmed'),
    solVaultLamports: await connection.getBalance(solVault, 'confirmed'),
  };
  const initiateSignature = await keeperProgram.methods.initiateWithdrawal(
    new BN(withdrawalRound.toString()),
    new BN(poolTokensIn.toString()),
    new BN(minimumLamportsOut.toString()),
    true,
  ).accountsStrict({
    config,
    round: roundPda,
    caller: keeper.publicKey,
    authority,
    solVault,
    withdrawalStake,
    stakePool: JITO_STAKE_POOL,
    validatorList: pool.validatorList,
    reserveStake: pool.reserveStake,
    validatorStake: TOP_VALIDATOR_STAKE,
    tokenVault,
    managerFeeAccount: pool.managerFeeAccount,
    poolMint: JITOSOL_MINT,
    stakePoolWithdrawAuthority: withdrawAuthority,
    clock: SYSVAR_CLOCK_PUBKEY,
    stakePoolProgram: STAKE_POOL_PROGRAM,
    tokenProgram: LEGACY_TOKEN_PROGRAM,
    stakeProgram: STAKE_PROGRAM,
    systemProgram: SystemProgram.programId,
  }).rpc();
  const parsedStake = await connection.getParsedAccountInfo(withdrawalStake, 'confirmed');
  await record(statePath, state, connection, 'cpi_withdraw_stake_and_deactivate', initiateSignature, {
    caller: keeper.publicKey.toBase58(),
    round: withdrawalRound.toString(),
    withdrawalStake: withdrawalStake.toBase58(),
    poolTokensIn: poolTokensIn.toString(),
    minimumLamportsOut: minimumLamportsOut.toString(),
    before: withdrawalBefore,
    after: {
      tokenVaultAmount: (await getAccount(connection, tokenVault, 'confirmed')).amount.toString(),
      validatorStakeLamports: await connection.getBalance(TOP_VALIDATOR_STAKE, 'confirmed'),
      solVaultLamports: await connection.getBalance(solVault, 'confirmed'),
      withdrawalStakeLamports: await connection.getBalance(withdrawalStake, 'confirmed'),
      parsedStake: parsedStake.value,
    },
  });

  state.currentWithdrawalRound = withdrawalRound.toString();
  state.withdrawalStake = withdrawalStake.toBase58();
  state.deactivationEpoch = (await connection.getEpochInfo('confirmed')).epoch;
  state.expectedFirstEligibleEpoch = Number(state.deactivationEpoch) + 1;
  saveState(statePath, state);
  process.stdout.write(`${JSON.stringify(state, null, 2)}\n`);
}

async function localWithdrawalOnly() {
  const connection = new Connection('http://127.0.0.1:8899', 'confirmed');
  const payer = readKeypair(FEE_PAYER_PATH);
  const { program } = programFor(connection, payer);
  const state = loadState(LOCAL_STATE_PATH);
  const config = pda(CONFIG_SEED);
  const authority = pda(AUTHORITY_SEED);
  const solVault = pda(SOL_VAULT_SEED);
  const tokenVault = getAssociatedTokenAddressSync(JITOSOL_MINT, authority, true);
  const pool = (await getStakePoolAccount(connection, JITO_STAKE_POOL)).account.data;
  const [withdrawAuthority] = PublicKey.findProgramAddressSync(
    [JITO_STAKE_POOL.toBuffer(), Buffer.from('withdraw')],
    STAKE_POOL_PROGRAM,
  );
  const withdrawalRound = 1n;
  const roundPda = pda(ROUND_SEED, withdrawalRound);
  const withdrawalStake = pda(WITHDRAWAL_STAKE_SEED, withdrawalRound);
  const poolTokensIn = minimumWithdrawalInput(pool, 1_000_000_000n) + 1_000_000n;
  const minimumLamportsOut = expectedWithdrawal(pool, poolTokensIn);
  const withdrawalBefore = {
    tokenVaultAmount: (await getAccount(connection, tokenVault, 'confirmed')).amount.toString(),
    validatorStakeLamports: await connection.getBalance(TOP_VALIDATOR_STAKE, 'confirmed'),
    solVaultLamports: await connection.getBalance(solVault, 'confirmed'),
  };
  const withdrawalBuilder = program.methods.initiateWithdrawal(
    new BN(withdrawalRound.toString()),
    new BN(poolTokensIn.toString()),
    new BN(minimumLamportsOut.toString()),
    true,
  ).accountsStrict({
    config,
    round: roundPda,
    caller: payer.publicKey,
    authority,
    solVault,
    withdrawalStake,
    stakePool: JITO_STAKE_POOL,
    validatorList: pool.validatorList,
    reserveStake: pool.reserveStake,
    validatorStake: TOP_VALIDATOR_STAKE,
    tokenVault,
    managerFeeAccount: pool.managerFeeAccount,
    poolMint: JITOSOL_MINT,
    stakePoolWithdrawAuthority: withdrawAuthority,
    clock: SYSVAR_CLOCK_PUBKEY,
    stakePoolProgram: STAKE_POOL_PROGRAM,
    tokenProgram: LEGACY_TOKEN_PROGRAM,
    stakeProgram: STAKE_PROGRAM,
    systemProgram: SystemProgram.programId,
  });
  const simulationTransaction = await withdrawalBuilder.transaction();
  simulationTransaction.feePayer = payer.publicKey;
  simulationTransaction.recentBlockhash = (await connection.getLatestBlockhash('confirmed')).blockhash;
  simulationTransaction.sign(payer);
  const simulation = await connection.simulateTransaction(simulationTransaction, undefined, true);
  if (simulation.value.err) {
    throw new Error(`Withdrawal simulation failed: ${JSON.stringify(simulation.value.err)}\n${simulation.value.logs?.join('\n')}`);
  }
  const signature = await withdrawalBuilder.rpc();
  const parsedStake = await connection.getParsedAccountInfo(withdrawalStake, 'confirmed');
  await record(LOCAL_STATE_PATH, state, connection, 'cpi_withdraw_stake_and_deactivate', signature, {
    caller: payer.publicKey.toBase58(),
    round: withdrawalRound.toString(),
    withdrawalStake: withdrawalStake.toBase58(),
    poolTokensIn: poolTokensIn.toString(),
    minimumLamportsOut: minimumLamportsOut.toString(),
    before: withdrawalBefore,
    after: {
      tokenVaultAmount: (await getAccount(connection, tokenVault, 'confirmed')).amount.toString(),
      validatorStakeLamports: await connection.getBalance(TOP_VALIDATOR_STAKE, 'confirmed'),
      solVaultLamports: await connection.getBalance(solVault, 'confirmed'),
      withdrawalStakeLamports: await connection.getBalance(withdrawalStake, 'confirmed'),
      parsedStake: parsedStake.value,
    },
  });
  state.currentWithdrawalRound = withdrawalRound.toString();
  state.withdrawalStake = withdrawalStake.toBase58();
  state.deactivationEpoch = (await connection.getEpochInfo('confirmed')).epoch;
  state.expectedFirstEligibleEpoch = Number(state.deactivationEpoch) + 1;
  saveState(LOCAL_STATE_PATH, state);
  process.stdout.write(`${JSON.stringify(state.transactions, null, 2)}\n`);
}

async function finalize(mode: NetworkMode, roundId: bigint) {
  const connection = new Connection(mode === 'local' ? 'http://127.0.0.1:8899' : clusterApiUrl('testnet'), 'confirmed');
  const payer = readKeypair(FEE_PAYER_PATH);
  const { program } = programFor(connection, payer);
  const statePath = mode === 'local' ? LOCAL_STATE_PATH : TESTNET_STATE_PATH;
  const state = loadState(statePath);
  const config = pda(CONFIG_SEED);
  const authority = pda(AUTHORITY_SEED);
  const solEscrow = pda(SOL_ESCROW_SEED);
  const round = pda(ROUND_SEED, roundId);
  const withdrawalStake = pda(WITHDRAWAL_STAKE_SEED, roundId);
  const escrowBefore = await connection.getBalance(solEscrow, 'confirmed');
  const stakeBefore = await connection.getBalance(withdrawalStake, 'confirmed');
  const signature = await program.methods.finalizeWithdrawal(new BN(roundId.toString()))
    .accountsStrict({
      config,
      round,
      caller: payer.publicKey,
      authority,
      withdrawalStake,
      solEscrow,
      clock: SYSVAR_CLOCK_PUBKEY,
      stakeHistory: SYSVAR_STAKE_HISTORY_PUBKEY,
      stakeProgram: STAKE_PROGRAM,
    }).rpc();
  await record(statePath, state, connection, `finalize_round_${roundId}`, signature, {
    caller: payer.publicKey.toBase58(),
    stakeBefore,
    stakeAfter: await connection.getBalance(withdrawalStake, 'confirmed'),
    escrowBefore,
    escrowAfter: await connection.getBalance(solEscrow, 'confirmed'),
  });
  state.complete = true;
  saveState(statePath, state);
  process.stdout.write(`${JSON.stringify(state, null, 2)}\n`);
}

async function main() {
  const command = process.argv[2];
  if (command === 'local-init') return localInitialize();
  if (command === 'local-withdraw') return localWithdrawalOnly();
  if (command === 'finalize') {
    const mode = process.argv[3] as NetworkMode;
    const round = BigInt(process.argv[4]);
    return finalize(mode, round);
  }
  throw new Error('Usage: lifecycle.ts local-init | finalize <local|testnet> <round>');
}

main().catch((error: unknown) => {
  process.stderr.write(`${error instanceof Error ? error.stack : inspect(error, { depth: 8 })}\n`);
  process.exitCode = 1;
});
