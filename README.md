# PIV1

PIV1 is the first production infrastructure brick of the HTFP Project: a
Solana program designed to hold perpetual principal, use JitoSOL as its initial
strategy, and distribute conservatively measured yield under the fixed PIV1
economics.

## Current status

- **CONFIRMED** Phase 0 and Task 0.5 are complete and founder-accepted.
- **CONFIRMED** the direct JitoSOL custody lifecycle was validated with one
  withdrawal leg on public Testnet.
- **CONFIRMED** production V1 requires a bounded multi-validator withdrawal-leg
  architecture; Task 1.3 now implements its pure cumulative state orchestration,
  but no handler, CPI, localnet, or live-cluster orchestration has been tested.
- **COMPLETE / FOUNDER-ACCEPTED** Task 1.1 production workspace scaffolding;
  accepted implementation commit:
  1d436570570fc31310e3e5d2c1d4d5e92320c65b.
- **COMPLETE / FOUNDER-ACCEPTED** Task 1.2 pure math crate; accepted
  implementation commit:
  43a3b7497653ff7a246a1e5cf9b760086dd33fcd.
- **COMPLETE / FOUNDER-ACCEPTED** Task 1.3 bounded state and transition model;
  final accepted implementation tip:
  527e381661fe0cfc27e07ad9b44e1601a638ae75.
- **COMPLETE / FOUNDER-ACCEPTED** Task 1.4 reproducible randomized/property,
  adversarial state-machine, and serialization/layout invariant testing;
  accepted implementation commit:
  06c39429f3237f6974e21217670c3f0d30b0a571.
- **COMPLETE / FOUNDER-ACCEPTED** the complete Phase 1 specification-as-code
  foundation.
- **COMPLETE / FOUNDER-ACCEPTED** Task 2.1 narrow deterministic stake-pool
  interface, fixed-capacity host-only mock, and deterministic tests; accepted
  initial implementation commit `33b1e539f969432f82635d1ca76c59d89f0ec233`
  and final corrected tip `cb90d468eff4dce60552ba15b2b267b364a47827`.
  This acceptance does not promote mock behavior to exact SPL/Jito behavior;
  the real protocol mapping remains Phase 3 work.
- **IN PROGRESS** Phase 2. Task 2.2 and later Phase 2 tasks are **NOT STARTED**;
  the exact next action is separate founder authorization of Task 2.2.
- The Phase 1 review was AI-assisted and is not a professional independent
  audit. Handler, CPI, localnet, external-account, and live-cluster validation
  remain deferred.
- No Mainnet deployment, production Program ID, guardian keys, recipient
  addresses, real-fund movement, or authority transfer is authorized or
  recorded here.
- The program boundary remains deliberately non-deployable: the interface and
  host mock add no Program ID, entrypoint, handler, CPI, account context,
  program/cluster mapping, usable provider wallet, or SBF cdylib.

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
  constraints. Its handler remains unimplemented for separately authorized
  Phase 2 work.
