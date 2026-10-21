# Bounded Testnet startup phase and decision plan — draft

**Ready for governance/acceptance discussion; preparation draft; live package NOT READY; no live trigger.** D-048 conditional permission is retained with readiness unmet. The single [M5 readiness register](PIV1_M5_TESTNET_READINESS.md) remains the authority. This plan describes a future exact startup package and its two explicitly confirmed founder decisions (D-049); it creates no competing readiness register or execution script. M4 remains TECHNICALLY INCOMPLETE / OPEN_UNWAIVED; stopped operations/drafts and consumed scopes stay untouched.

## Accepted profile and exact scope

Use only official Solana Testnet, genesis `4uhcVJyU9pJkvQyS88uRDiswHXSCkY3zQawwpjk2NsNY`, existing PIV `GAjHRaC3XM98SoRoFy5B8P2ypTS1ndf1UPhXmjWgtPux`, canonical ProgramData `6HV5GQesbZpTyBVTiMu64UU4JgCX2D2s489sKt26xpd8`, existing Squads `9mwto7RNokYc8n8MPtzHpubwwnKcborY2r9RnfZAScVS` and final vault 0 `A2UHpGyiCBdUZDMrofGswNiyS4ATD1ytBSiAsWNdxUTw`. Accepted HTFP/Team recipients are vaults 1/2, `4yNUJEkagan8hBNy1X9rkNSVwpBDidsSMWTF9ifZtLXJ` and `E3AMKJ31TmoYp7tkvQ6ffFrNNSkminYjWpV4jUzNBneo`. G0 remains accepted fee payer/creator/executor/ALT authority, H2 external rent payer, G0 through G3 approvers; six PIV slot identities and permutation [3,1,4,0,5,2] are retained exactly in JSON. These identities are accepted profile facts, not fresh control or state observations. The disposable guardian identities remain under the same VPS custody/control; numerical 4-of-6 governance does not establish independent custody.

The candidate is 894552 bytes, SHA256 `e29735e1467da98ecddb1eea9fd25abaffa4a450bd899ddc137f7d8c0da35b99`, SBPFv0, accepted capacity 1x. D-049 explicitly selects this artifact and the disclosed temporary-G0 bootstrap trust for Testnet only; executing either remains unauthorized. Reuse existing ALT `4SoVCaX8tzTE1R6jGMJTArPq75afvBtVyr93Mm6hbTFF` with the exact 16 target entries. The sequence is loader Buffer create/write, native atomic top-level bootstrap, actual-deficit two-recipient provisioning, nine existing-ALT Squads initializer phases, then paused production heartbeat. No new-source funding, wallet refill, ALT recreation or later economic lifecycle is included.

## Evidence and conditional budget

Retained consolidation covers 949 complete unsigned packets/963 zero slots and975 conditional ledger rows. Fifteen actual representative fee quotes total 145000 lamports;934 unquoted Write projections add 4,670,000, giving G0 projected fees 4815000 and pre-refund maximum draw 4755000. This is historical sizing, not current prices, an inclusion cap or936 independent Write quotes.

The conditional H2 required envelope is 4594387560 lamports, including the 650,240 required initializer residual **once**; conditional headroom is 102076760. Same-H2 loader Buffer advance/credit is 4545162360, Program rent 833120 and ProgramData 4545203000. After loader H2 has 150428200 in that model, after two 650240 recipient credits and 46,400,720 gross initialization it ends 102727000. Governance Buffer 8569960 refunds only creator G0; it never reduces H2 need. Projected G0 final 1003734960 remains conditional. These retained amounts do not establish present balances, whole-project sufficiency or later operational/test capital. Actual deficits, fees, ceilings and balances need exact fresh review; no third-vault transfer is introduced.

