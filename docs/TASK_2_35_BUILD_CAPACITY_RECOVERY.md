# Task 2.35 — Reversible historical Cargo intermediate archiving

Status: **TECHNICALLY VALIDATED / PENDING FOUNDER ACCEPTANCE**. Baseline
integration is `9a72e3ae83852615b8da5df7d89113290574fa0d`; main remains
`4cc4ea11e87c2f1a2f85b9ed48358f2a881821b3`. D-026 governs bounded technical work
and integration-only publication. This task does not authorize Bank dependencies,
claim Bank execution, extend main authority or change economics.

## Scope and initial inventory

Only twelve documented historical host-build targets are eligible: the Task 2.13
claims builds a/b/c, Tasks 2.14/2.28 claims builds, Task 2.30 builds a/b, Task 2.32
builds a/b/c and Task 2.33 builds a/b. Each root is bound to its existing build
result and recorded `CARGO_TARGET_DIR`. No SBF target tree, public Cargo source/archive cache, log,
receipt, executable or shared library is selected.

Writer metadata inspection and root's independent complete hash inventory found
**8368** non-executable regular single-link `.rlib`/`.rmeta`/`.o` candidates:
6420470983 logical bytes and 6437683200 allocated bytes. Eight hardlinked metadata
paths are excluded and retained. Root found **3902 unique contents**, totaling
2893304265 logical bytes; the largest is 22389024 bytes. All candidates belong to
jerem uid/gid 1001, and no target symlinks were observed. Root separately hashed
26901 preserved files. These counts are planning evidence, not reclaimed space.

The first root read-only inventory attempt rejected an atime comparison that
changed during reads. Comparing stable metadata corrected the inventory; the
complete subsequent pass took 47.118 seconds. No archive, deletion or test ran
in that preparation. Root receipts are under `/tmp/piv1-t235-pilot-review`.

## Narrow implementation and recovery contract

Three new files are added: `tools/archive_cargo_intermediates.py`, its `test_`
companion and this report. One delegated writer prepares them; root owns all
actual tests/actions, shared documents and Git. Separate review checks source,
commands, exact inventories and preservation. The tool uses standard-library
Python only: no subprocess, build, dependency, network or signing operation.

The fixed public CLI provides `plan`, `archive`, `verify`, `prune` and `restore`:

- Plan only reads/hashes the approved roots and emits a canonical JSON manifest.
  It records each selected path, size, SHA-256, mode, uid/gid, nanosecond mtime,
  device/inode/linkage, allocated blocks and stable change metadata. Existing pin
  files and build receipts are hash-bound; pinned paths and executables are excluded.
- Archive requires the reviewed manifest and exact tool hashes. All unique contents
  become separate SHA-named deterministic gzip objects in the fresh private durable
  directory `/home/jerem/piv1-evidence/task-2.35-intermediates-20260928-a`.
  Every object is fsynced and independently streamed back through gzip to verify
  its exact uncompressed hash/size. The manifest and complete ready receipt become
  durable before any prune can run. Archive removes no original.
- Verify checks complete object membership, gzip bytes, decompressed identities,
  manifest and ready receipt. Root independently audits all objects before prune.
- Prune rechecks all originals, bindings and the complete archive. A fsynced
  matching journal record precedes each exact original unlink; its parent directory
  is fsynced afterward. Unexplained missing files fail closed. An interrupted prune
  resumes only for absent files explained by the durable matching journal.
- Restore explicitly selects one original path, `all`, or `missing`. It verifies
  the archive, writes an exclusive nonce-named temporary, verifies bytes and exact
  mode/uid/gid/size/mtime, and publishes without overwriting by creating a hardlink.
  The archive is retained. `missing` restores only journal-explained missing files,
  enabling recovery from a partial prune while preserving surviving originals.

Descriptor-based directory traversal rejects symlinks. Actual selected files must
have the expected owner, type, non-executable mode and single linkage. Unexpected
extended attributes/ACLs are rejected rather than silently lost. All selected
Cargo `.cargo-lock` files and the archive lock are held nonblocking during actions.
These locks coordinate cooperating builders; this is not protection against a
malicious process running as the same user.

Restore guarantees content and recorded mode/uid/gid/nanosecond mtime, not original
inode, ctime or atime. A partial restoration temporary is journaled and retained;
a retry uses a fresh exclusive name rather than deleting an unknown file. If
publication succeeded before temporary cleanup or completion journaling was
interrupted, an explicit retry of that exact path recognizes only the verified
journal-bound destination and, when present, its matching second temporary link.
It then finishes cleanup/journaling without replacing the restored file. The
`missing` selection intentionally excludes already-published paths. A truncated
journal fails closed for explicit inspection; originals or verified recovery
objects remain available. No failed partial archive is automatically erased.

## Capacity preflight

The actual action uses a separately selected **268435456-byte (256-MiB) reserve**,
not Task 2.34's later 8-GiB Bank planning minimum. The original 512-MiB reserve
would not fit the complete unique worst-case input before any compression; it was
replaced before action, without inferring a compression ratio or Bank footprint.

