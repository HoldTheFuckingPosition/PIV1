# Historical pilot checkpoint snapshot

Status: **HISTORICAL**. Preserved before Task 2.13 closure on 2026-09-09.
For current execution state, read [PIV1_PILOT_STATE.md](../PIV1_PILOT_STATE.md).
The following prior checkpoint is retained verbatim; its current-state statements
are superseded by the live checkpoint.

# PIV1 technical pilot checkpoint

Execution: **ACTIVE — founder explicitly resumed with GPT-6 on 2026-09-09 at 18:33 UTC.**
The founder's newer "si c'est bon gpt 6 on peut reprendre stp" accepts the confirmed
GPT-6 family and supersedes the exact-Astra-label condition below. Root reverified
`jerem` uid 1001, the single worktree, branch `integration/piv1-testnet`, unchanged
HEAD `f9b462b3a1b5bf251ed9b196ee5ff5f5a99468b8` and preserved uncommitted Task 2.13
preparation. Reconcile the saved classifier/CRT corrections and stale freeze, verify
delegated availability, and complete separate review before compilation/runtime.
No founder acceptance or sensitive-action authorization follows from resumption.

First Task 2.13 build release (historical): the writer
completed and released the corrected eight-file freeze
`54f50ff926820c9930d350217fe69012227e830b44b47bd2fbececf0e0d8cf60`; the separate
reviewer returned PRE-EXECUTION PASS without remaining findings. T213-R1 and the
missing CRT pin are resolved in reviewed source. Corrected preflight-b passed,
with 11 successful commands and 22 log hashes independently checked by root and
reviewer. Root also personally ran six unchanged stdlib refusal tests: PASS.
Root now holds the single build slot for the final locked/offline, one-job,
`--no-run` command in the Task 2.13 report; output is
`/tmp/piv1-sbf-claims-build-20260909-a`. No harness runtime execution is released
until the actual built executable and its identity have been inspected. Earlier
pause/preparation entries below remain chronological evidence only.

Current Task 2.13: **HOST BUILD PASS — FIRST LOCAL RUNTIME RELEASED**.
Root's third build passed at 19:19:18 UTC after 169.294 seconds, with zero compiler
diagnostics, all 12 successful commands/24 log hashes independently checked and
preservation PASS. Result SHA `0ef283a217a21ba4d1699d7d20be8eb883e8304077bdc8ed5622f23b9617a505`.
Actual 96591016-byte x86-64 PIE executable `...build-20260909-c/target/debug/deps/claims-c12d45bbd63e9bf4`
has SHA `c3a8cf5106109c3986e86cbd1b886442c2b4516698fb56814ee0ae0a915cd356`.
Root inspected ELF headers/dynamic libraries and matched all four libraries to
existing pins; the separate reviewer returned run-only PASS on the exact binary,
source/build preservation and command. Root now releases direct execution through
the unchanged runner into fresh `/tmp/piv1-sbf-claims-run-20260909-a`. No runtime
success, founder acceptance, signatures, Bank rollback or live evidence is implied.

Historical third-build release:
T213-R2's full-account records and positional 64-byte/tail regression passed
separate delta review. Root independently verified the eight-file freeze
`1048fa3376527d765390f8b9c6214fbef46062336aeb91a95a8c659c69baaa36`, preflight-e
result `97e7769aa72743ece425e04eee9f472fb3e35f5efdc8f2464965a588f923df65` and its
11 successful commands/22 output hashes. Only two source pins changed; all other
pin sections remain equal. The writer released ownership. Root now holds the sole
slot for fresh `...build-20260909-c`, unchanged runner and pins
`a0b388a11ae94704038b9ff177978979f1c7c219cb9ea8c60c8733e5ac5c0ad7`.
Nineteen tests are prepared, none executed. Inspect the resulting executable
before runtime; this review does not establish compilation success.

Historical durable-evidence correction:
The API correction passed root inspection and separate build-only review on
freeze `eec1919112dacf2bada95e71a854f73f7913dea042f7ea9dbb121fbb189308d5`;
preflight-d passed. Before executing that command, root found **T213-R2**:
upstream `Account::Debug` truncates data at 64 bytes, so recorded Config/reward
tails would be missing despite complete in-memory assertions. The separate reviewer
confirmed this exact source behavior. The writer owns a minimal full-account log
formatter, a tail-coverage regression and refreshed pins/evidence. No build-c
execution occurred; its prior source-only release is superseded. Source/runtime
behavior and production remain unchanged; review the new delta before building.

