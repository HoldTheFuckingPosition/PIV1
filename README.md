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
recipient/funding constraints and current runtime proof remain deferred.
Task 2.23 [unsigned transport validation](docs/TASK_2_23_GENESIS_TRANSPORT.md)
shares that status: exact synthetic wire encoding fits an outer v0 ALT route;
legacy execution is oversized. D-028 authorizes main integration of Tasks
2.20–2.23 at `3282e1ebabcb0cd88491d48a391565b8b100afa7` plus reviewed records,
without broader founder acceptance or live readiness.
See the [current checkpoint](docs/PIV1_PILOT_STATE.md),
[integration review](docs/PIV1_INTEGRATION_REVIEW_2_3_TO_2_19.md) and
[execution plan](docs/PIV1_CODEX_EXECUTION_PLAN.md) for scope and provenance.

The native runtime-ID entrypoint currently dispatches only isolated `claim_kif`
and permissionless pending-contribution recognition. Claims use authenticated
state, byte persistence and a fixed signed System transfer; pending recognition
updates the two pending ledgers without moving funds. The crate has a `cdylib`
target. No dedicated live PIV1 Program ID or deployment is established.

Retained current-source evidence is **448 host tests +1 doctest / eight gates
PASS**, plus **9 Node tests/eight transport cases**. All source inputs and logs
were reverified; no tests were rerun for this documentation-only main integration.
Separately, the **historical Task 2.14 artifact** passed **24 local SBF tests
across 70 cases** for claims and pending recognition. That runtime evidence does
not cover later source additions. The accepted Phase 0 direct Jito lifecycle
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