The bound uses the official tagged
[zlib v1.3 deflateBound source](https://raw.githubusercontent.com/madler/zlib/v1.3/deflate.c):
for default windowBits 15/memLevel 8, raw deflate plus deterministic gzip's
18-byte header/trailer is bounded by `n+(n>>12)+(n>>14)+(n>>25)+25`.
One percent plus 8192 bytes per object conservatively dominates that expression.
Every object allowance is rounded to filesystem blocks, with additional manifest,
ready/journal metadata slack and the explicit reserve. Installed Python gzip uses
those defaults, compression level 6, empty filename, mtime zero and one final
flush; the tool refuses a different zlib runtime/default parameter set.

All unique worst-case bytes must fit before archive creation. Available space is
checked again before each new object against its bound plus reserve. If space
changes, the tool stops with all originals and any partial archive preserved.
Compression streams bounded chunks; it never buffers entire libraries. Root must
report measured filesystem recovery separately from logical/allocated candidate
sizes and leave the demonstrated restored record in place.

## Validation and actual actions

Root passed **22 focused regressions on the first execution** (unittest reported
0.225 seconds). Tests exercise deduplication and exact byte/metadata recovery,
input/receipt/manifest tampering, symlinks and hardlinks, xattrs and ownership,
space/lock refusal, corruption, existing outputs, and interrupted pruning and both
restore-publication boundaries. Source-review corrections preceded this execution;
no regression failed. No writer tests or filesystem actions were executed.

Every real stage passed on its first execution under the same frozen source:

| Stage | Exit | Process seconds |
| --- | ---: | ---: |
| `tests-a` | 0 | 0.417643 |
| `plan` | 0 | 27.721661 |
| `archive` | 0 | 147.518353 |
| `prune` | 0 | 89.943086 |
| `restore` | 0 | 24.983758 |
| `bank-check` | 2 | 0.067005 |

`bank-check` intentionally returns 2: the unchanged Task 2.34 checker reports
**space sufficient** (8839462912 available bytes against the 8589934592-byte
planning minimum), but all six direct candidate package paths are still absent.
Its overall **NOT_READY** is expected and supplies no Bank execution/readiness.

Root matched every manifest record against its separate pre-action hash inventory;
the reviewer independently compared all records, metadata, exclusions and bindings.
Archive creation removed no original. Root and reviewer then **each independently
decompressed all 3902 objects**, checking 2893304265 uncompressed bytes, their
hashes and the complete object set. All 8368 originals still existed at that gate.
Only afterward did the reviewed prune remove those exact 8368 paths. The actual
restore recreated the largest 22389024-byte library at its original path and left
it in place; **8367 paths remain archived**.

Root and reviewer **each rehashed all 26901 preserved files** after the actions,
checked exact final build-tree membership, unchanged archive objects and the
restored bytes/mode/uid/gid/nanosecond mtime. The reviewer also checked all 16739
journal transitions. No unexpected tree change occurred; eight hardlinked metadata
paths and every original executable/SBF artifact/log/receipt are preserved.
The restored inode necessarily differs and is separately recorded.

Measured archive allocation (all files, including manifest/ready/journal) is
**852205568 bytes**. The restored library allocates **22392832 bytes**, yielding
**5563084800 bytes (about 5.18 GiB) net selected-file recovery**. Filesystem
availability moved from 3276931072 bytes immediately before archiving to
**8839438336 bytes (8.23 GiB)** at the final audit: an observed increase of
5562507264 bytes. That separate observation includes other metadata/report writes;
it must not be conflated with selected-file accounting or Bank build requirements.

Exact source and durable archive identities:

- Tool SHA-256: `7af6be53b8cc42b746f6e3dcf0b454685482cd268a2bf5ae0059dd28cecf8da2`.
- Regression SHA-256: `f225769f32941e2f5ffb79bdfc477fe92ad02e68a78334f0bf1dd76db1f2b760`.
- Manifest SHA-256: `eb590f3a23f453776381c005aa0cc2b6dd19a1ca2d2fe96b9b02a1c310b03e35`.
- Ready receipt SHA-256: `0c20c726558fd7451a9a512e356cc284141393e3fd3c3ef7c48cd2abe489213e`.
- Restored library SHA-256: `a6a0830891f2c44da922208d67bac9c615541d661dab5f0431f6653e6690db67`.

Root receipts are under `/tmp/piv1-t235-pilot-review`: `source-freeze.json`,
`inventory-summary.json`, `selected-before.json`, `preserved-before.json`,
`approved-plan.json`, `approved-archive.json`, `final-preservation.json`,
`capacity-accounting.json`, and each named stage's `.started.json`, `.json`,
`.stdout` and `.stderr`. Before action, 159 prior source inputs, 119 tools/nine
aliases and ten pinned artifacts matched their retained hashes. Earlier Rust/SBF/
Node suites were not rerun or relabelled as current execution.

Separate review under `/tmp/piv1-t235-reviewer`:

| Receipt | SHA-256 |
| --- | --- |
| `source-command-review.json` | `7adb7159ca55d69f2063da1014527de299a00dc65c28d4fba94b55bc52c4574a` |
| `manifest-archive-command-review.json` | `29a32b4040081cdffef9200782f8b65dd3fef17667e6a111cb469cb5df4508e7` |
| `archive-evidence-review.json` | `ec97618ea42b79c8db4817fadf6990fc8cb8f1acee6dbf28531fb1b4ba6d1424` |
| `prune-restore-command-review.json` | `72b0dcea3ee2fb0fceb26d452741cf2095fb1652951dd1129a047cc988c98afa` |
| `final-preservation-evidence-review.json` | `318e32ea8338dd4927ce559987ecabd0d01ef25d2d5bcf50c76bfa70be3ea215` |
| `bank-prerequisite-evidence-review.json` | `551ab63e56dae0550d6b89d0836cd56b9ac0b46f4edef12a235ff30fdc15ebe0` |

## Commands and recovery

Root invoked `/usr/bin/python3 -I -B` with only `PATH=/usr/bin:/bin` and `LC_ALL=C`
in the environment and closed stdin. Complete argument arrays, source hashes,
start/finish times and log hashes are retained in the stage receipts. Actual stages:

```text
/usr/bin/python3 -I -B /home/jerem/piv1/tools/test_archive_cargo_intermediates.py
/usr/bin/python3 -I -B /home/jerem/piv1/tools/archive_cargo_intermediates.py plan
/usr/bin/python3 -I -B /home/jerem/piv1/tools/archive_cargo_intermediates.py archive --manifest /tmp/piv1-t235-pilot-review/plan.stdout --approved-manifest-sha256 eb590f3a23f453776381c005aa0cc2b6dd19a1ca2d2fe96b9b02a1c310b03e35 --approved-tool-sha256 7af6be53b8cc42b746f6e3dcf0b454685482cd268a2bf5ae0059dd28cecf8da2 --reserve-bytes 268435456
```

The same manifest/tool arguments were used for `prune`, then `restore --select`
with the exact path below. Both archive audits were independent read-only scripts,
not a rerun of the tool's `verify` command. The unchanged prerequisite checker ran
with `--output-parent /tmp --minimum-free-bytes 8589934592`.

```text
/tmp/piv1-sbf-claims-build-20260909-b/target/debug/deps/libsolana_sbpf-d61e3710625b0662.rlib
```

For future recovery, preserve the entire durable archive and use its own manifest;
recovery does not depend on the temporary planning copy. Check actual free space
first: restoring all remaining intermediates requires approximately 6.42 GB plus
journal/temporary headroom. After verifying the recorded tool and manifest hashes,
the following restores only the journal-explained missing originals, without
overwriting the already-restored library or any other existing destination:

```text
/usr/bin/python3 -I -B /home/jerem/piv1/tools/archive_cargo_intermediates.py restore --manifest /home/jerem/piv1-evidence/task-2.35-intermediates-20260928-a/manifest.json --approved-manifest-sha256 eb590f3a23f453776381c005aa0cc2b6dd19a1ca2d2fe96b9b02a1c310b03e35 --approved-tool-sha256 7af6be53b8cc42b746f6e3dcf0b454685482cd268a2bf5ae0059dd28cecf8da2 --select missing
```

That complete remaining restore is documented, **not executed** in this task.
The archive and ready receipt remain intact after a restore. Investigate any
unexpected existing file or truncated journal; do not force overwrite or remove
the recovery archive. Selected old target directories must not be reused for a
build unless restored or replaced by a fresh separately scoped target directory.

## Limits and next dependency

Archive storage remains local on the same disk, not an independent disaster backup.
Recovery requires retaining the manifest, objects and sufficient future free space.
Selected historical target trees cease being complete incremental build caches;
logs, receipts, source/cache and exact executables/SBF artifacts remain at their
original paths. No earlier compiler/runtime suite is rerun by this task.

After preservation and measured capacity are verified, the next dependency is
actual authenticated Bank/AccountsDB source/closure and compatible keyless API
preparation. No Bank readiness, signature validation, chain rollback or complete
Testnet readiness follows from archiving intermediates. Native initialization,
real Squads/ALT/control and the remaining lifecycle constraints stay open.

No Mainnet action, deployment, fund movement, secrets access, key creation,
signing, RPC/chain operation or authority transfer occurred. Technical validation
is not founder acceptance or a professional independent audit. Root checkpoints
this bounded task before stopping; Task 2.36 is not started.

## Files and publication

Nine repository files form this task: the new archiver, its regression file and
this report, plus root updates to `AGENTS.md`, `README.md`,
`docs/PIV1_PILOT_STATE.md`, `docs/PIV1_CODEX_EXECUTION_PLAN.md`,
`docs/PIV1_MASTER_SPEC.md` and `docs/PIV1_TEST_PLAN.md`. No production, economics,
Cargo manifest/lock, previous harness/probe or old pin changed. The external
recovery archive is not placed in Git. Root performs final source/document,
secret/generated-file, hook/CI, whitespace and ancestry checks before normal
integration-only publication. Git records the task commit; the external publication
receipt records verified remote refs and clean worktree. Main stays at
`4cc4ea11e87c2f1a2f85b9ed48358f2a881821b3`. Final founder acceptance is not inferred.
