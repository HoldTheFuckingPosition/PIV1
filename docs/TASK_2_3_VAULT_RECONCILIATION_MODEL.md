# Task 2.3 vault reconciliation and custody composition model

Date: 2026-09-08 UTC

Starting baseline: `66193769d1cbc59cd8630df295b9a784b9c64642`

Branch: `task/2.3-vault-reconciliation-model`

Status: **IN PROGRESS / AUTHORIZED**, not founder-accepted.

## Sources and implementation boundaries

The founder authorized this bounded host-only task in the current session.
Decisions P-014--P-016 preserve all contribution value through liquid funding;
P-033--P-035 isolate rent and next-cycle rewards/loss; A-005 fixes cumulative
leg accounting; K-012 isolates earned claims; D-024 accepts pending recognition
but explicitly defers custody composition. Master sections 6.2, 7.5--7.7,
9.6, 9.10--9.12 and 10.2, plus Phase 0 section 5.2, govern normalization.
The positive unexplained economic-vault delta rule is already CONFIRMED:
pending contribution, never historical yield. Safe recognition during an
active round is distinct from integration at the accepted completion boundary.

Planned normal commits (linear history, no publication or acceptance):

1. Record these equations and commit boundaries before implementation.
2. Implement pure derivations, atomic host composition, regressions and report;
   validate and commit the bounded result for founder review.

## Equations and physical locations

All SOL equations exclude fixed non-economic account floors. JitoSOL equations
use decoded token units only; token-account rent is separate.
Let P be recognized pending SOL, U be the active round's pending SOL use,
Hsol/Hj be config historical amounts, A be cumulative assigned token input,
C be prior next-cycle yield, Cu its pending-first liquid use, R be new cooldown
rewards, Z be settled zero-active KIF compound, and E be current escrow.

```text
pending SOL obligation = P - U (active); P (Idle)
pending JitoSOL obligation = recognized pending token units (no offset)
principal JitoSOL obligation = Hj - A (active); Hj (Idle)
A = cumulative fee units + cumulative burned units
principal SOL obligation = Hsol + C - Cu + Z (active; Z=0 before settlement)
principal SOL obligation = Hsol + config.next_cycle_yield (Idle)
E before settlement = U + Cu + finalized native - recovered stake rent
E after settlement = recorded escrow - actual paid HTFP - actual paid Team
                   - actual allocation moved to KIF
                 = net allocation dust + retained conservative dust + R
KIF SOL obligation = recorded global earned liability + collective KIF carry
```

Prior carry resides in PrincipalSolQueue separately from historical SOL. Opening
moves only Cu to escrow and stores C in the round before clearing config carry.
The unused C-Cu stays in principal custody and becomes historical at completion.
New R resides in escrow until integration moves it to PrincipalSolQueue while
retaining its next-cycle-yield category. Settlement moves net KIF to KifSolVault
and Z from KIF to PrincipalSolQueue. Old earned claims remain independent.

Integration moves only P-U native SOL, all pending JitoSOL units, and the
settled escrow remainder into principal custody. Its contribution HWM component
is P plus the conservative current value of pending tokens, once. New historical
SOL is observed principal SOL after these moves minus new R; new historical
token units are observed principal units after the pending-token movement.
No historical value or HWM proof is accepted from a test-selected final state.

## Atomic commit and trust boundary

Each host operation clones the complete world, stages source debits and exact
destination credits, derives transition facts from observations, executes the
pure transition, verifies custody and independent conservation, and commits once.
Errors discard the clone. A successful finalization returning RecoveryRequired
commits the recovered custody and recovery state. A settlement recovery outcome
commits only the accepted recovery header and no speculative payments.

Schema, sequence, snapshot, historical counters, pending snapshot, HWM and carry
relationships must validate before any active offset is used. Idle never uses a
completed summary as an offset. The host constructor starts at Idle; later
committed states are reached only by these atomic operations. Pure scalar
helpers cannot authenticate a historical transfer. Future fixed-account, owner,
PDA, mint, signer, Rent, Clock, exact transfer and CPI validation remain deferred.

The reserve baseline is deliberately absent from production derivation:
serialized state does not record initial operational funding and all outstanding
advances. Test-only initial reserve and transaction records can independently
prove rent conservation, but cannot identify arbitrary operational excess in
production. Unsupported operational surplus remains unclassified/unspendable;
this does not block proven economic-vault normalization. A current balance
cannot reveal an earlier unobserved debit replenished by indistinguishable credit.

## Validation and final result

Implementation evidence, precise supported/unsupported matrix, test coverage,
commands, compatibility checks, and final status will be recorded here after
implementation. No blockchain, wallet, key, handler, CPI, or deployment work is
authorized. This is AI-assisted engineering, not a professional independent audit.
