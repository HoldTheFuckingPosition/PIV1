# PIV1 technical pilot checkpoint

Execution: **ACTIVE — founder resumed on 2026-09-09 at 09:20 UTC.**
The overnight pause ended with the founder's explicit "on reprend". The pilot
verified user `jerem`, the single clean worktree, local/remote integration HEAD
`bff59bb53ebb56875a7b34ae055cc4aa8d031fb9`, unchanged accepted main and no running
subagents. No engineering work was performed during the pause. Resumption does
not grant founder acceptance or live-operation authorization.

Last verified: **2026-09-09 UTC**. On takeover, read this file and `AGENTS.md`,
then verify actual user, branch, HEAD, worktree and running agents. This is an
execution checkpoint, not a competing economic specification.

## Goal and mandate

Deliver reviewed PIV1 implementation, authorized public-Testnet end-to-end
evidence and a practical founder testing handover. D-026 activates the complete
[PIV1_TECHNICAL_PILOT_MANDATE.md](PIV1_TECHNICAL_PILOT_MANDATE.md): successive
bounded tasks, one delegated writer, separate review, normal commits and clean
reviewed development publication are authorized without routine permission.
Checkpoint each task before the next. Technical validation is not founder
acceptance. French brief founder reports; English code/docs/delegation/commits.

Authority: `PIV1_DECISIONS.md`, `PIV1_MASTER_SPEC.md`,
`PIV1_CODEX_EXECUTION_PLAN.md`, newer explicit component decisions. Economics,
guardian/upgrade custody, direct Jito strategy and protected accounting remain
unchanged. PIV1 is the first HTFP infrastructure component; other tokens, Team
Owner components and MTT are out of scope. `HTFP_MASTER_CONTEXT.md` was not found;
no other ChatGPT/browser-history access is assumed.

## Verified Git and acceptance

- User `jerem` (uid 1001), `/home/jerem/piv1`, one worktree.
- Remote: `github-piv1:HoldTheFuckingPosition/PIV1.git`. WeatherTrader2 is the
  connected project label, not another detected repository.
- Branch: `integration/piv1-testnet`. Latest implementation:
  **Task 2.9 `36152c157773737eca357e5dbfefd3f0900b6eb3`**. Reviewed closure
  **`44d70ec911ad3a78738fb90a04ac82eec3ca44f2`** is published and independently
  reread remotely from a clean worktree at publication. Inspect subsequent scoped
  changes on takeover.
  Task 2.8 implementation `c815474eea9a7854c3b495974d891f4dd1c67a27` and
  closure `440e83e26d36df911ccfafac97d89b79b8b4c694` are already published.
  Task 2.7 implementation `10dceb5b2eac691ff19840190e951bd2ec547984` and
  closure `37f25a8b84e0d4060b36fe0c86ff8bea8e4aa3aa` are already published.
  Previous Task 2.6 implementation:
  **`9f75aec59d732b2662c1b2c7626f2a8f48887619`**. Published validation/pause
  closure: **`ea6f09ccfde9811b7be233a960b984c1c74c1e6a`**. This subsequent
  documentation checkpoint records completed publication; verify actual HEAD
  and remote on return rather than treating an embedded hash as current forever.
- Accepted local/remote `main`: **`66193769d1cbc59cd8630df295b9a784b9c64642`**,
  independently reread remotely before Task 2.9 publication. Do not move main.
- Phase 0, Phase 1 and Tasks 2.1/2.2 are founder-accepted. Tasks 2.3–2.10 are
  **TECHNICALLY VALIDATED / PENDING FOUNDER ACCEPTANCE** only in reported scope.
- Original Task 2.3 publication: `46b448dbfd5670326a19d2292801181939ab2dd0`.
  Mandate activation: `df1250064011428b88a6ef7aae8b0c42521f5e95`.
