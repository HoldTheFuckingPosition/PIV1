# PIV1 Repository Instructions

## Current execution state

Task 2.48 / D-030 M2 with its required M3 finalization component is **TECHNICALLY VALIDATED / PENDING FOUNDER ACCEPTANCE**.
Production now finalizes a withdrawn Stake leg into fixed escrow, recovers exact
original Stake/metadata rent and atomically closes metadata with cumulative state.
Verified clean start as jerem: integration
`3b0a4f719e64bda45a199f31d53a3bf58be54f62`; main remains `1054ff3`. One delegated
writer and separate review were used. Root passed ten focused tests, 571 host
tests +1 doctest/eight gates and strict SBF. All 122 final source inputs and 266
historical records match. ELF SHA `c1b7ae07…` is static evidence;
actual new-path Stake/VM/Bank execution and heap/CU remain M4 obligations.

The first focused run passed nine tests and failed one incorrect overflow oracle.
A source balance of u64::MAX can safely split rent and pending credit. The reviewed
test-only correction retains that success and proves a separate actual destination
overflow using u128; production stayed unchanged. Both attempts, all initial source
bytes and 137 first-attempt records remain preserved. Final host/SBF gates passed
without diagnostics. No dependency/layout/economic change or cleanup/install.

Pinned Stake full-withdrawal success is authoritative inactivity proof; no legacy
activation formula substitutes for its runtime Clock/History. Original rent must
be covered and native balance minus original rent must not exceed current delegated
stake. Ambiguous Stake donation/rent-adjustment excess rejects; it is never called
yield or pending. Metadata excess normalizes to pending. Rewards/losses, original
rent, out-of-order legs, HWM recovery, carry/KIF and native Token quarantine retain
their accepted accounting. Actual runtime rollback/account purge remain unproved.

Next implement production settlement and pending integration, then the complete
local production lifecycle and exact Testnet package. Root owns reviewed integration-
only publication; Git and `/tmp/piv1-t248-pilot-review/publication.json` record its
identity. Main, founder acceptance and live gates remain separate. Preserve four
recovery archives/restorations, both Task 2.47 attempts and this new ELF. Save/end
this bounded session for economical usage; resume from actual refs/worktree.

The Task 2.47 record below is HISTORICAL; integration publication is complete.

Task 2.47 / D-030 M2 with its required M3 withdrawal component is **TECHNICALLY VALIDATED / PENDING FOUNDER ACCEPTANCE**.
Production now initiates a protected active-source withdrawal leg and immediately
deactivates its Stake account. Verified start integration/main was
`1054ff3751ca4169eab7e46d77452634b11fc695`; main remains there. One delegated
writer and separate review were used. Root passed 10 focused tests, 561 host tests
+1 doctest/eight gates on corrected source, plus strict SBF. All 119 final frozen
inputs and 42 historical records match. New ELF SHA `224b4c40…` is
static evidence, not new-path VM/Bank or actual nested SPL/Stake execution.

First SBF was rejected for a 4160-byte frame against the 4096-byte maximum, despite
Cargo exit zero. A reviewed bounded boxed-leg construction removed that diagnostic;
corrected host/SBF gates passed. Preserve both attempts, their source freezes and
the rejected ELF; 105 first-attempt records are retained. No host test failed.
The user's preexisting Solana-skills section below is preserved verbatim. No
migration, dependency/account-layout/economic change or cleanup/install occurred.

Current source/minimum/fees/rent, maximum fill, no stranded remainder, post-removal
round-floor/HWM and exact CPI receipts are rechecked. Unused System-owned empty
temporary PDA prefunds normalize to pending before full operational rent advances.
Config/round/new leg commit only after protected SPL withdrawal, authority checks
and Stake deactivation. Exact preferred active source remains supported; transient,
reserve, removal and preferred fallback remain unsupported. Native Token funding,
pending offsets, carry and KIF remain protected. Actual heap/CU and rollback remain
runtime proof obligations. See the active checkpoint and Task 2.47 report.

Next implement finalization with real Stake withdrawal/rent recovery, then
settlement and pending integration before the full local production lifecycle and
Testnet package. Root owns reviewed integration-only publication; Git and receipt
record its identity. Main, founder acceptance and live gates remain separate.
Preserve four recovery archives/restorations and add this ELF to the next baseline.
Save/end this bounded session for economical usage; resume from actual refs/worktree.

The D-033 record below is HISTORICAL; publication completed at 1054ff3.

D-033 authorizes main publication of the validated Tasks 2.40–2.46 sequence on
2026-10-03 UTC. Exact implementation tip:
`7624f93bf55721cb74687677a26c51fcf25cd669`; starting main:
`8912cfeffcd83fa12cc1a68937a9de8dd5c6b091`. The seven commits plus necessary
reviewed authorization/checkpoint documentation are **TECHNICALLY VALIDATED /
FOUNDER-AUTHORIZED MAIN INTEGRATION** within their documented scopes. This
supersedes only their historical integration-only publication restriction.

Root and a separate reviewer checked the cumulative sequence and retained
source/test/artifact evidence. The latest source still matches 115 frozen inputs
and 551 host tests +1 doctest/eight gates plus strict SBF evidence; no test/build
rerun or new implementation occurs in this publication scope. Preserve all prior
artifacts/logs, four recovery archives/restorations and Task 2.46's new ELF.
Root owns normal fast-forward publication and independent final-ref verification;
Git and the checkpoint publication receipt record the resulting identity.

M2/full M3–M6 remain open. Next implement actual protected withdrawal initiation
with pinned SPL withdrawal and Stake deactivation, then finalization/settlement/
pending integration and the full local lifecycle. Future work returns to reviewed
integration checkpoints under D-026/D-030. D-033 is not broader founder acceptance
or live Testnet/Mainnet, signing, funds or authority-transfer authorization.
Read the active checkpoint and verify actual refs/worktree before resuming.

The Task 2.46 record below is HISTORICAL; D-033 supersedes only its earlier
integration-only publication restriction, preserving its evidence and limitations.

Task 2.46 / D-030 M2 with its required M3 proof component is **TECHNICALLY VALIDATED / PENDING FOUNDER ACCEPTANCE**.
Production withdrawal preparation now uses a current standard active-validator
witness, canonical Stake minimum query and conservative target/multileg/HWM proofs.
Start integration was `17bc0760390034157a932e51de674f976d114922`; main remains
`8912cfe`. One writer and separate review were used. Root passed 10 focused tests, 551 host tests +1 doctest/eight gates and strict SBF.
All 115 frozen inputs match; ELF SHA `e10e3c89…` is static evidence,
not new-path VM/Bank or real Stake/pool execution. A reviewed default-lockup
negative-Clock guard has a passing regression; the first focused pass is retained
separately. No test/build failed. See the report for run history.

Valid insufficiency, including a zero computed target, changes only the 24-hour
clock and emits its factual event after commit. Preserve the old liquid ABI and
legacy helper error order. Exact preferred active source is supported; transient,
reserve, removal and preferred fallback remain unsupported. Rent/source feasibility
is a point-in-time proof; preparation does not create/debit a leg or withdraw.
The exact existing Stake-interface dependency moved to production; lock/packages,
account layout and economics remain unchanged. Preserve all four recovery archives,
restorations and historical evidence; add this ELF to the next baseline.

Next connect protected leg initiation with real pinned SPL withdrawal and Stake
deactivation, then finalization, settlement and pending integration. Reauthenticate
source/minimum/current rent at execution; do not rely on an old preparation witness.
Complete local lifecycle and Testnet packaging remain open. Root owns reviewed
integration-only publication; Git/checkpoint receipt record its identity. Main,
founder acceptance and live gates remain separate. Save/end this bounded session
for economical usage; resume from actual refs/worktree and the checkpoint.

The Task 2.45 record below is HISTORICAL; integration publication is complete.

