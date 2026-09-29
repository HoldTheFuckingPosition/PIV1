# PIV1 program and accounting library

This crate combines the founder-accepted Phase 1 state/accounting foundation and
Tasks 2.1–2.2 models with the **COMPLETE / FOUNDER-ACCEPTED** Tasks 2.3–2.19
milestone at `d9f3371be6ecb586675e3b38edcc57bd6e9519f8` (D-027, 2026-09-14).
Acceptance covers the recorded bounded scopes. The authorized milestone
fast-forward is published to main; separately reviewed documentation commits
maintain its acceptance records. Task 2.20 is **TECHNICALLY VALIDATED /
FOUNDER-AUTHORIZED MAIN INTEGRATION (D-028)** for fresh genesis/Jito/all-target
preflight and checked
rent-only shortfalls. It observes current accounts without creation, mutation
or classification of unsolicited prefunding. Task 2.21 is also **TECHNICALLY
VALIDATED / FOUNDER-AUTHORIZED MAIN INTEGRATION (D-028)** for fresh rent-only
allocation from a
distinct signing external payer and exact System CPI postchecks. The allocation
result is undispatched and incomplete: Token initialization and all state writes
must complete in the same successful transaction. Committing bare Token-owned
zero data is unsafe; see [Task 2.21](../../docs/TASK_2_21_GENESIS_ACCOUNT_ALLOCATION.md).
Task 2.22 is **TECHNICALLY VALIDATED / FOUNDER-AUTHORIZED MAIN INTEGRATION
(D-028)** for same-call
allocation, both Token initializations and all nine initial state envelopes with
exact postchecks. At that milestone it remained undispatched, preserving prefund/funding, recipient,
transport and runtime-proof limits. Task 2.22 evidence was 448 host tests +1
doctest/eight gates; see [Task 2.22](../../docs/TASK_2_22_GENESIS_ACCOUNT_INITIALIZATION.md).
Task 2.23 adds host-only unsigned transport encoding/packet evidence with the same
D-028 status: nine Node tests and eight report cases, without actual transport
execution. D-028 authorizes main integration of Tasks 2.20–2.23 at
`3282e1ebabcb0cd88491d48a391565b8b100afa7` plus reviewed records; it does not infer
broader founder acceptance.
Task 2.24 is **TECHNICALLY VALIDATED / PENDING FOUNDER ACCEPTANCE** on integration.
Its distinct same-call normalization API transfers native excess from the two
still-System-owned Token PDAs into PendingSol, preserving original external rent
funding and zero initial ledgers. Subsequent authenticated pending recognition
records that balance once. Existing raw APIs remain compatible; later Token-owned
native donations remain unsupported. See [Task 2.24](../../docs/TASK_2_24_GENESIS_TOKEN_PREFUND_NORMALIZATION.md).
Task 2.25 [recipient identity preflight](../../docs/TASK_2_25_GENESIS_RECIPIENT_PREFLIGHT.md)
is **TECHNICALLY VALIDATED / PENDING FOUNDER ACCEPTANCE** on integration. It
freshly verifies both approved recipients as funded System vault PDAs of the
same governance multisig without modifying accounts. Exclusive four-of-six
spending, delegated limits and live control remain separate checks.
Task 2.26 [recipient-checked initialization](../../docs/TASK_2_26_RECIPIENT_CHECKED_GENESIS_INITIALIZATION.md)
shares that pending-acceptance status on integration. Its fixed normalized path
checks recipient identity before effects and preserves both accounts after every
successful CPI and final completion, using one fresh full preflight.
Task 2.27 [recipient-checked transport](../../docs/TASK_2_27_RECIPIENT_CHECKED_GENESIS_TRANSPORT.md)
is **TECHNICALLY VALIDATED / PENDING FOUNDER ACCEPTANCE** for that complete
35/34-account fixture, with an explicitly selected unsigned host profile. Actual
transport lifecycle and current runtime proof remain deferred.
The [checkpoint](../../docs/PIV1_PILOT_STATE.md) and
[integration review](../../docs/PIV1_INTEGRATION_REVIEW_2_3_TO_2_19.md) define the
current scope; individual reports preserve earlier evidence and limitations.

The `lib`/`cdylib` crate uses a thin native entrypoint with the actual runtime
Program ID. It does not invent a static `declare_id!` or deployed identity.
Ten instruction paths are dispatched:

- `claim_kif`: exact 24-byte data and five accounts; authenticated earned-liability
  accounting, existing-account persistence and a fixed signed System transfer to
  the guardian. It remains available during pause and does not create rewards.
