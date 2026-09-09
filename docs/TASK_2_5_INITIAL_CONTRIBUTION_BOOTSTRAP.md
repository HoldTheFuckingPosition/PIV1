# Task 2.5 — Initial contribution bootstrap

Status: **TECHNICALLY VALIDATED / PENDING FOUNDER ACCEPTANCE** under D-026.
Baseline: published integration checkpoint
`a1d585d117802fb8e595f089b0604527d61047d2`, including technically validated
Task 2.4 `9f4f1064deeef78a3cbea2e9f84c560e87166f20`.

## Verified missing dependency and authority

P-004/P-014/P-015 require contributions to establish perpetual principal without
becoming historical yield. Actual contribution intake/reconciliation currently
changes only pending ledgers. Both `derive_pending_integration` and
`integrate_pending_and_complete` require Settled. Opening a round requires
positive historical-plus-carry yield. Therefore a genuine initial PIV with zero
historical assets, zero HWM and zero carry cannot bootstrap through these paths.
The existing Task 2.3 host world starts with pre-existing principal; this task
must add an honest empty-PIV fixture rather than pretend that principal came
from executed contributions.

The pilot and separate scoping reviewer inspected these actual boundaries.
Implement the smallest separate **initial bootstrap** boundary. General idle
integration is excluded: it could consume pending SOL before already-positive
yield is paid, conflicting with P-013 priority, or change the mutation boundary
of a valid insufficient attempt (P-029–P-031/master section 9.8).
No-yield/insufficient helpers and accepted distribution transitions stay intact.
This is a compatible missing initialization dependency, not new economics.

## Required invariant and state boundary

- Require valid, unpaused Config and a mutually bound initialized Idle header,
  with no completed round, successful preparation or valid-insufficient history.
  Require zero historical SOL/JitoSOL, protected HWM, next-cycle yield,
  contribution/distribution/KIF economic audit totals, KIF liability and carry.
  Reject existing or recovered principal even if current market value is zero.
  Preserve configured identities, guardian activity/membership, clock anchor and
  exact next sequence; do not invent an initial counter value from memory.
- Recognized pending SOL and/or token units must be positive. A positive token
  quantity may conservatively have zero floored SOL value; do not fabricate a
  one-lamport minimum, round up, or create replay through that case.
- Require normalized before custody with each vault's complete legitimate
  obligations covered and no remaining unexplained economic surplus. Existing
  supported normalization may run as its own prior atomic boundary.
- Derive integration from a validated PoolSnapshot and exact before/after
  EconomicCustodyObservation, using the existing checked book-value seam.
  Accept no caller-selected contribution value, historical balance or HWM proof.
- Move exactly all recognized pending SOL into PrincipalSolQueue and all
  recognized pending JitoSOL into PrincipalJitoVault. Require unchanged rent
  floors and exact matched same-asset deltas; pending becomes zero. Escrow,
  KIF, operational funding, recipients, pool state and temporary accounts remain
  unaffected. No protocol fee or external payment is part of bootstrap.
- Set historical SOL/token ledgers to those exact initial holdings. Increase
  HWM and cumulative contribution value by full pending SOL plus floored current
  value of pending token units. Appreciation before this boundary belongs to
  contribution principal. Gross yield, splits, dust/carry, KIF and clocks do not
  change; no snapshot, distribution sequence allocation or terminal summary is
  created. The Idle header is preserved byte-for-byte.
- Reject replay, pause, any active/recovery/settled/completed state, existing
  principal or historical economic records, deficit/surplus, malformed pool,
  wrong/missing movements, floor changes and overflow before any state commits.
  No zero-pending bootstrap is a successful economic event.

## Implementation and evidence budget

One delegated writer may add a small pure bootstrap module/API and result/error,
necessary exports, focused tests, a genuine initial host fixture/atomic method
and this report. Reuse existing accepted accounting/observation functions.
No dependencies, serialized fields, layout version/size changes, ordinary
transition rewrites, CPI, handler, initializer, live program ID or key are allowed.
Do not weaken old test oracles or reset an audit after an action to hide failure.
An empty fixture must establish its initial audit once after constructing the
initial state; preserve existing funded-world behavior and coverage counters.

Focused tests cover SOL-only, token-only and mixed initial contributions;
explicit/direct recognition and supported prior normalization; pool appreciation
while pending; zero-rounded token value; checked overflow and exact boundaries;
full ledger/header/clock/KIF/rent preservation; second-call rejection, including
zero-valued first tokens; every non-genesis state and malformed observation;
all existing host transfer/late-commit failure injections with conservation.
Show that token-backed initial principal can later earn genuine historical yield
without reclassifying the initial contribution as yield. A SOL-only bootstrap
ends in the principal SOL queue: actual protected staking/deposit is a later
bounded dependency, not an unobserved step in this task.