Task 2.45 / D-030 M2 is **TECHNICALLY VALIDATED / PENDING FOUNDER ACCEPTANCE**:
production no-yield/liquid-funded distribution preparation now binds current
pool valuation and Clock/guardian snapshots, funds pending SOL before prior carry,
and atomically commits Config/round after exact System CPI checks. Start integration
was `a18f33a2b5e9cd38ea1f6413d3e4e2e7d74a810c`; main remains `8912cfe`.
One writer and separate review were used. Root passed ten focused tests, 538 host
tests +1 doctest/eight gates and strict SBF on first execution without diagnostics.
All 112 frozen inputs match; ELF SHA `ede63be2…` is static evidence,
not new-path VM/Bank execution. No source defect or execution-driven correction.

Positive withdrawal shortfall rejects with error 6145 before any CPI, state or
cooldown mutation. No-yield is byte-identical; liquid success is independent of
prior insufficiency cooldown. HWM stays proposed until settlement, full pending
contribution value stays recorded, and carry/KIF/rent/native quarantine remain
protected. No dependency, account-layout or economic change; no cleanup/install.
Preserve all four recovery archives/restorations and historical evidence; add
this ELF to the next preservation baseline. Current caches remain incomplete.

Next close real withdrawal preparation: dynamic Stake/protocol minimum, source
residual constraints and conservative multileg target/fee/floor/HWM proofs. Then
connect delayed legs, settlement and pending integration to the local lifecycle.
Do not fabricate adapter revision/capacity/minimum or valid-insufficient results.
M2 and the full adapter remain incomplete. Root owns reviewed integration-only
publication; Git/checkpoint receipt record its identity. Main, founder acceptance
and live gates remain separate. Save/end this bounded session for economical
usage; resume from actual refs/worktree and the checkpoint.

The Task 2.44 record below is HISTORICAL; integration publication is complete.

Task 2.44 / D-030 M2 is **TECHNICALLY VALIDATED / PENDING FOUNDER ACCEPTANCE**:
production protected principal SOL staking now uses the pinned SPL deposit
instruction and exact pool/Mint/custody postconditions. Start integration was
`c47272acbb2f9628cc308935b1c6b5f1fa575406`; main remains `8912cfe`.
One delegated writer and separate review were used. Root passed ten focused tests,
528 host tests +1 doctest/eight gates and strict SBF on first execution without
diagnostics. All 109 frozen inputs match; ELF SHA `a61d5ed2…` is static evidence,
not new-path VM/Bank or nested stake-pool execution. The pre-execution authority
borrow correction has a passing regression; legacy API validation order remains.

Only zero-fee conversions preserving historical book value and HWM are supported.
Stored/Mint supply burn lag, pending assets, carry, liabilities, rent and native
Token quarantine stay protected. No dependency, layout or economic change; no
cleanup/install occurred. Preserve all four recovery archives/restorations and
historical binaries/logs. Add this new ELF to the next preservation baseline.

Next connect production distribution preparation to current valuation, Clock/
guardian snapshots, pending-first funding, cadence/insufficiency and conservative
withdrawal-target/minimum proofs; close demonstrated adapter dependencies without
fabricating model fields. The full adapter, remaining handlers and local lifecycle
remain open. Root owns reviewed integration-only publication; Git and the checkpoint
receipt record its identity. Main, acceptance and live gates remain separate.
Save/end this bounded session for economical usage; resume from actual refs/worktree.

The Task 2.43 record below is HISTORICAL; integration publication is complete.

Task 2.43 / D-030 M2 is **TECHNICALLY VALIDATED / PENDING FOUNDER ACCEPTANCE**:
initial pending SOL/JitoSOL now moves into principal through strict production
19/18-account bootstrap, authenticated current pool valuation and final-only
Config/HWM commit. Start integration was `9856ec8de01ca5706296bf59e63f9396bce4ecbf`;
main remains `8912cfe`. One writer and separate source/test/command/evidence review
were used. Root passed ten focused tests, 518 host tests +1 doctest/eight gates and
strict SBF on first execution without diagnostics. All 105 frozen inputs match;
ELF SHA `88bb9166…` is static evidence, not new-path VM/Bank execution.

Valuation uses the official stored pool ratio, exact Clock epoch and held units
≤ Mint supply ≤ recorded supply, tolerating legitimate direct-burn lag. Initial-only
history/replay rules and the legacy PoolSnapshot API remain. Bootstrap-only native
Token quarantine preserves all lamports without classifying them; the old strict
accessor remains unchanged. No dependency, layout or economic change. Available
disk capacity was about 32.2 GB on takeover; no cleanup/install occurred. Preserve
all four recovery archives/restorations and historical binaries/logs.

Next bounded production gap: protected principal SOL staking under Task 2.6,
including its required real pinned SPL deposit component. Record that M3 dependency
within the M2 handler scope; do not fabricate model revision/capacity or weaken
zero-fee, book-value/HWM, rounding/slippage or pause guards. Later integration and
distribution remain open; M2 is incomplete. Root owns reviewed integration-only
publication; Git and the checkpoint receipt record its identity. Main, founder
acceptance and live gates remain separate. Save/end this bounded session for
economical usage; resume from actual refs/worktree and the checkpoint.

The Task 2.42 record below is HISTORICAL; integration publication is complete.

Task 2.42 / D-030 M2 is **TECHNICALLY VALIDATED / PENDING FOUNDER ACCEPTANCE**:
production economic-surplus normalization now moves only proven excess to pending
custody, after each vault covers its own obligations. Start integration was
`3a5c3ce9179975c2cfd7ba09d379459aaa53a8d0`; main remains `8912cfe`. One writer
and separate source/test/command/evidence review were used. Root passed ten focused
tests, 508 host tests +1 doctest/eight gates and strict SBF on first execution,
without diagnostics. The 101-input freeze matches; ELF SHA `263493df…` is static
evidence, with no new-path VM/Bank execution. Dependencies/layout/economics remain.

Measured disk capacity required reviewed reversible archival of 706 historical
SBF intermediates into 424 objects; one file was restored and retained, with 705
paths restorable. Preserve `task-2.42-sbf-intermediates-20260929-a` under
`/home/jerem/piv1-evidence` and all three older archives/restorations. The first
read-only plan failed on a debug-lock assumption; the corrected wrapper uses six
existing release locks. Follow the Task 2.42 report for exact restoration; default
helper debug locks do not apply. Binaries/logs and 2,955 nonselected paths remain
intact. SBF retained its 2-GiB reserve. No installation or live operation occurred.

Next connect pending custody to principal bootstrap/integration with authenticated
pool valuation; explicitly resolve the remaining Token-native quarantine/bootstrap
compatibility without inventing pool revision/capacity facts. M2 remains in progress.
Token-native excess/operational funding stay untouched and unclassified; the old
strict accessor remains. Root owns reviewed integration-only publication; Git and
the checkpoint receipt record the resulting commit. Main/acceptance/live gates
remain separate. Save this checkpoint and end the bounded session for economical
usage; resume from actual refs/worktree, not the historical next-task statements.

The Task 2.41 record below is HISTORICAL; integration publication is complete.

Task 2.41 / D-030 M2 is **TECHNICALLY VALIDATED / PENDING FOUNDER ACCEPTANCE**:
production SOL/JitoSOL intake now uses canonical System/legacy Token transfers,
D-032 pause rejection, both prior custody obligations and exact postconditions.
Start integration was `1b1677c7df57925012e4f36ee597423aa641344d`; main remains
`8912cfe`. One delegated writer and separate review were used. Root passed nine
focused tests, 498 host tests +1 doctest/eight gates and strict SBF compilation.
An initial missing hash import failed before tests; a reviewed exact direct edge
to an already-locked SHA-256 package fixed it without new packages/versions.
New ELF SHA `6ebf6d2d…` is static evidence, not new-path VM/Bank execution.

