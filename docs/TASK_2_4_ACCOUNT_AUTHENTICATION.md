# Task 2.4 — Fixed account authentication

Status: **TECHNICALLY VALIDATED / PENDING FOUNDER ACCEPTANCE**. Scoped by the technical pilot
under D-026 on `integration/piv1-testnet`, after Task 2.3 technical validation
and checkpoint `3677fee97e3617ee65e2828d222008ba0952bb3e`.

## Requirement and boundary

The remaining Phase 2 plan needs actual account/handler protections before
pure observations can drive live economic transitions. This task implements
the smallest read-only account-authentication dependency: Config,
ActiveDistribution and the seven permanent native/token custody accounts.
It does not implement handlers, initialization, transfers, CPI or deployment.

Authority: canonical decisions A-003/P-014–P-019/P-033–P-035, master spec,
execution plan Phase 2, and founder-accepted Phase 0 report section 5.1.
Existing fixed seeds, externally owned custody topology, bounded layouts,
liability accounting and Task 2.3 economic derivations remain authoritative.
No new economic policy, recipient, deployed Program ID or guardian power is
selected by this implementation.

## Required implementation

- Authenticate actual `AccountInfo` keys, owner, executable flag, allocation,
  canonical PDA and stored bump under an explicit trusted runtime PIV1 program
  ID parameter. Never accept a caller-selected serialized program ID as trust.
  Use the confirmed seeds for Config, ActiveDistribution, PivAuthority and
  all seven permanent custody addresses; PivAuthority is address-only.
- Decode Config and ActiveDistribution with type-specific eight-byte Anchor
  account discriminators and existing Borsh payloads. Preserve every field,
  ordering and planned allocation. Reject invalid/truncated/oversized data,
  noncanonical padding, wrong version/init/reserve bytes and inconsistent
  state binding. Document discriminator construction and layout compatibility.
- Require canonical System, legacy Token and Stake Program identities in the
  authenticated config. External pool/mint/fee/list relationships remain a
  later real-adapter boundary; matching config alone is not official-pool proof.
- Correct the existing pure Config default-key rejection narrowly for the
  System Program role: Solana's canonical System Program ID is the all-zero
  public key. Keep default/alias rejection for all other roles and existing
  permitted manager/referrer alias. Abstract host fixtures need not become
  production address fixtures.
- Native custody is empty-data, nonexecutable System-owned, at its own fixed
  PDA. Obtain floors from a validated Rent sysvar account (canonical key,
  Sysvar owner, decoding) or a clearly explicit trusted runtime Rent boundary;
  never accept arbitrary caller-supplied floors as authenticated evidence.
- Token custody is distinct 165-byte legacy SPL Token state with the configured
  JitoSOL mint and shared canonical PivAuthority, initialized, not frozen or
  native, with no extraneous delegate/close authority. Validate rent and report
  token-account lamports separately from token units; native excess there is
  unsupported for economic normalization and must not silently become yield
  or a contribution. Operational SOL stays separately observed; its surplus
  remains `UnsupportedFundingBaseline`.
- Derive the existing `EconomicCustodyObservation` only after all relevant
  account and bound state checks succeed. Return owned checked observations
  and explicitly document that they are point-in-time evidence requiring
  revalidation after CPI, not a transfer receipt or authorization capability.
- Reject aliases, borrow conflicts and arithmetic failures without panics or
  mutation. A read-only validator does not require a signer or writable flag;
  future state-changing handlers must enforce their own privileges, authorized
  destinations, atomic transfer deltas and runtime trusted program/sysvars.
- Keep pause orthogonal: reading/authenticating already received balances is
  not an economic transition. Do not select provisional explicit-transfer
  pause policy or implement KIF claims/guardian rotation here.

## Dependency and file budget

One delegated writer may add the account validation module, focused actual
AccountInfo tests, necessary exports/errors, the narrow config correction and
this task's implementation report. Existing source layout and math stay intact.
The writer may add pinned `spl-token = =8.0.0` with `no-entrypoint` for canonical
legacy decoding, using the already cached/previously spike-pinned package.
Prefer existing Anchor 0.32.1 Solana reexports for account/sysvar/PDA primitives.
A pinned existing-lock `bincode = =1.3.3` dev dependency is allowed only if needed
for authentic sysvar fixtures. Document necessity and review exact lock changes;
no unrelated dependency upgrades or broad reformatting.

## Failure cases and evidence gate

Meaningful tests use real AccountInfo backing data and canonical PDA derivation
with explicit non-deployed fixture IDs. Cover a valid complete custody set,
idle and active binding where applicable, owner/key/bump/discriminator/layout/
init/version/padding failures, swapped/aliased vaults, forged System/Token/Stake
identities, token authority/mint/state/delegate/close/native errors, rent deficit
and native surplus handling, malformed/forged Rent when decoded, borrow failure,
zero System ID compatibility and retained rejection for other default roles.
Verify read-only byte/lamport preservation on success and rejection and prove
observations remain compatible with Task 2.3 pending offsets/deficit checks.

