# M4 closure review — P-023 withdrawal costs

Status: **SCOPED REVIEW COMPLETE / PRODUCTION-RUNTIME FEE GAP OPEN** (2026-10-18 UTC).
Analyzed integration HEAD: `2a852d210e009ac8ccbaf25c7b20f87ed644c11b`.
Scope: one requirement and the directly relevant P-026 fee/minimum context.
All source and canonical-document line references use the analyzed HEAD.
This is source/evidence inspection; no new test, build or runtime execution.
Separate refutation receipts are `reviewer-code-evidence.json` and
`reviewer-spec-evidence.json` under `/tmp/piv1-m4-fees-review-20261018-a`.

## Requirement and verdict

> Jito withdrawal/protocol costs reduce the outgoing beneficiary allocation, not protected principal or the 19.5% compound share.
> — `PIV1_DECISIONS.md:42`, P-023, CONFIRMED

Master specification §3.1 (`PIV1_MASTER_SPEC.md:425–428`) additionally requires
withdrawal fees and conversion floors to be calculated separately for each leg
and then accumulated. P-026 (`PIV1_DECISIONS.md:45`) makes the technical minimum
current-fee-, Rent-, Stake-minimum- and source-dependent.

**Implementation verdict: implemented within the supported active-source profile;
confidence high after separate code/evidence and documentary refutation.** Nonzero withdrawal fees are
explicitly calculated, transferred and accumulated. No zero-fee withdrawal guard
or demonstrated allocation defect was found. Principal deposits have a distinct,
accepted zero-fee/book-value-preserving restriction; this does not constrain the
withdrawal path. This verdict does not promise acceptance of every fee/source
configuration or prove actual nested execution from host callbacks.

**Runtime verdict: positive withdrawal-fee closure remains unproved by the
inspected lifecycle evidence.** The current production fixture initializes all
pool fees to numerator zero/denominator one; successful delayed-leg profiles
therefore do not witness a positive manager-fee transfer followed by reduced net
beneficiary settlement. General fee support is not inferred from zero-fee runs.

## Enforcement and paths walked

All line references below are relative to `programs/piv1/` at the analyzed HEAD.

1. `src/instruction_boundary.rs:86–104` dispatches production preparation,
   initiation, finalization and settlement to their real handlers.
   `src/distribution_preparation_execution.rs:228–261` fixes gross split first,
   funds outgoing from pending SOL/prior yield, and passes only the remaining
   outgoing budget to the withdrawal derivation. Its proposed HWM includes
   permanent compound and conservative dust; residual coverage is required.
2. `src/integrations/jito_withdrawal_preparation.rs:19–53,65–77` computes
   `fee(q) = ceil(q * numerator / denominator)` and
   `redeem(q) = floor((q - fee(q)) * total / supply)`. The fixed target fits the
   **gross book-value budget**, rather than buying extra principal tokens to
   restore fee-reduced proceeds. The round minimum reserves separate ceiling
   fee/floor effects for possible legs. `active_source:86–133` applies current
   Rent, real queried delegation minimum and source residual/capacity checks.
3. `src/integrations/jito_withdrawal.rs:19–68` quotes each actual leg with the
   fresh fee/ratio, stores fee plus burn separately, and checks minimum output,
   remaining-round feasibility and residual HWM after the whole fixed target.
   Unsupported or infeasible combinations fail closed; there is no general
   future fee/liquidity guarantee. `instruction:74–82` encodes protected pool
   variant 24 with the authenticated authorities and minimum output.
4. `src/withdrawal_leg_execution.rs:219–280` feeds the actual quote to the state
   transition, debits principal token custody by **fee + burn**, credits manager
   token custody by **fee**, and reduces pool/Mint supply by **burn only**.
   `dispatch:61–82` verifies complete expected records after each CPI, rereads
   authenticated pool/custody facts and rechecks the outlook before committing.
   `src/state/transitions.rs:550–575` checks fee + burn = assigned input and
   accumulates the per-leg amounts with checked arithmetic.
5. `src/withdrawal_finalization_execution.rs:259–280,308–310` separates original
   rent, delegated proceeds and cooldown reward/loss. In
   `src/state/transitions.rs:669–740`, finalized delegated proceeds accumulate
   independently of rent/reward; losses or deficient residual coverage enter
   recovery. Settlement waits for the complete assignment/finalization state.
6. `src/settlement_execution.rs:234–254` and
   `src/state/transitions.rs:807–839` derive eligible net from pending SOL used,
   prior yield used and finalized delegated proceeds, capped at gross outgoing.
   `src/state/distribution.rs:51–103` allocates that net by relative weights
   5900/1950/200 out of 8050, capped at each gross obligation; dust is retained.
   `transitions.rs:880–901` adds the unchanged proposed compound HWM delta plus
   retained dust/zero-active KIF compound and enters recovery if real protected
   value falls below it. `settlement_execution.rs:158–165` derives that value
   from current custody/ratio, excluding cooldown reward and pending SOL used.

