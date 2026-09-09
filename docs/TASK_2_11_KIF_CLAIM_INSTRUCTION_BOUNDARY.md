# Task 2.11 — KIF claim instruction boundary

Status: **TECHNICALLY VALIDATED / PENDING FOUNDER ACCEPTANCE**.
Baseline: reviewed, published Task 2.10 closure
`9f9a8dbba132f96e4c76a8385844570746b0636f` on `integration/piv1-testnet`.
Authorization: active D-026; technical validation remains separate from founder
acceptance. This task follows completed Task 2.10 publication and checkpoint.

## Canonical requirement and technical choice

Connect the completed isolated claim to a strict instruction boundary, using
K-012 and master sections 11.5, 14 and 15. Preserve all existing authentication,
positive amount, historical-owner, pause, backing, replay, CEI, exact custody,
rent, carry and postcondition protections. No other instruction becomes callable.
State remains the accounting authority; events are factual observability only.

D-003 allows a documented technical reason for native Solana Rust at a specific
layer. Anchor 0.32.1 program codegen (`anchor-syn/src/codegen/program/entry.rs`)
requires a static `ID`; no authorized dedicated live PIV1 identity exists yet.
Use Anchor's pinned native `entrypoint!` reexport for this thin boundary, receiving
the actual runtime Program ID without inventing a `declare_id!` placeholder.
Keep Anchor serialization, account types and pinned APIs for existing layers.
This is a narrow implementation exception, not a stack/economic decision change.

Preserve `forbid(unsafe_code)`, existing feature meanings and `cpi` implying
`no-entrypoint`. Gate the entrypoint macro on `not(feature = "no-entrypoint")`;
do not handwrite unsafe deserialization. Pinned SPL Token 8.0.0 demonstrates the
same macro under its unsafe-code prohibition. Add `cdylib` alongside `lib` to
the program target if needed, without dependency/lockfile/toolchain changes.
Update descriptions that would otherwise incorrectly call the whole crate
compile-only. Anchor.toml stays a non-deployment sentinel without live identity.

## Exact wire and account contract

The only accepted instruction is exactly 24 bytes:

| Offset | Value |
|---|---|
| 0 | Eight-byte Anchor-compatible discriminator `fd97ac0bca4d76aa` |
| 8 | `amount_lamports`, little-endian u64 |
| 16 | `expected_cumulative_claimed`, little-endian u64 |

The pilot independently computed the discriminator from
`SHA256("global:claim_kif")[..8]`. Provide a matching fixed-size encoder; decoded
arguments map directly to the existing KifClaimRequest, without policy changes.
Accept exactly five ordered accounts: Config, GuardianReward, KifSolVault,
guardian signer/destination, canonical executable System Program. No remaining
accounts, registry, Clock account, pool, round, backend selector or destination
argument. Existing Task 2.10 authenticates every account and pays only its fixed
transfer. Reject unknown/IDL/event-CPI tags, every short/extra byte length and
every missing/additional account count. Reordered accounts fail authentication.

Define and test this error precedence: strict instruction decode, exact account
count, ordinary host availability guard, runtime Rent acquisition, existing
execution, then success event. Malformed data returns InvalidInstructionData;
fewer than five accounts returns NotEnoughAccountKeys; more than five returns
InvalidArgument. No invalid count may access an account or invoke a callback.

The production processor takes only runtime program ID, account slice and data.
It acquires trusted `Rent::get()` through the pinned sysvar API, not caller data
or a synthetic runtime account. On ordinary hosts, valid instructions reject
explicitly before Rent/account access or mutation. Solana execution calls the
fixed Task 2.10 production function. Propagate Rent errors and original System
CPI ProgramError unchanged to the transaction boundary; never catch-and-commit.

Publish explicit stable custom error numbers for each Piv1Error through exhaustive
matching with literal values (a documented reserved range beginning at 6000 is
appropriate). Never cast enum order, use a wildcard catchall or conflate distinct
errors. Give HostRuntimeUnavailable its own documented non-colliding code.
These codes are a technical ABI choice, not a new state/error economic policy.

## Success event and host evidence

