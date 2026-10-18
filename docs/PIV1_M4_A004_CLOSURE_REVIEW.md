# M4 closure review — A-004 later principal compounding

**Follow-up — runtime W validated (2026-10-18 UTC).** The bounded post-completion
runtime gap identified below is now closed by actual protected principal compounding.
Root passed 42 runner tests, one locked/offline Bank build and W, with separate review.
The previously proposed S-terminal conversion executes with the exact amounts,
unchanged HWM/completed summary/KIF/categories, two pre-CPI refusals and selected
late-outer rollback/replay/distinct retry. The detailed earlier analysis and its
then-unexecuted proposal remain historical; they do not describe the latest runtime
status. See [Task 2.51 runtime W](TASK_2_51_REAL_PRODUCTION_LIFECYCLE.md) and evidence
`/tmp/piv1-t251-compound-review-20261018-a`. No production correction was necessary.
This closes only the stated boundary/profile; full M4 closure remains outstanding,
the platform-blocked case is unwaived, and M5 remains unauthorized under D-034.

## Historical analysis before runtime W

Reviewed 2026-10-18 UTC at `705e1518e7de17e31d3e5ef2c6c92d4a89683166`, as
`jerem`. Scope: A-004 boundary six and its directly related principal-deposit
requirements. This is a bounded source/evidence review, not full protocol
compliance or a professional audit. No build, test or runtime was executed.

The spec-to-code-compliance per-requirement format is used below. Its automatic
slash-command workflow is unavailable; a bounded delegated analysis and separate
root-coordinated review replace that workflow. The separate reviewer confirmed the
bounded finding and proposed arithmetic; staged-tree review is recorded in
`/tmp/piv1-m4-closure-review-20261018-a/reviewer-final.json`.

## A-004.6 — Later principal SOL/Jito compounding deposit

> "The production lifecycle has six logical boundaries: preparation/snapshot;
> protected withdrawal plus immediate deactivation; inactive-stake finalization
> to fixed escrow; atomic beneficiary settlement/accounting; pending-contribution
> integration; and later principal SOL/Jito compounding deposit. These are not a
> promise of exactly six transactions: a multi-validator round repeats leg
> transactions inside boundaries 2 and 3. Unproven later stages remain separate."
> — `docs/PIV1_DECISIONS.md:80`, A-004 (CONFIRMED).

> "After a completed distribution:"
> "3. deposit eligible principal SOL directly into Jito's stake pool;"
> "4. receive JitoSOL into the PIV principal token vault;"
> "5. update accounting from actual token output."
> — `docs/PIV1_MASTER_SPEC.md:955–964`, §10.1.

**Code verdict:** implemented within the explicitly supported zero-fee,
book-value-preserving adapter profile; confidence: high for this bounded path.
**Production-runtime evidence verdict:** absent for the later post-integration
invocation in the reviewed O/S/U/V evidence; confidence: high.
**Milestone finding:** concrete M4 coverage blocker. This is not a demonstrated
production defect and not M5 deployment packaging.

**What this demands of an implementation:** after settlement and pending
integration produce an Idle header with a prior completion summary, eligible
recognized historical SOL can be converted through the protected real pool path.
Operational rent, pending contributions, KIF obligations and separately tracked
next-cycle yield must not subsidize the conversion; accounting follows actual
output, with HWM and completed history preserved. The requirement does not demand
an automatic full-queue optimizer or permission to accept fees/book-value loss.

**Where enforcement lives:**

- `programs/piv1/src/lib.rs:52–54` exposes the real entrypoint;
  `instruction_boundary.rs:106–108` routes `PIV1SP01` to
  `principal_deposit_execution::process_instruction`.
  `instructions/stake_pending_sol.rs:3–19` requires the exact 25-byte, version-one
  ABI with independent amount and caller minimum fields.
- `principal_deposit_execution.rs:29–69` obtains real Clock/Rent, decodes and checks
  account count, rejects aliases, prepares the operation, validates writable
  borrows, and invokes with canonical principal-SOL PDA seeds. Full account
  records are compared after CPI; fixed/protocol identities are reauthenticated,
  and only the prepared Config envelope is committed before the event.
- `accounts.rs:191–193,205–260,266–307,328–372` authenticates fixed identities,
  canonical PDA bumps, owners, rent, exact state allocations/discriminators,
  zero padding, Token mint/authority and safe custody fields. Its principal
  observation retains Token-native quarantine while checking economic obligations.
- `state/reconciliation.rs:107–119,182–251` validates Idle prior-completion
  sequence binding and derives distinct custody obligations. At Idle,
  `next_cycle_yield_lamports` remains extra principal custody, separate from
  historical principal. KIF custody is its liability plus carry.
