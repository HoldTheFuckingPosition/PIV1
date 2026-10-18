# M4 closure review — supported economic-vault surplus normalization

Status: **REVIEWED SOURCE/EVIDENCE GAP** (2026-10-18 UTC).
Analyzed integration HEAD: `bd2af97fd8040e7cf1fb10cea06b520064092f09`.
Scope: master §7.8's economic-surplus requirement and its accepted Task 2.3
composition, exposed by Task 2.42. Line references use the analyzed HEAD.
This is source/evidence inspection, with no new test, build or runtime execution.

## Requirement and separate verdicts

> Proven excess at an economic vault moves to dedicated pending custody and is
> recognized once after its legitimate obligations and rent floors are covered.
> — `PIV1_MASTER_SPEC.md:692–693`, §7.8

D-025 (`PIV1_DECISIONS.md:124`) explicitly preserves historical assets, active
obligations, earned KIF liabilities/carry and rent. Its operational funding-baseline
limitation remains unchanged. Task 2.3 (`TASK_2_3_VAULT_RECONCILIATION_MODEL.md:156–166`)
classifies native principal/escrow/KIF surplus and principal-token surplus as
pending contributions, never historical yield, KIF debt or extra current-round
funding. Existing dedicated-pending surplus is recognized in the same operation.
Movement rejects during pause/recovery; already-pending recognition remains allowed.

**Code verdict: implemented for the supported economic dimensions; no demonstrated
production defect.** The handler derives each source's own obligation before
moving excess, verifies exact custody effects and commits only pending ledgers.
This source verdict is not a VM resource, CPI or rollback execution proof.

**Runtime verdict: a concrete production-path evidence gap remains in the inspected
lifecycle suite.** It exercises dedicated-pending reconciliation, but does not
invoke the separate thirteen-account normalization instruction. Retained Task 2.42
evidence is host execution plus static SBF compilation. Older pending-vault SBF
evidence proves a different, non-moving reconciliation boundary.

## Enforcement and paths walked

Source references below are relative to `programs/piv1/`.

1. `src/lib.rs:52–54` exports the real entrypoint;
   `src/instruction_boundary.rs:114–117` dispatches `PIV1RB01` to
   `economic_normalization::process_instruction`. The codec in
   `src/instructions/reconcile_untracked_balances.rs:5–9` accepts only exact
   nine-byte `PIV1RB01` plus version one. The handler (`src/economic_normalization.rs:28–31,52–59`)
   rejects wrong account count and unavailable native-host runtime, obtains Rent,
   rejects aliased keys/backing storage and prepares every movement before effects.
2. `src/accounts.rs:205–259,264–307,340–362` authenticates Config/round ownership,
   sizes, canonical PDA bumps, lifecycle binding and individual native/token rent.
   Token mint/authority, initialized legacy layout and absence of delegated,
   native-wrapped or close-authority modes are checked. The normalization accessor
   (`132–134`) requires all supported economic obligations; Token-owned native
   funding and operational funding remain outside those economic amounts.
3. `src/state/reconciliation.rs:168–243` derives pending SOL as P-U only for a
   valid active round; principal tokens as historical units minus active assigned
   input; principal SOL including its correct carry/compound component; phase-specific
   escrow; and KIF liability plus carry. Rent is subtracted separately (`26–54`).
   Checked subtraction rejects any category's deficit, even with unrelated excess.
4. `src/economic_normalization.rs:129–160` authenticates fixed System/Token IDs,
   authority, Mint/rent and writability, then derives exact post-movement balances.
   `src/state/reconciliation.rs:253–282` gates actual movement during pause/recovery,
   requires unchanged floors and exact complete post-custody, and calls
   `src/state/contributions.rs:181–226`. That function changes only the two pending
   ledgers, preserving the active P-U offset and validating both assets atomically.
   A final zero-surplus check establishes idempotence after recognition.
5. `src/economic_normalization.rs:163–185` builds at most three signed System
   transfers (principal/escrow/KIF to PendingSol) and one canonical-authority
   legacy `TransferChecked` (PrincipalJito to PendingJito). Zero amounts skip CPI.
   It derives amounts itself, with no caller amount, valuation or source selection.
   `195–224` checks Mint and acquires later required borrows before the first CPI.
6. The dispatch loop (`60–77`) verifies every supplied account's exact metadata,
   lamports and data hash after each successful CPI; it reauthenticates complete
   custody before a single Config replacement and factual event. Error propagation
   depends on actual transaction rollback, not host callback staging. Persistence
   (`src/state_persistence.rs:177–218`) validates prior bytes/identity/rent, acquires
   all mutable borrows and only then copies the prepared bytes.

