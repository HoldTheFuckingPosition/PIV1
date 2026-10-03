# Task 2.47 — Protected withdrawal initiation and immediate deactivation

Status: **TECHNICALLY VALIDATED / PENDING FOUNDER ACCEPTANCE**, D-026/D-030 M2 with its required bounded M3 component.
Start integration/main: `1054ff3751ca4169eab7e46d77452634b11fc695` after D-033.
No new main integration, founder acceptance or live operation is inferred.

## Accepted contract and implementation boundary

Master specification sections 9.4–9.6, Phase 0 sections 9.6/10.1/11.2/12.1 and
existing `LegInitiationInput`/`initiate_withdrawal_leg` define the contract.
Implement one real production initiation for the existing bounded active-source
profile; preparation alone did not execute this lifecycle. Preserve exact
configured preference and fail-closed transient/reserve/removal/fallback limits.
No caller-chosen amount, minimum, capacity, destination or PDA bump is accepted.

Authenticate current pool/list/Clock, actual Mint supply, custody, source state,
runtime Stake minimum and current Rent. Compute maximum-safe remaining-target
fill; honor both stored/current technical floors and avoid a nonzero subminimum
remainder. Recompute conservative remaining-round output and residual proposed
HWM at the post-removal ratio. Existing fees, pending, KIF, carry, native Token
quarantine and prior leg liabilities remain protected.

Derive metadata/stake PDAs from the actual round sequence and next u64 index.
Unsolicited prefunds on unused System-owned empty nonexecutable PDAs move in full
to fixed PendingSol before allocation; full current rent then comes from the
operational vault above its permanent floor. Existing pending reconciliation
recognizes only this proven incoming surplus, preserving the active-round offset.
Never sweep already-owned/nonempty temporary accounts or claim a prior rent origin.

Build exact pinned SPL `WithdrawStakeWithSlippage`, with authenticated PivAuthority
as both token transfer signer and new stake authority. Both destination stake
authorities must match before direct Stake deactivation in the same transaction.
Record actual rent advances separately from Stake 5.1's pseudo rent metadata.
Check exact source/destination, pool/list/Mint/manager/custody changes after every
CPI and preserve every unrelated account byte/lamport/metadata field. No whole
validator-list clone/iteration is permitted. Only after all effects/checks succeed
may Config, the cumulative round and newly allocated leg commit together.

## Pinned instruction and account contract

The version-1 `PIV1IL01` ABI contains only one u32 validator-list index and is
exactly 13 bytes. Its 25-account profile uses the existing 19 bootstrap roles,
then Clock, Stake program, selected active stake source, pool withdrawal
authority, metadata PDA and destination Stake PDA. The 24-account profile omits
the distinct referrer role when it equals the manager fee account. All outer
roles must be unique. The pinned SPL instruction intentionally binds its two
authority metas to the same authenticated PivAuthority.

Retained SPL Stake Pool 2.0.3 `WithdrawStakeWithSlippage` is Borsh variant 24,
with 13 exact metas. The principal vault loses fee plus burned units; the manager
receives only the fee, while Mint and recorded pool supply lose only burned
units. The recorded pool total, source lamport balance and selected list
active-lamports field fall by the exact conservative redemption; the list account
lamport balance itself is preserved.
Stake 5.1's active partial split copies delegation fields, preserves source rent,
and gives the new account a delegated amount equal to the transferred lamports.
Both new stake authorities become PivAuthority. Direct Stake Deactivate uses
bincode variant 5; the runtime minimum query uses variant 13 and authenticates
the producer and exact eight-byte response. These are retained upstream source
facts, not an attestation of any deployed cluster binary.

Continuation reserves fee and conversion rounding for the maximum future leg
count permitted by the immutable snapshot input floor and remaining stored leg
slots. A larger current technical minimum cannot reduce this rounding reserve:
a later pool ratio may lower that minimum. Current post-withdrawal feasibility
is checked separately. HWM coverage excludes the whole fixed outgoing target,
pending donations and operational rent. All of these facts are rechecked from
fresh accounts before final persistence; they do not promise future liquidity.

## Review and evidence plan

One delegated writer and one separate reviewer; root owns shared docs, execution
and Git. Use retained pinned SPL 2.0.3 and Stake 5.1.0 sources verified in Task 2.46,
without installing/upgrading dependencies. Security-CPI guidance covers program
and return-origin pinning, canonical PDA signing and fresh post-CPI observations.
Use independent full-account/state and arithmetic boundary/property oracles,
including prefunds, per-CPI faults, replay, stale source, rent, partial assignment,
fee/rounding, floor/HWM and deactivation-authority failures. Host modeled effects
or discarded worlds cannot establish actual nested runtime or Bank rollback.

Evidence root: `/tmp/piv1-t247-pilot-review`. Takeover preserved the user's existing
AGENTS.md Solana-skills addition, 352 tracked-file hashes and 42 historical records.
Four recovery archives and all restorations/logs/artifacts remain protected.
Root froze 119 source inputs after separate source/test/README review. The first
focused execution passed all 10 tests (24 unfiltered numeric cases within the
numeric group); the first full execution passed 561 host tests, one doctest and
eight gates, without diagnostics. The first strict SBF gate rejected a
4160-byte `prepare` frame exceeding the 4096-byte limit by 64 bytes. Cargo exited
zero but emitted stack/call-frame diagnostics, so its artifact is rejected.
All first-attempt inputs, logs and the emitted ELF are retained (105 protected
files). The separately reviewed correction isolates staged-leg construction in a
non-inlined helper returning a bounded boxed value, dropped after prepared
writes. This does not establish runtime heap capacity or allocator reclamation. Exact transition, proof, CPI and commit ordering remain unchanged.
Only `withdrawal_leg_execution.rs` changes between the two 119-input freezes.
Reversing that narrow correction reproduces the original source hash exactly;
the original file bytes are also retained. No guard or resource limit is relaxed.

