# PIV1 offline initializer planner

This command checks a public manifest and prepares unsigned initializer,
Squads and address lookup table (ALT) instruction drafts. It uses the existing
pinned Node 24.19.0 and web3.js 1.98.4 installation. Run it from the repository:

```sh
NODE=/home/jerem/.local/piv1-toolchains/node-v24.19.0-linux-x64/bin/node
"$NODE" clients/cli/initializer-plan.cjs --template
"$NODE" clients/cli/initializer-plan.cjs --manifest clients/cli/initializer-plan.example.json
```

The supplied example deliberately leaves identities and choices `null`. Its
successful result is `INCOMPLETE`, with missing field paths and `plan: null`.
Copy it to a public JSON file and fill only choices you have explicitly selected.
A complete declaration produces `OFFLINE_PLAN_ONLY`. Neither status grants
public testing, deployment or transaction authorization.

Only `--template` and `--manifest <file>` are supported. The planner reads the
explicit public manifest and pinned repository/SDK inputs, then writes JSON to
stdout. It does not read wallet configuration, authentication files or secrets,
connect to RPC, create keys, sign, submit, create accounts or deploy anything.
Do not provide wallet or credential files as the public manifest.

The manifest has a fixed schema; unknown or duplicate JSON fields, invalid keys,
incorrect types and unsupported aliases reject with a nonzero exit. Missing
keys or explicit nulls remain incomplete. All public-key strings must be canonical
base58. Every public key used as an ordinary transaction signer must be on the
Ed25519 curve; this check does not establish who controls it. All `u64` and `i64`
fields are decimal strings, including transaction indices, slots and the signed
anchor timestamp. Vault/buffer indices and the guardian permutation use bounded
JSON integers. Pause, anchor, recipients, approvers and slot assignments have no
silent defaults.

Declare the existing multisig address and its public create key, the current and
stale transaction indices, the proposed next index, governance vault and buffer
indices, creator, executor, six guardians and four distinct approvers. Guardian
declarations are sorted by their raw 32-byte keys before applying the explicit
six-element permutation. The output shows canonical members and final PIV slots.
The creator, executor and approvers must belong to that declared set; current
membership, permissions, autonomous 4/6 threshold and timelock remain unverified.

Declare the fee payer and external initializer rent payer separately. They may
be the same wallet. The rent payer must differ from every guardian and protected
PIV/protocol/recipient role. The fee payer may be a guardian or another ordinary
signer where the compiled privilege union permits it. The output counts the
actual required signatures in every packet, including all 64-byte zero
placeholders. ALT authority may be another explicitly declared ordinary signer.

Production pool/program/mint, Token, Stake, System, Squads and loader identities
are fixed integration constants. Declare the validator list, reserve, manager
fee receiver and referrer explicitly. Sharing the manager/referrer gives 34
native roles; distinct receivers give 35. The two recipient vault indices derive
vault PDAs of the declared multisig and must differ from the governance vault
and each other. No ownership, funding, control or live account linkage is inferred.

Complete output includes the exact 315-byte native payload, canonical PDAs,
named account roles and privileges, lookup-free stored message commitment,
buffer upload/from-buffer, Active proposal creation, four approval drafts and
the final v0 execute draft. Upload chunk lengths are selected using the actual
compiled signer union and then checked by SDK serialization and decompilation.
Every packet must fit the legacy/v0 1,232-byte limit. The final execute requests
1,400,000 CU, 262,144 heap bytes and 8,388,608 loaded-account bytes. These are
requests based on the reviewed local initializer profile, not measured public
consumption or a minimum budget for your declarations.

ALT create and extend drafts contain the 16 derived PIV targets in exact order.
The ALT used to compile v0 is explicitly hypothetical. Before future execution,
an independent process must verify successful creation and extension, the actual
owner/authority/active state/content, recent-slot eligibility and a current slot
strictly later than the final extension slot. The tool reports actual extension
slot and warm-up verification as open; it performs none of those actions.

The declared blockhash, last valid block height and reference slot are encoding
inputs only. Uploads, approvals and timelock can outlive that blockhash. Regenerate
each future signed step using independently refreshed context and recheck all
account/authority/resource conditions. A precomputed bundle is never send-ready.

Account byte sizes are reported for planning. Actual fees, rent, funding quotes,
wallet control, current snapshots and exact package/artifact selection remain
open. Other public operator handlers and the founder workflow remain open. M4's
unwaived evidence reservation is unchanged.

Focused verification uses the built-in Node test runner without additional
dependencies:

```sh
"$NODE" --test clients/cli/initializer-plan.test.cjs
```

The tests use independent literal byte/role expectations, deterministic public
points and SDK roundtrips. Historical transport tools and tests remain intact.


