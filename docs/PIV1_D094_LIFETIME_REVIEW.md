# D094 corrected timing and definitive upgrade-path assessment

## Current reviewed outcome — 2026-11-02 Europe/Warsaw

**STOPPED; no live GO. Conventional blockhash execution: STILL UNKNOWN / UNQUALIFIED. Another live qualification: NOT justified by this evidence.** Source baseline `24cb75ec2d7b6a584d8d963069140a015e8058f8`, `integration/piv1-testnet`, jerem/WeatherTrader2, one worktree. This document is the single sanitized external-review packet; it contains source excerpts and exact measurements without private configuration, transaction packets or the historical evidence corpus. The containing publication commit identifies the final checkpoint; the file SHA256 is supplied separately because a file cannot contain its own final digest.

The latest founder request is `/home/jerem/.codex/attachments/5a86dda3-b6b1-48ca-b7ac-4048d58bd1df/Texte collé.txt`, SHA256 `2f4fd8a65ffb817b609622f701edc30fe1d37d6ba1f4716fb761bdb2b3beab03`. It authorizes this observer-only correction, ONE bounded offline timing invocation and, if conventional lifetime remains unqualified, ONE public offline nonce feasibility assessment. It explicitly withholds nonce creation/funding, signing/blockhash policy changes and activation without founder approval. D087/D093/D094 are unchanged. No new decision is recorded or inferred.

### What changed and what the corrected model proves

Only a new **inactive observation derivative**, not production signer/core/Store/history policy, changes. `R=/home/jerem/piv1-pilot-checkpoints/m5-standing-functional-lifecycle-20261031-a`; `U=R/upgrade-upload-preparation`. Observer `U/observe-governed-D094-whole-original.prepared-v5.py`, SHA256 `f15701b7948b7ba5c55464c4e77d3f1984a86a5a6b00e6dff57e845cf0c6dcf1`, starts one deadline before public setup and final admission, measures actual representative public response capture and its guards inside it, then pads only its measured residual. It never guesses a favorable pre-capture allowance. The outer post-invoke current guard, forensic VERIFY proxy, SEND_INTENT durability and complete before-send admission remain outside the envelope. No callback or current/historical check is removed.

**Faithful envelope boundary: YES; exact native execution/scheduling simulation: NO.** This is explicitly `LIMIT_RESERVATION_ONLY_NO_NATIVE_SUCCESS`. Its padded total is at least20s; a real invoke must pass its final check strictly below20s. Public setup/capture are representative, not private backend execution; real A839 native duration remains UNKNOWN. Capture precedes residual padding: this bounds the total reservation without asserting the state/timing of capture after a genuinely slow signer. It cannot qualify an actual signature or establish chain-height headroom. No real qualification namespace, once, rights or live pre-sign budget was created.

Exact existing production contract, `tools/startup_release_operator.py:511–517,532–536,547–548` (omissions shown):

```python
    deadline=time.monotonic()+20; captured=False
    repo=j.pins();backend=signer_backend_source();raw=encode(payload)
    # ... bounded setup ...
    process=subprocess.Popen([str(j.NODE),str(backend),"--allow-live-sign"], ...)
    # j.NODE evaluates final_admission before native Popen.
    # ... bounded native I/O ...
    if _raw_capture is not None:
        captured=True; _raw_capture(bytes(output),True)
    remaining=deadline-time.monotonic();need(remaining>0,"signer_timeout")
    need(process.wait(timeout=remaining)==0,"signer_backend_refused")
    response=j.parse(bytes(output))
    # ... unchanged cleanup ...
    need(time.monotonic()<deadline,"signer_attempt_unresolved")
    return response
```

Corrected observer, `observe-governed-D094-whole-original.prepared-v5.py:198–215` (metrics-only assignments abbreviated):

```python
started=time.monotonic(); deadline=started+NATIVE_RESERVATION_SECONDS
setup()
need(time.monotonic()<deadline,'D094_shared_envelope_setup_timeout')
admit()
need(time.monotonic()<deadline,'D094_shared_envelope_admission_timeout')
capture()
need(time.monotonic()<deadline,'D094_shared_envelope_capture_timeout')
# The private intermediate marker must be cleared before padding.
while True:
    remaining=deadline-time.monotonic()
    if remaining<=0: break
    time.sleep(min(.25,remaining))
# Report LIMIT_RESERVATION_ONLY_NO_NATIVE_SUCCESS, never native success.
```

