# PIV1 requirements-to-evidence checklist

**D-031 — Founder-authorized main integration (2026-09-28):** the founder
explicitly requests publication of all already-validated work missing from main.
This covers Tasks 2.33–2.39 and D-030 guidance through
`bf32d87e06a2d54c8e1c0192faaa7855d246dbc0`, plus the reviewed authorization record,
by normal fast-forward from `4cc4ea11e87c2f1a2f85b9ed48358f2a881821b3`.
Git and the [pilot checkpoint](PIV1_PILOT_STATE.md) record the resulting publication.
Status: **TECHNICALLY VALIDATED / FOUNDER-AUTHORIZED MAIN INTEGRATION**.
This is integration authority, not broader founder acceptance or live-operation
permission. At publication M2 had not started; see the current checkpoint for later
integration-only work.

This is an execution index under D-026/D-030, not a new specification or acceptance.
Canonical requirements remain in `PIV1_DECISIONS.md` and `PIV1_MASTER_SPEC.md`.
Current commit, actual executions and active task are in `PIV1_PILOT_STATE.md`.
Update this checklist when a bounded task closes; do not infer runtime evidence
from passing host tests.

## Active milestone evidence plan — D-030

The founder's six ordered production milestones in the
[execution plan](PIV1_CODEX_EXECUTION_PLAN.md) govern validation.
**M1 is complete within Task 2.39's documented initializer boundary/runtime scope;
M2 is IN PROGRESS (Task 2.45); the full M3 adapter and M4–M6 remain OPEN.** For M1, root passed 32 focused tests,
479 host tests +1 doctest/eight gates, nine runner tests and twelve actual production
Bank messages across four profiles. Root and separate review each checked 786
complete account records. Approval/reinitialization/error negatives have host
evidence; actual Bank evidence covers success, late failure, replay and retry.
Earlier claim/pending SBF and Node transport results are historical, not rerun.
See [Task 2.39](TASK_2_39_PRODUCTION_INITIALIZER.md) for artifacts and exact limits.

Task 2.40 closes the first bounded M2 source/host/static-build block: 10 focused
regressions, 489 host tests +1 doctest/eight gates and a clean strict SBF build,
with separate source/test/artifact/evidence review. New-path VM/Bank execution is
unproved; the tests model runtime context and signer privilege.
Task 2.41 adds nine grouped intake regressions, 498 host tests +1 doctest/eight
gates and clean strict SBF compilation on the corrected 98-input freeze. Tests
cover D-032 pause rejection before effects, exact native/Token transfer bytes and
postconditions, both prior deficits, surplus, self-delegate allowance/padding,
alias/borrow/arithmetic/CPI failures and full active-round preservation. The Token
callback runs the pinned host Processor; System transfer and discard/retry are
host models, not SVM/Bank rollback evidence. The initial import compile failure
and capacity-inventory failure are retained; reviewed corrections preserve guards.
See [Task 2.41](TASK_2_41_CONTRIBUTION_RUNTIME_INTAKE.md) for exact pins and recovery.
Task 2.42 passed ten grouped normalization tests, 508 host tests +1 doctest/eight
gates and strict SBF with separate review. Its host oracles cover: full six-dimension
deficits/surplus, signed fixed CPI bytes/seeds, every-CPI postconditions, no-op
idempotence, pause/recovery differences and partial multi-CPI failure/retry models.
Token-native excess and operational funding must remain unchanged; old strict
accessor tests must continue to reject unsupported Token-native surplus.
Task 2.43 passed ten focused tests, 518 host tests +1 doctest/eight gates and
strict SBF with separate review. It covers initial bootstrap composition with current
pool/Mint/Clock facts, conservative stored-supply valuation (including direct-burn
lag), exact HWM contribution accounting, native-funding quarantine and full partial
failure oracles. Legacy pure API validation and replay/history gates must remain.
Task 2.44 below covers protected principal SOL deposit with the pinned CPI and
exact pool/custody postconditions, retaining Task 2.6's zero-fee/book-value/HWM guards.
Post-settlement integration remains future work; M4 must still execute the complete
real production lifecycle. Current new-handler host/static evidence is not VM proof.
Task 2.44 source/test review, ten focused tests and 528 host tests +1 doctest/eight
gates passed. Its protected principal
deposit tests cover exact pinned CPI bytes/seeds, independent pool/Mint/custody
oracles, stored-supply burn lag, fee/rounding/HWM gates, carry/native-funding
preservation, legacy API error order and partial-effect discard/retry. The pinned
Token processor runs on host; nested stake-pool effects remain modeled. Strict
SBF passed; reviewed publication is integration-only. Next validate the
production distribution-preparation boundary and its required protocol facts;
complete local production-path execution remains the M4 gate.

Task 2.45 is technically validated on integration: production no-yield/liquid-funded
preparation uses
current pool valuation and Clock/guardian snapshots. Positive withdrawal shortfall
must reject before effects/cooldown: actual dynamic Stake minimum, source residual
and conservative multileg proofs are missing M3 prerequisites. Pending SOL funds
first, then recorded carry; preserve HWM proposals, full contribution ledgers,
liabilities and native funding. See the [Task 2.45 report](TASK_2_45_LIQUID_DISTRIBUTION_PREPARATION.md)
for scope and evidence: ten focused tests, 538 host tests +1 doctest/eight gates
and strict SBF passed with separate review. No new-path VM/Bank proof. Full
preparation, settlement and local lifecycle remain open.

