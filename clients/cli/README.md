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
