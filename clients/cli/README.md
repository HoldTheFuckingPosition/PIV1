# PIV1 offline initializer planner

**Current AN observation — full-cost target MISSED (2026-10-24):** root executes two new current-source937 builds and one candidate-first/legacy-scheduling-with-current-guards pair, once. Whole action including fsync/response/readback/cleanup takes 27.157954s current versus26.692696s counterfactual, net -0.465258s against provisional>=3s. Named semantics and exact simulated transport transcripts match; parent CPU is lower by1.054757s, which neither reverses the wall result nor proves its cause. No new tests/RPC/private/native/signatures/send; unchanged61/61 evidence is reused. Next is the FIRST separately reviewed fresh current7601 public original-lifetime observation, with actual genesis/version/height/validity/fee reads, unchanged >=40 at every boundary and an independent never-forwarded observer. The provisional >=3s full-cost target stays MISSED; prospectively replacing that experimental prerequisite with direct observation of the accepted >=40 criterion is a technical experiment dependency change, not a security waiver. AK used b468; current7601 has never had that public-lifetime observation. No unchanged cost retry, unproved CREATE micro-patch, old fixture adoption or signing/send authority follows. See operator `atomicCompoundCostObservation` and [pilot checkpoint](../../docs/PIV1_PILOT_STATE.md). M4 OPEN_UNWAIVED; M5 NOT READY / NO LIVE TRIGGER; no new live/main authority. Earlier dated outcomes retain their historical scope.

**Current assessment — identity matched, clone prototype rejected (2026-10-24):** two official read-only calls match pinned `4.4.0-beta.1` / features `3993424257`, not full compatibility. The prototype passes 68/68 but saves 0.169664s/replay, below 0.4s/0.5s targets; no adoption or lifetime trial. Exact original `b4687f24…`/`790b7954…` sources are restored, with original 57-test evidence; prototype bytes/tests remain outside Git. AK35<40/INTENT is unchanged. Next: source feasibility/review of atomic-stage or scheduling consolidation preserving distinct fresh barriers and original-only recovery; design remains OPEN. Evidence: operator `runtimeIdentityAndCloneAssessment`. M4 OPEN_UNWAIVED/M5 NOT READY / NO LIVE TRIGGER; no sensitive/live/main authority.

**Current compound observation — margin refused (2026-10-24):** one fresh `b4687f24…` late937 host flow stops at `post_intent_lifetime_margin`: original headroom 148→61→35, five below ≥40 despite raw validity true. No observer/SUBMITTED/UNKNOWN/forwarding; preserve offline INTENT without retry. Five read-only HTTP calls/13 requests cover lifetime/fee; synthetic accounts/Clock/NON-signatures and sparse Write limits remain. Unchanged 57/57 evidence is reused. Next: a reviewed materially sufficient controller/scheduling reduction with a target justified by the deficit and uncertainty before another fresh ≥40 observation. Exact receipts/seams: operator `currentCompoundLifetimeObservation`. No causal comparison with AI or native/readiness proof; M4 OPEN_UNWAIVED/M5 NOT READY / NO LIVE TRIGGER, D-056/D-057 consumed.

**Current replay-cost correction — original lifetime OPEN (2026-10-24):** replay-local source SHA sharing retains every fresh read/check; a fresh validated phase index preserves malformed-input fallbacks. Root passes 57/57 focused host tests first run. One fresh current-source sparse synthetic 937 prefix has identical complete replay states at 4.965143s enabled first versus 6.274904s literal SHA second: 1.309761s saved, meeting 1.2s but below preferred 1.6s. Indexing stays enabled in both; five-times 6.548806s is arithmetic only, with no compound/lifetime/native/readiness qualification. Historical 148→41→15 refusal remains preserved. Next: one reviewed fresh current-source compound observation retaining original ≥40 at all unchanged boundaries before remaining readiness checks. Exact evidence/seams: operator `replaySourceWorkCorrection`. No new RPC/signatures/private/native/live/main authority; M4 OPEN_UNWAIVED/M5 NOT READY / NO LIVE TRIGGER.

**Current controller observation — original lifetime refused (2026-10-24):** one fresh current-source late937 `CAPTURE_SIGN_SEND` host flow commits A/B/INTENT, then stops at `post_intent_lifetime_margin`: capture/preflight/post-INTENT margins 148/41/15 against ≥40. Raw validity true does not complete later product guards. No interception/SUBMITTED/UNKNOWN/forwarding; preserve this unresolved offline intent without retry/adoption. Synthetic state and fake NON-signature delays remain explicit; this is no native/public-state/readiness proof. Exact receipts: operator `compoundLifetimeObservation`; AH 49/49 remains retained unchanged. D-056/D-057 consumed, main `2c10794` unchanged; integration only, M4 OPEN_UNWAIVED/M5 NOT READY / NO LIVE TRIGGER.

The release operator accepts this request shape (placeholders are not executable inputs):

```json
{
  "schema": "piv1-startup-release-operator-v1",
  "action": "CAPTURE_SIGN_SEND",
  "expectedTail": "<exact current operator tail>",
  "data": {
    "packet": {"id": "<exact current phase>", "wireBase64": "<approved unsigned wire>"},
    "signerProfilePath": "<operator-supplied absolute private path outside repository>"
  }
}
```

Both `--allow-live-sign` and `--allow-live-send` plus the exact launcher authorization are required; output uses `REPORT` (`COMMIT_RECEIPT` remains PREPARE/SIGN only). Python test seams must supply both signer and transport or neither; injected history cannot become live. The profile path stays in the ephemeral request/backend payload, never compound journal records. Preparation uses a domain-separated content commitment over predecessor/entries/quote; existing standalone actions remain available. A lost response after the first compound commit permits strict original-response `IMPORT` only, and a second-checker failure leaves that unresolved attempt. No automatic resume, replacement signature, resend or weakened original-lifetime/INTENT/UNKNOWN recovery is introduced. The compound action exclusively creates its response/preflight directory before A; after A and both fresh barriers, it retains the bounded canonical returned response before later authorization or checker refusal. One fresh ephemeral import/attempt replay precedes public preflight; one durable C contains import, attempt, exact response receipt and complete INTENT. Exact postcommit readback rebinds dispatch inputs, then the original post-INTENT guards run. Before C, only explicit original-response IMPORT/HOLD is allowed; later standalone SEND uses a new sequence/directory and retains the orphan. After C, original-signature recovery/UNKNOWN remains mandatory. Four predispatch passes are host ordering, not late937/native/live readiness; a fresh exact package still needs separate live approval.

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


## Offline Squads creation plan

`squads-create-plan.cjs` encodes the exact retained Squads v4
`multisigCreateV2` interface with the existing pinned Node/SDK. It reads only an
explicit public manifest and pinned repository/SDK inputs. It has no RPC,
wallet/private-key reader, key generation, signing, submission or confirmation.
The [single M5 register](../../docs/PIV1_M5_TESTNET_READINESS.md) remains the status
authority: M5 preparation only, public package NOT READY, deployment NOT AUTHORIZED;
M4's criterion is open/unwaived. D-041's one-shot key/funding permission is consumed
and does not authorize creation or additional wallet spending.

```sh
NODE=/home/jerem/.local/piv1-toolchains/node-v24.19.0-linux-x64/bin/node
"$NODE" --max-old-space-size=1024 clients/cli/squads-create-plan.cjs --template
"$NODE" --max-old-space-size=1024 clients/cli/squads-create-plan.cjs --manifest clients/cli/squads-create-plan.example.json
"$NODE" --max-old-space-size=1024 --test clients/cli/squads-create-plan.test.cjs
```

Copy the null example to a separate public manifest, fill only its declared
fields, then use the same `--manifest` command with that file. Null/missing required
fields return `INCOMPLETE` with `plan:null`; malformed known partials, unknown or
duplicate fields, wrong network/genesis, unsupported profile or aliases fail.
Only `--template` and `--manifest FILE` are supported. No account or context is
acquired automatically. The package draft is an index, not this CLI's manifest.

Declare six distinct `guardians`, an explicit member `createKey`, and a separate
`payer` that differs from all six. For the dedicated funded inventory, root's
proposed role choice is guardian-0 (`BUuu…BYoT`) as createKey and external payer
(`H2Aa…pxt4`) as creator/rent/network-fee payer; copy full public addresses from
[the public identity record](../../docs/PIV1_TESTNET_IDENTITY_SETUP.json), never
from private files. Neither input inventory order nor sorted Squads members sets
PIV guardian slots. Encoded members sort by raw 32-byte key bytes. The explicit
createKey is stable if input order changes. Only payer/createKey must be direct
on-curve signer declarations; other members have no blanket curve restriction.
Reserved protocol/program/PDA roles and treasury/actor overlap are rejected by
this planner's supported profile. A reference blockhash overlap alone is encoding
context, not an account-role alias.

The six-guardian/threshold4 requirement is accepted. Its exact new creation profile
is still a proposal. Every profile field must be explicit: `threshold:4`,
`memberPermissionsMask:7`, `configAuthority:"none"`, `timeLockSeconds:0`,
`rentCollector:"none"`, `memo:"none"`. String `"none"` means a declared Borsh
None; JSON null means missing. Autonomous config authority, zero timelock and
no rent collector are not silently accepted by encoding. None rent collector
disables rent reclamation. This tool supports no alternate profile.

Supply `observations.programConfig` (canonical Squads ProgramConfig PDA),
`treasury`, exact decimal-u64 strings `creationFeeLamports`, `rent330Lamports`,
`slot`, and a canonical millisecond UTC `observedAtUtc` such as
`2026-10-19T00:00:00.000Z`. Rent must be positive. A nullable
`networkFeeLamports` is optional so the exact unsigned message can first be
quoted independently; when present it is a positive decimal-u64 string. The
conditional rent+creation total and optional combined total use checked u64
addition. Missing network fee leaves the combined total null and lists
`missingQuoteInputs`. These are declared, unverified zero-prefund costs, never
current quotes, available balances, a complete wallet budget or spending authority.
Later provisioning/ALT/PIV deployment/operational rent/capital/fees remain separate.

Supply `transport.recentBlockhash` and exact decimal-u64 strings
`lastValidBlockHeight`/`referenceSlot`. The expected genesis is the fixed official
Testnet genesis, not a live network check. Refresh genesis, actual deployed image,
ProgramConfig/current fee/treasury, actors/control, candidate PDA absence, rent,
fee quote and validity window before any separately authorized signed phase.
Derived ProgramConfig/multisig PDAs do not prove on-chain existence or absence.
The source checks treasury identity and skips fee transfer at zero fee; this does
not prove creation succeeds with an absent treasury. Root must review the exact
current composition and simulation separately.

Complete input returns `OFFLINE_PLAN_ONLY`. `plan` includes proposed profile,
input/sorted members, canonical PDA+bump derivations, six requested account roles,
219-byte native payload, declared cost/context limits, raw message/base64/hash and
one full 619-byte legacy packet. ProgramConfig is readonly; treasury/multisig are
writable; createKey is readonly signer; creator/payer is writable signer; System
is readonly. Both 64-byte all-zero signature slots and the full packet count toward
1,232 bytes. No ALT, compute-budget or priority-fee instruction is included;
default runtime limits apply and resource sufficiency is unverified. No current
membership, four approvals, signer control, authority, treasury validity, funding,
fee correctness, simulation, creation or readiness is established. The external
pinned source paths/hashes are retained interface references, not runtime file
requirements, deployed-source equivalence or a build attestation.

Focused host tests use independent literal payload and legacy-wire cursors,
canonical PDA/role oracles, all 720 member permutations with fixed createKey,
bounded generated full-u64 declarations with digit-arithmetic totals, strict
partials/aliases and CLI reports. These are finite encoding checks, not exhaustive
proof, runtime economics or live creation. Three actual CLI manifest/report pairs
are retained in a fresh `$TMPDIR/piv1-squads-create-public-test-*` directory. Their
quotes/contexts are synthetic; one uses the public funded inventory only, with no
private-file access. This planner never authorizes creation, deployment or further
funding; a future exact-package authorization must cover any signed action.


## Offline emergency pause and release through Squads

`pause-plan.cjs` prepares the exact unsigned governance sequence for an initialized
PIV program. Explicit `paused:true` requests pause; `paused:false` requests release.
This is an explicit set, never a toggle. It changes only `Config.paused` through
production's authenticated Squads boundary. Previous state, eligibility, the
`PauseChanged` event and successful execution remain unobserved.

```sh
NODE=/home/jerem/.local/piv1-toolchains/node-v24.19.0-linux-x64/bin/node
"$NODE" --max-old-space-size=1024 clients/cli/pause-plan.cjs --template
"$NODE" --max-old-space-size=1024 clients/cli/pause-plan.cjs --manifest clients/cli/pause-plan.example.json
"$NODE" --max-old-space-size=1024 --test clients/cli/pause-plan.test.cjs
```

The null example returns `INCOMPLETE` with `plan:null`. Copy it to a separate public
manifest and fill every declared field before generating packets. Only `--template`
and `--manifest FILE` are supported. Strict JSON rejects unknown/duplicate fields,
unused choices, malformed known partials, wrong action/network, role aliases and
unsafe numeric forms before returning incomplete reports. Complete input returns
`OFFLINE_PLAN_ONLY`; all current authority/control, actual votes, resources,
funding and execution authorization remain unverified.

Declare `programId`, external `feePayer`/`rentPayer`, `action:"set-pause"` and the
boolean `paused`. In `governance`, supply exact decimal-u64 strings
`registryRevision`, `transactionIndex`, `currentTransactionIndex` and
`staleTransactionIndex`; six `guardianKeys` in **current PIV slot order**;
`createKey`/canonical `multisigAddress`; explicit u8 `vaultIndex`; current member
`creator`/`executor`; four distinct current-member `approvers`; and explicit
`timeLockSeconds` (integer 0..7,776,000). No guardian sorting/remapping, slot/vault
selection, initial pause, anchor, caller Clock or automatic identity is supplied.
Known Testnet Squads `9mw…ScVS` does not authenticate a user manifest.

The declared next transaction index must equal current+1, fit u64 and exceed stale.
Creation uses actual current+1; stop if a competing creation invalidates that
context. Later unrelated creations may advance the global index: this action must
still satisfy actual index<=current and index>stale at execution. Current governance,
ProgramData authority, six-member correspondence and exact approved action must
still authenticate. Creator needs Initiate, executor Execute and approvers Vote;
source requires six current voters/threshold four/autonomous authority. These are
runtime prerequisites, not inferred permissions or four actual public approvals.

Fee/rent payers can overlap each other or guardian actors when canonical roles
remain distinct, giving one/two/three signer unions for creation and proposal.
Selected direct outer signers must be on curve; nonselected guardian declarations
have no blanket curve restriction. PDA/CPI signer-wallet composition is unsupported
by this direct client profile. Program/sysvar/native/PIV-PDA aliases are rejected,
including aliases already demonstrable in a partial manifest. A blockhash overlap
alone is encoding context and does not add an account role.

