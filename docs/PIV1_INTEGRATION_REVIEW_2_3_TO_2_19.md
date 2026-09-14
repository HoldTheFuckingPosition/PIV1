# Integration review — Tasks 2.3–2.19

Date: 2026-09-14. Status: **TECHNICAL INTEGRATION REVIEW PASS / PENDING FOUNDER
ACCEPTANCE**. This is a foundation-milestone review, not founder
acceptance, merge authorization or a Testnet readiness declaration.

Root verified the review baseline at integration
`1bf07eae13d90744c9c18e7dc3f5543185bb6284`, with accepted main
`66193769d1cbc59cd8630df295b9a784b9c64642` as ancestor: 41 commits and 98 changed
files before this documentation pass. The founder requested a recap PR toward
main and targeted integration review, without merge or acceptance. D-026 and
the [current checkpoint](PIV1_PILOT_STATE.md) retain the sensitive-action gates.

## Delivered layers and actual reachability

| Layer | Delivered scope | Integration limit |
|---|---|---|
| [2.3](TASK_2_3_VAULT_RECONCILIATION_MODEL.md), [2.5](TASK_2_5_INITIAL_CONTRIBUTION_BOOTSTRAP.md), [2.6](TASK_2_6_PROTECTED_PRINCIPAL_DEPOSIT.md) | Phase-dependent custody equations, atomic host lifecycle/normalization, initial principal accounting and protected principal SOL conversion model | Initial contribution accounting is not account initialization. Conversion supports only its documented zero-fee, historical-value/HWM-preserving cases; operational surplus has no authenticated funding baseline. |
| [2.4](TASK_2_4_ACCOUNT_AUTHENTICATION.md), [2.7](TASK_2_7_ISOLATED_KIF_CLAIMS.md), [2.8](TASK_2_8_GUARDIAN_CLOCK_SNAPSHOT_AUTHENTICATION.md), [2.9](TASK_2_9_STATE_ENVELOPE_PERSISTENCE.md) | Actual-account validation, isolated earned-claim accounting, current guardian/Clock evidence and atomic existing-account byte persistence | Authentication and structural persistence do not themselves authorize governance, create accounts or prove all historical liabilities. |
| [2.10](TASK_2_10_ISOLATED_KIF_CLAIM_EXECUTION.md)–[2.14](TASK_2_14_RUNTIME_PENDING_RECONCILIATION.md) | Claim execution/ABI/native entrypoint, keyless build/harness and narrow pending-recognition instruction | Two dispatched paths; historical local runtime evidence uses synthetic initial state and an earlier artifact. |
| [2.15](TASK_2_15_SQUADS_AUTHORITY_SNAPSHOT.md)–[2.17](TASK_2_17_SQUADS_BOOTSTRAP_AUTHORIZATION.md) | Present authority, exact direct invocation approval, and separate virgin-Config authorization | Library prerequisites; no governance handler, rotation, initialization or persistent effect-once capability. |
| [2.18](TASK_2_18_APPROVED_GENESIS_MODEL.md) | Exact approved 313-byte model format, fresh bootstrap authorization, deterministic proposed state and sixteen target descriptors | Model output, without actual target validation/creation, protocol authentication, funding classification, recipient control or KIF activity. |
| [2.19](TASK_2_19_JITO_ACCOUNT_IDENTITY.md) | Seven-account Jito/SPL identity against fixed source identities, bounded parsing and serializer evidence | Not composed with genesis; raw fees/epochs/supplies do not establish operational readiness or deployed cluster/artifact identity. |

The [native dispatch](../programs/piv1/src/instruction_boundary.rs) has exactly
these reachable instruction paths:

| Instruction | Exact ABI/accounts | Authorized effect |
|---|---|---|
| `claim_kif` | 24 bytes; Config, guardian reward, KifSolVault, guardian signer/destination, System Program | Positive recorded liability payment with replay counter, canonical state persistence, fixed signed System transfer, postchecks and success event. Available during pause; no new earnings or current-membership requirement for historically earned ownership. |
| Pending-contribution recognition | Eight-byte selector; Config, ActiveDistribution, PendingSolVault, PendingJitoVault | Permissionless idempotent update of only the two pending Config ledgers, including pause/recovery and committed active-round offsets. No transfer/CPI or HWM integration. |