- Corrected Task 2.3 branch local/remote:
  `3677fee97e3617ee65e2828d222008ba0952bb3e` (includes correction and checkpoint).
  Integration was created there by normal atomic push; main was not pushed.
- Verified completed integration publications: Task 2.4 closure
  `a1d585d117802fb8e595f089b0604527d61047d2`, then Task 2.5 closure
  `c58580fb3ad01e0e98243652c1c3d1f8dafdf01f`. Task 2.6 code and pause closure
  were then published successfully by normal fast-forward from `c58580f` to
  `ea6f09ccfde9811b7be233a960b984c1c74c1e6a`. No main push occurred.

## Completed technical sequence and evidence

| Task | Implementation | Actual pilot execution on final source | Separate source review |
|---|---|---|---|
| 2.3 severe-loss correction | `0559ebdaaaf28c7e9b8f423eda158abe13093b8d` | 168 tests +1 doctest; checks/docs PASS | `review_t23_final`: PASS |
| 2.4 fixed AccountInfo authentication | `9f4f1064deeef78a3cbea2e9f84c560e87166f20` | 191 tests +1 doctest; checks/docs PASS | `review_t23_final`: PASS |
| 2.5 initial contribution bootstrap | `9b997f364d62b0796008b2f7fb3f905acf64a2e5` | 209 tests +1 doctest; checks/docs PASS | `review_t23_final`: PASS |
| 2.6 protected principal SOL deposit composition | `9f75aec59d732b2662c1b2c7626f2a8f48887619` | **231 tests +1 doctest**; checks/docs PASS | `review_t23_final`: PASS |
| 2.7 isolated KIF claims | `10dceb5b2eac691ff19840190e951bd2ec547984` | **255 tests +1 doctest**; checks/docs PASS | `review_t23_final`: PASS |
| 2.8 current guardian/Clock snapshot authentication | `c815474eea9a7854c3b495974d891f4dd1c67a27` | **277 tests +1 doctest**; checks/docs PASS | `review_t23_final`: PASS |
| 2.9 validated state envelopes/atomic byte persistence | `36152c157773737eca357e5dbfefd3f0900b6eb3` | **298 tests +1 doctest**; checks/docs PASS | `review_t23_final`: PASS |
| 2.10 isolated KIF execution library/host model | Normal commit pending | **318 tests +1 doctest**; checks/docs PASS | `review_t23_final`: PASS |

Pilot commands use `/home/jerem/.cargo/bin/cargo +1.97.1`, `--locked --offline`:
`test --workspace --all-targets --quiet`, `test --workspace --doc`,
`check --workspace --all-targets`, the same check with `--all-features`, and
`doc --workspace --no-deps` with `RUSTDOCFLAGS='-D warnings'`. Rustfmt is absent;
no formatting pass was claimed or component installed. Final source was frozen
and hashes rechecked through gates. Diff whitespace and targeted ownership/
credential/generated-file checks passed. Host/model evidence is not runtime/CPI
or Testnet evidence, and AI review is not a professional independent audit.

**Preserved T23-R1 (P2), resolved:** original `World::finalize`/`settle` subtracted
already-used pending SOL from a smaller retained value before entering recovery.
Only the two recovery comparison inputs now use zero in that severe-loss case;
generic checked arithmetic and production math remain unchanged. Exact cases
retained 99 / pending use 4000 and retained 100 / pending use 8050 are regressions.
Finalization commits recovered stake and both rents while retaining HWM;
settlement commits only the recovery header and discards speculative payments,
KIF compound/carry and liability changes. The pilot personally ran the original
164 tests +1 doctest and reproduced both failures on `46b448d`. The writer's four
new tests failed on the old helper, then passed after correction. One older
review agent returned INCOMPLETE because of tool approval handling; it was never
counted as PASS. The replacement reviewed actual source. Old ephemeral binary
`/tmp/piv1-task23-recovery-review-46b448d` still represents OLD code; use committed
regressions. Full evidence: [Task 2.3 report](TASK_2_3_VAULT_RECONCILIATION_MODEL.md).

