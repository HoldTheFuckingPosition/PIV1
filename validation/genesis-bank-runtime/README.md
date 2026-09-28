# PIV1 isolated Bank validation

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
