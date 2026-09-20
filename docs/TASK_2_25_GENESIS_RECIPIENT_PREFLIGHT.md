# Task 2.25 — Fresh genesis recipient-vault identity preflight

Status: **TECHNICALLY VALIDATED / PENDING FOUNDER ACCEPTANCE**.
Base: `ceeac8618a66f662e5d6a0369ada76b080ebee84`, `integration/piv1-testnet`.
The founder resumed one economical bounded session under D-026, followed by a
saved checkpoint and STOP. Main remains `7b74be4b13c019b96a0c8abcbebfbcc361d31089`;
D-028's earlier integration authority does not extend to this task.

## Read-only contract and scope

`preflight_approved_genesis_recipients` composes the existing complete fresh
genesis preflight with two temporary beneficiary recipient AccountInfos. It
accepts the executing program, exact approved 313-byte genesis model, full
approved account slice and trusted roles. Two u8 vault-index witnesses must
derive the already-approved recipient keys; they add no instruction bytes,
configured index policy or independent authority. No detached preflight or
recipient observation can enter the function.

Recipient indices are bounded and distinct from existing roles and one another;
their keys and backing stores cannot alias other accounts. The unchanged
genesis preflight freshly authenticates the Squads action, current governance,
Jito identity and all sixteen unallocated targets. One runtime height/Clock/Rent
observation is retained through the composition. Each recipient must:

- equal its exact approved HTFP or Team Owner Config key;
- be the canonical PDA under the existing pinned Squads v4 Program ID using
  `[b"multisig", authenticated_multisig, b"vault", [vault_index]]`;
- remain distinct from the governance vault, PIV authority/targets, guardians,
  protocol roles and every other supplied account;
- be nonexecutable, System-owned and empty-data;
- have positive lamports and cover the same checked current empty-account rent
  floor. Zero-rent context does not make a zero-lamport account funded.

No recipient signature or writable privilege is required. Matching host flags
do not establish control or actual signatures. Private-field, non-Clone,
nonserialized observations expose each address, multisig, index/bump, exact
native balance and rent floor alongside the fresh genesis preflight. No recipient
balance sum, shortfall offset, funding, custody classification or mutation occurs.
Proposed Config, distribution and reward state remain the existing zero-history
model; actual target accounts stay unallocated. Both initial pause flags are
supported only through the existing fresh-bootstrap rules.

This profile implements a bounded identity prerequisite for G-006, K-002 and
master specification section 12.4. **Present Squads vault identity does not prove
exclusive four-of-six spending, absence of delegated spending-limit accounts,
absence of executable stale transactions, or live deployed-artifact identity.**
Those controls and real recipient identities require separate verification.
The observation is neither full recipient-control proof nor initializer approval.
The pinned vault/spending-limit and historical-approval source boundaries remain
those documented in [Task 2.15](TASK_2_15_SQUADS_AUTHORITY_SNAPSHOT.md).

## Writer validation and correction record

The focused command used direct pinned Cargo/Rust 1.97.1, an explicit clean
offline environment, one job and the existing host cache:

```text
/home/jerem/.rustup/toolchains/1.97.1-x86_64-unknown-linux-gnu/bin/cargo test --locked --offline --jobs 1 -p piv1 --test genesis_recipients
```

The first run passed 7/8 tests and reported one `unused_must_use` warning. The
failing test expected `AccountAlias` for a recipient equal to the PIV authority;
inspection of existing Config validation confirmed it first returns
`InvalidAddress` for that duplicate declaration. Only the exact test expectation
and success-result assertion were corrected; production source was unchanged.
The warning was resolved by asserting successful observation in the privilege
matrix. Initial failed logs remain at
`/tmp/piv1-t225-writer-20260920-oc626x43` (3.698717525 seconds, exit 101).

The single corrective retry passed **8/8 tests**, with no failures, ignored or
filtered tests, or diagnostics. Command time: **0.983108448 seconds**; suite:
**0.49 seconds**. Evidence: `/tmp/piv1-t225-writer-retry-20260920-tujlllux`.
Both runs record exact commands/environments, pinned binary hashes/versions,
92 unchanged input hashes before/after each run, complete logs and results.