Supply `transport.recentBlockhash` and exact decimal-u64 strings
`lastValidBlockHeight`/`referenceSlot`. Supply explicit requested
`resources.computeUnits` (1..1,400,000), `heapBytes` (32,768..262,144, multiple 1,024)
and `loadedAccountBytes` (1..8,388,608). Each standalone packet includes these three
budget instructions. They are requested bounds, not measured sufficiency or fee
quotes; no priority fee is set. Refresh state and blockhash validity independently
for every future phase, especially after voting, review or timelock waits.

Seven packets are emitted: direct `vault_transaction_create`, `proposal_create`
with `isDraft=false` (already Active; no separate activation packet), four approvals,
and direct `vault_transaction_execute`. The stored message contains exactly one
11-byte `PIV1PS01`/version1/vaultIndex/bool instruction and sixteen native roles:
Program, ProgramData, multisig, proposal, transaction, vault, Instructions, Config,
registry, six slot-ordered revision rewards, Clock. Only Config is native writable;
vault is a readonly native signer supplied by Squads CPI. Multisig/proposal/transaction
stay readonly in stored/native metas. Outer execution makes the proposal globally
writable; reports retain both requested and compiled privilege unions. Vault never
appears as a top-level signer. The Program self-role shares its stored program key.

The stored compact message is 549 bytes with no lookup or ephemeral signer; direct
creation data is 564 bytes. Full legacy packets count all zero signature slots and
fit 1,232 bytes: create 891/987/1,083 and proposal 343/439/535 for one/two/three signers;
approval 301/397 and execute 765/861 for one/two. No ALT, buffer, wrapper or invented
truncation is needed. Actual live account ownership/rent/target absence, valid
Approved status, nonstale index, released timelock/runtime Clock and exact direct
Squads composition must still be independently checked. This is not public
sign/submit/confirm transport.

Pause does not create a withdrawal/recovery exit, reset cadence/HWM, unlock a frozen
distribution or change economics. Every economic handler keeps its production
pause/state/custody checks. D-043's one-shot external creation authorization is
consumed; H2 was drained and no new funding or pause/release spending is authorized.
The [single register](../../docs/PIV1_M5_TESTNET_READINESS.md) remains the status
authority: M4 criterion open/unwaived, M5 preparation only, public NOT READY and
PIV deployment NOT AUTHORIZED.

Seven focused host groups use independent literal native/compact/Borsh and full
legacy-wire cursors, canonical PDA oracles, generated full-width indices/revisions,
slot-order checks and one/two/three signer unions. Six complete synthetic CLI
manifest/report pairs (pause/release × shared/middle/distinct) are retained in a fresh
`$TMPDIR/piv1-pause-public-test-*` directory printed in the test diagnostic. These
finite encoding checks do not prove live votes, runtime success or exhaustively
validate production semantics. Retained upstream source/IDL paths/hashes are
interface references, not runtime dependencies or deployed-source attestation.

## Offline governed recipient replacement

`recipient-plan.cjs` prepares the exact production `PIV1UR01` recipient update
through seven unsigned direct-Squads packets: transaction creation, already-Active
proposal creation (`isDraft=false`), four distinct approvals, and execution. It
reads only an explicit public manifest and pinned local source/SDK files. Use the
same pinned Node 24.19.0 and web3.js 1.98.4 environment as the other planners:

```bash
NODE=/home/jerem/.local/piv1-toolchains/node-v24.19.0-linux-x64/bin/node
"$NODE" clients/cli/recipient-plan.cjs --template
"$NODE" clients/cli/recipient-plan.cjs --manifest clients/cli/recipient-plan.example.json
"$NODE" --test clients/cli/recipient-plan.test.cjs
```

The null example returns `INCOMPLETE` with `plan:null`. Supply public declarations
for the existing governance/transport/resource fields and
`recipients.{htfpVaultIndex,teamOwnerVaultIndex,htfpRecipient,teamOwnerRecipient}`.
Each explicit recipient address must match its u8 vault index under the declared
current multisig. Recipient indices must differ from each other and the governance
vault index; recipient addresses must avoid native/reserved/guardian aliases.
Known malformed or conflicting inputs reject even when other fields are missing.
No actual PIV identity, guardian slot assignment or recipient index is chosen.

The native 12 bytes encode selector `PIV1UR01`, version 1, governance vault index,
HTFP vault index and Team Owner vault index. The exact 18 roles extend pause's 16
with the two readonly recipient vaults. Only Config is writable, and the governance
vault is the only native signer, supplied through Squads CPI. The compact message
is 616 bytes; create data is 631 bytes. Full legacy packets include all zero
signature slots and three explicit requested resource instructions:

| Packet | Full bytes for supported distinct signer unions |
|---|---|
| Create, one/two/three signers | 958 / 1,054 / 1,150 |
| Proposal, one/two/three signers | 343 / 439 / 535 |
| Each approval, one/two signers | 301 / 397 |
| Execute, one/two signers | 831 / 927 |

All fit 1,232 bytes with no ALT, buffer or top-level vault signature. Stored/native
governance roles stay readonly while the outer proposal becomes globally writable.
PIV guardian input remains current registry slot order; Squads member sorting must
not reorder it. Revision and transaction indices retain full u64 precision.

Runtime must authenticate the current ProgramData upgrade-authority vault/multisig,
Config/registry/reward tuples, actual four current approvals, nonstale index,
permissions, Clock/timelock and exact approved single child. Current authority is
dynamic; the tool establishes no historical multisig continuity. Each new recipient
must actually be System-owned, nonexecutable, empty-data, and funded with positive
lamports at least the current zero-data rent floor. The derivation proves neither
funding nor exclusive 4/6 spending control. Program-owned recipients are unsupported
by this boundary. No pause or distribution-phase prerequisite is invented.

Successful native execution replaces current Config recipients and emits
`RecipientsUpdated`; previous values and actual outcome remain unknown. Active
rounds keep their frozen destinations. Native update moves no economic funds or
rent, creates no account, and changes no pause, cadence, HWM or KIF rule. Parent
Squads state creation needs separately funded payer rent and network fees.

Complete reports remain `OFFLINE_PLAN_ONLY`; readiness/authorization checks stay
false and actual funding/resources/eligibility remain unknown. The shared reference
blockhash is encoding context: refresh each standalone phase after creation, votes
and timelock waits. There is no RPC, wallet access, key creation, signing, submission
or provisioning. D-043 is consumed, H2 was drained, and no further spending or
recipient update is authorized. M4 remains open/unwaived; M5 preparation/public
NOT READY/PIV deployment NOT AUTHORIZED.

The focused suite uses independent literal native/compact/Squads and full legacy
cursors, signer unions, finite u64 boundaries, deterministic slot/u64 cases and a
256-case cyclic u8 domain (not every distinct index triple). Its three complete
synthetic public CLI manifest/report pairs (`recipient-shared`, `recipient-middle`,
`recipient-distinct`) are retained under the printed `/tmp/piv1-recipient-public-test-*`
directory. These are host encoding results when run, not Solana runtime evidence.

## Offline guardian one-seat synchronization

`guardian-sync-plan.cjs` prepares only PIV synchronization **after** a separately
authorized and executed external Squads membership change. It supplies no external
membership rotation, old-set vote proof or historical multisig-continuity proof.
The current replacement set separately approves this exact PIV instruction.

```bash
NODE=/home/jerem/.local/piv1-toolchains/node-v24.19.0-linux-x64/bin/node
"$NODE" clients/cli/guardian-sync-plan.cjs --template
"$NODE" clients/cli/guardian-sync-plan.cjs --manifest clients/cli/guardian-sync-plan.example.json
"$NODE" --test clients/cli/guardian-sync-plan.test.cjs
```

Use public nullable declarations only. `governance.guardianKeys` and
`registryRevision` describe the old current PIV slots/revision;
`governance.currentMemberKeys` describes the six current Squads members after the
external change, without assigning their array order to PIV slots.
`replacement.{guardianIndex,newGuardian}` identifies exactly one old slot and a
key outside the old six. Replacing that slot must produce exactly the current
member set. Revision must allow checked u64+1. Current creator/executor/four
approvers must belong to the replacement set. Declare `nativeRentPayer` separately
from parent Squads `rentPayer` and `feePayer`; permitted actors may overlap.
Selected direct signers must be on curve; unselected members have no blanket curve
restriction. Known contradictory or malformed partial inputs reject before an
incomplete report. No actual identity, slot or index is chosen.

Native `PIV1UG01` is 51 bytes: selector, version1, governance vault u8, old slot u8,
expected old revision u64, new guardian32. Exactly24 roles include the old16,
six new reward PDAs at revision+1, native rent payer and System. Native signers are
the readonly governance vault via Squads CPI and writable external native payer;
writable roles are exactly Config, registry, six new rewards and native payer.
Old rewards remain readonly. Compact message853 bytes/create data868 bind the
exact complete child, no lookup or ephemeral signer.

| Packet | Full legacy bytes with three explicit resource requests |
|---|---|
| Direct create, one/two/three creation signers | 1,195 / 1,291 / 1,387 |
| Buffered create, two/three creation signers | 1,232 / 1,232 |
| Buffer initial chunk, two/three creation signers | 761 / 665 bytes |
| Following extend with distinct fee/creator | 492 / 589 |
| Create from buffer, one/two/three creation signers | 381 / 477 / 573 |
| Proposal, one/two/three creation signers | 343 / 439 / 535 |
| Each approval, one/two signers | 301 / 397 |
| Execute, one/two/three fee/executor/native-payer signers | 997 / 1,093 / 1,189 |

The planner uses direct creation when it fits1,232 bytes. Oversized creation uses
canonical adaptive buffer creation/extension and creation from buffer. Declare
`governance.bufferIndex` explicitly when buffering is needed; a missing index
returns `INCOMPLETE/plan:null`. Every supplied index is validated. Each upload
includes its actual signature union and resource overhead; initial chunks are
capped at800 bytes. Full-buffer SHA256/size and six-zero-byte finish placeholder
are exact. Buffer closure refunds its rent to **creator**. Parent buffer,
transaction/proposal creation/reallocation rent and network fees remain unknown.
After creation: already-Active proposal (`isDraft=false`), four distinct current-set
approvals, direct Squads execute. Every emitted packet fits1,232 bytes; no ALT or
top-level governance-vault signature is required. Native payer signs execute.

Runtime must freshly authenticate dynamic ProgramData authority/current multisig,
old Config/registry/reward tuples, exactly one-seat/current-set relationship,
actual current approvals/permissions, nonstale index, Clock/timelock and exact child.
No pause or phase gate is invented. New targets must actually be empty System
accounts; native execution funds each shortfall to the current84-byte rent floor,
allocates/assigns six new reward records, and leaves the native external payer its
zero-data rent floor. Actual prefunds, costs, balance/control and resource
sufficiency are unknown. Native rent comes from the external payer, separately
from economic funds and parent Squads costs.

Success changes current Config revision, registry and six new records. Continuing
guardians retain recorded last activity; newcomer activity is not invented. Old
earned records, original entitlement, liabilities/counters, frozen distribution
snapshots/recipients and economic ledgers remain preserved. Intended
`GuardianSetUpdated` is unobserved; actual funded rent and outcome stay unknown.

Null input returns `INCOMPLETE/plan:null`; complete output is `OFFLINE_PLAN_ONLY`
with all live readiness/authorization checks false. The pinned Node24.19.0 and
web3.js1.98.4 are unchanged. No RPC, keys, signing, submission, funding operation,
deployment or external membership change occurs. Refresh each standalone phase's
encoding blockhash/context after uploads, votes and waits. D-043 is consumed,
H2 was drained; M4 remains open/unwaived, M5 preparation/public NOT READY/PIV
deployment NOT AUTHORIZED.

Seven focused groups use independent native/compact/Squads/full legacy cursors,
bounded deterministic u64/slot/member-order cases, exact upload limit+1 checks,
hash/size reconstruction and signer unions. Three synthetic CLI pairs
`guardian-sync-{shared,middle,distinct}` retain25 complete packets under the
printed `/tmp/piv1-guardian-sync-public-test-*` directory when the suite runs.
These are host encoding results, not new Solana runtime or custody evidence.


### Initializer using a supplied existing ALT snapshot

`initializer-existing-alt-plan.cjs` reuses the initializer's source-pinned encoding
and emits only the remaining Squads buffer, proposal, four approvals and v0
execute packets. It emits no ALT creation or extension: `newAltSetupPackets` is 0
and `newAltSetupRentOutflow` is `"0"`. These values do not quote complete fees,
project funding or a historical rent refund. Old initializer tools are unchanged.

Use the pinned Node 24.19.0 and web3.js 1.98.4 installation:

```sh
node clients/cli/initializer-existing-alt-plan.cjs --template
node clients/cli/initializer-existing-alt-plan.cjs --manifest clients/cli/initializer-existing-alt-plan.example.json
node --test-reporter=tap --test clients/cli/initializer-existing-alt-plan.test.cjs
```

The public wrapper has `schema`, the original nullable `initializer` manifest,
and `existingAlt`: canonical `address`, `owner`, `executable`, decimal-string
`lamports` and `contextSlot`, and canonical `dataBase64`. The supported table has
exactly 568 bytes, active state, a present matching authority and the exact 16
source-derived targets in order. Raw u64 slots stay decimal strings. The supplied
context must equal `initializer.transport.referenceSlot` and be strictly later
than the table's last extension. Active-table reuse does not require the original
derivation slot to remain in current SlotHashes. No ALT-authority signature is
added merely for reuse; original initializer declaration validation is preserved.

Complete reports remain `OFFLINE_PLAN_ONLY`; missing critical inputs yield
`INCOMPLETE` with `plan: null`. Supplied bytes and context are unauthenticated,
not current observations or warmup/owner/control/rent-health/funding assurance.
Independently refresh the table and blockhash before any separately authorized
signed operation. No identity, KIF anchor, resource sufficiency or real eligibility
is selected. Focused generated tests cover finite domains, not all possible inputs;
the two retained synthetic CLI reports are not live observations.

## Offline pinned ELF upload-resume batch planner

`loader-upload-resume-plan.cjs` plans bounded unsigned loader-v3 Write batches for
the retained 894,552-byte candidate (`e29735e1467da98ecddb1eea9fd25abaffa4a450bd899ddc137f7d8c0da35b99`).
It accepts only the accepted Testnet PIV/G0 profile and the G0-seeded Buffer
`HGP9XhQmoHGCUbgoSATZ19zZHonxWB3uo1hKF3SK3M6K`, seed
`piv1-testnet-e29735e1`. Artifact and complete Buffer bytes must be supplied as
canonical base64 in the public manifest; no artifact/key path is accepted.

```sh
node clients/cli/loader-upload-resume-plan.cjs --template
node clients/cli/loader-upload-resume-plan.cjs --manifest clients/cli/loader-upload-resume-plan.example.json
PIV1_UPLOAD_TEST_ARTIFACT=/tmp/piv1-bank-smoke-guardians-sbf-20261018-b/artifacts/piv1.so node --test-reporter=tap --test clients/cli/loader-upload-resume-plan.test.cjs
```

Use the existing pinned Node v24.19.0 and web3.js 1.98.4 environment. Tests require
the public retained ELF through the test-only environment variable above. This is
not a planner file input. Root retains actual synthetic CLI fixtures and reports
under `/tmp/piv1-m5-upload-resume-20261021-a/`; a fixture manifest can be inspected
with `node clients/cli/loader-upload-resume-plan.cjs --manifest /tmp/piv1-m5-upload-resume-20261021-a/cli-zero64.manifest.json`.

