# Task 2.29 — Isolated genesis recipient-preflight probe preparation

Status: **TECHNICALLY VALIDATED / PENDING FOUNDER ACCEPTANCE** under D-026,
within the build-only scope below.
Baseline integration: `162f3b7b634633e2a5ab3011f0d746c4a4d15599`;
main remains `7b74be4b13c019b96a0c8abcbebfbcc361d31089`.

## Scope and boundaries

The next bounded prerequisite is reaching the production recipient preflight
through a real SBF entrypoint and signed CPI without exposing an initializer in
production. This task prepares two isolated nonpublishable probes, their host
boundary regressions and separately reviewed locked/offline compilation. **No SBF
probe is loaded or executed in this task.** Runtime closure remains subsequent
work; compilation cannot establish genesis total heap/compute or rollback.

The callee uses only the unchanged public
`preflight_approved_genesis_recipients`, with fixed 32/31-account topology and
recipient vault indices 0/255. It accepts the existing 313-byte model format,
choosing only shared/distinct manager-referrer roles; it exposes no initializer,
returns no authenticated facts, and invokes no System or Token operation.

The caller is a **synthetic validation ELF under the canonical Squads program
ID**, not the real Squads executable. A bounded parser reads the exact stored
single action and reconstructs its account order and requested privileges. It
requires canonical transaction/vault identities and checks current authority
identity using existing production code. It does not replace the callee's fresh
approval authentication. The outer writable proposal is downgraded to its exact
stored inner readonly privilege; the outer vault is a nonsigner and receives
only its canonical PDA signer through the SBF `invoke_signed` branch. Ordinary
host entrypoints reject before CPI; host tests invoke only read-only preparation.

Caller fixtures independently serialize through the existing Anchor/Borsh test
types. Their synthetic protocol/target accounts are sufficient for wire and
privilege tests, **not a successful production genesis preflight**. Literal full
metas/data/seeds, complete fixture preservation, all truncations, raw u32 count
overflow/zero, unsupported message extensions, canonical identity failures and
protected privilege escalations are covered. Callee tests bind fixed topology,
model rejection and the ordinary-host guard.

## Pinned standalone workspace and build guards

`validation/genesis-preflight-probes` contains two `cdylib`/`lib` crates. Its ordinary
non-test dependencies are already pinned Anchor **0.32.1** and the existing PIV1
path with `no-entrypoint`. This prevents the production entrypoint from being
linked into a probe while allowing the unchanged public library API. No new
registry package/version or production dependency is introduced. After the first
test compilation, the caller added the existing exact **SPL Token 8.0.0** as a
dev-only dependency: the independently serialized reused fixture references its
canonical program ID. This does not add a Token executable or CPI path.

The standalone lock was seeded from the unchanged root lock and resolved once
with pinned Cargo 1.97.1 `metadata --offline --format-version 1`. This offline
preparation command exited zero in **0.212962 seconds**; its local lock update added
only the two probe packages. All **155 registry entries** are an exact
name/version/source/checksum subset of the root's existing **166**; the complete
probe lock has 159 entries, including four local packages. The second offline
metadata command, adding only that caller dev-dependency edge, passed in
**0.142037 seconds** with the same 155 registry entries. Final lock SHA-256 is
`c5c095081125f70b52844de1e416dcdd48a2f52604414e38ebb61a7d72ad8b1c`.
Metadata, environment and command are retained at
`/tmp/piv1-t229-writer-preparation-20260921-a`. The release profile preserves
checked arithmetic, fat LTO, one codegen unit and the production build override.

`tools/build_genesis_preflight_probes.py` imports the unchanged production runner
only after checking its exact bytes against the reviewed SHA-256. It reuses tool,
production-source, raw-diagnostic, output and ELF guards without overriding helper
globals. Additive guards bind eleven probe inputs, the full resolved feature
graph, 155 checksum-bound archives and their complete extracted sources, all
relevant Cargo configuration absences, 107 protected production/harness inputs,
and three historical artifacts. Only fresh private `/tmp/piv1-genesis-probes-*`
outputs are accepted. Both artifact names are fixed; no loading, signing,
deployment, installation or network stage exists. Zero-exit stack/error diagnostics
and warnings reject. Failure records retain independent output/preservation and
complete log hashes. Runner tests use mocks and never execute a compiler.

