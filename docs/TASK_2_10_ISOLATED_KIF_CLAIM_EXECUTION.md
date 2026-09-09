# Task 2.10 — Isolated KIF claim execution and explicit host invocation evidence

Status: **TECHNICALLY VALIDATED / PENDING FOUNDER ACCEPTANCE**.
Baseline: reviewed, published Task 2.9 closure
`44d70ec911ad3a78738fb90a04ac82eec3ca44f2` on `integration/piv1-testnet`.
D-026 authorizes the next bounded dependency after its completed checkpoint.

## Requirement and scope

Connect the completed isolated AccountInfo authentication, checked claim plan and
state-envelope persistence in a concrete claim execution function. K-012 and
master section 11.5 require guardian-controlled fixed payments, existing earned
liabilities only, full backing and replay checks, availability during emergency
pause, and checks-effects-interactions. This task adds execution code plus explicit
host invocation modeling; it does not turn the library scaffold into a deployed
or externally callable Solana program.

Use exactly Config, the immutable earned GuardianReward, fixed KifSolVault,
the corresponding guardian signer/destination, and the canonical executable
System Program account. Reuse Task 2.7 authentication and request fields; require
all existing writable/signature/owner/PDA/rent/alias restrictions. Reject wrong or
non-executable System Program and supplied role aliases before any effect. No
registry, Clock, pool, round, other custody or caller-chosen recipient is loaded.
Executing Program ID and Rent remain trusted runtime inputs; eventual entrypoint
wiring and Rent acquisition are separate. No new economic, activity or pause rule.

Before persisting any bookkeeping, preflight mutable data AND lamport borrows
for both native transfer accounts, then release every guard. Immutable reads in
the existing authentication alone cannot reject a held shared native borrow;
the official signed CPI requires mutable access to both transfer accounts.
Preexisting shared/mutable native borrow conflicts must reject before state writes.

Prepare the accepted claim and all valid replacement envelopes before interaction.
Add only a narrow internal accessor for the plan's already-validated staged
Config/reward and expected custody, avoiding duplicate arithmetic. Expected
custody is a prediction, never an observed receipt. Persist the two complete state
envelopes before CPI to satisfy checks-effects-interactions. The CPI callback
must observe updated liabilities and unchanged unrelated state. Release all
account borrows before invoking. Use the existing persistence primitive and
preserve every accepted pure validator and transition outcome.

Build the pinned official System transfer instruction with exactly the derived
source, fixed guardian destination and positive checked amount. Invoke only that
System transfer, signing with one seed group `[b"kif-sol", canonical_bump]` under
the executing PIV1 program. PivAuthority is not the System-owned source signer.
The transfer has no SPL slippage variant; protection is exact native amount,
endpoints, full source backing and postconditions. The invocation receives only
the two transfer accounts and the canonical System Program account; no unrelated
state/custody is exposed to it. Use the locked official instruction constructor
and signed invocation; add no dependencies or arbitrary-program routing.

After CPI success, freshly authenticate actual claim state/custody, require exact
staged Config/reward bytes and the predicted source/destination/floor observations,
and recheck the accepted claim postconditions against the original states. Return
success only for the exact payment with carry/rent/original excess preserved.
Every CPI or postcheck error must propagate to the eventual transaction boundary;
never turn failed/missing/wrong payment into success or an earned claim receipt.
Preserve original error information through a narrow execution error if needed.

## Pinned host limitation and explicit invocation seam

Parent and reviewer inspected the actual locked sources: Anchor 0.32.1 reexports
solana-invoke 0.4.0, whose non-Solana syscall path is `unimplemented!`; the separate
solana-cpi 2.2.1 host path returns success without a transfer. Neither supplies
host runtime evidence. Do not invoke them on host and call the result payment.
Do not assume sysvar syscall stubs intercept Anchor CPI.

