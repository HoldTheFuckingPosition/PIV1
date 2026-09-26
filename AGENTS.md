# PIV1 Repository Instructions

## Current execution state

Task 2.32 is TECHNICALLY VALIDATED / FOUNDER-AUTHORIZED MAIN INTEGRATION under
D-029 after the 2026-09-26 session. Starting integration was
`60193d63b64f42211f98d9b4910dfc047ab876df`; starting main was
`7b74be4b13c019b96a0c8abcbebfbcc361d31089`. D-029 covers the reviewed Tasks
2.24–2.32 sequence, without broader founder acceptance or live authority.
A delegated writer prepared an isolated harness; root passed 13 runner tests
and 13 runtime cases against the three unchanged Task 2.31 ELFs. Root and separate
review independently checked 2040 complete account records. Four initialization
successes use 1063693/1057103/984673/978083 CU with default 32-KiB heap and an
explicit 1.4m ceiling. A first build failed on obsolete test Rent fields; a first
runtime failed on test trace ordering. Minimal reviewed harness corrections
preserved all oracles; final build/run passed and both failures are retained.
Late failure proves initialized raw state then Mollusk output discard, not
Bank/AccountsDB rollback or in-initializer failed-CPI atomicity. Actual Squads,
full recipient control, funding provenance and live readiness remain unproved.
Production/probes/old harnesses/economics and 143 protected inputs are unchanged;
326 earlier logs are verified retained evidence, not reruns. Only new pins adopt
an independently verified installed libexpat hash; no installation occurred.
Final document and cumulative publication review passed. Root owns shared docs,
Git and normal main/integration publication. Git records the task commit/publication identity; verify actual
refs/worktree and read the checkpoint. Save/STOP; Task 2.33 is NOT STARTED.
All sensitive-action gates remain unchanged.

The Task 2.31 record below is HISTORICAL; publication is complete.

Task 2.31 is TECHNICALLY VALIDATED / PENDING FOUNDER ACCEPTANCE within its
build-only scope. Verified baseline integration is
`6238088f6dc8ef42a266b5041db9e0e50f262c85`; main remains
`7b74be4b13c019b96a0c8abcbebfbcc361d31089`. A delegated writer prepared three
isolated full-initialization validation artifacts: fixed 35/34-account synthetic
caller/callee and a canonical-ID, InitializeAccount3-only SPL Token 8 wrapper.
Root passed 15 Rust boundary tests and 11 mocked runner regressions, zero-example
doctest discovery and warning-denied docs. First strict SBF build passed with
34 commands/68 verified logs and no diagnostics; separate artifact review passed.
No new SBF runtime was executed. Existing production/probes/harnesses, economics,
128 protected inputs and six historical artifacts are unchanged; 250 earlier logs
remain verified retained evidence, not reruns. Only an in-memory new target profile
uses the previously verified libexpat hash; no install or old-pin change. Root owns
shared docs, Git and integration-only publication under D-026; Git records the
commit. Read the checkpoint and verify refs/worktree. Save/STOP; Task 2.32 is
NOT STARTED. Initializer runtime/resources/atomicity, actual Squads/control and
live readiness remain unproved; main acceptance and sensitive-action gates remain.

The Task 2.30 record below is HISTORICAL; publication is complete.

Task 2.30 is TECHNICALLY VALIDATED / PENDING FOUNDER ACCEPTANCE within its
keyless local preflight-runtime scope. Verified baseline integration is
`352fe7d4ecd8609d93cf2b0a2a96009018d3a7de`; main remains
`7b74be4b13c019b96a0c8abcbebfbcc361d31089`. A delegated writer prepared an
isolated harness; separate source/runner/build/binary/runtime review covers both
unchanged Task 2.29 probe ELFs. Root passed eleven runner regressions and twelve
runtime tests/cases, independently checking 1674 complete account records. An
initial host build failed on two missing-module imports in reused test support;
a minimal test-root re-export fixed them before the clean second build and first
runtime execution. Both 32/31-account profiles succeed with height-two CPI and
runtime Instructions/Clock/Rent; representative invalid cases reject as expected.
Success uses 288277/282070 CU under the explicit 1.4m ceiling and default 32KiB
heap, exceeding 200k; no ordinary-budget or full initializer-resource claim.
One verified installed libexpat hash changes only in the new profile; no install.
Production, probes, existing harness, economics and historical artifacts remain
unchanged. Earlier tests are verified retained evidence, not rerun. Root owns
shared docs, Git and integration-only publication under D-026; Git records the
commit. Read the checkpoint and verify refs/worktree. Save and STOP;
Task 2.31 is NOT STARTED. Main acceptance and live gates remain unchanged.

