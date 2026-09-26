# Keyless full-genesis initialization runtime validation

This isolated, nonpublishable workspace loads the exact three Task 2.31 SBF
artifacts: a synthetic Squads caller, the full recipient-checked initialization
callee and the canonical Token-ID InitializeAccount3-only SPL Token 8 wrapper.
It uses pinned Mollusk 0.15.1 and its actual System builtin. No production source,
probe, earlier harness or economic definition changes.

The complete 35/34-account fixture reuses the unchanged Task 2.30 input serializer.
Its expected nine state envelopes, Token bytes, target sizes, original rent
shortfalls, prefund normalization and CPI instructions use independent literal
wire recipes. No production model factory or host success invoker is an oracle.
Actual runtime Instructions is generated from the complete submitted message;
Clock/Rent use runtime sysvars. Full supplied, raw-before, raw-after and returned
account bytes are recorded, with exact inner privileges and height-three calls.

Four intended success profiles combine distinct/shared fee receivers, fresh and
mixed prefunded targets, and normal/paused initialization. A two-instruction
message must first complete initialization then reject a malformed outer action.
Raw initialized state is distinct from Mollusk's returned original-account vector:
this is context discard, not Bank/AccountsDB rollback. Other cases reject approval,
timelock, recipient rent, mint, original payer shortfall, missing payer signature,
direct invocation and outer discriminator violations. The unchanged artifacts are
first tested at the default 32 KiB heap and explicit 1,400,000 CU ceiling. A resource
failure remains a failure; the oracle or budget must not silently widen.

Use only the reviewed `tools/validate_genesis_initialization.py` entrypoint with
reviewed runner/pin hashes. Build produces one host binary; execution separately
requires its reviewed exact hash. Every stage is locked/offline, uses a fresh
private output, and checks complete inputs/packages/tools/artifacts before/after.
No SBF rebuild, Bank, validator, RPC, key, signature or public-network action exists.
See `docs/TASK_2_32_GENESIS_INITIALIZATION_RUNTIME.md` for actual evidence and limits.
