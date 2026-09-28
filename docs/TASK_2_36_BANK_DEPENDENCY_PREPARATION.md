# Task 2.36 — Bank dependency preparation

Status: **TECHNICALLY VALIDATED / PENDING FOUNDER ACCEPTANCE**. Baseline integration is
`90132103af8a99164a3198214ce72c053673d645`; main remains
`4cc4ea11e87c2f1a2f85b9ed48358f2a881821b3`. D-026 covers this isolated technical
preparation and reviewed integration-only publication, without extending D-029,
founder acceptance, economics or live-operation authority.

## Bounded purpose

Task 2.35 recovered local capacity while preserving sources, receipts and exact
artifacts. Task 2.34's cache-path observation was not dependency authentication.
This task prepares an independently locked public Bank/AccountsDB graph aligned
with the existing Agave 4.2.0 modular runtime and records a source-reviewed local
entry that needs no secret-key creation or transaction signing. It does not build
or execute Bank, change production dependencies, replace existing pins, or claim
new commit/rollback evidence.

One delegated writer adds `validation/genesis-bank-runtime/Cargo.toml`, a
documentation-only `src/lib.rs`, its README and this report. Root owns registry
preparation, lock/metadata execution, dependency/evidence pins, shared documents
and Git; a separate reviewer checks source and actual preparation evidence.

The manifest fixes runtime 4.2.0 with `agave-unstable-api` and
`dev-context-only-utils`; account 4.3.1, genesis-config 4.0.0, message 4.4.0,
transaction 4.1.5, pubkey 4.2.0, hash 4.5.0, address 2.6.1 and short-vec 3.2.2.
The last two preserve the existing wincode 0.5 SDK trait boundary. The dev feature
adds test utilities, including program binaries; its transitive build behavior
must be reviewed before any future compilation.

## Source-reviewed API plan

The [workspace README](../validation/genesis-bank-runtime/README.md) records exact
source locations and the prospective construction/sanitation/commit path. Root's
authenticated direct runtime/AccountsDB/SVM 4.2.0 sources identify Agave VCS commit
`ac82b5d438b0c2303dc7169f52c748977713a111`.

Explicit `GenesisConfig` data and `Bank::new_with_paths_for_tests` avoid the
keypair-creating genesis helpers. The dev constructor eagerly creates public
leader identifiers even with an explicit leader; the traced implementation uses
only public counters/hashes. `Transaction::new_unsigned` supplies zero signature
placeholders. `prepare_entry_batch` sanitizes and locks the message but does not
cryptographically verify those signatures. This local test seam must not be
presented as normal validator signature validation.

`load_execute_and_commit_transactions` reaches the real account-saver and
AccountsDB storage path; `get_account` provides independent Bank reads afterward.
The future oracle must separate successful custody writes, failed program-state
rollback and the failed transaction's retained fee debit. Nonce effects are
additional and should be excluded from the first fixture. Failed processed
messages enter replay protection, so retry messages must differ deliberately
without clearing the status cache or resetting custody state.

These are source findings, not executed Bank behavior. Initializer/artifact loading,
genesis features, fees, resource limits and complete before/after state still need
a separately reviewed harness. Earlier Mollusk evidence remains output discard;
no Bank rollback, ledger durability, actual Squads/control or live readiness is
established here.

## Actual preparation and review evidence

Root used a fresh private public-only cache at
`/tmp/piv1-t236-preparation-20260928-a/cargo-home`. Existing caches and archived
build targets were not used or modified by Cargo. The command environment contains
only explicit tool/cache/output paths, locale and public registry settings; it
does not inherit HOME, credentials, proxies or user Cargo configuration. Ancestor
Cargo configuration is refused. The installed, previously pinned Cargo/rustc
1.97.1 tools were rehashed before use. No dependency build script, procedural
macro, bundled ELF, Bank or test binary was executed. No build target was created.

