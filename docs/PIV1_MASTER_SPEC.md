# PIV1 Master Specification and Technical Handoff v0.2

**Project:** HTFP Project  
**Component:** PIV1 - Perpetual Income Vault 1  
**Document date:** 2026-08-30
**Document language:** English for implementation clarity  
**Founder discussion language:** French  
**Status:** Phase 0 and Tasks 1.1-1.3 founder-accepted; Task 1.4 not started

---

## 1. Document purpose

This document consolidates the current product, economic, accounting, governance, security, testing, and delivery decisions for PIV1. It is the starting source of truth for the dedicated PIV1 development chat and Codex work on the VPS.

It does not authorize:

- mainnet deployment;
- movement of real funds;
- creation or publication of secrets;
- transfer of upgrade authority;
- use of unverified recipient addresses;
- irreversible actions.

External protocol details, addresses, versions, fees, and constraints must be reverified against official sources before they are used in code or deployment scripts.

---

## 2. Mission

PIV1 is the first production infrastructure brick of the HTFP Project.

Its mission is to:

1. accept irreversible SOL and JitoSOL contributions;
2. hold productive principal for the very long term;
3. stake liquid SOL through the Jito stake pool;
4. measure yield conservatively in SOL lamports;
5. periodically allocate that yield according to fixed economic shares;
6. pay external beneficiaries in SOL;
7. compound a fixed share into principal;
8. compensate active KIF guardians;
9. remain maintainable through a 4-of-6 upgrade authority.

PIV1 is not a depositor yield product. Contributors receive no shares, claims, withdrawal rights, ownership rights, or individualized rewards.

---

## 3. Economic split

For every valid distribution cycle, gross yield is allocated as follows:

- **59%** - HTFP SOL reserve destination;
- **19.5%** - permanently compounded into PIV1 principal;
- **19.5%** - Team Owner Pool destination;
- **2%** - active KIF guardians.

Total: 100%.

All calculations use integer arithmetic. Outgoing amounts are floored. No rounding operation may cause PIV1 to distribute more than the amount conservatively available.

### 3.1 Outgoing versus retained yield

- Outgoing gross allocation: 80.5% of gross yield.
- Retained compound allocation: 19.5% of gross yield.

Jito withdrawal/protocol costs reduce the outgoing allocation. In a
multi-validator round, withdrawal fees and conversion floors are calculated
separately for every leg and then accumulated. They must not reduce protected
principal or the 19.5% compound allocation.

Solana transaction fees and priority fees are paid by the external fee payer that submits the transaction. PIV1 does not reimburse them.

---

## 4. Initial staking strategy

### 4.1 Initial asset

PIV1 V1 uses JitoSOL.

### 4.2 Entry path

PIV1 converts eligible SOL to JitoSOL by depositing directly into the Jito stake pool. No DEX is used for the normal entry path.

Phase 0 confirmed the direct protocol path. Production and later integration
tests must preserve these properties:

- direct SOL deposit receives JitoSOL in the same transaction;
- no DEX market slippage, while current pool-state drift is bounded by the
  confirmed protected-instruction policy;
- stake-pool deposit fees and token amount are derived from official pool state;
- the JitoSOL mint and stake-pool accounts are cluster-specific and must be validated;
- the PIV-owned JitoSOL token account remains controlled by a PIV PDA.

### 4.3 Exit path

PIV1 uses delayed direct withdrawal from the Jito stake pool. Jupiter and other DEX swaps are excluded from the V1 core path.

Expected lifecycle:

1. fix the required total JitoSOL target through the official stake-pool path;
2. assign that target exactly across one or more validator withdrawal legs;
3. receive each leg in a deterministic stake account controlled by PIV1 authorities;
4. deactivate each successful stake leg immediately;
5. wait until each leg becomes withdrawable;
6. withdraw every leg's SOL into the same distribution escrow;
7. finalize the beneficiary distribution atomically only after all legs and cumulative accounting reconcile.

This path removes market slippage and liquidity impact but requires multiple transactions and an epoch/cooldown delay.

### 4.4 Future migration

The 4-of-6 upgrade authority may later migrate PIV1 to:

- another LST;
- another stake-pool implementation;
- native staking;
- another technically justified strategy.

The current program is intentionally upgradeable to survive protocol changes, bugs, deprecation, or catastrophic LST failure.

### 4.5 Production slippage policy

Production uses only `DepositSolWithSlippage` and
`WithdrawStakeWithSlippage`. Initial tolerance is 1 basis point. Config may set
0 or 1 bps, but the program has an immutable 1-bps hard cap and derives or
verifies every output floor itself. A caller cannot weaken a stored round floor.
Increasing the cap requires a reviewed program upgrade, not an ordinary
configuration transaction.

```text
require 0 <= configured_tolerance_bps <= 1
minimum_pool_tokens_out =
  floor(expected_deposit_units * (10_000 - configured_tolerance_bps) / 10_000)
minimum_lamports_out_i = max(
  runtime_minimum_delegation_i,
  floor(expected_withdraw_lamports_i *
        (10_000 - configured_tolerance_bps) / 10_000)
)
```

Preparation also stores the fixed round-level native floor. Its snapshot basis
must conservatively reserve for the maximum useful leg count and for both kinds
of split-call rounding: the extra ceiling-rounded pool-token fee and the extra
native-output floor that each separate SPL call may introduce. The maximum
useful leg count is derived from the fixed round target and a stored snapshot
technical leg-input floor; every leg must meet the greater of that stored floor
and its current technical minimum. Therefore even a configured tolerance of
zero never models a multi-leg round as one aggregate withdrawal. Per-leg current
quotes and the final cumulative delegated output must remain compatible with
the immutable conservative round floor.

Every operation also requires current pool/list validation, exact post-CPI
balance deltas, current dynamic minimums, and the residual-HWM invariant. Basic
unprotected variants are rejected for production.

### 4.6 Permissionless validator policy

The official PIV keeper queries Jito's current Preferred Withdraw Validator
List API and uses the minimum necessary number of recommended validators,
favoring large safe capacity while respecting the pinned SPL preferred-withdraw
and source-order rules. This HTTP recommendation is operational policy, not an
on-chain-provable invariant.

