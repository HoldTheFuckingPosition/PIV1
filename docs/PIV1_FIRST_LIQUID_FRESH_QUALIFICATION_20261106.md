# Fresh first-liquid qualification: state PASS, recent-hash lifetime FAIL

2026-11-06 UTC; source baseline `b56580b9eb701ca1b2eee31913a9af62a9afc0fa`, `integration/piv1-testnet`, jerem/WeatherTrader2. **STOPPED/no process remains. No real preparation, settlement or pending integration occurred.** Existing upgrade,10SOL stake and5SOL Deposit remain finalized. D087 remains valid; no security exception, on-chain program/economic change or Mainnet action.

## Confirmed result and exact blocker

The stale-opening problem was corrected. ONE genuine zero-effect qualification obtained a fresh complete36-account finalized opening461447426/Clock1794006358/epoch1080, decoded the ENTIRE current848-record validator list and authenticated official pool/list/reserve/mint relationships. Its six-economic/fee-row dual simulation PASS at461447443,965827CU, exact current model/CPI/event. Every35nonClock account remained **byte-for-byte and metadata/balance identical** at both later raw samples. No structural shared-Jito liveness failure was reproduced; there is no evidence-based reason to relax shared-account equality.

The ordinary recent-hash whole-original qualification nevertheless **FAILED before-send lifetime admission**, with the20-second native-deadline reservation retained. It created no real signature or send. Do not run another unchanged qualification or a live transaction from this result.

| Boundary | Finalized context / Clock Unix | Block height | Last valid | Remaining heights | Result |
| --- | --- | --- | --- | --- | --- |
| Fresh opening |461447426 /1794006358|412821050|not yet bound|not applicable|Full current state PASS. |
| Current dual simulation |461447443 /1794006361|not sampled|replacement not adopted|not execution authority|Full6poststates PASS. |
| New unsigned original reference |461447488 /not sampled|412821108|412821258|150|Fresh finalized blockhash captured. |
| Complete before-sign |461447518 /1794006378|412821138|412821258|120|validTRUE/fee5000/complete0.4735493455s PASS. |
| Before-send raw response |461447643 /1794006405|412821263|412821258|−5|validFALSE/fee5000; expired5,45short of required40. |

New unsigned blockhash `7iAxSt62Wv5km6wpxpvAXYaSZKA2vGZjKYVfyyYjgi8Q`;1092-byte unsigned wire SHA `8965d71cf7843f6fdca7d8c587a410f2430294033f62ac70013fefbf46b95d49`, message SHA `0be42dfeff6ddff2b760443bc70fcd668e7b2aaa8ca2ff2525f2b18325968f23`. One all-zero64-byte signature slot, not a transaction signature. Reference→before-sign consumed30heights; sign→send consumed125heights. To preserve40, that observed tail could consume at most80: **45heights of additional margin were missing**. No speculative seconds-to-height bound is asserted.

The real reserved interval was20.0001482489s, including public backend-source validation, unsigned size-proxy capture/fsync and required guards; **no native signer ran**. Actual native and successful signature-VERIFY durations remain UNKNOWN. The11-second staking-only VERIFY stress was not inherited. Root wall52.390130916s/normal exit1/no signal/resource stop/orphans/cleanup issue; observer failure elapsed51.484229910s. Before-sign was20Clock seconds after the opening, rather than1640. Before-send full response was47Clock seconds after it. Before-send complete receive age was **NOT REACHED**, because lifetime validation runs first. This is not evidence of a1-second receive-age failure, a paid failed transaction or universal impossibility of all actual recent-hash execution.

## Exact current accounting

Pool total4449472625757 / supply3316738623763, actual mint supply equal, lastUpdateEpoch1080/currentClockEpoch1080. Historical7454364674JitoSOL units conservatively value10000182505lamports; protectedHWM10000000000; **genuine current gross182505**. Recognized pending5000000000 remains separate; physical pending5000650240 includes650240rent. Principal SOL650240rent; G0cash983906600; actual principal Jito7454364674. RoundIdle/sequence0/lastSuccessfulNone/lastInsufficientNone/KIFperiod0/bitmap1/G0onlyactive/liability0.

Simulated-only split:107677HTFP /35588team /3650KIF /35588compound /2dust, outgoing146915pending→escrow. **Actual allocations/payments0; preparationBLOCKED, settlement/integrationNOTSTARTED.** Simulated preparation time1794006361 is not an actual cadence anchor. Actual successful-preparation timestamp and next+864000s boundary remain UNKNOWN. No forced withdrawal or automatic restake.

