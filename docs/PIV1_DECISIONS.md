# PIV1 Decision Register v0.2

**Project:** HTFP Project  
**Component:** PIV1 - Perpetual Income Vault 1  
**Date:** 2026-09-08
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
| D-021 | CONFIRMED | The founder separately authorized only Task 2.1, the narrow deterministic stake-pool interface and fixed-capacity host-only mock, on `task/2.1-mock-stake-pool-adapter` from accepted `main` baseline `7626c0bc0e46ab7437162be7939e01c1c6eff619`. Task 2.1 is implemented and pending founder acceptance. It adds no handler, account context, CPI, local validator, live address, Program ID, deployment, or fund movement and does not claim exact SPL/Jito behavior; real pinned SPL Stake Pool 2.0.3 mapping remains Phase 3 work. Phase 2 is in progress. Task 2.2 and later Phase 2 tasks have not started; the exact next action is founder review of Task 2.1. |
| D-022 | CONFIRMED | The founder accepted the complete corrected Task 2.1 implementation: initial commit `33b1e539f969432f82635d1ca76c59d89f0ec233` and final corrected tip `cb90d468eff4dce60552ba15b2b267b364a47827`. The accepted result covers the synchronous statically dispatched seven-method `StakePoolAdapter`, checked bounded value/error types, informational quotes separated from independently derived executions, exact snapshot-identity equality, the immutable one-basis-point maximum slippage, maximum-safe per-source withdrawal intent, dynamic technical minimum, delayed lifecycle states, explicit reward/loss/stake-rent/metadata-rent categories, replay protection, rejection atomicity, the fixed-capacity host-only `MockStakePool`, deterministic fixed/randomized tests, unique internal zero fee `0/1`, rejection of every other zero-numerator fraction, and future translation of validated external SPL zero-fee encoding to `FeeFraction::ZERO`. The mock is not exact production SPL Stake Pool 2.0.3/Jito behavior and is not exported by the production library; no handler, account context, CPI, Program ID, local validator, RPC operation, deployment, or fund movement exists. `PoolSnapshotIdentity.revision` remains deterministic mock state, while its collision-safe account-derived production representation and all exact real-protocol mappings remain Phase-3-provisional. Phase 2 remains in progress; Task 2.2 and later tasks are not started. The exact next action is separate founder authorization of Task 2.2. This AI-assisted review is not a professional independent audit. |
| D-023 | CONFIRMED | The founder separately authorized only Task 2.2, deterministic contribution intake and pending-queue accounting, from accepted baseline `a4cf7cdcabee7965e81525894627e8f5fc138e03`. Task 2.2 is implemented and pending founder acceptance. Its boundary is pure observation-based accounting for explicit SOL/JitoSOL contributions and idempotent unexplained-balance reconciliation in the two dedicated pending vaults, plus a fixed-size host-only custody mock and deterministic tests. It adds no serialized field, handler, transfer, CPI, Program ID, account derivation, live custody behavior, pending integration, HWM update, or fund movement. Explicit-deposit handler callability during pause remains PROVISIONAL; safe accounting of already-received incoming value remains recordable. Liability-aware reconciliation of principal, operational, distribution-escrow, and KIF custody is still deferred; PROVISIONAL Task 2.3 is the recommended later bounded task, but it is not authorized or started. The exact next action is founder review of Task 2.2. |
| D-024 | CONFIRMED | The founder accepted Task 2.2 implementation commit `e3233b96b533a620e8037d5231baede10877217f` within its documented pure-accounting and host-mock scope. The acceptance covers pure explicit SOL/JitoSOL contribution accounting, dedicated pending-vault reconciliation, checked arithmetic and custody-deficit rejection, idempotence, atomic updates, active-distribution and non-pending-accounting preservation, and the recorded deterministic test evidence. Observations still require future fixed-account and transfer validation; explicit-transfer handler callability during pause remains PROVISIONAL; no real custody, handler, CPI, or localnet behavior is proven; and all-vault normalization remains deferred. Future composition tests must cover pending SOL moved into distribution escrow, distinguish remaining physical pending custody from contribution value awaiting HWM integration, and prevent both double counting and false custody deficits. This acceptance selects no new accounting policy for those deferred cases. Phase 2 remains in progress; Task 2.3 and later tasks are not started. The exact next action is separate scoping and authorization of Task 2.3. |
| D-025 | CONFIRMED | The founder separately authorized bounded Task 2.3 from accepted baseline `66193769d1cbc59cd8630df295b9a784b9c64642` on `task/2.3-vault-reconciliation-model`. It is IMPLEMENTED / PENDING FOUNDER ACCEPTANCE for pure custody derivations and atomic host composition of Task 1.3/2.2 accounting with the Task 2.1 mock. Recognized pending SOL retains full contribution value until HWM integration; only a valid committed active round supplies the physical pending-SOL offset. Proven economic-vault excess is normalized into dedicated pending custody once, preserving historical assets, active obligations, earned KIF liabilities/carry and rent. This implements the already-confirmed contribution treatment, not a new economic destination. Operational surplus derivation remains unsupported without an authenticated funding baseline; real account/transfer validation, exact SPL/Jito mapping, handlers, CPI and localnet remain deferred. Serialized fields, math, dependencies and toolchain pins are unchanged. Task 2.4 and later work are NOT STARTED. The next action is founder review of the committed branch, not publication, acceptance integration or later work. This AI-assisted engineering is not a professional independent audit. |

## Activated technical coordination workflow

### D-026 — CONFIRMED — Technical pilot through founder Testnet testing

Activated in the connected Codex session on **2026-09-08 UTC**; the supplied
mandate itself is labelled **2026-09-09**. Its activation is the founder's
explicit instruction to apply it, not its preparation date. The complete text
is preserved in [PIV1_TECHNICAL_PILOT_MANDATE.md](PIV1_TECHNICAL_PILOT_MANDATE.md).

The pilot may coordinate successive bounded PIV1 implementation, corrective,
review, pinned-dependency, local-testing and normal-commit tasks toward founder
Testnet testing without renewed routine technical approval. This replaces the
earlier review-only scope and per-task fresh-permission/stop workflow; D-025
and older authorizations remain historical records of their original scope.
One writer and separate actual review must produce inspected evidence and an
updated [pilot checkpoint](PIV1_PILOT_STATE.md) for every completed task.

Reviewed development progresses on `integration/piv1-testnet`; technical status
may be `TECHNICALLY VALIDATED / PENDING FOUNDER ACCEPTANCE`, never inferred
founder acceptance. `main` stays at its actual founder-accepted state. Normal
publication of clean reviewed PIV1 development branches to the existing remote
is authorized after targeted secret/generated-file and hook/CI checks. No
force push, history rewrite, automatic release/tag or unrelated publication.

Confirmed economics, governance and custody powers are unchanged. The founder
retains material economic/security decisions and final acceptance. No Mainnet,
real-value movement, unrelated secrets access, new key creation or signing is
authorized. Before any new public-Testnet deployment or fund-moving lifecycle,
prepare the mandate's exact cluster/identity/artifact/budget/operation approval
card and obtain explicit approval. Authority transfers need separate approval.
Tool access is not authority to expand these boundaries.

### D-027 — CONFIRMED — Acceptance of the Tasks 2.3–2.19 foundation milestone

On **2026-09-14 UTC**, the founder explicitly stated (English translation):

> I accept the 2.3–2.19 milestone at commit d9f3371 and authorize its integration into main.

