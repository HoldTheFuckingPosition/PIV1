# Task 2.46 — Production withdrawal preparation with an active source witness

Status: **TECHNICALLY VALIDATED / PENDING FOUNDER ACCEPTANCE**, D-026/D-030 M2 with the required bounded M3 proof component.
Start integration: `17bc0760390034157a932e51de674f976d114922`; main remains
`8912cfeffcd83fa12cc1a68937a9de8dd5c6b091`. Root verified actual user, clean
single worktree and matching remote refs before edits. No acceptance is inferred.

## Scope and canonical rules

Expose real withdrawal preparation with one authenticated standard active-validator
witness. A distinct `PIV1PW01` version-1 profile supplies only a checked validator
record index, not an amount, minimum, capacity or caller price. Extend the current
27/26 custody/pool/guardian roles by canonical Stake program and candidate source.
Preserve the existing `PIV1PD01` liquid/no-yield ABI. Preferred selection supports
the exact configured vote; fallback, transient, reserve and removal profiles stay
explicitly unsupported. This closes a real preparation dependency, not a
validation-only milestone or complete protected withdrawal implementation.

Use accepted Phase 0 sections 9.3/10.1/11.2/12.1, P-013/P-020/P-023/P-026 and
A-001/A-002/A-005. Pending SOL funds first, then prior native carry. For positive
remaining budget B, pool total T and recorded supply S, derive the fixed input:
`q = min(historical_units, floor((((B + 1) * S) - 1) / T))`, checked in u128.
Derive the least fee-net input redeeming at least runtime minimum D and the
candidate's greatest safe input through bounded monotone searches. Apply exact
per-call ceiling fee and native floor, not a one-call fiction for multiple legs.
The maximum useful-leg bound is target divided by the snapshot technical floor.
Reserve up to n−1 extra fee units for nonzero fees and n−1 native floor lamports,
then apply Config's immutable 0–1 bps slippage to the conservative aggregate floor.
All subtractions and stored conversions remain checked and the native floor
must be positive. Conversion dust remains protected; reject if actual residual
historical value plus unused carry cannot cover proposed HWM, including the
one-lamport partition-floor case. Do not silently reduce that HWM/dust proof.

Authenticate an actual canonical Stake GetMinimumDelegation CPI response: exact
returning program, eight little-endian bytes and positive minimum. This query has
no economic effects; compare every full account before/after it. Unchanged decoded
snapshots remain valid; decode source facts after the query.
No caller assertion is a runtime minimum. Require current Rent for a 200-byte stake
account and the existing bounded WithdrawalLeg allocation, funded independently
from operational spendable SOL. Do not debit that rent during preparation.

Read one checked 73-byte current Active validator record, with no full-list scan.
Bind its vote, epoch, seed suffix and active balance to the exact derived source
PDA and typed Stake state, pool withdraw authority and default lockup that is not in force at the current Clock. A supplied
record above the pinned SPL residual+tolerance witnesses active-source precedence.
Require exact configured preference when present. Candidate maximum fill must
meet the greater technical floor and leave either zero or a viable remainder.
For the supported active/activating source, require positive delegated stake,
no deactivation, current epoch below u64::MAX, activation no later than current
(or the bootstrap sentinel), empty flags, source free lamports covering current
Rent, and split output bounded by both SPL residual and delegation minus D.
The later destination must be prefunded with current rent; no live split is proved.

Only after complete proofs may the accepted transition stage WithdrawalActive,
liquid funding movements and atomic final Config/round writes. A valid below-minimum
attempt updates only the separate 24-hour retry clock and emits its factual event after commit, never a snapshot, custody
or ten-day clock. Support zero computed target through a private compatible proof
path if needed; preserve public legacy API checks and error order. Invalid
accounts/runtime/source/rent/arithmetic and failed CPIs cannot mutate cooldown.
Freeze guardian/KIF, recipients, current pool fees and all prior liabilities exactly.
Native Token funding remains quarantined, contribution value remains fully recorded,
and HWM is proposed until settlement. All transfers get exact full-account checks.

## Source proof and dependency boundary

Accepted Agave 4.2.0 source at `ac82b5d438b0c2303dc7169f52c748977713a111`
embeds `core_bpf_stake-5.1.0.so`. Root fetched official Stake program tag 5.1.0's
commit `3b511b63093618bf12493728e416fc647cb19e5b`, including processor and exact
interface state, and verified file bytes against Git-tree blob identities. Source
receipts: `upstream-fetch.json`, retained tree/tag observations and eight files.
This establishes the selected source mapping, not installed/deployed binary
attestation, RPC feature activation or actual local nested execution.

Stake processor split lines 485–720 requires prefunded destination rent, residual
source rent and both delegation minima for the active/activating branch. Its
interface state lines 741/829 proves effective+activating equals positive delegated
stake under the selected guards regardless of history branches. Thus this bounded
predicate needs no supplied StakeHistory; it does not claim fully activated stake.
The deactivation path uses the checked stake transition; empty flags retain a
conservative compatibility boundary. Later actual leg execution must reauthenticate
all facts and verify the real CPI, not rely on preparation's point-in-time witness.

