# PIV1 non-deployable program library

This directory contains the founder-accepted Phase 1 bounded state layouts,
pure deterministic transition validation, and property/adversarial tests.
Task 2.1 now adds a narrow production-facing stake-pool contract plus a
fixed-capacity host-only mock and deterministic integration tests. Task 2.1 is
**COMPLETE / FOUNDER-ACCEPTED** at initial implementation commit
`33b1e539f969432f82635d1ca76c59d89f0ec233` and final corrected tip
`cb90d468eff4dce60552ba15b2b267b364a47827`. Task 2.2 adds pure
observation-based pending-contribution intake/reconciliation, a fixed-size
host-only custody mock, and deterministic tests; it is **IMPLEMENTED / PENDING
FOUNDER ACCEPTANCE**. Phase 2 is **IN PROGRESS**, and Task 2.3 and later Phase 2
tasks are **NOT STARTED**.

The library remains deliberately non-deployable: there is
no Program ID, `declare_id!`, `#[program]` entrypoint, instruction handler,
`Accounts` context, program-to-cluster mapping, usable provider wallet, or
`cdylib` target.

`PivConfig`, `ActiveDistribution`, `WithdrawalLeg`, `GuardianRegistry`, and
`GuardianReward` are fixed-size Anchor/Borsh-compatible payloads with planned
discriminator-inclusive account sizes. They do not use `#[account]`, so they
are not yet owner-bound Anchor accounts. Externally owned custody accounts stay
address/role markers. Instruction, event, and integration modules also remain
non-deployable: there is no CPI, transfer, live address, Clock decoding, or
fund movement. The mock exists only below `tests/support` and is not exported
from the production library.

See `../../docs/TASK_1_3_STATE_MODEL.md` for the accepted layouts and transition
boundaries, `../../docs/TASK_2_1_MOCK_STAKE_POOL_ADAPTER.md` for the adapter and
deferred Phase 3 mapping, and
`../../docs/TASK_2_2_CONTRIBUTION_PENDING_MODEL.md` for the new pending-custody
boundary. Mock behavior does not establish exact SPL/Jito/System/Token behavior.
The exact next action is founder review of Task 2.2. Adding a deployable
entrypoint, Program ID, handler, CPI, or Task 2.3 behavior remains outside the
accepted scope and requires separate authorization. Mainnet key material must
never be created or stored on this VPS.
