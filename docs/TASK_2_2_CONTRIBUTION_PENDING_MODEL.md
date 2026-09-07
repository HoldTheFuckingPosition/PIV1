# Task 2.2 contribution intake and pending-queue accounting model

Date: 2026-09-07 UTC

Branch: `task/2.2-contribution-pending-model`

Accepted starting baseline: `a4cf7cdcabee7965e81525894627e8f5fc138e03`

Accepted implementation: `e3233b96b533a620e8037d5231baede10877217f`

Status: **COMPLETE / FOUNDER-ACCEPTED**

Phase 2 status: **IN PROGRESS**

Task 2.3 and later Phase 2 work: **NOT STARTED**

## Founder acceptance

The founder accepts the pure explicit SOL/JitoSOL contribution accounting,
dedicated pending-vault reconciliation, checked arithmetic and custody-deficit
rejection, reconciliation idempotence, atomic updates, preservation of the
complete active distribution and non-pending accounting, host-only custody
mock, and recorded deterministic test evidence.

This acceptance does not promote observations to validated account facts.
Future handlers must still perform fixed-account and transfer validation;
explicit-transfer handler callability during pause remains **PROVISIONAL**; no
real custody, handler, CPI, or localnet behavior is proven; and all-vault
normalization remains deferred. Future composition tests must cover pending SOL
moved into distribution escrow, distinguish remaining physical pending custody
from contribution value awaiting HWM integration, and prevent both double
counting and false custody deficits. No new accounting policy for those
deferred cases is selected by this acceptance.

## Scope and result

Task 2.2 adds a narrow pure production seam for explicit SOL and JitoSOL
pending-contribution accounting and idempotent reconciliation of unexplained
positive balances in the two dedicated pending vaults. It also adds a
fixed-size host-only custody mock and deterministic fixed, boundary,
failure-injection, distribution-phase, pause, and randomized tests.

The result preserves the confirmed economic boundary:

- no contributor identity, balance, share, reward, ownership, or withdrawal
  right is stored;
- explicit and untracked incoming value becomes pending principal, never yield;
- intake and reconciliation do not change historical SOL, historical JitoSOL,
  HWM, cumulative contribution value, active-round economics, or KIF
  liabilities;
- pending JitoSOL is not valued or integrated by this task, so all appreciation
  before the later accepted integration boundary remains conservatively within
  contribution/principal value;
- no operation spends, stakes, integrates, distributes, settles, claims, or
  changes a recipient; and
- the existing five serialized layouts and planned spaces are unchanged.

This task does not implement an Anchor handler, `Accounts` context, System or
Token Program transfer, CPI, event, Program ID, IDL, deployment artifact,
local validator, RPC action, client, or live custody behavior.

## Production-facing observation boundary

`state::contributions` defines bounded scalar observations and three pure
operations:

| Operation | Effect |
| --- | --- |
| `record_explicit_sol_contribution` | Derives an exact spendable-lamport increase and advances only `accounted_pending_sol_lamports`. |
| `record_explicit_jitosol_contribution` | Derives an exact decoded token-unit increase and advances only `accounted_pending_jitosol_units`. |
| `reconcile_pending_contributions` | Atomically advances both pending ledgers by their unexplained positive physical deltas. |

The observation structs are deliberately not named or documented as validated
caller input. Future handlers must derive them from fixed program-controlled
accounts after all required account checks. Passing a number to the pure layer
does not prove custody, address, owner, mint, authority, data, or rent facts.

Every operation validates the existing `PivConfig` and complete
`ActiveDistribution`. The distribution is accepted by immutable reference and
cannot be changed by the API. Each config update is staged in a full clone,
validated, and committed once.

## Physical versus accounted balances

The model distinguishes four quantities:

1. total physical lamports in `PendingSolVault`;
2. a handler-validated non-economic SOL floor;
3. decoded physical JitoSOL token units in `PendingJitoVault`; and
4. the two already-accounted pending ledgers in `PivConfig`.

