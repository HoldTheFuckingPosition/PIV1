# Task 2.45 — Production liquid distribution preparation

Status: **TECHNICALLY VALIDATED / PENDING FOUNDER ACCEPTANCE**, D-026/D-030 M2. Starting integration:
`a18f33a2b5e9cd38ea1f6413d3e4e2e7d74a810c`; main remains
`8912cfeffcd83fa12cc1a68937a9de8dd5c6b091`. Root verified actual local/remote refs,
jerem and clean single worktree before edits. No founder acceptance is implied.

## Scope and required invariants

Expose the accepted no-yield and fully liquid-funded preparation transitions
through production. Actual withdrawal preparation has a demonstrated M3 dependency:
current Stake Program technical minimum, exact pool source residual constraints
and conservative aggregate multileg fee/floor/residual-HWM proofs. Existing mock
source/minimum/capacity values cannot establish those facts. Any positive native
shortfall must fail closed before CPI, state or cooldown updates; no fabricated
valid-insufficient attempt. This is a direct production lifecycle branch, not a
validation-only detour or completion of all preparation paths.

Implemented ABI: `PIV1PD01`, version 1, exactly nine bytes with no numeric arguments.
Fixed 27/26 roles extend the bootstrap 19/18 profile with the current registry,
six ordered guardian rewards and canonical Clock. Full pinned pool identity,
current epoch and held principal+pending units <= Mint supply <= stored supply
bind valuation. Preserve the official stored denominator through direct-burn lag.
Runtime Rent and Clock authenticate execution; guardian snapshot uses the current
PIV registry/revision, without inventing a new governance-sync transition.

Require each economic vault to match its bound obligations before preparation.
Compute historical SOL plus floor(historical units * stored total / stored supply),
then accepted gross yield including prior carry over HWM. Zero-yield validates
pause/Idle/cadence but leaves all bytes and timing unchanged. Positive outgoing
obligations use pending SOL first, then recorded next-cycle native carry; historical
SOL principal is never withdrawn. Preprove historical value plus unused carry
covers the proposed HWM. Store raw authenticated withdrawal fee observations,
zero conversion dust and exact current guardian/KIF snapshots.

Stage the accepted transition, predicted custody and both state envelopes before
up to two signed System transfers into escrow. Check exact full account metadata,
lamports and data after each CPI; hash borrowed buffers to bound large protocol
account memory. Commit Config and round atomically only after final authentication.
HWM is proposed until settlement; preserve pending contribution ledgers and explain
the physical debit through active-round offsets. Preserve all unrelated liabilities,
KIF, rent, operational funding and native Token quarantine. Legacy pure APIs,
strict accessor, account layouts, dependencies and accepted economics remain.

One delegated writer owns source/tests/program README; separate review covers
source, tests, driver and evidence. Root owns shared documents, all test/build
executions and reviewed integration publication. Require exact independent custody
and state oracles, both account topologies, pending/carry/mixed funding, no-yield,
HWM/cadence/Clock/KIF boundaries, stale/invalid pool facts, deficits/surplus, pause,
alias/borrow/overflow, replay, per-CPI corruption and partial-failure discard/retry.
Host callback evidence cannot prove runtime signatures, compute/heap, System CPI
execution or Bank rollback. Full production lifecycle remains M4.

## Takeover and validation evidence

Evidence directory: `/tmp/piv1-t245-pilot-review`. Root hash-verified 246 prior
inputs, 40 preserved records, four helpers and 76 retained Task 2.44 logs. These
are preservation evidence, not prior-suite reruns. Available capacity was
31,933,317,120 bytes; preserve all four recovery archives and restored paths.
No cleanup, restoration or installation needed.

The driver differs from Task 2.44 only by task/base/path/focused-target substitutions;
separate review passed. SHA-256:
`d55dc59cd7c0d08a00b01ed76e13c9979f62e74d110ed514541077b2b1b5a1bb`.
Focused target: `distribution_preparation_execution`. Root froze 112 reviewed source inputs (SHA-256
`8ddf57f98c00b5690c85e5c7bdda10d7bcfa7e71b5ef4e4ca16adb79afa4d112`).
Ten focused tests and 538 host tests plus one doctest/eight gates passed on first
execution, without diagnostics. Strict SBF passed on first execution with unchanged guards.
No execution-driven correction has been needed.

## Actual host validation