Final compilation required reversible cache archival within this task. Preserve
both `task-2.41*intermediates-20260928-a` archives under `/home/jerem/piv1-evidence`,
plus the untouched Task 2.35 archive. 3,008 main-target and 1,095 Bank-target paths
remain restorable, with one actual restoration retained per archive. Recorded
binaries/logs and nonselected paths are intact; these targets are incomplete
incremental caches and require restoration/recompilation planning. See the report
for exact restoration profiles. SBF retained its 2-GiB reserve; no installation.

M2 remains in progress. Next connect pending custody to principal bootstrap/
integration with authenticated current pool accounting; identify any concrete M3
bridge required. Remaining lifecycle, full adapter, local end-to-end and Testnet
package remain open. Root owns reviewed integration-only publication; Git and the
checkpoint receipt record its identity. No new main authority, founder acceptance
or live permission is inferred. Verify actual refs/worktree on takeover.

The Task 2.40 record below is HISTORICAL; integration publication is complete.

Task 2.40 / D-030 M2 is **TECHNICALLY VALIDATED / PENDING FOUNDER ACCEPTANCE**:
production guardian heartbeat and explicit Squads-authorized pause/unpause are
exposed, preserving liabilities, economic fields and frozen snapshots. Verified
clean start as jerem is `8912cfeffcd83fa12cc1a68937a9de8dd5c6b091` on
`integration/piv1-testnet`; main remains there after D-031. One delegated writer
and separate source/test/command/evidence review were used. Root passed 10 focused
tests, 489 host tests +1 doctest/eight gates and strict production SBF compilation,
first execution with no diagnostics. New ELF SHA `78f94aec…` is static evidence;
no new-path VM/Bank runtime occurred. All 20 historical records are intact.
The founder separately resolved explicit-deposit pause rejection in **D-032**;
direct transfers already received remain reconcilable. Next bounded M2 block:
explicit SOL/JitoSOL intake with real transfer and pending-custody checks. M2 is
in progress; M3–M6 and live gates remain open. Root owns reviewed integration-only
publication; Git records its exact identity. No new main authority or broader
founder acceptance is inferred. Read checkpoint/report and verify actual refs,
HEAD/worktree on takeover. Preserve historical evidence and Task 2.35's archive.

The D-031 record below is HISTORICAL; publication completed at `8912cfe`.

The founder explicitly authorized publication of all already-validated work
missing from `main` on 2026-09-28 UTC, recorded as **D-031**. Tasks 2.33–2.39
are **TECHNICALLY VALIDATED / FOUNDER-AUTHORIZED MAIN INTEGRATION** within their
documented scopes. The exact integration tip is
`bf32d87e06a2d54c8e1c0192faaa7855d246dbc0`; starting main is
`4cc4ea11e87c2f1a2f85b9ed48358f2a881821b3`. Authorization covers the eight-commit
sequence, including D-030 guidance, plus necessary reviewed publication records.
Root owns retained-evidence/source review, safe normal fast-forward publication
and independent final-ref verification. Git and the publication receipt record
the resulting identity; no future hash or completed publication is assumed.

This is a documentation/publication scope, with no test/build rerun or new code.
Task 2.39/M1 remains complete within its documented boundary/runtime scope;
**M2 is next and NOT STARTED**. D-030's ordered milestones remain active after
publication. Main integration is not broader founder acceptance, complete Testnet
readiness or authority for deployment, signing, secrets, funds or authority
transfer. Preserve all historical evidence/limitations and Task 2.35's archive.
Read the active checkpoint and verify actual user/refs/HEAD/worktree on takeover.

The Task 2.39 record below is HISTORICAL; D-031 supersedes only its previous
integration-only publication restriction, preserving its evidence and scope.

Task 2.39 / D-030 M1 is **TECHNICALLY VALIDATED / PENDING FOUNDER ACCEPTANCE**
from integration `9b386cd9c45de99e6f84185f65718df74221f27b`; main remains
`4cc4ea11e87c2f1a2f85b9ed48358f2a881821b3`. One delegated writer and separate
review were used. Root passed 32 focused tests, 479 host tests +1 doctest/eight
gates, nine runner regressions and one real production Bank test/four profiles/
twelve messages; root and reviewer independently checked 786 full account records.
Production now exposes strict recipient-checked normalized initialization alongside
claim/pending dispatch. Full original native approval bytes and arbitrary recipient
witnesses are authenticated. Legacy codec/state/economics remain unchanged.
A first SBF preflight rejected an outdated installed libexpat hash; reviewed OS
provenance justified only a new-profile refresh. First subsequent compilation,
Bank build/runtime passed. Historical artifacts/logs remain unchanged; the bounded
Bank cache is reused, not fresh dependency proof. Preserve all evidence and the
Task 2.35 recovery archive. Read the active checkpoint/report and verify actual
user/refs/HEAD/worktree on resumption; Git records integration publication identity.

**M2 is next and NOT STARTED.** D-030 prioritizes: (1) real production initializer
(M1 complete within its documented scope), (2) economic runtime handlers,
(3) real pinned SPL/Jito adapter/protected CPI, (4) complete local production
lifecycle, (5) exact Testnet package/founder workflow, (6) stop before live deployment
and obtain explicit founder authorization. Reorder only for a demonstrated blocker;
validation-only work must close a concrete critical-path gap. Use bounded work,
separate review, proportionate targeted tests/final gates and reviewed integration
checkpoints. Fixed per-session task stops do not override D-030; conserve credits
and checkpoint interruptions. Do not reopen economics or ask the founder to choose
implementation details. Main/acceptance/live gates remain separate.

Local unsigned oversized packets, synthetic governance/Token, actual recipient
control, internal failed-CPI rollback and live readiness remain unproved. See the
execution/test milestone matrices. No main integration, founder acceptance,
secrets/signing/deployment/fund movement or authority transfer is inferred.

The Task 2.38 record below is HISTORICAL; its publication is complete and its
old scheduling recommendation is superseded by D-030.

Task 2.38 is TECHNICALLY VALIDATED / PENDING FOUNDER ACCEPTANCE for exact
initializer probes through Bank. Start integration
`692d1384afb530bd3740fa8aaf5c7f8f8f448755`; main remains
`4cc4ea11e87c2f1a2f85b9ed48358f2a881821b3`. One writer and separate review were
used; root passed 15 final runner tests and one Bank test/four profiles/twelve
messages, independently checking 786 full account records; separate evidence review passed. Real V3 loader
accounts/slot-one Bank prove late-failure non-fee rollback, retained 15000 fees,
no-fee identical replay and distinct successful retry with original rent and
144-lamport normalization. First strict build and runtime passed. A temporary-path
runner fixture error and preventive scan-lock correction are recorded. The graph
adds only six existing root imports; all dependency nodes/features are unchanged.
223 inputs, 158 tools/50 aliases, 589 original source/archive trees, ten artifacts
and old Bank evidence match. Old suites were not rerun. Production/economics and
old harnesses remain unchanged. Preserve all outputs and Task 2.35's archive;
about 3.73 GiB remains. Root owns reviewed integration-only publication under D-026.
Read checkpoint/report; save/STOP before Task 2.39. Signature/packet/actual Squads,
failed in-initializer CPI, durability and live readiness remain unproved. No new
main acceptance or sensitive-operation authority is inferred.

The Task 2.37 record below is HISTORICAL; integration publication is complete.

