# Task 2.23 — Unsigned genesis transport feasibility

Status: **TECHNICALLY VALIDATED / PENDING FOUNDER ACCEPTANCE**.
Base: `208b7fb4b7f597fe409429c3a7483b12f73a66af`, `integration/piv1-testnet`.
The founder requested one more economical bounded step after Task 2.22, then a
saved checkpoint and STOP. This task measures an actual SDK wire-encoding route
for the current genesis template. It does not expose or execute an initializer.

## Concrete template and pinned source basis

The host-only harness lives in `validation/genesis-transport/`. It uses existing
Node 24.19.0 and the unchanged locked/installed `@solana/web3.js` 1.98.4 from the
accepted spike. No Rust source, manifest, lockfile, dependency, schema or runtime
instruction selector changes. The template follows Task 2.22's exact topology:
eight bootstrap roles including Config, fifteen additional targets, seven protocol
roles, distinct external payer, System and Token. That is **33 inner accounts**,
or **32** when the allowed manager/referrer receiver is shared. Six guardian keys match the synthetic fixture, and all target PDAs derive from
its identities and canonical seeds;
PivAuthority stays virtual. The external payer is distinct from every guardian.

The exact 313-byte `PIV1GM01` model format remains approved library data, not a
native instruction ABI. Tests bind the selector/version/size to the existing Rust
format and verify field offsets, guardian permutation, account ordering and
privileges. The compact Squads message uses u8 key/instruction/account lengths and
u16 instruction-data lengths. It has one PIV1 instruction, no stored lookup and
no ephemeral signer. Persisted VaultTransaction message fields instead use u32
Vec lengths; an independent literal-layout test distinguishes the two encodings.

Squads source revision remains
`64af7330413d5c85cbbccfd8c27a05d45b6e666f`. Root retrieved and checked the exact
three buffer instruction sources against the previously retained revision tree:

| Source | SHA-256 |
| --- | --- |
| `transaction_buffer_create.rs` | `9fa902a9892612c22afd11cd44546fe05e58c9fb647ef6bda7b1dcb27d80ab50` |
| `transaction_buffer_extend.rs` | `6fbd1f2ae16dfd22b19128fd959ffd979409458963578bc598c3fd293c528ea5` |
| `vault_transaction_create_from_buffer.rs` | `94f7127e60bb26c394dbcd5a61b2ec97ed4861e38f6c9d231f4f7675bce7a59d` |