The Task 2.29 record below is HISTORICAL; publication is complete.

Task 2.29 is TECHNICALLY VALIDATED / PENDING FOUNDER ACCEPTANCE within its
build-only scope. Verified baseline integration is
`162f3b7b634633e2a5ab3011f0d746c4a4d15599`; main remains
`7b74be4b13c019b96a0c8abcbebfbcc361d31089`. Two isolated SBF probes prepare the
existing fixed 32/31-account recipient preflight through a synthetic caller.
A first host compilation failed in test integration; reviewed test-only fixes
then passed. Writer and root independently passed 10 Rust +9 runner tests; root
also passed zero-doctest discovery and warning-denied docs. The final strict
workspace SBF build and separate two-artifact static review passed after an initial
unused-result warning was resolved by explicit discard in the validation callee. No probe
runtime, actual Squads/control or initializer readiness claim follows. Production
sources, ABI, economics, existing harness and all historical artifacts remain
unchanged; earlier host/Node/runtime evidence is verified retained evidence, not
rerun. Root owns shared documents, Git and integration-only publication under
D-026; Git records the task commit. Read the checkpoint and verify refs/worktree.
Save and STOP; Task 2.30 is NOT STARTED. Main acceptance and live gates remain.

The Task 2.28 record below is HISTORICAL; publication is complete.

Task 2.28 is TECHNICALLY VALIDATED / PENDING FOUNDER ACCEPTANCE after recovery
on 2026-09-21 UTC. Verified baseline integration is
`560ca09c9c17becb79564c164e8c308b196c7cbe`; main remains
`7b74be4b13c019b96a0c8abcbebfbcc361d31089`. Two strict builds rejected oversized
genesis frames; private preflight boxes and separate construction helpers fixed
them without ABI, account bytes, dependency, validation-order or economic changes.
Final source passed 53 delegated focused tests, root 469 host tests +1 doctest/
eight gates and separate source/test review. The third strict SBF build, artifact
review, fresh harness build and exact-binary run passed: 24 local tests/70 cases,
1629 complete account records independently checked. Only dispatched claim/pending
paths gain runtime evidence; genesis total heap/resource/rollback proof remains
unproved. Sixteen target-runner/seven harness-runner regressions passed; unchanged
15 Node tests/eight old plus sixteen recipient cases are retained, not rerun.
Verified OS provenance justified narrow file-hash refreshes without installation.
Historical artifacts and both rejected builds are preserved. Root owns final
shared-document review, Git and integration-only publication under D-026; Git
records the commit. Read the checkpoint and verify actual refs/worktree. Save and
STOP; Task 2.29 is NOT STARTED. Main acceptance and live-operation gates remain.

The Task 2.27 record below is HISTORICAL; publication is complete.

Task 2.27 is TECHNICALLY VALIDATED / PENDING FOUNDER ACCEPTANCE after the
2026-09-20 bounded session. An explicit unsigned host transport profile covers
the complete Task 2.26 recipient-checked 35/34-account fixture while preserving
the old default CLI bytes. Delegated writer and root independently passed 15 Node
tests and eight old/sixteen new report cases on 96 matching inputs; first execution,
no failures/diagnostics. Separate source/test review passed after a pre-execution
test-oracle correction. All 92 Rust inputs and retained logs match 468 tests +1
doctest/eight gates from Task 2.26; Rust was not rerun. Native ABI, dependencies,
economics and runtime evidence are unchanged. Root publishes integration only
under D-026 after final document review; Git records the commit. Verified baseline
integration is `648998b4f5767eadf14c511d1dd0034ffff29ee0`; main remains
`7b74be4b13c019b96a0c8abcbebfbcc361d31089`. Read the active checkpoint and verify
refs/worktree. Save/STOP; Task 2.28 is NOT STARTED. Live-operation gates remain.

The Task 2.26 record below is HISTORICAL; its publication is complete.

