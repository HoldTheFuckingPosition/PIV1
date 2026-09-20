# PIV1 technical pilot checkpoint

## Active checkpoint — Task 2.26 (2026-09-20 UTC)

Task 2.26 is **TECHNICALLY VALIDATED / PENDING FOUNDER ACCEPTANCE**. The founder
resumed one economical bounded step. Root verified `jerem` (uid 1001), one clean
integration worktree and local/remote HEAD at baseline
`3b45241aec5e0b6a9405f5df2516da66c42c6485`. Main remains
`7b74be4b13c019b96a0c8abcbebfbcc361d31089`; Task 2.3 remains
`3677fee97e3617ee65e2828d222008ba0952bb3e`. Task 2.25 publication is complete;
D-028 does not extend main authority to these later tasks.

[Task 2.26](TASK_2_26_RECIPIENT_CHECKED_GENESIS_INITIALIZATION.md) composes recipient
identity checks with normalized same-call genesis initialization. The fixed new
profile uses the allocator's single fresh full preflight and captured Rent,
checks both recipients before effects, then preserves exact identity, balances,
owner/data and privilege flags after every successful System/Token CPI and final
completion. Its selection cannot substitute another genesis role mapping. No
public effects path accepts detached evidence. Invocation errors propagate before
postchecks; production does not undo partial host effects. Original external rent
obligations, normalization, zero initial ledgers and later pending recognition are
preserved. Existing API profiles, model/schema/native dispatch and economics stay
unchanged; added library-only error variants require exhaustive matches to adapt.

One delegated writer completed **22 initialization +8 recipient-preflight tests
PASS**, including six new groups and 24 success worlds. Separate source/test
review passed. Root independently executed **468 host tests +1 doctest/eight gates
PASS**. Both executions passed first attempt with no failures, ignored tests,
diagnostics or corrective retry. All 92 inputs match inspection and both
executions; root verified sixteen gate logs, two writer logs and three tools.
Root evidence: `/tmp/piv1-t226-pilot-host-20260920-a/pilot-summary.json`;
writer: `/tmp/piv1-t226-writer-20260920-e5n764bq`. Six old transport-template
inputs are unchanged: nine Node tests/eight cases remain retained evidence of
the earlier exact 33/32-account template, not rerun. The new complete fixture has
35/34 accounts and no transport/runtime execution proof. Task 2.14 SBF evidence
remains historical; no current SBF build was run.

Root owns final documentation review and normal integration-only publication.
Git records the exact task commit/publication identity; verify actual refs and
clean worktree on takeover. Save and **STOP; Task 2.27 is NOT STARTED**. No later
implementation or build is running. Next scope the combined transport and
remaining initialization readiness prerequisites: funding provenance, later
Token-native donations, full recipient control, spending-limit/stale-action/
live-artifact checks and current runtime/resource/rollback evidence. Native
initializer exposure remains deferred. No credit balance is inferred. No Mainnet
action, deployment, fund movement, key creation/signing or authority transfer
occurred. Preserve the mandate's exact live-operation gates.

## Previous checkpoint — Task 2.25 (2026-09-20 UTC)

Task 2.25 is **TECHNICALLY VALIDATED / PENDING FOUNDER ACCEPTANCE**. The founder
resumed one economical bounded step. Root verified `jerem` (uid 1001), one clean
integration worktree, HEAD and remote integration at baseline
`ceeac8618a66f662e5d6a0369ada76b080ebee84`. Main remains
`7b74be4b13c019b96a0c8abcbebfbcc361d31089`; Task 2.3 remains
`3677fee97e3617ee65e2828d222008ba0952bb3e`. Task 2.24 publication is complete;
D-028 does not extend main authority to Tasks 2.24–2.25.

[Task 2.25](TASK_2_25_GENESIS_RECIPIENT_PREFLIGHT.md) freshly composes full genesis
preflight with both approved temporary recipients. Each must match a canonical
vault PDA of the same authenticated governance multisig, remain distinct from
other roles/backing, and be an empty nonexecutable System account with positive
lamports covering current rent. Vault indexes only witness the approved keys.
No account mutation or funding occurs; recipient signatures are not required.
Native dispatch, schema, dependencies, economics, existing APIs and the 313-byte
model are unchanged.
Present identity does not prove exclusive four-of-six spending, absence of Squads
spending limits or stale actions, or live artifact control. The 32/31-account
read-only fixture needs its own future composition/transport proof; old complete
initializer packet evidence covers only its exact 33/32-account template.

One delegated writer completed eight grouped tests with 72 positive worlds.
The initial focused run was **7/8, exit 101, one unused-result warning**. Root and
reviewer verified a test-only correction to the expected earlier `InvalidAddress`
rejection and the success assertion; one retry was **8/8 PASS**, no diagnostics.
Separate source/test review passed. Root independently executed **462 host tests
+1 doctest/eight gates PASS** on the corrected freeze, first root execution with
zero failures, ignored tests or diagnostics. All 92 inputs match inspection and
retry; root verified sixteen gate logs, all four writer logs and three tool hashes.
Root evidence: `/tmp/piv1-t225-pilot-host-20260920-a/pilot-summary.json`.
Writer evidence: `/tmp/piv1-t225-writer-20260920-oc626x43` and
`/tmp/piv1-t225-writer-retry-20260920-tujlllux`. Six old transport-template inputs
are unchanged: nine Node tests/eight packet cases remain retained, not rerun.
No current SBF/runtime proof was added; Task 2.14 remains historical evidence.

