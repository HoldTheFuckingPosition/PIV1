# PIV1 program and accounting library

This crate combines the founder-accepted Phase 1 state/accounting foundation and
Tasks 2.1–2.2 models with Tasks 2.3–2.19 technically validated work, pending
founder acceptance. The [checkpoint](../../docs/PIV1_PILOT_STATE.md) and
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

Current-source evidence: **416 host tests +1 doctest / eight gates PASS**, recorded
in [Task 2.19](../../docs/TASK_2_19_JITO_ACCOUNT_IDENTITY.md). The historical
[Task 2.14 artifact](../../docs/TASK_2_14_RUNTIME_PENDING_RECONCILIATION.md) passed
**24 local SBF tests /70 cases**, including actual local System CPI. Those tests
used synthetic initial state and do not prove initialization, signatures,
Bank rollback, public deployment or later-source runtime behavior.

No dedicated live Program ID, initialized PIV1, production Jito CPI or full
Testnet readiness is established. D-026 technical progression and a review PR do
not grant founder acceptance, merge, deployment, key/signing or authority-transfer
permission. Mainnet key material must never be created or stored on this VPS.