## Reviews and actual evidence

Root verified `jerem`, the clean sole worktree, baseline and matching remote refs.
Separate scope/source review passed. Before any compilation, a test fixture field
was corrected from `key` to the actual `account_key`; raw-count and protected
inner-privilege regressions were added after review. These were preparation
corrections, not failed tests. The review receipt is
`/tmp/piv1-t229-pilot-review/rust-source-review.json`.

Root's retained-evidence check matched **92 production inputs, 105 historical
harness inputs, eight transport inputs, 128 logs, 119 harness tools, 101 target
tools and the Task 2.28 ELF**. Thus the final Task 2.28 **469 host tests +1 doctest,
24 SBF tests/70 cases**, and earlier **15 Node tests/eight old plus sixteen new
transport cases** remain attributed retained evidence, not rerun here. The
receipt is `/tmp/piv1-t229-pilot-review/retained-evidence.json`. Its first reader
expected a command `name` field rather than the recorded `label`; the receipt-only
reader correction preceded the passing audit and was not a test/build failure.

The first writer host command stopped during compilation: **exit 101**, in
**62.201080 seconds**, with no tests executed. The reused fixture needed an
explicit test dependency on `spl_token`; a separately captured executor backing
account also failed the common `AccountInfo` lifetime requirement. The correction
adds only the pinned dev-dependency and changes test backing to one local vector,
including the executor. Tests compare that entire actually invoked vector before
and after preparation as well as the original fixture. Probe runtime sources are
unchanged. Original logs and all 100 source hashes are retained in
`/tmp/piv1-t229-writer-host-20260921-a`; no successful result is claimed for this
attempt. Separate correction review passed in `rust-source-review-v2.json`.
The focused retry at `/tmp/piv1-t229-writer-host-20260921-b` passed **ten tests**
(seven caller, three callee), with zero failures, ignored tests or diagnostics,
in **1.004527 seconds**. All 100 inspected input hashes stayed unchanged. The
exact pinned command was:

```text
/home/jerem/.rustup/toolchains/1.97.1-x86_64-unknown-linux-gnu/bin/cargo test --manifest-path /home/jerem/piv1/validation/genesis-preflight-probes/Cargo.toml --workspace --all-targets --locked --offline --jobs 1
```

The writer then executed the separately reviewed mocked runner regressions once:
`/usr/bin/python3 -I -B tools/test_build_genesis_preflight_probes.py` passed **nine
tests**, zero failures, in **0.321660 seconds**. These tests invoke no compiler,
SBF loader or network. Normal unittest output is on stderr. Exact command,
environment, Python/pin identities and all 118 before/after probe/protected input
hashes are retained at `/tmp/piv1-t229-writer-runner-20260921-a`; all matched.

The writer and root Rust executions use the same task-local target cache with
debug information and incremental compilation disabled to
conserve disk; an independent rerun does not imply a second clean compilation.
Root owns all actual SBF compilation and final evidence review.

Root independently executed the first frozen **ten Rust boundary tests and nine mocked
runner tests**, zero failures or diagnostics. Its **four gates** also cover
doctest discovery (**zero doctests**, not a claimed passing doctest) and
warnings-denied documentation. Combined command time was **16.553346 seconds**.
Root verified all eight logs, twelve source/runner/pin hashes and four host tools.
Writer independently rechecked those hashes without rerunning any command.
The full command/environment records are at
`/tmp/piv1-t229-pilot-host-20260921-a`; `results.json` SHA-256 is
`79a700f0cec5c89a02e21ae99fce31a27d808fca38866f29676d748f07c9d5b8`.
Production and existing runtime suites were retained, not rerun.

