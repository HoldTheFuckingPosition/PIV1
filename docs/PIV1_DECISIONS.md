# PIV1 Decision Register v0.2

**Project:** HTFP Project  
**Component:** PIV1 - Perpetual Income Vault 1  
**Date:** 2026-09-03
**Purpose:** Compact source of truth for the dedicated PIV1 development chat and Codex.

## Status legend

- **CONFIRMED**: explicit current founder decision.
- **PROVISIONAL**: current implementation direction, subject to validation.
- **OPEN**: unresolved or must be measured before final implementation.
- **HISTORICAL**: older direction, no longer current.
- **REJECTED**: explicitly abandoned.

## Product and economic decisions

| ID | Status | Decision |
|---|---|---|
| P-001 | CONFIRMED | PIV1 is the first production infrastructure brick of the HTFP Project. |
| P-002 | CONFIRMED | PIV1 has its own dedicated Solana Program ID, isolated from future PIVs. |
| P-003 | CONFIRMED | Principal is intended to remain locked perpetually. Deposits create no individual withdrawal, ownership, or reward rights. |
| P-004 | CONFIRMED | PIV1 accepts SOL and JitoSOL. |
| P-005 | CONFIRMED | PIV1 uses JitoSOL as its initial staking asset. |
| P-006 | CONFIRMED | PIV1 uses direct Jito stake-pool deposit and delayed direct withdrawal. Jupiter/DEX swaps are excluded from the core V1. |
| P-007 | CONFIRMED | All beneficiary outputs are in SOL. |
| P-008 | CONFIRMED | Yield split: 59% HTFP SOL reserve; 19.5% compound; 19.5% Team Owner Pool; 2% active KIF guardians. |
| P-009 | CONFIRMED | Distribution preparation is permissionless. The caller pays Solana transaction fees and receives no automatic caller reward. |
| P-010 | CONFIRMED | Minimum interval between distribution preparations: 10 days. It is a minimum, not a maximum. |
| P-011 | CONFIRMED | The 10-day clock starts when the distribution snapshot/preparation is successfully recorded, not when final payment occurs. |
| P-012 | CONFIRMED | Only one distribution may be active. A new one cannot begin until the previous one is finalized or recovered through an authorized upgrade. |
| P-013 | CONFIRMED | SOL received from outside is used first to fund outgoing distributions; only the missing amount is withdrawn from JitoSOL. |
| P-014 | CONFIRMED | Any unused external SOL contribution becomes principal after reconciliation. Economically, the entire valid contribution increases principal; using liquid SOL for payment reclassifies an equivalent amount of historical yield left staked into principal. |
| P-015 | CONFIRMED | Direct JitoSOL contributions are incorporated into principal at the next snapshot/reconciliation, not retroactively treated as historical yield. |
| P-016 | CONFIRMED | Contributions received while a distribution is active remain in separate pending queues until finalization/reconciliation. |
| P-017 | CONFIRMED | Principal accounting and high-water mark are denominated in SOL lamports. |
| P-018 | CONFIRMED | Yield is measured using the official Jito stake-pool exchange/redemption accounting, not a DEX market price. |
| P-019 | CONFIRMED | High-water mark is not reduced after a loss. No distribution occurs until the protected principal value is recovered. |
| P-020 | CONFIRMED | All distributable calculations use integer arithmetic and conservative floors. Unrepresentable fractions never leave PIV1. |
| P-021 | CONFIRMED | Rounding dust remains non-distributed value and is reconsidered conservatively in a later cycle. It must never cause principal erosion. |
| P-022 | CONFIRMED | Network transaction fees are paid by the transaction fee payer, never reimbursed by PIV1. |
| P-023 | CONFIRMED | Jito withdrawal/protocol costs reduce the outgoing beneficiary allocation, not protected principal or the 19.5% compound share. |
| P-024 | CONFIRMED | Final beneficiary payment is atomic: all beneficiary accounting and payments succeed together or none do. |
| P-025 | CONFIRMED | No arbitrary economic minimum is required merely to protect a caller from fees; the caller chooses whether the transaction is worthwhile. |
| P-026 | CONFIRMED | The delayed-withdrawal technical minimum is dynamic. It is derived from current Rent, the Stake Program minimum delegation, current stake-pool fees/exchange accounting, source residual rules, and checked integer floors; no cluster amount is hard-coded. |
| P-027 | CONFIRMED | No permanent economic SOL reserve is maintained. Only rent-exempt/account-operational balances may remain. |
| P-028 | CONFIRMED | No deposit-cap is planned for launch. |
| P-029 | CONFIRMED | If a valid distribution attempt is below the technical Jito-withdrawal minimum, no snapshot or withdrawal is created; yield continues accumulating. |
| P-030 | CONFIRMED | After a valid insufficient attempt, another insufficiency evaluation is blocked for 24 hours. This retry cooldown does not reset the 10-day distribution clock. |
| P-031 | CONFIRMED | Malformed or otherwise failing transactions must not update the 24-hour retry cooldown. |
| P-032 | CONFIRMED | PIV1 V1 allows only one active prepared distribution; parallel distributions are excluded. |
| P-033 | CONFIRMED | PIV1 maintains a small operational SOL reserve solely to pre-fund rent-exempt temporary withdrawal accounts. Recovered rent returns to and is recycled by this reserve. |
| P-034 | CONFIRMED | The operational rent reserve is excluded from principal, yield, contribution, and beneficiary accounting. |
| P-035 | CONFIRMED | Cooldown rewards are excluded from the fixed active distribution and recorded explicitly as next-cycle yield for the normal later split. They are not principal. Recovered stake-account and temporary-metadata rent returns to the operational category and is not yield. A cooldown loss enters `RecoveryRequired` and never lowers the HWM normally. |

