# Task 2.30 — Keyless local genesis recipient-preflight runtime validation

Status: **TECHNICALLY VALIDATED / PENDING FOUNDER ACCEPTANCE** under D-026,
within the bounded read-only runtime scope below. Baseline integration:
`352fe7d4ecd8609d93cf2b0a2a96009018d3a7de`; main remains
`7b74be4b13c019b96a0c8abcbebfbcc361d31089`.

## Scope and invariant

This task executes one isolated host test binary against the exact
two Task 2.29 SBF probes. Neither probe nor production code is rebuilt or changed.
The synthetic caller uses the canonical Squads identity and real `invoke_signed`
CPI; it is **not the actual Squads executable**. The callee reaches the unchanged
public production read-only recipient preflight at height two and returns no
serialized preflight facts. This is not initializer readiness, mutating rollback,
exclusive recipient control or actual governance execution evidence.

The fixed topology remains 32/31 accounts with recipient indices 0/255. A new
independent fixture derives all sixteen target PDAs from literal seeds, serializes
the Jito accounts through existing pinned test oracle types and reuses the existing
complete Squads transaction serializer. It never calls a host-context preflight or
production model preparation to obtain expected success. The old fixture's host
Instructions account is omitted; Mollusk generates Instructions from the actual
outer transaction. Clock and Rent use runtime sysvars. The deliberate substituted
Instructions-key negative is an invalid account, not claimed generated evidence.

The twelve cases cover both successful receiver profiles, insufficient approvals,
stale approval, unreleased runtime timelock, recipient owner/funding/canonical PDA,
invalid mint decimals, wrong Instructions identity, direct height-one callee
rejection, and wrong outer discriminator rejected before CPI. Complete supplied,
raw-before, raw-after and returned account bytes are recorded. Every raw account,
including generated Instructions and loaded program metadata, must be preserved.
The actual instruction trace checks all outer union privileges, all inner exact
privileges and bytes, height two, readonly inner proposal and PDA-signed inner
vault. The returned inner-instruction record is independently checked against the
compiled message. No return data or System/Token CPI is expected.

The pinned default 32 KiB heap and an explicit 1,400,000 CU ceiling are retained.
Observed CU values apply to these fixtures only; they are not a production budget,
minimum-heap measurement or complete initialization resource proof. Signers are
synthetic message privileges; no key, signature, Bank or AccountsDB exists here.

## Standalone dependency and execution boundary

The workspace uses Mollusk 0.15.1 / Agave 4.2 and the existing claims cache, with
no new registry identity. Its 389 registry name/version/source/checksum identities
match the unchanged claims lock exactly. Its 392 total packages include only the
three local packages PIV1, piv1-math and this nonpublishable harness. Existing
Borsh 1.8.0 and stake-interface 1.2.1 are direct test-fixture serializer edges,
not new dependency versions. The sole offline metadata preparation succeeded in
**0.476770 seconds** at `/tmp/piv1-t230-writer-preparation-20260925-a`.
No test or SBF execution is implied by metadata success. The first pin-generation
reader compared historical package list ordering directly; sorting both lists by
name/version fixed that preparation-only assertion. The exact 389 identity set
never changed; no compilation or test failure is attributed to that reader.

At takeover one installed OS library differed from historical tool pins. Root
verified Ubuntu-signed September 22 snapshot metadata through Packages to the
exact installed libexpat package payload. Only the new companion binds the current
`/usr/lib/x86_64-linux-gnu/libexpat.so.1.9.1` hash
`ec6c12d33bb8f9d0e90804121adf19930f36b1b2a4aeb6e1a454b89c7a50c801`
(186624 bytes). Historical pins remain untouched; no package was installed.
Receipt: `/tmp/piv1-t230-pilot-review/system-provenance.json`, SHA-256
`95fbf89b65538bc556b37d0331adb8766a60c0f1ef7aaeaabccf617a6be5b731`.


