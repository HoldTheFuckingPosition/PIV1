# PIV1

PIV1 (Perpetual Income Vault 1) is the first infrastructure component of the
HTFP Project. It is a Solana program designed to protect locked principal and
distribute perpetual income from Solana staking rewards. PIV1 uses JitoSOL to
earn staking yield and distributes only the amount that can be paid without
reducing protected principal.

## Current status

**Updated 2026-10-19: M5 preparation is underway; PIV1 is not yet deployed on
Solana Testnet. The public founder-testing package is not ready.**

The reviewed work through the offline governed pause/release planner was published
to `main` at `a28ff7c`. Publication records technical integration; it does not
establish founder acceptance of every later task or authorize deployment.
The [pilot checkpoint](docs/PIV1_PILOT_STATE.md) records continuing work;
the [M5 readiness register](docs/PIV1_M5_TESTNET_READINESS.md) owns the current
readiness assessment and remaining blockers. Older task reports describe their
own historical scope, rather than today's overall project status.

| Area | Current position |
| --- | --- |
| Production implementation | The native initializer, economic lifecycle handlers, protected SPL/Jito adapter, guardian operations and recipient updates are implemented with scoped local validation. |
| Local lifecycle evidence (M4) | Complete supported local production lifecycles have passed, including delayed withdrawal legs, Stake cooldown, settlement, pending integration and earned KIF claims. M4 is still technically incomplete: one internal-callee-failure evidence criterion remains open and unwaived. |
| Testnet preparation (M5) | Explicitly authorized under D-039 despite the M4 reservation. Compatibility work, offline operator tools and the founder package are in preparation; public readiness is not established. |
| Actual Testnet setup | Dedicated disposable test identities were funded and a six-member, 4-of-6 Squads multisig was created and finalized under the narrow D-041/D-043 authorizations. This is external governance setup, not deployment or execution of PIV1. |
| Deployment | PIV1 Testnet deployment and authority transfer have not occurred and remain unauthorized. Mainnet readiness is not established. |

See the [M4 closure review](docs/PIV1_M4_CLOSURE_STATUS.md) for the exact evidence
reservation. The stopped operation and its drafts remain stopped; M5 preparation
does not waive that criterion or accept the pending delivery-scope proposal.

## What has been validated

Retained reviewed host, SBF and local Bank evidence covers the production
initializer and supported economic paths: contributions and bootstrap, protected
principal staking, distribution preparation, multiple delayed withdrawal legs,
Stake deactivation/cooldown and finalization, atomic settlement, pending
integration, guardian activity and earned KIF claims. Recipient replacement and
guardian synchronization have additional scoped local runtime evidence.

M5 preparation also includes local compatibility execution using retained Testnet
Squads/Token images for initialization, and retained pool/Token images for a
supported pool lifecycle. These runs use documented synthetic accounts, clocks,
authorities or unsigned transport; they are not a complete public Testnet PIV1
lifecycle. Their precise artifact bindings, results and limitations are recorded
in the [readiness register](docs/PIV1_M5_TESTNET_READINESS.md).

The most recent offline pause/release planner passed seven Node test groups.
Six synthetic CLI reports contain 42 complete unsigned packets independently
checked by root and a separate reviewer. These checks validate encoding and
composition, not actual approvals, signer control, funding, resource sufficiency
or successful on-chain pause/release. This README update reruns no application
tests, builds, local runtime or chain observations; all these results are retained
evidence. AI-assisted technical review is not a professional independent audit.

## Testnet setup and operator preparation

The [public Squads setup record](docs/PIV1_TESTNET_SQUADS_SETUP.json) records the
finalized Testnet multisig:

```text
9mwto7RNokYc8n8MPtzHpubwwnKcborY2r9RnfZAScVS
```

It has six members and a four-vote threshold. The six disposable guardian keys
are held on one VPS; this is not independent guardian custody. Multisig creation
used two actual signatures and does not prove four real proposal approvals or a
PIV1 economic lifecycle. D-043's one-shot creation/return authorization is consumed;
it grants no automatic retry, refill or later spending permission.