Task 2.37 is TECHNICALLY VALIDATED / PENDING FOUNDER ACCEPTANCE for native
System Bank/AccountsDB commit/account-reread validation. Starting integration
`850bc6d1bd2fb4137ef6cb16cc26e78634f6224a`; main remains
`4cc4ea11e87c2f1a2f85b9ed48358f2a881821b3`. One delegated writer and separate
review were used. Root passed 12 final runner tests and one Bank test/four
messages; root and reviewer independently checked 100 complete account records.
Success, exact late-failure non-fee rollback, retained fees, replay rejection and
same-Bank distinct retry pass. This executes native System, not PIV1/SBF genesis,
cryptographic verification, disk restart or chain behavior. Three rejected full
builds and an optional failed Cargo check remain retained. Reviewed local IPC,
existing std-feature and one-line vendored proc-macro visibility corrections
preserve strict gates; the fourth build passed with zero diagnostics. There are
588 registry packages plus one local override; 217 inputs, 158 tools/50 aliases,
589 original archive/source trees and ten historical artifacts matched. No old
tests reran; production/economics are unchanged. Keep all evidence and Task 2.35's
recovery archive. About 4.93 GiB remains; short runtime sampling is not a peak
memory measurement. Root owns reviewed integration-only publication under D-026.
Read checkpoint/report and verify actual refs/worktree. Save/STOP before Task 2.38;
no new main acceptance or sensitive-operation authority is inferred.

The Task 2.36 record below is HISTORICAL; integration publication is complete.

Task 2.36 is TECHNICALLY VALIDATED / PENDING FOUNDER ACCEPTANCE for isolated Bank
source/dependency preparation. Starting integration `90132103af8a99164a3198214ce72c053673d645`;
main remains `4cc4ea11e87c2f1a2f85b9ed48358f2a881821b3`. One delegated writer and
separate review were used. Root passed public resolution and three locked/offline
metadata observations; root and reviewer verified the 589-package registry closure
and all extracted source bytes (25,891 files). The host graph has 552 packages and
56 build-script packages. No compile/test/build-script/Bank execution occurred.
The source-reviewed unsigned Bank commit/account-reread path is documented; fee
retention and replay require separate future oracles. Native prerequisites and
measured build resources remain unproved. A corrected optional-marker audit
assumption is retained; all metadata commands passed. 162 protected inputs and ten
artifacts are unchanged; old evidence was not rerun. Fresh cache/receipts:
`/tmp/piv1-t236-preparation-20260928-a`; source/lock/provenance pins are in
`validation/genesis-bank-runtime/preparation.json`. About 7.57 GiB remains, below
the former 8-GiB planning choice, not a measured Bank requirement. Preserve Task 2.35's
recovery archive and historical outputs. Root owns normal integration-only Git
publication under D-026. Read the checkpoint/report; save/STOP before Task 2.37.
No new main acceptance or sensitive-operation authority is inferred.

The Task 2.35 record below is HISTORICAL; integration publication is complete.

Task 2.35 is TECHNICALLY VALIDATED / PENDING FOUNDER ACCEPTANCE for reversible local
build-capacity recovery. Start integration `9a72e3ae83852615b8da5df7d89113290574fa0d`;
main remains `4cc4ea11e87c2f1a2f85b9ed48358f2a881821b3`. One writer and separate
review were used; root passed 22 focused tests first attempt. All 8368 selected
non-executable Cargo intermediates had verified durable archives before removal;
one 22389024-byte library was actually restored and left in place. Root verified
26901 preserved files and no unexpected tree changes. Current free space is
8839438336 bytes (8.23 GiB). Preserve the complete recovery archive at
`/home/jerem/piv1-evidence/task-2.35-intermediates-20260928-a`; it contains 3902
objects plus manifest/journal, and permits restoration of 8367 remaining paths.
Recorded executables/SBF/logs/receipts/source/cache and production/economics remain
unchanged. No old test/build/runtime, dependency or live action ran. Historical
targets are no longer complete incremental caches. Root owns reviewed integration-only
publication under D-026. Read the checkpoint/report before Bank dependency work.
Save/STOP; Task 2.36 is NOT STARTED. No broader acceptance or main authority.

The Task 2.34 record below is HISTORICAL; integration publication is complete.

Task 2.34 is TECHNICALLY VALIDATED / PENDING FOUNDER ACCEPTANCE for a read-only
Bank prerequisite checker. Start integration `0f27932e434441f79e17f85453f5b38ccc076f1e`;
main remains `4cc4ea11e87c2f1a2f85b9ed48358f2a881821b3`. Delegated writer and
separate review were used; root passed 19 focused Python tests first attempt.
Actual environment observation correctly returns NOT_READY: 3301838848 available
bytes below an explicit 8-GiB planning reserve, six direct candidate package paths
missing. This is no measured Bank footprint, approved dependency pin, full closure
or Bank execution proof. Root reverified 157 prior inputs, 119 tools/nine aliases,
ten artifacts/final Task 2.33 host binary and 100 Task 2.33 logs; no old test rerun.
Production/economics/dependencies/artifacts remain unchanged. No compilation,
installation, cleanup or live operation occurred. Root owns reviewed integration-only
publication under D-026; Git records the commit. Read the active checkpoint before
capacity/dependency preparation. Save/STOP; Task 2.35 is NOT STARTED.

The Task 2.33 record below is HISTORICAL; its integration publication is complete.

Task 2.33 is TECHNICALLY VALIDATED / PENDING FOUNDER ACCEPTANCE within its
keyless local resource-failure/retry scope. Verified starting main/integration:
`4cc4ea11e87c2f1a2f85b9ed48358f2a881821b3`; D-029 publication is complete and
main remains at that commit. One delegated writer prepared eight isolated files;
separate source/runner/command/binary/evidence review passed. Root passed 13 runner
regressions and eight runtime tests/sixteen message cases, independently checking
2400 complete account records. Both 35/34 profiles reject at fixed 200k or
inside first/second Token CPI; paused prefunded cases preserve exact original rent
and 144-lamport normalization in raw staged state. Each actual returned-original
vector succeeds on fresh-runtime retry at 1.4m CU/default 32-KiB heap. This proves
Mollusk output discard and retry, not Bank/AccountsDB rollback or chain recovery.
Initial runner tests had two fixture-path failures; initial runtime had two exact
preflight-error assertion failures. Narrow reviewed test-only corrections preserve
all guards, budgets and custody/state/trace oracles; both failed runs are retained.
Production/probes/old harnesses/economics, 151 protected inputs and ten artifacts
remain unchanged. All 119 tools/nine aliases match; no installation/pin refresh or
new dependency. Earlier 448 logs are verified retained evidence, not reruns.
Root owns shared docs/Git and reviewed integration-only publication under D-026;
Git records the commit. Read the checkpoint and verify actual refs/worktree.
Save/STOP; Task 2.34 is NOT STARTED. Main acceptance and live gates remain.

The Task 2.32 record below is HISTORICAL; main/integration publication is complete.

Task 2.32 is TECHNICALLY VALIDATED / FOUNDER-AUTHORIZED MAIN INTEGRATION under
D-029 after the 2026-09-26 session. Starting integration was
`60193d63b64f42211f98d9b4910dfc047ab876df`; starting main was
`7b74be4b13c019b96a0c8abcbebfbcc361d31089`. D-029 covers the reviewed Tasks
2.24–2.32 sequence, without broader founder acceptance or live authority.
A delegated writer prepared an isolated harness; root passed 13 runner tests
and 13 runtime cases against the three unchanged Task 2.31 ELFs. Root and separate
review independently checked 2040 complete account records. Four initialization
successes use 1063693/1057103/984673/978083 CU with default 32-KiB heap and an
explicit 1.4m ceiling. A first build failed on obsolete test Rent fields; a first
runtime failed on test trace ordering. Minimal reviewed harness corrections
preserved all oracles; final build/run passed and both failures are retained.
Late failure proves initialized raw state then Mollusk output discard, not
Bank/AccountsDB rollback or in-initializer failed-CPI atomicity. Actual Squads,
full recipient control, funding provenance and live readiness remain unproved.
Production/probes/old harnesses/economics and 143 protected inputs are unchanged;
326 earlier logs are verified retained evidence, not reruns. Only new pins adopt
an independently verified installed libexpat hash; no installation occurred.
Final document and cumulative publication review passed. Root owns shared docs,
Git and normal main/integration publication. Git records the task commit/publication identity; verify actual
refs/worktree and read the checkpoint. Save/STOP; Task 2.33 is NOT STARTED.
All sensitive-action gates remain unchanged.

