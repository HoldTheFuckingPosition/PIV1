# PIV1 technical pilot checkpoint

## Active checkpoint — M1 / Task 2.39 (2026-09-28 UTC)

Task 2.39 / D-030 M1 is **TECHNICALLY VALIDATED / PENDING FOUNDER ACCEPTANCE**.
Starting clean integration was `9b386cd9c45de99e6f84185f65718df74221f27b` as
jerem (uid 1001); main remains `4cc4ea11e87c2f1a2f85b9ed48358f2a881821b3`.
Root owns normal reviewed integration-only publication under D-026/D-030; Git
records its exact identity. No new main authority or founder acceptance is inferred.

The real production boundary now dispatches the full recipient-checked normalized
initializer with strict 315-byte `PIV1IN01` version-one data and fixed 35/34 roles.
Both recipient derivation witnesses belong to the exact original approved bytes.
The public legacy 313-byte model codec, existing claim/pending paths, state layout
and economics remain unchanged. Validation errors receive explicit new codes;
actual runtime/CPI errors retain their exact values. Ordinary host calls fail closed.

One delegated writer implemented the production boundary, focused tests and narrow
synthetic caller; separate source, runner, exact-artifact and evidence reviews passed.
Root personally passed **32 focused tests, 479 host tests +1 doctest/eight final
gates, nine runner regressions and one production Bank test/four profiles/twelve
messages**. Root and reviewer independently checked **786 complete account
records**, sixteen snapshots, 256 target-presence records and 296 CPI records.
The actual production ELF initializes fresh/prefunded and paused/unpaused fixtures
with witnesses 31/202. Literal state/Token bytes, original rent, 144-lamport native
normalization, late-failure non-fee rollback, 15000 fees, no-fee identical replay
and distinct same-Bank retry match. Host tests separately cover approval tampering,
reinitialization, arbitrary witnesses/runtime IDs and precise CPI error propagation.

The first SBF preflight stopped before compilation on an outdated libexpat hash.
Installed Ubuntu package 2.6.1-2ubuntu0.6 was independently verified; only this new
profile adopted its current bytes. A reviewed preventive copy separates production
ELF evidence from the caller's no-entrypoint dependency build. The subsequent first
compiler attempt and first Bank build/runtime passed strict gates. No installation,
cleanup, dependency upgrade, ignored test or diagnostic suppression occurred.

The Bank build reused only the fixed Task 2.38 Cargo cache and verified empty
OpenSSL configuration directory, copying the new uniquely named executable into
fresh evidence storage. Old source versions remain in Git; twelve intentional
source changes and eight new inputs are explicit in the 231-source freeze, with
211 old inputs unchanged. All ten historical artifacts, the exact old Bank binary
and six old logs remain byte-identical. Cached dependencies are reused evidence,
not freshly rebuilt dependency proof. The failed preflight and every new output
remain retained. Current free space is 3558227968 bytes (3.31 GiB);
preserve the Task 2.35 recovery archive and all historical outputs.

Scope remains local unsigned entry with synthetic Squads and the restricted
pinned Token wrapper. Packets exceed 1232 bytes; actual signatures/ALT, governance
execution and exclusive recipient control, failed in-initializer CPI rollback,
funding provenance, durable restart and live Testnet readiness are not established.
Current claim/pending compatibility has host regression evidence; earlier SBF
claim/pending evidence remains historical. Old Node/model transport does not
serialize the new native initialization envelope. No Mainnet action, deployment,
key creation/secrets access/signing, real fund movement or authority transfer occurred.

**M1 is complete within this boundary/runtime scope; M2 is next and NOT STARTED.**
Expose canonical economic runtime handlers using existing model/adapter contracts,
then implement the real pinned protected adapter (M3), full production lifecycle
(M4), exact Testnet package (M5) and explicit deployment gate (M6), in D-030 order.
Do not restart initialization or create a validation-only detour without a concrete
critical-path blocker. D-023/D-024 pause treatment for explicit contributions remains
PROVISIONAL; resolve only if it becomes a material M2 economic decision. Official
Jito Testnet compatibility remains an M5 prerequisite, not assumed from reference IDs.

Evidence: `/tmp/piv1-t239-review`, `/tmp/piv1-t239-host-focused-a`,
`/tmp/piv1-t239-host-final-a`, and `/tmp/piv1-bank-smoke-production-{sbf,build,run}-t239-*`.
See [Task 2.39](TASK_2_39_PRODUCTION_INITIALIZER.md) for exact artifact identities,
commands and limitations. Technical validation remains separate from founder
acceptance; keep main unchanged and stop before any unapproved live operation.

## Previous checkpoint — D-030 Testnet convergence (2026-09-28 UTC)

**CONFIRMED direction; M1 NOT STARTED.** The founder prioritizes the first complete,
founder-testable production lifecycle on Solana Testnet. D-030 in
[the decisions](PIV1_DECISIONS.md) refines D-026's scheduling; economic/governance,
acceptance, main-integration and live-operation gates are unchanged. Verified
starting integration: `8eee7cb884f2e37bb2a31c48aed77557cf4fb62d`; main remains
`4cc4ea11e87c2f1a2f85b9ed48358f2a881821b3`. This turn records guidance and source
triage only, not milestone implementation or a documentation-only Task 2.39.
No code, dependency, pin or evidence artifact changes and no test/build execution
follow from the guidance update.

The six ordered milestones are:

1. Expose and validate the real PIV1 initializer through the production instruction boundary.
2. Expose the production runtime handlers required for the complete economic lifecycle.
3. Implement the real pinned SPL/Jito stake-pool adapter and protected CPI paths.
4. Execute and review a complete local end-to-end lifecycle using the real production instruction paths.
5. Prepare the exact Testnet deployment artifact, addresses, authorities, funding requirements and founder test workflow.
6. Stop before the first live Testnet deployment and request the founder's explicit deployment authorization with a concise checklist.

Task 2.38 is technically complete at the starting integration commit within its
bounded scope. Its 15 runner tests, twelve Bank messages and 786 independently
checked account records are retained evidence, not reruns here. No unresolved
failing test or demonstrated defect requires another isolated Bank probe task.
Unsigned oversized packets, synthetic governance/Token, untested internal CPI
failure and other limits remain recorded; each becomes prerequisite work only
when evidence connects it to a concrete production milestone blocker. Existing
failures, final outputs and the Task 2.35 recovery archive remain preserved.

**Next bounded work belongs to M1.** Actual `instruction_boundary.rs` dispatches
only `claim_kif` and `reconcile_pending`. The full recipient-checked normalized
initializer exists in `genesis_initialization.rs`; only the validation callee
currently exposes it. Connect that complete path to a reviewed production ABI,
then validate the real production artifact with exact approved data/accounts,
governance and guardian checks, protocol/recipient identities, fresh/prefunded
rent, initial pause and existing dispatch regressions. Allocation-only or
unchecked initialization is not the M1 completion target.

ABI review must bind the actual entire approved instruction bytes. Existing
Squads authorization compares those bytes while the model decoder expects
`PIV1GM01`; a production selector/envelope must not strip or reconstruct data
before approval authentication. Recipient vault indices are derivation witnesses,
not a new production requirement for probe-only values 0/255. These are ordinary
pilot-owned technical decisions; this guidance turn selects no ABI or witness
encoding.

Later tracked boundaries are not current M1 blockers: explicit contribution
callability during pause remains PROVISIONAL under D-023/D-024, and D-006 requires
verified official Jito Testnet compatibility before choosing the exact deployment
package. The current reference identity profile is not proof of Testnet identity.
M2 can use the existing adapter contract; M3 supplies real pinned CPI behavior;
M4 must demonstrate the real production paths rather than model/stub-only success.

The [execution plan](PIV1_CODEX_EXECUTION_PLAN.md) and
[test plan](PIV1_TEST_PLAN.md) map milestone blockers to completion evidence.
Use one writer and separate reviewer; correct demonstrated defects, run focused
tests while implementing and final gates at milestone completion, then checkpoint
and publish integration normally. Departures need a demonstrated dependency,
evidence and closure criterion. Fixed per-session task stops and the previous
automatic next Bank-failure task are superseded; conserve credits and checkpoint
before interruption. Ask only for genuinely missing economic/governance decisions
or required sensitive-operation authorization, not routine engineering choices.

M5 prepares the exact cluster/genesis, artifact and Program ID, authorities and
recipients, funding/fees/rent budget, bounded operations, stop/recovery conditions
and founder workflow. M6 requires the founder's explicit approval before the
first live deployment. No wallet/key creation, signing, deployment, live fund
movement or authority transfer is authorized by D-030. Founder acceptance remains
separate, main remains unchanged, and no Mainnet action is permitted by this turn.

Guidance checkpoint verification: root reverified jerem (uid 1001), the single
worktree/branch/HEAD and actual remote main/integration/protected Task 2.3 refs.
All 223 retained source pins, the Task 2.38 exact binary and six metadata/build/
runtime logs match. One delegated writer prepared the four requested guidance
files; separate source/priority review found no prerequisite defect in Task 2.38.
Only seven documentation files change, including the canonical D-030 decision
and root README/master-spec pointers. `git diff --check` passes; prior tests and
builds were not rerun. Receipts are in
`/tmp/piv1-testnet-convergence-20260928-a`. Root owns final document review and
normal integration-only publication; Git records its identity. M1 remains next,
with no new implementation or acceptance claim from this documentation change.

## Previous checkpoint — Task 2.38 (2026-09-28 UTC)

The implementation/evidence below is retained. Its former next-task and session
stop recommendations are historical and superseded by D-030 above.

Task 2.38 is **TECHNICALLY VALIDATED / PENDING FOUNDER ACCEPTANCE** for the exact
Task 2.31 initializer probes through actual local Bank/AccountsDB commit and
ancestor-visible account rereads. Starting jerem/clean integration and matching
remote: `692d1384afb530bd3740fa8aaf5c7f8f8f448755`. Local/remote main remains
`4cc4ea11e87c2f1a2f85b9ed48358f2a881821b3`; protected Task 2.3 remains
`3677fee97e3617ee65e2828d222008ba0952bb3e`. D-026 covers reviewed integration-only
publication; Git records the resulting task commit. No new main authority or
founder acceptance is inferred.

One delegated writer prepared the new current-SDK fixture/harness; root owns
runner/pins, lock/metadata, all executions, shared documents and Git. Separate
source, pin, command, exact-binary and independent actual-evidence reviews passed. Root passed **15 final runner regressions** and **one Bank test/four
profiles/twelve messages**, then independently inspected **786 complete account
records** across sixteen snapshots. Every profile completes initializer CPIs,
then fails at outer index 2 with InvalidInstructionData; all non-fee accounts
roll back. Identical replay returns AlreadyProcessed without another fee.
A distinct retry initializes on the same Bank's actual persisted state. Fees
are exactly [15000, 0, 15000], with a separate fee payer, rent payer and executor.
Fresh original rent is 34,779,120 lamports; prefunded original rent is 890,885,
with exactly 144 lamports normalized to PendingSol. These are synthetic balances.

Real V3 Program/ProgramData accounts contain the unchanged three Task 2.31 ELFs,
deployed synthetically at slot zero and visible in child Bank slot one. No cache
injection, account reset or post-construction account store occurs. Independent
literal initialized bytes and complete ordered CPI keys/data/heights match.
The explicit 1.4m CU/default 32-KiB heap profile passes; all four successful
initializations exceed 200k CU. Wire lengths are 1503–1586 bytes and exceed the
packet limit. This is unsigned local entry, not signature/ALT/public transport,
actual Squads/Token/Jito control, native production initializer dispatch, failed
in-initializer CPI rollback, ledger durability/restart or live readiness.

The first runner attempt had one temporary-path fixture error; the reviewed
UUID-only correction then passed all fifteen tests. A preventive source-review
fix moves account rereads outside the upstream scan callback lock. Both strict
compilation and actual Bank execution passed **first attempt**. Build time:
482.135576 seconds, sampled group RSS+swap 940,589,056 bytes, minimum free disk
4,054,081,536 bytes; no diagnostics. Runtime: 4.37 test seconds / 4.448860 guarded
seconds, empty stderr, sampled RSS+swap 29,646,848 bytes. Samples are not guaranteed
peaks or minimum requirements. All socket/resource/output guards remain intact.

Six existing locked packages become direct imports. Of 553 host resolve nodes,
only the root's six edges change; every dependency version/feature/edge remains
unchanged. No install or network resolution occurred. All **223 source inputs**,
158 tools/50 aliases, 589 original archive/source trees and ten artifacts match
after execution; 215 prior inputs are unchanged, with only manifest/lock updated.
Prior tests were not rerun. Production/economics/vendor/old harnesses are unchanged.