Its exact reference/admission path remains at lines226–242: finalized getGenesisHash/getLatestBlockhash/getBlockHeight; fresh BIND and exact immutable proof; PREPARED/SIGN_ATTEMPT; `admit_same_original('before_sign',reservation)`; corrected shared envelope; outer `self.full_guard()`; separate old-public forensic VERIFY **cost only, no adoption**; unsigned SEND_INTENT; `admit_same_original('before_send',self.intent_name)` and `final_admission('before_send')`. The prepared observer's later height-budget logic is not installed or validated as an execution budget.

### ONE offline timing invocation: exact observed result

Root ran only `python3 -B U/monitor-governed-D094-shared-envelope-offline.prepared-v2.py`; monitor SHA `ebb7d5bef9a3c35eb4333b571d8d4d4efbfac9d0f5eeef59eaa62f35fa3cdbc6`, driver `diagnose-governed-D094-shared-envelope.prepared-v2.py` SHA `550ba8c80599922e59681f949bbcbcc56b36f0a5144877df4971c514eb5904cb`. Inherited kernel socket denial covers its public SDK children; CPU600/wall900, file64MiB, groupRSS+swap1GiB, free reserve2GiB, no new AS limit. Normal exit0, wall226.461083912s, kernel CPU179.675765s, peak542670848B, minimum free9239883776B; no guard stop, signal, orphan or cleanup issue.

**Four changed-path fixed-clock cases PASS:** exact residual after setup/admission/capture; setup timeout; admission timeout; capture timeout. Actual core/Store/history/current-owner checks used representative retained public bodies and exact related metadata; old crypto/unchanged suites were reused. Three frozen fixture batches/19 methods, **zero network RPC, native calls, private signatures, sends or fees**. Both current-bank fixtures are the SAME explicitly frozen valid snapshot with synthetic hash/validity; no live account caching or chain-headroom claim follows. An old public A839 VERIFY proxy supplies cost only, never current identity or an executable original.

| Measured interval | Seconds | Interpretation |
| --- | ---: | --- |
| Reference fixture capture → before-sign complete cutoff |19.298579675| Inclusive; final admission overlaps the envelope below. |
| Before corrected shared envelope |17.847600129| Exclusive partition. |
| Public setup inside envelope |0.000203198| Representative public-only setup. |
| Final admission inside envelope |1.450773551| Includes its actual nested guards. |
| Guarded representative capture inside envelope |1.783435111| Includes durability and required callbacks. |
| Mandatory measured envelope work |3.234424243| Includes small measurement transitions. |
| Exact residual padding |16.765681737| No observed backend timing. |
| Total reserved envelope |20.000115074| Conservative total; not successful real invoke. |
| Envelope exit → final before-send cutoff |13.034652302| Exclusive; outer guard/VERIFY/SEND_INTENT/final admission retained. |
| Reference fixture capture → final before-send cutoff |**50.882367506**| Exclusive17.847600129+20.000115074+13.034652302; excludes real transport wait. |
| Before-sign complete response age |2.231650414| Local fixture cutoff, not a live qualification. |
| Before-send complete response age |2.169225324| Local fixture cutoff, not a live qualification. |

The earlier real D094 reference411197284/LVH411197434 and before-sign411197361 left73 sampled heights, only33 above40. Its before-send411197521 was87 expired. These facts remain unchanged; correcting a synthetic envelope does not recover or invent heights. No seconds-to-height conversion, retrospective PASS, exact real signer duration, current provider speed or empirical live pre-sign budget is supported. A839's real expiry remains separate evidence.

Result `U/governed-D094-shared-envelope-offline-host-a/result.public.json`, SHA `0e63c1dd50b7cf90b4d987d3095fe2d14fb1a3812b5b33cbee5d38261552b4de`; terminal `U/governed-D094-shared-envelope-offline-monitor-a/terminal.public.json`, SHA `a397539fab4040c534027a1de8a3ff71e18dab7b122a7bf1d89c67651a64aedf`. Independent source review `R/governed-D094-shared-envelope-offline-source-independent-review.public.json`, SHA `d94f31b383e93ee3101680a67245d491fedae6db81eb548b13a0b0c744660bb5`; independent actual review `R/governed-D094-shared-envelope-offline-independent-actual-review.public.json`, SHA `c3d7f23653ac3f912702300ae39a837ed23a34861f202454eca00b3294412532`; audited inventory SHA `248a0dd7bafb1e051c9e9967152dbc39dbffe5130e28663263e89bf9b0eb9321`. Reviewer verified57 files/4dirs/15evidence/4WAL/15anchors/16stage pairs and the exclusive partition; no further runtime by reviewer.

## Focused nonce and confirmed-bank feasibility

