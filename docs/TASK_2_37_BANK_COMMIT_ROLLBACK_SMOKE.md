# Task 2.37 — Native Bank commit/rollback smoke

Status: **TECHNICALLY VALIDATED / PENDING FOUNDER ACCEPTANCE**. Starting integration is
`850bc6d1bd2fb4137ef6cb16cc26e78634f6224a`; main remains
`4cc4ea11e87c2f1a2f85b9ed48358f2a881821b3`. D-026 covers this isolated technical
work and reviewed integration-only publication, without additional founder
acceptance, economics, main integration or live-operation authority.

## Purpose and authority boundary

Task 2.36 authenticated the public Bank dependency closure and reviewed an unsigned
local entry without compiling or executing it. This task adds the smallest actual
Bank smoke before attempting PIV1's initialization artifacts: native System
success, a late instruction failure, retained fees, replay rejection and a
distinct successful retry on the same Bank. PIV1 production, economics, existing
Mollusk harnesses, Task 2.31 ELFs and earlier evidence remain unchanged.

One delegated writer edits only the isolated manifest, `tests/bank.rs`, workspace
README and this draft. Root owns the lock/metadata/native-tool preparation,
resource-controlled build/runtime, shared documents, actual evidence and Git.
A separate reviewer checks source, commands and actual results. The writer does
not run Cargo, tests, compilers, runtime, network, installation or cleanup.

## Implementation and exact oracles

The [workspace README](../validation/genesis-bank-runtime/README.md) documents the
locked API path and fixture. All messages use `Transaction::new_unsigned` with
two zero signature placeholders and `Bank::prepare_entry_batch`, followed by
`load_execute_and_commit_transactions`. The entry assumes prior signature
verification. No signer, keypair helper, ProgramTest, nonce, RPC or actual
blockchain is involved. Fixed public-byte fixtures are local synthetic identities,
not proposed deployment or recipient addresses.

An explicitly constructed genesis has time 1,800,000,000, 6,960-lamport-per-byte
rent, no activated features and 5,000-lamport signature fees. The remaining SDK
defaults are source-pinned and the complete genesis hash is emitted. The Bank
uses one fresh retained AccountsDB path and BankForks only to keep its program
cache's fork graph alive. All messages run on slot zero without a parent or state
reset. Every before/after snapshot enumerates all accounts in the Bank slot and
rereads each independently through `Bank::get_account`.

| Case | Native instructions | Required result | Persisted account oracle |
| --- | --- | --- | --- |
| Success | Source transfers 10,000 to first destination | `Ok(())` | Exact transfer and 10,000 payer fee; all other complete accounts unchanged |
| Late failure | Source transfers 70, then attempts 1,000,000,000 to second destination | `InstructionError(1, Custom(1))` | All non-fee accounts exactly equal successful pre-state; payer loses exactly 10,000 |
| Exact replay | Byte-identical failed message and zero placeholders | `AlreadyProcessed` before commit | Every account exactly equals failed post-state, including payer |
| Distinct retry | Same first 70 transfer, second amount changed to 9 | `Ok(())` | Exact two transfers and one 10,000 fee from actual failed/replayed state |

The failure log must contain first-instruction System success before the exact
insufficient-lamports line showing 9,989,930 remaining. That value includes the
first leg's transient debit; a generic failure or pre-execution rejection cannot
satisfy this oracle. All account metadata and data bytes are compared, including
a 64-byte unrelated sentinel with another synthetic owner, the System executable
and runtime sysvars. Five complete snapshots, four serialized messages and four
commit/error observations are emitted as structured JSON lines for independent
inspection. No manual post-transaction account storage, status-cache clearing,
Bank reconstruction or hidden second execution supplies the asserted state.

## Dependency and resource bounds

Only already-locked serde_json 1.0.151, solana-svm 4.2.0,
solana-transaction-error 3.3.2, solana-instruction-error 2.4.0 and
solana-system-interface 3.2.0 become direct test imports. A failed compile exposed
the additional need for a direct exact `five8_core 0.1.2` dependency with `std`:
the locked `five8 1.0.0` edge selects that older core, whose `DecodeError` error
traits are feature-gated. The transitive SDK decoder requires those traits.
Separate source review approved enabling their existing implementations; no
keypair helper is called by this harness. Root verified all 589 registry lock
entries and 552 host package identities unchanged. Only the old core gains `std`
and the local root gains six direct edges at that stage. A later strict-build
diagnostic required the separately reviewed local override recorded below; no
registry cache bytes or production dependency are changed.
Historical `src/lib.rs`
and `preparation.json` remain unchanged; their old hashes are historical preparation
evidence, not pins for the new test source or manifest.

