# Task 2.43 — Production initial contribution bootstrap

Status: **TECHNICALLY VALIDATED / PENDING FOUNDER ACCEPTANCE** under D-026/D-030 M2. Starting integration is
`9856ec8de01ca5706296bf59e63f9396bce4ecbf`; main remains
`8912cfeffcd83fa12cc1a68937a9de8dd5c6b091`. Root verified jerem, clean single
worktree and matching remote refs. Technical validation is not founder acceptance.

## Scope and canonical requirements

Task 2.42 supplies real normalization before accepted Task 2.5 initial bootstrap.
This task connects that accepted transition to production custody and current
protocol-account facts. It is not general Idle or post-settlement integration:
those boundaries must preserve pending-first distribution priority and their
own active-cycle completion rules. Master §§7.3–7.8 and 8.1–8.2, P-014–P-016,
D-027 acceptance and the Task 2.5 report control the economics.

Require initialized, unpaused, bound Idle state with no prior economic/history,
completed/preparation/insufficient record, HWM, carry or KIF liability. Require
positive recognized pending SOL or JitoSOL; every economic vault must cover its
own obligation and have no unexplained surplus. Prior normalization is separate.
Move all recognized pending SOL to PrincipalSolQueue and all recognized pending
tokens to PrincipalJitoVault. Derive contribution value as SOL plus conservative
current official token value, then update only pending/historical/HWM/contribution
fields. Preserve the full round, next sequence, clocks, guardian data, liabilities,
rent and every unrelated byte. Positive zero-floor token principal prevents replay.
SOL remains in the principal queue; no staking/pool CPI is implied.

## Production composition and valuation

Strict `PIV1IB01` plus version 1 (nine bytes), with nineteen fixed roles or eighteen
when Config already declares the same manager/referrer receiver. The first thirteen
roles match normalization: Config, round, PendingSol, PrincipalSol, OperationalSol,
Escrow, KifSol, PrincipalJito, PendingJito, PivAuthority, Mint, System and legacy
Token. Append Jito program, pool, validator list, reserve, manager fee receiver and,
only when distinct, referrer. No caller amount, value, topology switch or signature.

Use runtime Rent/Clock and existing full Jito identity authentication. Preserve the
public `bootstrap_initial_contributions(PoolSnapshot)` API, validations and error
order. Factor its unchanged state body into a crate-private core whose value
callback runs at the former book-value line. The runtime callback derives its
value only from fresh authenticated pool/Mint facts and Clock; no externally
chosen value, revision, capacity, liquidity or model-fee mapping is introduced.

The source-pinned SPL 2.0.3 `calc_lamports_withdraw_amount` (state.rs 178–189)
uses floor(units × recorded total lamports / recorded pool-token supply).
`UpdateStakePoolBalance` (processor.rs 2140–2260, especially 2254–2257) refreshes
recorded supply from the Mint. Permissionless direct burns can leave a legitimate
Mint supply below recorded supply within the same epoch. Require held units ≤
Mint supply ≤ recorded supply and use the recorded denominator. Reject stale or
future pool epoch, inverted supply, held units above Mint supply and the accepted
one-sided empty profile. Both-zero pool facts support only zero token holdings;
SOL-only bootstrap remains possible. Fees are not charged by same-asset movement.
This proves no validator liquidity, deployed binary or cluster support.

Root verified pinned upstream revision `864ba3c1c564cc270ca62b6e6b558f57538ae092`
against retained Task 2.19 provenance: state.rs SHA-256
`64e9fde6944c036678eba10ab5ddd20d0b2b287b97222d5698c94960c20e6502`,
processor.rs SHA-256
`55fe3619f9eb786b89b349ba17956ef13070f1c059646f21d109f25e7736eefc`.
No new dependency or upstream execution is needed for this narrow ratio mapping.

Stage final Config and all checked destination credits before effects, preflight
all later borrows, then perform up to two signed nonzero transfers: pending SOL
using its own PDA seeds, followed by Token TransferChecked using shared authority
seeds. Verify every account after each CPI. After movement, old Config temporarily
expects the pending assets that just moved: reauthenticate fixed identities/schema/
floors without an old-Config coverage accessor. Exact predicted transfers and the
accepted transition already prove final Config obligations. Commit Config/event
only after both movements and all postconditions. Errors propagate; raw host
callback effects require explicit modeled discard and are not Bank rollback.

## Token-native compatibility and remaining limits

A bootstrap-only observation contract requires rent and economic obligation
coverage while quarantining both full Token native balances. The handler must
preserve those lamports after every CPI and exclude them from contribution value,
HWM and rent funding. Operational funding likewise stays untouched/unclassified.
The old strict accessor and normalization-only accessor remain unchanged. This
extends the initial-bootstrap composition profile only; it does not extract
Token-owned SOL, classify operational provenance or resolve later lifecycle use.
No close/recreate, direct Token-lamport debit, economic or serialized-state change.

