# One additional Source funding transaction — OPEN / TESTNET ONLY

The D103 transfer finalized once:3079670000 lamports, signature2KJeRL88hqtZCAVimJ2z94AbKMkkGMQS6CTEWWi5ms1e8AcAcd5fEQKeHP51Myw2QDF4SxNtXMRkHqbddtvsmb2e, fee5000. Source329332063807→326252388807; G0983901600→4063571600. It is consumed and must never be replayed.

The original funding calculation omitted the NEW Squads transaction/proposal account rents. Existing reviewed compact upgrade is277B; transaction space378/rent2570480, six-member proposal space646/rent3931920. Buffer rent4056466360 + governance rents6502400 + existing4300000fee allowance + unchanged2805240G0floor =4070074000 required. Exact current shortfall6502400 lamports. No Buffer/upload/governance transaction has started.

Requested amendment ONLY: allow ONE additional Source4WKQg3Sm8bvHS8DBmxoiMMi4Fev4mJU6ZLwGSTg6jDna→G0BUuuNvicRKF4ZFEUH4iS8xGzhAKQca3ou4U1s4NaBYoT System transfer of the fresh verified shortfall, currently6502400 lamports. Maximum20330000 (unused part of the existing3100000000 cumulative cap), deduct any new G0 funding credits, no unnecessary funding. Source pays fee<=10000 separately; maximum1native/1Ed25519/1send with immediate recent blockhash/standard preflight/finalized receipt reconciliation. Preserve known Source floor259000005000 and all historical holds. No automatic retry/replacement after ambiguity.

At current quotes, cumulative additional funding becomes3086172400, below the unchanged3100000000 cap. The earlier finalized Source fee5000 remains separately accounted; D1024.3mnewfees/10mmission/860sends/864native-Ed and4100000000Buffer-rent cap remain unchanged. Governance account creation is part of the already authorized same-Program4-of-6 upgrade; this proposal changes ONLY the Source funding transaction count. No new accepted decision ID until explicit founder approval.

After finalized sufficient funding, execute the approved D102 upload/byte verification/Squads upgrade/fresh settlement/NET-floor reconciliation/dependent functional tests without routine reapproval. No PIV principal/pending/escrow funding, new wallet/Program, Mainnet, cleanup, unrelated authority or economics changes. Preserve all evidence and the first finalized transfer.

Exact receipt and resource quote: `/home/jerem/piv1-pilot-checkpoints/m5-practical-functional-tests-20261107-a/D103-source-funding/result.public.json` and `whole-upgrade-resource-quote.public.json`.
