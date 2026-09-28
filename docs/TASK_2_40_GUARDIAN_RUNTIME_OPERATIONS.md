# Task 2.40 — Guardian activity and emergency-pause runtime operations

Status: **TECHNICALLY VALIDATED / PENDING FOUNDER ACCEPTANCE** under D-026/D-030.
This completes the first bounded implementation block of M2; M2 remains in progress.
Starting integration/main: `8912cfeffcd83fa12cc1a68937a9de8dd5c6b091`.
Root verified jerem, clean worktree and matching local/remote refs before changes.
Main remains at that commit; this work has integration-only publication authority.

## Scope and canonical basis

Expose two existing requirements through the production instruction boundary:
K-010/master §11.2 guardian-signed current-period activity, and K-002/G-003–G-005/
master §12.3 explicit multisig-authorized pause/unpause. These close real lifecycle
prerequisites without requiring the M3 stake-pool adapter. No confirmed economic
rule, serialized layout, dependency or custody mechanism is changed.

Activity-only heartbeat records the current period even during pause; the pause
rules block economic transitions, while this instruction creates no reward and
changes no distribution snapshot. This is composition of existing requirements,
not a new claim exception or permission to execute paused economic operations.
During this task the founder resolved D-023/D-024's open contribution policy:
D-032 blocks explicit SOL/JitoSOL deposits while paused; direct incoming value
remains reconcilable. Those contribution handlers are outside this implementation.
No vote activity is inferred from governance execution time.

One delegated writer implements source/tests; a separate reviewer checks source,
regressions and evidence. Root owns canonical documents, execution and Git.

## Implemented production boundary

| Operation | Strict data | Fixed accounts | Only persistent change |
| --- | --- | --- | --- |
| Guardian heartbeat | `PIV1HB01`, version 1, guardian slot, governance vault-index witness; 11 bytes | Config, registry, six current rewards, Clock, guardian signer, PIV1 Program, ProgramData, Squads multisig (13) | Selected reward's `last_active_period` |
| Set pause | `PIV1PS01`, version 1, governance vault-index witness, strict Boolean; 11 bytes | Existing fixed Squads execution topology (16) | Config `paused` |

Heartbeat authenticates the initialized current registry/reward bindings, signer,
fresh Clock/Rent and current Squads authority/member correspondence. The same
period is idempotent; earlier activity periods reject. It preserves all earned,
claimed and claimable balances, historical liabilities and frozen snapshots.

Set-pause authenticates a complete current Squads invocation and exact original
approved data/account privileges before setting the explicit Boolean. It never
toggles. Same-value requests are idempotent, but still require authentication.
Outer Squads proposal consumption supplies governance replay protection; byte
equality alone is not a replay receipt. Both operations use existing typed atomic
state persistence and factual success events. No new CPI or lamport movement is
part of this block. Ordinary host entry rejects execution; explicit host context
seams are modeling only and are absent from Solana builds.

## Root execution and separate review

The delegated writer executed no tests or builds. Root reviewed the implementation
and independently executed the following against 96 frozen Rust/manifest inputs:

| Gate | Actual result |
| --- | --- |
| Focused `guardian_operations` target | 10 tests passed, first execution |
| Workspace all-target tests | 489 passed, zero failed/ignored, first execution |
| Doctests | One passed, first execution |
| Checks: all targets, all features, no-entrypoint, cpi, idl-build | All five passed |
| Warning-denied workspace documentation | Passed |
| Strict production SBF compilation/static inspection | Passed first build, zero diagnostics |

The eight final host gates are workspace tests, doctests, the five check variants
and documentation. No diagnostic was emitted. Source, tests and command review
passed separately before execution. Pre-execution review corrected an array API
call and two test oracles (pre-anchor timestamp error 6037; Squads Executed status
5), without relaxing any guard. These were source-review corrections, not failed
runs. Small validation-driver binding/finally/cwd corrections also preceded runs.

The ten grouped regressions independently assert literal ABI and state envelopes,
all six guardian slots under both pause values, exact full-account/lamport
preservation, nonzero claims, global historical liability exceeding the six
current records, a frozen EscrowFunded activity snapshot, half-open periods,
idempotence/regression, modeled off-curve guardian signer privilege without invented wallet-owner
rules, invalid account/authority/Clock/rent/borrow/alias conditions, exact approved
Boolean/witness payload, consumed/stale/insufficient proposals and event ordering.
The round fixture validates the accepted pure transition; it is not a real pool
or custody proof. End-to-end pause coverage across every lifecycle phase remains
part of M4. No activity is inferred from vote execution time.

