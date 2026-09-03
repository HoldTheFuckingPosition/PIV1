# PIV1 Repository Instructions

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

- Work on one bounded task at a time and do not begin a later task automatically.
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
- Stop after the requested task.

Phase 0, Task 0.5, Tasks 1.1-1.4, and the complete Phase 1 specification-as-code foundation are COMPLETE / FOUNDER-ACCEPTED. The final accepted Task 1.3 implementation tip is `527e381661fe0cfc27e07ad9b44e1601a638ae75`; the accepted Task 1.4 implementation is `06c39429f3237f6974e21217670c3f0d30b0a571`. Phase 2 has not started. The exact next action is separate founder scoping and authorization of the first bounded Phase 2 task, beginning with the mock/localnet layer. This AI-assisted review is not a professional independent audit.
