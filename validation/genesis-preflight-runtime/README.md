# Keyless genesis recipient-preflight runtime validation

This isolated host harness loads the two exact Task 2.29 SBF probe artifacts.
The caller is synthetic code under the canonical Squads identity, not the Squads
executable or evidence of actual governance control. The callee executes the
unchanged public read-only recipient preflight; it has no initializer and returns
no preflight facts. No production code or existing validation workspace changes.

Run only through the reviewed `tools/validate_genesis_preflight.py` build/run
stages. Both artifacts, the new test executable, source/lock/package/tool inputs
and complete account evidence are bound by hashes. Instructions is generated
by Mollusk from the actual outer message; Clock and Rent use its runtime sysvars.
Inputs, raw transaction contexts and returned accounts must remain byte-identical.
Synthetic signer privileges are not signatures or a signed cluster transaction.
Read-only account preservation does not establish mutating initializer rollback.

The profile retains the pinned default 32 KiB heap and explicit 1,400,000 compute
unit ceiling. Measured use is an observation for these fixtures, not proof of
complete genesis resource sufficiency or a production transaction budget.
