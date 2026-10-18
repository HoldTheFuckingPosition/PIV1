# Genesis transport validation

This is an unsigned, host-only wire-encoding harness for the exact Task 2.22
genesis template and the explicitly selected Task 2.26 recipient-checked template.
It demonstrates a packet-sized Squads buffer upload and outer
Solana v0 execution route while keeping the approved stored Squads message free
of address-table lookups. The `PIV1GM01` payload remains undispatched by PIV1.

The harness uses the already installed, locked `@solana/web3.js` **1.98.4** from
`spikes/task-0.4-jito/node_modules`, with Node **24.19.0**. It adds no package or
dependency and performs no installation, RPC, signing or wallet/key creation.
Every public identity is either a pinned protocol constant or an explicitly
synthetic test value/PDA. All transaction signatures are zero placeholders.

Run with the retained toolchain:

```sh
/home/jerem/.local/piv1-toolchains/node-v24.19.0-linux-x64/bin/node --test validation/genesis-transport/transport.test.cjs
/home/jerem/.local/piv1-toolchains/node-v24.19.0-linux-x64/bin/node validation/genesis-transport/transport.cjs
/home/jerem/.local/piv1-toolchains/node-v24.19.0-linux-x64/bin/node validation/genesis-transport/transport.cjs --recipient-checked
```

The first command runs fifteen regression tests (the original nine plus six new
groups). The second preserves the byte-identical Task 2.23 deterministic JSON
report for two synthetic Program IDs, distinct/shared fee receivers and
presence/absence of two illustrative compute-budget prefix instructions. These
prefix values measure packet overhead only; they are not selected or measured
runtime compute/heap requirements.

For the 33-account template with distinct fee receivers:

| Encoding | Bytes | Result against the pinned legacy/v0 1,232-byte limit |
| --- | ---: | --- |
| Compact stored-message upload payload | 1,412 | Transport in chunks |
| Direct legacy transaction creation | 1,794 | Too large; calculated from SDK-compiled layout, serializer rejects |
| Buffer creation with 800-byte first chunk | 1,215 | Fits |
| Buffer extension with remaining 612 bytes | 957 | Fits |
| Create transaction from completed buffer | 421 | Fits |
| Legacy execution | 1,334 | Too large |
| v0 execution, one synthetic table with 16 target PDAs | 874 | Fits |
| Same v0 execution plus illustrative compute/heap instructions | 922 | Fits |

The third command selects Task 2.26's complete 35/34-account fixture. Two same-
multisig recipient vaults (indices 0/255) are appended after Token as readonly
nonsigners, with their keys in the unchanged 313-byte model format. These fixed
host role witnesses are not a new native ABI. Four retained Rust source pins
and independent literal topology/payload assertions bind this profile. The report
covers sixteen cases: two Program IDs, both receiver topologies, both initial
pause values and presence/absence of illustrative prefixes. Unknown CLI profiles
reject. The default profile remains the exact historical 33/32-account template.

| Recipient-checked encoding | Distinct receivers | Shared receiver |
| --- | ---: | ---: |
| Compact upload payload | 1,478 | 1,445 |
| Direct legacy creation, calculated; SDK rejects | 1,860 | 1,827 |
| Buffer creation, first 800 bytes | 1,215 | 1,215 |
| Buffer extension | 1,023 | 990 |
| Create from buffer | 421 | 421 |
| Legacy execution, serialized but oversized | 1,400 | 1,367 |
| v0 execution with 16 target lookups | 940 | 907 |
| Same v0 execution with illustrative prefixes | 988 | 955 |

Without prefixes the minimum number of loaded targets is 7/6 (distinct/shared);
with prefixes it is 9/8. Tests measure each neighboring oversized and fitting
packet. Smaller candidates refused by the SDK have no fabricated wire bytes or
roundtrip evidence. Recipient keys remain static readonly nonsigners; the lookup
table continues to contain only target PDAs.

Feasible packets are actually serialized, deserialized, reserialized and compared
using the pinned SDK. Account identities/order, global privilege union and data
must match. The outer guardian and external payer remain static signers; the
Squads vault is an inner PDA signer and an outer nonsigner. The proposal is
writable in the outer instruction and readonly in the approved inner invocation.
The table compresses only outer transaction keys; it adds no stored Squads lookup
or inner account. Preparation uses the same distinct external payer and member,
including the duplicated creator and writable privilege union at buffer closure.

Buffer serialization follows Squads revision
`64af7330413d5c85cbbccfd8c27a05d45b6e666f`: compact `SmallVec` u8/u16 lengths,
SHA-256 commitment, bounded chunks, exact six-zero-byte create-from-buffer
placeholder and no ephemeral signer or memo. The executable buffer bound is
10,128 bytes; the stale 4,000-byte source comment does not override it. Buffer rent
closes to the creator under the pinned program, which is separate from PIV1's
rent-only allocation policy and requires explicit future preparation budgeting.

Limits: this does not execute Squads, System, Token or PIV1; verify signatures;
create or authenticate a live lookup table; prove table warm-up/authority/funding;
measure compute, heap, account locks or rollback; establish recipient/economic
readiness; or authorize deployment. The 1,232-byte comparison applies to the
pinned legacy/v0 formats, with no v1 cluster-support claim. Updating the genesis
roles, payload or pinned sources requires re-review of this concrete template.

