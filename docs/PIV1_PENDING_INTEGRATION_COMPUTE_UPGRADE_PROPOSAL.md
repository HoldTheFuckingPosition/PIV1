# One pending-integration compute correction: Testnet upgrade proposal

**OPEN / PROVISIONAL / SOLANA TESTNET ONLY — explicit founder approval required.**

This proposal authorizes nothing by itself. D102's exact settlement ELF upgrade and D103/D104 funding sends are consumed. No new confirmed decision ID is assigned. The requested action is ONE different, exact-candidate upgrade of the SAME Program, followed by ONE fresh integration of the already settled round. Preparation, settlement, the original deposit and initial stake must not be repeated.

## Exact candidate and safety

- Source: `programs/piv1/src/pending_integration_execution.rs`, SHA256 `83eae5669b49c5c5583bfdd64942ceb00302e4ecc54f5a65b261eafd48d519dd`.
- Stripped ELF: **800216 bytes**, SHA256 `85b6fd0be8265d159c78b6cd1195a89c06d592856c8885547c24306233a98584`.
- Existing `testnet-initial-principal-rounding` build feature is unchanged. Its historical initial-only exception is not extended to another stake.
- Only runtime-readonly post-CPI body hashes are reused. Initial complete authentication/hashing, all current metadata/identity/owner/privilege/lamport/length/borrow checks, writable body hashes, host mutation detection, exact System/Token CPI receipts, current Jito valuation, HWM, atomicity and replay protection remain.
- No ABI/layout/economic/account/recipient/authority change. The settlement correction remains unchanged.

Candidate and compact local evidence: `/home/jerem/piv1-pilot-checkpoints/m5-practical-functional-tests-20261107-a/pending-integration-correction/`. The canonical pilot checkpoint records the final test measurements and independent review. Source review and local SBF success are preparation evidence, not authorization or live finality.

**Local validation PASS:** two changed Record tests and ten focused integration host tests. Captured finalized Settled state at461510651 executes the complete integration in **594108/1400000 CU**, leaving805892CU (**57.6%**) headroom. Actual local System transfers are4999811306 and1; HWM15000045712/JitoSOL7454364674/Idle; incorrect custody6017, replay6034 and actual late-outer-failure Bank rollback pass. Necessary captured-settlement regression against this same ELF also passes **694052CU**, with unchanged actual NET payouts. No whole suite, historical qualification, live simulation or chain transaction was repeated.

Evidence: `bank-integration-c.public.json`, `bank-settlement-regression.public.json`, `host-record-tests.public.json`, `host-integration-tests.public.json`, `sbf-build.public.json`, `source-review.public.json`. Local Bank uses the existing pinned synthetic loaders, unsigned local payer, captured economic Clock/state and explicit prior RPC-rounded rentEpochMAX normalization. It proves this captured runtime path, not public-chain finality or a guarantee for later changed state. One initial compilation attempt and a test-fixture absence assertion failure are retained; the fixture correction preserves the unfunded readonly authority PDA rather than funding it. No program change followed the successful ELF build.

## Exact identities and proposed finite operations

Cluster Testnet, genesis `4uhcVJyU9pJkvQyS88uRDiswHXSCkY3zQawwpjk2NsNY`.

| Role | Existing or proposed identity |
|---|---|
| SAME Program | `GAjHRaC3XM98SoRoFy5B8P2ypTS1ndf1UPhXmjWgtPux` |
| SAME ProgramData | `6HV5GQesbZpTyBVTiMu64UU4JgCX2D2s489sKt26xpd8` |
| Existing G0 | `BUuuNvicRKF4ZFEUH4iS8xGzhAKQca3ou4U1s4NaBYoT` |
| Existing Source | `4WKQg3Sm8bvHS8DBmxoiMMi4Fev4mJU6ZLwGSTg6jDna` |
| Existing Squads multisig | `9mwto7RNokYc8n8MPtzHpubwwnKcborY2r9RnfZAScVS` |
| Unchanged Vault0 upgrade authority | `A2UHpGyiCBdUZDMrofGswNiyS4ATD1ytBSiAsWNdxUTw` |
| ONE new deterministic Buffer, presently absent | `2UcoRXfknn7Wnmxg92WkQ6CugFvpfYrzwSrKVHUMLY5r` |

Buffer uses existing G0 `createAccountWithSeed`, seed `d101-integration-cu-v1`, owner `BPFLoaderUpgradeab1e11111111111111111111111`. No new key or wallet. After explicit approval only:

