# Review Tasks 2.3–2.19: accounting, runtime and initialization foundations

This draft reviews the development sequence from accepted `main` to
`integration/piv1-testnet`. It extends the accepted pure accounting foundation
with authenticated state/custody, two narrow runtime instruction paths and
library prerequisites for initialization. It requests foundation review;
founder acceptance and merge are separate decisions.

- Adds phase-dependent custody reconciliation, initial principal and bounded
  protected-deposit models, including T23-R1 severe-loss recovery correction.
- Connects authenticated earned KIF claims to fixed System transfers and adds
  permissionless recognition of already-received pending contributions. These
  are the only dispatched instructions; recognition moves no funds.
- Adds guardian/Clock evidence, atomic existing-account byte persistence,
  current Squads authority and exact direct invocation approval checks.
- Adds approved deterministic genesis model preparation and source-pinned
  seven-account Jito identity. These remain separate prerequisites; no initializer,
  production Jito CPI or complete distribution/governance lifecycle is callable.
- Corrects stale README/master technical summaries that still denied the
  implemented entrypoint/CPI/artifact boundary (IR-001).

Confirmed economics, six-guardian/4-of-6 authority, pause behavior, protected HWM,
isolated custody and earned KIF ownership remain unchanged. Fee-bearing deposit
models, operational funding provenance and other unsupported paths are not
silently promoted into production behavior.

Validation: **416 host tests +1 doctest / eight gates PASS** on current Task 2.19
source. Separately, the **historical Task 2.14 artifact** passed **24 local SBF
tests/70 cases**; that runtime evidence does not cover later source additions,
actual initialization, Squads CPI, Bank rollback or public deployment. Root
verified unchanged source, retained logs/tools and the historical ELF. Separate
targeted integration review passed with no additional actionable foundation-level
blocker. This recap review adds documentation only and reuses the existing test
executions; it does not claim a new test run.

Before a usable PIV1 lifecycle: compose fresh approved genesis/protocol evidence,
validate and atomically create actual accounts with funding provenance, prove
initializer transport, implement protocol/governance/activity handlers and refresh
full-source runtime evidence. Official cluster/artifact identities and exact
deployment/signing/authority approvals remain separate. This PR does not assert
Testnet readiness or grant merge, acceptance or live-operation permission.

See the [integration review](https://github.com/HoldTheFuckingPosition/PIV1/blob/integration/piv1-testnet/docs/PIV1_INTEGRATION_REVIEW_2_3_TO_2_19.md) for delivered
layers, exact entrypoint reachability, T23-R1, IR-001 and remaining dependencies;
the [checkpoint](https://github.com/HoldTheFuckingPosition/PIV1/blob/integration/piv1-testnet/docs/PIV1_PILOT_STATE.md) records actual publication state. The review
baseline was `1bf07eae13d90744c9c18e7dc3f5543185bb6284` against accepted main
`66193769d1cbc59cd8630df295b9a784b9c64642` (41 commits/98 files before recap docs).
AI-assisted review is not a professional independent audit.