The Task 2.31 record below is HISTORICAL; publication is complete.

Task 2.31 is TECHNICALLY VALIDATED / PENDING FOUNDER ACCEPTANCE within its
build-only scope. Verified baseline integration is
`6238088f6dc8ef42a266b5041db9e0e50f262c85`; main remains
`7b74be4b13c019b96a0c8abcbebfbcc361d31089`. A delegated writer prepared three
isolated full-initialization validation artifacts: fixed 35/34-account synthetic
caller/callee and a canonical-ID, InitializeAccount3-only SPL Token 8 wrapper.
Root passed 15 Rust boundary tests and 11 mocked runner regressions, zero-example
doctest discovery and warning-denied docs. First strict SBF build passed with
34 commands/68 verified logs and no diagnostics; separate artifact review passed.
No new SBF runtime was executed. Existing production/probes/harnesses, economics,
128 protected inputs and six historical artifacts are unchanged; 250 earlier logs
remain verified retained evidence, not reruns. Only an in-memory new target profile
uses the previously verified libexpat hash; no install or old-pin change. Root owns
shared docs, Git and integration-only publication under D-026; Git records the
commit. Read the checkpoint and verify refs/worktree. Save/STOP; Task 2.32 is
NOT STARTED. Initializer runtime/resources/atomicity, actual Squads/control and
live readiness remain unproved; main acceptance and sensitive-action gates remain.

The Task 2.30 record below is HISTORICAL; publication is complete.

Task 2.30 is TECHNICALLY VALIDATED / PENDING FOUNDER ACCEPTANCE within its
keyless local preflight-runtime scope. Verified baseline integration is
`352fe7d4ecd8609d93cf2b0a2a96009018d3a7de`; main remains
`7b74be4b13c019b96a0c8abcbebfbcc361d31089`. A delegated writer prepared an
isolated harness; separate source/runner/build/binary/runtime review covers both
unchanged Task 2.29 probe ELFs. Root passed eleven runner regressions and twelve
runtime tests/cases, independently checking 1674 complete account records. An
initial host build failed on two missing-module imports in reused test support;
a minimal test-root re-export fixed them before the clean second build and first
runtime execution. Both 32/31-account profiles succeed with height-two CPI and
runtime Instructions/Clock/Rent; representative invalid cases reject as expected.
Success uses 288277/282070 CU under the explicit 1.4m ceiling and default 32KiB
heap, exceeding 200k; no ordinary-budget or full initializer-resource claim.
One verified installed libexpat hash changes only in the new profile; no install.
Production, probes, existing harness, economics and historical artifacts remain
unchanged. Earlier tests are verified retained evidence, not rerun. Root owns
shared docs, Git and integration-only publication under D-026; Git records the
commit. Read the checkpoint and verify refs/worktree. Save and STOP;
Task 2.31 is NOT STARTED. Main acceptance and live gates remain unchanged.

The Task 2.29 record below is HISTORICAL; publication is complete.

Task 2.29 is TECHNICALLY VALIDATED / PENDING FOUNDER ACCEPTANCE within its
build-only scope. Verified baseline integration is
`162f3b7b634633e2a5ab3011f0d746c4a4d15599`; main remains
`7b74be4b13c019b96a0c8abcbebfbcc361d31089`. Two isolated SBF probes prepare the
existing fixed 32/31-account recipient preflight through a synthetic caller.
A first host compilation failed in test integration; reviewed test-only fixes
then passed. Writer and root independently passed 10 Rust +9 runner tests; root
also passed zero-doctest discovery and warning-denied docs. The final strict
workspace SBF build and separate two-artifact static review passed after an initial
unused-result warning was resolved by explicit discard in the validation callee. No probe
runtime, actual Squads/control or initializer readiness claim follows. Production
sources, ABI, economics, existing harness and all historical artifacts remain
unchanged; earlier host/Node/runtime evidence is verified retained evidence, not
rerun. Root owns shared documents, Git and integration-only publication under
D-026; Git records the task commit. Read the checkpoint and verify refs/worktree.
Save and STOP; Task 2.30 is NOT STARTED. Main acceptance and live gates remain.

The Task 2.28 record below is HISTORICAL; publication is complete.

Task 2.28 is TECHNICALLY VALIDATED / PENDING FOUNDER ACCEPTANCE after recovery
on 2026-09-21 UTC. Verified baseline integration is
`560ca09c9c17becb79564c164e8c308b196c7cbe`; main remains
`7b74be4b13c019b96a0c8abcbebfbcc361d31089`. Two strict builds rejected oversized
genesis frames; private preflight boxes and separate construction helpers fixed
them without ABI, account bytes, dependency, validation-order or economic changes.
Final source passed 53 delegated focused tests, root 469 host tests +1 doctest/
eight gates and separate source/test review. The third strict SBF build, artifact
review, fresh harness build and exact-binary run passed: 24 local tests/70 cases,
1629 complete account records independently checked. Only dispatched claim/pending
paths gain runtime evidence; genesis total heap/resource/rollback proof remains
unproved. Sixteen target-runner/seven harness-runner regressions passed; unchanged
15 Node tests/eight old plus sixteen recipient cases are retained, not rerun.
Verified OS provenance justified narrow file-hash refreshes without installation.
Historical artifacts and both rejected builds are preserved. Root owns final
shared-document review, Git and integration-only publication under D-026; Git
records the commit. Read the checkpoint and verify actual refs/worktree. Save and
STOP; Task 2.29 is NOT STARTED. Main acceptance and live-operation gates remain.

The Task 2.27 record below is HISTORICAL; publication is complete.

Task 2.27 is TECHNICALLY VALIDATED / PENDING FOUNDER ACCEPTANCE after the
2026-09-20 bounded session. An explicit unsigned host transport profile covers
the complete Task 2.26 recipient-checked 35/34-account fixture while preserving
the old default CLI bytes. Delegated writer and root independently passed 15 Node
tests and eight old/sixteen new report cases on 96 matching inputs; first execution,
no failures/diagnostics. Separate source/test review passed after a pre-execution
test-oracle correction. All 92 Rust inputs and retained logs match 468 tests +1
doctest/eight gates from Task 2.26; Rust was not rerun. Native ABI, dependencies,
economics and runtime evidence are unchanged. Root publishes integration only
under D-026 after final document review; Git records the commit. Verified baseline
integration is `648998b4f5767eadf14c511d1dd0034ffff29ee0`; main remains
`7b74be4b13c019b96a0c8abcbebfbcc361d31089`. Read the active checkpoint and verify
refs/worktree. Save/STOP; Task 2.28 is NOT STARTED. Live-operation gates remain.

The Task 2.26 record below is HISTORICAL; its publication is complete.

Task 2.26 is TECHNICALLY VALIDATED / PENDING FOUNDER ACCEPTANCE after the
2026-09-20 bounded session. Its fixed normalized initialization profile checks
both recipients against the allocator's single fresh full preflight before any
effect and preserves their exact accounts after every successful System/Token
CPI and final completion. A delegated writer completed 30 focused tests; separate
source/test review passed. Root independently executed 468 host tests +1 doctest/
eight gates, zero failures or diagnostics. Both executions passed first attempt;
all 92 inputs and log/tool hashes match. Existing profiles, schema, model and
native dispatch remain unchanged; additive library error variants are documented.
Full recipient control, 35/34-account transport and current runtime proof remain
unproved, alongside funding provenance and later Token-native donation handling.
Root publishes integration only under D-026 after final documentation review;
Git records the task commit/publication identity. Baseline integration was
`3b45241aec5e0b6a9405f5df2516da66c42c6485`; main remains
`7b74be4b13c019b96a0c8abcbebfbcc361d31089`. Read the checkpoint and verify refs.
Save/STOP after this task; Task 2.27 is NOT STARTED. Live-operation gates remain.

