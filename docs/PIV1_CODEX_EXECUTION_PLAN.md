# PIV1 Codex Execution Plan v0.2

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

## Phase 1 tasks

### Task 1.1 - Scaffold Anchor workspace

Accepted Task 1.1 scope: **scaffold the modular Anchor workspace and
compile-only placeholders on a new branch from the accepted main baseline.** Create no
economic implementation, Jito CPI, deployment, key, or fund-moving test.

Status: **COMPLETE / FOUNDER-ACCEPTED**. Accepted implementation commit:
1d436570570fc31310e3e5d2c1d4d5e92320c65b.

### Task 1.2 - Implement pure math crate

Status: **COMPLETE / FOUNDER-ACCEPTED**. Accepted implementation commit:
43a3b7497653ff7a246a1e5cf9b760086dd33fcd.

Task 1.3 is **IMPLEMENTED / PENDING FOUNDER ACCEPTANCE**. Task 1.4 has not
started. The exact next action is **founder review of Task 1.3**.

Implement and test:

- basis-point split;
- high-water mark;
- yield calculation interfaces;
- KIF zero-active 50/50 rule;
- KIF equal active split;
- conservative rounding;
- checked arithmetic.

### Task 1.3 - State and transition model

Status: **IMPLEMENTED / PENDING FOUNDER ACCEPTANCE** on the dedicated
`task/1.3-state-transition-model` branch.

Implement account structs and transition validation without Jito CPI.

### Task 1.4 - Property tests

Status: **NOT STARTED**.

Add randomized tests for invariants.

Stop for architecture review.

Exact next action: **founder review of Task 1.3**. Do not begin Task 1.4 before
that review.

## Phase 2 tasks

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
