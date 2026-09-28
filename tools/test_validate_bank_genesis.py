#!/usr/bin/python3
"""Artifact, diagnostic and reviewed-build boundaries; no Bank execution."""
import importlib.util
import json
from pathlib import Path
import shutil
import unittest
import uuid
from unittest.mock import patch

spec = importlib.util.spec_from_file_location('genesis_runner',
    Path(__file__).with_name('validate_bank_genesis.py'))
runner = importlib.util.module_from_spec(spec)
spec.loader.exec_module(runner)


class GenesisRunnerBoundaries(unittest.TestCase):
    def setUp(self):
        # tempfile random suffixes may contain underscores, which the actual
        # runner intentionally refuses. Use its permitted alphabet explicitly.
        self.root = Path('/tmp') / ('piv1-bank-smoke-test-' + uuid.uuid4().hex)
        self.root.mkdir(mode=0o700)
        self.addCleanup(shutil.rmtree, self.root)
        self.helper = runner.load_helper()
        self.artifact = self.root / 'fixture.so'
        self.artifact.write_bytes(b'public fixture ELF placeholder; never executed')
        entry = {'path': str(self.artifact), 'sha256': runner.digest(self.artifact),
                 'bytes': self.artifact.stat().st_size}
        self.pins = {'artifacts': {name: dict(entry) for name in runner.ARTIFACT_NAMES},
                     'historical_artifacts': [dict(entry)],
                     'tools': {str(runner.ENV): runner.digest(runner.ENV)}}

    def test_helper_drift_refused_before_import(self):
        with patch.object(runner, 'HELPER', self.artifact):
            with self.assertRaisesRegex(RuntimeError, 'helper drift'):
                runner.load_helper()

    def test_artifact_set_must_be_exact(self):
        self.pins['artifacts']['unexpected'] = self.pins['artifacts']['caller']
        with self.assertRaisesRegex(RuntimeError, 'artifact set'):
            runner.verify_artifacts(self.pins)

    def test_missing_artifact_refused(self):
        self.pins['artifacts']['caller']['path'] += '-missing'
        with self.assertRaises(FileNotFoundError):
            runner.verify_artifacts(self.pins)

    def test_artifact_content_drift_refused(self):
        runner.verify_artifacts(self.pins)
        self.artifact.write_bytes(b'changed')
        with self.assertRaisesRegex(RuntimeError, 'artifact drift'):
            runner.verify_artifacts(self.pins)

    def test_historical_artifact_size_drift_refused(self):
        self.pins['historical_artifacts'][0]['bytes'] += 1
        with self.assertRaisesRegex(RuntimeError, 'artifact drift'):
            runner.verify_artifacts(self.pins)

    def test_artifact_symlink_refused(self):
        link = self.root / 'link.so'
        link.symlink_to(self.artifact)
        self.pins['artifacts']['caller']['path'] = str(link)
        with self.assertRaisesRegex(RuntimeError, 'artifact path alias'):
            runner.verify_artifacts(self.pins)

    def test_exact_public_environment_wrapper(self):
        binary = self.root / 'reviewed-binary'
        command = runner.runtime_command(binary, self.pins)
        self.assertEqual(command[0], '/usr/bin/env')
        self.assertEqual(command[-3:], [str(binary), '--test-threads=1', '--nocapture'])
        variables = dict(item.split('=', 1) for item in command[1:-3])
        self.assertEqual(set(variables), {f'PIV_GENESIS_{name}_{field}'
            for name in ['CALLER', 'CALLEE', 'TOKEN'] for field in ['PATH', 'SHA256', 'BYTES']})
        self.assertEqual(variables['PIV_GENESIS_CALLER_PATH'], str(self.artifact))
        self.assertEqual(variables['PIV_GENESIS_TOKEN_BYTES'], str(self.artifact.stat().st_size))

    def test_environment_wrapper_drift_refused(self):
        self.pins['tools'][str(runner.ENV)] = '0' * 64
        with self.assertRaisesRegex(RuntimeError, 'wrapper drift'):
            runner.runtime_command(self.root / 'binary', self.pins)

    def test_build_is_offline_locked_no_run_single_target(self):
        metadata, build = runner.build_commands(self.helper)
        for command in [metadata, build]:
            self.assertIn('--locked', command)
            self.assertIn('--offline', command)
        self.assertEqual(build[build.index('--test') + 1], 'genesis_bank')
        self.assertEqual(build[build.index('--jobs') + 1], '1')
        self.assertIn('--no-run', build)

    def build_fixture(self):
        target = self.root / 'target'
        target.mkdir()
        binary = target / 'genesis_bank-fixture'
        binary.write_bytes(b'never executable')
        message = {'reason': 'compiler-artifact', 'target': {'name': 'genesis_bank'},
                   'executable': str(binary)}
        (self.root / 'build.stdout').write_text(json.dumps(message) + '\n')
        (self.root / 'build.stderr').write_text('')
        return binary

    def test_strict_build_accepts_one_matching_artifact(self):
        binary = self.build_fixture()
        self.assertEqual(runner.inspect_build(self.helper, self.root), binary)

    def test_raw_future_compat_warning_refused(self):
        self.build_fixture()
        (self.root / 'build.stderr').write_text('warning: future incompatibility\n')
        with self.assertRaisesRegex(RuntimeError, 'diagnostics'):
            runner.inspect_build(self.helper, self.root)
        self.assertTrue((self.root / 'diagnostics.json').is_file())

    def test_structured_warning_refused(self):
        self.build_fixture()
        with (self.root / 'build.stdout').open('a') as stream:
            stream.write(json.dumps({'reason': 'compiler-message', 'message': {'level': 'warning'}}) + '\n')
        with self.assertRaisesRegex(RuntimeError, 'diagnostics'):
            runner.inspect_build(self.helper, self.root)

    def test_wrong_target_refused(self):
        self.build_fixture()
        path = self.root / 'build.stdout'
        path.write_text(path.read_text().replace('"name": "genesis_bank"', '"name": "bank"'))
        with self.assertRaisesRegex(RuntimeError, 'expected one'):
            runner.inspect_build(self.helper, self.root)

    def test_review_binding_refuses_stale_source_pins_and_binary(self):
        binary = self.build_fixture()
        pins_file = self.root / 'pins.json'
        pins_file.write_text('{}')
        record = {'status': 'BUILD_PASS', 'runner_sha256': runner.digest(runner.SCRIPT),
                  'pins_sha256': runner.digest(pins_file), 'helper_sha256': runner.HELPER_SHA256,
                  'binary': {'path': str(binary), 'sha256': runner.digest(binary)}}
        receipt = self.root / 'result.json'
        receipt.write_text(json.dumps(record))
        approved = runner.digest(binary)
        with patch.object(runner, 'PINS', pins_file):
            self.assertEqual(runner.reviewed_binary(self.helper, self.root, approved), binary)
            pins_file.write_text('{"changed":true}')
            with self.assertRaisesRegex(RuntimeError, 'input/status'):
                runner.reviewed_binary(self.helper, self.root, approved)
            pins_file.write_text('{}')
            binary.write_bytes(b'changed binary')
            with self.assertRaisesRegex(RuntimeError, 'unreviewed'):
                runner.reviewed_binary(self.helper, self.root, approved)

    def test_review_requires_explicit_hash(self):
        for approved in [None, '', 'f' * 63, 'unreviewed']:
            with self.subTest(approved=approved), self.assertRaisesRegex(RuntimeError, 'reviewed build/hash'):
                runner.reviewed_binary(self.helper, self.root, approved)


if __name__ == '__main__':
    unittest.main(verbosity=2)
