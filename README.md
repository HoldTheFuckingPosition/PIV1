# PIV1

PIV1 is the first production infrastructure brick of the HTFP Project: a
Solana program designed to hold perpetual principal, use JitoSOL as its initial
strategy, and distribute conservatively measured yield under the fixed PIV1
economics.

## Current status

Phase 0, the complete Phase 1 foundation and Tasks 2.1–2.2 are
**COMPLETE / FOUNDER-ACCEPTED**. Phase 2 remains **IN PROGRESS**. Tasks 2.3–2.19
are **COMPLETE / FOUNDER-ACCEPTED** within their recorded scopes at
`d9f3371be6ecb586675e3b38edcc57bd6e9519f8` under
[D-027](docs/PIV1_DECISIONS.md).
The authorized milestone fast-forward is published to main. Necessary acceptance
records are maintained in separately reviewed documentation commits. Task 2.20
[genesis account preflight](docs/TASK_2_20_GENESIS_ACCOUNT_PREFLIGHT.md) is
**TECHNICALLY VALIDATED / FOUNDER-AUTHORIZED MAIN INTEGRATION (D-028)**. It composes fresh
genesis/Jito checks and sixteen target observations with rent-only shortfalls.
Task 2.21 [genesis allocation](docs/TASK_2_21_GENESIS_ACCOUNT_ALLOCATION.md) is also
**TECHNICALLY VALIDATED / FOUNDER-AUTHORIZED MAIN INTEGRATION (D-028)**: a distinct signing payer
funds only missing rent, with canonical System allocation/assignment and exact
postconditions. Task 2.22 [genesis initialization](docs/TASK_2_22_GENESIS_ACCOUNT_INITIALIZATION.md)
is also **TECHNICALLY VALIDATED / FOUNDER-AUTHORIZED MAIN INTEGRATION (D-028)**: it completes
allocation, both Token initializations and all nine state writes in one call,
with exact final checks. It remains a library function; native exposure,
recipient/funding constraints and genesis runtime proof remain deferred.
Task 2.23 [unsigned transport validation](docs/TASK_2_23_GENESIS_TRANSPORT.md)
shares that status: exact synthetic wire encoding fits an outer v0 ALT route;
legacy execution is oversized. D-028 authorizes main integration of Tasks
2.20–2.23 at `3282e1ebabcb0cd88491d48a391565b8b100afa7` plus reviewed records,
without broader founder acceptance or live readiness.
Task 2.24 [genesis prefund normalization](docs/TASK_2_24_GENESIS_TOKEN_PREFUND_NORMALIZATION.md)
is **TECHNICALLY VALIDATED / PENDING FOUNDER ACCEPTANCE** on integration. A distinct
same-call path moves native excess from the two unallocated Token PDAs into
PendingSol while preserving every original externally paid rent obligation.
Later donations to already Token-owned accounts remain unsupported.
Task 2.25 [recipient identity preflight](docs/TASK_2_25_GENESIS_RECIPIENT_PREFLIGHT.md)
is **TECHNICALLY VALIDATED / PENDING FOUNDER ACCEPTANCE** on integration. It
freshly verifies both approved recipients as funded System vault PDAs of the
same governance multisig without modifying accounts. Exclusive four-of-six
spending, delegated limits and live control remain separate checks.
Task 2.26 [recipient-checked initialization](docs/TASK_2_26_RECIPIENT_CHECKED_GENESIS_INITIALIZATION.md)
shares that pending-acceptance status on integration. Its fixed normalized path
checks recipient identity before effects and preserves both accounts after every
successful CPI and final completion, using one fresh full preflight.
Task 2.27 [recipient-checked transport](docs/TASK_2_27_RECIPIENT_CHECKED_GENESIS_TRANSPORT.md)
is **TECHNICALLY VALIDATED / PENDING FOUNDER ACCEPTANCE** for that complete
35/34-account fixture, with an explicitly selected unsigned host profile. Actual
transport lifecycle and genesis runtime proof remain deferred.
See the [current checkpoint](docs/PIV1_PILOT_STATE.md),
[integration review](docs/PIV1_INTEGRATION_REVIEW_2_3_TO_2_19.md) and
[execution plan](docs/PIV1_CODEX_EXECUTION_PLAN.md) for scope and provenance.

