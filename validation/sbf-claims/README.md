# Isolated keyless SBF claim and pending-recognition validation

This unpublished workspace validates the explicitly reviewed artifact in the pin companion in Mollusk
0.15.1 / Agave 4.2.0 / SBPF 0.21.1. It has its own manifest and lock. It does not
rebuild the SBF program, change the production graph, or add a native PIV processor.

Preparation does not authorize compilation or execution. The exact source,
registry archives, selected features, compiler/native inputs and commands require
the Task 2.13 separate review and pilot release. Use the bounded runner in
`../../tools/validate_sbf_claims.py`; its default is preflight. The `build` stage
uses locked/offline Cargo with one job and `--no-run`. The `run` stage requires
the separately identified test executable's reviewed SHA-256 and invokes that
executable directly. Every stage needs fresh private output under `/tmp`.

The source of execution is the exact ELF path/hash/size in
`tools/sbf_claims_pins.json`; the runner supplies that identity to the binary.
Task 2.12/2.13 artifacts remain unchanged historical evidence. A new artifact
requires separate target-build and harness execution review.
The older host fixture is imported read-only for state construction. A fresh
runtime audit includes its initial synthetic funding once; accepted claims never
reset that original audit. Account comparisons include complete bytes, native
balances, owners, executable flags and rent epochs. No native host claim method
is called to produce runtime effects or expected transitions.

The required success allowance is 200000 CU. A separate case records Mollusk's
1400000-CU default. Every budget is selected before constructing its program cache
and loading the ELF. Default heap, frame/depth, features and System implementation
remain intact. Rent compatibility is asserted through the serialized ABI.

The explicit fixture payer and guardian signer flags are synthetic message
privileges, not signatures. The runtime checks PIV's actual signed System CPI;
inner traces prove the instruction/endpoints/amount, not the raw signer seed bytes.
Passive callbacks copy actual context accounts and logs without modifying runtime
state. A failed message's returned original accounts demonstrate Mollusk's output
discard. Raw earlier effects and events can still appear in the recorded context
and logs. This is not Bank/AccountsDB rollback, signature verification, deployment
verification, public-cluster configuration, or continuous custody from an older
host World. Consumers of events must require overall transaction success.

Public registry source inspection includes build scripts and their helper/native
sources. Compiling existing cryptographic library definitions does not create
wallet keys. Internal ephemeral VM/JIT hardening randomness remains enabled.
No wallet, signing, validator, network blockchain operation or deployment is part
of this harness. See `../../docs/TASK_2_13_KEYLESS_SBF_CLAIM_EXECUTION.md` for the
reviewed contract and attributed evidence.

Task 2.14 adds pending recognition in the same test binary while retaining all
nineteen claim/evidence tests. The pending tests use an actual System donation
and synthetic preloaded token units; they do not claim SPL Token transfer,
initialization or general economic normalization. Token-account native excess
is retained separately without becoming principal or pending SOL.
