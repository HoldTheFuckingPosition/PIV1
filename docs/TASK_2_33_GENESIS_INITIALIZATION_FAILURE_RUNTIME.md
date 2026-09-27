# Task 2.33 — Keyless in-initializer failure and retry runtime validation

Status: **TECHNICALLY VALIDATED / PENDING FOUNDER ACCEPTANCE**. Verified baseline integration and
main: `4cc4ea11e87c2f1a2f85b9ed48358f2a881821b3`. Publication is integration-only
under D-026; D-029 covered Tasks 2.24–2.32. Task 2.34 is not started.

## Bounded contract

Eight new files provide an isolated harness, its guarded runner/pins/regressions
and this report. The workspace contains `Cargo.toml`, `Cargo.lock`, `README.md`
and `tests/runtime.rs`; the three new tools are
`validate_genesis_initialization_failure.py`, its `test_` companion and
`genesis_initialization_failure_runtime_pins.json`. One delegated writer prepared
them; separate review inspects
source, commands, executable and actual evidence. Root owns shared documents,
all compiler/test/runtime execution and Git. Production, earlier harnesses,
Task 2.31 probes, economics and dependency identities remain unchanged.

The harness reuses Task 2.32's fixture and independent literal output oracles
read-only and loads the same three hash-bound Task 2.31 ELFs. The real pinned
System builtin and restricted canonical Token InitializeAccount3 artifact run
beneath the synthetic caller. This is not actual Squads or deployed Token proof.

Both 35/34-account profiles cover four fixed resource boundaries. Every failure
requires exact full raw/returned account vectors, bytes, owners, lamports,
executable/rent metadata, Instructions/Clock/Rent and exact CPI trace/privileges.
The Token-boundary expected error is `InstructionError(0, ProgramFailedToComplete)`
plus the specific originating `exceeded CUs meter at BPF instruction` log,
excluding panic/access faults that share the instruction-error category. The
200k preflight boundary instead requires exactly `ComputationalBudgetExceeded`
and the originating `Computational budget exceeded` log. Pinned Agave 4.2.0
`ComputeMeter::consume_checked` returns this InstructionError, preserved by
InvokeContext's typed syscall-error branch; a non-InstructionError SBF exception
maps to ProgramFailedToComplete. Default heap remains 32 KiB and SBF frames remain 4096.

For first/second Token failure, every System operation must already have succeeded;
payer debit and all target sizes/owners/rent are exact, all nine PIV state buffers
remain zero, and zero/one Token account has its exact initialized bytes. Paused
prefunded second-Token cases require the 144-lamport native sweep and original
890885-lamport rent debit. Preflight resource failure must have no System/Token
trace or mutation. A fresh 1.4m-CU runtime retries each actual returned-original
vector with the identical instruction, requiring all complete independent success
oracles. Eight tests comprise sixteen message cases; no host success stub runs.

## Fixed budgets and preparation

Budgets were fixed before execution using the unchanged successful
[Task 2.32](TASK_2_32_GENESIS_INITIALIZATION_RUNTIME.md) run-b logs. Each Token
InitializeAccount3 consumed 1964 CU; subtracting its logged entry balance from
1400000 gives the prefix. The failure ceiling is that prefix plus one CU, too
little to serialize a Token account. No adaptive runtime calibration is used.

| Boundary | Distinct ceiling | Shared ceiling |
| --- | ---: | ---: |
| Preflight | 200000 | 200000 |
| First Token, fresh | 953565 | 947057 |
| Second Token, fresh | 972990 | 966482 |
| Second Token, prefunded/paused | 893968 | 887460 |

The writer's only execution was one approved locked/offline metadata command,
first pass in 0.439997 seconds, exit zero and empty stderr. Manifest/lock did not
change. All 389 registry identities and dependency edges match Task 2.32; only
the local harness package name changes. Exact command/environment/input/tool
hashes and stdout/stderr are retained at
`/tmp/piv1-t233-writer-preparation-20260927-a`.

```text
/home/jerem/.rustup/toolchains/1.97.1-x86_64-unknown-linux-gnu/bin/cargo metadata --manifest-path /home/jerem/piv1/validation/genesis-initialization-failure-runtime/Cargo.toml --locked --offline --format-version 1 --filter-platform x86_64-unknown-linux-gnu
```

No writer/reviewer tests, compilation or SBF execution are claimed. The clean
explicit environment retains the existing private public Cargo cache, no HOME
assignment or credentials. All 119 host-tool hashes and nine aliases match
Task 2.32, including its previously verified libexpat; no install or pin refresh.
The new profile protects 151 earlier inputs, three executed ELFs and seven other
historical artifacts including the final Task 2.32 host executable. Six new source
inputs are bound independently from this evolving report and shared documents.
Root verified 448 retained earlier logs; earlier suites are not rerun here.

## Root execution and review

