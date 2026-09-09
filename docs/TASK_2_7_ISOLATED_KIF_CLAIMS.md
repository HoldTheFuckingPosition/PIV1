# Task 2.7 — Isolated KIF claim authentication and accounting

Status: **TECHNICALLY VALIDATED / PENDING FOUNDER ACCEPTANCE**.
Implementation: `10dceb5b2eac691ff19840190e951bd2ec547984`.
Baseline: `bff59bb53ebb56875a7b34ae055cc4aa8d031fb9` on
`integration/piv1-testnet`. The founder resumed the technical pilot on
2026-09-09 UTC. This bounded task follows reviewed Task 2.6 under D-026.

## Confirmed requirements and baseline gap

K-007/K-012 and master specification section 11.5 require payment only of an
already-earned guardian liability from isolated `KifSolVault`, including during
global emergency pause. The entitled guardian authorizes payment; an arbitrary
caller cannot select a different destination. Claims cannot requalify activity,
create earnings, change a distribution, consume carry or rent, or access other
economic custody. Both individual and global liabilities decrease by the same
checked amount exactly once. Failures reject atomically.

Existing `GuardianReward` validates `earned - claimed = claimable`, and Config
validates the corresponding global identity. At the baseline there was no production claim
transition or account authentication. `World::claim_effect` is a host effect
simulation; `ClaimKif` remains a marker. Task 2.4's full custody authenticator
does not authenticate guardian state and cannot establish isolated claim
liveness because it depends on unrelated accounts.

Accepted `credit_snapshot` and guardian-rotation tests preserve earnings for an
old guardian/revision after the live registry changes. A claim must authenticate
the earned record's owner, not require current registry membership or revision
equality. The current reward payload has no alternate payout field.

## Bounded contract

Authenticate only four actual AccountInfos: Config, one GuardianReward, fixed
KifSolVault and the entitled guardian account, which is both signer and fixed
payment destination. Require writable privileges for those four accounts,
the guardian signer flag, distinct account roles, fixed source, exact guardian
key and appropriate owner/executable/layout/rent checks. Reuse Task 2.4's
checked account-envelope primitives through narrow crate-private access where
appropriate. Runtime executing Program ID and Rent remain explicit trusted
inputs until the future handler obtains them from its execution context.

The supported destination is System-owned, empty-data and non-executable, with
the recorded guardian key and signer/writable flags. This guard establishes
native-spending control; it is not a native-transfer program requirement.
Do not require an on-curve key: a System-owned PDA wallet can sign through a
valid CPI. Program-owned smart wallets and nonempty destinations remain
unsupported until their control is separately validated. Do not add configurable
alternative payouts or a caller-selected destination.
No current registry, ActiveDistribution, Clock, pool, token vault or principal
account is required to pay an already-earned liability. Use initialized Config
validation, not the emergency-pause gate. Do not modify the claim marker into
a purported callable handler without the separate runtime/CPI foundation.

Previously unspecified reward PDA seed bytes are selected here as an ordinary
technical derivation under D-026:

```text
["guardian-reward", guardian_pubkey_bytes, registry_revision_le_u64, guardian_index_u8]
```

The authenticated record's immutable tuple and canonical bump must match.
No current registry equality or numeric revision freshness check applies to an
already-earned record; changing its tuple without its matching address rejects.
This preserves distinct old earned ledgers across key/revision changes; a
slot-only overwrite must not erase unpaid ownership. No registry initialization,
rotation write, ledger migration/closure or forfeiture rule is implemented.
No production Program ID or public destination is chosen. Existing registry
and reward allocations remain 210 and 84 bytes; a reward with `None` activity
requires its eight unused tail bytes to remain zero. No serialized field changes.

Add a pure checked preparation/transition for a positive claim amount and an
expected pre-claim `cumulative_claimed` counter. A stale counter rejects even
after a partial claim or later credit, without a new serialized nonce. Check
both accounting identities and selected earned/claimed/liability totals against
their global counterparts. Require native custody to cover rent plus the
entire global liability plus collective carry before and after payment.

Allow unrelated excess in KifSolVault and preserve it exactly. Requiring a
normalized source would let an unsolicited lamport block a paused claim. Derive
the exact fixed transfer and staged effects before the simulated transfer;
verify exact source debit, destination credit, unchanged rent and preservation
of carry/excess. Only four accounting fields may change: Config's global
liability/cumulative claimed and the selected reward's claimable/cumulative
claimed. HWM, earnings, activity, identity, carry and all other state stay intact.

