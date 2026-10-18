# PIV1 isolated Bank validation

## Task 2.38 initializer probes through Bank

`tests/genesis_bank.rs` uses the three unchanged Task 2.31 ELFs with real
upgradeable-loader Program and ProgramData accounts. `tests/genesis_fixture.rs`
ports the existing synthetic input recipes and independent literal output/CPI
oracles to the current SDK. No Anchor, PIV1 host library, SPL host processor or
recorded successful result supplies the expected state. See the
[Task 2.38 report](../../docs/TASK_2_38_BANK_GENESIS_INITIALIZATION.md) for actual
execution status and evidence; source assertions alone are not executed proof.

One Rust test iterates four profiles: distinct/shared protocol fee receivers,
each fresh/unpaused or mixed-prefunded/paused. Each profile creates its own Bank
and retained AccountsDB child directory, executes a valid initializer followed
by a malformed outer caller, rejects an exact replay without another fee, then
executes a distinct message containing only the valid initializer. This gives
twelve message cases and four successful initializations if all assertions pass.
All three messages of a profile use the same child Bank and state; no account
reset, cache injection, post-construction account store or deployment instruction
is used.

Genesis creates all three ProgramData accounts at deployment slot zero with the
exact pinned ELF bytes after their 45-byte headers. The callee's synthetic upgrade
authority remains the expected governance vault. A normal child Bank at slot one
makes those programs visible; creating the child freezes the genesis Bank. Clock
time is fixed to 100, all feature flags are inactive, and actual Bank Clock/Rent
bytes are checked. The current Rent encoding uses 6,960 lamports per byte and
threshold bytes representing 1.0, giving the same floors as the old ELF ABI's
3,480 × 2 recipe. SBPF V0 is enabled by this empty feature set.

The new Bank fixture explicitly funds non-target governance/protocol accounts to
their rent floors because zero-lamport accounts are absent from Bank. Target
prefunds retain the old exact values, including the absent PendingSol account and
55/89-lamport excess in future Token accounts. Fresh targets are absent rather
than synthetic zero-balance Account records. Stored genesis fixtures use maximum
rent epoch, as does SVM loading of missing targets. System metadata comes from
the native builtin; the restricted Token wrapper is loaded at its canonical ID.
The read-only synthetic Jito program metadata is never invoked and establishes
no real Jito deployment or execution proof.

The message fee payer, initializer rent payer and executor are separate public
fixtures. Their three required signature slots are all zero placeholders. Each
processed message pays exactly 15,000 lamports, while the exact replay pays zero;
failed initialization rolls back every non-fee account, including the rent payer.
A ComputeBudget instruction at index zero explicitly requests 1.4m CU; the
initializer caller is index one and the malformed caller is index two. Default
heap remains 32 KiB. Ordered CPI records check program IDs, account keys, data and
stack heights, with exact callee/caller success log ordering before the late
failure. Bank's compiled inner records do not expose nested signer/writable
flags, and this harness does not claim that additional evidence or aborted raw
account-state inspection.

Four full snapshots per profile use `Bank::scan_all_accounts` across ancestry,
then independently reread every visible account with the child Bank. Every byte,
owner, lamport balance, executable flag and rent epoch is emitted and compared;
absent targets are explicit. The retained instructions, error, fee and CPI
records are `PIV1_BANK_GENESIS_EVIDENCE` JSON lines. Legacy wire length is measured
and must exceed 1,232 bytes: this deliberately proves unsigned local Bank entry,
not packet transport, ALT lifecycle or valid signed public transactions.

The executable requires the existing `RAYON_NUM_THREADS=1` and fresh absent
`PIV1_BANK_ACCOUNTS_DIR`, plus all nine
`PIV_GENESIS_{CALLER,CALLEE,TOKEN}_{PATH,SHA256,BYTES}` variables. It verifies their
three fixed artifact identities itself before genesis construction. Root supplies
these variables through the reviewed runner. The base and four profile storage
directories remain after execution. Resource settings match Task 2.37; unchanged
upstream four-thread hashing remains separate from one-thread controllable pools.

Six exact existing packages become direct imports: solana-instruction 3.4.0,
solana-sdk-ids 3.1.0, solana-loader-v3-interface 7.0.0 with bincode,
solana-compute-budget-interface 3.0.0, sha2 0.10.9 and bincode 1.3.3. Root verifies
the lock/feature graph before compilation. Historical `bank.rs`, preparation
records, vendor override, probes and earlier workspaces remain unchanged.

## Task 2.37 native System commit/rollback smoke

