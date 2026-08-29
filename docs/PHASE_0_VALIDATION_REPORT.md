# PIV1 Phase 0 production architecture validation report

Date: 2026-08-29 UTC

Task: PIV1 Task 0.5

Branch: task/0.5-phase-0-report

Accepted Phase 0 baseline: 2774a7c8b463ae1da03100eca85037585a120ec4

## Status vocabulary and scope

This report uses the task-required evidence labels:

- CONFIRMED BY LIVE TEST: observed on public Solana Testnet or by current read-only public-cluster account inspection.
- CONFIRMED BY LOCAL TEST: exercised against the real cloned programs and accounts in the Task 0.4 local validator.
- PROVISIONAL: a production architecture recommendation that still needs founder approval or production implementation tests.
- OPEN: a decision that cannot be settled from technical evidence alone.
- REJECTED: excluded from production V1.

This report is an architecture proposal, not production code and not a professional
independent audit. It does not authorize Mainnet activity.

## 1. Executive result

**Result: CONFIRMED BY LIVE TEST — Phase 0 is technically sufficient to begin
Phase 1 only after the entry criteria in section 18 are approved.**

The complete custody path was demonstrated on public Testnet:

SOL → JitoSOL → deterministic Stake Program-owned PDA → deactivation → native
SOL in a fixed PIV escrow.

The evidence proves the core protocol compatibility, PDA authority model,
permissionless fee-payer model, current official cluster topology, dynamic
minimum mechanism, rent recovery, fixed-destination finalization, and replay
protection. No confirmed PIV1 requirement is technically contradicted.