The ordinary runtime-facing function must be fixed to the canonical official
System invocation. On non-Solana hosts, that ordinary path must reject explicitly
before state mutation or CPI, avoiding a panic or silent success. Keep its pinned
invocation wiring type-checked by ordinary builds; actual SBF execution is deferred.
Provide a narrow explicit host-only test seam to record and emulate the same
fully derived instruction, account list and signer seeds through the shared
execution logic. No caller-selected invocation backend may be routed by future
instruction data or exposed as a Solana production path. Rust test API visibility
is a compatible implementation choice; avoid warning suppression or a new feature
solely to conceal untested code.

The execution primitive relies on Solana transaction rollback after CPI/postcheck
failure; it must not claim to rewind arbitrary CPI effects by itself. A dedicated
host transaction fixture may clone all modeled state/accounts, run the real shared
execution logic with the explicit System-transfer emulator and commit the clone
only on success. Preserve the original immutable audit baseline and record actual
modeled native payment flows once. Discard failed staged transactions completely.
Tests/report must distinguish that modeled rollback from actual SVM rollback and
must make clear that callers cannot catch/ignore execution errors and commit state.

## Required regressions and invariants

- Independently pin the official System transfer bytes and metas, exact source/
  destination/amount, one correct seed group and account list. No custom PIV
  instruction ABI/discriminator or live Program ID is selected here.
- Positive partial/full claims, historical inactive owners after rotation, global
  pause and unrelated Config phase/history fields preserve the accepted isolation.
  Assert exactly four accounting-field changes, exact native deltas, conserved
  totals and unchanged carry, rent, unsolicited excess and unrelated accounts.
- Every invalid request/account/backing/replay/overflow/borrow/encoding case rejects
  with zero invocations and no earlier state effect. Reuse existing tests where
  they already prove a lower-level condition; exercise the composed boundary.
  Explicitly hold shared and mutable data/lamport borrows on both native source
  and destination and assert zero invocations plus unchanged actual state.
- Wrong/non-executable/aliased System Program is rejected before effects. The
  runtime-facing Rust function rejects on host without invoking or writing.
- At the one successful invocation, observe committed bookkeeping before payment;
  verify no outstanding state/lamport borrows prevent canonical invocation.
- CPI error, false success without payment, wrong amount/destination/source,
  post-CPI state tampering and postcheck failure reject and roll back the complete
  host transaction. No retries double-pay or double-reduce liabilities. The raw
  execution function's runtime rollback reliance remains explicit and tested in
  contrast with the transaction fixture where useful.
- Fresh authentication/postchecks reject malformed or altered state instead of
  trusting the earlier snapshot. Successful execution leaves original audit
  equations reconciled without rebasing, fabricated receipts or test-only repair.
- Existing 298-test baseline, doctest, accepted layouts/dependencies, old claim/
  custody/persistence tests and sensitive-action boundaries remain intact.

No initialization, entrypoint/ABI, Program ID, new keys, signatures, validator/SBF
setup, live CPI execution, deployment, real funds, authority transfer, new recipient
policy, heartbeat policy or unrelated task is authorized by this contract. Writing
runtime invocation code is distinct from executing a blockchain operation.

## Coordination and completion gate

Use existing sole writer `implement_t26_deposit` and separate reviewer
`review_t23_final`; pilot owns shared documentation and Git. Written scope review
precedes implementation. Writer owns the bounded claim execution module/tests,
narrow staged-plan accessor/error/export changes and its report append. No
competing builds, new dependencies, oracle changes or warning suppression.

Writer runs focused +1.97.1 locked/offline gates, records actual failures/corrections
and evidence limits, freezes exact source and releases the build slot. Pilot reads
all final code/tests and runs the established frozen workspace tests, doctest,
default/all-feature checks and warnings-denied docs. Final separate review covers
exact diff, tests, evidence and the production/host seam boundary. Commit, checkpoint
and publish reviewed integration only after passing. Technical validation is not
founder acceptance or a professional independent audit.