The harness requires `RAYON_NUM_THREADS=1` and an absent absolute AccountsDB child
under a canonical existing parent. Foreground/background/index-flush pools use
one thread; the index uses two in-memory bins and capacity 128. Read-cache low/high
watermarks are 2/4 MiB with 16 shards, and the write-cache limit is 8 MiB. The
unchanged upstream async account hasher uses four threads with 8-MiB stacks.
These bounds do not cover total memory; root separately supervises process-group
RSS/swap, free disk and timeout with a single compiler/runtime slot. No historical
cache cleanup or archive deletion is part of this task.

`tools/validate_bank_smoke.py` finally binds 217 repository inputs (169 before
the vendor/provenance addition), 158 installed tools,
50 aliases, 1,962 files in three public Perl library trees and all 589 authenticated
archives/source trees. It refuses existing output directories and Cargo config or
credential files by existence without reading them. Builds use an explicit
scrubbed environment, locked/offline Cargo, one job, no incremental compilation
and zero test/dev debug information. No installation or network fetch is needed.

The Linux x86_64 child filter denies new sockets, connect/bind/listen/accept and
io_uring while permitting only local AF_UNIX socketpairs for compiler IPC.
Inherited descriptors are closed except stdin on `/dev/null` and the two supplied
ordinary output logs. The child has
no-new-privileges, no core dumps, a 1-GiB per-file and 4-GiB per-process virtual
address-space ceiling. Every 0.25 seconds root samples total process-group RSS
plus swap against 2.5 GiB, filesystem availability against a 2-GiB reserve and
elapsed time against 1,800 seconds for build or 60 for runtime. Breaches kill the
process group and retain logs. These are sampled guards, not atomic cgroup limits
or a general filesystem sandbox. Shared mappings can be counted more than once.
The smoke does not authorize io_uring; attempted use fails closed.

## Actual validation and evidence

Root evidence is in `/tmp/piv1-t237-pilot-review`; independent review receipts are
in `/tmp/piv1-t237-reviewer`. Earlier Rust/SBF/Node evidence is retained, not rerun.
The public source cache remains `/tmp/piv1-t236-preparation-20260928-a/cargo-home`.
The Task 2.35 recovery archive and all historical outputs remain protected.

Actual attempts, including failures:

1. Takeover matched 162 prior protected inputs, 119 tools/nine aliases and ten
   artifacts. Native review covers all 56 selected build scripts and bundled
   programs, public installed Perl/make/shell/linker prerequisites and their
   hashes. The selected native build paths write to their fresh Cargo output;
   no system install or download path is selected. This is not an exhaustive
   compiler/header/OS image provenance claim.
2. A read-only network-namespace probe failed because the environment refused
   the user namespace map. No bypass or privilege escalation followed. Root used
   the additional child socket filter instead. The pinned rustfmt executable is
   absent; that formatting command failed before execution. No tool was installed.
3. Initial offline metadata passed and preserved the complete registry graph.
   Eleven initial runner regressions passed. First guarded build `...-a` stopped
   before the first rustc process: `socketpair(AF_UNIX, SOCK_SEQPACKET|SOCK_CLOEXEC)`
   was denied. A retained syscall trace reproduced the precise cause. The narrow
   reviewed IPC correction added one actual local-pair/foreign-domain regression.
   All **12 final runner tests passed**, first execution after correction, in
   0.264764 seconds (unittest 0.096 seconds). Network-denial, timeout, resource
   termination, source drift, environment and output-preservation guards remain.
4. Fresh guarded build `...-b` compiled dependencies but failed after 60.756769
   seconds with two E0277 diagnostics in `solana-keypair 3.1.2`: the locked older
   `five8_core` lacked its `std` error traits. Peak sampled group RSS+swap was
   539,709,440 bytes and minimum free space 7,791,276,032; no guard triggered.
   The reviewed feature-only correction above preserves every version/checksum.
   Guarded offline metadata `...-metadata-t237-20260928-c` passed and independently
   confirmed that exact feature/root-edge delta before a fresh build.