Any wallet may execute or resume a leg. There is no whitelist, caller reward,
fee reimbursement, or caller custody. A caller-supplied candidate must pass
strict pool, validator-list record/index, vote, derivation, status, epoch,
source-order, remaining-balance, minimum-delegation, mint, authority, liquidity,
slippage, and HWM checks. A different technically safe candidate is permitted
when the API is unavailable or a recommendation changes. This may cause limited
cooldown-reward or pool-rebalancing inefficiency, but cannot redirect or steal
principal, JitoSOL, stake, recovered rent, escrow, or final SOL. Guardians set
general policy, monitor, and pause verified incidents; they do not approve
routine candidates or legs.

---

## 5. Dedicated Program ID

PIV1 has a dedicated Solana Program ID.

Future PIVs may reuse libraries, patterns, tests, or modules, but must not automatically share PIV1 state or Program ID. Isolation prevents an authority/configuration mistake in another PIV from automatically affecting PIV1.

---

## 6. Assets accepted

PIV1 accepts:

- SOL;
- JitoSOL.

Deposits are permissionless and irreversible.

### 6.1 Explicit deposits

Recommended instructions:

- `deposit_sol(amount)`;
- `deposit_jitosol(amount)`.

These instructions may emit contributor, asset, amount, and timestamp/slot information.

### 6.2 Direct transfers

Solana accounts and SPL token accounts can receive transfers outside PIV1 instructions. PIV1 must tolerate and reconcile unexpected/direct balance increases as contributions.

Important limitation: PIV1 cannot automatically emit a custom deposit event for a transfer that did not execute a PIV1 instruction. The public dashboard may still detect such contributions by transaction indexing, but this is an off-chain concern.

### 6.3 No per-depositor state

PIV1 does not create one state account per contributor. Deposit-origin tracking is informational only.

---

## 7. Accounting model

### 7.1 Units

All protected-principal and high-water-mark values are denominated in SOL lamports.

JitoSOL token quantities remain integer token units with 9 decimals. Conversions use official stake-pool accounting and checked integer math.

### 7.2 Core accounting categories

PIV1 must keep these categories logically and, where useful, physically separated:

1. **Protected principal** - SOL-equivalent high-water-mark value.
2. **Principal JitoSOL assets** - JitoSOL controlled by the principal vault.
3. **Pending SOL contributions** - newly received SOL not yet reconciled.
4. **Pending JitoSOL contributions** - newly received JitoSOL not yet reconciled.
5. **Current-cycle gross yield** - calculated only at snapshot.
6. **Distribution obligation** - outgoing allocation reserved for the active cycle.
7. **Compound allocation** - retained 19.5% added to protected principal.
8. **Withdrawal legs and stake accounts** - bounded temporary metadata plus one Stake Program-owned cooldown account per successful validator leg.
9. **Distribution SOL escrow** - SOL ready for final payment.
10. **KIF claim liabilities** - amounts owed to active guardians.
11. **Technical rent balances** - operational lamports excluded from economic principal/yield where necessary.

### 7.3 High-water mark

The protected-principal high-water mark never decreases automatically.

If the SOL-equivalent value of historical principal falls below the high-water mark:

- distributable yield is zero;
- no recovery increase below the high-water mark is treated as yield;
- distributions resume only after value exceeds the high-water mark;
- guardians may upgrade/migrate the strategy if the loss is due to a protocol failure.

### 7.4 Conservative rounding

The phrase "round in favor of long-term safety" means:

- outgoing amounts are floored;
- distributable JitoSOL amount is floored;
- conversions never assume more SOL will be received than the official conservative calculation permits;
- fractions/dust remain inside PIV1;
- checked arithmetic is mandatory;
- no floating-point arithmetic is permitted on-chain.

### 7.5 Pending JitoSOL contributions

A JitoSOL contribution received after the last completed snapshot is not allowed to inflate historical yield.

At the next reconciliation, its current official SOL value is added to protected principal. Any JitoSOL appreciation occurring before reconciliation is conservatively treated as part of the contribution/principal, not distributable historical yield.

### 7.6 Pending SOL contributions

Pending SOL is used first to fund outgoing distributions.

This does not reduce the economic value of the contribution. When pending SOL pays an outgoing obligation, an equivalent amount of historical JitoSOL yield remains staked and is reclassified into principal.

Therefore, after a successful cycle:

`new protected principal = old protected principal + total new contribution value + 19.5% compound + retained conservative dust`

This holds whether the new SOL contribution was physically sent to recipients or staked into JitoSOL.

### 7.7 Contributions during active distribution

Contributions received after a distribution snapshot must remain pending and must not change that distribution's fixed obligations.

They are reconciled only after the active distribution is finalized or recovered.

---

## 8. Yield calculation

### 8.1 Source of truth

Yield uses the official Jito/SPL stake-pool exchange accounting, derived from verified pool state.

DEX prices are not used for principal or yield calculations.

### 8.2 Conceptual calculation

At a snapshot:

1. determine the current conservative SOL value of the historical principal JitoSOL position;
2. compare it with the protected-principal high-water mark, excluding pending contributions and technical rent balances;
3. if current value is not above the high-water mark, gross yield is zero;
4. otherwise, gross yield is the positive difference, subject to integer floors and protocol-account validation.

The exact formula must use the official SPL stake-pool functions or an independently reproduced and thoroughly tested equivalent. Codex must not invent exchange-rate formulas from memory.

### 8.3 Split calculation

Recommended denominator: 10,000 basis points or an exact rational structure that represents 59%, 19.5%, 19.5%, and 2% without floating point.

Example with basis points:

- HTFP: 5,900 bps;
- compound: 1,950 bps;
- Team Owner: 1,950 bps;
- KIF: 200 bps.

Each allocation is floored independently, and the remainder stays in PIV1.

---

## 9. Distribution cadence and state machine

### 9.1 Timing

A distribution preparation may begin no earlier than 10 days after the previous successful `prepare_distribution` snapshot.

The next 10-day interval begins when preparation is successfully recorded, not when final payment occurs.

However, only one distribution may be active. If the prior distribution has not been finalized, a new distribution cannot begin even if 10 days have elapsed.

The program uses Solana's Clock sysvar. No user-supplied timestamp is trusted.

### 9.2 Confirmed state predicates

- `Idle`
- `PreparedWithdrawal` with a nonzero remaining target
- `AssigningWithdrawalLegs`
- `WithdrawalTargetAssigned`
- `AwaitingLegInactivity`
- `PartiallyFinalized`
- `EscrowFunded`
- `Settled`
- `RecoveryRequired`

