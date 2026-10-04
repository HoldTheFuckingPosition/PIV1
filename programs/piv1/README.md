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
Thirteen instruction profiles are dispatched:

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

- `prepare_distribution`: exact nine-byte `PIV1PD01` version-1 data; fixed 27/26
  accounts. Current authenticated pool ratio and guardian/Clock snapshot drive
  no-yield evaluation or fully liquid-funded preparation. Pending SOL pays first,
  then recorded next-cycle yield from principal SOL. Any positive remaining native
  shortfall rejects with custom error 6145 before effects, without recording an
  insufficiency cooldown. Withdrawal preparation remains unavailable through this original liquid ABI;
  use the separate bounded active-source profile below.

- Active-source withdrawal preparation: exact 13-byte `PIV1PW01` version 1 plus a
  little-endian u32 validator-record index; fixed 29/28 accounts. The old profile
  remains unchanged. A read-only Stake query authenticates the dynamic minimum;
  one current active-source witness, operational rent, canonical target and
  multileg fee/floor/residual-HWM proofs gate opening a withdrawal round.

Ordinary host entrypoint calls reject execution. Explicit host seams model
context, invocation and rollback. The existing claim callback seam cannot execute
initialization, guardian operations, deposits, economic normalization or initial
bootstrap, staking or preparation. Separate host seams model their context
and effects. Heartbeat/pause use the existing atomic envelope commit and emit
factual events only after success; logs still require successful transactions.
- Active-source leg initiation: exact 13-byte `PIV1IL01` version 1 plus a
  little-endian u32 validator-list index; strict 25/24 account profiles. Derives
  the maximum-safe input, normalizes unused temp-PDA prefunds, funds full current
  rents from OperationalSOL, executes protected SPL withdrawal and immediately
  deactivates the stake before the final atomic Config/round/leg commit.

Remaining governance update markers are unimplemented. Already-received direct transfers remain reconcilable during
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

## Liquid distribution preparation

The preparation profile uses bootstrap's first 19/18 roles (the manager/referrer
role is shared only when Config records equal keys), followed by current registry,
six reward accounts in registry order, and canonical Clock. The supplied Clock
must match runtime Clock in every field. Config and round require write privilege
for positive preparation, alongside the escrow and every nonzero funding source.
No caller-selected amount, fee, valuation, minimum or target is accepted.

Historical value uses the authenticated stored pool ratio and current epoch;
combined held tokens must fit current Mint supply, which may lag stored supply
following direct Token burns. Pending contributions and previous next-cycle yield
remain separate. The liquid branch proves residual historical value plus unused
prior yield covers the proposed HWM before up to two signed System transfers.
The accepted transition snapshots the exact raw stake withdrawal fee, current KIF
period/activity, recipients and liabilities; Config HWM does not increase until
settlement. Pending's full contribution ledger remains with its active-round
physical offset. Both state envelopes commit only after exact full-account CPI
postconditions. Token-native balances and operational funding stay quarantined.

No-yield evaluation changes no bytes or clocks. Liquid success remains possible
inside the previous valid-insufficiency retry interval. Unsupported withdrawal
is a failed transaction, never a valid-insufficient result and never a fabricated
technical minimum. The separate active-source profile below supplies the bounded dynamic
minimum, source residual and aggregate withdrawal proofs. Unsupported source
branches and actual withdrawal execution remain separate from this preparation ABI;
`PIV1IL01` below now provides bounded active-source initiation. Host System/signature effects and explicit staged-world discard are
models, not new-path VM/Bank execution or rollback evidence.

## Active-source withdrawal preparation

`PIV1PW01` extends the liquid profile's 27/26 roles with canonical executable
Stake Program and one candidate active stake account. Only a positive native
shortfall uses this profile; use `PIV1PD01` for no-yield/liquid evaluation. The ABI
provides a checked list index, never an amount, target, capacity or minimum.
The read-only `GetMinimumDelegation` CPI is followed by exact full-account checks
and authenticated origin/eight-byte return data. Failed queries propagate without
custody or state writes. The production edge to already-locked Stake interface
1.2.1 (`borsh`) adds no package/version; its types authenticate the stake bytes.

