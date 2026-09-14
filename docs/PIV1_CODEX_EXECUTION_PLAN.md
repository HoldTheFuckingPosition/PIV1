# PIV1 Codex Execution Plan v0.2

## Current pilot workflow (D-026)

The founder activated [the technical pilot mandate](PIV1_TECHNICAL_PILOT_MANDATE.md).
The current task and verified evidence live in [PIV1_PILOT_STATE.md](PIV1_PILOT_STATE.md).
Historical fresh-permission/stop and no-publication language below describes the
earlier authorizations. D-026 now permits successive bounded reviewed technical
tasks on a development integration branch without inferring founder acceptance.
It does not authorize Mainnet or new public-Testnet deployment/signing/fund-moving
operations before the mandate's concrete live-operation approval gate.

## Accepted foundation milestone (D-027)

On **2026-09-14**, the founder accepted Tasks 2.3–2.19 at
`d9f3371be6ecb586675e3b38edcc57bd6e9519f8` within their documented bounded
scopes and authorized integration into `main`. Root published the normal
fast-forward from `66193769d1cbc59cd8630df295b9a784b9c64642` and independently
verified remote main/integration at the accepted commit. Necessary acceptance
records are maintained in separately reviewed documentation commits; the
checkpoint and Git record the current publication identity. No PR was created.

The milestone is **COMPLETE / FOUNDER-ACCEPTED**; Phase 2 remains **IN PROGRESS**
and Task 2.20 was not started at acceptance. That milestone retains 416 host tests +1 doctest/
eight gates; 24 SBF tests/70 cases apply only to the historical Task 2.14 artifact.
No new test run, current-source runtime proof, economics or live scope follows
from this documentation update. See [D-027](PIV1_DECISIONS.md), the
[integration review](PIV1_INTEGRATION_REVIEW_2_3_TO_2_19.md) and
[current checkpoint](PIV1_PILOT_STATE.md). Task-specific evidence and deferred
boundaries below remain limited to their original scopes.

## Operating model

- The dedicated ChatGPT PIV1 development chat acts as architect, reviewer, security lead, and task planner.
- Codex CLI runs inside the PIV1 Git repository on the Linux VPS.
- The founder approves product decisions and all irreversible actions.
- Codex receives one bounded task at a time.
- Every task ends with a report, tests, diff summary, and commit hash.

## Hard restrictions for Codex

Codex must never:

- deploy to Mainnet;
- transfer real funds;
- create or reveal mainnet seed phrases/private keys;
- store secrets in Git;
- transfer upgrade authority;
- invent missing recipient addresses;
- silently change confirmed economics;
- disable tests to make CI pass;
- make irreversible VPS changes without backup/commit;
- use unpinned dependencies without justification;
- treat Testnet or Devnet addresses as Mainnet addresses;
- claim an audit was performed by a professional third party.

## Phase 0 task sequence — COMPLETE

Founder review accepted the Phase 0 report and all schema-blocking decisions on
2026-08-30. The corrected dual-token-vault topology and scalable multi-validator
withdrawal architecture are confirmed production requirements. The single-leg
custody lifecycle is confirmed by public Testnet; multi-leg orchestration is
architecture-derived and must not be described as live-tested.

### Task 0.1 - VPS and repository inventory

Codex must report:

- OS/version;
- CPU/RAM/disk availability;
- existing Rust/Solana/Anchor/Node/Codex versions;
- Git status and target directory;
- network/RPC constraints;
- any conflicting existing projects.

No installs or edits before report unless explicitly approved.

### Task 0.2 - Initialize repository safely

Create:

- Git repository;
- `.gitignore` covering keys, env files, build artifacts, ledgers;
- `AGENTS.md` containing project constraints;
- `docs/` with the supplied handoff files;
- initial signed/normal commit.

Acceptance:

- no secrets tracked;
- clean Git status;
- commit hash supplied.

### Task 0.3 - Pin development toolchain

Use official documentation to select compatible current versions for:

- Rust;
- Solana CLI/toolchain;
- Anchor;
- Node/package manager;
- SPL stake-pool libraries;
- TypeScript tooling.

Create version files and lockfiles. Explain compatibility evidence.

### Task 0.4 - Jito technical validation spike

Before PIV1 code, create a separate experimental area or branch that:

- fetches and decodes official Testnet Jito pool state;
- deposits Testnet SOL to receive Testnet JitoSOL;
- calculates official pool-token/SOL conversion;
- initiates delayed withdrawal;
- identifies generated stake account and authorities;
- deactivates and withdraws after readiness;
- measures fees, rent, account counts, compute, and technical minimums;
- documents current official addresses and their verification source.

This spike must not be copied blindly into production code.

### Task 0.5 - Phase 0 report

Produce `docs/PHASE_0_VALIDATION_REPORT.md` containing:

- verified toolchain;
- exact current Testnet/Mainnet protocol references;
- transaction diagrams;
- minimum amounts;
- PDA feasibility;
- observed fees;
- compute/account constraints;
- differences from the master spec;
- recommended final state machine;
- open founder decisions, only if unavoidable.

Status: **COMPLETE / FOUNDER-ACCEPTED**. The report, canonical decisions, master
specification, and this plan are synchronized. Phase 1 entry criteria are
satisfied for a separately authorized Task 1.1. Task 1.1 was not started at
Phase 0 closure and was subsequently completed and founder-accepted.

## Phase 1 tasks — COMPLETE / FOUNDER-ACCEPTED

### Task 1.1 - Scaffold Anchor workspace

Accepted Task 1.1 scope: **scaffold the modular Anchor workspace and
compile-only placeholders on a new branch from the accepted main baseline.** Create no
economic implementation, Jito CPI, deployment, key, or fund-moving test.

Status: **COMPLETE / FOUNDER-ACCEPTED**. Accepted implementation commit:
1d436570570fc31310e3e5d2c1d4d5e92320c65b.

### Task 1.2 - Implement pure math crate

Status: **COMPLETE / FOUNDER-ACCEPTED**. Accepted implementation commit:
43a3b7497653ff7a246a1e5cf9b760086dd33fcd.

Task 1.3 is **COMPLETE / FOUNDER-ACCEPTED**. Its initial implementation commit
is 33978cf3eda918e4c438b80ed0e12a47b8347519 and its final accepted
implementation tip is 527e381661fe0cfc27e07ad9b44e1601a638ae75. Task 1.4 and
the complete Phase 1 specification-as-code foundation are founder-accepted.

Implement and test:

- basis-point split;
- high-water mark;
- yield calculation interfaces;
- KIF zero-active 50/50 rule;
- KIF equal active split;
- conservative rounding;
- checked arithmetic.

### Task 1.3 - State and transition model

Status: **COMPLETE / FOUNDER-ACCEPTED**. Initial implementation commit:
33978cf3eda918e4c438b80ed0e12a47b8347519. Final accepted implementation tip:
527e381661fe0cfc27e07ad9b44e1601a638ae75.

Implement account structs and transition validation without Jito CPI.

### Task 1.4 - Property tests

Status: **COMPLETE / FOUNDER-ACCEPTED**. Accepted implementation commit:
06c39429f3237f6974e21217670c3f0d30b0a571.

Reproducible randomized/property, adversarial model-state, and
serialization/layout invariant tests are implemented without changing the
accepted production layouts or dependency graph.

The accepted suite remains pure-state/property evidence; handler, CPI,
localnet, external-account, and live-cluster validation remain deferred. This
AI-assisted review is not a professional independent audit.

Phase 1 status: **COMPLETE / FOUNDER-ACCEPTED**.

Task 2.1 is **COMPLETE / FOUNDER-ACCEPTED** at initial implementation commit
`33b1e539f969432f82635d1ca76c59d89f0ec233` and final corrected tip
`cb90d468eff4dce60552ba15b2b267b364a47827`. Task 2.2 is **COMPLETE /
FOUNDER-ACCEPTED** at implementation commit
`e3233b96b533a620e8037d5231baede10877217f`. Tasks 2.3–2.19 are **COMPLETE /
FOUNDER-ACCEPTED** within the D-027 milestone and the bounded scopes below.

## Phase 2 tasks