The accepted commit is `d9f3371be6ecb586675e3b38edcc57bd6e9519f8` on
`integration/piv1-testnet`. Tasks 2.3–2.19 are **COMPLETE / FOUNDER-ACCEPTED**
within the bounded scopes recorded in the
[integration review](PIV1_INTEGRATION_REVIEW_2_3_TO_2_19.md) and individual task
reports, including T23-R1. This explicitly supersedes their pending-acceptance
status and the recap request's no-main-integration restriction. Earlier task
reports and the prepared PR body remain historical evidence; no PR was created.

The authorization covers normal fast-forward integration into `main` from
`66193769d1cbc59cd8630df295b9a784b9c64642` of the accepted milestone plus necessary
reviewed acceptance-record documentation. It permits no unrelated source work,
force push or history rewrite. Root completed the normal main fast-forward on
2026-09-14 and independently verified remote main/integration at the accepted
commit; the Task 2.3 branch
remained unchanged. Necessary acceptance records are maintained in separately
reviewed documentation commits. The pilot checkpoint and Git record the current
publication identity.

Acceptance retains **416 host tests +1 doctest/eight gates** on the verified
unchanged source and the separate **historical Task 2.14 artifact's 24 local
SBF tests/70 cases**. No new test run or current-source SBF proof is claimed.
The accepted foundation does not establish a complete PIV1 lifecycle or Testnet
readiness. Actual genesis/protocol composition, prefunding-safe creation,
initializer transport, production protocol/governance/activity handlers and
refreshed runtime evidence remain deferred. Task 2.20 has not started.

No economics, custody powers, governance threshold or live-operation scope
changes. D-026 continues to govern bounded technical progression. Mainnet,
deployment, funds, keys/signing and authority transfers retain their existing
separate approval gates. AI-assisted review is not a professional independent audit.

### D-028 — CONFIRMED — Conditional main integration of Tasks 2.20–2.23

On **2026-09-19 UTC**, the founder instructed (English translation):

> If everything is good, publish to main on GitHub please.

This authorizes normal fast-forward integration from
`5a067ad34d2f2ab0a27f2c6bff312a8b1f772d24` of the reviewed Tasks 2.20–2.23
implementation at `3282e1ebabcb0cd88491d48a391565b8b100afa7`, plus its necessary
reviewed authorization-record documentation. It applies the mandate's existing
exception: "unless a newer explicit founder instruction authorizes integration."
Status is **TECHNICALLY VALIDATED / FOUNDER-AUTHORIZED MAIN INTEGRATION**;
no broader unconditional founder acceptance is inferred.

Root verified the four-commit ancestry, all 94 source/manifest inputs and 20
retained logs. Separate source review and publication safety checks passed.
The prior 448 host tests +1 doctest/eight gates and nine Node tests/eight transport
cases are retained evidence, **not rerun for this documentation-only publication**.
Final documentation review precedes the normal main/integration publication;
Git and the [checkpoint](PIV1_PILOT_STATE.md) record its identity and evidence.
No history rewrite, unrelated work, new economics, Testnet readiness, deployment,
fund movement, keys/signing or authority transfer is authorized. D-026's sensitive-
action gates remain unchanged. Task 2.24 is NOT STARTED; STOP after publication.

### D-029 — CONFIRMED — Conditional main integration after Task 2.32 validation

On **2026-09-26 UTC**, the founder resumed technical work and instructed
(English translation):

> Let's move forward, and if everything works well, commit to main please.

This authorizes normal fast-forward integration of the reviewed development
sequence after the bounded Task 2.32 validation and separate review pass. The
verified starting integration is `60193d63b64f42211f98d9b4910dfc047ab876df`
(Tasks 2.24–2.31); main is `7b74be4b13c019b96a0c8abcbebfbcc361d31089`.
Task 2.32 targets keyless local full-genesis initialization using the three
Task 2.31 validation artifacts. Root must verify the complete intervening diff,
applicable actual/retained evidence, final source review, publication safety and
unchanged protected refs before publication. Git and the pilot checkpoint will
record the exact resulting implementation and publication identity.

Task 2.32 subsequently passed 13 runtime cases and 13 runner regressions. Root
and separate review independently checked 2040 complete account records against
the exact unchanged Task 2.31 artifacts. The first build and first runtime failed
on two test-harness assumptions; reviewed test-only corrections preserved the
independent oracles, and both failed attempts remain recorded. Final source,
binary and runtime-evidence review passed. Cumulative Tasks 2.24–2.31 source and
source-bound retained evidence were also reviewed; earlier suites were not rerun.
The reviewed sequence is **TECHNICALLY VALIDATED / FOUNDER-AUTHORIZED MAIN
INTEGRATION** within its documented scopes. Final documentation and publication
checks passed; root publishes main and integration by normal fast-forward.
Git records the exact commit; the checkpoint retains evidence paths and limits.

This is **conditional main integration authority**, not unconditional founder
acceptance, complete Testnet readiness or new economic/governance authority.
The condition does not permit relabelling failed initialization as success,
weakening tests, hiding unresolved defects or publishing an unvalidated current
task to main. Technical development and compatible corrections continue under
D-026; if the condition is unmet, preserve reviewed progress and explain it.
No force push, history rewrite, automatic release/tag, new live operation,
Mainnet action, deployment, fund movement, secrets access, keys/signing or
upgrade-authority transfer is authorized. All sensitive-action gates remain.

### D-030 — CONFIRMED — Converge on a complete founder-testable Testnet lifecycle

On **2026-09-28 UTC**, after Task 2.38 publication at
`8eee7cb884f2e37bb2a31c48aed77557cf4fb62d`, the founder issued the
**PIV1 — Testnet convergence directive** in the connected pilot session.
This is a confirmed execution-priority decision under D-026. It does not change
accepted economics, governance, custody powers, founder acceptance or main/live
publication authority. Task 2.38 remains technically validated within its recorded
scope; no new founder acceptance of that implementation is inferred.

The priority is the first **complete founder-testable PIV1 economic lifecycle on
Solana Testnet**, using production instruction paths. The next milestones are:

1. Expose and validate the real PIV1 initializer through the production
   instruction boundary.
2. Expose the production runtime handlers required for the complete economic
   lifecycle.
3. Implement the real pinned SPL/Jito stake-pool adapter and protected CPI paths.
4. Execute and review a complete local end-to-end lifecycle using the real
   production instruction paths.
5. Prepare the exact Testnet deployment artifact, addresses, authorities,
   funding requirements and founder test workflow.
6. Stop before the first live Testnet deployment and request the founder's
   explicit deployment authorization with a concise checklist.

Follow this order unless a demonstrated dependency requires otherwise. Record
the concrete blocker, evidence and affected milestone before a departure; resume
the ordered path once it is resolved. Do not create validation-only micro-tasks
unless they close a concrete blocker on this Testnet critical path. Existing
harnesses and source-bound evidence should be reused where applicable. Retained
limitations are not automatically separate prerequisite tasks, and no guard or
required validation is waived to accelerate progress.

For each milestone, use bounded implementation work, separate review, fixes for
demonstrated defects, proportionate targeted tests plus final gates, and reviewed
integration checkpoints. Preserve `main` unless separately authorized. Technical
validation remains distinct from founder acceptance. Keep progress durable before
continuing or interruption; avoid repetitive suites or expanding isolated proof
work without a direct milestone dependency. This replaces the earlier proposed
next isolated Bank failure task and automatic per-task stop scheduling. Historical
reports keep their actual scope, evidence, failures and contemporaneous plans.

Do not reopen accepted economics or ask the founder to choose Rust/Solana
implementation details. Ask only for genuinely missing economic/governance
decisions or sensitive live-operation authorization. Any actual unresolved Task
2.38 defect that blocks this path must be resolved first. Source/result inspection
in this guidance turn found no such unresolved defect: production initializer
dispatch, rather than another synthetic probe, is the next implementation gap.