Prior v5 local startup passes 11 entries/23 zero slots/115,000 local fees/46 ordered CPIs,41 initializer,13 maps/633 account records, but separately funded vault 0 at genesis. The new absent-vault derivative passes its sole guarded host build and positive process, with independent root and separate actual audits: 722 emitted rows, 620 account records in 13 full maps, 14 genuine-absence records, 11 successful entries, 23 zero signature slots, 115,000 local fees and 46 ordered CPIs (41 initializer). Vault 0 is absent in GenesisConfig and every fresh Bank map; no zero-account placeholder or third funding is inserted. This bounded local gap is **CLOSED ONLY FOR THIS EXACT LOCAL PROFILE**. Removing 650,240 unrelated fixture lamports preserves H2 final 104,027,480 and G0 final 1,008,434,960. Native role 5 is the canonical readonly signer supplied by real Squads invocation; its outer execute role is a readonly nonsigner. All 16 literal states, custody, complete account deltas and CPI order match. New bootstrap/initializer usage is 5,940/992,404 CU and 895,430/3,832,942 loaded bytes; heartbeat is 59,920 CU/897,649 loaded bytes. Earlier funded-vault initializer loaded bytes 3,833,006 remain distinct historical evidence. Both local fixtures separately capitalize recipients, explaining local H2 final 104027480 versus consolidated102727000. Bank 4.2/fixture Clock 100/anchor 0/external images/three features are not Testnet 4.4 replay, upload proof or public signatures/control.

## Phase prerequisites, results and STOP/RESUME rules

### P0 — Freeze exact readiness and fresh public preflight

- Bind the explicit D-049 candidate/bootstrap and limited M4 deliverable confirmations; preserve the independent readiness assessment and separate final live-authorization gate.
- Bind official Testnet endpoint/genesis, current node/version/features, program loader deployability, exact candidate bytes/hash/source provenance and current external program/ProgramData images and authorities. Assess retained Bank 4.2 versus observed Testnet 4.4 limitations; changed image/version requires review, never assumed equivalence.
- Refresh a consistent-context account subset and raw bytes: actors, current Squads members/permissions/threshold/timelock/current and stale indices, Program/PD/seeded Buffer, governance buffer/transaction/proposal, canonical recipients, all 16 targets and existing ALT. Preserve exact integers and distinguish absent from empty.
- Refresh rent, fees for actual messages, actor balance floors, per-action resources, safe blockhash margins and exact spending scope. No whole-upload lifetime is inferred.

Expected result: A separately reviewed immutable public package manifest with dynamic snapshots, amount/fee ceilings, actual sequential indices, per-phase messages and later real Clock capture boundary; no sensitive action occurs during preparation.

STOP: Any missing decision, unresolved critical/high defect, unsupported changed image/version/state, insufficient budget or absent review; do not infer readiness from D-048.

RESUME: Prepare a new reviewed preflight version using only public data; sensitive execution remains gated by the single M5 register and exact approved scope.

### P1 — Create and initialize the seeded loader Buffer

- Exact reviewed seeded address, G0 writer/fee payer and H2 rent payer; Program/PD absent; Buffer genuinely absent.
- Exact create+initialize atomic packet, current quoted rent/fees/resources/lifetime and actual signatures verified before any permitted send.

Expected result: Upgradeable-loader-owned894,589-byte Buffer, exact header Some(G0), correctly funded; no Program/PD or application state.

STOP: Existing Buffer with unexpected owner/header/authority/size/nonzero content; uncertain create transaction status; no overwrite or automatic replacement.

RESUME: Resolve original signature/status and account state. If matching Buffer already exists, separately verify the complete raw snapshot and resume upload only; never recreate a matching account.

### P2 — Upload all 936 aligned ELF chunks in batches of1..64

- Before every batch, freshly verify complete raw Buffer/header/authority, same-context Program/PD absence, exact candidate and current lifetime/fee/balance/resource limits.
- Run the existing bounded planner against the authenticated public snapshot; full 936-chunk classification precedes packet generation. Match956-byte chunks and692-byte tail; preserve already matching chunks; schedule only zero nonmatching chunks.
- Freeze each actual packet/message and expected signer; collect and verify fresh signatures; independently audit signed bytes before any permitted submission.

