# PIV1 Task 0.4 — JitoSOL technical validation

Date: 2026-08-29 UTC

Branch: `spike/task-0.4-jito-validation`

Status: `COMPLETE`

The direct deposit, direct JitoSOL contribution, delayed stake withdrawal,
PDA authority, immediate deactivation, epoch-delayed finalization, stake-account
closure, and fixed native-SOL escrow receipt paths are `CONFIRMED BY PUBLIC
TESTNET`. The real withdrawal stake deactivated in epoch `1018`; finalization
succeeded in epoch `1021` and moved its complete recoverable balance, including
epoch rewards and stake-account rent, to the fixed PIV escrow.

## Public Testnet round-0 finalization on 2026-08-29

The continuation began from accepted commit
`be5496bf7d6858e1208f274b4dad226bbf097455` on the clean
`spike/task-0.4-jito-validation` branch. The persistent fee-payer and program
keypairs still derived to, respectively,
`4WKQg3Sm8bvHS8DBmxoiMMi4Fev4mJU6ZLwGSTg6jDna` and
`BbHNk57mVmZmfH1HfiyPaapwJ5FVPe3c7MsidbGVewG6`.

At finalized slot `435,648,912`, Testnet was in epoch `1021`. Solana's stake
inspection reported round-0 stake PDA
`5VwP8uSSAPur125jTYPj4mFwXmrCCN9nLmY79NMJL8KF` as undelegated. Its staker and
withdrawer were both the fixed PIV authority
`EnhPAyW2g7JZHd4dbBjcu2uwaVR1E6tf9ryKz6ZL6tFL`; its deactivation epoch remained
`1018`. The program-owned round account
`G6wUeeEQZkwqKvqdMTBaqrexo5irqcjYTtofRbef2yct` still identified round `0`, the
same stake PDA and configuration, and status byte `1` (`Deactivating`). It had
not already been finalized.

The only submitted continuation command was:

```sh
npx tsx scripts/lifecycle.ts finalize testnet 0
```

Finalization signature
`2pfoSHp1SXuTq5XSheTyrws2Ewb5CQN8VNDCJN6v2rsCvyaHvxh94ynw6WuoWBmBRJDxEHaVSGy9k7XvYyxNnY2t`
succeeded at slot `435,649,022`, epoch `1021`, with block time
`2026-08-29T20:56:51Z`. It consumed `28,453` CU and charged only the Testnet fee
payer a `5,000`-lamport fee.

Finalized transaction balance arrays reconcile exactly:

- withdrawal stake: `1,004,623,871 -> 0` lamports, and the account is closed;
- fixed SOL escrow: `890,880 -> 1,005,514,751` lamports;
- exact escrow delta: `1,004,623,871` lamports;
- delegated value at withdrawal: `1,001,324,424` lamports;
- epoch rewards before finalization: `1,016,567` lamports;
- delegated value including rewards: `1,002,340,991` lamports;
- recovered stake-account rent: `2,282,880` lamports;
- reconciliation: `1,001,324,424 + 1,016,567 + 2,282,880 = 1,004,623,871`;
- caller: `1,466,319,727 -> 1,466,314,727` lamports, exactly the transaction
  fee and no custody receipt.

After finalization, the round remains program-owned and status byte `2`
(`Finalized`). A signed, non-broadcast replay simulation failed with
`WrongRoundStatus` (custom error `6026`) after `13,565` CU. The closed stake PDA
and terminal round state independently prevent replay. Persistent lifecycle
state remains mode `0600` outside Git and now records `complete: true` plus the
finalization transaction and account deltas.

## Funded public Testnet lifecycle on 2026-08-26

### Accepted baseline and preflight

Commit `41791750561f6aea1405b843f532ecca04e33b04` is the direct child of the
accepted funding commit `04105ce11c4a09875120c9d7df688bc0cf00c950`. Its
patch contains only the accepted project-identity, ignore-rule, Testnet
documentation, and balance-reporting changes. The spike branch was
fast-forwarded without a merge, rebase, amend, or history rewrite.

The finalized fee-payer balance was exactly `6,000,000,000` lamports. The
current complete-lifecycle requirement was recalculated as `4,554,230,273`
lamports, leaving `1,445,769,727` lamports of headroom. The change from the
earlier requirement is the current `1,325,750`-lamport direct-client deposit
needed to mint the `1,000,000` JitoSOL units used for the direct contribution.

At `2026-08-26T22:00:53.077Z`, Testnet was in epoch `1018`; the pool and all
`1,129` decoded validator-list entries had `last_update_epoch = 1018`. The pool
totals were `4,367,920,287,905` lamports and `3,294,679,564,826` pool-token
units. The dynamic minimum was `755,045,269` JitoSOL units for exactly
`1,000,000,000` delegated lamports. No permissionless update transaction was
needed or sent.

### Probe deployment

