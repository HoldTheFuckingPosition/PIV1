# Test suites

**Current project status — 2026-10-23.** The reviewed seven-commit M5 continuation through `3b79bb3` and this documentation checkpoint accompany the founder-authorized main publication ([D-053](../docs/PIV1_DECISIONS.md#d-053--confirmed--founder-authorized-main-publication-of-reviewed-m5-continuation)). Canonical startup coverage and durable public signer-request/response binding are validated within their documented offline scopes; actual signer backend/control and final resource/lifetime/upload/readiness work remain open. M4 stays technically incomplete/open/unwaived; the public Testnet package is NOT READY and D-049 requires separate exact live authorization. See the [pilot state](../docs/PIV1_PILOT_STATE.md) and [readiness/evidence register](../docs/PIV1_M5_TESTNET_READINESS.md). Historical descriptions below retain their original scopes. No program/test code or artifact changes, new test/build/runtime execution or chain refresh occur in this documentation publication.

The `unit`, `integration`, `adversarial`, and `fixtures` boundaries mirror the
accepted test architecture. Task 1.1 adds only a TypeScript compilation
sentinel; it adds no local-validator, Jito, transaction, deployment, or
fund-moving test. Functional tests begin only in their separately authorized
tasks.