Root owns final documentation review and normal integration-only publication.
Git records the exact task commit/publication identity; verify actual refs and
clean worktree on takeover. Save and **STOP; Task 2.26 is NOT STARTED**. No later
implementation or build is running. Next scope remaining funding provenance,
post-initialized Token-native donations and complete recipient-control evidence
before native initializer exposure; actual transport lifecycle and current
runtime/resource/rollback proof remain necessary. No credit balance is inferred.
No Mainnet action, deployment, fund movement, key creation/signing or authority
transfer occurred. Preserve the mandate's exact live-operation gates.

## Previous checkpoint — Task 2.24 (2026-09-20 UTC)

Task 2.24 is **TECHNICALLY VALIDATED / PENDING FOUNDER ACCEPTANCE**. The founder
resumed one bounded development step. Root verified `jerem` (uid 1001), one clean
integration worktree, and local/remote main plus integration at baseline
`7b74be4b13c019b96a0c8abcbebfbcc361d31089`. D-028 publication is complete and
does not cover this new task. Main stays at that baseline; Task 2.3 stays at
`3677fee97e3617ee65e2828d222008ba0952bb3e`.

[Task 2.24](TASK_2_24_GENESIS_TOKEN_PREFUND_NORMALIZATION.md) adds a distinct
same-call initialization path that moves only native excess from still-empty,
System-owned Token target PDAs into PendingSol before ownership changes. All
original rent shortfalls and the external payer debit remain unchanged: these
contributions cannot fund missing PendingSol rent. Initial history and pending
ledgers stay zero; actual authenticated reconciliation subsequently recognizes
the entire pending balance once. Existing raw APIs, non-Token prefund treatment,
approved message format, native ABI, schema, dependencies and economics remain
unchanged. Initial pause applies only to fresh state; initialized paused Config
rejects before effects. This is no general pause exception.

One delegated writer completed implementation and **16 focused tests PASS**.
Separate source/test review passed. Root inspected the frozen source/tests and
independently executed **454 host tests +1 doctest/eight gates PASS**, with no
failures, ignored tests, diagnostics or corrective retry. All 90 source inputs
match inspection and writer execution; root verified 16 gate logs, two writer
logs and three pinned tool hashes. Root evidence:
`/tmp/piv1-t224-pilot-host-20260920-a/pilot-summary.json`; writer evidence:
`/tmp/piv1-t224-writer-20260920-i_4xbogh`. Six retained transport-template inputs
remain unchanged: **nine Node tests/eight cases are retained, not rerun**.
No SBF/runtime execution was refreshed; Task 2.14 remains historical evidence.

Root owns final documentation review and normal integration publication under
D-026. Git records this task's exact commit/publication identity; verify actual
refs and clean worktree on takeover. Save and **STOP; Task 2.25 is NOT STARTED**.
No later implementation or build is running. Next scope the remaining funding
provenance, recipient control and post-initialized Token-native donations before
native initializer exposure; actual transport lifecycle and current runtime,
resource and rollback evidence remain necessary. No numerical credit balance is
inferred. No Mainnet action, deployment, fund movement, key creation/signing or
authority transfer occurred. Preserve the mandate's exact live-operation gates.

## Previous checkpoint — Main integration published (2026-09-19 UTC)

The founder instructed: "If everything is good, publish to main on GitHub please"
(English translation). D-028 records **TECHNICALLY VALIDATED / FOUNDER-AUTHORIZED
MAIN INTEGRATION** of Tasks 2.20–2.23 at
`3282e1ebabcb0cd88491d48a391565b8b100afa7`, plus this reviewed authorization record.
This is explicit integration authority, not broader founder acceptance or Testnet
readiness. Root verified `jerem`, clean integration and the four-commit ancestry
from main `5a067ad34d2f2ab0a27f2c6bff312a8b1f772d24`.

Root verified 94 unchanged inputs (90 Rust, two CJS, two spike manifests), 20
retained logs, prior **448 host tests +1 doctest/eight gates** and **9 Node tests /
eight transport cases**. Separate combined source review and targeted hook/CI,
secret/generated-file checks passed. **No tests were rerun in this publication
turn.** Evidence: `/tmp/piv1-main-integration-2-20-2-23-20260919/evidence-check.json`.
Historical Task 2.14 SBF evidence still covers only its earlier artifact.

After final documentation review, root records one documentation-only commit,
fast-forwards main and atomically publishes main plus integration. Git records
that commit and actual publication identity; verify local/remote refs and clean
worktree on takeover. No follow-up task starts: **Task 2.24 is NOT STARTED; STOP
after publication**. No source or build work is running. Native initializer
exposure, economic prefund/funding and recipient constraints, actual transport
lifecycle and current runtime/resource/rollback proof remain deferred. No
Mainnet action, deployment, fund movement, key creation/signing or authority
transfer occurred or is authorized by this publication instruction.

The checkpoints below are HISTORICAL records of their original sessions; their
pending-acceptance and integration-only restrictions are superseded only to the
extent of D-028's explicit main integration authority.

## Previous checkpoint — Task 2.23 published (2026-09-19 UTC)

Task 2.23 is **TECHNICALLY VALIDATED / PENDING FOUNDER ACCEPTANCE**.
The founder requested one further bounded step after Task 2.22 publication.
Verified baseline: `jerem` (uid 1001), one clean worktree on
`integration/piv1-testnet`, local/remote `208b7fb4b7f597fe409429c3a7483b12f73a66af`.
Accepted main remains `5a067ad34d2f2ab0a27f2c6bff312a8b1f772d24` and Task 2.3
remains `3677fee97e3617ee65e2828d222008ba0952bb3e`. No numerical usage balance is
inferred. Save and STOP after reviewed integration publication; Task 2.24 is NOT
STARTED. No later implementation or build is running.