Root fetched the three direct runtime/AccountsDB/SVM archives from
`static.crates.io`, matched their hashes to the exact non-yanked 4.2.0 records in
the HTTPS crates.io sparse index, and compared their principal source files with
the immutable official Agave commit. All three published VCS records identify
`ac82b5d438b0c2303dc7169f52c748977713a111`. This is registry/source byte
authentication, not verification of publisher signatures or an exhaustive code
audit. The [preparation record](../validation/genesis-bank-runtime/preparation.json)
contains exact URLs, archive/index/source hashes, tool identities and receipts.

The independent workspace's lock was seeded with a copy of the Task 2.33 lock.
The resulting complete all-platform closure contains **589 registry packages**:
**300** retain an exact prior package version/checksum and **289** are new records
for this workspace. Every registry archive was checked against both the new lock
and its observed sparse-index entry; **25,891 extracted source files** match the
authenticated archive members. All observed selected entries were non-yanked.
Archives total **71,994,120 bytes** and source members **467,114,767 bytes**;
these are logical sizes, not the entire preparation's allocated disk footprint.

Some older-only dependency families disappear or resolve differently in this new
graph. Cargo reports updates for base64, heck, feature-gate-interface,
program-pack, the older pubkey family and system-interface; additional parallel
major versions are needed by new transitive dependencies. The preparation record
compares every shared family rather than describing the closure as unchanged.
The direct SDK compatibility pins and Agave 4.2.0 boundary are preserved; no old
manifest, lock, source or production dependency was upgraded. Each new transitive
version/checksum is fixed by this isolated Cargo.lock.

Root executed these metadata commands with the exact Cargo binary
`/home/jerem/.rustup/toolchains/1.97.1-x86_64-unknown-linux-gnu/bin/cargo`, from the
private preparation directory, against the absolute manifest path
`/home/jerem/piv1/validation/genesis-bank-runtime/Cargo.toml`:

```text
cargo metadata --format-version 1 --manifest-path <absolute-manifest>
cargo metadata --format-version 1 --manifest-path <absolute-manifest> --locked --offline
cargo metadata --format-version 1 --manifest-path <absolute-manifest> --locked --offline --filter-platform x86_64-unknown-linux-gnu
cargo metadata --format-version 1 --manifest-path <absolute-manifest> --locked --offline
```

