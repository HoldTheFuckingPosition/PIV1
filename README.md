# PIV1

PIV1 is the first production infrastructure brick of the HTFP Project: a
Solana program designed to hold perpetual principal, use JitoSOL as its initial
strategy, and distribute conservatively measured yield under the fixed PIV1
economics.

## Current status

Phase 0, the complete Phase 1 foundation and Tasks 2.1–2.2 are
**COMPLETE / FOUNDER-ACCEPTED**. Phase 2 remains **IN PROGRESS**. Tasks 2.3–2.19
are **TECHNICALLY VALIDATED / PENDING FOUNDER ACCEPTANCE** within their recorded
scopes. See the [current checkpoint](docs/PIV1_PILOT_STATE.md),
[integration review](docs/PIV1_INTEGRATION_REVIEW_2_3_TO_2_19.md) and
[execution plan](docs/PIV1_CODEX_EXECUTION_PLAN.md) for scope and provenance.

The native runtime-ID entrypoint currently dispatches only isolated `claim_kif`
and permissionless pending-contribution recognition. Claims use authenticated
state, byte persistence and a fixed signed System transfer; pending recognition
updates the two pending ledgers without moving funds. The crate has a `cdylib`
target. No dedicated live PIV1 Program ID or deployment is established.

Current-source validation is **416 host tests +1 doctest / eight gates PASS**.
Separately, the **historical Task 2.14 artifact** passed **24 local SBF tests
across 70 cases** for claims and pending recognition. That runtime evidence does
not cover later source additions. The accepted Phase 0 direct Jito lifecycle
proof used one withdrawal leg on public Testnet; it did not deploy this PIV1
program or establish production multi-validator orchestration.

Accounting models, account/guardian authentication, bounded Squads authorization,
approved genesis model preparation and Jito account identity are library layers.
Actual initialization, authenticated composition/creation, governance handlers,
production SPL/Jito CPI and the complete distribution lifecycle remain deferred.
This foundation is not a complete locally executable or Testnet-ready PIV1.

[D-026](docs/PIV1_TECHNICAL_PILOT_MANDATE.md) permits bounded reviewed progression
on `integration/piv1-testnet`; `main` remains founder-accepted. A review PR does
not grant merge, acceptance, deployment, key/signing, fund movement or authority
transfer permission. AI-assisted review is not a professional independent audit.

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
