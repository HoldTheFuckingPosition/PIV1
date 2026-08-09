# PIV1 Master Specification and Technical Handoff v0.2

**Project:** HTFP Project  
**Component:** PIV1 - Perpetual Income Vault 1  
**Document date:** 2026-08-03  
**Document language:** English for implementation clarity  
**Founder discussion language:** French  
**Status:** Ready to begin Phase 0 technical validation and local development planning  

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

Jito withdrawal/protocol costs reduce the outgoing allocation. They must not reduce protected principal or the 19.5% compound allocation.

Solana transaction fees and priority fees are paid by the external fee payer that submits the transaction. PIV1 does not reimburse them.

---

## 4. Initial staking strategy

### 4.1 Initial asset

PIV1 V1 uses JitoSOL.

### 4.2 Entry path

PIV1 converts eligible SOL to JitoSOL by depositing directly into the Jito stake pool. No DEX is used for the normal entry path.

Expected properties to validate during Phase 0 and integration tests:

- direct SOL deposit receives JitoSOL in the same transaction;
- no market slippage;
- stake-pool deposit fees and token amount are derived from official pool state;
- the JitoSOL mint and stake-pool accounts are cluster-specific and must be validated;
- the PIV-owned JitoSOL token account remains controlled by a PIV PDA.

### 4.3 Exit path

PIV1 uses delayed direct withdrawal from the Jito stake pool. Jupiter and other DEX swaps are excluded from the V1 core path.

Expected lifecycle:

1. burn/withdraw the required JitoSOL amount through the official stake-pool path;
2. receive a stake account controlled by PIV1 authorities;
3. deactivate that stake account;
4. wait until it becomes withdrawable;
5. withdraw SOL into the distribution escrow;
6. finalize the beneficiary distribution atomically.

This path removes market slippage and liquidity impact but requires multiple transactions and an epoch/cooldown delay.

### 4.4 Future migration

The 4-of-6 upgrade authority may later migrate PIV1 to:

- another LST;
- another stake-pool implementation;
- native staking;
- another technically justified strategy.

The current program is intentionally upgradeable to survive protocol changes, bugs, deprecation, or catastrophic LST failure.

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
8. **Withdrawal stake account** - stake being cooled down for the active distribution.
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

### 9.2 Proposed states

- `Idle`
- `PreparedLiquidOnly`
- `WithdrawalRequested`
- `CoolingDown`
- `ReadyToFinalize`
- `Paused`

The exact enum may be refined, but ambiguous state combinations are forbidden.

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
10. otherwise initiate delayed Jito withdrawal or transition to a state that requires a separate withdrawal-initiation instruction, depending on compute/account constraints;
11. store `prepared_at` from Clock.

No second cycle may be prepared until this cycle finishes.

### 9.4 Delayed withdrawal initiation

The exact Jito withdrawal operation may be part of `prepare_distribution` or a separate `initiate_withdrawal` instruction.

Decision rule:

- combine only if account count, compute, signers, PDA stake-account creation, and auditability remain safe;
- otherwise keep it separate and make it permissionless.

The active distribution state must make duplicate withdrawal impossible.

### 9.5 Cooldown monitoring

A keeper/CLI checks the withdrawal stake account state.

The program must validate readiness from on-chain stake state; it must not trust the keeper's assertion.

### 9.6 `finalize_distribution`

Permissionless instruction. Caller pays transaction fees.

Preconditions:

- correct active distribution ID;
- withdrawal stake account is fully inactive/withdrawable if one exists;
- recipient addresses match stored configuration;
- enough SOL is available for the fixed net distribution;
- no arithmetic or liability mismatch;
- program not paused, unless an explicit recovery finalization is authorized by upgraded code.

Actions, atomically:

1. withdraw SOL from the inactive stake account into the distribution escrow, if needed;
2. determine actual net SOL available after protocol withdrawal costs;
3. allocate net outgoing SOL proportionally between HTFP, Team Owner, and KIF based on gross weights 59 / 19.5 / 2;
4. transfer HTFP SOL;
5. transfer Team Owner SOL;
6. credit individual KIF claim balances for active guardians;
7. reconcile pending contribution queues into the next principal baseline;
8. move/stake eligible excess SOL into JitoSOL directly or leave it pending for a separate permissionless `stake_pending_sol` instruction if transaction complexity requires;
9. finalize the new protected-principal high-water mark;
10. close or reclaim temporary withdrawal accounts where safe;
11. mark the distribution complete and return to `Idle`;
12. emit complete events.

