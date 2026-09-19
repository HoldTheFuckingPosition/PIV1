# Task 2.22 — Same-call genesis account initialization

Status: **TECHNICALLY VALIDATED / FOUNDER-AUTHORIZED MAIN INTEGRATION (D-028)**.

D-028 (2026-09-19) authorizes main integration of Tasks 2.20–2.23 at
`3282e1ebabcb0cd88491d48a391565b8b100afa7` plus reviewed authorization records,
after final verification. This changes publication authority, not broader founder
acceptance or live readiness. No tests were rerun in this publication turn; see
[current checkpoint](PIV1_PILOT_STATE.md). The original session narrative below,
including its integration-only publication restrictions, is HISTORICAL.

Base: `cbe5611f625b3e1be4acf38f25cca7dc5f0defb9`, `integration/piv1-testnet`.
The founder requested one additional economical task after Task 2.21 publication,
then a saved checkpoint and STOP. This D-026 task completes the allocation-only
library intermediate with both Token initializations and all nine state writes
in one call. It adds no native initializer or new deployment permission.

## Fresh same-call composition

`initialize_approved_genesis_accounts` accepts the executing Program ID, exact
313-byte approved genesis parameters, account slice and trusted handler roles.
The new role mapping contains the Task 2.21 allocation roles and one additional
canonical executable legacy Token Program account in that same approved slice.
Bounds, role separation and Token-program backing aliases are checked before any
System effect. The Token role cannot reuse any bootstrap, protocol, target, payer
or System role. The mint must have its exact existing 82-byte allocation; mutable
data-borrow availability is checked because the pinned Token initializer borrows
mint data mutably while reading it. No guard is retained across CPI.

Task 2.21's private execution dispatcher becomes crate-visible solely for internal
composition. The new call then freshly performs all existing exact Squads/Jito/
target/payer checks and the rent-only allocation batch. It cannot consume a detached
preflight or allocation receipt. Stack/Clock/Rent are still acquired once through
the original chain, with the same captured Rent used for state-write rent checks
and final fixed-account authentication. Existing external-payer signature, exact
rent-shortfall debit, payer rent floor and all sixteen target constraints remain
unchanged. Raw prefunds remain where observed, unaggregated and unclassified.

After allocation, existing `StateEnvelope` constructors prepare nine validated
fixed-size values: Config, ActiveDistribution, GuardianRegistry and six guardian
rewards in model order. There is no new serialization format, discriminator or
economic transition. All states exactly match the fresh approved model, including
idle distribution, zero economic histories and no invented guardian activity.

## Token initialization and initial state writes

Both Token CPIs precede every state write. The pinned `spl-token` 8.0.0
`initialize_account3` constructor creates an instruction to the canonical legacy
Token Program for each distinct Jito vault:

```text
data: opcode 18 followed by the 32-byte derived PivAuthority
metas: target (writable, nonsigner), official JitoSOL mint (readonly, nonsigner)
actual CPI accounts: target, mint, canonical executable Token Program
program signer seeds: none
```

The authority is encoded in the instruction, not supplied as a signer account.
Source inspection of the pinned Token processor confirms its Rent acquisition
and non-native initialization shape. The expected 165 bytes contain the official
mint, derived PivAuthority, zero amount, `Initialized` state, no native reserve,
delegate or close authority, zero delegated amount and canonical zero option
payloads. Tests independently construct this layout from fixed byte offsets.
No new dependency, source pin or network lookup is needed.

Before the first Token call, after each Token call and after state persistence,
the new stage check compares the exact payer balance and all sixteen target keys,
owners, writable/nonexecutable flags, balances, sizes and expected bytes. A target
that has not been initialized/written must remain all zero. Earlier initialized
Token state must remain exact during the second call. Mint owner, balance and
complete bytes must remain unchanged from the pre-allocation observation. A Token
initializer that falsely returns success or changes an unrelated protected target
therefore rejects before successful completion.

The private initial-state writer acquires all nine mutable data guards and checks
every model-derived address, PIV1 owner, writable/nonexecutable flag, exact size,
rent coverage and all-zero prior data before writing any envelope. The final copy
loop cannot allocate, serialize, borrow or fail. This is a narrow zero-to-approved-
genesis operation; the general existing-account `PreparedStateWrite` and
`commit_state_writes` contracts remain unchanged. Their requirement for a valid
typed before-envelope is not weakened to accept arbitrary zeros.

Final byte comparisons cover all nine envelopes and both Token vaults. Existing
fixed-account authentication freshly decodes Config/ActiveDistribution and validates
all fixed custody relationships, then compares the state to the approved model
and requires both token amounts to be zero. Exact model envelopes also preserve
registry/reward identities, bindings and zero-history values.

The returned private-field, non-Clone object records point-invocation completion
and historical allocation facts. It is not a durable replay receipt or authority
to initialize another slice. Existing initialized/allocated Config fails fresh
bootstrap authentication on retry. This still does not prove an address was never
initialized in its history.

## Focused validation evidence

The explicit host context/invoker seam is absent from Solana/native dispatch.
Ordinary host calls reject before account access. The native dispatcher remains
limited to claim and pending recognition; the genesis parameter selector still
rejects before Rent, CPI or event callbacks.

The delegated writer executed:

```text
/home/jerem/.rustup/toolchains/1.97.1-x86_64-unknown-linux-gnu/bin/cargo test --locked --offline --jobs 1 -p piv1 --test genesis_initialization
```