## Coordination checkpoint

The separate written-scope review required explicit mutable data/lamport borrow
preflight for both native accounts before bookkeeping persistence. The pilot
incorporated that concrete correction and its shared/mutable-borrow regressions.
The remaining scope passed review. `implement_t26_deposit` is reused as sole
writer, with `review_t23_final` reserved for separate final review. No implementation
or test result is claimed at dispatch. The writer owns the focused build slot
until source freeze; the pilot owns shared documentation and Git.

## Writer implementation evidence — 2026-09-09 UTC

Writer `implement_t26_deposit` verified user `jerem`, branch
`integration/piv1-testnet`, HEAD `44d70ec911ad3a78738fb90a04ac82eec3ca44f2`, and only
expected pilot-owned documentation changes before editing. The writer read current
AGENTS/checkpoint, the approved contract, canonical K-012/master section 11.5 and
accepted authentication, claim, persistence and fixture source. The API/CEI/host
seam checkpoint was sent to the pilot before substantial edits and approved.
Written-scope and preliminary source review are distinct from final validation,
separate implementation review and founder acceptance.

Writer-owned changes:

- `programs/piv1/src/kif_claim_execution.rs` (new): fixed execution composition,
  narrow execution errors and explicit host-only invocation seam.
- `programs/piv1/src/state/kif_claim.rs`: one crate-private read-only getter for
  already-staged Config, reward and predicted custody; no changed transition logic.
- `programs/piv1/src/lib.rs`: execution module export and accurate CPI description.
- `programs/piv1/tests/kif_claim_execution.rs` (new): 20 execution regressions and
  dedicated five-account transaction fixture.
- This report: append-only writer implementation/validation evidence.

Shared documents/Git remain pilot-owned. No existing authentication/persistence
function, pure validator/outcome, state layout, instruction marker, adapter,
World/claim fixture, audit equation, old test, manifest, lockfile or toolchain file
was changed by the writer. No new dependency or warning suppression was added.

### Execution composition

`KifClaimExecutionAccounts` contains the existing four-account claim bundle and
one canonical executable System Program account. `execute_kif_claim` receives
trusted executing Program ID/Rent and the accepted amount/cumulative-claimed
request. Its `KifClaimExecutionError` preserves the original `Piv1Error` or original
invocation `ProgramError`; `HostRuntimeUnavailable` is an explicit separate result.
This is an ordinary Rust library API, not an instruction ABI or entrypoint.

The private shared core rejects System identity/executable/key aliases, runs the
accepted isolated authentication and pure claim preparation, and encodes both
complete before/replacement state envelopes through Task 2.9. It also captures
Config/reward account lamports for exact preservation. It constructs the pinned
official System transfer and canonical signer seed group before bookkeeping.

Both native accounts' mutable data and lamport guards are held simultaneously in
a preflight scope, then all four are released. Preexisting shared/mutable native
borrow conflicts, including differently keyed synthetic shared backing, therefore
reject before bookkeeping. Task 2.9 acquires the two state data borrows atomically
and persists the complete checked Config/reward replacements before invocation.
All account guards have been released at invocation.

The sole invocation is the official System transfer to the authenticated guardian,
with exactly source, guardian destination and System Program AccountInfos. There
is exactly one signer group `[b"kif-sol", canonical_bump]`; PivAuthority is not used
as the System-owned source signer. No other state/custody account is exposed to
the invocation. The private core accepts no instruction or recipient supplied by
the caller.

After invocation success, the core freshly authenticates the actual four claim
accounts, compares actual custody with the prediction, compares both complete
state envelopes against their staged replacements, and checks unchanged state
account lamports. It then invokes the accepted plan's `commit` against original
owned Config/reward values using freshly observed custody, and checks the resulting
states against fresh authentication. Predicted custody is never substituted for
an observed receipt. Exact source/destination changes, full aggregate liability,
carry, rent and original excess are protected by the unchanged accepted plan.

