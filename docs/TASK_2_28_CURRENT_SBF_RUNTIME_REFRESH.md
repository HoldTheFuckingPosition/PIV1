# Task 2.28 — Current SBF artifact and dispatched-path runtime refresh

Status: **TECHNICALLY VALIDATED / PENDING FOUNDER ACCEPTANCE**.
Baseline: `560ca09c9c17becb79564c164e8c308b196c7cbe`,
`integration/piv1-testnet`; main remains
`7b74be4b13c019b96a0c8abcbebfbcc361d31089`.
The founder resumed bounded work under D-026 on 2026-09-20 UTC. This task
refreshes the current production artifact and the existing isolated claim/pending
runtime evidence. Root executes stages only after separate technical review.
Technical validation is not founder acceptance or authority to integrate into main.

## Scope and unchanged boundaries

The initial refresh exposed actual oversized genesis stack frames. Root extended
this bounded task to a private observation-layout correction and its regression;
native ABI, dependency versions, runtime features and runtime fixtures remain
unchanged. The existing runners retain their staged separation:
target compilation and static inspection; host harness compilation without test
execution; then direct execution of the separately reviewed host binary on the
separately reviewed SBF artifact. Commands use fresh private `/tmp` directories,
locked/offline Cargo and one compiler job. No dependency installation or upgrade
is performed. The provenance-only public OS package downloads are recorded below.

Only the currently dispatched isolated `claim_kif` and pending-contribution
recognition paths can gain refreshed runtime evidence. Compiling the full current
crate does not execute or establish runtime readiness of unreachable genesis,
Squads governance or Jito library paths. **24 tests /70 cases** are the existing
Task 2.14 expectations at preparation; the current actual results are recorded
below. Synthetic signer privileges and
fixture custody remain distinct from actual signatures, live initialization and
continuous economic history. Mollusk output discard remains distinct from
Bank/AccountsDB rollback. No new initializer is exposed.

## Pin preparation and detected host drift

Writer independently verified `jerem`, the clean integration branch and exact
baseline HEAD before edits. The target runner's source inventory grows from
**73 to 92 files**: nineteen additions and six updated hashes, with no removals.
These additions are the already-reviewed Squads, Jito identity and genesis
modules/tests. Updated entries are the root lock, program manifest,
`guardian_clock_accounts.rs`, `instructions/initialize.rs`, `integrations/mod.rs`
and `lib.rs`. This task does not alter those six sources; the subsequent stack
correction updates only `src/genesis_preflight.rs` and its existing test file.
The program manifest's
intervening dependency additions are test-only oracles; production dependencies
remain unchanged.

The target pins update `baseline_head` and `source_files`, setting the former to
this task's exact baseline, plus the five verified OS hashes described below.
Original inventory provenance, compiler pins, build-script inventory,
`reviewed_tools`, resolved paths and library file sets remain unchanged.
Target runner SHA-256 remains
`e4455e55a88fe5b22796cc21ef1a938522a5cf55cc226bee3570df9756bf3cb8`.

The harness source inventory currently grows from **85 to 105 files**: the same
nineteen additions plus the extracted oracle's LICENSE. It also includes the
root-owned program and harness READMEs and target pin companion, so their final
freeze must precede harness pinning and build. The harness's fixed `baseline_head`
**`fd48c3b1644faed6d30fdb92774c1bd8c03a2658`** remains unchanged because its runner
checks that historical closure explicitly. A read-only ancestry check confirms
that closure is an ancestor of the current baseline. Current-source provenance
comes from the exact refreshed source inventory and this report; the historical
guard is not weakened. Harness runner SHA-256 remains
`ff4706b6101b57c6385b0ae43b175a3ed252cb40e150f7a3120a84beb7c01fc8`.

Read-only inspection found **no missing file among 646 known pinned tools,
support/build-script files, registry archives and historical artifacts**, and all
389 pinned extracted-package directories exist. Archive hashes matched. At that
preparation point, full extracted-file and actual resolved-feature checks still
required the later runner stages; those stages passed as recorded below.

