# Task 2.20 — Approved genesis account preflight

Status: **TECHNICALLY VALIDATED / PENDING FOUNDER ACCEPTANCE**.
Base: `5a067ad34d2f2ab0a27f2c6bff312a8b1f772d24`, `integration/piv1-testnet`.
The accepted Tasks 2.3–2.19 source milestone remains `d9f3371` under D-027.
This bounded D-026 source/host task composes existing prerequisites into a
read-only observation. It does not initialize PIV1 or extend founder acceptance.
The founder requested one economical task, publication/checkpoint, then stop.

## Fresh composition and target observations

`preflight_approved_genesis_accounts` takes the actual runtime Program ID,
ordered account slice, exact instruction bytes and trusted handler role indices.
It freshly prepares the Task 2.18 approved model from those same inputs. The
existing model dispatcher is reused through a crate-private visibility change;
its public behavior and decode/authorization ordering are unchanged. Stack,
Clock and Rent are acquired once. The same Rent value is retained for target
observations, while the same Clock still drives approval and KIF period checks.
Ordinary host calls reject before reads; the explicit modeled host seam is absent
from Solana. No public API accepts detached model/identity proofs or independent
replacement declarations.

Jito declarations derive solely from the fresh proposed Config. Task 2.19 then
authenticates seven actual protocol accounts selected from the same approved
slice. Its fixed source identities, bounded parser, raw fee/epoch/supply facts
and operational limitations remain unchanged. Recipient key approval does not
prove recipient control; no new protocol readiness or cluster/artifact claim
is added.

All sixteen model-derived targets must match their actual canonical keys and
be writable, nonexecutable, currently System-owned and empty through fallible
borrows. This is current unallocated status, not proof of never having been
initialized. Intended owners/sizes remain descriptors: nine future PIV1 state
accounts, five native System vaults and two future 165-byte legacy Token accounts.
The shared PIV authority remains virtual, not a seventeenth target.

Role groups are disjoint except target zero reusing bootstrap Config and optional
manager/referrer sharing of one protocol index. Bounds and target uniqueness are
checked before new indexed access. Existing exact approval validation preserves
global account-key uniqueness and approved privileges. Differently keyed targets
cannot share data or lamport `Rc` backing with another supplied account; this
also prevents synthetic fixtures from counting one backing store twice. Existing
guardian/recipient overlap remains allowed. No signer or payer role is invented.

For each target, the existing checked `accounts::rent_floor` prices its intended
allocation using the captured Rent. The output records its descriptor, raw
observed lamports, rent minimum and checked shortfall:

```text
shortfall = rent_minimum - observed_lamports, when observed_lamports < rent_minimum
shortfall = 0, otherwise
total_rent_shortfall = checked sum of the sixteen shortfalls
```

Zero, partial, exact and excess prefunding remain untouched. No raw-prefunding
aggregate is calculated; surplus at one target cannot offset another's shortfall.
No prefunding enters principal, pending, yield, claims, carry or operational
accounting. The proposed model remains identical when only target balances vary.
Rent shortfalls identify neither a funding source nor permission to spend; they
exclude transaction costs and required operational liquidity/funding provenance.

`GenesisAccountPreflight` has private fields, no public constructor and no Clone
implementation. Getters expose the model, protocol identity and target/rent
observations. This is point-invocation evidence, not a creation/persistence
capability, durable snapshot, readiness certificate or replay receipt. Repeating
preflight freshly reads the accounts; callers must refresh after mutation/CPI.

## Validation and preserved boundaries

The source/oracle basis is unchanged: [Task 2.18](TASK_2_18_APPROVED_GENESIS_MODEL.md),
[Task 2.19](TASK_2_19_JITO_ACCOUNT_IDENTITY.md), their pinned Squads/Jito/SPL
references, existing checked Rent helper and schemas. Tests reuse the accepted
Squads invocation fixture and exact-source Jito oracle/actual Stake serializer.
No network research, package install, dependency/source pin or schema change
was needed. Existing instruction/error ABI and entrypoint remain unchanged:
only claims and pending recognition are dispatched. The genesis model selector
still rejects, including through the explicit host dispatch seam.

After source/test inspection, one focused command used direct verified 1.97.1
tools, a clean explicit environment, existing cache and one job:

```text
/home/jerem/.rustup/toolchains/1.97.1-x86_64-unknown-linux-gnu/bin/cargo test --locked --offline --jobs 1 -p piv1 --test genesis_preflight
```

**10 tests PASS**, zero failures, ignored/filtered tests or diagnostics; elapsed
5.641 seconds. No preparation/build/test failure or corrective retry occurred.
Pre-run API inspection removed an unsupported role Eq derive and unused import;
no accepted API changed. Regressions cover both modeled Program IDs, all sixteen
intended owners/spaces, independent default-rent arithmetic, all eighteen zero
economic histories, unchanged idle/reward state, every target's key/owner/
executable/data/writable and borrow failures, invalid/group/ backing aliases,
allowed manager/referrer sharing, exact-message substitutions, fresh approval/
authority/protocol/Clock/depth after prior success, malformed Rent, individual
valid floors with aggregate overflow, and prefunding through `u64::MAX`.
Complete account/instruction/context comparisons establish input preservation,
including bitwise comparison of modeled NaN Rent. These are synthetic host
observations, not actual initialization or Squads runtime execution.

Evidence: `/tmp/piv1-t220-writer-20260914-hf1slcng` contains exact command/environment,
verified tool hashes, before/after input manifests, result and logs. Inputs are
unchanged by validation. Stdout SHA-256:
`02d857dc363a93a4bff33b1bd3bbc0226a262e022205877d24fbf8b59335c048`;
stderr SHA-256:
`d733be8d1d7826cc93e228c62373e3ab98d37d04f65b77627bb5c1e495981a5f`.
Root inspected all production changes and ten tests. Separate frozen source/test
review passed with no actionable findings. Root then executed **426 host tests
+1 doctest / eight gates PASS**, zero failures, ignored tests or diagnostics, on
the same reviewed source. No preparation/build/test failure or retry occurred.
All commands used direct verified Rust/Cargo 1.97.1, a clean explicit environment,
the existing workspace target cache and `--locked --offline --jobs 1`:

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

Root evidence: `/tmp/piv1-t220-pilot-host-20260914-a/pilot-summary.json`, adjacent
exact commands/environment, before/after source manifests, tool hashes and logs.
Elapsed gates: 86.083 seconds. All 86 inputs match the inspected source and
remained unchanged during validation; the writer's scoped freeze also matches.
Root verified all sixteen gate log hashes, both writer log hashes and the three
unchanged previously reviewed host tools. Results SHA-256:
`190573e28aa0952509295ddfbf2ac9b9faa04d7b35fd773975e26eaef35a5fcf`.
Root inspected-source manifest SHA-256:
`3e5d6493157d1e30318af62d42e054ad300f7aceb099e2470d6303ffad87f423`.
This is new host execution evidence. The older 24 SBF tests/70 cases still apply
only to the Task 2.14 artifact; no SBF/runtime evidence was refreshed here.

Writer files: new `programs/piv1/src/genesis_preflight.rs`, `src/lib.rs` export,
the crate-private dispatcher comment/visibility in `src/genesis_model.rs`, new
`programs/piv1/tests/genesis_preflight.rs` and this report. No other production,
fixture, manifest, lock, toolchain or historical task report changes were made.
Actual creation, funding authorization/provenance, recipient control, final
initializer transport, governance/activity integration and current-source
SBF/runtime resource/rollback evidence remain deferred. No follow-on task is
started in this session.

The writer performed no Git operation or commit; root owns final status and
publication on integration, preserving accepted main. No SBF build, RPC/chain
operation, Mainnet action, deployment, fund movement, key creation/signing or
authority transfer occurred. AI-assisted review is not a professional audit.

The task commit also updates the pilot checkpoint, repository instructions,
current README/master summaries, execution plan and requirements-to-evidence
index. Git records its exact implementation/publication identity. Normal
fast-forward publication is limited to `integration/piv1-testnet`; accepted main
remains `5a067ad34d2f2ab0a27f2c6bff312a8b1f772d24`. After final documentation and
publication checks, the session stops as requested. Next session: separately scope
prefunding-safe atomic account creation and its explicit funding provenance,
recipient/transport constraints and post-creation state validation. No Task 2.21
implementation or live-operation authorization is implied.
