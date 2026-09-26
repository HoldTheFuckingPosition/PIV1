# Task 2.32 — Keyless full-genesis initialization runtime validation

Status: **TECHNICALLY VALIDATED / FOUNDER-AUTHORIZED MAIN INTEGRATION (D-029)**
within the bounded local-runtime scope below. Baseline integration:
`60193d63b64f42211f98d9b4910dfc047ab876df`. Separate source, actual-evidence and
final document review passed. Root owns normal publication under D-029 main
integration, not broader founder acceptance or any live operation.

## Scope and expected evidence

A new isolated Mollusk harness loads all three exact Task 2.31 artifacts without
rebuilding or editing production, probes, earlier harnesses or pinned dependencies.
The synthetic Squads caller uses real signed CPI; it is not actual Squads control.
The Token artifact has canonical ID and executes the pinned SPL Token 8 processor
only for InitializeAccount3; it is not a deployed or general Token artifact.
The System processor is Mollusk's actual pinned builtin.

The fixed 35/34-account fixture reuses the unchanged Task 2.30 input serializer.
Expected nine state envelopes, target sizes, Token bytes, rent debit, pending
prefund movement and System/Token instructions are independently encoded from
literal accepted layout rules. No production model factory or host success stub
supplies expected output. Runtime Instructions is generated from the actual full
message. Clock/Rent come from runtime sysvars. Full account bytes/lamports/owner/
executable/rent_epoch and exact nested privileges are checked.

Four intended success profiles combine distinct/shared protocol fee receivers,
fresh/mixed prefunded targets and paused/unpaused initialization. Two future Token
accounts' excess moves to PendingSol; the external payer still pays every original
rent shortfall. All initial HWM, pending, principal, yield and KIF ledgers stay zero.
A two-instruction case requires complete real initialization before a malformed
second outer action fails. Complete raw staged state and returned original state
are recorded separately. Mollusk discards its returned mutated account vector on
failure; this is not Bank/AccountsDB rollback proof. Representative early failures
cover approval, Clock, recipient rent, mint, rent funding, outer payer signature,
direct height-one invocation and outer discriminator.

Initial resource contract is default 32 KiB heap and explicit 1,400,000 CU.
Unexpected resource failure remains a failing result; no silent budget increase,
weakened oracle, initializer readiness or ordinary-budget claim is permitted.

## Preparation and execution boundary

One delegated writer owns nine new files: five workspace files, three tool/pin
files and this report. Separate
review inspects source, runtime oracles, commands, host executable and actual
evidence. Root owns the single compiler/runtime slot, seven shared documents
(including the D-029 decision record) and Git.
No writer/reviewer test, compilation or SBF execution is claimed.

The standalone lock renames only the local Task 2.30 harness package. Its 389
registry identities and direct pinned dependency edges are unchanged. One exact
locked/offline metadata call passed first attempt in 0.488128 seconds with empty
stderr and unchanged manifest/lock. Full command/environment/tool/input hashes:
`/tmp/piv1-t232-writer-preparation-20260926-a`.

```text
/home/jerem/.rustup/toolchains/1.97.1-x86_64-unknown-linux-gnu/bin/cargo metadata --manifest-path /home/jerem/piv1/validation/genesis-initialization-runtime/Cargo.toml --locked --offline --format-version 1 --filter-platform x86_64-unknown-linux-gnu
```

The guarded runner imports the unchanged claims helper only after hash checking,
without overriding its module globals. It binds complete new sources, resolved
feature graph, package archives/extracted source bytes, tools and aliases,
protected earlier inputs and all three required SBF artifacts. Every command
uses an explicit workspace/repository cwd and reviewed clean environment. Build
only produces one host executable; run requires its separately approved hash.
Before/after preservation and full log hashes survive failed stages.

