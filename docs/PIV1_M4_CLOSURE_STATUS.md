# M4 closure-gate analysis

Scope: one milestone requirement, at `c71e9fa20fd666b2ffd0335a4b440fa2c657129e`.
All line anchors below refer to that analyzed commit, before this checkpoint.
Source/evidence inspection only; no test, build, runtime or live operation executed.
This is bounded manual requirement analysis, not exhaustive compliance or a
professional independent audit. Separate source and documentary review passed
for this bounded milestone analysis.

## Controlling requirement and closure rules

D-030 says: “Execute and review a complete local end-to-end lifecycle using the
real production instruction paths” (`docs/PIV1_DECISIONS.md:276`). It also says
“Follow this order unless a demonstrated dependency requires otherwise” (`:283`)
and “Retained limitations are not automatically separate prerequisite tasks”
(`:288`). The current M4 table requires the reviewed exact artifact, complete
contributions-to-KIF lifecycle, adversarial/failure/retry gates and requirement-to-
evidence closure (`docs/PIV1_CODEX_EXECUTION_PLAN.md:604`). Lines 608–615 require
proportionate final gates and evidence reuse where source/artifact scope applies.
D-034 (`docs/PIV1_DECISIONS.md:406`) prohibits starting M5 after M4 completion
without explicit founder authorization; this review declares no completion.

## Evidence already supporting the economic lifecycle

The current register (`docs/PIV1_TEST_PLAN.md:66`) and Task 2.51 distinguish:
O's full delayed two-leg lifecycle, pause and earned claim; N/P/Q recovery-entry
profiles; S repeated liquid cycles/cadence/carry; U valid-insufficiency retry;
V cooldown reward through later split; W post-completion principal compounding;
X2 per-leg positive fees through net settlement; and Y active economic-surplus
normalization through full pending contribution/HWM integration.
These profiles remain valid within their recorded scope. No production defect or
new economic runtime branch blocker is demonstrated by this inspection.

Y's retained evidence is 54 runner tests, locked/offline Bank build and runtime,
8,352 account records/114 maps, 58 transaction pairs, 113 CPIs and 12 PIV events
(`docs/PIV1_TEST_PLAN.md:80`). Prior production host/SBF evidence applies only to
unchanged sources/artifacts; no fresh execution is inferred. Root separately owns
hash/provenance verification. The milestone-table row at execution plan 604 omits
X2/Y and needs reconciliation, not rerunning their validated profiles.

## Remaining gates and actual implementation debt

1. The internally mutating callee-failure case is explicitly STOPPED/UNWAIVED in
   `AGENTS.md:80` and `docs/PIV1_TEST_PLAN.md:123`. Late-outer rollback is different
   evidence (`docs/PIV1_TEST_PLAN.md:76`). No draft or underlying rejected action
   was inspected, resumed or rerouted. This known gate prevents completion and
   must not be renamed, waived or bypassed by another harness task.
2. Final independent requirement-to-evidence closure and applicable final gates
   remain open (`docs/PIV1_TEST_PLAN.md:123`). This bounded analysis contributes to
   closure; it is not the complete final review. Reuse unchanged evidence and run
   new gates only for relevant changed code or demonstrated unresolved concerns.
3. M2 has a concrete missing capability, not merely missing runtime permutations:
   recipient/guardian replacement handlers. The M2 table expressly retains these
   operations (`docs/PIV1_CODEX_EXECUTION_PLAN.md:602`), and canonical handler scope
   requires “Authenticated pause/unpause, recipient updates and guardian-set
   changes with confirmed governance and liability preservation” (`:697`).
   `programs/piv1/src/instructions/update_config.rs:7` and `:8` are marker types
   only. Complete native dispatch (`programs/piv1/src/instruction_boundary.rs:89`
   through `:158`) has no replacement route. Guardian dispatch only supports
   heartbeat/pause (`programs/piv1/src/guardian_operations.rs:61`). The property
   test at `programs/piv1/tests/property_invariants.rs:1597` replaces the registry
   directly in a model; it is not authenticated production rotation.

