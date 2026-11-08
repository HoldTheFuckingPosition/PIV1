# D111 bounded same-round recovery — local implementation

**PROVISIONAL / TESTNET ONLY. Local validation complete; deployment NOT authorized.**
D111 approves implementation of the unchanged [historical proposal](PIV1_RECOVERY_REQUIRED_EXIT_PROPOSAL.md), not a new public-chain operation or Mainnet economics. The optional founder-supplied global HTFP context stays outside Git; later exact PIV1 decisions control. Baseline d5ecbe56c3be2e507c9fc7c3e212cde64d5e99b6, branch `codex/piv1-hybrid-liquidity`.

## State, accounting and instructions

V1 remains891B; V2 remains924B. V3 is **1199B**: the complete padded924B V2 prefix (only version becomes3), followed by optional136B active receipt and optional137B completed receipt, each with a one-byte tag. Unused bytes must be zero. Streaming serialization and a fixed768B snapshot-hash buffer avoid repeated bump-allocator temporaries. Maximum codec tests measure these sizes. Config, guardian, custody and leg layouts are unchanged.

| Selector | Bytes | Exact action |
| --- | ---: | --- |
| `PIV1RC01` |17| Version1 + leg index u64. Existing finalizer topology23/24 accounts. Collect an already initiated inactive canonical leg; stay RecoveryRequired. |
| `PIV1RR01` |103| Version1 +94B `ResumeRecovery` Borsh request. Exact current Squads4-of-6 action, no transfer. |
| `PIV1MR03` |50| Version1 + vault u8 + SHA256 of the complete old924B round + external rent cap u64. Governed V2 Idle/Recovery→V3 only; never resume. |

RR01 request order: vault u8, sequence u64, generation u64, frozen hash32, cumulative loss u64, finalized count u64, successful count u64, assigned units u64, economic escrow u64, destination u8(0=EscrowFunded/1=WithdrawalActive), validator index u32. Runtime Program/round/all account keys and privileges are bound by the actual stored Squads message. EF requires validator index0. Native endianness is never used: all integers are Borsh little-endian.

RR01 retains the fixed custody/Jito prefix0..17 (separate referrer adds18); base is18/19. Then Clock, Program, ProgramData, multisig, proposal, transaction, Vault, Instructions, current registry, six CURRENT rewards; partial continuation adds Stake program and canonical current source. Thus EF33/34 accounts, WA35/36. Only round1 is writable; Vault at base+6 is signer. MR03 uses governance0..15 (Config7), round16, pendingSOL17, principalSOL18, Operational19, escrow20, KIF21, principalJito22, pendingJito23, System24, external signer-payer25. Only16/25 writable;5/25 signers. Existing guardian wallets may pay operational rent; their earned KIF custody/records are separate and untouched. Internal custody/protocol/recipient aliases are forbidden.

Full initial source-account authentication and actual current4-of-6 proof remain. Fresh exact account metadata and body checks preserve the same proof within ONE invocation. Only actual runtime-readonly bodies may reuse their initial hash after CPI; HOST always rehashes, writable bodies always rehash, and every metadata/length/lamport check stays fresh. This matches existing settlement/integration practice; no historical/off-chain proof exception or cache across operations is introduced.

Actual cumulative cooldown loss is deducted **once**: `eligible=P+carryUsed+finalizedDelegated−loss+swapOutput`. Existing capped5900/1950/200 relative NET floors apply. Recovery requires loss≤frozen outgoing and positive SUM of floored payments. Rewards and rent remain separate. No gross edit, HWM decrease, principal release, pending-as-repair or old-KIF haircut exists. EF proves prospective settlement protection including pending replacement; WA additionally proves all current legs collected, positive collected NET, feasible current canonical source and full-target residual≥proposedHWM+pendingUsed. The latter bound is rechecked at subsequent actual initiation.