The accepted `504,896`-byte artifact has SHA-256
`4c0210a706d09cae0c7e94e469b60ee0cb4647feebd9baa9c23da3fa21081420`.
It matches the accepted source, persistent program keypair, `declare_id!`,
and `Anchor.toml`. No rebuild was required.

The probe was deployed at slot `434,307,781`:

| Item | Public Testnet evidence |
| --- | --- |
| Program | `BbHNk57mVmZmfH1HfiyPaapwJ5FVPe3c7MsidbGVewG6` |
| Program owner | `BPFLoaderUpgradeab1e11111111111111111111111` |
| Program rent / space | `1,141,440` lamports / `36` bytes |
| Program-data | `E32nS62CyGunPeuYG3vFxbXAypwCC1MoHVRiLHbirw24` |
| Program-data rent / space | `3,515,280,240` lamports / `504,941` bytes |
| Upgrade authority | `4WKQg3Sm8bvHS8DBmxoiMMi4Fev4mJU6ZLwGSTg6jDna` |
| Final deployment signature | `wUVTW7GcYZe7u6ZURDfGmmvRFM4KFZsUrGW3mHtgTdzEBpUGCNmoWZhgTKQry8qNTfRcNokEgxtGW5BpD5wUS58` |
| Final deployment fee / CU | `10,000` lamports / `2,670` CU |

The official RPC throttled the first RPC-send deployment after it created
buffer `8WqC99w6wC5AgGs3YULQcTwCFremfjCKe7LBE34Z5a1g` and wrote `19,232`
artifact bytes. The buffer remained loader-owned, controlled by the preserved
fee payer, and funded with `3,515,280,240` lamports. The remaining bytes were
written through the TPU path; the full buffer hash matched the accepted
artifact before final deployment. The loader then closed the buffer and used
its rent for program data. Deployment used `501` successful loader
transactions (`1` buffer creation, `499` writes, and `1` final deployment) and
paid exactly `2,515,000` lamports in transaction fees. All signatures, slots,
error statuses, exact fees, artifact hashes, owners, rents, and the observed
buffer-creation/final-deployment logs and compute units are recorded in
`PIV1_TASK_0_4_TESTNET_DEPLOYMENT.json`. The public RPC rate-limited individual
`getTransaction` reads for the 499 write transactions, so their per-write
logs and compute-unit fields remain explicitly unavailable rather than
invented.

### Live custody lifecycle

The smallest practical direct-client deposit used the existing fee payer
directly, not the helper-generated temporary signer: `1,325,750` SOL lamports
minted exactly `1,000,000` JitoSOL units. This basic direct deposit funded only
the controlled contribution demonstration; the PIV CPI path used the required
slippage-protected instruction.

The PIV SOL vault was funded with `1,004,174,763` lamports: the exact
`1,001,001,003`-lamport CPI input, `2,282,880` lamports of withdrawal-stake
rent, and the retained `890,880`-lamport zero-data rent reserve. The fixed
escrow was funded with its separate `890,880`-lamport rent exemption. The
configuration account rent was `2,596,080` lamports, each JitoSOL token
account rent was `2,039,280`, and the round-state rent was `1,642,560`.

`DepositSolWithSlippage` moved `1,001,001,003` lamports from the system-owned
PIV SOL-vault PDA to the official reserve and minted exactly `755,045,269`
JitoSOL units into the PIV-controlled ATA, equal to the decoded conservative
minimum. Both SOL deposit fees were zero at the observed pool configuration.
The caller then contributed exactly `1,000,000` JitoSOL units directly into
the same PIV vault with no DEX, swap, or additional signer.

Validator-list entry `485` was current and active. Its vote account was
`vouNpQ4b6mZRAKHG312QrBhbG3t5QdBLRuWXr2YYevo`, and its standard pool stake
account derived to `GTpapQCpq64AhLskXapChCApM7XiWAZ2GUjrTfbXRC4D` with
`273,765,741,767` lamports before withdrawal.

Round `0` created deterministic stake PDA
`5VwP8uSSAPur125jTYPj4mFwXmrCCN9nLmY79NMJL8KF`. The combined
`WithdrawStakeWithSlippage` and `Deactivate` transaction consumed
`756,045,269` JitoSOL units:

- withdrawal fee: `756,046` pool-token units;
- burned pool tokens: `755,289,223` units;
- delegated stake output: `1,001,324,424` lamports;
- stake balance including rent: `1,003,607,304` lamports;
- validator stake after: `272,764,417,343` lamports;
- PIV JitoSOL vault after: `0` units;
- PIV SOL vault after paying stake rent: `890,880` lamports;
- staker and withdrawer: `EnhPAyW2g7JZHd4dbBjcu2uwaVR1E6tf9ryKz6ZL6tFL`;
- voter: `vouNpQ4b6mZRAKHG312QrBhbG3t5QdBLRuWXr2YYevo`;
- deactivation epoch: `1018`.

