"""Reject incomplete lifecycle evidence and altered executable inputs."""
import importlib.util
import json
from pathlib import Path
import tempfile
import unittest


SPEC = importlib.util.spec_from_file_location(
    'lifecycle_runner', Path(__file__).with_name('validate_production_lifecycle.py'))
RUNNER = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(RUNNER)


def module(name):
    spec = importlib.util.spec_from_file_location(name, Path(__file__).with_name(name + '.py'))
    loaded = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(loaded)
    return loaded


PROGRAMS = module('build_lifecycle_programs')
SBF = module('build_keyless_sbf')


class LifecycleEvidenceTests(unittest.TestCase):
    def completion(self, **changes):
        record = dict(kind='lifecycle_complete', status='PASS', legs=2,
                      pending_sol=0, pending_tokens=0, live_ready=False,
                      signature_verified=False,
                      actual_programs=['PIV1', 'System', 'Token8.0.0',
                                       'SPLpool2.0.3', 'Stake5.1.0'])
        record.update(changes)
        return 'PIV1_LIFECYCLE_EVIDENCE ' + json.dumps(record) + '\n'

    def transcript(self, record):
        return ('test complete_delayed_production_lifecycle ... ' + record
                + 'test result: ok. 1 passed; 0 failed;\n')

    def test_complete_record_requires_actual_test_success(self):
        with self.assertRaises(RuntimeError):
            RUNNER.verify_runtime_output(self.completion())

    def test_success_without_lifecycle_completion_rejects(self):
        with self.assertRaises(RuntimeError):
            RUNNER.verify_runtime_output(self.transcript(''))

    def test_incomplete_or_overclaimed_lifecycle_rejects(self):
        for change in [dict(legs=1), dict(pending_sol=1), dict(pending_tokens=1),
                       dict(live_ready=True), dict(signature_verified=True),
                       dict(actual_programs=['PIV1', 'synthetic Token'])]:
            with self.subTest(change=change), self.assertRaises(RuntimeError):
                RUNNER.verify_runtime_output(self.transcript(self.completion(**change)))

    def test_ambiguous_duplicate_completion_rejects(self):
        with self.assertRaises(RuntimeError):
            RUNNER.verify_runtime_output(self.transcript(self.completion() * 2))

    def test_complete_scoped_evidence_parses(self):
        self.assertEqual(RUNNER.verify_runtime_output(
            self.transcript(self.completion()))['status'], 'PASS')

    def test_artifact_substitution_and_missing_program_reject(self):
        with tempfile.TemporaryDirectory() as directory:
            path = Path(directory) / 'program.so'
            path.write_bytes(b'reviewed bytes')
            entry = dict(path=str(path), bytes=path.stat().st_size,
                         sha256=RUNNER.digest(path))
            pins = dict(artifacts={role: entry.copy()
                                  for role in ['piv', 'caller', 'token', 'pool', 'stake']})
            RUNNER.verify_artifacts(pins)
            pins['artifacts'].pop('stake')
            with self.assertRaises(RuntimeError):
                RUNNER.verify_artifacts(pins)
            pins['artifacts']['stake'] = entry.copy()
            path.write_bytes(b'altered  bytes')
            with self.assertRaises(RuntimeError):
                RUNNER.verify_artifacts(pins)


