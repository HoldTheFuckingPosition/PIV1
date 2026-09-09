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
- Branch at this checkpoint: `task/2.3-vault-reconciliation-model`.
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
  was inferred. Task 2.4 implementation has not started at this checkpoint.
- Next: create `integration/piv1-testnet` from the reviewed correction plus this
  checkpoint. The remote branch was absent at inspection. Preserve any newer
  legitimate branch/work instead of recreating or resetting it.

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

## Next bounded dependency

Scope Task 2.4 from the remaining Phase 2 account/handler protections:
authenticate Config, ActiveDistribution and fixed economic custody using actual
`AccountInfo`, then derive observations for existing reconciliation. Use the
accepted Phase 0 section 5.1 seeds and existing bounded payloads. Keep guardian
reward/rotation policy, claim/heartbeat handlers, CPI and entrypoint separate.
The next task needs an explicit scope/tests/completion gate before implementation.

Known ordinary mapping correction to address there: the pure Config validator
currently rejects every zero public key, including Solana's actual System
Program ID. Permit that identity narrowly for its program role while retaining
all other default-address/alias rejection; real account validation must enforce
canonical program identities. This is not a Task 2.3 or economic change.

## Deferred risks and permission boundary

- Host observations do not yet authenticate real accounts/owners/PDAs/mints,
  signer/destination, transfers/CPI, runtime locks or Clock/Rent.
- Operational surplus lacks authenticated funding evidence. Unexpected native
  excess in token/temporary accounts is unsupported; no reserve sweep is implied.
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