If any required action fails, no beneficiary must be paid twice and the distribution state must remain recoverable.

### 9.7 Technical minimum

No arbitrary economic minimum is required merely because the caller might lose money on transaction fees.

However, delayed withdrawal to a stake account may impose a mandatory technical minimum caused by:

- rent-exempt stake-account balance;
- current cluster minimum stake delegation;
- Jito/SPL pool withdrawal constraints;
- protocol fees;
- integer conversion floors.

Phase 0/1 must measure this. If pending SOL does not cover the outgoing obligation and the required Jito withdrawal is below the technical minimum, `prepare_distribution` must not lock PIV1 in an unfinishable cycle. Yield simply continues accumulating.

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

PIV1 V1 permits only one successfully prepared distribution at a time. Parallel withdrawal stake accounts and overlapping distribution snapshots are excluded from V1 to keep principal accounting and recovery paths auditable.

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

Its only intended purpose is to pre-fund rent-exempt temporary accounts required by the Jito delayed-withdrawal lifecycle. When a temporary withdrawal stake account is fully withdrawn and closed, its recovered rent lamports return to this operational reserve and may fund the next temporary account.

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

Provisional KIF period: 30 days.

A guardian is active for a period if it signs an on-chain guardian heartbeat/attestation or participates in a qualifying governance vote during that period.

The simplest V1 mechanism is a dedicated permissionless `guardian_heartbeat` instruction requiring the guardian signature and recording `last_active_period`.

The program must not depend on an off-chain database to decide KIF eligibility.

### 11.3 Allocation when at least one guardian is active

The full available KIF allocation is divided equally among active guardians.

- inactive guardians receive zero;
- no retroactive catch-up;
- rewards are credited to claimable balances;
- guardians may accumulate balances before claiming;
- division is floored;
- dust stays in the KIF accounting reserve or returns to PIV1 according to the final invariant chosen during implementation.

### 11.4 Allocation when zero guardians are active

Let `available_kif` be the current cycle's 2% KIF allocation plus any prior carry.

Provisional exact implementation of the founder's rule:

- `compound_from_kif = floor(available_kif / 2)`;
- `kif_carry_next = available_kif - compound_from_kif`.

The compounded half permanently increases protected principal. The carried half is added to the next KIF allocation to incentivize guardians to become active.

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

## 13. Proposed on-chain account model

Names are provisional. Codex must validate size, ownership, PDA seeds, rent, and close behavior.

### 13.1 `PivConfig` PDA

Stores:

- version;
- bump;
- paused flag;
- Jito cluster configuration references;
- JitoSOL mint;
- principal token vault;
- SOL deposit queue;
- JitoSOL deposit queue;
- distribution escrow;
- recipient addresses;
- split constants;
- minimum interval;
- last prepared timestamp;
- next distribution sequence;
- current distribution state reference;
- protected principal lamports;
- relevant accounting totals;
- KIF period configuration;
- guardian registry reference;
- reserved future-migration fields.

### 13.2 `PrincipalJitoVault`

PDA-owned SPL token account holding principal JitoSOL.

### 13.3 `PendingJitoVault`

PDA-owned SPL token account receiving unreconciled JitoSOL contributions.

Separating pending and principal token accounts is strongly recommended for unambiguous accounting.

### 13.4 `PendingSolVault`

PDA/system account receiving SOL contributions pending reconciliation.

Operational rent/ownership details must be validated. The design must distinguish economic SOL from rent-exempt lamports.

### 13.5 `DistributionEscrow`

PDA/system account holding liquid SOL assigned to the active distribution.

### 13.6 `DistributionState` PDA

One active sequence at a time. Stores:

- sequence ID;
- state enum;
- prepared timestamp/slot/epoch;
- old protected principal;
- historical principal token amount/value inputs;
- gross yield;
- gross allocations;
- pending SOL used;
- required JitoSOL withdrawal;
- withdrawal stake-account address;
- expected/actual net SOL;
- active KIF guardian bitmap or immutable eligibility snapshot;
- KIF carry inputs;
- final payment/accounting flags.

### 13.7 Withdrawal stake account

A temporary stake account used for Jito delayed withdrawal.

Phase 0/1 must prove:

- how it is created;
- whether a PDA address can safely be used;
- required System/Stake Program instructions and signatures;
- stake and withdraw authorities;
- how deactivation occurs;
- how final withdrawal occurs;
- how rent is recovered;
- how duplicate/replacement accounts are prevented.