The fee payer was the permissionless caller for the public lifecycle. It paid
all transaction fees and temporarily owned only the source JitoSOL account;
it never became token-vault owner, stake staker, stake withdrawer, or final SOL
destination. It did remain the explicitly configured Testnet probe upgrade
authority, which is separate from live asset custody.

### Lifecycle transaction evidence

| Stage | Signature | Slot | Fee | CU | Result |
| --- | --- | ---: | ---: | ---: | --- |
| Direct-client deposit | `Amvd6NaE3e9CUoFScptWSH6iHZQpFrseFkCNk1ZNWK4YiawjPSei4QP7fuAYjdt6E6zrQd8vMYPSwpdYmBkAgS4` | 434,309,386 | 5,000 | 24,811 | success |
| Fund PIV accounts | `2QgqDhj5WAuWEBsucaNwkc5AZ6g7qgk45DJhsZo9PxhNJyDjG2R4wLWiThkX4Lx2cAqTUeAcrc5hni1vFwTzJ2z7` | 434,309,396 | 5,000 | 13,818 | success |
| Initialize probe | `3WYEEBGP1H92iVYXNmvGxN28W4GwB9Y8t88nehxQeXXtPefrMzAyztNEHbe7tfD7YXc1iD7fE6GuyDbHB81sdBEu` | 434,309,404 | 5,000 | 27,253 | success |
| PIV SOL-to-JitoSOL CPI | `4FUS4JLJdo6WBaS1sDLVEhfiEvmiaeH99TFqjGyzRzqupPV2psT7vGA3epsK8hLfysfwU654PoEDbEWHqzetD46J` | 434,309,550 | 5,000 | 32,243 | success |
| Direct JitoSOL contribution | `61xkvvU1grZnWRrVGGJ3xtEuuRbfrBChbaGBEpKV56YFBQfnst3HwHX73GELnFWGnxhAWKWroxyrEk8c2WCXQoeK` | 434,309,559 | 5,000 | 10,312 | success |
| Withdraw stake + deactivate | `3ScMY9GmtN3KCMoTbWc8LFQLhtsUhAX8kVtymBJB11VArsGkMbrxJbZgvWNrQod1P7VAdh5fgwBkffjm7xmeve5F` | 434,309,569 | 5,000 | 158,952 | success |
| Premature finalization | `5WLpPa6wQqG2XN1R18MSFBVoJNdgNxGWNdf7Gex1MPuzAseU1PU1jkWML7qiLxtwHUUz78bj58iuxa8xkPMLHBLZ` | 434,309,682 | 5,000 | 17,577 | expected custom error `6024` |
| Finalize round 0 | `2pfoSHp1SXuTq5XSheTyrws2Ewb5CQN8VNDCJN6v2rsCvyaHvxh94ynw6WuoWBmBRJDxEHaVSGy9k7XvYyxNnY2t` | 435,649,022 | 5,000 | 28,453 | success |

Lifecycle transactions paid `40,000` lamports and consumed `313,419` CU in
total, including the expected failed same-epoch finalization and the successful
epoch-delayed finalization. The finalized fee-payer balance after the lifecycle
is `1,466,314,727` lamports. The fixed SOL escrow contains `1,005,514,751`
lamports after receiving the complete `1,004,623,871`-lamport withdrawal-stake
balance.

Completed lifecycle state is mode `0600` at
`/home/jerem/.local/share/piv1/task-0.4-jito/testnet-lifecycle.json`.

## Public Testnet continuation on 2026-08-10

This continuation resumed the accepted commit
`a57b67fd7ca04f87ad34f21bad1ce4ec01c72b2b` without repeating the accepted
local validation. The preserved fee-payer and probe-program keypairs still
derive, respectively, to `4WKQg3Sm8bvHS8DBmxoiMMi4Fev4mJU6ZLwGSTg6jDna`
and `BbHNk57mVmZmfH1HfiyPaapwJ5FVPe3c7MsidbGVewG6`. The latter also matches
the built program keypair and `declare_id!`. The public Testnet probe account
remains undeployed.

At `2026-08-10T11:34:00.309Z`, public Testnet was in epoch `1002`. The pool's
`last_update_epoch` was `1002`, all 1,129 decoded validator entries were current,
and the aggregate pool state was current. No permissionless pool-update
transaction was required or sent.

The minimum funding target was calculated as `4,554,218,843` lamports:

| Component | Lamports |
| --- | ---: |
| Program account rent, 36 bytes | 1,141,440 |
| Program-data rent, 504,941 bytes | 3,515,280,240 |
| Conservative deployment transaction-fee budget | 3,000,000 |
| PIV SOL-vault deposit for 761,611,375 JitoSOL units | 1,001,001,003 |
| Direct-client SOL deposit for 1,000,000 contributed JitoSOL units | 1,314,320 |
| Withdrawal stake-account rent | 2,282,880 |
| Retained zero-data SOL-vault rent | 890,880 |
| Fixed zero-data SOL-escrow rent | 890,880 |
| PIV and caller JitoSOL token-account rent | 4,078,560 |
| Configuration and one round-state rent | 4,238,640 |
| Lifecycle transaction-fee budget | 100,000 |
| Small operational margin | 20,000,000 |