Historical second-build/API correction evidence:
The second build ran 18:54:39–18:58:02 UTC (203.361 seconds), Cargo 101 / FAIL.
All selected dependencies compiled, then the harness reported two references to
nonexistent `mollusk_svm::result::TransactionResult`, a missing `Debug` implementation
for `SVMFeatureSet`, and one deprecated `NotEnoughAccountKeys` warning. Root verified
all 24 command output hashes and preservation PASS. Output `...build-20260909-b`
remains intact; result SHA `8ac9e323ac6a7284622759ea37c9e9bc5cb104e1ae5f42d66c200cae192d3e15`.
No executable/runtime evidence exists. The sole writer may correct those isolated
API/provenance references and refresh source pins/report for separate delta review;
retain the old program's exact insufficient-account error semantics. Production,
dependencies and runner remain unchanged. Root released the build slot.
The founder requested economical credit use and a progress/publication explanation;
avoid redundant validation and broad delegation. Root re-read the actual remote:
integration remains published at `f9b462b`, accepted main at `6619376`.

Historical second build release:
At 18:54 UTC the separate reviewer returned PRE-EXECUTION DELTA PASS for the
two-package correction, with no actionable blocker. Root independently verified
the eight-file freeze `e63d325bb26d91a6936448a52d52949214c576300f94aad56a708904507a78fa`,
preflight-c result `59d0a412f6d978065aad56fc4981e1f858146eb7fcf9a0db7a5708778eb2d5b4`,
all 11 successful commands and 22 log hashes, the exact four removals/two additions,
unchanged retained features/checksums and all 138 PIV dependency nodes/edges.
The pilot holds the sole build slot for the report's locked/offline, one-job
`--no-run` command into fresh `/tmp/piv1-sbf-claims-build-20260909-b`, with runner
`3d103a372d556e0090f78b4dae97e45fc6a4ec90694af990931d877822e37c8f` and pins
`c404577ac9b03876d153c33910b36ee5badbfaa9b4742556788d43e1e4168e1a`.
No compilation success or runtime result is inferred from this release.

Historical first build and correction:
Root's actual first build ran at 18:38:55–18:39:52 UTC and failed with Cargo 101
after 57.24 seconds. Eight E0277 diagnostics in `solana-instruction 3.4.0` expose
a `wincode 0.5`/`0.6` trait mismatch: its `Pubkey` re-export resolved through
`solana-address 2.7.0`, while the revision-bound Mollusk lock uses address 2.6.1.
No harness executable/test was reached. All 24 command log hashes independently
matched; source/tool/package/ELF/ref preservation passed. Failed output
`/tmp/piv1-sbf-claims-build-20260909-a` remains intact, result SHA-256
`7faa00da200039b124bc63922e029e27cee296065322474dc1a1f15fd20b0908`.
Root released the build slot. The sole writer may inspect and constrain the
isolated upstream-compatible address version, acquire its pinned public source,
refresh lock/metadata/pins and report the precise changed inputs for separate
review. Production, SBF artifact and other selected SDK/runtime versions stay
unchanged. No build/runtime retry is released before that delta review; this is
a harness dependency compilation failure, not a demonstrated PIV runtime defect.

The address-only intermediate graph revealed the same cross-crate trait mismatch
through `solana-short-vec 3.3.0`: message 4.4.0 / transaction 4.1.5 require the
0.5 serialization traits for its `ShortU16`, but that newer package implements
0.6 traits. Root and reviewer independently confirmed the exact source usage.
The writer's bounded correction therefore also pins upstream short-vec 3.2.2,
with default features disabled on both explicit compatibility constraints.
The address-only intermediate metadata is retained; it was never compiled.
All other selected versions and production inputs must remain unchanged.

Historical model pause:
On 2026-09-09 the founder explicitly instructed that technical progression must
not restart until the exact Astra model is confirmed. The available GPT-6 family
instruction alone does not establish that exact label. Both delegated agents
`implement_t26_deposit` and `review_t23_final` are errored on usage limits; neither
returned a final Task 2.13 preparation review PASS. Do not restart agents, compile,
run the SBF harness or begin later work while this model condition is unresolved.

