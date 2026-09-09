# Task 2.12 — Keyless SBF compilation and artifact inspection

Status: **TECHNICALLY VALIDATED / PENDING FOUNDER ACCEPTANCE**.
Baseline: reviewed, published Task 2.11 closure
`14106d664c107b3a2f705ac87361768af42d0786` on `integration/piv1-testnet`.
This is the next bounded D-026 dependency after completed review/host validation
and publication. Founder acceptance and live-operation authorization remain absent.

## Requirement and evidence boundary

Task 2.11 connects the isolated K-012 claim to a real instruction boundary, but
host compilation cannot prove Solana-target compatibility. Establish a repeatable
keyless local SBF compilation route and inspect the exact artifact and diagnostics
before introducing any runtime harness. Preserve the accepted economics, claim
semantics, account layouts, dependencies, release profile and runtime safety gates.

Initial writer scope is a small reusable local build/inspection runner, this report,
and actual target compilation of the frozen Task 2.11 code. Production code changes
are not initially authorized by this task: if compilation exposes incompatibility,
report exact diagnostics/artifact evidence to the pilot first. The pilot may scope
and separately review compatible corrections within D-026 before writer edits.
Do not suppress diagnostics, change limits/flags to hide problems, or claim a
successful artifact while a known stack/target error remains unresolved.

No SBF execution, CPI, validator, transaction, payer or live operation belongs to
this task. A compiled artifact is not proof of runtime account privileges, Rent,
System CPI, signature validation, compute/heap usage or transaction rollback.
Those remain a later separately pinned keyless harness dependency.

## Verified route and safe output handling

