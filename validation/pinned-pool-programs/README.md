# Pinned external executable programs

This isolated workspace resolves the exact published `spl-stake-pool 2.0.3` crate
with its real entrypoint enabled. Root authenticates
the complete locked source/archive closure and builds that package's own SBF
cdylib directly. This library is a resolution anchor, not a processor substitute.
Production and historical validation dependency graphs remain unchanged.

The local lifecycle Bank loads this artifact and the separate Token artifact at their canonical program IDs,
the separately pinned bundled Core BPF Stake 5.1.0 executable, the exact reviewed
PIV1 production artifact and the existing explicitly synthetic governance caller.
The fixture addresses/accounts do not establish deployment or external ownership.
No live operation, key generation, signing or public transaction submission occurs.

Token 8.0.0 is resolved and built in `../pinned-token-program` because the pool
graph enables Token `no-entrypoint`. Package selection does not isolate unified
features. Preserve the rejected first combined-graph artifact and evidence; neither
changing an ELF gate nor accepting a missing entrypoint resolves that defect.