Pause verification: user `jerem`, single worktree on `integration/piv1-testnet`,
HEAD `f9b462b3a1b5bf251ed9b196ee5ff5f5a99468b8`. Task 2.12 remains completed in its
documented technical scope; Task 2.13 means phase 2, task 13, and remains prepared
but not compiled or runtime-validated. All uncommitted preparation is preserved.
The writer saved the T213-R1 classifier correction and added the `crtendS.o` pin
before interruption, but the final freeze/report and separate review were not
completed. Current observed SHA-256: `tests/claims.rs`
`7e7b0217cf3e483643531500a2191c0219bfbb03c27314ab8d5911ac1f4911e8`;
`tools/sbf_claims_pins.json`
`647c28cf52db63d33f6c5fed049a341090283a0706c4748706949fdf860e4fef`.
These supersede the initial preparation hashes for current file identity only;
they do not inherit its preflight evidence or imply final review approval.
When resumption is permitted, reconcile the actual files and stale freeze/report,
finish separate review, then release a build. No code, commit, push, deployment,
fund movement, key creation, signing or authority transfer occurred in recording
this pause; only documentation was updated.

Historical execution: founder resumed on 2026-09-09 at 09:20 UTC.
The overnight pause ended with the founder's explicit "on reprend". The pilot
verified user `jerem`, the single clean worktree, local/remote integration HEAD
`bff59bb53ebb56875a7b34ae055cc4aa8d031fb9`, unchanged accepted main and no running
subagents. No engineering work was performed during the pause. Resumption does
not grant founder acceptance or live-operation authorization.

Last verified: **2026-09-09 UTC**. On takeover, read this file and `AGENTS.md`,
then verify actual user, branch, HEAD, worktree and running agents. This is an
execution checkpoint, not a competing economic specification.

## Goal and mandate

Deliver reviewed PIV1 implementation, authorized public-Testnet end-to-end
evidence and a practical founder testing handover. D-026 activates the complete
[PIV1_TECHNICAL_PILOT_MANDATE.md](PIV1_TECHNICAL_PILOT_MANDATE.md): successive
bounded tasks, one delegated writer, separate review, normal commits and clean
reviewed development publication are authorized without routine permission.
Checkpoint each task before the next. Technical validation is not founder
acceptance. French brief founder reports; English code/docs/delegation/commits.

Authority: `PIV1_DECISIONS.md`, `PIV1_MASTER_SPEC.md`,
`PIV1_CODEX_EXECUTION_PLAN.md`, newer explicit component decisions. Economics,
guardian/upgrade custody, direct Jito strategy and protected accounting remain
unchanged. PIV1 is the first HTFP infrastructure component; other tokens, Team
Owner components and MTT are out of scope. `HTFP_MASTER_CONTEXT.md` was not found;
no other ChatGPT/browser-history access is assumed.

## Verified Git and acceptance

- User `jerem` (uid 1001), `/home/jerem/piv1`, one worktree.
- Remote: `github-piv1:HoldTheFuckingPosition/PIV1.git`. WeatherTrader2 is the
  connected project label, not another detected repository.
- Branch: `integration/piv1-testnet`. Latest implementation:
  **Task 2.12 `cee6072ad7b3155d7d5b30e6c0830beb6d00d4eb`**; reviewed closure
  **`f9b462b3a1b5bf251ed9b196ee5ff5f5a99468b8`** was normally fast-forward
  published and independently reread remotely from a clean worktree. Previous published **Task 2.11 `2eeefba0abc226bcfcadddb5f248f12ca589e09d`**. Reviewed closure
  **`14106d664c107b3a2f705ac87361768af42d0786`** was normally fast-forward published and
  independently reread remotely from a clean worktree at publication. Task 2.10
  closure **`9f9a8dbba132f96e4c76a8385844570746b0636f`** was independently
  reread remotely from a clean worktree at publication. Task 2.9
  closure **`44d70ec911ad3a78738fb90a04ac82eec3ca44f2`** was independently
  verified remotely from a clean worktree. Inspect actual HEAD/remote on takeover.
  Task 2.8 implementation `c815474eea9a7854c3b495974d891f4dd1c67a27` and
  closure `440e83e26d36df911ccfafac97d89b79b8b4c694` are already published.
  Task 2.7 implementation `10dceb5b2eac691ff19840190e951bd2ec547984` and
  closure `37f25a8b84e0d4060b36fe0c86ff8bea8e4aa3aa` are already published.
  Previous Task 2.6 implementation:
  **`9f75aec59d732b2662c1b2c7626f2a8f48887619`**. Published validation/pause
  closure: **`ea6f09ccfde9811b7be233a960b984c1c74c1e6a`**. This subsequent
  documentation checkpoint records completed publication; verify actual HEAD
  and remote on return rather than treating an embedded hash as current forever.