## KIF guardian decisions

| ID | Status | Decision |
|---|---|---|
| K-001 | CONFIRMED | Six guardian keys exist; each key corresponds to one KIF guardian. |
| K-002 | CONFIRMED | Governance threshold is 4 of 6 for pause, upgrades, key changes, recipient changes, and other program modifications. |
| K-003 | CONFIRMED | Initially, the founder may control all six wallets. Guardians will later be rotated to independent people/entities. |
| K-004 | CONFIRMED | Future guardian rotation occurs by replacing a public key in the multisig. Private keys/seed phrases are never transferred to another person. |
| K-005 | CONFIRMED | Only guardians active in the applicable KIF period earn that period's KIF allocation. Inactive guardians earn nothing and receive no retroactive compensation. |
| K-006 | CONFIRMED | When at least one guardian is active, the full available KIF allocation, including approved prior carry, is divided equally among active guardians with conservative floors. The division remainder stays in `KifSolVault` as explicit collective KIF carry; it is not assigned preferentially or moved to another economic category. |
| K-007 | CONFIRMED | KIF earnings are credited to individual claimable balances; guardians may claim later. |
| K-008 | CONFIRMED | If zero guardians are active, 50% of the available KIF allocation is compounded into principal and 50% is carried into the next KIF allocation. |
| K-009 | CONFIRMED | In every successive zero-active period, apply the 50/50 rule again to the entire available KIF pool: current net KIF allocation plus all approved prior carry. The compounded floor permanently increases protected principal; the remainder remains collective KIF carry and creates no inactive-guardian claim. |
| K-010 | CONFIRMED | A KIF activity period is exactly 2,592,000 seconds (30 days). A configuration anchor and Solana Clock `unix_timestamp` define monotonic period IDs with half-open boundaries `period_start <= timestamp < period_end`. A valid guardian heartbeat or qualifying governance vote counts in its current period; post-snapshot activity is not retroactive, and no off-chain database determines eligibility. |
| K-011 | OPEN | Exact expansion/meaning of the acronym KIF. The identifier may remain unexplained in code until branding is confirmed. |
| K-012 | CONFIRMED | `claim_kif` remains allowed while the global emergency pause is active because it pays only an already-earned recorded liability. A claim uses only the isolated `KifSolVault`; requires the corresponding guardian signer or an explicitly configured guardian-controlled payout destination that an arbitrary caller cannot select; cannot exceed the guardian's stored claimable balance; and deducts the same amount exactly once from both that balance and the global KIF claim liability. Overclaim, replay, wrong guardian, wrong destination, wrong vault, liability mismatch, insufficient vault balance, or arithmetic failure rejects atomically. A claim cannot create rewards, change eligibility or history, modify an active distribution, reduce the protected HWM, consume principal, pending contributions, distribution escrow, or operational rent, or access another guardian's allocation. Global pause continues to block every Task 1.3 distribution-economic transition. The handler remains unimplemented and belongs to separately authorized Phase 2 work. |

