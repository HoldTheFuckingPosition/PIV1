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

## Current main integration authority (D-028)

The founder conditionally authorized main publication of Tasks 2.20–2.23 at
`3282e1ebabcb0cd88491d48a391565b8b100afa7`, plus reviewed authorization records,
on 2026-09-19. Source/evidence and separate review passed; main and integration
were published at `7b74be4b13c019b96a0c8abcbebfbcc361d31089` and independently
verified on resumption. This is integration authority, not broader founder
acceptance. Prior 448 host tests +1 doctest/eight gates and nine Node tests/eight
transport cases were retained evidence; no tests were rerun in that publication
turn. See [D-028](PIV1_DECISIONS.md) and the [checkpoint](PIV1_PILOT_STATE.md).
Tasks 2.24–2.26 subsequently completed under D-026 on integration only; see their
bounded scopes below. Task 2.28 publication completed; Task 2.29 is technically validated as recorded below.

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

Status: **TECHNICALLY VALIDATED / FOUNDER-AUTHORIZED MAIN INTEGRATION (D-028)**.
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
That session stopped after reviewed publication. The founder resumed on
2026-09-19 UTC for the next bounded dependency below.

### Task 2.21 - Prefunding-safe genesis allocation

Status: **TECHNICALLY VALIDATED / FOUNDER-AUTHORIZED MAIN INTEGRATION (D-028)**.
Freshly compose Task 2.20 with an explicitly signing, approved external System
payer and canonical System Program. Pay only checked rent shortfalls, preserving
the payer rent floor and every target prefund. Allocate and assign eleven data
accounts with internally derived PDA seeds; leave five native vaults empty and
System-owned. Check exact effects after every CPI and across the complete batch.
No economic classification, reimbursement, schema, dependency or runtime selector.

This library result is an incomplete intermediate: Token initialization and
all state serialization must follow within the same successful outer transaction.
Never expose allocation as a standalone instruction or catch execution errors.
Committing bare Token-owned zero data would permit hostile initialization.
Tests must distinguish host modeled rollback from actual runtime atomicity.
One writer ran twelve focused tests; separate scope/source/test review passed.
Root inspected the source and executed 438 host tests +1 doctest/eight
locked/offline gates, with no failures or diagnostics. All 88 inputs and evidence
hashes were verified. See [report](TASK_2_21_GENESIS_ACCOUNT_ALLOCATION.md).
Task 2.21 was published and checkpointed at `cbe5611`. The founder then
explicitly requested the additional bounded Task 2.22 below, superseding the
planned STOP. Recipient/funding constraints and transport/runtime proof remain
separate readiness requirements.

### Task 2.22 - Same-call approved genesis initialization

Status: **TECHNICALLY VALIDATED / FOUNDER-AUTHORIZED MAIN INTEGRATION (D-028)**.
Scope review passed. Complete fresh approved allocation, both pinned legacy
Token InitializeAccount3 CPIs and all nine typed genesis envelopes in one call.
Authenticate the additional Token Program before effects, preserve the same
Rent/model and verify exact final bytes, topology, balances and mint preservation.
All nine state buffers must validate and be borrowed before the first copy.
Every error propagates; explicit host models do not establish runtime rollback.
No new native ABI, dependency, schema or economic prefund classification.
Recipient/funding constraints, transport and runtime resource/rollback proof
remain deferred before native initializer exposure. Ten focused tests and
separate source/test review passed; root executed 448 host tests +1 doctest/eight
locked/offline gates, zero failures or diagnostics, on the verified 90-input
freeze. See [report](TASK_2_22_GENESIS_ACCOUNT_INITIALIZATION.md).
Task 2.22 was published and checkpointed at `208b7fb`. The founder subsequently
requested the bounded Task 2.23 below, superseding that STOP.
Next assess prefund/funding and recipient constraints, exact Squads transport and
native initializer/runtime evidence requirements.

### Task 2.23 - Host genesis transport encoding and packet-size validation