The writer runs focused locked/offline host tests. The pilot inspects actual
source and lock changes, runs applicable workspace/default/all-feature/doc gates
on the frozen result, and obtains separate source/test review. Correct findings
before technical validation. Commit and checkpoint files changed, commands,
actual test/review evidence, limitations and next dependency before continuing.
No host account fixture is evidence of Solana runtime/CPI or live-cluster safety.

## Sensitive actions

No new keys, blockchain signing, RPC mutation, deployment, fund movement,
Mainnet operation or authority transfer is included. Live-operation approval
under D-026 remains NONE. Founder acceptance remains pending.


## Writer implementation evidence — 2026-09-09 UTC

Status: **IMPLEMENTED / READY FOR SEPARATE REVIEW / NOT FOUNDER-ACCEPTED**.
Writer verified `jerem`, `integration/piv1-testnet`, and starting HEAD
`3677fee97e3617ee65e2828d222008ba0952bb3e`. Parent-owned documentation edits
were preserved. No commit or publication was performed by the writer.

### Implemented boundary

`programs/piv1/src/accounts.rs` exports `authenticate_fixed_accounts`, the fixed
seed/discriminator definitions, `FixedAccountInfos`, `AuthenticatedFixedAccounts`
and `TokenVaultBalance`. Actual `AccountInfo` backing data is checked for the
required key, owner, nonexecutable state, allocation and rent exemption. All ten
fixed PDAs, including address-only PivAuthority, are rederived with canonical
bumps; the nine actual account roles cannot alias. Config's other bound roles
also cannot alias Config itself. Guardian registry/reward PDA authentication
and guardian/recipient rotation policy remain deferred.

The authentication function takes an explicitly trusted runtime executing program ID and
`Rent`. A future handler must obtain these from its execution context and
`Rent::get()` or independently authenticated sysvar. They must never come from
instruction data or unvalidated account/config fields. No caller-selected rent
floor or serialized PIV1 program ID is treated as authenticated evidence. No
Rent `AccountInfo` decoder is implemented, so no bincode dev dependency is needed.

Rent pricing delegates to the pinned `Rent::minimum_balance` primitive after
checked integer overflow preflight, finite/nonnegative threshold checks and a
valid burn-percent check. Saturated output is rejected. State accounts must
cover their exact allocation's rent minimum; native custody uses the empty-data
minimum; token custody uses the 165-byte minimum. No PIV1 economic calculation
adds floating-point arithmetic. Anchor 0.32.1 does not reexport the Stake module
or rent overhead; the source documents the canonical Stake ID from pinned
`solana-sdk-ids 2.2.1` and the 128-byte overhead from `solana-rent 2.2.1`.

Config and ActiveDistribution use the first eight SHA-256 bytes of
`account:PivConfig` and `account:ActiveDistribution`, respectively. Exact
allocations remain **1014 and 891 bytes**, with all existing Borsh field order,
versions and economics unchanged. Decoding consumes the actual variable Borsh
payload and requires every remaining allocation byte to be zero. This is an
explicit account-envelope rule, stronger than generic Anchor serialization:
future state writers must clear the unused tail when an Option shrinks. Invalid
option/bool/enum tags, discriminator, size, initialization, reserved bytes and
cross-object accounting/sequence bindings reject before economic observations.

Canonical System, legacy Token and Stake IDs are enforced. The pure Config
validator now allows the all-zero key only for the System Program role; all
other default-role and forbidden-alias rejection remains, including the existing
allowed manager/referrer alias. Abstract host fixtures remain compatible. A
configured external pool/mint/list/fee relationship is still not official Jito
proof; that requires the later real adapter.

Legacy token custody must have the configured mint and canonical shared
PivAuthority, initialized/unfrozen state, no native flag, no delegate, zero
delegated amount and no close authority. Token units and token-account native
lamports are returned separately. The base authentication result tolerates
native token-account excess and economic balance deficits for inspection, but
its `economic_observation()` accessor requires each economic vault's own
Task 2.3 obligation to be covered and rejects token native excess explicitly as
`UnsupportedTokenNativeExcess`. Unrelated surplus cannot hide a deficit.
Operational native custody stays separate and always reports
`UnsupportedFundingBaseline`.

**Remaining normalization liveness limitation:** one unsolicited native lamport
in either token account does not prevent base authentication, but blocks this
economic accessor. No safe normalization/sweep mechanism is implemented or
implied. This must be resolved at a separately bounded dependency before
production handler integration; token native excess is never silently classified
as yield or contribution. State-account native rent/surplus is likewise outside
the seven-vault economic normalization scope.

The result is an owned, read-only point-in-time snapshot, not a transfer receipt
or authorization capability. Future handlers must reauthenticate after CPI or
state/account mutation, enforce signer/writable privileges, allowed destinations,
atomic deltas and their own pause policy. Authentication itself remains allowed
during pause and does not implement transfer callability, KIF claims or governance.

### Exact writer commands and results

All cargo commands used `/home/jerem/.cargo/bin/cargo +1.97.1`.