Before any live deployment or related signing/fund-moving operation, D-026's
existing gate still requires a concrete package: verified cluster/genesis,
exact artifact/hash and public Program ID, required authorities and recipients,
funding source/requirements and bounded fee/rent/test-SOL budget, proposed
operations, stop/recovery conditions and practical founder workflow. Do not invent
addresses, balances or approval. Present the concise checklist and obtain explicit
founder authorization before execution; preparation is not that authorization.
No Mainnet action, real funds, secrets/key creation/signing, deployment, authority
transfer or release/tag is authorized by this directive itself. Retain every
other permanent safety restriction.

The [execution plan](PIV1_CODEX_EXECUTION_PLAN.md) and
[test plan](PIV1_TEST_PLAN.md) contain the actionable milestone/evidence gates;
the [pilot checkpoint](PIV1_PILOT_STATE.md) records current progress and blockers.

### D-031 — CONFIRMED — Publish the validated Tasks 2.33–2.39 sequence to main

On **2026-09-28 UTC**, after Task 2.39/M1 integration publication, the founder
instructed in the connected pilot session (English translation):

> Please publish to main everything that is not there yet but has already been validated.

This explicitly authorizes normal fast-forward main integration of the exact
reviewed implementation tip `bf32d87e06a2d54c8e1c0192faaa7855d246dbc0`, plus
necessary reviewed documentation of this authorization/publication. Starting
main is `4cc4ea11e87c2f1a2f85b9ed48358f2a881821b3`. The eight-commit sequence
covers Tasks 2.33–2.39 and D-030's Testnet convergence guidance. These tasks are
**TECHNICALLY VALIDATED / FOUNDER-AUTHORIZED MAIN INTEGRATION** within their
recorded scopes; no unvalidated future task belongs to this authorization.

Root verifies the intervening diff, retained source-bound evidence, independent
review and publication safety before publishing main/integration normally, then
checks the actual resulting refs. This documentation-only publication turn does
not rerun tests/builds or start M2. Existing task reports retain their actual
executions, failed attempts, limited evidence and contemporaneous status. Git,
the publication receipt and [pilot checkpoint](PIV1_PILOT_STATE.md) record the
final identity and checks without inventing a future commit hash.

This is explicit main integration authority, **not broader founder acceptance**,
complete Testnet readiness or a change to accepted economics/governance. D-030
still places M2 next after M1's bounded completion. No force push, history rewrite,
automatic release/tag, secrets/key creation/signing, deployment, fund movement,
Mainnet operation or authority transfer is authorized. D-026's exact live-Testnet
approval gate and every other sensitive-action restriction remain unchanged.

### D-032 — CONFIRMED — Block explicit contributions during emergency pause

On **2026-09-28 UTC**, during Task 2.40/M2, the founder answered the outstanding
D-023/D-024 contribution-handler policy question (English translation):

> Block explicit deposits during the pause (recommended).

Production `deposit_sol` and `deposit_jitosol` must reject while Config is paused,
before any transfer or contribution-state effect. The question explicitly retained
accounting of direct incoming transfers already received in either option; this
remains required during pause. Reconciliation does not authorize staking,
integration, distribution or other paused economic operations. This resolves only
explicit contribution callability previously marked PROVISIONAL; it does not
change the accepted contribution/HWM accounting or the isolated K-012 claim rule.
No deployment, main publication or broader acceptance is authorized by this choice.

### D-033 — CONFIRMED — Publish the validated Tasks 2.40–2.46 sequence to main

On **2026-10-03 UTC**, after Task 2.46 integration publication, the founder
instructed in the connected pilot session (English translation):

> Please publish everything that is validated to main.

This authorizes normal fast-forward main integration from
`8912cfeffcd83fa12cc1a68937a9de8dd5c6b091` through the exact reviewed tip
`7624f93bf55721cb74687677a26c51fcf25cd669`, plus necessary reviewed
authorization/publication documentation. The covered sequence is:

| Task | Reviewed implementation commit |
| --- | --- |
| 2.40 — Guardian heartbeat and authorized pause | `1b1677c7df57925012e4f36ee597423aa641344d` |
| 2.41 — Explicit SOL/JitoSOL contribution intake | `3a5c3ce9179975c2cfd7ba09d379459aaa53a8d0` |
| 2.42 — Economic surplus normalization | `9856ec8de01ca5706296bf59e63f9396bce4ecbf` |
| 2.43 — Initial contribution bootstrap | `c47272acbb2f9628cc308935b1c6b5f1fa575406` |
| 2.44 — Protected principal deposit | `a18f33a2b5e9cd38ea1f6413d3e4e2e7d74a810c` |
| 2.45 — Liquid distribution preparation | `17bc0760390034157a932e51de674f976d114922` |
| 2.46 — Active-source withdrawal preparation | `7624f93bf55721cb74687677a26c51fcf25cd669` |

Status: **TECHNICALLY VALIDATED / FOUNDER-AUTHORIZED MAIN INTEGRATION** within
each task's documented scope. Root verifies actual refs, cumulative ancestry,
retained source-bound validation, separate review and hooks/CI before normal
publication, then independently verifies resulting main/integration refs. This
is a documentation/publication operation; no tests/builds or implementation are
repeated. Git, the publication receipt and pilot checkpoint record the final
identity. Historical reports keep their actual findings, corrections and limits.

This authorization does not cover unvalidated future work, broader founder
acceptance, complete Testnet readiness or an economic/governance change. M2/full
M3–M6 remain open under D-030. No force push, history rewrite, release/tag, secrets,
key creation/signing, deployment, fund movement or authority transfer is inferred.
The exact live-Testnet approval gate remains unchanged. Future technical work
uses reviewed integration checkpoints under D-026 until separately authorized
for main publication.

### D-034 — CONFIRMED — Stop after technically validated M4; M5 requires explicit authorization

On **2026-10-12 UTC**, the founder instructed in the connected pilot session:

> When M4 is fully completed and technically validated, stop before starting M5 and notify me with a concise completion report.
>
> Do not start M5 until I explicitly authorize it.

This adds a mandatory milestone boundary to D-026/D-030: finish and technically
validate M4, retain its separately reviewed completion checkpoint, then stop and
report the completed scope, actual validation evidence and remaining limitations
briefly in French. Do not begin M5 package preparation until explicit founder
authorization to start M5 is recorded. Generic continuation or full technical
access is not that authorization. The separate live-deployment authorization gate
still applies after any future M5 authorization.

M4 is currently incomplete and blocked under Task 2.51; this directive is not a
completion claim or an immediate stop of authorized M4 work. Accepted economics,
founder acceptance and main-publication limits are unchanged. Historical D-030
instructions to continue through the ordered milestones are subject to this new
stop boundary.

Later D-039 narrowly authorizes M5 preparation with the incomplete M4 reservation;
the original instruction above remains its historical decision record.

### D-035 — CONFIRMED — Publish reviewed validated work to main after bounded continuation

On **2026-10-12 UTC**, the founder explicitly instructed the pilot to continue
and then publish all technically validated work missing from main. Actual takeover
was clean integration `c481433b9de498384aba299fda445116cbb19396`, main
`1054ff3751ca4169eab7e46d77452634b11fc695`, protected Task 2.3 `3677fee`.

This authorizes normal fast-forward main publication of the separately reviewed
validated implementation sequence currently missing from main, plus the bounded
Task 2.51 continuation only to the extent its correction is technically validated,
and necessary reviewed authorization/checkpoint records. Root must verify exact
ancestry, final source-bound evidence, separate review, hooks/CI and actual remote
refs. The final publication receipt and Git identify the included commits.