The separate funding command computes a conditional rent ledger from an explicit
public manifest and supplied rent observations:

```sh
"$NODE" clients/cli/initializer-funding.cjs --template
"$NODE" clients/cli/initializer-funding.cjs --manifest clients/cli/initializer-plan.example.json --observations clients/cli/initializer-funding.example.json
"$NODE" --test clients/cli/initializer-funding.test.cjs
```

The observation example preserves historical finalized Testnet rent responses
captured on 2026-10-18, bracketed by slots 454098679–454098682. It is not a current
price or account snapshot. `--template` prints a separate template with null
amounts and variable metadata. Copy it to an explicit public JSON file if using
other independently obtained observations. The format fixes the official Testnet
endpoint, genesis and commitment; all 14 account sizes must have positive,
nondecreasing decimal-string lamport values. Unknown or duplicate keys, malformed
known values and unsupported flags reject with a nonzero exit. Missing or null
observation fields return exit zero, `INCOMPLETE` and `scenarios: null`.

Complete observations return `CONDITIONAL_RENT_ONLY`, even when identities remain
unselected. An incomplete manifest gives both supported manager/referrer variants
using symbolic rent-payer, creator and fee-payer roles. It supplies no account
addresses or signature counts. A complete manifest uses the unchanged initializer
planner's actual phases, addresses and one/two/three signature unions. The file
digests bind supplied bytes; the command does not authenticate observations,
contact RPC, refresh prices, read wallet/secret configuration or sign/send anything.

Every scenario explicitly assumes zero-prefunded valid PIV targets and newly
created ALT/Squads resources without donations. Actual target shortfalls depend
on original balances plus owner/data/freshness checks, which remain open. Excess
SOL on token targets moves internally to PendingSol; it is not refunded to the
payer and cannot replace PendingSol's original shortfall. Actual prefunded or
allocated ALT/Squads accounts are outside this finite scenario.

The rent payer funds the ALT at 56 bytes and only its growth to 568 bytes; the
stored transaction similarly grows from 102 bytes to its final size. The buffer
closes after the stored transaction is fully funded. Its entire actual balance
goes to the creator, who differs from the rent payer under the supported guards.
The report assumes only the original buffer rent is refunded. It does not assume
a transfer back to the rent payer. Stored transaction, proposal, ALT and PIV
balances remain held through this plan; no later reclamation is planned.

Gross rent-payer outflow, creator refund and retained account balances are separate
quantities. The rent payer must additionally retain the zero-data rent floor after
initializer allocation; this residual is not spent rent. The two recipient vaults
must already meet their funded System/empty identity prerequisites. Their actual
balances, deficits and provisioning are open and excluded from packet outflows.

Actual fees, priority fees, wallet balances, funding transfers, peak liquidity,
operational liquidity, governance provisioning and production deployment/artifact
costs remain null/open. Signature counts are not fee quotes. Conditional rent
arithmetic is not an actual funding shortfall, complete wallet requirement,
deployment budget or readiness decision. All public/deployment readiness flags
remain false; M4's unwaived reservation and the original planner's live account,
authority, blockhash refresh and ALT warm-up requirements remain unchanged.


The intake command prepares exactly one explicit action draft:

```sh
"$NODE" clients/cli/intake-plan.cjs --template
"$NODE" clients/cli/intake-plan.cjs --manifest clients/cli/intake-plan.example.json
"$NODE" --test clients/cli/intake-plan.test.cjs
```

The example leaves action, identities, amounts, context and resources null.
Missing required fields return exit zero, `INCOMPLETE` and `plan: null`; valid
complete declarations return `OFFLINE_PLAN_ONLY`. Use `action: "contribute-sol"`,
`"contribute-jitosol"` or `"bootstrap-initial"`, then fill the matching `sol`,
`jitosol` or `bootstrap` block. Keep the other two blocks null or entirely null.
Non-null unused inputs, unknown/duplicate fields, malformed known values and
unsupported flags reject with a nonzero exit. No action or identity is inferred.

For SOL, declare the external donor and positive decimal-string `amountLamports`.
For JitoSOL, declare its owner, explicit existing source token account and positive
`amountUnits` in base units. Exact u64 amounts include values above JavaScript's
safe Number range; JSON numbers, zero and overflow reject. The source can be an
ATA or another legacy Token account, including an off-curve account address. No
account creation is planned. The token owner must sign, rather than a delegated-only
authority. Its chain account owner need not be System, but production excludes
PIV/internal identities and the exact Token-owned 355-byte SPL multisig form.
If delegate equals owner, Token may still require delegated allowance; verify it.