[Task 2.28](docs/TASK_2_28_CURRENT_SBF_RUNTIME_REFRESH.md) is **TECHNICALLY VALIDATED /
PENDING FOUNDER ACCEPTANCE**. A private preflight-memory correction resolves actual SBF stack diagnostics
without changing native ABI, account bytes, dependencies or economics. Final
source passed 469 host tests +1 doctest/eight gates and the strict SBF build.
The current artifact also passed 24 local SBF tests/70 cases after separate
artifact and executable review. Root verified 1629 complete account records.
Full genesis initialization still has no total heap/resource or runtime proof.

[Task 2.29](docs/TASK_2_29_GENESIS_PREFLIGHT_PROBES.md) is **TECHNICALLY VALIDATED /
PENDING FOUNDER ACCEPTANCE** within build-only preparation. Two isolated probes
prepare the existing read-only genesis recipient preflight. Writer and root each
passed 10 Rust boundary tests +9 runner tests; strict SBF compilation and separate
static artifact review passed. No SBF probe execution, native initializer or actual
Squads/control proof is claimed. Production and existing evidence inputs remain
unchanged; earlier host/runtime/transport results are retained, not rerun.

[Task 2.30](docs/TASK_2_30_GENESIS_PREFLIGHT_RUNTIME.md) is **TECHNICALLY VALIDATED /
PENDING FOUNDER ACCEPTANCE** for keyless local execution of those exact probes.
Root passed twelve runtime tests/cases plus eleven runner regressions and checked
1674 complete account records. Both profiles reach actual height-two SBF CPI;
runtime-generated Instructions and Clock/Rent are checked, with expected negative
rejections and exact account preservation. A test-support import correction
resolved the first host compilation failure; the second build and first runtime
passed separate review. Successful cases consume 288277/282070 CU with the default
32-KiB heap and explicit 1.4m-CU ceiling, exceeding 200k. The synthetic caller is
not actual Squads; read-only preflight does not prove complete initialization,
mutating rollback, full recipient control or Testnet readiness. Earlier suites
remain verified retained evidence. Task 2.30 publication is complete at `6238088`.

[Task 2.31](docs/TASK_2_31_GENESIS_INITIALIZATION_PROBES.md) is **TECHNICALLY VALIDATED /
PENDING FOUNDER ACCEPTANCE** within build-only preparation. Three isolated
artifacts prepare the existing full recipient-checked normalized initializer,
with a synthetic 35/34-account caller and a restricted canonical Token wrapper.
Root passed 15 Rust boundary tests +11 runner regressions; first strict SBF build
passed without diagnostics. Separate source/command/host-evidence review passed;
static artifact review passed. No new SBF runtime was executed: complete
initialization, total resources and mutating rollback remain unproved. Production,
economics and earlier artifacts are unchanged; historical suites are verified
retained evidence. Main remains unchanged. Save/STOP; Task 2.32 is NOT STARTED.

The native runtime-ID entrypoint currently dispatches only isolated `claim_kif`
and permissionless pending-contribution recognition. Claims use authenticated
state, byte persistence and a fixed signed System transfer; pending recognition
updates the two pending ledgers without moving funds. The crate has a `cdylib`
target. No dedicated live PIV1 Program ID or deployment is established.