Use the direct cached Cargo/Rust compiler stage proposed in the
[Task 2.11 assessment](TASK_2_11_KIF_CLAIM_INSTRUCTION_BOUNDARY.md#read-only-next-dependency-assessment).
The pilot independently read the pinned compiler invocation, post-processing and
Rust target sources. Builder post-processing creates keys; never invoke Anchor,
cargo-build-sbf, rustup installation/linking, deployment tooling or key generators.

Add `tools/build_keyless_sbf.py` using Python standard library only. Use structured
subprocess argument arrays, explicit repository cwd and a minimal clean environment;
never a shell, inherited compiler wrappers, arbitrary user flags or command text.
Select the existing v1.54 platform Cargo/Rust/LLVM binaries directly, with the
reviewed environment and target flags. Use default PIV1 features, release profile,
`--package piv1 --lib --release --target sbpf-solana-solana --locked --offline
--jobs 1`. Keep root manifest/profile, dependency lock and toolchain file unchanged.
The v0 target is a local diagnostic choice, not a live deployment architecture
decision. No stack-size override, feature bypass or relaxed overflow checks.

The runner must verify expected paths and pin/record actual tool identities,
version metadata, relevant compiler driver/shared libraries and target libraries.
Check applicable Cargo configuration locations without reading credentials;
if unexpected configuration exists, stop and report it rather than ignore it.
Inspect selected locked dependency build scripts and record their identities.
The pilot's read-only locked/offline metadata inventory found 22 build scripts;
targeted scanning found compiler probes/generated code, not key creation. This is
not a professional supply-chain audit. Do not modify cached dependencies/tools.

Create a fresh private temporary task directory outside Git, never reuse/delete
an existing output path. Capture source/manifest/lock/tool hashes, complete clean
build command/environment, logs, exit code and before/after Git state. Verify the
source set/hashes and protected Git refs remain unchanged through compilation.
Keep generated objects/binaries out of Git; record hashes and reproducible commands
in the report. Do not inspect any preexisting key material. Audit only newly created
output paths for unexpected key/credential files; do not open such files if found.

Before any actual SBF build, the writer presents the runner and exact command for
separate technical review. The pilot handles that review directly. This is the
final technical preparation gate, not a founder permission request. Source review
must confirm the command cannot invoke builder post-processing or key handling.
One build at a time; writer owns the first build slot only after this review.

## Artifact and diagnostics checks

Record every diagnostic, including compiler output that might accompany exit zero.
Treat stack-limit/target errors as failure regardless of Cargo status. Preserve
the original complete log. Do not silence warnings to manufacture a pass.

On successful compilation, inspect the unstripped artifact with cached LLVM tools:
SHA-256/size, ELF class/endianness/machine/flags, entrypoint definition/export and
executable segment, sections/relocations, unresolved symbols/syscalls and stack
diagnostics. Use pinned target/runtime primary sources for interpretations.
If useful, create a distinct stripped artifact with the reviewed direct
`llvm-objcopy --strip-all INPUT OUTPUT`; retain and hash both originals/results.
Do not patch ELF headers or change architecture implicitly. Record the combined
cdylib/lib LTO caveat from the published builder and actual compiler behavior.

Inspect the exact call path and allocation/stack observations relevant to claims.
Absent compiler diagnostics alone do not establish safe heap, call depth, compute
or runtime behavior. Keep unknowns explicit rather than asserting deployability.
If no artifact is produced, record the reproducible failure and required compatible
correction; no later runtime work begins before this dependency is resolved.

## Completion and coordination

One delegated writer `implement_t26_deposit` owns the runner/report and bounded
compilation evidence. Separate reviewer `review_t23_final` reviews this contract,
the runner before execution and final exact source/evidence. The pilot owns shared
checkpoint/status documents, inspects actual evidence, and coordinates corrections.
No routine founder technical approval is needed under D-026.

Meaningful validation: runner rejects existing output paths without mutation;
exact command/environment/tool identities pass pre-execution review; actual locked,
offline Solana-target compilation and independent artifact/log inspection agree;
source hashes and Git state remain unchanged. Do not rerun the 335+1 host baseline
merely for documentation/build-runner changes. If production compatibility edits
become necessary, freeze them and run appropriate host/target regressions plus
separate final review before normal commits/publication and the next checkpoint.

No new dependencies, key creation, secrets access, signing, deployment, Mainnet,
fund movement, live identities or authority transfer. AI-assisted technical review
is not founder acceptance or a professional independent audit.


## Written-scope review and preparation dispatch

Separate reviewer `review_t23_final` returned PASS with no required corrections.
The writer may now prepare the runner/report; actual SBF compilation remains
behind the specified separate runner/command review. No production edit is scoped.
The pilot verified executable/version metadata, 27 compiler/root and v0 target
library hashes including the resolved rustc driver, and absence of Cargo config
at repository/ancestor/home locations. Temporary reviewed inventory:
`/tmp/piv1-keyless-reviewed-tool-hashes.json`; locked dependency build-script list:
`/tmp/piv1-keyless-build-script-inventory.json`. Reverify these against actual files
and preserve concrete tool pins with the reusable runner. No compiler has run.

## Writer preparation evidence — no target build executed

The writer verified `jerem`, `integration/piv1-testnet`, and actual HEAD
`14106d664c107b3a2f705ac87361768af42d0786`. The five modified shared/publication
documents and this new scope report were pilot-owned on entry and were preserved.
Only `tools/build_keyless_sbf.py`, `tools/keyless_sbf_pins.json`, and this evidence
append are writer changes. No program source, test, manifest, release profile,
lockfile, dependency or toolchain change was made.

The stdlib-only runner defaults to preflight. It uses fixed argument arrays and a
complete replacement environment, a private mode-0700 direct `/tmp` directory,
and atomic creation that refuses existing files, directories and dangling links.
No existing output is reused or removed. It checks Cargo configuration absence
without reading any configuration or credential contents. It records commands,
full separate stdout/stderr byte logs, statuses, log hashes/sizes, input hashes,
and before/after Git state. The 67 source/manifests/lock/toolchain/test paths are
bound to their exact recorded contents. The published baseline must be an ancestor
of actual HEAD; later documentation/runner commits can therefore use the same
frozen build inputs. Branch, HEAD, local/remote-tracking refs and source/tool/runner
hashes must remain unchanged through a run. Worktree status is recorded separately
because pilot documentation is shared work.

The companion preserves all 37 pilot-reviewed path/hash entries verbatim. It
extends this to 101 pinned path identities (100 distinct resolved paths), including
host Rust sysroot libraries, selected native archives, the host `cc`/`collect2`/`ld`
linking support and resolved ELF shared libraries. Sixteen selected ELF roots have
checked `ldd` dependency paths. The 22 locked custom-build targets are bound to 47
selected build-script/manifest/helper file hashes and rechecked against direct
locked/offline Cargo metadata. These are selected build inputs and provenance
checks, not a complete dependency-tree or professional supply-chain audit. The
runner does not sandbox build scripts. Cached dependencies/tools remain unchanged.

Observed version commands report cached Cargo `1.89.0 (d96b80636 2026-01-27)`,
Rust `1.89.0-dev` (its verbose command reports commit hash/date as unknown), Clang
`20.1.7-rust-dev` and LLD `20.1.7`. The separately pinned `version.md` contains the
Rust/Cargo/newlib source revisions recorded by the pilot; the runner does not
substitute those metadata revisions for the compiler's reported identity.

Execution mode additionally requires the exact separately reviewed runner and pin
hashes before any output creation. Those arguments detect drift; they are not an
authorization capability. The following command is **proposed, not executed** and
requires the separate pre-execution review PASS and the pilot's build-slot release:

```sh
/usr/bin/python3 -I /home/jerem/piv1/tools/build_keyless_sbf.py \
  --output /tmp/piv1-keyless-sbf-build-20260909-a \
  --execute \
  --approved-runner-sha256 a8e15d17f34ea8ac778e5983735840ffce470fbb1ce6f0f377d0764d646eace8 \
  --approved-pins-sha256 594c86ae2dd1257fe091f93e9fe2b1809ee777cc568622193bfe5777c258767f
```

Its fixed compiler argv, with cwd `/home/jerem/piv1`, is:

```text
/home/jerem/.cache/solana/v1.54/platform-tools/rust/bin/cargo build --manifest-path /home/jerem/piv1/programs/piv1/Cargo.toml --package piv1 --lib --release --target sbpf-solana-solana --locked --offline --jobs 1 --target-dir /tmp/piv1-keyless-sbf-build-20260909-a/target
```

The complete compiler/version/inspection environment for that proposed output is:

```text
PATH=/usr/bin:/bin
LC_ALL=C
CARGO_HOME=/home/jerem/.cargo
RUSTC=/home/jerem/.cache/solana/v1.54/platform-tools/rust/bin/rustc
RUSTDOC=/home/jerem/.cache/solana/v1.54/platform-tools/rust/bin/rustdoc
CC=/home/jerem/.cache/solana/v1.54/platform-tools/llvm/bin/clang
AR=/home/jerem/.cache/solana/v1.54/platform-tools/llvm/bin/llvm-ar
OBJDUMP=/home/jerem/.cache/solana/v1.54/platform-tools/llvm/bin/llvm-objdump
OBJCOPY=/home/jerem/.cache/solana/v1.54/platform-tools/llvm/bin/llvm-objcopy
CARGO_TARGET_SBPF_SOLANA_SOLANA_RUSTFLAGS=-Zremap-cwd-prefix= -C linker=/home/jerem/.cache/solana/v1.54/platform-tools/rust/lib/rustlib/x86_64-unknown-linux-gnu/bin/rust-lld
CARGO_TERM_COLOR=never
TMPDIR=/tmp/piv1-keyless-sbf-build-20260909-a/tmp
```

Local Git reads use a separate fixed environment with system/global Git configuration
disabled and optional locks disabled. No inherited compiler wrapper, Cargo profile,
encoded Rust flags, target directory or PATH is forwarded. The reviewed default
features, v0 target and release profile remain fixed. The existing combined
`cdylib`/`lib` declaration retains the published builder's LTO caveat: that builder
reports its expected LTO cannot be applied to the combined types. Actual direct
compiler behavior is still unobserved; no profile adjustment has been made.

After a future build, raw diagnostics are retained even with Cargo exit zero. The
runner independently rejects error/stack-bound/target diagnostic patterns, then
inspects an existing artifact using the pinned LLVM readelf and disassembler.
It hashes the original artifact, records ELF header/sections/segments/symbols,
requires an aligned entry inside executable `.text` and a matching file-backed
executable PT_LOAD mapping, and checks its exported `entrypoint` symbol. Its ELF64,
little-endian, machine-247, flags-zero checks deliberately describe the expected
v0 compiler output, not all runtime-supported ELF variants. Unexpected output is
retained in the raw inspection logs and rejected for investigation without header
or compiler-flag changes. The pilot's referenced sbpf v0.12.2 loader accepts more
than one machine value; that reference is not a selected runtime dependency.
Unresolved symbols and complete disassembly remain review evidence, not a syscall
allowlist or proof of runtime callability. No stripped artifact is requested.

Preparation commands and results:

- Python `compile(source, filename, 'exec')` syntax checks passed without writing
  repository bytecode caches. Initial exploratory `rg --files tools` found no
  directory; the authorized `tools` directory was subsequently created.
- `/usr/bin/python3 tools/build_keyless_sbf.py --output
  /tmp/piv1-keyless-sbf-preflight-20260909-a` passed on the initial saved runner.
  It ran version/metadata inspection only; `target_build_started=false`.
- The first `/usr/bin/python3 -I
  /tmp/piv1-keyless-runner-preparation-checks.py` attempt exited 1 because its
  `tempfile` suffix contained an underscore, outside the runner's allowed output
  alphabet. This was a harness naming error. Its intended existing-path assertions
  were not counted as valid evidence. The harness now uses hexadecimal names and
  asserts the specific refusal reason. No target compilation was reached.
- The corrected harness passed 33 checks on the intermediate source, then all 33
  again after explicit incomplete-preflight guarding and artifact-identity capture.
  The final run is `/tmp/piv1-keyless-sbf-checks-4ddabb85d69d459bb5452622d1c3a9b2`.
  Its `checks-summary.json` lists every assertion. Malformed/nonobject/schema-invalid
  temporary pins intentionally produced durable FAIL records with zero subprocess
  commands and no target build; the original parse/schema error remains separate
  from the explicit inability to assert an unverified baseline. The actual
  companion was never modified for these tests.
- Final preflight evidence is
  `/tmp/piv1-keyless-sbf-checks-4ddabb85d69d459bb5452622d1c3a9b2-preflight`.
  All 26 version/dependency/metadata commands exited zero; every stderr log was
  empty. Preflight passed under deliberately unusable inherited wrapper/flag/profile
  variables, confirmed their absence from the replacement environment, and verified
  mode 0700 plus preserved source/tool hashes and protected refs.
- The 33 checks cover existing directory/file/dangling-link refusal, missing/wrong
  review hashes, repository-output rejection, malformed and nonregular/symlink pin
  companions, baseline ancestry, diagnostic classification, eight synthetic ELF
  boundary cases, and clean-environment/preflight preservation. Synthetic ELF bytes
  exercise only the parser; none is an SBF compiler artifact or execution result.

Preparation source freeze:

| File | SHA-256 |
| --- | --- |
| `tools/build_keyless_sbf.py` | `a8e15d17f34ea8ac778e5983735840ffce470fbb1ce6f0f377d0764d646eace8` |
| `tools/keyless_sbf_pins.json` | `594c86ae2dd1257fe091f93e9fe2b1809ee777cc568622193bfe5777c258767f` |
| Temporary check harness | `79f8f742496ddae26c96d3167fc9cafc12d9acb6dcda567c20b244a932734026` |
| Final preflight `result.json` | `469928b88fa18d72aac28360226ccd6c15196f0ae419f0a069db49d0fb0149ab` |

No actual SBF build, artifact production or runtime test has occurred. Stack,
heap, call depth, compute, actual CPI/privileges/Rent and transaction rollback
remain unvalidated. No host baseline suite was repeated for this runner-only work.
No Anchor, cargo-build-sbf, rustup, key generator, signing, validator, installation,
network blockchain operation, deployment, Mainnet action, fund movement or authority
transfer occurred. No key/secret material was accessed. The writer made no Git
mutation, commit or push; HEAD and accepted main remain unchanged. Source is frozen
for separate pre-execution review. No SBF build slot has been used or assumed.

## Pilot preparation inspection and independent preflight

The pilot inspected the final runner, companion inventory, report and actual
33-check preparation harness/results. All 37 original reviewed tool pins are
preserved verbatim; the final runner/pin hashes match the frozen table above.
The pilot independently checked the writer's 26 command statuses, empty stderr
and recorded log hashes against actual files. These are inspected writer results,
not pilot executions of the 33-check harness.

The pilot then independently executed exactly:

```sh
/usr/bin/python3 -I /home/jerem/piv1/tools/build_keyless_sbf.py --output /tmp/piv1-keyless-sbf-pilot-preflight-20260909-a
```

Result: `PREFLIGHT_PASS_NO_TARGET_BUILD`, exit zero, 26 commands all exit zero,
all stderr logs empty, output mode 0700, identical before/after Git state and
verified source/tool/runner/pin preservation. Actual logs and hashes were checked.
Result JSON SHA-256:
`daae1873047a2842059f26f7146f4d6a0fdb45aa3e9702b232e49608239faba6`.
No target build occurred. Separate exact runner/command review is pending;
the writer remains frozen without a build slot. No sensitive operation occurred.

## Separate pre-execution review and first-build release

Reviewer `review_t23_final` returned **PRE-EXECUTION PASS**, no required
corrections, for the exact frozen runner/pins and proposed first command above.
It independently read the complete runner, companion, report and 33-check harness;
verified 101 tool/support identities, library sets, 47 selected build-script inputs
and 67 source hashes. It ran only read/hash/Git inspection commands, not builds.
Its reviewed report hash predates the pilot evidence append:
`fd8745bacb48b356c3dbe974233816e577d02dbfaafe53de0c63e5ef627f9e15`.
Runner/pins and compiled inputs are unchanged. The pilot now releases the sole
writer's first build slot for that exact command and fresh output directory.
This approves only the local compilation attempt; final diagnostics/artifact
inspection and separate review remain required. No production edit is authorized.

## Writer first target attempt — failed validation, artifact retained

After the explicit PRE-EXECUTION PASS and sole build-slot release, the writer
reverified user/branch/HEAD, the two reviewed runner/pin hashes, all 67 frozen
input hashes and absence of the proposed output. The exact proposed command above
was executed once. Cargo ran from `2026-09-09T12:21:57.085345Z` to
`2026-09-09T12:27:03.588937Z` (306.504 seconds), returned **zero**, and reported
`Finished release profile [optimized]`. **Target validation failed.** The retained
6559-byte `build.stderr` contains seven compiler error diagnostics:

| Function | Compiler-estimated frame | Excess over 4096 bytes | Additional errors |
| --- | ---: | ---: | --- |
| `accounts::authenticate_fixed_accounts` | 5248 | 1152 | None |
| `guardian_clock_accounts::authenticate_guardian_clock_snapshot` | 5632 | 1536 | None |
| `kif_claim_execution::execute_kif_claim` | 10688 | 6592 | Four calls reported to overwrite frame values |

The compiler explicitly warns that the excessive offsets/call overwrites may
cause undefined behavior during execution. Cargo's zero exit and artifact presence
do not override these errors. No warning suppression, stack override, header edit,
profile change, correction or retry occurred. The original complete stdout is
empty; the complete stderr and original failed `result.json` are retained unchanged
under `/tmp/piv1-keyless-sbf-build-20260909-a`.

The first runner also exposed an independent preparation defect: after Cargo
finished, its filename-only audit rejected the generated fingerprint name
`target/sbpf-solana-solana/release/.fingerprint/solana-sysvar-id-3587705b029f3e0b/lib-solana_sysvar_id.json`.
The broad `id.json` suffix rule matches this ordinary Cargo metadata name. Its
contents were **not opened**. The runner exited 1 with `FAIL` before its diagnostic
classification and ELF inspection, and the same audit failure prevented its final
preservation checks. That original failure record has not been amended into a pass.
A separate names/metadata-only walk counted 1526 generated paths, this one flagged
name, zero symlinks and zero special files; the flagged file remained unopened.

After that failure, the writer independently verified all 67 source inputs, 101
pinned tool/support paths, relevant library sets, 47 selected build-script inputs,
runner/pin hashes and Cargo configuration absence. Actual user, branch, HEAD and
all recorded local/remote-tracking refs matched the pre-build record. Thus source
and tool preservation is independent post-failure evidence, not an assertion that
the failed runner completed its own preservation stage. The failed directory was
left untouched. Read-only LLVM inspection logs and separate preservation/ELF
observations were written in a new private directory:
`/tmp/piv1-keyless-sbf-first-attempt-inspection-20260909-a`.

The two static commands, both exit zero with empty stderr, used the same complete
fixed environment and repository cwd recorded for the build:

```text
/home/jerem/.cache/solana/v1.54/platform-tools/llvm/bin/llvm-readelf --file-header --program-headers --sections --symbols --dyn-syms --relocations /tmp/piv1-keyless-sbf-build-20260909-a/target/sbpf-solana-solana/release/piv1.so
/home/jerem/.cache/solana/v1.54/platform-tools/llvm/bin/llvm-objdump --disassemble --demangle /tmp/piv1-keyless-sbf-build-20260909-a/target/sbpf-solana-solana/release/piv1.so
```

The original unstripped artifact is 176296 bytes, SHA-256
`a9c856b3cffb1005bfd7ebcbb24d1fcd6b900674f2f50f38d9609777e0ad8138`.
Its actual header is ELF64, little-endian, DYN/type 3, version 1,
**machine 263 (EM_SBPF)**, flags zero. The frozen parser's deliberately narrow
machine-247 expectation also rejects this actual output; it was not changed or
bypassed to report a pass. Complete raw inspection records the unexpected identity.
This is a parser expectation to review against the pinned compiler/loader sources,
not a reason to patch an ELF header or alter the selected target.

Static facts independent of that mismatch:

- Entry `0x104e8` is a global, default-visibility function in both `.dynsym` and
  `.symtab`, size 448, section `.text`. The `.text` interval begins at `0x120`,
  is 124200 bytes long, and is executable. Entry offset `0x103c8` (66504 bytes)
  is divisible by eight and maps to the same file offset in executable PT_LOAD
  segment 0. Symbol presence alone was not used as entry-location evidence.
- The 12 sections and four program headers are recorded in `elf.stdout`.
  `.rel.dyn` contains 411 relocations: 397 `R_SBF_64_RELATIVE` (type 8) and 14
  `R_SBF_64_32` (type 10). No loader or relocation application was executed.
- Nine dynamic unresolved symbols are recorded: `abort`, `sol_log_`,
  `sol_memcmp_`, `sol_memcpy_`, `sol_log_data`, `sol_invoke_signed_rust`,
  `sol_get_rent_sysvar`, `sol_memset_`, and `sol_try_find_program_address`.
  Their presence is not syscall-resolution or runtime-execution evidence.
- The linked claim function survives at `0x6828`. Actual disassembly contains
  `stxdw [r10 - 0x29a8], r1` at `0x72d0` and the corresponding load at `0x7ba8`:
  a direct offset of 10664 bytes, independently corroborating the reported
  oversized claim frame. This direct-offset observation is not an alternative
  complete frame-size calculation. The two other diagnostic function names are
  absent from the linked artifact's symbols; their compiler errors remain failures
  for this compilation and are not discarded as irrelevant.

The source call path remains dispatch → `execute_kif_claim` → pre-authentication,
claim preparation, four encoded envelopes, native borrow preflight, two-state
persistence, fixed signed System CPI, fresh authentication/postchecks and pure
plan commit → factual event. Source and pinned dependency inspection also identify
limitations that no artifact inspection resolves: the default entrypoint allocates
its `Vec<AccountInfo>` before the exact-five dispatch check; the default bump
allocator does not free allocations; the four envelope buffer requests total 2196
bytes, which is not total heap use; and the event reserves 256 bytes after execution,
so an allocation abort at that point still needs runtime transaction rollback.
Anchor 0.32.1 reexports `solana-invoke` 0.4.0 for this path, whose
`StableInstructionBorrowed` borrows existing instruction buffers. The unrelated
`solana-cpi` cloning path is not substituted into this explanation. These are pinned
source observations; safe heap, maximum call depth, compute use and runtime rollback
have not been measured. Root/reviewer independently traced the same dependencies.

| Retained evidence | SHA-256 |
| --- | --- |
| Original `build.stderr` | `801c3359bd0066a8164c05e9e1ac571ceac0d9b1d9188f60874f2a6f6167fe88` |
| Original failed `result.json` | `ee57ae1f14ff86e19a91ad55038a0ffc5d02a651d4fb68d91b1453752b6416d1` |
| Separate `inspection.json` | `bcef199866a3040054e177c680eddcde7da15aff31aa3abcffba766fcbedf3d7` |
| Separate `elf-static-observations.json` | `0ca2f48f2944d79b3d5372d5adaf9cdc82e0d7baad2d057e27ed576426e8dea0` |

During read-only source tracing, one exploratory `rg` command used three nonexistent
module paths and exited 2; `rg --files` located the actual modules, which were then
read. This did not change the build or its evidence. No host tests or additional
builds were run. The runner/pins retain their exact reviewed hashes; only this
writer evidence was appended after the build, preserving pilot report additions.

The writer **releases the build slot and report ownership** to the pilot. A separately
reviewed compatible amendment is required before runner filename/diagnostic-retention
and ELF-expectation corrections or source frame reductions. No correction or retry
is authorized by this evidence append itself. HEAD remains
`14106d664c107b3a2f705ac87361768af42d0786`; accepted main and protected refs remain
unchanged. No Git mutation, key/secret access or creation, signing, validator,
SBF/runtime execution, deployment, fund movement, live blockchain action, Mainnet
action or authority transfer occurred. The first attempt remains failed validation.

## Compatible correction amendment — reviewed and released

The pilot independently inspected the seven raw compiler diagnostics, original
failed result, artifact hash/header, actual excessive stack instruction, and
writer static evidence. It reverified every pinned compiled input/tool identity,
runner/pin hash and protected Git ref after the failed build. It independently
read the relevant cached entrypoint/allocator/event/invocation sources and
verified the reviewer's nine source hashes; temporary inventory:
`/tmp/piv1-t212-pilot-call-path-source-hashes.json`.

Separate reviewer `review_t23_final` returned PASS for this compatible correction
scope, including the classifier amendment. No economic or authority decision is
needed. The sole writer may now edit only the runner/pins, a reusable stdlib
runner regression test file, the affected authentication/claim implementation
and corresponding Rust regressions, and this report. Preserve all existing tests
and independent custody oracles. Root retains shared checkpoint documents.

Runner correction requirements:

- Exempt only the ordinary regular Cargo fingerprint metadata for locked
  `solana-sysvar-id 2.2.1`, with its exact basename and restricted fingerprint
  path structure. Retain sensitive-name checks elsewhere and reject symlinks,
  nonregular objects and exception near-misses without reading them.
- Record compiler exit/diagnostics, output-audit failure and input/Git preservation
  independently. Read only known diagnostic logs after regular-file checks. Any
  failed audit or stack/call-frame diagnostic must make the run fail, including
  Cargo zero. Add a combined-failure regression retaining all those facts.
- Accept the explicit ELF machine set `{247 (EM_BPF), 263 (EM_SBPF)}` and record
  the actual value. The primary reference's `validate` accepts both:
  [sbpf v0.12.2 elf.rs](https://github.com/anza-xyz/sbpf/blob/v0.12.2/src/elf.rs#L731).
  This reference is not a selected runtime dependency. Keep flags zero, ELF64LE,
  DYN/version, bounds, exact exported entry and executable file mapping checks.
  Add synthetic accepted-machine and rejected-machine cases. Never patch headers.
- Preserve the reviewed tool/build-script pins and commands/environment/profile.
  Refresh only source hashes for the exact reviewed compatible changes, retaining
  baseline ancestry and the original attempt evidence. Present new runner/pin
  hashes and exact next command for separate pre-execution review before retry.

Stack correction requirements:

- Reduce the three evidenced oversized frames with the smallest coherent private
  helper/refactoring boundaries. Start with non-inlined decoding/validation,
  reward decoding, claim preparation and post-transfer verification; pass existing
  large values by reference and separate scratch lifetimes. Do not add heap
  allocations as an unmeasured workaround, a large substitute aggregate, unsafe
  code, dependency changes, limit overrides, feature exclusions or profile changes.
- Preserve public signatures, serialized layouts, PDA and account checks, error
  identities and precedence, checked arithmetic, economics, native simultaneous
  borrow preflight, complete CEI writes before the one fixed CPI, fresh actual
  post-authentication, full byte and state-lamport comparisons, pure `plan.commit`,
  success event and transaction-error propagation. No other instruction is added.
- Guardian snapshots authenticate all six reward envelopes, propagate their errors
  in slot order, then validate membership. Preserve that two-phase ordering; add a
  mixed-fault regression against an earlier binding error and later malformed
  envelope. Do not combine authentication and binding in one loop.
- Claim postchecks retain fresh authentication → custody equality → full bytes →
  state lamports → `plan.commit` → decoded-state equality. Add relevant mixed-fault
  precedence regressions while retaining raw-failure/rollback and original-audit
  tests. A later helper boundary must not omit any check or fabricate observations.

The writer receives the focused host validation slot for these corrections only.
No second target build is released yet. Freeze corrected source/runner/pins and
report actual commands/results; separate review precedes the next fresh locked,
offline target build. Final completion requires all applicable host regressions,
checks/docs on final source, zero stack/call-frame diagnostics across the compiled
library, independent artifact/entry/stack/call-depth observations, and separate
final source/evidence PASS. Non-inlined helper calls do not prove runtime safety.
Keep the first failed artifact/output intact. No live/sensitive gate changes.

## Writer compatible correction candidate — host validation and next freeze

The writer implemented the released amendment and froze the candidate without a
second target attempt. These are candidate frame reductions; host tests do not
establish that the optimized SBF frames fit.

Five Rust source files change only helper boundaries and inline policy:
`accounts.rs`, `guardian_clock_accounts.rs`, `kif_claim_accounts.rs`,
`kif_claim_execution.rs`, and `state/kif_claim.rs`. Shared state decoding and reward
authentication are non-inlined. Borrowed private helpers keep Config/PDA validation
scratch outside the large owned authentication result frames. Snapshot reward
envelope/PDA authentication still runs for all six accounts before errors are
propagated in slot order and before any membership check. The claim path separates
fresh post-transfer authentication/comparison and final plan commit/comparison;
claim preparation/commit also have explicit non-inlined boundaries. No new heap
allocation, aggregate state substitute or unsafe code was introduced.

The original claim order remains fresh authentication → custody equality → full
replacement bytes → state-account lamports → accepted pure `plan.commit` →
decoded-state equality. Native simultaneous borrow preflight, CEI two-account
persistence, the fixed single signed System CPI, error propagation and event timing
are unchanged. Public signatures, account layouts, accepted validators/arithmetic,
serialized payloads and all independent host custody/audit oracles remain intact.

Two added guardian tests pin the two-phase envelope/binding precedence and slot
error order. One added claim test covers malformed authentication plus incorrect
payment, incorrect payment plus valid state-byte/lamport tampering, and valid
state-byte plus lamport tampering. It checks the original error precedence, raw
effects after failure and modeled full rollback with the original audit unchanged.
All earlier functional assertions and tests remain present.

The corrected runner permits only the regular generated basename
`lib-solana_sysvar_id.json` beneath the exact target release fingerprint structure
and a `solana-sysvar-id-` directory with sixteen lowercase hexadecimal characters.
The unchanged pinned lock fixes that package to 2.2.1. Similar names/paths, symlinks,
directories and FIFOs remain rejected without payload reads. Diagnostic logs are
read only after regular-file checks; compiler diagnostics, output-audit failure,
input/ref preservation and log capture now have independent durable outcomes.
An output audit immediately after preflight prevents unexpected preflight outputs
from reaching the target build. Stack/call-frame errors still fail with Cargo zero.
The static parser accepts only the explicit machine set `{247, 263}` and records
its actual value while preserving every other type/flags/entry/mapping/bounds guard.

`tools/test_build_keyless_sbf.py` is a reusable stdlib-only suite. It uses synthetic
ELF bytes, harmless marker files and an explicitly mocked compiler stage; it never
runs a target compiler or runtime. Combined-failure cases retain compiler zero,
stack/call-frame diagnostics, the failed audit, known-log hashes and either successful
preservation or a distinct preservation failure. Additional tests pin exact/near-miss
fingerprint paths, nonregular/symlink refusal, sensitive-name refusal without reads,
preflight audit rejection before any build, actual machine recording, ELF bounds,
existing output preservation, review-hash refusal and malformed/nonregular pins.

Commands/results during this correction:

```sh
/home/jerem/.cargo/bin/cargo +1.97.1 test -p piv1 --test account_authentication --test guardian_clock_authentication --test isolated_kif_claims --test kif_claim_execution --test kif_claim_instruction --test state_persistence --locked --offline
/usr/bin/python3 -I tools/test_build_keyless_sbf.py
/usr/bin/python3 -I tools/build_keyless_sbf.py --output /tmp/piv1-keyless-sbf-correction-preflight-20260909-a
```

The focused Cargo command passed on its first attempt: **130 tests** (23 fixed
account authentication, 24 guardian/Clock, 24 isolated claims, 21 execution,
17 instruction boundary, 21 persistence), zero failures and no warnings/errors.
No Rust source changed after this host run. The Python suite passed 13 tests first,
14 after the preflight-audit guard/regression, and **16 tests** after adding durable
malformed/nonregular pin regressions. These are successive passing versions, not
cumulative independent test counts. Python syntax checks passed without a bytecode
cache. Several exploratory `sed` reads used nonexistent test paths and exited 2;
`rg --files` located the actual targets before they were read/edited. No compilation
or test failed during this correction. No formatting-tool pass is claimed.

The final corrected preflight passed with `target_build_started=false`: all 26
version/dependency/metadata commands exited zero and all stderr logs were empty.
It checked actual source/tool identities and preserved input/ref state. Evidence:
`/tmp/piv1-keyless-sbf-correction-preflight-20260909-a`; result SHA-256
`a3a5523ba935a445fd1b1cba3a9782ad6d5bec2e44b7c951184f0542bdb7a0b3`.
Only the five changed Rust source and two changed Rust test hash entries were
refreshed in the companion. Every tool, selected build-script, provenance, library,
baseline and other input pin is identical to the original first-attempt inventory.

The ten-file source/test/runner freeze is recorded at
`/tmp/piv1-keyless-correction-source-freeze-20260909-a.json`, SHA-256
`fffc2dfca0a518de0791ab6d3113a7cd110e18577f7c70153852cb865b3f6733`:

| Frozen file | SHA-256 |
| --- | --- |
| `programs/piv1/src/accounts.rs` | `46a0a15c4bf8d01538b8d2c3ce76b09e1a994f9863e286881d891acfa071e245` |
| `programs/piv1/src/guardian_clock_accounts.rs` | `482c42132c795295341b2785849482797a2ac86844e2c1fe365508964b8ed521` |
| `programs/piv1/src/kif_claim_accounts.rs` | `0d4e546ed6973c994a57e7d895a3d45907113ba3f1d87fc07d7c2e3cb3f76f4d` |
| `programs/piv1/src/kif_claim_execution.rs` | `2dcca8085899fcd318ea85131cac13153c93f5f0df1e624742632e27102d5dca` |
| `programs/piv1/src/state/kif_claim.rs` | `5b495dd9b1756d8ac01ea26a50501acd34ed6ee0cb28a89a7836aa83bae3ebf4` |
| `programs/piv1/tests/guardian_clock_authentication.rs` | `c639d07e01dc341542db65afa31972968a31c736170f5ed3246b29bc8753fcc3` |
| `programs/piv1/tests/kif_claim_execution.rs` | `9c4d63c9f258da55235f590ba03bb465ab9dee9a78bf295efc4026a6ade80166` |
| `tools/build_keyless_sbf.py` | `e4455e55a88fe5b22796cc21ef1a938522a5cf55cc226bee3570df9756bf3cb8` |
| `tools/keyless_sbf_pins.json` | `dd8cfab52f7adb04f7fbf9a8f856542020addfdb9bf10d51ec2c564124e1ce3d` |
| `tools/test_build_keyless_sbf.py` | `911928f9a8101a79d2b9d2642effb8fceeac0f250955020ed6692605cd712d80` |

The next exact command is **proposed only**, pending separate review of this
corrected freeze and an explicit target build-slot release:

```sh
/usr/bin/python3 -I /home/jerem/piv1/tools/build_keyless_sbf.py \
  --output /tmp/piv1-keyless-sbf-build-20260909-b \
  --execute \
  --approved-runner-sha256 e4455e55a88fe5b22796cc21ef1a938522a5cf55cc226bee3570df9756bf3cb8 \
  --approved-pins-sha256 dd8cfab52f7adb04f7fbf9a8f856542020addfdb9bf10d51ec2c564124e1ce3d
```

Compiler argv, fixed environment, default features, v0 flags and release profile
remain as reviewed for the first attempt, with only the fresh output-root name
changing from `...-a` to `...-b` in `--target-dir` and `TMPDIR`. The first failed
output/artifact/result remains unchanged and failed. No artifact/header patch,
stack-size limit change, diagnostic suppression, dependency or manifest/toolchain
change occurred. Actual optimized frame sizes, call depth and runtime resource
safety await the separately reviewed target retry and later runtime evidence.

The writer releases the focused **host** validation slot and report ownership.
No target retry slot was used or assumed. No Git mutation/commit/push, secrets/key
access or creation, signing, validator/SBF/runtime execution, deployment, live
blockchain action, Mainnet action, fund movement or authority transfer occurred.
HEAD remains `14106d664c107b3a2f705ac87361768af42d0786`; the shared pilot document
changes are preserved and accepted main/protected refs remain unchanged.

## Pilot corrected-candidate inspection and independent checks

The pilot read the five complete Rust refactor diffs, three new mixed-fault tests,
corrected runner and all 16 reusable stdlib tests, then verified the complete
ten-file freeze against actual files. Only the seven listed Rust hash entries
changed in the source pin companion; every other pin field remains identical to
the first-attempt `inputs-before.json`. No additional source finding was identified.
This source/host inspection does not establish successful target frame reduction.

Actual pilot executions on that freeze:

```sh
/usr/bin/python3 -I tools/test_build_keyless_sbf.py
/usr/bin/python3 -I tools/build_keyless_sbf.py --output /tmp/piv1-keyless-sbf-pilot-correction-preflight-20260909-a
```

All **16 stdlib tests passed**. Independent corrected preflight returned
`PREFLIGHT_PASS_NO_TARGET_BUILD`: 26 commands exited zero, every stderr was empty,
all recorded log hashes matched actual files, and complete before/after Git state
and source/tool/runner/pin hashes were preserved. Result JSON SHA-256:
`5c35e45edaa5ef84a4179a299168517fd1bc8972f040a8586d171a6ed232128a`.
The 130 focused Rust results above remain writer execution evidence; the pilot's
final broad host gates will run on the successful frozen target candidate.

The pilot independently analyzed the first failed artifact using an ephemeral
read-only script, `/tmp/piv1-t212-pilot-static-analysis.py`, and saved observations
at `/tmp/piv1-t212-pilot-first-artifact-observations-v2.json`. The separate reviewer
validated the bounded method and independently checked 171 unique labels,
15027 parsed instructions, all 542 direct CALL immediates and 14 CALL relocations
against actual bytes. The pilot added explicit flags-zero, byte-correspondence,
parse-coverage and relocation checks before reuse. An initial helper ELF-header
tuple index error failed before output, was corrected, and the actual rerun passed.

The first artifact has a longest path of 15 functions in the **partial syntactic
direct-call graph**, 86 directly reachable labels, 10 reachable indirect calls and
13 reachable unresolved direct calls. This is neither an observed runtime depth
nor a proven feasible path. Indirect/relocated/runtime paths and register/pointer
aliases are excluded. The 10664-byte direct stack access confirms the KIF defect;
small direct offsets cannot prove safe complete frames. No artifact was loaded,
relocated, modified or executed. Final artifact observations must repeat these
checks and retain all limitations. No sensitive action occurred.

## Corrected pre-execution review and second-build release

Separate reviewer `review_t23_final` returned PRE-EXECUTION PASS, no actionable
findings, for the exact corrected `...-20260909-b` command. It read the complete
seven-file Rust diff, all three new Rust tests, runner, all 16 Python tests and
final report evidence; verified the ten frozen hashes and unchanged non-source
pins. It also inspected the pilot's actual successful preflight/log hashes.
Reviewed report hash before this verdict append:
`3ca1a75d0ac6a0ed33abd8ffe84d3083588b151ec3965342bd4e7888e05b02d5`.
The pilot releases only the sole writer's second target build slot for that exact
command. Source/runner/pins remain frozen. Inspect actual diagnostics before any
further correction; the previous failed artifact stays failed and unchanged.

## Writer second target attempt — frame failures remain

After the second exact PRE-EXECUTION PASS and sole target-slot release, the writer
verified all ten frozen source/test/runner files, all 67 compiled input hashes,
`jerem`, `integration/piv1-testnet`, unchanged HEAD and absence of output `...-b`.
The exact second command above ran once, with no source/runner/pin/Git mutation.
Cargo ran from `2026-09-09T12:48:58.640155Z` to
`2026-09-09T12:54:39.986319Z` (341.346 seconds), exited zero and wrote a new artifact.
The corrected runner exited **1 / FAIL**, correctly retaining thirteen compiler
errors despite Cargo zero:

| Function | First estimated frame | Second estimated frame | Remaining excess over 4096 |
| --- | ---: | ---: | ---: |
| `accounts::authenticate_fixed_accounts` | 5248 | 5120 | 1024 |
| `guardian_clock_accounts::authenticate_guardian_clock_snapshot` | 5632 | 4800 | 704 |
| `kif_claim_execution::execute_kif_claim` | 10688 | 9280 | 5184 |

The other ten diagnostics report claim function calls overwriting frame values.
The first attempt had four such diagnostics. Smaller estimated frames therefore
do not constitute a successful correction or runtime safety evidence. Full error
text and original line numbers are in `result.json`, independently classified from
the complete 8147-byte `build.stderr`; stdout is empty.

The corrected evidence path worked as intended: all 26 preflight commands passed,
the output audit passed without opening the ordinary Cargo fingerprint payload,
and source/tool/runner/pin/protected-ref preservation passed independently of the
compiler failure. No audit, preservation or log-capture error is recorded. Since
target diagnostics failed, the runner did not proceed to LLVM artifact inspection.
No failed artifact was relabeled as a pass and no flags/limits were relaxed.

The generated original artifact is retained at
`/tmp/piv1-keyless-sbf-build-20260909-b/target/sbpf-solana-solana/release/piv1.so`,
179488 bytes, SHA-256
`a46bc2b732361e0e14a4c6bcaff472dab0d980f3c9a522ef4069d049a2fbf541`.
Its hash/size are identity evidence only; no successful ELF/stack/call-depth or
runtime validation is claimed for this artifact. Exact retained second-attempt
metadata is also summarized at `/tmp/piv1-keyless-second-attempt-summary-20260909.json`.

| Second-attempt evidence | SHA-256 |
| --- | --- |
| `build.stderr` | `6557d5c82801cff3f039cffdae74d51140d3e485731789172b2e226b31d86948` |
| Original failed `result.json` | `2c40340ea5a72cd534e21fb16803533354b6f4e32b6bef524ab73356e8c3cb4a` |

The writer independently rechecked the ten corrected freeze hashes and the first
attempt's result/log/artifact hashes after the second run; all remain unchanged.
Both failed output directories are retained without reuse or alteration. Only
this evidence was appended to the task report, preserving the pilot's review
sections. No additional host suite, target retry or source edit was performed.

The writer **releases the target build slot and report ownership**. The remaining
frame errors were reported to the pilot before any further changes; another bounded
reviewed correction/retry dispatch is needed. No target/runtime execution, keys,
secrets, signing, validator, deployment, live blockchain or Mainnet action, fund
movement, authority transfer or Git mutation occurred. HEAD and protected refs
remain unchanged. Task 2.12 is still unresolved; the second attempt is failed
validation, with complete diagnostic and preservation evidence.

## Bounded private-field boxing amendment — reviewed and released

The pilot independently verified the second result, all thirteen raw diagnostics,
log hashes and successful audit/input/ref preservation. Helper-only splitting
improved frames but did not resolve the large owned aggregate/Result copies.
Writer and separate reviewer independently assessed the remaining code read-only
and recommended the same private-field boxing design. Separate design review is
PASS; this is a compatible memory representation correction, not an economic,
authority, account-storage or instruction-ABI decision.

The sole writer may now make these bounded changes:

- Store Config in a private `Box<PivConfig>` field in each of the three owned
  authentication snapshots; also box the fixed snapshot's ActiveDistribution.
  Keep their public types, functions and getters unchanged. Decode `Box<T>` inside
  the existing non-inlined `decode_state` boundary, keeping large `Result<T>`
  values out of callers. Pinned Borsh still materializes T inside the decoder;
  this is not stackless deserialization and requires fresh target diagnostics.
- Box only the private before/next Config fields in `PreparedKifClaim`. Allocate
  fixed Config clones inside a small non-inlined helper as needed. Preserve every
  full value comparison and move `*boxed_next_config` into the supplied Config at
  commit. Do not add a production plan clone or a commit-time allocation.
- Add one small private compile-time memory-budget module if useful, with its
  module declaration and report evidence. Do not change serialized payload types,
  layouts, discriminators, public signatures, errors, error order, custody oracles,
  CEI, fresh postchecks, event timing, checked arithmetic or economics. Existing
  clone/replay/owned-snapshot/serialization/mixed-fault/rollback regressions remain
  mandatory. Add a regression only where an actual ownership/value gap remains.
- Keep all runner/tool/lock/profile/allocator/heap-limit settings unchanged.
  Refresh only the exact source hash entries/set after reviewable edits. This
  amendment expressly permits the fixed allocations below; it does not authorize
  an arbitrary heap workaround, new dependency, unsafe code or limit override.

The writer and pilot independently inspected existing HOST DWARF in
`target/debug/deps/libpiv1.rlib`, SHA-256
`188d186885e820d99b43d0f13769575e975f4db9873300244111a6212076f3ec`.
Sizes/alignment: Config 1024/8, ActiveDistribution 896/8, GuardianReward 88/8,
Registry 208/8, fixed snapshot 2064/8, guardian snapshot 1832/8, claim snapshot
1136/8, prepared claim 2328/8 and AccountInfo 48/8. These are host observations.
Require compile-time assertions on the actual target for the budget's assumptions:
Config 1024/8, ActiveDistribution 896/8, AccountInfo 48/8, AccountMeta 34/1,
pointer/usize size and alignment eight, and RefCell native/slice payloads 16/8
and 24/8. No target layout is inferred merely from the host metadata.

The normal accepted five-account claim adds four Config allocations: pre-auth
snapshot, two plan Configs before CPI, and fresh post-auth Config after CPI.
An additional plan clone would allocate two more Configs; the production path
does not clone its plan. No allocations are reclaimed by the default bump allocator.

| Listed normal-claim allocations | Requested bytes |
| --- | ---: |
| Four Config boxes | 4096 |
| Four existing state-envelope buffers | 2196 |
| Five AccountInfos in the entrypoint vector | 240 |
| Five native and five data Rc allocations | 360 |
| Two System metas and serialized transfer data | 80 |
| Existing Anchor event buffer | 256 |
| Total listed requests | 7228 |

There are 22 listed allocations. Fifteen require alignment eight; the remainder
are byte-aligned. The separate review's tight padding allowance plus the eight-
byte bump cursor yields 7341 bytes. A simpler conservative allowance of seven
bytes for every listed allocation yields 7390 bytes. Use an **8192-byte source
budget ceiling for this inventory**, enforced with the target layout assertions;
this does not change the existing 32768-byte heap configuration. This bounds only
the listed normal-claim requests/padding, not measured total heap consumption.
Additional caller snapshot/plan clones, invalid-account/error/panic paths,
oversized entrypoint account lists and runtime-internal costs are excluded.

The reviewer verified the Rc representation against the exact target-platform
Rust revision, not the host toolchain. The pilot independently read/hashed the
cached sources. [Pinned RcInner definition](https://github.com/anza-xyz/rust/blob/daa3af4a1110ec3f10ce08083bb0b7855a88416f/library/alloc/src/rc.rs#L281)
is `repr(C)` with two usize Cell headers and its value. Given target RefCell
assertions, the two requests are 32/40 bytes. Target-cache SHA-256 identities:

| Source | SHA-256 |
| --- | --- |
| `library/alloc/src/rc.rs` | `d504d52fb27ca4783ae9ad9049ac2750a983bbd93405520610271c761aefa16b` |
| `library/core/src/cell.rs` | `f456f9c081543626b06875266590282abe11d5f4da5d62bfd2b3668fbb88f939` |
| `library/alloc/src/boxed.rs` | `079d731f42a02e9691b8e5502dcb79b15f3a6635531f47845037f1f1024dcd0c` |

Standard Box allocation can abort on exhaustion; no stable fallible Box allocator
is introduced or recoverable PIV1 allocation error claimed. Fresh authentication
and event allocation after CPI still depend on transaction rollback. No runtime
OOM, total heap, compute or complete call-depth evidence exists yet.

The writer receives the focused HOST validation slot only for these changes.
Freeze and separately review the final source/pins and exact next command before
a third fresh target attempt. Final host gates and zero target stack/call-frame
diagnostics with independent artifact inspection remain mandatory. Keep both
failed outputs intact. No sensitive/live authorization or founder acceptance.

## Writer boxed candidate — host validation and third-command preparation

The writer read the complete released boxing amendment and reverified `jerem`,
`integration/piv1-testnet`, HEAD `14106d664c107b3a2f705ac87361768af42d0786`
and the expected pilot-owned documentation changes. No Git mutation occurred.
This append records an implemented and host-tested candidate, not a third target
build or successful correction of the two failed artifacts.

The three authentication snapshots now privately own `Box<PivConfig>`; the fixed
snapshot also owns `Box<ActiveDistribution>`. Their public signatures/getters and
value equality remain unchanged. The existing non-inlined `decode_state` returns
`Box<T>` directly for those fields, retaining its complete owner/executable/data
borrow/size/discriminator/Borsh/padding/rent order. This still materializes T inside
the pinned Borsh decoder; target frame diagnostics remain necessary.

`PreparedKifClaim` now privately boxes only its before/next Config. A non-inlined
fixed Config clone helper allocates at the original preparation clone sites.
Every validation, arithmetic operation, full before-state comparison and error
precedence remains in order. Commit moves `*next_config` into the supplied state;
it adds no allocation. The production execution path does not clone the plan.
The previous execution/refactoring and mixed-fault tests are unchanged by this
amendment, as are CEI, fresh postchecks, all serialized payload definitions,
public instruction/errors/events, custody fixtures and original audit equations.

The new private `allocation_budget` module and its `lib.rs` declaration assert
Config/ActiveDistribution/AccountInfo/AccountMeta/pointer/usize/Box/RefCell layouts
on whichever target is compiled. The normal five-account claim requests four
Config boxes, three before CPI and one during post-CPI authentication: 4096 bytes.
Together with the existing listed buffers/AccountInfo/Rc/instruction/event
allocations, the const inventory enforces 7228 requested bytes, 22 allocations,
7390 bytes including conservative padding/cursor, and an 8192-byte ceiling.
The actual 32768-byte heap limit and allocator remain unchanged. These are source
inventory assertions, not measured total heap consumption; they have passed on
the host only so far. The target compiler must independently evaluate them.
Additional caller clones, oversized entrypoint lists, invalid/error/panic paths
and runtime-internal costs remain excluded. No-op deallocation is counted;
Box exhaustion can abort, and post-CPI allocation still needs transaction rollback.

One focused regression was added to `isolated_kif_claims.rs`: cloned snapshots
remain independent after the original is dropped and actual Config bytes change;
cloned plans commit into separate supplied state pairs with equal exact serialized
results after the source snapshot is dropped. Actual fixture bytes, lamports and
its original audit baseline remain unchanged. These supplied after-observations
are pure value evidence, not a transfer receipt or runtime rollback evidence.
Existing replay, owned-snapshot, serialized-envelope, precedence and rollback
regressions were retained and executed.

Actual focused command, once on this candidate:

```text
/home/jerem/.cargo/bin/cargo +1.97.1 test -p piv1 --test account_authentication --test guardian_clock_authentication --test isolated_kif_claims --test kif_claim_execution --test kif_claim_instruction --test state_persistence --locked --offline
```

**PASS: 131 tests**, comprising 23 fixed-auth, 24 guardian/Clock, 25 isolated-claim,
21 execution, 17 instruction and 21 persistence tests; no failures, ignored tests,
warnings or errors. The first build succeeded (4.08 seconds build profile output).
`git diff --check` also passed. No formatting component was installed and no
rustfmt pass is claimed. No broad workspace repetition was performed here.

Only seven compiled source entries changed from the second target candidate:
the four boxed-ownership modules, `lib.rs`, the new private budget module and the
new isolated-claim test. The pinned source set now contains 68 files. All non-source
pin data, original reviewed tools, resolved tools/libraries, build-script hashes,
runner and its 16-test reusable Python suite remain byte-for-byte unchanged.
The unchanged Python suite was not rerun by this writer during this amendment.

Default preflight command, once and without `--execute`:

```text
/usr/bin/python3 -I /home/jerem/piv1/tools/build_keyless_sbf.py --output /tmp/piv1-keyless-sbf-boxing-preflight-20260909-a
```

**PREFLIGHT_PASS_NO_TARGET_BUILD**: all 26 read-only commands returned zero, all
stderr logs are empty, output audit and source/tool/runner/pin/ref preservation
passed. Complete exact argv/environment are retained in `proposed-build.json` and
`result.json`; default features, v0 target, release profile, offline/locked flags,
fixed tools and clean environment remain unchanged. Result SHA-256:
`8ac5eba8bf9f786fe3f03ef54eba8b04be6137989cd18b4812cf14f3e1dcbf6c`.
A follow-up read-only evidence check initially used the nonexistent `returncode`
field and failed with `KeyError`; correcting it to the actual `exit_code` field
passed all 26-status, stderr, preservation and prior-artifact checks. This was an
inspection-script error, not a compiler/preflight failure; no runner or output was
modified and no build was retried.

The writer independently rechecked both failed attempts' original result, stderr
and artifact hashes: all unchanged. All thirteen frozen candidate files match
`/tmp/piv1-keyless-boxing-source-freeze-20260909-a.json`, SHA-256
`dff39a713cecbc1c9d30fab352d61a493b9016ef96bda48aece05370dd2a7b56`:

| Frozen writer file | SHA-256 |
| --- | --- |
| `programs/piv1/src/accounts.rs` | `0989cb76ac10ba8dc2c654c2c600a80d1c1dfd5f1f2da4b5ea1a4d5334be7467` |
| `programs/piv1/src/allocation_budget.rs` | `2a6324e3cef78fdc32959c60b227fc95e81bb6a5063ae922a852a20bf978cc25` |
| `programs/piv1/src/guardian_clock_accounts.rs` | `6d4b48d787c3eed6acc181b5d2389a4e5bf02c32dca7eb2c9a907e0597ec9448` |
| `programs/piv1/src/kif_claim_accounts.rs` | `9259221886dd4183b469d79d4860b17c1de7e274563ae07b65a532ecf8b08493` |
| `programs/piv1/src/kif_claim_execution.rs` | `2dcca8085899fcd318ea85131cac13153c93f5f0df1e624742632e27102d5dca` |
| `programs/piv1/src/lib.rs` | `88ec4c149f4fe481352d25ab638a25fb8d76857773bd049ae94a8d11671bdbdb` |
| `programs/piv1/src/state/kif_claim.rs` | `3e58b3554233774f8f244e2771ffb492b98e2073b802208c8c08704659af7014` |
| `programs/piv1/tests/guardian_clock_authentication.rs` | `c639d07e01dc341542db65afa31972968a31c736170f5ed3246b29bc8753fcc3` |
| `programs/piv1/tests/isolated_kif_claims.rs` | `98fb1954b080b0ff672937ba7523e760455832387928970ea31b5dc6237611db` |
| `programs/piv1/tests/kif_claim_execution.rs` | `9c4d63c9f258da55235f590ba03bb465ab9dee9a78bf295efc4026a6ade80166` |
| `tools/build_keyless_sbf.py` | `e4455e55a88fe5b22796cc21ef1a938522a5cf55cc226bee3570df9756bf3cb8` |
| `tools/keyless_sbf_pins.json` | `c62a97d3a2b68cbabd9c5d828b39b04ac8ea27a40ed361c0701f9db73fba8963` |
| `tools/test_build_keyless_sbf.py` | `911928f9a8101a79d2b9d2642effb8fceeac0f250955020ed6692605cd712d80` |

Exact proposed third command, **NOT EXECUTED**, pending separate pre-execution
review and explicit target-slot release:

```text
/usr/bin/python3 -I /home/jerem/piv1/tools/build_keyless_sbf.py --output /tmp/piv1-keyless-sbf-build-20260909-c --execute --approved-runner-sha256 e4455e55a88fe5b22796cc21ef1a938522a5cf55cc226bee3570df9756bf3cb8 --approved-pins-sha256 c62a97d3a2b68cbabd9c5d828b39b04ac8ea27a40ed361c0701f9db73fba8963
```

The proposed output path was confirmed absent; it must be checked again immediately
before a released target attempt. Source, runner and pins are frozen. The writer
**releases the focused host build slot and report ownership** to the pilot for
review/checkpoint. Git remains on the unchanged closure HEAD with only the scoped
writer changes and pilot-owned documentation; no commit or push was made.
Task 2.12 is still unresolved until actual target diagnostics and required final
inspection/gates pass. No target retry, artifact execution, SVM/validator, Anchor,
SBF wrapper, key creation, secrets, signing, deployment, live blockchain/Mainnet
action, fund movement or authority transfer occurred during this amendment.

## Writer third target attempt — static build pass

After the separate exact PRE-EXECUTION PASS and explicit third target-slot
release, the writer rechecked the thirteen-file freeze, all 68 compiled source
hashes, `jerem`, `integration/piv1-testnet`, unchanged closure HEAD and absence of
`/tmp/piv1-keyless-sbf-build-20260909-c`. The exact reviewed `...-c` command above
ran **once**, with runner `e4455e55a88fe5b22796cc21ef1a938522a5cf55cc226bee3570df9756bf3cb8`
and pins `c62a97d3a2b68cbabd9c5d828b39b04ac8ea27a40ed361c0701f9db73fba8963`.
No source, pin, tool, profile, dependency or Git mutation occurred through the run.

Cargo ran from `2026-09-09T13:21:50.796690Z` to
`2026-09-09T13:27:29.788129Z` (338.991 seconds), exited zero and reported the
optimized release build complete. The runner exited **0 / STATIC_BUILD_PASS**.
The writer read all 4394 bytes of original compiler stderr: only normal compilation
progress and the final completion line, with **zero warnings, compiler errors,
stack/frame exceedances or call-overwrite diagnostics**. Compiler stdout is empty.
The independent diagnostic classification in `result.json` is also empty. A broad
manual error/stack/frame search matched only ordinary dependency package names
containing `error`, not diagnostics. Unlike the first two attempts, Cargo zero
and the actual diagnostic evidence both pass.

All 29 recorded commands passed: 26 preflight commands, the single build and two
LLVM inspections. Output audit and source/tool/runner/pin/protected-ref preservation
passed, with no audit, preservation or log-capture error. The compile-time layout
and 8192-byte listed-allocation budget assertions now also passed on the actual
pinned SBF target. They remain source inventory evidence, not measured total heap
usage or a runtime OOM/compute/rollback guarantee.

The unchanged runner retained complete LLVM ELF metadata and disassembly, then
validated its bounded ELF checks. The actual artifact is ELF64 little-endian DYN,
machine 263 (`EM_SBPF`), flags zero. `entrypoint` is a global function at `0x10360`,
matching the ELF entry address inside executable `.text`; the section-relative
entry offset is `0x10240`, aligned to eight bytes and covered by executable LOAD
segment 0. The writer cross-checked these values against retained `elf.stdout`.
The raw relocation listing contains 407 entries and the symbol inventory retains
ten distinct unresolved names. No runtime syscall allowlist or loader compatibility
is asserted from those observations.

The artifact remains at:
`/tmp/piv1-keyless-sbf-build-20260909-c/target/sbpf-solana-solana/release/piv1.so`.
It is 176064 bytes, SHA-256
`0392bb822a3e767674ccd75486ad2685320bce5ffadb426ea8a93b08625bb6c8`.
No artifact was modified, loaded, relocated or executed. The pilot's independent
final artifact/call-graph inspection and broad host gates follow separately;
this writer result does not substitute for them or the final reviewer verdict.

| Retained third-attempt evidence | Bytes | SHA-256 |
| --- | ---: | --- |
| `result.json` | — | `56f086a69028fce3d0f722b0abc2521889571d251ffab0679a822952f1b57d6e` |
| `build.stderr` | 4394 | `8474ed99728e42d080af1b4584a9ec73f9b28a1ff158758d495ccda87e8d70d2` |
| `elf.stdout` | 85642 | `a92b09cd626eda20b4efe3e8032383642695d86bf17dc922144d93ff84c04644` |
| `disassembly.stdout` | 1071034 | `f5c42a131eee793b4a13cab4a42ec4401aaf54370bde0a61adbee159dd354199` |

The writer rechecked all thirteen frozen files and all 68 source inputs after
completion. Both original failed attempts' result, compiler log and artifact
hashes remain unchanged; their FAIL outcomes are preserved. Protected local and
remote-tracking refs still match the first/second attempts and closure HEAD remains
`14106d664c107b3a2f705ac87361768af42d0786`. Only this attributed evidence was
appended to the report, preserving the pilot's preceding review/release text.

The writer **released the target build slot immediately after completion and
now releases report ownership**. No further build, retry or code correction was
performed. The scoped writer changes and pilot documentation remain uncommitted;
no commit/push, key creation, secrets, signing, SVM/validator/runtime/CPI execution,
Anchor/SBF wrapper, deployment, live blockchain/Mainnet operation, fund movement
or authority transfer occurred. Static target compilation is now demonstrated;
final technical closure and all runtime/Testnet evidence remain separate gates.

## Pilot final validation — closure review pending

The pilot independently read the exact corrected ownership/helpers, private budget
module, new tests and unchanged surrounding claim/persistence/dispatch paths.
Separate pre-execution review of all thirteen frozen files and the exact third
command returned PASS, without findings. Writer focused results remain attributed:
the final 131-test output exists in its tool transcript, not saved shell logs.

The pilot personally verified the third result and all 58 retained log hashes,
29 zero-exit commands, zero warning/error/frame diagnostics, artifact identity,
actual ELF headers and all 68 compiled source hashes plus thirteen frozen writer
files. Evidence: `/tmp/piv1-t212-pilot-final-evidence-20260909-a.json`, generated
by the read-only `/tmp/piv1-t212-pilot-final-evidence.py`. All protected local and
remote-tracking refs remained unchanged. No remote publication is inferred from
that local ref check.

The separately reviewed disassembly method was repeated successfully on the final
artifact; every parsed instruction matched the actual ELF bytes, all instruction
lines/CALL syntax were covered and every CALL relocation matched immediate -1.
There are 174 function labels, 14973 instructions, 539 direct calls and 407
relocations (392 type 8, fifteen type 10). The partial direct graph reaches 89
functions and has a longest syntactic path of sixteen functions without a detected
cycle. Ten reachable indirect calls and fourteen unresolved direct calls are
excluded. This is neither an observed execution nor a complete runtime depth bound.

All directly observed negative r10 offsets are at most 4096. Claim execution's
largest observed direct offset is 4096; claim authentication 1016, fresh postcheck
296, commit-and-compare 1624, prepare 392 and the Config clone helper 288. These
are lower-bound direct access observations, not complete frame sizes or pointer
range proofs. Fixed/guardian snapshot authentication is not reachable from this
claim-only entrypoint; clean target compilation does not establish their runtime
resources. Evidence: `/tmp/piv1-t212-pilot-final-artifact-observations-20260909-a.json`.
No claimed safe margin is inferred merely from the absence of compiler errors.

The ten distinct unresolved symbols are `abort`, `sol_log_`, `sol_memcmp_`,
`sol_memcpy_`, `sol_log_data`, `sol_invoke_signed_rust`, `sol_get_rent_sysvar`,
`sol_memset_`, `sol_try_find_program_address` and `sol_memmove_`. The last is newly
present compared with the first artifact. A later pinned runtime must supply and
validate these; this task does not assert a runtime syscall allowlist or loader pass.

After writer build-slot release, the pilot ran all eight final host gates on the
unchanged candidate using `/home/jerem/.cargo/bin/cargo +1.97.1`, locked/offline:

```text
test --workspace --all-targets --locked --offline --quiet
test --workspace --doc --locked --offline
check --workspace --all-targets --locked --offline
check --workspace --all-targets --all-features --locked --offline
check -p piv1 --all-targets --features no-entrypoint --locked --offline
check -p piv1 --all-targets --features cpi --locked --offline
check -p piv1 --all-targets --features idl-build --locked --offline
doc --workspace --no-deps --locked --offline
```

The docs command used `RUSTDOCFLAGS='-D warnings'`. **339 workspace tests and one
doctest passed; all eight gates passed without warning/error diagnostics.** Logs,
commands/results, frozen input identities and independently hashed summary are in
`/tmp/piv1-t212-pilot-20260909T132833Z`. Summary SHA-256:
`dd11aed4571f1eecb7477cb1dc8f8c507afc2db0ccdb16d49b41819523be57af`.
All 77 broader source inputs stayed unchanged, including nine files in the
historical excluded spike; all 68 actual compiled inputs match the target pins.
Before these gates, the pilot's temporary launcher stopped before invoking Cargo
because it incorrectly compared the broader 77-file preservation inventory with
the 68-file target map. Read-only comparison showed only those nine excluded
historical paths and zero hash mismatches. The temporary launcher was corrected
to compare the exact compiled subtree while preserving all 77 files; the actual
gates then ran once. No product, runner, pin or test expectation was changed.

The pilot's earlier actual sixteen stdlib runner tests remain valid for the
byte-identical runner/test file. They model compiler output and filesystem failure
cases; they are not sixteen additional Rust/runtime tests. No unnecessary rerun
or formatting-component installation occurred.

The thirteen writer files are listed in the freeze above. Six pilot/report files
complete this task's documentation: `AGENTS.md`, `PIV1_CODEX_EXECUTION_PLAN.md`,
`PIV1_PILOT_STATE.md`, `PIV1_TEST_PLAN.md`, the Task 2.11 publication append and
this Task 2.12 report. A targeted scan of all nineteen publication candidates found
only regular jerem-owned files, no generated artifacts, private-key block or
matched credential-bearing value/URL. Effective `core.hooksPath` is unset,
`.git/hooks` contains samples only and no tracked `.github`/`.cargo` automation or
`.gitattributes` exists. This is a targeted check, not a general secrets audit.

Final separate source/evidence review is pending. HEAD is still the Task 2.11
closure, with the nineteen expected uncommitted files. No Mainnet action,
deployment, fund movement, key creation, secrets access, signing, authority
transfer, runtime execution or Git publication occurred in Task 2.12 so far.
Both failed attempts remain failed and untouched. The next justified dependency
after reviewed closure/publication is a separately scoped pinned keyless SBF
runtime harness for this claim boundary; no later implementation has started.

## Final separate review and technical closure

`review_t23_final` returned **PASS without actionable findings** for the exact
source, tests, final artifact, attributed execution evidence and shared status
documents. It independently verified all thirteen frozen files, 68 compiled and
77 broader preserved inputs, 58 target log hashes, all eight host logs and the
complete 123840-byte `.text` against disassembly, including all 407 relocations.
The report it reviewed before this mechanical closure append had SHA-256
`1e0fe30c090a6c7ab09d72b639e2850597765c71908794a108bdeb769165979e`.
The reviewer used read-only parsing/hashing and ran no builds or runtime.

Task 2.12 is **TECHNICALLY VALIDATED / PENDING FOUNDER ACCEPTANCE** only for this
keyless compilation/static artifact scope. The two original failures remain in
this report, and no runtime, resource/OOM, CPI, transaction rollback, Testnet or
deployment guarantee is inferred. Economics, serialized state, dependencies,
compiler profile, stack/heap limits and actual main remain unchanged.

Normal implementation commit: `cee6072ad7b3155d7d5b30e6c0830beb6d00d4eb`
(`fix: resolve SBF stack errors and pin keyless builds`), containing exactly the
thirteen frozen source/test/tool files. The six documentation files then record
this hash, verdict, commands, tests, limitations and next dependency in the
following normal closure commit. No history rewrite or artifact is included.
Publication must use only the reviewed integration branch after final targeted
candidate/hook checks, with an independent remote read and clean-worktree check.
No Mainnet action, deployment, fund movement, key creation, unrelated secret
access, blockchain signing or authority transfer occurred.
