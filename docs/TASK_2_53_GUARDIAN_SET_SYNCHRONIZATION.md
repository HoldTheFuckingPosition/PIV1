# Task 2.53 — Authenticated one-seat guardian synchronization

Status: **IMPLEMENTED / HOST AND STATIC SBF VALIDATED / PRODUCTION RUNTIME PENDING**.
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

**Current continuation — Task 2.53 one-seat guardian synchronization (2026-10-18 UTC).**
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