The narrow runner verifies its own approved hash and pin companion, checks the
unchanged claims helper hash before importing it, and never overrides its globals.
It binds the new source/lock/resolved graph, protected historical source/tool inputs,
all package archives and extracted source bytes, both exact probe ELFs and prior
artifacts. Every command has an explicit new-workspace or repository cwd. Build
only invokes pinned locked/offline host Cargo with one job and no debug/incremental
cache to conserve disk. Run accepts only the exact separately reviewed successful
build executable. Both stages use fresh private task-specific `/tmp` outputs and
retain full commands, stdout/stderr hashes, preservation and independent failure
records. No install, target build, validator, RPC, wallet or network stage exists.

## Evidence and review

Separate source/runner review passed after pre-execution oracle tightening and
post-run executable-preservation guards were added. Root executed the eleven
mocked runner tests successfully, exit zero in **0.117030 seconds**. Receipt:
`/tmp/piv1-t230-pilot-review/runner-tests.json`. The writer performed no tests,
compilation or SBF execution. Root owns every compiler/runtime execution and final
publication.

The first root host build at
`/tmp/piv1-genesis-runtime-build-t230-20260925-a` failed before test execution:
Cargo exit **101** after **178.363117 seconds**, with two E0433 diagnostics from
the reused oracle's `crate::integrations` references. The unchanged production
test normally supplies that crate-root import. A minimal new-harness correction
re-exports `piv1::integrations` at its test-crate root; the protected oracle,
dependencies and runtime assertions are untouched. Only the runtime source hash
changes in the new pin companion. All initial inputs, logs and rejected build
outputs remain preserved; no successful test or SBF run is attributed to it.
Separate minimal-correction and retry-command review passed in
`/tmp/piv1-t230-reviewer/correction-build-review.json`. The unchanged runner
SHA-256 is `48881f609bab92ca2bda9bca756610dbf3ca8e79f4b55770d381efa0b820b4ab`;
final pin SHA-256 is
`8388126eff1bde34290500e563a6c12ce69238637ccbc620cc197f69c480226a`.

The fresh second build at
`/tmp/piv1-genesis-runtime-build-t230-20260925-b` passed in
**180.714091 seconds**, with Cargo exit zero and no warnings or errors.
Root verified all 24 logs from its twelve commands and the exact host ELF:
22982472 bytes, SHA-256
`72e8f0609018f94b9eb4b9dbcdfc04c50d944ceff7ea1b9bcb5e9954d99d2ca0`.
Build result SHA-256:
`8a791384a3d26fc617769773c34aada58a3cc50c8de59d0f13f8e67d23309931`.
Root's `host-build-evidence.json` records static host inspection; a separate
exact-binary/run-command review passed in
`/tmp/piv1-t230-reviewer/runtime-command-review.json` before any SBF execution.

The first actual runtime execution at
`/tmp/piv1-genesis-runtime-run-t230-20260925-a` passed **twelve tests / twelve
cases**, with zero failures or ignored tests, in **0.721794 seconds**. Both
32/31-account successes reached the actual height-two SBF callee. The ten negative
cases returned their precise expected errors, including the distinct caller
discriminator error and direct height-one refusal. No runtime retry or assertion
weakening was needed. Root independently decoded full raw logs, actual trace
privileges, Instructions bytes and Clock/Rent observations; **1674 complete
account records** matched their before/after invariants. This is a second reading
of the same execution evidence, not a second runtime execution.

The two successful cases consumed **288277 CU** (distinct manager/referrer) and
**282070 CU** (shared manager/referrer). Both exceed 200000 CU: this task makes
**no 200000-CU viability claim**. Both run under the explicit 1400000-CU ceiling
and default 32768-byte heap. Beneficiary recipients remain distinct in both
profiles. No initializer or real Squads executor resource claim follows.

Runtime result SHA-256:
`7dba3508ce7bbad6bb05cfc0e65a162d8cd23e239888b6a1fe17894ce879c138`;
raw stdout SHA-256:
`929f7600f7325539aee148fc168d7f0636f9ae8f971287f956940401a45d5f7f`.
Root's independent decoding receipt is
`/tmp/piv1-t230-pilot-review/runtime-evidence.json`. All 24 run-stage logs,
seven new sources, 119 protected sources, tool/package inputs and both exact
probe artifacts match. The exact host binary hash also matched after execution.
Separate runtime evidence review passed in
`/tmp/piv1-t230-reviewer/runtime-evidence-review.json`: the reviewer independently
decoded all 1674 records, exact generated Instructions serialization and complete
trace privileges from the same retained execution without rerunning it. The task
report matches the actual logs. Root retains final shared-document/publication
review and Git ownership.

