# Task 2.1 narrow stake-pool interface and deterministic host mock

Date: 2026-09-03 UTC

Branch: `task/2.1-mock-stake-pool-adapter`

Accepted starting baseline: `7626c0bc0e46ab7437162be7939e01c1c6eff619`

Status: **IMPLEMENTED / PENDING FOUNDER ACCEPTANCE**

Phase 2 status: **IN PROGRESS**

Task 2.2 and later Phase 2 tasks: **NOT STARTED**

## Scope and result

Task 2.1 replaces the empty `StakePoolAdapter` marker with a synchronous,
statically dispatched contract, bounded value and error types, and a
fixed-capacity host-only mock. It adds deterministic fixed and randomized tests
for snapshots, deposits, maximum-safe withdrawal legs, delayed finalization,
failure atomicity, replay protection, and conservation.

The task does not add an Anchor handler, `Accounts` context, account decoder,
CPI, live address, RPC/HTTP dependency, local validator, Program ID, SBF
artifact, deployment, wallet, keypair, or fund movement. The program crate
remains a non-deployable ordinary Rust library.

The mock is a deterministic simulation of the interface. It is not an
implementation or independent validation of exact SPL Stake Pool 2.0.3 or Jito
behavior. Exact pinned protocol instruction, account, source-order, fee,
conversion, stake-state, Rent, and balance-delta mapping remains Phase 3 work.

## Interface

`StakePoolAdapter` has seven synchronous methods:

| Method | Purpose |
|---|---|
| `pool_snapshot` | Return one intrinsically valid and current bounded pool observation. |
| `quote_sol_deposit` | Derive gross tokens, fee, net output, and protected minimum without mutation. |
| `execute_protected_sol_deposit` | Revalidate and execute the deposit intent, independently deriving the result. |
| `quote_stake_withdrawal` | Derive one maximum-safe source fill, fee, burn, output, minimum, and capacity without mutation. |
| `initiate_protected_stake_withdrawal` | Revalidate the intent, initiate the delayed leg, and return exact categorized results. |
| `delayed_withdrawal` | Return the current initiated/deactivating/inactive/finalized view for one identifier. |
| `finalize_delayed_stake_withdrawal` | Finalize one inactive identifier once and return native, reward/loss, and rent categories. |

There is no `dyn` use, boxed value, collection parameter, callback, future,
network handle, wall-clock input, or arbitrary object labeled as validated.
Quote objects are informational and are not accepted by execution methods.
Execution accepts the original operation intent, compares its exact bounded
snapshot identity, and independently rederives all economic output.

The request's caller minimum cannot weaken PIV1 protection. Each adapter must
derive the hard-capped slippage floor and enforce the greater of that floor and
the caller minimum. The boundary accepts only `0..=1` bps and fixes the
denominator at 10,000.

## Production-facing values

### Fee and snapshot

`FeeFraction` stores a `u64` numerator and denominator. PIV1 has exactly one
internal zero-fee representation: `0 / 1`. Every other zero-numerator fraction
is rejected as noncanonical with `InvalidFee`; a zero denominator is always
`DivisionByZero`. A nonzero numerator must be strictly below its nonzero
denominator. If the external SPL representation uses a zero denominator for no
fee, Phase 3 must translate it to `FeeFraction::ZERO` only after validated
protocol decoding. That normalization belongs to the future real adapter.

`PoolSnapshotIdentity` stores current epoch, pool `last_update_epoch`, and a
bounded `u64` revision. Exact quote-to-execution identity equality is required.
The current revision counter is deterministic mock state; it does not assume
that SPL Stake Pool exposes this exact `u64` revision. A collision-safe identity
derived from validated accounts remains Phase-3-provisional and must be
resolved when the real adapter is implemented.

`PoolSnapshot` stores:

- current epoch and pool `last_update_epoch`;
- total pool lamports and pool-token supply;
- SOL-deposit and stake-withdrawal fee fractions;
- current minimum delegation in native lamports;
- bounded maximum deposit capacity;
- available native withdrawal liquidity after adapter safety reservations;
- revision.