Confirmed cumulative fees4260000, actual native/Ed25519/send counts859/862/850, unchanged. New scope fees/signatures/sends0. Preserve **135000UNKNOWN/HOLD**,370000separate reserve,1056640lockednonce rent,1645000unallocated missionfee pool under unchanged6m ceiling. No new financial reservation from either caps0 scope.

Consumed real `prepare-liquid-a` remains unsigned/no recorded native or send/no confirmed fee. Opening observed its OLD reference invalid and H412821050>LVH412811726; **this proves reference expiry, not original finality or fee clearance**. Its owner-initiatedSIGKILL−9 and unproved normal worker exit remain retained; its5000hold is not released. Independent renewed disposition at oldN/`prepare-liquid-a-consumed-disposition-20261106-b.public.json`, SHA046084de; no replay, replacement, recovery adoption or old-window renewal.

## Raw public account comparison

For each row below, owner/nonexecutability/existence/cash/full raw bytes and rentEpoch are identical in the fresh opening, before-sign and refused before-send responses. The same complete equality was independently checked for ALL35nonClock rows, including multisig, ProgramData, registry/rewards, both beneficiaries and nonce. Clock is the sole difference. The before-send comparison is an offline retained-response check, **not successful send admission**.

| Role / exact key | Owner | Lamports in all3samples | Data bytes | Full data SHA256 in all3samples |
| --- | --- | --- | --- | --- |
|Jito pool `Jito4APyf642JPZPx3hGc6WWJ8zPKtRbRs4P815Awbb`|`SPoo1Ku8WFXoNDMHPsrGSTSG1Y47rzgn41SLUNakuHy`|5143440|611|`e4ee141261b992811832d512e4a26cb0ea4655412d626f59c2ce404c93799e4c`|
|Validator list `G5N6K3qW86GSkNEpywcbJk42LjEZoshzECFg1LNVjSLa`|`SPoo1Ku8WFXoNDMHPsrGSTSG1Y47rzgn41SLUNakuHy`|5081753520|730009|`e8fd3e4cc79b7992a3b3dbc62d4e008e81a0598d394594fb5f1905f2ecc39c30`|
|Jito reserve `CzKqc9cs4XpyG6y4peQgk3vBjPyqhktmfqaMuMBCXCqm`|`Stake11111111111111111111111111111111111111`|4359655565362|200|`a60a15f610a908fd6e7b05299dac1ac525d022866722dc2f27f5a5ded4d12c04`|
|JitoSOL mint `J1toso1uCk3RLmjorhTtrVwY9HJ7X8V9yYac6Y7kGCPn`|`TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA`|1461600|82|`93d6fa6a249061731688627acf5229b6188f3d6872c2ce317da14dac94289ccb`|
|Config `5Wcfjj1bY2bM88asdYwPwDffeiA2bxas6mXDDLX1Xwjj`|`GAjHRaC3XM98SoRoFy5B8P2ypTS1ndf1UPhXmjWgtPux`|5801360|1014|`4f135c9840bb65f5b034cdd26cadd3532dac8029aba8cfa0e4c18f2d4e55a494`|
|Round `23WaDJj9SEXtdhBeeqCMfX3LTYvciuXWGCVYNd7jdmoP`|`GAjHRaC3XM98SoRoFy5B8P2ypTS1ndf1UPhXmjWgtPux`|5176520|891|`f00e99aaf4c166c86e415d8da87f80f882c13d180ff8f4fe9db975c2a2a09c2a`|
|G0 `BUuuNvicRKF4ZFEUH4iS8xGzhAKQca3ou4U1s4NaBYoT`|`11111111111111111111111111111111`|983906600|0|`e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855`|
|Pending SOL `Hx4CLsvx2mB1JC57h3wyquk5ZamDNbjuDRDST3CQNRKK`|`11111111111111111111111111111111`|5000650240|0|`e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855`|
|Principal SOL `FyKuVXqSyntbgDSYhftth9GouEFifhM1PpFytnea1X44`|`11111111111111111111111111111111`|650240|0|`e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855`|
|Principal JitoSOL `4Rq5Ko33vPsP5LC4AdoqUYV3atxqie8UFvjqg5Bc1c9t`|`TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA`|1488440|165|`4ce2c57fd7f60186e2c9e8c953ebf0ea66e79a9a515e0df9f34a54210991396d`|
|Escrow `G5CPGXhihm1uTmXmzjAi1KKhCxMMQuBRY9KQD32mNeFf`|`11111111111111111111111111111111`|650240|0|`e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855`|
|Existing nonce `5fnA8B9cCfCfMrzoZqZAPLZt7CM3gy6F2mh4LxKspeSd`|`11111111111111111111111111111111`|1056640|80|`8e3ee7d9317b4fd59db91cc1fd19aed7c9ff0c3bb39493b20a1fe065fb17fce4`|

