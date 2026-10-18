# Task 2.51 — Real local production lifecycle

Status: **IN PROGRESS / P-023 POSITIVE WITHDRAWAL-FEE RUNTIME GAP IDENTIFIED** (2026-10-18 UTC).

The subsequent P-023 closure review below adds no implementation, test, build or
runtime execution. Runtime W is the latest validated execution. It closes the bounded later principal
compounding gap identified in the earlier October 18 source review. Prior attempts
and reviews retain their actual source/evidence scope; full M4 remains incomplete.

## Critical path and scope

D-030 requires complete local production instruction execution before an exact
Testnet package. Task 2.50 supplies the final economic integration boundary but
its host models/static ELF do not prove nested runtime custody, rollback or heap/CU.
The previous Bank initializer harness uses a restricted synthetic Token program,
never executes stake-pool, and has no native Stake builtin. This is a concrete
M3/M4 dependency, not a validation-only scheduling detour.

Clean actual takeover: jerem, integration `370660a1758f8ab3000db0997968780cf7c5cfc6`,
main `1054ff3751ca4169eab7e46d77452634b11fc695`, protected Task 2.3 `3677fee`.
Root captured 369 tracked files, 128 matching prior source inputs, 773 preserved
records and 48,502,198,272 available bytes before delegated edits.
Evidence root: `/tmp/piv1-t251-pilot-review`.

One writer owns the isolated upstream-program resolution workspace and new Bank
lifecycle fixture/test/README/target. A separate reviewer checks requirements,
source/oracles, commands and actual evidence. Root owns shared guidance, runner,
pins, all executions and normal integration-only publication. Economics, persisted
production layouts and production dependencies are unchanged.

## Exact program prerequisites

`validation/pinned-token-program` selects published Token 8.0.0; the separate
`validation/pinned-pool-programs` selects SPL stake-pool 2.0.3. Each builds its
upstream cdylib with a genuine entrypoint, without restricted processor shims.
Root generated the isolated lock offline; all 242 registry archive checksums and
7,630 extracted source files match their immutable archives. No dependency download
or compilation occurred during this resolution. The earlier apparent missing
source paths were an inspection-pattern error, corrected before dependency action.

Stake 5.1.0 is the real embedded ELF from already-locked
`solana-program-binaries 4.2.0`: 212,056 bytes, SHA-256
`3d2d39c596ce8be2d47816b4ee5db9fc759d80fde54b08c930ad0b6daed64c2c`.
Root and reviewer checked archive provenance and ELF64/ET_DYN/machine 263/flags zero.
It must be loaded explicitly; this Bank has no Stake native builtin.
The exact Task 2.50 PIV ELF remains the starting production artifact.

## Required execution and boundaries

Start with absent PIV state and invoke its actual initializer, then SOL/Jito intake,
bootstrap, protected zero-fee SOL staking, authenticated real pool update, two
protected delayed legs and Stake deactivation, active-withdrawal rejection, epoch/
StakeHistory readiness, finalization with original rent recovery, settlement,
pending integration and earned KIF claim. Compare full state/custody/fee records,
actual late/internal failure rollback, replay and distinct retry. Record actual CU,
heap requests and transaction wire-size limits. State transitions and token effects
must come from real production instructions, not injected host callbacks.

External pool/validator Stake genesis, voter references and governance caller may be explicitly
synthetic local fixtures. A documented reward stimulus may donate to reserve then
invoke real SPL maintenance; it does not prove validator-consensus rewards or live
Jito provenance. Do not rewrite PIV economic state to manufacture a successful
cycle. Epoch advancement must update real Bank history; a frozen warp result needs
a child before transactions and Clock behavior must be explicit.

The first upstream compilation attempt returned Cargo success but strict ELF
inspection rejected Token: pool dependency feature unification enabled
`no-entrypoint`, producing a 4,640-byte ELF with entry zero (SHA-256
`569126fb931145f0aaffb3f484deaf065bea8aef829135d71fec551139eeb43b`). Pool compilation
was not reached. Preserve `/tmp/piv1-bank-smoke-lifecycle-programs-t251-a` and
`program-attempt-a-sources`/`program-attempt-a-preserved.json` under the evidence
root (75 records). All 773 historical records remained unchanged. Separate Token
and pool workspaces/targets remove this demonstrated feature-unification blocker;
the entrypoint gate remains strict. At that point no Bank lifecycle had run.

Complete lifecycle, applicable production/adapter
profiles, authentic Squads, signatures/transport, live readiness and founder
acceptance remain open until separately proved. No secrets, keys, signing, live
RPC, deployment, funds or authority transfer. Preserve prior failed evidence and
all four recovery archives; no cleanup/install is planned.

## Reviewed harness and command preparation

Separate review approved the complete account oracles, literal production ABI,
exact Stake rejection classification, rollback/retained fees/replay and distinct
retry structure for execution. The runner uses the existing bounded Bank cache,
locked/offline unchanged dependency resolve, fresh AccountsDB, copied approved
binary, exact five program artifacts and network-denied resource guard. Six root
runner regressions passed, rejecting incomplete/duplicate/overclaimed completion
and missing/substituted program artifacts. These are runner checks, not lifecycle
execution evidence.

Attempt B has independent Token/pool manifests and target directories, with
resolved `no-entrypoint` rejected before compilation. Token resolves 109 packages;
pool resolves 242; their union is exactly the original 242 archive-verified
packages. Separate exact command review is recorded in
`reviewer-program-build-plan-b.json`; full Bank source review is in
`reviewer-bank-source.json`. Two outer-wrapper pre-execution refusals (receipt not
yet present, then different receipt field schema) launched no build; root retained
the earlier wrapper and adapted only field lookup before the actual attempt.

Attempt B compiled Token successfully without diagnostics (44.62 seconds), then
the inherited output-name audit refused the known `solana-sysvar-id` fingerprint
because its exception was rooted at `target/`, not the isolated `target-token/`.
No pool build or ELF acceptance was reached. Preserve the complete B directory,
its raw Token ELF and 75 source/evidence records in `program-attempt-b-*`. The
local audit adapter maps only the exact two isolated target prefixes into the
unchanged inherited fingerprint predicate; it keeps metadata-only scanning and
symlink/type/name rejection. Nine root runner tests passed, including exact-path
acceptance and malformed/nested/other-path/directory/symlink rejection. Attempt C
received separate exact review before execution; all prior evidence is retained.

## Upstream executable result

Attempt C passed strict offline compilation, both diagnostic streams, output-name
audit, ELF entrypoint validation and preservation. Token is 127,920 bytes, SHA-256
`7cfba90fd41e64650c361cc5177bc2ba2e03f078f7665ae2567a1c52603ff8f5`; pool is 421,712
bytes, SHA-256 `314cefab54443c66f46d270f0c90ddd0dacc0fda7b2bbe10215391742b9ecf30`.
Both are ELF64/ET_DYN/machine 263/flags zero with genuine executable entrypoints.
Evidence: `/tmp/piv1-bank-smoke-lifecycle-programs-t251-c` (68 retained diagnostic
logs). The original 773 historical records remain identical. These are real
upstream binaries; static acceptance is not yet runtime execution.

The Bank attempt A freeze contains 273 source inputs, 997 preserved records and
five exact artifacts; initial source bytes are in `bank-attempt-a-sources`. The
existing Task 2.50 PIV executable is reused because its 128 source inputs remain
unchanged. No old host suite or PIV SBF rebuild is claimed in this task.

Bank build A passed on its first execution without diagnostics, using the existing
cache and unchanged locked dependency/features graph. Actual build took 487.30 s;
resource sampling observed 935,632,896 bytes group RSS+swap and at least
46,802,763,776 free bytes (sampling, not exact peaks). All 273 source inputs and
997 preserved records matched after execution. Copied executable SHA-256:
`bf8e857d34aad1525b33fc08a7d75a6fe69856ec296a91962ce11bc1e43dfaf6`.
Build evidence: `/tmp/piv1-bank-smoke-production-lifecycle-build-t251-a`. Separate exact binary review preceded runtime A; compilation alone did not close M4.

## First actual Bank execution

Runtime A failed at production withdrawal preparation. The actual nested Stake
`GetMinimumDelegation` instruction (`0d000000`, no accounts) returned
`UnsupportedSysvar` after 340 CU; the complete PIV instruction used 132,673 CU.
Before that failure, initializer, operational funding, SOL/Jito intake, bootstrap,
real pool deposit/update, heartbeat and the pending contribution all passed their
whole-account literal oracles. Root independently checked 1,710 full account
records across 26 snapshots, including exact non-fee rollback and fee-free replay
for the intake and pool-deposit late outer failures. Largest observed transaction
was 1,544 bytes and initializer used 1,067,599 CU. Unsigned Bank entry does not prove
packet transport or signature verification. Later lifecycle stages were not reached.

Retain `/tmp/piv1-bank-smoke-production-lifecycle-run-t251-a`,
`bank-attempt-a-preserved.json` (15 build/runtime files), all initial source bytes
and `root-runtime-a-review.json`. All 997 prior records stayed unchanged.

Independent source diagnosis identified absent EpochRewards in the synthetic
genesis: Stake queries it before instruction dispatch; Agave syscall propagation
can abort the VM on a missing cache entry before the guest fallback can run. The
pinned runtime already exposes `add_genesis_epoch_rewards_account`, which creates
the canonical rent-backed, Sysvar-owned, inactive 81-byte account. The bounded
fixture correction uses that helper, with explicit initial-account assertions;
it does not activate features, inject CPI results or change production economics.

## Corrected runtime B and resumable production blocker

Root applied the reviewed genesis correction after bounded delegated diagnosis;
only the Bank test and its README changed. The first attempted patch rejected an
incorrect README heading atomically; the corrected patch applied with prior bytes
preserved. Bank B build passed without diagnostics in 3.12 seconds from cache.
Freeze: 273 source inputs, 1,288 preserved records; all five program artifacts are
unchanged. Copied host executable SHA-256:
`fd764065498d22c7fa1741b6bd6a1355163e4654c11e4933c7ed2e7081a7bdde`.

Runtime B confirms the inactive canonical EpochRewards account and real Stake
minimum success (1,000,000,000 lamports, 630 CU). It then fails inside PIV preparation
with `memory allocation failed, out of memory` / `ProgramFailedToComplete` at
254,055 total CU. No System funding CPI was reached; the fail-fast test has no
post-state snapshot for that preparation, so its rollback is not independently
claimed. The existing PIV default bump allocator is 32 KiB and deallocation is a
no-op, despite the transaction's 262,144-byte VM heap request. The exact failing
allocation is not localized. Query AccountRecord collection, staged Config/Round
and four envelopes, then record collection/clone are investigation candidates.

Root and separate reviewer independently checked 1,736 B account records across
26 snapshots, 14 transaction/commit records and both earlier complete non-fee
rollback/retained-fee/identical no-fee replay cases. Reviewer parsed all 54 CPI records and checked the failing preparation trace. Largest observed packet remains 1,544 bytes; initializer uses
1,067,599 CU. Success before preparation does not prove later delayed legs, rent,
settlement, pending integration, KIF, all adapter profiles or live readiness.
Late outer-instruction rollback is distinct from an external CPI that mutates then
fails internally; this run does not prove the latter. No complete marker exists.

Preserve `bank-attempt-b-sources`, `bank-attempt-b-preserved.json` (15 execution
files), `root-runtime-b-review.json`, `reviewer-runtime-b-blocker.json`, both build/
runtime directories and every previous attempt. No production source, persisted
layout, economics, dependencies or historical evidence changed. Old 591 host tests
+1 doctest/eight gates and PIV SBF were NOT rerun; they remain Task 2.50 evidence.
Current executions are nine runner tests, strict upstream builds (two rejected
attempts before C success), two clean Bank builds and two failed Bank runtimes.

Next continue this same Task 2.51 with a bounded allocation correction, separate
review and proportionate targeted/final gates, preserving exact custody/receipt,
HWM/pending/rent/KIF/carry, error-order and atomicity guards. Prefer investigating
allocation demand before any allocator contract change; merely requesting a larger
VM heap is already disproved as a fix. Retain the failing ELF and create a new
frozen production artifact for the corrected run. Reuse the reviewed real-program
harness and upstream artifacts. Complete the delayed cycle before Testnet packaging.

Publication is a reviewed integration-only checkpoint of work in progress and the
exact blocker, not technical validation of a completed lifecycle or founder
acceptance. Main stays `1054ff3`; no signing, secrets, live RPC/deployment, funds or
authority transfer. Git and `/tmp/piv1-t251-pilot-review/publication.json` record
the actual checkpoint identity. Save this bounded session to conserve credits;
resume from actual refs/worktree and the checkpoint without restarting the work.

## October 12 resumption: bounded allocation corrections and runtimes C/D

Actual takeover was clean jerem integration `b9b0f0775ab6381bc001fee2c744cbbce0c38efb`,
matching remote; main `1054ff3` and protected Task 2.3 `3677fee` were independently
verified unchanged. Root froze 385 tracked files and 1,360 preserved records.
One bounded writer and a separate reviewer coordinated directly; root inspected
source, executed all commands and owns integration-only publication.

Preparation's two fallible record collections now reserve exact account counts,
and prediction of two distinct-source movements no longer clones the full record
vector. Every original record/receipt, cumulative escrow prediction, read/error
order and final atomic commit stays intact. Pinned Rust fallible collection has
zero lower size hint and grows 4/8/16/32; the nonfreeing 32-KiB allocator strands
old buffers. At 128 bytes per record, this removes 11,776/11,648 cumulative buffer
bytes for 28/29 accounts. This is source-derived demand, not peak heap measurement
or localization of runtime B's exact failing allocation.

Root's first focused run passed 1 new capacity/full-record/tamper/borrow regression
and both 10-test preparation suites. First final gates passed 592 host tests plus
one doctest and all eight gates without diagnostics. First strict SBF passed;
retained 843,984-byte ELF SHA-256:
`6f17ff0c7e2e7e1decdf6952c74602226f80020047af2bcf0b47d98b8eee5132`.
Source freeze/copies: 128 production inputs, then 273 Bank inputs / 1,610 prior
records. C retains the unchanged Bank harness and all four external program ELFs.

