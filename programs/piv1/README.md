# PIV1 program scaffold

This directory is the Task 1.1 compile-only production program boundary. It
is deliberately not deployable: no Program ID, `declare_id!`, `#[program]`
entrypoint, program-to-cluster mapping, usable provider wallet, or `cdylib`
target exists yet. Anchor's required provider block is fixed to localnet with
/dev/null as an unusable signing sentinel. This prevents signing or deployment,
but Anchor's build wrapper still generated a disposable ignored keypair artifact
during validation; it was securely deleted without being read or used. Do not
rerun that wrapper until a documented keyless build method is available or key
generation is separately authorized.

The state, instruction, event, error, math, and integration types are marker
interfaces only. They contain no account layouts, account sizes, handlers,
constraints, transitions, formulas, CPI, live addresses, or fund movement.
The confirmed custody roles and logical transaction boundaries are documented
beside the relevant markers so later tasks cannot accidentally merge economic
categories or lifecycle stages.

Adding a deployable entrypoint or a Program ID is outside Task 1.1 and requires
separate authorization. Mainnet key material must never be created or stored
on this VPS.
