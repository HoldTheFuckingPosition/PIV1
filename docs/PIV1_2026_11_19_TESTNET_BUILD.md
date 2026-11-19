# Build and verification for the 19 November 2026 Testnet release

The [public receipt](PIV1_2026_11_19_INSTALLATION_RESULT.public.json) binds the approved
source diff and exact program bytes to the finalized Testnet installation on
the existing Idle V3 layout. The feature set is the qualified Testnet profile.

From the repository root, locate the preinstalled Solana platform-tools **v1.54**
(Rust 1.89.0-dev, LLVM 20.1.7), set `PIV_PLATFORM_TOOLS` to that directory and
use the workspace's existing `Cargo.lock`. The offline build requires the
locked packages already in the Cargo cache.

```bash
export LC_ALL=C CARGO_TERM_COLOR=never NO_DNA=1
export RUSTC="$PIV_PLATFORM_TOOLS/rust/bin/rustc"
export RUSTDOC="$PIV_PLATFORM_TOOLS/rust/bin/rustdoc"
export CC="$PIV_PLATFORM_TOOLS/llvm/bin/clang"
export AR="$PIV_PLATFORM_TOOLS/llvm/bin/llvm-ar"
export CARGO_TARGET_SBPF_SOLANA_SOLANA_RUSTFLAGS="-Zremap-cwd-prefix= -C linker=$PIV_PLATFORM_TOOLS/rust/lib/rustlib/x86_64-unknown-linux-gnu/bin/rust-lld"
"$PIV_PLATFORM_TOOLS/rust/bin/cargo" build --manifest-path programs/piv1/Cargo.toml \
  --package piv1 --lib --release --target sbpf-solana-solana --locked --offline --jobs 1 \
  --features testnet-initial-principal-rounding,testnet-bounded-recovery --target-dir target
"$PIV_PLATFORM_TOOLS/llvm/bin/llvm-objcopy" --strip-all \
  target/sbpf-solana-solana/release/piv1.so target/piv1-testnet-combined.so
wc -c target/piv1-testnet-combined.so
sha256sum target/piv1-testnet-combined.so
```

The release result is 956,856 bytes with SHA-256
`005b578cd53534828772c7c05d3f5d1354063ef3f2af0f7f98f88c7d6ee07465`.
The finalized Loader payload adds 6,048 zero bytes after that ELF.

The [Bank validation source](../validation/genesis-bank-runtime/tests/production_lifecycle.rs)
uses the independent [Bank workspace lock](../validation/genesis-bank-runtime/Cargo.lock)
and the pinned SBF programs named by the harness. The focused large-list test
is `delayed_withdrawal_initiates_with_dated_testnet_list_geometry` in that
`production_lifecycle` target. Its prior qualification measured 679,856 CU
with 848 populated records in a 730,009-byte list. See the
[offline planner tests and invocation](../clients/cli/README.md) for source
selection coverage. The finalized Testnet read records the observed route
and its pending public eligibility.
