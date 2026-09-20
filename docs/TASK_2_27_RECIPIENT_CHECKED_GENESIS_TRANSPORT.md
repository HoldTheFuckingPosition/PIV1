# Task 2.27 — Recipient-checked genesis transport validation

Status: **TECHNICALLY VALIDATED / PENDING FOUNDER ACCEPTANCE**.
Base: `648998b4f5767eadf14c511d1dd0034ffff29ee0`, `integration/piv1-testnet`.
The founder resumed one economical bounded task under D-026 on 2026-09-20 UTC,
then checkpoint and STOP. Main remains `7b74be4b13c019b96a0c8abcbebfbcc361d31089`;
D-028 does not authorize this later task's integration into main.

## Explicit additional host profile

The existing unsigned harness now offers `--recipient-checked` and
`recipientCheckedReport()`. The default CLI and `report()` preserve Task 2.23's
exact historical JSON bytes and eight scenarios. Unknown CLI arguments or fixture
profiles fail explicitly. No Rust source, dependency, manifest, schema, model
format or native instruction changes.

The new profile follows Task 2.26's complete initialization fixture: **35 inner
accounts**, or **34** with its shared manager/referrer receiver. Two canonical
Squads vault PDAs use literal seeds `["multisig", multisig, "vault", [index]]`
and illustrative indices **0/255**, under the same synthetic multisig as the
governance vault at index 7. They follow Token in the exact account order and
remain readonly/nonsigner. Their public keys occupy bytes **235..267** and
**267..299** of the unchanged **313-byte `PIV1GM01` format**. The pause flag
changes byte 10; no new field is added. These values are synthetic fixture
witnesses, not selected live recipient addresses or a new native role ABI.

Four exact retained Rust file hashes bind the report to the model decoder,
complete initialization fixture, Squads fixture and recipient checks at the
baseline. The report verifies these files and records their hashes. Tests also
independently construct the full ordered account list and payload from literal
public identities, PDA seeds, field offsets and expected privileges. A mutually
generated fixture is not the sole oracle.

The approved compact Squads message remains lookup-free and contains one PIV1
instruction, with two inner signers: the external payer and governance vault.
Execution has four fixed Squads accounts plus all 35/34 stored keys, **39/38
instruction account occurrences**. The distinct current guardian executor is an
additional outer signer, absent from the inner list. Payer and guardian remain
static keys with two zero signature placeholders. The governance vault is an
outer nonsigner; Squads would supply its inner PDA signature. The proposal's
outer writable/inner readonly distinction is preserved. Both recipients remain
static readonly/nonsigner keys in the decoded outer transaction.

## Measured legacy/v0 packets

Sixteen cases cover two synthetic Program IDs, distinct/shared receivers, both
initial pause values and presence/absence of the illustrative compute/heap
prefix. The existing pinned Node **24.19.0**, web3.js **1.98.4** and Squads source
revision `64af7330413d5c85cbbccfd8c27a05d45b6e666f` remain unchanged. Source and
dependency provenance is retained in Task 2.23; nothing was fetched or installed.

| Payload or packet | Distinct receivers | Shared receiver |
| --- | ---: | ---: |
| Compact approved message | 1,478 | 1,445 |
| Direct legacy creation, calculated compiled layout | 1,860 | 1,827 |
| Buffer creation with 800-byte first chunk | 1,215 | 1,215 |
| Buffer extension with remaining bytes | 1,023 | 990 |
| Create from buffer | 421 | 421 |
| Legacy execution | 1,400 | 1,367 |
| v0 execution with sixteen target lookups | 940 | 907 |
| v0 execution with illustrative compute/heap prefix | 988 | 955 |

All buffer packets and fitting execution packets are actual SDK bytes that
serialize, deserialize and reserialize identically, with exact reconstructed
data, account order and privilege checks. Legacy execution serializes but
exceeds the pinned **1,232-byte legacy/v0 bound**. Direct creation raises the
SDK's oversized-instruction-buffer `RangeError`; its size is counted from the
compiled layout, never presented as fabricated wire output. The bound is not a
claim about v1 format or current cluster availability.

The synthetic outer ALT holds only the sixteen writable PIV1 target PDAs. It
adds no inner lookup or recipient compression. Scanning the actual SDK encoding
finds these neighboring threshold packets:

| Receiver / prefix | Last oversized | First within 1,232 |
| --- | --- | --- |
| Distinct / absent | 6 targets, 1,250 bytes | 7 targets, 1,219 bytes |
| Shared / absent | 5 targets, 1,248 bytes | 6 targets, 1,217 bytes |
| Distinct / present | 8 targets, 1,236 bytes | 9 targets, 1,205 bytes |
| Shared / present | 7 targets, 1,234 bytes | 8 targets, 1,203 bytes |

Some smaller lookup counts exceed the SDK's message serialization buffer before
outer wire bytes exist: counts 0–2 / 0–1 for distinct/shared without prefix,
and 0–3 / 0–2 with prefix. The report records refusals separately. The prefix's
48 bytes reflect illustrative requests only; resource requirements were not
measured or approved.

Buffer creation and one extension assemble the exact 1,478/1,445-byte compact
message, bound by size and SHA-256. Buffer account sizes are 1,590/1,557 bytes.
The pinned six-zero-byte from-buffer placeholder, duplicate creator privilege
union and creator refund destination remain unchanged. Buffer rent closure to
the creator, despite the distinct payer funding creation, is external Squads
preparation accounting and does not create a PIV1 reimbursement or funding rule.