### Runtime and host boundaries

The writer independently inspected locked local sources: Anchor 0.32.1 reexports
`solana-invoke` 0.4.0, whose checked `invoke_signed` preflights account borrows but
whose host syscall is `unimplemented!`. The ordinary execution API uses a
`cfg!(target_os = "solana")` guard to reject hosts before any account access or
mutation, while leaving its fixed official invocation wiring type-checked in
ordinary builds. No real signed invocation was executed by a host test. The
separate `solana-cpi` host behavior is not used as transfer evidence.

`execute_kif_claim_with_host_invoker` exists only under
`#[cfg(not(target_os = "solana"))]`. It passes a recording/emulation callback to the
same private execution core. There is no Solana-callable backend selector, no new
feature and no backend choice routed from future instruction data. The callback
records the real constructed instruction/account list/seeds and deliberately
models native effects; it does not emulate the SVM or authenticate signatures.

The raw execution function does not rewind persisted state or arbitrary invocation
effects after an error. Production must propagate every error to the eventual
transaction boundary. Tests explicitly show raw invocation failure/false success
can leave bookkeeping changed, and a post-transfer error can leave modeled native
effects as well. Retaining those raw effects after catching an error would be wrong.

The dedicated host transaction fixture clones the five-account model and original
claim audit, runs the shared execution, and commits that clone only on success.
After success it measures actual modeled source debit and destination credit,
requires equality, records the payment flow once and validates the unchanged
original audit equations. It never rebases the audit. Failure discards the complete
staged transaction; traces are returned separately from ledger state. This is
explicit modeled rollback, not evidence of actual Solana transaction rollback.

### Focused evidence

The 20 tests cover:

- Independently pinned official System transfer bytes (`2` as little-endian u32,
  followed by amount as little-endian u64), two exact writable metas, source signer
  requirement, fixed guardian destination, three invocation accounts and exactly
  one canonical seed group. The callback independently reconstructs the four
  accounting changes, observes those complete bytes before payment, and verifies
  mutable data/lamport borrows are available for native and state accounts.
- Partial followed by full historical inactive claims, pause availability, exact
  global-liability exhaustion leaving rent plus collective carry, unsolicited
  excess, and a later modeled credit without resetting the audit or enabling an
  old request. Modeled credit remains funding/ledger evidence, not proof of earning
  eligibility.
- Exactly four changed accounting fields, exact native payment, conserved totals,
  unchanged state rent/carry/excess/unrelated fields and unchanged System account.
  External HTFP/Team guardian-wallet overlap and System-owned PDA guardians remain
  supported. Synthetic extreme unrelated Config history/custody values demonstrate
  isolation only; no healthy distribution phase or global ledger proof is claimed.
- The ordinary runtime-facing host call returns `HostRuntimeUnavailable` even
  while accounts are mutably borrowed, preserving the entire fixture without
  reaching authentication, writes or the pinned host invocation.
- Shared/mutable data and lamport borrows on each native endpoint reject before
  bookkeeping with zero invocations. State data/lamport conflicts, including the
  second state account, preserve all prior bytes and native balances. Differently
  keyed native shared-data/shared-lamport backing rejects in simultaneous preflight.
- Wrong/non-executable/aliased System Program, all claim-role key aliases, static
  PIV-controlled destinations, wrong owners/privileges/keys/shapes/signature flags,
  canonical stored bumps, invalid state initialization/economics/envelopes/Option
  tags/zero tails, stale/zero/overclaims, complete backing deficits despite guardian
  surplus, arithmetic overflow, invalid trusted program/Rent and state/source rent.
  All composed precondition failures produce zero invocations and no earlier effect.
