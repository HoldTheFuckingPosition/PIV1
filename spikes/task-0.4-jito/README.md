# Task 0.4 JitoSOL CPI probe

This is an isolated validation probe, not the production PIV1 program. It
demonstrates PDA-controlled direct SOL deposit, direct JitoSOL custody,
stake-pool withdrawal, deactivation, and native-SOL finalization against a
local validator containing cloned official Testnet programs and accounts.

The funded public Testnet lifecycle reached successful withdrawal and immediate
deactivation in epoch `1018`. The expected same-epoch finalization rejection is
recorded, and finalization must resume in epoch `1019` or later. Task 0.4 remains
`IN_PROGRESS — WAITING FOR EPOCH` until native SOL reaches the fixed escrow.
No Mainnet transaction was sent. Complete evidence and production conclusions
are in `../../docs/research/PIV1_TASK_0_4_JITO_VALIDATION.md`.

## Custody model

- `config`: probe configuration and monotonically increasing round counter.
- `authority`: signs JitoSOL token and Stake Program operations.
- `sol-vault`: empty-data, system-owned PDA that signs direct SOL deposit and
  pays reusable stake-account rent.
- `token-vault`: JitoSOL ATA whose owner is `authority`.
- `round + withdrawal-stake`: deterministic, unique PDAs derived from a round.
- `sol-escrow`: fixed empty-data, system-owned final SOL destination.

The caller is only a transaction fee payer. No instruction accepts a caller
chosen token vault, stake destination, authority, or final SOL destination.

## Pinned validation commands

Run repository, build, RPC, and keypair commands as `jerem` with the accepted
Task 0.3 toolchain:

```sh
export PATH=/home/jerem/.local/share/solana/install/active_release/bin:/home/jerem/.local/piv1-toolchains/node-v24.19.0-linux-x64/bin:/home/jerem/.cargo/bin:/usr/local/bin:/usr/bin:/bin
cd /home/jerem/piv1/spikes/task-0.4-jito

npm ci --offline
npm run check
npm run test:client
cargo +1.97.1 check --workspace --all-targets --locked --offline
cargo +1.97.1 test --workspace --all-targets --locked --offline
CARGO_NET_OFFLINE=true anchor build
```

Topology inspection writes public metadata to the task's external persistent
state directory. The Mainnet command is read-only:

```sh
npm run inspect:testnet
npm run inspect:devnet
npm run inspect:mainnet-readonly
```

## Reproducing the local cloned lifecycle

The scripts use the isolated test-only keys already stored outside Git in
`/home/jerem/.local/share/piv1/task-0.4-jito`. Never print or copy those files
into the repository.

Agave 4.2 creates a one-entry stake-history sysvar after a large hard warp.
The fixture generator copies the selected public Testnet validator stake and
changes only its local activation epoch from 772 to 0. That causes the real
Stake Program to use its normal "older than retained history" behavior; it
does not change any public-cluster account.

```sh
npx tsx scripts/make-local-validator-stake-fixture.ts \
  GTpapQCpq64AhLskXapChCApM7XiWAZ2GUjrTfbXRC4D \
  /home/jerem/.cache/piv1/task-0.4-jito/validator-stake.json

solana-test-validator --reset --quiet \
  --ledger /home/jerem/.cache/piv1/task-0.4-jito/local-ledger \
  --url testnet --clone-feature-set --warp-slot 432600000 \
  --clone-upgradeable-program SPoo1Ku8WFXoNDMHPsrGSTSG1Y47rzgn41SLUNakuHy \
  --clone Jito4APyf642JPZPx3hGc6WWJ8zPKtRbRs4P815Awbb \
  --clone G5N6K3qW86GSkNEpywcbJk42LjEZoshzECFg1LNVjSLa \
  --clone CzKqc9cs4XpyG6y4peQgk3vBjPyqhktmfqaMuMBCXCqm \
  --clone J1toso1uCk3RLmjorhTtrVwY9HJ7X8V9yYac6Y7kGCPn \
  --clone 8yoigZfzZ1nNaadumY9uPVD118225UYHTDpmjpr2nrSa \
  --account GTpapQCpq64AhLskXapChCApM7XiWAZ2GUjrTfbXRC4D \
    /home/jerem/.cache/piv1/task-0.4-jito/validator-stake.json \
  --bpf-program BbHNk57mVmZmfH1HfiyPaapwJ5FVPe3c7MsidbGVewG6 \
    target/deploy/jito_cpi_probe.so \
  --faucet-per-request-sol-cap 100 --faucet-per-time-sol-cap 1000
```

In another shell, run the lifecycle and pre-epoch adversarial tests:

```sh
npx tsx scripts/lifecycle.ts local-init
npm run test:local-adversarial
```

Stop the validator, preserve its ledger, then advance to epoch 1002 and prove
finalization plus stale-pool rejection:

```sh
solana-test-validator --quiet \
  --ledger /home/jerem/.cache/piv1/task-0.4-jito/local-ledger \
  --warp-slot 433000000

npx tsx scripts/lifecycle.ts finalize local 1
npm run test:local-stale
```

Local transaction signatures are meaningful only inside that preserved local
ledger. The scripts store full logs, fees, compute units, and balance deltas in
the external persistent state directory.

## Resume the public Testnet lifecycle

Do not repeat initialization, deposits, contribution, or withdrawal. In epoch
`1019` or later, first verify the recorded round-0 stake PDA is inactive, then
run only:

```sh
npx tsx scripts/lifecycle.ts finalize testnet 0
```

The external resume state is
`/home/jerem/.local/share/piv1/task-0.4-jito/testnet-lifecycle.json`.

## Deliberate omissions

This probe does not implement PIV1 economics, beneficiaries, distribution
snapshots, governance, guardian rewards, production pause behavior, migration,
or Mainnet deployment. It does not use Jupiter, a DEX, instant unstaking, or the
Jito stake-deposit interceptor.
