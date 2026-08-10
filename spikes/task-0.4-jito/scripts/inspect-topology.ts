import { getAccount, getMint } from '@solana/spl-token';
import { getStakePoolAccount } from '@solana/spl-stake-pool';
import {
  clusterApiUrl,
  Connection,
  PublicKey,
  StakeProgram,
} from '@solana/web3.js';

import {
  JITOSOL_MINT,
  JITO_STAKE_POOL,
  LEGACY_TOKEN_PROGRAM,
  STAKE_POOL_PROGRAM,
} from './constants.js';

type Cluster = 'testnet' | 'devnet' | 'mainnet-beta';

const outputPath = process.argv[3];

interface Fee {
  denominator: { toString(radix?: number): string };
  numerator: { toString(radix?: number): string };
}

interface ValidatorEntry {
  index: number;
  activeStakeLamports: bigint;
  transientStakeLamports: bigint;
  lastUpdateEpoch: bigint;
  transientSeedSuffix: bigint;
  validatorSeedSuffix: number;
  status: number;
  voteAccount: PublicKey;
  stakeAccount: PublicKey;
  stakeAccountLamports: number | null;
  stakeAccountOwner: string | null;
}

function fee(value: Fee) {
  return {
    numerator: value.numerator.toString(),
    denominator: value.denominator.toString(),
  };
}

function ceilDiv(value: bigint, denominator: bigint): bigint {
  return (value + denominator - 1n) / denominator;
}

function withdrawalLamports(
  input: bigint,
  totalLamports: bigint,
  supply: bigint,
  feeNumerator: bigint,
  feeDenominator: bigint,
): bigint {
  const withdrawalFee = feeDenominator === 0n ? 0n : ceilDiv(input * feeNumerator, feeDenominator);
  return ((input - withdrawalFee) * totalLamports) / supply;
}

function minimumPoolTokensForLamports(
  target: bigint,
  totalLamports: bigint,
  supply: bigint,
  feeNumerator: bigint,
  feeDenominator: bigint,
): bigint {
  let low = 1n;
  let high = supply;
  while (low < high) {
    const mid = low + (high - low) / 2n;
    if (withdrawalLamports(mid, totalLamports, supply, feeNumerator, feeDenominator) >= target) {
      high = mid;
    } else {
      low = mid + 1n;
    }
  }
  return low;
}

function decodeValidatorEntries(data: Buffer): Omit<ValidatorEntry, 'stakeAccount' | 'stakeAccountLamports' | 'stakeAccountOwner'>[] {
  if (data.length < 9) throw new Error('Validator list is too short');
  const vectorLength = data.readUInt32LE(5);
  const entries: Omit<ValidatorEntry, 'stakeAccount' | 'stakeAccountLamports' | 'stakeAccountOwner'>[] = [];
  for (let index = 0; index < vectorLength; index += 1) {
    const offset = 9 + index * 73;
    if (offset + 73 > data.length) throw new Error(`Validator entry ${index} exceeds account data`);
    const voteAccount = new PublicKey(data.subarray(offset + 41, offset + 73));
    if (voteAccount.equals(PublicKey.default)) continue;
    entries.push({
      index,
      activeStakeLamports: data.readBigUInt64LE(offset),
      transientStakeLamports: data.readBigUInt64LE(offset + 8),
      lastUpdateEpoch: data.readBigUInt64LE(offset + 16),
      transientSeedSuffix: data.readBigUInt64LE(offset + 24),
      validatorSeedSuffix: data.readUInt32LE(offset + 36),
      status: data.readUInt8(offset + 40),
      voteAccount,
    });
  }
  return entries;
}

function deriveValidatorStake(vote: PublicKey, seedSuffix: number): PublicKey {
  const suffix = seedSuffix === 0 ? Buffer.alloc(0) : Buffer.alloc(4);
  if (seedSuffix !== 0) suffix.writeUInt32LE(seedSuffix);
  return PublicKey.findProgramAddressSync(
    [vote.toBuffer(), JITO_STAKE_POOL.toBuffer(), suffix],
    STAKE_POOL_PROGRAM,
  )[0];
}

async function accountMetadata(connection: Connection, address: PublicKey) {
  const account = await connection.getAccountInfo(address, 'confirmed');
  if (!account) return null;
  return {
    address: address.toBase58(),
    owner: account.owner.toBase58(),
    executable: account.executable,
    lamports: account.lamports,
    space: account.data.length,
  };
}