However, **thirteen system path hashes differ** from the historical pin
companions: five target paths and eight harness paths, including aliases of the
same underlying system libraries. They cover the glibc loader/libraries,
`/usr/bin/ldd`, `/usr/bin/python3` and `/usr/bin/python3.12`. No target
`reviewed_tools` entry changed. The exact expected/actual hashes and resolved paths
are saved in `system-pin-drift.json`. Existing guards correctly identify these
incompatible historical hashes before any runner execution.

Root then verified the cached signed Ubuntu InRelease metadata, the exact plain
Packages indexes, and each package's version, amd64 architecture, size and SHA-256.
The three official public packages were downloaded for byte comparison only:
`libc6` and `libc-bin` **2.39-0ubuntu8.9**, and `python3.12-minimal`
**3.12.3-1ubuntu0.17**. Every changed installed path matches its signed-index-bound
package payload: thirteen path spellings, nine unique file contents. No package
was installed. Root's receipt is
`/tmp/piv1-t228-pilot-review/system-provenance.json`, with package files and
signature logs retained alongside it. Writer inspected that receipt and verified
the matching installed bytes before applying root's explicitly authorized narrow
correction: **five `files[*].sha256` values in target pins and eight `tools`
values in harness pins**. Every other tool/package/feature pin is unchanged.
This proves the named files' package provenance, not complete host integrity or
a supply-chain audit. Neither runner nor protection was weakened.

Root's initial provenance reader assumed compressed `.lz4` indexes, while this
host stores plain indexes. The reader was corrected before any runner, build or
test execution. This is a retained inspection diagnostic, not a failed target
build, preflight or test.

Both historical SBF artifacts remain present and hash-identical: Task 2.12's
176064-byte ELF at its protected path, SHA-256
`0392bb822a3e767674ccd75486ad2685320bce5ffadb426ea8a93b08625bb6c8`, and Task 2.14's
229208-byte ELF, SHA-256
`46fd815847c236fb53ed5dc5ace79c48c5a21beac4019ffa107b80a5be69812f`.
The harness artifact identity still names the historical Task 2.14 ELF at this
stage. It will change only after root produces and the reviewer verifies the
new artifact's exact path/hash/size.

## Evidence retained at initial preparation

Writer checked all **96 retained input hashes**: 92 Rust inputs, two Node scripts
and two spike manifests. All match Task 2.27's recorded freeze. Writer also
verified **28 retained log hashes**: sixteen Task 2.26 Rust gate logs and twelve
Task 2.27 Node logs from writer/root executions. Thus **468 host tests +1 doctest /
eight gates**, and **15 Node tests plus eight historical/sixteen recipient report
cases**, remain attributed prior executions at that preparation checkpoint.
The later correction changes two Rust inputs, so the 468-test host result is no
longer claimed for the final source; root must rerun the full host gates. Neither
Node script, its four directly pinned Rust inputs nor the two spike manifests
change. Node evidence may be retained after verification of those actual inputs;
the original combined 96-file inventory must not be described as unchanged.

Preparation evidence is in `/tmp/piv1-t228-writer-preparation-20260920-a`:
original pin snapshots, current source inventories, source diff lists, known-file
inspection, per-companion system drift and retained-log verification. The root
Rust `results.json` hash remains
`2a056c41c49880dce78710b766e3409e051ba523ceff204678196e4da570b260`;
the root Node `results.json` hash remains
`993a2d3fa5210e6cd0a7758a7c8659b2d530890f590e84108acaaf65992e2d9e`.

## Reviewed stages before the stack correction

The first reviewed target pin SHA-256 was
`3de7a770d48780e7d5c4118de6b96f6a582963fdad2aec4bb3bd0dffc48a523e`.
The harness source/OS draft is
`e21de214f735e714476c17f11d9e8e8f209532db58c6e4ce309daa0192dc04f4`;
its artifact still names Task 2.14 and therefore is not released as new-artifact
evidence. Only `sources` and the eight authorized `tools` entries changed at this
stage; its historical baseline, artifact, aliases, packages and feature graph are
unchanged. Root's two README inputs are frozen within its 105-source inventory.

Separate preparation review passed the initial target freeze. Root executed the
first command below successfully; the second completed with rejected compiler
diagnostics as recorded below:

