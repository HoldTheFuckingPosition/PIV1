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