| Milestone | Critical gap | Required completion evidence |
| --- | --- | --- |
| M1 — Production initializer | COMPLETE: strict native dispatch and actual production initialization/Bank evidence passed; sensitive/live and full-lifecycle limits remain. | Separate ABI/account review and actual production ELF execution; exact approved bytes, guardian/governance/protocol/recipient authentication, fresh/prefunded rent and initial pause oracles, malformed/replay rejection and existing claim/pending dispatch regressions. |
| M2 — Economic runtime handlers | IN PROGRESS: Tasks 2.40–2.45 heartbeat/pause/intake/normalization/bootstrap/staking/liquid preparation and focused gates; remaining handlers and actual local runtime coverage remain open. | Authenticated SOL/JitoSOL intake and reconciliation, pending/bootstrap integration, distribution cadence/minimum/cooldown, deterministic delayed withdrawal legs, exact-target/all-leg settlement, rent/recovery/HWM, beneficiary/KIF accounting, heartbeat/earned claims and confirmed governance/pause paths; focused transition, accounting, replay and failure tests. |
| M3 — Real pinned SPL/Jito adapter | Task 2.44's protected deposit component has host evidence; full withdrawal/finalization and real nested pool execution remain unproved. | Reviewed pinned implementation of the existing seven-method contract; protected deposit/withdrawal variants, immutable 1-bps cap, conservative outputs, exact account/authority/fee mapping, postconditions and error propagation through production paths. |
| M4 — Complete local lifecycle | No complete production-path cycle is proved by the synthetic probes. | Exact production artifact/source pins, end-to-end contribution-to-settlement/pending/KIF cycles, deterministic multi-leg readiness, rent recovery, loss/recovery and pause/failure/retry evidence; final milestone gates and independent requirement-to-evidence review. Model/stub-only success cannot close this milestone. |
| M5 — Exact Testnet package | Actual cluster/protocol support, deployment identity, funding and founder workflow remain unverified. | Independently reviewed approval card: cluster/genesis, artifact/hash and Program ID, authorities/recipients, funding plus fee/rent budget, bounded operations, recovery/stop conditions and reproducible founder workflow. Verify D-006 official Jito Testnet compatibility; do not substitute reference identity pins. |
| M6 — Explicit deployment authorization | Technical evidence does not authorize the first live Testnet deployment. | Present the exact concise card, record explicit founder authorization, and stop before live deployment until it exists. Material package changes require updated authorization; acceptance and sensitive-action limits remain separate. |

During implementation, run proportionate targeted tests for changed behavior and
demonstrated defects, then the final gates required by that milestone. Use one
writer and separate review, correct verified defects, and checkpoint reviewed
integration work. Reuse unchanged evidence only within its actual source/artifact
scope; avoid wholesale reruns or new validation-only tasks unless a demonstrated
critical-path blocker names the missing evidence and its closure criterion.
Neither test economy nor D-030 waives existing security/diagnostic gates.

M1 must authenticate the actual entire Squads-approved instruction bytes: the
`PIV1GM01` model codec does not justify stripping/reconstructing a new envelope
before approval comparison. Recipient vault indices are derivation witnesses;
probe-only 0/255 values are not a production policy. Under D-032, test that both
explicit contribution handlers reject while paused before transfer/state effects,
while direct transfers already received remain reconcilable.

The prior automatic next Bank-failure task and fixed per-session stop are
superseded. Preserve their historical evidence/limitations, conserve credits and
checkpoint interruptions. Main publication, founder acceptance and live operations
retain their separate gates.

## Previous checkpoint — Task 2.38

**TECHNICALLY VALIDATED / PENDING FOUNDER ACCEPTANCE**. Root passed fifteen final
runner regressions and the first actual Bank runtime: one test, four profiles,
twelve messages, sixteen complete ancestry-visible snapshots and 786 full account
records. Its independent decoder checks literal PDA/state bytes, loader/ELF bytes,
raw messages, every ordered CPI tuple and full account metadata. Separate source,
command, binary and independent evidence reviews passed.

The exact three Task 2.31 ELFs load through real V3 ProgramData in slot-one Bank.
Completed initialization followed by malformed caller fails at index 2; every
non-fee account restores exactly. Fees are [15000, 0, 15000] for late failure,
identical AlreadyProcessed replay and distinct successful retry. Separate original
rent is 34,779,120 fresh or 890,885 prefunded lamports; native excess normalization
is exactly 144 for prefunded/paused profiles. Full recipient/sentinel/ELF/sysvar
bytes remain unchanged. Retry CU are [1065383, 1058793, 986363, 979773], under explicit
1.4m/default 32-KiB bounds, above 200k. No ordinary-budget claim follows.

The first strict build passes with zero diagnostics. Initial runner fixtures had
one path-alphabet error; its narrowly corrected second execution passes all 15.
A preventive source fix moves independent rereads outside the scan callback lock.
Original runner/resource/socket/diagnostic guards remain intact. Old suites are
retained evidence, not reruns. All 223 inputs, tool/cache and artifact pins match.
Actual runtime covers local unsigned entry only: serialized messages are 1503–1586
bytes, exceeding public packet size. No signature/ALT, real governance or full
recipient control, failed in-initializer CPI rollback, durability or chain proof.
See [Task 2.38](TASK_2_38_BANK_GENESIS_INITIALIZATION.md). Its former next-probe
and session-stop recommendation is historical; D-030 above controls next work.

## Previous checkpoint — Task 2.37

**TECHNICALLY VALIDATED / PENDING FOUNDER ACCEPTANCE** for native System Bank
commit/account rereads. Root executed 12 final runner regressions and one Bank
test/four messages. Root and separate reviewer independently decoded raw messages
and checked 100 complete account records: 20 accounts across five snapshots.
The oracles verify persisted success, second-instruction failure after the first
transfer's observed transient debit, exact non-fee rollback and fee debit,
unchanged-state/no-fee replay rejection, and distinct successful retry on the
same Bank. Account key sets, data, owners, lamports, executable flags and rent
epochs all match. Fees are [10000, 10000, 0, 10000]; observed CU are
[150, 300, not executed, 300]. First actual runtime execution passed.

The fourth full build passes the strict diagnostics gate. Three earlier builds
and an optional targeted Cargo-check resolver failure remain failed and retained.
The reviewed fixes preserve all test oracles and warning/resource/network gates.
The runner's actual subprocess regressions cover IPC versus denied socket domains,
resource/timeout termination and output/source preservation. Source/lock/vendor
provenance and exact binary were separately reviewed; older suites were not rerun.
The short 0.04-second runtime is not a reliable peak-memory measurement.

Scope excludes PIV1/SBF initialization, signature verification, disk restart,
Bank freeze/root, ledger replay and public-chain behavior. Full initializer Bank
account/retry evidence remains next, with exact loader/features and artifact pins.
See [Task 2.37](TASK_2_37_BANK_COMMIT_ROLLBACK_SMOKE.md). Save/STOP;
Task 2.38 is NOT STARTED. Main and live-action gates remain unchanged.

## Previous checkpoint — Task 2.36

Isolated Bank dependency preparation is **TECHNICALLY VALIDATED / PENDING FOUNDER
ACCEPTANCE**. Root passed public resolution and three locked/offline metadata
observations; root and separate review checked all 589 registry archives and
25,891 extracted source files. The host graph has 552 registry packages/56 build
scripts, against 70 scripts in the full graph. No compilation, test, runtime or build
script execution occurred in these metadata checks; prior suites were not rerun.
A retained read-only audit assumption about an optional Cargo marker was corrected
without changing dependencies or byte-comparison requirements.

