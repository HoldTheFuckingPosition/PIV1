#!/usr/bin/python3
"""Synthetic metadata regressions; never run a compiler, Bank or network command."""
import argparse
from contextlib import redirect_stderr, redirect_stdout
import errno
import importlib.util
import io
import json
from pathlib import Path
import tempfile
from types import SimpleNamespace
import unittest
from unittest.mock import patch

spec = importlib.util.spec_from_file_location(
    'bank_prerequisites', Path(__file__).with_name('check_bank_runtime_prerequisites.py'))
checker = importlib.util.module_from_spec(spec)
spec.loader.exec_module(checker)


class Prerequisites(unittest.TestCase):
    def setUp(self):
        self.directory = tempfile.TemporaryDirectory(prefix='piv1-bank-prerequisite-unit-')
        self.addCleanup(self.directory.cleanup)
        self.root = Path(self.directory.name)
        self.cache = self.root / 'public-cache'
        self.output = self.root / 'output-parent'
        self.output.mkdir()
        self.source = self.cache / 'registry/src' / checker.REGISTRY
        self.archive = self.cache / 'registry/cache' / checker.REGISTRY

    def populate(self, omit=None):
        self.source.mkdir(parents=True)
        self.archive.mkdir(parents=True)
        for name in (*checker.PACKAGES, 'solana-program-runtime'):
            if omit != ('source', name):
                (self.source / f'{name}-4.2.0').mkdir()
            if name in checker.PACKAGES and omit != ('archive', name):
                (self.archive / f'{name}-4.2.0.crate').write_bytes(b'public synthetic fixture; not a real crate')

    def inspect(self, minimum=4096, available_blocks=4):
        with patch.object(checker.os, 'statvfs', return_value=SimpleNamespace(
                f_bavail=available_blocks, f_frsize=1024, f_bfree=1_000_000)):
            return checker.check(self.cache, self.output, minimum)

    def test_synthetic_presence_and_exact_space_minimum_pass_without_provenance_claim(self):
        self.populate()
        report = self.inspect()
        self.assertTrue(report['ready'])
        self.assertEqual(report['available_bytes'], 4096)
        self.assertFalse(report['candidate_version_authenticated'])
        self.assertIn('not a complete resolved dependency closure', ' '.join(report['limitations']))

    def test_low_available_space_rejects_despite_large_free_block_count(self):
        self.populate()
        report = self.inspect(available_blocks=3)
        self.assertFalse(report['ready'])
        self.assertEqual(report['available_bytes'], 3072)
        self.assertEqual(report['space']['reason'], 'below_selected_planning_minimum')

    def test_missing_direct_source_is_not_ready(self):
        self.populate(('source', 'solana-runtime'))
        report = self.inspect()
        self.assertFalse(report['ready'])
        self.assertEqual(report['packages'][0]['source']['status'], 'missing')

    def test_missing_direct_archive_is_not_ready(self):
        self.populate(('archive', 'solana-accounts-db'))
        report = self.inspect()
        self.assertFalse(report['ready'])
        self.assertEqual(report['packages'][1]['archive']['status'], 'missing')

    def test_absent_cache_reports_all_missing_packages(self):
        report = self.inspect()
        self.assertFalse(report['ready'])
        for package in report['packages']:
            for role in ('source', 'archive'):
                self.assertEqual(package[role]['status'], 'missing')

    def test_missing_modular_runtime_source_rejects(self):
        self.populate(('source', 'solana-program-runtime'))
        report = self.inspect()
        self.assertFalse(report['ready'])
        self.assertEqual(report['modular_runtime_source']['status'], 'missing')

    def test_source_symlink_rejects_without_reading_target(self):
        self.populate(('source', 'solana-svm'))
        (self.source / 'solana-svm-4.2.0').symlink_to('/not-read-or-followed')
        report = self.inspect()
        self.assertFalse(report['ready'])
        self.assertEqual(report['packages'][2]['source']['reason'], 'symlink_component')

    def test_archive_symlink_rejects_without_reading_target(self):
        self.populate(('archive', 'solana-runtime'))
        (self.archive / 'solana-runtime-4.2.0.crate').symlink_to('/not-read-or-followed')
        report = self.inspect()
        self.assertFalse(report['ready'])
        self.assertEqual(report['packages'][0]['archive']['reason'], 'symlink_component')

    def test_cache_root_symlink_rejects_all_children(self):
        self.cache.symlink_to('/not-read-or-followed')
        report = self.inspect()
        self.assertFalse(report['ready'])
        for package in report['packages']:
            self.assertEqual(package['source']['component'], str(self.cache))
            self.assertEqual(package['archive']['reason'], 'symlink_component')

    def test_intermediate_cache_symlink_rejects(self):
        self.cache.mkdir()
        (self.cache / 'registry').symlink_to('/not-read-or-followed')
        report = self.inspect()
        self.assertFalse(report['ready'])
        self.assertEqual(report['packages'][0]['source']['component'], str(self.cache / 'registry'))

    def test_output_parent_symlink_and_symlinked_ancestor_reject_before_space_query(self):
        self.populate()
        link = self.root / 'linked-output'
        link.symlink_to(self.output, target_is_directory=True)
        for parent in (link, link / 'nested'):
            with self.subTest(parent=parent), patch.object(checker.os, 'statvfs') as statvfs:
                report = checker.check(self.cache, parent, 4096)
                self.assertFalse(report['ready'])
                self.assertEqual(report['output_parent']['reason'], 'symlink_component')
                statvfs.assert_not_called()

    def test_nonregular_archive_and_nondirectory_source_reject(self):
        self.populate(('archive', 'solana-runtime'))
        (self.archive / 'solana-runtime-4.2.0.crate').mkdir()
        report = self.inspect()
        self.assertFalse(report['ready'])
        self.assertEqual(report['packages'][0]['archive']['reason'], 'wrong_component_type')
        self.assertEqual(checker.observe(self.archive / 'solana-svm-4.2.0.crate', 'directory')['reason'],
                         'wrong_component_type')

    def test_unavailable_component_metadata_is_reported(self):
        self.populate()
        original = Path.lstat
        unavailable = self.source / 'solana-runtime-4.2.0'
        def guarded_lstat(path):
            if path == unavailable:
                raise PermissionError(errno.EACCES, 'synthetic denial')
            return original(path)
        with patch.object(Path, 'lstat', guarded_lstat):
            report = self.inspect()
        self.assertFalse(report['ready'])
        self.assertEqual(report['packages'][0]['source']['status'], 'unavailable')
        self.assertEqual(report['packages'][0]['source']['errno'], errno.EACCES)

    def test_unavailable_space_metadata_is_reported(self):
        self.populate()
        with patch.object(checker.os, 'statvfs', side_effect=OSError(errno.EIO, 'synthetic failure')):
            report = checker.check(self.cache, self.output, 4096)
        self.assertFalse(report['ready'])
        self.assertIsNone(report['available_bytes'])
        self.assertEqual(report['space']['reason'], 'space_metadata_unavailable')

    def test_invalid_space_metadata_rejects(self):
        self.populate()
        for blocks, size in ((-1, 1024), (4, 0)):
            with self.subTest(blocks=blocks, size=size), patch.object(checker.os, 'statvfs',
                    return_value=SimpleNamespace(f_bavail=blocks, f_frsize=size)):
                self.assertEqual(checker.check(self.cache, self.output, 4096)['space']['reason'],
                                 'invalid_space_metadata')

    def test_missing_relative_or_parent_traversal_output_rejects(self):
        self.populate()
        for parent, expected in ((self.root / 'missing', 'missing'), (Path('relative'), 'unsafe'),
                                 (self.output / '..', 'unsafe')):
            with self.subTest(parent=parent), patch.object(checker.os, 'statvfs') as statvfs:
                report = checker.check(self.cache, parent, 4096)
                self.assertFalse(report['ready'])
                self.assertEqual(report['output_parent']['status'], expected)
                statvfs.assert_not_called()

    def test_malformed_or_absent_budget_is_usage_error(self):
        for value in ('0', '-1', '1.5', '1e9', ' 1', '+1', '01', ''):
            with self.subTest(value=value), self.assertRaises(argparse.ArgumentTypeError):
                checker.positive_decimal(value)
        self.assertEqual(checker.positive_decimal('4096'), 4096)
        for value in (0, -1, True, '4096', 1.5):
            with self.subTest(value=value), self.assertRaises(ValueError):
                checker.check(self.cache, self.output, value)

    def test_cli_ready_and_not_ready_are_structured_without_command_execution(self):
        self.populate()
        for blocks, code in ((4, 0), (3, 2)):
            with self.subTest(blocks=blocks), patch.object(checker, 'CACHE_ROOT', self.cache), \
                    patch.object(checker.os, 'statvfs', return_value=SimpleNamespace(f_bavail=blocks, f_frsize=1024)), \
                    redirect_stdout(io.StringIO()) as output:
                actual = checker.main(['--output-parent', str(self.output), '--minimum-free-bytes', '4096'])
                self.assertEqual(actual, code)
                self.assertEqual(json.loads(output.getvalue())['ready'], code == 0)

    def test_cli_missing_or_malformed_budget_is_distinct_usage_exit(self):
        for arguments in ([], ['--output-parent', str(self.output)],
                          ['--output-parent', str(self.output), '--minimum-free-bytes', '0'],
                          ['--output-parent', str(self.output), '--minimum-free-bytes', '9' * 5000]):
            with self.subTest(arguments=arguments[:2]), redirect_stderr(io.StringIO()) as error, \
                    redirect_stdout(io.StringIO()) as output, patch.object(checker, 'check') as check, \
                    self.assertRaises(SystemExit) as exited:
                checker.main(arguments)
            self.assertEqual(exited.exception.code, 64)
            self.assertEqual(output.getvalue(), '')
            self.assertIn('error:', error.getvalue())
            check.assert_not_called()


if __name__ == '__main__':
    unittest.main()