Both intake actions require unpaused initialized Config/round/custody bindings and
coverage of both old pending obligations. Intake is not Idle-only. SOL requires an
external System-owned empty donor account; JitoSOL requires the rent-backed,
initialized, non-native 165-byte legacy source with the configured canonical mint
and token owner, plus valid mint/units/allowance. These remain live preconditions,
not checks performed by this client. A contribution records only the requested
pending increment and creates no ownership, withdrawal or reward right.

Initial bootstrap takes no amount, valuation or governance input. Only the outer
fee payer signs; neither PIV authority nor pending SOL is an external signer.
The 18/19-account shape depends on whether declared manager fee and referrer keys
are shared, and must match actual Config. Production derives all recognized
pending SOL/token amounts, moves them to principal custody and obtains initial
book value from current official pool state. It performs no pool/Stake CPI or
staking. Initial unpaused Idle/zero-history state, nonzero recognized pending,
normalized custody and current pool epoch/supply checks must hold. Positive token
units with floored value zero are allowed. Actual bootstrap amounts, eligibility
and value remain unknown; this is not a later integration command.

The common fee payer, executing program and transport context are explicit.
Ordinary signer keys must be on-curve, which does not prove control or ownership.
Fee payer may equal donor or token owner; packet signatures count the actual union.
Requested native roles and recovered outer privileges are both shown, including
token owner's writable escalation when it also pays fees. Protected aliases reject.

All three resource requests are required: `computeUnits` 1–1,400,000; `heapBytes`
32,768–262,144 in 1,024-byte increments; `loadedAccountBytes` 1–8,388,608. These
are client-supported request ranges based on the reviewed local profile, not all
cluster limits, measurements or a promise that selected values suffice. The draft
contains those three budget instructions and one production action, serialized as
a legacy packet with actual zero signature slots and an SDK roundtrip <=1,232 bytes.
No ALT is required. Refresh blockhash/context and recheck current accounts before
any future signed step; serialized bytes are never send-ready.

The finite error guide describes source-bound PIV codes and unresolved standard
or downstream errors. It is not a state inspector or universal live error decoder.
Account validity, state, pause, balances, signer control, current epoch, resources,
fees, rent, funding, exact artifact selection and public/deployment readiness remain
unverified. No RPC, wallet/config/secret reads, key generation, signing, sending or
live creation occurs. M4's stopped operation and unwaived reservation are unchanged.


The state inspector reads only supplied Config and ActiveDistribution records:

```sh
"$NODE" clients/cli/state-inspect.cjs --template
"$NODE" clients/cli/state-inspect.cjs --snapshot clients/cli/state-inspect.example.json
"$NODE" clients/cli/state-inspect.cjs --snapshot clients/cli/state-inspect.fixture.json
"$NODE" --test clients/cli/state-inspect.test.cjs
```

The null example returns exit zero, `INCOMPLETE` and `inspection: null`. A complete
supported pair returns `OFFLINE_STATE_INSPECTION`. Supply the explicit program,
context label/slot and each account's address, declared owner, executable flag,
lamports and canonical base64 bytes. All u64 input values are decimal strings.
Unknown or duplicate fields, malformed known inputs and unsupported flags reject
with a nonzero exit, including when another account is missing. Only `--template`
and `--snapshot <file>` are supported; there is no RPC or implicit snapshot path.

The populated fixture is a historical synthetic local Bank pair, extracted
unchanged from a reviewed retained lifecycle after pending integration at slot
524257. It is not public chain state or an approved public identity. Context
labels, cluster, slot and file digests are supplied provenance declarations, not
authentication, proof of coherence or freshness.

The inspector checks declared owner/nonexecutable metadata, exact 1,014/891-byte
allocations, discriminators, initialized V1 headers, Borsh bool/Option/phase tags,
zero migration reserve and zero allocation padding. It checks canonical current
PDA addresses/bumps and Config's reference to the supplied distribution. System,
Token and Stake constants are checked; other external bindings are decoded facts.
It does not inspect the actual accounts referenced by any stored key.

Both complete named objects and a readable stored-fact summary are returned:
pause, HWM, historical and pending ledgers, KIF, timing, cumulative totals,
coarse phase, preparation/leg counters, escrow and last completion. Signed times
and all 64-bit amounts remain exact decimal strings, without Date/Number coercion.
Config's two independent optional timestamps and the optional 88-byte completion
summary shift later Borsh fields; decoded payload length and trailing zero padding
are reported separately from the fixed allocation.

Current Config recipients and registry bindings remain separate from the frozen
round's recipient/guardian snapshot and revision. They may legitimately differ
after updates. Idle active fields and the retained completion summary are also
separate. Phase/counter/sequence normalization, economic/HWM/KIF reconciliation,
custody, rent health, valuation, timing eligibility and full Rust validation remain
unverified. A decoded pending ledger is not physical available custody or solvency.
`RecoveryRequired` means stop/report; this command invents no recovery operation.