The only valid zero-supply state has both total lamports and supply equal to
zero. It is the explicit mock bootstrap state. Exactly one zero, a future pool
update epoch, stale state, zero minimum delegation, invalid fees, or available
liquidity greater than total pool lamports is rejected explicitly.

### Protected SOL deposit

`SolDepositRequest` contains exact snapshot identity, nonzero native input,
caller minimum output, and bounded slippage. `SolDepositQuote` contains native
input, gross pool tokens, fee tokens, quoted net output, independently derived
slippage floor, and effective minimum. `SolDepositExecution` returns the quote
plus exact actual user output and actual fee output.

### Protected stake withdrawal

`WithdrawalId` is the deterministic `(u64 sequence, u64 leg_index)` pair.
`WithdrawalSourceId` is a bounded `u32` source identity suitable for later
mapping from a checked validator-list index.

`StakeWithdrawalRequest` contains exact snapshot identity, withdrawal and
source identities, the fixed round's remaining token target, caller minimum
native output, and bounded slippage. It does not contain a caller-selected leg
amount. `StakeWithdrawalQuote` returns:

- remaining target and current source capacity;
- dynamically calculated technical minimum in pool-token units;
- exact maximum-safe pool-token input;
- fee and burned units;
- expected delegated native output;
- derived slippage floor and effective native minimum.

`StakeWithdrawalInitiation` adds actual delegated native output, initiation and
deactivation epochs, first eligible finalization epoch, delayed status, and
stake/metadata rent values.

### Delayed finalization

`DelayedWithdrawalStatus` explicitly represents `Initiated`, `Deactivating`,
`Inactive`, and `Finalized`. `DelayedWithdrawal` is the read-only status view.
`FinalizeWithdrawalRequest` contains only the deterministic identifier.
`StakeWithdrawalFinalization` returns all epochs, delegated native value,
cooldown reward or loss, both recovered rent categories, final native value,
and terminal status.

## Error model

`StakePoolError` is an ordinary bounded Rust enum with no Anchor error number.
It distinguishes:

- `InvalidSnapshot` and `InvalidConfiguration`;
- `StalePool` and `StaleQuote`;
- `ZeroInput`;
- `DivisionByZero`, `ArithmeticOverflow`, and `NarrowingConversion`;
- `InvalidFee`, `InvalidSlippage`, and `SlippageExceeded`;
- `TechnicalMinimumNotMet`;
- `InsufficientPoolLiquidity`;
- `UnknownWithdrawalSource` and `InsufficientSourceCapacity`;
- `InsufficientOperationalRent`;
- `UnknownWithdrawalIdentifier`, `IdentifierReuse`, and `AlreadyFinalized`;
- `WithdrawalNotInactive`;
- `InjectedMockFailure`.

Mapping these categories to future production PIV1/Anchor errors is deferred
until handlers and the real adapter are separately authorized.

## Stability boundary

### Stable accepted requirements

- synchronous deterministic operation with static dispatch;
- bounded identifiers and values and no production collection growth;
- current pool state, exact quote identity, protected variants, and immutable
  one-basis-point slippage cap;
- zero-input rejection;
- checked integer arithmetic, conservative outgoing floors, and conservative
  fee ceilings;
- program-derived maximum-safe per-source leg input;
- dynamic technical minimum and explicit liquidity/source failures;
- delayed per-leg lifecycle, reward/loss/rent categorization, and replay
  protection;
- execution output must come from validated protocol observations/results, not
  an arbitrary caller claim;
- failure must not commit partial adapter mutation.

### Mock-only behavior

- the simple total-lamports/pool-supply formula described below;
- explicit bootstrap minting at one pool token per native lamport;
- `maximum_deposit_lamports` as a controllable simulated capacity;
- exactly eight source slots and sixteen withdrawal-record slots;
- source capacity directly configured in pool-token input units;
- exact deterministic mock output equal to the current quote;
- per-source cooldown epochs, reward/loss, and rent controls;
- explicit failure-injection points;
- in-memory revision and audit ledgers.

### Phase-3-provisional mapping

- collision-safe, account-derived meaning and representation of the revision
  token;
- translation of validated external zero-fee encoding to the unique internal
  `FeeFraction::ZERO` representation;