```text
/usr/bin/env -i PATH=/usr/bin:/bin LC_ALL=C /usr/bin/python3 -I /home/jerem/piv1/tools/build_keyless_sbf.py --output /tmp/piv1-keyless-sbf-preflight-t228-20260920-a
/usr/bin/env -i PATH=/usr/bin:/bin LC_ALL=C /usr/bin/python3 -I /home/jerem/piv1/tools/build_keyless_sbf.py --output /tmp/piv1-keyless-sbf-build-t228-20260920-a --execute --approved-runner-sha256 e4455e55a88fe5b22796cc21ef1a938522a5cf55cc226bee3570df9756bf3cb8 --approved-pins-sha256 3de7a770d48780e7d5c4118de6b96f6a582963fdad2aec4bb3bd0dffc48a523e
```

The target preflight returned **PREFLIGHT_PASS_NO_TARGET_BUILD** on its first
execution: all **26 commands exited zero**, with **52 recorded logs** and passed
source/tool/runner/pin and protected-Git-ref preservation. The actual record is
`/tmp/piv1-keyless-sbf-preflight-t228-20260920-a/result.json`, SHA-256
`79bdd7d457d0771366c17604d00b30eee6f99fd139a620d3696c5c1a8b42d703`.
This preflight did not compile the target.

After the verified Python refresh, root also executed the unchanged runner
regressions, **16 target-runner tests +7 harness-runner tests PASS** on the first
attempt, in command times **0.275696109 seconds** and **0.108952154 seconds**:

```text
/usr/bin/python3 -I -B tools/test_build_keyless_sbf.py
/usr/bin/python3 -I -B tools/test_validate_sbf_claims.py
```

Exact commands, logs and results are retained at
`/tmp/piv1-t228-runner-tests-20260920-a`. These are runner-protection tests,
separate from PIV1 host or SBF runtime tests. Normal unittest output uses stderr;
both suites report OK without a failed test. Writer inspected these root records
without executing them. Root separately verified the 96 retained inputs and
22 retained root logs (sixteen Rust and six Node); this is distinct from the
writer's 28-log verification above, which also includes prior writer Node logs.

The first target compilation in `/tmp/piv1-keyless-sbf-build-t228-20260920-a`
completed in 4 minutes 55 seconds. Cargo returned zero, but the strict runner
returned **FAIL** because it captured **243 stack diagnostics**. Seven frames
exceeded the 4096-byte target limit: allocation `execute` at 7552 bytes (three
monomorphizations), initialization `execute` at 7104 bytes (two), account preflight
at 4544 bytes, and recipient preflight at 8960 bytes. The remaining diagnostics
describe stack-access/frame-call overlap. Source, tools, runner, pins and Git
preservation checks passed. No artifact from that run was accepted. All original
logs and output remain intact; neither runtime harness stage had executed.

## Reviewed private layout correction and focused regression

After the founder resumed on 2026-09-21 UTC, root verified that the interrupted
writer had not applied Rust edits. A new delegated writer and separate reviewer
resumed the existing finding. `GenesisAccountPreflight` now owns its private
`ApprovedGenesisModel` and `AuthenticatedJitoIdentity` through two boxes. The
shared dispatcher is explicitly non-inlined so these large construction
temporaries do not merge into each caller. Public getter signatures, private
ownership, non-Clone behavior, validation/error order, single trusted observation,
account bytes and all custody/CPI/economic behavior remain unchanged. No detached
proof constructor or native initializer is added; no diagnostic gate is weakened.

A compile-time bound checks that the preflight observation occupies at most 64
inline bytes on the actual target. One new host regression also bounds the nested
Result and recipient/allocation/initialization wrappers. These bounds protect
aggregate movement; complete compiler frames required the subsequent strict SBF
retry. Measured pinned host sizes are respectively **32, 32, 208, 504 and 504
bytes**. The two new heap requests per successful preflight are **816 bytes for
the model and 1456 for protocol identity**, both with alignment eight: **2272
requested bytes cumulatively**, plus up to fourteen bytes of per-request alignment
padding. Existing nested model/target allocations are additional. A later failing
path retains every request already made because the SBF bump allocator does not
reclaim deallocations during the invocation. Repeated preflights accumulate these
requests. This is a host measurement and source inventory, **not proof of genesis
total heap sufficiency, execution, resource limits or rollback**.

Writer executed exactly one focused locked/offline command with pinned host Rust
1.97.1 on the corrected source:

