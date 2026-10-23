# PIV1 requirements-to-evidence checklist

**Current technical checkpoint — unused replay work reduced; controller lifetime gap measured (2026-10-23 Europe/Warsaw).**
Two small changes remove unused work: exact_typed loads fresh rules only when it encounters a typed descriptor, while literal coverage still performs its independent fresh Clock/source check; replay uses intent/terminal cardinality because existing insertion guards enforce terminal⊆intents with no removals. No new validation/result cache, replay removal, margin/cap relaxation or authority/economic change. The manifest source pin is updated; message/role/account derivations and program sources retain their behavior.
Validation is **42 unique passing groups across two runs, not one clean 42-group run**: the initial suite passed 41/42 in 133.762 s (134.190 s process). One new assertion expected deploymentMissing when a two-typed-account case first reached proposalMissing. The test now isolates each missing fact; that one affected method passed in 0.387 s (0.566 s process), with product bytes unchanged. Both executed test versions/failure remain. All 40 relevant regressions plus the two new groups are covered; unchanged four Node groups/build/runtime/949-message audit are reused. Separate source/actual reviews and nullable CLI pass, with zero Python network/native-backend attempts.
A single fresh source-bound fixture contains Buffer create+936 Writes: 7,497 records, 4,521,077,406 evidence bytes and 24,556,935 journal bytes including lock. Read-only replay measures **12.551 s** (15.004 s process); this is not an identical-byte before/after comparison with older fixtures. The canonical bootstrap controller path then measures PREPARE 32.654 s, fake SIGN 65.156 s and SEND 56.159 s; **153.969 s total, 118.213 s from original quote to fake dispatch, with eleven full replays on that critical path**. The three actions share module caches in one fresh process; this is one observation, not OS-cold or a per-CLI-process bound. An additional denied live promotion is timed separately and leaves no append.
**Measurement seams:** six ordered signer slots contain public NON-signature markers; both the signer callback and check_signed crypto/trace gate are explicit fakes. Public I/O/heights/Clock/finality are synthetic. Actual canonical messages/roles, all 57 prestate accounts, 14 rents, source/raw checks, durable journals and final lifetime/auth guards are consumed. Native signer and real crypto/trace-gate costs, actual Clock/block-height progression and inclusion are unmeasured; no wall-time-to-height conversion, real headroom, native bootstrap or full-949 claim follows. Bulk history construction bypasses append/crypto/per-file production fsync; measured replay bodies run unchanged through a timing wrapper. All retained file bytes and final 7,503-record chain are root-verified/fsynced after timing. Bootstrap is only marked SUBMITTED in the fake transport, not recovered; twelve canonical packets remain unreconciled. Minimal RPC metadata/retries/full evidence bounds remain explicit limitations.
**Next blocker/completion condition:** reduce the eleven postquote/pre-dispatch full replays through a separately proved change preserving fresh dependency/source/raw/account/authority/auth/lifetime and at-most-once/HOLD protections, then demonstrate the complete supported schedule, remaining twelve packets and exact resources/funding/evidence sufficiency before final independent readiness. Native key control, actual signer/checker costs and permanent lost-response/expired-authorization HOLD remain OPEN. October 21 observations/prices and D-051 remain historical, unchanged. **M4 TECHNICALLY INCOMPLETE / OPEN_UNWAIVED; M5/public NOT READY / NO LIVE TRIGGER.** No real keys/private reads/signing, simulation/RPC/send, funds, authority, deployment or Mainnet action. D-049 exact final live authorization remains mandatory. Integration only; main b7e30b2/protected Task 2.3 3677fee unchanged; preserve 2 GiB.

**Retained technical checkpoint — original lifetime and authorization checked after durable intent (2026-10-23 Europe/Warsaw).**
SEND now repeats the original blockhash validity/remaining-height checks after the complete INTENT replay/fsync, and checks the exact current authorization immediately before the single send call. The post-intent context floor dominates the accepted prestate; height cannot regress from either prepared or preflight height. Captured original expiry/margin remain fixed, while legacy packets retain exact latest-hash/expiry equality. Existing genesis/version, fees, every account/authority predicate, rents and all source/raw-evidence checks remain. Packing at most 20 preflight methods into four batches leaves one three-method post-intent check and one send: the existing six-query/five-method bounds stay unchanged, including all fourteen rent sizes.
Root passes **37/37 focused Python groups first execution** (two new plus 35 relevant regressions; process 139.470 s), the nullable CLI and separate source/actual review. New cases consume the actual durable append before advancing synthetic height, invalidating the original hash or expiring authorization. Each refuses send, retains one unresolved original INTENT/reservation, blocks resend/reprepare and permits inspection. The positive full-host case checks all fourteen rents and exact batches 5/5/5/5/3/1. Historical signatures and explicit synthetic ELF/account/message/transport seams prove offline guard consumption, not native signing or deployment. A late malformed/expired read can now leave an INTENT on hold; it must never be silently repaired or renewed.
One prior-source read-only profile reuses the unchanged 7,499-record synthetic prefix: 20.717 s profiled replay / 22.425 s process, with 6,559 fresh rule loads and 72,149 primary-source checks. Rule loading is 9.413 s cumulative, including 7.142 s source verification; these overlap and must not be added. Profiling overhead makes this attribution only, not a new latency baseline. The exact tail and journal metadata remain unchanged, with zero network/child/mutation attempts. No verification/result cache was added; no new speedup is claimed and old source-bound receipts are not rebound to this correction.
**Next blocker/completion condition:** demonstrate the complete preparation→SIGN→SEND schedule within the original lifetime and unchanged margins, including remaining twelve packets, exact phase-time resources/funding and evidence capacity/retries, then final independent readiness. Native reader/key control and lost-response/expired-authorization permanent HOLD remain OPEN. Unchanged Node/backend, selected 949-message audit, build/runtime, October 21 observations and D-051 are reused without refresh. **M4 TECHNICALLY INCOMPLETE / OPEN_UNWAIVED; M5/public NOT READY / NO LIVE TRIGGER.** No real keys/private reads/signing, simulation/RPC/send, funds, authority, deployment or Mainnet action. D-049 exact final live authorization remains mandatory. Integration only; main b7e30b2/protected Task 2.3 3677fee unchanged; preserve 2 GiB.

**Retained technical checkpoint — measured late-upload replay and compilation cost (2026-10-23 Europe/Warsaw).**
A physical offline history covers Buffer creation plus 936 Writes (937 completed phases; 12 packets remain). Its measured baseline contains 7,497 records, 4,520,729,571 evidence bytes and 24,576,627 journal bytes; physical allocation is 4,696,264,704 bytes. These are exact synthetic-prefix measurements, not full-949/retry/future-RPC bounds. Original v1 missing-ComputeBudget fixture STOP and v2 real 120-second PREPARE timeout remain preserved, including the PREPARE committed before that timeout. The fixture now uses the existing pinned native-account oracle; no product oracle changes.
The product fix caches only immutable compiled state-rule code. Every call still rereads and hashes rules source, rejects symlinks, creates fresh module globals and verifies all eleven primary sources. No state, account, authorization, Clock or raw-evidence result is cached. Root passes **35/35 focused Python groups first execution** (one new cache-boundary group plus 34 relevant regressions; suite 107.583 s / process 107.925 s) and the nullable CLI. Separate source/actual reviews pass; network/native-backend guards record zero attempts.
At this prefix, replay improves 50.037→16.447 s and append 45.048→15.366 s. Actual offline captured PREPARE completes in **52.815 s**, including **30.398 s after the injected blockhash response**, versus the preserved 120-second baseline timeout. Each version has one observation in a fresh Python process, not an OS-cold claim; the loader initializes compiled rules before the stage timer, and whole-process times are retained. Buffer/image caches start empty. Synthetic bulk construction bypasses append/crypto/per-file production fsync; measured replay/append/evidence scan/operate are unpatched. Public non-signature markers, synthetic Clock/status/finality and minimal RPC metadata prove no native signatures, bootstrap, consensus or real lifetime margin. All retained fixture bytes were independently inventoried and root-verified/fsynced after measurement.
**Next blocker/completion condition:** demonstrate the complete preparation→SIGN→SEND lifetime/resource/funding schedule without reducing margins or security checks, account for the remaining 12 packets, ordinary RPC metadata/retries and evidence capacity, then obtain final independent readiness. The measured 30.398-second postquote work remains a concrete scheduling gap. Native reader/key-control and lost-original-response/expired-authorization permanent HOLD remain OPEN. Unchanged four Node groups, selected 949-message audit, build/runtime, October 21 public observations and D-051 reconciliation are reused without refresh. **M4 TECHNICALLY INCOMPLETE / OPEN_UNWAIVED; M5/public NOT READY / NO LIVE TRIGGER.** No keys/private reads, real signing, simulation/RPC/send, funds, authority, deployment or Mainnet operation. D-049 exact completed-package live authorization remains mandatory. Integration only; main b7e30b2 / protected Task 2.3 3677fee unchanged; preserve 2 GiB.

**Retained technical checkpoint — captured original transaction lifetimes (2026-10-23 Europe/Warsaw).**
Existing PREPARE can capture original blockhash/expiry, per-message fees and current height into exclusive raw evidence. Initial strict unsigned decoding, an exact 32-byte blockhash replacement and semantic equality preserve all other message bytes. The existing PREPARE record binds quote/source/authorization/parent/predecessor/ordered messages; replay verifies raw receipts and advances context floors. SEND accepts a still-valid original hash after latest advances only with this bound quote. The newer hash cannot extend original expiry; legacy preparations retain exact-latest equality. Capture batches are at most 23, preserving six HTTP queries/five calls and existing record/storage limits; no extra per-packet record.
Root passes **34/34 focused Python groups first execution** (six new plus 28 relevant regressions), suite 112.110 s / process 112.485 s, and the nullable CLI. Python network and real signer-child guards record zero attempts. Positive cases use injected public responses, historical signatures and explicit synthetic full-host ELF/account/message seams. Separate source/actual reviews pass. The unchanged four Node backend groups and selected 949-message/role/cashflow, build/runtime and October 21 public observations are reused, not rerun or refreshed. No actual quote capture, native signing/private read or live operation occurred.
This closes the original-lifetime binding/usability defect, not whole-upload timing or complete resource/funding/evidence sufficiency. Static no-retry records remain 7598/8192; retained Buffer payload components total 4,465,798,272 bytes across 936 writes, not a complete serialized or retry bound. Cumulative replay/scanning still needs practical late-upload measurements. Native reader/key-control execution and original-response/expired-authorization HOLD limitations remain OPEN. Next measure and close practical upload/evidence/lifetime scheduling and exact phase-time resources/funding, then separate final readiness. **M4 TECHNICALLY INCOMPLETE / OPEN_UNWAIVED; M5/public NOT READY / NO LIVE TRIGGER.** D-049 exact completed-package live authorization remains mandatory. Integration only; main b7e30b2 / protected Task 2.3 3677fee unchanged; preserve 2 GiB.

**Retained technical checkpoint — future signer backend and controlled invocation (2026-10-23 Europe/Warsaw).**
SIGN implements a fixed pinned Node backend behind an explicit independent live-sign flag and strict launcher receipt v2 binding source/profile hashes. One durable request/attempt precedes possible key access; verified complete output uses the existing atomic IMPORT. The backend enforces the accepted Testnet signer universe, exact unsigned messages/roles, secure explicitly mapped files and every output signature. A full-six profile stays immutable across phases while only currently required keys are opened. No wallet discovery, private-path journal entries or raw child diagnostics; the controlled child disables core dumps. V1 cannot invoke this backend, and injected signing history cannot become live transport.
Root passes **28/28 Python groups and 4/4 Node groups in separate first executions**, process durations 78.735 s / 9.253 s; two CLI guards pass. Existing signatures and explicit fake subprocess/reader/key/sign callbacks are used. Native signing, key generation/derivation/import, private-file opens and network counters are zero. The full-host ELF/message/lifetime seams remain synthetic. Separate source and actual reviews pass; no Bank/build or unchanged application suite is rerun. The unchanged selected 949-message/role/balance audit, October 21 state/prices and D-051 are retained, not refreshed.
**Native reader/key-match/sign execution and current key control remain UNEXECUTED / OPEN.** An unavailable original signer response or expired original authorization can leave a permanent hold; no automatic retry, replacement, renewal or repair is implemented. Static no-retry accounting gives 7598/8192 records and 594 remaining, excluding an optional manual HANDOFF; this is not practical whole-upload/expiry/retry/evidence proof. Next close phase-time resources/funding/lifetimes and practical upload/evidence scheduling, account explicitly for native-control and hold/recovery gaps, and complete separate readiness review. **M4 TECHNICALLY INCOMPLETE / OPEN_UNWAIVED; M5/public NOT READY / NO LIVE TRIGGER.** No real profile/keys were read or created, no signatures generated, no simulation/RPC/send/funds/authority/deployment/Mainnet action. D-049 final exact live authorization remains mandatory. Integration only; main b7e30b2 / protected Task 2.3 3677fee unchanged after D-053; preserve 2 GiB. See [operator evidence](PIV1_TESTNET_RELEASE_OPERATOR.json).

