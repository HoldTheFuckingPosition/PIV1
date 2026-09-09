# PIV1 Repository Instructions

## Current execution state

The founder resumed on 2026-09-09 with the confirmed GPT-6 family. Task 2.13
keyless local SBF claim validation now has final separate technical PASS: 19 tests,
60 message cases and complete account evidence. Founder acceptance remains pending.
The pilot is recording and publishing the reviewed checkpoint; no build/runtime
process remains running. Read `docs/PIV1_PILOT_STATE.md` for actual Git/task state.
Historical pauses and preparation failures are preserved in the linked history.
The founder requested economical credit use: focus on changed inputs and avoid
redundant broad reviews or validation; keep all required safety gates.

## Active technical pilot mandate

Read [docs/PIV1_PILOT_STATE.md](docs/PIV1_PILOT_STATE.md) on takeover and verify
the actual user, branch, HEAD and worktree before acting. The founder activated
[the technical pilot mandate](docs/PIV1_TECHNICAL_PILOT_MANDATE.md), recorded as
D-026, in this connected session. It supersedes earlier review-only and
per-task permission/stop requirements for bounded PIV1 technical work toward
founder Testnet testing. Use one delegated writer and a separate reviewer;
checkpoint each completed task before continuing. Technical validation is not
founder acceptance. Keep `main` founder-accepted; use `integration/piv1-testnet`
for the reviewed development sequence. The mandate's economic-decision,
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

Phase 0, Task 0.5, Tasks 1.1-1.4, and the complete Phase 1 specification-as-code foundation are COMPLETE / FOUNDER-ACCEPTED. The final accepted Task 1.3 implementation tip is `527e381661fe0cfc27e07ad9b44e1601a638ae75`; the accepted Task 1.4 implementation is `06c39429f3237f6974e21217670c3f0d30b0a571`. Task 2.1 is COMPLETE / FOUNDER-ACCEPTED at initial implementation commit `33b1e539f969432f82635d1ca76c59d89f0ec233` and final corrected tip `cb90d468eff4dce60552ba15b2b267b364a47827`. Task 2.2 is COMPLETE / FOUNDER-ACCEPTED at implementation commit `e3233b96b533a620e8037d5231baede10877217f`. Phase 2 is IN PROGRESS. Task 2.3 is TECHNICALLY VALIDATED / PENDING FOUNDER ACCEPTANCE after correction `0559ebdaaaf28c7e9b8f423eda158abe13093b8d` on `task/2.3-vault-reconciliation-model`; Task 2.4 fixed-account authentication is TECHNICALLY VALIDATED / PENDING FOUNDER ACCEPTANCE at `9f4f1064deeef78a3cbea2e9f84c560e87166f20` on `integration/piv1-testnet`; Task 2.5 initial contribution bootstrap is TECHNICALLY VALIDATED / PENDING FOUNDER ACCEPTANCE at `9b997f364d62b0796008b2f7fb3f905acf64a2e5`; Task 2.6 protected principal SOL deposit composition is TECHNICALLY VALIDATED / PENDING FOUNDER ACCEPTANCE at `9f75aec59d732b2662c1b2c7626f2a8f48887619`, limited to zero-fee conversions preserving historical book value and HWM coverage; Task 2.7 isolated KIF claim authentication/accounting is TECHNICALLY VALIDATED / PENDING FOUNDER ACCEPTANCE at `10dceb5b2eac691ff19840190e951bd2ec547984` after 255 host tests, one doctest, checks/docs and separate review passed; Task 2.8 current guardian/Clock snapshot authentication is TECHNICALLY VALIDATED / PENDING FOUNDER ACCEPTANCE at `c815474eea9a7854c3b495974d891f4dd1c67a27` after 277 host tests, one doctest, checks/docs and separate review passed; Task 2.9 typed state envelopes and atomic existing-account byte persistence is TECHNICALLY VALIDATED / PENDING FOUNDER ACCEPTANCE at `36152c157773737eca357e5dbfefd3f0900b6eb3` after 298 host tests, one doctest, checks/docs and separate review passed; Task 2.10 isolated KIF claim execution with explicit host invocation/rollback evidence is TECHNICALLY VALIDATED / PENDING FOUNDER ACCEPTANCE at `c38a7b0d7122144bf3082cec7e57bbea7e61cc10` after 318 host tests, one doctest, checks/docs and separate review passed; Task 2.11 claim instruction ABI/runtime-ID boundary is TECHNICALLY VALIDATED / PENDING FOUNDER ACCEPTANCE at `2eeefba0abc226bcfcadddb5f248f12ca589e09d` after 335 host tests, one doctest, checks/docs and separate review passed, within `docs/TASK_2_11_KIF_CLAIM_INSTRUCTION_BOUNDARY.md`; Task 2.12 keyless SBF compilation is TECHNICALLY VALIDATED / PENDING FOUNDER ACCEPTANCE at `cee6072ad7b3155d7d5b30e6c0830beb6d00d4eb` after a clean target build, 339 host tests, one doctest, checks/docs and separate source/artifact review passed, limited to the static evidence in `docs/TASK_2_12_KEYLESS_SBF_COMPILATION.md`; Task 2.13 keyless local SBF claim execution is TECHNICALLY VALIDATED / PENDING FOUNDER ACCEPTANCE after 19 local tests, 60 message cases, full account evidence and separate final review in `docs/TASK_2_13_KEYLESS_SBF_CLAIM_EXECUTION.md`; subsequent bounded tasks are NOT STARTED. Task 2.1 accepts the narrow interface and host-mock evidence, not exact SPL/Jito behavior; the collision-safe, account-derived production snapshot identity and real protocol mapping remain Phase-3-provisional. Task 2.2 accepts only pure pending-vault intake/reconciliation and host-mock evidence. At Task 2.2 acceptance, fixed-account and transfer validation, real custody, handlers, CPI, localnet behavior, all-vault normalization, and composition cases involving pending SOL moved into distribution escrow were deferred; explicit-transfer handler callability during pause remains PROVISIONAL. Task 2.3 adds pure phase-dependent custody derivations and atomic host composition for pending-to-escrow-to-HWM and economic-vault normalization; it does not authenticate real accounts or transfers. Operational surplus derivation remains unsupported without an authenticated funding baseline. See `docs/TASK_2_3_VAULT_RECONCILIATION_MODEL.md` for the supported scope and deferred cases. That review-only next-action restriction is HISTORICAL after D-026 activation. The current technical task, reviewed development progression and permission boundaries are recorded in docs/PIV1_PILOT_STATE.md; founder acceptance remains pending. This AI-assisted review is not a professional independent audit.
