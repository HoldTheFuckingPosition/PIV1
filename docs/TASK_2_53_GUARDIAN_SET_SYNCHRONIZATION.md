# Task 2.53 — Authenticated one-seat guardian synchronization

Status: **TECHNICALLY VALIDATED FOR THE SUPPORTED LOCAL PROFILE**.
The implementation-stage narrative below is historical; the runtime checkpoint follows.
Starting integration: `32a5585c361e69915e60b96d6753e00659330f19`, clean as jerem.
Evidence: `/tmp/piv1-guardian-review-20261018-a`.
Main remains D-036 `705e151`; protected Task 2.3 remains `3677fee`.

## Requirements and supported choreography

K-001–K-007/K-010/K-012 and master §§11.1, 11.5, 13.11–13.12 require six
members, current 4/6 authority, public-key replacement rather than transferred
secrets, identity-bound activity and preserved already-earned rewards. This is
the required M2 guardian capability identified by the M4 closure analysis.
Accepted economics, persisted layouts and dependencies remain unchanged.

The bounded profile synchronizes one PIV guardian seat after a separately approved
Squads membership configuration change. A fresh exact current-member 4/6 action
must approve this PIV synchronization. The dedicated typed authorization boundary
requires the resulting six PIV keys to equal the current authenticated Squads set;
other five slots remain unchanged. Expected old revision and checked increment
bind the transition. No-op, duplicate and multiple-seat changes are unsupported.
Ordinary initialized authorization retains its correspondence guard.

Authority is bound to the current loader ProgramData and derived Squads vault.
Config stores no historical multisig identity. This action does not prove old-set
votes that caused the earlier external configuration change, live Squads behavior,
proposal consumption, stable historical multisig identity or exclusive control.
The mismatch interval is not an atomic cross-program rotation: ordinary heartbeat,
pause and recipient governance reject until correspondence is restored. Existing
frozen-round settlement and isolated earned claims retain their own tuple bindings.

## State, rewards and rent invariants

Six fresh revision-bound reward PDAs are required. The five unchanged identities
retain only their own activity marker; the replacement starts inactive. Every new
record has zero claimable, earned and claimed amounts. All six previous records
remain intact: retired keys can claim their earned balances, and an already
prepared round can still credit the original revision. No existing liability is
copied, redirected, forfeited or moved to carry. Future snapshots use new records.

Only Config's registry revision, registry membership/revision and six fresh reward
records are written. All economics, custody, global KIF liabilities, current-round
snapshots and prior reward records are preserved. A distinct approved signing
external System payer funds only exact rent shortfalls. Existing target prefunds
remain on their accounts; no PIV economic vault or operational reserve funds the
new records. Fixed System calls use canonical PDA seeds with fresh postconditions.
Failures propagate to the Solana transaction boundary; host callback tests alone
are not proof of runtime allocation rollback.

## Validation and remaining scope

Root owns actual locked/offline host gates and strict production SBF inspection;
one writer and a separate reviewer inspect the implementation. Actual results are recorded below with source freezes and preserved attempts.
A later separately reviewed local Bank profile must bind the fresh artifact and
execute synchronization, future activity, old-round settlement and retired claims.
Static compilation and host models do not establish those runtime outcomes.

Z remains the latest actual local execution on its original PIV/caller artifacts.
Historical lifecycle pins remain preserved; they must not relabel Z as proof for
this new production artifact. Current changed sources deliberately invalidate
those historical pins until the next reviewed runtime profile is prepared.

M4 remains incomplete; D-034 requires STOP before M5 after full M4 validation.
M5 remains unauthorized. The platform-blocked internal-failure operation stays
stopped/unwaived; neither drafts nor the underlying rejected operation are resumed
or rerouted. No live RPC/deployment, secrets, key creation/signing, real funds,
live authority transfer, cleanup or install occurs. Reviewed integration publication
is not founder acceptance or authority to update main. Preserve all prior evidence
and the 2-GiB reserve; save this bounded checkpoint before continuing.

## Actual implementation checkpoint and execution evidence

**Retained implementation — Task 2.53 one-seat guardian synchronization (2026-10-18 UTC).**
Clean takeover as jerem was integration `32a5585c361e69915e60b96d6753e00659330f19`.
Main remains D-036 `705e151`; protected Task 2.3 remains `3677fee`.
Status: **IMPLEMENTED / HOST AND STATIC SBF VALIDATED / PRODUCTION RUNTIME PENDING**.
One bounded writer and separate review added strict `PIV1UG01` (51 bytes/24 roles).
A fresh current 4/6 Squads action synchronizes exactly one changed PIV guardian
seat after an external membership change. Only its typed verifier permits this
checked mismatch; all ordinary correspondence guards remain intact. Authority
binds current ProgramData, not historical multisig continuity or old-set votes.

