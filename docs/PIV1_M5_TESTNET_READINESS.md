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
| Wire format and signatures | Local 1,544-byte legacy packet already includes all 64-byte zero signature placeholders. AB initialization used 1,067,685 CU. [Local construction/measurement](../validation/genesis-bank-runtime/tests/production_lifecycle.rs#L94). | Offline assembly must use actual 315-byte `PIV1IN01`, complete signatures, current metas and selected cluster-supported format. Evaluate v1 compatibility without upgrading pinned dependencies, or outer v0 ALT. Historical 313-byte `PIV1GM01` transport is not current-native-ABI proof. | Yes; current legacy packet cannot be submitted as-is. |
| Complete governance composition/resources | PIV checks direct stack-height-two Squads execution, fresh current 4/6 proposal, timelock, one exact PIV instruction and no stored lookups/ephemeral signers. [Verifier, lines 354–485](../programs/piv1/src/squads_execution.rs#L354). Outer v0 ALT can compress wire keys without stored Squads lookups. [Historical transport](TASK_2_27_RECIPIENT_CHECKED_GENESIS_TRANSPORT.md). | Assemble actual upload/create/approve/execute path; verify complete CU/heap/loaded-account/instruction limits including real governance overhead and the observed 730,009-byte validator list, larger than local fixtures. Local requests were 1,400,000-minus-nonce CU and 262,144-byte VM heap, with unchanged production 32-KiB allocator. No full public composition was measured. | Yes. |
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