- exact bootstrap behavior of the pinned live program;
- exact deposit-capacity mapping, if any;
- derivation of source token capacity from validator stake, source residuals,
  preferred/source ordering, and current pinned SPL logic;
- exact technical-minimum search inputs from Rent, Stake Program minimum
  delegation, pool fee/conversion math, and residual rules;
- exact account/balance deltas, stake inactivity decoding, and rent sources;
- detailed production/Anchor error mapping.

No mock-only rule is marked as confirmed real SPL/Jito behavior.

## Host-only mock state and controls

`MockStakePool` exists only at
`programs/piv1/tests/support/stake_pool_mock.rs`. The production crate neither
declares nor exports it. Its complete comparable state contains:

- current snapshot and initial pool accounting baselines;
- current/initial operational-rent amounts;
- `[MockWithdrawalSource; 8]` and initial source capacities;
- `[MockWithdrawalRecord; 16]`;
- `u8` withdrawal count;
- fixed `MockAudit` counters;
- one optional failure point.

Controls cover exchange-rate reward and loss, fee changes, fresh/stale epochs,
deposit and aggregate withdrawal liquidity, exact/partial/exhausted independent
source capacity, technical minimum inputs, epoch advancement, independent
source cooldown delays, reward, loss, stake rent, metadata rent, status, and
every injected failure point. Controls contain no RPC, HTTP, address, random
system input, or wall-clock source.

## Mock formulas and rounding

For native pool total `T`, supply `S`, input `x`, and validated fee `n / d`:

```text
fee(x, n, d) = ceil(x * n / d)

deposit_gross(L) = floor(L * S / T)
deposit_fee       = fee(deposit_gross, deposit_fee_n, deposit_fee_d)
deposit_user_out  = deposit_gross - deposit_fee

withdrawal_fee(q) = fee(q, withdrawal_fee_n, withdrawal_fee_d)
burn(q)           = q - withdrawal_fee(q)
delegated(q)      = floor(burn(q) * T / S)
```

The bootstrap mock uses `deposit_gross(L) = L` only when `T == 0 && S == 0`.
Every multiplication is routed through the accepted checked `u128`
multiply/divide helpers, every result narrows explicitly to `u64`, every value
addition/subtraction is checked, outgoing conversions floor, and fees ceiling.
No economic operation uses wrapping or saturating arithmetic. SplitMix64 uses
wrapping operations only inside the existing test-only deterministic generator,
never for an economic result.

The withdrawal technical minimum is found by a bounded monotonic binary search
over `1..=S` for the first input whose fee-adjusted delegated output reaches the
configured native minimum. A source leg is exactly:

```text
input = min(remaining fixed target, current source capacity)
```

The mock rejects a zero/exhausted source, a below-minimum leg, or a source fill
that would strand a nonzero below-minimum remainder.

These formulas intentionally simulate the interface. They do not replace the
exact SPL Stake Pool 2.0.3 calculations and validations required in Phase 3.

## Atomicity and failure injection

Every fallible economic mutation follows clone, validate, commit:

1. validate the current state, request, identity, fee, slippage, capacity, and
   lifecycle;
2. derive quote and execution values;
3. clone the complete mock;
4. apply all tentative mutations to the clone;
5. evaluate injected failures at the relevant simulated boundaries;
6. validate full conservation and bounded state;
7. assign the clone to the live mock once.

Rejections therefore preserve snapshot, revision, all sources, all withdrawal
records, operational rent, audit counters, withdrawal count, and the configured
failure point. Tested failure points are snapshot read; deposit before
validation, after quote, and before commit; withdrawal before validation, after
quote, after pool debit, and before commit; status read; and finalization before
validation, after readiness, after accounting, and before commit.

## Conservation model

After every successful mock mutation, `validate_conservation` independently
checks:

```text
initial pool native + deposits + external rewards
  = current pool native + delegated native + external losses

initial supply + user deposit tokens + deposit fee tokens
  = current supply + burned withdrawal tokens

withdrawal input = withdrawal fee tokens + burned tokens

initial operational rent + recovered stake rent + recovered metadata rent
  = current operational rent + all rent advanced

initial source capacity = current source capacity + assigned input per source

sum(initiated record inputs, fees, burns, delegated value, and advanced rent)
  = the matching audit categories

for each finalized record:
  delegated native + recovered stake rent + cooldown reward
    = finalized native + cooldown loss

sum(finalized record native, reward/loss, and recovered rent categories)
  = the matching audit categories
```