- Accepted local/remote `main`: **`66193769d1cbc59cd8630df295b9a784b9c64642`**,
  independently reread remotely after Task 2.12 publication. Do not move main.
- Phase 0, Phase 1 and Tasks 2.1/2.2 are founder-accepted. Tasks 2.3–2.12 are
  **TECHNICALLY VALIDATED / PENDING FOUNDER ACCEPTANCE** only in reported scope.
- Original Task 2.3 publication: `46b448dbfd5670326a19d2292801181939ab2dd0`.
  Mandate activation: `df1250064011428b88a6ef7aae8b0c42521f5e95`.
- Corrected Task 2.3 branch local/remote:
  `3677fee97e3617ee65e2828d222008ba0952bb3e` (includes correction and checkpoint).
  Integration was created there by normal atomic push; main was not pushed.
- Verified completed integration publications: Task 2.4 closure
  `a1d585d117802fb8e595f089b0604527d61047d2`, then Task 2.5 closure
  `c58580fb3ad01e0e98243652c1c3d1f8dafdf01f`. Task 2.6 code and pause closure
  were then published successfully by normal fast-forward from `c58580f` to
  `ea6f09ccfde9811b7be233a960b984c1c74c1e6a`. No main push occurred.

## Completed technical sequence and evidence

| Task | Implementation | Actual pilot execution on final source | Separate source review |
|---|---|---|---|
| 2.3 severe-loss correction | `0559ebdaaaf28c7e9b8f423eda158abe13093b8d` | 168 tests +1 doctest; checks/docs PASS | `review_t23_final`: PASS |
| 2.4 fixed AccountInfo authentication | `9f4f1064deeef78a3cbea2e9f84c560e87166f20` | 191 tests +1 doctest; checks/docs PASS | `review_t23_final`: PASS |
| 2.5 initial contribution bootstrap | `9b997f364d62b0796008b2f7fb3f905acf64a2e5` | 209 tests +1 doctest; checks/docs PASS | `review_t23_final`: PASS |
| 2.6 protected principal SOL deposit composition | `9f75aec59d732b2662c1b2c7626f2a8f48887619` | **231 tests +1 doctest**; checks/docs PASS | `review_t23_final`: PASS |
| 2.7 isolated KIF claims | `10dceb5b2eac691ff19840190e951bd2ec547984` | **255 tests +1 doctest**; checks/docs PASS | `review_t23_final`: PASS |
| 2.8 current guardian/Clock snapshot authentication | `c815474eea9a7854c3b495974d891f4dd1c67a27` | **277 tests +1 doctest**; checks/docs PASS | `review_t23_final`: PASS |
| 2.9 validated state envelopes/atomic byte persistence | `36152c157773737eca357e5dbfefd3f0900b6eb3` | **298 tests +1 doctest**; checks/docs PASS | `review_t23_final`: PASS |
| 2.10 isolated KIF execution library/host model | `c38a7b0d7122144bf3082cec7e57bbea7e61cc10` | **318 tests +1 doctest**; checks/docs PASS | `review_t23_final`: PASS |
| 2.11 claim ABI/runtime-ID instruction boundary | `2eeefba0abc226bcfcadddb5f248f12ca589e09d` | **335 tests +1 doctest**; eight gates PASS | `review_t23_final`: PASS |
| 2.12 keyless SBF build/stack correction | `cee6072ad7b3155d7d5b30e6c0830beb6d00d4eb` | **339 tests +1 doctest**; eight gates PASS; independent static artifact inspection | `review_t23_final`: PASS |

Pilot commands use `/home/jerem/.cargo/bin/cargo +1.97.1`, `--locked --offline`:
`test --workspace --all-targets --quiet`, `test --workspace --doc`,
`check --workspace --all-targets`, the same check with `--all-features`, and
`doc --workspace --no-deps` with `RUSTDOCFLAGS='-D warnings'`. Rustfmt is absent;
no formatting pass was claimed or component installed. Final source was frozen
and hashes rechecked through gates. Diff whitespace and targeted ownership/
credential/generated-file checks passed. Host/model evidence is not runtime/CPI
or Testnet evidence, and AI review is not a professional independent audit.