5. Build `...-c` reached compiler exit zero in 512.620487 seconds, with no structured
   compiler diagnostics. The unchanged strict gate then rejected Cargo's future
   incompatibility warning for `proc-macro-error2 2.0.1`. Its retained report shows
   E0365: private `extern crate proc_macro;` is publicly re-exported. This attempt
   remains **FAIL**, and its 49,375,560-byte binary is not executed. Peak sampled
   group RSS+swap was 950,673,408 bytes; minimum free disk was 6,543,863,808.
   The exact rejected binary hash is
   `1153ba8f843a47614f7fe93ab3f0460bbb22a3b3256e3c0d228ced696d7068ce`.

The public [registry response](https://crates.io/api/v1/crates/proc-macro-error2)
observed during correction lists 2.0.1 as the newest published version; the response
is retained with SHA-256 `d44a473d4b641bc20b1b0ce0434d468891b3aabf116b60b4584921849cfce578`.
Root therefore scoped a genuine local compatibility correction: an isolated
provenance-pinned copy of that exact authenticated package. Root and reviewer each
compared all 47 archive files: 46 are byte-identical, both licenses are preserved,
and only `pub` is added to the offending declaration. The registry cache stays
byte-identical. This supersedes the initial no-local-override implementation
assumption; it does not change PIV1, economics, Bank or the warning gate.
`vendor-provenance.json` records the archive, upstream VCS and every member hash.
The nested upstream workflow is preserved package data, not a root CI workflow.

Guarded offline metadata `...-metadata-t237-20260928-d` passed: substituting only
the old registry identity with the local package identity makes the complete
resolve graph identical to the preceding graph. Versions, features and dependency
edges are otherwise unchanged. There are now **588 resolved registry packages
plus one local override**; the host graph has **551 registry packages plus that
override**, with the root as the only workspace member. All original 589 archives
and source trees remain pinned for provenance, alongside the 47 vendor files and
their receipt. The final lock SHA-256 is
`5d6e16a79cd083400d4f09bbd32de6ce5cfa590ae5318bdd57799ad9107b7400`.
The final pin file SHA-256 is
`6753c88d61c911289833a57a0e0148bc91954feb1b18f8db43d121c0ae124a08`.

An optional targeted `cargo check --locked --offline --jobs 1 -p proc-macro-error2
--lib --message-format=json` attempt in
`/tmp/piv1-bank-smoke-vendor-check-t237-20260928-a` failed before compilation:
Cargo's feature resolver panicked while requesting a NormalOrDev feature set for
this host-only dependency. Its full graph diagnostic and exit-101 receipt are
retained. This is not a passing check or evidence about the patch. The normal
full-workspace build must compile the dependency in its actual host context and
satisfy the unchanged strict diagnostics gate.

Build directories use `/tmp/piv1-bank-smoke-build-t237-20260928-` plus the listed
suffix. Complete commands, environments, exit codes, timings and log hashes are
in each `*.started.json`, `*.json` and `result.json`. Failed attempts are preserved;
no failed result is treated as success.

The fourth fresh full build, `...-d`, passed the unchanged strict gate: compiler
exit zero, no structured or raw warning/error diagnostics, **498.474109 seconds**,
peak sampled group RSS+swap **941,891,584 bytes**, minimum free disk
**5,295,411,200 bytes**. No resource guard triggered. Root statically checked the
49,375,640-byte ELF64 x86_64 host executable and its four needed runtime libraries,
which are in the installed-tool pins. It is
`/tmp/piv1-bank-smoke-build-t237-20260928-d/target/debug/deps/bank-687e8606c0fd7b0c`,
SHA-256 `c3444648e6856c54af544e7813c566bd3eab645dccdd2e3193b0f12b3d3502d8`.
Build stdout SHA-256 is
`da0de61c01d4118c281af4443b6f6a53630f7fd53843c81fce64c2b775c1e183`;
stderr SHA-256 is
`cd31f7678b42dce7183aeb8f85c1efbc6cf8aa2a702c2302aed9d558551d8c06`.
Separate exact-binary and runtime-command review passed before execution.

Root's final runner/build commands are:

```sh
/usr/bin/python3 -I -B tools/test_validate_bank_smoke.py
/usr/bin/python3 -I -B tools/validate_bank_smoke.py build --output /tmp/piv1-bank-smoke-build-t237-20260928-d
/usr/bin/python3 -I -B tools/validate_bank_smoke.py run --output /tmp/piv1-bank-smoke-run-t237-20260928-a --build-output /tmp/piv1-bank-smoke-build-t237-20260928-d --approved-sha256 c3444648e6856c54af544e7813c566bd3eab645dccdd2e3193b0f12b3d3502d8
```

The **first actual Bank execution passed**: one Rust test, four messages and five
complete snapshots of 20 accounts, totaling **100 full account records** in 115
JSON observations. Root and reviewer each independently parsed the actual output,
decoded the four serialized legacy messages and compared every complete account
against exact deltas. All intended oracles above passed without test-source or
oracle changes. Observed compute units were 150 for success, 300 for late failure,
no execution for replay, and 300 for retry. Exact payer fee debits were
`[10000, 10000, 0, 10000]`.

The test reports 0.04 seconds, with 0.267182 seconds guarded wall time and empty
stderr. The largest sampled process-group RSS+swap observation was 15,921,152
bytes; **the test is shorter than the sampling interval, so this is not a runtime
peak-memory measurement or capacity requirement**. No guard triggered. Runtime
stdout SHA-256 is `aaea989aea4b44457717b2fe455d5ca951776894a000b2208b1e124aed4dc213`.
Root's independent inspection command was:

```sh
/usr/bin/python3 -I -B /tmp/piv1-t237-pilot-review/inspect_bank_evidence.py /tmp/piv1-bank-smoke-run-t237-20260928-a/runtime.stdout /tmp/piv1-t237-pilot-review/bank-evidence.json
```

That inspector SHA-256 is
`826564547bd2ca699e69fc805de07840425bf37eb23ca42f7d5eda402241fdb6`.
Root rechecked all 217 input hashes, 158 tools/50 aliases, public Perl trees and
all original 589 registry archives/source trees after execution. Exactly 164
pinned baseline files and ten historical artifacts remain unchanged; only this
workspace's manifest/lock change among preexisting pinned inputs. The rejected
build-c binary is unchanged. Task 2.35's recovery directory remains present and
untouched; its entire archive was not rehashed again. No prior suite was rerun.
Observed free space after verification is 5,297,123,328 bytes (about 4.93 GiB).

Separate receipts under `/tmp/piv1-t237-reviewer` cover native/source review,
both compatibility corrections, the exact graph, every build command amendment,
the final artifact and independent runtime inspection. The runtime receipt is
`runtime-evidence-review.json`, SHA-256
`6d6e4f562a98b222f0bda662392b67f0db7aeba49a8b6e78e1908ca50689db45`.
The reviewer parsed evidence independently and did not execute a second Bank run.
Final shared-document and publication review precedes root's normal unsigned
integration-only commit/push; Git records the resulting identity.
The default indexed whitespace check flagged only the 201 original CRLF endings
in the byte-identical upstream Apache license. Root preserved those authenticated
bytes; `git -c core.whitespace=blank-at-eol,blank-at-eof,space-before-tab,cr-at-eol
diff --cached --check` passed. Both observations are retained in `whitespace.json`
and its logs; source/test/compiler gates were unchanged.

## Limitations and continuation

Even a passing run proves only this native System smoke through the real
in-process Bank/AccountsDB account-saver and reread path. It does not execute PIV1,
the synthetic initialization caller/callee or Token wrapper. It does not establish
disk durability, Bank freeze/root behavior, ledger replay, validator consensus,
signature validity, actual Squads, full recipient control, funding provenance,
Testnet deployment or Mainnet readiness. Successful future artifact loading needs
separate feature, loader, resource and initializer oracles.

Changed files are the isolated manifest/lock, new `tests/bank.rs`, workspace README,
47 vendored files and their provenance JSON; `tools/validate_bank_smoke.py`,
`tools/test_validate_bank_smoke.py`, `tools/bank_smoke_pins.json`; this report and
six shared documents (`AGENTS.md`, root README, pilot checkpoint, execution plan,
master specification and test plan). Existing `src/lib.rs` and `preparation.json`
remain unchanged. Root records the exact scope and normal integration commit after
review. Main remains unchanged. Next prepare exact Task 2.31 SBF loading and Bank
initializer oracles, with reviewed loader/features/resources; that work is not
started. Checkpoint and STOP before Task 2.38. No Mainnet
operation, deployment, fund movement, key creation/signing, secrets access,
RPC/chain operation or authority transfer occurs. Technical validation is not
founder acceptance or a professional independent audit.
