# Task 2.6 — Protected principal SOL deposit composition

Status: **TECHNICALLY VALIDATED / PENDING FOUNDER ACCEPTANCE**.
Baseline: `c58580fb3ad01e0e98243652c1c3d1f8dafdf01f` on
`integration/piv1-testnet`. This bounded technical dependency follows reviewed
Task 2.5 under D-026. It does not amend confirmed economics.

## Requirements and inspected protocol evidence

P-014 preserves the full recognized contribution in principal; P-019 forbids a
normal HWM reduction. P-020 requires checked conservative integer accounting.
P-023 and master specification section 3.1 prevent protocol costs from consuming
protected principal or the compound allocation. A-001 requires protected deposit
instructions and an immutable one-basis-point cap. Sections 10.1–10.3 require
actual token-output accounting, separate liabilities/rent and a permissionless
later principal SOL deposit. G-004 blocks conversions during pause.

The pilot inspected cached, pinned `spl-stake-pool 2.0.3` source:
`state.rs:164–176`, `state.rs:222–236`, `state.rs:940–951`, and
`processor.rs:2561–2729`. The deposit processor floors gross minted units,
ceils the configured token fee, checks the protected minimum against net user
units, and increases pool lamports and supply by the full native input and gross
minted units. The accepted host adapter derives the same narrow arithmetic;
its capacity and revision behavior remain mock-specific. This source inspection
is not evidence of a PIV CPI, current cluster fees or live behavior.
Versioned upstream references: [state source](https://docs.rs/crate/spl-stake-pool/2.0.3/source/src/state.rs)
and [processor source](https://docs.rs/crate/spl-stake-pool/2.0.3/source/src/processor.rs).
The initial browser opens of these versioned URLs failed; search-indexed pages
identified version 2.0.3 and corroborated the formulas. Exact source analysis
used the local pinned crate, not an assumed latest deployment.

Verified arithmetic illustrates why the slippage floor is insufficient by
itself. With pool lamports 10,100,000 and supply 10,000,000, a zero-fee input
of 700 mints 693 units, passing a one-basis-point floor of 692, but those units
have post-deposit book value 699. An input of 707 mints 700 units with post-book
value 707. With a 1/100 fee, input 1010 mints 1000 gross / 990 net units and
has post-book value 999. No implementation may silently fund either shortfall
from HWM, rent, pending contributions, carry or a different yield allocation.

## Bounded implementation contract

Add a pure unpaused, bound Idle-to-Idle accounting boundary and an atomic host
composition with the accepted protected adapter. Only recognized historical
principal SOL may be converted. Preserve pending assets, next-cycle yield,
KIF carry and earned claims, operations, rent, the complete round header,
sequences, timing, HWM and all cumulative economic counters.

Require normalized individual-vault observations before execution, exact checked
native/token custody deltas, current validated pool observations and the protected
0–1 bps floor derived from Config. A stronger caller minimum is allowed; a caller
cannot weaken the floor. Verify actual minted units and post-pool lamports/supply
rather than accepting an arbitrary receipt or caller-selected book value.

The supported path has zero actual deposit fee and post-conversion historical
book value at least both its pre-conversion historical book value and the HWM.
Historical book value excludes separate next-cycle carry. The stronger before/
after comparison prevents consuming already-accrued unallocated historical yield.
Reject a fee-bearing conversion even if a preexisting buffer could mask its cost.
Only the historical SOL and JitoSOL unit fields change on successful conversion.
Failure preserves all custody, pool state and accounting, leaving SOL queued.

This is a conservative supported path, not a new fee allocation or a guarantee
that every queued amount can be staked. Some zero-fee inputs fail because of
integer rounding; all fee-bearing inputs are unsupported. General fee/rounding
loss support remains OPEN and must not be enabled by weakening this guard.
No new contribution netting rule, external subsidy, operational expense category,
economic minimum, automatic amount optimizer or HWM exception is introduced.

The host audit must record deposited SOL and actual newly minted user tokens,
reconcile them against the independent pool audit, and preserve its original
initial totals and all prior withdrawal/burn/fee/rent equations. Do not reset an
oracle after a conversion or represent minted tokens as external contributions.

## Scope and validation boundaries

Allowed files: a new pure state module, its export and narrow error additions;
`tests/support/vault_custody_model.rs`; a focused integration test target; this
report. The pilot owns shared checkpoint/specification/plan documentation.
No dependency, toolchain, serialized payload, accepted adapter behavior, math
crate, distribution transition, handler, transfer instruction or CPI change.
No keys, signing, validator setup, live operation or deployment.

Regression coverage must include genuine SOL contribution → bootstrap → deposit
→ later yield/distribution; partial and repeated safe conversions; preservation
of all unrelated assets/state; fee and one-lamport rounding rejection; stronger
caller floors; stale/malformed pool or receipt; pause/non-Idle/zero/overflow;
individual deficits, surplus and changed rent floors; all host transfer and
adapter failure points with full-world equality and conservation.

One delegated writer and a separate reviewer inspect bounded actual source.
The pilot runs final locked/offline workspace tests, doctest, default/all-feature
checks, warnings-denied documentation and targeted Git checks on frozen source.
Writer, reviewer and pilot evidence are recorded separately. Formatting is not
claimed if the pinned rustfmt component remains unavailable.

## Execution evidence

Scope review by `review_t23_final`: **PASS** against the concrete contract and
canonical requirements. The reviewer inspected the exact pinned source and
independent integer examples; no builds or implementation pass were claimed.
The pilot independently reproduced the three small integer examples above.

Sole writer `implement_t26_deposit` completed and froze the implementation after
95 focused/affected passing tests. Separate reviewer `review_t23_final` inspected
the exact five-file Rust/test diff and all 22 new test bodies: **PASS within the
bounded scope**, with no actionable code/test finding. The reviewer ran no builds.
The pilot completed the final gates below on unchanged frozen source. Founder
acceptance remains pending.


### Delegated writer implementation and validation — 2026-09-09 UTC

Writer `implement_t26_deposit` verified user `jerem`, branch
`integration/piv1-testnet`, and baseline HEAD
`c58580fb3ad01e0e98243652c1c3d1f8dafdf01f`. The two existing pilot-owned report
and checkpoint changes were preserved. The writer made no Git mutation or commit;
implementation acceptance, final review and workspace gates belong to the pilot.
This evidence supersedes the initial "Not started" execution note only for the
writer's implementation and focused validation; it grants no founder acceptance.

Files created or modified by the writer:

- `programs/piv1/src/state/principal_deposit.rs`: pure
  `record_protected_principal_deposit`, ordinary observation/result types and
  private checked combined-supply validation.
- `programs/piv1/src/state/mod.rs`: module and public API exports.
- `programs/piv1/src/errors.rs`: seven narrow deposit errors and display text.
- `programs/piv1/tests/support/vault_custody_model.rs`: atomic
  `World::deposit_principal_sol` and independent deposited-native/minted-user-token
  audit counters. All existing fixture bumps, original audit baselines,
  withdrawal/burn/fee/rent equations and severe-loss recovery correction remain.
- `programs/piv1/tests/principal_sol_deposit.rs`: 22 focused regressions.
- This report: append-only writer evidence; pilot scope text was preserved.

The public observation includes the accepted `SolDepositRequest` and
`SolDepositExecution`, before/after pool snapshots, and before/after individual
vault observations. The boundary independently reconstructs the entire zero-fee
quote and verifies actual token output, native/token custody deltas, checked pool
lamports/supply deltas, Config's exact tolerance, stronger caller minimum, bound
current epoch, unchanged fee facts, and combined principal-plus-pending token
holdings against global supply. It does not require a post-operation revision
increment or mock-specific capacity/liquidity changes. Account provenance, Clock
provenance, exact production snapshot identity and atomic CPI coupling remain
external trust boundaries for future implementation.

Only recognized historical SOL and historical JitoSOL unit fields change.
Both historical book values exclude separate next-cycle carry. A fee-bearing
conversion is rejected even if preexisting yield could conceal its cost. The
one-lamport loss at input 700 is rejected while input 707 succeeds; no fee/loss
allocation, contribution netting, subsidy, HWM reduction or automatic amount
optimizer is added. Some queued zero-fee amounts remain unsupported, and every
fee-bearing deposit remains unsupported. This is limited conversion support,
not complete general principal staking.

The host records measured principal native debits and actual principal token
credits separately and reconciles them to the unchanged pool audit. Native
conservation remains the original equation because the deposit is an internal
native transfer. Token conservation adds actual minted user tokens to the
original initial/external token side; minted tokens are never classified as
external contributions. A genuine second deposit after withdrawal, fee/burn,
finalization, settlement and integration preserves cooldown carry 13, both
recovered rents totaling 30, previous counters, completed history and audit
baselines. Full-world equality also covers unrelated vaults, pending assets,
KIF liabilities, operations, token rent, guardians, timestamps and headers.

Commands and actual results:

| Writer execution | Result |
|---|---|
| Initial baseline read under the default sandbox | Failed before execution: `bwrap: loopback: Failed RTM_NEWADDR: Operation not permitted`; scoped escalated reads/edits then worked |
| `/home/jerem/.cargo/bin/cargo +1.97.1 test --package piv1 --test principal_sol_deposit --locked --offline` | PASS: initial 20 tests; a progress message initially hand-counted 22 and was corrected to the actual 20 before adding two regressions |
| `/home/jerem/.cargo/bin/cargo +1.97.1 test --package piv1 --test principal_sol_deposit --test stake_pool_adapter --test initial_bootstrap --test vault_reconciliation --test contribution_pending --locked --offline` | PASS: final 22 deposit +24 adapter +18 bootstrap +22 composition +9 pending =95 tests, zero failed/ignored |
| `git diff --check` | PASS on the writer's final source; pilot checks final shared documentation separately |

No test failed during writer execution. The final two added regressions cover
successful deposit after a real host withdrawal/fee/burn/rent/carry lifecycle and
compensated individual host deficits. The unchanged composition target still
passes all Task 2.3 severe-loss corrections. Synthetic empty-pool/u64 boundary
and carry observations are labeled pure arithmetic/state evidence; they do not
reset or claim physically funded host custody. No rustfmt component was installed
or formatting pass claimed. Final workspace/all-feature/doc gates and separate
final review are not writer results.

Security boundary: no Mainnet action, deployment, live network operation, real
fund movement, key creation, signing, validator setup, authority transfer, secret
access or storage occurred. No dependency, manifest, lockfile, toolchain, accepted
adapter, math crate, serialized payload, handler or distribution transition was
changed. This AI-assisted work is not a professional independent audit. Source
was frozen after the final 95-test run and the build slot released to the pilot.


### Pilot final review and validation — 2026-09-09 UTC

The pilot inspected all five changed Rust/test files, the independent native/
minted-token audit extension, all 22 new tests and the exact source diff against
`c58580fb3ad01e0e98243652c1c3d1f8dafdf01f`. Source SHA-256 values were captured
before final review/gates and rechecked unchanged. Existing dependencies,
toolchain, math, payload schemas, accepted adapter behavior and distribution
transitions are unchanged. The separate reviewer identified only a stale report
status paragraph, corrected here; no source correction was required after freeze.

| Pilot execution on final frozen source | Actual result |
|---|---|
| `/home/jerem/.cargo/bin/cargo +1.97.1 test --workspace --all-targets --locked --offline --quiet` | PASS: 231 tests, zero failed/ignored |
| Same Cargo/toolchain: `test --workspace --doc --locked --offline` | PASS: 1 doctest |
| Same Cargo/toolchain: `check --workspace --all-targets --locked --offline` | PASS |
| Same check with `--all-features` | PASS |
| `RUSTDOCFLAGS='-D warnings'` with same Cargo/toolchain: `doc --workspace --no-deps --locked --offline` | PASS |
| Frozen-source equality, scoped changed paths, uid 1001 ownership, targeted credential/generated-file markers, `git diff --check` | PASS |
| Publication-trigger inspection | No active Git hooks, configured hooks path, or tracked `.github`/`.cargo` automation found |

Normal source and closure commit identities and publication evidence are recorded
in `PIV1_PILOT_STATE.md`. Technical validation is limited to the pure/host contract;
no AccountInfo authentication, transfer instruction, runtime transaction, CPI,
SBF or live-cluster deposit is demonstrated by these tests. Some zero-fee inputs
and all fee-bearing deposits remain unsupported; no cost policy was inferred.
No Mainnet action, deployment, fund movement, key creation, signing, authority
transfer or secrets access occurred. This AI-assisted review is not a professional
independent audit.