- Pending-contribution recognition: exact eight-byte data and four accounts;
  authentication and idempotent updates to the two Config pending ledgers,
  including during pause/recovery. It makes no transfer or CPI.
- `initialize_piv1`: exact 315-byte `PIV1IN01` version-1 data and fixed 35/34
  accounts. It performs fresh exact-message Squads/bootstrap, guardian, protocol
  and recipient checks, funds original rent shortfalls, normalizes native Token
  prefunds, initializes both Token accounts and writes all nine state envelopes.
  Recipient vault-index witnesses are authenticated bytes, not fixed 0/255 policy.
  Every CPI/runtime error propagates; no claim event is emitted.
- `guardian_heartbeat`: exact 11-byte `PIV1HB01` version-1 data (guardian slot,
  governance vault-index witness), 13 accounts. Current registry/reward bindings,
  runtime Clock/Rent, fresh Squads membership/upgrade-vault correspondence and
  the guardian signature authenticate a current-period activity write. It works
  during pause, is idempotent within a period and rejects time regression. It
  credits no liability and cannot change a previously recorded distribution.
- `set_pause`: exact 11-byte `PIV1PS01` version-1 data (governance vault witness,
  strict Boolean), 16 accounts. Complete current Squads invocation authentication
  binds the original approved bytes and accounts before changing only the Config
  pause byte. Explicit same-value setting is idempotent; no toggle, transfer,
  new replay receipt, guardian rotation or recipient replacement is introduced.
- `deposit_sol` and `deposit_jitosol`: exact 17-byte `PIV1DS01`/`PIV1DJ01`
  version-1 data followed by a little-endian u64 amount, with six/eight accounts.
  D-032 rejects both during pause before any transfer or accounting change. The
  canonical System transfer or legacy Token `TransferChecked` moves only the
  requested amount into pending custody; no conversion or Jito pool CPI occurs.
- `reconcile_untracked_balances`: exact nine-byte `PIV1RB01` version-1 data
  and 13 fixed accounts. Proven economic surplus moves into pending custody and
  is recognized once; each vault first covers its own obligations. It supplies
  the production normalization prerequisite for initial bootstrap.
- `bootstrap_initial_contributions`: exact nine-byte `PIV1IB01` version-1 data
  and fixed 19/18 accounts. It moves all recognized initial pending SOL/JitoSOL
  into principal and establishes the HWM from authenticated current pool book
  value. It rejects pause, noninitial history, unnormalized custody and replay.
- `stake_pending_sol`: exact 25-byte data: `PIV1SP01` version 1 followed by
  little-endian u64 SOL amount and stronger caller minimum; fixed 20/19 accounts.
  It converts recognized historical SOL through the protected SPL deposit CPI,
  preserving historical book value and HWM with zero actual deposit fees.

Ordinary host entrypoint calls reject execution. Explicit host seams model
context, invocation and rollback. The existing claim callback seam cannot execute
initialization, guardian operations, deposits, economic normalization or initial
bootstrap or staking. Separate host seams model their context
and effects. Heartbeat/pause use the existing atomic envelope commit and emit
factual events only after success; logs still require successful transactions.
Other markers remain unimplemented, including distribution and remaining
governance updates. Already-received direct transfers remain reconcilable during
pause. The legacy
313-byte `PIV1GM01` model codec
remains unchanged and is not accepted as a native initializer ABI. Both internal
formats authenticate their entire original input; no reconstructed legacy bytes
replace the actual native approval. See [Task 2.39](../../docs/TASK_2_39_PRODUCTION_INITIALIZER.md)
for exact roles, error assignments, validation status and remaining limits.

The heartbeat account order is Config, registry, six slot-ordered rewards, Clock,
guardian signer, runtime PIV1 Program, ProgramData and current Squads multisig.
Only the selected reward needs writable access; no wallet-owner/on-curve rule is
added. Pause uses the existing initialized Squads order: Program, ProgramData,
multisig, proposal, transaction, governance vault, Instructions, Config, registry,
six rewards and Clock; Config must be writable. Neither path receives the active
distribution or custody accounts. Existing error assignments remain unchanged.
The Task 2.40 scope targets host validation and strict production SBF compilation;
actual runtime execution of these new paths remains unproved until the local
production lifecycle work. M2 and complete Testnet readiness remain incomplete.

Both deposits start with Config, active distribution, PendingSol and PendingJito.
SOL appends a writable signing System-owned empty-data donor and executable System
program; Config and PendingSol are writable. JitoSOL appends a writable source
Token account, signing Token owner, configured readonly initialized Mint and
executable legacy Token program; Config and PendingJito are writable. Sources
cannot be PIV1 custody or its authority. The Token profile supports the owner
signer, preserving a distinct delegate and close authority. If owner and delegate
coincide, SPL 8 consumes the delegated allowance and clears only its tag when
exhausted; that exact result is checked. Third-party delegate authorization and
SPL multisig owners are outside this bounded ABI. No general wallet-owner or
on-curve requirement is added. Native donors may be fully depleted; transaction
rent-state checks remain the runtime's responsibility.