```text
/home/jerem/.rustup/toolchains/1.97.1-x86_64-unknown-linux-gnu/bin/cargo test -p piv1 --test genesis_preflight --test genesis_allocation --test genesis_initialization --test genesis_recipients --locked --offline --jobs 1 -- --nocapture
```

**53 tests PASS** on the first focused attempt: 11 preflight, 12 allocation,
22 initialization and eight recipient tests, with no warning/error diagnostics.
Elapsed command time was 13.957841 seconds. The exact clean environment, command,
three verified tool hashes, all 92 before/after source hashes and both log hashes
are retained in `/tmp/piv1-t228-writer-focused-20260921-a`. All source hashes were
stable through execution. Separate code/test review passed; root's full host and
target validation are recorded separately below.

## Root host validation and retained Node evidence

Root independently executed **469 host tests +1 doctest /eight gates PASS** on
the first private-boxing freeze, all on its first execution. There were zero failures,
ignored tests or warning/error diagnostics. The gates cover workspace/all-target
tests, doctests, default and all-feature checks, explicit `no-entrypoint`, `cpi`
and `idl-build` checks, and warnings-denied documentation. All commands use the
same pinned Rust 1.97.1 tools, `--locked --offline --jobs 1`; exact arguments and
clean environment are retained in
`/tmp/piv1-t228-pilot-host-20260921-a`. Combined command time was 57.134969 seconds.

All **92 Rust input hashes** match root inspection and the writer's focused run,
and stayed unchanged through root validation. Root verified sixteen gate logs,
both writer logs and all three pinned host tools. Writer independently rechecked
those 92 current hashes, sixteen root log hashes and three tools without rerunning
commands. Root's `results.json` SHA-256 is
`87f3d78b7b324b3503d9f16e17fd2ac22aafab78d95dad5efdfbe1a8d2e6a2c2`;
`pilot-summary.json` records the full counts, preservation and retained evidence.
The only changed Rust inputs are the two preflight source/test files above.

Root also verified the **eight concrete Node transport inputs**: the two scripts,
two spike manifests, and the four Rust files pinned by the transport oracle
(`instructions/initialize.rs`, `tests/genesis_initialization.rs`,
`tests/support/squads_invocation.rs`, and `genesis_recipients.rs`). All remain
byte-identical to Task 2.27, with matching tool hashes and six retained root logs.
Writer separately rechecked those eight current source hashes. Thus **15 Node
tests plus eight historical-profile and sixteen recipient-profile cases remain
retained Task 2.27 evidence, not rerun**. Their root `results.json` SHA-256 remains
`993a2d3fa5210e6cd0a7758a7c8659b2d530890f590e84108acaaf65992e2d9e`.
The original broader 96-file snapshot has two changed Rust entries and is not
claimed unchanged.

## Target retries and final artifact

The first private-boxing target pin SHA-256 was
`2ad7d636cc02ece1871a495c910f3183abb0ce9a212548463e350e5a2ca3f696`.
Only the two corrected source entries changed from the previously reviewed target
pins; all tool provenance and baseline guards remain. Its production file SHA-256 was
`cbb29b6c176d31e403e35fdfb169094f501f6cf323758de62ca22bd91576c6bf`;
the regression file is
`7ea38ce10baeb6c719e8fa6472555d646f5d54a345d3a4c3f4cf97d5f34ec76b`.
Separate source and retry-command reviews passed; their receipts are
`/tmp/piv1-t228-pilot-review/stack-source-review.json` and
`target-retry-review.json`. Root executed the strict target retry at the fresh
`/tmp/piv1-keyless-sbf-build-t228-20260921-b` path. Its command is retained in
`target-retry-command.json`. The retry again returned **FAIL** despite Cargo exit
zero: **45 diagnostics** identified three remaining oversized frames, all
`genesis_preflight::dispatch` monomorphizations, at **5696, 5696 and 5824 bytes**.
The original allocation, initialization and recipient-preflight frame diagnostics
were gone. The strict gate remained unchanged; the second output is preserved
and no artifact from it is accepted.