Status: **IN PROGRESS**. Task 2.1's narrow deterministic stake-pool interface
and fixed-capacity host-only mock are **COMPLETE / FOUNDER-ACCEPTED**. This
acceptance does not establish exact SPL/Jito behavior or a production revision
mechanism; those mappings remain Phase-3-provisional. Task 2.2's pure
contribution-intake, pending-vault reconciliation, fixed-size host custody mock,
and recorded deterministic evidence are **COMPLETE / FOUNDER-ACCEPTED** at
implementation commit `e3233b96b533a620e8037d5231baede10877217f`. Observations
still require future fixed-account and transfer validation; explicit-transfer
handler callability during pause remains PROVISIONAL; and no real custody,
handler, CPI, or localnet behavior is proven. Task 2.2 deferred custody
composition. Task 2.3 now demonstrates the supported
pure/host economic-vault normalization paths and pending-SOL-to-escrow-to-HWM
lifecycle while preserving full recognized contribution value. Its status is
**COMPLETE / FOUNDER-ACCEPTED** under D-027, with no serialized-layout change.
Operational surplus derivation remains unsupported without a funding baseline;
real account/transfer authentication and exact protocol mapping remain deferred.
Tasks 2.4–2.19 are **COMPLETE / FOUNDER-ACCEPTED** within their documented
scopes under D-027. The confirmed K-012 policy
requires future `claim_kif` handling to remain available during global pause
only for already-earned liabilities isolated in `KifSolVault`; no claim handler
is implemented by Task 2.2.

### Task 2.3 - Vault reconciliation and host custody composition

The founder authorized this bounded task from accepted baseline
`66193769d1cbc59cd8630df295b9a784b9c64642` on
`task/2.3-vault-reconciliation-model`. Status: **COMPLETE / FOUNDER-ACCEPTED**
under D-027, including T23-R1 correction `0559ebd`, 168 passing host tests, one
doctest and separate source review. The report is [TASK_2_3_VAULT_RECONCILIATION_MODEL.md](TASK_2_3_VAULT_RECONCILIATION_MODEL.md).
It records the per-vault obligations, atomic movement/state boundaries, supported
normalization, independent conservation, fixed/model regressions and deferred
operational evidence. The original review-only next-action restriction is historical under D-026.
The pilot may continue with separately bounded reviewed technical dependencies
on `integration/piv1-testnet`; founder acceptance and live-operation approval
remain separate. Current task scope and evidence are in `PIV1_PILOT_STATE.md`.

### Task 2.4 - Fixed account authentication

Status: **COMPLETE / FOUNDER-ACCEPTED** within the documented scope (D-027).
Implementation `9f4f1064deeef78a3cbea2e9f84c560e87166f20`; 191 host tests, one doctest,
checks/docs and separate source review PASS. Exact scope and limitations:
[TASK_2_4_ACCOUNT_AUTHENTICATION.md](TASK_2_4_ACCOUNT_AUTHENTICATION.md).
Authenticate actual Config/ActiveDistribution and permanent economic custody
accounts and derive checked read-only observations. Handlers, transfers and
CPI remain later dependencies. Existing economics and layouts are preserved.

### Task 2.5 - Initial contribution bootstrap

Status: **COMPLETE / FOUNDER-ACCEPTED** within the documented scope (D-027).
Implementation `9b997f364d62b0796008b2f7fb3f905acf64a2e5`; 209 host tests, one doctest,
checks/docs and separate source review PASS. Scope and limits:
[TASK_2_5_INITIAL_CONTRIBUTION_BOOTSTRAP.md](TASK_2_5_INITIAL_CONTRIBUTION_BOOTSTRAP.md).
Establish genuine initial principal from recognized pending contributions,
without a fabricated yield snapshot or pre-funded PIV principal. This boundary
is limited to initial state; no-yield/insufficient and normal distribution
semantics remain unchanged. General idle integration and production principal
staking remain later dependencies; bounded host conversion is covered by Task 2.6.

### Task 2.6 - Protected principal SOL deposit composition

Status: **COMPLETE / FOUNDER-ACCEPTED** within the documented scope (D-027).
Implementation `9f75aec59d732b2662c1b2c7626f2a8f48887619`; 231 host tests, one doctest,
checks/docs and separate source review PASS. Scope and limitations:
[TASK_2_6_PROTECTED_PRINCIPAL_DEPOSIT.md](TASK_2_6_PROTECTED_PRINCIPAL_DEPOSIT.md).
The pure Idle accounting boundary and atomic host composition permit only
zero-fee deposits preserving historical book value and HWM coverage. Exact
receipt, custody and post-pool deltas are checked; minted tokens are independently
audited. Some zero-fee inputs and all fee-bearing conversions remain unsupported
and leave SOL queued. No new fee allocation or HWM exception is approved.
Actual accounts, handlers and CPI remain later integration work.