The inspector produces no transaction packets and establishes no action eligibility,
resource sufficiency, fees, funding or public/deployment readiness. It reads only
the explicit public JSON and pinned source/SDK files, writes JSON to stdout, and
performs no network, wallet/config/secret read, key creation, signing or sending.
Existing clients and M4's unwaived stopped-operation reservation remain unchanged.


The principal-staking planner drafts one protected conversion of already recognized
historical principal SOL, keeping pending contributions untouched:

```sh
"$NODE" clients/cli/principal-stake-plan.cjs --template
"$NODE" clients/cli/principal-stake-plan.cjs --manifest clients/cli/principal-stake-plan.example.json
"$NODE" --test clients/cli/principal-stake-plan.test.cjs
```

Declare `programId`, an external `feePayer`, `amountLamports`,
`callerMinimumPoolTokensOut`, the four `protocol` addresses, reference `transport`
context and all three `resources` requests. Missing/null required values return
exit zero, `INCOMPLETE` and `plan: null`. Complete declarations return only
`OFFLINE_PLAN_ONLY`. Unknown/duplicate fields, malformed known fields, unsupported
aliases or other flags reject with a nonzero structured error. The schema selects
this single action; there is no amount optimizer, account creation or implicit input.

Amounts are canonical decimal-string u64 values: native amount must be positive;
the explicitly supplied caller minimum may be zero. No minimum is chosen for you,
and the caller minimum need not be less than the native amount because those are
different assets and this tool has no quote. Production uses the maximum of its
Config-derived floor and caller minimum, separately requires positive minted output
and enforces exact/no-loss economics. Zero caller minimum adds no stronger caller
floor; it does not disable those guards. The numeric effective minimum and minted
output remain unknown, including when the derived integer floor could be zero.

The native instruction is exactly `PIV1SP01`, version 1 and two u64LE values
(25 bytes), with 19 shared-receiver or 20 distinct-receiver accounts. Canonical
PIV PDAs and the final official pool withdraw PDA are derived. Only Config,
principal SOL, principal token, mint, pool, reserve and fee/referrer accounts are
writable; pending custody and the validator list are readonly. Every native role
is an external nonsigner. PIV signs the principal SOL PDA internally for protected
pool CPI variant 25. No outer Squads, guardian, Stake, Clock or Rent account is added.

The fee payer is the sole outer signer and must be a distinct on-curve ordinary
signer; that check proves neither control nor a valid funded fee account. Program
and protocol addresses do not have a blanket on-curve requirement. All unsupported
known aliases reject, including the canonical withdraw PDA as a declared PIV
program, fee payer or protocol role. Shared manager/referrer is represented once
and must match actual Config. The compiled legacy packet contains three explicit
budget requests and one PIV action, counting its full zero signature slot and
checking SDK roundtrip and the 1,232-byte cap. It needs no ALT in this profile.

Resource requests use the existing client ranges: 1–1,400,000 CU, 32,768–262,144
heap bytes aligned to 1,024, and 1–8,388,608 loaded-account bytes. They are requests,
not measured public sufficiency or complete cluster limits. Blockhash/last valid
height/reference slot are unverified encoding inputs; refresh and recheck state
before any future signed step. A serialized packet is never send-ready.

Production still requires unpaused Idle state, valid normalized whole custody,
backed recognized historical SOL, current authenticated official pool accounts,
current pool epoch/supply and a permissionless zero-fee SOL-deposit profile. It
rejects even one lamport of historical book-value loss or value below HWM. Pending
assets, HWM, KIF/carry, rent and full token-native quarantine cannot subsidize a
loss and remain preserved. PIV authority must be System-owned, nonexecutable and
empty. Pool withdraw must be canonical/nonexecutable and exclude the exact
Token-owned 355-byte multisig form; it need not be System-owned.

These are unresolved runtime preconditions. The finite source error guide does
not diagnose a live transaction or replace the Rust guards. Output leaves quote,
effective minimum, before/after book values, eligibility, account/control/state,
resource sufficiency, fees, rent and funding unknown or unverified. It performs no
RPC, wallet/config/secret read, key generation, signing, sending, live conversion
or deployment. Public/deployment readiness remains false; M4's stopped operation
and unwaived criterion remain unchanged.


The distribution planner prepares four explicitly selected offline boundaries:

```sh
"$NODE" clients/cli/distribution-plan.cjs --template
"$NODE" clients/cli/distribution-plan.cjs --manifest clients/cli/distribution-plan.example.json
"$NODE" --test clients/cli/distribution-plan.test.cjs
```