Replace only the KifClaimed unit event marker with a factual event containing,
in order, GuardianReward public key, paid guardian public key, paid lamports.
Use the Anchor event convention: discriminator `04c9abfbd77711d0` from
`SHA256("event:KifClaimed")[..8]`, then 32 + 32 + 8 little-endian payload bytes.
Prefer the pinned Anchor event/log mechanism. Emit exactly once only after
successful execution and all postchecks, deriving fields from authenticated
accounts and actual successful transfer. No CPI-event account expansion or Clock
field. Emission is instruction success evidence only; a later transaction failure
can still leave observable logs, so consumers must require successful transactions.

A clearly named `cfg(not(target_os = "solana"))` test seam may inject Rent,
record/emulate the single System invocation and capture the event through the
same dispatcher. It must execute real Task 2.10 composition, not a mocked handler
returning success. No public on-chain callback or instruction-selected backend.
Model transaction rollback by staging the entire existing claim fixture and audit,
committing only on success; do not alter/rebase prior audit equations or count
predictions as receipts. Distinguish callback evidence from runtime/SBF proof.

Keep the historical ClaimKif unit marker compatible if useful, with separate ABI
arguments. The old property test may be renamed/documented as marker compatibility
because its claim of unimplemented pause policy is obsolete; preserve assertions
and every existing functional test. Other markers and instruction policies stay
unchanged. No initialization, recipient policy, registry update, heartbeat pause
decision, account schema change or real adapter work belongs to this task.

## Validation and completion gate

Add meaningful independent ABI/event literal-byte tests, u64 boundaries, all
short/extra lengths, unknown discriminators and exact error codes. Cover malformed
input/account-count precedence, reordering, trusted runtime ID, Rent failure,
ordinary host rejection without mutation, and host dispatch invoking execution
once/event once only on success. Preserve paused/historical claim, replay,
backing/carry/rent, CPI/postcheck failure and modeled rollback guarantees. Reuse
existing fixture/audit without weakening it. Check every supported feature build;
claim only host/typechecking evidence for the native macro/runtime code.

One writer (`implement_t26_deposit`) owns bounded source/tests and this report;
the pilot owns shared checkpoint/canonical status documents. Separate reviewer
`review_t23_final` must review the written scope and exact final diff/tests.
Writer runs focused locked/offline host tests, freezes files/hashes and releases
the build slot. Pilot inspects source and runs workspace tests + doctest,
default/all-feature checks and warnings-denied docs on the frozen source, plus
targeted source/path/ownership/hook checks before normal commits/publication.
Record failed commands and warnings honestly, distinct from final passing evidence.
No Rustfmt component is available; do not install one or claim a formatting pass.

## Build and deployment limits

No Anchor/SBF build is authorized in this bounded task. Task 1.1 records Anchor
automatically creating an unwanted ignored keypair; read-only inspection of the
installed cargo-build-sbf also found `generate_keypair` metadata. A verified
keyless build route is required before executing such tooling. Cached platform
tools alone do not prove safe invocation. Do not read existing key material.

No SBF artifact, export/stack/heap validation, runtime Rent/signatures/privileges,
actual signed System CPI or transaction rollback is established by these tests.
No suitable runtime test dependency is currently locked. Those remain later
bounded evidence tasks. No live Program ID, keys/signing, deployment, Mainnet,
fund movement, authority transfer or unrelated secrets access is authorized.
No founder acceptance or professional independent audit is implied.


## Written-scope review and dispatch

Separate reviewer `review_t23_final` returned **PASS / no required corrections**
for this exact written contract before implementation. It checked D-003/pinned
entrypoint and event code, strict error precedence, retained marker assertions,
feature handling, host modeling and build-key limits. This is scope review only;
final source/test review remains required. No reviewer build or mutation occurred.
The pilot now dispatches sole writer `implement_t26_deposit` for bounded code/tests,
focused locked/offline host validation and this report. Shared status documents
remain pilot-owned; no competing build is running.

## Writer implementation evidence — 2026-09-09 UTC

Writer `implement_t26_deposit` verified user `jerem`, branch
`integration/piv1-testnet`, HEAD `9f9a8dbba132f96e4c76a8385844570746b0636f` and the
expected pilot-owned documentation changes before editing. The writer read current
AGENTS/checkpoint, this approved contract, applicable D-003/K-012/master claim and
event requirements, and existing instruction/event/execution/state/error/audit
code. The initial exact file plan was sent to and approved by the pilot before
implementation. Scope approval and preliminary inspection are distinct from final
review, pilot validation and founder acceptance.