### Task 2.7 - Isolated KIF claim authentication and accounting

Status: **COMPLETE / FOUNDER-ACCEPTED** within the documented scope (D-027).
Implementation `10dceb5b2eac691ff19840190e951bd2ec547984`.
Scope: [TASK_2_7_ISOLATED_KIF_CLAIMS.md](TASK_2_7_ISOLATED_KIF_CLAIMS.md).
Authenticate only Config, the earned guardian record, fixed KifSolVault and
its guardian signer/destination. Stage positive partial/full claims with
counter-based replay protection, full aggregate backing and atomic host
custody effects. Preserve historical earned ownership across registry rotation,
claim availability during pause, collective carry, rent and unsolicited excess.
No handler, CPI, production state writer or new economic policy is implied.
The pilot ran 255 workspace tests and one doctest, default/all-feature checks
and warnings-denied documentation; all passed. Separate review of the frozen
source and all 24 claim tests passed with no actionable finding.

### Task 2.8 - Current guardian and Clock snapshot authentication

Status: **COMPLETE / FOUNDER-ACCEPTED** within the documented scope (D-027).
Implementation `c815474eea9a7854c3b495974d891f4dd1c67a27`.
Scope: [TASK_2_8_GUARDIAN_CLOCK_SNAPSHOT_AUTHENTICATION.md](TASK_2_8_GUARDIAN_CLOCK_SNAPSHOT_AUTHENTICATION.md).
Authenticate current Config/registry/six reward records and canonical Clock,
then derive the unchanged half-open KIF period and exact activity bitmap.
Preserve historical earned claims, all layouts and read-only behavior. No
heartbeat policy, state writes, handler, signing or runtime execution is implied.
The pilot ran 277 workspace tests and one doctest, default/all-feature checks
and warnings-denied documentation; all passed. Separate review of the exact
five-file diff, all 22 new tests and the full evidence report passed without
actionable findings.

### Task 2.9 - Validated state envelopes and atomic existing-account persistence

Status: **COMPLETE / FOUNDER-ACCEPTED** within the documented scope (D-027).
Implementation `36152c157773737eca357e5dbfefd3f0900b6eb3`.
Scope: [TASK_2_9_STATE_ENVELOPE_PERSISTENCE.md](TASK_2_9_STATE_ENVELOPE_PERSISTENCE.md).
Encode the four already-authenticated state types with checked fixed envelopes
and zero tails. Stage identity-bound full before/after bytes and acquire all
mutable account borrows before a bounded batch writes any byte. Preserve rent,
lamports and allocation; keep authorization, initialization, handlers, transfers
and WithdrawalLeg persistence separate. Pilot validation passed 298 workspace
tests, one doctest, default/all-feature checks and warnings-denied documentation.
Separate final review of the exact four-file diff, all 21 tests and the complete
evidence report passed without actionable findings.

### Task 2.10 - Isolated KIF claim execution and explicit host invocation evidence

Status: **COMPLETE / FOUNDER-ACCEPTED** within the documented scope (D-027).
Implementation `c38a7b0d7122144bf3082cec7e57bbea7e61cc10`.
Scope: [TASK_2_10_ISOLATED_KIF_CLAIM_EXECUTION.md](TASK_2_10_ISOLATED_KIF_CLAIM_EXECUTION.md).
Connect existing claim authentication, checked effects and state persistence to
fixed signed System-transfer invocation code, preserving checks-effects-interactions
and exact fresh postconditions. The ordinary host path must reject before mutation;
tests use an explicit isolated invocation/transaction model. Preflight mutable
native data/lamport borrows before bookkeeping. No ABI, entrypoint, live identities,
runtime execution or heartbeat policy is implied. Pilot gates passed 318 workspace
tests, one doctest, default/all-feature checks and warnings-denied documentation.
Separate final review of the exact four-file diff, all 20 execution tests and
complete evidence passed without actionable findings.

