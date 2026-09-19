# Task 2.21 — Prefunding-safe genesis account allocation

Status: **TECHNICALLY VALIDATED / PENDING FOUNDER ACCEPTANCE**.
Base: `748faf81e5bd8f22c05b7588d7d4bb6d14d42848`, `integration/piv1-testnet`.
This bounded D-026 task advances actual System CPI composition after the read-only
[Task 2.20 preflight](TASK_2_20_GENESIS_ACCOUNT_PREFLIGHT.md). It does not add a
native initializer, change accepted economics or extend founder acceptance.

## Allocation-only contract

`allocate_approved_genesis_accounts` accepts the executing Program ID, the exact
approved genesis bytes, ordered account slice and trusted handler role mapping.
It freshly invokes the complete Task 2.20 dispatcher inside execution, preserving
one acquisition of stack height, Clock and Rent. No detached preflight/model can
authorize a new allocation. The existing preflight dispatcher becomes
crate-visible solely for this composition; its public behavior is unchanged.
Existing Squads approval, Jito identities, sixteen target checks and source
limitations remain in force.

Two additional accounts belong to that same exact approved message: a distinct
external rent payer and the canonical executable System Program. The payer must
be writable, signing, nonexecutable, empty-data and System-owned. Its role cannot
reuse bootstrap, protocol, target or System roles. Its key cannot be a proposed
recipient, current guardian or virtual PIV authority; actual full-slice key
uniqueness and backing-store separation remain mandatory. The existing Squads
validation also requires the non-vault payer signer in the outer instruction.
Host signer flags only model that evidence; no actual signature is proven here.

The payer funds **only each target's checked rent shortfall**, derived from the
fresh Rent and intended allocation. Its checked final balance must retain its
own empty-System-account rent floor. No fees, reimbursement, operational reserve
liquidity, principal or beneficiary payment are charged or inferred. Authorization
establishes this specific source and exact modeled transfers, not a claim about
the ultimate historical origin of the payer's funds. Existing raw prefunds remain
at their original targets; they are never summed, netted across targets or
classified into an economic ledger. The proposed economic model stays unchanged.

All sixteen PDA seed sets and bounded System instructions are prepared before
the first effect. Every target and payer mutable data/lamport borrow must be
available simultaneously, then all guards are released before CPI. For each
target in canonical model order:

1. Transfer its missing rent from the external payer, only when nonzero, with no
   program signer seeds; the actual payer signature authorizes the debit.
2. For the nine PIV1 state and two legacy Token data accounts, invoke System
   `Allocate` followed by `Assign` with that target's exact canonical PDA seeds.
3. Leave the five native vaults empty and System-owned.

This uses existing pinned `solana-system-interface` 1.0.0 constructors through
the accepted Anchor/Solana reexport: transfer opcode 2, allocate opcode 8, assign
opcode 1. Each instruction supplies only its required writable/signing accounts
plus the canonical executable System Program. No dependency is added or changed.
After every CPI, verify the exact payer balance and all sixteen target balances,
owners, sizes, writable/nonexecutable status and zero bytes against the expected
stage. Previous target changes, false success and unexpected CPI effects reject.
Invocation failures are preserved and must propagate to the transaction boundary.
Production code does not attempt to undo partially executed CPI effects.

The private-field, non-Clone, `must_use` result contains the approved model,
historical pre-allocation observations and exact payer before/after/funding facts.
It is **only an intermediate**: Token accounts are not initialized, nine state
accounts contain no serialized state, and zero data is not an initialization
receipt. A future caller must initialize both Token accounts and serialize and
validate all nine states **in the same atomic transaction before success**.
Committing bare Token-owned zero data permits initialization takeover. Never
expose this function as a stand-alone native handler or persist an allocation
checkpoint between transactions. The existing native dispatcher still has only
claim and pending-recognition selectors; genesis bytes remain rejected.

## Focused validation

The host-only seam accepts explicit modeled invocation context and an effectful
`FnMut` invoker; it is absent from Solana/native dispatch. Ordinary production
calls on host reject before accessing accounts or decoding data. Tests extend the
accepted Task 2.20 synthetic fixture/oracle construction locally without changing
its existing tests. The invoker independently assembles pinned System payloads,
metas, actual account lists and every fixed/guardian-specific seed group, including
the canonical bump under two runtime IDs. Allocation replaces test data slices
with owned host buffers; it does not use runtime-only SDK resize memory assumptions.

Twelve focused tests cover zero, partial, exact, excess and `u64::MAX` prefunding;
both runtime IDs and optional manager/referrer sharing; exact payer retention and
one-lamport shortages; payer metadata, protected roles/keys, inner/outer signature,
backing aliases and every target/payer mutable-borrow conflict; fresh approval,
authority, protocol, target, Clock and stack checks; malformed Rent and checked
aggregate overflow; replay rejection and deliberately uninitialized Token data.
Every one of 38 CPI boundaries is exercised for failure before/after modeled
effects and for false success. Wrong amounts, size, zero contents, owner, prior
target mutation and unexpected payer credit also reject.