1. Refresh Testnet/genesis, exact wallet balances/known reservations, current Program/ProgramData/Vault0, candidate, new Buffer absence and Squads index/new PDAs; ensure no competing writer or unresolved same intent.
2. If needed, ONE fresh Source→G0 transfer of verified shortfall, at most **20216520 lamports**, using existing signing infrastructure; one signature/send, Source fee at most10000. No double funding; deduct any newly verified external credit first. Record EXTERNAL OPERATIONAL FUNDING only.
3. Create ONE Buffer, make **838 canonical Writes** of at most956 bytes using fresh originals, and fully verify candidate bytes and current G0 authority. Batches at most100; no new launcher or historical qualification campaign.
4. Transfer ONLY this Buffer authority to existing Vault0. Create the next distinct Squads transaction/proposal (index5 if current index4 remains unchanged), obtain four actual signatures from the existing six guardians, and execute ONE same-Program Loader upgrade. No ProgramData extension. Verify actual finalized ProgramData bytes,92136-byte zero tail, unchanged authority/custody/economics, and Loader return of Buffer rent to G0.
5. Construct ONE fresh `PIV1IP01` only after that verification. Standard preflight, one send, finalized receipt and exact economic reconciliation. Never adopt/re-sign/resend the consumed refused integration original.

G0 execution maximum **848 sends /851 native calls /851 Ed25519 signatures**, including Buffer create,838Writes,eight governed operations and one integration. Source funding adds at most1send/1native/1Ed separately. No automatic replacement/failed-original retry. A new window of at most two hours is recorded at activation; transactions use ordinary fresh recent blockhashes and immediate signing/preflight/submission under D101, without artificial qualification delays.

## Financial request and current quote

Read-only finalized quote at slot461516739: G0 **4059326600**, Source **326245881407**. Existing deployed ELF is9e07c307, not this candidate. ProgramData892397 bytes has892352 bytes of ELF capacity;800216 fits without extension. New Buffer is absent.

| Required resource | Lamports |
|---|---:|
| New Buffer rent,800253 bytes | 4065935480 |
| New Squads VaultTransaction rent,378 bytes | 2570480 |
| New six-member Proposal rent,646 bytes | 3931920 |
| New G0-paid fee ceiling | 4300000 |
| Existing G0 retained floor | 2805240 |
| Required G0 before create | **4079543120** |
| Current G0 | 4059326600 |
| Verified additional funding required | **20216520** |

Fresh quotes/sufficiency remain mandatory. Source fee at most10000 is separately funded/paid by Source; Source known retained floor259000005000 and existing reservations remain. No PIV principal/pending/escrow funds may finance this work. Buffer rent returns only after successful upgrade;6502400Squads rent remains locked. Both rents count in upfront funding; none is classified as a fee, income or yield.

Request approval of this **new** additional-funding right and cumulative total up to **3106388920 lamports** (previous3086172400 +20216520). This explicitly exceeds the old3.1SOL ceiling by6388920; old funding rights remain consumed. No unapproved Source send or presumed unused old cap is used.

Expected new G0 fees at unchanged5000/signature:4255000; bounded maximum4300000, each transaction at most10000. Request a new **mission confirmed-fee ceiling12830000** (=8520000 already confirmed +4300000 new G0 ceiling +10000 new Source ceiling), preserving **135000 UNKNOWN/HOLD**, separate **370000 reserve** and **1056640 existing nonce rent**. This is a requested ceiling, not an incurred fee or confirmed decision. Current10m ceiling and consumed D102 package do not authorize this candidate.

## Functional completion and stop rules

For the retained eligible Settled state: pending_SOL→principal **4999811306**, escrow→principal **1** lamport; full recognized **5000000000** contribution increases HWM once from10000045712 to **15000045712**. Principal JitoSOL stays **7454364674 units**; no beneficiary payment or JitoSOL withdrawal; round becomes Idle. Fresh eligibility is rechecked, never hardcoded over changed state.

Stop on insufficient funds, altered identity/authority/candidate/custody, failed preflight/transaction, unexpected economics, resource exhaustion or ambiguous outcome. Preserve original/message/receipt and all earlier HOLD/consumed evidence. For uncertainty, observe only the original signature; never blindly resend, replace, cancel or adopt it. Pausing does not renew consumed rights.

After finalized reconciled integration, the next separate D101 functional scope is staking eligible integrated principal using fresh current Jito state and unchanged normal principal-protection rules. This proposal does not extend initial rounding exceptions, authorize another distribution/upgrade, repeat previous results, change governance, or permit Mainnet, cleanup, new wallets or authority removal. Integration publication only; `main` remains unchanged.
