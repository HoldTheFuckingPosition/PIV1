# PIV1 non-deployable program library

This directory now contains the Task 1.3 bounded state layouts and pure
deterministic transition validation. Task 1.3 is implemented and pending
founder acceptance. The library remains deliberately non-deployable: there is
no Program ID, `declare_id!`, `#[program]` entrypoint, instruction handler,
`Accounts` context, program-to-cluster mapping, usable provider wallet, or
`cdylib` target.

`PivConfig`, `ActiveDistribution`, `WithdrawalLeg`, `GuardianRegistry`, and
`GuardianReward` are fixed-size Anchor/Borsh-compatible payloads with planned
discriminator-inclusive account sizes. They do not use `#[account]`, so they
are not yet owner-bound Anchor accounts. Externally owned custody accounts stay
address/role markers. Instruction, event, and integration modules also remain
markers: there is no CPI, transfer, live address, Clock decoding, or fund
movement.

See `../../docs/TASK_1_3_STATE_MODEL.md` for layouts, spaces, lifecycle rules,
transition boundaries, tests, and deferred handler validation. Task 1.4 has not
started; the exact next action is founder review of Task 1.3. Adding a
deployable entrypoint or Program ID remains outside this task and requires
separate authorization. Mainnet key material must never be created or stored on
this VPS.
