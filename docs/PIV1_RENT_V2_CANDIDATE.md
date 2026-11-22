# PIV1 rent-resilient V2 candidate

22 November 2026 · Source release review

The historical Testnet program `GAjHRaC3XM98SoRoFy5B8P2ypTS1ndf1UPhXmjWgtPux` at slot **466309305** runs source `ae30dbf262ecc86852611dc1cbbab143c510db23` and ELF `005b578cd53534828772c7c05d3f5d1354063ef3f2af0f7f98f88c7d6ee07465`. It uses Config V1 and an Idle V3 round. The [isolated Config V2 Testnet instance](PIV1_2026_11_22_ISOLATED_TESTNET_V2.md) passed scoped public genesis. The code has no V1 import instruction. Existing Testnet staking, liquid settlement, pending integration, restaking, KIF and genuine Squads receipts keep their historical scope. [Published installation checkpoint](https://github.com/HoldTheFuckingPosition/PIV1/tree/ae30dbf262ecc86852611dc1cbbab143c510db23)

## Confirmed accounting and governance

Config V2 retains its 1,014-byte allocation. Five independent little-endian `u64` fields in its existing extension record authenticated storage funding for Pending SOL, Principal SOL, Operational SOL, Distribution Escrow and KIF SOL. Fresh governed IN02 pays the full zero-data rent minimum into every permanent SOL vault, separately from unsolicited SOL. Existing independently funded economic receipts retain their classification. Economic SOL is the actual balance less the recorded technical funding. A rent reduction leaves the recorded funding and economic balances unchanged. Financial paths hold if the authenticated technical reserve cannot cover a higher live floor.

`PIV1RT01` carries an eight-byte selector, one vault-role byte and an exact little-endian `u64` amount. The five ordered accounts are Config, the canonical round, the selected vault, an independent external signer who pays the top-up and the executable System Program. It accepts only the positive live minimum less the stored reserve. The instruction checks economic coverage before transfer, checks both actual balances afterward and writes only the selected technical amount. A covered increase needs no action. A zero, excessive, stale or repeated payment fails. The sponsor earns no refund, payout or other authority.

Strict HWM and principal protection, the D-113 preferred-source exception, the 59% / 19.5% / 19.5% / 2% yield split, heartbeat-only KIF under D-116 and six-guardian four-of-six Squads governance are preserved. The first contribution is externally acquired official JitoSOL under DJ01, integrated atomically through IB01 after Idle V3. In-program SOL conversions keep the strict post-conversion historical book value and HWM floors. [Source decision register](PIV1_DECISIONS.md) · [Top-up](../programs/piv1/src/rent_top_up.rs)

## Genuine local genesis qualification

The **967,696-byte** stripped SBF `2ea454e9eac013bab063651c6691bf1cf84fad3e5d3e0b67375f2603e4138109` completed unpaused IN02 → MR02 → MR03 → atomic DJ01 → IB01 in Agave Bank 4.2.0. The pinned Squads V4 binary accepted four cryptographically verified in-memory local signers for each of the governed actions and invoked PIV1 from canonical Vault 0. The initial local six-member multisig and warmed ALT were seeded. The source JitoSOL account was funded only inside the Bank. The official mint and pool snapshot originated from finalized Testnet slot 467523128, epoch 1095.

| Successful packet | Wire bytes | Executed CU | Loaded bytes |
| --- | ---: | ---: | ---: |
| Squads execute with IN02 | 1,059 | 1,016,648 | 3,906,020 |
| MR02 | 508 | 75,921 | 970,865 |
| Squads execute with MR03 | 795 | 799,444 | 1,963,118 |
| Atomic DJ01 → IB01 | 984 | 1,050,910 | 2,918,037 |

IN02 funded 16 accounts and recorded **650,240** technical lamports in each permanent SOL vault. MR02 used **167,640** separately funded operating lamports and MR03 used **1,397,000** external lamports subject to its exact cap. The **1,118,043,205** JitoSOL base units became **1,499,999,999** protected book lamports, with zero opening yield, pending balances or KIF liability. The exact Bank evidence also preserves rollback when the second instruction refuses, duplicate initializer and bootstrap refusal, and no artificial economic value after a fixture rent reduction. Prior rent and client evidence adds 23 passed Rust methods, a nine-message native Bank rent test and eight passed offline client checks. The independently encoded JavaScript atomic message has the same ordered native actions, effective privileges, signer set and wire size as the Bank packet. Its static key order differs. The resource table belongs to the executed message. [Qualification summary and original evidence digests](evidence/piv1-rent-v2/qualification-summary.json) · [Exact retained Bank harness source](../validation/genesis-bank-runtime/tests/fresh_rent_v2_genesis.rs)

## Reproduce the candidate source profile

Use the locked Cargo graph and Solana platform-tools **v1.54** for the `sbpf-solana-solana` target. From repository root, point `PIV1_PLATFORM_TOOLS` at the extracted v1.54 directory and `PIV1_TARGET_DIR` at a separate build directory. Use an already populated Cargo registry cache.

```sh
sha256sum --check docs/evidence/piv1-rent-v2/production-sources.sha256
export LC_ALL=C NO_DNA=1
export RUSTC="$PIV1_PLATFORM_TOOLS/rust/bin/rustc"
export RUSTDOC="$PIV1_PLATFORM_TOOLS/rust/bin/rustdoc"
export CC="$PIV1_PLATFORM_TOOLS/llvm/bin/clang"
export AR="$PIV1_PLATFORM_TOOLS/llvm/bin/llvm-ar"
export CARGO_TARGET_SBPF_SOLANA_SOLANA_RUSTFLAGS="-Zremap-cwd-prefix= -C linker=$PIV1_PLATFORM_TOOLS/rust/lib/rustlib/x86_64-unknown-linux-gnu/bin/rust-lld"
"$PIV1_PLATFORM_TOOLS/rust/bin/cargo" build --manifest-path programs/piv1/Cargo.toml \
  --package piv1 --lib --release --target sbpf-solana-solana --locked --offline \
  --jobs 1 --target-dir "$PIV1_TARGET_DIR" --no-default-features --features testnet-bounded-recovery
"$PIV1_PLATFORM_TOOLS/llvm/bin/llvm-objcopy" --strip-all \
  "$PIV1_TARGET_DIR/sbpf-solana-solana/release/piv1.so" "$PIV1_TARGET_DIR/piv1-rent-v2.so"
sha256sum "$PIV1_TARGET_DIR/piv1-rent-v2.so"
```

The candidate feature list excludes `testnet-initial-principal-rounding`. The earlier independent two-build artifact `d802ae9a44fc7c66eb0482f2ffca5d6012e19d64a4594eff2486174b1c1f02e3` is the different strict predecessor. The evidence binds the exact V2 program source and separately the published validation package. The source handoff did not rerun either build or a completed test campaign.

The contained [offline V2 client](../validation/rent-v2-client/rent-v2-client.cjs) uses pinned Node v24.19.0, web3.js 1.98.4 from the existing SDK lock, and expects the absolute path of the stripped artifact in `PIV1_V2_ELF`. The old profile CLI and its historical source pins stay in their original locations. The focused tests for this V2 validation package are in [rent-v2-client.test.cjs](../validation/rent-v2-client/rent-v2-client.test.cjs). Packaging only relocated imports, the fixture/source manifest paths and the explicit local artifact path.

The exact [Agave Bank harness source](../validation/genesis-bank-runtime/tests/fresh_rent_v2_genesis.rs) can be selected with `cargo test --manifest-path validation/genesis-bank-runtime/Cargo.toml --locked --offline --test fresh_rent_v2_genesis -- --exact fresh_piv1_v2_genesis` on Rust 1.97.1 with the cached graph. The harness requires explicit `PIV_GENESIS_INPUTS`, its `PIV_GENESIS_INPUTS_SHA256`, the selected `PIV_GENESIS_ELF` and a fresh absent `PIV_GENESIS_BANK_DIR`. The [public capture index](evidence/piv1-rent-v2/public-capture-index.json) records account metadata and exact checksums for the retained public observations. A replay requires the matched historical raw data files. Executables, raw RPC bundles, local signatures and VPS details are excluded from this source package. The original harness was run against the existing locked dependencies. Its new Cargo entry points to the same harness and versions, using dependency packages already present in the lock.

## Open gates

M4 remains **OPEN_UNWAIVED**. The local outer rollback does not cover the stopped internal-callee case. Genuine D-114 incident recovery remains **OPEN and platform-paused**. Fresh initialization and the ordinary governed MR03 are distinct from recovery. The release has no legacy Config import. The isolated Testnet campaign is the only new installation covered by its authorization. Any future cutover requires separate authorization. Natural public delayed-withdrawal eligibility and production admission remain open. There is no Mainnet authorization.
