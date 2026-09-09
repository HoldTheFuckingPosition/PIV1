# Task 2.3 vault reconciliation and custody composition model

Date: 2026-09-08 UTC

Starting baseline: `66193769d1cbc59cd8630df295b9a784b9c64642`

Branch: `task/2.3-vault-reconciliation-model`

Current status: **TECHNICALLY VALIDATED / PENDING FOUNDER ACCEPTANCE**, including
the T23-R1 correction reviewed below. Earlier implementation-stage status and
executor evidence in this report are historical records of their respective runs.

## Sources and implementation boundaries

The founder authorized this bounded host-only task in the current session.
Decisions P-014--P-016 preserve all contribution value through liquid funding;
P-033--P-035 isolate rent and next-cycle rewards/loss; A-005 fixes cumulative
leg accounting; K-012 isolates earned claims; D-024 accepts pending recognition
but explicitly defers custody composition. Master sections 6.2, 7.5--7.7,
9.6, 9.10--9.12 and 10.2, plus Phase 0 section 5.2, govern normalization.
The positive unexplained economic-vault delta rule is already CONFIRMED:
pending contribution, never historical yield. Safe recognition during an
active round is distinct from integration at the accepted completion boundary.

Planned normal commits (linear history, no publication or acceptance):

1. Record these equations and commit boundaries before implementation.
2. Implement pure derivations, atomic host composition, regressions and report;
   validate and commit the bounded result for founder review.

## Equations and physical locations

All SOL equations exclude fixed non-economic account floors. JitoSOL equations
use decoded token units only; token-account rent is separate.
Let P be recognized pending SOL, U be the active round's pending SOL use,
Hsol/Hj be config historical amounts, A be cumulative assigned token input,
C be prior next-cycle yield, Cu its pending-first liquid use, R be new cooldown
rewards, Z be settled zero-active KIF compound, and E be current escrow.

```text
pending SOL obligation = P - U (active); P (Idle)
pending JitoSOL obligation = recognized pending token units (no offset)
principal JitoSOL obligation = Hj - A (active); Hj (Idle)
A = cumulative fee units + cumulative burned units
principal SOL obligation = Hsol + C - Cu + Z (active; Z=0 before settlement)
principal SOL obligation = Hsol + config.next_cycle_yield (Idle)
E before settlement = U + Cu + finalized native - recovered stake rent
E after settlement = recorded escrow - actual paid HTFP - actual paid Team
                   - actual allocation moved to KIF
                 = net allocation dust + retained conservative dust + R
KIF SOL obligation = recorded global earned liability + collective KIF carry
```

Prior carry resides in PrincipalSolQueue separately from historical SOL. Opening
moves only Cu to escrow and stores C in the round before clearing config carry.
The unused C-Cu stays in principal custody and becomes historical at completion.
New R resides in escrow until integration moves it to PrincipalSolQueue while
retaining its next-cycle-yield category. Settlement moves net KIF to KifSolVault
and Z from KIF to PrincipalSolQueue. Old earned claims remain independent.

Integration moves only P-U native SOL, all pending JitoSOL units, and the
settled escrow remainder into principal custody. Its contribution HWM component
is P plus the conservative current value of pending tokens, once. New historical
SOL is observed principal SOL after these moves minus new R; new historical
token units are observed principal units after the pending-token movement.
No historical value or HWM proof is accepted from a test-selected final state.

## Atomic commit and trust boundary

Each host operation clones the complete world, stages source debits and exact
destination credits, derives transition facts from observations, executes the
pure transition, verifies custody and independent conservation, and commits once.
Errors discard the clone. A successful finalization returning RecoveryRequired
commits the recovered custody and recovery state. A settlement recovery outcome
commits only the accepted recovery header and no speculative payments.

Schema, sequence, snapshot, historical counters, pending snapshot, HWM and carry
relationships must validate before any active offset is used. Idle never uses a
completed summary as an offset. The host constructor starts at Idle; later
committed states are reached only by these atomic operations. Pure scalar
helpers cannot authenticate a historical transfer. Future fixed-account, owner,
PDA, mint, signer, Rent, Clock, exact transfer and CPI validation remain deferred.