Select `prepare-liquid`, `prepare-withdrawal`, `settle` or `integrate-pending` in
`action`. Declare the PIV program, external fee payer, four protocol accounts,
reference transport context and all three resource requests. Preparation also
requires `currentGuardians.registryRevision` and six `guardianKeys` in CURRENT
registry slot order. Withdrawal preparation additionally requires the explicit
`withdrawal.sourceStake` and `validatorIndex`. The index is a canonical decimal
u32 string; registry revisions are canonical decimal u64 strings, including values
above JavaScript's safe integer range. No list index implies a vote, suffix or
source address; production authenticates those current facts.

Settlement instead requires `frozenSettlement`: the active round's FROZEN
registry revision, six guardian keys in frozen slot order, `htfpRecipient` and
`teamOwnerRecipient`. These may differ from current governance state. Settlement
supplies no current registry account. Integration requires none of these blocks.
Keep unused blocks null or with only null members; populated unused action inputs
reject. Keys are never sorted or silently reassigned. Exactly six distinct
nonzero guardian identities are required; they need not be on curve and can also
be external beneficiaries or the fee payer. Guardian identities are not native
signer account roles. Canonical reward PDAs bind guardian bytes, u64LE revision
and u8 slot after the `guardian-reward` prefix.

Each native instruction starts with its eight-byte selector and version 1:
`PIV1PD01`, `PIV1SD01` and `PIV1IP01` are nine bytes; `PIV1PW01` appends only the
u32LE validator index and is thirteen bytes. Every native role is a nonsigner.
Shared/distinct manager-referrer shapes have 26/27 preparation accounts, 28/29
withdrawal-preparation accounts, 27/28 settlement accounts and 19/20 integration
accounts. Shared receivers appear once. Preparation makes Config, distribution,
pending SOL, principal SOL and escrow writable to cover both liquid funding
sources. Settlement makes Config, distribution, principal SOL, escrow, KIF vault,
two frozen recipients and six frozen rewards writable. Integration makes Config,
distribution, pending/principal SOL, escrow and both token vaults writable. Other
roles are readonly; canonical Clock is included and only withdrawal preparation
adds Stake plus the explicit source. Native data contains no caller amount,
target, technical minimum, source capacity, timestamp, eligibility bitmap,
recipient override or claim/leg instruction. Frozen recipient addresses appear
in settlement account metas; production binds them to the active round and grants
no caller-authorized recipient override.

The sole outer signer is the distinct on-curve fee payer. Actual native roles
must not alias, including recipients, source stake and derived reward addresses;
only the explicit manager/referrer collapse is supported. This syntax/control
check does not authenticate accounts, spending authority or funding. Complete
legacy packets contain three explicit resource requests and a full 64-byte zero
signature slot, with SDK serialization/decompilation and the 1,232-byte cap.
Shared/distinct sizes are respectively 1,092/1,125; 1,162/1,195; 1,125/1,158; and
861/894 bytes for the four actions above. No ALT is required by this profile.
Resource ranges match the intake planner: 1–1,400,000 CU, 32,768–262,144 heap bytes
aligned to 1,024, and 1–8,388,608 loaded-account bytes. They remain unverified
requests; reference blockhash/height/slot inputs must be refreshed independently.

All four economic transitions require unpaused state and authenticated whole
custody, current pool epoch/supply, rent, pending and HWM protections. Preparation
requires Idle and the ten-day minimum measured from successful preparation.
Liquid no-yield evaluation leaves state unchanged without a snapshot or clock
reset; positive shortfall rejects 6145 without recording valid insufficiency.
Withdrawal preparation authenticates the current active-source profile and Stake
minimum-delegation return data. A valid-insufficient result updates only its retry
timestamp/event and requires the separate 24-hour retry interval. A prepared
result freezes the round and liquid funding; this boundary creates no withdrawal
stake/leg and performs no token burn, deactivation or withdrawal.

Settlement requires EscrowFunded with exact target assignment/finalized legs,
escrow and protected-value reconciliation. Production derives net payments after
fees, frozen KIF eligibility, zero-active compounding/carry and retained dust; it
may instead record RecoveryRequired without payments. Integration requires Settled
and moves authenticated pending assets/residual escrow exactly once, includes full
contribution value in HWM and completes Idle. It does not stake. Rent, KIF
liabilities/carry, token-native quarantine and cooldown-yield classification stay
protected. RecoveryRequired means stop/report; this planner invents no recovery.

Missing/null required values return exit zero, `INCOMPLETE` and `plan: null`;
complete declarations return only `OFFLINE_PLAN_ONLY`. Unknown/duplicate JSON
fields, malformed known values, unsupported aliases/flags and mixed current/frozen
inputs reject before missing-input reporting. Output keeps outcomes, yield,
quotes, target, minimum, capacity, actual payments/value, eligibility, fees, rent,
funding and resource sufficiency unknown or unverified. The finite source error
guide diagnoses no live failure. These are source/wire drafts with no state
parser, economic validator, RPC, secret/config/wallet read, key creation, signing,
sending, funds, authority transfer or deployment. Public readiness stays false;
M4's stopped operation and unwaived criterion remain unchanged.