Task 2.26 is TECHNICALLY VALIDATED / PENDING FOUNDER ACCEPTANCE after the
2026-09-20 bounded session. Its fixed normalized initialization profile checks
both recipients against the allocator's single fresh full preflight before any
effect and preserves their exact accounts after every successful System/Token
CPI and final completion. A delegated writer completed 30 focused tests; separate
source/test review passed. Root independently executed 468 host tests +1 doctest/
eight gates, zero failures or diagnostics. Both executions passed first attempt;
all 92 inputs and log/tool hashes match. Existing profiles, schema, model and
native dispatch remain unchanged; additive library error variants are documented.
Full recipient control, 35/34-account transport and current runtime proof remain
unproved, alongside funding provenance and later Token-native donation handling.
Root publishes integration only under D-026 after final documentation review;
Git records the task commit/publication identity. Baseline integration was
`3b45241aec5e0b6a9405f5df2516da66c42c6485`; main remains
`7b74be4b13c019b96a0c8abcbebfbcc361d31089`. Read the checkpoint and verify refs.
Save/STOP after this task; Task 2.27 is NOT STARTED. Live-operation gates remain.

The Task 2.25 record below is HISTORICAL; its publication is complete.

Task 2.25 is TECHNICALLY VALIDATED / PENDING FOUNDER ACCEPTANCE after the
2026-09-20 bounded session: a distinct read-only full genesis preflight verifies
the two approved recipient keys as funded System vault PDAs of the same current
governance multisig. One delegated writer completed eight focused tests after
one documented test-only correction; separate source/test review passed. Root
independently executed 462 host tests +1 doctest/eight gates, zero failures or
diagnostics, with 92 unchanged inputs. Existing APIs/economics remain unchanged.
Present recipient identity does not establish exclusive four-of-six spending,
absence of Squads spending limits/stale transactions or live artifact control.
The expanded preflight topology still needs future transport/runtime evidence.
Root publishes integration under D-026 after final documentation review; Git
records the task commit/publication identity. Verified baseline integration was
`ceeac8618a66f662e5d6a0369ada76b080ebee84`; main remains
`7b74be4b13c019b96a0c8abcbebfbcc361d31089`. Read the checkpoint and verify refs.
Save/STOP after this task; Task 2.26 is NOT STARTED. Live-operation gates remain.

The Task 2.24 completion record below is HISTORICAL; its publication is complete.

Task 2.24 is TECHNICALLY VALIDATED / PENDING FOUNDER ACCEPTANCE after the
2026-09-20 bounded session: same-call native prefund normalization from the two
unallocated Token PDAs into PendingSol preserves all original externally paid
rent obligations. A delegated writer completed implementation and 16 focused
tests; separate source/test review passed. Root independently executed 454 host
tests +1 doctest/eight gates, first pass, with 90 unchanged inputs and verified
logs/tools. Nine Node tests/eight transport cases remain retained evidence, not
rerun; current runtime proof remains deferred. Existing raw APIs and economic
decisions are preserved. Later Token-owned native donations remain unsupported.
Root publishes integration only under D-026 after final documentation review.
Main remains `7b74be4b13c019b96a0c8abcbebfbcc361d31089`; D-028 publication is
complete and does not cover Task 2.24. Git records the new task commit and
publication identity. Read the checkpoint and verify actual refs/worktree.
Task 2.25 is NOT STARTED; save and STOP after this bounded task.

The following D-028 publication record is HISTORICAL; actual Git refs were
reverified above on resumption.

On 2026-09-19 UTC, the founder conditionally authorized publication to main if
verification passed (D-028). Tasks 2.20–2.23 are TECHNICALLY VALIDATED /
FOUNDER-AUTHORIZED MAIN INTEGRATION for implementation
`3282e1ebabcb0cd88491d48a391565b8b100afa7`, plus its reviewed authorization record.
Root verified the four-commit ancestry from main
`5a067ad34d2f2ab0a27f2c6bff312a8b1f772d24`, all 94 inputs and 20 retained logs;
separate source review and publication safety checks passed. Prior 448 host tests
+1 doctest/eight gates and 9 Node tests/eight transport cases are retained evidence,
not rerun in this documentation-only publication turn. This authorizes main
integration, not broader founder acceptance, new economics or live operations.
Root publishes main and integration by normal fast-forward after final document
review; Git records the resulting documentation commit and publication identity.
Verify actual user, refs, HEAD and worktree on takeover. See the checkpoint and
D-028. Task 2.24 is NOT STARTED; save and STOP after publication. Native initializer
exposure, funding/prefund and recipient constraints, real transport lifecycle and
current runtime/resource/rollback proof remain deferred.