Writer-owned files:

- `programs/piv1/src/instructions/claim_kif.rs`: strict decoder and fixed encoder,
  retaining the existing `ClaimKif` unit marker.
- `programs/piv1/src/instruction_boundary.rs` (new): shared dispatch, runtime
  processor and explicit host-only callback seam.
- `programs/piv1/src/instruction_errors.rs` (new): exhaustive literal custom error
  codes and exact execution-error conversion.
- `programs/piv1/src/events.rs`: factual Anchor `KifClaimed` only; all other event
  unit markers remain unchanged.
- `programs/piv1/src/lib.rs`: exports, native entrypoint macro and current description.
- `programs/piv1/src/instructions/mod.rs`: accurate namespace description only.
- `programs/piv1/tests/kif_claim_instruction.rs` (new): 17 ABI/dispatch regressions
  and a dedicated host transaction wrapper around the unchanged claim fixture.
- `programs/piv1/tests/property_invariants.rs`: rename/comment only for the old
  marker-compatibility test; both existing assertions remain unchanged.
- `programs/piv1/Cargo.toml`: `cdylib` alongside `lib` and accurate package description.
- `Anchor.toml`: sentinel comment only; no provider/identity/signing configuration.
- This report: writer implementation/validation evidence appended to the contract.

Shared/canonical documents and Git remain pilot-owned. Task 2.10 execution,
authentication, accepted state/pure transitions, persistence, existing fixture/audit
code, layouts, adapters, dependency versions, lockfile and toolchain are unchanged.
No warning suppression was introduced. `forbid(unsafe_code)` and existing feature
relationships, including `cpi` implying `no-entrypoint`, remain intact.

### Instruction, error and event boundary

The matching encoder/decoder accepts exactly the specified 24 bytes: fixed
`fd97ac0bca4d76aa`, little-endian amount, then little-endian cumulative-claimed
counter. There is no backend, destination or runtime identity argument. Decoding
is shape-only: zero/maximal u64 values decode and retain their existing execution
policy. Unknown/IDL/event-CPI tags and nonexact lengths reject.

`process_instruction` takes only runtime Program ID, account slice and bytes.
A single private dispatcher enforces this order: strict decode, exact five-account
count, ordinary host availability guard, `Rent::get()`, accepted execution, then
one event. Malformed data maps to `InvalidInstructionData`, missing accounts to
`NotEnoughAccountKeys`, and extra accounts to `InvalidArgument`. Neither invalid
counts nor ordinary host rejection can access account data or call Rent/execution.
No remaining accounts are accepted. Rent errors and System invocation ProgramErrors
propagate unchanged. The dispatcher never catches an error to commit effects.

D-003's narrow native layer uses Anchor's pinned `entrypoint!` reexport under
`not(feature = "no-entrypoint")`. It accepts the runtime Program ID and avoids
Anchor program codegen's required static `ID`. No placeholder `declare_id!` or live
PIV1 identity is introduced. The macro compiled with the existing unsafe-code
prohibition; the writer added no unsafe decoder. `cdylib` is a host-build target
configuration here, not an SBF/deployment claim.

`instruction_errors::piv1_error_code` exhaustively matches all 74 current Piv1Error
variants to explicit literal numbers 6000–6073. It does not cast enum order and
has no wildcard arm; a new variant requires an explicit reviewed assignment.
6000–6998 is reserved for state errors and 6999 is exclusively the local
HostRuntimeUnavailable code. Existing numbers must not change or be reused.
Invocation ProgramErrors are preserved even if their external custom number
numerically overlaps a PIV1 code; they are not translated or conflated internally.
The complete published state mapping appears below.

Only `KifClaimed` became an Anchor event. Its fields are GuardianReward public key,
paid guardian public key and paid lamports, in that order. Its 80-byte event data
is `04c9abfbd77711d0`, 32 + 32 key bytes and little-endian u64 amount. The dispatcher
uses the authenticated record key and the successful Task 2.10 transfer, and emits
via the pinned Anchor mechanism only after execution/postchecks return success.
No Clock, CPI-event accounts or extra policy is introduced. State remains the
accounting authority. This event proves instruction success only: a later
transaction failure can leave observable logs, so consumers must require successful
transactions before treating an event as a completed payment.

