# Task 2.14 — Runtime recognition of pending contributions

Status: **TECHNICALLY VALIDATED / PENDING FOUNDER ACCEPTANCE**.
Implementation: `7442cab7e97c422c7ee06290d5fc9d11c8b13ee6`.
Baseline: reviewed Task 2.13 implementation
`fd5735976eef1e2728ccf54726145501573db60d`, published closure
`fd48c3b1644faed6d30fdb92774c1bd8c03a2658`, on `integration/piv1-testnet`.
This is the next bounded dependency under D-026, not founder acceptance.

Separate reviewer `review_t23_final` passed scope SHA
`ffa4adf127e462a8531355e8e3b139e2913e2867d628de51be54e3ad10088234`
without required corrections. Writer `implement_t26_deposit` owns implementation
and focused locked/offline host validation. New target-build/runtime inputs must
be reviewed before execution. Scope PASS is not an implementation verdict.

## Requirement and authority

P-014–P-016, D-024 and master specification 7.7 permit recognition of
already-received SOL/JitoSOL in the pending ledgers during Idle, active
distribution, emergency pause and RecoveryRequired. Recognition cannot change
historical principal/HWM, cumulative contribution value, active obligations,
KIF, carry, recipients or eligibility. Economic integration remains later work.
The existing `state::reconcile_pending_contributions` implements the approved
phase-dependent physical-SOL offset and joint, idempotent pending recognition.
Task 2.9 provides canonical existing-account persistence; Task 2.13 establishes
the local SBF harness and claim regression baseline.

Implement an actual permissionless instruction for this narrow recognition only.
No transfer, deposit/staking conversion, initialization, sweep, HWM integration,
recovery transition or general all-vault normalization is added. This avoids
inventing a transfer-handler pause policy or initialization root of trust.

## Account and instruction boundary

Use trusted runtime Program ID and Rent. Authenticate exact Config and active
distribution envelopes, owner/PDA/bump/size/discriminator/version/zero-tail and
mutual state bindings. Authenticate the dedicated pending System SOL PDA and
165-byte legacy Token JitoSOL PDA using existing canonical validation: configured
mint/shared authority, initialized/unfrozen, no native token flag, delegate,
delegated amount or close authority. Protect both native rent floors.

The standalone instruction needs four fixed roles: writable Config, active
distribution, PendingSolVault and PendingJitoVault. The latter three are read-only
in the standalone message, but transaction-wide writable privilege union must not
invalidate a genuine prior donation. Require Config writable and reject aliases.
No guardian signature, caller-selected destination or caller-supplied observation,
amount, rent floor, pool price or backend is accepted. Keep the message fee payer
distinct in negative tests so compilation cannot repair a missing privilege.

Add a narrowly named pending-custody authentication result/observation. Reuse the
existing fixed-config, state and token/native validation, rather than duplicating
or weakening it. Existing full economic observation behavior stays intact. Native
excess on the pending token account remains separately visible, unclassified and
unchanged; it is neither JitoSOL nor newly recognized SOL. It must not prevent
this pending-only recognition. This does not solve extraction of that excess,
other-vault normalization or operational funding provenance, and makes no claim
about unrelated vault backing or official pool authenticity.

Use an exact discriminator-only, explicitly named pending-reconciliation ABI.
Reject malformed length/unknown selectors and wrong account counts. Preserve the
existing claim ABI, error precedence and host/runtime boundary. Reuse established
error codes; any genuinely necessary new code must be appended, never renumbered.
If an event is added, name it for pending recognition, report factual checked
deltas and emit only after successful persistence/postchecks. Do not advertise
the existing broad untracked-normalization marker as fully implemented.

## Atomic transition and independent oracles

Derive the actual pending observation and call the existing checked transition.
Preserve committed active-round pending SOL usage: physical pending custody is
recognized pending SOL minus that usage, not the gross ledger. A deficit in either
pending asset rejects before committing either ledger. Stage canonical Config
bytes with the existing persistence mechanism and preserve all other fields.
Only the two pending ledger fields may change. All native/token balances, complete
round bytes and every unrelated account field remain unchanged by this instruction.
Repeated observations are no-ops. No audit baseline is reset after recognition.

Require focused host/account/dispatch regression coverage for:

- SOL only, JitoSOL only, joint recognition, no-op and repeated no-op;
- paused, active and recovery states, including nonzero committed pending-SOL
  offsets before/after settlement, without touching fixed obligations or HWM;
- either-asset deficit, rent deficit, checked overflow and combined rejection;
- malformed/forged owner/PDA/bump/layout/version/tail/token state, aliases,
  missing writable Config, ABI/count failures and borrow rejection;