For native SOL:

```text
spendable_sol = physical_pending_sol_vault_lamports
              - validated_non_economic_floor_lamports
```

The subtraction is checked. The floor is not hard-coded and cannot become
contribution value. For JitoSOL, only decoded token units participate; lamports
funding the token account are not an asset amount in this model.

An unexplained surplus is the checked difference between current physical
economic balance and accounted pending balance. A physical balance below the
accounted ledger is a custody deficit and is rejected rather than hidden by an
economic reclassification.

## Explicit SOL contribution

For a positive expected amount `E`, total before/after balances `B0` and `B1`,
and one validated non-economic floor `F`:

```text
S0 = checked_sub(B0, F)
S1 = checked_sub(B1, F)
observed = checked_sub(S1, S0)
require observed == E
pending_sol_after = checked_add(pending_sol_before, observed)
require pending_sol_before <= S0
require pending_sol_after <= S1
```

Zero expected amounts, an invalid floor, a decreasing balance, a mismatched
increase, checked overflow, an invalid config, or an invalid distribution are
rejected without mutation. A successful record changes only
`accounted_pending_sol_lamports`.

The before/after model assumes the handler has established that one unchanged
validated non-economic floor applies to the atomic observation window. Exact
real rent/account mechanics remain deferred.

## Explicit JitoSOL contribution

For positive expected token units `E` and decoded before/after token amounts
`Q0` and `Q1`:

```text
observed = checked_sub(Q1, Q0)
require observed == E
pending_jitosol_after = checked_add(pending_jitosol_before, observed)
require pending_jitosol_before <= Q0
require pending_jitosol_after <= Q1
```

Zero, decrease, mismatch, checked overflow, and malformed state reject
atomically. A successful record changes only
`accounted_pending_jitosol_units`. No token-account lamport balance is accepted
by this API.

## Direct-transfer reconciliation

One combined operation reconciles both dedicated pending vaults atomically:

```text
current_sol = checked_sub(physical_sol, non_economic_floor)
sol_delta = checked_sub(current_sol, accounted_pending_sol)
jitosol_delta = checked_sub(current_jitosol_units,
                            accounted_pending_jitosol_units)

accounted_pending_sol += sol_delta
accounted_pending_jitosol += jitosol_delta
```

If either current physical balance is below its ledger, neither ledger changes.
If both deltas are zero, the function returns a deterministic no-change result.
Repeating an unchanged observation therefore cannot double-count. A positive
delta is classified only as pending contribution; it never reaches historical
value, yield, HWM, a distribution obligation, or cumulative contribution value
in Task 2.2.

## Active-distribution isolation

Intake and reconciliation were exercised against these valid stored/derived
phases:

- Idle;
- PreparedWithdrawal;
- AssigningWithdrawalLegs;
- PartiallyFinalized;
- WithdrawalTargetAssigned / AwaitingLegInactivity;
- EscrowFunded;
- Settled; and
- RecoveryRequired.

In every case the complete Borsh encoding of `ActiveDistribution` remained
byte-for-byte identical. Consequently no fixed target, cumulative assignment,
finalized amount, beneficiary obligation, leg counter, escrow amount, KIF
snapshot, settlement value, HWM proof, recovery flag, or terminal summary can
be changed or satisfied by Task 2.2 intake.

## Pause behavior

Reconciliation is intentionally not gated by `ensure_unpaused`. Direct incoming
transfers cannot be prevented, and recording their already-present positive
balances as pending neither spends nor integrates value. The tests run explicit
accounting and reconciliation in both paused and unpaused configurations while
requiring every non-pending field and the full active distribution to remain
unchanged.

Whether a future explicit `deposit_sol` or `deposit_jitosol` transfer handler
may itself be invoked during pause is **PROVISIONAL** because the canonical
documents do not conclusively settle that handler policy. Task 2.2's pure
explicit record functions are allowed during pause only as safe accounting of
an exact already-observed incoming delta. They do not authorize a transfer
handler, staking, pending integration, HWM change, settlement, KIF claim, or
recipient change during pause.