The new profile binds seven source files, 143 protected earlier inputs, all
389 registry packages, 119 host tools, nine aliases, three required SBF artifacts
and six additional historical artifacts. All prior source, ABI, economics,
artifacts and pins remain unchanged. At takeover root independently verified
142 prior inputs and **326 retained logs**, including the Task 2.31 host/static
build evidence. Earlier host, Node and SBF results remain retained evidence,
not reruns in this task. Root's takeover receipt SHA-256 is
`ac380e1cd8fadb236411c538e5e3aefb5d4c8584f7478c764feab58c1e037444`.

The installed libexpat changed independently since Task 2.31. Root verified the
Ubuntu-signed cached InRelease, its Packages index, the official downloaded
`libexpat1_2.6.1-2ubuntu0.6_amd64.deb` and its payload against the installed file.
The signature and `dpkg --verify` checks passed. Separate provenance review
passed. Package SHA-256:
`494b8e672f722130c6bca6a7bc4cc31a43ca891a31d60d868bfdd699a3c20b13`.
Only this new profile binds the existing 186624-byte
`/usr/lib/x86_64-linux-gnu/libexpat.so.1.9.1` to SHA-256
`286682ecbc5e59a638963b1a4e6351e65eb32fcf4bdcb9cb7569b6a61fe06a8d`.
The other 118 tools remain unchanged. No package was installed and no old pin
was edited. This is exact package provenance, not a general host-integrity or
freshness assertion. Root receipt:
`/tmp/piv1-t232-pilot-review/system-provenance.json`, SHA-256
`b3950e8155ee28aaf405a6d5b13c92b65b12ba5f4308537c02bdcb12e72dbb17`.
Separate receipt:
`/tmp/piv1-t232-reviewer/system-provenance-review.json`, SHA-256
`c5103d7c365af449e638458678259b8772e4a2ed9e08477921feac7410801503`.

## Root mocked runner regressions

Root passed **13 mocked runner tests** on their first execution, exit zero in
0.191178 seconds, including both missing and drifted third Token artifact guards.
These regressions execute no Cargo, SBF or network command. They have not changed
through the two runtime-test-only corrections and were not rerun. Exact command:

```text
/usr/bin/python3 -I -B /home/jerem/piv1/tools/test_validate_genesis_initialization.py
```

Evidence: `/tmp/piv1-t232-pilot-review/runner-tests.json`, SHA-256
`97c3375c10c61057278e274df81b40e2ebe61070908e41445a2320b295466791`.

## Guarded command record

Every guarded command below runs with the explicit outer prefix
`env -i PATH=/usr/bin:/bin LC_ALL=C`; its runner records the complete explicit
clean tool environment. Cargo uses only the unchanged private public cache at
`/tmp/piv1-t213-preparation-20260909-a/cargo-home`. It runs locked/offline with
one job, no debug information or incremental compilation, and a fresh private
task-specific target directory. No user HOME or secrets are read or reassigned.
The immutable runner SHA-256 throughout is
`a93d385405a4f645a5459b0112c6bb40f267a5b7d6513360b235c282b1971f5d`.

All three host builds use this exact inner Cargo command, with the target directory
set by the stage's recorded environment:

```text
/home/jerem/.rustup/toolchains/1.97.1-x86_64-unknown-linux-gnu/bin/cargo test --manifest-path /home/jerem/piv1/validation/genesis-initialization-runtime/Cargo.toml --locked --offline --jobs 1 --test runtime --no-run --message-format=json
```

The executed guarded build/run commands are:

```text
/usr/bin/python3 -I -B /home/jerem/piv1/tools/validate_genesis_initialization.py --output /tmp/piv1-genesis-initialization-runtime-build-t232-20260926-a --stage build --approved-runner-sha256 a93d385405a4f645a5459b0112c6bb40f267a5b7d6513360b235c282b1971f5d --approved-pins-sha256 f6b9811b65dc7093d8329d1900ad9656021b7f4ab6bf371dcef0c4a238b4476a
/usr/bin/python3 -I -B /home/jerem/piv1/tools/validate_genesis_initialization.py --output /tmp/piv1-genesis-initialization-runtime-build-t232-20260926-b --stage build --approved-runner-sha256 a93d385405a4f645a5459b0112c6bb40f267a5b7d6513360b235c282b1971f5d --approved-pins-sha256 ca83f069c3c164691af1826cdf6dd1cb17e6d602237ae16122673dee94afb107
/usr/bin/python3 -I -B /home/jerem/piv1/tools/validate_genesis_initialization.py --output /tmp/piv1-genesis-initialization-runtime-run-t232-20260926-a --stage run --approved-runner-sha256 a93d385405a4f645a5459b0112c6bb40f267a5b7d6513360b235c282b1971f5d --approved-pins-sha256 ca83f069c3c164691af1826cdf6dd1cb17e6d602237ae16122673dee94afb107 --build-output /tmp/piv1-genesis-initialization-runtime-build-t232-20260926-b --approved-executable-sha256 c2068b528cde900fc4d8ecf6a3086161d85ea6fcde0747cb3a819d396e52fe9f
/usr/bin/python3 -I -B /home/jerem/piv1/tools/validate_genesis_initialization.py --output /tmp/piv1-genesis-initialization-runtime-build-t232-20260926-c --stage build --approved-runner-sha256 a93d385405a4f645a5459b0112c6bb40f267a5b7d6513360b235c282b1971f5d --approved-pins-sha256 ff9b72f3bad0b5e6265fe096d949a3b75bf162bffacf6811b8223e6c136c775a
```

All three builds and both runtime executions are now complete. Each guarded
stage records 12 commands and 24 full logs; preserved failures are not rewritten
as passes. The final runtime command is recorded below.

## First root build and minimal test correction

Root's first guarded host build failed before any runtime execution at
`/tmp/piv1-genesis-initialization-runtime-build-t232-20260926-a`.
Cargo returned 101 after 159.420424 seconds, with two errors
(E0609/E0277) and one deprecation warning: the new harness incorrectly accessed
the old Rent API's fields on the pinned `solana-rent` 4.3.0 runtime value.
The correction compares the complete runtime Rent to `solana_rent::Rent::default()`
and retains all sixteen independent literal `minimum_balance` assertions.
No suppression, dependency, resource contract, production or artifact changed.
Only the new runtime-source hash changes in its pin companion. The failed output
is preserved. Its `result.json` SHA-256 is
`8ce879f2c72bb03e5c0fbb2ecae138f1717253562ac57030884ce83ddbb43794`.

The fresh second build at
`/tmp/piv1-genesis-initialization-runtime-build-t232-20260926-b` passed in
168.633505 seconds with no diagnostics. Its `result.json` SHA-256 is
`67aa5fa6bb50afbf767fd2299008833b5235f41c15fd433816708e3ff8e17710`.
The 23173320-byte ELF64 little-endian x86_64 test binary is
`target/debug/deps/runtime-5344c87abc05bd9a`, SHA-256
`c2068b528cde900fc4d8ecf6a3086161d85ea6fcde0747cb3a819d396e52fe9f`.
Separate build/binary/runtime-command review passed in
`/tmp/piv1-t232-reviewer/build-runtime-command-review.json`, SHA-256
`5a383d23f5bd7489fee81f83e493f7d5d37b727d65b5eee3f509f2a84adbfded`.

## First root runtime and precise trace-index correction

After the reviewed Rent correction, the second host build passed. Root's first
exact-binary runtime run at
`/tmp/piv1-genesis-initialization-runtime-run-t232-20260926-a` returned 101 in
1.466826 seconds: **12 tests passed and one failed**. All four initialization
profiles completed at the unchanged default 32 KiB heap and 1.4m CU ceiling.
The late-error case also reached and passed its exact raw-state, returned-account
discard, resource and error-category assertions before its trace-index assertion
failed. This is a failed suite, not final validation. Exact inner execution:

```text
/tmp/piv1-genesis-initialization-runtime-build-t232-20260926-b/target/debug/deps/runtime-5344c87abc05bd9a --test-threads=1 --nocapture
```

Run-a `result.json` SHA-256:
`b9abc626ed4e5bbfd72db6c1c6833b2c7a19612dcfb9fec25c2cf277aa064591`;
stdout SHA-256:
`355ad6cf21a272754715b1ac7cdd66b249aeec60bc92c6016306f3e623a08995`.
Observed compute in this failed suite is 1063693 CU for fresh distinct receivers,
1057103 for fresh shared, 984673 for prefunded distinct, 978083 for prefunded
shared and 995443 for the late-error message. All exceed 200000 CU; no ordinary
budget, minimal heap or production-ready resource claim follows. Separate review
checked 2040 complete account records from this failed run, without a rerun;
receipt `/tmp/piv1-t232-reviewer/account-evidence-review-run-a.json`, SHA-256
`dda4abb78172761a8babe83360360641936a4141c739bad056459f71f0812f60`.

The harness incorrectly treated the raw trace as a chronological flat list.
Pinned Agave 4.2 reserves every top-level instruction slot first and appends CPI
slots after that prefix. The correction uses the actual message length as the
first CPI index, checks every reserved outer slot's exact bytes/metas/height,
and retains every callee/System/Token trace assertion. For the two-instruction
case, trace slot 1 is the rejected outer action; the callee begins at slot 2.
No state, resource, rejection or preservation oracle is weakened or changed.
The failed run and its exact executable remain preserved; a fresh separately
reviewed host rebuild and runtime retry subsequently passed as recorded below.
The corrected runtime-source SHA-256 is
`10aace52c98d1beb42e02633935616c66f849143a76916f995591283a4ceab69`;
new pins SHA-256:
`ff9b72f3bad0b5e6265fe096d949a3b75bf162bffacf6811b8223e6c136c775a`.
Separate correction/build-c review passed in
`/tmp/piv1-t232-reviewer/trace-correction-review.json`, SHA-256
`9e2f1879b9d4d1b44cbd81a1f9e7d401ec37392b4a05121d7f75bd82a875ff22`.

## Final root build and runtime evidence

The fresh third host build at
`/tmp/piv1-genesis-initialization-runtime-build-t232-20260926-c` returned
**BUILD_PASS**, Cargo exit zero in **172.577382 seconds**, without diagnostics.
Its `result.json` SHA-256 is
`2b9241ed061c963665bf11c84932ea76b77d684890796e444b10caca701a8b7e`.
The exact 23177104-byte host ELF is `target/debug/deps/runtime-5344c87abc05bd9a`,
SHA-256 `ef31a259225a4cc41de8642de1421d2c6d5609effe9dfb718287e896061b9ecf`.
Separate binary/command inspection preceded root's second runtime execution:

```text
/usr/bin/python3 -I -B /home/jerem/piv1/tools/validate_genesis_initialization.py --output /tmp/piv1-genesis-initialization-runtime-run-t232-20260926-b --stage run --approved-runner-sha256 a93d385405a4f645a5459b0112c6bb40f267a5b7d6513360b235c282b1971f5d --approved-pins-sha256 ff9b72f3bad0b5e6265fe096d949a3b75bf162bffacf6811b8223e6c136c775a --build-output /tmp/piv1-genesis-initialization-runtime-build-t232-20260926-c --approved-executable-sha256 ef31a259225a4cc41de8642de1421d2c6d5609effe9dfb718287e896061b9ecf
/tmp/piv1-genesis-initialization-runtime-build-t232-20260926-c/target/debug/deps/runtime-5344c87abc05bd9a --test-threads=1 --nocapture
```