These names describe bounded header predicates and may coexist where, for
example, a ready leg finalizes while a target still remains. `Paused` is an
orthogonal configuration flag. Readiness is derived independently
for each stake leg from current Stake Program state, Clock, and Stake History;
it is not a caller assertion. The exact enum may be refined, but ambiguous state
combinations are forbidden.

### 9.3 `prepare_distribution`

Permissionless instruction.

Preconditions:

- program not paused;
- no active distribution;
- 10-day minimum elapsed;
- all required protocol and vault accounts validated;
- positive distributable yield;
- if Jito delayed withdrawal is required, the missing amount meets all technical stake-account/protocol minimums.

Actions:

1. reconcile the last completed state without incorporating contributions received after this snapshot begins;
2. calculate historical-principal value and gross yield;
3. calculate gross allocations;
4. fix the cycle's accounting values;
5. determine pending SOL available for outgoing payments;
6. determine JitoSOL withdrawal amount required for the shortfall;
7. credit the 19.5% compound to protected principal accounting;
8. record KIF period allocation data;
9. if liquid SOL fully covers outgoing payments, move to a finalizable liquid state;
10. otherwise store the fixed total JitoSOL withdrawal target and transition to `PreparedWithdrawal`; no validator leg is required in the preparation transaction;
11. store `prepared_at` from Clock.

No second cycle may be prepared until this cycle finishes.

### 9.4 Multi-validator delayed-withdrawal initiation

Each distribution keeps one reusable bounded `ActiveDistribution` header and
creates one temporary `WithdrawalLeg` metadata PDA plus one Stake Program-owned
`WithdrawalStake` PDA for every successful source:

```text
WithdrawalLeg:   ["withdrawal-leg", round_sequence_le_u64, leg_index]
WithdrawalStake: ["withdrawal-stake", round_sequence_le_u64, leg_index]
```

The header stores no unbounded vector. It stores the fixed total JitoSOL target,
cumulative assigned input, per-round cumulative withdrawal fees, burned units,
expected/delegated native output, finalized SOL, recovered rent, cooldown
rewards/losses, the next index, successful/finalized counts, target-assigned and
all-finalized flags, fixed beneficiary gross obligations, stored slippage floor,
and HWM proof values. Arithmetic uses checked `u128` intermediates and checked
conversion to bounded stored types.

For each permissionless initiation:

1. validate current official pool state and the supplied candidate;
2. enforce the on-chain preferred withdraw validator and pinned SPL source order;
3. calculate that source's maximum safe capacity after every required residual;
4. set `leg_input = min(remaining_fixed_target, candidate_maximum_safe_capacity)`;
5. reject a caller-selected smaller or technically invalid amount;
6. derive unused `(sequence, leg_index)` metadata and stake PDAs;
7. advance current rent for both accounts from `OperationalSolVault`;
8. execute `WithdrawStakeWithSlippage`, set `PivAuthority` as staker and withdrawer, and deactivate in the same transaction;
9. record the exact input, per-leg fee, burn, delegated output, rent, validator, epoch, and slippage values;
10. atomically update the cumulative header.

A failed candidate attempt leaves the leg accounts and every cumulative counter
unchanged. `cumulative_jitosol_assigned` may never exceed the fixed target, and
no new leg may open after exact equality. Unique indices, the technical minimum,
maximum-safe fill, finite target, validated capacity, and available operational
rent bound useful leg creation without imposing a low economic distribution cap.
A supplied candidate is rejected if its mandatory maximum-safe fill would leave
a nonzero target below the greater of the stored snapshot leg-input floor and
the current per-leg technical minimum.

### 9.5 Cooldown monitoring and resumability

A keeper/CLI checks each recorded withdrawal stake account. The program validates
readiness from on-chain stake state and never trusts the keeper's assertion.
Different legs may become inactive in different epochs.

Another permissionless caller can resume safely:

- after only some legs were initiated, by quoting the remaining exact target;
- after a candidate becomes unavailable or an epoch changes, by refreshing pool,
  list, source, fee, and slippage data and selecting another valid candidate;
- after only some legs become inactive, by finalizing only ready legs;
- after only some legs finalize, from checked cumulative counts and totals;
- while the Jito API is unavailable, by supplying any candidate that passes all
  enforceable on-chain rules, accepting only the documented efficiency trade-off;
- after stale pool state, once permissionless SPL maintenance makes it current;
- after insufficient operational rent, once the approved operational category is
  replenished, without consuming a new leg index in the failed attempt.

### 9.6 Leg finalization and atomic distribution settlement

`finalize_withdrawal_leg` is permissionless. It validates the exact round, leg,
stake PDA, stake authorities, and inactivity; withdraws the complete stake
balance to the fixed `DistributionEscrow`; reconciles delegated SOL, cooldown
reward/loss, and recovered stake rent; updates cumulative totals atomically;
prevents replay; and closes or safely transitions temporary metadata while
returning its rent to `OperationalSolVault`.

`settle_distribution` is permissionless and is allowed only when:

```text
cumulative_jitosol_assigned == fixed_round_jitosol_target
successful_leg_count == finalized_leg_count
all successful stake accounts are finalized
fixed escrow and cumulative accounting reconcile
```

Settlement then atomically determines net outgoing SOL, transfers HTFP and Team
amounts, funds/credits KIF liabilities, and commits the fixed cycle accounting
and HWM delta. No beneficiary receives a partial multi-leg distribution. Pending
contribution integration and later principal SOL/Jito compounding remain separate
logical boundaries. If any required action fails, no beneficiary is paid twice
and the distribution remains resumable.

### 9.7 Technical minimum

No arbitrary economic minimum is required merely because the caller might lose money on transaction fees.

However, delayed withdrawal to a stake account may impose a mandatory technical minimum caused by:

- rent-exempt stake-account balance;
- current cluster minimum stake delegation;
- Jito/SPL pool withdrawal constraints;
- protocol fees;
- integer conversion floors.

Phase 0 confirmed the dynamic formula and demonstrated point-in-time boundaries;
Phase 1 must implement it and later tests must remeasure current values. Per-leg
feasibility also includes bounded metadata rent, candidate maximum safe
capacity, and a nonzero remaining target that is not stranded below the next
technical minimum. If pending SOL does not cover the outgoing obligation and
the required Jito withdrawal is below the technical minimum,
`prepare_distribution` must not lock PIV1 in an unfinishable cycle. Yield simply
continues accumulating.