One selected record cannot recompute the sum of every current and historical
reward ledger. The exact global invariant must be maintained by authenticated
initialization, credits and claims and independently audited at those boundaries.
Do not present local/global identity checks as a fresh full-ledger sum proof;
requiring six current rewards would incorrectly omit unpaid historical owners.

## Host composition and regressions

Use a dedicated isolated host custody fixture with one initial audit baseline,
actual AccountInfo validation and staged state/transfer commit. Do not use
`World::atomic`'s whole-pool/all-vault health check as proof of isolated liveness.
Healthy full-World lifecycle composition is supplemental; existing Task 2.3
effects and conservation oracles must not be weakened.

Cover owner/PDA/bump/discriminator/length/padding, signer/writable flags, aliases,
wrong guardian/destination/source, changed record identities, invalid individual
and global accounting, overclaim, stale-counter replay, later credit, overflow,
rent and aggregate backing deficits, protected carry/excess, zero and partial
claims, full rollback at each transfer/late commit failure and retry. Show claims
while paused and alongside active/recovery phases, for inactive/old-revision
guardians with earned balances, without unrelated account health dependencies.
Keep synthetic corrupted-state arithmetic distinct from healthy host custody.

## Scope, review and limitations

Allowed implementation: new isolated claim/account-authentication modules and
exports, narrow error/helper additions, dedicated host support/tests, and this
report. The pilot owns shared instructions, checkpoint and canonical status
documentation. Preserve existing math, adapter behavior, distribution transitions,
payload schemas, dependencies and toolchain. No actual handler/entrypoint,
System transfer CPI, production state-envelope writer, initialization, heartbeat,
governance/rotation, account closure or alternate payout implementation.

Separate scope reviewer `review_t23_final` inspected canonical requirements and
actual source and found no material economic decision blocking this path. The
reviewer confirms the ownership, isolation, backing and replay constraints above,
including the explicit destination-control and old-revision treatment. Writer
dispatch reuses the available `implement_t26_deposit` agent for Task 2.7 because
creating another agent reached the thread limit. This is actual native delegated
work, not a new founder-owned conversation. Writer and pilot validation are
recorded below; final separate review passed within the bounded scope.

Use one delegated writer and separate final source/test review. The pilot runs
locked/offline host workspace tests, doctest, default/all-feature checks,
warnings-denied documentation and targeted Git checks on frozen final source.
Record writer, reviewer and pilot evidence separately. No keys, signing, secrets,
live operations, deployments, fund movement or authority transfer are authorized
by this task. This AI-assisted review is not a professional independent audit.


## Delegated writer implementation and validation — 2026-09-09 UTC

Writer `implement_t26_deposit`, reused for this bounded Task 2.7, verified user
`jerem`, branch `integration/piv1-testnet`, and HEAD
`bff59bb53ebb56875a7b34ae055cc4aa8d031fb9`. The pilot-owned resume/scope changes
in AGENTS, the pilot checkpoint, test plan, execution plan and approved contract
were preserved. No writer Git mutation, commit or publication occurred. The
following records writer evidence only; separate final review, pilot gates and
founder acceptance are not implied.

### Implemented files and behavior

- `programs/piv1/src/state/kif_claim.rs`: pure positive-claim preparation,
  private staged plan, expected cumulative-claimed replay guard, exact fixed
  transfer, full pre-state equality, checked commit and complete source backing.
- `programs/piv1/src/kif_claim_accounts.rs`: read-only authentication of four
  actual AccountInfos, immutable reward tuple PDA/discriminator, required
  writable/signature privileges and controlled native guardian destination.
- `programs/piv1/src/accounts.rs`: only four helper visibility changes to
  `pub(crate)` for state decoding, fixed PDA validation, native balance and rent;
  existing helper bodies and full-account authentication behavior are unchanged.
- `programs/piv1/src/errors.rs`, `programs/piv1/src/lib.rs` and
  `programs/piv1/src/state/mod.rs`: eight narrow privilege/claim errors and exports.
- `programs/piv1/tests/support/kif_claim_custody.rs` and support `mod.rs`:
  dedicated four-account staged host fixture and independent initial audit.
