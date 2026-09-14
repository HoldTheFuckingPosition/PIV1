# Task 2.17 — Squads bootstrap invocation authorization

Status: **TECHNICALLY VALIDATED / PENDING FOUNDER ACCEPTANCE**.
Implementation: `69f7289e4c3ea2821141c4bcaded5ae942eed979`.
Base: `cadff2b0fbdbaee278beac4d44f2331b552e6acb`, `integration/piv1-testnet`.
This D-026 source/host prerequisite does not initialize PIV1 or constitute
founder acceptance, deployed-artifact verification or public-Testnet evidence.

## Boundary

Task 2.16 requires initialized Config, registry and rewards. The separate
bootstrap library boundary removes that trust cycle without admitting vacant
state to the existing-governance API. Eight trusted handler roles select the
actual Program, ProgramData, multisig, proposal, transaction, vault, Instructions
sysvar and Config accounts from the actual ordered entrypoint slice.

Config must be its canonical PDA under the trusted runtime PIV1 ID, System-owned,
nonexecutable, writable and empty through a fallible data borrow. Zero and
prefunded lamports are accepted; lamports are not read, classified or reassigned.
Program-owned or allocated Config accounts reject, including all-zero allocation.
No initialized PIV1 state is read. There is no registry, reward or Clock-account
requirement in this bootstrap boundary.

The runtime wrapper obtains its own stack height, Clock and Rent. Ordinary host
calls reject before context/account reads; an explicitly named host-only seam
supplies modeled context. Both checking bodies remain host-typechecked. Private
shared functions preserve Task 2.16's fresh authority and initialized guardian
authentication ordering and reuse its unchanged bounded proposal, transaction
and Instructions-sysvar parsers.

Both boundaries require direct top-level Squads execution at stack two; current
loader-v3/Squads authority; nonstale Approved proposal with at least four distinct
current eligible votes; checked Clock timelock; canonical account identities and
links; the actual vault signer; exact single-inner PIV1 program/accounts/data/
privileges; authenticated outer executor and static-account sequence; and
read-only inner Squads metadata. Stored-message lookups, ephemeral signers,
batches and multiple inner actions remain unsupported. Surplus outer global
privileges remain allowed. No permissive alternate decoder was introduced.

The non-Clone bootstrap output exposes invocation identities, Config address,
transaction/top-level indices, the six sorted current Squads members and an
approval bitmap in that order. It has no PIV1 slot assignment, registry revision,
activity or parameter-semantics claim. Reauthenticate after mutation/CPI.
Repeated checks while Config remains virgin may succeed: no persistent replay
receipt, account creation, initialization transition or effect-once mechanism
exists here. Task 2.16 still authenticates existing PIV1 slots independently.

## Source and transport prerequisite

