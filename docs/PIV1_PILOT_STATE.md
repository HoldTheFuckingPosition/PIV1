# PIV1 technical pilot checkpoint

Execution: **ACTIVE under D-026** toward founder Testnet testing. Founder resumed
on 2026-09-09 at 18:33 UTC; earlier model/overnight pauses are historical.
Last verified: **2026-09-09 UTC**, user `jerem` uid 1001, `/home/jerem/piv1`.

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

- One worktree, branch `integration/piv1-testnet`.
- Last published closure: Task 2.13 `fd48c3b1644faed6d30fdb92774c1bd8c03a2658`;
  its implementation is `fd5735976eef1e2728ccf54726145501573db60d`.
- Task 2.14 implementation: `7442cab7e97c422c7ee06290d5fc9d11c8b13ee6`. Its 18 source/tool
  files exactly match the tested freeze. This checkpoint accompanies reviewed
  documentation closure; normal publication follows. Read actual local/remote HEAD.
- Accepted `main`, local and remote:
  `66193769d1cbc59cd8630df295b9a784b9c64642`; keep unchanged.
- Task 2.3 branch tip, local and remote:
  `3677fee97e3617ee65e2828d222008ba0952bb3e`.
- Remote: `github-piv1:HoldTheFuckingPosition/PIV1.git`. All three remote refs were
  independently checked before publication preparation.
- Native writer `implement_t26_deposit` released all source/report ownership;
  separate reviewer `review_t23_final` returned final technical/report PASS.
  Root owns closure/Git. No compiler/runtime remains running; reverify on takeover.
- Only expected task source/docs are changed. Two untracked tool bytecode files
  were preserved at `/tmp/piv1-t214-preserved-bytecode-20260909-a`; no source was
  discarded. No non-sample hooks, custom hooks path or `.github`/`.cargo` automation
  was found. Repeat targeted checks before publishing any newer change.

## Current task: 2.14 — runtime pending recognition

[Task 2.14 report](TASK_2_14_RUNTIME_PENDING_RECONCILIATION.md).
**TECHNICALLY VALIDATED / PENDING FOUNDER ACCEPTANCE**.
Four authenticated roles expose the existing phase-dependent pending transition.
Only Config's two pending fields change. Native/token balances, full round,
HWM, claims/carry and all other state remain protected. Recognition works during
pause and leaves native token-account excess untouched/unclassified. No transfer,
CPI, event, initialization, sweep or new economic rule is added by the instruction.

Separate scope, frozen production/host-source, corrected harness-source and
artifact/build-packet reviews passed. T214-R1 corrected an unexecuted fixture
assertion: old/new Rent structures differ while actual minimum balances agree.
Only the assertion/source pin changed; defaults, funding and production remained
unchanged. An earlier host run's fixture-import warnings were corrected with
narrow imports, without suppression. Preserve both preparation histories.

Evidence attribution:

- Writer: 71 focused host tests, zero diagnostics; root inspected actual logs.
- Root: **349 host tests +1 doctest**, eight gates, no diagnostics/failures/ignored
  tests, 16 checked log hashes and source preservation PASS.
- Root: new SBF build PASS, 304.424 seconds, 29 zero exits/58 log hashes, no stack
  diagnostics. Static direct frame references are within -4096 through -1;
  this is not a complete stack/heap proof.
- Root: native harness build PASS, 178.910 seconds, no diagnostics; exact ELF,
  interpreter and four resolved libraries inspected against unchanged pins.
- Root: **24 local SBF tests PASS /70 message cases**, including all 19 old claim
  tests, 3.198 seconds at 20:31:25–20:31:28 UTC. Runtime stage has 12 zero exits,
  24 checked log hashes and preservation PASS.
- Root and reviewer independently checked all **1629 complete account records**,
  accepted Borsh offsets, exact flows and unchanged unrelated state. Donation 100
  + recognition + repeated no-op uses 151832/200000 CU; paused token-native excess
  cases use 75844 CU. Shared success/failure shows raw first effects then Mollusk
  output discard, not Bank rollback. Claim reduced-compute failures remain
  pre-effects; ordinary claim uses 73837 CU.
- Root: seven stdlib runner tests PASS. Rustfmt/Clippy remain absent and unrun.

Frozen identities and evidence:

- Final 19-file freeze: `/tmp/piv1-t214-artifact-binding-20260909-a/preparation-freeze.json`,
  SHA `2225ec779d349c04f2272f0346deeacce77a6fe0dd15a4d7041e369864b80826`.
- Target ELF: `/tmp/piv1-keyless-sbf-build-t214-20260909-a/target/sbpf-solana-solana/release/piv1.so`,
  229208 bytes, SHA `46fd815847c236fb53ed5dc5ace79c48c5a21beac4019ffa107b80a5be69812f`.
- Native build: `/tmp/piv1-sbf-claims-build-t214-20260909-a`;
  executable `target/debug/deps/claims-c12d45bbd63e9bf4`, 96996056 bytes,
  SHA `587fc09cbdbdae17a236473f6a978ad4158d71cbc4dbf7c49ec302dc4d2c750a`.
- Run: `/tmp/piv1-sbf-claims-run-t214-20260909-a`; result SHA
  `d0b8854f68af9cc610b5c6970c8a4a9d7be593c6efedc2f7ab229475fbf5542e`.
- Host gates: `/tmp/piv1-t214-pilot-host-20260909-a`; independent account evidence:
  `/tmp/piv1-t214-pilot-account-observations-20260909-a.json`.

Final technical and five-document closure reviews passed without remaining
findings. The technical verdict reviewed report SHA
`62605db97ad00a4294313067546395d6e071f8e9cc6238d30f575110ca182f1e`
before its final status append. Source is committed; next: commit the reviewed
documentation closure, publish normally, verify
remote refs, then scope the next concrete dependency. Later tasks are not started.
Do not restart Task 2.3 or repeat unchanged dependency/Task 2.13 reviews.

## Preserved reviewed progression

Every row is **TECHNICALLY VALIDATED / PENDING FOUNDER ACCEPTANCE**. Reports retain
exact implementation identities, commands, attribution and limitations.

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
technical passes do not extend founder acceptance.

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
  Do not access unrelated secrets or modify accepted main.
- Normal integration fast-forward publication only; no force/history rewrite,
  automatic release/tag or unrelated publication. No sensitive operation occurred.
  AI-assisted review is not a professional independent audit.

Prior details remain in task reports, committed checkpoints and the
[historical checkpoint](history/PIV1_PILOT_STATE_PRE_T213_CLOSURE_20260909.md).
Temporary logs may disappear; code/documents are durable once committed.
Other chats and missing `HTFP_MASTER_CONTEXT.md` are not assumed accessible.
