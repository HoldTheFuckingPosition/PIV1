# Task 2.39 — Production initializer boundary (M1)

Status: **TECHNICALLY VALIDATED / PENDING FOUNDER ACCEPTANCE**.

This is bounded M1 implementation under D-026/D-030, starting at integration
`9b386cd9c45de99e6f84185f65718df74221f27b`. Main remains
`4cc4ea11e87c2f1a2f85b9ed48358f2a881821b3`. Production previously dispatched only
earned KIF claims and pending recognition. It now routes the full recipient-checked
normalized initializer through the actual runtime Program ID. No allocation-only
or unchecked initialization instruction is exposed. M1's production boundary and local-runtime completion gates passed below. This
is not founder acceptance, complete lifecycle or live Testnet readiness.

## Native data and fixed account contract

`InitializePiv1Parameters` is exactly 315 bytes, with no vectors/trailing data:

| Offset | Field |
| --- | --- |
| 0–7 | ASCII `PIV1IN01` selector |
| 8 | Native version 1 |
| 9 | Governance vault index |
| 10 | Initial pause, exactly 0 or 1 |
| 11–234 | Seven protocol public keys, existing model order |
| 235–298 | HTFP and team-owner recipient public keys |
| 299–306 | Little-endian signed KIF anchor timestamp |
| 307–312 | Six guardian slot indices forming an exact permutation |
| 313 | HTFP recipient vault-index witness |
| 314 | Team-owner recipient vault-index witness |

The public legacy `GenesisModelParameters` codec remains exactly 313 bytes with
selector `PIV1GM01` and its original version/field layout. Legacy data cannot
dispatch native initialization. Both strict codecs share bounded field decoding;
internal model preparation selects the recognized format, then passes the
**entire original instruction slice** into Squads approval authentication. It
never strips witnesses, authenticates reconstructed model data or accepts a
detached decoded model as authorization. The two witnesses are arbitrary u8
derivation inputs checked against the approved recipient keys and authenticated
multisig; there is no 256-PDA scan or hard-coded 0/255 policy.

Manager/referrer key equality chooses the complete 34-account profile; otherwise
the profile has 35 accounts. Too few accounts return `NotEnoughAccountKeys`, too
many return `InvalidArgument`, and malformed data returns `InvalidInstructionData`
before account access. Account indices are fixed by the boundary:

| Role | 35-account profile | 34-account profile |
| --- | --- | --- |
| PIV1 Program / ProgramData / multisig / proposal / transaction / governance vault / Instructions | 0 / 1 / 2 / 3 / 4 / 5 / 6 | Same |
| Config / distribution / guardian registry / six guardian rewards | 7 / 8 / 9 / 10–15 | Same |
| PendingSol / PrincipalSol / OperationalSol / distribution escrow / KifSol | 16 / 17 / 18 / 19 / 20 | Same |
| PrincipalJito / PendingJito | 21 / 22 | Same |
| Stake-pool program / pool / validator list / reserve / mint / manager fee | 23 / 24 / 25 / 26 / 27 / 28 | Same |
| Referrer | 29 | 28, shared with manager fee |
| External signing rent payer / System Program / Token Program | 30 / 31 / 32 | 29 / 30 / 31 |
| HTFP recipient / team-owner recipient | 33 / 34 | 32 / 33 |

The underlying fresh checks still authenticate exact ordered accounts, approved
privileges, guardian mapping, proposal/time lock, ProgramData upgrade authority,
protocol relationships and recipient identities. Original external rent shortfalls
remain payable before normalization. Both Token accounts and all nine state
envelopes complete in the same call; every error returns to the transaction
boundary. No initializer event is added, and claim/pending behavior is preserved.

## Error ABI and host seams

Existing `Piv1Error` codes 6000–6073 and host-unavailable 6999 are unchanged.
Malformed model/native formats map to `InvalidInstructionData`. Exhaustive nested
matches preserve System/Token invocation and trusted runtime `ProgramError`
values unchanged, including errors below Squads authorization. New literal codes:

| Codes | Ordered meanings |
| --- | --- |
| 6100–6106 | Squads invalid invocation, Instructions sysvar, proposal, transaction, unsupported message, message mismatch, time lock |
| 6107–6118 | Protocol invalid identity, alias, owner, executable, borrow, unsupported program, pool, fee, list, reserve, mint, receiver |
| 6119 | Reserved, unassigned |
| 6120 | Invalid preflight roles |
| 6121–6124 | Allocation invalid roles, invalid payer, insufficient payer rent, observation mismatch |
| 6125–6129 | Recipient invalid roles, unapproved key, invalid vault, unfunded vault, observation mismatch |
| 6130–6131 | Initialization invalid roles, observation mismatch |