Expected result: Only confirmed matching writes count as progress. Final entire894,552-byte ELF and header are freshly reread and match e29735e1467da98ecddb1eea9fd25abaffa4a450bd899ddc137f7d8c0da35b99; no Program/PD.

STOP: Unexpected nonzero content anywhere including late chunk 935; writer/header/size mismatch; insufficient margin; unknown transaction outcome; stale snapshot.

RESUME: Resolve each prior submitted signature/outcome and full poststate before new planning. Match is byte equality, not transaction-history proof. Refresh outer blockhash only after ambiguity is resolved, discard all old outer signatures and obtain new signatures. No blind resend/rebind or overwrite.

### P3 — Native atomic top-level bootstrap and final vault 0 authority

- Decision A explicitly selected; full Buffer bytes/hash and Some(G0) authority verified immediately before signing/submission, with acknowledged residual mutability until inclusion.
- Program/PD absent and exact actual Program/H2/G0/G1/G2/G3 six-signature outer message; top-level System createProgram, native loader Deploy, final SetAuthority to canonical vault 0 in the intended same transaction.
- Fresh fees/resources/lifetime and same-H2 rent credit/debit model reviewed; no direct Deploy CPI.

Expected result: Successful atomic commit: Program executable/loader-owned, canonical ProgramData correct size and entire ELF hash, final authority exactly vault 0; loader Buffer consumed and its4545162360 model credit goes to same H2. Vault0 funding is not introduced.

STOP: Any uncertain status, wrong Program/PD/authority/image/balance poststate or missing confirmation. Do not send another deploy or separate authority fallback.

RESUME: Resolve signature and fresh Program/PD/Buffer state. Completed expected deployment advances to recipients; a mismatch stops for review. No deployment retry or authority repair is implied.

### P4 — Provision the two accepted recipients to actual deficits

- Freshly verify canonical vaults 1/2 of the same current multisig: addresses, empty data, System owner when present, no executable bit and actual balances.
- Derive each deficit=max(current rent floor for0bytes minus actual balance,0); absent recipients use the full floor. The retained two 650240 credits are historical-model amounts.
- Reconcile changed deficits/fees against exact reviewed funding scope and H2 residual. No vault 0 transfer, source credit or ALT setup.

Expected result: Both recipient vaults meet current funded empty-System recipient predicates, without counting their external balances as PIV principal or yield.

STOP: Wrong address/owner/data, ambiguous prior provisioning status or unsupported funding amount/scope.

RESUME: Resolve prior status and fresh balances before recalculating deficits; never blindly repeat the old transfer. Changed amount requires a new exact packet/budget review within established authority.

### P5 — Freeze Clock and the exact initializer before storage/votes

- Refresh current multisig membership/permissions/4-of-6/timelock/current and stale transaction indices. Derive actual next sequential transaction/proposal and available buffer index before fixing native account roles. Retained index 1/buffer 1 are examples, not current facts.
- Verify current existing ALT address/owner/active state, authority/metadata, exact ordered16 targets and strict later-slot warmup. Reuse adds no rent/signature solely for ALT authority; no ALT recreation.
- Capture one official Testnet Clock literal with context/raw bytes after prerequisites and before the first initializer Buffer storage or vote; encode and freeze it in native Config bytes and compact message.
- Freeze actual protocol fields/recipients/guardian order/permutation/pause=true/native roles, compact bytes and hashes; validate all 16 expected target states/floors and external H2 signer.

Expected result: A hash-bound immutable native initializer/compact message tied to actual indices and the captured Clock literal. Anchor remains unselected until this future capture; sizing anchor0 is not usable as a real anchor.

STOP: Clock/indices/state inconsistent or any proposed payload change after storage/voting. Pause does not stop Clock, and elapsed periods are not reset.

RESUME: For a necessary payload change, stop and separately review a new proposal/message with a newly explicit approval process and valid indices; never silently recapture/rebind anchor or mutate the voted transaction. Refreshing only an outer blockhash leaves stored/voted bytes unchanged.

