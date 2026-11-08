# Minimal same-round RecoveryRequired exit proposal

**PROVISIONAL / UNAPPROVED — implementation decision only.** No recovery authority,
economic change, deployment, migration, funding or execution is approved by this
document. No confirmed decision ID is assigned. Recommendation: retain the frozen
round, collect its existing withdrawal legs safely, and permit one narrowly
governed resumption only when real custody and protected-value checks pass.
Otherwise remain blocked. Do not implement cancellation in the first version.

Baseline: `831f780dd2a98f682845136693324d77a64c1aa8`,
`codex/piv1-hybrid-liquidity`, 2026-11-08 UTC. The last retained public observation
at slot461935674 shows Idle V2, not RecoveryRequired. This is source-based design,
not a report of a current chain incident. The last verified active Testnet ELF is
`047c819b717bb0b74602596297a02bf5d84deaec327241a9981926fe144696ac`.
Founder task directive SHA256:
`7e58b8de3ab3b267a1273bdd57af4c6816996e7da4f7c10b325e10b02b85fd08`.
Evidence, synthetic worksheet, source bindings and independent review are retained
in `/home/jerem/piv1-pilot-checkpoints/recovery-required-design-20261108-a/`.

## 1. Actual behavior and the missing capability

The only committed entry sites found are below. Flags are defined in
[constants.rs](../programs/piv1/src/constants.rs#L25): cooldown loss `0x01`, residual
HWM failure `0x02`; both may occur together.

| Entry / exact condition | Commit, assets and remaining work |
| --- | --- |
| [`finalize_withdrawal_leg`](../programs/piv1/src/state/transitions.rs#L651): observed net stake value, after exact stake-rent exclusion, is below the leg's original delegated value (`cooldown_loss_lamports > 0`); flag0x01. Alternatively/currently also, current residual value after excluding the **entire fixed withdrawal target** is below `stored_residual_hwm_floor_lamports`; flag0x02. | Returns **Ok(RecoveryRequired)**. [Runtime finalization](../programs/piv1/src/withdrawal_finalization_execution.rs#L67) can complete successfully: withdraw the fully inactive canonical stake to escrow, return exact stake rent to Operational, commit Config/round aggregates, close leg metadata and return metadata rent to Operational. Metadata excess is recognized as pending, not repair funding. Economic net native value remains in escrow. Other initiated legs can still be active/cooling/inactive; some frozen target can remain unassigned. |
| [`settle_distribution`](../programs/piv1/src/state/transitions.rs#L915): projected post-payment protected value is below the exact proposed settled HWM, including protected compound/dust and the zero-active KIF rule; flag0x02. | Returns **Ok(RecoveryRequired)**. [Runtime branch](../programs/piv1/src/settlement_execution.rs#L65) commits **only the round**, emits recovery, executes no beneficiary/KIF transfer, and changes no Config HWM or reward credit. Escrow remains funded as observed; the eligible withdrawal target must already be complete for normal EscrowFunded admission. Liquid/swap-funded rounds have no withdrawal legs. |

For either entry, an actual subsequent CPI, postcondition, account or persistence
error rolls back the complete transaction. A successful recovery-recording receipt
is therefore different from a failed transaction. Finalization's runtime
[closure](../programs/piv1/src/withdrawal_finalization_execution.rs#L337) destroys
the completed leg metadata and empties the stake account: recovery cannot demand
that all old finalized metadata accounts still exist. Authenticate current legs;
preserve trusted monotonic aggregates and actual events/receipts for closed legs.

The [round validator](../programs/piv1/src/state/distribution.rs#L551) requires
RecoveryRequired to have valid nonzero flags and no settlement effects. Its
[withdrawal identity](../programs/piv1/src/state/distribution.rs#L820) also requires
`cumulative_cooldown_losses > 0` iff flag0x01 is set. Normal lifecycle states
currently require zero recovery flags. Consequently **clearing flags while
retaining a real loss is invalid**, even before the escrow identity is checked.
No hidden recovery mutator or unexposed exit was found in
[native dispatch](../programs/piv1/src/instruction_boundary.rs) or public
[state transitions](../programs/piv1/src/state/transitions.rs).

| Situation | Current interpretation, not an interchangeable recovery reason |
| --- | --- |
| Cadence, stale Jito epoch, inactive-stake prerequisite, technical minimum, missing source, insufficient rent | Ordinary unmet prerequisite/error; no committed recovery entry merely because it is refused. Preparation/leg-initiation HWM errors also roll back. Pending integration may fail while remaining Settled; it does not create a third recovery entry. |
| Approved withdrawal fee / conversion rounding; valid D108 swap costs | Already bounded and reflected in ordinary conditional NET calculation. Not a cooldown loss. A bad swap/price/CPI rolls back; D108's5% bound does not authorize any cooldown-loss absorption. |
| Recorded cooldown loss | Actual economic shortfall, preserved in leg event/round counters; never repaired by relabeling or reducing HWM. |
| Wrong custody, owner, account, receipt or cumulative identity | Error/rollback; no generic committed custody-deficit recovery flag. Proposed recovery must refuse unexplained discrepancies. |
| Off-chain UNKNOWN | Transaction evidence problem, not an on-chain lifecycle flag. Resolve the original signature/status; never adopt/replay/replace it through recovery. |

Currently new preparation, initiation, further finalization, settlement, pending
integration and ordinary principal staking are blocked by their lifecycle/actual
guards; further finalization is blocked even for an otherwise ready remaining
leg. Movement-bearing economic normalization is also blocked. Separately:

- Earned isolated KIF claims remain possible under their existing owner/backing
  rules, including pause; they cannot modify the round.
- Already-received pending transfers remain reconcilable during pause/recovery.
  Explicit contributions are possible when unpaused, but remain pending.
- Heartbeats and existing exact governed actions retain their own checks.
  [Pause/unpause](../programs/piv1/src/guardian_operations.rs#L119) only changes
  `Config.paused`; it does not clear recovery, losses or lifecycle.

References: [reconciliation](../programs/piv1/src/state/reconciliation.rs#L251),
[contributions](../programs/piv1/src/contribution_execution.rs#L135),
[claim authentication](../programs/piv1/src/kif_claim_accounts.rs),
[principal staking](../programs/piv1/src/principal_deposit_execution.rs).

## 2. Confirmed boundaries and missing decisions

| Status | Authority / consequence |
| --- | --- |
| CONFIRMED | [P017–P020/P035](PIV1_DECISIONS.md), [spec§9.11](PIV1_MASTER_SPEC.md#911-cooldown-rewards-losses-and-rent): official Jito valuation, conservative floors, no normal HWM decrease; a real cooldown loss records recovery. A historical deficit remains visible until assets genuinely recover. |
| CONFIRMED | P008/P021: permanent19.5% compound and retained dust remain protected. Preparation stores a **proposed** increase; Config HWM increases only on successful settlement. Do not write it early or erase the reserved protection because Config still shows the old HWM. |
| CONFIRMED | P013–P016: full recognized contribution increases principal on integration, including SOL used earlier from pending to fund escrow. Later contributions remain separate. Assets and required HWM rise together; ordinary contributions cannot repair a historical deficit. |
| CONFIRMED | P023/P024 and [NET code](../programs/piv1/src/state/distribution.rs#L51): frozen gross calculations/destinations lead to conditional, conservatively rounded NET payments after approved costs. No partial beneficiary payment is committed. `outstanding_active_round_liability` reserves the frozen round; its name does not make frozen gross equal to individually earned KIF. Cancellation is not thereby authorized. |
| CONFIRMED | K006–K010/K012/D061, [spec§11.5](PIV1_MASTER_SPEC.md#115-claims): **previously credited** KIF is already-earned debt with preserved owner/counters. This round's frozen KIF gross/bitmap is a conditional input, credited only by successful settlement. Keep prior collective carry backed and follow the frozen period/eligibility; later activity or member replacement cannot redirect it. |
| CONFIRMED | P033–P035: escrow economic SOL, rent advances/refunds, cooldown rewards and operational funds are separate. Cooldown rewards are next-cycle yield, excluded from this round's NET and protected-value repair test. Rent is never loss compensation or yield. |
| CONFIRMED decision / PROVISIONAL Testnet policy | [D107/D108](PIV1_DECISIONS.md#d-108--confirmed--five-percent-combined-realized-swap-loss-ceiling-and-local-hybrid-implementation): exact hybrid routing and combined swap-only500bps ceiling, official-book debit and actual swap output. Preserve its receipt; do not apply this tolerance again to cooldown losses or failed custody. |
| CONFIRMED pattern; new action OPEN | K002 specifies4-of-6 for existing governance/program modifications. Actual [Squads authentication](../programs/piv1/src/squads_execution.rs) checks current authority, proposal, stored action and approvals. Demonstrated upgrades do **not** approve new loss allocation or an unrestricted recovery action. Extend that authority only by the explicit decisions below. |
| HISTORICAL / unchanged | Completed D109/D110, KIF simulations, old grants/HOLDs and artifacts stay consumed/source-scoped. They grant no recovery execution. |
| OPEN | Which conditional category absorbs recorded cooldown losses, exact recovery permission/authority, and any cancellation/compensation treatment. No existing decision resolves those by implication. |
| REJECTED by confirmed boundaries | HWM reset, consuming principal/compound/earned KIF/rent, redirecting recipients, classifying contributions as yield, arbitrary account edits, permanent buffer/emergency fee/repayment obligation. |

Consulted available local **Solana Dev** skill and **Solana CPI Safety** guidance
on post-CPI fresh reads, fixed authenticated callees and canonical PDA signing:
`/home/jerem/.codex/skills/solana-dev/SKILL.md` and
`/home/jerem/.codex/skills/solana-cpi-safety/{SKILL.md,account-reload.md,pda-cpi-signing.md}`.
Their generic SDK/migration preferences do not override project pins/authority.
**HTFP Master Context unavailable in this task:** no `HTFP_MASTER_CONTEXT.md` was
found in the repository/local skill catalog or bounded Codex filename search,
and no HTFP-context provider is exposed by the available tools. Its content is
not inferred. This is a reference limitation, not a waiver of project decisions.

## 3. Compare only the material choices

| Choice | Assessment / recommendation |
| --- | --- |
| A: resume the same frozen round after truthful reconciliation | **Recommended, PROVISIONAL.** Preserve history, contributions, cadence, recipients and KIF ownership. Natural valuation recovery may cure a residual-only condition. Recorded losses require explicit loss-aware NET policy plus durable resolution accounting; collection cannot be skipped. |
| A subset: resume residual-only with zero recorded cooldown loss | Smaller code, potentially no loss-ledger extension, but leaves every genuine cooldown-loss round without an exit. Documented alternative, not the recommended complete bounded design. |
| B: cancel/end and return Idle | **OPEN, not recommended for this version.** Must separately define fate of frozen obligations, used pending value, partially burned target, compound/dust, sequence/cadence and outstanding stakes. Conditional gross is not permission to confiscate it. No automatic cancellation, delete, refund/depositor withdrawal or Idle reset. |
| C: remain blocked | **Required fallback.** Applies to insolvency, unexplained custody, impossible remaining target, zero allocated NET or absent governance. An indefinite safe stop is more truthful than a synthetic completion. |

## 4. Recommended bounded implementation contract — all PROVISIONAL

### 4.1 Persistent facts and two narrow operations

Use a **versioned V3 extension of the existing round**, preserving the complete
V2 frozen body/swap receipt and all factual loss/flag counters. Add a bounded
recovery receipt with a monotonic generation, unresolved/acknowledged status,
acknowledged cumulative loss, frozen-snapshot commitment and last approving
Squads transaction index/Clock. No arbitrary recipient, transferable capability,
new custody wallet or generic editing instruction. Exact byte sizing/ABI/rent
are implementation deliverables after approval, not claimed measured here.

V3 validation must distinguish historical incident flags from unresolved
recovery. Do not merely allow any nonzero flags in normal states: continuation
requires an exact acknowledged receipt/generation/cumulative-loss identity.
V2 decoding remains unchanged. A separately approved governed V2→V3 migration
may preserve an Idle or internally valid RecoveryRequired body byte-for-byte
except version and appended receipt, initializing an existing incident unresolved.
It cannot rewrite losses/custody or declare recovery resolved. An Idle-only
migration would otherwise strand an already-active V2 recovery incident.

1. **Collect existing recovery leg**, permissionless, **unpaused only**:
   `RecoveryRequired → RecoveryRequired`. Reuse the exact existing leg/stake
   PDA, owner/authority/vote, actual inactivity/Clock/Stake History, fixed Stake
   withdrawal, escrow/rent destinations, fresh post-CPI receipts, closure and
   atomicity. Process each already-initiated leg once as it becomes inactive;
   no mandatory index-order wait, new leg, new burn, reassignment, beneficiary
   payment or caller reward. Preserve all flags; add actual losses/rewards and
   counts, never use rewards/rent as repair. A new loss or fresh residual failure
   invalidates any previous acknowledgement/generation. Pause still blocks it.
2. **Resume exact recovered round**, governed and unpaused, with **no transfer**:
   - `RecoveryRequired → EscrowFunded` only if the fixed target is fully assigned
     and all initiated legs finalized; liquid/swap rounds have zero target/legs.
   - `RecoveryRequired → WithdrawalActive` only for a positive unassigned
     remainder, after **all currently initiated legs have been collected**, with
     feasible current canonical source/technical-minimum/maximum-useful-leg
     checks and restored full-target residual protection. Resume only the same
     remaining target under existing permissionless initiation/finalization;
     never increase it, choose a new route or promise future liquidity/prices.
   - Otherwise remain RecoveryRequired. Do not require a fully assigned target
     merely to allow the second destination: that would deadlock partial rounds.

### 4.2 Explicit loss allocation, not a fictitious repair

Propose assigning **only the authenticated actual cooldown loss** to this round's
still-conditional outgoing pool, maintaining its existing relative NET weights.
This is a **new economic decision**, not already authorized by P023 or D108.
Require loss no larger than frozen outgoing and a strictly positive **sum of
actual floored/capped allocations**; a positive numerator alone is insufficient.
Each governance action acknowledges the exact observed cumulative loss, with no
unused discretionary loss allowance and no percentage inherited from swapping.

Let `P`=pending SOL already used, `C`=prior carry used, `D`=finalized delegated
native value before cooldown, `L`=actual cumulative cooldown loss, `Q`=cooldown
rewards, `S`=actual swap SOL output, and `O`=frozen gross outgoing. All are exact
round receipts; normal protocol fees already reduced `D` and must not be charged
again. Use checked arithmetic:

```text
actual economic escrow = P + C + D - L + Q + S
eligibleRecovery      = P + C + D - L + S
netBase               = min(O, eligibleRecovery)
HTFP = min(frozen HTFP, floor(netBase * 5900 / 8050))
TEAM = min(frozen TEAM, floor(netBase * 1950 / 8050))
KIF  = min(frozen KIF,  floor(netBase *  200 / 8050))
allocatedNET = HTFP + TEAM + KIF > 0
netDust = netBase - allocatedNET
retainedConservativeDust = eligibleRecovery - netBase
escrow after settlement = netDust + retainedConservativeDust + Q
```

This fixes the current mismatch between delegated-value NET eligibility and the
physically loss-reduced escrow. It changes no frozen gross/percentage/recipient.
Loss enters the cumulative eligibility expression **once**, never once per
resumption; the receipt acknowledges it, supplies no SOL and forgives no HWM.
Update every V3 settlement/round/custody identity consistently, preserving the
unchanged V1/V2 paths. Earned old KIF debt and prior carry are never debited to
absorb this loss; new credits follow the frozen KIF bitmap/50:50 zero-active rule.

### 4.3 Fresh on-chain admission and real solvency

Authenticate canonical initialized/rent-backed Config/round/custody/registry,
current official Jito pool/mint/epoch and conservative valuation. All actual
economic balances must match reconciled obligations, including full pending,
old KIF liability/carry and exact operational rent. Unexplained donations/deficits
are not a new baseline or repair receipt.

For EscrowFunded, derive the exact prospective normal settlement and integration:
preserve old HWM plus permanent compound, conversion/split/NET dust, retained
conservative dust and the existing zero-active KIF compound, then add the **full**
recognized contribution at integration. In particular, the existing
[protected-value oracle](../programs/piv1/src/settlement_execution.rs#L171) excludes
`Q` and subtracts `P`; escrow-funded pending is not free principal. Require all
these guards, not simply enough escrow to pay beneficiaries. No balance is
invented or saturated to satisfy a transfer/HWM check.

For WithdrawalActive, preserve the existing full-fixed-target residual oracle
and strengthen admission with the explicit pending-replacement bound:

```text
R_full = historicalNative
       + book(historicalJito - ENTIRE fixedTarget)
       + unusedPriorCarry
R_full >= proposed_hwm_after_settlement + pending_SOL_used
```

Use current authenticated official accounting; this is not merely the old stored
residual floor. For example, proposedHWM1001950/P1000/R_full1002450 passes the old
floor but remains500 short and must refuse. NET/retained dust and zero-active KIF
compound add equally to protected assets and HWM; they cannot bridge this gap.
Require a currently feasible bounded remaining source/technical minimum and
**positive actual floored NET from already collected real funding**. Do not count
future delegated output, book target value or future cooldown rewards as cash.
This conservative first version stays blocked if that cash condition fails,
even if a later withdrawal might succeed. Every later leg, settlement and
integration revalidates actual state; another loss or valuation failure stops or
records a new recovery incident according to its actual entry path.

The resume itself changes only lifecycle and the bounded acknowledgement receipt.
Subsequent settlement transfers only to the frozen HTFP/Team/KIF destinations,
credits KIF and HWM atomically; normal integration alone creates the completed
summary and Idle. V3 completion **must** preserve the completed sequence, factual
cumulative loss and final acknowledged generation/authority, bound to immutable
events/receipts; current V2 summary has no loss/acknowledgement fields. Do not lose
that history when active counters/receipt are cleared at Idle. A later round may
neither reuse it as authorization nor subtract its loss again. Retain sequence,
`next_distribution_sequence`, old completed
summary, preparation Clock and ten-day anchor. No new preparation, gross yield,
guardian eligibility snapshot, activity rewrite or repayment right is created.

### 4.4 Authority, second incidents and replay

Recommend the existing **current six guardians /4-of-6 Squads** pattern for this
new restricted action, explicitly subject to founder approval. Reuse actual
ProgramData upgrade-authority/current multisig correspondence and authenticated
approved proposal/stored instruction/invocation/timelock checks. A Vault0 signer
flag, old multisig continuity, modelled context or four claimed signatures is
insufficient. Current governance controls approval; frozen recipients/old earned
owners remain the original ones.

Bind the action to Program/round/sequence, exact incident generation, frozen
snapshot digest, observed cumulative loss, collected-leg/assigned-unit counters,
expected escrow and destination lifecycle. Derive all values from authenticated
accounts; expected arguments only constrain, never overwrite them. Validate live
earned claims/pending changes rather than freezing an unrelated whole Config.
Consume the acknowledgement/nonce atomically with the one-shot Squads action.

On **every new incident**, including a residual-only recurrence with unchanged
loss, advance generation and mark unresolved. Old executed or still-pending
approvals cannot become valid after a price rebound. A later loss fitting an old
numeric cap is not approved. No cumulative-loss reset, repeated deduction,
automatic replacement/resume or blind transaction retry.

Emit bounded collection/resumption events: sequence/generation, immutable snapshot
commitment, old/new state, factual flags/loss/reward/rent totals, custody/valuation
and protected floors, acknowledged loss, approving multisig/index and Clock.
Show actual custody and remaining deficit to the operator, plus remaining target,
outstanding legs, pause and original-signature confirmation state. Any failed
CPI/check rolls back that operation; any newly committed loss remains recorded.

## 5. Synthetic accounting examples — not Testnet observations

One integer worksheet was checked; no program/runtime test was run. All figures
below are synthetic lamports, not current quotes, fees, stake minima or rents.
The initial state and A/B use an illustrative1:1 Jito book ratio. In C, after
the native legs have finalized, the **same canonical remaining token units** have
a lower current official-book value of990000; no missing units are assumed.
Initial HWM1,000,000; historical value1,010,000;
gross yield10,000 gives HTFP5900/compound1950/Team1950/KIF200, outgoing8050,
split/conversion dust0. Recognized contribution1000 is already entirely used in
escrow. Withdrawal book debit7050, illustrative approved fees/rounding10, delegated
value7040. Prior carry/rewards0. Old **earned** KIF liability600 remains separately
backed; two of six guardians are active in the frozen new allocation.

| Quantity after collecting all current legs | A: residual restored, no cooldown loss | B: factual cooldown loss | C: historical assets genuinely below HWM |
| --- | ---: | ---: | ---: |
| Principal custody, conservative value | 1002950 | 1002950 | 990000 |
| Pending still in pending custody | 0 | 0 | 5000 new contribution |
| Full contribution to recognize eventually | 1000 | 1000 | 6000 |
| Escrow before settlement | 8040 | 7940 | 7940 |
| Actual cooldown loss | 0 | 100 | 100 |
| Old earned KIF custody/liability | 600 | 600 | 600 |
| Proposed NET: HTFP / Team / new KIF | 5892 /1947 /199 | 5819 /1923 /197 | Same calculation, **payment forbidden** |
| NET rounding dust | 2 | 1 | 1 |
| Prospective settlement HWM | 1001952 | 1001951 | 1001951 |
| Actual protected value after proposed NET, excluding used pending | 1001952 | 1001951 | 989001 |
| Prospective final HWM after full contribution | 1002952 | 1002951 | 1007951 |
| Actual principal after prospective integration | 1002952 | 1002951 | 995001 |
| Remaining deficit to that plan | 0 | 0 | **12950** |

**A:** assume a previously recorded residual-only flag, with valuation now
restored. All frozen/old liabilities and principal are covered. ChoiceA permits a
governed same-round resume with acknowledged loss0. The old600 KIF debt remains;
new KIF199 produces198 earned liability plus1 carry. No compensation is necessary.
ChoiceB is unnecessary and unapproved; choiceC remains safe pending approval.

**B, before collection:** leg1 expected4000 and returned3900, recording loss100;
leg2 still holds3040 economic value and is cooling. Escrow holds1000 pending-used
plus3900 =4900; pending's full economic contribution remains1000, not zero.
Conservation including old KIF: `1010000+1000+600-10-100 =
1002950+4900+3040+600 =1011490`. Synthetic refundable operational rent budget1000
is separately890 in Operational +100 stake rent +10 metadata rent; collection of
leg2 returns110 to Operational and adds3040 economic SOL to escrow. Base account
rent floors are additional unchanged constants, never payments or compensation.

Only after actual inactivity/collection may choiceA acknowledge loss100 and use
the proposed NET rule: escrow7940 pays7939 with1 dust retained; new KIF197 gives
196 earned plus1 carry, **without touching the old600**. Final HWM/principal both
1002951. Existing code cannot do this: flag/counter validation and the old
delegated-value NET identity both reject it. ChoiceB cannot simply erase the
borrowed contribution, remaining stake or frozen conditional calculation.
Without the new policy and implementation, choiceC is the valid behavior.

**C:** even withholding **all** conditional outgoing would leave historical value
`990000+7940-1000 =996940`, below the old1,000,000 HWM by3060. Under the specified
same-round NET plan, the full protected/contribution deficit is12950. The new5000
pending raises assets and required HWM equally and fixes none of that gap.
ChoiceA fails; cancellation cannot repair the assets or silently erase rights.
ChoiceC is mandatory until genuine valuation recovery or separately approved
economic repair. No HWM haircut, rent seizure or fabricated yield is proposed.

**External compensation is excluded from version1.** A possible future explicitly
nonrepayable *loss-repair credit* would need its own classification/receipt,
verified source and funding authorization: it would restore a measured deficit,
not create yield, contribution principal, donor claim or repayment debt. In C,
12950 of correctly classified new assets would cover this illustrative plan;
this is **not funding approval**. Existing direct donations are ordinary pending/
surplus under current reconciliation, not such a repair credit. No special top-up
instruction or reinterpretation is included in this recommendation.

## 6. Minimum founder decisions — proposed answers, none approved

| Decision required before implementation | Recommended exact answer (PROVISIONAL / TESTNET ONLY) |
| --- | --- |
| Economic treatment of a recorded cooldown loss | Authorize the checked cumulative loss-aware NET formula in§4.2 only for a same frozen round, with exact observed-loss acknowledgement, loss<=frozen outgoing, positive actual allocated NET and all protected-value/full-contribution/earned-KIF tests. Reduce only the still-conditional HTFP/Team/current KIF allocation proportionally; preserve fixed gross/owners, compound, HWM, old KIF and rent. No D1085% inheritance, new percentage entitlement or Mainnet policy. |
| Exact new authority and transitions | Extend current6-guardian/4-of-6 authenticated Squads solely to sequence/generation-bound resumption in§4.1–4.4. Permit unpaused permissionless collection of already-initiated canonical legs while staying Recovery; no new initiation while unresolved. Permit governed Recovery→EscrowFunded or Recovery→WithdrawalActive only under the stated distinct gates. Approve a factual versioned receipt, never flag-clear/Idle/arbitrary edits. |
| Unrecoverable cases and scope of development | Keep RecoveryRequired when gates fail. Exclude cancellation, compensation credits, refunds, HWM resets, new buffers/fees/debt and governance overrides from this implementation. Authorize only this bounded V3 policy implementation plus focused local tests; any exact Testnet artifact/upgrade/migration/rent/funding/live trial requires its later separate reviewed approval. All changes remain PROVISIONAL and do not accept Mainnet economics. |

These choices approve **no current transaction**. No new decision ID may be
recorded as CONFIRMED until the founder explicitly approves the exact proposal.

## 7. Implementation and tests only after that approval

Small coherent source scope: round V3 codec/validation and recovery transitions;
existing finalization collection mode; loss-aware settlement/custody derivation;
pending-integration conservation/mandatory versioned completed-loss summary; narrowly extended
Squads action authentication/native dispatch; one existing-client composition.
Retain actual loss history in the V3 completion receipt/summary;
do not discard it when normal integration returns Idle. No generic
disaster framework or new signer/launcher is necessary.

Focused required checks, reusing the existing host/Bank/SBF machinery:

1. Exact current two entry branches, successful recovery recording, actual stake/
   metadata closure/rent, current-balance receipts, and refusal/atomic rollback on
   permitted new-path mismatch tests. Never rerun the platform-stopped M4 draft;
   these tests cannot close or waive that reservation.
2. Multiple existing legs collected once, one still active, no fabricated old
   metadata, wrong PDA/owner/vote/callee/authority, lost rent, donations and pause.
   No new burn or target enlargement during unresolved collection.
3. Exact4-of-6 approved stored recovery action/current authority; wrong action,
   insufficient approvals, mere signer flag, wrong round/snapshot/generation,
   old pending proposal and second incident with identical numeric loss rejected.
4. The three examples, zero/all-floor-zero NET, loss bounds, no double subtraction,
   full pending contribution, old earned KIF/carry/zero-active compound and NET
   dust; protect official-book HWM at settlement **and** integration.
5. Target complete→EscrowFunded versus partial→WithdrawalActive, remaining-source
   infeasibility, fresh Jito epoch/quote, second-loss relatch and no old allowance
   reuse. Preserve normal V1/V2/approved hybrid cost behavior and exact cadence.
6. Versioned migration/receipt integrity and completed summary; authentic local
   pinned external-program execution of collection→governed resume→settlement→
   integration, with compute/resource measurements. Local success is not a public
   Testnet recovery or Mainnet qualification. Reuse unchanged suites; any later
   live trial needs its own finite fresh scope and exact deployment approval.

**STOP after this proposal.** No RPC, keys, signatures, transaction, funding,
program edit/build/deployment/migration or previous-operation replay occurred.
Mission fees17480000, headroom15000, UNKNOWN/HOLD135000, separate reserve370000,
nonce rent1056640 and all prior evidence remain unchanged. Recovery exit remains
**unimplemented and unapproved**; M4 OPEN_UNWAIVED/public M5 NOT_READY remain.
