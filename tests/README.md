# Test suites

**Current project status — 2026-10-21.** The reviewed M5 preparation through `5663d68` and the current README/checkpoint update are included in the founder-authorized main publication ([D-050](../docs/PIV1_DECISIONS.md#d-050--confirmed--founder-authorized-main-publication-including-readmes)). M4 remains technically incomplete/open/unwaived; the public Testnet package is NOT READY and no deployment is authorized. See the [current pilot state](../docs/PIV1_PILOT_STATE.md) and [readiness/evidence register](../docs/PIV1_M5_TESTNET_READINESS.md). Historical descriptions below retain their original scope and do not imply new test executions or broader founder acceptance.

The `unit`, `integration`, `adversarial`, and `fixtures` boundaries mirror the
accepted test architecture. Task 1.1 adds only a TypeScript compilation
sentinel; it adds no local-validator, Jito, transaction, deployment, or
fund-moving test. Functional tests begin only in their separately authorized
tasks.
