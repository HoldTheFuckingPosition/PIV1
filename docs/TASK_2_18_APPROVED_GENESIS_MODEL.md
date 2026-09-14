# Task 2.18 — Approved genesis model preparation

Status: **TECHNICALLY VALIDATED / PENDING FOUNDER ACCEPTANCE**.
Base: `212e5a9fa4de2316e7abe2f5ff5cfe837c29b490`, `integration/piv1-testnet`.
This D-026 source/host task prepares proposed state only. It does not initialize
PIV1, authenticate official Jito or recipients, establish payout readiness or
constitute founder acceptance.

## Format and authorization composition

The initialization marker remains unchanged. A distinct library model domain,
ASCII `PIV1GM01`, introduces this exact fixed-size preparation format:

| Offset | Field | Bytes |
|---|---|---:|
| 0 | Model selector | 8 |
| 8 | Format version, exactly 1 | 1 |
| 9 | Squads vault index | 1 |
| 10 | Approved initial pause, canonical bool | 1 |
| 11 | Seven declared protocol keys | 224 |
| 235 | HTFP and Team Owner recipients | 64 |
| 299 | Approved KIF anchor, signed little-endian timestamp | 8 |
| 307 | Six-slot permutation of sorted current Squads members | 6 |
| Total | | 313 |

Protocol key order: pool program, pool, validator list, reserve stake, JitoSOL
mint, manager fee account and referrer account. These are explicitly unverified
declarations. Strict decoding rejects wrong length/domain/version, noncanonical
bool, duplicate/out-of-range slots and trailing bytes. Encoding and decoding
use fixed arrays without input-sized allocation. No live addresses are embedded.

`prepare_approved_genesis_model` decodes the actual instruction bytes and freshly
calls Task 2.17 using those same bytes, actual ordered accounts and runtime ID.
There is no public constructor accepting detached approval evidence and separate
parameters. A small crate-private visibility change reuses the existing bootstrap
body; Task 2.16/2.17 public behavior and parsers are unchanged. One fetched Clock
value drives both authorization and `derive_kif_period`; runtime stack and Rent
also come from the wrapper. Ordinary host calls reject before context/account
reads. The explicitly named host seam supplies modeled context only.

The current native entrypoint rejects this model selector, including through its
host dispatch seam. This is not a callable initializer ABI or a transport claim.

## Deterministic proposed state

All addresses and bumps derive from the trusted runtime PIV1 ID. The model
contains sixteen expected target descriptors: Config, idle ActiveDistribution,
registry, six rewards, five native System vaults and two 165-byte legacy Token
vaults. The nine state accounts use existing schema sizes and PIV1 ownership.
The shared PIV authority is a virtual address, not a seventeenth created account.
Config, round and the fixed descriptor array use bounded boxed construction
with separated construction frames; this does not prove an SBF resource budget.

The proposed Config uses immutable split/timing/constants, canonical System/
Token/Stake IDs, **initial slippage exactly 1 bps** under A-001, the approved pause
value and anchor, sequence/revision zero, zero economic and audit histories,
`None` prior timestamps and zero migration reserve. The idle round and six
inactive zero-liability reward records use existing constructors. The approved
permutation assigns the freshly authenticated six members to explicit PIV1 slots.
Config, registry, rewards and their cross-bindings are checked.

All sixteen target addresses and PIV authority must be distinct and cannot alias
external declarations or authentication metadata, except the required supplied
Config identity. Guardians cannot alias PIV state/custody/authority/reward PDAs.
Existing Config default/separation rules apply; manager/referrer alias and
guardian/beneficiary overlap remain allowed. A Squads treasury recipient is not
arbitrarily prohibited. No new on-curve or guardian signer constraint is inferred.

An approved explicit anchor may be negative or precede execution. Existing checked
period arithmetic rejects future anchors and subtraction/end overflow and preserves
half-open periods. No new backdating policy is selected. Rewards remain inactive
model records: Squads votes do not automatically become KIF activity. Whether and
how an initialization vote qualifies under K-010 remains an integration dependency
before payout readiness; this task does not select a vote-exclusion policy.

## Evidence and limitations

Authoritative basis: [A-001/A-003 and K-010](PIV1_DECISIONS.md), master
[sections 11–13](PIV1_MASTER_SPEC.md), accepted state constructors and the immutable
Squads references preserved in [Task 2.17](TASK_2_17_SQUADS_BOOTSTRAP_AUTHORIZATION.md).
No repeated transport/source research or new source pin was needed.

