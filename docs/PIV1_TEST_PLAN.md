# PIV1 requirements-to-evidence checklist

This is an execution index under D-026, not a new specification or acceptance.
Canonical requirements remain in `PIV1_DECISIONS.md` and `PIV1_MASTER_SPEC.md`.
Current commit, actual executions and active task are in `PIV1_PILOT_STATE.md`.
Update this checklist when a bounded task closes; do not infer runtime evidence
from passing host tests.

## Verified baseline: Task 2.9 after corrected Task 2.3 and Tasks 2.4–2.8

Pilot executions on Task 2.9 source at `36152c157773737eca357e5dbfefd3f0900b6eb3`:
298 host tests, one doctest,
default/all-feature checks and warnings-denied documentation pass. Separate
source review passed within the documented scope. Founder acceptance is pending.
The task reports distinguish original executor, writer and pilot evidence; see
[Task 2.9](TASK_2_9_STATE_ENVELOPE_PERSISTENCE.md),
[Task 2.8](TASK_2_8_GUARDIAN_CLOCK_SNAPSHOT_AUTHENTICATION.md),
[Task 2.7](TASK_2_7_ISOLATED_KIF_CLAIMS.md),
[Task 2.6](TASK_2_6_PROTECTED_PRINCIPAL_DEPOSIT.md),
[Task 2.5](TASK_2_5_INITIAL_CONTRIBUTION_BOOTSTRAP.md),
[Task 2.4](TASK_2_4_ACCOUNT_AUTHENTICATION.md),
[corrected Task 2.3](TASK_2_3_VAULT_RECONCILIATION_MODEL.md),
[Task 2.2](TASK_2_2_CONTRIBUTION_PENDING_MODEL.md),
[Task 2.1](TASK_2_1_MOCK_STAKE_POOL_ADAPTER.md), and
[Phase 1 properties](TASK_1_4_PROPERTY_TESTS.md).

| Requirement | Current inspected evidence | Evidence still required before founder Testnet handover |
| --- | --- | --- |
| P-003/P-008/P-020–P-024: locked principal, fixed split, checked floors, external fees | Math unit/property tests; legal/illegal lifecycle tests; Task 1.2–1.4 reports | Actual handler account privileges, fixed native destinations, transfer/CPI deltas and failure rollback |
| P-013–P-019: full pending contribution value, HWM and loss recovery | `contribution_pending`, `vault_reconciliation`, property tests; Task 2.3 T23-R1 severe-loss regressions | Authenticated observations, atomic runtime custody movement, real pool valuation |
| P-009–P-012/P-029–P-032: permissionless cadence, one round, insufficient cooldown | Legal/illegal transitions and randomized property ordering | Trusted Clock decoding, instruction privileges and malformed transaction rollback |
| P-004/P-014/P-015: first-contribution bootstrap and idle principal intake | Task 2.5 initial-only bootstrap tests with a genuine empty host fixture and later contributed-token yield | Actual initialization/transfers and protected SOL deposit; general idle integration remains separately scoped; never invent funded initial principal or contribution yield |
| A-003: distinct fixed economic custody | Task 2.3 per-vault obligations and host normalization; Task 2.4 AccountInfo/PDA/owner/token/rent/deficit tests | Complete authenticated initialization, real transfers and rent-preserving normalization |
| Direct/untracked SOL and token contributions | Task 2.2 idempotence and Task 2.3 all supported economic-vault host paths | Actual transfer races; unsupported native surplus in Token/temporary accounts; operational funding provenance |
| A-001/P-018/P-026: official accounting, protected CPI, dynamic minima | Task 2.1 mock interface and fee/slippage/minimum boundary tests | Pinned real SPL/Jito pool/list/mint/source validation, account-derived snapshot identity and exact protected instruction behavior |
| A-002/A-005: permissionless multi-leg source order, maximum fills, exact target | Mock source-capacity tests; lifecycle/property tests; Task 2.3 composition | Real preferred/source-order checks, deterministic stake/metadata authentication, adversarial runtime and multi-leg Testnet evidence |
| P-035: delayed readiness, rent recovery, cooldown reward/loss | Mock finalization and Task 2.3 exact custody/recovery tests | Stake/Clock/Stake History decoding, actual closure, both rent destinations and exact post-CPI deltas |
| P-024/A-004: atomic settlement, later pending integration and compounding | Pure transitions and staged host rollback; Task 2.6 zero-fee deposit, exact deltas/mint audit, carry/HWM/failure regressions | Runtime transaction rollback and real protected CPI; general fee/rounding-loss support remains OPEN |
| K-005–K-010: active snapshots, half-open 30-day periods, repeated carry | Timing/guardian unit tests, math/property/lifecycle and host custody tests; Task 2.8 nine-AccountInfo current registry/rewards/Clock authentication and 22 activity/identity/borrowing regressions | Actual runtime inputs, authorized signed heartbeat and verified qualifying governance activity; global historical-ledger and earning-provenance invariants |
| K-012: earned isolated claims remain available during pause | Task 2.7 four-AccountInfo authentication, immutable earned-owner PDA, partial/full claim preparation and exact atomic host custody; 24 regressions; Task 2.9 byte-only Config/reward persistence regression preserves lamports and intentionally fails the untouched custody audit | Actual handler/signatures/CPI/locking with coupled state writes, maintained historical-ledger sum, runtime failure and pause evidence; instruction remains a marker |
| G-003–G-005: explicit emergency pause and economic gates | Pure illegal-transition pause matrix; Task 2.3 pending recognition and recovery preservation | Governed pause/unpause plus handler/runtime rejection for snapshots, deposits/conversions, withdrawals, finalization and migrations; preserve K-012 claims exception |
| K-001–K-004/G-001/G-002/G-007/G-008: six guardians, 4-of-6 governance and upgrade custody | Bounded registry validation and immutable snapshot tests | Squads authority/membership authentication, pause/recipient/rotation implementation and non-bypass tests; exact live authority identities and upgrade-authority verification; actual authority-transfer rehearsal requires its separate explicit authorization |
| Layout and failure atomicity | Five bounded schemas, Option length/property tests, full-state rollback assertions; Task 2.9 four typed fixed envelopes and 21 existing-AccountInfo atomic byte/alias/borrow/stale-state regressions | Authorized handlers coupling state and actual custody, runtime rollback/privileges, initialization replay/alias protection, WithdrawalLeg persistence and SBF limits |
| Reproducible and reviewed delivery | Pinned Rust/Anchor stack, locked/offline host gates, actual separate AI source reviews | Real-adapter dependency review, SBF artifact/build reproducibility, compute/size/rent measurements and final adversarial review |
| Founder testing path | Planned CLI/runbook in canonical spec; not implemented | Usable non-Rust test workflow, verified addresses/artifact, actual approved lifecycle signatures/states, cooldown/retry/recovery instructions |

