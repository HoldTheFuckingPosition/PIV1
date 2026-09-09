# PIV1 technical pilot checkpoint

Last verified: **2026-09-09 UTC**. Read this file and `AGENTS.md`, then verify
actual user, branch, HEAD, worktree and running agents before acting. This is
an execution checkpoint, not a competing economic specification.

## Goal and authority

- Goal: reviewed PIV1 implementation, authorized public-Testnet end-to-end
  evidence and practical founder testing handover. Mainnet is excluded.
- Canonical order: `PIV1_DECISIONS.md`, `PIV1_MASTER_SPEC.md`,
  `PIV1_CODEX_EXECUTION_PLAN.md`, newer explicit component decisions.
- **CONFIRMED D-026** activates [the complete pilot mandate](PIV1_TECHNICAL_PILOT_MANDATE.md).
  It permits successive bounded technical tasks, separate review, normal
  commits and reviewed development publication without routine approval.
  It does not grant founder acceptance or change economic/security decisions.
- French brief founder reports; English code/docs/delegation/commits.
- HTFP: PIV1 is the first infrastructure component. Other tokens/Team Owner
  components/MTT are out of scope. `HTFP_MASTER_CONTEXT.md` was not found in
  this project. No other ChatGPT/browser history access is assumed.
- Keep the canonical 59% / 19.5% / 19.5% / 2% split, six guardians/4-of-6,
  direct JitoSOL strategy, native SOL outputs, protected HWM, separate pending,
  KIF/carry, rent and fee categories. The mandate changes workflow only.

## Verified Git and acceptance

- User `jerem` (uid 1001), `/home/jerem/piv1`, one worktree.
- Remote `github-piv1:HoldTheFuckingPosition/PIV1.git`. WeatherTrader2 is the
  connected project label supplied by the founder, not another detected repo.
- Current branch: `integration/piv1-testnet`, created from checkpoint
  `3677fee97e3617ee65e2828d222008ba0952bb3e`. Worktree was clean at task entry.
- Actual local and remotely inspected accepted main:
  `66193769d1cbc59cd8630df295b9a784b9c64642`. Do not move main.
- Published original Task 2.3 implementation:
  `46b448dbfd5670326a19d2292801181939ab2dd0` (verified with `git ls-remote`).
- Mandate/context activation commit:
  `df1250064011428b88a6ef7aae8b0c42521f5e95`.
- **Task 2.3 corrected implementation:**
  `0559ebdaaaf28c7e9b8f423eda158abe13093b8d`.
- Phase 1 and Tasks 2.1/2.2 are founder-accepted. Task 2.3 is
  **TECHNICALLY VALIDATED / PENDING FOUNDER ACCEPTANCE**. No founder acceptance
  was inferred. Task 2.4 is also **TECHNICALLY VALIDATED / PENDING FOUNDER
  ACCEPTANCE** at `9f4f1064deeef78a3cbea2e9f84c560e87166f20`.
- Task 2.5 initial bootstrap is **TECHNICALLY VALIDATED / PENDING FOUNDER
  ACCEPTANCE** at `9b997f364d62b0796008b2f7fb3f905acf64a2e5`.
- Publication completed: normal atomic push advanced the existing Task 2.3
  branch from `46b448d` to `3677fee` and created `integration/piv1-testnet` at
  the same checkpoint. Both track the existing origin; main was not pushed.
- Task 2.4 publication completed by normal fast-forward: integration advanced
  from `3677fee` to closure checkpoint
  `a1d585d117802fb8e595f089b0604527d61047d2`. Remote main was independently
  reread at `6619376`; the Task 2.3 branch remains at `3677fee`.

## Completed Task 2.3 correction and evidence

**T23-R1, P2 — technically resolved.** Severe pool loss previously caused
`World::finalize`/`settle` to fail when retained value was below already-used
pending SOL, bypassing `RecoveryRequired`. Only the two recovery comparison
inputs now use a conservative zero in that case. Generic checked arithmetic,
normalized custody and pure-transition validation remain intact.

Finalization commits recovered stake and both rent amounts while retaining HWM;
settlement commits only the recovery header and discards speculative payments,
KIF compound/carry and liability changes. The exact reproductions remain covered:
retained 99 / pending use 4000 and retained 100 / pending use 8050, respectively.

| Evidence source | Actual result |
|---|---|
| Pilot initial review on 46b448d | 164 tests, 1 doctest, all-feature check PASS; both severe-loss failures reproduced with a temporary stdin Rust harness |
| Delegated writer `implement_t23_recovery` | Four new regression tests FAIL on old helper, then 22 composition tests PASS after correction |
| Pilot on final corrected Rust source / 0559ebd | 168 workspace tests, 1 doctest, default/all-feature checks, warnings-denied documentation PASS |
| Separate reviewer `review_t23_final` | Current source and exact frozen diff inspected; PASS / no actionable findings within host scope; no tests executed by reviewer |
| Git/targeted checks | Diff whitespace, changed-file ownership and credential-marker checks PASS; production source/math/manifests/lock/toolchain unchanged |

Commands used `/home/jerem/.cargo/bin/cargo +1.97.1` with `--locked --offline`:
`test --workspace --all-targets --quiet`, `test --workspace --doc`,
`check --workspace --all-targets`, the same check with `--all-features`, and
`doc --workspace --no-deps` with `RUSTDOCFLAGS='-D warnings'`.
Full writer/pilot evidence and limitations:
[TASK_2_3_VAULT_RECONCILIATION_MODEL.md](TASK_2_3_VAULT_RECONCILIATION_MODEL.md).

