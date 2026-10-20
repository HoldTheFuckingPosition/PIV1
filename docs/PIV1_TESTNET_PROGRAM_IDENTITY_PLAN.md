# Proposed disposable Testnet PIV program identity

**PROPOSED_NOT_AUTHORIZED — generation only, one key, no network action.**
Prepared on 2026-10-20 from integration `a42aed643a3dc202e804bab98525336c5931e787`.
The [machine index](PIV1_TESTNET_PROGRAM_IDENTITY_PLAN.json) describes this proposal;
the [single readiness register](PIV1_M5_TESTNET_READINESS.md) remains authoritative.
No proposed private directory, key file or key has been created by this task.

The dedicated PIV Program ID remains unknown. [Master specification §5](PIV1_MASTER_SPEC.md#5-dedicated-program-id)
requires a dedicated identity. D-041's seven guardian/payer keys and D-043's one
Squads creation/return are completed, consumed scopes; neither authorizes this key.
Generic continuation instructions do not replace explicit human authorization.

## Exact authorization card

| Item | Proposed limit |
|---|---|
| Purpose | One disposable **Testnet-only PIV program identity** |
| Key kind/count | One conventional on-curve Ed25519 keypair, OS-backed cryptographic randomness |
| New guardians, payer or ALT-authority keys | Zero |
| Funding, economic funds or account rent | Zero lamports |
| Transactions, signatures, network fees | Zero transactions, zero signing, zero lamports |
| Network actions | Zero RPC calls, submissions or deployment |
| Upgrade-authority transfer, Mainnet action, main publication | Not included |
| Authorization | Explicit human approval of this exact one-use generation scope, still missing |
| Stop point | One securely stored keypair and its public key only; stop before funding/signing/deployment |

No ongoing grant, automatic retry, regeneration or later action follows from
approval or elapsed time. A failure does not renew this one-attempt scope. Future
funding, deployment, signing and authority transfer require their own exact approval.

## Proposed private storage and failure boundary

The sole proposed key file is
`/home/jerem/.local/share/piv1/testnet-program-identity/setup-20261020-a/piv1-program-keypair.json`.
Its new setup directory is owned by `jerem`, mode `0700`; the ordinary Solana JSON
keypair file is owned by `jerem`, mode `0600`. It stays outside Git and all public
evidence. Required private parents must have safe ownership/permissions; never
follow symlinks. No global wallet/configuration setting is changed.

Future execution must exclusively create the setup directory and file without
following symlinks or overwriting anything. If the setup directory/file already
exists, ownership/path checks fail, or an attempt partially fails: **STOP**. Preserve
any private partial state without printing it; report only nonsecret status. Do not
retry, delete, overwrite or generate a replacement automatically. Validate the one
completed keypair locally without signing a message, then disclose only its public
key and nonsecret completion metadata. No seed, private bytes, mnemonic, private
file hash or private contents may enter logs, chat, Git or public receipts.

No backup or cloud-sync copy is created under this scope. This is an ordinary
POSIX file **unencrypted at rest**, not independent custody. No guarantee about
VPS snapshots, pre-existing synchronization or memory zeroization is asserted.
Future generation reads no pre-existing private key, including the six guardians,
payer/H2 and original 4WK funding source. All remain untouched.

## Retained artifact and governance facts

The retained PIV candidate at
`/tmp/piv1-bank-smoke-guardians-sbf-20261018-b/artifacts/piv1.so` is **894,552 bytes**,
SHA-256 `e29735e1467da98ecddb1eea9fd25abaffa4a450bd899ddc137f7d8c0da35b99`.
Retained receipt `/tmp/piv1-m5-founder-package-20261019-a/artifact-binding.json`
(SHA-256 `8b7883bb122e9156cc5006191c97f972870a9fb323227995088642e313184f8c`)
binds 132 inputs at its recorded `6c44989` source checkpoint, fingerprint
`20b6707947eed5d3ab19fcb4e444065bcf1e7728dac883ffe503746793cb6f4a`.
These are retained bindings; root's local fresh hash check is separate evidence.
This candidate is not selected for deployment, publicly ready or professionally
audited. No new build, reproduction, ELF execution or live portability proof is claimed.

The native [entrypoint](../programs/piv1/src/lib.rs) passes the actual runtime
Program ID into the [instruction boundary](../programs/piv1/src/instruction_boundary.rs).
Generating and later binding a public identity therefore requires no Rust edit
solely to insert a static ID. This source fact does not select an ELF or prove its
future loader/runtime behavior. Future exact artifact, public ID, derived accounts
and loader/authority state still require their own reviewed binding.

[Retained D-043 evidence](PIV1_TESTNET_SQUADS_SETUP.json) records six members/4-of-6
at `9mwto7RNokYc8n8MPtzHpubwwnKcborY2r9RnfZAScVS`, finalized slot454165126,
with H2 drained. This is not fresh RPC, independent custody or four public PIV votes.
No existing guardian/payer key or multisig is replaced or recreated.

## Remaining package and scope

After separately authorized generation, record the actual public key; it is null
now. Exact deployment artifact selection, program-derived accounts, funded payer
and fee/rent envelope, ALT authority/table and provisioning, governance/HTFP/Team
vault indices and funded recipients, PIV guardian slot assignment, initial pause,
KIF anchor and authorized test amounts remain open. Existing economics and the
six-guardian/4-of-6 requirement are retained; no missing input is silently chosen.
Public signing/submission and the exact deployment/authority package need separate
review and authorization. External Squads member rotation remains a separate scope.

M4 is technically incomplete, criterion **OPEN_UNWAIVED**, stopped drafts untouched.
M5 is preparation only; public package **NOT READY**, PIV deployment **NOT AUTHORIZED**.
This documentation creates no identity, moves no funds and grants no authority.
