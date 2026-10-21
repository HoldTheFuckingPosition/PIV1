# Offline Buffer upload resume — bounded client evidence

Date: 2026-10-21 Europe/Warsaw. Source checkpoint: `ed3d332`.
Status: **TECHNICALLY VALIDATED / PENDING FOUNDER ACCEPTANCE**, limited to this
offline client. **D-048 conditional / READINESS UNMET / NO LIVE TRIGGER / PUBLIC
NOT READY**. M4 OPEN_UNWAIVED and stopped operations/drafts remain untouched.
The [single M5 register](PIV1_M5_TESTNET_READINESS.md) owns overall readiness;
the [JSON evidence index](PIV1_TESTNET_UPLOAD_RESUME.json) binds exact sources,
commands, actual host reports and separate reviews.

The planner accepts a bounded public manifest containing the pinned 894,552-byte
candidate, the exact seeded loader Buffer, explicit same-context Program/PD
absence and declared lifetime. It reads no wallet or artifact path and invokes no
network API. It validates all 936 aligned chunks before generating any packet:
matching bytes are preserved, zero nonmatching chunks may be scheduled, and any
unexpected nonzero content stops the plan. This is stricter than loader overwrite
capability. It does not support arbitrary prior fragment boundaries.

Each batch contains 1..64 unsigned legacy Write packets. Full chunks use 956 bytes
and a 1,232-byte wire; the last 692-byte chunk uses 968. The adjacent 957-byte
payload would exceed the wire cap at 1,233. There is one zero signature slot per
packet, belonging to G0 as fee payer/writer; the requested readonly writer becomes
globally writable because it is payer. Three retained resource caps are explicit,
not new measurements. No create, deploy, authority or initializer packet is emitted.

## Actual new host evidence

Root runs eight focused groups once: all pass, with no failures/skips. Independent
literal cursors check complete wires, global roles, payloads and full-width u64
margins. Eight deterministic mixed-state domains preserve preexisting correct
chunks; a separate full replay reconstructs all 936 writes in 15 bounded batches.
These are synthetic in-memory transitions, not successful blockchain transactions.

| Standalone synthetic case | Matched chunks | Pending chunks | Emitted packets |
| --- | ---: | ---: | ---: |
| Zero Buffer, batch 64 | 0 | 936 | 64 |
| Matching chunks 0/468/935, batch 3 | 3 | 933 | 3 |
| Complete exact Buffer | 936 | 0 | 0 |
| Late conflicting chunk 935 | 1 before conflict | No plan | 0; expected exit1 |

Root and separate audits independently check all 67 standalone wires/zero slots
and 2,808 success-case chunk records, then replay emitted bytes without changing
already-matching content. Null input reports 29 missing fields and plan:null.
The negative CLI has empty stdout and the exact conflict error. Pre-execution
fixture expectation/parser corrections remain preserved; no unexpected application
failure occurred. See [CLI usage](../clients/cli/README.md) for actual commands.

## Limits and next dependency

Snapshots, rent minimum and blockhash heights are supplied declarations; there is
no authentication, live control, current-state or throughput proof. The synthetic
height difference of 107 is exact beyond JS safe-number precision, and says nothing
about current Testnet lifetime. Do not infer confirmed progress from a planned
batch: independently refresh and verify the whole Buffer/state and lifetime before
each eventual batch. MATCHED_BYTES proves equality only, never transaction history.
The client performs no automatic rebind, signing/send, confirmation or live retry.

No new RPC, keys/private reads, signing, simulation, funds, build/Bank, deployment
or authority operation occurs. The previous 41-read startup budget and local Bank
proof remain retained, not refreshed. Candidate/bootstrap trust/real KIF anchor
remain unselected; G0 Buffer mutability, absent-vault runtime, version and current
external-image limitations persist. Clock must be bound before initializer storage
and votes; no anchor is chosen here. Consumed scopes stay consumed.

Next complete that exact founder/readiness package and remaining execution,
confirmation and lifetime/resume controls. Normal reviewed integration publication
only; main `824342d` and protected Task 2.3 `3677fee` remain unchanged. All 366 other
retained non-checkpoint inputs, the prior 67,665-byte CLI usage prefix and four
selected package sections are preserved. Keep the 2-GiB reserve and all evidence.