The calculation used the current pool totals of `4,328,538,446,106` lamports
and `3,293,367,448,422` pool-token units. The newly decoded minimum valid
withdrawal input was `761,611,375` units, yielding exactly `1,000,000,000`
delegated lamports. The planned vault input added a 1,000,000-unit margin and
would have yielded `1,001,313,006` delegated lamports. The temporary deployment
buffer requires `3,515,224,560` lamports, but the upgradeable loader drains it
back to the payer before funding the program-data account, so it is not an
additional permanent rent allocation.

Six bounded official `https://api.testnet.solana.com` `requestAirdrop` attempts
requested `1`, `1`, `1`, `1`, `0.5`, and `0.1` SOL, with capped exponential
backoff and a confirmed balance read after each attempt. The first returned an
internal faucet error; the remaining five returned HTTP 429 with the official
RPC reporting that the daily limit was reached or the faucet was dry. The
verified final balance at `2026-08-10T11:40:07Z` was `0` lamports. No alternative
credentialed, interactive, paid, browser-automated, or Mainnet funding source
was used. The continuation therefore remains `BLOCKED_LIVE_TESTNET_FUNDING`.

## Scope and safety

This spike validates technical custody only. It does not implement the PIV1
distribution split, recipients, guardians, governance, migration, or a
production state machine. It uses no DEX, Jupiter route, instant exit, or
stake-deposit interceptor.

No Mainnet transaction, deployment, fund movement, or authority transfer
occurred. Mainnet access was read-only. No Mainnet, personal, or production
keypair was created or used. Isolated Testnet-only fee-payer and probe-program
keypairs were generated outside Git. They remain mode `0600` under a mode
`0700` persistent directory. The upgradeable loader required one ephemeral
Testnet buffer signer; the CLI created it in memory, it was never stored, and
its public buffer account closed during deployment. Testnet SOL moved only
through the explicitly authorized probe lifecycle. No secret material was
printed or committed.

## Authoritative and official sources

Repository authority was applied in this order: `PIV1_DECISIONS.md`,
`PIV1_MASTER_SPEC.md`, then `PIV1_CODEX_EXECUTION_PLAN.md`. The older plan's
Devnet instruction conflicts with the newer Task 0.4 cluster correction.

Primary sources inspected:

- [Jito stake/unstake reference](https://github.com/jito-foundation/jito-stake-unstake-reference),
  commit `b553e90d39e1ff583011dab344a11b5d9bfd284c`.
- [Jito omnidocs](https://github.com/jito-foundation/jito-omnidocs), commit
  `14df8bb2f7169328d984393b0090b8cc32863e45`.
- Exact published `spl-stake-pool = 2.0.3` source from the local Cargo cache;
  `.cargo_vcs_info.json` identifies commit
  `864ba3c1c564cc270ca62b6e6b558f57538ae092` (`program@v2.0.3`).
- [Official stake-pool repository](https://github.com/solana-program/stake-pool),
  inspected checkout `5b8de048f2e6cdf2c9b75387300421abe9ec7704`.
- [Agave](https://github.com/anza-xyz/agave) 4.2.0 source identity
  `ac82b5d438b0c2303dc7169f52c748977713a111` for the local-validator
  stake-history diagnosis.
- [Solana clusters](https://solana.com/docs/references/clusters),
  [Stake Program](https://solana.com/docs/references/staking/stake-program),
  and official public RPC account data.

The Jito reference's assisted SOL deposit calls the SPL client helper, which
creates one ephemeral funding signer. Its manual SOL deposit instead passes the
wallet directly as the funding signer. The assisted/manual delayed withdrawal
paths generate client-side transfer-authority and stake-destination keypairs.
`CONFIRMED BY LOCAL TEST`: those are client implementation choices, not protocol
requirements; deterministic program PDAs successfully replaced them.

The Jito interceptor is used by the official reference for deposits of existing
stake accounts. Direct native SOL uses the ordinary SPL stake-pool
`DepositSol` path. Adding the interceptor to PIV1 native SOL deposit is
`REJECTED`.

## Cluster resolution

`CONFIRMED BY LIVE TEST`: the official Jito reference explicitly supports
Mainnet and Testnet and exposes the same public pool and mint constants on both.
Testnet is the officially supported non-Mainnet target for this spike.

A pool and mint at those same addresses currently exist on Devnet, but the
decoded Devnet pool was 19 epochs stale (`last_update_epoch = 1097`, current
epoch `1116`) and its topology differs. Its mere existence is not evidence of
current official support or operability. Using Devnet for Task 0.4 is
`REJECTED`.

Required execution-plan correction (`PROVISIONAL` until the founder approves a
canonical documentation edit): replace the Task 0.4 references to “Devnet”
with “the current officially supported Jito non-Mainnet cluster, resolved from
official sources and verified on-chain; Testnet as of 2026-08-10.” Require the
pool, mint, related accounts, owners, and current epoch state to be decoded at
execution time. No canonical decision or master-specification file was edited.

## Verified Testnet topology

The following was read from `https://api.testnet.solana.com` at
`2026-08-10T02:06:28.463Z` and is `CONFIRMED BY LIVE TEST`:

| Item | Value |
| --- | --- |
| Stake pool | `Jito4APyf642JPZPx3hGc6WWJ8zPKtRbRs4P815Awbb` |
| JitoSOL mint | `J1toso1uCk3RLmjorhTtrVwY9HJ7X8V9yYac6Y7kGCPn` |
| SPL stake-pool program | `SPoo1Ku8WFXoNDMHPsrGSTSG1Y47rzgn41SLUNakuHy` |
| Legacy SPL Token program | `TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA` |
| Validator list | `G5N6K3qW86GSkNEpywcbJk42LjEZoshzECFg1LNVjSLa` |
| Reserve stake | `CzKqc9cs4XpyG6y4peQgk3vBjPyqhktmfqaMuMBCXCqm` |
| Manager fee account | `8yoigZfzZ1nNaadumY9uPVD118225UYHTDpmjpr2nrSa` |
| Withdraw-authority PDA | `6iQKfEyhr3bZMotVkW6beNZz5CPAkiwvgV2CTje9pVSS` |
| Stake deposit authority | `74opVa3v51hUmTrsZn8YusZw4fXB16vGQY4WYHt9UegR` |
| SOL deposit authority | none |
| SOL withdrawal authority | none |
| Preferred deposit/withdraw validator | none / none |
| Epoch / pool last update | `1002` / `1001` (stale) |
| Total pool lamports | `4,325,464,231,449` |
| Pool-token supply | `3,293,273,887,802` |
| Exchange ratio | `4,325,464,231,449 / 3,293,273,887,802`, about `1.313423777922068` SOL per JitoSOL |
| Epoch fee | `4/100` |
| Stake deposit fee | `0/0` (zero) |
| Stake withdrawal fee | `1/1000` |
| SOL deposit fee | `0/0` (zero) |
| SOL withdrawal fee | `1/1000` |
| Referral percentages | stake `0`, SOL `0` |
| Minimum stake delegation | `1,000,000,000` lamports |
| 200-byte stake rent exemption | `2,282,880` lamports |

The stake-pool account and validator list are owned by the executable stake-pool
program. The mint and manager fee account are owned by the legacy Token program;
both use the JitoSOL mint. Mint authority is the derived pool withdraw authority
and mint supply equals pool state supply. The reserve is a 200-byte initialized
Stake Program account whose staker and withdrawer are the pool authority.

The validator list decoded to 1,129 real entries, 699 active entries, and 1,120
entries updated for epoch 1002. Because nine entries and the aggregate pool
state were not current, deposit/withdraw must reject until permissionless pool
maintenance finishes. This exact staleness is a point-in-time observation, not
a permanent topology property.

Read-only Mainnet comparison at epoch 1014 found the same pool, mint, programs,
withdraw authority, fee fractions, and absent SOL authorities/preferences, but
different validator list (`3R3nGZpQs2aZo5FDQvd2MUQ6R7KhAPainds6uT6uE2mn`),
reserve (`BgKUXdS29YcHCFrPm5M8oLHiTzZaMDjsebggjoaQ6KFL`), and stake-deposit
authority. No Mainnet transaction was made.

## Probe architecture

`CONFIRMED BY LOCAL TEST`: program ID
`BbHNk57mVmZmfH1HfiyPaapwJ5FVPe3c7MsidbGVewG6` implements:

| Role | Address / derivation |
| --- | --- |
| Configuration | `7jUjUtnnR1YGg8Ru25Qsxo5H3vJiqq48gUCHaa5wdbbC`, seed `config` |
| PIV authority | `EnhPAyW2g7JZHd4dbBjcu2uwaVR1E6tf9ryKz6ZL6tFL`, seed `authority` |
| System SOL vault | `GHb7nYSV9h4REwhK2ahUnHxA9PsiCoU3qsT6tYob6BDu`, seed `sol-vault` |
| JitoSOL vault ATA | `Ck4eDSzRWPXtx11GPccHFng2a7ScuupUvZCudDxdTptD`, owner PIV authority |
| Fixed SOL escrow | `CreDKMV4QaacjtPBZRMYgcyCtrYajoT5PLgQWKnSWzeQ`, seed `sol-escrow` |
| Round state | seeds `round` + little-endian `u64` round |
| Withdrawal stake | seeds `withdrawal-stake` + little-endian `u64` round |

The SOL vault and escrow are empty-data, system-owned PDAs. The authority is an
address-only PDA. The round counter is monotonic, and round/stake accounts are
created with `init`, preventing replay. Every Jito account is bound through
decoded pool state plus fixed pool, mint, and program IDs.

## Direct SOL deposit and JitoSOL contribution

`CONFIRMED BY LOCAL TEST`: a system-owned SOL-vault PDA signed
`DepositSolWithSlippage` through `invoke_signed`. For 1,200,000,000 lamports:

- SOL vault: `2,500,000,000 -> 1,300,000,000`;
- official reserve clone: `1,265,258,019,074 -> 1,266,458,019,074`;
- PIV JitoSOL vault: `10,000,000 -> 923,642,664`;
- JitoSOL output: `913,642,664`, exactly the conservative pool calculation;
- transaction: 32,243 CU, 5,000 lamport fee, 603 serialized bytes, 15 message
  account keys.

The permissionless caller was an ephemeral fee-payer
`FUrLuphF3UMeKdNnVnL9PVpnpiHtmMgrm1SGSqFU1XJt`; it received no JitoSOL and
held no custody authority.

The direct-client control used the official `@solana/spl-stake-pool` helper:
1,050,000,000 SOL lamports produced 799,437,331 JitoSOL units, used one helper
generated funding signer, consumed 11,895 CU, and paid 10,000 lamports for two
signatures. This confirms the basic `DepositSol` semantics locally; it does not
replace the CPI test.

`CONFIRMED BY LOCAL TEST`: transferring 10,000,000 JitoSOL units directly from
a caller-owned JitoSOL account into the PIV ATA consumed 10,312 CU. The vault's
decoded token owner remained the PIV authority PDA. No swap or DEX was used.

## Delayed withdrawal, authority, and deactivation

The selected source was validator-list entry 485:

- vote account `vouNpQ4b6mZRAKHG312QrBhbG3t5QdBLRuWXr2YYevo`;
- deterministic validator stake
  `GTpapQCpq64AhLskXapChCApM7XiWAZ2GUjrTfbXRC4D`;
- observed balance before withdrawal `268,734,152,043` lamports.

`CONFIRMED BY LOCAL TEST`: round 1 deterministically created stake PDA
`J8q9kbh7GFbzm52htMPdtGY1th9PmLH2swPmimoNk244`. The system SOL vault signed
its creation and paid 2,282,880 rent. The PIV authority PDA signed
`WithdrawStakeWithSlippage`, supplied both the new stake authority and token
transfer authority, and then signed `Deactivate` in the same transaction.

Exact withdrawal evidence:

- token input removed from PIV vault: `763,131,020` units;
- 0.1% withdrawal fee (ceiling): `763,132` units;
- pool tokens burned: `762,367,888` units;
- delegated stake output: `1,001,312,111` lamports;
- stake account balance including rent: `1,003,594,991` lamports;
- validator stake after: `267,732,839,932` lamports;
- JitoSOL vault after: `160,511,644` units;
- staker and withdrawer: PIV authority PDA;
- deactivation epoch: 1001; voter remained the selected vote account;
- transaction: 156,453 CU, 5,000 lamport fee, 768 bytes, 20 message account
  keys.

Same-transaction deactivation succeeded, so a second transaction is not
required in the demonstrated topology. A separate permissionless
`deactivate_withdrawal_stake` instruction remains implemented for operational
fallback. Whether future runtime/CU changes make the split preferable is
`PROVISIONAL`; it is not a current protocol constraint.

## Native SOL finalization

`CONFIRMED BY LOCAL TEST`: finalization in deactivation epoch 1001 was rejected
with `StakeNotDeactivated`. After advancing the preserved ledger to epoch 1002,
the PIV authority signed Stake Program `Withdraw` and sent the entire
1,003,594,991-lamport stake balance to the fixed SOL escrow:

- stake account: `1,003,594,991 -> 0` and closed;
- SOL escrow: `4,173,760 -> 1,007,768,751`;
- recovered value includes the 2,282,880-lamport stake rent;
- transaction: 27,323 CU, 5,000 lamport fee, 436 bytes, 10 message account
  keys.

A separate initialized-never-delegated round-0 stake proved same-epoch final
withdrawal: 3,282,880 lamports moved to the fixed escrow, the stake account
closed, creation consumed 35,014 CU, and finalization consumed 24,333 CU.

The initiator and finalizer were different permissionless callers. Neither was
a stake authority or destination owner. No keeper keypair is required beyond a
normal top-level fee-payer signature.

## Instruction choice and complete CPI metas

The basic and slippage-protected constructors use identical account metas; the
protected forms additionally encode a minimum output. Unit tests compare their
serialized instructions. The basic client deposit and protected CPI deposit
were both exercised. `CONFIRMED BY LOCAL TEST`: use
`DepositSolWithSlippage` and `WithdrawStakeWithSlippage` in production. Basic
variants are `REJECTED` for production because they provide no transaction-time
output floor.

`DepositSolWithSlippage` ordered metas are: mutable pool; read-only pool
withdraw authority; mutable reserve; mutable funding SOL account **signer**;
mutable destination pool-token account; mutable manager fee token account;
mutable referrer fee token account; mutable pool mint; read-only System Program;
read-only Token Program. Testnet has no optional SOL deposit-authority signer.
The PIV SOL-vault PDA is the funding signer. The probe safely aliases the
zero-referral account to the manager fee account.

`WithdrawStakeWithSlippage` ordered metas are: mutable pool; mutable validator
list; read-only pool withdraw authority; mutable validator stake source;
mutable new stake destination; read-only new stake authority; read-only pool
token transfer authority **signer**; mutable PIV JitoSOL vault; mutable manager
fee token account; mutable pool mint; Clock; Token Program; Stake Program. The
PIV authority PDA fills both authority roles and signs through `invoke_signed`.

Stake-PDA creation calls System Program `CreateAccount`: the SOL-vault PDA and
new withdrawal-stake PDA both sign through their seeds. Deactivation passes
stake, Clock, PIV authority signer, and Stake Program. Finalization passes stake,
fixed SOL escrow, Clock, Stake History, PIV authority signer, and Stake Program.

## Fees, rounding, and technical minimum

All math uses checked integers. Exchange conversion floors outgoing value; the
configured fee is ceiling-rounded, matching `spl-stake-pool 2.0.3`:

```text
fee(x) = ceil(x * stake_withdrawal_fee_numerator / denominator)
net_pool_units(x) = x - fee(x)
stake_lamports(x) = floor(net_pool_units(x) * total_lamports / pool_token_supply)
minimum_input = least x such that stake_lamports(x) >= runtime_minimum_delegation
```

At the inspected Testnet state, the exact minimum is 762,131,020 JitoSOL base
units. It yields 1,000,000,001 stake lamports; one unit less yields 999,999,999
and must be rejected. This threshold is dynamic and must never be hard-coded.
The 2,282,880 stake rent is funded separately from the PIV operational SOL
reserve, so total immediately available SOL resources are 1,002,282,880
lamports while the JitoSOL gate targets only delegated stake value.

SOL deposit fee is currently zero; deposit output is
`floor(lamports_in * supply / total_lamports)`. Stake withdrawal fee is paid in
pool-token units and is included before computing the output floor. Current fee
values are observations, not protocol constants.

Production slippage tolerance is `OPEN` for founder approval. The safe
mechanism is confirmed: calculate an expected conservative output from a
current decoded pool, reduce it only by an explicitly approved tolerance, pass
that as the minimum, and recheck observed balance deltas after CPI.

## Local and adversarial validation

Host tests cover stable/distinct PDA derivations, required signer metas, basic
versus slippage-protected instruction semantics, zero rejection, checked fee
and exchange rounding, exact minimum boundary, and round ordering.

Eleven local runtime simulations all rejected as expected:

1. amount zero;
2. impossible deposit slippage minimum;
3. wrong pool;
4. wrong mint;
5. token account not controlled by the caller;
6. withdrawal below technical minimum;
7. wrong validator stake;
8. wrong deterministic withdrawal-stake PDA;
9. round/stake reuse;
10. arbitrary final SOL destination;
11. finalization before the inactive epoch.

After the epoch advance, a deposit against pool epoch 1001 at Clock epoch 1002
was rejected with `StalePool` (custom error 6014). The successful CPI deposit
and withdrawal used a caller different from the persistent fee payer, proving
permissionless caller behavior.

Host build, host tests, TypeScript type-check, client tests, SBF build, and
offline cached rebuilds are required validation gates. Final command results are
recorded in the task handoff and the external state. Rust formatting was not
run because the accepted Task 0.3 toolchain has no `rustfmt` component; the
toolchain was deliberately not altered.

## Failures and fixes

- The official RPC throttled the first deployment after buffer creation and 19
  write transactions. The public buffer address, owner, authority, rent, and
  matching `19,232`-byte prefix were verified before the same buffer was
  completed over TPU. Its full artifact hash matched before final deployment;
  the loader closed it normally and no rent was stranded.
- The official RPC also returned rate-limit responses while post-transaction
  lifecycle evidence was being read. Deterministic accounts and payer history
  proved that withdrawal/deactivation and the expected premature-finalization
  failure had landed, so neither was resent. The bounded resume automation and
  external evidence file preserve the confirmed state.
- Official Testnet RPC airdrops of 5, 2, and 1 SOL were rate-limited; the
  dedicated fee payer was at 0 SOL during the accepted 2026-08-10 continuation.
  A read-only check on 2026-08-23 found 2 SOL, still below the accepted funding
  estimate. No Testnet SOL was spent or otherwise used by this update.
- The live Testnet pool was one aggregate epoch stale at final inspection, so
  even a funded transaction would need pool maintenance before CPI.
- Anchor's first build generated an ignored local program keypair and rewrote
  `declare_id!`. This was caught before any deployment; the persistent probe
  program key was installed in ignored `target/deploy`, and the declared ID was
  restored. No secret was tracked or printed.
- Large `test-validator --warp-slot` starts replace Stake History with a single
  epoch-0 entry. Agave 4.2's stake-history syscall asserts when a cloned stake
  activated at epoch 772 requests a missing contiguous entry. The local-only
  fixture changes that cloned public stake's activation epoch to 0, invoking
  the Stake Program's canonical “older than retained history” behavior. Real
  stake-pool and Stake programs, public addresses, account ownership, pool
  state, list membership, voter, balance, and CPI paths remain in use. This fix
  is local evidence only and does not modify Testnet.
- A first one-lamport escrow transfer was below zero-data rent exemption and
  rolled back. Funding the system-owned escrow with its exact 890,880-lamport
  exemption fixed creation.

## Production conclusions

The following are `CONFIRMED BY LOCAL TEST`; the final withdrawal, recovered
rent, fixed-escrow receipt, and fee-only caller delta are also `CONFIRMED BY
PUBLIC TESTNET`:

- A system-owned, empty-data SOL PDA can custody SOL and sign direct stake-pool
  deposit. This ownership form is required by the System Program transfer used
  by `DepositSol`.
- JitoSOL may be held in any correctly bound legacy Token account owned by the
  PIV authority; an ATA is the simplest deterministic production choice, not a
  stake-pool protocol requirement.
- A deterministic Stake Program-owned destination PDA is accepted.
- The PIV authority can be both staker and withdrawer and can sign the token
  burn/fee transfer.
- Withdrawal and deactivation fit one legacy transaction. The largest measured
  transaction used 20 keys and 768 of the 1,232 legacy-byte limit, with 156,453
  CU. A production instruction can exceed those measurements when additional
  state/economic checks are added, so remeasure and set a compute budget before
  treating one transaction as final architecture.
- Stake rent must be prefunded; the operational SOL vault can pay it, and full
  final withdrawal recovers it to the fixed escrow.
- The permissionless caller signs and pays the transaction fee. The PIV
  operational SOL reserve advances stake-account rent, which is recovered into
  the fixed PIV escrow during finalization.
- Finalization is possible after the deactivation epoch is older than Clock;
  normally this means waiting across an epoch boundary. Wall-clock duration is
  cluster-dependent.

Caller-provided validator accounts are operationally unavoidable because a
withdraw transaction must name one source account. Looping through a 1,129-entry
validator list on-chain is impractical. An off-chain permissionless keeper
should decode current pool/list state, choose an active standard validator stake
with sufficient lamports, and pass its list index, vote, and derived stake
address. Production must bind the pool/list, verify list status/current epoch
and the standard stake PDA (preferably by zero-copy reading the supplied entry),
and still rely on the stake-pool program's own membership/liquidity checks.

If a candidate loses liquidity or becomes transient between quote and execution,
the protected CPI fails atomically and a keeper retries another current
candidate. If pool state is stale, permissionless keepers must first run SPL
stake-pool validator-list balance updates in chunks and the aggregate pool
balance update, or wait for another keeper. These update-policy details are
`PROVISIONAL`; the safe atomic-failure behavior is confirmed.

No confirmed PIV1 requirement was found technically impossible. The production
program must integrate the dynamic minimum gate before opening a distribution,
reserve/recover rent separately, bind pending contributions and active rounds,
and withdraw only the native-SOL shortfall. This spike intentionally does not
implement or validate those production economic rules.

## Open findings and required follow-up

- `CONFIRMED BY PUBLIC TESTNET`: deployment, initialization, fixed PDA account
  funding, slippage-protected SOL deposit CPI, direct JitoSOL contribution,
  validator selection, slippage-protected stake withdrawal, PIV stake
  authorities, same-transaction deactivation, and premature-finalization
  rejection, epoch-delayed finalization, complete native-SOL recovery to the
  fixed escrow, stake closure, recovered stake rent, and replay rejection.
- `OPEN`: founder-approved slippage tolerance.
- `OPEN`: production validator-selection and pool-update keeper policy.
- `PROVISIONAL`: ATA choice and combined withdraw/deactivate as the production
  implementation; both worked locally but need review in the full program.
- `PROVISIONAL`: execution-plan Testnet wording correction described above.
- `REJECTED`: Devnet as Task 0.4's supported target, basic non-slippage variants
  for production, the stake interceptor for native SOL, client keypairs as a
  program requirement, DEX/Jupiter/instant exits, and marking Task 0.4 complete
  before native SOL reaches the fixed escrow.

Exact next task: founder review and acceptance of the completed Task 0.4
evidence. Task 0.5 may begin only as a separately authorized task; it was not
started here.