A work-in-progress lifecycle report does not declare the lifecycle complete.
Existing task-level host/static/runtime scopes and unresolved blockers remain
explicit; an independently validated correction may be published without claiming
full M3/M4 completion. Do not publish an unreviewed or unvalidated new correction.
This is main-integration authority, not broader founder acceptance, changed
economics, live operations or permission to start M5. D-034's mandatory stop after
fully technically validated M4 remains binding. No force push, history rewrite,
release/tag, signing, secrets, deployment, funds or authority transfer is inferred.
Future work outside this bounded publication returns to reviewed integration
checkpoints unless separately authorized.

The bounded continuation closes the first complete positive local lifecycle
with reviewed fixture compatibility and settlement allocation corrections: 595
host tests +1 doctest/eight gates, strict SBF and Bank I pass. Full M4 remains
incomplete for its documented adverse runtime coverage. Preserve all prior failed
attempts and source/artifact-specific limits; main publication changes none of them.

### D-036 — CONFIRMED — Publish the reviewed continuation through runtime V to main

On **2026-10-16 UTC**, the founder explicitly requested publication of completed
work to GitHub `main`. This authorizes normal fast-forward publication of the nine
reviewed integration commits after `2f8d08368742b7a6c48a8f4e00096be217138fe9`
through `07133aa5fe18f0879843b5d2bf2ed184aa7608aa`, plus the necessary reviewed
D-036 authorization/checkpoint documentation. Root verifies ancestry, retained
source-bound validation and separate review, hooks/CI, and actual remote refs.
Git and `/tmp/piv1-main-publication-20261016-a/publication.json` identify the final
published main/integration commit. Protected Task 2.3 remains `3677fee`.

This supersedes earlier integration-only/main `2f8d083` publication statements
only for the covered sequence; historical runtime results, failed attempts and
limitations remain unchanged. No tests, builds or runtime are rerun for this
documentation/publication operation. Technical validation and main publication
do not establish founder acceptance or M4 completion. D-034 still requires the
reviewed M4 completion report and STOP before M5, which remains unauthorized.
The separately platform-blocked internal-failure case remains stopped/unwaived.
No economic change, force push, history rewrite, release/tag, secrets, signing,
key creation, deployment, funds or authority transfer is authorized. Subsequent
work returns to reviewed integration checkpoints unless separately authorized
for main publication.

### D-037 — CONFIRMED — Revisit suspended local M4 work under normal platform controls

On **2026-10-18 UTC**, after discussing the historical Daybreak/Astra interruption,
the founder explicitly authorized revisiting and continuing the suspended work
in the connected chat, then reiterated the request to advance. The original
messages are retained in the session and the resumption-authority evidence.
This permits reassessing and normally resuming the local M4 regression within
D-026, superseding the repository's automatic-resumption prohibition for that
attempt. It does not authorize disabling or bypassing platform controls, changing
accepted economics, waiving evidence, declaring acceptance, publishing to main,
starting M5 or performing a live operation. D-034 remains binding.

Normal delegation/design review began, but the writer then received a fresh
platform rejection before any test, build or runtime execution. Root stopped
without rephrasing, retrying, changing model or routing execution elsewhere.
Partial edits were preserved with hashes and restored out of the working tree;
no failed or incomplete implementation is published as validated. The renewed
interruption is recorded in the pilot checkpoint and
`/tmp/piv1-m4-internal-resume-20261018-a/interruption.json`.
This decision is owner authorization, not evidence of platform access clearance.
M4 remains incomplete; its outstanding criterion is unwaived and M5 unauthorized.

### D-038 — CONFIRMED — Publish reviewed integration checkpoints to main

**D-038 — Founder-authorized main publication (2026-10-18 UTC).**
The founder requested updating GitHub and publishing all already validated work
on main. This authorizes normal fast-forward publication of the 14 reviewed
integration commits after `705e1518e7de17e31d3e5ef2c6c92d4a89683166` through
`4adedfd266f96288fc898501a8e49dbcadc2a032`, plus this reviewed publication checkpoint.
The scoped acceptance proposal remains a proposal, not an accepted decision.
M4 remains technically incomplete; its internal-callee-failure evidence gap is
unwaived and the rejected operation remains stopped. M5 is not authorized.
No new test, build or runtime result is claimed; retained reviewed evidence binds
the unchanged implementations. No deployment, secrets, signing, authority transfer,
release/tag or history rewrite is included. Protected Task 2.3 remains `3677fee`.
Git and `/tmp/piv1-main-publication-20261018-a/publication.json` record the actual
publication result and identity. Earlier main/integration-only statements describe
historical scope; this authorization supersedes them only for this bounded sequence.
Subsequent work returns to reviewed integration checkpoints unless separately
authorized for main. Publication does not establish founder acceptance or M4 closure.

### D-039 — CONFIRMED — M5 preparation with an explicit M4 reservation

On **2026-10-18 UTC**, the founder sent
`PIV1_M5_REPRISE_GPT_6_1_SOL_2026-10-18.md` to the existing pilot. Its exact copy
is retained as `/tmp/piv1-m5-readiness-20261018-a/founder-directive.md`
(SHA-256 `359c3dece304be7296b70ac35adfeae72ca109f66da3d3a1f76eef8f883c1d45`).

The founder explicitly authorizes **M5 PREPARATION** while M4 remains
**TECHNICALLY INCOMPLETE / ONE RECORDED EVIDENCE CRITERION OPEN**. This is a
narrow change to D-034's sequencing/authorization rule, not M4 completion,
acceptance of the validation-scope proposal, removal of the internal-callee-failure
criterion or a claim that source reasoning replaces runtime evidence. Carry that
reservation into separate readiness review before any live approval. The
platform-rejected operation and interrupted drafts remain stopped; do not retry,
rephrase, reroute or execute them, including through a model change.

Permitted work includes a finite actual-Testnet readiness matrix, official public
documentation and narrowly scoped read-only RPC after cluster/genesis checks;
offline transaction/governance assembly and bounded compatible technical work with
separate review; and an exact founder testing package. Preserve accepted economics,
custody, schemas, pinned production dependencies/toolchain and evidence. No new
economic policy or validation-scope waiver is implicit. Missing public identities
remain OPEN, without invented addresses or private-key access. Use the single
[M5 readiness register](PIV1_M5_TESTNET_READINESS.md) and existing canonical files.

Use economical bounded work and a separate reviewer. Use GPT-6.1 Sol/High for
routine pilot work; Extra High is limited to bounded critical transaction,
authority, code or final-readiness reviews. No silent Astra fallback or Max/Ultra.
Verify observable custom-agent overrides and report only observed settings;
do not rewrite global configuration. Standard speed is requested, but its actual
setting is not exposed and must not be reported as verified. Reuse source-bound validation; run
new checks for changed behavior or demonstrated defects, not because the model
changed. The [initial register](PIV1_M5_TESTNET_READINESS.md) records new read-only
observations, no new tests/builds/runtime and a public package that is NOT READY.

Normal reviewed publication to `integration/piv1-testnet` remains authorized under
the mandate. D-038 covered its exact past main sequence; new main publication is
not authorized. Live Testnet deployment remains **NOT AUTHORIZED** and Mainnet is
out of scope. No signing, wallet/key operations, on-chain account/lookup-table/
proposal creation, transaction submission, airdrops, funds movement, authority
transfer, release/tag or history rewrite is authorized. A future live decision
requires a reviewed exact package, disclosed residual risks and separate explicit
founder authorization; never deploy automatically.

### D-040 — CONFIRMED — Publication of reviewed M5 preparation to main

