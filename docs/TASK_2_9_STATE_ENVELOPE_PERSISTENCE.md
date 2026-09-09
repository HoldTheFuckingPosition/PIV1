# Task 2.9 — Validated state envelopes and atomic existing-account persistence

Status: **TECHNICALLY VALIDATED / PENDING FOUNDER ACCEPTANCE**.
Baseline: reviewed, published Task 2.8 closure
`440e83e26d36df911ccfafac97d89b79b8b4c694` on `integration/piv1-testnet`.
D-026 permits this bounded dependency after the completed Task 2.8 checkpoint.

## Requirement and useful boundary

The accepted bounded schemas and Task 2.4/2.7/2.8 production decoders require
exact discriminator-inclusive allocations and zero unconsumed Borsh tail bytes.
Only test support currently constructs these envelopes. Future claims also need
Config and GuardianReward state persisted together: a later borrow or validation
failure must not partially update an earlier account in the host primitive.

Implement typed validated encoding and staged atomic byte persistence for
`PivConfig`, `ActiveDistribution`, `GuardianRegistry` and `GuardianReward`.
Retain allocations 1014, 891, 210 and 84 bytes, existing discriminators, schemas,
versions and canonical PDA derivations. Reuse existing state validators and
checked rent/account helpers. Do not accept caller-selected discriminators,
arbitrary serializers, sizes or raw unchecked replacement bytes.

Encoding allocates a fresh fixed-size zero-filled envelope, writes the fixed
discriminator and checked Borsh payload without panics or truncation, and leaves
every unconsumed tail byte zero. Validate the typed payload before encoding.
Pure encoding is a structural utility, not approval of the represented transition.

Prepare private owned write records binding the target key, trusted executing
program identity, state type, complete canonical expected-before envelope and
validated replacement envelope. Both states must belong to the same immutable
account identity and canonical PDA/bump. In particular, historical reward tuples
retain guardian/revision/slot ownership without requiring current membership.
Config uses its initialized validator; round/registry/reward use their accepted
validators. No WithdrawalLeg codec or persistence is added: its valid Vacant
model does not prove initialized on-chain state.

Commit a statically bounded batch of prepared writes to existing AccountInfos.
Commit receives the trusted executing Program ID again and verifies every
prepared record belongs to it. Fixed registry PDA identity does not make its
revision/membership immutable; authorization of any rotation remains external.
The caller's batch shape is a Rust implementation input, never an instruction-
provided unbounded length. Recheck target binding/PDA, trusted owner, non-executable
and writable status, exact allocation, rent coverage, and the entire expected
before envelope. Reject duplicate target keys and shared mutable data backing.
All replacement encoding, validation, allocation, borrowing and length checks must
finish before the first copy. Acquire every mutable data borrow before writing
any account; retain them until all exact-size copies finish. After the first copy,
perform no fallible validation, serialization, allocation, or further borrowing.
Failures leave all accounts byte-for-byte and lamport-for-lamport unchanged.
Success changes only requested state bytes; no owner, key, lamport or allocation
mutation. Reuse narrow existing errors when accurate; add checked errors as needed.

## Authority and safety boundaries

This library utility is not an instruction entrypoint or an authorization
capability. Future handlers must supply the trusted executing program ID/Rent,
authenticate the complete relevant topology, enforce signer/governance/pause and
lifecycle rules, and derive the replacement through the proper checked transition.
Per-account structural/PDA checks do not prove cross-account economic consistency,
global historical-ledger sums, legal HWM changes or membership rotation authority.
The helper introduces no generic pause gate, exception, activity policy or custody
power. Fresh authentication remains required around CPI and actual state changes.

Do not add initialization, closure, realloc, migration, funding, lamport transfers,
handlers, CPI, Program ID, SBF/validator setup, dependencies, key creation, signing
or live actions. Do not alter accepted economics, state schemas, pure transitions,
old fixture accounting or previously reviewed authentication behavior. No-op byte
persistence is not general replay protection; claim counters remain responsible.
Runtime rollback, locking, compute/heap limits and callable instructions remain
unproven by this host AccountInfo evidence.