### P6 — Execute the nine existing-ALT Squads initializer phases

- In order: Buffer create, Buffer extend, transaction-from-buffer, proposal create, G0 approval, G1 approval, G2 approval, G3 approval, execute.
- For each phase verify actual current prestate, exact signed outer wire/signatures/lifetime/fees/resources and resolved previous status; after storage/from-buffer/proposal read back bytes/hash/account identities and actual state.
- At each approval confirm exact frozen transaction/native/compact/anchor and current eligible member; at execute confirm actual threshold/approval set/timelock/current index/payload/ProgramData authority/recipients and virgin target predicates, plus H2 signing rent payer.
- Execute source requests 1.4M CU, 262,144-byte heap and 8 MiB loaded data are retained limits. The new absent-vault profile observes 992,404 CU and 3,832,942 loaded bytes, versus 3,833,006 in earlier funded-vault v5. Fresh Testnet sufficiency remains to be established.

Expected result: Proposal Executed; actual multisig index consumes the intended sequence; exact frozen paused Config/Idle distribution/registry/six revision0 rewards/five System targets/two Token vault states match; economic ledgers zero, custody/owners/lengths/rents exact. Governance Buffer closes and8569960 historical-model refund goes only to creator G0.

STOP: Any mismatch, unauthorized new member/index/payload/authority, failed transaction or uncertain outcome; stop before the next approval/execute.

RESUME: Resolve signatures and authenticated governance/account state before continuing the exact existing proposal. Do not duplicate rent accounts, votes or execution; no automatic payload/anchor change. New proposal or recovery action needs separate review/scope.

### P7 — Paused production heartbeat and bounded handoff checkpoint

- Read back final Program/PD/image/authority and all 16 initialized states; verify no economic movement and pause=true.
- Submit only the exact reviewed activity heartbeat for G0 with current valid outer lifetime/resources/fee scope, after all predecessor statuses are resolved.

Expected result: Heartbeat changes only guardian reward 0 activity plus G0 network fee; paused configuration/custody/economic ledgers and other rewards remain unchanged. Preserve public signatures, statuses, full poststate and independently reviewed startup receipt.

STOP: Any unexpected delta, unresolved confirmation or RecoveryRequired/unsupported state.

RESUME: Stop and report exact evidence; no unpause, deposit, stake, withdrawal, distribution, claim, governance rotation or recovery exit follows from startup completion.

## Outer lifetime versus stored/voted bytes

An eventual authorized workflow may refresh an outer blockhash only after prior uncertain outcomes are resolved. Every signature collected on that outer message becomes invalid and must be replaced after full message/roles/resources/fees/lifetime review. This does not change the native/compact bytes stored and approved in Squads. Anchor, payload, indices and identities remain immutable through voting/execution. A necessary payload change stops the flow and requires a new exact reviewed approval process; no silent anchor refresh or automatic proposal replacement.

Before any permitted submission, independently verify complete signed wire/message hash, exact signer set and cryptographic signatures. Bind the returned signature to that exact packet; record status, commitment, last-valid height and expected poststate. Select concrete status/history/confirmation sources in the final package. Unknown status blocks dependent actions and replacement. Blockhash expiry alone does not establish non-inclusion; reconcile signature history and account state. If evidence cannot resolve the outcome, STOP. The existing offline planner does not implement signing, sending or this confirmation controller.

## Founder-confirmed limited decisions — D-049

**A — exact candidate and Testnet-only temporary bootstrap exception.** Recommended scope is selection of only this candidate and the described native bootstrap for a future separately reviewed bounded startup. G0 can mutate Buffer until inclusion and is temporary first authority. G0/G1/G2/G3 plus H2/PIV exact-message signatures are endorsements, not on-chain Squads votes, membership/threshold enforcement or an ELF hash guard. Native signer capabilities permit alternate messages without those Squads constraints. A last preflight hash check cannot freeze Buffer through inclusion. Final 4-of-6 vault 0 authority in the intended atomic transaction does not retroactively protect bootstrap. Numerical 4-of-6 control under the same VPS custody is not independent custody. Decision A is explicitly confirmed by the founder at the exact scope below; no live execution authority follows.