The schema `piv1-loader-upload-resume-plan-v1` requires `cluster`, `genesisHash`,
`programId`, canonical `programDataAddress`, `writer`, `feePayer`, `bufferAddress`
and `bufferSeed`; all identities are pinned. `artifact` supplies `bytes`, `sha256`
and `dataBase64`. `snapshot` supplies a decimal-string `contextSlot`, a `buffer`
object with explicit `exists: true`, address/owner/executable/lamports/declared
`rentExemptMinimumLamports` and full `dataBase64`, and separate `program` and
`programData` objects with exact addresses, explicit `exists: false` and matching
`contextSlot`. Missing existence inputs remain missing; they never mean absent.
The Buffer must be non-executable, loader-owned, correctly sized, at least the
positive declared rent floor, with exact 37-byte Buffer/Some(G0) metadata.
Supplied rent and absence assertions are not authenticated chain observations.

`transport` requires a canonical `recentBlockhash`, decimal-string u64
`currentBlockHeight`, `lastValidBlockHeight` and positive
`minimumRemainingBlockHeights`. Their exact integer difference must satisfy the
declared margin. The margin does not guarantee throughput or a whole upload's
lifetime. `batch.maxPackets` is an integer from 1 through 64. Each generated batch
uses exactly its one supplied blockhash. No refresh or automatic next batch runs.
Public input is limited to 3 MiB, depth eight, exact known fields, canonical base64
and u64 decimals; duplicate JSON keys and trailing/extra account bytes are rejected.

The planner checks the entire artifact and all 936 chunks before assembling any
packet. It first compares each complete chunk with the target bytes. Equal bytes
are `MATCHED_BYTES`; remaining entirely zero chunks are `ZERO_PENDING`; any other
bytes stop the whole plan, including a conflict beyond the first batch. Equality
proves no earlier Write, signature or transaction. It emits only the lowest pending
indices up to the explicit batch bound: 956 bytes per full chunk and a 692-byte
tail. Retained resource requests are 1,400,000 CU, 262,144 heap bytes and 64 MiB
loaded bytes; these are deliberate caps, without new consumption measurements.
Full and tail unsigned legacy packets are 1,232 and 968 bytes with one zero G0
signature. A 957-byte full chunk would need 1,233 bytes and is unsupported.

Reports include all chunk statuses/digests, supplied progress, `pendingIndices`,
`selectedIndices`, and `remainingIndices` excluding this batch's planned writes.
Excluding an index does not claim its write executed. Packet reports include full
wire/message base64, hashes, accounts, privileges and signature roles. Even a fully
matching Buffer emits zero packets with operational/current/authorized flags false.
Null inputs return `INCOMPLETE` and `plan: null`. No fees, wallet sufficiency,
bootstrap trust, artifact selection, real Clock anchor, deployment or readiness is
inferred. Independently refreshed state and lifetime must precede each separately
authorized future batch; D-048 remains conditional and M4 remains OPEN_UNWAIVED.

### Offline public startup packet and reconciliation checker

`startup-public-check.cjs` verifies supplied complete signed legacy/v0 packets and a supplied hash-linked public phase trace. It has no wallet, signing, RPC, simulation, submission or journal-write API. This checker does not authorize or execute startup and always reports `liveReady:false`, `authorized:false`, `publicInputsAuthenticated:false` and `founderAuthorizationVerified:false`.

```sh
node clients/cli/startup-public-check.cjs --template
node clients/cli/startup-public-check.cjs clients/cli/startup-public-check.example.json
node clients/cli/startup-public-check.cjs /path/to/public-startup-check.json
node --test clients/cli/startup-public-check.test.cjs
```

Use the existing pinned Node toolchain where Node is not on PATH. `PIV1_PUBLIC_CHECK_REPO` optionally locates the public repository when reviewing E-directory copies; it does not locate a wallet. The module pins the existing lockfile and web3.js1.98.4 entry file. No installation or dependency change is required. This is a bounded fixed-packet trace checker: use it for a bounded upload segment or frozen P6 proposal, not the whole P0..P7 controller. All future phases need not and cannot be signed before P5 captures the real Clock. A new explicit Clock-freeze boundary and authenticated linkage between segments remain external; the 16MiB input cap cannot hold936 repeated full Buffer snapshots. The nullable example returns INCOMPLETE; supplied invalid/pending/unknown traces return STOP (CLI exit1); only fully reconciled supplied phases return VERIFIED_PUBLIC_DECLARATIONS_ONLY (exit0). Successful checking is not live readiness or authority to send/replay a packet.

Top-level fields are exactly `schema`, `frozen`, `cumulativeFeeCapLamports`, `phases`, `trace`. Unknown/missing fields and duplicate JSON keys are rejected. Integers representing lifetimes, slots, indices, balances and fees are canonical unsigned decimal strings within u64. Startup anchor is null for a pre-anchor segment, or a nonnegative signed-i64 string after the explicit Clock-freeze boundary. A null-to-value change inside one trace is rejected; negative anchors and i64 overflow are unsupported.

`frozen` contains `packageSha256`, `payloadSha256`, `anchorUnixTimestamp`, `transactionIndex`. Each trace entry repeats these exact values. This binds supplied values within a trace; the checker does not decode the native initializer payload to independently prove that the supplied payload/anchor/index describes it. Source-specific payload/Clock/index and founder package review remain necessary. Anchor0 in historical fixtures is an encoding declaration, not an actual selected Testnet anchor.

Each `phases` item contains:

- `id`: unique1..64 ASCII letters/digits/underscore/hyphen; array order is the strict dependency order.
- `packet`: `signedWireBase64` and `reviewed`. The reviewed object contains the exact `messageBase64`, `messageSha256`, `feePayer`, `requiredSigners` in actual header order, `accountRoles` (`address`, `signer`, `writable` for every resolved account), and `lookupTables` (each `address` plus ordered `addresses`). All tables needed by v0 must be supplied. These are reviewed public declarations, not authenticated account snapshots. Strict parsing rejects noncanonical base64/shortvec, truncation/trailing bytes, unsupported versions, invalid indices, instruction program index0, empty v0 lookup descriptors and duplicate resolved accounts. All signatures must be present, nonzero and valid Ed25519 signatures on the exact reviewed message; Node native `verify` uses public SPKI keys only.
- `lifetime`: `recentBlockhash`, `currentBlockHeight`, `lastValidBlockHeight`, `minimumRemainingBlockHeights`. Blockhash must equal the actual message. The margin must be positive and fit. Values are supplied declarations; no current chain/blockheight/time is obtained. Packet, signature and lifetime cannot be rebound within a supplied trace.
- `feeQuoteLamports` and `feeCapLamports`: exact declared quote<=cap. A successful/failed finalized receipt must declare the same actual fee. Reserved submission fees and finalized actual fees are separately accumulated within u64 and the overall cap; a pending fee is not silently refunded or spent twice.
- `expectedPoststate`: a nonempty reviewed account subset keyed by public address. Every account uses exactly `exists`, `owner`, `executable`, `lamports`, `dataBase64`. Absence requires the last four fields null. Present data is complete canonical base64 and balances are u64 strings. Expectations come from independent source/state review. Exact equality proves only this declared subset, not the whole account universe or the completeness of the chosen state oracle.

Each `trace` entry has exactly `previousDigest`, `phase`, `state`, `packetSha256`, `signatureHex`, `frozen`, `observation`. The first predecessor is null; subsequent predecessors are SHA-256 of the previous entry's recursively key-sorted canonical JSON (`entryDigest` export). `signatureHex` is the complete128-lowercase-hex-character first wire signature, normalized from public RPC evidence by the collector. Hash/signature bindings must equal the phase's complete packet. The checker rejects reuse of a packet or first signature under another phase.

Supported transitions are PREPARED → SUBMISSION_START_RECORDED → UNKNOWN or FINALIZED/FAILED; UNKNOWN may remain UNKNOWN or resolve to FINALIZED/FAILED. The next phase can begin only after its predecessor finalized with exact expected poststate. Successive finalized phases must have strictly later supplied inclusion slots; equal and earlier slots return STOP because this bounded sequential workflow has no same-slot transaction-order witness. This is a supported-workflow restriction, not a claim that Solana forbids same-slot dependencies. PREPARED and SUBMISSION_START_RECORDED are supplied records, not proof that an exclusive durable journal or network send exists. A submission-start record reserves the declared fee once. A timeout is represented as UNKNOWN. No state allows replacing, re-signing, resending or refreshing an UNKNOWN packet. Null history/status and blockhash expiry never establish non-inclusion. FINALIZED_FAILURE returns STOP with its fee and no phase advancement. This checker does not provide a retry/recovery authorization path.

For FINALIZED, `observation` contains:

- `status`: `signatureHex`, `slot`, `confirmationStatus:"finalized"`, `err:null`.
- `transaction`: `slot`, exact full `wireBase64` from a getTransaction base64 result, `feeLamports`, `err:null`. Slot must match status and the complete wire must match the frozen packet.
- `poststate`: `contextSlot` at least the transaction slot and `accounts` with exactly the reviewed expected subset's keys and complete raw values.

A FAILED receipt instead requires matching nonnull status/transaction errors and `poststate:null`; it records a supplied finalized failure only and proves no rollback/state predicate. Nonterminal `observation` must be null. Unknown outcome, mismatched fields, missing subset entries, changed payload/Clock/index, excessive fees or insufficient lifetime all stop the check. Reconciliation does not authenticate a fabricated status/transaction/map; TLS collection and retained raw response binding must be reviewed separately. Hash-linking a supplied trace does not prove its history, chronology or authenticity.

The self-contained7,711-byte `startup-public-check.fixture.json` retains three already-public signed D-047 legacy transactions/five signatures, the actual fund source/H2/G0 raw poststate subset and retained unsigned v0/ALT16 metadata. Its source paths/hashes describe provenance; focused tests do not require those /tmp files or a full ProgramData image. All signed packets are immutable consumed historical evidence, never executable startup packets. Positive real-signed v0 evidence is unavailable. A retained unsigned current initializer provides v0 structural/ALT coverage and must fail actual-signature verification. Deterministic generated properties cover signature corruption, all u16 shortvec roundtrips and truncation/trailing boundaries without new keys, signing, dependencies or a property-testing framework.

Remaining operator work includes authenticated TLS/raw RPC collection, correct expected-state oracle coverage, fresh Clock/payload binding and dynamic resources/lifetimes, exclusive durable journal writes, signer custody/authorization checks, a controlled submission path, actual status/history availability and crash recovery. These are not mechanically supplied by this pure checker. A/B acceptance, D-048/D-049 and a successful public declaration report do not authorize new live operations; the completed exact bounded package still needs separate founder authorization.

### Fixed-origin public startup reconciliation collector

`tools/collect_startup_public_evidence.py` collects read-only evidence from the literal official Solana Testnet origin `https://api.testnet.solana.com`. It does not sign, simulate, send transactions, move funds, change authorities, or create an execution/submission journal. It always reports live readiness/authorization and cryptographic chain-state authentication false. The normalized receipt is an input to the separate pinned `clients/cli/startup-public-check.cjs`; collection does not itself verify Ed25519 signatures or expected application state.

```sh
python3 tools/collect_startup_public_evidence.py --template
python3 tools/collect_startup_public_evidence.py --validate tools/collect_startup_public_evidence.example.json
python3 tools/collect_startup_public_evidence.py --collect /path/to/public-request.json --output /brand-new/public-evidence-directory
python3 -m unittest discover -s tools -p test_collect_startup_public_evidence.py
```

Only `--collect` performs network reads. Template and validation commands do not create evidence directories or use the network. The nullable example is INCOMPLETE. `PIV1_PUBLIC_CHECK_REPO` optionally locates the public repository for E-directory source review; it cannot select a network endpoint, wallet or response fixture. Programmatic transport injection exists only for offline unit tests, is not exposed by the CLI, and marks reports OFFLINE_INJECTED_TEST. A non-None hook must be callable and is selected explicitly even if falsey; it never silently falls back to real TLS.

The request has exactly the fields in the template:

- `wireBase64`: complete canonical base64 legacy/v0 wire, at most1232bytes; the first signature is derived from a canonical bounded signature-count prefix. The source validates the header's signature count and binds wire/message hashes. Complete message/ALT parsing and signature verification remain the pinned checker’s responsibility.
- `expectedWireSha256`: exact reviewed lowercase SHA-256. `expectedMessageSha256` is optional/null; when present it must equal the derived full message digest. `signature` is optional/null; when present its canonical64-byte Base58 value must equal the first wire signature.
- `expectedFeeLamports`: required exact reviewed fee, as a canonical unsigned decimal string within u64. Actual transaction metadata must match this amount; no fee is inferred from signature count or presumed priority pricing.
- `accountKeys`:1..64 unique canonical32-byte public Base58 keys. The upstream reviewer chooses a sufficient expected subset; the collector cannot establish its completeness.
- `minimumContextSlot`: required canonical u64 decimal string. The account query uses max(this minimum, actual included transaction slot). Actual returned context must meet it.
- `includeExpiry`: boolean, default false. Optional current finalized height/blockhash-validity reads report only present validity, never non-inclusion. An already-finalized old transaction may reconcile with an expired blockhash.

A successful sequence uses three HTTP calls/four logical reads: verify exact Testnet genesis; getSignatureStatuses(searchTransactionHistory=true) plus getTransaction(base64/finalized/maxSupportedTransactionVersion=0); getMultipleAccounts(base64/finalized/minContextSlot). Optional expiry adds two logical reads in one HTTP call, so the hard bound is six logical reads/four HTTP attempts. There is no automatic retry, alternate endpoint, redirect, proxy or fallback cluster. Direct stdlib HTTPS uses certificate verification and the fixed host/port/path. Nonempty HTTP_PROXY/HTTPS_PROXY/ALL_PROXY and lowercase equivalents cause rejection; NO_PROXY cannot enable a proxy. A proxy rejection can be recorded as a started transport attempt with zero completed response. Request IDs must match exactly; batch response order may differ. RPC errors, malformed/duplicate IDs or JSON, HTTP/redirect/transport errors stop without another attempt.

Raw requests, started records, raw responses, HTTP status/headers and response-completion UTC (alongside the request-start UTC), a normalized observation when reached, and a final manifest are preserved with byte lengths/SHA-256. The output directory must be new; existing directories/symlinks are rejected. Parent directories are opened component-by-component using O_NOFOLLOW; new directory0700 and files0600 use exclusive creation, file fsync and directory fsync. Files are never overwritten or silently reused. A request and started record are fsynced before transport; only after that durable intent succeeds are attempt/logical-start counters committed and the transport invoked. If intent persistence fails, transport is not invoked and those counters remain unchanged. A crash after the boundary leaves intent evidence, without proving that the socket reached the server. This is public evidence persistence; it is not a durable exclusive submission journal or a promise of recovery after host/disk failure. If storage fails, collection cannot report success; already-created evidence remains and the directory must not be reused.