Task 2.4 authenticates Config, ActiveDistribution and seven fixed custody accounts
using actual host AccountInfo bytes, canonical PDAs/bumps, owner/layout/rent/token
checks and state binding. It narrowly permits the canonical zero System Program
ID in that role. Pinned SPL Token 8.0.0 added six locked transitive packages; the
pilot verified existing package identities/checksums unchanged. No layout change.
[Task 2.4 report](TASK_2_4_ACCOUNT_AUTHENTICATION.md) records 23 writer account
tests plus nine Config tests and exact authentication/trust limitations.

Task 2.5 establishes initial principal from full recognized pending SOL/tokens,
with no economic history, fabricated yield or arbitrary initial sequence. The
true empty host fixture establishes its audit once before actions. Zero-valued
positive token units remain principal and prevent replay. No general Idle intake
or staking was implied. Writer: 49 focused/affected tests PASS; old composition
counters unchanged. [Task 2.5 report](TASK_2_5_INITIAL_CONTRIBUTION_BOOTSTRAP.md).

Task 2.6 independently verifies protected zero-fee deposit receipts, normalized
individual custody, combined holdings/supply, exact post-pool deltas and historical
book value. Only historical SOL/token units change; HWM and all other state remain
unchanged. Deposited native/minted-user audit counters reconcile to the unchanged
pool audit; prior withdrawal/burn/fee/rent/recovery equations and baselines remain.
Writer `implement_t26_deposit`: initial 20 tests, then final 95 focused/affected
tests including 22 deposit regressions PASS. Pilot read all source/tests and ran
the 231-test workspace gates. Reviewer read exact final diff/all 22 tests and
found no actionable defect; no reviewer builds. Report status cleanup only after
freeze. Genuine later deposit preserves cooldown carry 13, both rents totaling
30, fees/burns and completed history. [Task 2.6 report](TASK_2_6_PROTECTED_PRINCIPAL_DEPOSIT.md).

## Current state and next action

Task 2.7 is **TECHNICALLY VALIDATED / PENDING FOUNDER ACCEPTANCE** after the
founder resumed from published checkpoint `bff59bb`. Its isolated four-account
claim path authenticates immutable earned ownership and protects full aggregate
backing, carry, rent, excess, replay and exact atomic host effects. The
[Task 2.7 report](TASK_2_7_ISOLATED_KIF_CLAIMS.md) records the contract and limits.
No new economic decision was required.

The available `implement_t26_deposit` agent completed the sole Task 2.7 writer
assignment; `review_t23_final` completed final separate review with **PASS / no
actionable findings** on the exact nine-file diff and all 24 tests. New-agent
creation hit the thread limit, but existing native agents are callable. The
writer froze source and released the build slot after 69 affected tests passed
(24 claim, 23 fixed-account and 22 unchanged reconciliation tests). The pilot
then independently ran **255 workspace tests +1 doctest**, default/all-feature
checks and warnings-denied documentation: all PASS, with source hashes unchanged.
Detailed evidence and limitations are in the Task 2.7 report; temporary logs are
`/tmp/piv1-t27-pilot-20260909T095218Z`. No build or review remains active. All nine
source hashes still match the frozen inventory. The source/report wording
clarifications distinguish modeled credits and newly baselined imported state
from actual earning authority and continuous cross-World custody evidence.
Implementation commit: `10dceb5b2eac691ff19840190e951bd2ec547984`. Its reviewed
closure `37f25a8b84e0d4060b36fe0c86ff8bea8e4aa3aa` was normally fast-forward
pushed from `bff59bb` and independently verified remotely; main remains `6619376`.
No history rewrite, other branch push or sensitive operation occurred.