Proposed decision text: “I select the exact 894,552-byte e29735e1 candidate for the named disposable Testnet PIV identity and accept the disclosed temporary-G0 native bootstrap exception only for the bounded startup package. I understand the Buffer mutability, temporary first authority, endorsement-versus-Squads-vote distinction and alternate-message capability. This selection is not a live trigger and does not authorize later lifecycle/funds.”

**B — supported local M4 deliverable with an empirical reservation.** Recommended scope is the existing [limited deliverable proposal](PIV1_M4_VALIDATION_SCOPE_REVIEW.md), preserving independent readiness assessment. The missing observation remains a callee actually mutating then failing internally, followed by complete fee-only rollback, identical no-fee replay and distinct successful retry. Source/host/positive/late-outer evidence does not supply that observation. No stopped operation is inspected, retried, rephrased or rerouted. Decision B is explicitly confirmed at the limited scope below; technical M4 remains incomplete/open/unwaived.

Proposed decision text: “I accept only the supported local M4 deliverable with the recorded internal-callee-failure runtime-evidence reservation. I do not treat that test as passed, waive its criterion or authorize circumvention of the stopped operation. Keep M4 TECHNICALLY INCOMPLETE / OPEN_UNWAIVED and preserve the reservation for separate readiness assessment before live action. This limited delivery-scope acceptance is not deployment approval or final functional acceptance.”

These decisions address actual delivery/governance risks. They do not ask the founder to choose Rust/tool details or repeat routine technical permission. D-048 conditional permission stays retained; even explicit decisionsA/B would leave the exact dynamic preflight, precise spending/operating/test scope, concrete submission/confirmation controls and separate readiness review outstanding. The register must record the exact package and readiness conclusion before any live action; no automatic trigger follows from this draft.

## End boundary

The proposed startup ends paused after an activity-only heartbeat and independent review of final states/signatures/statuses. It approves no unpause, contribution, deposit, stake, delayed withdrawal, distribution, KIF claim, membership change, recovery exit or later funds. Later test amounts/donors/operating liquidity remain unselected. Natural yield, fixed Clock periods, ten-day cadence and real withdrawal availability govern later lifecycle tests; no fabricated appreciation, shortened Clock or guaranteed withdrawal date is offered. Mainnet, main publication and final founder functional acceptance remain separate.

The writer prepared only E-directory sources and this plan; root executed one reviewed guarded host build and one positive Bank process. No PIV ELF build, fresh RPC, key/private access, signing, send, live funds or live authority action occurred. Repository source and all selected package sections are unchanged by the writer. Root owns any reviewed canonical edit and publication.

## Exact actual evidence and remaining technical preflight

The JSON evidence index binds original sources/assembly, unchanged lock, separate source/binary gates, the actual build/process, complete raw output and independent root/separate audits. The separate actual review is `reviewer-actual-absent-vault0-v1.json` (15,414 bytes, SHA256 `08e39649176f917fd3d107aeb0c0fefe1bdac49a38c42fc1687f7aa5f96aa637`); source freeze is `2b17b114181d5f75f107f474fda19ff2a8a612385a9a280ff3f87681987ae6d2`, binary `4603858875ae6ae591aa57126ba910995df50794baad172cd87679c827e32c65`. Runtime stdout is 104,180,833 bytes, SHA256 `c0d06f4b113f2a36646a93336d477bf77a610b2906980363a9379e9acd96ff3f`. Retained 128,292-byte assembled input remains `3bf393b5036a8706db859438ea1f2edf1613954a8fe3424ec06362266655522b`; its historical source provenance is recorded separately from current pins. No helper reassembly or PIV ELF build occurred.

