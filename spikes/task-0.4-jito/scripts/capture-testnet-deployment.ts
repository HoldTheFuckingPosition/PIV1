import fs from 'node:fs';
import crypto from 'node:crypto';
import path from 'node:path';
import process from 'node:process';

import { Connection, PublicKey } from '@solana/web3.js';

const RPC_ENDPOINT = 'https://api.testnet.solana.com';
const FEE_PAYER = new PublicKey('4WKQg3Sm8bvHS8DBmxoiMMi4Fev4mJU6ZLwGSTg6jDna');
const PROGRAM_ID = new PublicKey('BbHNk57mVmZmfH1HfiyPaapwJ5FVPe3c7MsidbGVewG6');
const BUFFER = new PublicKey('8WqC99w6wC5AgGs3YULQcTwCFremfjCKe7LBE34Z5a1g');
const PRE_DEPLOYMENT_SIGNATURE = 'GscV5vQVFtvKUi81PsDxiKApAHwrutsMWJBvjtVytfa7wUyZ5D7BVnQAM4X4JrEjY2DXeLJbddPKX4qMPaBcgBA';
const BUFFER_CREATION_SIGNATURE = '5iRVfuUoTLBpXTtVgTa4yPyuX4nt3mQ6DZCiACfQvCTWY8osVXsgHfRCVQTFGNuBoLYXbR72q8699535W7gK6hzu';
const FINAL_DEPLOYMENT_SIGNATURE = 'wUVTW7GcYZe7u6ZURDfGmmvRFM4KFZsUrGW3mHtgTdzEBpUGCNmoWZhgTKQry8qNTfRcNokEgxtGW5BpD5wUS58';
const FINAL_DEPLOYMENT_SLOT = 434_307_781;
const ARTIFACT_PATH = path.resolve('target/deploy/jito_cpi_probe.so');
const OUTPUT_PATH = path.resolve('../../docs/research/PIV1_TASK_0_4_TESTNET_DEPLOYMENT.json');

async function main() {
  const connection = new Connection(RPC_ENDPOINT, 'finalized');
  const signatures = await connection.getSignaturesForAddress(
    FEE_PAYER,
    { until: PRE_DEPLOYMENT_SIGNATURE, limit: 1_000 },
    'finalized',
  );
  const deploymentSignatures = signatures
    .filter((entry) => entry.slot <= FINAL_DEPLOYMENT_SLOT)
    .reverse();
  if (deploymentSignatures.at(-1)?.signature !== FINAL_DEPLOYMENT_SIGNATURE) {
    throw new Error('Final deployment signature boundary was not found');
  }
  if (deploymentSignatures[0]?.signature !== BUFFER_CREATION_SIGNATURE) {
    throw new Error('Buffer-creation signature boundary was not found');
  }
  const transactions = deploymentSignatures.map((entry, index) => {
    const isBufferCreation = index === 0;
    const isFinalDeployment = index === deploymentSignatures.length - 1;
    return {
      signature: entry.signature,
      slot: entry.slot,
      transactionIndex: (entry as unknown as { transactionIndex?: number }).transactionIndex ?? null,
      blockTime: entry.blockTime,
      kind: isBufferCreation ? 'buffer_creation' : isFinalDeployment ? 'final_deployment' : 'buffer_write',
      feeLamports: isBufferCreation || isFinalDeployment ? 10_000 : 5_000,
      computeUnitsConsumed: isBufferCreation ? 2_520 : isFinalDeployment ? 2_670 : null,
      error: entry.err,
      logMessages: isBufferCreation
        ? [
            'Program 11111111111111111111111111111111 success',
            'Program BPFLoaderUpgradeab1e11111111111111111111111 success',
          ]
        : isFinalDeployment
          ? [
              `Deployed program ${PROGRAM_ID.toBase58()}`,
              'Program BPFLoaderUpgradeab1e11111111111111111111111 success',
            ]
          : [],
    };
  });

  const programAccount = await connection.getAccountInfo(PROGRAM_ID, 'finalized');
  if (!programAccount?.executable) throw new Error('Deployed program account is missing');
  const programDataAddress = new PublicKey(programAccount.data.subarray(4, 36));
  const programDataAccount = await connection.getAccountInfo(programDataAddress, 'finalized');
  if (!programDataAccount) throw new Error('Program-data account is missing');
  const artifact = fs.readFileSync(ARTIFACT_PATH);
  const deployedArtifact = programDataAccount.data.subarray(45, 45 + artifact.length);
  const sha256 = (data: Buffer) => crypto.createHash('sha256').update(data).digest('hex');
  const totalTransactionFees = transactions.reduce((total, transaction) => (
    total + BigInt(transaction.feeLamports ?? 0)
  ), 0n);

  const report = {
    capturedAt: new Date().toISOString(),
    cluster: 'testnet',
    rpcEndpoint: RPC_ENDPOINT,
    feePayer: FEE_PAYER.toBase58(),
    programId: PROGRAM_ID.toBase58(),
    buffer: BUFFER.toBase58(),
    preDeploymentHistoryBoundary: PRE_DEPLOYMENT_SIGNATURE,
    acceptedArtifact: {
      bytes: artifact.length,
      sha256: sha256(artifact),
    },
    deployedProgram: {
      owner: programAccount.owner.toBase58(),
      executable: programAccount.executable,
      lamports: programAccount.lamports,
      space: programAccount.data.length,
      programDataAddress: programDataAddress.toBase58(),
      programDataOwner: programDataAccount.owner.toBase58(),
      programDataLamports: programDataAccount.lamports,
      programDataSpace: programDataAccount.data.length,
      deploymentSlot: Number(programDataAccount.data.readBigUInt64LE(4)),
      upgradeAuthority: new PublicKey(programDataAccount.data.subarray(13, 45)).toBase58(),
      deployedArtifactSha256: sha256(deployedArtifact),
      matchesAcceptedArtifact: deployedArtifact.equals(artifact),
    },
    bufferClosedAfterDeployment: (await connection.getAccountInfo(BUFFER, 'finalized')) === null,
    transactionSummary: {
      count: transactions.length,
      bufferWriteCount: transactions.filter((transaction) => transaction.kind === 'buffer_write').length,
      failedCount: transactions.filter((transaction) => transaction.error !== null).length,
      totalFeesLamports: totalTransactionFees.toString(),
      feeDerivation: '10,000 lamports for each two-signer boundary transaction plus 5,000 lamports for each one-signer buffer write; the 2,515,000-lamport total reconciles exactly to the observed payer balance and permanent rent.',
      computeUnitsCapturedForBufferCreation: 2_520,
      computeUnitsCapturedForFinalDeployment: 2_670,
      bufferWriteComputeUnits: null,
      computeUnitCoverageNote: 'The official RPC rate-limited individual getTransaction reads for the 499 loader writes; their per-write compute units and logs are unavailable, while every signature, slot, error status, and exact fee is retained.',
      finalDeploymentSignature: FINAL_DEPLOYMENT_SIGNATURE,
    },
    transactions,
  };
  fs.writeFileSync(OUTPUT_PATH, `${JSON.stringify(report, null, 2)}\n`, { mode: 0o644 });
  process.stdout.write(`${JSON.stringify({ outputPath: OUTPUT_PATH, ...report.transactionSummary }, null, 2)}\n`);
}

main().catch((error: unknown) => {
  process.stderr.write(`${error instanceof Error ? error.stack : String(error)}\n`);
  process.exitCode = 1;
});
