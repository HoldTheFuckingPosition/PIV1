# Task 2.15 — Squads authority snapshot

Status: **TECHNICALLY VALIDATED / PENDING FOUNDER ACCEPTANCE**.
Implementation: `da0241fd2a9f9c3247bbdeabb1a6b9c37dabc912`.
This is source/host work under D-026, not founder acceptance or Testnet readiness.
Base: `d967a7969cbbd87dce638e4e215e70c772213237`, `integration/piv1-testnet`.

## Scope and identity

The read-only library authenticates the current runtime PIV1 Program/ProgramData
relationship and observes that its upgrade authority equals one explicitly
indexed Squads vault PDA. It authenticates the current autonomous six-voter,
threshold-four Squads configuration. No instruction is added or invoked.

Sources are immutable, read as inert public text, and not new dependencies:

- Squads v4 revision `64af7330413d5c85cbbccfd8c27a05d45b6e666f`:
  [official non-testing program ID](https://github.com/Squads-Protocol/v4/blob/64af7330413d5c85cbbccfd8c27a05d45b6e666f/programs/squads_multisig_program/src/lib.rs),
  [Multisig schema/invariants](https://github.com/Squads-Protocol/v4/blob/64af7330413d5c85cbbccfd8c27a05d45b6e666f/programs/squads_multisig_program/src/state/multisig.rs),
  [discriminator](https://github.com/Squads-Protocol/v4/blob/64af7330413d5c85cbbccfd8c27a05d45b6e666f/sdk/multisig/src/generated/accounts/Multisig.ts),
  [PDA derivations](https://github.com/Squads-Protocol/v4/blob/64af7330413d5c85cbbccfd8c27a05d45b6e666f/sdk/multisig/src/pda.ts).
- Existing locked `solana-loader-v3-interface = 3.0.0`, reexported by Anchor,
  [state and sizes at its package revision](https://github.com/anza-xyz/solana-sdk/blob/1c1d667f161666f12f5a43ebef8eda9470a8c6ee/loader-v3-interface/src/state.rs).

Squads source ID is fixed as `SQDS4ep65T869zMMBKyuUq6aD6EgTu8psMjkvj52pCf`.
Neither account ownership nor this source pin verifies a deployed Squads artifact.
No Squads executable account or binary is inspected. Public-Testnet availability,
actual public accounts, genesis, and deployed source equivalence remain OPEN.
An unsupported external program identity is rejected, not caller-selectable.

## Enforced boundaries

- Trusted runtime PIV1 ID equals the executable loader-v3-owned Program account.
  Its data is exactly 36 bytes with little-endian enum tag 2 and the canonical
  ProgramData PDA. The supplied nonexecutable ProgramData has the same loader
  owner, at least 45 bytes, tag 3, and a canonical `Some` upgrade authority.
  Bytes beyond metadata are opaque, not evidence of executable validity.
- The nonexecutable Multisig account has the fixed Squads owner and canonical
  `["multisig", "multisig", create_key]` PDA/bump. Its discriminator matches.
  `["multisig", multisig_address, "vault", u8_index]` derives the vault, which must
  equal ProgramData's current authority. No vault account or signature is needed.
- The bounded reader decodes exactly six members without an input-sized
  allocation. Members are distinct, nonzero and strictly sorted; all have Vote,
  at least one has Initiate and one Execute, with no unknown permission bits.
  Threshold is four and config authority zero. Index order and the upstream
  7,776,000-second maximum timelock are checked. Nonzero supported timelocks and
  mixed permission combinations remain valid.
- The rent-collector option tag is at offset 94. None puts bump/count/members at
  95/96/100 and ends the six-member payload at 298; Some uses 127/128/132 and ends
  at 330. Minimum allocation is 330 for either. Larger allocations and nonzero
  stale tails are accepted without copying; PIV1's zero-tail rule does not apply.
- Account-role aliases and conflicting data borrows reject without mutation.
  Signer/writable privilege unions and account lamports do not alter this read.
- Separate correspondence accepts only an `AuthenticatedGuardianClockSnapshot`
  with the same trusted runtime PIV1 ID and the same six keys as a set. A private
  runtime-ID field/getter adds provenance to that existing owned snapshot, with
  no serialized change. PIV1 slot/reward ordering is preserved.

Both owned observations are point-in-time evidence. Refresh them after mutation
or CPI; correspondence does not synchronize or rotate either registry.

## Authorization limits and next dependency

Current configuration is not proposal approval or historical configuration proof.
Squads [vault execution](https://github.com/Squads-Protocol/v4/blob/64af7330413d5c85cbbccfd8c27a05d45b6e666f/programs/squads_multisig_program/src/instructions/vault_transaction_execute.rs)
and [batch execution](https://github.com/Squads-Protocol/v4/blob/64af7330413d5c85cbbccfd8c27a05d45b6e666f/programs/squads_multisig_program/src/instructions/batch_execute_transaction.rs)
allow previously approved stale proposals. An action approved under threshold one
may survive restoration to four. Tests explicitly accept the current restored
snapshot without pretending to observe, execute, or validate that proposal.

Separate spending-limit accounts are not enumerable from Multisig. The pinned
[spending-limit path](https://github.com/Squads-Protocol/v4/blob/64af7330413d5c85cbbccfd8c27a05d45b6e666f/programs/squads_multisig_program/src/instructions/spending_limit_use.rs)
only invokes System transfer or Token/Token-2022 transfer_checked; the pinned
[Anchor 0.29.0 helper](https://github.com/coral-xyz/anchor/blob/fc9fd6d24b9be84abb2f40e47ed3faf7b11864ae/spl/src/token_2022.rs)
forwards four fixed accounts without hook/remaining accounts. No arbitrary PIV1
CPI route is evidenced through that path, and absence of spending limits is not
claimed. Normal approved vault execution can forward signer privilege through CPI.

Next authorization work must bind exact proposal approvals, transaction/message,
active execution and replay state. Rotation must distinguish old approving keys
from the new synchronized set. Loader genesis/initialization, approvals, signing,
ProgramData history, guardian key control, Squads execution and runtime deployment
remain outside this task. No economic or governance policy was changed.

## Evidence and changed files

Writer files: `programs/piv1/src/squads_accounts.rs`, module export in `lib.rs`,
four provenance lines in `guardian_clock_accounts.rs`,
`programs/piv1/tests/squads_authentication.rs`, and this report. Existing error ABI,
serialized layouts, entrypoint, manifests, lockfile and dependency pins are unchanged.

Fourteen new test functions cover loader identities and metadata, every short
allocation, malformed options/counts, PDA/bump/authority substitutions, permissions,
timelocks/indexes, aliases, data-borrow conflicts, complete input immutability,
oversized/nonzero tails, privilege unions, the historical-approval limitation,
and authenticated same-program guardian correspondence with preserved slots.

Writer's single focused command used direct verified Cargo/Rust 1.97.1, a clean
environment, locked/offline mode and one job:

```text
/home/jerem/.rustup/toolchains/1.97.1-x86_64-unknown-linux-gnu/bin/cargo test --locked --offline --jobs 1 -p piv1 --test squads_authentication --test guardian_clock_authentication
```

Preparation failed before tests: the independent fixture's AnchorSerialize derive
needed the existing `anchor_lang::prelude::borsh` import. Six E0433 diagnostics;
exit 101, 2.228 seconds, zero tests executed. The import was corrected without a
dependency change. Production compiled; no test PASS is attributed to this failed run. Raw command,
environment, logs and unchanged-during-run source hashes are preserved under
`/tmp/piv1-t215-writer-20260914-iqwfub2_`. The corrected four-file source freeze is
`corrected-source-freeze.json` in that directory. Root validation and separate source/test review passed on that corrected freeze.
`git diff --check` passed after correction. No SBF build/runtime run is claimed.

Root executed the eight final host gates on the corrected frozen source:
**363 tests +1 doctest PASS**, zero failed/ignored tests or diagnostics, 63.045
seconds total. Gates were workspace all-target tests, doctests, all-target check,
all-feature check, explicit no-entrypoint/cpi/idl-build checks, and warnings-denied
documentation. All used `--locked --offline --jobs 1` with direct Cargo/Rust 1.97.1
and the established clean environment. Exact commands, environment, 16 verified
log hashes and matching before/after source inventories are saved at
`/tmp/piv1-t215-pilot-host-20260914-a`; `pilot-summary.json` records root's checks.
`results.json` SHA-256:
`ef942593ca4f2dbab1efa9e8e82f64db7b7e5762664b5777bfc52904d4f7eda7`.
Root verified the three direct toolchain hashes are unchanged from Task 2.14.

Reproduction entry point for this execution was:

```text
env -i PATH=/usr/bin:/bin LC_ALL=C /usr/bin/python3 -I /tmp/piv1-t215-pilot-host-validation.py
```

Separate reviewer `review_squads` inspected the actual four-file source/test
freeze and returned bounded PASS with no actionable findings. Root independently
inspected those source/tests and actual test logs; no final production correction
was needed. Root also verified 41 inert upstream source files against the recorded
research manifest. Rustfmt/Clippy remain unavailable and were not run or installed.
This task has no new SBF build or execution proof. Existing Task 2.14 SBF artifacts,
source pins and results remain historical; refresh applicable source pins and
build/test a new artifact before claiming this code has runtime evidence.

Root owns shared coordination documents and Git. No writer commit was created.
Founder-accepted main remains `66193769d1cbc59cd8630df295b9a784b9c64642`. No Mainnet action, deployment, fund movement, key creation, signing or
authority transfer occurred. This AI-assisted work is not an independent audit.

Final documentation/evidence review passed; two stale shared-checkpoint status
lines were corrected during closure. Root owns normal fast-forward publication;
verify actual local/remote HEAD on takeover. No technical finding remains in this
bounded source/host task. Founder acceptance and live-operation gates remain.