Ordinary host processing still returns 6999 after strict decode/count checks.
The existing claim callback seam cannot supply initializer execution: it reaches
the ordinary guarded initializer. A separate non-Solana initializer-only seam
shares the same dispatch/role mapping and accepts explicit modeled context/CPI.
It neither emits claim events nor rolls back partial host effects. These host
checks do not replace root's required actual production-artifact execution.

## Implementation and focused regression scope

The bounded source changes are the initializer codec, native boundary, exhaustive
error mapping and internal authenticated-format selection, plus relevant module
documentation. No dependency version, economics, state layout, existing selector
or historical artifact changes. New native regressions reuse the existing host
World/CPI emulator through a child module; old fixture/test bodies remain intact.

The focused cases cover literal native/legacy byte compatibility, strict lengths,
selector/version/boolean/permutation rejection, both account counts, ordinary and
old-seam host guards, exact nested error mapping, arbitrary witnesses 23/172,
fresh/prefunded and paused variants with two runtime IDs, complete old/new state
equivalence, original rent and 144-lamport normalization, changed approved tails,
wrong witnesses despite approval, authorization/reinitialization rejection,
account order/runtime-ID binding, System/Token errors and false-success postchecks.
Partial host effects remain observable on failure; no runtime rollback is claimed
from these tests. Existing claim/pending regressions form the compatibility gate.

`validation/genesis-production-caller` is an isolated synthetic local caller for
root's production Bank profile. It retains the Task 2.31 caller's parsing,
bounded account/privilege checks and signed governance-vault CPI behavior, but
strictly decodes the 315-byte native envelope and forwards its original bytes.
It is **not real Squads governance**. Historical caller source/ELF remain intact.
Its standalone workspace uses the same exact Anchor dependency and local PIV1
no-entrypoint feature; root owns locked/offline lock generation and graph review.

## Execution and review evidence

Task 2.39 / D-030 M1 is **TECHNICALLY VALIDATED / PENDING FOUNDER ACCEPTANCE**.
Starting clean integration was `9b386cd9c45de99e6f84185f65718df74221f27b` as
jerem (uid 1001); main remains `4cc4ea11e87c2f1a2f85b9ed48358f2a881821b3`.
Root owns normal reviewed integration-only publication under D-026/D-030; Git
records its exact identity. No new main authority or founder acceptance is inferred.

The real production boundary now dispatches the full recipient-checked normalized
initializer with strict 315-byte `PIV1IN01` version-one data and fixed 35/34 roles.
Both recipient derivation witnesses belong to the exact original approved bytes.
The public legacy 313-byte model codec, existing claim/pending paths, state layout
and economics remain unchanged. Validation errors receive explicit new codes;
actual runtime/CPI errors retain their exact values. Ordinary host calls fail closed.

One delegated writer implemented the production boundary, focused tests and narrow
synthetic caller; separate source, runner, exact-artifact and evidence reviews passed.
Root personally passed **32 focused tests, 479 host tests +1 doctest/eight final
gates, nine runner regressions and one production Bank test/four profiles/twelve
messages**. Root and reviewer independently checked **786 complete account
records**, sixteen snapshots, 256 target-presence records and 296 CPI records.
The actual production ELF initializes fresh/prefunded and paused/unpaused fixtures
with witnesses 31/202. Literal state/Token bytes, original rent, 144-lamport native
normalization, late-failure non-fee rollback, 15000 fees, no-fee identical replay
and distinct same-Bank retry match. Host tests separately cover approval tampering,
reinitialization, arbitrary witnesses/runtime IDs and precise CPI error propagation.

The first SBF preflight stopped before compilation on an outdated libexpat hash.
Installed Ubuntu package 2.6.1-2ubuntu0.6 was independently verified; only this new
profile adopted its current bytes. A reviewed preventive copy separates production
ELF evidence from the caller's no-entrypoint dependency build. The subsequent first
compiler attempt and first Bank build/runtime passed strict gates. No installation,
cleanup, dependency upgrade, ignored test or diagnostic suppression occurred.

The Bank build reused only the fixed Task 2.38 Cargo cache and verified empty
OpenSSL configuration directory, copying the new uniquely named executable into
fresh evidence storage. Old source versions remain in Git; twelve intentional
source changes and eight new inputs are explicit in the 231-source freeze, with
211 old inputs unchanged. All ten historical artifacts, the exact old Bank binary
and six old logs remain byte-identical. Cached dependencies are reused evidence,
not freshly rebuilt dependency proof. The failed preflight and every new output
remain retained. Current free space is 3558227968 bytes (3.31 GiB);
preserve the Task 2.35 recovery archive and all historical outputs.

Scope remains local unsigned entry with synthetic Squads and the restricted
pinned Token wrapper. Packets exceed 1232 bytes; actual signatures/ALT, governance
execution and exclusive recipient control, failed in-initializer CPI rollback,
funding provenance, durable restart and live Testnet readiness are not established.
Current claim/pending compatibility has host regression evidence; earlier SBF
claim/pending evidence remains historical. Old Node/model transport does not
serialize the new native initialization envelope. No Mainnet action, deployment,
key creation/secrets access/signing, real fund movement or authority transfer occurred.

