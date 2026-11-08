# PIV1 hybrid distribution liquidity — Phase A

**Current overlay D108:** the founder explicitly selected `R=9500`, maximum TOTAL realized loss500bps against the exact official-book debit. The former OPEN question and STOP below are historical Phase A results, resolved by D108; local implementation/validation may proceed. No live deployment/main publication is authorized.

## D108 implementation and focused validation — 2026-11-08 UTC

**Local candidate PASS; NOT deployed.** Source index89505ba14469dcb7c125a70f15c24d02c7c0fa6e478df5e9d683a837962e8de5, ELF882256bytes/047c819b717bb0b74602596297a02bf5d84deaec327241a9981926fe144696ac. [Exact same-Program upgrade/migration proposal](PIV1_HYBRID_TESTNET_UPGRADE_PROPOSAL.md) is OPEN and requires separate founder approval. No new live scope is activated.

The actual new swap instruction is **PIV1JS01/version1**, not the Phase A draft's PIV1PS01 (which already names deployed pause). The independent review found/fixed that collision and a real top-level pause regression passes. `PIV1MR02/version1` permissionlessly extends only an unpaused Idle V1 round891→924bytes, preserves completed summary/Config and pays only exact current rent deficit from OperationalSOL. V1 encoding/allocation remains exact; V2 appends an optional four-u64 factual swap receipt. No Config/old instruction/authority/economic-field reinterpretation. Current decoders, settlement and pending custody recognize V2 explicitly; migration is not implicit.

The selected adapter is a single internally encoded current Jupiter `route_v2`, one Manifest JitoSOL→WSOL ExactIn hop. The40-byte ABI is authenticated from the current JUP6-owned IDL (e6c2b425…), not an outdated SDK/IDL guess. Caller supplies canonical accounts/market, never amount, minimum, slippage, fees or opaque instruction bytes. PIV derives largest safe exact-book input and ceil95% output, with zero router quote-relative slippage/platform/positive-slippage fees. It validates exact program/ProgramData pointers, canonical vault/global headers, Token ownership/mints/delegates/privileges and full receipts. Duplicate route metas make SwapAuthority signer AND writable; it is a distinct canonical PDA with exactly zero native balance. OperationalSOL, G0 and PivAuthority are never forwarded to Jupiter. Any route needing a funded rent payer or account growth refuses; no silent prefunding exception.

All three transient PDA prefunds normalize into recognized pending contribution before shortfall calculation. Two Token rents are advanced/recovered exactly; input is bounded, actual WSOL/native deltas must agree, full token bytes and every protected account remain checked after CPI. Swap output/spent units have distinct receipts/offsets, never fake stake burns or withdrawal legs. NET settlement caps outgoing allocations; compound/HWM/conversion dust, surplus, KIF liability/carry and full contribution are retained exactly. Failure commits no successful preparation/cadence; Solana runtime rollback is verified separately from host models.

| Evidence | Actual result |
|---|---|
| Arithmetic/domain |6PASS incl4096 seeded cases, exhaustive small oracle, ceil95%/one-below/combined loss/better output/thresholds |
| New host swap/codec/custody/dispatch |8PASS; modeled router/System effects, real host SPL transfers/closures; no runtime claim |
| Focused new V2 delayed threshold + corrected version rejection |2PASS |
| Relevant earlier liquid10/settlement13/integration10/delayed10 |43PASS reused after final selector/allocation corrections, unchanged relevant behavior |
| Current unsigned hybrid helper/decoder/source bindings |5PASS; historical old clients/pinsets not rebound/requalified |
| Authentic local Bank |1scenario PASS: migration/replay, real JUP→Manifest→Token, price refusal, complete late rollback, exact own event/NET payouts/KIF/HWM and full pending integration |

