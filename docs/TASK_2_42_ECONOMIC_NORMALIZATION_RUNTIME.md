# Task 2.42 — Production economic-surplus normalization

Status: **TECHNICALLY VALIDATED / PENDING FOUNDER ACCEPTANCE**, bounded D-030 M2 production work under D-026.
Starting integration: `3a5c3ce9179975c2cfd7ba09d379459aaa53a8d0`.
Main remains `8912cfeffcd83fa12cc1a68937a9de8dd5c6b091`.
Root verified jerem (uid 1001), clean single worktree and matching remote refs.

## Concrete dependency and canonical scope

The founder-accepted Task 2.5 bootstrap requires normalized before-custody, with
no unexplained economic surplus. At takeover, production recognized only dedicated
pending custody; actual principal/escrow/KIF normalization existed only in the
host `World::normalize`. `record_economic_normalization` remains the unchanged
pure transition now composed by the production handler.
A direct native-vault or principal-token donation could therefore block principal
bootstrap without a production remedy. Separate design review agrees that exposing
normalization is the smallest coherent prerequisite before the bootstrap/value
bridge. This implements a real M2 handler, not a validation-only milestone.

Master specification §§7.7–7.8, D-024/D-025 and accepted Task 2.3 equations govern
all six economic dimensions. Each vault must cover its own complete obligation;
other surplus cannot hide a deficit. Move only proven principal SOL, escrow SOL,
KIF SOL and principal JitoSOL excess into the respective pending vaults. Recognize
existing pending excess in the same atomic record. Preserve active P-U and assigned
token offsets, rent floors, historical assets, snapshots, HWM, earned KIF/carry
and every non-pending field. Operational funding is a separate unchanged category.

The existing pure transition controls callability: movement rejects during pause
or RecoveryRequired, while no-movement recognition may succeed in those states.
D-032's explicit-deposit rejection and the older pending-recognition path remain
unchanged. No clock, pool quote, fee, slippage, valuation or depositor right is
introduced. This block does not bootstrap principal or integrate a settled round.

## Implemented production boundary

Strict `PIV1RB01` plus version 1 (nine bytes), with exactly thirteen accounts:
Config, active round, PendingSol, PrincipalSol, OperationalSol, DistributionEscrow,
KifSol, PrincipalJito, PendingJito, PivAuthority, configured Mint, canonical System
and canonical legacy Token program. No caller-supplied amount, account role,
backend or authority is accepted. The operation is permissionless.

Derive all obligations/surpluses, checked aggregate credits and the final Config
replacement before any effect. Config is writable; actual transfer participants
must be writable. Preflight all later data/lamport borrows before the first CPI.
In fixed order, execute up to three signed System transfers from principal,
escrow and KIF to PendingSol, then one signed `TransferChecked` from PrincipalJito
to PendingJito. Use each native source's canonical PDA seeds and the common Token
authority seeds. Do not invoke zero-value transfers.

Verify every supplied account's metadata, native balance and borrowed data hash
after each successful CPI, then reauthenticate before the single final Config
commit and factual event. Preserve raw Token option padding and every unrelated
byte. Errors propagate for transaction rollback; injected host callbacks do not
provide rollback automatically. Native host entry remains fail-closed.

## Explicit limits

A new normalization-only observation accessor can exclude native funding in the
Token accounts while requiring rent and all six supported economic obligations.
Those native lamports remain exactly unchanged and unclassified, as in Task 2.41
intake. This neither sweeps Token-owned SOL nor silently waives the old strict
`economic_observation` guard or bootstrap restrictions. Legacy Token native-excess
recovery remains unsupported; bootstrap tolerance of that category requires its
own later justified review. No close/recreate or direct debit of Token-owned
lamports is allowed here. Operational funding provenance remains unsupported.

The pool-accounting bridge is unnecessary for same-asset normalization and stays
with subsequent bootstrap work. No production mapping of the model's provisional
revision/capacity fields is invented. Existing economics, serialized state,
normalization transition, dependency versions and old instruction paths remain.
Current-path VM/Bank execution and the complete local lifecycle remain future
M4 evidence; host tests/static SBF do not establish signatures or chain rollback.

## Coordination and validation

One writer owns code/tests/program README; a separate reviewer checks design,
source/tests, commands and evidence. Root owns canonical docs, actual validation
and integration-only Git publication. Meaningful tests cover all six surplus and
deficit dimensions across genuine lifecycle states; exact signed CPI bytes/seeds;
pause/recovery movement versus no-movement; idempotent no-op; every-CPI postchecks;
late failure after earlier successful transfers and explicitly discarded host
staging; borrow/rent/alias/arithmetic rejection; preserved Token-native/operational
balances and the unchanged strict accessor guard.

Evidence directory: `/tmp/piv1-t242-pilot-review`. The baseline binds 235 prior
source inputs and 33 preserved records: 22 historical artifact/evidence records,
nine manifest/ready/journal records across the three recovery archives and both
manifest-verified Task 2.41 restored files.
Task 2.41's 76 successful host/SBF logs are hash-verified, not rerun.

