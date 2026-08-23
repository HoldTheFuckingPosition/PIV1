# PIV1 Task 0.4 — JitoSOL technical validation

Date: 2026-08-10 UTC

Branch: `spike/task-0.4-jito-validation`

Status: `BLOCKED_LIVE_TESTNET_FUNDING`

The intended custody lifecycle is `CONFIRMED BY LOCAL TEST` with the real SPL
Stake Pool and Stake programs plus cloned official Jito Testnet accounts. The
official Testnet fee payer held 2,000,000,000 lamports (2 SOL) in a read-only
RPC check at finalized slot 432,898,862 on 2026-08-23. This remains
2,554,218,843 lamports below the accepted 4,554,218,843-lamport funding
estimate, so no live Testnet deployment or transaction was performed. A
complete public-cluster cycle is therefore `OPEN`; Task 0.4 is not complete.

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
`0700` persistent directory. No secret material was printed or committed.

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

The following are `CONFIRMED BY LOCAL TEST`:

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

- `OPEN`: live deployment and every live Testnet transaction stage.
- `OPEN`: public-cluster end-to-end timing and finalization after a real epoch.
- `OPEN`: founder-approved slippage tolerance.
- `OPEN`: production validator-selection and pool-update keeper policy.
- `PROVISIONAL`: ATA choice and combined withdraw/deactivate as the production
  implementation; both worked locally but need review in the full program.
- `PROVISIONAL`: execution-plan Testnet wording correction described above.
- `REJECTED`: Devnet as Task 0.4's supported target, basic non-slippage variants
  for production, the stake interceptor for native SOL, client keypairs as a
  program requirement, DEX/Jupiter/instant exits, and marking Task 0.4 complete.

Exact next task: resume Task 0.4 on this branch and existing external state.
Wait until the existing dedicated fee payer reaches the accepted funding
estimate, then reinspect official Testnet until
`last_update_epoch == current_epoch`; if both gates pass, deploy the existing
probe program ID and execute/record the baseline, vault funding, CPI deposit,
direct contribution, CPI stake withdrawal,
deactivation, and finalization. If the real stake is deactivating, preserve the
state and return `IN_PROGRESS — WAITING FOR EPOCH`; after native SOL reaches the
fixed escrow, update this report and mark Task 0.4 complete. Do not begin Task
0.5.
