# D094 whole-original lifetime and repeated-guard review

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