Config/registry revision advances once. Six fresh reward PDAs have zero monetary
history; only the five unchanged identities retain their own activity. Old records
remain untouched for retired claims and frozen-round credits. A distinct external
signer pays exact rent shortfalls; prefunds remain, with no PIV custody funding.
Current round, liabilities, HWM, economics, layouts and dependencies are preserved.

Root passed 13 focused tests, 619 host tests plus 1 doctest/eight gates
and strict production SBF/static inspection. Source review corrected an oversized
ProgramData copy, overly broad guardian alias restrictions and a draft test's
Program metadata padding before execution. First SBF attempt a failed strict stack checks (dispatch frame 5,504 > 4,096 bytes
and replacement-verifier call frames); its artifacts/logs remain preserved. The
corrected attempt b moves bounded state into heap-backed, non-inlined helpers;
no allocator, ABI, layout or economic guard is relaxed. Full b gates were rerun.
All 132 frozen production/host inputs passed postchecks. The new ELF has
not executed. Host evidence includes exact state/rent/CPI oracles, new activity,
retired-owner claims and frozen-revision settlement in liquid/withdrawn profiles;
it is not a chained Bank execution or proof of runtime allocation rollback.

Next execute and independently review one supported local production synchronization
with future activity, old-round settlement and retired claims, binding this new
artifact and exact local caller. Z remains the latest actual runtime on its prior
artifact; previous results remain retained evidence. Historical lifecycle pins
are preserved and reject changed production sources until a new reviewed profile.
Keep one-seat scope, retained old accounts/rent, synthetic governance and unsigned
transport limitations explicit. The platform-blocked internal-failure case remains
stopped/unwaived; its drafts and rejected operation were not inspected or rerouted.
M4 is incomplete. D-034 requires STOP before M5 after full validation; M5 remains
unauthorized. No founder acceptance or new main-publication authority is inferred.

Root owns reviewed integration-only publication; Git and
`/tmp/piv1-guardian-review-20261018-a/publication.json` record identity.
Baseline verified 393 tracked files, 279 historical pins, 130 prior production
inputs, five ELFs and 15,048 retained records. Preserve all evidence and the 2-GiB
reserve; final prepublication free space was 22,480,896,000 bytes. No cleanup/
install, secrets, keys/signing, live RPC/deployment, real funds or live authority
transfer occurred. Save/end this bounded implementation checkpoint.

Source review covers ten production/test/README files. Root commands use
`/tmp/piv1-guardian-review-20261018-a/validate.py` with `freeze b`,
`focused b`, `final b` and `sbf b`. Host tools are
pinned Rust 1.97.1; SBF uses pinned platform-tools 1.54. Cargo is locked/offline,
one job. Monitors deny new network sockets and enforce wall-time, memory and disk
limits; eight final gates cover tests/doctests/check/all-features/no-entrypoint/
cpi/idl-build/docs. Exact logs, tool hashes and immutable source copies are retained.

Fresh artifact: `/tmp/piv1-bank-smoke-guardians-sbf-20261018-b/artifacts/piv1.so`.
Bytes: 894,552; SHA-256:
`e29735e1467da98ecddb1eea9fd25abaffa4a450bd899ddc137f7d8c0da35b99`.
Source freeze: 132 inputs, SHA-256 `bdb94c0ce6e09face16274ab0723cb4010d2b33cd2c8da099e2718eae29ca878`.
This artifact has not been executed. Previous Z and caller evidence do not prove
this changed handler's production runtime. All earlier artifacts remain intact.

The fixed allocation sequence transfers only the shortfall, then allocates and
assigns each canonical PDA to PIV. Bounded state snapshots and exact writable
roles avoid copying executable/governance payloads into the 32-KiB production
heap. Read-only governance padding is covered by host tests; runtime resource
consumption and actual System atomicity remain next-profile obligations. No state
layout, dependency, economic or allocator change was needed. Old revision accounts
are intentionally retained; account closing and rent refunds are not implemented.

## Actual production runtime AA2

**Current continuation — Task 2.53 guardian synchronization runtime AA2 (2026-10-18 UTC).**
Clean takeover as jerem was integration `6e9c4eeff063a5dd63e726821774fb2a2fcfadb5`.
Main remains D-036 `705e151`; protected Task 2.3 remains `3677fee`.
Task 2.53 is **TECHNICALLY VALIDATED FOR THE SUPPORTED LOCAL PROFILE**.
One bounded writer and separate review extended the restricted synthetic caller
and compact local Bank scenario. All 132 production/host inputs, the current PIV
ELF, dependencies, layouts, allocator and accepted economics remain unchanged.
The previous 200,532-byte Rust lifecycle source remains an exact prefix.