New committed incidents increment generation even at unchanged numeric loss. Factual flags/loss and exact approving multisig/index/action survive in the completed V3 receipt through Idle. New rounds inherit history, never authority. Collection/resume events expose snapshot/generation, loss/counters/escrow, rent/rewards, Clock and branch-specific value/floor. Migration changes no economic right and initializes a legacy incident unresolved. Default builds cannot dispatch the three new operations; the explicit `testnet-bounded-recovery` feature is required.

## Evidence and practical limits

Evidence root `E=/home/jerem/piv1-pilot-checkpoints/recovery-implementation-20261108-a/` is preserved. Source index SHA256 **ee4e06070e0099f8f0d6497ba3521c3c24e4bea95259b15a53f2253af4b65b46**. Final stripped ELF **952664B**, SHA256 **fd53a813aaa2153411539039eab75f47af1d07b5d9a30de49ad3a2524b5103aa**, `E/recovery-candidate-final.stripped.so`. Features: existing initial-rounding plus bounded-recovery. Final rebuild is byte-identical to the artifact used by the retained Bank qualification; no repeat qualification needed.

PASS: five new state/codec tests, ten focused handler tests (one exports the bounded fixture), default-feature refusal, V3 JavaScript decoder, legacy codec checks and51 affected HOST regressions. Finalizer's10 relevant mutation/CPI regressions passed again after its actual hash-path change. Three approved accounting examples A/B/C execute production handlers: A/B exact NET/old600KIF preservation; C remains blocked. Multiple legs, immature/pause/replay/closed-metadata refusals, exact/stale/insufficient governance, current-source mutation and injected post-CPI readonly governance mutation are covered. HOST callback failures are failed worlds, not runtime rollback proof.

Actual local Bank with **730009B** validator list and authentic Stake/Token ELFs, `E/bank-h.log`:

| Successful operation | Whole transaction CU |
| --- | ---: |
| Governed Recovery V2→V3 migration |829445|
| Existing-leg recovery collection |707221|
| Exact governed EF resumption |1248658|
| Loss-aware settlement |756883|
| Full pending integration→Idle |657050|

The actual Stake program refuses the immature leg; deliberate later-instruction failure rolls back migration/rent/payer; stale resume/integration replay fail. Exact old padded prefix, external rent debit, NET beneficiaries, KIF credits/carry, full contribution/HWM, JitoSOL and completed loss77 are asserted independently. Max recovery cost leaves about151k CU below1.4m. Existing authentic Jupiter/Manifest local regression also PASS (`E/hybrid-bank.log`), premium1254414CU; loss-floor refusal and atomic rollback preserved. No public Jupiter availability or public recovery execution is inferred.

Local governance/market/Clock/StakeHistory/ALT fixtures are explicitly synthetic; the caller is a bounded test proxy, not real Squads vote execution. WA is covered by production HOST handlers, not a complete WA Bank lifecycle. Requested262144B heap frame is NOT measured heap peak; PIV's default32KiB bump allocator remains unchanged, and the final successful SBF run has no allocation failure or compiler stack overflow. Earlier build/Bank failures remain in E; none is a PASS. M4 OPEN_UNWAIVED/public M5 NOT_READY remain.

Future public tests remain separate: after exact installation approval, the current Idle pilot can demonstrate only governed V2→V3 migration and unchanged protected state. Public recovery collection requires a genuinely initiated inactive leg in a real RecoveryRequired round; full/partial resumption, incident recurrence and loss-aware completion require the corresponding authenticated incident, current solvency and actual eligible lifecycle/yield/cadence. Idle is not evidence of those paths. Do not manufacture loss/yield, alter public Clock/HWM, or infer router availability to create eligibility.

**STOP before live actions.** Zero RPC/private-key access/signatures/sends/funding/Buffer/deployment/migration in this task. Deployed D109 ELF/V2 state and all consumed originals/HOLDs remain unchanged. Fees17480000, historical HOLD135000, separate reserve370000 and nonce rent1056640 unchanged. [One future upgrade proposal](PIV1_RECOVERY_V3_TESTNET_UPGRADE_PROPOSAL.md) requires separate explicit approval. Historical launchers/source pins were not rebound or re-enabled.
