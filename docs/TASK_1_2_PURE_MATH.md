# PIV1 Task 1.2 pure math implementation record

Date: 2026-08-30 UTC

Branch: `task/1.2-pure-math-crate`

Starting baseline: `e6d04530ccfa65ca3a204fcfcb15d37033317654`

Accepted implementation commit: `43a3b7497653ff7a246a1e5cf9b760086dd33fcd`

Status: COMPLETE / FOUNDER-ACCEPTED

## Founder acceptance

The implementation branch was published for independent founder review. The
founder accepted the exact implementation commit above after finding no code,
arithmetic, invariant, test, scope, or security defect requiring a source
correction. The acceptance record changes documentation only.

## Scope

Task 1.2 implements only the dependency-free, host-testable integer arithmetic
foundation in `crates/piv1-math`:

- checked multiply/divide with floor and ceiling;
- the confirmed fixed gross-yield split;
- gross yield against the high-water mark;
- checked high-water-mark increases;
- KIF allocation for one through six active guardians;
- KIF allocation for zero active guardians; and
- deterministic unit and boundary tests.

The implementation consumes pure numeric inputs already validated and
reconciled by later integration and state-transition work. It does not decide
when a value becomes eligible for accounting.

## Public API

The crate exposes:

- `Amount = u64`;
- fixed basis-point and guardian-count constants;
- `MathError`;
- `GrossYieldSplit`;
- `HighWaterMarkComponents` and `HighWaterMarkUpdate`;
- `ActiveGuardianKifAllocation`, `ZeroActiveGuardianKifAllocation`, and
  `KifAllocation`;
- `checked_mul_div_floor`;
- `checked_mul_div_ceil`;
- `calculate_gross_yield`;
- `split_gross_yield`;
- `checked_increase_high_water_mark`; and
- `allocate_kif`.

Explicit structures and the active/zero-active KIF enum prevent ambiguous
tuple positions and invalid combinations of mutually exclusive KIF outputs.

## Integer and intermediate types

Public economic values use `u64` because Solana lamports and legacy SPL token
base-unit amounts are represented as unsigned 64-bit quantities. Guardian
counts use `u8` and are validated against the confirmed maximum of six.

Multiply/divide operations convert both `u64` factors to `u128`, perform a
checked `u128` multiplication, divide in `u128`, and use a checked conversion
back to `u64`. Two `u64` factors mathematically cannot overflow `u128`, but the
private wide kernel still returns an explicit multiplication error and is
directly boundary-tested. Public results can exceed `u64` after division, so
failed narrowing is a reachable and separately tested error.

All stored additions, multiplications, residual subtractions, and narrowing
conversions are checked. No floating-point arithmetic, unchecked cast,
wrapping arithmetic, `unsafe`, production `unwrap`, production `expect`, or
production panic is used.

## Error model

`MathError` deliberately has no Anchor error numbers. It distinguishes:

- `DivisionByZero`;
- `AdditionOverflow`;
- `MultiplicationOverflow`;
- `SubtractionUnderflow`;
- `NarrowingConversion`; and
- `InvalidActiveGuardianCount { active_guardians }`.

For KIF, the active count is validated before the available-amount addition,
giving invalid input deterministic precedence over an otherwise overflowing
amount pair.

## Confirmed formulas and rounding

### Checked floor and ceiling

```text
floor_result = floor(multiplicand * multiplier / denominator)
ceil_result  = ceil(multiplicand * multiplier / denominator)
```

Ceiling uses quotient plus a nonzero-remainder increment. It does not use
`product + denominator - 1`, avoiding an unnecessary overflow surface. A zero
denominator is rejected even when either factor is zero.

### Gross-yield split

```text
denominator = 10_000

htfp_reserve       = floor(Y * 5_900 / 10_000)
permanent_compound = floor(Y * 1_950 / 10_000)
team_owner_pool    = floor(Y * 1_950 / 10_000)
kif                = floor(Y *   200 / 10_000)
dust               = Y
                     - htfp_reserve
                     - permanent_compound
                     - team_owner_pool
                     - kif
```

Every named share is floored independently. Dust is the checked exact
residual, remains protected inside PIV1, and is not assigned to a beneficiary.

### Yield against the high-water mark

```text
gross_yield = max(0, historical_value - high_water_mark)
```

Historical recovery at or below the high-water mark returns zero yield. This
calculation does not modify or lower the high-water mark.

### Checked high-water-mark increase

```text
increase = contribution_value
         + normal_compound_allocation
         + split_dust
         + conversion_dust
         + net_allocation_dust
         + zero_active_kif_compound

new_high_water_mark = old_high_water_mark + increase
```

Every addition is checked. The result returns both `increase` and
`new_high_water_mark` so later callers can reconcile their exact relationship.
The function does not determine component eligibility.

### KIF with active guardians

For an already snapshotted count in `1..=6`:

```text
kif_available = current_kif_allocation + approved_prior_carry
per_guardian   = floor(kif_available / active_guardians)
credited_total = per_guardian * active_guardians
carry_next     = kif_available - credited_total
```

