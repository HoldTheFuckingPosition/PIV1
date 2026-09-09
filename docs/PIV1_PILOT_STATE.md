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
  **Task 2.11 `2eeefba0abc226bcfcadddb5f248f12ca589e09d`**. Normal reviewed
  integration publication follows this hash-recording checkpoint. Task 2.10
  closure **`9f9a8dbba132f96e4c76a8385844570746b0636f`** was independently
  reread remotely from a clean worktree at publication. Task 2.9
  closure **`44d70ec911ad3a78738fb90a04ac82eec3ca44f2`** was independently
  verified remotely from a clean worktree. Inspect actual HEAD/remote on takeover.
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
  independently reread remotely after Task 2.10 publication. Do not move main.
- Phase 0, Phase 1 and Tasks 2.1/2.2 are founder-accepted. Tasks 2.3–2.11 are
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
| 2.10 isolated KIF execution library/host model | `c38a7b0d7122144bf3082cec7e57bbea7e61cc10` | **318 tests +1 doctest**; checks/docs PASS | `review_t23_final`: PASS |
| 2.11 claim ABI/runtime-ID instruction boundary | `2eeefba0abc226bcfcadddb5f248f12ca589e09d` | **335 tests +1 doctest**; eight gates PASS | `review_t23_final`: PASS |

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

## Reviewed progression after resumption

- [Task 2.7](TASK_2_7_ISOLATED_KIF_CLAIMS.md): isolated immutable earned ownership,
  historical/paused claims, full backing/carry/rent/excess and exact atomic host
  custody. Writer 69 affected tests; pilot 255 +1; separate exact review PASS.
  Closure `37f25a8b84e0d4060b36fe0c86ff8bea8e4aa3aa` published normally.
  Modeled credits do not prove earning authority; imported fixtures establish
  a new baseline, not continuous cross-World custody. Original audits remain.
- [Task 2.8](TASK_2_8_GUARDIAN_CLOCK_SNAPSHOT_AUTHENTICATION.md): read-only current
  registry/six rewards/canonical Clock snapshot, exact period equality, preserved
  historical claims and layouts. No current-six/global liability equality or
  heartbeat pause policy. Writer 69 affected; pilot 277 +1; separate review PASS.
  Closure `440e83e26d36df911ccfafac97d89b79b8b4c694` published normally.
- [Task 2.9](TASK_2_9_STATE_ENVELOPE_PERSISTENCE.md): canonical zero-tail typed
  envelopes and atomic existing-account byte writes for four authenticated state
  types. No transition authority, initialization or WithdrawalLeg persistence.
  Fixture lifetime errors, mistyped target, Config Option offset and sequence
  assertions were corrected before freeze; no production defect found.
  Writer 112 affected; pilot 298 +1; separate review PASS. Closure
  `44d70ec911ad3a78738fb90a04ac82eec3ca44f2` published normally.
- [Task 2.10](TASK_2_10_ISOLATED_KIF_CLAIM_EXECUTION.md): fixed System transfer
  wiring, simultaneous mutable native data/lamport preflight, canonical CEI
  persistence and exact fresh postconditions. Ordinary host execution rejects;
  explicit invocation/transaction modeling retains the unchanged custody audit.
  Raw errors can leave effects and must propagate to runtime rollback. Writer's
  unused import warning was removed; final 132 affected tests passed. Pilot
  318 +1, checks/docs, frozen source hashes and separate exact review PASS.
  Normal atomic fast-forward publication `44d70ec` to closure
  `9f9a8dbba132f96e4c76a8385844570746b0636f` completed. Independent remote
  reads confirmed integration, unchanged main/Task 2.3 and a clean worktree.
  No actual signed CPI/SBF runtime evidence or founder acceptance is implied.

The reports contain full source inventories, failed/final commands, individual
review verdicts and temporary evidence paths. Committed code/reports are durable;
`/tmp` logs can disappear. New-agent creation reached the thread limit, but the
existing native writer/reviewer are callable and reused. No delegation is assumed.

## Current state and next action

Current scoped dependency: [Task 2.11 KIF claim instruction boundary](TASK_2_11_KIF_CLAIM_INSTRUCTION_BOUNDARY.md).
The pilot inspected canonical K-012/master claim, instruction and event requirements,
D-003, pinned Anchor/native entrypoint/event/Rent code and the Task 1.1 build-key
record. The written scope defines exact 24-byte ABI/five accounts, runtime-ID
entrypoint, runtime Rent, explicit stable errors and factual success-only event.
It preserves existing execution/rollback protections and all sensitive gates.
Separate written-scope review returned PASS with no required correction.
The pilot used the existing `implement_t26_deposit` as sole writer with
the focused build slot, now released. `review_t23_final` completed separate
read-only prerequisites research and final exact source/test review.
Task 2.11 is TECHNICALLY VALIDATED / PENDING FOUNDER ACCEPTANCE.
No Anchor/SBF build or new key creation is permitted in this bounded task.
The writer froze the ten source/test/manifest/comment files and released the
build slot. Final writer evidence: 94 affected tests and all supported feature
checks PASS after correcting missing Discriminator/IdlBuild imports and an event
tag slice-to-array fixture conversion; initial two unused test-import warnings
were resolved without suppression. The pilot inspected all source and all 17 new
tests, then ran **335 workspace tests +1 doctest**, default/all-feature plus
explicit no-entrypoint/cpi/idl-build checks and warnings-denied docs: all PASS,
no warnings/errors, complete source set and ten frozen hashes unchanged.
Evidence: `/tmp/piv1-t211-pilot-20260909T114436Z`; inventory and exact patch:
`/tmp/piv1-t211-frozen-source.json`, `/tmp/piv1-t211-final-9f9a8db.diff`.
No build remains active. Complete writer/pilot evidence and separate final review
of the ten-file diff, all 17 tests, hashes and full report returned PASS with no
actionable findings. No source correction after freeze. Implementation commit:
`2eeefba0abc226bcfcadddb5f248f12ca589e09d`. This documentation closure records the
hash before normal reviewed integration publication.
Read-only separate research identified a proposed direct cached platform-tools
Cargo/Rust compilation stage that avoids builder post-processing. The pilot
independently read pinned compiler/post-processing/target primary sources.
No such build has been executed or authorized inside Task 2.11. Preserve the
future artifact/entrypoint/stack/heap/runtime gates; do not infer runtime proof.


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