The separate final Rust review is `rust-source-review-v2.json`; the exact runner,
pins, lock/feature closure and static-build command review passed in
`/tmp/piv1-t229-pilot-review/runner-command-review.json`. Reviewer inspection
verified eleven probe source hashes, 107 protected input hashes, three historical
artifacts, all 155 lock entries and the 22-script subset of the existing build
inventory. It did not execute a compiler or independently repeat every registry
source-byte check; the actual runner performs those checks before and after its
compilation stage.

The reviewed runner SHA-256 is
`7ae81f372ef9e9cab2c71e817940fe6a30cf2395f275241a0eb6c02d7ffdcab2`;
its initial pin companion SHA-256 was
`5f37fed4dbc482693177f52bebc29dc1733ed60be53e040d706d1e2b9552bb36`.
Root's first strict build at `/tmp/piv1-genesis-probes-build-t229-20260921-a`
returned **FAIL** despite Cargo exit zero. The target compilation took
**310.970210 seconds**, with **no stack/error diagnostics**, but one
`unused_must_use` warning and its compiler summary (two recorded warning lines).
The callee's successful observation was intentionally discarded but needed an
explicit `let _ =` binding in its SBF branch. The unchanged strict warning gate
correctly rejected the output. All source/package/tool/helper/pin/ref preservation
checks passed; this output remains preserved and no artifact from it is accepted.

The narrow correction adds only that explicit discard, preserving `?` error
propagation and all production/library behavior. No allowance attribute, diagnostic
suppression or gate change is used. The corrected callee SHA-256 is
`a9b53ffd6523f85d17519a386f339a148a19cf64f6c6c897663dda8e8834a0a8`;
only its source entry changes the final probe pin SHA-256 to
`43cf200ec459fb1e3c09d2dfb2222ad2100963098b415a5c4246276b0a4e2a48`.
The separate correction review passed in
`/tmp/piv1-t229-pilot-review/callee-discard-review.json`: removing only `let _ =`
reconstructs the prior callee hash, and all other pin fields and ten source inputs
are unchanged. Earlier host executions above belong to the pre-correction freeze.
The final writer rerun at `/tmp/piv1-t229-writer-host-20260921-c` passed **ten Rust
tests**, zero diagnostics or failures, in **0.623174 seconds**, with all 100 input
hashes preserved. Its `result.json` SHA-256 is
`d0e603f313c2af7d6c1c1b69ed166710e2537bc04c85d9e1ba525be8d2fac84f`.
The final mocked runner rerun at `/tmp/piv1-t229-writer-runner-20260921-b` passed
**nine tests** in **0.243366 seconds**, with all 118 input hashes preserved;
`result.json` SHA-256 is
`fb5cf36e0df17dfca8a12987df7324cec6f25c9a67415feddeac9894c6228f5f`.

Root independently repeated its **four gates** on the corrected freeze: **ten
Rust tests, nine runner tests, zero discovered doctests and warnings-denied
documentation**, zero failures or diagnostics, in **1.920979 seconds**. Evidence
is retained at `/tmp/piv1-t229-pilot-host-20260921-b`; `results.json` SHA-256 is
`b37420aef74a30e2a30fa09fe606aa53dd7f2c91d248a2746cccfcbf120326be`.
Writer rechecked all eight log hashes, twelve source/runner/pin inputs and four
host tool hashes against the recorded bytes.

Root's fresh second strict build at
`/tmp/piv1-genesis-probes-build-t229-20260921-b` passed with **zero diagnostics**;
target compilation took **315.410695 seconds**. Its 32 commands and 64 log hashes
are recorded. The runner verified eleven probe inputs, 107 protected inputs,
all 155 registry packages and unchanged tool/helper/pin/ref inputs. Three
historical artifacts and both rejected-build artifacts remain preserved. Root's
inspection is `/tmp/piv1-t229-pilot-review/target-final-evidence.json`;
the build `result.json` SHA-256 is
`feb58b36ffed2bfadc3d821b53425bd3f6aab8d810f5cdb28154f84e4a743605`.
The exact reviewed command, run from `/home/jerem/piv1` with only
`PATH=/usr/bin:/bin` and `LC_ALL=C` in the parent environment, is retained in
`/tmp/piv1-t229-pilot-review/target-command-b.json`; separate release is recorded
in `runner-command-review-v2.json` in the same review directory:

```text
/usr/bin/python3 -I -B /home/jerem/piv1/tools/build_genesis_preflight_probes.py --output /tmp/piv1-genesis-probes-build-t229-20260921-b --execute --approved-runner-sha256 7ae81f372ef9e9cab2c71e817940fe6a30cf2395f275241a0eb6c02d7ffdcab2 --approved-pins-sha256 43cf200ec459fb1e3c09d2dfb2222ad2100963098b415a5c4246276b0a4e2a48
```

Both outputs are under that directory's `target/sbpf-solana-solana/release`:

| Artifact | Bytes | SHA-256 |
| --- | ---: | --- |
| `piv1_genesis_preflight_caller.so` | 64,368 | `938e6c1c63554ac26f75f3c4daef614087051ea88d8f9eb130692940278508b2` |
| `piv1_genesis_preflight_probe.so` | 180,232 | `1d87ab760fae785cc74a8ef069722ee5bb3c48ad43e029118eba512cd1c97c53` |

Writer checked these exact sizes/hashes and the result hash. Both ELF byte streams
equal those from the rejected first build: the explicit discard corrects the
source warning without changing generated code. This equality does not accept
the rejected build or weaken its strict gate.

Separate static artifact review passed in
`/tmp/piv1-t229-pilot-review/artifact-review-v2.json`, SHA-256
`f71003a2d2542f38f7efa96a31813ec802180c62de21f5b96429eaa395db3044`.
This clarifies the expected standard `custom_panic` export alongside `entrypoint`;
the original inspection receipt is preserved and artifact/build results are unchanged.
Reviewer independently parsed both ELF identities and inspected all 64 retained
logs. Disassembly covers all 44,352 caller and 147,112 callee `.text` bytes;
1,186 and 4,789 direct frame-memory accesses respectively remain within
`[-4096, -1]`. This checks direct `r10` accesses only, not pointer aliases, the
complete dynamic call graph, total heap/compute, runtime success or rollback.
Reviewer ran no compiler, host test binary or target runtime. Neither probe was
loaded or executed. Root gates publication on separate final documentation review.

## Files and publication boundary

This task adds nine files in `validation/genesis-preflight-probes`: the workspace
manifest, lock and README, plus each crate's manifest, `src/lib.rs` and
`tests/boundary.rs`. It also adds `tools/build_genesis_preflight_probes.py`,
`tools/genesis_preflight_probe_pins.json`,
`tools/test_build_genesis_preflight_probes.py` and this report. Root maintains
the six shared documents: `AGENTS.md`, `README.md`,
`docs/PIV1_CODEX_EXECUTION_PLAN.md`, `docs/PIV1_MASTER_SPEC.md`,
`docs/PIV1_PILOT_STATE.md` and `docs/PIV1_TEST_PLAN.md`. These are nineteen files
in total; production and historical validation inputs are unchanged. Root owns
the reviewed commit and normal integration-only publication. Git records the
resulting commit identity; the verified baseline and unchanged main are above.

## Completion limits

Production Rust, its tests, native ABI, dependencies, economics, existing runtime
harness/pins/runners and historical artifacts remain unchanged. No Mainnet action,
deployment, chain operation, fund movement, key creation/signing, secrets access
or authority transfer is performed. Real probe execution, actual Squads artifact
verification, Instructions-sysvar/Clock/Rent observations and complete account
effects require later review and execution. Funding provenance, later Token-native
donations, full recipient control and initializer resource/CPI/rollback evidence
remain deferred. Technical validation is not founder acceptance or Testnet
readiness. Root owns shared documentation, the reviewed task commit and ordinary
integration-only publication; main authority is unchanged. Checkpoint and STOP
after this bounded task; Task 2.30 is NOT STARTED.