**10 tests PASS**, zero failures, ignored/filtered tests or diagnostics; first
execution, **6.351 seconds** including compilation and 1.34 seconds of tests.
No corrective retry occurred. Direct verified Rust/Cargo 1.97.1 tools, existing
cache and a clean explicit offline environment were used with one job.

Evidence: `/tmp/piv1-t222-writer-20260919-32mv1qub`, including exact command/env,
tool hashes, before/after manifests of **90 source inputs**, result and full logs.
Inputs remained unchanged. Stdout SHA-256:
`e39fad294b75531fb275a0aa26cc9237cb4966f6229ac646d84a8494602ab4ca`;
stderr SHA-256:
`feb7a4bd30ec24e706fff67f33c4bb43db950dd5c4eea41a12754a541525877c`.

Tests reuse the accepted Task 2.21 synthetic account/System oracle construction
without modifying its tests. They independently check System/Token instruction
bytes, metas, account lists, seed groups, literal state discriminators/sizes and
raw Token layout. Twenty success worlds cover two runtime IDs, optional shared
manager/referrer and zero/partial/exact/excess/`u64::MAX` prefunding. All nine
persisted states are deserialized and compared with the separately prepared
approved model; guardian reward bindings and inactive/zero-claim state are checked.
Untouched accounts, including the mint, match their complete original snapshots.

Adversarial coverage includes Token role bounds/metadata/backing aliases and mint
borrows before any effect; exact approved bytes, payer inner/outer signatures,
fresh approval, Clock and stack; failure before and after every one of 40 System/
Token CPI boundaries; both Token false-success cases; every relevant Token field
and dirty unused option payload; payer, native, state and mint tampering; and
corruption of the first initialized Token vault only during the second Token CPI.
Each of nine late state-write borrow failures occurs after both Token calls and
proves that every state allocation still contains zeros: there is no partial
initial-envelope copy. Replay and detached allocated-world retry also reject.

The transaction fixture explicitly clones a whole world and discards it on error.
Separate raw calls demonstrate partial CPI effects surviving errors, while Token
failures precede all state writes. This is a **host transaction model**, not SVM,
Bank/AccountsDB rollback, signature, loaded Token processor or compute/heap proof.
Production performs no compensating rollback and every error must propagate to
the outer transaction. Separate source/test review passed, including the final
prior-Token regression. Root inspected the actual code/tests and verified the
focused evidence before independently executing the workspace gates below.

## Pilot validation and publication checkpoint

Root executed **448 host tests +1 doctest / eight gates PASS**, zero failures,
ignored tests or diagnostics, on the separately reviewed frozen source. All
commands used direct verified Rust/Cargo 1.97.1, the existing workspace cache,
an explicit clean environment and `--locked --offline --jobs 1`:

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

Evidence: `/tmp/piv1-t222-pilot-host-20260919-a/pilot-summary.json`, adjacent
exact commands/environment, source manifests, tool hashes and logs. Gates took
96.399 seconds with no failed execution or corrective retry. Root verified all
90 source inputs against its inspected freeze and the focused before/after
manifests, all sixteen gate logs, both focused logs and three unchanged tools.
Results SHA-256:
`58f9ec6cf7f7491a85e154aa96c27f6ff0eb2f802cf2f30d2d88ea5b1444d4df`.
Inspected-source manifest SHA-256:
`0fd10c9b7da7f23869a7621c11025bfba6c1dccd8a6122ac6d227ca644bae5cf`.
These are new host executions; they do not extend historical Task 2.14 SBF proof.

Root updates instructions, checkpoint, execution plan, README summaries, master
status and the requirements-to-evidence index in this task commit. Separate
final documentation/evidence review and targeted secret/generated-file and
hook/automation checks precede normal integration-only publication. Git records
the exact commit/publication identity; verify actual refs and worktree on takeover.
Save and STOP after this additional task. Task 2.23 is NOT STARTED. No RPC/chain
operation, Mainnet action, deployment, fund movement, key creation/signing or
authority transfer occurred. Founder acceptance remains pending.

## Preserved limits and next dependency

Successful account bytes do not establish full operational or Testnet readiness.
Recipient control, final initializer transport, deployed artifacts, current-source
SBF/resource/rollback evidence, economic prefund normalization and an operational
funding baseline remain unresolved before a native initializer can be exposed.
In particular, `authenticate_fixed_accounts` permits raw native excess in Token
accounts, but its separate `economic_observation` accessor still rejects that
unsupported condition. The new completion boundary deliberately does not call it
as a success gate or assign the excess a new economic destination; tests preserve
and demonstrate that limitation through `u64::MAX` prefunding.

No schema, runtime instruction ABI, dependency, accepted economics or founder-
accepted main source changes. The historical Task 2.14 SBF evidence still applies
only to its old artifact. No SBF build or public operation occurs here. The
next session must scope the remaining readiness dependency and its evidence;
no later implementation starts during this economical session.

Writer files: new `programs/piv1/src/genesis_initialization.rs`, its `src/lib.rs`
export, crate-only dispatcher visibility/comment in `src/genesis_allocation.rs`,
new `programs/piv1/tests/genesis_initialization.rs` and this report. Root owns
shared status documents, final gates, Git and reviewed integration publication.
The writer performed no Git mutation, network/RPC/chain operation, Mainnet action,
deployment, real fund movement, key creation/signing or authority transfer.
Accepted main remains `5a067ad34d2f2ab0a27f2c6bff312a8b1f772d24`. This AI-assisted
engineering/review is not a professional independent audit.
