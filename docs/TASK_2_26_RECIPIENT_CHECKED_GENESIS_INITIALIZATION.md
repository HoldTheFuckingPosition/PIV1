# Task 2.26 — Recipient-checked normalized genesis initialization

Status: **TECHNICALLY VALIDATED / PENDING FOUNDER ACCEPTANCE**.
Base: `3b45241aec5e0b6a9405f5df2516da66c42c6485`, `integration/piv1-testnet`.
The founder resumed one economical bounded task under D-026, then checkpoint and
STOP. Main remains `7b74be4b13c019b96a0c8abcbebfbcc361d31089`; D-028 does not
authorize integration of this later task into main.

## Fixed same-call profile

`initialize_approved_genesis_with_checked_recipients` combines Task 2.24's
normalizing initialization with Task 2.25's recipient identity checks. Its
`RecipientCheckedGenesisRoles` contains one authoritative initialization mapping
and a `GenesisRecipientSelection` with only two account indices and two u8 vault
witnesses. A selection cannot supply another preflight mapping or replace the
approved Config keys. The new profile always normalizes Token-native prefunds.
All earlier public function signatures and their existing profiles remain.

The allocator performs the same complete fresh genesis/Squads/Jito preflight
once, with the same single runtime height, Clock and Rent. After existing payer
validation, a crate-private helper checks the recipients against that exact
fresh preflight and retained Rent. It does not call the public recipient
preflight or authenticate the full genesis message a second time. No public
effects function accepts detached preflight, recipient or allocation evidence.

Before any CPI, each recipient must satisfy the existing exact approved-key,
same-multisig canonical vault PDA, positive rent-covered balance, System owner,
empty data, nonexecutable and key/backing separation constraints. The additional
payer, System Program and Token Program roles are explicitly excluded. Recipient
indices cannot replace each other or any existing protected role. Earlier
initializer/mint/payer validations retain their existing ordering and failures.

Private observations retain both identities, exact balances, owner, empty-data
length, executable flag and signer/writable flags. Their exact preservation is
checked immediately after **every successful System and Token CPI**, after the
allocation batch and after final initialization. No recipient borrow guard is
retained across CPI. A failing invocation returns its original error before any
recipient postcheck, including when the modeled invocation also corrupts a
recipient. Production has no undo mechanism; errors must reach the outer
transaction boundary.

Normalized custody remains unchanged: the external payer funds all sixteen
original rent shortfalls; native excess from the two still-System-owned Token
PDAs moves above PendingSol's already-funded rent floor; both Token initializations
and nine exact state envelopes complete in the same call. Initial pending
ledgers remain zero. Subsequent authenticated pending reconciliation separately
recognizes the full contribution once, preserving HWM/history, KIF, carry,
operations and both recipients.

The library adds `GenesisAllocationError::Recipient`,
`GenesisInitializationError::Recipient` and
`GenesisRecipientError::ObservationMismatch`. Exhaustive Rust enum consumers
must handle the new variants. No instruction error mapping, serialized error,
native selector, 313-byte model format, schema, dependency or economic rule
changes. Old initialization/allocation profiles supply no recipient selection;
their existing error behavior is preserved.

## Writer validation

One focused direct pinned command ran with an explicit clean offline environment,
existing cache and one job:

```text
/home/jerem/.rustup/toolchains/1.97.1-x86_64-unknown-linux-gnu/bin/cargo test --locked --offline --jobs 1 -p piv1 --test genesis_initialization --test genesis_recipients
```

**22 initialization tests +8 recipient-preflight tests PASS**, including the
unchanged sixteen prior initialization tests and all eight Task 2.25 tests.
First execution passed with no failures, ignored/filtered tests, diagnostics or
corrective retry. Command time: **8.595647234 seconds**; suites: **3.18 seconds**
and **0.49 seconds**. All 92 source/manifest inputs stayed unchanged during the
run. Cargo, Rustc and Rustdoc hashes matched the retained direct pinned tools.

Evidence: `/tmp/piv1-t226-writer-20260920-e5n764bq`, containing the exact command,
environment, tool hashes/versions, before/after input manifests, logs and result.

| Log | SHA-256 |
| --- | --- |
| `stdout.log` | `f5fc84388b9a5395a7df2007b0dbf6f87a4f7a96bdf270e8d16d4ea8c76c4f13` |
| `stderr.log` | `57101587b73b7d2d87b2ca29602f689dff12e655c6c7fd9a799ebf24eba62c7d` |