## Required regressions and completion gate

- Independent discriminator/size/byte expectations and round trips through existing
  decoders; all four types, including a valid active round and historical reward.
- Some-to-None, None-to-Some, maximum Option presence and repeated encodings clear
  every unused byte. Explicitly label shrinking activity/history as codec cases,
  not authorized deletion of history or activity.
- Invalid typed replacement/before state, malformed discriminator/version/Option,
  nonzero tail, allocation mismatch and stale full before bytes reject unchanged.
- Wrong key/program/owner, noncanonical PDA/bump, executable/read-only accounts,
  deficient/invalid rent and borrow conflicts reject before writes. No arbitrary
  replacement can rebind the immutable account identity.
- A bad or borrowed second/last account cannot partially update earlier accounts;
  duplicate keys and differently keyed shared-data aliases reject safely.
- Successful two-account Config/reward bookkeeping persistence derived from the
  accepted claim transition decodes through existing authentication and preserves
  unrelated fields/lamports and historical-owner compatibility. This demonstrates
  state-byte persistence only; do not fabricate an actual transfer or full handler.
- No-op behavior and stale replay limitations are explicit. Existing account,
  guardian/Clock, claim and custody regressions remain unchanged and passing.

Use one existing delegated writer and a separate read-only reviewer. Writer owns
new codec/persistence code/tests, narrow helper/error/export edits and its report
append; pilot owns shared docs and Git. No competing builds. Writer runs focused
locked/offline tests, reports actual attempts, freezes source and releases build
slot. Pilot inspects actual source/tests and runs the established final locked/
offline workspace tests, doctest, default/all-feature checks and warnings-denied
documentation on frozen inputs. No Rustfmt installation or unsupported PASS claim.
Final separate review must cover exact diff, all new tests and complete evidence.
Then commit, checkpoint and publish only the reviewed development branch. Technical
validation is not founder acceptance or a professional independent audit.


## Coordination checkpoint

Separate read-only assessment and exact written-scope review by
`review_t23_final` passed without a required correction. The pilot incorporated
its Program-ID-at-commit and immutable-PDA-versus-registry-membership clarifications.
The existing `implement_t26_deposit` agent is reused as sole Task 2.9 writer;
`review_t23_final` remains the separate reviewer. No agent creation is claimed.
The writer receives the single focused build slot; the pilot will wait for source
freeze before running final gates. Shared documentation and Git belong to the
pilot. No Task 2.9 implementation or test result is claimed at dispatch.

## Writer implementation evidence — 2026-09-09 UTC

Writer: `implement_t26_deposit`, reused as the sole delegated writer. Entry
verification found user `jerem`, branch `integration/piv1-testnet`, HEAD
`440e83e26d36df911ccfafac97d89b79b8b4c694` and the expected pilot-owned documentation
changes. The writer read the current repository instructions, pilot checkpoint,
D-026 mandate, approved report contract, applicable canonical requirements and
existing state/authentication/claim/custody source. Shared documents and Git remain
pilot-owned. Written-scope approval is distinct from final implementation review,
pilot validation and founder acceptance.

The writer changed these files only:

- `programs/piv1/src/state_persistence.rs` (new): closed typed envelope constructors,
  private prepared records and atomic fixed-batch byte persistence.
- `programs/piv1/src/lib.rs`: module export and accurate library description.
- `programs/piv1/src/errors.rs`: two narrow envelope errors and general account
  writable/borrow wording that also describes persistence.
- `programs/piv1/tests/state_persistence.rs` (new): 21 focused host regressions.
- This report: append-only writer implementation/validation evidence.

No accepted state validator, transition, authentication helper, serialized schema,
World/claim support, oracle, existing test, instruction marker, adapter, manifest,
lockfile or toolchain file was changed by the writer.

### Implementation and public boundary