Measured composed swap1239162CU (160838/11.49% below1400000), packet348B with LOCAL v0 ALT,42locks,5589655loaded bytes, maximum CPI depth4; settlement787587CU/integration633723CU. Requested heap frame262144B is a budget; actual allocator peak is not exposed. PIV's existing default32KiB bump allocator remains unchanged; exact-capacity records/CPI views avoid repeated growth, and this complete path succeeds within it. Compiler final build has no stack-overflow diagnostics (4096B frame limit); actual per-frame stack peak is unmeasured. These are this route/fixture's bounds, not all-market guarantees or an ALT existing on Testnet.

E=`/home/jerem/piv1-pilot-checkpoints/hybrid-liquidity-phaseb-20261108-a/`: source index/build-e,8+1+1 final Rust logs/client-tests-final, bank-g command/raw log/metrics, and independent source/runtime review3b2758ec69c126c5686732c0d2001156d27155dde26ef43b6afd5c4fa55e7a57. Early stack128B overflow, initializer-fixture refusal before PIV CPI (underlying cause not investigated), combined bootstrap setup CU failure, real new-handler allocator exhaustion and missing local SBPFv3 capability are preserved, with distinct corrections/results; none is relabeled a successful test. No unchanged full test suite or new validation framework.

Authentic router payload53d53f44…/Manifest32eb0eef… and cloned canonical market accounts came from bounded PUBLIC READ-ONLY Mainnet captures (coherent466717391). Local PIV Program ID, official-pool economics, funding and Clock are synthetic; Manifest needs the explicitly activated local SBPFv3 capability. Source-to-binary reproducibility for external payloads is not established. This is authentic local execution, not a Mainnet transaction or Testnet quote/finality. Fresh public Testnet461887347 confirms JUP6 System/nonexecutable/empty and Manifest absent; a genuine public swap remains BLOCKED. No substitute deployment/fabricated yield is permitted.

Fresh proposed funding428093240lamports includes167640 migration rent; no transfer is authorized/performed. Active85b6ELF/15000045712HWM/11181318975Jito/native21/Idle/pending0 remain. Existing12785000fees/135000HOLD/370000reserve/1056640nonce and consumed scopes are preserved. Main/public readiness/M4 reservations are unchanged.

## HISTORICAL Phase A record (price decision subsequently resolved by D108)

**2026-11-08 Europe/Warsaw. CONFIRMED product direction D107; PROVISIONAL design; price-admission decision OPEN.** No implementation, new ELF or deployment is approved by this report. Baseline `86c24a1e95b72b48d358f8a81eee38705b03cac7` includes the founder's newly published concept-first documentation; main is preserved. Development branch: `codex/piv1-hybrid-liquidity`. Existing deployed800216-byte ELF remains `85b6fd0be8265d159c78b6cd1195a89c06d592856c8885547c24306233a98584`.

## Confirmed selector and preserved economics

Use the existing preparation calculation, not physical SOL balances:

```text
O = floor-based HTFP gross obligation + Team gross obligation + KIF gross obligation
pending_used = min(recognized eligible pending SOL, O)
carry_used = min(eligible prior-cycle yield, O - pending_used)
L = pending_used + carry_used
S = O - L

S == 0                     -> existing liquid path
0 < S < 1_500_000_000       -> Jupiter path
S >= 1_500_000_000          -> existing delayed Jito withdrawal
```

Equality selects delayed withdrawal. A large distribution with only0.2SOL missing selects Jupiter. Protected historical native SOL, rent, KIF liabilities and unrelated physical balances are not eligible funding. There is no new economic SOL buffer or automatic small-swap fallback to delayed withdrawal.

P006's DEX exclusion is superseded for this new small-shortfall route; direct staking and the large delayed path remain. P013 extends only the method used for missing liquidity. P014/full contribution recognition, P017/SOL HWM, P018/official Jito valuation, P019/no downward HWM reset, P020–P021/floors and retained dust, P022/external network fees, P024/atomic beneficiary payment and P026/dynamic delayed technical minimum remain. P023's outgoing-cost category is extended to actual swap execution loss; the59/19.5/19.5/2 split and protected compound are unchanged. The existing1bps Jito/SPL protection is not an approved DEX price parameter. Old decisions and deployed behavior remain historical evidence, not silently rewritten implementation.