### Host and runtime evidence limits

`process_instruction_with_host_callbacks` is explicitly gated to non-Solana hosts.
It injects a Rent provider, a System invocation emulator and an event sink through
the same private dispatcher. Its executor always runs the real Task 2.10 host
composition; it is not a handler stub returning success. No public Solana callback
or instruction-selected backend exists. Ordinary host processing rejects with
custom code 6999 before calling runtime Rent or account/execution APIs.

The transaction wrapper clones the entire existing claim fixture/audit and modeled
System account, runs actual shared dispatch/execution, measures native debit and
credit, advances the original paid-flow counter once and validates the unchanged
original audit equations before committing. Failed staged transactions are discarded.
The original baseline is never reconstructed. Modeled credit retains the prior
fixture's explicit funding/ledger meaning; it does not authenticate earning
eligibility. A raw failure test demonstrates persisted effects remain until the
transaction boundary rolls back. Another test deliberately retains a success event
trace while discarding a transaction after a modeled later instruction failure.
Neither test claims SVM rollback or runtime logging semantics were executed.

The writer inspected pinned local Anchor 0.32.1 program/event codegen, native
entrypoint and Rent sysvar implementations. Python SHA-256 independently confirmed
both required discriminators. Existing runtime execution wiring remains fixed;
no real signed System CPI, runtime Rent fetch, signature or privilege check was
executed on Solana. No SBF build/artifact/export/stack/heap validation, initialization,
full IDL/client flow, deployment or end-to-end runtime evidence is claimed.

### Focused tests and actual attempts

The 17 new tests independently pin exact instruction/event bytes, u64 boundaries,
all 0–23-byte short inputs, 25–1024-byte extra-length inputs, all single-byte tag
mutations and known IDL/event-CPI tags. Exact length guards reject all other excess
lengths by construction. They check all 74 literal state-code assignments and
uniqueness, host code separation and unchanged invocation error propagation.

Dispatch tests cover strict malformed/count/host/Rent precedence while holding
mutable account data borrows, missing/additional account counts, all 119
nonidentity permutations of five accounts, wrong supplied runtime identities and
original Rent errors. Native shared/mutable data/lamport conflicts remain checked
before effects and map to the exact custom code without invocation/event.

The successful host callback checks completed CEI accounting, canonical System
instruction/account list/seeds and available account borrows. The event arrives
once after payment and postchecks. Paused historical inactive claims, partial/full
payments, stale replay, later modeled credits, one-lamport backing deficits,
overclaim, missing signature and overflow preserve accepted behavior. CPI errors,
false success, wrong amount and malformed/tampered post-state propagate without
success events; complete modeled rollback and retry preserve the original audit.
The first success comparison pins exactly four accounting changes, both native
payment endpoints, the paid-flow counter, and all unrelated fixture state.

Actual writer command sequence, all host Cargo commands using
`/home/jerem/.cargo/bin/cargo +1.97.1` with `--locked --offline`:

1. `check --package piv1 --locked --offline` failed because Anchor's event macro
   required `Discriminator` in scope. The trait import was added.
2. `test --package piv1 --test kif_claim_instruction --locked --offline` failed
   compilation with three diagnostics caused by using the event-CPI tag slice as
   an eight-byte array. It also reported two unused-import warnings while the
   planned exhaustive error-code tests had not yet been appended. The tag was
   converted to its fixed array and the completed tests use those imports.
3. The same focused test command then passed **17 tests**, zero failed/ignored,
   without warnings.
4. `test --package piv1 --test kif_claim_instruction --test kif_claim_execution
   --test isolated_kif_claims --test state_persistence --test property_invariants
   --locked --offline` passed **94 tests**: 17 new instruction, 20 execution,
   24 claim, 21 persistence and 12 property tests, without warnings.
5. `check --package piv1 --all-targets --locked --offline`, followed sequentially
   by the same command with each of `--features no-entrypoint`, `cpi`, `no-idl`,
   `no-log-ix-name`, `anchor-debug`, `custom-heap` and `custom-panic`, all passed.