### 9.8 Insufficient-attempt anti-spam cooldown

If a valid permissionless distribution attempt determines that the available outgoing amount is below the measured technical minimum:

- no distribution snapshot is created;
- no yield or contribution amount is reclassified;
- no withdrawal is initiated;
- the accumulated yield remains available for a later attempt;
- the program records the unsuccessful valid attempt time;
- another insufficiency evaluation is rejected for 24 hours.

A transaction that fails account validation, signature checks, arithmetic checks, or other preconditions must not update this cooldown. This prevents an attacker from extending the cooldown with malformed transactions.

The 24-hour retry cooldown is independent of the normal 10-day distribution interval. It does not reset the 10-day clock and cannot postpone a distribution once the amount is technically sufficient.

### 9.9 Single active distribution

PIV1 V1 permits only one successfully prepared distribution at a time. That one
round may have multiple concurrent validator withdrawal stake legs. Overlapping
distribution snapshots remain excluded; temporary legs are bound to the active
sequence and checked cumulative header.

### 9.10 Confirmed logical transaction boundaries

The production lifecycle preserves six logical boundaries:

1. distribution preparation and snapshot;
2. protected JitoSOL withdrawal plus immediate deactivation;
3. inactive-stake finalization into the fixed distribution escrow;
4. atomic beneficiary settlement and accounting;
5. pending-contribution integration;
6. later principal SOL/Jito compounding deposit.

Multi-validator rounds may repeat transactions inside boundaries 2 and 3, so
this is not a promise of exactly six transactions. Final stake withdrawal,
beneficiary fan-out, pending integration, and Jito compounding are not recombined
without later production evidence.

### 9.11 Cooldown rewards, losses, and rent

Each leg records exact delegated output and rent advances. Finalization compares
observed native value net of recovered rent with delegated output. Positive
cooldown rewards are excluded from the already-fixed distribution, recorded as
next-cycle yield, and receive the normal `59% / 19.5% / 19.5% / 2%` split in a
later eligible cycle. They are not silently principal. Recovered stake and
metadata rent returns to the operational category and is never yield. Any
cooldown loss or residual-HWM failure enters `RecoveryRequired`; normal logic
never reduces the HWM.

### 9.12 Per-leg cumulative accounting

The fixed round input is assigned with checked arithmetic:

```text
remaining_i = fixed_round_jitosol_target - cumulative_jitosol_assigned
leg_input_i = min(remaining_i, candidate_maximum_safe_input_i)
require leg_input_i >= max(snapshot_leg_input_floor, current_technical_minimum_i)
fee_i = 0 if current_fee_denominator_i = 0, otherwise
        ceil(leg_input_i * current_fee_numerator_i / current_fee_denominator_i)
burn_i = leg_input_i - fee_i
expected_native_i = floor(burn_i * current_total_lamports_i / current_supply_i)

cumulative_jitosol_assigned += leg_input_i
cumulative_fee_units += fee_i
cumulative_burn_units += burn_i
cumulative_delegated_native += observed_delegated_native_i
```

The program requires `leg_input_i` to equal the maximum-safe fill shown above,
rejects caller-selected micro amounts, and maintains
`cumulative_jitosol_assigned <= fixed_round_jitosol_target`. Fees and floors are
calculated per SPL call, never once over an aggregate as if one validator held
the target. Beneficiary funding uses reconciled finalized native SOL, excluding
recovered rent and current-round-ineligible cooldown rewards, and never exceeds
the fixed outgoing gross allocation.

Settlement requires:

```text
cumulative_jitosol_assigned == fixed_round_jitosol_target
successful_leg_count == finalized_leg_count
all successful stake accounts are finalized
fixed escrow and cumulative accounting reconcile
```

---

## 10. SOL-to-JitoSOL compounding

### 10.1 Priority order

After a completed distribution:

1. preserve only technical rent/operational lamports;
2. retain any SOL explicitly required for already-recorded liabilities;
3. deposit eligible principal SOL directly into Jito's stake pool;
4. receive JitoSOL into the PIV principal token vault;
5. update accounting from actual token output.

### 10.2 Recyclable operational rent reserve

PIV1 maintains a small permanent operational SOL reserve that is excluded from protected principal, yield, beneficiary allocations, and pending contributions.

Its only intended purpose is to pre-fund rent-exempt temporary stake and leg
metadata accounts required by the Jito delayed-withdrawal lifecycle. When each
temporary account closes, its recovered rent lamports return to this operational
reserve and may fund later legs.

The reserve is not an economic liquidity buffer and must never be used to increase a beneficiary payment. Its exact required size is measured from current cluster rules during integration testing rather than hardcoded from assumptions.

### 10.3 Separate instruction option

For safety and compute simplicity, `stake_pending_sol` may be a separate permissionless instruction rather than part of finalization.

This instruction:

- validates current official Jito accounts;
- never stakes SOL reserved for an active distribution or KIF liability;
- uses exact balance deltas;
- records actual JitoSOL received;
- leaves the caller as fee payer.

This separation is recommended unless prototype evidence shows the combined transaction is comfortably safe.

---

## 11. KIF guardian system

### 11.1 Membership

Six guardians correspond to six multisig members/keys.

Initially, the founder may control all six wallets. This is operational key separation, not decentralization. Later, public keys are rotated to independent people/entities.

Private keys are never transferred. Each future guardian creates a fresh wallet and provides only its public key. Squads replaces the old member address by approved 4-of-6 configuration change.

### 11.2 Activity

The KIF period is exactly 2,592,000 seconds (30 days). Configuration stores an
anchor timestamp, and the program derives a monotonic period ID from Solana
Clock `unix_timestamp`. Periods are half-open:

```text
period_seconds = 2_592_000
require unix_timestamp >= configured_anchor_timestamp
period_id = floor(
  (unix_timestamp - configured_anchor_timestamp) / period_seconds
)
period_start = configured_anchor_timestamp + period_id * period_seconds
period_end = period_start + period_seconds
period_start <= unix_timestamp < period_end
```

A guardian is active for a period if it signs an on-chain guardian
heartbeat/attestation or participates in a qualifying governance vote during
that period. Activity after a distribution snapshot is not retroactive for that
distribution.

The simplest V1 mechanism is a dedicated permissionless `guardian_heartbeat` instruction requiring the guardian signature and recording `last_active_period`.

The program must not depend on an off-chain database to decide KIF eligibility.