Runtime C passed preparation at 341,233 CU, including genuine Stake minimum and
pending-first System funding with complete independent Config/round/custody bytes.
Post-snapshot 0.2 SOL plus 0.5 JitoSOL intake also passed. First leg returned 6014
(StateEnvelopeEncodingFailed) at 225,117 CU after the real Stake minimum query,
before any funding CPI. Fixed Config/round/leg serialization fits its envelopes;
source diagnosis points to reservation failure but does not identify an individual
envelope/branch by runtime instrumentation. The test expected a later outer
instruction failure, so it stopped before an after-snapshot for this rejected leg.
Root/reviewer parsed 2,008 records / 30 snapshots, 16 transactions/commits and 58
CPIs; full preparation and pending intake maps and the two earlier rollback/fee/
replay cases passed. The full lifecycle did not pass.

A second bounded correction preallocates only the leg's two record snapshots.
It retains the full expected clone, Step construction, signer groups, envelopes,
complete per-CPI receipts and error order. Source-derived cumulative buffer savings
are 9,216/8,960 bytes for 24/25 accounts. No allocator, dependency, layout, economics
or remaining-handler change was made. A safe new regression covers exact capacity,
all record fields, full data hash, final-byte tampering and borrow rejection.

Crossed test-name messages produced a naming-only preexecution freeze mismatch;
review refused that stale freeze before commands. Both source copies/freezes remain
preserved. Final short-name test source `9db2936f…` and `source-freeze-final.json`
match; no tests/builds were executed on the stale naming freeze. Root then passed
1 new unit +10 existing leg tests on first execution, followed by final 593 host
tests +1 doctest/eight gates without diagnostics. First strict SBF passed; retained
846,192-byte final ELF SHA-256:
`f0faebae1f80296c0297512ed154d7d7f7f2491680f63ff305556efb70026c5f`.
Both cached Bank builds passed; unchanged copied harness SHA:
`fd764065498d22c7fa1741b6bd6a1355163e4654c11e4933c7ed2e7081a7bdde`.
No fresh Bank dependency rebuild is claimed. Existing runner regressions were not
rerun because runner/harness stayed unchanged; their prior nine passes remain
historical evidence.

Runtime D again passes preparation and post-snapshot intake. It gets past the
6014 staging failure and executes genuine System CreateAccount for 263-byte leg
metadata, debiting 2,721,360 lamports from operational custody within that failed
transaction. The System CPI succeeds. PIV then logs memory allocation failure and
returns ProgramFailedToComplete at 288,351 total CU, before a pool withdrawal CPI.
There are only two recorded leg CPIs: Stake minimum and metadata creation. This
proves progress through staging and the first System invocation; it does not prove
successful leg initiation. The exact failing allocation is unlocalized. Runtime
CPI buffers, retained staging/expected clone and preflight vector growth are next
candidates, not proven failing lines.

The harness asserts expected late-outer error before its after-snapshot; therefore
no full post-failure account map exists for this leg. No independent rollback,
metadata purge or committed economic movement is claimed for it. Earlier intake
and pool-deposit late-outer failures still prove full non-fee rollback, retained
fees and identical no-fee replay; they are distinct from a mutating external CPI
that fails internally. Root/reviewer independently checked D's 2,008 complete
account records / 30 snapshots, 16 transactions/commits and 59 CPIs, and complete
preparation/post-snapshot intake oracles. Maximum packet remains 1,544 unsigned
bytes; initializer uses 1,067,618 CU. All external artifacts remain byte-identical.

Preserve evidence roots `/tmp/piv1-t251-heap-review-20261012-a` and
`/tmp/piv1-t251-leg-heap-review-20261012-a`, source copies, both new SBF directories,
Bank build/runtime C/D directories and all earlier attempts. D froze 273 inputs
and 2,282 prior records; later execution/review/publication records are preserved
separately in `publication-evidence.json`. All four recovery archives/restorations
remain intact. No cleanup, installation, secrets, signatures, live RPC/deployment,
live fund movement or authority transfer occurred.

This is a reviewed integration-only work-in-progress checkpoint. Preparation's
runtime blocker is closed; leg allocation pressure still blocks the complete
cycle. M3/M4 and Testnet packaging remain open; no founder acceptance is inferred.
Next localize/reduce this demonstrated leg demand without weakening any guard,
then rerun the unchanged lifecycle through both delayed legs, rent recovery,
settlement, pending integration and KIF. Save this bounded session economically.
Git and the D evidence root's `publication.json` identify publication; main and
protected Task 2.3 remain unchanged.

## October 12 runtime E: withdrawal heap correction and Stake finalization blocker

Clean takeover was jerem integration `a3d2cb127ea7318f3e500efda6b46d99af9a905d`,
matching remote; main `1054ff3` and protected Task 2.3 `3677fee` were unchanged.
Root froze 385 tracked paths and 2,596 prior records with about 10.98 GB free.
One bounded writer and a separate reviewer coordinated directly. Root inspected
code and evidence, executed the validation and owns integration-only publication.
D-034's stop-before-M5 instruction remains mandatory; M4 is still incomplete.

The single production-file correction replaces three allocating Stake serializers
with fixed 200-byte scratch, two allocating zero hashes with bounded scratch,
per-CPI AccountInfo/signer slice vectors with fixed private-constructor bounds and
growing preflight borrow vectors with twelve held slots. Noninline helpers isolate
scratch frames. Source serialized prefixes preserve opaque trailing bytes; fresh
Stake destinations retain zero padding. Exact account/seed prefixes, checked CPI
borrow handling, full snapshots/expected clone, staged writes, authentication,
read/error order and all economic/receipt/atomic-commit guards remain unchanged.
No dependency, persisted layout or allocator change was introduced.

Pinned source-derived eliminated cumulative allocations are 6,255 bytes without
prefunds / 6,751 with both prefunds. This is not measured total heap use or proof
for every topology. A new regression compares every Stake variant and active/
deactivated nonzero fields with pinned serialization, verifies source trailing
bytes, destination padding and borrow failure. Existing full-record and exact
instruction/account/seed integration oracles remain. A proposed arithmetic-only
allocation test was removed before freezing/execution; its derivation is retained
in the writer receipt. Focused/final host and strict SBF gates passed on first
execution; the separate Bank runtime failure is described below.

Root passed two focused units plus ten integration tests, then 594 host tests and
one doctest with all eight final gates. Fresh strict SBF passed without diagnostics
in 5m 14s; all 128 production inputs/source copies match. Retained ELF:
`/tmp/piv1-bank-smoke-leg-runtime-sbf-t251-20261012-a/artifacts/piv1.so`, 845,800 bytes,
SHA-256 `801b6c632bc0a32c84dc82dc4e5854dd12985d6b7db56f455a87d6fa1ba395a1`.
The cached Bank build passed and retained the unchanged harness SHA-256
`fd764065498d22c7fa1741b6bd6a1355163e4654c11e4933c7ed2e7081a7bdde`.
All four external program ELFs, runner and harness stayed byte-identical. Prior
runner regressions were not rerun; the previous nine passes remain historical.
Bank E freezes 273 source inputs/copies and 2,843 preserved records.

Runtime E repeats successful preparation at 341,233 CU and pending-after-snapshot
intake. Both real protected pool withdrawal legs now succeed, with immediate Stake
deactivation at 526,405 / 539,716 CU. Full account oracles verify source reduction,
destination authority/deactivation/balance, metadata, original operational rents,
pool/list/Mint burn, custody and cumulative round bytes. Complete late-outer failure
snapshots prove both temporary accounts disappear, every other non-fee account
returns unchanged and the fee remains; identical replays reject AlreadyProcessed
without another fee. The active-Stake finalization InsufficientFunds failure also
passes its full rollback and no-fee replay oracles. This establishes runtime fit
for the exercised two-leg profile, not every fee/prefund/account topology.

A real Bank epoch transition and epoch-one pool maintenance pass. Finalizing the
second leg then fails inside the actual pinned Stake 5.1 Withdraw CPI with
`unsupported BPF instruction`, propagated as ProgramFailedToComplete. Stake uses
7,114 CPI CU; the transaction uses 310,079 CU. The harness expected an inactivity
InsufficientFunds rejection and stops on the different error. The complete after-
snapshot precedes that classification assertion: all 72 accounts match the before
snapshot except 5,000 lamports of retained transaction fee. Root and reviewer
independently verify this rollback. No replay of this final failed transaction or
internal mutation before its failure is proved. The exact opcode/cause is not yet
established; successful finalization/account closure, rent recovery, settlement,
pending integration, KIF claim and the complete lifecycle remain unexecuted.

Root and separate reviewer checked 3,210 complete account records / 47 snapshots,
25 transaction/commit records and 99 CPIs, including all newly reached economic
and full account-map oracles. Initializer uses 1,067,617 CU; maximum observed
packet remains 1,544 unsigned bytes. All program account bytes match the preserved
ELFs. Synthetic external genesis/governance, signatures/transport and an external
CPI proved to mutate then fail internally remain explicit limitations. No M3/M4
completion, general live compatibility or founder acceptance is inferred.

Evidence root: `/tmp/piv1-t251-leg-runtime-review-20261012-a`, including writer,
source/command, host, SBF/Bank and independent runtime receipts. Preserve Bank E
build/run directories, all prior A–D attempts, source copies, ELFs and the four
recovery archives. Later evidence is frozen separately for publication. No cleanup,
installation, secret access, signing, live RPC/deployment, live fund movement or
authority transfer occurred.

Read-only delegated diagnosis finds a directly evidenced capability mismatch:
the original archived Stake ELF imports `sol_get_sysvar` with an R_BPF_64_32
relocation at `0x1a248`, while pinned Agave 4.2 registers that syscall only when
`get_sysvar_syscall_enabled` (SIMD-0127) is active. The synthetic Bank starts with
no active features. Pinned source shows cooldown after deactivation enters the
StakeHistory lookup path; the epoch-zero attempt can avoid it. Epoch-one evidence
already contains StakeHistory epoch zero, so missing history data is not the same
issue. Pinned sBPF maps an unresolved call to the observed UnsupportedInstruction.
The writer receipt records exact local source paths/hashes and distinguishes the
locally available Stake-interface API from the ELF's unverified exact embedded
build dependency. Attribution to this syscall remains a strongly supported
inference: no failing-PC trace or feature-corrected replay was executed. Any next
bounded fixture correction must assert only demonstrated required capabilities,
retain production artifacts/economic state and all lifecycle oracles, and undergo
separate review; blanket activation or error reclassification is not justified.

Next localize the pinned Stake/runtime failure across epoch progression, checking
actual failing instruction and feature/sysvar compatibility before correcting a
proven mismatch. Do not guess activation settings, substitute a synthetic Stake
result or relax inactivity/error/postcondition checks. Use bounded correction,
separate review and the unchanged complete lifecycle. This reviewed integration-
only checkpoint records progress and the exact remaining blocker; main and founder
acceptance remain separate. Save/end this session economically. Git and this
evidence root's `publication.json` record the actual publication identity.

## October 12 continuation — coherent Stake fixture and settlement allocation

Actual clean takeover was jerem on `integration/piv1-testnet` at
`c481433b9de498384aba299fda445116cbb19396`, matching origin. Main was
`1054ff3751ca4169eab7e46d77452634b11fc695`; protected Task 2.3 remains
`3677fee97e3617ee65e2828d222008ba0952bb3e`. One bounded writer and separate
review were used. D-035 authorizes normal main publication of the eight missing
reviewed commits and this independently validated continuation, within each
recorded scope. It does not declare M4 complete, accept the product or authorize
M5/live operations. The exact final identity is in Git and the publication receipt.

The local Bank feature profile now explicitly supplies three canonical slot-zero
feature accounts: `get_sysvar_syscall_enabled` (SIMD-0127),
`reduce_stake_warmup_cooldown` and `upgrade_bpf_stake_program_to_v5_1`.
Pinned local Agave/Stake source establishes the required history syscall, 9% rate
and fixed-point history arithmetic. Exact feature IDs, activation slots, runtime
flags, complete canonical feature accounts and unchanged Stake executable and
ProgramData bytes/authority are asserted at genesis, initial child, every warp
and every epoch child. Already-active `Some(0)` accounts do not enter the pinned
new-feature upgrade-buffer path. No blanket activation, migration or dependency
change occurs. EpochRewards stays inactive; its helper does not activate features.
This is an explicit synthetic compatibility profile, not live-cluster evidence.

Runtime F activates only the syscall capability. It resolves E's unsupported BPF
failure, but cannot finalize within 12 epochs: Bank uses legacy 25% history while
Stake 5.1 uses 9% fixed-point math. Cluster deactivating history reaches zero at
epoch 5 while the selected Stake retains 253,075,014 effective lamports. The
legitimate InsufficientFunds results and failed bound are retained. F has 6,469
full account records / 91 snapshots, 47 transaction/commit pairs and 110 CPIs.
No production artifact changed in F–H.

Runtime G aligns all three capabilities. The pinned integer recurrence for actual
assigned legs 5,000,000,000 and 3,608,636,361 lamports, with retained active stake
3,391,363,639, derives a finite 14-epoch fixture bound. Epoch 13 retains
75,545,467 / 54,523,224 lamports; epoch 14 reaches zero. This bounds the test only:
actual successful full Stake withdrawal is the sole readiness oracle. Both real
finalizations pass at epoch 14 (465,928 / 440,347 CU), with complete maps proving
original rent recovery and temporary account closure.

G then correctly commits only RecoveryRequired: projected protected value is
121,109,545,457 against required HWM 121,109,545,458. No payment or reward occurs;
all 71 accounts are unchanged except round recovery bytes and the 5,000-lamport
fee. Its success-only test oracle fails after this valid protective outcome.
The earlier late-outer settlement probe proves recovery-state rollback, not
payment rollback. G has 7,819 records / 107 snapshots, 55 transaction/commit pairs
and 115 CPIs. An independent parser initially made the same success-only
assumption; its failure and corrected explicit recovery review are retained.
No production protection was changed to satisfy that oracle.