**Preserved T23-R1 (P2), resolved:** original `World::finalize`/`settle` subtracted
already-used pending SOL from a smaller retained value before entering recovery.
Only the two recovery comparison inputs now use zero in that severe-loss case;
generic checked arithmetic and production math remain unchanged. Exact cases
retained 99 / pending use 4000 and retained 100 / pending use 8050 are regressions.
Finalization commits recovered stake and both rents while retaining HWM;
settlement commits only the recovery header and discards speculative payments,
KIF compound/carry and liability changes. The pilot personally ran the original
164 tests +1 doctest and reproduced both failures on `46b448d`. The writer's four
new tests failed on the old helper, then passed after correction. One older
review agent returned INCOMPLETE because of tool approval handling; it was never
counted as PASS. The replacement reviewed actual source. Old ephemeral binary
`/tmp/piv1-task23-recovery-review-46b448d` still represents OLD code; use committed
regressions. Full evidence: [Task 2.3 report](TASK_2_3_VAULT_RECONCILIATION_MODEL.md).

Task 2.4 authenticates Config, ActiveDistribution and seven fixed custody accounts
using actual host AccountInfo bytes, canonical PDAs/bumps, owner/layout/rent/token
checks and state binding. It narrowly permits the canonical zero System Program
ID in that role. Pinned SPL Token 8.0.0 added six locked transitive packages; the
pilot verified existing package identities/checksums unchanged. No layout change.
[Task 2.4 report](TASK_2_4_ACCOUNT_AUTHENTICATION.md) records 23 writer account
tests plus nine Config tests and exact authentication/trust limitations.

Task 2.5 establishes initial principal from full recognized pending SOL/tokens,
with no economic history, fabricated yield or arbitrary initial sequence. The
true empty host fixture establishes its audit once before actions. Zero-valued
positive token units remain principal and prevent replay. No general Idle intake
or staking was implied. Writer: 49 focused/affected tests PASS; old composition
counters unchanged. [Task 2.5 report](TASK_2_5_INITIAL_CONTRIBUTION_BOOTSTRAP.md).

Task 2.6 independently verifies protected zero-fee deposit receipts, normalized
individual custody, combined holdings/supply, exact post-pool deltas and historical
book value. Only historical SOL/token units change; HWM and all other state remain
unchanged. Deposited native/minted-user audit counters reconcile to the unchanged
pool audit; prior withdrawal/burn/fee/rent/recovery equations and baselines remain.
Writer `implement_t26_deposit`: initial 20 tests, then final 95 focused/affected
tests including 22 deposit regressions PASS. Pilot read all source/tests and ran
the 231-test workspace gates. Reviewer read exact final diff/all 22 tests and
found no actionable defect; no reviewer builds. Report status cleanup only after
freeze. Genuine later deposit preserves cooldown carry 13, both rents totaling
30, fees/burns and completed history. [Task 2.6 report](TASK_2_6_PROTECTED_PRINCIPAL_DEPOSIT.md).

## Reviewed progression after resumption

- [Task 2.7](TASK_2_7_ISOLATED_KIF_CLAIMS.md): isolated immutable earned ownership,
  historical/paused claims, full backing/carry/rent/excess and exact atomic host
  custody. Writer 69 affected tests; pilot 255 +1; separate exact review PASS.
  Closure `37f25a8b84e0d4060b36fe0c86ff8bea8e4aa3aa` published normally.
  Modeled credits do not prove earning authority; imported fixtures establish
  a new baseline, not continuous cross-World custody. Original audits remain.
- [Task 2.8](TASK_2_8_GUARDIAN_CLOCK_SNAPSHOT_AUTHENTICATION.md): read-only current
  registry/six rewards/canonical Clock snapshot, exact period equality, preserved
  historical claims and layouts. No current-six/global liability equality or
  heartbeat pause policy. Writer 69 affected; pilot 277 +1; separate review PASS.
  Closure `440e83e26d36df911ccfafac97d89b79b8b4c694` published normally.
- [Task 2.9](TASK_2_9_STATE_ENVELOPE_PERSISTENCE.md): canonical zero-tail typed
  envelopes and atomic existing-account byte writes for four authenticated state
  types. No transition authority, initialization or WithdrawalLeg persistence.
  Fixture lifetime errors, mistyped target, Config Option offset and sequence
  assertions were corrected before freeze; no production defect found.
  Writer 112 affected; pilot 298 +1; separate review PASS. Closure
  `44d70ec911ad3a78738fb90a04ac82eec3ca44f2` published normally.