**Retained publication checkpoint — founder-authorized main update (2026-10-23 Europe/Warsaw).**
D-053 authorizes normal fast-forward publication of the seven reviewed commits after main e2bb2a3 through integration 3b79bb3, plus this separately reviewed documentation checkpoint. The root/CLI/program/test READMEs and current guidance reflect the reviewed final state. Git refs and the publication receipt record the resulting common main/integration identity; protected Task 2.3 stays 3677fee. Earlier integration-only/main-e2bb2a3 notes below retain their historical scopes. Subsequent development returns to integration-only publication unless separately authorized.
Publication checks reuse unchanged source/actual-review evidence, including the latest 21/21 focused groups; prior 31-group provenance and failed runs remain preserved. No code, dependency, artifact, accepted economics or historical measurements change in this documentation checkpoint; no test/build/Bank/SBF or chain-state/price refresh is repeated. All twelve cumulative non-documentation paths match their final reviewed source pins.
**M4 TECHNICALLY INCOMPLETE / OPEN_UNWAIVED; M5/public NOT READY / NO LIVE TRIGGER.** Actual signer backend/control, phase-time resources/funding/lifetimes, practical upload/evidence scheduling and final readiness remain open. D-049 separate exact completed-package live authorization stays mandatory; no new keys/private reads/signing/simulation/send/funds/authority/deployment/Mainnet operation is authorized or performed. This is main publication, not broader founder acceptance. Preserve 2 GiB. See [D-053](PIV1_DECISIONS.md#d-053--confirmed--founder-authorized-main-publication-of-reviewed-m5-continuation).

**Retained technical checkpoint — durable exact signer requests and responses (2026-10-23 Europe/Warsaw).**
HANDOFF now durably binds Testnet/genesis, operator source and predecessor, parent/manifest/seal, launcher authorization, PREPARE record identities, exact messages and ordered signers. INSPECT recovers lost public output. Strict IMPORT consumes the complete ordered response atomically; reprepare, reseal, replacement requests, changed/expired authorization and UNKNOWN prevent stale advancement. Canonical full mode refuses legacy list imports; SEND and intent replay retain the same authorization binding. Existing pinned checks verify every signature at import and before SEND; replay rederives linkage and metadata without repeating historical cryptographic checks.
Root passes **21/21 focused groups first run** (eight new, twelve operator regressions, one adapted full-account case), in 73.188 s (runner), 73.512 s (whole process); separate source and actual reviews pass. Python connection/DNS denial records zero attempts. Only historical public signatures are used; synthetic full-host ELF/account/template/lifetime seams remain explicit. The unchanged 949-message/role/balance audit and prior runtime/build/application evidence are reused, not rerun. October 21 public state/prices and D-051 remain historical.
**Actual signer backend/key control remain OPEN; no signer was invoked.** Next implement that bounded backend, then close phase-time resources/funding/lifetimes and practical whole-upload/evidence scheduling, followed by final independent readiness. No private/key reads, new keys, signing, simulation, RPC/send, funds, authority, deployment or Mainnet operation. **M4 TECHNICALLY INCOMPLETE / OPEN_UNWAIVED; M5/public NOT READY / NO LIVE TRIGGER.** D-049 final exact live authorization remains mandatory. Integration only; main e2bb2a3 / protected Task 2.3 3677fee unchanged; preserve 2 GiB. See [operator evidence](PIV1_TESTNET_RELEASE_OPERATOR.json).

**Retained checkpoint — canonical startup manifest and mandatory phase coverage (2026-10-23 Europe/Warsaw).**
The selected startup adapter constructs all 949 ordered descriptors from the pinned public profile and selected ELF, and the existing controller consumes source-derived PRE/POST predicates. Each loader Write requires ten relevant accounts; every other phase, including each Squads vote, requires all 57. Exact messages/signers, actor fees/rents/refunds, complete Buffer progress, external images/authorities/features, Squads storage/indices/approvals, ALT, zero principal/pending/yield and the four prior typed rules are enforced. Full mode rejects Clock regressions, changed versions/rents and ambiguous heartbeat periods. Partial legacy manifests cannot reach actual fixed-origin transport. No caller account map or readiness flag replaces these predicates.
**Validation comprises 31 unique groups across retained and corrected runs, not one clean 31-group run:** initial 28/30 (20 prior groups plus eight new) exposed the mandatory 1,106,125-byte external image above old 1-MiB bounds and an all-zero fixture assumption. A narrowly selected complete-image path fixes that product defect, with old collector/checker limits and sources unchanged. Recovery verifies a complete 57-account snapshot after the bounded 56-account signature proof and binds both exclusive raw receipts during replay. Corrected tests passed 10/11; the remaining 5,000-lamport expected-value typo was corrected and its one method passed. Both failed runs/executed bytes remain. Separate source and actual reviews pass. Root's retained real-image audit matches all 949 masked messages/roles and actor-balance rows; actual manifest/CREATE record are 3,415,550/3,416,041 bytes, reopen 1.130s. Constructor/template/formula equality justifies reusing that audit after the size fix. Local raw storage formulas also match retained Bank bytes. No Bank/build or unchanged application suite is rerun. Host signature/template/image/account seams and synthetic anchor0/lifetimes are explicit; no actual startup, signer control or real Clock selection follows.
**Zero new Solana RPC, signing, simulation, send, keys/private wallet reads, funds, authority, deployment or Mainnet action.** October21 public state/prices, ELF/runtime and D-051 reconciliation are retained, not refreshed. The shared source is excluded from startup spending; historical fees and the proposed-only future namespace remain separate. **Next blocker/completion condition:** integrate the controlled signer handoff/response for exact canonical prepared packets, then demonstrate phase-time resources/funding/lifetimes and practical whole-upload/evidence scheduling within the existing bounds. Complete final independent readiness before presenting the exact live package. **M4 TECHNICALLY INCOMPLETE / OPEN_UNWAIVED; M5/public NOT READY / NO LIVE TRIGGER.** D-049 separate completed-package authorization remains mandatory; integration only, main `e2bb2a3` and protected Task2.3 `3677fee` unchanged; preserve 2 GiB. See [current manifest continuation evidence](PIV1_TESTNET_RELEASE_OPERATOR.json).

**Retained checkpoint — consumed startup runtime state rules (2026-10-21 Europe/Warsaw).**
The concrete four-field oracle blocker is closed within the declared typed subset: ProgramData inclusion slot/full selected ELF/vault authority, all sixteen initialized account bytes with sealed Config anchor, causal Squads Proposal timestamps/approval prefixes, and guardian0 heartbeat activity. SEND/RECOVER consume these source-pinned rules; immutable intent/terminal facts and raw Clock windows are checked again during replay. Changed rent quotes or ambiguous KIF periods stop; exact old account expectations and UNKNOWN protections remain intact. No generic masks or nullable security oracles are added.
Root passes **20/20 focused groups** (eight new plus twelve relevant unchanged operator regressions), first suite execution in 48.579s, and the corrected nullable CLI. Root's initial literal-null CLI input correctly refused `object_fields`; both that driver mistake and the corrected four-field request are retained. Separate source review passed before execution. Independent comparison reuses the old positive Bank trace: full selected ProgramData, sixteen literal targets, six Proposal stages and eighteen heartbeat-state accounts, plus three exact anchor/ELF/slot refusals. No Bank/build or unchanged application suite is rerun. Synthetic signed-transfer/ELF-seam operator tests prove rule consumption, not a native bootstrap. The nonzero sealed-anchor host case is synthetic; the real anchor remains null.
Three public GitHub GETs recover missing Squads files at the already pinned revision, matching retained SHA256 and Git blob identities. **Zero new Solana RPC, real operator transport or Python blocked network attempts.** Retained public prices/state, local runtime and D-051 funding reconciliation are not refreshed. No private keys, new keys, signing, simulation, real send, funds, authority, deployment or Mainnet action occurred. Historical fees and the proposed-only future fee namespace remain separate.
**Next concrete blocker/completion condition:** assemble the exact P0..P7 manifest with mandatory message/account/authority coverage and demonstrate that the controller consumes every required predicate. Actual signer/control integration, phase-time resource/funding/lifetime checks, practical upload scheduling and final separate readiness review remain open. No omitted account or null oracle may close these gaps. **M4 TECHNICALLY INCOMPLETE / OPEN_UNWAIVED; M5/public NOT READY / NO LIVE TRIGGER.** D-049 final completed-package authorization remains mandatory; integration only, main `e2bb2a3` and protected Task2.3 `3677fee` unchanged. Preserve 2 GiB. See [current operator continuation evidence](PIV1_TESTNET_RELEASE_OPERATOR.json).

**Retained checkpoint — reusable release operator validated offline (2026-10-21 Europe/Warsaw).**
Resumed exact paused drafts from `2431dad`, preserving originals and handoffs. Root passes **12/12 focused operator tests** in 42.431s and one nullable CLI check under Python socket denial (zero blocked attempts, zero actual RPC/HTTP/signing/send). The first run had one pass/11 errors in a new synthetic parent fixture; its extra metadata violated the unchanged strict checker schema. That failed run and executed bytes remain preserved; only the fixture changed before the passing run. No old suite/build/runtime was rerun.
The operator enforces exact unsigned/signed message linkage, durable one-send intent, UNKNOWN blocking, original-signature read-only recovery after authorization expiry, unique retained evidence attempts and same-parent PROFILE/SEAL lineage. The positive host case binds nine actual sealed planner packets, prepares the first and reopens; it is not a real Clock capture or public initialization. It has no wallet reader or signer implementation. Live actions still need the future exact launcher-bound receipt and explicit transport flag; no current receipt exists.
Historical fees/caps remain immutable and separate from a **PROPOSED ONLY** future 4,815,000-lamport fee namespace; new counters start at zero and no consumed scope is renewed. Evidence has an 8-GiB aggregate ceiling, 256-MiB action envelope and 2-GiB free-space floor. These and the 949-descriptor/8192-record limits are bounds, not proof of complete upload throughput or retry capacity. The earlier 39-read public preflight, selected ELF/local runtime and D-051 faucet reconciliation are retained, not refreshed or replayed.
**Release blockers remain:** source-derived runtime state rules for Config anchor, ProgramData deployment slot, Proposal timestamp and heartbeat activity; mandatory account/authority coverage and exact P0..P7 manifest; actual signer integration/control and phase-time resources/lifetimes/upload scheduling; final independent readiness review. Null/omitted oracles and generic masks are not permitted shortcuts. Next implement those startup-specific state/phase adapters using the retained literal source oracles, reusing unchanged passing tests.
**M4 TECHNICALLY INCOMPLETE / OPEN_UNWAIVED; M5/public NOT READY / NO LIVE TRIGGER.** No new private wallet reads, keys, signing, simulation, real send, funds, authority, deployment or Mainnet occurred; stopped drafts untouched. D-049 final authorization remains mandatory. Reviewed integration only; main `e2bb2a3` and protected Task2.3 `3677fee` unchanged. See [operator evidence](PIV1_TESTNET_RELEASE_OPERATOR.json).

**Source funding clarification — D-051 (2026-10-21 Europe/Warsaw).**
The founder identifies 4WK as a shared operational Testnet wallet regularly replenished from the faucet for PIV1/other projects. Root reads 7 public RPC requests/two HTTPS batches and verifies two 64-byte signatures on 267-byte legacy transactions plus two tamper refusals: external credits 5 SOL + 5 SOL at 454930106/454930140 (02:06:06/02:06:14 UTC,04:06 project time) reconcile128,763,968,087→133,763,968,087→138,763,968,087lamports. Fresh finalized 454985153 confirms the same final balance. The external sender pays both 5,495 fees; 4WK signs neither. Faucet/operator attribution is founder testimony; raw transfer proof is public.
This specific +10 SOL difference is **RECONCILED EXTERNAL WALLET FUNDING**, unrelated to PIV1 principal/pending contributions/yield, not an unexplained protocol anomaly or spending authority. Preserve original STOP/expectations/raw records/journals unchanged; do not silently rebind or positive-rerun. A shared source needs fresh contextual sufficiency and exact scoped debit/fee checks for future authorized actions, not an indefinite whole-wallet balance lock; strict protocol custody/security checks remain intact. No executable/product tests/build or old-suite rerun; no new private/key/sign/simulation/send/funds/authority/deployment/Mainnet. Only source history/balance was refreshed.
M4 remains **TECHNICALLY INCOMPLETE / OPEN_UNWAIVED**; M5/public **NOT READY / NO LIVE TRIGGER**. D-049 separate final live authorization remains mandatory; D-050 publication is complete and this task is integration only. Next controlled sequence/status controller and final readiness package; preserve 2 GiB. See [source reconciliation](PIV1_TESTNET_EXTERNAL_SOURCE_RECONCILIATION.md).

**Current checkpoint — composed governance and Clock readbacks (2026-10-21 Europe/Warsaw).**
D-049 preparation continues; D-050 main publication is complete at e2bb2a3. **M4 TECHNICALLY INCOMPLETE / OPEN_UNWAIVED; M5/public NOT READY / NO LIVE TRIGGER.** Separate explicit completed-package authorization remains mandatory before live operations; consumed scopes/stopped drafts stay untouched.
One bounded writer and separate source review add a composed fixed-origin Squads/Clock collector. Root passes **12/12 new focused tests**, five CLI checks (including the expected synthetic-live guard refusal) and six independent response replays (2 positives/4 exact refusals, **11 injected HTTP calls / zero actual network or blocked attempts**). Seven network-denial launch audits pass. Root verifies77 artifact hashes, both raw snapshots, checked-i64 delay3600 at3700-100 and refusal at3699-100. The seven parent files/tail and historical5000-lamport ledger remain unchanged.
Code checks official genesis then one finalized four-account snapshot and feeds actual normalized bytes to the pinned storage verifier; fresh Clock context/slot must dominate caller/seal floors. Timing arithmetic is separate from release/eligibility/auth flags, which stay false. Null requests import no helpers; the pre-freeze reader defect/correction is preserved and first executed v1 passes. Output cannot enter the parent; exclusive evidence is capped40MiB with2GiB reserve.
No actual TLS/chain refresh, private wallet keys/new keys/signing/simulation/send/funds/authority/deployment/Mainnet/build/Bank/SBF or old-suite rerun. Prior artifact/runtime/prices/+10SOL source mismatch are retained, not refreshed/rebound. No old executable, economic rule/dependency/ELF or four selected package sections changes. Real anchor/test amounts remain null. Causal storage-before-votes, vote signatures, controlled successor/sign/send and complete oracle/resources/lifetimes/capacity remain open.
Next implement controlled phase/status/ledger continuation and the final bounded package. Integration only; main e2bb2a3/protected3677fee; preserve2GiB. See [readback evidence](PIV1_TESTNET_GOVERNANCE_READBACK.json).

**Retained publication checkpoint — founder-authorized main update including READMEs (2026-10-21).**
D-050 authorizes normal fast-forward main publication of the **25 reviewed commits after 824342d through 5663d68**, plus this separately reviewed documentation checkpoint. Git and `/tmp/piv1-main-publication-20261021-a/publication.json` record the actual final main/integration identity; protected Task 2.3 stays 3677fee. Earlier integration-only/main824342d statements below retain their historical scope.
Root/CLI READMEs reflect the current preparation; program/test READMEs link to the canonical current state while preserving historical test/acceptance scopes. This task changes documentation only; prior implementation/tests/runtime/chain observations are retained, not rerun/refreshed. Review bindings, ancestry, README links, selected sections, hooks/workflows, sensitive/generated-file checks and the 2-GiB reserve are publication gates.
**M4 TECHNICALLY INCOMPLETE / OPEN_UNWAIVED; M5/public NOT READY / NO LIVE TRIGGER.** Main publication is not deployment, economic or broader functional acceptance. D-049 still requires separate exact completed-package live authorization; stopped operations/drafts and consumed scopes remain untouched. No new keys/private wallet reads/signing/simulation/send/funds/authority/deployment/Mainnet/build/Bank/SBF. Next continue controlled readbacks/operator and final readiness work on integration; future main publication needs separate authority.

See [D-050](PIV1_DECISIONS.md#d-050--confirmed--founder-authorized-main-publication-including-readmes).

**Retained technical checkpoint — read-only initializer storage binding (2026-10-21 Europe/Warsaw).**
D-049 A/B remain narrowly confirmed; M4 stays **TECHNICALLY INCOMPLETE / OPEN_UNWAIVED**. Preparation only: **PUBLIC NOT READY / READINESS UNMET / NO LIVE TRIGGER**. Separate completed-package authorization is required before any new live operation; stopped drafts and consumed scopes stay untouched.
One bounded writer and separate source review add a read-only three-account Squads gate. Root passes **13/13 new focused tests**, template/nullable commands and **8 snapshot CLI checks** (2 positives/6 exact expected refusals). Tests reject all 1,548 transaction prefix truncations and XOR1 changes at all 1,548 byte positions. Root raw audit binds Borsh 1,548-byte storage to native 315 / compact 1,447 bytes and 34 ordered roles, with PRE Active/no votes and POST Approved/exact selected four.
The existing stable lock/CAS and pinned seal rederivation preserve all seven parent files and inherited historical 5,000-lamport ledger. No new fees/cap/child journal. Multisig/Proposal dirty allocation tails and variable Option offsets remain valid. Each positive gate uses three pinned Node helpers. Timestamp100/index73/large context and 1-lamport raw accounts are synthetic, not real anchor, rent or funding proof.
This closes supplied-byte comparison only. Collection/freshness, inspection-before-votes, vote signatures, timelock release, controlled successor/sign/send and full oracle/resources/lifetimes/capacity remain open. No prior executable, production/economics/dependency/toolchain/ELF or four D-049 selected sections change. No new RPC/private wallet keys/keys/signing/simulation/send/funds/authority/deployment/Mainnet/build/Bank/SBF; old suites and the +10SOL source mismatch are retained, not rerun/refreshed/rebound.
Next connect exact storage readbacks and controlled continuation before final readiness. Integration only; main824342d/protected3677fee; reserve2GiB. See [storage evidence](PIV1_TESTNET_INITIALIZER_STORAGE_BINDING.json).

**Retained checkpoint — offline durable Clock and initializer binding (2026-10-21 Europe/Warsaw).**
D-049 A/B remain confirmed within their exact scopes; M4 is **TECHNICALLY INCOMPLETE / OPEN_UNWAIVED**. Preparation only: **PUBLIC NOT READY / READINESS UNMET / NO LIVE TRIGGER**. Separate explicit completed-package authorization remains mandatory before any new live operation; stopped operations/drafts and consumed scopes stay untouched.
Root passes **23/23 focused tests** (13 necessary journal regressions and 10 new Clock tests), nullable CLI and **14 standalone CLI operations** with 8 expected accepts/6 expected refusals. One synthetic complete parent commits PROFILE before Clock and terminal SEAL; root verifies six immutable records, native 315 / compact 1,447 bytes, 34 ordered roles and 9 unsigned packets / 16 zero slots against an independent retained oracle. Timestamp 100/large slots are synthetic; the real selected anchor stays null. Parent 5,000 lamports are historical fees; zero new fees/RPC.
PROFILE durably freezes the exact anchor-null manifest and fee ledger; raw 40-byte Clock supplies only the anchor for actual pinned planner bytes/roles. Same lock/CAS/fsync and rederivation block stale Clock, ordinary appends, reprofile and recapture. A v1 final-slot availability finding was fixed before first execution; snapshots/HOLD remain retained. Generic parent hash labels do not prove founder-profile acceptance. Nonnegative Clock anchors are the utility subset; native signed-i64 support is unchanged.
This closes only local declaration/byte/seal linkage. Real Clock capture/freshness, storage/readback/votes, controlled post-seal successor/signing/send, full oracle/resources/lifetimes/capacity and final readiness remain open. Production/economics/dependencies/toolchains/ELF and four selected D-049 sections are unchanged; only the prior journal executable is extended. No new private keys/keys/signing/simulation/send, funds/authority/deployment/Mainnet, Bank/SBF/build or old application-suite rerun. Prior actual source +10 SOL mismatch and wider observations are retained, not refreshed/rebound.
Next build the controlled post-seal successor/operator with exact payload and ledger linkage before the final bounded package. Integration only; main 824342d / protected 3677fee; preserve 2 GiB. See [Clock binding evidence](PIV1_TESTNET_CLOCK_INITIALIZER_BINDING.json).

**Retained checkpoint — durable public journal and linked segments (2026-10-21 Europe/Warsaw).**
D-049 A/B remain confirmed within their exact scopes; M4 is **TECHNICALLY INCOMPLETE / OPEN_UNWAIVED**. Preparation only: **PUBLIC NOT READY / READINESS UNMET / NO LIVE TRIGGER**. Separate explicit completed-package authorization is required before any new live operation; stopped operations/drafts and consumed scopes remain untouched.
One bounded writer and separate source review add an offline durable public journal. Root passes **13/13 focused tests**, nullable CLI with no created journal, and **18 standalone journal CLI commands** across separate processes. Three retrospective segments preserve 3 old packets/5 historical signatures/9 synthetic events and 25,000 historical fees; **zero new fees**. Independent root audits 15 immutable records. The prior actual source +10 SOL/unknown-provenance poststate mismatch is rejected again offline without changing its expectations, accepted tail or bytes; no current-state positive rerun or RPC occurs.
Records persist with exclusive owner/mode/lock/CAS/fsync checks; actual pinned-checker replay enforces global freeze/cap, replay, fees and successful-slot linkage. UNKNOWN blocks dependent segments. Partial/uncertain writes fail closed without repair/retry/adoption. Caller receipts/local hashes are not authenticated history, consensus proofs or privileged-host rollback protection; stored submission labels prove no send. Bounds: 32 segments / 1024 records / 16 MiB combined checker input / 64 MiB journal do not prove the whole 949-packet controller.
The semantic Clock boundary remains **OPEN**: no real anchor selection or raw 40-byte Clock→native 315-byte/compact/roles/storage/vote binding; null→anchor relabeling is rejected. Production/economics/old executables/dependencies/toolchains/ELF and four D-049 selected sections remain unchanged. No new RPC/private keys/new keys/signing/simulation/send, fees/funds/authority/deployment/Mainnet, Bank/SBF/build or old suite rerun. Prior chain/prices/runtime observations and source mismatch are retained, not refreshed.
Next implement explicit durable Clock pre→post linkage and payload binding, then controlled operator/full oracle/resources/lifetimes/capacity/final package and separate readiness review. Integration only; main 824342d / protected 3677fee; preserve 2 GiB. See [journal evidence](PIV1_TESTNET_STARTUP_PUBLIC_JOURNAL.json).

**Retained checkpoint — read-only public reconciliation collection (2026-10-21 Europe/Warsaw).**
D-049 A/B remain confirmed within their exact scopes; M4 is **TECHNICALLY INCOMPLETE / OPEN_UNWAIVED**. Preparation only: **PUBLIC NOT READY / READINESS UNMET / NO LIVE TRIGGER**. Separate explicit authorization of the completed exact bounded package is required before any new live operation; stopped drafts and consumed scopes remain untouched.
One bounded writer and separate source review add a fixed-origin public Testnet collector and exclusive raw-evidence storage. Root passes **12/12 focused offline tests**, template and nullable validation. One actual collection succeeds with **6 read-only RPC requests / 4 HTTP calls**, no retries/errors: original ALT extension finalized at454845694, four current accounts at454937310, two historical signatures independently verified. The old blockhash is now invalid; that does not undo historical inclusion.
The composed checker actually **STOPPED / poststate_mismatch / exit1**: source4WK is138,763,968,087 versus retained128,763,968,087 lamports, a+10SOL difference of unknown provenance. H2/G0/ALT match. No expectation is rebound, positive rerun or funding/control conclusion made. This STOP remains evidence; collection success is not whole-package success. Four static v1 findings were fixed before first execution; v1 remains preserved.
TLS-origin/current subset evidence is not a consensus proof, inclusion-time poststate, complete state oracle or durable submission journal. Production/economics/old executables/dependencies/toolchains/ELF and four D-049 selected sections remain unchanged. No private keys/new keys/signing/simulation/send, new fees/funds/authority/deployment/Mainnet, Bank/SBF/build or old suite rerun. Prior wider preflight, prices and runtime evidence are retained, not refreshed.
Next close exclusive durable journal/segment linkage, Clock-before-storage binding, controlled operator and exact resource/lifetime/cap/state-oracle package gaps; retain the current mismatch and require separate final readiness review. Integration only; main824342d/protected3677fee; reserve2GiB. See [collector evidence](PIV1_TESTNET_PUBLIC_RECONCILIATION_COLLECTION.json).

**Retained checkpoint — bounded public signature and reconciliation checks (2026-10-21 Europe/Warsaw).**
D-049 A/B remain confirmed only within their exact scopes. M4 stays **TECHNICALLY INCOMPLETE / OPEN_UNWAIVED**; stopped operations/drafts and consumed scopes remain untouched. Preparation only: **PUBLIC NOT READY / READINESS UNMET / NO LIVE TRIGGER**. Separate explicit authorization of the completed exact bounded package is required before any new live operation.
One writer and separate source/actual reviews add a pure public checker, portable 7,711-byte fixture, tests and nullable example; CLI README preserves its entire 72,046-byte prefix. Root passes one focused suite, **13/13 groups**, plus six standalone CLI commands. All five historical public signatures verify; 80 generated corruptions, all 65,536 u16 encodings, truncation/ALT/u64/i64/fees/frozen-state/UNKNOWN/chronology guards are covered. Actual complete CLI checks three old packets/five signatures/nine synthetic retrospective events and 26 retained account records; 25,000 lamports are historical fees, **zero new fees**. UNKNOWN and altered signatures STOP as intended; independent Python verifies five historical signatures and five tamper rejections.
Three v1 static findings were corrected before the first execution; v1 sources remain preserved. New code verifies supplied bytes/declared subsets only and always reports authorization/readiness/authenticity/persistence false. It does not provide authenticated collection, durable exclusive journals/segment linkage, a Clock-freeze controller, signing/send or complete state-oracle coverage; no real-signed v0 positive fixture is claimed.
Production/economics/old executables/dependencies/toolchains/ELF and all four D-049 selected sections are unchanged. No RPC, private keys/new keys/signing/simulation/send, funds/authority/deployment/Mainnet, Bank/SBF/build or old suite rerun. Prior public snapshots/budgets/runtime evidence are retained, not refreshed. Next close collector/durable-linkage/controlled operator and final resource/lifetime/package gaps, then separate readiness review and the final authorization card. Integration only; main 824342d/protected 3677fee; reserve 2 GiB. See [public controls evidence](PIV1_TESTNET_STARTUP_PUBLIC_CONTROLS.json).

**Retained checkpoint — D-049 confirmations and reviewed public startup preflight (2026-10-21 Europe/Warsaw).**
A selects exact894,552-byte/e29735e1 Testnet candidate and disclosed temporary-G0 bootstrap exception; B accepts only the supported local M4 deliverable with its empirical reservation. M4 remains **TECHNICALLY INCOMPLETE / OPEN_UNWAIVED**. No stopped operation/draft or consumed scope is reopened.
The newer founder instruction requires **separate explicit authorization of the completed exact package before any new live operation**. Preparation only: **READINESS UNMET / PUBLIC NOT READY / NO LIVE TRIGGER**. D-048 does not override this gate.
Root/separate provenance checks match132 HEAD/worktree source pins, exact candidate/header and retained build receipts, without rebuilding or claiming reproducibility. Root makes39 read-only official Testnet RPC requests in7 HTTP calls:49 primary accounts at454921958,9 dependencies at454921974,14 rents and15 sizing fee quotes. Four full external images/authorities match retained profiles; node4.4.0-beta.1/feature3993424257 remains observed, not a local4.2 whole-runtime replay.
Squads six members/permissions7/threshold4/timelock0/current+stale0 and existing exact16-entry active/warm ALT pass. Program/PD/Buffers/tx1/proposal1/16targets/vault0/two recipients are absent. Shared managerFee/referrer mapping is source-supported and selected for paused startup. Real Clock anchor stays null.
Observed H2/G0=4,696,464,320/999,980,000; fresh prices preserve conditional H2 required4,594,387,560/headroom102,076,760 and4,815,000 projected fees. Only15quotes are actual;934Write fees are projections. Historical-payload/anchor0 packets remain sizing-only, not executable. Refund8,569,960 belongs to G0 only.
No new application tests/build/Bank/SBF, keys/private reads, signing/simulation/send, live funds/authority/deployment or Mainnet. Next close concrete journal/signature/status/confirmation/resume enforcement and exact final resources/lifetimes/package under bounded implementation/separate review; then present the complete package for one separate live authorization. No repeat A/B or routine technical approval question. Integration only, main824342d/protected3677fee; preserve2GiB. See [public preflight](PIV1_TESTNET_STARTUP_PREFLIGHT.json) and [preparation card](PIV1_TESTNET_STARTUP_AUTHORIZATION.md).

## Current evidence priority — M5 preparation under D-039 (2026-10-21 Europe/Warsaw)

**Retained checkpoint — absent-vault startup proof and bounded release plan (2026-10-21 Europe/Warsaw).**
D-048 remains conditional: **READINESS UNMET / NO LIVE TRIGGER / PUBLIC NOT READY**. M4 stays TECHNICALLY INCOMPLETE / OPEN_UNWAIVED; stopped operations/drafts and consumed scopes remain untouched.
One guarded locked/offline host build and one positive local Bank process pass. Root/separate audits verify 11 entries, 23 zero signature slots, 115,000 local fees, 722 evidence rows, 620 account records, 13 full maps and 46 ordered CPIs (41 initializer).
Canonical governance vault0 is absent from genesis and all 13 fresh maps, with 14 explicit absence records and no placeholder, allocation or third funding. The concrete prior separately funded-vault fixture gap closes only for this exact local profile.
All 16 literal PIV states/custody and H2/G0 cashflows remain exact. New initializer usage is 992,404 CU / 3,832,942 loaded bytes; earlier funded-vault 3,833,006 is retained, not relabeled. Local H2 final 104,027,480 excludes separately capitalized recipient fixtures; conditional all-startup H2 final 102,727,000 remains a different retained budget model.
The bounded phase plan separates two pending founder choices (exact candidate/temporary-G0 bootstrap trust; supported local M4 deliverable with reservation) from remaining dynamic preflight, precise spending scope and signing/status/confirmation controls. Neither choice is inferred, closes technical M4 or triggers live work.
No production/economic/client/dependency/toolchain/ELF change, RPC/private/key/signing/simulation/send/LIVE funds/authority/deployment or Mainnet action. Local SetAuthority is a Bank fixture operation only. All 370 retained non-checkpoint inputs and four selected package sections remain unchanged; 63 current source pins matched before preparation metadata changed.
Next resolve those genuine governance/acceptance choices and finish exact fresh package/Clock-before-storage/votes/lifetimes/resources/confirmation/readiness work under the retained mandate, without routine technical approvals. Root publishes reviewed integration only; main 824342d/protected 3677fee, reserve 2 GiB. See [bounded release plan](PIV1_TESTNET_STARTUP_RELEASE_PLAN.md).

**Retained checkpoint — offline Buffer upload-resume client (2026-10-21 Europe/Warsaw).**
D-048 remains conditional: **READINESS UNMET / NO LIVE TRIGGER / PUBLIC NOT READY**. M4 OPEN_UNWAIVED and stopped operations/drafts stay untouched.
One bounded writer and separate review add a pinned public-input planner, eight host groups, nullable example and append-only CLI usage; no production/economic/dependency/toolchain/ELF change.
Root passes all eight groups plus template/null CLI (29 missing inputs, plan null). Three synthetic CLI reports retain 64/3/0 unsigned packets; root/separate raw audits verify all 67 wires/zero slots and 2,808 chunk classifications. A fourth CLI rejects the late chunk-935 conflict with no stdout.
A new host property test reconstructs all 936 writes in 15 bounded batches; this is synthetic byte replay, not a blockchain upload. Existing correct chunks remain unchanged; unknown nonzero content stops planning before any packet output.
Exact u64 lifetime margins and batches of 1..64 are explicit declarations, not authenticated freshness or throughput. Only Write packets are emitted; artifact/bootstrap trust/real Clock anchor stay unselected, and G0 Buffer mutability remains.
No RPC/private/key/signing/simulation/send/funds/build/Bank/deployment/authority action; prior runtime, 41-read budget observations and consumed scopes are retained, not rerun or renewed.
All 366 prior non-checkpoint inputs other than CLI usage remain unchanged; its 67,665-byte prefix and the four selected package sections are preserved. Root owns reviewed integration publication; main 824342d/protected 3677fee, reserve 2 GiB.
Next complete the exact artifact/bootstrap trust/version/resource/funding package, absent-vault runtime limitation, Clock-before-storage/votes and the remaining end-to-end execution/confirmation/resume controls. No routine technical approval or live trigger follows. See [upload-resume evidence](PIV1_TESTNET_UPLOAD_RESUME.json).

**Retained checkpoint — consolidated startup sizing and conditional budget (2026-10-21 Europe/Warsaw).**
D-048 conditional permission remains **READINESS UNMET / NO LIVE TRIGGER / PUBLIC NOT READY**; M4 OPEN_UNWAIVED and stopped drafts stay untouched.
One assembly/budget run and seven host groups pass; root/separate audits verify 949 unsigned packets, 963 zero signature slots and 975 conditional rows.
Root made 41 read-only RPC requests in seven HTTP calls: 15 actual fee quotes plus 934 unquoted Write projections give 4,815,000 projected fees.
Same-H2 required envelope is 4,594,387,560 lamports including one required 650,240 residual; conditional headroom is 102,076,760. Buffer refund 8,569,960 belongs only to G0.
Two recipients remain absent; existing ALT reuse adds no rent. Absent vault 0 has no source rent predicate, but that runtime case was not proved by the prior funded local fixture.
One blockhash was valid only at the recorded freshness read; no whole-upload lifetime, funding sufficiency, real anchor, artifact or bootstrap-trust selection follows.
No new Bank/build/private/sign/simulation/send/funds; prior local runtime is retained. Next finalize the concrete startup trust/version/funding/lifetime/resume package and Clock-before-votes binding.
Integration only, main 824342d/protected 3677fee, reserve 2 GiB; consumed scopes are not renewed. See [consolidated evidence](PIV1_TESTNET_CONSOLIDATED_STARTUP_BUDGET.json).

**Retained checkpoint — local deployment, governed initialization and heartbeat pass (2026-10-21 Europe/Warsaw).**
D-048 conditional permission is retained: **READINESS UNMET / NO LIVE TRIGGER / PUBLIC NOT READY**.
Final v5 passes 11 local entries/23 zero signature slots/115,000 local fees; root/separate audits verify 13 maps, 633 account records and 46 ordered CPIs (41 initializer).
Program/PD and all 16 PIV targets start absent; native deployment, four local Squads approvals, paused initialization and activity-only heartbeat succeed.
Measured initializer CU/loaded bytes are 992,404/3,833,006; same-H2 final is 104,027,480 and G0 final 1,008,434,960, including G0-only buffer refund 8,569,960.
Four host builds and three processes are preserved, including compiler and harness failures; the final complete pass does not erase them.
Buffer/recipient/external/ALT/Clock fixtures and unsigned Bank 4.2 limit this proof: no live control, upload, whole lifecycle or Testnet 4.4 replay; artifact/bootstrap trust/real anchor remain unselected.
Next consolidate the exact public startup/trust/version/resource/funding package and freeze the real Clock anchor before storage/votes. M4 OPEN_UNWAIVED/stopped drafts and consumed scopes remain untouched.
No new RPC/private/signing or LIVE funds/authority/deployment; 367 retained tracked inputs and 63 source pins matched before this documentation metadata change.
Integration only, main 824342d/protected 3677fee, reserve 2 GiB. See [local startup evidence](PIV1_TESTNET_STARTUP_GOVERNED_INITIALIZER.json).

**Retained checkpoint — offline initializer reuses an existing ALT (2026-10-21 Europe/Warsaw).**
D-048 conditional permission is retained: **READINESS UNMET / NO LIVE TRIGGER / PUBLIC NOT READY**.
Root v2 passes seven host groups and template/null commands (48 missing inputs); root/separate audits check 18 synthetic packets and 38 zero signature slots.
The supplied 568-byte ALT preserves full-u64 slots, exact 16 targets and strict later-context checks; no ALT setup or reuse-only authority signature is emitted.
V1 had six passing groups and one fixture failure before the planner call; only the fixture changed, then v2 passed. Both runs remain preserved.
Snapshots are unauthenticated/not refreshed; zero new ALT rent outflow is not a fee, refund or complete funding quote. Prior local runtime evidence is retained, not rerun.
Next assemble the exact governed initializer/trust/version/resource/funding package and freeze the real Clock anchor before storing/votes; artifact/bootstrap selection remains open.
No RPC/private/signing/funds/build/Bank or live action; M4 OPEN_UNWAIVED/stopped drafts and consumed scopes stay untouched.
Integration only, main 824342d/protected 3677fee, reserve 2 GiB. See [existing-ALT initializer evidence](PIV1_TESTNET_EXISTING_ALT_INITIALIZER.json).

**Retained checkpoint — local atomic bootstrap and deployed heartbeat pass (2026-10-21 Europe/Warsaw).**
D-048 conditional permission is retained: **READINESS UNMET / NO LIVE TRIGGER / PUBLIC NOT READY**.
One guarded host build/run passes two local transactions, 1,035/601 bytes, seven zero signature slots and 35,000 local fees.
Native loader verifies the retained ELF at the actual PIV ID; fresh Program/PD show final vault 0 authority; slot 2 production heartbeat succeeds.
Root/separate audits verify four full maps, 159 account records and one native PD-create CPI at outer 7/stack 2; heartbeat changes only G0 fee/reward 0 activity.
Measured bootstrap/heartbeat CU are 5,940/59,920; loaded bytes 895,430/897,649. Buffer/app fixtures and unsigned entry limit this proof.
Local Bank 4.2 is not retained observed Testnet 4.4/full replay; no upload/initializer/full lifecycle, real signatures or Squads votes are proved.
Bootstrap trust/artifact selection and consolidated readiness remain open; prior direct Deploy CPI stays INVALID_DIRECT_DEPLOY_CPI_DO_NOT_EXECUTE.
Next prepare the exact governed initializer/trust/version/resource/funding package. M4 OPEN_UNWAIVED/stopped drafts and consumed scopes stay untouched.
No RPC/private reads/new keys/signing or LIVE funds/authority/deployment action; no PIV rebuild; 198 inputs unchanged.
Integration only, main 824342d/protected 3677fee, reserve 2 GiB.
See [bounded local runtime evidence](PIV1_TESTNET_ATOMIC_BOOTSTRAP_RUNTIME.json); no routine approval question or live trigger follows.


**Retained preparation — unselected atomic top-level bootstrap proposal (2026-10-21 Europe/Warsaw).**
D-048 conditional permission is retained: **READINESS UNMET / NO LIVE TRIGGER / PUBLIC NOT READY**.
One offline assembly and six host groups pass: four complete wires 510/1,232/968/1,035 bytes; the final has six zero signatures.
Proposed G0/G1/G2/G3 endorsements cover the exact message with H2/PIV, but are not Squads votes or on-chain membership/hash/threshold enforcement.
G0 remains Buffer writer until inclusion and temporary initial authority; atomic handover intends final vault0, with no selected bootstrap exception.
Retained rents/fees project H2 peak 4,546,036,120 and G0 fees 4,720,000; no fresh quote, runtime, signatures or complete funding proof.
The prior direct loader-CPI recipe remains INVALID_DIRECT_DEPLOY_CPI_DO_NOT_EXECUTE; this wire pass does not close ELF/runtime compatibility.
Next prepare separate positive validation and the exact package/trust controls; no automatic fallback, routine approval request or new live action.
Zero RPC/private/new-key/sign/funds/build/Bank here; 198 retained inputs match. M4 OPEN_UNWAIVED/stopped drafts and consumed scopes stay untouched.
Integration only; main 824342d/protected 3677fee, reserve 2 GiB. See [proposal](PIV1_TESTNET_ATOMIC_BOOTSTRAP_PROPOSAL.json).


**Retained checkpoint — D-048 conditional permission retained; loader recipe blocked (2026-10-20 UTC).**
Readiness is unmet: **NO LIVE TRIGGER / PUBLIC NOT READY**; no M4 waiver or scope renewal.
One guarded host build passed; the first local Bank run failed after seven successful setup/governance steps.
The atomic step rejects loader Deploy CPI before native loader execution: **INVALID_DIRECT_DEPLOY_CPI_DO_NOT_EXECUTE**.
Eight packets contain 15 zero signature slots; local feeDetails total 75,000. No ELF deployment/canary/upload/initializer proof.
Local Bank 4.2 and official 4.4 source share this CPI restriction; loader-branch-only review was insufficient.
Four Testnet read-only RPC/two HTTP plus four official source GET; no keys/signing/live funds or PIV build; 198 retained inputs match.
Next prepare a separately reviewed, unselected top-level bootstrap/handover alternative and exact remaining package; no automatic fallback.
M4 OPEN_UNWAIVED/stopped drafts untouched; D-041/D-043/D-045/D-047 consumed; integration only, main 824342d/protected 3677fee, reserve 2 GiB.
See [actual blocked result](PIV1_TESTNET_ARTIFACT_LOADER_READINESS.json); conditional D-048 does not make this invalid recipe executable.


**Retained continuation — unsigned recipient provisioning proposal (2026-10-20 UTC).**
One atomic legacy 360-byte/two-zero-signature proposal transfers 650,240 each from
H2 to accepted HTFP/Team vaults 1/2; G0 fee quote/proposed cap 10,000 is separate.
Both recipients remain absent at 454849119; no funding occurred. H2/G0 were refreshed,
source and ALT state were not. Final height 406365015 exceeds 406364757: ARCHIVED
EXPIRED—DO NOT SIGN OR SEND. Nine read-only RPC/five HTTP and separate reviews pass.
D-039 preparation/D-046 purposes only; D-047 consumed, no new live authorization.
No private/key/sign/simulation/send/funds/application tests; 198 retained inputs unchanged.
M4 open/unwaived/stopped drafts untouched; M5/public NOT READY/PIV NOT AUTHORIZED.
Next prepare artifact compatibility/exact governed package; later live actions need scope.
Integration a9944ed/main 824342d/protected 3677fee; preserve 2 GiB and immutable prior records.

**Retained continuation — D-047 funding and ALT finalized, scope consumed (2026-10-20 UTC).**
Root finalized exactly one 4.7-SOL source→H2 credit, one ALT create and exact 16
extension at 454845162/454845438/454845694; fees 5,000/10,000/10,000, total 25,000.
Actual ALT 4SoVCaX8tzTE1R6jGMJTArPq75afvBtVyr93Mm6hbTFF holds 3,535,680;
final 454845700 verifies owner/authority G0/active/all 16 and strict later-slot warmup.
Final H2/source/G0 are 4,696,464,320/128,763,968,087/999,980,000 lamports.
These are factual balances, not future spending authority. D-047 is consumed: no repeat.
Root and separate phase/aggregate reviews verify five signatures/full deltas.
92 RPC/90 HTTP comprise 86 ordinary reads, three signed simulations and three sends.
No retry/new keys/program-key read/recipient/PIV/authority action beyond authorized ALT.
Canonical Rent/image and exact actual debits pass; no inclusion-time atomic rent cap.
Old tests/Bank/SBF and D-046 expired archives are retained, not rerun or rewritten.
M4 open/unwaived/stopped drafts untouched; M5/public NOT READY/PIV NOT AUTHORIZED.
Next prepare remaining package under existing mandate; later sensitive live actions need exact scope.
Integration baseline 3b3ee0f; main 824342d/protected 3677fee unchanged; preserve 2 GiB.

**Retained continuation — D-046 profile and exact unsigned funding/ALT packets (2026-10-20 Europe/Warsaw).**
Clean takeover is integration `77968ce` as jerem/WeatherTrader2; main `824342d`,
protected Task 2.3 `3677fee`. The founder accepts the proposed profile and requests
exact funding/ALT packet preparation only. Accepted roles/slots/permutation, vaults
0/1/2 and recipient purposes, pause=true and 1x capacity policy now populate the
package. Actual anchor/artifact/funding/live ALT/transport remain unset.
Root derives a new candidate from actual finalized SlotHashes, then prepares three
complete unsigned packets: source→H2 4.7 SOL, ALT create and exact 16 extension.
Full sizes 215/346/858 count 1/2/2 zero signatures; observed fees 5,000/10,000/10,000.
Candidate 4wDohUABtup3qve9qxXYHEqif3QcbjKKpUqZWsKeWMcS was absent at 454837433.
R56/R568=934,720/3,535,680; H2 projected 4,696,464,320 requires actual prior funding,
not simulated state. Source/G0/H2 observations prove no private-key control or send.
Final freshness proves blockhash expired and ALT recent slot left SlotHashes.
All three wires are ARCHIVED EXPIRED—DO NOT SIGN OR SEND; 11 read-only RPCs/five HTTP
batches and separate actual-wire/freshness reviews are bound without rebinding.
No recipient packet, keys/private reads, signing, simulation, send, funds, ALT or
authority operation, build/application tests/Bank/SBF. Old runtime evidence is retained;
current pool/referrer/Squads facts and artifact selection are not refreshed.
D-046 is profile acceptance/preparation only; D-041/D-043/D-045 remain consumed.
M4 open/unwaived, stopped drafts untouched; M5/public NOT READY/deploy NOT AUTHORIZED.
Next requires exact future scope plus fresh state/lifetimes before any signing/send.
Root owns reviewed integration publication; preserve evidence and the 2-GiB reserve.

**Retained continuation — consolidated authorization card proposal (2026-10-20 Europe/Warsaw).**
Clean takeover is integration `6818e62` as jerem/WeatherTrader2; main `824342d`,
protected Task 2.3 `3677fee`. A non-executable card consolidates proposed vaults
0/1/2, G0..G5 slots/permutation [3,1,4,0,5,2], G0 roles and exact ALT16 contents.
It recommends initial pause=true and technical loader1x, without selecting either
or the artifact. KIF anchor remains null: propose a fresh official Clock literal
before initializer storage, then exact signed/voted bytes, fixed 30-day periods; pause does not stop Clock/KIF.
A proposed future one 4WK→H2 credit is 4,700,000,000 lamports, source fee cap 10,000;
no G0 refill. Conditional H2 model is 4,611,938,960, counting one required initializer residual,
with 88,061,040 unallocated headroom, not spending authority. G0 fees are separate.
Actual live ALT/recent slot, fees, funding packets and total wallet sufficiency remain
unproved. Funding/ALT/two-recipient scope would require future explicit authorization;
it would not authorize buffer upload/handover, deployment, initializer or economics.
Existing selected package inputs remain unchanged; role/purpose acceptance is false.
This documentation task has zero RPC/private reads/keys/signing/simulation/submission/
funds/authority/build/tests. Prior 3/40 RPC records and runtime evidence are retained,
not refreshed. No signer/helper/runtime readiness is claimed or new decision made.
D-039 preparation; D-041/D-043/D-045 consumed. M4 open/unwaived, stopped drafts
untouched; M5/public NOT READY, deployment NOT AUTHORIZED. Next resolve explicit
profile/purpose acceptance and separately scoped fresh ALT/funding/provisioning review.
Root owns reviewed integration publication; preserve evidence and the 2-GiB reserve.

**Retained continuation — proposed governed loader recipe (2026-10-20 Europe/Warsaw).**
Clean takeover is integration `1628ff0` as jerem/WeatherTrader2; main `824342d`,
protected Task 2.3 `3677fee`. A concrete unapproved loader-v3 recipe derives a
G0 seeded Buffer without a new key, atomic create/initialize, and 936 bounded
uploads. Proposed buffer-only handover to Squads vault0 precedes fresh full-byte
verification and votes; no hot PROGRAM authority or onchain hash guard is claimed.
Final Program creation and Squads deployment are atomic: 815 bytes, G0/H2/existing
PIV program outer signatures, vault CPI signature only. Separate review checks
956/957-byte upload boundary, direct compact318 and 945 packets/953 zero slots.
Conditional 1x/2x H2 peaks are 4,552,746,800/9,097,070,960 lamports; Buffer returns
to the same H2 before ProgramData funding. Optional H2 reserve is a budget choice,
not a loader rule. G0 fee projection is 4,765,000, with no fresh recipe fee quote.
Other prices are retained; root's three new read-only RPCs check genesis, named
feature accounts and rent419. The named old-SBPF disable account is absent, not
full runtime/deployability proof. No wallet refresh, keys/private reads, signing,
simulation, sends, funds, authority operation, artifact execution or new build/runtime.
Existing selected inputs remain unchanged; recipe/capacity/artifact choices are unaccepted.
D-039 preparation continues; D-041/D-043/D-045 stay consumed. M4 open/unwaived,
stopped drafts untouched; M5/public NOT READY, deployment NOT AUTHORIZED.
Root owns reviewed integration publication; preserve evidence and 2-GiB reserve.
The single M5 register owns readiness; exact future handover/funding/deployment need scope.
Next consolidate remaining recipient/ALT/funding/startup inputs into one exact founder approval card.

**Retained continuation — proposed governance, ALT and budget (2026-10-20 Europe/Warsaw).**
Clean takeover is integration `76cbe9d` as jerem/WeatherTrader2; main `824342d`,
protected Task 2.3 `3677fee`. The public governance/ALT/budget proposal is concrete
and unapproved: vault indices 0/1/2, PIV slots G0..G5, permutation [3,1,4,0,5,2],
six revision-0 reward PDAs, G0 creator/executor/fee payer/ALT authority, G0..G3
approvers and distinct H2 rent payer. Actual selected package fields remain null.
Root made 40 read-only official Testnet RPC requests in seven HTTP batches, and ran
one offline sizing run. Separate review checks 12 unsigned packets/22 zero slots,
ALT16 and the conditional rent cashflows. Eleven contextual fee quotes sum to
100,000 lamports; execute fee is null after the missing-ALT rejection, so no full
fee budget or cap is established. Conditional H2 envelopes are 51,887,120 shared34
or 52,222,400 distinct35 rent-only lamports, before fees/operations/deployment.
G0 alone receives the buffer refund; it never reduces H2 funding. H2 is absent/zero.
Loader allocation components and operational advances are conditional, not a real
peak budget. ALT/live indices, startup pause, KIF anchor, funding, test capital and
artifact deployment selection remain open. No keys/private reads, signing, simulation,
sends, funds, deployment or authority changes; old tests/Bank/SBF are retained.
D-041/D-043/D-045 remain consumed; the source's additional 10 SOL has unknown provenance.
M4 remains open/unwaived, stopped drafts untouched; M5 preparation/public NOT READY,
deployment NOT AUTHORIZED. Root publishes reviewed integration only; preserve the
public proposal/evidence and 2-GiB reserve. The single M5 register owns readiness.

**Retained continuation — offline PIV address map (2026-10-20 Europe/Warsaw).**
Clean takeover is integration `7c8d926` as jerem/WeatherTrader2; main `824342d`,
protected Task 2.3 `3677fee`. The [public address map](PIV1_TESTNET_PROGRAM_ADDRESS_MAP.json)
binds the actual D-045 program ID to 12 deterministic addresses: ten fixed genesis
targets, virtual PivAuthority and loader-v3 ProgramData. Exact seeds, canonical
bumps, derivation domains, target indices and source-expected owners/sizes are public.
PivAuthority is not an allocated genesis account; ProgramData total size/rent is unknown.
Six reward addresses remain null: PIV guardian slots are unselected. Genesis revision
0 is a source constant only, not an observed current revision or a new economic choice.

Root executes one public offline helper with 22 pinned inputs. Separate independent
SHA-256/Base58/Edwards arithmetic confirms all 12 addresses and canonical bumps.
This is host derivation evidence, not account existence/absence/virginity, balances,
prefunds, actual ownership, current authority, rent, resource or runtime proof.
No private inspection, new keys, RPC, signing, transactions, funds or deployment occurs.
Old tests/Bank/SBF, artifact binding and chain observations are retained, not refreshed.
Only this deterministic mapping gap closes. ALT, funded recipients/vault indices,
PIV slots, pause/KIF anchor, funding/fees and artifact deployment selection stay open.
D-041/D-043/D-045 remain consumed; H2 was last observed empty. M4 open/unwaived,
stopped drafts untouched; M5 preparation/public NOT READY/PIV deployment NOT AUTHORIZED.
The single M5 register owns readiness. Root publishes reviewed integration only;
preserve receipts and the 2-GiB reserve. Git and
`/tmp/piv1-m5-program-address-map-20261020-a/publication.json` record identity.

**Retained continuation — Testnet program identity generated once (2026-10-20 Europe/Warsaw).**
Clean takeover is integration `9f3459a` as jerem/WeatherTrader2; main `824342d`,
protected Task 2.3 `3677fee`. Founder **“je confirme et j'autorise”** authorizes
D-045's exact one-key scope. Actual disposable Testnet program public identity:
`GAjHRaC3XM98SoRoFy5B8P2ypTS1ndf1UPhXmjWgtPux`.
[Public setup](PIV1_TESTNET_PROGRAM_IDENTITY_SETUP.json) records completed D-045,
now consumed: no regeneration, retry or further sensitive action is authorized.

Helper v2 `7f041107db97952d31819aa6027fd4f90f5846464455838518e242508a6be14d`
passes separate source review; root executes once and independent public-result
review passes. New outside-Git directory0700/file0600 belongs to jerem, fsynced;
one new-descriptor read validates64-byte JSON/public derivation without signing.
Existing private-key reads0; RPC/signatures/transactions/funds/fees/deployment/
Mainnet/authority transfer0. No private bytes/file hashes are published. Storage
is unencrypted, not independent custody; snapshot/zeroization are not assured.
Account existence/balance/loader proof stays null/unknown, without fresh RPC.

This is new host key-generation evidence only. All198 retained inputs stay intact;
old tests/Bank/SBF, artifact/source binding and D-041/D-043 chain evidence are not
rerun/refreshed. Setup/evidence retain pre-execution metadata/resolver/Path.open
corrections; none is a failed generation/application run. Proposal files stay immutable.
Only public ID is selected; artifact deployment selection, funded fee/rent package,
ALT authority/table, slots/vault indices/recipients, pause/KIF anchor/test amounts
remain open. D-041/D-043 consumed; H2 last observed empty. M4 open/unwaived/stopped
drafts untouched; M5 preparation/public NOT READY/PIV deployment NOT AUTHORIZED.
Root publishes reviewed integration only; preserve receipts and 2-GiB reserve.
Git and `/tmp/piv1-testnet-program-identity-execution-20261020-a/publication.json` record identity.

**Retained preparation — dedicated PIV program identity proposal (2026-10-20 Europe/Warsaw).**
Historical pre-D-045 scope; only the completed one-key authorization/result above supersedes it.
Clean takeover is integration `a42aed6` as jerem/WeatherTrader2; main stays
`824342d`, protected Task 2.3 `3677fee`. One configured Sol/High documentation
writer and separate Sol/Extra High review prepare the exact generation-only
[program identity proposal](PIV1_TESTNET_PROGRAM_IDENTITY_PLAN.md).
The proposal is NOT AUTHORIZED: one disposable Testnet Ed25519 program key,
zero guardians/payers/ALT keys, zero RPC/signing/transactions/funds/fees/deployment.
D-041 explicitly excluded the PIV program key; D-041/D-043 are consumed. A routine
continuation does not authorize this sensitive action. All existing private keys,
including the original funding source, remain outside the proposed scope.

Root freshly matches candidate894,552-byte ELF/e29735e1, its header, all132
production/host source files against HEAD/worktree/retained binding, retained build
receipt/log hashes and198 other retained inputs. Native runtime-ID wiring supports
later identity binding without a static-ID source edit; no real-address deployability,
new build/reproduction/runtime, application test or chain observation is claimed.
Existing tests/Bank/SBF and D-043 evidence remain retained, not rerun/refreshed.

The future key would use a new outside-Git0700 directory/0600 exclusive file owned
by jerem, no symlinks/overwrite/regeneration/backups/global wallet changes. Existing
paths or partial failure require STOP. Only public-key/completion metadata may be
published; no private content/hash/log/chat/Git exposure. Ordinary local JSON is
unencrypted, not independent custody; provider snapshots/zeroization are not assured.
No private directory/file/key is created or inspected by this preparation.

Next obtain explicit approval for that one-key generation scope, then bind the
actual public identity during authorized preparation; future funding/signing/PIV
deployment/authority transfer remain separately gated. No guardian slots, vault
indices, recipients, initial pause, KIF anchor or test amounts are silently chosen.
M4 open/unwaived/stopped drafts untouched; M5 preparation/public NOT READY/PIV
deployment NOT AUTHORIZED. Root publishes reviewed integration only; Git and
`/tmp/piv1-m5-program-identity-preparation-20261020-a/publication.json` record identity.
Preserve all proposal drafts, source/artifact receipts and the 2-GiB reserve.

**Retained continuation — offline one-seat guardian synchronization (2026-10-20 Europe/Warsaw).**
Clean takeover is integration `32aae62` as jerem/WeatherTrader2; main stays
`824342d`, protected Task 2.3 `3677fee`. One configured Sol/High writer and separate
Sol/Extra High review add a public-manifest PIV synchronization planner, seven Node
groups, nullable example and append-only CLI usage. Production/economics, old
executable clients, dependencies, toolchains and ELFs are unchanged.

Root's first suite passes 7/7 plus template/null CLI; the null example has 38
missing unconditional inputs and plan null. Three actual synthetic CLI reports
retain 25 full unsigned legacy packets (direct7, buffered9/9). Independent cursors
bind PIV1UG01/51 bytes/24 roles, compact853/create868, old/current/new set distinction,
full-u64 revision+1/slot PDAs and the external native rent payer's execute signature.
Direct create1195 fits; 1291/1387 use canonical buffer first chunks761/665, both1232
bytes, remaining92/188 and finish477/573. Execute997/1093/1189 fits for1/2/3 signers.
All packets carry three explicit resource requests; no ALT/top-level vault signature.
Buffer closure refunds creator, not automatically the parent rent payer.

This is only PIV synchronization AFTER a separately approved/executed external
Squads member change. The current NEW set approves this exact one-seat action;
old PIV slots/revision authenticate old rewards. It proves neither old-set votes,
historical same-multisig continuity nor actual membership change. External Squads
rotation composition remains unimplemented. Six next-revision reward PDAs need
current rent/prefund checks and an external signing payer retaining its rent floor.
Old earned records/liabilities and frozen distributions persist; unchanged identities
retain prior activity. No actual current state/control/votes/funding/fees/resources,
new account allocation, on-chain synchronization or live eligibility is established.

Only new host/CLI results are claimed; old tests/Bank/SBF and D-041/D-043 chain
observations are retained, not rerun/refreshed. Pre-execution arithmetic, test-fixture
and root-parser syntax corrections are preserved; no product defect/failed app run.
All 198 other retained inputs and prior CLI README59,945-byte prefix persist.
M4 open/unwaived/stopped drafts untouched; M5 preparation/public NOT READY/PIV
deployment NOT AUTHORIZED. D-043 consumed/H2 empty. No RPC, keys/signing, submission,
fund movement, authority transfer, deployment or Mainnet action occurs here.
Next assemble the exact funded PIV identity/artifact/ALT/recipient/founder package;
external member-change preparation stays a separate open scope. No actual guardian,
slot/vault index, pause or KIF anchor is selected. Root publishes reviewed integration
only; Git and `/tmp/piv1-m5-guardian-sync-plan-20261020-a/publication.json` record identity.
Preserve all attempts, public reports, reviews and the 2-GiB reserve.

**Retained continuation — offline governed recipient replacement (2026-10-20 Europe/Warsaw).**
Clean takeover as jerem/WeatherTrader2 is `824342d`; main stays there and protected
Task 2.3 stays `3677fee`. One configured Sol/High writer and separate Sol/Extra High
review add one offline public-manifest planner, seven Node groups, nullable example
and append-only usage. Production/economics, existing executable clients,
dependencies, toolchains and ELFs are unchanged.

Root's first suite passes 7/7; standalone template/null commands pass, with 33
missing fields and plan null in the example. Three actual synthetic CLI reports
(shared/middle/distinct actors) retain 21 complete unsigned legacy packets.
Independent source/PDA/byte cursors bind native PIV1UR01/12 bytes/18 roles,
616-byte compact and 631-byte create data. Full create958/1054/1150,
proposal343/439/535, approval301/397, execute831/927 count every signature slot.
No ALT/buffer/top-level vault signature is needed. Explicit recipient indices and
addresses bind distinct current-multisig vault PDAs; PIV guardian slot order and
full-u64 revision/indices remain exact. Runtime funding/System ownership/current
authority/four actual votes/Clock/timelock/resources/eligibility are not proved.
Native update writes only current Config recipients; active rounds keep frozen
recipients. No native economic/rent movement; parent Squads creation needs external
rent. Supported recipients are funded empty System vault PDAs, not general
program-owned destinations or proof of exclusive 4/6 spending control.

These are new host/CLI executions only. Old tests/Bank/SBF and D-041/D-043 chain
observations are retained, not rerun/refreshed. The pre-execution test-fixture
context correction is preserved in v1/v2; it is no product defect or failed test run.
All 198 other retained inputs and the 55,442-byte prior CLI README prefix persist.
M4 open/unwaived/stopped drafts untouched; M5 preparation/public NOT READY/PIV
deployment NOT AUTHORIZED. D-043 consumed/H2 empty; no RPC, keys/signing, submission,
funds, authority transfer, deployment or Mainnet action occurs in this task.
Next finish remaining guardian-change/synchronization composition and exact PIV
identity/ALT/recipient/funding/founder package; no slot/index/pause/KIF-anchor choice
is inferred. Root publishes reviewed integration only; Git and
`/tmp/piv1-m5-recipient-plan-20261020-a/publication.json` record identity.
Preserve all frozen drafts, public CLI reports, reviews and the 2-GiB reserve.

**Retained continuation — offline governed pause/release operator (2026-10-19 Europe/Warsaw).**
Clean takeover `92d9d0f` as jerem/WeatherTrader2; main stays `e8f33f3`, protected
Task 2.3 `3677fee`. One bounded Sol/High writer and separate Sol/Extra High review
add one public-manifest pause planner, seven tests, null example and append-only
usage. Production/economics, old client executables, dependencies, toolchains
and ELFs are unchanged; root model/speed unexposed.

Root passes seven new Node groups plus standalone template/null commands; null
example reports 30 missing inputs/plan null. Six actual synthetic CLI reports
(pause/release × shared/middle/distinct actors) retain 42 complete unsigned legacy
packets. Root and separate review independently decode native 11-byte PIV1PS01,
16 roles, 549-byte stored compact, direct creation/proposal/four approvals/execute,
full-u64 revision/index seeds, slot order and all global privileges/signature slots.
Create 891/987/1083 bytes, proposal 343/439/535, approval 301/397, execute 765/861.
No ALT/buffer or top-level vault signature; stored governance stays readonly while
the outer proposal is writable. Explicit bool sets pause/release, never toggles.
Requested resources/current declarations are not authority, actual votes, fees,
rent, live eligibility or runtime success. Native pause moves no economic funds;
Squads transaction/proposal accounts require external payer rent.

These are new host/CLI results only; prior tests/Bank/SBF/D-043 chain evidence
are retained, not rerun/refreshed. No RPC, keys/signing, transaction, funds,
PIV deployment, Mainnet or authority transfer. The 198 other retained inputs and
49,050-byte old README prefix are preserved. Partial-alias/source wording fixes
precede first tests; root guard/parser inspection corrections are preserved and
are not failed application runs. M4 open/unwaived/stopped drafts untouched;
M5 preparation/public NOT READY/PIV deployment NOT AUTHORIZED. D-043 remains
consumed, H2 empty, six guardians last observed 1 SOL each; no new live scope.
Next complete exact PIV identity/ALT/recipient/funding and remaining governed
operator package. No slot/vault-index/initial-pause/KIF-anchor choice is inferred.
Root owns reviewed integration publication; Git and
`/tmp/piv1-m5-pause-plan-20261019-a/publication.json` record identity.
Preserve all receipts/synthetic CLI reports and the 2-GiB reserve.

**Retained continuation — D-043 finalized Testnet Squads 4/6 and atomic return (2026-10-19 Europe/Warsaw).**
Root executed the founder-authorized 668-byte transaction once after one bounded
Sol/High writer and separate Sol/Extra High source review. Actual finalized slot
454165126, fresh reread 454165136: multisig `9mwto7RNokYc8n8MPtzHpubwwnKcborY2r9RnfZAScVS`
has the exact six members/mask 7, threshold 4, autonomous authority, zero timelock,
no rent collector, and full 330-byte oracle. Both actual Ed25519 signatures,
all eight transaction balances and fourteen-account snapshots pass root audits;
separate actual-evidence review is retained with the checkpoint.

Payer 1 SOL becomes zero/purged; original 4WK source receives 997,663,360 lamports
and now has 123,463,973,087 at that read. Rent 2,326,640 plus network fee 10,000 =
net own cost 2,336,640 (cap 3,000,000). All six guardians remain 1 SOL; Config,
program/image, treasury absence and all other accounts remain unchanged.
One successful signed simulation and one finalized external-Squads execution;
15 RPC calls include 14 read-only calls and one send. No new keys, PIV deployment,
authority transfer or Mainnet action. One-shot D-043 scope is consumed; no refill
or other spending. Old drafts remain disabled and preserved.

See `docs/PIV1_TESTNET_SQUADS_SETUP.json` and the single M5 readiness register.
Baseline `be4b387`; main stays `e8f33f3`, protected Task 2.3 `3677fee`.
All 199 production/client/dependency/toolchain/artifact inputs stay unchanged;
old host/Bank/SBF tests are retained, not rerun. M4 open/unwaived and stopped
drafts untouched; M5 preparation/public NOT READY/PIV deployment NOT AUTHORIZED.
Next prepare the exact remaining PIV identity/ALT/recipient/governed transport and
funded founder package without inventing slots/vault indices/pause/KIF anchor.
H2 is empty; later funding requires its own explicit scope. Current creation does
not prove four public approvals, independent custody or a PIV lifecycle.
Root publishes reviewed integration only; preserve
`/tmp/piv1-squads-atomic-execution-20261019-a`, receipts and the 2-GiB reserve.
Git and its publication receipt record the final reviewed checkpoint identity.

**Retained continuation — authorized Squads creation held on the exact debit cap (2026-10-19 Europe/Warsaw).**
Founder reply “oui j'autorise” approves the exact reviewed 619-byte Testnet
Squads creation/profile, total payer debit at most 3,000,000 lamports (D-042).
Takeover is clean integration `ca9bc19` as jerem on WeatherTrader2; main stays
`e8f33f3`, protected Task 2.3 `3677fee`. Fresh official read-only observations
at slot 454158959 find the candidate absent, all seven wallets 1 SOL, unchanged
external image/Config, creation fee 0 and rent 2,326,640. No signing/send occurred.

One configured Sol/High source writer and separate Sol/Extra High review find
that `multisig_create_v2` has no maximum creation-fee argument: mutable Config is
read at inclusion. Fresh quotes/simulation cannot atomically enforce the agreed
cap against a later fee change while the payer holds 1 SOL. Execution helper is
preserved UNEXECUTED; its approval hash stays null. This is a newly demonstrated
budget-control limitation, not an M4/runtime regression or founder rejection.

A separately prepared atomic refund-plus-creation proposal returns unused payer
funds to their original 4WK source before the same Squads instruction. Root
successfully simulates the unsigned 668-byte/two-signature packet; fee 10,000,
refund 997,663,360, rent 2,326,640 and final payer zero. All actual balances remain
unchanged. Additional refund/changed-envelope approval is required; the existing
creation/profile authorization stays recorded. See the single readiness register for exact
amounts, unsigned evidence and limitations. Do not execute either draft helper
automatically. Production/economics, clients, dependencies, toolchains and ELFs
are unchanged; prior tests/runtime are retained, not rerun. M4 open/unwaived,
stopped drafts untouched; M5 preparation/public NOT READY/PIV deployment NOT
AUTHORIZED. Root publishes reviewed integration only; preserve evidence in
`/tmp/piv1-squads-create-execution-20261019-a` and the 2-GiB reserve.

**Retained continuation — offline Squads creation plan and unsigned Testnet simulation (2026-10-19 Europe/Warsaw).**
Clean takeover was integration `0b1a19e` as jerem on WeatherTrader2; main remains
`e8f33f3`, protected Task 2.3 `3677fee`. One configured Sol/High writer and separate
Sol/Extra High review add one offline public-manifest planner, seven focused groups,
null example and append-only usage. Production/economics, old executable clients,
dependencies, toolchains and ELFs are unchanged; root model/speed unexposed.

Root passes seven new Node groups plus template/null CLI: 720 member permutations,
full-width declared costs and literal complete wire/roles. B's two actual-input CLI
reports (unknown/known fee) encode the same 619-byte legacy packet, 219-byte payload,
six native roles and two zero signature slots. Thirteen unique read-only official
Testnet RPC requests include one API account-return-limit rejection before execution
and one successful unsigned Squads creation simulation. Simulated account 330 bytes
(logical 298 plus 32 zero tail), six sorted members/mask 7, threshold 4/autonomous,
proposed timelock 0/rentCollector None are checked. Fee 10,000 + rent 2,326,640 =
2,336,640 lamports; CU 18,984 / loaded 980,726 bytes. These are simulation observations,
not verified signatures or committed creation. All seven actual wallets remain 1 SOL;
multisig remains absent. No keys/signing, send, funds or PIV operation occurred.

See the [exact public proposal](PIV1_TESTNET_SQUADS_CREATION_PLAN.json) and
[single register](PIV1_M5_TESTNET_READINESS.md). Next obtain explicit founder
approval for this one creation, proposed total payer cap 0.003 SOL; D-041's wallet
funding does not authorize it. Fresh context and same immutable profile/roles/body
are required before future signing. Accepted six guardians/4-of-6 remain unchanged.
M4 open/unwaived, stopped drafts untouched; M5 preparation, public NOT READY,
PIV deployment NOT AUTHORIZED. Root publishes reviewed integration only; Git and
`/tmp/piv1-squads-create-preparation-20261019-a/publication.json` identify completion.
Preserve all evidence and the 2-GiB reserve. No new Rust/Bank/SBF build or PIV runtime execution.

**Retained continuation — authorized Testnet identities and wallet funding (2026-10-19 Europe/Warsaw).**
D-041 authorizes only seven disposable Testnet keys and up to 1 SOL per address
from `4WKQg3Sm8bvHS8DBmxoiMMi4Fev4mJU6ZLwGSTg6jDna`, plus network fee.
Root generated the seven keys privately once, then finalized one System-only
funding transaction: seven 1-SOL credits, total 7 SOL and 5,000-lamport fee.
This one-shot scope is consumed: no refill, regeneration or further spending.
Public identities and result: [identity setup](PIV1_TESTNET_IDENTITY_SETUP.json) and [single register](PIV1_M5_TESTNET_READINESS.md).
Two prior signed simulations expired before submission and incurred no charge;
reviewed just-in-time blockhash refresh C supplied the sole submitted transaction.
Earlier no-key/no-signing paragraphs are historical, superseded only by D-041's
exact key-generation/funding scope. No Squads creation, PIV deployment, authority
transfer, Mainnet or new main-publication authority occurred or is authorized.
Accepted six-guardian/4-of-6 requirement is unchanged; exact new creation profile,
including proposed timelock0/rentCollectorNone, remains unapproved.
PIV ID, multisig, slots, vault indices, pause and KIF anchor stay OPEN. One-VPS keys are not independent custody.
Existing compatibility/artifact/source and old tests are retained; no new suite,
build or PIV lifecycle runtime. M4 criterion open/unwaived; stopped drafts untouched.
M5 preparation only, public NOT READY, deployment NOT AUTHORIZED. Next prepare
exact Squads creation/fees/zero-fee treasury preflight and obtain its authorization.
Root owns reviewed integration publication; main stays `e8f33f3`, Task 2.3 `3677fee`.
Preserve evidence and the 2-GiB reserve.

**Retained founder-package/setup checkpoint:** documentation/data only plus 11
fresh read-only official Testnet RPC requests; separate review checks source,
artifact binding and saved observations. All four deployed integration images
match retained locally exercised Testnet images. Pool is current at epoch 1064;
ProgramConfig fee 0/treasury absent and 330-byte multisig rent 2,326,640 lamports are
observations, not successful creation or total funding proof. 132 production/host
inputs match retained PIV candidate/recipe; no new build, application suite or
runtime. Existing test evidence is retained, not rerun. See the
[workflow](PIV1_TESTNET_FOUNDER_WORKFLOW.md) and
[register](PIV1_M5_TESTNET_READINESS.md) for exact hashes, scope and limitations.

The founder requests six dedicated Testnet guardians/4-of-6 Squads prepared by
the pilot; key generation and transactions require the explicitly requested
prior authorization. Reviewed proposed Stage A: seven local keys including separate
payer, private storage outside Git, directories 0700/files 0600, no key material in logs/chat/Git.
Stop after public manifest; Stage B needs actual reviewed packet/addresses and
fresh funding/fee caps before authorization. No independent custody claim,
Devnet/protocol substitution or implicit PIV deployment. Next obtain scoped Stage A
approval after review, then close actual-identity/funding/transport gaps. M4 open/
unwaived, stopped drafts untouched, M5 preparation/public NOT READY/deployment
unauthorized. Main stays `e8f33f3`; reviewed integration only.

**Current guardian-inspection host evidence:** root passes 7 new groups plus
standalone template/null-example on first execution. Four complete synthetic CLI
snapshot/report pairs receive two independent Python source-derived Borsh/PDA audits:
18 supplied records, every named Config/registry/reward field, metadata/layout/hash,
canonical owner/address/bump/tuple declarations and raw observed counter summaries.
Registry210 and Reward84, strict Option0/1, None8-byte zero tail, full-width unsigned
values, standalone earned>=claimed/claimable identity, current versus historical
isolation, unknown/unused/known-partial/CLI inputs are covered by finite host oracles.
No global historical sum, real activity/claimability or full Rust semantic proof.

All 211 frozen inputs match. Literal fixtures explicitly lack rent/global economic
reconciliation; these are parser/source results, not new Solana runtime or retained
Bank trace review. Old Node/Rust/Bank/SBF suites and results retain their own scopes.
Unexecuted report-scope wording and malformed-partial test refinements are preserved;
no application test/runtime failure. M4 open/unwaived, M5 preparation/public NOT READY,
deployment unauthorized. See the [register](PIV1_M5_TESTNET_READINESS.md).

**Current guardian/KIF client host evidence:** root passes 7 new groups plus
standalone template/null input on first execution. Four complete synthetic CLI
reports receive independent raw byte/PDA/source review: heartbeat601/697 full bytes
and claim382/478 with one/two full zero-signature slots. Tests bind HB11-byte codec/
13roles, Anchor24-byte claim/5roles, exact u64 fields/slot/vault byte ranges, canonical
loader/Squads/reward derivations, only-selected HB reward writable, old earned tuple
isolation, action/partial/alias/resource/CLI failures and global payer privilege unions.
HB includes its own instruction program as native account; shared guardian payer
becomes globally writable. No new liability, activity time, claimability or runtime
success is inferred. Direct EOA encoding does not remove runtime PDA/CPI support.
All 207 frozen inputs match; no old suite/Rust/Bank/SBF/RPC run. Prior evidence
retains its scope. A design-only claim size estimate was corrected before code;
its original is retained. Known-partial direct-signer validation was tightened
before execution, with the first module draft preserved; no test/runtime failure. M4 open/unwaived, public NOT
READY, deployment unauthorized; see the [register](PIV1_M5_TESTNET_READINESS.md).

**Current withdrawal-leg client host execution:** root passes 7 new Node
groups and standalone template/null-example on first execution. Independent
literal native codecs/role matrices and raw full-wire/PDA audits cover four
actual CLI reports: initiation 1,030/1,063 bytes and finalization 1,001/1,034.
Each has one zero 64-byte signature, three resource requests and exact native
flags. Payloads are selector/version/u32-validator (13 bytes) and selector/
version/u64-leg (17 bytes); both temporary PDA seeds use u64 round and leg.
Full-width integers, action isolation, invalid partials/aliases, resource/CLI
bounds and missing inputs are checked. No client economic/readiness oracle.

Root and separate reviewer independently decode retained actual reports without
rerunning Node. All 204 frozen inputs match; previous production/clients/
dependencies/toolchains/artifacts remain unchanged. Tests use synthetic declarations,
not live state or new Solana runtime/Stake execution. No older suite, Rust build,
Bank/SBF or RPC is newly run. Source rent/pending/reward/loss/recovery and HWM
limits are documented, not demonstrated by these wire tests. M4's stopped criterion
remains open/unwaived; M5 preparation/public NOT READY/deployment unauthorized.
Evidence and exact commands are in the [register](PIV1_M5_TESTNET_READINESS.md).
The D-040 and prior client results below are retained evidence.

**D-040 publication evidence (2026-10-19 Europe/Warsaw):** root reconciles all
ten existing commits after `a524cc6` through `78ab8a4` against exact retained final
review/publication trees and hashes. A separate bounded review checks eligibility
and the six-document authorization checkpoint before normal main fast-forward.
No application tests, builds, Solana runtime or RPC are new; all results below
remain retained evidence in their individual scopes. No production/dependency
change is introduced. Git and
`/tmp/piv1-main-publication-20261019-a/publication.json` identify completion.
M4 criterion is open/unwaived; publication does not establish public readiness,
founder acceptance or deployment authorization. Future publication is integration
only unless separately authorized; earlier main limits below are historical.

**Current distribution planner host execution:** root passes 7 new groups
and template/null-example on first execution. All eight action/shared/distinct CLI
reports are retained from suite subprocesses; root and separate reviewer decode
actual legacy wires, single zero signature, requested/global privilege union,
three budgets, native selector/version/index bytes and canonical guardian slot/
u64 revision PDAs. Sizes are 1,092/1,125 liquid, 1,162/1,195 withdrawal preparation,
1,125/1,158 settlement and 861/894 integration. This is host/source/wire evidence,
not new Solana runtime, live source eligibility, current balances or resources.

Preparation uses CURRENT guardian tuple, settlement FROZEN tuple/recipients and no
registry account; integration has neither. Known malformed/unused/partial input,
slot permutation, full-width u32/u64 and fee/guardian/recipient alias distinctions
are covered. No quote, target, payout, timing or economic validator is introduced.
A draft README sentence about recipient metas versus overrides was clarified before
execution; its initial freeze remains unexecuted and code/tests are unchanged.
All 83 frozen inputs match; static/evidence/final-tree review is retained.
Old suites/Bank/SBF remain retained only; no RPC/build or stopped M4 operation.
M4/public/deployment limits persist; see the [register](PIV1_M5_TESTNET_READINESS.md).
The principal-plan evidence below is retained.

**Retained protected-principal planner host execution:** root passes 7 focused
Node groups and template/null-example commands on first execution. Shared/distinct
complete CLI manifests and reports from suite subprocesses are retained, not
rerun. Independent literal ABI/source/metas and actual legacy-wire/PDA review
check 25 bytes (`PIV1SP01`, v1, u64LE amount/minimum), 19/20 native nonsigner roles,
exact writable union, one zero signature, fixed 3 budget selectors and 877/910
full bytes. Amount must be positive; explicit caller minimum spans zero..u64MAX
without a fabricated quote or native/token-unit comparison. Context/resources
remain declarations, and source preconditions remain unverified.

Root's independent Python base58/Ed25519/SHA256/shortvec decoder checks both actual
CLI outputs without importing SDK helpers. All 67 frozen inputs match; separate
review covers code, evidence and final staged tree. Earlier Node/Rust/Bank/SBF/
production suites are retained only; no new RPC/build/Solana runtime. M4's stopped
criterion is open/unwaived; public package/deployment remain unavailable. See the
[single register](PIV1_M5_TESTNET_READINESS.md). Inspector evidence below is retained.

**Retained Config/distribution inspector host execution:** root passes 7 new
focused groups and six standalone commands (template/null plus four same-phase
retained local snapshots). Both source-derived Python cursors independently bind
all named decoded fields; the entire retained 558,238,168-byte trace is hash-verified,
not reexecuted. Options shift subsequent fields; exact signed/unsigned decimal
strings, strict Borsh tags, zero reserve/tail, allocation/discriminator and declared
owner/PDA/bump/cross-reference checks are bounded structural evidence.

Actual CLI phases are bootstrap Idle, WithdrawalActive, Settled and Idle with prior
completion. Last HWM is 121,955,000,003, pending zero, earned KIF liability
216,149,757 in those retained synthetic bytes. Snapshot labels/slots do not prove
provenance, coherence or freshness; no rent/custody/valuation/full-state-health or
eligibility claim follows. Frozen recipients/guardian revision are not required to
equal later Config. All 69 frozen inputs match. A draft guardian-revision inequality
oracle was corrected before execution; no failed test or production defect resulted. Separate static/evidence/final-tree
review is retained. No new RPC/build/Bank/SBF/prior suite; M4/live limits persist.
See the [register](PIV1_M5_TESTNET_READINESS.md). Intake evidence below is retained.

**Current contribution/bootstrap host execution:** root's first seven-group suite
and standalone template/null-example commands pass. Literal 17/17/9-byte codecs,
6/8/18-or-19 native roles/PDAs and both bootstrap receiver shapes agree with source.
Fourteen positive-u64 values per intake preserve exact bytes/roles; SDK tests check
actual one/two signer unions, token-owner writable escalation and off-curve source
account support. Eight compute/loaded request combinations are encoded; 225 legal
heap choices are validated, with min/max/invalid/alignment and action/schema/alias
negatives. These are finite host domains, not exhaustive live-state coverage.

All three complete synthetic CLI reports/manifests are retained. Root separately
decodes actual headers/shortvec/static keys/instructions/zero slots/global flags,
resource bytes and amounts: SOL 504B/two slots, Jito 474B/one, shared bootstrap 828B/one.
The separate reviewer checks actual evidence without rerunning Node. Bootstrap
native authority is nonsigner; only outer fee payer signs, and current amounts/
book value/eligibility remain unknown. Finite error descriptions are source guidance,
not observed transaction diagnosis or a state inspector. All 52 frozen inputs match.
One draft null-example wrapper key was corrected before execution; no failed run
or production defect occurred. Earlier initializer 11/funding 7/production host/Bank/
SBF/runtime are retained only. No new RPC/build/Solana runtime. Public/M4/
deployment limits persist; see the [register](PIV1_M5_TESTNET_READINESS.md).

The conditional-rent execution below is retained preceding evidence.

**Current conditional rent-ledger execution:** root passed all seven new Node
groups on the first guarded run, plus standalone template and observed-report CLI.
Independent literal irregular prices and 32 positive-price conservation cases
cover both size variants, ALT/stored growth deltas, refund ordering/destination,
held balances and residual separation. Eight complete manifest cases preserve
actual one/two/three signer unions and allowed fee-payer aliases. Partial-known
aliases remain symbolic; null/malformed fields, full-width integers and CLI paths
are checked. Aggregate BigInt arithmetic is not a feasible-wallet balance proof.

Root's eighteen new read-only requests (three HTTP batches) observe Testnet genesis/
slot brackets and fourteen finalized rent sizes. Separate raw/evidence review and
root independent integer comparison verify the actual CLI's two symbolic scenarios.
All 51 frozen inputs remain exact. Draft symbolic-role/test-oracle/template-wrapper
corrections preceded execution; no failed test/run occurred. Old initializer11,
production host/Bank/SBF/runtime suites remain retained, not rerun. No production,
economic, dependency or artifact change. Exact amounts and scopes are in the
[single register](PIV1_M5_TESTNET_READINESS.md). Fees, actual balances/shortfalls,
provisioning, peak liquidity and deployment costs remain OPEN; M4 reservation and
live-operation limits persist.

The public-input planner execution below is retained preceding evidence.

**Current public-input planner host execution:** root's first guarded offline
execution passes all 11 groups with no failures/skips, followed by successful
standalone template/example commands. Eighteen complete plans compare literal
315-byte payloads and 34/35 roles across receiver/payer/vault variants; all 720
slot permutations check codecs/PDAs without constructing 720 full bundles.
Other focused assertions cover full-width decimal integers, buffer commitment/
reassembly, exact ALT u64/16-target order, actual one/two/three signature unions,
SDK serialization/decompilation, 1,232-byte fit and adjacent rejection, malformed/
partial declarations and an actual complete synthetic CLI subprocess. The null
example has 42 missing fields, no packets and readiness false.

Separate source/command/evidence review binds 33 frozen inputs and 22 source/SDK
pins. Native/Rust/Bank/SBF/application runtime and previous suites were not rerun;
prior results remain retained evidence. Public identities/control, actual ALT
creation/extension/warm-up, real signatures, current governance/protocol state,
fees/rent/funding and other public operators remain unproved. Exact commands,
receipts and scope are in the [single register](PIV1_M5_TESTNET_READINESS.md).
M4's unwaived stopped criterion and all deployment limits persist.

The retained-image fee-cycle execution below is retained preceding evidence.

**Current retained-image fee-cycle execution:** root reused source-bound reviewed
AB executable and passed the existing exact positive-withdrawal-fee profile once
with actual retained Testnet pool/Token images. No build or source/oracle/dependency
change. Root and separate full-trace review cover 108 maps / 7,890 accounts / 55 pairs,
118 CPIs / 19 events: protected variants 25/24, exact per-leg ceiling fees/manager credits,
burn/supply, Stake epoch14/out-of-order rent recovery, net settlement and final
Idle HWM 121,955,000,003 with zero pending. A root immutable-selection analysis assertion
was corrected without an application rerun; original parser/failure are preserved.
The three-feature/synthetic-caller fixture and its two-entry 155-byte validator list
remain distinct
from actual Squads/730,009-byte-list initializer B. Maximum legacy packet 1,544 remains
non-public; observed max CU 1,064,915 is this fixture only. Raw historical version
labels are not deployed provenance. See the [register](PIV1_M5_TESTNET_READINESS.md)
for exact hashes/receipts and residual scope. No new RPC/SBF/full-suite execution;
old results retained. Next public workflow/transport/identity preparation, with M4
reservation unwaived and no deployment/main authority inferred.

The real-governance execution below is retained preceding evidence.

**Current real-governance execution:** root ran two locked/offline targeted Bank
builds successfully, one failed A run and one corrected B run successfully.
A lacks the Token image's generic-sysvar capability; B selects only that canonical
feature, also observed active by two new read-only Testnet RPC requests. Original
A evidence remains intact; no postfailure account map is claimed. B has nine
successes, ten complete maps / 461 accounts and 45 CPIs: actual Squads approvals,
native PIV initialization, two Token calls and final Executed proposal. Packet
maximum 1,215 bytes; full execute 963 bytes, 1,016,484 CU, 3,833,006 loaded bytes.
Root's separate parser checks actual wire, approval bytes, account changes, rent,
fees/refund and targets. No production/image/economic/dependency change or old suite
rerun. See the [M5 register](PIV1_M5_TESTNET_READINESS.md) for exact receipts and
review. Signed transport, seeded multisig/ALT lifecycle, replay, pool/Stake calls,
public identities and release readiness remain unproved; M4 criterion unchanged.

The native-wire execution below is retained preceding evidence.

**Current native-wire execution:** root passed the new ten-group suite on its
first execution and the first sixteen-case CLI. Independent literal ABI/roles,
all 720 codec permutations, signed-i64/u8 boundaries, selected PDA permutations,
truncation/alias/privilege/lookup/buffer-tamper negatives and actual SDK packet
roundtrips cover the bounded current-native profile. Execution v0 is 907–988
bytes with complete zero-signature storage; this is no real signature, ALT,
Squads consumption or application-runtime proof. See the
[M5 register](PIV1_M5_TESTNET_READINESS.md) for exact commands/receipts and limits.
The old historical tool/test bytes and default eight-case JSON golden are
unchanged. Their fifteen-test execution is retained, not rerun: two recipient
checks bind older source revisions and are not current-native gates. No
production host/SBF/caller/Bank suite was rerun. The following no-new-test paragraph
records the initial read-only assessment at `21cd68e`.

M5 PREPARATION is explicitly authorized with M4 TECHNICALLY INCOMPLETE and its
internal-callee-failure evidence criterion open/unwaived. Preserve the rejected
operation/drafts without retry or rerouting. The
[finite M5 readiness register](PIV1_M5_TESTNET_READINESS.md) distinguishes thirteen
new read-only RPC/public-source observations and local decoding/PDA checks from
retained host/SBF/AB evidence. No new tests/builds/runtime occurred. Matching
account fields do not establish deployed binary compatibility, real governance
consumption, current native-ABI wire/resource/signature usability or a complete
funded founder workflow. Assess each blocker and the M4 reservation in separate
readiness review before a live decision. Next bounded checks concern deployed
provenance/compatibility and offline current-native-ABI assembly; do not rerun
whole suites or invent another M4 inventory. Public package NOT READY; deployment
NOT AUTHORIZED. Earlier M5-unauthorized statements below are historical under
D-039's narrow sequencing change; their technical evidence remains scoped.

## Retained interruption — renewed platform rejection (2026-10-18 UTC)

The founder explicitly authorized revisiting the suspended local regression
(D-037). After ordinary design review, the writer received a fresh platform
cybersecurity rejection. No test/build/runtime executed. Root preserved the three
partial draft files and their diff, then restored only those verified turn changes.
No rejected operation was retried, rephrased or rerouted after this new rejection.
See the [active pilot checkpoint](PIV1_PILOT_STATE.md) and
`/tmp/piv1-m4-internal-resume-20261018-a/interruption.json`.
M4 remains incomplete, its internal-callee-failure requirement unwaived, and M5
unauthorized. Runtime AB and all prior results below are retained evidence.
Do not automatically run the incomplete drafts or prepared wrappers. Current
owner authorization does not establish platform clearance; the underlying
account/provisioning cause remains unknown. No production/economic change occurred.


**Historical D-035 — Founder-authorized main integration (2026-10-12), completed at `2f8d083`:** authorized publication of
the eight reviewed commits from `1054ff3` through `c481433` plus the technically
validated Task 2.51 fixture/settlement correction and necessary checkpoint records.
Normal fast-forward only; Git and the pilot publication receipt identify the
result. Historical integration-only restrictions for this exact sequence are
superseded. Full M4, founder acceptance, M5 and live-operation gates remain open.

**Historical D-033 — Founder-authorized main integration (2026-10-03), completed at `1054ff3`:** the founder
requests publication of all validated work to main. This covers the seven reviewed
Tasks 2.40–2.46 commits through `7624f93bf55721cb74687677a26c51fcf25cd669`,
plus this reviewed authorization/checkpoint record, by normal fast-forward from
`8912cfeffcd83fa12cc1a68937a9de8dd5c6b091`. Status: **TECHNICALLY VALIDATED /
FOUNDER-AUTHORIZED MAIN INTEGRATION** within the recorded task scopes. Git and the
[pilot checkpoint](PIV1_PILOT_STATE.md) record the resulting publication identity.
This supersedes the historical integration-only restrictions for this exact
sequence, without broader founder acceptance or live-operation permission.
M2/full M3–M6 remain open; next implement protected withdrawal initiation and
Stake deactivation. Existing tests/artifacts are verified retained evidence;
this documentation/publication operation reruns no tests/builds.

**Historical D-031 — Founder-authorized main integration (2026-09-28):** the founder
explicitly requests publication of all already-validated work missing from main.
This covers Tasks 2.33–2.39 and D-030 guidance through
`bf32d87e06a2d54c8e1c0192faaa7855d246dbc0`, plus the reviewed authorization record,
by normal fast-forward from `4cc4ea11e87c2f1a2f85b9ed48358f2a881821b3`.
Git and the [pilot checkpoint](PIV1_PILOT_STATE.md) record the resulting publication.
Status: **TECHNICALLY VALIDATED / FOUNDER-AUTHORIZED MAIN INTEGRATION**.
This is integration authority, not broader founder acceptance or live-operation
permission. At publication M2 had not started; see the current checkpoint for later
integration-only work.

This is an execution index under D-026/D-030, not a new specification or acceptance.
Canonical requirements remain in `PIV1_DECISIONS.md` and `PIV1_MASTER_SPEC.md`.
Current commit, actual executions and active task are in `PIV1_PILOT_STATE.md`.
Update this checklist when a bounded task closes; do not infer runtime evidence
from passing host tests.

## Current evidence register — M4 closure status (2026-10-18 UTC)

**Current closure / runtime AB:** The [finite M4 closure review](PIV1_M4_CLOSURE_STATUS.md)
is reconciled after Z/AA2. No missing mandatory supported handler is demonstrated.
The current exact PIV/caller now execute the unchanged full positive delayed
lifecycle: 19,497 records / 187 maps / 101 transaction pairs / 145 CPIs /
40 event payloads match reviewed O under explicit artifact/native-hash/CU deltas.
One locked/offline Bank build and actual AB were newly executed; prior 619+1/eight
host/SBF, caller 15/check/docs/SBF and 66 runner tests are retained, not rerun.
**M4 remains blocked on the stopped/unwaived internal-callee-failure criterion.**
The finite review is complete with that criterion unsatisfied; no additional
executable critical-path task is demonstrated here. Preserve/report the blocker,
do not invent more micro-tasks or resume its rejected operation. M5 is unauthorized.

**Retained Task 2.53 / runtime AA2:** [guardian synchronization](TASK_2_53_GUARDIAN_SET_SYNCHRONIZATION.md)
now passes actual three-cycle production execution with existing earned debt,
frozen old-round credit, retired claims and current-revision activity. Root passes
15 caller tests/check/docs/SBF, 66 runner tests and locked/offline Bank build/run.
Separate raw-trace review checks 5,628 records/70 maps, 33 transaction pairs,
104 CPIs and 24 event payloads. Final HWM 27,111,500,000; earned liability
76,000,000; claimed 38,000,000; pending/carry zero. AA's preexecution prefund correction
and prior production 619+1/eight gates/SBF remain separately scoped evidence.
Full M4 closure remains open; the blocked operation stays stopped and M5 unauthorized.

**Retained implementation Task 2.53:** [one-seat guardian synchronization](TASK_2_53_GUARDIAN_SET_SYNCHRONIZATION.md)
is implemented with dedicated current 4/6 approval, exact membership delta and
externally funded revision reward accounts. Root passed 13 focused tests,
619 host tests +1 doctest/eight gates and strict SBF/static inspection.
Host models preserve old earned owners, activity identity and frozen settlement;
actual new-artifact synchronization/old-round/future-activity/retired-claim runtime
remains next. Original Z evidence and pins are preserved, not reattributed. M4 is
incomplete, the internal-failure case stopped/unwaived and M5 unauthorized.

**Retained Task 2.52 / runtime Z:** supported authenticated recipient replacement
is now technically validated through actual production paths across two cycles.
Executed/three-vote approvals reject; a prepared round keeps old destinations,
while the later round pays the new pair. Complete maps/events/CPI and selected
late-outer rollback/replay/retries pass. See [Task 2.52](TASK_2_52_RECIPIENT_UPDATES.md).
Root passed 60 runner tests, 13 caller tests/check/docs/strict SBF and Bank build/run.
Independent raw-trace review covers 3,444 account records/44 maps, 22 transaction
pairs, 67 CPIs and nine events. Current PIV and rebuilt caller execute in Z;
605 production host tests +1 doctest/eight gates/strict SBF are retained unchanged.
Guardian replacement remains required open M2 work, preserving earned ownership.
The stopped internal-failure case and final independent closure remain open;
M4 is incomplete, M5 unauthorized. This adds no live-governance/transport proof.

Y is retained validated runtime evidence. It closes the bounded supported active
[economic-normalization gap](PIV1_M4_NORMALIZATION_CLOSURE_REVIEW.md), distinct from
pending-only reconciliation. Root and separate review check actual donor transfers,
normalization, no-op repeat and onward settlement/integration with exact complete maps.
The earlier X2 execution closes the bounded [positive withdrawal-fee gap](PIV1_M4_P023_CLOSURE_REVIEW.md).
Earlier profiles retain their distinct evidence scopes.

The October 18 source/evidence review identified A-004's later-compounding gap.
Runtime W now closes that bounded gap through actual production paths, as recorded
in [the A-004 review follow-up](PIV1_M4_A004_CLOSURE_REVIEW.md). W remains a retained
validated execution; the older profiles retain their distinct evidence scopes.

The documentation reconciliation at `afc388a` established the inventory below.
The October 16 continuation added actual V positive cooldown reward-to-later-split evidence
through unchanged production programs; O/N/P/Q/S/U retain their distinct scopes. Earlier milestone
narratives and the mixed-age baseline table remain historical. This is not exhaustive
specification-to-code review, a professional audit, M4 completion, founder acceptance
or a scope waiver. Full current execution details are in the pilot checkpoint.

Canonical requirements remain [decisions](PIV1_DECISIONS.md), especially
P-029–P-031/P-035 and D-030/D-034, and [master specification](PIV1_MASTER_SPEC.md)
sections 9.6–9.11. Runtime attempts, artifacts, review receipts, errors and exact
limitations are retained in [Task 2.51](TASK_2_51_REAL_PRODUCTION_LIFECYCLE.md).

| Requirement / boundary | Existing evidence and exact scope | Remaining distinction |
| --- | --- | --- |
| P-003/P-008/P-013–P-024, A-003: initialization, contributions, protected principal and custody | Task 2.51 O preserves the complete positive/pause cycle: actual initializer, SOL/Jito intake, bootstrap, protected staking, pending-after-snapshot intake, settlement and full-value pending/HWM integration, with complete account maps. | Synthetic external genesis and supported adapter profiles; not arbitrary custody topologies, real signatures or public-cluster proof. Old claims that all real transfers/initialization remain absent no longer describe these paths. |
| A-001/A-002/A-004/A-005/P-018/P-026/P-035: real adapter, delayed legs and rent | O executes pinned Token 8.0.0, SPL pool 2.0.3 and Stake 5.1.0 through production paths: two protected legs, real epoch-14 readiness, out-of-order finalization, exact original rent recovery and temporary-account closure. | General fee/rounding-loss support and unsupported Stake donation/rent-adjustment profiles remain limited. This does not prove every adapter branch or live Jito configuration. |
| A-004 boundary six / master 9.10 and 10.1: later principal SOL/Jito compounding | W executes three production cycles and then the protected deposit from actual completed Idle custody at 13:10. 2,458,999,998 lamports mint 1,891,538,460 units; HWM 27,159,000,000, prior summary, next sequence, pending/KIF/rents remain exact. Exact 6032/6028 refusals occur before CPI; complete fee-only rollback, identical no-fee replay, distinct success and actual event/protected CPI records pass. Root and separate review bind all S-prefix maps and independently derive the suffix. | Closes the identified bounded local runtime gap, not every adapter profile. Two economic lamports remain to preserve book value; no permanent reserve policy, general rounding-loss/nonzero-fee support, live signatures/transport or native Stake execution is claimed. Host evidence remains complementary. |
| P-023 with P-026 context: withdrawal fees and protected allocation | X2 executes a supported 1/1000-fee two-leg production cycle. Actual TransferChecked credits 4,587,922 + 3,303,329 fee units; only burns reduce supply. Delegated proceeds 8,600,027,724 plus pending use fund net 8,700,027,724; payouts 6,376,417,834 / 2,107,460,131 / 216,149,757 retain two dust lamports. Compound stays 2,109,545,454; final HWM 121,955,000,003, rents returned, pending zero. Complete maps, exact CPI/events and selected late-outer rollback/replay/retry pass. | Closes the supported local positive-fee-through-settlement gap. Explicit synthetic external fee/genesis and three-lamport reserve appreciation; not every fee regime, current live Jito configuration or fee-bearing principal deposits. Historical Phase-0 positive-fee CPI proof remains distinct. |
| D-025 / master §7.8 / Task 2.3: supported economic-vault surplus normalization | Y executes six real donor transfers during WithdrawalActive with nonzero P-U/assigned-token offsets. The thirteen-account production instruction moves only excess through three System transfers and one TransferChecked; pending credits rise 60 lamports/11 units with other accounting/rent unchanged. Repeat no-op, exact CPI/events, selected late-outer rollback/replay/retry and later integration pass full maps. Final HWM/book value 121,955,000,075 includes full contribution 845,454,617 once; payouts remain unchanged. | Closes the bounded supported active normalization gap. KIF liability/carry are zero at normalization; positive preexisting debt and other lifecycle combinations retain complementary host evidence. Operational provenance/Token-owned native excess excluded; explicit three-lamport reserve appreciation, synthetic genesis/governance and unsigned transport remain. No blanket coverage or full M4 claim. |
| P-013–P-019/P-035: loss and recovery | N proves one-lamport last-leg cooldown loss; P proves first-leg residual-HWM recovery with an unchanged outstanding companion; Q proves severe retained-below-pending settlement zero floor. Complete maps, recovery flags, rollback/replay/retry and subsequent 6073/no-CPI guards are retained; Q additionally verifies exact settlement recovery event bytes. | Balanced synthetic external losses, not live slashing. P is the finalizer residual guard, Q the settlement subtraction branch. Governed recovery exit and completion of the outstanding companion after early recovery remain unimplemented/unproved; no bypass or exit policy is inferred. |
| P-009–P-012/P-032 and K-005–K-010: cadence, active snapshot and carry | S executes three liquid-funded production cycles. Exact 6038 before ten days and success at the boundary distinguish preparation from delayed settlement; 6034 rejects an active round. Successive zero-active periods compound/carry 19M then 28.5M; three active guardians receive 22,166,666 each with carry two. Late heartbeats do not earn retroactively. | Clock-only synthetic timestamps, with full account maps and cached Clock agreement; not naturally elapsed time. These cases do not exercise the distinct valid-insufficiency branch below. |
| G-003–G-005/K-012: pause and already-earned KIF | O retains L's complete pause profile: 16 production pause/unpause changes, nine exact 6035 economic rejections, reconciliation of already-received direct transfers, heartbeat and earned KIF claim while paused. Claim follows earning in the same local lifecycle. | Synthetic governance and unsigned transaction transport. This advances past historical claim-only fixtures without proving real Squads proposal consumption, signatures or governance replacement paths. |
| K-002/G-007 and required M2 governance changes | Task 2.52 Z validates authenticated recipient replacement, frozen old destinations and later new destinations. Task 2.53 AA2 validates one-seat guardian synchronization with prior debt, old-round credit, retired claim and future activity. Both have actual native dispatch and separate source/host/SBF/runtime review. | Supported capability/runtime gaps closed. Preserve current-authority-only, synthetic-governance, unsigned transport, proposal non-consumption and historical artifact scope. No arbitrary final-program recipient or historical old-set approval proof inferred. |
| P-024/A-004: settlement/integration atomicity and layout | O and S include complete success maps and selected late-outer failures, retained transaction fee, identical no-fee replay and distinct successful retry. S covers prior-completion layout and three round sequences. | Late-outer failure is not a callee demonstrated to mutate and then fail internally. That concrete local coverage remains open and platform-blocked; interrupted drafts are not execution evidence. |
| P-029/P-030/P-031: valid insufficiency and its independent 24-hour cooldown | U completes a liquid cycle with existing preparation time 864,000, then records Config-only insufficiency at 1,728,000 through real production preparation/Stake minimum query. Immediate/+86,399 retries reject 6039; exact +86,400 succeeds. The ten-day timestamp remains unchanged; one-second-early cadence rejects 6038. Two malformed-version cases preserve both clocks; sufficient preparation succeeds one second after renewed insufficiency. Complete maps, exact events, query-only CPIs, fees and seven identical no-fee replays are independently checked. Task 2.46 host cases remain complementary evidence. | Bounded zero-fee active-source profile with synthetic Clock and declared ordinary donor reserve/pending inputs. It ends after sufficient preparation, without a second completed withdrawal cycle. It does not establish arbitrary minimum/fee/source profiles or real walltime progression. |
| P-035: positive withdrawn-stake cooldown reward, exclusion and next-cycle split | V executes a balanced 10,000-lamport synthetic reward observation after withdrawal/deactivation and before real cooldown. Actual Stake rejects early finalization and succeeds at epoch 14; both legs return original rents. First fixed payouts exclude the reward; pending integration preserves next-cycle yield and exact HWM. The later liquid cycle splits 5,900 HTFP / 1,950 compound / 1,950 team / 200 KIF; final HWM 121,955,001,953, next yield/pending zero, KIF liability 216,363,836. Complete maps, raw event payloads and selected rollback/replay/retry pass. Tasks 2.48–2.50 host/model tests remain complementary. | Abstract accrued-reward input increases both lamports and delegation, debiting an external donor; no native inflation/vote-credit execution or live-yield claim. Credits, rents, authorities and PIV state are unchanged at input. O's separate three-lamport reserve appreciation is retained explicitly. The later Clock is synthetic; no second-cycle leg or KIF claim occurs in V. |

**Retained Y evidence and gates.** First root executions passed 54 runner tests,
one locked/offline Bank build and actual Y. Root/reviewer frozen-parser inspections
agree: 8,352 records / 114 maps, 58 transaction pairs (53 distinct/five replays),
113 CPIs and 12 PIV events. All 35 successful maps and 18 fee-only failures pass;
five identical replays charge no new fee. Normalization uses 199,614 CU, no-op
168,366, initializer 1,067,516; maximum unsigned packet 1,544 bytes. Source freeze
binds 277 inputs / 11,424 prior records. Production/caller/five ELFs unchanged;
prior suites and executions retained, not rerun. O supplies unchanged initializer
layouts/leg metadata; remaining economic maps are independently derived.

**Retained X2 evidence and gates.** Root passed 48 runner tests after one test API
correction, a corrected locked/offline Bank build after a JSON macro compilation
failure, and the first actual X2 runtime. Independent parser correction replaces
its mistaken Transfer opcode assumption with pinned TransferChecked mint/decimals
semantics; all original failures and source copies are preserved. Root and separate
review check 7,890 records / 108 maps, 55 transaction pairs, 118 CPIs and ten actual
events: 32 successes, 18 fee-only failures, five no-fee replays. Final X2 freeze is
277 inputs / 11,055 prior records; later parser/review receipts are separate.
Production/caller/five ELFs and prior suites remain unchanged/retained, not rerun.

**Retained W evidence and gates.** Root passed 42 runner tests, one locked/offline
Bank build and actual W on their first executions. Root and separate review check
5,435 account records / 77 maps, 37 transaction/commit pairs and 77 CPIs. The new
suffix has 781 records / 11 maps, seven transaction/commit pairs (four distinct,
three replays) and six CPIs. It includes one successful full map, three fee-only
failures, three no-new-fee replays and two actual event payloads. The S prefix is
compared exactly with retained audited evidence; it is freshly executed inside W,
not independently rederived in full. W freezes 277 inputs / 10,387 prior records.
Production, its existing host-unit tests, caller sources and five ELFs are unchanged;
prior host/SBF suites
are retained, not rerun. This is not final exhaustive M4 closure.

**Retained runtime V evidence and gates.** Root previously executed 36 runner tests, one locked/offline
Bank build and actual V, all passing on their first executions. Separate review
audits 8,750 account records / 120 snapshots, 59 transaction/commit pairs (54 distinct
plus five replays), 110 CPIs and nine actual PIV event payloads. All 36 successful
maps and 18 fee-only failures pass; five identical replays charge no new fee.
The V freeze binds 277 inputs and 10,027 prior records. Preexecution draft oracle
corrections are documented, not failed executions or production defects. Prior
595 host tests +1 doctest/eight gates/strict SBF and caller 11 tests/check/docs/SBF
remain hash-bound to unchanged sources/five ELFs; they were not rerun. O/N/P/Q/S/U
remain retained runtime evidence. No full final milestone-gate claim is made.

**Remaining M4 closure.** The finite requirement/source/evidence review is complete
with an unsatisfied criterion: internally mutating callee failure remains stopped/
unwaived under the platform restriction. AB closes the current-artifact positive
composition gap; Z/AA2 close supported governance capability/runtime gaps. This is
not exhaustive compliance, a waiver or full M4 completion. Preserve/report the
blocker without automatically retrying the rejected operation. Existing governance,
recovery and adapter limits remain scoped; they do not automatically justify another
harness task. Accepted economics and governance remain unchanged.

**Later milestones and operations.** Exact deployment packaging, addresses,
authorities, funding and founder workflow belong to M5, which has not started.
Unsigned transport (maximum observed packet 1,544 bytes), absent real signatures,
synthetic governance/proposal non-consumption and initializer resource limits
remain explicit readiness constraints; none is declared acceptable for live use.
D-034 requires a reviewed M4 completion report and STOP before M5 until explicit
founder authorization. Deployment/signing/funds/authority transfer and founder
acceptance remain separate gates. This register prepares no M5 package.

## Active milestone evidence plan — D-030

The founder's six ordered production milestones in the
[execution plan](PIV1_CODEX_EXECUTION_PLAN.md) govern validation.
**M1 is complete within Task 2.39's documented initializer boundary/runtime scope;
M2 is IN PROGRESS (Task 2.50); the full M3 adapter and M4–M6 remain OPEN.** For M1, root passed 32 focused tests,
479 host tests +1 doctest/eight gates, nine runner tests and twelve actual production
Bank messages across four profiles. Root and separate review each checked 786
complete account records. Approval/reinitialization/error negatives have host
evidence; actual Bank evidence covers success, late failure, replay and retry.
Earlier claim/pending SBF and Node transport results are historical, not rerun.
See [Task 2.39](TASK_2_39_PRODUCTION_INITIALIZER.md) for artifacts and exact limits.

Task 2.40 closes the first bounded M2 source/host/static-build block: 10 focused
regressions, 489 host tests +1 doctest/eight gates and a clean strict SBF build,
with separate source/test/artifact/evidence review. New-path VM/Bank execution is
unproved; the tests model runtime context and signer privilege.
Task 2.41 adds nine grouped intake regressions, 498 host tests +1 doctest/eight
gates and clean strict SBF compilation on the corrected 98-input freeze. Tests
cover D-032 pause rejection before effects, exact native/Token transfer bytes and
postconditions, both prior deficits, surplus, self-delegate allowance/padding,
alias/borrow/arithmetic/CPI failures and full active-round preservation. The Token
callback runs the pinned host Processor; System transfer and discard/retry are
host models, not SVM/Bank rollback evidence. The initial import compile failure
and capacity-inventory failure are retained; reviewed corrections preserve guards.
See [Task 2.41](TASK_2_41_CONTRIBUTION_RUNTIME_INTAKE.md) for exact pins and recovery.
Task 2.42 passed ten grouped normalization tests, 508 host tests +1 doctest/eight
gates and strict SBF with separate review. Its host oracles cover: full six-dimension
deficits/surplus, signed fixed CPI bytes/seeds, every-CPI postconditions, no-op
idempotence, pause/recovery differences and partial multi-CPI failure/retry models.
Token-native excess and operational funding must remain unchanged; old strict
accessor tests must continue to reject unsupported Token-native surplus.
Task 2.43 passed ten focused tests, 518 host tests +1 doctest/eight gates and
strict SBF with separate review. It covers initial bootstrap composition with current
pool/Mint/Clock facts, conservative stored-supply valuation (including direct-burn
lag), exact HWM contribution accounting, native-funding quarantine and full partial
failure oracles. Legacy pure API validation and replay/history gates must remain.
Task 2.44 below covers protected principal SOL deposit with the pinned CPI and
exact pool/custody postconditions, retaining Task 2.6's zero-fee/book-value/HWM guards.
Post-settlement integration remains future work; M4 must still execute the complete
real production lifecycle. Current new-handler host/static evidence is not VM proof.
Task 2.44 source/test review, ten focused tests and 528 host tests +1 doctest/eight
gates passed. Its protected principal
deposit tests cover exact pinned CPI bytes/seeds, independent pool/Mint/custody
oracles, stored-supply burn lag, fee/rounding/HWM gates, carry/native-funding
preservation, legacy API error order and partial-effect discard/retry. The pinned
Token processor runs on host; nested stake-pool effects remain modeled. Strict
SBF passed; reviewed publication is integration-only. Next validate the
production distribution-preparation boundary and its required protocol facts;
complete local production-path execution remains the M4 gate.

Historical Task 2.45 evidence (its liquid profile remains unchanged): production no-yield/liquid-funded
preparation uses
current pool valuation and Clock/guardian snapshots. Positive withdrawal shortfall
must reject before effects/cooldown: actual dynamic Stake minimum, source residual
and conservative multileg proofs are missing M3 prerequisites. Pending SOL funds
first, then recorded carry; preserve HWM proposals, full contribution ledgers,
liabilities and native funding. See the [Task 2.45 report](TASK_2_45_LIQUID_DISTRIBUTION_PREPARATION.md)
for scope and evidence: ten focused tests, 538 host tests +1 doctest/eight gates
and strict SBF passed with separate review. No new-path VM/Bank proof. Full
preparation, settlement and local lifecycle remain open.

Task 2.46 is technically validated on integration: bounded active-source withdrawal
preparation now authenticates the runtime Stake minimum and current rent/source,
then derives canonical target/multileg/HWM proofs. Valid insufficiency changes only
its timestamp/event. Unsupported source/preferred-fallback paths reject. Root passed
10 focused tests, 551 host tests +1 doctest/eight gates and strict SBF, with separate review.
Host query/System/discard models do not establish real Stake/pool/Bank execution.
Next implement protected leg initiation/deactivation, finalization and settlement;
future execution must reauthenticate all source/minimum/rent facts. See
[Task 2.46](TASK_2_46_WITHDRAWAL_PREPARATION.md).

Task 2.47 is technically validated, pending founder acceptance: protected active-
source initiation and immediate Stake deactivation now bind canonical temporary
PDAs, prefund normalization, current maximum fill/minimum/rent/fees, exact CPI
receipts and post-removal round-floor/HWM proofs. Corrected source passed 10
focused tests, 561 host tests +1 doctest/eight gates and strict SBF with separate
review. Initial SBF rejected a 4160-byte frame; a reviewed bounded allocation
correction passed the same strict gate. Both attempts remain preserved. Actual
nested VM/Bank execution and heap/CU are still M4 obligations. Next close real
Stake finalization/rent recovery, settlement and pending integration.
See [Task 2.47](TASK_2_47_PROTECTED_WITHDRAWAL_INITIATION.md).

Task 2.48 is technically validated, pending founder acceptance: production full
Stake withdrawal uses CPI-enforced inactivity, exact original rent recovery,
current residual HWM and atomic metadata closure/cumulative accounting. Ambiguous
Stake native excess rejects; provable metadata excess goes to pending. Root passed
ten focused tests, 571 host tests +1 doctest/eight gates and strict SBF, with separate
review. An initial overflow test expectation was corrected without production
changes; all failed evidence is retained. Actual Stake runtime readiness/rollback,
heap/CU and account purge remain unproved. Next: settlement and pending integration.
See [Task 2.48](TASK_2_48_WITHDRAWAL_FINALIZATION.md).

Task 2.49 is technically validated, pending founder acceptance: production atomic
settlement binds frozen recipients/reward tuples, exact System receipts, snapshot
KIF carry/compounding and eight-account state commit. Valid insufficient protection
commits only recovery with no payout, preserving T23-R1 and accepted error ordering.
Root passed 10 focused tests, 581 host tests +1 doctest/eight gates and strict SBF;
separate source/test/command/evidence review passed. Actual System runtime rollback,
heap/CU and full local lifecycle remain open. Next implement pending integration.
See [Task 2.49](TASK_2_49_ATOMIC_DISTRIBUTION_SETTLEMENT.md).

Task 2.50 is technically validated, pending founder acceptance: post-settlement
integration uses current authenticated pool value, exact P-U/Q/escrow transfers,
full contribution HWM and final Config/Idle-round commit. Cooldown/KIF/rent and
Token-native quarantine remain protected; legacy derivation order is unchanged.
Root passed 10 focused tests, 591 host tests +1 doctest/eight gates and strict SBF; separate review passed.
Real runtime rollback, heap/CU and complete local lifecycle are still unproved.
Next close that production lifecycle and demonstrated adapter dependencies before
the exact Testnet package. See [Task 2.50](TASK_2_50_POST_SETTLEMENT_PENDING_INTEGRATION.md).

Task 2.51 remains **IN PROGRESS / REPEATED-CYCLE CADENCE AND KIF CARRY VALIDATED**.
Clean takeover as jerem was integration `c8a8a54416ff2d42e101fd1bbfe8b8e1618c197c`.
Main remains completed D-035 `2f8d083`; protected Task 2.3 remains `3677fee`.
This D-026 continuation publishes only reviewed integration. Git and
`/tmp/piv1-t251-cycle-review-20261015-a/publication.json` record its identity.
Technical validation is not founder acceptance.

One bounded writer and separate review added an isolated compact three-round
production profile. Root passed 24 runner tests, two corrected locked/offline Bank
builds and actual runtime S. The first build’s JSON macro error and R’s final cache-observation
failure remain preserved. R completed all economic maps before that assertion; S
uses a separately reviewed read-only current-slot cache check and passes throughout.
All production/caller sources, five fixture ELFs, dependencies,
layouts, allocator and accepted economics remain unchanged. The entire prior Rust
lifecycle source remains an exact byte prefix. O/P/Q and earlier host/SBF results
are hash-verified retained evidence, not rerun. S executes PIV, System, real Token
8.0.0, SPL pool 2.0.3 and the synthetic initializer caller; Stake is loaded but not
executed. Prior delayed-leg/pause/loss evidence remains separately applicable.

Actual initialization, SOL/Jito intake and bootstrap precede three liquid-funded
preparation/settlement/pending-integration cycles. Declared external reserve
transfers and real pool updates produce yield. Canonical synthetic Clock updates
change only timestamp bytes, with full maps and syscall-cache agreement; slot and
epoch stay fixed. No natural time progression or live-yield proof is claimed.
A preparation rejects exact 6038 one second before ten days and succeeds at the
boundary, only nine days after delayed settlement. An active round rejects 6034.
Both guards preserve all accounts except fees and identical replay charges nothing.

Successive zero-active periods reapply 50/50 to current KIF plus prior carry:
19,000,000 then 28,500,000 lamports compound and carry respectively. Period two
snapshots three active guardians, credits 22,166,666 each and retains two collective
carry lamports. Later heartbeats never earn retroactively. Full pending contribution
value raises HWM despite partially funding outgoing custody. Final HWM is
27,159,000,000, pending zero, earned KIF liability 66,499,998 and zero-active compound
47,500,000. Complete account maps cover prior-completion layouts, sequence, rent,
all recipients/rewards and selected late-outer rollback/replay/distinct retries.
Root and separate review checked 4,654 account records / 66 snapshots, 30 transaction/
commit pairs and 71 CPIs; all R/S economic records match exactly. Initializer and
maximum unsigned packet remain subject to the limits documented in the task report.

M4 is NOT complete. Next close a callee proved to mutate before failing internally,
then independent requirement-to-evidence closure and any demonstrated blocker.
Governed recovery exit remains unimplemented/unproved. Synthetic genesis/governance,
proposal non-consumption, synthetic Clock, unsigned signatures/transport and supported
adapter-profile limits remain explicit. D-034 requires the reviewed completion report
and STOP before M5 after full M4 validation. M5 has not started and is unauthorized.

Preserve A–S attempts, source copies, ELFs, receipts and four recovery archives.
Final preexecution freeze: 277 inputs / 9,019 prior records; later execution/review/
publication records are separate. Evidence root: `/tmp/piv1-t251-cycle-review-20261015-a`.
No cleanup/install, secret access, signing, live RPC/deployment, real funds or
authority transfer occurred. Preserve the 2-GiB disk reserve before further large
validation. Post-S available disk was 2,398,650,368 bytes, only about 251 MB above
the reserve; recheck actual capacity before any further build/run and do not delete
retained evidence. Save/end this bounded session economically; verify actual refs/
worktree and the Task 2.51 report on resume.

See [Task 2.51](TASK_2_51_REAL_PRODUCTION_LIFECYCLE.md).

**D-034 — Mandatory stop after M4 (2026-10-12 UTC).** Once M4 is fully
completed and technically validated, save the reviewed completion checkpoint,
stop before starting M5, and notify the founder with a concise completion report
covering scope, validation evidence and remaining limitations. Do not start M5
until the founder explicitly authorizes it. Routine continuation or technical
access does not satisfy this authorization. This gate does not declare M4 complete
or change accepted economics, founder acceptance, main publication or live-operation
limits. M4 currently remains incomplete; see the active pilot checkpoint.

| Milestone | Critical gap | Required completion evidence |
| --- | --- | --- |
| M1 — Production initializer | COMPLETE: strict native dispatch and actual production initialization/Bank evidence passed; sensitive/live and full-lifecycle limits remain. | Separate ABI/account review and actual production ELF execution; exact approved bytes, guardian/governance/protocol/recipient authentication, fresh/prefunded rent and initial pause oracles, malformed/replay rejection and existing claim/pending dispatch regressions. |
| M2 — Economic runtime handlers | Supported required handlers implemented and locally exercised through Tasks 2.40–2.53, O–Y/Z/AA2 and current-artifact AB. No additional missing mandatory handler demonstrated by the finite closure review. | Authenticated SOL/JitoSOL intake and reconciliation, pending/bootstrap integration, distribution cadence/minimum/cooldown, deterministic delayed withdrawal legs, exact-target/all-leg settlement, rent/recovery/HWM, beneficiary/KIF accounting, heartbeat/earned claims and confirmed governance/pause paths; focused transition, accounting, replay and failure tests. |
| M3 — Real pinned SPL/Jito adapter | Current-artifact AB validates the supported protected deposit/two-leg delayed lifecycle. Historical N/P/Q/X2 and other profiles retain specific loss, fee, rent and guard evidence on unchanged adapter source. | Reviewed pinned implementation of the existing seven-method contract; protected deposit/withdrawal variants, immutable 1-bps cap, conservative outputs, exact account/authority/fee mapping, postconditions and error propagation through production paths. |
| M4 — Complete local lifecycle | INCOMPLETE / BLOCKED: finite closure review reconciled; AB closes current exact-artifact positive lifecycle, Z/AA2 close supported replacement governance. Internal-callee failure remains stopped/unwaived and prevents completion. No further executable critical-path gate demonstrated by this review. | Exact production artifact/source pins, end-to-end contribution-to-settlement/pending/KIF cycles, deterministic multi-leg readiness, rent recovery, loss/recovery and pause/failure/retry evidence; final milestone gates and independent requirement-to-evidence review. Model/stub-only success cannot close this milestone. |
| M5 — Exact Testnet package | PREPARATION AUTHORIZED under D-039 with M4 reservation. Initial official cluster/Jito account observations are recorded; deployed compatibility, complete native transaction/governance composition, public identities, budget and workflow remain open. | Follow the single [readiness register](PIV1_M5_TESTNET_READINESS.md). Independently reviewed exact approval card, including the unresolved M4 reservation, cluster/genesis, artifact/Program ID, authorities/recipients, funding/fees/rent, bounded commands, recovery/stop conditions and reproducible founder workflow. Public package remains NOT READY. |
| M6 — Explicit deployment authorization | Technical evidence does not authorize the first live Testnet deployment. | Present the exact concise card, record explicit founder authorization, and stop before live deployment until it exists. Material package changes require updated authorization; acceptance and sensitive-action limits remain separate. |

During implementation, run proportionate targeted tests for changed behavior and
demonstrated defects, then the final gates required by that milestone. Use one
writer and separate review, correct verified defects, and checkpoint reviewed
integration work. Reuse unchanged evidence only within its actual source/artifact
scope; avoid wholesale reruns or new validation-only tasks unless a demonstrated
critical-path blocker names the missing evidence and its closure criterion.
Neither test economy nor D-030 waives existing security/diagnostic gates.

M1 must authenticate the actual entire Squads-approved instruction bytes: the
`PIV1GM01` model codec does not justify stripping/reconstructing a new envelope
before approval comparison. Recipient vault indices are derivation witnesses;
probe-only 0/255 values are not a production policy. Under D-032, test that both
explicit contribution handlers reject while paused before transfer/state effects,
while direct transfers already received remain reconcilable.

The prior automatic next Bank-failure task and fixed per-session stop are
superseded. Preserve their historical evidence/limitations, conserve credits and
checkpoint interruptions. Main publication, founder acceptance and live operations
retain their separate gates.

## Previous checkpoint — Task 2.38

**TECHNICALLY VALIDATED / PENDING FOUNDER ACCEPTANCE**. Root passed fifteen final
runner regressions and the first actual Bank runtime: one test, four profiles,
twelve messages, sixteen complete ancestry-visible snapshots and 786 full account
records. Its independent decoder checks literal PDA/state bytes, loader/ELF bytes,
raw messages, every ordered CPI tuple and full account metadata. Separate source,
command, binary and independent evidence reviews passed.

The exact three Task 2.31 ELFs load through real V3 ProgramData in slot-one Bank.
Completed initialization followed by malformed caller fails at index 2; every
non-fee account restores exactly. Fees are [15000, 0, 15000] for late failure,
identical AlreadyProcessed replay and distinct successful retry. Separate original
rent is 34,779,120 fresh or 890,885 prefunded lamports; native excess normalization
is exactly 144 for prefunded/paused profiles. Full recipient/sentinel/ELF/sysvar
bytes remain unchanged. Retry CU are [1065383, 1058793, 986363, 979773], under explicit
1.4m/default 32-KiB bounds, above 200k. No ordinary-budget claim follows.

The first strict build passes with zero diagnostics. Initial runner fixtures had
one path-alphabet error; its narrowly corrected second execution passes all 15.
A preventive source fix moves independent rereads outside the scan callback lock.
Original runner/resource/socket/diagnostic guards remain intact. Old suites are
retained evidence, not reruns. All 223 inputs, tool/cache and artifact pins match.
Actual runtime covers local unsigned entry only: serialized messages are 1503–1586
bytes, exceeding public packet size. No signature/ALT, real governance or full
recipient control, failed in-initializer CPI rollback, durability or chain proof.
See [Task 2.38](TASK_2_38_BANK_GENESIS_INITIALIZATION.md). Its former next-probe
and session-stop recommendation is historical; D-030 above controls next work.

## Previous checkpoint — Task 2.37

**TECHNICALLY VALIDATED / PENDING FOUNDER ACCEPTANCE** for native System Bank
commit/account rereads. Root executed 12 final runner regressions and one Bank
test/four messages. Root and separate reviewer independently decoded raw messages
and checked 100 complete account records: 20 accounts across five snapshots.
The oracles verify persisted success, second-instruction failure after the first
transfer's observed transient debit, exact non-fee rollback and fee debit,
unchanged-state/no-fee replay rejection, and distinct successful retry on the
same Bank. Account key sets, data, owners, lamports, executable flags and rent
epochs all match. Fees are [10000, 10000, 0, 10000]; observed CU are
[150, 300, not executed, 300]. First actual runtime execution passed.

The fourth full build passes the strict diagnostics gate. Three earlier builds
and an optional targeted Cargo-check resolver failure remain failed and retained.
The reviewed fixes preserve all test oracles and warning/resource/network gates.
The runner's actual subprocess regressions cover IPC versus denied socket domains,
resource/timeout termination and output/source preservation. Source/lock/vendor
provenance and exact binary were separately reviewed; older suites were not rerun.
The short 0.04-second runtime is not a reliable peak-memory measurement.

Scope excludes PIV1/SBF initialization, signature verification, disk restart,
Bank freeze/root, ledger replay and public-chain behavior. Full initializer Bank
account/retry evidence remains next, with exact loader/features and artifact pins.
See [Task 2.37](TASK_2_37_BANK_COMMIT_ROLLBACK_SMOKE.md). Save/STOP;
Task 2.38 is NOT STARTED. Main and live-action gates remain unchanged.

## Previous checkpoint — Task 2.36

Isolated Bank dependency preparation is **TECHNICALLY VALIDATED / PENDING FOUNDER
ACCEPTANCE**. Root passed public resolution and three locked/offline metadata
observations; root and separate review checked all 589 registry archives and
25,891 extracted source files. The host graph has 552 registry packages/56 build
scripts, against 70 scripts in the full graph. No compilation, test, runtime or build
script execution occurred in these metadata checks; prior suites were not rerun.
A retained read-only audit assumption about an optional Cargo marker was corrected
without changing dependencies or byte-comparison requirements.

The source-reviewed unsigned Bank path requires actual commit and account rereads
for future runtime evidence, with fee-payer debit and replay treated separately.
Native scripts/libraries, exact ELF/genesis feature loading and measured resource
budgets remain open. Existing production/economics/locks and 162 protected inputs
remain intact. See [Task 2.36](TASK_2_36_BANK_DEPENDENCY_PREPARATION.md) for commands,
checksums and source limits. Save/STOP; Task 2.37 is NOT STARTED.

## Previous checkpoint — Task 2.35

Task 2.35 is **TECHNICALLY VALIDATED / PENDING FOUNDER ACCEPTANCE** under D-026 for
reversible recovery of local build capacity. Root passed **22 focused Python
regressions**, first attempt; separate source, manifest, archive and action review
passed. Exactly **8368** non-executable Cargo intermediates in twelve known host
builds were archived as **3902** verified gzip objects before originals were
removed. One **22389024-byte** library was restored at its original path and left
in place; **8367** paths remain archived. Root independently rechecked **26901
preserved files**, with no unexpected build-tree change. Recorded executables,
SBF artifacts, logs/receipts, source/cache and production/economics are preserved.

Available space after recovery is **8839438336 bytes (8.23 GiB)**; the durable
archive occupies **852205568 allocated bytes**. Selected-file accounting recovers
**5563084800 bytes net** after the archive and demonstration restore; filesystem
availability is a separate observation that can include unrelated activity.
The archive is `/home/jerem/piv1-evidence/task-2.35-intermediates-20260928-a`.
Its manifest, objects and journal must be retained for restoration. This is local
recovery storage on the same disk, not a disaster backup. Historical target trees
are no longer complete incremental caches; no earlier compiler/runtime test was
rerun. Actual Bank dependencies and commit/rollback evidence remain unprepared.

Regressions cover exact recovery, deduplication, tampered inputs/archives, locks,
space loss, symlink/hardlink/xattr boundaries, interruption recovery and refusal
to overwrite. Root independently matched all manifest records, decompressed all
3902 objects before prune and rehashed all 26901 preserved files afterward.
Earlier Rust/SBF/Node results are retained evidence, not new executions. See
[Task 2.35](TASK_2_35_BUILD_CAPACITY_RECOVERY.md) for exact commands and review.
Save/STOP; Task 2.36 is NOT STARTED.

## Previous checkpoint — Task 2.34

**TECHNICALLY VALIDATED / PENDING FOUNDER ACCEPTANCE**, limited to read-only prerequisite
tooling. Root passed 19 Python regressions first attempt; separate review passed.
Coverage includes disk boundaries, available-versus-free blocks, missing/wrong-type/
symlinked paths, inaccessible metadata, malformed/absent budgets, distinct CLI
usage errors and JSON results. Synthetic files prove no genuine Bank closure.
The actual check correctly returns NOT_READY: 3301838848 bytes available, an
explicit unmeasured 8-GiB planning reserve, six missing direct candidate paths
and an existing modular runtime source. It executes no commands on any outcome.
No Rust/SBF/runtime suite was rerun. All Task 2.33 source/artifact hashes and
100 logs were reverified, including failures; older 448-log verification remains
historical. See [Task 2.34](TASK_2_34_BANK_RUNTIME_PREREQUISITES.md) for commands,
hashes, final review and limits. Save/STOP; Task 2.35 is NOT STARTED.

## Previous checkpoint — Task 2.33

Task 2.33 is **TECHNICALLY VALIDATED / PENDING FOUNDER ACCEPTANCE** for
initializer-internal resource failures and retries. Root passed **13 runner tests
and eight runtime tests/sixteen message cases**; root and separate review each
checked **2400 complete account records**. Both 35/34 profiles cover fixed
200k pre-effect exhaustion and actual first/second Token-CPI exhaustion, including
paused mixed prefunding. Exact partial raw state, original rent debit, 144-lamport
normalization, zero unwritten state buffers and protected accounts are checked.
Each actual returned-original vector then initializes successfully in a fresh
runtime at the existing 1.4m-CU ceiling/default 32-KiB heap. This is Mollusk output
discard/retry evidence, not Bank rollback or recovery from committed partial state.

A first runner execution failed two temporary-path fixtures; a first runtime
failed the two exact preflight-error expectations. Minimal reviewed test-only
corrections preserve strict guards, fixed budgets and all state/custody/trace
oracles. Final build/run and separate review passed; both failures remain recorded.
Production/probes/old harnesses/economics and dependencies are unchanged; all 119
tools/nine aliases match. No installation or old-pin refresh occurred. Earlier
448 logs are verified retained evidence, not reruns. See
[Task 2.33](TASK_2_33_GENESIS_INITIALIZATION_FAILURE_RUNTIME.md) for commands/hashes.
D-026 covers reviewed integration-only publication; main remains at `4cc4ea1`.
Broader founder acceptance, actual governance/control, funding provenance, native
initializer exposure and Testnet readiness remain open. **Save/STOP; Task 2.34 is
NOT STARTED.**

## Previous checkpoint — Task 2.32

TECHNICALLY VALIDATED / FOUNDER-AUTHORIZED MAIN INTEGRATION under D-029.
Root passed 13 mocked runner regressions and 13 local runtime cases against the
three unchanged Task 2.31 artifacts. Separate source/runner/command/exact-binary/
evidence review passed. Root and the reviewer independently decoded 2040 complete
account records, checking nine literal state envelopes, two exact Token layouts,
original payer rent debit, 144-lamport native prefund sweep, zero initial ledgers,
protected accounts, full raw/returned state and runtime Instructions/Clock/Rent.

Four successes cover both fixed 35/34-account profiles with fresh or mixed
prefunding (the latter initially paused). Compute is 1063693/1057103/984673/978083 CU
under an explicit 1.4m ceiling/default 32-KiB heap; all exceed 200k. Eight early
negative cases and one late second-instruction failure match expected errors and
exact call traces. Late failure demonstrates completed raw initialization followed
by Mollusk's returned-original output discard, not Bank/AccountsDB rollback or
failed-CPI atomicity within the initializer. Synthetic caller/restricted Token
execution does not establish actual Squads/deployed Token/ALT lifecycle behavior.

The first build failed on obsolete test Rent fields; a minimal default-Rent
comparison retained all sixteen literal rent-floor assertions. The first runtime
passed 12 cases and failed one trace-order assertion after correct account/result
checks. Pinned Agave reserves top-level trace slots before appended CPIs; the
reviewed correction checks that exact topology. Final build-c and run-b passed
with 24 verified logs each/no diagnostics. Both failed attempts are preserved;
no weakened oracle, production edit or warning suppression was used.

Seven new inputs, 143 protected old inputs, 119 host tools, nine aliases, 389
existing registry identities and nine artifacts are bound. Only the new profile
adopts independently verified installed libexpat bytes; no installation/old-pin
change occurred. Root verified 326 earlier logs; the earlier 469 host tests +1
doctest/eight gates, claim/pending runtime, Node transport and Tasks 2.29–2.31
suites remain retained evidence, not rerun. See the
[report](TASK_2_32_GENESIS_INITIALIZATION_RUNTIME.md) for commands and hashes.
D-029 authorizes the reviewed Tasks 2.24–2.32 for normal main integration after
final checks; this is not broader acceptance or Testnet readiness. Initializer
failure/resource boundaries, native exposure, full recipient control, funding
provenance and actual governance/transport remain deferred. Save/STOP;
Task 2.33 is NOT STARTED. Live-operation gates are unchanged.

## Previous checkpoint — Task 2.31

TECHNICALLY VALIDATED / PENDING FOUNDER ACCEPTANCE within build-only preparation.
Root executed 15 Rust boundary tests (nine caller/four callee/two Token) and 11
mocked runner regressions, all passing first attempt. Three doctest suites contain
zero examples; warning-denied docs pass. Four gates took 82.682023 seconds without
diagnostics. Separate source/runner/command and host-evidence review passed.

The first strict SBF build passed in 362.604063 seconds, with 34 commands and 68
verified logs. Three new ELF artifacts expose only the fixed synthetic caller,
full recipient-checked normalized initializer and restricted InitializeAccount3
Token wrapper. Root checked all sizes/hashes and 14 new inputs, 128 protected
inputs, 155 existing registry identities and six historical artifacts. Separate
actual-artifact static review passed. No loader or new SBF runtime was invoked.

The tests cover literal topology/metas/data/seeds and payer privileges, malformed
wire/roles, exact CPI error propagation, host refusal, restricted Token input,
immutable profile derivation and required third-artifact/strict-diagnostic guards.
They do not prove successful initializer or Token execution. Total heap/compute,
mutating atomicity/rollback, actual Squads/ALT governance and full recipient
control remain deferred. No production source, ABI, economics or old pins changed.
The new target profile derives only the already-verified libexpat file hash in
memory without modifying the old helper/profile or installing anything.

Root verified 250 earlier logs: prior 469 host tests +1 doctest/eight gates,
24 claim/pending SBF tests/70 cases, 15 Node tests/eight old/sixteen recipient cases,
Task 2.29's 10 boundary +9 runner tests and Task 2.30's 12 runtime +11 runner tests
remain retained evidence, not rerun. See the
[report](TASK_2_31_GENESIS_INITIALIZATION_PROBES.md) for exact commands, hashes and
limits. Save/STOP after integration publication; Task 2.32 is NOT STARTED.
Main acceptance and live-operation gates remain unchanged.

## Previous checkpoint — Task 2.30

TECHNICALLY VALIDATED / PENDING FOUNDER ACCEPTANCE for keyless local read-only
preflight execution. Root passed eleven mocked runner regressions and twelve
runtime tests/cases against the exact unchanged Task 2.29 probe ELFs. Both 32/31
profiles succeed through actual height-two SBF CPI. Eight invalid callee cases,
one direct-height rejection and one outer-caller rejection have precise expected
outcomes and traces. Complete supplied/raw-before/raw-after/returned account
preservation is independently checked across 1674 account records, including the
actual runtime-generated Instructions bytes and its metadata. Clock/Rent are
runtime sysvars; no host-context preflight result supplies the success oracle.

The first host build failed before tests with two E0433 module errors from reused
test support. A minimal new test-root re-export fixed them; the fresh second build
passed without diagnostics, followed by a successful first runtime execution.
Separate source/runner/build/exact-binary/command/evidence review passed. Success
uses 288277/282070 CU with default 32-KiB heap and explicit 1.4m ceiling, exceeding 200k.
No ordinary-budget claim, complete initializer-resource/rollback proof or actual
Squads governance/recipient-control evidence follows.

Root verified 128 older logs, 64 retained probe-build logs and 8 retained probe-host
logs; production 469 tests +1 doctest/eight gates, 24 claim/pending SBF tests/70 cases,
15 Node tests/eight old plus 16 recipient cases, and 10 probe host +9 old runner
tests remain retained evidence, not rerun. One installed libexpat hash is verified
through signed Ubuntu metadata/package bytes and bound only in the new profile;
no installation or historical-pin update occurred. See the
[report](TASK_2_30_GENESIS_PREFLIGHT_RUNTIME.md). Save/STOP after reviewed integration
publication; Task 2.31 is NOT STARTED. Main and live-operation gates remain.

## Previous checkpoint — Task 2.29

TECHNICALLY VALIDATED / PENDING FOUNDER ACCEPTANCE within build-only preparation.
Two isolated probes bind the unchanged read-only 32/31-account recipient preflight
and a synthetic caller. Meaningful tests cover exact roles/metas/data/seeds,
truncations and raw count overflows, privilege changes, canonical identity and
host rejection. Nine mocked runner tests cover source/helper/configuration/lock
refusals, zero-exit diagnostics, malformed ELF/archive and independent output audit.

The first delegated host compilation failed before tests; reviewed test-only
fixture/dependency corrections then passed. Writer and root each passed 10 Rust
boundary tests +9 runner tests. Root's four gates also include doctest discovery
(zero examples) and warning-denied docs. Final strict workspace SBF compilation
passed; separate static inspection bound both artifacts to actual output bytes.
No probe runtime, real Squads/control, total genesis resources or rollback proof.

Root verified 92 production/105 existing harness inputs, eight transport inputs,
128 retained logs and tools/artifact hashes. Earlier 469 host tests +1 doctest/
eight gates, 24 SBF tests/70 cases and 15 Node tests/eight old plus sixteen recipient
cases remain retained evidence, not rerun. See the [report](TASK_2_29_GENESIS_PREFLIGHT_PROBES.md)
for commands, initial failure, hashes and boundaries. Save/STOP after integration
publication; Task 2.30 is NOT STARTED.

## Previous checkpoint — Task 2.28

TECHNICALLY VALIDATED / PENDING FOUNDER ACCEPTANCE under D-026.
The strict current-source target build exposed seven oversized genesis frames (243 diagnostics despite Cargo exit zero). Private
preflight indirection resolved the caller frames; the second build still rejected
three shared-dispatch frames (45 diagnostics). Two separate producer/boxing
boundaries then passed the unchanged strict compiler gate with no diagnostics.
Both rejected attempts remain recorded; no safety gate was weakened.

The final source independently passed 53 writer-focused tests and root 469 host
tests +1 doctest/eight gates, with separate source/test review. Two additional
private heap requests measure 2272 bytes under the host layout; this is no total
genesis heap/runtime proof. A compile-time footprint guard and host nested-result
bounds complement, but do not replace, actual target diagnostics.

The reviewed current artifact is 229888 bytes, SHA-256
`0eb5e62389c9baa5311fddca99d1e705f86b1fd698e869a8cdcec778aa68cd54`.
Fresh current-artifact validation passed 24 local SBF tests/70 cases after
separate artifact/executable review. Root independently checked 1629 complete
account records, including raw effects versus Mollusk returned-output discard.
No signed cluster or Bank/AccountsDB rollback proof follows. Root also passed
16 target-runner and seven harness-runner refusal tests in this task.
The 15 Node tests/eight old plus sixteen recipient cases are retained Task 2.27 evidence after verification
of unchanged transport inputs/tools/logs; they were not rerun. Native initializer,
total genesis resources, actual Squads transport/CPI and live readiness remain
unproved. See [Task 2.28](TASK_2_28_CURRENT_SBF_RUNTIME_REFRESH.md).

## Previous checkpoint — Task 2.27

Task 2.27 is TECHNICALLY VALIDATED / PENDING FOUNDER ACCEPTANCE on integration.
Writer and root independently executed 15 Node tests plus eight old-profile and
sixteen recipient-profile CLI cases, first attempt with no failures, skips or
diagnostics. All 96 source/manifest inputs match both executions and inspection.
The old CLI golden is byte-identical; the complete 35/34-account Task 2.26 profile
binds literal PDA seeds, full account order/privileges and exact 313-byte payload.
Negative cases cover missing/swapped/substituted/privilege-altered recipients,
approved bytes, stale topology, lookup reconstruction and buffer commitments.
T227-R1 changed a duplicate-key substitution into a unique unrelated key with the
specific canonical identity error; corrected before execution, no failed run.
Separate source/test review passed.

Real SDK roundtrips measure buffered creation and v0 execution below 1232 bytes
with the synthetic 16-target ALT. Neighboring minimum-fit/oversized cases are
measured separately from SDK-refused candidates; compute/heap prefixes are sizing
illustrations only. Main packets are 940/907 bytes, or 988/955 with prefixes. No
signatures, real ALT/buffer lifecycle, native initializer, account-control proof
or runtime/resource/rollback execution is established.

Root verified twelve new logs and sixteen retained Rust logs. **468 Rust tests
+1 doctest/eight gates** remain Task 2.26 evidence with all 92 Rust inputs unchanged,
not rerun. Historical Task 2.14 SBF evidence remains restricted to its old artifact.
Root evidence: `/tmp/piv1-t227-pilot-host-20260920-a/pilot-summary.json`;
writer: `/tmp/piv1-t227-writer-20260920-e_q97383`. See the
[report](TASK_2_27_RECIPIENT_CHECKED_GENESIS_TRANSPORT.md). Save/STOP after reviewed
integration publication; Task 2.28 is NOT STARTED.

## Previous checkpoint — Task 2.26

Task 2.26 is TECHNICALLY VALIDATED / PENDING FOUNDER ACCEPTANCE on integration.
Root personally executed **468 host tests +1 doctest/eight gates PASS**; the
writer executed **22 initialization +8 recipient-preflight tests PASS**. Both
passed first attempt without failures, ignored tests or diagnostics. All 92
source inputs and retained log/tool hashes match the separately reviewed freeze.
See [report](TASK_2_26_RECIPIENT_CHECKED_GENESIS_INITIALIZATION.md).

Six new groups cover 24 complete 35/34-account worlds, original rent/custody and
recipient preservation, actual persisted pending recognition twice, zero-effect
invalid recipients/roles/borrows, both recipients' balance/owner/data tampering
at all 40 CPI boundaries, invocation failures and error precedence, shared reads,
replay and native closure. Explicit clone/discard modeling is separate from raw
host partial effects and establishes no actual Solana rollback. The helper
extraction retains all eight Task 2.25 tests and earlier API profiles.

New transport and current runtime proof remain absent. Nine Node tests/eight
cases are retained for the earlier exact initializer template, not rerun; Task
2.14 SBF evidence remains historical. Full recipient-control, spending-limit/
stale-action/live-artifact and funding prerequisites remain deferred. Save/STOP
after integration publication; Task 2.27 is NOT STARTED.

## Historical checkpoint — Task 2.25

Task 2.25 is TECHNICALLY VALIDATED / PENDING FOUNDER ACCEPTANCE on integration.
Root personally executed **462 host tests +1 doctest/eight gates PASS**, zero
failures, ignored tests or diagnostics, on 92 frozen inputs. Separate source/test
review passed. The writer's initial focused run was 7/8 with one warning; one
independently justified test-only correction produced 8/8 PASS without diagnostics.
Both executions and all log/tool hashes are retained in the [report](TASK_2_25_GENESIS_RECIPIENT_PREFLIGHT.md).

Eight groups exercise 72 positive recipient worlds, independent PDA/rent/model
oracles, exact approved keys and same-multisig derivations, positive funding under
zero/default/changed rent, metadata/role/key/backing/guardian aliases, mutable
borrow failures, fresh approval/context/full-message binding and host/native
guards. Full fixtures remain unchanged; no initialization, payment or ledger
recognition occurs. Current vault identity does not establish exclusive spending
control or absence of delegated limits/stale actions. The new 32/31-account
preflight fixture has no transport/runtime execution proof. Nine Node tests/eight
cases remain retained evidence of the old exact initializer template, not rerun;
Task 2.14 SBF evidence remains historical. Task 2.26 is NOT STARTED; save/STOP.

## Historical checkpoint — Task 2.24

Task 2.24 is **TECHNICALLY VALIDATED / PENDING FOUNDER ACCEPTANCE** on integration.
Root personally executed **454 host tests +1 doctest/eight gates PASS** on the
separately reviewed source; the writer executed **16 focused tests PASS**. Both
passed first execution, with zero failures, ignored tests or diagnostics. All
90 inputs match inspection and both executions; log/tool hashes were verified.

Six new regression groups cover 48 prefund/pause/topology success worlds with
independent rent/balance oracles, actual persisted pending recognition twice,
aggregate/destination overflow before effects, all 40 CPI failure boundaries,
full-batch tampering, fresh approvals/roles/borrows, replay and partial state.
Both raw partial host effects and explicit clone/discard modeling are recorded;
neither proves runtime rollback. Original rent obligations remain external and
initial history remains zero. See [report](TASK_2_24_GENESIS_TOKEN_PREFUND_NORMALIZATION.md).

The six transport-template inputs remain unchanged: nine Node tests/eight cases
are retained, not rerun. Task 2.14 SBF evidence is historical only. Later donations
to already Token-owned accounts, operational funding provenance, recipient
control, native exposure and current runtime proof remain unresolved. Save/STOP
after reviewed integration publication; Task 2.25 is NOT STARTED.

## Historical main integration checkpoint (D-028)

Tasks 2.20–2.23 are **TECHNICALLY VALIDATED / FOUNDER-AUTHORIZED MAIN INTEGRATION**
at implementation `3282e1ebabcb0cd88491d48a391565b8b100afa7` plus reviewed records.
This authorizes publication, not broader founder acceptance or runtime readiness.
Root reverified 94 inputs and 20 retained logs: **448 host tests +1 doctest/eight
gates** from Task 2.22, plus **9 Node tests/eight unsigned transport cases** from
Task 2.23. **No tests were rerun in this publication turn.** Separate source
review passed; see the [checkpoint](PIV1_PILOT_STATE.md) for exact evidence and
publication state. Native initializer exposure and current runtime proof remain
deferred. Task 2.24 is NOT STARTED; STOP after publication. Status sections below
are HISTORICAL records of their original sessions.

## Historical status after Task 2.22

Tasks 2.3–2.19 remain founder-accepted under D-027; Tasks 2.20–2.22 are technically
validated pending founder acceptance. Root personally executed **448 host tests
+1 doctest/eight gates**, zero failures, ignored tests or diagnostics, on the
separately reviewed source. The writer ran ten focused initialization tests.
All 90 inputs and retained log/tool hashes were verified; no corrective retry.

Task 2.22 composes fresh allocation, both legacy Token initializations and nine
initial typed envelopes. Tests independently check instruction/meta/seed and
account-byte layouts, twenty prefunding worlds, all forty CPI failure positions,
false/malformed Token success, the first vault during the second Token call,
nine late shared state borrows with no partial envelope writes, and replay.
Raw partial effects and explicit staged-discard modeling remain distinct from
runtime rollback. The native selector remains closed. Recipient/prefund/funding
constraints, Squads transport and current runtime/resource/rollback proof remain
required; Token-native excess still rejects in the separate economic accessor.
See [Task 2.22](TASK_2_22_GENESIS_ACCOUNT_INITIALIZATION.md). Save/STOP after
reviewed publication; Task 2.23 is NOT STARTED. The Task 2.14 artifact remains the
latest historical SBF evidence, covering only its earlier source.

## Historical status after Task 2.21

Tasks 2.3–2.19 remain founder-accepted under D-027; Tasks 2.20–2.21 are technically
validated pending founder acceptance. Root personally executed **438 host tests
+1 doctest/eight gates**, zero failures, ignored tests or diagnostics, on the
separately reviewed source. The writer independently ran twelve focused tests.
All 88 source inputs and retained log/tool hashes were verified. These are host
executions; Task 2.14 remains the latest historical SBF artifact evidence.

Task 2.21 proves fresh approved rent-only allocation composition in the host
model: external payer inner/outer signing constraints, rent-retaining debit,
fixed PDA seeds and pinned System instruction bytes/metas, all sixteen target
postconditions, prefunding preservation through u64::MAX, and adversarial
failures at all 38 CPI boundaries. Direct partial effects and staged-discard
rollback modeling are explicitly distinguished. The new function is undispatched;
Token accounts and nine states remain uninitialized. Same-transaction completion,
recipient/funding constraints, transport and actual runtime rollback/resource
proof remain required. See [report](TASK_2_21_GENESIS_ACCOUNT_ALLOCATION.md).
This session saves/STOPs after reviewed integration publication; Task 2.22 is
NOT STARTED.

## Historical status after Task 2.20

D-027 records founder acceptance of the bounded Tasks 2.3–2.19 milestone. Task
2.20 is **TECHNICALLY VALIDATED / PENDING FOUNDER ACCEPTANCE**. Root executed
426 host tests +1 doctest/eight gates on the separately reviewed source; the
writer independently ran ten focused preflight tests. Earlier status/count
paragraphs below are chronological evidence, not the current acceptance boundary.
No current-source SBF validation is inferred from these host executions.

## Verified baseline: Task 2.11 after corrected Task 2.3 and Tasks 2.4–2.10

Pilot executions on frozen Task 2.11 source, implementation `2eeefba0abc226bcfcadddb5f248f12ca589e09d`:
335 host tests, one doctest,
default/all-feature plus explicit no-entrypoint/cpi/idl-build checks and
warnings-denied documentation pass. Separate
source review passed within the documented scope. Founder acceptance is pending.
The task reports distinguish original executor, writer and pilot evidence; see
[Task 2.11](TASK_2_11_KIF_CLAIM_INSTRUCTION_BOUNDARY.md),
[Task 2.10](TASK_2_10_ISOLATED_KIF_CLAIM_EXECUTION.md),
[Task 2.9](TASK_2_9_STATE_ENVELOPE_PERSISTENCE.md),
[Task 2.8](TASK_2_8_GUARDIAN_CLOCK_SNAPSHOT_AUTHENTICATION.md),
[Task 2.7](TASK_2_7_ISOLATED_KIF_CLAIMS.md),
[Task 2.6](TASK_2_6_PROTECTED_PRINCIPAL_DEPOSIT.md),
[Task 2.5](TASK_2_5_INITIAL_CONTRIBUTION_BOOTSTRAP.md),
[Task 2.4](TASK_2_4_ACCOUNT_AUTHENTICATION.md),
[corrected Task 2.3](TASK_2_3_VAULT_RECONCILIATION_MODEL.md),
[Task 2.2](TASK_2_2_CONTRIBUTION_PENDING_MODEL.md),
[Task 2.1](TASK_2_1_MOCK_STAKE_POOL_ADAPTER.md), and
[Phase 1 properties](TASK_1_4_PROPERTY_TESTS.md).

**Historical mixed-age reference table.** This table originated at Task 2.11 and
received selected later updates. It is preserved as evidence history, not a current
exhaustive gap list. Use the current evidence register above and the active pilot
checkpoint for superseding bounded runtime results and remaining M4 work.

| Requirement | Current inspected evidence | Evidence still required before founder Testnet handover |
| --- | --- | --- |
| P-003/P-008/P-020–P-024: locked principal, fixed split, checked floors, external fees | Math unit/property tests; legal/illegal lifecycle tests; Task 1.2–1.4 reports | Actual handler account privileges, fixed native destinations, transfer/CPI deltas and failure rollback |
| P-013–P-019: full pending contribution value, HWM and loss recovery | Prior model/property/host evidence and Task 2.51 O complete pending/HWM cycle; P residual-HWM recovery with outstanding custody; Q actual severe retained-below-pending settlement zero floor, exact event, round-only recovery and rollback/retry | Governed recovery exit, broader custody/adapter profiles and public-cluster proof remain. P uses the finalizer residual guard, not settlement subtraction; Q alone proves that zero-floor branch. |
| P-009–P-012/P-029–P-032: permissionless cadence, one round, insufficient cooldown | Prior transition/property tests; Task 2.51 S actual production preparation rejects 6038 one second before ten days, succeeds at exact boundary only nine days after settlement, and rejects 6034 while active; full fee-only maps and no-fee replay | S uses canonical synthetic Clock timestamps, not naturally elapsed time. Actual 24-hour valid-insufficiency runtime coverage and broader malformed/runtime scope remain subject to final requirement closure. |
| P-004/P-014/P-015: first-contribution bootstrap and idle principal intake | Task 2.5 initial-only bootstrap tests with a genuine empty host fixture and later contributed-token yield | Actual initialization/transfers and protected SOL deposit; general idle integration remains separately scoped; never invent funded initial principal or contribution yield |
| A-003: distinct fixed economic custody | Task 2.3 per-vault obligations and host normalization; Task 2.4 AccountInfo/PDA/owner/token/rent/deficit tests | Complete authenticated initialization, real transfers and rent-preserving normalization |
| Direct/untracked SOL and token contributions | Task 2.2 idempotence and Task 2.3 all supported economic-vault host paths | Actual transfer races; unsupported native surplus in Token/temporary accounts; operational funding provenance |
| A-001/P-018/P-026: official accounting, protected CPI, dynamic minima | Task 2.1 mock interface and fee/slippage/minimum boundary tests | Pinned real SPL/Jito pool/list/mint/source validation, account-derived snapshot identity and exact protected instruction behavior |
| A-002/A-005: permissionless multi-leg source order, maximum fills, exact target | Mock source-capacity tests; lifecycle/property tests; Task 2.3 composition | Real preferred/source-order checks, deterministic stake/metadata authentication, adversarial runtime and multi-leg Testnet evidence |
| P-035: delayed readiness, rent recovery, cooldown reward/loss | Task 2.51 O positive cycle; retained N ordinary last-leg loss; P first-leg residual recovery with exact rent/two closures and unchanged companion; Q both legs/rents followed by severe settlement recovery; full rollback/replay/retry and exact 6073/no-CPI guards | N/P/Q use declared balanced synthetic external losses, not live slashing. Governed recovery exit and completion of a companion after early recovery remain unimplemented/unproved; no exit policy or guard bypass is invented. |
| P-024/A-004: atomic settlement, later pending integration and compounding | Pure transitions and staged host rollback; Task 2.6 zero-fee deposit, exact deltas/mint audit, carry/HWM/failure regressions | Runtime transaction rollback and real protected CPI; general fee/rounding-loss support remains OPEN |
| K-005–K-010: active snapshots, half-open 30-day periods, repeated carry | Prior unit/property/host evidence plus Task 2.51 S periods 0/1 zero-active: 19M then 28.5M KIF compound/carry; period 2 active three: 22,166,666 each and carry two; late heartbeats nonretroactive, full Config/reward/custody maps and rollback/retry | Declared synthetic Clock/governance and unsigned transport; actual signed governance/heartbeat and global earning provenance remain outside this local proof. |
| K-012: earned isolated claims remain available during pause | Tasks 2.7–2.11 authentication, persistence, host composition and strict ABI; Task 2.12 actual SBF artifact; Task 2.13 exact local SBF/Rent/System CPI, complete account effects, paused historical claims, replay/errors, events and shared-context observations (19 tests / 60 cases), retained on the Task 2.14 artifact | Signatures, Bank/AccountsDB commit and rollback, authorized initialization/earning and historical-ledger sum, actual public-cluster pause behavior; reduced-compute post-CEI failure remains unproven |
| G-003–G-005: explicit emergency pause and economic gates | Prior pure/host matrices plus Task 2.51 Bank L: 16 actual production pause/unpause calls, nine exact economic rejection/rollback/replay gates, direct pending reconciliation and K-012 earned KIF claim while paused | Local lifecycle pause profile validated with synthetic governance; unimplemented migrations and real Squads proposal consumption remain outside this evidence. |
| K-001–K-004/G-001/G-002/G-007/G-008: six guardians, 4-of-6 governance and upgrade custody | Bounded registry validation and immutable snapshot tests; Task 2.15 inspected source/host current Squads configuration and ProgramData linkage, same-program guardian set correspondence (363 host tests +1 doctest, eight gates and final separate review passed); Task 2.16 exact direct single-inner current-approval binding has source/host evidence (383 +1, eight gates, separate source/test review); Task 2.17 separately authenticates virgin Config and current Squads approvals (392 +1, eight gates, separate source/test review) | Actual Squads/PIV1 CPI, signatures, persisted replay/rollback and resource evidence for action binding; initialization transport; pause/recipient/rotation implementation and non-bypass tests; exact live authority identities and upgrade-authority verification; actual authority-transfer rehearsal requires its separate explicit authorization |
| Layout and failure atomicity | Five bounded schemas, Option length/property tests, full-state rollback assertions; Task 2.9 four typed fixed envelopes and 21 existing-AccountInfo atomic byte/alias/borrow/stale-state regressions | Authorized handlers coupling state and actual custody, runtime rollback/privileges, initialization replay/alias protection, WithdrawalLeg persistence and SBF limits |
| Reproducible and reviewed delivery | Pinned Rust/Anchor stack, locked/offline host gates, actual separate AI source reviews | Real-adapter dependency review, SBF artifact/build reproducibility, compute/size/rent measurements and final adversarial review |
| Founder testing path | Planned CLI/runbook in canonical spec; not implemented | Usable non-Rust test workflow, verified addresses/artifact, actual approved lifecycle signatures/states, cooldown/retry/recovery instructions |

## Evidence levels and deployment gate

- **Host math/state/mock:** controlled Rust data and atomic staged models only.
- **AccountInfo fixtures:** actual byte/owner/key validation on host fixtures;
  not proof of runtime locks, invocation privilege rules or CPI rollback.
- **Local runtime:** record exact harness/program artifacts, supported instruction
  paths and adversarial cases. Do not label a hand-mutated fixture as a validator.
- **Public Testnet:** record independently verified genesis/cluster, approved
  program/authority/recipient identities, artifact, transaction signatures and
  account states. Historical Phase 0 one-leg spike evidence is not the new PIV1
  deployment, multi-leg orchestration, or founder-testable milestone.

No public-Testnet deployment/fund-moving lifecycle or new key/signing workflow
is authorized yet. D-026 requires its concrete live-operation approval package.
Mainnet and authority transfers remain outside the technical development mandate.
Critical predeployment checks precede deployment regardless of phase labels.
No known critical/high defect may be hidden by an evidence-level distinction.

## Chronological dependency tracking

Tasks 2.4–2.12 are technically validated within their bounded scopes, pending
founder acceptance. Task 2.7 implementation `10dceb5` and closure `37f25a8`
are published with accepted main unchanged. Task 2.8 current-registry
activity/Clock authentication passed final separate review and pilot gates; its
reviewed closure `440e83e26d36df911ccfafac97d89b79b8b4c694` is published.
Task 2.9 typed state envelopes and atomic existing-account byte persistence
passed final separate review and pilot gates; reviewed closure
`44d70ec911ad3a78738fb90a04ac82eec3ca44f2` is published. Task 2.10 is scoped
for isolated claim execution and explicit host invocation/rollback modeling.
It passed final separate review and pilot gates; reviewed closure
`9f9a8dbba132f96e4c76a8385844570746b0636f` is published and independently verified. Task 2.11 strict claim ABI/runtime-ID boundary passed final pilot gates and
separate exact review. Reviewed closure
`14106d664c107b3a2f705ac87361768af42d0786` is published and independently verified. The next bounded
dependency is actual keyless SBF compilation/artifact inspection before a genuine
runtime harness. Task 2.12 retains two failed builds and their complete diagnostics;
helper splitting was insufficient. Separately reviewed private-state boxing then
passed the third actual locked/offline target build with zero warning/error/frame
diagnostics and unchanged source/tools/pins/protected refs. The pilot independently
inspected artifact headers, exact instruction bytes and bounded direct-call/stack
observations. Those are static evidence, not complete runtime resource proofs.
The pilot personally ran 339 tests, one doctest and all eight final host gates:
PASS, source unchanged. Sixteen previously executed Python runner tests remain
valid for the unchanged tool. Final separate source/evidence review returned PASS
without findings. Task 2.12 is technically validated, pending founder acceptance.
Both prior failures and the temporary host inventory-check correction remain
recorded in [Task 2.12 evidence and scope](TASK_2_12_KEYLESS_SBF_COMPILATION.md).
A later pinned keyless runtime harness must establish loader, actual execution/CPI
and rollback evidence; no runtime or deployment success is implied.

General fee/rounding-loss support for principal deposits remains OPEN: a protected
slippage floor alone does not preserve full historical book value. General idle
integration remains deferred; preserve pending-SOL priority and no-yield/
insufficiency behavior. Before economic handlers rely on an observation, resolve
or explicitly contain Token-native-surplus liveness and operational/rent funding
provenance. State serialization/initialization, handler privileges and exact real
adapter mapping remain separate bounded work items. Numbering does not itself
authorize or define an implementation.

## Prior local runtime milestone: Task 2.13

The [Task 2.13 report](TASK_2_13_KEYLESS_SBF_CLAIM_EXECUTION.md) is technically
validated, pending founder acceptance, at implementation
`fd5735976eef1e2728ccf54726145501573db60d`. Root executed the unchanged Task 2.12 ELF
through the pinned local SVM: **19 tests passed across 60 message cases**. Separate
final review passed without findings. This is additional local runtime evidence;
the unchanged production baseline remains Task 2.12's 339 host tests +1 doctest.

Root and reviewer independently inspected all 1366 complete account records,
four ledger/two native changes, exact System CPI/events, paused historical claims,
and sequential audit continuity. Ordinary claims used 73834/200000 CU. Shared
success/stale replay shows real first effects followed by Mollusk output discard.
All four reduced-compute cases retained unchanged raw state; post-CEI compute
failure remains unproven. Synthetic signers are not signatures; no Bank rollback,
deployment-verifier or public-network evidence is added.

The remaining initialization/earning, complete lifecycle/governance handlers,
real SPL/Jito integration and founder testing workflow remain required.
## Historical local runtime milestone: Task 2.14

[Task 2.14 pending recognition](TASK_2_14_RUNTIME_PENDING_RECONCILIATION.md) is
**TECHNICALLY VALIDATED / PENDING FOUNDER ACCEPTANCE** after final separate review.
Implementation: `7442cab7e97c422c7ee06290d5fc9d11c8b13ee6`.
Root executed 349 host tests +1 doctest/eight gates and 24 local SBF tests across
70 cases, with no failures/ignored tests. All 19 earlier claim tests remain.
New target ELF SHA: `46fd815847c236fb53ed5dc5ace79c48c5a21beac4019ffa107b80a5be69812f`.

Root and reviewer independently checked 1629 complete account records. A real
System donation of 100 lamports plus recognition and repeated no-op succeeds at
151832/200000 CU. Paused pending recognition retains native token-account excess;
either-asset/rent deficits and ABI/privilege/alias failures preserve all accounts.
Only two pending Config fields change; other accounting and custody stay intact.
Shared success/failure provides raw effects and Mollusk output discard evidence.

Active/settled/recovery offsets have host/pure evidence, while these runtime
fixtures cover Idle/pause. Token units and initialized owned state are synthetic.
This adds no actual SPL transfer, initialization/earning authority, full KIF
historical-liability proof, Bank rollback, signatures or public-cluster evidence.
The existing funding/fee/normalization/governance/lifecycle boundaries remain.
Later Tasks 2.15 and 2.16 add source/host governance prerequisites only. Task 2.16
root executed 383 host tests +1 doctest/eight gates with no diagnostics and
separate source/test review passed; see its
[report](TASK_2_16_SQUADS_INVOCATION_AUTHORIZATION.md). These tests model context,
proposal status and privileges; they do not execute Squads or prove initialization,
rotation, signatures, durable effect-once/rollback or target resource limits.
Exact Git identities, closure and next action are in the checkpoint.

Task 2.17 adds a separate source/host bootstrap authorization boundary, preserving
Task 2.16's initialized-guardian requirements. Root's 392 host tests +1 doctest and
eight gates passed on the separately reviewed freeze. Prefunded empty Config is
accepted without lamport classification; sorted Squads approvals do not establish
PIV1 slots, activity, parameter semantics, initialized state or a replay receipt.
[Evidence and limits](TASK_2_17_SQUADS_BOOTSTRAP_AUTHORIZATION.md).


Task 2.18 adds a source/host approved genesis model, with exact same-message
bootstrap authorization and deterministic initial state/target descriptors.
Root executed 404 host tests +1 doctest/eight gates on the separately reviewed
freeze. Twelve new tests cover strict format, parameter substitution, fresh
context, explicit slots/pause, two runtime IDs, full topology, zero histories,
checked periods, alias boundaries, prefunding preservation and compatibility with
existing fixed-account and guardian authentication. Official protocol/recipient
identity, actual non-Config targets, account creation, funding provenance, initial
vote activity, payout readiness and new SBF evidence remain unproven.
[Evidence and limits](TASK_2_18_APPROVED_GENESIS_MODEL.md).


Task 2.19 adds source/host Jito account identity evidence. Root's 416 host tests
+1 doctest/eight gates passed on the separately reviewed freeze. Twelve new tests
include all 432 option/future-fee combinations against exact-source extracted
upstream types and actual Stake serialization, plus identity substitutions,
owner/borrow failures, list geometry, reserve/Token restrictions, valid aliases,
raw fees and complete input preservation. Five test-only dependency edges retain
all 168 locked package identities. Official source constants and account bindings
do not establish cluster genesis, deployed artifact equivalence, validator/current
pool readiness, economic quotes, genesis composition or real account creation.
[Evidence and limits](TASK_2_19_JITO_ACCOUNT_IDENTITY.md).

## Task 2.20 — Fresh approved genesis account preflight

The same approved bytes, complete account slice and one Clock/Rent context now
compose genesis authorization/model, source-pinned Jito identity and observations
of all sixteen intended targets. Checks cover canonical addresses, current empty
System ownership, writable/nonexecutable privileges, role/backing separation,
fallible borrows and checked rent-only shortfalls. Root verified all 86 source
hashes, sixteen gate logs, two writer logs and existing tool hashes.

Actual funding source/provenance, recipient control, historical replay protection,
account creation, token initialization, persistence and initializer transport
remain required. Prefunding is preserved and unclassified; rent shortfall totals
do not establish liquidity or authorize spending. No instruction selector or
CPI is added; Task 2.14 remains the latest historical SBF artifact evidence.
[Task 2.20 report](TASK_2_20_GENESIS_ACCOUNT_PREFLIGHT.md) records exact commands,
regressions and limitations. The founder requested STOP after this saved task;
Task 2.21 was not started at that checkpoint; its later evidence is recorded above.