The bounded profile checks one 73-byte current Active record and the exact derived
200-byte Stake account, voter, pool authorities/nonbinding lockup, nondeactivation and empty
flags. Positive delegation with activation no later than Clock, or bootstrap
activation, proves the active-or-activating split branch; it does not claim full
activation. Source nondelegated backing must cover current rent; maximum output
preserves both current delegated minimum and pinned SPL residual. An active record
above residual plus the SPL token-value tolerance witnesses active source order.
A configured preferred withdraw vote must match this candidate. Transient, reserve,
removal and exhausted-preference fallback are unsupported; no full list is scanned.

The future split destination must be an uninitialized 200-byte Stake account
prefunded with current rent. Preparation verifies enough operational funding for
that rent plus `WithdrawalLeg::SPACE` metadata, but creates/funds neither account.
The `PIV1IL01` initiation profile below repeats the current checks, creates the
exact destination, executes protected SPL withdrawal and immediately deactivates
atomically.

Canonical Phase0 inverse math fixes the largest bounded token target whose gross
book cost fits the shortfall. Exact fee/redemption monotone searches derive the
minimum and candidate maximum; maximum-safe first fill may not strand a nonzero
subminimum remainder. The round reserves for per-leg fee ceilings and native floors,
then stores its immutable configured 0–1 bps output floor. Conversion dust increases
the proposed HWM. Exact residual book value must cover that HWM; even a one-lamport
partition-floor failure rejects, without reducing protection or changing economics.

A valid target below the measured minimum, including an exact zero inverse,
commits only the 24-hour retry timestamp and factual insufficient event. The legacy
public pure helper still rejects zero; the private authenticated path carries the
additional real-account proof. Malformed candidate/query/rent failures never set
cooldown; a sufficient attempt bypasses a previous insufficiency interval. Positive
preparation retains pending-first funding, optional prior-yield funding, exact
post-CPI fingerprints and final atomic Config/round writes. No stake withdrawal,
source reservation or future liquidity guarantee is implied by preparation.

Host tests model the query, System transfers and rollback/discard. Pinned SPL 2.0.3
source and separately verified Stake 5.1.0 split/activation source establish the
bounded proof; this does not attest a live cluster's deployed program revision.
Full nested execution and production lifecycle remain unproved.


## Protected active-source leg initiation

`PIV1IL01` reuses the initial-bootstrap 19/18 roles, then appends canonical Clock,
canonical executable Stake Program, current active validator source, canonical
pool withdraw authority, `WithdrawalLeg` PDA and `WithdrawalStake` PDA. Metadata
and Stake addresses derive from `withdrawal-leg` / `withdrawal-stake`, active
sequence u64 LE, next leg index u64 LE and their canonical bumps. Caller data
contains only a validator-record index: no amount, price, minimum or PDA bump.

The handler reuses the authenticated active-source proof, queries the actual
Stake minimum with exact producer/length/value checks, and computes the greatest
safe input bounded by the fixed remaining target. It enforces both current and
snapshot input floors, no stranded remainder, current per-leg protected output,
the remaining useful-leg bound, the immutable round output floor and residual
HWM after assignment of the entire fixed target. Partial-round output feasibility
uses current post-withdraw pool accounting and fees with a conservative rounding
reserve based on the immutable snapshot input floor and remaining useful slots.
This is current feasibility, not a promise about future liquidity or fees.

Only unused, System-owned, empty, nonexecutable canonical temporary PDAs qualify.
All observed prefunding goes to fixed PendingSol before creation and is recorded
once as a pending contribution, retaining the active round's existing physical
pending offset. OperationalSol then advances the full current Rent for each new
263-byte metadata and 200-byte Stake account. Source metadata reserve, the pinned
Stake 5.1.0 destination pseudo reserve of 2,282,880 lamports, and those actual rent
advances are distinct. No prefund becomes operational recovery or cooldown yield.

