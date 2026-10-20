# Proposed consolidated Testnet authorization card

**PROPOSED, NOT APPROVED. This non-executable card authorizes no live action.**
The [single M5 register](PIV1_M5_TESTNET_READINESS.md) owns readiness. The
[JSON card](PIV1_TESTNET_AUTHORIZATION_CARD.json) consolidates the retained
[governance/ALT/budget](PIV1_TESTNET_GOVERNANCE_ALT_BUDGET_PROPOSAL.md) and
[governed loader recipe](PIV1_TESTNET_GOVERNED_LOADER_PROPOSAL.md).
Actual selected package fields and artifact selection remain unchanged. Target only
official Solana Testnet; freshly match genesis
`4uhcVJyU9pJkvQyS88uRDiswHXSCkY3zQawwpjk2NsNY` before any future live action.

| Proposed profile | Recommendation, not selection |
| --- | --- |
| Existing Squads governance | Six members, threshold 4; vault index0 `A2UHpGyiCBdUZDMrofGswNiyS4ATD1ytBSiAsWNdxUTw` |
| HTFP recipient purpose | Index1 `4yNUJEkagan8hBNy1X9rkNSVwpBDidsSMWTF9ifZtLXJ`, HTFP allocation under unchanged economics |
| Team/Owner recipient purpose | Index2 `E3AMKJ31TmoYp7tkvQ6ffFrNNSkminYjWpV4jUzNBneo`, Team/Owner allocation under unchanged economics |
| PIV slots | Existing G0..G5 creation order; raw-byte sorted permutation `[3,1,4,0,5,2]` |
| Creator/executor/fee payer/ALT authority | Existing G0; four proposed approvers G0..G3 |
| Rent payer | Existing distinct H2, last observed absent/zero |
| Initial pause | **true**, recommended; no automatic unpause or immediate yield |
| Loader capacity | **1x** recommended (894,552 bytes); 2x is comparison only |

Recipient purposes/roles need explicit acknowledgement. Derivation is not funded
existence, current owner/control or exclusive 4/6 spending proof; delegated spending
limits and current governance still require review. Both recipients are distinct
same-multisig System vaults. Rent-floor transfers are not economic distributions.
G0 ALT control cannot singly authorize PIV economic changes. One-VPS keys do not
prove independent custody; proposed approvers are not actual votes.

The retained 894,552-byte candidate remains **unselected**. Any future governed
loader package must use Squads vault authority from first deployment, with buffer-only
handover before full byte verification/votes, atomic Program creation/deployment,
and separate sensitive-action authorization. None is authorized by this card.

For the KIF anchor, propose capturing a **fresh official Testnet Clock timestamp
before storing the initializer**. Separately select that exact literal; the signed
stored payload and governance votes must bind it. No old fixture or timestamp is
selected here. This is a proposed capture policy: freeze the literal before storage
and votes; never silently refresh it thereafter. Pause does not stop Solana Clock
or KIF periods. Production requires anchor <= runtime Clock, with fixed **30-day /
2,592,000-second** periods. This does not schedule immediate yield, waive activity or
claim conditions, change ten-day distribution cadence or its separate valid-
insufficiency 24-hour retry rule, or authorize unpause.

The JSON lists all **16 exact ordered ALT entries**: Config, ActiveDistribution,
GuardianRegistry, six proposed reward PDAs, five SOL targets and two legacy Token
targets. G0 is proposed authority. A separately scoped future capture must select a
fresh eligible recent slot and derive the table from authority raw32 plus slot-u64-LE
under the ALT program. **Actual address/recent slot remain null**; no historical
fixture address is reused. Exact creation/extension packets and fees require review.
Before use, verify owner, authority, active state, all16 entries/order/no extras and
execute strictly after the actual final extension slot. G0 can append/deactivate/
eventually close, not overwrite indexed entries; no freeze/close/deactivate is selected.

