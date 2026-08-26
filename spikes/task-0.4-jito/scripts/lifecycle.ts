import fs from 'node:fs';
import { inspect } from 'node:util';
import path from 'node:path';
import process from 'node:process';

import { AnchorProvider, BN, Idl, Program, Wallet } from '@coral-xyz/anchor';
import {
  createAssociatedTokenAccountIdempotentInstruction,
  getAccount,
  getAssociatedTokenAddressSync,
  getMint,
} from '@solana/spl-token';
import {
  depositSol as buildClientDeposit,
  getStakePoolAccount,
  StakePoolInstruction,
} from '@solana/spl-stake-pool';
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
const TOP_VALIDATOR_VOTE = new PublicKey('vouNpQ4b6mZRAKHG312QrBhbG3t5QdBLRuWXr2YYevo');
const TOP_VALIDATOR_INDEX = 485;

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

function minimumDepositInput(
  pool: Awaited<ReturnType<typeof getStakePoolAccount>>['account']['data'],
  targetPoolTokens: bigint,
) {
  let low = 1n;
  let high = 1n;
  while (expectedDeposit(pool, high) < targetPoolTokens) high *= 2n;
  while (low < high) {
    const mid = low + (high - low) / 2n;
    if (expectedDeposit(pool, mid) >= targetPoolTokens) high = mid;
    else low = mid + 1n;
  }
  return low;
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

async function testnetStart() {
  const connection = new Connection(clusterApiUrl('testnet'), 'confirmed');
  const payer = readKeypair(FEE_PAYER_PATH);
  const { program } = programFor(connection, payer);
  const state = loadState(TESTNET_STATE_PATH);
  const existingTransactions = (state.transactions ?? {}) as JsonRecord;
  if (Object.keys(existingTransactions).length > 0) {
    throw new Error('Testnet lifecycle state already contains transactions; use the recorded resume state');
  }

  const config = pda(CONFIG_SEED);
  const authority = pda(AUTHORITY_SEED);
  const solVault = pda(SOL_VAULT_SEED);
  const solEscrow = pda(SOL_ESCROW_SEED);
  const tokenVault = getAssociatedTokenAddressSync(JITOSOL_MINT, authority, true);
  const payerToken = getAssociatedTokenAddressSync(JITOSOL_MINT, payer.publicKey);
  const withdrawalRound = 0n;
  const roundPda = pda(ROUND_SEED, withdrawalRound);
  const withdrawalStake = pda(WITHDRAWAL_STAKE_SEED, withdrawalRound);

  const programAccount = await connection.getAccountInfo(PROBE_PROGRAM, 'confirmed');
  if (!programAccount?.executable) throw new Error('Probe program is not deployed and executable');
  if (await connection.getAccountInfo(config, 'confirmed')) {
    throw new Error('Probe config already exists without matching Testnet resume evidence');
  }

  const epochInfo = await connection.getEpochInfo('confirmed');
  let pool = (await getStakePoolAccount(connection, JITO_STAKE_POOL)).account.data;
  if (BigInt(pool.lastUpdateEpoch.toString()) !== BigInt(epochInfo.epoch)) {
    throw new Error(`Stake pool is stale: pool ${pool.lastUpdateEpoch.toString()}, clock ${epochInfo.epoch}`);
  }

  const validatorListAccount = await connection.getAccountInfo(pool.validatorList, 'confirmed');
  if (!validatorListAccount) throw new Error('Official validator list does not exist');
  if (!validatorListAccount.owner.equals(STAKE_POOL_PROGRAM)) throw new Error('Validator-list owner mismatch');
  const entryOffset = 9 + TOP_VALIDATOR_INDEX * 73;
  const selectedLastUpdateEpoch = validatorListAccount.data.readBigUInt64LE(entryOffset + 16);
  const selectedStatus = validatorListAccount.data.readUInt8(entryOffset + 40);
  const selectedVote = new PublicKey(validatorListAccount.data.subarray(entryOffset + 41, entryOffset + 73));
  if (selectedLastUpdateEpoch !== BigInt(epochInfo.epoch)) throw new Error('Selected validator entry is stale');
  if (selectedStatus !== 0) throw new Error('Selected validator entry is not active');
  if (!selectedVote.equals(TOP_VALIDATOR_VOTE)) throw new Error('Selected validator-list vote account changed');
  const derivedValidatorStake = PublicKey.findProgramAddressSync(
    [selectedVote.toBuffer(), JITO_STAKE_POOL.toBuffer(), Buffer.alloc(0)],
    STAKE_POOL_PROGRAM,
  )[0];
  if (!derivedValidatorStake.equals(TOP_VALIDATOR_STAKE)) throw new Error('Selected validator stake derivation changed');
  const validatorStakeAccount = await connection.getAccountInfo(TOP_VALIDATOR_STAKE, 'confirmed');
  if (!validatorStakeAccount?.owner.equals(STAKE_PROGRAM)) throw new Error('Selected validator stake owner mismatch');

  const [withdrawAuthority] = PublicKey.findProgramAddressSync(
    [JITO_STAKE_POOL.toBuffer(), Buffer.from('withdraw')],
    STAKE_POOL_PROGRAM,
  );
  const minimumDelegation = BigInt((await connection.getStakeMinimumDelegation({ commitment: 'confirmed' })).value);
  const stakeRent = BigInt(await connection.getMinimumBalanceForRentExemption(StakeProgram.space, 'confirmed'));
  const systemAccountRent = BigInt(await connection.getMinimumBalanceForRentExemption(0, 'confirmed'));

  state.network = 'public-testnet';
  state.rpcEndpoint = clusterApiUrl('testnet');
  state.programId = PROBE_PROGRAM.toBase58();
  state.feePayer = payer.publicKey.toBase58();
  state.startEpoch = epochInfo.epoch;
  state.publicAccounts = {
    config: config.toBase58(),
    authority: authority.toBase58(),
    solVault: solVault.toBase58(),
    solEscrow: solEscrow.toBase58(),
    tokenVault: tokenVault.toBase58(),
    payerToken: payerToken.toBase58(),
    validatorList: pool.validatorList.toBase58(),
    validatorListIndex: TOP_VALIDATOR_INDEX,
    validatorVote: selectedVote.toBase58(),
    validatorStake: TOP_VALIDATOR_STAKE.toBase58(),
    round: roundPda.toBase58(),
    withdrawalStake: withdrawalStake.toBase58(),
  };
  state.preflight = {
    poolLastUpdateEpoch: pool.lastUpdateEpoch.toString(),
    selectedValidatorLastUpdateEpoch: selectedLastUpdateEpoch.toString(),
    selectedValidatorStatus: selectedStatus,
    selectedValidatorStakeLamports: validatorStakeAccount.lamports,
    minimumDelegationLamports: minimumDelegation.toString(),
    stakeRentLamports: stakeRent.toString(),
    systemAccountRentLamports: systemAccountRent.toString(),
  };
  saveState(TESTNET_STATE_PATH, state);

  const directContributionAmount = 1_000_000n;
  const baselinePoolBefore = pool;
  const baselineLamportsIn = minimumDepositInput(baselinePoolBefore, directContributionAmount);
  const baselineExpectedOut = expectedDeposit(baselinePoolBefore, baselineLamportsIn);
  const baselinePayerBefore = await connection.getBalance(payer.publicKey, 'confirmed');
  const baselineReserveBefore = await connection.getBalance(pool.reserveStake, 'confirmed');
  const baselinePayerTokenBefore = (await connection.getAccountInfo(payerToken, 'confirmed'))
    ? (await getAccount(connection, payerToken, 'confirmed')).amount
    : 0n;
  const baselineSignature = await sendInstructions(connection, payer, [
    createAssociatedTokenAccountIdempotentInstruction(
      payer.publicKey,
      payerToken,
      payer.publicKey,
      JITOSOL_MINT,
    ),
    StakePoolInstruction.depositSol({
      stakePool: JITO_STAKE_POOL,
      withdrawAuthority,
      reserveStake: pool.reserveStake,
      fundingAccount: payer.publicKey,
      destinationPoolAccount: payerToken,
      managerFeeAccount: pool.managerFeeAccount,
      referralPoolAccount: pool.managerFeeAccount,
      poolMint: JITOSOL_MINT,
      lamports: Number(baselineLamportsIn),
    }),
  ]);
  const baselinePayerTokenAfter = (await getAccount(connection, payerToken, 'confirmed')).amount;
  if (baselinePayerTokenAfter - baselinePayerTokenBefore !== baselineExpectedOut) {
    throw new Error('Direct client deposit output differs from decoded pool calculation');
  }
  await record(TESTNET_STATE_PATH, state, connection, 'baseline_direct_client_deposit', baselineSignature, {
    caller: payer.publicKey.toBase58(),
    lamportsIn: baselineLamportsIn.toString(),
    expectedPoolTokensOut: baselineExpectedOut.toString(),
    before: {
      payerLamports: baselinePayerBefore,
      reserveLamports: baselineReserveBefore,
      payerJitoSol: baselinePayerTokenBefore.toString(),
    },
    after: {
      payerLamports: await connection.getBalance(payer.publicKey, 'confirmed'),
      reserveLamports: await connection.getBalance(pool.reserveStake, 'confirmed'),
      payerJitoSol: baselinePayerTokenAfter.toString(),
    },
    helperGeneratedSignerCount: 0,
  });

  pool = (await getStakePoolAccount(connection, JITO_STAKE_POOL)).account.data;
  const minimumPoolTokensAtDeposit = minimumWithdrawalInput(pool, minimumDelegation);
  const pivDepositLamports = minimumDepositInput(pool, minimumPoolTokensAtDeposit);
  const vaultFunding = pivDepositLamports + stakeRent + systemAccountRent;
  const setupBefore = {
    payerLamports: await connection.getBalance(payer.publicKey, 'confirmed'),
    solVaultLamports: await connection.getBalance(solVault, 'confirmed'),
    solEscrowLamports: await connection.getBalance(solEscrow, 'confirmed'),
  };
  const setupSignature = await sendInstructions(connection, payer, [
    createAssociatedTokenAccountIdempotentInstruction(
      payer.publicKey,
      tokenVault,
      authority,
      JITOSOL_MINT,
    ),
    SystemProgram.transfer({
      fromPubkey: payer.publicKey,
      toPubkey: solVault,
      lamports: Number(vaultFunding),
    }),
    SystemProgram.transfer({
      fromPubkey: payer.publicKey,
      toPubkey: solEscrow,
      lamports: Number(systemAccountRent),
    }),
  ]);
  await record(TESTNET_STATE_PATH, state, connection, 'fund_piv_accounts', setupSignature, {
    pivDepositLamports: pivDepositLamports.toString(),
    withdrawalStakeRentLamports: stakeRent.toString(),
    retainedSolVaultRentLamports: systemAccountRent.toString(),
    fixedSolEscrowRentLamports: systemAccountRent.toString(),
    before: setupBefore,
    after: {
      payerLamports: await connection.getBalance(payer.publicKey, 'confirmed'),
      solVaultLamports: await connection.getBalance(solVault, 'confirmed'),
      solEscrowLamports: await connection.getBalance(solEscrow, 'confirmed'),
      tokenVaultRentLamports: await connection.getBalance(tokenVault, 'confirmed'),
    },
  });

  const initializeSignature = await program.methods.initialize().accountsStrict({
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
  await record(TESTNET_STATE_PATH, state, connection, 'initialize_probe', initializeSignature, {
    configRentLamports: await connection.getBalance(config, 'confirmed'),
  });

  const poolBeforeDeposit = (await getStakePoolAccount(connection, JITO_STAKE_POOL)).account.data;
  if (BigInt(poolBeforeDeposit.lastUpdateEpoch.toString()) !== BigInt((await connection.getEpochInfo('confirmed')).epoch)) {
    throw new Error('Stake pool became stale before the CPI deposit');
  }
  const minimumPoolTokensOut = expectedDeposit(poolBeforeDeposit, pivDepositLamports);
  const depositBefore = {
    solVaultLamports: await connection.getBalance(solVault, 'confirmed'),
    reserveLamports: await connection.getBalance(pool.reserveStake, 'confirmed'),
    tokenVaultAmount: (await getAccount(connection, tokenVault, 'confirmed')).amount.toString(),
    poolSupply: (await getMint(connection, JITOSOL_MINT, 'confirmed')).supply.toString(),
  };
  const depositSignature = await program.methods.depositSol(
    new BN(pivDepositLamports.toString()),
    new BN(minimumPoolTokensOut.toString()),
  ).accountsStrict({
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
  }).rpc();
  const depositAfterToken = (await getAccount(connection, tokenVault, 'confirmed')).amount;
  await record(TESTNET_STATE_PATH, state, connection, 'cpi_direct_sol_deposit', depositSignature, {
    caller: payer.publicKey.toBase58(),
    lamportsIn: pivDepositLamports.toString(),
    minimumPoolTokensOut: minimumPoolTokensOut.toString(),
    observedPoolTokensOut: (depositAfterToken - BigInt(depositBefore.tokenVaultAmount)).toString(),
    before: depositBefore,
    after: {
      solVaultLamports: await connection.getBalance(solVault, 'confirmed'),
      reserveLamports: await connection.getBalance(pool.reserveStake, 'confirmed'),
      tokenVaultAmount: depositAfterToken.toString(),
      poolSupply: (await getMint(connection, JITOSOL_MINT, 'confirmed')).supply.toString(),
    },
  });

  const payerTokenBeforeContribution = (await getAccount(connection, payerToken, 'confirmed')).amount;
  const vaultBeforeContribution = (await getAccount(connection, tokenVault, 'confirmed')).amount;
  const contributionSignature = await program.methods.contributeJitosol(
    new BN(directContributionAmount.toString()),
  ).accountsStrict({
    config,
    caller: payer.publicKey,
    authority,
    sourceTokenAccount: payerToken,
    tokenVault,
    poolMint: JITOSOL_MINT,
    tokenProgram: LEGACY_TOKEN_PROGRAM,
  }).rpc();
  const payerTokenAfterContribution = (await getAccount(connection, payerToken, 'confirmed')).amount;
  const vaultAfterContribution = (await getAccount(connection, tokenVault, 'confirmed')).amount;
  await record(TESTNET_STATE_PATH, state, connection, 'direct_jitosol_contribution', contributionSignature, {
    caller: payer.publicKey.toBase58(),
    amount: directContributionAmount.toString(),
    sourceBefore: payerTokenBeforeContribution.toString(),
    sourceAfter: payerTokenAfterContribution.toString(),
    vaultBefore: vaultBeforeContribution.toString(),
    vaultAfter: vaultAfterContribution.toString(),
    vaultAuthority: (await getAccount(connection, tokenVault, 'confirmed')).owner.toBase58(),
  });

  const poolBeforeWithdrawal = (await getStakePoolAccount(connection, JITO_STAKE_POOL)).account.data;
  const withdrawalEpoch = (await connection.getEpochInfo('confirmed')).epoch;
  if (BigInt(poolBeforeWithdrawal.lastUpdateEpoch.toString()) !== BigInt(withdrawalEpoch)) {
    throw new Error('Stake pool became stale before the CPI withdrawal');
  }
  const minimumPoolTokensAtWithdrawal = minimumWithdrawalInput(poolBeforeWithdrawal, minimumDelegation);
  const poolTokensIn = (await getAccount(connection, tokenVault, 'confirmed')).amount;
  if (poolTokensIn < minimumPoolTokensAtWithdrawal) throw new Error('PIV token vault is below the dynamic withdrawal minimum');
  const minimumLamportsOut = expectedWithdrawal(poolBeforeWithdrawal, poolTokensIn);
  const managerFeeBefore = (await getAccount(connection, pool.managerFeeAccount, 'confirmed')).amount;
  const mintBeforeWithdrawal = await getMint(connection, JITOSOL_MINT, 'confirmed');
  const withdrawalBefore = {
    tokenVaultAmount: poolTokensIn.toString(),
    validatorStakeLamports: await connection.getBalance(TOP_VALIDATOR_STAKE, 'confirmed'),
    solVaultLamports: await connection.getBalance(solVault, 'confirmed'),
    managerFeeAmount: managerFeeBefore.toString(),
    poolMintSupply: mintBeforeWithdrawal.supply.toString(),
  };
  const initiateSignature = await program.methods.initiateWithdrawal(
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
  }).rpc();
  const parsedStake = await connection.getParsedAccountInfo(withdrawalStake, 'confirmed');
  const managerFeeAfter = (await getAccount(connection, pool.managerFeeAccount, 'confirmed')).amount;
  const mintAfterWithdrawal = await getMint(connection, JITOSOL_MINT, 'confirmed');
  await record(TESTNET_STATE_PATH, state, connection, 'cpi_withdraw_stake_and_deactivate', initiateSignature, {
    caller: payer.publicKey.toBase58(),
    round: withdrawalRound.toString(),
    withdrawalStake: withdrawalStake.toBase58(),
    dynamicMinimumPoolTokens: minimumPoolTokensAtWithdrawal.toString(),
    poolTokensIn: poolTokensIn.toString(),
    minimumLamportsOut: minimumLamportsOut.toString(),
    withdrawalFeePoolTokens: (managerFeeAfter - managerFeeBefore).toString(),
    burnedPoolTokens: (mintBeforeWithdrawal.supply - mintAfterWithdrawal.supply).toString(),
    before: withdrawalBefore,
    after: {
      tokenVaultAmount: (await getAccount(connection, tokenVault, 'confirmed')).amount.toString(),
      validatorStakeLamports: await connection.getBalance(TOP_VALIDATOR_STAKE, 'confirmed'),
      solVaultLamports: await connection.getBalance(solVault, 'confirmed'),
      withdrawalStakeLamports: await connection.getBalance(withdrawalStake, 'confirmed'),
      managerFeeAmount: managerFeeAfter.toString(),
      poolMintSupply: mintAfterWithdrawal.supply.toString(),
      parsedStake: parsedStake.value,
    },
  });

  const prematureTransaction = await program.methods.finalizeWithdrawal(new BN(withdrawalRound.toString()))
    .accountsStrict({
      config,
      round: roundPda,
      caller: payer.publicKey,
      authority,
      withdrawalStake,
      solEscrow,
      clock: SYSVAR_CLOCK_PUBKEY,
      stakeHistory: SYSVAR_STAKE_HISTORY_PUBKEY,
      stakeProgram: STAKE_PROGRAM,
    }).transaction();
  const prematureBlockhash = await connection.getLatestBlockhash('confirmed');
  prematureTransaction.feePayer = payer.publicKey;
  prematureTransaction.recentBlockhash = prematureBlockhash.blockhash;
  prematureTransaction.sign(payer);
  const prematureSignature = await connection.sendRawTransaction(prematureTransaction.serialize(), {
    skipPreflight: true,
    maxRetries: 5,
  });
  const prematureResult = await connection.confirmTransaction(
    { signature: prematureSignature, ...prematureBlockhash },
    'confirmed',
  );
  if (!prematureResult.value.err) throw new Error('Premature finalization unexpectedly succeeded');
  const prematureEvidence = await transactionEvidence(connection, prematureSignature);
  const prematureLogs = (prematureEvidence.logMessages ?? []) as string[];
  if (!prematureLogs.some((log) => log.includes('StakeNotDeactivated'))) {
    throw new Error(`Premature finalization failed for an unexpected reason: ${JSON.stringify(prematureEvidence)}`);
  }
  const transactions = (state.transactions ?? {}) as JsonRecord;
  transactions.premature_finalization_rejected = {
    ...prematureEvidence,
    expectedError: 'StakeNotDeactivated',
    expectedCustomErrorCode: 6024,
    accountChanges: {
      withdrawalStakeLamports: await connection.getBalance(withdrawalStake, 'confirmed'),
      solEscrowLamports: await connection.getBalance(solEscrow, 'confirmed'),
    },
  };
  state.transactions = transactions;

  const finalEpochInfo = await connection.getEpochInfo('confirmed');
  state.currentWithdrawalRound = withdrawalRound.toString();
  state.withdrawalStake = withdrawalStake.toBase58();
  state.deactivationEpoch = withdrawalEpoch;
  state.expectedFirstEligibleEpoch = withdrawalEpoch + 1;
  state.observedEpochAfterPrematureFinalization = finalEpochInfo.epoch;
  state.complete = false;
  state.resumeCommand = `npx tsx scripts/lifecycle.ts finalize testnet ${withdrawalRound}`;
  saveState(TESTNET_STATE_PATH, state);
  process.stdout.write(`${JSON.stringify(state, null, 2)}\n`);
}

async function reconcileTestnetAfterRpcLoss(withdrawalSignature: string, prematureSignature: string) {
  const connection = new Connection(clusterApiUrl('testnet'), 'confirmed');
  const state = loadState(TESTNET_STATE_PATH);
  const transactions = (state.transactions ?? {}) as JsonRecord;
  if (!transactions.direct_jitosol_contribution) {
    throw new Error('Cannot reconcile without the recorded direct-contribution stage');
  }
  if (transactions.cpi_withdraw_stake_and_deactivate || transactions.premature_finalization_rejected) {
    throw new Error('Withdrawal or premature-finalization evidence is already recorded');
  }

  const config = pda(CONFIG_SEED);
  const authority = pda(AUTHORITY_SEED);
  const solVault = pda(SOL_VAULT_SEED);
  const solEscrow = pda(SOL_ESCROW_SEED);
  const tokenVault = getAssociatedTokenAddressSync(JITOSOL_MINT, authority, true);
  const withdrawalRound = 0n;
  const roundPda = pda(ROUND_SEED, withdrawalRound);
  const withdrawalStake = pda(WITHDRAWAL_STAKE_SEED, withdrawalRound);
  const pool = (await getStakePoolAccount(connection, JITO_STAKE_POOL)).account.data;

  const withdrawalEvidence = await transactionEvidence(connection, withdrawalSignature);
  const withdrawalLogs = ('logMessages' in withdrawalEvidence ? withdrawalEvidence.logMessages : []) ?? [];
  if ('error' in withdrawalEvidence && withdrawalEvidence.error) {
    throw new Error(`Recorded withdrawal transaction failed: ${JSON.stringify(withdrawalEvidence.error)}`);
  }
  if (!withdrawalLogs.some((log) => log.includes('Instruction: WithdrawStakeWithSlippage'))
    || !withdrawalLogs.some((log) => log.includes('Instruction: Deactivate'))) {
    throw new Error(`Signature is not the successful combined withdrawal/deactivation: ${withdrawalSignature}`);
  }

  const prematureEvidence = await transactionEvidence(connection, prematureSignature);
  const prematureLogs = ('logMessages' in prematureEvidence ? prematureEvidence.logMessages : []) ?? [];
  if (!prematureLogs.some((log) => log.includes('StakeNotDeactivated'))) {
    throw new Error(`Signature is not the expected premature-finalization rejection: ${prematureSignature}`);
  }

  const contributionStage = transactions.direct_jitosol_contribution as JsonRecord;
  const contributionChanges = contributionStage.accountChanges as JsonRecord;
  const poolTokensIn = BigInt(contributionChanges.vaultAfter as string);
  const depositStage = transactions.cpi_direct_sol_deposit as JsonRecord;
  const depositChanges = depositStage.accountChanges as JsonRecord;
  const depositAfter = depositChanges.after as JsonRecord;
  const minimumPoolTokens = BigInt(depositChanges.minimumPoolTokensOut as string);
  const mintSupplyBefore = BigInt(depositAfter.poolSupply as string);
  const validatorStakeBefore = BigInt(((state.preflight as JsonRecord).selectedValidatorStakeLamports as number).toString());
  const solVaultBefore = BigInt((depositAfter.solVaultLamports as number).toString());
  const stakeRent = BigInt((state.preflight as JsonRecord).stakeRentLamports as string);
  const withdrawalStakeLamports = BigInt(await connection.getBalance(withdrawalStake, 'confirmed'));
  if (withdrawalStakeLamports <= stakeRent) throw new Error('Withdrawal stake account does not contain delegated stake');
  const stakeLamportsOut = withdrawalStakeLamports - stakeRent;
  const withdrawalFee = feeCeil(
    poolTokensIn,
    BigInt(pool.stakeWithdrawalFee.numerator.toString()),
    BigInt(pool.stakeWithdrawalFee.denominator.toString()),
  );
  const burnedPoolTokens = poolTokensIn - withdrawalFee;
  const managerFeeAfter = (await getAccount(connection, pool.managerFeeAccount, 'confirmed')).amount;
  const mintSupplyAfter = (await getMint(connection, JITOSOL_MINT, 'confirmed')).supply;
  const parsedStake = await connection.getParsedAccountInfo(withdrawalStake, 'confirmed');
  const roundAccount = await connection.getAccountInfo(roundPda, 'confirmed');
  if (!roundAccount?.owner.equals(PROBE_PROGRAM)) throw new Error('Withdrawal round account is missing or has the wrong owner');
  if (mintSupplyBefore - mintSupplyAfter !== burnedPoolTokens) {
    throw new Error('Mint supply delta does not match the withdrawal burn');
  }
  if (validatorStakeBefore - BigInt(await connection.getBalance(TOP_VALIDATOR_STAKE, 'confirmed')) !== stakeLamportsOut) {
    throw new Error('Validator stake delta does not match the withdrawal output');
  }

  transactions.cpi_withdraw_stake_and_deactivate = {
    ...withdrawalEvidence,
    recoveredAfterRpcResponseLoss: true,
    accountChanges: {
      caller: state.feePayer,
      round: withdrawalRound.toString(),
      roundPda: roundPda.toBase58(),
      roundRentLamports: roundAccount.lamports,
      withdrawalStake: withdrawalStake.toBase58(),
      dynamicMinimumPoolTokens: minimumPoolTokens.toString(),
      poolTokensIn: poolTokensIn.toString(),
      minimumLamportsOut: stakeLamportsOut.toString(),
      withdrawalFeePoolTokens: withdrawalFee.toString(),
      burnedPoolTokens: burnedPoolTokens.toString(),
      before: {
        tokenVaultAmount: poolTokensIn.toString(),
        validatorStakeLamports: validatorStakeBefore.toString(),
        solVaultLamports: solVaultBefore.toString(),
        managerFeeAmount: (managerFeeAfter - withdrawalFee).toString(),
        poolMintSupply: mintSupplyBefore.toString(),
      },
      after: {
        tokenVaultAmount: (await getAccount(connection, tokenVault, 'confirmed')).amount.toString(),
        validatorStakeLamports: (await connection.getBalance(TOP_VALIDATOR_STAKE, 'confirmed')).toString(),
        solVaultLamports: (await connection.getBalance(solVault, 'confirmed')).toString(),
        withdrawalStakeLamports: withdrawalStakeLamports.toString(),
        managerFeeAmount: managerFeeAfter.toString(),
        poolMintSupply: mintSupplyAfter.toString(),
        parsedStake: parsedStake.value,
      },
    },
  };
  transactions.premature_finalization_rejected = {
    ...prematureEvidence,
    recoveredAfterRpcResponseLoss: true,
    expectedError: 'StakeNotDeactivated',
    expectedCustomErrorCode: 6024,
    accountChanges: {
      withdrawalStakeLamports: withdrawalStakeLamports.toString(),
      solEscrowLamports: (await connection.getBalance(solEscrow, 'confirmed')).toString(),
    },
  };
  state.transactions = transactions;

  const finalEpochInfo = await connection.getEpochInfo('confirmed');
  state.currentWithdrawalRound = withdrawalRound.toString();
  state.withdrawalStake = withdrawalStake.toBase58();
  state.deactivationEpoch = finalEpochInfo.epoch;
  state.expectedFirstEligibleEpoch = finalEpochInfo.epoch + 1;
  state.observedEpochAfterPrematureFinalization = finalEpochInfo.epoch;
  state.complete = false;
  state.resumeCommand = `npx tsx scripts/lifecycle.ts finalize testnet ${withdrawalRound}`;
  saveState(TESTNET_STATE_PATH, state);
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
  if (command === 'testnet-start') return testnetStart();
  if (command === 'testnet-reconcile') {
    return reconcileTestnetAfterRpcLoss(process.argv[3], process.argv[4]);
  }
  if (command === 'finalize') {
    const mode = process.argv[3] as NetworkMode;
    const round = BigInt(process.argv[4]);
    return finalize(mode, round);
  }
  throw new Error('Usage: lifecycle.ts local-init | local-withdraw | testnet-start | testnet-reconcile <withdrawal-signature> <premature-signature> | finalize <local|testnet> <round>');
}

main().catch((error: unknown) => {
  process.stderr.write(`${error instanceof Error ? error.stack : inspect(error, { depth: 8 })}\n`);
  process.exitCode = 1;
});