### 11.3 Allocation when at least one guardian is active

The full available KIF allocation is divided equally among active guardians.

```text
kif_available = current net KIF allocation + approved prior carry
per_guardian = floor(kif_available / active_guardian_count)
credited = per_guardian * active_guardian_count
kif_rounding_remainder = kif_available - credited
```

- inactive guardians receive zero;
- no retroactive catch-up;
- rewards are credited to claimable balances;
- guardians may accumulate balances before claiming;
- division is floored;
- `kif_rounding_remainder = kif_available - credited` remains in
  `KifSolVault` as explicit collective carry for a later allocation;
- no remainder goes preferentially to a guardian, HTFP, Team Owner, an
  arbitrary recipient, or ordinary principal.

### 11.4 Allocation when zero guardians are active

Let `available_kif` be the current cycle's net KIF allocation plus all approved
prior KIF carry. The confirmed implementation is:

- `compound_from_kif = floor(available_kif / 2)`;
- `kif_carry_next = available_kif - compound_from_kif`.

The compounded half permanently increases protected principal. The carried half
remains a collective KIF carry. Apply the same rule again to the complete
available pool in every successive zero-active period; the carry is never an
individual inactive-guardian claim.

### 11.5 Claims

`claim_kif`:

- requires the guardian signer or an explicitly configured payout destination controlled by that guardian;
- pays only the guardian's accrued claimable balance;
- uses checks-effects-interactions ordering;
- cannot modify activity history;
- cannot claim another guardian's balance;
- may be called at any time when not blocked by a justified pause policy.

Whether claims remain enabled during pause must be decided in the threat-model phase. The safe default is to allow claims only when no accounting incident affects KIF liabilities.

---

## 12. Governance and upgrade authority

### 12.1 Multisig

Preferred implementation: Squads.

Configuration:

- six members;
- threshold four;
- no single-member bypass;
- program upgrade authority held by the Squads-controlled authority;
- recipient changes and pause calls executed through approved multisig transactions.

### 12.2 Full authority

Four of six guardians may approve a complete program upgrade. This is an explicit founder decision.

Consequences that must be documented publicly:

- 4/6 can alter economic rules;
- 4/6 can alter destinations;
- 4/6 can alter migration logic;
- 4/6 can theoretically introduce malicious behavior.

Security therefore depends on key custody, member independence, review discipline, and transparent code publication in addition to on-chain code.

### 12.3 Pause

An explicit pause instruction is included because deploying an upgrade is slower than activating a circuit breaker.

Pause blocks:

- new distribution snapshots;
- new Jito deposits;
- new delayed withdrawals;
- finalizations;
- migrations;
- other state-changing economic operations designated in the final threat model.

Direct incoming transfers may still occur and must remain reconcilable.

### 12.4 Temporary recipients

PIV1 may launch before the HTFP SOL Vault and Team Owner Pool programs exist.

Mainnet configuration therefore uses:

- a real temporary HTFP treasury controlled by the multisig;
- a real temporary Team Owner treasury controlled by the multisig;
- internal KIF claim accounting.

Later, 4/6 replaces temporary recipients with final program-controlled destinations.

No null, empty, invented, or unverified recipient address is permitted.

---

## 13. Confirmed production account model

The account roles and custody topology are confirmed. Phase 1 must still define
versioned bounded field layouts, exact account sizes, bumps, rent, and safe close
behavior without changing these roles.

### 13.1 `PivConfig` PDA

Stores:

- version;
- bump;
- paused flag;
- Jito cluster configuration references;
- JitoSOL mint;
- PrincipalJitoVault and PendingJitoVault;
- PendingSolVault and PrincipalSolQueue;
- OperationalSolVault, DistributionEscrow, and KifSolVault;
- recipient addresses;
- split constants;
- minimum interval;
- last prepared timestamp;
- next distribution sequence;
- current distribution state reference;
- protected principal lamports;
- relevant accounting totals;
- slippage tolerance constrained to 0–1 bps under an immutable 1-bps hard cap;
- KIF anchor timestamp and fixed 2,592,000-second period configuration;
- guardian registry reference;
- reserved future-migration fields.

### 13.2 `PrincipalJitoVault`

Distinct PIV1-derived account-address PDA initialized as a 165-byte legacy SPL
Token account. The legacy Token Program owns the initialized account; it is
bound to official JitoSOL and records `PivAuthority` as decoded token authority.
It is not an ATA.

### 13.3 `PendingJitoVault`

Different PIV1-derived account-address PDA, also initialized as a 165-byte
legacy SPL Token account owned by the legacy Token Program, bound to official
JitoSOL, controlled by `PivAuthority`, and not an ATA. It receives unreconciled
JitoSOL contributions. Physical and ledger separation from principal is
mandatory.

### 13.4 `PendingSolVault`

Empty-data System-owned PIV1 PDA receiving SOL contributions pending
reconciliation. Its rent floor is excluded from economics.

### 13.5 `PrincipalSolQueue`

Empty-data System-owned PIV1 PDA holding reconciled principal SOL until a later
protected direct Jito deposit. It cannot fund beneficiaries outside fixed round
accounting.

### 13.6 `OperationalSolVault`

Empty-data System-owned PIV1 PDA holding only the permanent operational reserve
and per-leg rent advances. Its rent floor and balance are excluded from
principal, yield, contributions, distributions, and KIF liabilities.

### 13.7 `DistributionEscrow`

Empty-data System-owned PIV1 PDA holding liquid SOL assigned to the active
distribution. Its rent floor is excluded and beneficiaries are paid only by the
atomic settlement boundary.

### 13.8 `KifSolVault`

Empty-data System-owned PIV1 PDA backing aggregate guardian claims and explicit
KIF carry. It is never mixed with principal, pending contributions, operational
rent, or beneficiary funds.

### 13.9 `ActiveDistribution` PDA

One reusable bounded header and one active sequence at a time. It stores no
unbounded leg vector. It proves:

- sequence ID;
- state enum;
- prepared timestamp/slot/epoch;
- old protected principal;
- historical principal token amount/value inputs;
- gross yield;
- gross allocations;
- pending SOL used;
- fixed total JitoSOL input target;
- snapshot technical leg-input floor, maximum useful-leg bound, and the
  conservative split-round native floor;
- cumulative assigned JitoSOL input, withdrawal fees, burned pool tokens,
  expected/delegated native output, finalized SOL, recovered rent, and cooldown
  rewards/losses;
