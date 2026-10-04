# Task 2.50 — Production post-settlement pending integration

Status: **TECHNICALLY VALIDATED / PENDING FOUNDER ACCEPTANCE** (2026-10-04 UTC).

## Scope and authority

D-026/D-030 authorize this bounded M2 completion boundary on integration, following
Task 2.49 settlement. Starting HEAD is `3d31571df86e23150e89495566f4cadfcaa8c07c`;
main stays `1054ff3751ca4169eab7e46d77452634b11fc695`. Actual user is jerem, clean
worktree and remote refs verified. Baseline captured before delegation: 365 tracked
files, 125 matching prior source inputs, 648 preserved records, 48,791,396,352 free
bytes. Evidence root: `/tmp/piv1-t250-pilot-review`.

Canonical authority: decisions A-004/D-018/D-025/D-027/D-030, Master Specification
integration boundary, accepted Task 2.3 custody derivation and transition. Economics,
layout and dependencies remain unchanged. One writer implements bounded production,
tests and program README; root maintains shared guidance and executes validation;
a separate reviewer inspects source/tests, command plans and final evidence.

## Required behavior

For a normalized Settled round, move remaining pending SOL P-U, all pending tokens
Q and the actual escrow remainder into principal custody. Derive contribution value
as full P plus floor(Q * current official pool SOL / stored pool supply), then add
that full value to settled HWM. Final historical SOL excludes new next-cycle yield;
final historical tokens equal observed principal custody. Require their current
book value to cover new HWM. Complete the existing summary and return the reusable
round to Idle atomically only after exact protected System/Token CPI receipts.

Authenticate runtime Program ID, fixed account identities and canonical PDA seeds,
strict selector/version/accounts, current Clock/Rent/pool/Mint and combined held
supply. Preserve native Token-account funding, original vault rent, operational
funding, earned KIF liabilities and collective carry. Do not fabricate mock adapter
revision/capacity or use early Mint burn-lag adjustment. Legacy public derivation
error order must remain unchanged; no new recovery policy is introduced.

## Implemented instruction and custody boundary

`PIV1IP01`, version 1, is exactly nine bytes with no arguments. Accounts use the
initial-bootstrap order: Config, ActiveDistribution, PendingSol, PrincipalSol,
OperationalSol, DistributionEscrow, KifSol, PrincipalJito, PendingJito, PivAuthority,
Mint, System program, legacy Token program, pinned stake-pool program, pool,
validator list, reserve Stake and manager fee account; a distinct referrer is
present only when configured. The authenticated Clock account follows, for exactly
20 accounts or 19 when manager and referrer share one account.

Config and round are writable; transfer source/destination custody must be writable
when the corresponding amount is nonzero. The caller chooses no recipient, amount,
valuation, authority or PDA bump. Current Clock timestamp/epoch/slot cannot precede
the preparation snapshot. Pool accounting is current in that epoch, aggregate held
tokens cannot exceed Mint supply, and Mint supply cannot exceed recorded pool
supply. A legitimate direct burn retains the official stored-supply denominator.

The bounded transfer order is remaining pending SOL to principal, escrow remainder
to principal, then pending JitoSOL to principal via legacy `TransferChecked`.
Zero amounts issue no CPI. Full account metadata, native balances and data hashes
are checked after every CPI; no unrelated account change is permitted. Both Token
native balances remain quarantined. The external Mint is authenticated, read-only
and unchanged; this boundary does not impose a new rent requirement on that external
account. PIV custody and state accounts retain their current required rent floors.

A narrow internal valuation callback reuses the accepted derivation without
fabricating mock pool revision/capacity. Its public legacy wrapper keeps valuation
calls and errors in their original positions. New narrowly named integration
accessors preserve the old strict native-Token-excess rejection. Fresh staged Idle
custody and HWM coverage are checked before atomic Config/round byte persistence;
the completion event is emitted only afterward. Recovery rejects this boundary;
underprotected integration fails without inventing an HWM reset or new recovery rule.

## Validation plan and limits