The fixed-recipient/current-guardian economic cycle does not itself need a
replacement instruction; Task 2.51 explicitly permits synthetic governance
(`docs/TASK_2_51_REAL_PRODUCTION_LIFECYCLE.md:49`–64). Thus the missing handlers do
not invalidate O–Y. They are uncompleted required M2 capability debt before the
ordered milestone sequence can be declared closed. No demonstrated dependency
justifies silently moving that capability to M5 or launch-only work.

## Next bounded implementation: supported recipient updates

Implement `UpdateRecipients` through the real production boundary next, using
existing pinned authority/recipient validation. K-002 already fixes 4-of-6
approval (`docs/PIV1_DECISIONS.md:61`); G-007 permits replacement (`:93`). Master
12.1/12.4 requires approved multisig updates and verified destinations
(`docs/PIV1_MASTER_SPEC.md:1163`, `:1203`). No missing economic/governance decision
is demonstrated for this narrow supported implementation.

- Authenticate the complete approved instruction bytes and current governance,
  trusted runtime context and fixed account topology; preserve current restrictions.
- Support verified funded, non-executable, empty System vault PDAs of the same
  authenticated multisig, using current genesis identity/rent checks
  (`programs/piv1/src/genesis_recipients.rs:140`–162). Do not assume the genesis-only
  wrapper itself is callable after initialization; share/adapt its checked identity
  logic under current-config authorization. Vault indices are derivation witnesses,
  not invented policy or live addresses. As the genesis module explicitly warns
  (`:3`–6), PDA derivation/funding does not establish exclusive 4-of-6 spending:
  spending limits, stale transactions and live Squads identity remain distinct.
- Change only the two Config recipients and factual event after successful checks.
  Preserve full active-round bytes, economic/custody state, all KIF liabilities,
  historical balances, HWM and existing rents. Preparation copies recipients into
  the immutable round (`programs/piv1/src/state/transitions.rs:467`–468); settlement
  uses those frozen recipients (`programs/piv1/src/settlement_execution.rs:231`–232).
  An update must not redirect an already-active round.
- Review strict ABI/roles, bad approval/current authority/recipient/alias/borrow/
  malformed/replay handling and exact unchanged state on rejection. Require focused
  tests plus proportionate applicable final host/SBF gates on the changed artifact.
  Review an actual production-path local update, preserved active-round settlement
  to old recipients and later preparation using new recipients, with exact state
  maps and appropriate failure/retry evidence. This closes an implementation
  dependency; it does not reopen the stopped internal-failure operation.
- Review and checkpoint the final change separately. Then handle guardian-set
  replacement as a distinct bounded implementation with explicit old earned-
  liability ownership and active snapshot preservation. Do not assume the existing
  Squads invocation verifier already solves rotation choreography: its stated
  scope excludes it (`programs/piv1/src/squads_execution.rs:3`–16).

This recipient profile does not establish arbitrary final program-controlled
recipient compatibility: current settlement accepts only non-executable empty
System accounts (`programs/piv1/src/settlement_execution.rs:194`–198). Keep the
supported scope explicit; do not invent those future programs or addresses.

## Limits that are not automatic new runtime tasks

Master 18.3 explicitly starts “Use a controllable mock stake pool/adapter capable
of” (`docs/PIV1_MASTER_SPEC.md:1559`). Its full test list cannot alone require a
fresh Bank profile for every lifecycle combination. Positive preexisting KIF debt
at Y normalization, every fee/source regime, every guardian count and every
rent/cooldown permutation need a demonstrated requirement dependency before
becoming separate critical-path work. Complementary host evidence remains relevant.

P-035 requires recovery entry without lowering HWM (`docs/PIV1_DECISIONS.md:54`);
N/P/Q provide bounded evidence. No automatic recovery exit or outstanding-leg
bypass policy is authorized by that rule. Keep the unimplemented exit explicit;
do not invent custody powers to close a checklist.