The reserve baseline is deliberately absent from production derivation:
serialized state does not record initial operational funding and all outstanding
advances. Test-only initial reserve and transaction records can independently
prove rent conservation, but cannot identify arbitrary operational excess in
production. Unsupported operational surplus remains unclassified/unspendable;
this does not block proven economic-vault normalization. A current balance
cannot reveal an earlier unobserved debit replenished by indistinguishable credit.


## Implemented result and evidence boundaries

Status: **IMPLEMENTED / PENDING FOUNDER ACCEPTANCE** for the supported bounded
pure/host scope. Phase 2 remains **IN PROGRESS**. Task 2.4 and later work are
**NOT STARTED**. The first equations commit is `f8b28f1`; the final implementation
commit is the tip of this dedicated branch, reported with the delivery.

The economic destinations, split, integer floors, one-basis-point cap, custody
topology and serialized layouts are unchanged. This task demonstrates executable
custody composition absent from Task 2.2. It does not retroactively claim that
Task 2.2 proved movements. The documented physical-location convention is
**PROVISIONAL** pending founder acceptance and later handler verification.
The underlying contribution treatment is already **CONFIRMED**.

### Pure production surface

`state::reconciliation` adds fixed observation/result types and these helpers:

| Helper | Derivation / commit responsibility |
|---|---|
| `validate_custody_state_binding` | Config/round schemas; sequence and last-completed ordering; preparation timestamp; historical snapshot quantities and floored pool book value; target bounded by historical units; pending snapshot bounded by recognized pending SOL; phase-specific HWM, cooldown carry and KIF carry. |
| `expected_pending_sol_lamports` | Canonical checked `P-U` while active, `P` at Idle. Used by explicit intake before/after, pending reconciliation, economic normalization and composition invariants. |
| `economic_custody_obligations` | Separate expected balances for all six economic custody dimensions. |
| `economic_custody_surplus` | Each observed economic amount minus its own obligation; any known deficit rejects the complete assessment. |
| `record_economic_normalization` | Re-derive excess from before observations, require exact source depletion to obligations and exact dedicated-pending credits with identical floors, then reconcile both pending ledgers atomically. |
| `observed_token_book_value` | Checked floor over an intrinsically current valid pool observation; rejects token units exceeding supply and impossible bootstrap inputs. Exact real protocol mapping is deferred. |
| `derive_pending_integration` | Check exact pre-normalization balances and all post-movement balances, then derive contribution value, resulting historical quantities and HWM from custody. Reject an underprotected result. |
| `assess_operational_surplus` | Validate the observed floor; return `UnsupportedFundingBaseline` without selecting an amount or authorizing reserve movement. |

The observations contain no account identity proof. Same-account, unchanged-floor
before/after observations are required under the future handler contract; scalar
equality alone cannot establish that contract. No caller-controlled movement flag,
arbitrary offset, receipt type, serialized audit history or migration-reserve use
is introduced. The production helpers contain no failure controls or mock oracle.

### Per-vault commit matrix

In this table all native amounts exclude the permanent validated floor; add that
floor to obtain the actual account balance. `Q` is recognized pending token units,
`L` global earned KIF liabilities, `K` collective KIF carry, `F` cumulative
finalized native (including stake rent), `Rs` recovered stake rent, `Dn` net
allocation dust and `Dc` retained conservative dust. `Z` is zero before settlement.

| Vault / dimension | Idle / before open | After open / leg initiation | After each leg finalization | After settlement | After integration / Idle |
|---|---|---|---|---|---|
| PendingSolVault SOL | `P` | `P-U` | `P-U` | `P-U` | 0, until new receipts |
| PendingJitoVault token units | `Q` | `Q` | `Q` | `Q` | 0, after all Q moved to principal |
| PrincipalJitoVault token units | `Hj` | `Hj-A` | `Hj-A` | `Hj-A` | `Hj-A+Q` becomes new Hj |
| PrincipalSolQueue SOL | `Hsol+C` | `Hsol+C-Cu` | unchanged | `Hsol+C-Cu+Z` | add physically remaining `P-U` and escrow remainder; subtract new R only when deriving historical ledger |
| DistributionEscrow SOL | 0 | `U+Cu` | `U+Cu+F-Rs` | `Dn+Dc+R` | 0 |
| KifSolVault SOL | `L+K` | `L+K` | `L+K` | prior balance + actual net KIF - Z = updated `L+K` | unchanged |
| OperationalSolVault SOL | Excluded operational category | exact stake/metadata rent advances debit reserve | actual stake/metadata rent recoveries credit reserve | unchanged | unchanged |
| Both token accounts' rent lamports | Separate fixed test floors | unchanged | unchanged | unchanged | unchanged |
| Temporary WithdrawalStake SOL | absent | actual delegated SOL + advanced stake rent | zero after matched finalization | zero for all completed legs | zero |
| Temporary WithdrawalLeg metadata rent | absent | exact advanced metadata rent | zero after closure/recovery | zero for all completed legs | zero |