On **2026-10-19 Europe/Warsaw**, the founder requested: “publie tout ce qui est ok
sur main stp”. This authorizes normal fast-forward publication of the ten already
reviewed integration commits after main `a524cc616624f41aa54141b3ac0838edd6c0f48b`
through `78ab8a4c7ea16ffde5a4fd632466cdd6ebfd84c7`, plus this separately reviewed
publication/authorization checkpoint. The bounded sequence is `21cd68e`,
`ffe054d`, `3c3589d`, `a0f785b`, `9418d9c`, `b4827e0`, `454fc59`, `7216a6b`,
`5a76a06`, `78ab8a4`. Exact full identities, parents, trees and retained review/
publication hashes are recorded in
`/tmp/piv1-main-publication-20261019-a/history-audit.json`.

The scope includes reviewed M5 readiness/compatibility evidence, validation tools
and offline client plans, within their documented limits. Publication is not
founder economic acceptance, M4 completion, a criterion waiver, public readiness or
live deployment approval. M4 remains technically incomplete; its stopped operation
and drafts are untouched/unwaived. D-039 continues to authorize M5 PREPARATION
only; the public package remains NOT READY and deployment NOT AUTHORIZED.

Existing ten commits are integrated unchanged. Only six canonical authorization/
checkpoint documents are newly edited in this publication turn. No application
suite, build, Solana runtime or RPC is rerun; reviewed historical results remain
retained evidence. No production/economic/dependency/toolchain/artifact change,
secrets, signing, funds movement, authority transfer, release/tag or history rewrite
is authorized or performed. Protected Task 2.3 remains `3677fee`. Root verifies
actual resulting main/integration refs; Git and
`/tmp/piv1-main-publication-20261019-a/publication.json` record completion/identity.
Later work returns to reviewed integration-only publication until separately
authorized for main. Earlier restrictions are superseded only for this sequence.

### D-041 — CONFIRMED — Seven Testnet identities and bounded wallet funding