Remaining technical work is independent of the two decisions:

- Fresh authenticated cluster, node/features, external images, actor/control/account/ALT/governance state and exact artifact/source applicability. Local unsigned Bank 4.2 entries establish neither public signatures/control nor current Testnet 4.4 execution or authenticated live simulations.
- Actual rent, fee, resources, transaction lifetimes and per-actor balances; precise startup spending ceilings and explicit separate operating/test-capital boundaries. Historical projections are not inclusion limits.
- Actual sequential Squads indices and exact native/compact bytes, with one real Clock literal frozen before first initializer storage/votes.
- A concrete separately reviewed signer/submission/confirmation/history/resume workflow, including resolution of every uncertain result before replacement and verification of full Buffer progress. This document supplies rules, not an executable controller.
- Independent readiness/risk assessment in the single M5 register, including the M4 reservation. Even explicit decisions A and B do not declare that assessment passed or authorize immediate partial deployment.

The absent-vault proof closes only its concrete local prerequisite; actual Buffer upload, recipient provisioning, live votes/control, full economic lifecycle, natural yield and real timing remain outside it. The historical task preserved all four package sections; D-049 subsequently updates only authorized present selection/mapping metadata: candidate and bootstrap trust are selected under D-049; the real anchor stays unselected.

## Publication and baseline provenance

The retained source-document byte/hash entries in JSON point to immutable snapshots
of checkpoint `a2f4f6b`. Package/readiness preparation metadata changes in this
checkpoint are not new runtime observations. Publication is reviewed integration
only; main `824342d` and protected Task 2.3 `3677fee` stay unchanged. Git and the
publication receipt record the resulting commit without a self-referential hash.

## D-049 current authority and new public preparation

The founder explicitly confirms A/B and requires separate explicit authorization before any new live operation. One future authorization may cover an exact bounded startup package; no repetitive command approval is required. See [fresh public preflight](PIV1_TESTNET_STARTUP_PREFLIGHT.json) and [preparation authorization card](PIV1_TESTNET_STARTUP_AUTHORIZATION.md). Current artifact/source, public authorities/images/state, rents/fees and conditional funding have now been checked by root and separate review. This does not close the controller, exact final package, real Clock, runtime-version limitations or final authorization. Readiness stays unmet/public NOT READY, and no signing, deployment, funds, authority or Mainnet action is authorized. Earlier execution/evidence paragraphs retain their original task scope.


## Read-only collection checkpoint (2026-10-21)

The [public collector evidence](PIV1_TESTNET_PUBLIC_RECONCILIATION_COLLECTION.json) records12 passing focused offline tests and one actual six-read/four-HTTP Testnet collection. Original extension inclusion at454845694 and current four-account map at454937310 are separate observations. The composed checker really STOPPED/exit1 on source4WK balance+10SOL versus its retained expectation; provenance is unknown. H2/G0/ALT match. No expected state is rebound, positive rerun or funding/control conclusion made. Raw responses and this STOP remain preserved. TLS/current subset collection is not a consensus proof, complete source oracle, inclusion-time map, submission journal or live trigger.

D-049 A/B remain confirmed; M4 remains TECHNICALLY INCOMPLETE / OPEN_UNWAIVED. No stopped operation/draft or consumed scope is reopened. The package remains NOT READY, with durable journal/segment linkage, Clock binding, controlled operator and exact final resource/lifetime/cap/state-oracle work outstanding. No new signing/simulation/send, fees/funds, authority, deployment, Mainnet or private-key operation occurred. Separate explicit completed-package authorization is required before any new live operation. Prior broader preflight/prices/runtime results are retained, not refreshed by this bounded collection.


## Durable public journal checkpoint (2026-10-21)