The PIV PDA should control the stake and withdraw authorities where technically supported.

### 13.8 `GuardianRegistry` PDA

Stores six guardian public keys and KIF timing configuration, or mirrors the Squads member set through an explicit synchronized configuration.

Do not assume the PIV program can cheaply query arbitrary Squads state without validating the integration. It may be safer to store guardian keys in PIV config and require 4/6-authorized updates.

### 13.9 `GuardianReward` PDAs

One account per guardian storing:

- guardian pubkey;
- last active period;
- claimable lamports;
- cumulative earned;
- cumulative claimed;
- bump/version.

Six fixed accounts are acceptable and avoid loops over an unbounded holder set.

---

## 14. Proposed instruction set

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
- optional `initiate_delayed_withdrawal`
- optional `deactivate_withdrawal_stake`
- `finalize_distribution`
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
10. A withdrawal cannot be initiated twice for one distribution.
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

---

## 17. Threat model summary

### External protocol risks

- Jito/SPL stake-pool bug or exploit;
- Jito program/account upgrade or deprecation;
- incorrect official accounts supplied to CPI;
- delayed withdrawal behavior changes;
- fee changes;
- stake-account state delays;
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

### 18.4 Devnet integration tests

Use the current official Jito Devnet program, stake pool, and mint only after revalidation.

Required demonstrations:

- direct SOL deposit into Jito Devnet pool;
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
- disposable Devnet keys;
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
│           │   ├── initiate_withdrawal.rs
│           │   ├── finalize_distribution.rs
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
│   ├── deploy-devnet.sh
│   ├── verify-build.sh
│   └── inspect-accounts.sh
├── docs/
│   ├── PIV1_MASTER_SPEC.md
│   ├── PIV1_DECISIONS.md
│   ├── PIV1_INVARIANTS.md
│   ├── PIV1_THREAT_MODEL.md
│   ├── PIV1_TEST_PLAN.md
│   ├── PIV1_DEVNET_RUNBOOK.md
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

No product code.

- inventory VPS;
- initialize private Git repository;
- pin toolchain versions;
- verify official Jito Mainnet/Devnet accounts;
- inspect Jito reference implementation and SPL stake-pool source;
- prove direct deposit/withdraw transactions manually on Devnet;
- measure technical withdrawal minimum;
- determine PDA stake-account creation/authority method;
- document findings and conflicts.

Gate: founder/chat approves Phase 0 report.

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

### Phase 4 - Devnet

- deploy PIV1 Devnet;
- use official current Jito Devnet deployment;
- execute complete cycles;
- verify events/accounting;
- CLI and runbook;
- failure/retry tests.

Gate: Devnet validation report.

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

Passing Devnet tests alone is not sufficient authorization for Mainnet.

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
- the program has been tested locally/Devnet according to published evidence.

---

## 24. Remaining non-blocking/open items

The project can begin Phase 0 without more founder product decisions.

Items to resolve through technical validation or later configuration:

1. exact current delayed-withdrawal minimum;
2. exact stake-account PDA creation and authority flow;
3. whether preparation and withdrawal initiation fit safely in one transaction;
4. whether final withdrawal, transfers, and KIF credit fit safely in one transaction;
5. exact KIF period implementation and Clock boundary rules;
6. exact treatment of KIF division dust;
7. exact acronym expansion for KIF;
8. final Program ID;
9. final six guardian public keys;
10. final temporary recipient addresses;
11. pinned toolchain/dependency versions;
12. exact operational/rent reserve accounting;
13. whether `stake_pending_sol` is separate from finalization.

The dedicated development chat should resolve technical items from evidence and ask the founder only when a genuine product/security choice remains.

---

## 25. Current official integration references to reverify

As checked on 2026-08-03, official Jito documentation lists distinct current Devnet deployment addresses and provides direct stake/unstake integration examples using the SPL stake-pool library. These values are not hardcoded here as timeless truth; the Phase 0 report must retrieve and verify them again before use.

Official source families to use:

- Jito Foundation JitoSOL documentation;
- Jito stake/unstake reference repository;
- Solana official program, stake-account, Clock, account, and verified-build documentation;
- SPL Stake Pool source/documentation;
- Anchor official documentation;
- Squads official documentation;
- OpenAI Codex official documentation.

No blog, aggregator, or AI-generated code should override primary-source behavior.