On **2026-10-19 Europe/Warsaw**, the founder explicitly approved seven new
Testnet-only keys (six disposable guardian candidates plus a separate payer) and
funding up to **1 SOL per new address**, total at most **7 SOL plus network fee**,
from public source `4WKQg3Sm8bvHS8DBmxoiMMi4Fev4mJU6ZLwGSTg6jDna`.
The trusted session scope is retained in
`/tmp/piv1-testnet-identity-funding-20261019-a/authorization.json`.
Root generated those keys privately once and finalized the single reviewed
seven-destination System funding transaction; [the register](PIV1_M5_TESTNET_READINESS.md#current-authorized-testnet-identities-and-wallet-funding--d-041)
and [public setup record](PIV1_TESTNET_IDENTITY_SETUP.json) identify execution.

This supersedes prior no-key/no-signing/no-funding statements **only for these
seven identities and this capped Testnet funding**. It is a one-shot initial
budget, now consumed by one finalized transfer: no refill, standing authority over
the source balance, regeneration or overwrite. The accepted six-guardian/4-of-6
requirement remains unchanged. It authorizes no exact new Squads creation or
configuration profile/transaction, including proposed timelock0/rentCollectorNone,
PIV program key/deployment, authority
transfer, Mainnet operation, new economics or new main publication. Further
Squads creation requires an exact reviewed packet/current fees/funding and separate
explicit authorization. Remaining slots, vault indices, pause and KIF anchor stay
OPEN. D-039's M4 incomplete/open/unwaived reservation and stopped-operation limits
remain binding; public package NOT READY and deployment NOT AUTHORIZED.

### D-042 — CONFIRMED — One exact Testnet Squads creation, subject to its debit cap

On **2026-10-19 Europe/Warsaw**, the founder replied **“oui j'autorise”** to
root's exact reviewed creation card at integration `ca9bc19`: create candidate
`9mwto7RNokYc8n8MPtzHpubwwnKcborY2r9RnfZAScVS` with the six existing guardian
members, threshold 4, permission mask 7, autonomous config authority, zero timelock,
no rent collector/memo. Existing payer/creator is
`H2AaoG8HZKfNyZzMhDGMB8n9edviyHtJGKpPL3uYpxt4`; createKey signer is guardian 0.
The unchanged proposal `PIV1_TESTNET_SQUADS_CREATION_PLAN.json` has SHA-256
`6ceb6b796464e53036708bda12e22de9529d8b6459c3c8efec0c9b0649b00bff`.
It describes the approved 219-byte payload/619-byte one-instruction packet;
only fresh blockhash/signatures may change. Total payer debit is at most
**3,000,000 lamports**. Authorization is retained in
`/tmp/piv1-squads-create-execution-20261019-a/authorization.json`.

This permits only that one Testnet creation with the prescribed existing signers,
subject to its budget. It does not authorize additional transfers/refills, new
keys, original-source use, PIV deployment, authority transfer, Mainnet, main
publication, economic change, M4 acceptance/waiver or stopped-operation resumption.
The immutable original proposal's pre-approval flags are historical; this decision
records the later trusted approval without rewriting that signed-off input.

Execution remains **HELD / NOT SUBMITTED**: separate source review establishes
that create_v2 reads a mutable global creation fee at inclusion and carries no
maximum-fee argument. With 1 SOL in the payer, preflight checks do not enforce
the agreed cap atomically. The separate atomic refund-plus-creation proposal in the readiness register is
**NOT AUTHORIZED**; obtain approval for its additional return transfer and changed
transaction envelope before any refund or revised execution. Preserve the existing approval rather
than asking to approve the unchanged multisig profile again. M4 remains open/
unwaived, public package NOT READY and PIV deployment NOT AUTHORIZED.

### D-043 — CONFIRMED — Exact atomic Testnet return and Squads creation

On **2026-10-19 Europe/Warsaw**, after reviewed integration `be4b387`, the founder
replied **“go j'autorise faut qu'on avance”** to root's exact revised card. This
explicitly authorizes **one atomic Testnet transaction** returning **997,663,360
lamports** from existing H2 payer to original source
`4WKQg3Sm8bvHS8DBmxoiMMi4Fev4mJU6ZLwGSTg6jDna`, then creating the already approved
4/6 Squads at `9mwto7RNokYc8n8MPtzHpubwwnKcborY2r9RnfZAScVS` in the same packet.
The payee/source never signs or supplies funds. The H2 payer is emptied:
**gross debit 1,000,000,000 lamports**, including the return; expected **net own
cost 2,336,640 lamports** (rent 2,326,640 plus network fee 10,000), capped at
3,000,000 excluding returned funds. Each guardian retains 1 SOL.

This narrowly supersedes D-042's single-instruction envelope/gross-debit basis
and pending additional-transfer gate. The unchanged creation profile, six ordered
roles, two existing signers and 219-byte payload stay approved. The added System
return changes the full legacy packet to 668 bytes/eight static keys/two
instructions; only a fresh blockhash and the two actual signatures may change
that exact reviewed envelope. Reviewed proposal
`PIV1_TESTNET_SQUADS_CAPPED_PAYER_PROPOSAL.json` SHA-256 is
`174af0437fb80b2bad309970c668ea102cf18251bcc5e16cbdbe2ec9b6efea5a`.
Its pre-approval flags are retained historical preparation; this newer direct
human decision supplies approval. Net exposure concerns originally funded funds,
not an immutable gross-fee argument against arbitrary unsolicited extra credits.

Root may privately load only existing payer and guardian-0 keys for this exact
transaction, verify signatures, signed-simulate, submit once and reconcile finalized
state. Fresh genesis/image/Config-zero-fee/rent/fee/balance/absence gates and separate
executor review remain mandatory. Instruction state commits/rolls back atomically;
a failed included transaction can charge its network fee. No refill, repricing,
automatic retry, new keys, guardian/source debit, PIV deployment, Mainnet,
authority transfer, new main publication, economic change, founder PIV acceptance
or M4 waiver. If outcome is uncertain, status reads only. Preserve old disabled
drafts unchanged. Authorization/result/public evidence are retained in
`/tmp/piv1-squads-atomic-execution-20261019-a`. Root subsequently finalized exactly one transaction at slot **454165126**,
fresh reread **454165136**, signature `51BqDxFxy9E8aejSiXEN8miwFr7Nqvs7S32UbzNu2cF9ivbvjWsFEKTnoVhTjP6aXVhekh6z1CbK1VVySLEnwwSn`.
The [public setup record](PIV1_TESTNET_SQUADS_SETUP.json) binds full bytes/state,
source credit, payer zero, exact cost and separate evidence review. This one-shot
authorization is now consumed; no automatic retry/refill or later operation.
Permission and the independently verified execution remain distinct evidence.

### D-044 — CONFIRMED — Reviewed main publication through the offline pause planner

**Current publication — D-044 founder-authorized main integration (2026-10-19 UTC).**
The founder explicitly requests publishing all validated work to GitHub main.
This covers the nine reviewed commits after `e8f33f311f427c02529108fdd39fb85691f99479`
through `df1f911f5a10a15fd573cab8e1d317f3ee6b2b8e`, plus this separately reviewed
documentation checkpoint, by normal fast-forward publication. Earlier main and
integration-only statements below retain their historical scope. Git and
`/tmp/piv1-main-publication-20261019-b/publication.json` record the actual resulting
main/integration identity. Protected Task 2.3 stays `3677fee`.
Only authorization/checkpoint documentation changes in this publication turn;
prior implementations, test results and D-041/D-043 chain observations are retained,
not rerun or refreshed. M4 remains technically incomplete with its criterion
open/unwaived and stopped drafts untouched. M5 preparation continues; public package
NOT READY and PIV deployment NOT AUTHORIZED. Consumed D-043 is not renewed.
No new economic decision, founder delivery acceptance, keys/signing, funds,
authority transfer, deployment, release/tag or history rewrite is authorized.
Next complete the exact PIV identity/ALT/recipient/funding and remaining governed
operator package. Subsequent work returns to reviewed integration publication
unless separately authorized for main. Preserve evidence and the 2-GiB reserve.

### D-045 — CONFIRMED — One disposable Testnet PIV program identity generation

On **2026-10-20 Europe/Warsaw**, after reviewed integration
`9f3459a57bff7793d9aabbf02b2ebcec01bbe47e`, the founder replied
**“je confirme et j'autorise”** to the exact one-key generation card. This
explicitly authorizes one new disposable **Testnet-only PIV program Ed25519 keypair**,
not another guardian, payer or ALT key. Approved proposal JSON SHA-256 is
`8437817b20db803f309afa990a2978f1a0dba7488d367de0a6a1908dce5c3651`;
its original pre-approval flags and guide remain unchanged historical inputs.

Root completed exactly one reviewed helper invocation at **2026-10-20 17:34:29 UTC**,
storing the key outside Git in the proposed exclusive setup directory owned by
jerem (0700), file (0600), without changing existing ancestor permissions.
The public program identity is
`GAjHRaC3XM98SoRoFy5B8P2ypTS1ndf1UPhXmjWgtPux`.
The helper reads only its newly created file once and validates public derivation
without signing; no pre-existing key is read. Public completion and evidence hashes
are in [the setup record](PIV1_TESTNET_PROGRAM_IDENTITY_SETUP.json).

This **one-shot authorization is consumed**: no overwrite, regeneration, automatic
retry, backup/copy or additional key creation. Zero RPC, signing, transactions,
funding, SOL movement, rent and network fees occurred or were authorized.
No PIV deployment, authority transfer, Mainnet, new main publication, economic
change, founder delivery acceptance or M4 waiver is included. Ordinary local JSON
is unencrypted and does not establish independent custody, snapshot protection
or memory-zeroization guarantees. No on-chain account, balance, loader state,
actual-address artifact portability or deployment readiness is proved.

Separate source review precedes execution; separate public-outcome and checkpoint
review follow without rereading or hashing the private file. Pre-execution
metadata-policy, dependency-resolution and root wrapper API corrections are
preserved; none was a failed key-generation attempt. M4 stays open/unwaived and
stopped drafts untouched; M5 preparation only, public package NOT READY and PIV
deployment NOT AUTHORIZED. D-041/D-043 remain consumed; main/protected Task 2.3
are unchanged. Next assemble the exact remaining public artifact/ALT/recipient/
funding/founder package, without silently selecting slots, vault indices, initial
pause, KIF anchor or test amounts.

### D-046 — CONFIRMED — Profile acceptance and three unsigned funding/ALT packets only

On **2026-10-20 Europe/Warsaw**, after reviewed integration
`77968ce582cc389a4bb91cd151cbe7ed4b5b6f68`, the founder replied:
**“ok je valide le profil proposé, prépare les transactions exactes pour le financement et l’ALT, sans rien envoyer ni déployer pour le moment stp”**.
The exact public reply and immutable card hash are bound in
[the unsigned packet record](PIV1_TESTNET_FUNDING_ALT_PACKETS.json).

This accepts the proposed profile: existing six PIV guardians in G0..G5 slot order,
permutation [3,1,4,0,5,2], four distinct G0..G3 approvers, governance/HTFP/Team
vault indices 0/1/2 with their stated recipient purposes, G0 creator/executor/fee
payer/ALT authority and distinct H2 rent payer. Initial pause=true, technical
capacity 1x/894,552 and the fresh-Clock-anchor capture/freeze policy are accepted.
Actual KIF timestamp and artifact deployment selection remain unset. No economic
policy, independent custody, observed authority or successful runtime is inferred.

The founder requests **exact unsigned preparation of only three packets**:
one fixed 4WK→H2 credit 4,700,000,000 lamports with source fee cap 10,000;
ALT create; and exact 16-entry ALT extension with G0 authority/fee payer and H2
rent payer, ALT fee caps 10,000 each. All five signature slots remain zero.
No recipient-funding packet is included. Root may perform bounded official
read-only observations and offline assembly/review; this is not authorization
to access private files, sign, simulate, submit, transfer funds, create/extend ALT,
hand over a buffer, deploy, initialize PIV, unpause or run economic actions.
No main publication or renewal of consumed D-041/D-043/D-045 is granted.

Root prepared the three full unsigned packets once after separate source review;
separate actual-wire and raw-freshness audits pass. Eleven read-only requests/five
HTTP batches include final observations proving the blockhash expired and selected
ALT slot left SlotHashes. All three packets are **ARCHIVED EXPIRED—DO NOT SIGN OR SEND**;
no helper rerun or packet/address rebinding occurred.

The mechanical ALT candidate and blockhash are snapshot-only evidence. Live ALT
selection/state, actual funding and signed execution stay unproved/unset. Preserve
expired packets without automatic refresh/rebinding. Future exact execution needs
its own explicit scope and fresh state/lifetimes; profile acceptance supplies none.
M4 stays technically incomplete/open/unwaived and stopped drafts untouched;
M5 preparation/public NOT READY/deployment NOT AUTHORIZED. Root publishes reviewed
integration only; main/protected references and 2-GiB reserve remain unchanged.

### D-047 — CONFIRMED AND CONSUMED — Fixed funding and one exact Testnet ALT

After reviewed integration `3b3ee0fd88335828e9a9bdfe694f9e9eb0a70257`, the founder
replied **“on avance j'autorise”** to the exact funding/ALT report. This authorizes
only one 4WK→H2 credit 4,700,000,000 lamports (source fee<=10,000), one fresh G0/H2
ALT creation and exact 16 extension (G0 fees<=10,000 each/20,000 total), including
a fresh SlotHashes candidate derived AFTER funding finalizes and frozen BEFORE
phase approval. Only the 32 blockhash bytes may refresh after approval before a
single signature attempt; no old archive is rebound. Root-only existing source/
H2/G0 signing, exact signed simulation and at most one send per phase are included. No recipient, new key,
program key, buffer handover, PIV deployment/initializer/economics or main publication.

Root finalized all three once at 454845162/454845438/454845694, fees 5,000/10,000/
10,000. Actual ALT `4SoVCaX8tzTE1R6jGMJTArPq75afvBtVyr93Mm6hbTFF` holds 3,535,680;
final reread 454845700 verifies G0 authority/exact 16/active/strict later-slot warmup.
H2/source/G0 final balances 4,696,464,320/128,763,968,087/999,980,000 are factual,
not renewed spending authority. The [public result](PIV1_TESTNET_FUNDING_ALT_SETUP.json)
binds all signed bytes/signatures, exact deltas and separate actual phase reviews.
92 RPC/90 HTTP comprise 86 ordinary reads, three signed simulations and three sends;
no retry or failed execution. Canonical Rent/image checks and actual costs pass;
no inclusion-time atomic rent cap is claimed. Public oracle/parser/count premise
corrections are retained, not failed application runs. D-046 expired archives stay exact.

**All D-047 scope is consumed. Never repeat funding, recreate/re-extend, re-sign or
send again.** No further private read/signing/funds/ALT change or future refill is
authorized. Any uncertainty is reconciled read-only with existing signatures.
D-041/D-043/D-045 stay consumed; M4 open/unwaived/stopped drafts untouched.
M5/public NOT READY/PIV deployment NOT AUTHORIZED. Old tests/Bank/SBF are retained,
not rerun; publication is reviewed integration only, main/protected unchanged.
Preserve active evidence and the 2-GiB reserve. Prepare remaining work under the
existing mandate; later sensitive live actions need their own exact scope.

### D-048 — CONFIRMED — Conditional Testnet deployment subject to actual readiness

After reviewed integration `fafdd48d1d6d8d94b901506c33e3bbd5495a7b7d`, the founder
states **“on avance j'autorise le deploiment si tout est pret sinon on avance poour que tout soit pret stp”**.
This records explicit conditional deployment permission and continued preparation.
Preserve that permission for the eventual exact reviewed Testnet package instead
of requesting routine technical approvals again. Its readiness condition is not
currently satisfied: the public package is NOT READY and M4's recorded criterion
remains technically incomplete/open/unwaived. This statement does not waive that
criterion, accept the pending M4 scope proposal, change economics/governance or
renew consumed D-041/D-043/D-045/D-047. It is not authorization to execute expired
packets or prematurely perform partial sensitive operations while readiness is unmet.

Continue local/read-only preparation and separate review of exact artifact, loader,
authorities, funding, recipients, transport and workflow. Before any live action,
record the independently reviewed exact package, its readiness conclusion and
any genuinely missing founder risk/governance decision. No live action follows
from this checkpoint; no Mainnet, main publication, secrets/signing or authority
change is performed. Normal reviewed integration publication remains authorized.
Stopped M4 operations/drafts stay stopped; preserve their evidence and the2GiB reserve.


D-048 preparation outcome (2026-10-20): one guarded host build passes; the first
local Bank run fails after seven setup/governance successes. At the atomic step,
the runtime rejects loader Deploy CPI before native loader invocation. The current
recipe is **INVALID_DIRECT_DEPLOY_CPI_DO_NOT_EXECUTE**; no ELF deployment/canary,
post-failure full map or rollback proof follows. Official reviewed 4.4 CPI source
has the same relevant restriction as local 4.2; loader-branch-only review was
insufficient. Original proposals remain immutable historical evidence. See the
[blocked result](PIV1_TESTNET_ARTIFACT_LOADER_READINESS.json) for raw receipts.
Conditional permission remains recorded, **READINESS UNMET / NO LIVE TRIGGER**.
Prepare a separately reviewed top-level atomic bootstrap/handover alternative only
as UNSELECTED/UNAPPROVED/UNEXECUTED: temporary authority and changed pre-vote
Buffer/four-approval/hash guarantees need exact review, not silent relaxation.
No new governance policy, M4 waiver, live action or consumed-scope renewal occurs.


D-048 technical continuation (2026-10-21 Europe/Warsaw): one offline assembler and
six host wire groups pass for an **UNSELECTED/UNAPPROVED** atomic top-level bootstrap
alternative. Complete final wire is 1,035 bytes/six zero signature slots: three
native zero self-transfers request G1/G2/G3 exact-message endorsements alongside
G0/H2/PIV, then consecutive Program create/Deploy with temporary G0/SetAuthority
to intended final vault0. This is not Squads votes or native membership/hash/threshold
or global alternate-message enforcement; G0 remains Buffer writer until inclusion.
It changes the prior unapproved technical recipe, not confirmed final 4/6 governance,
and does not establish a founder-confirmed first-instruction PDA rule. No exception,
artifact, authority recipe or live action is selected/approved by this host result.
The prior direct loader-CPI recipe stays INVALID_DIRECT_DEPLOY_CPI_DO_NOT_EXECUTE;
ELF/runtime compatibility remains unproved. Continue positive validation/exact package
preparation; D-048 remains conditional/readiness unmet/no live trigger, M4 open/unwaived,
stopped drafts untouched and consumed scopes unrenewed. No new decision is made.
See [proposal](PIV1_TESTNET_ATOMIC_BOOTSTRAP_PROPOSAL.json) for exact public bytes/limits.


D-048 technical continuation (2026-10-21 Europe/Warsaw): one isolated guarded host
build/run passes native atomic bootstrap 1,035B/six zero slots and newly deployed
production heartbeat 601B/one zero slot at slot 2. Root/separate audits verify exact
ELF loader acceptance at actual PIV ID, fresh Program/PD/final vault 0, full maps,
one PD-create CPI outer 7/stack 2 and activity-only heartbeat delta. Local fees total
35,000; CU 5,940/59,920 and loaded 895,430/897,649 are bounded local measurements.
Buffer/app state are explicit fixtures and all signatures zero; no upload/initializer/
full lifecycle, actual endorsements/Squads votes or exact Testnet 4.4 replay is proved.
The prior failed direct loader-CPI recipe remains invalid and immutable. This is
technical local success, not selection of the temporary bootstrap trust exception
or artifact, a new founder decision, M4 waiver or live trigger. Prepare the exact
remaining governed initializer/trust/version/resource/funding package under existing
mandate; D-048 permission stays conditional/readiness unmet, M4 open/unwaived and
stopped drafts/consumed scopes untouched. See [bounded runtime](PIV1_TESTNET_ATOMIC_BOOTSTRAP_RUNTIME.json).

D-048 technical continuation (2026-10-21 Europe/Warsaw): a new offline initializer
wrapper validates one supplied existing ALT snapshot and emits only the remaining
Squads buffer/proposal/four approvals/v0 execute. V1 had six passing groups and one
fixture error before the planner call; a test-only correction precedes v2 seven
passing groups plus template/null (48 missing). Root/separate literal audits verify
18 final synthetic packets/38 zero slots, exact 568-byte table, full-u64 slots and
16 target order; no ALT setup, extra reuse-only authority signature or rent re-spend.
Supplied snapshots are unauthenticated and not refreshed; no real Clock anchor,
artifact/bootstrap trust selection, fee/funding sufficiency, actual votes/signatures
or initializer runtime follows. Prior local runtime and invalid direct Deploy-CPI
archives stay unchanged. Continue exact governed package preparation under D-039;
D-048 remains conditional/readiness unmet/no live trigger, M4 open/unwaived and
stopped drafts/consumed scopes untouched. No new founder decision is made.
See [offline existing-ALT initializer](PIV1_TESTNET_EXISTING_ALT_INITIALIZER.json).

D-048 technical continuation (2026-10-21 Europe/Warsaw): final isolated startup
passes 11 local entries/23 zero slots/115,000 local fees: native deployment at the
actual public PIV ID, nine real Squads existing-ALT initializer steps/four local
approvals and production heartbeat. Program/PD/all 16 targets begin absent;
13 fresh maps, 633 account records and 46 ordered CPIs (41 initializer) match
root/separate literal audits. Initializer measures 992,404 CU/3,833,006 loaded bytes;
same-H2 final 104,027,480 and G0 final 1,008,434,960 include G0-only buffer refund.
Four builds/three processes remain preserved: first compile failure, pre-Bank JSON
precision failure, nine-step partial process halted on a harness ALT-order guard,
and final complete pass. These are not discarded or relabeled production failures.
Fixtures/unsigned Bank 4.2 do not prove upload, live guardian control, recipient
setup, whole lifecycle or Testnet 4.4 replay. No artifact/bootstrap trust/real Clock
anchor is selected; direct Deploy-CPI invalidity is unchanged. Continue consolidated
exact package preparation under the existing mandate; conditional permission stays
readiness unmet/no live trigger, M4 open/unwaived and stopped drafts/consumed scopes
untouched. No new founder decision or live action. See [startup evidence](PIV1_TESTNET_STARTUP_GOVERNED_INITIALIZER.json).

D-048 technical continuation (2026-10-21 Europe/Warsaw): one public offline
consolidation and seven host groups pass, with root/separate actual audits of
949 unsigned sizing packets/963 zero slots, all 936 ELF Writes and 975 conditional
cashflow rows. Root makes 41 read-only RPC requests/seven HTTP calls; 15 actual
representative quotes plus 934 unquoted Write projections give 4,815,000 projected
fees. Same-H2 required envelope 4,594,387,560 includes one required 650,240 residual;
conditional headroom is 102,076,760. The 8,569,960 buffer refund is G0-only.
Both recipients and vault 0 are absent; only two recipient credits are modeled.
No source vault-floor predicate is invented, and the absent-vault runtime case
remains unproved. Historical final blockhash validity cannot establish a complete
upload lifetime. No new Bank/build/private/sign/simulation/send/funds or selection;
prior local runtime and all failed-probe records remain retained. Continue the exact
startup trust/version/funding/lifetime/resume package under the existing mandate;
D-048 remains conditional/readiness unmet/no live trigger, M4 open/unwaived,
stopped drafts untouched and consumed scopes unrenewed. See [consolidated evidence](PIV1_TESTNET_CONSOLIDATED_STARTUP_BUDGET.json).

D-048 technical continuation (2026-10-21 Europe/Warsaw): one offline pinned Buffer
upload-resume client and eight host groups pass with separate source/actual review.
Root audits 67 standalone unsigned CLI wires and three complete 936-chunk reports;
a negative CLI rejects a late conflicting chunk before any stdout. Synthetic host
replay reconstructs all 936 writes in 15 bounded batches, preserving matched bytes.
Explicit u64 lifetime margins are unverified declarations, not throughput or live
confirmation. No automatic create/deploy/handover, Clock binding, trust selection,
RPC/signing/simulation/send/funds/build/Bank action or consumed-scope renewal occurs.
The pre-execution test expectation/parser corrections are retained; no unexpected
application failure occurred. Existing production/economics, artifact and four
selected package sections remain unchanged. Continue the exact remaining readiness
package under retained conditional permission, READINESS UNMET / NO LIVE TRIGGER;
M4 stays OPEN_UNWAIVED and stopped drafts untouched. See [upload-resume evidence](PIV1_TESTNET_UPLOAD_RESUME.json).

D-048 technical continuation (2026-10-21 Europe/Warsaw): one new guarded host
build and positive local startup pass with governance vault0 genuinely absent
from genesis and all 13 full maps. Root/separate actual audits verify 14 absence
records, 620 accounts, 722 evidence rows, 11 successful entries/23 zero slots,
115,000 local fees and 46 ordered CPIs (41 initializer). No third funding is added;
all 16 source-derived PIV states and H2/G0 cashflows remain exact. New initializer
loaded bytes are 3,832,942; earlier 3,833,006 remains historical funded-vault proof.
Only this concrete local fixture gap closes. Bank 4.2/recipient/Buffer/Clock
fixtures and unsigned transport are not public Testnet 4.4/control proof.
A bounded [startup phase and decision plan](PIV1_TESTNET_STARTUP_RELEASE_PLAN.md)
now separates the pending exact-candidate/temporary-G0 trust choice and supported
local M4 deliverable acceptance with reservation from technical preflight and
exact live scope. Neither decision is inferred: M4 stays technically incomplete
and its stopped operation/criterion remain untouched/open/unwaived. This is
preparation under retained conditional D-048, not renewed deployment permission,
funds/authority scope or a live trigger. No new RPC/private/key/sign/send/live
funds/authority/deployment, production/economic change or Mainnet action occurs.
Normal reviewed integration publication only; consumed scopes are not renewed.

### D-049 — CONFIRMED — Exact Testnet candidate/bootstrap selection and limited M4 acceptance; preparation only

2026-10-21, after reviewed integration `7a35238`. The founder explicitly confirms A and B at the exact limited scopes in [startup release plan](PIV1_TESTNET_STARTUP_RELEASE_PLAN.md). The [public preflight record](PIV1_TESTNET_STARTUP_PREFLIGHT.json) preserves the complete verbatim statement and separate evidence.

A selects the exact894,552-byte artifact SHA-256 `e29735e1467da98ecddb1eea9fd25abaffa4a450bd899ddc137f7d8c0da35b99` for dedicated disposable Testnet Program `GAjHRaC3XM98SoRoFy5B8P2ypTS1ndf1UPhXmjWgtPux`, and accepts disclosed temporaryG0 native bootstrap/Buffer mutability until inclusion with intended atomic handover to canonical Squads vault0. The founder acknowledges that outer endorsements are not on-chain4/6votes/hash enforcement and same-VPS custody is not independent. This selects a future recipe, not execution/authority-transfer permission. Direct loader Deploy CPI remains INVALID_DIRECT_DEPLOY_CPI_DO_NOT_EXECUTE.

B accepts **SUPPORTED LOCAL M4 DELIVERABLE WITH THE EXPLICIT INTERNAL-CALLEE-FAILURE EVIDENCE RESERVATION** only. M4 remains **TECHNICALLY INCOMPLETE / OPEN_UNWAIVED**. The missing test is neither passed nor waived; no circumvention, inspection/retry/rephrasing/rerouting/execution of stopped operations/drafts is authorized. Technical completion and final functional acceptance remain separate.

Continue authorized M5 preparation and independent readiness review: artifact provenance, current Testnet state, transaction feasibility, authorities, signatures and funding requirements. Resolve routine technical implementation within the mandate without repeated questions. **No signing, deployment, additional fund movements, authority transfers or Mainnet operations are authorized. Present the completed exact final package and request separate explicit authorization before any new live operation.** One such future authorization may cover its exact bounded multiple-transaction scope; no per-command approval loop is imposed. D-048 is retained historical conditional permission, not a trigger overriding this newer instruction; consumed scopes are not renewed. Main publication remains separately gated.

Root and separate review check current public/preparation evidence; 39 read-only RPC/7HTTP do not prove signatures/control, Testnet whole-runtime equivalence, current/future spending authority or live readiness. The single M5 register remains authoritative; public package NOT READY. No economic invariant, stack/dependency or missing technical criterion changes.

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