For Task 2.27, root and writer each executed **15 Node tests PASS**, plus **eight
old-profile and sixteen recipient-profile CLI cases**. The default CLI output
remains byte-identical to Task 2.23. Separate source/test review passed; a test
oracle was corrected before execution. At that freeze, all 96 inputs and twelve
new log hashes matched. **468 Rust tests +1 doctest/eight gates** are retained Task 2.26 evidence,
not rerun in Task 2.27, after verification of all 92 unchanged Rust inputs and retained logs.
Separately, the **historical Task 2.14 artifact** passed **24 local SBF tests
across 70 cases** for claims and pending recognition. That historical runtime evidence does
not cover later source additions; see Task 2.28 for the current refresh.
The accepted Phase 0 direct Jito lifecycle
proof used one withdrawal leg on public Testnet; it did not deploy this PIV1
program or establish production multi-validator orchestration.

Accounting models, account/guardian authentication, bounded Squads authorization,
approved genesis preparation, Jito identity and initialization are library layers.
Native initializer integration, recipient/funding constraints, governance handlers,
production SPL/Jito CPI and the complete distribution lifecycle remain deferred.
This foundation is not a complete locally executable or Testnet-ready PIV1.

[D-026](docs/PIV1_TECHNICAL_PILOT_MANDATE.md) permits bounded reviewed progression
on `integration/piv1-testnet`. D-027 records explicit milestone acceptance;
D-028 separately authorizes main integration of Tasks 2.20–2.23. Neither grants
deployment, key/signing, fund movement or authority-transfer permission. AI-assisted review is not a
professional independent audit.

## Project identity

- **CONFIRMED** official project email: `HoldTheFuckingPosition1@protonmail.com`.
- **CONFIRMED** official GitHub account: <https://github.com/HoldTheFuckingPosition>.
- **CONFIRMED** public repository: <https://github.com/HoldTheFuckingPosition/PIV1>.

Canonical files:

- [`docs/PIV1_DECISIONS.md`](docs/PIV1_DECISIONS.md): highest-authority decision register.
- [`docs/PIV1_MASTER_SPEC.md`](docs/PIV1_MASTER_SPEC.md): consolidated product, economic, technical, security, and deployment specification.
- [`docs/PIV1_CODEX_EXECUTION_PLAN.md`](docs/PIV1_CODEX_EXECUTION_PLAN.md): phased development plan and acceptance gates.
- [`docs/PHASE_0_VALIDATION_REPORT.md`](docs/PHASE_0_VALIDATION_REPORT.md): accepted Phase 0 evidence and production architecture.
- [`docs/research/PIV1_TASK_0_4_JITO_VALIDATION.md`](docs/research/PIV1_TASK_0_4_JITO_VALIDATION.md): public-Testnet and local-probe evidence.

Important:

- No mainnet deployment, authority transfer, real-fund movement, or secret handling is authorized by this pack.
- Current external addresses, program versions, fees, and protocol constraints must be reverified against official sources before use.
- The new development chat must not reopen confirmed product decisions unless a verified technical incompatibility is found.

## Accepted V1 foundations

- Confirmed JitoSOL direct deposit and delayed direct withdrawal; no Jupiter/DEX core path.
- Confirmed six guardians with 4/6 authority and current KIF rules.
- Confirmed slippage-protected SPL instructions with a 1-bps immutable hard cap.
- Confirmed one active distribution at a time, with as many deterministic
  validator withdrawal legs as safely required to assign its exact fixed target.
- Confirmed distinct principal/pending JitoSOL token accounts at PIV1-derived
  addresses, both controlled by the shared PIV authority and neither an ATA.
- Confirmed a recyclable operational rent reserve excluded from principal and yield.
- Confirmed exact 30-day KIF periods, repeated zero-active carry, and explicit
  carry of active-guardian division remainder.
- Confirmed `claim_kif` remains allowed during a global pause only to pay an
  already-earned recorded liability from the isolated `KifSolVault`, subject to
  guardian-controlled destination, exact-liability, balance, and atomicity
  constraints. Its narrow runtime boundary is technically validated within
  the [claim report](docs/TASK_2_11_KIF_CLAIM_INSTRUCTION_BOUNDARY.md) and
  [historical local SBF evidence](docs/TASK_2_14_RUNTIME_PENDING_RECONCILIATION.md).
