# Task 2.38 — Exact initializer probes through Bank

Status: **TECHNICALLY VALIDATED / PENDING FOUNDER ACCEPTANCE**. Starting integration is
`692d1384afb530bd3740fa8aaf5c7f8f8f448755`; main remains
`4cc4ea11e87c2f1a2f85b9ed48358f2a881821b3`. D-026 authorizes bounded technical
work and reviewed integration-only publication. No new founder acceptance,
main integration, economics or live-operation authority is inferred.

## Bounded implementation

Task 2.37 established the genuine local Bank commit/account-read path for native
System instructions. This task applies that path to the three unchanged Task 2.31
ELFs: synthetic Squads caller, full normalized recipient-checked initializer
callee, and canonical-ID restricted Token InitializeAccount3 wrapper. It remains
synthetic local validation, not actual Squads governance or deployed Token/Jito.

One writer owns the isolated manifest/README, new `tests/genesis_bank.rs`,
`tests/genesis_fixture.rs` and this report draft. Root owns lock/metadata, the new
guarded runner/pins/regressions, all actual execution, shared docs and Git.
Separate source, command, exact-binary and independent actual-evidence reviews passed. The
writer does not execute Cargo, compilation, runtime, tests, network or Git.

The fixture ports the input recipes from Tasks 2.30/2.32 and their independent
literal expected state and CPI recipes to the current SDK. It neither imports
the PIV1/Anchor/SPL host graph nor uses recorded success bytes as expected output.
Exact Borsh/Token/loader fields are encoded from reviewed source layouts. Existing
production, economic decisions, probes, artifacts and old harnesses stay unchanged.

## Real loader and Bank boundary

Genesis stores V3 Program accounts and derived ProgramData accounts for the three
exact pinned artifacts. Their 45-byte ProgramData headers use deployment slot zero;
the callee authority is the fixture's existing derived governance vault. The other
two validation program headers have no upgrade authority. ELF suffixes are verified
by exact length and SHA-256 before construction. No program-cache injection,
mocked builtin replacement, loader instruction or deployment occurs.

The child Bank is created normally at slot one, because V3 code becomes visible
at deployment slot plus one. This freezes genesis. All profile messages use that
same unfrozen child Bank and unchanged blockhash. Default inactive features permit
SBPF V0; actual Clock/Rent bytes are checked, with time 100 and runtime rent floors
matching the accepted old-ABI fixture. ComputeBudget explicitly selects 1,400,000
CU and default heap remains 32 KiB. No resource result is assumed from Mollusk.

Bank treats zero-lamport accounts as absent. Read-only synthetic governance and
protocol accounts therefore receive explicit rent-floor genesis balances; fresh
target PDAs are absent. Mixed target prefunds retain the exact previous recipe,
including absent PendingSol and the two 55/89-lamport Token-native excesses. All
existing genesis fixtures use maximum rent epoch, and SVM supplies that value for
missing writable targets. The actual System builtin and real V3 Token account
replace old host metadata placeholders. Read-only synthetic Jito program metadata
remains uninvoked; no substitute Jito executable is loaded.

## Four profiles and twelve message cases

Distinct 35-account and shared 34-account profiles each run fresh/unpaused and
mixed-prefunded/paused initialization. Each profile executes this sequence on the
same Bank and AccountsDB:

1. `[ComputeBudget, valid caller, malformed caller]` must complete all initializer
   CPIs before exact `InstructionError(2, InvalidInstructionData)`. Ordered inner
   records match the callee call and every expected System/Token call, including
   keys, data and stack heights. Exact callee/caller success logs precede the later
   failure. Every non-fee account must equal its full pre-message state afterward.
2. An identical message and identical three zero signature placeholders must
   return `AlreadyProcessed`, preserving every account and charging no second fee.
3. `[ComputeBudget, valid caller]` changes the message without altering any approved
   initializer bytes. It must initialize from the actual failed/replayed state,
   preserving original rent obligations and exact 144-lamport normalization for
   prefunded profiles, all nine state envelopes, two Token accounts, owner/size/
   rent values and zero initial principal/HWM/pending/yield/KIF ledgers.

The fee payer, external initializer rent payer and executor are distinct. Three
required outer signatures imply exactly 15,000 lamports per processed message at
5,000 per signature; the failed transaction's rent payer is fully preserved.
The governance vault signs only inside CPI. Outer signature bytes remain zero:
this source-reviewed Bank entry assumes signature verification already happened
and establishes no cryptographic verification evidence.