[Task 2.23](TASK_2_23_GENESIS_TRANSPORT.md) adds a host-only unsigned transport
harness for the exact synthetic Task 2.22 topology/model payload. With distinct
receivers, the legacy execute packet is 1,334 bytes against the pinned legacy/v0
1,232-byte limit; one synthetic outer ALT loading sixteen target PDAs reduces v0
execution to 874 bytes. Shared receivers reduce these to 1,301/841 bytes. Upload
through pinned Squads buffer create/extend/from-buffer packets fits, with exact
compact-message hash/length reconstruction. Both external payer and guardian
executor remain static signers with zero signature placeholders. The Squads
vault remains an inner PDA signer and outer nonsigner. No stored-message lookup,
Rust, dependency, schema, economics or native ABI change. This is wire evidence,
not actual Squads execution or native initializer exposure.

One delegated writer completed the harness, tests and report. Separate source/test
review passed. Root inspected source/tests and independently executed **9 Node
tests PASS** plus the deterministic eight-case report; its JSON hash matches the
writer's. Each first test run found the same test-oracle arithmetic error: the
from-buffer packet is 421 bytes, not 420. The independently verified correction
passed one retry each; original failed logs remain. Exact privilege checks were
also tightened before first execution. Final runs have no failures, skipped tests
or stderr diagnostics. Root evidence: `/tmp/piv1-t223-pilot-host-20260919-b`;
writer: `/tmp/piv1-t223-writer-retry-20260919-kpkz8j1o`.

Root verified the retained pinned Node binary, all 78 web3.js archive files against
the locked npm cache archive, and all 90 unchanged Task 2.22 Rust inputs before/
after validation. **448 host tests +1 doctest/eight Rust gates are retained Task
2.22 evidence, not rerun here.** Historical Task 2.14 SBF evidence still covers
only its earlier artifact. Root owns final documentation review/publication
checks. Git records the exact task commit/publication identity; verify actual
refs and clean worktree on takeover. Tasks 2.20–2.23 remain pending founder
acceptance on integration; main stays unchanged.

Next session: scope the remaining funding/prefund normalization and recipient
constraints before exposing a native initializer; then current-source runtime,
rollback and resource evidence. The synthetic ALT still needs a separately
verified creation/funding/authority/content/warm-up lifecycle; actual signatures,
Squads approval/buffer execution and external preparation rent/refunds are
unproven. Buffer rent closes to its creator under pinned Squads, not automatically
to the external payer; no PIV1 reimbursement rule is introduced. No RPC/chain
operation, Mainnet action, deployment, fund movement, key creation/signing or
authority transfer occurred. Preserve exact D-026 live-operation gates. Stop here.

## Previous checkpoint — Task 2.22 published (2026-09-19 UTC)

Task 2.22 is **TECHNICALLY VALIDATED / PENDING FOUNDER ACCEPTANCE**.
After Task 2.21 publication, the founder explicitly requested a little more
progress, superseding that task's planned STOP for one additional bounded task.
Verified clean integration baseline:
`cbe5611f625b3e1be4acf38f25cca7dc5f0defb9`, independently matched on the remote.
User remains `jerem` (uid 1001), one worktree in `/home/jerem/piv1`.
Accepted main is `5a067ad34d2f2ab0a27f2c6bff312a8b1f772d24`; Task 2.3 stays
`3677fee97e3617ee65e2828d222008ba0952bb3e`. No numerical usage balance/budget or
active goal is inferred. Save/STOP after reviewed publication; no later
implementation or build is running. Task 2.23 is NOT STARTED.

[Task 2.22](TASK_2_22_GENESIS_ACCOUNT_INITIALIZATION.md) completes fresh approved
allocation, both canonical legacy Token InitializeAccount3 CPIs and all nine
typed genesis state envelopes in one library call. Token roles/mint borrow checks
precede effects; the same fresh Rent/model is preserved; no detached receipt is
accepted. All nine state buffers validate and are borrowed before copying.
Exact full-target/payer/mint postconditions and final fixed authentication pass.
The function remains undispatched. Raw prefunds stay unclassified, including
Token-native excess that still blocks the separate economic accessor. No schema,
dependency, economic decision or native instruction ABI changed. Every error
must propagate; host staged discard is not actual transaction rollback proof.

Reused writer `genesis_writer` completed implementation/report and ten focused
tests. Separate reviewer `genesis_review` passed scope and frozen source/tests,
including preservation of the first Token vault during the second initialization.
Root inspected actual code/tests and executed **448 host tests +1 doctest /
eight locked/offline gates PASS**, zero failures, ignored tests or diagnostics.
All 90 inputs match the inspected/writer freeze and remained unchanged; sixteen
gate logs, both focused logs and three unchanged host tools were verified.
Root evidence: `/tmp/piv1-t222-pilot-host-20260919-a/pilot-summary.json`;
writer evidence: `/tmp/piv1-t222-writer-20260919-32mv1qub`. No corrective retry.
Historical Task 2.14 SBF evidence still covers only its earlier artifact.

Root owns final documentation/evidence review and targeted publication checks.
Git records the exact task commit and integration publication identity; verify
HEAD, remote refs and clean worktree on takeover. Tasks 2.20–2.22 are not founder-
accepted and main remains unchanged. No RPC/chain operation, Mainnet action,
deployment, fund movement, key creation/signing or authority transfer occurred.
D-026's exact sensitive-action gates remain unchanged.