Status: **TECHNICALLY VALIDATED / FOUNDER-AUTHORIZED MAIN INTEGRATION (D-028)**.
One delegated writer and separate source/test review passed. Retained pinned
web3.js 1.98.4/Node 24.19.0 construct exact synthetic Task 2.22 inner topology and
model bytes, compact Squads message and buffer create/extend/create-from-buffer
packets. Actual unsigned legacy/v0 serialization demonstrates oversized legacy
execution and fitting v0 packets with sixteen target addresses in a synthetic
outer ALT. Exact keys/bytes/union privileges, inner/outer signer separation, hash/length
reconstruction, malformed buffers and 1,232/1,233-byte boundaries are checked.
Root independently executed nine Node tests plus eight-case JSON report, matching
the writer report hash. The initial 420-byte from-buffer test expectation was
independently corrected to 421; both first failures and successful retries remain.
No Rust, dependency, economic or native ABI changes; no RPC, keys or signing.
All 90 Rust inputs remain unchanged: 448 tests +1 doctest/eight gates are retained
Task 2.22 evidence, not new executions. See [report](TASK_2_23_GENESIS_TRANSPORT.md).
Synthetic ALT availability, actual approval/buffer execution, external preparation
rent/refunds and runtime resource/rollback evidence remain deferred. Native
initializer remains closed. D-028 main integration completed; the founder then
resumed with bounded Task 2.24 below. Recipient/funding constraints and
current-source native-boundary/runtime readiness remain separate.

### Task 2.24 - Same-call genesis Token-native prefund normalization

Status: TECHNICALLY VALIDATED / PENDING FOUNDER ACCEPTANCE under D-026 after
founder resumption on 2026-09-20 UTC. Normalization is limited to the two
still-System-owned Token PDAs: retain their future
Token rent floor and move only excess into PendingSol before ownership changes.
Original preflight rent shortfalls remain entirely funded by the distinct payer;
contributions cannot replace missing destination rent. Reuse same-call Token and
state initialization through a distinct library path, preserving existing APIs.
Pre-effect overflow/role/borrow checks, exact full-target postconditions and
regressions prove subsequent authenticated pending recognition conserves value
once. Separate source/test review passed; the writer ran 16 focused tests and
root independently executed 454 host tests +1 doctest/eight gates. Both passed
first execution with 90 unchanged inputs. Nine Node tests/eight cases are retained
evidence, not rerun. See [report](TASK_2_24_GENESIS_TOKEN_PREFUND_NORMALIZATION.md).
The initial pause flag applies to newly created
Config; already initialized paused Config must reject before effects. No general
pause-policy change. No native ABI/schema/dependency
change or live-runtime claim. Operational/state-account prefunds, recipient
control and post-initialized Token donation handling remain separate. Checkpoint
and STOP followed that task; the founder subsequently resumed Task 2.25 below.

### Task 2.25 - Fresh genesis recipient-vault identity preflight

Status: TECHNICALLY VALIDATED / PENDING FOUNDER ACCEPTANCE under D-026.
A distinct read-only full genesis preflight checks both approved recipient keys
as canonical vault PDAs of the same freshly authenticated governance multisig.
It requires distinct accounts/backing, positive native balances covering current
rent, empty System ownership and nonexecutable state. Indexes only witness the
approved keys. Existing APIs, model bytes, schema and economics remain unchanged.
Separate source/test review passed. Eight focused tests passed after one verified
test-only correction; root independently ran 462 host tests +1 doctest/eight gates
with zero failures or diagnostics. All 92 inputs match the corrected freeze.
See [report](TASK_2_25_GENESIS_RECIPIENT_PREFLIGHT.md) for both execution records.

Current custody identity does not establish absence of Squads spending limits,
exclusive four-of-six spending, stale-action safety or deployed-artifact control.
The additional two recipient accounts need future composition/transport evidence;
no new native initializer or runtime proof is supplied. That task was published;
the founder subsequently resumed with Task 2.26 below.

### Task 2.26 - Recipient-checked same-call genesis initialization

Status: TECHNICALLY VALIDATED / PENDING FOUNDER ACCEPTANCE under D-026.
A fixed new normalized initialization profile checks both recipient identities
using the allocator's single fresh preflight and retained Rent before effects.
Private observations preserve both accounts after every successful System/Token
CPI and final completion. One authoritative role mapping and no public detached
proof path are retained; original invocation errors precede postchecks. Existing
profiles, model/schema/native dispatch and economics remain. Additive library
error variants and exhaustive-match implications are documented in the
[report](TASK_2_26_RECIPIENT_CHECKED_GENESIS_INITIALIZATION.md).