At every recognition boundary an explicit/direct arrival increases P or Q only.
It cannot change U, the snapshotted amount, eligible funding, target, obligations,
leg counters or settlement inputs. Direct external credits themselves need not
call PIV1; they first create observable excess. Normalization moves proven
non-pending excess to the correct pending vault before crediting recognized P/Q.
Immediate replay and subsequent reconciliation return zero.

A native surplus at principal, escrow or KIF is a pending SOL contribution after
source obligations and floors are covered. A principal-token surplus becomes
pending JitoSOL, preserving the historical token position. Existing pending
surplus is recognized in the same operation. No economic source donation becomes
historical yield, KIF debt or additional current-round funding.

The host normalization policy does not create a new recovery transition:
movement-bearing normalization rejects during pause or RecoveryRequired;
identification of excess remains possible and custody is preserved. Accounting
of already-received dedicated pending SOL/JitoSOL remains permitted in every
phase, including pause and RecoveryRequired, under the accepted Task 2.2
distinction. Explicit transfer-handler callability during pause remains
**PROVISIONAL**.

### Atomic composition and state evidence

`tests/support/vault_custody_model.rs` starts at an explicit explained Idle
fixture and composes Task 1.3 transitions, Task 2.2 recognition and the accepted
Task 2.1 mock adapter. Its complete world contains fixed custody arrays,
separate token-account rent, config/header/guardian state, fixed leg metadata,
adapter state and an independent initial/external-flow audit. No mock module is
exported by the library.

Opening derives historical value and pending-first funding, debits pending and
prior-carry principal custody, credits escrow, and supplies the observed
movements to `open_distribution`. The pure transition does not transfer funds.
The complete world commits once after invariants pass.

Initiation executes the deterministic adapter, observes the external pool native
and supply debits, debits principal tokens by fee plus burn, places delegated
native and advanced rent in distinct temporary custody, then records the leg.
Finalization requires mock inactivity, empties the stake account into escrow,
routes stake rent and metadata closure rent to operations, and derives residual
value and observed escrow for the accepted transition. Partial and out-of-order
finalizations reconcile all current leg records with the header. The mock's
capacity is 8 sources / 16 retained adapter withdrawal records; it is a bounded
test fixture, not a production leg cap.

Settlement stages exact HTFP/Team payments, net KIF credit and zero-active
KIF-to-principal movement. Its independent host allocation uses the accepted
5,900/1,950/200 weights over 8,050, with gross obligation caps. It derives the
protected-value input from the staged world:

```text
post-settlement protected value
  = principal SOL + floor(remaining principal tokens * pool SOL / supply)
    + post-payment escrow - new cooldown rewards - U
```

This excludes both physically pending contributions and the contribution value U
already substituted for historical yield left invested. Settlement then either
commits all payments and accounting or commits only the accepted recovery header.
A finalization returning successful RecoveryRequired retains its actual recovered
custody; an error discards every staged change.

Integration moves P-U, all Q, and the actual escrow remainder. The production
derivation checks before/after observations and derives:

```text
contribution value = P + floor(Q * current pool SOL / current pool supply)
new HWM = settled HWM + contribution value
new historical SOL = observed final principal SOL - new next-cycle yield
new historical tokens = observed final principal tokens
require new historical SOL + current book value(new historical tokens) >= new HWM
```

The active header's old historical amounts remain unchanged until integration;
after integration they are normalized from custody, and Idle does not subtract
the old round's U or A again. A later real principal SOL-to-Jito deposit is still
a separate boundary and is not implemented here.