Writer runs focused locked/offline host tests and freezes source. Pilot inspects
actual source/tests, runs final workspace/default/all-feature/doc gates and
obtains separate final diff review. Correct findings, commit, and checkpoint
exact files, commands/results, review, limitations and next dependency before
continuing. Technical validation never grants founder acceptance.

## Trust and sensitive-operation limits

PoolSnapshot/transfer observations remain a pure/host seam requiring authenticated
runtime pool inputs and real atomic custody deltas in later handlers. Task 2.4
AccountInfo fixtures are not runtime transfer proof. Unsupported token native
surplus and operational funding provenance remain explicit pre-handler work.
No new key, blockchain signing, deployment, fund movement, Mainnet action or
authority transfer is authorized here. D-026 live-operation approval remains NONE.


## Delegated implementation and focused evidence — 2026-09-09 UTC

Writer: `implement_t25_bootstrap`; actual user `jerem`, worktree
`/home/jerem/piv1`, branch `integration/piv1-testnet`, unchanged HEAD
`a1d585d117802fb8e595f089b0604527d61047d2`. The pilot's existing scope,
checkpoint, plan and other documentation edits were preserved. This section
records writer evidence; final workspace gates, separate review and commit
remain the pilot's responsibility. No founder acceptance is inferred.

### Implementation

`bootstrap_initial_contributions` takes only Config, the immutable Idle header,
a PoolSnapshot and complete before/after economic custody observations. It
validates initialized/unpaused state, accepted Config/header relationships and
the stored header bump against Config. That equality is a stored consistency
check, not proof of PDA derivation or actual accounts. All economic history,
completed/preparation/insufficiency history, HWM, carry and KIF liabilities must
be absent. Configured sequence values including `u64::MAX` are preserved; the
function allocates no sequence and writes no header or timestamp.

Normalized before custody and exact matched movements are required. All pending
native SOL goes to PrincipalSolQueue; all pending token units go to
PrincipalJitoVault. The existing conservative token book-value seam validates
the pool, its freshness and sufficient supply before full pending SOL plus
floored token value becomes HWM/contribution value. Only the two pending, two
historical, HWM and cumulative-contribution fields change, after all validation.
No fee, payout, split, KIF action, protocol deposit or new yield occurs.
Positive token units with zero floored value become historical units and prevent
another bootstrap despite zero HWM and cumulative contribution value.

The new `World::empty` and existing funded constructor share fixture construction;
the empty variant selects zero PIV economic assets/history/liabilities before the
single initial audit baseline is established. Existing funded holdings, earned
liabilities, guardian activity and initial audit behavior are retained. The new
atomic host method stages only the two same-asset transfers and pure transition,
then uses the unchanged complete-world conservation/commit checks. No audit is
reset after an action. Existing tests and property coverage counters are unchanged.

### Focused test evidence

The new 18-test target covers:

- genuine empty-PIV rejection of fabricated distribution opening/completion;
  unchanged no-yield evaluation; explicit SOL-only/token-only/mixed receipt;
- unrecognized direct pending receipts and prior supported economic-vault
  normalization, with cumulative contribution/HWM remaining zero until bootstrap;
- arbitrary next sequences, preserved anchor/guardian activity and serialized
  Idle header bytes, plus exact equality of every unchanged World field;
- pending-token appreciation becoming principal, zero-rounded positive tokens
  and replay after another pending receipt;
- every existing debit/credit/late-commit injection for SOL-only/token-only/mixed
  bootstrap, checked contribution-value overflow and destination native overflow;
- prior economic history, earned/claimed KIF history, pause, malformed Config,
  malformed/header-bump mismatch, genuine active/settled/recovery/completed states;
- every economic-vault surplus, independent pending deficits, missing/wrong or
  cross-asset movements, every native rent-floor change and below-floor input;
- stale/malformed pool, insufficient token supply, valid empty-pool SOL-only
  bootstrap, maximum exact `u64` contribution and no fee charged at nonzero fees;
- a full later historical-yield distribution from executed token contribution:
  initial 1,000,000 token units establish HWM 1,010,000; later pool reward raises
  its value by 10,000, which is the round's entire gross yield. Withdrawal,
  finalization, settlement and completion conserve custody while cumulative
  contribution value remains 1,010,000 and completion integrates zero new value.

Actual writer commands used `/home/jerem/.cargo/bin/cargo +1.97.1`:

| Command | Actual result |
|---|---|
| `test -p piv1 --test initial_bootstrap --locked --offline -- --nocapture` | Initial 16 tests PASS; final 18 tests PASS after clarifying the host test name; no compile/test failure |
| `test -p piv1 --test initial_bootstrap --test vault_reconciliation --test contribution_pending --locked --offline -- --nocapture` | Final 49 tests PASS: 18 bootstrap, 22 composition, 9 pending; zero failures/ignored |
| `git diff --check` and scoped Python trailing-whitespace/newline check | PASS, including both new Rust files |
| `pwd`, `id -un`, branch/HEAD/status reads, scoped `rg`/`cat`/`sed` reads | Verified actual repository and canonical/local scope |