Fresh runtime H adds a declared economic stimulus after both finalizations:
real System transfer of three external donor lamports into the pool reserve,
followed by real SPL pool maintenance. At the asserted ratio and held units,
0–2 lamports cannot increase the principal floor; 3 raises it by exactly one
lamport to 121,209,545,456. Full account maps prove only donor/reserve/pool/fee
changes, with PIV state and the frozen round unchanged. This is a distinct
success fixture, not a reset or bypass of G's recovery and not a capability-only
change. G remains the preserved negative case.

H reaches three successful System settlement transfers: HTFP 6,382,727,270,
Team 2,109,545,453 and KIF 216,363,636 lamports. PIV then exhausts its unchanged
32-KiB bump allocator at 316,975 total CU. Exact failing allocation remains
unlocalized. The harness checks the expected later outer failure before taking
its after-snapshot, so H proves no inspected settlement rollback, retained fee
or replay despite those successful nested transfers. H has 7,677 records / 105
snapshots, 54 transaction/commit pairs and 118 CPIs. All earlier reached maps pass.

The final bounded production correction changes only settlement record storage:
ordered full records reserve exact capacity instead of fallible geometric growth;
a fixed 28-entry lamport array replaces a full-record clone used solely to predict
cumulative transfers. The ABI already requires exactly 27/28 accounts. Checked
subtraction/addition, payment order, rent/error order, complete per-CPI snapshots,
recovery behavior, fresh final authentication and the atomic eight-account commit
remain unchanged. Source-derived successful-branch cumulative heap reduction is
7,680 bytes; recovery saves 4,224/4,096 bytes. These are structural calculations,
not measured total heap or a proven failing line. A genuine new regression covers
both snapshot sizes, all record fields, tail-data/lamport tampering and borrow
error order; ten existing full settlement tests retain cumulative/zero-active
payments, failures and custody/HWM protection.

Root executed the new focused regression plus ten existing settlement tests:
11 focused passes. The complete current workspace passed 595 host tests and one
doctest, with all eight final gates and no diagnostics. Fresh strict SBF passed
both compilation and diagnostic/4096-byte frame checks. The new 847,888-byte ELF is
`/tmp/piv1-bank-smoke-settlement-heap-sbf-t251-20261012-a/artifacts/piv1.so`, SHA-256
`f62237f6923058b0b3741f3a1d83f6511f12df8a32a3a49c7677863a433b859c`.
All 128 production inputs were frozen/copied; these are new root executions,
not relabeled previous 594-test evidence. The unchanged runner's nine regressions
remain historical and were not rerun for this correction.

Bank I binds 273 inputs/copies and 4,327 preserved prior records. Its guarded
build passes with the unchanged H harness executable SHA-256
`f11f7019c4809cbf47632e177f69ede13c46d0e8d56e602e5beadc3867c8d5ff`.
Only the production settlement source and PIV artifact differ from H; H's exact
feature profile, explicit external stimulus and full lifecycle oracles remain.
All four external program artifacts match their retained hashes.

**Actual Bank I result: RUN_PASS / first complete positive local lifecycle.**
Initialization, SOL/Jito contributions, initial bootstrap, protected SOL staking,
real pool maintenance, heartbeat, preparation and post-snapshot contributions,
two protected delayed legs and deactivation, active/early withdrawal rejections,
real 14-epoch Bank progression, out-of-order full finalizations, original rent
recovery, settlement, pending integration and earned KIF claim all pass complete
account-map oracles. Successful settlement uses 400,193 CU, pending integration
279,949 CU and KIF claim 101,236 CU. Final HWM is 121,955,000,003 lamports; both
pending ledgers are zero and KIF claimed is 216,363,636 lamports.

The successful settlement branch now reaches the intended later outer failure
(400,405 CU), with full after-snapshot rollback and retained fee, identical replay
without another fee, then a distinct committed success. Pending integration has
the same complete rollback/replay/retry evidence (late failure 280,161 CU), and
re-integration after Idle rejects. These add to the prior complete intake, pool
deposit and leg rollback/account-purge cases. They do not prove a callee mutated
before failing internally. H's missing after-snapshot is not retroactively filled.
Root and separate reviewer independently parse 8,671 complete account records /
119 snapshots, 62 transaction/commit pairs and 128 CPIs. The completion reports
54 distinct submitted test nonces plus recorded identical replays. All 30 feature
profiles and retained program account bytes remain bound to the actual artifacts.
Initializer uses 1,067,625 CU; largest observed packet remains 1,544 unsigned bytes.
Runtime heap fit is proved for this exercised profile, not every account topology.

M4 remains incomplete. Concrete remaining canonical coverage is production
cooldown-loss/severe-loss recovery, pause behavior, repeated-cycle cadence/carry,
and a callee demonstrated to mutate before failing internally with complete
rollback/retry proof, followed by independent requirements-to-evidence closure.
G is actual residual-HWM recovery on the preceding ELF, while current-source
recovery host tests pass; it is not a new-artifact negative runtime execution.
Synthetic external genesis/governance, transport/signatures and unsupported
adapter profiles remain explicit limits. This positive lifecycle does not assert
live compatibility, genuine validator rewards or founder functional acceptance.
D-034 requires stopping with a concise reviewed completion report before M5 once
M4 is fully technically validated. No M5 work is authorized or started here.

Evidence root: `/tmp/piv1-t251-stake-capability-review-20261012-a`, with separate
F/G/H preservation manifests, source/command reviews, root host/SBF/Bank logs and
independent runtime receipts. Preserve build/run directories suffixed `t251-f`,
`-g`, `-h`, `-i`, all A–E evidence, source copies and all four recovery archives.
No cleanup/install, secrets, signing, live RPC, deployment, live funds or authority
transfer occurred. Root's final publication freeze also preserves later review
records. D-035 authorizes normal fast-forward main and integration publication of
the eight prior reviewed commits plus this validated correction and truthful
checkpoint. Git and the evidence root's `publication.json` record its resulting
identity; exact final tree review precedes publication. Save/end this bounded
session economically and resume from actual refs/worktree and the active checkpoint.


## October 13 continuation — emergency pause through the production lifecycle

Clean takeover as jerem was `2f8d08368742b7a6c48a8f4e00096be217138fe9` on
`integration/piv1-testnet`, matching local/remote main and integration. Protected
Task 2.3 remains `3677fee`. D-035 publication is complete; this bounded D-026
continuation publishes only to integration. D-034 still requires full M4 closure
and a completion report before stopping for explicit M5 authorization.

Canonical D-032/G-004/K-012 and Task 2.40 require actual pause coverage, a concrete
M4 gap. One delegated writer and separate reviewer extend only local validation.
The synthetic caller admits a strict additional 16-role stored SetPause message,
20 outer roles, canonical vault PDA signing and exact 11-byte Boolean payload.
The existing 34/35-role initializer profile and original byte forwarding remain.
It is not a generic governance proxy. Sixteen distinct approved messages are
seeded only at genesis (transaction indices 20–35; synthetic counter 35), and
runtime pause/unpause calls go through the unchanged production PIV instruction.
No PIV economic state is injected. The synthetic caller does not consume proposals;
this is not real Squads governance or its replay-protection evidence.

The lifecycle fixture adds eight pause/unpause pairs and nine phase-valid
PausedOperation rejections, exact code 6035, before any CPI. It exercises explicit
SOL/Jito intake, bootstrap, principal staking, preparation, leg initiation, a
mature finalization, settlement and pending integration. Successful unpaused
operations retain the original full economic cycle. A late-outer pause failure
checks all-account rollback, retained fee, identical no-fee replay and distinct
retry. Actual direct transfers of one SOL lamport and one token unit while paused
are recorded through pending reconciliation; later explicit deposits subtract
those exact units, retaining the original 119-SOL bootstrap economics. Heartbeat
and payment of already-earned KIF remain allowed during pause. Every added
success/rejection uses a complete Bank account map, not only balances or events.

Pre-execution review caught an incomplete A source freeze: the old Bank profile
bound the historical caller only as an artifact, omitting its source/manifest/lock.
The corrected freeze explicitly binds all four caller inputs (277 total); original
A files/copies remain preserved. First actual host attempt B then stopped before
compilation because the standalone caller lock lacked current production edges.
Isolated offline metadata resolution changed only the existing PIV SHA/Stake and
Stake/instruction Borsh dependency lists. Attempt C compiled until the new boundary
test's shared fixture referenced `spl_token::ID` without a direct dev dependency;
no test executed. A reviewed exact dev-only SPL Token 8.0.0/no-entrypoint edge and
isolated lock synchronization fixed that missing declaration. All 158 package
versions, registry sources and checksums remain identical; no download/installation,
production dependency, layout, economics or allocator change occurred. Both failed
host attempts, original source copies and isolated resolutions remain preserved.

On final D source, root executed all 11 caller boundary tests successfully, then
all-target/all-feature check and warning-denied documentation. The exact 277 inputs
and copies match. The unchanged 128 production inputs retain the previous 595 host
tests +1 doctest/eight gates and strict PIV SBF evidence; those suites are not
reported as rerun here. Likewise the unchanged runner's earlier regressions remain
historical. A fresh strict caller-only SBF build and full Bank runtime are required
before this continuation is technically validated; the actual result follows.

### Actual caller SBF and Bank J result

The first fresh caller SBF execution passed strict diagnostics and ELF checks.
Caller artifact `/tmp/piv1-bank-smoke-pause-caller-sbf-t251-20261013-a/artifacts/piv1_genesis_production_caller.so`
is 61,856 bytes, SHA256
`284293d9b9b015571f7b755a36c3daac31793644646fa6281c4042163befb3ee`.
PIV/Token/pool/Stake artifacts remain unchanged. Bank J build passed; reviewed binary
SHA256 is `3e1b4d0687dcd6834a39c4f90f06e18cf6e7b9ff7a971a7ba741d7de79649e0a`.
Both exact commands use the existing locked/offline guarded runners, a single
worker, denied network sockets and retained source/artifact/evidence hashes.

Runtime J FAILED after successful epoch-14 second-leg finalization. Pinned
`solana-program-runtime-4.2.0/src/loaded_programs.rs:448` panics with
`Unexpected replacement of an entry` while processing `mature-finalization-pause`.
There is a submitted transaction but no commit result, nested trace or after-map
for that call. Do not infer rollback, success or the failing cache key/type.
The strict original independent parser also rejected this incomplete evidence
with a missing-commit KeyError; that parser failure is preserved. A new partial-
aware parser validates all messages but explicitly lists the uncommitted call;
it never manufactures a result or missing snapshot.

Root and independent reviewer checked 14,951 complete account records / 143
snapshots, 77 transactions / 76 commits and 122 CPIs. Eight pause/unpause flag
changes, six exact 6035/no-CPI rejections and identical no-fee replays pass. Covered
rejections are explicit SOL/Jito intake, bootstrap, principal staking, preparation
and leg initiation. Direct intake/reconciliation, heartbeat and the full late-outer
pause rollback/replay/distinct retry pass. Initializer uses 1,067,516 CU; maximum
message packet remains 1,544 unsigned bytes. Both withdrawal legs and the real
14-epoch second-leg finalization also pass their full economic account maps.
Mature-finalization/settlement/integration pause gates and paused KIF claim were
not reached. No `pause_profile_complete` or `lifecycle_complete` record exists.
The prior full positive I evidence remains valid and separate; it is not a J pass.

Read-only diagnosis confirms all five actual ELF ProgramData deployment slots are
zero. Missing later BankForks registration alone does not establish the cause for
those entries. A narrower candidate is a loader-owned non-executable ProgramData
account becoming a current-slot Closed cache entry whose slot is absent from the
synthetic fork graph. The failing key/type/environment is not logged, so this is
an inference to test, not a proven defect or authorization for a cache workaround.
Next use bounded read-only cache identity/environment instrumentation, then a
separately reviewed minimal fixture correction and rerun the unchanged lifecycle.
Do not clear caches, disable assertions, fabricate inactive Stake or weaken PIV.

### Preservation, publication and remaining scope

Bank J profile binds 277 inputs and 6,435 prior records; `attempt-j-preserved.json`
binds 6,738 records before any prospective correction. Later diagnostic/audit and
publication records are preserved separately. Commands actually executed here:
`validate_caller_b.py focused` (lock failure), `validate_caller_c.py focused`
(test compile failure), `validate_caller_d.py focused` and `sbf` (PASS),
`run_bank.py build` (PASS), `run_bank.py run` (FAIL), plus independent full-map
parsers and integrity audit. Evidence root:
`/tmp/piv1-t251-pause-review-20261013-a`; full runtime logs:
`/tmp/piv1-bank-smoke-production-lifecycle-run-t251-j`.

Reviewed publication is a partial development checkpoint on integration only;
Git and `publication.json` identify the commit. Main remains completed D-035
`2f8d083`, protected Task 2.3 remains `3677fee`. Production sources, accepted
economics and the prior PIV artifact are unchanged. No full pause validation,
M4 completion, founder acceptance or live readiness is declared. M4 still needs
the remaining pause cases, cooldown loss/recovery, repeated-cycle cadence/carry
and a callee proved to mutate then fail internally, plus requirement-to-evidence
closure. D-034 requires the reviewed M4 completion report and stop before M5;
M5 has not started. Save/end this bounded session economically and resume from
actual refs/worktree and the preserved blocker, without restarting implementation.

## October 13 continuation — exact cache diagnosis and complete pause profile

Clean takeover as jerem was integration `c4dc543ec78a3edd5970f2da666b853cdd11b219`,
matching the remote, with clean worktree. Main remains `2f8d083` and protected
Task 2.3 remains `3677fee`. Baseline preservation binds 386 tracked inputs and
6,756 previous records. This bounded D-026 continuation uses one delegated writer,
separate review, root-owned commands/checkpoints and integration-only publication.
No accepted economics, production/caller source, layout, dependency or allocator
change occurred. The critical-path objective is to finish the existing pause
profile; this is not a new validation-only detour or an M5 task.

### K — read-only reproduction before correction

