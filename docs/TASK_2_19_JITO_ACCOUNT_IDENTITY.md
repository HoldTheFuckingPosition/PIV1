# Task 2.19 — Jito/SPL account identity

Status: **TECHNICALLY VALIDATED / PENDING FOUNDER ACCEPTANCE**.
Base: `f4bed3a44bbe4da4cf1e5304f04e2caeb85d83f1`, `integration/piv1-testnet`.
This D-026 source/host prerequisite authenticates seven supplied accounts against
fixed source identities. It does not initialize PIV1 or establish execution readiness.

## Boundary

`authenticate_jito_identity` binds seven declarations to actual Program, pool,
validator list, reserve stake, mint, manager fee receiver and referrer accounts.
Program/pool/mint constants come independently from the pinned official Jito
reference. State accounts require their canonical owners and nonexecutable status.
Only manager/referrer account aliasing is allowed. Writable/signer flags are not
restricted; lamports are never borrowed or reclassified. All data borrows are
fallible and input accounts remain unchanged.

- The narrow executable profile is loader-v3: exact 36-byte Program encoding,
  tag 2 and canonical ProgramData PDA pointer. No ProgramData bytes, cluster
  genesis, deployed artifact hash or upgrade authority are authenticated.
- The complete SPL 2.0.3 pool prefix consumes 435–611 bytes through bounded
  borrowed reads with no input-sized allocation. Four optional keys and three
  `FutureEpoch` fee fields use their ordinary Borsh tags. Arbitrary trailing
  allocation, including stale nonzero bytes, is accepted. Malformed tags and
  truncations reject; a future incompatible layout is unsupported. Compatible
  prefixes alone cannot detect an upgraded implementation.
- Pool links bind the actual list/reserve/mint/manager fee and legacy Token ID.
  The withdraw PDA derives from `[pool, "withdraw"]` under the fixed SPL ID and
  must match the serialized bump. Pool lockup must be default.
- Validator list geometry requires type 2, a nine-byte header, positive maximum,
  count at most maximum and `maximum == floor((data_len - 9) / 73)`. Unused
  entries and 0–72 residual bytes are opaque. Entries are not scanned or attested.
- Reserve stake requires exactly 200 bytes and `StakeStateV2::Initialized`;
  both authorities equal the withdraw PDA and lockup is default. Its 124-byte
  logical payload permits a nonzero allocation tail. Serialized rent reserve
  remains metadata, without current rent or balance evidence.
- Existing Token 8 `Pack` validates the initialized 82-byte mint: nine decimals,
  withdraw PDA mint authority and no freeze authority. The 165-byte receivers
  must be initialized, unfrozen, nonnative JitoSOL accounts. Their token owners,
  delegates and close authorities are preserved as raw facts. The referrer is a
  declared valid receiver, without any inferred official referral designation.

All current/pending fractions retain raw denominator/numerator values, subject
only to numerator ≤ denominator; referral percentages are at most 100. Both
zero representations (0/0 and 0/nonzero) and 100% remain valid. No accounting
mock `FeeFraction` conversion, fee increase policy or economic fee choice occurs.
Optional authorities, validator preferences, future-fee stages, epochs, total
lamports and supplies remain raw observations. Current epoch, pool/mint supply
equality, bootstrap balance consistency, quotes and executable deposit/withdraw
conditions are deferred. Permissionless token burning is one reason identity
cannot imply stored pool supply equals current mint supply.

The private-field, non-Clone result is point-in-time evidence, with getters for
all decoded pool fields, list geometry, reserve metadata and Token facts. Refresh
after mutation/CPI. No governance, genesis-model composition, funding, custody,
recipient control, persistent capability or effect-once claim is added. Existing
genesis declarations remain explicitly unverified until a later composition.

## Sources and independent serializer evidence