Four complete snapshots per profile call `Bank::scan_all_accounts` over active
ancestry and independently reread each visible account. This includes inherited
genesis accounts, three full ELF-containing ProgramData accounts, runtime sysvars,
all target presence/absence, fee and rent payers and an unrelated 64-byte sentinel.
Success maps are independently constructed from literal accepted wire rules.
The harness never stores an account after construction, resets state, clears
replay protection or reconstructs Bank between attempts.

Compiled inner records do not expose nested signer/writable flags. This task does
not claim those flags or aborted raw account bytes; completed initializer logs and
ordered CPI records are distinct from actual post-commit stored-account evidence.
Every legacy transaction's serialized wire length is measured and exceeds 1,232
bytes. This is local unsigned Bank-entry evidence; ALT/packet/public transport
compatibility remains unproved.

## Dependencies, execution and evidence

Six already-locked imports are added only in this independent workspace:
solana-instruction 3.4.0, solana-sdk-ids 3.1.0, solana-loader-v3-interface 7.0.0
with bincode, solana-compute-budget-interface 3.0.0, sha2 0.10.9 and bincode 1.3.3.
Root owns the actual metadata/lock/feature comparison. No production dependency,
existing vendor patch or historical preparation pin is changed.

The runtime creates a fresh retained base directory and four profile children
from `PIV1_BANK_ACCOUNTS_DIR`. Its nine artifact environment values use the existing
`PIV_GENESIS_{CALLER,CALLEE,TOKEN}_{PATH,SHA256,BYTES}` contract. Resource settings
retain one-thread controllable pools, fixed cache bounds, the unchanged upstream
four-thread hasher and root's separately reviewed process/disk/time guards.

Root's offline metadata refresh and subsequent locked/offline observation passed,
with empty stderr. The host resolve has 553 nodes including this workspace root;
only six direct root edges change. Every non-root version, checksum, feature and
edge remains unchanged. The lockfile adds only those six root dependency names.
The original 589 registry archives/source trees and existing local proc-macro
override remain intact; there is no network resolution or installation.

Root's first runner regression attempt had 14 passes and one temporary-fixture
path error: a randomized underscore was correctly refused by the unchanged output
guard. The test fixture was narrowly changed to an explicit UUID hex suffix.
Separate review passed, then root passed all **15 focused runner tests** in
0.073 seconds. The failed attempt is retained as `runner-tests.*`, final evidence
as `runner-tests-b.*`, under `/tmp/piv1-t238-pilot-review`.

Separate source review identified a preventive scan-lock correction before any
build: collect scan records under the upstream callback, then independently
reread accounts after it returns. This avoids reacquiring an index read lock from
inside its own callback. All account/oracle coverage remains. Final harness hash:
`630cb01b0a154561d8b0f506a5877ca235a46d5b7ae4b8a11ae188b3b28c5a55`.
Fixture hash: `5cdab6479c4a8bc552f5680967a081c905750486dc2afaaeb332ece2046e1f5d`.

New pins `tools/bank_genesis_pins.json` cover **223 source inputs**, 158 tools,
50 aliases, all 589 original archives/source trees, three public library trees,
three exact runtime ELFs and seven historical artifacts. They inherit the
unchanged Task 2.37 guarded helper. The reviewer independently rehashed the 223
inputs and ten artifacts and checked the metadata delta. Of 217 prior source
pins, 215 remain unchanged; the two intentional changes are manifest and lock.
Six additional sources include the new runner/tests/harness/fixture, workspace
README and old pin manifest. Production, economics, old harnesses and all ten
artifacts remain unchanged; older suites were not rerun.

Actual commands so far (root execution):

```sh
/usr/bin/python3 -I -B tools/test_validate_bank_genesis.py
/usr/bin/python3 -I -B tools/validate_bank_genesis.py build --output /tmp/piv1-bank-smoke-genesis-build-t238-20260928-a
```

The second runner-test execution passed after the reviewed fixture correction.
Metadata commands, their exact scrubbed environments, hashes and receipts are in
`/tmp/piv1-bank-smoke-genesis-metadata-t238-20260928-a`. A third locked/offline
metadata observation inside the runner also passed before compilation.

The **first full build passed**, with no structured or raw diagnostics, in
482.135576 seconds. Its maximum sampled group RSS+swap was 940,589,056 bytes;
minimum sampled free disk was 4,054,081,536 bytes. All guards remained intact.
The final 52,358,608-byte host binary is:

`/tmp/piv1-bank-smoke-genesis-build-t238-20260928-a/target/debug/deps/genesis_bank-17e30741963bd812`

SHA-256: `ed8fc82fccedaee94be2914f3b18f3068f8d1d933cdcffba43cf0492b6d18ff2`.
Separate exact-binary review checked Cargo's sole matching artifact, source/pin
binding, ELF metadata, pinned native library identities, zero diagnostics and
all ten historical artifacts before approving the run.