The founder explicitly accepted the Tasks 2.3–2.19 milestone at
`d9f3371be6ecb586675e3b38edcc57bd6e9519f8` and authorized its integration into
`main` on 2026-09-14 UTC, recorded as D-027. Root published that exact commit by
normal fast-forward from `66193769`; independent remote reads confirmed main and
integration at `d9f3371`, with the Task 2.3 branch unchanged. The historical D-027
documentation-only record updated acceptance status without extending the accepted
implementation or evidence. Verify actual current refs and worktree on takeover.

Tasks 2.3–2.19 are COMPLETE / FOUNDER-ACCEPTED within their documented bounded
scopes. The separate integration review passed; all 84 source inputs still match
the retained 416-host-test, one-doctest/eight-gate evidence. No tests were rerun
for acceptance documentation. The 24 SBF tests/70 cases remain historical Task
2.14 artifact evidence. No PR was created; its preparation is now historical.
At that acceptance, Task 2.20 had not started; later scopes remain in their reports.
Native initializer readiness and complete current runtime proof remain deferred.
D-028 now authorizes the specific later main integration recorded above; future
bounded development remains on integration under D-026 without routine approval.
Live-operation gates are unchanged.

The goal tool returns no goal; previous `usageLimited` reports are historical,
not a current credit estimate. D-026 remains active without routine approval gates.
Read `docs/PIV1_PILOT_STATE.md` and verify user/Git/agent state on takeover. Preserve
sensitive-action limits and economical usage; reuse unchanged reviews/evidence.

## Active technical pilot mandate

Read [docs/PIV1_PILOT_STATE.md](docs/PIV1_PILOT_STATE.md) on takeover and verify
the actual user, branch, HEAD and worktree before acting. The founder activated
[the technical pilot mandate](docs/PIV1_TECHNICAL_PILOT_MANDATE.md), recorded as
D-026, in this connected session. It supersedes earlier review-only and
per-task permission/stop requirements for bounded PIV1 technical work toward
founder Testnet testing. Use one delegated writer and a separate reviewer;
checkpoint each completed task before continuing. Technical validation is not
founder acceptance. Keep `main` within explicit founder acceptance or integration
authorization (D-027/D-028/D-029); use `integration/piv1-testnet` for the
reviewed development sequence. The mandate's economic-decision,
secrets, signing and exact public-Testnet approval gates remain mandatory.

## Authority order

Use the following sources in descending order of authority:

1. `docs/PIV1_DECISIONS.md`
2. `docs/PIV1_MASTER_SPEC.md`
3. `docs/PIV1_CODEX_EXECUTION_PLAN.md`
4. Explicit newer founder decisions recorded in the repository
5. Older chats, memory, articles, and drafts only as historical context

If sources conflict, expose the conflict and follow the newest explicit founder decision for that exact component.

Use these statuses: `CONFIRMED`, `PROVISIONAL`, `OPEN`, `HISTORICAL`, and `REJECTED`.

Never invent a missing decision, address, percentage, mechanism, authority, version, or requirement.

## Current confirmed foundations

- PIV1 has its own dedicated Solana Program ID, not yet created in Task 0.2.
- Contributions use SOL and JitoSOL.
- JitoSOL is the initial strategy.
- SOL enters through direct Jito stake-pool deposit to JitoSOL.
- JitoSOL exits through delayed direct withdrawal via a stake account to SOL.
- V1 has no Jupiter/DEX core path.
- Beneficiary outputs are native SOL.
- The yield split is fixed at `59% / 19.5% / 19.5% / 2%`.
- Governance uses six guardians with a 4-of-6 threshold.
- Full program upgrade authority is held under a 4-of-6 Squads vault.
- The program includes an explicit emergency pause.
- Operations are permissionless and the caller pays transaction fees.
- Successful distribution preparations must be at least ten days apart.
- Only one distribution may be active at a time.
- A valid attempt below the technical withdrawal minimum creates no snapshot.
- A valid insufficient attempt starts a 24-hour cooldown.
- Malformed failed transactions do not update that cooldown.
- An operational SOL rent reserve is excluded from principal and yield.
- Principal and the high-water mark are accounted in SOL lamports.
- Yield uses official Jito/SPL pool accounting, not a DEX price.
- Production direct deposits and withdrawals use only the slippage-protected SPL variants, with a 1-bps immutable hard cap.
- One active distribution may use multiple deterministic validator withdrawal legs; settlement waits for exact target assignment and complete leg finalization.
- Validator discovery and leg execution are permissionless; Jito API preference is operational guidance while current on-chain SPL source-order and safety checks are authoritative.
- Principal and pending JitoSOL use distinct PIV1-derived legacy Token accounts controlled by the shared PIV authority; neither vault is an ATA.
- Cooldown rewards become explicit next-cycle yield, recovered temporary-account rent returns to operations, and cooldown loss enters recovery without reducing the HWM.
- Arithmetic is checked and outgoing calculations use conservative floors.
- The high-water mark has no normal downward reset.
- Pending contributions remain separate from historical yield.
- Claimable KIF rewards are earned only for active periods.
- With zero active guardians, 50% of available KIF compounds and 50% carries forward.
- KIF periods are fixed 2,592,000-second half-open intervals derived from Solana Clock.
- KIF carry is reapplied in every successive zero-active period, and active-guardian division remainder remains collective KIF carry.
- `claim_kif` remains allowed during the global emergency pause, but only for an already-earned recorded guardian liability paid from the isolated `KifSolVault` under the confirmed guardian and destination constraints.