### Task 2.11 - KIF claim instruction boundary

Status: **COMPLETE / FOUNDER-ACCEPTED** within the documented scope (D-027).
Implementation `2eeefba0abc226bcfcadddb5f248f12ca589e09d`.
Scope: [TASK_2_11_KIF_CLAIM_INSTRUCTION_BOUNDARY.md](TASK_2_11_KIF_CLAIM_INSTRUCTION_BOUNDARY.md).
Connect the completed claim execution to a strict ABI, runtime-ID native entrypoint,
trusted Rent acquisition, stable errors and success event. D-003's documented
layer exception avoids inventing a Program ID. Host dispatch evidence is distinct
from SBF/runtime proof. No new keys, build-generated keys or deployment is allowed.
The pilot ran 335 workspace tests, one doctest, default/all-feature and explicit
no-entrypoint/cpi/idl-build checks, and warnings-denied docs: all passed on frozen
source. Separate final review of the ten-file diff, all 17 new tests and complete
evidence returned PASS without actionable findings. SBF/runtime proof remains separate.

### Task 2.12 - Keyless SBF compilation and artifact inspection

Status: **COMPLETE / FOUNDER-ACCEPTED** within the documented scope (D-027).
Scope: [TASK_2_12_KEYLESS_SBF_COMPILATION.md](TASK_2_12_KEYLESS_SBF_COMPILATION.md).
Prepare and separately review a direct cached compiler runner before an actual
locked/offline SBF build. Inspect exact artifact/entrypoint and diagnostics with
unchanged source, dependencies/profile and no keys. Production compatibility
corrections require a concrete reviewed amendment after actual evidence. Runtime
harness, signing and deployment remain separate; compilation is not runtime proof.
The third target attempt resolved prior stack diagnostics with reviewed private
boxing, preserved ordering/serialized values and compile-time allocation bounds.
Final pilot 339 tests +1 doctest/eight gates and separate source/artifact review
passed. Implementation: `cee6072ad7b3155d7d5b30e6c0830beb6d00d4eb`.

### Task 2.13 - Keyless local SBF claim execution

Status: **COMPLETE / FOUNDER-ACCEPTED** within the documented scope (D-027).
Scope: [TASK_2_13_KEYLESS_SBF_CLAIM_EXECUTION.md](TASK_2_13_KEYLESS_SBF_CLAIM_EXECUTION.md).
After Task 2.12 publication, select and review an isolated pinned local SVM harness
for the unchanged artifact, actual Rent/System CPI, full account effects and
passively observed failures. Synthetic privileges and harness failure discard
must remain distinct from signatures, Bank rollback and public-chain proof.
After two preserved host-build failures and reviewed harness corrections, the
third build passed without diagnostics. The pilot executed 19 local tests across
60 message cases on the exact identified executable; all passed. Separate final
review independently checked full account bytes, real local System CPI/events,
paused/sequential claims and failure observations. Ordinary claims used 73834 of
200000 CU. All four reduced-compute probes failed before observed accounting or
payment; post-CEI compute failure remains unproven. This does not establish Bank
rollback, signatures, deployment or public-network behavior.
Implementation: `fd5735976eef1e2728ccf54726145501573db60d`.

### Task 2.14 - Runtime recognition of pending contributions

Status: **COMPLETE / FOUNDER-ACCEPTED** within the documented scope (D-027).
Implementation: `7442cab7e97c422c7ee06290d5fc9d11c8b13ee6`.
Scope: [TASK_2_14_RUNTIME_PENDING_RECONCILIATION.md](TASK_2_14_RUNTIME_PENDING_RECONCILIATION.md).
Expose the existing D-024 phase-dependent pending-recognition transition through
a narrow authenticated, permissionless instruction. Only Config's two pending
ledgers may change; preserve balances, round, HWM and all unrelated state.
Keep token native excess visible/unclassified and separate from pending assets.
Require host/account/dispatch regressions, new exact-artifact SBF evidence and
the preserved claim tests. Initialization, transfers, normalization/sweeps and
new economic policy remain separate. Task 2.13 is published at `fd48c3b`.
Root verified 349 host tests +1 doctest/eight gates and 24 local SBF tests across
70 cases on the new artifact. Separate final review passed. Real System donation
and exact pending recognition are proved locally; active/settled/recovery offsets
retain host evidence. Initialization, SPL Token transfer, Bank rollback and
public-cluster operation remain unproven.

