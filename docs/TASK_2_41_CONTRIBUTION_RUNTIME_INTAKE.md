# Task 2.41 — Production SOL and JitoSOL contribution intake

Status: **TECHNICALLY VALIDATED / PENDING FOUNDER ACCEPTANCE** under D-026/D-030.
Completed 2026-09-29 UTC; a bounded M2 production block.
Starting integration: `1b1677c7df57925012e4f36ee597423aa641344d`.
Main remains `8912cfeffcd83fa12cc1a68937a9de8dd5c6b091`.
Root verified jerem (uid 1001), clean single worktree and matching remote refs.
No main publication or live-operation authority is inferred.

## Requirements and scope

Master specification §§6, 7.5–7.8, 14, 16 and 18, D-024/D-025 and the accepted
Task 2.3 custody composition define contribution accounting. D-032 requires both
explicit deposit handlers to reject during pause before transfer or state effects.
Direct incoming value already received remains separately reconcilable.

Expose real canonical System and legacy Token transfers into the existing
separate pending vaults. There is no deposit receipt, depositor withdrawal right,
Jito pool CPI, conversion, principal bootstrap, HWM change or distribution operation
in this block. Serialized state and accepted economics remain unchanged. A narrow direct
dependency on the already-locked `solana-sha256-hasher = "=2.3.0"` exposes the
required bounded hash API; no package or version is added to the lock closure. One writer owns implementation/tests, a separate reviewer checks source
and evidence, and root owns shared documents, actual validation and Git.

## Instruction boundary

| Operation | Strict data | Fixed accounts |
| --- | --- | --- |
| SOL deposit | `PIV1DS01`, version 1, u64 LE amount (17 bytes) | Config, active round, PendingSol, PendingJito, signing writable donor, System (6) |
| JitoSOL deposit | `PIV1DJ01`, version 1, u64 LE amount (17 bytes) | Config, active round, PendingSol, PendingJito, source Token, signing owner, configured mint, legacy Token (8) |

SOL uses the canonical System transfer from an external System-owned, empty,
nonexecutable source. No external-donor rent reserve or on-curve restriction is
invented. Chain rent-state/signature rules still apply. JitoSOL uses the pinned
legacy `TransferChecked` with authenticated mint decimals and a signing source
owner. Third-party delegate and SPL multisig authority modes are outside this
fixed profile. A distinct existing delegate and close authority are preserved;
when owner equals delegate, allowance decrement/clear follows the pinned Token
processor. Preserve raw option padding when the delegate tag clears.

Reject PIV-controlled donor/source identities and account/backing aliases. Require
both current pending obligations to be covered before any transfer: native
`P-U` during an active round (`P` at Idle), and the complete pending token ledger.
An incoming deposit cannot hide a prior shortfall in either asset. Preexisting
unexplained surplus is preserved for separate reconciliation, not attributed to
the current contributor. Preflight zero amounts, arithmetic, source balance,
writable permissions and mutable-borrow availability before effects.

Prepare the accepted pure contribution transition and fixed Config envelope,
invoke only the canonical transfer, reread/authenticate actual accounts and verify
source debit, destination credit and all unrelated state/metadata/native balances.
Account records include full metadata, data length and borrowed SHA-256 fingerprints;
arbitrary authority/program buffers are not copied onto the SBF heap. Only fixed
authenticated token buffers are copied to predict their transfer changes.
Only then commit the relevant pending ledger and emit a factual event. Errors
propagate to the transaction boundary; this function cannot undo arbitrary CPI
side effects on an ordinary host. Explicit modeled discard is not Bank rollback.
Ordinary host entry fails closed; no instruction selects an injected backend.

## Validation and limitations

Root executed nine focused transfer/accounting/adversarial regressions and
eight locked/offline one-job host gates on the corrected 98-input source. All
498 host tests and one doctest passed, with zero ignored tests or diagnostics.
Separate source/test/command and host-evidence review passed. Strict production
SBF compilation passed on its first invocation after the capacity recovery below. Host Token callbacks can run the pinned Token
processor; this remains host execution without VM, actual signatures or Bank.
Meaningful oracles include actual transfer bytes/metas, complete fixed-account
bytes/balances, untouched active snapshots/HWM/liabilities/rent, surplus/deficit,
pause, raw CPI failure/false success/metadata corruption and staged discard.

