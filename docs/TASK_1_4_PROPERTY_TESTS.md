# Task 1.4 randomized property and adversarial invariant tests

Date: 2026-09-02 UTC

Branch: `task/1.4-property-invariant-tests`

Accepted starting baseline: `8a512656fc78eff17d2473e6fc37a08e4b77db4d`

Status: **IMPLEMENTED / PENDING FOUNDER ACCEPTANCE**

## Scope and result

Task 1.4 adds reproducible host-side randomized/property, adversarial
state-machine, and serialization/layout tests for the founder-accepted Task 1.2
math and Task 1.3 pure state-transition model.

The completed suite found no production math, transition, economic, layout, or
serialization defect. It changed no production Rust source, account field,
formula, percentage, rounding rule, custody boundary, pause rule, or lifecycle
boundary. Phase 2 has not started.

## Framework choice and dependency result

No Rust property-testing framework was present in `Cargo.lock` or in the
offline Cargo source cache at the accepted baseline. Adding `proptest` or a
similar framework would have introduced a new test-only dependency graph solely
for Task 1.4. The task authorization explicitly permits a small deterministic
seeded generator when a framework would create unnecessary offline or supply-
chain work.

The tests therefore use a local test-only SplitMix64 generator. SplitMix64 is
used only to enumerate deterministic inputs and action choices; it is not
production randomness. Every randomized test has a fixed seed and bounded case
count. Failure messages include the seed and case/action index. Re-running the
named Cargo test command reproduces the exact sequence. There is no wall-clock
input, unbounded fuzzer, system-package installation, runtime dependency, or
lockfile change.

This approach does not provide automatic shrinking. A failure is reduced by
re-running its exact seed/case and preserving a deterministic minimal case when
it identifies a real defect. The fixed boundary corpus runs before or alongside
the randomized corpus so critical edges do not depend on generator luck.

## Deterministic configuration

The exact top-level seeds are:

- multiply/divide: `0x504956314d554c44`;
- general math accounting: `0x5049563141434354`;
- state model: `0x5049563153544154`;
- serialization: `0x5049563153455244`;
- adversarial transitions: `0x5049563141445652`;
- state-level KIF: `0x1b00103153544154`;
- active-pending/snapshot cases: `0x000c18751a1a0654`;
- insufficient-attempt cases: `0x1907056407025652`.

The 192 state-machine seeds are derived deterministically as:

```text
0x5049563153544154 XOR (seed_index * 0x9e3779b9)
for seed_index in 0..192
```

The model executes 128 bounded action choices for each seed, for exactly 24,576
action attempts. A failure message prints the fully derived seed, seed index,
action index, and selector.

## Boundary corpus

The shared math corpus contains:

```text
0, 1, 2, 3, 5, 6, 49, 50, 9_999, 10_000, 10_001,
u32::MAX, u32::MAX + 1, u64::MAX / 2,
u64::MAX / 2 + 1, u64::MAX - 1, u64::MAX
```

Multiply/divide tests use the full Cartesian product of that corpus for both
factors and the denominator: 4,913 fixed triples. Additional fixed cases cover
zero denominators, denominator one, exact division, public narrowing failure,
and the ceiling-only narrowing boundary.

State boundaries include zero/one/full amounts, exact and one-before cadence
boundaries, timestamp regression, carry below/equal/above an HWM deficit,
zero/partial/exact/excessive target fills, maximum `u64` KIF carry/reward
overflow, escrow shortage/surplus, all 64 guardian bitmaps, active counts zero
through six, invalid active counts, and dust-producing allocation values.

## Pure-math property groups and counts

`crates/piv1-math/tests/property_tests.rs` implements:

1. Checked multiply/divide: 4,913 boundary triples plus 25,000 randomized
   triples. An independent direct `u128` product/quotient/remainder oracle
   checks floor, ceiling, exact division, at-most-one separation, rational
   bounds, zero-denominator rejection, and checked narrowing.
2. Gross yield, fixed split, and HWM: 17 fixed split values plus 25,000
   randomized iterations. Each iteration independently checks gross yield,
   every `5900/1950/1950/200` floor, explicit dust, outgoing bounds, all six
   HWM components, monotonic updates, and overflow rejection.
3. KIF: 119 fixed `(available, active_count)` cases, 25,000 randomized cases,
   and 65 successive zero-active periods from `u64::MAX`. The oracle checks
   counts zero through six, invalid-count precedence, equal active credits,
   collective remainder, zero-active floor/residual, repeated carry, exact
   conservation, and addition overflow.

The math oracle does not call the production split or allocation method to
derive expected fractions. It uses direct independent `u128` rational
calculations.

## State and adversarial property groups and counts

`programs/piv1/tests/property_invariants.rs` implements:

- 4,096 carry/HWM cases plus fixed below/equal/above-deficit cases;
- 4,096 full state-level KIF settlement cases over random guardian bitmaps,
  gross yield, and prior carry;