6. The same all-target check with `--features idl-build` initially failed because
   Anchor event codegen additionally required `IdlBuild` in scope. A feature-gated
   import fixed that issue; no warning suppression was used.
7. `check --package piv1 --all-targets --features idl-build --locked --offline`
   passed on the corrected source. The all-target check with `--all-features`
   also passed. These checks emitted no warnings.
8. The complete five-target **94-test** command from step 4 was rerun after the
   final feature-gated import correction: all passed, zero failed/ignored and no
   warnings. Source was then frozen and the build slot released.

Two initial read-only path guesses (`events/mod.rs` and `instructions.rs`) failed;
`rg --files` and the actual `events.rs`/`instructions/mod.rs` paths resolved them.
Routine verification used user/branch/HEAD/status/diff commands and local source
reads. `git diff --check` passed for the shared tracked diff; a targeted read-only
Git comparison confirms unchanged lock/toolchain, execution/authentication/state/
persistence and prior support sources. Rustfmt is unavailable; no component was
installed and no formatting pass is claimed. No functional test was disabled or
weakened. The historical ClaimKif property test retains both original assertions.

The pilot's final frozen workspace/doctest/check/docs results and separate final
review are distinct evidence to be recorded by the pilot.

### Stable custom state-error assignments

| State error | Code |
|---|---|
| `InvalidAccountOwner` | `6000` |
| `AccountNotWritable` | `6001` |
| `MissingGuardianSignature` | `6002` |
| `ZeroKifClaim` | `6003` |
| `StaleKifClaim` | `6004` |
| `KifClaimExceeded` | `6005` |
| `KifClaimBackingDeficit` | `6006` |
| `KifClaimStateChanged` | `6007` |
| `KifClaimObservationMismatch` | `6008` |
| `InvalidClockAccount` | `6009` |
| `ExecutableAccount` | `6010` |
| `InvalidAccountSize` | `6011` |
| `InvalidAccountDiscriminator` | `6012` |
| `InvalidAccountData` | `6013` |
| `StateEnvelopeEncodingFailed` | `6014` |
| `StateEnvelopeChanged` | `6015` |
| `InvalidAccountPda` | `6016` |
| `AccountAlias` | `6017` |
| `InvalidProgramIdentity` | `6018` |
| `AccountBorrowFailed` | `6019` |
| `AccountRentDeficit` | `6020` |
| `InvalidRent` | `6021` |
| `InvalidTokenCustody` | `6022` |
| `UnsupportedTokenNativeExcess` | `6023` |
| `InvalidVersion` | `6024` |
| `InvalidInitialization` | `6025` |
| `InvalidBootstrapState` | `6026` |
| `ZeroPrincipalDeposit` | `6027` |
| `PrincipalDepositExceedsQueue` | `6028` |
| `UnsupportedPrincipalDepositFee` | `6029` |
| `InvalidPrincipalDepositPool` | `6030` |
| `PrincipalDepositObservationMismatch` | `6031` |
| `PrincipalDepositMinimumNotMet` | `6032` |
| `PrincipalDepositHistoricalValueLoss` | `6033` |
| `InvalidLifecycle` | `6034` |
| `PausedOperation` | `6035` |
| `InvalidTimestamp` | `6036` |
| `TimestampRegression` | `6037` |
| `PreparationIntervalNotElapsed` | `6038` |
| `InsufficientAttemptCooldownActive` | `6039` |
| `InvalidInsufficientAttempt` | `6040` |
| `SequenceMismatch` | `6041` |
| `LegIndexMismatch` | `6042` |
| `ZeroTarget` | `6043` |
| `ZeroInput` | `6044` |
| `ZeroContribution` | `6045` |
| `InvalidCustodyObservation` | `6046` |
| `CustodyBalanceDecreased` | `6047` |
| `ContributionObservationMismatch` | `6048` |
| `PendingCustodyDeficit` | `6049` |
| `EconomicCustodyDeficit` | `6050` |
| `TargetExceeded` | `6051` |
| `NonMaximumSafeLegFill` | `6052` |
| `TechnicalFloorNotMet` | `6053` |
| `UsefulLegBoundExceeded` | `6054` |
| `Replay` | `6055` |
| `AlreadyFinalized` | `6056` |
| `TargetNotAssigned` | `6057` |
| `CountMismatch` | `6058` |
| `CumulativeReconciliationMismatch` | `6059` |
| `EscrowReconciliationMismatch` | `6060` |
| `ObligationExceeded` | `6061` |
| `OutstandingLiability` | `6062` |
| `SettlementReplay` | `6063` |
| `HighWaterMarkDecrease` | `6064` |
| `InvalidGuardianBitmap` | `6065` |
| `InvalidGuardianCount` | `6066` |
| `InvalidGuardianSet` | `6067` |
| `InvalidAddress` | `6068` |
| `InvalidSlippage` | `6069` |
| `InvalidSplit` | `6070` |
| `InvalidTimingConfiguration` | `6071` |
| `ArithmeticOverflow` | `6072` |
| `RecoveryRequired` | `6073` |

