# PIV1

PIV1 is the first production infrastructure brick of the HTFP Project: a
Solana program designed to hold perpetual principal, use JitoSOL as its initial
strategy, and distribute conservatively measured yield under the fixed PIV1
economics.

## Current status

**D-031 — Founder-authorized main integration (2026-09-28):** the founder
explicitly requests publication of all already-validated work missing from main.
This covers Tasks 2.33–2.39 and D-030 guidance through
`bf32d87e06a2d54c8e1c0192faaa7855d246dbc0`, plus the reviewed authorization record,
by normal fast-forward from `4cc4ea11e87c2f1a2f85b9ed48358f2a881821b3`.
Git and the [pilot checkpoint](docs/PIV1_PILOT_STATE.md) record the resulting publication.
Status: **TECHNICALLY VALIDATED / FOUNDER-AUTHORIZED MAIN INTEGRATION**.
This is integration authority, not broader founder acceptance or live-operation
permission. At publication M2 had not started; see the current checkpoint for later
integration-only work.

**Current priority — D-030:** converge on a complete founder-testable PIV1
lifecycle on Solana Testnet. The real production initializer is exposed and locally
validated. Next implement production lifecycle handlers, the pinned SPL/Jito adapter, complete local
end-to-end execution and the exact Testnet handover package. Stop before live
deployment for explicit founder authorization. Validation-only work must close
a demonstrated blocker on that path; later main updates require explicit founder authority. See the
[ordered plan](docs/PIV1_CODEX_EXECUTION_PLAN.md) and
[active checkpoint](docs/PIV1_PILOT_STATE.md). Economics and founder acceptance
remain unchanged.

Current implementation: [Task 2.46](docs/TASK_2_46_WITHDRAWAL_PREPARATION.md) is
**TECHNICALLY VALIDATED / PENDING FOUNDER ACCEPTANCE** on integration.
Production withdrawal preparation now binds an active source, runtime Stake minimum,
current rent and conservative multileg/HWM proofs. Root passed 10 focused tests, 551 host tests +1 doctest/eight gates and strict SBF
with separate review. Valid insufficiency updates only its clock/event. Actual
protected leg execution, new-path VM/Bank and full lifecycle remain open. Main
stays `8912cfe`; economics and live gates remain unchanged.

Previous implementation: [Task 2.45](docs/TASK_2_45_LIQUID_DISTRIBUTION_PREPARATION.md) is
**TECHNICALLY VALIDATED / PENDING FOUNDER ACCEPTANCE** on integration.
Production no-yield/liquid-funded preparation now uses current pool valuation and
guardian/Clock snapshots. Ten focused tests, 538 host tests +1 doctest/eight gates
and strict SBF passed with separate review. Withdrawal shortfall rejects before
any effects/cooldown until real protocol minimum/source proofs exist. New-path
VM/Bank and full lifecycle remain unproved; main stays `8912cfe`.

Previous checkpoint: [Task 2.44](docs/TASK_2_44_PROTECTED_PRINCIPAL_DEPOSIT_RUNTIME.md)
is **TECHNICALLY VALIDATED / PENDING FOUNDER ACCEPTANCE** on integration.
Protected principal SOL staking now uses the pinned deposit instruction with exact
pool/Mint/custody checks. Ten focused tests, 528 host tests +1 doctest/eight gates
and strict SBF passed with separate review. Zero-fee, historical-value/HWM,
slippage and pause protections remain. New-path VM/Bank and complete lifecycle
proof remain open; next connect production distribution preparation. Main stays
at `8912cfe`; no live operation or broader acceptance is implied.

Previous checkpoint: [Task 2.43](docs/TASK_2_43_INITIAL_BOOTSTRAP_RUNTIME.md)
is **TECHNICALLY VALIDATED / PENDING FOUNDER ACCEPTANCE** on integration.
Initial contributions now become principal through authenticated current pool
valuation and exact signed custody transfers. Ten focused tests, 518 host tests
+1 doctest/eight gates and strict SBF passed, with separate review. No new-path
VM/Bank execution is claimed. Next connect principal SOL to the real protected
pool deposit, preserving accepted zero-fee/HWM guards. Main remains `8912cfe`;
remaining distribution/integration, M2–M6 and live gates remain open.

