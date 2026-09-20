# Task 2.24 — Genesis Token-native prefund normalization

Status: **TECHNICALLY VALIDATED / PENDING FOUNDER ACCEPTANCE**.
Base: `7b74be4b13c019b96a0c8abcbebfbcc361d31089`, `integration/piv1-testnet`.
The founder requested one economical development session, then checkpoint and
STOP. This D-026 task adds a distinct genesis-only custody normalization path;
it does not extend D-028's earlier main-integration authorization or expose a
native initializer. The existing raw-preserving public APIs remain compatible.

## Bounded same-call behavior

`initialize_approved_genesis_accounts_normalizing_token_prefunds` freshly composes
the same approved 313-byte parameters, accounts, Squads authorization, Jito
identity, target observations and external rent-payer checks as Task 2.22. It
uses the same single Clock/Rent context and rejects detached observations or
allocation receipts. Canonical Token-program/mint role and mutable-borrow checks
still precede every System effect.

The internal allocation mode prepares all original rent shortfalls and two
possible native transfers before the first effect. For each future Token target:

```text
excess = max(original native lamports - future 165-byte Token rent floor, 0)
final Token native balance = future Token rent floor
final PendingSol balance = max(original PendingSol balance, native rent floor)
                         + both checked Token excesses
external payer debit = sum of all sixteen ORIGINAL rent shortfalls
```

Each excess, their total and the final PendingSol balance use checked arithmetic.
An unrepresentable total or destination rejects before funding or allocation.
The external payer retains its own rent floor. Contributions cannot replace any
of PendingSol's original missing rent: its original top-up occurs at target slot
9, before the two Token-target transfers at slots 14 and 15. Each nonzero transfer
uses the source target's canonical PDA seeds and pinned System transfer encoding
while that source is still writable, empty and System-owned, before its Allocate
and Assign calls. Zero excess produces no transfer.

Full target/payer postconditions follow every CPI, including exact source and
destination deltas and untouched targets. Private final balance expectations feed
the existing Token/state completion checks. Both legacy Token initializations
and all nine exact zero-history state envelopes still complete in the same call.
The final normalized path additionally requires `economic_observation` to pass;
both Token accounts have exact rent, canonical bytes and zero JitoSOL units.
The mint and all unrelated accounts remain unchanged. Other target prefunds,
including operational and state-account lamports, receive no new classification
or movement beyond their original rent-only top-ups.

This is **custody normalization, followed separately by ledger recognition**.
Initial Config pending ledgers remain zero. The existing authenticated
`execute_pending_reconciliation` subsequently recognizes the entire PendingSol
economic balance once; it changes neither historical principal nor HWM, KIF,
carry, operational balances or any other Config field. The treatment follows
the confirmed economic-surplus rule in master specification section 7 and
P-014–P-016; it does not select an operational funding baseline.

Both `initially_paused` values are supported only for fresh approved bootstrap.
An already initialized Config, paused or otherwise, rejects before effects.
No general exception to economic pause gates is introduced. Every error must
propagate to the outer transaction; production contains no undo mechanism.

## Writer execution evidence

One direct pinned host command ran with a clean explicit environment and existing
cache, without dependency changes or network access:

```text
/home/jerem/.rustup/toolchains/1.97.1-x86_64-unknown-linux-gnu/bin/cargo test --locked --offline --jobs 1 -p piv1 --test genesis_initialization
```

**16 tests PASS**, including all ten retained Task 2.22 tests and six new tests.
First execution passed with no corrective retry, failures, ignored/filtered tests
or compiler diagnostics. Command wall time: **7.022202463 seconds**; test suite:
**2.36 seconds**. All 90 source/manifest inputs stayed unchanged during execution.
Direct Cargo, Rustc and Rustdoc binary hashes matched the retained pinned tools.

Evidence directory: `/tmp/piv1-t224-writer-20260920-i_4xbogh`. It records exact
command/environment, tool hashes/versions, before/after inputs, result and logs.

| Log | SHA-256 |
| --- | --- |
| `stdout.log` | `7ae5be72b13a73f0ed03286fba6bb52ea0a3bddff7182f37ed74492ad7827187` |
| `stderr.log` | `7736dbb3ccec53eb24cb6b984f926718f7cef607832ca28f252fb1e80332ff29` |