- [Task 2.10](TASK_2_10_ISOLATED_KIF_CLAIM_EXECUTION.md): fixed System transfer
  wiring, simultaneous mutable native data/lamport preflight, canonical CEI
  persistence and exact fresh postconditions. Ordinary host execution rejects;
  explicit invocation/transaction modeling retains the unchanged custody audit.
  Raw errors can leave effects and must propagate to runtime rollback. Writer's
  unused import warning was removed; final 132 affected tests passed. Pilot
  318 +1, checks/docs, frozen source hashes and separate exact review PASS.
  Normal atomic fast-forward publication `44d70ec` to closure
  `9f9a8dbba132f96e4c76a8385844570746b0636f` completed. Independent remote
  reads confirmed integration, unchanged main/Task 2.3 and a clean worktree.
  No actual signed CPI/SBF runtime evidence or founder acceptance is implied.

The reports contain full source inventories, failed/final commands, individual
review verdicts and temporary evidence paths. Committed code/reports are durable;
`/tmp` logs can disappear. New-agent creation reached the thread limit, but the
existing native writer/reviewer are callable and reused. No delegation is assumed.

## Prior completed tasks and chronological preparation evidence

Completed [Task 2.11](TASK_2_11_KIF_CLAIM_INSTRUCTION_BOUNDARY.md) is technically
validated, pending founder acceptance. Code `2eeefba0abc226bcfcadddb5f248f12ca589e09d`,
closure `14106d664c107b3a2f705ac87361768af42d0786`: normal atomic fast-forward
publication and independent remote/main/clean-worktree checks completed. The pilot
ran 335 tests +1 doctest and eight checks/docs gates on frozen source; separate
source/test review PASS. ABI/runtime-ID/event/error details and primary compiler
research are in that report. No runtime proof or live authorization followed.

Completed [Task 2.12](TASK_2_12_KEYLESS_SBF_COMPILATION.md) is **TECHNICALLY
VALIDATED / PENDING FOUNDER ACCEPTANCE** after actual keyless SBF compilation,
final pilot host gates and separate exact source/artifact/evidence PASS. Branch
remains `integration/piv1-testnet`; implementation
`cee6072ad7b3155d7d5b30e6c0830beb6d00d4eb` contains thirteen reviewed source/test/tool
files. Six status/report files complete closure
`f9b462b3a1b5bf251ed9b196ee5ff5f5a99468b8`. Normal atomic fast-forward publication
from `14106d6` completed at 13:37 UTC. Independent remote reads confirmed integration,
unchanged accepted main/Task 2.3, and the worktree was clean at publication.
This follow-up publication checkpoint and the Task 2.13 scope are the current
root-owned documentation work.

Two earlier target attempts remain FAILED despite Cargo zero. The first had frames
5248/5632/10688 and four call-overwrite errors; helper-only correction still had
5120/4800/9280 and ten such errors. Both original outputs/results/artifacts at
`/tmp/piv1-keyless-sbf-build-20260909-a` and `...-b` are retained unchanged.
The report preserves exact diagnostics, runner correction and all hashes.

The separate reviewer passed the private-field boxing design and exact final
thirteen-file candidate/command. It preserves public interfaces, serialized
payloads, full state equality, error order, CEI/fresh postchecks and original audit
oracles. Private target layout assertions bound the listed normal-claim allocations
to 7390 bytes below an 8192 source ceiling; the actual 32768 heap is unchanged.
This is not measured total heap use; caller clones, invalid/oversized inputs,
error/panic/runtime costs and post-CPI allocation rollback remain explicit limits.

Third output `/tmp/piv1-keyless-sbf-build-20260909-c` is **STATIC_BUILD_PASS**:
29 zero-exit commands, no warning/error/frame diagnostics, successful output audit
and source/tool/runner/pin/protected-ref preservation. Artifact: 176064 bytes,
SHA-256 `0392bb822a3e767674ccd75486ad2685320bce5ffadb426ea8a93b08625bb6c8`.
Runner SHA-256 `e4455e55a88fe5b22796cc21ef1a938522a5cf55cc226bee3570df9756bf3cb8`;
pins `c62a97d3a2b68cbabd9c5d828b39b04ac8ea27a40ed361c0701f9db73fba8963`.
Freeze: `/tmp/piv1-keyless-boxing-source-freeze-20260909-a.json`, thirteen writer
files and 68 compiled inputs. All target tools/dependencies/profile are unchanged.