- next leg index, successful and finalized leg counts, target-assigned and
  all-finalized flags;
- fixed gross beneficiary obligations, stored slippage/HWM bounds, and actual
  net SOL;
- active KIF guardian bitmap or immutable eligibility snapshot;
- KIF carry inputs;
- final payment/accounting flags.

### 13.10 `WithdrawalLeg` and `WithdrawalStake`

For each successful leg index in the active sequence:

```text
WithdrawalLeg:   ["withdrawal-leg", sequence_le_u64, leg_index]
WithdrawalStake: ["withdrawal-stake", sequence_le_u64, leg_index]
```

`WithdrawalLeg` is temporary bounded PIV1-owned metadata recording candidate,
epoch, exact input, fee, burn, delegated output, both rent advances, slippage,
and finalized/replay state. `WithdrawalStake` is the temporary Stake
Program-owned destination for that one SPL withdrawal. `PivAuthority` is both
staker and withdrawer. Exact derivation and full lifecycle were proven for one
stake leg on public Testnet; the multi-leg cumulative orchestration remains a
confirmed architecture requirement to implement and test later.

### 13.11 `GuardianRegistry` PDA

Stores six guardian public keys and KIF timing configuration, or mirrors the Squads member set through an explicit synchronized configuration.

Do not assume the PIV program can cheaply query arbitrary Squads state without validating the integration. It may be safer to store guardian keys in PIV config and require 4/6-authorized updates.

### 13.12 `GuardianReward` PDAs

One account per guardian storing:

- guardian pubkey;
- last active period;
- claimable lamports;
- cumulative earned;
- cumulative claimed;
- bump/version.

Six fixed accounts are acceptable and avoid loops over an unbounded holder set.

---

## 14. Confirmed instruction roles

Final naming may change.

### Initialization and governance

- `initialize_piv1`
- `pause`
- `unpause`
- `update_recipients`
- `update_guardian_set`
- `update_jito_config` or migration-specific instructions
- optional `migrate_state_version`

### Deposits and staking

- `deposit_sol`
- `deposit_jitosol`
- `reconcile_untracked_balances`
- `stake_pending_sol`

### Distribution lifecycle

- `prepare_distribution`
- `initiate_withdrawal_leg`
- optional `deactivate_withdrawal_stake`
- `finalize_withdrawal_leg`
- `settle_distribution`
- `integrate_pending`
- optional permissionless `close_distribution_accounts`

### KIF

- `guardian_heartbeat`
- `claim_kif`

Every instruction needs explicit account constraints, owner checks, mint checks, PDA seed checks, replay protection, state-transition checks, and events.

---

## 15. Events

Recommended events:

- `PivInitialized`
- `SolContribution`
- `JitoSolContribution`
- `UntrackedBalanceReconciled`
- `PendingSolStaked`
- `DistributionPrepared`
- `DelayedWithdrawalInitiated`
- `WithdrawalLegInitiated`
- `WithdrawalLegFinalized`
- `WithdrawalReady`
- `DistributionFinalized`
- `GuardianHeartbeat`
- `KifRewardsCredited`
- `KifClaimed`
- `PauseChanged`
- `RecipientsUpdated`
- `GuardianSetUpdated`
- `StrategyConfigUpdated`

Events must never be the sole source of accounting truth; on-chain state remains authoritative.

---

## 16. Required invariants

1. No normal instruction permits a contributor to withdraw a deposit.
2. Outgoing value never exceeds conservatively calculated yield assigned to outgoing shares.
3. The 19.5% compound allocation is never used to pay Jito withdrawal costs.
4. Solana network fees are never reimbursed by PIV1.
5. Pending contributions are never misclassified as historical yield.
6. Contributions received during an active distribution do not alter fixed obligations.
7. Protected principal high-water mark never decreases through normal distribution logic.
8. A recovery from loss below the high-water mark is not yield.
9. At most one distribution is active.
10. A `(distribution sequence, leg index)` cannot be initiated or finalized twice, and a failed attempt consumes no index or cumulative capacity.
11. A distribution cannot be finalized twice.
12. HTFP and Team Owner recipients cannot be supplied arbitrarily by a permissionless caller.
13. KIF eligibility is fixed for the relevant cycle before rewards are credited.
14. Inactive guardians receive zero for inactive periods.
15. Claims cannot exceed claimable balances.
16. All divisions floor; all additions/multiplications use checked arithmetic.
17. Protocol accounts, mints, programs, owners, and PDAs are validated against configured values.
18. Pause does not create a withdrawal path.
19. Direct/unexpected balance increases remain recoverable as contributions.
20. Temporary-account rent recovery cannot be mistaken for yield or sent to an unauthorized recipient.
21. Cumulative JitoSOL input never exceeds the fixed round target, each successful leg uses the maximum safe supplied-source fill, and the target is exactly assigned before leg initiation closes.
22. Settlement is impossible until successful and finalized leg counts match, every successful stake leg is closed, and escrow plus cumulative accounting reconcile.
23. Per-leg fees, burn, floors, output, rent, rewards/losses, and metadata-rent recovery are accounted separately before checked accumulation.

---

## 17. Threat model summary

### External protocol risks

- Jito/SPL stake-pool bug or exploit;
- Jito program/account upgrade or deprecation;
- incorrect official accounts supplied to CPI;
- delayed withdrawal behavior changes;
- fee changes;
- stake-account state delays;
- validator-source capacity changes and fragmented multi-leg cooldowns;
- catastrophic JitoSOL loss.

Mitigation:

- official account validation;
- pinned/tested integrations;
- complete 4/6 upgrade authority;
- pause;
- open-source review;
- migration capability through upgrades;
- high-water mark.

### Accounting risks

- deposit counted as yield;
- yield counted twice;
- distribution finalized twice;
- pending deposit races;
- integer overflow;
- wrong exchange-rate direction;
- protocol fee charged to principal;
- rent balance treated as economic value.
- per-leg fee rounding incorrectly modeled as one aggregate fee;
- caller-created micro-leg or rent-drain attempts;

Mitigation:

- physically separated queues/escrows;
- explicit state machine;
- property tests;
- conservative floors;
- sequence IDs;
- balance-delta assertions;
- checked math.

### Governance risks

- founder controls all six keys initially;
- four compromised keys can upgrade the program;
- seed loss;
- malicious recipient change;
- accidental upgrade.

Mitigation:

- separate dedicated wallets;
- later distribution to independent guardians;
- hardware wallets when possible;
- Git/build verification;
- code review before upgrades;
- transaction descriptions and account decoding;
- public source and upgrade history.

### Operational risks

- VPS compromise;
- secret leakage;
- dependency supply-chain attack;
- wrong cluster or Program ID;
- stale Jito addresses;
- RPC inconsistency.

Mitigation:

- no mainnet seeds on VPS;
- pinned lockfiles/toolchains;
- minimal privileges;
- Git commits/checkpoints;
- cluster-genesis verification;
- official-address revalidation;
- reproducible builds;
- separate deployment/signing process.

---

## 18. Testing requirements

### 18.1 Pure math tests

- exact split examples;
- all rounding boundaries;
- 0/1 lamport cases;
- maximum supported values;
- checked overflow failures;
- high-water mark loss/recovery;
- protocol fee allocation;
- KIF 1-6 active guardians;
- zero active guardians across repeated periods;
- carry and dust.

### 18.2 Property tests/fuzzing

Properties:

- no outgoing amount exceeds gross yield;
- principal never decreases in normal positive-yield distribution accounting;
- deposits never decrease principal;
- inactive guardian reward is always zero;
- sum of credited/transferred/retained values never exceeds available value;
- arbitrary instruction order cannot finalize twice;
- random direct-balance changes cannot become historical yield.

### 18.3 Local integration tests

Use a controllable mock stake pool/adapter capable of:

- increasing exchange rate;
- decreasing exchange rate;
- charging configurable fees;
- delayed withdrawal states;
- failed CPI;
- exact and insufficient liquidity;
- minimum withdrawal failures;
- delayed epoch advancement.
- multiple source capacities and partial target assignment;
- independently delayed leg finalization;
- operational-rent exhaustion and later replenishment.

Test:

- SOL deposit;
- JitoSOL deposit;
- direct/untracked transfers;
- 10-day timing;
- prepare/finalize;
- pending deposits during active cycle;
- pause/unpause;
- recipient update;
- guardian rotation;
- claims;
- retries after failure.
- maximum-safe per-candidate fill and exact cumulative target;
- candidate failure or epoch change after partial assignment;
- different inactivity and finalization epochs across legs;
- repeated zero-active KIF carry and active-guardian remainder carry.

### 18.4 Testnet integration tests

Use the current official Jito Testnet program, stake pool, and mint only after revalidation.

Required demonstrations:

- direct SOL deposit into Jito Testnet pool;
- JitoSOL received by PDA-controlled account;
- official exchange-rate reading;
- delayed withdrawal initiation;
- stake-account authority correctness;
- deactivation and epoch wait;
- final SOL withdrawal;
- atomic final distribution;
- account closure/rent recovery;
- multiple cycles;
- pause during each lifecycle state.

### 18.5 Security/adversarial review

- substitute fake Jito program;
- substitute fake mint;
- fake stake pool;
- fake recipient;
- fake guardian;
- replay old distribution;
- double finalize;
- duplicate, skipped, reused, and out-of-order leg indices;
- settlement before exact target assignment or complete leg finalization;
- API-preferred versus technically safe non-API candidate behavior;
- metadata/stake-PDA replay and rent-drain attempts;
- direct-transfer race;
- stake account with wrong authority;
- wrong epoch/readiness claim;
- malicious remaining accounts;
- account close/redirection attempt;
- upgrade/config authority mismatch.

---

## 19. Development environment recommendation

### 19.1 VPS

Use the Linux VPS as the primary development environment.

Reasons:

- native Linux toolchain for Rust, Solana, Anchor, Node, and test validators;
- persistent repository and build cache;
- Codex CLI can inspect/edit/run the repository directly;
- easier repeatable automation and long test runs;
- Git history remains centralized.

### 19.2 Local computer

Use the local computer for:

- wallet interactions;
- guardian hardware-wallet setup;
- final transaction review;
- UI/browser testing;
- mainnet signing and Squads approval where practical.

### 19.3 Secret boundary

Allowed on VPS:

- disposable localnet keys;
- disposable Testnet keys;
- public addresses;
- non-secret RPC URLs where acceptable.

Forbidden on VPS/repository/Codex prompts:

- guardian seed phrases;
- mainnet treasury seed phrases;
- private keys controlling real funds;
- unencrypted backup phrases;
- production secrets.

---

## 20. Repository structure

Recommended initial structure:

```text
piv1/
├── AGENTS.md
├── Anchor.toml
├── Cargo.toml
├── Cargo.lock
├── package.json
├── pnpm-lock.yaml or yarn.lock
├── tsconfig.json
├── rust-toolchain.toml
├── programs/
│   └── piv1/
│       ├── Cargo.toml
│       └── src/
│           ├── lib.rs
│           ├── constants.rs
│           ├── errors.rs
│           ├── events.rs
│           ├── math.rs
│           ├── state/
│           │   ├── mod.rs
│           │   ├── config.rs
│           │   ├── distribution.rs
│           │   └── guardian.rs
│           ├── instructions/
│           │   ├── mod.rs
│           │   ├── initialize.rs
│           │   ├── deposit_sol.rs
│           │   ├── deposit_jitosol.rs
│           │   ├── stake_pending_sol.rs
│           │   ├── prepare_distribution.rs
│           │   ├── initiate_withdrawal_leg.rs
│           │   ├── finalize_withdrawal_leg.rs
│           │   ├── settle_distribution.rs
│           │   ├── integrate_pending.rs
│           │   ├── guardian_heartbeat.rs
│           │   ├── claim_kif.rs
│           │   ├── pause.rs
│           │   └── update_config.rs
│           └── integrations/
│               ├── mod.rs
│               ├── stake_pool.rs
│               └── jito.rs
├── crates/
│   └── piv1-math/
├── tests/
│   ├── unit/
│   ├── integration/
│   ├── adversarial/
│   └── fixtures/
├── clients/
│   └── cli/
├── scripts/
│   ├── setup-localnet.sh
│   ├── deploy-testnet.sh
│   ├── verify-build.sh
│   └── inspect-accounts.sh
├── docs/
│   ├── PIV1_MASTER_SPEC.md
│   ├── PIV1_DECISIONS.md
│   ├── PIV1_INVARIANTS.md
│   ├── PIV1_THREAT_MODEL.md
│   ├── PIV1_TEST_PLAN.md
│   ├── PIV1_TESTNET_RUNBOOK.md
│   └── PIV1_MAINNET_CHECKLIST.md
└── .github/
    └── workflows/
        ├── ci.yml
        └── reproducible-build.yml
```