The diagnostic reads cache entries for every message key, including metadata
Closed entries omitted by the flattened executable-only view. It records exact
variant/owner/deployment/effective slots, optional execution-environment equality,
BankForks membership and inputs to the explicitly derived pinned fork relationship.
It catches only the known mature-pause runtime panic, reads the poisoned lock for
diagnostic evidence and unconditionally resumes the original panic. Separate
pre-execution review added a nested catch so a failed diagnostic cannot replace
that original payload; both original and corrected source freezes are retained.
No runtime failure is suppressed or reclassified as success.

Locked/offline K build passed without diagnostics; binary SHA256:
`6426b97d7181270b02bba7d5e0834eb502f97495d4db397b439e8d11e67575c9`.
Actual K reproduced the same `loaded_programs.rs:448` assertion while submitting
`mature-finalization-pause`, still without a commit result or after-map. Root and
reviewer independently verified all 143 J/K full account maps are byte-identical
(14,951 records, 77 transactions / 76 commits, 122 CPIs). Original error and partial
evidence remain separate from the later success.

The directly observed PIV ProgramData key is
`98bMEPMDMvHMrAe5DSdhMK6i49otMmox8acRRe35R8XR`. Before the call, its cache has
one `ProgramCacheEntryType::Closed` entry at deployment/effective slot 1, with no
execution environment. Current Bank slot 524257 (epoch 14) is unregistered while
the fork graph's highest slot is 1; that entry is therefore Unknown/invisible.
After the panic, the cache is poisoned and contains another invisible Closed
entry at slot 524257. Actual ELF ProgramData deployment headers remain slot zero.
This is a non-executable metadata cache classification, not closure of the real
PIV program or a new deployment.

Pinned `program_loader.rs` filters all loader-owned message accounts, treats
ProgramData metadata as invalid executable data, and caches a current-slot Closed
entry. Combined with the observed missing fork registration and before/after
entries, this explains repeated metadata loading and the duplicate insertion
assertion. The failed `assign_program` stack-local key was not directly logged;
that last causal linkage is an inference from exact observations and pinned code.
The structural fixture defect itself is directly observed, not guessed.

### Reviewed correction and L — complete runtime pass

The only behavioral correction constructs each warped Bank normally before taking
the forks write lock, inserts it through `BankForks::insert`, then creates its child
through `Bank::new_from_parent_with_bank_forks`. No cache clearing, manual entry
insertion, environment replacement, rooting/pruning, assertion bypass, manual account
write or economic oracle change occurs. Thirty read-only regression observations
assert exact current Bank identity, registered ancestors and retention/visibility
of the original single Closed(1) metadata entry across all 14 epochs and before/
after the formerly failing mature-pause call. Diagnostics remain test-only.

Locked/offline L build passed without diagnostics; binary SHA256:
`d21c976b67f1358bbf5158c6ee6c1997c0e1f3a3439c3038df0abe9a30837d8e`.
Actual L completed the full lifecycle and pause profile with empty runtime stderr.
All 143 shared K/L account maps are byte-identical. The previously failing call
now sees the same singleton Closed(1) entry as an ancestor before and after, with
no replacement at slot 524257. The entire original economic success path remains.

Root and separate reviewer checked 19,497 complete account records / 187 snapshots,
101 transaction/commit pairs (83 distinct nonce transactions plus replays), 145
CPIs, 30 capability profiles, 30 registration records and all five exact artifacts.
Sixteen Config-only pause/unpause calls and nine exact PausedOperation 6035/no-CPI
rejections pass full fee-only rollback and identical no-fee replay. The rejected
operations are explicit SOL/Jito deposits, bootstrap, principal staking, preparation,
leg initiation, mature finalization, settlement and pending integration. Actual
direct transfers of one lamport and one token unit remain reconcilable while paused;
heartbeat and the already-earned KIF claim succeed while paused. Late-outer pause,
intake, deposit, leg, settlement and integration failures preserve their complete
rollback/replay/distinct retry evidence.

The full cycle retains two delayed legs, real epoch-14 cooldown, out-of-order
finalization and original rent recovery, settlement and pending integration.
Final HWM is 121,955,000,003, pending SOL/tokens are zero, and KIF claimed is
216,363,636 lamports. Initializer uses 1,067,516 CU; mature-pause call 168,942;
settlement 400,193; pending integration 279,949; KIF claim 101,240. Maximum packet
remains 1,544 unsigned bytes. The explicit three-lamport external reserve stimulus
is unchanged. G's earlier-artifact residual-HWM recovery remains a separate case.

### Actual commands, unchanged artifacts and checkpoint

Root executed `run_bank.py k build` (PASS), `run_bank.py k run` (preserved diagnostic
FAIL), `run_bank.py l build` (PASS), `run_bank.py l run` (PASS), the J/K and K/L full-
account prefix/cache audits, and the independent full pause-account parser. Existing
locked/offline runner guards enforce pinned dependencies, one worker, denied network
sockets, resource limits and exact source/artifact/evidence preservation. No runtime
artifact or economic fixture was swapped to obtain the successful result.

All five fixture ELFs remain exactly those loaded in J: PIV `f62237f6…`, synthetic
caller `284293d9…`, Token `7cfba90f…`, pool `314cefab…`, Stake `3d2d39c5…`.
The 128 production inputs still match the previous strict SBF freeze. Previous
595 host tests +1 doctest/eight gates and PIV SBF, plus the prior 11 caller boundary
tests/check/docs and caller SBF, are verified retained evidence, not rerun here.
Only the Bank test harness required rebuilding and actual runtime regression.

Evidence root: `/tmp/piv1-t251-cache-review-20261013-a`. K's final diagnostic freeze
has 277 inputs / 7,042 prior records. Before correction, 7,346 records are preserved
in `attempt-k-preserved.json`. L freezes 277 inputs / 7,351 prior records; subsequent
execution/review/publication receipts are retained separately. Keep every A–L run,
source freeze/copy, artifact, failed attempt and all four recovery archives.

Reviewed publication is integration-only; Git and the evidence `publication.json`
identify the resulting commit. Main remains `2f8d083`, protected Task 2.3 remains
`3677fee`. No secret access, signing, live RPC/deployment, real fund movement,
authority transfer, cleanup or installation occurred. The complete local pause
profile is technically validated within its scope, not founder-accepted.

M4 remains incomplete: next close actual cooldown loss/recovery, repeated-cycle
cadence/carry and a callee proved to mutate then fail internally, with exact
rollback/retry and separate requirement-to-evidence closure. Synthetic external
genesis/governance, proposal non-consumption, unsigned transport/signatures and
supported adapter limits remain explicit. This does not prove unimplemented
migrations or real Squads governance. D-034 requires a reviewed completion report
and stop before M5 once M4 is fully validated; M5 has not started or been authorized.
Save/end this bounded session economically; resume from actual refs and checkpoint.

## Bounded continuation — M positive refresh and N ordinary cooldown loss (2026-10-13 UTC)

**IN PROGRESS / ORDINARY COOLDOWN-LOSS PROFILE VALIDATED. M4 is not complete.**
Clean takeover as jerem was integration `e2d1e8203ef5e3eea464e9db5dea306f4d8cad5d`;
main remains completed D-035 `2f8d083`, protected Task 2.3 remains `3677fee`.
One bounded writer changed only the local Bank lifecycle harness/README and the
exact-profile runner/tests. Root coordinated separate source, command, evidence
and checkpoint review and owns D-026 integration-only publication. This is not
founder acceptance, broader main authority, M4 completion or M5 authorization.

### Scope and unchanged production

The runner defaults to `positive`, selecting exactly
`complete_delayed_production_lifecycle`. `cooldown-loss` selects exactly
`cooldown_loss_recovery_production_lifecycle` from the same reviewed Bank binary.
Unknown selectors, wrong test names, mixed/duplicate completion markers, malformed
evidence, wrong numeric/boolean types, altered accounting and live-provenance
overclaims reject. The loss marker cannot satisfy positive completion, or vice
versa. Existing source/artifact/history, approved binary, dependency, offline,
resource and Git guards remain in force. Actual full account maps are the economic
proof; completion markers alone are not a substitute.

Production and synthetic governance caller sources, all 128 production freeze
inputs, dependencies/layouts and the 32-KiB production allocator remain unchanged.
The actual five program artifacts remain PIV `f62237f6…`, caller `284293d9…`,
Token 8.0.0 `7cfba90f…`, SPL pool 2.0.3 `314cefab…` and Stake 5.1.0 `3d2d39c5…`.
Prior 595 host tests +1 doctest/eight final gates and strict PIV SBF, plus 11 caller
boundary tests/check/docs/caller SBF, are hash-verified retained evidence. They
were not rerun and no new SBF artifact is claimed.

### Actual commands and first-attempt correction

Root ran `python3 -I -B tools/test_validate_production_lifecycle.py -v`: 13 tests
passed and one new negative test errored. Its fabricated transcript omitted a
newline between shock JSON and the libtest footer, yielding JSON `Extra data`
before the intended semantic rejection. All four initial source files and the
failed logs are preserved under `python-attempt-1-source` and the first regression
receipt. The writer added only that missing fixture newline; separate review
confirmed the strict runner was unchanged. Root's second execution passed all 14
tests, including five new loss-profile regression groups. No Bank run had yet
occurred; this was not a production defect or runtime lifecycle failure.

Root then executed the reviewed `run_bank.py m build`, `run_bank.py m run`, and
`run_bank.py n run`. One locked/offline Bank build passed without diagnostics;
M explicitly selected `--profile positive`, N selected `--profile cooldown-loss`.
Omitted-selection default behavior is covered by the Python regression. Both
actual runtimes passed with empty stderr. Shared Bank binary SHA256:
`4df0830bf88bf928e7663c3028fe6af7915fcb8164bc892dcc27aca0a1a41839`.
Pinned dependencies/cache are reused; this is not a fresh dependency rebuild.

A supplemental reviewer audit initially compared raw diagnostic ancestor arrays
serialized from unordered sets. Its assertion failure is preserved in
`reviewer-supplement-attempt1.json`; the corrected comparison sorts only that set
serialization, leaving every other diagnostic/account/message field exact.
The supplemental audit then passed. No harness, production source or runtime
oracle was changed, and neither actual runtime failed.

### M — unchanged complete positive and pause lifecycle

Root compared every M/L complete account map, transaction record and completion:
all 187 snapshots / 19,497 account records and 101 transaction records are identical.
Root and separate review also executed the full account/economic/pause/rollback
parsers. M contains 145 CPIs and 83 distinct nonce transactions plus 18 replays.
Sixteen pause transitions, nine exact 6035 economic blocks, direct reconciliation,
heartbeat and earned KIF while paused retain their exact prior proofs.

Both delayed legs, real epoch-14 cooldown, out-of-order finalization/rent,
settlement, pending integration and KIF complete. Final HWM is 121,955,000,003,
pending SOL/tokens are zero and claimed KIF is 216,363,636 lamports. Settlement uses
400,193 CU, integration 279,949 CU, KIF claim 101,240 CU. The explicit three-lamport
external reserve appreciation from I remains. M does not reset or replace G's
separate earlier-ELF residual-HWM recovery case.

### N — explicit external loss, real finalization and guarded recovery

Immediately after both protected leg initiations/deactivations, before real Bank
epoch progression, the loss-only fixture supplies a balanced `Bank::store_accounts`
pair: external-owned Stake leg 0 loses one lamport and existing external System
donor `key(220)` receives it. Full before/after maps show exactly those two balance
changes; every owner/data/executable/rent field, Stake delegation and PIV byte stays
unchanged. Total account lamports and Bank capitalization stay unchanged, and
original rent remains covered. This declares a synthetic external loss input. It
is not a production transaction, an implemented slash mechanism or live-slashing
proof. The positive profile never performs it.

Real Stake full withdrawal remains the inactivity authority. Exact failed attempts
through epoch 13 preserve fee-only account effects; no readiness guard is bypassed.
The unaffected companion leg 1 finalizes first at epoch 14. A late outer failure
then follows a successful loss-bearing Stake/System finalization. Complete maps
prove rollback to the already loss-adjusted world, retained transaction fee and
identical no-fee replay. A distinct transaction retries successfully, closing leg 0
Stake/metadata and committing `RecoveryRequired` with cooldown-loss flag 1.

The independent byte oracle distinguishes original delegated value from actual
native received: cumulative finalized delegated value uses the original amount;
actual receipt is one lamport lower and cumulative cooldown loss is exactly one.
Both original Stake rents total 4,565,760, metadata rents total 5,442,720, all return
to operations, and all four temporary accounts are purged. Config remains exactly
as after assignment: HWM 119,000,000,000, pending SOL 300,000,000 and pending token
units 500,000,000. Beneficiaries, guardian rewards and KIF receive no payments or new
credits. N does not execute the positive reserve appreciation, settlement, pending
integration or earned KIF claim.

Fresh settlement, pending-integration and finalization calls then reject exact
6073 before any CPI. Each retains only its transaction fee; identical replay is
AlreadyProcessed with no additional fee or account change. This proves guarded
entry into recovery; no recovery exit, reset or repair is invented.

Root and separate reviewer inspected 17,348 complete account records / 166 snapshots,
89 transaction/commit pairs (72 distinct nonce transactions plus 17 replays), 128
CPIs, 30 capability and 30 cache-registration records. Exact late-outer finalization
and successful retry both contain pinned Stake Withdraw followed by original-rent
System transfer. Companion finalization uses 465,928 CU; the loss-bearing success
440,310 CU; late outer failure 440,522 CU. The three recovery guards use 64,723 /
59,689 / 62,092 CU. Initializer remains 1,067,516 CU and maximum observed packet
1,544 unsigned bytes. VM heap request remains 262,144; production allocator 32 KiB.

### Remaining M4 evidence and durable resumption

This N profile finalizes the loss-bearing leg last. It does not prove recovery
entered while another leg/rent remains outstanding. The existing 6073 guard blocks
normal subsequent finalization; canonical P-035/A-005 do not authorize bypassing it
or inventing a governed recovery exit. Retain this as a concrete adverse-evidence
gap. A next bounded severe-pool-loss profile may cover T23-R1's retained-value-below-
committed-pending-use boundary and early recovery with a companion still outstanding,
provided exact account, rent, HWM/pending and blocked-follow-up oracles are reviewed.
Ordinary last-leg loss does not close that severe-loss requirement.