The manifest counts attempted HTTP starts, completed responses, transport failures, logical reads started and observed RPC errors separately. A started attempt does not prove a socket request reached the server. Partial transport body bytes are retained when available; an EOF with positive remaining declared Content-Length is a transport failure even if the partial body happens to contain valid JSON. A malformed/error batch is preserved in full before parsing; server text is untrusted evidence, never an instruction. Status/slot/fee/lamports fields are exact Python integers converted to u64 strings, with bool/float/overflow rejected. Unused legitimate UI float fields are retained in raw JSON and do not affect integer checks. JSON duplicate keys and NaN/Infinity literals are rejected. Account data must be canonical base64 at most1MiB per selected account; larger external program images remain a separate preflight scope.

Only actual None accounts become explicit absent records (`exists:false` and four null fields). Present accounts preserve exact owner/executable/lamports/full data. A normalized observation has exactly the checker’s `status`, `transaction`, `poststate` shape. Successful collection requires supplied finalized status with err=null, exact returned full transaction wire, matching wire-domain version/inclusion slot and reviewed fee, plus a current finalized raw account subset at an adequate context slot. Null/pruned history or pending status yields UNKNOWN, mismatches/errors yield STOP, and a finalized transaction failure yields STOP with poststate=null. These states never authorize advancement or replacement. Expiry and null history do not prove non-inclusion.

The collector’s current account maps are observed at their returned contexts, not reconstructed transaction-inclusion poststate. An old transaction plus today's accounts cannot prove historical deltas. The subsequent checker must use independently reviewed expectations appropriate to the actual current context; no arbitrary account subset becomes a complete state oracle. TLS origin/certificate policy is transport evidence, not a chain light client or proof of RPC honesty. Sources, observations and raw receipt hashes still need separate review. The collector does not choose the Clock anchor, payload, governance index, spending scope, resource limits, signing/send path, durable segment linkage, retry policy or final startup authorization. M4 remains TECHNICALLY INCOMPLETE / OPEN_UNWAIVED, and D-049 requires separate authorization of the completed exact bounded package before new live operations.

### Offline durable public journal and linked checker segments

`tools/startup_public_journal.py` persists public descriptors/events across process reopen using exclusive creation, hash-linked immutable records, a stable lock, caller-retained tail compare-and-swap, and file+directory fsync. It invokes the existing pinned public checker on reconstructed actual packet/trace inputs. It has no network, signer, transaction sender, automatic retry/adoption, recovery or live-authorization API. Every report retains liveReady=false and authorized=false.

```sh
python3 tools/startup_public_journal.py --template
python3 tools/startup_public_journal.py --request tools/startup_public_journal.example.json --journal /never-created/nullable-example
python3 tools/startup_public_journal.py --request /public/request.json --journal /public/journal-directory
python3 -m unittest discover -s tools -p test_startup_public_journal.py
```

The nullable example returns INCOMPLETE without opening the named directory. Requests have exactly `schema`, `action`, `expectedTail`, `segment`, `event`; schema is `piv1-startup-public-journal-request-v1`.

| Action | expectedTail | segment | event |
|---|---|---|---|
| CREATE | null | `{id,input}` | null |
| START_SEGMENT | exact last acknowledged `{sequence,digest}` | `{id,input}` | null |
| APPEND_EVENT | exact last acknowledged `{sequence,digest}` | null | one complete existing-checker trace event |
| INSPECT | exact externally retained `{sequence,digest}` | null | null |

`sequence` is a canonical u64 decimal string; `digest` is lowercase SHA-256. CREATE requires a brand-new directory and creates record0 for the first segment. Every reopen/inspect/mutation requires the caller's retained receipt; the tool never defaults it from disk. Protect that receipt independently of the journal. A stale/missing/wrong receipt stops, even if local records otherwise look valid. Whole-directory rollback cannot be detected without a protected expected receipt; local hash chains are not external authenticity. A receipt does not authenticate founder approval or chain state.

A segment has a unique ASCII `id` and `input`, which is the complete existing `piv1-startup-public-check-v1` input with trace exactly[]. Its signed packets/reviewed messages, frozen tuple, fee caps, lifetime declarations, expected account subset and phase order become immutable. APPEND_EVENT adds exactly one supplied checker trace event, rebuilding each segment from its original descriptor plus durable event prefix. No caller-supplied success report is accepted. New segments require every predecessor actually verified complete by the pinned checker. UNKNOWN can be durably recorded and resolved using exact valid evidence, but cannot open a dependent segment; FAILED halts its segment. Any signature, poststate, lifetime or other checker rejection prevents the append and surfaces its diagnostic without changing successful history. Retain rejection stdout externally; it is not inserted into the success chain. Existing mismatch expectations are never rebound.

Across all segments the tool requires the identical frozen package/payload/anchor/index tuple and global cumulative fee cap, unique segment IDs/phase IDs/full packets/first signatures, global cumulative reserved/actual fees within u64 and the cap, and strictly later successful finalized inclusion slots. A segment boundary cannot reset fees, reuse a packet or change Clock/payload/index labels. `nextSegmentAllowedByDeclarations` is true only after the current supplied segment actually passes the existing checker; this remains a declaration check, never permission for an operation.

The Python wrapper pins the fixed Node24.19.0 executable/hash, existing checker/hash and established SDK entry/lockfile hashes, then uses one fixed Node script over bounded stdin. It invokes no shell and accepts no executable, argument, preload, endpoint, response or checker-report override. The child environment contains only the public repository locator; NODE_OPTIONS/NODE_PATH/OpenSSL/loader preload variables are not inherited. `PIV1_PUBLIC_CHECK_REPO` supports E-directory review against the actual public repository; all pinned contents must still match. A single process rechecks all reconstructed segments per successful operation, with20-second timeout,16MiB combined input,512KiB stdout and16KiB stderr bounds. Any source/process/timeout/output failure stops before appending. This rechecks signatures under the pinned source; it does not authenticate current freshness or source-specific state expectations.

The directory is0700; lock/records0600, regular/current-owner/single-link and opened with O_NOFOLLOW through directory descriptors. Record/lock descriptors use O_NONBLOCK so an unexpected FIFO cannot block before its type is rejected. Lock creation happens only in CREATE. Missing/replaced/corrupt lock or unexpected files stop. Initial records bind the stable lock device/inode/ctime, repeated and hash-protected in later records, to detect replacement on reopen. Advisory flock serializes cooperating processes across complete replay, validation, CAS and write/fsync. Every numbered canonical JSON record binds sequence, predecessor and content digest. Reordered/gapped/missing/noncanonical/truncated/altered records stop; no truncate/delete/repair/retry occurs. Stable lock identity and local hashes assume host/kernel integrity; they are not protection against a privileged actor rewriting all files and unprotected external receipts.

Successful append output appears only after exclusive record creation, complete write, file fsync and directory fsync. A write/fsync failure is COMMIT UNCERTAIN: it may leave a complete or partial record, and the tool never auto-adopts it or retries. The caller keeps its last acknowledged receipt; a complete unacknowledged record causes stale-CAS STOP and a partial record causes corruption STOP. Resolve uncertainty externally before any new action. This journal does not perform or authorize transaction submission, so a SUBMISSION_START_RECORDED event remains a supplied retrospective/intent label, not proof of a send or a one-send controller.

Limits are32 segments,1024 records,16MiB per record/combined checker input and64MiB journal bytes. A2GiB free-space reserve is checked before directory creation/writes. These are bounded host utility limits, not a capacity proof for all949 startup packets,936 full Buffer poststates or production throughput. Every successful reopen rechecks all retained segments; larger retention/archival policy is a separate design problem. Tests use deterministic public historical signatures and synthetic retrospective traces, never current lifetimes or live operations.

This chunk deliberately does not implement the semantic Clock boundary. A null pre-anchor tuple cannot become an anchored tuple across these linked segments. No real Clock is fetched/selected; no40-byte sysvar decoding,315-byte PIV1IN01 anchor-at299 proof, payload/roles semantic binding, stored-message/vote readback or Clock-before-storage controller is supplied. Those requirements remain OPEN. The collector's actual source+10SOL poststate_mismatch remains preserved, without revised expectations or a positive rerun. Controlled signing/send, authenticated segment receipts/state oracles, final resources/lifetimes/caps, independent readiness review and separate final startup authorization also remain outside this journal. M4 stays TECHNICALLY INCOMPLETE / OPEN_UNWAIVED; stopped operations/drafts and consumed scopes stay untouched.

### Durable pre-profile commitment and one-time offline Clock initializer seal

The public journal now supports a bounded PROFILE→SEAL boundary. It commits a complete existing-ALT initializer profile with anchor=null before accepting a supplied raw Clock. It then derives the exact anchored native315-byte payload, compact stored message, ordered account roles and nine unsigned initializer packets through the pinned existing planner. The parent becomes terminally sealed. This is local public encoding/persistence validation; it selects no actual Testnet anchor, signs/submits nothing and leaves every liveReady/authorized flag false.

The existing request schema/CLI/old journal records remain supported:

```sh
python3 tools/startup_public_journal.py --request tools/startup_clock_binding.example.json --journal /never-created/nullable-example
python3 tools/startup_public_journal.py --request /public/freeze-profile.json --journal /public/completed-parent
python3 tools/startup_public_journal.py --request /public/seal-clock.json --journal /public/completed-parent
python3 -m unittest discover -s tools -p test_startup_clock_binding.py
```

All requests still contain exactly schema/action/expectedTail/segment/event. The nullable example performs no directory or process operation. Two additional actions use `segment` for their dedicated payload and require event=null:

| Action | Preconditions | segment payload |
|---|---|---|
| FREEZE_INITIALIZER_PROFILE | Externally retained exact completed parent tail; all actual pinned-checker segment results successful; parent frozen anchor=null | Complete `piv1-initializer-existing-alt-plan-v1` wrapper with only `initializer.initialization.anchorUnixTimestamp:null` |
| SEAL_INITIALIZER_CLOCK | Externally retained exact PROFILE tail, no existing seal | `{address,owner,executable,contextSlot,dataBase64}` for the declared Clock account |

FREEZE invokes the actual pinned existing-ALT validator and requires its missing list to be exactly `["initializer.initialization.anchorUnixTimestamp"]`; the supplied shape must exactly match normalization. Its transactionIndex must equal the parent's unchanged generic frozen transactionIndex. It commits the exact preManifest, its canonical hash, the completed parent tail and current retained global fee ledger in one fsynced PROFILE record. Generic package/payload labels are retained separately: label equality is not source-specific profile validation, founder acceptance or evidence that old historical packets initialized this profile.

FREEZE first requires room for both its PROFILE record and exactly one following SEAL record within the existing record-count bound; it rejects without mutation when only one slot remains. This reservation does not guarantee future disk capacity or process success. PROFILE immediately prevents ordinary APPEND_EVENT and START_SEGMENT as well as another profile freeze. Only INSPECT and the single Clock seal remain. SEAL reads the durable preManifest; the caller cannot substitute a new profile, native payload, compact message, role array or success report. The postManifest is derived by changing only the anchor to the raw Clock unix_timestamp, then the pinned full planner must return OFFLINE_PLAN_ONLY/missing[]. One SEAL record binds the pre-profile parent tail, PROFILE tail, exact pre/post manifests and hashes, raw/decoded Clock, context floor, source-derived native/compact/ordered-role hashes and full unsigned initializer plan. Successful output follows record file+directory fsync under the same stable lock/CAS. INSPECT recomputes and compares the complete derivation; stored hash labels alone are insufficient.

Clock identity must be `SysvarC1ock11111111111111111111111111111111`, owner `Sysvar1111111111111111111111111111111111111`, executable=false. Data is exactly40bytes of canonical base64: slot@0/u64, epoch_start_timestamp@8/i64, epoch@16/u64, leader_schedule_epoch@24/u64, unix_timestamp@32/i64. Context is a canonical full-u64 string; raw slot must be<=context, and both must be at least the maximum of parent inclusion slots, all retained successful raw poststate contexts, planner referenceSlot and existing ALT contextSlot. This bounded operational utility supports only nonnegative i64 anchors; the native initializer supports negative signed-i64 anchors in general. This restriction changes no on-chain/economic rule. Signed epoch-start values remain exact. These are supplied public fields, not authenticated current chain data or a proof of when the Clock was captured.

Python independently checks native size315, selector PIV1IN01/version1 and signed little-endian anchor bytes299..306; full native and compact byte hashes; compact lookup-free single-instruction payload equality; program address and every indexed ordered account's address/signer/writable privileges. It also checks nine actual generated packets have zero signature slots. The planner supplies all bytes/roles from pinned source, never caller assertions. Negative or stale Clock data, substituted profiles, native/compact/role inconsistencies, malformed fields or source/process failures stop before an accepted record.

Hashes of manifests/role arrays/plans use the journal's existing encoding: recursively sorted JSON keys, ASCII JSON escaping, compact separators and one trailing newline, then SHA-256. Raw Clock/native/compact hashes cover the actual bytes. Generic parent frozen fields/caps are not rewritten. All previous reserved/actual fees, replay sets and successful-slot ordering remain verified in the original prefix. No child journal, fresh cap or historical-packet relabeling is created.

After SEAL every mutation—including old APPEND/START, reprofile and recapture—is rejected. INSPECT remains available. Concurrent sealers share the same lock and expected-tail CAS; one can commit and the stale competitor stops. Existing corruption/symlink/FIFO/missing-lock/uncertain-fsync behavior remains fail-closed. A failed fsync may leave unacknowledged data; there is no auto-adoption, truncation, recapture or retry. Protect external receipts; local hashes/host metadata do not protect against a privileged actor rewriting both journal and receipts.

Successful PROFILE/SEAL/INSPECT validation uses one existing-checker Node process plus one initializer-planner process, reported separately. The same fixed Node path/hash, minimal child environment, timeout/output limits and SDK/source-pin rules apply, with extra pins for existing-ALT/base initializer modules; the planner verifies its retained source set too. Existing old-only journal operations retain one checker process. No new dependency or arbitrary executable/argument/response override exists.

The4,360-byte portable fixture is fully synthetic public encoding data, including Clock slots above2^53, timestamp1234567890 and a signed negative epoch-start. Its initializer inputs come from the retained synthetic shared fixture with anchor changed to null. The old historical signed fund packet is only a retrospective completed parent fixture; it is never evidence of initializer execution or authorized again. Selected actual package anchor staysnull, and the earlier actual source+10SOL poststate_mismatch remains unchanged.

This terminal output deliberately has no post-anchor execution path. Authentic Clock collection, exact signed outer lifetimes, resource/funding/state-oracle review, storage/vote readback and enforcement, controlled signer/send, post-seal continuation/cumulative spending design and independent readiness review remain separate. The unsigned plan is an encoding artifact with supplied lifetime/ALT declarations, not a send-ready package. M4 stays TECHNICALLY INCOMPLETE / OPEN_UNWAIVED; stopped operations/drafts and consumed scopes stay untouched. D-049 still requires separate explicit authorization of the completed exact bounded package before new live operations.

### Read-only sealed initializer storage evidence

`tools/startup_initializer_storage.py` checks three supplied raw Squads accounts against an exact terminal Clock SEAL in an existing local public journal. It never changes that journal or creates a successor, reserves fees, signs, sends, or fetches accounts. PRE_VOTE and POST_VOTE are independent declared snapshots; a successful PRE check does not prove it happened before actual votes or stop external signers. All readiness, authorization, timelock-release, execution-eligibility and chain-authenticity fields remain false.