## Permanent safety restrictions

Codex must never:

- deploy to Mainnet without explicit approval at that exact step;
- move real funds;
- create, reveal, or store Mainnet private keys or seed phrases;
- store secrets in Git;
- invent wallet or recipient addresses;
- transfer upgrade authority without explicit approval at that exact step;
- silently alter confirmed economics;
- silently replace JitoSOL or the delayed direct-withdrawal strategy;
- introduce a Jupiter/DEX core path in V1;
- disable or bypass tests or security gates;
- treat Testnet or Devnet addresses as Mainnet addresses;
- perform irreversible VPS actions without explicit authorization and a recovery path;
- use unpinned dependencies without written justification;
- claim that an AI review is a professional independent audit.

Mainnet keys must never be stored on this VPS, even in ignored files.

## Working protocol

- Work on one bounded task at a time. Under D-026, continue to the next justified
  technical dependency only after review, applicable validation and a checkpoint.
- Read applicable repository instructions and authoritative documents before editing.
- Keep code, comments, documentation, public interfaces, commit messages, and technical names in English.
- Ask the founder only when a verified technical incompatibility or genuinely unresolved economic or security decision materially affects implementation.
- Never reopen confirmed decisions merely to discuss alternatives.
- Keep principal, yield, pending contributions, beneficiary allocations, KIF claims, operational rent, and external fees separately accounted.
- Use checked integer arithmetic and conservative rounding.
- Preserve unrelated files and existing work.
- Do not add dependencies unless the active task authorizes them.
- Do not store secrets, wallet files, credential-bearing URLs, or private configuration in the repository.
- End every task with files changed, commands, validation/tests, security observations, Git status, and commit hash.
- Explicitly state whether any Mainnet action, deployment, fund movement, key creation, or authority transfer occurred.
- Outside the active D-026 mandate, stop after the requested task. Within it,
  preserve the sensitive-action gates and checkpoint before any interruption.

