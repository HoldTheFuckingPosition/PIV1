# First liquid lifecycle: current-state refusal before signing

Source baseline `5da08d34b87b3a093618ff6e81219febb0bd1745`, `integration/piv1-testnet`, jerem/WeatherTrader2, 2026-11-06 UTC. **STOPPED; preparation BLOCKED, settlement and pending integration NOT STARTED.** Existing836Writes/SAMEProgram upgrade/initial10SOL stake/5SOL Deposit remain finalized. M4OPEN_UNWAIVED/publicM5NOT_READY. D087 remains valid; no new security exception or economic rule was introduced.

## Actual protected state and economics

Last complete finalized observation: context=Clock461437891, Unix1794004286, epoch1080. Config, round, principal/pending custody, official pool totals and mint, beneficiaries, G0 and nonce all equal the opening state. Historical7454364674JitoSOL units conservatively value10000182505lamports using pool4449472625757/3316738623763, mint supply equal; HWM10000000000, genuine gross182505. The5SOL contribution remains separately recognized pending, physical5000650240 including650240rent. Principal SOL remains650240rent; G0cash983906600. RoundIdle/sequence0, lastSuccessful=None/lastInsufficient=None, KIFperiod0/bitmap1/G0onlyactive/liability0. Both actual beneficiary destinations are System/nonexecutable/empty-data accounts holding650240 each; no recipient funding was needed.

| Economic stage | Actual result |
| --- | --- |
| First liquid preparation | BLOCKED before native signer/send; no successful preparation timestamp. |
| Settlement | NOT STARTED. Actual HTFP/team/KIF allocation and payment:0 each. |
| Pending integration | NOT STARTED. Recognized5SOL is still pending; HWM remains10SOL. |

The exact noncommitting preparation predicts107677HTFP/35588team/3650KIF/35588compound/2dust and146915pending→escrow. These are **simulated effects, not paid allocations**. Real preparation must derive them again from eligible current inputs. No ten-day clock starts from staking, the Deposit or a simulation; next successful-preparation timestamp/next+864000s remain UNKNOWN. No auto-restake or forced Jito withdrawal occurred.

## What passed, without a policy relaxation

`R=/home/jerem/piv1-pilot-checkpoints/m5-first-liquid-lifecycle-20261106-a`.

One authenticated native HTTP batch simulated the SAME exact unsigned1092-byte production message twice, returning five accounts each. Exact finalized context461435181/API4.4.0, replacement blockhash/lifetime, all execution fields/logs/CPI/CU, four overlapping full raw account rows and the six-account union matched. All six mutable/fee poststates were observed, including G0−5000 and unchanged principalSOL. Full source-derived Config/round, metadata, rent, exact System transfer and event passed965827CU. No sixth row was fabricated or inferred. Independent actual review `R/prepare-pair-independent-actual-review.public.json`, SHA `311fcc749312c4b1e78f4b308d8fbf560228c08a070082005e79d9f58531f777`. This does not qualify a live original's lifetime or receive age.

The dedicated first-stage client `R/execute-single-lifecycle.prepared.py`, SHAd63d6944, inherits unchanged ordinary77fe recent-hash/fresh/sign/send/Store controls and Deposit private-original proof/sticky failure protection. Full36 current accounts remain exact. Actual preparation Clock can replace only three existing Round integers and Config's optional timestamp, bound to receipt inclusion/current epoch/pre-send and post-readback Clock; every other byte remains exact. No nonce/governance/Loader timing exception is inherited.

Seven focused effects categories and seven changed-client categories passed; unchanged suites were not repeated. The latter includes realistic106closure files/29010846B and432held files, actual network-denied publicSDK BIND-once/zero-signature refusal, immutable original mutation refusal and dynamic Clock/custody/HWM/recipient mutation tests. Closure resource caps alone increased64files/32MiB→256files/64MiB to retain ALL prior60 closure bodies plus45 actual Deposit bodies and its review. Full historical audits remain at unchanged boundaries; individual reads remain bounded128MiB. Prepared source/HOST/owner review `R/first-liquid-prepared-independent-review.public.json`, SHA63a4f07e. No generic launcher, new framework, deployment repeat or optimization campaign.

## Exact failed scope and confirmed blocker

`R/prepare-liquid-a`, package `a61ed99688d6ecde4335477be06274bb23c1ccef72736936b4a2d9b688624026`; independent exact GO `bfcb29719ed3653f66e9e903ab93b51dc4e49623581d9a268bcfe10c7f11438a`. Fixed1794004171–1794004471, one original/native/signature/send maximum,10000fee/rent0/funding0/replacement0. Rights and once are CONSUMED; no reentry or renewal. An inert header builder initially refused duplicate placeholder text; only the four top-level owner literals were completed under the SAME anchor before the single reviewed invocation. No failed original was replayed.

