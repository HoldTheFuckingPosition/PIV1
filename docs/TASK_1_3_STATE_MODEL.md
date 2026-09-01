# Task 1.3 State and Transition Model

## Status and scope

Task 1.3 is **IMPLEMENTED / PENDING FOUNDER ACCEPTANCE** on
`task/1.3-state-transition-model` from accepted baseline
`055c93eebd8cde2d2efac593a8d1f0aaacc949d4`.

This task implements bounded serialized state, exact planned sizes, pure timing
helpers, checked state transitions, state-specific errors, and deterministic
tests. It does not implement instruction handlers, `Accounts` contexts, CPI,
Clock or stake-account decoding, transfers, claims, deployment, a Program ID,
or randomized/property tests. Task 1.4 is **NOT STARTED**. The exact next action
is founder review of Task 1.3.

## Serialization and ownership

The five PIV1 metadata schemas derive `AnchorSerialize`, `AnchorDeserialize`,
and `InitSpace`. They deliberately do not use `#[account]`: the accepted
scaffold has no production Program ID or `crate::ID`. Consequently, `SPACE`
means the planned future allocation `8 + maximum Borsh payload`; it is not a
claim that the type is already an owner-bound deployable Anchor account.

Only these PIV1 metadata schemas are serialized:

| Schema | Maximum payload | Planned `SPACE` |
|---|---:|---:|
| `PivConfig` | 1,006 bytes | 1,014 bytes |
| `ActiveDistribution` | 883 bytes | 891 bytes |
| `WithdrawalLeg` | 255 bytes | 263 bytes |
| `GuardianRegistry` | 202 bytes | 210 bytes |
| `GuardianReward` | 76 bytes | 84 bytes |

`PivConfigBumps` is an embedded fixed 11-byte structure.
`CompletedDistributionSummary` is an embedded fixed 88-byte structure. Neither
is a separate account schema.

The canonical field-by-field serialized order is the Rust declaration order in
[`PivConfig`](../programs/piv1/src/state/config.rs),
[`ActiveDistribution` and `WithdrawalLeg`](../programs/piv1/src/state/distribution.rs),
and [`GuardianRegistry` and `GuardianReward`](../programs/piv1/src/state/guardian.rs).
The sections below account for every declaration group and type; the exact-size
tests bind those declarations to the maximum payload and planned `SPACE` values
shown above.

The externally owned `PendingSolVault`, `PrincipalSolQueue`,
`OperationalSolVault`, `DistributionEscrow`, `KifSolVault`,
`PrincipalJitoVault`, `PendingJitoVault`, and `WithdrawalStake` remain address
or role markers. `PivAuthority` also remains an address-only PDA role. Recording
their addresses in `PivConfig` does not change their System, legacy Token,
Stake Program, or address-only ownership.

## Layouts

All stored collections are fixed arrays. No state layout contains `Vec`,
`String`, a map, or another unbounded field.

### `PivConfig`

`PivConfig` contains:

- `u8` version, two `bool` initialization/pause flags, and the fixed 11-byte
  bump bundle;
- 22 explicit `Pubkey` bindings: the official stake-pool program, pool,
  validator list, reserve stake, JitoSOL mint, token/stake/system programs,
  manager fee and referrer accounts, PIV authority, active header, seven fixed
  custody addresses, two recipient addresses, and guardian registry;
- immutable V1 economics as `u16` basis-point bindings and configured/hard-cap
  slippage in `0..=1` bps;
- fixed `i64` ten-day and 24-hour durations, two `Option<i64>` timestamps, and a
  `u64` next sequence;
- protected HWM, historical/pending SOL and JitoSOL quantities, explicit
  next-cycle yield, KIF claim liability, and collective carry as separate
  `u64` ledgers;
- monotonic contribution, gross-yield, beneficiary, KIF, compound, retained
  dust, zero-active compound, and cooldown-yield audit totals;
- `i64` KIF anchor/duration, `u64` guardian revision, and an explicitly zeroed
  `[u8; 64]` migration reserve.

Validation rejects an unsupported version, an uninitialized layout, default or
invalidly aliased required addresses, altered fixed economics/timing, slippage
above one basis point, nonzero reserved bytes, and unreconciled KIF claims.
Sequence and HWM helpers stage checked updates without wrapping.

