# Task 2.34 — Read-only Bank runtime preparation prerequisites

Status: **TECHNICALLY VALIDATED / PENDING FOUNDER ACCEPTANCE**.
Verified integration baseline:
`0f27932e434441f79e17f85453f5b38ccc076f1e`; main remains
`4cc4ea11e87c2f1a2f85b9ed48358f2a881821b3`. D-026 covers bounded technical work
and reviewed integration publication. This task creates no Bank runtime evidence,
dependency authorization, new pin or main integration authority.

## Finding and bounded change

The approved public cache contains modular Agave 4.2.0 runtime components but
does not contain source directories or crate archives for `solana-runtime`,
`solana-accounts-db` or `solana-svm`. The ordinary public Cargo cache also lacked
these packages during read-only investigation. Approximately 3.1 GiB was available
on the shared repository/output filesystem; this is an observation, not a measured
Bank build requirement. A complete resolved Bank dependency closure is unavailable.

The existing pinned Mollusk 0.15.1 explicitly excludes Bank/AccountsDB (`src/lib.rs`
lines 16–18). Its transaction processing returns supplied originals on error
(lines 1218–1222); `MolluskContext::consume_transaction_result` stores accounts
only after success (lines 1725–1731). Its `src/account_store.rs` defines a store
trait and HashMap implementation. Wrapping that store would not establish genuine
Bank commit/rollback. Existing `solana-svm-transaction` traits similarly do not
provide a Bank transaction processor or persistence layer.

Three new files provide a standard-library-only metadata checker, regression
tests and this report. One delegated writer prepares them; root owns tests,
the actual environment observation, shared documentation and Git. Separate review
checks source, regressions, actual results and limitations. No old source, probe,
harness, dependency, economic rule or evidence is modified.

`tools/check_bank_runtime_prerequisites.py` checks only six exact public
source/archive paths for those three direct candidate packages, plus the existing
modular `solana-program-runtime` source directory. The fixed version **4.2.0**
aligns with the existing modular runtime; it remains an unauthenticated candidate,
not a verified available/resolved Bank release. Each path component is checked
with `lstat`; symlinks, wrong types and unavailable metadata fail closed. The
checker never reads package contents, configuration, credentials or arbitrary
directories and never runs commands, downloads, compiles, installs or deletes.

An explicit positive decimal `--minimum-free-bytes` is mandatory. The caller
selects this planning minimum; it is not a performance estimate or Bank build
measurement. Available bytes are exactly `statvfs.f_bavail * statvfs.f_frsize`
on the supplied existing real `--output-parent`. Output is deterministic structured
JSON containing paths, metadata observations, available bytes, the selected
minimum, missing/unsafe reasons and explicit limitations. Exit 0 means only the
local direct prerequisites are present and the selected minimum is met; exit 2
means not ready. Argument usage errors print to stderr and exit 64, separately
from a completed not-ready observation. A pass executes nothing and authorizes
nothing. The metadata observation is not an atomic filesystem snapshot or an
execution guard against concurrent replacement.

## Verification record

Writer verification is read-only source/cache investigation; no writer tests,
build, metadata resolution, runtime, installation, network or Git mutation ran.
Root executed **19 focused Python regressions**, all passing on the first attempt:
process duration **0.166396 seconds** (unittest reported 0.041 seconds), exit zero.
No test failed and no correction or retry was needed. Both commands used an
isolated environment containing only `PATH=/usr/bin:/bin` and `LC_ALL=C`,
with stdin closed and a 60-second timeout. Exact command:

```text
/usr/bin/python3 -I -B /home/jerem/piv1/tools/test_check_bank_runtime_prerequisites.py
```

The actual read-only environment observation then returned the expected **exit 2 /
NOT_READY** in **0.064602 seconds**, with empty stderr. All six direct candidate
source/archive paths were missing; the modular runtime source directory was
present. Available space was **3301838848 bytes**, below the caller-selected
**8589934592-byte (8-GiB) planning minimum**. This threshold is not a measured Bank
requirement or assurance that a future Bank build fits. The expected NOT_READY
result is a successful checker observation, not a failed test or Bank execution.
Exact command:

```text
/usr/bin/python3 -I -B /home/jerem/piv1/tools/check_bank_runtime_prerequisites.py --output-parent /tmp --minimum-free-bytes 8589934592
```