The [offline CLI guide](clients/cli/README.md) documents public-input planners and
inspectors for initialization and rent funding, contributions/bootstrap, protected
principal staking, distribution and withdrawal operations, guardian activity and
KIF claims, state inspection, Squads creation, and governed pause/release.
Complete unsigned plans remain offline preparation; incomplete inputs are not
filled with invented identities or defaults. The existing Squads creation must
not be repeated.

The [founder workflow](docs/PIV1_TESTNET_FOUNDER_WORKFLOW.md) and
[public package draft](docs/PIV1_TESTNET_PACKAGE_DRAFT.json) explain the remaining
handover. Before founder testing, the project still needs to:

1. Complete the exact PIV program identity/artifact, ALT, recipient and authority
   mapping without silently selecting unresolved governance inputs.
2. Finish the remaining governed operator composition and exact founder workflow,
   with current fees, rent, funding and resource requirements.
3. Review the complete package and resolve or explicitly retain each readiness
   condition, including the open M4 reservation.
4. Present the exact deployment scope and obtain explicit founder authorization
   before the first live PIV1 Testnet deployment.

Preparation or GitHub publication does not authorize key handling, signing,
transaction submission, fund movement, deployment or authority transfer beyond
an explicitly approved live-operation scope. No Mainnet operation is authorized.

## Accepted economic foundations

- Principal is perpetual; contributions create no individual withdrawal,
  ownership or reward rights.
- SOL and JitoSOL are accepted. The core strategy uses direct Jito stake-pool
  deposit and delayed withdrawal, without a Jupiter/DEX swap path.
- Yield allocation is 59% HTFP SOL reserve, 19.5% compound, 19.5% Team Owner Pool
  and 2% active KIF guardians. Beneficiary payments are in SOL.
- Principal and the high-water mark are accounted in SOL lamports. Conservative
  integer arithmetic and pool-derived valuation protect principal; a loss does
  not lower the high-water mark.
- Only one distribution may be active. Contributions received during it remain
  pending until the defined integration/reconciliation step.
- Governance has six guardians and a 4-of-6 authority threshold. Earned KIF
  liabilities and carry follow the accepted activity-period rules.
- Explicit SOL/JitoSOL deposits are blocked during emergency pause. Already-earned
  KIF claims remain permitted under their authenticated liability and custody
  checks; direct incoming transfers remain reconcilable.

The [decision register](docs/PIV1_DECISIONS.md) and
[master specification](docs/PIV1_MASTER_SPEC.md) govern the complete rules.
This overview does not change accepted economics or founder acceptance.

## Repository guide

| Location | Purpose |
| --- | --- |
| [Program](programs/piv1) | Production Solana program and account/instruction logic. |
| [Math crate](crates/piv1-math) | Integer economic calculations. |
| [CLI guide](clients/cli/README.md) | Offline public-manifest planning and inspection tools. |
| [Test guide](tests/README.md) and [test plan](docs/PIV1_TEST_PLAN.md) | Validation procedures, evidence scopes and remaining gates. |
| [Decision register](docs/PIV1_DECISIONS.md) | Confirmed founder decisions and explicit authorization boundaries. |
| [Master specification](docs/PIV1_MASTER_SPEC.md) | Product, economic, technical and security requirements. |
| [Execution plan](docs/PIV1_CODEX_EXECUTION_PLAN.md) | Ordered milestones and acceptance gates. |
| [Pilot checkpoint](docs/PIV1_PILOT_STATE.md) | Durable continuation state and evidence references. |
| [M5 readiness register](docs/PIV1_M5_TESTNET_READINESS.md) | Single current Testnet readiness assessment. |
| [Phase 0 report](docs/PHASE_0_VALIDATION_REPORT.md) | Accepted historical feasibility evidence. |

## Project identity

- Official project email: `HoldTheFuckingPosition1@protonmail.com`.
- Official GitHub account: <https://github.com/HoldTheFuckingPosition>.
- Public repository: <https://github.com/HoldTheFuckingPosition/PIV1>.