The [journal evidence](PIV1_TESTNET_STARTUP_PUBLIC_JOURNAL.json) records 13 passing focused tests, nullable CLI and 18 standalone journal commands. Three retrospective segments bind 3 historical packets/5 signatures/9 synthetic events, with 25,000 historical fees and zero new fees. Owner/mode/no-follow/stable-lock/external-tail CAS/fsync and actual pinned-checker replay enforce bounded immutable records, global fees/replay/slot/frozen linkage and UNKNOWN blocking. The retained actual source +10 SOL/unknown-provenance mismatch is rejected offline without changing its expectations, accepted tail or bytes. No fresh RPC or current-state positive rerun occurs.

This is local public persistence, not authentic submission history or a complete 949-packet controller. Protected external receipts remain necessary against whole-directory/receipt rollback; advisory locks cover cooperating processes. Limits: 32 segments / 1024 records / 16 MiB combined checker input / 64 MiB journal remain explicit. The semantic Clock pre→post boundary/native 315-byte offset 299/compact/roles/storage/votes binding is OPEN, and no real anchor or null→anchor transition is selected. Controlled signer/send wiring, complete oracle/resource/lifetime/capacity/final package and separate readiness review remain required.

D-049 A/B remain narrowly confirmed, M4 TECHNICALLY INCOMPLETE / OPEN_UNWAIVED, public NOT READY / NO LIVE TRIGGER. No stopped operation/draft or consumed scope is reopened. No new private keys/keys/signing/simulation/send, fees/funds, authority, deployment, Mainnet, build/Bank/SBF or old-suite rerun occurred. Separate explicit completed-package authorization remains mandatory before any new live operation. Prior broader preflight/prices/runtime observations are retained, not refreshed.


## Offline Clock/profile seal checkpoint (2026-10-21)

The [Clock binding evidence](PIV1_TESTNET_CLOCK_INITIALIZER_BINDING.json) records 23 passing focused tests (13 journal regressions + 10 Clock tests), nullable CLI and 14 standalone operations / 6 expected refusals. PROFILE durably commits the full anchor-null initializer under the existing tail/lock/CAS, then SEAL derives the sole anchor from exact 40-byte Clock and actual pinned native 315 / compact 1,447 / 34-role bytes. Nine generated packets retain 16 zero signature slots. Raw Clock/context must dominate parent inclusion/poststate and planner/ALT contexts; nonnegative anchor restriction is a utility subset, not a protocol change. Existing generic hash labels are not founder-profile acceptance. Same lock/CAS/fsync/rederivation terminally blocks ordinary mutations, reprofile and recapture, with one SEAL slot reserved before PROFILE. V1 HOLD/correction remains preserved; first executed v2 passes.

Timestamp 100 and large slots are synthetic, the real selected anchor stays null, and historical parent fee 5,000 means zero new fees. No new RPC, private keys/keys/signing/simulation/send, funds, authority, deployment, Mainnet, build/Bank/SBF or old application-suite rerun occurred. Prior source +10 SOL/unknown-provenance mismatch and wider observations remain retained, not refreshed/rebound. Actual Clock capture/freshness/storage/readback/votes, controlled successor/sign/send, full oracle/resources/lifetimes/capacity/final package and separate readiness review remain OPEN. D-049 A/B stay narrowly confirmed, M4 OPEN_UNWAIVED, public NOT READY / NO LIVE TRIGGER; separate explicit completed-package authorization is mandatory before live work. No stopped operation/draft or consumed scope is reopened.


## Read-only initializer storage checkpoint (2026-10-21)

One bounded writer and separate source review add a read-only three-account Squads gate. Root passes **13/13 new focused tests**, template/nullable commands and **8 snapshot CLI checks** (2 positives/6 exact expected refusals). Tests reject all 1,548 transaction prefix truncations and XOR1 changes at all 1,548 byte positions. Root raw audit binds Borsh 1,548-byte storage to native 315 / compact 1,447 bytes and 34 ordered roles, with PRE Active/no votes and POST Approved/exact selected four. See [storage evidence](PIV1_TESTNET_INITIALIZER_STORAGE_BINDING.json).

