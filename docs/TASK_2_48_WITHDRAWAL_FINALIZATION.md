# Task 2.48 — Production withdrawal finalization and rent recovery

Status: **TECHNICALLY VALIDATED / PENDING FOUNDER ACCEPTANCE** under D-026/D-030 M2 and its bounded M3 component.
Start integration: `3b0a4f719e64bda45a199f31d53a3bf58be54f62`; main: `1054ff3`.

## Accepted contract and bounded scope

Master specification 9.2/9.6, Phase 0 section 9.6, decisions P-035/A-004/A-005
and the existing leg-finalization model require authenticated inactivity, complete
Stake-to-fixed-escrow native receipt, exact original rent recovery, checked cumulative
reward/loss/HWM handling, replay protection and atomic temporary metadata closure.
Expose these production operations with a strict permissionless boundary while
preserving pause, custody, pending offsets, carry, KIF and historical principal.

Canonical Stake 5.1 full-balance Withdraw rejects nonzero effective stake using
its current runtime Clock and Stake History, then zeroes stake data on full drain.
That CPI and exact full-closure receipt provide authoritative inactivity proof.
The retained interface 1.2.1 exposes older activation arithmetic: no claim is made
that its legacy calculation equals Stake 5.1 V2, and no dependency migration occurs.

Let B be observed Stake balance, R its recorded original rent and D its current
owner-authenticated delegated amount. Support R <= B and B-R <= D. Full B enters
escrow; exact R returns to operations. Reward/loss compares B-R with the recorded
original delegated receipt, following accepted Phase 0 arithmetic. A deficit below
R rejects because full rent recovery is not proved. Excess B-R>D also rejects:
Agave can adjust delegation when current rent rises, so apparent excess cannot
always be labeled a donation or reward from present fields. This bounded profile
may block a donated/rent-adjusted Stake account; provenance recovery remains open.
An earlier masked deficit cannot be reconstructed from current balance alone.

PIV-owned metadata earns no staking rewards. Its exact original rent returns to
operations and proven additional lamports normalize to PendingSol. Config pending
recognition retains the active-round offset. Exact protocol/custody/Clock checks
and all required borrows precede final Config/round commit plus metadata zeroing
and lamport transfers; no fallible work follows the first state mutation. Actual
runtime purge/rollback and compute/heap remain separate integration obligations.

## Pinned source and observation rationale

Retained Stake 5.1 `process_withdraw` authenticates the withdrawer and lockup,
computes effective stake with its V2 arithmetic and runtime Stake History, rejects
full withdrawal while effective stake is nonzero, and resizes data to zero on full
drain. The legacy five-account Withdraw instruction is bincode variant 4; its
Clock/History positions remain exact even though this pinned processor obtains
the current sysvars through runtime calls. Canonical IDs/owners, Clock bytes and
bounded history geometry are authenticated and preserved by PIV.

The retained locked Agave `solana-runtime` 4.2.0 archive independently authenticates
`src/inflation_rewards/mod.rs` and
`src/bank/partitioned_epoch_rewards/distribution.rs`; receipt:
`/tmp/piv1-t248-pilot-review/retained-reward-source.json`. Rewards increase native
balance and the delegated state; optional rent adjustment can reduce delegation
without removing native lamports. For example, original rent 10/delegation 100,
reward 20 and new rent 20 can produce balance 130/delegation 110. The apparent
10-lamport difference is not proof of a donation. This source finding rejected an
unconditional donation-splitting design before implementation. The bounded balance
rule preserves the accepted full-receipt formula without inventing rent history.
No deployed feature activation or cluster binary is attested here.

## Review and evidence

One delegated writer and one separate reviewer. Root owns shared docs, test/build
execution and normal integration-only publication. Pinned-source CPI safety and
bounded independent specification review apply; no unavailable skill workflow is
claimed. Evidence root `/tmp/piv1-t248-pilot-review` preserves 357 baseline tracked
file hashes and 266 prior records, including both Task 2.47 attempts. Four recovery
archives/restorations remain protected. Root executed the validation below.

## Validation and preserved failure history

