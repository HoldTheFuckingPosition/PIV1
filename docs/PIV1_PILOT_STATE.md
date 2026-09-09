# PIV1 technical pilot checkpoint

Execution: **ACTIVE under D-026**. Founder resumed on 2026-09-09 at 18:33 UTC
with “si c'est bon gpt 6 on peut reprendre stp”. The earlier model/overnight pauses
are historical. Last verified: **2026-09-09 UTC**, user `jerem` uid 1001.

## Read on takeover

Verify actual user, branch, HEAD, worktree and agent state before acting.
Read `AGENTS.md`, this checkpoint and the current task report. Authority remains
`PIV1_DECISIONS.md` > `PIV1_MASTER_SPEC.md` > `PIV1_CODEX_EXECUTION_PLAN.md`,
with the newest explicit founder decision controlling its exact component.
The [technical pilot mandate](PIV1_TECHNICAL_PILOT_MANDATE.md), D-026, authorizes
successive bounded technical work, normal commits and reviewed development
publication. Technical validation is never founder acceptance.

Founder-facing reports are brief French; code, documentation and delegation are
English. The founder requested economical credit use: use exact changed inputs
and saved evidence, avoid repeated broad reads/reviews and unnecessary test runs.
Keep required safety/validation gates. No numerical credit budget was supplied.

## Git and ownership

- Repository: `/home/jerem/piv1`; one worktree.
- Development branch: `integration/piv1-testnet`.
- Task 2.13 implementation: `fd5735976eef1e2728ccf54726145501573db60d`.
  This checkpoint accompanies its reviewed documentation closure. Read actual
  local/remote HEAD on takeover; do not infer it from an implementation hash.
- Last remote read before Task 2.13 publication:
  `f9b462b3a1b5bf251ed9b196ee5ff5f5a99468b8` (Task 2.12 closure).
- Accepted `main`, local and remote:
  `66193769d1cbc59cd8630df295b9a784b9c64642`; keep it unchanged.
- Task 2.3 published branch tip:
  `3677fee97e3617ee65e2828d222008ba0952bb3e`.
- Existing remote: `github-piv1:HoldTheFuckingPosition/PIV1.git`.
- The exact eight Task 2.13 source/tool files are committed at the implementation
  above. Its report, shared status/checklist and historical checkpoint archive
  form the accompanying documentation closure. Preserve any newer work.
- Reuse native writer `implement_t26_deposit` and separate reviewer
  `review_t23_final`. Writer released source and final report ownership;
  source remains frozen. Root owns closure/shared documents and Git.
  No build or runtime process remains running. Reverify agent availability.

## Current task: 2.13 — final evidence review and publication

[Task 2.13 report](TASK_2_13_KEYLESS_SBF_CLAIM_EXECUTION.md).
**TECHNICALLY VALIDATED / PENDING FOUNDER ACCEPTANCE** after final separate review.
Implementation `fd5735976eef1e2728ccf54726145501573db60d` contains exactly the tested
eight-file freeze. Founder acceptance remains pending.

Root personally executed the third locked/offline, one-job host build and the
first run of its exact identified executable. Build completed at 19:19:18 UTC,
169.294 seconds, zero warnings/errors. Runtime ran 19:21:24–19:21:27 UTC:
**19 tests passed, none failed/ignored/filtered**, 60 message cases.
Both stages have 12 successful subprocess commands, 24 independently checked
log hashes and preservation PASS. The earlier two failed builds remain intact.

- Frozen eight-file candidate:
  `1048fa3376527d765390f8b9c6214fbef46062336aeb91a95a8c659c69baaa36`.
- Runner: `3d103a372d556e0090f78b4dae97e45fc6a4ec90694af990931d877822e37c8f`.
- Pins: `a0b388a11ae94704038b9ff177978979f1c7c219cb9ea8c60c8733e5ac5c0ad7`.
- Build: `/tmp/piv1-sbf-claims-build-20260909-c`; result
  `0ef283a217a21ba4d1699d7d20be8eb883e8304077bdc8ed5622f23b9617a505`.
- Host executable: `target/debug/deps/claims-c12d45bbd63e9bf4` below that build;
  96591016 bytes, SHA `c3a8cf5106109c3986e86cbd1b886442c2b4516698fb56814ee0ae0a915cd356`.
- Runtime: `/tmp/piv1-sbf-claims-run-20260909-a`; result
  `26d070f234678a40aecad3cd22e692d0a4ae2ddad4eaf24aa2e87d7491ac3a78`.
- Unchanged Task 2.12 PIV1 ELF:
  `/tmp/piv1-keyless-sbf-build-20260909-c/target/sbpf-solana-solana/release/piv1.so`,
  176064 bytes, SHA `0392bb822a3e767674ccd75486ad2685320bce5ffadb426ea8a93b08625bb6c8`.

