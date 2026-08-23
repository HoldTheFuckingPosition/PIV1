# PIV1 Decision Register v0.2

**Project:** HTFP Project  
**Component:** PIV1 - Perpetual Income Vault 1  
**Date:** 2026-08-03  
**Purpose:** Compact source of truth for the dedicated PIV1 development chat and Codex.

## Status legend

- **CONFIRMED**: explicit current founder decision.
- **PROVISIONAL**: current implementation direction, subject to validation.
- **OPEN**: unresolved or must be measured before final implementation.
- **HISTORICAL**: older direction, no longer current.
- **REJECTED**: explicitly abandoned.

## Product and economic decisions

| ID | Status | Decision |
|---|---|---|
| P-001 | CONFIRMED | PIV1 is the first production infrastructure brick of the HTFP Project. |
| P-002 | CONFIRMED | PIV1 has its own dedicated Solana Program ID, isolated from future PIVs. |
| P-003 | CONFIRMED | Principal is intended to remain locked perpetually. Deposits create no individual withdrawal, ownership, or reward rights. |
| P-004 | CONFIRMED | PIV1 accepts SOL and JitoSOL. |
| P-005 | CONFIRMED | PIV1 uses JitoSOL as its initial staking asset. |
| P-006 | CONFIRMED | PIV1 uses direct Jito stake-pool deposit and delayed direct withdrawal. Jupiter/DEX swaps are excluded from the core V1. |
| P-007 | CONFIRMED | All beneficiary outputs are in SOL. |
| P-008 | CONFIRMED | Yield split: 59% HTFP SOL reserve; 19.5% compound; 19.5% Team Owner Pool; 2% active KIF guardians. |
| P-009 | CONFIRMED | Distribution preparation is permissionless. The caller pays Solana transaction fees and receives no automatic caller reward. |
| P-010 | CONFIRMED | Minimum interval between distribution preparations: 10 days. It is a minimum, not a maximum. |
| P-011 | CONFIRMED | The 10-day clock starts when the distribution snapshot/preparation is successfully recorded, not when final payment occurs. |
| P-012 | CONFIRMED | Only one distribution may be active. A new one cannot begin until the previous one is finalized or recovered through an authorized upgrade. |
| P-013 | CONFIRMED | SOL received from outside is used first to fund outgoing distributions; only the missing amount is withdrawn from JitoSOL. |
| P-014 | CONFIRMED | Any unused external SOL contribution becomes principal after reconciliation. Economically, the entire valid contribution increases principal; using liquid SOL for payment reclassifies an equivalent amount of historical yield left staked into principal. |
| P-015 | CONFIRMED | Direct JitoSOL contributions are incorporated into principal at the next snapshot/reconciliation, not retroactively treated as historical yield. |
| P-016 | CONFIRMED | Contributions received while a distribution is active remain in separate pending queues until finalization/reconciliation. |
| P-017 | CONFIRMED | Principal accounting and high-water mark are denominated in SOL lamports. |
| P-018 | CONFIRMED | Yield is measured using the official Jito stake-pool exchange/redemption accounting, not a DEX market price. |
| P-019 | CONFIRMED | High-water mark is not reduced after a loss. No distribution occurs until the protected principal value is recovered. |
| P-020 | CONFIRMED | All distributable calculations use integer arithmetic and conservative floors. Unrepresentable fractions never leave PIV1. |
| P-021 | CONFIRMED | Rounding dust remains non-distributed value and is reconsidered conservatively in a later cycle. It must never cause principal erosion. |
| P-022 | CONFIRMED | Network transaction fees are paid by the transaction fee payer, never reimbursed by PIV1. |
| P-023 | CONFIRMED | Jito withdrawal/protocol costs reduce the outgoing beneficiary allocation, not protected principal or the 19.5% compound share. |
| P-024 | CONFIRMED | Final beneficiary payment is atomic: all beneficiary accounting and payments succeed together or none do. |
| P-025 | CONFIRMED | No arbitrary economic minimum is required merely to protect a caller from fees; the caller chooses whether the transaction is worthwhile. |
| P-026 | OPEN | A mandatory technical minimum may be required when a delayed Jito withdrawal must create a stake account. It must be measured from current rent, minimum delegation, pool constraints, and protocol fees. |
| P-027 | CONFIRMED | No permanent economic SOL reserve is maintained. Only rent-exempt/account-operational balances may remain. |
| P-028 | CONFIRMED | No deposit-cap is planned for launch. |
| P-029 | CONFIRMED | If a valid distribution attempt is below the technical Jito-withdrawal minimum, no snapshot or withdrawal is created; yield continues accumulating. |
| P-030 | CONFIRMED | After a valid insufficient attempt, another insufficiency evaluation is blocked for 24 hours. This retry cooldown does not reset the 10-day distribution clock. |
| P-031 | CONFIRMED | Malformed or otherwise failing transactions must not update the 24-hour retry cooldown. |
| P-032 | CONFIRMED | PIV1 V1 allows only one active prepared distribution; parallel distributions are excluded. |
| P-033 | CONFIRMED | PIV1 maintains a small operational SOL reserve solely to pre-fund rent-exempt temporary withdrawal accounts. Recovered rent returns to and is recycled by this reserve. |
| P-034 | CONFIRMED | The operational rent reserve is excluded from principal, yield, contribution, and beneficiary accounting. |

