# Task 2.8 — Current guardian and Clock snapshot authentication

Status: **TECHNICALLY VALIDATED / PENDING FOUNDER ACCEPTANCE**.
Baseline: reviewed and published Task 2.7 closure
`37f25a8b84e0d4060b36fe0c86ff8bea8e4aa3aa` on `integration/piv1-testnet`.
D-026 authorizes this bounded dependency after the completed Task 2.7 checkpoint.

## Requirements and scope

K-010 and master specification sections 11.2–11.3 derive KIF activity from the
configured anchor and Solana Clock, using exact 2,592,000-second half-open periods.
The accepted `derive_kif_period` and `GuardianRegistry::activity_bitmap` already
implement pure timing and six-key eligibility. They still receive unauthenticated
host state. Task 2.7 authenticates an earned owner for claims, deliberately without
current membership; future distribution preparation instead needs the exact
current registry and its six current reward/activity records.

Implement one read-only authentication path with trusted executing Program ID
and Rent plus exactly nine actual AccountInfos: Config, current GuardianRegistry,
six current GuardianReward accounts ordered by registry slot, and canonical Clock.
Return owned validated states and Clock, the derived KIF period, six-bit activity
bitmap and active count. No signer or writable privilege is required for reading.
Success and rejection preserve every input byte and lamport. Pause and active
round state do not prevent inspection; this does not authorize economic execution
or decide heartbeat callability during pause.

Authenticate canonical Config, its initialized payload/program-ID constraints
and static role separation. For the registry, the previously unspecified fixed
seed is selected as the compatible technical derivation `[b"guardian-registry"]`
under D-026. The canonical address, Config reference and stored bump, registry
own bump and exact revision must agree. This does not select a production Program
ID or deployed address. Registry allocation remains 210 bytes, with discriminator
SHA-256(`account:GuardianRegistry`)[0..8] = `[72, 14, 254, 2, 76, 233, 97, 92]`.

Each current reward must have the existing Task 2.7 immutable guardian/revision/
slot PDA, canonical bump, exact owner and unchanged 84-byte envelope. Validate
its standalone earned-minus-claimed identity and exact current registry binding.
Do not overwrite or invalidate historical reward accounts: they remain entitled
to Task 2.7 claims but cannot substitute for current activity inputs. Current
records do not enumerate all historical earnings or liabilities. This read-only
path does not freshly prove global ledger sums or earning provenance, and must
not require six current claimable balances to equal global KIF liability.

Reject pairwise supplied-account aliases and static PIV-controlled state,
authority or custody role aliases, without loading unrelated accounts. Preserve
the permitted overlap between a guardian wallet and an external HTFP/Team
beneficiary. No new wallet ownership or on-curve requirement is inferred from
read-only registry inspection. Future initialization and heartbeat handlers must
authenticate actual membership authority and signing control.

Reuse existing checked account-envelope/rent helpers and Task 2.7 reward identity
rules through narrow crate-private sharing where useful. Apply checked runtime
rent to PIV-owned state accounts, with exact allocations, discriminators, Borsh
payloads and zero unconsumed padding. Config/reward/registry schemas, seeds already
selected for other roles and existing economic transitions remain unchanged.

## Clock decoding and timing boundary

The locked stack contains Anchor 0.32.1, solana-clock 2.2.3, solana-sysvar 2.3.0,
solana-sdk-ids 2.2.1 and bincode 1.3.3. Anchor already exposes Clock and the Sysvar
trait with the bincode feature; no new dependency or upgrade is needed.

Use the pinned official Clock decoder after checking its canonical account key,
canonical sysvar owner, non-executable status and exact 40-byte serialized shape.
The pinned `Sysvar::from_account_info` checks only the key and then uses an
infallible data borrow and Bincode. Perform a fallible borrow preflight so a
conflicting borrow returns a checked error instead of panicking. Tests may use
the pinned official sysvar serialization helper for fixture bytes, supplemented
by independent field/length checks. Do not invent a Clock rent requirement or
require a lamport borrow when no Clock lamports are read.

Derive the period only from the authenticated Clock `unix_timestamp` and stored
Config anchor through the accepted timing helper. Return the complete Clock
alongside the period, rather than silently replacing it with caller time. No
extra epoch/slot relationship or timestamp policy is added. Exact activity
semantics remain unchanged: None, older and newer stored activity are all
inactive for the queried period. The accepted model deliberately permits
querying period 40 after activity recorded in period 41. Monotonic activity
recording is a separate writer obligation, not a new authentication rejection.