The Task 2.25 record below is HISTORICAL; its publication is complete.

Task 2.25 is TECHNICALLY VALIDATED / PENDING FOUNDER ACCEPTANCE after the
2026-09-20 bounded session: a distinct read-only full genesis preflight verifies
the two approved recipient keys as funded System vault PDAs of the same current
governance multisig. One delegated writer completed eight focused tests after
one documented test-only correction; separate source/test review passed. Root
independently executed 462 host tests +1 doctest/eight gates, zero failures or
diagnostics, with 92 unchanged inputs. Existing APIs/economics remain unchanged.
Present recipient identity does not establish exclusive four-of-six spending,
absence of Squads spending limits/stale transactions or live artifact control.
The expanded preflight topology still needs future transport/runtime evidence.
Root publishes integration under D-026 after final documentation review; Git
records the task commit/publication identity. Verified baseline integration was
`ceeac8618a66f662e5d6a0369ada76b080ebee84`; main remains
`7b74be4b13c019b96a0c8abcbebfbcc361d31089`. Read the checkpoint and verify refs.
Save/STOP after this task; Task 2.26 is NOT STARTED. Live-operation gates remain.

The Task 2.24 completion record below is HISTORICAL; its publication is complete.

Task 2.24 is TECHNICALLY VALIDATED / PENDING FOUNDER ACCEPTANCE after the
2026-09-20 bounded session: same-call native prefund normalization from the two
unallocated Token PDAs into PendingSol preserves all original externally paid
rent obligations. A delegated writer completed implementation and 16 focused
tests; separate source/test review passed. Root independently executed 454 host
tests +1 doctest/eight gates, first pass, with 90 unchanged inputs and verified
logs/tools. Nine Node tests/eight transport cases remain retained evidence, not
rerun; current runtime proof remains deferred. Existing raw APIs and economic
decisions are preserved. Later Token-owned native donations remain unsupported.
Root publishes integration only under D-026 after final documentation review.
Main remains `7b74be4b13c019b96a0c8abcbebfbcc361d31089`; D-028 publication is
complete and does not cover Task 2.24. Git records the new task commit and
publication identity. Read the checkpoint and verify actual refs/worktree.
Task 2.25 is NOT STARTED; save and STOP after this bounded task.

The following D-028 publication record is HISTORICAL; actual Git refs were
reverified above on resumption.

On 2026-09-19 UTC, the founder conditionally authorized publication to main if
verification passed (D-028). Tasks 2.20–2.23 are TECHNICALLY VALIDATED /
FOUNDER-AUTHORIZED MAIN INTEGRATION for implementation
`3282e1ebabcb0cd88491d48a391565b8b100afa7`, plus its reviewed authorization record.
Root verified the four-commit ancestry from main
`5a067ad34d2f2ab0a27f2c6bff312a8b1f772d24`, all 94 inputs and 20 retained logs;
separate source review and publication safety checks passed. Prior 448 host tests
+1 doctest/eight gates and 9 Node tests/eight transport cases are retained evidence,
not rerun in this documentation-only publication turn. This authorizes main
integration, not broader founder acceptance, new economics or live operations.
Root publishes main and integration by normal fast-forward after final document
review; Git records the resulting documentation commit and publication identity.
Verify actual user, refs, HEAD and worktree on takeover. See the checkpoint and
D-028. Task 2.24 is NOT STARTED; save and STOP after publication. Native initializer
exposure, funding/prefund and recipient constraints, real transport lifecycle and
current runtime/resource/rollback proof remain deferred.

The founder explicitly accepted the Tasks 2.3–2.19 milestone at
`d9f3371be6ecb586675e3b38edcc57bd6e9519f8` and authorized its integration into
`main` on 2026-09-14 UTC, recorded as D-027. Root published that exact commit by
normal fast-forward from `66193769`; independent remote reads confirmed main and
integration at `d9f3371`, with the Task 2.3 branch unchanged. The historical D-027
documentation-only record updated acceptance status without extending the accepted
implementation or evidence. Verify actual current refs and worktree on takeover.

Tasks 2.3–2.19 are COMPLETE / FOUNDER-ACCEPTED within their documented bounded
scopes. The separate integration review passed; all 84 source inputs still match
the retained 416-host-test, one-doctest/eight-gate evidence. No tests were rerun
for acceptance documentation. The 24 SBF tests/70 cases remain historical Task
2.14 artifact evidence. No PR was created; its preparation is now historical.
At that acceptance, Task 2.20 had not started; later scopes remain in their reports.
Native initializer readiness and complete current runtime proof remain deferred.
D-028 now authorizes the specific later main integration recorded above; future
bounded development remains on integration under D-026 without routine approval.
Live-operation gates are unchanged.

The goal tool returns no goal; previous `usageLimited` reports are historical,
not a current credit estimate. D-026 remains active without routine approval gates.
Read `docs/PIV1_PILOT_STATE.md` and verify user/Git/agent state on takeover. Preserve
sensitive-action limits and economical usage; reuse unchanged reviews/evidence.

## Active technical pilot mandate

Read [docs/PIV1_PILOT_STATE.md](docs/PIV1_PILOT_STATE.md) on takeover and verify
the actual user, branch, HEAD and worktree before acting. The founder activated
[the technical pilot mandate](docs/PIV1_TECHNICAL_PILOT_MANDATE.md), recorded as
D-026, in this connected session. It supersedes earlier review-only and
per-task permission/stop requirements for bounded PIV1 technical work toward
founder Testnet testing. Use one delegated writer and a separate reviewer;
checkpoint each completed task before continuing. Technical validation is not
founder acceptance. Keep `main` within explicit founder acceptance or integration
authorization (D-027/D-028/D-029/D-031/D-033); use `integration/piv1-testnet` for the
reviewed development sequence. D-030 supplies the active six-milestone order and
critical-path test policy; earlier task-stop recommendations are historical.
The mandate's economic-decision,
secrets, signing and exact public-Testnet approval gates remain mandatory.

## Authority order

Use the following sources in descending order of authority:

1. `docs/PIV1_DECISIONS.md`
2. `docs/PIV1_MASTER_SPEC.md`
3. `docs/PIV1_CODEX_EXECUTION_PLAN.md`
4. Explicit newer founder decisions recorded in the repository
5. Older chats, memory, articles, and drafts only as historical context

If sources conflict, expose the conflict and follow the newest explicit founder decision for that exact component.

Use these statuses: `CONFIRMED`, `PROVISIONAL`, `OPEN`, `HISTORICAL`, and `REJECTED`.

Never invent a missing decision, address, percentage, mechanism, authority, version, or requirement.

## Current confirmed foundations

- PIV1 has its own dedicated Solana Program ID, not yet created in Task 0.2.
- Contributions use SOL and JitoSOL.
- JitoSOL is the initial strategy.
- SOL enters through direct Jito stake-pool deposit to JitoSOL.
- JitoSOL exits through delayed direct withdrawal via a stake account to SOL.
- V1 has no Jupiter/DEX core path.
- Beneficiary outputs are native SOL.
- The yield split is fixed at `59% / 19.5% / 19.5% / 2%`.
- Governance uses six guardians with a 4-of-6 threshold.
- Full program upgrade authority is held under a 4-of-6 Squads vault.
- The program includes an explicit emergency pause.
- Explicit SOL/JitoSOL deposits reject during pause before transfer or state
  effects (D-032); direct transfers already received remain reconcilable.