Run-b returned **RUNTIME_TESTS_PASS**: **13 tests/cases passed**, zero failed,
ignored or filtered, exit zero in **1.490205 seconds**. Root independently
verified all **2040 complete account records** and **24 complete logs** in
`/tmp/piv1-t232-pilot-review/runtime-evidence.json`.
Run `result.json` SHA-256:
`0f680990edf2229c126cff56dfa1a90209fb7537b87e5d0649b766cf82f54627`;
stdout SHA-256:
`a3881d3aceedd88fb3aae68d3b253a8055ea8553e43eef0cea9cffd9de6a24d6`.
Separate final actual-evidence review passed, independently checking all 2040
account records, complete state/Token layouts, PDA derivations, rent/prefund and
raw-versus-returned state. Receipt:
`/tmp/piv1-t232-reviewer/runtime-evidence-review.json`, SHA-256
`482e69fce9be9148f545325565542ea1d3c63f2f0ebdd9412bfc3ee5e83e9db5`.
No additional reviewer runtime execution is claimed.

| Successful initialization profile | CU | Original payer rent debit | Token-native prefund moved to PendingSol |
| --- | ---: | ---: | ---: |
| Fresh, distinct fee receivers | 1063693 | 34779120 | 0 |
| Fresh, shared fee receiver | 1057103 | 34779120 | 0 |
| Prefunded and paused, distinct fee receivers | 984673 | 890885 | 144 |
| Prefunded and paused, shared fee receiver | 978083 | 890885 | 144 |

All custody amounts are lamports. Every profile retains the explicit 1.4m CU ceiling,
default 32768-byte heap and 4096-byte stack frames. These exact fixtures pass;
they do not establish minimal resources or ordinary 200000-CU suitability.
The complete nine initial state envelopes and both Token accounts match the
independent literal byte oracles. All sixteen final rent balances, original payer
debit, conservation, zero economic ledgers and complete protected account fields
match expectations. Exact height-three System and Token calls and their nested
privileges are checked, including PDA-only target signers and the real outer payer.

The late-error message consumes 995443 CU: actual initialization first completes,
then outer instruction 1 rejects with `InvalidInstructionData`. Both raw staged
contexts retain the fully initialized account state while Mollusk returns the
original supplied vector. Exact current Instructions indices and all outer/CPI
trace entries pass. This verifies failure propagation and Mollusk context discard;
it remains **not Bank/AccountsDB rollback evidence**. Eight other cases reject
before effects with their exact expected error categories and account preservation.

The three unchanged SBF identities loaded in every runtime case are:

| Artifact | Bytes | SHA-256 |
| --- | ---: | --- |
| Caller | 61080 | `e7fc7bf5d75a9494fd3a4787733df53adf8c3b724653a113ae36a9574707ec97` |
| Initialization callee | 376720 | `07934627a3fdab928ab1aca2abf424eb683ea9e680681389c9b53dc2391a66b7` |
| Restricted Token wrapper | 126424 | `c0f42a30da4079601711bec29bd0ca780674654ea71790eb32c76b5cedad4499` |

All belong to the unchanged Task 2.31 build directory. No SBF rebuild or production
edit occurred. The two test-only corrections changed only the new harness source
and its companion source hash. Both failed attempts and the intermediate binary
remain preserved. Root's 13 runner regressions remain valid unchanged evidence;
historical 326 logs are verified retained evidence, not reruns.

## Completion gate and limits

Root's corrected build, exact-binary runtime tests and independent account-evidence
verification passed, including separate final evidence/document review. Cumulative
source/evidence review and publication checks also passed. Root publishes main and
integration by normal fast-forward under D-029;
technical validation does not imply broader founder acceptance. Git records the
resulting commit/publication identity. Save and STOP after this bounded task;
Task 2.33 is NOT STARTED.

Failed-CPI/resource-boundary atomicity within initialization remains unproved;
the late-error case runs after successful initialization. Native initializer
exposure, actual Squads/ALT lifecycle, exclusive recipient
control, funding provenance, later Token-owned native donations and public-Testnet
readiness remain outside this bounded harness. No Mainnet action, deployment,
fund movement, key creation, signing, secrets access, authority transfer or live
blockchain operation is included. AI-assisted review is not a professional audit.