The official [Jito constants](https://github.com/jito-foundation/jito-stake-unstake-reference/blob/b553e90d39e1ff583011dab344a11b5d9bfd284c/constants/index.ts)
and [reference README](https://github.com/jito-foundation/jito-stake-unstake-reference/blob/b553e90d39e1ff583011dab344a11b5d9bfd284c/README.md)
pin revision `b553e90d39e1ff583011dab344a11b5d9bfd284c`. Rechecked public master
copies matched these pinned bytes on 2026-09-14. The fixed identities are
`SPoo1Ku8WFXoNDMHPsrGSTSG1Y47rzgn41SLUNakuHy`,
`Jito4APyf642JPZPx3hGc6WWJ8zPKtRbRs4P815Awbb`, and
`J1toso1uCk3RLmjorhTtrVwY9HJ7X8V9yYac6Y7kGCPn`.
Source support is not proof that these accounts exist on a particular cluster;
this task makes no verified Testnet deployment claim.

SPL 2.0.3 [state](https://github.com/solana-program/stake-pool/blob/864ba3c1c564cc270ca62b6e6b558f57538ae092/program/src/state.rs),
[processor](https://github.com/solana-program/stake-pool/blob/864ba3c1c564cc270ca62b6e6b558f57538ae092/program/src/processor.rs)
and [PDA/program definitions](https://github.com/solana-program/stake-pool/blob/864ba3c1c564cc270ca62b6e6b558f57538ae092/program/src/lib.rs)
pin revision `864ba3c1c564cc270ca62b6e6b558f57538ae092`. The cached archive SHA-256
is `6f0db03f091f43b5766296e80088718491b50949cd3eb4cce3e0cfed58fe2c18`;
full `state.rs` SHA-256 is
`64e9fde6944c036678eba10ab5ddd20d0b2b287b97222d5698c94960c20e6502`.

Test support copies exact `state.rs` lines 30–159 (`AccountType`/`StakePool`),
865–887 (`FutureEpoch` plus small impls) and 921–932 (`Fee`, followed by its
blank separator). Imports and fixture/assertion code are locally authored.
The real locked Borsh 1.8.0 serializer, actual Pubkey and pinned
[stake-interface types](https://github.com/solana-program/stake/blob/aa180e320bca496163d8768c3b9a4bcea2cfb463/interface/src/state.rs)
provide the oracle. This is extracted-type serialization evidence, not execution
of the complete SPL program. The reserve oracle serializes the actual
`StakeStateV2::Initialized` type, independently confirming its 124-byte prefix.
The stake-interface 1.2.1 archive SHA-256 matches the existing lock checksum:
`5269e89fde216b4d7e1d1739cf5303f8398a1ff372a81232abbee80e554a838c`.

Source manifests and hashes are retained at
`/tmp/piv1-jito-identity-readonly-20260914-hgntcri4/manifest.json` and
`/tmp/piv1-t219-root-stake-source-20260914-a/manifest.json`.
The former preserves an initial inert-copy interruption at a missing
`.cargo-checksum.json`, followed by successful source collection; this was not
a build/test failure. The oracle license was copied from the installed
`/usr/share/common-licenses/Apache-2.0`, the standard
[Apache License 2.0](https://www.apache.org/licenses/LICENSE-2.0), **not** extracted
from the SPL archive (which has no license file). Its retained SHA-256 is
`cfc7749b96f63bd31c3c42b5c471bf756814053e847c10f3eb003417bc523d30`.
Upstream Cargo metadata declares Apache-2.0; source attribution is retained.

## Dependency and validation evidence

Production adds no dependency. Authorized test-only direct edges are exact
`borsh1 = { package = "borsh", version = "=1.8.0", features = ["derive"] }`
and `solana-stake-interface = { version = "=1.2.1", features = ["borsh"] }`.
The explicit Borsh alias avoids substituting Anchor's 0.10 serializer. A full
SPL dependency was unnecessary; root's static spike-lock comparison found 94
transitive package/version entries whose names are absent from production, not
an actual feature-resolution measurement for this task.

Offline `cargo metadata --no-deps` left the lock unchanged; full offline metadata
resolution then added only five dependency edges: two from PIV1, two existing
Borsh versions from stake-interface, and Borsh 1.8 from solana-instruction. All
168 package identities, versions, sources and checksums remain unchanged. The
delta was inspected before compilation. No install or package download occurred.

After source inspection, the single focused command used direct verified
Rust/Cargo 1.97.1, a clean explicit environment, existing cache and one job:

```text
/home/jerem/.rustup/toolchains/1.97.1-x86_64-unknown-linux-gnu/bin/cargo test --locked --offline --jobs 1 -p piv1 --lib --test jito_identity jito_identity
```

**12 tests PASS**: three private parser unit tests and nine integration tests;
zero failures, ignored tests or warnings, elapsed 9.60 seconds. The oracle covers
all 16 option masks × 27 future-tag combinations (432 cases), comparing every
field, all maximum-prefix truncations, malformed tags, raw fee edge cases and
nonzero tails. Integration cases cover every role's identity/owner/executable/
borrow failures, loader metadata, pool links/lockups, all list residual sizes,
actual stake serialization, Token restrictions and allowed authority/alias
profiles. Success and failure paths compare complete input snapshots; holding
all lamport borrows does not prevent identity authentication. No Cargo preparation,
build or test command failed and no corrective test retry was needed.

Review corrections removed the public test decoder in favor of private unit
tests and rejected native flags on JitoSOL receivers. Before compilation, API
inspection removed an unsupported `Eq` derive because Token 8 exposes
`PartialEq`. These were source corrections, not failed test attempts.

Evidence directory: `/tmp/piv1-t219-writer-20260914-cbfq5yru`, containing exact
commands/environment, lock comparison, verified tool hashes, logs and unchanged
before/after source manifests. Stdout SHA-256:
`fed7604105614df47c2785b30d47acf5fafd4f71c30205b7fedc4adce7187139`;
stderr SHA-256:
`7759fb48396feca93f2d05215d31685f2d6476bd5171a2f3df0ab379f8fe4ff0`.
Root independently inspected the complete source/tests and exact upstream excerpts,
verified the five lock edges and all 168 unchanged package identities, then executed
the eight final workspace gates on the same freeze: **416 host tests +1 doctest
PASS**, zero failures, ignored tests or diagnostics. Every command used the direct
verified 1.97.1 toolchain, a clean explicit environment, existing cache and
`--locked --offline --jobs 1`:

```text
cargo test --workspace --all-targets --quiet
cargo test --workspace --doc
cargo check --workspace --all-targets
cargo check --workspace --all-targets --all-features
cargo check -p piv1 --all-targets --features no-entrypoint
cargo check -p piv1 --all-targets --features cpi
cargo check -p piv1 --all-targets --features idl-build
RUSTDOCFLAGS='-D warnings' cargo doc --workspace --no-deps
```

Final root evidence: `/tmp/piv1-t219-pilot-host-20260914-a/pilot-summary.json` and
adjacent exact commands, environment, tool/source manifests and logs. Elapsed gate
time: 100.068 seconds. All sixteen final log hashes and both writer logs matched;
full workspace source preservation, the inspected/writer freeze and unchanged
previously verified host tools were checked. Results SHA-256:
`b07ecd412036150c189932489ede73b686d3606da5ba71e3bfa31fa697d07108`.
Root inspected-source manifest SHA-256:
`a31e0975ba744b95b681052542d4d72f4476e624df127d4347c0a25b9a41a4fa`.
Separate final source/test/dependency/report review passed with no actionable
findings. Publication/checkpoint closure follows; this is not founder acceptance.

Next dependency: compose fresh approved genesis parameters with current protocol
identity, validate actual target accounts and establish prefunding-safe atomic
creation plus funding provenance. These need a bounded technical scope before
implementation. This task grants no new deployment, key/signing or live-operation
authorization. Earlier SBF artifacts remain historical evidence for older source.

Writer changes: integration module/export, `tests/jito_identity.rs`, oracle
support/license, PIV1 test-only manifest edges, minimal lock edges and this report.
Existing markers, genesis preparation, state schemas, instruction/error ABI and
runtime entrypoint remain unchanged. No writer Git operation or commit occurred;
root owns status/publication and main remains under its preservation check.
No SBF build, RPC, live account read, deployment, Mainnet action, fund movement,
key creation, signing or authority transfer occurred. AI-assisted review is not
a professional independent audit.