`tests/bank.rs` adds one bounded Bank/AccountsDB sequence: a successful native
System transfer, a two-instruction transaction whose second transfer fails,
an exact-message replay, and a successful distinct-message retry. All four
messages use the same Bank, blockhash, fee payer and persisted accounts. This
exercises the real local Bank entry, account-saver and AccountsDB read path; it
does not load PIV1 or the Task 2.31 probe ELFs. Actual execution status, exact
commands and evidence belong to [Task 2.37's report](../../docs/TASK_2_37_BANK_COMMIT_ROLLBACK_SMOKE.md).

Five existing Task 2.36 locked packages become direct test dependencies:
serde_json 1.0.151, solana-svm 4.2.0, solana-transaction-error 3.3.2,
solana-instruction-error 2.4.0 and solana-system-interface 3.2.0. Root verifies
the resulting isolated lock and feature graph before build. Historical
`src/lib.rs` and `preparation.json` stay unchanged; their preparation-only
description and hashes refer to Task 2.36.

A bounded compilation attempt exposed an additional host feature requirement:
the already-locked `five8 1.0.0` edge selects `five8_core 0.1.2` through its
published `>=0.1.1,<2` range. That older core exposes `DecodeError`'s `Display`
and `std::error::Error` implementations only with its `std` feature, while the
transitive `solana-keypair 3.1.2` decoder requires an error convertible into a
boxed error. Its separate direct dependency on `five8_core 1.0.0` does not enable
traits on the older type returned by `five8::decode_64`. The isolated manifest
therefore also pins `five8_core = 0.1.2` with only `std` enabled. This is an
explicit host feature unification, not a package upgrade or third-party source
patch. Root verifies the exact metadata delta before rebuilding and preserves
the failed compiler output. No keypair decoder or key-generation helper is
invoked by this harness; compiling a transitive utility is not executing it.

A subsequent compilation completed with a future-incompatible E0365 diagnostic
in `proc-macro-error2 2.0.1`, which the strict build gate rejected. Its existing
hidden module publicly reexports a privately declared `proc_macro` extern crate.
The isolated workspace now overrides that exact dependency with an authenticated
vendored copy whose sole source change is `extern crate proc_macro;` to
`pub extern crate proc_macro;`. This corrects visibility without suppressing the
diagnostic or changing macro expansion logic. Root observed that 2.0.1 was the
newest published registry version; no version upgrade substitutes for the fix.

The full 47-file published archive is retained under
`vendor/proc-macro-error2-2.0.1`, including both licenses, upstream tests, manifest,
VCS metadata and documentation. Forty-six files remain byte-identical; only
`src/lib.rs` has the one-line visibility correction. Cargo-generated extraction
markers are not upstream archive members and are not vendored. The exact archive
SHA-256, upstream VCS commit, original member hashes and changed-source hashes
are recorded in [vendor-provenance.json](vendor-provenance.json). The original
registry archive/source cache is unchanged. This patch affects only this
independent host validation workspace; production and historical workspaces
retain their original dependencies. Root must validate the new local-source
lock/metadata delta and rebuild before claiming the strict gate passes. The root
workspace explicitly excludes the vendor path from membership, keeping it a
dependency rather than enrolling its upstream tests and development dependencies
in this workspace. Its published manifest remains byte-identical.

The executable requires `RAYON_NUM_THREADS=1` and an absolute
`PIV1_BANK_ACCOUNTS_DIR` naming an absent child of a canonical existing directory.
It creates that child exclusively, keeps the directory and performs no cleanup.
Root owns the scrubbed build/runtime environment, fresh target directory,
single-thread test invocation, resource monitoring and retained logs. Do not run
the binary against an old target or an existing AccountsDB directory.

The genesis fixtures use fixed public bytes, creation time 1,800,000,000, no
activated features, rent 6,960 lamports per byte (890,880 minimum for an empty
account), and 5,000 lamports per required signature. Other genesis fields use the
locked SDK defaults, including Development cluster and no explicit program
accounts or feature activation. Their serialized genesis hash is emitted.
There are no nonce accounts or priority-fee instructions. Every unsigned
transaction requires two signature slots, both left as zero placeholders;
processed messages therefore debit exactly 10,000 lamports from a separate
100,000,000-lamport fee payer. The local entry assumes signature verification
already happened; these fixtures do not verify signatures and cannot be submitted
as valid signed public transactions.

The source starts with 10,000,000 lamports and two destinations with 2,000,000
each. A separate untouched account carries 64 deterministic bytes and a distinct
synthetic owner. First a 10,000-lamport transfer persists. The failed message
transfers 70 first, then attempts 1,000,000,000 from the same source; its log must
show a successful first instruction and the second instruction observing
9,989,930 remaining lamports. The exact error must be
`InstructionError(1, Custom(1))`, the native System insufficient-lamports error.
Every non-fee account must return to its complete pre-message value while the fee
payer retains one exact fee debit. Repeating that identical message must return
`AlreadyProcessed` with no account change or second fee. The retry changes only
the second transfer amount to 9, and must persist both transfers from the actual
failed/replayed state with one new fee. The harness never manually stores a
post-message account, resets a cache, clears replay state or reconstructs Bank.

Five snapshots enumerate every slot-zero account (the Bank has no parent) and
independently reread each through `Bank::get_account`. JSON evidence records key,
owner, lamports, executable flag, rent epoch and every data byte for all accounts,
including runtime-created builtins and sysvars. Expected maps compare complete
account values and exact key sets. Message bytes/hashes, zero signatures, logs,
fees, errors and measured compute units are emitted as `PIV1_BANK_EVIDENCE` lines.
This establishes in-process Bank/AccountsDB commit selection and rereads when
executed successfully; it is not disk durability, frozen/rooted Bank behavior,
ledger replay, validator consensus or public-cluster evidence.

AccountsDB foreground/background and index flush pools are explicitly one thread,
with two in-memory index bins and initial capacity 128. Read-cache low/high limits
are 2/4 MiB, with 16 shards and eviction sample four; write-cache limit is 8 MiB.
These are cache settings, not a total RSS bound. The unchanged upstream
asynchronous accounts hasher has **four** threads with 8-MiB stacks; global Rayon
and rewards pools honor the required environment setting. Root separately guards
process-group memory, time and free disk. No third-party source is patched to
reduce a pool or alter Bank semantics.

## Historical Task 2.36 preparation

Task 2.36 prepares an isolated dependency graph for future local Bank/AccountsDB
validation. `src/lib.rs` contains documentation only. No Bank is constructed, no
transaction is signed or executed, and no compilation or rollback claim follows
from Cargo resolution. See [the task report](../../docs/TASK_2_36_BANK_DEPENDENCY_PREPARATION.md)
for actual preparation commands, authenticated inputs and validation status.

The manifest pins Agave `solana-runtime = 4.2.0` and preserves the existing SDK
versions: account 4.3.1, message 4.4.0, transaction 4.1.5, pubkey 4.2.0 and hash
4.5.0. Genesis config 4.0.0 matches the runtime requirement. Address 2.6.1 and
short-vec 3.2.2 retain the existing wincode 0.5 trait boundary. The independent
workspace does not change production or earlier validation workspaces.

`agave-unstable-api` exposes the Bank API. `dev-context-only-utils` exposes the
test constructor without requiring a signed validator bootstrap; it also enables
additional transitive utilities and `solana-program-binaries`. The resolved
program-binaries 4.2.0 manifest has `build = false` and its library embeds bundled
ELFs; it does not have a download build script. The full graph's native build
prerequisites still require review before compilation. Dependency presence does
not authorize executing tests, keypair helpers or build scripts.

## Source-reviewed future API path

The primary runtime source is the published `solana-runtime 4.2.0` crate,
authenticated by root against its registry checksum. Its VCS metadata identifies
Agave commit `ac82b5d438b0c2303dc7169f52c748977713a111`. Line references below
refer to the resolved published sources under
`/tmp/piv1-t236-preparation-20260928-a/cargo-home/registry/src/index.crates.io-1949cf8c6b5b557f/`;
this is a plan, not executed evidence. The linked upstream sources identify the
same API path; actual preparation uses the checksum-bound published crate bytes.

1. Construct `GenesisConfig` directly with explicit synthetic public account
   bytes, time, rent and fee settings. Its `Default`/`new` implementations do not
   create keypairs, although the default creation time is wall-clock dependent.
   Do not call the SDK or runtime `create_genesis_config` helpers: those create
   secret key material. See the pinned
   [GenesisConfig 4.0.0 source](https://raw.githubusercontent.com/anza-xyz/solana-sdk/genesis-config@v4.0.0/genesis-config/src/lib.rs)
   and the resolved crate's `src/lib.rs:87–143`.
2. Use `Bank::new_with_paths_for_tests` (`bank.rs:6910`) with an isolated AccountsDB
   path and explicit public leader, or the lower-level `new_from_genesis`
   (`bank.rs:1254`) with fully specified runtime/database configuration. The
   former also aligns the Bank fee structure with genesis. The dev constructor
   eagerly evaluates `SlotLeader::new_unique` even when a leader is provided
   (`bank.rs:3188`). This creates two public identifiers only: the
   [exact Agave leader source](https://raw.githubusercontent.com/anza-xyz/agave/ac82b5d438b0c2303dc7169f52c748977713a111/leader-schedule/src/lib.rs)
   (resolved leader-schedule 4.2.0 `src/lib.rs:28`)
   calls `Pubkey::new_unique`, which is the pinned address 2.6.1 public
   counter/hash implementation (`src/lib.rs:243`), not a keypair generator.
3. Build a legacy message from fixture public keys with a valid Bank blockhash,
   then `Transaction::new_unsigned` (transaction 4.1.5 `src/lib.rs:329`). This
   fills the required signature slots with default bytes; it does not sign.
   `Bank::prepare_entry_batch` (`bank.rs:3633`) constructs a locked, sanitized
   `RuntimeTransaction` batch. Its
   [SDK transaction adapter](https://raw.githubusercontent.com/anza-xyz/agave/ac82b5d438b0c2303dc7169f52c748977713a111/runtime-transaction/src/runtime_transaction/sdk_transactions.rs)
   (resolved runtime-transaction 4.2.0
   `src/runtime_transaction/sdk_transactions.rs:78`)
   sanitizes message/signature structure and computes message metadata, without
   cryptographic signature verification. This local entry assumes prior signature
   verification; unsigned fixtures cannot establish signature-validation coverage.
4. Call `Bank::load_execute_and_commit_transactions` (`bank.rs:4506`) with explicit
   recording settings. It calls `commit_transactions`, which selects account
   writes through `account_saver::collect_accounts_to_store` and invokes real
   `Accounts::store_accounts_seq` (`bank.rs:4334`). Re-read every fixture account
   with `Bank::get_account` (`bank.rs:5023`), which loads from the Bank's AccountsDB
   ancestry. Do not substitute Mollusk's returned account vector, a mocked store,
   simulation-only execution or manual post-execution `store_account` calls.

## Required future assertions and limits

Successful execution must persist the complete expected custody/state accounts.
An executed failure selects rollback accounts (`account_saver.rs:82–108`): fees
remain charged, and durable nonce transactions can also retain nonce changes.
Fee subtraction precedes rollback capture (`solana-svm 4.2.0`
`src/transaction_processor.rs:750–761`). The first harness should avoid durable
nonces, keep the synthetic fee payer separate, and assert its exact fee debit
independently of unchanged program custody accounts. Whole-account equality
including the fee payer would be the wrong rollback oracle.

Processed failures enter the status cache (`bank.rs:3485`), and repeated message
hashes can return `AlreadyProcessed` (`bank/check_transactions.rs:274`). A retry
must use a deliberately distinct message, such as a reviewed changed compute
budget, while proving the exact custody input persisted from the failed attempt.
Do not clear the status cache to hide replay behavior.

Future tests must load the three unchanged Task 2.31 ELFs under their documented
synthetic identities and verify their hashes; pin genesis features, program loader
accounts, instruction/Clock/Rent bytes, fees, heap and compute limits. The Bank
constructor applies genesis features and builtins (`bank.rs:6050`), so prior
Mollusk compute observations cannot automatically become Bank failure oracles.
Record complete pre/post account data and originating errors/logs before claiming
commit or rollback evidence. Bank account persistence is not ledger replay,
restart durability, validator consensus or public-cluster execution.

Actual Squads execution/control, production Token behavior beyond the restricted
fixture wrapper, recipient control, funding provenance and Testnet/Mainnet
readiness remain outside this preparation. No live keys, signatures, RPC, fund
movement, deployment or authority transfer is involved. Native dependency builds,
tool prerequisites and the required peak disk/memory budget remain unproved until
a separately reviewed bounded build; the earlier 8-GiB reserve is not a measured
Bank build requirement. AccountsDB construction creates storage paths and Rayon
pools (`solana-accounts-db 4.2.0`, `src/accounts_db.rs:1038`), including 8-MiB
foreground thread stacks. A future run needs fresh explicit storage paths and
reviewed bounded thread/cache settings; test defaults alone are not a capacity
bound.

## Task 2.51 production delayed lifecycle

Runtime attempt A reached the real Stake minimum query, then failed with
`UnsupportedSysvar`: the synthetic genesis omitted EpochRewards. The pinned
Stake 5.1 program queries it before dispatch; Agave propagates the missing cache
entry as a host syscall error. The fixture now calls the existing runtime
`add_genesis_epoch_rewards_account` helper and checks the entire rent-backed,
Sysvar-owned, inactive 81-byte account after Bank initialization. That correction
activated no feature and injected no CPI return. Preserve
attempt A's source bytes, binary and complete execution evidence.

Runtime E later reached both real withdrawal/deactivation legs, then Stake's
epoch-1 Withdraw failed with `unsupported BPF instruction`. The unchanged Stake
5.1 ELF imports `sol_get_sysvar`, used for cooldown history; Agave 4.2.0 registers
that syscall only under SIMD-0127. The previous empty feature profile omitted it.
Runtime F enabled exactly SIMD-0127 and passed that capability assertion, then
all twelve full Stake withdrawals returned the expected `InsufficientFunds`.
Its Bank history cooled at the legacy 25% rate and reached zero deactivating
stake by epoch 5, while Stake 5.1 always uses the 9% rate from epoch zero. The
withdrawal calculation retained a positive remainder when the faster Bank
history reached zero. Increasing the wait alone does not repair that mismatch.

The fixture now calls the pinned runtime's `genesis_utils::activate_feature` for
exactly these three Stake 5.1 compatibility requirements:

- `get_sysvar_syscall_enabled` (SIMD-0127),
  `CLCoTADvV64PSrnR6QXty6Fwrt9Xc6EdxSJE4wLRePjq`;
- `reduce_stake_warmup_cooldown`,
  `GwtDQBghCTBgmX2cpEGNPxTEBUTQRaDMGTr5qychdGMj`;
- `upgrade_bpf_stake_program_to_v5_1`,
  `s51VGwCAgebo2745DSUris72RavoLkXGUmVJosESCXr`, which selects the Bank's
  fixed-point stake arithmetic to match the loaded Stake 5.1 program.

The helper creates canonical feature accounts already active at slot zero.
Bank applies these genesis features without the epoch-boundary upgrade path;
that path considers pending feature activations, not these `Some(0)` accounts.
No upgrade buffer, replacement executable, hand-built feature serialization or
new dependency is introduced. Root, initial child, each epoch warp and its child
assert the exact three-feature active set, activation slots, capability flags,
rate activation epoch, complete canonical feature accounts and unchanged Stake
Program/ProgramData accounts, including authority. Each emits `feature_profile`
evidence. EpochRewards remains inactive; its helper activates no features.

The bound is fourteen epochs, derived from the pinned integer recurrence and
asserted fixture: the two deactivating stakes are 5,000,000,000 and 3,608,636,361
lamports; retained effective stake is 3,391,363,639. For each leg remaining `r`,
previous total deactivating `D` and effective `E`, subtract
`max(1, min(r, floor(r * E * 900 / (D * 10000))))`. At epoch 13 the remaining
stakes are 75,545,467 and 54,523,224; at epoch 14 both reach zero. This bounds
the attempts; actual full Stake withdrawal remains the readiness oracle.
Preserve E and F source, artifacts and failure evidence. The fixture correction
is not proof of a completed lifecycle until root executes the lifecycle messages
and full economic/error oracles successfully.

Runtime G then passed the three-feature assertions, the fourteen-epoch cooldown
and both real finalizations, including rent recovery and account closure.
Settlement correctly entered `RecoveryRequired`: projected protected value was
121,109,545,457 lamports versus required HWM 121,109,545,458. Its complete account
comparison changed only the round's lifecycle/recovery flag and the payer's
5,000-lamport fee. Preserve G as valid evidence of the exact one-lamport recovery
case; no beneficiary payments or recovery completion are claimed.

The fresh success fixture explicitly adds a second external economic stimulus
before settlement: a real System transfer of three lamports to the pool reserve,
followed by the actual SPL update. With 111,108,750,001 principal tokens and pool
ratio 1,191,391,363,639 / 1,092,108,750,001, three reserve lamports are the minimum
increase that raises held floor value by one lamport. The test asserts that two
would not, checks the complete account map and leaves the full frozen round
unchanged. This declared appreciation supplies the missing protected value; it
does not increase current-cycle payouts, alter HWM rules, reset G's recovery or
write PIV economic state. All settlement, integration and claim oracles remain.
This fresh run still requires actual Bank validation; preserved G did not pass
the complete lifecycle.

The separate `production_lifecycle` target loads the reviewed current PIV1 ELF,
the real published Token 8.0.0 and SPL stake-pool 2.0.3 executables, and the
checksum-bound Core BPF Stake 5.1.0 artifact. It retains the existing explicitly
synthetic Squads caller for approved genesis transport; it proves neither real
Squads execution nor actual governance/recipient control. The older initializer
harnesses and their restricted Token artifact remain unchanged.

The new genesis contains consistent synthetic external pool/Mint/reserve and two
canonical bootstrap-active validator Stake accounts. Their voter references are
public fixture addresses, not initialized Vote accounts; no validator setup,
voting, rewards inflation or external deployment is claimed. All sixteen PIV1
accounts are absent. The real production initializer creates them. Subsequent
transactions execute real SOL/JitoSOL intake, initial bootstrap, protected
principal SOL staking, guardian activity, withdrawal preparation, two protected
SPL withdrawal/deactivation legs, inactive finalization, beneficiary/KIF
settlement, pending integration and earned KIF claim. Direct System reserve
donation followed by real SPL pool maintenance supplies the explicit local
valuation stimulus; it never writes PIV1 economic state.

Full account maps are scanned and independently reread through Bank after each
step. Independent byte encodings and integer arithmetic cover changed custody,
Config, headers, reward records, temporary accounts, Token/Mint, pool/list and
Stake state. Late outer failures after real CPIs compare every non-fee account
with its prior value; exact replay charges no additional fee and distinct retries
use unchanged custody. Active finalization and any intermediate cooldown rejection
must originate from real Stake full-withdrawal `InsufficientFunds`, with the exact
outer error and nested instruction evidence. No unrelated failure counts as
inactivity. Out-of-order finalization must recover original rent and remove both
Stake and metadata accounts before settlement. Bounded real child-Bank epoch and
StakeHistory progression supplies inactivity; no economic account is manually
stored or rewritten after genesis.

The explicit local entry remains unsigned and bypasses cryptographic verification.
Wire sizes are recorded but public packet transport is not established. The
transaction requests a 262,144-byte heap frame and at most 1.4 million CU; emitted
executed CU measures consumed units, whereas requested heap is a configured bound,
not allocator peak usage. PIV1 retains its compiled default 32-KiB allocator;
a larger VM request does not enlarge that allocator automatically. Only the
three explicit Stake compatibility features above are active; every other
feature remains inactive.
This is a synthetic local profile, not current Testnet feature equivalence.
One complete delayed cycle does not prove every supported branch or successive
cycle cadence. Exact executed status and any concrete blockers belong to the task
report and retained root-run artifacts; source assertions are not execution proof.

Root supplies all fifteen `PIV_LIFECYCLE_{PIV,TOKEN,POOL,STAKE,CALLER}_{PATH,SHA256,BYTES}`
variables, `RAYON_NUM_THREADS=1`, and a fresh absent `PIV1_BANK_ACCOUNTS_DIR`. The
new host target reuses the existing pinned Bank dependency graph and bounded
compiler cache; the separate external-program workspace has its own reviewed lock.
No live RPC, deployment, key generation, signature, fund movement or authority
transfer occurs. Preserve all historical artifacts and recovery archives.


## Task 2.51 bounded production pause profile

The lifecycle fixture additionally exercises D-032, G-004 and K-012 through the
unchanged production PIV ELF. A separate rebuilt synthetic caller admits only the
exact 16-account explicit `PIV1PS01` profile in addition to its unchanged strict
34/35-account initializer profile. It forwards the complete stored instruction
and invokes with the canonical Squads vault PDA seeds. That PDA is never a
transaction signer. PIV authenticates the real caller stack, Instructions sysvar,
current authority/member snapshot, stored approval payload and exact privileges.

Sixteen distinct synthetic approved proposal/transaction pairs (indices 20–35)
are created only in genesis, with the synthetic current transaction index set to
35. The initializer still uses index 19. These fixtures are not live governance:
the synthetic caller does not consume proposals or prove real Squads replay
protection. Bank identical-message replay rejection is reported separately.
No runtime account store, PIV state injection or raw pause-byte mutation occurs;
expected-map byte construction is an assertion oracle only.

Eight explicit pause/unpause pairs bracket otherwise valid lifecycle states.
Nine economic instructions must reject with exact error 6035 before any CPI:
explicit SOL and Jito intake, bootstrap, principal staking, withdrawal preparation,
leg initiation, mature-leg finalization, settlement and pending integration.
Complete account maps must show only retained transaction fees, followed by
identical no-fee replay. A late outer failure after a successful real pause CPI
must roll back Config and all other non-fee accounts, followed by a distinct
successful retry. Unpaused economic operations retain the complete positive cycle.
The mature finalization probe occurs only after the companion leg has actually
withdrawn successfully under the real Stake program.

While paused, actual direct System/Token transfers contribute one lamport and one
token unit; production pending reconciliation records both without changing HWM.
Subsequent explicit deposits are reduced by those exact amounts, preserving the
original 100 SOL / 19 JitoSOL bootstrap economics. Guardian heartbeat and the
full previously earned KIF claim succeed during pause with exact account maps.
The final unpause changes only Config's flag and transaction fees. These checks
do not cover unimplemented migrations, real Squads proposal consumption, live
transport/signatures, cooldown-loss or repeated-cycle/carry profiles. Actual run
status remains in the Task 2.51 report; source assertions are not execution proof.


## Task 2.51 ordinary cooldown-loss profile

The default runner profile remains `--profile positive`, selecting exactly
`complete_delayed_production_lifecycle`. Its complete positive/pause lifecycle,
economic quantities and account oracles remain unchanged. The same reviewed Bank
binary also contains `cooldown_loss_recovery_production_lifecycle`, selected only
by `--profile cooldown-loss`; it shares the real initialization, two-leg creation,
pause checks and canonical epoch/fork progression.

Only the loss profile uses an explicit external fixture mutation. Immediately
after both real protected withdrawals/deactivations, before actual Bank cooldown,
`Bank::store_accounts` supplies a declared balanced pair: temporary
Stake leg 0 loses one lamport and the existing external System donor receives one.
Full before/after maps require every owner, executable flag, rent epoch and data
byte unchanged, including the complete Stake delegation and every PIV account.
Both aggregate account lamports and Bank capitalization must remain unchanged;
original Stake rent remains fully covered. This is synthetic loss input to the
production boundary, not a production transaction or proof of live slashing.
The original positive profile never performs this mutation.

Real Stake withdrawal still determines inactivity. The unaffected companion leg
finalizes first at epoch 14. A late outer failure after the loss-bearing finalization
proves exact rollback to the already loss-adjusted input, retained transaction fee
and identical no-fee replay. A distinct retry must execute real Stake/System CPIs,
close both remaining temporary accounts, recover the exact original rents, and
commit only the expected finalization accounting and `RecoveryRequired` with
cooldown-loss flag 1. Config/HWM, pending contributions, beneficiaries, guardian
rewards and KIF balances remain unchanged. Subsequent settlement, integration and
finalization calls reject with exact 6073 before CPI; their complete account maps,
fees and identical replay are checked. No recovery exit/reset policy is invented.

The distinct `cooldown_loss_complete` marker requires two finalized legs, epoch
14, loss 1, state 4/flags 1, unchanged HWM 119,000,000,000, pending SOL 300,000,000
and tokens 500,000,000, original Stake rent 4,565,760, metadata rent 5,442,720,
four closed temporary accounts and zero beneficiary/KIF payments. The runner also
requires exactly one explicit balanced synthetic-shock record. Unknown profiles,
wrong test names, mixed/duplicate completion markers, malformed evidence and
weakened accounting/provenance claims reject. Historical artifact, source, binary,
resource, offline and Git guards remain unchanged.

This bounded case covers ordinary P-035 native cooldown loss. It does not cover
T23-R1's separate severe pool-loss boundary where retained value is below committed
pending SOL use; the distinct profiles below address that scope. The loss profile stops in
recovery without settlement/pending integration/KIF completion and cannot be
reported as a successful economic distribution. Actual executions and limitations
are recorded separately in the Task 2.51 report.

## Task 2.51 severe external pool-loss profiles

Two additional exact runner selectors share the same production artifacts and
setup: `--profile early-pool-loss` selects
`early_pool_loss_recovery_production_lifecycle`, and
`--profile settlement-pool-loss` selects
`settlement_pool_loss_recovery_production_lifecycle`. The default positive profile
and the ordinary cooldown-loss profile retain their original control paths and
account oracles. This section describes assertions; actual execution evidence
and validation status belong to the Task 2.51 report.

At epoch 14, a declared external fixture shock reduces the pool reserve and both
remaining active validator Stake sources to their original 2,282,880-lamport rent
floors. `Bank::store_accounts` supplies a balanced four-account group, crediting
the existing external System donor exactly 1,191,386,797,879 lamports. Complete
before/after maps require every data byte, owner and account flag unchanged,
including all PIV accounts, both withdrawn legs and the original delegations.
Aggregate account lamports and Bank capitalization remain unchanged. This is
synthetic loss input, not a production loss instruction or proof of live slashing;
the deficit source accounts are not claimed to be healthy delegated accounts.

Real pinned SPL validator-list and pool maintenance then publishes the reduced
ratio. The assertion oracle permits only the exact list balances, pool total and
transaction fee to change. Remaining validator rents produce pool total 4,565,760
against unchanged supply 1,092,108,750,001; retained principal token value is
464,510. Reserve-only loss would leave token value 345,494,412, above committed
pending SOL use of 100,000,000, so it cannot prove the intended severe boundary.
Neither pool accounting nor PIV valuation is hand-written into runtime state.

The early profile shocks the external pool before the first finalization. Real
Stake finalization of leg 1 still returns original Stake rent 2,282,880 and metadata
rent 2,721,360 and closes those two accounts. The direct residual-HWM predicate
commits `RecoveryRequired` with flag 2 and no cooldown loss. Complete maps preserve
the outstanding leg 0 Stake (5,002,282,880 lamports) and metadata (2,721,360), their
data, Config/HWM, pending balances, beneficiaries and earned KIF. Residual value is
checked against the stored residual HWM floor. This finalizer has no pending-use
subtraction; the early profile does not claim T23-R1 zero-floor execution.

The settlement profile first finalizes both legs normally and recovers all original
Stake rent 4,565,760 and metadata rent 5,442,720. It applies the external pool shock
before the positive profile's appreciation. Projected retained value is exactly
464,512 (token value plus two lamports of escrow remainder), below committed
pending use 100,000,000. Actual production settlement must succeed with no CPI or
payment, changing only the round header to recovery state 4/flag 2 and retaining
the transaction fee. The exact 57-byte `DistributionSettlementRecovery` event
from actual program logs must report protected comparison value zero. Both the
rolled-back attempt's event and the successful retry's event are checked against
Config, sequence, discriminator and flags; emitting an event alone never proves
that the attempted state committed.

Each profile includes a late outer failure after the relevant real production
operation, complete rollback with retained fee, identical no-fee replay and a
distinct successful retry. Subsequent settlement, integration and finalization
reject with exact 6073 before CPI, with complete maps and replay checks. No
recovery exit or governed treatment of the outstanding leg is invented. HWM
119,000,000,000, pending SOL 300,000,000 and pending tokens 500,000,000 remain intact;
no beneficiary or KIF payment occurs. Distinct `early_pool_loss_complete` and
`settlement_pool_loss_complete` records bind these outcomes. The runner rejects
cross-profile evidence, wrong types or amounts, missing conservation/provenance,
and missing, malformed or inconsistent attempted/committed zero-floor events.

### Repeated liquid cycles, cadence and collective KIF carry

The isolated `repeated-cycles` runner profile selects only
`repeated_cycles_production_lifecycle`. It requires real production initialization,
SOL/Jito intake and bootstrap, followed by three liquid-funded distributions through
`PIV1PD01`, settlement and pending integration. The earlier positive, pause and loss
profile bodies remain unchanged; retained O/P/Q evidence covers their delayed Stake
paths. This compact profile loads the same five ELFs but does not execute Stake.
Its actual calls exercise PIV, the synthetic governance initializer caller, native
System, Token 8.0.0 and SPL pool 2.0.3; loaded/executed programs are reported separately.

Clock progression is an explicitly synthetic input through the pinned canonical
`Bank::set_sysvar_for_tests` helper. Complete before/after account maps require only
Clock's `unix_timestamp` bytes to change; slot, epoch, epoch-start timestamp,
capitalization and every economic account remain exact. This is not elapsed real
time or an instruction argument supplied to PIV. PIV reads the real Clock account
and syscall cache. The first preparation at 1,728,000 settles at 1,814,400. The next
preparation rejects 6038 at 2,591,999 and succeeds at 2,592,000, exactly ten days after
preparation and only nine after settlement. A simultaneous active-round attempt
rejects 6034. Both rejections require no CPI, fee-only full maps and identical
no-fee replay.

Bootstrap recognizes 19 billion Jito units and one SOL, with HWM 20 SOL. For each
round, an external donor transfers 100 SOL into the actual pool reserve followed
by the actual pool update, and explicitly contributes two SOL to pending custody.
The resulting 1.9 SOL yield allocates 1.121 SOL HTFP, 0.3705 SOL Team, 0.3705 SOL
ordinary permanent compound and 0.038 SOL current KIF; outgoing is 1.5295 SOL.
Pending retains its full two-SOL contribution ledger while physically funding that
outgoing amount; remaining 0.4705 SOL integrates later and full two SOL raises HWM.
These are declared local reserve stimuli, not inflation or live-yield proof.

Periods zero and one have zero active guardians in their frozen snapshots. KIF
compounds 19,000,000 then 28,500,000 lamports, carrying the same respective amounts.
The second allocation reapplies 50/50 to current 38,000,000 plus prior 19,000,000.
Period two snapshots three active guardians: current KIF plus carry is 66,500,000,
so each earns 22,166,666 and two lamports remain collective carry. Heartbeats after
the first and third snapshots cannot retroactively create eligibility. Full account
oracles cover all rewards, custody, Config, optional prior-completion serialization,
monotonic sequence and exact HWM 22,389,500,000 / 24,788,500,000 / 27,159,000,000.
Final protected native principal is 2,459,000,000 plus 19 billion Jito units valued
at 24.7 SOL; pending is zero, earned KIF liability 66,499,998 and collective carry two.
Operational rent and all unused external accounts remain protected.

Preparation, settlement and integration each have a selected late outer failure,
full rollback with retained fee, identical no-fee replay and distinct successful
retry. These are not callee-internal mutation failures. Strict runner contracts
reject missing/mixed profiles, changed arithmetic, nested wrong types, missing
Clock provenance and altered guard outcomes. These are source-level assertions
until actual reviewed execution receipts establish their results; consult the
Task 2.51 checkpoint for execution status. No production bytecode, dependency,
accepted economics, deployment or M5 work is changed by this profile.

### Valid technical insufficiency and its independent retry clock

The isolated `valid-insufficiency` runner profile selects only
`valid_insufficiency_production_lifecycle`. Real initialization, intake, bootstrap
and a completed liquid distribution establish an existing preparation clock at
864,000, HWM 22,389,500,000, 19 billion principal token units, 1,489,500,000 native
principal and 19,000,000 collective KIF carry. A heartbeat after that first snapshot
cannot earn retroactively; it remains eligible for the later period-zero snapshot.
The existing delayed, pause, loss and repeated-cycle source remains unchanged.

Declared ordinary donor transfers to the reserve and actual pool updates raise
pool total from 1,100 to 1,150 SOL, against 1,000 billion recorded token units.
The resulting 950,000,000 gross yield has outgoing 764,750,000 and computed target
665,000,000 token units. The real pinned Stake GetMinimumDelegation response is
one SOL, requiring 869,565,218 token units. Production withdrawal preparation
therefore records a valid insufficient attempt without creating a snapshot,
funding escrow, reserving rent, reclassifying custody or changing the ten-day clock.

Canonical synthetic Clock inputs retain slot one and epoch zero; full maps require
only Clock timestamp bytes to change and the syscall cache to agree. Preparation
rejects 6038 at 1,727,999, one second before ten days, and records insufficiency at
1,728,000. Immediate and +86,399-second attempts reject 6039. The exact +86,400 retry
records 1,814,400. Invalid ABI version attempts before the first attempt and before
the exact-boundary retry reject without changing either timer. All these cases
include complete fee-only or Config-timestamp-only maps, identical no-fee replay,
and distinct transaction identities. Cadence/version rejection has no CPI;
insufficiency/cooldown evaluation performs exactly one read-only Stake minimum
query, with its instruction bytes and actual eight-byte return log asserted.
The two successful insufficient outcomes alone emit the exact 64-byte factual
`DistributionPreparationInsufficient` event, checked against actual program logs.

Another ordinary 50-SOL reserve transfer and pool update raises gross yield to
1.9 SOL. An explicit one-lamport pending contribution makes the independently
rounded retained principal exactly meet proposed HWM, preserving the existing
partition-floor guard. At 1,814,401 the same production withdrawal ABI successfully
prepares despite the last insufficient attempt being only one second old. Target
is 1,274,583,333, technical minimum 833,333,334, one useful leg and protected native
floor 1,529,347,049. Pending's single physical lamport funds escrow while the full
pending ledger remains one. Existing HWM stays 22,389,500,000; proposed HWM is
22,760,000,000. The prior completion, active guardian snapshot, KIF carry, complete
new round bytes and every other account are checked exactly.

This timing profile ends in WithdrawalActive with sequence one and no leg created;
it does not claim a second completed distribution. The same five ELFs are loaded;
Stake executes only its minimum query. Synthetic time, genesis/governance and
unsigned transport limits remain. No inflation proof, deployment, production
change or M4 completion follows from source assertions; actual execution/review
receipts in Task 2.51 establish the achieved scope separately.

## Task 2.51 positive cooldown reward profile

`cooldown_reward_production_lifecycle` is an isolated P-035 profile selected by
`--profile cooldown-reward`. It preserves all earlier Rust profile bytes. Actual
initialization, deposits, staking, preparation and two protected withdrawals
precede a declared synthetic external reward input: the existing donor loses
10,000 lamports, withdrawn Stake leg zero gains 10,000 lamports and its delegated
amount gains 10,000. Complete maps allow only those changes; authority, original
metadata receipts, rent, credits observed and every PIV-owned byte stay exact.
Canonical Bank storage updates the stakes cache. This abstracts an accumulated
reward observation; it is neither a direct donation nor native inflation/vote-credit
execution. No production instruction or live operation creates this input.

The input precedes the active-Stake rejection and actual Bank cooldown. The pinned
9% fixed-point recurrence predicts remaining legs [75,547,234,54,524,390] at epoch 13
and [0,0] at epoch 14; only the real Stake withdrawal decides readiness. Original
Stake and metadata rents return separately. The first fixed distribution excludes
all 10,000 reward lamports; full pending integration retains them as explicit
next-cycle yield. The separate three-lamport external reserve appreciation from O
remains necessary and is not the withdrawn-Stake reward.

A declared canonical Clock input at timestamp 2,592,000 and a period-one heartbeat
precede a liquid second round. Historical value equals HWM 121,955,000,003 exactly,
so its pure 10,000 reward receives 5,900 HTFP, 1,950 compound, 1,950 Team and 200
earned KIF. Completing settlement/integration retains HWM 121,955,001,953, no next
yield or pending contribution, and cumulative reward 10,000 exactly once. All
transition maps include unrelated accounts; selected late-outer failures retain
only fees, identical replay charges nothing, and distinct retries succeed.

Source assertions describe intended coverage. Actual execution, review, failures
and preserved evidence are recorded in the Task 2.51 report. Synthetic genesis,
governance/reward/Clock, unsigned transport and supported adapter limits remain.
This profile does not exercise native inflation or claim M4 completion.


## Task 2.51 later principal compounding profile

`post_cycle_compounding_production_lifecycle`, selected by `--profile
post-cycle-compounding`, appends A-004 boundary-six coverage to an isolated copy of
S's three completed liquid cycles. The previous Rust source remains an exact prefix;
S's economic messages, labels, nonce order and complete account oracles are retained.
Its original standalone completion is replaced by the later-compounding completion.
No production code, dependency, original deposit helper or executable changes.

The actual completed Idle state binds historical SOL 2,459,000,000, historical
JitoSOL units 19,000,000,000, HWM 27,159,000,000, prior summary sequence two, KIF
liability 66,499,998 and carry two. At the actual 13:10 pool ratio, an explicit-minimum
ABI helper deposits 2,458,999,998 lamports and requires 1,891,538,460 token units.
Two lamports remain, preserving exact historical book value. Entire account maps
permit only the corresponding SOL/token/pool/mint custody changes, Config's two
historical asset-unit fields and transaction fees. HWM, pending, KIF, original rents,
all other Config fields and the full completed summary remain unchanged.

A caller minimum one unit above quote rejects 6032, and historical SOL plus one
lamport rejects 6028, both before CPI or event emission. Fee-only maps and identical
no-new-fee replays are required. A separate late-outer failure follows a successful
protected pool call and PIV event, rolls back every non-fee change, and has an
identical no-new-fee replay before the distinct successful retry. The actual protected
pool instruction, account order and 72-byte PendingSolStaked event are checked.
This outer failure does not represent an internal-callee failure.

Source assertions specify intended coverage; actual execution and independent
review receipts determine achieved evidence. Synthetic genesis/governance/Clock,
unsigned transport, zero-fee/book-value-preserving adapter limits and the absence
of a live deployment remain explicit. No M4 completion or M5 authorization follows.


## Task 2.51 positive withdrawal-fee profile

`positive_withdrawal_fee_production_lifecycle`, selected by `--profile
positive-withdrawal-fee`, isolates the P-023 fee-through-settlement coverage gap.
External genesis sets only the pool's stake withdrawal fee to 1/1000. Deposit fees
remain zero; no PIV economic account exists before the actual initializer runs.
Existing Rust profiles remain byte-for-byte unchanged. Actual intake, bootstrap,
protected principal staking and preparation fix target 7,891,249,999 token units.

Two protected legs use their fresh pool ratios. Their inputs 4,587,921,256 and
3,303,328,743 charge separate ceiling fees 4,587,922 and 3,303,329. Manager custody
receives 7,891,251 units, one more than rounding an aggregate fee. Only the remaining
7,883,358,748 units burn; native delegated proceeds total 8,600,027,724 lamports.
Real Stake rejects premature finalization, and actual cooldown permits companion
leg one then leg zero at epoch 14. Original Stake and metadata rents return exactly;
all four temporary accounts close. No cooldown reward or loss is injected.

With 100,000,000 pending SOL used, beneficiary net is 8,700,027,724: HTFP receives
6,376,417,834, Team 2,107,460,131 and earned KIF 216,149,757, retaining two dust
lamports. The fixed compound share stays 2,109,545,454; split dust stays two.
An explicit three-lamport external reserve appreciation closes the same one-lamport
settlement floor issue as the positive fixture; projected retained value includes
the two escrow dust lamports. Full pending integration leaves historical value and
HWM exactly 121,955,000,003, pending/next yield zero, and earned KIF unchanged.
This is a bounded supported fee configuration, not every fee regime or live Jito.

Complete account-map oracles cover every transaction. Selected late-outer failures
at fee-bearing initiation, finalization, settlement and integration retain only fees;
identical replays add no fee and distinct retries succeed. These are outer failures,
not the separately stopped internal-callee case. Root execution and separate review
receipts determine achieved evidence; source assertions alone are not passing tests.
Synthetic genesis/governance and unsigned transport limits remain; no M4 completion,
M5 authorization, live deployment, signature verification or founder acceptance is claimed.


## Task 2.51 supported economic-normalization profile

`economic_normalization_production_lifecycle`, selected by `--profile
economic-normalization`, closes the scoped master §7.8 / Task 2.42 production
runtime gap. It appends a standalone zero-fee two-leg cycle, preserving every
previous Rust profile as an exact prefix. Actual initialization, intake, bootstrap,
principal staking and pool maintenance precede fixed preparation and both real
withdrawal legs. No PIV economic state is seeded or edited outside its instruction.

While WithdrawalActive, recognized pending SOL is 300,000,000 but physical pending
SOL is 200,000,000 because 100,000,000 already funds escrow. Historical tokens are
119,000,000,000; cumulative assigned input 7,891,249,999 leaves physical principal
111,108,750,001. Real donor System transfers add principal 11, escrow 13, KIF 17
and pending 19 lamports. Real donor Token transfers add seven principal and four
pending units. These transfers alone do not change any PIV ledger.

The real thirteen-account `PIV1RB01` instruction moves only 11/13/17 native excess
and seven principal tokens, recognizing all 60 native / 11 token units as pending.
Exact four-CPI bytes/order/accounts, actual event bytes and complete account maps
preserve every non-pending Config field, frozen round, obligation, rent and other
account. KIF liability and carry are zero at this particular normalization boundary;
this profile does not claim protection of a positive preexisting KIF debt there.
A fresh repeat emits zero new contribution with no CPI and fee-only account changes.
Selected late-outer failures, identical no-fee replays and distinct successful
retries cover normalization, finalization, settlement and pending integration.

Real Stake cooldown permits out-of-order finalization at epoch 14 and all original
Stake/metadata rents return. The prior positive profile's explicit three-lamport
external reserve appreciation remains necessary and declared; no rounding guard
is relaxed. Current fixed payouts remain HTFP 6,382,727,270 / Team 2,109,545,453 /
KIF 216,363,636, with fixed compound 2,109,545,454. Later integration credits the
full recognized contribution 845,454,617, including pending SOL already consumed
by the cycle. Final historical SOL is 200,000,062 and tokens 111,608,750,012;
book value equals HWM 121,955,000,075 exactly. Pending and next-cycle yield are zero.
The donation to KIF becomes contribution, never an extra earned reward.

The runner has six literal regression groups for profile selection, missing or
duplicate evidence, source/offset mistakes, idempotence/economic misclassification,
typed provenance/order and wrong-profile/unexecuted output. Passing execution and
independent raw-record review must be established by root receipts; source is not
execution evidence. Operational provenance, Token-owned native excess, other
lifecycle combinations and the stopped internal-callee case remain outside this
bounded profile. Synthetic genesis/governance, unsigned transport and previous
adapter limits persist. No M4 completion, M5 work, deployment or acceptance follows.

## Task 2.52 recipient updates in the local production lifecycle

The isolated `recipient-updates` profile uses a strict eighteen-account synthetic
caller route for `PIV1UR01`; the initializer and explicit pause profiles remain
unchanged. This is not a generic governance proxy or the live Squads program.

The profile loads the new Task 2.52 PIV artifact and separately rebuilt caller,
with unchanged pinned Token/pool/Stake binaries. Genesis explicitly provides two
new funded same-multisig System vault PDAs (indices 32/203), one Approved action,
one Executed action and one action with only three approvals. These are synthetic
governance inputs, not real proposals, signatures or exclusive 4/6 spending proof.
The initializer creates all PIV economic state through its production instruction.

Two actual liquid-funded production cycles share one Bank. The first preparation
freezes the old recipients. Executed and three-vote updates reject with exact 6102
after the caller enters PIV, preserving all non-fee accounts. An approved update
then succeeds internally before an ordinary late outer failure; the complete map
rolls back except its fee, exact replay adds no fee, and a distinct retry changes
only Config's future recipient keys. The actual 208-byte `RecipientsUpdated`
payload and sole eighteen-role PIV CPI are checked on both successful-inner attempts.
The update does not include the active round or perform custody CPI.

Settlement supplied the new keys rejects exact 6000. The existing prepared round
pays the old destinations; after pending integration, a later preparation freezes
and settles to the new pair. A selected late outer failure/replay/distinct retry
also covers the second settlement. Both cycles use declared external reserve
transfers and canonical synthetic Clock observations at 1,728,000 / 2,592,000.
Each distributes HTFP 1,121,000,000 and Team Owner 370,500,000 lamports. Zero-active
KIF compounds 19,000,000 then 28,500,000, retaining 28,500,000 collective carry.
Final Idle HWM equals historical value 24,788,500,000, with 1,988,500,000 historical
SOL and 19,000,000,000 pool-token units. Pending and earned KIF liability are zero.
Full account maps preserve rent, guardian records and all unrelated accounts.

The source assertions and runner contract require execution plus separate raw-trace
review before being called evidence. The runner tests reject missing/duplicate/
out-of-order proof rows, wrong frozen destinations, altered event bytes, incorrect
authorization, state preservation, accounting and typed provenance. Complete raw
maps, stored-message privileges, actual events and CPI records require the independent
reviewer audit as well. Bank compiled inner records do not themselves expose nested
signer/writable flags. No Stake instruction is executed in this compact profile.
Synthetic proposal non-consumption, unsigned transport, synthetic Clock and current
supported recipient-identity limitations remain explicit. The platform-blocked
internal-callee-failure operation is separate and is not inspected or resumed.

Task 2.53 guardian synchronization runtime profile is documented below.

The bounded `guardian-synchronization` profile runs three actual production liquid
cycles, with two active guardians. First-cycle earned debt exists before one-seat
synchronization during the second prepared round. Declared synthetic external
Squads membership changes only current multisig bytes; it proves no historical
old-set votes or real Squads execution. The fresh exact approved PIV instruction
creates six revision-bound rewards using protected System rent/allocation CPIs.
Actual donor prefunds and the external rent payer remain separate from PIV custody.
Old reward debt and frozen eligibility survive; a retired signer claims its two
cycles of earned debt. New activity never rewrites the old round, and the third
cycle snapshots/credits the current revision. Selected late-outer rollback,
identical replay and distinct retry cover synchronization and retired claims.
The restricted synthetic caller admits only the exact 24-account guardian ABI.
Prior profiles remain an exact Rust source prefix. No production source, dependency,
layout, allocator or economic change is introduced. This profile is unsigned local
Bank evidence with synthetic genesis/Clock/governance; no Stake execution, live
membership operation, proposal consumption, account closing or old-rent refund is
claimed. Actual results require root execution and separate raw-trace review.