`StateEnvelope::{config, distribution, registry, reward}` accepts only its named
typed state and invokes the accepted initialized/standalone validator. There is no
public raw-byte, discriminator, allocation-size or generic-serializer input. The
private encoder fallibly reserves a fixed allocation, zero-fills it, writes the
existing discriminator, and serializes into a bounded mutable slice. Insufficient
capacity or allocation failure returns `StateEnvelopeEncodingFailed`; successful
serialization leaves every unused byte zero. The four allocations remain exactly
1014, 891, 210 and 84 bytes. Encoding does not authenticate a PDA or authorize the
represented state change.

`PreparedStateWrite::new` consumes two validated envelopes and binds their entire
bytes, state identity, target key and trusted executing program. It checks both
canonical PDAs/bumps and requires equal immutable identity. That identity is the
fixed Config, distribution or registry seed/bump, or the complete historical
reward guardian/revision/slot/bump tuple. Registry membership and revision are
payload content; the fixed registry address neither freezes nor authorizes them.
Read-only getters expose the target and complete before/replacement bytes, without
allowing mutation of private records.

`commit_state_writes<const N: usize>` receives the trusted executing program and
Rent again, plus a fixed Rust array of prepared-record/account pairs. It verifies
every record's program, rejects duplicate keys and shared data `Rc` backing,
rechecks target/PDA/owner/non-executable/writable/rent, and then acquires all mutable
data borrows. While holding them, it checks exact allocations and every canonical
expected-before byte. Any differing current bytes, including a malformed payload
or a structurally valid stale value, return `StateEnvelopeChanged`. An exact match
to the private validated envelope needs no second deserialization. All successful
borrows remain held until every copy finishes. The final loop contains only
previously size-checked copies; no serialization, allocation, further borrowing or
fallible validation follows the first copy. Existing account lamports, owner, key,
flags and allocation are never modified.

Empty and no-op batches are supported. Full-byte optimistic matching detects stale
records, but a repeatable no-op is not a general replay oracle; claim counters and
authorized lifecycle rules remain required. The utility supplies no pause,
heartbeat, governance, rotation, earning, legal-HWM or cross-account economic
policy. Trusted runtime context and fresh complete account authentication belong
to future handlers. There is no WithdrawalLeg writer.

### Focused evidence and limits

The 21 tests cover:

- Independently pinned discriminators, sizes, Config/round byte prefixes and
  Option boundaries, and complete little-endian registry/reward byte expectations.
  All four envelopes round-trip through existing production readers, including a
  valid active-round payload imported from the unchanged World model.
- Repeated Some/None growth/shrink for Config timestamps, completed history and
  guardian activity, maximum Option presence, exact zero tails, and no-op writes.
  History/activity shrinking is explicitly a synthetic codec case, not authorized
  deletion. Synthetic Config/history pairs retain the existing sequence binding.
- Invalid initialized/typed before or replacement values; malformed discriminator,
  version and Borsh Option tags; nonzero tails; short/long allocations; and valid
  stale fields in each record. The Config Option tag is independently pinned at
  byte 756, and tag 2 is also rejected directly by the Borsh decoder.
- Wrong target/program/owner, forbidden canonical program identities, wrong bumps,
  valid but noncanonical off-curve PDAs, executable/read-only accounts, deficient
  rent, invalid Rent parameters and checked rent overflow. A foreign-program
  prepared record in the last batch position rejects earlier eligible writes.
- Shared/mutable data conflicts and mutable lamport conflicts in every position;
  all earlier temporary borrows are released on failure. Shared lamport reads and
  native excess permit byte-only writes. Every failure comparison preserves all
  bytes, metadata and lamports, with a genuinely changed first replacement.
- Duplicate keys with distinct data and differently keyed synthetic accounts
  sharing one data `Rc`, every pair among four positions, plus duplicate plans.
- Exact successful one-, two- and four-account writes; complete immutable reward
  tuple protection; structural registry content changes without claiming rotation
  authority or synchronized current-registry authentication.
- Config/reward persistence derived from the accepted pure claim transition for
  an inactive historical guardian record. Synthetic pure after-observations derive
  bookkeeping only. All four actual fixture lamport balances and all unrelated
  account fields remain unchanged, and existing claim authentication decodes the
  persisted state. The original audit baseline/counters remain unchanged and the
  test explicitly expects that audit to reject bookkeeping without payment. This
  is not an actual transfer, continuous conservation PASS, or claim handler.