State alone is not a transfer proof. Within this host model the evidence is
inductive: an explained Idle world plus successful atomic operations establishes
every subsequent committed state and its movements. Negative tests intentionally
mutate public test fields to challenge this invariant. A future handler must
authenticate fixed configured accounts, Program IDs/owners, PDA/mint/authority
bindings, signer/destination, writable/aliasing constraints, Clock/Rent, pool
freshness/source order, inactivity and exact pre/post CPI effects. None of those
on-chain facts is proven by the host tests.

### Rent, KIF and carry

Old cooldown carry is held in principal SOL separately from historical SOL.
Opening clears config's carry only after retaining C in the immutable header;
Cu moves to escrow, and the remaining C-Cu remains explained in principal. At
completion the unused old carry becomes part of historical principal through
the normal split/HWM accounting. New cooldown rewards stay in escrow, are
recorded once at settlement, and move to principal custody at integration while
remaining next-cycle yield. Tests cover prior carry 0 / 2,000 / 20,000 / 100,000,
including unused carry and subsequent rounds.

Stake and metadata rent advances and recoveries match exactly. The test oracle
tracks the initial legitimate reserve, outstanding advances and recovered rent
separately from floors. The same amounts reconcile against the adapter's audit.
Rent never increments recognized contributions, principal or HWM.

KIF custody always covers global recorded claims plus collective carry. An
explicit initial fixture holds 90 earned lamports across six guardian records.
Settlement credits only eligible snapshots; zero-active settlement sends the
compounded floor from KIF into principal and leaves the rest in KIF. Repeated
zero-active periods reapply the rule to all approved carry. Economic-vault
donations do not increase earned guardian liabilities. Bounded `claim_effect`
simulations debit the recorded guardian/global liability and isolated KIF
custody once, including during pause; they are not a production `claim_kif`
implementation or signer/destination authentication proof.

Cooldown loss and a residual-HWM failure each successfully commit finalization
and recovered custody into RecoveryRequired without reducing HWM. A settlement
HWM failure commits no speculative payments or new guardian liabilities.

### Independent conservation

The host maintains explicit initial balances and an external-flow audit solely
for testing. It never exports those values as production operational evidence.
Checked u64 production amounts and fixed host custody arrays are used; bounded
u128 test totals keep asset dimensions independent.

```text
initial native (vaults + token rent + external pool)
  + external SOL credits + pool rewards + cooldown rewards
  = current native (same custody universe) + pool losses + cooldown losses

initial held token units + external token credits
  = current held pending/principal/fee token units + burned units

withdrawal input units = fee units + burned units
principal input debit = external fee credit + external burn
initial reserve + recovered rent = legitimate reserve balance + advanced rent
each internal source decrease = its required destination increase
sum(current round leg facts) = checked cumulative header facts
sum(guardian claimable amounts) = global earned liability
KIF physical custody >= global earned liability + collective carry
```

Pool token supply is the conversion denominator, not a second token asset added
to custody. Adapter audits independently reconcile input/fee/burn, delegated
value, reward/loss and both rent categories. The model has no external SOL/Jito
deposit conversion implementation: withdrawal conversions use the accepted
deterministic adapter and are not exact live SPL/Jito evidence.

The randomized recognition oracle separately compares final P/Q against initial
pending custody plus external credited amounts, before deriving integration.
This prevents a normalizer's own reported output from supplying its expected
contribution totals. HWM assertions separate contribution value from actual
compound and dust. Per-account obligations additionally prevent global balance
conservation from concealing theft between categories.

### Fixed and randomized regressions

All 18 new tests pass. Fixed coverage includes:

- exactly zero, full and partial pending SOL use, arrivals before opening,
  after snapshot, after funding and after settlement, explicit/direct SOL and
  JitoSOL interleavings, unchanged complete rounds and immediate replay;
- a valid scaled 100/60/10 example for both explicit and direct credits:
  gross yield 74,536 produces outgoing 60,000 under the accepted floors;
  P=100,000 becomes physical 40,000 after funding, then P=110,000 / physical
  50,000 after arrival. Settlement compound/dust adds 14,537; contribution
  integration adds exactly 110,000; final HWM is 1,124,537 and historical SOL
  is 50,001. The internal 60,000 is neither lost principal nor a second credit;
