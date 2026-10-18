# M4 validation-scope review without Daybreak

Status: **REVIEWED EVIDENCE-SCOPE PROPOSAL / FOUNDER DECISION PENDING**.
Baseline: `2f696a9b79a83c90b55c7c709030943c3c7990ec`, 2026-10-18 UTC.
This proposal does not close M4, waive a gate or authorize M5.

**Later authorization (D-039, 2026-10-18 UTC):** a separate founder instruction
authorizes M5 PREPARATION with the M4 reservation carried forward. It does not
accept this proposal, close M4, waive its criterion or clear the stopped operation.
See the [M5 readiness register](PIV1_M5_TESTNET_READINESS.md). The proposal and
pre-D-039 M5 restrictions below remain their historical record; live deployment
is still unauthorized.

## Finding and provenance

Daybreak is not a Solana dependency or an economic requirement of PIV1.
The current obstacle is a platform-rejected validation operation and a remaining
runtime-evidence obligation. No PIV1 defect or Testnet malfunction is demonstrated
by that rejection. The rejected operation and its drafts remain stopped.

Canonical P-024/A-002/A-004 and master specification section 9.6 require atomic
custody/accounting, no partial beneficiary payout and safe retry. Section 18.3
requires failed-CPI and retry coverage. These prescribe behavior, not a particular
resource-limit test. However, the pilot mandate delegates completion-gate design:
Task 2.51 already required actual late/internal failure rollback in `b9b0f077`
(October 4), and `2f8d0836` (October 12) explicitly identified a callee observed
mutating before internal failure. Both predate the platform interruption. This
is a legitimate delegated technical gate, not a demonstrated documentary error
that can simply be removed to obtain a passing milestone.

## What existing evidence supports

- Reviewed production source propagates invocation errors before final accounting
  persistence; settlement commits its state writes only after successful transfers
  and receipt checks (`programs/piv1/src/settlement_execution.rs:55–91`).
- Existing host tests cover modeled transfer failures, partial effects and explicit
  failed-world discard. They are not actual Bank rollback observations.
- Current-artifact AB exercises the full delayed production lifecycle and selected
  late-outer rollback/replay/retry. Retained independent reviews cover 19,497 account
  records, 187 maps, 101 transaction pairs, 145 CPIs and 40 events. Other retained
  profiles cover the documented economic, fee, loss, cadence and governance branches.
- The source, host tests and real Bank results support a compositional engineering
  argument for atomicity. They do not establish the missing empirical observation
  simply by being combined. No claim of exhaustive correctness or professional
  independent audit follows.

The still-unproved proposition is an invoked callee observed changing state and
then failing internally on the relevant production artifact/runtime, followed by
complete fee-only rollback, identical no-fee replay and a distinct successful retry.
The current [closure register](PIV1_M4_CLOSURE_STATUS.md) remains incomplete.

## Concrete proposed scope decision

If the founder wishes to accept the completed supported local deliverable using existing evidence,
the precise proposal is **acceptance of the supported local M4 deliverable with
one explicit unresolved runtime-evidence reservation**. This would be an informed
change to delivery/acceptance scope, not a new test result, a claim that the old
technical criteria passed, or proof that the residual risk is zero.

A possible founder decision, not yet given, is:

> I accept the supported local M4 deliverable with the explicitly recorded
> internal-callee-failure runtime-evidence reservation. I do not treat that test
> as passed or authorize its circumvention. Preserve the reservation for an
> independent validation assessment before any live-deployment approval. Stop
> before M5; this acceptance does not authorize M5 or a deployment.

Even if this limited deliverable is accepted, maintain **M4 TECHNICALLY INCOMPLETE**
under the unchanged criteria; record **SUPPORTED LOCAL DELIVERABLE ACCEPTED WITH
RESERVATION** separately. Such acceptance does not by itself close technical M4.
Until a decision is given, even that limited acceptance remains pending. A generic
request to finish without Daybreak
is not a passed test or inferred risk acceptance. If the founder requires full
technical closure under those unchanged criteria, the remaining evidence must
actually be supplied through a legitimate permitted validation process; this
review provides no rerouted execution instructions for the rejected operation.

No economics, custody invariant, production guard, test oracle or dependency is
changed. Main publication, founder functional acceptance and live-operation
authorizations remain separate. D-034 still prohibits starting M5 without an
explicit later authorization, regardless of any qualified delivery acceptance.

## Review evidence and limits

One native delegated requirement/provenance review and a separate refutation
concur on the distinction between supported evidence and the missing runtime
observation. Root independently checked the original Task 2.51 paragraph,
canonical requirements and production error propagation. Root verified all 281
current pinned source hashes and five ELF hashes unchanged. No test, build or
runtime was executed; earlier test counts and runtime traces are retained results,
not freshly rerun or exhaustively reaudited. No rejected draft was inspected or
executed in this review. Evidence: `/tmp/piv1-m4-validation-scope-review-20261018-a`.