Six new grouped tests reuse the existing independent System/Token CPI oracle.
They cover 24 positive worlds across program identity, initial pause, protocol
receiver sharing and prefunding/recipient metadata; original payer rent debit,
exact custody and every unrelated account; actual pending recognition twice;
invalid recipients and conflicting roles before effects; both recipients'
native balance, owner and data tampering at all forty CPI boundaries; mutable
borrow conflicts before and after effects, successful shared reads and no
retained own guards across CPI; every invocation failure before/after effects;
invocation-error precedence over recipient corruption; fresh approval, replay,
later Token-native donation rejection and closed native dispatch.

Raw failed host calls retain partial effects. Only the explicit clone/discard
test model restores the fixture. This is not actual Solana rollback evidence.

## Root validation and separate review

Root personally inspected the three implementation diffs, fixture changes and
six new regression groups, then independently executed **468 host tests +1
doctest/eight gates PASS**. First execution passed with zero failures, ignored
tests or diagnostics; neither writer nor root needed a corrective retry in this
task. Gate-command wall time: **99.151361704 seconds**. The same direct pinned
Rust 1.97.1 tools and a clean explicit offline environment were used. Each command
below included `--locked --offline --jobs 1`:

```text
cargo test --workspace --all-targets --quiet
cargo test --workspace --doc
cargo check --workspace --all-targets
cargo check --workspace --all-targets --all-features
cargo check -p piv1 --all-targets --features no-entrypoint
cargo check -p piv1 --all-targets --features cpi
cargo check -p piv1 --all-targets --features idl-build
cargo doc --workspace --no-deps  # RUSTDOCFLAGS=-D warnings
```

Evidence: `/tmp/piv1-t226-pilot-host-20260920-a`, containing exact commands/
environment, before/after source manifests, tool hashes, all gate logs, results
and pilot summary. All 92 inputs match root inspection and both executions.
Root verified sixteen gate logs, two writer logs and three actual pinned tools.
Exactly four Rust inputs changed from Task 2.25. The `results.json` SHA-256 is
`2a056c41c49880dce78710b766e3409e051ba523ceff204678196e4da570b260`.

Separate scope, source/test and execution-evidence reviews passed without an
actionable blocker. The reviewer independently verified all 92 inputs, eighteen
logs and three tools, and ran no tests/builds. Final documentation review precedes
publication. Reviewed source/test SHA-256:

| File | SHA-256 |
| --- | --- |
| `programs/piv1/src/genesis_recipients.rs` | `ae262f8404f6caf3dff5aac0e8bd844a503511cd6e8f0c42d1dd0686f52989fc` |
| `programs/piv1/src/genesis_allocation.rs` | `d7878a592ad670396a9158e7cd5c1edc31f390302af373ac16d8cf69b56e968e` |
| `programs/piv1/src/genesis_initialization.rs` | `ebcbe6acc1eb3666e4c7fdbdd6cbd1f93c45d28bd75bad1824b86bcc1b248aab` |
| `programs/piv1/tests/genesis_initialization.rs` | `d49612e303f2e8cfc8513e292b2161dd12cbde283813d8988ca66bb1351674c4` |

Six earlier transport-template inputs remain unchanged. Nine Node tests/eight
packet cases are retained Task 2.23 evidence of its original topology, not rerun
or extended to the new profile. No SBF/runtime build or tests were refreshed;
historical Task 2.14 evidence still applies only to its earlier artifact.

## Remaining limits and handoff

The complete fixture now contains **35/34 AccountInfos** (distinct/shared protocol
receiver): the previous initializer's 33/32 plus two recipients. It needs new
transport and runtime evidence; Task 2.23 packet results cover only the earlier
exact template. Present recipient identity and preservation still do not prove
exclusive four-of-six spending, absence of Squads spending limits or executable
stale actions, real identity/control or the live Squads artifact. Funding
provenance, operational baseline, later Token-native donations and current-source
SBF/resource/rollback proof remain deferred before native initializer exposure.

Writer files: `programs/piv1/src/genesis_recipients.rs`,
`programs/piv1/src/genesis_allocation.rs`,
`programs/piv1/src/genesis_initialization.rs`,
`programs/piv1/tests/genesis_initialization.rs`, and this report. Root updated
`AGENTS.md`, both READMEs, master specification, execution plan, test plan and
pilot checkpoint: twelve files in total. Root owns final documentation review,
narrow staging, normal commit and integration-only publication under D-026.
Git records the exact task commit/publication identity; verify remote refs and
clean worktree after publication. Main remains at the baseline recorded above.
Task 2.27 is NOT STARTED; save/STOP after reviewed publication. No SBF build,
RPC, deployment, Mainnet action, fund movement,
key creation/signing or authority transfer occurred. AI-assisted review is not
a professional independent audit.