The exact structure may evolve, but a one-file program is explicitly unacceptable.

---

## 21. Phased delivery plan

### Phase 0 - Environment and protocol validation

**COMPLETE / FOUNDER-ACCEPTED on 2026-08-30.** No product code was created.

- inventory VPS;
- initialize private Git repository;
- pin toolchain versions;
- verify official Jito Mainnet/Testnet accounts;
- inspect Jito reference implementation and SPL stake-pool source;
- prove direct deposit/withdraw transactions manually on Testnet;
- measure technical withdrawal minimum;
- determine PDA stake-account creation/authority method;
- document findings and conflicts.

Gate: **SATISFIED.** The founder accepted the report, seven schema decisions,
the corrected dual-token-vault topology, and scalable multi-validator V1
withdrawals. The one-leg custody lifecycle is live-tested; multi-leg
orchestration is architecture-confirmed and not yet live-tested.

### Phase 1 - Specification-as-code foundations

- repository scaffold;
- AGENTS.md;
- math crate;
- state enums and account schemas;
- no Jito CPI yet;
- unit/property tests.

Gate: invariants and tests pass.

### Phase 2 - Mock localnet PIV

- mock stake pool/adapter;
- deposits;
- accounting;
- distribution state machine;
- KIF;
- pause/config;
- adversarial tests.

Gate: full local suite and state-machine review.

### Phase 3 - Real SPL/Jito adapter

- direct SOL deposit;
- JitoSOL vaults;
- official exchange-rate reading;
- delayed withdrawal account lifecycle;
- integration behind stable internal interface.

Gate: local cloned-program tests plus code review.

### Phase 4 - Testnet

- deploy PIV1 Testnet;
- use official current Jito Testnet deployment;
- execute complete cycles;
- verify events/accounting;
- CLI and runbook;
- failure/retry tests.

Gate: Testnet validation report.

### Phase 5 - Security hardening

- independent AI review sessions;
- community review where possible;
- fuzz/property expansion;
- dependency audit;
- reproducible build;
- security.txt;
- verified build process;
- Squads dry run.

Gate: no unresolved critical/high issues.

### Phase 6 - Mainnet preparation

- create six guardian wallets securely;
- create 4/6 Squads multisig;
- create temporary HTFP and Team Owner treasuries;
- verify all addresses multiple times;
- set final constants;
- prepare deterministic deployment artifacts;
- deploy with founder approval;
- transfer upgrade authority to Squads;
- verify deployed binary against source;
- publish source;
- deposit approximately 1-2 SOL only after checks.

Gate: explicit founder approval at every irreversible step.

---

## 22. Mainnet launch conditions

Mainnet is blocked until all are true:

- all Phase 0-5 gates pass;
- official Jito addresses are reverified;
- Program ID is final;
- six guardian public keys exist and are backed up;
- 4/6 Squads is tested;
- temporary recipient addresses exist and are verified;
- no placeholder/null address remains;
- build is reproducible/verifiable;
- source is ready for publication;
- upgrade authority transfer transaction is reviewed;
- rollback/incident communication plan exists;
- founder explicitly approves deployment;
- founder explicitly approves authority transfer;
- founder explicitly approves first real deposit.

Passing Testnet tests alone is not sufficient authorization for Mainnet.

---

## 23. Public communication constraints

Do not claim:

- guaranteed safety;
- guaranteed perpetual yield;
- guaranteed principal value in fiat terms;
- professional independent audit without one;
- legal or regulatory compliance without qualified review;
- zero risk.

Accurate statements may include:

- principal withdrawals are not available through normal PIV1 instructions;
- code and deployed build are publicly verifiable once published;
- upgrade authority is controlled by a 4/6 multisig;
- JitoSOL introduces external protocol risk;
- yield and timing vary;
- the program has been tested locally/Testnet according to published evidence.

---

## 24. Remaining non-blocking/open items

Phase 0 schema-blocking founder decisions are resolved. Phase 1 must implement
and test the confirmed bounded architecture without inventing launch inputs.

Items intentionally deferred to implementation, later configuration, or
launch authorization:

1. future instruction-handler validation of official external accounts,
   sysvars, ownership, balances, and protocol-derived inputs;
2. production CU, transaction-size, loaded-data, and rent measurements for a
   withdrawal leg and settlement;
3. local multi-leg orchestration, property, adversarial, and later Testnet tests;
4. exact acronym expansion for KIF;
5. final Program ID;
6. final six guardian public keys;
7. final temporary recipient addresses;
8. any separately justified future toolchain/dependency update.

No economic maximum distribution size, caller reward, per-leg guardian approval,
or HTTP API oracle is inferred. Task 1.1 is COMPLETE / FOUNDER-ACCEPTED at
implementation commit 1d436570570fc31310e3e5d2c1d4d5e92320c65b.

Task 1.2 is **COMPLETE / FOUNDER-ACCEPTED** at implementation commit
43a3b7497653ff7a246a1e5cf9b760086dd33fcd. Task 1.3 is **COMPLETE /
FOUNDER-ACCEPTED** at final implementation tip
527e381661fe0cfc27e07ad9b44e1601a638ae75; its initial implementation commit is
33978cf3eda918e4c438b80ed0e12a47b8347519. Task 1.4 has not started. The exact
next task is **Task 1.4 randomized/property and adversarial invariant testing**,
which requires separate founder authorization and a dedicated branch.

---

## 25. Current official integration references to reverify

Phase 0 validation resolved Testnet as the current officially supported Jito non-Mainnet cluster. Devnet is rejected for Task 0.4 because current official support was not confirmed and its observed pool state was stale. Cluster-specific values are not hardcoded here as timeless truth; the Phase 0 report must retrieve and verify them again before use.

Official source families to use:

- Jito Foundation JitoSOL documentation;
- Jito Preferred Withdraw Validator List API documentation;
- Jito StakeNet/Steward documentation and source for operational context;
- Jito stake/unstake reference repository;
- Solana official program, stake-account, Clock, account, and verified-build documentation;
- SPL Stake Pool source/documentation;
- Anchor official documentation;
- Squads official documentation;
- OpenAI Codex official documentation.

No blog, aggregator, or AI-generated code should override primary-source behavior.