Strict shape/account-count checks reject unknown or malformed calls. Ordinary
host processing rejects execution; explicitly named host seams model context and
invocation. Rust exports, instruction markers and the genesis model selector do
not add runtime routes. Initialize, deposits, staking, distribution/settlement,
heartbeat, pause/config/recipient changes and guardian rotation are not dispatched.

## Integration invariants and resolved correction

The [canonical decisions](PIV1_DECISIONS.md) and
[specification](PIV1_MASTER_SPEC.md) retain the 59%/19.5%/19.5%/2% split,
protected lamport HWM, separate principal/pending/escrow/KIF/operational custody,
caller-paid transaction fees and delayed direct Jito withdrawal. Initial slippage
remains exactly 1 bps with an immutable 1-bps cap; the zero-fee model does not
approve a policy for unsupported fee-bearing or rounding-loss conversions.
Six guardians/4-of-6, pause, half-open 2,592,000-second periods, active-only KIF,
collective carry and historical earned claims remain unchanged.

T23-R1 is included: severe pool loss previously made the host finalization/
settlement wrapper fail subtraction before reaching `RecoveryRequired`.
Only the historical-value comparison now becomes zero when retained value is
below committed pending use. Finalization preserves recovered custody/rent;
settlement preserves only its recovery header and discards speculative payments.
The correction does not reduce HWM, forgive a custody deficit, change pending
value or replace general checked arithmetic. Its four initially failing
regressions and corrected evidence remain in the Task 2.3 report.

Current Squads configuration alone does not prove four approvals. The separate
invocation gate checks fresh nonstale current-voter approvals and exact actual
instruction bytes/accounts under the direct single-inner profile. Genesis model
preparation decodes those same approved bytes and freshly invokes bootstrap
authorization with one Clock context; no detached authorization token plus
independent parameters is admitted. Generic persistence is structural, not an
alternative governance gate. Guardian set correspondence preserves existing
slots, and rotation/activity semantics remain unimplemented.

Jito identity is also point-in-time evidence. Its official source constants and
raw protocol observations do not validate a future CPI or silently convert the
genesis declarations into authenticated inputs. These boundaries must be composed
freshly by later handlers, with reauthentication after relevant mutation/CPI.

## Evidence ledger

| Evidence | Scope and result |
|---|---|
| Current Task 2.19 source | **416 host tests +1 doctest; eight gates PASS**, zero failed/ignored tests or diagnostics. Direct verified 1.97.1 tools, clean environment, locked/offline, one job. Root verified unchanged-source/tool/log correspondence for this recap. |
| Historical Task 2.14 artifact | **24 local SBF tests/70 cases**, including claim System CPI, real local System donation plus pending recognition, and full account comparisons. ELF SHA-256 `46fd815847c236fb53ed5dc5ace79c48c5a21beac4019ffa107b80a5be69812f`. Later source has no refreshed SBF artifact/runtime validation. |
| Separate task reviews and current integration review | Recorded bounded source/test/report passes through Task 2.19. Separate reviewer `review_squads` inspected the current cross-module seams and all five frozen writer documents: scoped PASS with no additional actionable foundation-level blocker. No new build or test execution. |

The eight host gates comprise workspace tests, doctests, default/all-feature
checks, explicit no-entrypoint/cpi/idl-build checks and warnings-denied docs.
Exact commands/log hashes: [Task 2.19 report](TASK_2_19_JITO_ACCOUNT_IDENTITY.md),
`/tmp/piv1-t219-pilot-host-20260914-a/pilot-summary.json`. Historical SBF records:
[Task 2.14 report](TASK_2_14_RUNTIME_PENDING_RECONCILIATION.md). Runtime fixture
output discard is not Bank/AccountsDB rollback or signature verification. The
accepted Phase 0 public-Testnet protocol spike is distinct from PIV1 deployment.
Root's `/tmp/piv1-integration-review-20260914-a/evidence-reuse.json` and
`reviewed-source.json` verify all 84 current source/manifest/lock/toolchain hashes
against the Task 2.19 freeze, all sixteen retained gate log hashes, three host
tool hashes and eight successful exits. The historical ELF also matches its
recorded 229,208 bytes/hash; it is not a current-HEAD artifact. No source,
manifest, test or tool change, build or test run occurs in this recap.

Across the aggregate PR, pinned legacy Token decoding adds `spl-token` and six
transitive packages relative to accepted main: 161 to 168 locked package
identities, with no removed identity. Task 2.19's 168 unchanged package
identities/five added dev-feature edges describe only its comparison with Task
2.18, not the aggregate PR. Config serialization remains stable; its address
validation correctly admits the all-zero System Program ID in that dedicated
role while preserving the required default/alias exclusions elsewhere.