The following production choices remain PROVISIONAL or OPEN and must not be
silently converted into founder decisions: slippage tolerance, approval of the
recommended validator/update policy, final vault/token-account form, final
transaction boundaries, cooldown-reward treatment, and the remaining KIF
period/carry details. Section 17 separates unavoidable founder decisions from
items already resolved by evidence or confirmed economics.

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
| [SPL Stake Pool program v2.0.3](https://github.com/solana-program/stake-pool/tree/864ba3c1c564cc270ca62b6e6b558f57538ae092/program) | crate VCS commit 864ba3c1c564cc270ca62b6e6b558f57538ae092; tag program@v2.0.3 | Exact pinned on-chain interfaces, math, validation, and list layout |
| [Agave v4.2.0](https://github.com/anza-xyz/agave/tree/ac82b5d438b0c2303dc7169f52c748977713a111) | ac82b5d438b0c2303dc7169f52c748977713a111 | Accepted runtime/CLI identity and stake behavior |
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

PROVISIONAL production rule: preserve these exact pins through initial Phase 1
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

### 5.1 Recommended account model

The following is a PROVISIONAL exact production proposal. Names and seed bytes
are architectural identifiers, not deployed addresses; no production Program
ID or recipient address is invented here.

| Account | Owner / derivation | Role and invariant |
| --- | --- | --- |
| PivConfig | PIV1 PDA ["config"] | Version, pause, official Jito bindings, recipients, split constants, timing, HWM, sequence, contribution ledgers, KIF configuration, and accounting totals |
| PivAuthority | Address-only PIV1 PDA ["authority"] | Sole token owner and stake staker/withdrawer; signs by invoke_signed only |
| ActiveDistribution | PIV1 PDA ["distribution"] | One reusable active-round record with monotonic sequence; avoids per-cycle program-account rent |
| PendingSolVault | Empty-data System-owned PDA ["pending-sol"] | Native contributions not reconciled; usable first for outgoing allocation; rent floor excluded |
| PrincipalSolQueue | Empty-data System-owned PDA ["principal-sol"] | Reconciled principal SOL waiting for direct Jito deposit; cannot fund beneficiaries except through fixed round accounting |
| OperationalSolVault | Empty-data System-owned PDA ["operational-sol"] | Only permanent operational/rent reserve; advances withdrawal-stake rent; excluded from economics |
| DistributionEscrow | Empty-data System-owned PDA ["distribution-escrow"] | Fixed native-SOL destination and source for the active round; its rent floor is excluded |
| KifSolVault | Empty-data System-owned PDA ["kif-sol"] | Backs aggregate guardian claim liabilities; never mixed with principal or pending contributions |
| PrincipalJitoVault | Legacy Token ATA(authority, JitoSOL) | Reconciled principal JitoSOL |
| PendingJitoVault | Legacy Token ATA(PivAuthority, JitoSOL) | Unreconciled JitoSOL contributions; separate balance and ledger |
| WithdrawalStake | Stake Program-owned PDA ["withdrawal-stake", sequence_le_u64] | Unique temporary stake for one round; PivAuthority is staker and withdrawer |
| GuardianRegistry / rewards | PIV1 PDA(s) | Six keys, activity, carry, claimable/cumulative amounts; bounded fixed set |
| HTFP and Team recipients | Pubkeys stored in config | Fixed writable native-SOL destinations; must be real, non-default, governance-approved addresses |

Recommended ATA choice: use two canonical legacy-token ATAs controlled by the
PivAuthority. An ATA is deterministic and tooling-friendly, but the stake-pool
protocol only requires correctly bound legacy Token accounts. Founder approval
of this final form is still required.

Recommended reusable ActiveDistribution: store the current sequence and
terminal summary in one initialized program-owned account. Config.next_sequence
increments exactly once when a valid snapshot opens. The unique stake PDA still
uses the sequence. This avoids an unbounded permanent-rent series and avoids
requiring permissionless callers to donate round-account rent.

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
  historical position. This prevents a direct transfer to that ATA from
  becoming yield.
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
- PROVISIONAL: the exact multi-vault/dual-ATA form above and reusable round PDA
  require founder approval before schemas are frozen.

## 6. Transaction and account diagrams

### 6.1 Direct SOL contribution and JitoSOL minting

~~~mermaid
sequenceDiagram
    participant C as Contributor
    participant PS as Pending SOL PDA
    participant P as PIV1
    participant QS as Principal SOL PDA
    participant J as Jito stake pool
    participant PJ as Principal JitoSOL ATA
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
    participant PJ as Pending JitoSOL ATA
    C->>P: deposit_jitosol(amount)
    P->>PJ: Token TransferChecked CPI
    P->>P: record actual balance delta as pending
    Note over C,PJ: ATA owner remains a PIV PDA and caller has no custody
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
    J -- Yes --> L[Snapshot obligation and unique sequence then PreparedWithdrawal]
~~~

### 6.4 JitoSOL withdrawal into deterministic stake PDA

~~~mermaid
sequenceDiagram
    participant K as Permissionless caller
    participant P as PIV1
    participant O as Operational SOL PDA
    participant S as Round stake PDA
    participant J as SPL/Jito pool
    participant A as PIV authority PDA
    K->>P: initiate_withdrawal(sequence, candidate)
    P->>O: verify spendable rent reserve
    O->>S: System CreateAccount CPI, both PDAs sign
    P->>J: WithdrawStakeWithSlippage CPI
    J->>S: split delegated stake
    A->>S: fixed staker and withdrawer
    P->>P: verify exact token/stake deltas
~~~

### 6.5 Deactivation and epoch wait

~~~mermaid
sequenceDiagram
    participant P as PIV1
    participant S as Round stake PDA
    participant ST as Stake Program
    participant C as Clock and Stake History
    P->>ST: Deactivate CPI in withdrawal transaction
    ST->>S: deactivation_epoch = Clock.epoch
    loop Keeper readiness checks
        C-->>P: effective / deactivating / inactive stake
    end
    P->>P: ReadyToFinalize only when no effective or deactivating stake
~~~

### 6.6 Finalization into native-SOL escrow

~~~mermaid
sequenceDiagram
    participant K as Permissionless caller
    participant P as PIV1
    participant S as Round stake PDA
    participant ST as Stake Program
    participant E as Fixed SOL escrow
    K->>P: finalize_withdrawal(sequence)
    P->>P: validate state, PDA, authorities, inactivity
    P->>ST: Withdraw entire stake balance CPI
    ST->>E: delegated SOL + rewards + recovered rent
    P->>P: record escrow delta and stake PDA closure
~~~

### 6.7 Settlement and compounding

~~~mermaid
sequenceDiagram
    participant P as PIV1
    participant E as Distribution escrow
    participant O as Operational reserve
    participant H as HTFP recipient
    participant T as Team recipient
    participant K as KIF vault and ledgers
    E->>O: return exact advanced stake rent
    P->>P: derive net outgoing amount and atomic allocations
    E->>H: fixed HTFP native SOL
    E->>T: fixed Team native SOL
    E->>K: fund KIF liabilities
    P->>P: atomically commit compound and HWM accounting
    Note over P,K: Any failure rolls back every beneficiary action
~~~

### 6.8 Pending-contribution integration

~~~mermaid
flowchart LR
    PS[Pending SOL ledger/vault] --> R[Atomic reconciliation]
    PJ[Pending JitoSOL ledger/ATA] --> R
    R --> H[Increase HWM by conservative contribution value]
    R --> QS[Principal SOL queue]
    R --> QJ[Principal JitoSOL ATA/ledger]
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
withdraw bump/PDA, optional authority; System/Token program IDs; ATA mint/owner;
input not reserved for liabilities; expected output and post-CPI balance deltas.

External programs: SPL Stake Pool, which invokes System and Token programs.

#### WithdrawStakeWithSlippage

Ordered SPL metas:

1. stake pool — writable;
2. validator list — writable;
3. pool withdraw authority — read-only;
4. selected validator stake source — writable;
5. deterministic round stake PDA — writable;
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
sufficient withdrawable lamports, preferred-validator rule, round token amount
and slippage floor, token vault owner/mint, unique stake PDA, and residual
principal invariant. The caller cannot choose the destination, authority, token
vault, fee account, or escrow.

External programs: SPL Stake Pool, which invokes Stake and Token programs.

#### System CreateAccount for withdrawal stake

Metas: OperationalSolVault writable signer/payer; deterministic WithdrawalStake
writable signer/new account; System Program. New owner is Stake Program and data
length is StakeStateV2::size_of() (currently 200). Signer seeds:
["operational-sol", bump] and ["withdrawal-stake", sequence_le_u64, bump].
Validate zero lamports/empty data before creation, exact sequence, current Rent
sysvar requirement, operational category balance, and no reused address.

#### Stake Deactivate

Metas: WithdrawalStake writable; Clock read-only; PivAuthority signer; Stake
Program. Signer seeds: ["authority", bump]. Validate stake owner, round binding,
both stake authorities, voter/source record, and round status. Recommended
boundary: same transaction as successful WithdrawStakeWithSlippage; retain a
separate permissionless fallback only if the combined CPI fails after production
remeasurement.

#### Stake Withdraw to fixed escrow

Metas: WithdrawalStake writable; DistributionEscrow writable; Clock and Stake
History read-only; PivAuthority signer; Stake Program. Signer seeds:
["authority", bump]. Validate exact sequence/PDA/status/authorities, fully
inactive effective stake, fixed escrow, whole current balance, pre/post deltas,
stake closure, and single finalization.

#### Token TransferChecked for direct JitoSOL contribution

Metas: contributor source writable; JitoSOL mint read-only; PendingJitoVault
writable; contributor token authority signer; legacy Token program. No PIV PDA
signer is needed. Validate source owner/mint/authority, destination ATA/mint/PIV
owner, mint decimals 9 from decoded mint, amount > 0, and exact destination
delta. The source/authority are the only economically caller-selected accounts.

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
accounts, and both legacy-token ATAs should be initialized before accepting
funds. ATA creation invokes the Associated Token and System programs and must
bind the legacy Token program, JitoSOL mint, and PivAuthority. Exact
initialization funding/authority is a Phase 1/launch design item; it is not
implemented here.

## 8. Recommended production state machine

### 8.1 State representation

PROVISIONAL enum:

- Idle
- PreparedWithdrawal
- CoolingDown
- EscrowFunded
- Settled
- RecoveryRequired

ReadyToFinalize is a derived predicate over CoolingDown plus current Clock/Stake
History, not a keeper assertion. Completed is a terminal round result recorded
immediately before the reusable account returns to Idle. Retryable failures
normally do not need a state: Solana atomic rollback leaves the same safe state.
Pause is an orthogonal config flag that blocks the economic transitions required
by the confirmed pause policy.

### 8.2 Transition matrix

| Transition | Trigger / permission | Preconditions and invariants | Modified accounts | Replay / recovery | Boundary |
| --- | --- | --- | --- | --- | --- |
| Idle → no-yield result | prepare_distribution, anyone | unpaused; no active round; 10 days; all accounts valid/current; historical value ≤ HWM | event only | no snapshot, no clock reset, remain Idle | one tx |
| Idle → valid-insufficient result | prepare_distribution, anyone | positive yield; native shortfall; all validation completed; dynamic Jito amount below protocol minimum | config last_valid_insufficient_at only | 24h blocks another insufficiency evaluation; no snapshot/reclassification; malformed failures roll back | one tx |
| Idle → EscrowFunded (liquid path) | prepare_distribution, anyone | timing; positive yield; checked split; pending SOL fully covers outgoing; fixed recipients/KIF eligibility; no active round | config, ActiveDistribution, PendingSolVault, escrow | sequence opens once; last_prepared_at set now; failed tx changes nothing | one tx |
| Idle → PreparedWithdrawal | prepare_distribution, anyone | same, plus dynamic minimum, operational rent, current candidate feasibility, and residual principal check | config, ActiveDistribution, PendingSolVault, escrow | snapshotted pending SOL moves to fixed escrow; obligation, eligibility, q input, floors, pool quote, and sequence are fixed; candidate may be retried | one tx |
| PreparedWithdrawal → CoolingDown | initiate_withdrawal, anyone | current pool/list; approved slippage; candidate validation; unique empty stake PDA; sufficient rent; no loss of residual-principal invariant | op vault, principal Jito vault, pool/list/source, stake PDA, round | CPI failure/requote leaves PreparedWithdrawal; success consumes round exactly once | one tx: create + withdraw + deactivate |
| CoolingDown → ReadyToFinalize | derived/read-only | stake PDA/authorities match; effective and deactivating stake are zero | none | keepers wait and retry; no trusted timestamp | no tx required |
| ReadyToFinalize → EscrowFunded | finalize_withdrawal, anyone | full inactivity; whole stake balance; fixed escrow; expected state | stake PDA closes, escrow, round | premature call fails atomically; success cannot replay | one tx |
| EscrowFunded → Settled | settle_distribution, anyone | escrow delta reconciled; rent returned; net amounts fit; recipient keys fixed; KIF snapshot fixed; residual HWM safe | escrow, op/KIF vaults, recipients, guardian ledgers, config/round | all beneficiary payments and accounting atomic; failure remains EscrowFunded | one tx |
| Settled → Completed → Idle | integrate_pending, anyone | exact current pending deltas; conservative contribution values; no unpaid round liability | pending/principal ledgers and vaults, HWM, config/round | sequence terminal summary stored; contributions after lock remain pending; failure remains Settled | one tx |
| Idle → Idle compounding | stake_principal_sol, anyone | unpaused; no active liabilities consume input; current pool; slippage floor | PrincipalSolQueue, pool/reserve/mint/fee, PrincipalJitoVault | exact deltas; failure leaves SOL principal queued | one tx |
| Any active state → same state | retry after atomic failure, anyone | same fixed destinations/obligation; refreshed current external accounts | only on success | no lowering caller-selected floors or destination changes | multiple tx over time |
| Any → RecoveryRequired/Paused | 4-of-6 authorized response or upgraded recovery | verified protocol/accounting incident; no normal HWM reduction | config and explicitly governed recovery state | no permissionless cancellation or withdrawal path | governance-specific |

Preparation fixes eligibility, gross split, pending-SOL snapshot used, maximum
JitoSOL input, slippage constraints, proposed compound/HWM delta, KIF bitmap,
and recipient keys. Global HWM is not irrevocably increased during preparation;
the exact proposed delta is stored and committed atomically at settlement. This
preserves the economics while avoiding a partially credited failed cycle.

Recommended transaction boundaries:

1. preparation/snapshot;
2. create stake + protected stake withdrawal + immediate deactivation;
3. full stake finalization into escrow;
4. atomic beneficiary settlement and cycle accounting;
5. pending-contribution integration/completion;
6. later principal-SOL deposit.

Combining withdrawal and deactivation is CONFIRMED BY LIVE TEST. Keeping
preparation, stake finalization, beneficiary settlement, and later compounding
separate is PROVISIONAL and requires founder approval.

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
stake_book_cost = exchange_value(q_round)
stake_native_out = redemption_value(q_round)
protocol_cost = stake_book_cost - stake_native_out
conversion_dust = max(0, stake_gross_budget - stake_book_cost)
post_snapshot_book_increase = max(0, stake_book_cost - stake_gross_budget)

eligible_stake_native = min(stake_native_out, stake_gross_budget)
post_snapshot_native_excess = max(0, stake_native_out - stake_gross_budget)
beneficiary_net_total =
  min(outgoing_gross, pending_sol_used + eligible_stake_native)
total_outgoing_reduction =
  outgoing_gross - beneficiary_net_total
~~~

This charges withdrawal fees and conservative conversion dust only against the
outgoing 80.5% allocation. The compound share and protected principal are not
used to pay them. q_round and the gross budget are fixed at preparation. The
same q is revalued against current validated state before execution. If its book
value has risen, beneficiary funding remains capped at the fixed outgoing gross
allocation; any native excess and other post-snapshot appreciation remain
separately tracked next-cycle yield. Execution also requires the residual-HWM
invariant, so a changed exchange rate cannot silently consume principal or the
compound allocation.

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
compound_from_kif = floor(kif_available / 2)
kif_carry_next = kif_available - compound_from_kif
~~~

Whether prior carry is included repeatedly in kif_available and where
active-guardian division remainder carries are still founder decisions; section
17 does not invent them.

### 9.6 Cooldown rewards and rent

Store at initiation:

- delegated_out = actual stake lamports received from the protected CPI;
- rent_advanced = actual lamports prefunded into the stake PDA;
- escrow_before_finalization.

At finalization:

~~~text
recovered_total = escrow_after - escrow_before
native_after_rent = recovered_total - rent_advanced

cooldown_rewards = max(0, native_after_rent - delegated_out)
cooldown_loss = max(0, delegated_out - native_after_rent)
~~~

The exact rent_advanced is returned from escrow to OperationalSolVault and is
never yield. Cooldown rewards are not eligible for the already-fixed round.
The recommended treatment is to carry them as separately identified
post-snapshot yield for the next cycle; founder approval is required. Any
cooldown loss or residual-HWM failure enters recovery handling and cannot
reduce H normally.

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
R = Rent sysvar minimum_balance(StakeStateV2::size_of())

q_protocol = least positive q such that redemption_value(q) >= D
gross_shortfall_minimum = exchange_value(q_protocol)

operational_spendable =
  OperationalSolVault.balance - OperationalSolVault.permanent_rent_floor

may_open_with_withdrawal only if all are true:
  stake_gross_budget >= gross_shortfall_minimum
  q_round >= q_protocol
  Q_h_available >= q_round
  selected validator can split q_round while retaining its required minimum
  operational_spendable >= R
  minimum_lamports_out >= D
  residual historical assets after the planned burn satisfy the new HWM
~~~

q_protocol must be found with a checked monotone binary search using the exact
pinned fee/redemption functions. R and D come from the runtime/Rent sysvar, not
an RPC-only constant. The selected validator entry, stake balance, status,
derived address, and source stake state also participate in feasibility.

Current point-in-time examples:

| Cluster/snapshot | q_protocol | Fee units | Gross book cost | Net delegated | Stake rent |
| --- | ---: | ---: | ---: | ---: | ---: |
| Task 0.4 Testnet epoch 1018 | 755,045,269 | 755,046 | 1,001,001,002 | 1,000,000,000 | 2,282,880 |
| Testnet epoch 1021 inspection | 753,972,377 | 753,973 | 1,001,001,002 | 1,000,000,000 | 2,077,224 |
| Mainnet epoch 1024 read-only | 770,931,087 | 770,932 | 1,001,001,003 | 1,000,000,001 | 2,282,880 |

These values demonstrate the formula and must not be production constants.

### 10.2 Slippage and transaction-fee relationship

The technical gate uses current expected output and the approved
minimum_lamports_out. A tolerance cannot lower the effective minimum below D or
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
| Operational reserve | Separate current stake-account rent plus permanent vault rent floor; recycled after finalization |
| Founder policy minimum | No extra default is justified by confirmed decisions; any additional threshold is OPEN only if the founder wants a non-caller-fee policy reason |

A valid amount-insufficient evaluation writes only the 24-hour timestamp/event
and succeeds without snapshot, withdrawal, reclassification, 10-day-clock
reset, or active round. Every malformed or otherwise failed transaction rolls
back and cannot extend that cooldown.

## 11. Validator selection and pool updates

### 11.1 Bounded off-chain selection

PROVISIONAL production policy:

1. A permissionless keeper reads the configured pool, validator list, Clock
   epoch, stake accounts, fees, and preferred-withdraw setting.
2. It considers only standard active validator stake entries whose
   last_update_epoch equals the current epoch, status is Active, derived stake
   account exists under the Stake Program, and available lamports safely cover
   the fixed round withdrawal while leaving the source-required minimum.
3. If a preferred withdraw validator is configured and still has withdrawable
   active lamports, it must be selected. SPL Stake Pool rejects another
   validator in that situation.
4. The keeper supplies the list index, vote account, seed suffix, and standard
   validator stake account to PIV1. It may not supply the stake destination,
   token vault, authority, escrow, fee account, mint, pool, or program.
5. If the candidate changes or loses liquidity, the protected CPI fails
   atomically. Another keeper retries the same fixed round amount and
   destination with another valid current candidate.

V1 should not proactively select transient stake or reserve stake. SPL supports
ordered fallback sources, but standard active validator stake was live-tested,
is easiest to bind, and avoids materially more source-state branches. A future
expansion requires separate evidence and approval.

### 11.2 Bounded on-chain validation

The pinned validator list has a 9-byte header/vector prefix and fixed 73-byte
ValidatorStakeInfo records containing active lamports, transient lamports,
last-update epoch, transient seed suffix, unused value, validator seed suffix,
status, and vote address. Production should:

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

Recommended policy:

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
The exact keeper policy needs founder approval, but stale-state rejection and
atomic retry are mandatory.

## 12. Slippage policy

### 12.1 Required calculation

Stake-pool direct operations have no DEX market-price slippage, but they remain
exposed to epoch updates, rewards, fee changes, concurrent transactions, and
stale quotes. Both CPI variants therefore require an on-chain output floor.

Let t be a founder-approved tolerance in basis points, with 0 ≤ t < 10,000:

~~~text
expected_deposit = deposit_net_units(lamports_in)
minimum_pool_tokens_out =
  floor(expected_deposit * (10000 - t) / 10000)

expected_withdraw = redemption_value(q_round)
minimum_lamports_out =
  max(
    runtime_minimum_delegation,
    floor(expected_withdraw * (10000 - t) / 10000)
  )
~~~

The program, not the caller, derives or verifies these values from Config and
the round snapshot. The caller cannot submit a weaker floor. After CPI, exact
source/destination balance deltas must also equal the accepted protocol result.

### 12.2 Snapshotted and current values

Preparation should store:

- pool address/program and pool last_update_epoch;
- T, S, relevant current fee fractions, and quote slot/epoch;
- input lamports or q_round;
- expected output and configured tolerance;
- exact minimum output;
- technical minimum and selected outgoing gross budget;
- residual-HWM proof inputs.

Execution must decode current pool state again. Stale pool/list state is always
rejected. A current quote that still satisfies the immutable stored minimum and
residual-HWM invariant may execute; otherwise it fails without state mutation.

Safe retries refresh current pool/list/candidate accounts while preserving the
round's sequence, maximum token input, fixed destination, eligibility, gross
allocation, and exact stored floor. A retry may never lower that floor or
accept a tolerance above Config merely to force completion. If a protocol
change makes the stored floor permanently unreachable, pause/governed recovery
is required.

### 12.3 Bounded options for founder approval

| Option | Bound | Trade-off |
| --- | --- | --- |
| A — exact | 0 bps | Strongest deterministic bound; more retries when pool state changes between quote and execution |
| B — very tight | Configured 0–1 bps | Small state-drift allowance; must still preserve runtime minimum and residual HWM |
| C — tight configurable | Configured 0–5 bps with an immutable program hard cap approved in advance | More liveness during concurrent updates, but permits a larger outgoing reduction |

No final tolerance is selected here. A broad caller-selected tolerance and the
unprotected basic SPL variants are REJECTED.

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
fees were zero. Current fee fractions are dynamic pool state.

Task 0.4 rents:

- withdrawal stake: 2,282,880 lamports, fully recovered;
- zero-data System vault/escrow: 890,880 each;
- 165-byte legacy token account: 2,039,280 each;
- probe config: 2,596,080;
- probe round: 1,642,560.

Those are observed amounts, not production constants. Current read-only
Testnet stake rent is already different.

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

CONFIRMED safe to combine, subject to production remeasurement:

- withdrawal-stake creation;
- WithdrawStakeWithSlippage;
- immediate Stake Deactivate.

Required to combine:

- all HTFP/Team transfers, KIF-vault funding/credits, and the corresponding
  cycle accounting in one atomic settlement transaction.

Keep separate:

- preparation from external stake withdrawal;
- cooldown wait from finalization;
- full stake withdrawal to escrow from beneficiary settlement;
- pending contribution integration from already-paid settlement;
- principal-SOL Jito deposit from settlement.

These boundaries minimize duplicate-payment risk and make every partial success
recoverable. A versioned transaction/Address Lookup Table may reduce message
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
- initiate_withdrawal and any measured deactivation fallback;
- finalize_withdrawal;
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
- cannot choose the Jito pool, mint, programs, fee account, PIV token vaults,
  withdrawal-stake PDA, escrow, HTFP/Team recipients, KIF liability vault, or
  round sequence.

PIV1 advances only the temporary withdrawal-stake rent from its separately
accounted operational reserve. The entire recorded rent is returned to that
reserve after the stake account closes. Network fees remain external.

Every transition is status- and sequence-guarded. Duplicate calls either fail
without mutation or, for idempotent maintenance/reconciliation checks, observe
that no work remains. No caller-controlled remaining account can redirect
value.

If nobody calls, no maximum deadline is violated: the 10-day rule is a minimum.
Yield and pending contributions continue accumulating. If a round is already
active, its fixed state remains until a keeper resumes it; there is no caller
reward or automatic expiry. A verified protocol incident uses pause and 4-of-6
upgrade/recovery authority, not an invented permissionless escape hatch.

## 15. Threat and failure analysis

| Threat/failure | Prevention | Detection | Recovery |
| --- | --- | --- | --- |
| Wrong pool/program | Config-pinned program/pool; executable/owner checks | decoded key/owner mismatch | atomic failure; governance update only through approved strategy migration |
| Wrong mint/token program | Pool binding, mint owner/decimals/authority/supply, legacy Token ID | decoded mismatch or ATA constraint | atomic failure; pause/migration if official topology changes |
| Wrong validator/list entry | exact list owner/key; checked index/record; derived standard stake PDA | record/vote/seed/status/delegation mismatch | retry another validated current candidate |
| Wrong reserve | require pool.reserve_stake and Stake Program state/authority | key/state mismatch | atomic failure; refresh official pool state |
| Wrong manager fee/referrer | require pool.manager_fee_account and fixed referrer | token mint/key/owner mismatch | atomic failure; no caller substitution |
| Wrong token authority | canonical ATA plus decoded PIV owner/mint | Token account decode and balance-delta check | atomic failure; governance recovery if account corrupted |
| Wrong withdrawal stake PDA | sequence-derived PDA and fixed owner/space | seed/bump/key/owner mismatch | atomic failure |
| Stake-PDA reuse | monotonic sequence; require zero lamports/empty before CreateAccount | pre-create check and System create failure | never reuse; governance recovery only for sequence corruption |
| Round reuse/double withdrawal | one active round, monotonic next_sequence, status guards | sequence/status mismatch; token/stake deltas | reject replay; terminal summary plus closed stake |
| Stale pool/list | require aggregate and selected entry epoch equal Clock | decoded last_update_epoch | permissionless chunk updates then aggregate update/retry |
| Slippage failure | protected variants and Config-derived floor | SPL ExceededSlippage or post-delta mismatch | refresh quote under same bounded policy; never weaken caller-side |
| Validator liquidity change | precheck source residual; SPL source-order/liquidity checks | atomic CPI error | retry same round with another valid candidate |
| Below technical minimum | dynamic binary-search gate before snapshot | q_round < q_protocol or candidate infeasible | record only valid-insufficient 24h timestamp; yield accumulates |
| Insufficient operational reserve | separate balance and runtime rent check | spendable reserve < current stake rent | no snapshot if known; replenish approved operational category and retry |
| Negative/no yield | max(0, historical value − H), no H reduction | checked comparison | no snapshot; wait for recovery or governed migration |
| Mid-round contribution | physical pending vaults plus accounted-unit ledgers | positive unaccounted balance delta | keep pending until post-settlement integration |
| Premature finalization | Stake state, Clock, Stake History, round status | effective/deactivating stake nonzero | atomic failure; wait and retry |
| Replay/double settlement | state transition before/with effects, monotonic sequence | status mismatch | reject; failed settlement leaves EscrowFunded with no partial payments |
| Overflow/underflow | u128 intermediates, checked operations/conversions | explicit arithmetic error | atomic failure; property/boundary tests before deployment |
| Rounding exploitation | all outgoing floors; fee ceiling; exact balance deltas; dust retained | invariant sum checks | atomic failure; dust remains protected |
| Account substitution | no unchecked economic remaining accounts; config/PDA/pool derivations | key/owner/data mismatch | atomic failure |
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

## 16. Differences from the master specification

The master specification is not edited by this task.

| Existing specification text/requirement | Task 0.4 / current evidence | Remains valid? | Required factual correction | Recommended clarification | Founder approval? |
| --- | --- | --- | --- | --- | --- |
| 4.2 direct SOL deposit has “no market slippage” | Protected direct CPI succeeded; current state can still drift | Yes | No DEX market slippage does not mean no output drift | Require DepositSolWithSlippage and current pool | Tolerance yes; protected variant no |
| 4.3 delayed direct withdrawal to stake, deactivate, wait, withdraw SOL | Entire lifecycle succeeded publicly | Yes | None | Destination can be deterministic PDA; same authority can hold both roles | No |
| 4.3 final step combines escrow withdrawal and beneficiary finalization conceptually | Task 0.4 finalized only to fixed escrow | Yes as product goal | Probe did not test beneficiaries | Split escrow funding from atomic beneficiary settlement | Yes, boundary |
| 7.6 new HWM includes contributions, compound, and dust | Probe did not implement economics | Yes | None | Store proposed delta at preparation; commit globally at settlement | Yes, state boundary |
| 9.2 proposed states Idle/PreparedLiquidOnly/WithdrawalRequested/CoolingDown/Ready/Paused | Live test proved more precise custody stages | Concept remains valid | Ready is derivable; fixed escrow-funded and settled retry states are needed | Use section 8 state machine with pause orthogonal | Yes |
| 9.3 prepare may initiate withdrawal | Combined withdrawal/deactivation fit 158,952 CU, but full production checks were absent | Technically possible, not final | Probe CU is not production CU | Separate preparation from withdrawal | Yes |
| 9.4 deactivation may be separate | Immediate deactivation succeeded locally and publicly | Fallback remains valid | Separate deactivation is not a protocol requirement | Combine by default; retain measured fallback | Yes |
| 9.6 finalize may withdraw, pay, reconcile, stake, and close in one transaction | Only withdrawal-to-escrow measured; production fan-out unmeasured | Economic intent valid | One large transaction is not validated | Separate escrow funding, atomic payout, integration, and later staking | Yes |
| 9.7 / P-026 technical minimum was open to measurement | Exact dynamic formula and live boundary now known | Yes, now technically resolved | Threshold is dynamic, not one amount | Use q_protocol plus candidate/rent/HWM gates | Formula approval only |
| 10.2 operational rent reserve | Rent was advanced and fully recovered publicly | Yes | Testnet rent later changed from 2,282,880 to 2,077,224 | Read Rent sysvar each creation; recycle exact recorded advance | No economic decision |
| 10.3 separate stake_pending_sol option | Deposit CPI measured 32,243 CU; combined production finalization not tested | Yes | None | Keep principal-SOL deposit separate | Yes |
| 13.3 PendingSolVault ownership/details needed validation | System-owned empty-data PDA signed successfully | Yes | System-owned form is technically confirmed | Separate pending, principal, operational, and escrow roles | Final form yes |
| 13.7 stake PDA creation/authority was provisional | Deterministic Stake-owned PDA and dual PIV authority succeeded publicly | Yes, resolved | Client-generated keypairs are not required | Unique sequence PDA; caller has no custody | No |
| 18.4 Testnet “atomic final distribution” requirement | Task 0.4 proved only custody to escrow, not PIV beneficiary economics | Still required for later PIV Testnet phase | Task 0.4 must not be described as beneficiary distribution proof | Keep as Phase 4 production-program criterion | No |
| 20 historical DEVNET runbook naming / older Devnet direction | Official supported PIV integration was validated on Testnet | No for current PIV path | Use Testnet; never interchange cluster accounts | Current decision/master already contain the correction | No |
| 24.1 exact delayed-withdrawal minimum open | Dynamic q boundary confirmed and refreshed | Resolved technically | Replace “exact constant” with runtime formula | Keep policy minimum separate | Formula approval |
| 24.2 PDA creation/authority open | Public lifecycle confirms it | Resolved | None | PivAuthority is staker/withdrawer; op PDA funds rent | No |
| 24.3 prepare + withdrawal boundary open | Custody combination measured, full preparation absent | Still open | None | Separate | Yes |
| 24.4 final withdraw + transfers + KIF boundary open | Final withdrawal alone measured 28,453 CU | Still open | None | Separate escrow funding from atomic payout | Yes |
| 24.12 exact operational-rent accounting | Exact advance/recovery observed | Mechanism resolved, value dynamic | Do not hard-code 2,282,880 | Separate vault and recorded rent_advanced | Account-model approval |
| 25 current official integration references | Current Jito docs distinguish Testnet/Mainnet from a separate Devnet deployment | Yes | Devnet addresses are not Testnet/Mainnet addresses | Continue accepted Testnet integration target | No |

## 17. Open founder decisions

### 17.1 Decisions required before Phase 1 schemas are frozen

| Status | Decision | Evidence-bounded recommendation |
| --- | --- | --- |
| OPEN | Production slippage tolerance | Choose section 12 option A, B, or C; mechanism and protected variants are already resolved |
| OPEN | Final validator selection and pool-update policy | Approve standard-active off-chain selection, O(1) PIV validation, direct permissionless four-entry SPL updates, and atomic candidate retries |
| OPEN | Exact production vault/token-account form | Approve separate pending/principal/operational/escrow/KIF System PDAs, dual legacy-token ATAs, and reusable ActiveDistribution PDA |
| OPEN | Transaction boundaries | Approve the six boundaries in section 8; combined withdraw/deactivate is technically resolved |
| OPEN | Cooldown stake rewards | Recommended: exclude from fixed round and carry as explicitly tracked next-cycle yield; do not silently compound |
| OPEN | KIF period and repeated zero-active carry | Confirm or change provisional 30-day heartbeat period and whether every zero-active period applies 50/50 to new allocation plus prior carry |
| OPEN | Active-guardian KIF division remainder | Recommended: keep as explicit KIF carry; never give it to an inactive guardian or arbitrary recipient |

### 17.2 Evaluated items that are not open

- Technical versus policy minimum: the protocol minimum is dynamic and
  mandatory. Confirmed P-025 supplies no extra economic threshold merely for
  caller fees. An extra policy threshold does not exist unless the founder
  supplies a different reason.
- Deterministic stake PDA: resolved by public Testnet evidence.
- Combined protected withdrawal/deactivation: resolved as feasible; only the
  production transaction-boundary approval remains.
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
implementation identifiers.

## 18. Phase 1 entry criteria

Production scaffolding may begin only when all are true:

1. The founder has reviewed and accepted this report.
2. Every section 17.1 decision needed for schemas/interfaces has a recorded
   founder answer.
3. The final account model, ownership, PDA seed namespace, token-account form,
   and initialization funding model are approved.
4. The state enum, transition boundaries, retry/recovery model, and pause
   interaction are approved.
5. Principal/yield/pending/shortfall/KIF/rent/reward formulas and every rounding
   direction are approved.
6. Dynamic-minimum and slippage policies are approved without a hard-coded
   cluster amount.
7. Validator selection/update policy is approved.
8. The accepted dependency/toolchain pins remain confirmed, or a separate
   justified pin-update task has completed.
9. No unresolved contradiction exists between the approved architecture,
   decision register, master specification, and execution plan.
10. The next task explicitly authorizes production scaffolding and nothing
    beyond it.

Phase 1 entry does not authorize Testnet/Mainnet deployment, real funds,
recipient invention, guardian-key creation, upgrade-authority transfer, or Jito
CPI implementation.

## 19. Exact next bounded task

Proposed next task: **PIV1 Task 1.1 — scaffold the modular Anchor workspace** on
a new branch such as task/1.1-anchor-scaffold.

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