Pins: `tools/bank_genesis_pins.json`. Final 52,358,608-byte binary SHA-256:
`ed8fc82fccedaee94be2914f3b18f3068f8d1d933cdcffba43cf0492b6d18ff2`.
Build/run directories: `/tmp/piv1-bank-smoke-genesis-build-t238-20260928-a` and
`/tmp/piv1-bank-smoke-genesis-run-t238-20260928-a`. Root receipts:
`/tmp/piv1-t238-pilot-review`; stdout SHA-256:
`91da24ad4eeb9fb9071975a70837bd6c04d1bd62bcb8e20b8c6b74d971cb2fcb`.
Keep all old/failed outputs and the Task 2.35 recovery archive at
`/home/jerem/piv1-evidence/task-2.35-intermediates-20260928-a`; its contents were
not rehashed in this task. No cleanup occurred. Post-verification free disk:
4,008,194,048 bytes (about 3.73 GiB).

See [Task 2.38](TASK_2_38_BANK_GENESIS_INITIALIZATION.md) for commands, exact
results and limitations. Next validate failures inside initializer CPIs through
Bank, with a fresh capacity check and unchanged artifacts/oracles. Actual Squads,
ALT transport, recipient control, funding provenance and remaining lifecycle
work remain open. **Save and STOP; Task 2.39 is NOT STARTED.** No Mainnet action,
deployment, real-fund movement, secrets access, key creation/signing, RPC/chain
operation or authority transfer occurred. Technical validation is not founder
acceptance or a professional independent audit.

## Previous checkpoint — Task 2.37 (2026-09-28 UTC)

Task 2.37 is **TECHNICALLY VALIDATED / PENDING FOUNDER ACCEPTANCE** for the first
native System smoke through the actual local Bank/AccountsDB commit and account
reread path. Root verified jerem (uid 1001), one clean starting integration tree
and matching remote at `850bc6d1bd2fb4137ef6cb16cc26e78634f6224a`. Local/remote
main remains `4cc4ea11e87c2f1a2f85b9ed48358f2a881821b3`; protected Task 2.3
remains `3677fee97e3617ee65e2828d222008ba0952bb3e`. D-026 covers normal reviewed
integration-only publication; Git records the resulting task commit. No main
integration authority or new founder acceptance is inferred.

One delegated writer and separate reviewer were used. Root passed **12 final
runner regressions** and **one Bank test/four message cases**, with independent
root/reviewer inspection of **100 complete account records** (20 accounts across
five snapshots). Native success persists; a failure in the second instruction
restores all non-fee account bytes while charging the exact fee. Identical replay
rejects without another fee; a distinct retry succeeds on the same Bank's actual
persisted state. Raw instruction bytes, error index/code, transient-balance logs,
fees and every account's metadata/data match. Runtime passed first execution;
the measured test duration is 0.04 seconds, guarded wall time 0.267182 seconds.
This is genuine in-process Bank account-saver evidence for native System only,
not PIV1/SBF initialization, signatures, disk restart durability or chain behavior.

Three full build attempts were rejected and retained: a denied compiler UNIX
socketpair, a missing SDK error-trait feature, then a future compiler incompatibility
in proc-macro-error2. Narrow separately reviewed corrections allow only local
UNIX socketpairs, enable the existing five8_core 0.1.2 std feature, and vendor
proc-macro-error2 2.0.1 with one declaration made public. Root and reviewer each
checked all 47 archive files (46 unchanged), licenses and exact graph delta.
An optional targeted Cargo check also failed in the resolver before compilation;
it remains failed. No warning suppression, dependency version change, registry
source patch or production change occurred. The fourth fresh strict build passed
without diagnostics in 498.474109 seconds; sampled group RSS+swap reached
941,891,584 bytes, with minimum free disk 5,295,411,200 bytes. Guards were intact.

Current pins: `tools/bank_smoke_pins.json`; local-override provenance:
`validation/genesis-bank-runtime/vendor-provenance.json`. All 217 pinned inputs,
158 tools/50 aliases and all original 589 registry archives/source trees matched
after execution; 164 baseline inputs and ten historical artifacts remain unchanged.
The final graph is 588 registry packages plus one local override (551 registry
plus one override on the host). Older tests were not rerun. Runtime's very short
sample is not a measured peak-memory requirement. Final host binary SHA-256:
`c3444648e6856c54af544e7813c566bd3eab645dccdd2e3193b0f12b3d3502d8`.

Build/run evidence: `/tmp/piv1-bank-smoke-build-t237-20260928-d` and
`/tmp/piv1-bank-smoke-run-t237-20260928-a`; root receipts:
`/tmp/piv1-t237-pilot-review`; separate review: `/tmp/piv1-t237-reviewer`.
Keep every failed attempt, the rejected build-c binary, historical artifacts and
`/home/jerem/piv1-evidence/task-2.35-intermediates-20260928-a`. Its archive contents
were not rehashed again in this task; no cleanup occurred. Current free space
observed after verification is 5,297,123,328 bytes (about 4.93 GiB).

See [Task 2.37](TASK_2_37_BANK_COMMIT_ROLLBACK_SMOKE.md) for exact commands,
failures, corrections and limitations. Next prepare exact Task 2.31 SBF loading
and initializer success/failure/account-reread oracles on Bank, with reviewed
loader/features and resource bounds. Actual Squads/ALT, recipient control,
funding provenance, native initializer exposure and remaining lifecycle work stay
open. **Save and STOP; Task 2.38 is NOT STARTED.** No Mainnet action, deployment,
real-fund movement, secrets access, key creation/signing, RPC/chain operation or
authority transfer occurred. Technical validation is not founder acceptance or a
professional independent audit.

## Previous checkpoint — Task 2.36 (2026-09-28 UTC)

Task 2.36 is **TECHNICALLY VALIDATED / PENDING FOUNDER ACCEPTANCE** for isolated
Bank dependency preparation under D-026. Actual jerem/clean starting integration
was `90132103af8a99164a3198214ce72c053673d645`; local/remote main remains
`4cc4ea11e87c2f1a2f85b9ed48358f2a881821b3`. One delegated writer prepared the
minimal independent workspace and API plan; root performed registry preparation,
metadata/byte checks and shared documentation. Separate source/evidence review
passed; final document review precedes root's normal integration-only publication.
Git records the resulting task commit. No new main authority or acceptance.

The exact Agave 4.2.0 graph resolves **589 registry packages**: 300 prior exact
version/checksum pairs and 289 new records, confined to this workspace. Root and
review independently checked all archive/source bytes; root counted **25,891
source files**. Public resolution and three locked/offline metadata observations
passed; the final repeat accounts only for the newly discovered local README.
The Linux host graph has 552 registry packages and 56 build-script packages
(70 in the full graph). **No compilation, test or Bank runtime was executed**;
build scripts, procedural macros and embedded ELFs were not executed either.
One read-only audit initially required an optional Cargo marker; the retained
correction preserves full archive/member comparisons. No metadata command failed.

The source-reviewed unsigned path can construct Bank without keypair helpers,
sanitize messages, invoke actual commit APIs and reread AccountsDB accounts.
Future failure oracles must isolate retained fees/nonce effects from program
account preservation, and respect replay protection. This is source inspection,
not cryptographic signature validation, compiler compatibility or Bank rollback
proof. Native tools/libraries, build scripts, genesis features/bundled programs,
artifact loading and measured disk/memory requirements remain to validate.

Manifest/lock/tool/provenance/evidence pins are in
`validation/genesis-bank-runtime/preparation.json`; lock SHA-256 is
`06332f984c80bdc13092df0f01be190c14514db56968726913feb1e14b0c6af8`.
The fresh public cache and complete root receipts are under
`/tmp/piv1-t236-preparation-20260928-a`; review receipts are under
`/tmp/piv1-t236-reviewer`. Observed free space is **8,127,578,112 bytes (7.57 GiB)**;
the former 8-GiB planning reserve is no longer met and never measured Bank needs.
Preserve the Task 2.35 recovery archive and historical targets; no cleanup ran.

All 162 protected inputs and ten recorded artifacts matched before/after; root
also verified 119 tool hashes/nine aliases. Old tests/logs were not rerun or fully
rehashed again. Existing production/economics/dependencies and old caches remain
unchanged. See [Task 2.36](TASK_2_36_BANK_DEPENDENCY_PREPARATION.md) for the exact
commands, source map, delta, failed audit assumption and evidence boundaries.
Next: prepare a genuine bounded Bank harness/build with reviewed native/resource
requirements, then inspect stored-account success/failure/retry behavior.
Native initializer, real Squads/ALT, recipient control, funding provenance and
remaining lifecycle work stay open. **Save and STOP; Task 2.37 is NOT STARTED.**
No Mainnet action, deployment, fund movement, secrets access, key creation/signing,
RPC/chain operation or authority transfer. This is not a professional audit.

## Previous checkpoint — Task 2.35 (2026-09-28 UTC)

Task 2.35 is **TECHNICALLY VALIDATED / PENDING FOUNDER ACCEPTANCE** under D-026 for
reversible recovery of local build capacity. Root passed **22 focused Python
regressions**, first attempt; separate source, manifest, archive and action review
passed. Exactly **8368** non-executable Cargo intermediates in twelve known host
builds were archived as **3902** verified gzip objects before originals were
removed. One **22389024-byte** library was restored at its original path and left
in place; **8367** paths remain archived. Root independently rechecked **26901
preserved files**, with no unexpected build-tree change. Recorded executables,
SBF artifacts, logs/receipts, source/cache and production/economics are preserved.

Available space after recovery is **8839438336 bytes (8.23 GiB)**; the durable
archive occupies **852205568 allocated bytes**. Selected-file accounting recovers
**5563084800 bytes net** after the archive and demonstration restore; filesystem
availability is a separate observation that can include unrelated activity.
The archive is `/home/jerem/piv1-evidence/task-2.35-intermediates-20260928-a`.
Its manifest, objects and journal must be retained for restoration. This is local
recovery storage on the same disk, not a disaster backup. Historical target trees
are no longer complete incremental caches; no earlier compiler/runtime test was
rerun. Actual Bank dependencies and commit/rollback evidence remain unprepared.

Root verified jerem (uid 1001), one clean starting integration worktree at
`9a72e3ae83852615b8da5df7d89113290574fa0d`; local/remote main remains
`4cc4ea11e87c2f1a2f85b9ed48358f2a881821b3`, protected Task 2.3 remains
`3677fee97e3617ee65e2828d222008ba0952bb3e`. One delegated writer prepared
two tools and the report; root owns shared docs, all actual actions and Git.
Root publishes normal integration-only under D-026 after final document review;
Git records the exact commit. No broader acceptance or main authority is inferred.

The first read-only inventory stopped on an access-time comparison; the corrected
stable-metadata inspection completed before action. Source-review corrections
improved disk-bound accounting and interruption recovery before tests. All 22
regressions and actual archive/prune/restore stages passed first execution.
Before archiving, 159 prior source inputs, 119 tools/nine aliases and ten pinned
artifacts matched. Exact plan matched the independent complete file inventory.
Each original removal followed durable verified recovery bytes; actual restore
preserves bytes/mode/uid/gid/nanosecond mtime, not inode/ctime/atime. Eight initially
hardlinked metadata paths were untouched. No broad cleanup or installation ran.

See [Task 2.35](TASK_2_35_BUILD_CAPACITY_RECOVERY.md) for commands, hashes, exact
recovery instructions and limits. Root receipts: `/tmp/piv1-t235-pilot-review`;
separate review: `/tmp/piv1-t235-reviewer`. Do not remove the durable archive.
Next: authenticate and pin the real Bank/AccountsDB dependency closure and verify
a compatible nonsigning local entry before any heavier build. Space alone proves
no build capacity or runtime readiness. Native initializer, real Squads/ALT,
recipient control, funding provenance and remaining lifecycle work remain open.
**Save and STOP; Task 2.36 is NOT STARTED.** No Mainnet action, deployment, fund
movement, secrets access, key creation/signing, RPC/chain or authority transfer.
Technical validation is not founder acceptance or a professional audit.

## Previous checkpoint — Task 2.34 (2026-09-27 UTC)

Task 2.34 is **TECHNICALLY VALIDATED / PENDING FOUNDER ACCEPTANCE** within its
read-only Bank prerequisite-tooling scope under D-026. Actual takeover verified
jerem (uid 1001), one clean integration worktree at
`0f27932e434441f79e17f85453f5b38ccc076f1e`, and matching remote integration.
Local/remote main remains `4cc4ea11e87c2f1a2f85b9ed48358f2a881821b3`; the
protected Task 2.3 branch remains `3677fee97e3617ee65e2828d222008ba0952bb3e`.
Root owns normal integration-only publication; Git records the resulting commit.
D-029 does not authorize this new task's integration into main.

One delegated writer prepared a standard-library Python checker, regression tests
and report. Separate source/test/command review passed; root executed **19 focused
regressions**, first attempt, in 0.166396 seconds (unittest 0.041 seconds).
No test failed. A separate read-only environment observation returned the expected
**NOT_READY / exit 2** in 0.064602 seconds: **3301838848 available bytes** against
an explicit **8589934592-byte (8-GiB) planning reserve**, with all six candidate
Bank/AccountsDB/SVM source/archive paths absent and modular runtime source present.
The reserve is not a measured Bank footprint; version 4.2.0 is only a candidate
aligned with the current modular runtime, not an authenticated Bank dependency pin.
The checker reports metadata only, executes nothing and establishes no complete
closure, integrity, build capacity, keyless API or Bank commit/rollback proof.
Final evidence/document/publication review is recorded in the task report.