The real `before_sign` response refused `full_causal_bytes_balances_owners_authorities_no_extra_effects` because ONLY these two nonClock rows changed from the opening:

| Shared Jito account | Exact retained difference |
| --- | --- |
| Reserve `CzKqc9cs4XpyG6y4peQgk3vBjPyqhktmfqaMuMBCXCqm` |4360657231602→4359655565362lamports, debit1001666240; full data unchanged. |
| Validator list `G5N6K3qW86GSkNEpywcbJk42LjEZoshzECFg1LNVjSLa` |730009bytes unchanged length; SHA0bc8dd06…→e8fd3e4c…; declared record count847→848. |

This is consistent with shared Jito activity. Its originating transaction and legitimacy are **UNKNOWN**; no transaction investigation was performed. The approved full-state guard correctly refused a baseline mismatch. The opening was1640Clock seconds older than this sample; this is NOT a measured1-second admission age. No current check was weakened or row ignored.

Original finalized reference461437864/H412811576/LVH412811726; before-signH412811603 leaves123heights, valid=true, exactfee5000/genesis correct. **This was not a demonstrated lifetime or freshness failure.** Complete before-sign receive-age admission was not reached. No finalized transaction receipt was obtained; matching PIV1 state is not a substitute for original finality or fee evidence.

Only original and before-sign RPC frames exist. Durable `0003-HOLD.json` records native0/nativeReserved1/send0/reserved5000/economicExposure0/currentOriginalFeeUNKNOWN. No native result, SEND_INTENT, signature or send frame exists. Root terminal `R/prepare-liquid-live-root-a/terminal.public.json`, SHA `ae0f0d6410c99a2666e683efe201115456f8c6ea57274c004c8434be0145b85b`: observed parent exit1/20.041895566s/no resource stop/orphan/cleanup issue. Owner killed worker withSIGKILL after its durableHOLD; worker normal termination is **not proved**. Preserve this distinction, the partial public stderr, raw frames, four WAL records, anchor and all controls; no recovery/retry was attempted. No process remains.

Independent actual STOP disposition: `R/first-liquid-live-stop-independent-actual-review.public.json`, SHA `827261bfea58979bd78eda69f3ad25a73cc40ec728b46071cd4d2f0abe179d6c`. It verifies30 retained files/ten anchor leaves/four legal WAL records, exact unsigned all-zero-signature original, current-state delta, held/closure/source pins and process termination distinctions. It grants no retry or new live scope.

## Funds, evidence and smallest next action

Confirmed fees remain4260000; actual native/Ed25519/send counts859/862/850. No new fee is confirmed. Keep all prior130000UNKNOWN/HOLD and this scope's additional5000reservation: **135000 held**, separate370000reserve,1056640lockednonce rent. Remaining unallocated mission fee pool is conservatively1645000 under the unchanged6m ceiling. Original fee/finality uncertainty is not cleared by the lack of a recorded send or the later process absence. Existing nonce is untouched; D099 is consumed and supplies no lifecycle nonce authority.

All old scopes/originals/HOLDs, includingK241/A839, and all current files remain preserved at their exact paths. Required artifact/source/Node/private-profile identities are unchanged. Only existing disposable profile metadata was accessed by the reviewed owner; no native signer/private signing-key invocation occurred. No Source access, new account, funding, irreversible authority change, cleanup, Mainnet or main publication.

**One next bounded action:** independently disposition this consumed refusal using its original evidence, then obtain ONE fresh coherent opening immediately before a distinct reviewed preparation package. Reuse unchanged client/tests/six-account simulation evidence only if every simulation-dependent economic input remains identical. Rebind only a NEW scope to that verified full baseline, with a NEW original and all ordinary1s/40/full-account guards; never reuse this scope or silently accept shared-account changes. The fresh public read must establish original expiry/eligibility where required; expiry is not inferred here from wall time. If an invariant-based alternative to full byte equality is needed, review its exact security-policy implications separately before implementation. No such relaxation is proposed or applied in this task.

Settlement may follow only a finalized preparation and independently verified complete effects; integration may follow only the exact Settled round. Delayed withdrawals require genuine uncovered outgoing need, protocol capacity/minima and real epoch/cooldown conditions. The functional lifecycle remains incomplete.