Before either CPI, both old pending obligations must be covered: pending SOL
minus the active round's committed SOL use, and pending JitoSOL units. Arithmetic,
rent, identity, signer/writable and required buffer borrows are checked first.
Prior untracked surplus and Token-account native excess remain unclassified.
After the fixed CPI, every supplied account's full metadata and borrowed SHA-256
data fingerprint must match its exact predicted result. Only authenticated fixed
Token buffers are copied; arbitrary authority/program data is not heap-copied.
The typed Config write then records only the new amount, preserving history,
HWM, rent, KIF and the complete active distribution. Events follow commit;
successful repeated instructions are new contributions, not idempotent receipts.
Failures propagate for transaction rollback. The explicit host seam neither
undoes a partial CPI nor proves Bank/SBF execution; focused host tests use pinned
SPL Token code and explicit staged discard/retry, with native execution deferred
to the complete production lifecycle milestone.

Economic normalization uses Config, active distribution, PendingSol,
PrincipalSolQueue, OperationalSol, DistributionEscrow, KifSol, PrincipalJito,
PendingJito, PivAuthority, configured Mint, System and legacy Token in that order.
Config and actual transfer endpoints must be writable; no caller signature is
required and zero transfers are omitted. Runtime Rent, fixed custody and all
program/Mint/authority checks run even for a no-op. The shared authority is its
canonical empty System PDA, which may be unallocated and has no invented funding
requirement. System transfers use each source vault's own PDA seeds; Token
`TransferChecked` uses the shared authority's seeds. Order is principal SOL,
escrow SOL, KIF SOL, then principal JitoSOL: at most four CPIs, with complete
metadata/borrowed-data fingerprint checks after every successful leg.

The unchanged pure normalization transition stages the complete Config update
before effects; final persistence and a factual event occur only after all
postconditions. Pending-only recognition/no-op remains available during pause
or RecoveryRequired; any movement retains the accepted pause/recovery gates.
Historical assets, active SOL/token offsets, earned KIF liabilities/carry and
rent cannot be swept as surplus. Repeated normalization creates no additional
credit. Original CPI errors propagate, including failure after earlier legs.

The new normalization-only observation accessor preserves both full Token-native
balances and the entire operational reserve unchanged/unclassified. It adds no
extraction, close/recreate, funding-baseline inference or valuation. The old
strict economic accessor remains unchanged. Initial bootstrap has its separate
bounded quarantine contract below; Token-native extraction and compatibility
with later lifecycle handlers remain separate work.
Focused tests model signed System transfers and execute the pinned host Token
processor, independently checking PDA seeds and all account bytes. Staged host
effects are explicitly discarded after failure; this is not new VM/Bank rollback
evidence or complete Testnet readiness.

Initial bootstrap uses the same first 13 accounts, then the configured Jito
program, pool, validator list, reserve and manager fee account. A distinct
referrer is the nineteenth account; Config equality selects the shared receiver
profile without a duplicate account. Current runtime Clock/Rent and the full
source-pinned protocol identity checks authenticate the observation. Valuation
requires the current pool epoch and `held units <= Mint supply <= recorded pool
supply`, then floors `units * total pool lamports / recorded pool supply`.
Permissionless burns can reduce Mint supply before maintenance updates the pool;
the stored denominator remains authoritative during that lag. Paired empty
pool accounting supports SOL-only bootstrap. No capacity, revision, fee quote
or executable pool-operation facts are fabricated from these fields.

The accepted initial-only transition stages the complete Config/HWM update
before effects. At most two nonzero signed transfers run: PendingSol to
PrincipalSolQueue, then PendingJito to PrincipalJito via `TransferChecked`.
Config and actual transfer endpoints must be writable; the caller supplies no
signature or value. All future buffer borrows are checked before the first CPI,
and complete metadata/borrowed-data fingerprints are verified after each CPI.
Only then is Config committed and its factual event emitted. Rent, operational
funding, both Token-native balances, the entire distribution and unrelated
protocol accounts remain exact. The bootstrap-only observation accessor excludes
Token-native funding without classifying, extracting or valuing it. The public
legacy `PoolSnapshot` bootstrap API retains all its checks and their order.

