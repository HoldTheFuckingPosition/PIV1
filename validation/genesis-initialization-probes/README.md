# Isolated full-genesis initialization probes

This nonpublishable workspace prepares three validation-only SBF artifacts for
later keyless local runtime work. Production native initialization remains
closed. Task 2.31 is build-only; no probe is loaded or executed here.

- `caller` is synthetic code under the canonical Squads ID. Its bounded parser
  requires one stored 313-byte initialization action, no lookups or ephemeral
  signers, exact fixed inner privileges and canonical transaction/vault PDAs.
  It signs only the vault with the existing canonical seeds. The external payer
  must already be an outer signer and writable. The proposal is readonly inside
  CPI despite the outer writable union. This is not actual Squads governance.
- `callee` calls only the unchanged public
  `initialize_approved_genesis_with_checked_recipients`. A successful call must
  finish normalized allocation, both Token initializations and all state writes.
  Actual nested System/Token errors propagate unchanged; other validation errors
  map to the probe-only `0x2311` category. No partial allocation succeeds.
- `token` accepts only canonical Token ID, exactly two accounts and exactly the
  33-byte InitializeAccount3 encoding (opcode 18 plus owner 32). It then calls the
  unchanged pinned SPL Token 8.0.0 `Processor::process` under SBF. It is not a
  deployed Token artifact or general Token replacement. There is no mock CPI.

Every ordinary-host entrypoint fails closed without invoking any syscall stub.
The host tests exercise only wire/profile/preparation/error boundaries. Caller
fixtures reuse independent Anchor/Borsh serialization and test complete backing
preservation; their synthetic protocol/target bytes do not claim a successful
production initialization. Host test success is not SBF runtime evidence.

## Fixed account contract

| Roles | Distinct receivers | Shared receiver |
| --- | --- | --- |
| Bootstrap | 0..7 | 0..7 |
| Sixteen targets | 7..22 | 7..22 |
| Protocol | 23..29 | 23..28 |
| External payer | 30 | 29 |
| System / Token | 31 /32 | 30 /31 |
| Recipients (vault 0 /255) | 33 /34 | 32 /33 |
| Inner accounts /outer metas | 35 /39 | 34 /38 |
| Stored transaction bytes | 1579 | 1546 |

Only vault role 5 and payer sign the inner action. Only targets 7..22 and payer
are writable. The caller's vault seed group is exactly
`[b"multisig", multisig, b"vault", [vault_index], [vault_bump]]`.
The initializer retains its own unchanged canonical target seeds and validation.

## Build boundary

The standalone lock uses only existing registry name/version/source/checksum
identities. Anchor 0.32.1, PIV1 `no-entrypoint`, and SPL Token 8.0.0 `no-entrypoint`
are pinned; no production dependency or old workspace changes. Release arithmetic
checks, fat LTO and one codegen unit follow the existing target profile.

`tools/build_genesis_initialization_probes.py` builds all three libraries with
locked/offline Cargo and strict diagnostic/ELF guards. Its immutable helper and
historical profile must match exact reviewed hashes before use. The new profile
is derived in memory with only the separately verified installed libexpat hash
refresh from Task 2.30; old pins and helper globals stay unchanged. No installation
occurs. New sources, full feature graph, archives/extracted package sources,
protected prior inputs/artifacts, tools and Git refs are bound before/after.
Only fresh private `/tmp/piv1-genesis-initialization-probes-*` outputs are used.

Exact commands, attempts, reviews and artifacts belong in
`docs/TASK_2_31_GENESIS_INITIALIZATION_PROBES.md`. Root owns all compilation,
testing, static artifact inspection and Git publication. This preparation does
not establish successful initialization, total heap/compute, Bank rollback,
funding provenance, later Token-native donation handling, recipient control,
actual Squads/ALT lifecycle or founder Testnet readiness. No keys/signing, RPC,
deployment, funds, Mainnet operation or authority transfer is included.