**Structural compatibility: YES for all eight exact retained outer messages; end-to-end nonce execution readiness: OPEN.** The pinned web3.js1.98.4 compiled unsigned nonce variants of839–846 with G0 as the existing fee payer and proposed nonce authority, and no additional signer. Every original instruction's program, data, account order and global signer/writable privileges, including the stored Squads upgrade payload and four guardian votes, remained identical. Only a first System AdvanceNonceAccount and its required accounts/nonce lifetime were added. No account/key was created and no nonce value or account was read on-chain; the derived public address/value are explicit OFFLINE FIXTURES, never accepted operational identities.

| Logical phase | Existing wire bytes | Nonce wire bytes | Required Ed25519 slots |
| --- | ---: | ---: | ---: |
|839 Buffer handover|240|346|1|
|840 Squads transaction create|619|693|1|
|841 Proposal create|343|417|1|
|842 Guardian vote1|301|407|1|
|843 Guardian vote2|397|503|2|
|844 Guardian vote3|397|503|2|
|845 Guardian vote4|397|503|2|
|846 Existing Program upgrade|565|671|1|

All eight fit1232 bytes; total11 existing signature slots. An unsigned setup SHAPE using G0 as payer/base/authority, createAccountWithSeed then nonceInitialize, compiled to428 bytes/one signature/80-byte nonce account. Its zero-lamport placeholder is deliberately NOT a rent quote or valid funded setup. Availability, exact live nonce address/value/authority, rent, fee quote, compute sufficiency and setup execution feasibility remain UNKNOWN. Setup would still need its own ordinary recent-blockhash and unchanged applicable security/freshness gates; this assessment does not solve or waive that prerequisite.

Relevant pinned SDK source `spikes/task-0.4-jito/node_modules/@solana/web3.js/lib/index.cjs.js`, SHA `796768bf5a689d808fb6fabfc5ce0db27eee064782cf87621331b807bab9f7e2`, lines1444–1451:

```javascript
if (this.nonceInfo) {
  recentBlockhash = this.nonceInfo.nonce;
  if (this.instructions[0] != this.nonceInfo.nonceInstruction) {
    instructions = [this.nonceInfo.nonceInstruction, ...this.instructions];
  } else {
    instructions = this.instructions;
  }
}
```

The public assessment `U/assess-governed-nonce-public-unsigned.prepared-v2.cjs` lines19–40 uses the existing Transaction.populate(Message.from(...)) and the following exact compilation/check structure (metadata and additional assertions omitted):

```javascript
const advance=w.SystemProgram.nonceAdvance({noncePubkey,authorizedPubkey:g0});
const tx=new w.Transaction({feePayer:g0,
  nonceInfo:{nonce:nonceValue,nonceInstruction:advance}}).add(...old.instructions);
const message=tx.compileMessage();
const wire=tx.serialize({requireAllSignatures:false,verifySignatures:false});
const decoded=w.Transaction.from(wire);
assert(wire.length<=1232,'PACKET_SIZE');
const expectedAdvance=shape(advance);
expectedAdvance.keys[2].writable=true; // Existing G0 fee-payer GLOBAL privilege.
assert(JSON.stringify(shape(decoded.instructions[0]))===
  JSON.stringify(expectedAdvance),'ADVANCE_FIRST');
assert(JSON.stringify(decoded.instructions.slice(1).map(shape))===
  JSON.stringify(old.instructions.map(shape)),'EXACT_EXISTING_INSTRUCTIONS');
```

**One focused assessment, two short offline invocations, transparently retained:** invocation-a stopped normally after0.828686s with an unavailable Node exception. Independent source inspection found the harness wrongly compared local nonce authority read-only metadata with G0's compiled global writable fee-payer role. The source defect is independently established by pinned SDK populate1940–1959 and nonceAdvance3035–3059; its actual thrown Node error was discarded and remains UNKNOWN, not retrospectively recovered. Only this representation assertion and allowlisted failure output/durability routing were corrected; no security privilege changed. Invocation-b passed in0.543264s, kernel CPU0.577109s including descendants; zero network/private signing/send/fee, no resource stop/signal/orphan. No additional iteration followed. First failed namespace and all unrun drafts remain unchanged.