Root reverified six Task 2.33 inputs, 151 protected inputs, 119 tool hashes,
nine aliases, ten pinned artifacts and the final Task 2.33 host executable.
Exactly **100 Task 2.33 logs**, including both failed attempts, were rehashed.
The older 448-log verification remains historical; no Rust/SBF/runtime suite was
rerun. Production, economics, dependencies, old harnesses/probes/pins and artifacts
remain unchanged. Only two tools and documentation are added/updated.

Evidence/commands: [Task 2.34 report](TASK_2_34_BANK_RUNTIME_PREREQUISITES.md).
Root receipts: `/tmp/piv1-t234-pilot-review`; separate review:
`/tmp/piv1-t234-reviewer`. Validation receipt SHA-256:
`59ff08cea5be3081e5af258047a263a203bc6f3aa309bc8a487797e22e24992a`.

Next: inventory reclaimable compiler intermediates or add capacity while preserving
all receipts, logs, exact artifacts, source/cache and executables; then authenticate
and pin the genuine Bank closure and establish a nonsigning local entry before
any heavier build. No cleanup or dependency preparation occurred in this task.
Native initializer exposure, actual Squads/ALT, recipient control, funding
provenance, later Token-native donations and the remaining lifecycle stay open.
**Save and STOP; Task 2.35 is NOT STARTED.** No Mainnet action, deployment, fund
movement, secrets access, key creation/signing, RPC/chain or authority transfer.
Technical validation is not founder acceptance or a professional audit.

## Previous checkpoint — Task 2.33 (2026-09-27 UTC)

Task 2.33 is **TECHNICALLY VALIDATED / PENDING FOUNDER ACCEPTANCE** under D-026.
Root verified jerem (uid 1001), one initially clean integration worktree, and local/
remote main/integration at `4cc4ea11e87c2f1a2f85b9ed48358f2a881821b3`. Task 2.32
publication under D-029 is complete. Main remains at that commit; the protected
Task 2.3 branch remains `3677fee97e3617ee65e2828d222008ba0952bb3e`.

One delegated writer prepared eight new isolated files; root owns six shared
documents, the sole compiler/test/runtime slot and Git. Separate source, runner,
command, exact-binary and runtime-evidence review passed. Root and reviewer each
independently checked **2400 complete account records**. Final document and
publication checks passed. Root publishes integration only under D-026; Git
records the task commit/publication identity. No new main authority is inferred.

Root passed **13 mocked runner regressions and eight runtime tests/sixteen message
cases**. Eight failures use ceilings fixed before execution from retained success
logs: both 35/34 profiles at 200k before effects, and first/second Token CPI with
fresh funding or second Token CPI with paused mixed prefunding. Two early cases
return exact ComputationalBudgetExceeded; six actual Token-VM failures return
ProgramFailedToComplete with the specific instruction-meter exhaustion log.
Panic/access violations cannot satisfy those specific originating-error oracles.

Before each Token failure, all expected System calls have succeeded. Full raw
accounts show exact original payer rent debit, all sixteen targets at their expected
sizes, owners and balances, nine zero state buffers and zero or one initialized Token vault. Prefunded
cases retain the exact 144-lamport sweep and 890885 original rent debit; fresh cases
debit 34779120. Complete account bytes/owners/balances/metadata, exact CPI traces/
privileges and runtime Instructions/Clock/Rent match independent expectations.
Each actual returned-original vector then succeeds in a fresh runtime with the
identical instruction under the existing **1.4m-CU ceiling/default 32-KiB heap**.
Successful retries use 1063693/1057103 CU (fresh distinct/shared) or 984673/978083
(prefunded distinct/shared), with exact initial state/Token bytes and zero ledgers.
Mollusk output discard and retry are **not Bank/AccountsDB rollback or recovery
from partially committed chain state**; raw partial state is never the retry input.

Actual execution history, including both failures:

- Writer performed only one locked/offline metadata call: first pass in 0.439997
  seconds, empty stderr, unchanged manifest/lock and all 389 registry identities.
- Root's first 13 mocked guards had 11 passes/two fixture-path failures. Python
  temporary suffixes can contain underscores rejected by the strict path guard.
  A reviewed UUID-hex fixture correction preserves the guard and adds explicit
  underscore rejection. The second run passed all 13 in 0.139159 seconds.
- First guarded host build passed in 152.588191 seconds, no diagnostics/24 logs.
  First runtime had six passes/two preflight-error assertion failures in 1.851599
  seconds: 14 message cases/2100 complete account records. Both early cases were
  unmutated; all six Token failures and six retries matched exact oracles.
- A narrow preflight-only error/log correction reflects pinned checked-charge
  behavior; Token errors, budgets, state, custody and traces remain unchanged.
  Fresh second build passed in 165.934385 seconds with zero diagnostics/
  24 verified logs; the subsequent complete runtime passed. No failed attempt is
  relabelled, removed or overwritten. The unchanged 13 runner results are retained
  after the Rust-only correction, not rerun.

Evidence and exact commands: [Task 2.33 report](TASK_2_33_GENESIS_INITIALIZATION_FAILURE_RUNTIME.md).

- Build-b: `/tmp/piv1-genesis-initialization-failure-runtime-build-t233-20260927-b`;
  result SHA-256 `a4806c159faaa794ff526582b5451939ac56c33efa52c509838ab48faa675f06`.
- Host executable: 23208208 bytes, SHA-256
  `1e919318e405405b7f668d8bf00cc65ad40d8ff313ec7f84a060d703cec05ae6`.
- Run-b: `/tmp/piv1-genesis-initialization-failure-runtime-run-t233-20260927-b`;
  result SHA-256 `3dc7811de130c298dd9d4724a81da9838be28d6ea3a70dca344d580788615e93`.
- Runner SHA-256 `47409e204d21e1b79aa0838892bd2affc3498e2b69dde9b2749721029b91f2e8`;
  final pins `fc561ee67bd5be87aa84762f7b1067289f828722b1cad77a89d53c8127ba2379`.
- Root receipts: `/tmp/piv1-t233-pilot-review`; separate review:
  `/tmp/piv1-t233-reviewer`; runtime review SHA-256 `c14dc37f18eb26a87798d574e63b9a5baad723dca4c58acd891eb281087823e7`.

Six new source inputs, 151 protected old inputs, 119 unchanged tools, nine aliases,
389 existing registry identities, three executed Task 2.31 ELFs and seven historical
artifacts are bound. No new SBF build, dependency, installation or tool-pin refresh.
Production/native ABI/economics, previous probes/harnesses/pins and artifacts remain
unchanged. **448 earlier logs** were verified: prior host/doctest/gate, transport,
SBF and Task 2.32 results remain retained evidence, not reruns in this task.

Remaining work includes runtime commit/rollback evidence beyond returned-vector
discard, broader failure/heap boundaries, native initializer exposure, actual
Squads/ALT lifecycle, full recipient control, funding provenance, later Token-owned
native donations and the remaining PIV1 lifecycle before founder Testnet testing.
**Save and STOP; Task 2.34 is NOT STARTED.** No Mainnet action, deployment, fund
movement, secrets access, key creation/signing, chain/RPC or authority transfer.
No credit balance is inferred; AI-assisted review is not a professional audit.

## Previous checkpoint — Task 2.32 (2026-09-26 UTC)

Task 2.32 is **TECHNICALLY VALIDATED / FOUNDER-AUTHORIZED MAIN INTEGRATION**
under D-029, within keyless local full-initialization execution. Root verified
jerem (uid 1001), one initially clean integration worktree, local/remote starting
integration `60193d63b64f42211f98d9b4910dfc047ab876df` and starting main
`7b74be4b13c019b96a0c8abcbebfbcc361d31089`. The Task 2.3 branch remains protected
at `3677fee97e3617ee65e2828d222008ba0952bb3e`. D-029 records the founder's request
to progress and publish to main if verification passes. It covers the reviewed
Tasks 2.24–2.32 sequence, not broader founder acceptance or live-operation authority.

One delegated writer prepared nine isolated harness/runner/pin/test/report files;
root owns seven shared documents, all compiler/test/runtime execution and Git.
Separate source, runner, command, exact-binary and actual-evidence review passed.
The cumulative eight earlier task commits and their unchanged evidence inputs
were separately reviewed. Final document review and publication checks passed.
Root publishes by normal main and integration fast-forward; Git records the exact task commit and
publication identity. Verify actual refs and worktree on resumption.

Root passed **13 mocked runner regressions and 13 runtime tests/cases**. The exact
three unchanged Task 2.31 SBF artifacts execute both complete fixed 35/34-account
profiles with actual System and restricted canonical Token InitializeAccount3.
Independent oracles cover all nine state envelopes, both Token layouts, payer
rent debit, native prefund sweep into PendingSol, zero initial ledgers, protected
accounts and actual Instructions/Clock/Rent. Root and the separate reviewer each
checked **2040 complete supplied/raw-before/raw-after/returned account records**.

Four successes use **1063693/1057103/984673/978083 CU**, with default **32-KiB heap**
and explicit **1.4m-CU ceiling**, above 200k. Fresh profiles debit 34779120 lamports
of original rent; mixed-prefund profiles debit 890885 and sweep 144 native lamports
while leaving initial principal/pending ledgers zero. Eight early negative cases
reject as expected. One late second-instruction error follows complete successful
initialization: raw initialized accounts remain observable, while Mollusk returns
the supplied originals. This proves output discard, **not Bank/AccountsDB rollback
or in-initializer failed-CPI atomicity**. The synthetic caller is not actual Squads;
the restricted Token wrapper is not the deployed/general Token program.

Execution history is preserved without relabelling failures:

- Writer's sole locked/offline metadata preparation passed (0.488128 seconds);
  all 389 existing registry identities are unchanged.
- Root's 13 mocked runner tests passed first attempt (0.191178 seconds). Their
  runner/test inputs remain unchanged after the two Rust-only corrections.
- First build `...build-t232-20260926-a` failed on obsolete test Rent fields
  (two errors, one warning). A minimal default-Rent comparison preserved all
  sixteen literal rent-floor assertions; no production edit or suppression.
- Second build `...build-t232-20260926-b` passed, then first runtime
  `...run-t232-20260926-a` had 12 passes/one trace-order assertion failure.
  All four initialization outcomes and the intended late error occurred correctly.
  Pinned Agave reserves every top-level trace slot before appending CPI slots;
  the harness now checks that exact topology without weakening state oracles.
- Final build-c passed in 172.577382 seconds with 24 verified logs/no diagnostics.
  Final run-b passed all 13 cases in 1.490205 seconds with 24 verified logs.
  Both rejected attempts and all raw evidence remain preserved.

Exact final evidence and commands are in the [Task 2.32 report](TASK_2_32_GENESIS_INITIALIZATION_RUNTIME.md):

- Build: `/tmp/piv1-genesis-initialization-runtime-build-t232-20260926-c`;
  result SHA-256 `2b9241ed061c963665bf11c84932ea76b77d684890796e444b10caca701a8b7e`.
- Host executable: 23177104 bytes; SHA-256
  `ef31a259225a4cc41de8642de1421d2c6d5609effe9dfb718287e896061b9ecf`.
- Run: `/tmp/piv1-genesis-initialization-runtime-run-t232-20260926-b`;
  result SHA-256 `0f680990edf2229c126cff56dfa1a90209fb7537b87e5d0649b766cf82f54627`.
- Runner SHA-256 `a93d385405a4f645a5459b0112c6bb40f267a5b7d6513360b235c282b1971f5d`;
  pins SHA-256 `ff9b72f3bad0b5e6265fe096d949a3b75bf162bffacf6811b8223e6c136c775a`.
- Root inspection: `/tmp/piv1-t232-pilot-review`; separate review:
  `/tmp/piv1-t232-reviewer`. Independent runtime review receipt SHA-256
  `482e69fce9be9148f545325565542ea1d3c63f2f0ebdd9412bfc3ee5e83e9db5`.

Seven new source inputs, 143 protected old inputs, 119 host tools, nine aliases,
389 registry packages, three executed artifacts and six historical artifacts are
bound by the new profile. Root verified 142 prior task inputs and **326 retained
logs**: earlier 469 host tests +1 doctest/eight gates, 24 claim/pending SBF tests/
70 cases, 15 Node tests/eight old/sixteen recipient cases, Task 2.29's 10 boundary
+9 runner tests, Task 2.30's 12 runtime +11 runner tests and Task 2.31's 15 boundary
+11 runner tests remain **retained evidence, not rerun** in Task 2.32.