Completed task: [Task 2.8 current guardian/Clock snapshot authentication](TASK_2_8_GUARDIAN_CLOCK_SNAPSHOT_AUTHENTICATION.md).
The exact written scope passed separate review after the pilot/reviewer inspected
canonical requirements and pinned Clock code. The existing `implement_t26_deposit`
agent is reused as sole writer, with `review_t23_final` reserved for final review.
Authentication reads exactly Config, current registry, six current reward records
and canonical Clock; it preserves all layouts, exact period equality and historical
claim ownership. No current-six/global liability equality is imposed. No heartbeat
pause policy, state writer or live operation is introduced. The writer's final 69 affected tests
passed (22 snapshot, 23 fixed-account, 24 isolated-claim), without warnings or
failures, and the slot was released. The pilot inspected all source/tests and
independently ran **277 workspace tests +1 doctest**, default/all-feature checks
and warnings-denied documentation: all PASS with unchanged source hashes.
Temporary evidence is `/tmp/piv1-t28-pilot-20260909T102244Z`; frozen diff/inventory
are `/tmp/piv1-t28-final-37f25a8.diff` and `/tmp/piv1-t28-frozen-source.json`.
Separate final review of the five-file diff, all 22 tests, frozen hashes and
complete report returned **PASS / no actionable findings**. No build remains
active. Task 2.8 is **TECHNICALLY VALIDATED / PENDING FOUNDER ACCEPTANCE**.
Implementation commit: `c815474eea9a7854c3b495974d891f4dd1c67a27`. This documentation
checkpoint was normally fast-forward published as closure
`440e83e26d36df911ccfafac97d89b79b8b4c694` and independently verified remotely;
accepted main remains `6619376`. No history rewrite or sensitive operation occurred.

Completed task: [Task 2.9 state-envelope persistence](TASK_2_9_STATE_ENVELOPE_PERSISTENCE.md).
Read-only separate assessment confirmed a real gap: production readers require
zero-filled tails, while envelope writing exists only in host support. The scoped
work adds typed encoding and atomic existing-account byte persistence for the
four already-authenticated state types. WithdrawalLeg persistence and all
authorization/handler policies are excluded. Exact written-scope review passed
with no required correction. The existing
`implement_t26_deposit` agent receives the sole writer assignment and focused
build slot; `review_t23_final` will review the final exact diff separately.
Task 2.9 is **TECHNICALLY VALIDATED / PENDING FOUNDER ACCEPTANCE**.
The four source/test files are frozen; no build remains active. Writer evidence:
initial six fixture lifetime compilation errors, corrected 19 tests PASS,
a mistyped target failing before build, then final **112 affected tests PASS**
(21 persistence plus 91 existing account/Clock/claim/reconciliation tests).
The pilot inspected all source/tests and ran **298 workspace tests +1 doctest**,
default/all-feature checks and warnings-denied docs: all PASS without warnings,
with source set/hashes unchanged. Evidence is
`/tmp/piv1-t29-pilot-20260909T104827Z`; final diff and inventory are
`/tmp/piv1-t29-final-440e83e.diff` and `/tmp/piv1-t29-frozen-source.json`.
The Config Option-tag and synthetic sequence fixture corrections are inspected
and resolved; no production correction was needed. Separate final review of the exact four-file diff, all 21 tests, hashes and
complete report returned **PASS / no actionable findings**. Implementation
commit: `36152c157773737eca357e5dbfefd3f0900b6eb3`. This checkpoint records completed
review/validation and was normally fast-forward published as closure
`44d70ec911ad3a78738fb90a04ac82eec3ca44f2`; independent remote reads confirm that
ref and unchanged accepted main. No history rewrite or sensitive operation occurred.