The source-reviewed unsigned Bank path requires actual commit and account rereads
for future runtime evidence, with fee-payer debit and replay treated separately.
Native scripts/libraries, exact ELF/genesis feature loading and measured resource
budgets remain open. Existing production/economics/locks and 162 protected inputs
remain intact. See [Task 2.36](TASK_2_36_BANK_DEPENDENCY_PREPARATION.md) for commands,
checksums and source limits. Save/STOP; Task 2.37 is NOT STARTED.

## Previous checkpoint — Task 2.35

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

Regressions cover exact recovery, deduplication, tampered inputs/archives, locks,
space loss, symlink/hardlink/xattr boundaries, interruption recovery and refusal
to overwrite. Root independently matched all manifest records, decompressed all
3902 objects before prune and rehashed all 26901 preserved files afterward.
Earlier Rust/SBF/Node results are retained evidence, not new executions. See
[Task 2.35](TASK_2_35_BUILD_CAPACITY_RECOVERY.md) for exact commands and review.
Save/STOP; Task 2.36 is NOT STARTED.

## Previous checkpoint — Task 2.34

**TECHNICALLY VALIDATED / PENDING FOUNDER ACCEPTANCE**, limited to read-only prerequisite
tooling. Root passed 19 Python regressions first attempt; separate review passed.
Coverage includes disk boundaries, available-versus-free blocks, missing/wrong-type/
symlinked paths, inaccessible metadata, malformed/absent budgets, distinct CLI
usage errors and JSON results. Synthetic files prove no genuine Bank closure.
The actual check correctly returns NOT_READY: 3301838848 bytes available, an
explicit unmeasured 8-GiB planning reserve, six missing direct candidate paths
and an existing modular runtime source. It executes no commands on any outcome.
No Rust/SBF/runtime suite was rerun. All Task 2.33 source/artifact hashes and
100 logs were reverified, including failures; older 448-log verification remains
historical. See [Task 2.34](TASK_2_34_BANK_RUNTIME_PREREQUISITES.md) for commands,
hashes, final review and limits. Save/STOP; Task 2.35 is NOT STARTED.

## Previous checkpoint — Task 2.33

Task 2.33 is **TECHNICALLY VALIDATED / PENDING FOUNDER ACCEPTANCE** for
initializer-internal resource failures and retries. Root passed **13 runner tests
and eight runtime tests/sixteen message cases**; root and separate review each
checked **2400 complete account records**. Both 35/34 profiles cover fixed
200k pre-effect exhaustion and actual first/second Token-CPI exhaustion, including
paused mixed prefunding. Exact partial raw state, original rent debit, 144-lamport
normalization, zero unwritten state buffers and protected accounts are checked.
Each actual returned-original vector then initializes successfully in a fresh
runtime at the existing 1.4m-CU ceiling/default 32-KiB heap. This is Mollusk output
discard/retry evidence, not Bank rollback or recovery from committed partial state.

A first runner execution failed two temporary-path fixtures; a first runtime
failed the two exact preflight-error expectations. Minimal reviewed test-only
corrections preserve strict guards, fixed budgets and all state/custody/trace
oracles. Final build/run and separate review passed; both failures remain recorded.
Production/probes/old harnesses/economics and dependencies are unchanged; all 119
tools/nine aliases match. No installation or old-pin refresh occurred. Earlier
448 logs are verified retained evidence, not reruns. See
[Task 2.33](TASK_2_33_GENESIS_INITIALIZATION_FAILURE_RUNTIME.md) for commands/hashes.
D-026 covers reviewed integration-only publication; main remains at `4cc4ea1`.
Broader founder acceptance, actual governance/control, funding provenance, native
initializer exposure and Testnet readiness remain open. **Save/STOP; Task 2.34 is
NOT STARTED.**

## Previous checkpoint — Task 2.32

TECHNICALLY VALIDATED / FOUNDER-AUTHORIZED MAIN INTEGRATION under D-029.
Root passed 13 mocked runner regressions and 13 local runtime cases against the
three unchanged Task 2.31 artifacts. Separate source/runner/command/exact-binary/
evidence review passed. Root and the reviewer independently decoded 2040 complete
account records, checking nine literal state envelopes, two exact Token layouts,
original payer rent debit, 144-lamport native prefund sweep, zero initial ledgers,
protected accounts, full raw/returned state and runtime Instructions/Clock/Rent.

Four successes cover both fixed 35/34-account profiles with fresh or mixed
prefunding (the latter initially paused). Compute is 1063693/1057103/984673/978083 CU
under an explicit 1.4m ceiling/default 32-KiB heap; all exceed 200k. Eight early
negative cases and one late second-instruction failure match expected errors and
exact call traces. Late failure demonstrates completed raw initialization followed
by Mollusk's returned-original output discard, not Bank/AccountsDB rollback or
failed-CPI atomicity within the initializer. Synthetic caller/restricted Token
execution does not establish actual Squads/deployed Token/ALT lifecycle behavior.

The first build failed on obsolete test Rent fields; a minimal default-Rent
comparison retained all sixteen literal rent-floor assertions. The first runtime
passed 12 cases and failed one trace-order assertion after correct account/result
checks. Pinned Agave reserves top-level trace slots before appended CPIs; the
reviewed correction checks that exact topology. Final build-c and run-b passed
with 24 verified logs each/no diagnostics. Both failed attempts are preserved;
no weakened oracle, production edit or warning suppression was used.

Seven new inputs, 143 protected old inputs, 119 host tools, nine aliases, 389
existing registry identities and nine artifacts are bound. Only the new profile
adopts independently verified installed libexpat bytes; no installation/old-pin
change occurred. Root verified 326 earlier logs; the earlier 469 host tests +1
doctest/eight gates, claim/pending runtime, Node transport and Tasks 2.29–2.31
suites remain retained evidence, not rerun. See the
[report](TASK_2_32_GENESIS_INITIALIZATION_RUNTIME.md) for commands and hashes.
D-029 authorizes the reviewed Tasks 2.24–2.32 for normal main integration after
final checks; this is not broader acceptance or Testnet readiness. Initializer
failure/resource boundaries, native exposure, full recipient control, funding
provenance and actual governance/transport remain deferred. Save/STOP;
Task 2.33 is NOT STARTED. Live-operation gates are unchanged.

## Previous checkpoint — Task 2.31

TECHNICALLY VALIDATED / PENDING FOUNDER ACCEPTANCE within build-only preparation.
Root executed 15 Rust boundary tests (nine caller/four callee/two Token) and 11
mocked runner regressions, all passing first attempt. Three doctest suites contain
zero examples; warning-denied docs pass. Four gates took 82.682023 seconds without
diagnostics. Separate source/runner/command and host-evidence review passed.