## Offline withdrawal-leg plans

`withdrawal-plan.cjs` accepts public declarations for two separate production
boundaries: `initiate-withdrawal-leg` and `finalize-withdrawal-leg`. It uses the
existing pinned SDK and encodes one unsigned legacy packet with three explicit
resource requests. Missing values produce `INCOMPLETE` and `plan:null`; a complete
manifest produces only `OFFLINE_PLAN_ONLY`, with public readiness and deployment
authorization false. It performs no RPC, wallet/key access, signing or sending.

```sh
node clients/cli/withdrawal-plan.cjs --template
node clients/cli/withdrawal-plan.cjs --manifest clients/cli/withdrawal-plan.example.json
node --test clients/cli/withdrawal-plan.test.cjs
```

Copy the null example and provide the exact public program, external fee payer,
Config-bound protocol tuple, declared `roundSequence`, reference blockhash/context
and requested CU/heap/loaded caps. All u64 values and the u32 validator index use
canonical decimal **strings**. Resource requests use integers under the existing
policy; requests do not prove actual resource or funding sufficiency. Unknown,
duplicate, malformed or non-null unused action inputs reject, including malformed
known partials before missing-value reporting. No economic amount, minimum quote,
timestamp, readiness flag or arbitrary destination is accepted.

For initiation, populate only `initiation`: `nextLegIndex`, `validatorIndex` and
`sourceStake`. Leave `finalization` null or unpopulated. Round and next-leg values
are declarations for deterministic `withdrawal-leg`/`withdrawal-stake` PDA seeds
(u64LE for each); only the validator index enters the 13-byte `PIV1IL01` payload.
Production reads the current sequence/index itself. Another successful initiation
can invalidate the draft. The source must pass current Active validator list,
canonical vote/suffix PDA, preference/source-order, Stake/lockup/authority, epoch,
capacity and technical-floor checks. Production selects the maximum-safe fill,
protected pool variant24 minimum and HWM continuation proof. This client calculates
none of them. Temp prefunds become pending contributions; OperationalSOL must
separately fund full current Stake/metadata rents. Withdrawal and immediate Stake
deactivation execute together; this action does not finalize or pay beneficiaries.

For finalization, populate only `finalization.legIndex` and leave `initiation` null
or unpopulated. The existing leg is derived from declared round/index, and the
17-byte `PIV1FL01` payload contains the u64 leg index. The fixed Clock, Stake and
StakeHistory roles are explicit. Real Stake execution must establish inactivity;
elapsed epochs and this offline report cannot prove cooldown readiness. Ready legs
may finalize out of order. Exact original advanced rents return to OperationalSOL,
metadata excess becomes pending, and ambiguous Stake surplus rejects. Cooldown
rewards remain next-cycle yield; observed loss or residual HWM failure records
`RecoveryRequired` without reducing HWM. Stop and report that state. Finalization
moves the complete ready balance into fixed escrow, closes metadata and updates
cumulative state; settlement remains separate.

Both actions require unpaused `WithdrawalActive` and fresh authenticated custody,
pool and round state. All native roles are nonsigner, with only the external fee
payer occupying one all-zero signature slot. Native addresses remain distinct;
shared manager/referrer uses one role. The payer must be on curve and distinct
from native/program/protected addresses. Payer/reference-blockhash overlap is
allowed solely as a public encoding declaration and does not prove a valid funded
payer. Operational rents are separate from the outer payer's network fees. Every
full packet, including its zero signature, is bounded at 1,232 bytes. No live
identity/control, account validity, rent/fees/funding, amount, eligibility, resource
sufficiency or action outcome follows from successful encoding. M4 remains
incomplete with its recorded criterion open/unwaived; M5 is preparation only,
public package NOT READY and deployment NOT AUTHORIZED.

## Offline guardian heartbeat and earned KIF claim plans

`guardian-kif-plan.cjs` encodes two separate production actions from public inputs:
`guardian-heartbeat` and `claim-kif`. Missing values return `INCOMPLETE` with
`plan:null`. Complete inputs return only `OFFLINE_PLAN_ONLY`; public readiness and
deployment authorization stay false. No RPC, wallet/key access, keypair creation,
signing or sending is implemented. The current heartbeat tuple and historical
earned-claim tuple remain independent.

```sh
node clients/cli/guardian-kif-plan.cjs --template
node clients/cli/guardian-kif-plan.cjs --manifest clients/cli/guardian-kif-plan.example.json
node --test clients/cli/guardian-kif-plan.test.cjs
```