`HostRuntimeUnavailable` maps to `6999`; malformed data and account counts use
the standard ProgramErrors specified above. Original Rent/invocation errors pass
through unchanged.

### Frozen inputs and handoff

The complete ten-file inventory is `/tmp/piv1-t211-writer-frozen-source.json`.
No source, test, manifest or sentinel-comment edits follow this freeze.

| File | SHA-256 |
|---|---|
| `Anchor.toml` | `e649868bf200a6d029d66051ca0153b5a6b2be615fbfd01efe6c248c8b600c5a` |
| `programs/piv1/Cargo.toml` | `ad11cd2b9ec6a294ec4c068dcb1cacc6a604842bb5149d8b3831e23bf95f7c14` |
| `programs/piv1/src/events.rs` | `df1a83c4c97ba1c5601345c8fd10e37d998ad0b9071909f2cd19c354f4bc8d97` |
| `programs/piv1/src/lib.rs` | `ca190668fd0238eee9d2542eb97b4e2a1666b9e112e025ac0b2a5fedb6bbf035` |
| `programs/piv1/src/instructions/mod.rs` | `10cfdacda2babe649d28e001af2fcbe16842e6848844d9bdaa90c089c9b2200c` |
| `programs/piv1/src/instructions/claim_kif.rs` | `39ce641ad53ec6cdcc98c7061af9fe9cdb8db4462899ef7e7c0819ecb769e30c` |
| `programs/piv1/src/instruction_boundary.rs` | `2985b8012a32eb09505f423a42ac2accade415406e10ce2a4591e6d995901d5b` |
| `programs/piv1/src/instruction_errors.rs` | `46c0977bd24927fe580fa2562691fee2be8e4ca6a4cc8f96df87b0417e4a92ab` |
| `programs/piv1/tests/property_invariants.rs` | `5cc48dd97b46666a7617d6f736614b893484985d0c2fabf966c05beda6b00dc5` |
| `programs/piv1/tests/kif_claim_instruction.rs` | `04b57b3fa9ff39af07f4af3ea77d066036f5c05d33a995d19b1090b4d03504d4` |

At writer handoff, branch remains `integration/piv1-testnet` and HEAD remains
`9f9a8dbba132f96e4c76a8385844570746b0636f`. The writer made no Git mutation or
commit; pilot-owned documentation changes are preserved. Source/report are
pending pilot checkpoint and final review, not founder acceptance.

No Anchor command, cargo-build-sbf/SBF tool, network installation, key creation,
secret/key-material access, signing, deployment, live Program ID selection,
Mainnet action, fund movement or authority transfer occurred. A verified keyless
SBF build route remains required before that tooling can be authorized. This
host evidence and AI-assisted review are not a professional independent audit.

## Pilot inspection and actual frozen validation

The pilot inspected every production/test change, all 17 new test bodies and the
complete writer report. Independent Python hashing confirmed both discriminator
constants. The ten-file writer inventory matched actual files and was captured
as `/tmp/piv1-t211-frozen-source.json`; the exact baseline diff, including new
files, is `/tmp/piv1-t211-final-9f9a8db.diff`. No production correction was needed
after writer freeze. The final gated Anchor trait imports were inspected; no
lint/test suppression or functional-oracle weakening was introduced.