- `state/distribution.rs:312–326` accepts Idle only in normalized
  `idle_with_summary` form. `state/transitions.rs:1033–1071` creates that completed
  layout during pending integration. There is no initial-only restriction on
  the principal-deposit handler.
- `principal_deposit_execution.rs:111–150` authenticates pool and custody,
  derives expected native/token/pool/mint changes, prepares a Config write and
  emits the exact deposited/minted and historical-value observations.
  `:153–200` binds configured external identities, System/Token executables,
  canonical pool withdrawal authority, rent and all required mutable borrows.
- `integrations/jito_identity.rs:102–189` binds official pinned program/pool/mint,
  pool/list/reserve relationships, withdrawal PDA, reserve authorities, mint and
  token receivers, and the supported loader profile. It does not attest live
  cluster genesis or deployed upstream revision.
- `integrations/jito_deposit.rs:18–75` checks current epoch, permissionless pool,
  supported zero fee, mint/supply bounds, checked mint/output arithmetic and
  protected minimum. `:80–95` constructs pinned SPL variant 25
  `DepositSolWithSlippage`, with explicit minimum and fixed account order.
- `state/principal_deposit.rs:81–162` checks unpaused Idle, positive amount no
  larger than historical SOL, exact normalized custody/floors, zero-fee profile,
  observed supply/deltas and book values. Any historical-value loss or HWM
  shortfall rejects. Only historical SOL and historical token units change.
  `:212–219` permits the caller minimum to strengthen the configured floor.
- `state_persistence.rs:75–84,117–162,177–219` creates validated, padded Config
  envelopes and checks complete before bytes/identity/rent/writability before
  copying the replacement. The round is not a write target for this instruction.

**Paths walked:**

1. Entrypoint → exact ABI → fixed custody/protocol authentication → native
   deposit profile → shared principal transition → protected pool CPI → exact
   post-CPI comparison → reauthentication → Config persistence/event. All errors
   propagate without a fallback to an unprotected deposit.
2. Completed integration → normalized Idle with `last_completed` → principal
   deposit: source accepts the prior-summary layout and requires the matching
   next sequence. A non-Idle round rejects in the shared transition.
3. Unsupported fees, stale pool, caller minimum above computed output,
   historical amount exceeded, custody mismatch, or book-value erosion reject;
   this review does not waive these documented adapter limits.
4. Host evidence is complementary: `tests/principal_deposit_execution.rs:147–159`
   exercises completed history, carry/liabilities and Token-native quarantine via
   callbacks; `tests/principal_sol_deposit.rs:149–170,574–689` exercises completed
   accounting history in the model. These were inspected, not executed here, and
   cannot replace actual production-runtime composition after a completed cycle.

**Searched and inspected:**

- `rg 'principal_deposit\(|PIV1SP01' validation/genesis-bank-runtime`: one ABI
  helper at `tests/lifecycle_fixture.rs:67`; four call sites in
  `tests/production_lifecycle.rs:399–401,961`. The first three are initial
  pause/rejection, late-outer rollback and successful staking before the first
  preparation; V's call is also before its first preparation. S/U have no call.
- `rg 'last_completed|completed' programs/piv1/tests/principal*`: positive host
  coverage exists at the cited tests; this is not evidence of runtime absence in
  the implementation itself.
- Read every transaction `message_hex` from retained O/S/U/V stdout, independently
  decoded the legacy compact-length message/instruction encoding to exact end,
  and searched actual instruction data for the `PIV1SP01` selector. Also searched
  recorded CPI data for that selector (zero nested matches). Total: **215**
  transaction records, including identical replays. No post-integration deposit
  appeared under an alternative case label.

| Runtime | Transaction records | Actual deposit instruction positions (one-based) | Later boundary evidence |
|---|---:|---|---|
| O | 101 | 22/23 paused/replay; 25/26 outer-failure/replay; 27 initial success | Successful pending integration is transaction 96; no later deposit. |
| S | 30 | None | Three completed liquid cycles preserve prior summaries; no deposit. |
| U | 25 | None | One completed liquid cycle, then next active withdrawal preparation; no deposit. |
| V | 59 | 6 initial success | Pending integrations are transactions 53 and 59; no later deposit. |

The independently recomputed stdout hashes match the retained reviewer receipts:

| Runtime stdout path | SHA-256 |
|---|---|
| `/tmp/piv1-bank-smoke-production-lifecycle-run-t251-o/runtime.stdout` | `32fbc02ba873c39c435c47e887a571548f2d20400b75a2113d4dc5dcba090f70` |
| `/tmp/piv1-bank-smoke-production-lifecycle-run-t251-s/runtime.stdout` | `3806d656cb6fbe937911c4a72c96821aef6b93e2c71e5ff1b2f1d7752b933f6f` |
| `/tmp/piv1-bank-smoke-production-lifecycle-run-t251-u/runtime.stdout` | `c6be11adf6422f4c833631ff806ccc53e0d8e2ba7aaa7595ff0ca323fc31f36c` |
| `/tmp/piv1-bank-smoke-production-lifecycle-run-t251-v/runtime.stdout` | `228d2d444d64bc34a5c58188af81e63b4f98032ed7a99771ca52a1708c2a0182` |

