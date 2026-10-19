# Founder Testnet workflow — preparation draft

**PREPARATION ONLY / PUBLIC PACKAGE NOT READY / DEPLOYMENT NOT AUTHORIZED.**
The [single M5 register](PIV1_M5_TESTNET_READINESS.md) owns readiness status,
evidence and remaining decisions; this guide explains how to use that register
and [the public package draft](PIV1_TESTNET_PACKAGE_DRAFT.json). The JSON is a
package index, not a manifest accepted by any CLI. Its null fields are unselected,
not defaults. [D-039/D-040](PIV1_DECISIONS.md) leave M4 technically incomplete,
its internal-callee-failure criterion open/unwaived and rejected operation/drafts
stopped. This guide neither resumes that work nor grants live authorization.
Subsequent checkpoints use reviewed integration; new main publication requires
separate authority.

## Current creation approval and additional-transfer decision

[D-042](PIV1_DECISIONS.md) records the founder's “oui j'autorise” for the exact
619-byte creation/profile and 0.003-SOL total payer limit. That approval is retained;
no repeat approval for the same 4/6/autonomous/no-delay profile is needed.
Execution is held because create_v2 cannot atomically cap a mutable global fee
while its payer holds 1 SOL. No transaction has been signed or submitted.

The [reviewed extra-transfer proposal](PIV1_TESTNET_SQUADS_CAPPED_PAYER_PROPOSAL.json)
would empty the existing H2 payer (gross debit **1 SOL**), return
**0.99766336 Testnet SOL** to original
`4WKQg3Sm8bvHS8DBmxoiMMi4Fev4mJU6ZLwGSTg6jDna`, then execute the unchanged
Squads creation instruction **within the same atomic transaction**. Total expected
cost is **0.00233664 SOL**, within 0.003 SOL; six guardian wallets stay 1 SOL.
Both instruction state changes commit together or roll back; failed inclusion
can still charge the 0.00001-SOL network fee. Successful creation empties the payer;
later funding requires separate authorization.

This bounds originally funded own exposure, not gross debit under arbitrary
unsolicited credits. The unsigned atomic 668-byte/two-signature simulation passed;
actual balances are unchanged and multisig remains absent. Extra refund and changed
envelope require approval because the original card allowed only one creation
instruction. The 4/6 profile stays approved. Ordinary source adaptation/review
needs no implementation decision from the founder; old executor stays disabled.
The earlier separate-refund proposal/215-byte simulation is superseded and must
not run. Only the atomic 997,663,360-lamport return is now proposed. See the single
register for exact bytes, raw evidence, budget basis and failure/uncertainty limits.

M4 open/unwaived, public package NOT READY, PIV deployment NOT AUTHORIZED. The
following pre-approval card is retained historical preparation, superseded only
by the narrow D-042 approval above.

## Retained approval card — exact Testnet Squads creation before D-042

The [public creation proposal](PIV1_TESTNET_SQUADS_CREATION_PLAN.json) prepares
multisig `9mwto7RNokYc8n8MPtzHpubwwnKcborY2r9RnfZAScVS`, six existing guardian
candidates, accepted threshold 4, and proposed autonomous/no-delay configuration,
permissions 7 and no rent collector. Existing guardian 0 signs as createKey; the
funded external payer `H2AaoG8HZKfNyZzMhDGMB8n9edviyHtJGKpPL3uYpxt4` is creator
and fee payer. Creation uses these two signatures, not four proposal approvals.
No new key, refill, original 4WK source debit, PIV deployment or authority transfer.

Unsigned official-Testnet simulation succeeds and its whole simulated 330-byte
multisig is checked against source. Observed rent 0.00232664 SOL + fee 0.00001SOL =
**0.00233664 SOL**; creation fee 0. Proposed maximum total payer debit **0.003 SOL**.
Actual finalized rereads show no created multisig and all seven wallets still 1 SOL.
The absent configured treasury loads as a zero-System placeholder and receives no
transfer. This is simulation evidence, not verified signatures or committed state.