Owned snapshots are point-in-time evidence, not reusable runtime capabilities.
Later account/activity/Clock changes require fresh authentication for future
execution. No handler, heartbeat, snapshot creation, reward credit, transfer,
CPI, governance/rotation, initialization, state writer, signing or live operation
is implemented by this task. Runtime authenticity and account locking are not
proven by host AccountInfo flags or fixture bytes.

## Meaningful regressions and completion gate

Cover all 64 activity masks and exact active counts; None/older/current/newer
activity; half-open period edges, negative anchors, timestamp regression and
checked overflow; exact current registry revisions and wrong old/future records;
reordered/duplicated/wrong-slot rewards; owner/executable/size/discriminator/
padding/rent/PDA/bump and borrow failures for every applicable role; static alias
rejection and permitted external-beneficiary overlap; malformed/wrong-owner/
wrong-ID/short/long Clock with no invented rent requirement; historical unpaid
liability alongside zero current rewards; and read-only success/failure with
pause or unrelated round/custody health. A later activity update must preserve
an earlier owned snapshot while fresh authentication reflects the new activity.
Do not alter existing World oracles or call a newly funded fixture continuous
custody evidence. No signing or validator setup is needed for these tests.

Separate scope reviewer `review_t23_final` inspected canonical requirements,
accepted source and pinned Clock code and found no material economic decision
blocking the scope. The pilot independently confirmed the Clock decoder's
owner/length/borrow gaps, available locked reexports and registry discriminator.
One delegated writer and separate final review must inspect actual implementation
and tests. The pilot then runs frozen-source locked/offline workspace tests,
doctest, default/all-feature checks and warnings-denied documentation, plus Git
scope/ownership/credential/hook checks before normal commits/publication.

The writer may edit new authentication/tests, narrow shared authentication
helpers/exports/errors and append evidence here. The pilot owns shared status
and canonical documentation and all Git operations. Preserve accepted economics,
state layouts, dependencies and existing tests. Final technical validation is
not founder acceptance, and AI-assisted review is not a professional audit.


## Coordination checkpoint

The exact written scope passed separate reviewer inspection. The available
`implement_t26_deposit` agent is reused as the sole Task 2.8 writer; this is
actual native delegation, with `review_t23_final` reserved for separate review.
The pilot owns shared documentation and Git. Writer and pilot evidence are
recorded below; the final separate review passed with no actionable findings.
Current-six aggregate component checks may be appropriate at a later accounting
boundary, but are unnecessary for this bounded structural eligibility reader.


## Delegated writer implementation and validation — 2026-09-09 UTC

Writer `implement_t26_deposit`, reused for Task 2.8, verified user `jerem`, branch
`integration/piv1-testnet` and baseline HEAD
`37f25a8b84e0d4060b36fe0c86ff8bea8e4aa3aa`. Expected pilot-owned checkpoint,
plan and Task 2.7 publication documentation were preserved. The separate scope
approval authorized implementation only; the following is writer execution
evidence, not final separate review, pilot validation or founder acceptance.

### Files and resulting boundary

- `programs/piv1/src/guardian_clock_accounts.rs`: read-only nine-account
  authentication and owned Config/registry/six-reward/complete-Clock output with
  derived KIF period, activity bitmap and active count.
- `programs/piv1/src/kif_claim_accounts.rs`: extract the existing standalone
  reward envelope/identity/PDA block into one crate-private helper. Validation
  order and all existing claim behavior remain unchanged; current membership
  is checked only by the new snapshot path.
- `programs/piv1/src/lib.rs`: export the new authentication module.
- `programs/piv1/src/errors.rs`: one narrow canonical Clock-key error.
- `programs/piv1/tests/guardian_clock_authentication.rs`: nine-account read-only
  fixture and 22 focused regressions.
- This report: appended writer evidence without rewriting the approved scope.

`authenticate_guardian_clock_snapshot` authenticates only Config, its exact
current registry, six ordered current reward accounts and canonical Clock.
The fixed registry PDA seed is `guardian-registry`; Config's reference/bump and
the registry bump/revision agree. The existing reward tuple seeds and all
serialized layouts remain unchanged. Python SHA-256 independently reproduced the
registry discriminator `[72, 14, 254, 2, 76, 233, 97, 92]`. State envelopes retain
exact allocation, discriminator, checked runtime rent, variable Borsh and zero
unconsumed padding requirements. Valid off-curve but noncanonical registry or
reward PDA/bump alternatives reject.