Baseline receipt: `/tmp/piv1-t241-pilot-review/baseline.json`, binding 233 prior
source inputs and 21 historical artifact/evidence records. The unchanged 76
Task 2.40 host/SBF logs were hash-verified, not rerun. Initial validation driver:
`/tmp/piv1-t241-pilot-review/validate.py`; reviewed SHA-256
`9b2bfabe5e98eec50b25e844b9875b9c077dc63a69ac7989c9363995ad3ea32b`.
It preserves existing source/tool/diagnostic/resource guards and disables Python
bytecode writes. The first focused build failed before tests because Anchor's
reduced Solana facade does not expose `hash`. The reviewed correction uses the
existing exact SHA-256 package directly, storing `[u8; 32]` fingerprints. Its
pinned implementation hashes locally on host and calls `sol_sha256` on Solana.
The only lock change is PIV1's direct edge; all 168 package records otherwise
match the baseline (`dependency-correction.json`). The failed logs and original
98-input freeze are retained. The corrected 98-input freeze is
`source-freeze-b.json`; `validate-b.py` changes only output/freeze suffixes and has
SHA-256 `4c6df32bc9aea6ca56e75f11914ab1cc326b8ddc2af4e7b243e5b62cc8af3951`.
The corrected source passed nine focused tests and all eight host gates
(498 host tests +1 doctest); separate source/test/command review passed.
Host feature builds reduced free space to 1,581,273,088 bytes, below the
unchanged SBF 2-GiB guard. The reviewed lossless recovery below closed that
concrete gate blocker before the first SBF invocation. Historical artifacts/logs and Task 2.35's
recovery archive remain intact; no installation occurs.

Actual current-artifact VM/Bank execution, complete economic lifecycle, the M3
real pool adapter and Testnet readiness remain unproved. M2 remains in progress;
M3–M6 remain open. No deployment, Mainnet action, secrets/signing/key creation,
fund movement or authority transfer occurs in this task.

## Build-capacity recovery within this task

The host gates generated new incremental cache bytes, leaving less than the
unchanged SBF 2-GiB free-space reserve. This is a concrete compilation blocker,
not a new validation-only milestone. Root prepared a lossless recovery profile
using the unchanged Task 2.35 archive library (SHA-256
`7af6be53b8cc42b746f6e3dcf0b454685482cd268a2bf5ae0059dd28cecf8da2`).
Only this session's 3,009 nonexecutable, owned, single-link files under
`target/debug/incremental` qualify: 2,487 `.o` files and 174 each of
`dep-graph.bin`, `query-cache.bin` and `work-products.bin`. The timestamp cutoff
is the first Task 2.41 source freeze; no historical target is broadly cleaned.
The reviewed in-memory extension profile does not edit the old tool or archive.

The first read-only inventory could not open a historical root-owned empty lock
file. No archive or prune occurred. The retained corrected wrapper records the
known empty bytes plus size/type/owner/mode without opening or changing that lock;
nonempty files still require actual hashing. All 22,461 nonselected target files
are bound to a preservation inventory. Candidates total 1,155,658,833 logical
bytes; worst-case archive bound 1,211,708,000 plus the separate 256-MiB archival
reserve fits observed 1,570,861,056 free bytes. This does not reduce the SBF guard.

Plan/manifest/wrapper and failures are retained in
`/tmp/piv1-t241-pilot-review/capacity-*` and `recover_capacity{,-b}.py`.
Manifest SHA-256: `4c394d621269754b30e6b95dd0637b6a1b3cd86fa49f420aed0253431b94091c`.
Any pruning requires a complete, decompressed-hash-verified archive under a Cargo
lock, with durable per-file journal and restoration path. The separate destination
is `/home/jerem/piv1-evidence/task-2.41-intermediates-20260928-a`; Task 2.35's
archive remains untouched. Root archived all 2,866 objects, verified them before
pruning all 3,009 originals, then restored one real `work-products.bin` and left it
in place. All 22,461 nonselected paths, frozen sources/logs and 21 historical
records remained unchanged. 3,008 paths remain restorable. The resulting
2,219,982,848 free bytes exceed 2 GiB but cannot fit the prior measured 230-MiB
SBF build footprint while retaining that reserve.

A second exact profile therefore selects 1,096 nonexecutable `.rlib`/`.rmeta`/`.o`
files under `/tmp/piv1-bank-smoke-genesis-build-t238-20260928-a/target`, totaling
935,260,343 bytes. It uses the unchanged helper's original extension policy and
actual Cargo lock, with 3,709 nonselected target files hashed. Its conservative
archive bound plus 256-MiB archival reserve fits current free space. The new
separate destination is
`/home/jerem/piv1-evidence/task-2.41-bank-intermediates-20260928-a`; the first
archive's manifest/ready/journal and restore receipt are additionally hash-bound.
Exact wrapper/plan/manifest are retained as `recover_bank_capacity.py` and
`bank-capacity-*`; manifest SHA-256
`0d6d2dc943622c60d30e2c80af52a6f10b8662cce37ec49ccadb97e8fe9c3fba`.
Root archived and verified all 1,096 objects before pruning those paths, then
restored one real object and retained it. All 3,709 nonselected target paths,
first-archive pins, sources/logs and historical records matched after each phase.
1,095 Bank paths remain restorable. Post-recovery free space was 2,868,199,424
bytes; before SBF root required the original 2-GiB reserve plus a 260-MiB margin,
exceeding the previously measured 230-MiB target footprint. The first guarded
strict SBF build passed; no SBF build was attempted at insufficient capacity.