Previous checkpoint: [Task 2.42](docs/TASK_2_42_ECONOMIC_NORMALIZATION_RUNTIME.md)
is **TECHNICALLY VALIDATED / PENDING FOUNDER ACCEPTANCE** on integration.
Production normalization moves only proven economic surplus to pending custody,
with each vault's obligations and all per-CPI postconditions preserved. Ten focused
tests, 508 host tests +1 doctest/eight gates and strict SBF passed, with separate
review. Reversible archival preserved build evidence; retain all recovery archives.
New-path VM/Bank execution remains unproved. Next: principal bootstrap/integration
with authenticated valuation and explicit Token-native compatibility. Main remains
`8912cfe`; M2 and live gates stay open.

Previous checkpoint: [Task 2.41](docs/TASK_2_41_CONTRIBUTION_RUNTIME_INTAKE.md)
is **TECHNICALLY VALIDATED / PENDING FOUNDER ACCEPTANCE** on integration.
Explicit SOL/JitoSOL intake enforces D-032 pause rejection and exact pending-custody
checks. Root passed nine focused tests, 498 host tests +1 doctest/eight gates and
strict SBF compilation, with separate review. Reversible cache archival restored
build capacity; preserve the recovery archives. New-path VM/Bank execution remains
unproved. Next connect pending custody to principal bootstrap/integration with
authenticated pool accounting. Main remains `8912cfe`; M2 and live gates stay open.

Previous checkpoint: [Task 2.40](docs/TASK_2_40_GUARDIAN_RUNTIME_OPERATIONS.md)
is **TECHNICALLY VALIDATED / PENDING FOUNDER ACCEPTANCE** on integration.
Production heartbeat and authenticated pause/unpause passed 10 focused tests,
489 host tests +1 doctest/eight gates, strict SBF compilation and separate review.
No new-path runtime execution is claimed; M2 remains in progress. Main stays at
`8912cfe`. D-032 now requires explicit SOL/JitoSOL deposits to reject during pause;
direct incoming transfers remain reconcilable. The next block implements that
contribution intake with real transfers and pending-custody checks.

Previous checkpoint: [Task 2.39](docs/TASK_2_39_PRODUCTION_INITIALIZER.md), D-030 M1,
is **TECHNICALLY VALIDATED / FOUNDER-AUTHORIZED MAIN INTEGRATION (D-031)**. The real production
instruction now performs the complete recipient-checked normalized initialization.
Root passed 479 host tests +1 doctest/eight gates, nine runner tests and twelve
production Bank messages; separate review passed and 786 complete account records
were independently checked. This closes M1's local boundary/runtime scope, not
full lifecycle or live readiness. D-031 authorizes this reviewed checkpoint
on main. M2 economic runtime handlers are next. Exact ABI, artifacts, retained
preflight failure and limitations are in the report/checkpoint.

Previous checkpoint: [Task 2.38](docs/TASK_2_38_BANK_GENESIS_INITIALIZATION.md)
is **TECHNICALLY VALIDATED / PENDING FOUNDER ACCEPTANCE**. Root passed 15 runner
tests and twelve message cases across four real local Bank profiles, independently
checking 786 full account records. Exact initializer probes now demonstrate
late-failure rollback, retained fees, no-fee replay rejection and successful retry
with correct original rent and prefund normalization. The first strict build and
runtime passed. Separate source/binary and independent evidence reviews passed.
Unsigned local entry still exceeds the public packet limit; real Squads/ALT and
failed in-initializer CPI behavior remain open. Production/economics are unchanged;
publication is integration-only, main remains `4cc4ea1`. About 3.73 GiB remains.
The retained Task 2.38 evidence remains valid; D-030 determines the next work.

Previous checkpoint: [Task 2.37](docs/TASK_2_37_BANK_COMMIT_ROLLBACK_SMOKE.md)
is **TECHNICALLY VALIDATED / PENDING FOUNDER ACCEPTANCE**. Root passed 12 runner
regressions and four message cases on the real local Bank; root and separate
reviewer independently checked 100 complete account records. Native System
success, non-fee rollback, retained fees, replay rejection and same-Bank retry
are verified. Three rejected builds and reviewed compatibility corrections are
retained; the final strict build has no diagnostics. This does not execute PIV1's
initializer or establish Testnet readiness. Production/economics are unchanged;
publication is integration-only, with main at `4cc4ea1`. About 4.93 GiB remains.
[Checkpoint](docs/PIV1_PILOT_STATE.md) saved; **Task 2.38 is NOT STARTED.**