- complete two-round lifecycles under four initial pending amounts and both
  arrival modes, ten-day timing, pending-token appreciation as contribution,
  single-leg/multi-leg full fee-plus-burn debits, finalization before target
  assignment completes and both finalization orders;
- economic source surpluses through opening/funding/settlement/Idle, token
  donations during active withdrawal, all source obligations retained, and
  normalization followed by reconciliation/integration without double credit;
- failed source debit, omitted/wrong destination credit, late before-commit
  injection at every movement-bearing operation, pure normalization
  observation mismatches and changed floors;
- natural late arithmetic errors after staged settlement/integration, malformed
  sequence/timestamp/historical snapshot/pending use/pool basis, wrong or stale
  leg/round, overconsumption, replay and reuse of a completed-round offset;
- deficits in every economic native/token category despite unrelated surplus,
  explicit intake preserving a real pre-existing deficit, u64::MAX recognition
  and active-offset overflow, floor underflow and impossible token supply;
- KIF claims/carry isolation, zero-active custody and repeated carry, legitimate
  rent/metadata closure, rewards, loss and both residual-HWM recovery outcomes;
- paused and RecoveryRequired pending recognition, isolated earned-claim
  effects, and rejected economic progression or movement-bearing normalization;
- operational excess explicitly unsupported while economic normalization
  proceeds without moving that reserve.

The new deterministic SplitMix64 model uses seed
`0x504956315641554c`, 128 cases, bounded action indices plus leg/source labels,
and coverage guards. Final counters: **128 completed lifecycles, 7,301 counted
successful actions, 1,471 checked rejected actions, 43 liquid rounds,
85 multi-leg rounds and 209 successful/finalized legs**. It exercises random
intake/donations/reconciliation, active and post-settlement normalization,
fees/guardian counts/carry, premature finalizations, replays and both
finalization orders. Whole-world equality follows every checked rejection.
Every case reaches completion; coverage is not a count of trivial failures.
The lifecycle integration wrapper adds seed/case/action context to assertion
failures. Fixed boundary tests do not depend on random coverage.

Retained Task 2.2 seed `0x50495631434f4e54` still runs 1,024 cases x 64 actions.
Only its authorized composition preconditions/oracle equations changed:
phase configs now bind to their real snapshot, physical pending starts at P-U,
and reconciliation's independent recognized oracle is physical+U. Existing
complete-state, immutable-round and rejection protections are retained.
The original small intake mock remains an assumed-phase fixture; the new
Idle-origin composition supplies movement evidence. Task 1.3/1.4/2.1 and math
test files are unchanged.

### Compatibility and remaining limitations

The five serialized field declarations/order/types and migration reserve are
unchanged. Only config comments changed. Planned spaces remain **1014 / 891 /
263 / 210 / 84** for config, active distribution, withdrawal leg, registry and
reward respectively. All observation/result types are ordinary nonserialized
values. One nonserialized error category, `EconomicCustodyDeficit`, was added.
No math implementation, dependency, manifest, lockfile or toolchain pin changed.

**OPEN / unsupported derivations:**

1. Operational funding/excess: P-033/P-034 and master 10.2 define the category,
   but PivConfig does not serialize an initial reserve baseline or a complete
   authenticated funding/advance history. Different legitimate initial reserve
   amounts can explain the same current balance and active header. Production
   cannot uniquely derive excess from these fields; the result remains
   `UnsupportedFundingBaseline`. The test oracle is not a proposed production
   substitute. No serialized change is required for supported economic paths.
2. Unexpected native lamports in legacy token accounts or temporary accounts
   beyond the recorded rent/reward model are not normalized by this task.
   Exact owner-constrained extraction, rent floors and stake accounting require
   future real account/handler evidence. Token **units** at both dedicated token
   vaults are supported. Account-rent lamports are kept separately.
3. The host uses an explained Idle fixture, current mock pool observations and
   existing recorded guardian liabilities. It does not authenticate arbitrary
   externally loaded active states, reconstruct unobserved transfer history,
   initialize real vaults, implement governance/recovery/claim handlers or prove
   contributor source signatures.