The source basis remains Squads revision
`64af7330413d5c85cbbccfd8c27a05d45b6e666f`:
[execution and status ordering](https://github.com/Squads-Protocol/v4/blob/64af7330413d5c85cbbccfd8c27a05d45b6e666f/programs/squads_multisig_program/src/instructions/vault_transaction_execute.rs),
[proposal schema/votes](https://github.com/Squads-Protocol/v4/blob/64af7330413d5c85cbbccfd8c27a05d45b6e666f/programs/squads_multisig_program/src/state/proposal.rs)
and the unchanged references in [Task 2.16](TASK_2_16_SQUADS_INVOCATION_AUTHORIZATION.md).
During CPI, `.take()` changes Anchor's in-memory transaction value; original
bytes remain and Proposal Executed persists after successful outer completion.
Synthetic host inputs do not prove actual signatures, Squads CPI or rollback.

The preceding read-only assessment identified a 26-account custody/authentication
lower bound, excluding additional protocol/funding accounts. For `A` inner/static
accounts and `D` inner data bytes, the illustrative one-signature legacy scenario
has compact Squads message `10 + 33A + D`, direct creation `296 + 33A + D` and
execution `181 + 33A` bytes for the illustrated 26–32 account counts and bounded
data lengths. The creator also pays creation rent/fees; the executor pays execution
fees and is outside the inner keys. No memo, extra instruction or lookup is
included. Additional signatures or different funding arrangements change the budget. At `A=27,D=96`, creation is 1,283 bytes and execution
1,072 bytes versus the documented [1,232-byte transaction limit](https://solana.com/docs/core/transactions).
No initializer ABI exists; this is not a final transport budget. Calculations are
in `/tmp/piv1-init-transport-20260914-fafe3rgv/{calculate.py,results.json}`.
Pinned [buffer creation](https://github.com/Squads-Protocol/v4/blob/64af7330413d5c85cbbccfd8c27a05d45b6e666f/programs/squads_multisig_program/src/instructions/transaction_buffer_create.rs)
and [create-from-buffer](https://github.com/Squads-Protocol/v4/blob/64af7330413d5c85cbbccfd8c27a05d45b6e666f/programs/squads_multisig_program/src/instructions/vault_transaction_create_from_buffer.rs)
can transport a stored message without requiring fragmented PIV1 initialization.
Those two files were inspected through web tool output, not retained as local
source files. Outer Solana v0 lookups are distinct from stored Squads-message
lookups; their transport remains unverified. No transport implementation occurs.

## Writer validation and handoff

Changed files: `programs/piv1/src/squads_execution.rs`, new
`programs/piv1/tests/squads_bootstrap.rs`, and this report. Existing fixtures,
Task 2.16 tests, exports, handlers, instruction/error ABI, serialized layouts,
state constructors, manifests, lockfile, dependencies and source pins are unchanged.

After source inspection, one direct verified Rust/Cargo 1.97.1 run used the
existing target cache, clean environment and one offline/locked job:

```text
/home/jerem/.rustup/toolchains/1.97.1-x86_64-unknown-linux-gnu/bin/cargo test --locked --offline --jobs 1 -p piv1 --test squads_execution --test squads_bootstrap
```

**29 tests PASS**: nine new bootstrap tests and all twenty Task 2.16 tests;
zero failures, ignored tests or diagnostics. Total elapsed time was 5.382 seconds.
Coverage includes virgin/prefunded inputs, full input immutability, old/new
boundary separation, Config identity/ownership/allocation/privileges/borrows,
every role's bounds/alias checks, sorted bitmap meaning, repeated checks, fresh
authority/proposal/context rejection and exact message/outer binding.
No corrective retry or failed preparation occurred.

Evidence: `/tmp/piv1-t217-writer-20260914-071gpuuf` contains `command.json`,
`result.json`, logs and before/after source manifests. Source preservation passed.
Stdout SHA-256: `4efb5ff03d0ce53393f46dd72dc051a11a22813467d31ac65576b3bbd51d1e44`.
Stderr SHA-256: `239e26575059917c948f912a6c1e3a849e907e30abef4ba51d0e9c453fc3bfd9`.
Root independently checked those two focused logs and the inspected source hashes.
The writer performed no Git operations or commits and did not alter main.

Root executed the final eight host gates on the same frozen source using the
verified direct Cargo/Rust/Rustdoc 1.97.1 tools, clean environment, existing target
cache and `--locked --offline --jobs 1` on every command:

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

**392 host tests +1 doctest / eight gates PASS**, zero failures, ignored tests
or diagnostics; 65.657 seconds across those commands. Root verified all sixteen
log hashes, unchanged full source/manifests/lockfile, correspondence to the
separately reviewed and root-inspected freeze, and unchanged verified tool hashes.
Evidence: `/tmp/piv1-t217-pilot-host-20260914-a`, including exact commands,
environment/tools, logs, before/after source manifests and `pilot-summary.json`.
`results.json` SHA-256:
`03bbed207395030c8eb2f6b3e88fc8f105e3b85dee31e360a90d712f4bf78d0f`.
Writer source manifest SHA-256:
`e9e31e8dc96abc460e6475dbbf96a11f429d3af70c78981c7d1cf6afe52fb4d3`.

Separate reviewer `review_squads` passed the final two-file source/test change,
checked source hashes and focused logs, and reported no actionable findings.
Root inspected the complete source delta and all nine new tests. The existing
bounded parsers, twenty invocation tests and shared fixture are unchanged.
Final separate shared-documentation/evidence review also passed with no findings.
Root committed
the implementation above and the reviewed documentation closure at
`09a02cbe485ab8abd7cb55f155539002df9251a8`. Normal atomic fast-forward publication
completed; independent remote reads matched integration, accepted main and Task2.3
refs, and worktree was clean. This following checkpoint records that receipt.
Source pins and Task 2.14 SBF evidence remain historical;
no new target evidence, rustfmt or Clippy execution is claimed.

Remaining dependencies include exact initializer parameters/zero-state construction,
official protocol account authentication, atomic prefunding-safe account creation,
funding provenance, vote-activity timing, exact transport budget and target/runtime
validation. Authorization here does not validate parameter semantics, other target
accounts, actual replay/rollback or complete Testnet readiness.

No Mainnet action, deployment, SBF build/run, package installation, chain/RPC
access, fund movement, signing, key creation or authority transfer occurred.

Accepted main remains unchanged. This AI-assisted review is not a professional
independent audit. Exact publication receipts belong in the pilot checkpoint.
