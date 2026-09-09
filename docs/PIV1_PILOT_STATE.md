# PIV1 technical pilot checkpoint

Execution: **PAUSED AT FOUNDER REQUEST — awaiting the founder's return.**
The founder asked to stop for the night after a durable checkpoint. Do not start
later technical work or automatic goal progression until they return. This is
an execution pause, not goal completion, a blocker or founder acceptance.

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
- Branch: `integration/piv1-testnet`. Current implementation HEAD:
  **`9f75aec59d732b2662c1b2c7626f2a8f48887619`**. Closure documents are being
  checkpointed normally; verify actual HEAD before acting.
- Accepted local/remote `main`: **`66193769d1cbc59cd8630df295b9a784b9c64642`**,
  independently reread remotely before Task 2.6 publication. Do not move main.
- Phase 0, Phase 1 and Tasks 2.1/2.2 are founder-accepted. Tasks 2.3–2.6 are
  **TECHNICALLY VALIDATED / PENDING FOUNDER ACCEPTANCE** only in reported scope.
- Original Task 2.3 publication: `46b448dbfd5670326a19d2292801181939ab2dd0`.
  Mandate activation: `df1250064011428b88a6ef7aae8b0c42521f5e95`.
- Corrected Task 2.3 branch local/remote:
  `3677fee97e3617ee65e2828d222008ba0952bb3e` (includes correction and checkpoint).
  Integration was created there by normal atomic push; main was not pushed.
- Verified completed integration publications: Task 2.4 closure
  `a1d585d117802fb8e595f089b0604527d61047d2`, then Task 2.5 closure
  `c58580fb3ad01e0e98243652c1c3d1f8dafdf01f`. Before the next normal push,
  remote integration still equals `c58580f`; Task 2.6 publication is pending.

## Completed technical sequence and evidence

| Task | Implementation | Actual pilot execution on final source | Separate source review |
|---|---|---|---|
| 2.3 severe-loss correction | `0559ebdaaaf28c7e9b8f423eda158abe13093b8d` | 168 tests +1 doctest; checks/docs PASS | `review_t23_final`: PASS |
| 2.4 fixed AccountInfo authentication | `9f4f1064deeef78a3cbea2e9f84c560e87166f20` | 191 tests +1 doctest; checks/docs PASS | `review_t23_final`: PASS |
| 2.5 initial contribution bootstrap | `9b997f364d62b0796008b2f7fb3f905acf64a2e5` | 209 tests +1 doctest; checks/docs PASS | `review_t23_final`: PASS |
| 2.6 protected principal SOL deposit composition | `9f75aec59d732b2662c1b2c7626f2a8f48887619` | **231 tests +1 doctest**; checks/docs PASS | `review_t23_final`: PASS |

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

Task 2.6 code is committed; the pilot is finishing only the pause checkpoint
and its normal reviewed publication. All delegated writers/reviewers are finished;
all started builds have exited. No later task has started. Execution then remains
paused until the founder returns; no background engineering is claimed.

When the founder returns, first verify actual user/branch/HEAD/worktree/remote
and agent state, then scope the remaining authenticated KIF claim/guardian
protections against K-012 and actual existing code. Select the smallest useful
bounded implementation after separate scope review. No later implementation has
started. [PIV1_TEST_PLAN.md](PIV1_TEST_PLAN.md) maps requirements to evidence and
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
  before handlers. Future writers must zero unused state-envelope padding.
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