## Current Jupiter research and validation boundary

Official [Swap API](https://developers.jup.ag/docs/swap) and [Router build guide](https://developers.jup.ag/docs/swap/build) select V2 `GET /swap/v2/build`, Metis raw instructions and custom/CPI composition. `/order` managed execution and legacy V1 recipes are not the selected path. API instruction/setup/cleanup/tip data remain untrusted; PIV1 must never execute the returned list wholesale.

The [current build schema](https://developers.jup.ag/docs/api-reference/swap/build) documents an input-token amount, destination choices, account limits and v0 lookup-table addresses. No ExactOut request control was found. One unsigned research probe returned HTTP200, `ExactIn`, router `JUP6LkbZbjS1jKKwapdHNy74zcZ3tLUZoi5QNyVTaV4`, discriminator `bb64facc31c4af14`,25 swap account metas and1lookup table. It exchanged a quoted1000000 Jito units for1309675 wrapped-SOL units in the API's supported market environment. This was not a Testnet quote, simulation or execution. No ExactOut guarantee or PIV CPI ABI compatibility is inferred. The probe's zero slippage setting and the API's default50bps are not founder policy. Keyless schema/overview documentation differs; actual keyless build and label requests both returned200 at this observation. No key was accessed or dependency installed.

The [official CPI example](https://github.com/jup-ag/sol-swap-cpi/blob/main/programs/swap-to-sol/src/lib.rs) identifies the same V6 router and demonstrates WSOL closure. It is a mechanism reference, not an acceptable PIV security implementation: its route-discriminator TODO and forwarding pattern must not be copied. The archived `jupiter-cpi` crate is not adopted as a new dependency. Exact route ABI, supported variants, approved AMMs/pools and their code provenance still need focused implementation review.

**Testnet read-only result, finalized461860990:** the actual API router address exists as a **System-owned, nonexecutable, empty-data account**,21000000lamports. It is not a deployed Jupiter router. Official JitoSOL mint `J1toso1uCk3RLmjorhTtrVwY9HJ7X8V9yYac6Y7kGCPn`, legacy Token WSOL mint `So11111111111111111111111111111111111111112` and the approved PIV ELF exist. A real Jupiter Testnet swap through this router is unavailable at that snapshot. Other undocumented deployments/liquidity are not presumed. No Mainnet RPC/signature/send was performed; API availability is not independently verified Mainnet binary provenance.

Mainnet market quotes and Testnet Jito book ratios must never be mixed to calculate actual costs or claim an eligible Testnet swap. After the price decision, local Bank tests can exercise isolated coherent fixtures with verified router/AMM artifacts and explicit synthetic PIV economics. Mocks prove only their stated host invariants; even real local CPI does not close public Testnet finality/liquidity evidence.

## Selected custody and lifecycle architecture

Use ONE new atomic `prepare_distribution_swap` instruction (proposed distinct selector `PIV1PS01`, version1). Its bounded route encoding supplies no authoritative yield, beneficiary, quote, input budget or clock. PIV1 authenticates current state, derives the exact shortfall and deterministically derives the largest safe input; the caller cannot choose a micro-input to consume the cadence. Unsupported or unrepresentable routes fail without opening a round.

1. Require unpaused Idle, genuine positive yield, ten-day cadence, current official Jito epoch/valuation, custody, current guardian snapshot and the small-shortfall selector.
2. Stage only the bounded historical JitoSOL input into a canonical transient Token account. Its owner is a dedicated canonical SwapAuthority PDA; the permanent PivAuthority and principal/pending custody are never exposed as Jupiter signer/source capabilities.
3. Use a distinct canonical legacy WSOL output account. OperationalSOL advances only current refundable technical rent. Normalize unsolicited prefunds with existing contribution rules; do not count them as swap proceeds.
4. Decode and restrict the router instruction/variant, exact amount, approved executable programs/pools, mints, output and privileges. Reject aliases, arbitrary downstream programs, unsupported shared/ledger/fee variants, platform/referral fees and tips. Do not forward protected state/custody/beneficiary accounts or the payer's signer privilege to Jupiter. An API label is not a program allowlist or code attestation.
5. Invoke only with the canonical SwapAuthority seeds and staged budget. Reload actual balances/authorities after CPI. Check exact input consumption, actual WSOL output, the independently enforced price floor, unchanged official Jito accounting and complete residual HWM/compound protection. Never trust a return value or quote as the SOL receipt.
6. Unwrap to the fixed escrow. Return exact separately recorded temporary rents to OperationalSOL; reconcile token closures, escrow delta and all economic accounts. The [Token native-mint rules](https://solana.com/docs/tokens/basics/sync-native) distinguish wrapped amount from rent. Refunds and rent are never beneficiary yield.
7. Only after all checks commit the preparation and `Idle -> EscrowFunded`. Separate existing atomic settlement then pays all destinations or none; integration recognizes the complete contribution value exactly once.

Failed CPI, bad price, missing route, wrong poststate or failed unwrap rolls the whole preparation back. The successful preparation timestamp/sequence advances only on successful funded preparation; no quote-dependent `SwapPending` state, new retry penalty or stranded active round is introduced. Once committed, the round owns native SOL and no longer needs Jupiter. Combining settlement with preparation in one transaction is optional and unproved, not required for safe preparation atomicity.

## Accounting and required source/layout changes

At the same authenticated unchanged official ratio, use exact conservative book debit, not just the separately rounded input value:

```text
D = book(U_before) - book(U_before - actual_spent_units)
require 0 < D <= S
require remaining protected assets >= old_HWM + compound + existing protected dust
conversion_dust = S - D
A = actual received native SOL, excluding all rents/prefunds
net_available = min(O, L + A)
realized_swap_loss = max(0, D - A)
```

Reuse existing relative net allocation `5900:1950:200`, per-beneficiary gross caps, KIF carry/zero-active rules and conservative settlement remainders. Keep permanent compound and split dust unchanged. Execution loss comes only from outgoing value, never principal, compound, pending contributions or rent. Favorable execution remains subject to the fixed outgoing cap and existing retained-value/HWM accounting; it is not an excuse to inflate frozen obligations or call actual market gains fictitious stake rewards.

The current source cannot truthfully encode swap funding: `DistributionFunding` has only Liquid/Withdrawal; `open_distribution`, `ActiveDistribution::validate`, `settle_distribution`, the settlement executor and `economic_custody_obligations` recognize only initial liquid funding and assigned/finalized stake withdrawals. Swap transfers do not burn JitoSOL supply and must not be relabeled as withdrawal burns, legs or finalized delegated SOL.

Proposed minimal persistent extension: explicit per-round funding mode and fouru64 facts (swap input units, exact book debit, actual native output, enforced minimum output), with corresponding events. Appending these to the891-byte header would require924bytes; this is a proposed layout, not compiled sizing evidence. Add an explicitly versioned round decoder/Idle-only migration/reallocation preserving the completed summary. Do not reuse Config's zero-only migration reserve or globally reinterpret all V1 account versions. Config's nominal historical units remain the opening position while the active round subtracts one distinct swap offset; integration retires that offset exactly once, as it already does assigned withdrawals.

Affected code after approval: new instruction/adapter and entry dispatch; preparation; distribution funding/validation/persistence; custody/reconciliation; settlement and pending integration; source-bound CLI; events/errors; targeted tests. Config economics, direct staking, old instruction identifiers, custody identities and delayed leg proofs remain unchanged. Migration and new temporary-account rent require an exact reviewed deployment/funding scope later, not execution now. At the observed Rent, two165-byte transient Token accounts would advance2976880lamports; actual rent/top-up needs fresh quotes and balances. No permanent economic reserve is proposed.

## Resource and test gates

Use pinned existing v0 tooling first: conservative packet budget1232bytes,64 transaction account locks, at most1400000CU, existing bounded heap up to262144bytes and conservative CPI depth5; do not assume new cluster features or migrate SDKs to make the design fit. [Solana transaction](https://solana.com/docs/core/transactions) and [CPI](https://solana.com/docs/core/cpi) limits distinguish packet/lock/stack limits. LUT compression does not remove locks or CPI privilege checks. Prefer a bounded supported route; its measured composed CU/heap/packet and full account set decide feasibility. The API's25-meta sample is not a composed PIV transaction measurement.

After the economic decision, reuse existing HOST/Bank infrastructure for threshold examples, pending/carry coverage, no-yield and unrepresentable dust, missing route, invalid/excess input/output, malicious programs/accounts, signer isolation, changed official ratio, rollback/replay, rents/prefunds, exact net allocation and real composed CPI resource measurements. Reuse unchanged evidence and run only necessary liquid-settlement/delayed regressions. No HOST program test, SBF run or new CU measurement has been performed for this design. Nine pure arithmetic selector checks and a mathematical adverse-price counterexample are design evidence only.

## One material decision required before Phase B

Input custody and HWM checks alone do not price-protect outgoing yield. In a **synthetic counterexample**, genuine yield1SOL gives0.805SOL outgoing and0.195SOL protected compound. A route can spend0.805SOL of book-valued JitoSOL, return1lamport, and leave principal plus compound intact. A manipulated genuine approved pool can capture804999999lamports of outgoing value without violating those custody checks. Nonzero output, an API quote or ExactOut with only an HWM input cap does not exclude this outcome.

**Recommended rule, OPEN:** the program, not the caller/API, must enforce `A >= ceil(D * R / 10000)` for a founder-selected minimum retained-value parameter `R`. This includes all actual DEX fees, price impact and slippage in one measurable book-relative loss bound. The value of `R` is **not selected or approved**. The current1bps Jito guard and API defaults do not supply it. An additional oracle would introduce further policy/trust/dependencies rather than resolve an absent acceptance rule automatically. Even an approved bound permits bounded adverse execution; it is not a promise of MEV-free best execution.

Bind any later approved value into the reviewed program implementation and freeze the resulting exact minimum output in each successful round's receipt. No new policy account, caller override or administrative pricing power is proposed. Changing that value later would require its own explicit economic decision and governed code change.

A policy-free restrictive alternative is full book-debit coverage (zero realized swap loss), allowing fees only when offset by a market premium. It does not implement ordinary flexible-loss distributions and is not silently substituted for the request. The founder must specify the permitted loss bound or explicitly select that restrictive alternative. No safe flexible-loss execution is claimed until that choice is recorded.

**STOP at Phase A as required by the directive's sections3/8.** No production code, upgrade candidate, new ELF, Buffer, migration or deployment scope is prepared. Active ELF85b6fd0b and all completed/consumed scopes,135000HOLD,370000reserve,1056640nonce rent and12785000confirmedfees remain intact. A later exact same-Program/Squads4of6 upgrade needs separate explicit approval; main publication is not authorized here.

Evidence: `/home/jerem/piv1-pilot-checkpoints/hybrid-liquidity-phasea-20261108-a/` contains the exact directive/approval baseline, bounded official-source fetches, unexecuted API response, Testnet account snapshot, availability summary and design-only checks. Available HTFP Solana Dev/CPI Safety references were consulted; `HTFP_MASTER_CONTEXT.md` was not located locally or in the connected scoped GitHub filename search, and no contents are inferred. This is design review, not a professional audit or real swap proof.