Root independently checked all 58 log hashes, raw diagnostics, ELF headers,
artifact identity and frozen inputs. ELF64LE/DYN/machine 263/flags 0, entry 0x10360,
.text 123840 bytes, four program headers/twelve sections. Disassembly byte/coverage/CALL
checks passed: 174 labels, 14973 instructions, 539 direct calls, fifteen CALL
relocations. Direct observed r10 offsets are at most 4096, not complete frame-size
proof. The partial sixteen-function syntactic path excludes ten indirect and
fourteen unresolved reachable calls; it is not a runtime call-depth bound. Ten
undefined names, including newly present sol_memmove_, need later runtime support.
Final evidence files are `/tmp/piv1-t212-pilot-final-evidence-20260909-a.json` and
`/tmp/piv1-t212-pilot-final-artifact-observations-20260909-a.json`.

Root personally executed **339 workspace tests +1 doctest and all eight final
checks/docs gates**, PASS without warnings/errors. Logs/commands/frozen source:
`/tmp/piv1-t212-pilot-20260909T132833Z`. All 77 broader source inputs were preserved;
all 68 actual compiled inputs match target pins. Initial temporary host-launcher
inventory assertion failed before Cargo because it also included nine excluded
historical spike paths; root corrected that inventory comparison while preserving
all 77 inputs. No product/test/pin change. Writer's focused 131 results remain
attributed tool-transcript evidence. Earlier root execution of all 16 reusable
Python tests applies to the byte-identical runner/tests. Full details in report.

Writer has released target slot and report ownership; no builds are running.
`review_t23_final` returned final PASS without findings after exact source/host/
target evidence and shared-doc inspection. Root committed the exact source freeze,
recorded its hash in the reviewed closure, published the clean integration sequence
and independently verified remote refs before proceeding. Targeted scan
of nineteen candidates and hooks/automation check passed. No sensitive/live action
or founder acceptance.

[Task 2.13](TASK_2_13_KEYLESS_SBF_CLAIM_EXECUTION.md) initial preparation entry
(historical; current state is at the top): **PREPARATION — NO BUILD RELEASED**.
Separate written-scope review returned PASS
without corrections for contract SHA-256
`61f2cca820533737357e4eca0321de08be71f80d5ee062bf81af92bacfc92f55`.
Writer `implement_t26_deposit` has the sole preparation/acquisition dispatch:
new isolated `validation/sbf-claims` workspace/lock/fixture/tests and optional new
bounded tool runner/pins, plus its report append. Existing production sources,
tests/manifests/lock/toolchain, SBF runner/pins and artifact must remain unchanged.
Root owns shared documents. Only public package acquisition into a fresh private
cache, metadata/build-script/tool inspection and source preparation are released.
No compilation, SBF runtime, key/signing or live action is authorized by this step.

Selected technical candidate: Mollusk 0.15.1 at revision f432ef136ee9779d2a814ebf2b80f44c10607255,
only inner-instructions/invocation-inspect-callback features; Agave runtime 4.2.0,
SBPF 0.21.1, explicit SDK versions in report. Root independently checked upstream
lock versions/checksums and installed direct host Cargo/Rust 1.97.1 identities.
Writer's source assessment and separate reviewer agree on potential V0/Rent
compatibility, but actual loading is unproven. Require exact Rent ABI/minimums,
success at 200000 CU, explicit separate payer and compiled privilege checks,
passive raw top-level observations and standard System CPI traces. A shared
success/stale/unreachable-third sequence must distinguish real execution effects
from Mollusk returning original accounts on failure; no Bank rollback/signatures
or deployment-verifier proof. New runtime fixture funding establishes one initial
audit, never a rebased existing audit. The complete scope preserves remaining gates.

Next: inspect the writer's actual resolved source/lock/feature graph and every
selected build script/native helper; compare the unchanged production dependency
closure. Separately review the frozen runner/tools/environment/exact commands
before releasing one host build, then the identified harness runtime execution.
No later tasks have started. Do not infer execution from a source assessment.