The first strict SBF build passed in 362.604063 seconds, with 34 commands and 68
verified logs. Three new ELF artifacts expose only the fixed synthetic caller,
full recipient-checked normalized initializer and restricted InitializeAccount3
Token wrapper. Root checked all sizes/hashes and 14 new inputs, 128 protected
inputs, 155 existing registry identities and six historical artifacts. Separate
actual-artifact static review passed. No loader or new SBF runtime was invoked.

The tests cover literal topology/metas/data/seeds and payer privileges, malformed
wire/roles, exact CPI error propagation, host refusal, restricted Token input,
immutable profile derivation and required third-artifact/strict-diagnostic guards.
They do not prove successful initializer or Token execution. Total heap/compute,
mutating atomicity/rollback, actual Squads/ALT governance and full recipient
control remain deferred. No production source, ABI, economics or old pins changed.
The new target profile derives only the already-verified libexpat file hash in
memory without modifying the old helper/profile or installing anything.

Root verified 250 earlier logs: prior 469 host tests +1 doctest/eight gates,
24 claim/pending SBF tests/70 cases, 15 Node tests/eight old/sixteen recipient cases,
Task 2.29's 10 boundary +9 runner tests and Task 2.30's 12 runtime +11 runner tests
remain retained evidence, not rerun. See the
[report](TASK_2_31_GENESIS_INITIALIZATION_PROBES.md) for exact commands, hashes and
limits. Save/STOP after integration publication; Task 2.32 is NOT STARTED.
Main acceptance and live-operation gates remain unchanged.

## Previous checkpoint — Task 2.30

TECHNICALLY VALIDATED / PENDING FOUNDER ACCEPTANCE for keyless local read-only
preflight execution. Root passed eleven mocked runner regressions and twelve
runtime tests/cases against the exact unchanged Task 2.29 probe ELFs. Both 32/31
profiles succeed through actual height-two SBF CPI. Eight invalid callee cases,
one direct-height rejection and one outer-caller rejection have precise expected
outcomes and traces. Complete supplied/raw-before/raw-after/returned account
preservation is independently checked across 1674 account records, including the
actual runtime-generated Instructions bytes and its metadata. Clock/Rent are
runtime sysvars; no host-context preflight result supplies the success oracle.

The first host build failed before tests with two E0433 module errors from reused
test support. A minimal new test-root re-export fixed them; the fresh second build
passed without diagnostics, followed by a successful first runtime execution.
Separate source/runner/build/exact-binary/command/evidence review passed. Success
uses 288277/282070 CU with default 32-KiB heap and explicit 1.4m ceiling, exceeding 200k.
No ordinary-budget claim, complete initializer-resource/rollback proof or actual
Squads governance/recipient-control evidence follows.

Root verified 128 older logs, 64 retained probe-build logs and 8 retained probe-host
logs; production 469 tests +1 doctest/eight gates, 24 claim/pending SBF tests/70 cases,
15 Node tests/eight old plus 16 recipient cases, and 10 probe host +9 old runner
tests remain retained evidence, not rerun. One installed libexpat hash is verified
through signed Ubuntu metadata/package bytes and bound only in the new profile;
no installation or historical-pin update occurred. See the
[report](TASK_2_30_GENESIS_PREFLIGHT_RUNTIME.md). Save/STOP after reviewed integration
publication; Task 2.31 is NOT STARTED. Main and live-operation gates remain.

## Previous checkpoint — Task 2.29

TECHNICALLY VALIDATED / PENDING FOUNDER ACCEPTANCE within build-only preparation.
Two isolated probes bind the unchanged read-only 32/31-account recipient preflight
and a synthetic caller. Meaningful tests cover exact roles/metas/data/seeds,
truncations and raw count overflows, privilege changes, canonical identity and
host rejection. Nine mocked runner tests cover source/helper/configuration/lock
refusals, zero-exit diagnostics, malformed ELF/archive and independent output audit.

The first delegated host compilation failed before tests; reviewed test-only
fixture/dependency corrections then passed. Writer and root each passed 10 Rust
boundary tests +9 runner tests. Root's four gates also include doctest discovery
(zero examples) and warning-denied docs. Final strict workspace SBF compilation
passed; separate static inspection bound both artifacts to actual output bytes.
No probe runtime, real Squads/control, total genesis resources or rollback proof.

Root verified 92 production/105 existing harness inputs, eight transport inputs,
128 retained logs and tools/artifact hashes. Earlier 469 host tests +1 doctest/
eight gates, 24 SBF tests/70 cases and 15 Node tests/eight old plus sixteen recipient
cases remain retained evidence, not rerun. See the [report](TASK_2_29_GENESIS_PREFLIGHT_PROBES.md)
for commands, initial failure, hashes and boundaries. Save/STOP after integration
publication; Task 2.30 is NOT STARTED.

## Previous checkpoint — Task 2.28

TECHNICALLY VALIDATED / PENDING FOUNDER ACCEPTANCE under D-026.
The strict current-source target build exposed seven oversized genesis frames (243 diagnostics despite Cargo exit zero). Private
preflight indirection resolved the caller frames; the second build still rejected
three shared-dispatch frames (45 diagnostics). Two separate producer/boxing
boundaries then passed the unchanged strict compiler gate with no diagnostics.
Both rejected attempts remain recorded; no safety gate was weakened.

The final source independently passed 53 writer-focused tests and root 469 host
tests +1 doctest/eight gates, with separate source/test review. Two additional
private heap requests measure 2272 bytes under the host layout; this is no total
genesis heap/runtime proof. A compile-time footprint guard and host nested-result
bounds complement, but do not replace, actual target diagnostics.

The reviewed current artifact is 229888 bytes, SHA-256
`0eb5e62389c9baa5311fddca99d1e705f86b1fd698e869a8cdcec778aa68cd54`.
Fresh current-artifact validation passed 24 local SBF tests/70 cases after
separate artifact/executable review. Root independently checked 1629 complete
account records, including raw effects versus Mollusk returned-output discard.
No signed cluster or Bank/AccountsDB rollback proof follows. Root also passed
16 target-runner and seven harness-runner refusal tests in this task.
The 15 Node tests/eight old plus sixteen recipient cases are retained Task 2.27 evidence after verification
of unchanged transport inputs/tools/logs; they were not rerun. Native initializer,
total genesis resources, actual Squads transport/CPI and live readiness remain
unproved. See [Task 2.28](TASK_2_28_CURRENT_SBF_RUNTIME_REFRESH.md).

## Previous checkpoint — Task 2.27

