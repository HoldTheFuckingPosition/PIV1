import fs from 'node:fs';
import process from 'node:process';

import { clusterApiUrl, Connection, PublicKey } from '@solana/web3.js';

const accountAddress = process.argv[2];
const output = process.argv[3];

if (!accountAddress || !output) {
  throw new Error('Usage: make-local-validator-stake-fixture.ts <stake-account> <output.json>');
}

async function main() {
  const connection = new Connection(clusterApiUrl('testnet'), 'confirmed');
  const address = new PublicKey(accountAddress);
  const account = await connection.getAccountInfo(address, 'confirmed');
  if (!account) throw new Error(`Testnet account ${address.toBase58()} does not exist`);

// StakeStateV2::Stake is bincode-encoded as:
// variant (4) + Meta (120) + Delegation voter (32) + stake (8), followed by
// activation_epoch.  A test-validator hard warp retains only an epoch-0 stake
// history entry, although its Clock reports epoch 1001.  Marking this cloned
// validator stake active since epoch 0 selects the Stake Program's canonical
// "older than retained history" path and avoids using inconsistent history.
// No public-cluster account is changed; this fixture is local-validator only.
  const activationEpochOffset = 4 + 120 + 32 + 8;
  const originalActivationEpoch = account.data.readBigUInt64LE(activationEpochOffset);
  account.data.writeBigUInt64LE(0n, activationEpochOffset);

  const fixture = {
    pubkey: address.toBase58(),
    account: {
      lamports: account.lamports,
      data: [account.data.toString('base64'), 'base64'],
      owner: account.owner.toBase58(),
      executable: account.executable,
      // RPC exposes u64::MAX as an imprecise JavaScript number; local genesis
      // accepts zero and recreates the rent-exempt account deterministically.
      rentEpoch: 0,
      space: account.data.length,
    },
  };

  fs.writeFileSync(output, `${JSON.stringify(fixture)}\n`, { mode: 0o600 });
  process.stdout.write(JSON.stringify({
    sourceCluster: 'testnet',
    originalActivationEpoch: originalActivationEpoch.toString(),
    localActivationEpoch: '0',
  }));
}

main().catch((error: unknown) => {
  process.stderr.write(`${error instanceof Error ? error.stack : String(error)}\n`);
  process.exitCode = 1;
});