Actual pilot commands below used `/home/jerem/.cargo/bin/cargo +1.97.1`:

| Command | Pilot result on frozen source |
|---|---|
| `test --workspace --all-targets --locked --offline --quiet` | **335 tests PASS**, zero failed/ignored |
| `test --workspace --doc --locked --offline` | **1 doctest PASS** |
| `check --workspace --all-targets --locked --offline` | PASS |
| `check --workspace --all-targets --all-features --locked --offline` | PASS |
| `check -p piv1 --all-targets --features no-entrypoint --locked --offline` | PASS |
| `check -p piv1 --all-targets --features cpi --locked --offline` | PASS |
| `check -p piv1 --all-targets --features idl-build --locked --offline` | PASS |
| `doc --workspace --no-deps --locked --offline`, `RUSTDOCFLAGS=-D warnings` | PASS |

Evidence: `/tmp/piv1-t211-pilot-20260909T114436Z`, containing `results.json`, eight
logs and whole-source hashes; runner `/tmp/piv1-t211-pilot-gates.py`. All gates
returned zero. The pilot scanned every log: no warning/error diagnostic remained.
Complete source set/hashes stayed unchanged through all gates; all ten explicit
frozen hashes were also rechecked afterward. No build is active. These are the
pilot's executions, distinct from the writer's 94 affected tests and feature checks.
No further test repetition is justified without a new change or finding.

Targeted checks confirmed every changed file belongs to `jerem`, no credential
pattern or generated/key/artifact path was introduced, and `git diff --check`
passed. Existing claim execution/authentication/state/persistence/support,
Cargo.lock, root dependency/profile configuration and toolchain are unchanged.
Effective `core.hooksPath` is unset, hooks are sample-only, and no tracked
`.github`/`.cargo` automation exists. Independent pre-publication remote reads
confirm integration `9f9a8dbba132f96e4c76a8385844570746b0636f`, accepted main
`66193769d1cbc59cd8630df295b9a784b9c64642` and Task 2.3
`3677fee97e3617ee65e2828d222008ba0952bb3e`.

The pilot owns accompanying AGENTS/master/plan/checkpoint/test-plan changes and
Task 2.10 publication evidence. Normal implementation/hash-recording commits and
reviewed integration publication follow final separate review. No SBF artifact,
actual runtime Rent/signatures/System CPI/rollback or deployment was produced.
No Mainnet action, funds, new keys, signing, authority transfer or unrelated
secret access occurred. Founder acceptance and live authorization remain absent.

## Read-only next-dependency assessment

Separate reviewer `review_t23_final` inspected build prerequisites while the
writer held the host build slot. This did not start a subsequent implementation
or execute a compiler/builder. The pilot independently read the pinned public
compiler invocation, post-processing and Rust target definitions.

