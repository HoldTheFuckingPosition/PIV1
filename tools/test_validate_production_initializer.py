"""Tampering and diagnostic regressions; no compiler/runtime/live invocation."""
import importlib.util
import json
from pathlib import Path
import tempfile
import unittest
from unittest.mock import patch

spec = importlib.util.spec_from_file_location('production_runner', Path(__file__).with_name('validate_production_initializer.py'))
r = importlib.util.module_from_spec(spec)
spec.loader.exec_module(r)


class RunnerTests(unittest.TestCase):
    def setUp(self):
        self.temp = tempfile.TemporaryDirectory()
        self.addCleanup(self.temp.cleanup)
        self.base = Path(self.temp.name)
        self.source = self.base / 'source.rs'; self.source.write_text('reviewed source')
        self.old = self.base / 'old'; self.old.mkdir()
        (self.old / 'target').mkdir(); (self.old / 'openssl-config').mkdir()
        self.artifact = self.base / 'historical'; self.artifact.write_bytes(b'historical binary')
        self.pins = {'sources': {'source.rs': r.digest(self.source)}, 'helpers': {},
            'preserved': [{'path': str(self.artifact), 'sha256': r.digest(self.artifact), 'bytes': self.artifact.stat().st_size}]}
        for name, value in [('ROOT', self.base), ('OLD_BUILD', self.old), ('CACHE', self.old / 'target')]:
            context = patch.object(r, name, value); context.start(); self.addCleanup(context.stop)

    def test_source_change_blocks_validation(self):
        r.verify(self.pins)
        self.source.write_text('changed source')
        with self.assertRaisesRegex(RuntimeError, 'source drift'): r.verify(self.pins)

    def test_historical_binary_change_blocks_validation(self):
        self.artifact.write_bytes(b'changed historical binary')
        with self.assertRaisesRegex(RuntimeError, 'historical evidence drift'): r.verify(self.pins)

    def test_artifact_symlink_is_refused(self):
        alias = self.base / 'alias'; alias.symlink_to(self.artifact)
        self.pins['preserved'][0]['path'] = str(alias)
        with self.assertRaises(RuntimeError): r.verify(self.pins)

    def test_cache_symlink_and_nonempty_config_are_refused(self):
        (self.old / 'target').rmdir(); (self.old / 'target').symlink_to(self.base)
        with self.assertRaisesRegex(RuntimeError, 'cache/config'): r.verify(self.pins)
        (self.old / 'target').unlink(); (self.old / 'target').mkdir()
        (self.old / 'openssl-config/unexpected.cnf').write_text('not used')
        with self.assertRaisesRegex(RuntimeError, 'must remain empty'): r.verify(self.pins)

    def test_final_tool_warning_rejects_even_successful_exit(self):
        log = self.base / 'disassembly-caller.stderr'; log.write_text('warning: malformed relocation\n')
        with self.assertRaisesRegex(RuntimeError, 'diagnostics'): r.strict_diagnostics(self.base)
        log.write_text(''); r.strict_diagnostics(self.base)

    def test_reviewed_result_requires_matching_input_identity_and_status(self):
        path = self.base / 'result.json'
        value = {'status': 'BUILD_PASS', 'pins_sha256': 'reviewed', 'runner_sha256': r.digest(r.SCRIPT)}
        path.write_text(json.dumps(value)); self.assertEqual(r.reviewed_result(path, 'reviewed', 'BUILD_PASS'), value)
        for name in value:
            changed = {**value, name: 'unreviewed'}; path.write_text(json.dumps(changed))
            with self.assertRaises(RuntimeError): r.reviewed_result(path, 'reviewed', 'BUILD_PASS')

    def test_sbf_result_replacement_requires_new_review_hash(self):
        path = self.base / 'sbf-result.json'
        value = {'status': 'SBF_PASS', 'pins_sha256': 'reviewed', 'runner_sha256': r.digest(r.SCRIPT)}
        path.write_text(json.dumps(value)); approved = r.digest(path)
        r.reviewed_sbf(path, 'reviewed', approved)
        path.write_text(json.dumps({**value, 'artifacts': {'callee': 'different'}}))
        with self.assertRaisesRegex(RuntimeError, 'unreviewed SBF'): r.reviewed_sbf(path, 'reviewed', approved)

    def test_zero_filtered_tests_and_incomplete_runtime_evidence_are_refused(self):
        for text in ['test result: ok. 0 passed; 0 failed;', 'test result: ok. 1 passed; 0 failed;']:
            with self.assertRaises(RuntimeError): r.verify_runtime_output(text)
        complete = {'kind': 'complete', 'status': 'PASS', 'profiles': 4, 'message_cases': 12, 'successful_initializations': 4}
        text = f'test {r.TEST} ...\nPIV1_BANK_GENESIS_EVIDENCE {json.dumps(complete)}\ntest result: ok. 1 passed; 0 failed;'
        r.verify_runtime_output(text)
        with self.assertRaises(RuntimeError): r.verify_runtime_output(text.replace('"profiles": 4', '"profiles": 3'))

    def test_guarded_compile_uses_only_fixed_reviewed_cache(self):
        command = r.cached_command(['/fixed/cargo', 'test', '--locked', '--offline'])
        self.assertEqual(command, ['/usr/bin/env', f'CARGO_TARGET_DIR={r.CACHE}',
            f'OPENSSL_CONFIG_DIR={r.OLD_BUILD / "openssl-config"}', '/fixed/cargo', 'test', '--locked', '--offline'])


if __name__ == '__main__': unittest.main()