| Command | Actual writer result |
| --- | --- |
| `update -p piv1 --offline` | Lock resolution only; seven authorized package additions, no existing version/checksum changes |
| `check -p piv1 --locked --offline` (initial source) | Failed: Anchor does not reexport `stake` or rent `ACCOUNT_STORAGE_OVERHEAD`; corrected without another dependency |
| `test -p piv1 --test account_authentication --locked --offline` | 23 PASS on first complete implementation; rerun after the Rent delegation adjustment: 23 PASS |
| `test -p piv1 state::config::tests --lib --locked --offline` | 9 PASS, 26 filtered out |
| `/home/jerem/.cargo/bin/rustfmt +1.97.1 --edition 2021 programs/piv1/src/accounts.rs programs/piv1/tests/account_authentication.rs` | Formatting unavailable: invocation reported the rustfmt component absent; no formatter binary found in installed toolchains; no installation attempted |
| `git diff --check` | PASS before final report append; final handoff check recorded by the pilot |

The focused suite uses real AccountInfo fixtures and canonical PDA derivation
under an explicitly non-deployed deterministic ID. It checks all nine owners,
keys, executable flags, fixed sizes and rent floors; all ten canonical bumps and
an alternate valid off-curve noncanonical bump; malformed state and Options;
all 21 non-System default roles; canonical programs; swapped/aliased custody;
both token accounts' malformed/forbidden state; all nine data/lamport borrow
conflicts; invalid/overflowing Rent; state/accounting overflow; read-only byte
and lamport preservation; pause; token native and operational excess; stale
owned snapshots; active state mismatches; all six economic deficit categories;
and actual host opening movements for pending SOL of 0, 4000, 8050 and 10000.
The last cases preserve full recognized contribution value and demonstrate no
false physical pending deficit through the existing reconciliation seam.

### Dependency and changed-file record

The only new direct dependency is exactly pinned `spl-token = =8.0.0` with
`no-entrypoint`, needed for canonical legacy account decoding. Offline lock
resolution adds `spl-token 8.0.0`, `arrayref 0.3.9`, `num-derive 0.4.2`,
`num_enum 0.7.6`, `num_enum_derive 0.7.6`, `thiserror 2.0.20` and
`thiserror-impl 2.0.20`. Existing locked packages and their checksums remain;
existing thiserror references gain version qualifiers only. No dependency
upgrade, math edit, payload schema change or unrelated reformatting is included.

Writer-owned files: `Cargo.lock`, `programs/piv1/Cargo.toml`,
`programs/piv1/src/accounts.rs`, `programs/piv1/src/errors.rs`,
`programs/piv1/src/lib.rs`, `programs/piv1/src/state/config.rs`,
`programs/piv1/tests/account_authentication.rs` and this report. Parent owns
AGENTS, execution-plan, pilot-state and readiness-checklist edits.

These are host checks only. Separate review, final workspace/default/all-feature
and documentation gates, Git checkpoint and publication belong to the pilot.
There is no claim of Solana runtime/CPI, localnet, deployed-account, official
Jito mapping, founder acceptance or professional independent audit evidence.
No Mainnet action, deployment, fund movement, key creation, signing or authority
transfer occurred. Live-operation approval remains NONE.

### Pilot validation and separate review — 2026-09-09 UTC

The pilot inspected the actual final source and tests and captured their hashes
before running the final gates. They remained unchanged through these checks.
All cargo commands below used `/home/jerem/.cargo/bin/cargo +1.97.1`.

| Final pilot command | Own execution result |
| --- | --- |
| `test --workspace --all-targets --locked --offline --quiet` | **191 PASS**, zero failures/ignored |
| `test --workspace --doc --locked --offline` | **1 doctest PASS** |
| `check --workspace --all-targets --locked --offline` | PASS |
| `check --workspace --all-targets --all-features --locked --offline` | PASS |
| `doc --workspace --no-deps --locked --offline`, with `RUSTDOCFLAGS='-D warnings'` | PASS |
| `git diff --check` | PASS |

The pilot independently recalculated both Anchor discriminator SHA-256 prefixes
and parsed the original/current lockfiles: all existing package names, versions,
sources and checksums are preserved; exactly the seven reported packages are
added. Targeted changed-file ownership, generated-path and credential-marker
checks passed. Effective custom hooks path is unset, hooks are samples only,
and no tracked GitHub workflow is present. Main remains at the accepted
`66193769d1cbc59cd8630df295b9a784b9c64642`.

Separate reviewer `review_t23_final` inspected the exact frozen Task 2.4 diff
against `3677fee`, complete account module, all 23 test bodies, manifest/lock,
errors/export/Config changes and report. Result: **PASS / no new actionable
findings within the bounded read-only authentication scope**. The reviewer ran
no builds; the command evidence above belongs to the pilot. The report retains
the token-native-excess liveness prerequisite and all trusted-runtime/real-adapter,
initialization, handler, transfer and CPI limitations. This is AI-assisted
engineering review, not a professional independent audit or founder acceptance.

The implementation commit and post-task documentation checkpoint are recorded
in `PIV1_PILOT_STATE.md`. No Mainnet action, deployment, fund movement, key
creation, blockchain signing or authority transfer occurred. Live-operation
approval remains NONE. Next work must be separately bounded and checkpointed
under D-026; no production readiness is inferred from this host validation.