- one-lamport and larger token-account native excess preserved separately;
- exact two-field Config changes, unchanged custody/round/unrelated state and
  original flow/audit equations across sequential observations.

Synthetic initial custody is explicit. Add local SBF evidence for actual System
donation followed by recognition and idempotence, including a shared message
where appropriate. Token units may be preloaded in an authenticated synthetic
fixture; do not claim an actual SPL Token transfer from that fixture setup.
Use complete durable account records and passive observations, not truncated
Account Debug. Prove ordinary 200000-CU execution without changing runtime limits.
Failure output discard remains distinct from Bank rollback/signature evidence.
Retain and run the existing 19 claim/runtime tests on the new program artifact.

## Execution and completion gates

One delegated writer owns implementation/tests and its report appendix; root owns
shared documents/Git. A separate reviewer checks scope, exact source changes and
actual final evidence. Reuse existing pinned dependencies/tools and reviewed local
runtime facilities; do not clone large harnesses or repeat unchanged package review.
No new dependency or tool installation is authorized by this scope.

Production source changes require appropriate focused tests, one final workspace
validation pass, a newly frozen keyless SBF build and local runtime tests on that
exact new artifact. Task 2.12/2.13 artifacts/results remain immutable historical
evidence. Update existing source/artifact pin companions explicitly for the new
task; review changed runner/harness inputs and exact commands before execution.
Use one compiler slot, locked/offline commands and fresh evidence directories.
Existing format/Clippy absence remains explicit; never suppress a new diagnostic
or weaken an oracle to obtain PASS. Record discovered incompatibilities before
making compatible corrections.

After separate final review, checkpoint and normally commit/publish the bounded
result before later work. Keep main founder-accepted. No keys, signing, secrets,
validator, deployment, public-chain operation, real funds or authority transfer.
Initialization authorization, operational provenance, token-native extraction,
fee/rounding-loss treatment, real SPL/Jito mapping and other handlers remain open
delivery dependencies. No new economic policy is established here.

## Writer implementation and preparation handoff

Verified `jerem`, `integration/piv1-testnet`, baseline HEAD
`fd48c3b1644faed6d30fdb92774c1bd8c03a2658`; pilot-owned documents were preserved.
Implemented four-role pending authentication, exact eight-byte
`reconcile_pending_contributions` ABI (`e1801d669d18acce`), and shared dispatch.
The existing checked transition supplies the two pending field updates. Canonical
Config persistence is the final fallible operation; no CPI, event, transfer or
fallible post-copy work was added. Token-native excess remains separately visible,
unclassified and untouched. Claim routing/errors and full economic authentication
behavior remain intact; no new error code, layout, dependency or authority exists.

Production files: new `pending_accounts.rs`, `pending_reconciliation.rs` and
`instructions/reconcile_pending.rs`; narrow helper visibility in `accounts.rs`,
dispatch/module wiring and a separate source-only pending allocation inventory.
The inventory requests 5452 bytes, conservatively 5558 with listed padding/cursor;
it neither changes the 32-KiB heap nor proves total runtime use. New focused tests
and reusable `tests/support/pending_custody.rs` preserve complete account/round
state and original synthetic funding flows. Genuine World phase imports are
supplemental compatibility evidence, not continuous cross-World custody proof.

Focused command (direct installed Rust/Cargo 1.97.1, locked/offline, one job):

```text
/home/jerem/.rustup/toolchains/1.97.1-x86_64-unknown-linux-gnu/bin/cargo test --locked --offline --jobs 1 -p piv1 --test pending_reconciliation --test account_authentication --test kif_claim_instruction --test state_persistence
```

The exact clean environment/argv and raw outputs are saved in
`/tmp/piv1-t214-focused-host-20260909-{a,b}`. First run: 70 tests passed, but seven
unique dead-code warnings appeared in three unrelated targets because fixture
wiring was too broad. Narrow imports corrected them without suppression. Added
cross-state sequence/HWM/snapshot rejection coverage; final run: **71 PASS**
(23 fixed-account, 17 claim-instruction, 10 pending, 21 persistence), no warnings
or errors. Result-b SHA-256
`1ac9647a3a8b9d3a62c557de46bc344e4ac7570a0b40a75d33f2cd9d0a6c2053`.
Only one module-description comment changed afterward. The ten-file subset was
frozen and independently reviewed PASS, as relayed by the pilot. Its manifest is
`/tmp/piv1-t214-preparation-20260909-a/production-host-freeze.json`, SHA-256
`a26ab8721750571589a26e35d7e5c15a90070ed3dd28fb23134bc80911c0ca01`.

