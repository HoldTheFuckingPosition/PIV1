# Proposed Testnet governance, ALT and budget

**PROPOSED, NOT APPROVED. No funding, signing, transaction or deployment is authorized.**
The [single M5 register](PIV1_M5_TESTNET_READINESS.md) owns readiness. This
[public JSON summary](PIV1_TESTNET_GOVERNANCE_ALT_BUDGET_PROPOSAL.json) is not a CLI
manifest. All actual package selections remain unchanged; founder acceptance is false.

The proposal uses existing six-member 4/6 Squads and the actual D-045 program ID.
It proposes PIV slots in guardian-0 through guardian-5 creation order, revision 0.
The raw-byte sorted member order is G3, G1, G5, G0, G2, G4; therefore the PIV-slot
to sorted-member-index permutation is **[3, 1, 4, 0, 5, 2]**. The JSON records all
six reward PDAs, exact seed bytes and bumps. These are proposed tuples, not allocations.

| Proposed role | Choice |
| --- | --- |
| Governance vault | Index 0: `A2UHpGyiCBdUZDMrofGswNiyS4ATD1ytBSiAsWNdxUTw` |
| HTFP recipient | Index 1: `4yNUJEkagan8hBNy1X9rkNSVwpBDidsSMWTF9ifZtLXJ` |
| Team recipient | Index 2: `E3AMKJ31TmoYp7tkvQ6ffFrNNSkminYjWpV4jUzNBneo` |
| Creator, executor, fee payer, ALT authority | Existing G0 |
| Four distinct approvers | Existing G0, G1, G2, G3 |
| External rent payer | Existing H2, freshly absent/zero; future explicit funding required |

Any future deployment must establish the Squads governance vault as upgrade authority
from the start. No hot bootstrap authority or later authority transfer is proposed.
Existing keys on one VPS do not establish independent custody or four approvals.

The proposed ALT has exactly 16 ordered entries: Config, ActiveDistribution,
GuardianRegistry, six proposed reward PDAs, five System SOL targets and two legacy
Token targets. G0 can append, deactivate and eventually close the table, but cannot
rewrite existing indexed entries or singly authorize PIV economic changes. Before use,
verify the actual ALT owner, authority, active state, exact contents/order, and an
execution slot strictly later than its final extension slot. No freeze, close or
deactivation policy/action is selected. Actual ALT address and recent slot remain open.

Only a counterfactual shared34 fixture has complete unsigned wires. It uses initially
paused=true and the observed Clock timestamp 1792520248 solely for encoding, transaction
index 1, buffer index 0, and an expired historical ALT reference slot 454165136.
None selects startup policy, KIF anchor, live indices or an eligible table. The shared
manager-fee/referrer identity is a sizing fixture only. Distinct35 is rent-only with
no invented referrer. Wires and manifest remain in retained /tmp evidence, not this package.

Root produced 11 initializer packets plus one separate recipient-prefunding packet:
12 packets, 22 zero signature slots, all at most 1,232 bytes. Independent literal
wire review agrees, including 963-byte v0 execute. Eleven contextual legacy fee quotes
total **100,000 lamports**; execute's quote was rejected with -32602 because the ALT
does not exist. Its fee and the complete fee budget/cap remain **unknown**. These
expired-context observations are not simulation, runtime success or current fee limits.

| Conditional zero-prefund rent ledger, lamports | Shared34 | Distinct35 rent only |
| --- | ---: | ---: |
| H2 gross initializer allocations | 49,936,400 | 50,271,680 |
| Later buffer refund to G0 only | 8,569,960 | 8,737,600 |
| Final held initializer balances | 41,366,440 | 41,534,080 |
| Separate two recipient R0 credits | 1,300,480 | 1,300,480 |
| Separate H2 R0 residual | 650,240 | 650,240 |
| Conditional H2 envelope before fees/operations/deployment | **51,887,120** | **52,222,400** |

The initializer does not provision recipients. A separate unsigned sizing packet has
two H2 transfers of 650,240 lamports, G0 fee payer and two signature slots. Recipients
must be funded before execute. Do not net G0's buffer refund against H2's requirement:
stored transaction funding precedes that refund, and H2 and G0 are distinct wallets.
G0 pays network fees separately. Source routing/refill authorization and fees remain open.

The retained 894,552-byte candidate is not selected for deployment. Current loader-v3
allocation components are Program36: 833,120; Buffer(37+image): 4,545,162,360;
ProgramData(45+maxlen): 4,545,203,000 at 1x or 9,089,527,160 at 2x maxlen.
The respective three-component sums, 9,091,198,480 and 13,635,522,640 lamports,
are neither a minimum/net deployment cost nor a real peak wallet budget. Exact recipe,
wallet roles, reuse/refunds, upload fees, selected artifact and funding remain open.

A current conditional withdrawal-leg advance is R200+R263 = 3,652,520 lamports;
two concurrent legs require 7,305,040, above the already-counted operational vault
floor. Additional funding fees, operational liquidity, pending principal and test
capital remain separate and open. Natural yield is not promised; current source
capacity/minimum delegation, ten-day cadence and valid-insufficiency 24-hour retry
limits remain production constraints. The observed minimum delegation is dynamic.

Root made 40 read-only official Testnet RPC requests in seven HTTP batches, including
fresh governance, Clock/pool-prefix, wallet and rent observations, fee queries and
13 proposed-account absence observations. These are not a coherent Bank snapshot,
full current pool-fee validation or a fresh external-image check. G0..G5 each had 1 SOL;
H2 was absent/zero. The original source's additional 10 SOL has unknown provenance
and grants no spending authority. JSON evidence references bind the actual report,
source pins and separate wire/fee/cashflow review. Reviewer's initial approval-size
prediction correction was source arithmetic only, not a helper failure or rerun.

No keys/private files, signing, simulation, sends, funds, deployment or authority
changes occurred. Prior tests/Bank/SBF evidence is retained, not rerun. D-041/D-043/
D-045 remain consumed. M4 remains open/unwaived with stopped drafts untouched;
M5 is preparation, public NOT READY, deployment NOT AUTHORIZED. Reviewed integration
only; main and protected Task 2.3 remain unchanged. Preserve the 2-GiB reserve.

Official background: [fees](https://solana.com/docs/core/fees) and
[lookup tables](https://solana.com/developers/cookbook/transactions/lookup-tables).
Pinned sources and captured RPC evidence, rather than these background links,
support the reported arithmetic; neither grants execution authority.