The affected main/Bank targets cease to be complete incremental caches. Later
builds need restoration or recompilation; Task 2.39's fast cache reuse is no
longer a planning assumption. Recorded binaries, ELFs and runtime logs remain
valid retained evidence. The immutable archive helper's `restore` API with the
bound manifest and explicit `missing` selection provides the recovery path.
For the first archive, reproduce the reviewed task-local `{'.o', '.bin'}` profile
and independently enforce its exact incremental directory, three `.bin` basenames
and timestamp constraints before calling that API. The unchanged default helper
profile rejects `.bin`; its historical CLI and the wrappers' `restore-one` modes
are not general restoration commands. The second archive uses the default
`{'.rlib', '.rmeta', '.o'}` profile and exact historical Bank target restriction.
Verify bindings, space and source absence before restoration; never overwrite
newer paths. Preserve both new archives and Task 2.35's archive.

## Commands and recovery evidence

Root used `python3 -B /tmp/piv1-t241-pilot-review/validate.py freeze` and `focused`
for the retained failed initial build. Corrected-source commands use
`validate-b.py freeze`, `focused`, `final`, then `sbf`. The focused target is
`cargo test -p piv1 --test contribution_execution --locked --offline --jobs 1`.
Final gates are workspace all-target tests, doctests, all-target check,
all-features/no-entrypoint/cpi/idl-build checks and warning-denied docs, each locked,
offline and single-job. Host gates use their existing explicit clean environment;
they are not resource-guarded. Host source/tools/log hashes are independently
rechecked in `host-evidence.json`; source-freeze SHA-256 is
`d8ef536c2eed07197a77849a1eef33c0c4c8a35a54c443eb3f01d364602532fe`.

Recovery used the separately reviewed `recover_capacity-b.py` and
`recover_bank_capacity.py` `plan`, `archive`, `prune` and `restore-one` modes,
with exclusive receipts and preserved failure evidence. Each archive completed
and verified compressed/decompressed identities before any unlink. No executable,
ELF, source, dependency package, tool, prior validation log or secret was removed.
No installation, host privilege change, blockchain RPC or live operation occurred.

Next bounded M2 dependency: connect recognized pending custody to the accepted
principal bootstrap/integration model, deriving pool value from authenticated
current protocol accounting and executing exact same-asset transfers. Any required
M3 accounting/adapter component must be identified and implemented as that concrete
dependency; caller-selected pool values or mock revision counters cannot substitute.
Remaining distribution/withdrawal/settlement handlers, full real adapter, complete
local lifecycle and exact Testnet package remain open under D-030.

## Final SBF artifact and publication gate

Strict build elapsed 352.33 seconds, with no warning/error/stack diagnostic and
58 verified logs. The actual guarded command is recorded in
`/tmp/piv1-bank-smoke-contribution-sbf-t241-a/command.json`: clean explicit target
environment, `cargo build --package piv1 --lib --release --target sbpf-solana-solana
--locked --offline --jobs 1` and a fresh target directory. Inherited guards deny
new external network sockets while allowing compiler UNIX socketpairs, sample
2.5-GiB group RSS+swap and 2-GiB disk reserve, limit per-process address space to
4 GiB and time to 1,800 seconds. This is not a full filesystem sandbox or exact
peak-resource measurement. Sampled maximum RSS+swap was 614,150,144 bytes and
minimum free space 2,630,975,488 bytes. Existing tool/source/diagnostic guards and
the previously reviewed libexpat profile adjustment were preserved.

Immutable artifact: `/tmp/piv1-bank-smoke-contribution-sbf-t241-a/artifacts/piv1.so`,
494,816 bytes, SHA-256
`6ebf6d2d67424b06eb8e746239c5ceec8f8408cdd4255d3d0011c91959053683`.
Result SHA-256: `b69c095ba1972b22d28701b7135ccfb064de9424f3ecc6a3450393cafaf04552`.
Raw ELF64/little-endian machine 263, flags 0 and executable entry 330,864 match
static inspection; imported `sol_sha256` is registered in the pinned runtime.
This establishes compilation/static structure, not loader execution, CPI resource
sufficiency or Bank transaction rollback. No new-path VM test was run.

Root independently verified 98 final sources, all 76 new successful host/SBF logs,
three host tools, SBF tool/profile guards and 21 prior records. Separate final
artifact/document/publication review precedes integration-only publication.
Git and `/tmp/piv1-t241-pilot-review/publication.json` record the exact commit;
main remains `8912cfe`. No broader founder acceptance or live authority follows.
Changed files are the contribution module/codecs/dispatch/events and regressions,
the justified manifest/lock edge, program README and canonical status documents.
No deployment, Mainnet action, fund movement, key creation/signing, secrets access
or authority transfer occurred. No professional independent audit is claimed.