- `programs/piv1/tests/isolated_kif_claims.rs`: 24 focused regressions.
- This report: appended writer evidence while preserving the approved contract.

Authentication enforces the canonical Config and KIF source, the reward PDA
`["guardian-reward", guardian, revision_le_u64, slot_u8]`, canonical bump, owner,
allocation, discriminator, zero padding and runtime rent. The discriminator was
independently derived with Python SHA-256 as
`[169, 109, 89, 17, 75, 171, 105, 39]`. GuardianReward and GuardianRegistry spaces
remain 84 and 210 bytes. An inactive `None` reward leaves exactly eight checked
zero padding bytes. The current registry revision and activity are not claim
eligibility inputs; both old and numerically newer immutable earned tuples are
supported without asserting live-registry provenance.

The guardian must be a writable signer with its exact stored key, System owner,
empty data and non-executable state. No on-curve requirement was added. Static
alias checks reject a guardian destination matching PIV-controlled authority,
state or economic custody roles, and reject Config's own key aliasing configured
roles. Guardian overlap with an external HTFP or Team beneficiary wallet remains
permitted and has separate successful tests. No alternate destination field,
rotation writer, ledger closure or forfeiture rule was introduced.

Authentication is structural point-in-time evidence. Preparation checks both
accounting identities, selected-component/global bounds and full
`rent + global liability + collective carry` backing, including checked sums.
It derives all four accounting changes and the exact source/destination payment
before the host transfer. Commit requires the complete captured Config/reward
pre-state and exact after-custody/floor observations, then validates backing
against the staged reduced liability and preserves source excess exactly.
The host authenticates actual staged accounts before and after transfer and
again after writing the staged envelopes; only then can the whole fixture commit.
This ordering permits zero-excess exact-backed claims, including payment of the
full aggregate liability leaving only rent plus carry.

Only Config's global claim liability/cumulative claimed and the selected reward's
claimable/cumulative claimed fields change. Full-fixture equality verifies every
other byte and lamport, including earnings, identity, activity, protected HWM,
pending values, carry, state-account rent and the original audit baseline.
All nine injected transfer/state-envelope/late-commit failures roll back entirely
and retry successfully. Later modeled credit restoring the same claimable amount
does not restore an old cumulative-claimed counter; a prepared plan also rejects
selected credit, another guardian's global claim effects or any other captured
state change.

### Actual writer commands and results

All Cargo executions used `/home/jerem/.cargo/bin/cargo +1.97.1` with
`--locked --offline`, without competing writer builds.

| Execution | Actual result |
|---|---|
| `test --package piv1 --test isolated_kif_claims --locked --offline` | Initial 22 tests PASS; one unused test import warning, subsequently removed |
| `test --package piv1 --test isolated_kif_claims --test account_authentication --test vault_reconciliation --locked --offline` | Intermediate 23 isolated +23 account +22 composition =68 tests PASS, no warnings |
| Same affected-target command after wording clarification and the snapshot/capability regression | Final **24 isolated +23 account +22 composition =69 tests PASS**, zero failed/ignored and no warnings |
| `git diff --check` | PASS on final writer source/shared tracked diff |

No command or test failed during Task 2.7 writer execution. The initial warning
was fixed explicitly; no tests or gates were disabled. No rustfmt component was
installed and no formatting pass is claimed. The unchanged Task 2.3 composition
tests, including all severe-loss corrections, remain passing. The writer froze
source and released the build slot after the final 69-test run. Final workspace,
all-feature and documentation gates belong to the pilot.

### Evidence limitations and security boundary

The independent fixture creates its four-account funding/audit baseline once.
It records later modeled credits/native funding, unsolicited excess and measured
payments as flows, without resetting that baseline. Its initial global liability
and selected earned ledger are assumed fixture state. It verifies the selected
ledger and corresponding global identities/components, not a fresh sum of every
current and historical reward account. Authenticated initialization and earning/
claim boundaries must maintain that global invariant in a future implementation.

The fixture's `credit_snapshot` reuses the authenticated record tuple to model
credit and funding for replay tests; it does not authenticate earning eligibility
or an external earning snapshot. Supplemental tests actually exercise existing
World transitions through Idle, withdrawal, funded, settled and recovery states,
and an empty-contribution lifecycle genuinely earns a reward through settlement.
They import earned/phase-compatible state into a newly funded isolated fixture;
this is **not continuous cross-World custody evidence**. The original World
transitions, `claim_effect`, whole-World audit and unrelated custody are unchanged.
Synthetic corruption of unrelated pool/principal health is labeled separately,
and those accounts never become inputs to the isolated claim path. The u64
maximum tests are pure arithmetic evidence, not claims of physically funded host
custody at those values.