Every active guardian receives the same amount. The division remainder stays
collective carry and is never assigned preferentially.

### KIF with zero active guardians

```text
kif_available    = current_kif_allocation + approved_prior_carry
compound_from_kif = floor(kif_available / 2)
carry_next        = kif_available - compound_from_kif
```

The complete prior carry is supplied again in every successive zero-active
period. For an odd amount, the extra unit remains in `carry_next`. No inactive
guardian claim is represented by the result.

## Deterministic tests and invariants

The Rust unit suite covers:

- zero factors, denominator one, exact and non-exact floor/ceiling, zero
  denominator, maximum values, true wide multiplication overflow, public
  narrowing, and a ceiling-only narrowing boundary;
- zero, one lamport, allocation thresholds, exact 10,000-lamport weights,
  non-divisible values, and `u64::MAX` fixed splits;
- component bounds, beneficiary-outgoing bounds, and exact split-plus-dust
  reconciliation;
- historical value below, equal to, one above, normally above, and at the
  boundaries of the high-water mark;
- all-zero, each independently nonzero, combined, maximum-valid, and
  overflowing high-water-mark increases;
- zero KIF availability, even and odd zero-active splits, repeated full-carry
  behavior, maximum odd availability, all active counts one through six,
  exact and non-divisible active allocations, invalid counts, and prior-carry
  addition overflow; and
- exact `credited_total + carry_next == kif_available` and
  `compound_from_kif + carry_next == kif_available` reconciliation.

Task 1.4 remains reserved for randomized/property testing. Task 1.2 adds no
randomized or fuzzing dependency.

## Validation

All commands were run as `jerem` with `HOME=/home/jerem`, the accepted pinned
Rust 1.97.1 toolchain, existing lockfiles, and offline Cargo mode.

```text
cargo +1.97.1 check -p piv1-math --all-targets --locked --offline
PASS

cargo +1.97.1 test -p piv1-math --all-targets --locked --offline
PASS — 32 unit tests

cargo +1.97.1 test -p piv1-math --doc --locked --offline
PASS — 1 documentation test

cargo +1.97.1 check --workspace --all-targets --locked --offline
PASS

cargo +1.97.1 check --workspace --all-targets --all-features --locked --offline
PASS

cargo +1.97.1 test --workspace --all-targets --locked --offline
PASS

RUSTDOCFLAGS="-D warnings" cargo +1.97.1 doc -p piv1-math \
  --no-deps --document-private-items --locked --offline
PASS

git diff --check
PASS

git fsck --full --strict
PASS — no integrity error; 11 unreachable dangling blobs reported and left
untouched

reachable tracked secret-path and private-key/credential-marker checks
PASS
```

An initial secret-scan wrapper invocation was discarded as invalid because the
non-login `jerem` shell did not expose `rg` and the Git pattern required an
explicit `-e`. The corrected scan used available explicit tool paths, checked
every exit status strictly, passed, and searched only current tracked/task files
and objects reachable from Git refs. Neither invocation inspected ignored
working-tree artifacts.

An independent read-only final review reported no implementation,
arithmetic-safety, test-coverage, scope, or documentation finding.

The installed pinned toolchain contains `cargo`, `rustc`, and the host standard
library only. `rustfmt` and `clippy` were not installed, so their checks were
not run and no component was installed or upgraded for this task.

No test used RPC, a wallet, a keypair, a validator, network access, or funds.
`anchor build` was deliberately not run because it is outside Task 1.2 and the
accepted Task 1.1 investigation showed that it creates an unwanted ignored key
artifact in this compile-only workspace.

## Dependency and lockfile result

No dependency was added. `crates/piv1-math/Cargo.toml`, the root workspace
manifest, `Cargo.lock`, and JavaScript manifests/lockfiles are unchanged.

## Explicit exclusions and deferrals

Task 1.2 does not implement:

- Solana accounts, Clock use, Anchor code, state transitions, serialization,
  instruction logic, CPI, or network/RPC code;
- Jito/SPL state decoding, exchange conversion, deposit/withdrawal fees,
  dynamic minimums, slippage, validator or withdrawal-leg calculations;
- pending-SOL-first budgeting, cooldown reward/loss reconciliation, transfers,
  claims, guardian keys, heartbeat/period eligibility, or client code;
- Program ID, deployment, transaction scripts, TypeScript tests, or changes to
  the root TypeScript configuration; or
- randomized/property tests reserved for Task 1.4.

The master specification's broad Phase 1 pure-math test list mentions protocol
fee allocation. The newer, task-specific founder authorization explicitly
excludes withdrawal/deposit fees from Task 1.2, so fee formulas are deliberately
deferred without changing their confirmed later accounting treatment.

Task 1.3 and Task 1.4 have not started. The exact next task is Task 1.3 — the
state and transition model — and it requires separate founder authorization
and a dedicated branch.

## Safety statement

Task 1.2 created no key or Program ID, read or modified no preserved ignored
key artifact, deployed no program, sent no transaction, moved no funds, changed
no authority, and performed no Mainnet action. The implementation branch was
published only for founder review and is now founder-accepted.