## Writer execution and source freeze

One focused test execution and the two CLI commands ran with the clean explicit
environment `PATH=/usr/bin:/bin`, `LC_ALL=C`, `TZ=UTC`:

```text
/home/jerem/.local/piv1-toolchains/node-v24.19.0-linux-x64/bin/node --test validation/genesis-transport/transport.test.cjs
/home/jerem/.local/piv1-toolchains/node-v24.19.0-linux-x64/bin/node validation/genesis-transport/transport.cjs
/home/jerem/.local/piv1-toolchains/node-v24.19.0-linux-x64/bin/node validation/genesis-transport/transport.cjs --recipient-checked
```

**15/15 tests PASS**, zero failures, skips, cancellations or diagnostics. The
original nine tests remain; six new groups cover the historical JSON golden,
source bindings and independent literal layout, sixteen measured cases and ALT
thresholds, missing/swapped/substituted/privilege-altered recipients and approved
bytes, stale topology, altered lookup resolution, and buffer/hash/compact binding.
Rehashing modified recipient bytes cannot replace the expected approved message.
A self-consistent unique replacement address still fails the canonical recipient
witness. Review finding T227-R1 strengthened that test oracle before execution:
the original replacement reused an existing key and could fail for duplication
before reaching the canonical witness. The unique replacement and exact error
assertion prove the intended path. This was not a production defect; no failed
run or corrective execution was needed.

Command times were **2.812561602 seconds** for tests, **0.607639035 seconds** for
the historical CLI and **1.287555143 seconds** for the new CLI. Node's test-suite
duration was **2,773.530331 ms**. All **96 inputs** remained unchanged: 92 Rust
source/manifest inputs, two CJS scripts and two spike manifests. The direct Node
binary, installed package/bundle and locked version hashes matched retained pins.

Evidence: `/tmp/piv1-t227-writer-20260920-e_q97383`, containing exact commands,
environment, tools, before/after manifests, preservation record and all six logs.

| Output | SHA-256 |
| --- | --- |
| Test stdout | `3bbb9b245b691ab9ba2bf2f0a277cb647a2187c57544299179d6e0389fe46de8` |
| Historical CLI stdout, unchanged golden | `5eb9f7cac5b1acd5b715e6dd12a9340406429e353bf39d1e9e97ab79ac87b771` |
| Recipient CLI stdout | `d8760a5525c5ede99ad6965a374646eee970c9747891d248c0f0195eea18efb8` |
| Each empty stderr | `e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855` |

Frozen source/test SHA-256:

| File | SHA-256 |
| --- | --- |
| `validation/genesis-transport/transport.cjs` | `a7694ff07577988e96546a53008bc8ab85bbc0ae7cb9353dcfc25e6c5a3c3724` |
| `validation/genesis-transport/transport.test.cjs` | `626294af1840e82e438242d7dcd56150a3f64f30862b6f9daeb56e9f82175856` |

Root independently verified all 92 unchanged Rust inputs, sixteen retained gate
logs, three pinned Rust tools and Node/dependency pins in
`/tmp/piv1-t227-pilot-review/retained-evidence.json`. **468 host tests +1 doctest /
eight gates** are retained Task 2.26 executions, **not rerun** here.

## Independent execution and review

Root independently executed the same three Node commands on the reviewed freeze:
**15 tests, eight historical cases and sixteen recipient cases PASS** on the
first attempt, with no failures, ignored tests or stderr diagnostics. Total
command time was **4.507463932 seconds**; the test command took **2.741444588
seconds**, with a **2,686.920395 ms** suite. Both CLI outputs are byte-identical
to the writer's results, including the original default golden.

Evidence: `/tmp/piv1-t227-pilot-host-20260920-a/pilot-summary.json` and its adjacent
commands, environment, tools, source manifests and logs. Root verified all 96
inputs against inspection and both executions, twelve new Node logs and sixteen
retained Rust gate logs. The root `results.json` SHA-256 is
`993a2d3fa5210e6cd0a7758a7c8659b2d530890f590e84108acaaf65992e2d9e`.

Separate scope and final source/test review passed. The reviewer confirmed
T227-R1 was corrected before either execution and found no remaining source/test
blocker. Final documentation review, checkpoint and integration publication are
root responsibilities.

## Limits and handoff

This proves host wire encoding for the expanded fixture, not an operational
initializer. The 313-byte model remains undispatched. Real ALT creation,
authority, funding, warm-up and lifecycle; transaction buffer/proposal/approval
lifecycle; signatures; live artifact identity; runtime CPI/resource/rollback
behavior; and native initializer exposure remain unproved. Recipient PDA identity
does not establish exclusive four-of-six control or absence of delegated spending
limits and executable stale actions. Funding provenance, operational baseline and
later Token-owned native donations remain deferred. No economic decision changed.

Writer files are the two CJS scripts and this report. Root owns shared documents,
the harness README, final Git status, normal integration-only publication and
commit identity. Verify actual refs/worktree after publication; Git records the
resulting commit. **Task 2.28 is NOT STARTED; save and STOP after this task.** No
Rust/SBF build, RPC, signing, key creation, deployment, Mainnet action, fund
movement or authority transfer occurred. AI-assisted review is not a professional
independent audit.