Takeover detected installed libexpat1 2.6.1-2ubuntu0.6. Root and separate review
verified signed cached Ubuntu metadata, the exact public package and installed
library bytes, without installation. Only the new profile adopts hash
`286682ecbc5e59a638963b1a4e6351e65eb32fcf4bdcb9cb7569b6a61fe06a8d`;
118 other host tools/nine aliases match. Old pins/helpers are unchanged. Production
sources/ABI/economics, previous probes/harnesses and all historical artifacts are
unchanged. No new dependencies or target SBF build were needed.

A next candidate is failure/resource-boundary coverage within initialization;
Task 2.32's late case occurs after complete initialization. Native initializer
exposure, actual Squads/ALT lifecycle, complete recipient control, funding
provenance, later Token-owned native donations and founder Testnet readiness
remain unproved. **Save and STOP; Task 2.33 is NOT STARTED.** No Mainnet action,
deployment, fund movement, secrets access, key creation/signing, RPC/chain operation
or authority transfer occurred. No credit balance is inferred. AI-assisted review
is not a professional independent audit. Earlier checkpoints below describe
historical publication/status boundaries; D-029 governs this reviewed integration.

## Previous checkpoint — Task 2.31 (2026-09-26 UTC)

Task 2.31 is **TECHNICALLY VALIDATED / PENDING FOUNDER ACCEPTANCE** under D-026,
within build-only preparation. Root verified jerem (uid 1001), one initially clean
integration worktree and matching local/remote baseline
`6238088f6dc8ef42a266b5041db9e0e50f262c85`. Task 2.30 publication is complete.
Main remains `7b74be4b13c019b96a0c8abcbebfbcc361d31089`; the Task 2.3 branch remains
`3677fee97e3617ee65e2828d222008ba0952bb3e`.

One delegated writer prepared twelve new workspace files, three new tool/pin files
and the [Task 2.31 report](TASK_2_31_GENESIS_INITIALIZATION_PROBES.md); root owns six
shared documents, all test/compiler execution and Git. Separate source/runner/
command/host-evidence review passed; actual-artifact static review passed.
The writer performed only one locked/offline metadata preparation (first pass,
0.267849 seconds). No writer/reviewer compilation or test rerun is claimed.

The three new artifacts prepare the complete fixed 35/34-account topology: a
synthetic Squads caller with exact single-action bytes/metas, canonical vault PDA
seeds and an actual outer signing/writable rent payer; a callee invoking only the
unchanged full recipient-checked normalized initializer; and a canonical-ID Token
wrapper accepting only InitializeAccount3 through pinned SPL Token 8. There is no
successful bare-allocation return. Actual nested CPI errors propagate unchanged.
All ordinary-host entrypoints fail closed. The caller is not actual Squads and the
restricted Token wrapper is not a deployed Token artifact or general replacement.

Root's first host execution passed **15 Rust boundary tests +11 mocked runner
regressions**, with zero failures/diagnostics. Three doctest suites discovered
zero examples; warning-denied docs passed. Four gates took 82.682023 seconds.
The first strict target build passed with **34 commands/68 verified logs** in
362.604063 seconds and no diagnostics. No new SBF runtime execution occurred.
Root independently verified all artifact sizes/hashes, sources, packages and logs.
Separate static inspection matched every disassembled text byte, all expected
entrypoints/imports and direct frame accesses within the 4096-byte limit. It also
independently checked all 155 package archives and 4468 extracted source files.
This is static evidence only; runtime heap, compute and atomicity remain unproved.
Artifact-review receipt SHA-256:
`6b857cbb97b6144f7c659dbdc0897c5e984604b9fd36f1f583562f7221769c38`.

Evidence and exact commands are in the task report:

- Host gates: `/tmp/piv1-t231-pilot-host-20260926-a`; eight complete logs;
  result SHA-256 `73d28a6e64c3fe14783ec2d7b8ba2428f5f30b1c41df74f2a4ab2c6250e725b5`.
- Strict SBF build: `/tmp/piv1-genesis-initialization-probes-build-t231-20260926-a`;
  result SHA-256 `0af51d4105d6474d6056fed0c1d1245e4fdc6330653a8a762b9f32c27a3044b9`.
- Caller: 61080 bytes; SHA-256
  `e7fc7bf5d75a9494fd3a4787733df53adf8c3b724653a113ae36a9574707ec97`.
- Callee: 376720 bytes; SHA-256
  `07934627a3fdab928ab1aca2abf424eb683ea9e680681389c9b53dc2391a66b7`.
- Restricted Token wrapper: 126424 bytes; SHA-256
  `c0f42a30da4079601711bec29bd0ca780674654ea71790eb32c76b5cedad4499`.
- Root inspection: `/tmp/piv1-t231-pilot-review`; separate review:
  `/tmp/piv1-t231-reviewer`. No failed host/target build occurred in this task.

Fourteen new source/runner inputs, 128 protected earlier inputs, 155 existing
registry identities and six historical artifacts remain bound and verified. Host
execution also checked 119 tools. The new target profile is derived in memory
from the immutable historical profile; the helper remains unchanged. Precisely one
current libexpat hash uses Task 2.30's verified signed OS/package provenance;
no old-pin change, new download or installation occurred.
Production source/ABI/economics, old probes/harnesses and historical artifacts are
unchanged. All prior rejected outputs remain preserved.

Root verified **250 retained logs** (50 Task 2.30, 128 older, 64 Task 2.29 target,
eight Task 2.29 host). Earlier 469 host tests +1 doctest/eight gates, 24 claim/pending
SBF tests/70 cases, 15 Node tests/eight old/sixteen recipient cases, 10 Task 2.29
boundary +9 runner tests and 12 Task 2.30 runtime +11 runner tests are **retained
evidence, not rerun**. Task 2.30's 1674 complete account records and 288277/282070-CU
successes remain read-only preflight evidence, not full initialization proof.

Complete initialization runtime/resources/atomicity is the next candidate. Native
initializer exposure, actual Squads/ALT lifecycle, full recipient control, funding
provenance, later Token-native donations and founder Testnet readiness remain
unproved. Root completes final document review and normal integration-only
publication; Git records the task commit/publication identity. Verify actual refs
and clean worktree on takeover. **Save and STOP; Task 2.32 is NOT STARTED.** Main
acceptance and exact live-operation gates remain unchanged. No Mainnet action,
deployment, fund movement, secrets access, key creation/signing, RPC/chain operation
or authority transfer occurred. No credit balance is inferred; AI-assisted review
is not a professional independent audit.

## Previous checkpoint — Task 2.30 (2026-09-25 UTC)

Task 2.30 is **TECHNICALLY VALIDATED / PENDING FOUNDER ACCEPTANCE** under D-026,
within keyless local read-only preflight runtime scope. Root verified jerem
(uid 1001), one initially clean integration worktree and matching local/remote
baseline `352fe7d4ecd8609d93cf2b0a2a96009018d3a7de`. Task 2.29 publication is complete.
Main remains `7b74be4b13c019b96a0c8abcbebfbcc361d31089`; Task 2.3 remains
`3677fee97e3617ee65e2828d222008ba0952bb3e`. D-028 does not cover this task.

One delegated writer prepared the new isolated harness/runner/pins/report; a
separate reviewer inspected source, test oracles, build/binary/commands and actual
evidence. Root owns the single compiler/runtime slot, six shared documents, Git
and integration-only publication. No writer/reviewer compilation or SBF execution
is claimed. Root passed **11 mocked runner regressions +12 runtime tests/cases**.
The first host compilation failed before tests with two E0433 module-path errors
in reused support; a minimal new test-root re-export fixed them without changing
assertions, production, shared oracle, probes or dependencies. The fresh second
build passed without diagnostics; the first runtime execution passed all cases.
Both rejected and successful outputs are preserved.

The exact Task 2.29 caller/callee ELFs execute actual height-two SBF CPI for both
32/31-account fee-receiver profiles. Runtime-generated Instructions, Clock/Rent,
exact outer/inner data/privileges and precise rejection categories are checked.
Root independently decoded **1674 complete supplied/raw-before/raw-after/returned
account records**, including generated Instructions, and verified preservation.
Eight callee negatives, direct height-one rejection and outer discriminator
rejection are expected test results. Neither return data nor System/Token CPI is
produced. Success consumes **288277/282070 CU**, above 200k, under the explicit
**1,400,000-CU ceiling/default 32-KiB heap**. No ordinary-budget viability, minimum
heap or complete initialization resource claim follows.

Evidence and exact commands are in [Task 2.30](TASK_2_30_GENESIS_PREFLIGHT_RUNTIME.md):

- Failed build: `/tmp/piv1-genesis-runtime-build-t230-20260925-a`.
- Successful build: `/tmp/piv1-genesis-runtime-build-t230-20260925-b`;
  result SHA-256 `8a791384a3d26fc617769773c34aada58a3cc50c8de59d0f13f8e67d23309931`.
- Exact host test binary: 22982472 bytes, SHA-256
  `72e8f0609018f94b9eb4b9dbcdfc04c50d944ceff7ea1b9bcb5e9954d99d2ca0`.
- Successful run: `/tmp/piv1-genesis-runtime-run-t230-20260925-a`;
  result SHA-256 `7dba3508ce7bbad6bb05cfc0e65a162d8cd23e239888b6a1fe17894ce879c138`.
- Root inspection: `/tmp/piv1-t230-pilot-review`; separate review:
  `/tmp/piv1-t230-reviewer`. Each build/run has 24 bound command logs.

All seven new inputs, 119 protected inputs, 389 existing registry identities and
the exact probe/historical artifacts remain verified. One installed libexpat
hash differs from historical pins; root and separate review verified its exact
bytes through signed Ubuntu snapshot metadata and package payload. Only the new
profile binds the current hash; no installation or historical-pin modification.
Root reverified 128 earlier logs, 64 probe-build logs and 8 probe-host logs. Earlier
469 host tests +1 doctest/eight gates, 24 local SBF tests/70 cases, 15 Node tests/
eight old plus 16 recipient cases and 10 probe host +9 old runner tests are verified
retained evidence, **not rerun**. The eleven new runner tests were run once before
the Rust-only correction; their unchanged runner/test source remains verified.

The caller is synthetic code under the Squads ID, not actual Squads governance
or recipient control. Read-only preservation is not mutating initializer custody,
Bank/AccountsDB rollback or serialized preflight-fact evidence. Production source,
ABI/economics, probes, old harness and historical artifacts remain unchanged.
Full initialization runtime/resource/atomicity preparation is the next candidate;
funding provenance, later Token-native donations, actual Squads/ALT lifecycle,
full recipient control and founder Testnet readiness remain deferred.

Root completes final documentation review and normal integration-only publication;
Git records this task's commit and publication identity. Verify actual refs and
worktree on takeover; keep main at its authorized milestone. **Save and STOP;
Task 2.31 is NOT STARTED.** Founder acceptance and exact live-operation gates remain.
No secrets access, key creation/signing, RPC/chain operation, Mainnet action,
deployment, fund movement or authority transfer occurred. No credit balance is
inferred; AI-assisted review is not a professional independent audit.

## Previous checkpoint — Task 2.29 (2026-09-21 UTC)

Task 2.29 is **TECHNICALLY VALIDATED / PENDING FOUNDER ACCEPTANCE** under D-026,
within its build-only scope. Root verified jerem (uid 1001), one initially clean
integration worktree and matching local/remote baseline
`162f3b7b634633e2a5ab3011f0d746c4a4d15599`. Task 2.28 publication is complete.
Main remains `7b74be4b13c019b96a0c8abcbebfbcc361d31089`; the Task 2.3 branch remains
`3677fee97e3617ee65e2828d222008ba0952bb3e`. D-028 does not cover this task.

[Task 2.29](TASK_2_29_GENESIS_PREFLIGHT_PROBES.md) prepares two isolated validation
SBF probes. The callee uses the unchanged public recipient preflight and fixed
32/31-account roles (recipient vault indices 0/255). The synthetic caller preserves
the stored inner bytes/privileges and uses canonical vault signing seeds only in
its SBF branch. Both ordinary-host entrypoints fail closed. This caller is not
actual Squads governance or recipient-control evidence; the callee returns no
serialized preflight facts. No production source/ABI/economic change, initializer
exposure or SBF probe runtime execution is included.

One delegated writer and a separate reviewer completed source/test/runner review.
Initial writer test compilation failed before tests: the reused fixture needed
an explicit existing Token dev-dependency and a shared AccountInfo backing lifetime.
Those test-only fixes passed separate review. Final writer and root independently
passed **10 Rust boundary tests +9 mocked runner tests**. Root also passed doctest
discovery (zero examples) and documentation with warnings denied: four gates,
zero failures/diagnostics. Rust executions share a task-local compiler cache;
independent execution is not a second clean compilation. The initial failure is
preserved. Production sources and tests remain unchanged.

The first target build rejected one unused-result warning plus its summary despite
Cargo zero. An explicit observation discard in the validation-only callee resolved
it without a lint suppression or weaker gate; final writer/root host gates passed
again. The second strict locked/offline workspace build passed without diagnostics;
separate static inspection passed for both exact artifacts:

- Callee: 180232 bytes, SHA-256 `1d87ab760fae785cc74a8ef069722ee5bb3c48ad43e029118eba512cd1c97c53`.
- Caller: 64368 bytes, SHA-256 `938e6c1c63554ac26f75f3c4daef614087051ea88d8f9eb130692940278508b2`.

The guarded runner binds eleven probe inputs, 107 protected inputs, three
historical artifacts, the existing tools and complete package source bytes.
All 155 registry identities remain a subset of the root's 166. The test-only
SPL Token 8.0.0 edge adds no registry version. Source/lock/tool/ref preservation
passed. Static inspection is not runtime loading, total heap/compute, CPI/rollback
or successful genesis proof. Historical artifacts remain intact.

Root separately verified 92 production/105 existing harness inputs, eight concrete
transport inputs, 128 retained logs and tool/artifact hashes. **469 host tests
+1 doctest/eight gates, 24 SBF tests/70 cases, and 15 Node tests/eight old plus
sixteen recipient cases are retained earlier evidence, not rerun in Task 2.29.**
Evidence: `/tmp/piv1-t229-pilot-host-20260921-b/pilot-summary.json`,
`/tmp/piv1-genesis-probes-build-t229-20260921-b`, and
`/tmp/piv1-t229-pilot-review`. The task report records writer attempts, commands,
exact artifacts, reviews and limitations.

Root owns final documentation review and ordinary integration-only publication;
Git records the resulting task commit/publication identity. Verify actual refs
and clean worktree on takeover. Save and **STOP; Task 2.30 is NOT STARTED**. The
next bounded candidate is keyless local execution of these exact probes with real
height-two CPI and runtime-derived Instructions/Clock/Rent, still with synthetic
caller limitations. Full initialization, funding provenance, later Token-native
donations, full recipient control and actual Squads/ALT lifecycle remain deferred.
No Mainnet action, deployment, RPC/chain operation, fund movement, key creation or
signing, secrets access or authority transfer occurred. Preserve exact live gates;
no credit balance is inferred. AI-assisted review is not a professional audit.

## Previous checkpoint — Task 2.28 (2026-09-21 UTC)

Task 2.28 is **TECHNICALLY VALIDATED / PENDING FOUNDER ACCEPTANCE** under D-026.
After the credit interruption, root reverified jerem (uid 1001), one worktree,
expected unfinished preparation changes and local/remote integration baseline
`560ca09c9c17becb79564c164e8c308b196c7cbe`. Main remains
`7b74be4b13c019b96a0c8abcbebfbcc361d31089`; Task 2.3 remains
`3677fee97e3617ee65e2828d222008ba0952bb3e`. D-028 does not cover this task.
No Rust correction had landed; root resumed with one writer and a separate
reviewer, preserving all completed preparation and rejected build evidence.

[Task 2.28](TASK_2_28_CURRENT_SBF_RUNTIME_REFRESH.md) refreshes current production
SBF compilation and existing claim/pending runtime evidence. The first strict
build rejected seven oversized stack frames (243 diagnostics despite Cargo zero).
Private preflight indirection removed the caller frames; the second build still
rejected three dispatch frames (45 diagnostics). Two non-inlined producer/boxing
helpers resolved the remaining construction temporaries. The third strict build
passed without diagnostics. Native/public signatures, account/model bytes,
validation/error order, custody, dependencies and economics are unchanged.
A compile-time observation bound and host nested-result regression were added.
The two added allocations request 2272 bytes under the measured host layout,
plus alignment; total genesis heap/resource sufficiency remains unproved.

On the final source, writer passed 53 focused tests and root independently passed
**469 host tests +1 doctest/eight gates**, without failures or diagnostics.
Intermediate correction host successes remain separately attributed. Root's
**16 target-runner +7 harness-runner regressions** also passed earlier in this
same task. Final target compilation, separate static artifact review, fresh
harness compilation and separately reviewed exact-binary execution passed.
**24 local SBF tests /70 cases /1629 complete account records** passed; root
independently checked raw and returned full-account states. This covers only
existing dispatched claim/pending paths with synthetic fixtures and privileges.
Mollusk output discard is not Bank/AccountsDB rollback or signed cluster proof.

Current ELF: 229888 bytes, SHA-256
`0eb5e62389c9baa5311fddca99d1e705f86b1fd698e869a8cdcec778aa68cd54`.
The target/harness freezes cover 92/105 exact source inputs. Historical artifacts
are preserved. Thirteen changed OS path hashes (nine unique files) were narrowly
updated only after signed Ubuntu metadata and exact package-payload verification;
no installation or dependency upgrade. **15 Node tests/eight old plus sixteen
recipient cases are retained Task 2.27 evidence, not rerun**, after eight concrete
transport inputs and retained tools/logs matched. The earlier combined 96-input
freeze is historical because the two Rust preflight inputs changed.

Evidence: `/tmp/piv1-t228-pilot-host-20260921-b/pilot-summary.json`,
`/tmp/piv1-keyless-sbf-build-t228-20260921-c`,
`/tmp/piv1-sbf-claims-build-t228-20260921-a`,
`/tmp/piv1-sbf-claims-run-t228-20260921-a`, and
`/tmp/piv1-t228-pilot-review`. The task report records exact commands, failures,
artifact/tool hashes, runtime observations, reviews and limitations.

Root owns final documentation review and normal integration-only publication;
Git records the exact task commit/publication identity. Verify refs and clean
worktree on takeover. Save and **STOP; Task 2.29 is NOT STARTED**. Next scope the
remaining initializer prerequisites, including dedicated genesis runtime/resource
proof, operational funding provenance, later Token-native donations and full
recipient control. Native initializer exposure, actual Squads/ALT/buffer lifecycle,
production Jito operations and complete founder Testnet readiness remain deferred.
No Mainnet action, deployment, fund movement, key creation/signing, secrets access
or authority transfer occurred. Preserve the exact live-operation gates; no
credit balance is inferred. AI-assisted review is not a professional audit.

## Previous checkpoint — Task 2.27 (2026-09-20 UTC)

Task 2.27 is **TECHNICALLY VALIDATED / PENDING FOUNDER ACCEPTANCE** under D-026.
The founder resumed one economical bounded step. Root verified `jerem` (uid 1001),
one clean integration worktree and baseline local/remote integration
`648998b4f5767eadf14c511d1dd0034ffff29ee0`; Task 2.26 publication is complete.
Main remains `7b74be4b13c019b96a0c8abcbebfbcc361d31089`; Task 2.3 remains
`3677fee97e3617ee65e2828d222008ba0952bb3e`. D-028 does not extend main authority.

[Task 2.27](TASK_2_27_RECIPIENT_CHECKED_GENESIS_TRANSPORT.md) extends the pinned
unsigned Node harness with an explicit recipient-checked profile for Task 2.26's
complete 35/34-account fixture. Both same-multisig vaults (indices 0/255) are appended
after Token as readonly nonsigners, bound to the approved recipient fields in the
unchanged 313-byte model format. Literal topology/payload assertions and retained
Rust source pins bind this host witness; no native role ABI is introduced.
Buffered creation and v0 execution with the synthetic 16-target ALT fit the pinned
1232-byte limit. Actual v0 packets are 940/907 bytes (distinct/shared receivers),
or 988/955 with illustrative compute/heap prefixes; these are not resource budgets.
SDK-refused candidates have no fabricated serialized evidence. Default Task 2.23
CLI output remains byte-identical to its historical golden report.

One delegated writer and root independently executed **15 Node tests PASS**, plus
**eight historical-profile and sixteen recipient-profile CLI cases**, on matching
96 input hashes (92 Rust, two scripts, two manifests). Both executions passed on
the first attempt without failures, skips or diagnostics. Separate source/test
review passed after T227-R1: a substitution regression used a unique unrelated
key to reach the intended canonical identity check; corrected before any test
execution, not a production defect or failed run. Root verified twelve new logs,
sixteen retained Rust gate logs and pinned tools. **468 Rust tests +1 doctest/eight
gates are retained Task 2.26 evidence, not rerun**, with all 92 inputs unchanged.
No current SBF/runtime evidence was added; Task 2.14 remains historical.

Root evidence: `/tmp/piv1-t227-pilot-host-20260920-a/pilot-summary.json`;
writer: `/tmp/piv1-t227-writer-20260920-e_q97383`;
review/retained evidence: `/tmp/piv1-t227-pilot-review`.
Root owns final documentation review and normal integration-only publication;
Git records the exact task commit/publication identity. Verify refs and clean
worktree on takeover. Save and **STOP; Task 2.28 is NOT STARTED**. Next scope
remaining initializer prerequisites: operational funding provenance, later
Token-native donation handling, full recipient control and current runtime/
resource/rollback evidence. Native exposure and actual ALT/buffer/Squads lifecycle
remain deferred; packet fit does not establish founder Testnet readiness.
No Mainnet action, deployment, fund movement, key creation/signing or authority
transfer occurred. Preserve exact live-operation gates; no credit balance inferred.

## Previous checkpoint — Task 2.26 (2026-09-20 UTC)

Task 2.26 is **TECHNICALLY VALIDATED / PENDING FOUNDER ACCEPTANCE**. The founder
resumed one economical bounded step. Root verified `jerem` (uid 1001), one clean
integration worktree and local/remote HEAD at baseline
`3b45241aec5e0b6a9405f5df2516da66c42c6485`. Main remains
`7b74be4b13c019b96a0c8abcbebfbcc361d31089`; Task 2.3 remains
`3677fee97e3617ee65e2828d222008ba0952bb3e`. Task 2.25 publication is complete;
D-028 does not extend main authority to these later tasks.

[Task 2.26](TASK_2_26_RECIPIENT_CHECKED_GENESIS_INITIALIZATION.md) composes recipient
identity checks with normalized same-call genesis initialization. The fixed new
profile uses the allocator's single fresh full preflight and captured Rent,
checks both recipients before effects, then preserves exact identity, balances,
owner/data and privilege flags after every successful System/Token CPI and final
completion. Its selection cannot substitute another genesis role mapping. No
public effects path accepts detached evidence. Invocation errors propagate before
postchecks; production does not undo partial host effects. Original external rent
obligations, normalization, zero initial ledgers and later pending recognition are
preserved. Existing API profiles, model/schema/native dispatch and economics stay
unchanged; added library-only error variants require exhaustive matches to adapt.

One delegated writer completed **22 initialization +8 recipient-preflight tests
PASS**, including six new groups and 24 success worlds. Separate source/test
review passed. Root independently executed **468 host tests +1 doctest/eight gates
PASS**. Both executions passed first attempt with no failures, ignored tests,
diagnostics or corrective retry. All 92 inputs match inspection and both
executions; root verified sixteen gate logs, two writer logs and three tools.
Root evidence: `/tmp/piv1-t226-pilot-host-20260920-a/pilot-summary.json`;
writer: `/tmp/piv1-t226-writer-20260920-e5n764bq`. Six old transport-template
inputs are unchanged: nine Node tests/eight cases remain retained evidence of
the earlier exact 33/32-account template, not rerun. The new complete fixture has
35/34 accounts and no transport/runtime execution proof. Task 2.14 SBF evidence
remains historical; no current SBF build was run.

Root owns final documentation review and normal integration-only publication.
Git records the exact task commit/publication identity; verify actual refs and
clean worktree on takeover. Save and **STOP; Task 2.27 is NOT STARTED**. No later
implementation or build is running. Next scope the combined transport and
remaining initialization readiness prerequisites: funding provenance, later
Token-native donations, full recipient control, spending-limit/stale-action/
live-artifact checks and current runtime/resource/rollback evidence. Native
initializer exposure remains deferred. No credit balance is inferred. No Mainnet
action, deployment, fund movement, key creation/signing or authority transfer
occurred. Preserve the mandate's exact live-operation gates.

## Previous checkpoint — Task 2.25 (2026-09-20 UTC)

Task 2.25 is **TECHNICALLY VALIDATED / PENDING FOUNDER ACCEPTANCE**. The founder
resumed one economical bounded step. Root verified `jerem` (uid 1001), one clean
integration worktree, HEAD and remote integration at baseline
`ceeac8618a66f662e5d6a0369ada76b080ebee84`. Main remains
`7b74be4b13c019b96a0c8abcbebfbcc361d31089`; Task 2.3 remains
`3677fee97e3617ee65e2828d222008ba0952bb3e`. Task 2.24 publication is complete;
D-028 does not extend main authority to Tasks 2.24–2.25.

[Task 2.25](TASK_2_25_GENESIS_RECIPIENT_PREFLIGHT.md) freshly composes full genesis
preflight with both approved temporary recipients. Each must match a canonical
vault PDA of the same authenticated governance multisig, remain distinct from
other roles/backing, and be an empty nonexecutable System account with positive
lamports covering current rent. Vault indexes only witness the approved keys.
No account mutation or funding occurs; recipient signatures are not required.
Native dispatch, schema, dependencies, economics, existing APIs and the 313-byte
model are unchanged.
Present identity does not prove exclusive four-of-six spending, absence of Squads
spending limits or stale actions, or live artifact control. The 32/31-account
read-only fixture needs its own future composition/transport proof; old complete
initializer packet evidence covers only its exact 33/32-account template.

