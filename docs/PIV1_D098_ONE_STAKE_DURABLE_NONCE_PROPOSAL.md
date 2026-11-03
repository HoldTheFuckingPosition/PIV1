# PIV1 — One initial Testnet stake with the existing durable nonce

**OPEN proposal — PROVISIONAL / TESTNET ONLY. No nonce-staking authorization or executable scope exists.**

## Exact founder decision requested

Approve ONLY one initial **10000000000-lamport SOL → JitoSOL** transaction on Solana Testnet, using the existing PIV1 Program, G0 and nonce account identified below. Authorize these explicit differences from ordinary-recenthash D-098:

1. Put native System `AdvanceNonceAccount` first, followed by the existing two compute instructions and unchanged staking instruction ABI, roles and amount. Bind a distinct original to the nonce's exact verified current value. Do not inherit or reopen a D-095 governance scope.
2. Replace ONLY the recent-blockhash validity/last-valid-height/40-remaining-heights predicates with fresh, exact nonce-account/value/authority/original-binding predicates at every corresponding boundary. Keep the ordinary **1.000-second response age**, **0.900-second qualification target**, **20-second native deadline**, current-state checks and all source/history/WAL/financial controls.
3. Keep D-098's five fully checked simulated economic poststates. Explicitly allow the newly writable nonce's simulated poststate to be inferred from its exact current state, native Advance instruction and successful simulation, then observed in full after actual execution. The provider's five-account limit prevents adding it as a sixth return. This is a NEW, narrowly scoped pre-effect observation exception, in addition to D-098's three omitted SOL poststates; it is not silently covered by D-098.

Accept that an already signed nonce transaction may remain executable **after the off-chain scope deadline** until the nonce is consumed/changed. Accept that a later staking error can still consume the nonce and charge the transaction fee. No automatic cancellation, standalone nonce advancement, replacement, rebroadcast or second stake is authorized.

Policy approval would permit implementation, focused testing/review, ONE distinct zero-effect qualification and, only after complete PASS and independent live GO, ONE distinct live execution scope under D-087. It would not itself activate signing. Scope limits: **1 native invocation / 1 Ed25519 slot / 1 controlled send / ≤10000 lamports fees / 0 new rent / 0 funding / ≤300 seconds**, fixed once at activation without renewal. No other staking, economic, governance or Mainnet operation is included.

## Confirmed starting point and engineering choice

Source/publication baseline: `d9eb0472298a4fbd1ac813792543259889260c55`, `integration/piv1-testnet`, 2026-11-03. All836 Loader Writes, nonce bootstrap and eight governed upgrade operations are FINALIZED. The same Program runs the approved798544-byte ELF. Initial real staking has NOT started. The D098-C reader, seven-category retained-response regression, real noncommitting five-account simulation and complete before-sign admission PASS are reused.

**Path A: NOT DEMONSTRATED, not impossible.** C's complete before-sign admission was0.288500871s. Its original had height411475877, last valid411476027 and before-sign height411475903, leaving124heights. Before-send returned height411476062 and invalid blockhash:35expired/75short of the required40. The20.000161199s native-envelope and chosen11s VERIFY stress were reservations, not actual signing/verification measurements. The checked VERIFY-stress interval was12.091971842s inclusive; its audits overlap other timings. Final complete admission and whole-original budget did not pass. The failure's35.795257659s sample age refers to the previous before-sign sample, not new RPC latency. No seconds-to-guaranteed-heights conversion or shorter reservation establishes a viable conventional budget; actual native/VERIFY durations remain unknown. Another conventional qualification is not recommended.

**Path B: structurally compatible, execution qualification OPEN.** One bounded offline projection used the installed pinned web3.js1.98.4/Node24.19.0 and C's explicitly unsigned CAPS0 packet. Kernel networking was denied; no keypair, signer, RPC, signed-wire verification or old signed original was used. The real SDK produced878message bytes /943unsigned-wire bytes /24keys /header[1,0,15] /oneG0 signature slot, within the1232-byte legacy limit. Every pre-existing global account privilege and trailing instruction remained identical. Observed host wall time0.158532224s; SDK load0.084021089s, unsigned decode0.003847985s, compile/serialize0.021958546s and round-trip/role checks0.001212778s. These measure structural host work only and supply no actual signer/VERIFY budget.