It also checks occupied fixed records against the bounded withdrawal count.
Reward, loss, stake rent, metadata rent, delegated value, and finalized native
value remain explicit categories rather than being silently combined.

## Deterministic test matrix

`programs/piv1/tests/stake_pool_adapter.rs` is a Cargo-executed integration
test. Fixed tests cover:

- fresh, stale, future-epoch, impossible-total/supply, excess-liquidity, and
  invalid-fee snapshots;
- explicit zero-supply bootstrap and first deposit;
- independent exchange-rate increase and decrease directions;
- revision and epoch quote mismatch;
- zero, exact, rounded, zero-fee, nonzero-fee, exact-minimum, one-unit slippage,
  capacity, arithmetic-overflow, narrowing, and injected-failure deposits;
- exact technical minimum and one unit below;
- exact, partial, exhausted, subminimum, and stranded source capacities;
- exact and insufficient pool liquidity and operational rent;
- zero/nonzero withdrawal fees, fee-plus-burn identity, expected native output,
  exact slippage threshold, and one-unit failure;
- multiple independent sources, maximum-safe fills, and a two-leg minimum fill
  of a target that would require more smaller sources;
- identifier reuse and every withdrawal injection point;
- immediate and one-epoch-before finalization rejection, exact first eligible
  epoch, later epoch, independent readiness, out-of-order finalization,
  initiated/deactivating/inactive/finalized status, reward, loss, both rent
  categories, unknown identifier, replay, arithmetic failure, and every
  finalization injection point;
- full-state equality after every tested rejected mutation;
- conservation after every tested successful mutation.

The randomized test uses seed `0x5049563141445054` and exactly 1,024 cases. Each
case independently constructs a bounded pool, selects zero/nonzero deposit and
withdrawal fees, executes deposit, withdrawal, epoch advancement, and
finalization, compares fee/conversion results with direct independent `u128`
calculations, and validates complete conservation. Every failure message
contains the seed and case/action label. Fixed edge cases do not depend on the
random corpus.

## Future Task 1.3 transition-input mapping

This table is documentation only. Task 2.1 does not call or modify `PivConfig`,
`ActiveDistribution`, `WithdrawalLeg`, guardian rewards, or any Task 1.3
transition.

| Future transition input | Adapter-derived source | Additional trusted handler derivation |
|---|---|---|
| `OpenDistributionInput.prepared_epoch` | `PoolSnapshot.current_epoch` may be cross-checked | Authoritative Clock supplies preparation time/slot/epoch. |
| `snapshot_pool_total_lamports`, `snapshot_pool_token_supply`, withdrawal fee fraction | `PoolSnapshot` | Handler validates configured program/pool/accounts and stores the returned values. |
| `historical_value_lamports` | Checked official book-value calculation over adapter snapshot | Accounted historical units and SOL queue come from validated PIV1 custody/ledgers. |
| withdrawal target, snapshot leg floor, maximum useful legs, stored round minimum | Adapter snapshot/quotes and checked round math | Handler fixes the complete round target, split-call reserves, HWM proof, and pending-first funding. |
| `LegInitiationInput` pool/fee/minimum/capacity/input/fee/burn/expected/minimum fields | `StakeWithdrawalQuote` | Sequence/index and source identity are checked against the active round and decoded validator-list/account facts. |
| `observed_delegated_native_lamports`, initiation epoch, stake/metadata rent | `StakeWithdrawalInitiation` | Real handler uses exact validated post-CPI balance/stake deltas and Rent/account creation. |
| `LegFinalizationInput.finalized_epoch`, finalized native, rent, reward, loss | `StakeWithdrawalFinalization` | Handler derives stake inactivity from Stake state/Clock/Stake History and checks exact escrow/operational balance deltas. |
| `validated_residual_historical_value_lamports` | Fresh adapter snapshot plus checked official value calculation | Handler combines validated remaining custody with the stored HWM floor. |
| `escrow_available_after_lamports` | Not accepted as an adapter claim | Handler derives it from the fixed escrow account's observed balance after exact rent routing. |