Task 2.51 permits declared synthetic external genesis/governance/reward inputs
(`:58`–64). Synthetic proposal non-consumption, unsigned transport, initializer
resources and observed 1,544-byte unsigned packet remain real readiness limits,
not proof of usable live deployment. Exact package/address/authority/funding/
founder workflow belongs to M5 (`docs/PIV1_CODEX_EXECUTION_PLAN.md:605`), forbidden
until explicitly authorized. This does not waive those limits or claim they can
all be fixed by packaging. Live operations/signing and founder acceptance remain
separate gates; no M5 package is prepared here.

## Search provenance and exclusions

Read actual AGENTS current/mandate sections, decisions D-030/D-034/K/G, master
12/14/16/18.3, execution-plan milestone table/canonical handler scope, current test
register and Task 2.51's scope/evidence. Searched `M4|failure|retry|final gate`,
`recipient|rotation|replacement|recovery`, and Rust `UpdateRecipients|UpdateGuardianSet`.
Inspected full native dispatch, marker modules, guardian action switch, recipient
identity/settlement and round snapshot assignments. Symbol search found marker
and re-export only; source inspection independently confirms missing dispatch.
Two initial guessed document names were absent; actual master filename was then
resolved with `rg --files`; no missing file contents are inferred.

Not checked: exhaustive protocol compliance, every test assertion/trace, fresh
upstream/live behavior, real Squads consumption, guardian rotation implementation
or final-program recipient design, secret material, blocked drafts, deployment
artifacts/addresses/budgets. No tests, builds or runtime were executed.

## Implementation follow-up — Task 2.52 (2026-10-18 UTC)

The marker-only recipient finding above is historical after
[Task 2.52](TASK_2_52_RECIPIENT_UPDATES.md). The strict supported handler is implemented
and separately reviewed; root passed nine focused tests, 605 host tests plus one
doctest/eight gates and fresh strict SBF/static inspection. Production runtime of
this new artifact is still pending. Next execute update/old-round settlement/new-
round preparation with independent exact evidence; then guardian replacement.
This is implementation progress, not M2/M4 completion, founder acceptance or M5
permission. All blocked-case and live-operation limits above remain binding.

## Runtime follow-up — Task 2.52 Z (2026-10-18 UTC)

The previously pending recipient runtime gate is now closed for the supported
local profile; see [Task 2.52](TASK_2_52_RECIPIENT_UPDATES.md). Current PIV and rebuilt
restricted caller execute actual authorized replacement during a prepared round,
old frozen settlement and later preparation/settlement to new recipients. Root
passes 60 runner tests, 13 caller tests/check/docs/SBF and Bank build/runtime;
separate raw-trace review verifies all maps, events, custody, selected rollback,
replays and retries. Prior 605 production host tests +1 doctest/eight gates/SBF
remain retained evidence. Earlier analysis and line anchors above are historical.
Required guardian replacement remains the next bounded capability; preserve old
earned ownership and frozen activity snapshots. Final M4 closure remains open,
the blocked internal-failure operation remains stopped/unwaived and M5 unauthorized.

## Implementation follow-up — Task 2.53 (2026-10-18 UTC)

[Task 2.53](TASK_2_53_GUARDIAN_SET_SYNCHRONIZATION.md) implements the required
one-seat synchronization capability with dedicated current 4/6 exact approval,
checked revision and new externally funded reward records. Old earned claims,
prepared snapshots and same-identity activity are preserved. Root host/static SBF
gates and separate source/evidence review pass; actual new-artifact runtime remains
pending. Next bind one supported local synchronization lifecycle, future activity,
old-round credits and retired claims. Earlier marker-only findings are historical;
this is neither full M2/M4 completion nor M5 authorization. Z retains its recorded
artifact scope and the platform-blocked internal-failure case remains stopped.
