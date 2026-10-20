# Existing-ALT initializer — offline evidence checkpoint

As of 2026-10-21 Europe/Warsaw. D-048 conditional permission remains recorded;
**READINESS UNMET / NO LIVE TRIGGER / PUBLIC NOT READY**. The single
[M5 register](PIV1_M5_TESTNET_READINESS.md#finite-readiness-matrix) owns readiness.
This [public evidence index](PIV1_TESTNET_EXISTING_ALT_INITIALIZER.json) is not an
executable manifest or authorization.

The new `initializer-existing-alt-plan.cjs` wraps the original initializer manifest
and one raw public ALT snapshot. It preserves the native 315-byte initializer and
34/35 roles, then emits only Squads buffer creation/extension, stored transaction,
active proposal, four approvals and v0 execution. The old initializer is unchanged.
No ALT creation/extension or authority signature solely for reuse is emitted;
zero new ALT rent outflow neither refunds historical rent nor quotes complete fees
or funding. The old funding report is not used to charge ALT setup again.

The supported table is exactly 568 bytes: variant 1, active max-u64 deactivation,
present matching authority, valid start index/zero padding and 16 exact ordered
source-derived targets. Slots use full-width decimal strings/BigInt, avoiding the
SDK's numeric slot decoder. Supplied context equals the initializer reference and
is strictly later than last extension. Active-table reuse does not require its
historical creation slot to remain in current SlotHashes. Known malformed partial
inputs reject before INCOMPLETE; missing critical inputs keep `plan: null`.

Root's first suite had six passing groups and one fixture failure before the
planner call: the native literal fixture was invoked after its anchor became null.
The minimal test-only fix derives the reward target first, then requires the
planner's specific alias error. Production planner code is unchanged after that
run. V1 outputs/receipts remain separate; its suite did not pass.

Root's v2 suite passes seven groups plus standalone template/null commands;
the null example has 48 missing fields. Finite generated domains cover u64 slots,
roles, aliases and strict bytes without claiming exhaustive properties. Two final
synthetic CLI reports contain 18 complete packets, 38 zero signature slots and
1,447/1,480-byte compact messages. Shared execute is 963 bytes/two slots; distinct
execute is 1,092 bytes/three slots. Root and independent literal cursors verify
all bytes, PDAs, privileges, signatures and 16 writable lookup indexes. A separate
source-oracle estimate missed one extra index byte (996, not 995, for its conditional
two-signature distinct case); that metadata correction is not an application failure.

Source freezes, exact TAP commands, both test runs, report manifests/full public
wires and root/separate reviews are hash-bound in the JSON index. All 65,596 bytes
of the old README prefix are unchanged. There were 26 public source bindings before
documentation; package metadata now changes that one pin while the four selected
package sections remain exact. Final document freeze records this transition;
no tests are rerun to relabel the new package hash as pre-execution evidence.

Snapshots are unauthenticated and not current chain, warmup, rent-health, guardian
control, approvals, resources, fees or funding proof. The suite's retained D-047
ALT shape is historical, not refreshed. No real Clock anchor or artifact/bootstrap
trust choice is selected. No RPC, private access, key generation, signing, simulation,
submission, live funds/authority operation, build or Bank/SBF run occurred here.
Prior local loader/heartbeat success and invalid direct Deploy-CPI evidence remain
retained, not rerun or rewritten; no initializer runtime/full lifecycle follows.

Next prepare the exact governed initializer/trust/version/resource/funding package,
with actual sequential Squads indices and the real Clock literal frozen before
storing/votes. M4 stays OPEN_UNWAIVED and stopped drafts stay untouched; consumed
scopes are not renewed. Reviewed integration only; main 824342d/protected 3677fee,
reserve 2 GiB. No routine approval question or live trigger follows this checkpoint.