## KIF guardian decisions

| ID | Status | Decision |
|---|---|---|
| K-001 | CONFIRMED | Six guardian keys exist; each key corresponds to one KIF guardian. |
| K-002 | CONFIRMED | Governance threshold is 4 of 6 for pause, upgrades, key changes, recipient changes, and other program modifications. |
| K-003 | CONFIRMED | Initially, the founder may control all six wallets. Guardians will later be rotated to independent people/entities. |
| K-004 | CONFIRMED | Future guardian rotation occurs by replacing a public key in the multisig. Private keys/seed phrases are never transferred to another person. |
| K-005 | CONFIRMED | Only guardians active in the applicable KIF period earn that period's KIF allocation. Inactive guardians earn nothing and receive no retroactive compensation. |
| K-006 | CONFIRMED | When at least one guardian is active, the full available KIF allocation is divided equally among active guardians, with conservative rounding. |
| K-007 | CONFIRMED | KIF earnings are credited to individual claimable balances; guardians may claim later. |
| K-008 | CONFIRMED | If zero guardians are active, 50% of the available KIF allocation is compounded into principal and 50% is carried into the next KIF allocation. |
| K-009 | PROVISIONAL | If zero guardians remain active for multiple periods, apply the 50/50 rule to the total available KIF pool for that period: new 2% allocation plus prior carry. |
| K-010 | PROVISIONAL | KIF activity period is 30 days. A signed heartbeat/attestation counts as activity; participation in a real governance vote also counts. |
| K-011 | OPEN | Exact expansion/meaning of the acronym KIF. The identifier may remain unexplained in code until branding is confirmed. |

## Governance and authority decisions

| ID | Status | Decision |
|---|---|---|
| G-001 | CONFIRMED | PIV1 remains upgradeable under a 4/6 multisig; guardians are intentionally allowed to modify the full program when necessary. |
| G-002 | CONFIRMED | Squads is the preferred multisig implementation unless Phase 0 finds a concrete incompatibility. |
| G-003 | CONFIRMED | An explicit emergency pause instruction is included despite full upgradeability. |
| G-004 | CONFIRMED | Pause blocks new snapshots, Jito deposits/conversions, delayed withdrawals, finalizations, and migrations. Incoming direct transfers may still arrive and remain pending. |
| G-005 | CONFIRMED | Upgrades and pause may execute immediately after reaching 4/6 approval; no mandatory public timelock is required. |
| G-006 | CONFIRMED | Temporary mainnet recipient treasuries are allowed before HTFP Vault and Team Owner Pool exist. They must be real controlled addresses, not null/empty addresses. |
| G-007 | CONFIRMED | Recipient addresses can later be replaced under 4/6 authority. |
| G-008 | CONFIRMED | Mainnet program upgrade authority must ultimately be held by the 4/6 Squads multisig, not a single hot wallet. |

## Development and launch decisions

| ID | Status | Decision |
|---|---|---|
| D-001 | CONFIRMED | Development is performed with Codex on a Linux VPS. |
| D-002 | CONFIRMED | Mainnet keys, guardian seed phrases, and final deployment signing secrets must not be stored in the repository or given to Codex. |
| D-003 | CONFIRMED | Rust + Anchor is the default stack unless a documented technical reason requires native Solana Rust for a specific layer. |
| D-004 | CONFIRMED | Tests include Rust unit/property tests and TypeScript integration tests. |
| D-005 | CONFIRMED | Local testing uses a controllable mock stake pool/Jito adapter before real cluster integration. |
| D-006 | CONFIRMED | Integration then moves to Solana Testnet using the current officially supported Jito Testnet deployment. Devnet is rejected for the Jito integration unless future official support and current on-chain state are reverified. |
| D-007 | CONFIRMED | Mainnet comes only after local and Testnet validation, adversarial review, verified/reproducible build work, recipient verification, and explicit founder approval. |
| D-008 | CONFIRMED | Initial mainnet capital is expected to be approximately 1-2 SOL, but this is not an on-chain cap. |
| D-009 | CONFIRMED | Code is published by public launch/mainnet launch and must support independent build verification. |
| D-010 | CONFIRMED | AI and community reviews are used, but the project must not claim an independent professional audit unless one actually occurs. |
| D-011 | CONFIRMED | CLI comes before the public dashboard. |
| D-012 | CONFIRMED | A public dashboard is planned, potentially as part of the later HTFP website. |

## Historical/rejected directions

| ID | Status | Direction |
|---|---|---|
| H-001 | HISTORICAL | Native staking as PIV1 V1. It remains a possible future migration option. |
| H-002 | HISTORICAL | Five guardians with 3/5 LST changes and 4/5 major changes. Replaced by six guardians and 4/6 authority. |
| H-003 | HISTORICAL | Fully immutable PIV1 after launch. Replaced by full 4/6 upgrade authority. |
| H-004 | REJECTED | Jupiter/DEX swap as the normal JitoSOL-to-SOL exit path in V1. |
| H-005 | REJECTED | Transferring an existing guardian wallet's private key to a new guardian. |
| H-006 | REJECTED | NFT-based guardian seats for PIV1 V1. |
| H-007 | REJECTED | Paying inactive guardians retroactively. |
