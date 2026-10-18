# Task 2.52 — Authenticated recipient updates

Status: **TECHNICALLY VALIDATED FOR THE SUPPORTED LOCAL PROFILE / REVIEWED INTEGRATION**.

Starting integration: `be9640fc8c911a640e88adcd7fbc326804f30562`, clean as jerem.
Evidence: `/tmp/piv1-recipients-review-20261018-a`.
Main remains D-036 `705e151`; protected Task 2.3 remains `3677fee`.

## Requirement and implementation

This closes the implementation portion of required M2 recipient replacement,
identified by the [M4 closure analysis](PIV1_M4_CLOSURE_STATUS.md). K-002/G-007
and master §12 authorize exact current 4/6 governance replacement. Economics,
account layouts and dependencies are unchanged; guardian replacement is separate.

The native boundary dispatches `PIV1UR01`, version one, exactly twelve bytes.
The last three bytes witness the governance, HTFP and Team Owner vault indices.
Its fixed eighteen accounts are the pause profile's sixteen, then the two approved
recipient accounts. Full original bytes, keys and privileges are checked against
current Squads approval and current program authority/guardian correspondence.
Actual stack height, Clock and Rent are obtained by the runtime handler; ordinary
native host entry and the existing claim/initializer seams grant no authority.
Only the explicitly named host modeling seam accepts synthetic context.

The two recipients must be distinct canonical vault PDAs of that same authenticated
multisig: funded to current rent floor, non-executable, empty-data and System-owned.
Keys and shared data/lamport backing cannot alias another supplied account; current
guardian/PIV authority and non-recipient Config bindings are excluded. Existing
recipient identities can be retained or swapped. This supplies no arbitrary
program-owned recipient support and proves no exclusive 4/6 spending control.

After all checks, one typed atomic write changes exactly Config's two recipient
fields, then emits factual `RecipientsUpdated` with old/new keys and transaction
index. No custody account or active round is supplied; no CPI or economic/KIF/rent
mutation occurs. Pause does not forbid this authorized non-economic update.
Prepared rounds retain frozen destinations; later preparations use current Config.
Executed/stale approvals reject. Repeating unchanged synthetic Approved state can
succeed idempotently: PIV does not consume Squads proposals or create a replay receipt.

## Verification boundaries

Nine focused host tests check full account backing across active/Idle and pause,
exact Config-only replacement/event, repeated same-value updates, retained/swapped
keys, strict wire/count and runtime guards, rejected original-message/witness/key/
current-authority/Clock/proposal/account/alias/borrow/context variants. Every
rejection preserves all supplied state and emits no success event. An Idle model
preparation freezes the new destinations.

A separate regression exercises existing production settlement through its host
callback seam for liquid and withdrawn profiles. Changing current Config does not
redirect the frozen round: substituting the new keys rejects before CPI, while
old recipients receive the exact complete-map oracle payouts. This is complementary
host evidence, not one chained Bank execution of update/settlement/next preparation.

One bounded writer and separate source/command reviewer inspected eight files.
Root owns actual focused/full host and strict SBF execution and evidence review.
The code freeze has 130 inputs, including immutable copies; seven Rust files differ
from takeover. README is separately source-reviewed. The baseline checked 390
tracked inputs, 277 retained lifecycle pins, 128 prior production inputs, five ELFs
and 11,768 historical records. No historical artifact or failed attempt is replaced.

## Historical implementation gates and artifact

All first executions passed: nine focused tests, **605 host tests plus one doctest**,
and all eight host gates (all-target tests, doctests, check, all-features,
no-entrypoint, cpi, idl-build and docs). There were no ignored/failed tests or
warning/error diagnostics. Source review caught a duplicate draft event marker
and unused import before any build; README wording was clarified before final
review. No failed compiler/test attempt or demonstrated production defect is claimed.

Root commands, with exact pinned tools/environments recorded by the driver:

```text
python3 /tmp/piv1-recipients-review-20261018-a/validate.py freeze a
python3 /tmp/piv1-recipients-review-20261018-a/validate.py focused a
python3 /tmp/piv1-recipients-review-20261018-a/validate.py final a
python3 /tmp/piv1-recipients-review-20261018-a/validate.py sbf a
```

