# Task 2.51 — Real local production lifecycle

Status: **IN PROGRESS / COMPLETE LOCAL PAUSE PROFILE VALIDATED** (2026-10-13 UTC).

The October 13 runtime L continuation at the end of this report is current; earlier runs and
source/implementation claims below describe their historical evidence.

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