Three actual liquid cycles establish old earned debt, synchronize one guardian
seat during the second prepared round, settle that old snapshot and exercise a
retired claim before the third cycle uses current-revision activity. Each cycle
credits 19,000,000 lamports to each of two active guardians. Synchronization
preserves the old records and frozen round, creates six zero-monetary revision
records and preserves activity only for unchanged identities. A post-snapshot
new guardian heartbeat earns nothing from the old round. The retired guardian
claims its actual 38,000,000 earned lamports; the unchanged old record retains
38,000,000 and the two new records earn 19,000,000 each. Final earned liability
is 76,000,000, cumulative credits 114,000,000, claimed 38,000,000 and carry zero.
Final Idle HWM/book value is 27,111,500,000; historical SOL 2,411,500,000 and tokens
19,000,000,000; pending is zero. Principal protection and original rents persist.

Actual donor prefunds total 2,366,471 lamports. The separate external signing
payer supplies exactly 6,486,720 rent lamports through 17 protected System CPIs.
All six new accounts retain their required rent and the first retains 71 excess
lamports. No economic custody pays this rent. Three-vote, executed and stale-tuple
attempts reject with fee-only maps. Selected late-outer failures roll back both
synchronization and retired claim; identical replays add no fee, distinct retries
succeed. Exact 24-role PIV CPI, System calls and event payloads are checked.

Root passed 15 caller tests/check/docs, strict caller SBF/static inspection,
66 runner tests, one locked/offline Bank build and actual AA2 execution. The 66
runner tests passed both before and after a preexecution prefund correction.
Draft corrections fixed the membership byte offset, frozen completion summary,
independent literal fixture and parser offset. Root and reviewer then identified
that a standalone one-lamport prefund would violate Bank rent-state rules; the
corrected System rent floor preserves a nonzero allocation shortfall. AA's source
freeze was never built or run; all drafts, freezes and receipts remain preserved.
The first independent v3 trace audit then rejected its stale zero-active event
expectation. Raw event bytes and the unchanged schema require bitmap three;
corrected v4 checks that exact value and all contribution/bootstrap events,
including both intake payloads in their actual order. Failed audit logs and all parser versions
remain preserved. No failed Bank execution or production defect is claimed.

Root and separate review audit 5,628 account records / 70 complete maps,
33 transaction/commit pairs (28 distinct plus five identical replays), 104 CPIs
and 24 actual event payloads. All 23 success maps and five fee-only failures
pass; both executions of the frozen independent parser agree. Synchronization
uses 388,632 CU; initializer 1,067,685 CU; maximum packet is
1,544 unsigned bytes. Prior 619 production host tests +1 doctest/eight gates,
strict production SBF and O–Z runtime results remain retained evidence, not rerun;
their distinct scopes are not automatically transferred to this new execution.

Next continue final independent requirement-to-evidence closure and address only
demonstrated critical-path blockers. M4 remains incomplete. The platform-blocked
internal-failure case stays stopped/unwaived; no drafts or rejected operation were
inspected, resumed or rerouted. D-034 requires STOP before M5 after full validation;
M5 remains unauthorized. Synthetic genesis/governance/membership/Clock, unsigned
transport, proposal non-consumption, one-seat scope and retained old-account rents
remain explicit. Current 4/6 approval is proved locally; historical old-set votes,
real Squads execution, historical multisig continuity and exclusive spending
control are not. AA2 executes no Stake instruction or governed recovery exit.
No founder acceptance or new main-publication authority is inferred.

Root owns reviewed integration-only publication; Git and
`/tmp/piv1-t253-runtime-review-20261018-a/publication.json` record identity.
The AA2 freeze binds 281 sources and 19,783 prior records; later receipts
remain separate. Preserve all evidence and the 2-GiB reserve. Runtime minimum
sampled free space was 21,868,261,376 bytes. No cleanup/install, secrets,
keys/signing, live RPC/deployment, real funds or live authority transfer occurred.
Save/end this bounded runtime checkpoint.

Actual commands: `run_runner_tests.py 2`, `validate_caller.py freeze a`,
`validate_caller.py focused a`, `validate_caller.py sbf a`, `prepare_profile.py aa2`,
`run_bank.py aa2 build`, `run_bank.py aa2 run`, `run_independent_audit_aa2_v4.py` and
`audit_execution_aa2.py`, under the evidence root above. Host Rust 1.97.1, SBF
platform-tools 1.54, locked/offline dependencies and bounded monitors remain pinned.

PIV artifact SHA-256: `e29735e1467da98ecddb1eea9fd25abaffa4a450bd899ddc137f7d8c0da35b99` (894,552 bytes).
New restricted caller SHA-256: `073a31d24c7b6b3cd1ba0b72ae677afb541d9287b80529167f0b173c14f3d4bc` (63,424 bytes).
Independent parser v4 SHA-256: `14eecbe6bf372fdd92fea698fba64721e75f912c40b3dc9654378ce88c7fc334`. Earlier parser copies and the failed v3 audit are preserved separately;
v1/v2 were not executed against the runtime. Exact source freezes, raw messages,
account bytes, logs, monitors and independent reviews remain in the evidence root.