async function main() {
  const cluster = (process.argv[2] ?? 'testnet') as Cluster;
  if (!['testnet', 'devnet', 'mainnet-beta'].includes(cluster)) {
    throw new Error(`Unsupported cluster ${cluster}`);
  }
  const endpoint = clusterApiUrl(cluster);
  const connection = new Connection(endpoint, 'confirmed');
  const epochInfo = await connection.getEpochInfo('confirmed');
  const poolAccount = await getStakePoolAccount(connection, JITO_STAKE_POOL);
  const pool = poolAccount.account.data;
  const poolRaw = await connection.getAccountInfo(JITO_STAKE_POOL, 'confirmed');
  if (!poolRaw) throw new Error('Jito stake-pool account does not exist');
  if (!poolRaw.owner.equals(STAKE_POOL_PROGRAM)) throw new Error('Jito stake-pool owner mismatch');
  if (!pool.poolMint.equals(JITOSOL_MINT)) throw new Error('JitoSOL mint mismatch in pool state');
  if (!pool.tokenProgramId.equals(LEGACY_TOKEN_PROGRAM)) throw new Error('Token program mismatch in pool state');

  const [withdrawAuthority, withdrawBump] = PublicKey.findProgramAddressSync(
    [JITO_STAKE_POOL.toBuffer(), Buffer.from('withdraw')],
    STAKE_POOL_PROGRAM,
  );
  if (withdrawBump !== pool.stakeWithdrawBumpSeed) throw new Error('Withdraw bump mismatch');

  const mint = await getMint(connection, JITOSOL_MINT, 'confirmed', LEGACY_TOKEN_PROGRAM);
  if (!mint.mintAuthority?.equals(withdrawAuthority)) throw new Error('JitoSOL mint authority mismatch');
  if (mint.supply !== BigInt(pool.poolTokenSupply.toString())) throw new Error('Mint supply differs from pool state');

  const managerFee = await getAccount(connection, pool.managerFeeAccount, 'confirmed', LEGACY_TOKEN_PROGRAM);
  if (!managerFee.mint.equals(JITOSOL_MINT)) throw new Error('Manager fee account mint mismatch');

  const validatorRaw = await connection.getAccountInfo(pool.validatorList, 'confirmed');
  if (!validatorRaw) throw new Error('Validator list does not exist');
  if (!validatorRaw.owner.equals(STAKE_POOL_PROGRAM)) throw new Error('Validator list owner mismatch');
  const decodedEntries = decodeValidatorEntries(validatorRaw.data);
  const activeEntries = decodedEntries.filter((entry) => entry.status === 0 && entry.activeStakeLamports > 0n);
  const selectedCandidates = [...activeEntries]
    .sort((left, right) => left.activeStakeLamports > right.activeStakeLamports ? -1 : 1)
    .slice(0, 12);
  const candidateAddresses = selectedCandidates.map((entry) => deriveValidatorStake(entry.voteAccount, entry.validatorSeedSuffix));
  const candidateAccounts = await connection.getMultipleAccountsInfo(candidateAddresses, 'confirmed');
  const candidates: ValidatorEntry[] = selectedCandidates.map((entry, index) => ({
    ...entry,
    stakeAccount: candidateAddresses[index],
    stakeAccountLamports: candidateAccounts[index]?.lamports ?? null,
    stakeAccountOwner: candidateAccounts[index]?.owner.toBase58() ?? null,
  }));

  const totalLamports = BigInt(pool.totalLamports.toString());
  const supply = BigInt(pool.poolTokenSupply.toString());
  const withdrawalFeeNumerator = BigInt(pool.stakeWithdrawalFee.numerator.toString());
  const withdrawalFeeDenominator = BigInt(pool.stakeWithdrawalFee.denominator.toString());
  const stakeRent = await connection.getMinimumBalanceForRentExemption(StakeProgram.space, 'confirmed');
  // Independently verified in the report with `solana stake-minimum-delegation`.
  const minimumDelegation = 1_000_000_000n;
  const minimumPoolTokens = minimumPoolTokensForLamports(
    minimumDelegation,
    totalLamports,
    supply,
    withdrawalFeeNumerator,
    withdrawalFeeDenominator,
  );

  const reserveParsed = await connection.getParsedAccountInfo(pool.reserveStake, 'confirmed');
  const report = {
    inspectedAt: new Date().toISOString(),
    cluster,
    rpcEndpoint: endpoint,
    currentEpoch: epochInfo.epoch,
    poolLastUpdateEpoch: Number(pool.lastUpdateEpoch.toString()),
    poolIsCurrent: BigInt(pool.lastUpdateEpoch.toString()) === BigInt(epochInfo.epoch),
    addresses: {
      stakePool: JITO_STAKE_POOL.toBase58(),
      poolMint: pool.poolMint.toBase58(),
      stakePoolProgram: STAKE_POOL_PROGRAM.toBase58(),
      tokenProgram: pool.tokenProgramId.toBase58(),
      validatorList: pool.validatorList.toBase58(),
      reserveStake: pool.reserveStake.toBase58(),
      managerFeeAccount: pool.managerFeeAccount.toBase58(),
      stakePoolWithdrawAuthority: withdrawAuthority.toBase58(),
      stakeDepositAuthority: pool.stakeDepositAuthority.toBase58(),
      solDepositAuthority: pool.solDepositAuthority?.toBase58() ?? null,
      solWithdrawAuthority: pool.solWithdrawAuthority?.toBase58() ?? null,
      preferredDepositValidator: pool.preferredDepositValidatorVoteAddress?.toBase58() ?? null,
      preferredWithdrawValidator: pool.preferredWithdrawValidatorVoteAddress?.toBase58() ?? null,
    },
    decodedPool: {
      totalLamports: totalLamports.toString(),
      poolTokenSupply: supply.toString(),
      exchangeRateLamportsPerPoolToken: {
        numerator: totalLamports.toString(),
        denominator: supply.toString(),
        decimalApproximation: Number(totalLamports) / Number(supply),
      },
      epochFee: fee(pool.epochFee),
      stakeDepositFee: fee(pool.stakeDepositFee),
      stakeWithdrawalFee: fee(pool.stakeWithdrawalFee),
      solDepositFee: fee(pool.solDepositFee),
      solWithdrawalFee: fee(pool.solWithdrawalFee),
      stakeReferralPercent: pool.stakeReferralFee,
      solReferralPercent: pool.solReferralFee,
    },
    technicalMinimum: {
      minimumDelegationLamports: minimumDelegation.toString(),
      stakeAccountRentExemptionLamports: stakeRent.toString(),
      minimumPoolTokenInput: minimumPoolTokens.toString(),
      netStakeLamportsAtMinimumInput: withdrawalLamports(
        minimumPoolTokens,
        totalLamports,
        supply,
        withdrawalFeeNumerator,
        withdrawalFeeDenominator,
      ).toString(),
      operationalSolNeededIncludingRent: (minimumDelegation + BigInt(stakeRent)).toString(),
    },
    accountVerification: {
      stakePool: await accountMetadata(connection, JITO_STAKE_POOL),
      poolMint: await accountMetadata(connection, JITOSOL_MINT),
      stakePoolProgram: await accountMetadata(connection, STAKE_POOL_PROGRAM),
      validatorList: await accountMetadata(connection, pool.validatorList),
      reserveStake: await accountMetadata(connection, pool.reserveStake),
      managerFeeAccount: await accountMetadata(connection, pool.managerFeeAccount),
      mintAuthority: mint.mintAuthority?.toBase58() ?? null,
      mintSupply: mint.supply.toString(),
      mintDecimals: mint.decimals,
      managerFeeMint: managerFee.mint.toBase58(),
      managerFeeOwner: managerFee.owner.toBase58(),
      reserveParsed: reserveParsed.value,
    },
    validatorList: {
      decodedEntryCount: decodedEntries.length,
      activeEntryCount: activeEntries.length,
      currentEpochEntryCount: decodedEntries.filter(
        (entry) => entry.lastUpdateEpoch === BigInt(epochInfo.epoch),
      ).length,
      sampleCandidates: candidates.map((candidate) => ({
        ...candidate,
        activeStakeLamports: candidate.activeStakeLamports.toString(),
        transientStakeLamports: candidate.transientStakeLamports.toString(),
        lastUpdateEpoch: candidate.lastUpdateEpoch.toString(),
        transientSeedSuffix: candidate.transientSeedSuffix.toString(),
        voteAccount: candidate.voteAccount.toBase58(),
        stakeAccount: candidate.stakeAccount.toBase58(),
      })),
    },
  };
  const json = `${JSON.stringify(report, null, 2)}\n`;
  if (outputPath) {
    fs.mkdirSync(path.dirname(outputPath), { recursive: true, mode: 0o700 });
    fs.writeFileSync(outputPath, json, { mode: 0o600 });
  }
  process.stdout.write(json);
}

main().catch((error: unknown) => {
  process.stderr.write(`${error instanceof Error ? error.stack : String(error)}\n`);
  process.exitCode = 1;
});
import fs from 'node:fs';
import path from 'node:path';