## Review finding and remaining blockers

**IR-001 — stale technical status documentation: RESOLVED / REVIEW PASS.**
Root README, program README and master technical summaries still said no
entrypoint/handler/CPI/cdylib or described SBF as wholly future. They now describe
the two paths, current host validation and separately historical runtime proof,
linking maintained evidence instead of extending obsolete task lists. Economic
requirements, accepted decisions and historical task evidence are unchanged.

The next dependencies are concrete implementation/evidence gaps, not reasons to
reopen confirmed economics:

1. Compose exact fresh Squads-approved genesis parameters with current Jito
   identity and recipient constraints; validate all sixteen actual targets;
   implement prefunding-safe atomic creation/persistence and explicit funding
   provenance. Existing models do not establish operational rent funding.
2. Prove the final initializer ABI/account/transaction/compute budget and runtime
   behavior. The 313-byte model format is not that ABI. Stored Squads lookups are
   unsupported in the bounded gate; outer v0 transport is distinct, and pinned
   Squads buffering may preserve one atomic initializer. Transport remains unproven.
3. Implement real SPL/Jito mapping/CPI and lifecycle handlers with current epoch,
   exchange/fees, validator/source-order, slippage, exact deltas, rent, liquidity,
   HWM and multi-leg settlement checks. Unsupported fee/rounding paths, operational
   surplus and token-account native-excess extraction retain their documented limits.
4. Implement governance/rotation synchronization and heartbeat/qualifying-vote
   activity, without erasing old earned claims or inferring activity from an
   approval bitmap. Genesis inactive records do not establish first-payout readiness.
5. Refresh SBF/runtime evidence for the complete composed source, including actual
   Squads invocation, initialization and lifecycle failure/rollback boundaries.
   Verify official cluster identities and deployed artifacts before public use;
   fixed source addresses do not prove Squads/Jito Testnet availability. Dedicated
   PIV1 identity, final guardians/recipients and exact deployment/signing/authority
   approvals remain outside this PR.

Recommendation: present these delivered layers and explicit limits in a
**draft foundation-milestone PR**, with separate founder acceptance before merge.
The scoped technical review is complete; no new foundation-level source blocker
was found. This conclusion reuses the bounded task reviews and directly checks
the executable seams described above; it is not a fresh review of every line.

Root verified the writer's freeze, retained evidence, unchanged accepted math,
constants, distribution/guardian/timing/transition modules, abstract stake-pool
interface and historical spike. Config layout is unchanged; the validation
correction above is explicit. Review commands included `git diff`, ancestry and
independent remote-ref reads, source/log/tool SHA-256 comparisons, locked package
identity comparison and final documentation/whitespace/publication checks.
No source, manifest, lockfile or test is changed by this recap.

The seven changed documents are root/program READMEs, master technical summaries,
this report, exact PR text, `AGENTS.md` and the pilot checkpoint. Git records the
recap commit on `integration/piv1-testnet`; accepted main remains at `66193769`.
No Mainnet action, deployment, fund movement, key creation/signing or authority
transfer occurred. The writer/reviewer ran no Git/network/build/test operation;
root used read-only remote checks and normal integration publication only.
AI-assisted review is not a professional independent audit.

## Prepared PR opening

Git SSH publication is available, but no authenticated GitHub API/PR capability
is configured. A public API read found no open PR for the exact integration/main
pair. **The PR is prepared, not created.** The title and body are preserved in
[PIV1_INTEGRATION_PR_2_3_TO_2_19.md](PIV1_INTEGRATION_PR_2_3_TO_2_19.md).

