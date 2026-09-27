# Keyless initialization failure runtime validation

Task 2.33 exercises the exact three unchanged Task 2.31 SBF artifacts through
pinned Mollusk. It reuses Task 2.32's fixture and independent literal state/CPI
oracles read-only. Neither the synthetic caller nor the restricted canonical
Token InitializeAccount3 artifact establishes deployed Squads/Token control.

Eight tests cover both fixed 35/34-account profiles at four fixed compute
boundaries: 200k before effects, first Token CPI, second Token CPI, and second
Token CPI with prefunding/paused initialization. Token limits are the retained
Task 2.32 successful entry consumption plus one CU; no adaptive search changes
limits to obtain a pass. Default 32-KiB heap and 4096-byte SBF frames remain.

Every failure must have its exact runtime error, specific CU-exhaustion log,
complete expected trace and full raw account state. Before Token failure, all
System allocations and original payer rent debit must already exist; initial
PIV state buffers remain zero, with zero or exactly one initialized Token vault.
Prefunded cases additionally require the exact 144-lamport sweep into PendingSol.
Mollusk's returned original vector is explicitly distinguished from staged raw
state and is not Bank/AccountsDB rollback evidence. Each failure is followed by
one fresh-runtime 1.4m-CU retry using its actual returned originals, asserting all
nine independent state envelopes, both Token layouts, rent, sweep and protection
of unrelated accounts. Eight tests comprise sixteen transaction-message cases.

Use `tools/validate_genesis_initialization_failure.py` from the repository.
Build only compiles a host executable; execution requires separate review and
its exact approved hash. Source/pin/tool/package/artifact guards, offline locked
Cargo, fresh private outputs and full failed-stage retention remain mandatory.
See `docs/TASK_2_33_GENESIS_INITIALIZATION_FAILURE_RUNTIME.md` for actual evidence.
No RPC, signing, wallet creation, deployment or funds are involved. No general
System-CPI failure, heap exhaustion, complete lifecycle or Testnet readiness
claim follows from these deliberately bounded resource cases.