Production invokes only pinned SPL 2.0.3 `WithdrawStakeWithSlippage` (variant 24)
with PivAuthority signing, then immediately invokes Stake `Deactivate` (variant 5)
with that authority. Full fingerprints after each CPI bind exact pool/list/Mint,
fee receiver, source delegation/lamports, destination authorities/delegation/rent,
all unrelated bytes, Token-native quarantine and every custody balance. Validator
list checks hash a bounded selected-record patch without cloning the whole list.
Fresh post-CPI identity, staged custody and economic proofs precede the single
atomic Config/round/new-leg write and factual event. No intermediate leg index or
state header is committed. CPI errors propagate; actual atomic rollback belongs
to Solana's transaction boundary, not the explicit host effects/discard model.

This profile retains exact preferred-active-source support; transient, reserve,
removal and preferred fallback remain unsupported. Finalization, settlement,
post-settlement integration, actual nested pool/Stake VM execution, transaction
resource measurements and the complete founder Testnet lifecycle remain open.
No dependency, account payload layout, toolchain or accepted economics changes.

### Task 2.48: protected withdrawal-leg finalization

`PIV1FL01`, version 1, is a strict 17-byte native ABI carrying only a u64 leg
index. Its 24/23 roles are the existing bootstrap 19/18 roles followed by Clock,
Stake program, canonical Stake History, the initialized leg PDA and its Stake PDA.
The actual active-round sequence binds both PDAs; the caller supplies no amount,
readiness flag, rent, authority or destination. The fixed custody, current pool
valuation, original metadata, both Stake authorities, voter, deactivation epoch,
Clock and lockup are authenticated before effects. Global pause still blocks it.

Canonical pinned Stake `Withdraw` (variant 4) withdraws the complete balance to
fixed distribution escrow. Its successful execution is the authoritative current
inactivity proof using Stake 5.1's actual runtime Clock/Stake History; PIV1 does
not duplicate the older interface's activation algorithm or impose an epoch-wait
heuristic. The exact postcondition is zero Stake lamports and zero data length,
with unchanged owner inside the call. A second canonical System transfer returns
only the originally recorded Stake rent advance from escrow to operations.
Metadata closure returns only its original recorded rent to operations; metadata
excess moves to pending contributions with the active-round pending offset intact.
Closure borrows remain held across the atomic state commit, followed only by
infallible closure writes.
Zeroed, zero-lamport metadata cannot regain initiated status through a refund.

The supported native observation is `original stake rent <= balance` and
`balance - original stake rent <= current authenticated delegation`. Greater
balances reject before effects: they can reflect a native donation or historical
runtime rent adjustment, and the current record cannot distinguish them safely.
No unsupported excess becomes yield or an invented contribution. Balances below
the original rent also reject rather than inventing full rent recovery. Within
the supported profile, observed value net of original rent yields the accepted
cooldown reward/loss. Rewards stay in escrow outside the fixed active allocation;
loss or current whole-target residual-HWM failure records `RecoveryRequired`
without lowering HWM. Partial and out-of-order finalization do not fund settlement
until the exact target and all successful legs reconcile. Existing APIs, account
layouts, dependencies and economics are unchanged. Actual nested Stake execution,
heap/CU and rollback require subsequent runtime validation; host effect callbacks
and modeled failed-world discard do not establish those facts.

### Task 2.49: atomic distribution settlement