Next session: assess remaining native-initializer readiness, especially Token-
native prefund normalization, operational funding provenance, recipient constraints
and the exact Squads transaction transport/account budget. Then scope the required
native boundary and current-source SBF/runtime rollback/resource evidence before
exposure. Completed library bytes are not a Testnet handover milestone. Stop here.

## Previous checkpoint — Task 2.21 published (2026-09-19 UTC)

Task 2.21 is **TECHNICALLY VALIDATED / PENDING FOUNDER ACCEPTANCE**.
The founder returned and requested bounded progress in this existing chat.
Verified baseline: `jerem` (uid 1001), one clean worktree on
`integration/piv1-testnet` at `748faf81e5bd8f22c05b7588d7d4bb6d14d42848`, with
matching remote integration. Accepted local/remote main remains
`5a067ad34d2f2ab0a27f2c6bff312a8b1f772d24`; Task 2.3 remains
`3677fee97e3617ee65e2828d222008ba0952bb3e`. No goal or numerical usage budget is
active. This checkpoint closes one economical task with reviewed publication;
no later implementation/build is running. Task 2.22 is NOT STARTED.

[Task 2.21](TASK_2_21_GENESIS_ACCOUNT_ALLOCATION.md) composes fresh Task 2.20
approval/preflight with a distinct signing System payer. It pays only checked
rent shortfalls for all sixteen targets while preserving the payer rent floor,
then allocates/assigns eleven data accounts through canonical PDA-signed System
CPI. Exact full-batch postchecks follow every call. Raw prefunds stay untouched
and unclassified; no economic ledger, schema, dependency or runtime ABI changes.
This allocation-only intermediate is absent from native dispatch. Both Token
accounts and all nine states MUST be initialized in the same successful outer
transaction; bare Token allocation permits takeover. Every execution error must
propagate. Host staged discard is not actual runtime rollback evidence.

Delegated writer `genesis_writer` completed implementation/report and 12 focused
tests. Separate reviewer `genesis_review` passed scope and frozen source/tests.
Root inspected actual source/tests and executed **438 host tests +1 doctest /
eight locked/offline gates PASS**, zero failures, ignored tests or diagnostics.
All 88 source inputs match the inspected/writer freeze and stayed unchanged;
sixteen gate logs, both focused logs and three host tools were verified.
Evidence: `/tmp/piv1-t221-pilot-host-20260919-a/pilot-summary.json` and the report;
writer evidence: `/tmp/piv1-t221-writer-20260919-_5e4tkwr`. No corrective retry.
Historical Task 2.14 SBF evidence still covers only its earlier artifact.

Root owns final documentation review/publication checks. Git records the exact
task commit and integration publication identity; verify HEAD, remote refs and
clean worktree on takeover. Main stays at the founder-accepted milestone plus
acceptance records; neither Task 2.20 nor 2.21 is founder-accepted. No RPC/chain
operation, Mainnet action, deployment, fund movement, key creation/signing or
authority transfer occurred. Preserve exact D-026 live-operation gates.

Next session: scope safe same-transaction Token initialization, all genesis state
serialization and fresh post-initialization validation. Resolve remaining
recipient/funding and transport/resource constraints before exposing a native
initializer. Operational funding baseline and prefund classification remain
separate; no new economic decision is inferred. Save and STOP after this task.

## Previous session checkpoint — STOP after Task 2.20

The founder requested one economical development session followed by a saved
checkpoint and STOP. Task 2.20 is **TECHNICALLY VALIDATED / PENDING FOUNDER
ACCEPTANCE**, within its read-only source/host scope. Implementation and validation
are complete; this checkpoint closes reviewed publication. No later implementation
or build is running. Task 2.21 is NOT STARTED; resume only when the founder returns.
Verified session baseline: user `jerem`, uid 1001, branch
`integration/piv1-testnet`, one clean worktree; local/remote main and integration
both `5a067ad34d2f2ab0a27f2c6bff312a8b1f772d24`. Task 2.3 remains `3677fee`.
The goal tool returns no goal; no numerical usage budget or balance is inferred.

Task 2.20 technical scope: fresh approved genesis/Jito identity composition from
the same exact approved bytes, account slice and runtime context; all sixteen
canonical writable currently unallocated System targets; checked rent floors, raw prefunding
observations and per-target/aggregate shortfalls. The output is read-only
point-in-time evidence, not a creation/funding capability. No CPI, account
creation, funding classification, recipient-control proof, new instruction ABI,
schema or dependency is included. Preserve all economic ledgers and inputs.

One writer `squads_prerequisite` completed the module, ten focused tests and
[task report](TASK_2_20_GENESIS_ACCOUNT_PREFLIGHT.md). Separate reviewer
`review_squads` passed the frozen source/tests; root inspected actual code/tests
and executed **426 host tests +1 doctest / eight locked/offline gates**, with no
failures, ignored tests or diagnostics. All 86 source inputs were preserved and
matched the inspected/writer freeze; sixteen gate logs, two focused logs and
three unchanged tool hashes were verified. Writer evidence is ten passing tests,
separate from root's workspace execution. Evidence and exact commands:
`/tmp/piv1-t220-pilot-host-20260914-a/pilot-summary.json` and the task report.
The old Task 2.14 artifact's 24 SBF tests/70 cases remain historical evidence.