Root authorized a second minimal correction in the same production file: two
private non-inlined construction helpers isolate the large model and protocol
`Result`/`Box` temporaries from the shared dispatcher. They add **no further heap
requests**; ordering remains model, retained Rent, role validation, protocol
authentication and target observation. The earlier 53-test and root 469-test host
passes above belong to the first correction. Fresh focused/full-host validation
and a third strict target attempt were required on the final helper source.

The second source correction passed separate source review. Writer reran the
same focused command once on the helper freeze: **53 tests PASS**, zero failures,
ignored tests or diagnostics, in **10.553025 seconds**. The retained record is
`/tmp/piv1-t228-writer-focused-20260921-b/result.json`, SHA-256
`0ee7d6fd2900c0e7d8287c0b9a5d6265459d638c2915ec2b3b9ac8706378e600`.
All 92 inputs stayed unchanged; aggregate sizes and the two cumulative heap
requests are identical to the earlier observation. Both focused runs passed on
their first executions; they belong to distinct source freezes, not a hidden
failed-test retry. The strict target failures above remain recorded separately.

Final helper-source SHA-256 is
`478c8fc69f409b911478ff07592eb8ec95f31b197b132971134eab6df579da4c`;
the regression file remains
`7ea38ce10baeb6c719e8fa6472555d646f5d54a345d3a4c3f4cf97d5f34ec76b`.
Only that production entry changed in the target source companion for the third
attempt. Its current SHA-256 is
`8bbfca7d66c6612ee970ea6e1058b5dd9efd152aadd4f6c4af86348767238cd5`.

Root reran all eight host gates on this final helper source: **469 tests +1
doctest PASS**, zero failures, ignored tests or diagnostics, in **64.776158
seconds**. Each gate passed on its first execution for this freeze. Evidence is
`/tmp/piv1-t228-pilot-host-20260921-b`; `results.json` SHA-256 is
`540fc8319d50b851f737613ab3bc509f4b788ec27c7779a577da6fdec5228969`.
Root checked all 92 source hashes against inspection and the final writer run,
sixteen gate logs, two focused logs and three host tools. Writer independently
rechecked current sources, sixteen root logs and three tools. The same eight
transport inputs and retained Node evidence remain unchanged, with no Node rerun.

Root's third strict target attempt at
`/tmp/piv1-keyless-sbf-build-t228-20260921-c` returned **STATIC_BUILD_PASS**:
Cargo exit zero, **zero target diagnostics**, **29 zero-exit commands and 58
recorded logs**, with source/tool/runner/pin and protected-Git-ref preservation.
The target build command took **292.493958 seconds**. Its `result.json` SHA-256
is `86f0410981f07ee75d22f9fbb7153d923a7822447c3e571eba03557815ddfd68`.
This successful attempt does not erase either earlier rejected build.

The produced artifact is
`/tmp/piv1-keyless-sbf-build-t228-20260921-c/target/sbpf-solana-solana/release/piv1.so`,
**229888 bytes**, SHA-256
`0eb5e62389c9baa5311fddca99d1e705f86b1fd698e869a8cdcec778aa68cd54`.
The runner inspected little-endian ELF64 structure, machine 263, executable
entrypoint and static symbol/relocation records. Separate artifact review passed;
the receipt is `/tmp/piv1-t228-pilot-review/artifact-review.json`. The reviewer
bound **21056 disassembled instructions to all 173296 text bytes** and checked
**4803 direct frame memory accesses**, including access widths, within offsets
**-4096 through -1**. The ten unresolved imports match the earlier artifact.
Root released this exact artifact for harness binding. Compilation and static
structure are not loader,
syscall, total heap/compute, genesis execution or rollback evidence.

After root froze both dependent READMEs, writer refreshed the harness's same
105-source inventory; only the two corrected Rust hashes and final target-pin
hash changed from the prepared draft. After root's release, writer independently
rechecked the artifact bytes/hash and all 105 source hashes, then bound only the
artifact's exact path, size and SHA-256 above. All 389 packages, feature graph,
historical baseline, aliases, tools and runner remain unchanged from the reviewed
preparation. The frozen harness pin SHA-256 is
`ff0a1f94d409173f7ecafd512fc1fabbbf13ea60b165d941983997bd77bb4127`.
The separately reviewed build and runtime stages are recorded next.
The writer's first binding inspection expected a shortened review-status string;
that assertion stopped before pin mutation. Reading the receipt's actual
`PASS_STATIC_ARTIFACT_REVIEW_FOR_HARNESS_BINDING` status resolved the mismatch.
This was a local inspection diagnostic, not a build or runtime failure.
Hash arguments bind separate review;
they do not authorize live operations.