`PIV1SD01`, version 1, is exactly nine bytes with no caller economic inputs. Its
28/27 roles are the existing bootstrap 19/18 roles followed by Clock, the frozen
HTFP and Team recipients, and the six reward PDAs in frozen snapshot slot order.
Current pool valuation and normalized custody bind the complete `EscrowFunded`
round. Prior target/count/closure proofs remain inductive in the authenticated
round; closed leg metadata is not enumerated. Snapshot recipient keys and guardian
key/revision/index tuples stay authoritative despite later configuration or
activity changes. Current earned rewards authenticate independently of a current
registry; their selected claim sum may be less than the global historical
liability. No new recipient-control or guardian-earning provenance claim is made.

Existing capped `5900 / 1950 / 200` weights over `8050` derive the net transfers;
KIF uses the frozen eligibility bitmap, all prior carry and the accepted repeated
zero-active 50/50 rule. Protected value includes projected zero-active compound
once, remaining physical principal tokens at the current official ratio, and
post-payment escrow; it excludes new cooldown rewards and already-used pending
SOL. Only the final pending-use subtraction floors at zero for the accepted
T23-R1 recovery classification. The existing state transition runs once with that
actual projected value. A valid HWM failure changes only the recovery header and
emits a distinct no-payment event; it performs no CPI or Config/reward credit.

Success uses exact System transfers to the two empty System-owned frozen
recipients and KIF, then KIF-to-principal compounding if required. Zero amounts
skip CPI; positive recipient credits must leave the recipient rent-backed, while
an initially empty balance is otherwise supported. Every CPI verifies all supplied
account bytes, native balances and metadata. Fresh final custody/ratio checks
precede an atomic eight-account Config/round/reward byte commit and factual event.
Pending integration remains separate. Native Token lamports, original operational
rent, historical/pending ledgers and earned KIF liabilities stay isolated. Host
callbacks and failed-world discard do not prove actual runtime payment rollback,
heap/CU or a complete production lifecycle; those remain subsequent runtime gates.


## Production post-settlement pending integration

Task 2.50 adds strict `PIV1IP01`, version 1, exactly nine bytes without amounts.
Accounts 0–17/18 use the bootstrap fixed custody/protocol order, followed by exact
Clock: 20 accounts with distinct manager/referrer, 19 with their configured shared
receiver. Config and round must be writable; a vault needs writability only for
an actual transfer. The permissionless instruction requires unpaused, normalized
Settled custody and a current official pool epoch, with aggregate held units no
larger than Mint supply and Mint supply no larger than recorded pool supply.
Stored supply remains the book-value denominator during legitimate direct-burn
lag. Zero total and supply are supported together only with zero held units.

The bounded transfers move physical pending SOL `P-U`, the actual escrow remainder
and all pending JitoSOL `Q` to principal. SOL uses the canonical source PDA and
System transfer; legacy Token `transfer_checked` uses the canonical PIV authority.
Full recognized contribution value `P + floor(Q * current total / stored supply)`
raises HWM once. Final historical SOL excludes new cooldown carry; actual final
principal tokens become historical tokens, and current protected value must cover
new HWM. The completed Idle header removes old round offsets and preserves its
summary. Existing KIF claims/carry, zero-active compounding, operational rent and
both full Token-native balances remain untouched or already accounted once.

Whole-account receipts follow every CPI. Fresh fixed custody, pool/Clock and
staged Idle obligations are checked before atomic Config/round persistence and
`PendingContributionsIntegrated`. Insufficient protected value rejects before
transfers; it does not invent another recovery transition. Narrow integration
observation accessors preserve Token-native quarantine; the old strict accessor
and public model derivation validation/error order remain unchanged.

The ten focused host groups use independent integer/full-account expectations,
manual CPI bytes/metas/seeds, both receiver topologies, liquid/withdrawn rounds,
KIF eligibility and cooldown, post-snapshot contributions, no-transfer completion,
current-ratio and direct-burn cases, pool loss, aliases, replay, malformed context,
CPI corruption/failure and both late state borrows. Host failed-world discard is
explicit; actual production System/Token rollback, heap/CU, complete local lifecycle
and live Testnet readiness remain separate validation obligations. No persisted
layout, dependency or economic change is introduced.