## Production architecture decisions

| ID | Status | Decision |
|---|---|---|
| A-001 | CONFIRMED | Production uses only `DepositSolWithSlippage` and `WithdrawStakeWithSlippage`. Initial tolerance is 1 bps; Config may allow 0–1 bps; the immutable program hard cap is 1 bps; a caller cannot weaken the derived floor. A value above 1 bps requires a reviewed program upgrade. Current-pool validation, the stored round floor, exact post-CPI deltas, dynamic technical minimums, and the residual-HWM invariant remain mandatory. Basic unprotected variants are REJECTED. |
| A-002 | CONFIRMED | Validator discovery and every withdrawal leg are permissionless; no whitelist, caller reward, fee reimbursement, or caller custody exists. The official keeper queries Jito's current Preferred Withdraw Validator List API and uses the minimum necessary number of recommended sources. Any on-chain preferred withdraw validator is respected and exhausted under the pinned SPL source-order rules before another validator source. Every candidate must pass current pool/list, checked-record, derived-address, stake-state, source-order, residual, minimum, mint, authority, liquidity, slippage, and HWM checks. API preference is an operational policy, not an on-chain-provable invariant; another caller may choose a different candidate only if all enforceable checks pass. A failed attempt is atomic. Guardians govern policy and pause incidents, not individual validators or legs. |
| A-003 | CONFIRMED | Production custody uses `PendingSolVault`, `PrincipalSolQueue`, `OperationalSolVault`, `DistributionEscrow`, `KifSolVault`, distinct `PrincipalJitoVault` and `PendingJitoVault`, shared decoded token authority `PivAuthority`, and reusable `ActiveDistribution`. Each JitoSOL vault has a different PIV1-derived account-address PDA, is initialized as a 165-byte legacy SPL Token account owned by the legacy Token Program, is bound to official JitoSOL, is controlled by `PivAuthority`, and is not an ATA. Economic categories remain physically and logically separated. |
| A-004 | CONFIRMED | The production lifecycle has six logical boundaries: preparation/snapshot; protected withdrawal plus immediate deactivation; inactive-stake finalization to fixed escrow; atomic beneficiary settlement/accounting; pending-contribution integration; and later principal SOL/Jito compounding deposit. These are not a promise of exactly six transactions: a multi-validator round repeats leg transactions inside boundaries 2 and 3. Unproven later stages remain separate. |
| A-005 | CONFIRMED | One active distribution may use multiple deterministic withdrawal legs. `ActiveDistribution` is a bounded reusable cumulative header; each successful leg uses temporary `WithdrawalLeg` metadata and a Stake Program-owned `WithdrawalStake`, both derived from `(round_sequence, leg_index)`. The target must be assigned exactly using the supplied candidate's maximum safe capacity, with no caller-selected micro amount. Per-leg fee, burn, output, slippage, rent, reward/loss, finalization, and replay state roll into checked cumulative counters. Settlement is forbidden until the exact target is assigned, every successful leg is finalized, and escrow/accounting reconcile. The one-leg public Testnet proof validates an individual leg; multi-leg orchestration remains architecture-confirmed, not live-tested. |