## Current harness build and runtime execution

Separate preparation review approved the exact harness build command and its
105 sources, 119 tools, nine aliases and 389 unchanged package records. The
runner verified extracted package files and the exact resolved feature graph
before compiling with pinned Rust 1.97.1, locked/offline Cargo and one job.
Root's first build returned **BUILD_PASS**, twelve zero-exit commands and 24
verified logs, with no warning/error diagnostics. The `--no-run` compilation
took **182.811617 seconds** and did not execute a test. Its record is
`/tmp/piv1-sbf-claims-build-t228-20260921-a/result.json`, SHA-256
`961bad892f59cfe42229ae0c81034efb22d24e4591616c5430a185864b3306cc`.

The newly compiled executable is
`/tmp/piv1-sbf-claims-build-t228-20260921-a/target/debug/deps/claims-c12d45bbd63e9bf4`,
**98226152 bytes**, SHA-256
`89d388ffe8a60382694d0d87e415901d9f6ff6df737f538fe2f372f4f48572d0`.
Root and separate reviewer inspected ELF64/x86-64 structure, interpreter and
dynamic libraries: four canonical pinned libraries, no RPATH or RUNPATH.
Root's first inspection parser counted five path spellings, including a loader
alias; canonical resolution correctly yields four libraries. Original and v2
inspection logs remain retained. This parser correction was not a build or test
failure. Build, executable and exact runtime-command reviews are recorded in
`harness-build-review.json`, `harness-executable-review.json` and
`harness-run-review.json` under `/tmp/piv1-t228-pilot-review`.

The exact successful stage commands used a clean outer environment:

```text
/usr/bin/env -i PATH=/usr/bin:/bin LC_ALL=C /usr/bin/python3 -I /home/jerem/piv1/tools/build_keyless_sbf.py --output /tmp/piv1-keyless-sbf-build-t228-20260921-c --execute --approved-runner-sha256 e4455e55a88fe5b22796cc21ef1a938522a5cf55cc226bee3570df9756bf3cb8 --approved-pins-sha256 8bbfca7d66c6612ee970ea6e1058b5dd9efd152aadd4f6c4af86348767238cd5
/usr/bin/env -i PATH=/usr/bin:/bin LC_ALL=C /usr/bin/python3 -I /home/jerem/piv1/tools/validate_sbf_claims.py --output /tmp/piv1-sbf-claims-build-t228-20260921-a --stage build --approved-runner-sha256 ff4706b6101b57c6385b0ae43b175a3ed252cb40e150f7a3120a84beb7c01fc8 --approved-pins-sha256 ff0a1f94d409173f7ecafd512fc1fabbbf13ea60b165d941983997bd77bb4127
/usr/bin/env -i PATH=/usr/bin:/bin LC_ALL=C /usr/bin/python3 -I /home/jerem/piv1/tools/validate_sbf_claims.py --output /tmp/piv1-sbf-claims-run-t228-20260921-a --stage run --approved-runner-sha256 ff4706b6101b57c6385b0ae43b175a3ed252cb40e150f7a3120a84beb7c01fc8 --approved-pins-sha256 ff0a1f94d409173f7ecafd512fc1fabbbf13ea60b165d941983997bd77bb4127 --build-output /tmp/piv1-sbf-claims-build-t228-20260921-a --approved-executable-sha256 89d388ffe8a60382694d0d87e415901d9f6ff6df737f538fe2f372f4f48572d0
```

Root's first runtime execution returned **RUNTIME_TESTS_PASS: 24 tests, 70
message cases, zero failures or ignored tests**, in **2.820003 seconds**. Its
record is `/tmp/piv1-sbf-claims-run-t228-20260921-a/result.json`, SHA-256
`c8a2eb3359609fb73bb3ce19e91f6ccedd16d6fb10da06e848e5dba85549ad9f`.
Root verified 24 runtime logs and preservation of all 105 sources, 389 packages,
the executable, artifact and exact harness pins. Writer independently verified
all 48 build/runtime log hashes, current 105 source hashes and both binary hashes
without executing the binaries.