No historical amount/HWM, distribution snapshot, KIF liability or carry is written
by this path. Native balances of both Token accounts and the operational account
are included in exact post-CPI preservation records, not classified or swept.

## Evidence distinction and search record

The inspected `tests/economic_normalization.rs:95–158` covers six economic
dimensions across model-produced lifecycle states, idempotence, individual
deficits, pause/recovery and quarantined categories. `160–305` covers wire,
identity, borrows, overflows, host-stage discard and per-CPI checks. These are
explicit callback tests: `src/economic_normalization.rs:34–42` states that failed
host effects do not automatically roll back.

Task 2.42 reports ten grouped tests, 508 host tests plus one doctest/eight gates
and a 523,832-byte SBF ELF (`TASK_2_42_ECONOMIC_NORMALIZATION_RUNTIME.md:94–114`).
It explicitly defers current-path VM/Bank evidence (`69–72`). Those historical
results were not rerun here and the old ELF is not the latest lifecycle artifact.
Current host/SBF and X2 results retain their recorded scope; no normalization
execution is inferred from compilation or inclusion in a newer artifact.

Searches and inspected counterevidence:

- `PIV1RB01|RECONCILE_UNTRACKED|reconcile_untracked|economic_normalization`
  across `validation/` (excluding lockfiles) found no invocation or import;
  across runner `.py/.mjs/.rs` sources under `tools/` found no hit.
- `reconcile_pending|direct_token|normaliz` in the lifecycle source/fixture found
  the pending-only path: `validation/genesis-bank-runtime/tests/lifecycle_fixture.rs:145–148`
  uses selector `e1801d669d18acce` and just Config, round, PendingSol and PendingJito.
  `production_lifecycle.rs:389–391` transfers one native/token unit directly into
  those pending vaults while paused and recognizes them. It does not move excess
  from principal, escrow or KIF. The token transfer helper fixes PendingJito as destination.
- Older runtime counterexample search inspected
  `validation/sbf-claims/tests/pending.rs:1–15,61–69`: Task 2.14 uses
  `encode_reconcile_pending`, synthetic initial token units and actual System
  donations into PendingSol. It cannot establish the later Task 2.42 movement path.
- `normaliz.*(bank|runtime|sbf)|reconcile_untracked|PIV1RB01` across task reports
  found Task 2.42's static artifact and initializer-specific normalization, not
  a claimed execution of economic-vault normalization. Genesis Token-native
  prefund handling is a different requirement and is not substituted here.

The absence conclusion is bounded to this inspected validation suite and its task
reports, not an assertion that no external/private execution ever occurred.
Root and separate review independently decoded retained O/S/U/V/W/X2: 307 transaction records and
580 CPI records contain zero `PIV1RB01` calls. O has one pending-only call;
the other five profiles have none. This is trace inspection, not new execution
or a full account-map reaudit. The hash-bound per-profile inventory is
`/tmp/piv1-m4-normalization-review-20261018-a/reviewer-retained-normalization.json`;
its parser SHA-256 is `591ede16295c181d4097aa8392c6f5c24029a8b944089c542633bdb0f117f927`.
Root's `root-retained-normalization.json` in that directory matches the review
inventory exactly as parsed JSON. No tests or runtime were rerun by either agent.

## Concrete next closure and limits

D-030's production lifecycle closure depends on this path for ordinary direct
contributions outside dedicated pending custody. Bootstrap requires zero residual
economic surplus (`src/state/bootstrap.rs:97–98`); post-settlement integration
does too (`src/state/reconciliation.rs:328–329`). Dedicated-pending recognition
cannot move those donations. This is an existing required handler lacking actual
runtime evidence, not a proposed new economic mechanism or optional M5 packaging.

Close one bounded supported active mixed-surplus production profile using actual
donor System/Token transfers, real normalization and onward settled integration.
Derive exact source obligations, pending credits and eventual HWM independently;
prove normalization itself creates no current-round/historical/KIF credit, while
later integration recognizes the full contribution in HWM exactly once. Include
exact rent/untouched accounts, repeat no-op and nonzero P-U/assigned-token offsets.
Reuse the current Bank/ELFs, with separate review and proportionate gates. Selected
late-outer rollback, identical replay and distinct retry can bind atomic custody
without claiming the separately stopped internal-callee operation.

**notChecked:** exhaustive requirements, every lifecycle/amount combination, live
governance/transport, operational provenance, Token-owned native excess destination,
general recovery exit and raw transaction decoding outside the six named profiles.
No missing economic decision is inferred for the supported scope. The blocked
internal-failure drafts were not inspected or resumed; that gap remains unwaived.
Separate code/evidence and documentary refutation passed for this bounded finding;
this is not implementation validation. M4 remains incomplete; D-034 forbids starting M5 without explicit
founder authorization. No founder acceptance or main-publication authority follows.