### Task 2.15 - Read-only Squads authority snapshot

Status: **COMPLETE / FOUNDER-ACCEPTED** within the documented scope (D-027).
Implementation: `da0241fd2a9f9c3247bbdeabb1a6b9c37dabc912`.
Scope: [TASK_2_15_SQUADS_AUTHORITY_SNAPSHOT.md](TASK_2_15_SQUADS_AUTHORITY_SNAPSHOT.md).
Authenticate the current loader-v3 PIV1 Program/ProgramData authority binding to
an official-source Squads v4 vault and six-voter, threshold-four autonomous
multisig. Bind set correspondence to same-program authenticated guardian evidence
without reordering PIV1 slots or changing historical claims. Use bounded decoding
and host adversarial tests, with no new package, instruction, schema or error ABI.
This is current-state evidence only: stale already-approved Squads proposals can
survive a threshold change. Real action approval, execution binding, rotation,
initialization, deployed-artifact verification and Testnet availability remain
separate prerequisites. Separate source/test review and root executions passed: 363 host tests, one
doctest and eight gates with no diagnostics. Final documentation/evidence review
also passed. D-027 accepts this bounded scope without extending its runtime evidence.

### Task 2.16 - Bounded direct Squads invocation authorization

Status: **COMPLETE / FOUNDER-ACCEPTED** within the documented scope (D-027).
Implementation: `343a496fb16edc8fd8d68746a89323d7545ab36d`.
Report: [TASK_2_16_SQUADS_INVOCATION_AUTHORIZATION.md](TASK_2_16_SQUADS_INVOCATION_AUTHORIZATION.md).
Bind current nonstale four-of-six proposal approval to an exact direct Squads
execute invocation and a single stored PIV1 instruction. Require fresh authority
and same-program guardian evidence, canonical proposal/transaction/vault links,
runtime Clock/context and exact current inner accounts/data/privileges. Keep
outer global privilege unions distinct; bound persisted Borsh vectors and reject
unsupported batches/ALT/ephemeral signers. Use explicit host evidence and a guarded
runtime-facing wrapper, without adding a handler or claiming runtime execution.
Rotation, initializer message budget, persistent effect-once/rollback and deployed
Squads equivalence remain separate. Separate source/test review and root's 383
host tests, one doctest and eight gates passed with no diagnostics. Executor
flexibility is preserved using the authenticated outer executor; no inner executor
is mandatory. Final separate documentation/evidence review passed. The next dependency is a
bounded initialization/transport assessment, not an approved implementation scope.

### Task 2.17 - Separate preinitialization Squads authorization

Status: **COMPLETE / FOUNDER-ACCEPTED** within the documented scope (D-027).
Implementation: `69f7289e4c3ea2821141c4bcaded5ae942eed979`.
[Task report](TASK_2_17_SQUADS_BOOTSTRAP_AUTHORIZATION.md).
Reuse exact Task 2.16 invocation checks while preserving its initialized-governance
boundary. Authenticate a writable canonical virgin System-owned Config PDA and
fresh current Squads authority/approvals without requiring existing PIV1 state.
Permit prefunding; do not infer its economic purpose. Output sorted current-member
approval evidence, without registry slot/revision or activity semantics. No account
creation, initializer parameter validation, handler, schema, dependency or transport
is included. Separate source/test review and root 392 host tests +1 doctest/eight
gates passed with no diagnostics. Final separate documentation/evidence review
also passed. Published closure: `09a02cbe485ab8abd7cb55f155539002df9251a8`, with
independently verified remote refs and clean worktree at that closure.

### Task 2.18 - Approved genesis model preparation