4. Exact live pool book/withdrawal target, round-minimum/slippage, source-order,
   fee, inactivity, account ownership/PDA and snapshot-identity mapping remain
   **PROVISIONAL** for Phase 3. The host fixes a gross floor target, applies the
   adapter's dynamic leg minimum and slippage, and derives custody/HWM checks;
   this does not validate exact production multi-leg target/minimum sizing.
5. A current balance alone cannot identify an earlier unobserved debit replenished
   by an indistinguishable credit. Known deficits reject; the model does not
   claim forensic provenance or add unsupported production history assumptions.
6. Later principal compounding deposits, real SOL entry, localnet, handlers,
   signers, CPI and runtime locking remain deferred. No new transfer-handler
   policy during pause is selected. Movement-bearing normalization during
   recovery awaits authorized recovery handling, while safe pending recognition
   remains supported.

No conflict requiring a new economic destination or serialized-layout change was
found. Older statements that Task 2.3 was unstarted are historical at their
recorded acceptance point. Master 7.7's deferral is read with later accepted
D-024 and this authorization as deferring integration, not safe receipt
recognition. Non-pending economic donations are not declared permanently OPEN.

### Commands and validation

All repository mutations, file creation, builds and tests ran as `jerem` with
`HOME=/home/jerem` and explicit
`PATH=/home/jerem/.cargo/bin:/usr/local/bin:/usr/bin:/bin`. Repository-local Git
identity remains HTFP Project <HoldTheFuckingPosition1@protonmail.com>.
The initial root Git read refused dubious ownership; it was rerun as jerem
without changing safe-directory/global configuration. `rg` was unavailable on
that explicit user PATH, so subsequent bounded searches used grep/find/Python.

| Command | Result |
|---|---|
| `git rev-parse --show-toplevel`, `git status --short --branch`, `git worktree list`, `git log -6 --oneline`, local identity reads | Clean main at accepted baseline; single worktree; expected identity |
| `git ls-remote origin refs/heads/main` | Public main matched 66193769d1cbc59cd8630df295b9a784b9c64642 |
| `git merge-base --is-ancestor 66193769d1cbc59cd8630df295b9a784b9c64642 HEAD` | PASS |
| `git switch -c task/2.3-vault-reconciliation-model` | Created from accepted clean main |
| `cargo +1.97.1 check -p piv1 --tests --locked --offline` | Initial development compile identified two incorrect test constructor names; corrected |
| `cargo +1.97.1 test -p piv1 --test vault_reconciliation --test contribution_pending --locked --offline -- --nocapture` | PASS after development corrections; 17 composition + 9 contribution tests at that pass |
| `cargo +1.97.1 test -p piv1 --test vault_reconciliation --locked --offline -- --nocapture` | Final targeted PASS: 18 tests and recorded seed counters |
| `cargo +1.97.1 check --workspace --all-targets --locked --offline` | PASS |
| `cargo +1.97.1 check --workspace --all-targets --all-features --locked --offline` | PASS |
| `cargo +1.97.1 test --workspace --all-targets --locked --offline` | PASS: 164 tests, zero failures/ignored |
| `cargo +1.97.1 test --workspace --doc --locked --offline` | PASS: 1 math doctest; no program doctests |
| `RUSTDOCFLAGS="-D warnings" cargo +1.97.1 doc --workspace --no-deps --locked --offline` | PASS, no warnings |
| `rustup component list --installed --toolchain 1.97.1` | cargo/rust-std/rustc only; no installation performed |

The final workspace run contains 35 state/unit, 9 contribution, 20 illegal
transition, 11 legal lifecycle, 12 state property, 24 adapter, 18 composition,
32 math unit and 3 math property tests. Existing layout/property assertions are
included. Every required gate ran once after the final code change; none is
unrun or represented as passed without execution. Rustfmt/clippy are unavailable
and not required gates; neither was installed. No Anchor command ran.

Development corrections were test constructor/adapter-argument API mismatches,
two retained physical-versus-recognized fixture/oracle expectations, and explicit
mock pool maintenance after epoch advancement before current valuation.
Subsequent review added snapshot pool-basis and supply checks, observed full
supply/burn reconciliation, exclusion of U from residual protected value,
independent recognition assertions, and settlement recovery tests. No accepted
math formula, security gate or unrelated test was weakened.

Git diff/secret/generated-path/compatibility/ownership/integrity checks and final
publication status are recorded below.