One delegated writer completed eight grouped tests with 72 positive worlds.
The initial focused run was **7/8, exit 101, one unused-result warning**. Root and
reviewer verified a test-only correction to the expected earlier `InvalidAddress`
rejection and the success assertion; one retry was **8/8 PASS**, no diagnostics.
Separate source/test review passed. Root independently executed **462 host tests
+1 doctest/eight gates PASS** on the corrected freeze, first root execution with
zero failures, ignored tests or diagnostics. All 92 inputs match inspection and
retry; root verified sixteen gate logs, all four writer logs and three tool hashes.
Root evidence: `/tmp/piv1-t225-pilot-host-20260920-a/pilot-summary.json`.
Writer evidence: `/tmp/piv1-t225-writer-20260920-oc626x43` and
`/tmp/piv1-t225-writer-retry-20260920-tujlllux`. Six old transport-template inputs
are unchanged: nine Node tests/eight packet cases remain retained, not rerun.
No current SBF/runtime proof was added; Task 2.14 remains historical evidence.

Root owns final documentation review and normal integration-only publication.
Git records the exact task commit/publication identity; verify actual refs and
clean worktree on takeover. Save and **STOP; Task 2.26 is NOT STARTED**. No later
implementation or build is running. Next scope remaining funding provenance,
post-initialized Token-native donations and complete recipient-control evidence
before native initializer exposure; actual transport lifecycle and current
runtime/resource/rollback proof remain necessary. No credit balance is inferred.
No Mainnet action, deployment, fund movement, key creation/signing or authority
transfer occurred. Preserve the mandate's exact live-operation gates.

## Previous checkpoint — Task 2.24 (2026-09-20 UTC)

Task 2.24 is **TECHNICALLY VALIDATED / PENDING FOUNDER ACCEPTANCE**. The founder
resumed one bounded development step. Root verified `jerem` (uid 1001), one clean
integration worktree, and local/remote main plus integration at baseline
`7b74be4b13c019b96a0c8abcbebfbcc361d31089`. D-028 publication is complete and
does not cover this new task. Main stays at that baseline; Task 2.3 stays at
`3677fee97e3617ee65e2828d222008ba0952bb3e`.

[Task 2.24](TASK_2_24_GENESIS_TOKEN_PREFUND_NORMALIZATION.md) adds a distinct
same-call initialization path that moves only native excess from still-empty,
System-owned Token target PDAs into PendingSol before ownership changes. All
original rent shortfalls and the external payer debit remain unchanged: these
contributions cannot fund missing PendingSol rent. Initial history and pending
ledgers stay zero; actual authenticated reconciliation subsequently recognizes
the entire pending balance once. Existing raw APIs, non-Token prefund treatment,
approved message format, native ABI, schema, dependencies and economics remain
unchanged. Initial pause applies only to fresh state; initialized paused Config
rejects before effects. This is no general pause exception.

One delegated writer completed implementation and **16 focused tests PASS**.
Separate source/test review passed. Root inspected the frozen source/tests and
independently executed **454 host tests +1 doctest/eight gates PASS**, with no
failures, ignored tests, diagnostics or corrective retry. All 90 source inputs
match inspection and writer execution; root verified 16 gate logs, two writer
logs and three pinned tool hashes. Root evidence:
`/tmp/piv1-t224-pilot-host-20260920-a/pilot-summary.json`; writer evidence:
`/tmp/piv1-t224-writer-20260920-i_4xbogh`. Six retained transport-template inputs
remain unchanged: **nine Node tests/eight cases are retained, not rerun**.
No SBF/runtime execution was refreshed; Task 2.14 remains historical evidence.

Root owns final documentation review and normal integration publication under
D-026. Git records this task's exact commit/publication identity; verify actual
refs and clean worktree on takeover. Save and **STOP; Task 2.25 is NOT STARTED**.
No later implementation or build is running. Next scope the remaining funding
provenance, recipient control and post-initialized Token-native donations before
native initializer exposure; actual transport lifecycle and current runtime,
resource and rollback evidence remain necessary. No numerical credit balance is
inferred. No Mainnet action, deployment, fund movement, key creation/signing or
authority transfer occurred. Preserve the mandate's exact live-operation gates.

## Previous checkpoint — Main integration published (2026-09-19 UTC)

The founder instructed: "If everything is good, publish to main on GitHub please"
(English translation). D-028 records **TECHNICALLY VALIDATED / FOUNDER-AUTHORIZED
MAIN INTEGRATION** of Tasks 2.20–2.23 at
`3282e1ebabcb0cd88491d48a391565b8b100afa7`, plus this reviewed authorization record.
This is explicit integration authority, not broader founder acceptance or Testnet
readiness. Root verified `jerem`, clean integration and the four-commit ancestry
from main `5a067ad34d2f2ab0a27f2c6bff312a8b1f772d24`.

Root verified 94 unchanged inputs (90 Rust, two CJS, two spike manifests), 20
retained logs, prior **448 host tests +1 doctest/eight gates** and **9 Node tests /
eight transport cases**. Separate combined source review and targeted hook/CI,
secret/generated-file checks passed. **No tests were rerun in this publication
turn.** Evidence: `/tmp/piv1-main-integration-2-20-2-23-20260919/evidence-check.json`.
Historical Task 2.14 SBF evidence still covers only its earlier artifact.

After final documentation review, root records one documentation-only commit,
fast-forwards main and atomically publishes main plus integration. Git records
that commit and actual publication identity; verify local/remote refs and clean
worktree on takeover. No follow-up task starts: **Task 2.24 is NOT STARTED; STOP
after publication**. No source or build work is running. Native initializer
exposure, economic prefund/funding and recipient constraints, actual transport
lifecycle and current runtime/resource/rollback proof remain deferred. No
Mainnet action, deployment, fund movement, key creation/signing or authority
transfer occurred or is authorized by this publication instruction.

The checkpoints below are HISTORICAL records of their original sessions; their
pending-acceptance and integration-only restrictions are superseded only to the
extent of D-028's explicit main integration authority.

## Previous checkpoint — Task 2.23 published (2026-09-19 UTC)

Task 2.23 is **TECHNICALLY VALIDATED / PENDING FOUNDER ACCEPTANCE**.
The founder requested one further bounded step after Task 2.22 publication.
Verified baseline: `jerem` (uid 1001), one clean worktree on
`integration/piv1-testnet`, local/remote `208b7fb4b7f597fe409429c3a7483b12f73a66af`.
Accepted main remains `5a067ad34d2f2ab0a27f2c6bff312a8b1f772d24` and Task 2.3
remains `3677fee97e3617ee65e2828d222008ba0952bb3e`. No numerical usage balance is
inferred. Save and STOP after reviewed integration publication; Task 2.24 is NOT
STARTED. No later implementation or build is running.

[Task 2.23](TASK_2_23_GENESIS_TRANSPORT.md) adds a host-only unsigned transport
harness for the exact synthetic Task 2.22 topology/model payload. With distinct
receivers, the legacy execute packet is 1,334 bytes against the pinned legacy/v0
1,232-byte limit; one synthetic outer ALT loading sixteen target PDAs reduces v0
execution to 874 bytes. Shared receivers reduce these to 1,301/841 bytes. Upload
through pinned Squads buffer create/extend/from-buffer packets fits, with exact
compact-message hash/length reconstruction. Both external payer and guardian
executor remain static signers with zero signature placeholders. The Squads
vault remains an inner PDA signer and outer nonsigner. No stored-message lookup,
Rust, dependency, schema, economics or native ABI change. This is wire evidence,
not actual Squads execution or native initializer exposure.

One delegated writer completed the harness, tests and report. Separate source/test
review passed. Root inspected source/tests and independently executed **9 Node
tests PASS** plus the deterministic eight-case report; its JSON hash matches the
writer's. Each first test run found the same test-oracle arithmetic error: the
from-buffer packet is 421 bytes, not 420. The independently verified correction
passed one retry each; original failed logs remain. Exact privilege checks were
also tightened before first execution. Final runs have no failures, skipped tests
or stderr diagnostics. Root evidence: `/tmp/piv1-t223-pilot-host-20260919-b`;
writer: `/tmp/piv1-t223-writer-retry-20260919-kpkz8j1o`.

Root verified the retained pinned Node binary, all 78 web3.js archive files against
the locked npm cache archive, and all 90 unchanged Task 2.22 Rust inputs before/
after validation. **448 host tests +1 doctest/eight Rust gates are retained Task
2.22 evidence, not rerun here.** Historical Task 2.14 SBF evidence still covers
only its earlier artifact. Root owns final documentation review/publication
checks. Git records the exact task commit/publication identity; verify actual
refs and clean worktree on takeover. Tasks 2.20–2.23 remain pending founder
acceptance on integration; main stays unchanged.

Next session: scope the remaining funding/prefund normalization and recipient
constraints before exposing a native initializer; then current-source runtime,
rollback and resource evidence. The synthetic ALT still needs a separately
verified creation/funding/authority/content/warm-up lifecycle; actual signatures,
Squads approval/buffer execution and external preparation rent/refunds are
unproven. Buffer rent closes to its creator under pinned Squads, not automatically
to the external payer; no PIV1 reimbursement rule is introduced. No RPC/chain
operation, Mainnet action, deployment, fund movement, key creation/signing or
authority transfer occurred. Preserve exact D-026 live-operation gates. Stop here.

## Previous checkpoint — Task 2.22 published (2026-09-19 UTC)

Task 2.22 is **TECHNICALLY VALIDATED / PENDING FOUNDER ACCEPTANCE**.
After Task 2.21 publication, the founder explicitly requested a little more
progress, superseding that task's planned STOP for one additional bounded task.
Verified clean integration baseline:
`cbe5611f625b3e1be4acf38f25cca7dc5f0defb9`, independently matched on the remote.
User remains `jerem` (uid 1001), one worktree in `/home/jerem/piv1`.
Accepted main is `5a067ad34d2f2ab0a27f2c6bff312a8b1f772d24`; Task 2.3 stays
`3677fee97e3617ee65e2828d222008ba0952bb3e`. No numerical usage balance/budget or
active goal is inferred. Save/STOP after reviewed publication; no later
implementation or build is running. Task 2.23 is NOT STARTED.

[Task 2.22](TASK_2_22_GENESIS_ACCOUNT_INITIALIZATION.md) completes fresh approved
allocation, both canonical legacy Token InitializeAccount3 CPIs and all nine
typed genesis state envelopes in one library call. Token roles/mint borrow checks
precede effects; the same fresh Rent/model is preserved; no detached receipt is
accepted. All nine state buffers validate and are borrowed before copying.
Exact full-target/payer/mint postconditions and final fixed authentication pass.
The function remains undispatched. Raw prefunds stay unclassified, including
Token-native excess that still blocks the separate economic accessor. No schema,
dependency, economic decision or native instruction ABI changed. Every error
must propagate; host staged discard is not actual transaction rollback proof.

Reused writer `genesis_writer` completed implementation/report and ten focused
tests. Separate reviewer `genesis_review` passed scope and frozen source/tests,
including preservation of the first Token vault during the second initialization.
Root inspected actual code/tests and executed **448 host tests +1 doctest /
eight locked/offline gates PASS**, zero failures, ignored tests or diagnostics.
All 90 inputs match the inspected/writer freeze and remained unchanged; sixteen
gate logs, both focused logs and three unchanged host tools were verified.
Root evidence: `/tmp/piv1-t222-pilot-host-20260919-a/pilot-summary.json`;
writer evidence: `/tmp/piv1-t222-writer-20260919-32mv1qub`. No corrective retry.
Historical Task 2.14 SBF evidence still covers only its earlier artifact.

Root owns final documentation/evidence review and targeted publication checks.
Git records the exact task commit and integration publication identity; verify
HEAD, remote refs and clean worktree on takeover. Tasks 2.20–2.22 are not founder-
accepted and main remains unchanged. No RPC/chain operation, Mainnet action,
deployment, fund movement, key creation/signing or authority transfer occurred.
D-026's exact sensitive-action gates remain unchanged.

Next session: assess remaining native-initializer readiness, especially Token-
native prefund normalization, operational funding provenance, recipient constraints
and the exact Squads transaction transport/account budget. Then scope the required
native boundary and current-source SBF/runtime rollback/resource evidence before
exposure. Completed library bytes are not a Testnet handover milestone. Stop here.

## Previous checkpoint — Task 2.21 published (2026-09-19 UTC)

Task 2.21 is **TECHNICALLY VALIDATED / PENDING FOUNDER ACCEPTANCE**.
The founder returned and requested bounded progress in this existing chat.
Verified baseline: `jerem` (uid 1001), one clean worktree on
`integration/piv1-testnet` at `748faf81e5bd8f22c05b7588d7d4bb6d14d42848`, with
matching remote integration. Accepted local/remote main remains
`5a067ad34d2f2ab0a27f2c6bff312a8b1f772d24`; Task 2.3 remains
`3677fee97e3617ee65e2828d222008ba0952bb3e`. No goal or numerical usage budget is
active. This checkpoint closes one economical task with reviewed publication;
no later implementation/build is running. Task 2.22 is NOT STARTED.