```sh
/usr/bin/python3 -I -B tools/validate_bank_genesis.py run --output /tmp/piv1-bank-smoke-genesis-run-t238-20260928-a --build-output /tmp/piv1-bank-smoke-genesis-build-t238-20260928-a --approved-sha256 ed8fc82fccedaee94be2914f3b18f3068f8d1d933cdcffba43cf0492b6d18ff2
```

The **first actual Bank runtime passed**: one test, four profiles, twelve message
cases, four successful initializations and sixteen snapshots. Test duration was
4.37 seconds, guarded wall time 4.448860 seconds, with empty stderr and no guard
stop. Maximum sampled group RSS+swap was 29,646,848 bytes; minimum sampled free
disk was 4,009,504,768 bytes. These samples are observations, not a guaranteed
peak or a measured minimum resource requirement. No older suite was rerun.

| Profile | Complete account records | Late-failure CU | Retry CU | Original rent debit | Native sweep |
| --- | ---: | ---: | ---: | ---: | ---: |
| Distinct 35, fresh/unpaused | 176 | 1,076,146 | 1,065,383 | 34,779,120 | 0 |
| Shared 34, fresh/unpaused | 172 | 1,069,271 | 1,058,793 | 34,779,120 | 0 |
| Distinct 35, prefunded/paused | 221 | 997,126 | 986,363 | 890,885 | 144 |
| Shared 34, prefunded/paused | 217 | 990,251 | 979,773 | 890,885 | 144 |

All four exact replays return `AlreadyProcessed` with no execution or additional
fee. Every profile pays `[15000, 0, 15000]` across its three messages; the failed
message leaves all non-fee stored accounts exactly unchanged. Each retry succeeds
from the actual failed/replayed state. The entire account map, including external
recipients, executor, sentinel, program/ProgramData bytes and sysvars, matches the
independent literal oracle. Original prefund rent is not subsidized by the sweep.
All successful initializations exceed 200,000 CU; only the explicit 1.4m/default
32-KiB profile is proved. Serialized retry/failure wire lengths are 1536/1586 bytes
for distinct and 1503/1552 for shared; identical replay retains failure length.
All exceed the packet limit, so unsigned Bank entry is not public transport proof.

Root independently decoded all serialized messages, PDA addresses, loader/ELF
payloads, literal state envelopes, full stored account maps and ordered CPI tuples:
**786 complete account records**, sixteen snapshots and twelve messages passed.
Audit source/receipt: `/tmp/piv1-t238-pilot-review/inspect_genesis_evidence.py` and
`independent-evidence.json`. The separate reviewer independently checked all 786 complete account records,
256 target-presence records, 296 CPI tuples, sixteen snapshots and twelve raw
messages, without importing the root parser or rerunning Bank. Its first read-only
audit passed; no runtime/source correction was required.
Runtime stdout is 24,438,915 bytes, SHA-256
`91da24ad4eeb9fb9071975a70837bd6c04d1bd62bcb8e20b8c6b74d971cb2fcb`.
Complete build/runtime stdout/stderr and command/environment/resource receipts
are retained in the build/run directories. Root's post-execution hash verification
is in `/tmp/piv1-t238-pilot-review/post-execution.json`; old tests are retained
evidence, not reruns. Task 2.35's recovery archive remains untouched and its
contents were not rehashed in this task.

Changed files are the isolated workspace manifest/lock/README and two new test
sources, the new runner/pins/runner-tests, this report, and the six coordination
documents (`AGENTS.md`, root README, pilot state, execution plan, master spec and
test plan). Production, economics, historical harnesses, vendor source and prior
artifacts remain unchanged. Git records the final integration-only commit;
publication follows final source/evidence/document review and scoped checks.

## Continuation and limits

The passing run establishes the bounded synthetic initializer through actual
Bank/AccountsDB commit selection and ancestor-visible rereads, including rollback
after completed initialization and exact fee/replay/retry behavior. It does not
establish initializer-internal failed-CPI rollback, native production dispatch,
actual Squads/control, full recipient control, funding provenance, later Token
native-donation handling, complete lifecycle, signature validation, valid packet
transport, child freeze/root behavior, disk durability, ledger replay, validator
consensus or Testnet/Mainnet readiness.

Root records the final files, commands, Git identity and clean checkpoint after
reviewed integration-only publication. Main remains unchanged. Save and STOP
before Task 2.39. Next validate failures inside initializer CPIs through Bank,
with a fresh capacity check and unchanged artifacts/oracles. No Mainnet action,
deployment, real-fund movement, key creation/signing,
secrets access, RPC/chain operation or authority transfer. Technical validation
is not founder acceptance or a professional independent audit.
