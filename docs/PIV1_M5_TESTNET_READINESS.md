# M5 Testnet preparation and readiness register

Status on **2026-10-18 UTC**: **PREPARATION AUTHORIZED (D-039) / PUBLIC PACKAGE NOT READY / DEPLOYMENT NOT AUTHORIZED**.
M4 remains **TECHNICALLY INCOMPLETE / ONE RECORDED EVIDENCE CRITERION OPEN**.
The internal-callee-failure observation is unwaived; the rejected operation and
its drafts remain stopped. The [limited-deliverable proposal](PIV1_M4_VALIDATION_SCOPE_REVIEW.md)
is still pending, not accepted by this authorization.

This is the single finite M5 register under [D-039](PIV1_DECISIONS.md#d-039--confirmed--m5-preparation-with-an-explicit-m4-reservation).
Baseline HEAD/main/integration was `a524cc616624f41aa54141b3ac0838edd6c0f48b`,
clean as jerem on WeatherTrader2; protected Task 2.3 remains
`3677fee97e3617ee65e2828d222008ba0952bb3e`. New reviewed work publishes only to
`integration/piv1-testnet`; D-038 grants no new main-publication authority.

## Current native initializer transport checkpoint

Continuation baseline: clean integration `21cd68e5c02def1a9620320e126e6a3dcd7fa013`;
main stays `a524cc6`, protected Task 2.3 `3677fee`. This task adds
[the native tool](../validation/genesis-transport/production-initializer.cjs) and
[ten regression groups](../validation/genesis-transport/production-initializer.test.cjs)
without modifying the historical tool/tests, Rust production, dependencies or ELFs.

Root ran these commands once, with pinned Node 24.19.0/web3.js 1.98.4, explicit
`PATH=/usr/bin:/bin`, `LC_ALL=C`, `TZ=UTC`, `NO_DNA=1`:

```sh
/home/jerem/.local/piv1-toolchains/node-v24.19.0-linux-x64/bin/node --test validation/genesis-transport/production-initializer.test.cjs
/home/jerem/.local/piv1-toolchains/node-v24.19.0-linux-x64/bin/node validation/genesis-transport/production-initializer.cjs
```

The first execution passed **10/10**, no failures/skips/stderr, in 18.628 seconds;
the first CLI passed **16 cases** in 2.925 seconds. Tests use independent literal
315-byte IN01/account/privilege oracles, all 720 codec permutations, signed-i64
and every-u8 decoding boundaries, selected reward-PDA/permutation/recipient
cases, strict truncation/alias/privilege/buffer commitment negatives and adjacent
ALT/packet limits. Unknown or operational CLI arguments reject. No new PBT
framework or dependency was added. Separate review corrected one draft size
oracle before execution: the stored native payload gains two bytes, while the
outer execute instruction retains its eight-byte selector. There was no failed
execution. Historical default eight-case JSON still matches its exact golden;
the historical fifteen-test suite was not rerun because two recipient-source-pin
cases bind an older codec. Those checks remain unchanged retained evidence,
not disabled current-native gates.

| Actual encoded packet/payload | Distinct manager/referrer | Shared manager/referrer |
| --- | ---: | ---: |
| Stored compact native message | 1,480 | 1,447 |
| Direct legacy create, calculated only; SDK refuses serialization | 1,862 | 1,829 |
| Buffer create / extend / create from buffer | 1,215 / 1,025 / 421 | 1,215 / 992 / 421 |
| Outer legacy execute, no prefix / illustrative prefix | 1,400 / 1,448 | 1,367 / 1,415 |
| Outer v0 execute, no prefix / illustrative prefix | 940 / 988 | 907 / 955 |
| Active proposal create / each of four approval plans | 383 / 341 | 383 / 341 |

All feasible packets are actual SDK serialized/decompiled bytes, including two
64-byte zero signature slots. Fee and inner rent payer are the same synthetic
key 222, distinct from executor/guardian 91. AB used a separate fee payer 242
and three signatures: do not label these packets as the same transport or add
signature overhead again. Governance vault 7 is fixed; selected recipient vault
witnesses and permutation-derived rewards match native source. The stored Squads
message has no lookups/ephemeral signers; only the synthetic, uncreated outer
ALT holds the sixteen writable targets. The 1,400,000-CU / 262,144-byte heap
prefix is illustrative sizing, not a selected or measured full-governance budget.

The schema plans for buffer upload, proposal create (`draft=false`), four distinct
approvals (`memo=None`) and execute bind pinned Squads source/IDL, not deployed
Squads execution. They do not prove permissions, actual 4/6 approval, timelock,
proposal consumption/replay, real signatures, rent/refund/funding or ALT creation,
authority, warm-up and lookup lifecycle. Current wire feasibility is established
only for this synthetic host profile; the public package is still NOT READY.

Bounded official-source research obtained no source/build attestation for the
three observed Testnet pool/Token/Squads images. Official
[verified-build guidance](https://solana.com/docs/programs/verified-builds)
warns versions can differ by cluster and scopes remote verification to Mainnet;
[Squads' deployment list](https://github.com/Squads-Protocol/v4/blob/main/README.md)
names Mainnet/Devnet. Neither fact proves Testnet unsupported. Three public
build-status endpoints were inaccessible through the browser; no metadata or
attestation-absence result was obtained. Existing artifact compatibility remains
OPEN, with no demonstrated new ABI defect. A reproducible source-equality proposal
is one way to establish provenance, not a newly imposed full-build gate before
permitted functional compatibility preparation.

New evidence: `/tmp/piv1-m5-native-transport-20261018-a` contains baseline and
toolchain hashes, writer/static review freezes, exact commands/stdout/stderr,
execution receipts, source-bound report and bounded provenance research.
No new chain RPC, Rust/SBF build, application runtime, simulation, signing,
key creation, installation, deployment, fund movement or authority transfer
occurred. Prior 13 RPC observations and 619+1/eight host gates, caller 15,
runner 66 and AB remain retained within their original scopes. Routine agents
were requested as Sol/High and critical review as Sol/Extra High; root runtime
model remains unexposed. No configuration was changed. Main publication is not
authorized for this new work. The M4 criterion remains OPEN/unwaived and the
stopped operation/drafts were not inspected, retried, executed or rerouted.

The observed-environment section below is the retained initial read-only
assessment, not a fresh network observation or an assertion that this new task
ran no host tests.

## Observed public environment

Thirteen read-only RPC receipts from `https://api.testnet.solana.com` and local
decoding are retained in `/tmp/piv1-m5-readiness-20261018-a`. The observed genesis
is `4uhcVJyU9pJkvQyS88uRDiswHXSCkY3zQawwpjk2NsNY`, matching the official
[Solana genesis reference](https://github.com/solana-labs/solana/blob/master/sdk/src/genesis_config.rs).
RPC reported `4.4.0-beta.1`, epoch 1063; contextual reads span slots
454,072,540–454,074,464. These are separate observations, not a coherent single-slot
snapshot or an invocation of production account authentication.

The official [Jito deployment list](https://www.jito.network/docs/jitosol/jitosol-liquid-staking/security/deployed-programs/)
and observed accounts agree on the fixed program/pool/mint below. The direct
document fetch returned HTTP 403; the browser read succeeded and is retained.
The official [Solana transaction upgrade page](https://solana.com/upgrades/larger-transaction-sizes)
reports v1 active on Testnet with 4,096-byte transactions; legacy/v0 remain 1,232
bytes. The observed Feature-owned `txv1aq4pp281K9um3tnPgkfX8UqtFT6wcVW3hNezGLL`
records activation slot 437,276,256. This does not prove this project's client,
wallet, Squads composition or signatures support v1.

The retained PIV ELF has `e_flags=0` (SBPFv0). The official
[SBPFv3 deployment upgrade page](https://solana.com/es/upgrades/sbpfv3-programs)
identifies SIMD-0500 gate `B8JJXCy5amZyWG9r7EnUYLwzXSXTxG7GZ1qZ1qggo83g`;
its observed account was absent at slot 454,074,464. No activation was observed
for that identified gate. General deployment-format compatibility is not proved;
recheck relevant features before exact artifact selection. No toolchain migration
is authorized by this observation.

## Finite readiness matrix

| Requirement | Existing evidence | Missing proof and next permitted action | Blocks public testing? |
| --- | --- | --- | --- |
| M4 reservation | AB validates the supported current-artifact delayed lifecycle; its separate internal-callee-failure criterion remains unsatisfied. [Closure review](PIV1_M4_CLOSURE_STATUS.md). | Separate readiness review must assess the reservation and identify required evidence or a separately reviewed founder scope/risk decision before live approval. Do not retry, rephrase, reroute or execute rejected drafts. | Yes; preparation may proceed, release clearance is absent. |
| Official cluster and Jito topology | Fixed IDs exist on verified Testnet. Pool update epoch 1063; nine-decimal mint supply equals pool supply; default lockup; reserve/manager identities and derived withdraw PDA decoded. [Identity guards, lines 109–163](../programs/piv1/src/integrations/jito_identity.rs#L109). | Refresh coherent dependencies before any future exact package. Fixed identities and compatible serialized prefixes do not attest deployed binaries. | Initial identity prerequisite observed; freshness and compatibility remain open. |
| Deployed external artifacts | Observed Stake image exactly matches local pinned Stake 5.1.0. Pool and Token images differ from retained local ELFs, including comparison allowing local-prefix/zero-padding equality. | Establish bounded deployed provenance and compatibility for the supported calls; Squads deployed equivalence also remains open. A binary difference is an evidence gap, not a demonstrated ABI defect. No automatic dependency/toolchain change. | Yes, until compatibility is independently supported. |
| Deposit and withdrawal regimes | Observed SOL deposit fee 0/0, no SOL deposit authority; stake withdrawal fee 1/1000, no pending change or preferred withdrawal vote. Production requires permissionless zero-fee SOL deposits and protected SPL variants 25/24. [Deposit, lines 35–47](../programs/piv1/src/integrations/jito_deposit.rs#L35), [withdrawal, lines 19–34](../programs/piv1/src/integrations/jito_withdrawal.rs#L19). | Match deployed behavior, current epoch and actual quote/postconditions. Fees/preferences/authorities may change; do not change economics to fit a fixture. | Observed fields fit the supported regime; deployed execution remains unproved. |
| Supported withdrawal source and rent | List has 864 records in 730,009 allocated bytes. Index 75 candidate matches PDA, vote, Stake owner, authorities, list balance, flags zero and nondeactivation. Runtime minimum delegation observation is 1,000,000,000 lamports. [Source guards, lines 86–133](../programs/piv1/src/integrations/jito_withdrawal_preparation.rs#L86). | Reobserve source priority, capacity, current rent/funding and dynamic token minimum at construction/execution. Only Active validator sources are supported; no transient/reserve/removal/preferred fallback. Candidate is not a permanent selected source or successful CPI quote. | Availability condition remains dynamic. |
| Wire format and signatures | Current 315-byte native tool passes ten tests and sixteen SDK cases. Synthetic outer v0 executes in 907–988 bytes with all zero-signature storage counted; stored message has no lookups. Historical/AB packet evidence keeps its own signer composition. | Real identities/signatures, ALT lifecycle and actual governance/resources remain unproved; v1 is not implemented or required by this bounded fitting v0 profile. | Host current-native wire feasibility closed in this scope; exact public transaction usability remains OPEN. |
| Complete governance composition/resources | Current native upload/create/approve/execute wire plans fit; PIV verifier and pinned source/schema are bound. Proposal/approval plans are serialized/decompiled, not executed. | Establish real deployed Squads compatibility, actual lifecycle/consumption and complete CU/heap/loaded-account/instruction limits including the 730,009-byte validator list. Illustrative resource prefixes do not establish those budgets. | Yes. |
| Real governance consumption and recipients | Squads Program/ProgramData exist. Local caller leaves proposals unconsumed; production only reads Approved state during CPI. Pinned Squads source persists Executed after successful outer completion. [Verifier limitation, lines 13–17](../programs/piv1/src/squads_execution.rs#L13). | Identify actual multisig, six guardians, proposal lifecycle, wallet control and supported funded recipient vaults; prove replay/consumption behavior for the real path. Recipient identity does not establish exclusive 4/6 spending. | Yes; no real selected multisig/proposal lifecycle is established. |
| Guardian updates, recovery and founder workflow | AA2 covers local one-seat synchronization/old earned claims; Z covers frozen/current recipients. Authority is current ProgramData, not historical multisig continuity. Governed recovery exit is unimplemented. | Preserve one-seat/mismatch limitations; prepare non-Rust inspect/contribute/operate/readiness/claim/error workflow and exact stop conditions. Natural yield, the ten-day cadence and dynamic minimum delegation govern attainable delayed tests; no fabricated appreciation or shortened Clock. Public cooldown is not promised to equal local epoch 14. RecoveryRequired means stop and report, not an invented exit. | Exact usable package/workflow remains open. |
| Exact package identities and budget | No new PIV Program ID, founder public identities, control evidence or funding workflow is established. Historical spike identities are not selected production identities. | Keep Program ID/artifact provenance, authorities, recipients, public signer identities, rent/fee/funding envelope and command sequence OPEN. Continue independent preparation; request only minimal identity authorization when it genuinely blocks further work. Never request or read private keys. | Yes; package is not deployment-ready. |

## Observed identities, amounts and artifact comparison

The fixed production program is `SPoo1Ku8WFXoNDMHPsrGSTSG1Y47rzgn41SLUNakuHy`,
pool `Jito4APyf642JPZPx3hGc6WWJ8zPKtRbRs4P815Awbb`, mint
`J1toso1uCk3RLmjorhTtrVwY9HJ7X8V9yYac6Y7kGCPn`; the fixed Squads ID is
`SQDS4ep65T869zMMBKyuUq6aD6EgTu8psMjkvj52pCf`.
Observed pool links: validator list `G5N6K3qW86GSkNEpywcbJk42LjEZoshzECFg1LNVjSLa`,
reserve `CzKqc9cs4XpyG6y4peQgk3vBjPyqhktmfqaMuMBCXCqm`, manager fee
`8yoigZfzZ1nNaadumY9uPVD118225UYHTDpmjpr2nrSa`, withdraw PDA
`6iQKfEyhr3bZMotVkW6beNZz5CPAkiwvgV2CTje9pVSS`.

Candidate `8jTAUTjdd4nJ5HdnSNEhcXBSaFapy7eE9hLYG6oqtkrP` at list index 75 had
balance 280,250,066,236 and delegated stake 280,248,399,996 lamports. Its computed
native capacity upper bound was 279,247,783,356, not a production quote. Current
observed rent floors were 1,666,240 for 200-byte Stake and 1,986,280 for the
263-byte leg metadata: 3,652,520 operational lamports per new leg before other
costs. Embedded historical Stake rent metadata is separately 2,282,880 and must
not be replaced with today's rent. Final package funding/fees remain OPEN.

`artifact-comparison.json` retains complete hashes. Observed deployed allocated
ELF-image SHA-256 values are pool
`6c0cb7ef5da80a23c3ad85d7fec566f3cc17b815d428daec84bb916441a2a143`,
Token `8190d3f7ceb6cb7a7a8d8924bff89f9f611e15ce1f806f2b6237f3311a98f697`,
and Stake `3d2d39c596ce8be2d47816b4ee5db9fc759d80fde54b08c930ad0b6daed64c2c`.
The last equals local Stake; the others do not equal local pool/Token. Squads
ProgramData image hash is retained in `dependencies-decoded.json`; it has not
been established as the pinned source's deployed build.

The retained current PIV candidate is 894,552 bytes, SHA-256
`e29735e1467da98ecddb1eea9fd25abaffa4a450bd899ddc137f7d8c0da35b99`;
`retained-piv-artifacts.json` records a hash recheck, not a rebuild or deployment
selection. Exact build provenance/reproducibility and package approval remain
OPEN. The separately retained synthetic caller is local evidence infrastructure,
not production Squads or an approved deployment artifact. The exploratory
391-byte rent receipt is not WithdrawalLeg rent; the 894,597-byte rent receipt is
only illustrative PIV-plus-ProgramData-header sizing, not a full deployment budget.

## Evidence and bounded continuation

New operations were public documentation reads, thirteen read-only RPC requests,
Python data/binary decoding and an existing pinned Node 24.19.0/web3.js 1.98.4
PDA cross-check. No new tests, builds, runtime transactions, simulation, signatures,
wallet/key operations, installation, deployment, funds movement or authority
transfer occurred. Prior 619 production host tests +1 doctest/eight gates/strict
SBF, caller 15 tests/check/docs/SBF, runner 66 tests and AB remain retained evidence
within their individual source/artifact scopes, not fresh executions.

Evidence includes `baseline.json`, `founder-directive.md`, the thirteen raw RPC
receipts, `pool-decoded.json`, `dependencies-decoded.json`, `candidate-decoded.json`,
`identity-details.json`, `artifact-comparison.json`, `sdk-pda-crosscheck.json`,
`public-sources.json`, `web-official-references.txt`, `model-observation.json`,
`source-inspection.json` (eleven unchanged inputs), `observation-manifest.json`,
`retained-piv-artifacts.json` and `sbpf-deployment-decoded.json`.
One evidence-index draft attempted to parse textual web output as JSON; correcting
file selection caused no RPC/runtime rerun. No Solana MCP was available and none
was installed; official web/read-only RPC was the permitted fallback.
`HTFP_MASTER_CONTEXT.md` was unavailable in the project and `/home/jerem`; other
chat contents were unavailable. No claim is made that either was read.

Root's actual session model is not exposed. Saved global configuration reports
`gpt-6-astra`/High; routine writer/source agent requests explicitly selected
`gpt-6.1-sol`/High, and the separate final readiness reviewer was explicitly
requested as `gpt-6.1-sol`/Extra High. No observed custom-agent override or global configuration edit
is recorded. This is an observation, not proof of root model identity.

Next bounded work is deployed-binary provenance/compatibility and offline
current-native-ABI transaction/governance assembly, followed by separate review.
Do not turn the retained M4 reservation into another automatic validation loop.
A reviewed exact package, its residual-risk assessment and separate explicit
founder authorization remain required before any live Testnet deployment.
