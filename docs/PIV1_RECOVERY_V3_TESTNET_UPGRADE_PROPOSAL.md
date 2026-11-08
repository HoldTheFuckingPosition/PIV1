# Proposed D111 candidate installation — separate approval required

**UNAPPROVED / PROVISIONAL / SOLANA TESTNET ONLY. NO LIVE TRIGGER.**
This is ONE proposed same-Program upgrade plus governed Idle migration, not authority to execute. D111 authorized local development only. Existing D109/D110 and every previous scope remain consumed. No new decision ID is assigned.

## Exact candidate and effects

Source index **ee4e06070e0099f8f0d6497ba3521c3c24e4bea95259b15a53f2253af4b65b46**; stripped ELF **952664B / fd53a813aaa2153411539039eab75f47af1d07b5d9a30de49ad3a2524b5103aa**. [Implementation, ABI and local evidence](PIV1_BOUNDED_RECOVERY_IMPLEMENTATION.md). Artifact `E/recovery-candidate-final.stripped.so`, E=`/home/jerem/piv1-pilot-checkpoints/recovery-implementation-20261108-a/`.

Preserve existing Testnet Program `GAjHRaC3XM98SoRoFy5B8P2ypTS1ndf1UPhXmjWgtPux`, ProgramData `6HV5GQesbZpTyBVTiMu64UU4JgCX2D2s489sKt26xpd8`, Squads6guardians/4votes and Vault0 `A2UHpGyiCBdUZDMrofGswNiyS4ATD1ytBSiAsWNdxUTw`. Verify Testnet genesis `4uhcVJyU9pJkvQyS88uRDiswHXSCkY3zQawwpjk2NsNY`. No new Program, wallet, initialization, authority removal or economic instruction.

The retained capacity892352B is insufficient: extend SAME ProgramData by **60312B**, then install exact candidate. One deterministic Buffer **A2bcpT2WigxUxoVBqZ7zUwaii9B1XNLhKm4FKBajPuJn**, derived locally from G0 `BUuuNvicRKF4ZFEUH4iS8xGzhAKQca3ou4U1s4NaBYoT`, seed `d111-recovery-v3`, upgradeable Loader. This address is only derived, not created. Up to **997** canonical Writes, exact complete readback, handover to existing Vault0, genuine four-of-six upgrade. Buffer spill/rent returns only to G0.

A SECOND exact current4-of-6 stored action performs MR03 on canonical round `23WaDJj9SEXtdhBeeqCMfX3LTYvciuXWGCVYNd7jdmoP`: expected current **Idle V2/924B**, exact fresh complete old-body hash, external operational G0 rent payer, **V3/1199B**, full old prefix preserved except version. No Recovery resumption, leg, swap, distribution, staking, payment or contribution is included. If actual state is not Idle V2, STOP and revise the exact proposal; do not opportunistically migrate a live Recovery round.

## Proposed finite ceilings

Use D101 straightforward existing upload/Squads clients, standard preflight, fresh blockhash, immediate sign/send once, durable original and finalized reconciliation. No new launcher or historical qualification campaign. Two distinct governance indices must be freshly verified; do not reuse prior proposals. One technical window≤4h anchored once after approval/review/fresh opening; no automatic renewal. Maximum **1014 G0-funded sends /1024 native invocations /1024 Ed25519 signatures**, inclusive of Buffer,997Writes,handover,capacity extension and two sets of create/propose/4approve/execute. No blind retry, resign or replacement after uncertain results. Retain at least2GiB free plus512MiB working headroom; new live evidence≤256MiB, stopping before its cap.

| Resource | Proposed ceiling / retained-rate estimate |
| --- | ---: |
| New G0 fees, all guardian operations included |6000000 lamports|
| Optional ONE Source funding transaction fee |10000 lamports|
| New Buffer rent estimate |4840371320 lamports|
| Same ProgramData extension rent estimate |306384960 lamports|
| Round extension rent estimate |1397000 lamports|
| Two Squads action/proposal accounts combined rent cap |25000000 lamports|
| New aggregate rent/capital ceiling |5173153280 lamports|
| Mission confirmed-fee ceiling if approved |23490000 lamports|

Rent estimates use the last retained5080-lamport per-byte exemption rate, NOT a fresh quote. Future approval must retain these caps; recheck current exact quotes, balances and source reservations before any funding/signature. Failed sufficiency/unsupported Loader extension means STOP, no silent cap increase or substitute deployment. Buffer rent is refundable on successful upgrade; ProgramData/round/Squads rent is separate from fees and PIV economics.

If G0's fresh available balance is insufficient, propose at most **ONE Source→G0 transfer of the actual shortfall, maximum700000000 Testnet lamports**, Source `4WKQg3Sm8bvHS8DBmxoiMMi4Fev4mJU6ZLwGSTg6jDna`, max10000Sourcefee/one signature/send. Reuse existing authorized signing infrastructure; verify no Source reservation or competing transfer. No transfer if unnecessary; deduct any intervening funding. This would be a NEW explicit allowance, not reuse of a consumed Source authorization. Classify exclusively EXTERNAL OPERATIONAL FUNDING.

At last recorded G04485518608, aggregate rent estimate5173153280 +G0fees6000000 +135000HOLD+370000reserve gives a provisional shortfall **694139672**. This is arithmetic on retained balances, not current sufficiency or authority. Never fund from principal, pending, escrow or KIF. Preserve nonce rent1056640 and all prior uncertainty; no nonce operation.

## Completion and stop

After each finalized effect, reconcile its exact receipt before the dependent step. Verify deployed bytes/zero tail, unchanged Vault authority and full protected state; after MR03 verify exact preserved V2 prefix and Idle V3 with no active recovery receipt. Preserve HWM15000045712, historical JitoSOL11181318975/native21, prior summary/cadence/earned KIF and all holds, subject to fresh lawful observation rather than hardcoded overwrite. If any outcome is ambiguous, retain original and HOLD; do not replace it. STOP after upgrade/migration reconciliation and checkpoint publication on `codex/piv1-hybrid-liquidity`. No main publication/Mainnet/live recovery is proposed.