Exact final assessment source SHA `f978a6aa23b0a38c1cdbfd49185269e56ebc1e8e31766e0a36633ff936d08349`; Python driver `assess-governed-nonce-public-unsigned.prepared-v3.py` SHA `12a2bab945b8296bc4bc8f8b98009ddade87ee870704e2d25aa2ee14b8cd0e22`; existing monitor projection `monitor-governed-nonce-public-offline.prepared-v3.py` SHA `71ff7ffe4acc449c1e67f20c62bf2acd86002e48eba15cbb7c9de978a0f26333`. Same inherited socket denial and bounded resources; Node child hard20s. The exact846-operation public plan SHA `6feb2bfddf35a62b9fab491cb5b5dd46e56f01e98cfb2e098b083612ad83dd88` and pinned Node SHA `bc17c508ffeed0ec622934f9b7fa72f8e78da65350e63c3eceb56fa688aa5e12` were verified, not regenerated. Result `U/governed-nonce-public-offline-b/result.public.json` SHA `844d8f240a1ac11fb0058c3df296997fd37ee2d051a628c0f0356b556d5ff60b`; monitor-b terminal SHA `86c21d076123194f1be863b97f420cff9c6b643f350fa192644ff8dc12aeb470`; failed monitor-a terminal SHA `e6cd7cbb93017a9d7f14bfea09f55e8c4b41c45845921d60ad818810a7723966`. Independent nonce source/actual review: `R/governed-nonce-public-offline-independent-review.public.json`, SHA `d237ddc199797d8744ee36575f741a24473cd3efb5e0c672bd03e8c7dc064815` (embedded exact11-file/four-directory inventory)..

### What a nonce would require; no drop-in activation

