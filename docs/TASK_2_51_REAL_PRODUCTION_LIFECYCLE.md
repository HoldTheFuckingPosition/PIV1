# Task 2.51 — Real local production lifecycle

Status: **IN PROGRESS / EXACT WITHDRAWAL-LEG HEAP BLOCKER** (2026-10-12 UTC).

The October 12 resumption at the end of this report is current; earlier runs and
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