Propose exactly **one future 4WK→H2 credit of 4,700,000,000 lamports (4.7 SOL)**,
with source network fee capped at **10,000 lamports**: maximum proposed source debit
4,700,010,000. Exact packet, fresh state/fee and explicit authorization are missing.
This is an incoming H2 budget, not an all-in project cost or net-balance top-up.
No G0 refill is proposed. Its retained 1-SOL balance exceeds modeled fees but proves
neither current balance nor complete fee sufficiency or standing spending authority.
The source's previously observed extra 10 SOL has unknown provenance and grants no scope.

| Conditional 1x H2 model, retained prices | Lamports |
| --- | ---: |
| Governed loader peak, same-H2 Buffer credit reused before ProgramData | 4,552,746,800 |
| Shared34 initializer gross + two recipient floors, no H2 residual | 51,236,880 |
| Two leg rent advances above already-counted operational vault floor | 7,305,040 |
| ONE required initializer payer residual, counted once | 650,240 |
| Conditional combined reserve | **4,611,938,960** |
| Proposed credit less this model | **88,061,040** |

The initializer requires its payer to retain R0 after allocations; the single
650,240 residual is mandatory in this combined model. Earlier loader-only optional
reserve language stays historical. Shared34 uses a sizing-only managerFee/referrer
alias: actual pool/referrer selection is open. Distinct35 adds 335,280, giving
need 4,612,274,240/headroom87,725,760; fresh mandatory source/state validation remains.

Headroom is **not a spending allowance**. No G0 creator refund is netted against H2.
G0 pays fees separately: loader projection 4,765,000. The retained 11 successful
setup quotes total 100,000: ten initializer legacy phases contribute 90,000 and
the separate recipient packet 10,000; initializer execute remains unquoted. New recipe/funding/operational messages
have no fresh fee quote or complete fee cap. Test contribution capital is unselected;
operational advances are not principal or promised test capital. Actual total wallet
requirement, current eligibility and funding of each later action remain unproved.
The 2x combined comparison is 9,156,263,120 and does not fit this proposed credit.
Under the retained-rent model, ALT creation/extension and two recipient credits
would debit H2 by 4,836,160, already included above, leaving 4,695,163,840 after
PhaseA; do not add these costs again. Native ALT create/extend has no maximum-rent
argument; this model is not an inclusion-time enforced cap.
The three two-signature packets project G0 fees 30,000. Propose caps 10,000 each/
30,000 total, no priority fee, exact fresh quotes/packets required. Source transfer
fee cap 10,000 is separate. The PhaseA 30,000 fee projection is a subset of the
retained setup quote set, not additive. None of these caps authorizes spending now.

| Phase | Present scope / future boundary |
| --- | --- |
| Profile/startup/recipient purposes | Proposed only; acceptance separate |
| One H2 funding transfer, exact ALT creation/extension, two recipient provisions | Could be a future explicitly authorized narrow scope after exact review; **not authorized now** |
| Buffer creation/upload and buffer-only authority handover | Separate; **not authorized**, even if the narrow scope is later approved |
| Deployment, initializer storage/votes/execution, economic operations/unpause | Separate; **not authorized**, even if H2 is funded or ALT/recipients exist |

A later explicitly authorized bounded package may cover multiple named phases;
already covered technical steps do not require repeated routine approval. The narrow
funding/ALT/recipient scope still excludes deployment and later economic actions.

Loader and initializer need fresh, sequential actual Squads indices; their two
retained index1 sizing fixtures cannot form a live sequence. No index2 is guessed.

All prior RPC/rent/fee/feature observations are retained, not refreshed. This task is
documentation only: no RPC, private files, keys, signing, simulation, submissions,
funds, authority action, build or tests. No signer/helper/runtime readiness is claimed.
D-039 permits preparation; D-041/D-043/D-045 remain consumed. M4 stays open/unwaived
with stopped drafts untouched; M5/public NOT READY/deployment NOT AUTHORIZED.
Next resolve explicit profile/purpose acceptance and a separately scoped fresh ALT
slot/funding/provisioning packet review; no live action follows automatically.
Root publishes reviewed integration only; main `824342d`, protected `3677fee` and
the 2-GiB reserve remain unchanged. This is not a new decision or founder acceptance.
