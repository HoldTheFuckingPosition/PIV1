# Task 2.49 — Production atomic distribution settlement

Status: **TECHNICALLY VALIDATED / PENDING FOUNDER ACCEPTANCE** under D-026/D-030 M2.
Start integration: `a76d1bc106c92f9cf9947c107da7c7a501fe9878`; main: `1054ff3`.

## Accepted requirements

Decisions P-008/P-014/P-019/P-020/P-023/P-024, K-005–K-010, A-004/A-005,
G-004 and D-018; master specification 9.6; the accepted Task 2.3 custody model and
T23-R1 correction; existing `state::settle_distribution` remain authoritative.
Settlement requires exact target assignment and all successful legs finalized,
normalized escrow/custody and checked cumulative accounting. No partial payouts.
Derive the capped 5900/1950/200 allocation over 8050 from current authenticated
custody and frozen round facts. Exclude cooldown rewards, rent and pending value
from outgoing yield. Preserve earned liabilities and apply existing active/zero-
active KIF allocation to current net KIF plus frozen carry exactly once.

Protected value uses projected principal SOL, current official pool value of
remaining principal tokens and post-payment escrow, less new cooldown rewards
and pending SOL already used. T23-R1 maps only the final pending-use subtraction
to zero for valid recovery classification; it does not relax checked arithmetic.
An HWM shortfall commits only RecoveryRequired in the round, without speculative
CPI, beneficiary payment, Config update or guardian credit. Success couples exact
native movements with atomic typed Config/round/six reward state writes.

Authenticate snapshot guardian key/revision/index reward PDAs without requiring
current membership or activity. Six selected claim liabilities may be less than
global liabilities because other earned revisions survive; never equate them.
Frozen recipient addresses receive payment even after a live-config rotation.
Real recipient control is inherited configuration provenance, not newly proved.
All external effects require canonical System CPI/signers, exact receipts,
unchanged unrelated account records, and fresh pool/custody postconditions.

## Validation and scope

One writer prepares bounded production source, independent complete-account test
oracles and program documentation; a separate reviewer checks requirements,
source/tests/commands/evidence. Root owns shared guidance, source freezing,
locked/offline focused tests/eight final gates, strict SBF and normal integration-
only publication. Root executed the final gates recorded below. Host callbacks model effects and
transaction discard, never actual VM/Bank atomicity or signed live behavior.
No dependency, persisted layout or economic change occurred. Pending integration,
full local lifecycle, exact Testnet package and live approval remain separate.

Evidence root `/tmp/piv1-t249-pilot-review` retains 361 initial tracked-file hashes
and 525 protected records, including all Task 2.48 attempts/source/artifact. All
122 prior frozen inputs matched the clean takeover. Preserve four recovery
archives/restorations. No cleanup/install, secret access, signing, key creation,
deployment, live RPC, funds or authority transfer is authorized by this task.

## Reviewed execution design

The strict `PIV1SD01` version-1 selector has nine bytes and no caller amount or
recipient. Its 28/27 roles append Clock, frozen HTFP/Team recipients and six
snapshot reward accounts to the existing 19/18 bootstrap roles. Current registry
membership/activity is deliberately not an input to old snapshot earnings.
Recipient accounts must match the frozen keys and be empty, nonexecutable System
accounts. A positive credit must leave current rent coverage; an initially empty
balance is supported. A zero payment skips CPI and adds no new rent requirement.
This checks safe receipt, not present spending control or Squads configuration.

Derive projected movements using the existing allocation functions, then invoke
the accepted settlement transition once with the actual projected protected value.
Do not probe the transition with maximum protection: success-only cumulative-counter
overflow must not block a valid recovery-only result. Preserve the existing model's
reward-credit overflow precedence before recovery. A projected zero-active KIF
compound is counted through principal custody exactly once. Only the final pending-
used subtraction saturates for recovery classification; all custody arithmetic is
checked. Exact target/all-leg closure relies on authenticated cumulative state
established by prior production handlers; closed temporary legs are not enumerated.

## Focused evidence design and pre-execution review

Ten test groups include 20 liquid/withdrawn, shared/separate referrer and guardian-
eligibility combinations. Independent arithmetic covers the valid 1800-lamport
net boundary (1319 HTFP, 436 Team, 44 KIF and one dust), a zero-transfer boundary,
recipient rent/overflow, frozen configuration rotation and post-snapshot claims/
activity, six carry values with zero-active reapplication, severe T23-R1 recovery,
reward-overflow error precedence, and omitted pending/cooldown counterexamples.
Each of the three/four possible System calls has failure/noop/receipt-corruption
cases; all eight final state-borrow conflicts are exercised. Identity, alias,
custody deficit/surplus, selected/global liability and malformed ABI guards remain.

Before execution root and separate review strengthened the unfinished-leg test:
a valid WithdrawalActive header with no finalized sums and exact U+carry escrow
must reject with InvalidLifecycle before CPI. The old malformed-counter cases
remain. Matching Clock epoch and validated snapshot-slot regressions cover the
new monotonic checks. No production economics or existing state helper changed.

Fixture setup reuses the existing production preparation host boundary and older
independent initiation/finalization fixture oracles. The new settlement oracle
uses explicit integer allocation, exact literal System instructions/metas/seeds,
full account vectors and manually derived Config/round/reward state. It never
calls production settlement/proof/persistence to obtain the expected result.
Carry reapplication uses separate valid fixture rounds, not a demonstrated full
production cycle; post-settlement integration and actual runtime proof remain open.

## Final source, review and validation evidence

One writer completed the bounded implementation and independent custody/byte/state
oracle tests. Root and a separate reviewer inspected the source, tests and command
plan before execution. Both found one unused import, removed before the source
freeze. Root ran `validate.py freeze`, `focused`, `final`, `sbf` and the two
`check_evidence.py` modes. First executions passed: 10 focused tests, 581 host tests +1 doctest/eight gates and strict SBF.
The eight final gates include all-target tests, doctests, workspace check,
all-features, no-entrypoint, cpi, idl-build and warning-denied documentation.
No old VM/Bank suite was rerun. Test callbacks model System effects and explicit
failed-world discard; compilation is static evidence only.

Final freeze: 125 inputs, SHA-256 `07d05688a003e45bb560ccba2f593351076076a726d5a282dd2c3b0f8a40c9f5`.
ELF: `/tmp/piv1-bank-smoke-settlement-sbf-t249-a/artifacts/piv1.so` (809888 bytes),
SHA-256 `e35af306c28ac68e7d97b127149db3c05f1109e8d6b3ebab21171f07f990e3c9`.
Independent raw ELF inspection confirms ELF64 little endian, ET_DYN, machine 263,
flags zero and the same 15 imports as Task 2.48. All 18 host and 58 SBF logs match their
receipts. Build elapsed 302.275s; sampled peak group
RSS+swap 614166528 bytes; sampled minimum
free disk 48863297536 bytes. These observations do not
measure instruction heap/CU or prove transaction rollback.

Exact final source/evidence and the 15-file integration tree receive separate
review before normal publication. Git and `publication.json` record the commit,
parent, tree, actual remote refs and clean worktree. Main stays 1054ff3; protected
Task 2.3 stays 3677fee. Technical validation is not founder acceptance. Preserve
all 525 historical records, four recovery archives/restorations and this artifact.
No new dependency/layout/economic rule or sensitive operation occurred.

Next: post-settlement pending integration, then M4 complete real production-path
lifecycle, M5 exact Testnet package and M6 explicit founder deployment approval.
Stop this bounded session after the reviewed integration checkpoint to conserve
usage; resume from actual refs/worktree. Historical task limits remain documented.
