# D108 hybrid liquidity: one Testnet upgrade and Idle migration proposal

**OPEN / PROVISIONAL / SOLANA TESTNET ONLY. This document grants no execution authority.** D108 authorizes the local implementation and validation, and explicitly requires separate approval before live Buffer creation, funding, migration or upgrade. D105 and all prior scopes remain consumed. No new confirmed decision ID is assigned to this proposal.

## Exact candidate and validation

Production source index SHA256 **89505ba14469dcb7c125a70f15c24d02c7c0fa6e478df5e9d683a837962e8de5**; canonical sorted source/hash index and compiler command are retained under `E=/home/jerem/piv1-pilot-checkpoints/hybrid-liquidity-phaseb-20261108-a/`. Stripped ELF `E/hybrid-candidate-e.stripped.so`: **882256 bytes**, SHA256 **047c819b717bb0b74602596297a02bf5d84deaec327241a9981926fe144696ac**. Existing pinned compiler/dependencies and initial-only Testnet rounding feature are preserved; no new staking exception.

The candidate implements the fixed combined500bps loss limit against exact official-book debit, staged isolated custody and internal restricted Jupiter/Manifest route, factual V2 swap receipt, existing NET settlement/contribution integration, and Idle-only `PIV1MR02` migration. New swap selector is **PIV1JS01/version1**; deployed **PIV1PS01 remains pause**. Round V1 is891bytes unchanged; V2 is924bytes. Config, custody identities, HWM, percentages, cadence and genuine six-guardian/4-of-6 governance remain.

Focused arithmetic/host/client tests and authentic local Bank composition PASS. The measured swap is1239162CU,42locks,348-byte v0 packet using a synthetic local ALT,5589655loaded bytes, depth4; settlement787587CU and integration633723CU. Successful swap plus later failure proves complete fee-only rollback; adverse price produces actual Jupiter6001 after Manifest CPI and no state effects. Exact PIV event, NET recipient/KIF/HWM and full contribution assertions PASS. The local PIV ID/economics/Clock are synthetic; authenticated public Jupiter/Manifest payloads are loaded under canonical external IDs, with their required SBPFv3 capability. Requested frame262144bytes is a budget, not measured peak; PIV's default32KiB bump allocator is unchanged. No public Testnet swap/finality or general all-route resource guarantee is claimed. Independent review `E/phaseb-source-runtime-review.public.json`, SHA256 **3b2758ec69c126c5686732c0d2001156d27155dde26ef43b6afd5c4fa55e7a57**.

## What public Testnet can demonstrate

Fresh coherent finalized **461887347** confirms Jupiter JUP6 remains System-owned/nonexecutable/empty (21000000lamports), and canonical Manifest is absent. Therefore this proposal CANNOT produce a genuine public Testnet Jupiter swap with current public deployments. No substitute router, AMM deployment, Mainnet trade or fake yield is authorized. An approved upgrade could verify the SAME Program's new bytes, migration, preservation of state, and existing eligible liquid/delayed paths; public small-shortfall execution remains BLOCKED until genuine supported router/AMM/liquidity and packet accounts are independently available. A future public swap scope would need its exact usable market/LUT/funding and fresh eligibility; this proposal creates no LUT or transient swap accounts and starts no distribution.

## Exact identities and finite sequence requested

Testnet genesis `4uhcVJyU9pJkvQyS88uRDiswHXSCkY3zQawwpjk2NsNY`.

| Role | Identity |
|---|---|
| SAME Program | `GAjHRaC3XM98SoRoFy5B8P2ypTS1ndf1UPhXmjWgtPux` |
| SAME ProgramData | `6HV5GQesbZpTyBVTiMu64UU4JgCX2D2s489sKt26xpd8` |
| Existing G0 | `BUuuNvicRKF4ZFEUH4iS8xGzhAKQca3ou4U1s4NaBYoT` |
| Existing Source | `4WKQg3Sm8bvHS8DBmxoiMMi4Fev4mJU6ZLwGSTg6jDna` |
| Existing Squads multisig | `9mwto7RNokYc8n8MPtzHpubwwnKcborY2r9RnfZAScVS` |
| Unchanged Vault0 authority | `A2UHpGyiCBdUZDMrofGswNiyS4ATD1ytBSiAsWNdxUTw` |
| ONE proposed deterministic Buffer, absent at opening | `49rXYA8MJe3MYcyK3kKWoCmHd3m62wnRWHChkURdWrLg` |
| Existing OperationalSOL | `7df3DSrQvMjEjLgTDFGuCqefdDzKAsfjpWwzRzanpvJg` |
| Existing round, migrated in place | `23WaDJj9SEXtdhBeeqCMfX3LTYvciuXWGCVYNd7jdmoP` |