Repeated-cycle cadence/carry, a callee proved to mutate before failing internally,
and independent requirement-to-evidence closure also remain. A late outer failure
after successful CPI is not an internally failing mutating callee. Synthetic
external genesis/governance, unconsumed proposals, unsigned transport/signatures,
initializer compute/packet constraints and supported adapter-profile limits remain
explicit. No general cluster readiness, real Squads workflow or unimplemented
migration is inferred.

Evidence root: `/tmp/piv1-t251-loss-review-20261013-a`. Shared M/N freeze contains
277 inputs and 7,692 prior records; execution, independent review and publication
receipts are retained separately. Preserve every A–N attempt, original source
copy/freeze, ELF, failed test and all four recovery archives. Git and `publication.json`
identify the reviewed integration checkpoint. Main stays `2f8d083`; protected
Task 2.3 stays `3677fee`. No cleanup/install, secrets, signing, live RPC/deployment,
real funds or authority transfer occurred.

Save/end this bounded session economically and verify actual refs/worktree on
resumption. D-034 requires a reviewed completion report and STOP before M5 once
M4 is fully validated. M4 remains incomplete; M5 has not started or been authorized.

## Bounded continuation — O positive refresh, P early recovery and Q severe settlement loss (2026-10-13 UTC)

**IN PROGRESS / SEVERE POOL-LOSS PROFILES VALIDATED. M4 is not complete.**
Clean takeover as jerem was integration `ffc2bb39f9e1510eb5485c0f92b879e3abe60967`;
main remains completed D-035 `2f8d083`, protected Task 2.3 remains `3677fee`.
One bounded writer changed only the local Bank harness/README and exact-profile
runner/tests; root prepared pins, executed validation and coordinated separate
source, command, evidence and checkpoint review. Publication is D-026 integration
only, not founder acceptance, main publication authority or M5 authorization.

### Source scope, executed checks and preserved history

Two additional exact selectors bind tests and typed completion markers:
`early-pool-loss` / `early_pool_loss_recovery_production_lifecycle` and
`settlement-pool-loss` / `settlement_pool_loss_recovery_production_lifecycle`.
The runner rejects mixed/unknown completions, altered loss accounting, wrong types
and missing/changed actual settlement recovery events. Positive defaults and all
prior source/artifact/dependency/offline/resource guards remain. Seven prior
loss/pause helper bodies and N's completion body were compared byte-for-byte with
HEAD; ordinary-loss enum routing retains its old semantics. N was not rerun.

Root executed `python3 -I -B tools/test_validate_production_lifecycle.py -v`:
all 19 tests passed on first execution, including five new severe-profile groups.
Reviewed wrappers then executed `run_bank.py o build` and sequential `o run`,
`p run`, `q run`, selecting positive, early-pool-loss and settlement-pool-loss.
One locked/offline Bank build and all three actual runs passed without diagnostics;
runtime stderr is empty. The shared Bank binary SHA256 is
`7e30f4c8a3622bb0d1754a6efbb3c6b553b562503af7a34c0771e93ca6b9939e`.
Pinned caches/dependencies were reused, not rebuilt afresh or upgraded.

All 128 production freeze inputs, caller source, dependencies/layouts, accepted
economics and production 32-KiB allocator remain unchanged. All five actual ELFs
remain PIV `f62237f6…`, caller `284293d9…`, Token 8.0.0 `7cfba90f…`, SPL pool 2.0.3
`314cefab…` and Stake 5.1.0 `3d2d39c5…`. Prior 595 host tests +1 doctest/eight gates
and strict PIV SBF, plus 11 caller tests/check/docs/caller SBF, are hash-verified
retained evidence, not new executions. No new production bytecode is claimed.

Root's supplemental old-function comparison initially captured the next function's
comments and failed equality. `root-retained-n-audit-attempt1.json` preserves that
audit-only failure. Restricting extraction to the actual column-zero closing brace
proved all seven function bodies and the N completion body identical. No source,
test oracle, build or runtime changed; no actual test/build/runtime failed here.

### O — exact complete positive and pause refresh

Every full account map, transaction and completion matches M: 19,497 records / 187
snapshots, 101 transaction/commit pairs (83 distinct nonce transactions +18 replays)
and 145 CPIs. Root and reviewer independently parsed economic/pause/rollback maps.
All prior actual protected deposit, two delayed legs, epoch-14 readiness, original
rents, out-of-order finalization, settlement, pending integration and KIF complete.
HWM is 121,955,000,003, pending SOL/tokens zero, KIF claimed 216,363,636. Sixteen
pause transitions and nine exact economic blocks retain their old evidence. The
explicit three-lamport external reserve appreciation remains. P/Q do not take that
positive appreciation/payment path; they do not replace N or G's earlier evidence.

### Shared severe fixture and canonical requirement distinction

P-019 retains HWM after loss; P-035/A-005 protect original rents and exact multileg
custody. T23-R1 separately permits a zero floor for the protected-value comparison
when projected retained value is below already committed pending SOL use; it does
not permit negative custody arithmetic, payments or reduction of recorded pending.
Production finalization compares residual historical value directly against the
stored residual HWM floor. It does not subtract pending use. Consequently P proves
that early residual guard; Q executes the actual settlement zero-floor branch.

At epoch 14, after ordinary pool maintenance, a declared `Bank::store_accounts`
fixture changes four balances only. Reserve 1,187,997,717,120 and remaining source
balances 1,002,282,880 / 2,393,646,519 each fall to original rent 2,282,880. Existing
external System donor `key(220)` receives exactly 1,191,386,797,879. Full maps prove
all owners/data/executable/rent fields, PIV state and withdrawn legs unchanged;
aggregate lamports and Bank capitalization are conserved. This is synthetic loss
input, not a production loss instruction or real slash. The resulting source data/
balance deficit is explicit; these are not claimed healthy delegated sources.

Actual pinned `UpdateValidatorListBalance` and `UpdateStakePoolBalance` instructions
then publish total 4,565,760 / unchanged supply 1,092,108,750,001. The former reads
actual native balances; no manual pool/list/Mint data mutation substitutes for it.
Complete maps constrain list/pool changes and transaction fees, with no loss fee
mint. Principal 111,108,750,001 units value to 464,510. Independent arithmetic shows
reserve-only depletion would value them at 345,494,412, above pending use 100,000,000;
it cannot prove this severe boundary. Root/reviewer inspected the pinned processor
and independently derived the actual four-account fixture and resulting ratio.

### P — first successful finalization enters recovery with a companion outstanding

The shock precedes the first successful epoch-14 finalization. Actual Stake/System
execution finalizes leg 1, receives native value 3,608,636,361 excluding original
Stake rent, returns Stake rent 2,282,880 and metadata rent 2,721,360 to operations,
and closes those two temporary accounts. Residual value 464,510 is below the stored
floor; the round enters RecoveryRequired/flag 2, with no native cooldown loss.
Companion leg 0 remains byte-for-byte unchanged, including authority and balances
5,002,282,880 Stake / 2,721,360 metadata. Config stays identical: HWM 119,000,000,000,
pending SOL 300,000,000 and pending tokens 500,000,000. No payment/new reward credit.

A late outer failure after successful finalization proves complete rollback of
custody/rents/closures/round, retained transaction fee and identical no-fee replay;
a distinct successful transaction then commits recovery. Three subsequent calls
(settlement, integration and outstanding companion finalization) reject exact 6073
before CPI with fee-only maps and identical replay. That last guard is preserved,
not a demonstrated method for recovering the companion. No exit policy is invented.

### Q — real severe settlement zero floor after all legs and rents complete

Both normal finalizations occur before the shock; all four temporary accounts close
and original rents 4,565,760 Stake +5,442,720 metadata return to operations. The
projected post-payment escrow remainder is two lamports, making retained value
464,512 below pending use 100,000,000. Actual production settlement succeeds with
only round header state4/flag2 committed, plus the transaction fee; Config, custody,
recipients and rewards remain exact. There is no CPI, payment or new reward credit.

Both the attempted late-outer-failure transaction and distinct successful retry
contain the exact real 57-byte `DistributionSettlementRecovery` event in actual
Bank transaction logs: discriminator, Config address, sequence 0, observed protected
value 0 and flag 2. Attempted program logs are explicitly distinguished from a
successful enclosing transaction. A small fixed-length encoder introduces no new
dependency. Independent parsing checks the actual log payload against the complete
account-derived protected-value oracle, not only a harness completion marker.

Complete rollback/retained fee/identical no-fee replay precede the distinct retry;
fresh settlement/integration/finalization calls then reject exact 6073 before CPI,
with identical no-fee replay. HWM
and full pending values stay unchanged. Q does not claim governed recovery exit.

### Actual runtime measurements and checkpoint limits

Root and separate review inspect P's 16,540 full account records / 158 snapshots,
85 transaction/commit pairs (69 distinct transactions +16 replays), 124 CPIs,
30 capability records and 28 cache registrations. P finalization uses 465,411 CU,
its late outer failure 465,623 CU. Q contains 17,746 records / 170 snapshots,
91 transaction/commit pairs (74 distinct +17 replays), 126 CPIs, 30 capability and
30 cache-registration records. Actual recovery settlement uses 249,650 CU; its
late outer failure 249,862 CU. Severe maintenance uses 25,757 CU in both profiles.
Initializer is 1,067,516 CU and largest packet 1,544 unsigned bytes in all profiles;
VM heap request remains 262,144 bytes with unchanged production 32-KiB allocator.

At post-run inspection, free disk was 2,533,289,984 bytes, above the mandatory
2,147,483,648-byte reserve but with limited headroom. No logs/artifacts were deleted.
Further large runs require an actual capacity check; this session adds no new task.

M4 still needs repeated-cycle cadence/carry, an actual callee proved to mutate before
failing internally, and independent requirement-to-evidence closure plus any concrete
blocker. A late outer failure after successful CPI is not that internal failure.
Governed recovery exit, including an outstanding companion after early recovery,
remains unimplemented/unproved; these entry/guard tests do not invent its policy.
Synthetic external genesis/governance, unconsumed proposals, unsigned transport/
signatures, initializer compute/packet constraints and supported adapter-profile
limits remain. No general cluster readiness or live slash/Squads proof is claimed.

Evidence root: `/tmp/piv1-t251-severe-loss-review-20261013-a`; shared O/P/Q freeze has
277 inputs and 8,046 prior records. Subsequent execution/review/publication evidence
is separately preserved. Keep A–Q attempts/source copies/ELFs/receipts and all four
recovery archives. Git and `publication.json` identify the reviewed integration
checkpoint. Main stays `2f8d083`, protected Task 2.3 `3677fee`. No cleanup/install,
secrets, signing, live RPC/deployment, real funds or authority transfer occurred.

Save/end this bounded session economically. On resume verify actual refs/worktree
and disk capacity, preserving the 2-GiB runner reserve and every historical record.
D-034 requires a reviewed completion report and STOP before M5 once M4 is fully
validated. M4 is not complete. M5 has not started and remains unauthorized.


## Bounded continuation — S repeated production cycles, cadence and KIF carry (2026-10-15 UTC)

**IN PROGRESS / REPEATED-CYCLE CADENCE AND KIF CARRY VALIDATED. M4 remains incomplete.**
Clean takeover as jerem was integration `c8a8a54416ff2d42e101fd1bbfe8b8e1618c197c`,
with a clean sole worktree, matching remote integration, main `2f8d083` and protected
Task 2.3 `3677fee`. One bounded writer implemented an isolated profile; a separate
reviewer checked requirements, source, commands, complete runtime maps and checkpoint.
Root owns actual executions and reviewed D-026 integration-only publication.

### Scope, review and executed validation

The previous 59,890-byte lifecycle Rust file remains an exact prefix. New helpers
and `repeated_cycles_production_lifecycle` are appended; the old fixture and positive/
pause/loss bodies are unchanged. The selector `repeated-cycles` requires three exact
ordered cycle summaries, five explicit Clock stimuli, both exact rejection records,
typed final accounting and honest loaded/executed program identities. Five new
runner regression groups reject changed arithmetic, omitted/duplicate/mixed markers,
wrong nested types, false Clock provenance and incorrect guard results.

Root executed `python3 -I -B tools/test_validate_production_lifecycle.py -v`: all
24 tests passed on the first execution. Before execution, separate review corrected
two draft oracle mistakes (a nonzero cooldown-reward field in the completion summary
and colliding Clock/rejection phase labels); executed versus loaded programs were
also distinguished. These were pre-execution review findings, not runtime failures.

The first actual locked/offline Bank build failed at the new JSON evidence macro:
an indexed array value required parentheses. Sources `source-r/`, the first build
logs/result and all earlier records remain intact. The writer changed exactly that
expression; root and reviewer confirmed no oracle/economic value changed. Python
runner/test sources stayed identical, so their 24 passing results were retained
rather than rerun. A new `source-r2/` freeze and output directory preserved both
attempts. The corrected R2 Bank build passed, but actual runtime R failed after all
three economic cycles and final cumulative assertions, at the final old cache check:
`original tombstone is on the registered ancestor branch`. R remains FAIL; root and
reviewer separately audited its 4,654 account records / 66 snapshots and all economic
results as partial evidence only.

The old cache observer serves the epoch-progressing profile. This compact Bank stays
at slot one, where the unchanged singleton Closed ProgramData metadata entry has
deployment/effective slot one: its relation is Equal, not Ancestor. Pinned BankForks
and loaded-program source accepts Equal or Ancestor visibility. This classification
does not mean the executable PIV program is closed. A separate review approved an
isolated read-only observer: exact current Bank pointer, root/highest/cache-root,
ancestor set, singleton entry/type/slots/environment. It observes after the first
heartbeat and after all cycles. No cache clear, injection, account/fork mutation,
economic oracle change or old-profile assertion removal occurs. Reversing the
observer-only change reconstructs the R2 source exactly. An intermediate placement
after initialization was moved after the first heartbeat before any execution; both
source correction receipts remain preserved.