The executed source identities remain unchanged:

- Checker SHA-256 `fef97ec18eeb162991e140781550a91124b88429a5054d941091ac4049018f0e`.
- Regression source SHA-256 `fa578855d96d0ce8f4abb80b45331d19c98af2e25418729a344cd9d67158c40e`.

Complete commands, timing, output hashes and input binding are retained under
`/tmp/piv1-t234-pilot-review`:

- `checker-tests.json`: SHA-256 `93352af24307cbd31ad0acc111f9eb02e0a5a8ee4260f2e6d96514ee454fe79a`.
- `actual-prerequisites.json`: SHA-256 `729836eac7cf6d76d3bc9fd6e289263119c3fa00c0e6fd091af651bec5b05d0a`.
- `validation.json`: SHA-256 `59ff08cea5be3081e5af258047a263a203bc6f3aa309bc8a487797e22e24992a`.

Each command retains complete `.stdout` and `.stderr` files. No Rust compilation,
SBF runtime or Bank transaction was executed. Separate source, command and actual
evidence review passed. Its receipt is
`/tmp/piv1-t234-reviewer/source-evidence-review.json`, SHA-256
`d0774529d0171a71864f581cb6d428866161d8be97ff5dea6f7b2270ed734918`.
The reviewer independently checked all 19 passing regressions, four output-log
hashes, source freeze and the complete actual NOT_READY report. Final document
and publication review are required before root publishes the resulting commit.

The regressions cover synthetic success, the exact selected-space boundary,
available-versus-free blocks, missing source/archive/cache/modular components,
symlinked archives/sources/cache/parents/ancestors, wrong file types, unavailable
or invalid metadata, path traversal, malformed/absent budgets, distinct usage
errors and structured CLI success/not-ready output. Synthetic marker files are
deliberately not authenticated crates; a fixture pass makes no Bank claim.

Root's takeover independently matched six current and 151 protected inputs,
119 tools, nine aliases, ten pinned artifacts and the final Task 2.33 host binary.
It reverified 100 Task 2.33 logs including both failed attempts. No earlier test
suite was rerun. The older 448-log verification chain remains historical evidence;
those older logs were not rehashed during this task. Root receipts are under
`/tmp/piv1-t234-pilot-review`.

## Next dependency and limits

Before a genuine Bank harness, separately inventory reclaimable compiler
intermediates while preserving receipts, logs, exact artifacts and executable
identities. No deletion is part of this task. Then authenticate the appropriate
official Bank/AccountsDB sources, resolve and review their entire pinned closure,
and assess real build/storage needs before preparation. Do not silently upgrade
the accepted runtime or treat direct package presence as full dependency readiness.

A future keyless Bank test must exercise actual commit APIs and reread stored
accounts, distinguish failed-transaction fee-payer debits from unchanged program
accounts, and keep synthetic unsigned sanitation separate from signature-validation
coverage. Genuine Bank API availability and a compatible keyless entry point are
not established by the currently cached modular traits. The unchanged Task 2.31
ELFs and existing fixtures are intended reusable inputs, not new execution proof.

No Mainnet action, deployment, fund movement, secrets access, key creation,
signing, RPC/chain operation or authority transfer occurred. Technical validation
is not founder acceptance or an independent professional audit. Root checkpoints
this bounded preparation task before stopping; Task 2.35 is not started.

## Files and publication

New files: `tools/check_bank_runtime_prerequisites.py`,
`tools/test_check_bank_runtime_prerequisites.py` and this report. Root updates
`AGENTS.md`, `README.md`, `docs/PIV1_PILOT_STATE.md`,
`docs/PIV1_CODEX_EXECUTION_PLAN.md`, `docs/PIV1_MASTER_SPEC.md` and
`docs/PIV1_TEST_PLAN.md`: nine files total. Shared updates record evidence,
limitations and the next dependency; confirmed decisions and production are intact.

Root checks the exact nine-file diff, whitespace, unexpected generated/sensitive
content, hooks/CI, source hashes and authorized ancestry before a normal commit
and integration-only push under D-026. Git records the task commit identity;
the external publication receipt records actual remote refs and clean worktree.
Main remains at `4cc4ea11e87c2f1a2f85b9ed48358f2a881821b3`. Save and STOP;
Task 2.35 is NOT STARTED. No broader founder acceptance is inferred.