The private-field, non-Clone `ApprovedGenesisModel` exposes proposed-state and
derived-descriptor getters. It is not an authenticated custody snapshot or a
ready-to-create capability. No non-Config target validity, actual ownership/data,
balance, rent, funding provenance or custody is authenticated. Prefunding is
untouched and never enters proposed principal, pending, yield or operational
accounting. Official cluster/program/pool/list/reserve/mint/fee/referrer validation,
recipient control, actual prefunding-safe creation, protocol integration, guardian
activity and first-payout readiness remain deferred. Reprepare after mutation/CPI;
no effect-once receipt or persistent initialization transition exists.

Writer changes: `programs/piv1/src/genesis_model.rs`, its `lib.rs` export,
`src/instructions/initialize.rs`, the crate-private bootstrap seam comment/
visibility in `src/squads_execution.rs`, `tests/genesis_model.rs`, and this report.
Existing tests/support, state layouts, instruction/error mappings, entrypoint,
manifests, lockfile, dependencies and source pins remain unchanged.

After source inspection, one direct verified Rust/Cargo 1.97.1 run used the
existing cache, clean environment, locked/offline mode and one job:

```text
/home/jerem/.rustup/toolchains/1.97.1-x86_64-unknown-linux-gnu/bin/cargo test --locked --offline --jobs 1 -p piv1 --test squads_execution --test squads_bootstrap --test genesis_model
```

**41 tests PASS**: twelve new model tests plus all nine Task 2.17 and twenty
Task 2.16 tests. Zero failures, ignored tests or diagnostics; elapsed 7.671 seconds.
No failed preparation or corrective retry occurred. Coverage includes independent
format offsets/malformed inputs, exact approval substitutions, fresh context,
slot/pause choices, multiple runtime IDs and complete topology, all zero ledgers,
timing limits, aliases/allowed overlaps, prefunding and full input immutability.
Synthetic initialized accounts serialized through existing StateEnvelope helpers
pass both fixed-account and guardian/Clock authentication, with inactive guardians
and unchanged fixture contents. This is modeled compatibility, not account creation.

Evidence: `/tmp/piv1-t218-writer-20260914-jmwtlsl1` contains exact `command.json`,
`result.json`, logs and before/after source manifests. Source preservation passed.
Stdout SHA-256: `52c8b26cd8b4e256f32aa90add07d2e09b25d057efa15027b2cfef68c0c19ffd`.
Stderr SHA-256: `7bc6bd067ec2b37e487ec0a903bf97469fffd61d572cdda6afb71ccf9e0d3183`.
The writer performed no Git operations or commits and did not alter main.

Root independently inspected all five changed source/test files and executed the
eight final host gates on that unchanged freeze: **404 host tests +1 doctest PASS**,
zero failed/ignored tests or diagnostics. All commands used the direct verified
1.97.1 Cargo/Rust/Rustdoc toolchain, a clean explicit environment, existing cache,
`--locked --offline --jobs 1`:

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

Evidence: `/tmp/piv1-t218-pilot-host-20260914-a/pilot-summary.json` and adjacent
exact commands, environment, tool/source manifests and logs. Elapsed gate time:
68.785 seconds. Root verified all sixteen final log hashes, both writer logs,
full source preservation, the independently inspected source freeze and unchanged
previously verified host tool hashes. Results SHA-256:
`cec00614da6ac030f932c8dfc5d5de533565e31f3aae44ed2ca6c79e02225834`.
Root inspected-source manifest SHA-256:
`7a9122b192da5f55038a2bdc9113bd3f1406aaff7d857610f2e1600abdeef6c8`.
No new SBF build or execution is claimed; Task 2.14 artifacts remain historical.
Separate final source/test/report review passed with no actionable findings. The
reviewer independently matched the writer's source freeze/log hashes and root
results identity and confirmed the model-only boundaries. Git closure follows
in the pilot checkpoint. This AI-assisted review is not a professional audit.

The next technical dependency is source-pinned read-only protocol-account
authentication against an explicit trusted deployment identity. Actual target
readiness, rent shortfalls and atomic prefunding-safe funding/allocation/assignment/
Token initialization remain a separate boundary. Do not promote this model to
ready-to-create evidence or infer official identity from the approved declarations.

No Mainnet action, deployment, target build/run, package installation, RPC/chain
access, fund movement, signing, key creation or authority transfer occurred.