| Run/log | SHA-256 |
| --- | --- |
| First stdout | `0b21e2f9a769734aafb437ff4f48461e8f4d075c43edce50ef1a2b3f92f79b14` |
| First stderr | `46dbf3bde6c479356c83871e1766a9e2bf1c374755a4bf3abf6a80b22f8b9eb1` |
| Retry stdout | `3a29806ad4b028ce2993729e537d6b46538fb053e77c9f778a31f6a262e688a9` |
| Retry stderr | `6c898fbe3e244653f644f17148a5fbd5426f3bd577f9d094154c5e277969b6d9` |

Eight grouped tests cover 72 success worlds across two runtime program IDs,
shared/distinct protocol receivers, both initial pause flags, three index pairs
including 0/255, and exact/excess/maximum balances. Independent literal PDA seeds,
rent arithmetic and model offsets check exact observations without mutation.
Negative cases exercise approved-key/PDA/index/multisig failures, owner/data/
executable/rent constraints including zero-rent funding, role/key/backing aliases,
mutable borrow conflicts, stale approval and old observations, full-account
message mismatches, unchanged protocol/target checks and ordinary host/native
guards. Shared read borrows succeed; complete fixtures remain unchanged.

## Root validation and separate review

Root inspected the implementation and tests, verified the existing Config
validation order underlying the test correction, and independently executed
**462 host tests +1 doctest/eight gates PASS** on the corrected source. This was
the first root execution, with zero failures, ignored tests or diagnostics;
it does not erase the writer's earlier failed run. Gate-command wall time:
**95.886570692 seconds**. Direct pinned Rust 1.97.1 tools and a clean explicit
offline environment were used. Each command below included
`--locked --offline --jobs 1`:

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

Evidence: `/tmp/piv1-t225-pilot-host-20260920-a`, including commands/environment,
before/after manifests, logs, tool hashes, results and pilot summary. All 92
inputs match inspection and the corrected writer run; root verified sixteen
gate logs, all four writer logs and three actual pinned tool hashes. The only
change to previous Rust inputs is the module export; two Rust files are new.
The `results.json` SHA-256 is
`e243ad16011f728f986365614cb9ccad434ba57695946fdea9d6bc5b54b5feba`.

Separate scope, final source/test and execution-evidence reviews passed without
an actionable blocker. The reviewer independently checked the same input/log/
tool evidence and ran no tests/builds. Final documentation review precedes normal
integration publication. Reviewed source/test SHA-256:

| File | SHA-256 |
| --- | --- |
| `programs/piv1/src/genesis_recipients.rs` | `5c41ea5d9096df95ab0df93e286a935c044432e4dbfa2ad8c7d5cbf649274af5` |
| `programs/piv1/src/lib.rs` | `341c4f362443a3e984ffa24f9ba546186721a3e0d6bf2f62696e23178eb3a524` |
| `programs/piv1/tests/genesis_recipients.rs` | `1d141ceacd2919a9335574b3bb3c3f35149144403eb05012bafb62ac834e2075` |

Six old transport-template inputs remain unchanged: nine Node tests/eight packet
cases are retained Task 2.23 evidence, not rerun and not proof of the expanded
recipient composition. No SBF/runtime build or tests were refreshed; historical
Task 2.14 evidence applies only to its earlier artifact.

## Preserved limits and handoff

This read-only fixture has **32/31 AccountInfos** (distinct/shared protocol
receiver), comprising the original preflight's 30/29 plus two recipients. The
historical Task 2.23 complete-initializer transport template has 33/32 accounts
with separate payer/System/Token roles. A future composition needs its own
topology and packet/runtime proof; no new transport measurement is claimed.

All existing public APIs, schema, dependencies, genesis bytes and native dispatch
remain unchanged. Funding provenance, operational baseline, later Token-native
donations, full recipient control, live Squads lifecycle and current-source
SBF/resource/rollback proof remain separate prerequisites. No CPI, account mutation, SBF
build, RPC, signing, keys, deployment, Mainnet action, fund movement or authority
transfer occurred. AI-assisted review is not a professional independent audit.

Writer files: new `programs/piv1/src/genesis_recipients.rs`, new
`programs/piv1/tests/genesis_recipients.rs`, one module export in
`programs/piv1/src/lib.rs`, and this report. Root updated `AGENTS.md`, both READMEs,
master specification, execution plan, test plan and pilot checkpoint: eleven
files in total. Root owns final documentation review, narrow staging, normal
commit and integration-only publication under D-026. Git records the exact task
commit/publication identity; verify remote refs and clean worktree afterward.
Main remains at the baseline recorded above. Task 2.26 is NOT STARTED; save and
STOP after reviewed publication.