The initial focused run compiled successfully, passed nine tests and failed one
boundary expectation: setting metadata native balance to u64::MAX alone need not
overflow because its original rent goes to operations and only the remainder goes
to pending. Root, writer and reviewer independently identified the incorrect test
assumption. The initial review had missed it. The production source did not change.
The reviewed correction retains the original case as full-state success, and adds
a truly overflowing destination with consistent pending ledger/custody plus an
independent u128 inequality. No assertion, security guard or required gate weakened.
`initial-source/`, `source-freeze.json`, `host-focused-a/` and
`initial-attempt-records.json` retain all 122 original inputs and 137 initial records.

Final `validate-b.py freeze`, `focused`, `final` and `sbf`, plus
`check_evidence-b.py host`/`sbf`, passed against the corrected freeze. Ten focused
tests exercise 24 paired reward/metadata-donation combinations, complete independent
account/state and conservation oracles, original/current rent, native loss/residual
HWM recovery, partial target and both-order two-leg finalization, replay/refund
revival, strict ABI/identity/authority/Clock/history, all modeled CPI failure/noop/
corruption paths, late borrow failure and checked boundaries. The full suite passed
571 host tests and one doctest; eight final gates include all-target/features,
no-entrypoint, cpi, idl-build and warning-denied docs. The first SBF compilation
passed with no diagnostics. No earlier VM/Bank suites were rerun.

Final source freeze SHA-256 `189bacb034fcaaf3e658fbab9828a7ca9d9b1f448490fff0b0b8b2cb2fce55af` (122 inputs).
Final ELF: `/tmp/piv1-bank-smoke-withdrawal-finalization-sbf-t248-b/artifacts/piv1.so`; 758960 bytes;
SHA-256 `c1b7ae073af55d69dacd2a66ff2201b468d161e3c5da8fdf251481ef63ef3419`.
Raw ELF inspection confirms ELF64 little endian, ET_DYN, machine 263, flags zero
and the same 15 imports as Task 2.47. All 18 host and 58 SBF logs match their
receipts. Build elapsed 314.289s; sampled peak group
RSS+swap 606642176 bytes; sampled minimum
free disk 49155792896 bytes. These describe compilation,
not runtime heap/CU, effective-stake correctness or transaction rollback.

Initial baseline collection compared working inputs after the authorized writer
began and stopped on those edits. Corrected baseline hashes use verified HEAD;
all 119 prior inputs match. This read-only orchestration failure was not a failed
test/build. No unexpected changes were discarded. The 266 preserved historical
records and four recovery archives/restorations remain intact.

## Production boundary and remaining work

PIV1FL01 version 1 accepts exactly 17 bytes with a u64 leg index. Sequence derives
from the authenticated active round. Its 24/23 accounts append canonical Clock,
Stake program, Stake History, metadata PDA and Stake PDA to the 19/18 bootstrap
roles. Every outer key/backing identity is distinct. Fixed escrow/authority signer
seeds, exact five-meta Stake Withdraw and System rent transfer are independently
asserted. Config/round and metadata closure commit only after all current custody/
protocol checks; emitted facts require transaction success. Zeroed metadata retains
its allocation in the instruction; actual account purge is a runtime obligation.

Changed paths are seven shared guidance/report files and eight writer-owned code,
test/fixture and program README files. No manifest, lock, dependency version,
economic rule or persisted account layout changed. Existing persistence remains
typed; the closure helper adds no generic mutation bypass. Current Stake balance
with ambiguous donation/rent-adjustment provenance remains unsupported. Active or
partially inactive rejection is modeled as a trusted callee error, not executed
against the actual pinned Stake runtime. Earlier masked deficits remain unknowable
from present fields. Host callbacks never prove Bank atomicity, signatures or
live readiness. M4 must execute real nested CPI and failure/retry/heap/CU paths.

Next implement production settlement and pending integration, then the full local
lifecycle and exact Testnet package. Root owns indexed review and normal integration
commit/push; Git and `publication.json` record the resulting identity and clean
refs. Main remains 1054ff3 and protected Task 2.3 remains 3677fee. No secrets/signing,
key creation, deployment, live RPC, funds or authority transfer occurred. Founder
acceptance and explicit first-live-deployment authorization remain separate.
Save this bounded checkpoint and stop; resume from actual refs/worktree.