Retained reviewer receipt hashes independently recomputed in this review:

| Receipt path | SHA-256 |
|---|---|
| `/tmp/piv1-t251-severe-loss-review-20261013-a/reviewer-runtime-o.json` | `f1c408ce72ee83d10dd9d8300f83e11ef472459cda5c6085f6f4bbba401e214c` |
| `/tmp/piv1-t251-cycle-review-20261015-a/reviewer-runtime-s.json` | `1a3b7bedfe219ce3cfea352eec0ca6085befa5c01e055f7e4b0336ba11ff7e19` |
| `/tmp/piv1-t251-insufficiency-review-20261015-a/reviewer-runtime-u.json` | `35ea652b4ef4a923a318f54a357930a833bf47cef19f4d1af92ba2f3137f415a` |
| `/tmp/piv1-t251-reward-review-20261016-a/reviewer-runtime-v.json` | `923000600a6e42bc1fbf4219fae52bc3ffae63274c31935124316420fe210d50` |

**How the verdict was reached:** A-004 and Master §9.10 explicitly separate the
later principal conversion from settlement/pending integration. Master §10.1
places it after a completed distribution. Existing runtime evidence covers the
same handler only before the first distribution, at a 1:1 pool price and without
prior completion history. Source and host tests support the later path, but do
not demonstrate its nested runtime execution, account layout or actual effects.
D-030 (`docs/PIV1_DECISIONS.md:268–280`) requires complete local production-path
execution before the exact Testnet package. Therefore this missing canonical
boundary belongs to M4. Task 2.51's required execution paragraph (`:48–55`) lists
initial protected staking but does not override A-004 or establish later staking.
The evidence finding is not classified as an asset-loss vulnerability.

---

## Minimal bounded follow-up, not executed

Extend a fresh execution of the unchanged S economic sequence, preserving its
existing oracles, to invoke later principal compounding after its third completed
integration (`production_lifecycle.rs:734–756`). This directly supplies the missing
boundary and prior-completion layout; it is not a general new validation campaign.

S's final state has pool total **1,300,000,000,000**, pool supply
**1,000,000,000,000**, historical tokens **19,000,000,000**, historical SOL
**2,459,000,000**, protected HWM **27,159,000,000**, KIF liability **66,499,998**
and carry **2**, with zero pending and Idle summary sequence two. A proposed
bounded exact conversion is **2,458,999,998 lamports** (a multiple of 13), minting
**1,891,538,460 units** at the 13:10 ratio. Independent integer arithmetic gives:

- principal SOL decreases by 2,458,999,998; two economic lamports remain;
- pool reserve/total increase by that amount; total becomes 1,302,458,999,998;
- mint/pool supply increase by 1,891,538,460; supply becomes 1,001,891,538,460;
- principal/history tokens become 20,891,538,460;
- historical book value and HWM remain exactly 27,159,000,000;
- only Config's two historical asset-unit fields change; prior summary, next
  sequence, KIF balances/credits/carry, pending, operational and rent bytes/balances
  remain exact, except the external transaction payer's fee.

These are proposed arithmetic expectations, not a passing execution. Revalidate
against the actual terminal account map and mint supply before freezing the test.
`lifecycle_fixture.rs:67` currently encodes caller minimum equal to native amount;
that is appropriate to the old 1:1 setup but too strict after appreciation. A new
bounded helper must encode an independently derived explicit minimum (for this
case, exact expected mint output), while retaining the original helper unchanged
and preserving every production slippage/HWM guard.

Use complete account maps and actual event/CPI evidence. Targeted refusal coverage
should include caller minimum above quote and an amount exceeding recognized
historical SOL, with fee-only failures before CPI. Add a selected late-outer
rollback, identical no-new-fee replay and distinct successful retry for the later
conversion, preserving all prior-summary bytes. Do not describe such an outer
failure as internal-callee failure. Inspect actual heap/CU and packet observations
rather than inheriting the initial-staking figures.

**Open questions / not checked:** no post-cycle conversion has yet been executed;
full-queue rounding-loss and general nonzero-fee support remain the documented
adapter limits, not new economic decisions. This review does not exhaust other
M4 requirements. The separately platform-blocked internal-failure operation stays
stopped and unwaived; its drafts were not inspected or resumed. M4 remains
incomplete; D-034 forbids starting M5 without explicit founder authorization.