The same isolated harness retains all nineteen prior tests and adds five prepared
pending tests: actual System donation then recognition/repeat, paused token-native
excess, joint/rent deficits, ABI/alias/write privileges, and successful recognition
followed by a failed instruction with raw-state versus returned-discard evidence.
All complete account logging/oracles remain. **24 runtime tests are prepared,
not executed.** No token transfer or initialization is inferred from setup.
The harness runner now takes path/hash/size from its reviewed artifact pin, passes
that identity through its clean environment, and still protects the historical
Task 2.12 artifact. The pilot independently ran the seven stdlib runner refusal
tests: PASS (`/tmp/piv1-t214-pilot-runner-tests-20260909-a`); the writer did not
repeat them. Tool/library/package/feature pins are unchanged.

The full nineteen-file packet is
`/tmp/piv1-t214-preparation-20260909-a/preparation-freeze.json`, SHA-256
`91235fadfa511aadcdd507e0c4261d1535780bb0683897e157a51820ad8cb262`.
It includes exact production/test/harness/runner/pin hashes. Target pins change
only the baseline and source hashes; harness pins change baseline/source hashes
and add the explicit new artifact identity. Its hash/size are intentionally null
until the newly reviewed target build produces an artifact; harness preflight
rejects that incomplete identity. No new artifact or harness execution is claimed.

The unchanged target runner's default preflight executed once:

```text
/usr/bin/python3 -I /home/jerem/piv1/tools/build_keyless_sbf.py --output /tmp/piv1-keyless-sbf-preflight-t214-20260909-a
```

Actual **PREFLIGHT_PASS_NO_TARGET_BUILD**: all 26 commands returned zero; source,
tool, configuration, output and protected-ref preservation checks passed. Full
result SHA-256 `f5f40c752b4c653cb880728705d67fcdf5066caf7a55a3fc7425b6431a169fa5`.
Exact proposed target build, pending separate packet review and pilot release:

```text
/usr/bin/python3 -I /home/jerem/piv1/tools/build_keyless_sbf.py --output /tmp/piv1-keyless-sbf-build-t214-20260909-a --execute --approved-runner-sha256 e4455e55a88fe5b22796cc21ef1a938522a5cf55cc226bee3570df9756bf3cb8 --approved-pins-sha256 b88ea4ffebe97230dce61172e688967d63f7e55aa1884a14c994c2ff86fbefee
```

Source/report ownership is frozen and released. The writer holds no compiler
slot; the pilot took it for final workspace gates. New SBF compilation/runtime,
actual artifact identity binding and final evidence review remain pending. The
format/Clippy components remain absent; nothing was installed. Existing Task
2.12/2.13 artifacts and failure/result directories are untouched. No Git mutation,
commit, push, keys, signing, validator, deployment, network blockchain operation,
real fund movement or authority transfer occurred. Initialization, operational
provenance, token-native extraction, staking fee/rounding policy and other handlers
remain separate delivery boundaries; founder acceptance is still pending.

## T214-R1 — prepared runtime fixture Rent comparison

Before any target/harness execution, review identified full Rent equality in
`tests/pending.rs`: old defaults use 3480/2.0 while the runtime bridge carries
6960/1.0. The structures differ although their actual minimum balances agree.
This was a prepared, unexecuted fixture defect; no runtime failure is claimed.

The assertion now compares minimum balances for empty data, Config,
ActiveDistribution and the 165-byte Token allocation against both bridged and
actual runtime Rent. Runtime defaults, helper behavior, funding baseline and
complete-account oracles are unchanged. Only two frozen inputs changed:

- `validation/sbf-claims/tests/pending.rs`:
  `1f4833fdfce4309ded12232d1fc444a8499bbdfc817dc893e4f675f2dcbd04fd`.
- `tools/sbf_claims_pins.json` (that source hash only):
  `66ee2291f5e0649883d41ce86a9ac7adcaf699f8a7c9a2c3c3210719264aa251`.

Corrected nineteen-file freeze:
`/tmp/piv1-t214-rent-fixture-correction-20260909-a/preparation-freeze.json`,
SHA-256 `5025f1c4f60133ad322cbe4d8c0047217951392aa05bdeb28f44d51168a04402`.
The prior freeze, saved before-file and focused diff remain available. All ten
production/host inputs and target pins `b88ea4ff...` were verified unchanged;
the proposed target command is unchanged. No compilation, runtime, new preflight,
dependency/helper/tool change or Git mutation occurred. Correction/report ownership
is released for review; the pilot retains its slot.

## Pilot verification and target release