These files and the Git blob identities are retained at
`/tmp/piv1-t223-squads-sources/sources.json`. Existing cached transaction creation,
execution, SmallVec, transaction-buffer state and pinned IDL were reused. The IDL
SHA-256 is `cb9a0a29040ec3853a5105c547ffc608bb0e3175d78dacf30d463aff939f9b9b`.
Primary references: [buffer creation](https://github.com/Squads-Protocol/v4/blob/64af7330413d5c85cbbccfd8c27a05d45b6e666f/programs/squads_multisig_program/src/instructions/transaction_buffer_create.rs),
[buffer extension](https://github.com/Squads-Protocol/v4/blob/64af7330413d5c85cbbccfd8c27a05d45b6e666f/programs/squads_multisig_program/src/instructions/transaction_buffer_extend.rs),
[creation from buffer](https://github.com/Squads-Protocol/v4/blob/64af7330413d5c85cbbccfd8c27a05d45b6e666f/programs/squads_multisig_program/src/instructions/vault_transaction_create_from_buffer.rs).

Root independently compared all 78 installed web3.js package files to the npm
cache archive matching the locked SHA-512 integrity. Evidence is recorded in
`/tmp/piv1-t223-pilot-review/tools-dependencies.json`. Writer also checked installed
and locked versions and observed these exact tool/input hashes:

- Node binary: `bc17c508ffeed0ec622934f9b7fa72f8e78da65350e63c3eceb56fa688aa5e12`.
- web3.js CJS bundle: `796768bf5a689d808fb6fabfc5ce0db27eee064782cf87621331b807bab9f7e2`.
- Spike package lock: `a47e4310aa29ffb2ec0943626616bfefb0f0b8d7408090b360018b7e5e215e89`.

## Measured packet route

The comparison is explicitly against the **1,232-byte legacy/v0 bound of the
pinned SDK formats**. It makes no claim about v1 format or current cluster
availability. All public values are source constants or synthetic fixture values;
no deployment address, live table, keypair, signature or account is created.

The compact message is too large for direct creation. Buffer creation plus one
extension uploads the exact message, committed by SHA-256 and final size. The
model validator checks current synthetic creator identity, signer roles, canonical
buffer PDA, chunk order/length, total size/hash and exact final bytes. The executable
buffer bound is 10,128 bytes; its stale 4,000-byte source comment is not used.
Create-from-buffer supplies the pinned six-zero-byte placeholder and repeated
creator account; the outer compiler correctly upgrades both creator occurrences
to writable. Pinned closure refunds buffer rent to the creator, although the
distinct external payer paid creation rent in this template. This is an external
Squads preparation cost/refund, not a PIV1 reimbursement or economic reclassification.

The execution instruction has four fixed Squads accounts plus all 33/32 stored
keys in their exact order: **37/36 total instruction accounts**. Its transaction
requires two actual future signers, the external payer and distinct current
guardian executor. Both remain static keys in v0. The Squads vault is a nonsigner
in the outer transaction and gains its approved PDA signature inside Squads CPI.
Outer proposal privileges are writable while its inner PIV1 meta stays readonly.

One synthetic outer lookup table contains all sixteen target PDAs. It compresses
only the outer wire keys: the stored Squads message still has zero lookups, and the
table is not passed as an extra inner account. SDK decoding resolves the exact
outer account list and global privilege union; reconstructing the approved inner
metas preserves the deliberate privilege reduction before the conceptual PIV1 CPI.
No runtime CPI is executed by this reconstruction.

| Packet/payload | Distinct receivers | Shared receiver |
| --- | ---: | ---: |
| Compact message | 1,412 | 1,379 |
| Direct legacy creation, calculated from compiled layout | 1,794 | 1,761 |
| Buffer creation, 800-byte initial chunk | 1,215 | 1,215 |
| Buffer extension | 957 | 924 |
| Create from buffer | 421 | 421 |
| Legacy execution | 1,334 | 1,301 |
| v0 execution, sixteen target lookups | 874 | 841 |
| v0 plus illustrative compute/heap prefix | 922 | 889 |

Feasible packets are actual SDK-produced bytes, serialized/deserialized/
reserialized identically with complete data/account/privilege comparisons. Every
signature is a zero placeholder. Legacy execution bytes serialize but exceed the
packet bound. Direct creation is rejected by the SDK's oversized instruction
buffer; its reported size is independently counted from typed compiled fields,
not fabricated serialized output. The create-from-buffer field sum is:

```text
129 signatures + 3 header + 1 key count + 224 keys + 32 blockhash
+ 1 instruction count + 1 program index + 1 account count + 7 account indices
+ 1 data length + 21 instruction data = 421 bytes
```

The prefix contains illustrative compute-unit/heap requests only to measure their
48-byte packet overhead. These are not measured/approved resource requirements.
At the 33-account baseline, four loaded targets are insufficient; five fit. The
chosen sixteen-target table leaves more packet space. Buffer-creation boundary
tests independently observe an 817-byte first chunk at exactly 1,232 bytes and
818 bytes at 1,233. No uploader or network submission is supplied.

## Executed validation and correction

Writer executed the following direct commands with a clean minimal environment
(`PATH=/usr/bin:/bin`, `LC_ALL=C`, `TZ=UTC`), without network/package activity:

```text
/home/jerem/.local/piv1-toolchains/node-v24.19.0-linux-x64/bin/node --test validation/genesis-transport/transport.test.cjs
/home/jerem/.local/piv1-toolchains/node-v24.19.0-linux-x64/bin/node validation/genesis-transport/transport.cjs
```

The first run produced **8 passed / 1 failed** because a test literal expected
420 bytes for create-from-buffer. The SDK returned 421. Root independently
reproduced the same failure and both parties independently summed the field
layout above. The correction changed only that test expectation into the explicit
421-byte sum; no encoder or program behavior was altered. Both failed runs are
retained, not replaced:

- Writer: `/tmp/piv1-t223-writer-20260919-nxhsa_n5`.
- Root: `/tmp/piv1-t223-pilot-host-20260919-a`.

One writer corrective retry then passed **9/9 Node tests**, no skips/cancellations
or stderr diagnostics, in **1.514 seconds**. CLI JSON generation passed in
**0.815 seconds**, also with empty stderr. The result covers eight concrete
scenario reports: two Program IDs, distinct/shared receivers, with/without the
illustrative prefix. Source/tests remained unchanged during the successful run.

Writer final evidence: `/tmp/piv1-t223-writer-retry-20260919-kpkz8j1o`, containing
exact commands/env, tools, failed-run link, results, input manifests and full logs.
The **94 unchanged input hashes** include all 90 retained Rust inputs, both CJS
files and the two unchanged spike manifests. Final source hashes:

- `transport.cjs`: `72b427d00fd9733355bf0fa7aea5157704cfa043e0f39b99535e07bb669556e3`.
- `transport.test.cjs`: `03f82d158743757b0ade8d04de1f9fb957589d21f5cafb035eb40efeffadd80a`.
- Test stdout: `062ff3c2bf168cdf863f7ed610eec553623c1b31479ae7888e9c30f6c3f2ba75`.
- CLI JSON: `5eb9f7cac5b1acd5b715e6dd12a9340406429e353bf39d1e9e97ab79ac87b771`.

Regressions cover independent literal instruction discriminators, compact versus
persisted lengths, every truncation of the 1,412-byte compact message, trailing
bytes/unsupported lookup payloads, duplicate/out-of-range indices, exact signer/
writable semantics, altered data, wrong/missing/reordered lookup contents, buffer
authority/hash/size/chunk order/placeholder, oversized packets and SDK roundtrips.
Separate source/test review passed after the corrected expectation. Root then
independently executed the corrected suite and CLI on that same inspected freeze:
**9/9 Node tests PASS** in **1.070 seconds** (998.365 ms reported by Node), CLI
generation PASS in **0.669 seconds**, with empty stderr for both commands. This
was root's one corrective retry after its independently reproduced 8/9 result.
Root verified the input preservation, retained Rust freeze and matching writer/
root CLI JSON hash above. Evidence:
`/tmp/piv1-t223-pilot-host-20260919-b/pilot-summary.json`, SHA-256
`261330b7e3545ab4859c932780bbec989ec21dcbd26750c1a58b83f26cf04863`.
Root also verified all twelve first-run/final stdout/stderr logs and all 94
writer input hashes. Root owns final documentation checks and reviewed publication.

No Rust tests were rerun: all 90 Rust inputs match Task 2.22's retained **448 host
tests +1 doctest/eight-gate** evidence. This is historical unchanged-source
evidence, not new execution in Task 2.23. Task 2.14 SBF evidence likewise remains
limited to its earlier artifact.

## Remaining gates and checkpoint

The measured route removes the packet-size uncertainty for this concrete unsigned
template. It does not prove Squads buffer/ALT execution, approval choreography,
actual signatures, account locking, compute/heap/CPI limits or transaction rollback.
The table is synthetic: live identity, authority, funding, activation/warm-up and
contents remain unverified. Exact public-Testnet authorities, recipients, cluster,
artifact and budget still require the D-026 approval package. The current model
selector is still undispatched, so these bytes are not a runnable PIV1 initializer.

Economic Token-native prefunds, operational funding provenance and recipient
constraints remain before native exposure. Current-source SBF/runtime evidence
and an actual authorized end-to-end lifecycle also remain before founder Testnet
testing. No new economic choice or blanket live-operation permission is inferred.
Next session should scope the remaining initializer readiness prerequisite using
this retained transport evidence; no later task starts during this session.

Writer changes are limited to the three files in `validation/genesis-transport/`
and this report. Root updated `AGENTS.md`, `docs/PIV1_PILOT_STATE.md` and
`docs/PIV1_CODEX_EXECUTION_PLAN.md`; seven task files in total. Root owns Git and
integration publication. The exact commit/publication identity is recorded by
Git and must be reverified with remote refs and worktree state on takeover.
Writer performed no Git mutation, network/RPC/chain operation, signing/key creation,
Mainnet action, deployment, real fund movement or authority transfer. Accepted
main remains `5a067ad34d2f2ab0a27f2c6bff312a8b1f772d24`; this task is pending
founder acceptance and integration-only. This AI-assisted review is not a
professional independent audit. Save and STOP after reviewed publication.
