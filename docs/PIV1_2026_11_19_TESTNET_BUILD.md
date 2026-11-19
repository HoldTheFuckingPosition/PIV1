# PIV1 pre-launch review

PIV1 holds SOL and JitoSOL principal, recognizes new contributions in full and protects
principal with a SOL-denominated high-water mark (HWM). The gross allocation is 59% to
the HTFP SOL reserve, 19.5% compounded, 19.5% to the Team Owner Pool and 2% to KIF.
Eligible outgoing NET accounts for approved costs and rounding. Ordinary routes use
available SOL, the bounded Jupiter/Manifest route for a positive shortfall below
1.5 SOL, and delayed Jito stake withdrawal for a shortfall of at least 1.5 SOL.
Delayed withdrawal preserves one frozen round with separately attributed legs.
Four approvals from the six current Squads members govern upgrades and the bounded
governed recovery action. The protected principal, pending contributions, escrow,
operating rent and earned KIF have separate obligations. [Architecture](PIV1_TECHNICAL_ARCHITECTURE.md) ·
[Specification](PIV1_MASTER_SPEC.md) · [Decisions](PIV1_DECISIONS.md)

## Source and artifact identities

| Status | Source and feature profile | Stripped ELF bytes | SHA-256 |
| --- | --- | ---: | --- |
| Installed on Testnet at finalized slot 466309305 | `ae30dbf262ecc86852611dc1cbbab143c510db23`, both Testnet features | 956,856 | `005b578cd53534828772c7c05d3f5d1354063ef3f2af0f7f98f88c7d6ee07465` |
| D-113 local candidate, undeployed | `e8768500401db12ab6a47f0c8bf3d120957f0dbd`, default | 933,968 | `8c2f1175338cc91b34a44ec9721db98b16e9e150ce58fedec5c6fa56c0ef400a` |
| D-113 local candidate, undeployed | `e8768500401db12ab6a47f0c8bf3d120957f0dbd`, both Testnet features | 961,704 | `94f86fd7296b2347e833c7cbf7b7257cf2ed12647d561b3bea15d2eb668e826c` |

The [finalized installation receipt](PIV1_2026_11_19_INSTALLATION_RESULT.public.json)
identifies the Testnet program, ProgramData, Vault 0 and post-install readback.
The installed release preserved Idle V3 and the protected accounts. Its Loader payload
has 6,048 zero bytes after the ELF. D-113 retains the instruction bytes and persisted
layouts. When its precise rule applies, the first leg appends one read-only,
nonsigner alternate Stake witness. The continuation uses a separate ordinary
initiation with refreshed accounts and pool state. [D-113 rule](PIV1_PROVISIONAL_PREFERRED_PARTITION.md) ·
[Client format](../clients/cli/README.md)

## Reproduce the reviewed builds

From a clean checkout of the source commit in the table, set `PIV_PLATFORM_TOOLS`
to the Linux x86-64 Solana platform-tools **v1.54** directory (Rust 1.89.0-dev,
LLVM 20.1.7). The [workspace lock](../Cargo.lock) has SHA-256
`1c6a4ca4123d904e59f5e605046bf156cf3bf1df254d04d284c591c99f9bf805`.
Populate the Cargo cache with its locked dependencies before using `--offline`.
Run the two build commands at D-113. Run the Testnet command at the installed
source commit to reproduce the installed ELF.

```bash
export LC_ALL=C CARGO_TERM_COLOR=never NO_DNA=1
export RUSTC="$PIV_PLATFORM_TOOLS/rust/bin/rustc"
export RUSTDOC="$PIV_PLATFORM_TOOLS/rust/bin/rustdoc"
export CC="$PIV_PLATFORM_TOOLS/llvm/bin/clang"
export AR="$PIV_PLATFORM_TOOLS/llvm/bin/llvm-ar"
export CARGO_TARGET_SBPF_SOLANA_SOLANA_RUSTFLAGS="-Zremap-cwd-prefix= -C linker=$PIV_PLATFORM_TOOLS/rust/lib/rustlib/x86_64-unknown-linux-gnu/bin/rust-lld"
piv_build() {
  local output=$1
  shift
  "$PIV_PLATFORM_TOOLS/rust/bin/cargo" build --manifest-path programs/piv1/Cargo.toml \
    --package piv1 --lib --release --target sbpf-solana-solana --locked --offline \
    --jobs 1 --target-dir target "$@" || return
  "$PIV_PLATFORM_TOOLS/llvm/bin/llvm-objcopy" --strip-all \
    target/sbpf-solana-solana/release/piv1.so "$output" || return
  wc -c "$output"
  sha256sum "$output"
}
piv_build target/piv1-default.so --no-default-features
piv_build target/piv1-testnet-combined.so --features testnet-initial-principal-rounding,testnet-bounded-recovery
```

## Focused verification

The host and offline client checks below use the D-113 checkout, its pinned
[host Rust toolchain](../rust-toolchain.toml), and Node.js with `node:test`.