Separate production/host-source review passed the ten-file freeze without findings.
Root independently verified its hashes and the writer's actual 71-test logs.
Root then executed one complete locked/offline, one-job workspace validation pass:
**349 host tests +1 doctest, zero failures/ignored tests, all eight gates PASS**.
The gates cover all-target tests, doctests, all-target checking, all-feature checking,
individual `no-entrypoint`/`cpi`/`idl-build` checks and warning-denying documentation.
Direct installed Cargo/Rust 1.97.1 ran in a clean environment; tool hashes matched
the existing reviewed pins. Sources were unchanged throughout, and root checked
all 16 log hashes/sizes and zero diagnostics. Exact argv/environment/results are
in `/tmp/piv1-t214-pilot-host-20260909-a`; verified summary SHA-256
`b6a91b01af6de920b7b69044ba7e8d6aa2aca847cd71924770993c3e37d37a59`.
The separate reviewer inspected these saved results without rerunning the gates.

Root and reviewer independently checked target preflight's 26 zero-exit commands,
52 log hashes/sizes and source/Git preservation. Only the documented source and
baseline pin entries changed; dependency/tool/feature entries are identical.
Separate review passed the exact proposed target build; root started it in the
fresh output above with `env -i PATH=/usr/bin:/bin LC_ALL=C` around that command.
At that checkpoint no target result was claimed. T214-R1 subsequently received
corrected harness-source PASS on the nineteen-file correction freeze. Artifact
identity binding, host harness compilation and runtime execution then proceeded
as recorded below. Neither source review nor host success is founder acceptance.

## Actual target and local runtime evidence

All executions in this section belong to root. The separate reviewer inspected
the saved target/command evidence before harness compilation; no production source or
production dependency changed after the final workspace gates.

| Stage | Evidence directory under `/tmp` | Actual result |
| --- | --- | --- |
| New target build | `piv1-keyless-sbf-build-t214-20260909-a` | STATIC_BUILD_PASS; 304.424 seconds; 29 zero-exit commands, 58 verified log hashes/sizes, no diagnostics |
| Native harness build | `piv1-sbf-claims-build-t214-20260909-a` | BUILD_PASS; 178.910 seconds; 12 zero-exit commands, 24 verified log hashes, no diagnostics |
| Exact executable run | `piv1-sbf-claims-run-t214-20260909-a` | RUNTIME_TESTS_PASS; 24 passed, zero failed/ignored/filtered; 3.198 seconds, 20:31:25–20:31:28 UTC |

Every stage passed source/tool/artifact/Git preservation checks. New target ELF:
`target/sbpf-solana-solana/release/piv1.so` under the target evidence directory,
229208 bytes, SHA-256
`46fd815847c236fb53ed5dc5ace79c48c5a21beac4019ffa107b80a5be69812f`.
Root and reviewer checked ELF64/little-endian/machine 263/flags 0. Reviewer matched
20988 disassembly instructions to artifact bytes; all 4808 direct `r10` references
are within -4096 through -1. These are limited static checks, not a complete
stack/heap proof. Root bound only the verified artifact hash/size into harness
pins. Final nineteen-file freeze:
`/tmp/piv1-t214-artifact-binding-20260909-a/preparation-freeze.json`, SHA-256
`2225ec779d349c04f2272f0346deeacce77a6fe0dd15a4d7041e369864b80826`.

The exact host executable is `target/debug/deps/claims-c12d45bbd63e9bf4` under the
harness build directory: 96996056 bytes, SHA-256
`587fc09cbdbdae17a236473f6a978ad4158d71cbc4dbf7c49ec302dc4d2c750a`.
Root verified its ELF64/x86-64 header, interpreter, absence of RPATH/RUNPATH and
all four actually resolved libraries against unchanged pins before execution.
The native loader's library-list output includes the interpreter itself as the
fourth resolved entry. Inspection record:
`/tmp/piv1-t214-pilot-host-executable-20260909-a.json`.

Exact harness commands, each prefixed with
`env -i PATH=/usr/bin:/bin LC_ALL=C /usr/bin/python3 -I`:

```text
/home/jerem/piv1/tools/validate_sbf_claims.py --output /tmp/piv1-sbf-claims-build-t214-20260909-a --stage build --approved-runner-sha256 ff4706b6101b57c6385b0ae43b175a3ed252cb40e150f7a3120a84beb7c01fc8 --approved-pins-sha256 bab50a1064078e6a02bec8eaac2ebc64b7cf28c49cd3748ffe2b7df88ad5518c
/home/jerem/piv1/tools/validate_sbf_claims.py --output /tmp/piv1-sbf-claims-run-t214-20260909-a --stage run --approved-runner-sha256 ff4706b6101b57c6385b0ae43b175a3ed252cb40e150f7a3120a84beb7c01fc8 --approved-pins-sha256 bab50a1064078e6a02bec8eaac2ebc64b7cf28c49cd3748ffe2b7df88ad5518c --build-output /tmp/piv1-sbf-claims-build-t214-20260909-a --approved-executable-sha256 587fc09cbdbdae17a236473f6a978ad4158d71cbc4dbf7c49ec302dc4d2c750a
```