## Coordination and validation

One delegated writer owns code/tests/program README; a separate reviewer checks
design/source/tests and evidence. Root owns shared docs, actual execution and Git.
Tests must independently cover nineteen/eighteen-role SOL/token/mixed success,
appreciation, zero-floor replay, current-epoch/supply edge cases and official
integer-ratio correspondence; all accepted history/state/custody restrictions;
Token-native/operational preservation; exact signed CPI bytes and seeds with the
pinned Token Processor; pre-effect alias/borrow/rent/arithmetic failure; full raw
prefix accounts for each late failure/corruption, followed by explicit retry.
Legacy PoolSnapshot validation must remain unchanged, including unused model fields.

Evidence: `/tmp/piv1-t243-pilot-review`. The baseline binds 238 prior inputs and
38 preserved records, including all four archive metadata sets and retained
Task 2.41/2.42 restorations. Task 2.42's 76 logs are retained/hash-verified, not rerun.
Observed free capacity is now about 32.2 GB; no cleanup occurred or cause is inferred.
The driver is the unchanged Task 2.42 workflow with baseline, paths and focused
target substituted; SHA-256
`89c567ee0b16670eb9005ee16ed831a041e3df1e96eb67df3f06914fe09f781e`.
Locked/offline one-job host gates keep `CARGO_INCREMENTAL=0`; the fresh SBF build
keeps the original network/diagnostic/resource limits. Source/test/driver review
passed. Root passed ten focused tests and 518 full host tests plus one doctest,
with all eight gates passing on the first execution and no diagnostics. Separate
review rehashed all eighteen logs and the 105-input freeze (SHA-256
`10b9449fb24d19d2346d70f84887d9a4004b724ef973efc67769a305b4a756b3`).
Strict SBF also passed on its first execution, without diagnostics. The artifact
`/tmp/piv1-bank-smoke-bootstrap-sbf-t243-a/artifacts/piv1.so` is 550,064 bytes,
SHA-256 `88bb9166fdb3f291a1bd3c5611b49b602051c8a8aacc5d4f2b9fd97864936fc2`.
Result receipt SHA-256:
`38bc0a8a64e09ac8b59b781e8791f7545390eb245f7b47c15ccc873a26ed2cf1`.
All 58 SBF logs match. Raw ELF64/little-endian ET_DYN, machine 263, flags 0 and
entry 378,536 match the static inspection; fourteen unresolved imports are the
same as Task 2.42. The 318.21-second build had no guard stop. Sampled peak group
RSS+swap was 591,351,808 bytes and minimum free space 31,950,790,656 bytes.
Limits remained 2.5 GiB sampled RSS+swap, 4 GiB per-process address space,
2-GiB disk reserve, 1,800 seconds and network socket denial with local UNIX
socketpair allowed. These are build limits, not runtime handler resource proof.
No new-path VM/Bank proof or full production lifecycle completion is implied.

Root executed `python3 -B /tmp/piv1-t243-pilot-review/validate.py` with modes
`freeze`, `focused`, `final`, then `sbf`. Host commands use pinned Rust 1.97.1 Cargo
with `--locked --offline --jobs 1`; the focused target is
`test -p piv1 --test initial_bootstrap_execution`. Final gates cover workspace
all-target tests, doctests, default/all-feature checks, `no-entrypoint`, `cpi`,
`idl-build` and warning-denied documentation. The writer and reviewer executed
no tests/builds. No compile/test failure, source correction after execution,
dependency change, installation or capacity recovery was required.

Changed files: new production handler, ABI, focused test and protocol fixture;
bootstrap composition core, fixed-account accessor, dispatch/exports, existing
error-mapper visibility and factual event; program README; and shared guidance,
checkpoint, plans, specification, README and this report. No serialized state or
error number changed. All 38 protected records and upstream source pins remain
intact. Root publishes the separately reviewed exact integration tree; Git and
`/tmp/piv1-t243-pilot-review/publication.json` record its commit, final clean
worktree and unchanged protected main/Task 2.3 refs.

Normal reviewed publication is integration-only. Main, founder acceptance and
exact live-Testnet approval remain separate. No secrets access, key creation,
signing, deployment, Mainnet action, fund movement or authority transfer occurred.

## Next concrete dependency

After this initial bootstrap, SOL remains queued in principal. The next bounded
production gap is protected principal SOL staking under the accepted Task 2.6
rules. That composition requires a real pinned SPL deposit and authenticated
before/after pool facts; it cannot safely use fabricated model revision/capacity
fields. Scope the necessary M3 deposit component together with its M2 handler,
as D-030 permits a demonstrated adapter dependency. Preserve the accepted zero-fee,
book-value/HWM, slippage and pause guards; general fee/rounding-loss support stays
open. Post-settlement integration and the remaining distribution lifecycle still
need their own production paths. No new economic decision is inferred.