See [Task 2.23 evidence](../../docs/TASK_2_23_GENESIS_TRANSPORT.md), including the
retained initial failing size assertion and corrected validation. No Rust test
result is inferred from these Node executions.

See [Task 2.27 evidence](../../docs/TASK_2_27_RECIPIENT_CHECKED_GENESIS_TRANSPORT.md)
for the separate new-profile executions and pre-execution test-oracle correction.
The unchanged 468 Rust tests +1 doctest/eight gates are retained Task 2.26 evidence,
not inferred from Node and not rerun for Task 2.27.

## Current production initializer (separate M5 preparation profile)

`production-initializer.cjs` prepares the current **315-byte `PIV1IN01`**
instruction at source baseline `21cd68e5c02def1a9620320e126e6a3dcd7fa013`.
The historical tool and its tests remain byte-identical. This additive tool uses
pinned Node 24.19.0 / web3.js 1.98.4 and the historical pure compact-message,
buffer and outer-v0 helpers. It performs no RPC, installation, key creation,
signing or runtime execution.

```sh
/home/jerem/.local/piv1-toolchains/node-v24.19.0-linux-x64/bin/node --test validation/genesis-transport/production-initializer.test.cjs
/home/jerem/.local/piv1-toolchains/node-v24.19.0-linux-x64/bin/node validation/genesis-transport/production-initializer.cjs
```

The report binds current Rust codec, boundary, allocation, recipient, Squads
verification and Bank fixture sources plus the unchanged historical helpers.
It uses only two fixed synthetic PIV identities and synthetic public account
fixtures; none is a selected deployment identity or wallet. The approved stored
message has no lookups and no ephemeral signers. The outer v0 table contains
only the sixteen writable PIV targets and is synthetic and uncreated.

The fixture covers 35 distinct-fee / 34 shared-fee account roles, both initial
pause values, exact current signer/writable flags, and native recipient vault
witnesses at offsets 313/314. Default recipients are vaults 0/255; these are
valid alternatives to the Bank fixture's 31/202. Other selected u8 boundaries
are tested; every u8 witness decodes, while duplicate recipients and recipient
aliases with the governance vault reject. Guardian-slot permutations rederive
the six reward PDAs; the deterministic codec corpus covers all 720 permutations.
The stored proposal is readonly; the outer execute proposal is writable.

Governance vault index **7** is deliberately fixed because the reused buffer
creation helpers encode it. Recipient witnesses do not change that vault index.
Signed i64 timestamp boundary checks concern encoding only; they do not prove
any timestamp is accepted by production Clock/authentication.

The synthetic **fee payer and inner rent payer are both key 222**, with current
member 91 as the executor. Both outer signatures are present as 64-byte zero
placeholders; Squads supplies the inner vault PDA signature. Current production
allocation rejects a guardian as its rent payer. AB used a separate fee payer
242, inner payer 222 and executor 91, with **three** signatures. These measurements
prepare a different two-signature transport composition and do not replace AB's
1,544-byte observation or establish funded wallet control.

The tool assembles buffer creation, extension, creation from the completed
buffer, Active proposal creation (`draft=false`), four distinct approvals with
`memo=None`, and execute packets. Proposal instruction roles and fields were
independently inspected in the retained Squads IDL at revision
`64af7330413d5c85cbbccfd8c27a05d45b6e666f`, SHA-256
`cb9a0a29040ec3853a5105c547ffc608bb0e3175d78dacf30d463aff939f9b9b`.
Their literal schema is retained in the new tool; execution does not depend on
the temporary cache. Each feasible packet is serialized, deserialized,
reserialized and decompiled with exact bytes/accounts/global privilege unions
checked. Direct legacy creation size is calculated from the SDK layout when the
SDK refuses serialization. The report also measures the neighboring oversized
and fitting ALT counts.

The optional 1,400,000-CU / 262,144-byte heap instructions measure packet overhead
only. Actual Squads status transitions, permissions, timelock, proposal
consumption/replay, fees/rent/refunds, ALT funding/creation/warm-up, full runtime
CU/heap/account-data limits and real signatures remain unproved. Individual
approval packets are plans, not proof that four approvals succeed. No v1 code or
toolchain migration is introduced. Public readiness remains **NOT READY**;
M4's separate evidence reservation remains open and deployment is unauthorized.

Root's first new-suite execution passed **10/10** and the first native CLI passed
**16 cases**. Actual unsigned packet sizes, with both signature slots included:

| Packet/payload | Distinct receivers | Shared receiver |
| --- | ---: | ---: |
| Stored native message | 1,480 | 1,447 |
| Buffer create / extend / finish | 1,215 / 1,025 / 421 | 1,215 / 992 / 421 |
| Legacy execute / with illustrative prefix | 1,400 / 1,448 | 1,367 / 1,415 |
| Outer v0 execute / with illustrative prefix | 940 / 988 | 907 / 955 |
| Proposal create / each approval plan | 383 / 341 | 383 / 341 |

The historical fifteen-test results remain retained. Two old recipient-source-pin
cases bind Task 2.26 sources and are not current-native gates; their original
bytes are preserved. The new suite checks the unchanged default eight-case
historical report golden without misusing the old recipient-source bindings.