- Operations are permissionless and the caller pays transaction fees.
- Successful distribution preparations must be at least ten days apart.
- Only one distribution may be active at a time.
- A valid attempt below the technical withdrawal minimum creates no snapshot.
- A valid insufficient attempt starts a 24-hour cooldown.
- Malformed failed transactions do not update that cooldown.
- An operational SOL rent reserve is excluded from principal and yield.
- Principal and the high-water mark are accounted in SOL lamports.
- Yield uses official Jito/SPL pool accounting, not a DEX price.
- Production direct deposits and withdrawals use only the slippage-protected SPL variants, with a 1-bps immutable hard cap.
- One active distribution may use multiple deterministic validator withdrawal legs; settlement waits for exact target assignment and complete leg finalization.
- Validator discovery and leg execution are permissionless; Jito API preference is operational guidance while current on-chain SPL source-order and safety checks are authoritative.
- Principal and pending JitoSOL use distinct PIV1-derived legacy Token accounts controlled by the shared PIV authority; neither vault is an ATA.
- Cooldown rewards become explicit next-cycle yield, recovered temporary-account rent returns to operations, and cooldown loss enters recovery without reducing the HWM.
- Arithmetic is checked and outgoing calculations use conservative floors.
- The high-water mark has no normal downward reset.
- Pending contributions remain separate from historical yield.
- Claimable KIF rewards are earned only for active periods.
- With zero active guardians, 50% of available KIF compounds and 50% carries forward.
- KIF periods are fixed 2,592,000-second half-open intervals derived from Solana Clock.
- KIF carry is reapplied in every successive zero-active period, and active-guardian division remainder remains collective KIF carry.
- `claim_kif` remains allowed during the global emergency pause, but only for an already-earned recorded guardian liability paid from the isolated `KifSolVault` under the confirmed guardian and destination constraints.

## Permanent safety restrictions

Codex must never:

- deploy to Mainnet without explicit approval at that exact step;
- move real funds;
- create, reveal, or store Mainnet private keys or seed phrases;
- store secrets in Git;
- invent wallet or recipient addresses;
- transfer upgrade authority without explicit approval at that exact step;
- silently alter confirmed economics;
- silently replace JitoSOL or the delayed direct-withdrawal strategy;
- introduce a Jupiter/DEX core path in V1;
- disable or bypass tests or security gates;
- treat Testnet or Devnet addresses as Mainnet addresses;
- perform irreversible VPS actions without explicit authorization and a recovery path;
- use unpinned dependencies without written justification;
- claim that an AI review is a professional independent audit.

Mainnet keys must never be stored on this VPS, even in ignored files.

## Utilisation des skills Solana

- Respecter les versions de la toolchain et des dépendances présentes dans le projet.
- Respecter les règles économiques validées et documentées dans le dépôt.
- Les recommandations des skills doivent être adaptées au projet, sans déclencher automatiquement une migration.
- Toute migration ou modification des règles économiques doit être proposée séparément et recevoir mon accord avant d’être appliquée.

## Working protocol

- Work on one bounded implementation at a time. Under D-026/D-030, continue along
  the ordered production milestones after separate review, proportionate tests,
  demonstrated-defect correction and an integration checkpoint. A validation-only
  departure requires a documented critical-path blocker and closure evidence.
- Read applicable repository instructions and authoritative documents before editing.
- Keep code, comments, documentation, public interfaces, commit messages, and technical names in English.
- Ask the founder only for genuinely missing economic/governance decisions or required sensitive-operation authorization. Resolve ordinary technical incompatibilities within the mandate; preserve the permanent safety gates.
- Never reopen confirmed decisions merely to discuss alternatives.
- Keep principal, yield, pending contributions, beneficiary allocations, KIF claims, operational rent, and external fees separately accounted.
- Use checked integer arithmetic and conservative rounding.
- Preserve unrelated files and existing work.
- Do not add dependencies unless the active task authorizes them.
- Do not store secrets, wallet files, credential-bearing URLs, or private configuration in the repository.
- End every task with files changed, commands, validation/tests, security observations, Git status, and commit hash.
- Explicitly state whether any Mainnet action, deployment, fund movement, key creation, or authority transfer occurred.
- Outside the active D-026 mandate, stop after the requested task. Within it,
  preserve the sensitive-action gates and checkpoint before any interruption.