The public-registry resolution passed in **13.021083 seconds**. The first full
locked/offline observation passed in **0.566712 seconds** with byte-identical
JSON; host-filtered metadata passed in **0.557911 seconds**. The final full
observation passed in **0.554943 seconds** after the writer added the README.
Its only metadata change is the local package's automatically discovered README
path; the full registry graph and feature sets are identical. Both full and host
offline stderr logs are empty; resolution has package/download notices and no
warnings or errors. All four commands passed on their first execution; this is
metadata validation, **not a test or compilation result**. See Cargo's primary
[metadata documentation](https://doc.rust-lang.org/cargo/commands/cargo-metadata.html)
for the distinction between the full graph and platform filtering.

The host graph has **552 registry packages** and **56 build-script packages**;
the full graph has **70 build-script packages**. Their identities are recorded,
not approved for unchecked execution. Native dependencies include OpenSSL,
LZ4, Zstandard, ring and blst. The resolved OpenSSL features select **vendored**
OpenSSL source, requiring review of Perl/Configure, make, the C compiler and their
disk/time costs; checking system libssl alone is insufficient. Zstandard selects
bundled C/assembly compilation without bindgen/cmake/pkg-config features.
`solana-version` invokes Git; `solana-perf` observes AVX/AVX2 and blst can observe
ADX during a future build. The program-binaries crate embeds ten ELF files,
without a download build script. Their feature inclusion does not itself load
them through the proposed direct GenesisConfig constructor. A future harness must explicitly
control genesis features and loaded program bytes; it must not substitute these
bundled programs for the exact Task 2.31 artifacts silently. Native tool/library
availability, build-script behavior, compiler/type compatibility and measured
peak resource use still need the next bounded build's review and validation.

One root read-only audit attempt stopped because it incorrectly required the
optional `.cargo-checksum.json` extraction marker. That attempt and its error
are retained. The corrected audit keeps full archive/member byte comparisons
and checks the marker when present; it then verified the complete closure.
No dependency bytes, lock, version, feature or oracle were changed to obtain
that pass. The Cargo metadata commands themselves did not fail.

Exact identities:

- Manifest SHA-256: `2f8d9ed1346d20381dfb2e60971a9267d3af9705faff60aa0d281e4d2b14c1ef`.
- Lock SHA-256: `06332f984c80bdc13092df0f01be190c14514db56968726913feb1e14b0c6af8`.
- Final full metadata SHA-256: `8b3d5cb556c04851467335dad2b02cbe55605d3317b3e40213a81a294e0d6f3e`.
- Host metadata SHA-256: `d0fa252baf786da1f8947380c35cf4934939ec22cb67ae65c76f363b62ce3b16`.

All actual command arguments, scrubbed environments, timings and complete logs
are retained under `/tmp/piv1-t236-preparation-20260928-a`: `resolve.*`,
`offline.*`, `host.*`, `offline-final.*`, `direct-sources.json`,
`official-source-match.json`, `closure-audit.json`, `closure-audit-attempt-a.*`
and `preservation.json`. Receipt hashes are pinned in `preparation.json`.
The writer ran only source/document inspection and assigned-file edits; the
separate reviewer did not execute Cargo, compiler, runtime or tests. Root's
takeover matched **162 protected inputs, 119 tools, nine aliases and ten artifacts**.
The 162 inputs and ten artifacts were rechecked after preparation. Earlier test
results remain retained evidence, not reruns. Task 2.35's archive manifest and
ready receipt identities were rechecked; its 3,902 objects were not rehashed
again in this task. Keep that complete recovery archive intact.

Observed remaining space is **8,127,578,112 bytes (7.57 GiB)**. Source/cache
preparation uses capacity, so the earlier chosen 8-GiB planning reserve is no
longer met. That threshold was never a measured Bank requirement. The next task
must assess a bounded build's actual disk/memory needs, native prerequisites and
explicit account-database/thread configuration before execution. No automatic
cleanup, installation or heavy build follows from metadata success.

Separate command review passed before resolution; its receipt is
`/tmp/piv1-t236-reviewer/direct-source-metadata-command-review.json`, SHA-256
`d02caa427d61c8ff3535c981644bde9b8010b857bc032a221aeb8417667674c0`.
The reviewer independently repeated the complete 589-archive/25,891-member
comparison, checked all eight metadata logs and all 61 shared-family version
comparisons, and confirmed the host/full graph distinction. The receipt is
`/tmp/piv1-t236-reviewer/complete-closure-evidence-review.json`, SHA-256
`565fbab998f14fe273ced2c186c3994ee8913eacaffb8caf2d54d1c2f2845615`.
The separate source/API review also passed, binding 23 reviewed source files and
the ten bundled ELF hashes: `/tmp/piv1-t236-reviewer/source-api-review.json`,
SHA-256 `6fdbc51549e11536b17124c65931c43d09288ea753e13e47d83f4970ef508d89`.
Final indexed document/publication review precedes Git publication. No AI-assisted
review is represented as a professional audit.

## Files, continuation and publication

Six new files: the workspace's `Cargo.toml`, `Cargo.lock`, `src/lib.rs`,
`README.md`, `preparation.json` and this report. Root updates the six shared
documents: `AGENTS.md`, `README.md`, `docs/PIV1_PILOT_STATE.md`,
`docs/PIV1_CODEX_EXECUTION_PLAN.md`, `docs/PIV1_MASTER_SPEC.md` and
`docs/PIV1_TEST_PLAN.md`. No economic rule or native instruction changes.
After final review and publication checks, root commits and pushes integration
normally under D-026; Git records the resulting identity. Main remains at
`4cc4ea11e87c2f1a2f85b9ed48358f2a881821b3`. Save and STOP;
**Task 2.37 is NOT STARTED**. Next prepare the bounded genuine Bank harness/build
using this authenticated closure, then independently inspect actual stored-account
success/failure/retry evidence. Native initializer exposure, real Squads/ALT,
recipient control, funding provenance and the remaining lifecycle stay open.

No Mainnet action, deployment, fund movement, secrets access, key creation/signing,
RPC/chain operation or authority transfer is authorized or performed by this
source-only preparation. Technical validation is not founder acceptance or a
professional independent audit.