- Invocation errors before payment, after debit and after transfer preserve their
  original `ProgramError`. False success without payment, too-small/too-large
  amounts, wrong source/destination, malformed/tampered post-state and state-account
  lamport changes reject. Fresh authentication and exact staged-state checks detect
  these errors. Complete modeled rollback preserves the original fixture/audit,
  then retry succeeds once and replay fails without a second payment.

Accepted lower-level state/persistence tests remain unchanged and cover their
additional typed-envelope and allocation/serialization constraints. No allocator
exhaustion was injected, and no replacement serialization failure is fabricated:
the execution replacements come only from the accepted valid plan. Runtime
privileges, transaction rollback, SBF compute/heap behavior, actual System CPI,
initialization, caller ABI and deployed/Testnet behavior remain deferred.

### Writer commands and outcomes

1. `/home/jerem/.cargo/bin/cargo +1.97.1 test --package piv1
   --test kif_claim_execution --locked --offline` passed **16 tests**, zero
   failed/ignored, with one unused `PivConfig` import warning in the new test target.
   The writer removed that import and added four composed-boundary regressions;
   no warning suppression or production correction was used.
2. `/home/jerem/.cargo/bin/cargo +1.97.1 test --package piv1
   --test kif_claim_execution --test isolated_kif_claims --test state_persistence
   --test account_authentication --test guardian_clock_authentication
   --test vault_reconciliation --locked --offline` passed **132 tests**:
   20 execution, 24 isolated-claim, 21 persistence, 23 fixed-account,
   22 guardian/Clock and 22 vault-reconciliation. Zero failed/ignored and no warnings.

No command/build/test failed during the writer's Task 2.10 work. Read-only
verification used `whoami`, branch/HEAD/status/diff commands, `rg`, `cat` and `sed`.
`git diff --check` passed for the shared tracked diff. Rustfmt is unavailable;
no component was installed and no formatting pass is claimed. The pilot's full
workspace/doctest/check/documentation gates and separate final review are distinct
completion evidence to be recorded by the pilot.

### Frozen source and handoff

The writer froze source after the final 132-test run and released the sole build
slot before the pilot's gates. Exact inventory:
`/tmp/piv1-t210-writer-frozen-source.json`.

| File | SHA-256 |
|---|---|
| `programs/piv1/src/kif_claim_execution.rs` | `0be268fa1cf719b110b0c5e44fdd8c0022b315f2cbee9da1ead54d50be72b091` |
| `programs/piv1/src/state/kif_claim.rs` | `ddcc913ec8923b9f754930e0cfd39eda5363af469a00fc61ac50d1a5db85ce0d` |
| `programs/piv1/src/lib.rs` | `b4e6cab3db4079327c14d51a92ac4af77d3482ccec15fb0d68c1ead1db3802de` |
| `programs/piv1/tests/kif_claim_execution.rs` | `07ccea6f49c96e8e7c7be53ad219398c3dfe820934a35edb590092ad8178d0bb` |

At writer handoff, branch remains `integration/piv1-testnet`, HEAD remains
`44d70ec911ad3a78738fb90a04ac82eec3ca44f2`, and parent-owned documentation changes
are preserved. The writer made no Git mutation or commit. The new source/test and
report plus narrow accessor/export edits remain for pilot review/checkpoint.

No Mainnet action, deployment, live blockchain/CPI operation, real fund movement,
key creation, signing, secret access, validator/SBF setup or authority transfer
occurred. Writing runtime invocation code is separate from executing a blockchain
operation. This host evidence is not founder acceptance, runtime/Testnet validation
or a professional independent audit.


## Independent pilot validation — 2026-09-09 UTC

The pilot independently read the full execution module, the narrow staged-plan
accessor/export diff and all 20 test bodies, including the final trusted-context,
role-alias, simultaneous shared-native-backing and stored-bump regressions. It
verified the pinned Anchor/System invocation source and its host limitations.
Separate scope review added native mutable data/lamport preflight before effects;
that requirement and its regressions are present. No production defect or needed
functional correction was found in the pilot pass.