Root ran `validate.py freeze`, `focused` and `final`. The pinned Rust 1.97.1
commands use `--locked --offline --jobs 1`, unchanged host target, clean environment
and `CARGO_INCREMENTAL=0`. Final gates cover workspace/all-target tests, doctests,
default/all-feature checks, `no-entrypoint`, `cpi`, `idl-build` and warning-denied
documentation. Host execution is not covered by the SBF resource guard.
All ten focused tests, 538 workspace tests and one doctest passed with zero
failed/ignored tests and no diagnostics. All eighteen raw logs match their command
receipts; the source freeze, tool pins and forty preserved records match.
Separate host evidence review independently confirmed the results and hashes.
Host receipt SHA-256:
`f8f1a12507e4a322fdb298baf70059b31dd82b4cff2e953b4cb250b64349c175`.

One writer and a separate reviewer examined source/tests/driver before execution;
root independently inspected the implementation and literal-arithmetic, complete
Config/round/account oracles. Review requested data-corruption, interface, timing,
prior-summary and overflow cases before freezing; no source defect was found.
Neither delegated agent executed suites or builds. No rustfmt run is claimed.

The ten groups cover both 27/26-role topologies; pending-only, carry-only and mixed
funding; historical loss offset by carry; unused carry, earned KIF and zero-active
KIF separation; read-only no-yield and paired-empty pool; exact ten-day boundary
and previous insufficient cooldown independence; Clock/registry/KIF-period binding;
prior completed summary/replay; direct-burn supply lag and raw 0/0, 0/N and nonzero
withdrawal fee snapshots; all six custody surplus/deficit dimensions; native Token
and operations quarantine; overflow, pause, ABI/version/count, protocol-key,
writable, alias and hostile-borrow rejection. Four failure cases cover before/after
effects at both transfers, exact raw partial accounts, explicit modeled discard
and successful retry. Fifty-four lamport and ten data corruptions across the two
CPI positions reject before state commit. Tests model System effects, signatures
and transaction discard; they do not execute these new paths in VM/Bank.

## Remaining dependency and publication gate

Complete real withdrawal-target/minimum proofs and preparation, delayed legs,
settlement and post-settlement integration, then execute the full local production
lifecycle. No complete distribution, full adapter or Testnet readiness claim.
Root may publish only reviewed integration work by normal unsigned commit and
non-force push. Main, acceptance and live authorization remain separate. No secret
access, key creation, signing, funds, deployment or authority transfer is involved.

## Static artifact and final publication

The first strict SBF build passed without diagnostics. Root and separate review
verified `/tmp/piv1-bank-smoke-distribution-preparation-sbf-t245-a/artifacts/piv1.so`,
609,664 bytes, SHA-256 `ede63be229139c261f68b8005d2e4c84060ddd41de5babfafb1d1c3d003c1532`.
Result receipt SHA-256: `b0db8c4ba55627981552fbd739422fea8ad77c945f13179207b4e266f11027a8`.
The raw header is ELF64 little-endian ET_DYN, machine 263, flags 0, entry
431,048, with the same fourteen unresolved imports as
Task 2.44. The immutable copy equals the Cargo output; all 58 SBF logs match.

Compilation took 301.82 seconds, exit 0, no guard stop;
sampled peak group RSS+swap 608,108,544 bytes,
minimum free space 31,664,898,048 bytes. Unchanged guards:
2.5-GiB sampled RSS+swap, 4-GiB per-process address space, 2-GiB disk reserve,
1,800 seconds, network socket creation denied and local UNIX socketpair allowed.
These are compilation guards, not runtime resources. No VM/Bank execution.

All 112 frozen sources, 40 preserved records, helpers and tool pins match. No
source correction or repeated validation was needed after first execution. Old
pure APIs, account layouts, dependencies and economics are unchanged; the new
instruction, factual event and previously reserved custom code 6145 are explicit
ABI additions. Source/test/driver and execution-evidence reviews passed; exact
indexed-tree/document review is the final publication gate.

Root publishes only reviewed integration by normal unsigned commit and non-force
push. Git and `/tmp/piv1-t245-pilot-review/publication.json` record actual commit,
independent remote refs, clean worktree and preserved main/Task 2.3. No future
commit is assumed. Preserve all historical evidence and four recovery archives.
Save/end this bounded session. Technical validation is not founder acceptance,
main authority, complete lifecycle proof or live deployment authorization.