### `ActiveDistribution`

The single reusable header contains no leg list. It stores:

- version, bump, explicit initialization, coarse lifecycle, recovery flags,
  settlement replay flag, active sequence, and optional prior terminal summary;
- preparation `i64` timestamp, `u64` slot/epoch, old HWM, historical SOL/JitoSOL
  quantities and value, and official pool/fee snapshot inputs;
- exact gross yield, the consumed prior-next-cycle-yield snapshot, four fixed
  gross allocations, split dust, outgoing obligation, pending-SOL snapshot/use,
  and conversion dust;
- fixed JitoSOL target, snapshot input floor, mathematical useful-leg bound,
  conservative round native/HWM floors, and `u16` slippage;
- fourteen `u64` cumulative leg quantities covering assigned/fee/burn,
  expected/delegated/finalized native output, both rent categories, cooldown
  reward/loss, indices, and counts;
- recorded escrow, outstanding active liability, fixed recipients, registry
  revision, `[Pubkey; 6]` guardian snapshot, six-bit activity bitmap/count,
  period ID, and prior KIF carry;
- proposed and actual settlement values, conservative retained dust, KIF
  liability/carry/zero-active compound, actual HWM delta, and settled HWM.

The exact split, gross-yield basis, proposed HWM components, pending-SOL use,
escrow equations, fee-plus-burn identity, count relationships, recovery flags,
actual obligation caps, KIF allocation, and HWM equations are rederived by
`validate`. A terminal summary survives the return to `Idle`.

### `WithdrawalLeg`

Each fixed metadata record is bound to one `(u64 sequence, u64 leg_index)` and
contains:

- version, metadata/stake bumps, initialization, status, and recovery flags;
- `u32` validator-list index and seed suffix plus fixed vote/source keys;
- initiation epoch, current pool/fee inputs, technical floor, exact JitoSOL
  input, fee/burn, expected/delegated/minimum native output, and both rent
  advances;
- `Option<u64>` finalized epoch, final native output, both recovered rent
  categories, cooldown reward, and cooldown loss.

Vacant, initiated, and finalized forms are validated separately. Final epoch
cannot precede initiation; fee plus burn equals input; recovered rent equals
its advance; reward and loss are mutually exclusive; and final native value
reconciles exactly with delegated value, rent, reward, and loss.

### Guardian layouts

`GuardianRegistry` stores `u8` version/bump, `u64` revision, and exactly
`[Pubkey; 6]`. Keys must be nondefault and unique. It validates six reward
bindings and derives a checked six-bit activity snapshot for one KIF period.

`GuardianReward` stores version, bump, fixed `u8` slot, registry revision,
guardian key, `Option<u64>` last-active period, and separate claimable,
cumulative-earned, and cumulative-claimed `u64` values. `None` is the explicit
never-active representation. Credits and activity updates are staged. A
snapshot-credit method remains bound to the immutable round key/slot/revision
even if the live registry later rotates.

## Integer-width decisions

- Lamports, token units, sequences, leg indices, slots, epochs, counts, and
  cumulative values use `u64`. This avoids imposing an unapproved low economic
  or useful-leg cap.
- Solana Clock timestamps and configured durations use `i64`.
- Validator-list indices and seed suffixes use `u32`, matching their bounded
  indexing role without using platform-dependent `usize` in serialization.
- Basis points use `u16`; versions, bumps, compact flags, the guardian count,
  and bitmap use `u8`.
- Intermediate economic arithmetic uses the founder-accepted `piv1-math`
  checked operations where semantics match; all state additions/subtractions
  are checked. There is no floating point or `unsafe`.

## Lifecycle representation

A single enum cannot represent the confirmed non-linear lifecycle because an
early leg may finalize while later target assignment continues. The normalized
stored lifecycle is therefore deliberately coarse:

```text
Idle
WithdrawalActive
EscrowFunded
Settled
RecoveryRequired
```

Pause is an orthogonal `PivConfig` flag. `Completed` is recorded as a bounded
terminal summary before the reusable header returns to `Idle`.

The following conceptual states are derived, not redundantly stored:

| Conceptual predicate | Derivation |
|---|---|
| `PreparedWithdrawal` | active withdrawal, nonzero target, zero assigned |
| `AssigningWithdrawalLegs` | active withdrawal, `0 < assigned < target` |
| `WithdrawalTargetAssigned` | nonzero target and `assigned == target` |
| `AwaitingLegInactivity` | `successful_count > finalized_count` |
| `PartiallyFinalized` | finalized count is nonzero and withdrawal is incomplete |
| all successful legs finalized | nonzero successful count and equal counts |
| withdrawal complete | exact target assigned and all successful legs finalized |

Stored settlement/recovery replay flags are validated against the lifecycle and
counters. Exact equality, not a caller-provided boolean, triggers target
assignment and completion.

## Transition API

All functions consume numeric/account facts that future handlers must first
derive from trusted accounts and sysvars. They perform no RPC, CPI, transfer, or
account decoding.

| Function | Legal source | Successful state effect |
|---|---|---|
| `record_no_yield_evaluation` | `Idle`, unpaused, ten-day gate | no mutation; prior next-cycle yield also must be zero |
| `record_valid_insufficient_attempt` | `Idle`, unpaused, timing gates | updates only the 24-hour timestamp |
| `open_distribution` | `Idle`, unpaused | allocates one sequence, snapshots immutable economics/guardians, consumes prior next-cycle yield once, then enters `EscrowFunded` or `WithdrawalActive` |
| `initiate_withdrawal_leg` | `WithdrawalActive`, unpaused | records exact maximum-safe fill and advances checked cumulative counters/index once |
| `finalize_withdrawal_leg` | initiated in-range leg, unpaused | records distinct rent/reward/loss and enters `EscrowFunded`, remains active, or commits `RecoveryRequired` |
| `settle_distribution` | exactly reconciled `EscrowFunded` | records bounded actual allocations, KIF/HWM accounting, and enters `Settled`, or commits residual-HWM recovery |
| `integrate_pending_and_complete` | `Settled`, zero liability | integrates all currently accounted pending contributions, records summary, returns header to `Idle` |

Opening uses pending SOL first, then the amount of prior next-cycle yield needed
for outgoing liquid funding. A liquid-only round validly stores a zero JitoSOL
target. A withdrawal-tagged round requires a nonzero target and exact finite
useful-leg bound.

Task 1.3 does not invent how protocol-cost shortfall is apportioned among HTFP,
Team Owner, and KIF. `SettlementInput` therefore receives actual amounts already
validated by a future handler under the policy in force. The state transition
requires each amount to be at or below its immutable gross obligation, requires
their sum plus conservative allocation dust to equal the exact available net,
and revalidates escrow, KIF, carry, and HWM identities. The exact shortfall
allocation policy remains **OPEN** for the later handler task; no percentage or
priority is silently introduced here.

## Mutation safety and replay protection

Every multi-object transition copies the complete mutable inputs, performs all
checked arithmetic and invariant validation on the copies, and commits only
after every check succeeds. Errors therefore preserve complete pre-state,
including sequence/index overflow, reward-credit overflow, late settlement
overflow, and cross-object reconciliation failures.

Sequence allocation happens once during opening. Active transitions require the
config next sequence to equal `active_sequence + 1`. A retained terminal summary
prevents reopening its completed sequence. Leg initiation requires a vacant
record and the exact next index; finalization requires an initiated leg whose
index is already inside the header's consumed range. Settlement and finalization
replays are explicit errors. No normal reset, cancellation, contributor
withdrawal, or permissionless recovery path exists.

## Timing and pause

Pure helpers enforce 864,000-second preparation spacing, the exact 86,400-second
valid-insufficient retry cooldown, signed timestamp regression checks, and
2,592,000-second KIF periods with half-open boundaries derived from a configured
anchor. Exact boundaries pass and checked overflow fails.

The confirmed narrow pause policy gates distribution evaluations/opening, new
withdrawal-leg initiation, and leg finalization. Settlement and pending
integration validate configuration but are not pause-gated because G-004 does
not yet explicitly designate those resumability boundaries. Whether they should
freeze during an incident remains **OPEN** for the later threat-model/handler
task; Task 1.3 does not simulate governance authorization with a caller boolean.

## Error model