The nonce eliminates this particular recent-blockhash expiry constraint; it does not prove simulation, complete admissions, resource lifetime, signing or finality. Official documentation still describes native nonce execution and warns of possible future deprecation; successful retained D-095 operations and the fresh observation below support this installed route, without guaranteeing future runtime availability. [Solana durable nonces](https://solana.com/docs/core/transactions/durable-nonces).

## Exact identities and fresh public observation

| Role | Existing identity |
|---|---|
| Testnet genesis | `4uhcVJyU9pJkvQyS88uRDiswHXSCkY3zQawwpjk2NsNY` |
| PIV1 Program | `GAjHRaC3XM98SoRoFy5B8P2ypTS1ndf1UPhXmjWgtPux` |
| ProgramData | `6HV5GQesbZpTyBVTiMu64UU4JgCX2D2s489sKt26xpd8` |
| Preserved Squads/Vault0 upgrade authority | `A2UHpGyiCBdUZDMrofGswNiyS4ATD1ytBSiAsWNdxUTw` |
| G0 fee payer and nonce authority | `BUuuNvicRKF4ZFEUH4iS8xGzhAKQca3ou4U1s4NaBYoT` |
| Existing nonce | `5fnA8B9cCfCfMrzoZqZAPLZt7CM3gy6F2mh4LxKspeSd` |
| Expected current nonce value | `3bQDWu8bLzkXTtJWbjoRLCSYHJNjDfGnMyV7PMck412V` |
| Official Jito Testnet pool | `Jito4APyf642JPZPx3hGc6WWJ8zPKtRbRs4P815Awbb` |
| JitoSOL mint | `J1toso1uCk3RLmjorhTtrVwY9HJ7X8V9yYac6Y7kGCPn` |

ONE read-only HTTP batch used the unchanged pinned native-TLS public transport and same Testnet provider: genesis, version, one five-account finalized snapshot, nonce signature history(limit16), and fee quote for the exact unsigned projected message. Context/Clock/fee slot460096322, ClockUnix1793712539; RPC4.4.0/feature-set3722100051. Transport elapsed0.914093629s is NOT complete admission timing. A local caller syntax error was corrected before creating the read scope or making any RPC; the HTTP request was not retried.

Nonce: System-owned/nonexecutable,80bytes, version1/Initialized1, G0 authority,1056640lamports,5000lamports/signature; full data SHA `e9a9ae583c854284e48ea7c62c9e766ee435f5c069deb9d4f3ad2389b2c62248`. It exactly matches the independently finalized operation846 successor. Rent is5080lamports/byte-year ×1.0 threshold, including128-byte overhead; RecentBlockhashes is a fresh native Sysvar with150entries/6008bytes. The projected message received a5000-lamport fee quote at the same context.

All nine visible nonce signatures are finalized without error: the known bootstrap and eight governed originals. Retained closure and the unchanged controlled signing ledger contain no unresolved signed original for this current nonce value; later A/B/C staking qualifiers had zero signing rights/effects. No competing writer was observed. This is bounded local/visible-history evidence, NOT proof of global absence of copies or hidden external signatures. Fresh exclusive-use checks remain mandatory. An unexpected nonce successor, outstanding right/original or competing writer causes HOLD, not silent baseline adoption. No new nonce is justified by the evidence; this proposal authorizes no account creation, key generation, funding or replacement nonce.

## Exact transaction and implementation delta

The projection is a legacy transaction, without address lookup tables. Original size804/869bytes and22keys becomes878/943bytes and24keys. Sole signer remains G0; no nonce-account private key is needed. The two added global keys are writable nonce and readonly `SysvarRecentB1ockHashes11111111111111111111`. G0's existing fee-payer writability is unchanged.

| Index | Instruction | Required meaning |
|---|---|---|
| 0 | Native System AdvanceNonceAccount | Exact data `04000000`; operands nonce(writable), RecentBlockhashes(readonly), G0(authority signer) |
| 1 | Existing compute limit | Exact data `02c05c1500`,1400000CU |
| 2 | Existing heap request | Exact data `0100000400`,262144bytes |
| 3 | Existing PIV1 initial principal stake | Same19roles/privileges, native-SOL profile and10000000000 amount; fresh exact protected minimum JitoSOL output |

The projected7454364674-unit minimum and message SHA `e8b7bc8adff8b39465c9e52e4b7b660d451d40e7d79feaebd58f6a3e4970beb6` are observation identities, not live templates or economic results. Operational minimum units must be freshly derived by the existing reviewed builder and satisfy its exact initial-one-lamport eligibility rule. The signed original must then bind the exact reviewed packet; no reused C reference/Store/rights or rebinding follows.

After approval, make one narrow staking nonce derivative using the existing nonceInfo/BIND/VERIFY primitives. The governed SDK's839–846 plan/controller is not reusable as a staking client. The generic pinned native backend needs no new key/profile or relaxed signature check; compatibility must still be reviewed against the actual new packet.

Update only the nonce-specific message/lifetime parser, admissions and simulation/receipt adapters. The staking CPI outer group changes **2→3**; exact pool/System-transfer/Token-mintTo order, operands, amounts and stacks **2/3/3** stay unchanged. Existing strict representation-aware CPI semantics and exact PIV event bytes/values remain. Account-key indices and receipt balance vectors must be derived from the new24-key message, never hard-coded old indices. No on-chain program/ELF, instruction ABI, accepted economic rule or recipient changes.

## Required admission and observation boundaries

Keep all existing pre-original/full-static/source/Node/artifact/old-HOLD-body/WAL/durability/current-control audits and their timing. No D-093–D-097 metadata, history, Node-body or receive-age exception transfers to staking. Current account snapshots must be newly read at every required admission; full union becomes32→34accounts, including nonce and RecentBlockhashes.

Bind the exact nonce state at reference/BIND, both complete admissions and immediate signing/sending boundaries: existing account, System owner,80bytes/version1/Initialized1, G0 authority, exact original nonce value/full-data hash, protected rent and fee field. Require coherent current account context/Clock, exact fee quote for that original message, unchanged protected custody/authorities, current official Jito pool/validator state and all existing financial gates. RecentBlockhashes is dynamic ABI metadata: validate its fresh native shape/owner; do not demand its data remain immutable.

For this nonce variant only, do not invent lastValidBlockHeight or report `isBlockhashValid=true` for a stored nonce. Its equality/authority/state/original checks replace those conventional lifetime predicates explicitly. Preserve monotonic current chain observations and the1s response-age gate through final admission. Any failed identity/lifetime/current predicate latches HOLD. No current-state caching or added audit deferral is proposed.

Simulation must use the exact nonce-bound unsigned message, `sigVerify:false`, **`replaceRecentBlockhash:false`**, finalized commitment and required context floor. The existing replacement setting would not demonstrate the nonce lifetime. Keep the SAME five economic return rows and compare every full poststate, owner/data invariant, exact CPI semantics and event. Require success and≤1400000CU; old931728CU is a baseline only, not a measured nonce cost. [Solana simulateTransaction parameters](https://solana.com/docs/rpc/http/simulatetransaction).

The nonce's hypothetical poststate is not returned. The explicitly requested inference uses its verified current state, exact native Advance operands/data, runtime nonce validation and simulation success. Its actual full80-byte poststate, fee and rent must be read after actual execution. D-098's three omitted writable SOL simulation rows and six readonly omissions remain exactly bounded; all original14 actual economic closure checks remain. An unexpected nonce result could be discovered after effect, without guaranteed undo. No sixth-return support, successful nonce simulation or atomic restoration is claimed.

## Success, failure and long-lived-original recovery

Before FINALIZED success, verify the original signature/status and exact finalized transaction/message, soleG0 signature, Advance-first and unchanged trailing instructions, actual fee, all24 pre/post balance entries, exact staking CPI/event/token amounts, full14 economic closure conditions and full current nonce state. Require its advance under unchanged owner/version/authority/rent and the qualified fee-field profile. A nonce change alone proves neither original finality nor actual fees. Conservative value, principal/HWM10000000000, exact principal queue debit/JitoSOL custody and one-lamport deficit classification remain mandatory; no simulated amount becomes an actual result.

If validation rejects the transaction, do not assume fee/state effects from a generic error. If nonce validation succeeds but a later instruction fails, native nonce advancement and fee debit may persist while staking effects roll back. Obtain the exact original failed receipt/fee and verify complete economic rollback plus nonce/account state, then HOLD. Do not report staking success or automatically prepare another original. [Native nonce failure semantics](https://solana.com/docs/core/transactions/durable-nonces).

The finite off-chain deadline stops the operator; it cannot expire a signed nonce packet on-chain. Keep pending signed bytes in the protected owner/evidence namespace, never in Git/shareable documents or public logs. Preserve durable original/signature/WAL/once, at-most-one controlled send and zero provider retries. A signing or sending ambiguity freezes both its fee reservation and the potentially outstanding10SOL operation; forbid another stake, nonce use, cancellation/advance, resign/rebind/resend/replacement or receipt adoption. Recovery is read-only for the ORIGINAL signature and associated state. Null status, missing receipt, expiry of the scope or changed nonce do not clear UNKNOWN/HOLD. Any later invalidation/disposition needs a separate exact reviewed authority; it is not granted here.

## Financial boundaries and required gates after approval

| Category | Preserved/current limit in lamports |
|---|---:|
| Principal and recorded HWM | 10000000000 |
| Confirmed cumulative fees | 4250000 |
| Old UNKNOWN/HOLD | 130000(90000mission +40000historical) |
| Separate operational reserve | 370000 |
| Remaining recorded6m mission fee pool | 1660000 |
| Existing locked nonce rent | 1056640 |
| Fresh G0 cash observation | 5983916600 |
| Maximum new transaction fees | 10000, inside existing ceiling |
| New rent / funding / nonce setup | 0 /0 /none |

Fees come only from G0, never the protected principal. Verify fresh solvency including all held amounts, separate reserve and the full scoped fee budget. There is no spending-ceiling increase. Keep the10SOL HWM and conservative actual valuation; if the qualified initial conversion has a one-lamport deficit, record the actual internal conversion cost, not yield, rent or a principal reduction. No invented rewards, epoch advance or distribution timing change.

Implementation requires one focused changed-path suite: nonce message/roles/order and old-variant isolation; nonce state/authority/value mutation; simulation replacement/index/CPI/event/five poststate mutations; exact fee/source/WAL failures; full success/failed-stake receipt/nonce/rollback; ambiguous signature/nonce movement and deadline-with-live-signature HOLD. Reuse unchanged Bank/program/rounding/CPI/cryptographic evidence; no unchanged full suite or new framework. Independently review the actual changed source and complete source pins before RPC.

Then conduct ONE new bounded zero-effect qualification with all effect caps0, a distinct namespace, complete actual final admission≤0.9s and preserved1s gates,20s native reservation and conservative VERIFY/durability/resource envelope without shortening artificial reservations merely to pass. Demonstrate nonce identity remains bound throughout and full terminal closure; do not call compilation or simulation alone PASS. Failure stops further runtime attempts. Only after independent actual PASS and separate exact live GO may the one1/1/1/10000/rent0/≤300s scope execute. Keep existing finite read/confirmation bounds (at most16 read batches plus the sole send), source-bound body/output/resource caps,2GiB free and no competing writer.

## Source and evidence identities

Absolute bases: `REPO=/home/jerem/piv1`; `R=/home/jerem/piv1-pilot-checkpoints/m5-standing-functional-lifecycle-20261031-a`; `U=R/upgrade-upload-preparation`; `S=R/post-upgrade-stake-preparation`; `A=S/staking-lifetime-assessment-a`. Paths below expand from those bases. Full packages, routing configuration, private keys, raw signed packets and large account bodies are not embedded.

| Source/evidence | SHA-256 |
|---|---|
| Approved798544-byte ELF | `beb0bd71e0b675355b2f6f584436a587f6e332c02da5fbc39d04a0092fcc1819` |
| Full approved ProgramData892397bytes | `a34994d4f5c08a7f5dbcf537c665fc31e3937edc43904c4a5a0891a507c4953f` |
| `S/execute-single-stake-D098-CPI-reader.prepared.py` | `77fe0a84d928116feedb4b2094bcbc570d202b6b1a79c533de3f71bad8068361` |
| `S/observe-single-stake-D098-whole-original-C.prepared.py` | `5bbb556d4becc7f358194f695193a055018ce49231b109ddf1c9dd88b79d8706` |
| `S/D098-C-staking-blockhash-lifetime-diagnostic.md` | `f5e1cdfed83b0f6489c339c49c52758e93e49e0290a724f0e4f48134ff76f624` |
| `S/D098-C-whole-original-readonly-independent-actual-review.public.json` | `c62330123a00860b46ea370ed88928e0c477fba6ffa9e81f7d45326a7d79069e` |
| `R/D095-governed-nonce-eight-operation-independent-actual-review.public.json` | `b279873e975f1746d8d0828b0d1d652d59f6f052088cd74a6e6df01887e1c35f` |
| `R/D097-isolated-bootstrap-live-independent-actual-review.public.json` | `20b0c716aea78f36b1bd620fb890342f63d00edccbb10ef2d9f46fb17f05a118` |
| `U/governed-D095-nonce-policy.prepared-v2.py` | `6bcbabf32ae2f7b7304d28dfa73d506bb082d7aab3813b7288c110f8c0421235` |
| `U/governed-D095-nonce-sdk.prepared.cjs` | `c83051d3bab97004555ad3722fb31d0ab5ebbe4a00628f4d71f384a9d0518768` |
| `REPO/tools/startup_signer_backend.cjs` | `88138f40cc24e83c46474318f89f06713dbde84da45e1f119a786ceeaadf36e4` |
| `REPO/tools/startup_release_operator.py` | `f91bb04e912e2e0c8df48e393f4cf791706792a21fd1300661216d6bf2bf7fd8` |
| `REPO/tools/collect_startup_public_evidence.py` | `96309ac61a07dc23f204cd7c799b25ec67624af4d1103b9057b25728db85b7f6` |
| `REPO/spikes/task-0.4-jito/node_modules/@solana/web3.js/lib/index.cjs.js` | `796768bf5a689d808fb6fabfc5ce0db27eee064782cf87621331b807bab9f7e2` |
| Existing Node executable | `bc17c508ffeed0ec622934f9b7fa72f8e78da65350e63c3eceb56fa688aa5e12` |
| `A/nonce-structure.offline.cjs` | `9731b213f38574db34d00393624c258a4bd07ecb5a8e2c7f2c54f165afda99af` |
| `A/offline.stdout` | `e7590bdefd5cfe5ebcc3cd2ec418e1c7584bc8560cce42b409e8d421589d54bd` |
| `A/offline-terminal.public.json` | `47fab23a921da1dc39f7687d60eec699b04eba38e0410434319d7de2b2bd7e4a` |
| `A/public-read-scope.public.json` | `6d2af2e20a4c3f7236bf79b8548689d692e67c4a85b93e856bfee4906756c29e` |
| `A/public-read.request.json` | `4071d1ee20066ea1285827d0c212578ea524ea68a76d9da14772d155e8b69e19` |
| `A/public-read.response.json` | `4ddb9b8387eec060fd788d917bf92392e4e24e4e695fb3c6ed2ac549b1d82d35` |
| `A/public-read-result.public.json` | `b129f9397ab1c5ec19f04d4a62896a50199e0c5a5ce5f7ddf89eb01f76f00304` |

No nonce-staking implementation, new qualification, private signing, send, nonce advance, funding or economic effect has occurred in this assessment. All prior HOLDs/originals/consumed scopes remain preserved. Await explicit approval of this exact proposal before enabling its changed policy. Mainnet, irreversible authority changes and final economic acceptance remain outside scope.
