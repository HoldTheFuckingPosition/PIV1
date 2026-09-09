# Task 2.13 — Keyless local SBF claim execution

Status: **TECHNICALLY VALIDATED / PENDING FOUNDER ACCEPTANCE**.
Implementation: `fd5735976eef1e2728ccf54726145501573db60d`.
The founder resumed with the confirmed GPT-6 family on 2026-09-09 at 18:33 UTC.
The first two host builds remain recorded failures. After separate review of
the isolated dependency/API and evidence corrections, the third build passed and
the pilot executed the exact reviewed binary: 19 tests passed. Final independent
technical review passed; founder acceptance remains pending. Earlier preparation,
pause and release statements below are chronological evidence.
Baseline: reviewed and published Task 2.12 closure
`f9b462b3a1b5bf251ed9b196ee5ff5f5a99468b8`, implementation
`cee6072ad7b3155d7d5b30e6c0830beb6d00d4eb`, on `integration/piv1-testnet`.
Task 2.12's final SBF artifact is 176064 bytes, SHA-256
`0392bb822a3e767674ccd75486ad2685320bce5ffadb426ea8a93b08625bb6c8`.
Task 2.12 established static compilation evidence; the actual Task 2.13 local
runtime evidence is recorded in the final appendix below.

## Requirement and evidence boundary

D-026 permits the next bounded technical dependency after Task 2.12's separate
review, final gates, normal publication and checkpoint. K-012 and the completed
Tasks 2.7–2.11 require isolated already-earned claims, including while paused,
with fixed guardian ownership/destination, source backing, exact accounting,
protected rent/carry/excess, replay rejection, CEI and propagated failures.

Execute the exact unchanged PIV1 ELF in a pinned local SVM using its runtime Rent
syscall, real SBF instruction boundary, ordinary System CPI implementation and
actual account serialization. Compare actual outcomes with independent complete
state/account expectations. This advances beyond injected host invocation.
It does not prove signatures, Bank/AccountsDB transaction commit or rollback,
public cluster configuration, deployment verification or Testnet operation.

Only isolated claims are included. No initialization/earning authority, global
historical liability enumeration, distribution/deposit/governance handler,
economic decision, live address or real SPL/Jito adapter change belongs here.
Keep all production sources, tests, manifests, lockfiles, target build pins,
profile and the final SBF artifact unchanged. Any genuine product incompatibility
must first be reported and scoped separately; never hide it in the harness.

## Proposed runtime and unresolved preparation details