**M1 is complete within this boundary/runtime scope; M2 is next and NOT STARTED.**
Expose canonical economic runtime handlers using existing model/adapter contracts,
then implement the real pinned protected adapter (M3), full production lifecycle
(M4), exact Testnet package (M5) and explicit deployment gate (M6), in D-030 order.
Do not restart initialization or create a validation-only detour without a concrete
critical-path blocker. D-023/D-024 pause treatment for explicit contributions remains
PROVISIONAL; resolve only if it becomes a material M2 economic decision. Official
Jito Testnet compatibility remains an M5 prerequisite, not assumed from reference IDs.

Evidence: `/tmp/piv1-t239-review`, `/tmp/piv1-t239-host-focused-a`,
`/tmp/piv1-t239-host-final-a`, and `/tmp/piv1-bank-smoke-production-{sbf,build,run}-t239-*`.
See [Task 2.39](TASK_2_39_PRODUCTION_INITIALIZER.md) for exact artifact identities,
commands and limitations. Technical validation remains separate from founder
acceptance; keep main unchanged and stop before any unapproved live operation.

### Exact current artifacts

| Role | Bytes | SHA-256 |
| --- | --- | --- |
| callee | 424040 | `c3357f487a62262ac4ff91bf3680a8b2c0d1bd96bfcc33c23a5c35509e83c4da` |
| caller | 61376 | `40173c99a1677002de595e4e344f6d547887e5bd8806ee2832a039c56d61f3c4` |
| token | 126424 | `c0f42a30da4079601711bec29bd0ca780674654ea71790eb32c76b5cedad4499` |

Reviewed host executable: `/tmp/piv1-bank-smoke-production-build-t239-a/genesis_production`, SHA-256
`31c9e814dceb8928d0c170eba81e564d626502a3b692478b4f9611e00383d2d3`. SBF copies live below
`/tmp/piv1-bank-smoke-production-sbf-t239-b/artifacts`.

### Executed commands and resource profile

All compilation is locked/offline, one job. Host commands and their clean
environment are retained in `results.json` under the focused/final host outputs.
The final host suite passed on its first run; ten new tests cover the native ABI
and shared initialization seam. Nine runner regressions passed on both reviewed
runner freezes; an initial seven-test subset also passed. No test failure was hidden.

```
/usr/bin/python3 -I -B tools/validate_production_initializer.py sbf --output /tmp/piv1-bank-smoke-production-sbf-t239-b
/usr/bin/python3 -I -B tools/validate_production_initializer.py bank-build --output /tmp/piv1-bank-smoke-production-build-t239-a
/usr/bin/python3 -I -B tools/validate_production_initializer.py bank-run --output /tmp/piv1-bank-smoke-production-run-t239-a --build-output /tmp/piv1-bank-smoke-production-build-t239-a --sbf-output /tmp/piv1-bank-smoke-production-sbf-t239-b --approved-sha256 31c9e814dceb8928d0c170eba81e564d626502a3b692478b4f9611e00383d2d3 --approved-sbf-result-sha256 6cded3cd36e8944d2ff3a665b10f755194388fc5105a8f717217eb3f722a9ff9
```

The exact runtime hashes and executed argv are retained in the root run receipt.
Hash-bound inherited tools preserve seccomp socket denial (local UNIX compiler IPC
allowed), 2.5-GiB sampled process-group RSS+swap, 4-GiB per-process address-space
limit, 2-GiB disk reserve and bounded timeouts. Sampling is not a peak/cgroup proof.
Static ELF checks require correct exported entrypoint and SBPF V0; stack/target
errors and warnings fail closed even if Cargo exits zero. No header patch or
oversized-stack bypass is allowed. Actual execution uses an explicit 1.4m CU cap
and default 32-KiB heap; it does not claim ordinary 200k-budget support.

A shared Bank helper refactor retains the legacy constructor and target, while a
new unique `genesis_production` target runs exactly one selected native test. The
runner rejects zero-test success, binds the copied host binary and exact reviewed
SBF result by hash, and rechecks all artifact bytes even on runtime failure.
Independent root and reviewer audits compare literal PDAs/state, complete accounts,
fees, loader bytes, approved native payload and exact ordered CPI keys/data/heights.
Nested CPI privilege flags and raw aborted intermediate state are not claimed.

### Actual compute and packet observations

| Profile | Retry CU | Late-failure CU | Retry wire bytes |
| --- | --- | --- | --- |
| distinct35-fresh-unpaused | 1066315 | 1077078 | 1536 |
| shared34-fresh-unpaused | 1059725 | 1070203 | 1503 |
| distinct35-prefunded-paused | 987331 | 998094 | 1536 |
| shared34-prefunded-paused | 980741 | 991219 | 1503 |