[Open the prefilled PR creation form](https://github.com/HoldTheFuckingPosition/PIV1/compare/main...integration/piv1-testnet?quick_pull=1&title=Review+Tasks+2.3%E2%80%932.19%3A+accounting%2C+runtime+and+initialization+foundations&body=This+draft+reviews+the+development+sequence+from+accepted+%60main%60+to%0A%60integration%2Fpiv1-testnet%60.+It+extends+the+accepted+pure+accounting+foundation%0Awith+authenticated+state%2Fcustody%2C+two+narrow+runtime+instruction+paths+and%0Alibrary+prerequisites+for+initialization.+It+requests+foundation+review%3B%0Afounder+acceptance+and+merge+are+separate+decisions.%0A%0A-+Adds+phase-dependent+custody+reconciliation%2C+initial+principal+and+bounded%0A++protected-deposit+models%2C+including+T23-R1+severe-loss+recovery+correction.%0A-+Connects+authenticated+earned+KIF+claims+to+fixed+System+transfers+and+adds%0A++permissionless+recognition+of+already-received+pending+contributions.+These%0A++are+the+only+dispatched+instructions%3B+recognition+moves+no+funds.%0A-+Adds+guardian%2FClock+evidence%2C+atomic+existing-account+byte+persistence%2C%0A++current+Squads+authority+and+exact+direct+invocation+approval+checks.%0A-+Adds+approved+deterministic+genesis+model+preparation+and+source-pinned%0A++seven-account+Jito+identity.+These+remain+separate+prerequisites%3B+no+initializer%2C%0A++production+Jito+CPI+or+complete+distribution%2Fgovernance+lifecycle+is+callable.%0A-+Corrects+stale+README%2Fmaster+technical+summaries+that+still+denied+the%0A++implemented+entrypoint%2FCPI%2Fartifact+boundary+%28IR-001%29.%0A%0AConfirmed+economics%2C+six-guardian%2F4-of-6+authority%2C+pause+behavior%2C+protected+HWM%2C%0Aisolated+custody+and+earned+KIF+ownership+remain+unchanged.+Fee-bearing+deposit%0Amodels%2C+operational+funding+provenance+and+other+unsupported+paths+are+not%0Asilently+promoted+into+production+behavior.%0A%0AValidation%3A+%2A%2A416+host+tests+%2B1+doctest+%2F+eight+gates+PASS%2A%2A+on+current+Task+2.19%0Asource.+Separately%2C+the+%2A%2Ahistorical+Task+2.14+artifact%2A%2A+passed+%2A%2A24+local+SBF%0Atests%2F70+cases%2A%2A%3B+that+runtime+evidence+does+not+cover+later+source+additions%2C%0Aactual+initialization%2C+Squads+CPI%2C+Bank+rollback+or+public+deployment.+Root%0Averified+unchanged+source%2C+retained+logs%2Ftools+and+the+historical+ELF.+Separate%0Atargeted+integration+review+passed+with+no+additional+actionable+foundation-level%0Ablocker.+This+recap+review+adds+documentation+only+and+reuses+the+existing+test%0Aexecutions%3B+it+does+not+claim+a+new+test+run.%0A%0ABefore+a+usable+PIV1+lifecycle%3A+compose+fresh+approved+genesis%2Fprotocol+evidence%2C%0Avalidate+and+atomically+create+actual+accounts+with+funding+provenance%2C+prove%0Ainitializer+transport%2C+implement+protocol%2Fgovernance%2Factivity+handlers+and+refresh%0Afull-source+runtime+evidence.+Official+cluster%2Fartifact+identities+and+exact%0Adeployment%2Fsigning%2Fauthority+approvals+remain+separate.+This+PR+does+not+assert%0ATestnet+readiness+or+grant+merge%2C+acceptance+or+live-operation+permission.%0A%0ASee+the+%5Bintegration+review%5D%28https%3A%2F%2Fgithub.com%2FHoldTheFuckingPosition%2FPIV1%2Fblob%2Fintegration%2Fpiv1-testnet%2Fdocs%2FPIV1_INTEGRATION_REVIEW_2_3_TO_2_19.md%29+for+delivered%0Alayers%2C+exact+entrypoint+reachability%2C+T23-R1%2C+IR-001+and+remaining+dependencies%3B%0Athe+%5Bcheckpoint%5D%28https%3A%2F%2Fgithub.com%2FHoldTheFuckingPosition%2FPIV1%2Fblob%2Fintegration%2Fpiv1-testnet%2Fdocs%2FPIV1_PILOT_STATE.md%29+records+actual+publication+state.+The+review%0Abaseline+was+%601bf07eae13d90744c9c18e7dc3f5543185bb6284%60+against+accepted+main%0A%6066193769d1cbc59cd8630df295b9a784b9c64642%60+%2841+commits%2F98+files+before+recap+docs%29.%0AAI-assisted+review+is+not+a+professional+independent+audit.).
Choose a draft foundation review when submitting it in an authenticated GitHub
session; opening this form does not itself create or merge a PR. The link uses
GitHub's documented `quick_pull`, `title` and `body` parameters
([GitHub reference](https://docs.github.com/en/pull-requests/reference/using-query-parameters-to-create-a-pull-request)).
