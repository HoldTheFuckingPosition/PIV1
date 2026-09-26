# Task 2.31 — Isolated full-genesis initialization probe preparation

Status: **TECHNICALLY VALIDATED / PENDING FOUNDER ACCEPTANCE** under D-026,
within the build-only scope below.
Baseline integration: `6238088f6dc8ef42a266b5041db9e0e50f262c85`.
Main remains `7b74be4b13c019b96a0c8abcbebfbcc361d31089`.

The founder resumed one economical bounded session after Task 2.30. This task
prepares isolated full-initialization caller/callee and narrowly scoped canonical
Token validation artifacts without changing production, earlier probes or
harnesses. No SBF runtime execution is included. Technical validation and
founder acceptance remain distinct; D-028 does not extend main authority here.

## Source boundary

The new nonpublishable `validation/genesis-initialization-probes` workspace uses
the complete fixed 35/34-account profile, including external payer, System, Token
and both approved same-multisig recipient vault witnesses 0/255. The synthetic
caller reconstructs exact stored bytes and privileges, requires the real outer
payer signer/writable privilege, downgrades the proposal to inner readonly, and
adds only the canonical Squads vault PDA signer with the established seed group.
Stored transaction lengths 1579/1546 remain below the existing 1600-byte bound.
This caller is not the actual Squads executable or governance/control evidence.

The callee invokes only the unchanged public full
`initialize_approved_genesis_with_checked_recipients`. No successful allocation-only
return exists: both Token CPIs and nine state writes must complete. Direct and
nested System/Token invocation errors propagate unchanged. Other validation
errors map explicitly to the isolated probe's `0x2311` category; errors are never
caught and continued. The restricted Token entrypoint verifies canonical ID,
two accounts and exact opcode 18/33-byte InitializeAccount3 data before calling
the unchanged pinned SPL Token 8.0.0 `Processor::process` under SBF.

Every ordinary-host entrypoint fails closed. Host tests cover literal topology,
exact metas/data/seeds, stored transaction lengths, truncations/count overflow,
unsupported extensions, fixed privilege mutations, outer/inner payer privileges,
canonical identities, complete backing preservation, error propagation and Token
wire rejection. They use the existing independent fixture serializer and do not
claim actual initializer or Token success. No host CPI success oracle is added.

## Build and validation record

One delegated writer prepares only new files; separate review checks actual
source/tests/runner/commands/artifacts. Root owns all compilation and test
execution, six shared documents, Git and integration-only publication. The host
gates, first strict target compilation and separate static artifact review below
have passed. Final shared-document review and publication remain root-owned
completion steps. No SBF runtime execution occurs in this task.

The new guarded runner imports the immutable target helper only after exact hash
verification. It derives the historical target profile in memory with precisely
one previously verified libexpat file hash refresh, retaining the original
resolved path and every other profile field. It never edits old pins or helper
globals. Original profile SHA, exact old/new hash and adjustment metadata are
checked; the complete derived profile still passes all original tool/source
checks. This follows Task 2.30's signed OS provenance without installation.

All previous production/probe/harness inputs and historical artifacts remain
protected; earlier suites are retained evidence only, never claimed rerun here.
The standalone lock adds only local packages and uses existing locked registry
identities. The new runner binds full resolved features, complete archive/source
bytes, configuration absences, three required static artifacts and strict raw
diagnostics. Build outputs use fresh private temporary directories and preserve
every rejected attempt. Static compilation cannot prove resource sufficiency or
successful runtime initialization.

The writer's sole locked/offline metadata preparation passed on its first attempt
in 0.267849 seconds, without diagnostics or manifest/lock changes. Exact command,
clean environment, tool hashes, before/after inputs and logs are retained in
`/tmp/piv1-t231-writer-preparation-20260926-a`. The standalone lock was prepared
from the unchanged old probe lock by renaming its two local probe packages and
adding only the local Token wrapper package and its already-locked dependency
edges. Cargo metadata accepted it with `--locked`; no registry resolution update
was needed. All 155 registry identities remain an exact subset of the root lock.
The local/registry-free package entries are PIV1, piv1-math and the three
probes: five local packages in total (160 total packages including the registry
closure). Lock SHA-256:
`cd1d636d60452626bd5e4b81cf11175650cb86f1b2697c94ddd9717556a23c6f`.