- 1,024 randomized valid serialization cases for all five schemas, including
  512 settled headers and 512 initiated withdrawal legs;
- 4,096 randomized mutation selections for each of `PivConfig`,
  `ActiveDistribution`, `WithdrawalLeg`, `GuardianRegistry`, and
  `GuardianReward`, totaling 20,480 malformed decoded-schema validations;
- 24,576 reference-model action attempts across 192 seeds;
- 1,024 active-distribution late-contribution and guardian-rotation cases;
- 4,096 valid/invalid technical-insufficiency cases;
- 4,096 adversarial leg-initiation cases and 4,096 adversarial leg-finalization
  cases;
- deterministic escrow shortage/surplus, recovery, pause, overflow, replay,
  layout, future-claim marker, and accepted 1,800-lamport settlement cases.

The successful model run recorded:

- 271 distribution opens;
- 193 successful leg initiations;
- 150 successful leg finalizations, including 7 out of order;
- 94 successful settlements;
- 84 successful pending integrations/completions;
- 112 recovery-required entries;
- 21,351 rejected action attempts that preserved every supplied state object;
- paused rejections in all seven modeled economic action categories.

## Independent model action set

The small reference model tracks only independently necessary lifecycle facts:

- idle, withdrawal-active, escrow-funded, settled, and recovery-required phase;
- next and active sequence;
- fixed target and cumulative assignment;
- successful/finalized counters;
- four bounded test-leg statuses; and
- the immutable active snapshot and last observed HWM.

Randomized actions attempt liquid and withdrawal opening, maximum-safe leg
initiation, arbitrary-leg finalization, settlement, pending integration,
wrong-sequence settlement, pause/unpause, residual-HWM recovery, valid
insufficiency, no-yield evaluation, wrong-index/replayed initiation, and
replayed/out-of-order finalization. The reference model determines whether the
ordering is legal without using production validation as its oracle.

Every successful action is followed by validation of all affected schemas and
independent sequence, target, count, fee-plus-burn, snapshot, conservation, and
HWM checks. Every rejected action compares the complete config, header, all six
rewards, and all four supplied test legs against their pre-call copies.

## Invariants exercised

The combined new and retained Phase 1 suite attempts to falsify all requested
Task 1.4 invariants:

- only one active distribution; monotonic non-reusable sequences;
- valid state after every success and full atomic preservation after errors;
- no double settlement, completion, leg initiation, or leg finalization;
- exact sequence/index binding and wrong-sequence/index rejection;
- cumulative assignment at or below the target and exact target before
  settlement;
- fee plus burn equals input; cumulative successful/finalized counters remain
  consistent;
- randomized out-of-order finalization and finalization before later target
  assignment;
- settlement is blocked before exact assignment and complete finalization;
- immutable economic, recipient, guardian, KIF, and protocol snapshots;
- late contributions do not change fixed obligations and integrate only at the
  accepted completion boundary;
- HWM monotonicity, loss recovery exclusion, carry-before-HWM treatment, and
  one-time carry consumption;
- deterministic caller-independent net beneficiary allocation, including the
  exact `1,319 / 436 / 44 / 1` result for 1,800 lamports;
- net-allocation dust enters protected HWM accounting exactly once;
- paid, credited, carried, retained, compounded, and escrow remainder values
  reconcile without exceeding availability;
- post-preparation guardian rotation cannot rewrite eligibility or keys;
- pause rejects every modeled Task 1.3 distribution-economic category and
  preserves all state;
- the future `claim_kif` type remains a zero-sized marker with no implemented
  transition; its pause policy remains `OPEN`;
- recovery-required state cannot resume through normal transitions;
- timestamp regressions, one-before cadence, and exact cadence boundaries;
- valid technical insufficiency changes only its timestamp, while invalid or
  malformed attempts change neither timing gate;
- malformed lifecycle, counter, flag, cumulative, KIF, address, split,
  slippage, timing, liability, escrow, and HWM fields fail validation; and
- state-level KIF and reward-credit overflow paths reject atomically.

## Serialization and layout protection

The tests bind the accepted sizes and planned discriminator-inclusive spaces:

| Schema | Payload | Planned space |
|---|---:|---:|
| `PivConfig` | 1,006 | 1,014 |
| `ActiveDistribution` | 883 | 891 |
| `WithdrawalLeg` | 255 | 263 |
| `GuardianRegistry` | 202 | 210 |
| `GuardianReward` | 76 | 84 |

Randomized valid objects serialize, deserialize, compare field-for-field, and
pass their schema validation. One accounting or invariant field at a time is
then corrupted, serialized, decoded, and required to fail validation. Task 1.4
adds no field to any accepted account layout.

## Counterexamples and regressions

No production counterexample was found and no Task 1.2 or Task 1.3 production
code was changed.