The founder must explicitly approve this one creation and proposed profile/cap
before signing/submission. D-041's initial wallet-funding authority is consumed.
A later authorized executor must refresh current image/Config/treasury/fees/rent/
wallets/absence/genesis and blockhash while preserving the exact immutable body,
roles/profile/cap; verify actual signatures, simulate, submit once and confirm by
rereading state. Stop on changed scope or uncertain submission. No automatic
creation, main publication, PIV operation or M4 waiver follows from this draft.

## Retained narrow completion — identities and wallet funding only

[D-041](PIV1_DECISIONS.md#d-041--confirmed--seven-testnet-identities-and-bounded-wallet-funding)
authorizes seven disposable Testnet-only keys and at most 1 SOL per address
from `4WKQg3Sm8bvHS8DBmxoiMMi4Fev4mJU6ZLwGSTg6jDna`, total 7 SOL plus network
fee. Root generated the keys privately once and finalized one 509-byte System-only
transaction: seven 1-SOL credits and fee 5,000 lamports. Signature/slots/balances,
public keys and receipts are in the [identity setup](PIV1_TESTNET_IDENTITY_SETUP.json)
and [single register](PIV1_M5_TESTNET_READINESS.md#current-authorized-testnet-identities-and-wallet-funding--d-041).
Earlier Stage A/no-key/no-funding statements below are historical, superseded
only for these keys and this exact capped funding. This one-shot budget is now
consumed: no refill, further spending, regeneration or overwrite is authorized.

The six guardian candidates are listed in **local creation order**, not PIV slot
assignment and not current Squads membership. The new funded payer is distinct
from the original funding source. The public inventory does not establish
independent custody or an accepted multisig profile. Multisig creation, PIV ID/
deployment, slot permutation, vault indices, initial pause and KIF anchor remain
OPEN/unapproved. The accepted six-guardian/4-of-6 requirement is unchanged;
the exact new multisig creation/configuration, including proposed timelock0/
rentCollectorNone, remains unapproved. Funding authorizes no creation/configuration
transaction, authority transfer or main publication. Next prepare the exact Squads creation packet and fresh
fee/treasury/funding/simulation evidence, then obtain its separate authorization.

## 1. Retained public setup proposal — before D-041

The founder has confirmed that no identities exist yet and wants the pilot to
prepare six dedicated disposable **Testnet-only** guardian keypairs and a 4-of-6
Squads setup. This setup intent is CONFIRMED; actual keys, addresses, control and
provisioning remain unselected. **Key generation and transactions are NOT
AUTHORIZED.** Do not ask the founder to create keys, supply private material or
make Rust/implementation choices. The required first check has now observed deployed Squads v4 alongside Jito on
official Solana Testnet, with exact allocated image hashes matching retained
tested images; the bounded result below supports further setup preparation,
not creation execution or readiness.

After a supported result, the pilot prepares an exact secure setup card covering
public roles, custody/storage, funding and phased creation for founder approval
**before any key generation or transaction**. Private keys must never enter Git,
logs or chat. If Squads is unavailable or incompatible, report supported options
and STOP for founder decision; do not switch to Devnet or replace the protocol.
The founder may initially control all six wallets; this is operational separation,
not decentralized independent custody.
The pilot's concrete setup card must name initial pause, KIF anchor timestamp, PIV
slot assignment and the three vault indices (governance, HTFP, Team). Ordinary
technical mappings are prepared by the pilot for review; the founder is not asked
to design them. Unapproved selections remain null, with no silent pause/anchor/
permutation/index defaults. The initializer
sorts the six declared guardians by raw public-key bytes, then applies the explicit
six-element permutation to obtain PIV slots. Threshold remains the accepted 4/6.
Creator, executor and four distinct approvers must be selected from current
members, with their actual permissions/control independently verified.

The initializer's rent payer must differ from every guardian, including its
member creator/executor/approvers, and from protected PIV/protocol/recipient roles.
The fee payer may be a guardian and may equal the rent payer where the supported
compiled union permits. ALT authority is explicit. A separate refund recipient
choice is unavailable: closing the upload buffer refunds its actual balance to
the creator, not automatically the rent payer. The rent payer's own System-owned,
empty signed funding source, current balance/shortfall and required residual need
independent proof; a Squads vault signature is not substituted for that source.

The 16 PIV targets (Config, round, registry, six rewards and seven custody accounts)
are distinct canonical addresses. Governance/HTFP/Team vaults are three distinct
PDAs of the actual multisig and must also avoid protected-role aliases. Recipient
accounts must meet the existing funded System/empty/nonexecutable prerequisites.
The program identity, public actor control and all derived addresses remain OPEN.
Do not create or provision anything from this intake.

## 2. Source, artifact and public facts

The retained PIV candidate is **894,552 bytes**, SHA-256
`e29735e1467da98ecddb1eea9fd25abaffa4a450bd899ddc137f7d8c0da35b99`, at
`/tmp/piv1-bank-smoke-guardians-sbf-20261018-b/artifacts/piv1.so`.
Root's local binding receipt is
`/tmp/piv1-m5-founder-package-20261019-a/artifact-binding.json`, SHA-256
`8b7883bb122e9156cc5006191c97f972870a9fb323227995088642e313184f8c`.
It binds retained PASS/profile/build evidence and 132 production/host inputs that
match tracked checkpoint `6c44989b6a77a52cc6562acabe9b3a06858f95a5` and the worktree.
Source-set fingerprint: `20b6707947eed5d3ab19fcb4e444065bcf1e7728dac883ffe503746793cb6f4a`.
The retained profile baseline `560ca09c9c17becb79564c164e8c308b196c7cbe` is context
only, not proof of a clean compilation commit; the ELF is not asserted to have
been built from the current checkpoint. Read the receipt's exact scope and limitations before selecting an artifact.
This is a retained candidate, not final deployment selection, a new build,
reproducibility proof or professional audit. The synthetic caller is excluded
from the deployment package. Public external images retain the compatibility and
provenance limits already recorded; no new blanket provenance gate is added.

The known endpoint is `https://api.testnet.solana.com`, expected genesis
`4uhcVJyU9pJkvQyS88uRDiswHXSCkY3zQawwpjk2NsNY`. Fixed integration IDs and retained
historical observations appear separately in the JSON. They are not selected PIV
or founder identities and do not prove today's chain state. Refresh cluster/genesis,
actual program/loader authority, pool/list/reserve/mint/manager/referrer, epoch,
fees, source availability, rent, balances and public governance state before any
future live decision. Record source/time/slot/commitment and raw evidence; do not
present mixed-slot observations as a coherent snapshot.

The existing clients **do not acquire live snapshots, provision accounts/ALT/
multisig, sign, submit, confirm or resume live transactions**. A usable reviewed
operator transport/provisioning workflow remains OPEN. This document does not
supply it. There is also no existing pause/unpause, guardian synchronization/
rotation or recipient-update client composition. Those governed operator paths
remain OPEN even though their supported production boundaries have retained
evidence. If `initiallyPaused=true` is selected, economic steps require a separately
reviewed and authorized governed unpause workflow first; none is supplied here.
Only a separately authorized future operator could carry out the live stages
below after the exact package and required compositions pass readiness review.
One explicit exact-package authorization may cover its bounded phases. Every phase
still needs fresh context and coverage by that authorization; changed identities,
artifact, budget or authority envelope require revised approval. This draft is
unapproved.

The fresh official-Testnet check made **11 read-only RPC requests** on 2026-10-19,
bracketed from slot454140091 to454140546, genesis unchanged. Receipt:
`/tmp/piv1-m5-founder-package-20261019-a/testnet-current/observation-manifest.json`,
SHA-256 `040580018e3447d18ef2d3333746b7f60cac80a9010a6e06f5fd013b868bc689`. Squads is executable with canonical loader
ProgramData; its allocated image is980096 bytes, SHA-256
`95d0811d25f9df7d21f230e34ce93fcb37d22a9b4fe1365f07cd339e9ad165f1`.
Squads, pool, Token and Stake match retained images used in their bounded local
compatibility profiles. Pool update epoch1064 matches the observed epoch, with
permissionless zero-fee SOL deposit and stake withdrawal fee1/1000. These separate
observations are not one coherent bank snapshot, source equivalence, a new runtime
test or full public-path proof.

Squads ProgramConfig `BSTq9w3kZwNwpBXJEvTZz2G9ZTNyKBvoSeXMvwb4cNZr` is144 bytes,
canonical bump255, with observed creation fee0 and treasury
`69MLgFq538VoruFrD7geDRo5S1hD2stG9hU9cd32LLf5`. **That treasury account query
returned null: it is not observed existing/funded.** The pinned creation source
skips its transfer when fee0; this observation does not prove actual creation
composition succeeds. Exact composition and simulation remain pending before
Stage B. Do not invent a treasury provisioning/funding transaction. Fresh finalized
rent for a330-byte six-member multisig was2,326,640 lamports; it is an observation,
not the full payer requirement or a Stage B budget.

## Retained secure setup card — Stage A complete, Stage B unapproved

**Historical proposal before D-041:** seven-key generation and capped wallet
funding are now completed under the later explicit authorization. Squads creation
Stage B and the proposed governance settings remain unapproved. The pilot's
proposed technical profile is six new dedicated Testnet guardians plus a seventh
external payer. Guardian0 also signs as Squads createKey; the external payer is
Squads creator/fee payer and can later be PIV rent payer, distinct from all six
guardians. No separate create-key keypair is needed. Proposed members each receive
permission mask7 (Initiate/Vote/Execute), threshold4, autonomous
`configAuthority=None`, Testnet timelock0, `rentCollector=None` and `memo=None`.
Timelock0 is an explicit proposal, not a selected default; no actual multisig
exists. None rent collector disables transaction rent reclamation in this profile.
Holding all six keys on one VPS is an operational fixture, not independent custody.

Proposed **future, unexecuted** generation method: pinned Node24.19.0
`node:crypto.randomBytes(32)` independently per key, then pinned
`@solana/web3.js`1.98.4 `Keypair.fromSeed`. No user-chosen, static test or reused
seed is accepted, and no seed/secret bytes are logged. Keep derivation in memory;
save each 64-byte secret-key array only to its exclusive0600 private file under
umask077 and the0700 directory. Refuse symlinks and existing files; resolve/verify
the directory outside Git before writing. Only public keys and public metadata
may enter the review manifest. No generation script has been added or executed.

The source-bound creation interface is `multisigCreateV2` in pinned Squads commit
`64af7330413d5c85cbbccfd8c27a05d45b6e666f`. Its six ordered roles are ProgramConfig
(readonly), treasury (writable), new multisig (writable), createKey (readonly signer),
creator (writable signer), System (readonly). Guardian0's signature protects the
create-key seed against front-running; it need not differ from a member. The
multisig PDA uses `["multisig","multisig",createKey]`; six members allocate330 bytes.
Creator funds that account's fresh rent and the actual ProgramConfig creation fee
to ProgramConfig's exact treasury. These costs are separate from the historical
initializer rent table. This describes pinned source/IDL mechanics; no existing
client composes this creation transaction, and prior seeded-multisig runtime
evidence did not execute creation. Exact creation transport/budget stays OPEN.

| Gate | Concrete proposed work and stopping point |
| --- | --- |
| Stage A — separate authorization for seven keypairs only | After the bounded official-Testnet Squads/Jito evidence and Stage A approval, generate six guardian keys plus one external payer privately. Proposed directory `/home/jerem/.local/share/piv1/testnet-identities/setup-20261019-a`, owner `jerem`, mode0700; `guardian-0.json` through `guardian-5.json` and `payer.json`, mode0600. Refuse existing paths/files; no overwrites, private bytes on stdout, shell tracing, Git/log/chat exposure or global wallet configuration. No unencrypted backup or shared-sync copy; no backup is created now. Loss before funding abandons that public draft without overwriting files; loss after funding means stop/report, with no automatic regeneration. No directory or key is created now. Publish only derived public keys/addresses and custody metadata for review, then STOP before transactions. |
| Stage B — separate bounded creation authorization | Use Stage A's actual public keys, refreshed ProgramConfig/treasury/creation fee, fresh330-byte rent, exact composed and simulated creation transaction and per-actor fee/funding cap. Resolve the observed absent treasury under the zero-fee source branch without inventing provisioning; creation success is unproved. Review the six sorted members/permissions, threshold4/autonomous configuration, explicit proposed timelock/rent collector, createKey/creator signer union and expected PDA. Authorize only that reviewed Testnet creation transaction; confirm success and reread actual multisig. No transaction is authorized now, and funding/provisioning sources remain OPEN. |
| Later PIV package | Program key, deployment, authority transfer, recipient funding, ALT/initializer, contributions and other fund movements need their own exact reviewed scope. Neither Stage A nor the proposed Stage B implicitly authorizes them. |

If a later preflight finds official Testnet Squads unavailable or incompatible, report
options and STOP for founder decision. No Devnet switch, protocol substitution,
key generation or transaction is a fallback action.

## 3. Existing offline commands

Run from the repository with the pinned Node path. These examples inspect null
samples; they do not execute the operational stages. No command below was rerun
for this documentation task.

```sh
/home/jerem/.local/piv1-toolchains/node-v24.19.0-linux-x64/bin/node --max-old-space-size=512 clients/cli/initializer-plan.cjs --manifest clients/cli/initializer-plan.example.json
/home/jerem/.local/piv1-toolchains/node-v24.19.0-linux-x64/bin/node --max-old-space-size=512 clients/cli/initializer-funding.cjs --manifest clients/cli/initializer-plan.example.json --observations clients/cli/initializer-funding.example.json
/home/jerem/.local/piv1-toolchains/node-v24.19.0-linux-x64/bin/node --max-old-space-size=512 clients/cli/state-inspect.cjs --snapshot clients/cli/state-inspect.example.json
/home/jerem/.local/piv1-toolchains/node-v24.19.0-linux-x64/bin/node --max-old-space-size=512 clients/cli/guardian-inspect.cjs --snapshot clients/cli/guardian-inspect.example.json
/home/jerem/.local/piv1-toolchains/node-v24.19.0-linux-x64/bin/node --max-old-space-size=512 clients/cli/intake-plan.cjs --manifest clients/cli/intake-plan.example.json
/home/jerem/.local/piv1-toolchains/node-v24.19.0-linux-x64/bin/node --max-old-space-size=512 clients/cli/principal-stake-plan.cjs --manifest clients/cli/principal-stake-plan.example.json
/home/jerem/.local/piv1-toolchains/node-v24.19.0-linux-x64/bin/node --max-old-space-size=512 clients/cli/distribution-plan.cjs --manifest clients/cli/distribution-plan.example.json
/home/jerem/.local/piv1-toolchains/node-v24.19.0-linux-x64/bin/node --max-old-space-size=512 clients/cli/withdrawal-plan.cjs --manifest clients/cli/withdrawal-plan.example.json
/home/jerem/.local/piv1-toolchains/node-v24.19.0-linux-x64/bin/node --max-old-space-size=512 clients/cli/guardian-kif-plan.cjs --manifest clients/cli/guardian-kif-plan.example.json
```

All null planner/inspector samples give `INCOMPLETE` with `plan:null` or
`inspection:null`. The funding example is the exception: its complete historical
rent observations give `CONDITIONAL_RENT_ONLY` despite incomplete identities.
Valid complete planner inputs give only `OFFLINE_PLAN_ONLY`; supplied snapshot
inspectors give `OFFLINE_STATE_INSPECTION` or `OFFLINE_GUARDIAN_INSPECTION`.
Malformed/unknown/duplicate/unused known fields or unsupported flags reject.
`--template` is available on each tool and prints its own input template.
Use separate public files based on those schemas; do not pass the package index
as a manifest. Whole packets include all-zero signature slots within 1,232 bytes;
requests and encoded blockhash/context do not prove actual resource sufficiency,
wallet control or a send-ready transaction. Refresh each future signed step.

## 4. Future PIV/Squads sequence — wallet funding is separate

| Stage / existing offline input | Required future observation and expected committed result | Stop condition |
| --- | --- | --- |
| Provision exact program/governance/recipients; initializer manifest | Actual actors/control, deployment/loader state, six-member multisig permissions/4-of-6/timelock, distinct funded recipients and own rent payer source; exact artifact selected. Provisioning/deployment transport is OPEN and needs future authorization. | Missing identity/control/funding, artifact mismatch or authority uncertainty. |
| Initializer ALT and Squads phases | Confirm ALT creation, extension with all 16 PIV targets and actual authority/content; warm-up slot strictly after final extension. Then upload adaptive chunks, create stored transaction from buffer and create the proposal with the existing active profile (`isDraft=false`); there is no separate activation packet in this client. Obtain four actual member approvals, respect timelock and execute exact stored message. Each phase refreshes blockhash/current indices and confirms its own result. These are unsigned drafts only, all NOT RUN. | Stale/consumed index, ALT missing/not warm/incorrect, incomplete approvals, changed message/authority, resources/funding insufficient. |
| Inspect initialized state (`state-inspect`, `guardian-inspect` current-set) | Confirm proposal `Executed`, actual initialized Config/Idle round, 16 targets and current six slot tuples against exact manifest. Initializer has no emitted `PivInitialized` success event; that name is only a marker. Inspectors only decode supplied bytes and bounded references. | Any mismatch, unexplained custody surplus/deficit, unknown pause/current governance. |
| Contribute (`intake-plan`: `contribute-sol` / `contribute-jitosol`) | Explicit positive caller amount; SOL donor or legacy JitoSOL owner signs with valid source. Committed `SolContribution` / `JitoSolContribution` increments pending only; no depositor ownership, withdrawal or reward right. Intake is not Idle-only but requires unpaused valid custody. | Paused, wrong mint/owner/source/allowance, deficient prior pending backing or arithmetic failure. |
| Initial bootstrap (`bootstrap-initial`) | Unpaused initial Idle/zero-history, normalized recognized pending and current pool facts. Production derives initial holdings/value; `InitialContributionsBootstrapped` records initial principal/HWM. No caller amount, pool/Stake CPI or staking. | Not the initial profile, no recognized pending, stale pool/custody or failed guards. |
| Protected principal conversion (`principal-stake-plan`) | Explicit historical SOL amount and caller minimum; production protected pool variant25, zero supported SOL deposit fee, exact no-loss historical value/HWM. `PendingSolStaked` is the actual event name despite historical-principal scope. Pending is not staked. | Even one-lamport historical loss, HWM failure, fees/authority/profile unsupported, minimum not met. |
| Prepare (`distribution-plan`: `prepare-liquid` / `prepare-withdrawal`) | Idle, unpaused valid custody/current pool, ten days from the last successful preparation, positive actual yield. CURRENT guardian revision/slots are snapshotted; recipients/eligibility then freeze in the round. `DistributionPrepared` opens `EscrowFunded` liquid or `WithdrawalActive` delayed path. Withdrawal preparation creates no leg/burn/deactivation. | No invented public yield/Clock: valid liquid no-yield has no snapshot/event/clock reset; withdrawal no-yield rejects `ZeroTarget`. Liquid positive shortfall rejects6145; it is not valid-insufficiency evidence. |
| Valid technical insufficiency (withdrawal preparation only) | Authenticated `DistributionPreparationInsufficient` updates only its retry timestamp, no round. Separate 24-hour retry clock, not a restart of the ten-day cadence. | Malformed/failed attempts do not set that clock; do not claim success, force a smaller leg or weaken source checks. |
| Delayed legs (`withdrawal-plan`: initiate then finalize as ready) | Declare active sequence/current next-leg index and validator source/index; production chooses maximum-safe fill, protected variant24, immediate deactivation and `WithdrawalLegInitiated`. Multiple sources may be needed. Refresh after each success. Actual runtime Stake inactivity, not a date or local epoch14 recipe, governs finalization. `WithdrawalLegFinalized` closes metadata/updates totals. | Source preference/order/capacity or technical floor fails, missing operational rent, stale index, real Stake not inactive; no smaller arbitrary amount or cooldown promise. |
| Delayed rent/reward/loss accounting | OperationalSOL advances full current temporary rents. Prefunds normalize to pending; original recorded Stake/metadata rents return to operations; metadata excess goes pending. Cooldown reward is next-cycle yield, excluded from fixed round. Settlement waits for exact target assignment and every successful leg finalized. | Ambiguous Stake surplus rejects. Observed cooldown loss/residual-HWM failure enters `RecoveryRequired` without HWM reset: stop/report; no governed recovery exit is supplied. |
| Settle (`distribution-plan`: `settle`) | `EscrowFunded`, exact escrow/all legs/target and HWM proof. Use FROZEN guardian revision/slots and HTFP/Team recipients, which may differ from current Config. `DistributionSettled` commits net beneficiary payments, KIF earned liability/carry and protected HWM. | `DistributionSettlementRecovery` is committed no-payment `RecoveryRequired`, not settlement. Stop/report; never pay a partial multi-leg distribution. |
| Integrate (`integrate-pending`) | `Settled`; `PendingContributionsIntegrated` includes full contribution value in HWM, transfers remaining physical pending once and completes Idle. Pending SOL already used for outgoing payments keeps its full contribution-value treatment. No staking here. | Wrong lifecycle/normalization, repeated integration, custody/value/HWM mismatch. |
| Heartbeat (`guardian-kif-plan`: `guardian-heartbeat`) | CURRENT registry/six rewards, runtime Clock and current loader/Squads correspondence; direct guardian signature. `GuardianHeartbeat` records current-period activity only; no new liability, four approvals or Squads CPI. This path has no pause check. | Wrong current tuple/authority/signature or time regression. Post-snapshot activity is not retroactive. |
| Earned claim (guardian inspector earned-record; `claim-kif`) | Independent EARNED guardian/revision/slot, positive amount and refreshed stored `cumulative_claimed` source counter. `KifClaimed` pays fixed guardian signer from isolated KIF vault; pause allowed, old earned rights survive rotation/inactivity. | Overclaim/global liability/backing/rent/replay/arithmetic failure or wrong fixed destination. Stored claimable/counter alone proves no available payment. |

For every future result require whole-transaction success, confirmations and fresh
account rereads; logs alone can remain after a later transaction failure. Failed
operations must propagate errors for rollback. Preserve raw receipts and all
attempts. Economic pause gates remain intact, with the recorded isolated claim
exception and heartbeat's actual separate source behavior. Accepted split remains
59% / 19.5% / 19.5% / 2%; there is no new payout policy.

Direct heartbeat/claim plans support ordinary on-curve signer declarations, with
one zero signature if payer=guardian or two if distinct. Production can accept
runtime PDA signing through CPI within its recorded profile, including a
System-owned empty PDA claim destination; this client does not compose that
transport. Unselected member declarations have no blanket curve restriction.

## 5. Conditional rent is not the wallet budget

The [retained conditional rent checkpoint](PIV1_M5_TESTNET_READINESS.md#retained-conditional-initializer-rent-checkpoint)
binds historical finalized observations from 2026-10-18, slots454098679–454098682.
Assume zero-prefunded valid targets and supported new ALT/Squads resources only.

| Lamports | Shared manager/referrer | Distinct receivers |
| --- | ---: | ---: |
| Gross rent-payer outflow before fees | 49,936,400 | 50,271,680 |
| Account balances held after buffer refund | 41,366,440 | 41,534,080 |
| Buffer refund to creator only | 8,569,960 | 8,737,600 |
| Separate required payer residual (not spent rent) | 650,240 | 650,240 |

The creator refund does not replenish the distinct rent payer automatically.
Each recipient vault's historical 650,240-lamport floor is separate and excluded
from these outflows; its actual balance, shortfall and provisioning remain OPEN.
Actual original prefunds/shortfalls, recipient balances/provisioning, loader and
deployment, multisig setup, operational leg rents, contribution capital, network/
priority fees, peak liquidity and full per-wallet funding remain OPEN. No exact
funding requirement or fee quote is supplied. Do not use principal/pending/KIF to fund deployment, fees or operational rent,
or use token-target excess to replace the payer's original required shortfall.

## 6. Next action and evidence handoff

This finite map points into the existing register; it does not open a second
readiness register. Keep actual statuses and any changed evidence there.

| Unresolved handoff | Next concrete preparation | Evidence / completion boundary |
| --- | --- | --- |
| Founder public choices/control | Seven generated identities are funded under D-041; review actual public inventory and prepare exact Stage B creation scope. Slot/pause/anchor/vault/profile choices remain OPEN. | Public identity setup and register; local order is not PIV slots or current membership. |
| Retained candidate vs deployment selection | Review root's source/artifact binding and select an exact artifact only in a later reviewed package. | Receipt below and [register](PIV1_M5_TESTNET_READINESS.md); no rebuild or audit inference. |
| Fresh accounts/protocol/resources | Arrange reviewed read-only acquisition with genesis checks, actual metadata/raw bytes and source guards. | Coherent/fresh account, custody, protocol, rent and resource evidence; supplied snapshot decoding alone is insufficient. |
| Funding and live operator tooling | Complete scoped provisioning/transport/confirmation and real per-actor funding/fees/peak plan. | Existing [rent checkpoint](PIV1_M5_TESTNET_READINESS.md#retained-conditional-initializer-rent-checkpoint) is conditional only; workflow/tooling remains OPEN. |
| Governed operator compositions | Prepare separate pause/unpause, guardian-sync/rotation and recipient-update compositions when required; paused initialization cannot proceed to economics until governed unpause. | Production scope and limitations stay in the [register](PIV1_M5_TESTNET_READINESS.md#finite-readiness-matrix); no such client workflow is supplied. |
| Readiness and live decision | Separate review reconciles exact identities/artifact/commands/budget/residual risks and M4 reservation in the register. | Future explicit founder authorization for the exact sensitive steps; none is requested or granted by this draft. |

**Further approval card — DRAFT, NOT READY, NOT AUTHORIZATION:** The seven keys
and capped wallet funding are complete under D-041; that authority is not reopened
or extended here. Squads creation, actual accepted governance settings, exact PIV
program/artifact/ALT/recipients, deployment budget and PIV transport remain OPEN.
Prepare the actual multisig-creation packet/current fees/zero-fee treasury
condition/simulation and request its separate bounded authorization before execution.
No Squads creation, PIV deployment or authority transfer occurred. M4 remains
technically incomplete with criterion unwaived and stopped drafts untouched;
M5 preparation only, public NOT READY and deployment NOT AUTHORIZED. Preserve
all attempts/receipts and the 2-GiB reserve; future publication is reviewed
integration only, with main remaining `e8f33f3`.