Root owns shared status documents and final publication checks. The task commit
is recorded in Git history and published only to integration; on takeover verify
HEAD, clean worktree and remote refs. Accepted main remains `5a067ad34d2f2ab0a27f2c6bff312a8b1f772d24`.
No Mainnet action, deployment, fund movement, key creation/signing or authority
transfer occurred. No dependencies, serialized layout or runtime instruction ABI
changed. The existing exact live-operation approval boundaries remain in force.

Next session: define a bounded prefunding-safe atomic account-creation task,
including explicit funding provenance, recipient/transport constraints and
post-creation validation. Preflight observations do not authorize spending,
establish historical noninitialization or prove recipient control/operational
funding. No further task starts in this session.

## Accepted main milestone

Execution: **FOUNDER-ACCEPTED — Tasks 2.3–2.19 integrated into main**.
On 2026-09-14 UTC the founder explicitly accepted the milestone at
`d9f3371be6ecb586675e3b38edcc57bd6e9519f8` and authorized its integration into main
(D-027). Root published that exact commit by normal fast-forward and independently
verified remote main/integration at `d9f3371`. This acceptance-record update is
documentation only. At that acceptance checkpoint Task 2.20 was not started;
the new short session is recorded above. D-026 remains active without routine
approval gates.

Review baseline: `1bf07eae13d90744c9c18e7dc3f5543185bb6284`, the published receipt
after Task 2.19 closure `e9c1b991fbd5066630612848c33ba5fbdb21776a`.
Root independently verified remote integration/main/Task 2.3 refs and a clean
starting worktree. Actual user `jerem`, uid 1001, one worktree in
`/home/jerem/piv1` on `integration/piv1-testnet`. The goal tool returned
**no goal**; historical `usageLimited` reports are not a current credit estimate.
No billing balance is inferred and no goal controls were changed.

## Read on takeover

Verify actual user, Git branch/HEAD/worktree and agents before acting. Preserve
unexpected work. Read `AGENTS.md`, this checkpoint and the current task report.
Authority: decisions > master specification > execution plan, with the newest
explicit founder decision controlling its exact component. The
[technical pilot mandate](PIV1_TECHNICAL_PILOT_MANDATE.md), D-026, authorizes bounded
successive technical work, normal commits and reviewed development publication.
Technical validation is never founder acceptance.

Founder reports are brief French; code/docs/delegation/commits are English.
The founder requested economical credit use; no numerical budget was supplied.
Reuse unchanged tool/package reviews and saved evidence. Run required validation
on changed inputs without redundant broad reads or unnecessary agent turns.

## Git and ownership

- Session baseline: clean local/remote integration
  `212e5a9fa4de2316e7abe2f5ff5cfe837c29b490`; independent remote reads matched all
  three refs. Task 2.18 implementation is
  `dc51450396a0e369e690d9038b7dde1a80e2ecd6`, published closure
  `f4bed3a44bbe4da4cf1e5304f04e2caeb85d83f1`. Normal atomic fast-forward push
  completed; independent integration/main/Task 2.3 remote reads matched and
  worktree was clean before this receipt/assessment checkpoint.
- Task 2.19 implementation: `b9f6f43d21713b3ec0bf81403378819f7cd3e44e`; published
  closure `e9c1b991fbd5066630612848c33ba5fbdb21776a`. Separate source and final
  documentation review passed. Normal atomic fast-forward push from `f4bed3a`
  completed; all three remote refs matched and the worktree was clean.
- New accepted implementation milestone:
  `d9f3371be6ecb586675e3b38edcc57bd6e9519f8`. Main was normally fast-forwarded
  from former accepted `66193769d1cbc59cd8630df295b9a784b9c64642` to that exact
  commit; an independent remote read verified main and integration at `d9f3371`.
  The subsequent reviewed acceptance record changes documentation only; normal
  publication maintains that record on both branches. Its identity is in Git history;
  keep main at the accepted milestone and these acceptance records. Later code
  continues on integration and requires its own eventual founder acceptance.
- Task 2.3 branch tip, local and remote:
  `3677fee97e3617ee65e2828d222008ba0952bb3e`.
- Remote: `github-piv1:HoldTheFuckingPosition/PIV1.git`. All three refs were
  independently verified after publication. Normal atomic fast-forward only.
- Writer `squads_prerequisite` owns seven acceptance/status documents; separate
  reviewer `review_squads` checks that delta and the root-owned instructions and
  checkpoint. Their earlier source-seam/recap review passed with no additional
  actionable foundation-level blocker. Root owns evidence verification and Git.
  Reuse these agents when available; no later implementation or build is running.
- Targeted secret/generated-file and hook/automation checks passed on the scoped
  change. No custom hooks path, non-sample hooks or `.github`/`.cargo` automation
  was found. These checks were repeated before main integration. The current change is
  documentation only; preserve any unexpected changes before publication.
- Prior preserved bytecode remains outside Git at
  `/tmp/piv1-t214-preserved-bytecode-20260909-a`; no source was discarded. Earlier
  publication receipts and evidence remain in their task reports/Git history.

## Founder acceptance and main publication

**CONFIRMED — D-027.** The founder's instruction, translated into English:
"I accept the 2.3–2.19 milestone at commit d9f3371 and authorize its integration
into main." The full accepted commit is recorded above; no additional source
change is included. This accepts the delivered bounded foundation, not complete
Testnet readiness or unsupported protocol/initialization/lifecycle behavior.