Positive tokens with zero floored value still establish historical token units
and prevent replay. Prior distributions, insufficient attempts or any economic
history cannot enter this initial-only path; post-settlement integration remains
open. Host tests compare complete account bytes,
literal signed transfer instructions and independent integer valuation, using
the pinned host Token processor. Exact failed prefixes remain visible before
explicit staged discard/retry; new-path VM rollback and resource behavior await
the local lifecycle milestone. Reference protocol identities do not establish
the supported Testnet package or deployment readiness.

Protected principal staking appends the derived Jito withdraw authority to the
bootstrap topology, with the same Config-selected shared/distinct fee receiver
profile. The principal SOL queue, principal JitoSOL vault, pool, reserve, Mint
and both fee receiver roles are writable, along with Config. The fixed inner
SPL instruction duplicates a shared receiver in its two required positions;
outer accounts remain unique. Its exact pinned `DepositSolWithSlippage` variant
25 carries only amount and the greater of the Config-derived 0–1 bps floor and
the caller minimum. The principal SOL PDA signs one CPI; no caller signature,
unprotected fallback or optional deposit-authority signer mode is supported.
The pool must currently have no SOL deposit authority.

Both pool/Mint observations are authenticated. Recorded supply remains the
denominator, and principal plus pending token holdings must fit actual Mint
supply, which may lag recorded supply after burns. Both supplies independently
increase by the exact newly minted amount, preserving the gap. Valid external
zero-fee encodings (`0/0` and `0/N`) are supported without changing the legacy
model's canonical fee representation or validation order. Nonzero fees, a
one-lamport historical book-value loss, HWM shortfall, pause, non-Idle state or
unnormalized custody reject. Separate next-cycle yield cannot subsidize a loss;
only recognized historical SOL is eligible, and only the two historical unit
fields change. There is no automatic amount optimizer or new fee allocation.

The full transition is staged before effects. Every account's metadata and
data fingerprint must then match the exact result, including the complete pool
allocation tail, unchanged fee recipients and both quarantined Token-native
balances. Only reserve/source native balances, principal token amount, Mint
supply and pool totals may change during CPI; Config and its factual event
commit last. Host tests run the pinned Token `MintTo` processor inside a modeled
stake-pool callback and explicitly retain/discard failed effect prefixes. They
do not execute the stake-pool processor or prove SBF/Bank rollback/resources.
This is the bounded deposit component required by M2; the full M3 adapter,
withdrawal lifecycle, M4 local production lifecycle and Testnet package remain.

Fixed Anchor/Borsh-compatible state payloads now have authenticated owner/PDA/
size/discriminator/version/zero-tail envelopes and atomic existing-account byte
persistence. The library also contains guardian/Clock and Squads checks, genesis
model preparation and source-pinned Jito identity authentication. Initializer
composition now uses those layers; only the bounded protected stake-pool deposit
CPI is exposed. The remaining adapter and complete economic lifecycle stay open.
The stake-pool/custody mocks remain test-only and do not establish exact SPL/Jito
behavior. Jito identity evidence leaves freshness, fees and execution readiness
separate; current-state Squads authority alone is not action approval.

Root and writer each executed **15 Node tests PASS**, plus **eight old-profile
and sixteen recipient-profile CLI cases**, for Task 2.27. The default CLI output
remains byte-identical to Task 2.23. Separate source/test review passed; a test
oracle was corrected before execution. All 96 inputs and twelve new log hashes
match. **468 Rust tests +1 doctest/eight gates** are retained Task 2.26 evidence,
not rerun, after verification of all 92 unchanged Rust inputs and retained logs.
The historical
[Task 2.14 artifact](../../docs/TASK_2_14_RUNTIME_PENDING_RECONCILIATION.md) passed
**24 local SBF tests /70 cases**, including actual local System CPI. Those tests
used synthetic initial state and do not prove initialization, signatures,
Bank rollback, public deployment or later-source runtime behavior.

No dedicated live Program ID, initialized PIV1, executed production Jito CPI or full
Testnet readiness is established. D-027 authorizes the accepted milestone's
main integration; D-028 authorizes the bounded Tasks 2.20–2.23 integration.
Deployment, key/signing, funds and authority transfers retain
the separate D-026 approval gates. Mainnet key material must never be created or
stored on this VPS.

[Task 2.28](../../docs/TASK_2_28_CURRENT_SBF_RUNTIME_REFRESH.md) records the separate
current-source SBF build and local runtime refresh for these same dispatched
claim/pending paths, including exact artifact identity and execution outcomes.
Use that report and the checkpoint for current evidence; retain Task 2.14 as
historical. Compilation does not prove execution of undispatched genesis,
Squads, Jito or other library code. Task 2.28 exposed no native initializer;
Task 2.39's separate production-boundary evidence must not be inferred from it.