Task 2.27 is TECHNICALLY VALIDATED / PENDING FOUNDER ACCEPTANCE on integration.
Writer and root independently executed 15 Node tests plus eight old-profile and
sixteen recipient-profile CLI cases, first attempt with no failures, skips or
diagnostics. All 96 source/manifest inputs match both executions and inspection.
The old CLI golden is byte-identical; the complete 35/34-account Task 2.26 profile
binds literal PDA seeds, full account order/privileges and exact 313-byte payload.
Negative cases cover missing/swapped/substituted/privilege-altered recipients,
approved bytes, stale topology, lookup reconstruction and buffer commitments.
T227-R1 changed a duplicate-key substitution into a unique unrelated key with the
specific canonical identity error; corrected before execution, no failed run.
Separate source/test review passed.

Real SDK roundtrips measure buffered creation and v0 execution below 1232 bytes
with the synthetic 16-target ALT. Neighboring minimum-fit/oversized cases are
measured separately from SDK-refused candidates; compute/heap prefixes are sizing
illustrations only. Main packets are 940/907 bytes, or 988/955 with prefixes. No
signatures, real ALT/buffer lifecycle, native initializer, account-control proof
or runtime/resource/rollback execution is established.

Root verified twelve new logs and sixteen retained Rust logs. **468 Rust tests
+1 doctest/eight gates** remain Task 2.26 evidence with all 92 Rust inputs unchanged,
not rerun. Historical Task 2.14 SBF evidence remains restricted to its old artifact.
Root evidence: `/tmp/piv1-t227-pilot-host-20260920-a/pilot-summary.json`;
writer: `/tmp/piv1-t227-writer-20260920-e_q97383`. See the
[report](TASK_2_27_RECIPIENT_CHECKED_GENESIS_TRANSPORT.md). Save/STOP after reviewed
integration publication; Task 2.28 is NOT STARTED.

## Previous checkpoint — Task 2.26

Task 2.26 is TECHNICALLY VALIDATED / PENDING FOUNDER ACCEPTANCE on integration.
Root personally executed **468 host tests +1 doctest/eight gates PASS**; the
writer executed **22 initialization +8 recipient-preflight tests PASS**. Both
passed first attempt without failures, ignored tests or diagnostics. All 92
source inputs and retained log/tool hashes match the separately reviewed freeze.
See [report](TASK_2_26_RECIPIENT_CHECKED_GENESIS_INITIALIZATION.md).

Six new groups cover 24 complete 35/34-account worlds, original rent/custody and
recipient preservation, actual persisted pending recognition twice, zero-effect
invalid recipients/roles/borrows, both recipients' balance/owner/data tampering
at all 40 CPI boundaries, invocation failures and error precedence, shared reads,
replay and native closure. Explicit clone/discard modeling is separate from raw
host partial effects and establishes no actual Solana rollback. The helper
extraction retains all eight Task 2.25 tests and earlier API profiles.

New transport and current runtime proof remain absent. Nine Node tests/eight
cases are retained for the earlier exact initializer template, not rerun; Task
2.14 SBF evidence remains historical. Full recipient-control, spending-limit/
stale-action/live-artifact and funding prerequisites remain deferred. Save/STOP
after integration publication; Task 2.27 is NOT STARTED.

## Historical checkpoint — Task 2.25

Task 2.25 is TECHNICALLY VALIDATED / PENDING FOUNDER ACCEPTANCE on integration.
Root personally executed **462 host tests +1 doctest/eight gates PASS**, zero
failures, ignored tests or diagnostics, on 92 frozen inputs. Separate source/test
review passed. The writer's initial focused run was 7/8 with one warning; one
independently justified test-only correction produced 8/8 PASS without diagnostics.
Both executions and all log/tool hashes are retained in the [report](TASK_2_25_GENESIS_RECIPIENT_PREFLIGHT.md).

Eight groups exercise 72 positive recipient worlds, independent PDA/rent/model
oracles, exact approved keys and same-multisig derivations, positive funding under
zero/default/changed rent, metadata/role/key/backing/guardian aliases, mutable
borrow failures, fresh approval/context/full-message binding and host/native
guards. Full fixtures remain unchanged; no initialization, payment or ledger
recognition occurs. Current vault identity does not establish exclusive spending
control or absence of delegated limits/stale actions. The new 32/31-account
preflight fixture has no transport/runtime execution proof. Nine Node tests/eight
cases remain retained evidence of the old exact initializer template, not rerun;
Task 2.14 SBF evidence remains historical. Task 2.26 is NOT STARTED; save/STOP.

## Historical checkpoint — Task 2.24

Task 2.24 is **TECHNICALLY VALIDATED / PENDING FOUNDER ACCEPTANCE** on integration.
Root personally executed **454 host tests +1 doctest/eight gates PASS** on the
separately reviewed source; the writer executed **16 focused tests PASS**. Both
passed first execution, with zero failures, ignored tests or diagnostics. All
90 inputs match inspection and both executions; log/tool hashes were verified.

Six new regression groups cover 48 prefund/pause/topology success worlds with
independent rent/balance oracles, actual persisted pending recognition twice,
aggregate/destination overflow before effects, all 40 CPI failure boundaries,
full-batch tampering, fresh approvals/roles/borrows, replay and partial state.
Both raw partial host effects and explicit clone/discard modeling are recorded;
neither proves runtime rollback. Original rent obligations remain external and
initial history remains zero. See [report](TASK_2_24_GENESIS_TOKEN_PREFUND_NORMALIZATION.md).

The six transport-template inputs remain unchanged: nine Node tests/eight cases
are retained, not rerun. Task 2.14 SBF evidence is historical only. Later donations
to already Token-owned accounts, operational funding provenance, recipient
control, native exposure and current runtime proof remain unresolved. Save/STOP
after reviewed integration publication; Task 2.25 is NOT STARTED.

## Historical main integration checkpoint (D-028)

Tasks 2.20–2.23 are **TECHNICALLY VALIDATED / FOUNDER-AUTHORIZED MAIN INTEGRATION**
at implementation `3282e1ebabcb0cd88491d48a391565b8b100afa7` plus reviewed records.
This authorizes publication, not broader founder acceptance or runtime readiness.
Root reverified 94 inputs and 20 retained logs: **448 host tests +1 doctest/eight
gates** from Task 2.22, plus **9 Node tests/eight unsigned transport cases** from
Task 2.23. **No tests were rerun in this publication turn.** Separate source
review passed; see the [checkpoint](PIV1_PILOT_STATE.md) for exact evidence and
publication state. Native initializer exposure and current runtime proof remain
deferred. Task 2.24 is NOT STARTED; STOP after publication. Status sections below
are HISTORICAL records of their original sessions.

## Historical status after Task 2.22