The accepted commit was pushed with `git push --atomic origin
d9f3371be6ecb586675e3b38edcc57bd6e9519f8:refs/heads/main`, followed by independent
`git ls-remote` checks. All 84 source hashes still match the reviewed 416-host-test
freeze. The acceptance delta touches only decisions, current technical statuses,
the two READMEs, recap notices, instructions and this checkpoint. Root verifies
the reviewed doc freeze, local links, whitespace, secret/generated-file scope,
hooks and final refs before closure; no new tests/builds are required for it.

The ordinary local tracking fetch could not write root-owned `.git/FETCH_HEAD`.
Root preserved that file and used `git fetch --no-write-fetch-head` instead;
no permissions or ownership were changed. Main publication had already succeeded;
this local metadata failure was not a test failure or rejected publication.

No PR was created. The earlier PR/API limitation no longer blocks this direct,
explicitly authorized Git integration. No Mainnet action, deployment, fund
movement, key creation/signing or authority transfer occurred. The exact live
approval boundaries remain unchanged.

## Historical recap review and PR preparation

The [integration review](PIV1_INTEGRATION_REVIEW_2_3_TO_2_19.md) maps delivered
layers, exact two-selector reachability and remaining dependencies. The
[prepared PR text](PIV1_INTEGRATION_PR_2_3_TO_2_19.md) targets `main` from
`integration/piv1-testnet`; it does not grant merge or founder acceptance.
At review time, source review found no foundation-level integration blocker.
IR-001 corrected
obsolete technical status in both READMEs and master progress summaries without
changing economics. Final separate source/documentation review passed.
Review scope, evidence and commands are recorded in the integration report;
normal publication changes seven documentation files on the integration branch.
The recap commit is recorded in Git history rather than duplicated inside its
own contents. On takeover verify HEAD, clean worktree and remote refs before acting.

Root verified unchanged evidence instead of rerunning the suite: 84 current
source/manifest/lock/toolchain hashes match Task 2.19's **416 host tests +1 doctest
/ eight gates**; all sixteen gate logs and three host tools match recorded hashes.
The Task 2.14 ELF still matches its 229,208 bytes and recorded SHA-256; its
**24 local SBF tests /70 cases** remain historical, not current-source evidence.
Receipt: `/tmp/piv1-integration-review-20260914-a/evidence-reuse.json`.
No build or test execution occurred during this documentation-only recap.

Git SSH publication works, but no authenticated GitHub API, callable GitHub app,
`gh`/`hub` binary or configured CLI authentication is available. Discovery checked
only tool availability/configuration presence; no credential value was inspected.
Automatic PR creation is unavailable. A public read-only GitHub API check found
no open PR for this exact head/base. Reviewed English title/body and the prefilled
creation link are complete in the integration report; **no PR was created**.
The earlier next action was to open the prepared PR; the founder's later D-027
acceptance and explicit main instruction supersede that workflow. The prepared
PR remains historical evidence and is not needed to complete this integration.
No Mainnet action, deployment, fund movement, key creation/signing or authority
transfer occurred during the recap review. Main was unchanged then; the newer
accepted publication is recorded above.

## Preserved Task 2.16 — bounded Squads invocation authorization

**COMPLETE / FOUNDER-ACCEPTED within the recorded scope (D-027)**.
Implementation: `343a496fb16edc8fd8d68746a89323d7545ab36d`.
[Task report](TASK_2_16_SQUADS_INVOCATION_AUTHORIZATION.md).
One writer completed the four-file source/test change and report. Separate scope,
source/test and writer-report review passed. Root inspected all production/tests/
fixtures, executed **383 host tests +1 doctest / eight gates**, and verified all
sixteen log hashes, source preservation, reviewed freeze and existing tool hashes.
Evidence: `/tmp/piv1-t216-pilot-host-20260914-a/pilot-summary.json`.
The writer's separate focused execution passed 20 tests; root verified its two logs.
No failure/ignored test or diagnostic occurred in either execution.

T216-R1 is resolved: the eligible signer derives from the authenticated outer
instruction, preserving executor flexibility. Three executors use identical
approved inner bytes without an inner executor account. The narrow profile binds
fresh same-program governance and Clock evidence to current nonstale four-voter
approval, exact direct single-inner message and current outer Squads execution.
No handler or state mutation exists. Injected context/statuses are modeled host
evidence; actual Squads CPI, persisted effect-once/rollback and resource limits
remain unproven. No SBF build/artifact refresh, initialization, rotation, schema,
error ABI, dependency, source pin or live operation changed.

Separate final shared-documentation/evidence review passed with no findings.
Targeted secret/generated-file and hook/automation checks passed. Normal atomic fast-forward publication completed at
`cadff2b0fbdbaee278beac4d44f2331b552e6acb`. Root independently read remote integration,
main and Task 2.3 refs; all matched local refs and worktree was clean. This following
checkpoint records that receipt. Accepted main was unchanged at that publication.

## Preserved Task 2.17 — separate bootstrap authorization

**COMPLETE / FOUNDER-ACCEPTED within the recorded scope (D-027)**.
Implementation: `69f7289e4c3ea2821141c4bcaded5ae942eed979`.
[Task report](TASK_2_17_SQUADS_BOOTSTRAP_AUTHORIZATION.md).
The single writer changed only the existing Squads module, nine new tests and
report. Separate source/test review passed with no findings. Root read the full
delta/tests, executed **392 host tests +1 doctest / eight gates**, verified sixteen
log hashes, full source preservation, reviewed freeze and unchanged tool hashes.
Evidence: `/tmp/piv1-t217-pilot-host-20260914-a/pilot-summary.json`.
Writer focused evidence is 29 passing tests (9 new +20 unchanged); root verified
both logs. No failed preparation, retry, failure, ignored test or diagnostic.

