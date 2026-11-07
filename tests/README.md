# Test suites

**Current evidence and publication — 2026-11-07.** Reuse the recorded focused source-bound host/Bank checks, independent reviews and actual Testnet receipts in the [canonical checkpoint](../docs/PIV1_PILOT_STATE.md). The active ELF's captured SBF integration consumes594108CU, settlement regression694052CU; actual public integration consumes594108CU and the latest strict staking930851CU. Five changed amount-selection/construction checks pass for the latest strict partial stake. These are distinct scoped results, not a new whole-suite pass. Delayed withdrawal/cooldown remains untested on public Testnet. D106 main publication runs only targeted Git/content/secret/link checks; no unchanged test suite, build or chain operation is repeated. Historical descriptions below retain their original evidence scope.

**Current project status — 2026-10-23.** The reviewed seven-commit M5 continuation through `3b79bb3` and this documentation checkpoint accompany the founder-authorized main publication ([D-053](../docs/PIV1_DECISIONS.md#d-053--confirmed--founder-authorized-main-publication-of-reviewed-m5-continuation)). Canonical startup coverage and durable public signer-request/response binding are validated within their documented offline scopes; actual signer backend/control and final resource/lifetime/upload/readiness work remain open. M4 stays technically incomplete/open/unwaived; the public Testnet package is NOT READY and D-049 requires separate exact live authorization. See the [pilot state](../docs/PIV1_PILOT_STATE.md) and [readiness/evidence register](../docs/PIV1_M5_TESTNET_READINESS.md). Historical descriptions below retain their original scopes. No program/test code or artifact changes, new test/build/runtime execution or chain refresh occur in this documentation publication.

The `unit`, `integration`, `adversarial`, and `fixtures` boundaries mirror the
accepted test architecture. Task 1.1 adds only a TypeScript compilation
sentinel; it adds no local-validator, Jito, transaction, deployment, or
fund-moving test. Functional tests begin only in their separately authorized
tasks.