Root then froze `source-r3/`, built successfully and executed actual runtime S to
completion without diagnostics. Both cache observations passed with the same exact
entry and current-slot visibility. Root and reviewer compared every R/S economic,
account, transaction, commit and CPI record exactly; only the two read-only cache
observations and final completion marker are new. S is successful new execution,
not a relabelling of failed R. Python runner/test sources stayed unchanged since the
24-test first pass; these tests were not redundantly rerun.

Actual root wrappers: `run_runner_tests.py 1`, `prepare_profile.py r`,
`run_bank.py r build` (failed), `prepare_profile_retry.py r2`,
`run_bank_retry.py r build` (passed) / `r run` (failed), then
`prepare_profile_retry2.py r3` and `run_bank_retry2.py r build` / `r run` (S passed).
R2 binary SHA256: `d40e9a52e564bedbd01bc133ddf0ea5172d5eca4a07e88e404887c1feed493f1`.
Final R3 binary SHA256: `271dde864207f7c8e680a3af0e1861b1526458c8be5cf1756cbf29964be7c8db`.
Metadata/dependencies match prior O; pinned caches are reused, not rebuilt afresh.
The first freeze contains 277 inputs / 8,411 prior records; R2 contains 277 / 8,707;
final R3 contains 277 / 9,019 including failed attempts, source copies and immutable
review parsers. Later execution/review/publication records are preserved separately.

All 128 production freeze inputs, caller source, dependencies/layouts, accepted
economics and the production 32-KiB allocator are unchanged. The same five ELFs
remain PIV `f62237f6…`, caller `284293d9…`, Token 8.0.0 `7cfba90f…`, SPL pool 2.0.3
`314cefab…` and Stake 5.1.0 `3d2d39c5…`. S actually invokes PIV, System, Token, pool
and the synthetic initializer caller. Stake is loaded but not executed. O/P/Q and
all prior delayed/pause/loss results are hash-verified retained evidence, not fresh
executions. The same distinction applies to the prior 595 host tests +1 doctest/
eight gates/strict PIV SBF and 11 caller tests/check/docs/SBF. No new production
bytecode, dependency installation or economic rule is introduced.

### Exact production cycle and independently derived accounting

Actual initialization starts with absent PIV accounts; production SOL/Jito intake
and bootstrap recognize one SOL plus 19 billion Jito units, setting HWM 20 SOL.
Each of three rounds receives a declared external donor transfer of 100 SOL into
the pool reserve followed by real SPL maintenance, and an explicit two-SOL pending
contribution. Current pool valuation produces 1.9 SOL historical yield each time:
1.121 SOL HTFP, 0.3705 SOL Team, 0.3705 SOL ordinary compound and 0.038 SOL KIF.
No PIV or pool accounting bytes are manually changed to manufacture these outputs.

Actual liquid `PIV1PD01` preparation uses 1.5295 SOL of physical pending custody,
while its recorded contribution remains the full two SOL. Production settlement
and subsequent integration move the remaining 0.4705 SOL and add the full two SOL
to HWM. There is no withdrawal leg or new cooldown yield in this profile.
Complete account oracles include all unchanged accounts, fixed recipients, rewards,
prior-completion optional serialization, sequence, operational rent and exact fees.
The prior summary expands subsequent active-round offsets by 88 bytes; the new
oracles encode this explicitly rather than reusing first-round-only offsets.

Period zero and period one snapshot no active guardians. Current KIF plus prior
carry therefore compounds/carries 19,000,000 / 19,000,000, then 28,500,000 / 28,500,000
lamports. The second calculation includes all prior collective carry, not only the
new allocation. Period two snapshots three active guardians: 66,500,000 available
credits each 22,166,666 and leaves two collective carry lamports. Heartbeats after
the first and third snapshots never change their frozen eligibility or grant
retroactive claims. Old period-zero activity does not count in period one.

HWM progresses 22,389,500,000 → 24,788,500,000 → 27,159,000,000. Final protected
native principal is 2,459,000,000 plus 19 billion token units worth 24.7 SOL.
Pending SOL/tokens are zero; earned KIF liability 66,499,998 plus carry two exactly
matches KIF custody. Total zero-active KIF compound is 47,500,000. No KIF claim is
executed in S; previously validated claim evidence remains separate.

### Synthetic Clock, cadence, atomicity and evidence limits

The pinned Bank warp helper retains its parent's timestamp, so epoch warping alone
would not prove the required ten-day/30-day rules. S instead explicitly uses the
canonical `Bank::set_sysvar_for_tests` helper to set Clock timestamps. Full maps
allow only the eight timestamp bytes to change; slot 1, epoch 0, other Clock fields,
capitalization and all economic accounts remain unchanged, and the Bank cache
agrees. PIV reads the actual Clock account/syscall, not a supplied instruction time.
This is a declared local input, not natural walltime or public-cluster progression.

First preparation at 1,728,000 settles one day later at 1,814,400. At 2,591,999 the
next preparation rejects exact 6038 before CPI; at 2,592,000 it succeeds, exactly
ten days from preparation and only nine from settlement. The third preparation
is at 5,184,000. These select half-open KIF periods 0, 1 and 2. A second preparation
while already active rejects 6034. Both failures retain only transaction fees;
identical transaction replay changes no account and charges no additional fee.

Selected preparation, zero-active settlement and final pending integration each
execute a late outer failure, prove full rollback with fees retained, reject
identical no-fee replay, and succeed on a distinct retry. This is not proof of a
callee that mutates and then fails internally. Root and separate review reconstruct
all success and failure maps independently from literal rules and pinned layouts,
including continuity between snapshots. Initialization compares the 16 actual
created PIV accounts to retained reviewed O initialization plus current payer/fee
accounting; no production model is imported by the independent parser.

Root and separate review audited 4,654 full account records /
66 snapshots, 30 transaction/commit pairs (25 distinct nonce
transactions plus five replay records) and 71 CPIs. All complete maps, five
Clock-only stimuli, three late-outer rollback cases, two guard/replay cases and
distinct successful retries pass. R/S comparison preserves the exact economic trace.
Initializer uses 1,067,516 CU; maximum unsigned packet remains
1,544 bytes. Per-round preparation/settlement/integration CU are:

| Round | Preparation CU | Settlement CU | Integration CU |
| --- | ---: | ---: | ---: |
| 0 | 274,700 | 411,658 | 258,330 |
| 1 | 274,731 | 411,383 | 257,698 |
| 2 | 274,655 | 400,180 | 258,104 |

M4 still needs a demonstrated internally mutating callee failure and independent
requirement-to-evidence closure, including any concrete coverage/implementation
blocker it identifies. The 24-hour valid-insufficiency runtime case is not added
by S; retained host proof must not be relabelled as runtime execution. Governed
recovery exit remains unimplemented/unproved. Synthetic external genesis/governance,
proposal non-consumption, synthetic Clock, unsigned transport/signatures and supported
adapter-profile limits remain. No general cluster readiness, live yield or new
Stake execution is inferred. M5 has not started and remains unauthorized.

Post-S available disk was 2,398,650,368 bytes, about 251 MB above the 2-GiB
reserve. Check actual remaining capacity before a new build/run; do not delete
retained evidence to create headroom.

Preserve all A–S attempts, all three build source freezes and logs, prior ELFs/receipts
and four recovery archives. Evidence root: `/tmp/piv1-t251-cycle-review-20261015-a`;
Git and `publication.json` identify the reviewed integration checkpoint. Main and
protected Task 2.3 stay unchanged. No cleanup/install, secrets, signing, live RPC,
deployment, real fund movement, key creation or authority transfer occurred.
Save/end this bounded session economically. On resume verify actual refs/worktree
and disk capacity; retain the mandatory 2-GiB reserve. D-034 requires a reviewed
completion report and STOP before M5 once M4 is fully technically validated.

## Continuation interrupted by platform filtering (2026-10-15 UTC)

**Continuation checkpoint — platform interruption (2026-10-15 UTC).**
Clean takeover as jerem was integration `936a0db67ca0835df951857adec5af247bf2f811`.
Runtime S remains the latest technically validated execution. The next bounded
validation was assessed in read-only source review, but the delegated writer turn
was rejected by the platform's cybersecurity filter before any repository edit.
The founder reported a Daybreak/Astra availability message. This is a platform
restriction, not a demonstrated PIV1 failure; its application-level cause is unknown.
No rejected delegation was retried, routed through another model or executed by root.

No new implementation, build, test or runtime execution resulted. Prepared wrappers,
the command-design review and an incomplete, unexecuted independent parser remain in
`/tmp/piv1-t251-internal-failure-review-20261015-a`; `interruption.json` records the
preservation check and actual state. These drafts are not approved execution evidence;
do not run them automatically. Both subagents have stopped. The concrete internal
failure coverage and independent requirement-to-evidence closure remain open.
Ordinary requirements/documentation review may proceed without bypassing the blocked
operation. M4 is incomplete; D-034 still forbids starting M5 without explicit founder
authorization. Main stays `2f8d083`; no acceptance or live-operation authority changes.
No cleanup/install, secrets, signing, key creation, deployment, real funds or authority
transfer occurred. Recheck disk capacity and retained evidence before resuming.

## M4 requirement-to-evidence reconciliation (2026-10-15 UTC)

**Current continuation — M4 evidence reconciliation (2026-10-15 UTC).**
Clean takeover as jerem was integration `1bb3db5fcc99c7bc09c9771c475c3f89b3ddf575`.
This documentation-only review reconciles existing evidence; runtime S remains the
latest validated execution. No implementation, build, test or runtime was rerun.
Root verified 386 tracked inputs and 9,361 retained records; separate review bound
the unchanged 128 production/host inputs and retained 595 tests +1 doctest/eight
host gates, and checked O/N/P/Q/S runtime receipts. These are retained results.

The current register in [the test plan](PIV1_TEST_PLAN.md) supersedes its mixed-age
reference table for closure planning. Two normal economic branches still lack actual
production-runtime evidence: P-029–P-031 valid insufficiency with the 24-hour retry
clock, and P-035 positive withdrawn-stake cooldown reward through its later split.
Both have host/model tests. O's three-lamport external reserve appreciation is not
withdrawn-stake cooldown reward; S has no withdrawal or valid-insufficiency sequence.
These are evidence gaps, not demonstrated production defects or new economic rules.

Next permitted work is bounded normal functional coverage of valid insufficiency,
then positive cooldown reward, each with separate review and proportionate gates.
The platform-blocked internal-failure operation remains stopped; its drafts are not
execution evidence. Do not waive it or reroute the rejected operation. Final independent
requirement-to-evidence closure remains outstanding; this inventory is not exhaustive
code compliance, an independent professional audit, or M4 completion. Existing adapter,
recovery-exit, synthetic governance/Clock and unsigned transport limits remain explicit.
D-034 still requires STOP before M5 after actual M4 completion; M5 is not authorized.
Main remains `2f8d083`; technical validation and founder acceptance remain separate.

Baseline free disk was 6,054,473,728 bytes; preserve the 2-GiB reserve and all prior
evidence. No cleanup/install, secrets, signing, key creation, deployment, real funds
or authority transfer occurred. Root owns reviewed integration-only publication;
Git and `/tmp/piv1-t251-evidence-review-20261015-a/publication.json` record its identity.
Verify actual refs/worktree, available capacity and this checkpoint on resumption.

The earlier mixed-age table combined Task 2.11 missing-runtime statements with later
Task 2.51 additions. The new current register preserves that reference history while
separating exercised local profiles, source-bound host evidence, remaining local
coverage and later live/operational requirements. It does not claim every accepted
requirement has undergone fresh source review or silently move obligations to M5.

P-029–P-031 inspection includes
`dynamic_minimum_valid_insufficiency_zero_inverse_and_retry_clock_only` in
`withdrawal_preparation_execution.rs`: host-injected valid insufficiency, immediate
cooldown rejection, a sufficient-attempt bypass and a retry after 86,400 seconds.
Host malformed/source/rent cases check no cooldown change. S proves ten-day cadence
but finishes with no valid-insufficiency timestamp; it cannot close the 24-hour case.

P-035 host inspection includes the reward/rent matrix in
`withdrawal_finalization_execution.rs`, the 97-lamport reward settlement fixtures,
post-settlement integration coverage and
`cooldown_reward_becomes_next_cycle_yield_and_is_split_on_the_next_open` in
`legal_lifecycle.rs`. The actual positive Bank profile asserts zero cumulative
withdrawn-stake cooldown rewards. Its later three-lamport pool reserve input cannot
stand in for a positive withdrawn-leg reward, and S has no withdrawn leg.

No blocked test/parser was resumed or executed. Only documentation changed, with a
bounded writer and independent review. Current program/caller sources, dependencies,
layouts, economic decisions and all five ELFs are unchanged. Source/evidence hashes
and `git diff --check` are the applicable checks for this documentation-only change;
no new passing test count is claimed. Normal integration publication preserves main,
protected Task 2.3, founder acceptance and all live-operation limits.

## Runtime U — normal valid-insufficiency retry clock (2026-10-15 UTC)

**Historical continuation — valid-insufficiency runtime U (2026-10-15 UTC).**
Task 2.51 remains **IN PROGRESS / INSUFFICIENCY RETRY CLOCK VALIDATED**.
Clean takeover as jerem was integration `afc388a0be53a63001ad8a4584c03a4b8ad8e13a`.
One bounded writer and separate review added an isolated normal functional profile.
Root passed 30 runner tests, the corrected locked/offline Bank build and actual U.
The first build failed on two JSON integer literals inferred as i32; only explicit
u64 suffixes corrected them. That failed build and its source freeze remain intact.
Production/caller, dependencies, layouts, economics, allocator and five ELFs are
unchanged; the entire previous 76,812-byte Rust test source is an exact prefix.
Prior 595 host tests +1 doctest/eight gates, strict SBF, caller 11 tests/check/docs/SBF
and O/N/P/Q/S executions are retained evidence, not rerun in this continuation.