The first root mocked-runner execution had 11 passes and two failures before
any compiler/runtime execution. Python TemporaryDirectory's random suffix may
contain an underscore, while the unchanged runner accepts only lowercase letters,
digits and hyphens. Two valid-output tests therefore hit path rejection before
their intended oracle. A new test-only context manager creates an exclusive
0700 directory with a UUID hex suffix and removes only that created fixture.
All three valid-output contexts use it; an explicit underscore rejection remains
in the existing invalid-path test. The strict runner and all security oracles
are unchanged. Failed logs remain at
`/tmp/piv1-t233-pilot-review/runner-tests.{json,stdout,stderr}`. The failed
receipt SHA-256 is
`932e2a83917ef10d15e055214f287adf0d37ec36217a3525c20f61b6a42233f0`.

Task 2.32's unchanged historical guard-test fixture retains this possible
underscore-name rejection on future reruns. Its original results remain retained,
not rerun here. This is a test-fixture limitation; the production guard fails closed.

Root's corrected 13 mocked runner regressions passed in 0.139159 seconds, with
separate correction review. Evidence is
`/tmp/piv1-t233-pilot-review/runner-tests-b.{json,stdout,stderr}`; receipt SHA-256
`ade948edba5d8ca74081a4c41571f209757ca427815c0d206fae40050ab763c2`.
The runner and corrected guard-test sources did not change through the later
Rust-only correction, so these thirteen tests were not redundantly rerun. The first host
build passed in 152.588191 seconds, with 24 verified logs and no diagnostics:
`/tmp/piv1-genesis-initialization-failure-runtime-build-t233-20260927-a`.
The resulting exact host executable, separately reviewed before execution, has
SHA-256 `48d35708c52760a86fca8477e9fa45114b3a2030a79967bde26e14bcd24ee706`.
Build-a result SHA-256:
`a61cd519b554abc9deb2bdf41971c70a99667ce37660933dd15d5b466e17d6a5`.

The first runtime at
`/tmp/piv1-genesis-initialization-failure-runtime-run-t233-20260927-a`
had **six passing tests and two failures** in 1.851599 seconds, comprising fourteen
executed messages and 24 verified logs. Root verified the logs and record counts;
separate review independently decoded all 2100 complete account records and exact
traces: both 200k failures
had unchanged raw accounts, and the six Token failures/six retries matched their
expected partial/complete states. The overall runtime remains **FAILED**.
All six Token-failure/retry pairs passed. Both 200k preflight cases failed their
exact error assertion: checked syscall-meter exhaustion returns the preserved
`ComputationalBudgetExceeded`, whereas the test expected the Token VM-exception
category `ProgramFailedToComplete`. The pre-execution review had explicitly
identified this boundary uncertainty. Both 200k retries were therefore not run.
Raw account evidence was printed before assertion. Failed result SHA-256:
`1d6b52ea513c3c2588a717218a5fe4860323089344f1a8324005300436baebf7`.

The narrow new-harness correction selects exactly one expected error and one
originating failure log for Preflight, as dictated by the pinned meter/InvokeContext
source. Token expectations, all fixed budgets, account/trace/rent/prefund/retry
oracles and artifacts are unchanged. No alternatives are accepted and the failed
run is retained. The fresh second build passed in **165.934385 seconds**, with
24 verified logs and no diagnostics. Separate exact-binary review passed before
execution. Final run-b passed **eight tests / sixteen message cases** in
**1.607907 seconds**, with 24 verified logs. Root independently decoded and checked
**2400 complete account records**, exact errors/limits, all partial raw states,
unchanged protected accounts and actual returned-original retry input. Separate
actual-evidence review independently checked all 2400 records,
558 trace entries, sixteen messages and 24 logs, including exact source/binary/
pin preservation, and passed. Its receipt is
`/tmp/piv1-t233-reviewer/runtime-evidence-review.json`, SHA-256
`c14dc37f18eb26a87798d574e63b9a5baad723dca4c58acd891eb281087823e7`.
Final shared-document/publication review passed. Root publishes the reviewed
integration branch under D-026; Git records the exact commit and publication identity.

Final evidence:

- Build: `/tmp/piv1-genesis-initialization-failure-runtime-build-t233-20260927-b`;
  result SHA-256 `a4806c159faaa794ff526582b5451939ac56c33efa52c509838ab48faa675f06`.
- Exact host executable: `target/debug/deps/runtime-9169a8ae99b91d1e`, 23208208 bytes;
  SHA-256 `1e919318e405405b7f668d8bf00cc65ad40d8ff313ec7f84a060d703cec05ae6`.
- Run: `/tmp/piv1-genesis-initialization-failure-runtime-run-t233-20260927-b`;
  result SHA-256 `3dc7811de130c298dd9d4724a81da9838be28d6ea3a70dca344d580788615e93`.
- Final pins SHA-256 `fc561ee67bd5be87aa84762f7b1067289f828722b1cad77a89d53c8127ba2379`;
  immutable runner SHA-256 `47409e204d21e1b79aa0838892bd2affc3498e2b69dde9b2749721029b91f2e8`.