The host Cargo gates and production SBF compilation are locked/offline, one job.
The reviewed monitor denies new network sockets, caps wall time at 1,800 seconds,
retains a sampled 2-GiB disk reserve, a sampled 2.5-GiB group RSS/swap bound and
4-GiB per-process address-space limit. Strict SBF compilation plus ELF/static
inspection passed without stack or other diagnostics. All post-gate source/tool/
helper/profile and inherited-evidence checks passed.

Fresh production artifact: `/tmp/piv1-bank-smoke-recipients-sbf-20261018-a/artifacts/piv1.so`

- Bytes: 857,656.
- SHA-256: `d0dd2f9c21f02e9c0fa4aa3f2e4e8714f34a43b27cf474dbe8f50bb16bfd19c6`.
- Source freeze: `recipients-source-freeze-a.json`, 130 inputs, copied in `source-a`.
- At that implementation checkpoint, runtime was pending; Z below now executes it.

Separate review binds final source, command design and observed root gate evidence.
Git and `publication.json` record reviewed integration publication identity;
`publication-evidence.json` freezes retained and new validation records. Later
review/publication receipts remain separately preserved.

## Historical remaining gate after implementation

Execute and independently review the new handler in one local production lifecycle:
update during a prepared round, complete its settlement to old recipients, then
prepare a later round using the new Config destinations. Bind the new PIV artifact,
exact local caller extension, full state/custody maps, actual event and appropriate
ordinary rollback/replay/retry evidence. Do not treat host models or static build
as this runtime proof. Guardian replacement follows as a separate implementation.

Y remains the latest validated economic runtime on its original artifact. Historical
O–Y and caller results are retained evidence, not new runs or proof for this new ELF.
The unchanged `production_lifecycle_pins.json` deliberately binds the retained Y
source/artifacts and will fail source verification after this production change;
prepare a separately reviewed current profile before further execution rather than
silently relabeling old artifacts. The restricted current synthetic caller has no
recipient-update route yet. No general final-recipient/live Squads/signature/packet
transport or exclusive spending-control proof is inferred.

M4 remains incomplete. D-034 requires STOP before M5 after actual full closure;
M5 is unauthorized. The platform-blocked internal-failure case remains stopped and
unwaived; no draft or underlying rejected action was inspected/resumed/rerouted.
Final independent requirement-to-evidence closure and applicable final gates remain.
No cleanup/install, secrets, keys/signing, live RPC/deployment, real funds or authority
transfer occurred. Publication is reviewed integration only, not founder acceptance.
Preserve prior evidence and the 2-GiB reserve. Save this bounded checkpoint.

## Actual recipient-update runtime Z and current next action

**Current continuation — Task 2.52 recipient-update runtime Z (2026-10-18 UTC).**
Clean takeover as jerem was integration `64deb3cf004f58d9030b8f4335d41a5671ae107d`.
Main remains D-036 `705e151`; protected Task 2.3 remains `3677fee`.
Task 2.52 is **TECHNICALLY VALIDATED FOR THE SUPPORTED LOCAL PROFILE**.
One bounded writer and separate review extended the restricted synthetic caller
and local Bank scenario. Production's 130 inputs, dependencies, layouts and
accepted economics are unchanged; the previous 182,518-byte Rust test is an exact
prefix. Z first executes the current recipient-capable PIV ELF and new caller ELF.

During a prepared round, actual authorized `PIV1UR01` changes only the two future
Config recipient keys and emits its exact 208-byte event. Executed and three-vote
approvals reject 6102 after entering PIV. Supplying new keys to the old settlement
rejects 6000. The old round pays its frozen recipients; the next preparation and
settlement use the new pair. Two liquid cycles each pay HTFP 1,121,000,000 and
Team Owner 370,500,000 lamports. Final Idle HWM/book value is 24,788,500,000,
pending and earned KIF liability zero, collective carry 28,500,000. Historical
SOL is 1,988,500,000, tokens 19,000,000,000; zero-active compound totals 47,500,000.
Original rents, guardians and unrelated accounts remain exact.

