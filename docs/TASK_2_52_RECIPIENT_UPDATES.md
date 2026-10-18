# Task 2.52 — Authenticated recipient updates

Status: **IMPLEMENTED / HOST AND STATIC SBF VALIDATED / PRODUCTION RUNTIME PENDING**.

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

## Actual root execution and artifact

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
- Actual runtime execution of this artifact: **not yet performed**.

Separate review binds final source, command design and observed root gate evidence.
Git and `publication.json` record reviewed integration publication identity;
`publication-evidence.json` freezes retained and new validation records. Later
review/publication receipts remain separately preserved.

## Remaining gate and next action

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
