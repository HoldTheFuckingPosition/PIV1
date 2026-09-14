# Task 2.16 — Squads invocation authorization prerequisite

Status: **TECHNICALLY VALIDATED / PENDING FOUNDER ACCEPTANCE**.
Implementation: `343a496fb16edc8fd8d68746a89323d7545ab36d`.
This D-026 source/host prerequisite is not founder acceptance, a callable
governance instruction, initializer transport, or public-Testnet evidence.
Base: `11f4d701f4ca58f18a64383dce5e088377afec77`, `integration/piv1-testnet`.

## Supported profile

The new library checks a modeled/runtime PIV1 invocation against its exact
Squads proposal and stored message. Trusted handler role indices select sixteen
authentication accounts from the actual ordered entrypoint account slice; they
do not define a serialized ABI or admit detached account snapshots. The complete
inner account slice must be distinct and match the approved message.

- The ordinary wrapper fetches actual stack height, Clock and Rent. Host calls
  reject before reading that context. An explicit host-only seam injects modeled
  context and is absent from the Solana build. The common wrapper/checking bodies
  remain host-typechecked; no new SBF build or target execution is claimed.
- Stack height is exactly two. The canonical Instructions sysvar owner/key,
  executable flag, borrow, minimum length, offset table and current index are
  checked. The selected outer instruction is parsed as borrowed bounded bytes,
  avoiding the owned Instruction/Vec decoder's input-sized allocation.
- The current outer instruction is the fixed Squads program's exact eight-byte
  `VaultTransactionExecute` selector. Its first three accounts equal the supplied
  multisig, proposal and transaction. Its fourth account is a signer and current
  Execute member. Remaining accounts exactly match the stored static-key order.
  Required outer privileges hold; surplus outer global privilege unions are valid.
- Fresh Task 2.15 loader-v3/Squads authority authentication and fresh guardian/
  Clock authentication must agree under the same runtime PIV1 ID. The Clock
  account must also equal the fetched Clock. No stale owned snapshot is accepted
  in place of current accounts. The derived vault signs the actual inner call.
- Proposal and VaultTransaction are nonexecutable, fixed-Squads-owned accounts
  with correct discriminators, canonical PDAs/bumps, multisig/index links and
  vault index/bump. The index is nonzero, above the current stale index and no
  greater than the current transaction index: only current-configuration actions.
- Proposal is Approved, with at least four distinct current eligible approvals.
  All three vote vectors are bounded to six sorted current members. Approved and
  rejected sets are disjoint; three rejections or four cancellations reject.
  Cancellation is collective: fewer than four cancellations may overlap approvals.
  Checked current Clock arithmetic enforces the current timelock.
- Exactly one inner instruction targets the actual runtime PIV1 ID. Its ordered
  distinct account keys, exact inner signer/writable flags and data bytes match
  the actual entrypoint inputs. Static keys are distinct and referenced; header,
  index and data counts are bounded before reads. No batch, extra inner action,
  address lookup, ephemeral signer, or trailing transaction payload is supported.
  Governance metadata remains read-only in the inner invocation.

The message reader uses persisted ordinary Borsh vector lengths, not the compact
creation-instruction format. Proposal vote-vector allocation slack may be nonzero.
Unrelated approved-proposal substitution, prior-threshold-one approval, nested
signer delegation and earlier same-message configuration mutation do not satisfy
this profile. The single-inner restriction removes inner-position ambiguity.

The output records the current invocation identities, transaction/top-level
indices, guardian revision and approvals bitmap in existing PIV1 slot order. It
is non-Clone point-invocation evidence, not a reusable capability or persistent
effect-once/replay receipt. No state or balances are mutated by this library.

## Review correction and source basis

T216-R1 was resolved before the focused run: an initial mandatory inner executor
role would have bound an approved action to one executor key. The final check
uses the authenticated outer fourth account, preserving Squads' ability for any
current Execute member to execute identical approved inner bytes. Regression
coverage accepts three such executors and rejects missing signer, nonmember and
non-Execute cases. No economics or governance rights changed.

All Squads references use immutable revision
`64af7330413d5c85cbbccfd8c27a05d45b6e666f`:

- [Execution accounts, status ordering and vault signing](https://github.com/Squads-Protocol/v4/blob/64af7330413d5c85cbbccfd8c27a05d45b6e666f/programs/squads_multisig_program/src/instructions/vault_transaction_execute.rs).
- [Proposal schema, approval and cancellation](https://github.com/Squads-Protocol/v4/blob/64af7330413d5c85cbbccfd8c27a05d45b6e666f/programs/squads_multisig_program/src/state/proposal.rs).
- [Stored transaction/message Borsh format](https://github.com/Squads-Protocol/v4/blob/64af7330413d5c85cbbccfd8c27a05d45b6e666f/programs/squads_multisig_program/src/state/vault_transaction.rs).
- [CPI account/meta construction and protected proposal](https://github.com/Squads-Protocol/v4/blob/64af7330413d5c85cbbccfd8c27a05d45b6e666f/programs/squads_multisig_program/src/utils/executable_transaction_message.rs).
- [Exact outer instruction selector](https://github.com/Squads-Protocol/v4/blob/64af7330413d5c85cbbccfd8c27a05d45b6e666f/sdk/multisig/src/generated/instructions/vaultTransactionExecute.ts).

Squads' pinned Anchor 0.29.0 has separate owned/deserialized account values and
AccountInfo bytes ([account storage](https://github.com/coral-xyz/anchor/blob/fc9fd6d24b9be84abb2f40e47ed3faf7b11864ae/lang/src/accounts/account.rs),
[mutable-field exit serialization](https://github.com/coral-xyz/anchor/blob/fc9fd6d24b9be84abb2f40e47ed3faf7b11864ae/lang/syn/src/codegen/accounts/exit.rs)).
Thus `.take()` changes only its in-memory VaultTransaction value. During PIV1
CPI the original transaction bytes and Approved proposal remain readable;
Executed persists only after successful outer completion. This source observation
does not turn the modeled host statuses into actual Squads execution/rollback.

The existing Anchor 0.32.1 runtime reexports use already-locked
`solana-instruction = 2.3.3` and `solana-instructions-sysvar = 2.2.2`. The checked
current-index helper is called only after a held immutable borrow and length
preflight because its internal final-two-byte read lacks a size check.

## Validation and limits

Writer source/test files: `programs/piv1/src/squads_execution.rs`, `lib.rs` export,
`programs/piv1/tests/squads_execution.rs` and its dedicated
`tests/support/squads_invocation.rs`. The support builds complete synthetic
authority/guardian/proposal/message accounts with independent Anchor/Borsh
serialization and the pinned host Instructions-sysvar constructor. This report
is the fifth writer file. No existing handler, error ABI/mapping, serialized
layout, manifest, lockfile, dependency or source pin changed. Task 2.15's
authentication logic and report are unchanged.

After source inspection and T216-R1, one focused direct verified Cargo/Rust 1.97.1
run used the existing host target cache, clean environment, offline/locked mode
and one job:

```text
/home/jerem/.rustup/toolchains/1.97.1-x86_64-unknown-linux-gnu/bin/cargo test --locked --offline --jobs 1 -p piv1 --test squads_execution
```

**20 tests PASS**, zero failures, ignored tests or diagnostics; 4.855 seconds
including compilation, 2.47 seconds reported test time. Complete account/input
immutability is asserted for successes, failures and data-borrow conflicts.
Coverage includes exact message/vote/context binding, executor flexibility,
current-index selection, malformed offsets/counts, every proposal-prefix and
transaction/sysvar truncation, nonzero proposal slack, protected inner flags,
outer unions and the explicit absence of a host replay receipt.

Evidence directory: `/tmp/piv1-t216-writer-20260914-29zfses4` contains exact
`command.json`, `result.json`, `stdout.log`, `stderr.log` and before/after hashes.
Source preservation passed. Stdout SHA-256:
`d6a6af1e595bfca7f9ce9fd39ba7e33388cc7d181dab22793f3199ba9776c3cd`;
stderr: `7f08c7709a38fd3927de294f1a972b81eb85196b5cc8de4cd1a94130b1c13226`.
`git diff --check` passed. Root independently verified both focused log hashes and
the reviewed source freeze; this remains writer execution, not a second root run.

Root executed the final eight workspace gates on that same frozen source using
verified direct Cargo/Rust/Rustdoc 1.97.1, the existing target cache and an explicit
clean environment. Every command below includes `--locked --offline --jobs 1`:

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

**383 host tests +1 doctest / eight gates PASS**, zero failures, ignored tests
or diagnostics; 64.043 seconds across the eight commands. Root checked all sixteen
log hashes, unchanged full source/manifests/lockfile, correspondence to the
reviewed writer freeze, and unchanged verified tool hashes. Evidence directory:
`/tmp/piv1-t216-pilot-host-20260914-a`; exact commands/environment/tools, results,
logs, before/after source hashes and `pilot-summary.json` are retained there.
`results.json` SHA-256:
`1d8d15ebd4da5df6a5be15709cb3a3971cdd3d1c26b1e756a853d124fbf7187f`.
Writer source manifest SHA-256:
`67d929f155a986b78ef468e9c447738e2ca1d7b14d62e14de6bb8573af351c77`.

Separate reviewer `review_squads` inspected the frozen source/tests and report,
verified all four source hashes and actual focused logs, and returned bounded
PASS with no remaining actionable finding. Root separately inspected all source,
tests and fixtures. Final separate shared-documentation/evidence closure review also passed.
These are host executions; Task 2.14's unchanged SBF artifacts/pins are historical.
No new SBF build, target execution, dependency, rustfmt or Clippy run is claimed.

Initialization's account/message budget and ABI are unproven; this profile must
not force fragmented initialization. Guardian rotation needs a separate old/new
approver-set and synchronization design. Live artifact/source equivalence,
public-Testnet Squads availability, actual CPI/signatures, replay/effect-once,
rollback and total runtime resources remain unproven. Future handlers must check
their own state/pause policy and propagate failures atomically.

Root owns shared documents and Git. The four reviewed implementation files are
committed at the identity above; this report, AGENTS, execution plan, test checklist
and pilot checkpoint form the separate documentation closure. Normal integration
publication is pending that closure; verify actual Git on takeover. Accepted main
is unchanged. No Mainnet action, deployment, fund movement, key creation, signing
or authority transfer occurred. AI-assisted review is not a professional
independent audit.