The retained SPL 2.0.3 archive hash is
`6f0db03f091f43b5766296e80088718491b50949cd3eb4cce3e0cfed58fe2c18`.
The existing Stake-interface 1.2.1 archive matches lock checksum
`5269e89fde216b4d7e1d1739cf5303f8398a1ff372a81232abbee80e554a838c`.
Nine source files match those archives without extraction/install. The historical
interface VCS commit is now unavailable from GitHub (404/422), so no unfetched
processor behavior was inferred from it. Promote the exact already-locked
Stake-interface 1.2.1 typed decoder/borsh feature to production; no version upgrade,
new package or broad dependency update. Record actual graph changes separately.

## Validation and publication plan

One delegated writer owns bounded source/tests/program README and the exact
production dependency edge. Separate review covers source/math/tests/driver/evidence;
root independently verifies findings, owns shared docs and executes all gates.
Require literal/reference-search math tests, monotonicity and extreme bounds,
per-leg partitions, both role topologies, exact dynamic return-data/CPI integrity,
source/status/epoch/PDA/preference/rent failures, stranding and HWM-dust boundaries,
valid insufficient/cooldown independence, old liquid regressions, complete state
and per-CPI custody oracles, partial failure/discard/retry and borrow/alias errors.
Host callbacks do not prove actual Stake/System execution or Bank rollback.

Evidence: `/tmp/piv1-t246-pilot-review`. Takeover verified 249 prior inputs,
41 preserved records, four helpers and 76 retained Task 2.45 logs (not reruns).
Free capacity: 31,653,212,160 bytes. Preserve all four recovery archives and their
restorations. No cleanup/install needed. The driver uses narrow task/base/path/
focused-target substitutions, SHA-256
`c298ee37fbc6093d78e0f169b9b94fa1e8f3df27769080b4b59eee5c46169dcf`.
Target: `withdrawal_preparation_execution`; eight host gates and strict SBF keep
the existing limits. A new return-data syscall import is expected and must be
explicitly checked. Root passed 10 focused tests, 551 host tests +1 doctest/eight
gates and strict SBF; see the evidence below.

Root publishes only separately reviewed integration by normal unsigned commit
and non-force push. Main, founder acceptance and sensitive live gates remain
separate. Next: real protected leg initiation/deactivation and finalization,
settlement and pending integration, followed by the complete M4 local lifecycle.
M2/full M3–M6 remain open. No secrets, key creation/signing, deployment, live funds
or authority transfer is authorized or performed in this task.

## Review correction and retained first execution

The initial reviewed 115-input freeze and ten focused host tests passed. Before
final gates or SBF, additional source review found a reachable negative-Clock
edge: default lockup timestamp zero is in force at a negative current timestamp,
even with an otherwise-valid negative KIF anchor. SPL later changes withdraw
authority without a custodian, so preparation must reject that source. The
original matching-future-lockup suggestion was already excluded by the existing
default-pool-lockup identity rule; only the negative-Clock edge required correction.

The narrow correction checks typed `lockup.is_in_force(clock, None)` and adds a
regression with default pool/source lockup, Clock -1 and KIF anchor -1000. No
economic or timestamp policy was changed. Initial `source-freeze.json`,
`host-focused-a` and preservation receipts remain untouched as prior passing
evidence, not evidence for the corrected source. Final `source-freeze-b.json` and
b outputs use driver SHA-256
`a0ba799075e4a6183bd411c4836fbf93f9f8cfd3ff7f4cb361a87140e7f37b1e`,
with only freeze/output/preservation names changed. Separate review passed the
correction and guards before root reran the focused suite and first executed the
final gates and SBF. Every execution passed without diagnostics; the correction
came from source review, not a failed test. The two original focused logs remain
separate from the 76 final host/SBF logs. Final freeze SHA-256:
`80fe075c06044daebb8431ca6fc9be62f25945d72f340c8ae4440e38fcbc07a4`.

## Executed evidence and remaining limits

Root executed the reviewed locked/offline driver. Focused suite: 10 tests;
full workspace: 551 tests plus one doctest; all eight final gates passed without
diagnostics. The source freeze contains 115 inputs; host evidence verifies all
18 logs and retained inputs/tools. Separate review inspected source,
math, independent full-state/event/custody oracles, command guards and evidence.

Strict SBF passed with 58 verified logs and unchanged resource guards.
Artifact: `/tmp/piv1-bank-smoke-withdrawal-preparation-sbf-t246-b/artifacts/piv1.so`, 633,960 bytes,
SHA-256 `e10e3c8947d847faebd155bc32d130f73c97e1ffb90576fb202cae472b39c6eb`. Exact static imports are the previous 14 plus
`sol_get_return_data`; ELF64 little-endian ET_DYN, machine 263, flags zero.
Compilation took 309.14 seconds, sampled peak RSS+swap
614,178,816 bytes, minimum free space
30,815,997,952 bytes and no guard stop. This is compilation/
static evidence only. No new-path VM/Bank, actual nested Stake/pool execution,
transaction rollback, compute-budget or runtime heap claim follows.

Receipts: `source-freeze-b.json`, `host-evidence.json`, `sbf-artifact-review.json`,
`dependency-edge.json`, `reviewer-final.json`, `staged.json` and `publication.json`
under the evidence root. Cargo.lock remains unchanged; the only manifest edge
change promotes exactly Stake-interface 1.2.1/borsh from dev to production.
No archive, historical artifact/log, helper, account layout or economic change.
No cleanup/install or live operation occurred. Git and the publication receipt
record the integration commit; do not infer an unpublished future hash.