Existing composition seed `0x504956315641554c` retained its exact counters:
128 cases/completed lifecycles, 7,301 accepted actions, 1,471 rejected actions,
43 liquid rounds, 85 multi-leg rounds and 209 legs. The existing pending target
also passed its deterministic randomized model. The writer did not execute the
final workspace/default/all-feature/docs gates, which belong to the pilot.
The maximum exact `u64` acceptance cases use explicitly synthetic pure numeric
observations; they are not host custody or conservation evidence. The separate
full-world value-overflow and destination-overflow regressions execute actual
host receipt/movement staging without resetting any audit.
Rustfmt is unavailable per the existing checkpoint; it was neither installed
nor counted as passed. No dependencies/toolchain components were added.

The default execution and apply_patch sandbox failed before execution with
`bwrap: loopback: Failed RTM_NEWADDR: Operation not permitted`. Narrowly scoped
read/write/build commands succeeded through automatic escalation review; the
fallback editor was a literal Python heredoc. No founder authorization was
requested and no automatic approval rejection occurred.

### Writer files, Git state and limitations

Exact writer-owned source/test changes:

- `programs/piv1/src/state/bootstrap.rs` (new pure API/result);
- `programs/piv1/src/state/mod.rs` (module/result/function exports);
- `programs/piv1/src/errors.rs` (`InvalidBootstrapState`, nonserialized error);
- `programs/piv1/src/state/reconciliation.rs` (existing floor equality becomes
  crate-visible for reuse; no algorithm changed);
- `programs/piv1/tests/support/vault_custody_model.rs` (empty fixture and bootstrap);
- `programs/piv1/tests/initial_bootstrap.rs` (new focused target);
- this report (writer section appended).

At writer freeze these changes and the pilot's documentation edits remain
uncommitted in the shared worktree. No writer commit or publication occurred;
HEAD is still `a1d585d117802fb8e595f089b0604527d61047d2`. Serialized account fields,
allocations, math, accepted transition algorithms, dependency manifests/lock and
pins are unchanged. Later general Idle integration and protected principal SOL
staking remain separate dependencies. Pool/transfer observations are pure/host
facts; runtime account/transfer authentication and exact SPL/Jito mapping remain
unproven. Operational funding provenance and unexpected token native surplus
retain their existing unsupported boundaries. No handler, initializer, CPI,
localnet/live execution, professional independent audit or founder acceptance
is claimed.

No Mainnet action, deployment, blockchain signing/transaction, fund movement,
key/seed/wallet creation or authority transfer occurred. The D-026 live-operation
approval state remains NONE.

## Final pilot gates and separate review — 2026-09-09 UTC

The pilot inspected the final bootstrap API, shared fixture construction and
all tests. Source hashes captured before validation remained unchanged through
the final gates. All commands used `/home/jerem/.cargo/bin/cargo +1.97.1`.

| Final pilot command | Own execution result |
| --- | --- |
| `test --workspace --all-targets --locked --offline --quiet` | **209 PASS**, zero failed/ignored |
| `test --workspace --doc --locked --offline` | **1 doctest PASS** |
| `check --workspace --all-targets --locked --offline` | PASS |
| `check --workspace --all-targets --all-features --locked --offline` | PASS |
| `doc --workspace --no-deps --locked --offline`, `RUSTDOCFLAGS='-D warnings'` | PASS |
| `git diff --check` | PASS |

The pilot verified that dependencies/toolchain, math, payload schemas and all
accepted distribution transitions are unchanged against `a1d585d`. The shared
reconciliation edit changes only `same_floors` visibility for the sibling module.
Targeted changed-file ownership, credential-marker and generated-file checks
passed; custom hook path remains unset, hooks are samples only, and there is no
tracked GitHub CI/Cargo configuration. Accepted main remains `6619376`.

Separate reviewer `review_t23_final` inspected the exact frozen six-file Rust/
test diff plus report against `a1d585d`, then actual source. Result: **PASS / no
actionable findings within the initial-bootstrap scope**. The reviewer confirmed
empty-history gating, full contribution/HWM derivation, exact custody deltas,
zero-valued-token replay rejection, unchanged header/sequence/clocks/KIF and
existing transition semantics. The shared funded fixture is preserved; the
empty fixture establishes its audit before actions. Synthetic maximum-value
checks remain distinguished from actual host movement/conservation cases.
No reviewer builds were run; the table above is pilot execution evidence.

The implementation hash and post-task checkpoint are recorded in
`PIV1_PILOT_STATE.md`. This is technically validated pure/host behavior, not
founder acceptance, real custody/runtime proof or a professional independent
audit. SOL-only principal still waits in its separate queue for a later protected
deposit boundary. General idle integration, handlers/CPI and unsupported native
surplus remain deferred. No Mainnet action, deployment, fund movement, key
creation, blockchain signing or authority transfer occurred. Live-operation
approval under D-026 remains NONE.
