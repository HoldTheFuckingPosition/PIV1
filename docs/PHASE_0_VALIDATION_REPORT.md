# PIV1 Phase 0 production architecture validation report

Date: 2026-08-30 UTC

Task: PIV1 Task 0.5

Branch: task/0.5-phase-0-report

Accepted Phase 0 baseline: 2774a7c8b463ae1da03100eca85037585a120ec4

## Status vocabulary and scope

This report uses the task-required evidence labels:

- CONFIRMED BY LIVE TEST: observed on public Solana Testnet or by current read-only public-cluster account inspection.
- CONFIRMED BY LOCAL TEST: exercised against the real cloned programs and accounts in the Task 0.4 local validator.
- CONFIRMED: explicit founder-approved production policy or architecture requirement; this label does not imply implementation or live-test evidence.
- PROVISIONAL: an implementation detail that still needs later production tests and does not replace confirmed policy.
- OPEN: a decision that cannot be settled from technical evidence alone.
- REJECTED: excluded from production V1.

This report records the founder-accepted Phase 0 production architecture. It is
not production code and not a professional independent audit. It does not
authorize Mainnet activity.

## 1. Executive result

**Result: CONFIRMED — the founder reviewed and accepted Phase 0. Task 0.5 is
complete, and the section 18 entry criteria are satisfied for the separately
bounded Task 1.1 scaffold. Task 1.1 has not started.**

The complete custody path was demonstrated on public Testnet:

SOL → JitoSOL → deterministic Stake Program-owned PDA → deactivation → native
SOL in a fixed PIV escrow.

The evidence proves the core protocol compatibility, PDA authority model,
permissionless fee-payer model, current official cluster topology, dynamic
minimum mechanism, rent recovery, fixed-destination finalization, and replay
protection. No confirmed PIV1 requirement is technically contradicted.

The founder confirmed the 1-bps slippage policy, permissionless validator
policy, corrected custody topology, six logical transaction boundaries,
cooldown-reward treatment, exact KIF period/repeated zero-active carry, and
active-guardian division-remainder carry. Production V1 also requires one
active distribution to scale across multiple deterministic validator legs.

The public Testnet lifecycle remains valid evidence for each individual leg:
one protected SPL withdrawal used one source and one deterministic destination,
was immediately deactivated, then finalized to fixed escrow. Multi-leg target
assignment, cumulative accounting, partial resumption, and complete-before-
settlement orchestration are CONFIRMED architecture derived from the pinned SPL
interface; they have not been compiled or live-tested.

Production implementation must not copy the Task 0.4 probe. The probe omitted
the economic split, high-water mark, pending queues, guardians, recipients,
pause, governance, and full production state machine.

## 2. Accepted evidence baseline

### 2.1 Repository and branch history

The task began from a clean repository on
spike/task-0.4-jito-validation at exactly:

- 2774a7c8b463ae1da03100eca85037585a120ec4 — accepted Task 0.4 completion.
- be5496bf7d6858e1208f274b4dad226bbf097455 — funded Testnet withdrawal resume.
- 41791750561f6aea1405b843f532ecca04e33b04 — project identity/publication readiness.
- 04105ce11c4a09875120c9d7df688bc0cf00c950 — Testnet funding retry record.
- a57b67fd7ca04f87ad34f21bad1ce4ec01c72b2b — local JitoSOL CPI lifecycle validation.
- fa18ed0797a55c701ef8daf7ee39ffc0aa86ff48 — initial repository baseline.

Task 0.5 was branched without rewriting history from the accepted completion
commit as task/0.5-phase-0-report. Repository integrity passed git fsck and the
repository-local identity remained:

HTFP Project <HoldTheFuckingPosition1@protonmail.com>

### 2.2 Authoritative repository evidence

- [Decision register](./PIV1_DECISIONS.md)
- [Master specification](./PIV1_MASTER_SPEC.md)
- [Execution plan](./PIV1_CODEX_EXECUTION_PLAN.md)
- [Task 0.4 validation report](./research/PIV1_TASK_0_4_JITO_VALIDATION.md)
- [Task 0.4 deployment/finalization evidence](./research/PIV1_TASK_0_4_TESTNET_DEPLOYMENT.json)
- [Experimental probe handoff](../spikes/task-0.4-jito/README.md)

The deployment JSON contains 501 successful loader transactions, exact fees,
the accepted artifact hash, deployed-program metadata, and the final round-0
reconciliation. The 499 loader writes have signatures, slots, status, and fees;
the official RPC did not provide their individual logs/CU values after rate
limiting, and this report does not invent them.

### 2.3 Official primary sources

The source identities used for architecture conclusions are:

| Source | Identity used | Purpose |
| --- | --- | --- |
| [Jito stake/unstake reference](https://github.com/jito-foundation/jito-stake-unstake-reference/tree/b553e90d39e1ff583011dab344a11b5d9bfd284c) | b553e90d39e1ff583011dab344a11b5d9bfd284c, still upstream master/HEAD at inspection | Mainnet/Testnet support and direct deposit/delayed withdrawal reference |
| [Jito deployed-program documentation](https://github.com/jito-foundation/jito-omnidocs/blob/0bd6a39d1edfd906ddcc33ac2cbdc09d7eaa9595/jitosol/jitosol-liquid-staking/security/deployed-programs/index.md) | 0bd6a39d1edfd906ddcc33ac2cbdc09d7eaa9595, upstream master/HEAD at inspection | Official cluster program/pool/mint declarations |
| [Jito Preferred Withdraw Validator List API](https://www.jito.network/docs/jitosol/jitosol-liquid-staking/for-developers/stake-pool-api/#10-preferred-withdraw-validator-list) | current official documentation inspected 2026-08-30 | Operational candidate recommendations, capacity fields, and rebalancing rationale |
| [Jito StakeNet](https://github.com/jito-foundation/stakenet) | current official repository inspected 2026-08-30 | Steward/validator-management context; not an oracle inside PIV1 |
| [SPL Stake Pool program v2.0.3](https://github.com/solana-program/stake-pool/tree/864ba3c1c564cc270ca62b6e6b558f57538ae092/program) | crate VCS commit 864ba3c1c564cc270ca62b6e6b558f57538ae092; tag program@v2.0.3 | Exact pinned on-chain interfaces, math, validation, and list layout |
| [Agave v4.2.0](https://github.com/anza-xyz/agave/tree/ac82b5d438b0c2303dc7169f52c748977713a111) | ac82b5d438b0c2303dc7169f52c748977713a111 | Accepted runtime/CLI identity and stake behavior |
| [Solana token-account and ATA derivation documentation](https://solana.com/docs/tokens/basics/create-token-account#what-is-an-associated-token-account) | current official documentation | Token-account program owner versus token authority, ATA seeds, and one ATA address per wallet/token-program/mint combination |
| [Solana stake accounts](https://solana.com/docs/references/staking/stake-accounts) | current official documentation | Authorities, cooldown, withdrawal, and account closure |
| [Solana compute budget](https://solana.com/docs/core/fees/compute-budget) | current official documentation | Compute ceilings and safety-margin guidance |
| [Solana CPI reference](https://solana.com/docs/core/cpi) | current official documentation | PDA signing, privilege propagation, and CPI depth |
| [Solana clusters](https://solana.com/docs/references/clusters) | current official documentation | Official public RPC endpoints and cluster separation |

Task 0.4 also inspected Jito omnidocs commit
14df8bb2f7169328d984393b0090b8cc32863e45 and a stake-pool checkout at
5b8de048f2e6cdf2c9b75387300421abe9ec7704. The exact production dependency is
the published spl-stake-pool 2.0.3 crate whose embedded VCS identity is
864ba3c1c564cc270ca62b6e6b558f57538ae092.

### 2.4 Public Testnet program and transaction evidence

Probe program:
[BbHNk57mVmZmfH1HfiyPaapwJ5FVPe3c7MsidbGVewG6](https://explorer.solana.com/address/BbHNk57mVmZmfH1HfiyPaapwJ5FVPe3c7MsidbGVewG6?cluster=testnet)

| Stage | Public evidence | Result |
| --- | --- | --- |
| Deploy accepted artifact | [wUVT…US58](https://explorer.solana.com/tx/wUVTW7GcYZe7u6ZURDfGmmvRFM4KFZsUrGW3mHtgTdzEBpUGCNmoWZhgTKQry8qNTfRcNokEgxtGW5BpD5wUS58?cluster=testnet) | success |
| Direct-client deposit | [Amvd…AgS4](https://explorer.solana.com/tx/Amvd6NaE3e9CUoFScptWSH6iHZQpFrseFkCNk1ZNWK4YiawjPSei4QP7fuAYjdt6E6zrQd8vMYPSwpdYmBkAgS4?cluster=testnet) | success |
| Initialize probe | [3WYE…dBEu](https://explorer.solana.com/tx/3WYEEBGP1H92iVYXNmvGxN28W4GwB9Y8t88nehxQeXXtPefrMzAyztNEHbe7tfD7YXc1iD7fE6GuyDbHB81sdBEu?cluster=testnet) | success |
| SOL-to-JitoSOL CPI | [4FUS…D46J](https://explorer.solana.com/tx/4FUS4JLJdo6WBaS1sDLVEhfiEvmiaeH99TFqjGyzRzqupPV2psT7vGA3epsK8hLfysfwU654PoEDbEWHqzetD46J?cluster=testnet) | success |
| Direct JitoSOL contribution | [61xk…QoeK](https://explorer.solana.com/tx/61xkvvU1grZnWRrVGGJ3xtEuuRbfrBChbaGBEpKV56YFBQfnst3HwHX73GELnFWGnxhAWKWroxyrEk8c2WCXQoeK?cluster=testnet) | success |
| Withdraw stake + deactivate | [3ScM…ve5F](https://explorer.solana.com/tx/3ScMY9GmtN3KCMoTbWc8LFQLhtsUhAX8kVtymBJB11VArsGkMbrxJbZgvWNrQod1P7VAdh5fgwBkffjm7xmeve5F?cluster=testnet) | success |
| Premature finalization | [5WLp…LHBLZ](https://explorer.solana.com/tx/5WLpPa6wQqG2XN1R18MSFBVoJNdgNxGWNdf7Gex1MPuzAseU1PU1jkWML7qiLxtwHUUz78bj58iuxa8xkPMLHBLZ?cluster=testnet) | expected error 6024 |
| Finalize round 0 | [2pfo…NnY2t](https://explorer.solana.com/tx/2pfoSHp1SXuTq5XSheTyrws2Ewb5CQN8VNDCJN6v2rsCvyaHvxh94ynw6WuoWBmBRJDxEHaVSGy9k7XvYyxNnY2t?cluster=testnet) | success |

The replay was signed and simulated but not broadcast. It failed with
WrongRoundStatus / 6026.

## 3. Verified toolchain

The accepted coherent stack was re-read from the installed tools and locked
manifests; the full compatibility build was not repeated because no
contradiction was discovered.

| Component | Accepted version | Verification |
| --- | --- | --- |
| Node.js | 24.19.0 | node --version |
| npm | 11.17.0 | npm --version |
| rustup | 1.29.0 | rustup --version |
| host Rust/Cargo | 1.97.1 | rustc/cargo +1.97.1 |
| Anchor CLI/framework | 0.32.1 | anchor --version and lockfile |
| Agave/Solana CLI | 4.2.0 | solana-cli source ac82b5d4 |
| cargo-build-sbf | 4.1.0 | cargo-build-sbf --version |
| SBF platform-tools | v1.54 | cargo-build-sbf --version |
| SBF Rust | 1.89.0 | platform-tools output |
| spl-stake-pool | 2.0.3 | locked Cargo dependency/tree |
| TypeScript | 5.9.3 | npx tsc --version |

CONFIRMED Phase 1 baseline rule: preserve these exact pins through initial Phase 1
scaffolding. Any upgrade must be a separately justified dependency task with
host/SBF compatibility, lockfile, source identity, and regression evidence.

## 4. Official Jito topology

### 4.1 Current read-only cluster inspection

Read-only inspection used official public RPC at approximately
2026-08-29T21:13Z. These are point-in-time facts; production must decode and
validate dynamic pool data at execution time.

| Relationship | Testnet | Mainnet |
| --- | --- | --- |
| Stake-pool program | SPoo1Ku8WFXoNDMHPsrGSTSG1Y47rzgn41SLUNakuHy | same |
| Jito stake pool | Jito4APyf642JPZPx3hGc6WWJ8zPKtRbRs4P815Awbb | same |
| JitoSOL mint | J1toso1uCk3RLmjorhTtrVwY9HJ7X8V9yYac6Y7kGCPn | same |
| Token program | TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA | same |
| Withdraw-authority PDA | 6iQKfEyhr3bZMotVkW6beNZz5CPAkiwvgV2CTje9pVSS | same |
| Validator list | G5N6K3qW86GSkNEpywcbJk42LjEZoshzECFg1LNVjSLa | 3R3nGZpQs2aZo5FDQvd2MUQ6R7KhAPainds6uT6uE2mn |
| Reserve stake | CzKqc9cs4XpyG6y4peQgk3vBjPyqhktmfqaMuMBCXCqm | BgKUXdS29YcHCFrPm5M8oLHiTzZaMDjsebggjoaQ6KFL |
| Stake-deposit authority | 74opVa3v51hUmTrsZn8YusZw4fXB16vGQY4WYHt9UegR | 6hg6RMfjn3QSW6MdjH2Dg3dhF9XsVEE8CQXhjYwkRCiK |
| SOL deposit authority | none | none |
| SOL withdrawal authority | none | none |
| Manager fee account | 8yoigZfzZ1nNaadumY9uPVD118225UYHTDpmjpr2nrSa | same |
| Preferred deposit/withdraw validator | none / none | none / none |
| Current epoch / pool epoch | 1021 / 1021 | 1024 / 1024 |
| Total pool lamports | 4,374,396,068,805 | 10,055,451,327,284,113 |
| Pool-token supply | 3,294,875,623,041 | 7,744,307,943,019,095 |
| Approx. SOL/JitoSOL | 1.3276361748573862 | 1.2984312350787055 |
| Decoded real / active validators | 1,129 / 699 | 687 / 687 |
| Entries current for epoch | 1,129 | 687 |
| Runtime minimum delegation | 1,000,000,000 lamports | same |
| Current 200-byte stake rent | 2,077,224 lamports | 2,282,880 lamports |
| Current minimum JitoSOL input | 753,972,377 units | 770,931,087 units |
| Net delegated output at minimum | 1,000,000,000 lamports | 1,000,000,001 lamports |

Ownership/relationship checks:

- Pool and validator-list accounts are owned by the executable SPL stake-pool
  program.
- The mint and manager fee account are owned by the legacy Token program and
  use the JitoSOL mint.
- Mint authority equals the stake-pool withdraw-authority PDA; mint supply
  equals pool state supply.
- The withdraw authority derives under the stake-pool program from
  [stake_pool_address, "withdraw"] and its bump matches pool state.
- Both reserve stakes are 200-byte Stake Program accounts. Their staker and
  withdrawer are the pool withdraw authority.
- The manager fee token-account owner is
  GSyXx6WRm2o6Qu4RWxTH17swLZKpTKQdQTS2uGcus1NF on both inspected clusters.
- Current fees on both clusters were: epoch 4/100, stake deposit zero,
  stake withdrawal 1/1000, SOL deposit zero, SOL withdrawal 1/1000, and both
  referral percentages zero. These are observations, not constants.

The Testnet stake-rent value changed from the 2,282,880 lamports funded and
recovered by Task 0.4 to 2,077,224 at this inspection. That is direct evidence
that rent and the technical minimum must be runtime-derived.

### 4.2 Material topology differences and cluster correction

Testnet and Mainnet share the pool, mint, stake-pool program, token program,
manager fee account, and withdraw PDA, but not the validator list, reserve,
stake-deposit authority, pool balances, validator population, exchange rate, or
current rent. Cluster genesis/RPC selection and decoded pool bindings must be
validated; equal-looking addresses are not permission to reuse other
cluster-specific accounts.

The historical execution-plan direction to use Devnet for Jito integration is
HISTORICAL. Task 0.4 validated the official non-Mainnet PIV path on Testnet, and
the accepted current decision register/master spec already direct integration
testing to Testnet. Current Jito documentation also lists a distinct newer
Devnet program/pool/mint; those addresses are not interchangeable with the
Testnet/Mainnet topology and were not validated by Task 0.4. This report does
not change PIV1's accepted Testnet target.

The Jito stake-deposit interceptor is for deposits of existing stake accounts.
It is REJECTED for PIV1 native-SOL deposits. PIV1 uses the ordinary SPL
DepositSolWithSlippage path.

## 5. Production custody architecture

### 5.1 Confirmed account model

The following custody roles and deterministic relationships are CONFIRMED.
Names and seed bytes are architectural identifiers, not deployed addresses; no
production Program ID or recipient address is invented here. Phase 1 must give
every program-owned schema an explicit bounded Anchor account size.

| Account | Owner / derivation | Role and invariant |
| --- | --- | --- |
| PivConfig | PIV1 PDA ["config"] | Version, pause, official Jito bindings, recipients, split constants, 0–1-bps configured slippage, timing, HWM, sequence, contribution ledgers, 2,592,000-second KIF configuration/anchor, and accounting totals |
| PivAuthority | Address-only PIV1 PDA ["authority"] | Shared token authority recorded in both JitoSOL vaults and stake staker/withdrawer; not either vault's account address or program owner; signs by invoke_signed only |
| ActiveDistribution | PIV1 PDA ["distribution"] | One reusable bounded active-round header with monotonic sequence and cumulative leg counters; contains no unbounded leg vector |
| PendingSolVault | Empty-data System-owned PDA ["pending-sol"] | Native contributions not reconciled; usable first for outgoing allocation; rent floor excluded |
| PrincipalSolQueue | Empty-data System-owned PDA ["principal-sol"] | Reconciled principal SOL waiting for direct Jito deposit; cannot fund beneficiaries except through fixed round accounting |
| OperationalSolVault | Empty-data System-owned PDA ["operational-sol"] | Only permanent operational/rent reserve; advances each temporary leg-metadata and withdrawal-stake rent; excluded from economics |
| DistributionEscrow | Empty-data System-owned PDA ["distribution-escrow"] | Fixed native-SOL destination and source for the active round; its rent floor is excluded |
| KifSolVault | Empty-data System-owned PDA ["kif-sol"] | Backs aggregate guardian claim liabilities; never mixed with principal or pending contributions |
| PrincipalJitoVault | Account address: PIV1 PDA ["principal-jito-vault"]; program owner: legacy SPL Token Program; decoded token authority: PivAuthority | Reconciled principal JitoSOL in its own 165-byte token account |
| PendingJitoVault | Account address: distinct PIV1 PDA ["pending-jito-vault"]; program owner: legacy SPL Token Program; decoded token authority: PivAuthority | Unreconciled JitoSOL contributions; separate token-account balance and ledger |
| WithdrawalLeg | Temporary PIV1 PDA ["withdrawal-leg", sequence_le_u64, leg_index] | Bounded metadata for one successful source, its exact per-leg inputs/outputs/rent/slippage, status, and replay protection |
| WithdrawalStake | Stake Program-owned PDA ["withdrawal-stake", sequence_le_u64, leg_index] | Unique temporary stake for one SPL withdrawal leg; PivAuthority is staker and withdrawer |
| GuardianRegistry / rewards | PIV1 PDA(s) | Six keys, activity, carry, claimable/cumulative amounts; bounded fixed set |
| HTFP and Team recipients | Pubkeys stored in config | Fixed writable native-SOL destinations; must be real, non-default, governance-approved addresses |

Confirmed corrected token-vault topology: use two different token-account
addresses derived under the PIV1 program, one for each economic category. The
[official Solana derivation](https://solana.com/docs/tokens/basics/create-token-account#what-is-an-associated-token-account)
uses the wallet/token authority, token program, and mint as ATA seeds and yields
exactly one ATA for that combination. PrincipalJitoVault and PendingJitoVault
therefore cannot both be ATAs for the same PivAuthority, legacy Token program,
and JitoSOL mint.

The account-address PDA and token-authority PDA are separate concepts:

~~~text
PrincipalJitoVault address = PIV1 PDA ["principal-jito-vault"]
PendingJitoVault address   = PIV1 PDA ["pending-jito-vault"]
shared decoded token authority = PivAuthority PDA ["authority"]
program owner of each initialized account = legacy SPL Token Program
mint stored in each initialized account = JitoSOL
~~~

Neither vault is an ATA. Each vault address must be rederived with its own seed
and bump under PIV1, while each initialized token account must decode to the
legacy Token program, JitoSOL mint, and PivAuthority token authority. The
stake-pool protocol requires these bindings, not an ATA. The founder accepted
this topology. Exact serialized fields, bumps, account sizes, and initialization
funding mechanics remain Phase 1 implementation work, not open custody policy.

The reusable `ActiveDistribution` stores the current sequence, fixed total
JitoSOL target, cumulative assigned input, pool-token fees and burn, expected /
delegated native output, finalized SOL, recovered rent, cooldown rewards or
losses, next leg index, successful/finalized counts, target-assigned and
all-finalized flags, fixed gross beneficiary obligations, the snapshot
technical leg-input floor and maximum useful-leg bound, the conservative stored
slippage floor, HWM proof values, and terminal summary. It stores no leg vector.
`Config.next_sequence` increments exactly once when a valid snapshot opens.
Each temporary leg is discoverable from `(sequence, leg_index)` and may close
after its exact amounts have been rolled into checked header counters.

### 5.2 Separation and direct-transfer reconciliation

Physical separation is necessary but not sufficient because any known system or
token account may receive direct transfers:

- Explicit SOL contributions enter PendingSolVault. After reconciliation,
  unused principal SOL moves to PrincipalSolQueue and is later deposited
  directly to Jito.
- Explicit JitoSOL contributions enter PendingJitoVault.
- A positive unaccounted balance delta at any economic vault is classified as a
  pending contribution, never historical yield.
- Accounted token units, not the raw PrincipalJitoVault balance, define the
  historical position. This prevents a direct transfer to that token account
  from becoming yield.
- Contributions arriving after round preparation remain pending and cannot
  change its fixed eligibility or obligation.
- Vault/escrow rent floors, operational reserve, active liabilities, and
  transaction fees are excluded from principal and yield.

### 5.3 Custody conclusions

- CONFIRMED BY LIVE TEST: an empty-data System-owned PDA can sign native SOL
  operations and a deterministic Stake Program-owned PDA can receive the split.
- CONFIRMED BY LIVE TEST: PivAuthority can be both stake staker/withdrawer and
  JitoSOL transfer authority; the permissionless caller receives no custody.
- CONFIRMED BY LIVE TEST: the stake account can close into a fixed PIV escrow,
  and replay is rejected by both terminal state and the closed stake PDA.
- CONFIRMED: the exact custody roles, distinct non-ATA token-vault topology,
  reusable round header, and per-leg temporary-account model are founder-
  approved. Their production schemas and multi-leg orchestration remain to be
  implemented and tested in later phases.

## 6. Transaction and account diagrams

### 6.1 Direct SOL contribution and JitoSOL minting

~~~mermaid
sequenceDiagram
    participant C as Contributor
    participant PS as Pending SOL PDA
    participant P as PIV1
    participant QS as Principal SOL PDA
    participant J as Jito stake pool
    participant PJ as Principal JitoSOL vault PDA
    C->>PS: Tx A: irreversible SOL contribution
    P->>P: reconcile as pending contribution
    P->>QS: classify unused SOL as principal
    QS->>J: Tx B: DepositSolWithSlippage CPI
    J->>PJ: mint actual JitoSOL output
    Note over C,PJ: No DEX, caller pays both transaction fees
~~~

### 6.2 Direct JitoSOL contribution

~~~mermaid
sequenceDiagram
    participant C as Contributor
    participant P as PIV1
    participant PJ as Pending JitoSOL vault PDA
    C->>P: deposit_jitosol(amount)
    P->>PJ: Token TransferChecked CPI
    P->>P: record actual balance delta as pending
    Note over C,PJ: Address is a PIV1 PDA, token authority is PivAuthority, Token Program owns account
~~~

### 6.3 Distribution preparation and shortfall

~~~mermaid
flowchart TD
    A[Idle and 10 days elapsed] --> B[Validate current pool/list and ledgers]
    B --> C{Historical value above HWM?}
    C -- No --> D[No snapshot and remain Idle]
    C -- Yes --> E[Floor 59 / 19.5 / 19.5 / 2 split]
    E --> F[Use snapshotted pending SOL first]
    F --> G{Native shortfall?}
    G -- No --> H[Move fixed SOL to escrow and use liquid EscrowFunded path]
    G -- Yes --> I[Compute gross-budget JitoSOL and dynamic minimum]
    I --> J{Technically feasible?}
    J -- No --> K[Only 24h insufficient timestamp and no snapshot]
    J -- Yes --> L[Snapshot fixed target, obligation, floors, and unique sequence]
~~~

### 6.4 Multi-validator target assignment

~~~mermaid
sequenceDiagram
    participant K as Permissionless caller
    participant P as PIV1
    participant O as Operational SOL PDA
    participant L as Withdrawal leg PDA
    participant S as Withdrawal stake PDA
    participant J as SPL/Jito pool
    participant A as PIV authority PDA
    loop Until cumulative input equals fixed target
        K->>P: initiate_withdrawal_leg(sequence, index, candidate)
        P->>P: validate candidate and maximum safe capacity
        P->>P: input = min(remaining target, maximum safe capacity)
        P->>O: advance exact leg and stake rents
        O->>L: create bounded metadata PDA
        O->>S: create Stake Program-owned PDA
        P->>J: WithdrawStakeWithSlippage CPI
        J->>S: split one delegated stake leg
        A->>S: fixed staker and withdrawer
        P->>S: immediate Deactivate CPI
        P->>P: exact deltas and checked cumulative counters
    end
    Note over K,A: Failure creates no leg and consumes no index or target
~~~

### 6.5 Independent leg cooldowns

~~~mermaid
sequenceDiagram
    participant P as PIV1
    participant S1 as Withdrawal stake leg i
    participant S2 as Withdrawal stake leg j
    participant ST as Stake Program
    participant C as Clock and Stake History
    P->>ST: each successful leg already deactivated
    ST->>S1: deactivation epoch recorded
    ST->>S2: possibly different deactivation epoch
    C-->>P: leg i inactive, leg j still deactivating
    P->>P: finalize ready leg i and preserve leg j
    C-->>P: leg j later becomes inactive
~~~

### 6.6 Permissionless per-leg finalization into one escrow

~~~mermaid
sequenceDiagram
    participant K as Permissionless caller
    participant P as PIV1
    participant L as Withdrawal leg PDA
    participant S as Withdrawal stake PDA
    participant ST as Stake Program
    participant E as Fixed SOL escrow
    participant O as Operational reserve
    K->>P: finalize_withdrawal_leg(sequence, index)
    P->>P: validate exact leg, stake, authorities, inactivity
    P->>ST: Withdraw entire stake balance CPI
    ST->>E: delegated SOL + rewards + recovered rent
    P->>P: reconcile exact per-leg values and close stake
    E->>O: return recorded stake rent
    L->>O: close metadata and return its rent
    P->>P: increment finalized count exactly once
~~~

### 6.7 Settlement and compounding

~~~mermaid
sequenceDiagram
    participant P as PIV1
    participant E as Distribution escrow
    participant H as HTFP recipient
    participant T as Team recipient
    participant K as KIF vault and ledgers
    P->>P: require exact target and all successful legs finalized
    P->>P: require escrow and cumulative accounting reconciliation
    P->>P: derive net outgoing amount and atomic allocations
    E->>H: fixed HTFP native SOL
    E->>T: fixed Team native SOL
    E->>K: fund KIF liabilities
    P->>P: atomically commit compound and HWM accounting
    Note over P,K: No beneficiary receives a partial multi-leg distribution
~~~

### 6.8 Pending-contribution integration

~~~mermaid
flowchart LR
    PS[Pending SOL ledger/vault] --> R[Atomic reconciliation]
    PJ[Pending JitoSOL ledger/PDA-addressed token account] --> R
    R --> H[Increase HWM by conservative contribution value]
    R --> QS[Principal SOL queue]
    R --> QJ[Principal JitoSOL PDA-addressed token account/ledger]
    QS -->|later permissionless direct deposit| J[Jito pool]
    J --> QJ
    N[Contributions after reconciliation lock] --> PS
    N --> PJ
~~~

## 7. Exact instruction variants and CPI boundaries

### 7.1 Required stake-pool variants

Production must use:

- DepositSolWithSlippage(lamports_in, minimum_pool_tokens_out)
- WithdrawStakeWithSlippage(pool_tokens_in, minimum_lamports_out)

CONFIRMED BY LOCAL TEST: the protected and basic variants use the same account
metas; protected variants additionally encode an output floor. Both protected
variants succeeded through PIV PDA CPIs. Basic DepositSol and WithdrawStake are
REJECTED for production because they permit execution without a transaction-time
minimum output.

### 7.2 CPI account matrix

In the tables below, caller-provided means present in the transaction, not
trusted. Every key is rederived, matched to config/pool state, decoded, and
owner-checked before CPI.

#### DepositSolWithSlippage

Ordered SPL metas:

1. stake pool — writable;
2. pool withdraw authority — read-only;
3. reserve stake — writable;
4. PrincipalSolQueue funding account — writable signer;
5. PrincipalJitoVault destination — writable;
6. manager fee token account — writable;
7. fixed referrer token account — writable;
8. JitoSOL mint — writable;
9. System Program — read-only;
10. legacy Token program — read-only;
11. optional SOL deposit authority — read-only signer if pool configures one.

PDA signer seeds: ["principal-sol", bump]. The referrer must be fixed in config;
until separate referral economics are approved, the manager fee account is the
safe non-caller-controlled alias. Validate executable stake-pool program; exact
pool; current last_update_epoch; pool reserve, mint, manager fee, token program,
withdraw bump/PDA, optional authority; System/Token program IDs; destination
address equals the PrincipalJitoVault PDA derived under PIV1; destination account
is program-owned by the legacy Token program; decoded destination mint is
JitoSOL and decoded token authority is PivAuthority; input is not reserved for
liabilities; and expected output and post-CPI balance deltas match.

External programs: SPL Stake Pool, which invokes System and Token programs.

#### WithdrawStakeWithSlippage

Ordered SPL metas:

1. stake pool — writable;
2. validator list — writable;
3. pool withdraw authority — read-only;
4. selected validator stake source — writable;
5. deterministic withdrawal stake PDA for `(sequence, leg_index)` — writable;
6. PivAuthority as new stake authority — read-only;
7. PivAuthority as pool-token transfer authority — read-only signer;
8. PrincipalJitoVault — writable;
9. manager fee token account — writable;
10. JitoSOL mint — writable;
11. Clock sysvar — read-only;
12. legacy Token program — read-only;
13. Stake Program — read-only.

PDA signer seeds: ["authority", bump]. Caller supplies the candidate index,
vote, and account views plus fixed external accounts. Validate all pool bindings,
current pool/list, direct validator-list entry, Active status, vote and seed
suffix, derived standard validator stake PDA, Stake Program owner/delegation,
sufficient maximum-safe withdrawable capacity, preferred-validator/source-order
rule, remaining fixed target, program-derived maximum-fill leg token amount and
per-leg slippage floor, PrincipalJitoVault account-address PDA, legacy Token program
owner, decoded PivAuthority token authority and JitoSOL mint, unique stake PDA,
metadata PDA, and residual-HWM invariant. The caller cannot choose the input
amount, authority, token vault, fee account, or escrow.

External programs: SPL Stake Pool, which invokes Stake and Token programs.

#### System CreateAccount for withdrawal stake

Metas for the stake account: OperationalSolVault writable signer/payer;
deterministic WithdrawalStake writable signer/new account; System Program. New
owner is Stake Program and data length is StakeStateV2::size_of() (currently
200). Signer seeds include ["operational-sol", bump] and
["withdrawal-stake", sequence_le_u64, leg_index, bump]. A separate create/init
for bounded WithdrawalLeg metadata uses
["withdrawal-leg", sequence_le_u64, leg_index, bump] and PIV1 ownership.
Validate zero lamports/empty data before creation, exact sequence/index, current
Rent requirements for both accounts, operational category balance, and no
reused or previously recorded address.

#### Stake Deactivate

Metas: WithdrawalStake writable; Clock read-only; PivAuthority signer; Stake
Program. Signer seeds: ["authority", bump]. Validate stake owner, exact
round/index metadata binding, both stake authorities, voter/source record, and
leg/round status. Confirmed boundary: same transaction as successful
WithdrawStakeWithSlippage; retain a separate permissionless fallback only if
later production remeasurement proves it technically necessary.

#### Stake Withdraw to fixed escrow

Metas: WithdrawalLeg and WithdrawalStake writable; ActiveDistribution writable;
DistributionEscrow writable; Clock and Stake History read-only; PivAuthority
signer; Stake Program. Signer seeds: ["authority", bump]. Validate exact
sequence/index/PDAs/status/authorities, fully inactive effective stake, fixed
escrow, whole current balance, pre/post deltas, stake closure, per-leg
reward/loss and rent reconciliation, cumulative checked updates, and single
finalization. Recover stake and metadata rent only to OperationalSolVault.

#### Token TransferChecked for direct JitoSOL contribution

Metas: contributor source writable; JitoSOL mint read-only; PendingJitoVault
writable; contributor token authority signer; legacy Token program. No PIV PDA
signer is needed. Validate source program owner/mint/token authority; destination
address equals the PendingJitoVault PDA derived under PIV1 and is distinct from
PrincipalJitoVault; destination account is program-owned by the legacy Token
program; decoded destination mint is JitoSOL and decoded token authority is
PivAuthority; mint decimals are 9 from the decoded mint; amount is greater than
zero; and the destination delta is exact. The contributor source/authority are
the only economically caller-selected accounts.

#### System transfers

System transfers cover SOL contribution into PendingSolVault, snapshotted
pending SOL into DistributionEscrow, rent recovery from escrow to
OperationalSolVault, beneficiary transfers, KIF-vault funding, and KIF claims.
Every PIV source is a fixed System-owned PDA signing with its own seeds.
Recipients must equal config or the relevant guardian payout binding.
Post-transfer balances must preserve rent floors, recorded liabilities,
principal, and operational separation. Contributor-source transfers require the
contributor signer; permissionless keepers never sign for a contributor.

#### Initialization-only account creation

Config, the reusable distribution account, permanent System vaults, guardian
accounts, and both PDA-addressed legacy-token accounts should be initialized
before accepting funds. The Associated Token Program is not used for either
JitoSOL vault.

For each token-vault initialization, the account set must include:

1. creation payer/funding source — writable, with its approved transaction or
   PDA signer authority;
2. the respective PrincipalJitoVault or PendingJitoVault address PDA — writable
   new account and signer only inside the System CPI through invoke_signed;
3. JitoSOL mint — read-only;
4. PivAuthority — read-only token authority;
5. System Program — read-only; and
6. legacy Token program — read-only.

A System CreateAccount CPI creates and rent-funds the distinct 165-byte vault
address, with that address PDA signing by invoke_signed and the legacy Token
program assigned as program owner. A legacy Token initialization CPI then
records JitoSOL as mint and PivAuthority as the decoded token authority.
Initialization must rederive the relevant vault seed and bump and prove that
the two vault addresses differ. The topology and roles are CONFIRMED. Exact
creation payer mechanics, initialization variant, serialized layouts, and
account sizes remain Phase 1 implementation work and are not implemented here.

## 8. Confirmed production state machine

### 8.1 State representation

CONFIRMED conceptual states/predicates (some may coexist in the bounded header):

- Idle
- PreparedWithdrawal with remaining target
- AssigningWithdrawalLegs
- WithdrawalTargetAssigned / AwaitingLegInactivity
- PartiallyFinalized
- EscrowFunded
- Settled
- RecoveryRequired

Readiness is a derived predicate for each leg over current stake state, Clock,
and Stake History, not a keeper assertion. Target assignment and complete
finalization are checked header predicates. Completed is a terminal round result
recorded immediately before the reusable account returns to Idle. Retryable
failures normally do not need a state because Solana atomic rollback leaves the
same safe state. Pause is an orthogonal config flag.

### 8.2 Transition matrix

| Transition | Trigger / permission | Preconditions and invariants | Modified accounts | Replay / recovery | Boundary |
| --- | --- | --- | --- | --- | --- |
| Idle → no-yield result | prepare_distribution, anyone | unpaused; no active round; 10 days; all accounts valid/current; historical value ≤ HWM | event only | no snapshot, no clock reset, remain Idle | one tx |
| Idle → valid-insufficient result | prepare_distribution, anyone | positive yield; native shortfall; all validation completed; dynamic Jito amount below protocol minimum | config last_valid_insufficient_at only | 24h blocks another insufficiency evaluation; no snapshot/reclassification; malformed failures roll back | one tx |
| Idle → EscrowFunded (liquid path) | prepare_distribution, anyone | timing; positive yield; checked split; pending SOL fully covers outgoing; fixed recipients/KIF eligibility; no active round | config, ActiveDistribution, PendingSolVault, escrow | sequence opens once; last_prepared_at set now; failed tx changes nothing | one tx |
| Idle → PreparedWithdrawal | prepare_distribution, anyone | same, plus dynamic per-leg feasibility, operational reserve, and residual principal check | config, ActiveDistribution, PendingSolVault, escrow | fixes total q target, obligation, eligibility, floors, quote bounds, and sequence; no leg exists yet | one tx |
| PreparedWithdrawal / AssigningWithdrawalLegs / PartiallyFinalized → AssigningWithdrawalLegs or PartiallyFinalized | initiate_withdrawal_leg, anyone | remaining target; current pool/list; supplied candidate passes all checks; unique metadata/stake PDAs; both rents available; computed input equals `min(remaining, maximum safe capacity)` | op vault, principal Jito vault, pool/list/source, leg/stake PDAs, round | failed attempt consumes no index/capacity; success records exact per-leg data and increments cumulative values once | one tx per leg: create + protected withdraw + deactivate |
| AssigningWithdrawalLegs / PartiallyFinalized → WithdrawalTargetAssigned (possibly also PartiallyFinalized) | successful leg completion | checked cumulative assigned input equals fixed target exactly | ActiveDistribution | no further leg accepted; greater-than target is impossible | same leg tx |
| WithdrawalTargetAssigned → AwaitingLegInactivity | derived/read-only | one or more successful stake legs remain effective/deactivating | none | callers wait; legs may be ready in different epochs | no tx required |
| AssigningWithdrawalLegs / AwaitingLegInactivity / PartiallyFinalized / WithdrawalTargetAssigned → PartiallyFinalized | finalize_withdrawal_leg, anyone | exact recorded leg/PDAs/authorities; that leg fully inactive; whole balance to fixed escrow | stake/leg PDAs, escrow, op vault, round | premature leg call fails; success closes stake, reconciles exact values, recovers rents, and increments finalized count once; remaining target is unchanged | one tx per ready leg |
| PartiallyFinalized → EscrowFunded | final successful leg finalization | assigned target exact; successful count equals finalized count; all stake legs closed; escrow and cumulative totals reconcile | escrow, op vault, round | no iteration over closed legs; checked counters and deterministic replay guards prove completion | final leg tx |
| EscrowFunded → Settled | settle_distribution, anyone | escrow delta reconciled; rent returned; net amounts fit; recipient keys fixed; KIF snapshot fixed; residual HWM safe | escrow, op/KIF vaults, recipients, guardian ledgers, config/round | all beneficiary payments and accounting atomic; failure remains EscrowFunded | one tx |
| Settled → Completed → Idle | integrate_pending, anyone | exact current pending deltas; conservative contribution values; no unpaid round liability | pending/principal ledgers and vaults, HWM, config/round | sequence terminal summary stored; contributions after lock remain pending; failure remains Settled | one tx |
| Idle → Idle compounding | stake_principal_sol, anyone | unpaused; no active liabilities consume input; current pool; slippage floor | PrincipalSolQueue, pool/reserve/mint/fee, PrincipalJitoVault | exact deltas; failure leaves SOL principal queued | one tx |
| Any active state → same state | retry after atomic failure, anyone | same fixed destinations/obligation; refreshed current external accounts | only on success | no lowering caller-selected floors or destination changes | multiple tx over time |
| Any → RecoveryRequired/Paused | 4-of-6 authorized response or upgraded recovery | verified protocol/accounting incident; no normal HWM reduction | config and explicitly governed recovery state | no permissionless cancellation or withdrawal path | governance-specific |

Preparation fixes eligibility, gross split, pending-SOL snapshot used, total
JitoSOL input target, slippage constraints, proposed compound/HWM delta, KIF bitmap,
and recipient keys. Global HWM is not irrevocably increased during preparation;
the exact proposed delta is stored and committed atomically at settlement. This
preserves the economics while avoiding a partially credited failed cycle.

Confirmed logical transaction boundaries:

1. preparation/snapshot;
2. protected stake withdrawal + immediate deactivation, repeated per leg;
3. inactive stake finalization into escrow, repeated per leg;
4. atomic beneficiary settlement and cycle accounting;
5. pending-contribution integration/completion;
6. later principal-SOL/Jito compounding deposit.

These are logical boundaries, not exactly six transactions. Combining one
withdrawal leg and its deactivation is CONFIRMED BY LIVE TEST. The founder
confirmed the separation of preparation, per-leg stake finalization,
beneficiary settlement, pending integration, and later compounding.

### 8.3 Permissionless resumption

Another caller resumes from the current header and deterministic leg state:

- after some legs initiate, it fills only the exact remaining target;
- after a candidate disappears or an epoch changes, it refreshes current
  pool/list/fees/source and supplies another valid candidate;
- after some legs become inactive, it finalizes only those ready;
- after partial finalization, it uses checked counts/totals and cannot replay a
  closed or recorded leg;
- when the API is unavailable, it may use a technically safe candidate passing
  every enforceable check, with only the accepted efficiency trade-off;
- when pool state is stale, no counter changes until permissionless SPL
  maintenance makes the relevant state current;
- when operational rent is temporarily insufficient, no leg/index is consumed,
  and work resumes after the approved operational category is replenished.

Settlement does not iterate historical validators or closed legs. Deterministic
temporary accounts, bounded indices, checked cumulative counters, and replay
flags are sufficient.

## 9. Principal and yield accounting

### 9.1 Variables and conversion functions

All values are unsigned checked integers. All multiplication uses a checked
u128 intermediate and every conversion back to u64 is checked.

At a current, validated pool state:

- T = pool total_lamports.
- S = pool pool_token_supply.
- q = JitoSOL base units.
- f_n / f_d = current stake-withdrawal fee.
- H = protected-principal high-water mark in lamports.
- Q_h = accounted historical principal JitoSOL units, excluding pending units.
- L_h = reconciled historical principal SOL in PrincipalSolQueue.
- P_s = snapshotted pending SOL contribution value excluding vault rent.
- P_j = snapshotted pending JitoSOL units.

Functions:

~~~text
exchange_value(q) = floor(q * T / S)

fee_apply(x, n, d) =
  0                                      if d = 0
  ceil(x * n / d)                        otherwise

withdrawal_fee_units(q) =
  fee_apply(q, f_n, f_d)

burn_units(q) = q - withdrawal_fee_units(q)

redemption_value(q) = floor(burn_units(q) * T / S)

deposit_gross_units(lamports) = floor(lamports * S / T)
deposit_fee_units =
  fee_apply(deposit_gross_units, sol_deposit_fee_n, sol_deposit_fee_d)
deposit_net_units = deposit_gross_units - deposit_fee_units
~~~

exchange_value is the official book/exchange value used for HWM comparison.
redemption_value is the conservative native stake output after the current
withdrawal fee. A DEX price is never used.

### 9.2 Yield and fixed gross split

~~~text
historical_value = exchange_value(Q_h) + L_h + prior_undistributed_yield_carry
gross_yield Y = max(0, historical_value - H)

htfp_gross     = floor(Y * 5900 / 10000)
compound       = floor(Y * 1950 / 10000)
team_gross     = floor(Y * 1950 / 10000)
kif_gross      = floor(Y *  200 / 10000)
split_dust     = Y - htfp_gross - compound - team_gross - kif_gross
outgoing_gross = htfp_gross + team_gross + kif_gross
~~~

If historical_value ≤ H, Y is zero, no distribution opens, and H never
decreases. Split dust remains PIV1 value and is protected in the settled HWM
delta; it never permits principal erosion.

### 9.3 Pending SOL first and JitoSOL shortfall

~~~text
pending_sol_used = min(P_s, outgoing_gross)
stake_gross_budget = outgoing_gross - pending_sol_used

max_q_for_gross_budget(B) =
  floor((((B + 1) * S) - 1) / T), checked in u128

q_round = min(Q_h_available, max_q_for_gross_budget(stake_gross_budget))
snapshot_leg_input_floor = q_protocol at preparation

cumulative_q_assigned = 0
for each successful leg i:
  remaining_q = q_round - cumulative_q_assigned
  q_i = min(remaining_q, candidate_maximum_safe_q_i)
  require q_i equals that maximum-safe fill
  require q_i >= max(snapshot_leg_input_floor, q_protocol_i)
  fee_i = fee_apply(q_i, fee_n_i, fee_d_i)
  burn_i = q_i - fee_i
  expected_i = floor(burn_i * T_i / S_i)
  minimum_out_i = max(
    runtime_minimum_delegation_i,
    floor(expected_i * 9999 / 10000)
  )
  cumulative_q_assigned += q_i
  require cumulative_q_assigned <= q_round
  cumulative_fee_units += fee_i
  cumulative_burn_units += burn_i
  cumulative_delegated_out += observed_delegated_out_i

require cumulative_q_assigned == q_round before settlement

stake_book_cost = exchange_value(q_round) at snapshot
eligible_finalized_native = sum(
  min(observed_native_after_rent_i, observed_delegated_out_i)
)
protocol_and_split_cost = max(0, stake_book_cost - eligible_finalized_native)
conversion_dust = max(0, stake_gross_budget - stake_book_cost)
post_snapshot_book_increase = max(0, stake_book_cost - stake_gross_budget)

beneficiary_net_total =
  min(outgoing_gross, pending_sol_used + eligible_finalized_native)
total_outgoing_reduction =
  outgoing_gross - beneficiary_net_total
~~~

This charges withdrawal fees and conservative conversion dust only against the
outgoing 80.5% allocation. The compound share and protected principal are not
used to pay them. `q_round` and the gross budget are fixed at preparation, but
each SPL call calculates its own ceiling-rounded fee, burn, expected output,
1-bps floor, technical minimum, and exact deltas from current validated state.
The round must never calculate split fees as if all input were one withdrawal.
Any additional per-leg fee rounding or conversion dust caused by splitting is
charged only to the outgoing 80.5%, never protected principal or compound.
All accumulations use checked `u128` intermediates and checked storage
conversions.

If book value rises between legs, beneficiary funding remains capped at the
fixed outgoing gross allocation. Cooldown rewards and other post-snapshot
appreciation are separately tracked next-cycle yield. If observed native value
net of rent is below delegated output, the round enters `RecoveryRequired`
instead of paying through a loss. Execution also requires the residual-HWM
invariant, so changed pool state cannot silently consume principal or compound.

Net beneficiary allocation after costs uses the confirmed outgoing relative
weights:

~~~text
outgoing_weight = 5900 + 1950 + 200 = 8050
htfp_net = min(htfp_gross, floor(beneficiary_net_total * 5900 / 8050))
team_net = min(team_gross, floor(beneficiary_net_total * 1950 / 8050))
kif_net  = min(kif_gross,  floor(beneficiary_net_total *  200 / 8050))
net_allocation_dust =
  beneficiary_net_total - htfp_net - team_net - kif_net
~~~

The implementation must prove by checked assertions that each net share is no
greater than its gross share and their sum is no greater than available native
SOL. Any net dust remains in PIV1 and is added to protected retained value.

### 9.4 Contributions and new HWM

At pending integration:

~~~text
pending_jitosol_value = exchange_value(P_j) at integration
contribution_value = P_s_total + pending_jitosol_value

new_H =
  old_H
  + contribution_value
  + compound
  + split_dust
  + conversion_dust
  + net_allocation_dust
  + any confirmed zero-active-KIF compound
~~~

Using pending SOL for beneficiaries does not reduce its contribution value:
the same amount of historical yield remains physically staked and is
reclassified into principal. P_j appreciation before integration is
conservatively part of the contribution, not historical yield. Contributions
arriving after the integration account lock remain pending for the next
reconciliation.

### 9.5 KIF allocation

KIF activity uses Solana Clock `unix_timestamp`, not an off-chain database:

~~~text
period_seconds = 2_592_000
require unix_timestamp >= configured_anchor_timestamp
period_id = floor(
  (unix_timestamp - configured_anchor_timestamp) / period_seconds
)
period_start = configured_anchor_timestamp + period_id * period_seconds
period_end = period_start + period_seconds
period_start <= unix_timestamp < period_end
~~~

A valid guardian heartbeat or qualifying governance vote counts for its current
period. The distribution snapshots its eligibility; activity after that
snapshot is not retroactive for the distribution.

If a > 0 guardians are active:

~~~text
kif_available = kif_net + approved prior carry
per_guardian = floor(kif_available / a)
credited = per_guardian * a
kif_rounding_remainder = kif_available - credited
~~~

Only snapshotted active guardians receive credit. If a = 0, the confirmed rule
is:

~~~text
kif_available = kif_net + all approved prior carry
compound_from_kif = floor(kif_available / 2)
kif_carry_next = kif_available - compound_from_kif
~~~

For `a > 0`, `kif_rounding_remainder` stays in `KifSolVault` as explicit
collective carry for a later allocation. It is not preferentially assigned and
does not enter HTFP, Team Owner, arbitrary-recipient, or ordinary-principal
accounting. Inactive guardians receive no claim and no retroactive credit.

For `a = 0`, `kif_available` includes current net KIF plus all approved prior
carry, and the 50/50 formula is applied again in every successive zero-active
period. The compounded floor permanently increases HWM-protected principal;
the remainder remains collective KIF carry.

### 9.6 Cooldown rewards and rent

For each leg `i`, store at initiation:

- `delegated_out_i` = actual stake lamports received from the protected CPI;
- `stake_rent_advanced_i` = actual lamports prefunded into the stake PDA;
- `metadata_rent_advanced_i` = actual lamports funding the leg PDA;
- `escrow_before_i` and the leg status.

At finalization:

~~~text
recovered_total_i = escrow_after_i - escrow_before_i
native_after_rent_i = recovered_total_i - stake_rent_advanced_i

cooldown_rewards_i = max(0, native_after_rent_i - delegated_out_i)
cooldown_loss_i = max(0, delegated_out_i - native_after_rent_i)

cumulative_finalized_sol += recovered_total_i
cumulative_recovered_stake_rent += stake_rent_advanced_i
cumulative_recovered_metadata_rent += observed_metadata_close_delta_i
cumulative_cooldown_rewards += cooldown_rewards_i
cumulative_cooldown_losses += cooldown_loss_i
~~~

Each exact stake-rent advance returns from escrow to `OperationalSolVault`.
Closing the temporary metadata returns its exact rent to the same operational
category. Neither is yield. Cooldown rewards are excluded from the already-
fixed round and recorded explicitly as next-cycle yield for the normal later
`59% / 19.5% / 19.5% / 2%` split; they are not silently principal. Any per-leg
cooldown loss or residual-HWM failure enters `RecoveryRequired` and cannot
reduce H normally. All cumulative updates are checked and atomic with that
leg's finalization.

### 9.7 Task 0.4 arithmetic reconciliation

Observed live pool snapshot:

~~~text
T = 4,367,920,287,905
S = 3,294,679,564,826

deposit input = 1,001,001,003 lamports
floor(input * S / T) = 755,045,269 JitoSOL units

withdraw input q = 756,045,269 units
fee = ceil(q / 1000) = 756,046 units
burn = 755,289,223 units
exchange_value(q) = 1,002,326,752 lamports
redemption_value(q) = 1,001,324,424 lamports
protocol book cost = 1,002,328 lamports
~~~

Finalization:

~~~text
delegated_out                  1,001,324,424
cooldown rewards                  1,016,567
recovered stake rent              2,282,880
recovered_total               1,004,623,871
~~~

The escrow increased by exactly 1,004,623,871 lamports, the stake PDA closed,
and the caller lost only the 5,000-lamport transaction fee.

## 10. Dynamic technical minimum

### 10.1 Protocol minimum

For current validated pool state and runtime stake rules:

~~~text
D = Stake Program runtime minimum delegation
R_stake = Rent sysvar minimum_balance(StakeStateV2::size_of())
R_leg = Rent sysvar minimum_balance(bounded WithdrawalLeg::SPACE)

q_protocol = least positive q such that redemption_value(q) >= D
gross_shortfall_minimum = exchange_value(q_protocol)

candidate_maximum_safe_q_i = greatest q such that:
  redemption_value_i(q) <=
    validated_source_lamports_i - pinned_SPL_required_residual_i

operational_spendable =
  OperationalSolVault.balance - OperationalSolVault.permanent_rent_floor

may_open_with_withdrawal only if all are true:
  stake_gross_budget >= gross_shortfall_minimum
  q_round >= q_protocol
  Q_h_available >= q_round
  at least one current candidate supports a valid first leg
  operational_spendable >= R_stake + R_leg
  every per-leg minimum_lamports_out_i >= D
  residual historical assets after the planned burn satisfy the new HWM

for each supplied candidate i:
  candidate_maximum_safe_q_i is derived from its current validated source
  q_i = min(q_round - cumulative_q_assigned, candidate_maximum_safe_q_i)
  q_i >= max(snapshot_leg_input_floor, current q_protocol_i)
  remaining_after_i = q_round - cumulative_q_assigned - q_i
  remaining_after_i == 0 or
    remaining_after_i >= max(snapshot_leg_input_floor, current q_protocol_i)
~~~

`q_protocol` and the inverse capacity bound must be found with checked monotone
searches using the exact pinned fee/redemption and source-residual rules. Rent
and D come from the runtime/Rent sysvar, not
an RPC-only constant. The selected validator entry, stake balance, status,
derived address, and source stake state also participate in feasibility. A
candidate whose mandatory maximum fill would strand a nonzero sub-minimum
remainder is rejected atomically; another candidate can be supplied without
consuming the index or target.

Current point-in-time examples:

| Cluster/snapshot | q_protocol | Fee units | Gross book cost | Net delegated | Stake rent |
| --- | ---: | ---: | ---: | ---: | ---: |
| Task 0.4 Testnet epoch 1018 | 755,045,269 | 755,046 | 1,001,001,002 | 1,000,000,000 | 2,282,880 |
| Testnet epoch 1021 inspection | 753,972,377 | 753,973 | 1,001,001,002 | 1,000,000,000 | 2,077,224 |
| Mainnet epoch 1024 read-only | 770,931,087 | 770,932 | 1,001,001,003 | 1,000,000,001 | 2,282,880 |

These values demonstrate the formula and must not be production constants.

### 10.2 Slippage and transaction-fee relationship

Each leg's technical gate uses its current expected output and program-derived
`minimum_lamports_out_i`. The configured tolerance is 0 or 1 bps under the
immutable 1-bps cap and cannot lower the effective minimum below D or
allow the residual-HWM invariant to fail. If the current quote no longer meets
the snapshotted bound, the CPI fails and the same round is safely retried after
pool maintenance/requote under the approved policy.

Network and priority fees are not part of protected principal or
q_protocol. The external fee payer must have enough SOL; insufficient fee
balance fails the transaction atomically and must not update the 24-hour
cooldown.

### 10.3 Separate thresholds

| Threshold | Production treatment |
| --- | --- |
| Protocol minimum | Mandatory dynamic q_protocol and validator/source constraints |
| Economically sensible minimum | None required merely to protect the caller; P-025 remains confirmed |
| Operational reserve | Separate current per-leg stake and metadata rent plus permanent vault rent floor; each advance is recycled after finalization/closure |
| Founder policy minimum | None beyond confirmed technical and accounting gates; no economic cap or threshold is invented |

A valid amount-insufficient evaluation writes only the 24-hour timestamp/event
and succeeds without snapshot, withdrawal, reclassification, 10-day-clock
reset, or active round. Every malformed or otherwise failed transaction rolls
back and cannot extend that cooldown.

## 11. Validator selection and pool updates

### 11.1 Confirmed operational selection

1. The official permissionless keeper queries Jito's current Preferred
   Withdraw Validator List API, then reads the configured pool, validator list,
   Clock epoch, stake accounts, fees, and on-chain preferred-withdraw setting.
2. It selects the minimum necessary number of Jito-recommended candidates,
   favoring larger safe capacity where compatible with pinned SPL ordering, so
   the fixed target is not unnecessarily fragmented.
3. If a preferred withdraw validator is configured and still has withdrawable
   capacity, it is used and exhausted according to the pinned SPL rule before
   another validator source. The stake-pool program also enforces active,
   transient, and reserve source order.
4. The keeper supplies the list index, vote account, seed suffix, and standard
   validator source to PIV1. It cannot choose an arbitrary smaller leg amount,
   the stake/metadata destinations, PIV1-derived token-vault address,
   PivAuthority, escrow, fee account, mint, pool, or program.
5. If the candidate changes or loses liquidity, the protected CPI fails
   atomically without consuming an index or target. Another keeper retries the
   exact remaining target with another current candidate.

The API returns operational recommendations and reported withdrawable capacity;
it is not an authenticated on-chain oracle. Any wallet may supply a different
candidate when the API is unavailable or changed, but only if every enforceable
on-chain check passes. The accepted residual risk is limited cooldown-reward or
pool-rebalancing inefficiency; fixed custody and destinations prevent theft or
redirection. The one standard active source used in Task 0.4 is live-tested;
multi-source selection and SPL fallback branches require later implementation
tests.

### 11.2 Bounded on-chain validation

The pinned validator list has a 9-byte header/vector prefix and fixed 73-byte
ValidatorStakeInfo records containing active lamports, transient lamports,
last-update epoch, transient seed suffix, unused value, validator seed suffix,
status, and vote address. Production must:

- validate stake-pool program ownership and the pool's exact validator-list key;
- validate header/account type, vector length, data length, and checked index
  arithmetic;
- zero-copy only the supplied 73-byte record at the checked index;
- require its vote, status, last-update epoch, and seed suffix to match;
- derive the standard validator stake PDA from
  [vote, configured pool, optional nonzero seed suffix] under the configured
  stake-pool program;
- decode the supplied Stake Program account and require the same voter and
  pool-authority relationship;
- require enough source balance to leave its rent reserve plus runtime minimum
  delegation and the pinned SPL tolerance;
- calculate and enforce the candidate's maximum safe input, reject caller-
  selected micro amounts and sub-minimum stranded remainders, and bind the
  unique round/index metadata and stake PDAs;
- still rely on the stake-pool program's own full membership, source-order,
  fee, mint, liquidity, and state checks.

PIV1 must not iterate the complete list. Testnet had 1,129 real entries and
Mainnet 687 at inspection; the pinned SPL withdrawal already performs the
protocol-required list work and Task 0.4 measured the resulting compute.

### 11.3 Permissionless pool maintenance

SPL Stake Pool 2.0.3 allows at most four validators per
UpdateValidatorListBalance instruction. Each chunk has seven fixed accounts and
two stake accounts per validator. After every stale validator record is
updated, UpdateStakePoolBalance recomputes aggregate totals, mints the epoch
manager fee, activates scheduled fee changes, updates supply, and sets the pool
epoch.

Confirmed policy:

- keepers call the SPL update instructions directly; PIV1 does not wrap them;
- update only stale chunks where possible, up to four entries per transaction;
- then call the aggregate pool update;
- PIV1 prepare/deposit/withdraw instructions reject unless both aggregate pool
  and selected entry are current for Clock.epoch;
- no PIV caller reward or fee reimbursement exists;
- after an epoch boundary or list mutation, keepers refresh the complete quote
  and candidate before PIV execution.

At the inspected sizes, a fully stale list could require up to 283 Testnet or
172 Mainnet four-entry chunk transactions plus the aggregate update. This is
why PIV1 cannot synchronously update the entire pool or iterate it on-chain.
This maintenance policy is CONFIRMED. If the official API is unavailable,
direct on-chain reads and maintenance remain usable; if pool state is stale,
PIV1 mutates no round/leg counter until current-state validation succeeds.

## 12. Slippage policy

### 12.1 Required calculation

Stake-pool direct operations have no DEX market-price slippage, but they remain
exposed to epoch updates, rewards, fee changes, concurrent transactions, and
stale quotes. Both CPI variants therefore require an on-chain output floor.

Let `t_config` be the configured tolerance. The initial production value is 1
bps, Config accepts only 0 or 1 bps, and `HARD_CAP_BPS = 1` is immutable without
a reviewed program upgrade:

~~~text
expected_deposit = deposit_net_units(lamports_in)
minimum_pool_tokens_out =
  floor(expected_deposit * (10000 - t_config) / 10000)

snapshot_leg_input_floor = q_protocol at preparation
max_useful_legs = floor(q_round / snapshot_leg_input_floor)

snapshot_aggregate_fee =
  fee_apply(q_round, snapshot_fee_n, snapshot_fee_d)

fee_ceiling_reserve_units =
  0                              if the snapshot fee is identically zero
  max_useful_legs - 1            otherwise

snapshot_conservative_burn =
  q_round
  - snapshot_aggregate_fee
  - fee_ceiling_reserve_units

conversion_floor_reserve_lamports = max_useful_legs - 1
round_expected_withdraw_lower_bound =
  floor(snapshot_conservative_burn * snapshot_T / snapshot_S)
  - conversion_floor_reserve_lamports

stored_round_minimum_native =
  floor(round_expected_withdraw_lower_bound *
        (10000 - t_config) / 10000)

expected_withdraw_i = redemption_value_i(q_i)
minimum_lamports_out_i =
  max(
    runtime_minimum_delegation_i,
    floor(expected_withdraw_i * (10000 - t_config) / 10000)
  )

require 0 <= t_config <= HARD_CAP_BPS == 1
require each q_i >= max(snapshot_leg_input_floor, current q_protocol_i)
require final cumulative delegated output satisfies the stored round floor
~~~

The program, not the caller, derives or verifies these values from Config and
the round snapshot. The caller cannot submit a weaker floor. After CPI, exact
source/destination balance deltas must also equal the accepted protocol result.
All subtractions and products in the round-floor derivation are checked. Because
every successful leg meets the stored snapshot leg-input floor,
`max_useful_legs` is a mathematical bound from the real target and technical
minimum, not a low economic cap. For `n` calls using the snapshot fee, the sum
of ceiling fees can exceed the one-call ceiling by at most `n - 1` pool-token
units, and the sum of native conversion floors can trail the aggregate floor by
at most `n - 1` lamports. Reserving both amounts prevents even `t_config = 0`
from pretending a multi-leg round is one SPL withdrawal. Current per-leg fee,
pool, minimum, slippage, exact-delta, and residual-HWM checks still apply; any
later state drift must also preserve this immutable conservative round floor.

### 12.2 Snapshotted and current values

Preparation must store:

- pool address/program and pool last_update_epoch;
- T, S, relevant current fee fractions, and quote slot/epoch;
- deposit input or fixed total `q_round` target;
- snapshot technical leg-input floor, maximum useful-leg bound, conservative
  split-round expected output, configured tolerance, and immutable stored round
  minimum;
- technical minimum and selected outgoing gross budget;
- residual-HWM proof inputs.

Each successful withdrawal leg separately stores its current T/S/fee/epoch,
exact `q_i`, expected output, and `minimum_lamports_out_i`.

Execution must decode current pool state again. Stale pool/list state is always
rejected. A current quote that still satisfies the immutable stored minimum and
residual-HWM invariant may execute; otherwise it fails without state mutation.

Safe retries refresh current pool/list/candidate accounts while preserving the
round's sequence, fixed total token target, already assigned input, fixed
destination, eligibility, gross allocation, and exact stored floor. A retry may
never lower that floor or
accept a tolerance above Config merely to force completion. If a protocol
change makes the stored floor permanently unreachable, pause/governed recovery
is required.

### 12.3 Confirmed bound and upgrade rule

The 0–1-bps configuration range and immutable 1-bps cap are CONFIRMED. No caller
can supply or authorize a weaker floor, and an ordinary configuration change
cannot raise the cap. Any value above 1 bps requires a reviewed program upgrade.
Broad caller-selected tolerances and unprotected basic SPL variants are
REJECTED.

## 13. Fees, rent, compute, and transaction constraints

### 13.1 Live observed costs

| Stage | Fee | Compute | Other measured constraint |
| --- | ---: | ---: | --- |
| Direct-client contribution funding deposit | 5,000 lamports | 24,811 CU | one direct client operation |
| PIV account funding | 5,000 | 13,818 CU | Testnet probe only |
| Probe initialization | 5,000 | 27,253 CU | experimental |
| PIV DepositSolWithSlippage | 5,000 | 32,243 CU | local form: 603 bytes, 15 message keys |
| Direct JitoSOL contribution | 5,000 | 10,312 CU | TransferChecked |
| Create + WithdrawStakeWithSlippage + Deactivate | 5,000 | 158,952 CU | local comparable: 768 bytes, 20 keys |
| Expected premature finalization | 5,000 | 17,577 CU | correctly failed |
| Successful finalization to escrow | 5,000 | 28,453 CU | local comparable: 436 bytes, 10 keys |

The successful finalization fee was paid only by the caller. The stake-pool
stake-withdrawal fee was 756,046 JitoSOL units on 756,045,269 input; deposit
fees were zero. This is one-leg evidence. In a multi-leg round every
`WithdrawStakeWithSlippage` call independently ceiling-rounds its fee and burns
its own net pool units; current fee fractions are dynamic pool state.

Task 0.4 rents:

- withdrawal stake: 2,282,880 lamports, fully recovered;
- zero-data System vault/escrow: 890,880 each;
- 165-byte legacy token account: 2,039,280 per account; a PIV1-PDA-addressed
  account has the same token-account data size, and the two production vaults
  must each be rent-funded independently;
- probe config: 2,596,080;
- probe round: 1,642,560.

Those are observed amounts, not production constants. Current read-only
Testnet stake rent is already different. Production also advances rent for each
bounded `WithdrawalLeg` metadata account; its exact size and rent are Phase 1
schema measurements. Stake and metadata rent are recorded separately per leg,
recovered only to the operational category, and never counted as yield.

### 13.2 Runtime transaction limits and safety margins

Current legacy/v0 transactions are limited to 1,232 serialized bytes. The
runtime permits at most 1,400,000 CU per transaction; a user-program instruction
defaults to 200,000 CU unless an explicit compute-budget instruction changes
the limit. The current top-level-plus-CPI stack depth is five (feature-gated
future depth may be higher). CPI compute is charged to the same transaction.

The measured 158,952-CU withdrawal leaves too little room under an implicit
200,000-CU default for production accounting, zero-copy list validation,
events, and invariant checks. Production clients must:

- simulate the exact transaction against representative worst-case state;
- set an explicit compute-unit limit with at least the official recommended
  10% margin, then add further measured margin for worst-case production
  branches;
- set priority fees only at caller expense;
- remeasure serialized size, keys, CU, loaded data, and CPI depth after every
  account/schema/CPI change;
- keep a regression test below an approved size/CU ceiling rather than relying
  on the protocol maximum.

The observed CPI stack is safe: PIV1 invokes SPL Stake Pool, which invokes the
Token or Stake Program; subsequent Deactivate is a separate sequential CPI
after the stake-pool CPI returns. It does not nest the full lifecycle.

### 13.3 Safe and unsafe combinations

CONFIRMED safe for one leg, subject to production remeasurement:

- withdrawal-stake creation;
- WithdrawStakeWithSlippage;
- immediate Stake Deactivate.

Required to combine:

- all HTFP/Team transfers, KIF-vault funding/credits, and the corresponding
  cycle accounting in one atomic settlement transaction.

Keep separate:

- preparation from external stake withdrawal;
- cooldown wait from finalization;
- every per-leg full stake withdrawal to escrow from beneficiary settlement;
- pending contribution integration from already-paid settlement;
- principal-SOL Jito deposit from settlement.

These boundaries minimize duplicate-payment risk and make every partial success
recoverable. Boundaries 2 and 3 repeat for as many valid legs as the fixed
target requires. No low permanent economic leg or distribution cap is approved;
any safely large technical index/account bound required by Phase 1 must be
evidenced and distinguished from an economic cap. A versioned transaction/Address Lookup Table may reduce message
key bytes, but must not weaken account validation and is not required by the
probe evidence.

### 13.4 Epoch timing

Deactivation readiness is state-based, not time-based. Official documentation
warns that cooldown can take multiple epoch boundaries depending on network
stake behavior. Jito's user-facing delayed path is commonly described as up to
an epoch, but Task 0.4 only proves deactivation in epoch 1018 and confirmed
withdrawability/finalization when checked in epoch 1021; it does not prove the
earliest ready slot. Keepers may wait indefinitely and retry. The program must
read Clock and Stake History and require zero effective/deactivating stake.

## 14. Permissionless keeper model

Anyone may call:

- prepare_distribution;
- initiate_withdrawal_leg and any measured deactivation fallback;
- finalize_withdrawal_leg;
- settle_distribution;
- integrate_pending;
- stake_principal_sol;
- reconcile untracked balances;
- SPL validator-list/pool maintenance directly;
- guardian claims only with the entitled guardian authorization.

The caller:

- signs the top-level transaction and pays base/priority fees;
- may provide refreshed external pool/list/candidate account views;
- receives no automatic reward, reimbursement, token, stake authority, escrow
  authority, or recipient choice;
- cannot choose the Jito pool, mint, programs, fee account, PIV1-derived
  JitoSOL vault addresses, PivAuthority token authority, withdrawal-leg/stake
  PDAs, arbitrary smaller input, escrow, HTFP/Team recipients, KIF liability
  vault, or round sequence.

PIV1 advances only the temporary leg-metadata and withdrawal-stake rents from
its separately accounted operational reserve. Every recorded rent advance is
returned there after the respective account closes. Network fees remain
external.

Every transition is status- and sequence-guarded. Duplicate calls either fail
without mutation or, for idempotent maintenance/reconciliation checks, observe
that no work remains. No caller-controlled remaining account can redirect
value.

If nobody calls, no maximum deadline is violated: the 10-day rule is a minimum.
Yield and pending contributions continue accumulating. If a round is already
active, its fixed state remains until a keeper resumes it; there is no caller
reward or automatic expiry. A verified protocol incident uses pause and 4-of-6
upgrade/recovery authority, not an invented permissionless escape hatch.

Guardians approve or change general policy through governance, monitor the
system, pause during a verified incident, and may approve a future reviewed
integration change if Jito architecture changes. They do not approve routine
candidates, legs, deactivation, or finalization.

## 15. Threat and failure analysis

| Threat/failure | Prevention | Detection | Recovery |
| --- | --- | --- | --- |
| Wrong pool/program | Config-pinned program/pool; executable/owner checks | decoded key/owner mismatch | atomic failure; governance update only through approved strategy migration |
| Wrong mint/token program | Pool binding, mint owner/decimals/authority/supply, legacy Token ID, vault AccountInfo.owner, and decoded vault mint | account program-owner or decoded mint mismatch | atomic failure; pause/migration if official topology changes |
| Wrong validator/list entry | exact list owner/key; checked index/record; derived standard stake PDA | record/vote/seed/status/delegation mismatch | retry another validated current candidate |
| Wrong reserve | require pool.reserve_stake and Stake Program state/authority | key/state mismatch | atomic failure; refresh official pool state |
| Wrong manager fee/referrer | require pool.manager_fee_account and fixed referrer | token mint/key/owner mismatch | atomic failure; no caller substitution |
| Wrong token-vault address | Distinct PIV1 derivations for PrincipalJitoVault and PendingJitoVault; exact seed/bump/key checks; never derive both through the ATA program | vault-address derivation mismatch, equality, or wrong PIV1 program | atomic failure; correct accounts before funds; governance recovery if initialized state is corrupted |
| Wrong token authority | Decode both legacy token accounts and require PivAuthority in each token-account owner/authority field while the legacy Token program remains the account program owner | token-account decode, authority mismatch, or balance-delta mismatch | atomic failure; governance recovery if account is corrupted |
| Wrong leg/stake PDA | sequence-and-index-derived metadata/stake PDAs with fixed owners/spaces | seed/bump/key/owner or metadata binding mismatch | atomic failure |
| Leg/stake-PDA reuse | monotonic sequence/index; recorded-leg guard; require zero lamports/empty before creation | header/metadata check and create failure | reject without consuming index/capacity; closed/recorded legs never reopen |
| Round reuse/double withdrawal | one active round, monotonic next_sequence/index, cumulative target and status guards | sequence/index/status mismatch; token/stake deltas | reject replay; terminal counters plus closed temporary accounts |
| Stale pool/list | require aggregate and selected entry epoch equal Clock | decoded last_update_epoch | permissionless chunk updates then aggregate update/retry |
| Slippage failure | protected variants and Config-derived floor | SPL ExceededSlippage or post-delta mismatch | refresh quote under same bounded policy; never weaken caller-side |
| Validator liquidity change | precheck source residual; SPL source-order/liquidity checks | atomic CPI error | retry same round with another valid candidate |
| Caller micro-fragments target | program computes `min(remaining, candidate maximum safe capacity)`; candidate cannot supply amount | input/capacity equality and nonzero-remainder minimum checks | atomic rejection; another keeper supplies a useful source |
| Jito API unavailable or caller ignores recommendation | API is operational guidance; all enforceable safety is on-chain; custody/destinations fixed | keeper monitoring and candidate differs from recommendation | safe candidate may proceed; accepted limited cooldown-reward/rebalancing inefficiency only |
| Below technical minimum | dynamic binary-search gate before snapshot | q_round < q_protocol or candidate infeasible | record only valid-insufficient 24h timestamp; yield accumulates |
| Insufficient operational reserve | separate balance and runtime stake-plus-metadata rent checks | spendable reserve < current leg rents | no leg/index/capacity mutation; replenish approved operational category and retry |
| Negative/no yield | max(0, historical value − H), no H reduction | checked comparison | no snapshot; wait for recovery or governed migration |
| Mid-round contribution | physical pending vaults plus accounted-unit ledgers | positive unaccounted balance delta | keep pending until post-settlement integration |
| Premature leg finalization | Stake state, Clock, Stake History, exact leg status | effective/deactivating stake nonzero | atomic failure; wait and retry; other ready legs may finalize |
| Partial initiation/finalization abandoned | bounded cumulative header, deterministic indices, no caller-exclusive lock | target/count/status predicates show exact remaining work | another permissionless caller resumes from current pool and leg states |
| Per-leg rounding treated as one withdrawal | independent fee ceiling, burn, floor, exact delta, rent, reward/loss records for every leg | cumulative sum assertions disagree with per-leg observed values | atomic leg failure or `RecoveryRequired`; never charge principal/compound |
| Settlement before all legs complete | exact target equality; successful count equals finalized count; all-stake-closed flag; escrow reconciliation | complete-before-settlement predicates fail | remain resumable; no beneficiary transfer occurs |
| Replay/double settlement | state transition before/with effects, monotonic sequence | status mismatch | reject; failed settlement leaves EscrowFunded with no partial payments |
| Overflow/underflow | u128 intermediates, checked operations/conversions | explicit arithmetic error | atomic failure; property/boundary tests before deployment |
| Rounding exploitation | all outgoing floors; fee ceiling; exact balance deltas; dust retained | invariant sum checks | atomic failure; dust remains protected |
| Account substitution | no unchecked economic remaining accounts; config/PDA/pool derivations, including distinct token-vault address PDAs | key/owner/data mismatch | atomic failure |
| Malicious permissionless caller | fixed inputs/destinations; no custody/reward; caller only fee payer | signer/status/account constraints | reject; another keeper retries |
| Jito pause/failure/deprecation | explicit PIV pause, HWM, upgradeable 4-of-6 authority | repeated official CPI/account failures and monitoring | pause; reviewed upgrade/migration; no silent DEX fallback |
| Abandoned active distribution | fixed recoverable states and no caller-exclusive lock | elapsed time monitoring | another keeper resumes; verified incident requires governed recovery |
| Epoch delay | readiness derived from live stake state, no fixed deadline | Clock/Stake History | keep waiting; no new round while active |
| Cooldown loss/reward variance | store delegated/rent amounts; exact final delta | rewards/loss formulas and residual-HWM check | carry approved rewards; loss enters RecoveryRequired, no HWM reduction |
| Failed beneficiary settlement | all payments/KIF credits/accounting in one transaction | transaction error and unchanged EscrowFunded state | retry exact settlement; no duplicate/partial payout |
| Unexpected direct transfer | ledgered balances rather than raw vault total | reconciliation delta | classify as pending contribution; never historical yield |
| Pool fee/authority change | current decode each operation; scheduled-fee-aware quote | config/pool comparison and output floor | reject/requote; governance only if binding policy must change |
| Pause during active round | pause blocks confirmed economic operations | config flag | guardians review and unpause or deploy approved recovery; direct incoming remains pending |

No proposed recovery creates a contributor withdrawal, caller reward, DEX exit,
normal HWM reduction, or recipient redirection.

### 15.1 Required later multi-leg tests

Task 1.1 creates placeholders only. Later authorized implementation tasks must
add unit/property/local integration coverage for:

- Config values 0 and 1 bps, immutable rejection above 1 bps, stored-round and
  per-leg floors, exact post-CPI deltas, and residual-HWM failures;
- exact target assignment across one, two, and many source capacities, including
  checked overflow/conversion boundaries and rejection above the target;
- maximum-safe fill, arbitrary micro-amount rejection, sub-minimum remainder
  rejection, unique/closed/replayed index rejection, and rent-drain attempts;
- independent per-leg ceiling fees, burn, native floors, stake/metadata rent,
  cooldown rewards/losses, recovered rent, and cumulative reconciliation;
- candidate failure, epoch change, API outage, stale pool, and temporary
  operational-rent exhaustion after partial initiation;
- independently inactive legs, out-of-order and partial finalization, and
  resume by different permissionless callers;
- settlement rejection until target equality, matching successful/finalized
  counts, all stake closures, and fixed-escrow reconciliation;
- exact KIF Clock boundaries, post-snapshot non-retroactivity, repeated
  zero-active carry, and active-guardian division remainder carry.

The public Task 0.4 lifecycle supplies the one-leg custody control case. It does
not satisfy these multi-leg production tests.

## 16. Differences integrated into the master specification

This task synchronizes the master specification with accepted Phase 0 evidence
and founder decisions.

| Earlier specification point | Integrated result | Evidence status |
| --- | --- | --- |
| Direct operations described without a fixed tolerance | Protected variants only; Config 0–1 bps, initial 1 bps, immutable 1-bps cap | Policy CONFIRMED; protected calls tested |
| One withdrawal stake account represented the active distribution | One bounded reusable header plus deterministic `(sequence, leg_index)` metadata/stake accounts | Architecture CONFIRMED; one leg live-tested, orchestration not tested |
| A validator was expected to cover the whole round | Exact target may span the minimum necessary number of current safe sources | Architecture CONFIRMED; multi-source path not tested |
| Final withdrawal, payout, reconciliation, and compounding could be combined | Six confirmed logical boundaries; leg initiation/finalization may repeat | Policy CONFIRMED; only one-leg custody boundaries live-tested |
| JitoSOL token-vault form was provisional | Two distinct non-ATA PIV1-derived 165-byte legacy token accounts share decoded `PivAuthority` | Policy CONFIRMED; production initialization not implemented |
| Technical minimum was open | Runtime-derived per leg from current pool fee/math, minimum delegation, source residual, and rents | Formula/evidence CONFIRMED; cluster values remain dynamic |
| Operational rent covered one stake account | Record and recover stake plus bounded leg-metadata rent separately for every leg | Architecture CONFIRMED; metadata schema/rent not yet measured |
| Cooldown reward treatment was open | Explicit next-cycle yield; recovered rent operational; loss is `RecoveryRequired` | Policy CONFIRMED |
| KIF period and repeated carry were previously unfixed | Exact 2,592,000-second Clock periods; repeated total-pool 50/50 rule | Policy CONFIRMED |
| Active KIF division remainder was previously unresolved | Explicit collective carry in `KifSolVault` | Policy CONFIRMED |
| Devnet appeared in historical planning | Current supported non-Mainnet integration target is Testnet | Evidence/decision CONFIRMED |

The Task 0.4 Testnet proof did not exercise production beneficiary economics,
multi-leg cumulative state, multi-source selection, metadata closure, or atomic
multi-leg settlement. Those remain later implementation and validation work.

## 17. Founder decisions and remaining items

### 17.1 Confirmed in this review

- A-001: protected variants and the 1-bps hard-cap policy.
- A-002: permissionless validator discovery/execution, Jito API operational
  preference, strict on-chain validation, atomic candidate failure, and no
  guardian-per-leg approval.
- A-003: confirmed separated custody and dual non-ATA token-vault topology.
- A-004: six logical transaction boundaries.
- A-005: scalable multi-validator round header and temporary leg model.
- P-035: cooldown reward/loss/rent treatment.
- K-006, K-009, and K-010: active remainder carry, repeated zero-active carry,
  and exact KIF Clock periods.
- D-013: founder acceptance of Phase 0 and completion of Task 0.5.

### 17.2 Remaining non-blocking items

- Technical versus policy minimum: the protocol minimum is dynamic and
  mandatory. Confirmed P-025 supplies no extra economic threshold merely for
  caller fees. An extra policy threshold does not exist unless the founder
  supplies a different reason.
- Deterministic stake PDA: resolved for one leg by public Testnet evidence;
  sequence-plus-index orchestration awaits production implementation tests.
- Combined protected withdrawal/deactivation: resolved as feasible for one leg;
  production CU/size must be remeasured after full checks and metadata.
- Distribution dust destination: confirmed to remain in PIV1 and never erode
  principal. The recommended ledger adds it to protected retained value.
- Prolonged inactivity: permissionless operations have no maximum cadence and
  no reward. With no caller, yield accumulates; an active round remains
  resumable. A new automatic expiry/cancellation would conflict with the
  accepted single-active-round/recovery model and is not assumed.
- Caller reward: none.
- DEX/Jupiter/instant exit: REJECTED.
- Devnet/Testnet interchangeability: REJECTED.

Final Program ID, six guardian public keys, and real recipient addresses remain
deferred launch inputs. They must be supplied and verified later, but this
report does not invent them and they do not change the Phase 1 accounting
architecture. The KIF acronym expansion is a branding item and need not block
implementation identifiers. Exact bounded layouts/account sizes, a safely large
technical leg-index representation, metadata rent, worst-case CU/transaction
size, and local/public multi-leg tests are PROVISIONAL implementation work, not
open economics. No economic maximum distribution size is approved.

## 18. Phase 1 entry criteria

**SATISFIED for beginning the separately bounded Task 1.1 scaffold.**

1. The founder reviewed and accepted this report.
2. All seven schema-blocking founder decisions have recorded answers.
3. The separated custody model, shared authority, distinct non-ATA token-vault
   addresses, reusable header, and temporary multi-leg topology are approved.
4. The state predicates, six logical boundaries, permissionless retry model,
   recovery conditions, and pause role are approved.
5. Principal/yield/pending/shortfall/KIF/per-leg rent/reward formulas and
   conservative rounding directions are approved at architecture level.
6. The dynamic-minimum and 0–1-bps slippage policies are approved without a
   hard-coded cluster amount.
7. Validator selection/update policy and API/on-chain trust boundary are
   approved.
8. Accepted dependency/toolchain pins remain the Phase 1 baseline unless a
   separate justified update task is authorized.
9. The decision register, master specification, execution plan, README, and
   this report are synchronized.
10. Task 1.1 still requires its own bounded authorization and has not started.

Phase 1 entry does not authorize Testnet/Mainnet deployment, real funds,
recipient invention, guardian-key creation, upgrade-authority transfer, or Jito
CPI implementation.

## 19. Exact next bounded task

**Task 1.1 — scaffold the modular Anchor workspace and compile-only placeholders
on a new branch from the accepted main baseline.**

Bounded scope after Phase 0 approval:

- create the production workspace/module directory structure;
- preserve the accepted pinned toolchain and lock dependencies exactly;
- add compile-only module and interface placeholders for config, state,
  instructions, events, errors, math, and integrations;
- document the approved account/state boundaries in English comments;
- add no economic implementation, Jito CPI, live account address, recipient,
  guardian key, deployment script execution, or fund-moving test;
- do not begin pure math (Task 1.2), state-transition implementation (Task
  1.3), local mock work, or any Testnet/Mainnet activity;
- validate the scaffold with the accepted pinned build/check commands, commit
  it, report the clean status, and stop.

Program-ID/key handling must be explicitly scoped in that task. No Mainnet key
may be created or stored on this VPS.

## Final Phase 0 safety statement

This task produced documentation only. It performed read-only official Testnet
and Mainnet account inspection. It created no keypair, sent no transaction,
deployed no program, moved no funds, changed no authority, and performed no
state-changing Mainnet action. Mainnet interaction was limited to read-only
public RPC inspection. Phase 1 was not started.