After source freeze and release of the single build slot, the pilot actually ran
the following gates with `/home/jerem/.cargo/bin/cargo +1.97.1`. These are distinct
from the writer's initial 16-test warning and final clean 132-test execution.

| Command | Actual pilot result |
|---|---|
| `test --workspace --all-targets --locked --offline --quiet` | **318 tests PASS**, zero failed/ignored |
| `test --workspace --doc --locked --offline` | **1 doctest PASS** |
| `check --workspace --all-targets --locked --offline` | PASS |
| `check --workspace --all-targets --all-features --locked --offline` | PASS |
| `RUSTDOCFLAGS='-D warnings' ... doc --workspace --no-deps --locked --offline` | PASS |
| `git diff --check` | PASS |

All gate logs are free of compiler warnings/errors. Every Rust/manifest input
hash and the input file set remained unchanged through the gates. Detailed
summaries/logs are in `/tmp/piv1-t210-pilot-20260909T111423Z`; the exact four-file
patch is `/tmp/piv1-t210-final-44d70ec.diff`, and parent inventory
`/tmp/piv1-t210-frozen-source.json` matches the writer inventory. Committed source
and this report are durable evidence; temporary artifacts may disappear.

Existing authentication/persistence, state validators and transition outcomes,
instruction markers, adapters, old tests/support/audit equations, manifests,
lockfile and toolchain remain unchanged. The only prior claim-plan edit exposes
its validated staging values internally. Targeted ownership/credential/generated-
path checks passed; all task files belong to `jerem`. Effective `core.hooksPath`
is unset, hooks are sample-only and no tracked `.github`/`.cargo` automation exists.
Before publication, independent remote reads still show accepted main
`66193769d1cbc59cd8630df295b9a784b9c64642`, integration closure
`44d70ec911ad3a78738fb90a04ac82eec3ca44f2` and Task 2.3
`3677fee97e3617ee65e2828d222008ba0952bb3e`.

The pilot maintains shared status/evidence in `AGENTS.md`, master specification,
execution plan, pilot checkpoint and test plan, plus completed Task 2.9 publication
evidence. Final separate review passed as recorded below; normal Git/publication
closure is in progress. No actual signed CPI, SBF build, entrypoint/ABI, new keys, signing,
deployment, Mainnet action, fund movement, authority transfer or unrelated secret
access occurred. Host invocation/rollback modeling is not SVM/Testnet proof or
founder acceptance.


## Final separate review and technical closure

Reviewer `review_t23_final` inspected the exact four-file diff against `44d70ec`,
all 20 tests, both complete evidence appends, actual hashes and the captured patch.
Verdict: **PASS within the bounded execution-library and host-model scope; no
actionable findings**. Both inventories and actual source agree. The reviewer
inspected the pilot results file and independently verified preserved baseline
code, but ran no builds. Writer and pilot executions remain separately attributed.

The review confirms native preflight, checks-effects-interactions, fixed System
instruction/accounts/KIF seeds, fresh custody/state-byte postconditions and state
rent preservation. Historical/paused claims, replay, false receipts, CPI failures,
tampering and modeled rollback retain the original audit. No production correction
was needed. This does not demonstrate SBF execution, actual signed CPI, a callable
instruction ABI or Solana rollback. Future runtime callers must propagate every
error instead of committing raw partial effects.

Changed files: the four source/test files listed above, this report, `AGENTS.md`,
`docs/PIV1_MASTER_SPEC.md`, `docs/PIV1_CODEX_EXECUTION_PLAN.md`,
`docs/PIV1_PILOT_STATE.md`, `docs/PIV1_TEST_PLAN.md` and Task 2.9 publication evidence.
Implementation commit: `c38a7b0d7122144bf3082cec7e57bbea7e61cc10`.
This documentation closure records its hash; normal reviewed integration
publication follows. Founder acceptance remains pending. No sensitive live
action or live-operation authorization occurred.