Primary metadata identifies non-yanked `mollusk-svm =0.15.1`, published 2026-08-29,
archive checksum `283ff0ca84f639081a577c16d2bd705e6a45cce79428965860c5973aae6c2169`.
Tag and packaged VCS metadata identify revision
`f432ef136ee9779d2a814ebf2b80f44c10607255`. The proposed enabled features are only
`inner-instructions` and `invocation-inspect-callback`, with default features off.
No fuzz, debugger, trace ejection, custom syscall, native PIV1 replacement or
replacement System processor is permitted. The upstream standard System builtin
must execute the CPI. [Pinned Mollusk source](https://github.com/anza-xyz/mollusk/tree/f432ef136ee9779d2a814ebf2b80f44c10607255).

The writer is assessing exact Agave/SBPF/SDK dependency identities, unchanged
default feature/budget compatibility and an isolated `validation/sbf-claims`
workspace with its own manifest/lock. Root is reviewing requirements and source;
the separate reviewer has assessed the evidence boundary read-only. No manifest,
lock, package fetch, harness build or SBF execution is released yet. Complete
the dependency and command appendix, then separately review the written scope
before preparation, and review exact resolved source/lock/commands before builds.

A required dependency acquisition phase may fetch only pinned public registry
sources into an isolated task cache, inspect build scripts and record checksums
before executing them. It must not update the production dependency graph or
existing compiler installation. Use fixed installed host Cargo/Rust paths, a
clean explicit environment and private task outputs outside Git. The complete
lock and selected build-script inputs need review before compilation; final
validation uses locked/offline commands. No Cargo/SBF wrapper, installer,
validator or key generator is part of this route.

## Runtime setup and passive observations

Read and hash the exact artifact bytes, verify the Task 2.12 source/tool freeze,
and load those bytes through the explicit ELF API. Do not use implicit search
paths or register a host PIV1 processor. Record loader/SBPF/syscall provenance,
feature set, compute budget, heap, frame/call-depth limits and actual artifact.
Keep cache construction and execution configurations consistent and frozen before
loading. Upstream execution-mode loading is not a deployment verifier pass.

Use the shared `process_transaction_instructions` API, including for a single
instruction, with an explicit distinct public fixture payer. These are synthetic
message privileges, not signatures or a funded live payer. The PIV1 instruction
still has exactly five metas. Assert compiled privileges so key deduplication,
payer selection or another instruction cannot repair a missing signer/writable
negative case. Supply unique account-state entries and explicit System state.

Use one consistent runtime Rent source; supplied sysvar accounts can override
the harness's configured sysvars. Verify old/new Rent ABI and actual minimum
balances explicitly before importing fixture bytes. Any extra guardian/payer
fixture rent funding is included when the new isolated runtime audit is created
once. Do not change an existing audit baseline or describe imported state as
continuous custody from the older host World.

Enable passive after-invocation observations of complete account fields and logs.
The callback runs after a top-level invocation, before final result extraction;
it is not a per-CPI hook and provides no result argument. Correlate observations
with instruction ordinal and returned raw error. Never change account data,
lamports, privileges, sysvars, compute meter or context through callbacks.

Resolve inner-instruction indices through the actual returned message. On a
successful claim, require exactly one standard System transfer with the fixed
KIF source, guardian destination, exact amount and expected nesting. Separately
decode the factual claim event from logs, not return data. Record observed
compute usage; absence of failure does not prove a total heap/resource bound.

Require a successful single claim at the pinned Agave non-builtin instruction
allowance of **200000 CU**, in addition to documenting Mollusk's larger default
message budget. Build the case's cache and execution environment from the same
already-selected budget before loading the ELF. Failure at that allowance is a
finding; do not raise it merely to pass. [Agave 4.2.0 execution budget](https://github.com/anza-xyz/agave/blob/v4.2.0/program-runtime/src/execution_budget.rs#L31).

## Required cases and independent oracles

- Successful positive partial/full claims, paused and unpaused, including an
  already-earned historical guardian tuple after current registry changes.
  Assert the four accounting deltas, full Config/reward bytes, exact native
  payment, source rent/carry/excess and every unrelated supplied account field.
- Sequential accepted claims preserve the original runtime audit and cumulative
  replay guard. Stale replay, zero amount, overclaim, backing deficit, liability
  mismatch and arithmetic failure reject with their actual error and no accepted
  payment/event. Do not synthesize a fresh audit after a claim.
- ABI/count, owner/PDA/discriminator/layout/padding/rent, wrong guardian/source,
  missing signer, readonly and alias failures exercise the actual SBF boundary
  or documented earlier runtime preparation boundary. Distinguish program errors
  from runtime errors and never repair a negative fixture with message privileges.
- A shared-context message executes a successful claim, a stale replay second,
  and a third instruction that must not run. Passive observations must show the
  first actual payment/state/event and the subsequent error before discard.
  The final returned accounts must equal the complete original input vector.
- Bounded reduced-compute rejection cases may lower the synthetic execution
  budget only as explicit fault tests. Require actual observed staged accounting
  or payment before labeling an error post-CEI. Otherwise record its observed
  earlier boundary. Do not weaken the success configuration or modify the meter
  during invocation to create a desired trace.

Compare all supplied account bytes, lamports, owner, executable flag and rent
epoch, including the fixture payer/System and preserved sentinels. Use checked
independent arithmetic and original baseline/flow equations. The harness returns
the original account vector on failure; unchanged returned accounts alone prove
only that discard behavior. Raw staged state, CPI trace, errors and logs must
remain visible in evidence. Logs from earlier successful instructions can remain
when the enclosing message fails; event consumers still require overall success.

## Completion, coordination and sensitive limits

One delegated writer owns the isolated harness, dependency lock, scoped runner
and report evidence. The separate reviewer reviews actual source, pinned inputs,
commands and final results; root personally inspects evidence and executes final
applicable checks. No competing builds. Checkpoint and normally commit/publish
the reviewed task before continuing. Host/model, local SVM and Bank/public-chain
evidence remain distinct. Technical validation is not founder acceptance or a
professional independent audit.

No wallet/keypair creation, blockchain signing, secrets access, deployment,
validator, Mainnet action, real funds or authority transfer. Internal ephemeral
VM/JIT hardening entropy is not a wallet, seed phrase or signing identity and
must not be disabled. Public-Testnet and authority-transfer gates remain intact.

## Read-only prerequisite evidence

The pilot verified actual direct host compiler paths under
`/home/jerem/.rustup/toolchains/1.97.1-x86_64-unknown-linux-gnu/bin` using standalone
version commands in a minimal environment. Cargo 1.97.1 is commit `c980f4866`;
Rust 1.97.1 is `8bab26f4f68e0e26f0bb7960be334d5b520ea452`, LLVM 22.1.6.
No rustup link/install, compilation or tool change occurred. Temporary identities:
`/tmp/piv1-t213-host-tool-identities-20260909-a.json`.

| Host tool | SHA-256 |
| --- | --- |
| cargo | `828980723df339d62434390e9fb8ef8831036583343ae2316b7ab5646b5c1953` |
| rustc | `d3a664c970a9fd8361b64194861bebc1ae37b9054e5ee3400dc1c9e691797eea` |
| rustdoc | `3b0c7aef20e0dfdfe2dd19ddb0b40d3b744f76318a44031813f501e1912934df` |

Writer and pilot independently checked the selected upstream lock versions;
the pilot downloaded only public metadata/text into private temporary review
directories. The revision-bound upstream lock has SHA-256
`6a507f1f29c7dc4c1a7603c2fef90f1e709bc09ad9bfc82b152a5044fa06acbd`.
It is provenance for selection, not the future resolved PIV1 harness lock.

| Selected upstream package | Version |
| --- | --- |
| mollusk-svm | 0.15.1 |
| Agave loader/runtime/compute/context/syscalls/System/feature/log family | 4.2.0 |
| solana-sbpf | 0.21.1 |
| solana-account | 4.3.1 |
| solana-instruction | 3.4.0 |
| solana-pubkey used by the harness | 4.2.0 |
| solana-rent | 4.3.0 |
| solana-instruction-error | 2.4.0 |
| solana-transaction-error | 3.3.2 |

The exact isolated manifest/lock and their transitive graph remain pending.
Preserve the production path dependency's original package identities/checksums
and dependency edges when extending a copy of its lock; report any necessary
host-only divergence before a build. The future isolated harness may import the
existing host fixture file read-only and convert public-key bytes explicitly;
it must not accidentally invoke a native host claim instead of the loaded ELF.

The separate reviewer assessed the callback and shared-context APIs, standard
System builtin, account compilation, Rent precedence and failure-discard semantics
against the selected revision. It found no semantic blocker for this bounded
milestone and required the passive observations and explicit payer described
above. That assessment is not yet a review of a resolved lock, runner or runtime
execution. Source compatibility/default-budget details reported by the writer
must become exact assertions and recorded configuration in the harness.

## Proposed preparation contract — awaiting separate review

The writer's completed read-only assessment recommends the following isolated
direct declarations. Remove a declaration if final source does not use it;
do not change a version or enable additional features without reporting the
concrete technical reason for review. Use a private, unpublished package with
its own `[workspace]` and lock, outside the production workspace members.

```toml
[workspace]

[dependencies]
piv1 = { path = "../../programs/piv1", features = ["no-entrypoint"] }
anchor-lang = "=0.32.1"
spl-token = { version = "=8.0.0", features = ["no-entrypoint"] }
mollusk-svm = { version = "=0.15.1", default-features = false, features = [
    "inner-instructions", "invocation-inspect-callback",
] }
solana-account = "=4.3.1"
solana-instruction = "=3.4.0"
solana-pubkey = "=4.2.0"
solana-rent = { version = "=4.3.0", features = ["serde"] }
solana-instruction-error = "=2.4.0"
solana-transaction-error = "=3.3.2"
solana-compute-budget = { version = "=4.2.0", features = ["agave-unstable-api"] }
solana-program-runtime = { version = "=4.2.0", features = ["agave-unstable-api"] }
solana-transaction-context = { version = "=4.2.0", features = [
    "agave-unstable-api", "dev-context-only-utils",
] }
solana-svm-log-collector = { version = "=4.2.0", features = ["agave-unstable-api"] }
bincode = "=1.3.3"
sha2 = "=0.10.9"
```

Retain the inspected Agave runtime family at 4.2.0 and SBPF 0.21.1 in the resolved
lock. Source inspection found the all-enabled feature set admits V0 through V3:
the re-enable-V0 setting overrides disable-V0. The legacy parser accepts machine
263 with flags zero, matching the unchanged artifact. Do not alter those features
to accommodate it. Actual loader compatibility must still be demonstrated.
[Agave runtime construction](https://github.com/anza-xyz/agave/blob/v4.2.0/syscalls/src/lib.rs),
[SBPF 0.21.1 parser](https://github.com/anza-xyz/sbpf/blob/v0.21.1/src/elf.rs).

The selected Rent 4.3.0 representation retains the field positions and 17-byte
serialized layout consumed by the older SDK. Its default rate/threshold encoding
is 6960/1.0 rather than the older 3480/2.0, with equivalent default minimums.
Use an explicit bincode compatibility test and assert actual configured minimums
for 0, 84 and 1014 data bytes, struct size/alignment and full serialized roundtrip.
No unsafe cast or presumed SDK identity. [Selected Rent source](https://docs.rs/solana-rent/4.3.0/src/solana_rent/lib.rs.html).

Once separate written-scope review passes, the pilot may release **preparation
only** to `implement_t26_deposit`:

1. Create `validation/sbf-claims` manifest/lock, fixture/audit and test source,
   plus one bounded standard-library runner/pin companion under `tools` if useful.
   These new paths and a writer appendix in this report are the only write scope;
   root owns shared instructions/checkpoint/status documents. No production file
   or existing SBF runner/pin may change.
2. Create fresh private task Cargo/target/tmp locations outside Git. Use the
   directly verified installed host compiler, no rustup proxy/installation.
   Resolve an isolated copy of the production lock and fetch only public registry
   dependencies, without executing build scripts. Record all commands/results.
   No existing output or credential/configuration directory is reused. Reject
   unexpected Cargo configs rather than reading or bypassing them.
3. Verify exact archive checksums, packaged provenance, complete dependency/feature
   graph and preserved production closure. Inspect every selected build script
   and native helper before compilation; pin required installed host C/link tools
   and relevant libraries. Explain any unavoidable host-only graph divergence
   and obtain technical review before building. No opportunistic upgrades.
4. Freeze the actual source/lock/runner/pins, artifact identity, clean environment
   and exact locked/offline build/run commands. The build first compiles the
   harness without running tests; only its identified test executable may later
   execute under the reviewed environment against the exact artifact. No competing
   builds or implicit SBF wrapper/test-program compilation.
5. Return the concrete preparation packet, release write ownership and await
   separate exact pre-execution review and pilot build-slot release. Preparation
   itself grants no compilation or runtime execution. Report failures or gaps;
   never bypass a test, verifier or source-preservation check to finish.

No required runtime packages were found in the inspected source/archive cache;
the first dependency acquisition is therefore necessary and must be recorded.
The independent reviewer already confirmed that passive raw observations, actual
System CPI traces and shared-context failure propagation can support this bounded
local milestone. A second review of the exact resolved implementation and commands
is still mandatory before execution. The founder is not asked to relay or approve
these ordinary technical coordination steps under D-026.

## Scope review and preparation release

Separate reviewer `review_t23_final` returned **SCOPE / PREPARATION PASS**, no
required corrections, on the complete 281-line contract at SHA-256
`61f2cca820533737357e4eca0321de08be71f80d5ee062bf81af92bacfc92f55`.
It independently verified the referenced 200000-CU constant, V0 feature selection
and Rent representation. Root released the sole writer's bounded preparation
and recorded public dependency acquisition described above. This supersedes the
earlier read-only hold only for those preparation actions. No compilation or
runtime is released before the exact resolved source/input/command review.


## Writer preparation evidence — frozen, no compilation or runtime execution

The sole writer verified user `jerem`, branch `integration/piv1-testnet` and HEAD
`f9b462b3a1b5bf251ed9b196ee5ff5f5a99468b8`, read the released contract, and preserved
the pilot-owned documentation changes. This appendix records preparation only;
the separate scope/preparation PASS is not a source/build/runtime validation PASS.

Eight new files form this preparation freeze. The independent workspace is
unpublished, has its own lock and `autotests = false`, and explicitly selects only
the `claims` integration target. Production sources/tests/manifests/lock/toolchain,
the Task 2.12 runner/pins and the final SBF artifact are unchanged.

| Frozen file | SHA-256 |
| --- | --- |
| `validation/sbf-claims/Cargo.toml` | `45e13ca33f3d05550b4601f43c8c92a9b71c8158f62f0f7d0762aeca52c644c0` |
| `validation/sbf-claims/Cargo.lock` | `2562f8cc9dc74b87d51b4c162e823d620a5b8f398f82ac5d7651f9e662c5bbca` |
| `validation/sbf-claims/README.md` | `528573a4d87cfcb7756eeb9c35e1370f38e3e37cd795ac11fbc69727d2862247` |
| `validation/sbf-claims/tests/claims.rs` | `b6b3689c94e8cc45d640200da070eae290fc95fd7eddc36e206c3f911cc70961` |
| `validation/sbf-claims/tests/support.rs` | `5c0bb21ca6965f7b328c1bcdcfd0a74e843f653e7df6823eeee770c1477ddd8e` |
| `tools/validate_sbf_claims.py` | `3d103a372d556e0090f78b4dae97e45fc6a4ec90694af990931d877822e37c8f` |
| `tools/sbf_claims_pins.json` | `cad47269ccfc267cdbb837f467b68ce9b8f30632858721a82ed21731c14d6273` |
| `tools/test_validate_sbf_claims.py` | `f627c8fe52181cd4720a2fd2f97f2eeecf558ec28ecbfc1d9f8f0616cebace70` |

The machine-readable freeze is
`/tmp/piv1-t213-preparation-20260909-a/preparation-freeze.json`.
The new pin companion binds 79 source inputs, 118 tool/library inputs, nine tool
aliases, every registry package in the isolated lock and the complete selected
host resolve/feature graph. It does not claim a complete operating-system or
supply-chain/reproducibility audit. CPU-selected native code and internal ephemeral
JIT/probe randomness can make generated host outputs nondeterministic.

### Dependency acquisition and inspected graph

The authorized first acquisition used the direct installed Cargo 1.97.1 binary:

```text
/home/jerem/.rustup/toolchains/1.97.1-x86_64-unknown-linux-gnu/bin/cargo fetch --manifest-path /home/jerem/piv1/validation/sbf-claims/Cargo.toml
```

It extended an isolated copy of the root lock and returned exit 0. No build script
was executed. The exact argv/environment and raw output remain in
`/tmp/piv1-t213-preparation-20260909-a/acquisition-command.json` and
`logs/fetch.{stdout,stderr}`. The private public-registry cache is
`/tmp/piv1-t213-preparation-20260909-a/cargo-home`; no existing credential/provider
or Cargo configuration was reused. The initial acquisition and first metadata
commands incorrectly assigned `HOME` to a newly created empty private directory.
The pilot identified this instruction deviation; the original logs and empty
directory were retained. All subsequent commands and the final runner omit HOME
entirely and use only task-specific CARGO_HOME/TMPDIR. The initial jobs=2 setting
executed no compiler; the final build command fixes `--jobs 1`.

The isolated lock contains 394 packages, including three local packages and 391
registry archives. All retained original lock package versions, sources and
checksums remain identical. A matching host-filtered comparison by the pilot
retains all 134 original host package identities/checksums in the imported PIV
closure; the candidate closure has 138. Thirteen existing package feature sets
change through runtime feature unification. Five shared packages gain six direct
edges: curve25519-dalek to serde; digest to const-oid and subtle; generic-array to
zeroize; rand_core to getrandom; zeroize to zeroize_derive. Seven unused IDL-only
root-lock entries and three optional IDL edges are absent. This is explicitly
reviewed isolated host graph divergence, not all-lock/all-features equivalence or
a production dependency change. The separate reviewer passed that divergence
before any build; the whole frozen packet still requires its own review.

Selected host metadata reaches 357 nodes and 34 custom-build scripts. Mollusk's
actual packaged `.cargo_vcs_info.json` matches
`f432ef136ee9779d2a814ebf2b80f44c10607255`, path `harness`. The runtime family remains
4.2.0, SBPF 0.21.1, the exact reviewed SDK versions remain unchanged, and Mollusk
has only the two authorized features. Complete acquisition/final metadata,
selected-package and script inventories remain under the preparation directory:
`metadata.json`, `final-metadata.stdout`, `final-metadata-command.json`,
`selected-packages.json`, `build-scripts.json`, `build-script-review.txt`, and
`production-lock-comparison.json`.

The preflight independently compared all 391 archive checksums with the lock,
then each of 12944 extracted packaged files with its actual verified archive
member. It rejects extra/missing/changed files or symlinks rather than trusting a
mutable `.cargo-checksum.json` claim. This covers the complete packaged native and
build-helper trees as well as Rust sources. No archive was executed or modified.

The pilot inspected blake3/blst and cc/native helper selection; the separate
reviewer inspected the other 32 scripts and their referenced Rust probes/codegen
helpers. The selected native route compiles blake3's four packaged Unix x86_64
assembly files and blst's packaged server.c/assembly.S through cc. Native keygen
functions are existing library definitions, not executed wallet creation. Fixed
CC/AR/RANLIB/linker tools, cc1, collect2, assembler, linker libraries and the Rust
host sysroot are pinned. The runner rejects a libblst.a override and missing
packaged blst/src rather than allowing its upstream parent-directory fallback.
The host CPU report has AVX2 and lacks ADX/AVX512; compiler flag support is not CPU
execution support. Actual native build selection/output remains to be recorded.

`libc`'s optional PATH lookup of `emcc -dumpversion` will fail under the reviewed
PATH; both `/usr/bin/emcc` and `/bin/emcc` are asserted absent before a build.
LIBC_CI and compiler/wrapper overrides are unset. No package is patched to suppress
these probes. The tool inventory is `host-native-tools.json`: two `ldd` probes
returned nonzero because libgcc_s.so and libLLVM-22-rust-1.97.1-stable.so are GNU
linker text scripts, not dynamic executables. Their referenced targets are pinned.
These are expected inspection diagnostics, not compile failures. The other source
lookup misses during preparation were exploratory reads, not failed build tests.

### Prepared runtime evidence and limitations

Seventeen Rust test bodies are saved but have **not been compiled or run**. They
cover a successful exact-backed claim at 200000 CU, a separately labeled 1400000
Mollusk default case, paused historical full claims, repeated payments with an
unchanged original audit, replay, ABI/count/privilege/account-state/native-custody
failures, global backing/ledger errors and checked destination overflow. Four
fixed reduced-compute probes classify their actual observed failure boundary;
none assumes a late failure or changes an active compute meter.

The fixture imports the original host fixture only for state construction and
establishes its own complete runtime funding/audit baseline exactly once. Native
success is derived from actual debit/credit deltas and checked against independent
four-field arithmetic plus complete account equality. The expected state does
not call a host claim or pure prepare/commit transition. Paused/historical fixtures
model already-earned initialized state; there is no executed earning or rotation
handler and no continuous custody claim from a prior host World.

Every case constructs its selected budget and program cache consistently before
loading the exact ELF bytes. The source checks actual loaded-ELF variant, V0,
V0..=V3 runtime range, frame4096/depth64, heap32768, invocation depth9, trace64,
normal execution-mode ELF loading, default features and instruction metering. It
prints the actual loaded VM configuration, configured Rent and budget. The old/new
Rent binary roundtrip, 17-byte serialization, 24-byte/8-aligned host structures and
minimums for 0/84/1014-byte accounts are asserted. No extra Rent account can override
the runtime sysvar. These remain prepared assertions until actual execution.

The actual compiled message payer/signer/writable privileges are checked, with a
distinct public fixture payer. Passive callbacks record complete actual context
account fields and cumulative logs after each top-level invocation. They do not
mutate context, and an unreferenced supplied sentinel is checked in the returned
vector without inventing its presence in the context. Success requires one nested
standard System transfer, exact endpoints/amount and an 80-byte event with literal
discriminator `04c9abfbd77711d0`. Inner traces do not expose raw signer seed arrays.

The shared message is claim-success, stale replay, unreachable third instruction.
It requires actual first effects/event before the second error, exactly two callback
observations, and complete original returned accounts after Mollusk discards the
failed context. That is execution plus harness output discard, not Bank/AccountsDB
rollback. Earlier successful event logs can remain in a failed message; consumers
must require overall success. Synthetic signer flags do not prove signatures or
wallet ownership. No loader/deployment-verifier or cluster-configuration claim is
made, and passing specific paths would not prove a total runtime heap bound.

### Actual preparation checks and next commands

The stdlib refusal suite ran twice: initial six tests PASS (tool transcript), then
final six tests PASS (recorded `stdlib-final-command.json` and
`logs/stdlib-final.{stdout,stderr}` in the preparation directory). These tests do
not invoke Cargo or an SBF runtime. No test was suppressed and no dependency warning
was hidden. Both actual metadata calls and the final preflight metadata returned
exit 0; there has been no host compilation or runtime command to report.

The actual final preflight was:

```text
/usr/bin/python3 -I /home/jerem/piv1/tools/validate_sbf_claims.py --output /tmp/piv1-sbf-claims-preflight-20260909-a --approved-runner-sha256 3d103a372d556e0090f78b4dae97e45fc6a4ec90694af990931d877822e37c8f --approved-pins-sha256 cad47269ccfc267cdbb837f467b68ce9b8f30632858721a82ed21731c14d6273
```

Result: **PREFLIGHT_PASS**, 11 read-only subprocess commands, with durable raw
stdout/stderr, UTC command times, package/source/tool evidence and independent
Git/source/artifact preservation checks under
`/tmp/piv1-sbf-claims-preflight-20260909-a`. Both hash arguments here name the
candidate being reviewed; this preflight did not grant build approval.

The exact proposed next **build-only** command, pending separate review and pilot
slot release, is:

```text
/usr/bin/python3 -I /home/jerem/piv1/tools/validate_sbf_claims.py --output /tmp/piv1-sbf-claims-build-20260909-a --stage build --approved-runner-sha256 3d103a372d556e0090f78b4dae97e45fc6a4ec90694af990931d877822e37c8f --approved-pins-sha256 cad47269ccfc267cdbb837f467b68ce9b8f30632858721a82ed21731c14d6273
```

Its compilation subprocess is fixed to:

```text
/home/jerem/.rustup/toolchains/1.97.1-x86_64-unknown-linux-gnu/bin/cargo test --manifest-path /home/jerem/piv1/validation/sbf-claims/Cargo.toml --locked --offline --jobs 1 --test claims --no-run --message-format=json
```

The exact environment is produced by `environment()` in the frozen runner:
PATH `/usr/bin:/bin`, LC_ALL `C`, private CARGO_HOME above, TMPDIR and
CARGO_TARGET_DIR under the fresh output, direct fixed RUSTC/RUSTDOC, color `never`,
CARGO_NET_OFFLINE `true`, CC `/usr/bin/cc`, AR `/usr/bin/ar`, RANLIB `/usr/bin/ranlib`,
and the x86_64 host linker `/usr/bin/cc`. HOME, wrappers, custom Rust flags,
credentials and inherited runtime/debug/Cargo settings are absent. This is a
bounded clean command environment, not an OS network sandbox.

Only after a successful build and review of its identified executable/hash may
the pilot release a separate `--stage run` command using fresh
`/tmp/piv1-sbf-claims-run-20260909-a`, the exact build output and
`--approved-executable-sha256 <actual reviewed hash>`. That future hash is unknown
at preparation; no execution is currently released. The run stage invokes only
that identified host executable with `--test-threads=1 --nocapture`, never Cargo
or a native substitute for PIV1. Compiler/runtime diagnostics remain raw evidence;
nonzero results or compiler warnings require review rather than silent retries.

All writer-owned source is now frozen for separate review. Report append ownership
and preparation write ownership are released to the pilot. No build slot was used
or is retained. No production/Git mutation, commit, push, key creation, blockchain
signature, validator, deployment, live-chain action, real fund movement or authority
transfer occurred. Production reference HEAD remains the reviewed Task 2.12 closure
above. The remaining work is exact pre-execution review, then separately released
compilation and runtime validation; no runtime result is claimed here.


## Writer correction and final preparation freeze — 2026-09-09 resumption

The founder resumed the bounded task after the writer's usage-limit interruption;
the pilot released preparation only. The writer reverified `jerem`,
`integration/piv1-testnet`, HEAD `f9b462b3a1b5bf251ed9b196ee5ff5f5a99468b8`, all
frozen source/tool inputs and the unchanged exact ELF. No compilation or runtime
execution occurred before or after the interruption.

Two pre-execution review corrections supersede the initial candidate above:

- The reduced-compute classifier accepts the direct
  `ComputationalBudgetExceeded` error separately. A wrapped
  `ProgramFailedToComplete` now requires the pinned exact diagnostic substring
  `exceeded CUs meter at BPF instruction`. A new small test rejects the distinct
  maximum-call-depth message and unrelated failures; the earlier generic
  `exceeded` match is gone. No limit, budget, production check or runtime behavior
  was changed. Eighteen Rust test bodies are now prepared, still uncompiled and
  unexecuted; the new classifier regression has no claimed test result yet.
- The selected native tool/library pins now include the matching GCC PIE/shared
  end CRT object `/usr/lib/gcc/x86_64-linux-gnu/13/crtendS.o`, SHA-256
  `9c8b2caad195193301ef072895dc89bea1d23371d4c19accd821debe27773063`.
  There are now 119 selected tool/library inputs and nine aliases. These pins are
  not a fully hermetic toolchain, operating-system or system-header closure.

The initial eight-file freeze is preserved verbatim as
`/tmp/piv1-t213-preparation-20260909-a/preparation-freeze-initial.json`.
`preparation-freeze.json` now names the final candidate below, also preserved in
`preparation-freeze-v3.json`; the intermediate v2 file remains untouched. Only the
claim test source and pin companion differ from the initial eight-file candidate.
The manifest/lock, runner, support, README and six-test stdlib suite are unchanged.

| Final frozen file | SHA-256 |
| --- | --- |
| `validation/sbf-claims/Cargo.toml` | `45e13ca33f3d05550b4601f43c8c92a9b71c8158f62f0f7d0762aeca52c644c0` |
| `validation/sbf-claims/Cargo.lock` | `2562f8cc9dc74b87d51b4c162e823d620a5b8f398f82ac5d7651f9e662c5bbca` |
| `validation/sbf-claims/README.md` | `528573a4d87cfcb7756eeb9c35e1370f38e3e37cd795ac11fbc69727d2862247` |
| `validation/sbf-claims/tests/claims.rs` | `7e7b0217cf3e483643531500a2191c0219bfbb03c27314ab8d5911ac1f4911e8` |
| `validation/sbf-claims/tests/support.rs` | `5c0bb21ca6965f7b328c1bcdcfd0a74e843f653e7df6823eeee770c1477ddd8e` |
| `tools/validate_sbf_claims.py` | `3d103a372d556e0090f78b4dae97e45fc6a4ec90694af990931d877822e37c8f` |
| `tools/sbf_claims_pins.json` | `647c28cf52db63d33f6c5fed049a341090283a0706c4748706949fdf860e4fef` |
| `tools/test_validate_sbf_claims.py` | `f627c8fe52181cd4720a2fd2f97f2eeecf558ec28ecbfc1d9f8f0616cebace70` |

The writer executed this fresh **preflight only** against the corrected pins:

```text
/usr/bin/python3 -I /home/jerem/piv1/tools/validate_sbf_claims.py --output /tmp/piv1-sbf-claims-preflight-20260909-b --approved-runner-sha256 3d103a372d556e0090f78b4dae97e45fc6a4ec90694af990931d877822e37c8f --approved-pins-sha256 647c28cf52db63d33f6c5fed049a341090283a0706c4748706949fdf860e4fef
```

Actual result: **PREFLIGHT_PASS**, with no original or preservation error.
Its 11 read-only subprocess commands, UTC timings, raw outputs, exact environment,
391 verified archives / 12944 verified packaged files, final 357-node metadata,
source/tool/artifact checks and unchanged Git HEAD/refs are recorded under
`/tmp/piv1-sbf-claims-preflight-20260909-b`. The initial successful preflight
`...-a` and all acquisition/research/stdlib evidence remain unchanged. The unchanged
stdlib runner suite's last executed result remains six PASS; it was not rerun
merely to regenerate equivalent output. No Cargo compiler or SBF test was run.

The following is the **final proposed build-only command**, still requiring the
separate exact pre-execution review and pilot build-slot release. It replaces the
older pin-hash command in the initial candidate appendix:

```text
/usr/bin/python3 -I /home/jerem/piv1/tools/validate_sbf_claims.py --output /tmp/piv1-sbf-claims-build-20260909-a --stage build --approved-runner-sha256 3d103a372d556e0090f78b4dae97e45fc6a4ec90694af990931d877822e37c8f --approved-pins-sha256 647c28cf52db63d33f6c5fed049a341090283a0706c4748706949fdf860e4fef
```

The fixed locked/offline, one-job `cargo test --no-run` subprocess and clean
explicit environment above are unchanged. Later direct test execution remains a
separate release requiring its actual identified executable SHA-256, which cannot
be provided before compilation. Source compatibility/preflight evidence must not
be relabeled runtime evidence.

All eight writer-owned files and the report appendix are released/frozen for
review. No build slot was used or is retained. The pilot-owned AGENTS/checkpoint/
plan/Task 2.12 publication changes are preserved. Production sources/tests,
production manifests/lock/toolchain, existing Task 2.12 runner/pins and ELF are
unchanged. No Git mutation, commit, push, key creation, wallet/signature operation,
validator, deployment, live-chain action, fund movement or authority transfer
occurred. No additional task or implementation work is authorized by this handoff.

## Final pre-execution review and pilot build release

Separate reviewer `review_t23_final` returned **PRE-EXECUTION PASS**, with no
remaining actionable finding, on the corrected eight-file freeze SHA-256
`54f50ff926820c9930d350217fe69012227e830b44b47bd2fbececf0e0d8cf60` and report
`e9ba79e8136b3d686c48ae02ae1344c36b79770e29eef898c770bed3aa3f7a9b` before this
status append. It independently checked the corrected CU classification/CRT
pin, 79 source inputs, 119 tool/library inputs, nine aliases, unchanged original
Task 2.12 freeze/ELF, scoped harness oracles and final locked/offline build command.
Both reviewer and pilot verified all 22 log hashes from the corrected preflight's
11 successful commands. The pilot personally executed the six unchanged stdlib
refusal tests with `/usr/bin/python3 -I -B tools/test_validate_sbf_claims.py` in
a clean PATH/LC_ALL environment: six PASS, no Cargo or SBF invocation.

The writer released all file ownership and holds no build slot. The pilot now
releases and executes exactly the final build-only command above into the fresh
`/tmp/piv1-sbf-claims-build-20260909-a`. This does not release runtime execution;
that requires inspecting the actual resulting executable, diagnostics and hash.
Neither preparation review nor compilation is founder acceptance.


## Writer isolated SDK compatibility correction — first host build remains FAIL

The pilot, after separate pre-execution PASS, executed the reviewed build-only
attempt `.../piv1-sbf-claims-build-20260909-a`. Cargo returned **101** after
57.236 seconds (2026-09-09 18:38:55.243833–18:39:52.479913 UTC). Its eight E0277
errors, plus one explanatory failure-note, were in `solana-instruction 3.4.0`:
Pubkey supplied wincode 0.6 `SchemaRead`/`SchemaWrite` implementations while
instruction serialization required the distinct wincode 0.5 traits. No harness
executable or runtime result was produced. This is an isolated host dependency
compatibility failure, not a PIV1 program or observed SBF runtime failure.

The original failed output remains untouched and FAIL. The writer independently
rechecked these pilot-recorded evidence hashes:

| Original failed evidence | SHA-256 |
| --- | --- |
| `result.json` | `7faa00da200039b124bc63922e029e27cee296065322474dc1a1f15fd20b0908` |
| `diagnostics.json` | `88f7a950517df343321eeaeb245a95acf63d51e675e5445ac98042b259757958` |
| `build.stdout` | `96a7f7d59b59201130e742d2ef1d34b5653acd40a286dd5d392a1901525c0d65` |
| `build.stderr` | `2cdb97da5c7f72b80c21a2dd32e94e39215c3605ffe0f676bdb3f6eccfca8be7` |

The failed attempt's source/tool/package/Git preservation checks passed. The pilot
released only a bounded dependency correction preparation, without a build or
runtime slot. The writer inspected the actual published manifests and source,
then added these isolated direct compatibility constraints with default features
disabled; existing Mollusk/runtime/direct SDK versions and all test behavior remain
unchanged:

```toml
solana-address = { version = "=2.6.1", default-features = false }
solana-short-vec = { version = "=3.2.2", default-features = false }
```

Both versions match the revision-bound upstream and packaged Mollusk lock.
Address 2.7.0 had moved to wincode 0.6, whereas 2.6.1 retains 0.5. After the first
address-only update, metadata still retained wincode 0.6 through short-vec 3.3.0.
Its ShortU16 traits cross into the message 4.4.0 / transaction 4.1.5 wincode 0.5
containers. The writer reported this concrete second incompatibility before
acquiring/resolving another package. The pilot and separate reviewer independently
confirmed and authorized the narrow short-vec 3.2.2 constraint. The address-only
intermediate lock/metadata/logs are preserved, not presented as the final graph.

Only these two exact public archives were acquired into the existing private
public-registry cache, with recorded URLs/timestamps/checksums:

| Package | Archive SHA-256 | Packaged VCS revision |
| --- | --- | --- |
| solana-address 2.6.1 | `39c93e262f671bf402e1040e4a7e40b05d81da5956c7681948c975a0997517bb` | `14a725d6e9180e6cfbd98054473d61ef3aabde57` |
| solana-short-vec 3.2.2 | `7d8250a4495aad49ad20556a607da53bdcb20de78da10b65afbf918b7f1de647` | `699d7993b9c2a42ed1ad7b9305a46ecd67b636de` |

Both declare `build = false`. Their complete packaged Rust file sets and bytes
are identical to their respective previous versions; the relevant active change
is the wincode manifest constraint. Optional inactive frozen-ABI constraints,
documentation metadata and development-only constraints also differ. No package
source was patched, and no new script/native helper is selected.

The writer ran two exact offline `cargo update --package <old-version> --precise
<reviewed-version>` commands and their locked/offline host-filtered metadata
commands, all exit 0. No compiler or build script was executed in this correction
preparation. Exact argv, environment, timestamps and raw output are retained in
`/tmp/piv1-t213-address-correction-20260909-a/commands.json` and
`final-commands.json`; public acquisitions and source comparisons are in the same
directory. The first update selected only address 2.6.1; the second selected only
short-vec 3.2.2 and removed wincode 0.6.1 / wincode-derive 0.5.1. Cargo's note about
an unchanged package being behind latest is informational; no broad update was
performed. Removed public cached archives/source remain untouched.

Final metadata has **355 selected host nodes**, **392 lock entries / 389 registry
archives**, and **33 build scripts**. Only the obsolete wincode 0.6.1 build script
leaves the selected script set. Every retained package feature set/checksum is
unchanged from the failed candidate. The complete imported PIV host closure's
138 nodes, dependency edges and features are exactly unchanged, as are every
production source/manifest/lock/toolchain and the exact final SBF ELF.
`delta.json` records the package/features/scripts/closure comparison. Existing
native helper/tool identities and nine aliases remain unchanged at 119 selected
tool/library inputs; these are still not a hermetic system-header/toolchain proof.

The following eight-file candidate supersedes the previous preparation freeze.
Only the isolated manifest, isolated lock and new pin companion differ. All
harness/support tests, runner and stdlib tests remain byte-identical. The old v3
freeze remains preserved; the new freeze is
`/tmp/piv1-t213-address-correction-20260909-a/preparation-freeze.json` and the
canonical preparation-freeze.json now matches it.

| Corrected frozen file | SHA-256 |
| --- | --- |
| `validation/sbf-claims/Cargo.toml` | `9bf5c36728e284a8d7b6184f7e91b0ed092fc91d66d43b9356e345893f0c75e0` |
| `validation/sbf-claims/Cargo.lock` | `1e4707abc7edf450bb3f1fb85a6fdd661d498d6963ec0062c567e624390297dd` |
| `validation/sbf-claims/README.md` | `528573a4d87cfcb7756eeb9c35e1370f38e3e37cd795ac11fbc69727d2862247` |
| `validation/sbf-claims/tests/claims.rs` | `7e7b0217cf3e483643531500a2191c0219bfbb03c27314ab8d5911ac1f4911e8` |
| `validation/sbf-claims/tests/support.rs` | `5c0bb21ca6965f7b328c1bcdcfd0a74e843f653e7df6823eeee770c1477ddd8e` |
| `tools/validate_sbf_claims.py` | `3d103a372d556e0090f78b4dae97e45fc6a4ec90694af990931d877822e37c8f` |
| `tools/sbf_claims_pins.json` | `c404577ac9b03876d153c33910b36ee5badbfaa9b4742556788d43e1e4168e1a` |
| `tools/test_validate_sbf_claims.py` | `f627c8fe52181cd4720a2fd2f97f2eeecf558ec28ecbfc1d9f8f0616cebace70` |

The writer executed a fresh preflight with the corrected pin companion:

```text
/usr/bin/python3 -I /home/jerem/piv1/tools/validate_sbf_claims.py --output /tmp/piv1-sbf-claims-preflight-20260909-c --approved-runner-sha256 3d103a372d556e0090f78b4dae97e45fc6a4ec90694af990931d877822e37c8f --approved-pins-sha256 c404577ac9b03876d153c33910b36ee5badbfaa9b4742556788d43e1e4168e1a
```

Actual result: **PREFLIGHT_PASS**, 11 read-only subprocess commands, all 389
archives and 12895 extracted packaged files verified; source/tool/graph/ELF
and Git preservation checks passed with no original or preservation error.
Full evidence is `/tmp/piv1-sbf-claims-preflight-20260909-c`. Earlier preflights
and the failed build remain unchanged. No new Rust test result is claimed; the
same eighteen prepared Rust test bodies and six previously passing stdlib refusal
tests remain intact. The unchanged stdlib tests were not unnecessarily rerun.

Exact proposed next **build-only** command, pending separate changed-input review
and a new pilot slot release:

```text
/usr/bin/python3 -I /home/jerem/piv1/tools/validate_sbf_claims.py --output /tmp/piv1-sbf-claims-build-20260909-b --stage build --approved-runner-sha256 3d103a372d556e0090f78b4dae97e45fc6a4ec90694af990931d877822e37c8f --approved-pins-sha256 c404577ac9b03876d153c33910b36ee5badbfaa9b4742556788d43e1e4168e1a
```

The runner's direct installed compiler, clean environment, locked/offline one-job
`cargo test --no-run`, fresh outputs and later separately hash-approved direct
executable stage are unchanged. The correction is source/graph preparation,
not a successful compilation or runtime result. Further actual compiler findings
must remain visible before any additional changes.

Writer-owned source/report append are frozen and released to the pilot/reviewer.
No build slot is held. The writer made no production or Git mutation, commit,
push, key creation, wallet/signature operation, validator, deployment, live-chain
action, real fund movement or authority transfer. Pilot-owned shared documents
are preserved. HEAD remains `f9b462b3a1b5bf251ed9b196ee5ff5f5a99468b8`.

## Second build release after independent dependency delta review

At 18:54 UTC, separate reviewer `review_t23_final` returned **PRE-EXECUTION DELTA
PASS**, with no actionable blocker, for the exact `-b` build-only command above.
Both reviewer and pilot independently verified the final eight-file freeze
`e63d325bb26d91a6936448a52d52949214c576300f94aad56a708904507a78fa`, preflight-c
result `59d0a412f6d978065aad56fc4981e1f858146eb7fcf9a0db7a5708778eb2d5b4`, all 11
successful commands and 22 output hashes. Both confirmed the exact package delta,
unchanged retained features/checksums, complete 138-node PIV closure and no new
build scripts. The pilot's initial ad hoc metadata inspection had two Python
identity/field assumptions corrected before its successful comparison; those
read-only helper failures were not compiler or product failures.

The pilot reverified `jerem` uid 1001, single worktree, unchanged integration HEAD,
expected uncommitted paths, clean `git diff --check`, available local resources
and unused output `...-b`. Writer ownership is released. The pilot now executes
that command with outer `env -i PATH=/usr/bin:/bin LC_ALL=C`; the runner constructs
the already-reviewed explicit child environment. Only compilation is released.
Actual executable identity and diagnostics must be inspected before runtime.

## Writer harness API correction after second failed build

The pilot's second build (`/tmp/piv1-sbf-claims-build-20260909-b`) failed with
Cargo 101 after 203.361 seconds, 18:54:39–18:58:02 UTC. It reached the isolated
harness and reported three errors: two incorrect `TransactionResult` module
paths and a `Debug` requirement unsupported by `SVMFeatureSet`. It also reported
one deprecated insufficient-account variant warning. No harness executable or
runtime result exists. The original output remains intact; its result SHA-256 is
`8ac9e323ac6a7284622759ea37c9e9bc5cb104e1ae5f42d66c200cae192d3e15`, with pilot-verified
preservation PASS. This is harness API compatibility evidence, not a PIV1 runtime
failure or successful validation.

The bounded correction changes only `tests/support.rs`, `tests/claims.rs` and
those two source hashes in the pin companion:

- Both annotations now use the actual public
  `mollusk_svm::result::types::TransactionResult`. Its result, account, message and
  inner-instruction fields match the retained assertions.
- Feature provenance explicitly records every one of the pinned
  `solana-svm-feature-set 4.2.0` source's 52 public boolean fields and asserts all
  are true, matching its unchanged `all_enabled` defaults. No runtime feature,
  loader or compute configuration is changed.
- The four-account error oracle converts the accepted old SDK's
  `ProgramError::NotEnoughAccountKeys` into its numeric ABI, asserts `11 << 32`,
  and decodes that through current `InstructionError::from`. The pinned runtime
  maps this to its legacy insufficient-keys variant, which is explicitly asserted
  distinct from `MissingAccount`. Existing `num-traits` is already enabled.
  No deprecation allowance, replacement error or feature change is introduced.

| Changed frozen file | SHA-256 |
| --- | --- |
| `validation/sbf-claims/tests/support.rs` | `1378d5c97fa683ef90c1f968c4025c6f0858d73f8071bf6308c8b60b4baaa9df` |
| `validation/sbf-claims/tests/claims.rs` | `34e0fa31428efd573b06f69ac26a5ff3dbf07b7c57f30237379cba3d797b3f53` |
| `tools/sbf_claims_pins.json` | `255c510c959fd33a852293f5b10613c225ff98cd7a9231a789e792bf130dca96` |

The complete eight-file freeze is
`/tmp/piv1-t213-api-correction-20260909-a/preparation-freeze.json`, SHA-256
`eec1919112dacf2bada95e71a854f73f7913dea042f7ea9dbb121fbb189308d5`;
the canonical preparation freeze matches. Saved before-files and focused diffs
are in the same evidence directory. The manifest, lock, runner, README, stdlib
tests, all tool/package/feature pins, production sources and exact SBF artifact
are unchanged. Eighteen Rust test declarations remain; none has executed.

The writer ran exactly one corrected read-only preflight:

```text
/usr/bin/python3 -I /home/jerem/piv1/tools/validate_sbf_claims.py --output /tmp/piv1-sbf-claims-preflight-20260909-d --approved-runner-sha256 3d103a372d556e0090f78b4dae97e45fc6a4ec90694af990931d877822e37c8f --approved-pins-sha256 255c510c959fd33a852293f5b10613c225ff98cd7a9231a789e792bf130dca96
```

Actual result **PREFLIGHT_PASS**: all 11 read-only commands returned zero; all 22
raw log hashes were checked. Source/tool/package/graph/ELF and Git preservation
passed. Result `/tmp/piv1-sbf-claims-preflight-20260909-d/result.json` has SHA-256
`e7b2cced4694ac4e55cf1d392ed093e079c1d1a65c12ce559344abe142709b17`.
No compilation, runtime or unchanged stdlib test repetition occurred during this
correction. Rustfmt was not run; the pinned component is absent and no tool was
installed. This preflight does not establish compilation or runtime success.

Exact proposed fresh **build-only** command, pending separate delta review and
pilot slot release:

```text
/usr/bin/python3 -I /home/jerem/piv1/tools/validate_sbf_claims.py --output /tmp/piv1-sbf-claims-build-20260909-c --stage build --approved-runner-sha256 3d103a372d556e0090f78b4dae97e45fc6a4ec90694af990931d877822e37c8f --approved-pins-sha256 255c510c959fd33a852293f5b10613c225ff98cd7a9231a789e792bf130dca96
```

Writer-owned source and report are frozen and released. No build slot is held.
Verified user remains `jerem`, branch `integration/piv1-testnet`, HEAD
`f9b462b3a1b5bf251ed9b196ee5ff5f5a99468b8`. Expected uncommitted pilot documents
and new Task 2.13 paths remain preserved. No Git mutation, commit, push, Mainnet
action, key creation, signing, deployment, validator, live-chain operation,
real fund movement or authority transfer occurred.

## Writer T213-R2 correction: complete account evidence

Before build-c execution, the pilot and separate reviewer found that pinned
`Account::Debug` prints only the first 64 data bytes. The existing in-memory
assertions remain valid, but those logs could not reconstruct full Config/reward
state after failed execution. No build-c or runtime had occurred.

The isolated support now additionally logs every actual account field, key,
complete lowercase hexadecimal data and data length for the supplied vector,
returned vector and each raw after-invocation observation. Case, stage, invocation
ordinal and account index identify each record. Existing Debug context, errors,
privileges, CPI/event evidence and all state/custody oracles remain unchanged.
One pure formatter regression checks all metadata plus distinct bytes across the
64-byte boundary and through both final bytes at Config/reward lengths. The
harness now has 19 prepared tests; none has executed.

| Changed frozen file | SHA-256 |
| --- | --- |
| `validation/sbf-claims/tests/support.rs` | `9c08b8a61542b1e25139b1c2150016d030db81d27bd1da26f4498217ecfd8416` |
| `validation/sbf-claims/tests/claims.rs` | `f70147fd64dcbf063b6b1f3ea2c02b97b0534b774f3f23639e3afcd38d7c383a` |
| `tools/sbf_claims_pins.json` | `a0b388a11ae94704038b9ff177978979f1c7c219cb9ea8c60c8733e5ac5c0ad7` |

Full freeze: `/tmp/piv1-t213-account-evidence-20260909-a/preparation-freeze.json`,
SHA-256 `1048fa3376527d765390f8b9c6214fbef46062336aeb91a95a8c659c69baaa36`;
the canonical freeze matches. Before-files and focused diffs are alongside it.
Only these two source hashes changed in the pins. Production, dependencies,
runner, feature configuration, prior failure evidence and exact ELF are intact.

One fresh corrected preflight was executed:

```text
/usr/bin/python3 -I /home/jerem/piv1/tools/validate_sbf_claims.py --output /tmp/piv1-sbf-claims-preflight-20260909-e --approved-runner-sha256 3d103a372d556e0090f78b4dae97e45fc6a4ec90694af990931d877822e37c8f --approved-pins-sha256 a0b388a11ae94704038b9ff177978979f1c7c219cb9ea8c60c8733e5ac5c0ad7
```

Actual **PREFLIGHT_PASS**: 11 successful read-only commands, all 22 raw log hashes
verified, preservation checks passed. Result SHA-256 `97e7769aa72743ece425e04eee9f472fb3e35f5efdc8f2464965a588f923df65`.
No compiler, formatter, test binary or runtime was executed during this correction.

The previous unexecuted build-c proposal is superseded by this exact command,
pending separate delta review and pilot build release:

```text
/usr/bin/python3 -I /home/jerem/piv1/tools/validate_sbf_claims.py --output /tmp/piv1-sbf-claims-build-20260909-c --stage build --approved-runner-sha256 3d103a372d556e0090f78b4dae97e45fc6a4ec90694af990931d877822e37c8f --approved-pins-sha256 a0b388a11ae94704038b9ff177978979f1c7c219cb9ea8c60c8733e5ac5c0ad7
```

Source/report ownership is frozen and released; no build slot is held. Git HEAD,
branch and expected uncommitted paths remain unchanged. No Git mutation, commit,
push, Mainnet action, key creation, signing, validator, deployment, live operation,
real fund movement or authority transfer occurred. Compilation, actual runtime
execution and final separate validation remain pending.

## Final writer evidence inspection: pilot build and local runtime PASS

The pilot executed the exact reviewed build-c command above on the frozen inputs.
Its direct child command was:

```text
/home/jerem/.rustup/toolchains/1.97.1-x86_64-unknown-linux-gnu/bin/cargo test --manifest-path /home/jerem/piv1/validation/sbf-claims/Cargo.toml --locked --offline --jobs 1 --test claims --no-run --message-format=json
```

Actual **BUILD_PASS**, Cargo 0, 19:16:28.943602–19:19:18.237283 UTC, 169.293698
seconds. Both compiler diagnostic arrays are empty. The identified executable is
`/tmp/piv1-sbf-claims-build-20260909-c/target/debug/deps/claims-c12d45bbd63e9bf4`,
96,591,016 bytes, SHA-256
`c3a8cf5106109c3986e86cbd1b886442c2b4516698fb56814ee0ae0a915cd356`.
The first two failed builds and every earlier review/correction remain intact.

After separate execution release, the pilot ran this exact invocation:

```text
env -i PATH=/usr/bin:/bin LC_ALL=C /usr/bin/python3 -I /home/jerem/piv1/tools/validate_sbf_claims.py --output /tmp/piv1-sbf-claims-run-20260909-a --stage run --approved-runner-sha256 3d103a372d556e0090f78b4dae97e45fc6a4ec90694af990931d877822e37c8f --approved-pins-sha256 a0b388a11ae94704038b9ff177978979f1c7c219cb9ea8c60c8733e5ac5c0ad7 --build-output /tmp/piv1-sbf-claims-build-20260909-c --approved-executable-sha256 c3a8cf5106109c3986e86cbd1b886442c2b4516698fb56814ee0ae0a915cd356
```

The runner directly invoked that binary with `--test-threads=1 --nocapture`.
Actual **RUNTIME_TESTS_PASS**, exit 0, 19:21:24.952265–19:21:27.092124 UTC:
**19 passed, 0 failed, 0 ignored, 0 filtered**, test duration 2.13 seconds
(process duration 2.139870 seconds). The writer inspected saved evidence only;
no build or runtime repetition occurred.

| Durable evidence | SHA-256 |
| --- | --- |
| `...build-20260909-c/result.json` | `0ef283a217a21ba4d1699d7d20be8eb883e8304077bdc8ed5622f23b9617a505` |
| `...run-20260909-a/result.json` | `26d070f234678a40aecad3cd22e692d0a4ae2ddad4eaf24aa2e87d7491ac3a78` |
| `...run-20260909-a/claims.stdout` | `a5335a996c44f911cbafbb93bdd8f14f06d46c6fd3c6cab75e5736db24240b4b` |

These paths have prefix `/tmp/piv1-sbf-claims-`. Each result records exact argv,
environment, UTC timing and both hashes for every command's raw output. The writer
verified all 48 output hashes across both stages, both preservation PASS records,
and the unchanged eight-file source freeze. All 1,366 explicit account records
contain their declared full byte lengths: 480 supplied, 480 returned and 406 raw.

The nineteen tests cover exact-backed partial and paused historical full claims,
repeated claims against the original audit, Rent compatibility, ABI/error
precedence, compiled privileges, identity/serialization/backing/rent/overflow
failures, shared-context failure, reduced compute, and the two pure evidence/error
classification regressions. The required 200,000-CU exact-backed claim consumed
73,834 CU; the separately configured 1,400,000-CU case also used 73,834 CU, and the
paused full claim used 73,838 CU. Successful cases verified actual account bytes,
native transfer, ordinary System CPI trace and exact 80-byte KifClaimed event.

The shared sequence consumed 92,860 CU: its first instruction made the exact
100-lamport claim; the second failed at index 1 with `Custom(6004)` and the third
was never invoked. Both raw callbacks retain the first payment and corresponding
state bytes; the returned complete account vector equals the original supplied
vector. Logs are cumulative, so the event retained in the second callback is the
first instruction's event. This demonstrates execution and Mollusk output discard,
not Bank transaction rollback.

All four reduced probes failed at their selected limits. Budgets 1, 5,000 and
20,000 returned `ProgramFailedToComplete` with the specific instruction-meter
exhaustion log; 10,000 returned `ComputationalBudgetExceeded`. The pilot's full-byte
comparison, independently corroborated by the reviewer, finds exact unchanged raw
and returned accounts in every probe, with no observed bookkeeping or native
payment. **No post-bookkeeping compute-failure boundary was reached.** That
classification uses complete bytes, not the diagnostic booleans alone. The
pilot's 60-case account analysis is
`/tmp/piv1-t213-pilot-account-observations-20260909-a.json`, SHA-256
`045881acad02a6ff435b530c0d6b0d0de176d1b92bf103105f1daae54bebd9de`.

The scope remains synthetic initialized fixtures and exact local claim execution.
It proves neither earning/initialization authority nor a global historical ledger,
signatures, Bank/AccountsDB commit/rollback, deployment verification, other handlers,
public cluster behavior or continuous custody from an older host model. Events
must be conditioned on overall transaction success. Rustfmt and Clippy are absent
from the pinned host toolchain and were not run or installed. Unchanged production
339-test plus one-doctest evidence remains attributed to Task 2.12 and was not rerun.

Only this report's current status and final appendix changed during this handoff.
Source/pins/tools remain frozen; report ownership is released to the pilot. No
slot is held. Integration HEAD remains `f9b462b3a1b5bf251ed9b196ee5ff5f5a99468b8`
with expected uncommitted Task 2.13/pilot documents; the writer made no commit,
push or other Git mutation. The local harness changed synthetic account balances;
no real funds, Mainnet action, keys, signatures, validator, deployment, live-chain
operation or authority transfer were involved. Final closure review was pending
at the writer handoff; the following pilot verdict supersedes that status.

## Pilot final technical verdict

Separate reviewer `review_t23_final` returned **FINAL BOUNDED TECHNICAL PASS**
without actionable findings on report SHA-256
`2dc9a96613cbc8cc2ef80c81ae2c4a48b5694985b80d5dac40346884c47fb183` before this append.
Root independently reconstructed the four ledger changes from the accepted Borsh
field order and checked all 1366 complete logged account records, including both
raw shared-message states and all four unchanged reduced-compute cases. Its saved
analysis is `/tmp/piv1-t213-pilot-account-observations-20260909-a.json`, generated
by `/tmp/piv1-t213-pilot-account-evidence.py`; neither analysis reran the harness.
Reviewer independently checked the exact System transfer/80-byte event evidence
and complete account bytes with the same conclusion. Executions remain root's.

Status: **TECHNICALLY VALIDATED / PENDING FOUNDER ACCEPTANCE**, only within this
report's local claim scope. Source, runtime inputs and all earlier failed evidence
remain unchanged. The pilot is completing shared documentation and normal Git
publication. No later task, public-network execution or acceptance is implied.

The separate reviewer subsequently passed the concise closure documents after
correcting released-writer ownership and clarifying unchanged production
dependencies. The prior 484-line/34322-byte checkpoint is retained verbatim beneath
the historical archive banner; the current checkpoint carries concise state and
links. The pilot's targeted 15-file credential/generated-output check passed,
with the eight tested files still matching their frozen hashes. Git hooks were
sample-only, effective `core.hooksPath` unset, and no repository `.github`/`.cargo`
automation was present. A fresh remote read matched the known integration,
accepted-main and Task 2.3 tips. The exact eight-file source freeze was committed
without signing at `fd5735976eef1e2728ccf54726145501573db60d`; this documentation
closure records its evidence before normal integration publication. No accepted
main update, force push, history rewrite, release, tag or sensitive action belongs
to that publication.