### Safety and next action

No Mainnet action, RPC, local validator, blockchain transaction, fund movement,
wallet/key/seed creation, Program ID, handler, Accounts context, CPI, deployment,
authority transfer, push, PR, tag or merge was performed. The known ignored
Task 0.4 artifacts were not inspected or modified. Task 2.4/later work did not
start. The next action is founder review of the committed dedicated branch.
This is AI-assisted engineering and testing, not a professional independent audit.

### Files and repository checks

The task changes 19 files:

- `AGENTS.md`, `README.md`, `programs/piv1/README.md`;
- `docs/PIV1_DECISIONS.md`, `docs/PIV1_MASTER_SPEC.md`,
  `docs/PIV1_CODEX_EXECUTION_PLAN.md`;
- `docs/TASK_1_3_STATE_MODEL.md`,
  `docs/TASK_2_2_CONTRIBUTION_PENDING_MODEL.md`,
  `docs/TASK_2_3_VAULT_RECONCILIATION_MODEL.md`;
- `programs/piv1/src/errors.rs`, `programs/piv1/src/state/config.rs`,
  `programs/piv1/src/state/contributions.rs`,
  `programs/piv1/src/state/mod.rs`,
  `programs/piv1/src/state/reconciliation.rs`;
- `programs/piv1/tests/contribution_pending.rs`,
  `programs/piv1/tests/vault_reconciliation.rs`,
  `programs/piv1/tests/support/mod.rs`,
  `programs/piv1/tests/support/contribution_custody_mock.rs`,
  `programs/piv1/tests/support/vault_custody_model.rs`.

The new implementation is 356 lines of pure reconciliation helpers, 743 lines
of fixed host custody support (including the explicit config fixture), and
765 lines of composition tests. Existing production changes are the shared
contribution obligation/binding checks, one error, module registration and
config comments. Task 1.3 transitions and the accepted adapter/math are unchanged.

Repository checks completed:

- `git diff --check` and `git diff --cached --check`: PASS.
- Read-only Python comparisons using `git show BASE:path`: 16 protected
  math/manifest/lock/pin files byte-identical; distribution, guardian,
  transitions and timing sources byte-identical; config non-comment tokens
  identical, preserving the serialized field declarations.
- Existing maximum-space and randomized serialization tests passed for all
  five schemas in the final workspace gate.
- Read-only scans of tracked/new content and changed paths: no high-confidence
  private-key/credential marker, credential-bearing URL, generated target,
  cache, ledger, wallet/key file or node_modules path included. Ignored files
  were neither enumerated for content nor read.
- Changed production source, excluding existing cfg(test) sections: no runtime
  program/account markers, unsafe code, panic extraction, wrapping or saturating
  arithmetic. Initial scan scripts were corrected to ignore comment whitespace
  and existing unit-test code; these were verification-scope corrections.
- Ownership: all 19 changed/new files plus touched Git index/ref files belong to
  jerem; no task-caused ownership repair or unrelated file deletion was needed.
- `git fsck --full --strict`: PASS; 16 informational dangling blobs, no corrupt
  or missing object. Their contents were not inspected or modified.
- Local `main` remains at the accepted baseline; the task history is linear,
  with the equations commit followed by a normal implementation commit.
- The delivery leaves the dedicated branch committed, worktree/index clean,
  unpublished and unmerged. No acceptance marker, tag or PR was created.


## Review correction T23-R1 — 2026-09-08 UTC

D-026 authorizes this compatible correction under the active technical pilot.
Status: **TECHNICALLY VALIDATED / PENDING FOUNDER ACCEPTANCE** after the separate
review and pilot gates recorded below.
The preceding implementation report remains historical evidence for `46b448d`;
the pilot checkpoint records the subsequent review and development sequence.

The host finalization and settlement wrappers previously subtracted committed
pending SOL use from retained value with the general checked-subtraction helper.
A severe pool loss could make retained value smaller than that pending use,
causing `CumulativeReconciliationMismatch` before the accepted pure transition
could enter `RecoveryRequired`. Finalization consequently discarded the staged
stake and rent recovery; settlement kept its old `EscrowFunded` state.