## Evidence levels and deployment gate

- **Host math/state/mock:** controlled Rust data and atomic staged models only.
- **AccountInfo fixtures:** actual byte/owner/key validation on host fixtures;
  not proof of runtime locks, invocation privilege rules or CPI rollback.
- **Local runtime:** record exact harness/program artifacts, supported instruction
  paths and adversarial cases. Do not label a hand-mutated fixture as a validator.
- **Public Testnet:** record independently verified genesis/cluster, approved
  program/authority/recipient identities, artifact, transaction signatures and
  account states. Historical Phase 0 one-leg spike evidence is not the new PIV1
  deployment, multi-leg orchestration, or founder-testable milestone.

No public-Testnet deployment/fund-moving lifecycle or new key/signing workflow
is authorized yet. D-026 requires its concrete live-operation approval package.
Mainnet and authority transfers remain outside the technical development mandate.
Critical predeployment checks precede deployment regardless of phase labels.
No known critical/high defect may be hidden by an evidence-level distinction.

## Next dependency tracking

Tasks 2.4–2.9 are technically validated within their bounded scopes, pending
founder acceptance. Task 2.7 implementation `10dceb5` and closure `37f25a8`
are published with accepted main unchanged. Task 2.8 current-registry
activity/Clock authentication passed final separate review and pilot gates; its
reviewed closure `440e83e26d36df911ccfafac97d89b79b8b4c694` is published.
Task 2.9 typed state envelopes and atomic existing-account byte persistence
passed final separate review and pilot gates; Git/publication closure is in
progress. Actual runtime/adapter integration remains required;
derive the next bounded scope from these dependencies without changing economics.

General fee/rounding-loss support for principal deposits remains OPEN: a protected
slippage floor alone does not preserve full historical book value. General idle
integration remains deferred; preserve pending-SOL priority and no-yield/
insufficiency behavior. Before economic handlers rely on an observation, resolve
or explicitly contain Token-native-surplus liveness and operational/rent funding
provenance. State serialization/initialization, handler privileges and exact real
adapter mapping remain separate bounded work items. Numbering does not itself
authorize or define an implementation.