class CooldownLossEvidenceTests(unittest.TestCase):
    def records(self):
        # Independent literal contract, not copied from the runner's constants.
        completion = dict(kind='cooldown_loss_complete', status='PASS', transactions=90,
            legs=2, finalized_legs=2, epoch=14, loss_lamports=1, recovery_state=4,
            recovery_flags=1, protected_hwm=119_000_000_000, pending_sol=300_000_000,
            pending_tokens=500_000_000, recovered_stake_rent=4_565_760,
            recovered_metadata_rent=5_442_720, temporary_accounts_closed=4,
            beneficiary_paid=0, kif_claimed=0, synthetic_external_loss=True,
            synthetic_loss_conservation=True, live_ready=False, signature_verified=False,
            actual_programs=['PIV1', 'System', 'Token8.0.0', 'SPLpool2.0.3', 'Stake5.1.0'])
        shock = dict(kind='external_cooldown_loss', synthetic=True,
            production_instruction=False, live_slashing_proof=False, lamports=1,
            epoch=0, capitalization_before=1_000_000_000_000,
            capitalization_after=1_000_000_000_000,
            all_data_unchanged=True, all_piv_accounts_unchanged=True)
        return [shock, completion]

    def transcript(self, records, test='cooldown_loss_recovery_production_lifecycle'):
        return ('test ' + test + ' ... ' + '\n'.join(
            'PIV1_LIFECYCLE_EVIDENCE ' + json.dumps(row) for row in records)
            + '\ntest result: ok. 1 passed; 0 failed;\n')

    def test_fixed_profiles_select_exact_tests_and_loss_evidence(self):
        self.assertEqual(RUNNER.parse_args(['bank-build', '--output', '/tmp/unused']).profile, 'positive')
        self.assertEqual(RUNNER.parse_args(['bank-run', '--output', '/tmp/unused', '--profile', 'cooldown-loss']).profile, 'cooldown-loss')
        self.assertEqual(RUNNER.selected_test('positive'), 'complete_delayed_production_lifecycle')
        self.assertEqual(RUNNER.selected_test('cooldown-loss'), 'cooldown_loss_recovery_production_lifecycle')
        self.assertEqual(RUNNER.verify_runtime_output(self.transcript(self.records()),
            'cooldown-loss')['recovery_state'], 4)
        for profile in ['', 'loss', '../cooldown-loss', 'positive --ignored']:
            with self.subTest(profile=profile), self.assertRaises(RuntimeError):
                RUNNER.verify_runtime_output(self.transcript(self.records()), profile)

    def test_cross_profile_missing_duplicate_or_wrong_test_rejects(self):
        records = self.records()
        for transcript in [self.transcript(records, 'complete_delayed_production_lifecycle'),
                           self.transcript(records[:1]), self.transcript(records + records[1:]),
                           self.transcript(records + [dict(kind='lifecycle_complete')]),
                           self.transcript(records + [dict(kind='invented_complete')])]:
            with self.subTest(transcript=transcript), self.assertRaises(RuntimeError):
                RUNNER.verify_runtime_output(transcript, 'cooldown-loss')
        with self.assertRaises(RuntimeError):
            RUNNER.verify_runtime_output(self.transcript(records))
        positive = LifecycleEvidenceTests().completion()
        with self.assertRaises(RuntimeError):
            RUNNER.verify_runtime_output(LifecycleEvidenceTests().transcript(positive +
                'PIV1_LIFECYCLE_EVIDENCE ' + json.dumps(records[0]) + '\n'))

    def test_loss_economic_or_provenance_overclaims_and_wrong_types_reject(self):
        changes = [dict(loss_lamports=0), dict(recovery_state=2), dict(recovery_flags=3),
            dict(finalized_legs=1), dict(protected_hwm=118_999_999_999), dict(pending_sol=0),
            dict(pending_tokens=0), dict(recovered_stake_rent=4_565_759),
            dict(recovered_metadata_rent=0), dict(temporary_accounts_closed=3),
            dict(beneficiary_paid=1), dict(kif_claimed=1), dict(synthetic_external_loss=False),
            dict(synthetic_loss_conservation=False), dict(live_ready=True),
            dict(signature_verified=True), dict(loss_lamports=True), dict(epoch=14.0),
            dict(transactions=True), dict(transactions=0)]
        for change in changes:
            records = self.records(); records[1].update(change)
            with self.subTest(change=change), self.assertRaises(RuntimeError):
                RUNNER.verify_runtime_output(self.transcript(records), 'cooldown-loss')

    def test_external_shock_must_be_unique_balanced_and_explicitly_synthetic(self):
        for change in [dict(synthetic=False), dict(production_instruction=True),
            dict(live_slashing_proof=True), dict(lamports=2), dict(epoch=14),
            dict(capitalization_after=999), dict(capitalization_before=0, capitalization_after=0),
            dict(all_data_unchanged=False), dict(all_piv_accounts_unchanged=False)]:
            records = self.records(); records[0].update(change)
            with self.subTest(change=change), self.assertRaises(RuntimeError):
                RUNNER.verify_runtime_output(self.transcript(records), 'cooldown-loss')
        records = self.records()
        for altered in [records[1:], records + records[:1]]:
            with self.assertRaises(RuntimeError):
                RUNNER.verify_runtime_output(self.transcript(altered), 'cooldown-loss')

    def test_malformed_evidence_rejects(self):
        for bad in [None, [], 42, {}, dict(kind=12)]:
            with self.subTest(bad=bad), self.assertRaises(RuntimeError):
                RUNNER.verify_runtime_output(self.transcript(self.records() + [bad]), 'cooldown-loss')
        with self.assertRaises(ValueError):
            RUNNER.verify_runtime_output('PIV1_LIFECYCLE_EVIDENCE {bad json', 'cooldown-loss')


class IsolatedTargetAuditTests(unittest.TestCase):
    def fingerprint(self, root, prefix='target-token', package='solana-sysvar-id-0123456789abcdef',
                    filename='lib-solana_sysvar_id.json'):
        return root / prefix / SBF.TARGET / 'release' / '.fingerprint' / package / filename

    def test_exact_metadata_allowed_in_each_isolated_target(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            for prefix in ['target-token', 'target-pool']:
                path = self.fingerprint(root, prefix)
                path.parent.mkdir(parents=True)
                path.write_text('{}')
            self.assertEqual(len([p for p in PROGRAMS.audit_outputs(SBF, root)
                                  if p.endswith('.json')]), 2)

    def test_other_prefix_or_metadata_identity_rejects(self):
        for changes in [dict(prefix='target-other'), dict(prefix='nested/target-token'),
                        dict(package='solana-sysvar-id-not-a-hash'), dict(filename='id.json')]:
            with self.subTest(changes=changes), tempfile.TemporaryDirectory() as directory:
                root = Path(directory)
                path = self.fingerprint(root, **changes)
                path.parent.mkdir(parents=True)
                path.write_text('{}')
                with self.assertRaises(RuntimeError):
                    PROGRAMS.audit_outputs(SBF, root)

    def test_directory_or_symlink_cannot_use_metadata_exception(self):
        for symlink in [False, True]:
            with self.subTest(symlink=symlink), tempfile.TemporaryDirectory() as directory:
                root = Path(directory)
                path = self.fingerprint(root)
                path.parent.mkdir(parents=True)
                if symlink:
                    path.symlink_to(root / 'does-not-exist')
                else:
                    path.mkdir()
                with self.assertRaises(RuntimeError):
                    PROGRAMS.audit_outputs(SBF, root)


if __name__ == '__main__':
    unittest.main()