Current list header: tag2/max10000/count848; all848declared73-byte rows decoded,848Active/status0, all lastUpdateEpoch1080, no duplicate/zero votes or checked-balance overflow; active sum89818726635/transient0. Full730009-byte body SHAe8fd3e4cc79b7992a3b3dbc62d4e008e81a0598d394594fb5f1905f2ecc39c30. Unused capacity/residual bytes are hashed opaque, not falsely required zero. Current pool owner/topology and reserve Initialized200B/withdraw-PDA staker+withdrawer/default lockup, mintauthority/9decimals/no freeze are verified. The historical mutating transaction remains UNKNOWN; no claim that prior activity was malicious or harmless is needed to authenticate the current state.

## Measured call timings and relevant source

These are inclusive OUTER RPC-call-start→receipt and receipt→RPC-return measurements, including wrapper/Store work where executed. They are not network-only latency, exclusive guard timings or additive stage totals.

| Call | Outer start→receipt seconds | Receipt→RPC return seconds |
| --- | --- | --- |
|opening|1.700724950|0.279726947|
|simulation|1.205264591|0.284758300|
|original|0.971375080|0.287302727|
|before_sign|2.160783752|0.417992563|
|before_send|2.031580650|0.436502742|

Frozen ordinary client: `/home/jerem/piv1-pilot-checkpoints/m5-standing-functional-lifecycle-20261031-a/post-upgrade-stake-preparation/execute-single-stake-D098-CPI-reader.prepared.py`, SHA77fe0a84. Formatted excerpt from `OneStake.fresh` (lifetime refusal precedes account-view/receive-age completion):

```python
need(genesis == GENESIS
     and fee['value'] == self.p['quotedFeeLamports']
     and fee['context']['slot'] >= self.last_slot
     and valid['value'] is True
     and valid['context']['slot'] >= self.last_slot
     and int(self.life['currentBlockHeight']) <= height
         <= int(self.life['lastValidBlockHeight']) - 40,
     'fresh_original_cluster_fee_validity_40heights')
self.pre_accounts, self.pre_clock = self.view(accounts)
self.current = {'responseAt': at, 'height': height}
self.age()  # unchanged maximum1second
```

Production native boundary remains `tools/startup_release_operator.py:508`, SHAf91bb04e, with `deadline=time.monotonic()+20`. The C observer represents this budget without invoking it. Formatted source-order excerpt:

```python
self.full_guard()
self.fresh('before_sign')
self.store.current_record(reservation)
self.static_identity(); self.authorized(); self.age()
# Require COMPLETE cutoff <=0.9s; production limit remains1s.
# Backend-source check + unsigned capture/fsync + guards within20s.
while time.monotonic() - start < 20:
    time.sleep(max(0, min(.25, 20 - (time.monotonic() - start))))
self.full_guard()
self.checked(lambda: {'mode': 'SIGNATURE_VERIFY_NOT_EXECUTED_EXISTING_PASSING_EVIDENCE_REUSED'})
self.store.record('SEND_INTENT', {'mode': 'READONLY_UNSIGNED_NEVER_SEND', ...})
self.full_guard()
self.fresh('before_send')  # lifetime gate refuses here
```

No current-state caching across admissions, hash commitment change, repeated shared-row baseline adoption, native deadline shortening, audit cadence exception or nonce inheritance occurred. Full473held files/67452892B,106closure/29010846B and current source/Node/artifact/Store controls were retained at existing boundaries. `current-jito-structure.prepared.py` SHAc21f0802 adds full current list parsing; `execute-first-liquid-fresh.prepared.py` SHAbfd1f46a only extends baseline validation, inheriting all native/send/fresh methods. One actual848-list regression and four focused new-opening protection categories PASS; unchanged14prior categories/source evidence reused. No full suite, program build, upload, governance or initial stake was repeated.

## Reproducible evidence and consumed scopes