Root commands, with explicit clean host environments and locked/offline one-job
Cargo arguments, are recorded in the log receipts:

```text
python3 /tmp/piv1-t240-pilot-review/validate.py focused
python3 /tmp/piv1-t240-pilot-review/validate.py final
python3 /tmp/piv1-t240-pilot-review/validate.py sbf
```

Driver SHA-256: `2f32554127435ee9e6b247b7a30a05f2be826c8524da4d9863df9c8e167c327b`.
Source-freeze JSON SHA-256:
`f0df5aefb4f3db7242f79e4d4716c65e76507176ddbd57d1ef94f4eb2566809b`.
Evidence: `/tmp/piv1-t240-pilot-review/host-focused-a`, `host-final-a`, baseline,
source-freeze/review and preservation receipts in that parent directory.

The fresh SBF build output is `/tmp/piv1-bank-smoke-guardian-sbf-t240-a`.
The build passed in 306.41 seconds; all 58 raw logs are hash-verified. The unchanged
resource guard recorded 610,451,456 bytes sampled RSS+swap and minimum free space
3,225,153,536 bytes. The final ELF64/SBPF-v0 artifact is 462,912 bytes, machine 263,
entrypoint 306,448, with SHA-256:
`78f94aeca0b5eb4f4a89a63b78423ec49ad80daaa10d3f89f7431eb56514443a`.
Immutable artifact: `artifacts/piv1.so` under that output directory. Result JSON
SHA-256: `fd99d7b79b045e9de975c829acb41bd2a7f9d5e7d3ac18546e4ffbc621480075`.
Root inspected raw ELF structure, diagnostics and log hashes; separate source,
command, evidence and artifact review passed. All preservation receipts are true.
The reviewer's initial import-inspection assertion assumed only `sol_*` names;
the pinned `solana-syscalls` 4.2.0 source also explicitly registers `abort`
(`src/lib.rs:389`). Correcting that inspection assumption required no production
change, compiler rerun or gate waiver. The driver's two generated Python bytecode
files were preserved outside the worktree under the root evidence directory;
they are not publication inputs.
It composes the unchanged hash-bound target and resource/socket guards with the
Task 2.39 verified installed libexpat hash; historical pins remain unchanged.
Only the new in-memory profile uses current source hashes. The SBF command uses
`env -i`, denies new external sockets, retains every diagnostic, and applies the
existing sampled 2.5-GiB process-group RSS+swap, 4-GiB per-process address-space,
2-GiB free-space reserve and 1,800-second build limit. These are sampled safeguards,
not true-peak or general filesystem sandbox claims. Host gates use the historical
host workflow, not these SBF resource limits. No install or cleanup occurred.

222 of the prior 231 source inputs remain byte-identical; nine intentional changes
are the program README and eight existing boundary/event/marker files. Two Rust
files are added. Root retains 20 historical artifact/evidence records, including
the Task 2.39 real initializer ELFs and copied Bank executable. Economic state/math,
serialized layouts, dependencies/locks, old fixtures, validation harnesses/tools,
all historical executable copies and Task 2.35's recovery archive are preserved.

## Evidence and operational limits

SBF compilation is static evidence, not execution of these new paths. Actual
Squads execution/consumption, public packet/signature transport, compute/heap and
new-path VM/Bank behavior remain to be established in the complete local lifecycle
work. Historical Task 2.39 initializer Bank evidence retains only its original
source/artifact scope; it does not execute this newly built artifact. Older Node,
SBF and Bank suites were not rerun. M2 remains incomplete; M3–M6 remain open.

Main stays at `8912cfe`; root publishes reviewed integration only and Git records
its exact commit. The checkpoint identifies the next bounded M2 dependency:
explicit SOL/JitoSOL contribution intake with D-032 pause rejection, actual transfer
and pending-custody checks, preserving active-round accounting. Recipient/guardian
updates, distribution handlers, the real adapter and full lifecycle remain open.
No deployment, Mainnet action, secrets/signing/key creation, fund movement or
authority transfer occurred. Technical validation is not founder acceptance or a
professional independent audit.