Owned authentication snapshots and pure prepared plans are not runtime signer
capabilities. Executing Program ID and Rent remain trusted runtime inputs; actual
signatures, invocation, account locks, transfer CPI and production envelope writes
are deferred. Destination rent exemption is not used as spending-control evidence;
source and state rent are protected. System-owned PDA signer support is host-flag
evidence and does not demonstrate an actual signed CPI.

No Mainnet action, deployment, live network/blockchain operation, real fund
movement, key creation, signing, secret access/storage, validator setup or authority
transfer occurred. No dependency, manifest, lockfile, toolchain, math, adapter,
serialized payload, distribution transition or instruction marker was changed.
This AI-assisted implementation/review is not a professional independent audit.

## Independent pilot validation — 2026-09-09 UTC

The pilot inspected the complete production/helper diff and all 24 claim tests,
then ran the following gates after the writer froze source and released the
single build slot. These are actual pilot executions, distinct from writer
reports. All Cargo commands used `/home/jerem/.cargo/bin/cargo +1.97.1`.

| Command | Actual pilot result |
|---|---|
| `test --workspace --all-targets --locked --offline --quiet` | **255 tests PASS**, zero failed/ignored |
| `test --workspace --doc --locked --offline` | **1 doctest PASS** |
| `check --workspace --all-targets --locked --offline` | PASS |
| `check --workspace --all-targets --all-features --locked --offline` | PASS |
| `RUSTDOCFLAGS='-D warnings' ... doc --workspace --no-deps --locked --offline` | PASS |
| `git diff --check` | PASS |

Ephemeral detailed logs and all Rust/manifest input hashes are in
`/tmp/piv1-t27-pilot-20260909T095218Z`. The input file set and every source hash
remained unchanged throughout the gates. The nine-file reviewed source diff and
freeze inventory are `/tmp/piv1-t27-final-bff59bb.diff` and
`/tmp/piv1-t27-frozen-source.json`; committed source and this report are the
durable evidence, rather than relying on those temporary files.

The pilot independently recomputed the reward discriminator and verified that
manifests, lockfile, toolchain, Config/GuardianReward payloads, claim marker and
existing World source match the baseline exactly. Targeted changed-file
ownership/credential/generated-path checks passed: all files belong to `jerem`.
Effective `core.hooksPath` is unset, only sample hooks exist, and no tracked
`.github`/`.cargo` automation was found. Independent remote reads still show
integration at `bff59bb`, accepted main at `6619376` and Task 2.3 at `3677fee`.
No source correction was required by the pilot pass. Final separate review
also passed, as recorded below. The normal implementation commit is `10dceb5b2eac691ff19840190e951bd2ec547984`.
This documentation closure precedes the normal reviewed development push; verify
actual local/remote refs on takeover. Accepted main remains unchanged.


## Final separate review and technical closure

Reviewer `review_t23_final` inspected the exact nine-file source/test diff against
`bff59bb`, actual implementation/support files, all 24 test bodies and the complete
report. Verdict: **PASS within isolated authentication, pure bookkeeping and
atomic host scope; no actionable findings**. Every actual source hash matched
the frozen inventory. The reviewer independently verified the discriminator,
performed read-only inspection/hash commands and ran no builds. Writer and pilot
test results remain separately attributed above.

The review confirms historical-owner claims, immutable tuple authentication,
protected-role alias rejection, permitted external beneficiary overlap, replay
and full pre-state checks, exact custody deltas, complete aggregate backing,
unchanged carry/excess and complete host rollback. The fixture wording findings
were clarified before the final source freeze; no functional correction was
required. The evidence does not prove runtime signatures, CPI, account locking,
production state-envelope writes or a freshly recomputed historical-ledger sum.

Technical validation is not founder acceptance. The next implementation must
first be scoped from remaining canonical dependencies after this task's Git and
checkpoint closure. Initialization/state writes, current guardian activity,
real adapter mapping and actual instruction/transfer integration remain open.
No live-operation authorization was granted or used.