Current scope: [Task 2.10 isolated KIF claim execution](TASK_2_10_ISOLATED_KIF_CLAIM_EXECUTION.md).
Separate read-only dependency assessment recommends connecting the completed
authentication, checked claim and persistence through a fixed System transfer.
Canonical checks-effects-interactions and exact post-custody verification apply.
The pilot independently verified the pinned host-CPI limitation: Anchor invokes
solana-invoke 0.4.0, which panics on host; solana-cpi 2.2.1 returns host success
without transfer. The scope requires an explicit recorded/emulated host seam,
never false runtime evidence. Written-scope review passed after adding explicit
mutable data/lamport borrow preflight on both native accounts before bookkeeping.
Existing `implement_t26_deposit` receives the sole writer assignment and focused
build slot; `review_t23_final` will review the final exact diff separately.
Task 2.10 is **TECHNICALLY VALIDATED / PENDING FOUNDER ACCEPTANCE**.
The writer froze four source/test files and released the focused build slot.
Writer evidence: initial 16 tests PASS with one unused test import warning;
import removed and four regressions added; final **132 affected tests PASS**
(20 execution, 24 claim, 21 persistence, 23 account, 22 Clock, 22 reconciliation),
without warnings/failures. No failed writer commands were reported.
The pilot read all final source/tests and ran **318 workspace tests +1 doctest**,
default/all-feature checks and warnings-denied docs: all PASS without warnings,
with source set/hashes unchanged. Evidence directory:
`/tmp/piv1-t210-pilot-20260909T111423Z`; final diff/inventory:
`/tmp/piv1-t210-final-44d70ec.diff`, `/tmp/piv1-t210-frozen-source.json`.
Separate final review of all four source/test files, all 20 tests, hashes and
complete evidence returned **PASS / no actionable findings**. No source
correction or active build remains. Normal commits/publication are next. A
read-only assessment is examining a concrete instruction ABI/entrypoint boundary
and safe build prerequisites; no later implementation is scoped or dispatched. Production fixed invocation remains unexecuted;
ordinary host calls reject, while the explicit host seam models payment and
transaction rollback with the unchanged original custody audit.
Heartbeat pause semantics remain unestablished here.
Initialization/state writes, heartbeat handling and real runtime/adapter
integration remain separate dependencies.
[PIV1_TEST_PLAN.md](PIV1_TEST_PLAN.md) maps requirements to evidence and
remaining runtime/Testnet gates.

## Deferred risks and sensitive gates

- **OPEN deposit liveness:** all actual deposit fees and any integer historical
  value loss reject. Pinned SPL arithmetic can turn a zero-fee 700-lamport input
  into tokens worth 699; 707 is exact in the documented fixture. Protected
  slippage alone does not solve this. Some SOL stays queued. General fee/loss
  support needs a confirmed resolution, never an implicit subsidy/HWM exception.
- Runtime executing program ID/Rent/Clock, official pool/mint/list/source identity,
  collision-safe production snapshot identity, initialization/state writes,
  signer/destination/privileges, real transfers/CPI and SBF remain unproven.
- Operational surplus has no authenticated funding baseline. Token/temporary
  native excess is unsupported; one extra token-account lamport is visible to
  base auth but blocks the economic accessor. Resolve/contain this liveness path
  before handlers. Task 2.9 now provides canonical zero-tail persistence for
  four state types; future handlers must use it with authorized transitions.
- General Idle integration must preserve pending-SOL priority and no-yield/
  insufficient behavior. Real multi-leg sizing/source order/minima/slippage,
  KIF claim handlers, governance and governed recovery remain separate work.
- **D-026 live-operation approval: NONE.** No new keys/signing, public-Testnet
  deployment/fund-moving lifecycle, Mainnet, real funds or authority transfer.
  Prepare the exact cluster/identities/artifact/budget/operations approval card
  before those live actions; continue independent safe engineering meanwhile.
- No Mainnet action, deployment, fund movement, key creation, signing, authority
  transfer or unrelated secret access occurred in this pilot sequence.
- Before publication the pilot found effective `core.hooksPath` unset, sample-only
  hooks, no tracked `.github`/`.cargo` automation. Recheck before publishing.
  No force push, history rewrite, automatic release/tag or unrelated publication.