Runtime's 12 commands and 24 log hashes passed; result SHA-256
`d0b8854f68af9cc610b5c6970c8a4a9d7be593c6efedc2f7ab229475fbf5542e`.
Its complete stdout SHA-256 is
`996355a939025eaec923058311ba16f3863675463e423534fe64fb2e76e0a6c7`.
The nineteen prior tests retained their sixty claim cases on the new artifact.
Five added tests produced ten pending cases:

- A real top-level System donation of 100 lamports followed by recognition and
  idempotent repetition succeeds in one message at **151832/200000 CU**. Donor and
  pending native changes are exact; only the two Config pending fields change.
- Paused recognition with one-lamport or one-million-lamport token-native excess
  succeeds at **75844 CU**. The excess remains untouched and unclassified;
  synthetic 20 token units are recognized without creating pending SOL.
- Either-asset deficits, token rent deficit, malformed ABI, aliased roles and
  missing writable Config return the expected errors with all raw/returned
  account state unchanged. The fee payer is distinct from Config.
- Successful recognition followed by malformed data and an unreachable third
  instruction consumes **77176 CU**. Both raw contexts retain the first token
  ledger update; returned originals show Mollusk output discard, not Bank rollback.

Root independently reconstructed the accepted Borsh field offsets and compared
all **1629 complete account records across 70 cases**, including unchanged bytes,
owners, native balances, metadata and original funding flows. The read-only
checker `/tmp/piv1-t214-pilot-account-evidence.py` did not execute any program;
its output `/tmp/piv1-t214-pilot-account-observations-20260909-a.json` has SHA-256
`c0f33604b012764f1b3c0ea01deb78c34f0ba2126a89e5591dfde8d8fb94474e`.
Ordinary KIF claim now uses 73837 CU; prior claim/account/error/event protections
and reduced-compute pre-effect failures remain covered.

Active/settled/recovery offsets have focused host and pure-model evidence; these
new runtime fixtures exercise Idle and pause. Token units and initial owned state
are synthetic; no actual SPL Token transfer, initialization/earning authority,
global historical KIF liability proof, Bank/AccountsDB rollback, signatures,
deployment verifier or public-cluster behavior is established. Operational funding
provenance, native-token excess extraction, fees/rounding losses, real SPL/Jito
mapping and remaining handlers remain separate dependencies.

Two untracked Python bytecode files for the validation tools were preserved in
`/tmp/piv1-t214-preserved-bytecode-20260909-a` with a hash manifest instead of
publishing generated files. No source work was discarded. No Mainnet action,
deployment, real-fund movement, wallet/key creation, signing or authority transfer
occurred. AI-assisted review is not a professional independent audit.

## Final technical disposition

Separate reviewer `review_t23_final` returned final bounded technical PASS without
remaining findings on report SHA-256
`62605db97ad00a4294313067546395d6e071f8e9cc6238d30f575110ca182f1e`
before this status append. The reviewer independently inspected the actual final
source/artifacts/logs and complete account effects; executions retain the
writer/root attribution above. Task 2.14 is technically validated within this
scope, pending founder acceptance. Root is completing normal development
publication; implementation identity and Git receipt follow in the checkpoint.

## Reviewed implementation and publication checkpoint

Implementation `7442cab7e97c422c7ee06290d5fc9d11c8b13ee6` contains exactly the eighteen reviewed
source/tool changes: eight production files, two host-test/fixture files, four
tool/pin files and four isolated-runtime harness/document files. The unchanged
target runner remains part of the nineteen-file validation freeze. Root verified
all hashes and the exact staged set before committing. The accompanying five
documents are this report, AGENTS, execution plan, live checkpoint and evidence
checklist; separate closure review passed without corrections.

Targeted checks found no credential-bearing additions or compiled/key files among
the 23 publication candidates. No custom Git hooks or CI automation were present.
Remote integration, accepted main and Task 2.3 refs matched the recorded baseline.
Root is preparing the normal fast-forward publication of implementation and
documentation closure to the existing development branch. Main remains unchanged.
The next task must begin from a verified publication receipt and this checkpoint;
no later task is started or founder acceptance inferred here.