```sh
python3 tools/startup_initializer_storage.py --template
python3 tools/startup_initializer_storage.py --request tools/startup_initializer_storage.example.json --journal /tmp/never-created-null-storage-journal
python3 tools/startup_initializer_storage.py --request /absolute/public/storage-request.json --journal /absolute/public/existing-sealed-journal
PYTHONDONTWRITEBYTECODE=1 python3 -m unittest discover -s tools -p test_startup_initializer_storage.py -v
```

The nullable example returns `INCOMPLETE` without opening the journal or importing its helper. A complete request has exactly `schema`, `expectedTail`, `stage`, and `snapshot`. Schema is `piv1-startup-initializer-storage-v1`; stage is `PRE_VOTE` or `POST_VOTE`. Supply the caller-retained `{sequence,digest}` terminal receipt, with a canonical u64 decimal sequence and lowercase SHA-256. `snapshot` has exactly `contextSlot` and `accounts`; `accounts` contains exactly the sealed Multisig, VaultTransaction and Proposal PDA addresses. Each value has exactly `{exists:true,owner:<Squads program>,executable:false,lamports:<positive canonical u64 string>,dataBase64:<complete canonical base64>}`. Context is a canonical u64 decimal string at least the sealed Clock context and original context floor. A single common context is a declaration, not authenticated RPC collection.

The gate opens the existing stable lock without writing it, holds an exclusive flock, verifies immutable record bytes and lock identity, checks external tail equality, and invokes the pinned actual journal validator. A complete PROFILE then terminal SEAL must rederive through the pinned checker and initializer planner. Existing UNKNOWN, failed, corrupt, mismatched or rebound histories are refused. The original packet/replay/context/cumulative-fee ledger is inherited unchanged as `inheritedLedgerReference`; no request field can reset its cap. Local receipts do not authenticate the founder or resist privileged whole-directory rollback. The fixed Node binary, SDK entry/lockfile, journal, planner and relevant native parser sources are pinned; no caller executable/argv/script, inherited Node preload environment or network is used.

Canonical PDA addresses and stored bumps are derived by the fixed pinned SDK for Multisig, transaction, proposal and vault. The supported startup subset requires six strictly byte-sorted distinct nonzero guardians, every permission mask 7, autonomous config authority, threshold 4, exact create key and timelock, current index exactly the sealed selected transaction index, and stale index exactly the sealed declaration and below the selected index. The current-index equality is an operational restriction; the native general validator also accepts a later current index. Rent-collector None/Some are decoded with their actual variable Borsh length. Source-valid unused allocation bytes in Multisig and Proposal may be nonzero and are included in raw account hashes.

VaultTransaction requires the exact selected creator, index, vault, bumps, empty ephemeral-signers vector, bounded Borsh u32 vectors, unique/fully used keys, one instruction, empty lookups and exact EOF. Its complete decoded message is re-encoded into compact u8/u16 form and compared byte-for-byte to the seal; native PIV1IN01/315 bytes and all ordered role privileges must match. Comparing a stored suffix or only a payload hash is insufficient. PRE_VOTE requires Active and all three vote vectors empty. POST_VOTE requires Approved, exactly the four intended eligible byte-sorted distinct approvers, and no reject/cancel votes. Other status variants are refused before their variable payload is parsed. Stored status timestamps retain the signed i64 domain; no timestamp>=anchor rule or current-time assertion is added.

A positive result is `VERIFIED_SUPPLIED_INITIALIZER_STORAGE_ONLY` and includes terminal/seal/profile references, snapshot and complete raw account hashes, native/compact/ordered-role commitments, the ledger reference, and actual subprocess counts (one checker, one planner, one PDA helper). STOP/exit1 gives no advancement or replacement permission. There are no writes to the parent. Inputs are bounded to 64 KiB; each raw account is at most 4 KiB. The journal retains its original 32-segment/1024-record/16-MiB-checker-input/64-MiB-journal limits; this gate does not increase controller capacity.

This closes only offline semantic comparison of supplied stored Squads bytes to the seal. Actual fixed-origin collection of these accounts, chain authentication/freshness, proof that storage preceded voting, vote signatures/causal history, later fresh Clock/timelock release, external ProgramData/artifact/ALT/PIV/protocol/recipient coverage, current lifetimes/resources/funding, controlled post-seal ledger continuation, signer/send enforcement and the complete operator remain open. Approved supplied bytes are not live execution eligibility. The real selected startup anchor remains null. D-049 requires separate final-package live authorization; M4 remains TECHNICALLY INCOMPLETE / OPEN_UNWAIVED, and the prior source +10 SOL mismatch and stopped drafts remain untouched.


### Current main publication and retained preparation scope (2026-10-21)