Run focused custody/state/byte oracles and finite composition matrices, pause,
replay, malformed accounts/Clock/pool, HWM/cooldown/offset/overflow guards, every-CPI
fault and final two-account borrow conflicts. Then locked/offline host final gates
and strict SBF compilation, with immutable source freeze and historical evidence
checks. Root executed the final gates recorded below. Host rollback/discard and signatures remain
models; real System/Token execution, rollback, instruction heap/CU and the complete
production lifecycle are M4 obligations. Preserve all four recovery archives and
prior failed attempts. No cleanup/install, keys/secrets/signing, deployment, live
RPC, funds or authority transfer is authorized by this scope.

## Next dependency

Close the complete local lifecycle through actual production paths and necessary
pinned adapter components, then prepare the exact Testnet artifact/identity/funding
and founder workflow. Stop before live deployment for the exact package approval.
Main publication and founder acceptance remain separate. Save this bounded task's
reviewed integration checkpoint before ending the session to conserve credits.

## Review corrections and focused regressions

Before the first execution, source review corrected a canonical escrow-seed
identifier typo. Test review corrected the paused-field identifier, made a positive
remaining pending-SOL transfer explicit in the three-CPI fault/writability fixtures,
and asserted the exact legacy custody-error precedence. The stored-supply burn-lag
case now proves both aggregate custody feasibility and different results from the
incorrect Mint denominator. An outdated program README marker sentence was corrected.
These were pre-execution corrections. First focused, final host and strict SBF
executions passed without diagnostics; no test/build failed.

The ten focused groups cover 20 receiver/eligibility/liquid-withdrawn profiles,
normalized post-snapshot contributions and current ratio changes, no-transfer
completion with positive fully substituted P, genuine current-pool loss, supply
bounds and a valid SOL-only empty pool, pause/Clock/sequence/recovery/replay and
legacy precedence, deficits/surplus/rent/identity/counter overflow, all three CPI
faults/corruption with both late state borrows, alias/writability/early borrows,
and strict ABI/topology/production host guard. The 29 injected fault variants
explicitly discard failed modeled worlds before retry; no runtime rollback is
inferred from that discard. System and Token effects both use independent modeled
instruction/seed/account expectations. No pinned Token processor runs in this suite.

## Final source and execution evidence

Root ran the reviewed `validate.py` modes `freeze`, `focused`, `final`, `sbf`, and
both `check_evidence.py` modes. Final source passed 10 focused tests, 591 host tests +1 doctest/eight gates and strict SBF.
Eight host gates: all-target workspace tests, doctests, workspace check,
all-features, no-entrypoint, cpi, idl-build and warning-denied documentation.
No old VM/Bank suite was rerun. Callbacks model System effects and explicit failed
transaction discard; Token effects are modeled too; this suite does not execute the pinned Token processor.

Source freeze: 128 inputs, SHA-256 `9a05a1a0529195e6ee148a6109b65cdd09fd0632e92640da6c47c4526a647f62`.
ELF: `/tmp/piv1-bank-smoke-pending-integration-sbf-t250-a/artifacts/piv1.so` (841256 bytes), SHA-256
`fe3efd30424211963e1128888bfc982ad8cd9ca1fb9e31e288886a05ffe442b8`. Independent raw ELF inspection confirms ELF64 little endian,
ET_DYN, machine 263, flags zero, and the same 15 imports as Task 2.49.
All 18 host and 58 SBF logs match receipts. SBF build elapsed
291.835 seconds, sampled peak RSS+swap
605,806,592 bytes, sampled minimum free disk
48,508,813,312 bytes. These are build observations,
not instruction heap/CU or rollback measurements.

Exact source/tests, drivers, documentation and the 17-file integration tree receive
separate final review before normal publication. Git and `publication.json` record
actual commit, parent, tree, remote refs and clean worktree. Main stays 1054ff3;
protected Task 2.3 stays 3677fee. Preserve all 648 historical records,
four recovery archives/restorations and the new ELF. Technical validation is not
founder acceptance or live deployment authorization.