U completes a liquid cycle at timestamp 864,000, then proves P-029–P-031 through
actual production withdrawal preparation and the real Stake minimum query. Valid
insufficiency at 1,728,000 changes only its Config timestamp plus transaction fee,
without a new snapshot or funding. Immediate/+86,399 attempts reject 6039; +86,400
records the next valid insufficiency. The existing ten-day clock stays 864,000;
its own one-second-early attempt rejects 6038. Invalid ABI versions leave both
clocks unchanged. At 1,814,401 sufficient preparation succeeds despite the last
insufficiency being one second old. Explicit donor reserve inputs and one pending
lamport preserve the exact HWM partition-floor guard; no economic guard is relaxed.

Root and reviewer audit 3,944 complete account records / 56 snapshots, 25 transaction/
commit pairs (18 distinct plus seven replays), 59 CPIs and both actual event payloads.
Five rejections retain only fees; seven identical replays charge no new fee. Six
canonical synthetic Clock changes preserve all other accounts. U ends WithdrawalActive,
sequence one, with no leg created: HWM 22,389,500,000, proposed HWM 22,760,000,000,
pending/escrow one lamport and KIF carry 19,000,000. No second completed withdrawal
cycle is claimed. Initializer uses 1,067,516 CU; largest packet is 1,544 unsigned bytes.

Next close positive withdrawn-stake cooldown reward through its later-cycle split.
The separately platform-blocked internal-failure case remains stopped/unwaived;
final independent requirement-to-evidence closure and demonstrated blockers remain.
Synthetic genesis/governance/Clock, proposal non-consumption, unsigned transport,
adapter and governed recovery-exit limits persist. M4 is NOT complete. D-034 requires
STOP before M5 after full M4 validation; M5 has not started and is not authorized.
Main stays `2f8d083`; protected Task 2.3 stays `3677fee`; no founder acceptance inferred.

Root owns reviewed integration-only publication; Git and
`/tmp/piv1-t251-insufficiency-review-20261015-a/publication.json` record its identity.
The final U freeze binds 277 inputs / 9,690 prior records; later execution/review/
publication receipts are retained separately. Preserve all earlier attempts, ELFs
and recovery archives. Runtime sampled free space reached a minimum of 5,750,239,232 bytes;
recheck capacity and preserve the 2-GiB reserve on resume. No cleanup/install, secrets,
signing, key creation, live RPC/deployment, real funds or authority transfer occurred.

### Actual commands, artifacts and retained failed attempt

Evidence root: `/tmp/piv1-t251-insufficiency-review-20261015-a`.
Root executed `run_runner_tests.py 1` (30 PASS), `prepare_profile.py u`, and
`run_bank.py u build`. The first build at
`/tmp/piv1-bank-smoke-production-lifecycle-build-t251-u` failed solely because
JSON completion literals 22,389,500,000 and 22,760,000,000 defaulted to i32.
`source-u` and its original pins/diagnostics remain. The bounded writer added only
two u64 suffixes; independent review verified the exact byte replacement.

Root then executed `prepare_profile_u2.py u2`, `run_bank_u2.py u2 build` and
`run_bank_u2.py u2 run`. Actual commands use the existing pinned compiler/cache,
`--locked --offline`, one job/thread and guarded resources; no new dependency
installation or full dependency-cache rebuild is claimed. Corrected build-u2
passes without diagnostics and preserves Bank binary SHA-256
`6734670e6bc62db587bac8cbaebf2ea1301fd97a40b8ba008751cac30ef65d00`.
Runtime U passes on its first execution at
`/tmp/piv1-bank-smoke-production-lifecycle-run-t251-u`: stdout 209,444,109 bytes,
SHA-256 `c6be11adf6422f4c833631ff806ccc53e0d8e2ba7aaa7595ff0ca323fc31f36c`;
stderr is empty. Runtime lasts 19.30 seconds under the existing network-denied
local harness. T was never executed in the separate interrupted operation; U
uses a fresh identity and does not reuse or resume those drafts.

Root and separate reviewer execute the immutable `review_insufficiency_runtime.py`
(SHA-256 `f41e893cc3c0b3ed5a09bd009ddb4e02bfc79944c26892ad7b906de282a0e91d`)
against actual U logs into `root-runtime-u.json` and `reviewer-runtime-u.json`.
All 13 success maps, five rejection fee-only maps, seven no-fee replay maps and
six Clock-only changes pass. The parser checks transaction bytes, CPI identities,
minimum response, actual program event logs, complete Config/round layouts and
final custody. Final source/preservation checks and staged review bind publication.

### Numeric scope and remaining limitations

The primer completes production initialization, 1-SOL/19-billion-token intake,
bootstrap, liquid preparation, settlement and full pending integration. HWM becomes
22,389,500,000; principal is 19 billion tokens plus 1,489,500,000 lamports. Zero-active
KIF compounds 19,000,000 and retains carry 19,000,000; the late heartbeat is not
retroactive. A real donor reserve transfer and pinned pool update give total/supply
1,150/1,000 SOL-equivalent units, gross 950,000,000 and outgoing 764,750,000.
Target 665,000,000 is below minimum 869,565,218 derived from the real 1-SOL Stake
minimum. Only the valid attempt timestamp changes; actual 64-byte events match.
No snapshot, funding, rent reservation or contribution reclassification occurs.

The sufficient stimulus is another explicit 50-SOL reserve transfer plus one
ordinary pending lamport, not unexplained yield or changed PIV state. At pool
ratio 1.2, target 1,274,583,333 and minimum 833,333,334 allow one useful leg;
protected floor is 1,529,347,049 and retained historical value exactly covers proposed
HWM 22,760,000,000. Production executes the Stake query then exactly one System
transfer from pending to escrow. The full pending ledger remains one. Existing
HWM and 50,000,000 operational spendable lamports remain unchanged, and no new
leg/Stake account exists. Valid insufficiency uses 208,685 / 208,646 CU; sufficient
preparation uses 341,409 CU. Initializer remains 1,067,516 CU.

U proves this normal timing branch, not every malformed/source/minimum/fee case.
It does not execute a second leg/finalization/settlement or positive withdrawn-stake
cooldown reward, governed recovery exit, actual Squads proposal consumption or real
signatures/transport. Those distinctions, M4 closure and D-034 remain explicit.
No new production SBF compilation was needed for unchanged production bytecode;
the host/SBF counts above are retained evidence, not freshly executed results.

## Runtime V — positive cooldown reward and later split (2026-10-16 UTC)

**Current continuation — positive cooldown reward runtime V (2026-10-16 UTC).**
Task 2.51 remains **IN PROGRESS / COOLDOWN REWARD AND LATER SPLIT VALIDATED**.
Clean takeover as jerem was integration `c1b0a2210b48cafa4f488957e8912c46d6c8248a`.
One bounded writer and separate review added an isolated normal functional profile.
Root passed all 36 runner tests, one locked/offline Bank build and actual runtime V
on their first executions. Draft test-oracle corrections preceded execution; no
production defect or failed build/run is claimed. The previous 90,843-byte Rust
lifecycle source remains an exact prefix. Production/caller sources, dependencies,
layouts, economics, allocator and five ELFs are unchanged. Prior 595 host tests
+1 doctest/eight gates, strict SBF, caller 11 tests/check/docs/SBF and O/N/P/Q/S/U
executions are hash-verified retained evidence, not rerun.

V completes two production cycles. Immediately after real withdrawal/deactivation,
a declared balanced synthetic reward observation adds 10,000 lamports and delegated
stake to leg zero, debiting the existing external donor. Original rent, credits,
authorities, PIV accounts and Bank capitalization stay unchanged. This is an abstract
accrued-reward input, not native inflation, vote-credit or live-yield execution.
Actual Stake rejects premature withdrawal and permits out-of-order finalization at
epoch 14. Both legs return exact original rents and close all temporary accounts.
The separate three-lamport external pool-reserve appreciation remains explicit.

The 10,000 reward is excluded from the first fixed payouts and historical principal;
first pending integration leaves HWM exactly 121,955,000,003 and next-cycle yield
10,000. A canonical synthetic Clock then permits the next liquid cycle: gross 10,000
splits HTFP 5,900 / compound 1,950 / team 1,950 / KIF 200, without double counting.
Historical SOL increases by the compound only at final integration. Final Idle HWM
is 121,955,001,953, pending and next-cycle yield zero, earned KIF liability
216,363,836 and KIF carry zero. No KIF claim or second-cycle withdrawal leg is claimed.

Root and separate review audit 8,750 account records / 120 full snapshots, 59
transaction/commit pairs (54 distinct plus five replays), 110 CPIs and nine actual
PIV event payloads. All 36 successful transaction maps and 18 fee-only failures
pass; five identical replays charge no new fee. Selected late-outer failures and
distinct successful retries are preserved. Initializer uses 1,067,516 CU; largest
packet remains 1,544 unsigned bytes. This is bounded local runtime evidence.

Next perform independent requirement-to-evidence closure and address demonstrated
blockers within existing limits. The separately platform-blocked internal-failure
case remains stopped/unwaived; do not resume or reroute its drafts. Synthetic
genesis/governance/Clock, proposal non-consumption, unsigned transport, adapter and
governed recovery-exit limits persist. M4 is NOT complete. D-034 requires STOP before
M5 after full M4 validation; M5 has not started and is not authorized. Main stays
`2f8d083`; protected Task 2.3 stays `3677fee`; no founder acceptance is inferred.

Root owns reviewed integration-only publication; Git and
`/tmp/piv1-t251-reward-review-20261016-a/publication.json` record its identity.
The V freeze binds 277 inputs / 10,027 prior records; subsequent execution/review/
publication receipts remain separate. Preserve all earlier attempts, ELFs and recovery
archives. Runtime sampled free space reached a minimum of 5,112,201,216 bytes;
recheck capacity and preserve the 2-GiB reserve on resume. No cleanup/install, secrets,
signing, key creation, live RPC/deployment, real funds or authority transfer occurred.

### Actual commands and immutable evidence

Evidence root: `/tmp/piv1-t251-reward-review-20261016-a`.
Root executed `run_runner_tests.py 1` (36 PASS), `prepare_profile.py v`,
`run_bank.py v build` and `run_bank.py v run`. The guarded build uses the existing
pinned compiler/cache with `--locked --offline`, one job/thread; no installation or
full dependency-cache rebuild is claimed. Build V passes without compiler diagnostics.
The Bank binary SHA-256 is
`3dd5f106a3706adbc5870f6b12232dc7cd02893476552ed09cb07be5a1aeda8b`.
The selected run is exactly `cooldown_reward_production_lifecycle`, profile
`cooldown-reward`, in `/tmp/piv1-bank-smoke-production-lifecycle-run-t251-v`.
(The build command's default profile label is `positive`; its single binary contains
all profiles and the run explicitly selects the reviewed reward test.)
Runtime stdout is 449,050,143 bytes, SHA-256
`228d2d444d64bc34a5c58188af81e63b4f98032ed7a99771ca52a1708c2a0182`;
stderr is empty. Guarded runtime takes 47.90 seconds with network sockets denied.

Root and separate reviewer each execute the immutable `review_reward_runtime.py`,
SHA-256 `4badf1972041de752b70637de35dfcaf61b70a36ce64e208207d0c29218fab76`,
against actual logs into `root-runtime-v.json` and `reviewer-runtime-v.json`.
It independently reconstructs every successful transaction's full account map,
checks failure/replay maps, transaction bytes, actual CPI identities, authenticated
PIV event payloads, reward input, PIV/leg preservation across epochs and both cycles.
The canonical Clock-only input preserves every other account. No blanket claim
that every Bank-created account stays unchanged across epoch warps is made.
`root-final-source-guards.json` verifies 128 unchanged production/host inputs,
277 frozen source inputs, all preserved records and five unchanged program ELFs.
Staged-tree review and publication receipts bind the resulting integration commit.

Preexecution review corrected two draft Rust expectations: historical SOL changes
at final integration, not preparation; the first completion summary records the
10,000 cooldown reward. Python regressions use independent literal transcripts,
not values imported from the validator. These corrections preceded all tests/builds;
no failed V execution or production defect is inferred. The six new runner tests
reject missing/reordered stages, wrong test/profile, unbalanced reward or donation,
rent/principal misclassification, premature distribution, double counting and false
KIF claims. All prior 30 runner regressions also pass.

### Exact economic and runtime scope

The profile begins with actual initializer, 100-SOL/19-billion-token intake,
bootstrap HWM 119 SOL, operational funding and protected principal staking.
Declared reserve yield, heartbeats and contributions establish the first fixed
snapshot; pending-after-snapshot intake remains 200M SOL units plus 500M token units.
Two actual withdrawn delegations initially hold 5,000,000,000 and 3,608,636,361
lamports. Immediately after deactivation at epoch zero, the paired synthetic
observation adds 10,000 to leg zero's lamports and delegated stake and debits the
existing external System donor. All other Stake bytes, original rents, authorities,
credits, PIV state, full-map lamport sum and Bank capitalization are unchanged.
Canonical Bank storage updates the stakes cache; no readiness override occurs.
This abstract observation does not calculate inflation, update vote credits or
prove provenance of a live reward. It is distinct from an unsupported naked donation.

Real Stake execution rejects active/epochs 1–13 finalization and accepts companion
leg one at epoch 14, followed by rewarded leg zero. Total original rent recovered:
4,565,760 Stake plus 5,442,720 metadata lamports; all four temporary accounts close.
First payouts remain HTFP 6,382,727,270 / team 2,109,545,453 / KIF 216,363,636.
The separate three-lamport donor-to-pool reserve appreciation is unchanged from O;
it is not part of withdrawn-stake reward. First pending integration yields historical
111,608,750,001 tokens worth 121,755,000,001 plus historical SOL 200,000,002,
exactly HWM 121,955,000,003. Physical principal SOL is 200,010,002: the additional
10,000 stays separately recorded as next-cycle yield and cumulative cooldown reward.