[Task 2.21](TASK_2_21_GENESIS_ACCOUNT_ALLOCATION.md) composes fresh Task 2.20
approval/preflight with a distinct signing System payer. It pays only checked
rent shortfalls for all sixteen targets while preserving the payer rent floor,
then allocates/assigns eleven data accounts through canonical PDA-signed System
CPI. Exact full-batch postchecks follow every call. Raw prefunds stay untouched
and unclassified; no economic ledger, schema, dependency or runtime ABI changes.
This allocation-only intermediate is absent from native dispatch. Both Token
accounts and all nine states MUST be initialized in the same successful outer
transaction; bare Token allocation permits takeover. Every execution error must
propagate. Host staged discard is not actual runtime rollback evidence.

Delegated writer `genesis_writer` completed implementation/report and 12 focused
tests. Separate reviewer `genesis_review` passed scope and frozen source/tests.
Root inspected actual source/tests and executed **438 host tests +1 doctest /
eight locked/offline gates PASS**, zero failures, ignored tests or diagnostics.
All 88 source inputs match the inspected/writer freeze and stayed unchanged;
sixteen gate logs, both focused logs and three host tools were verified.
Evidence: `/tmp/piv1-t221-pilot-host-20260919-a/pilot-summary.json` and the report;
writer evidence: `/tmp/piv1-t221-writer-20260919-_5e4tkwr`. No corrective retry.
Historical Task 2.14 SBF evidence still covers only its earlier artifact.

Root owns final documentation review/publication checks. Git records the exact
task commit and integration publication identity; verify HEAD, remote refs and
clean worktree on takeover. Main stays at the founder-accepted milestone plus
acceptance records; neither Task 2.20 nor 2.21 is founder-accepted. No RPC/chain
operation, Mainnet action, deployment, fund movement, key creation/signing or
authority transfer occurred. Preserve exact D-026 live-operation gates.

Next session: scope safe same-transaction Token initialization, all genesis state
serialization and fresh post-initialization validation. Resolve remaining
recipient/funding and transport/resource constraints before exposing a native
initializer. Operational funding baseline and prefund classification remain
separate; no new economic decision is inferred. Save and STOP after this task.

## Previous session checkpoint — STOP after Task 2.20

The founder requested one economical development session followed by a saved
checkpoint and STOP. Task 2.20 is **TECHNICALLY VALIDATED / PENDING FOUNDER
ACCEPTANCE**, within its read-only source/host scope. Implementation and validation
are complete; this checkpoint closes reviewed publication. No later implementation
or build is running. Task 2.21 is NOT STARTED; resume only when the founder returns.
Verified session baseline: user `jerem`, uid 1001, branch
`integration/piv1-testnet`, one clean worktree; local/remote main and integration
both `5a067ad34d2f2ab0a27f2c6bff312a8b1f772d24`. Task 2.3 remains `3677fee`.
The goal tool returns no goal; no numerical usage budget or balance is inferred.

Task 2.20 technical scope: fresh approved genesis/Jito identity composition from
the same exact approved bytes, account slice and runtime context; all sixteen
canonical writable currently unallocated System targets; checked rent floors, raw prefunding
observations and per-target/aggregate shortfalls. The output is read-only
point-in-time evidence, not a creation/funding capability. No CPI, account
creation, funding classification, recipient-control proof, new instruction ABI,
schema or dependency is included. Preserve all economic ledgers and inputs.

One writer `squads_prerequisite` completed the module, ten focused tests and
[task report](TASK_2_20_GENESIS_ACCOUNT_PREFLIGHT.md). Separate reviewer
`review_squads` passed the frozen source/tests; root inspected actual code/tests
and executed **426 host tests +1 doctest / eight locked/offline gates**, with no
failures, ignored tests or diagnostics. All 86 source inputs were preserved and
matched the inspected/writer freeze; sixteen gate logs, two focused logs and
three unchanged tool hashes were verified. Writer evidence is ten passing tests,
separate from root's workspace execution. Evidence and exact commands:
`/tmp/piv1-t220-pilot-host-20260914-a/pilot-summary.json` and the task report.
The old Task 2.14 artifact's 24 SBF tests/70 cases remain historical evidence.

Root owns shared status documents and final publication checks. The task commit
is recorded in Git history and published only to integration; on takeover verify
HEAD, clean worktree and remote refs. Accepted main remains `5a067ad34d2f2ab0a27f2c6bff312a8b1f772d24`.
No Mainnet action, deployment, fund movement, key creation/signing or authority
transfer occurred. No dependencies, serialized layout or runtime instruction ABI
changed. The existing exact live-operation approval boundaries remain in force.

Next session: define a bounded prefunding-safe atomic account-creation task,
including explicit funding provenance, recipient/transport constraints and
post-creation validation. Preflight observations do not authorize spending,
establish historical noninitialization or prove recipient control/operational
funding. No further task starts in this session.

## Accepted main milestone

Execution: **FOUNDER-ACCEPTED — Tasks 2.3–2.19 integrated into main**.
On 2026-09-14 UTC the founder explicitly accepted the milestone at
`d9f3371be6ecb586675e3b38edcc57bd6e9519f8` and authorized its integration into main
(D-027). Root published that exact commit by normal fast-forward and independently
verified remote main/integration at `d9f3371`. This acceptance-record update is
documentation only. At that acceptance checkpoint Task 2.20 was not started;
the new short session is recorded above. D-026 remains active without routine
approval gates.

Review baseline: `1bf07eae13d90744c9c18e7dc3f5543185bb6284`, the published receipt
after Task 2.19 closure `e9c1b991fbd5066630612848c33ba5fbdb21776a`.
Root independently verified remote integration/main/Task 2.3 refs and a clean
starting worktree. Actual user `jerem`, uid 1001, one worktree in
`/home/jerem/piv1` on `integration/piv1-testnet`. The goal tool returned
**no goal**; historical `usageLimited` reports are not a current credit estimate.
No billing balance is inferred and no goal controls were changed.

## Read on takeover

Verify actual user, Git branch/HEAD/worktree and agents before acting. Preserve
unexpected work. Read `AGENTS.md`, this checkpoint and the current task report.
Authority: decisions > master specification > execution plan, with the newest
explicit founder decision controlling its exact component. The
[technical pilot mandate](PIV1_TECHNICAL_PILOT_MANDATE.md), D-026, authorizes bounded
successive technical work, normal commits and reviewed development publication.
Technical validation is never founder acceptance.

Founder reports are brief French; code/docs/delegation/commits are English.
The founder requested economical credit use; no numerical budget was supplied.
Reuse unchanged tool/package reviews and saved evidence. Run required validation
on changed inputs without redundant broad reads or unnecessary agent turns.

## Git and ownership

- Session baseline: clean local/remote integration
  `212e5a9fa4de2316e7abe2f5ff5cfe837c29b490`; independent remote reads matched all
  three refs. Task 2.18 implementation is
  `dc51450396a0e369e690d9038b7dde1a80e2ecd6`, published closure
  `f4bed3a44bbe4da4cf1e5304f04e2caeb85d83f1`. Normal atomic fast-forward push
  completed; independent integration/main/Task 2.3 remote reads matched and
  worktree was clean before this receipt/assessment checkpoint.
- Task 2.19 implementation: `b9f6f43d21713b3ec0bf81403378819f7cd3e44e`; published
  closure `e9c1b991fbd5066630612848c33ba5fbdb21776a`. Separate source and final
  documentation review passed. Normal atomic fast-forward push from `f4bed3a`
  completed; all three remote refs matched and the worktree was clean.
- New accepted implementation milestone:
  `d9f3371be6ecb586675e3b38edcc57bd6e9519f8`. Main was normally fast-forwarded
  from former accepted `66193769d1cbc59cd8630df295b9a784b9c64642` to that exact
  commit; an independent remote read verified main and integration at `d9f3371`.
  The subsequent reviewed acceptance record changes documentation only; normal
  publication maintains that record on both branches. Its identity is in Git history;
  keep main at the accepted milestone and these acceptance records. Later code
  continues on integration and requires its own eventual founder acceptance.
- Task 2.3 branch tip, local and remote:
  `3677fee97e3617ee65e2828d222008ba0952bb3e`.
- Remote: `github-piv1:HoldTheFuckingPosition/PIV1.git`. All three refs were
  independently verified after publication. Normal atomic fast-forward only.
- Writer `squads_prerequisite` owns seven acceptance/status documents; separate
  reviewer `review_squads` checks that delta and the root-owned instructions and
  checkpoint. Their earlier source-seam/recap review passed with no additional
  actionable foundation-level blocker. Root owns evidence verification and Git.
  Reuse these agents when available; no later implementation or build is running.
- Targeted secret/generated-file and hook/automation checks passed on the scoped
  change. No custom hooks path, non-sample hooks or `.github`/`.cargo` automation
  was found. These checks were repeated before main integration. The current change is
  documentation only; preserve any unexpected changes before publication.
- Prior preserved bytecode remains outside Git at
  `/tmp/piv1-t214-preserved-bytecode-20260909-a`; no source was discarded. Earlier
  publication receipts and evidence remain in their task reports/Git history.

## Founder acceptance and main publication

**CONFIRMED — D-027.** The founder's instruction, translated into English:
"I accept the 2.3–2.19 milestone at commit d9f3371 and authorize its integration
into main." The full accepted commit is recorded above; no additional source
change is included. This accepts the delivered bounded foundation, not complete
Testnet readiness or unsupported protocol/initialization/lifecycle behavior.

The accepted commit was pushed with `git push --atomic origin
d9f3371be6ecb586675e3b38edcc57bd6e9519f8:refs/heads/main`, followed by independent
`git ls-remote` checks. All 84 source hashes still match the reviewed 416-host-test
freeze. The acceptance delta touches only decisions, current technical statuses,
the two READMEs, recap notices, instructions and this checkpoint. Root verifies
the reviewed doc freeze, local links, whitespace, secret/generated-file scope,
hooks and final refs before closure; no new tests/builds are required for it.

The ordinary local tracking fetch could not write root-owned `.git/FETCH_HEAD`.
Root preserved that file and used `git fetch --no-write-fetch-head` instead;
no permissions or ownership were changed. Main publication had already succeeded;
this local metadata failure was not a test failure or rejected publication.

No PR was created. The earlier PR/API limitation no longer blocks this direct,
explicitly authorized Git integration. No Mainnet action, deployment, fund
movement, key creation/signing or authority transfer occurred. The exact live
approval boundaries remain unchanged.

## Historical recap review and PR preparation

The [integration review](PIV1_INTEGRATION_REVIEW_2_3_TO_2_19.md) maps delivered
layers, exact two-selector reachability and remaining dependencies. The
[prepared PR text](PIV1_INTEGRATION_PR_2_3_TO_2_19.md) targets `main` from
`integration/piv1-testnet`; it does not grant merge or founder acceptance.
At review time, source review found no foundation-level integration blocker.
IR-001 corrected
obsolete technical status in both READMEs and master progress summaries without
changing economics. Final separate source/documentation review passed.
Review scope, evidence and commands are recorded in the integration report;
normal publication changes seven documentation files on the integration branch.
The recap commit is recorded in Git history rather than duplicated inside its
own contents. On takeover verify HEAD, clean worktree and remote refs before acting.

Root verified unchanged evidence instead of rerunning the suite: 84 current
source/manifest/lock/toolchain hashes match Task 2.19's **416 host tests +1 doctest
/ eight gates**; all sixteen gate logs and three host tools match recorded hashes.
The Task 2.14 ELF still matches its 229,208 bytes and recorded SHA-256; its
**24 local SBF tests /70 cases** remain historical, not current-source evidence.
Receipt: `/tmp/piv1-integration-review-20260914-a/evidence-reuse.json`.
No build or test execution occurred during this documentation-only recap.

Git SSH publication works, but no authenticated GitHub API, callable GitHub app,
`gh`/`hub` binary or configured CLI authentication is available. Discovery checked
only tool availability/configuration presence; no credential value was inspected.
Automatic PR creation is unavailable. A public read-only GitHub API check found
no open PR for this exact head/base. Reviewed English title/body and the prefilled
creation link are complete in the integration report; **no PR was created**.
The earlier next action was to open the prepared PR; the founder's later D-027
acceptance and explicit main instruction supersede that workflow. The prepared
PR remains historical evidence and is not needed to complete this integration.
No Mainnet action, deployment, fund movement, key creation/signing or authority
transfer occurred during the recap review. Main was unchanged then; the newer
accepted publication is recorded above.

## Preserved Task 2.16 — bounded Squads invocation authorization

