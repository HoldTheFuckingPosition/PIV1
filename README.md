# PIV1

PIV1 (Perpetual Income Vault 1) is the first infrastructure component of the
HTFP Project. It is a Solana program designed to protect locked principal and
distribute perpetual income from Solana staking rewards. PIV1 uses JitoSOL to
earn staking yield and distributes only the amount that can be paid without
reducing protected principal. A portion of the yield is reinvested into principal
to support its growth over time.

## Current status

**Updated 2026-10-21: the founder confirms the exact disposable Testnet artifact/bootstrap exception and accepts the supported local M4 deliverable with its evidence reservation. M4 remains technically incomplete/open/unwaived; public startup package NOT READY.**

Since main checkpoint `e2bb2a3`, integration preparation adds a fixed-origin collector that reads the sealed Squads accounts and Clock together, then applies the existing storage verifier and native timelock arithmetic. Root passes 12 new tests, five CLI checks and six independent simulated response cases. Zero actual network calls occurred; raw evidence and the parent fee ledger are preserved. Separate source review passed.

This closes the collected-byte composition in offline fixtures. Actual readbacks/freshness, causal storage-before-votes, vote signatures, controlled continuation/signing/submission, full oracle/resources/lifetimes/capacity and final readiness review remain open. Timing arithmetic does not establish live release or eligibility. The real anchor stays null and the unexplained +10 SOL source mismatch remains retained. M4 stays technically incomplete/open/unwaived; the public package is NOT READY. D-049 requires separate exact live authorization; D-050's main publication is complete and this new work is integration only.

See the [pilot checkpoint](docs/PIV1_PILOT_STATE.md), [M5 readiness register](docs/PIV1_M5_TESTNET_READINESS.md), [composed readback evidence](docs/PIV1_TESTNET_GOVERNANCE_READBACK.json) and [startup preparation card](docs/PIV1_TESTNET_STARTUP_AUTHORIZATION.md).

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

The retained [local startup proof](docs/PIV1_TESTNET_STARTUP_GOVERNED_INITIALIZER.json)
passes native deployment, real Squads governed initialization and a production
heartbeat: 11 local entries, 23 zero signature slots and 115,000 local fees.
Root/separate audits check exact states, resources and cashflows. All four builds
and three process outcomes remain recorded, including harness/compiler failures.
Unsigned Bank 4.2 with Buffer/external/recipient/Clock fixtures does not prove
public signer control, upload, full lifecycle or exact Testnet 4.4 compatibility.
That local checkpoint made no chain observation, signing or live action. Older evidence retains
its original limits; AI-assisted review is not a professional independent audit.

The retained [consolidated startup budget](docs/PIV1_TESTNET_CONSOLIDATED_STARTUP_BUDGET.json)
checks all 949 unsigned sizing packets/963 zero slots and seven host groups.
Fifteen actual representative fee quotes plus 934 unquoted Write projections give
4,815,000 projected fees. The conditional H2 envelope is 4,594,387,560 lamports,
including one required payer residual; headroom is 102,076,760. Creator buffer
refund belongs only to G0. These are startup-only projections, not complete capital
or spending authority. The recorded blockhash validity is historical and does not
prove upload lifetime; artifact, bootstrap trust and the real Clock anchor remain
unselected. No new Bank/build/signing/simulation/send or funds movement occurred.

The retained [offline upload-resume client](docs/PIV1_TESTNET_UPLOAD_RESUME.md) now plans
up to 64 unsigned Write packets from supplied public Buffer bytes, preserving
matching chunks and rejecting unexpected content. Eight host groups pass,
including full 936-write replay in 15 batches; three CLI plans retain 67 audited
unsigned packets. This closes the bounded encoding/progress-planning gap only:
no current-state authentication, whole-upload lifetime or live upload is proved.

The new [bounded startup release plan](docs/PIV1_TESTNET_STARTUP_RELEASE_PLAN.md)
includes the source-reviewed local case with governance vault0 genuinely absent:
11 entries, 13 full maps and 46 ordered CPIs pass without allocating or funding
that vault. All 16 PIV states and cashflows match; initializer usage is 992,404 CU
and 3,832,942 loaded bytes. It remains an unsigned Bank 4.2 fixture proof.
The plan separates pending candidate/bootstrap-trust and qualified M4 acceptance
choices from fresh technical preflight, spending and signing/confirmation controls.
It neither selects those choices nor triggers deployment.

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

1. Consolidate the recorded identities/ALT with the exact artifact/bootstrap trust,
   current version/state, recipient provisioning and lifetime/resume transport package.
2. Bind current fees, funding/resources and actual sequential Squads indices;
   freeze the real Clock anchor before storing the initializer and voting.
3. Review the complete package and resolve or explicitly retain each readiness
   condition, including the open M4 reservation.
4. Record the exact independently reviewed readiness conclusion and any genuinely
   missing scope/risk decision before a live trigger under conditional D-048.

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
