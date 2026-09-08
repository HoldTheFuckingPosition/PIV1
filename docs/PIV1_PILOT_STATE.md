# PIV1 technical pilot checkpoint

Last verified: **2026-09-08 UTC**. Read this file, `AGENTS.md`, then verify Git
before acting. This is an execution checkpoint, not a competing economic spec.

## Goal, sources and authority

- Goal: reviewed PIV1 implementation, authorized public-Testnet end-to-end
  evidence, and a practical founder testing handover. Mainnet is excluded.
- Source order: `PIV1_DECISIONS.md`, `PIV1_MASTER_SPEC.md`,
  `PIV1_CODEX_EXECUTION_PLAN.md`, newer explicit component decisions. D-026
  activates [the complete mandate](PIV1_TECHNICAL_PILOT_MANDATE.md).
- Founder discussion: French and brief. Code/docs/delegation/commits: English.
- HTFP context: PIV1 is the first HTFP infrastructure component. Other tokens,
  Team Owner components and MTT are outside this mandate. No
  `HTFP_MASTER_CONTEXT.md` was found in this project; no access to other chats
  or browser history is assumed.
- Confirmed split is 59% / 19.5% / 19.5% / 2%; six guardians with 4-of-6 authority.
  Direct JitoSOL strategy, native SOL payments, protected SOL HWM, pending
  isolation, rent isolation and KIF rules remain canonical, not pilot choices.

## Verified repository and acceptance

- User: `jerem`, uid 1001. Repository: `/home/jerem/piv1`; one worktree.
- Existing remote: `github-piv1:HoldTheFuckingPosition/PIV1.git`.
  WeatherTrader2 is the connected project label in the mandate, not a different
  repository discovered by this pilot.
- Current branch at activation: `task/2.3-vault-reconciliation-model`.
- Reviewed implementation HEAD: `46b448dbfd5670326a19d2292801181939ab2dd0`.
- Actual local accepted `main`: `66193769d1cbc59cd8630df295b9a784b9c64642`,
  an ancestor of the reviewed implementation. Worktree/index were clean.
- Phase 1 and Tasks 2.1/2.2: founder-accepted. Task 2.3: IMPLEMENTED /
  PUBLISHED FOR REVIEW (founder handover) / NOT FOUNDER-ACCEPTED.
  No live remote fetch was used to infer publication during the initial review.
- Development integration branch `integration/piv1-testnet` will be created
  from the corrected, separately reviewed Task 2.3 tip. `main` must not move.
- Git checkpoint identity is the commit containing this document; code/test
  identities are recorded explicitly below, avoiding self-referential hashes.

## Active bounded task: Task 2.3 recovery correction

Preserve the existing implementation and scope in
[TASK_2_3_VAULT_RECONCILIATION_MODEL.md](TASK_2_3_VAULT_RECONCILIATION_MODEL.md).

**OPEN finding T23-R1, P2:** the host `World::finalize` and `World::settle`
subtract used pending SOL (`U`) from remaining protected value before calling
the recovery-capable pure transition. If value is below U, checked subtraction
returns `CumulativeReconciliationMismatch`, bypassing `RecoveryRequired`.
Finalization also discards staged stake and rent recovery. Both independent
review subagents confirmed the source reasoning; the pilot executed reproduction.

- Finalization reproduction: `World::new(4000,0,0,0,3,ZERO,100000)`, open at
  900000, initiate source 1, reduce pool total to 1000, advance epoch, finalize
  leg 0. Remaining principal book value = 99; U = 4000. Result: error, unchanged
  `WithdrawalActive`, recovered rent = 0.
- Liquid settlement reproduction: initial pending SOL 10000, open at 900000,
  reduce pool total to 1000, settle. Remaining value = 100; U = 8050. Result:
  error, unchanged `EscrowFunded`.
- Expected: explicit conservative handling of underprotection; finalization
  commits recovered custody/rent and recovery state; settlement commits only
  recovery header, with no speculative payments or new liabilities. HWM never
  decreases. Preserve ordinary malformed/overflow rejection and atomicity.
- Gate: smallest compatible correction; meaningful regressions for both paths
  and boundary behavior; targeted and workspace locked/offline validation;
  a separate reviewer of the exact final diff; pilot evidence inspection;
  normal commit and checkpoint. No new dependency, schema, handler or CPI.

## Pilot-executed evidence on 46b448d

Rust executable prefix: `/home/jerem/.cargo/bin/`; toolchain 1.97.1.

| Command / inspection | Observed result |
|---|---|
| `cargo +1.97.1 test --workspace --all-targets --locked --offline` | 164 passed, zero failed/ignored |
| `cargo +1.97.1 test --workspace --doc --locked --offline` | 1 math doctest passed |
| `cargo +1.97.1 check --workspace --all-targets --all-features --locked --offline` | Passed |
| `git diff --check 6619376..HEAD` | Passed |
| Temporary stdin `rustc +1.97.1` reproducer linked to existing host rlibs | Both T23-R1 paths reproduced; no source edits |
| Separate source reviews | Two actual subagents; no additional established defect in bounded scope |

The older executor also reported 164 tests and one doctest; that report is
historical evidence, independently reproduced above. The temporary executable
is `/tmp/piv1-task23-recovery-review-46b448d`; durable regressions will replace
reliance on this disposable artifact. No source change was made during review.

## Deferred risks and permission boundary

- Host observations do not authenticate accounts, owners, PDAs, mint, signers,
  destinations, real transfers, CPI, runtime locks or Clock/Rent.
- Operational surplus cannot be derived without authenticated funding evidence.
  Unexpected native lamports in token/temporary accounts remain unsupported.
- Exact live SPL/Jito mapping, snapshot identity, multi-leg target/minimum/
  slippage sizing, principal deposits and production KIF claim handling remain
  deferred. Existing scalar claim effects are not a handler proof.
- Founder acceptance remains pending for Task 2.3 and any later technical work.
- Live-operation authorization: **NONE under D-026**. No new keys/signing,
  public-Testnet deployment or fund-moving lifecycle until the concrete package
  is approved. No Mainnet, real-value movement or authority transfer authorized.
- Initial hook check: no local `core.hooksPath`; only `.sample` files in
  `.git/hooks`; no tracked `.github` or `.cargo` files. Recheck before publication
  and inspect any newly discovered hook/CI before executing it.

## Next action and continuity

Record this activation/checkpoint, delegate one writer for T23-R1, then obtain
a separate final-diff review and validate. Update this checkpoint before
starting the next dependency from the remaining canonical Phase 2 plan;
Task 2.4 has no inferred content merely from its number. Do not stop for a
routine technical approval. A paused run is not continuing execution: resume
by reading this checkpoint and verifying Git, agents and actual evidence.