Copy the null example and declare the exact program, fee payer, selected action,
reference blockhash/context and CU/heap/loaded limits. All u64 revisions, amounts,
counters and reference numbers are canonical decimal strings. Guardian indices
are integers 0..5; the heartbeat vault index is an integer 0..255. Unknown,
duplicate, malformed known partials and non-null unused action inputs reject.
Requested resource caps do not establish runtime sufficiency, fees or funding.

For `guardian-heartbeat`, populate only `heartbeat`: the CURRENT registry revision,
six guardian keys in PIV slot order, selected `guardianIndex`, `vaultIndex`,
`multisigAddress` and public Squads `createKey`. Leave `claim` null or unpopulated.
The planner checks the canonical multisig/create-key derivation and derives loader
ProgramData, six reward PDAs and the authority vault. The vault is a derivation for
review, not an instruction account or signer. Production must authenticate current
Config/registry/rewards, runtime Clock, loader Program/ProgramData authority and the
current Squads six-voter membership/permissions/threshold correspondence. This is a
direct guardian signature, with no proposal, four approvals or Squads CPI.

The exact 11-byte `PIV1HB01` payload contains version1, guardian index and vault
index. Its 13 native roles include the runtime Program again at role10; compilation
unions it with the instruction program once. Only the selected reward is requested
writable. The guardian is requested readonly signer and becomes globally writable
when it also pays the network fee. Runtime Clock and the Config anchor determine
the fixed 2,592,000-second half-open period; no caller activity time, period or
eligibility assertion is accepted. Heartbeat records activity without creating
KIF liabilities, transferring funds or changing a frozen distribution snapshot.
Post-snapshot activity is not retroactive. This production heartbeat path has no
pause check. It checks the selected signature but imposes no generic System-owner
or empty-account requirement on the heartbeat signer.

For `claim-kif`, populate only `claim`: the EARNED `guardian`, `registryRevision`,
`guardianIndex`, positive `amountLamports` and `expectedCumulativeClaimed`. Leave
`heartbeat` null or unpopulated. The reward PDA binds the earned guardian/revision/
slot even after guardian rotation or inactivity. No current registry, membership,
Clock, distribution, pool or Squads context is required. The 24-byte payload is the
Anchor `claim_kif` discriminator followed by two u64LE values: amount and expected
claimed counter. All four claim roles (Config, reward, KIF vault and guardian) are
writable; canonical executable System Program is readonly. The guardian must sign
and is the exact fixed payout destination. No alternative payout/source is accepted.

Claims remain allowed during global pause and pay only an already-earned liability.
Production requires exact current replay counter, individual/global claimability,
backing for source rent plus all KIF liabilities and collective carry, checked
arithmetic and exact post-transfer state/custody. It preserves carry, source excess,
state rents, principal, pending funds, operational rent, distribution custody and
HWM. A full-width encoded amount/counter may still fail these checks; no claimable
balance, backing, overflow safety or successful outcome is inferred here.

This planner supports direct EOA signer packets: selected guardian and fee payer
must be on curve. Other current guardian declarations may be off curve within the
production identity/alias rules. Production can support runtime PDA signers through
CPI, including a System-owned empty nonexecutable PDA claim wallet; composing that
CPI/signing workflow is unsupported by this planner, not forbidden by production.
Claim destinations must be System-owned empty nonexecutable accounts, while the
heartbeat signer has the separate signature/current-identity constraints above.
A fee payer can equal the selected guardian (one zero signature slot) or be distinct
(two slots), including another current guardian. Complete packets count every
signature and global signer/writable union within 1,232 bytes: heartbeat601/697 and
claim382/478 bytes for shared/distinct payer. Reference blockhash overlap alone is
an encoding declaration and does not prove a valid funded payer. No live control,
account validity, eligibility, liability backing, resource sufficiency or funding
is proved. M4 criterion remains open/unwaived; M5 preparation only, public package
NOT READY and deployment NOT AUTHORIZED.

## Offline guardian registry and earned reward inspection

`guardian-inspect.cjs` reads supplied public snapshot metadata/base64 bytes for
`current-set` or `earned-record`. It creates no transaction or command and performs
no RPC, wallet/key access, signing or sending. Missing fields return `INCOMPLETE`
with `inspection:null`; complete inputs return `OFFLINE_GUARDIAN_INSPECTION` while
public readiness and deployment authorization remain false.

```sh
node clients/cli/guardian-inspect.cjs --template
node clients/cli/guardian-inspect.cjs --snapshot clients/cli/guardian-inspect.example.json
node --test clients/cli/guardian-inspect.test.cjs
```