Status: **COMPLETE / FOUNDER-ACCEPTED** within the documented scope (D-027).
Implementation: `dc51450396a0e369e690d9038b7dde1a80e2ecd6`.
Decode an exact bounded preparation format and freshly bind the same bytes/context
to Task 2.17 authorization. Derive canonical initial PDAs/state and immutable
initial economics from approved declarations, preserving guardian order. Output
an explicitly modeled candidate and sixteen intended target descriptors. Protocol
bindings/recipient control and actual target/funding validation remain unproven.
No handler, account creation, persistence, schema or dependency change is included.
Separate source/test/report review and root 404 host tests +1 doctest/eight gates
passed without diagnostics. Twelve new model tests preserve the earlier 29 direct
invocation/bootstrap tests. See [evidence](TASK_2_18_APPROVED_GENESIS_MODEL.md).
The next dependency is source-pinned protocol authentication against a trusted
deployment identity, before actual prefunding-safe account creation.

### Task 2.19 - Source-pinned Jito account identity

Status: **COMPLETE / FOUNDER-ACCEPTED** within the documented scope (D-027).
Implementation: `b9f6f43d21713b3ec0bf81403378819f7cd3e44e`.
Authenticate seven actual protocol accounts against independently fixed official
source identities and declared relationships. Use bounded borrowed parsing and
existing legacy Token decoding, with exact-source extracted serializer tests.
Only pinned test-only Stake interface and existing Borsh1 dependency edges are
justified;
production dependencies and locked package versions/sources stay unchanged.
Keep output distinct from governance, genesis creation, deployed binary/cluster
attestation and operational freshness/liquidity/fee readiness. Separate final
source/test/dependency/report review and root 416 host tests +1 doctest/eight gates
passed. The 12 new tests include 432 parser combinations; all 168 locked package
identities remain unchanged. See [evidence](TASK_2_19_JITO_ACCOUNT_IDENTITY.md).
The next dependency is actual genesis composition and prefunding-safe creation,
with technical scope review and a checkpoint before implementation.

### Task 2.20 - Genesis account preflight

Status: **TECHNICALLY VALIDATED / PENDING FOUNDER ACCEPTANCE** under D-026.
Compose fresh approved genesis parameters and Jito identity from the same
authenticated account slice and one runtime context. Check all sixteen expected
targets are currently writable, System-owned, empty and nonexecutable; derive
checked rent-only shortfalls without changing balances or economic ledgers.
Protect semantic role separation and modeled account backing aliases. Results
remain read-only observations; actual creation, funding provenance, recipient
control, transport, CPI and runtime initialization are separate dependencies.
No schema, instruction ABI, dependency or accepted economics changes. Ten focused
tests and separate source/test review passed; root executed 426 host tests +1
doctest/eight gates with no failures or diagnostics on the reviewed freeze.
See [evidence and limitations](TASK_2_20_GENESIS_ACCOUNT_PREFLIGHT.md).
After reviewed documentation/publication checks, STOP this session without
starting Task 2.21. Actual creation and funding provenance remain next.

Remaining Phase 2 plan, executed as bounded tasks under D-026:

Build a mock stake-pool adapter and complete localnet behavior:

- deposits;
- pending queues;
- snapshots;
- two-stage distribution;
- guardian activity and claims;
- pause;
- recipient update;
- direct/untracked transfer reconciliation;
- failure/retry paths;
- insufficient-attempt behavior with no snapshot or accounting mutation;
- 24-hour anti-spam retry cooldown;
- malformed failed calls cannot extend the cooldown;
- operational rent reserve recycling after temporary account closure.

Every subtask gets its own commit.

## Phase 3 tasks

Replace mock adapter with real SPL/Jito integration behind a narrow interface.

Do not mix protocol integration changes with unrelated accounting changes in one commit.

## Phase 4 tasks

Deploy only to Testnet after explicit approval. Use a disposable Testnet authority. Execute documented test scenarios and capture transaction signatures/account states.

## Phase 5 tasks

Security hardening:

- review with Codex `/review`;
- separate adversarial AI reviews;
- dependency and supply-chain review;
- fuzzing/property tests;
- reproducible build;
- verified-build rehearsal;
- Squads authority-transfer rehearsal on Testnet;
- public documentation draft.

## Phase 6 tasks

Mainnet preparation only. Codex prepares commands and checklists but does not execute irreversible commands without explicit founder approval at that exact step.

## Required Codex response format after every task

1. Goal completed.
2. Files created/modified.
3. Key design choices.
4. Commands run.
5. Test results.
6. Security observations.
7. Remaining risks/open points.
8. Git status.
9. Commit hash.
10. Explicit statement that no Mainnet action or real-fund movement occurred.