Previous checkpoint: [Task 2.36](docs/TASK_2_36_BANK_DEPENDENCY_PREPARATION.md)
is **TECHNICALLY VALIDATED / PENDING FOUNDER ACCEPTANCE** for isolated Bank
dependency preparation. The 589-package closure and extracted sources were
verified; public resolution and locked/offline metadata passed, with separate
review. The unsigned Bank API path and fee/replay limits are documented.
No compilation, test or Bank execution occurred. Next prepare the bounded real
Bank harness/build, including native and resource requirements. Existing PIV1
behavior is unchanged; publication is integration-only and main stays `4cc4ea1`.
[Checkpoint](docs/PIV1_PILOT_STATE.md) saved; **Task 2.37 is NOT STARTED.**

Previous checkpoint: [Task 2.35](docs/TASK_2_35_BUILD_CAPACITY_RECOVERY.md)
is **TECHNICALLY VALIDATED / PENDING FOUNDER ACCEPTANCE** for reversible build-capacity
recovery. Root passed 22 focused tests and a real archive/restore demonstration;
separate review passed. About **8.23 GiB** is now available. All recorded
executables and validation evidence are preserved; retain the documented recovery
archive. No Bank runtime or new PIV1 behavior is claimed. Next prepare the genuine
pinned Bank dependencies. Publication is integration-only; main remains `4cc4ea1`.
[Checkpoint](docs/PIV1_PILOT_STATE.md) saved; **Task 2.36 is NOT STARTED.**

Previous checkpoint: [Task 2.34](docs/TASK_2_34_BANK_RUNTIME_PREREQUISITES.md)
is **TECHNICALLY VALIDATED / PENDING FOUNDER ACCEPTANCE** for read-only Bank
prerequisite tooling. Root passed 19 focused Python tests first attempt; separate
review passed. The actual check correctly reports NOT_READY: about 3.1 GiB free
and direct candidate Bank packages absent. No heavy build, installation or cleanup
occurred. This adds no Bank rollback or Testnet-readiness evidence. Next recover
build capacity without losing evidence, then prepare a genuine pinned Bank harness.
Reviewed publication is integration-only under D-026; main remains `4cc4ea1`.
See the [pilot checkpoint](docs/PIV1_PILOT_STATE.md). **Save/STOP; Task 2.35 is
NOT STARTED.**

Previous checkpoint: [Task 2.33](docs/TASK_2_33_GENESIS_INITIALIZATION_FAILURE_RUNTIME.md)
is **TECHNICALLY VALIDATED / PENDING FOUNDER ACCEPTANCE**. Root passed 13 runner
tests and eight local runtime tests/sixteen messages; separate review passed,
including independent verification of 2400 complete account records. Fixed
resource failures inside initialization preserve exact expected partial raw state;
each returned-original vector succeeds on retry. This is local Mollusk output
discard evidence, not Bank rollback or complete Testnet readiness. Both initial
test-fixture/classification failures and their reviewed corrections are recorded.
Production/economics and prior artifacts remain unchanged. D-026 covers reviewed
integration-only publication; main remains at `4cc4ea1`. **Save/STOP; Task 2.34 is
NOT STARTED.**

Previous checkpoint: [Task 2.32](docs/TASK_2_32_GENESIS_INITIALIZATION_RUNTIME.md)
is **TECHNICALLY VALIDATED / FOUNDER-AUTHORIZED MAIN INTEGRATION (D-029)**.
Root passed 13 local runtime cases and 13 runner tests; separate review passed,
including independent checks of 2040 complete account records. Both fixed genesis
profiles initialize through the unchanged Task 2.31 artifacts with actual System
and restricted Token execution. Four successes use 978083–1063693 CU with 32-KiB
heap and an explicit 1.4m ceiling. A failed first build and first runtime assertion
were resolved by reviewed test-only corrections; both attempts remain documented.
Late-failure output discard does not establish Bank rollback or failed-CPI
atomicity within initialization. Actual Squads/ALT, funding provenance, complete
recipient control, native initializer exposure and Testnet readiness remain open.
D-029 authorizes the reviewed Tasks 2.24–2.32 sequence for normal main/integration
publication after final checks; Git records the exact commit. Broader founder
acceptance and live-operation gates are unchanged. **Save/STOP; Task 2.33 is NOT
STARTED.** The task-specific records below retain their historical evidence limits.

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
That task's runtime evidence covers claim/pending paths; Task 2.32 separately
adds bounded full-genesis initialization evidence.

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
retained evidence. Task 2.31 publication completed at `60193d6`; Task 2.32 above
adds runtime evidence without extending these historical build-only claims.

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
D-028 separately authorizes main integration of Tasks 2.20–2.23, and D-029 covers
the reviewed Tasks 2.24–2.32. These publication decisions do not grant
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