Every external fact must ultimately come from validated accounts, sysvars,
observed balance deltas, or independently rederived adapter results. A future
handler must never copy a permissionless caller's claimed quote, fee, capacity,
status, epoch, rent, or output directly into a Task 1.3 input.

## Files changed

- `AGENTS.md`
- `README.md`
- `docs/PIV1_DECISIONS.md`
- `docs/PIV1_MASTER_SPEC.md`
- `docs/PIV1_CODEX_EXECUTION_PLAN.md`
- `docs/TASK_2_1_MOCK_STAKE_POOL_ADAPTER.md`
- `programs/piv1/README.md`
- `programs/piv1/src/instructions/pause.rs`
- `programs/piv1/src/integrations/mod.rs`
- `programs/piv1/src/integrations/stake_pool.rs`
- `programs/piv1/tests/stake_pool_adapter.rs`
- `programs/piv1/tests/support/mod.rs`
- `programs/piv1/tests/support/stake_pool_mock.rs`

No manifest, lockfile, Task 1.2 formula, Task 1.3 state layout, or Task 1.3
transition semantics changed. No dependency was added.

## Validation record

All required commands are run as `jerem` with explicit `HOME=/home/jerem`,
the explicit `/home/jerem/.cargo/bin/cargo` path, pinned Rust 1.97.1, locked
dependencies, and offline Cargo mode.

| Command or check | Result |
|---|---|
| `cargo +1.97.1 check -p piv1 --all-targets --locked --offline` | PASS |
| `cargo +1.97.1 test -p piv1 --all-targets --locked --offline` | PASS: 102 tests |
| `cargo +1.97.1 check --workspace --all-targets --locked --offline` | PASS |
| `cargo +1.97.1 check --workspace --all-targets --all-features --locked --offline` | PASS |
| `cargo +1.97.1 test --workspace --all-targets --locked --offline` | PASS: 137 tests |
| `cargo +1.97.1 test --workspace --doc --locked --offline` | PASS: 1 doctest |
| `RUSTDOCFLAGS="-D warnings" cargo +1.97.1 doc --workspace --no-deps --locked --offline` | PASS: no warnings |
| `cargo +1.97.1 test -p piv1 --test stake_pool_adapter --locked --offline -- --nocapture` | PASS: 24 tests, including 1,024 cases at seed `0x5049563141445054` |
| Compare accepted Task 1.2/1.3 math, constants, errors, state, layouts, and transitions against baseline | PASS: unchanged; only the authorized adapter replacement and pause comment changed in production source |
| Compare manifests, lockfiles, toolchain file, and TypeScript/Anchor configuration against baseline | PASS: unchanged; no dependency added |
| `git diff --check` and staged equivalent | PASS before staging; repeated after staging |
| Sensitive-path, private-key, credential, long numeric keypair-array, and generated-artifact scans | PASS for changed content and reachable history; no high-confidence material found in reported dangling blobs |
| Rust-source scan for `declare_id!`, `#[program]`, and `#[account]` | PASS |
| Adapter/mock scan for unsafe code, heap collection types, dynamic dispatch, wrapping arithmetic, and saturating arithmetic | PASS; SplitMix64 wrapping remains isolated to the test generator |
| `git fsck --full --strict` | PASS with no corruption; Git reported 12 unreachable dangling blobs |

`rustup component list --installed --toolchain 1.97.1` reported `cargo`,
`rust-std`, and `rustc`; `rustfmt` and `clippy` are not installed for the
accepted toolchain. They were not installed and the toolchain was not changed.
No Anchor command was run.

## Safety result and exact next action

Task 2.1 created no Program ID, wallet, keypair, seed phrase, or live address;
ran no Anchor command or local validator; contacted no RPC; deployed no
program; sent no transaction; moved no funds; and changed no authority. It
performed no Mainnet action. The host mock cannot move funds and is not exposed
by the production crate.

The exact next action is founder review of Task 2.1. Do not begin Task 2.2,
Phase 3 mapping, founder acceptance, publication, merge, tag, or deployment.