Auxiliary accounts used for decoder compatibility are newly constructed host
fixtures. Imported World payloads do not create continuous custody evidence.
Neither successful fixed-account structural authentication nor current-six reward
decoding proves physically balanced custody or a global historical ledger sum.
The tests do not inject allocator exhaustion; the fallible reserve/serialization
error path is source-inspected. Runtime rollback/locking, SBF compute/heap limits,
transaction privileges and callable instructions remain unproven.

### Writer commands and actual outcomes

All builds used `/home/jerem/.cargo/bin/cargo +1.97.1` and `--locked --offline`:

1. `test --package piv1 --test state_persistence --locked --offline` initially
   failed compilation with six fixture lifetime errors. The fixture combined
   closure-bound AccountInfos with captured auxiliary accounts; construction was
   moved into shared explicit local scopes. No production compile error occurred.
2. The same focused command then passed **19 tests**, zero failed/ignored, with
   no warnings. Two further independent-byte/stale-field regressions were added.
3. `test --package piv1 --test state_persistence --test account_authentication
   --test guardian_clock_authentication --test isolated_kif_claims
   --test vault_reconciliation_model --locked --offline` failed before building
   because `vault_reconciliation_model` is not a test target. The correct target
   name is `vault_reconciliation`.
4. `test --package piv1 --test state_persistence --test account_authentication
   --test guardian_clock_authentication --test isolated_kif_claims
   --test vault_reconciliation --locked --offline` passed **112 tests**:
   21 persistence, 23 fixed-account, 22 guardian/Clock, 24 isolated-claim and
   22 vault-reconciliation tests. Zero failed/ignored; no warnings.

Routine read-only inspection used `whoami`, `git branch --show-current`,
`git rev-parse HEAD`, `git status --short`, `rg`, `cat` and `sed`. One initial source
read used the nonexistent plural `state/guardians.rs`; it was corrected to the
existing `state/guardian.rs`. `git diff --check` passed for the shared tracked diff.
Rustfmt is unavailable; no component was installed and no formatting PASS is
claimed. No tests, warnings or gates were suppressed.

### Frozen source and handoff

After the final 112-test run, the writer froze these exact source hashes and
released the build slot before the pilot's independent gates. Inventory:
`/tmp/piv1-t29-writer-frozen-source.json`.

| File | SHA-256 |
|---|---|
| `programs/piv1/src/state_persistence.rs` | `88c59578cfd7ab8461614bc4b7f3051c128dc776eda9a6d820000bd2375de2e8` |
| `programs/piv1/src/errors.rs` | `2aa1f8abda9e16465a23a426fded03799b8ce2fd027e04de2c9adec4be5bbb8e` |
| `programs/piv1/src/lib.rs` | `4a31f5b34d96d5e5b90075e4df288b96724ea8458ee8f1aaf67789a3e97182e0` |
| `programs/piv1/tests/state_persistence.rs` | `3330d62f7074d53bbedfd71b49c2f7fd0da22bb905cef6e3269d38b5074dd139` |

The final writer checkpoint retains branch `integration/piv1-testnet` and HEAD
`440e83e26d36df911ccfafac97d89b79b8b4c694`. The writer made no Git mutation or
commit. Expected dirty pilot documentation remains preserved; writer code/report
is pending parent review/checkpoint. Independent pilot workspace gates and the
separate final review are separate evidence, to be recorded by the pilot.

No Mainnet action, deployment, validator setup, live blockchain operation, fund
movement, key creation, signing, secret access or authority transfer occurred.
This host implementation and AI-assisted review are not founder acceptance,
runtime/Testnet evidence or a professional independent audit.


## Independent pilot validation — 2026-09-09 UTC