Tasks 2.3–2.19 remain founder-accepted under D-027; Tasks 2.20–2.22 are technically
validated pending founder acceptance. Root personally executed **448 host tests
+1 doctest/eight gates**, zero failures, ignored tests or diagnostics, on the
separately reviewed source. The writer ran ten focused initialization tests.
All 90 inputs and retained log/tool hashes were verified; no corrective retry.

Task 2.22 composes fresh allocation, both legacy Token initializations and nine
initial typed envelopes. Tests independently check instruction/meta/seed and
account-byte layouts, twenty prefunding worlds, all forty CPI failure positions,
false/malformed Token success, the first vault during the second Token call,
nine late shared state borrows with no partial envelope writes, and replay.
Raw partial effects and explicit staged-discard modeling remain distinct from
runtime rollback. The native selector remains closed. Recipient/prefund/funding
constraints, Squads transport and current runtime/resource/rollback proof remain
required; Token-native excess still rejects in the separate economic accessor.
See [Task 2.22](TASK_2_22_GENESIS_ACCOUNT_INITIALIZATION.md). Save/STOP after
reviewed publication; Task 2.23 is NOT STARTED. The Task 2.14 artifact remains the
latest historical SBF evidence, covering only its earlier source.

## Historical status after Task 2.21

Tasks 2.3–2.19 remain founder-accepted under D-027; Tasks 2.20–2.21 are technically
validated pending founder acceptance. Root personally executed **438 host tests
+1 doctest/eight gates**, zero failures, ignored tests or diagnostics, on the
separately reviewed source. The writer independently ran twelve focused tests.
All 88 source inputs and retained log/tool hashes were verified. These are host
executions; Task 2.14 remains the latest historical SBF artifact evidence.

Task 2.21 proves fresh approved rent-only allocation composition in the host
model: external payer inner/outer signing constraints, rent-retaining debit,
fixed PDA seeds and pinned System instruction bytes/metas, all sixteen target
postconditions, prefunding preservation through u64::MAX, and adversarial
failures at all 38 CPI boundaries. Direct partial effects and staged-discard
rollback modeling are explicitly distinguished. The new function is undispatched;
Token accounts and nine states remain uninitialized. Same-transaction completion,
recipient/funding constraints, transport and actual runtime rollback/resource
proof remain required. See [report](TASK_2_21_GENESIS_ACCOUNT_ALLOCATION.md).
This session saves/STOPs after reviewed integration publication; Task 2.22 is
NOT STARTED.

## Historical status after Task 2.20

D-027 records founder acceptance of the bounded Tasks 2.3–2.19 milestone. Task
2.20 is **TECHNICALLY VALIDATED / PENDING FOUNDER ACCEPTANCE**. Root executed
426 host tests +1 doctest/eight gates on the separately reviewed source; the
writer independently ran ten focused preflight tests. Earlier status/count
paragraphs below are chronological evidence, not the current acceptance boundary.
No current-source SBF validation is inferred from these host executions.

## Verified baseline: Task 2.11 after corrected Task 2.3 and Tasks 2.4–2.10

Pilot executions on frozen Task 2.11 source, implementation `2eeefba0abc226bcfcadddb5f248f12ca589e09d`:
335 host tests, one doctest,
default/all-feature plus explicit no-entrypoint/cpi/idl-build checks and
warnings-denied documentation pass. Separate
source review passed within the documented scope. Founder acceptance is pending.
The task reports distinguish original executor, writer and pilot evidence; see
[Task 2.11](TASK_2_11_KIF_CLAIM_INSTRUCTION_BOUNDARY.md),
[Task 2.10](TASK_2_10_ISOLATED_KIF_CLAIM_EXECUTION.md),
[Task 2.9](TASK_2_9_STATE_ENVELOPE_PERSISTENCE.md),
[Task 2.8](TASK_2_8_GUARDIAN_CLOCK_SNAPSHOT_AUTHENTICATION.md),
[Task 2.7](TASK_2_7_ISOLATED_KIF_CLAIMS.md),
[Task 2.6](TASK_2_6_PROTECTED_PRINCIPAL_DEPOSIT.md),
[Task 2.5](TASK_2_5_INITIAL_CONTRIBUTION_BOOTSTRAP.md),
[Task 2.4](TASK_2_4_ACCOUNT_AUTHENTICATION.md),
[corrected Task 2.3](TASK_2_3_VAULT_RECONCILIATION_MODEL.md),
[Task 2.2](TASK_2_2_CONTRIBUTION_PENDING_MODEL.md),
[Task 2.1](TASK_2_1_MOCK_STAKE_POOL_ADAPTER.md), and
[Phase 1 properties](TASK_1_4_PROPERTY_TESTS.md).

