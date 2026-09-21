# Isolated genesis recipient-preflight probes

These nonpublishable validation crates are **not production PIV1 or Squads
artifacts**. Task 2.29 prepares source, host boundary tests and strict offline
SBF compilation only. Neither SBF probe is loaded, executed, deployed or signed in
this task. See [the task report](../../docs/TASK_2_29_GENESIS_PREFLIGHT_PROBES.md).

`callee` exposes only the unchanged production read-only recipient preflight,
using the 32/31-account fixture: eight bootstrap roles, fifteen additional target
accounts, seven/six protocol roles, then the two recipients at vault indices
0/255. The existing model bytes select only the documented shared/distinct
manager-referrer topology. It calls no host-context seam, exposes no initializer,
performs no funding or Token/System CPI, and returns no authenticated facts.

`caller` is a synthetic Squads-ID caller. It bounds and decodes the stored single
approved instruction, checks canonical current authority identities, and prepares
exact account order, data and inner privileges. It does not authenticate proposal
approval as a substitute for the callee. Outer proposal writability is not copied
to the inner proposal; the outer vault must be a nonsigner. Only its SBF branch
can use the canonical `invoke_signed` seeds. Ordinary-host entrypoints fail closed
before CPI. The explicit read-only `prepare` function authorizes no execution.
Caller host fixtures use synthetic protocol/target accounts and independently
encoded Anchor/Borsh messages. Their tests establish wire/privilege preparation
and input preservation, **not successful production genesis preflight**.

Both crates use the existing pinned Anchor 0.32.1 and PIV1 `no-entrypoint` path
dependency. Their lock was seeded from the root lock and resolved offline; all
155 registry versions and checksums are a subset of its existing 166. No new
registry package or dependency version is introduced. The caller also declares
the existing pinned SPL Token 8.0.0 as a dev-only dependency because the reused
independent host fixture references its canonical program ID. No Token executable
is compiled as a separate artifact or loaded. Release arithmetic/LTO
settings match production. The independent guarded build runner retains the
production compiler, full raw diagnostic rejection and ELF/output guards while
binding both probe artifacts and their complete source/lock closure.

Later runtime work requires separately reviewed loading of these exact probe
ELFs, actual height-two CPI, runtime-derived Clock/Rent and the actual top-level
Instructions sysvar. The synthetic caller must never be described as the deployed
Squads implementation. Static compilation is not a claim about genesis total
heap/compute, recipient control, actual governance, initialization or rollback.
The current production artifact and its claim/pending evidence remain separate.