Task 2.13 preparation update: public registry acquisition and locked/offline
host-filtered metadata completed in `/tmp/piv1-t213-preparation-20260909-a`;
no compilation or runtime execution has occurred. The initial isolated lock
SHA-256 is `2562f8cc9dc74b87d51b4c162e823d620a5b8f398f82ac5d7651f9e662c5bbca`.
Root personally generated the matching production host metadata and dependency
comparison at `/tmp/piv1-t213-pilot-dependency-comparison-host-20260909-a.json`.
Separate reviewer independently confirmed all 134 selected production host
identities/checksums retained, a 138-package PIV closure, thirteen feature changes
and six additional direct edges across five shared packages. Seven unused optional
root-lock entries are absent from the isolated lock; no whole-lock/all-features
equivalence is claimed. An earlier comparison used unfiltered production metadata
and was superseded by the matching host comparison, not treated as a product defect.
The reviewer found this isolated-host divergence acceptable; final source/scripts/
tools/commands still need separate review before any build release. The root lock
and exact Task 2.12 source/artifact remain unchanged.

The initial acquisition log records a temporary HOME override and jobs=2, with
no compilation. Root corrected the preparation protocol: future commands omit
HOME and use task-specific CARGO_HOME/TMPDIR only; actual builds use one slot and
`--jobs 1`. Historical logs and the created temporary directory remain intact.
The sole writer is preparing sixteen runtime tests and the runner/pin packet;
root and reviewer are inspecting 34 build scripts across 357 reachable host
packages. These counts describe resolved source, not executed compiler units.

Pre-execution review update: root finished the two native build scripts and `cc`
selection paths; the reviewer finished the other 32 scripts and referenced
helpers. No blocking behavior was found under the fixed environment. The libc
script attempts `emcc -dumpversion` even on Linux; root verified both permitted
PATH candidates absent and the runner now guards that absence. Root independently
matched 118 final tool/library hashes and nine aliases. Two recorded `ldd` exits
of 1 concern small GNU linker scripts, whose actual shared-library targets are
pinned; they are not successful dynamic-executable probes. Writer's initial
preflight passed without compilation or SBF execution.

Root found **T213-R1 (test classification, correction in progress)**: accepting
`ProgramFailedToComplete` with any log containing `exceeded` can mistake a maximum
call-depth failure for compute exhaustion. The writer must require the specific
SBPF instruction-meter error for that branch and preserve direct
`ComputationalBudgetExceeded` separately, then refresh affected source/pin hashes
for the ongoing final separate review. No product defect or actual runtime result
is inferred. Initial preparation/preflight evidence remains intact. No build slot
has been released.

## Deferred risks and sensitive gates

- **OPEN deposit liveness:** all actual deposit fees and any integer historical
  value loss reject. Pinned SPL arithmetic can turn a zero-fee 700-lamport input
  into tokens worth 699; 707 is exact in the documented fixture. Protected
  slippage alone does not solve this. Some SOL stays queued. General fee/loss
  support needs a confirmed resolution, never an implicit subsidy/HWM exception.
- Runtime executing program ID/Rent/Clock, official pool/mint/list/source identity,
  collision-safe production snapshot identity, initialization/state writes,
  signer/destination/privileges, real transfers/CPI and SBF runtime execution remain
  unproven.
- Operational surplus has no authenticated funding baseline. Token/temporary
  native excess is unsupported; one extra token-account lamport is visible to
  base auth but blocks the economic accessor. Resolve/contain this liveness path
  before handlers. Task 2.9 now provides canonical zero-tail persistence for
  four state types; future handlers must use it with authorized transitions.
- General Idle integration must preserve pending-SOL priority and no-yield/
  insufficient behavior. Real multi-leg sizing/source order/minima/slippage,
  actual KIF runtime claims, governance and governed recovery remain separate work.
- **D-026 live-operation approval: NONE.** No new keys/signing, public-Testnet
  deployment/fund-moving lifecycle, Mainnet, real funds or authority transfer.
  Prepare the exact cluster/identities/artifact/budget/operations approval card
  before those live actions; continue independent safe engineering meanwhile.
- No Mainnet action, deployment, fund movement, key creation, signing, authority
  transfer or unrelated secret access occurred in this pilot sequence.
- Before publication the pilot found effective `core.hooksPath` unset, sample-only
  hooks, no tracked `.github`/`.cargo` automation. Recheck before publishing.
  No force push, history rewrite, automatic release/tag or unrelated publication.
