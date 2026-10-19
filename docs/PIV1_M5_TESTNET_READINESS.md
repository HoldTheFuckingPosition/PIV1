# M5 Testnet preparation and readiness register

Status on **2026-10-19 Europe/Warsaw**: **PREPARATION AUTHORIZED (D-039) / PUBLIC PACKAGE NOT READY / DEPLOYMENT NOT AUTHORIZED**.
M4 remains **TECHNICALLY INCOMPLETE / ONE RECORDED EVIDENCE CRITERION OPEN**.
The internal-callee-failure observation is unwaived; the rejected operation and
its drafts remain stopped. The [limited-deliverable proposal](PIV1_M4_VALIDATION_SCOPE_REVIEW.md)
is still pending, not accepted by this authorization.

This is the single finite M5 register under [D-039](PIV1_DECISIONS.md#d-039--confirmed--m5-preparation-with-an-explicit-m4-reservation).
Baseline HEAD/main/integration was `a524cc616624f41aa54141b3ac0838edd6c0f48b`,
clean as jerem on WeatherTrader2; protected Task 2.3 remains
`3677fee97e3617ee65e2828d222008ba0952bb3e`. New reviewed work publishes only to
`integration/piv1-testnet`; D-038 grants no new main-publication authority.

## Current offline guardian registry and earned-reward inspection

Baseline integration `bb21213e5d17bc047443633840f1be0be54f9c49`, clean as jerem on
WeatherTrader2; main `e8f33f3`, protected Task 2.3 `3677fee`. Four additive client
paths: `clients/cli/guardian-inspect.cjs`, focused tests, null example and append-only
usage. Configured GPT-6.1 Sol/High writer and separate Sol/Extra High source/parser/
final-tree review; root model/speed unexposed, no configuration change. AI review
is not a professional audit. Existing production/economics, clients, dependencies,
toolchains and artifacts unchanged. Root publishes reviewed integration only.

| Mode | Supplied accounts | Bounded checks / output |
| --- | --- | --- |
| current-set | Config, registry, six ordered rewards | Existing Config decoder/topology; supplied current registry address/revision/bump and exact slot/key/revision reward binding; stored fields only. |
| earned-record | One reward | Self guardian/revision/slot and canonical PDA/bump, independent of current Config/registry/membership/Clock; stored earned balance/counters only. |

Registry is exactly210 bytes: discriminator, version/bump, u64LE revision, six keys.
Reward is exactly84 bytes: discriminator, version/bump/slot, u64LE revision, guardian,
Option<u64> activity then three u64 ledgers. None places monetary fields eight bytes
earlier and leaves eight zero allocation-tail bytes; Some0 differs from None.
Strict discriminator/version/Option/size/tail and canonical base64/metadata/address/
bump checks preserve all u64 values as decimal strings. Registry keys are distinct/
nonzero; reward slot is0..5, earned>=claimed and claimable=earned-claimed. These are
bounded standalone identities, not full Rust state validation or runtime authentication.
Current mode reuses unchanged Config validation, including canonical topology;
it requires no distribution snapshot. Earned mode does not impose current revision,
membership, activity or signer/curve conditions; it cannot classify actual retirement
from one record. Mode-specific structural-check lists describe only supplied accounts.

Summaries expose last_active_period, stored claimable, cumulative earned/claimed
and the exact cumulative_claimed source field for a future expected counter. This
is a stored observation requiring refresh, not replay-validity, available payment,
backing, solvency or claim eligibility. Current six records do not enumerate all
historical liabilities and are never summed as a global liability. No real Clock/
period/activity bitmap, membership authority, rent health, snapshot coherence/freshness,
wallet control, funding/resources or executable action is inferred. The inspector
produces no transaction or heartbeat/claim command. Separate reviewed client plans
still require independently authenticated execution inputs and source guards.

Strict unknown/duplicate/unused/malformed-known-partial/schema/identity inputs reject;
null/missing inputs give INCOMPLETE/inspection:null. Complete input gives only
OFFLINE_GUARDIAN_INSPECTION with public/deployment readiness false. Public supplied
metadata/labels/slot are not authenticated chain observations. No RPC/wallet/config/
secret access, key generation, signing, sending or new economic rule is implemented.

Actual new guarded host executions:

```sh
python3 /tmp/piv1-m5-guardian-inspect-20261019-a/node-driver.py tests
python3 /tmp/piv1-m5-guardian-inspect-20261019-a/node-driver.py template
python3 /tmp/piv1-m5-guardian-inspect-20261019-a/node-driver.py example
```

All 7 groups and both standalone commands pass on first execution. Four actual
CLI snapshot/report pairs (current-none/current-mixed/earned-none/earned-some) retain
18 small literal synthetic account records, not valid public identities, rent-backed
runtime states or globally reconciled economics. Root independently decodes all named
Config fields with retained reviewed source cursor definitions and fresh literal
registry/reward/PDA cursors; separate review uses its own parser. Every decoded field,
metadata/layout/hash/context, supplied current tuple and observed-counter summary agrees.
Finite full-width/Option/precision, malformed bytes/identities/partials, ledger and CLI
cases are checked. One report wording refinement limits earned scope to Reward84;
a malformed-partial test refinement keeps invalid fields present. Original unexecuted
drafts are preserved; no failed application test/runtime run is claimed.

All 211 frozen inputs match. Evidence root
`/tmp/piv1-m5-guardian-inspect-20261019-a` binds source/static/host/data/final-tree
reviews and publication identity. Old suites/runtime are retained, not rerun; no
Bank trace processing, Rust build, Bank/SBF or RPC is new. Preserve evidence and the
2-GiB reserve. No keys/signing, live transactions, funds, Mainnet or authority transfer.

Next assemble exact public identities/artifact/funding and the founder workflow,
then separate readiness review. Remaining live identity/control, freshness/custody,
fees/rent/funding/resources, supported profiles and M4 reservation stay OPEN.
M4 technically incomplete, criterion open/unwaived and stopped drafts untouched;
M5 PREPARATION only, public package NOT READY, deployment NOT AUTHORIZED.

## Retained offline guardian heartbeat and earned KIF claim checkpoint

Baseline integration `ddd47234ffd4ee39e1264665d0ab5d48270967ca`, clean as jerem on
WeatherTrader2; main `e8f33f3`, protected Task 2.3 `3677fee`. Additive client
`clients/cli/guardian-kif-plan.cjs`, focused tests, nullable public example and
append-only usage were written by configured GPT-6.1 Sol/High, separately reviewed
by Sol/Extra High. Root/speed unexposed; no configuration change. AI review is not
a professional audit. Old clients, production/economics, dependencies/toolchains/
artifacts unchanged. Reviewed integration publication only; main remains preserved.

| Action | Source payload | Native roles | Full bytes: shared/distinct payer | Signature slots |
| --- | --- | --- | --- | --- |
| guardian-heartbeat | `PIV1HB01`, v1, u8 slot, u8 vault; 11 bytes | 13 | 601 / 697 | 1 / 2 |
| claim-kif | `fd97ac0bca4d76aa`, u64LE amount, u64LE expected cumulative claimed; 24 bytes | 5 | 382 / 478 | 1 / 2 |

Heartbeat uses current revision/six ordered guardian keys, selected slot, current
multisig/create key and vault index. It derives six canonical reward PDAs, loader
ProgramData, Squads multisig and expected vault authority. Vault is not supplied
or signed; there is no proposal/approval or Squads CPI. Production authenticates
current authority/membership correspondence and derives the half-open 2,592,000-second
period from real Clock. Only selected reward is requested writable; selected guardian
is requested readonly signer. Program is both instruction target and native role10,
counted once. Shared fee payer makes guardian globally writable; distinct payer
leaves it readonly signer. No pause gate, payout, new liability or retroactive frozen
snapshot change occurs in this heartbeat path. Client computes no current period.

Claim takes the stored earned guardian/revision/slot tuple, positive exact u64
amount and explicit u64 cumulative-claimed counter, independent current membership.
It derives Config/reward/KifSOL PDAs; all four isolated state/source/destination
roles are writable, guardian signs and is fixed destination, System is readonly.
No current registry/other guardians, Clock, pool, round, principal/pending/escrow/
operations account is supplied. Retired/inactive guardians retain prior earned rights;
pause does not block these claims. Production checks individual/global accounting,
claimable/backing/rent, replay/counter and checked arithmetic, then exact transfer/
state effects. Client does not infer available amount, backing, fees or success.
No arbitrary payout, activity/reward creation, carry consumption or HWM reset.

Direct transaction plans support on-curve external signer declarations only; one
zero64 signature when payer=guardian, two otherwise. Production System-owned PDA
wallet signing by CPI remains supported within its recorded profile, but this
planner supplies no such caller/Squads signing composition. Unselected heartbeat
member keys have no blanket curve constraint. Actual owner/data/control, current
identities/authority, reference blockhash/slot/height and CU/heap/loaded requests
remain unverified. Declared complete inputs yield only OFFLINE_PLAN_ONLY; missing
values produce INCOMPLETE/plan:null. Strict unknown/duplicate/unused/malformed
partials, identities/aliases, byte/u64/resource bounds and unsupported CLI reject.
No wallet/config/secret read, key generation, signing, send or arbitrary destination.

Actual new guarded host executions:

```sh
python3 /tmp/piv1-m5-guardian-kif-plan-20261019-a/node-driver.py tests
python3 /tmp/piv1-m5-guardian-kif-plan-20261019-a/node-driver.py template
python3 /tmp/piv1-m5-guardian-kif-plan-20261019-a/node-driver.py example
```

All 7 groups and both standalone commands pass on first execution. Four complete
actual synthetic CLI manifests/stdout reports are retained, independently decoded
by root and separate reviewer against literal pre-client source oracles. Complete
packet headers/zero signatures/key/program union/requested and global privileges,
three resource selectors, payloads/PDAs and hashes agree. 207 frozen inputs match.
One draft design size estimate was corrected before implementation; original design
retained. Known-partial direct-signer validation was tightened before execution,
with the first unexecuted module draft preserved; no test/runtime failure. Evidence root
`/tmp/piv1-m5-guardian-kif-plan-20261019-a` binds static/source/host/wire/final-tree
reviews and publication identity. No old Node suite, Rust build, Bank/SBF, Solana
runtime, RPC, live activity/claim or wallet-control proof is new. Older results
retain their individual scope. Preserve all evidence and the 2-GiB reserve.

Next complete readable guardian-registry and earned-reward/counter inspection and exact identities,
funding/artifact/founder workflow, then independent readiness review. Current Config/
distribution inspection does not expose every earned-reward input by itself.
Public package NOT READY: live state, selected identities/control, actual funding/
fees/rent/resources and M4 reservation remain OPEN. M4 technically incomplete,
criterion open/unwaived, stopped drafts untouched; M5 PREPARATION only, deployment
NOT AUTHORIZED. No keys/signing, live transactions, funds, Mainnet or authority transfer.

## Retained offline delayed-withdrawal leg client checkpoint

Baseline main/integration `e8f33f311f427c02529108fdd39fb85691f99479`, clean as
jerem on WeatherTrader2; protected Task 2.3 `3677fee`. One explicitly configured
GPT-6.1 Sol/High writer adds only `clients/cli/withdrawal-plan.cjs`, focused tests,
null example and append-only usage. Separate Sol/Extra High review binds source,
actual CLI evidence and final staged tree. Root/speed unexposed; no global config
change. AI review is not a professional audit. Main remains at baseline; root
publishes reviewed integration only.

| Action | Exact native data | Shared/distinct nonsigner roles | Actual full CLI bytes |
| --- | --- | --- | --- |
| initiate-withdrawal-leg | `PIV1IL01`, v1, u32LE validator index; 13 bytes | 24 / 25 | 1,030 / 1,063 |
| finalize-withdrawal-leg | `PIV1FL01`, v1, u64LE existing-leg index; 17 bytes | 23 / 24 | 1,001 / 1,034 |

Both derive temporary metadata/Stake PDAs from literal `withdrawal-leg`/
`withdrawal-stake`, u64LE round sequence and u64LE leg index. Initiation's declared
next index is not in its payload; production reads it from current round state.
A competing successful initiation requires refreshed declarations. Finalization
accepts an existing leg, including out-of-order ready legs. Full-width values
encode exactly but do not establish valid counters or eligibility. Initiation
adds source/pool withdraw PDA; finalization instead adds canonical Stake History.
All native roles are nonsigner; only the external declared fee payer occupies
one zero 64-byte slot. Shared manager/referrer is one role; distinct referrer
stays readonly, and initiation/finalization writable matrices stay separate.
Three budget selectors 2/1/4 encode explicit resource requests, not measured
sufficiency. Full packets fit the retained 1,232-byte cap without ALT.

Strict schema/unknown/duplicate fields, unsupported CLI values, malformed known
partials, non-null unused action inputs, invalid keys/aliases and wrong resource
ranges reject. Missing inputs give INCOMPLETE/plan:null, complete declarations
only OFFLINE_PLAN_ONLY; public/deployment readiness false. No RPC/wallet/secret
configuration, signing/send path or arbitrary destination/amount/minimum exists.
Reference blockhash/slot/height are unverified declarations, not replay protection,
current blockhash lifetime or selected public identities/control.

Production requires unpaused WithdrawalActive, matching current round, normalized
custody, rent/owner/authority checks and current pool epoch/supply. Initiation
supports only active-validator sources satisfying exact list index/vote/suffix/
canonical source, priority/preference, balance, flags/lockup and dynamic minimum.
It derives maximum-safe fill, fee/burn, snapshot/current floors, protected variant24
and residual HWM; immediate deactivation/metadata recording are atomic. Client
selects no quote, optimizer, amount, minimum or authenticated candidate.

Both temporary PDA prefunds become pending before current actual rents are advanced
from OperationalSOL. They are not caller rent payments or historical yield.
Finalization's real Stake withdrawal proves inactivity against runtime Clock/History;
no elapsed-epoch guess or public cooldown deadline is emitted. Complete Stake value
enters fixed escrow; original recorded Stake/metadata advances return operations,
metadata excess enters pending. Ambiguous Stake surplus above original advance plus
current delegation rejects. Cooldown reward is later-cycle yield, not this round's
payout; loss or protected-value failure records recovery without lowering HWM.
Finalization closes metadata and may complete funding only under exact target/
leg/count predicates; settlement remains separate. RecoveryRequired means stop/report,
with no invented governed exit or partial beneficiary payout.

Actual new executions, using the retained pinned Node/SDK and resource guard:

```sh
python3 /tmp/piv1-m5-withdrawal-plan-20261019-a/node-driver.py tests
python3 /tmp/piv1-m5-withdrawal-plan-20261019-a/node-driver.py template
python3 /tmp/piv1-m5-withdrawal-plan-20261019-a/node-driver.py example
```

All 7 groups plus both standalone commands pass on first execution. Four complete
synthetic manifests/reports are retained from suite subprocesses. Root's independent
Python shortvec/base58/SHA256/Ed25519-PDA audit and separate source-bound byte audit
check complete packet/signature/header/key/role/privilege/resource/native data and
hashes. These are host wire/source results, not new Solana runtime or real staking.
Old suites and runtime results retain their prior individual scope; no new RPC,
Rust build, Bank/SBF, dependency/toolchain/artifact/production/economic change.
Freeze binds 204 inputs. Evidence root
`/tmp/piv1-m5-withdrawal-plan-20261019-a` records guards, source/static/actual-wire/
final-tree reviews and publication identity. Preserve attempts/evidence/2-GiB reserve.

Next prepare guardian heartbeat and earned-claim commands, then exact identities,
funding/artifact/founder workflow and readiness review. Dynamic source availability,
quotes, actual rent/funding/fees/resources/control remain OPEN; this client does
not close those blockers. M4 technically incomplete, stopped criterion open/unwaived
and drafts untouched. M5 PREPARATION only, public NOT READY, deployment NOT
AUTHORIZED. No keys/signing, live transactions, funds or authority transfer occurred.

## Retained D-040 main publication checkpoint

The founder explicitly requests main publication of all already validated work.
D-040 covers the ten reviewed integration commits after `a524cc6` through
`78ab8a4`, plus this reviewed authorization/checkpoint documentation, by normal
fast-forward. The [decision](PIV1_DECISIONS.md#d-040--confirmed--publication-of-reviewed-m5-preparation-to-main)
and `/tmp/piv1-main-publication-20261019-a/history-audit.json` identify exact
commits, retained review trees and hashes. Root verifies actual resulting local/
remote main and integration refs; Git and the evidence directory's
`publication.json` record completion/identity. Protected Task 2.3 stays `3677fee`.

Only six canonical documents are newly edited in this publication turn; all ten
existing implementations are integrated unchanged. No application test/build/
Solana runtime/RPC is rerun, and prior evidence retains its individual scope.
No production/economic/dependency/toolchain/artifact change, founder acceptance,
M4 closure or criterion waiver is inferred. M4 remains technically incomplete,
criterion open/unwaived; M5 PREPARATION only, public NOT READY, deployment NOT
AUTHORIZED, stopped drafts untouched. The main/integration-only statements above
and below retain historical scope and are superseded only for this bounded sequence.
Future work returns to reviewed integration checkpoints until new main authority.
Next: withdrawal-leg and guardian/claim commands, exact funded artifact/founder
workflow and separate readiness review before any future live authorization card.

## Retained distribution preparation and completion client checkpoint

Baseline integration `5a76a0646bd3e1c36018e639e388c7e3fc406e4c`; main `a524cc6`,
protected Task 2.3 `3677fee`. [Usage](../clients/cli/README.md) adds an offline
`distribution-plan.cjs --manifest PATH` / `--template` planner and null manifest.
Four client paths (module/test/example/append-only usage) were written by an
explicitly configured Sol/High agent and separately reviewed by Sol/Extra High.
Production/economics, old clients, Rust, dependencies, toolchains and ELFs are
unchanged. Root model/speed stay unexposed; no global setting changed. Separate
AI review is not a professional audit.

The explicit action selects only the supplied public declaration profile. Common
program/fee payer/protocol/context/resources are required; preparation additionally
requires CURRENT registry revision/six guardian slots, withdrawal an explicit u32
validator index/source Stake address, and settlement the FROZEN revision/six slots/
two recipients. Integration takes none of those action-specific contexts. Non-null
unused inputs, malformed known partials, duplicates/unknown JSON fields, invalid
identities/aliases and unsupported values reject. Missing values give INCOMPLETE/
plan:null; complete values give only OFFLINE_PLAN_ONLY. No implicit signer/input,
RPC, wallet/config/secret read, key creation, signing or sending is implemented.

| Action | Native payload | Shared/distinct native roles | Actual full CLI bytes |
| --- | --- | --- | --- |
| prepare-liquid | `PIV1PD01`, v1, 9 bytes | 26 / 27 | 1,092 / 1,125 |
| prepare-withdrawal | `PIV1PW01`, v1, u32LE index, 13 bytes | 28 / 29 | 1,162 / 1,195 |
| settle | `PIV1SD01`, v1, 9 bytes | 27 / 28 | 1,125 / 1,158 |
| integrate-pending | `PIV1IP01`, v1, 9 bytes | 19 / 20 | 861 / 894 |

Every native role is nonsigner; only a distinct external fee payer occupies one
zero 64-byte signature slot. Exact flags preserve readonly protocol/list accounts,
current preparation reward records and frozen settlement writable rewards/recipients.
Preparation has registry + six current rewards + Clock; withdrawal adds Stake and
explicit source. Settlement has Clock, two frozen recipients and six frozen rewards,
with no registry. Integration adds Clock only. Canonical reward seeds preserve
`guardian-reward`, guardian bytes, u64LE revision and u8 slot; no sorting/remapping.
Guardian wallets are not native signer roles: fee-payer/guardian and external
recipient/guardian overlap remain supported. Native roles and payer remain distinct.

References and resource requests are unverified declarations: three selectors
2/1/4 encode the existing CU/heap/loaded ranges, not measured public sufficiency.
Largest packet is 1,195 bytes, 37 below the 1,232-byte cap; this fitting profile
needs no ALT or dependency change. No quote, source capacity, validator-list match,
gross yield, target, technical minimum, exact payout/integration value, due time or
eligibility follows from encoding. Index spans u32 including zero; revisions span
u64 and six ordered slots have no blanket curve restriction. Production authenticates
actual current or frozen tuples and all custody/pool/Clock facts.

Source guards preserve unpaused valid lifecycle, ten days from successful preparation
rather than settlement, and the separate 24-hour retry after valid insufficiency.
No-yield liquid evaluation leaves PIV state/clocks unchanged without snapshot/event;
withdrawal no-yield rejects ZeroTarget. Authenticated insufficient preparation writes
only its timestamp/event without a round or new ten-day clock. Withdrawal preparation
queries Stake minimum; it creates no leg, burns no tokens and deactivates no stake.
Settlement requires EscrowFunded and fully finalized target/legs, pays frozen net
beneficiaries and credits KIF under preserved fee/carry/HWM guards, or records recovery
without payments. Integration requires Settled, adds full contribution value to HWM
while moving remaining physical custody once, and completes to Idle without staking.
All outcomes remain unknown to this tool; RecoveryRequired means stop/report.

Root executes only three new frozen guarded phases:

```sh
python3 /tmp/piv1-m5-distribution-plan-20261019-a/node-driver.py tests
python3 /tmp/piv1-m5-distribution-plan-20261019-a/node-driver.py template
python3 /tmp/piv1-m5-distribution-plan-20261019-a/node-driver.py example
```

All 7 groups pass on first execution. Eight complete synthetic CLI manifests and
stdout reports from suite subprocesses are retained. Root's independent Python
base58/Ed25519/SHA256/shortvec decoder and separate review reconstruct exact actual
wires and PDAs against independent source/role matrices. Full-width index/revision,
slot order, current/frozen isolation, aliases, missing/invalid inputs and resource/
CLI paths are checked; these are finite host domains, not economic runtime coverage.
Evidence root `/tmp/piv1-m5-distribution-plan-20261019-a` binds 83 source/tool/
evidence inputs, design/static/actual-wire/final-tree reviews and guarded receipts.
Pinned Node 24.19.0 / SDK 1.98.4, retained socket-denial filter, 512-MiB Node heap,
monitored 2.5-GiB RSS+swap and 2-GiB disk reserve. No new RPC/Rust build/Bank/SBF or
old suite run; previous results remain retained. No live state/control/resource,
funding/fees, M4 completion, founder acceptance or deployment authority is inferred.

Next prepare withdrawal-leg and guardian/claim commands plus exact funded artifact/
founder workflow and readiness review. M4 criterion remains open/unwaived, stopped
drafts untouched; M5 PREPARATION only, public package NOT READY, deployment NOT
AUTHORIZED and reviewed integration only. No new main-publication authority.

## Retained protected historical-principal SOL client checkpoint

Baseline integration `7216a6b58428bb8e17836e9f9acfcfe0a324c220`; main `a524cc6`,
protected Task 2.3 `3677fee`. [Usage](../clients/cli/README.md) adds
`principal-stake-plan.cjs --manifest PATH` and `--template`. Three new client files
plus append-only README were written by bounded Sol/High and separately reviewed
by Sol/Extra High. Production/economics, old clients, Rust, dependencies, toolchain
and ELFs are unchanged; actual root model/speed remain unexposed, global settings
unchanged. Separate AI review is not a professional audit.

The sole action converts explicitly requested already recognized historical SOL.
It does not recognize, integrate or stake pending contributions. Public program/
fee payer/protocol/context/resources and amount/minimum are explicit declarations;
missing input gives `INCOMPLETE`/`plan:null`, known invalid input rejects, complete
input gives only `OFFLINE_PLAN_ONLY`. Strict duplicate/unknown JSON and canonical
public keys/decimal strings reject unsupported inputs without secret/config reads.
Positive amount spans 1..u64MAX; explicit caller token-base-unit minimum spans
0..u64MAX. A minimum larger than native lamports is syntactically valid: the units
differ and the client has no quote. Zero adds no caller strengthening but production
still enforces its Config-derived protected minimum and positive minted output.

The exact native payload is `PIV1SP01`/v1/u64LE amount/u64LE caller minimum, 25 bytes.
There are 19 shared or 20 distinct manager/referrer native roles; all are nonsigners.
Writable indices are 0/3/7/10/14/16/17, plus 18 only for distinct referrer. List stays
readonly; canonical pool withdraw authority is always final, derived under the
fixed official pool program from `[pool, "withdraw"]`, not caller-selected. Sole
outer signer is the declared fee payer. Production signs principal SOL internally
for protected pool variant 25; the client builds no speculative pool CPI, Squads
approval or extra guardian/Clock/Rent/Stake account.

Required live prerequisites stay explicit: unpaused valid Idle and bound custody,
amount within recognized/backed historical SOL, normalized economic custody,
current authenticated pool epoch/total/supply, permissionless zero-fee SOL deposit
(valid parsed 0/0 or 0/N), positive output meeting max(Config floor,caller minimum),
exact CPI observations, no loss of even one lamport in historical book value and
no HWM breach.
Only historical asset-unit ledgers change; pending/HWM/rent/quarantined native
Token balances/KIF liability/carry and unrelated state are preserved. No amount
optimizer, quote, current book value, eligibility or full state validator is added.
A rounding-rejected amount remains queued; no guard is relaxed to force staking.

Root actually executes only these new guarded phases:

```sh
python3 /tmp/piv1-m5-principal-stake-plan-20261019-a/node-driver.py tests
python3 /tmp/piv1-m5-principal-stake-plan-20261019-a/node-driver.py template
python3 /tmp/piv1-m5-principal-stake-plan-20261019-a/node-driver.py example
```

All 7 groups pass on first execution. The suite retains actual complete shared/
distinct synthetic CLI inputs/outputs for separate review. Shared full wire is
877 bytes/22 static keys, distinct 910/23; both headers `[1,0,14]`, one zero
64-byte signature slot and three explicit resource requests (selectors 2/1/4).
Independent root Python and separate review bind literal source roles/flags/
payload, canonical PDAs, raw legacy shortvec/keys/budgets/amount/minimum and hashes.
Boundary values include full-width amounts/minima and zero minimum; protocol
sharing, protected/native fee aliases, strict partial/schema/resource/CLI negatives
are covered. These are finite host/wire results, not live execution/resource/control
proof. The source error guide is finite/nonexhaustive, not an observed diagnosis.

Evidence root: `/tmp/piv1-m5-principal-stake-plan-20261019-a`; design/source/static/
wire/execution/final-tree review receipts and 67-input freeze are retained.
Pinned Node 24.19.0 / SDK 1.98.4 and unchanged Linux socket-denial guard, 512-MiB Node
heap, monitored 2.5-GiB RSS+swap/2-GiB disk reserve. No RPC/Rust build/Bank/SBF or
old suite was rerun. Previous production/local results remain retained evidence.

All reports leave quotes/effective minimum/minted output/book values/eligibility,
fees/rent/funding and public state/control/resources unverified or unknown. M4
criterion stays open/unwaived; stopped drafts are untouched. M5 PREPARATION only,
public package NOT READY, deployment NOT AUTHORIZED, reviewed integration only.
Next prepare remaining distribution/claim operator plans, then exact funded artifact/
founder workflow and separate readiness review before a future live approval card.
No new main authority, secret/key/signing, live operations, funds or authority change.

## Retained offline Config and distribution inspection checkpoint

Baseline integration `454fc5988f523693c1cb401a37d794d3c80e7200`; main `a524cc6`
and protected Task 2.3 `3677fee` remain unchanged. [Usage](../clients/cli/README.md)
adds `state-inspect.cjs --snapshot PATH` and `--template`. Four new client files
plus append-only usage were written by bounded Sol/High, separately reviewed by
Sol/Extra High. No production/economic/Rust/dependency/toolchain/ELF change; previous
initializer/funding/intake modules and suites are unchanged.

Explicit supplied Config and ActiveDistribution envelopes produce only offline
inspection; null/missing required inputs stay INCOMPLETE with no inspection.
Known malformed input rejects. Strict JSON/canonical base64, public identities, exact
1014/891-byte allocation, discriminator/version/initialized flags, strict bool/
Option/phase tags, zero migration reserve/tail and named canonical PDA/bump
bindings are checked. This is a finite byte/identity subset, not the Rust economic
validators. Both Config timestamp Options and the 88-byte completion-summary
Option shift all following fields; every u64/i64 remains a decimal string. All
named fields and stored guardian slots are exposed; frozen/current recipient and
registry bindings are separate. No computed custody, book valuation, next action,
claimability or live Clock deadline is emitted.

Root executes the new focused suite and six standalone CLI phases once each:

```sh
python3 /tmp/piv1-m5-state-inspection-20261019-a/node-driver.py tests
python3 /tmp/piv1-m5-state-inspection-20261019-a/node-driver.py template
python3 /tmp/piv1-m5-state-inspection-20261019-a/node-driver.py example
python3 /tmp/piv1-m5-state-inspection-20261019-a/node-driver.py fixture
python3 /tmp/piv1-m5-state-inspection-20261019-a/node-driver.py bootstrap
python3 /tmp/piv1-m5-state-inspection-20261019-a/node-driver.py withdrawal
python3 /tmp/piv1-m5-state-inspection-20261019-a/node-driver.py settled
```

All 7 groups pass on first execution. A draft guardian-revision inequality oracle
was corrected before execution; no production defect or failed run is claimed. Tests cover the four Config Option combinations crossed with
both summary shapes, full-width signed/unsigned boundaries, strict malformed/
partial inputs, padding versus reserve, identity/bump/reference negatives and
no false full-semantic validity claim. Root and reviewer independently decode
all fields from four pairs extracted directly from retained real-pool stdout
`/tmp/piv1-bank-smoke-real-pool-run-20261018-a/runtime.stdout` (558,238,168 bytes,
SHA256 `f6eb3fed9c2fe21016025f50e2595275684212648e55b26001c0d97d50693058`).
Both compare actual CLI reports across bootstrap, active withdrawal, settlement
and completed Idle. The repository fixture is byte-exact to the final supplied
pair (fixture SHA256 `0bdafa7694994c6ed66e5d48ae45dd1cf996455a9becfe6db9754e42e8c2b8f6`).
Final stored HWM is 121,955,000,003, historical SOL 200,000,002, historical tokens
111,608,750,001, pending zero and earned KIF liability 216,149,757. Config has
998 payload bytes plus 8-byte discriminator/8 zero tail; completed distribution
883 plus discriminator, with no trailing slack. Context labels do not authenticate
these synthetic local observations as public Testnet PIV1 accounts.

Evidence root: `/tmp/piv1-m5-state-inspection-20261019-a`. Design/static/fixture/
execution/all-field/final-tree receipts and a 69-input freeze preserve review.
Pinned Node24.19.0, existing SDK1.98.4, unchanged socket-denial/resource guard;
512-MiB Node heap, monitored 2.5-GiB RSS+swap and 2-GiB disk reserve. No new RPC,
Rust build, Bank/SBF or prior client suite was executed; old results are retained.
Separate AI review is not a professional audit. Root model/speed unexposed,
global settings unchanged.

All reports keep provenance/coherence/freshness/custody/rent health/full Rust state
validation false or unknown; fees/funding/resources/valuation/eligibility remain
unknown. Public readiness and deployment authorization stay false. Next prepare
protected principal-staking/remaining operator plans and exact funded artifact/
founder workflow. M4 remains technically incomplete, criterion open/unwaived and
stopped drafts untouched. Reviewed integration publication only; no main authority,
secrets/keys/signing, live operations, funds or authority changes are inferred.

## Retained contribution and initial-bootstrap client checkpoint

Baseline integration `b4827e09dc5092469afbd7fceb9ee6cc08cd4b6c`; main stays
`a524cc6`. [Client usage](../clients/cli/README.md) now documents exactly three
additive offline actions: `contribute-sol`, `contribute-jitosol` and
`bootstrap-initial`. Four client files are new/updated; prior initializer/funding
code/tests/templates, production/economics, Rust, dependencies, toolchain and ELFs
are unchanged. Bounded Sol/High writer and separate Sol/Extra High review were
explicit; root model/speed remain unexposed, global settings unchanged.

Each action uses an explicit public manifest, never an inferred identity/action.
Missing required fields return `INCOMPLETE`/`plan:null`; malformed known fields,
non-null unused action inputs, unsupported aliases and unknown/duplicate JSON keys
reject. Complete declarations yield only `OFFLINE_PLAN_ONLY`. Positive u64 amounts
are decimal strings; bootstrap has no caller amount, value or governance input.
Context is unverified and must be refreshed before any future signed step.

SOL uses native `PIV1DS01`/v1/u64LE, 17 bytes and 6 accounts. JitoSOL uses
`PIV1DJ01`/v1/u64LE, 17 bytes and 8 accounts. The source is an explicit existing
legacy Token account, ATA or otherwise, possibly off-curve. The owner signs;
arbitrary authority-account ownership is supported except PIV/internal restrictions
and exact Token-owned 355-byte multisig. A delegate-only signer is unsupported;
self-delegate allowance can still matter. Neither source nor owner state/control
is authenticated by this tool. Both actions require unpaused valid bindings and
coverage of both old pending obligations; intake is not Idle-only. They record
only the requested pending increment, with no new withdrawal/reward right or HWM change.

Initial bootstrap uses `PIV1IB01`/v1, 9 bytes and 18/19 accounts according to the
actual configured shared/distinct receiver shape. PIV authority is a native
nonsigner; only outer fee payer signs. Production derives all recognized pending
amounts and gross pool book value from current authenticated Config/protocol/Clock,
then transfers custody to principal and updates initial history/HWM. Initial
unpaused Idle/zero history, normalized custody, nonzero recognized pending and
current epoch/supply constraints remain mandatory/unverified. Positive units with
floored book value zero are valid. No Squads approval, caller valuation, staking,
pool/Stake CPI, later integration or replay support is implied.

All three explicit resource requests are required: client ranges 1..1,400,000 CU,
32,768..262,144 heap bytes in 1,024 increments and 1..8,388,608 loaded bytes. They
are request ranges for this client, not full cluster limits or measured public
sufficiency. The single legacy packet contains selectors 2/1/4 budget requests and
one native action. Actual global signer/writable union is checked independently
from requested native roles, including token-owner writable escalation as fee payer.
No ALT is required for this fitting profile; zero signatures are not real control.
The finite error guide is nonexhaustive source guidance, not a live error/state decoder.

Root actually executed each frozen phase once:

```sh
python3 /tmp/piv1-m5-intake-plan-20261019-a/node-driver.py tests
python3 /tmp/piv1-m5-intake-plan-20261019-a/node-driver.py template
python3 /tmp/piv1-m5-intake-plan-20261019-a/node-driver.py example
```

All seven groups pass on first execution, zero failures/skips; guarded elapsed
3.880 seconds. Literal codecs/roles/PDAs, 14 positive u64 values per intake, actual
one/two signer unions, both bootstrap shapes, explicit resource domains and
strict partial/schema/alias/CLI paths are covered. Eight CU/loaded combinations
are encoded; 225 aligned heap choices are validated without claiming 225 runtime
executions. The supplied null example produces no plan. Three complete synthetic
CLI manifests and exact stdout reports are retained for independent inspection:

| Actual retained CLI action | Native bytes/accounts | Zero signature slots | Full wire bytes |
| --- | --- | ---: | ---: |
| SOL, separate fee payer | 17/6 | 2 | 504 |
| JitoSOL, owner also fee payer | 17/8 | 1 | 474 |
| Initial bootstrap, shared receiver | 9/18 | 1 | 828 |

Root's separate Python decode checks legacy headers/shortvec/static keys,
all zero 64-byte slots, instructions, amounts, budget bytes, flags and file/wire
hashes against source-derived expectations. Separate review checks the same
retained evidence without Node rerun. Other supported fee/receiver forms execute
in focused SDK tests; the independent pre-execution arithmetic gives 408/504 SOL,
474/570 Jito and 828/861 bootstrap bytes, with its assumptions recorded separately.
The table above is the three retained actual CLI artifacts, not public transactions.
No Solana program execution, signature verification, cluster submission, actual state
eligibility, fee/compute/loaded consumption or founder identity/control is proved.

Freeze binds 52 source/tool/evidence inputs, including 17 direct action source pins
and unchanged reused helpers. A draft guard key (`packets` versus actual `plan`)
was corrected before first execution; no failed Node run or production change.
The retained guard denies sockets, caps Node heap to 512 MiB and sampled group RSS+swap
2.5 GiB, and preserves 2 GiB free disk. Bank-specific virtual-AS cap is omitted for
V8 and receipts accurately record inherited unlimited AS. Maximum sampled RSS+swap
was 236,793,856 bytes; minimum free was 20,072,071,168 bytes. Prior initializer 11,
funding 7 and production host/Bank/SBF/runtime results remain retained, not rerun.
No new RPC/build/install/cleanup occurred.

Evidence: `/tmp/piv1-m5-intake-plan-20261019-a`; guarded outputs in
`/tmp/piv1-bank-smoke-intake-plan-{tests,template,example}-20261019-a`.
The suite diagnostic points to the three retained public-only manifests/reports.
Git and `publication.json` identify reviewed integration-only publication.

Next prepare readable current-state inspection and protected principal-staking/
remaining operator plans, then an exact funded/artifact workflow. Live identities,
source/Config/custody/protocol facts, fee/rent/funding, resource sufficiency and the
complete founder workflow remain OPEN. No founder economic decision is needed for
independent preparation. Public package NOT READY, deployment NOT AUTHORIZED;
M4 remains technically incomplete with its stopped criterion open/unwaived. No
stopped drafts were inspected/resumed, secrets/keys/signing accessed, live transaction,
fund movement, authority transfer or new main-publication authority occurred.

## Retained conditional initializer rent checkpoint

Baseline integration `9418d9c9b7eb8e3da10dd8b7a07dff53e717bd36`; main remains
`a524cc6`. [Client usage](../clients/cli/README.md) now includes an additive offline
funding report. This is a **conditional zero-prefund rent ledger**, not an actual
funding shortfall, complete wallet requirement, peak liquidity or deployment budget.
Existing initializer code/tests/template, all production/economics, Rust, dependencies,
toolchains and ELFs remain unchanged. One bounded Sol/High writer and separate
Sol/Extra High critical review were used; root model/speed remain unexposed.

Root made exactly three HTTPS batches / eighteen logical READ-ONLY requests to
`https://api.testnet.solana.com`: genesis/slot before, fourteen
[getMinimumBalanceForRentExemption](https://solana.com/docs/rpc/http/getminimumbalanceforrentexemption)
queries at finalized, then genesis/slot after. Both genesis values match official
Testnet. Slots bracket 454,098,679–454,098,682; capture completed at
2026-10-18T22:54:04.991751+00:00. Individual rent responses have no contextual slot:
these observations are not a single-bank account snapshot or perpetual quote.
No balance/control/fee/identity or state-changing RPC was performed.

The committed example preserves exact observation bytes, SHA-256
`537e702858a6a8f5d5c6c88877ba8300b690d3f2c6d4bd078b56d8636d09a0b3`.
The CLI does not contact RPC or authenticate the supplied endpoint/genesis/price
metadata. Its input digest is provenance only; root and separate review bind the
retained requests/responses/receipts. The optional prepared normalized adapter was
not used. Null observations produce `INCOMPLETE` and no monetary scenarios. Complete
observations produce only `CONDITIONAL_RENT_ONLY`; incomplete identities retain two
symbolic scenarios, without invented addresses or packet signature counts. Complete
manifests reuse the unchanged planner's actual phases/signature unions.

The named scenario assumes all sixteen valid virgin PIV targets and supported new
ALT/Squads resource addresses have no prefunds/donations. Actual target shortfall
requires fresh owner/data/balance checks and each ORIGINAL prefund separately.
Token-target excess is an internal PendingSol transfer, not a payer refund, and
cannot replace PendingSol's original rent shortfall. Provisioned recipients and
existing multisig/program/governance prerequisites remain separate OPEN items.

| Conditional rent quantity, lamports | Shared manager/referrer, 34 roles | Distinct receivers, 35 roles |
| --- | ---: | ---: |
| Sixteen PIV target floors | 25,384,760 | 25,384,760 |
| ALT final 568-byte floor | 3,535,680 | 3,535,680 |
| Temporary buffer funded, later refunded to creator | 8,569,960 | 8,737,600 |
| Stored transaction retained | 8,514,080 | 8,681,720 |
| Proposal retained | 3,931,920 | 3,931,920 |
| Gross rent-payer outflow, before fees | 49,936,400 | 50,271,680 |
| Aggregate account rent held after creator refund | 41,366,440 | 41,534,080 |
| Required payer post-initializer residual, not spent rent | 650,240 | 650,240 |

ALT create funds 934,720 at 56 bytes; extend adds only 2,600,960. Stored creation
funds 1,168,400 at 102 bytes, then only the delta to final size. The buffer closes
AFTER stored funding and refunds its entire actual balance to the creator. Under
the supported guards creator and rent payer differ; that refund cannot reduce
payer gross outflow. Stored/proposal/ALT/PIV balances remain held through this
plan, not burned or automatically reclaimed. Recipient vaults each require an
existing 650,240 floor, with actual balance/shortfall/provisioning OPEN and excluded
from these packet outflows. The separate payer residual is not another cost.

Root actually executed each fixed phase once:

```sh
python3 /tmp/piv1-m5-funding-plan-20261019-a/node-driver.py tests
python3 /tmp/piv1-m5-funding-plan-20261019-a/node-driver.py template
python3 /tmp/piv1-m5-funding-plan-20261019-a/node-driver.py observed
```

All seven focused groups pass on first execution, zero failures/skips; guarded
suite elapsed 4.159 seconds. Tests use independent literal irregular prices,
32 conservation cases for both variants, growth/refund/residual flows, eight
complete manifests with actual one/two/three signer unions and fee-payer aliases,
partial-known alias regression, integer/schema/missing-input domains and actual
CLI subprocesses. The template has null prices/variable metadata; the historical
CLI matches root's separately derived integer oracle and remains incomplete for
42 identity/choice fields, with null actual fees/funding/peak/deployment costs
and readiness false.
Draft symbolic-role, unrelated-null test-oracle and template-wrapper corrections
preceded execution; no failed Node run or production defect is claimed.

The source/tool/evidence freeze binds 51 inputs. The same reviewed socket denial,
512-MiB Node heap, sampled 2.5-GiB group RSS+swap and 2-GiB disk reserve apply;
Bank-specific virtual-AS cap is omitted for V8, accurately recorded as inherited
unlimited AS. Sampled maximum RSS+swap was 262,615,040 bytes; minimum free space
20,076,953,600 bytes. New execution is only public read-only observations and
focused Node host validation. Old initializer11/production host/Bank/SBF/runtime
results are retained, not rerun. No build/install/cleanup occurred.

Evidence: `/tmp/piv1-m5-funding-plan-20261019-a` contains raw RPC records,
source/design/static/evidence/final reviews, independent arithmetic and publication
receipts. Raw Node outputs are in
`/tmp/piv1-bank-smoke-initializer-funding-{tests,template,observed}-20261019-a`.
Git and `publication.json` identify reviewed integration-only publication.

Next prepare contribution/bootstrap client commands and readable state/error
inspection, then remaining operators and an exact funded/artifact workflow.
Actual public identities/control, account validity/prefunds, fees/prioritization,
wallet requirements, peak/operational liquidity, recipient/governance provision
and deployment/ProgramData costs remain OPEN. No founder decision is needed for
independent command preparation. Public package NOT READY, deployment NOT
AUTHORIZED; M4 technically incomplete with its stopped criterion open/unwaived.
No stopped drafts were inspected/resumed; no secrets, keys/signing, live transaction,
fund movement, authority transfer or new main-publication authority occurred.

## Retained public-input offline initializer checkpoint

Baseline integration `a0f785b6dc321c8aca776fb4cdf522aae3176f41`; main remains
`a524cc6`. [The client](../clients/cli/README.md) now accepts an explicit public
manifest and derives native315 initialization, Squads buffer/create/approvals/
execute and ALT create/extend instruction drafts. Four bounded client files are
new/updated; production, economics, Rust, historical transport tests/tools,
dependencies, toolchains and ELFs are unchanged. Routine Sol/High writer and
separate critical Sol/Extra High review were explicit; root model/speed are not
exposed, and settings were unchanged.

Every identity and initializer choice is declared, including fee/rent payers,
multisig/create key, creator/executor/four approvers, six guardians/permutation,
indices, recipients, pause and anchor. Null/missing fields return `INCOMPLETE`
with no packets; malformed/unsupported declarations reject. No synthetic fallback
or founder choice is silently selected. A complete declaration yields only
`OFFLINE_PLAN_ONLY`, never public/deployment readiness. All ordinary outer signers
must be on curve in this client profile; that is not proof of wallet control or a
new protocol restriction on program-mediated governance.

The planner validates full-width decimal u64/i64 fields, raw-byte member ordering,
canonical PDAs and the sole supported manager/referrer collapse (34/35 roles).
The stored compact message remains lookup-free. Actual compiled signer/privilege
unions determine every packet's one/two/three zero-signature slots and adaptive
upload chunk lengths. Each emitted packet undergoes SDK roundtrip/decompilation
and the 1,232-byte bound. Final v0 requests 1.4M CU, 256-KiB heap and 8-MiB loaded
bytes based on retained initializer B; these are requests, not measured public
consumption or a budget guarantee for arbitrary declarations.

ALT create/extend plans contain exactly sixteen derived targets. The lookup
account used for compilation is explicitly hypothetical: current owner, authority,
active state, exact content, successful creation/extension, SlotHashes eligibility
and execution strictly after the actual final extension slot remain OPEN.
Declared recent-slot/reference-slot coherence uses BigInt; no freshness is inferred.
Each future signed step requires independently refreshed blockhash/validity and
account context, especially after approvals/timelock. No bundle is send-ready.

Root actually executed these three frozen commands, each once:

```sh
python3 /tmp/piv1-m5-public-plan-20261019-a/node-driver.py tests
python3 /tmp/piv1-m5-public-plan-20261019-a/node-driver.py template
python3 /tmp/piv1-m5-public-plan-20261019-a/node-driver.py example
```

First suite: 11/11 groups pass, zero failures/skips, 17.817 seconds. It covers
18 full literal ABI/role plans, all 720 permutation codecs, integer boundaries,
buffer commitments/reassembly, ALT encoding, signer unions, wire reconstruction,
1,232/1,233-byte adjacent behavior, strict schema/alias negatives and an actual
complete synthetic public-manifest CLI subprocess. Standalone template matches
the committed nullable example; the example reports 42 missing fields and no
packets. Complete synthetic output remains readiness false. These are host-wire
and input-validation results, not application-runtime or cluster proofs.

Source/command freeze binds 33 inputs, including 22 direct client source/SDK pins,
pinned Node24.19.0/web3.js1.98.4 and unchanged guard helper. Socket denial,
512-MiB Node heap, sampled 2.5-GiB group RSS+swap and 2-GiB disk reserve apply.
The Bank-specific virtual-address cap is omitted for V8's reservation; actual
inherited RLIMIT_AS is unlimited and receipts explicitly record that distinction.
A draft hardcoded guard-receipt field was corrected before execution; no failed
Node/test run occurred. Maximum sampled RSS+swap was 273,764,352 bytes; minimum
free space across phases was 20,082,081,792 bytes. No Bank/SBF/build/RPC or old
suite execution occurred. Prior real-image/runtime evidence is retained.

Evidence: `/tmp/piv1-m5-public-plan-20261019-a`, including writer/static/evidence/
final review, freeze and root receipts. Raw outputs and guarded receipts are in
`/tmp/piv1-bank-smoke-public-planner-{tests,template,example}-20261019-a`.
Git and `publication.json` identify reviewed integration publication.

Actual public identities/control, governance/account observations, ALT lifecycle,
current fees/rent/funding, exact artifact selection, other operator commands and
founder workflow remain OPEN. Account byte sizes are planning values, not rent
quotes. Next prepare remaining operator workflow/state inspection/funding without
inventing identities or economic choices; no new founder decision is required for
independent preparation. Public package NOT READY, deployment NOT AUTHORIZED,
main publication not authorized. M4 remains technically incomplete with the stopped
criterion open/unwaived; its operation/drafts were not inspected or resumed.
No keys/signing, live transaction, fund movement or authority transfer occurred.

## Retained Testnet pool-call compatibility checkpoint

Baseline integration `3c3589d6f55bde7b497e9f5a8299bc4c194a900c`; main stays
`a524cc6`. The existing exact `positive_withdrawal_fee_production_lifecycle`
profile and all Rust/oracles/dependencies/ELFs are unchanged. Root reused reviewed
AB executable `96407683b96a24ca18928ff7854736db82c98f2048ca8a62cd9b9fc0107d2ae0`
after nine current Cargo/fixture/vendor inputs matched both its original profile
and source freeze, its original build/log/review/locked graph matched, and a fresh
copy was byte-verified. This turn ran NO compiler, SBF build or dependency build.
The separate reviewer approved source-bound reuse; this is retained build evidence,
not a new reproducible build. Routine writer Sol/High and critical review
Sol/Extra High were explicit; root model/speed remain unexposed, settings unchanged.

Root actually ran once:

```sh
python3 /tmp/piv1-m5-real-pool-20261018-a/driver.py reuse
python3 /tmp/piv1-m5-real-pool-20261018-a/driver.py run
```

The run is the exact named test, with five `PIV_LIFECYCLE_*` image path/hash/length
triples, scrubbed environment, no network sockets, one thread and unchanged resource
guards. It passed on its FIRST execution in 52.029 seconds with empty stderr and
no guard stop. Initial draft compiler branches were preserved unexecuted; the final
driver supports reuse/run only. One draft receipt-prose correction clarified
"no dependency-build execution"; runtime library code naturally executes.

| Loaded role | Exact executed image SHA-256 | Bytes |
| --- | --- | ---: |
| PIV | `e29735e1467da98ecddb1eea9fd25abaffa4a450bd899ddc137f7d8c0da35b99` | 894,552 |
| Synthetic governance caller | `073a31d24c7b6b3cd1ba0b72ae677afb541d9287b80529167f0b173c14f3d4bc` | 63,424 |
| Retained observed Testnet pool | `6c0cb7ef5da80a23c3ad85d7fec566f3cc17b815d428daec84bb916441a2a143` | 1,106,080 |
| Retained observed Testnet Token | `8190d3f7ceb6cb7a7a8d8924bff89f9f611e15ce1f806f2b6237f3311a98f697` | 108,600 |
| Retained observed Stake, equal to pinned 5.1.0 | `3d2d39c596ce8be2d47816b4ee5db9fc759d80fde54b08c930ad0b6daed64c2c` | 212,056 |

The raw Rust fixture/summary still prints historical `Token8.0.0`/`SPLpool2.0.3`
recipe strings. They are NOT deployed-version/source attestations. The wrapper's
explicit five artifact identities, actual ProgramData payloads and independent
review identify execution; raw output was not rewritten or relabelled in place.
No new RPC was performed. Prior public observations, initializer B and host/SBF/
caller/runner suites remain retained evidence in their original scopes.

Actual production initialization/intake/bootstrap precede protected deposit
variant 25: 100 SOL becomes 100,000,000,000 token units at the fixture's initial
1:1 ratio. Declared external reserve transfers and actual maintenance produce
yield. Prepared distribution freezes fee1/1000 and its target. Two protected
variant24 calls execute the following independently checked outcomes:

| Leg | Input tokens | Manager fee | Burned tokens | Delegated lamports |
| --- | ---: | ---: | ---: | ---: |
| 0 | 4,587,921,256 | 4,587,922 | 4,583,333,334 | 5,000,000,000 |
| 1 | 3,303,328,743 | 3,303,329 | 3,300,025,414 | 3,600,027,724 |

Separate ceiling fees total 7,891,251 token units, one above the aggregate ceiling;
actual manager credit, burns/mint supply, pool accounting and source/destination
Stake bytes match. Real Stake rejects premature finalization, then permits
out-of-order finalization at local epoch14. Original rents return exactly:
4,565,760 Stake plus 5,442,720 metadata; four temporary accounts close.

Net available 8,700,027,724 lamports pays HTFP 6,376,417,834 and team 2,107,460,131,
credits earned KIF liability 216,149,757, and retains two net dust lamports. Fixed compound remains
2,109,545,454. After pending integration, Idle HWM equals historical value
121,955,000,003 with historical SOL 200,000,002 and tokens 111,608,750,001; pending
and next-cycle yield are zero. KIF liability is earned, not claimed by this profile.
The existing explicit three-lamport reserve appreciation remains necessary in this
fixture; it is not natural yield, a reserve policy or an altered HWM guard.

Root's streaming audit and a separate complete-map/CPI/event audit verify 7,890
account records / 108 full maps, 55 transaction/commit pairs (50 distinct plus five
replays) and 118 CPIs. There are 32 successful maps and 18 fee-only failures;
five identical replays add no fee. All 19 real PIV event payloads are checked, including
successful inner attempts whose outer transaction later fails. These are unchanged
historical outer-failure checks, not the stopped internal-callee-failure criterion.
Root's first analysis parser incorrectly selected every large account as immutable,
including epoch sysvars. Its preserved assertion failure was corrected to the exact
five programs and their ProgramData references; monetary/state checks were retained
and the corrected parser passed. No application defect or failed runtime occurred.

Observed CU: initializer/max 1,064,915; protected deposit 191,702; two withdrawals
559,470 / 556,777. These use the existing 1.4M-minus-nonce limit / 256-KiB heap.
The unsigned legacy maximum is still 1,544 bytes, not a public packet. Loaded-byte
statistics are not emitted by this historical harness; no new 8-MiB/full-list or
complete governance budget proof is inferred. The separate real-Squads initializer
B packet/resource evidence retains its own 963-byte/730,009-byte-list scope.

Fixture boundaries remain: synthetic pool/Stake genesis and a 155-byte two-entry validator list,
synthetic caller at the Squads ID (not the real Squads image), synthetic Rent 6,960,
Clock/epoch warps, three canonical capability features active at local slot zero,
inactive genesis EpochRewards, unsigned signatures and local ProgramData slot-zero
headers (external authorities locally None). The synthetic zero SOL deposit fee is encoded 0/1; the separately observed public
encoding was 0/0. This run proves the bounded zero-fee behavior, not byte-identical
public configuration. The 100-SOL donation and three-lamport appreciation are declared
stimuli. This is compatibility of exact retained images
on the supported economic path, not execution on a coherent current public pool,
source equivalence, real approval/recipient control or combined public usability.
Do not rerun every old profile or invent extra image-equality prerequisites.

Evidence: `/tmp/piv1-m5-real-pool-20261018-a`; raw trace and guarded receipt:
`/tmp/piv1-bank-smoke-real-pool-run-20261018-a`. Raw SHA-256 is
`f6eb3fed9c2fe21016025f50e2595275684212648e55b26001c0d97d50693058`
(558,238,168 bytes). A streamed non-account projection feeds the unchanged existing
summary oracles; it does not substitute for the separate full-map audit. Frozen
inputs: 23 files, five artifacts, ten tools, four evidence inputs and ten tool aliases.
Runtime sampled peak group RSS+swap was 61,464,576 bytes, minimum free space
20,093,988,864 bytes; preserve the 2-GiB reserve and all prior evidence.

Next prepare usable public transaction/ALT lifecycle, identities, budget and founder
workflow, followed by independent readiness review including M4's open reservation.
Current pool fields/availability and image identities need freshness checks before
an exact live package. No new economic decision is required for independent
preparation. Public package NOT READY; deployment NOT AUTHORIZED; reviewed integration
publication only. No cleanup/install, keys/signing, public transaction, funds or
authority transfer, new main authority or founder acceptance is inferred. The stopped
M4 operation/drafts were not inspected, resumed or rerouted; its criterion is unwaived.

## Retained real Squads initializer compatibility checkpoint

Baseline integration `ffe054d5702ad143ce55a1a7a9ecf8e234274c68`; main remains
`a524cc6`. The new [isolated Bank profile](../validation/genesis-bank-runtime/tests/real_squads_initializer.rs)
uses unchanged native315 PIV ELF plus exact Squads/Token images extracted from the
retained Testnet ProgramData observations. Pool image/account data are present;
no pool or Stake instruction executes. Historical genesis/lifecycle sources are
unchanged; only the production test module declaration is appended. No Cargo,
production, economic, allocator, dependency, toolchain or ELF change occurred.

Root actually executed two targeted `genesis_production --no-run` builds with
`--locked --offline --jobs 1`, both with no compiler diagnostics, and two exact
`real_squads_initializer::real_squads_native_initializer_success_only` runs.
Attempt A failed: eight setup/approval entries succeeded, then Token reported
`unsupported BPF instruction` after 49 CU. The assertion stopped before a final
CPI/account dump: no final rollback map, Executed state or complete initializer
success is inferred from A. Its original source, build, driver and raw trace are
preserved under the evidence root and the separate `...-20261018-a` build/run dirs.

Read-only ELF/cached-runtime inspection found the Token `sol_get_sysvar` import
and Bank 4.2.0's feature-gated registration. The exact failing PC was not traced.
B changes only the synthetic capability profile from empty to canonical
`CLCoTADvV64PSrnR6QXty6Fwrt9Xc6EdxSJE4wLRePjq`, using the existing genesis helper.
It asserts the exact singleton feature, canonical Feature-owned account and enabled
syscall at root/child/every step; full maps preserve that account. No image or success
oracle was substituted. One new READ-ONLY official Testnet batch (two RPC requests:
genesis and feature account) verifies the cluster and observes activation slot
316,748,256 at context slot 454,084,266. This is separate from the thirteen retained
initial RPC requests and from local synthetic activation at slot zero.

Corrected B passed its first build and first exact run (2.847 / 8.270 seconds),
with empty runtime stderr. Its nine distinct unsigned successful entries create
and extend a 1,447-byte buffer, create transaction index 19, create an Active
proposal, register four approvals, and execute actual Squads -> PIV -> System/Token.
No buffer/proposal/transaction or PIV target is injected into genesis. Root's
independent raw parser verifies 461 account records / ten complete maps, nine
transaction/commit pairs, 45 CPI records, packet bytes, exact account deltas,
approval bytes, original rent, fee/refund totals and all sixteen created targets.
Final `Executed` is directly reread; `Approved` during CPI is an inference from
the unchanged PIV guard and pinned source, without a mid-CPI account dump.

| Actual B packet | Bytes | CU |
| --- | ---: | ---: |
| Buffer create / extend | 1,215 / 992 | 17,384 / 12,053 |
| Transaction from buffer / proposal create | 421 / 383 | 66,948 / 14,466 |
| Four approvals | 341 each | 9,726 / 10,208 / 10,707 / 11,331 |
| Full v0 governance execute, three budget instructions | 963 | 1,016,484 |

Every packet includes two 64-byte zero signature slots. Setup uses the default
200,000-CU / 32-KiB heap profile; final execute selects 1,400,000 CU, 262,144-byte
heap and 8,388,608 loaded-byte ceiling. Actual execute loads 3,833,006 bytes,
including the 730,009-byte list. This is a successful selected budget, not its
minimum or whole-Testnet compatibility. The final 963-byte prefix includes the
loaded-data limit in addition to the two illustrative budget instructions in the
older 955-byte host case. Only the outer message uses the seeded warmed ALT.

Synthetic payer 222 pays 90,000 transaction fees plus 11,741,520 buffer rent,
11,664,960 stored-transaction rent, 5,387,040 proposal rent and 34,779,120 PIV rent:
gross debit 63,662,640 lamports. Creator 91 receives the full buffer-rent refund;
that is not a refund to payer 222. These use synthetic Rent 6,960 and must not be
quoted as the current public funding envelope. The final transaction bytes remain
unchanged while its proposal becomes Executed. No effect-once/replay test is claimed.

Multisig 4/6, timelock zero, PIV authority/vault and warmed ALT are explicit genesis
inputs. Clock, local deployment-slot-zero loader headers and Rent are synthetic;
external upgrade-authority bytes remain observed, PIV authority is synthetic.
Retained pool/mint/list/reserve/manager observations span different old slots.
There is no real signing, wallet/recipient-control proof, multisig/ALT creation,
nonzero-timelock passage, pool/Stake CPI, source-equality attestation or live operation.
Compatibility of these Squads/Token images is established only for this initializer
path; other required external calls remain open. M4's stopped criterion is unchanged.

Evidence root: `/tmp/piv1-m5-real-squads-20261018-a`, including both source freezes,
original-attempt preservation, guarded commands/logs, image/input pins, new read-only
RPC, root raw audit and separate review receipts. B executable SHA-256:
`2224c4d734b0428a3399e520e081e4025401828249eceb1455b32d5588a72966`.
Both builds reuse the existing locked dependency cache; no full fresh reproducible
build is claimed. No old host/SBF/caller/runner/AB suite was rerun. New ordinary
primary-source reads are pinned in `upstream/sources.json`; no installation,
cleanup, secrets, keys/signing, public transaction/deployment or authority transfer
occurred. Python's generated helper bytecode was preserved outside Git in the evidence
root. Routine writer Sol/High and critical reviewer Sol/Extra High were explicit;
root model/actual speed remain unexposed and no global configuration changed.
Next establish supported pool-call compatibility, then identities/funding/workflow
and independent release review. Public package NOT READY; no founder decision is
needed for that independent preparation. Deployment still requires an exact reviewed
package and separate explicit authorization. Reviewed integration publication only.

## Retained native initializer transport checkpoint

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
| Deployed external artifacts | Observed Stake equals pinned 5.1.0; retained Squads/Token pass initializer B, and actual retained pool/Token pass unchanged protected25/24 fee lifecycle with mint/burn/transfers/maintenance. | Bounded supported-call compatibility is established. Refresh identities/current fields for the exact package and assess provenance/residual risks; no source equality, whole-cluster equivalence or automatic dependency change follows. | Local supported-call compatibility CLOSED in scope; exact public package remains OPEN. |
| Deposit and withdrawal regimes | Observed SOL deposit fee 0/0, no SOL deposit authority; stake withdrawal fee 1/1000, no pending change or preferred withdrawal vote. Production requires permissionless zero-fee SOL deposits and protected SPL variants 25/24. [Deposit, lines 35–47](../programs/piv1/src/integrations/jito_deposit.rs#L35), [withdrawal, lines 19–34](../programs/piv1/src/integrations/jito_withdrawal.rs#L19). | Actual retained images execute the synthetic zero-deposit-fee/1/1000-withdrawal-fee path with exact postconditions. Public epoch, quote, preferences/authorities and source availability still need fresh construction/execution checks. Do not change economics to fit a fixture. | Bounded fee semantics established; public availability remains dynamic. |
| Supported withdrawal source and rent | List has 864 records in 730,009 allocated bytes. Index 75 candidate matches PDA, vote, Stake owner, authorities, list balance, flags zero and nondeactivation. Runtime minimum delegation observation is 1,000,000,000 lamports. [Source guards, lines 86–133](../programs/piv1/src/integrations/jito_withdrawal_preparation.rs#L86). | Reobserve source priority, capacity, current rent/funding and dynamic token minimum at construction/execution. Only Active validator sources are supported; no transient/reserve/removal/preferred fallback. Candidate is not a permanent selected source or successful CPI quote. | Availability condition remains dynamic. |
| Wire format and signatures | Initializer 11 groups, contribution/bootstrap 7 and protected principal 7 are validated in their own scopes. Three retained intake/bootstrap CLI packets are 504/474/828 bytes; two new principal CLI packets are 877/910 bytes with one zero signature and 19/20 nonsigner native roles. Eight new distribution CLI packets range 861..1,195 bytes; source roles/current-frozen tuples and requested/global privileges agree. Retained B executes its 963-byte initializer; stored message is lookup-free. | Actual identities/signatures/state/resources, initializer ALT lifecycle and remaining operator composition stay unproved; all current client outputs are offline-only. v1 is not required by these fitting profiles. | Local supported action encoding progresses; exact public usability remains OPEN. |
| Complete governance composition/resources | B actually executes upload/create/approve/execute with retained deployed Squads/Token, native315 PIV and the 730,009-byte list: full execute 963 bytes / 1,016,484 CU / 3,833,006 loaded bytes. | B closes the bounded initializer composition/resource gap. Exact public identities, supported other actions, real transport/signatures and ALT lifecycle remain unproved. | Local initializer composition closed; exact public package remains open. |
| Real governance consumption and recipients | B creates the proposal through actual retained Squads, records four approvals and directly reads final Executed; older synthetic-caller evidence keeps its non-consumption limit. [Verifier limitation, lines 13–17](../programs/piv1/src/squads_execution.rs#L13). | Identify actual multisig, six guardians, wallet control and supported funded recipient vaults; replay/effect-once and multisig/ALT creation are not tested by B. Recipient identity does not establish exclusive 4/6 spending. | Yes; no selected public multisig/control or complete public governance workflow is established. |
| Guardian updates, recovery and founder workflow | Offline Config/distribution inspection now passes targeted byte/identity tests and four retained-snapshot field comparisons; it does not prove live custody, validity or eligibility. AA2 covers local one-seat synchronization/old earned claims; Z covers frozen/current recipients. Authority is current ProgramData, not historical multisig continuity. Governed recovery exit is unimplemented. | Preserve one-seat/mismatch limitations; complete the remaining non-Rust operate/readiness/claim/error workflow and exact stop conditions. Natural yield, the ten-day cadence and dynamic minimum delegation govern attainable delayed tests; no fabricated appreciation or shortened Clock. Public cooldown is not promised to equal local epoch 14. RecoveryRequired means stop and report, not an invented exit. | Exact usable package/workflow remains open. |
| Exact package identities and budget | Public manifest/initializer/ALT commands and reviewed conditional rent ledger now exist. Fourteen historical Testnet rent observations give zero-prefund payer outflow49,936,400/50,271,680, with creator refund/residual separate. No selected public identities or actual full funding quote. | Fees, account prefunds/validity, control, provisioning, wallet/peak needs, operational and loader/deployment costs stay OPEN. Prepare remaining distribution/claim/workflow commands independently; request only genuinely blocking public identity/governance decisions. Never request/read private keys. | Yes; rent preparation is scoped progress, not deployment readiness. |

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
