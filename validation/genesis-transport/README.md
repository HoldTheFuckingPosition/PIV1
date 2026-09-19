# Genesis transport validation

This is an unsigned, host-only wire-encoding harness for the exact Task 2.22
genesis template. It demonstrates a packet-sized Squads buffer upload and outer
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
```

The first command runs nine regression tests. The second emits a deterministic
JSON report for two synthetic Program IDs, distinct/shared fee receivers and
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