Before execution, separate review caught a missing test-root import for the
reused unchanged oracle; it was restored. Review also added a concrete minimum-
drift regression: target 7705, snapshot floor 101, maximum 76 legs and stored
native floor 7546. With current delegation minimum 200, fee 12/1000 and source
capacity 4000, the correct future-rounding reserve yields 7542 and rejects.
Incorrectly using the larger current minimum yields 7578 and would accept. This
is additional regression coverage, not a production economic-rule change.
No host test failed. The first SBF diagnostic rejection is retained explicitly;
it is not a successful compilation claim.

Host oracles enumerate bounded quote inputs independently and compare the whole
Config/round/leg and all account records. They cover both account profiles,
zero/nonzero fees, Mint direct-burn lag, zero/small/large prefunds, actual Rent
above pseudo rent, quarantine and pending-use offsets, two current sources,
maximum fill/no stranded remainder, stale identity/Clock/fees/minimum, pause,
replay, aliases, writable/borrow failures, query origin/length/mutation, checked
overflows, CPI no-ops and bad deltas. Every CPI fails both before and after its
modeled effects; exact raw prefixes are retained, Config/round remain uncommitted,
and explicitly discarded worlds allow retry. A late metadata borrow also proves
that no partial final state write occurs in this host model.

## Root execution commands and retained evidence

Commands run by root as jerem from the repository:

```text
python3 /tmp/piv1-t247-pilot-review/validate.py freeze
python3 /tmp/piv1-t247-pilot-review/validate.py focused
python3 /tmp/piv1-t247-pilot-review/validate.py final
python3 /tmp/piv1-t247-pilot-review/check_evidence.py host
python3 /tmp/piv1-t247-pilot-review/validate.py sbf
```

The reviewed driver pins the existing Rust 1.97.1 host toolchain, clean environment,
locked/offline single-job Cargo and unchanged helper hashes. Final gates comprise
workspace all-target tests, doctests, all-target check, all-features, no-entrypoint,
cpi, idl-build and warning-denied documentation. The SBF profile retains the
existing platform-tools v1.54, pinned tools/sources, diagnostic rejection, network
socket denial, 2-GiB disk reserve and existing time/memory guards. It does not
install dependencies or run a validator. Raw commands, environments, tool hashes,
stdout/stderr, preservation checks and result receipts are retained outside Git.

Initial-attempt evidence paths: `source-freeze.json`, `host-focused-a`, `host-final-a`,
`host-evidence.json` beneath `/tmp/piv1-t247-pilot-review`; SBF output is
`/tmp/piv1-bank-smoke-withdrawal-initiation-sbf-t247-a`. The final artifact/checkpoint
review binds the corrected inputs and retained first attempt. Earlier evidence is preserved evidence,
not a rerun of historical VM/Bank suites.

## Remaining scope and publication boundary

Next dependencies are finalization, settlement and pending integration, followed
by M4 complete real production lifecycle and M5 exact Testnet deployment package.
Stop before the first live deployment for the existing explicit approval gate.

## Corrected validation and integration checkpoint

The corrected source passed all 10 focused tests, 561 host tests, one doctest and
eight final gates, then strict production SBF compilation with zero diagnostics.
Root ran the same four driver phases using `validate-b.py`, followed by
`check_evidence-b.py host` and `check_evidence-b.py sbf`. Fresh b output paths
preserve all a evidence; guards, tools, dependency pins and final gate scope are
unchanged. No historical VM/Bank suite was rerun. Final source freeze SHA-256:
`7d36406a2e0ecfcf15c2c197225dc53e50eb37777383b3751f0ec1f7aa721f6f` (119 inputs).

Final ELF: `/tmp/piv1-bank-smoke-withdrawal-initiation-sbf-t247-b/artifacts/piv1.so`; 716768 bytes;
SHA-256 `224b4c40681d0e5635a01f836b499582b5c6cd4d9137f59f2cef8a23a918219c`.
The independent raw ELF check confirms ELF64 little endian, ET_DYN, machine 263,
flags zero and the same 15 imports as Task 2.46. All 58 final SBF logs and 18 final
host logs match their receipts. Build elapsed 294.583s;
sampled peak group RSS+swap 607825920 bytes;
sampled minimum free disk 30222467072 bytes. These are
static compilation observations, not runtime heap/CU or nested execution proof.

Changed files comprise seven shared guidance/report files, program README,
accounts/events/dispatch/ABI/persistence wiring, the shared active-source proof,
the protected adapter/handler and independent test/fixture. No manifests, lock,
payload layout, dependency versions or economics changed. The preexisting user
AGENTS Solana-skills addition is included unchanged. Root owns exact indexed-tree
review and normal integration-only commit/push; Git and `publication.json` record
the resulting identity and clean final refs. Main stays at 1054ff3; protected
Task 2.3 stays 3677fee. No live action, secrets/signing, deployment, fund movement,
key creation or authority transfer occurred. Technical validation is not founder
acceptance. Save this bounded checkpoint and stop; next resume from actual refs.