Pairwise supplied accounts must be distinct. Static checks protect Config's own
role separation, PIV-controlled role separation and guardian keys against supplied
PIV state-account aliases. Guardian overlap with external HTFP/Team wallets
remains allowed; no actual guardian wallet account, signing privilege, wallet
owner or on-curve test is required for inspection. No unrelated pool, custody,
active round or historical reward enumeration is fetched. Pause and writable/
signer flags do not change read-only inspection.

### Clock and eligibility evidence

The writer inspected local locked Anchor 0.32.1, solana-clock 2.2.3,
solana-sysvar 2.3.0 and solana-sdk-ids 2.2.1 sources. Existing Anchor
`Clock`/`SolanaSysvar` reexports expose the official pinned Bincode decoder and
fixture serialization helper. No dependency addition, upgrade, custom Clock
parser or deprecation suppression was needed.

Before `Clock::from_account_info`, the new path checks canonical Clock key,
canonical sysvar owner, non-executable state and exactly 40 data bytes. A fallible
immutable data borrow preflights conflicts and remains held through the official
decoder's nested immutable borrow. A mutable data conflict returns the checked
borrow error instead of reaching the pinned decoder's infallible borrow panic.
No Clock lamports are read and no Clock rent requirement is invented: zero
lamports, one lamport, maximum lamports and an outstanding mutable Clock-lamport
borrow all remain compatible with inspection. Tests independently verify all
five serialized integer field offsets and preserve the complete Clock output.
No extra slot, epoch or epoch-start relationship is imposed.

The accepted `derive_kif_period` and `GuardianRegistry::activity_bitmap` remain
unchanged. All 64 masks and exact active counts are covered. None, older and
newer recorded activity are inactive for the queried period. A period-40 query
after activity at period 41 succeeds as inactive, then a fresh period-41 Clock
read reports it active. Half-open edges, negative anchors, timestamp regression
and checked signed overflow are covered. An earlier owned snapshot remains
unchanged after modeled activity/Clock byte updates; future execution must use
fresh authentication.

Current records receive standalone ledger identity checks and exact current
membership checks only. Their balances are not equated to global KIF liability,
and no new earning-eligibility or global-economic bound is introduced. A fixture
with global unpaid liability 960 and six zero current reward balances succeeds.
An old tuple cannot substitute for a current activity record but still succeeds
through the unchanged isolated Task 2.7 claim path.

### Actual writer commands and results

All Cargo executions used `/home/jerem/.cargo/bin/cargo +1.97.1` and
`--locked --offline` with the sole writer build slot.

| Execution | Actual result |
|---|---|
| `test --package piv1 --test guardian_clock_authentication --locked --offline` | Initial **21 tests PASS**, no failures or warnings |
| `test --package piv1 --test guardian_clock_authentication --test account_authentication --test isolated_kif_claims --locked --offline` | Final **22 snapshot +23 fixed-account +24 isolated-claim =69 tests PASS**, zero failed/ignored/warnings |
| `git diff --check` | PASS on the final shared tracked diff |

No command or test failed during writer execution. The final added regression
covers alternate valid-but-noncanonical registry/reward PDA bumps. All existing
Task 2.4 and 2.7 authentication regressions pass after helper extraction. No
rustfmt component was installed and no formatting pass is claimed. Source was
frozen and the build slot released after the final 69-test run; workspace,
doctest, all-feature, documentation and final separate review results belong to
the pilot/reviewer and are not writer execution claims.

### Limits, Git and sensitive actions

The fixture reuses only a valid Config value and existing host envelope helpers;
its nine snapshot AccountInfos are the complete actual inputs. Fixture byte
construction/corruption is not initialization, rotation or heartbeat execution.
Synthetic unrelated custody quantities are explicitly labeled; no physical
backing or continuous cross-World custody proof is claimed. Existing World,
claim support and conservation oracles were not modified. Structural reads do
not prove live earning provenance or the sum of all current/historical ledgers.