| Requirement | Current inspected evidence | Evidence still required before founder Testnet handover |
| --- | --- | --- |
| P-003/P-008/P-020–P-024: locked principal, fixed split, checked floors, external fees | Math unit/property tests; legal/illegal lifecycle tests; Task 1.2–1.4 reports | Actual handler account privileges, fixed native destinations, transfer/CPI deltas and failure rollback |
| P-013–P-019: full pending contribution value, HWM and loss recovery | `contribution_pending`, `vault_reconciliation`, property tests; Task 2.3 T23-R1 severe-loss regressions; Task 2.14 authenticated pending instruction, host phase-offset checks and local SBF donation/recognition | Remaining runtime custody composition, HWM integration and real pool valuation |
| P-009–P-012/P-029–P-032: permissionless cadence, one round, insufficient cooldown | Legal/illegal transitions and randomized property ordering | Trusted Clock decoding, instruction privileges and malformed transaction rollback |
| P-004/P-014/P-015: first-contribution bootstrap and idle principal intake | Task 2.5 initial-only bootstrap tests with a genuine empty host fixture and later contributed-token yield | Actual initialization/transfers and protected SOL deposit; general idle integration remains separately scoped; never invent funded initial principal or contribution yield |
| A-003: distinct fixed economic custody | Task 2.3 per-vault obligations and host normalization; Task 2.4 AccountInfo/PDA/owner/token/rent/deficit tests | Complete authenticated initialization, real transfers and rent-preserving normalization |
| Direct/untracked SOL and token contributions | Task 2.2 idempotence and Task 2.3 all supported economic-vault host paths | Actual transfer races; unsupported native surplus in Token/temporary accounts; operational funding provenance |
| A-001/P-018/P-026: official accounting, protected CPI, dynamic minima | Task 2.1 mock interface and fee/slippage/minimum boundary tests | Pinned real SPL/Jito pool/list/mint/source validation, account-derived snapshot identity and exact protected instruction behavior |
| A-002/A-005: permissionless multi-leg source order, maximum fills, exact target | Mock source-capacity tests; lifecycle/property tests; Task 2.3 composition | Real preferred/source-order checks, deterministic stake/metadata authentication, adversarial runtime and multi-leg Testnet evidence |
| P-035: delayed readiness, rent recovery, cooldown reward/loss | Mock finalization and Task 2.3 exact custody/recovery tests | Stake/Clock/Stake History decoding, actual closure, both rent destinations and exact post-CPI deltas |
| P-024/A-004: atomic settlement, later pending integration and compounding | Pure transitions and staged host rollback; Task 2.6 zero-fee deposit, exact deltas/mint audit, carry/HWM/failure regressions | Runtime transaction rollback and real protected CPI; general fee/rounding-loss support remains OPEN |
| K-005–K-010: active snapshots, half-open 30-day periods, repeated carry | Timing/guardian unit tests, math/property/lifecycle and host custody tests; Task 2.8 nine-AccountInfo current registry/rewards/Clock authentication and 22 activity/identity/borrowing regressions | Actual runtime inputs, authorized signed heartbeat and verified qualifying governance activity; global historical-ledger and earning-provenance invariants |
| K-012: earned isolated claims remain available during pause | Tasks 2.7–2.11 authentication, persistence, host composition and strict ABI; Task 2.12 actual SBF artifact; Task 2.13 exact local SBF/Rent/System CPI, complete account effects, paused historical claims, replay/errors, events and shared-context observations (19 tests / 60 cases), retained on the Task 2.14 artifact | Signatures, Bank/AccountsDB commit and rollback, authorized initialization/earning and historical-ledger sum, actual public-cluster pause behavior; reduced-compute post-CEI failure remains unproven |
| G-003–G-005: explicit emergency pause and economic gates | Pure illegal-transition pause matrix; Task 2.3 pending recognition/recovery preservation; Task 2.14 actual paused pending recognition | Governed pause/unpause plus handler/runtime rejection for snapshots, deposits/conversions, withdrawals, finalization and migrations; preserve K-012 claims exception |
| K-001–K-004/G-001/G-002/G-007/G-008: six guardians, 4-of-6 governance and upgrade custody | Bounded registry validation and immutable snapshot tests; Task 2.15 inspected source/host current Squads configuration and ProgramData linkage, same-program guardian set correspondence (363 host tests +1 doctest, eight gates and final separate review passed); Task 2.16 exact direct single-inner current-approval binding has source/host evidence (383 +1, eight gates, separate source/test review); Task 2.17 separately authenticates virgin Config and current Squads approvals (392 +1, eight gates, separate source/test review) | Actual Squads/PIV1 CPI, signatures, persisted replay/rollback and resource evidence for action binding; initialization transport; pause/recipient/rotation implementation and non-bypass tests; exact live authority identities and upgrade-authority verification; actual authority-transfer rehearsal requires its separate explicit authorization |
| Layout and failure atomicity | Five bounded schemas, Option length/property tests, full-state rollback assertions; Task 2.9 four typed fixed envelopes and 21 existing-AccountInfo atomic byte/alias/borrow/stale-state regressions | Authorized handlers coupling state and actual custody, runtime rollback/privileges, initialization replay/alias protection, WithdrawalLeg persistence and SBF limits |
| Reproducible and reviewed delivery | Pinned Rust/Anchor stack, locked/offline host gates, actual separate AI source reviews | Real-adapter dependency review, SBF artifact/build reproducibility, compute/size/rent measurements and final adversarial review |
| Founder testing path | Planned CLI/runbook in canonical spec; not implemented | Usable non-Rust test workflow, verified addresses/artifact, actual approved lifecycle signatures/states, cooldown/retry/recovery instructions |

## Evidence levels and deployment gate

- **Host math/state/mock:** controlled Rust data and atomic staged models only.
- **AccountInfo fixtures:** actual byte/owner/key validation on host fixtures;
  not proof of runtime locks, invocation privilege rules or CPI rollback.
- **Local runtime:** record exact harness/program artifacts, supported instruction
  paths and adversarial cases. Do not label a hand-mutated fixture as a validator.
- **Public Testnet:** record independently verified genesis/cluster, approved
  program/authority/recipient identities, artifact, transaction signatures and
  account states. Historical Phase 0 one-leg spike evidence is not the new PIV1
  deployment, multi-leg orchestration, or founder-testable milestone.

No public-Testnet deployment/fund-moving lifecycle or new key/signing workflow
is authorized yet. D-026 requires its concrete live-operation approval package.
Mainnet and authority transfers remain outside the technical development mandate.
Critical predeployment checks precede deployment regardless of phase labels.
No known critical/high defect may be hidden by an evidence-level distinction.

## Chronological dependency tracking

Tasks 2.4–2.12 are technically validated within their bounded scopes, pending
founder acceptance. Task 2.7 implementation `10dceb5` and closure `37f25a8`
are published with accepted main unchanged. Task 2.8 current-registry
activity/Clock authentication passed final separate review and pilot gates; its
reviewed closure `440e83e26d36df911ccfafac97d89b79b8b4c694` is published.
Task 2.9 typed state envelopes and atomic existing-account byte persistence
passed final separate review and pilot gates; reviewed closure
`44d70ec911ad3a78738fb90a04ac82eec3ca44f2` is published. Task 2.10 is scoped
for isolated claim execution and explicit host invocation/rollback modeling.
It passed final separate review and pilot gates; reviewed closure
`9f9a8dbba132f96e4c76a8385844570746b0636f` is published and independently verified. Task 2.11 strict claim ABI/runtime-ID boundary passed final pilot gates and
separate exact review. Reviewed closure
`14106d664c107b3a2f705ac87361768af42d0786` is published and independently verified. The next bounded
dependency is actual keyless SBF compilation/artifact inspection before a genuine
runtime harness. Task 2.12 retains two failed builds and their complete diagnostics;
helper splitting was insufficient. Separately reviewed private-state boxing then
passed the third actual locked/offline target build with zero warning/error/frame
diagnostics and unchanged source/tools/pins/protected refs. The pilot independently
inspected artifact headers, exact instruction bytes and bounded direct-call/stack
observations. Those are static evidence, not complete runtime resource proofs.
The pilot personally ran 339 tests, one doctest and all eight final host gates:
PASS, source unchanged. Sixteen previously executed Python runner tests remain
valid for the unchanged tool. Final separate source/evidence review returned PASS
without findings. Task 2.12 is technically validated, pending founder acceptance.
Both prior failures and the temporary host inventory-check correction remain
recorded in [Task 2.12 evidence and scope](TASK_2_12_KEYLESS_SBF_COMPILATION.md).
A later pinned keyless runtime harness must establish loader, actual execution/CPI
and rollback evidence; no runtime or deployment success is implied.