```text
/home/jerem/.rustup/toolchains/1.97.1-x86_64-unknown-linux-gnu/bin/cargo metadata --manifest-path /home/jerem/piv1/validation/genesis-initialization-probes/Cargo.toml --locked --offline --format-version 1
```

### Root host gates and separate review

Root's first execution passed **15 Rust boundary tests** (nine caller, four
callee, two Token) and **11 mocked runner regressions**, with zero failures or
diagnostics. Three doctest suites discovered **zero examples**; this is not a
claim of passing doctest examples. Warning-denied documentation also passed.
The four gate commands took **82.682023 seconds** in total. The delegated writer
did not execute these tests or a compiler, and separate evidence review checks
the same root execution without claiming another run.

All commands ran from `/home/jerem/piv1` through the reviewed root script
`/tmp/piv1-t231-pilot-host-validation.py` with the recorded clean explicit offline
environment and pinned Rust 1.97.1 tools. Exact commands:

```text
/home/jerem/.rustup/toolchains/1.97.1-x86_64-unknown-linux-gnu/bin/cargo test --workspace --all-targets --manifest-path /home/jerem/piv1/validation/genesis-initialization-probes/Cargo.toml --locked --offline --jobs 1
/home/jerem/.rustup/toolchains/1.97.1-x86_64-unknown-linux-gnu/bin/cargo test --workspace --doc --manifest-path /home/jerem/piv1/validation/genesis-initialization-probes/Cargo.toml --locked --offline --jobs 1
/home/jerem/.rustup/toolchains/1.97.1-x86_64-unknown-linux-gnu/bin/cargo doc --workspace --no-deps --manifest-path /home/jerem/piv1/validation/genesis-initialization-probes/Cargo.toml --locked --offline --jobs 1
/usr/bin/python3 -I -B /home/jerem/piv1/tools/test_build_genesis_initialization_probes.py
```

The documentation command additionally sets `RUSTDOCFLAGS=-D warnings`.
Evidence: `/tmp/piv1-t231-pilot-host-20260926-a`, with exact commands/environment,
before/after input manifests, eight complete logs and results. Root's receipt
`/tmp/piv1-t231-pilot-review/host-evidence.json` verifies all eight log hashes,
15 source/pin inputs, 128 protected earlier inputs and 119 host tools. The
`results.json` SHA-256 is
`73d28a6e64c3fe14783ec2d7b8ba2428f5f30b1c41df74f2a4ab2c6250e725b5`.
Separate source/runner/command review passed before execution, and
`/tmp/piv1-t231-reviewer/host-evidence-review.json` confirms the root evidence
without a rerun; review SHA-256:
`0e31b691a240c9065779e81890c584a54988add1559f5d977672fda6dbc9cc09`.
The scope, source, host-command and target-command review receipts reside in the
same reviewer directory, separately from root's evidence receipts.

### Strict target build and separate static review

After the host gates and separate command review passed, root ran the first
strict target build into the fresh private output
`/tmp/piv1-genesis-initialization-probes-build-t231-20260926-a`:

```text
/usr/bin/python3 -I -B /home/jerem/piv1/tools/build_genesis_initialization_probes.py --output /tmp/piv1-genesis-initialization-probes-build-t231-20260926-a --execute --approved-runner-sha256 39ca69e7be92a2bd35faa31460950b5459a5dc73d323c5d412a046673dab29f5 --approved-pins-sha256 5b818bf453c5eaf49189ff0d636a234d292000cf43464ae6a89c9bfba6a7c916
```

The exact guarded target command used the recorded explicit tool environment:

```text
/home/jerem/.cache/solana/v1.54/platform-tools/rust/bin/cargo build --manifest-path /home/jerem/piv1/validation/genesis-initialization-probes/Cargo.toml --workspace --lib --release --target sbpf-solana-solana --locked --offline --jobs 1 --target-dir /tmp/piv1-genesis-initialization-probes-build-t231-20260926-a/target
```

The new pins bind 14 new source/runner inputs, 128 protected earlier inputs,
155 existing registry identities, the complete resolved feature graph and six
historical artifacts, including the Task 2.30 exact host executable. No old
source, pin, tool profile, artifact or Git ref may change during compilation.