Root verified 128 retained earlier logs, 64 probe target logs and eight probe
host logs in `retained-evidence.json`, `retained-probe-target.json` and
`retained-probe-host.json` under its review directory. The prior **469 production
host tests +1 doctest/eight gates, 24 claim/pending SBF tests /70 cases, 15 Node
tests and eight old plus sixteen recipient transport cases** remain retained
evidence; none was rerun here. Task 2.29's ten boundary tests and static probe
compilation also remain retained evidence. This task adds the new root executions
of eleven mocked runner tests and twelve actual SBF probe-runtime tests.

Exact retained probe identities:

- Caller: 64368 bytes, SHA-256 `938e6c1c63554ac26f75f3c4daef614087051ea88d8f9eb130692940278508b2`.
- Callee: 180232 bytes, SHA-256 `1d87ab760fae785cc74a8ef069722ee5bb3c48ad43e029118eba512cd1c97c53`.

The exact first and corrected build commands were:

```text
/usr/bin/python3 -I -B /home/jerem/piv1/tools/validate_genesis_preflight.py --stage build --output /tmp/piv1-genesis-runtime-build-t230-20260925-a --approved-runner-sha256 48881f609bab92ca2bda9bca756610dbf3ca8e79f4b55770d381efa0b820b4ab --approved-pins-sha256 7eb9832e9c7e8b1782fb7d2fda9d372a429cab9b8cbe2bafd2a3e7adf2000a75
/usr/bin/python3 -I -B /home/jerem/piv1/tools/validate_genesis_preflight.py --stage build --output /tmp/piv1-genesis-runtime-build-t230-20260925-b --approved-runner-sha256 48881f609bab92ca2bda9bca756610dbf3ca8e79f4b55770d381efa0b820b4ab --approved-pins-sha256 8388126eff1bde34290500e563a6c12ce69238637ccbc620cc197f69c480226a
```

Each build internally invoked pinned Cargo 1.97.1:

```text
/home/jerem/.rustup/toolchains/1.97.1-x86_64-unknown-linux-gnu/bin/cargo test --manifest-path /home/jerem/piv1/validation/genesis-preflight-runtime/Cargo.toml --locked --offline --jobs 1 --test runtime --no-run --message-format=json
```

The separately reviewed run command was:

```text
/usr/bin/python3 -I -B /home/jerem/piv1/tools/validate_genesis_preflight.py --stage run --output /tmp/piv1-genesis-runtime-run-t230-20260925-a --build-output /tmp/piv1-genesis-runtime-build-t230-20260925-b --approved-executable-sha256 72e8f0609018f94b9eb4b9dbcdfc04c50d944ceff7ea1b9bcb5e9954d99d2ca0 --approved-runner-sha256 48881f609bab92ca2bda9bca756610dbf3ca8e79f4b55770d381efa0b820b4ab --approved-pins-sha256 8388126eff1bde34290500e563a6c12ce69238637ccbc620cc197f69c480226a
```

It executed only:

```text
/tmp/piv1-genesis-runtime-build-t230-20260925-b/target/debug/deps/runtime-93b79fec5778ebcd --test-threads=1 --nocapture
```

The full bounded environment and command/log hashes are in each stage's result.
Only the new validation workspace, its three tools/pins/tests files and this
report were changed by the delegated writer. Root maintains the six shared
project documents and owns Git. The task commit is recorded in Git after final
review; no founder acceptance or main integration is implied.

## Deferred work and operations

Full initialization, total genesis heap/compute/rollback, operational funding
provenance, later Token-owned native donations, actual Squads governance and
recipient control, real ALT/buffer lifecycle, signed cluster execution and founder
Testnet handover remain deferred. No economic rule, native ABI or production
account bytes change. Technical validation is not founder acceptance and AI-assisted
review is not a professional independent audit. No Mainnet operation, deployment,
fund movement, key creation/signing, secrets access or authority transfer is allowed
or performed by this task. Root publishes integration only after final review,
checkpoints and stops; Task 2.31 is NOT STARTED.