The pilot inspected the complete four-file production/test change and all 21
new tests, including the final independent byte-prefix and valid-stale-state
cases. Preliminary test inspection corrected the Config Option tag from byte
760 to 756, added direct malformed Borsh rejection and aligned the synthetic
Idle completed-summary sequence with Config. The writer independently corrected
the tag during this review and fixed its fixture lifetime errors. No production
correction was required; no accepted validator, transition or oracle was weakened.

After source freeze and release of the single build slot, the pilot actually ran
the following commands using `/home/jerem/.cargo/bin/cargo +1.97.1`. These are
pilot executions, distinct from the writer's reported attempts and 112-test run.

| Command | Actual pilot result |
|---|---|
| `test --workspace --all-targets --locked --offline --quiet` | **298 tests PASS**, zero failed/ignored |
| `test --workspace --doc --locked --offline` | **1 doctest PASS** |
| `check --workspace --all-targets --locked --offline` | PASS |
| `check --workspace --all-targets --all-features --locked --offline` | PASS |
| `RUSTDOCFLAGS='-D warnings' ... doc --workspace --no-deps --locked --offline` | PASS |
| `git diff --check` | PASS |

No gate log contains a compiler warning or error. All Rust/manifest input hashes
and the input file set remained unchanged throughout the gates. Detailed
summaries/logs and the input inventory are in
`/tmp/piv1-t29-pilot-20260909T104827Z`. The exact final four-file diff including
both new files is `/tmp/piv1-t29-final-440e83e.diff`; parent freeze inventory is
`/tmp/piv1-t29-frozen-source.json`, matching the writer inventory. Committed source
and this report provide durable evidence; temporary paths may disappear.

The pilot independently verified that existing account readers, state models,
transitions, instruction markers, adapters, old support/tests, manifests, lockfile
and toolchain remain unchanged. Targeted changed-file ownership, credentials and
generated-path checks passed; all task files belong to `jerem`. Effective
`core.hooksPath` is unset, hooks are sample-only and no tracked `.github`/`.cargo`
automation exists. An independent remote read still shows integration at
`440e83e26d36df911ccfafac97d89b79b8b4c694`, accepted main at
`66193769d1cbc59cd8630df295b9a784b9c64642` and Task 2.3 at
`3677fee97e3617ee65e2828d222008ba0952bb3e`.

The pilot owns shared status/evidence updates in `AGENTS.md`, the execution plan,
master specification, pilot checkpoint and test plan, plus completed Task 2.8
publication evidence. Final separate review passed as recorded below; normal
Git/publication closure is in progress. No Mainnet action, deployment, fund movement, key creation,
signing, authority transfer or unrelated secret access occurred. Host byte
persistence is not runtime transfer/handler evidence or founder acceptance.


## Final separate review and technical closure

Reviewer `review_t23_final` inspected the exact four-file diff against `440e83e`,
all 21 tests and the complete writer/pilot evidence. Verdict: **PASS within the
validated-envelope and atomic state-byte persistence scope; no actionable
findings**. All actual hashes matched both freeze inventories, and the captured
patch matched actual source. The reviewer independently confirmed unchanged
prior schemas, transitions, readers, oracles and dependencies, and inspected the
pilot results file. The reviewer ran no builds; execution attribution is retained.

The review confirms zero tails, canonical immutable identity, complete before
matching, all mutable borrows held before copying and later-account rejection
without earlier partial writes. Historical reward ownership remains intact.
The byte-only claim example deliberately retains actual lamports/audit counters
and expects the unchanged custody audit to reject bookkeeping without payment.
No production correction was required after freeze. Authorization, cross-account
economics, real transfers, runtime rollback/locking and SBF resource limits remain
separate dependencies.

Changed files: the four source/test files listed above, this report,
`AGENTS.md`, `docs/PIV1_MASTER_SPEC.md`, `docs/PIV1_CODEX_EXECUTION_PLAN.md`,
`docs/PIV1_PILOT_STATE.md`, `docs/PIV1_TEST_PLAN.md` and the Task 2.8 report's
publication evidence. Normal implementation commit/hash recording and reviewed
integration publication follow this checkpoint. Founder acceptance remains
pending; no live-operation authorization was granted or used.