Root and reviewer independently checked all 1366 complete account records:
exact four ledger changes, two native flows, full unrelated state and protection
of rent/carry/excess. Ordinary 100-lamport claim used 73834/200000 CU; paused
300-lamport historical claim used 73838. Real local SBF/System CPI and 80-byte
event evidence pass. Sequential 70/130/100 claims retain the original audit.
Shared success/stale/unreachable-third execution shows the first 100 payment in
both raw contexts, second-instruction error 6004, then the original returned
vector. This is Mollusk discard, not Bank/AccountsDB rollback. All four reduced
budgets (1/5000/10000/20000) fail with completely unchanged raw/returned accounts:
**no observed post-CEI compute failure**. Signatures/deployment remain unproven.

Resolved preparation findings: incompatible wincode branches (isolated address
2.6.1 / short-vec 3.2.2 pins); harness API paths/feature logging; T213-R1 precise
compute-exhaustion classification; T213-R2 complete account logs plus a positional
tail regression. No production correction or production dependency drift accompanies Task
2.13. The source/package/tool review is already complete; do not restart it.
Rustfmt/Clippy components are absent and were not run. Six unchanged stdlib
runner tests previously passed. Production's own final Task 2.12 evidence remains
339 host tests +1 doctest/eight gates; it was not rerun for this isolated harness.

Separate reviewer returned final bounded technical PASS without actionable
findings on report SHA `2dc9a96613cbc8cc2ef80c81ae2c4a48b5694985b80d5dac40346884c47fb183`
before the final status append. Root and reviewer inspected actual evidence
independently; all executions above belong to root.

Final shared-document review passed after two wording corrections. Targeted
checks passed for all 15 source/document candidates; hooks are sample-only,
`core.hooksPath` is unset and no repository `.github`/`.cargo` automation was found.
Current remote integration/accepted-main/Task 2.3 refs matched the known baseline.
Next: finish normal documentation closure/publication and independently verify
actual remote HEAD, then select the next bounded dependency. No Task 2.14 started.

## Preserved reviewed progression

All rows below are **TECHNICALLY VALIDATED / PENDING FOUNDER ACCEPTANCE**.
Implementation identities and complete evidence are in each task's report.

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

Task 2.3's original reported 164 tests are historical executor evidence. Root
also executed that baseline, reproduced severe-loss T23-R1, verified four failing
old/passing corrected regressions, and preserved HWM/recovery/custody invariants.
Do not restart or discard that correction. Accepted Phase 0/1 and Tasks 2.1/2.2
remain as recorded in canonical decisions; later technical passes do not extend
founder acceptance.

## Remaining delivery limits and decisions

- Follow [requirements-to-evidence checklist](PIV1_TEST_PLAN.md). Complete actual
  initialization and remaining contribution/distribution/activity/governance
  handlers, atomic runtime custody, failure/retry/pause behavior, real SPL/Jito
  adapter and founder test workflow before claiming complete Testnet readiness.
- **OPEN deposit liveness:** actual fees or any historical integer value loss
  reject. Some SOL remains queued; no implicit subsidy or HWM exception.
- Operational surplus lacks an authenticated funding baseline. Native excess in
  Token/temporary accounts and general Idle pending-SOL priority remain unresolved.
- Official pool/mint/list/source authentication, collision-safe snapshot identity,
  protected instruction mapping, real multi-leg source order/minima/fees/slippage,
  delayed readiness and both rent returns still require protocol/runtime proof.
- Current six reward records do not prove the global historical liability sum.
  Initialization/earning authority and heartbeat pause policy remain separate.
- Local SVM evidence is limited to already-earned claims, synthetic message
  privileges and its recorded feature configuration. No signatures, Bank rollback,
  deployment verifier, public-cluster behavior or complete heap bound is proved.
- **Live authorization: NONE.** No new wallet/key creation, signing, public-Testnet
  deployment/fund-moving lifecycle, Mainnet, real funds or authority transfer.
  Prepare the exact cluster/genesis/public identities/artifact/budget/operations
  card before the mandate's required live authorization; authority transfer has
  its own gate. No unrelated secrets access.
- Recheck Git hooks/CI and scoped secret/generated-file candidates before publish.
  Normal integration fast-forward only; no force/history rewrite/releases/tags,
  accepted-main modification or unrelated publication.
- No Mainnet action, deployment, real-fund movement, key creation, signing or
  authority transfer occurred in this sequence. AI review is not a professional audit.

Full prior checkpoint text is retained as a
[historical snapshot](history/PIV1_PILOT_STATE_PRE_T213_CLOSURE_20260909.md).
Task reports retain exact chronology, findings and evidence; temporary logs may
disappear, while committed code/documents are durable. Other chats and a missing
`HTFP_MASTER_CONTEXT.md` have not been assumed accessible.