An older review agent was blocked in inherited manual tool-approval handling
and returned INCOMPLETE. It is not counted as a pass. The replacement reviewer
used working `cat`/`sed` reads and a pilot-captured exact diff. Old ephemeral
reproducer `/tmp/piv1-task23-recovery-review-46b448d` still reflects the OLD code;
use committed regressions for the corrected behavior. No review is represented
as a professional independent audit.

## Completed Task 2.4 and next dependency

Task 2.4 authenticates actual host `AccountInfo` backing for Config,
ActiveDistribution and the seven permanent native/token custody accounts.
Canonical PDA/bump, owner, allocation, discriminator, variable Borsh/zero-tail,
rent, token state and existing Config/round binding checks precede observations.
The canonical System Program zero key is now permitted narrowly for that role.
Existing payload fields/allocations and economics remain unchanged.

- Implementation: `9f4f1064deeef78a3cbea2e9f84c560e87166f20` on `integration/piv1-testnet`.
- Writer `implement_t24_accounts`: 23 focused account tests and 9 Config tests PASS.
- Pilot frozen-source gates: **191 workspace tests, 1 doctest**, default and
  all-feature checks, warnings-denied docs and diff whitespace PASS.
- Separate reviewer `review_t23_final`: actual final diff/source/all 23 test bodies
  inspected; PASS within the bounded read-only scope, no reviewer builds.
- Pilot independently verified discriminator hashes and every existing lock
  package identity/checksum. New direct dependency is pinned SPL Token 8.0.0;
  its six transitive additions are locked. No prior package changed.
- Missing rustfmt was reported, not installed or counted as a formatting pass.
- Report: [TASK_2_4_ACCOUNT_AUTHENTICATION.md](TASK_2_4_ACCOUNT_AUTHENTICATION.md).
  Host account fixtures do not prove runtime invocation, CPI or live behavior.

## Completed Task 2.5 and next dependency

Task 2.5 adds only the initial pending-to-principal boundary. It requires an
unpaused bound Idle state with no economic history and derives the full recognized
contribution value from normalized exact before/after custody plus checked pool
book value. It creates no yield snapshot and preserves the complete header,
sequence, clocks, guardian/KIF, rent and all unrelated accounting. Positive token
units with zero floored SOL value remain principal and still prevent replay.

- Implementation: `9b997f364d62b0796008b2f7fb3f905acf64a2e5`.
- Writer `implement_t25_bootstrap`: 18 bootstrap +22 composition +9 pending
  tests PASS; existing composition counters unchanged. Empty fixture audit
  is established once before actions; prior funded fixture behavior is preserved.
- Pilot: **209 workspace tests, 1 doctest**, default/all-feature checks,
  warnings-denied documentation and diff whitespace PASS on frozen source.
- Separate reviewer `review_t23_final`: complete actual six-file Rust/test diff
  and report inspected; PASS in scope. No reviewer builds were run.
- Dependencies, math, payload schemas and accepted distribution transitions
  remain unchanged. Only the shared floor-comparison visibility is broadened
  to the sibling bootstrap module. Targeted checks passed; main unchanged.
- Report: [TASK_2_5_INITIAL_CONTRIBUTION_BOOTSTRAP.md](TASK_2_5_INITIAL_CONTRIBUTION_BOOTSTRAP.md).
  Synthetic maximum-value checks are not host custody/conservation evidence.

Next action: scope the protected principal-SOL deposit dependency using the
actual accepted adapter and pinned SPL/Jito source. Verify fee, rounding,
actual minted-unit and HWM treatment against canonical requirements before
selecting the smallest implementation. Initial SOL remains in PrincipalSolQueue;
no staking or general idle integration was implemented by Task 2.5. Do not
select a new cost allocation or economic exception by implication. Complete this
checkpoint before the next bounded implementation task.

## Requirements and evidence index

[PIV1_TEST_PLAN.md](PIV1_TEST_PLAN.md) maps confirmed requirements to inspected
host evidence and remaining account/runtime/Testnet gates. It is an execution
index, not a replacement specification or a claim of completed integration.

## Deferred risks and permission boundary

- Task 2.4 adds host AccountInfo authentication, but the runtime executing
  program/Rent inputs remain explicit trust boundaries. Actual initialization,
  official pool/mint provenance, signer/destination/Clock, transfers/CPI and
  runtime locks remain unimplemented or unproven.
- Operational surplus lacks authenticated funding evidence. Unexpected native
  excess in token/temporary accounts is unsupported; no reserve sweep is implied.
  One extra token-account lamport is visible to base authentication but blocks
  the economic accessor. Resolve this liveness limitation before handlers.
  Future state writers must zero unused account-envelope padding.
- Narrow initial bootstrap is now modeled and tested. General idle integration
  remains deferred because its timing must preserve pending-SOL-first funding
  and no-yield/insufficiency rules. Actual initialization remains unimplemented.
- Exact SPL/Jito snapshot identity, multi-leg sizing/minimum/slippage mapping,
  principal deposits, production KIF claims and governed recovery remain deferred.
- **Live-operation approval under D-026: NONE.** No new keys or blockchain
  signing, public-Testnet deployment/fund-moving lifecycle, Mainnet, real funds
  or authority transfer. Prepare the exact mandate approval card before those
  live actions. Ordinary technical work continues under D-026.
- Hook/CI checks: effective `core.hooksPath` unset; `.git/hooks` contains only
  `.sample` hooks; no tracked `.github`/`.cargo` files. Recheck before publication.
- No force push, amend/rebase/squash/history rewrite, automatic tag/release,
  unrelated publication or unauthorized key/secret access.

Checkpoint every completed task before the next one. A paused turn is not
ongoing execution; resume by checking this document against actual Git and agents.