Separately reviewed driver: `validate.py`, SHA-256
`bb11f37c83cdb8a6568b67c867591d7051818df67516a79ba59051c9d24769f9`.
It reuses the previous exact-source, locked/offline, one-job focused/full host
gates and strict fresh SBF workflow. The only host environment change is
`CARGO_INCREMENTAL=0`, avoiding regeneration of the previous session's large
incremental metadata caches. Existing compiler/diagnostic/network/resource guards
stay intact. Root passed all ten focused tests and 508 full host tests plus one
doctest/eight gates on the first execution, without diagnostics. The 101-input
freeze SHA-256 is `2c5e3df523255572a7823020c791ff73b7aa05b536ecb5472740fcc30a6a220e`;
all eighteen host logs match. Host compilation is not resource-guarded; the strict
SBF build retained its resource guards and passed on its first execution.
The 523,832-byte ELF at
`/tmp/piv1-bank-smoke-normalization-sbf-t242-a/artifacts/piv1.so` has SHA-256
`263493df16d42a96dfc6d631f4b8dee6b965c8c8e8165d380c9886478b8a148f`.
Its result receipt SHA-256 is
`ca8639f491479f942d7d011e66d364ec131b1f66f219741fe42d46a392e2072f`.
All 58 SBF logs match. The 296.35-second build reported no diagnostics, sampled
peak RSS+swap 604,176,384 bytes and minimum free space 2,309,865,472 bytes. Network
socket creation was denied (local UNIX socketpair allowed); the unchanged limits
were 2.5 GiB sampled group RSS+swap, 4 GiB per-process address space, 2-GiB disk
reserve and 1,800 seconds. Raw ELF64/little-endian ET_DYN machine 263, flags 0 and
entry 355,856 match the inspected receipt. Fourteen unresolved imports match Task
2.41, including `sol_sha256`. This is static inspection, not VM/Bank execution.

Host recompilation reduced measured free space to 2,110,488,576 bytes, below the
unchanged 2-GiB SBF reserve before new output. A bounded reviewed recovery resolved that blocker:
archive nonexecutable `.rlib`/`.rmeta`/`.o` files only from the exact historical
Task 2.39/2.40/2.41 SBF targets, preserving artifacts/logs and all nonselected paths.
The initial read-only plan failed because the old helper assumed a debug lock;
the corrected local wrapper locks both existing release paths for each target.
No dummy lock, helper mutation or weaker reserve was introduced. Original failure
receipt and wrapper are retained. No installation occurred.

The root executed the reviewed `recover_capacity-b.py` (SHA-256
`077a28e0f9654f39ec87d4de480a5c196f62fdddb551cc1eb7f772667171d05a`)
with `plan`, then `archive`, `prune`, `restore-one`, checking each result separately.
The unchanged helper is `tools/archive_cargo_intermediates.py` at
`7af6be53b8cc42b746f6e3dcf0b454685482cd268a2bf5ae0059dd28cecf8da2`.
All 424 compressed/decompressed objects were verified before removing 706 selected
paths (354 `.rlib`, 349 `.rmeta`, three `.o`). All 2,955 nonselected paths and the
33 prior preservation records remained unchanged. One 3,573-byte `.rmeta` was
actually restored and retained; 705 missing paths remain restorable. Separate
review verified compressed hashes, full manifest membership and journal history.
Post-recovery free space was 2,546,946,048 bytes.

Preserve `/home/jerem/piv1-evidence/task-2.42-sbf-intermediates-20260929-a`
(objects, manifest, ready receipt and journal), all three earlier archives and
all restored demonstrations. Its manifest SHA-256 is
`909c831a8a5850437f7e613fe05212cee42e8f58b3b66f190267b2e68ac71489`.
The three exact targets are `/tmp/piv1-bank-smoke-production-sbf-t239-b/target`,
`/tmp/piv1-bank-smoke-guardian-sbf-t240-a/target` and
`/tmp/piv1-bank-smoke-contribution-sbf-t241-a/target`. They now have incomplete
intermediate caches; their recorded ELF copies/executables/logs remain intact.

Restoration must use the pinned helper plus the exact reviewed task-local adapter
that locks `target/release/.cargo-lock` and
`target/sbpf-solana-solana/release/.cargo-lock` for each of the three roots.
The default helper CLI assumes a debug lock and does not apply. Verify manifest,
all bindings, archive membership/bytes, journal, destination absence and free
space; select missing records through `restore(manifest, raw, archive, "missing")`
with that adapter. Never overwrite regenerated paths. `restore-one` only names
the already-retained demonstration and is not a general restoration command.
The next preservation baseline must bind the new archive metadata and restored
file in addition to prior protected records.

Root validation commands were `python3 -B /tmp/piv1-t242-pilot-review/validate.py`
with modes `freeze`, `focused`, `final`, `sbf`. The recorded host commands use
pinned Rust 1.97.1 Cargo with `--locked --offline --jobs 1`; the eight final gates
cover workspace/all-target tests, doctests, default/all-feature checks,
`no-entrypoint`, `cpi`, `idl-build` and warning-denied documentation. Recovery and
validation receipts live in `/tmp/piv1-t242-pilot-review`; Git records the reviewed
integration commit after final evidence and document review.

Changed files: the new normalization handler/test/fixture; fixed-account accessor;
instruction codec/dispatch/exports; factual event; program README; and the root
README, AGENTS, checkpoint, specification, execution/test plans and this report.
No pure transition, serialized state or dependency changed. All source/test fixes
were completed before the first test execution; the only failed command was the
retained read-only recovery plan. Git records the integration commit; publication
receipts verify final clean worktree and protected main/Task 2.3 refs.

Next implement pending-to-principal bootstrap/integration with authenticated current
pool valuation, explicitly resolving the remaining Token-native compatibility.
The complete production lifecycle and M3–M6 remain open. No secrets access, key
creation, signing, deployment, Mainnet action, fund movement or authority transfer
occurred. Technical validation is not founder acceptance. Main stays unchanged unless
separately authorized. No secrets access, key creation, signing, deployment,
Mainnet action, fund movement or authority transfer is permitted by this task.