Canonical synthetic timestamp 2,592,000 and a period-one heartbeat enable a later
liquid production cycle at epoch 14. Pure gross 10,000 has no additional historical
surplus or pending contributions. Preparation transfers 8,050 from principal to
escrow, records prior carry 10,000 / used 8,050 / remaining 1,950 and creates no leg.
Settlement pays 5,900 HTFP / 1,950 team / 200 KIF and raises protected HWM by 1,950.
Only final integration adds that compound to historical SOL, reaching 200,001,952.
Final Idle sequence one / next sequence two has HWM 121,955,001,953, zero pending,
zero next yield and earned KIF liability 216,363,836, with no claim or KIF carry.

Four selected late-outer failures (rewarded finalization, first settlement,
first integration and later preparation) roll back completely except transaction
fees; identical replay has no new fee, and distinct retries succeed. The initial
active-finalization rejection also has identical no-fee replay. These are not the
separately blocked internally mutating callee-failure operation. Nine exact actual
event payloads include attempted events whose outer transactions roll back, clearly
distinguished from committed events. CU: initializer 1,067,516; rewarded finalization
440,348; first settlement 400,193; first integration 279,949; later preparation
274,728; later settlement 400,088; later integration 249,000. Largest unsigned packet
is 1,544 bytes. No live transport/signature or general cluster-readiness claim follows.

This closes the bounded normal P-035 runtime branch under the declared input.
The platform-blocked internal-failure gap, independent final requirements closure,
demonstrated blockers and existing governance/recovery/adapter limits remain.
M4 is not complete; M5 has not started and remains unauthorized under D-034.

## A-004 later compounding closure review (2026-10-18 UTC)

**Current continuation — A-004 closure review (2026-10-18 UTC).**
Clean takeover as jerem was `705e1518e7de17e31d3e5ef2c6c92d4a89683166`,
matching local/remote main and integration; D-036 publication is complete there.
One bounded writer and a separate reviewer examined A-004's sixth boundary,
later principal SOL/Jito compounding after completed settlement and pending
integration. The scoped result is documented in
[the A-004 closure review](PIV1_M4_A004_CLOSURE_REVIEW.md).
Production has a guarded path and host coverage for Idle with prior completion;
no production defect was demonstrated. Retained actual O/S/U/V messages have no
post-completion principal deposit: O/V deposits precede their first cycle, S/U
have none. Initial staking does not establish this explicitly later boundary.
This is a concrete remaining M4 runtime coverage gap, not an M5 packaging task.

Next extend a retained completed-cycle fixture with actual protected principal
compounding at its current pool ratio. Parameterize the test helper's initial-1:1
minimum and derive independent exact expected custody/accounting; preserve HWM,
pending, KIF, original rents and the prior completion summary. Cover selected
late-outer rollback, identical no-fee replay and a distinct successful retry.
No production guard, economic rule or dependency change is justified by this review.

No tests, builds or runtime were executed in this documentation/source review.
Root verified 386 tracked files, 277 pinned inputs, 128 unchanged production/host
inputs, five ELFs and 10,350 retained records. O/S/U/V and prior host/SBF results
remain retained evidence. Runtime V remains the latest validated execution.
The review covers one logical boundary, not exhaustive compliance or final M4
closure. The platform-blocked internal-failure case remains stopped/unwaived;
no drafts were resumed or rerouted. Other documented limits and D-034 persist:
M4 is incomplete, M5 not started/unauthorized, no founder acceptance inferred.

This continuation publishes only reviewed integration. Main stays `705e151`,
protected Task 2.3 `3677fee`; Git and
`/tmp/piv1-m4-closure-review-20261018-a/publication.json` identify the checkpoint.
Preserve all evidence and the 2-GiB reserve; baseline free space was 5,220,188,160
bytes. No cleanup/install, secrets, keys/signing, live RPC/deployment, real funds
or authority transfer occurred. Save/end this bounded review session economically.

## Runtime W — principal compounding after completed cycles (2026-10-18 UTC)

**Current continuation — later principal compounding runtime W (2026-10-18 UTC).**
Task 2.51 remains **IN PROGRESS / A-004 LATER COMPOUNDING RUNTIME VALIDATED**.
Clean takeover as jerem was integration `9abfd5c16ee94863ebbcb1df3924d866c8fe750a`;
main remains completed D-036 `705e151`, protected Task 2.3 `3677fee`.
One bounded writer and separate review closed the specifically identified A-004
boundary-six runtime gap: protected principal compounding after a completed cycle.
Root passed 42 runner tests, one locked/offline Bank build and actual runtime W
on their first executions. A draft parser-test count was corrected from 34 to 29
before execution; no production defect or failed build/run is claimed. The previous
117,637-byte Rust source remains an exact prefix. Production/caller sources,
dependencies, layouts, economics, allocator and all five ELFs are unchanged.

W executes S's three liquid cycles, then performs a protected principal deposit
from their actual completed Idle state at the appreciated 13:10 pool ratio.
2,458,999,998 lamports mint 1,891,538,460 JitoSOL units; two economic lamports remain.
Final historical tokens are 20,891,538,460 and HWM stays exactly 27,159,000,000.
The full prior completion summary, next sequence three, pending, next-cycle yield,
original rents, earned KIF liability 66,499,998 and collective carry two are unchanged.
The bounded amount preserves book value exactly; it is not a permanent reserve
policy or proof of general rounding-loss/nonzero-fee adapter support.

Caller minimum above quote rejects exact 6032; amount above historical SOL rejects
6028, both before CPI/events with fee-only maps. A late-outer failure rolls back the
successful protected pool/PIV operation except fees; identical replay adds no fee,
and a distinct retry succeeds. Both actual PendingSolStaked payloads and protected
pool/System/Token CPI instructions are checked. No internal-callee failure is claimed.
Root and reviewer audit 5,435 account records / 77 snapshots, 37 transaction/commit
pairs (29 distinct plus eight replays) and 77 CPIs. W's S prefix exactly matches the
retained reviewed execution; the 781-record/11-map suffix is independently derived.
Success uses 191,690 CU; initializer 1,067,516 CU; maximum packet 1,544 unsigned bytes.

Prior 595 host tests +1 doctest/eight gates/strict SBF, caller 11 tests/check/docs/SBF
and O/N/P/Q/S/U/V are retained evidence, not separately rerun. S's economic sequence
is newly executed within W; matching its retained trace is not a new independent
rederivation of every S assertion. Synthetic genesis/governance/Clock, unsigned
transport, proposal non-consumption, supported adapter and recovery-exit limits persist.
Next continue independent requirement-to-evidence closure and demonstrated blockers.
The separately platform-blocked internal-failure case remains stopped/unwaived;
its drafts were not resumed or rerouted. M4 is NOT complete. D-034 requires STOP
before M5 after full M4 validation; M5 has not started and remains unauthorized.
No founder acceptance or main-publication authority is inferred.

Root owns reviewed integration publication; Git and
`/tmp/piv1-t251-compound-review-20261018-a/publication.json` record its identity.
The W freeze binds 277 inputs / 10,387 prior records; later execution/review/publication
receipts remain separate. Preserve all earlier attempts, ELFs and recovery archives.
Runtime sampled minimum free space was 4,852,903,936 bytes; recheck capacity and
preserve the 2-GiB reserve on resume. No cleanup/install, secrets, keys/signing,
live RPC/deployment, real funds or authority transfer occurred. Save/end this session.

### Execution, review and exact retained inputs

Evidence root: `/tmp/piv1-t251-compound-review-20261018-a`. Root executed
`run_runner_tests.py 1` (42 PASS), `prepare_profile.py w`, `run_bank.py w build`
and `run_bank.py w run`. Fresh outputs are
`/tmp/piv1-bank-smoke-production-lifecycle-build-t251-w` and
`/tmp/piv1-bank-smoke-production-lifecycle-run-t251-w`. The build reuses the pinned
compiler/cache with `--locked --offline`, one job/thread, and no new dependency
installation. It passed without compiler diagnostics; this is not a fresh rebuild
of every dependency or production SBF. Bank binary SHA-256:
`611f82f908884f8af9094fcb55c30eed4088e8b4e84931b5262225d89c42d74c`.
The build's default profile label is `positive`; the reviewed run explicitly selects
`post-cycle-compounding` / `post_cycle_compounding_production_lifecycle` from the
shared binary. Runtime takes 27.09 seconds under the network-denied local harness.
Stdout is 287,952,620 bytes, SHA-256
`d5d699f6249c757bc15c0d2b7400cdddac317b82bd9c640827cd685436162f08`;
stderr is empty. No failed W build, test or runtime attempt occurred.

Root and reviewer each execute the immutable `review_compound_runtime.py`, SHA-256
`76c420b417308f17f9f5977637de684d4ca3e195d0da3b45070318c614bcb8ac`,
into `root-runtime-w.json` and `reviewer-runtime-w.json`. The actual full S-prefix
rows match the hash-bound retained S execution: 4,654 account records / 66 snapshots,
30 transaction/commit pairs and 71 CPIs. Only the fixture profile and standalone
completion marker differ; all economic rows, messages, logs, fees, cache/capability
observations and account maps remain exact. This reuses S's separately reviewed
independent oracle rather than claiming a fresh rederivation of every S assertion.

The added suffix is independently reconstructed from raw accounts/messages, not
completion summaries. It covers 781 account records / 11 snapshots, seven transaction/
commit pairs, six CPIs and two exact 72-byte PendingSolStaked payloads. Actual
protected `DepositSolWithSlippage` variant 25, System transfer and Token MintTo
instruction bytes/account order match in both the rolled-back and committed
attempts. All eight total identical replays charge no new fee; three belong to
the suffix. Complete before/after maps distinguish attempted events from commit.

The old 117,637-byte source has SHA-256
`dd0dc1d1e399ccf93394adf4fbccb6091c5ba3d5b09ec1bea4d11d1ba68eb3e5`
and remains an exact prefix. The copied S body is byte-identical after only test
and fixture-name normalization. The old fixture deposit helper is untouched;
a new local explicit-minimum helper requires the independently calculated output.
Six new runner regressions use literal expectations and reject missing/duplicate/
misordered stages, altered accounting/categories, wrong guards/events/rollback,
cross-profile completion and false test success. The one draft test count correction
34 to 29 occurred before execution and did not change production or runtime code.

### Actual post-completion conversion and limitations

The three-cycle primer ends with historical SOL 2,459,000,000 and 19,000,000,000
token units, pool total/supply 1,300,000,000,000 / 1,000,000,000,000, HWM
27,159,000,000 and Idle prior summary sequence two / next sequence three.
Exact conversion 2,458,999,998 -> 1,891,538,460 leaves historical SOL two and tokens
20,891,538,460. Pool total/supply become 1,302,458,999,998 / 1,001,891,538,460.
The ratio remains exactly 13:10 and total historical book value equals the unchanged
HWM. Only Config's two historical asset-unit fields change. Pending, next-cycle
yield, KIF liability 66,499,998, carry two, all six guardian records, rents, operational
custody, recipients and the entire completion header remain exact, excluding payer
fees. The two-lamport residual is a supported exact-value test choice, not a new
reserve policy or automatic full-queue conversion claim.

Minimum 1,891,538,461 for the valid amount rejects 6032 at 80,651 CU; amount
2,459,000,001 with zero caller minimum rejects the historical queue guard 6028 at
76,904 CU. Both have no CPI/event and only fees persist. A valid protected operation
followed by a failing outer instruction consumes 191,902 CU and rolls back all
non-fee changes; its identical replay charges nothing. The distinct valid retry
uses 191,690 CU and commits the exact oracle. These are ordinary local refusal and
late-outer cases, not the stopped internal-callee-failure operation. Initializer
remains 1,067,516 CU and maximum unsigned packet 1,544 bytes. Real PIV/System/Token/
pool/caller execution occurs; Stake is loaded but not executed in W. No live cluster,
signature, governance proposal consumption or naturally elapsed Clock claim is made.

Root final source/preservation checks bind the unchanged 128 production/host inputs,
277 frozen inputs and five ELFs. Shared documentation, exact staged-tree review and
publication receipts record closure of this bounded gap and preserve the remaining
M4 obligations. M5 has not started; D-034 remains binding.

## P-023 withdrawal-fee closure review (2026-10-18 UTC)

**Current continuation — P-023 withdrawal-fee closure review (2026-10-18 UTC).**
Clean takeover as jerem was integration `2a852d210e009ac8ccbaf25c7b20f87ed644c11b`.
Main remains completed D-036 `705e151`; protected Task 2.3 remains `3677fee`.
One bounded requirement analysis and separate code/evidence and documentary
refutation distinguish nonzero withdrawal fees from the principal-deposit
zero-fee restriction. See [the P-023 review](PIV1_M4_P023_CLOSURE_REVIEW.md).
The inspected production path supports fee-adjusted withdrawal and net beneficiary
allocation with fixed compound/HWM protection; no production defect is demonstrated.
The retained local lifecycle profiles do not establish the positive withdrawal-fee
branch. Next close one supported positive-fee multi-leg production lifecycle,
checking actual fee-token credits, burn/supply, per-leg rounding, net payouts and
preserved compound/HWM protection. This is not a demand for every fee configuration,
a fee-bearing principal deposit, or live Jito configuration validation.

Root and separate review independently decode the retained O/W pool account
records: all 187/77 snapshots have zero withdrawal fees. Historical Phase-0
positive-fee experimental PIV PDA CPI withdrawal remains valid evidence, but does not execute the
current PIV production beneficiary settlement.

This is source/evidence review only: no tests, builds or runtime were executed.
Root verified 387 tracked inputs, 277 pinned sources, 128 unchanged production/host
inputs, five ELFs and 10,702 retained records. Runtime W remains the latest validated
execution; previous test/runtime results remain retained evidence. Full independent
M4 closure is unfinished. The platform-blocked internal-failure case remains
stopped/unwaived; its drafts were not resumed or rerouted. M4 is incomplete;
D-034 requires STOP before M5 after full validation. M5 remains unauthorized.

Root owns reviewed integration-only publication; Git and
`/tmp/piv1-m4-fees-review-20261018-a/publication.json` record its identity.
Baseline free space was 4,847,226,880 bytes; preserve the 2-GiB reserve and all prior
evidence. No production/economic/dependency change, cleanup/install, secrets,
keys/signing, live RPC/deployment, real funds or authority transfer occurred.
No founder acceptance or new main-publication authority is inferred.