After explicit approval of this exact package only:

1. Refresh balances/reservations, exact active ProgramData/Vault0, Idle/unpaused state, candidate/source, Buffer absence, multisig membership/threshold/index and absence of competing/unresolved same-intent writers. Opening multisig index5 suggests fresh index6, never reuse an old proposal.
2. ONE Source→G0 transfer only if needed, for actual shortfall up to428093240lamports, fee<=10000 paid by Source. Preserve Source's known259000005000-lamport floor and every other reservation; verify sufficiency and no competing funding. Classify EXTERNAL OPERATIONAL FUNDING only; deduct new external credits and never double-fund. No private-key creation/export/copy.
3. ONE G0→existing OperationalSOL transfer, at most167640lamports, only the fresh migration-rent deficit. No principal/pending/escrow/KIF funding. Create ONE Loader Buffer via G0 `createAccountWithSeed`, seed `d108-hybrid-v1`, space882293, owner `BPFLoaderUpgradeab1e11111111111111111111111`; no new wallet. Upload exactly923 Writes, max956bytes, independently bounded batches<=100, fresh originals and full final byte readback.
4. Transfer ONLY this Buffer to existing Vault0; seven genuine Squads operations (vault transaction, proposal, four guardian approvals, execute) perform ONE upgrade of the SAME Program. Capacity892352bytes fits: no extension. Verify actual finalized ELF and remaining10096zero bytes, unchanged ProgramData/Vault0/guardians and complete economic state; Buffer rent returns only to existing G0 on actual successful Loader closure.
5. ONE fresh permissionless `PIV1MR02` on the existing Idle V1 round. Verify V2/924bytes, exact167640 migration rent at current quote, complete Config and completed-summary preservation, all custody/HWM/authorities unchanged and OperationalSOL floor preserved. STOP at this boundary; no swap, preparation, settlement, deposit or stake is included.

Use existing D101 clients and signing infrastructure, ordinary fresh recent blockhash, immediate signing/standard preflight/one send, durable exact originals/receipts, finalized reconciliation before dependency, at-most-once and UNKNOWN/HOLD. No blind retry/replacement/adoption. Maximum934 G0-paid sends and937 native/Ed25519 signatures for those transactions (three additional guardian signatures); Source adds at most one signature/send, for aggregate maximum935 sends/938 signatures. One activation-fixed window<=2hours, no restart extension. At most one writer, existing pinned toolchains, <=1GiB additional local artifacts with >=2GiB free plus build headroom. Stop on actual failure, ambiguity, unexpected account/economic effect, missing signatures or exceeded finite limits; preserve every original/HOLD and remaining reservations.

## Exact proposed funding and ceilings

Read-only snapshot461887347: G0 **4068780720**, Source **335425654887**, OperationalSOL **650240** (zero economic spendable). Active800216-byte ELF85b6fd0b is unchanged. Round891-byte account holds5176520; new924-byte rent5344160 requires167640 from OperationalSOL. Current quotes and a new unsigned standard single-signature fee quote5000 are retained in `E/deployment-{accounts,quote,budget}.public.json`; first quote reader's enum assertion failure is preserved and corrected without re-observing historical chain state. These quotes are proposal inputs, not execution guarantees; refresh before any authorized effect.

| Upfront G0 requirement | Lamports |
|---|---:|
| Buffer rent882293bytes, refundable after successful upgrade | 4482698680 |
| Squads VaultTransaction378bytes + Proposal646bytes, locked rent | 6502400 |
| Migration OperationalSOL top-up, locked in enlarged round | 167640 |
| New G0 fee ceiling | 4700000 |
| Preserved G0 retained floor | 2805240 |
| Total required G0 | **4496873960** |
| Current G0 | 4068780720 |
| Proposed maximum fresh Source shortfall | **428093240** |

Expected G0 fees at5000/signature: **4685000**, within4700000; per-transaction fees<=5000 with one signer or<=10000 with two. Source fee<=10000 separately. Request cumulative operational funding cap **3534482160** (previous3106388920 +428093240), and mission confirmed-fee ceiling **17495000** (=12785000 confirmed +4700000 new G0 cap +10000 Source cap). No old unspent envelope or consumed funding right authorizes these increases. Preserve **135000UNKNOWN/HOLD**, separate **370000reserve**, **1056640nonce rent**, all uncertain signatures and the protected **15000045712HWM**, principal11181318975Jito units/native21, pending0 and completed round. No economic-rule/authority change, Mainnet, main publication, cleanup or automatic lifecycle execution.