## Atomicity and replay behavior

Production operations validate first, derive all checked results, stage a full
`PivConfig` clone, validate the candidate config, and commit once.
`ActiveDistribution` is immutable input. Every rejected production operation
therefore preserves both objects.

The host mock clones its complete custody world before each fallible action.
This additionally preserves physical balances, excluded SOL, both ledgers,
audit totals, and persistent failure-injection configuration on rejection.
Reconciliation replay is safe because the second delta is exactly zero.

Production code introduces no `unsafe`, panic path, `unwrap`, or `expect`.

## Host-only custody mock

`tests/support/contribution_custody_mock.rs` contains a fixed-size scalar model
that is not declared or exported by the production library. It stores:

- total physical pending-vault SOL;
- excluded non-economic SOL;
- physical pending JitoSOL token units;
- a complete `PivConfig` and `ActiveDistribution`;
- fixed audit totals for explicit credits, direct credits, reconciled direct
  credits, and forbidden outgoing movement;
- fixed initial accounting baselines; and
- one optional persistent deterministic failure point.

The mock has no vector, map, network source, wall clock, address derivation,
System/Token Program behavior, PDA/ATA rule, CPI, or stake-pool behavior. It is
evidence for the pure accounting contract only.

After every successful action, conservation requires:

```text
physical SOL spendable
  = initial accounted SOL + explicit SOL + direct SOL

accounted pending SOL
  = initial accounted SOL + explicit SOL + reconciled direct SOL

direct SOL
  = reconciled direct SOL + unexplained SOL
```

The same three equations apply independently to JitoSOL token units. Both mock
outgoing counters must remain zero.

## Errors

Task 2.2 adds these bounded `Piv1Error` categories:

- `ZeroContribution`;
- `InvalidCustodyObservation`;
- `CustodyBalanceDecreased`;
- `ContributionObservationMismatch`; and
- `PendingCustodyDeficit`.

Existing `ArithmeticOverflow` covers checked addition failure. The host mock
uses a separate test-only `InjectedFailure` category and does not expand the
production error model for simulation controls.

## Deterministic tests

`tests/contribution_pending.rs` contains nine top-level tests covering:

- zero SOL and JitoSOL rejection;
- exact explicit SOL and JitoSOL intake;
- mismatched, decreasing, and malformed before/after observations;
- excluded SOL floor behavior;
- checked subtraction/addition and `u64::MAX` boundaries;
- independent and simultaneous direct-transfer deltas;
- zero-delta and repeated idempotent reconciliation;
- SOL and JitoSOL custody deficits;
- complete mock equality after mismatch, overflow, and all five injected
  failures;
- all eight relevant distribution phases, both paused and unpaused;
- byte-for-byte active-distribution preservation;
- unchanged historical, HWM, cumulative contribution, distribution, and KIF
  fields;
- zero outgoing mock movement; and
- physical/accounted/unexplained conservation after every successful action.

The randomized test uses local test-only SplitMix64 with seed
`0x50495631434f4e54`. It runs exactly 1,024 cases with 64 bounded actions each,
for 65,536 action attempts. Every randomized assertion reports seed, case,
action, selector, and distribution phase. Fixed boundary cases do not depend on
generator coverage. SplitMix64 wrapping arithmetic is test-only and never used
for an economic result.

At final Task 2.2 validation:

- targeted Task 2.2: 9 tests passed;
- targeted accepted Task 2.1 adapter: 24 tests passed, including its 1,024
  randomized cases at seed `0x5049563141445054`;
- complete `piv1` all-target suite: 111 tests passed;
- complete workspace all-target suite: 146 tests passed;
- doctests: one `piv1-math` doctest passed and zero `piv1` doctests existed;
- warning-denied workspace documentation generation passed; and
- every fixed and randomized conservation assertion passed.

The Cargo validation commands were:

```text
cargo +1.97.1 check -p piv1-math --all-targets --locked --offline
cargo +1.97.1 check -p piv1 --all-targets --locked --offline
cargo +1.97.1 test -p piv1-math --all-targets --locked --offline
cargo +1.97.1 test -p piv1 --all-targets --locked --offline
cargo +1.97.1 check --workspace --all-targets --locked --offline
cargo +1.97.1 check --workspace --all-targets --all-features --locked --offline
cargo +1.97.1 test --workspace --all-targets --locked --offline
cargo +1.97.1 test --workspace --doc --locked --offline
RUSTDOCFLAGS="-D warnings" cargo +1.97.1 doc --workspace --no-deps --locked --offline
cargo +1.97.1 test -p piv1 --test contribution_pending --locked --offline -- --nocapture
cargo +1.97.1 test -p piv1 --test stake_pool_adapter --locked --offline -- --nocapture
```

Targeted accepted layout tests, baseline identity comparisons, unstaged and
staged whitespace checks, tracked/reachable sensitive-material scans,
generated-artifact scans, and `git fsck --full --strict` also passed. No
dependency or lockfile changed. The accepted `piv1-math` source and identity
remained unchanged. `rustfmt` and `clippy` were not installed and were not
added. No Anchor command was run.

## Serialized-layout result

No field, enum variant, derive, declaration order, maximum payload, or planned
space changed for any serialized schema:

| Schema | Payload | Planned space |
| --- | ---: | ---: |
| `PivConfig` | 1,006 bytes | 1,014 bytes |
| `ActiveDistribution` | 883 bytes | 891 bytes |
| `WithdrawalLeg` | 255 bytes | 263 bytes |
| `GuardianRegistry` | 202 bytes | 210 bytes |
| `GuardianReward` | 76 bytes | 84 bytes |

Task 2.2 uses the existing pending fields exactly. Observation and result types
are ordinary non-serialized production values. Mock audit state is test-only.

## Deferred handler validation

A future authorized handler must derive and validate, as applicable:

- the configured fixed pending-vault addresses and PDA derivations under an
  authorized real Program ID;
- System/legacy Token Program ownership and executable program identities;
- JitoSOL mint, decoded token authority, token-account initialization/data,
  and decoded token amount;
- contributor source authority and exact pre/post transfer deltas;
- current Rent and the exact non-economic System-account balance floor;
- account aliasing, writable/signer constraints, and atomic CPI/system effects;
- event facts and transaction-clock metadata; and
- concurrent account locking and all post-transfer custody invariants.

Task 2.2 does not claim that its host observations prove any of those external
facts.

## Deferred all-vault reconciliation

Task 2.2 reconciles only `PendingSolVault` and `PendingJitoVault`. Positive
unexplained balances may also reach principal custody, `OperationalSolVault`,
`DistributionEscrow`, or `KifSolVault`. Those categories cannot share this
simple rule because physical normalization must preserve historical principal,
operational rent, fixed active-round escrow obligations, KIF liabilities and
carry, and next-cycle yield without inventing ownership.

The recommended later bounded Phase 2 task is **PROVISIONAL Task 2.3 —
liability-aware all-vault physical normalization and adversarial accounting
tests**. It should classify unexplained principal, operational, escrow, and KIF
surpluses only after fixed-account handler observations and recorded liabilities
are available. It must never count principal-vault surplus as yield, sweep
escrow or KIF liabilities, repurpose operational value, or modify an active
distribution to consume unexplained funds.

Task 2.3 is not authorized or started. The exact next action is separate
scoping and authorization of Task 2.3.

## Security and safety boundary

Task 2.2 created no Program ID, wallet, keypair, seed phrase, address, provider,
IDL, deployable entrypoint, or SBF artifact. It ran no Anchor command, validator,
RPC operation, or transaction; moved no funds; changed no authority; and
performed no Mainnet action. The ignored Task 0.4 Testnet key artifact was not
inspected, read, copied, moved, deleted, staged, or modified.

This AI-assisted implementation and review are not a professional independent
audit.