```bash
cargo test --locked --offline -p piv1 --no-default-features --test withdrawal_leg_execution
cargo test --locked --offline -p piv1 --no-default-features --test recovery_execution \
  d111_default_build_cannot_dispatch_testnet_recovery -- --exact
cargo test --locked --offline -p piv1 --features testnet-initial-principal-rounding,testnet-bounded-recovery \
  --test withdrawal_leg_execution --test withdrawal_preparation_execution
node --test clients/cli/withdrawal-plan.test.cjs clients/cli/withdrawal-source-selection.test.cjs
```

Retained D-113 evidence covers 33 focused Rust checks and 16 client checks. In a
pinned local Agave 4.2.0 Bank on a 730,009-byte list with 848 populated rows, actual
SPL Stake Pool 2.0.3, Stake 5.1.0 and Token 8.0.0 executed both separately submitted
legs. The zero-fee case consumed 796,147 and 719,563 CU. The positive-fee case consumed
862,072 and 739,028 CU. A still-exclusive preference refused the alternate before
SPL withdrawal. The D-113 Testnet rebuild matched its earlier qualified image byte
for byte. On the installed-source profile, realistic single-leg initiation previously
committed at 679,856 CU. These are local SDK/Bank fixtures with synthetic state and
unsigned messages. The D-113 Bank regression ends after Stake deactivation.
[Bank fixture and assertions](../validation/genesis-bank-runtime/tests/production_lifecycle.rs) ·
[Bank dependency lock](../validation/genesis-bank-runtime/Cargo.lock)

## Bounded recovery before production admission

**CONFIRMED.** Production admission requires a qualified D-111-style recovery path
with genuine four-of-six Squads execution. The repository's `testnet-bounded-recovery`
feature currently gates recovery collection (`PIV1RC01`), governed resume (`PIV1RR01`)
and governed V2-to-V3 migration (`PIV1MR03`). The default candidate leaves that gate
disabled. The two-feature candidate also enables the provisional Testnet initial
rounding concession. A future production release must keep
`testnet-initial-principal-rounding` disabled and explicitly record how it enables
the reviewed bounded recovery path. The recovery-only feature combination identifies
the current source path to qualify. Its release artifact and supported initial state
are still open. [Production requirement](PIV1_DECISIONS.md#d-114--confirmed--bounded-governed-recovery-required-for-production-admission) ·
[Feature definitions](../programs/piv1/Cargo.toml) · [Recovery implementation](PIV1_BOUNDED_RECOVERY_IMPLEMENTATION.md)

Recovery collection is permissionless for already initiated canonical inactive legs.
It returns original operational rent, keeps ambiguous excess out of outgoing NET and
recognizes actual cooldown loss once. Resume transfers no funds. It binds the same
round, incident generation, cumulative loss, frozen snapshot, leg counters, funded
escrow and requested destination to the actual Squads approval. A newer incident
requires fresh approvals. Protected coverage and obligations remain enforceable.

| Resume destination | Release-specific acceptance condition |
| --- | --- |
| `EscrowFunded` | All initiated legs collected and the delayed target complete or zero. Actual funded, positive outgoing NET and prospective protected-value coverage pass. Subsequent settlement and integration reconcile the one-time loss, original rent, recipient amounts, earned KIF, full contributions and HWM. |
| `WithdrawalActive` | All initiated legs collected and a positive portion of the fixed target remains unassigned. A fresh authenticated ordinary Active-source quote must cover the required continuation under the current minimum, full-target residual protection and pending replacement checks. Recovery requires the ordinary quote. The D-113 alternate witness applies only to initiation. An unavailable ordinary quote keeps the round in recovery. |

**OPEN exact-artifact proof.** Retained local Bank evidence uses authentic Stake and
Token with a synthetic governance caller and covers the `EscrowFunded` branch through
settlement and integration. The partial `WithdrawalActive` branch has focused host
evidence. Existing four-of-six upgrade receipts establish governance for those
upgrades. The release-specific recovery proof needs the exact PIV1 program, feature
set and ELF, target cluster and runtime, real pinned Squads executable and ProgramData,
actual multisig, canonical Vault PDA, current membership and threshold, proposal and
stored transaction, and a compatible incident and custody account set. It must check
the actual client encoding, payer/rent budget, fresh account observations and resource
headroom on both branches. Acceptance requires the real Vault signer on the exact
approved Squads execution and exact accounting. A stale incident or changed action,
insufficient approvals, unavailable viable source or real coverage deficit must refuse
without releasing the round or moving funds. The current offline client covers V3
inspection and withdrawal planning. Exact release-specific governed recovery message
assembly and review remain open. The pending qualification remains paused.
[Recovery execution](../programs/piv1/src/recovery_execution.rs) ·
[Squads invocation authentication](../programs/piv1/src/squads_execution.rs) ·
[Client boundaries](../clients/cli/README.md)

Public delayed withdrawal still requires natural eligibility, current authenticated
pool and stake, and operating funds. Production also needs its own authenticated
dependency and recipient tuple. The installed Testnet deployment and local D-113
qualification leave M4 `OPEN_UNWAIVED` and M5/public `NOT_READY`.
[Pilot state](PIV1_PILOT_STATE.md) · [Readiness register](PIV1_M5_TESTNET_READINESS.md)