The new boundary requires a canonical writable, nonexecutable, System-owned empty
Config PDA and fresh exact Squads approval. Prefunding remains untouched. Existing
Task 2.16 guardian authentication and bounded parsers are preserved. Sorted-member
approval evidence has no PIV1 slot/revision/activity or parameter-semantics meaning.
Repeated virgin checks may succeed. No creation, handler/ABI/schema, new dependency,
transport, target build, source pin or live operation changed. Separate
documentation/evidence review passed with no findings. Normal atomic
publication completed through `09a02cbe485ab8abd7cb55f155539002df9251a8`; root
independently verified remote integration/main/Task2.3 refs and clean worktree.
This following receipt-only checkpoint records that completed publication.

## Preserved Task 2.18 — approved genesis model preparation

**COMPLETE / FOUNDER-ACCEPTED within the recorded scope (D-027)**.
Implementation: `dc51450396a0e369e690d9038b7dde1a80e2ecd6`.
[Task report](TASK_2_18_APPROVED_GENESIS_MODEL.md).
One writer completed five source/test files and the report. Separate scope and
final source/test/report review passed with no actionable findings. Root inspected
all changed source/tests, executed **404 host tests +1 doctest / eight gates**, and
verified sixteen final logs, both writer logs, the full 77-file writer freeze,
source preservation and unchanged previously verified host tools. No failed run,
retry, failure, ignored test or diagnostic occurred. Writer evidence: 41 focused
tests (12 new +29 preserved). Root evidence:
`/tmp/piv1-t218-pilot-host-20260914-a/pilot-summary.json`.

The strict 313-byte model-domain format binds approved slot/pause/anchor and
unverified protocol/recipient declarations to fresh Task 2.17 authorization using
identical bytes/accounts/runtime ID and one Clock. Derived proposed state has
immutable economics, initial slippage exactly 1 bps, zero histories/revision,
Idle distribution and six inactive zero-liability rewards. Sixteen descriptors
specify intended addresses/owners/sizes; expanded aliases preserve valid overlaps.

This remains a model, with no official protocol/recipient authentication, actual
non-Config target validity, rent/funding provenance, creation/persistence, handler,
new ABI/schema/dependency, target build or live action. KIF approvals do not prove
vote timing/activity; inactive model records do not select a vote-exclusion policy
or establish first-payout readiness. Prefunding remains untouched and unclassified.
Task 2.14 SBF artifacts are historical, not evidence for this newer source.

## Last completed technical task: 2.19 — source-pinned Jito account identity

**COMPLETE / FOUNDER-ACCEPTED within the recorded scope (D-027)**.
Implementation: `b9f6f43d21713b3ec0bf81403378819f7cd3e44e`.
[Task report](TASK_2_19_JITO_ACCOUNT_IDENTITY.md).
One writer completed the source, tests, extracted oracle/license, test-only
manifest/lock edges and report. Separate scope and final source/test/dependency/
report review passed with no actionable findings. Root inspected all source/tests,
verified exact oracle excerpts and source archives, checked the five lock edges
and all 168 unchanged package identities, then executed **416 host tests +1
doctest / eight gates**. Sixteen final log hashes, both writer logs, full source
preservation, the writer/inspected freeze and unchanged tool hashes were verified.
Evidence: `/tmp/piv1-t219-pilot-host-20260914-a/pilot-summary.json`.
Writer focused evidence: 12 passing tests, including 432 parser combinations.
No Cargo preparation/build/test failed; no failure, ignored test or diagnostic.

The seven-account boundary checks fixed official source identities and actual
program/pool/list/reserve/mint/manager/referrer relationships. Pool parsing is
bounded; list geometry permits residual slack without scanning entries. Canonical
withdrawal authority, default lockups, legacy mint and nonnative receiving accounts
are checked. Raw fees, optional authorities, preferences, epochs and supplies stay
separate from economic conversion and operational readiness. No genesis composition,
actual target creation, handler, ABI/schema, CPI, SBF or live operation is included.

Initial draft review removed the public test-only decoder and added the native
receiver rejection/regression. An unsupported Eq derive was removed before
compilation. The read-only source assessment's missing checksum-sidecar copy
interruption is retained in the report, separately from successful Cargo/tests.
The oracle is exact-source extracted types with actual pinned Borsh1/Stake types,
not complete SPL execution. Two pinned dev-dependencies use already locked
packages; all production dependencies and package versions/sources are preserved.
The report retains exact sources, checksums, license provenance and limitations.
Normal publication completed at `e9c1b991fbd5066630612848c33ba5fbdb21776a` after
separate final documentation review and targeted secret/generated-file/hook checks.
No sensitive operation occurred; accepted main was unchanged at that publication.

## Next dependency after Task 2.20

Task 2.20 now freshly composes approved genesis/protocol identity and authenticates
all sixteen currently unallocated targets, with checked rent-only shortfalls.
Actual prefunding-safe atomic creation, funding provenance and post-creation
validation remain next. Define a coherent bounded scope and obtain separate
technical review when the founder resumes; this is a pilot responsibility, not a
routine approval gate. Reuse prior identity, transport and state evidence.

No deployed binary/cluster identity, current pool/validator readiness, complete
liability history, actual initial vote activity or founder Testnet readiness is
established. No key/signing, live-operation or authority-transfer scope is released.
The reviewed test-only lock changes do not refresh historical SBF artifacts.

