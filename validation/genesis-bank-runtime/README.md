# PIV1 Bank dependency preparation

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