The existing stable lock/CAS and pinned seal rederivation preserve all seven parent files and inherited historical 5,000-lamport ledger. No new fees/cap/child journal. Multisig/Proposal dirty allocation tails and variable Option offsets remain valid. Each positive gate uses three pinned Node helpers. Timestamp100/index73/large context and 1-lamport raw accounts are synthetic, not real anchor, rent or funding proof. This closes supplied-byte comparison only. Collection/freshness, inspection-before-votes, vote signatures, timelock release, controlled successor/sign/send and full oracle/resources/lifetimes/capacity remain open. No prior executable, production/economics/dependency/toolchain/ELF or four D-049 selected sections change. No new RPC/private wallet keys/keys/signing/simulation/send/funds/authority/deployment/Mainnet/build/Bank/SBF; old suites and the +10SOL source mismatch are retained, not rerun/refreshed/rebound. D-049 still requires separate explicit completed-package authorization before live work; M4 OPEN_UNWAIVED/public NOT READY/NO LIVE TRIGGER. This checker is not the controlled successor or causal before-votes enforcement.


## Composed governance readback checkpoint (2026-10-21)

One bounded writer and separate source review add a composed fixed-origin Squads/Clock collector. Root passes **12/12 new focused tests**, five CLI checks (including the expected synthetic-live guard refusal) and six independent response replays (2 positives/4 exact refusals, **11 injected HTTP calls / zero actual network or blocked attempts**). Seven network-denial launch audits pass. Root verifies77 artifact hashes, both raw snapshots, checked-i64 delay3600 at3700-100 and refusal at3699-100. The seven parent files/tail and historical5000-lamport ledger remain unchanged. See [readback evidence](PIV1_TESTNET_GOVERNANCE_READBACK.json).

Code checks official genesis then one finalized four-account snapshot and feeds actual normalized bytes to the pinned storage verifier; fresh Clock context/slot must dominate caller/seal floors. Timing arithmetic is separate from release/eligibility/auth flags, which stay false. Null requests import no helpers; the pre-freeze reader defect/correction is preserved and first executed v1 passes. Output cannot enter the parent; exclusive evidence is capped40MiB with2GiB reserve. No actual TLS/chain refresh, private wallet keys/new keys/signing/simulation/send/funds/authority/deployment/Mainnet/build/Bank/SBF or old-suite rerun. Prior artifact/runtime/prices/+10SOL source mismatch are retained, not refreshed/rebound. No old executable, economic rule/dependency/ELF or four selected package sections changes. Real anchor/test amounts remain null. Causal storage-before-votes, vote signatures, controlled successor/sign/send and complete oracle/resources/lifetimes/capacity remain open. D-049 remains preparation only; no live trigger or M4 waiver.

## D-051 source-wallet clarification — external faucet funding

[Public reconciliation](PIV1_TESTNET_EXTERNAL_SOURCE_RECONCILIATION.md) explains the specific +10 SOL source discrepancy with two finalized 5 SOL credits and independently verified message signatures. The founder reports personally replenishing the shared4WK operational Testnet wallet from the faucet at approximately 04:00 local. Public times are02:06:06/02:06:14 UTC (04:06 Europe/Warsaw); the source 128,763,968,087→138,763,968,087lamport change is exact. This is external wallet funding, with zero PIV1 principal/pending contribution/yield credit or exclusive project allocation. The original STOP/expected states/raw evidence/journals remain immutable; earlier unknown-provenance descriptions are retained knowledge-at-that-time. No checker exception, positive replay or current whole-package pass is claimed.

Future authorized source operations need fresh contextual sufficiency, exact authorized debits/fees and all custody/security checks; unrelated legitimate source deposits do not become unexplained protocol behavior or an indefinite whole-wallet freeze. No new source spending/refill enters P0..P7. This clarification authorizes public verification/documentation only, not keys/private reads/signing/simulation/send/funds/authority/deployment/Mainnet. M4 OPEN_UNWAIVED, public NOT READY/NO LIVE TRIGGER, consumed scopes and D-049’s separate completed-package authorization remain unchanged.