## Preserved Task 2.15

Implementation `da0241fd2a9f9c3247bbdeabb1a6b9c37dabc912`, published closure
`11f4d701f4ca58f18a64383dce5e088377afec77`, now founder-accepted within its
recorded scope under D-027. [Report](TASK_2_15_SQUADS_AUTHORITY_SNAPSHOT.md) preserves immutable
upstream sources, 14 tests, root 363 +1/eight gates, source preservation and final
separate review. Its one missing-import preparation failure occurred before tests;
failed logs and subsequent corrected final proof are retained. Current-state
configuration alone does not prove historical action approval. Existing Task 2.14
SBF artifacts remain historical. Remote refs matched and worktree was clean at
Task 2.15 publication before the Task 2.16 changes.

## Preserved Task 2.14

[Runtime pending recognition report](TASK_2_14_RUNTIME_PENDING_RECONCILIATION.md):
implementation `7442cab`, published closure `a189159`, now founder-accepted
within its recorded scope under D-027. Root executed 349 host tests +1 doctest/eight gates
and 24 local SBF tests/70 cases, with separate final review. Exact hashes, logs,
1629 account observations and limitations remain in that report and prior Git
checkpoints. These are historical executions, not a new Task 2.19 runtime claim.

## Preserved reviewed progression

Every row is now **COMPLETE / FOUNDER-ACCEPTED within its recorded scope**, under
D-027. Earlier task-report and publication statuses remain historical evidence;
their commands, exact implementation identities and limitations are unchanged.

| Task | Scope | Implementation | Root host tests + doctest |
| --- | --- | --- | --- |
| 2.3 | Atomic vault reconciliation and severe-loss correction T23-R1 | `0559ebd` | 168 +1 |
| 2.4 | Fixed-account authentication | `9f4f106` | 191 +1 |
| 2.5 | Initial contribution bootstrap | `9b997f3` | 209 +1 |
| 2.6 | Zero-fee, book-value-preserving principal SOL deposit | `9f75aec` | 231 +1 |
| 2.7 | Isolated earned KIF authentication/accounting | `10dceb5` | 255 +1 |
| 2.8 | Current guardian/Clock snapshot authentication | `c815474` | 277 +1 |
| 2.9 | Typed envelopes and existing-account persistence | `36152c1` | 298 +1 |
| 2.10 | KIF execution with explicit host invocation modeling | `c38a7b0` | 318 +1 |
| 2.11 | Claim ABI/runtime-ID/error/event boundary | `2eeefba` | 335 +1 |
| 2.12 | Keyless SBF compilation; reviewed stack correction | `cee6072` | 339 +1 |
| 2.13 | Keyless local SBF KIF execution | `fd57359` | Unchanged 339 +1; 19 local tests /60 cases |

Task 2.3's original reported 164 tests are historical executor evidence. Root also
executed that baseline, reproduced severe-loss T23-R1, verified four failing-old/
passing-corrected regressions and preserved HWM/recovery/custody invariants.
Accepted Phase 0/1 and Tasks 2.1/2.2 remain as recorded in canonical decisions;
D-027 adds explicit founder acceptance for Tasks 2.3–2.19. Technical passes alone
still cannot extend acceptance to later work.

## Remaining delivery limits and permissions

Follow the [requirements-to-evidence checklist](PIV1_TEST_PLAN.md). Complete actual
authorized initialization, remaining contribution/distribution/activity/governance
handlers, atomic runtime custody, failure/retry/pause behavior, real SPL/Jito
adapter and a usable founder workflow before claiming complete Testnet readiness.

- **OPEN:** deposit fees or historical integer value loss currently reject;
  some SOL remains queued. No implicit subsidy or HWM exception is selected.
- Operational surplus lacks authenticated funding provenance. Native excess in
  Token/temporary accounts and general Idle pending-SOL priority still need
  broader integration; Task 2.14 only preserves excess during pending recognition.
- Official pool/mint/list/source authentication, collision-safe snapshot identity,
  protected instruction mapping, real multi-leg ordering/minima/fees/slippage,
  delayed readiness and both rent returns still require protocol/runtime proof.
- Current guardian reward records do not prove the global historical liability
  sum. Initialization/earning authority and heartbeat pause policy remain separate.
- Active/settled/recovery pending offsets have host/pure evidence; new runtime
  fixtures cover Idle/pause with synthetic initial state/token units. No actual
  SPL transfer, signatures, Bank/AccountsDB rollback, deployment verifier,
  public-cluster behavior or total resource bound is established.
- **Live authorization: NONE.** No new wallet/key creation, signing, public-Testnet
  deployment/fund-moving lifecycle, Mainnet, real funds or authority transfer.
  Prepare the exact cluster/genesis/public identities/artifact/budget/operations
  card before the mandate's live gate. Authority transfer has its own gate.
  Do not access unrelated secrets or advance main beyond founder-accepted work
  and its reviewed acceptance records.
- D-027 authorizes this milestone's normal main fast-forward and acceptance
  record publication. Later bounded development publishes to integration; no force/history rewrite,
  automatic release/tag or unrelated publication. No sensitive operation occurred.
  AI-assisted review is not a professional independent audit.

Prior details remain in task reports, committed checkpoints and the
[historical checkpoint](history/PIV1_PILOT_STATE_PRE_T213_CLOSURE_20260909.md).
Temporary logs may disappear; code/documents are durable once committed.
Other chats and missing `HTFP_MASTER_CONTEXT.md` are not assumed accessible.