**COMPLETE / FOUNDER-ACCEPTED within the recorded scope (D-027)**.
Implementation: `343a496fb16edc8fd8d68746a89323d7545ab36d`.
[Task report](TASK_2_16_SQUADS_INVOCATION_AUTHORIZATION.md).
One writer completed the four-file source/test change and report. Separate scope,
source/test and writer-report review passed. Root inspected all production/tests/
fixtures, executed **383 host tests +1 doctest / eight gates**, and verified all
sixteen log hashes, source preservation, reviewed freeze and existing tool hashes.
Evidence: `/tmp/piv1-t216-pilot-host-20260914-a/pilot-summary.json`.
The writer's separate focused execution passed 20 tests; root verified its two logs.
No failure/ignored test or diagnostic occurred in either execution.

T216-R1 is resolved: the eligible signer derives from the authenticated outer
instruction, preserving executor flexibility. Three executors use identical
approved inner bytes without an inner executor account. The narrow profile binds
fresh same-program governance and Clock evidence to current nonstale four-voter
approval, exact direct single-inner message and current outer Squads execution.
No handler or state mutation exists. Injected context/statuses are modeled host
evidence; actual Squads CPI, persisted effect-once/rollback and resource limits
remain unproven. No SBF build/artifact refresh, initialization, rotation, schema,
error ABI, dependency, source pin or live operation changed.

Separate final shared-documentation/evidence review passed with no findings.
Targeted secret/generated-file and hook/automation checks passed. Normal atomic fast-forward publication completed at
`cadff2b0fbdbaee278beac4d44f2331b552e6acb`. Root independently read remote integration,
main and Task 2.3 refs; all matched local refs and worktree was clean. This following
checkpoint records that receipt. Accepted main was unchanged at that publication.

## Preserved Task 2.17 — separate bootstrap authorization

**COMPLETE / FOUNDER-ACCEPTED within the recorded scope (D-027)**.
Implementation: `69f7289e4c3ea2821141c4bcaded5ae942eed979`.
[Task report](TASK_2_17_SQUADS_BOOTSTRAP_AUTHORIZATION.md).
The single writer changed only the existing Squads module, nine new tests and
report. Separate source/test review passed with no findings. Root read the full
delta/tests, executed **392 host tests +1 doctest / eight gates**, verified sixteen
log hashes, full source preservation, reviewed freeze and unchanged tool hashes.
Evidence: `/tmp/piv1-t217-pilot-host-20260914-a/pilot-summary.json`.
Writer focused evidence is 29 passing tests (9 new +20 unchanged); root verified
both logs. No failed preparation, retry, failure, ignored test or diagnostic.

The new boundary requires a canonical writable, nonexecutable, System-owned empty
Config PDA and fresh exact Squads approval. Prefunding remains untouched. Existing
Task 2.16 guardian authentication and bounded parsers are preserved. Sorted-member
approval evidence has no PIV1 slot/revision/activity or parameter-semantics meaning.
Repeated virgin checks may succeed. No creation, handler/ABI/schema, new dependency,
transport, target build, source pin or live operation changed. Separate
documentation/evidence review passed with no findings. Normal atomic
publication completed through `09a02cbe485ab8abd7cb55f155539002df9251a8`; root
independently verified remote integration/main/Task 2.3 refs and clean worktree.
This following receipt-only checkpoint records that completed publication.

## Preserved Task 2.18 — approved genesis model preparation

**COMPLETE / FOUNDER-ACCEPTED within the recorded scope (D-027)**.
Implementation: `dc51450396a0e369e690d9038b7dde1a80e2ecd6`.
[Task report](TASK_2_18_APPROVED_GENESIS_MODEL.md).
One writer completed five source/test files and the report. Separate scope and
final source/test/report review passed with no actionable findings. Root inspected
all changed source/tests, executed **404 host tests +1 doctest / eight gates**, and
verified sixteen final logs, both writer logs, the full 77-file writer freeze,
source preservation and unchanged previously verified host tools. No failed run,
retry, failure, ignored test or diagnostic occurred. Writer evidence: 41 focused
tests (12 new +29 preserved). Root evidence:
`/tmp/piv1-t218-pilot-host-20260914-a/pilot-summary.json`.

The strict 313-byte model-domain format binds approved slot/pause/anchor and
unverified protocol/recipient declarations to fresh Task 2.17 authorization using
identical bytes/accounts/runtime ID and one Clock. Derived proposed state has
immutable economics, initial slippage exactly 1 bps, zero histories/revision,
Idle distribution and six inactive zero-liability rewards. Sixteen descriptors
specify intended addresses/owners/sizes; expanded aliases preserve valid overlaps.

This remains a model, with no official protocol/recipient authentication, actual
non-Config target validity, rent/funding provenance, creation/persistence, handler,
new ABI/schema/dependency, target build or live action. KIF approvals do not prove
vote timing/activity; inactive model records do not select a vote-exclusion policy
or establish first-payout readiness. Prefunding remains untouched and unclassified.
Task 2.14 SBF artifacts are historical, not evidence for this newer source.

## Last completed technical task: 2.19 — source-pinned Jito account identity

**COMPLETE / FOUNDER-ACCEPTED within the recorded scope (D-027)**.
Implementation: `b9f6f43d21713b3ec0bf81403378819f7cd3e44e`.
[Task report](TASK_2_19_JITO_ACCOUNT_IDENTITY.md).
One writer completed the source, tests, extracted oracle/license, test-only
manifest/lock edges and report. Separate scope and final source/test/dependency/
report review passed with no actionable findings. Root inspected all source/tests,
verified exact oracle excerpts and source archives, checked the five lock edges
and all 168 unchanged package identities, then executed **416 host tests +1
doctest / eight gates**. Sixteen final log hashes, both writer logs, full source
preservation, the writer/inspected freeze and unchanged tool hashes were verified.
Evidence: `/tmp/piv1-t219-pilot-host-20260914-a/pilot-summary.json`.
Writer focused evidence: 12 passing tests, including 432 parser combinations.
No Cargo preparation/build/test failed; no failure, ignored test or diagnostic.

The seven-account boundary checks fixed official source identities and actual
program/pool/list/reserve/mint/manager/referrer relationships. Pool parsing is
bounded; list geometry permits residual slack without scanning entries. Canonical
withdrawal authority, default lockups, legacy mint and nonnative receiving accounts
are checked. Raw fees, optional authorities, preferences, epochs and supplies stay
separate from economic conversion and operational readiness. No genesis composition,
actual target creation, handler, ABI/schema, CPI, SBF or live operation is included.

Initial draft review removed the public test-only decoder and added the native
receiver rejection/regression. An unsupported Eq derive was removed before
compilation. The read-only source assessment's missing checksum-sidecar copy
interruption is retained in the report, separately from successful Cargo/tests.
The oracle is exact-source extracted types with actual pinned Borsh1/Stake types,
not complete SPL execution. Two pinned dev-dependencies use already locked
packages; all production dependencies and package versions/sources are preserved.
The report retains exact sources, checksums, license provenance and limitations.
Normal publication completed at `e9c1b991fbd5066630612848c33ba5fbdb21776a` after
separate final documentation review and targeted secret/generated-file/hook checks.
No sensitive operation occurred; accepted main was unchanged at that publication.

## Next dependency after Task 2.20

Task 2.20 now freshly composes approved genesis/protocol identity and authenticates
all sixteen currently unallocated targets, with checked rent-only shortfalls.
Actual prefunding-safe atomic creation, funding provenance and post-creation
validation remain next. Define a coherent bounded scope and obtain separate
technical review when the founder resumes; this is a pilot responsibility, not a
routine approval gate. Reuse prior identity, transport and state evidence.

No deployed binary/cluster identity, current pool/validator readiness, complete
liability history, actual initial vote activity or founder Testnet readiness is
established. No key/signing, live-operation or authority-transfer scope is released.
The reviewed test-only lock changes do not refresh historical SBF artifacts.

## Preserved Task 2.15

Implementation `da0241fd2a9f9c3247bbdeabb1a6b9c37dabc912`, published closure
`11f4d701f4ca58f18a64383dce5e088377afec77`, now founder-accepted within its
recorded scope under D-027. [Report](TASK_2_15_SQUADS_AUTHORITY_SNAPSHOT.md) preserves immutable
upstream sources, 14 tests, root 363 +1/eight gates, source preservation and final
separate review. Its one missing-import preparation failure occurred before tests;
failed logs and subsequent corrected final proof are retained. Current-state
configuration alone does not prove historical action approval. Existing Task 2.14
SBF artifacts remain historical. Remote refs matched and worktree was clean at
Task 2.15 publication before the Task 2.16 changes.

## Preserved Task 2.14

[Runtime pending recognition report](TASK_2_14_RUNTIME_PENDING_RECONCILIATION.md):
implementation `7442cab`, published closure `a189159`, now founder-accepted
within its recorded scope under D-027. Root executed 349 host tests +1 doctest/eight gates
and 24 local SBF tests/70 cases, with separate final review. Exact hashes, logs,
1629 account observations and limitations remain in that report and prior Git
checkpoints. These are historical executions, not a new Task 2.19 runtime claim.

## Preserved reviewed progression

Every row is now **COMPLETE / FOUNDER-ACCEPTED within its recorded scope**, under
D-027. Earlier task-report and publication statuses remain historical evidence;
their commands, exact implementation identities and limitations are unchanged.

| Task | Scope | Implementation | Root host tests + doctest |
| --- | --- | --- | --- |
| 2.3 | Atomic vault reconciliation and severe-loss correction T23-R1 | `0559ebd` | 168 +1 |
| 2.4 | Fixed-account authentication | `9f4f106` | 191 +1 |
| 2.5 | Initial contribution bootstrap | `9b997f3` | 209 +1 |
| 2.6 | Zero-fee, book-value-preserving principal SOL deposit | `9f75aec` | 231 +1 |
| 2.7 | Isolated earned KIF authentication/accounting | `10dceb5` | 255 +1 |
| 2.8 | Current guardian/Clock snapshot authentication | `c815474` | 277 +1 |
| 2.9 | Typed envelopes and existing-account persistence | `36152c1` | 298 +1 |
| 2.10 | KIF execution with explicit host invocation modeling | `c38a7b0` | 318 +1 |
| 2.11 | Claim ABI/runtime-ID/error/event boundary | `2eeefba` | 335 +1 |
| 2.12 | Keyless SBF compilation; reviewed stack correction | `cee6072` | 339 +1 |
| 2.13 | Keyless local SBF KIF execution | `fd57359` | Unchanged 339 +1; 19 local tests /60 cases |

Task 2.3's original reported 164 tests are historical executor evidence. Root also
executed that baseline, reproduced severe-loss T23-R1, verified four failing-old/
passing-corrected regressions and preserved HWM/recovery/custody invariants.
Accepted Phase 0/1 and Tasks 2.1/2.2 remain as recorded in canonical decisions;
D-027 adds explicit founder acceptance for Tasks 2.3–2.19. Technical passes alone
still cannot extend acceptance to later work.

## Remaining delivery limits and permissions

Follow the [requirements-to-evidence checklist](PIV1_TEST_PLAN.md). Complete actual
authorized initialization, remaining contribution/distribution/activity/governance
handlers, atomic runtime custody, failure/retry/pause behavior, real SPL/Jito
adapter and a usable founder workflow before claiming complete Testnet readiness.

- **OPEN:** deposit fees or historical integer value loss currently reject;
  some SOL remains queued. No implicit subsidy or HWM exception is selected.
- Operational surplus lacks authenticated funding provenance. Native excess in
  Token/temporary accounts and general Idle pending-SOL priority still need
  broader integration; Task 2.14 only preserves excess during pending recognition.
- Official pool/mint/list/source authentication, collision-safe snapshot identity,
  protected instruction mapping, real multi-leg ordering/minima/fees/slippage,
  delayed readiness and both rent returns still require protocol/runtime proof.
- Current guardian reward records do not prove the global historical liability
  sum. Initialization/earning authority and heartbeat pause policy remain separate.
- Active/settled/recovery pending offsets have host/pure evidence; new runtime
  fixtures cover Idle/pause with synthetic initial state/token units. No actual
  SPL transfer, signatures, Bank/AccountsDB rollback, deployment verifier,
  public-cluster behavior or total resource bound is established.
- **Live authorization: NONE.** No new wallet/key creation, signing, public-Testnet
  deployment/fund-moving lifecycle, Mainnet, real funds or authority transfer.
  Prepare the exact cluster/genesis/public identities/artifact/budget/operations
  card before the mandate's live gate. Authority transfer has its own gate.
  Do not access unrelated secrets or advance main beyond founder-accepted work
  and its reviewed acceptance records.
- D-027 authorizes this milestone's normal main fast-forward and acceptance
  record publication. Later bounded development publishes to integration; no force/history rewrite,
  automatic release/tag or unrelated publication. No sensitive operation occurred.
  AI-assisted review is not a professional independent audit.

Prior details remain in task reports, committed checkpoints and the
[historical checkpoint](history/PIV1_PILOT_STATE_PRE_T213_CLOSURE_20260909.md).
Temporary logs may disappear; code/documents are durable once committed.
Other chats and missing `HTFP_MASTER_CONTEXT.md` are not assumed accessible.