`Piv1Error` has concrete Task 1.3 categories for version/initialization,
lifecycle/pause/recovery, timestamp and cadence failures, sequence/index/replay,
zero or excessive target/input, maximum-safe fill and technical floors, useful
leg bounds, count/cumulative/escrow/obligation/liability reconciliation, HWM,
guardian bitmap/count/set bindings, address/slippage/split/timing bindings, and
checked arithmetic. `piv1-math` arithmetic failures map to state arithmetic;
invalid active-guardian count maps to its dedicated state error. No CPI- or
protocol-instruction-specific error was added.

## Deterministic test matrix

The deterministic Rust suite covers:

- exact derived/manual maximum sizes, planned `SPACE`, maximum enum/option
  branches, round trips, fixed arrays, and reserved bytes for all five schemas;
- versions, initialization, default addresses, 0/1-bps slippage, pause,
  sequence/HWM overflow, exact timing boundaries, guardian count/bitmap,
  registry rotation snapshots, and KIF activity/carry;
- no-yield, valid insufficiency, liquid-only, one- and multi-leg assignment,
  both finalization orders, finalization before target assignment, escrow,
  settlement, all-pending integration, completion, cooldown yield reuse, and
  both recovery causes;
- wrong state/sequence/index, zero input/target, micro-leg, floor, overshoot,
  extra/useful-bound legs, initiation/finalization/settlement replay, premature
  settlement, counts, escrow, obligation/HWM/liability mismatch, cancellation,
  recovery blocking, epoch regression, and arithmetic overflow;
- complete pre-state equality after important rejected transitions.

Randomized/property tests, fuzzing, RPC, validators, wallets, keypairs, and funds
are absent. They remain outside Task 1.3.

## Deferred external validation and exclusions

Future instruction/handler work must validate:

- real PDA/address derivation after a real Program ID is authorized;
- official Jito/SPL program, pool, mint, validator-list, reserve, source-order,
  fee/conversion, maximum-capacity, residual, and slippage facts;
- Clock, epoch, Stake History, stake inactivity, stake authority, account owners,
  exact custody deltas, rents, and escrow balances;
- the founder-approved protocol-cost shortfall allocation policy;
- contribution conversion values and resulting historical asset quantities;
- actual governance signatures and the final pause/claim threat policy.

Task 1.3 contains no handler, `Accounts` derive, CPI, Jito formula, dynamic
minimum calculation, validator selection, transfer, claim, event emission,
Program ID, key generation, deployment configuration, RPC/client integration,
property test, Task 1.4, or Phase 2 work.

## Validation record

The final implementation is validated with the pinned Rust 1.97.1 toolchain and
offline locked dependencies using:

```text
cargo +1.97.1 check -p piv1 --all-targets --locked --offline
cargo +1.97.1 test -p piv1 --all-targets --locked --offline
cargo +1.97.1 check --workspace --all-targets --locked --offline
cargo +1.97.1 check --workspace --all-targets --all-features --locked --offline
cargo +1.97.1 test --workspace --all-targets --locked --offline
cargo +1.97.1 test --workspace --doc --locked --offline
RUSTDOCFLAGS="-D warnings" cargo +1.97.1 doc --workspace --no-deps --locked --offline
git diff --check
git fsck --full --strict
```

Final results on 2026-09-01, executed as `jerem` with explicit
`HOME=/home/jerem`, are:

- both `piv1` check commands passed;
- the `piv1` all-target test command passed 62 deterministic tests: 33 unit,
  19 illegal/replay, and 10 legal-lifecycle tests;
- both workspace check commands passed, including all features;
- the workspace all-target test command passed 94 deterministic tests: the 62
  `piv1` tests plus the 32 founder-accepted `piv1-math` unit tests;
- workspace documentation tests passed: zero `piv1` doctests and one
  `piv1-math` doctest;
- warning-denied workspace documentation generation passed;
- working and staged diff whitespace checks passed;
- dependency/lockfile diffs were empty and their SHA-256 values matched the
  accepted baseline byte for byte;
- `git fsck --full --strict` exited successfully with no corrupt or missing
  objects (only informational unreachable dangling-blob notices); and
- current tracked/new Task 1.3 files and reachable Git history passed
  secret-like path and high-confidence credential/private-key marker scans.

The `rustfmt` and `clippy` components were not installed and were not added.
`anchor build` was intentionally not run because it may generate an
unauthorized key artifact.
