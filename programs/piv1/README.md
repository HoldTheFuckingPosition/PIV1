# PIV1 non-deployable program library

This directory contains the founder-accepted Phase 1 bounded state layouts,
pure deterministic transition validation, and property/adversarial tests.
Task 2.1 now adds a narrow production-facing stake-pool contract plus a
fixed-capacity host-only mock and deterministic integration tests. Task 2.1 is
**COMPLETE / FOUNDER-ACCEPTED** at initial implementation commit
`33b1e539f969432f82635d1ca76c59d89f0ec233` and final corrected tip
`cb90d468eff4dce60552ba15b2b267b364a47827`. Task 2.2 adds pure
observation-based pending-contribution intake/reconciliation, a fixed-size
host-only custody mock, and recorded deterministic evidence; it is **COMPLETE /
FOUNDER-ACCEPTED** at implementation commit
`e3233b96b533a620e8037d5231baede10877217f`. Phase 2 is **IN PROGRESS**, and Task
2.3 is **TECHNICALLY VALIDATED / PENDING FOUNDER ACCEPTANCE** for the bounded pure and
host custody-composition scope. Task 2.4 fixed-account authentication is **TECHNICALLY VALIDATED / PENDING
FOUNDER ACCEPTANCE** at `9f4f106`; 191 host tests, one doctest and separate
source review pass. Later bounded work has not started.

The library remains deliberately non-deployable: there is
no Program ID, `declare_id!`, `#[program]` entrypoint, instruction handler,
`Accounts` context, program-to-cluster mapping, usable provider wallet, or
`cdylib` target.

`PivConfig`, `ActiveDistribution`, `WithdrawalLeg`, `GuardianRegistry`, and
`GuardianReward` are fixed-size Anchor/Borsh-compatible payloads with planned
discriminator-inclusive account sizes. They do not use `#[account]`, so they
are not yet deployable owner-bound Anchor accounts. Task 2.4 now authenticates
actual host `AccountInfo` backing for Config, ActiveDistribution and permanent
native/token custody under explicit trusted runtime program/Rent parameters.
It uses fixed discriminators and zero-padded existing payloads. Initializers and
state writers remain unimplemented. Instruction, event, and integration modules remain
non-deployable: there is no CPI, transfer, live address, Clock decoding, or
fund movement. The mock exists only below `tests/support` and is not exported
from the production library.

See `../../docs/TASK_1_3_STATE_MODEL.md` for the accepted layouts and transition
boundaries, `../../docs/TASK_2_1_MOCK_STAKE_POOL_ADAPTER.md` for the adapter and
deferred Phase 3 mapping, and
`../../docs/TASK_2_2_CONTRIBUTION_PENDING_MODEL.md` for the new pending-custody
boundary. See `../../docs/TASK_2_3_VAULT_RECONCILIATION_MODEL.md` for the
phase-dependent physical obligations, atomic host lifecycle, economic-vault
normalization and unsupported operational balance derivation. See
`../../docs/TASK_2_4_ACCOUNT_AUTHENTICATION.md` for fixed-account validation and
the token-native-excess limitation that must be resolved before handler integration.
Mock behavior does
not establish exact SPL/Jito/System/Token behavior. D-026 now authorizes bounded
reviewed technical progression toward founder Testnet testing; see
`../../docs/PIV1_PILOT_STATE.md` for the current task and actual evidence.
Founder acceptance and the mandate's live-operation/key/signing gate remain
separate. Mainnet key material must never be created or stored on this VPS.