Phase 0, Task 0.5, Tasks 1.1-1.4, and the complete Phase 1 specification-as-code foundation are COMPLETE / FOUNDER-ACCEPTED. The final accepted Task 1.3 implementation tip is `527e381661fe0cfc27e07ad9b44e1601a638ae75`; the accepted Task 1.4 implementation is `06c39429f3237f6974e21217670c3f0d30b0a571`. Task 2.1 is COMPLETE / FOUNDER-ACCEPTED at initial implementation commit `33b1e539f969432f82635d1ca76c59d89f0ec233` and final corrected tip `cb90d468eff4dce60552ba15b2b267b364a47827`. Task 2.2 is COMPLETE / FOUNDER-ACCEPTED at implementation commit `e3233b96b533a620e8037d5231baede10877217f`. Phase 2 is IN PROGRESS. Task 2.3 is COMPLETE / FOUNDER-ACCEPTED within its documented scope (D-027) after correction `0559ebdaaaf28c7e9b8f423eda158abe13093b8d` on `task/2.3-vault-reconciliation-model`; Task 2.4 fixed-account authentication is COMPLETE / FOUNDER-ACCEPTED within its documented scope (D-027) at `9f4f1064deeef78a3cbea2e9f84c560e87166f20` on `integration/piv1-testnet`; Task 2.5 initial contribution bootstrap is COMPLETE / FOUNDER-ACCEPTED within its documented scope (D-027) at `9b997f364d62b0796008b2f7fb3f905acf64a2e5`; Task 2.6 protected principal SOL deposit composition is COMPLETE / FOUNDER-ACCEPTED within its documented scope (D-027) at `9f75aec59d732b2662c1b2c7626f2a8f48887619`, limited to zero-fee conversions preserving historical book value and HWM coverage; Task 2.7 isolated KIF claim authentication/accounting is COMPLETE / FOUNDER-ACCEPTED within its documented scope (D-027) at `10dceb5b2eac691ff19840190e951bd2ec547984` after 255 host tests, one doctest, checks/docs and separate review passed; Task 2.8 current guardian/Clock snapshot authentication is COMPLETE / FOUNDER-ACCEPTED within its documented scope (D-027) at `c815474eea9a7854c3b495974d891f4dd1c67a27` after 277 host tests, one doctest, checks/docs and separate review passed; Task 2.9 typed state envelopes and atomic existing-account byte persistence is COMPLETE / FOUNDER-ACCEPTED within its documented scope (D-027) at `36152c157773737eca357e5dbfefd3f0900b6eb3` after 298 host tests, one doctest, checks/docs and separate review passed; Task 2.10 isolated KIF claim execution with explicit host invocation/rollback evidence is COMPLETE / FOUNDER-ACCEPTED within its documented scope (D-027) at `c38a7b0d7122144bf3082cec7e57bbea7e61cc10` after 318 host tests, one doctest, checks/docs and separate review passed; Task 2.11 claim instruction ABI/runtime-ID boundary is COMPLETE / FOUNDER-ACCEPTED within its documented scope (D-027) at `2eeefba0abc226bcfcadddb5f248f12ca589e09d` after 335 host tests, one doctest, checks/docs and separate review passed, within `docs/TASK_2_11_KIF_CLAIM_INSTRUCTION_BOUNDARY.md`; Task 2.12 keyless SBF compilation is COMPLETE / FOUNDER-ACCEPTED within its documented scope (D-027) at `cee6072ad7b3155d7d5b30e6c0830beb6d00d4eb` after a clean target build, 339 host tests, one doctest, checks/docs and separate source/artifact review passed, limited to the static evidence in `docs/TASK_2_12_KEYLESS_SBF_COMPILATION.md`; Task 2.13 keyless local SBF claim execution is COMPLETE / FOUNDER-ACCEPTED within its documented scope (D-027) after 19 local tests, 60 message cases, full account evidence and separate final review in `docs/TASK_2_13_KEYLESS_SBF_CLAIM_EXECUTION.md`; Task 2.14 runtime pending recognition is COMPLETE / FOUNDER-ACCEPTED within its documented scope (D-027) at `7442cab7e97c422c7ee06290d5fc9d11c8b13ee6` after 349 host tests, one doctest, eight gates, 24 local SBF tests and final separate review in `docs/TASK_2_14_RUNTIME_PENDING_RECONCILIATION.md`; Task 2.15 read-only Squads authority snapshot is COMPLETE / FOUNDER-ACCEPTED within its documented scope (D-027) at `da0241fd2a9f9c3247bbdeabb1a6b9c37dabc912` after 363 host tests, one doctest, eight gates and separate source/documentation review in `docs/TASK_2_15_SQUADS_AUTHORITY_SNAPSHOT.md`; Task 2.16 bounded direct Squads invocation authorization is COMPLETE / FOUNDER-ACCEPTED within its documented scope (D-027) at `343a496fb16edc8fd8d68746a89323d7545ab36d` after 383 host tests, one doctest, eight gates and separate source/test review in `docs/TASK_2_16_SQUADS_INVOCATION_AUTHORIZATION.md`; Task 2.17 separate bootstrap authorization is COMPLETE / FOUNDER-ACCEPTED within its documented scope (D-027) at `69f7289e4c3ea2821141c4bcaded5ae942eed979` after 392 host tests, one doctest, eight gates and separate source/test review in `docs/TASK_2_17_SQUADS_BOOTSTRAP_AUTHORIZATION.md`; Task 2.18 approved genesis model preparation is COMPLETE / FOUNDER-ACCEPTED within its documented scope (D-027) at `dc51450396a0e369e690d9038b7dde1a80e2ecd6` after 404 host tests, one doctest, eight gates and separate source/test/report review in `docs/TASK_2_18_APPROVED_GENESIS_MODEL.md`; Task 2.19 source-pinned Jito account identity authentication is COMPLETE / FOUNDER-ACCEPTED within its documented scope (D-027) at `b9f6f43d21713b3ec0bf81403378819f7cd3e44e` after 416 host tests, one doctest, eight gates and separate source/test/dependency/report review in `docs/TASK_2_19_JITO_ACCOUNT_IDENTITY.md`; Task 2.20 genesis account preflight is TECHNICALLY VALIDATED / FOUNDER-AUTHORIZED MAIN INTEGRATION (D-028) after 426 host tests, one doctest, eight gates and separate review within docs/TASK_2_20_GENESIS_ACCOUNT_PREFLIGHT.md; Task 2.21 genesis account allocation is TECHNICALLY VALIDATED / FOUNDER-AUTHORIZED MAIN INTEGRATION (D-028) after 438 host tests, one doctest, eight gates and separate source/test review within docs/TASK_2_21_GENESIS_ACCOUNT_ALLOCATION.md; Task 2.22 same-call genesis initialization is TECHNICALLY VALIDATED / FOUNDER-AUTHORIZED MAIN INTEGRATION (D-028) after 448 host tests, one doctest, eight gates and separate source/test review within docs/TASK_2_22_GENESIS_ACCOUNT_INITIALIZATION.md; Task 2.23 host genesis transport validation is TECHNICALLY VALIDATED / FOUNDER-AUTHORIZED MAIN INTEGRATION (D-028) after nine Node tests, eight deterministic packet cases and separate source/test review within docs/TASK_2_23_GENESIS_TRANSPORT.md; Task 2.24 same-call genesis Token-native prefund normalization is TECHNICALLY VALIDATED / PENDING FOUNDER ACCEPTANCE after 454 host tests, one doctest, eight gates and separate source/test review within docs/TASK_2_24_GENESIS_TOKEN_PREFUND_NORMALIZATION.md; Task 2.25 fresh genesis recipient-vault identity preflight is TECHNICALLY VALIDATED / PENDING FOUNDER ACCEPTANCE after 462 host tests, one doctest, eight gates and separate source/test review within docs/TASK_2_25_GENESIS_RECIPIENT_PREFLIGHT.md; Task 2.26 recipient-checked normalized genesis initialization is TECHNICALLY VALIDATED / PENDING FOUNDER ACCEPTANCE after 468 host tests, one doctest, eight gates and separate source/test review within docs/TASK_2_26_RECIPIENT_CHECKED_GENESIS_INITIALIZATION.md; Tasks 2.27 recipient-checked transport, 2.28 current SBF/runtime refresh, 2.29 isolated probe preparation, 2.30 keyless preflight runtime and 2.31 full-initialization probe preparation are TECHNICALLY VALIDATED / PENDING FOUNDER ACCEPTANCE within their task reports. Task 2.32 full-initialization runtime is TECHNICALLY VALIDATED after 13 runtime cases, 13 runner tests and independent review of 2040 account records. D-029 authorizes normal main integration of reviewed Tasks 2.24–2.32, without broader founder acceptance. Task 2.33 internal resource failures/retries are TECHNICALLY VALIDATED / PENDING FOUNDER ACCEPTANCE after eight runtime tests/sixteen messages, 13 runner tests and independent full-account review. Task 2.34 read-only Bank prerequisite tooling is TECHNICALLY VALIDATED / PENDING FOUNDER ACCEPTANCE after 19 focused Python tests and separate review; actual Bank prerequisites remain NOT_READY. Task 2.35 reversible build-capacity recovery is TECHNICALLY VALIDATED / PENDING FOUNDER ACCEPTANCE after 22 focused regressions, verified archival and real restoration, with production/evidence preserved. Task 2.36 isolated Bank dependency preparation is TECHNICALLY VALIDATED / PENDING FOUNDER ACCEPTANCE with authenticated source and metadata evidence. Task 2.37 native System Bank smoke is TECHNICALLY VALIDATED / PENDING FOUNDER ACCEPTANCE after 12 runner tests, four Bank message cases and independent inspection of 100 complete account records; the task report records compatibility corrections and retained failed builds. Task 2.38 is technically validated at `8eee7cb884f2e37bb2a31c48aed77557cf4fb62d`. Task 2.39 completes M1 within its documented production boundary/runtime scope at `bf32d87e06a2d54c8e1c0192faaa7855d246dbc0`; D-031 authorizes main integration of the validated Tasks 2.33–2.39 sequence without broader founder acceptance. M2 is in progress under D-030; Task 2.40 exposes heartbeat/pause with host/static-SBF evidence, while remaining lifecycle handlers and current full-runtime proof remain open. Task 2.1 accepts the narrow interface and host-mock evidence, not exact SPL/Jito behavior; the collision-safe, account-derived production snapshot identity and real protocol mapping remain Phase-3-provisional. Task 2.2 accepts only pure pending-vault intake/reconciliation and host-mock evidence. At Task 2.2 acceptance, fixed-account and transfer validation, real custody, handlers, CPI, localnet behavior, all-vault normalization, and composition cases involving pending SOL moved into distribution escrow were deferred; explicit-transfer handler callability during pause was PROVISIONAL and is now resolved by D-032 (blocked while paused). Task 2.3 adds pure phase-dependent custody derivations and atomic host composition for pending-to-escrow-to-HWM and economic-vault normalization; it does not authenticate real accounts or transfers. Operational surplus derivation remains unsupported without an authenticated funding baseline. See `docs/TASK_2_3_VAULT_RECONCILIATION_MODEL.md` for the supported scope and deferred cases. That review-only next-action restriction is HISTORICAL after D-026 activation. The current technical task, reviewed development progression and permission boundaries are recorded in docs/PIV1_PILOT_STATE.md; Tasks 2.3–2.19 were founder-accepted under D-027. This AI-assisted review is not a professional independent audit.