`R=/home/jerem/piv1-pilot-checkpoints/m5-first-liquid-fresh-state-20261106-b`; `E=R/prepare-readonly-c/public-evidence`; oldN=`/home/jerem/piv1-pilot-checkpoints/m5-first-liquid-lifecycle-20261106-a`.

Bcaps0 scopefcdd4fe1/fixed1794005977–6277 stopped before constructor/RPC on a missing standalone repository import path; rootnormal1/.278952s. Its original source/window/outputs remain unchanged and consumed. Narrow new C observer adds only the trusted repository import prefix and distinct namespace. ONE exact-root-environment/noargs prefix test reached the expected argument guard before constructor/RPC, .244907726s; noPYTHONPATH requirement or other infrastructure change. This local startup defect and correction are disclosed, not counted as a chain qualification PASS.

Actual C scope300ba986/fixed1794006291–6591, exactGOSHA6228b3ac; observer-v2 SHA1ccc05bdc11697810e5bb7369361016c344b62b5ff8581bc3c0f1ec2311e1c24/root-v2 SHA6f29955298ad9a38bfdb01015f9d5ebea61c1c8431c66e8f0e1cf5f84f7402b0. ALL caps0, no private profile/key access, native, successful VERIFY, signature, send, fee, live grant or live original. The unsigned PREPARED/SIGN_ATTEMPT/SEND_INTENT are explicitly QUALIFICATION_ONLY and may never be adopted for execution. Only5batches/22methods, no retry/resample; all25sidecars/fiveWAL/anchor preserved.

Independent actual STOP review `R/fresh-qualification-C-independent-actual-stop-review.public.json`, SHA `96d8dd6c9a64a2b161f34e2aaa5473247658fb67a5cf6b3a56f2c424d57116e3`;36-file inventory/25anchor leaves/5WAL/raw original+state+source audits. Failure SHAbee3c67e19d19f999b6c146950fea4b628592c6079b8dd9402e0c823ee7a8f1c; root terminal SHA2a5e5176cc9f1ca619095a41d318d081a53e8a6496d3d76df9e4645a57da5d9f. Prepared review2a9613aa and Bdisposition/Cstartup reviewdf29c487 retained. Exact critical raw evidence:

- `E/rpc-01-opening.response.json`: `b6839cc2f7f193e6eaececb464d764f26087e7fc7d439baac360fe22171a78d5`.
- `E/rpc-02-simulation.request.json`: `d65efbcbcc1da3002e815781d852f5b3733175efe2818666878ca57564a8ef51`.
- `E/rpc-02-simulation.response.json`: `5a082f2c1c347b45540bb182f33debc8b42ceadc7e42e631b17a76e5ca108c7d`.
- `E/rpc-03-original.response.json`: `1152b90c377b064680aa7dcc5267e23e7977dc7b3503877176e6b48b1df40192`.
- `E/rpc-04-before_sign.response.json`: `b0f29826914b7b51e87d2d7009d3c18896951bcb0437552458d5c8bc5ce1d99d`.
- `E/rpc-05-before_send.response.json`: `688ba04fc806a9fbde9677e5f1ab0ad0641545b26e8e4eb9dd0969584184d2b8`.
- `E/fresh-engine-inputs.public.json`: `b3289cbeafb8444255ba9e37f77e0986abfa23ecba8dc840b5dd9ec0300e01f1`.
- `E/0004-HOLD.json`: `546ba5f0b064bcbd9d53d2c71b7f28b953e0bea5a3797f03dc5bbb7d1d96195e`.

**Next bounded action:** review this measured whole-original lifetime deficit before any new qualification. A candidate must credibly retain the full native deadline, all mandatory audits/current checks, exact new original,1s receive rule and40heights; the unchanged path has not demonstrated that. Do not repeat it hoping a faster signer or quieter Jito will rescue it. No new policy exception is proposed or activated here. Existing D095/D099 nonce rights are limited/consumed and do not authorize liquid preparation, settlement or integration. If a nonce or different deadline is chosen later, its exact authority and security implications need their own explicit review/decision; none is inferred from D087.

No shared-Jito invariant exception is justified by this run. Preserve all135000held and every prior original/scope, includingK241/A839. No Mainnet, funding, new account/ProgramID, reinitialization, authority removal, automatic restake, cleanup or main publication. M4OPEN_UNWAIVED/publicM5NOTREADY; the real economic lifecycle remains incomplete.