Separate source/test review passed. The writer ran 22 initialization and eight
recipient-preflight tests; root independently executed 468 host tests +1 doctest/
eight gates. Both passed first attempt without failures or diagnostics, with 92
unchanged inputs. Six new groups cover custody/recipient preservation, every CPI
boundary and error precedence, borrowing, actual pending recognition and replay.
Combined 35/34-account transport, full recipient control, funding provenance and
current runtime proof remain separate. Integration publication and STOP completed;
the founder subsequently resumed with Task 2.27 below.

### Task 2.27 - Recipient-checked genesis transport validation

Status: TECHNICALLY VALIDATED / PENDING FOUNDER ACCEPTANCE under D-026.
The explicit recipient-checked unsigned host profile binds Task 2.26's complete
35/34-account topology and fixed recipient witnesses to the unchanged 313-byte
model format. The old default report remains byte-identical. Buffered creation,
exact stored/outer privileges and lookup reconstruction pass; v0 packets with 16
targets measure 940/907 bytes (988/955 with illustrative prefixes). Refused SDK
serialization is distinct from actual oversized packets. No Rust/dependency/native
ABI changes, signatures or runtime operations. See the
[report](TASK_2_27_RECIPIENT_CHECKED_GENESIS_TRANSPORT.md).

Writer and root each passed 15 Node tests plus eight old/sixteen new CLI cases,
first execution without failures or diagnostics, on 96 matching inputs. Separate
review passed after a pre-execution correction to the recipient substitution
oracle. All 92 Rust inputs remain unchanged: 468 tests +1 doctest/eight gates from
Task 2.26 are retained, not rerun. Actual transport lifecycle, funding/recipient
control and current genesis runtime/resource/rollback proof remain deferred.
Integration-only publication and STOP completed; the founder subsequently
resumed with Task 2.28 below.

### Task 2.28 - Current production SBF and dispatched-path runtime refresh

Status: TECHNICALLY VALIDATED / PENDING FOUNDER ACCEPTANCE under D-026.
Current production SBF compilation exposed oversized genesis stack frames;
private preflight indirection and separate construction helpers resolved them.
Two rejected builds remain recorded; the third strict build passed unchanged
safety gates. Public/native ABI, account/model bytes, validation order,
dependencies and economics are preserved. A footprint regression was added.

Final source: 53 delegated focused tests; root 469 host tests +1 doctest/eight
gates; separate source/test review. Current 229888-byte ELF, artifact review,
fresh harness build and exact-binary execution passed 24 local tests/70 cases.
Root checked 1629 complete account records. Runner regressions: 16+7 PASS.
Prior 15 Node tests/eight old plus sixteen recipient cases are retained evidence,
not rerun. OS hash refreshes followed signed package provenance; no installation.

Only existing dispatched claim/pending paths receive runtime evidence. Added
2272 host-layout heap bytes plus alignment are cumulative; genesis total heap,
resource, CPI/rollback and live initialization remain unproved. Preserve historical
artifacts, economic decisions and sensitive-action gates. See the
[task report](TASK_2_28_CURRENT_SBF_RUNTIME_REFRESH.md) and checkpoint for exact
commands, artifact identity and failures. Integration-only publication and STOP
completed; the founder subsequently resumed with Task 2.29 below.

### Task 2.29 - Isolated genesis preflight SBF probe preparation

Status: TECHNICALLY VALIDATED / PENDING FOUNDER ACCEPTANCE under D-026.
Baseline integration: `162f3b7`. Two isolated validation probes prepare the
unchanged read-only 32/31-account recipient preflight through a synthetic caller.
Exact stored inner privileges, canonical vault signing seeds and fail-closed host
entrypoints are preserved. Production source/ABI/economics and existing harness
are unchanged; all registry versions/checksums remain in the root lock.

An initial test-compilation failure was corrected and separately reviewed.
Writer and root independently passed 10 Rust +9 runner tests; root also passed
zero-doctest discovery and warning-denied documentation. The final strict locked/
offline workspace SBF build and separate static review of both artifacts passed.
Prior production host, claim/pending runtime and transport suites are retained
after verification, not rerun. No SBF probe execution, real Squads/control or
initializer readiness is claimed. See the [report](TASK_2_29_GENESIS_PREFLIGHT_PROBES.md).

Root publishes integration only, checkpoints and stops; Task 2.30 is NOT STARTED.
Next candidate: keyless execution of these exact probes with actual height-two
CPI and runtime Instructions/Clock/Rent. Full genesis resources/rollback, funding
provenance, later Token-native donations and live readiness remain unproved.

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