Tests explicitly distinguish direct execution, which may retain partial effects
on error, from a separately labeled cloned-world transaction model that discards
the staged world on failure. Neither is evidence of Solana Bank/AccountsDB
rollback, real signatures, loaded System CPI execution or runtime resource limits.
The delegated writer executed the following focused command using verified direct
Rust/Cargo 1.97.1 tools, a clean explicit environment, existing package/target cache
and one offline job:

```text
/home/jerem/.rustup/toolchains/1.97.1-x86_64-unknown-linux-gnu/bin/cargo test --locked --offline --jobs 1 -p piv1 --test genesis_allocation
```

**12 tests PASS**, zero failures, ignored/filtered tests or compiler diagnostics.
Elapsed: **70.566 seconds**, including offline recompilation of cached dependency
sources; test execution itself took 1.23 seconds. No compilation/test failure or
corrective retry occurred. An unused import was removed during source preparation,
before the run. No dependency, lockfile or tool version changed.

Evidence directory: `/tmp/piv1-t221-writer-20260919-_5e4tkwr`. It records the exact
command/environment, verified cargo/rustc/rustdoc hashes, before/after manifests
of all **88 source inputs**, result and complete logs. Inputs remained identical.
Stdout SHA-256:
`f1fda1cb3dcdddc6050c7e110a22382b1e64da9d04f7c3f51e41d899a829ed88`;
stderr SHA-256:
`cb748c560b50c0bb197639982467540cb1a1e4d5ea309860032ef09ff035e597`.

Root independently inspected the actual source/tests and verified every source
input and both log hashes against the focused execution. Separate source/test
review passed with no actionable findings. Root subsequently completed the final
workspace gates and evidence verification recorded below; source/tests remained
frozen. Root owns publication. The prior
Task 2.20 result remains 426 host tests +1 doctest/eight gates on its earlier
freeze; no workspace result is inferred from the new focused run. Historical
Task 2.14 SBF evidence still applies only to its recorded earlier artifact.

## Pilot validation and publication checkpoint

Root executed **438 host tests +1 doctest / eight gates PASS**, with zero
failures, ignored tests or diagnostics, after separate source/test review.
All commands used direct verified Rust/Cargo 1.97.1, the existing workspace
cache, an explicit clean environment and `--locked --offline --jobs 1`:

```text
cargo test --workspace --all-targets --quiet
cargo test --workspace --doc
cargo check --workspace --all-targets
cargo check --workspace --all-targets --all-features
cargo check -p piv1 --all-targets --features no-entrypoint
cargo check -p piv1 --all-targets --features cpi
cargo check -p piv1 --all-targets --features idl-build
RUSTDOCFLAGS='-D warnings' cargo doc --workspace --no-deps
```

Evidence: `/tmp/piv1-t221-pilot-host-20260919-a/pilot-summary.json`, adjacent
exact commands/environment, logs, tool hashes and before/after source manifests.
The gates took 100.471 seconds; no failed run or corrective retry occurred.
Root verified all 88 inputs against the inspected source and focused freeze,
all sixteen gate logs, both focused logs and the three unchanged host tools.
Results SHA-256:
`c2e780b87b84f32e1ede2e1fa91943d5a1448791c4ea583160b0b3bdef3d02c8`.
Inspected-source manifest SHA-256:
`b1a9cc127e56ed6842b7cd224713eea8c17a869b10fb41718bb7a968147a5348`.
These are new host executions, distinct from the writer's focused run and the
historical Task 2.14 SBF artifact. No new SBF build/runtime proof is claimed.

Root updates the instructions, checkpoint, execution plan, README summaries,
master status and requirements-to-evidence index in this same task commit.
Separate final documentation review and targeted publication checks precede
normal fast-forward publication on `integration/piv1-testnet` only. Git records
the exact task/publication identity; verify actual refs and worktree on takeover.
The session stops after this task; Task 2.22 is NOT STARTED. No RPC/chain operation,
Mainnet action, deployment, fund movement, key creation/signing or authority
transfer occurred. Founder acceptance remains pending.

## Preserved limits and next dependency

No schema, runtime instruction ABI, dependency, economic decision or accepted
main source changes. No Token initialization/state persistence, economic prefund
classification, recipient-control proof, operational funding baseline, transport
solution, initialized replay receipt or current-source SBF/runtime proof is added.
Current unallocated state still does not prove historical noninitialization.
This is AI-assisted engineering, not a professional independent audit.

The next bounded dependency is safe same-transaction completion: Token
initialization, all state serialization, recipient/funding constraints and fresh
post-initialization validation before any native initializer is exposed. Actual
transport and resource/rollback evidence remain prerequisites to deployment.

Writer files: new `programs/piv1/src/genesis_allocation.rs`, its `src/lib.rs`
export, minimal dispatcher visibility in `src/genesis_preflight.rs`, new
`programs/piv1/tests/genesis_allocation.rs` and this report. Root owns shared
status documents, evidence gates and Git. The writer performed no Git mutation,
network/RPC/chain operation, Mainnet action, deployment, real fund movement,
key creation/signing or authority transfer. Publication is integration-only;
accepted main remains `5a067ad34d2f2ab0a27f2c6bff312a8b1f772d24`.