Root's first executions pass 60 runner tests, 13 caller tests/check/docs, strict
caller SBF/static inspection, one locked/offline Bank build and actual runtime Z.
Root and separate review audit 3,444 account records / 44 complete maps,
22 transaction/commit pairs (17 distinct plus five identical replays),
67 CPIs and nine actual PIV event payloads. Twelve successful maps and five
fee-only failure maps pass; replays charge no new fee. Selected late-outer failures
roll back update/settlement, followed by distinct successful retries. Both executions
of the frozen independent parser agree. Prior 605 production host tests plus one
doctest/eight gates/strict SBF and O–Y remain retained evidence, not rerun.
Their distinct scopes are not automatically transferred to the new PIV artifact.

Next address required guardian replacement separately, preserving old earned
ownership and prepared activity snapshots, then independent closure/final gates.
M4 remains incomplete; M5 remains unauthorized and D-034 requires STOP before M5
once M4 is fully validated. Synthetic genesis/governance/Clock, unsigned transport,
proposal non-consumption, supported recipient/adapter and recovery-exit limits
persist. Z executes no Stake instruction or KIF claim and proves no exclusive
4/6 spending control. The platform-blocked internal-failure case stays stopped/
unwaived; no drafts or rejected operation were inspected, resumed or rerouted.

Root owns reviewed integration-only publication; Git and
`/tmp/piv1-t252-runtime-review-20261018-a/publication.json` record its identity.
Z froze 279 sources and 14,724 retained records; later receipts remain separate.
Runtime minimum sampled free space was 23,035,379,712 bytes. Preserve
all evidence and the 2-GiB reserve. No cleanup/install, secrets, keys/signing,
live RPC/deployment, real funds or live authority transfer occurred. No founder
acceptance or new main-publication authority is inferred. Save/end this session.

Exact execution evidence: `/tmp/piv1-bank-smoke-production-lifecycle-run-t252-z/runtime.stdout`
(166,002,218 bytes, SHA-256 `871395e9584c0db9eb9bacc970f5a62f4e21ac23687be9d7637bdff85df9d4ab`).
Bank binary SHA-256: `f754f609f6dc3f87675d23694ae55ffe55e12511109ac76b19c1fe46bba562df`.
Frozen independent parser SHA-256: `47c7aeead370af8ce28984118dcbfad06d40f11f129b30b8fb8de2cda9b9c2ed`.
Current production artifact remains `d0dd2f9c21f02e9c0fa4aa3f2e4e8714f34a43b27cf474dbe8f50bb16bfd19c6`;
new caller is 62,448 bytes, SHA-256
`7eb70e4a8795b085b79d83399f4b3cf5ba231bea18883749eecef1fe2007b8f2`.
Pinned Token/Pool/Stake binaries and all historical artifacts remain intact.
Updated lifecycle pins bind this new execution; the prior Y profile is preserved.

Actual initializer uses 1,067,452 CU; recipient update uses
185,814 CU. Maximum packet is 1,544
unsigned bytes; no signature verification or live packet transport is claimed.
The independent parser derives governance records, full maps, economic CPI and
actual event bytes. It binds retained O initializer layouts, while current ELF
accounts and all recipient/economic changes use current raw evidence. Compiled
inner records do not independently expose nested signer/writable flags; source
and caller boundary tests establish those restrictions.

Root executed the reviewed caller validator (`freeze a`, `focused a`, `sbf a`),
`run_runner_tests.py 1`, `prepare_profile.py z`, `run_bank.py z build`,
`run_bank.py z run` and `run_independent_audit.py` under the evidence directory.
Builds are locked/offline, one job, pinned tools; monitored builds/runs deny new
network sockets and preserve resource limits. Source/evidence postchecks passed.
No failed build/test/runtime or production defect was encountered. A draft caller
validator path was corrected during review before execution. Guardian replacement
is the next bounded required capability; this session stops at the reviewed
recipient checkpoint without starting that separate implementation.