Published cargo-build-sbf 4.1.0 identifies source commit
`aa4455ad7ae9545220fe0a47682f37a0ac4cca46`. Both release/debug post-processing can
create a keypair; no suppression option was found. Compilation is a separate
ordinary Cargo stage. See [package provenance](https://docs.rs/crate/cargo-build-sbf/4.1.0/source/.cargo_vcs_info.json),
[post-processing](https://github.com/anza-xyz/cargo-build-sbf/blob/aa4455ad7ae9545220fe0a47682f37a0ac4cca46/cargo-build-sbf/src/post_processing.rs#L96)
and [compiler invocation](https://github.com/anza-xyz/cargo-build-sbf/blob/aa4455ad7ae9545220fe0a47682f37a0ac4cca46/cargo-build-sbf/src/bin/build_sbf.rs#L44).

Reviewer-reported local inspection: installed builder under the 4.2.0 release
directory embeds version 4.1.0; SHA-256
`7f696e537560a2f8b7f6e3a5b9423ae2d4be41f6ed1c2487982111d490e57462`.
Disassembly shows post-process calls to `generate_keypair` at `0x322148` and
`0x322187`. This establishes the installed hazard, not a reproducible provenance
match to the published source. Cached platform-tools v1.54 has Cargo/Rust, LLVM,
linker and v0–v3 target libraries. Its metadata identifies Rust source
`daa3af4a1110ec3f10ce08083bb0b7855a88416f` and Cargo source
`d96b806368b1a00ea55a3e2f08876edc4e86bc3e`. Reverify paths/hashes before use.

A proposed direct compilation route avoids builder post-processing. This is
**NOT EXECUTED and NOT AUTHORIZED WITHIN TASK 2.11**. A later bounded scope must
review final features/build scripts/tool hashes and select a fresh output path:

```bash
env -i \
  PATH=/usr/bin:/bin \
  CARGO_HOME=/home/jerem/.cargo \
  RUSTC=/home/jerem/.cache/solana/v1.54/platform-tools/rust/bin/rustc \
  RUSTDOC=/home/jerem/.cache/solana/v1.54/platform-tools/rust/bin/rustdoc \
  CC=/home/jerem/.cache/solana/v1.54/platform-tools/llvm/bin/clang \
  AR=/home/jerem/.cache/solana/v1.54/platform-tools/llvm/bin/llvm-ar \
  OBJDUMP=/home/jerem/.cache/solana/v1.54/platform-tools/llvm/bin/llvm-objdump \
  OBJCOPY=/home/jerem/.cache/solana/v1.54/platform-tools/llvm/bin/llvm-objcopy \
  CARGO_TARGET_SBPF_SOLANA_SOLANA_RUSTFLAGS='-Zremap-cwd-prefix= -C linker=/home/jerem/.cache/solana/v1.54/platform-tools/rust/lib/rustlib/x86_64-unknown-linux-gnu/bin/rust-lld' \
  /home/jerem/.cache/solana/v1.54/platform-tools/rust/bin/cargo \
  build --manifest-path /home/jerem/piv1/programs/piv1/Cargo.toml \
  --package piv1 --lib --release --target sbpf-solana-solana \
  --locked --offline --jobs 1 \
  --target-dir /tmp/piv1-keyless-sbf-candidate
```

The command uses existing binaries without rustup/install or inherited wrapper
configuration. Its target configuration comes from the
[pinned Rust target](https://github.com/anza-xyz/rust/blob/daa3af4a1110ec3f10ce08083bb0b7855a88416f/compiler/rustc_target/src/spec/base/sbf_base.rs#L61).
This candidate is v0, a local validation choice only; deployment architecture
compatibility remains unproven. A future explicit LLVM strip step can create a
separate artifact without key handling. The published builder also warns that
combined cdylib/lib targets preclude its expected LTO; record actual compiler
behavior rather than silently changing the accepted release profile.

Required later gates include actual compilation, ELF/entrypoint/relocation/syscall
inspection, artifact identity, stack/heap diagnostics and a separately pinned
keyless runtime harness proving real Rent, privileges, signed System CPI and
transaction rollback. No suitable runtime harness dependency is currently locked.
Never suppress stack problems or present host callbacks as this runtime evidence.


## Final separate review and technical closure

Reviewer `review_t23_final` returned **PASS within the bounded instruction ABI
and host-evidence scope; no actionable findings**. The reviewer read the exact
ten-file patch against `9f9a8db`, all actual source and all 17 new tests, the
complete writer/pilot reports and the read-only build assessment. All ten actual
hashes match both inventories and the report; every patch section matches its
file. Inventory SHA-256 is
`ffd820b838b4b880d5a1268dbfc894f413c8895fea6e5e658fadbd170d08e444`.
The reviewer independently checked discriminator hashes, all 74 variant/code
assignments, preserved old code and pilot results.json; no reviewer builds ran.

The review confirms strict decode/count/host/Rent precedence, trusted runtime
identity, fixed execution and factual event ordering, stable errors and unchanged
CPI/Rent errors, historical/paused claims, backing/carry/replay/borrows/audit,
unsafe-code prohibition, features and retained marker assertions. No source
correction was required after freeze. Host logs/rollback and future artifact
limitations remain explicit. This is not founder acceptance or an independent
professional audit.

Changed files: ten writer inputs listed above, this report, AGENTS.md, master
specification, execution plan, pilot checkpoint, test plan and Task 2.10 publication
evidence. Implementation commit: `2eeefba0abc226bcfcadddb5f248f12ca589e09d`.
This documentation closure records the hash before normal reviewed integration
publication. No sensitive live action or new live authorization occurred.