Only the two HWM comparison inputs now use `historical_value_for_recovery`.
When retained value is below committed pending use, the comparison value is
explicitly zero: no retained value is available for historical protection.
That value is below the positive protected floor of a prepared round and reaches
the existing recovery transition. Otherwise the exact checked difference is
preserved. The helper is used solely for recovery classification; it never
changes custody, pending value, HWM, payments or integration accounting.
General checked subtraction and every existing custody/state validation remain.

Four new operation-level regressions cover:

- the exact reported finalization failure (retained value 99, pending use
  4,000), plus retained values 3,999 / 4,000 / 4,001;
- the exact reported liquid-settlement failure (retained value 100, pending use
  8,050), plus retained values 8,049 / 8,050 / 8,051;
- zero-active KIF settlement recovery with prior carry 101: speculative compound
  150 and carry 151 are discarded along with all beneficiary payments;
- complete rollback of injected debit/credit/commit failures, malformed pending
  offsets, known KIF custody deficit, pause, and the existing checked KIF-credit
  overflow despite severe pool loss.

Finalization assertions establish finalized leg state, exact escrow credit,
both rent recoveries, native conservation and unchanged HWM/pending/KIF state.
Settlement asserts equality of the entire world except the required recovery
header. Replays and integration attempts reject atomically after recovery.
Existing ordinary recovery, multi-leg, carry, rewards and normalization tests
remain unchanged.

Writer-executed evidence on the uncommitted correction atop documentation
checkpoint `df1250064011428b88a6ef7aae8b0c42521f5e95`:

| Command | Result |
|---|---|
| `cargo +1.97.1 test -p piv1 --test vault_reconciliation --locked --offline severe_pool_loss -- --nocapture` before the host correction | All 4 new tests failed with the original subtraction error; no host implementation change preceded this run |
| `cargo +1.97.1 test -p piv1 --test vault_reconciliation --locked --offline -- --nocapture` after the host correction | PASS: 22 tests; 128 completed deterministic lifecycles, 7,301 counted successful actions, 1,471 rejected actions, 43 liquid rounds, 85 multi-leg rounds, 209 legs |
| `git diff --check` | PASS |

Commands ran as `jerem` using `/home/jerem/.cargo/bin/cargo`; no dependencies,
production sources, serialized layouts or accepted economics changed. The
separate reviewer and pilot own the final workspace gates and checkpoint.
This remains host-only evidence with the earlier account/CPI/protocol/recovery
limitations unchanged. No commit, publication, merge, secret access, Mainnet or
other blockchain operation, deployment, fund movement, key creation, signing,
or authority transfer occurred during this delegated correction.

### Pilot final validation and separate review — 2026-09-09 UTC

The pilot inspected the correction and independently executed these gates on
its final unchanged Rust source (the frozen correction against `df125006`):

| Command | Pilot-observed result |
|---|---|
| `cargo +1.97.1 test --workspace --all-targets --locked --offline --quiet` | PASS: 168 tests, zero failed/ignored |
| `cargo +1.97.1 test --workspace --doc --locked --offline` | PASS: 1 math doctest |
| `cargo +1.97.1 check --workspace --all-targets --locked --offline` | PASS |
| `cargo +1.97.1 check --workspace --all-targets --all-features --locked --offline` | PASS |
| `RUSTDOCFLAGS='-D warnings' cargo +1.97.1 doc --workspace --no-deps --locked --offline` | PASS |
| `git diff --check` | PASS |

The separate `review_t23_final` subagent inspected current source, canonical
recovery requirements and the exact frozen three-file diff and returned
**PASS / no actionable findings**. It did not execute builds; the workspace
results above are the pilot's own executions. An earlier reviewer was blocked
in its inherited tool-approval context and returned INCOMPLETE; that aborted
review is not counted as a pass. The successful replacement reviewer used
working `cat`/`sed` reads and a pilot-captured exact diff.

T23-R1 is technically resolved without changing production source, serialized
layouts, dependencies or economics. Task 2.3's original scope plus this
correction is technically validated; founder acceptance is still pending.
The exact correction commit and subsequent development dependency are recorded
in [PIV1_PILOT_STATE.md](PIV1_PILOT_STATE.md). Public-Testnet operations and keys
remain behind D-026's live-operation gate. This is AI-assisted engineering and
review, not a professional independent audit.