General fee/rounding-loss support for principal deposits remains OPEN: a protected
slippage floor alone does not preserve full historical book value. General idle
integration remains deferred; preserve pending-SOL priority and no-yield/
insufficiency behavior. Before economic handlers rely on an observation, resolve
or explicitly contain Token-native-surplus liveness and operational/rent funding
provenance. State serialization/initialization, handler privileges and exact real
adapter mapping remain separate bounded work items. Numbering does not itself
authorize or define an implementation.

## Prior local runtime milestone: Task 2.13

The [Task 2.13 report](TASK_2_13_KEYLESS_SBF_CLAIM_EXECUTION.md) is technically
validated, pending founder acceptance, at implementation
`fd5735976eef1e2728ccf54726145501573db60d`. Root executed the unchanged Task 2.12 ELF
through the pinned local SVM: **19 tests passed across 60 message cases**. Separate
final review passed without findings. This is additional local runtime evidence;
the unchanged production baseline remains Task 2.12's 339 host tests +1 doctest.

Root and reviewer independently inspected all 1366 complete account records,
four ledger/two native changes, exact System CPI/events, paused historical claims,
and sequential audit continuity. Ordinary claims used 73834/200000 CU. Shared
success/stale replay shows real first effects followed by Mollusk output discard.
All four reduced-compute cases retained unchanged raw state; post-CEI compute
failure remains unproven. Synthetic signers are not signatures; no Bank rollback,
deployment-verifier or public-network evidence is added.

The remaining initialization/earning, complete lifecycle/governance handlers,
real SPL/Jito integration and founder testing workflow remain required.
## Historical local runtime milestone: Task 2.14

[Task 2.14 pending recognition](TASK_2_14_RUNTIME_PENDING_RECONCILIATION.md) is
**TECHNICALLY VALIDATED / PENDING FOUNDER ACCEPTANCE** after final separate review.
Implementation: `7442cab7e97c422c7ee06290d5fc9d11c8b13ee6`.
Root executed 349 host tests +1 doctest/eight gates and 24 local SBF tests across
70 cases, with no failures/ignored tests. All 19 earlier claim tests remain.
New target ELF SHA: `46fd815847c236fb53ed5dc5ace79c48c5a21beac4019ffa107b80a5be69812f`.

Root and reviewer independently checked 1629 complete account records. A real
System donation of 100 lamports plus recognition and repeated no-op succeeds at
151832/200000 CU. Paused pending recognition retains native token-account excess;
either-asset/rent deficits and ABI/privilege/alias failures preserve all accounts.
Only two pending Config fields change; other accounting and custody stay intact.
Shared success/failure provides raw effects and Mollusk output discard evidence.

Active/settled/recovery offsets have host/pure evidence, while these runtime
fixtures cover Idle/pause. Token units and initialized owned state are synthetic.
This adds no actual SPL transfer, initialization/earning authority, full KIF
historical-liability proof, Bank rollback, signatures or public-cluster evidence.
The existing funding/fee/normalization/governance/lifecycle boundaries remain.
Later Tasks 2.15 and 2.16 add source/host governance prerequisites only. Task 2.16
root executed 383 host tests +1 doctest/eight gates with no diagnostics and
separate source/test review passed; see its
[report](TASK_2_16_SQUADS_INVOCATION_AUTHORIZATION.md). These tests model context,
proposal status and privileges; they do not execute Squads or prove initialization,
rotation, signatures, durable effect-once/rollback or target resource limits.
Exact Git identities, closure and next action are in the checkpoint.

Task 2.17 adds a separate source/host bootstrap authorization boundary, preserving
Task 2.16's initialized-guardian requirements. Root's 392 host tests +1 doctest and
eight gates passed on the separately reviewed freeze. Prefunded empty Config is
accepted without lamport classification; sorted Squads approvals do not establish
PIV1 slots, activity, parameter semantics, initialized state or a replay receipt.
[Evidence and limits](TASK_2_17_SQUADS_BOOTSTRAP_AUTHORIZATION.md).


Task 2.18 adds a source/host approved genesis model, with exact same-message
bootstrap authorization and deterministic initial state/target descriptors.
Root executed 404 host tests +1 doctest/eight gates on the separately reviewed
freeze. Twelve new tests cover strict format, parameter substitution, fresh
context, explicit slots/pause, two runtime IDs, full topology, zero histories,
checked periods, alias boundaries, prefunding preservation and compatibility with
existing fixed-account and guardian authentication. Official protocol/recipient
identity, actual non-Config targets, account creation, funding provenance, initial
vote activity, payout readiness and new SBF evidence remain unproven.
[Evidence and limits](TASK_2_18_APPROVED_GENESIS_MODEL.md).


Task 2.19 adds source/host Jito account identity evidence. Root's 416 host tests
+1 doctest/eight gates passed on the separately reviewed freeze. Twelve new tests
include all 432 option/future-fee combinations against exact-source extracted
upstream types and actual Stake serialization, plus identity substitutions,
owner/borrow failures, list geometry, reserve/Token restrictions, valid aliases,
raw fees and complete input preservation. Five test-only dependency edges retain
all 168 locked package identities. Official source constants and account bindings
do not establish cluster genesis, deployed artifact equivalence, validator/current
pool readiness, economic quotes, genesis composition or real account creation.
[Evidence and limits](TASK_2_19_JITO_ACCOUNT_IDENTITY.md).

## Task 2.20 — Fresh approved genesis account preflight

The same approved bytes, complete account slice and one Clock/Rent context now
compose genesis authorization/model, source-pinned Jito identity and observations
of all sixteen intended targets. Checks cover canonical addresses, current empty
System ownership, writable/nonexecutable privileges, role/backing separation,
fallible borrows and checked rent-only shortfalls. Root verified all 86 source
hashes, sixteen gate logs, two writer logs and existing tool hashes.

Actual funding source/provenance, recipient control, historical replay protection,
account creation, token initialization, persistence and initializer transport
remain required. Prefunding is preserved and unclassified; rent shortfall totals
do not establish liquidity or authorize spending. No instruction selector or
CPI is added; Task 2.14 remains the latest historical SBF artifact evidence.
[Task 2.20 report](TASK_2_20_GENESIS_ACCOUNT_PREFLIGHT.md) records exact commands,
regressions and limitations. The founder requested STOP after this saved task;
Task 2.21 was not started at that checkpoint; its later evidence is recorded above.