## Governance and authority decisions

| ID | Status | Decision |
|---|---|---|
| G-001 | CONFIRMED | PIV1 remains upgradeable under a 4/6 multisig; guardians are intentionally allowed to modify the full program when necessary. |
| G-002 | CONFIRMED | Squads is the preferred multisig implementation unless Phase 0 finds a concrete incompatibility. |
| G-003 | CONFIRMED | An explicit emergency pause instruction is included despite full upgradeability. |
| G-004 | CONFIRMED | Pause blocks new snapshots, Jito deposits/conversions, delayed withdrawals, finalizations, and migrations. Incoming direct transfers may still arrive and remain pending. |
| G-005 | CONFIRMED | Upgrades and pause may execute immediately after reaching 4/6 approval; no mandatory public timelock is required. |
| G-006 | CONFIRMED | Temporary mainnet recipient treasuries are allowed before HTFP Vault and Team Owner Pool exist. They must be real controlled addresses, not null/empty addresses. |
| G-007 | CONFIRMED | Recipient addresses can later be replaced under 4/6 authority. |
| G-008 | CONFIRMED | Mainnet program upgrade authority must ultimately be held by the 4/6 Squads multisig, not a single hot wallet. |

## Development and launch decisions

| ID | Status | Decision |
|---|---|---|
| D-001 | CONFIRMED | Development is performed with Codex on a Linux VPS. |
| D-002 | CONFIRMED | Mainnet keys, guardian seed phrases, and final deployment signing secrets must not be stored in the repository or given to Codex. |
| D-003 | CONFIRMED | Rust + Anchor is the default stack unless a documented technical reason requires native Solana Rust for a specific layer. |
| D-004 | CONFIRMED | Tests include Rust unit/property tests and TypeScript integration tests. |
| D-005 | CONFIRMED | Local testing uses a controllable mock stake pool/Jito adapter before real cluster integration. |
| D-006 | CONFIRMED | Integration then moves to Solana Testnet using the current officially supported Jito Testnet deployment. Devnet is rejected for the Jito integration unless future official support and current on-chain state are reverified. |
| D-007 | CONFIRMED | Mainnet comes only after local and Testnet validation, adversarial review, verified/reproducible build work, recipient verification, and explicit founder approval. |
| D-008 | CONFIRMED | Initial mainnet capital is expected to be approximately 1-2 SOL, but this is not an on-chain cap. |
| D-009 | CONFIRMED | Code is published by public launch/mainnet launch and must support independent build verification. |
| D-010 | CONFIRMED | AI and community reviews are used, but the project must not claim an independent professional audit unless one actually occurs. |
| D-011 | CONFIRMED | CLI comes before the public dashboard. |
| D-012 | CONFIRMED | A public dashboard is planned, potentially as part of the later HTFP website. |
| D-013 | CONFIRMED | The founder reviewed and accepted the Phase 0 report, the corrected dual-token-vault topology, and scalable multi-validator V1 architecture. Task 0.5 is complete and Phase 1 entry criteria are satisfied for the separately bounded Task 1.1 scaffold. At the time of Phase 0 acceptance, Task 1.1 had not started. |
| D-014 | CONFIRMED | The founder accepted Task 1.1 implementation commit 1d436570570fc31310e3e5d2c1d4d5e92320c65b. Task 1.1 is complete. Task 1.2 has not started; the exact next task is the pure math crate and requires separate authorization and a dedicated branch. |
| D-015 | CONFIRMED | The founder separately authorized Task 1.2 only on the dedicated `task/1.2-pure-math-crate` branch from accepted baseline e6d04530ccfa65ca3a204fcfcb15d37033317654. Its implementation is pending founder acceptance. Task 1.3 and Task 1.4 have not started; the exact next action is founder review of Task 1.2. |
| D-016 | CONFIRMED | The founder accepted Task 1.2 implementation commit 43a3b7497653ff7a246a1e5cf9b760086dd33fcd after independent review found no correction required. Task 1.2 is complete. Task 1.3 and Task 1.4 have not started; the exact next task is Task 1.3, the state and transition model, which requires separate founder authorization and a dedicated branch. |
| D-017 | CONFIRMED | The founder separately authorized Task 1.3 only on the dedicated `task/1.3-state-transition-model` branch from accepted baseline `055c93eebd8cde2d2efac593a8d1f0aaacc949d4`. The bounded state and pure transition model is implemented and pending founder acceptance. Task 1.4 has not started; the exact next action is founder review of Task 1.3. |
| D-018 | CONFIRMED | The founder accepted the complete corrected Task 1.3 implementation: initial commit `33978cf3eda918e4c438b80ed0e12a47b8347519`, final accepted tip `527e381661fe0cfc27e07ad9b44e1601a638ae75`. The four review areas are resolved: prior-cycle yield carry enters the checked historical-value basis before the protected-HWM comparison; settlement derives capped HTFP/Team/KIF amounts with weights `5,900 / 1,950 / 200` over `8,050` and protects residual dust exactly once (`1,800` net gives `1,319 / 436 / 44` and `1` dust); pause gates every Task 1.3 distribution-economic transition through settlement and pending integration/completion; and valid technical insufficiency requires a complete checked positive-shortfall proof. The five serialized layouts and planned spaces are unchanged. Future external-account/handler validation remains deferred, and future `claim_kif` pause behavior remains OPEN. Task 1.4 has not started; its randomized/property and adversarial invariant testing requires separate founder authorization and a dedicated branch. |
| D-019 | CONFIRMED | The founder separately authorized only Task 1.4 on `task/1.4-property-invariant-tests` from accepted `main` baseline `8a512656fc78eff17d2473e6fc37a08e4b77db4d`. Reproducible deterministic property, adversarial model-state, and serialization/layout tests are implemented without a new dependency or account-layout change and are pending founder acceptance. No production correction was required. The future `claim_kif` pause policy remains OPEN. The exact next action is founder architecture review; Phase 2 has not started and requires separate authorization. |
| D-020 | CONFIRMED | The founder accepted Task 1.4 implementation commit `06c39429f3237f6974e21217670c3f0d30b0a571` and the complete Phase 1 specification-as-code foundation. The accepted evidence uses independent deterministic math and state-machine oracles, boundary corpora, coverage guards, rejection atomicity, and all five serialized schemas; it required no production correction, account-layout change, or dependency. This pure-state/property evidence does not replace later handler, CPI, localnet, external-account, Testnet, or Mainnet validation. K-012 resolves claims during pause, while the zero-sized `ClaimKif` marker remains unimplemented. Phase 2 has not started and requires separate founder authorization; the exact next action is scoping and authorization of its first bounded mock/localnet task. This AI-assisted review is not a professional independent audit. |

## Historical/rejected directions

| ID | Status | Direction |
|---|---|---|
| H-001 | HISTORICAL | Native staking as PIV1 V1. It remains a possible future migration option. |
| H-002 | HISTORICAL | Five guardians with 3/5 LST changes and 4/5 major changes. Replaced by six guardians and 4/6 authority. |
| H-003 | HISTORICAL | Fully immutable PIV1 after launch. Replaced by full 4/6 upgrade authority. |
| H-004 | REJECTED | Jupiter/DEX swap as the normal JitoSOL-to-SOL exit path in V1. |
| H-005 | REJECTED | Transferring an existing guardian wallet's private key to a new guardian. |
| H-006 | REJECTED | NFT-based guardian seats for PIV1 V1. |
| H-007 | REJECTED | Paying inactive guardians retroactively. |