Copy the null example and declare `programId`, `mode` and source context. Every
account contains `address`, `owner`, `executable:false`, decimal-string `lamports`
and canonical `dataBase64`. Context slot is a u64 decimal string, not an assertion
of freshness or coherence. Unknown, duplicate, malformed known partials and
populated unused mode inputs reject. Account owner must equal the declared PIV
program; canonical addresses and stored bumps are checked against its PDAs.
Lamports are displayed metadata and are not tested against current Rent.

For `current-set`, populate `current.config`, `current.registry` and exactly six
slot-ordered `current.rewards`; leave `earned` null or unpopulated. The existing
Config decoder and bounded topology validator check the supplied Config owner,
address, initialized/version header, canonical PIV addresses/bumps and fixed
System/Token/Stake IDs. The registry must match Config's stored address, bump and
revision. Each reward must match its exact registry guardian, revision and slot;
slots are preserved without sorting. No Clock or Squads authority snapshot is
accepted or inferred. This is a declared current set, not proof of live membership
or current governance authority.

Registry allocation is exactly 210 bytes: discriminator, version, bump, u64LE
revision and six public keys. Reward allocation is exactly 84 bytes: discriminator,
version, bump, slot, u64LE revision, guardian key, variable Borsh `Option<u64>`
`last_active_period`, then three u64LE ledgers (`claimable_lamports`,
`cumulative_earned`, `cumulative_claimed`). `None` places the three ledgers at
byte52 and leaves eight trailing zero bytes; `Some` places the period at byte52
and ledgers at byte60, with no trailing allocation padding. Strict tags, zero tail,
version, nonzero/distinct registry keys, reward index and standalone reward ledger
identity (`claimed <= earned`, `claimable = earned - claimed`) are checked. This
bounded identity check is not full Rust semantic validation or custody accounting.

For `earned-record`, populate only `earned.reward` and leave `current` null or
unpopulated. The old reward's self tuple (guardian/revision/slot), owner, canonical
PDA and bump are checked independently of current Config/registry, membership,
Clock, distribution or pool. Retired guardians and off-curve stored guardian keys
are inspectable; no wallet signature/control or destination ownership is proved.
The summary exposes `expectedCumulativeClaimedSource` as the stored
`cumulative_claimed` field. It is an observation from supplied bytes, with freshness,
replay validity and claimability unverified; no ready claim command is produced.

All decoded u64 fields retain exact decimal strings, including full-width values.
Account hashes, allocation size, consumed payload and zero padding are reported.
Stored activity period is displayed without computing a current period, timestamp,
activity bitmap or eligibility. Current reward records are never summed into a
global liability because they do not enumerate historical earned records. Stored
claimable balances and Config ledgers prove no available payment, backing, rent
health, physical custody or solvency. Provenance, coherent snapshot, live freshness,
current authority, wallet control, resources and funding remain unverified. The
focused tests use explicitly labelled small synthetic literal bytes and retain
four complete CLI snapshot/report pairs; these are host decoding results only.
M4 criterion remains open/unwaived; M5 preparation only, public package NOT READY
and deployment NOT AUTHORIZED.

## Founder package and future operator workflow

The [founder workflow](../../docs/PIV1_TESTNET_FOUNDER_WORKFLOW.md) brings these
existing offline tools into one ordered preparation guide. The
[public package draft](../../docs/PIV1_TESTNET_PACKAGE_DRAFT.json) is an index of
source/artifact evidence, historical public observations and null founder/operator
selections; it is **not** a manifest accepted by any CLI. Use each tool's exact
existing schema and flags. The workflow lists pinned full-path Node sample
commands, expected offline statuses, actual source events, custody distinctions,
conditional rent and stop conditions. No existing client or test was changed or
rerun to prepare these documents.

The [single M5 register](../../docs/PIV1_M5_TESTNET_READINESS.md) remains the status,
evidence and decision authority. The retained PIV candidate/source binding is not
final artifact selection or deployment approval. Missing public identities/control,
fresh chain facts, full funding/fees/peak needs and usable live snapshot acquisition,
provisioning/signing/submission/confirmation transport remain OPEN. The founder has confirmed the intent for six disposable Testnet guardians and a
4-of-6 Squads setup. The workflow proposes a separate seventh external payer,
secure files and a Stage A key-generation-only approval, followed by a separately
reviewed Stage B creation scope after public identities and exact costs. Fresh
read-only image/config observations support bounded compatibility, not creation
execution. No key generation or transaction is authorized; private material never
belongs in Git/log/chat. The documents add no transport implementation.
M4 remains technically incomplete with its criterion open/unwaived and stopped
drafts untouched; M5 preparation only, public package NOT READY and deployment
NOT AUTHORIZED. Subsequent publication remains reviewed integration only.