The first attempt returned **PROBE_STATIC_BUILD_PASS**, Cargo exit zero, with
**zero warnings or errors** and no corrective build. The target compilation took
**362.604063 seconds**. Root verified all **34 commands and 68 complete log hashes**,
all 14 new and 128 protected input hashes, the complete 155-package registry
archive/source closure and all six historical artifacts before/after. Production,
earlier probes/harnesses/tools/pins and protected Git refs remained unchanged.
Root's receipt is `/tmp/piv1-t231-pilot-review/target-evidence.json`; build
`result.json` SHA-256:
`0af51d4105d6474d6056fed0c1d1245e4fdc6330653a8a762b9f32c27a3044b9`.

All three outputs reside under the build directory's
`target/sbpf-solana-solana/release`:

| Artifact | Bytes | SHA-256 |
| --- | ---: | --- |
| `piv1_genesis_initialization_caller.so` | 61,080 | `e7fc7bf5d75a9494fd3a4787733df53adf8c3b724653a113ae36a9574707ec97` |
| `piv1_genesis_initialization_probe.so` | 376,720 | `07934627a3fdab928ab1aca2abf424eb683ea9e680681389c9b53dc2391a66b7` |
| `piv1_genesis_initialization_token.so` | 126,424 | `c0f42a30da4079601711bec29bd0ca780674654ea71790eb32c76b5cedad4499` |

Separate inspection of all three exact artifacts passed in
`/tmp/piv1-t231-reviewer/artifact-review.json`, SHA-256:
`6b857cbb97b6144f7c659dbdc0897c5e984604b9fd36f1f583562f7221769c38`.
The reviewer independently parsed ELF bytes and checked the retained readelf/
disassembly logs. Each artifact is little-endian ELF64, type 3, machine 263,
flags zero, with matching executable entrypoint and the standard defined dynamic
symbols `entrypoint` and `custom_panic`. Exact ELF-bound disassembly covers every
`.text` byte:

| Artifact | Text bytes | Disassembled instructions | Direct frame memory instructions |
| --- | ---: | ---: | ---: |
| Caller | 41,120 | 4,899 | 1,047 |
| Callee | 301,440 | 36,685 | 9,580 |
| Token wrapper | 90,152 | 10,742 | 3,030 |

Every inspected direct `r10` memory access remains within `[-4096, -1]`.
This checks direct frame accesses only; it does not analyze pointer aliases,
the complete dynamic call graph, total heap/compute, successful initialization
or rollback. No target was loaded or executed. Static imports, including the
Token processor's linked return-data symbol, do not establish executed behavior.

The separate reviewer also verified all 34 commands/68 logs, 14 new inputs,
128 protected inputs, six historical artifacts and exact resolved metadata.
An independent post-build check covered all 101 target-profile files, three
library trees and 155 checksum-bound registry archives with **4,468 complete
extracted source files**, in addition to matching root before/after package
records. The reviewer ran no compiler, tests, loader or network operation.
No source correction was needed after the first validation execution, and no
host gate or target build failed in this bounded task. Pre-execution review
clarified a caller comment and added profile/third-artifact guard coverage.

### Retained earlier evidence

Root verified **250 retained logs**: 50 from Task 2.30, 128 earlier logs, 64 from
the Task 2.29 target build and eight from its final host gates. The earlier
469 production host tests plus one doctest/eight gates, 24 local SBF tests/70
cases, 15 Node tests/eight old plus sixteen recipient cases, Task 2.29 probe
boundary/build evidence and Task 2.30 twelve preflight runtime cases remain
attributed retained evidence, **not rerun** in Task 2.31. Their unchanged source,
tool and artifact identities do not establish runtime success for these new
mutating initializer artifacts.

## Limits and next dependency

Production sources, ABI, economics, existing harnesses, probe bytes, tools and
historical pins remain unchanged. Full initialization runtime/resource/atomicity
evidence remains the subsequent candidate. Funding provenance, later Token-native
donations, actual Squads/ALT lifecycle, full recipient control and founder
Testnet readiness remain deferred. No Mainnet action, deployment, chain/RPC
operation, fund movement, key creation/signing, secrets access or authority
transfer occurs. AI-assisted review is not a professional independent audit.

Files: twelve new workspace files, three new tool/pin files and this report;
root owns the six shared documents and final publication. Separate final review
checked the 22-file scope, unchanged frozen inputs, shared documents and this
report; minor wording clarifications precede the final review receipt. Root
records the reviewed commit/publication identity through Git and verifies actual
remote refs and the worktree after normal integration-only publication. Checkpoint
and STOP after this bounded task.
Task 2.32 is NOT STARTED; main acceptance and live-operation gates are unchanged.