Nonce advance must be first and the nonce account writable, System-owned/initialized and controlled by the exact reviewed authority. It can consume the nonce and charge fees even if a later instruction fails. A signed nonce packet can remain relayable after an off-chain execution window expires. Changed nonce bytes alone do NOT prove which signature executed, finality, fees or the Squads effect. Official semantics: [Solana durable nonces](https://solana.com/docs/core/transactions/durable-nonces).

The existing dedicated SDK BIND/VERIFY, signer public checker, original binding/lifetime gates, current account collector and receipt oracle expect recent-blockhash templates. They require a minimal explicitly reviewed nonce variant, not silent template adoption. In particular, core `execute-governed-D094.prepared-v3.py:725` currently requires `innerInstructions == [{'index':3,...}]` for the relevant Squads CPIs. Prepending Advance shifts that exact top-level index to4; the revised oracle must bind the expected index and unchanged exact CPI effects, NOT accept arbitrary indices. Nonce advance introduces an intentional state/fee effect on execution failure; old all-effects rollback assumptions cannot be inherited unchanged.

One nonce value may bind at most ONE signed original, with durable exact intent/wire/signature before the controlled at-most-once send. Every required current snapshot must freshly include the complete nonce account/owner/authority/value and original message, fee, account/custody state; no cached on-chain sample between admissions. An ambiguous signed original latches UNKNOWN/HOLD and blocks reuse of that nonce value, replacement, automatic nonce advance/cancel or a new original. Original-signature-only receipts plus actual nonce/governance/financial state are required; nonce change alone is insufficient. Off-chain window expiry is never substituted for on-chain invalidity. Old A839 and all other consumed originals remain untouched.

Exact nonce-message fee support must be qualified with a non-null quote; the standard RPC promises the charge at the requested commitment, not an assumed zero. [getFeeForMessage](https://solana.com/docs/rpc/http/getfeeformessage). Solana's October2026 notice says native nonces may be removed after a future replacement is ready, with no final date; current Testnet support/activation is unverified here. This bounded fallback would not authorize migration to a new signer program/PDA or new authorities. [Foundation deprecation notice](https://solana.com/nl/upgrades/durable-nonce-deprecation).

### Options and ONE recommended decision

| Option | Security/implementation | Evidence of solving lifetime |
| --- | --- | --- |
|Later finalized capture / same-boundary coalescing|Would preserve existing policy if every required callback stays; remaining relocation is limited, further speed work not justified.|No measured whole-original fit; do not start another optimization campaign.|
|Confirmed reference + finalized account observations|Requires reference/validity/height/fee/preflight compatible with the confirmed bank, distinct context floors from finalized custody snapshots, and a new explicit bank-policy decision. Newer hash can be unknown to finalized preflight; one higher shared minContextSlot could erase its benefit.|Potential gain and fork exposure unmeasured; does not remove finite lifetime. Not a sufficient basis for another qualification.|
|Native durable nonce for839–846|Small outer-message addition fits all eight messages and keeps signers/Squads payload. Requires explicit lifetime/recovery/state/receipt changes and setup support; no migration/framework.|Removes the specific recent-hash expiration mechanism by protocol design; structural fit verified, actual setup/client/current-RPC qualification still OPEN.|

Confirmed-bank assessment is a design inference from the retained finalized path and [official confirmation guidance](https://solana.com/developers/cookbook/transactions/confirmation); no provider/current bank coherence was sampled and no existing blockhash commitment was changed. Do not assume BlockhashNotFound is solely latency. The20s native deadline and all current checks remain independent controls under any option.

**ONE recommendation: request the narrowly bounded PROVISIONAL / TESTNET ONLY nonce-lifetime decision below before implementation.** It is outside confirmed D087/D093/D094 exact lifetime rules, so work stops at this policy boundary. No repeated conventional live qualification is justified. Structural compatibility is sufficient to present this decision, not to promise upgrade completion.

**Proposed decision, INACTIVE:** authorize a native durable-nonce variant ONLY for the eight remaining logical governed upgrade operations839–846 of the SAME existing Program/Buffer through existing Squads4-of-6. Replace ONLY their recent-blockhash/LVH/pre-sign height-budget/40-height predicates with exact fresh initialized System-owned nonce-account/value/authority validity at every mandatory admission and explicit nonce-aware original/recovery checks. Keep D093/D094 full historical/current/source/Node/artifact/authority/custody/WAL/financial checks,3.000s receive-age/2.900s complete qualification target, exact signatures/messages/fee quotes,20s native limit, durable at-most-once sends and all UNKNOWN/HOLD protections. Permit no uncertain nonce reuse/replacement/automatic cancellation and no inference that a signed nonce packet expires with its operational window. Acknowledge its longer relayability and nonce/fee consumption on a failed governance instruction. No staking, economic, Mainnet or other-operation inheritance.

After such approval only: implement the minimal existing-client/checker/receipt variant; focused tests for exact advance/order/roles, nonce mutation/authority races, signed-original isolation, fee/failure semantics, CPI index and HOLD/reopening/durability; independent review; ONE distinct zero-effect complete qualification before live GO. At most one separately reviewed G0-payer/base/authority80-byte setup may be prepared if no suitable authorized account exists; no new private key/authority transfer. Its fresh exact address/rent/fee/funding and ordinary recent-blockhash execution feasibility must pass before creation; this proposal does not waive or extend setup freshness. No nonce is created or funded now. Setup/live execution still require exact bounded reviewed scopes under the approved mechanism.

For prospective technical budgeting ONLY, current848native/848Ed/839sends +8governed native/11Ed/8sends +at most one setup1/1/1 would require857native/860Ed/848sends, versus unchanged existing855/858/847. This is not an activated cap change; any adjustment needs independent prospective review and fresh exact fees/rent, retaining the6m spending ceiling and all130000held/370000reserve. Planning allowance at most60000 new fees (55000 existing governance +5000 setup), not an actual quote/spend; nonce rent remains UNKNOWN. Protected10SOL principal/HWM is never setup funding. A failed prerequisite stops safely rather than triggering another policy or optimization loop.

## Preserved state and stopping boundary

All836 Loader Writes/full798544-byte approved Buffer remain COMPLETE; no upload repeated. Same Program `GAjHRaC3XM98SoRoFy5B8P2ypTS1ndf1UPhXmjWgtPux`, Buffer `HJSB6xLyKxMet5sxXrc5SMpppXxF9BcLAvSaCxPVfroE`, G0 `BUuuNvicRKF4ZFEUH4iS8xGzhAKQca3ou4U1s4NaBYoT`, Vault/Squads4-of-6/approved ELF payload SHA `beb0bd71e0b675355b2f6f584436a587f6e332c02da5fbc39d04a0092fcc1819` unchanged. The upgrade remains BLOCKED and real JitoSOL staking NOT STARTED.

Confirmed fees4190000, UNKNOWN/HOLD130000 (including A8395000), separate reserve370000, remaining6m fee pool1720000, protected10SOL principal/HWM; actual848native/848Ed/839sends and existing855/858/847 technical caps remain unchanged. Every prior original, consumed scope, signed packet, failed run, HOLD and uncertainty is preserved. A839 finality/fees/non-inclusion remain UNKNOWN. No new funding, rent, signature, fee, policy activation, authority change, Mainnet or main publication. M4 remains OPEN_UNWAIVED/publicM5 NOT_READY.

## Historical source-only review (published in24cb75ec; retained, superseded only where explicitly stated above)

Status: CONFIRMED source/evidence findings; conventional execution readiness OPEN. No implementation or execution GO.

Reviewed 2026-11-02, Europe/Warsaw, from clean `integration/piv1-testnet` HEAD `2461b370e3614fddc0e69d2c44b18f13b0cfd98d`. No competing writer, new RPC, signing, sending, qualification, build or test run. Existing passing tests and reviews were reused. No historical-body replay or private locator/profile/key-file read was needed for this review.

## Exact inputs

Evidence root `R=/home/jerem/piv1-pilot-checkpoints/m5-standing-functional-lifecycle-20261031-a`; `U=R/upgrade-upload-preparation`.

| Input under U | SHA256 |
| --- | --- |
| `observe-governed-D094-whole-original.prepared-v3.py` | `40de1165953a3f8ed1e167d7398959fece5360b479629b88015e9236ae38697a` |
| `execute-governed-D094.prepared-v3.py` | `c1c632433b276b4f6fc20d82455bccf4565669ec1faf976929c2935d90e7821c` |
| `remaining-history.governed-D094.prepared.py` | `ec746308fe5cc6c9e3baf21dcb1ef5de1c22937efcb3e8c4d29a01bd96de31d3` |
| `dedicated_store.scope-finalized.prepared-v4.py` | `5402896000fbc88b45fa945841e2dc3b02320d12bc496d86028280249797da6a` |
| `execute-governed-D093-owner.prepared-v2.py` | `7c94931297148d6617035df086075efb1543bbd781c3b760d7baa79c35d63219` |
| `scope-governed-a/execute-upgrade-upload.active.py` | `ecbec80281bd5c8aa622f3927d62c5d25096d19f9c0ffe079e70b9211a22d417` |
| `governed-readonly-D094-whole-original-a/readonly-failure.public.json` | `262e3891a8c99a59ea42d08b9e6c05748310088c9476373d74acb6a481809ce6` |

The D094 actual review `R/governed-D094-whole-original-readonly-independent-actual-review.public.json` (`6fbe50fb…0658b95f`) and its `c087ec44…07f96cff` inventory remain authoritative retained observations. A839's `R/governed-D093-live-independent-actual-review.public.json` (`d48413cd…fc458618`) remains separate. Nothing below upgrades either original's evidence or clears a HOLD.

## Chronological path and the 36 guards

This is a static census of the source path reaching the recorded failure, consistent with the observed total36. It is not a recovered per-call timing trace. `full_guard()` calls `guard(full_static=True)`; `intermediate_guard()` retains a current guard and applies only the D094 OLD PUBLIC metadata exception.

| Order | Actual source action | CURRENT_GUARD calls | Relation to reference |
| --- | --- | ---: | --- |
| 1 | Constructor SCOPE record pre/post, then explicit full_guard | 3 | Before capture |
| 2 | prepare_original_history → full_history_guard | 1 | Before capture |
| 3 | Main public SDK PREPARE; separate forensic SDK PREPARE and BIND, two guards each | 6 | Before capture |
| 4 | Reference RPC request write pre/post; returned response capture postguard | 3 | Two before transport; one after response |
| 5 | Fresh main SDK BIND pre/post | 2 | After capture |
| 6 | Explicit after-BIND full_guard | 1 | After capture |
| 7 | Store.begin_original → owner.guard | 1 | After capture |
| 8 | bind_original_history → full_guard | 1 | After capture |
| 9 | PREPARED record pre/post | 2 | After capture |
| 10 | SIGN_ATTEMPT record pre/post | 2 | After capture |
| 11 | before_sign: admission preparation, fresh pre-RPC guard, outer capture postguard, final_admission guard | 4 | Same bound original |
| 12 | Twenty-second sleep, then native-result stand-in capture postguard, callback full_guard, outer post-invoke full_guard | 3 | After before_sign cutoff |
| 13 | Separate forensic VERIFY pre/post | 2 | Stands in for required live signed-wire VERIFY |
| 14 | SEND_INTENT record pre/post | 2 | After signing stand-in |
| 15 | Failed before_send: admission preparation, fresh pre-RPC guard, capture postguard on refusal | 3 | Final cutoff not reached |
| | Total at failure-envelope construction | **36** | **12 before reference transport;24 after response capture begins** |

Four direct static audits explain40 CURRENT_SOURCE_NODE_ARTIFACT calls versus36 guards: constructor bootstrap, begin_static_original, and two owner.private_route → owner.full_guard → source_audit calls. Those owner calls do not invoke the measured engine guard. Later HOLD handling is after the failure timing snapshot; do not assign its work to the36-call snapshot.

`Store.batch` already coalesces nested writes using `depth`. During `admit_same_original`, the outer `Store.capture(lambda: fresh(...))` holds depth1: RPC request/response writes do NOT each add another guard pair. Only its outer postguard runs, including on failure. A capture intentionally has no preguard: it preserves literal returned bytes before a failing postguard can refuse progress. Removing that property would lose evidence.

Each real guard still checks current source/Node/artifacts, loaded source/control equality, owner/current private metadata controls, authorization/budgets, current Store/WAL/independent anchor, and static identities. Private historical context/inputs and complete old metadata surround required current checks; intermediate callbacks defer only the approved old metadata. Fresh RPC admissions additionally validate coherent full accounts/Clock, custody/authorities, fee/rent, exact blockhash lifetime and financial state. `final_admission` freshly guards, checks the exact durable record and old identities/inventories, then enforces age/lifetime before the effect. No current bank snapshot may cross between the two admission samples.

## Timings: what can and cannot be attributed

| Retained observation | Meaning / exclusive-time limitation |
| --- | --- |
| Reference411197284; LVH411197434 |150 observed heights at reference sample. |
| before_sign411197361, validTRUE |77-height advance;73 observed heights remain at the RPC sample, not a new height measurement at native Popen. |
| before_sign complete2.065843527s |Real response-to-complete-cutoff interval; includes nested guard work. |
| final_admission1.335408480s |One inclusive measured subinterval of that cutoff; do not add it to2.065843527. |
| account view0.021786023s |One leaf interval relative to the instrumented stage wrappers. |
| sleep20.000123913s |An exclusive intentional wait, not observed native signing. |
| before_send411197521, validFALSE |160 heights after the before_sign RPC sample;87 expired/127 short of40. This sample-to-sample interval also includes the earlier post-response admission work. |
| before_send refusal0.795467189s after receive |Not a completed cutoff; account view/final admission not reached. |
| guard36/114.403398s; source audit40/18.458618s |Inclusive whole-run aggregates; neither is exclusively post-blockhash. Do not distribute them proportionally across rows or add them. |

The `postFreshReceive*` aggregate is also not one admission window: its label can remain before_sign during the native stand-in, VERIFY, SEND_INTENT and preparation of the next RPC. Its12 guards/11.566379s cannot be assigned to a single3-second cutoff.

No D094 per-call exclusive durations for BIND, native capture, Store/fsync, owner metadata, VERIFY or RPC transport were retained. Old, non-overlapping HOST top-level stage intervals exist: Store.begin_original1.348798s, guarded BIND2.602169s (IPC0.004222s nested), capture placeholder3.982663s, guarded forensic VERIFY2.670038s (IPC0.057696s nested). These are D093 HOST fixture measurements, not real A839 signing times or forecasts for D094. SDK initialization and full history preparation already precede reference capture; D094 measured its full preparation23.626222s inclusively there. Relocating them again cannot recover blockhash lifetime.

## Source-confirmed timing-model defect

`tools/startup_release_operator.py:508–548`, invoke_signer, sets `deadline=monotonic()+20` BEFORE repository/backend/input setup and the `j.NODE` property evaluation. Core sign's NativeAdmission.NODE calls final_admission(before_sign). Returned stdout then calls raw_capture; its Store.capture/fsync/postguard and callback full_guard are INSIDE this SAME20-second deadline. The wait/return deadline checks follow capture. The outer core sign full_guard after invoke returns is outside it.

Observer40de instead completes final_admission first, starts a NEW20-second sleep, then performs the capture callbacks outside that sleep. Thus the native envelope is conservatively over-reserved: the measured1.335408480s final-admission subinterval and an unmeasured capture portion are charged outside an envelope that includes them in production. This is a fidelity defect capable of producing an overly pessimistic qualification; it is not a security bypass or proof that correcting it will fit. The complete failure remains valid for the actual observer that ran; no retrospective PASS.

A839's503-byte native response contains schema/requestId/packets; its18-byte complete marker says true. Neither contains elapsed timestamps. Its later SEND_INTENT proves the guarded invoke returned successfully and passed its final shared20s deadline check; this is neither a backend-duration measurement nor a hard real-time completion guarantee across scheduler pauses. The actual signing duration remains UNKNOWN. A839 also expired, so this observer defect is not the sole demonstrated cause of live failure.

The extra forensic SDK worker and its BIND are diagnostic-only and already before capture. The forensic VERIFY after the wait is an imperfect cost proxy for the REQUIRED fresh signed-wire VERIFY in run_phase. Removing its two guards and treating all that work as a saving would omit live-path work. It never supplies the fresh original's identity.

## Eligible relocation and same-boundary redundancy

Later capture is allowed only before the FIRST binding. Full immutable history/body preparation, SDK initialization and template checks are already there. Store.begin_original's conditions use scope/Store identity, closed prefix and phase, not bound message/lifetime; moving that state-only opening before reference is a candidate relocation, retaining the complete after-BIND guard and one-shot history binding. Its historical HOST1.348798s does not establish enough present headroom. PREPARED/SIGN_ATTEMPT/SEND_INTENT must bind the fresh exact packet/request/signature and cannot become pre-blockhash placeholder authority. BIND, history binding and both admissions remain original-dependent. A signed or consumed original can never be rebound.

Inside ONE guard, _source_audit hashes static_files, then rereads overlapping current source_immutable/control/plan bytes; guard adds further current Store/static checks. begin_original immediately repeats Store/current-closed checks after owner.guard without a mutation. A boundary-private union could coalesce identical current-path reads while preserving conflict rejection, actual bytes, before/after custody, all current roles, owner controls, WAL/anchor, failure latch and proof disposal. It must never survive SDK work, durable write, transport or another mandatory callback; guard_busy is not reusable success authority.

Static configuration inspection found99 overlapping static/source-control paths,3,000,654 bytes. The125,989,464-byte Node executable is not duplicated in that overlap and remains freshly hashed at every required boundary. This is a bounded work-volume opportunity, not measured seconds or sufficient lifetime gain. Do not start a coalescing campaign or remove pre/post callbacks on this evidence.

## Blockhash alternatives, assessed only

**A — Later finalized capture:** technically compatible with one fresh original and exact identity, within D087/D093/D094 if all original-dependent checks remain afterward. Only limited remaining relocation is identified above; no execution fit is proved.

**B — confirmed reference with finalized accounts:** possible as a different reviewed design. Reference, blockhash validity, fee quote, height and preflight must use a compatible confirmed bank; custody/economic account snapshots and final receipts can remain finalized. Their context floors must be separate: reusing the higher confirmed hash slot as finalized-account minContextSlot can wait for finalization and erase the benefit. A newer hash may be unknown to finalized preflight. Gain and fork exposure are unmeasured here; changing this bank policy is outside the exact D093/D094 exception. No one-word commitment edit is approved. [Official confirmation guidance](https://solana.com/developers/cookbook/transactions/confirmation), [commitment-specific validity](https://solana.com/docs/rpc/http/isblockhashvalid). This separation is a design inference, not a qualified provider configuration.

**C — durable nonce fallback:** removes recent-blockhash expiry, but requires a System-owned nonce account, exact authority, AdvanceNonceAccount first, fresh nonce-state checks and revised outer messages. The nonce advances and fees can remain charged even when later execution fails. In principle an outer nonce instruction can precede a Squads instruction without changing the stored upgrade payload or4-of-6 vote rules; current dedicated BIND/lifetime/checker/template paths do not implement it. Installed pinned web3.js supports nonceInfo, which alone is not client qualification. Existing nonce account availability/rent/setup authority are unverified. Replacing the40-height predicate with nonce validity and revising expiry/recovery would require an explicit narrowly reviewed security decision, exact finite package/caps and UNKNOWN rules. No nonce decision is requested or assumed now. [Official durable-nonce semantics](https://solana.com/docs/core/transactions/durable-nonces), [Squads outer execution transaction](https://typedoc.squads.so/functions/transactions.vaultTransactionExecute.html).

## ONE recommendation and stopping condition

**Correct the observation model to use the SAME shared20-second invoke_signer envelope before deciding whether another execution mechanism is necessary.** Affected future code is observer run's timing block (currently204–213); unchanged production invoke_signer508–548 and core sign694–709 define its contract. Start the accounting at invoke entry, include final_admission and guarded raw capture within that envelope, and retain the outer post-invoke guard/VERIFY/SEND_INTENT/before_send as subsequent work. No callback or current check moves or disappears. A stand-in wait may use only the residual of this envelope after an independently justified allowance for its remaining capture/cleanup; choosing an arbitrary residual would merely create another unreliable qualification.

This correction fits D087/D093/D094 and requires no new security exception. Its evidenced effect is correcting the placement of1.335408480 measured seconds plus an UNKNOWN capture component, not a promised height saving. Corrected total fit remains OPEN:73 sampled heights leave33 before the40-height floor; retained traces cannot credit the127 missing heights or establish a conservative residual capture budget. Conventional execution is not disproved, but no executable whole-original path is demonstrated yet.

Required focused future checks: one deadline shared by setup/admission/wait/capture/return; fixed-clock timeout inside final admission and capture; mandatory callback ordering and late current mutation/HOLD; exact original/proof isolation; no native/send and no invented residual budget in the observer. Use the existing bounded HOST driver if new exclusive timing evidence is needed; no new framework or unchanged-suite rerun. First establish the complete envelope and remaining tail headroom offline. Only credible reviewed whole-original evidence can justify a DISTINCT future qualification/scope. No implementation, test or qualification was started here because that viability condition is not met. Do not activate the prospective856/859/847 caps or choose confirmed/nonce as a silent workaround.

All836 Writes/full approved Buffer, existing Program,4-of-6,10SOL principal/HWM,4190000 confirmed fees,130000 held,370000 separate reserve,1720000 remaining fee pool and all consumed originals/HOLDs remain unchanged. Upgrade/staking remain NOT STARTED. A839 finality/fees/non-inclusion remain UNKNOWN. No Mainnet, new Program, reinitialization, replay, fee-cap increase or authority removal.
