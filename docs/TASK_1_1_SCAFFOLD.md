# PIV1 Task 1.1 compile-only scaffold record

Date: 2026-08-30 UTC

Branch: task/1.1-anchor-workspace-scaffold

Starting baseline: 925ce91b5542cd64b2a733b39dbbfbd9ae129575

Status: COMPLETE / FOUNDER-ACCEPTED

Accepted implementation commit:
1d436570570fc31310e3e5d2c1d4d5e92320c65b

## Scope completed

- Created the pinned root Rust/Anchor and TypeScript workspaces.
- Created the production programs/piv1 modular source tree.
- Created the empty crates/piv1-math boundary for Task 1.2.
- Added compile-only markers for state, custody roles, instructions, events,
  errors, and SPL/Jito integrations.
- Added reserved unit, integration, adversarial, fixture, and CLI boundaries.
- Preserved the confirmed six logical lifecycle boundaries and distinct
  principal, pending, distribution, KIF, and operational-rent custody roles in
  English module documentation.

## Placeholder-only surface

The program crate is intentionally not deployable. It has no dedicated Program
ID, declare-id macro, Anchor program entrypoint, instruction handlers, Anchor
Accounts contexts, serialized account layouts, account sizes, stable error
codes, emit-ready events, state transitions, arithmetic, CPI, live addresses,
recipient addresses, guardian keys, transaction builder, or fund-moving test.

The root Anchor provider exists only because Anchor 0.32.1 requires one while
parsing. It is fixed to localnet and uses /dev/null as a deliberately unusable
wallet sentinel. No programs section maps PIV1 to a cluster.

The program crate is library-only. A cdylib target is deferred with the
dedicated Program ID and deployable entrypoint so Task 1.1 cannot produce a
deployable PIV1 binary.

## Confirmed boundaries represented

- PivConfig is the future bounded global configuration/accounting header.
- ActiveDistribution is one reusable bounded header with no unbounded leg
  vector.
- WithdrawalLeg and WithdrawalStake remain per-sequence/per-index temporary
  roles.
- PrincipalJitoVault and PendingJitoVault remain distinct non-ATA legacy Token
  accounts controlled by the shared address-only PivAuthority.
- PendingSolVault, PrincipalSolQueue, OperationalSolVault,
  DistributionEscrow, and KifSolVault remain separate native-SOL roles.
- GuardianRegistry and GuardianReward remain bounded fixed-set roles.
- Preparation, leg initiation/deactivation, leg finalization, beneficiary
  settlement, pending integration, and later principal compounding remain
  separate logical boundaries.

## Provisional items deliberately deferred

- Program ID, deployable entrypoint, cdylib target, and IDL.
- Exact PDA seed constants, bumps, owners, initialization funding, and close
  mechanics.
- Versioned account fields, serialization, discriminators, sizes, rent, and
  safely large technical integer/index bounds.
- Instruction arguments, Anchor account contexts, constraints, authorities,
  handlers, and transitions.
- Stable error codes and event fields/discriminators.
- Math APIs and all economic arithmetic, which belong to Task 1.2.
- State schemas and transition validation, which belong to Task 1.3.
- SPL/Jito adapter methods, account validation, and CPI, which belong to later
  integration tasks.
- Functional, property, mock-localnet, adversarial, Testnet, and deployment
  tests.

These deferrals do not change any confirmed economic, custody, governance,
slippage, timing, or multi-leg architecture decision.

## Validation

The accepted pinned toolchain was used as the jerem user.

~~~text
cargo +1.97.1 check --workspace --all-targets --locked --offline
PASS

cargo +1.97.1 test --workspace --all-targets --locked --offline
PASS (compile-only crates; zero functional tests by design)

npm ci --offline --ignore-scripts
PASS

npm run check
PASS

CARGO_NET_OFFLINE=true anchor build --no-idl -- --offline
PASS (release compilation; no deployable SBF artifact)
~~~

Rust formatting was not run because the accepted jerem toolchain deliberately
has no rustfmt component. No component or dependency version was upgraded.

## Build-key deviation

Anchor 0.32.1 generated an ignored target/deploy/piv1-keypair.json after the
successful build even though the crate is library-only and has no Program ID or
entrypoint. The file was never read, printed, tracked, copied, used, or funded.
It was immediately securely deleted, and a follow-up filesystem audit found no
PIV1 keypair or id.json under the production target directory.

This was a disposable local build artifact, not a Mainnet key or authorized
PIV1 Program ID. No transaction, deployment, fund movement, or authority change
occurred. Future compile-only validation must avoid rerunning the Anchor command
until a documented keyless method is available or key generation is separately
authorized.

## Next task

Task 1.2 has not started. The exact next task is Task 1.2 — implement and test
the pure math crate. It requires separate authorization and a dedicated branch
and must not begin automatically.