[D-050](../../docs/PIV1_DECISIONS.md#d-050--confirmed--founder-authorized-main-publication-including-readmes) authorizes main publication of the 25 reviewed integration commits after 824342d through 5663d68, plus this reviewed README/checkpoint update. Earlier publication/identity-selection statements in this append-only tool history keep their original scope; the [current pilot state](../../docs/PIV1_PILOT_STATE.md) and [M5 readiness register](../../docs/PIV1_M5_TESTNET_READINESS.md) control the present state. The latest tools cover public reconciliation, durable journal/Clock sealing and supplied Squads storage comparison. Their tests and source-bound limitations are retained, not rerun for Git publication.

M4 remains TECHNICALLY INCOMPLETE / OPEN_UNWAIVED; M5/public package NOT READY / NO LIVE TRIGGER. D-049 still requires separate explicit completed-package authorization before live work. No new signing, keys, funds, authority transfer, deployment or Mainnet action follows from GitHub publication. Future work returns to reviewed integration publication unless separately authorized for main.

### Collected sealed initializer storage and Clock evidence

`tools/collect_startup_initializer_storage.py` connects the existing raw Squads storage gate to two fixed-origin Testnet reads and complete public receipts. It uses the exact terminal parent receipt, derives three account addresses from its seal, reads them with Clock in one finalized response, invokes the existing storage checker on the retained normalized bytes, and checks POST timing with the native signed-i64 subtraction rule. The parent remains terminal and unchanged. This is evidence collection/comparison, with no signer, sender, successor journal, new fee reservation or phase advancement.

```sh
python3 tools/collect_startup_initializer_storage.py --template
python3 tools/collect_startup_initializer_storage.py --validate tools/startup_initializer_readback.example.json --journal /tmp/never-created-null-readback-parent
python3 tools/collect_startup_initializer_storage.py --collect tools/startup_initializer_readback.example.json --journal /tmp/never-created-null-readback-parent --output /tmp/never-created-null-readback-output
PYTHONDONTWRITEBYTECODE=1 python3 -m unittest discover -s tools -p test_collect_startup_initializer_storage.py -v
```

The nullable request returns `INCOMPLETE` and creates no output or journal; it performs no Node or network operation. Complete requests have exactly `schema: "piv1-startup-initializer-readback-v1"`, `expectedTail: {sequence,digest}`, `stage: "PRE_VOTE"|"POST_VOTE"`, and `minimumContextSlot`. Sequence/minimum are canonical u64 decimal strings and the digest is lowercase SHA-256. The exact externally retained terminal receipt is required. Inputs never supply RPC endpoints, methods, account addresses, raw snapshots, success labels, replacement payloads or fee caps. `--validate FILE --journal DIR` checks the request and rederives the parent entirely offline (two pinned Node helpers); it authorizes nothing.

A later explicit `--collect FILE --journal DIR --output NEWDIR` is the only network CLI. It cannot query synthetic fixture profiles: before creating output or invoking TLS it requires the sealed actual Testnet PIV identity `GAjHRaC3XM98SoRoFy5B8P2ypTS1ndf1UPhXmjWgtPux` and multisig `9mwto7RNokYc8n8MPtzHpubwwnKcborY2r9RnfZAScVS`. This restriction is not permission to run it or proof of founder profile acceptance. The current preparation uses no actual RPC and does not select the real anchor. There is no CLI response-file, transport or origin override. Python transport injection exists only for offline tests and always reports `OFFLINE_INJECTED_TEST`, no fixed-origin TLS receipt and zero live transport invocations.

The sequence is exactly `getGenesisHash` at literal `https://api.testnet.solana.com`, requiring genesis `4uhcVJyU9pJkvQyS88uRDiswHXSCkY3zQawwpjk2NsNY`, then one `getMultipleAccounts` of sealed Multisig/VaultTransaction/Proposal plus Clock, encoding base64, commitment finalized. `minContextSlot` is max(caller minimum, seal context floor, sealed Clock context). There are at most two logical reads/two HTTP attempts, with no retry. Existing TLS rejects redirects/proxies, uses the default certificate verification and preserves partial/truncated response failures. Strict JSON rejects duplicate keys/nonfinite values while leaving unused legitimate RPC floats alone; relevant raw u64 fields reject bool/float/overflow. Four normalized accounts use the one returned context. Exact declared common context is not a consensus or freshness proof.

Before collection, the pinned parent journal INSPECT holds its existing stable lock across full prefix/CAS/rederivation; the lock is released before HTTP and before the storage checker independently reopens/rechecks the same parent and seal. Successful composition requires exact agreement on parent tail, seal hash and inherited ledger. No nested flock or parent write occurs. Positive runs use five Node helpers: pre-inspection checker/planner, then storage-gate checker/planner/PDA helper. The existing source pins, complete Borsh→compact/native/ordered-role comparison, current/stale/member/permission/threshold restrictions and stage vote rules remain unchanged. Collected absence, corruption, stale contexts or mismatched state STOP; no expectation is rebound.

Clock must be a present positive-lamport nonexecutable sysvar-owned account with canonical base64/exact40B `<u64 slot, i64 epochStartTimestamp, u64 epoch, u64 leaderScheduleEpoch, i64 unixTimestamp>`. Raw slot and shared context must both dominate the combined caller/seal floor, and raw slot cannot exceed context. PRE_VOTE has no approval timelock check. POST_VOTE requires native checked signed-i64 `Clock.unixTimestamp - Approved.timestamp`: overflow, future approval or elapsed below the sealed configured delay STOP. No new timestamp>=anchor rule is imposed. `clockEvidence.timelockArithmeticSatisfiedByCollectedClock` is a snapshot arithmetic predicate; inherited and outer `timelockReleased`, `executionEligible`, `authorized` and `liveReady` remain false. It neither proves a later execution will see the same state/time nor grants permission to sign/send.

Output must be a new outside-parent directory, never the parent or a child. Existing secure Store creates it exclusively with0700 and files0600, no symlink traversal/overwrite; files and directories fsync before acknowledgement. A40MiB evidence allowance is reserved above2GiB free space, with per-write bound/reserve checks. Inputs are<=64KiB, account data<=4KiB each, inherited raw response bound16MiB each. Request/raw response/status/headers/IDs/start+completion UTC, partial failures, normalized three-account snapshot, raw Clock, actual storage report, timing predicate and final manifest/hash receipts are retained. `httpAttemptsStarted` counts durable intents immediately before invoking transport; separate live/injected counters distinguish paths, and do not claim physical TCP requests when transport fails early. Failed preconditions before Store create no directory; failures after Store retain public partial evidence and STOP. An I/O/fsync uncertainty may prevent a final manifest/receipt; it never returns success, repairs, overwrites or retries.

The successful status is `VERIFIED_COLLECTED_STORAGE_AND_CLOCK_EVIDENCE_ONLY`. It is not a controlled successor or complete P0..P7 operator. TLS-origin receipts are not chain-authenticated state, causal inspection-before-voting, historical storage/vote proof or actual vote signatures. Independent PRE/POST collections are not linked by a new journal. Protected caller receipts/host integrity, current resources/lifetimes/funding, full external state oracle and controlled signer/send/ledger continuation remain open. Existing raw-storage checker flags are preserved unchanged. Real startup anchor remains null; D-049 final-package live authorization, M4 TECHNICALLY INCOMPLETE / OPEN_UNWAIVED, stopped paths and retained source +10SOL mismatch remain untouched. D-050 main publication does not authorize a new main update or live deployment.

## Prepared release operator: exact public packets and recovery

`tools/startup_release_operator.py` closes the reusable message/intent/confirmation workflow, with 12 focused offline tests and a nullable CLI check. It has no wallet reader or signer. No real signing, RPC submission or deployment was performed. The exact P0..P7 startup manifest is still blocked on runtime-derived state rules and mandatory account coverage; this is not a ready deployment command.

```sh
python3 tools/startup_release_operator.py request.json /absolute/path/to/base-journal
```

Requests contain `schema`, `action`, `expectedTail` and `data`, with schema `piv1-startup-release-operator-v1`. Actions are CREATE, INSPECT, PREPARE, HANDOFF, IMPORT, BIND_SEAL, SEND and RECOVER. Null action is INCOMPLETE and creates no journal. The API returns before helper loading on null; CLI parsing loads pinned local helpers. CREATE freezes the reviewed manifest; PREPARE admits at most64 exact unsigned messages, allowing only a pre-intent outer-blockhash refresh. IMPORT verifies existing supplied signatures. HANDOFF invokes no signer.

The manifest requires immutable `historicalLedger` plus `proposedFeeBudget`, namespace `piv1-testnet-startup-release-<full reviewed package SHA-256>`, status `PROPOSAL_ONLY_NOT_AUTHORIZED`, fee cap `4815000` and exact historical reference. Old scope counters are never reset or counted again. Future counters begin0 and require separate explicit founder authorization binding the completed manifest, namespace and source before activation. A local launcher receipt is an external trust boundary, not cryptographic founder verification. Current preparations have no receipt; do not use `--allow-live-send`.

A future authorized SEND uses fixed official Testnet TLS, latest finalized blockhash/expiry binding, current fee/prestate checks and RPC preflight (`skipPreflight=false`), then persists INTENT before one send with maxRetries0. UNKNOWN blocks all dependent preparation/handoff/import/send; only the original signature may be recovered. An expired original receipt permits only matching read-only RECOVER, never new spending. Every collection has a unique retained ATTEMPT; no failed evidence is overwritten or adopted. After a thrown append, read and verify the complete chain/tail in the existing Journal API before a CAS INSPECT; a stale acknowledged tail is not automatically advanced, and partial/uncertain records require STOP, not repair.

The same base may append only PROFILE then SEAL at the Clock boundary, keeping all original records intact and becoming terminal. BIND_SEAL verifies original lineage, ledger and raw Clock floor; its nine message semantics come from the pinned sealed planner. PRE_VOTE precedes first approval, POST_VOTE/timelock precedes execute. The positive synthetic test binds the seal, prepares its first unsigned packet and reopens; it proves no real votes or initialized public state.

Storage stops at8GiB retained evidence,128MiB journal or8192 records while retaining2GiB free and a256MiB action envelope.949 descriptors and64-message batches are limits, not a whole-upload performance claim. Current exact-hash state maps cover only explicitly declared subsets; they cannot replace the missing startup-specific runtime state oracle. No signing implementation, actual control proof, complete resource/phase manifest or final founder approval is provided by this interface. See [release operator evidence](../../docs/PIV1_TESTNET_RELEASE_OPERATOR.json).


## Startup runtime state rules consumed by the release operator

`tools/startup_state_rules.py` supports four exact typed expectation entries:

```json
{"startupRule":"PROGRAM_DATA","lamports":"4545203000"}
```

The other kinds are `INITIALIZED_TARGET`, `PROPOSAL` and `HEARTBEAT_REWARD_0`.
Each entry has exactly those two fields, and a typed manifest must pin the exact
adapter hash in `sourcePins.stateRules`. Existing six-field exact account maps
remain compatible. This is a typed subset interface, not the complete startup
manifest or a live command recipe.

SEND verifies complete source-derived account bytes, checks fresh per-size rent
quotes against the frozen startup profile, and persists raw Clock/prior facts
before its one-send intent. RECOVER validates exact typed poststate before the
existing signed-wire checker and persists/replays the allowed new facts. Clock
windows enforce pre.slot <= pre.context <= inclusion.slot <= post.slot <=
post.context and nondecreasing timestamps. Intermediate approvals keep the old
Proposal timestamp; only creation, threshold and execution may learn a timestamp.
Heartbeat requires both Clock endpoints in the same checked KIF period; otherwise
it stops. ProgramData retains the first successful bootstrap inclusion slot and
full selected ELF/final authority. BIND_SEAL uses the pinned SDK once to derive
canonical target/authority/Proposal bumps and binds the immutable selected plan.

Root passes eight new focused groups plus twelve relevant operator regressions,
and independently compares the retained local startup bytes without rerunning
Bank. The public signed-transfer/synthetic-ELF integration fixture proves rule
consumption only. Run these two host test modules with
`PYTHONPATH=tools python3 -m unittest test_startup_release_operator test_startup_state_rules`.
The fixture is portable public data; no wallet or live transport is needed.

See [current evidence](../../docs/PIV1_TESTNET_RELEASE_OPERATOR.json). Mandatory
P0..P7 coverage, actual signer/control integration, complete phase-time resources,
upload scheduling and final readiness remain open. M4 OPEN_UNWAIVED; public
NOT READY / NO LIVE TRIGGER. No authorization receipt or live signing/send is
created by these tests. Earlier CLI usage and historical evidence remain retained.


## Canonical complete startup manifest

`tools/startup_manifest.py` constructs the selected 949-descriptor manifest from
one public selected ELF and a compact pinned `startup_manifest.oracle.json`.
The constructor API is `adapter.assemble(operator.operator_api(), journal,
elf_bytes, package_sha256=..., parent_tail=..., historical_ledger=...)` after
`adapter = operator.startup_manifest()`. This is an offline construction API.
The package digest/actual parent ledger must be reviewed; no live receipt is
created by construction. The nine initializer messages remain deferred to the
existing immutable Clock seal. Only the recent outer blockhash may change in
PREPARE; all other message bytes, resolved roles and signers stay source-bound.

Canonical descriptors require empty caller expectation maps; the controller
constructs mandatory PRE/POST maps itself. Each Write covers ten relevant
accounts and every other phase covers all 57, including all votes. Full mode
checks exact actor balances, complete Buffer progression, authorities, external
images/features, source-derived Squads storage, ALT and initialized custody.
The existing typed rules handle deployment slot, anchor, Proposal timestamps and
heartbeat activity. Clock regressions, changed versions/rents and ambiguous heartbeat periods stop. Legacy manifests remain
usable with injected offline transport and cannot route to actual live SEND or
RECOVER.

The selected 1,106,125-byte stake-pool image is fully checked through a dedicated
address/owner/size/hash bound. The old collector/checker retain their 1-MiB
limits. Full recovery verifies the original signature/status/wire/fee using its
bounded subset, then obtains one complete 57-account finalized snapshot at or
after that proof context/inclusion/floor. Both exclusive raw receipts and every
complete account digest are bound and rechecked during replay. Failed collection
leaves the original INTENT unresolved and dependent actions blocked.

Validation comprises 31 unique groups across retained/corrected runs, with both
failed runs preserved. New transport tests explicitly use synthetic image,
account and first-message seams plus an existing public signature. They are not
actual startup execution. The independent real selected-image audit compares
all 949 messages/roles/cashflows; no Bank or old application suite was rerun.
Only the manifest CREATE/reopen/PREPARE envelope was measured; whole-upload
performance/capacity, actual signer integration and final readiness remain open.
See [current continuation evidence](../../docs/PIV1_TESTNET_RELEASE_OPERATOR.json).
M4 remains OPEN_UNWAIVED; public NOT READY / NO LIVE TRIGGER. No signing, keys,
funds, deployment, authority transfer or Mainnet operation is authorized here.


### Durable public signer request and response boundary (2026-10-23)

`HANDOFF` now appends a source/authorization/manifest/parent/seal/PREPARE-bound request and returns it with the exact prepared packets. `INSPECT` returns the same `pendingSigningRequest` and `signingPackets` after lost output; neither action invokes a signer. Required signer order and the exact unsigned message are frozen. The future launcher-bound receipt is mandatory; no completed-package receipt exists today.

`IMPORT` accepts the complete response envelope below with the unchanged launcher receipt. Placeholders refer to the exact pending request and reviewed historical or future-authorized signed bytes. Full canonical mode rejects the former list-only format.

```json
{
  "schema": "piv1-startup-signing-response-v1",
  "requestId": "<exact pending requestId>",
  "packets": [{"id": "<exact requested phase>", "wireBase64": "<complete signed transaction>"}]
}
```

All requested packets must appear once in exact order. Missing/extra/reordered entries, changed messages, missing/invalid signatures, expired/changed authorization, stale PREPARE identities or unresolved intents STOP without consuming a valid subset. A new PREPARE invalidates the pending request even with identical bytes. IMPORT verifies every required signature with the unchanged pinned checker; SEND verifies again and retains the same authorization binding. Replay rederives request/response/wire/hash relationships without rechecking all past cryptographic signatures on each append.

Root passes 21/21 focused groups first execution (8 new, 12 operator, 1 adapted full-manifest method); separate source/actual review passes. Historical signatures and explicit synthetic host seams are not current control or startup proof. No wallet reader, private signer backend or signing invocation is implemented here. Actual backend/control, phase-time resources/funding/lifetimes, practical upload/evidence scheduling and final independent readiness remain open. M4 OPEN_UNWAIVED; M5 NOT READY/NO LIVE TRIGGER; D-049 exact completed-package live authorization remains mandatory. No new private/key/signing/simulation/RPC/send/funds/authority/deployment/Mainnet action.


### Founder-authorized main publication (2026-10-23)

[D-053](../../docs/PIV1_DECISIONS.md#d-053--confirmed--founder-authorized-main-publication-of-reviewed-m5-continuation) covers the reviewed seven-commit continuation through `3b79bb3` plus this documentation checkpoint. Current CLI/operator sources and historical tests/reviews are reused unchanged; this publication reruns no suites, builds or runtime and refreshes no chain observations. The durable public signing boundary is implemented; the actual signer backend/control, phase-time resources/funding/lifetimes and practical upload/evidence scheduling remain open. M4 OPEN_UNWAIVED; M5/public NOT READY/NO LIVE TRIGGER. No new signing, spending, authority transfer or deployment follows. Later development returns to reviewed integration publication; further main updates need separate authority.


### Future gated signer backend — implementation, not live authorization (2026-10-23)

`SIGN` prepares and durably records one exact source/manifest/PREPARE-bound request plus backend attempt in a single record, then invokes the fixed pinned Node backend. It requires a full canonical manifest, strict canonical launcher receipt v2, backend/profile hashes, and `--allow-live-sign`. That flag is independent of `--allow-live-send`; no v2 receipt or real profile is selected in this preparation. Do not run a live invocation before D-049 final exact authorization.

```json
{
  "schema": "piv1-startup-release-operator-v1",
  "action": "SIGN",
  "expectedTail": "<exact operator tail>",
  "data": {"ids": ["<ordered prepared phase>"], "signerProfilePath": "<approved private absolute path outside the repository>"}
}
```

Keep filled SIGN requests and profile files outside Git and public evidence. The private profile schema is `piv1-disposable-testnet-signer-profile-v1`, with exact `cluster: testnet`, official `genesisHash`, `disposableTestnetOnly: true`, and unique `signers: [{address, secretKeyPath}]`. Its exact bytes are bound by `signerProfileSha256`; launcher v2 also adds `signerBackendSha256` to existing receipt fields. The accepted immutable full profile contains only G0, G1, G2, G3, H2 and the dedicated PIV Program signer from the pinned oracle. Only keys required by the current packet union are opened. Shared source 4WK, guardians 4/5 and governance PDAs are excluded. Paths are syntax-checked; the needed profile/key files must pass anchored descriptor/no-symlink, owner, exact 0600, one-link, size and race checks. No private path/content enters the journal, CLI output or child diagnostics. The controlled Node child has core dumps disabled before exec. JavaScript/native buffers are not a privileged-host isolation or guaranteed memory-erasure boundary.

Every exact zero-signature input is independently decoded before private access. The future native branch verifies key/public-address consistency and signs the exact message bytes, then the unchanged checker verifies every output signature. The backend rechecks profile/source/expiry before returning; Python rechecks source/auth/request and signatures afterward without rereading private profile/key files. Current tests use explicit reader/key/signature callbacks and historical signatures; native reader/key-match/sign paths remain unexecuted.

Ambiguous/failed backend output leaves `SIGNER_ATTEMPT_UNRESOLVED_ORIGINAL_RESPONSE_ONLY`. SIGN/PREPARE/HANDOFF/BIND_SEAL/SEND stop; only the complete exact original response with unchanged, unexpired original authorization can be imported. No automatic renewal, retry, reprepare, replacement, partial import or journal repair exists. If that response is unavailable or authorization expires, a permanent hold requiring separate technical review remains possible. INSPECT labels a possible real invocation as uncertain, never definitely uninvoked or key control proven. Injected callbacks permanently mark the operator offline; they cannot be promoted to fixed-origin send.

Root passes 28/28 Python groups (7 new + 21 relevant) and 4/4 Node groups in separate first executions, plus two CLI guards; source/actual review passes. Native signing/key derivation/generation/import/private-open/network counts are zero; positive Node paths return 2 historical signatures through fake callbacks. Static planned no-retry journal use is 7598/8192 records, 594 spare, without a separate manual HANDOFF; this does not establish whole-upload capacity, timing, retry space, evidence budget or funding sufficiency. Actual control and phase-time scheduling/readiness remain open. M4 OPEN_UNWAIVED; M5/public NOT READY/NO LIVE TRIGGER. Main remains b7e30b2; this continuation is integration only. No new key/private read/signing/simulation/RPC/send/funds/authority/deployment/Mainnet action.


### Captured original lifetimes — offline-validated preparation (2026-10-23)

The existing `PREPARE` action additionally accepts `{"captureLifetime":true,"packets":[{"id":"<ordered phase>","wireBase64":"<exact unsigned scaffold>"}]}`. This canonical full-manifest path requires the exact launcher receipt and `--allow-live-send` for actual fixed-origin public reads; tests inject transport. The flag does not itself authorize any live action. No current receipt, real quote or live invocation is selected. Up to 23 packets fit the unchanged six-query/five-call budget. Legacy list input is unchanged and retains strict latest-hash equality.

The controller initially decodes exact zero-signature semantics, captures Testnet/genesis/latest, replaces only the 32 blockhash bytes, obtains each exact message fee, current height and original-hash validity at the captured context floor, then binds the normal five-field prepared packets plus quote into one existing PREPARE record. This is an initial decode plus byte-offset/semantic proof, not an independent post-rebind decode. Raw requests/responses and hashes bind source, authorization, parent/predecessor, manifest, packet order and transport. Replay verifies all artifacts and advances context floors; original expiry is never supplied by the caller on this path. These TLS-origin/local receipts are not consensus proofs or cryptographic founder authorization.

Only a verified captured quote allows SEND to accept a rotated latest hash. Fresh original validity, captured last-valid height, the original remaining-height margin, exact fees/rents, mandatory accounts/authorities and all other guards still apply. A newer latest expiry cannot extend the original. Quote authorization must remain identical through handoff/sign/send; historical expiry permits read-only replay only. Captured injected history cannot invoke the native signer or fixed-origin SEND.

Public-read failures preserve exclusive partial evidence without accepting PREPARE; a later attempt uses a new bounded directory, never adoption/overwrite. At most 64 quote attempts share one unchanged journal sequence. Failed and successful evidence count toward the existing 8-GiB ceiling, action envelope and 2-GiB reserve. No additional per-packet record is added; static no-retry schedule remains 7598/8192. This is not practical whole-upload timing, complete serialized evidence sizing or retry-capacity proof.

Root passes 34/34 focused Python groups first execution (six new and 28 regressions), plus nullable CLI, with zero Python network/native-backend attempts and separate source/actual review. The four unchanged Node groups and old runtime/selected-image/public-state evidence are retained, not rerun/refreshed. Full-host account/ELF/message seams and historical signatures remain synthetic. Native private-reader/key-match/signing/control and original-response HOLD limitations remain open. M4 OPEN_UNWAIVED; M5/public NOT READY/NO LIVE TRIGGER; D-049 final authorization required. No actual quote/RPC/private keys/signing/simulation/send/funds/authority/deployment/Mainnet action. Integration only; main b7e30b2 unchanged.


**Current technical checkpoint — measured late-upload replay and compilation cost (2026-10-23 Europe/Warsaw).**
A physical offline history covers Buffer creation plus 936 Writes (937 completed phases; 12 packets remain). Its measured baseline contains 7,497 records, 4,520,729,571 evidence bytes and 24,576,627 journal bytes; physical allocation is 4,696,264,704 bytes. These are exact synthetic-prefix measurements, not full-949/retry/future-RPC bounds. Original v1 missing-ComputeBudget fixture STOP and v2 real 120-second PREPARE timeout remain preserved, including the PREPARE committed before that timeout. The fixture now uses the existing pinned native-account oracle; no product oracle changes.
The product fix caches only immutable compiled state-rule code. Every call still rereads and hashes rules source, rejects symlinks, creates fresh module globals and verifies all eleven primary sources. No state, account, authorization, Clock or raw-evidence result is cached. Root passes **35/35 focused Python groups first execution** (one new cache-boundary group plus 34 relevant regressions; suite 107.583 s / process 107.925 s) and the nullable CLI. Separate source/actual reviews pass; network/native-backend guards record zero attempts.
At this prefix, replay improves 50.037→16.447 s and append 45.048→15.366 s. Actual offline captured PREPARE completes in **52.815 s**, including **30.398 s after the injected blockhash response**, versus the preserved 120-second baseline timeout. Each version has one observation in a fresh Python process, not an OS-cold claim; the loader initializes compiled rules before the stage timer, and whole-process times are retained. Buffer/image caches start empty. Synthetic bulk construction bypasses append/crypto/per-file production fsync; measured replay/append/evidence scan/operate are unpatched. Public non-signature markers, synthetic Clock/status/finality and minimal RPC metadata prove no native signatures, bootstrap, consensus or real lifetime margin. All retained fixture bytes were independently inventoried and root-verified/fsynced after measurement.
**Next blocker/completion condition:** demonstrate the complete preparation→SIGN→SEND lifetime/resource/funding schedule without reducing margins or security checks, account for the remaining 12 packets, ordinary RPC metadata/retries and evidence capacity, then obtain final independent readiness. The measured 30.398-second postquote work remains a concrete scheduling gap. Native reader/key-control and lost-original-response/expired-authorization permanent HOLD remain OPEN. Unchanged four Node groups, selected 949-message audit, build/runtime, October 21 public observations and D-051 reconciliation are reused without refresh. **M4 TECHNICALLY INCOMPLETE / OPEN_UNWAIVED; M5/public NOT READY / NO LIVE TRIGGER.** No keys/private reads, real signing, simulation/RPC/send, funds, authority, deployment or Mainnet operation. D-049 exact completed-package live authorization remains mandatory. Integration only; main b7e30b2 / protected Task 2.3 3677fee unchanged; preserve 2 GiB.


**Current technical checkpoint — original lifetime and authorization checked after durable intent (2026-10-23 Europe/Warsaw).**
SEND now repeats the original blockhash validity/remaining-height checks after the complete INTENT replay/fsync, and checks the exact current authorization immediately before the single send call. The post-intent context floor dominates the accepted prestate; height cannot regress from either prepared or preflight height. Captured original expiry/margin remain fixed, while legacy packets retain exact latest-hash/expiry equality. Existing genesis/version, fees, every account/authority predicate, rents and all source/raw-evidence checks remain. Packing at most 20 preflight methods into four batches leaves one three-method post-intent check and one send: the existing six-query/five-method bounds stay unchanged, including all fourteen rent sizes.
Root passes **37/37 focused Python groups first execution** (two new plus 35 relevant regressions; process 139.470 s), the nullable CLI and separate source/actual review. New cases consume the actual durable append before advancing synthetic height, invalidating the original hash or expiring authorization. Each refuses send, retains one unresolved original INTENT/reservation, blocks resend/reprepare and permits inspection. The positive full-host case checks all fourteen rents and exact batches 5/5/5/5/3/1. Historical signatures and explicit synthetic ELF/account/message/transport seams prove offline guard consumption, not native signing or deployment. A late malformed/expired read can now leave an INTENT on hold; it must never be silently repaired or renewed.
One prior-source read-only profile reuses the unchanged 7,499-record synthetic prefix: 20.717 s profiled replay / 22.425 s process, with 6,559 fresh rule loads and 72,149 primary-source checks. Rule loading is 9.413 s cumulative, including 7.142 s source verification; these overlap and must not be added. Profiling overhead makes this attribution only, not a new latency baseline. The exact tail and journal metadata remain unchanged, with zero network/child/mutation attempts. No verification/result cache was added; no new speedup is claimed and old source-bound receipts are not rebound to this correction.
**Next blocker/completion condition:** demonstrate the complete preparation→SIGN→SEND schedule within the original lifetime and unchanged margins, including remaining twelve packets, exact phase-time resources/funding and evidence capacity/retries, then final independent readiness. Native reader/key control and lost-response/expired-authorization permanent HOLD remain OPEN. Unchanged Node/backend, selected 949-message audit, build/runtime, October 21 observations and D-051 are reused without refresh. **M4 TECHNICALLY INCOMPLETE / OPEN_UNWAIVED; M5/public NOT READY / NO LIVE TRIGGER.** No real keys/private reads/signing, simulation/RPC/send, funds, authority, deployment or Mainnet action. D-049 exact final live authorization remains mandatory. Integration only; main b7e30b2/protected Task 2.3 3677fee unchanged; preserve 2 GiB.


**Current technical checkpoint — unused replay work reduced; controller lifetime gap measured (2026-10-23 Europe/Warsaw).**
Two small changes remove unused work: exact_typed loads fresh rules only when it encounters a typed descriptor, while literal coverage still performs its independent fresh Clock/source check; replay uses intent/terminal cardinality because existing insertion guards enforce terminal⊆intents with no removals. No new validation/result cache, replay removal, margin/cap relaxation or authority/economic change. The manifest source pin is updated; message/role/account derivations and program sources retain their behavior.
Validation is **42 unique passing groups across two runs, not one clean 42-group run**: the initial suite passed 41/42 in 133.762 s (134.190 s process). One new assertion expected deploymentMissing when a two-typed-account case first reached proposalMissing. The test now isolates each missing fact; that one affected method passed in 0.387 s (0.566 s process), with product bytes unchanged. Both executed test versions/failure remain. All 40 relevant regressions plus the two new groups are covered; unchanged four Node groups/build/runtime/949-message audit are reused. Separate source/actual reviews and nullable CLI pass, with zero Python network/native-backend attempts.
A single fresh source-bound fixture contains Buffer create+936 Writes: 7,497 records, 4,521,077,406 evidence bytes and 24,556,935 journal bytes including lock. Read-only replay measures **12.551 s** (15.004 s process); this is not an identical-byte before/after comparison with older fixtures. The canonical bootstrap controller path then measures PREPARE 32.654 s, fake SIGN 65.156 s and SEND 56.159 s; **153.969 s total, 118.213 s from original quote to fake dispatch, with eleven full replays on that critical path**. The three actions share module caches in one fresh process; this is one observation, not OS-cold or a per-CLI-process bound. An additional denied live promotion is timed separately and leaves no append.
**Measurement seams:** six ordered signer slots contain public NON-signature markers; both the signer callback and check_signed crypto/trace gate are explicit fakes. Public I/O/heights/Clock/finality are synthetic. Actual canonical messages/roles, all 57 prestate accounts, 14 rents, source/raw checks, durable journals and final lifetime/auth guards are consumed. Native signer and real crypto/trace-gate costs, actual Clock/block-height progression and inclusion are unmeasured; no wall-time-to-height conversion, real headroom, native bootstrap or full-949 claim follows. Bulk history construction bypasses append/crypto/per-file production fsync; measured replay bodies run unchanged through a timing wrapper. All retained file bytes and final 7,503-record chain are root-verified/fsynced after timing. Bootstrap is only marked SUBMITTED in the fake transport, not recovered; twelve canonical packets remain unreconciled. Minimal RPC metadata/retries/full evidence bounds remain explicit limitations.
**Next blocker/completion condition:** reduce the eleven postquote/pre-dispatch full replays through a separately proved change preserving fresh dependency/source/raw/account/authority/auth/lifetime and at-most-once/HOLD protections, then demonstrate the complete supported schedule, remaining twelve packets and exact resources/funding/evidence sufficiency before final independent readiness. Native key control, actual signer/checker costs and permanent lost-response/expired-authorization HOLD remain OPEN. October 21 observations/prices and D-051 remain historical, unchanged. **M4 TECHNICALLY INCOMPLETE / OPEN_UNWAIVED; M5/public NOT READY / NO LIVE TRIGGER.** No real keys/private reads/signing, simulation/RPC/send, funds, authority, deployment or Mainnet action. D-049 exact final live authorization remains mandatory. Integration only; main b7e30b2/protected Task 2.3 3677fee unchanged; preserve 2 GiB.


**Current technical checkpoint — post-signer replay merged into mandatory fresh import validation (2026-10-23 Europe/Warsaw).**
SIGN no longer performs a standalone post-callback whole-prefix replay. Its mandatory IMPORT precommit replay still freshly rederives all source/raw/request dependencies and compares the complete retained signer attempt to the original local attempt before writing. Current-request consumption and unique attempt insertion preserve the old equality check. Stable lock, callback-adjacent authorization/backend checks, public signature verification, pre-signer post-fsync replay, final report replay and all SEND/intent/lifetime/HOLD controls remain. Invalid historical raw can now incur bounded public signature-check work before refusal; it cannot commit IMPORT or invoke keys/transport through that checker. No validation cache, reused pre-fsync result or margin reduction is introduced.
Root passes **39/39 targeted Python groups in one execution**, including three new methods. Instrumentation consumes five full SIGN replays instead of six. Tests reject a replay-valid but changed attempt mode and historical quote bytes changed inside the fake callback before IMPORT; the durable original SIGN_ATTEMPT remains unresolved and blocks retry/prepare/send. Separate source/actual reviews and nullable CLI pass; Python connection/DNS and native-backend guards record zero attempts. Historical public signatures and existing explicit synthetic full-host ELF/account/message/transport seams remain. Unchanged four Node groups, rule-cache/lazy-loading tests, 949-message audit, builds and runtime evidence are reused.
**Scope of progress:** the unchanged surrounding path now structurally requires ten instead of eleven postquote/pre-dispatch full replays. This is not a new late-upload wall-time result: the prior 118.213-second/eleven-pass measurement and physical fixture retain their exact older source scope, untouched. No new 937-prefix fixture, native signature/control cost, real Clock/height progression or practical lifetime headroom is claimed.
**Next blocker/completion condition:** further reduce or schedule the ten remaining passes with equivalent fresh source/raw/account/authority/auth/lifetime validation across durability/callback boundaries, then demonstrate the supported complete schedule, twelve remaining packets and exact resource/funding/evidence sufficiency. **M4 TECHNICALLY INCOMPLETE / OPEN_UNWAIVED; M5/public NOT READY / NO LIVE TRIGGER.** D-049 exact final live authorization remains mandatory. No real keys/private reads/signing, simulation/RPC/send, funds, authority, deployment or Mainnet operation. October 21 public observations and D-051 remain historical. Integration only; main b7e30b2/protected3677fee unchanged; preserve 2 GiB.


**Current technical checkpoint — explicit durable-only PREPARE/SIGN receipts (2026-10-23 Europe/Warsaw).**
The opt-in CLI `--output-mode COMMIT_RECEIPT` supports PREPARE/SIGN only; REPORT remains default. Receipt mode skips only the final informational reporting replay and acknowledges the exact fsynced final record after stable-lock/secure-byte readback. Its dedicated schema includes `postCommitReplayPerformed:false`; it carries no derived state, readiness, authority or signer-control claim and cannot replace any existing strict request/authorization/response. Post-write source/raw changes can coexist with this durable-only receipt; the next action independently replays and refuses before new signing/send effects. Default REPORT still rejects those changes in its final replay. All entry, precommit, pre-signer post-fsync, callback/IMPORT, SEND/intent/auth/lifetime/HOLD checks remain. No validation-result cache, extra journal record or fee scope is added.
Root passes **36/36 focused Python groups first execution**, including five new groups. The same supported small-host captured PREPARE→injected SIGN→SEND path actually consumes ten postquote/pre-dispatch full replays with REPORT and eight with receipts (1+4+3). It uses historical real signatures with existing explicit synthetic full-host ELF/account/message and public-I/O seams, not the canonical six-signer late bootstrap. Tests retain one-send/UNKNOWN, post-fsync next-action refusal, unchanged default reports, uncertain commit/record-change refusal and callback-tamper original-attempt HOLD. Three CLI operations and separate source/actual reviews pass; Python network/native-backend counters remain zero. Unchanged eight handoff/four Node groups, pure cache/lazy tests, 949-message audit, builds/runtime and October 21 observations are reused.
A read-only profile opens the unchanged 7,503-record prior-source fixture through five exact copied retained sources; current replay AST matches. The known tail and journal namespace/size/mtime/ctime remain unchanged. **4,688 rule-module/provenance loads dominate attribution** (6.691 s cumulative including overlapping 5.055 s source verification in a 15.924 s profiled replay). Profiling overhead is included; this is neither a new product-flow latency baseline nor proof of lifetime headroom. No old receipt/source binding is rewritten and no large fixture is rebuilt/copied.
**Next blocker/completion condition:** implement and separately prove a replay-local rules-code lifetime with independent fresh loader/all-source checks at entry and successful return, unchanged per-record Clock/raw/account rules and no reuse across replay/fsync/callback; then measure the remaining eight-pass complete schedule and close twelve remaining packets, exact resources/funding/evidence capacity and native-control/final-readiness gaps. That optimization is DESIGN ONLY here. **M4 TECHNICALLY INCOMPLETE / OPEN_UNWAIVED; M5/public NOT READY / NO LIVE TRIGGER.** D-049 exact final live authorization remains mandatory. No real keys/private reads/new signatures, simulation/Solana RPC/send, funds, authority, deployment or Mainnet operation. D-051 and old observations remain historical. Integration only; main b7e30b2/protected3677fee unchanged; preserve 2 GiB.


**Current technical checkpoint — private rules-code lifetime within each replay (2026-10-23 Europe/Warsaw).**
Replay lazily creates one private rules module/API, shares only that code and its trusted globals inside the invocation, and independently invokes the captured real loader before successful return. Both loads hash the rules file and verify all eleven primary source pins using fresh globals. Every per-record Clock/window/raw/account/state predicate still executes; each active-phase manifest source pin remains. No module, state, account, authorization or validation result is cached across replay/fsync/callback boundaries. Paths with no rules remain lazy; failures return no accepted state. Persistent source changes refuse entry/use or return, including spoofed reused provenance dictionaries. This is sampled provenance, not an atomic guarantee against privileged transient changes restored between checks.
Root passes **41/41 focused Python groups first execution**, including five new groups and the prior receipt/controller regressions. Tests cover two independent loads, fresh globals after success/failure, real temporary copied rule/primary tampering, unchanged Clock/coverage/raw refusals, laziness and fsync/public-callback isolation. Separate source/actual reviews pass; Python network/native-backend guard attempts remain zero. The prior three CLI cases, eight handoff/four Node groups, direct-loader/cache/lazy tests, 949-message audit and builds/runtime are reused; main/operate/Journal.append and the receipt contract are unchanged.
One paired **ALGORITHM-ONLY** experiment applies the candidate replay function to the exact old copied namespace over the unchanged 7,503-record history. Old source/auth/quote bytes and checks stay old; only the reviewed replay AST is substituted. Complete 23,617,726-byte canonical states match SHA256 `0c6ae1858cf3f079503fad593a404053f45f1429ecff175d93ca033c2c2088f9`; inputs/tail/journal metadata remain identical. Fresh loader calls fall **4,688→2**, with observed isolated replay **11.159→6.184 s** (process13.548/8.485 s). The pair uses separate fresh processes/empty image+Buffer caches, guards and one observation each; it is not OS-cold/statistical or a new source-bound operator/action/full-flow/lifetime/native-control proof. No old history is adopted or rebound; no large fixture is copied or rebuilt.
The explicit PREPARE/SIGN receipt mode still removes two reporting passes; its supported small-host regression again checks eight versus ten postquote/pre-dispatch passes using historical signatures and declared full-host/public-I/O seams. **Next blocker/completion condition:** demonstrate the exact current-source canonical eight-pass preparation/sign/send schedule under original lifetime/margins and actual or explicitly bounded signer/checker costs, then close twelve remaining packets, phase-time resources/funding/evidence capacity/retries, native control and final independent readiness. Do not extrapolate eight times the isolated algorithm timing into headroom. **M4 TECHNICALLY INCOMPLETE / OPEN_UNWAIVED; M5/public NOT READY / NO LIVE TRIGGER.** D-049 exact final live authorization remains mandatory. No private keys/new signatures, simulation/Solana RPC/send, funds, authority, deployment or Mainnet operation; October 21 observations and D-051 remain historical. Integration only; main b7e30b2/protected3677fee unchanged; preserve 2 GiB.


**Current technical checkpoint — exact current-source late-bootstrap controller measured offline (2026-10-23 Europe/Warsaw).**
A fresh source-bound history completes Buffer creation plus 936 Writes (7,497 records). The canonical next bootstrap consumes all six ordered signer slots, 57 prestate accounts and 14 rents through the unchanged current controller. PREPARE/SIGN use durable-only receipts; SEND retains its report. Actual postquote/pre-dispatch replay counts are **eight (1+4+3)**. One measured flow takes **66.241 s**: PREPARE15.797/SIGN22.050/SEND28.394; original quote to fake dispatch is **44.445 s**. Whole process74.004 s includes the separately denied live promotion6.316 s and initialization. Prior118.213 s/eleven-pass evidence keeps its older source/history; these are single observations of the same canonical workload, not identical-byte, OS-cold or statistical benchmarks.
**Seams and limits:** both the signer and crypto/trace gate are explicit fakes; public transport/Clock/heights/finality and bulk construction are synthetic. Actual message/role/source/raw/account/authority/fee/rent/authorization/lifetime and durable controls execute. One process shares caches across the three actions. Native signer/checker costs, actual height progression, real lifetime headroom and full949 readiness remain unproved. Bootstrap is only fake-SUBMITTED, not recovered; twelve packets remain. No wall-time-to-height inference or margin reduction is made.
Root verifies/fsyncs all **45,039 fixture files**, rechecks the complete 7,503-record chain, and preserves the old fixture metadata/tail. Source/actual reviews pass. Python network/native/forbidden-child attempts are zero; one pinned public Node decoder runs. Product/program/dependencies/artifact remain unchanged; the reviewed **41/41 groups and prior CLI/Node/runtime evidence are reused**, not rerun. Capacity is recovered through two exact verified reversible public Cargo-cache archives (1,604 files, two real restoration demonstrations); pinned files, excluded paths and executable hashes stay intact. Actual prebuild9,117,286,400 bytes passes the unchanged8,589,934,592 guard; post-audit4,387,622,912 bytes exceeds the2-GiB floor. The first read-only inventory STOP on root-owned/hardlinked candidates is retained; those candidates are excluded, never changed. Archive manifests/journals and durable public-source backups provide the recovery path.
**Next blocker/completion condition:** establish a supported original-lifetime schedule including actual or explicitly bounded native signer/checker and public-I/O costs, then recover bootstrap and reconcile the twelve remaining canonical packets with exact phase resources/funding/evidence/retry capacity and final independent readiness. This observation closes the missing current-controller timing evidence only. Native key control and lost-response/expired-authorization HOLD remain open. **M4 TECHNICALLY INCOMPLETE / OPEN_UNWAIVED; M5/public NOT READY / NO LIVE TRIGGER.** D-049 exact final live authorization remains mandatory. No keys/private reads/new signatures, simulation/Solana RPC/send, funds, authority, deployment or Mainnet operation. October21 observations/D-051 stay historical; integration only, main b7e30b2/protected3677fee unchanged.


**Current technical checkpoint — production RPC response deadline and durable uncertainty (2026-10-23 Europe/Warsaw).**
The production Rpc transport for PREPARE/SEND now enforces one **30-second monotonic response-acceptance deadline**, including its isolated fixed child. The child calls the unchanged pinned fixed-origin TLS collector. Input/output/body/header bounds and complete canonical framing are checked; partial stdout is never HTTP evidence. Only a complete validated error frame from a successful child exit may retain genuine partial HTTP bytes. Failure kills the process group, falls back to the direct child when signaling fails, and unconditionally waits before reporting failure. This is an acceptance deadline with cleanup before failure, not an OS hard-real-time termination guarantee. Already emitted bytes may still be processed remotely: a timed-out send keeps its original durable INTENT/UNKNOWN, fee reservation and original-signature-only recovery; no retry, resend or replacement is enabled. Quote/preflight failures send nothing.
Production Rpc requires system-default TLS and rejects all existing proxy variables plus nonempty SSL_CERT_FILE/SSL_CERT_DIR/SSLKEYLOGFILE/OPENSSL_CONF/OPENSSL_MODULES/OPENSSL_ENGINES before spawning. The child receives only the pinned repository environment. Injected transports retain their explicit test mode; native c.collect/r.collect still receive no injected wrapper, preserving FIXED_ORIGIN_TLS provenance. Their own whole-operation deadlines remain **OPEN**. Only bounded_transport changes among 55 prior definitions; replay, operate, Journal, main, signer, accounting, state and lifetime/fee/resource predicates stay unchanged. Prior globals and older collector/readback/checker/journal/backend/oracle files remain unchanged.
Root passes **30/30 focused groups first execution** (eight new and 22 relevant regressions), suite118.078 s/runner118.648 s/process118.824 s. The exact worker literal runs only in-process with fake HTTP/streams/resource calls; process/clock/selector/pipe tests are fakes. Historical signatures and existing full-host synthetic seams remain. Python network/forbidden-child attempts are zero;136 pinned public Node helpers run. No actual Python transport child/TLS/native signer/key control is proved. Separate source/actual reviews pass; the cleanup error was corrected before execution and the unexecuted draft and executed final bytes are retained. Unchanged backend/handoff/receipt/rules/CLI/Node/application/runtime evidence is reused, not rerun. No new large fixture is built; all45,039 prior performance-fixture files precede this turn's metadata baseline and its7,503-record tail is unchanged. Its44.445-second result retains its older exact source/history, never rebound to this change. Its synthetic b*64 parent/absent base cannot satisfy actual BIND_SEAL and is not a complete-governance proof.
**Next blocker/completion condition:** bound native governance/recovery collector execution without changing origin/mode/evidence/journal semantics, then establish original-lifetime sufficiency with actual or explicitly bounded signer/checker/public-I/O costs and real height progression, native key control, a legitimate parent/Clock/seal and twelve remaining packets, exact phase resources/funding/evidence/retry capacity and final independent readiness. No margin reduction or automatic HOLD repair. **M4 TECHNICALLY INCOMPLETE / OPEN_UNWAIVED; M5/public NOT READY / NO LIVE TRIGGER.** D-049 separate exact completed-package live authorization remains mandatory. No private/key reads/new signatures, simulation/Solana RPC/send, funds, authority, deployment or Mainnet operation. October21 observations/D-051 stay historical. Integration only; main b7e30b2/protected3677fee unchanged; preserve2GiB.


**Current technical checkpoint — bounded native governance and original-signature collection (2026-10-23 Europe/Warsaw).**
Native governance/recovery collection now runs the unchanged pinned collectors in one fixed isolated worker, preserving `_transport=None` and FIXED_ORIGIN_TLS. A **180-second monotonic result-acceptance cap** covers the worker and parent receipt checks; this is neither a whole-operator/lifetime guarantee nor an OS hard-real-time cleanup bound. The parent securely rereads the exact canonical manifest and every listed artifact, request, original signature or parent/stage/seal binding. It retains the leader's waitable identity until group cleanup and reaps it before returning; an unexpected group-signal failure refuses acceptance even if direct-child cleanup succeeds. Descendant termination after a failed signal is not claimed. Late/incomplete output is not adopted; partial exclusive directories remain evidence only. An interrupted recovery leaves the durable original INTENT and fee reservation unresolved, without cursor advancement, resend or replacement. Existing authoritative journal replay remains mandatory. Proxy/TLS override refusals and all prior resource/evidence/security limits stay intact.
Root passes **28/28 focused groups**, including 7 new groups and 21 relevant operator/RPC/full-account regressions, under connection/DNS and forbidden-child denial. Separate source and actual reviews pass. Worker/process/clock/waitid/HTTP boundaries are explicit host fakes; no actual collector worker, TLS, native signer or chain request runs. Unchanged lifetime/backend/receipt/rules/application/build/runtime evidence is reused. Earlier timing retains its exact older source/history; no complete-startup headroom follows.
**Next completion condition:** establish original-lifetime sufficiency with actual or explicitly bounded native signer/checker/public-I/O costs and real height progression, legitimate parent/Clock/seal and twelve remaining packets; close exact phase resources/funding/evidence/retry capacity and final independent readiness. Native key control remains open. **M4 TECHNICALLY INCOMPLETE / OPEN_UNWAIVED; M5/public NOT READY / NO LIVE TRIGGER.** D-049 separate exact live authorization remains mandatory. No private/key reads, new signatures, simulation/Solana RPC/send, funds, authority, deployment or Mainnet action. October21 observations/D-051 stay historical; integration only, main b7e30b2/protected3677fee unchanged; preserve2GiB.


**Current technical checkpoint — complete native signer/checker acceptance windows (2026-10-23 Europe/Warsaw).**
The operator's native signer and signed-packet checker each enforce a **20-second monotonic result-acceptance window starting at function entry**. Signer pin/payload work and spawn are included; consumed startup budget refuses before invocation. A complete parsed response is accepted only after cleanup and a fresh deadline check. The checker likewise includes trace/payload construction and the unchanged pinned helper through return. Late signer output leaves the original durable SIGNER_ATTEMPT unresolved; no IMPORT or automatic resign follows. Existing source/authorization/signature/state/fee/lifetime/UNKNOWN guards remain. Backend, checker/journal helper, previous RPC30/collector180 paths, all other operator definitions and prior globals are unchanged. This bounds accepted results, not OS execution/cleanup duration, actual key control or whole-transaction headroom.
Root passes **25/25 focused groups first execution**, including 3 new boundary groups and 22 operator/backend/full-account regressions. Tests use fake child/clock/parser/checker-return seams and public historical signatures; no actual signer, key/private read, signing, network or collector child runs. Separate source/actual reviews pass. Unchanged broader suites, builds/runtime, selected artifact and earlier timing evidence are reused at their original scopes. The old post-spawn timeout fixture is adapted for the new entry check without weakening its assertion.
**Next completion condition:** establish supported original-lifetime scheduling with real height progression and actual or bounded remaining costs, legitimate parent/Clock/seal and twelve unreconciled packets (bootstrap recovery plus eleven successors), exact phase resources/funding/evidence/retry capacity and final independent readiness. Component acceptance caps cannot be added into a guaranteed whole-action bound; native key control remains open. **M4 TECHNICALLY INCOMPLETE / OPEN_UNWAIVED; M5/public NOT READY / NO LIVE TRIGGER.** D-049 separate exact completed-package live authorization remains mandatory. No new private/key reads, signatures, simulation/Solana RPC/send, funds, authority, deployment or Mainnet action. October21 observations/D-051 stay historical; integration only, main b7e30b2/protected3677fee unchanged; preserve2GiB.


**Current technical checkpoint — governance readbacks before captured packet lifetime (2026-10-23 Europe/Warsaw).**
For captured PREPARE at current initializer4/8, the existing PRE/POST_VOTE collection and durable gate insertion now precede blockhash capture/signing. The quote must name that exact gate predecessor; each recapture recollects, future gate boundaries refuse, and changed/missing/corrupt gates cannot use the legacy fallback. Genuinely unquoted preparation retains the existing SEND collection. No action/record schema, source pin, margin, authority, fee or economic rule changes.
Fresh replay securely rereads all15 gate artifacts and the exact bound terminal parent SEAL, derives the actual context minimum, and consumes unchanged pure storage/Clock checks with source-derived PDA facts. SEND retains its fresh57-account/authority/rent/fee/auth/original-lifetime/post-INTENT checks; SEND and replayed INTENT reject Clock regression against the gate and enforce current checked-i64 POST timelock arithmetic. No collection, Node helper or validation-result cache is added to the new pure readback validator.
Root validates **26 unique focused groups** across retained and corrected runs, including 4 new and22 relevant regressions, with separate source/actual review. The initial26 run has25 passes/one incomplete controller-fixture error; after adding four public prior-state fields, both affected controller groups pass. Product code is unchanged between runs; original executed bytes/error remain. Selected public unsigned packets, pure raw validation and actual exclusive quote capture/verification run. Parent semantic/BIND_SEAL validation is an explicit prior-state seam; scheduling uses an in-memory journal/replay, authorization and public-I/O seams. No actual SIGN, complete send flow, native collector/signer, new signatures or network runs. Source relocation removes the collection and three full replay call sites from these two postquote intervals; new pure validation still costs local work. No executed8-pass count, wall-time improvement or lifetime headroom is claimed. Unchanged broader suites/build/runtime and all older measurements retain their scopes; no large history is rebuilt or rebound.
**Next completion condition:** demonstrate the remaining original-lifetime schedule with real height progression, native key control, legitimate actual Clock/seal and twelve unreconciled packets (bootstrap plus eleven successors), exact phase resources/funding/evidence/retry capacity and final independent readiness. **M4 TECHNICALLY INCOMPLETE / OPEN_UNWAIVED; M5/public NOT READY / NO LIVE TRIGGER.** D-049 separate exact completed-package live authorization remains mandatory. No private/key reads, new signatures, simulation/Solana RPC/send, funds, authority, deployment or Mainnet action. October21 observations/D-051 stay historical; integration only, main b7e30b2/protected3677fee unchanged; preserve2GiB.


**Current capacity checkpoint — whole-upload evidence and host storage blockers (2026-10-23 Europe/Warsaw).**
Root independently reproduces the selected949-phase account-base64 payload subtotal: **4,827,430,700 bytes**, comprising4,466,412,288 for936Writes and361,018,412 for the other13phases. SEND retains one prestate copy; recovery retains three proof-subset copies and, for full57-account phases, two additional complete-snapshot copies. This is only a lower bound: governance gates, JSON/HTTP metadata, requests, journal, retries and filesystem allocation are excluded. The earlier Buffer-only subtotal remains historical and is not a whole-run bound.
Independent review also identifies a concrete accepted-input counterexample: one valid16-MiB whitespace-padded SEND response per phase alone totals **15,921,577,984 bytes**, above the existing8-GiB aggregate ceiling. Current admission guards stop safely; they do not guarantee completion. No response padding, RPC, native collection, new test/build or full-run fixture is executed. All executable/test/dependency/artifact bytes and prior passing/failed evidence remain unchanged and retain their original scopes.
At the retained root observation, only **4,349,784,064 bytes** are free. Even payload plus the2-GiB reserve requires6,974,914,348 bytes; the shortfall is2,625,130,284 before omitted costs. Reserving the existing8-GiB evidence ceiling,128-MiB journal and2-GiB floor gives a **10,871,635,968-byte planning threshold**, still excluding physical allocation and unrelated growth. Restore at least11GB free, with further headroom; this is a disk planning target, not proof of capacity/readiness. No historical evidence/cache/file is removed or rebound.
**Next completion condition:** restore host headroom and enforce source-derived successful-action serialization envelopes plus a finite failure/recovery reserve within unchanged limits. The proposed1-MiB per-success allowance is DESIGN ONLY / UNIMPLEMENTED / UNPROVED. Retain every failed raw attempt and preserve original-signature recovery: post-INTENT failures must not make the committed preflight prefix unreplayable. Then close legitimate Clock/seal, original-lifetime/native-control and twelve remaining-packet/resource/funding checks and final independent readiness. **M4 TECHNICALLY INCOMPLETE / OPEN_UNWAIVED; M5/public NOT READY / NO LIVE TRIGGER.** D-049 separate exact completed-package live authorization remains mandatory. No keys/private reads, new signatures, Solana RPC/simulation/send, funds, authority, deployment or Mainnet operation. Integration only; main b7e30b2/protected3677fee unchanged. See the capacity continuation in the existing operator evidence register.
