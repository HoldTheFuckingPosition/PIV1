# PIV1 program and accounting library

This crate combines the founder-accepted Phase 1 state/accounting foundation and
Tasks 2.1–2.2 models with the **COMPLETE / FOUNDER-ACCEPTED** Tasks 2.3–2.19
milestone at `d9f3371be6ecb586675e3b38edcc57bd6e9519f8` (D-027, 2026-09-14).
Acceptance covers the recorded bounded scopes. The authorized milestone
fast-forward is published to main; separately reviewed documentation commits
maintain its acceptance records. Task 2.20 is **TECHNICALLY VALIDATED /
FOUNDER-AUTHORIZED MAIN INTEGRATION (D-028)** for fresh genesis/Jito/all-target
preflight and checked
rent-only shortfalls. It observes current accounts without creation, mutation
or classification of unsolicited prefunding. Task 2.21 is also **TECHNICALLY
VALIDATED / FOUNDER-AUTHORIZED MAIN INTEGRATION (D-028)** for fresh rent-only
allocation from a
distinct signing external payer and exact System CPI postchecks. The allocation
result is undispatched and incomplete: Token initialization and all state writes
must complete in the same successful transaction. Committing bare Token-owned
zero data is unsafe; see [Task 2.21](../../docs/TASK_2_21_GENESIS_ACCOUNT_ALLOCATION.md).
Task 2.22 is **TECHNICALLY VALIDATED / FOUNDER-AUTHORIZED MAIN INTEGRATION
(D-028)** for same-call
allocation, both Token initializations and all nine initial state envelopes with
exact postchecks. It remains undispatched, preserving prefund/funding, recipient,
transport and runtime-proof limits. Retained source evidence is 448 host tests +1
doctest/eight gates; see [Task 2.22](../../docs/TASK_2_22_GENESIS_ACCOUNT_INITIALIZATION.md).
Task 2.23 adds host-only unsigned transport encoding/packet evidence with the same
D-028 status: nine Node tests and eight report cases, without actual transport
execution. D-028 authorizes main integration of Tasks 2.20–2.23 at
`3282e1ebabcb0cd88491d48a391565b8b100afa7` plus reviewed records; it does not infer
broader founder acceptance.
The [checkpoint](../../docs/PIV1_PILOT_STATE.md) and
[integration review](../../docs/PIV1_INTEGRATION_REVIEW_2_3_TO_2_19.md) define the
current scope; individual reports preserve earlier evidence and limitations.

The `lib`/`cdylib` crate uses a thin native entrypoint with the actual runtime
Program ID. It does not invent a static `declare_id!` or deployed identity.
Only two instruction paths are dispatched:

- `claim_kif`: exact 24-byte data and five accounts; authenticated earned-liability
  accounting, existing-account persistence and a fixed signed System transfer to
  the guardian. It remains available during pause and does not create rewards.
- Pending-contribution recognition: exact eight-byte data and four accounts;
  authentication and idempotent updates to the two Config pending ledgers,
  including during pause/recovery. It makes no transfer or CPI.

Ordinary host entrypoint calls reject execution. Explicit host seams model
context, invocation and rollback. Other instruction markers remain unimplemented,
including initialization, deposits, distribution, heartbeat and governance.
The approved 313-byte genesis model format is not a dispatched initializer ABI.

Fixed Anchor/Borsh-compatible state payloads now have authenticated owner/PDA/
size/discriminator/version/zero-tail envelopes and atomic existing-account byte
persistence. The library also contains guardian/Clock and Squads checks, genesis
model preparation and source-pinned Jito identity authentication. These layers
do not compose themselves into an initializer, protocol CPI, or complete lifecycle.
The stake-pool/custody mocks remain test-only and do not establish exact SPL/Jito
behavior. Jito identity evidence leaves freshness, fees and execution readiness
separate; current-state Squads authority alone is not action approval.

Retained current-source evidence: **448 host tests +1 doctest / eight gates PASS**,
recorded in [Task 2.22](../../docs/TASK_2_22_GENESIS_ACCOUNT_INITIALIZATION.md), plus
**9 Node tests/eight transport cases** in
[Task 2.23](../../docs/TASK_2_23_GENESIS_TRANSPORT.md). All source inputs and logs
were reverified; no tests were rerun for this documentation-only main integration.
The historical
[Task 2.14 artifact](../../docs/TASK_2_14_RUNTIME_PENDING_RECONCILIATION.md) passed
**24 local SBF tests /70 cases**, including actual local System CPI. Those tests
used synthetic initial state and do not prove initialization, signatures,
Bank rollback, public deployment or later-source runtime behavior.

No dedicated live Program ID, initialized PIV1, production Jito CPI or full
Testnet readiness is established. D-027 authorizes the accepted milestone's
main integration; D-028 authorizes the bounded Tasks 2.20–2.23 integration.
Deployment, key/signing, funds and authority transfers retain
the separate D-026 approval gates. Mainnet key material must never be created or
stored on this VPS.