Root's separate complete-account oracle passed **1629 account records across
all 70 cases**, including six successful claim messages and ten pending cases.
It compares complete raw and returned account states, balances and serialized
ledgers, including failure/discard distinctions. Its receipt is
`/tmp/piv1-t228-pilot-review/runtime-account-evidence.json`, SHA-256
`70e971e2e0a45f03c314b399bd98f715f3f5a9eb3a79f51d84743aa9c5d05c6b`.
`runtime-evidence.json` binds the current artifact/executable and actual results.

Selected observed compute units are **73837** for an ordinary exact-backed
claim; **152600 of the fixture's 200000-unit budget** for native donation plus
recognition and repeat; **76228** for paused pending recognition with Token-native
excess; and **77560** for successful pending recognition followed by a failing
instruction. These are measurements of the named fixtures on this artifact,
not budgets for undispatched genesis or a complete economic lifecycle. Later
Token-owned native excess remains unclassified and untouched. Mollusk returns
the original vector on the tested failed messages; raw intermediate mutations
are recorded, and this output-discard behavior is not Bank/AccountsDB rollback.

## Files, review and completion boundary

The bounded task changes thirteen repository files:

- `programs/piv1/src/genesis_preflight.rs` and
  `programs/piv1/tests/genesis_preflight.rs`: private layout/construction correction
  and its footprint regression.
- `tools/keyless_sbf_pins.json` and `tools/sbf_claims_pins.json`: current source and
  artifact binding, plus narrowly verified installed OS hashes.
- `AGENTS.md`, `README.md`, `docs/PIV1_MASTER_SPEC.md`,
  `docs/PIV1_CODEX_EXECUTION_PLAN.md`, `docs/PIV1_TEST_PLAN.md`,
  `docs/PIV1_PILOT_STATE.md`, `programs/piv1/README.md`,
  `validation/sbf-claims/README.md` and this report: durable state and evidence.

One delegated writer produced the correction and focused tests; a separate
reviewer inspected source/tests, target input commands, artifact and harness
execution boundaries. Root independently ran full host, target and harness gates,
inspected evidence and coordinated corrections. Separate runtime and report review
passed. The reviewer reused the inspected offline complete-account byte verifier
without executing the runtime tests or claiming a distinct independent oracle.
Review receipts include `stack-source-review-v2.json`, `runtime-review.json` and
the final thirteen-file freeze recorded by `final-review.json`, all under
`/tmp/piv1-t228-pilot-review`. The runtime-review receipt discloses corrected
review-only assertions for the recorded status enum and expected DEBUG stderr;
these were inspection diagnostics, not test failures. Root gates publication on
separate final documentation review. This AI-assisted review is not a professional
independent audit. Both rejected target attempts, intermediate passing host
freezes and local inspection diagnostics remain disclosed above.

The current artifact has refreshed runtime proof for only its dispatched claim
and pending paths. Genesis remains an undispatched library: successful compilation
does not establish its cumulative heap/compute fit, CPI lifecycle or rollback.
Operational funding provenance, later Token-native donation handling, full
recipient control, native initializer exposure and actual ALT/buffer/Squads
lifecycle evidence remain deferred. Packet fit, synthetic signer privileges and
these runtime fixtures do not establish founder Testnet readiness.

At the validation checkpoint, Git remains on `integration/piv1-testnet` at
baseline `560ca09c9c17becb79564c164e8c308b196c7cbe`, with only these thirteen task
files modified/new. Root owns the final reviewed commit and ordinary integration-
only publication under D-026; Git records the resulting commit and publication
identity. Main remains `7b74be4b13c019b96a0c8abcbebfbcc361d31089`; no additional
main integration or founder acceptance is inferred. Verify actual refs and clean
worktree after publication and on takeover.

Writer owns the two-file correction, two pin companions and this report. Root owns
shared docs, full validation, all target/harness executions, checkpoint and normal
integration publication. No Mainnet action, keys/signing, secrets, RPC, chain operation,
deployment, fund movement or authority transfer occurred.
Task 2.29 is NOT STARTED; checkpoint and STOP after this bounded task.