The six new tests cover 48 success worlds across two program identities, shared
or distinct protocol receivers, both initial pause flags and six prefund cases;
zero/partial/exact/excess balances, representable `u64::MAX` cases and exactly-full
PendingSol; aggregate/destination overflow before effects; independent System
bytes/metas/source seeds and original-rent preservation; all other account and
Config-field preservation; actual authenticated pending recognition with byte
persistence twice; fresh approvals, inner/outer payer signatures, role aliases,
borrow failures and insufficient rent; every one of the 40 CPI failure boundaries,
before and after effects; false/tampered sweep and Token success; replay, partial
initialization and later donations to already initialized Token accounts.

Direct failed host invocation deliberately retains partial effects. A separate
explicit clone/discard model restores the complete fixture on failure. These
are distinct observations and do not prove actual Solana transaction rollback.

## Root execution and separate review

Root personally inspected the frozen implementation and regression tests, then
executed **454 host tests +1 doctest/eight gates PASS** using the same direct
pinned Rust 1.97.1 toolchain and clean explicit offline environment. Every command
below used `--locked --offline --jobs 1`; Cargo, Rustc and Rustdoc used their
absolute pinned paths rather than toolchain dispatchers.

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

First execution passed: zero failures, ignored tests or compiler diagnostics,
with no corrective retry. Total gate-command wall time: **93.455350161 seconds**.
All 90 source inputs match root inspection and both writer/root before/after
manifests. Root verified all sixteen gate logs, both focused-run logs and three
actual pinned tool hashes. Evidence directory:
`/tmp/piv1-t224-pilot-host-20260920-a`, including exact commands/environment,
source manifests, logs, `results.json`, preservation record and pilot summary.
The `results.json` SHA-256 is
`3b045ff37fc2b5b9a209fbf155d0161756009c160bcf767421243189562a75e1`.

The separate reviewer passed the bounded scope, frozen source/tests and execution
evidence, independently verifying those 90 inputs, eighteen logs and three tools.
The reviewer executed no tests/builds and found no actionable blocker. Final
documentation review precedes publication. Source/test SHA-256:

| File | SHA-256 |
| --- | --- |
| `programs/piv1/src/genesis_allocation.rs` | `d3ce549d9095334e5e1d2a15f3cd3db115feeb708a344b6e6299c0ef060a621b` |
| `programs/piv1/src/genesis_initialization.rs` | `08d820bbe14bc3d906b8cc61a5b805c8818702fd5002071734507f0406596f41` |
| `programs/piv1/tests/genesis_initialization.rs` | `5d8451d1b2234e48a3ff604211ef774a60469449448d73da5c73173aa49b8450` |

Root verified six unchanged transport-template inputs: the two Task 2.23 CJS
files, approved message/account-order model inputs and two spike manifests.
The prior **nine Node tests/eight packet cases** are retained evidence, not rerun.
No SBF/runtime tests or build were refreshed. Passing host models does not prove
native exposure, Solana rollback or public-Testnet readiness.

## Remaining limits and handoff

Later native donations to already Token-owned accounts still produce
`UnsupportedTokenNativeExcess`; this genesis path cannot extract them. General
economic-vault normalization, operational funding provenance and advance history,
recipient control, actual Squads/ALT lifecycle and current-source SBF/resource/
rollback proof remain separate prerequisites before native initializer exposure.
The exact genesis message topology and existing unsigned transport template are
unchanged; retained Task 2.23 wire evidence is not a new transport execution.
Historical Task 2.14 SBF evidence still applies only to its earlier artifact.

Writer files: `programs/piv1/src/genesis_allocation.rs`,
`programs/piv1/src/genesis_initialization.rs`,
`programs/piv1/tests/genesis_initialization.rs`, and this report. No helper file,
schema, dependency, instruction selector or approved message format changed.
Root updated `AGENTS.md`, both repository/program READMEs, master specification,
execution plan, test plan and pilot checkpoint: eleven changed files in total.
No dependency, schema or economic decision changed. Root owns final document
review, narrow staging, normal commit and publication to `integration/piv1-testnet`
under D-026. Main stays at `7b74be4b13c019b96a0c8abcbebfbcc361d31089`; D-028
does not cover Task 2.24. Git records the exact commit and publication identity;
verify local/remote refs and clean worktree after publication. Task 2.25 is NOT STARTED;
STOP after reviewed publication. No Mainnet action, deployment, fund movement,
key creation/signing or authority transfer occurred. AI-assisted review is not
a professional independent audit.