Thus manager fees reduce each leg's delegated proceeds and then beneficiary net;
they are not subtracted from the fixed compound proposal or paid by advancing
additional protected principal. A debit from the physical principal token vault
is not itself a principal loss: that vault also holds the fixed outgoing target.

## Host evidence and runtime limits

The following tests were inspected, **not rerun**. Their previous passing results
remain retained evidence, subject to root's independent source/receipt binding.

- `tests/withdrawal_preparation_execution.rs:170–175` accepts a 1/1000 fee;
  `src/integrations/jito_withdrawal_preparation.rs:142–178` checks fee-aware
  minimum inversion and split rounding bounds, including impossible fee/floors.
- `tests/withdrawal_leg_execution.rs:57–70,86–102,207–222` covers nonzero/zero
  fees, shared/distinct fee roles, two sources and a bounded numeric domain.
  `tests/support/withdrawal_preparation_custody.rs:13` sets 1/1000 by default;
  the independent host oracle charges ceil fee and checks manager/burn deltas
  (`tests/support/withdrawal_leg_custody.rs:66–81,131–141`).
- `tests/settlement_execution.rs:74–88` includes withdrawn profiles originating
  from that fee-bearing host fixture (`tests/support/settlement_custody.rs:20–31`)
  and independently computes net shares/HWM (`:81–97`). These are modeled host
  effects, not actual Token/pool/Stake nested execution.
- Actual Task 2.51 O and V execute two delayed legs; S/W are liquid cycles, U
  stops after sufficient preparation, N/P/Q cover loss/recovery. Their retained
  reports do not claim a positive withdrawal fee. The exact source construction
  in `validation/genesis-bank-runtime/tests/lifecycle_fixture.rs:33–41` sets all
  fees to zero. The two-leg oracle in `production_lifecycle.rs:420–435` debits
  input equal to burn and leaves manager tokens unchanged. The O independent
  receipt at `/tmp/piv1-t251-severe-loss-review-20261013-a/reviewer-runtime-o.json`
  reports 187 snapshots/101 transaction pairs and the expected leg/settlement
  account oracles. A separate read-only parser decoded every retained O/W pool
  account record: respectively 187/77 observations all have withdrawal fee 0/1;
  whole stdout hashes match their retained receipts. Evidence is
  `/tmp/piv1-m4-fees-review-20261018-a/reviewer-retained-pool-fees.json`.
  This is new inspection of old runtime, not new runtime execution.
- Historical Task 0.4 experimental PIV PDA CPI proof did execute a positive
  one-leg withdrawal fee:
  756,046 units on 756,045,269 input (`PHASE_0_VALIDATION_REPORT.md:986,1297`).
  It does not execute the current PIV production fee-through-beneficiary-
  settlement lifecycle. Therefore the gap is not an absence of all historical
  nonzero-fee runtime evidence.

## Search record and next action

`rg` searches for `withdrawal_fee`, `fee`, `protocol_cost`, `beneficiar` and
`derive_net_beneficiary_allocation` across production integrations, withdrawal
handlers, settlement, state and corresponding host tests located the paths
above. `stake_withdrawal_fee` found fee-aware production quote/preparation and
host tests, rather than a nonzero-fee rejection. Fee rejection in
`src/state/principal_deposit.rs:176–177` is deposit-specific. Lifecycle fixture
fee initialization and current leg oracles were inspected; no existing positive
withdrawal-fee runtime witness was located in the current Task 2.51 profiles.

The narrow follow-up is one supported **positive withdrawal-fee** local lifecycle
through actual protected leg execution and beneficiary settlement, preferably
reusing the existing two-leg fixture. Independently derive per-leg ceil fees,
burns, manager credits, net proceeds, beneficiary reductions and unchanged
compound/HWM protection; retain rent/pending/KIF distinctions and existing
rollback/retry gates. No new economic decision or production change is justified
by this review alone. This addresses P-023/master §3.1 and §9.12 plus the Test
Plan M3 exact fee mapping and M4 requirement closure; it is not a task for every
possible fee regime. Task 2.51's original bounded zero-fee staking/positive
profile remains valid evidence, not a failed run or retroactively false claim.

**notChecked:** exhaustive fee/source combinations, live Jito configuration,
fee-bearing principal deposits, complete P-026 conformance, external protocol
upgrades, all other requirements, signatures/transport, governed recovery exit,
and professional audit. Stopped internal-failure drafts were not inspected,
resumed or rerouted and remain unwaived. M4 is incomplete; D-034 still requires
stopping before M5, which remains unauthorized. Main publication and founder
acceptance are separate. D-027 (`PIV1_DECISIONS.md:158–192`) accepted bounded
foundations, not full lifecycle closure or a P-023 waiver; D-030 (`:258–307`)
does not automatically promote every retained limitation into a prerequisite.
No implementation, tests, builds, Git mutations or live operations were performed
by this delegated review.