- Root independent inspection receipts:
  `/tmp/piv1-t233-pilot-review/build-b-evidence.json` and
  `/tmp/piv1-t233-pilot-review/runtime-evidence.json`.

Both 200k cases reject before effects. The six later failures preserve staged
System mutations and exactly zero/one initialized Token account; all nine PIV
state buffers remain zero. Prefunded failures preserve the 144-lamport sweep
and original 890885-lamport payer debit; fresh Token-CPI failures debit 34779120 lamports.
Every failure consumes exactly its fixed ceiling and every retry succeeds using
the returned originals. Retry costs are 1063693/1057103 CU for distinct/shared
fresh profiles and 984673/978083 CU for prefunded/paused profiles, under explicit
1.4m with default 32-KiB heap. No ordinary-200k completion claim follows.

## Actual root commands

The mocked guard command below was executed twice, first failing and then passing:

```text
/usr/bin/python3 -I -B /home/jerem/piv1/tools/test_validate_genesis_initialization_failure.py
```

All four guarded commands below use the exact outer environment prefix
`env -i PATH=/usr/bin:/bin LC_ALL=C`. Each stage records all twelve commands,
twenty-four full logs, before/after preservation and the explicit clean tool
environment. Each build uses locked/offline Cargo, one job, a fresh private target,
no incremental compilation and no debug information. Each runtime executes only
its separately reviewed exact executable with `--test-threads=1 --nocapture`.

```text
/usr/bin/python3 -I -B /home/jerem/piv1/tools/validate_genesis_initialization_failure.py --stage build --output /tmp/piv1-genesis-initialization-failure-runtime-build-t233-20260927-a --approved-runner-sha256 47409e204d21e1b79aa0838892bd2affc3498e2b69dde9b2749721029b91f2e8 --approved-pins-sha256 239b20ee9d41846d282b92cf236cb9ce5827ae912803fc9f5b5561ee13eb6740
/usr/bin/python3 -I -B /home/jerem/piv1/tools/validate_genesis_initialization_failure.py --stage run --output /tmp/piv1-genesis-initialization-failure-runtime-run-t233-20260927-a --build-output /tmp/piv1-genesis-initialization-failure-runtime-build-t233-20260927-a --approved-executable-sha256 48d35708c52760a86fca8477e9fa45114b3a2030a79967bde26e14bcd24ee706 --approved-runner-sha256 47409e204d21e1b79aa0838892bd2affc3498e2b69dde9b2749721029b91f2e8 --approved-pins-sha256 239b20ee9d41846d282b92cf236cb9ce5827ae912803fc9f5b5561ee13eb6740
/usr/bin/python3 -I -B /home/jerem/piv1/tools/validate_genesis_initialization_failure.py --stage build --output /tmp/piv1-genesis-initialization-failure-runtime-build-t233-20260927-b --approved-runner-sha256 47409e204d21e1b79aa0838892bd2affc3498e2b69dde9b2749721029b91f2e8 --approved-pins-sha256 fc561ee67bd5be87aa84762f7b1067289f828722b1cad77a89d53c8127ba2379
/usr/bin/python3 -I -B /home/jerem/piv1/tools/validate_genesis_initialization_failure.py --stage run --output /tmp/piv1-genesis-initialization-failure-runtime-run-t233-20260927-b --build-output /tmp/piv1-genesis-initialization-failure-runtime-build-t233-20260927-b --approved-executable-sha256 1e919318e405405b7f668d8bf00cc65ad40d8ff313ec7f84a060d703cec05ae6 --approved-runner-sha256 47409e204d21e1b79aa0838892bd2affc3498e2b69dde9b2749721029b91f2e8 --approved-pins-sha256 fc561ee67bd5be87aa84762f7b1067289f828722b1cad77a89d53c8127ba2379
```

Both builds execute this inner Cargo command with their recorded environment:

```text
/home/jerem/.rustup/toolchains/1.97.1-x86_64-unknown-linux-gnu/bin/cargo test --manifest-path /home/jerem/piv1/validation/genesis-initialization-failure-runtime/Cargo.toml --locked --offline --jobs 1 --test runtime --no-run --message-format=json
```

## Limits and security observations

Any staged partial raw state and returned originals are separate evidence:
Mollusk discards returned mutations on failure. This does not prove Bank or
AccountsDB rollback. Successful retry begins from those originals, never partial
raw state, and does not establish recovery from partially committed chain state.
These are resource-induced Token-CPI failures, not every System/Token semantic
error, heap exhaustion or every serialization boundary. Native initializer
exposure, actual Squads/ALT/control, funding provenance, later Token-owned native
donations and complete Testnet readiness remain deferred.

No Mainnet action, deployment, fund movement, secrets access, key creation,
signing, RPC/chain operation or authority transfer is authorized or performed.
Technical validation is not founder acceptance or a professional independent
audit. Root owns final review, the checkpoint and normal integration-only commit/
publication; Git records the resulting identity. Main remains at the verified
Task 2.32 baseline. Save and STOP; Task 2.34 is NOT STARTED.