During test development, state seed `0x5049563153544154`, case 25 exposed a
test-oracle error: the initial property equated gross KIF with post-cost net
KIF. Independent relative-weight flooring can instead retain one lamport as
net-allocation dust. The oracle was corrected to derive the capped `200/8050`
net share independently. The fixed seed remains in the permanent suite, and
the explicit 1,800-lamport regression separately binds the accepted
`1,319 / 436 / 44 / 1` allocation.

An initial model coverage guard observed four out-of-order finalizations where
the configured minimum was five. Coverage was expanded from 128 to 192 seeds
rather than lowering the guard; the final deterministic run records seven.
This was a corpus-size adjustment, not a production failure.

## Validation record

All Rust commands ran as `jerem` with explicit `HOME=/home/jerem`, explicit
`PATH=/home/jerem/.cargo/bin:/usr/local/bin:/usr/bin:/bin`, Rust 1.97.1, locked
dependencies, and offline Cargo mode.

```text
cargo +1.97.1 check -p piv1-math --all-targets --locked --offline
PASS

cargo +1.97.1 check -p piv1 --all-targets --locked --offline
PASS

cargo +1.97.1 test -p piv1-math --all-targets --locked --offline
PASS — 35 tests: 32 accepted unit tests plus 3 property tests

cargo +1.97.1 test -p piv1 --all-targets --locked --offline
PASS — 78 tests: 35 unit, 20 illegal/replay, 11 legal-lifecycle, 12 property/adversarial

cargo +1.97.1 check --workspace --all-targets --locked --offline
PASS

cargo +1.97.1 check --workspace --all-targets --all-features --locked --offline
PASS

cargo +1.97.1 test --workspace --all-targets --locked --offline
PASS — 113 tests

cargo +1.97.1 test --workspace --doc --locked --offline
PASS — one piv1-math doctest; zero piv1 doctests

RUSTDOCFLAGS="-D warnings" cargo +1.97.1 doc --workspace --no-deps --locked --offline
PASS

cargo +1.97.1 test -p piv1-math --test property_tests --locked --offline -- --nocapture
PASS — 3 property tests; 0.32 seconds including Cargo startup

cargo +1.97.1 test -p piv1 --test property_invariants --locked --offline -- --nocapture
PASS — 12 property/adversarial tests; 1.42 seconds including Cargo startup
```

The combined four package check/test commands completed in 2.60 seconds. The
combined workspace check, all-features check, full test, doctest, and
warning-denied documentation commands completed in 4.83 seconds. Times are
warm-cache wall-clock measurements from `/usr/bin/time`; they are evidence of
bounded normal-CI execution, not performance guarantees.

```text
git diff --check
git diff --cached --check
PASS

byte-for-byte SHA-256 comparison of every tracked Cargo/npm manifest and
lockfile against 8a512656fc78eff17d2473e6fc37a08e4b77db4d
PASS — all 11 files unchanged

current tracked/new-task and reachable-history sensitive-path scan
PASS

current-index and reachable-history high-confidence secret-marker scan
PASS

git fsck --full --strict
PASS — no corrupt or missing object; informational dangling blobs only
```

The unchanged root production identities are:

- `Cargo.toml`: `1c5cf8fa2d0d79021086a76ab8ef9f71980bda9ddf46a06a439b2c549ecca6`;
- `Cargo.lock`: `e713f52b51cb9b708e6d95d8f86e1e195ea1acc597ff156f79b8a91bbcabd751`;
- `crates/piv1-math/Cargo.toml`: `aa33a006040ec299bc9c073b9840115c5269f431337e12878beb873c1c8196e3`;
- `programs/piv1/Cargo.toml`: `8a04f84bf9eb260e42a0b003f59e9bb10239f46cfb2304e33430e049c11777c6`.

The pinned toolchain has `cargo`, `rustc`, and `rust-std` only. `rustfmt` and
`clippy` are not installed; neither was installed or used.

## Deferred external/CPI invariants

Task 1.4 does not and cannot validate future handler-derived facts involving:

- PDA derivation under a real PIV1 Program ID or account ownership;
- real Clock, epoch, Stake History, stake readiness, validator-list decoding,
  source order, residual capacity, or Rent sysvars;
- official Jito/SPL program, pool, mint, fee, conversion, slippage, and exact
  CPI delta checks;
- native/SPL transfers, escrow balances, account closure, or rent movement;
- governance signatures, recipient accounts, guardian keys, real claims, or
  the still-open `claim_kif` pause policy;
- RPC, validator, local adapter, Testnet, Mainnet, compute-unit, or transaction-
  size behavior.

These remain later separately authorized implementation/integration work. The
tests treat transition inputs as facts that a future handler must first derive
and validate from trusted accounts and sysvars.

## Safety and exact next action

Task 1.4 created no key or Program ID, inspected no ignored Task 0.4 Testnet key,
ran no `anchor build`, deployed no program, contacted no RPC or validator, sent
no transaction, moved no funds, and changed no authority. It performed no
Mainnet action.

The exact next action is founder architecture review of Task 1.4 and Phase 1.
Do not begin Phase 2 without separate founder authorization.