Phase 0, Task 0.5, Tasks 1.1-1.4, and the complete Phase 1 specification-as-code foundation are COMPLETE / FOUNDER-ACCEPTED. The final accepted Task 1.3 implementation tip is `527e381661fe0cfc27e07ad9b44e1601a638ae75`; the accepted Task 1.4 implementation is `06c39429f3237f6974e21217670c3f0d30b0a571`. Task 2.1 is COMPLETE / FOUNDER-ACCEPTED at initial implementation commit `33b1e539f969432f82635d1ca76c59d89f0ec233` and final corrected tip `cb90d468eff4dce60552ba15b2b267b364a47827`. Task 2.2 is COMPLETE / FOUNDER-ACCEPTED at implementation commit `e3233b96b533a620e8037d5231baede10877217f`. Phase 2 is IN PROGRESS. Task 2.3 is COMPLETE / FOUNDER-ACCEPTED within its documented scope (D-027) after correction `0559ebdaaaf28c7e9b8f423eda158abe13093b8d` on `task/2.3-vault-reconciliation-model`; Task 2.4 fixed-account authentication is COMPLETE / FOUNDER-ACCEPTED within its documented scope (D-027) at `9f4f1064deeef78a3cbea2e9f84c560e87166f20` on `integration/piv1-testnet`; Task 2.5 initial contribution bootstrap is COMPLETE / FOUNDER-ACCEPTED within its documented scope (D-027) at `9b997f364d62b0796008b2f7fb3f905acf64a2e5`; Task 2.6 protected principal SOL deposit composition is COMPLETE / FOUNDER-ACCEPTED within its documented scope (D-027) at `9f75aec59d732b2662c1b2c7626f2a8f48887619`, limited to zero-fee conversions preserving historical book value and HWM coverage; Task 2.7 isolated KIF claim authentication/accounting is COMPLETE / FOUNDER-ACCEPTED within its documented scope (D-027) at `10dceb5b2eac691ff19840190e951bd2ec547984` after 255 host tests, one doctest, checks/docs and separate review passed; Task 2.8 current guardian/Clock snapshot authentication is COMPLETE / FOUNDER-ACCEPTED within its documented scope (D-027) at `c815474eea9a7854c3b495974d891f4dd1c67a27` after 277 host tests, one doctest, checks/docs and separate review passed; Task 2.9 typed state envelopes and atomic existing-account byte persistence is COMPLETE / FOUNDER-ACCEPTED within its documented scope (D-027) at `36152c157773737eca357e5dbfefd3f0900b6eb3` after 298 host tests, one doctest, checks/docs and separate review passed; Task 2.10 isolated KIF claim execution with explicit host invocation/rollback evidence is COMPLETE / FOUNDER-ACCEPTED within its documented scope (D-027) at `c38a7b0d7122144bf3082cec7e57bbea7e61cc10` after 318 host tests, one doctest, checks/docs and separate review passed; Task 2.11 claim instruction ABI/runtime-ID boundary is COMPLETE / FOUNDER-ACCEPTED within its documented scope (D-027) at `2eeefba0abc226bcfcadddb5f248f12ca589e09d` after 335 host tests, one doctest, checks/docs and separate review passed, within `docs/TASK_2_11_KIF_CLAIM_INSTRUCTION_BOUNDARY.md`; Task 2.12 keyless SBF compilation is COMPLETE / FOUNDER-ACCEPTED within its documented scope (D-027) at `cee6072ad7b3155d7d5b30e6c0830beb6d00d4eb` after a clean target build, 339 host tests, one doctest, checks/docs and separate source/artifact review passed, limited to the static evidence in `docs/TASK_2_12_KEYLESS_SBF_COMPILATION.md`; Task 2.13 keyless local SBF claim execution is COMPLETE / FOUNDER-ACCEPTED within its documented scope (D-027) after 19 local tests, 60 message cases, full account evidence and separate final review in `docs/TASK_2_13_KEYLESS_SBF_CLAIM_EXECUTION.md`; Task 2.14 runtime pending recognition is COMPLETE / FOUNDER-ACCEPTED within its documented scope (D-027) at `7442cab7e97c422c7ee06290d5fc9d11c8b13ee6` after 349 host tests, one doctest, eight gates, 24 local SBF tests and final separate review in `docs/TASK_2_14_RUNTIME_PENDING_RECONCILIATION.md`; Task 2.15 read-only Squads authority snapshot is COMPLETE / FOUNDER-ACCEPTED within its documented scope (D-027) at `da0241fd2a9f9c3247bbdeabb1a6b9c37dabc912` after 363 host tests, one doctest, eight gates and separate source/documentation review in `docs/TASK_2_15_SQUADS_AUTHORITY_SNAPSHOT.md`; Task 2.16 bounded direct Squads invocation authorization is COMPLETE / FOUNDER-ACCEPTED within its documented scope (D-027) at `343a496fb16edc8fd8d68746a89323d7545ab36d` after 383 host tests, one doctest, eight gates and separate source/test review in `docs/TASK_2_16_SQUADS_INVOCATION_AUTHORIZATION.md`; Task 2.17 separate bootstrap authorization is COMPLETE / FOUNDER-ACCEPTED within its documented scope (D-027) at `69f7289e4c3ea2821141c4bcaded5ae942eed979` after 392 host tests, one doctest, eight gates and separate source/test review in `docs/TASK_2_17_SQUADS_BOOTSTRAP_AUTHORIZATION.md`; Task 2.18 approved genesis model preparation is COMPLETE / FOUNDER-ACCEPTED within its documented scope (D-027) at `dc51450396a0e369e690d9038b7dde1a80e2ecd6` after 404 host tests, one doctest, eight gates and separate source/test/report review in `docs/TASK_2_18_APPROVED_GENESIS_MODEL.md`; Task 2.19 source-pinned Jito account identity authentication is COMPLETE / FOUNDER-ACCEPTED within its documented scope (D-027) at `b9f6f43d21713b3ec0bf81403378819f7cd3e44e` after 416 host tests, one doctest, eight gates and separate source/test/dependency/report review in `docs/TASK_2_19_JITO_ACCOUNT_IDENTITY.md`; Task 2.20 genesis account preflight is TECHNICALLY VALIDATED / FOUNDER-AUTHORIZED MAIN INTEGRATION (D-028) after 426 host tests, one doctest, eight gates and separate review within docs/TASK_2_20_GENESIS_ACCOUNT_PREFLIGHT.md; Task 2.21 genesis account allocation is TECHNICALLY VALIDATED / FOUNDER-AUTHORIZED MAIN INTEGRATION (D-028) after 438 host tests, one doctest, eight gates and separate source/test review within docs/TASK_2_21_GENESIS_ACCOUNT_ALLOCATION.md; Task 2.22 same-call genesis initialization is TECHNICALLY VALIDATED / FOUNDER-AUTHORIZED MAIN INTEGRATION (D-028) after 448 host tests, one doctest, eight gates and separate source/test review within docs/TASK_2_22_GENESIS_ACCOUNT_INITIALIZATION.md; Task 2.23 host genesis transport validation is TECHNICALLY VALIDATED / FOUNDER-AUTHORIZED MAIN INTEGRATION (D-028) after nine Node tests, eight deterministic packet cases and separate source/test review within docs/TASK_2_23_GENESIS_TRANSPORT.md; Task 2.24 same-call genesis Token-native prefund normalization is TECHNICALLY VALIDATED / PENDING FOUNDER ACCEPTANCE after 454 host tests, one doctest, eight gates and separate source/test review within docs/TASK_2_24_GENESIS_TOKEN_PREFUND_NORMALIZATION.md; Task 2.25 fresh genesis recipient-vault identity preflight is TECHNICALLY VALIDATED / PENDING FOUNDER ACCEPTANCE after 462 host tests, one doctest, eight gates and separate source/test review within docs/TASK_2_25_GENESIS_RECIPIENT_PREFLIGHT.md; Task 2.26 recipient-checked normalized genesis initialization is TECHNICALLY VALIDATED / PENDING FOUNDER ACCEPTANCE after 468 host tests, one doctest, eight gates and separate source/test review within docs/TASK_2_26_RECIPIENT_CHECKED_GENESIS_INITIALIZATION.md; Tasks 2.27 recipient-checked transport, 2.28 current SBF/runtime refresh, 2.29 isolated probe preparation, 2.30 keyless preflight runtime and 2.31 full-initialization probe preparation are TECHNICALLY VALIDATED / PENDING FOUNDER ACCEPTANCE within their task reports. Task 2.32 full-initialization runtime is TECHNICALLY VALIDATED after 13 runtime cases, 13 runner tests and independent review of 2040 account records. D-029 authorizes normal main integration of reviewed Tasks 2.24–2.32, without broader founder acceptance; Task 2.33 is NOT STARTED. Task 2.1 accepts the narrow interface and host-mock evidence, not exact SPL/Jito behavior; the collision-safe, account-derived production snapshot identity and real protocol mapping remain Phase-3-provisional. Task 2.2 accepts only pure pending-vault intake/reconciliation and host-mock evidence. At Task 2.2 acceptance, fixed-account and transfer validation, real custody, handlers, CPI, localnet behavior, all-vault normalization, and composition cases involving pending SOL moved into distribution escrow were deferred; explicit-transfer handler callability during pause remains PROVISIONAL. Task 2.3 adds pure phase-dependent custody derivations and atomic host composition for pending-to-escrow-to-HWM and economic-vault normalization; it does not authenticate real accounts or transfers. Operational surplus derivation remains unsupported without an authenticated funding baseline. See `docs/TASK_2_3_VAULT_RECONCILIATION_MODEL.md` for the supported scope and deferred cases. That review-only next-action restriction is HISTORICAL after D-026 activation. The current technical task, reviewed development progression and permission boundaries are recorded in docs/PIV1_PILOT_STATE.md; Tasks 2.3–2.19 were founder-accepted under D-027. This AI-assisted review is not a professional independent audit.