Runtime executing Program ID and Rent remain trusted inputs. Clock account bytes
and AccountInfo flags are host evidence, not proof of runtime sysvar authenticity,
locking or invocation. Owned output is not an execution capability. No heartbeat
pause policy, handler, distribution snapshot creation, earning credit, transfer,
CPI, governance, state writer, initialization or public deployment is implemented.

HEAD remained `37f25a8b84e0d4060b36fe0c86ff8bea8e4aa3aa`; the writer made no
Git mutation, commit or publication. No Mainnet action, deployment, live network
or blockchain operation, fund movement, key creation, signing, secret access/
storage, validator setup or authority transfer occurred. Dependencies, manifests,
lockfiles, toolchain, serialized payloads, math, adapters and economic transitions
remain unchanged. This AI-assisted work is not a professional independent audit.


## Independent pilot validation — 2026-09-09 UTC

After the writer froze source and released the build slot, the pilot inspected
the complete five-file source/test diff and every one of the 22 test bodies,
including valid noncanonical PDA alternatives. The pilot independently ran these
gates with `/home/jerem/.cargo/bin/cargo +1.97.1`; they are distinct from writer
execution reports.

| Command | Actual pilot result |
|---|---|
| `test --workspace --all-targets --locked --offline --quiet` | **277 tests PASS**, zero failed/ignored |
| `test --workspace --doc --locked --offline` | **1 doctest PASS** |
| `check --workspace --all-targets --locked --offline` | PASS |
| `check --workspace --all-targets --all-features --locked --offline` | PASS |
| `doc --workspace --no-deps --locked --offline` with `RUSTDOCFLAGS='-D warnings'` | PASS |
| `git diff --check` | PASS |

Ephemeral logs/results and all source input hashes are in
`/tmp/piv1-t28-pilot-20260909T102244Z`. The input file set and every source hash
remained unchanged throughout the gates. The exact five-file review diff and
freeze inventory are `/tmp/piv1-t28-final-37f25a8.diff` and
`/tmp/piv1-t28-frozen-source.json`. Committed source and this report provide the
durable task evidence; temporary artifacts are supplementary.

The pilot independently checked locked Clock source and the registry
discriminator. Baseline comparisons confirm no change to manifests, lockfile,
toolchain, generic account helpers, accepted state/timing/math/adapters,
instruction markers or old World/claim support and tests. Targeted changed-file
ownership/credential/generated-path checks passed, all files owned by `jerem`.
Effective `core.hooksPath` remains unset, only sample hooks exist, and no tracked
`.github`/`.cargo` automation was found. Independent remote reads still show
integration `37f25a8` and accepted main `6619376` before this task's publication.

The pilot updates `AGENTS.md`, master specification status, execution plan,
pilot checkpoint and requirements-to-evidence index, preserving all economic,
acceptance and sensitive-action boundaries. Task 2.7 publication evidence is also
recorded. No functional correction was required by the pilot source/test pass.
Final separate review passed as recorded below. Normal Git/publication closure is in progress.


## Final separate review and technical closure

Reviewer `review_t23_final` inspected the exact five-file source/test diff against
`37f25a8b84e0d4060b36fe0c86ff8bea8e4aa3aa`, all 22 new test bodies and the complete
writer/pilot report. Verdict: **PASS within the read-only guardian/Clock
snapshot authentication scope; no actionable findings**. All five actual source
hashes matched the frozen inventory and captured patch. The reviewer performed
read-only source/hash/diff checks and inspected the pilot results file; no
reviewer build or test execution is claimed.

The review confirms canonical identities/bumps, exact current membership,
historical claim compatibility, protected-role separation, checked Clock decoding
and borrowing, half-open timing, exact activity equality and unchanged input
state. Structural snapshots do not prove global historical-ledger reconciliation,
earning provenance, authorized heartbeat/rotation, runtime authenticity or
callable handlers. No source correction was required after freeze.

Changed files: the five source/test files listed in the writer section, this
report, `AGENTS.md`, `docs/PIV1_MASTER_SPEC.md`,
`docs/PIV1_CODEX_EXECUTION_PLAN.md`, `docs/PIV1_PILOT_STATE.md`,
`docs/PIV1_TEST_PLAN.md` and the Task 2.7 report's publication evidence.
The implementation hash and publication checkpoint are recorded after normal
Git closure. Technical validation is not founder acceptance or a professional
independent audit. No Mainnet action, deployment, fund movement, key creation,
signing, authority transfer or unrelated secret access occurred.
